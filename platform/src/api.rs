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
use axum::extract::{Form, Path, Query, State};
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
#[cfg(feature = "bankpay")]
use crate::bankpay::api::{
    bankpay_health_handler, config_get_handler, config_post_handler, export_handler,
    import_handler, query_handler, transition_handler,
};
use crate::db::DatabaseManager;
#[cfg(feature = "bankpay")]
use crate::status::status_handler;

/// Maximum lengths accepted from the form (fail-closed input validation).
const MAX_DESCRIPTION_LEN: usize = 200;
const MAX_ACCOUNT_ID_LEN: usize = 64;
const SUPPORTED_CURRENCIES: [&str; 4] = ["TWD", "USD", "EUR", "JPY"];
/// Dashboard overview: how many recent transactions the summary table shows
/// (the full, paged list lives under the 交易管理 section — see M1).
const RECENT_TRANSACTION_LIMIT: i64 = 5;

/// Transaction list page size (M1, blueprint §M1: 20 rows per page).
const LIST_PAGE_SIZE: i64 = 20;

/// Fail-closed page parsing (M1): non-numeric, non-positive or absurd values
/// fall back to page 1 — a crafted `page` parameter must never reach SQL and
/// must never produce a 500. The upper bound is clamped against the real page
/// count via [`clamp_page`] once the total is known.
fn parse_page_param(raw: Option<&str>) -> i64 {
    raw.map(str::trim)
        .and_then(|s| s.parse::<i64>().ok())
        .filter(|&p| (1..=1_000_000).contains(&p))
        .unwrap_or(1)
}

/// Clamp a requested page into `[1, total_pages]` (past-the-end requests show
/// the last page, per blueprint §M1 acceptance).
fn clamp_page(requested: i64, total_pages: i64) -> i64 {
    requested.clamp(1, total_pages.max(1))
}

/// Shared application state for the web tier.
#[derive(Clone)]
pub struct AppState {
    pub(crate) pool: SqlitePool,
    engine: LedgerEngine,
    permissions: PermissionManager,
    /// Login lockout tracker (P1-7). Cloned per handle; the inner Mutex makes
    /// every clone share one counter map.
    pub(crate) rate_limiter: std::sync::Arc<auth::LoginRateLimiter>,
    /// Full-text search engine (Slice 13).
    pub(crate) search: crate::search::SearchEngine,
    /// E.Sun BankPay 配置（可變，供管理員於執行時切換模擬/生產模式）。
    #[cfg(feature = "bankpay")]
    pub(crate) bankpay_config:
        std::sync::Arc<tokio::sync::Mutex<crate::bankpay::model::EsunConfig>>,
    /// 系統狀態監控引擎（Slice 13）。
    #[cfg(feature = "bankpay")]
    pub(crate) status_engine: crate::status::StatusEngine,
}

impl AppState {
    /// Assemble state over a pool whose migrations have already been applied
    /// (via [`DatabaseManager::initialize`]).
    pub async fn new(pool: SqlitePool) -> Result<Self, Box<dyn std::error::Error>> {
        DatabaseManager::new(pool.clone()).initialize().await?;
        // Slice 11/12: purge expired session rows at startup so the table
        // stays bounded even when no logins occur for long periods.
        auth::purge_expired_sessions(&pool).await?;
        let permissions = PermissionManager::new(pool.clone()).await;
        let engine = LedgerEngine::with_pool(pool.clone()).await?;
        Ok(Self {
            pool,
            engine,
            permissions,
            rate_limiter: std::sync::Arc::new(auth::LoginRateLimiter::new()),
            search: crate::search::SearchEngine::new(),
            #[cfg(feature = "bankpay")]
            bankpay_config: std::sync::Arc::new(tokio::sync::Mutex::new(
                crate::bankpay::model::EsunConfig::default(),
            )),
            #[cfg(feature = "bankpay")]
            status_engine: crate::status::StatusEngine::new(env!("CARGO_PKG_VERSION")),
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

/// Render model for a recent audit log entry.
#[derive(Debug, Clone)]
pub struct AuditLogRow {
    pub log_id: String,
    pub user_id: String,
    pub username: String,
    pub action: String,
    pub resource: String,
    pub details: String,
    pub created_at: String,
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
    pub current_ym: String,
    /// Server-generated SVG (monthly debit/credit bars). Generated purely from
    /// integer aggregates and fixed literals — no user-controlled content —
    /// hence the `safe` filter in the template (audited, Slice 8 D1).
    pub chart_svg: String,
    pub db_connected: bool,
    pub transaction_count: String,
    pub total_amount_display: String,
    pub account_count: String,
    pub pending_review_count: String,
    pub prev_page: Option<String>,
    pub page: u32,
    pub total_pages: u32,
    pub total_transactions: String,
    pub next_page: Option<String>,
    pub transactions: Vec<TransactionRow>,
    /// Dashboard overview: 5 most recent entries (unpaged, M1).
    pub recent: Vec<TransactionRow>,
    pub accounts: Vec<String>,
    pub error: Option<String>,
    pub form: FormValues,
    // ── 跨模組狀態（Slice 13） ──
    pub search_index_count: usize,
    pub sync_pending_count: usize,
    pub com_bridge_objects: usize,
    pub ai_models_available: usize,
    pub recent_audit_logs: Vec<AuditLogRow>,
}

/// Build the zero-JS monthly bar chart SVG from integer aggregates.
/// Deterministic layout: 6 months, two bars each (debit/credit), value labels
/// derived with `format_minor`. No external assets, no JavaScript.
fn monthly_chart_svg(totals: &[flaps_core::ledger::engine::MonthlyTotal]) -> String {
    if totals.is_empty() {
        return String::new();
    }
    let max = totals
        .iter()
        .map(|t| t.debit_minor.max(t.credit_minor))
        .max()
        .unwrap_or(0)
        .max(1);
    let width = 60usize * totals.len().max(1) + 40;
    let height = 190;
    let mut svg = format!(
        "<svg viewBox=\"0 0 {width} {height}\" width=\"100%\" height=\"{height}\" role=\"img\" \
         aria-label=\"近六個月借貸合計長條圖\" xmlns=\"http://www.w3.org/2000/svg\">\
         <title>近六個月借貸合計</title>\
         <desc>每月借方與貸方合計，單位為整數分還原之貨幣金額。</desc>"
    );
    for (idx, t) in totals.iter().enumerate() {
        let base_x = 30 + idx * 60;
        let scale = |v: i64| -> usize { (v * 120 / max) as usize };
        let d_h = scale(t.debit_minor);
        let c_h = scale(t.credit_minor);
        svg.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"14\" height=\"{}\" fill=\"#005fcc\"></rect>",
            base_x,
            160 - d_h,
            d_h
        ));
        svg.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"14\" height=\"{}\" fill=\"#f57c00\"></rect>",
            base_x + 18,
            160 - c_h,
            c_h
        ));
        svg.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" font-size=\"9\" text-anchor=\"middle\" fill=\"#666666\">{}</text>",
            base_x + 16,
            175,
            t.ym
        ));
        svg.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" font-size=\"8\" text-anchor=\"middle\" fill=\"#1a1a1a\">{}</text>",
            base_x + 16,
            152 - d_h.max(c_h),
            format_minor(t.debit_minor, 2)
        ));
    }
    svg.push_str("</svg>");
    svg
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
        .route(
            "/transactions",
            get(transactions_page).post(create_transaction),
        )
        .route("/transactions/:id", get(transaction_detail_page))
        .route("/admin/users", get(admin_users).post(admin_create_user))
        .route("/admin/users/toggle", post(admin_toggle_user))
        .route("/admin/users/reset_password", post(admin_reset_password))
        .route("/import", get(import_upload_page))
        .route("/import/preview", post(import_preview))
        .route("/import/confirm", post(import_confirm))
        .route("/drafts", get(drafts_page))
        .route("/drafts/parse", post(drafts_parse))
        .route("/new", get(wizard_page))
        .route("/new/step2", post(wizard_step2))
        .route("/reports/monthly", get(report_monthly_page))
        .route("/reports/monthly.csv", get(report_monthly_csv))
        .route("/reports/pivot", get(report_pivot_page))
        .route("/reports/parallel", get(report_parallel_page))
        .route("/drafts/ai_suggest", post(ai_suggest))
        .route("/drafts/adopt", post(drafts_adopt))
        .route("/admin/parse_coverage", get(parse_coverage_page))
        .route("/states", get(states_page));

    // BankPay／系統狀態 API（並行流成果整合）：以 feature 隔離；
    // 未啟用時整段（handlers、狀態、路由）不參與編譯與執行。
    #[cfg(feature = "bankpay")]
    let protected = protected
        // ── BankPay API (E.Sun 整合) ──
        .route("/api/bankpay/export", post(export_handler))
        .route("/api/bankpay/import", post(import_handler))
        .route("/api/bankpay/query", get(query_handler))
        .route(
            "/api/bankpay/config",
            get(config_get_handler).post(config_post_handler),
        )
        .route("/api/bankpay/health", get(bankpay_health_handler))
        .route("/api/bankpay/transition", post(transition_handler))
        // ── System Status API ──
        .route("/api/status", get(status_handler));

    let protected = protected
        .route_layer(axum::extract::DefaultBodyLimit::max(IMPORT_MAX_BYTES))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::require_session,
        ))
        .merge(crate::cross_module_api::router(state.clone()));

    // OWASP secure headers on EVERY response (Slice 11): CSP with no script
    // sources at all (zero-JS UI), nosniff, frame denial, no-referrer and
    // no-store (session data must not be cached).
    public
        .merge(protected)
        .layer(axum::middleware::map_response(auth::security_headers))
        .with_state(state)
}

/// Query string for list pages: `?page=N` (M1). Parsed fail-closed.
#[derive(Deserialize)]
pub struct PageQuery {
    pub page: Option<String>,
}

async fn index(
    State(state): State<AppState>,
    session: Extension<Session>,
    Query(query): Query<PageQuery>,
) -> Response {
    let html = build_index(
        &state,
        &session,
        None,
        FormValues::default(),
        parse_page_param(query.page.as_deref()),
    )
    .await;
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

/// GET /api/health — machine-readable deployment probe (R2/M2).
///
/// Reports DB reachability, applied migration count and the transaction
/// total. When the database is unreachable the probe answers 503 with
/// `status = "degraded"` so orchestrators can take the instance out.
async fn health(State(state): State<AppState>) -> Response {
    let db_connected = sqlx::query("SELECT 1").fetch_one(&state.pool).await.is_ok();
    let (migrations, transactions): (i64, i64) = if db_connected {
        let m: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&state.pool)
            .await
            .unwrap_or(-1);
        let t: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions")
            .fetch_one(&state.pool)
            .await
            .unwrap_or(-1);
        (m, t)
    } else {
        (-1, -1)
    };
    let status = if db_connected { "ok" } else { "degraded" };
    let code = if db_connected {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        code,
        Json(serde_json::json!({
            "status": status,
            "db": db_connected,
            "migrations_applied": migrations,
            "transactions": transactions,
            "service": "flaps-server",
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
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
            1,
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
            let html = build_index(&state, &session, Some(message), saved_form, 1).await;
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
    let transaction_id = uuid::Uuid::new_v4().to_string();
    let transaction_code = format!("TX-{}", &transaction_id[..8]);
    //    Two legs are constructed here (P1-5): DEBIT on the debit account and
    //    CREDIT on the credit account, same amount and currency — balance is
    //    guaranteed by construction and re-verified by the engine.
    let transaction = match Transaction::builder()
        .transaction_id(transaction_id.clone())
        .transaction_code(transaction_code)
        .transaction_date(transaction_date)
        .description(description.to_string())
        .reference_number(None)
        .status("POSTED")
        .source_type(Some("web".to_string()))
        .created_by(acting_user.to_string())
        .build()
    {
        Ok(t) => t,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("交易建構失敗：{e}"),
            ));
        }
    };
    let (debit_item, credit_item) = match (
        TransactionItem::builder()
            .item_id(uuid::Uuid::new_v4().to_string())
            .transaction_id(transaction_id.clone())
            .account_id(debit_account)
            .item_type("DEBIT")
            .amount_minor(amount_minor)
            .currency(form.currency.clone())
            .exchange_rate_scaled(10_000) // 1.0000 scaled by 10^4
            .build(),
        TransactionItem::builder()
            .item_id(uuid::Uuid::new_v4().to_string())
            .transaction_id(transaction_id)
            .account_id(credit_account)
            .item_type("CREDIT")
            .amount_minor(amount_minor)
            .currency(form.currency.clone())
            .exchange_rate_scaled(10_000)
            .build(),
    ) {
        (Ok(d), Ok(c)) => (d, c),
        (Err(e), _) | (_, Err(e)) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("分錄建構失敗：{e}"),
            ));
        }
    };
    let items = vec![debit_item, credit_item];
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

    // P1-9: an account that is now inactive must lose its live sessions
    // immediately (a deactivated operator must not keep working until the
    // session naturally expires).
    let (is_active,): (i64,) = sqlx::query_as("SELECT is_active FROM users WHERE user_id = ?")
        .bind(&form.user_id)
        .fetch_one(&state.pool)
        .await
        .unwrap_or((1,));
    if is_active == 0 {
        let _ = sqlx::query("DELETE FROM sessions WHERE user_id = ?")
            .bind(&form.user_id)
            .execute(&state.pool)
            .await;
    }

    Redirect::to("/admin/users").into_response()
}

#[derive(Debug, Deserialize)]
pub struct AdminResetPasswordForm {
    pub csrf_token: String,
    pub user_id: String,
    pub new_password: String,
}

/// POST /admin/users/reset_password — administrator sets a new password for
/// an account (P1-8 residual). The old hash is replaced (Argon2id), all
/// existing sessions of that account are revoked (a leaked password must not
/// keep working after a reset), and the change is audited.
async fn admin_reset_password(
    State(state): State<AppState>,
    session: Extension<Session>,
    Form(form): Form<AdminResetPasswordForm>,
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
    if form.new_password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            "新密碼長度不足（至少 8 字元）；請設定更長的密碼。".to_string(),
        )
            .into_response();
    }
    let (exists,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE user_id = ?")
        .bind(&form.user_id)
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));
    if exists == 0 {
        return (StatusCode::BAD_REQUEST, "帳號不存在。".to_string()).into_response();
    }

    let hash = match auth::hash_password(&form.new_password) {
        Ok(h) => h,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("密碼雜湊失敗：{e}"),
            )
                .into_response()
        }
    };
    sqlx::query("UPDATE users SET password_hash = ? WHERE user_id = ?")
        .bind(&hash)
        .bind(&form.user_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("更新失敗：{e}")))
        .ok();

    // Security: revoke the target account's sessions after a reset.
    let _ = sqlx::query("DELETE FROM sessions WHERE user_id = ?")
        .bind(&form.user_id)
        .execute(&state.pool)
        .await;

    // Audit: password resets are security events (transaction_id NULL).
    let _ = sqlx::query(
        "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) VALUES (?, NULL, 'PASSWORD_RESET', ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(format!("{{\"user_id\":\"{}\"}}", form.user_id))
    .bind(&session.user_id)
    .execute(&state.pool)
    .await;

    Redirect::to("/admin/users").into_response()
}
// ---------------------------------------------------------------------------
// Slice 10 Layer F: parallel-coordinates projection (PAR-001/002)
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "report_parallel.html")]
pub struct ParallelTemplate {
    pub acting_user: String,
    pub months: i64,
    pub chart_svg: String,
}

