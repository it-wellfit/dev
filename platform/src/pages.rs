//! 工作室頁面批次（27+ 頁目標）：稽核、報表、銀行支付營運、個人設定、
//! 系統管理。全部 server-render 零 JS；共用 `panel`/`kpi-grid`/`table`/
//! `badge` 設計 Token；變更類操作一律 POST + CSRF + PRG（憲章 §3）。

use std::collections::HashMap;

use askama::Template;
use axum::extract::{Form, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Extension;
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::Session;
use crate::bankpay::api::verify_csrf_token;

type AuditRow = crate::cross_module_api::AuditRow;

/// 稽核事件檢視（模板用；已套用中文標籤與語義徽章）。
#[derive(Debug, Clone)]
pub struct AuditView {
    pub log_id: String,
    pub time: String,
    pub user_id: String,
    pub act_name: String,
    pub event_type: String,
    pub event_label: String,
    pub event_class: String,
    pub resource: String,
    pub details: String,
}

/// Legacy alias kept for compatibility with template code written against it.
pub type AuditViewLegacy = AuditView;

fn to_views(rows: Vec<AuditRow>) -> Vec<AuditView> {
    rows.into_iter()
        .map(
            |(log_id, user_id, username, event_type, resource, details, time)| {
                let (event_label, event_class) = crate::api::audit_event_display(&event_type);
                AuditView {
                    log_id,
                    time,
                    user_id,
                    act_name: username.unwrap_or_else(|| "unknown".to_string()),
                    event_label,
                    event_class: event_class.to_string(),
                    event_type,
                    resource,
                    details,
                }
            },
        )
        .collect()
}

/// 寫入稽核事件（頁面層共用；與 bankpay::api::write_audit 同一 SQL 形狀）。
async fn write_event(
    pool: &SqlitePool,
    event_type: &str,
    performed_by: &str,
    request_payload: serde_json::Value,
) {
    let event_data = serde_json::json!({ "request": request_payload, "response": null });
    let _ = sqlx::query(
        "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) \
         VALUES (?, NULL, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(event_type)
    .bind(event_data.to_string())
    .bind(performed_by)
    .execute(pool)
    .await;
}

/// 篩選後的稽核列（動態 WHERE 全參數化；已轉為模板檢視）。
async fn audit_rows_page(
    pool: &SqlitePool,
    user: Option<&str>,
    action: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
    limit: i64,
    offset: i64,
) -> Vec<AuditView> {
    let mut sql = "SELECT al.log_id, al.performed_by, u.username, al.event_type, COALESCE(al.transaction_id, ''), COALESCE(al.event_data, ''), al.performed_at FROM audit_log al LEFT JOIN users u ON al.performed_by = u.user_id WHERE 1=1".to_string();
    let mut params: Vec<String> = vec![];
    if let Some(u) = user.filter(|s| !s.is_empty()) {
        sql.push_str(" AND al.performed_by = ?");
        params.push(u.to_string());
    }
    if let Some(a) = action.filter(|s| !s.is_empty()) {
        sql.push_str(" AND al.event_type LIKE ?");
        params.push(format!("%{a}%"));
    }
    if let Some(s) = start.filter(|s| !s.is_empty()) {
        sql.push_str(" AND al.performed_at >= ?");
        params.push(s.to_string());
    }
    if let Some(e) = end.filter(|s| !s.is_empty()) {
        sql.push_str(" AND al.performed_at <= ?");
        params.push(format!("{e} 23:59:59"));
    }
    sql.push_str(" ORDER BY al.performed_at DESC LIMIT ? OFFSET ?");
    params.push(limit.to_string());
    params.push(offset.to_string());
    let mut qb = sqlx::query_as::<_, AuditRow>(sqlx::AssertSqlSafe(sql.as_str()));
    for p in &params {
        qb = qb.bind(p);
    }
    to_views(qb.fetch_all(pool).await.unwrap_or_default())
}

/// 同條件的總數（分頁計算）。
async fn audit_total(
    pool: &SqlitePool,
    user: Option<&str>,
    action: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> i64 {
    let mut sql = "SELECT COUNT(*) FROM audit_log al WHERE 1=1".to_string();
    let mut params: Vec<String> = vec![];
    if let Some(u) = user.filter(|s| !s.is_empty()) {
        sql.push_str(" AND al.performed_by = ?");
        params.push(u.to_string());
    }
    if let Some(a) = action.filter(|s| !s.is_empty()) {
        sql.push_str(" AND al.event_type LIKE ?");
        params.push(format!("%{a}%"));
    }
    if let Some(s) = start.filter(|s| !s.is_empty()) {
        sql.push_str(" AND al.performed_at >= ?");
        params.push(s.to_string());
    }
    if let Some(e) = end.filter(|s| !s.is_empty()) {
        sql.push_str(" AND al.performed_at <= ?");
        params.push(format!("{e} 23:59:59"));
    }
    let total: i64 = {
        let mut qb = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(sql.as_str()));
        for p in &params {
            qb = qb.bind(p);
        }
        qb.fetch_one(pool).await.unwrap_or(0)
    };
    total
}

fn render<T: Template>(t: T) -> Response {
    match t.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("模板渲染失敗：{e}"),
        )
            .into_response(),
    }
}

