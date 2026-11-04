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
    /// Login lockout tracker (P1-7). Cloned per handle; the inner Mutex makes
    /// every clone share one counter map.
    pub(crate) rate_limiter: std::sync::Arc<auth::LoginRateLimiter>,
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
            rate_limiter: std::sync::Arc::new(auth::LoginRateLimiter::new()),
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
    pub can_manage_users: bool,
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
        .route("/admin/users", get(admin_users).post(admin_create_user))
        .route("/admin/users/toggle", post(admin_toggle_user))
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
        let html = build_index(
            &state,
            &session,
            Some(
                "CSRF 驗證失敗（問題：表單權杖與工作階段不符；可能原因：頁面停留過久或重複開啟；處理方式：重新整理頁面後再送出一次）。"
                    .to_string(),
            ),
            FormValues::default(),
        )
        .await;
        return match html {
            Ok(body) => (StatusCode::FORBIDDEN, Html(body)).into_response(),
            Err(e) => (StatusCode::FORBIDDEN, format!("頁面渲染失敗：{e}")).into_response(),
        };
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

/// Validate the form and post the journal entry. Returns the HTTP status and
/// user-facing message on failure (problem + reason + next step, FR-004).
async fn validate_and_post(
    state: &AppState,
    form: &TransactionForm,
    session: &Session,
) -> Result<(), (StatusCode, String)> {
    let acting_user = &session.user_id;
    // 1) RBAC (FR-006): permission by code, live DB query, server-side, always.
    if !state
        .permissions
        .has_permission_db(acting_user, "LEDGER_WRITE")
        .await
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

// ---------------------------------------------------------------------------
// User administration (P1-8): ROLE_MANAGE permission required on every route
// ---------------------------------------------------------------------------

/// One account row for the administration page.
#[derive(Debug, Clone)]
pub struct AdminUserRow {
    pub user_id: String,
    pub username: String,
    pub status_display: String,
    pub roles_display: String,
    pub is_active: bool,
}

/// Server-rendered user administration page (template:
/// `platform/templates/admin_users.html`).
#[derive(Template)]
#[template(path = "admin_users.html")]
pub struct AdminUsersTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub users: Vec<AdminUserRow>,
    pub roles: Vec<(String, String)>,
    pub error: Option<String>,
    pub notice: Option<String>,
}

/// Helper: does `user_id` hold the `ROLE_MANAGE` permission? (Live DB query
/// so role assignments made after startup take effect immediately.)
async fn can_manage_users(state: &AppState, user_id: &str) -> bool {
    state
        .permissions
        .has_permission_db(user_id, "ROLE_MANAGE")
        .await
}

async fn render_admin_forbidden(state: &AppState, session: &Session) -> Response {
    let rendered = build_admin_page(
        state,
        session,
        Some("您沒有使用者管理權限（缺少 ROLE_MANAGE）；請聯絡系統管理員。".to_string()),
        None,
    )
    .await;
    match rendered {
        Ok(body) => (StatusCode::FORBIDDEN, Html(body)).into_response(),
        Err(e) => (StatusCode::FORBIDDEN, format!("頁面渲染失敗：{e}")).into_response(),
    }
}