/// GET /reports/parallel — per-account polylines across monthly axes
/// (LEDGER_READ; zero-JS alternative to interactive 3D).
async fn report_parallel_page(
    State(state): State<AppState>,
    session: Extension<Session>,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    if !state
        .permissions
        .has_permission_db(&session.user_id, "LEDGER_READ")
        .await
    {
        return (StatusCode::FORBIDDEN, "需要 LEDGER_READ 權限。".to_string()).into_response();
    }
    let months: i64 = query
        .get("months")
        .and_then(|m| m.parse().ok())
        .unwrap_or(6);
    let months = months.clamp(2, 12);
    let top: usize = query
        .get("top")
        .and_then(|t| t.parse().ok())
        .unwrap_or(10)
        .clamp(1, 20);

    // Per-account monthly debit totals over the last `months` months.
    let raw: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT i.account_id, strftime('%Y-%m', t.transaction_date) AS ym, \
         COALESCE(SUM(CASE WHEN i.item_type = 'DEBIT' THEN i.amount_minor ELSE 0 END), 0) AS d \
         FROM transaction_items i JOIN transactions t ON i.transaction_id = t.transaction_id \
         WHERE strftime('%Y-%m', t.transaction_date) >= date('now', printf('-%d months', ?), 'start of month') \
         GROUP BY i.account_id, ym ORDER BY i.account_id, ym",
    )
    .bind(months)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    // Axes (sorted month labels present in the data).
    let mut axes: Vec<String> = raw.iter().map(|(_, ym, _)| ym.clone()).collect();
    axes.sort();
    axes.dedup();

    // Aggregate per account into ordered series keyed by the axis list.
    use std::collections::BTreeMap;
    let mut per_account: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();
    for (account, ym, debit) in raw {
        *per_account
            .entry(account)
            .or_default()
            .entry(ym)
            .or_insert(0) += debit;
    }
    let mut series: Vec<flaps_core::parallel::ParallelSeries> = per_account
        .into_iter()
        .map(|(account, by_month)| {
            let values = axes
                .iter()
                .map(|ym| (ym.clone(), by_month.get(ym).copied().unwrap_or(0)))
                .collect();
            flaps_core::parallel::ParallelSeries {
                label: account,
                values,
            }
        })
        .collect();
    // Top-N by grand total (descending) — deterministic ordering.
    series.sort_by(|a, b| {
        let ta: i64 = a.values.iter().map(|(_, v)| v).sum();
        let tb: i64 = b.values.iter().map(|(_, v)| v).sum();
        tb.cmp(&ta).then_with(|| a.label.cmp(&b.label))
    });
    series.truncate(top);

    let chart_svg = flaps_core::parallel::render_parallel_coordinates(&series, 720, 260);
    let template = ParallelTemplate {
        acting_user: session.user_id.clone(),
        months,
        chart_svg,
    };
    render_or_500(template)
}

// ---------------------------------------------------------------------------
// Slice 10 Layer G: AI suggestion (AIS-001..003) - gated, human-confirmed
// ---------------------------------------------------------------------------

/// Read the AI feature flag from erp_config (default OFF).
async fn ai_suggest_enabled(state: &AppState) -> bool {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT config_value FROM erp_config WHERE config_category = 'ai' AND config_key = 'suggest_enabled'",
    )
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);
    row.map(|(v,)| v == "1").unwrap_or(false)
}

#[derive(Template)]
#[template(path = "ai_suggestion.html")]
pub struct AiSuggestionTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub suggestion: Option<flaps_core::ParsedSuggestion>,
    pub degrade_message: Option<String>,
}

fn render_with_status(template: impl askama::Template, status: StatusCode) -> Response {
    match template.render() {
        Ok(body) => (status, Html(body)).into_response(),
        Err(e) => (status, format!("頁面渲染失敗：{e}")).into_response(),
    }
}

/// POST /drafts/ai_suggest - AIS-001..003.
async fn ai_suggest(
    State(state): State<AppState>,
    session: Extension<Session>,
    Form(form): Form<DraftParseForm>,
) -> Response {
    // CSRF (T-08): synchronizer token check before any work.
    if form.csrf_token != session.csrf_token {
        return (
            StatusCode::FORBIDDEN,
            "CSRF 驗證失敗；請重新整理頁面後再操作。".to_string(),
        )
            .into_response();
    }
    // AIS-001: flag gate (default OFF) - 503 with enablement guidance.
    if !ai_suggest_enabled(&state).await {
        let template = AiSuggestionTemplate {
            acting_user: session.user_id.clone(),
            csrf_token: session.csrf_token.clone(),
            suggestion: None,
            degrade_message: Some(
                "AI 建議未啟用（問題：特性旗標 ai/suggest_enabled 未開啟；處理方式：請管理員於設定（erp_config）啟用，或改用規則解析／引導式精靈）。".to_string(),
            ),
        };
        return render_with_status(template, StatusCode::SERVICE_UNAVAILABLE);
    }

    let text = form.text.trim().to_string();
    if text.is_empty() || text.len() > 10_000 {
        return (
            StatusCode::BAD_REQUEST,
            "內容空白或超過 10,000 字元。".to_string(),
        )
            .into_response();
    }

    // Build the Ollama client from configuration (existing ai module reuse).
    let model = erp_config_value(&state, "model")
        .await
        .unwrap_or_else(|| "llama3".to_string());
    let base_url = erp_config_value(&state, "base_url")
        .await
        .unwrap_or_else(|| "http://127.0.0.1:11434".to_string());
    let client =
        crate::ai::ollama_client::OllamaClient::new(crate::ai::ollama_client::OllamaConfig {
            base_url,
            model,
            timeout_ms: 5_000,
            temperature: 0.2,
            top_p: 0.9,
            max_tokens: Some(512),
        });

    let prompt = format!(
        "你是會計分錄解析助理。僅輸出一個 JSON 物件，欄位：date（YYYY-MM-DD）、debit_account、credit_account、amount（兩位小數字串）、currency。不要輸出其他文字。原文：{text}"
    );

    let response_text = match client.generate(&prompt).await {
        Ok(t) => t,
        Err(e) => {
            // 韌性：Ollama 離線/逾時 -> graceful degradation.
            let template = AiSuggestionTemplate {
                acting_user: session.user_id.clone(),
                csrf_token: session.csrf_token.clone(),
                suggestion: None,
                degrade_message: Some(format!(
                    "AI 服務暫時不可用（可能原因：Ollama 未啟動或逾時；處理方式：請改用規則解析或引導式精靈）。詳情：{e}"
                )),
            };
            return render_with_status(template, StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    // AIS-003: schema gate - malformed output is rejected before any use.
    let parsed = match flaps_core::parse_ai_suggestion(&response_text) {
        Ok(p) => p,
        Err(msg) => {
            let template = AiSuggestionTemplate {
                acting_user: session.user_id.clone(),
                csrf_token: session.csrf_token.clone(),
                suggestion: None,
                degrade_message: Some(format!(
                    "AI 輸出未通過 schema 驗證（{msg}）；請改用規則解析。"
                )),
            };
            return render_with_status(template, StatusCode::BAD_REQUEST);
        }
    };

    // AIS-002: rendered as a SUGGESTION ONLY - nothing is stored or posted.
    // OQ-10-1: request audited WITHOUT the raw text (T-04 log masking); only
    // the length and prompt version are recorded.
    let _ = sqlx::query(
        "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) \
         VALUES (?, NULL, 'AI_SUGGEST_REQUESTED', ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(format!(
        "{{\"text_len\":{},\"prompt_version\":\"v1\",\"schema_gate\":\"passed\"}}",
        text.len()
    ))
    .bind(&session.user_id)
    .execute(&state.pool)
    .await;

    let template = AiSuggestionTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        suggestion: Some(parsed),
        degrade_message: None,
    };
    render_with_status(template, StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct DraftAdoptForm {
    pub csrf_token: String,
    pub date: String,
    pub debit_account: String,
    pub credit_account: String,
    pub amount: String,
    pub currency: String,
    pub description: String,
}

/// POST /drafts/adopt - human adopts a suggestion into a normal draft
/// (source='ai-adopted'); from here it follows the standard rule path.
/// AIS-002: this is the ONLY way an AI suggestion enters the system, and it
/// never touches the ledger directly.
async fn drafts_adopt(
    State(state): State<AppState>,
    session: Extension<Session>,
    Form(form): Form<DraftAdoptForm>,
) -> Response {
    if form.csrf_token != session.csrf_token {
        return (
            StatusCode::FORBIDDEN,
            "CSRF 驗證失敗；請重新整理頁面後再操作。".to_string(),
        )
            .into_response();
    }
    let amount_minor = match flaps_core::parse_decimal_to_minor(&form.amount, 2) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("金額錯誤：{e}")).into_response(),
    };
    let date = match chrono::NaiveDate::parse_from_str(&form.date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return (StatusCode::BAD_REQUEST, "日期格式不正確。".to_string()).into_response(),
    };
    let entry = flaps_core::DraftEntry {
        date: Some(date),
        amount_minor: Some(amount_minor),
        debit_account: Some(form.debit_account.trim().to_string()),
        credit_account: Some(form.credit_account.trim().to_string()),
        currency: Some(form.currency.trim().to_uppercase()),
        evidence: Vec::new(),
    };
    let draft_id = uuid::Uuid::new_v4().to_string();
    let insert = sqlx::query(
        "INSERT INTO drafts (draft_id, source, draft_json, created_by) VALUES (?, 'ai-adopted', ?, ?)",
    )
    .bind(&draft_id)
    .bind(serde_json::to_string(&entry).unwrap_or_default())
    .bind(&session.user_id)
    .execute(&state.pool)
    .await;
    match insert {
        Ok(_) => Redirect::to(&format!("/new?step=2&from_draft={draft_id}")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("草稿儲存失敗：{e}"),
        )
            .into_response(),
    }
}

async fn erp_config_value(state: &AppState, key: &str) -> Option<String> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT config_value FROM erp_config WHERE config_category = 'ai' AND config_key = ?",
    )
    .bind(key)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);
    row.map(|(v,)| v)
}

// ---------------------------------------------------------------------------
// Slice 10 Layer H: parse coverage measurement (COV-001, OQ-8-2 data)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CoverageRow {
    pub draft_id: String,
    pub completeness: String,
}

#[derive(Template)]
#[template(path = "parse_coverage.html")]
pub struct ParseCoverageTemplate {
    pub acting_user: String,
    pub total: usize,
    pub complete: usize,
    pub ratio_display: String,
    pub rows: Vec<CoverageRow>,
}

/// GET /admin/parse_coverage - rule-hit ratio over stored drafts (COV-001).
async fn parse_coverage_page(
    State(state): State<AppState>,
    session: Extension<Session>,
) -> Response {
    if !can_manage_users(&state, &session.user_id).await {
        return render_admin_forbidden(&state, &session).await;
    }
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT draft_id, draft_json FROM drafts ORDER BY created_at DESC LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut complete = 0usize;
    let coverage_rows: Vec<CoverageRow> = rows
        .iter()
        .map(|(id, json)| {
            let is_complete = serde_json::from_str::<flaps_core::DraftEntry>(json)
                .map(|d| d.is_complete())
                .unwrap_or(false);
            if is_complete {
                complete += 1;
            }
            CoverageRow {
                draft_id: id.clone(),
                completeness: if is_complete {
                    "完整".to_string()
                } else {
                    "不完整".to_string()
                },
            }
        })
        .collect();

    let total = coverage_rows.len();
    let ratio = (complete * 100).checked_div(total).unwrap_or(0);

    let template = ParseCoverageTemplate {
        acting_user: session.user_id.clone(),
        total,
        complete,
        ratio_display: format!("{ratio}%"),
        rows: coverage_rows,
    };
    render_or_500(template)
}

#[derive(Template)]
#[template(path = "states.html")]
pub struct StatesTemplate {
    pub acting_user: String,
}

/// GET /states - UI component-state showcase for design review and acceptance
/// (UI-UX spec E3.7): buttons, forms, badges, feedback, table, KPI, pagination.
async fn states_page(session: Extension<Session>) -> Response {
    render_or_500(StatesTemplate {
        acting_user: session.user_id.clone(),
    })
}

/// Minimal RFC 3986 percent-encoding for query values (unreserved characters
/// pass through; everything else, including every UTF-8 byte of CJK text, is
/// percent-encoded). Used when building search-preserving pagination links.
fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(*byte as char);
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// Map a transaction status to a badge class. Colour and text are always shown
/// together (WCAG 2.2 AA Use of Color).
fn status_badge(status: &str) -> &'static str {
    match status {
        "POSTED" | "COMPLETED" | "DONE" => "badge-success",
        "PENDING_REVIEW" | "PENDING" => "badge-warning",
        "VOID" | "VOIDED" | "REJECTED" | "FAILED" => "badge-error",
        "DRAFT" => "badge-neutral",
        _ => "badge-info",
    }
}

/// Search-aware paged transaction rows for the /transactions list (P1). The
/// unfiltered path reuses the engine helper; search uses bound LIKE
/// parameters (never concatenated SQL).
async fn query_transactions_for_list(
    state: &AppState,
    search: &str,
    limit: i64,
    offset: i64,
) -> Vec<flaps_core::ledger::engine::TransactionSummary> {
    if search.is_empty() {
        return state
            .engine
            .query_transaction_summaries_paged(limit, offset)
            .await
            .unwrap_or_default();
    }
    let pattern = format!("%{search}%");
    sqlx::query_as::<_, flaps_core::ledger::engine::TransactionSummary>(
        "SELECT t.transaction_id, t.transaction_code, t.transaction_date, t.description, \
         t.reference_number, t.status, t.source_type, t.created_by, t.created_at, t.updated_at, \
         COALESCE(SUM(CASE WHEN i.item_type = 'DEBIT' THEN i.amount_minor ELSE 0 END), 0) AS total_amount_minor, \
         COALESCE(MIN(i.currency), 'TWD') AS currency \
         FROM transactions t LEFT JOIN transaction_items i ON i.transaction_id = t.transaction_id \
         WHERE t.transaction_code LIKE ? OR t.description LIKE ? \
         GROUP BY t.transaction_id \
         ORDER BY t.transaction_date DESC, t.created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default()
}

/// Count of transactions matching the search term (P1 pagination math).
async fn count_transactions(state: &AppState, search: &str) -> i64 {
    if search.is_empty() {
        return sqlx::query_scalar("SELECT COUNT(*) FROM transactions")
            .fetch_one(&state.pool)
            .await
            .unwrap_or(0);
    }
    let pattern = format!("%{search}%");
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM transactions WHERE transaction_code LIKE ? OR description LIKE ?",
    )
    .bind(&pattern)
    .bind(&pattern)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0)
}