async fn primary_role(pool: &SqlitePool, user_id: &str) -> String {
    sqlx::query_scalar(
        "SELECT r.role_code FROM user_roles ur JOIN roles r ON ur.role_id = r.role_id \
         WHERE ur.user_id = ? ORDER BY r.role_code LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// 1. 稽核軌跡頁（治理稽核工作室主頁）
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "audit_page.html")]
pub struct AuditPageTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub role_code: String,
    pub rows: Vec<AuditView>,
    pub total: i64,
    pub page: i64,
    pub total_pages: i64,
    pub f_user: String,
    pub f_action: String,
    pub f_start: String,
    pub f_end: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct AuditPageQuery {
    pub user: Option<String>,
    pub action: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub page: Option<String>,
}

/// GET /audit — 稽核軌跡完整頁（篩選＋分頁）。
pub async fn audit_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(q): Query<AuditPageQuery>,
) -> Response {
    let page = q
        .page
        .as_deref()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1)
        .max(1);
    let limit = 25i64;
    let total = audit_total(
        &state.pool,
        q.user.as_deref(),
        q.action.as_deref(),
        q.start.as_deref(),
        q.end.as_deref(),
    )
    .await;
    let total_pages = ((total + limit - 1) / limit).max(1);
    let page = page.min(total_pages);
    let rows = audit_rows_page(
        &state.pool,
        q.user.as_deref(),
        q.action.as_deref(),
        q.start.as_deref(),
        q.end.as_deref(),
        limit,
        (page - 1) * limit,
    )
    .await;
    let role_code = primary_role(&state.pool, &session.user_id).await;
    render(AuditPageTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        role_code,
        rows,
        total,
        page,
        total_pages,
        f_user: q.user.unwrap_or_default(),
        f_action: q.action.unwrap_or_default(),
        f_start: q.start.unwrap_or_default(),
        f_end: q.end.unwrap_or_default(),
    })
}

// ---------------------------------------------------------------------------
// 2. 試算表（報表工作室）：借貸合計＋平衡檢查
// ---------------------------------------------------------------------------

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct TrialBalanceRow {
    pub account_code: String,
    pub account_name: String,
    pub debit: String,
    pub credit: String,
}

#[derive(Template)]
#[template(path = "trial_balance.html")]
pub struct TrialBalanceTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub rows: Vec<TrialBalanceRow>,
    pub total_debit: i64,
    pub total_credit: i64,
    pub balanced: bool,
}

/// GET /reports/trial-balance — 試算表（借貸合計＋平衡徽章；INV-01 整數分）。
pub async fn trial_balance_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Response {
    let rows: Vec<TrialBalanceRow> = sqlx::query_as(
        "SELECT a.account_code, a.account_name, \
           CAST(COALESCE(SUM(CASE WHEN ti.item_type = 'DEBIT' THEN ti.amount_minor END), 0) AS TEXT) AS debit, \
           CAST(COALESCE(SUM(CASE WHEN ti.item_type = 'CREDIT' THEN ti.amount_minor END), 0) AS TEXT) AS credit \
         FROM transaction_items ti JOIN accounts a ON ti.account_id = a.account_id \
         GROUP BY a.account_id, a.account_code, a.account_name ORDER BY a.account_code",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let total_debit: i64 = rows
        .iter()
        .map(|r| r.debit.parse::<i64>().unwrap_or(0))
        .sum();
    let total_credit: i64 = rows
        .iter()
        .map(|r| r.credit.parse::<i64>().unwrap_or(0))
        .sum();
    render(TrialBalanceTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        balanced: total_debit == total_credit,
        rows,
        total_debit,
        total_credit,
    })
}

