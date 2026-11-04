//! Zero-JS web tier (ADR-0002): Axum 0.7 HTTP server rendering the workspace
//! UI server-side.
//!
//! Hard rules honoured here:
//! - 表現層零 JavaScript：the only scripts are none; interaction is form POST +
//!   303 PRG + CSS `:target`.
//! - GET never mutates state; all writes go through POST handlers that enforce
//!   RBAC (`LEDGER_WRITE`) server-side.
//! - Money is parsed with exact string arithmetic into i64 minor units
//!   (INV-01, `flaps_core::money`), never through `f64`.
//! - All rendered values pass through Askama auto-escaping (XSS defense,
//!   THREAT_MODEL T-07).

use askama::Template;
use axum::extract::{Form, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use flaps_core::permission::PermissionManager;
use flaps_core::{
    format_minor, parse_decimal_to_minor, LedgerEngine, Transaction, TransactionItem,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::auth::{self, Session};
use crate::db::DatabaseManager;

/// Maximum lengths accepted from the form (fail-closed input validation).
const MAX_DESCRIPTION_LEN: usize = 200;
const MAX_ACCOUNT_ID_LEN: usize = 64;
const SUPPORTED_CURRENCIES: [&str; 4] = ["TWD", "USD", "EUR", "JPY"];
const RECENT_TRANSACTION_LIMIT: i64 = 20;

/// Shared application state for the web tier.
#[derive(Clone)]
pub struct AppState {
    pub(crate) pool: SqlitePool,
    engine: LedgerEngine,
    permissions: PermissionManager,
}

impl AppState {
    /// Assemble state over a pool whose migrations have already been applied
    /// (via [`DatabaseManager::initialize`]).
    pub async fn new(pool: SqlitePool) -> Result<Self, Box<dyn std::error::Error>> {
        DatabaseManager::new(pool.clone()).initialize().await?;
        let permissions = PermissionManager::new(pool.clone()).await;
        let engine = LedgerEngine::with_pool(pool.clone()).await?;
        Ok(Self {
            pool,
            engine,
            permissions,
        })
    }
}

/// Render model for a transaction row joined with its item totals.
#[derive(Debug, Clone)]
pub struct TransactionRow {
    pub transaction_code: String,
    pub description: String,
    pub transaction_date: String,
    pub amount_display: String,
    pub currency: String,
    pub status: String,
}

/// Values echoed back into the form after a failed submission (PRG failure
/// path): the user never retypes what they already entered. `Default` picks
/// sensible UI defaults (TWD) so a fresh form shows the common case.
#[derive(Debug, Clone)]
pub struct FormValues {
    pub debit_account: String,
    pub credit_account: String,
    pub amount: String,
    pub currency: String,
    pub description: String,
    pub transaction_date: String,
}

impl Default for FormValues {
    fn default() -> Self {
        Self {
            debit_account: String::new(),
            credit_account: String::new(),
            amount: String::new(),
            currency: "TWD".to_string(),
            description: String::new(),
            transaction_date: String::new(),
        }
    }
}

impl FormValues {
    fn from_form(form: &TransactionForm) -> Self {
        Self {
            debit_account: form.debit_account.clone(),
            credit_account: form.credit_account.clone(),
            amount: form.amount.clone(),
            currency: form.currency.clone(),
            description: form.description.clone(),
            transaction_date: form.transaction_date.clone(),
        }
    }
}

/// Server-rendered index page (template: `platform/templates/index.html`).
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub db_connected: bool,
    pub transaction_count: String,
    pub total_amount_display: String,
    pub account_count: String,
    pub transactions: Vec<TransactionRow>,
    pub accounts: Vec<String>,
    pub error: Option<String>,
    pub form: FormValues,
}

/// Posted form for creating a balanced journal entry: one debit leg and one
/// credit leg of the same amount (the minimal double-entry UI). `amount` is
/// the exact decimal string typed by the user; conversion to minor units
/// happens server-side. `csrf_token` must match the acting session (T-08).
#[derive(Debug, Deserialize)]
pub struct TransactionForm {
    pub csrf_token: String,
    pub debit_account: String,
    pub credit_account: String,
    pub amount: String,
    pub currency: String,
    pub description: String,
    pub transaction_date: String,
    #[serde(default)]
    pub ai_model: String,
}

/// Build the application router. Exposed for tests and the server binary.
///
/// Route split (ADR-0003):
/// - public: `/login` (GET+POST), `/logout` (CSRF-checked), embedded CSS,
///   `/api/health`
/// - protected (session middleware injects `Session`): `/`, `POST /transactions`
pub fn router(state: AppState) -> Router {
    let public = Router::new()
        .route("/login", get(auth::get_login).post(auth::post_login))
        .route("/logout", post(auth::post_logout))
        .route("/assets/design-system.css", get(design_system_css))
        .route("/api/health", get(health));

    let protected = Router::new()
        .route("/", get(index))
        .route("/transactions", post(create_transaction))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::require_session,
        ));

    public.merge(protected).with_state(state)
}