#[derive(Deserialize)]
pub struct TransactionsQuery {
    pub page: Option<String>,
    pub q: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TxListRow {
    pub id: String,
    pub code: String,
    pub description: String,
    pub date: String,
    pub amount_display: String,
    pub currency: String,
    pub status: String,
    pub status_badge: String,
}

#[derive(Template)]
#[template(path = "transactions.html")]
pub struct TransactionsTemplate {
    pub acting_user: String,
    pub query: String,
    pub rows: Vec<TxListRow>,
    pub total: i64,
    pub page: u32,
    pub total_pages: u32,
    pub prev_page: Option<String>,
    pub next_page: Option<String>,
}

/// GET /transactions - dedicated list page (P1, UI-UX E3.4): search + paging +
/// links to detail. Read-only; the search term is a bound parameter.
async fn transactions_page(
    State(state): State<AppState>,
    session: Extension<Session>,
    Query(params): Query<TransactionsQuery>,
) -> Response {
    let search = params.q.unwrap_or_default().trim().to_string();
    let total = count_transactions(&state, &search).await;
    let total_pages = ((total + LIST_PAGE_SIZE - 1) / LIST_PAGE_SIZE).max(1);
    let page = clamp_page(parse_page_param(params.page.as_deref()), total_pages);
    let offset = (page - 1) * LIST_PAGE_SIZE;

    let rows: Vec<TxListRow> = query_transactions_for_list(&state, &search, LIST_PAGE_SIZE, offset)
        .await
        .iter()
        .map(|s| TxListRow {
            id: s.transaction.transaction_id.clone(),
            code: s.transaction.transaction_code.clone(),
            description: s.transaction.description.clone(),
            date: s.transaction.transaction_date.to_string(),
            amount_display: format_minor(s.total_amount_minor, 2),
            currency: s.currency.clone(),
            status: s.transaction.status.clone(),
            status_badge: status_badge(&s.transaction.status).to_string(),
        })
        .collect();

    let page_link = |p: i64| -> String {
        if search.is_empty() {
            format!("/transactions?page={p}")
        } else {
            format!("/transactions?page={p}&q={}", percent_encode(&search))
        }
    };
    let prev_page = (page > 1).then(|| page_link(page - 1));
    let next_page = (page < total_pages).then(|| page_link(page + 1));

    render_or_500(TransactionsTemplate {
        acting_user: session.user_id.clone(),
        query: search,
        rows,
        total,
        page: page as u32,
        total_pages: total_pages as u32,
        prev_page,
        next_page,
    })
}

#[derive(Debug, Clone)]
pub struct TxItemRow {
    pub kind: String,
    pub kind_badge: String,
    pub account: String,
    pub amount_display: String,
    pub currency: String,
    pub rate_display: String,
}

#[derive(Template)]
#[template(path = "transaction_detail.html")]
pub struct TransactionDetailTemplate {
    pub acting_user: String,
    pub not_found: bool,
    pub tx_id: String,
    pub code: String,
    pub description: String,
    pub date: String,
    pub status: String,
    pub status_badge: String,
    pub created_by: String,
    pub created_at: String,
    pub reference: String,
    pub source: String,
    pub items: Vec<TxItemRow>,
    pub debit_total: String,
    pub credit_total: String,
    pub balanced: bool,
}

/// GET /transactions/:id - detail view (P2, UI-UX E3.4): header, entry legs and
/// the debit=credit check. Missing ids render the three-part not-found state
/// with HTTP 404 (FR-004 message shape).
async fn transaction_detail_page(
    State(state): State<AppState>,
    session: Extension<Session>,
    Path(id): Path<String>,
) -> Response {
    // (id, code, date, description, status, created_by, created_at,
    // reference, source) — the header fields of the detail view.
    type TxHeaderRow = (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    );

    let header: Option<TxHeaderRow> = sqlx::query_as(
        "SELECT transaction_id, transaction_code, transaction_date, description, status, \
             created_by, created_at, COALESCE(reference_number, ''), COALESCE(source_type, '') \
             FROM transactions WHERE transaction_id = ? LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let Some((tx_id, code, date, description, status, created_by, created_at, reference, source)) =
        header
    else {
        let not_found = TransactionDetailTemplate {
            acting_user: session.user_id.clone(),
            not_found: true,
            tx_id: id,
            code: String::new(),
            description: String::new(),
            date: String::new(),
            status: String::new(),
            status_badge: "badge-neutral".to_string(),
            created_by: String::new(),
            created_at: String::new(),
            reference: String::new(),
            source: String::new(),
            items: Vec::new(),
            debit_total: "—".to_string(),
            credit_total: "—".to_string(),
            balanced: false,
        };
        return (StatusCode::NOT_FOUND, render_or_500(not_found)).into_response();
    };

    let raw_items: Vec<(String, String, i64, String, i64)> = sqlx::query_as(
        "SELECT item_type, account_id, amount_minor, currency, exchange_rate \
         FROM transaction_items WHERE transaction_id = ? ORDER BY item_type DESC, item_id",
    )
    .bind(&tx_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut debit_sum: i128 = 0;
    let mut credit_sum: i128 = 0;
    let items: Vec<TxItemRow> = raw_items
        .iter()
        .map(|(kind, account, amount, currency, rate)| {
            if kind == "DEBIT" {
                debit_sum += i128::from(*amount);
            } else {
                credit_sum += i128::from(*amount);
            }
            TxItemRow {
                kind: if kind == "DEBIT" { "借方" } else { "貸方" }.to_string(),
                kind_badge: if kind == "DEBIT" {
                    "badge-primary"
                } else {
                    "badge-info"
                }
                .to_string(),
                account: account.clone(),
                amount_display: format_minor(*amount, 2),
                currency: currency.clone(),
                rate_display: format_minor(*rate, 4),
            }
        })
        .collect();
    let balanced = debit_sum == credit_sum;

    render_or_500(TransactionDetailTemplate {
        acting_user: session.user_id.clone(),
        not_found: false,
        tx_id,
        code,
        description,
        date,
        status: status.clone(),
        status_badge: status_badge(&status).to_string(),
        created_by,
        created_at,
        reference,
        source,
        items,
        debit_total: flaps_core::money::format_minor_i128(debit_sum, 2),
        credit_total: flaps_core::money::format_minor_i128(credit_sum, 2),
        balanced,
    })
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

// ---------------------------------------------------------------------------
// Slice 8 Layer A: CSV import pipeline (two-phase, traceable, IMP-001~003)
// ---------------------------------------------------------------------------

const IMPORT_MAX_BYTES: usize = 2 * 1024 * 1024;
const IMPORT_MAX_ROWS: usize = 5_000;

/// One validated, confirmable import row (persisted as JSON in import_rows).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportRowDraft {
    pub date: String,
    pub debit_account: String,
    pub credit_account: String,
    pub amount_minor: i64,
    pub currency: String,
    pub description: String,
}

/// View model for the import page's batch report.
#[derive(Debug, Clone)]
pub struct ImportBatchRowView {
    pub row_no: i64,
    pub status: String,
    pub status_display: String,
    pub error_msg: String,
    pub raw_line: String,
}

/// View model for the import page's batch header.
#[derive(Debug, Clone)]
pub struct ImportBatchView {
    pub batch_id: String,
    pub sha256_short: String,
    pub row_total: i64,
    pub row_ok: i64,
    pub status: String,
    pub status_display: String,
    pub is_draft: bool,
    pub rows: Vec<ImportBatchRowView>,
}

/// Server-rendered import page (upload + optional batch report).
#[derive(Template)]
#[template(path = "import.html")]
pub struct ImportTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub notice: Option<String>,
    pub batch: Option<ImportBatchView>,
}

async fn import_upload_page(
    State(state): State<AppState>,
    session: Extension<Session>,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    let batch = match query.get("batch") {
        Some(id) => load_batch_view(&state, id).await,
        None => None,
    };
    let notice = query.get("notice").map(|n| {
        let bytes = n.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                if let Ok(v) = u8::from_str_radix(&n[i + 1..i + 3], 16) {
                    out.push(v);
                    i += 3;
                    continue;
                }
            }
            out.push(bytes[i]);
            i += 1;
        }
        String::from_utf8_lossy(&out).to_string()
    });
    let template = ImportTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        notice,
        batch,
    };
    match template.render() {
        Ok(body) => Html(body).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("頁面渲染失敗：{e}"),
        )
            .into_response(),
    }
}

/// Load a batch (header + rows) for rendering.
async fn load_batch_view(state: &AppState, batch_id: &str) -> Option<ImportBatchView> {
    let (file_sha256, row_total, row_ok, status): (String, i64, i64, String) = sqlx::query_as(
        "SELECT file_sha256, row_total, row_ok, status FROM import_batches WHERE batch_id = ?",
    )
    .bind(batch_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten()?;

    let rows: Vec<(i64, String, Option<String>, String)> = sqlx::query_as(
        "SELECT row_no, status, error_msg, raw_line FROM import_rows WHERE batch_id = ? ORDER BY row_no",
    )
    .bind(batch_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    Some(ImportBatchView {
        batch_id: batch_id.to_string(),
        sha256_short: file_sha256.chars().take(12).collect(),
        row_total,
        row_ok,
        status: status.clone(),
        status_display: if status == "draft" {
            "待確認（未登帳）".to_string()
        } else {
            "已確認登帳".to_string()
        },
        is_draft: status == "draft",
        rows: rows
            .into_iter()
            .map(|(row_no, st, err, raw)| ImportBatchRowView {
                row_no,
                status: st.clone(),
                status_display: if st == "ok" {
                    "合格".to_string()
                } else {
                    "錯誤".to_string()
                },
                error_msg: err.unwrap_or_default(),
                raw_line: raw.chars().take(80).collect(),
            })
            .collect(),
    })
}

/// Validate one CSV record into an import draft (rules = same as the form).
fn validate_import_row(record: &csv::StringRecord) -> Result<ImportRowDraft, String> {
    if record.len() < 6 {
        return Err(format!(
            "欄位不足（需 6 欄：date, debit_account, credit_account, amount, currency, description），實得 {} 欄",
            record.len()
        ));
    }
    let date = chrono::NaiveDate::parse_from_str(record.get(0).unwrap().trim(), "%Y-%m-%d")
        .map_err(|_| format!("日期格式不正確：{}", record.get(0).unwrap()))?
        .to_string();
    if chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").expect("date already validated")
        > chrono::Utc::now().date_naive()
    {
        return Err("日期不得為未來".to_string());
    }
    let debit_account = record.get(1).unwrap().trim().to_string();
    if debit_account.is_empty() || debit_account.len() > MAX_ACCOUNT_ID_LEN {
        return Err("借方科目空白或過長".to_string());
    }
    let credit_account = record.get(2).unwrap().trim().to_string();
    if credit_account.is_empty() || credit_account.len() > MAX_ACCOUNT_ID_LEN {
        return Err("貸方科目空白或過長".to_string());
    }
    if debit_account.eq_ignore_ascii_case(&credit_account) {
        return Err("借貸科目相同".to_string());
    }
    let amount_minor = flaps_core::parse_decimal_to_minor(record.get(3).unwrap().trim(), 2)
        .map_err(|e| format!("金額錯誤：{e}"))?;
    if amount_minor <= 0 {
        return Err("金額必須大於 0".to_string());
    }
    let currency = record.get(4).unwrap().trim().to_uppercase();
    if !SUPPORTED_CURRENCIES.contains(&currency.as_str()) {
        return Err(format!("幣別不支援：{currency}"));
    }
    let description = record.get(5).unwrap().trim().to_string();
    if description.is_empty() || description.len() > MAX_DESCRIPTION_LEN {
        return Err("摘要空白或過長".to_string());
    }
    Ok(ImportRowDraft {
        date,
        debit_account,
        credit_account,
        amount_minor,
        currency,
        description,
    })
}

/// POST /import/preview — parse + validate + persist a draft batch (no ledger
/// side effects). IMP-001.
async fn import_preview(
    State(state): State<AppState>,
    session: Extension<Session>,
    mut multipart: axum::extract::Multipart,
) -> Response {
    if !state
        .permissions
        .has_permission_db(&session.user_id, "LEDGER_WRITE")
        .await
    {
        return (
            StatusCode::FORBIDDEN,
            "需要 LEDGER_WRITE 權限才能匯入。".to_string(),
        )
            .into_response();
    }

    let mut bytes: Option<Vec<u8>> = None;
    let mut form_csrf: Option<String> = None;
    while let Some(field) = match multipart.next_field().await {
        Ok(f) => f,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("上傳格式錯誤：{e}")).into_response();
        }
    } {
        match field.name() {
            Some("csrf_token") => {
                form_csrf = field.text().await.ok();
            }
            Some("file") => match field.bytes().await {
                Ok(b) => bytes = Some(b.to_vec()),
                Err(e) => {
                    return (StatusCode::BAD_REQUEST, format!("讀取檔案失敗：{e}")).into_response()
                }
            },
            _ => {}
        }
    }
    // CSRF (T-08): multipart forms carry the synchronizer token too.
    if form_csrf.as_deref() != Some(&session.csrf_token) {
        return (
            StatusCode::FORBIDDEN,
            "CSRF 驗證失敗；請重新整理頁面後再操作。".to_string(),
        )
            .into_response();
    }
    let Some(data) = bytes else {
        return (StatusCode::BAD_REQUEST, "缺少檔案欄位 file。".to_string()).into_response();
    };
    if data.len() > IMPORT_MAX_BYTES {
        return (
            StatusCode::BAD_REQUEST,
            format!("檔案超過 2MB 上限（{} bytes）。", data.len()),
        )
            .into_response();
    }

    use sha2::Digest;
    let sha = sha2::Sha256::digest(&data);
    let sha_hex = format!("{sha:x}");

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(data.as_slice());

    let batch_id = uuid::Uuid::new_v4().to_string();
    let mut drafts: Vec<(usize, ImportRowDraft)> = Vec::new();
    let mut errors: Vec<(usize, String, String)> = Vec::new();

    for (idx, record) in reader.records().enumerate() {
        if drafts.len() + errors.len() >= IMPORT_MAX_ROWS {
            errors.push((
                idx + 2,
                String::new(),
                format!("超過 {IMPORT_MAX_ROWS} 列上限，其餘列未處理"),
            ));
            break;
        }
        let line_no = idx + 2; // 1-based, after the header row
        match record {
            Ok(rec) => {
                let raw = rec.iter().collect::<Vec<_>>().join(",");
                match validate_import_row(&rec) {
                    Ok(draft) => drafts.push((line_no, draft)),
                    Err(msg) => errors.push((line_no, raw, msg)),
                }
            }
            Err(e) => errors.push((line_no, String::new(), format!("CSV 解析失敗：{e}"))),
        }
    }

    let row_total = (drafts.len() + errors.len()) as i64;
    let row_ok = drafts.len() as i64;

    if let Err(e) = persist_batch(
        &state,
        ImportBatchStaging {
            batch_id: &batch_id,
            sha_hex: &sha_hex,
            row_total,
            row_ok,
            created_by: &session.user_id,
            drafts: &drafts,
            errors: &errors,
        },
    )
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("批次暫存失敗：{e}"),
        )
            .into_response();
    }

    Redirect::to(&format!("/import?batch={batch_id}")).into_response()
}

/// Staging payload for [`persist_batch`] — packs the batch header and the
/// per-row parse results so the function takes two arguments (P2-1 close-out).
struct ImportBatchStaging<'a> {
    batch_id: &'a str,
    sha_hex: &'a str,
    row_total: i64,
    row_ok: i64,
    created_by: &'a str,
    drafts: &'a [(usize, ImportRowDraft)],
    errors: &'a [(usize, String, String)],
}