// ---------------------------------------------------------------------------
// 3. 總分類帳（報表工作室）：單科分明細＋累計餘額
// ---------------------------------------------------------------------------

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct LedgerEntryRow {
    pub transaction_date: String,
    pub description: String,
    pub item_type: String,
    pub amount_display: String,
}

#[derive(Template)]
#[template(path = "account_ledger.html")]
pub struct AccountLedgerTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub account: Option<AccountPickRow>,
    pub entries: Vec<LedgerEntryRow>,
    pub accounts: Vec<AccountPickRow>,
    pub selected: String,
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct AccountPickRow {
    pub account_id: String,
    pub account_code: String,
    pub account_name: String,
}

/// GET /reports/ledger?account=<account_id> — 總分類帳（累計餘額＝Σdebit−Σcredit）。
pub async fn account_ledger_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let accounts: Vec<AccountPickRow> = sqlx::query_as(
        "SELECT account_id, account_code, account_name FROM accounts WHERE is_active = 1 ORDER BY account_code",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let selected = q.get("account").cloned().unwrap_or_default();
    let account = accounts.iter().find(|a| a.account_id == selected).cloned();
    let entries: Vec<LedgerEntryRow> = if account.is_some() {
        sqlx::query_as(
            "SELECT t.transaction_date, t.description, ti.item_type, ti.amount_minor \
             FROM transaction_items ti JOIN transactions t ON ti.transaction_id = t.transaction_id \
             WHERE ti.account_id = ? ORDER BY t.transaction_date, t.transaction_code LIMIT 500",
        )
        .bind(&selected)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default()
    } else {
        vec![]
    };
    render(AccountLedgerTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        account,
        entries,
        accounts,
        selected,
    })
}

// ---------------------------------------------------------------------------
// 4-6. 銀行支付營運（批次總覽／詳情／例外佇列）
// ---------------------------------------------------------------------------

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct BatchRowFull {
    pub batch_id: String,
    pub esun_ref: Option<String>,
    pub amount_minor: i64,
    pub currency: String,
    pub status: String,
    pub created_by: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

async fn batches_query(pool: &SqlitePool, statuses: Option<&[&str]>) -> Vec<BatchRowFull> {
    let (sql, bind_list): (String, Vec<&str>) = match statuses {
        Some(list) => {
            let placeholders = list.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            (
                format!(
                    "SELECT batch_id, esun_ref, amount_minor, currency, status, created_by, created_at, completed_at \
                     FROM bankpay_batches WHERE status IN ({placeholders}) ORDER BY created_at DESC LIMIT 200"
                ),
                list.to_vec(),
            )
        }
        None => (
            "SELECT batch_id, esun_ref, amount_minor, currency, status, created_by, created_at, completed_at \
             FROM bankpay_batches ORDER BY created_at DESC LIMIT 200"
                .to_string(),
            vec![],
        ),
    };
    let mut qb = sqlx::query_as::<_, BatchRowFull>(sqlx::AssertSqlSafe(sql.as_str()));
    for b in &bind_list {
        qb = qb.bind(b);
    }
    qb.fetch_all(pool).await.unwrap_or_default()
}

/// 批次狀態 →（中文標籤，徽章類別）。
pub fn batch_badge(status: &str) -> (&'static str, &'static str) {
    match status {
        "COMPLETED" => ("已完成", "badge-success"),
        "SUBMITTED" => ("處理中", "badge-warning"),
        "FAILED" => ("失敗", "badge-error"),
        "REJECTED" => ("退件", "badge-warning"),
        _ => ("等待提交", "badge-info"),
    }
}

/// 批次顯示檢視（模板零邏輯：格式化金額、中文狀態、空值替代皆在此計算）。
#[derive(Debug, Clone)]
pub struct BatchView {
    pub batch_id: String,
    pub esun_ref: String,
    pub amount_display: String,
    pub currency: String,
    pub status: String,
    pub status_label: String,
    pub status_class: String,
    pub created_by: String,
    pub created_at: String,
    pub completed_at: String,
}

fn to_batch_views(rows: Vec<BatchRowFull>) -> Vec<BatchView> {
    rows.into_iter()
        .map(|r| {
            let (status_label, status_class) = batch_badge(&r.status);
            BatchView {
                batch_id: r.batch_id,
                esun_ref: r.esun_ref.unwrap_or_else(|| "—".to_string()),
                amount_display: flaps_core::format_minor(r.amount_minor, 2),
                currency: r.currency,
                status: r.status,
                status_label: status_label.to_string(),
                status_class: status_class.to_string(),
                created_by: r.created_by,
                created_at: r.created_at,
                completed_at: r.completed_at.unwrap_or_else(|| "—".to_string()),
            }
        })
        .collect()
}

#[derive(Template)]
#[template(path = "bankpay_batches.html")]
pub struct BankPayBatchesTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub batches: Vec<BatchView>,
    pub total: usize,
}