async fn index(State(state): State<AppState>, session: Extension<Session>) -> Response {
    let html = build_index(&state, &session, None, FormValues::default()).await;
    match html {
        Ok(body) => Html(body).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("頁面渲染失敗（可能原因：模板錯誤）；請聯絡管理員。詳情：{e}"),
        )
            .into_response(),
    }
}

/// The design system stylesheet is embedded at compile time: zero path
/// traversal surface, zero runtime file I/O, deterministic deployment.
const DESIGN_SYSTEM_CSS: &str = include_str!("../../web/assets/design-system.css");

async fn design_system_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        DESIGN_SYSTEM_CSS,
    )
}

async fn health(State(state): State<AppState>) -> Response {
    let db_connected = sqlx::query("SELECT 1").fetch_one(&state.pool).await.is_ok();
    Json(serde_json::json!({
        "status": "ok",
        "db": db_connected,
        "service": "flaps-server",
        "version": env!("CARGO_PKG_VERSION"),
    }))
    .into_response()
}

async fn create_transaction(
    State(state): State<AppState>,
    session: Extension<Session>,
    Form(form): Form<TransactionForm>,
) -> Response {
    // CSRF (T-08): the form token must match the acting session's token.
    if form.csrf_token != session.csrf_token {
        return (
            StatusCode::FORBIDDEN,
            Html(
                render_csrf_failure(&session)
                    .unwrap_or_else(|_| "CSRF 驗證失敗，請返回重新操作。".to_string()),
            ),
        )
            .into_response();
    }

    let saved_form = FormValues::from_form(&form);

    match validate_and_post(&state, &form, &session).await {
        // Redirect::to uses 303 See Other — the correct PRG verb after POST.
        Ok(()) => Redirect::to("/#transactions").into_response(),
        Err((status, message)) => {
            let html = build_index(&state, &session, Some(message), saved_form).await;
            match html {
                Ok(body) => (status, Html(body)).into_response(),
                Err(e) => (status, format!("錯誤頁面渲染失敗：{e}")).into_response(),
            }
        }
    }
}

/// Render the error page on a CSRF failure (session context preserved so the
/// operator can retry from the form without losing the page).
fn render_csrf_failure(session: &Session) -> Result<String, askama::Error> {
    build_index_for_csrf(session)
}

fn build_index_for_csrf(session: &Session) -> Result<String, askama::Error> {
    IndexTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        db_connected: true,
        transaction_count: String::new(),
        total_amount_display: String::new(),
        account_count: String::new(),
        transactions: Vec::new(),
        accounts: Vec::new(),
        error: Some(
            "CSRF 驗證失敗（問題：表單權杖與工作階段不符；可能原因：頁面停留過久或重複開啟；處理方式：重新整理頁面後再送出一次）。"
                .to_string(),
        ),
        form: FormValues::default(),
    }
    .render()
}