async fn persist_batch(
    state: &AppState,
    staging: ImportBatchStaging<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO import_batches (batch_id, file_sha256, row_total, row_ok, status, created_by) \
         VALUES (?, ?, ?, ?, 'draft', ?)",
    )
    .bind(staging.batch_id)
    .bind(staging.sha_hex)
    .bind(staging.row_total)
    .bind(staging.row_ok)
    .bind(staging.created_by)
    .execute(&state.pool)
    .await?;
    for (line_no, draft) in staging.drafts {
        sqlx::query(
            "INSERT INTO import_rows (batch_id, row_no, raw_line, status, draft_json) \
             VALUES (?, ?, '', 'ok', ?)",
        )
        .bind(staging.batch_id)
        .bind(*line_no as i64)
        .bind(serde_json::to_string(draft).unwrap_or_default())
        .execute(&state.pool)
        .await?;
    }
    for (line_no, raw, msg) in staging.errors {
        sqlx::query(
            "INSERT INTO import_rows (batch_id, row_no, raw_line, status, error_msg, draft_json) \
             VALUES (?, ?, ?, 'error', ?, '{}')",
        )
        .bind(staging.batch_id)
        .bind(*line_no as i64)
        .bind(raw)
        .bind(msg)
        .execute(&state.pool)
        .await?;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ConfirmImportForm {
    pub csrf_token: String,
    pub batch_id: String,
}

/// POST /import/confirm — post all valid rows of a draft batch. IMP-002/003.
async fn import_confirm(
    State(state): State<AppState>,
    session: Extension<Session>,
    Form(form): Form<ConfirmImportForm>,
) -> Response {
    if !state
        .permissions
        .has_permission_db(&session.user_id, "LEDGER_WRITE")
        .await
    {
        return (
            StatusCode::FORBIDDEN,
            "需要 LEDGER_WRITE 權限。".to_string(),
        )
            .into_response();
    }
    if form.csrf_token != session.csrf_token {
        return (
            StatusCode::FORBIDDEN,
            "CSRF 驗證失敗；請重新整理頁面後再操作。".to_string(),
        )
            .into_response();
    }

    // IMP-003: mechanical guard against double-confirmation.
    let claimed = sqlx::query(
        "UPDATE import_batches SET status = 'confirmed' WHERE batch_id = ? AND status = 'draft'",
    )
    .bind(&form.batch_id)
    .execute(&state.pool)
    .await
    .map(|r| r.rows_affected())
    .unwrap_or(0);
    if claimed == 0 {
        return (
            StatusCode::CONFLICT,
            "此批次已確認或不存在；不可重複匯入。".to_string(),
        )
            .into_response();
    }

    let rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT row_no, draft_json FROM import_rows WHERE batch_id = ? AND status = 'ok' ORDER BY row_no",
    )
    .bind(&form.batch_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut posted = 0i64;
    let mut failed: Vec<String> = Vec::new();

    for (row_no, draft_json) in rows {
        let draft: Result<ImportRowDraft, _> = serde_json::from_str(&draft_json);
        let Ok(draft) = draft else {
            failed.push(format!("列 {row_no}：草稿資料毀損"));
            continue;
        };
        let date = chrono::NaiveDate::parse_from_str(&draft.date, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Utc::now().date_naive());
        let transaction_id = uuid::Uuid::new_v4().to_string();
        let transaction = match Transaction::builder()
            .transaction_id(transaction_id.clone())
            .transaction_code(format!("TX-{}", &transaction_id[..8]))
            .transaction_date(date)
            .description(draft.description.clone())
            .reference_number(Some(format!("import:{}", form.batch_id)))
            .status("POSTED")
            .source_type(Some("import".to_string()))
            .created_by(session.user_id.clone())
            .build()
        {
            Ok(t) => t,
            Err(e) => {
                failed.push(format!("列 {row_no}：交易建構失敗：{e}"));
                continue;
            }
        };
        let (debit_item, credit_item) = match (
            TransactionItem::builder()
                .item_id(uuid::Uuid::new_v4().to_string())
                .transaction_id(transaction_id.clone())
                .account_id(draft.debit_account.clone())
                .item_type("DEBIT")
                .amount_minor(draft.amount_minor)
                .currency(draft.currency.clone())
                .exchange_rate_scaled(10_000)
                .build(),
            TransactionItem::builder()
                .item_id(uuid::Uuid::new_v4().to_string())
                .transaction_id(transaction_id.clone())
                .account_id(draft.credit_account.clone())
                .item_type("CREDIT")
                .amount_minor(draft.amount_minor)
                .currency(draft.currency.clone())
                .exchange_rate_scaled(10_000)
                .build(),
        ) {
            (Ok(d), Ok(c)) => (d, c),
            (Err(e), _) | (_, Err(e)) => {
                failed.push(format!("列 {row_no}：分錄建構失敗：{e}"));
                continue;
            }
        };
        let items = vec![debit_item, credit_item];
        match state.engine.post_journal_entry(transaction, items).await {
            Ok(()) => posted += 1,
            Err(e) => failed.push(format!("列 {row_no}：{e}")),
        }
    }

    // Batch-level audit trail (transaction_id NULL is allowed by the schema).
    let _ = sqlx::query(
        "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) \
         VALUES (?, NULL, 'IMPORT_CONFIRMED', ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(format!(
        "{{\"batch_id\":\"{}\",\"posted\":{posted},\"failed\":{}}}",
        form.batch_id,
        failed.len()
    ))
    .bind(&session.user_id)
    .execute(&state.pool)
    .await;

    let summary = if failed.is_empty() {
        format!("匯入完成：{posted} 筆登帳。")
    } else {
        format!(
            "匯入部分完成：{posted} 筆登帳、{} 筆失敗（{}）。失敗列未登帳，可修正後重新匯入。",
            failed.len(),
            failed.join("；")
        )
    };
    Redirect::to(&format!(
        "/import?batch={}&notice={}",
        form.batch_id,
        urlencoding_lite(&summary)
    ))
    .into_response()
}

/// Minimal percent-encoding for query-string values (digits and safe ASCII
/// pass through; everything else becomes %XX). Zero-dependency by design.
fn urlencoding_lite(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Slice 8 Layer B: guided wizard (server knows the rules; WIZ-001)
// ---------------------------------------------------------------------------

/// Account keyword→account mapping for the parser: built-in defaults first,
/// then erp_config rows (category='parser') as administrator overrides.
async fn parser_keywords(state: &AppState) -> Vec<(String, String)> {
    let mut map: Vec<(String, String)> = vec![
        ("現金".to_string(), "CASH".to_string()),
        ("銀行".to_string(), "BANK".to_string()),
        ("租金".to_string(), "RENT-EXP".to_string()),
        ("薪資".to_string(), "PAYROLL-EXP".to_string()),
    ];
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT config_key, config_value FROM erp_config WHERE config_category = 'parser'",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    map.extend(rows);
    map
}

/// A recent draft row for the drafts page.
#[derive(Debug, Clone)]
pub struct DraftListView {
    pub draft_id: String,
    pub created_display: String,
    pub source: String,
    pub completeness: String,
}

#[derive(Template)]
#[template(path = "drafts.html")]
pub struct DraftsTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub drafts: Vec<DraftListView>,
    pub error: Option<String>,
}

/// Three-step guided posting page (Layer B; step 1|2|3).
#[derive(Template)]
#[template(path = "wizard.html")]
pub struct WizardTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub step: i32,
    /// Selected transaction template code (cash-payment / cash-receipt /
    /// transfer / opening).
    pub template: String,
    pub accounts: Vec<String>,
    pub error: Option<String>,
    /// Prefill provenance note (B2: 每個預填標示來源).
    pub from_note: Option<String>,
    pub amount_display: String,
    pub form: FormValues,
}

/// GET /drafts — paste unstructured text; see recent drafts.
async fn drafts_page(State(state): State<AppState>, session: Extension<Session>) -> Response {
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT draft_id, source, draft_json FROM drafts WHERE created_by = ? ORDER BY created_at DESC LIMIT 10",
    )
    .bind(&session.user_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let drafts: Vec<DraftListView> = rows
        .into_iter()
        .map(|(draft_id, source, json)| {
            let parsed: Result<flaps_core::DraftEntry, _> = serde_json::from_str(&json);
            let completeness = match parsed {
                Ok(d) if d.is_complete() => "完整（可登帳）".to_string(),
                Ok(_) => "不完整（需人工補齊）".to_string(),
                Err(_) => "資料毀損".to_string(),
            };
            let created_display = source_display_time(&draft_id);
            DraftListView {
                draft_id,
                created_display,
                source,
                completeness,
            }
        })
        .collect();

    let template = DraftsTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        drafts,
        error: None,
    };
    match template.render() {
        Ok(body) => Html(body).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("頁面渲染失敗：{e}"),
        )
            .into_response(),
    }
}

/// Deterministic display time derived from the draft id (v4 uuid is random,
/// so we show the source and let the table ordering speak for recency).
fn source_display_time(_draft_id: &str) -> String {
    "本工作階段".to_string()
}

#[derive(Debug, Deserialize)]
pub struct DraftParseForm {
    pub csrf_token: String,
    pub text: String,
}

/// POST /drafts/parse — rule-based parse → store draft → 303 to wizard step 2.
async fn drafts_parse(
    State(state): State<AppState>,
    session: Extension<Session>,
    Form(form): Form<DraftParseForm>,
) -> Response {
    if form.csrf_token != session.csrf_token {
        return (
            StatusCode::FORBIDDEN,
            "CSRF 驗證失敗；請重新整理頁面後再操作。".to_string(),
        )
            .into_response();
    }
    let text = form.text.trim().to_string();
    if text.is_empty() || text.len() > 10_000 {
        return (
            StatusCode::BAD_REQUEST,
            "內容空白或超過 10,000 字元。".to_string(),
        )
            .into_response();
    }

    let keywords = parser_keywords(&state).await;
    let draft = flaps_core::parse_unstructured(&text, &keywords);
    let draft_id = uuid::Uuid::new_v4().to_string();

    let insert = sqlx::query(
        "INSERT INTO drafts (draft_id, source, draft_json, created_by) VALUES (?, 'text', ?, ?)",
    )
    .bind(&draft_id)
    .bind(serde_json::to_string(&draft).unwrap_or_default())
    .bind(&session.user_id)
    .execute(&state.pool)
    .await;

    match insert {
        Ok(_) => Redirect::to(&format!("/new?step=2&from_draft={draft_id}")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("草稿儲存失敗：{e}"),
        )
            .into_response(),
    }
}

/// Query params for the wizard.
#[derive(Debug, Default, Deserialize)]
pub struct WizardQuery {
    pub step: Option<String>,
    pub template: Option<String>,
    pub from_draft: Option<String>,
    pub from_batch: Option<String>,
    pub row_no: Option<i64>,
}

/// GET /new — three-step guided posting (B1/B2).
async fn wizard_page(
    State(state): State<AppState>,
    session: Extension<Session>,
    axum::extract::Query(query): axum::extract::Query<WizardQuery>,
) -> Response {
    let step = query.step.as_deref().unwrap_or("1");
    match step {
        "2" => {
            // Prefill from a draft or an import error row when requested (B2).
            let (prefilled, from_note) = if let Some(draft_id) = query.from_draft.as_deref() {
                let row: Option<(String,)> = sqlx::query_as(
                    "SELECT draft_json FROM drafts WHERE draft_id = ? AND created_by = ?",
                )
                .bind(draft_id)
                .bind(&session.user_id)
                .fetch_optional(&state.pool)
                .await
                .unwrap_or(None);
                match row
                    .and_then(|(json,)| serde_json::from_str::<flaps_core::DraftEntry>(&json).ok())
                {
                    Some(d) => (
                        FormValues {
                            debit_account: d.debit_account.clone().unwrap_or_default(),
                            credit_account: d.credit_account.clone().unwrap_or_default(),
                            amount: d
                                .amount_minor
                                .map(|m| format_minor(m, 2))
                                .unwrap_or_default(),
                            currency: "TWD".to_string(),
                            description: "（自草稿預填，請依據卡核對）".to_string(),
                            transaction_date: d.date.map(|d| d.to_string()).unwrap_or_default(),
                        },
                        Some("非結構化解析草稿（依據卡見草稿頁）".to_string()),
                    ),
                    None => (FormValues::default(), None),
                }
            } else if let (Some(batch_id), Some(row_no)) = (&query.from_batch, query.row_no) {
                let row: Option<(String,)> = sqlx::query_as(
                    "SELECT draft_json FROM import_rows WHERE batch_id = ? AND row_no = ? AND status = 'error'",
                )
                .bind(batch_id)
                .bind(row_no)
                .fetch_optional(&state.pool)
                .await
                .unwrap_or(None);
                match row.and_then(|(json,)| serde_json::from_str::<ImportRowDraft>(&json).ok()) {
                    Some(d) => (
                        FormValues {
                            debit_account: d.debit_account.clone(),
                            credit_account: d.credit_account.clone(),
                            amount: flaps_core::format_minor(d.amount_minor, 2),
                            currency: d.currency.clone(),
                            description: d.description.clone(),
                            transaction_date: d.date.clone(),
                        },
                        Some(format!(
                            "匯入批次 {batch_id} 第 {row_no} 列（原錯誤列，請修正後送出）"
                        )),
                    ),
                    None => (FormValues::default(), None),
                }
            } else {
                (FormValues::default(), None)
            };

            let template = query
                .template
                .clone()
                .unwrap_or_else(|| "cash-payment".to_string());
            let accounts: Vec<String> =
                sqlx::query_scalar("SELECT account_id FROM accounts ORDER BY account_id LIMIT 100")
                    .fetch_all(&state.pool)
                    .await
                    .unwrap_or_default();

            let rendered = WizardTemplate {
                acting_user: session.user_id.clone(),
                csrf_token: session.csrf_token.clone(),
                step: 2,
                template,
                accounts,
                error: None,
                from_note,
                amount_display: String::new(),
                form: prefilled,
            };
            render_or_500(rendered)
        }
        "3" => {
            // Step 3 requires the values from step 2 in the query string; the
            // step-2 form carries them as hidden inputs via POST — to keep the
            // flow zero-JS and stateless we accept a POST at /new/step2 that
            // renders step 3 directly (see wizard_step2).
            redirect_to_wizard_step1()
        }
        _ => {
            let rendered = WizardTemplate {
                acting_user: session.user_id.clone(),
                csrf_token: session.csrf_token.clone(),
                step: 1,
                template: String::new(),
                accounts: Vec::new(),
                error: None,
                from_note: None,
                amount_display: String::new(),
                form: FormValues::default(),
            };
            render_or_500(rendered)
        }
    }
}

fn redirect_to_wizard_step1() -> Response {
    Redirect::to("/new?step=1").into_response()
}