/// GET /bankpay/batches — 批次總覽（最近 200 筆）。
pub async fn bankpay_batches_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Response {
    let batches = to_batch_views(batches_query(&state.pool, None).await);
    let total = batches.len();
    render(BankPayBatchesTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        batches,
        total,
    })
}

#[derive(Template)]
#[template(path = "bankpay_batch_detail.html")]
pub struct BankPayBatchDetailTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub batch: Option<BatchView>,
    pub audits: Vec<AuditView>,
    pub not_found: bool,
}

/// GET /bankpay/batches/:id — 批次詳情（狀態＋相關稽核事件）。
pub async fn bankpay_batch_detail(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(batch_id): Path<String>,
) -> Response {
    let batch = sqlx::query_as::<_, BatchRowFull>(
        "SELECT batch_id, esun_ref, amount_minor, currency, status, created_by, created_at, completed_at \
         FROM bankpay_batches WHERE batch_id = ?",
    )
    .bind(&batch_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or_default()
    .map(|b| to_batch_views(vec![b]).remove(0));
    let audits = audit_rows_page(&state.pool, None, Some("BANKPAY"), None, None, 200, 0)
        .await
        .into_iter()
        .filter(|a| a.resource == batch_id)
        .collect();
    render(BankPayBatchDetailTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        not_found: batch.is_none(),
        batch,
        audits,
    })
}

#[derive(Template)]
#[template(path = "bankpay_exceptions.html")]
pub struct BankPayExceptionsTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub exceptions: Vec<BatchView>,
}

/// GET /bankpay/exceptions — 失敗/退件工作佇列（重試：重新匯出即 PENDING）。
pub async fn bankpay_exceptions_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Response {
    let exceptions =
        to_batch_views(batches_query(&state.pool, Some(&["FAILED", "REJECTED"])).await);
    render(BankPayExceptionsTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        exceptions,
    })
}

// ---------------------------------------------------------------------------
// 7. 匯入歷史（資料匯入工作室）
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "import_history.html")]
pub struct ImportHistoryTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub rows: Vec<AuditView>,
}

/// GET /import/history — 匯入相關稽核歷史（IMPORT_* 事件）。
pub async fn import_history_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Response {
    let rows = audit_rows_page(&state.pool, None, Some("IMPORT"), None, None, 100, 0).await;
    render(ImportHistoryTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        rows,
    })
}

// ---------------------------------------------------------------------------
// 8. 說明（角色作業指引）
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "help.html")]
pub struct HelpTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub role_code: String,
}

/// GET /help — 使用說明（依角色呈現對應工作流程）。
pub async fn help_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Response {
    let role_code = primary_role(&state.pool, &session.user_id).await;
    render(HelpTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        role_code,
    })
}

// ---------------------------------------------------------------------------
// 9. 系統狀態頁（HTML；meta refresh 零 JS 自動更新）
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "status_page.html")]
pub struct StatusPageTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub modules: Vec<crate::api::StatusChip>,
    pub overall: String,
    pub timestamp: String,
}

/// GET /status — 系統狀態頁（與 /api/status 同源；30 秒 meta refresh）。
pub async fn status_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Response {
    let modules_map = crate::status::probe_modules(&state).await;
    let labels: [(&str, &str); 6] = [
        ("database", "資料庫"),
        ("search", "搜尋"),
        ("sync", "同步"),
        ("com", "COM 橋接"),
        ("ai", "AI"),
        ("bankpay", "銀行支付"),
    ];
    let modules = labels
        .iter()
        .filter_map(|(key, label)| {
            modules_map.get(*key).map(|m| crate::api::StatusChip {
                key: (*key).to_string(),
                label: (*label).to_string(),
                status: m.status.to_string(),
                status_label: crate::status::zh_label(&m.status).to_string(),
                detail: m.message.clone().unwrap_or_default(),
                last_checked: m.last_checked.clone(),
            })
        })
        .collect();
    let overall = crate::status::overall_status(&modules_map);
    let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    render(StatusPageTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        modules,
        overall,
        timestamp,
    })
}