/// GET /admin/users — list accounts with roles and status.
async fn admin_users(State(state): State<AppState>, session: Extension<Session>) -> Response {
    if !can_manage_users(&state, &session.user_id).await {
        return render_admin_forbidden(&state, &session).await;
    }
    let rendered = build_admin_page(&state, &session, None, None).await;
    match rendered {
        Ok(body) => Html(body).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("頁面渲染失敗：{e}"),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct AdminCreateUserForm {
    pub csrf_token: String,
    pub username: String,
    pub password: String,
    pub role_code: String,
}

/// POST /admin/users — create an account and assign one seeded role.
async fn admin_create_user(
    State(state): State<AppState>,
    session: Extension<Session>,
    Form(form): Form<AdminCreateUserForm>,
) -> Response {
    if !can_manage_users(&state, &session.user_id).await {
        return render_admin_forbidden(&state, &session).await;
    }
    if form.csrf_token != session.csrf_token {
        return (
            StatusCode::FORBIDDEN,
            "CSRF 驗證失敗；請重新整理頁面後再操作。".to_string(),
        )
            .into_response();
    }

    match admin_create_user_validated(&state, &form, &session).await {
        Ok(()) => Redirect::to("/admin/users").into_response(),
        Err((status, message)) => {
            let rendered = build_admin_page(&state, &session, Some(message), None).await;
            match rendered {
                Ok(body) => (status, Html(body)).into_response(),
                Err(e) => (status, format!("頁面渲染失敗：{e}")).into_response(),
            }
        }
    }
}

async fn admin_create_user_validated(
    state: &AppState,
    form: &AdminCreateUserForm,
    session: &Session,
) -> Result<(), (StatusCode, String)> {
    // Username policy: lowercase ascii, digits, underscore, hyphen (safe to
    // embed in user_id), 3..=32 chars.
    let username = form.username.trim().to_lowercase();
    let valid_chars = username
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-');
    if username.len() < 3 || username.len() > 32 || !valid_chars {
        return Err((
            StatusCode::BAD_REQUEST,
            "帳號名稱格式不正確（3–32 字元，僅限小寫英文、數字、底線、連字號）；請修正後重新送出。"
                .to_string(),
        ));
    }

    if form.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            "密碼長度不足（至少 8 字元）；請設定更長的密碼。".to_string(),
        ));
    }

    // Role must exist in the seeded roles table (whitelist by construction).
    let role_id: Option<(String,)> =
        sqlx::query_as("SELECT role_id FROM roles WHERE role_code = ?")
            .bind(form.role_code.trim())
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("資料庫查詢失敗：{e}"),
                )
            })?;
    let Some((role_id,)) = role_id else {
        return Err((
            StatusCode::BAD_REQUEST,
            "角色不存在；請從清單選擇。".to_string(),
        ));
    };

    let hash = auth::hash_password(&form.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("密碼雜湊失敗：{e}"),
        )
    })?;
    let user_id = format!("user-{username}");

    sqlx::query(
        "INSERT INTO users (user_id, username, password_hash, is_active) VALUES (?, ?, ?, 1)",
    )
    .bind(&user_id)
    .bind(&username)
    .bind(&hash)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            (
                StatusCode::BAD_REQUEST,
                format!("帳號 {username} 已存在；請改用其他名稱。"),
            )
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("建立失敗：{e}"))
        }
    })?;

    sqlx::query(
        "INSERT OR IGNORE INTO user_roles (user_id, role_id, assigned_at, assigned_by) \
         VALUES (?, ?, CURRENT_TIMESTAMP, ?)",
    )
    .bind(&user_id)
    .bind(&role_id)
    .bind(&session.user_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("角色指派失敗：{e}"),
        )
    })?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct AdminToggleForm {
    pub csrf_token: String,
    pub user_id: String,
}

/// POST /admin/users/toggle — activate/deactivate an account (reversible).
/// Self-deactivation is rejected so an administrator cannot lock themselves
/// out of the user management page.
async fn admin_toggle_user(
    State(state): State<AppState>,
    session: Extension<Session>,
    Form(form): Form<AdminToggleForm>,
) -> Response {
    if !can_manage_users(&state, &session.user_id).await {
        return render_admin_forbidden(&state, &session).await;
    }
    if form.csrf_token != session.csrf_token {
        return (
            StatusCode::FORBIDDEN,
            "CSRF 驗證失敗；請重新整理頁面後再操作。".to_string(),
        )
            .into_response();
    }
    if form.user_id == session.user_id {
        return (
            StatusCode::BAD_REQUEST,
            "不得停用自己的帳號；請由其他管理員執行。".to_string(),
        )
            .into_response();
    }

    sqlx::query("UPDATE users SET is_active = 1 - is_active WHERE user_id = ?")
        .bind(&form.user_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("更新失敗：{e}")))
        .ok();

    Redirect::to("/admin/users").into_response()
}