fn render_or_500(template: impl askama::Template) -> Response {
    match template.render() {
        Ok(body) => Html(body).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("頁面渲染失敗：{e}"),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct WizardStep2Form {
    pub csrf_token: String,
    pub template: String,
    pub debit_account: String,
    pub credit_account: String,
    pub amount: String,
    pub currency: String,
    pub description: String,
    pub transaction_date: String,
}

/// POST /new/step2 — validate step 2, render the step-3 confirmation.
async fn wizard_step2(
    State(state): State<AppState>,
    session: Extension<Session>,
    Form(form): Form<WizardStep2Form>,
) -> Response {
    if form.csrf_token != session.csrf_token {
        return (
            StatusCode::FORBIDDEN,
            "CSRF 驗證失敗；請重新整理頁面後再操作。".to_string(),
        )
            .into_response();
    }

    let mut form_error: Option<String> = None;
    let mut amount_minor: Option<i64> = None;

    let debit = form.debit_account.trim().to_string();
    let credit = form.credit_account.trim().to_string();
    if debit.is_empty() || credit.is_empty() {
        form_error = Some("借貸科目皆為必填（規則 R-BALANCE）。".to_string());
    } else if debit.eq_ignore_ascii_case(&credit) {
        form_error = Some("借貸兩方不得為同一科目（規則 R-BALANCE）。".to_string());
    } else if debit.len() > MAX_ACCOUNT_ID_LEN || credit.len() > MAX_ACCOUNT_ID_LEN {
        form_error = Some("科目代碼過長（規則 R-ACCT-LEN）。".to_string());
    }

    if form_error.is_none() {
        match flaps_core::parse_decimal_to_minor(form.amount.trim(), 2) {
            Ok(m) if m > 0 => amount_minor = Some(m),
            Ok(_) => form_error = Some("金額必須大於 0（規則 R-MONEY）。".to_string()),
            Err(e) => form_error = Some(format!("金額錯誤（規則 R-MONEY）：{e}")),
        }
    }

    if form_error.is_none() && !SUPPORTED_CURRENCIES.contains(&form.currency.as_str()) {
        form_error = Some("幣別不支援（規則 R-CURRENCY）。".to_string());
    }

    if form_error.is_none() {
        match chrono::NaiveDate::parse_from_str(form.transaction_date.trim(), "%Y-%m-%d") {
            Ok(d) if d > chrono::Utc::now().date_naive() => {
                form_error = Some("日期不得為未來（規則 R-DATE-FUTURE）。".to_string());
            }
            Err(_) => {
                form_error = Some("日期格式不正確（規則 R-DATE-FORMAT）。".to_string());
            }
            _ => {}
        }
    }

    if let Some(err) = form_error {
        let accounts: Vec<String> =
            sqlx::query_scalar("SELECT account_id FROM accounts ORDER BY account_id LIMIT 100")
                .fetch_all(&state.pool)
                .await
                .unwrap_or_default();
        let rendered = WizardTemplate {
            acting_user: session.user_id.clone(),
            csrf_token: session.csrf_token.clone(),
            step: 2,
            template: form.template.clone(),
            accounts,
            error: Some(err),
            from_note: None,
            amount_display: String::new(),
            form: FormValues {
                debit_account: debit,
                credit_account: credit,
                amount: form.amount.clone(),
                currency: form.currency.clone(),
                description: form.description.clone(),
                transaction_date: form.transaction_date.clone(),
            },
        };
        return render_or_500(rendered);
    }

    let amount_minor = amount_minor.unwrap_or(0);
    let rendered = WizardTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        step: 3,
        template: form.template.clone(),
        accounts: Vec::new(),
        error: None,
        from_note: None,
        amount_display: format_minor(amount_minor, 2),
        form: FormValues {
            debit_account: debit,
            credit_account: credit,
            amount: form.amount.trim().to_string(),
            currency: form.currency.clone(),
            description: form.description.trim().to_string(),
            transaction_date: form.transaction_date.trim().to_string(),
        },
    };
    render_or_500(rendered)
}

// ---------------------------------------------------------------------------
// Slice 8 Layer D: monthly report + CSV export (RPT-001)
// ---------------------------------------------------------------------------

/// Validated `YYYY-MM` from a query param (reject anything else — 可推導).
fn parse_year_month(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.len() != 7 || !raw.as_bytes()[4] == b'-' {
        return None;
    }
    let mut parts = raw.split('-');
    let y = parts.next()?;
    let m = parts.next()?;
    if y.len() != 4 || m.len() != 2 {
        return None;
    }
    let y: u32 = y.parse().ok()?;
    let m: u32 = m.parse().ok()?;
    if !(1..=12).contains(&m) {
        return None;
    }
    Some(format!("{y:04}-{m:02}"))
}

#[derive(Template)]
#[template(path = "report_monthly.html")]
pub struct MonthlyReportTemplate {
    pub acting_user: String,
    pub ym: String,
    pub currency: String,
    pub rows: Vec<ReportRow>,
    pub total_debit_display: String,
    pub total_credit_display: String,
    pub total_net_display: String,
}

#[derive(Debug, Clone)]
pub struct ReportRow {
    pub account_id: String,
    pub debit_display: String,
    pub credit_display: String,
    pub net_display: String,
}

/// GET /reports/monthly — per-account monthly totals (LEDGER_READ).
async fn report_monthly_page(
    State(state): State<AppState>,
    session: Extension<Session>,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    if !state
        .permissions
        .has_permission_db(&session.user_id, "LEDGER_READ")
        .await
    {
        return (StatusCode::FORBIDDEN, "需要 LEDGER_READ 權限。".to_string()).into_response();
    }
    let Some(ym) = query.get("ym").and_then(|y| parse_year_month(y)) else {
        return (
            StatusCode::BAD_REQUEST,
            "月份參數格式不正確；請使用 ym=YYYY-MM。".to_string(),
        )
            .into_response();
    };

    let rows = state
        .engine
        .monthly_by_account(&ym)
        .await
        .unwrap_or_default();
    let report_rows: Vec<ReportRow> = rows
        .iter()
        .map(|r| ReportRow {
            account_id: r.account_id.clone(),
            debit_display: format_minor(r.debit_minor, 2),
            credit_display: format_minor(r.credit_minor, 2),
            net_display: format_minor(r.debit_minor - r.credit_minor, 2),
        })
        .collect();
    let total_debit: i64 = rows.iter().map(|r| r.debit_minor).sum();
    let total_credit: i64 = rows.iter().map(|r| r.credit_minor).sum();

    let template = MonthlyReportTemplate {
        acting_user: session.user_id.clone(),
        ym,
        currency: "TWD".to_string(),
        rows: report_rows,
        total_debit_display: format_minor(total_debit, 2),
        total_credit_display: format_minor(total_credit, 2),
        total_net_display: format_minor(total_debit - total_credit, 2),
    };
    render_or_500(template)
}

/// GET /reports/monthly.csv — machine-readable export (same RBAC).
async fn report_monthly_csv(
    State(state): State<AppState>,
    session: Extension<Session>,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    if !state
        .permissions
        .has_permission_db(&session.user_id, "LEDGER_READ")
        .await
    {
        return (StatusCode::FORBIDDEN, "需要 LEDGER_READ 權限。".to_string()).into_response();
    }
    let Some(ym) = query.get("ym").and_then(|y| parse_year_month(y)) else {
        return (
            StatusCode::BAD_REQUEST,
            "月份參數格式不正確；請使用 ym=YYYY-MM。".to_string(),
        )
            .into_response();
    };

    let rows = state
        .engine
        .monthly_by_account(&ym)
        .await
        .unwrap_or_default();
    let mut csv_text = String::from("account_id,debit_minor,credit_minor,net_minor\n");
    for r in &rows {
        csv_text.push_str(&format!(
            "{},{},{},{}\n",
            r.account_id,
            r.debit_minor,
            r.credit_minor,
            r.debit_minor - r.credit_minor
        ));
    }

    (
        [(header::CONTENT_TYPE, "text/csv; charset=utf-8")],
        [(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"monthly-{ym}.csv\""),
        )],
        csv_text,
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Slice 10 Layer E: multidimensional pivot (PIV-001/002, whitelisted SQL)
// ---------------------------------------------------------------------------

/// Whitelisted dimension expressions. Any value outside this table is
/// rejected with R-PIV-WHITELIST (SHARED_STANDARDS: dynamic columns must be
/// whitelisted — never interpolated from user input).
fn dim_expr(dim: &str) -> Option<&'static str> {
    match dim {
        "account" => Some("i.account_id"),
        "month" => Some("strftime('%Y-%m', t.transaction_date)"),
        "direction" => Some("i.item_type"),
        _ => None,
    }
}

/// Whitelisted measure expressions.
fn measure_expr(measure: &str) -> Option<&'static str> {
    match measure {
        "debit" => Some(
            "COALESCE(SUM(CASE WHEN i.item_type = 'DEBIT' THEN i.amount_minor ELSE 0 END), 0)",
        ),
        "credit" => Some(
            "COALESCE(SUM(CASE WHEN i.item_type = 'CREDIT' THEN i.amount_minor ELSE 0 END), 0)",
        ),
        "net" => Some(
            "COALESCE(SUM(CASE WHEN i.item_type = 'DEBIT' THEN i.amount_minor ELSE -i.amount_minor END), 0)",
        ),
        _ => None,
    }
}

/// One pivot matrix cell.
#[derive(Debug, Clone)]
pub struct PivotCell {
    pub display: String,
}

/// Rendered pivot matrix.
#[derive(Debug, Clone, Default)]
pub struct PivotMatrix {
    pub col_headers: Vec<String>,
    pub row_headers: Vec<String>,
    /// cells[row_index][col_index]
    pub cells: Vec<Vec<String>>,
    pub row_totals: Vec<String>,
    pub col_totals: Vec<String>,
    pub grand_total: String,
    /// OQ-10-2: true when row_keys exceeded the cap and folded into OTHER.
    pub rows_truncated: bool,
}

#[derive(Template)]
#[template(path = "report_pivot.html")]
pub struct PivotTemplate {
    pub acting_user: String,
    pub rows_dim: String,
    pub cols_dim: String,
    pub measure: String,
    pub ym: String,
    pub matrix: PivotMatrix,
}

/// GET /reports/pivot — whitelisted multidimensional matrix (LEDGER_READ).
async fn report_pivot_page(
    State(state): State<AppState>,
    session: Extension<Session>,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    if !state
        .permissions
        .has_permission_db(&session.user_id, "LEDGER_READ")
        .await
    {
        return (StatusCode::FORBIDDEN, "需要 LEDGER_READ 權限。".to_string()).into_response();
    }

    let rows_dim = query.get("rows").map(|s| s.as_str()).unwrap_or("account");
    let cols_dim = query.get("cols").map(|s| s.as_str()).unwrap_or("month");
    let measure = query.get("measure").map(|s| s.as_str()).unwrap_or("debit");
    let ym = query.get("ym").and_then(|y| parse_year_month(y));

    // PIV-002: whitelist gate (fail-closed, rule named).
    let (Some(rows_expr), Some(cols_expr), Some(measure_expr)) = (
        dim_expr(rows_dim),
        dim_expr(cols_dim),
        measure_expr(measure),
    ) else {
        return (
            StatusCode::BAD_REQUEST,
            "樞紐維度不在白名單（規則 R-PIV-WHITELIST：rows/cols 僅限 account、month、direction；measure 僅限 debit、credit、net）。".to_string(),
        )
            .into_response();
    };
    if rows_dim == cols_dim {
        return (
            StatusCode::BAD_REQUEST,
            "樞紐維度重複（規則 R-PIV-WHITELIST：rows 與 cols 必須不同）。".to_string(),
        )
            .into_response();
    }

    use sqlx::QueryBuilder;
    let mut qb: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new("SELECT ");
    qb.push(rows_expr);
    qb.push(" AS r, ");
    qb.push(cols_expr);
    qb.push(" AS c, ");
    qb.push(measure_expr);
    qb.push(
        " AS v FROM transaction_items i JOIN transactions t ON i.transaction_id = t.transaction_id",
    );
    if let Some(ref month) = ym {
        qb.push(" WHERE strftime('%Y-%m', t.transaction_date) = ");
        qb.push_bind(month.clone());
    }
    qb.push(" GROUP BY r, c");

    let raw_cells: Vec<(String, String, i64)> = qb
        .build_query_as()
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    let matrix = assemble_pivot(&raw_cells, measure, 20);
    let template = PivotTemplate {
        acting_user: session.user_id.clone(),
        rows_dim: rows_dim.to_string(),
        cols_dim: cols_dim.to_string(),
        measure: measure.to_string(),
        ym: ym.unwrap_or_else(|| "全部期間".to_string()),
        matrix,
    };
    render_or_500(template)
}

/// Assemble raw (row, col, value) triples into a full matrix with totals.
/// Pure function over sorted data — derivable and unit-testable.
fn assemble_pivot(raw: &[(String, String, i64)], measure: &str, max_rows: usize) -> PivotMatrix {
    use std::collections::BTreeMap;
    let mut row_keys: Vec<String> = Vec::new();
    let mut col_keys: Vec<String> = Vec::new();
    let mut cells: BTreeMap<(String, String), i64> = BTreeMap::new();
    for (r, c, v) in raw {
        if !row_keys.contains(r) {
            row_keys.push(r.clone());
        }
        if !col_keys.contains(c) {
            col_keys.push(c.clone());
        }
        *cells.entry((r.clone(), c.clone())).or_insert(0) += v;
    }
    row_keys.sort();
    col_keys.sort();
    // OQ-10-2: cap rows at max_rows (top by row total) so the matrix cannot
    // explode with thousands of accounts; truncated rows fold into an
    // explicit OTHER bucket (explainable, not silently dropped).
    let mut rows_truncated = false;
    if row_keys.len() > max_rows {
        rows_truncated = true;
        let mut ranked: Vec<(String, i64)> = row_keys
            .iter()
            .map(|r| {
                let total: i64 = col_keys
                    .iter()
                    .map(|c| cells.get(&(r.clone(), c.clone())).copied().unwrap_or(0))
                    .sum();
                (r.clone(), total)
            })
            .collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let dropped: Vec<String> = ranked[max_rows..].iter().map(|(r, _)| r.clone()).collect();
        row_keys = ranked[..max_rows].iter().map(|(r, _)| r.clone()).collect();
        for c in &col_keys {
            let sum: i64 = dropped
                .iter()
                .map(|r| cells.get(&(r.clone(), c.clone())).copied().unwrap_or(0))
                .sum();
            if sum != 0 {
                *cells.entry(("OTHER".to_string(), c.clone())).or_insert(0) += sum;
            }
        }
        row_keys.push("OTHER".to_string());
    }

    let fmt = |v: i64| -> String { format_minor(v, 2) };
    let mut row_totals: Vec<String> = Vec::new();
    let mut col_totals: Vec<i64> = vec![0; col_keys.len()];
    let mut grand: i64 = 0;
    let mut rendered_rows: Vec<Vec<String>> = Vec::new();

    for r in &row_keys {
        let mut row_cells: Vec<String> = Vec::new();
        let mut row_sum: i64 = 0;
        for (ci, c) in col_keys.iter().enumerate() {
            let v = cells.get(&(r.clone(), c.clone())).copied().unwrap_or(0);
            row_sum += v;
            col_totals[ci] += v;
            row_cells.push(fmt(v));
        }
        row_totals.push(fmt(row_sum));
        grand += row_sum;
        rendered_rows.push(row_cells);
    }
    let _ = measure;

    PivotMatrix {
        col_headers: col_keys,
        row_headers: row_keys,
        cells: rendered_rows,
        row_totals,
        col_totals: col_totals.iter().map(|v| fmt(*v)).collect(),
        grand_total: fmt(grand),
        rows_truncated,
    }
}