// ---------------------------------------------------------------------------
// 10. 個人設定（改密碼）
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub roles: Vec<String>,
    pub saved: bool,
    pub error: Option<String>,
}

async fn roles_of(pool: &SqlitePool, user_id: &str) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT r.role_name FROM user_roles ur JOIN roles r ON ur.role_id = r.role_id WHERE ur.user_id = ?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

/// GET /profile — 個人設定（帳號資訊＋改密碼）。
pub async fn profile_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let roles = roles_of(&state.pool, &session.user_id).await;
    render(ProfileTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        roles,
        saved: q.get("saved").map(|v| v == "1").unwrap_or(false),
        error: None,
    })
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordForm {
    pub csrf_token: String,
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

/// POST /profile/password — 變更密碼（驗現值→Argon2id→稽核→PRG）。
pub async fn profile_change_password(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(form): Form<ChangePasswordForm>,
) -> Response {
    let render_error = |msg: String, status: StatusCode| {
        let pool = state.pool.clone();
        let user_id = session.user_id.clone();
        let csrf = session.csrf_token.clone();
        async move {
            let roles: Vec<String> = sqlx::query_scalar(
                "SELECT r.role_name FROM user_roles ur JOIN roles r ON ur.role_id = r.role_id WHERE ur.user_id = ?",
            )
            .bind(&user_id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();
            (
                status,
                Html(
                    ProfileTemplate {
                        acting_user: user_id.clone(),
                        csrf_token: csrf,
                        roles,
                        saved: false,
                        error: Some(msg),
                    }
                    .render()
                    .unwrap_or_else(|e| format!("模板渲染失敗：{e}")),
                ),
            )
                .into_response()
        }
    };

    if let Err(e) = verify_csrf_token(&session, &form.csrf_token) {
        return render_error(e.message(), e.status()).await;
    }
    if form.new_password.len() < 8 {
        return render_error(
            "驗證失敗：新密碼至少 8 個字元。處理方式：設定更長的密碼後重試。".to_string(),
            StatusCode::BAD_REQUEST,
        )
        .await;
    }
    if form.new_password != form.confirm_password {
        return render_error(
            "驗證失敗：兩次輸入的新密碼不一致。處理方式：重新輸入。".to_string(),
            StatusCode::BAD_REQUEST,
        )
        .await;
    }
    let stored: Option<String> =
        sqlx::query_scalar("SELECT password_hash FROM users WHERE user_id = ?")
            .bind(&session.user_id)
            .fetch_optional(&state.pool)
            .await
            .unwrap_or_default();
    let Some(stored) = stored else {
        return render_error(
            "帳號不存在或已停用。處理方式：聯絡系統管理員。".to_string(),
            StatusCode::FORBIDDEN,
        )
        .await;
    };
    if !crate::auth::verify_password(&form.current_password, &stored) {
        return render_error(
            "驗證失敗：目前密碼不正確。處理方式：確認後重試。".to_string(),
            StatusCode::FORBIDDEN,
        )
        .await;
    }
    let Ok(new_hash) = crate::auth::hash_password(&form.new_password) else {
        return render_error(
            "系統錯誤：密碼雜湊失敗。處理方式：稍後重試或聯絡管理員。".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .await;
    };
    if sqlx::query("UPDATE users SET password_hash = ? WHERE user_id = ?")
        .bind(&new_hash)
        .bind(&session.user_id)
        .execute(&state.pool)
        .await
        .is_err()
    {
        return render_error(
            "系統錯誤：密碼更新失敗。處理方式：稍後重試。".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .await;
    }
    write_event(
        &state.pool,
        "USER_PASSWORD_CHANGE",
        &session.user_id,
        serde_json::json!({ "self_service": true }),
    )
    .await;
    Redirect::to("/profile?saved=1").into_response()
}

// ---------------------------------------------------------------------------
// 11. 角色權限矩陣（管理）
// ---------------------------------------------------------------------------

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct RoleRow {
    pub role_id: String,
    pub role_name: String,
    pub role_code: String,
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct PermissionRow {
    pub permission_id: String,
    pub permission_code: String,
    pub permission_name: String,
}

#[derive(Template)]
#[template(path = "admin_roles.html")]
pub struct AdminRolesTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub roles: Vec<RoleRow>,
    pub permissions: Vec<PermissionRow>,
    /// "role_id|permission_id"（模板以字串比對呈現 ✓/—）。
    pub grants: Vec<String>,
}

impl AdminRolesTemplate {
    /// 是否已授予（矩陣 ✓/— 判定；模板零邏輯原則下的唯一允許呼叫點）。
    pub fn has_grant(&self, role_id: &str, permission_id: &str) -> bool {
        let key = format!("{role_id}|{permission_id}");
        self.grants.iter().any(|g| g == &key)
    }
}

/// GET /admin/roles — 角色權限矩陣（ROLE_MANAGE）。
pub async fn admin_roles_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Response {
    let roles: Vec<RoleRow> =
        sqlx::query_as("SELECT role_id, role_name, role_code FROM roles ORDER BY role_code")
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();
    let permissions: Vec<PermissionRow> = sqlx::query_as(
        "SELECT permission_id, permission_code, permission_name FROM permissions \
         WHERE is_active = 1 ORDER BY permission_code",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let grants: Vec<String> =
        sqlx::query_scalar("SELECT role_id || '|' || permission_id FROM role_permissions")
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();
    render(AdminRolesTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        roles,
        permissions,
        grants,
    })
}

// ---------------------------------------------------------------------------
// 12. 活動工作階段（管理）
// ---------------------------------------------------------------------------

#[derive(Debug, sqlx::FromRow)]
pub struct SessionRow {
    pub username: String,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Template)]
#[template(path = "admin_sessions.html")]
pub struct AdminSessionsTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub sessions: Vec<SessionRow>,
    pub purged: bool,
}

/// GET /admin/sessions — 活動工作階段（ROLE_MANAGE）。
pub async fn admin_sessions_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let sessions: Vec<SessionRow> = sqlx::query_as(
        "SELECT u.username, s.created_at, s.expires_at FROM sessions s \
         JOIN users u ON s.user_id = u.user_id WHERE s.expires_at > datetime('now') \
         ORDER BY s.created_at DESC LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    render(AdminSessionsTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        sessions,
        purged: q.get("purged").map(|v| v == "1").unwrap_or(false),
    })
}