/// Validate the form and post the journal entry. Returns the HTTP status and
/// user-facing message on failure (problem + reason + next step, FR-004).
async fn validate_and_post(
    state: &AppState,
    form: &TransactionForm,
    session: &Session,
) -> Result<(), (StatusCode, String)> {
    let acting_user = &session.user_id;
    // 1) RBAC (FR-006): permission by code, server-side, always.
    if !state
        .permissions
        .has_permission(acting_user, "LEDGER_WRITE")
    {
        return Err((
            StatusCode::FORBIDDEN,
            "您沒有建立交易的權限（問題：缺少 LEDGER_WRITE 權限；可能原因：角色未授權；處理方式：請聯絡系統管理員指派權限）。".to_string(),
        ));
    }

    // 2) The two legs of the balanced entry (P1-5): debit account and credit
    //    account, both non-empty and distinct (self-transfer is meaningless).
    let validate_account = |raw: &str, label: &str| -> Result<String, (StatusCode, String)> {
        let id = raw.trim();
        if id.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("{label}為必填欄位，請輸入或選擇科目代碼。"),
            ));
        }
        if id.len() > MAX_ACCOUNT_ID_LEN {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("科目代碼過長（最多 {MAX_ACCOUNT_ID_LEN} 字元）；請縮短後重新送出。"),
            ));
        }
        Ok(id.to_string())
    };
    let debit_account = validate_account(&form.debit_account, "借方科目")?;
    let credit_account = validate_account(&form.credit_account, "貸方科目")?;
    if debit_account.eq_ignore_ascii_case(&credit_account) {
        return Err((
            StatusCode::BAD_REQUEST,
            "借方與貸方不得為同一科目；複式記帳需要兩個不同科目。".to_string(),
        ));
    }

    // 3) Amount: exact decimal → i64 minor units (INV-01). Never f64.
    let amount_minor = parse_decimal_to_minor(&form.amount, 2)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    if amount_minor <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "金額必須大於 0；請輸入正數金額。".to_string(),
        ));
    }
    if amount_minor > flaps_core::ledger::engine::MAX_AMOUNT_MINOR {
        return Err((
            StatusCode::BAD_REQUEST,
            "金額超出上限（最多 15 位整數＋2 位小數）；請拆分交易。".to_string(),
        ));
    }

    // 4) Posting sides are structural (DEBIT/credit legs built below); the
    //    engine re-verifies balance and item types.

    // 5) Currency whitelist.
    if !SUPPORTED_CURRENCIES.contains(&form.currency.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "幣別不支援（{}）；請從清單選擇。",
                SUPPORTED_CURRENCIES.join("、")
            ),
        ));
    }

    // 6) Description (trimmed, bounded).
    let description = form.description.trim();
    if description.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "摘要為必填欄位，請輸入交易說明。".to_string(),
        ));
    }
    if description.len() > MAX_DESCRIPTION_LEN {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("摘要過長（最多 {MAX_DESCRIPTION_LEN} 字元）；請縮短後重新送出。"),
        ));
    }

    // 7) Date: parseable and not in the future (FR-005).
    let transaction_date =
        chrono::NaiveDate::parse_from_str(form.transaction_date.trim(), "%Y-%m-%d").map_err(
            |_| {
                (
                    StatusCode::BAD_REQUEST,
                    "交易日期格式不正確；請以日期選擇器或 YYYY-MM-DD 重新輸入。".to_string(),
                )
            },
        )?;
    if transaction_date > chrono::Utc::now().date_naive() {
        return Err((
            StatusCode::BAD_REQUEST,
            "交易日期不得為未來日期；請修正後重新送出。".to_string(),
        ));
    }

    // 8) Post through the domain engine (idempotent UUID, audited, atomic).
    //    Two legs are constructed here (P1-5): DEBIT on the debit account and
    //    CREDIT on the credit account, same amount and currency — balance is
    //    guaranteed by construction and re-verified by the engine.
    let transaction_id = uuid::Uuid::new_v4().to_string();
    let transaction_code = format!("TX-{}", &transaction_id[..8]);
    let transaction = Transaction::new(
        transaction_id.clone(),
        transaction_code,
        transaction_date,
        description.to_string(),
        None,
        "POSTED".to_string(),
        Some("web".to_string()),
        acting_user.to_string(),
    );
    let items = vec![
        TransactionItem::new(
            uuid::Uuid::new_v4().to_string(),
            transaction_id.clone(),
            debit_account,
            "DEBIT".to_string(),
            amount_minor,
            form.currency.clone(),
            10_000, // 1.0000 scaled by 10^4
            None,
            None,
        ),
        TransactionItem::new(
            uuid::Uuid::new_v4().to_string(),
            transaction_id,
            credit_account,
            "CREDIT".to_string(),
            amount_minor,
            form.currency.clone(),
            10_000,
            None,
            None,
        ),
    ];
    state
        .engine
        .post_journal_entry(transaction, items)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "交易儲存失敗（可能原因：資料庫忙碌或磁碟錯誤）；請稍後重試，若持續失敗請聯絡管理員。詳情：{e}"
                ),
            )
        })
}