async fn build_admin_page(
    state: &AppState,
    session: &Session,
    error: Option<String>,
    notice: Option<String>,
) -> Result<String, askama::Error> {
    let rows: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT u.user_id, u.username, u.is_active FROM users u ORDER BY u.username",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let role_rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT ur.user_id, r.role_code, r.role_name FROM user_roles ur \
         JOIN roles r ON ur.role_id = r.role_id ORDER BY ur.user_id",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let users: Vec<AdminUserRow> = rows
        .into_iter()
        .map(|(user_id, username, is_active)| {
            let roles_display = role_rows
                .iter()
                .filter(|(uid, _, _)| uid == &user_id)
                .map(|(_, code, _)| code.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            AdminUserRow {
                user_id: user_id.clone(),
                username,
                status_display: if is_active == 1 {
                    "啟用".to_string()
                } else {
                    "停用".to_string()
                },
                roles_display,
                is_active: is_active == 1,
            }
        })
        .collect();

    let roles: Vec<(String, String)> =
        sqlx::query_as("SELECT role_code, role_name FROM roles ORDER BY role_code")
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

    AdminUsersTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        users,
        roles,
        error,
        notice,
    }
    .render()
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
        can_manage_users: can_manage_users(state, &session.user_id).await,
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

    #[tokio::test]
    async fn admin_creates_user_who_can_then_login() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        // GET /admin/users as admin (ROLE_MANAGE): page renders.
        let request = Request::builder()
            .uri("/admin/users")
            .header(header::COOKIE, cookie.clone())
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("帳號清單"), "admin page must render");
        assert!(body.contains("admin"), "seeded account must be listed");

        // POST /admin/users: create finance2 with the FINANCE role.
        let create =
            format!("csrf_token={csrf}&username=finance2&password=finance2-pass&role_code=FINANCE");
        let request = Request::builder()
            .method("POST")
            .uri("/admin/users")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie.clone())
            .body(Body::from(create))
            .unwrap();
        let (status, _, location) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::SEE_OTHER, "create must PRG");
        assert_eq!(location.as_deref(), Some("/admin/users"));

        // The new user can log in and post (FINANCE role has LEDGER_WRITE).
        let (cookie2, csrf2) = login(&app, &pool, "finance2", "finance2-pass").await;
        let today = chrono::Utc::now().date_naive().to_string();
        let request = Request::builder()
            .method("POST")
            .uri("/transactions")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie2)
            .body(Body::from(form_body(&csrf2, "77", &today)))
            .unwrap();
        let (status, _, _) = send(app, request).await;
        assert_eq!(
            status,
            StatusCode::SEE_OTHER,
            "new user must be able to post"
        );
    }

    #[tokio::test]
    async fn admin_users_forbidden_without_role_manage() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, _csrf) = login(&app, &pool, "store1", TEST_STORE_PASSWORD).await;

        let request = Request::builder()
            .uri("/admin/users")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "ROLE_MANAGE required (FR-006)"
        );
        assert!(
            body.contains("ROLE_MANAGE"),
            "rejection must name the permission"
        );
    }

    #[tokio::test]
    async fn deactivated_user_cannot_login() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        // Deactivate user-store1.
        let request = Request::builder()
            .method("POST")
            .uri("/admin/users/toggle")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie)
            .body(Body::from(format!("csrf_token={csrf}&user_id=user-store1")))
            .unwrap();
        let (status, _, _) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::SEE_OTHER);

        // store1 can no longer authenticate.
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(format!(
                "username=store1&password={TEST_STORE_PASSWORD}"
            )))
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::OK, "deactivated login re-renders form");
        assert!(
            body.contains("登入失敗"),
            "deactivated account must not log in"
        );
    }

    #[tokio::test]
    async fn admin_cannot_deactivate_self() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        let request = Request::builder()
            .method("POST")
            .uri("/admin/users/toggle")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie)
            .body(Body::from(format!("csrf_token={csrf}&user_id=user-admin")))
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "self-deactivation rejected"
        );
        assert!(body.contains("不得停用自己的帳號"));
    }
}