/// Query the data needed by the page and render the template. Any database
/// failure degrades to the disconnected layout (db_connected = false, FR-004)
/// instead of hiding the whole UI.
async fn build_index(
    state: &AppState,
    session: &Session,
    error: Option<String>,
    form: FormValues,
    requested_page: i64,
) -> Result<String, askama::Error> {
    let db_connected = sqlx::query("SELECT 1").fetch_one(&state.pool).await.is_ok();

    let chart_svg = if db_connected {
        let totals = state.engine.monthly_totals(6).await.unwrap_or_default();
        monthly_chart_svg(&totals)
    } else {
        String::new()
    };
    let current_ym = chrono::Utc::now().date_naive().format("%Y-%m").to_string();

    #[allow(clippy::type_complexity)]
    let (
        recent,
        transactions,
        accounts,
        transaction_count,
        account_count,
        total_amount_display,
        pending_review_count,
        page,
        total_pages,
        prev_page,
        next_page,
        total_transactions,
    ) = if db_connected {
        // M1 (blueprint §M1): server-side pagination, 20 rows per page. The
        // page number is parsed fail-closed and clamped against the real
        // count; LIMIT/OFFSET are bound parameters, never concatenated SQL.
        let transaction_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions")
            .fetch_one(&state.pool)
            .await
            .unwrap_or(0);
        let total_pages_i64 = ((transaction_count + LIST_PAGE_SIZE - 1) / LIST_PAGE_SIZE).max(1);
        let page_i64 = clamp_page(requested_page, total_pages_i64);
        let offset = (page_i64 - 1) * LIST_PAGE_SIZE;

        let summaries = state
            .engine
            .query_transaction_summaries_paged(LIST_PAGE_SIZE, offset)
            .await
            .unwrap_or_default();
        let transactions: Vec<TransactionRow> = summaries
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

        // Dashboard overview keeps the most recent entries (unpaged).
        let recent_summaries = state
            .engine
            .query_recent_transaction_summaries(RECENT_TRANSACTION_LIMIT)
            .await
            .unwrap_or_default();
        let recent: Vec<TransactionRow> = recent_summaries
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

        // M3 (blueprint §M3): 待覆核 = status 'PENDING_REVIEW'; real count,
        // 0 when none — never a placeholder dash.
        let pending_review_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM transactions WHERE status = 'PENDING_REVIEW'")
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

        let prev_page = if page_i64 > 1 {
            Some(format!("/?page={}", page_i64 - 1))
        } else {
            None
        };
        let next_page = if page_i64 < total_pages_i64 {
            Some(format!("/?page={}", page_i64 + 1))
        } else {
            None
        };

        (
            recent,
            transactions,
            account_ids,
            transaction_count.to_string(),
            account_count.to_string(),
            format_minor(total_minor, 2),
            pending_review_count.to_string(),
            page_i64 as u32,
            total_pages_i64 as u32,
            prev_page,
            next_page,
            transaction_count.to_string(),
        )
    } else {
        (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            "—".to_string(),
            "—".to_string(),
            "—".to_string(),
            "—".to_string(),
            1u32,
            1u32,
            None,
            None,
            "—".to_string(),
        )
    };

    // KPI strings are real counts on the connected path (M3); "—" only when
    // the database is unreachable (degraded layout, FR-004).
    IndexTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        can_manage_users: can_manage_users(state, &session.user_id).await,
        current_ym,
        chart_svg,
        db_connected,
        transaction_count,
        total_amount_display,
        account_count,
        pending_review_count,
        prev_page,
        page,
        total_pages,
        total_transactions,
        next_page,
        transactions,
        recent,
        accounts,
        error,
        form,
        // 跨模組狀態（Slice 13）
        search_index_count: state.search.document_count().await,
        sync_pending_count: 0,
        com_bridge_objects: 0,
        ai_models_available: 3,
        recent_audit_logs: {
            let rows: Vec<crate::cross_module_api::AuditRow> = sqlx::query_as(
                "SELECT al.log_id, al.performed_by, u.username, al.event_type, al.transaction_id, al.event_data, al.performed_at FROM audit_log al LEFT JOIN users u ON al.performed_by = u.user_id ORDER BY al.performed_at DESC LIMIT 10",
            )
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();
            rows.into_iter().map(|(log_id, performed_by, username, event_type, transaction_id, event_data, performed_at)| {
                AuditLogRow {
                    log_id,
                    user_id: performed_by,
                    username: username.unwrap_or_else(|| "unknown".to_string()),
                    action: event_type,
                    resource: transaction_id,
                    details: event_data,
                    created_at: performed_at,
                }
            }).collect()
        },
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
            "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) VALUES ('user-admin', 'admin', ?, 1)",
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

    /// Seed `count` minimal transactions with the given status (pagination and
    /// KPI tests need deterministic row counts, not full ledger entries).
    async fn seed_transactions(pool: &SqlitePool, count: i64, status: &str) {
        for i in 0..count {
            sqlx::query(
                "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, status, created_by) \
                 VALUES (?, ?, '2026-11-01', ?, ?, 'seed')",
            )
            .bind(format!("tx-seed-{i}"))
            .bind(format!("TX-SEED-{i}"))
            .bind(format!("seed row {i}"))
            .bind(status)
            .execute(pool)
            .await
            .expect("seed transaction");
        }
    }

    #[test]
    fn parse_page_param_is_fail_closed() {
        assert_eq!(parse_page_param(None), 1, "missing page defaults to 1");
        assert_eq!(parse_page_param(Some("")), 1, "empty page defaults to 1");
        assert_eq!(parse_page_param(Some("abc")), 1, "garbage defaults to 1");
        assert_eq!(parse_page_param(Some("0")), 1, "zero clamps to 1");
        assert_eq!(parse_page_param(Some("-3")), 1, "negative clamps to 1");
        assert_eq!(parse_page_param(Some("2")), 2, "valid page passes through");
        assert_eq!(parse_page_param(Some("99999999")), 1, "absurd falls back");
        assert_eq!(clamp_page(999, 2), 2, "past-the-end clamps to last page");
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
        assert!(
            body.contains("\"migrations_applied\":"),
            "health must report the applied migration count (M2)"
        );
        assert!(
            body.contains("\"transactions\":"),
            "health must report the transaction total (M2)"
        );
    }

    #[tokio::test]
    async fn health_degrades_to_503_when_db_unreachable() {
        let (app, pool) = test_app().await;
        pool.close().await;
        let (status, body, _) = send(
            app,
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::SERVICE_UNAVAILABLE,
            "unreachable DB must degrade to 503 (M2)"
        );
        assert!(
            body.contains("\"status\":\"degraded\""),
            "degraded status must be machine-readable"
        );
    }

    #[tokio::test]
    async fn pagination_splits_25_rows_into_20_plus_5() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, _) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;
        seed_transactions(&pool, 25, "POSTED").await;

        let request = Request::builder()
            .uri("/?page=1")
            .header(header::COOKIE, cookie.clone())
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body.contains("第 1 / 2 頁・共 25 筆"),
            "page 1 status line must render (M1)"
        );
        assert!(
            body.contains("href=\"/?page=2\""),
            "next-page link must be present on page 1"
        );

        let request = Request::builder()
            .uri("/?page=2")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body.contains("第 2 / 2 頁・共 25 筆"),
            "page 2 status line must render (M1)"
        );
        assert!(
            body.contains("href=\"/?page=1\""),
            "previous-page link must be present on page 2"
        );
    }

    #[tokio::test]
    async fn pagination_clamps_out_of_range_page_to_last() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, _) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;
        seed_transactions(&pool, 25, "POSTED").await;

        let request = Request::builder()
            .uri("/?page=999")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::OK, "clamped page must not 500");
        assert!(
            body.contains("第 2 / 2 頁・共 25 筆"),
            "page=999 must show the last page (M1)"
        );
    }

    #[tokio::test]
    async fn pagination_falls_back_to_page_1_on_garbage() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, _) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;
        seed_transactions(&pool, 25, "POSTED").await;

        for raw in ["abc", "0", "-3", ""] {
            let request = Request::builder()
                .uri(format!("/?page={raw}"))
                .header(header::COOKIE, cookie.clone())
                .body(Body::empty())
                .unwrap();
            let (status, body, _) = send(app.clone(), request).await;
            assert_eq!(status, StatusCode::OK, "page={raw:?} must not 500");
            assert!(
                body.contains("第 1 / 2 頁・共 25 筆"),
                "page={raw:?} must fall back to page 1 (M1)"
            );
        }
    }

    #[tokio::test]
    async fn pending_review_kpi_shows_real_count() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, _) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;
        seed_transactions(&pool, 2, "PENDING_REVIEW").await;

        let request = Request::builder()
            .uri("/")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body.contains("id=\"kpi-pending-review\">2</dd>"),
            "KPI must show the real pending-review count, never a dash (M3)"
        );
    }

    /// Seed two balanced legs (debit + credit) against known accounts.
    async fn seed_transaction_items(pool: &SqlitePool, tx_id: &str, amount_minor: i64) {
        for (account, name, kind) in [
            ("BANK-001", "Bank Account", "DEBIT"),
            ("SUPPLIES-EXP", "Supplies Expense", "CREDIT"),
        ] {
            sqlx::query(
                "INSERT OR IGNORE INTO accounts (account_id, account_code, account_name, account_type, currency, is_active) \
                 VALUES (?, ?, ?, 'ASSETS', 'TWD', 1)",
            )
            .bind(account)
            .bind(account)
            .bind(name)
            .execute(pool)
            .await
            .expect("seed account");
            sqlx::query(
                "INSERT INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency, exchange_rate) \
                 VALUES (?, ?, ?, ?, ?, 'TWD', 10000)",
            )
            .bind(format!("it-{}-{}", tx_id, kind.to_lowercase()))
            .bind(tx_id)
            .bind(account)
            .bind(kind)
            .bind(amount_minor)
            .execute(pool)
            .await
            .expect("seed item");
        }
    }

    #[test]
    fn percent_encode_handles_cjk_and_spaces() {
        assert_eq!(percent_encode("TX 2026"), "TX%202026");
        assert_eq!(percent_encode("文具"), "%E6%96%87%E5%85%B7");
        assert_eq!(percent_encode("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn status_badge_maps_known_statuses() {
        assert_eq!(status_badge("POSTED"), "badge-success");
        assert_eq!(status_badge("PENDING_REVIEW"), "badge-warning");
        assert_eq!(status_badge("VOID"), "badge-error");
        assert_eq!(status_badge("SOMETHING_ELSE"), "badge-info");
    }

    #[tokio::test]
    async fn transactions_list_searches_and_paginates() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, _) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;
        seed_transactions(&pool, 25, "POSTED").await;

        let request = Request::builder()
            .uri("/transactions")
            .header(header::COOKIE, cookie.clone())
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK, "list page must render");
        assert!(body.contains("第 1 / 2 頁・共 25 筆"), "list status line");
        assert!(
            body.contains("/transactions/tx-seed-"),
            "rows must link to detail pages"
        );

        // Search narrows to the 11 codes containing 'TX-SEED-1'.
        let request = Request::builder()
            .uri("/transactions?q=TX-SEED-1")
            .header(header::COOKIE, cookie.clone())
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("共 11 筆"), "search must narrow the count");

        // A term matching everything keeps two pages; links preserve `q`.
        let request = Request::builder()
            .uri("/transactions?q=seed")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("共 25 筆"), "bare term matches all rows");
        assert!(
            body.contains("q=seed"),
            "pagination links must preserve the search term"
        );
    }

    #[tokio::test]
    async fn transaction_detail_renders_items_and_404() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, _) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;
        sqlx::query(
            "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, status, created_by) \
             VALUES ('tx-detail-1', 'TX-DETAIL-1', '2026-11-05', 'detail test', 'POSTED', 'tester')",
        )
        .execute(&pool)
        .await
        .expect("seed detail tx");
        seed_transaction_items(&pool, "tx-detail-1", 12_345).await;

        let request = Request::builder()
            .uri("/transactions/tx-detail-1")
            .header(header::COOKIE, cookie.clone())
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK, "detail page must render");
        assert!(
            body.contains("借方") && body.contains("貸方"),
            "legs render"
        );
        assert!(body.contains("123.45"), "amounts render from minor units");
        assert!(
            body.contains("badge badge-success\">平衡</span>"),
            "balanced entries must show the balance badge"
        );
        assert!(body.contains("1.0000"), "exchange rate renders at 4 dp");

        let request = Request::builder()
            .uri("/transactions/tx-missing-999")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "missing tx must 404");
        assert!(
            body.contains("找不到交易"),
            "three-part not-found state (FR-004)"
        );
        assert!(
            body.contains("返回交易清單"),
            "recovery path must be offered"
        );
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

    // Slice 8 acceptance: import pipeline (IMP-001..003)
    // ------------------------------------------------------------------

    fn csv_multipart(csv_text: &str, csrf: &str) -> (header::HeaderValue, Body) {
        let boundary = "X-BOUNDARY-TEST";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{csrf}\r\n\
             --{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"rows.csv\"\r\n\
             Content-Type: text/csv\r\n\r\n{csv_text}\r\n--{boundary}--\r\n"
        );
        (
            header::HeaderValue::from_str(&format!("multipart/form-data; boundary={boundary}"))
                .unwrap(),
            Body::from(body),
        )
    }

    fn sample_csv(good_rows: usize, bad_rows: usize) -> String {
        let today = chrono::Utc::now().date_naive();
        let mut s = String::from("date,debit_account,credit_account,amount,currency,description\n");
        for _ in 0..good_rows {
            s.push_str(&format!("{today},ACC-D,ACC-C,100.00,TWD,import ok row\n"));
        }
        for i in 0..bad_rows {
            s.push_str(&format!(
                "{today},ACC-D,ACC-C,0.00{i},TWD,bad precision {i}\n"
            ));
        }
        s
    }

    #[tokio::test]
    async fn import_preview_reports_and_persists_no_ledger_effects() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        let (ctype, body) = csv_multipart(&sample_csv(3, 0), &csrf);
        let request = Request::builder()
            .method("POST")
            .uri("/import/preview")
            .header(header::CONTENT_TYPE, ctype)
            .header(header::COOKIE, cookie)
            .body(body)
            .unwrap();
        let (status, _, location) = send(app, request).await;
        assert_eq!(status, StatusCode::SEE_OTHER, "preview must PRG to report");
        let location = location.expect("location");

        // IMP-001: report exists, but the ledger was NOT touched.
        let batch_id = location.split("batch=").nth(1).unwrap_or("").to_string();
        assert!(!batch_id.is_empty());
        let (tx_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transactions")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(tx_count, 0, "preview must have zero ledger side effects");
        let (rows,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM import_rows WHERE status = 'ok'")
                .fetch_one(&pool)
                .await
                .expect("rows");
        assert_eq!(rows, 3, "all 3 valid rows must be staged");
    }

    #[tokio::test]
    async fn import_confirm_posts_only_valid_rows() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        // IMP-002: 2 good rows + 1 bad row (3-decimal amount).
        let (ctype, body) = csv_multipart(&sample_csv(2, 1), &csrf);
        let request = Request::builder()
            .method("POST")
            .uri("/import/preview")
            .header(header::CONTENT_TYPE, ctype)
            .header(header::COOKIE, cookie.clone())
            .body(body)
            .unwrap();
        let (_, _, location) = send(app.clone(), request).await;
        let batch_id = location
            .expect("location")
            .split("batch=")
            .nth(1)
            .unwrap_or("")
            .to_string();

        let request = Request::builder()
            .method("POST")
            .uri("/import/confirm")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie)
            .body(Body::from(format!("csrf_token={csrf}&batch_id={batch_id}")))
            .unwrap();
        let (status, _, _) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::SEE_OTHER, "confirm must PRG");

        // Only the 2 valid rows became balanced transactions.
        let (tx_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transactions")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(tx_count, 2, "only valid rows posted");
        let (audit_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM audit_log WHERE event_type = 'IMPORT_CONFIRMED'")
                .fetch_one(&pool)
                .await
                .expect("audit");
        assert_eq!(audit_count, 1, "batch confirm must be audited");
    }

    #[tokio::test]
    async fn import_confirm_is_replay_guarded() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        let (ctype, body) = csv_multipart(&sample_csv(1, 0), &csrf);
        let request = Request::builder()
            .method("POST")
            .uri("/import/preview")
            .header(header::CONTENT_TYPE, ctype)
            .header(header::COOKIE, cookie.clone())
            .body(body)
            .unwrap();
        let (_, _, location) = send(app.clone(), request).await;
        let batch_id = location
            .expect("location")
            .split("batch=")
            .nth(1)
            .unwrap_or("")
            .to_string();

        let confirm = |cookie: &str| {
            Request::builder()
                .method("POST")
                .uri("/import/confirm")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, cookie.to_string())
                .body(Body::from(format!("csrf_token={csrf}&batch_id={batch_id}")))
                .unwrap()
        };
        let (first, _, _) = send(app.clone(), confirm(&cookie)).await;
        assert_eq!(first, StatusCode::SEE_OTHER);
        // IMP-003: replay must be mechanically rejected.
        let (second, _, _) = send(app, confirm(&cookie)).await;
        assert_eq!(second, StatusCode::CONFLICT, "double-confirm must 409");
    }

    // ------------------------------------------------------------------
    // Slice 8 acceptance: wizard (WIZ-001) and drafts (DRAFT-001)
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn wizard_step2_rejects_same_account_and_preserves_values() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        let today = chrono::Utc::now().date_naive().to_string();
        let body = format!(
            "csrf_token={csrf}&template=cash-payment&debit_account=SAME&credit_account=SAME&amount=100&currency=TWD&description=wiz&transaction_date={today}"
        );
        let request = Request::builder()
            .method("POST")
            .uri("/new/step2")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie)
            .body(Body::from(body))
            .unwrap();
        let (status, page, _) = send(app, request).await;
        assert_eq!(status, StatusCode::OK, "wizard stays on step 2 with error");
        assert!(page.contains("步驟 2／3"), "must remain on step 2");
        assert!(
            page.contains("不得為同一科目"),
            "rule name must be shown (可解釋)"
        );
        assert!(page.contains("value=\"SAME\""), "values preserved");
    }

    #[tokio::test]
    async fn drafts_parse_creates_draft_and_wizard_prefills() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        // DRAFT-001: parse the worked example.
        let request = Request::builder()
            .method("POST")
            .uri("/drafts/parse")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie.clone())
            .body(Body::from(format!(
                "csrf_token={csrf}&text={}",
                urlencoding_lite("2026-11-04 支付租金 12,000 元 現金→租金費用")
            )))
            .unwrap();
        let (status, _, location) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::SEE_OTHER, "parse must PRG to wizard");
        let location = location.expect("location");
        assert!(
            location.contains("/new?step=2&from_draft="),
            "parse must lead to wizard step 2: {location}"
        );

        // Wizard step 2 must be pre-filled from the draft (依據卡在草稿頁).
        let request = Request::builder()
            .uri(&location)
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let (status, page, _) = send(app, request).await;
        assert_eq!(status, StatusCode::OK);
        assert!(page.contains("value=\"RENT-EXP\""), "debit prefilled");
        assert!(page.contains("value=\"CASH\""), "credit prefilled");
        assert!(page.contains("value=\"12,000.00\""), "amount prefilled");
        assert!(page.contains("依據卡"), "provenance note rendered");
    }

    // ------------------------------------------------------------------
    // Slice 8 acceptance: monthly report + CSV (RPT-001) and chart (ZJS-001)
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn monthly_report_page_and_csv_render_posted_data() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        // Post one balanced transaction this month via the verified path.
        let today = chrono::Utc::now().date_naive().to_string();
        let request = Request::builder()
            .method("POST")
            .uri("/transactions")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie.clone())
            .body(Body::from(form_body(&csrf, "1234.50", &today)))
            .unwrap();
        let (status, _, _) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::SEE_OTHER);

        let ym = today[..7].to_string();
        let request = Request::builder()
            .uri(format!("/reports/monthly?ym={ym}"))
            .header(header::COOKIE, cookie.clone())
            .body(Body::empty())
            .unwrap();
        let (status, page, _) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK);
        assert!(page.contains("1,234.50"), "report must show posted amount");
        assert!(page.contains("ACC-DEBIT"), "report must list accounts");

        let request = Request::builder()
            .uri(format!("/reports/monthly.csv?ym={ym}"))
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.expect("csv response");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .map(|v| v.to_str().unwrap().starts_with("text/csv"))
                .unwrap_or(false),
            "CSV content type"
        );
        let body = axum::body::to_bytes(response.into_body(), 1_048_576)
            .await
            .expect("csv body");
        let text = String::from_utf8_lossy(&body).to_string();
        assert!(
            text.starts_with("account_id,debit_minor,credit_minor,net_minor"),
            "CSV header uses integer minor units"
        );
        assert!(
            text.contains("ACC-DEBIT,123450,"),
            "integer minor units in CSV"
        );
    }

    #[tokio::test]
    async fn dashboard_renders_chart_svg_zero_js() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;
        let (cookie, csrf) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        // Post one entry so the chart has data.
        let today = chrono::Utc::now().date_naive().to_string();
        let request = Request::builder()
            .method("POST")
            .uri("/transactions")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie.clone())
            .body(Body::from(form_body(&csrf, "500", &today)))
            .unwrap();
        let _ = send(app.clone(), request).await;

        let request = Request::builder()
            .uri("/")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let (_, body, _) = send(app, request).await;
        // ZJS-001: chart is server-side SVG; the page has zero scripts.
        assert!(body.contains("<svg"), "server-side SVG chart must render");
        assert!(body.contains("近六個月借貸合計"), "chart a11y title");
        assert!(!body.contains("<script"), "zero-JS invariant");
    }

    // ------------------------------------------------------------------
    // Slice 10 helpers
    // ------------------------------------------------------------------

    /// Non-asserting login: returns (cookie, csrf) or None on failure.
    async fn try_login(
        app: &Router,
        pool: &SqlitePool,
        username: &str,
        password: &str,
    ) -> Option<(String, String)> {
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(format!(
                "username={username}&password={password}"
            )))
            .unwrap();
        let response = app.clone().oneshot(request).await.expect("login attempt");
        if response.status() != StatusCode::SEE_OTHER {
            return None;
        }
        let cookie = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .find_map(|v| v.to_str().ok())
            .map(|v| v.split(';').next().unwrap_or("").to_string())
            .expect("session cookie");
        let session_id = cookie.strip_prefix("flaps_session=").expect("cookie shape");
        let (csrf,): (String,) =
            sqlx::query_as("SELECT csrf_token FROM sessions WHERE session_id = ?")
                .bind(session_id)
                .fetch_one(pool)
                .await
                .expect("session row");
        Some((cookie, csrf))
    }

    async fn seeded_app_with_admin() -> (Router, SqlitePool, String, String) {
        let pool = in_memory_pool().await;
        let state = AppState::new(pool.clone()).await.expect("state");
        let app = router(state);
        let hash = auth::hash_password(TEST_ADMIN_PASSWORD).expect("hash");
        sqlx::query(
            "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
             VALUES ('user-admin', 'admin', ?, 1)",
        )
        .bind(&hash)
        .execute(&pool)
        .await
        .expect("seed admin");
        let (cookie, csrf) = try_login(&app, &pool, "admin", TEST_ADMIN_PASSWORD)
            .await
            .expect("admin login");
        (app, pool, cookie, csrf)
    }

    async fn get_with_location(
        app: &Router,
        uri: &str,
        cookie: &str,
    ) -> (StatusCode, String, Option<String>) {
        let request = Request::builder()
            .uri(uri)
            .header(header::COOKIE, cookie.to_string())
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.expect("response");
        let status = response.status();
        let location = response
            .headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let body = axum::body::to_bytes(response.into_body(), 1_048_576)
            .await
            .expect("body");
        let body = String::from_utf8_lossy(&body).to_string();
        (status, body, location)
    }
    // ------------------------------------------------------------------
    // Slice 10 acceptance: pivot (PIV-001/002)
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn pivot_matrix_matches_hand_computed_values() {
        let (app, pool, cookie, _csrf) = seeded_app_with_admin().await;

        sqlx::query(
            "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, status, created_by) \
             VALUES ('pv-t1', 'TX-PV1', '2026-10-05', 'p', 'POSTED', 'tester'), \
                    ('pv-t2', 'TX-PV2', '2026-11-05', 'p', 'POSTED', 'tester')",
        )
        .execute(&pool)
        .await
        .expect("seed headers");
        sqlx::query("INSERT INTO accounts (account_id, account_code, account_name, account_type, currency, is_active) VALUES ('ACC-A', 'A1', 'Account A', 'ASSETS', 'TWD', 1), ('ACC-B', 'B1', 'Account B', 'LIABILITIES', 'TWD', 1)")
        .execute(&pool)
        .await
        .expect("seed accounts");
        sqlx::query(
            "INSERT INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency, exchange_rate) \
             VALUES ('pv-i1', 'pv-t1', 'ACC-A', 'DEBIT', 1000, 'TWD', 10000), \
                    ('pv-i2', 'pv-t1', 'ACC-B', 'CREDIT', 1000, 'TWD', 10000), \
                    ('pv-i3', 'pv-t2', 'ACC-A', 'DEBIT', 2000, 'TWD', 10000), \
                    ('pv-i4', 'pv-t2', 'ACC-B', 'CREDIT', 2000, 'TWD', 10000)",
        )
        .execute(&pool)
        .await
        .expect("seed items");

        let (status, page, _) = get_with_location(
            &app,
            "/reports/pivot?rows=account&cols=month&measure=debit",
            &cookie,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        // PIV-001: hand-computed — ACC-A debits: 2026-10=10.00, 2026-11=20.00.
        assert!(page.contains(">10.00<"), "October ACC-A cell must be 10.00");
        assert!(
            page.contains(">20.00<"),
            "November ACC-A cell must be 20.00"
        );
        assert!(page.contains(">30.00<"), "ACC-A row total must be 30.00");
    }

    #[tokio::test]
    async fn pivot_rejects_unknown_dimensions_with_rule_name() {
        let (app, _pool, cookie, _csrf) = seeded_app_with_admin().await;
        let (status, body, _) = get_with_location(
            &app,
            "/reports/pivot?rows=account&cols=hacked&measure=debit",
            &cookie,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(
            body.contains("R-PIV-WHITELIST"),
            "rejection must name the whitelist rule (PIV-002)"
        );
    }

    #[tokio::test]
    async fn pivot_caps_rows_and_folds_into_other_bucket() {
        let (app, pool, cookie, _csrf) = seeded_app_with_admin().await;

        // PIV-003: 30 accounts x 1 month -> matrix capped at 20 rows, the
        // remaining totals folded into the OTHER row (explainable, not
        // silently dropped).
        sqlx::query(
            "WITH RECURSIVE seq(n) AS (SELECT 0 UNION ALL SELECT n + 1 FROM seq WHERE n < 29) \
             INSERT INTO accounts (account_id, account_code, account_name, account_type, currency, is_active) \
             SELECT 'PIV-' || n, 'PV' || n, 'Pivot ' || n, 'ASSETS', 'TWD', 1 FROM seq",
        )
        .execute(&pool)
        .await
        .expect("seed 30 accounts");

        sqlx::query(
            "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, status, created_by) \
             VALUES ('piv-t1', 'TX-PIV', '2026-11-05', 'p', 'POSTED', 'tester')",
        )
        .execute(&pool)
        .await
        .expect("seed header");

        // 30 debit legs (one per account), all on one transaction.
        let mut values: Vec<String> = Vec::new();
        for n in 0..30 {
            values.push(format!(
                "('piv-i{}', 'piv-t1', 'PIV-{}', 'DEBIT', {}, 'TWD', 10000)",
                n,
                n,
                100 + n
            ));
        }
        sqlx::query(&format!(
            "INSERT INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency, exchange_rate) VALUES {}",
            values.join(", ")
        ))
        .execute(&pool)
        .await
        .expect("seed 30 legs");

        let (status, page, _) = get_with_location(
            &app,
            "/reports/pivot?rows=account&cols=month&measure=debit",
            &cookie,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        // Capped at 20 rows + the OTHER fold row.
        assert!(
            page.contains("Rows capped at 20") && page.contains("OTHER"),
            "truncation must be visible"
        );
        // Top-20 = PIV-10..PIV-29 (1.10..1.29); dropped PIV-0..PIV-9 fold
        // into OTHER = 1045 minor = 10.45.
        assert!(page.contains(">1.29<"), "top row boundary must be kept");
        let mut piv_rows: Vec<String> = Vec::new();
        let mut search = 0usize;
        while let Some(pos) = page[search..].find("PIV-1") {
            let abs = search + pos;
            piv_rows.push(page[abs..(abs + 6).min(page.len())].to_string());
            search = abs + 6;
        }
        assert!(
            page.contains(">10.45<"),
            "OTHER folded total (1045 minor = 10.45)"
        );
        assert!(
            !page.contains(">1.00<"),
            "dropped rows must not appear as-is"
        );
        // Top-20 = PIV-10..PIV-29 (1.10..1.29).
        assert!(page.contains(">1.29<"), "top row boundary must be kept");
        assert!(page.contains(">1.10<"), "top row start must be kept");
    }

    #[tokio::test]
    async fn ai_suggest_rejects_forged_csrf_token() {
        let (app, _pool, cookie, _csrf) = seeded_app_with_admin().await;
        let request = Request::builder()
            .method("POST")
            .uri("/drafts/ai_suggest")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie)
            .body(Body::from("csrf_token=forged&text=test"))
            .unwrap();
        let (status, _, _) = send(app, request).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "ai_suggest must enforce CSRF (T-08)"
        );
    }

    // ------------------------------------------------------------------
    // Slice 10 acceptance: parallel coordinates (PAR-001/002)
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn parallel_projection_renders_one_line_per_account_zero_js() {
        let (app, pool, cookie, _csrf) = seeded_app_with_admin().await;
        sqlx::query(
            "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, status, created_by) \
             VALUES ('pp-t1', 'TX-PP1', '2026-11-05', 'p', 'POSTED', 'tester')",
        )
        .execute(&pool)
        .await
        .expect("seed header");
        sqlx::query("INSERT INTO accounts (account_id, account_code, account_name, account_type, currency, is_active) VALUES ('ACC-A', 'A1', 'Account A', 'ASSETS', 'TWD', 1)")
        .execute(&pool)
        .await
        .expect("seed accounts");
        sqlx::query(
            "INSERT INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency, exchange_rate) \
             VALUES ('pp-i1', 'pp-t1', 'ACC-A', 'DEBIT', 5000, 'TWD', 10000)",
        )
        .execute(&pool)
        .await
        .expect("seed item");

        let (status, page, _) =
            get_with_location(&app, "/reports/parallel?months=2", &cookie).await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            page.contains("<polyline"),
            "one polyline per account (PAR-001)"
        );
        assert!(
            page.contains("<title>ACC-A</title>"),
            "accessible line title"
        );
        assert!(!page.contains("<script"), "zero-JS invariant (PAR-002)");
    }

    // ------------------------------------------------------------------
    // Slice 10 acceptance: AI suggestion (AIS-001..003)
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn ai_suggest_disabled_by_default_returns_503() {
        let (app, _pool, cookie, csrf) = seeded_app_with_admin().await;
        let request = Request::builder()
            .method("POST")
            .uri("/drafts/ai_suggest")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie)
            .body(Body::from(format!(
                "csrf_token={csrf}&text=支付租金 12000 元"
            )))
            .unwrap();
        let (status, body, _) = send(app, request).await;
        // AIS-001: feature flag defaults to OFF -> explicit 503 with guidance.
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(body.contains("AI 建議"), "guidance must name the feature");
    }

    #[tokio::test]
    async fn ai_suggestion_adopt_never_posts_to_ledger() {
        let (app, pool, _admin_cookie, _admin_csrf) = seeded_app_with_admin().await;
        let (cookie, csrf) = try_login(&app, &pool, "admin", TEST_ADMIN_PASSWORD)
            .await
            .expect("admin login");

        // Adopt an AI suggestion through the human-confirmed path.
        let request = Request::builder()
            .method("POST")
            .uri("/drafts/adopt")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie)
            .body(Body::from(format!(
                "csrf_token={csrf}&date=2026-11-04&debit_account=RENT-EXP&credit_account=CASH&amount=12000.00&currency=TWD&description=ai adopted"
            )))
            .unwrap();
        let (status, diag_body, _location) = send(app.clone(), request).await;
        assert_eq!(
            status,
            StatusCode::SEE_OTHER,
            "adopt must PRG to wizard; body: {diag_body}"
        );

        // AIS-002: drafts grew by one; the ledger was NOT touched.
        let (drafts,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM drafts")
            .fetch_one(&pool)
            .await
            .expect("drafts count");
        assert_eq!(drafts, 1, "adopt creates exactly one draft");
        let (tx_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transactions")
            .fetch_one(&pool)
            .await
            .expect("tx count");
        assert_eq!(tx_count, 0, "AI suggestion must never auto-post");
    }

    #[test]
    fn ai_suggestion_schema_validation_rejects_malformed_output() {
        // AIS-003: malformed model output must be rejected by the schema gate.
        assert!(flaps_core::parse_ai_suggestion("{ \"date\": \"not-a-date\" }").is_err());
        assert!(flaps_core::parse_ai_suggestion(
            "{ \"date\": \"2026-11-04\", \"debit_account\": \"A\", \"credit_account\": \"B\", \"amount\": \"abc\", \"currency\": \"TWD\" }"
        )
        .is_err());
        assert!(flaps_core::parse_ai_suggestion("not json at all").is_err());
    }

    // ---------------------------------------------------------------------------
    // Slice 11: security headers (OWASP secure headers checklist)
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn security_headers_present_on_public_and_protected_responses() {
        let (app, _pool, cookie, _csrf) = seeded_app_with_admin().await;

        for uri in ["/login", "/api/health", "/assets/design-system.css"] {
            let request = Request::builder().uri(uri).body(Body::empty()).unwrap();
            let response = app.clone().oneshot(request).await.expect("response");
            let h = response.headers();
            assert!(
                h.get(header::CONTENT_SECURITY_POLICY)
                    .map(|v| v.to_str().unwrap().contains("default-src 'none'"))
                    .unwrap_or(false),
                "CSP missing on {uri}"
            );
            assert!(
                h.get(header::X_CONTENT_TYPE_OPTIONS)
                    .map(|v| v.to_str().unwrap() == "nosniff")
                    .unwrap_or(false),
                "nosniff missing on {uri}"
            );
            assert!(
                h.get(header::X_FRAME_OPTIONS)
                    .map(|v| v.to_str().unwrap() == "DENY")
                    .unwrap_or(false),
                "frame denial missing on {uri}"
            );
            assert!(
                h.get(header::REFERRER_POLICY)
                    .map(|v| v.to_str().unwrap() == "no-referrer")
                    .unwrap_or(false),
                "referrer policy missing on {uri}"
            );
            assert!(
                h.get(header::CACHE_CONTROL)
                    .map(|v| v.to_str().unwrap().contains("no-store"))
                    .unwrap_or(false),
                "no-store missing on {uri}"
            );
        }

        // Protected page too (session data must not be cached).
        let request = Request::builder()
            .uri("/")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.expect("response");
        assert!(
            response
                .headers()
                .get(header::CACHE_CONTROL)
                .map(|v| v.to_str().unwrap().contains("no-store"))
                .unwrap_or(false),
            "authenticated page must be no-store"
        );
    }

    // ---------------------------------------------------------------------------
    // Slice 11: config-driven Secure cookie flag (R-07 preparation)
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn login_sets_secure_cookie_when_configured() {
        let (app, pool, _cookie, _csrf) = seeded_app_with_admin().await;
        sqlx::query(
            "INSERT OR REPLACE INTO erp_config (config_id, config_category, config_key, config_value, config_type, is_sensitive) \
             VALUES ('cfg-secure', 'security', 'cookie_secure', '1', 'BOOLEAN', 0)",
        )
        .execute(&pool)
        .await
        .expect("enable secure flag");

        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(format!(
                "username=admin&password={TEST_ADMIN_PASSWORD}"
            )))
            .unwrap();
        let response = app.oneshot(request).await.expect("login");
        let cookie_hdr = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .find_map(|v| v.to_str().ok())
            .expect("set-cookie")
            .to_string();
        assert!(
            cookie_hdr.contains("Secure"),
            "Secure attribute required when cookie_secure=1: {cookie_hdr}"
        );
    }

    #[tokio::test]
    async fn expired_sessions_are_purged_at_login() {
        let (app, pool, _cookie, _csrf) = seeded_app_with_admin().await;

        // Plant an expired session row (yesterday).
        sqlx::query(
            "INSERT INTO sessions (session_id, user_id, csrf_token, expires_at) \
             VALUES ('expired-1', 'user-admin', 'csrf-exp', datetime('now', '-1 day'))",
        )
        .execute(&pool)
        .await
        .expect("plant expired session");
        let (before,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE expires_at <= datetime('now')")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(before, 1, "expired row must exist before login");

        // A login purges expired sessions (housekeeping at auth boundary).
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(format!(
                "username=admin&password={TEST_ADMIN_PASSWORD}"
            )))
            .unwrap();
        let response = app.clone().oneshot(request).await.expect("login response");
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let (after,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE expires_at <= datetime('now')")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(after, 0, "expired sessions must be purged at login");
    }

    // ------------------------------------------------------------------
    // Slice 10 acceptance: parse coverage (COV-001)
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn states_page_renders_component_showcase() {
        let (app, _pool, cookie, _csrf) = seeded_app_with_admin().await;
        let request = Request::builder()
            .method("GET")
            .uri("/states")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .expect("build request");
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::OK, "states page must return 200");
        assert!(body.contains("介面狀態"), "page title marker present");
        assert!(body.contains("btn-primary"), "button showcase present");
        assert!(body.contains("badge-success"), "badge showcase present");
        assert!(body.contains("pagination"), "pagination showcase present");
        assert!(
            body.contains("prefers-reduced-motion"),
            "motion note present"
        );
        assert!(!body.contains("<script"), "zero-JS invariant holds");
        assert!(!body.contains("onclick="), "no inline handlers");
    }

    #[tokio::test]
    async fn parse_coverage_computes_hit_ratio() {
        let (app, pool, cookie, _csrf) = seeded_app_with_admin().await;
        seed_credentials(&pool).await;

        // 4 drafts: 2 complete, 2 incomplete -> 50%.
        for (idx, complete) in [true, false, true, false].iter().enumerate() {
            let json = if *complete {
                r#"{"date":"2026-11-04","amount_minor":100,"debit_account":"A","credit_account":"B","currency":"TWD","evidence":[]}"#
            } else {
                r#"{"date":null,"amount_minor":null,"debit_account":null,"credit_account":null,"currency":null,"evidence":[]}"#
            };
            sqlx::query(
                "INSERT INTO drafts (draft_id, source, draft_json, created_by) VALUES (?, 'text', ?, 'user-admin')",
            )
            .bind(format!("cov-{idx}"))
            .bind(json)
            .execute(&pool)
            .await
            .expect("seed draft");
        }

        let request = Request::builder()
            .uri("/admin/parse_coverage")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let (status, body, _) = send(app, request).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("50%"), "coverage ratio must be 50% (COV-001)");
        assert!(body.contains("4"), "total drafts must be shown");
    }

    // ------------------------------------------------------------------
    // Slice 10 acceptance: builder refactor (H-001)
    // ------------------------------------------------------------------

    #[test]
    fn transaction_builder_constructs_domain_objects() {
        let built = flaps_core::ledger::transaction::Transaction::builder()
            .transaction_id("tx-b1")
            .transaction_code("TX-BUILDER")
            .transaction_date(chrono::NaiveDate::from_ymd_opt(2026, 11, 4).unwrap())
            .description("builder test")
            .reference_number(None)
            .status("POSTED")
            .source_type(Some("test".to_string()))
            .created_by("tester")
            .build()
            .expect("builder must construct");
        assert_eq!(built.transaction_code, "TX-BUILDER");

        let item = flaps_core::ledger::transaction::TransactionItem::builder()
            .item_id("it-b1")
            .transaction_id("tx-b1")
            .account_id("ACC-B")
            .item_type("DEBIT")
            .amount_minor(100)
            .currency("TWD")
            .exchange_rate_scaled(10_000)
            .build()
            .expect("builder must construct");
        assert_eq!(item.amount_minor, 100);
    }

    // ------------------------------------------------------------------
    // Slice 12: P2 - 100,000 transaction stress test (OQ-10-1 performance)
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn stress_10000_transactions_p95_under_500ms() {
        let (app, pool) = test_app().await;
        seed_credentials(&pool).await;

        // Seed accounts referenced by demo transactions.
        sqlx::query("INSERT OR IGNORE INTO accounts (account_id, account_code, account_name, account_type, currency, is_active)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
            .bind("BANK-001").bind("BANK-001").bind("Bank Account").bind("ASSETS").bind("TWD").bind(1)
            .execute(&pool).await.expect("seed BANK-001");
        sqlx::query("INSERT OR IGNORE INTO accounts (account_id, account_code, account_name, account_type, currency, is_active)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
            .bind("SUPPLIES-EXP").bind("SUPPLIES-EXP").bind("Supplies Expense").bind("EXPENSES").bind("TWD").bind(1)
            .execute(&pool).await.expect("seed SUPPLIES-EXP");

        // Seed 100,000 transactions via batched multi-row INSERTs (500 rows
        // per statement, one open transaction): identical rows to per-row
        // binds, ~100x faster setup in debug builds. All values are
        // test-generated constants, never user input.
        sqlx::query("BEGIN").execute(&pool).await.expect("begin tx");
        let mut tx_count = 0i64;
        let mut tx_rows: Vec<String> = Vec::with_capacity(500);
        let mut it_rows: Vec<String> = Vec::with_capacity(1_000);
        for i in 0..100_000i64 {
            let tx_id = format!("tx-stress-{}", i);
            let tx_code = format!("TX-STRESS-{}", i);
            let date = format!("2026-11-{:02}", 1 + (i % 30));
            let desc = format!("Stress test transaction {}", i);
            let amount = (1000 + (i % 990000)) * 100;

            tx_rows.push(format!(
                "('{}', '{}', '{}', '{}', 'POSTED', 'system')",
                tx_id, tx_code, date, desc
            ));
            it_rows.push(format!(
                "('it-{}-d', '{}', 'BANK-001', 'DEBIT', {}, 'TWD', 10000)",
                i, tx_id, amount
            ));
            it_rows.push(format!(
                "('it-{}-c', '{}', 'SUPPLIES-EXP', 'CREDIT', {}, 'TWD', 10000)",
                i, tx_id, amount
            ));
            tx_count += 1;

            if tx_rows.len() == 500 {
                let tx_sql = format!(
                    "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, status, created_by) VALUES {}",
                    tx_rows.join(", ")
                );
                sqlx::query(&tx_sql)
                    .execute(&pool)
                    .await
                    .expect("seed transactions batch");
                let it_sql = format!(
                    "INSERT INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency, exchange_rate) VALUES {}",
                    it_rows.join(", ")
                );
                sqlx::query(&it_sql)
                    .execute(&pool)
                    .await
                    .expect("seed items batch");
                tx_rows.clear();
                it_rows.clear();
            }
        }
        if !tx_rows.is_empty() {
            let tx_sql = format!(
                "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, status, created_by) VALUES {}",
                tx_rows.join(", ")
            );
            sqlx::query(&tx_sql)
                .execute(&pool)
                .await
                .expect("seed transactions remainder");
            let it_sql = format!(
                "INSERT INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency, exchange_rate) VALUES {}",
                it_rows.join(", ")
            );
            sqlx::query(&it_sql)
                .execute(&pool)
                .await
                .expect("seed items remainder");
        }
        sqlx::query("COMMIT")
            .execute(&pool)
            .await
            .expect("commit tx");
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transactions")
            .fetch_one(&pool)
            .await
            .expect("count seeded transactions");
        assert_eq!(count, 100_000, "expected 100,000 seeded transactions");

        // Login as admin for authenticated requests.
        let (cookie, _) = login(&app, &pool, "admin", TEST_ADMIN_PASSWORD).await;

        // Warm-up pass (untimed): the first query per month pays cold-start
        // disk I/O (SQLite page-cache miss), which is not representative of
        // steady-state server operation and would dominate a 3-sample P95.
        // The SLA asserted below governs warm operation.
        for month in ["09", "10", "11"] {
            let request = Request::builder()
                .method("GET")
                .uri(format!("/reports/monthly?ym=2026-{}", month))
                .header(header::CONTENT_TYPE, "text/html")
                .header(header::COOKIE, cookie.clone())
                .body(Body::empty())
                .expect("build warm-up request");
            let (status, _, _) = send(app.clone(), request).await;
            assert_eq!(status, StatusCode::OK, "warm-up must return 200");
        }

        // Measure steady-state P95: 7 rounds x 3 months = 21 samples. With 21
        // samples the P95 index (ceil(21*0.95)-1 = 19) tolerates a single
        // scheduler hiccup instead of degenerating to max-of-3.
        let mut timings_ms = Vec::new();
        for _round in 0..7 {
            for month in ["09", "10", "11"] {
                let start = std::time::Instant::now();
                let request = Request::builder()
                    .method("GET")
                    .uri(format!("/reports/monthly?ym=2026-{}", month))
                    .header(header::CONTENT_TYPE, "text/html")
                    .header(header::COOKIE, cookie.clone())
                    .body(Body::empty())
                    .expect("build request");
                let (status, _, _) = send(app.clone(), request).await;
                assert_eq!(status, StatusCode::OK, "monthly report must return 200");
                let elapsed_ms = start.elapsed().as_millis() as u64;
                timings_ms.push(elapsed_ms);
            }
        }

        timings_ms.sort_unstable();
        let p95_idx = (timings_ms.len() as f64 * 0.95).ceil() as usize - 1;
        let p95_ms = timings_ms[p95_idx];
        let max_ms = timings_ms.iter().max().unwrap_or(&0);
        let total_ms: u64 = timings_ms.iter().sum();

        println!(
            "Stress test: {} transactions, {} monthly queries, timings_ms={:?}, p95={}ms, max={}ms, total={}ms",
            tx_count, timings_ms.len(), timings_ms, p95_ms, max_ms, total_ms
        );
        assert!(
            p95_ms <= 500,
            "P95 query time {}ms exceeds 500ms SLA",
            p95_ms
        );
    }
}