/// Query the data needed by the page and render the template. Any database
/// failure degrades to the disconnected layout (db_connected = false, FR-004)
/// instead of hiding the whole UI.
async fn build_index(
    state: &AppState,
    session: &Session,
    error: Option<String>,
    form: FormValues,
) -> Result<String, askama::Error> {
    let db_connected = sqlx::query("SELECT 1").fetch_one(&state.pool).await.is_ok();

    let (transactions, accounts, transaction_count, account_count, total_amount_display) =
        if db_connected {
            let summaries = state
                .engine
                .query_recent_transaction_summaries(RECENT_TRANSACTION_LIMIT)
                .await
                .unwrap_or_default();
            let rows: Vec<TransactionRow> = summaries
                .iter()
                .map(|s| TransactionRow {
                    transaction_code: s.transaction.transaction_code.clone(),
                    description: s.transaction.description.clone(),
                    transaction_date: s.transaction.transaction_date.to_string(),
                    amount_display: format_minor(s.total_amount_minor, 2),
                    currency: s.currency.clone(),
                    status: s.transaction.status.clone(),
                })
                .collect();

            let transaction_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions")
                .fetch_one(&state.pool)
                .await
                .unwrap_or(0);
            let account_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts")
                .fetch_one(&state.pool)
                .await
                .unwrap_or(0);
            let total_minor: i64 =
                sqlx::query_scalar("SELECT COALESCE(SUM(amount_minor), 0) FROM transaction_items")
                    .fetch_one(&state.pool)
                    .await
                    .unwrap_or(0);

            let account_ids: Vec<String> =
                sqlx::query_scalar("SELECT account_id FROM accounts ORDER BY account_id LIMIT 100")
                    .fetch_all(&state.pool)
                    .await
                    .unwrap_or_default();

            (
                rows,
                account_ids,
                transaction_count.to_string(),
                account_count.to_string(),
                format_minor(total_minor, 2),
            )
        } else {
            (
                Vec::new(),
                Vec::new(),
                "—".to_string(),
                "—".to_string(),
                "—".to_string(),
            )
        };

    // "—" placeholders are only shown when the database is unreachable; the
    // template switches the KPI block to literals in that case, so the count
    // strings above are for the connected path.
    let _ = (&transaction_count, &account_count, &total_amount_display);
    IndexTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        db_connected,
        transaction_count,
        total_amount_display,
        account_count,
        transactions,
        accounts,
        error,
        form,
    }
    .render()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;

    const TEST_ADMIN_PASSWORD: &str = "test-admin-pass";
    const TEST_STORE_PASSWORD: &str = "test-store-pass";

    async fn in_memory_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite pool")
    }

    async fn seed_credentials(pool: &SqlitePool) {
        let admin_hash = auth::hash_password(TEST_ADMIN_PASSWORD).expect("hash admin");
        sqlx::query(
            "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
             VALUES ('user-admin', 'admin', ?, 1)",
        )
        .bind(&admin_hash)
        .execute(pool)
        .await
        .expect("seed admin user");

        let store_hash = auth::hash_password(TEST_STORE_PASSWORD).expect("hash store");
        sqlx::query(
            "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
             VALUES ('user-store1', 'store1', ?, 1)",
        )
        .bind(&store_hash)
        .execute(pool)
        .await
        .expect("seed store user");
    }

    async fn test_app() -> (Router, SqlitePool) {
        let pool = in_memory_pool().await;
        let state = AppState::new(pool.clone())
            .await
            .expect("app state with migrations applied");
        (router(state), pool)
    }

    async fn send(router: Router, request: Request<Body>) -> (StatusCode, String, Option<String>) {
        let response = router.oneshot(request).await.expect("router response");
        let status = response.status();
        let location = response
            .headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let set_cookie = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .find_map(|v| v.to_str().ok())
            .map(|v| v.split(';').next().unwrap_or("").to_string());
        let body = axum::body::to_bytes(response.into_body(), 1_048_576)
            .await
            .expect("body bytes");
        let body = String::from_utf8_lossy(&body).to_string();
        (status, body, set_cookie.or(location))
    }

    /// Login as `username`; returns (cookie, csrf_token).
    async fn login(
        app: &Router,
        pool: &SqlitePool,
        username: &str,
        password: &str,
    ) -> (String, String) {
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(format!(
                "username={username}&password={password}"
            )))
            .unwrap();
        let (status, _, cookie) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::SEE_OTHER, "login must redirect (PRG)");
        let cookie = cookie.expect("login must set a session cookie");
        let session_id = cookie
            .strip_prefix(auth::SESSION_COOKIE)
            .and_then(|rest| rest.strip_prefix('='))
            .expect("session cookie shape");
        let (csrf,): (String,) =
            sqlx::query_as("SELECT csrf_token FROM sessions WHERE session_id = ?")
                .bind(session_id)
                .fetch_one(pool)
                .await
                .expect("session row");
        (cookie, csrf)
    }

    fn form_body(csrf: &str, amount: &str, date: &str) -> String {
        format!(
            "csrf_token={csrf}&debit_account=ACC-DEBIT&credit_account=ACC-CREDIT&amount={amount}&currency=TWD&description=%E6%B8%AC%E8%A9%A6%E4%BA%A4%E6%98%93&transaction_date={date}"
        )
    }

    #[tokio::test]
    async fn unauthenticated_get_redirects_to_login() {
        let (app, _pool) = test_app().await;
        let (status, _, location) = send(
            app,
            Request::builder().uri("/").body(Body::empty()).unwrap(),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::SEE_OTHER,
            "protected route requires session"
        );
        assert_eq!(location.as_deref(), Some("/login"));
    }

    #[tokio::test]
    async fn login_failure_shows_generic_error() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from("username=admin&password=wrong-password"))
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::OK, "login failure re-renders the form");
        assert!(
            body.contains("登入失敗"),
            "generic failure message required"
        );
        assert!(
            !body.contains("wrong-password"),
            "password must never be echoed"
        );
    }

    #[tokio::test]
    async fn get_index_renders_zero_js_page() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, _) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        let request = Request::builder()
            .uri("/")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Flaps ERP 工作台"), "page title must render");
        assert!(!body.contains("<script"), "zero-JS invariant");
        assert!(body.contains("已連線"), "health badge reflects live DB");
        assert!(
            body.contains("csrf_token"),
            "write forms must carry a CSRF token"
        );
        assert!(body.contains("登出"), "logout control must be present");
    }

    #[tokio::test]
    async fn post_then_get_shows_posted_transaction_end_to_end() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        let today = chrono::Utc::now().date_naive().to_string();
        let request = Request::builder()
            .method("POST")
            .uri("/transactions")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie.clone())
            .body(Body::from(form_body(&csrf, "1234.50", &today)))
            .unwrap();
        let (status, _, location) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::SEE_OTHER);
        assert_eq!(location.as_deref(), Some("/#transactions"));

        let request = Request::builder()
            .uri("/")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let (_, body, _) = send(app, request).await;
        assert!(
            body.contains("1,234.50"),
            "posted amount must render exactly"
        );
        assert!(body.contains("測試交易"), "posted description must render");

        // P1-5: the posting must be a balanced two-leg entry in the database.
        let (debit_legs, credit_legs): (i64, i64) = sqlx::query_as(
            "SELECT COALESCE(SUM(CASE WHEN item_type = 'DEBIT' THEN 1 ELSE 0 END), 0), \
             COALESCE(SUM(CASE WHEN item_type = 'CREDIT' THEN 1 ELSE 0 END), 0) FROM transaction_items",
        )
        .fetch_one(&pool)
        .await
        .expect("items query");
        assert_eq!(debit_legs, 1, "one debit leg");
        assert_eq!(credit_legs, 1, "one credit leg");
    }

    #[tokio::test]
    async fn post_rejects_wrong_csrf_token() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, _csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        let today = chrono::Utc::now().date_naive().to_string();
        let request = Request::builder()
            .method("POST")
            .uri("/transactions")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie)
            .body(Body::from(form_body("forged-token", "10", &today)))
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "CSRF must be enforced (T-08)"
        );
        assert!(body.contains("CSRF"), "rejection must name the cause");
    }

    #[tokio::test]
    async fn post_rejects_same_debit_and_credit_account() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        let today = chrono::Utc::now().date_naive().to_string();
        let body = format!(
            "csrf_token={csrf}&debit_account=SAME&credit_account=SAME&amount=100&currency=TWD&description=x&transaction_date={today}"
        );
        let request = Request::builder()
            .method("POST")
            .uri("/transactions")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie)
            .body(Body::from(body))
            .unwrap();
        let (status, page, _) = send(app, request).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(
            page.contains("不得為同一科目"),
            "self-transfer must be rejected with guidance"
        );
    }

    #[tokio::test]
    async fn post_rejects_more_than_two_decimal_places() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        let today = chrono::Utc::now().date_naive().to_string();
        let request = Request::builder()
            .method("POST")
            .uri("/transactions")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie)
            .body(Body::from(form_body(&csrf, "0.005", &today)))
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("小數位數過多"), "error must name the problem");
        assert!(
            body.contains("value=\"0.005\""),
            "entered value must be preserved"
        );
    }

    #[tokio::test]
    async fn post_rejects_negative_amount() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        let today = chrono::Utc::now().date_naive().to_string();
        let request = Request::builder()
            .method("POST")
            .uri("/transactions")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie)
            .body(Body::from(form_body(&csrf, "-5", &today)))
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(
            body.contains("不得為負數"),
            "negative money must be rejected"
        );
    }

    #[tokio::test]
    async fn post_rejects_future_date() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        let future = (chrono::Utc::now().date_naive() + chrono::Duration::days(2)).to_string();
        let request = Request::builder()
            .method("POST")
            .uri("/transactions")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie)
            .body(Body::from(form_body(&csrf, "10", &future)))
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(
            body.contains("不得為未來日期"),
            "future dates must be rejected"
        );
    }

    #[tokio::test]
    async fn post_requires_ledger_write_permission() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        // user-store1 holds the STORE role (read-only) per RBAC seed.
        let (cookie, csrf) = login(&app, &pool, "store1", TEST_STORE_PASSWORD).await;

        let today = chrono::Utc::now().date_naive().to_string();
        let request = Request::builder()
            .method("POST")
            .uri("/transactions")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie)
            .body(Body::from(form_body(&csrf, "10", &today)))
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "RBAC must reject (FR-006)");
        assert!(
            body.contains("LEDGER_WRITE"),
            "rejection must name the missing permission"
        );
    }

    #[tokio::test]
    async fn logout_revokes_the_session() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        let request = Request::builder()
            .method("POST")
            .uri("/logout")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie.clone())
            .body(Body::from(format!("csrf_token={csrf}")))
            .unwrap();
        let (status, _, _) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::SEE_OTHER, "logout redirects (PRG)");

        // The revoked session no longer grants access.
        let request = Request::builder()
            .uri("/")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let (status, _, location) = send(app, request).await;
        assert_eq!(
            status,
            StatusCode::SEE_OTHER,
            "revoked session must be rejected"
        );
        assert_eq!(location.as_deref(), Some("/login"));
    }

    #[tokio::test]
    async fn health_reports_database_status() {
        let (app, _pool) = test_app().await;
        let (status, body, _) = send(
            app,
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "health probe is public");
        assert!(body.contains("\"db\":true"), "health must report live DB");
    }

    #[tokio::test]
    async fn design_system_css_is_served() {
        let (app, _pool) = test_app().await;
        let (status, body, _) = send(
            app,
            Request::builder()
                .uri("/assets/design-system.css")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body.contains("--color-primary"),
            "design tokens must be served"
        );
    }
}