#[derive(Debug, Deserialize)]
pub struct CsrfOnlyForm {
    pub csrf_token: String,
}

/// POST /admin/sessions/purge — 清除過期工作階段（ROLE_MANAGE + CSRF + PRG）。
pub async fn admin_sessions_purge(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(form): Form<CsrfOnlyForm>,
) -> Response {
    if verify_csrf_token(&session, &form.csrf_token).is_err() {
        return (StatusCode::FORBIDDEN, "CSRF 驗證失敗").into_response();
    }
    if state
        .permissions
        .has_permission_db(&session.user_id, "ROLE_MANAGE")
        .await
    {
        let _ = crate::auth::purge_expired_sessions(&state.pool).await;
    }
    Redirect::to("/admin/sessions?purged=1").into_response()
}

// ---------------------------------------------------------------------------
// 13. 系統資訊（管理）
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "admin_system.html")]
pub struct AdminSystemTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub version: String,
    pub migration_count: i64,
    pub transaction_count: i64,
    pub account_count: i64,
    pub audit_count: i64,
    pub batch_count: i64,
    pub session_count: i64,
}

/// GET /admin/system — 系統資訊（版本、遷移、資料量；ROLE_MANAGE）。
pub async fn admin_system_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Response {
    let count = |sql: &'static str| {
        let pool = &state.pool;
        async move {
            sqlx::query_scalar::<_, i64>(sql)
                .fetch_one(pool)
                .await
                .unwrap_or(0)
        }
    };
    let migration_count = count("SELECT COUNT(*) FROM _sqlx_migrations").await;
    let transaction_count = count("SELECT COUNT(*) FROM transactions").await;
    let account_count = count("SELECT COUNT(*) FROM accounts").await;
    let audit_count = count("SELECT COUNT(*) FROM audit_log").await;
    let batch_count = count("SELECT COUNT(*) FROM bankpay_batches").await;
    let session_count =
        count("SELECT COUNT(*) FROM sessions WHERE expires_at > datetime('now')").await;
    render(AdminSystemTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        migration_count,
        transaction_count,
        account_count,
        audit_count,
        batch_count,
        session_count,
    })
}
