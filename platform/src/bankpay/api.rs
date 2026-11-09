//! E.Sun BankPay API 端點處理函數。
//!
//! 提供匯出、匯入、查詢三大端點與設定管理，並整合稽核軌跡。
//! 路由註冊統一在 `api.rs` 的 `protected_api` 群組（`require_session_api`
//! 中間件注入 `Session` 並對未登入回 401）。
//!
//! 安全與正確性設計：
//! - **CSRF（T-08）**：所有變更類 POST 驗證 `X-CSRF-Token` 標頭與
//!   session token 一致（JSON API 的同步器權杖模式）。
//! - **RBAC**：匯出/匯入需 `LEDGER_WRITE`，查詢需 `LEDGER_READ`，
//!   設定需 `ROLE_MANAGE`（沿用既有種子權限，不新增規則）。
//! - **稽核 fail-closed**：每筆作業寫入 `audit_log`；稽核寫入失敗即回
//!   500（配合批次唯一索引的冪等重放，重試安全，不會重複支付）。
//! - **冪等**：`bankpay_batches.batch_id` 主鍵唯一；已 SUBMITTED/COMPLETED
//!   的批次重放時不重複呼叫銀行，直接回傳既有狀態。
//! - **路徑白名單**：匯入檔案必須位於 `data/bankpay/inbound/` 內且為
//!   `.csv`，防止任意伺服器檔案讀取（CWE-22）。
//! - **錯誤型別**：`ApiError(StatusCode, String)` 為小型錯誤（clippy
//!   `result_large_err` 合規），僅在 handler 邊界轉為 `Response`。

use askama::Template;
use axum::extract::{Form, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Json, Redirect, Response};
use axum::Extension;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api::{AppState, StatusChip};
use crate::auth::Session;
use crate::bankpay::client::EsunClient;
use crate::bankpay::model::{
    validate_amount, validate_currency, validate_identifier, ExportRequest, ExportResponse,
    ImportRequest, ImportResponse, QueryRequest, QueryResponse,
};
use crate::bankpay::state::{validate_transition, BankPayStatus};

/// 匯入檔白名單目錄（相對於工作目錄；僅接受其內的 `.csv`）。
const IMPORT_DIR: &str = "data/bankpay/inbound";

// ---------------------------------------------------------------------------
// 錯誤型別與共用輔助
// ---------------------------------------------------------------------------

/// 小型 API 錯誤（問題＋可能原因＋處理方式由呼叫端組入訊息）。
#[derive(Debug)]
struct ApiError(StatusCode, String);

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, message.into())
    }

    fn forbidden(message: impl Into<String>) -> Self {
        Self(StatusCode::FORBIDDEN, message.into())
    }

    fn internal(message: impl Into<String>) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, message.into())
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self(StatusCode::CONFLICT, message.into())
    }

    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

type ApiResult<T> = Result<T, ApiError>;

/// CSRF 同步器權杖驗證（同步器權杖模式：JSON API 以 `X-CSRF-Token` 標頭、
/// HTML 表單以 `csrf_token` 欄位攜帶，兩者統一於本函式比對 session token）。
fn verify_csrf_token(session: &Session, provided: &str) -> ApiResult<()> {
    if !provided.is_empty() && provided == session.csrf_token {
        Ok(())
    } else {
        Err(ApiError::forbidden(
            "CSRF 驗證失敗。問題：缺少或不正確的 CSRF 權杖；可能原因：請求非由本平台表單發起，或權杖已輪替；處理方式：重新登入取得新 session 後，以頁面顯示的權杖重試。",
        ))
    }
}

/// 從 JSON API 請求標頭取出 CSRF 權杖（供 verify_csrf_token 使用）。
fn csrf_from_header(headers: &HeaderMap) -> &str {
    headers
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
}

/// 寫入稽核軌跡（fail-closed：呼叫端必須處理 Err）。
///
/// `event_data` 為單一 JSON 物件：`{"request": …, "response": …}`，
/// 佔位符與 bind 嚴格一一對應（3 個 `?`、3 個 bind）。
async fn write_audit(
    pool: &SqlitePool,
    event_type: &str,
    performed_by: &str,
    request_payload: serde_json::Value,
    response_payload: Option<serde_json::Value>,
) -> Result<(), sqlx::Error> {
    let event_data = json!({
        "request": request_payload,
        "response": response_payload,
    });
    sqlx::query(
        "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) \
         VALUES (?, NULL, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(event_type)
    .bind(event_data.to_string())
    .bind(performed_by)
    .execute(pool)
    .await?;
    Ok(())
}

/// 稽核寫入失敗 → 500（fail-closed；重試安全，見模組注釋）。
fn audit_error(e: sqlx::Error) -> ApiError {
    ApiError::internal(format!(
        "稽核寫入失敗，作業未確認。問題：{e}；可能原因：資料庫忙碌或鎖定；處理方式：稍後重試（批次唯一索引保證重放安全）。"
    ))
}

/// `bankpay_batches` 列（顯式欄位清單，避免 SELECT * 欄數耦合）。
#[derive(Debug, sqlx::FromRow)]
struct BatchRow {
    pub batch_id: String,
    pub esun_ref: Option<String>,
    pub amount_minor: i64,
    pub currency: String,
    pub status: String,
}

fn parse_status(raw: &str) -> Option<BankPayStatus> {
    match raw {
        "PENDING" => Some(BankPayStatus::Pending),
        "SUBMITTED" => Some(BankPayStatus::Submitted),
        "COMPLETED" => Some(BankPayStatus::Completed),
        "FAILED" => Some(BankPayStatus::Failed),
        "REJECTED" => Some(BankPayStatus::Rejected),
        _ => None,
    }
}

/// 建立批次列（PENDING）；已存在時不變更（冪等 SSOT：主鍵唯一索引）。
async fn insert_batch_if_absent(
    pool: &SqlitePool,
    batch_id: &str,
    amount_minor: i64,
    currency: &str,
    created_by: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO bankpay_batches \
             (batch_id, esun_ref, amount_minor, currency, status, created_by, created_at) \
         VALUES (?, NULL, ?, ?, 'PENDING', ?, strftime('%Y-%m-%dT%H:%M:%fZ','now')) \
         ON CONFLICT(batch_id) DO NOTHING",
    )
    .bind(batch_id)
    .bind(amount_minor)
    .bind(currency)
    .bind(created_by)
    .execute(pool)
    .await?;
    Ok(())
}

async fn load_batch(pool: &SqlitePool, batch_id: &str) -> Result<Option<BatchRow>, sqlx::Error> {
    sqlx::query_as::<_, BatchRow>(
        "SELECT batch_id, esun_ref, amount_minor, currency, status \
         FROM bankpay_batches WHERE batch_id = ?",
    )
    .bind(batch_id)
    .fetch_optional(pool)
    .await
}

async fn load_batch_by_ref(
    pool: &SqlitePool,
    esun_ref: &str,
) -> Result<Option<BatchRow>, sqlx::Error> {
    sqlx::query_as::<_, BatchRow>(
        "SELECT batch_id, esun_ref, amount_minor, currency, status \
         FROM bankpay_batches WHERE esun_ref = ?",
    )
    .bind(esun_ref)
    .fetch_optional(pool)
    .await
}

/// 依狀態機推進批次狀態（非法轉換直接拒絕，絕不寫入）。
async fn advance_batch(
    pool: &SqlitePool,
    batch_id: &str,
    from: &BankPayStatus,
    to: &BankPayStatus,
    esun_ref: Option<&str>,
) -> ApiResult<()> {
    validate_transition(from, to).map_err(|msg| {
        ApiError::conflict(format!(
            "狀態機拒絕更新：{msg}。處理方式：請先查詢目前批次狀態，依畫面建議操作。"
        ))
    })?;
    let is_terminal = matches!(to, BankPayStatus::Completed | BankPayStatus::Rejected);
    let result = if let Some(ref_at) = esun_ref {
        if is_terminal {
            sqlx::query(
                "UPDATE bankpay_batches SET status = ?, esun_ref = COALESCE(?, esun_ref), \
                 completed_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE batch_id = ?",
            )
            .bind(to.to_string())
            .bind(ref_at)
            .bind(batch_id)
            .execute(pool)
            .await
        } else {
            sqlx::query(
                "UPDATE bankpay_batches SET status = ?, esun_ref = COALESCE(?, esun_ref) \
                 WHERE batch_id = ?",
            )
            .bind(to.to_string())
            .bind(ref_at)
            .bind(batch_id)
            .execute(pool)
            .await
        }
    } else if is_terminal {
        sqlx::query(
            "UPDATE bankpay_batches SET status = ?, \
             completed_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE batch_id = ?",
        )
        .bind(to.to_string())
        .bind(batch_id)
        .execute(pool)
        .await
    } else {
        sqlx::query("UPDATE bankpay_batches SET status = ? WHERE batch_id = ?")
            .bind(to.to_string())
            .bind(batch_id)
            .execute(pool)
            .await
    };
    result
        .map(|_| ())
        .map_err(|e| ApiError::internal(format!("批次狀態更新失敗：{e}")))
}

/// 解析匯入檔路徑：僅接受白名單目錄內的 `.csv`（防 CWE-22 路徑穿越）。
fn resolve_import_path(raw: &str) -> ApiResult<std::path::PathBuf> {
    if raw.trim().is_empty() {
        return Err(ApiError::bad_request(
            "驗證失敗：file_path 不得為空。處理方式：請提供白名單目錄內的檔名。",
        ));
    }
    let requested = std::path::Path::new(raw);
    let candidate = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        std::path::Path::new(IMPORT_DIR).join(raw)
    };
    let base = std::path::Path::new(IMPORT_DIR);
    if let Err(e) = std::fs::create_dir_all(base) {
        return Err(ApiError::internal(format!(
            "匯入目錄不可用：{e}。問題：無法建立 {IMPORT_DIR}；處理方式：檢查工作目錄寫入權限後重試。"
        )));
    }
    let (Ok(canon_path), Ok(canon_base)) = (candidate.canonicalize(), base.canonicalize()) else {
        return Err(ApiError::bad_request(format!(
            "驗證失敗：檔案「{raw}」不存在或無法解析。處理方式：將檔案放置於 {IMPORT_DIR} 後以檔名重試。"
        )));
    };
    if !canon_path.starts_with(&canon_base) {
        return Err(ApiError::bad_request(format!(
            "驗證失敗：路徑「{raw}」超出白名單。問題：匯入僅允許 {IMPORT_DIR} 目錄內的檔案（防目錄穿越）；可能原因：使用 .. 或絕對路徑指向其他目錄；處理方式：將檔案放入 {IMPORT_DIR} 後以相對檔名重試。"
        )));
    }
    if canon_path.extension().and_then(|e| e.to_str()) != Some("csv") {
        return Err(ApiError::bad_request(
            "驗證失敗：匯入檔必須為 .csv。處理方式：確認副檔名後重試。",
        ));
    }
    Ok(canon_path)
}

/// RBAC 檢查（沿用既有種子權限碼）。
async fn require_permission(state: &AppState, user_id: &str, code: &str) -> ApiResult<()> {
    if state.permissions.has_permission_db(user_id, code).await {
        Ok(())
    } else {
        Err(ApiError::forbidden(format!(
            "無權限執行此操作（需要 {code} 權限）。處理方式：聯絡管理員授權後重試。"
        )))
    }
}

// ---------------------------------------------------------------------------
// 匯出
// ---------------------------------------------------------------------------

/// POST /api/bankpay/export — 匯出付款批次（LEDGER_WRITE + CSRF + 冪等）。
pub async fn export_handler(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<ExportRequest>,
) -> Response {
    match export_inner(&state, &session, csrf_from_header(&headers), &req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn export_inner(
    state: &AppState,
    session: &Session,
    csrf: &str,
    req: &ExportRequest,
) -> ApiResult<ExportResponse> {
    verify_csrf_token(session, csrf)?;
    validate_identifier(&req.batch_id, "batch_id").map_err(ApiError::bad_request)?;
    validate_amount(req.amount_minor).map_err(ApiError::bad_request)?;
    validate_currency(&req.currency).map_err(ApiError::bad_request)?;
    require_permission(state, &session.user_id, "LEDGER_WRITE").await?;

    // 冪等 SSOT：先確保批次列存在（ON CONFLICT DO NOTHING），再讀回現況。
    insert_batch_if_absent(
        &state.pool,
        &req.batch_id,
        req.amount_minor,
        &req.currency,
        &session.user_id,
    )
    .await
    .map_err(|e| ApiError::internal(format!("批次建立失敗：{e}")))?;
    let batch = load_batch(&state.pool, &req.batch_id)
        .await
        .map_err(|e| ApiError::internal(format!("批次讀取失敗：{e}")))?
        .ok_or_else(|| ApiError::internal("批次建立後讀取失敗（資料庫不一致）"))?;

    let current = parse_status(&batch.status).ok_or_else(|| {
        ApiError::internal(format!(
            "批次狀態「{}」無法解析（資料庫一致性錯誤）",
            batch.status
        ))
    })?;

    // 已送出/已完成：冪等重放——不重複呼叫銀行，回傳既有狀態。
    if matches!(current, BankPayStatus::Submitted | BankPayStatus::Completed) {
        write_audit(
            &state.pool,
            "BANKPAY_EXPORT_SUCCESS",
            &session.user_id,
            serde_json::to_value(req).unwrap_or_default(),
            Some(json!({ "replay": true, "status": batch.status })),
        )
        .await
        .map_err(audit_error)?;
        return Ok(ExportResponse {
            status: batch.status,
            batch_id: batch.batch_id,
            esun_ref: batch.esun_ref,
        });
    }

    // FAILED/REJECTED → PENDING（依狀態機允許人工複核後重試），再送出。
    if matches!(current, BankPayStatus::Failed | BankPayStatus::Rejected) {
        advance_batch(
            &state.pool,
            &req.batch_id,
            &current,
            &BankPayStatus::Pending,
            None,
        )
        .await?;
    }

    let client = EsunClient::new(state.bankpay_config.lock().await.clone());
    match client.export(req).await {
        Ok(resp) => {
            advance_batch(
                &state.pool,
                &req.batch_id,
                &BankPayStatus::Pending,
                &BankPayStatus::Submitted,
                resp.esun_ref.as_deref(),
            )
            .await?;
            write_audit(
                &state.pool,
                "BANKPAY_EXPORT_SUCCESS",
                &session.user_id,
                serde_json::to_value(req).unwrap_or_default(),
                Some(serde_json::to_value(&resp).unwrap_or_default()),
            )
            .await
            .map_err(audit_error)?;
            Ok(resp)
        }
        Err(e) => {
            advance_batch(
                &state.pool,
                &req.batch_id,
                &BankPayStatus::Pending,
                &BankPayStatus::Failed,
                None,
            )
            .await?;
            write_audit(
                &state.pool,
                "BANKPAY_EXPORT_FAILURE",
                &session.user_id,
                serde_json::to_value(req).unwrap_or_default(),
                Some(json!({ "error": e })),
            )
            .await
            .map_err(audit_error)?;
            Err(ApiError::internal(format!("銀行支付匯出失敗：{e}")))
        }
    }
}

// ---------------------------------------------------------------------------
// 匯入
// ---------------------------------------------------------------------------

/// POST /api/bankpay/import — 匯入銀行回應檔（LEDGER_WRITE + CSRF + 白名單）。
pub async fn import_handler(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<ImportRequest>,
) -> Response {
    match import_inner(&state, &session, csrf_from_header(&headers), &req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn import_inner(
    state: &AppState,
    session: &Session,
    csrf: &str,
    req: &ImportRequest,
) -> ApiResult<ImportResponse> {
    verify_csrf_token(session, csrf)?;
    require_permission(state, &session.user_id, "LEDGER_WRITE").await?;

    let path = resolve_import_path(&req.file_path)?;
    let import_req = ImportRequest {
        file_path: path.to_string_lossy().to_string(),
    };

    let client = EsunClient::new(state.bankpay_config.lock().await.clone());
    match client.import(&import_req).await {
        Ok(resp) => {
            write_audit(
                &state.pool,
                "BANKPAY_IMPORT_SUCCESS",
                &session.user_id,
                json!({ "file_path": import_req.file_path }),
                Some(serde_json::to_value(&resp).unwrap_or_default()),
            )
            .await
            .map_err(audit_error)?;
            Ok(resp)
        }
        Err(e) => {
            write_audit(
                &state.pool,
                "BANKPAY_IMPORT_FAILURE",
                &session.user_id,
                json!({ "file_path": import_req.file_path }),
                Some(json!({ "error": e })),
            )
            .await
            .map_err(audit_error)?;
            Err(ApiError::internal(format!("銀行支付匯入失敗：{e}")))
        }
    }
}

// ---------------------------------------------------------------------------
// 查詢
// ---------------------------------------------------------------------------

/// GET /api/bankpay/query — 查詢交易狀態（LEDGER_READ）。
///
/// 狀態推進完全取自銀行（模擬器）回應，非攻擊者可控輸入；GET 輪詢僅同步
/// 本地批次與銀行真相，無 CSRF 風險面。
pub async fn query_handler(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(req): Query<QueryRequest>,
) -> Response {
    match query_inner(&state, &session, &req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn query_inner(
    state: &AppState,
    session: &Session,
    req: &QueryRequest,
) -> ApiResult<QueryResponse> {
    validate_identifier(&req.esun_ref, "esun_ref").map_err(ApiError::bad_request)?;
    require_permission(state, &session.user_id, "LEDGER_READ").await?;

    let client = EsunClient::new(state.bankpay_config.lock().await.clone());
    match client.query(req).await {
        Ok(mut resp) => {
            // 金額/幣別以本地批次帳為準（客戶端不虛構財務數值）。
            let batch = load_batch_by_ref(&state.pool, &req.esun_ref)
                .await
                .map_err(|e| ApiError::internal(format!("批次讀取失敗：{e}")))?;
            if let Some(row) = &batch {
                resp.amount_minor = Some(row.amount_minor);
                resp.currency = Some(row.currency.clone());
                if let (Some(from), Some(to)) =
                    (parse_status(&row.status), parse_status(&resp.status))
                {
                    if validate_transition(&from, &to).is_ok() {
                        advance_batch(&state.pool, &row.batch_id, &from, &to, None).await?;
                    }
                }
            }
            write_audit(
                &state.pool,
                "BANKPAY_QUERY_SUCCESS",
                &session.user_id,
                serde_json::to_value(req).unwrap_or_default(),
                Some(serde_json::to_value(&resp).unwrap_or_default()),
            )
            .await
            .map_err(audit_error)?;
            Ok(resp)
        }
        Err(e) => {
            write_audit(
                &state.pool,
                "BANKPAY_QUERY_FAILURE",
                &session.user_id,
                serde_json::to_value(req).unwrap_or_default(),
                Some(json!({ "error": e })),
            )
            .await
            .map_err(audit_error)?;
            Err(ApiError::internal(format!("銀行支付查詢失敗：{e}")))
        }
    }
}

// ---------------------------------------------------------------------------
// 設定
// ---------------------------------------------------------------------------

/// GET /api/bankpay/config — 讀取銀行連線設定（ROLE_MANAGE）。
pub async fn config_get_handler(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Response {
    if state
        .permissions
        .has_permission_db(&session.user_id, "ROLE_MANAGE")
        .await
    {
        (
            StatusCode::OK,
            Json(state.bankpay_config.lock().await.clone()),
        )
            .into_response()
    } else {
        ApiError::forbidden("無權限讀取銀行支付設定（需要 ROLE_MANAGE 權限）。").into_response()
    }
}

/// 更新銀行支付設定請求。
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigUpdateRequest {
    pub esun_mock_mode: Option<bool>,
    pub esun_url: Option<String>,
    pub source_system_id: Option<String>,
    pub customer_id: Option<String>,
    pub payer_account: Option<String>,
}

/// POST /api/bankpay/config — 更新設定（ROLE_MANAGE + CSRF；erp_config 為 SSOT）。
pub async fn config_post_handler(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<ConfigUpdateRequest>,
) -> Response {
    match config_update_inner(&state, &session, csrf_from_header(&headers), req).await {
        Ok(config) => (StatusCode::OK, Json(config)).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn config_update_inner(
    state: &AppState,
    session: &Session,
    csrf: &str,
    req: ConfigUpdateRequest,
) -> ApiResult<crate::bankpay::model::EsunConfig> {
    verify_csrf_token(session, csrf)?;
    require_permission(state, &session.user_id, "ROLE_MANAGE").await?;

    // 先序列化請求（欄位 Option<String> 隨後被 move）。
    let req_json = serde_json::to_value(&req).unwrap_or_default();

    let mut config = state.bankpay_config.lock().await.clone();
    if let Some(v) = req.esun_mock_mode {
        config.esun_mock_mode = v;
    }
    if let Some(v) = req.esun_url {
        if !v.is_empty() && !v.starts_with("https://") {
            return Err(ApiError::bad_request(
                "驗證失敗：esun_url 必須為 https:// 開頭。問題：明文 HTTP 會洩漏支付資料；處理方式：提供正式 HTTPS 端點。",
            ));
        }
        config.esun_url = v;
    }
    if let Some(v) = req.source_system_id {
        validate_identifier(&v, "source_system_id").map_err(ApiError::bad_request)?;
        config.source_system_id = Some(v);
    }
    if let Some(v) = req.customer_id {
        validate_identifier(&v, "customer_id").map_err(ApiError::bad_request)?;
        config.customer_id = Some(v);
    }
    if let Some(v) = req.payer_account {
        config.payer_account = Some(v);
    }

    // 先持久化（erp_config 為設定持久層 SSOT；冪等 UPSERT），成功後才更新
    // 記憶體；失敗即回報 500 且記憶體維持原值（fail-closed 一致性）。
    if let Err(e) = crate::bankpay::save_config(&state.pool, &config).await {
        return Err(ApiError::internal(format!(
            "銀行支付設定儲存失敗（可能原因：資料庫忙碌；處理方式：稍後重試）：{e}"
        )));
    }

    *state.bankpay_config.lock().await = config.clone();

    write_audit(
        &state.pool,
        "BANKPAY_CONFIG_UPDATE",
        &session.user_id,
        req_json,
        Some(serde_json::to_value(&config).unwrap_or_default()),
    )
    .await
    .map_err(audit_error)?;

    Ok(config)
}

// ---------------------------------------------------------------------------
// 健康與狀態機除錯
// ---------------------------------------------------------------------------

/// 銀行支付健康檢查回應。
#[derive(Debug, Serialize)]
pub struct BankPayHealthResponse {
    pub status: String,
    pub mock_mode: bool,
    pub configured: bool,
}

/// GET /api/bankpay/health — 銀行支付模組健康狀態。
pub async fn bankpay_health_handler(State(state): State<AppState>) -> Response {
    let config = state.bankpay_config.lock().await;
    (
        StatusCode::OK,
        Json(BankPayHealthResponse {
            status: "ok".to_string(),
            mock_mode: config.esun_mock_mode,
            configured: config.source_system_id.is_some() && config.customer_id.is_some(),
        }),
    )
        .into_response()
}

/// 狀態機轉換驗證請求（開發除錯；純函式，無副作用）。
#[derive(Debug, Deserialize)]
pub struct TransitionRequest {
    pub from: String,
    pub to: String,
}

/// 狀態機轉換驗證回應。
#[derive(Debug, Serialize)]
pub struct TransitionResponse {
    pub valid: bool,
    pub message: Option<String>,
}

/// POST /api/bankpay/transition — 狀態機轉換驗證（除錯用；不改任何狀態）。
pub async fn transition_handler(axum::Json(req): axum::Json<TransitionRequest>) -> Response {
    let parse = |s: &str, label: &str| -> ApiResult<BankPayStatus> {
        match s {
            "Pending" => Ok(BankPayStatus::Pending),
            "Submitted" => Ok(BankPayStatus::Submitted),
            "Completed" => Ok(BankPayStatus::Completed),
            "Failed" => Ok(BankPayStatus::Failed),
            "Rejected" => Ok(BankPayStatus::Rejected),
            _ => Err(ApiError::bad_request(format!(
                "無效的{label}狀態：{s}。問題：僅接受 Pending/Submitted/Completed/Failed/Rejected；處理方式：修正後重試。"
            ))),
        }
    };
    let (from, to) = match (parse(&req.from, "來源"), parse(&req.to, "目標")) {
        (Ok(f), Ok(t)) => (f, t),
        (Err(e), _) | (_, Err(e)) => return e.into_response(),
    };

    let response = match validate_transition(&from, &to) {
        Ok(()) => TransitionResponse {
            valid: true,
            message: Some("狀態轉換合法".to_string()),
        },
        Err(msg) => TransitionResponse {
            valid: false,
            message: Some(msg),
        },
    };
    (StatusCode::OK, Json(response)).into_response()
}

// ---------------------------------------------------------------------------
// 儀表板資料（供 api.rs build_index 組裝 BankPay 卡片）
// ---------------------------------------------------------------------------

/// 銀行支付批次摘要列（儀表板最近 5 筆）。
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct BatchSummaryRow {
    pub batch_id: String,
    pub esun_ref: Option<String>,
    pub amount_minor: i64,
    pub currency: String,
    pub status: String,
    pub created_at: String,
}

/// 銀行支付儀表板卡片（server-render，零 JS）。
#[derive(Debug, Serialize)]
pub struct BankPayCard {
    /// 目前連線模式（模擬模式必須讓使用者一眼辨識）。
    pub mock_mode: bool,
    /// 進行中（PENDING + SUBMITTED）批次數。
    pub active_batches: i64,
    pub completed_batches: i64,
    pub pending_batches: i64,
    pub failed_batches: i64,
    pub rejected_batches: i64,
    /// 已完成批次的金額合計（整數分，格式化後字串）。
    pub completed_amount_display: String,
    pub recent: Vec<BatchSummaryRow>,
}

/// 讀取 BankPay 儀表板卡片；`bankpay_batches` 表尚未遷移時回 None
/// （遷移 0006 之後必有值）。`mock_mode` 由呼叫端自 AppState 傳入。
pub async fn dashboard_card(pool: &SqlitePool, mock_mode: bool) -> Option<BankPayCard> {
    let counts: (i64, i64, i64, i64, i64, Option<i64>) = sqlx::query_as(
        "SELECT \
           COUNT(*) FILTER (WHERE status = 'PENDING'), \
           COUNT(*) FILTER (WHERE status = 'COMPLETED'), \
           COUNT(*) FILTER (WHERE status = 'SUBMITTED'), \
           COUNT(*) FILTER (WHERE status = 'FAILED'), \
           COUNT(*) FILTER (WHERE status = 'REJECTED'), \
           SUM(amount_minor) FILTER (WHERE status = 'COMPLETED') \
         FROM bankpay_batches",
    )
    .fetch_one(pool)
    .await
    .ok()?;
    let (pending, completed, submitted, failed, rejected, total_amount) = counts;
    let recent: Vec<BatchSummaryRow> = sqlx::query_as::<_, BatchSummaryRow>(
        "SELECT batch_id, esun_ref, amount_minor, currency, status, created_at \
         FROM bankpay_batches ORDER BY created_at DESC LIMIT 5",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    Some(BankPayCard {
        mock_mode,
        active_batches: pending + submitted,
        completed_batches: completed,
        pending_batches: pending,
        failed_batches: failed,
        rejected_batches: rejected,
        completed_amount_display: flaps_core::format_minor(total_amount.unwrap_or(0), 2),
        recent,
    })
}

// ---------------------------------------------------------------------------
// 作業頁（玉山 BankPay 工作室；零 JS 表單 + PRG）
// ---------------------------------------------------------------------------

/// BankPay 作業頁查詢字串（GET /bankpay）。
#[derive(Debug, Default, Deserialize)]
pub struct BankPayPageQuery {
    /// 匯出成功後聚焦的批次（PRG 目標）。
    pub batch: Option<String>,
    /// 匯入成功橫幅：`matched:mismatched`。
    pub imported: Option<String>,
    /// 設定已儲存橫幅。
    pub saved: Option<String>,
    /// 查詢表單（GET）目標參照。
    pub q: Option<String>,
}

/// 匯出表單（金額以小數字串輸入，經 `parse_decimal_to_minor` 精確轉整數分）。
#[derive(Debug, Deserialize)]
pub struct ExportForm {
    pub csrf_token: String,
    pub batch_id: String,
    pub amount: String,
    pub currency: String,
}

/// 匯入表單（檔名相對於白名單目錄）。
#[derive(Debug, Deserialize)]
pub struct ImportForm {
    pub csrf_token: String,
    pub file_name: String,
}

/// 設定表單（checkbox 未勾選時瀏覽器省略欄位 → 以 Option 呈現）。
#[derive(Debug, Deserialize)]
pub struct ConfigForm {
    pub csrf_token: String,
    pub esun_mock_mode: Option<String>,
    pub esun_url: Option<String>,
    pub source_system_id: Option<String>,
    pub customer_id: Option<String>,
    pub payer_account: Option<String>,
}

/// 查詢結果檢視（已格式化，模板零邏輯）。
#[derive(Debug)]
pub struct QueryView {
    pub esun_ref: String,
    pub status: String,
    pub amount_display: String,
    pub currency: String,
}

/// BankPay 作業頁（GET 渲染與表單錯誤回顯共用）。
#[derive(Template)]
#[template(path = "bankpay.html")]
pub struct BankPayPageTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    /// 具 ROLE_MANAGE 者顯示設定區。
    pub can_manage: bool,
    pub mock_mode: bool,
    /// source_system_id 與 customer_id 已設定（生產模式連線要件）。
    pub configured: bool,
    pub card: Option<BankPayCard>,
    /// 最近 20 筆批次（完整清單）。
    pub batches: Vec<BatchSummaryRow>,
    pub config: crate::bankpay::model::EsunConfig,
    /// PRG 聚焦批次（?batch=）。
    pub focus_batch: Option<BatchSummaryRow>,
    pub imported_matched: Option<usize>,
    pub imported_mismatched: Option<usize>,
    pub saved_banner: bool,
    pub query_result: Option<QueryView>,
    /// 各模組狀態（與首頁狀態列同源）。
    pub status_chips: Vec<StatusChip>,
    pub sys_status_overall: String,
    pub error: Option<String>,
}

/// 頁面靜態資料（設定/權限/卡片/批次清單/狀態列）。
struct PageData {
    can_manage: bool,
    mock_mode: bool,
    configured: bool,
    card: Option<BankPayCard>,
    batches: Vec<BatchSummaryRow>,
    config: crate::bankpay::model::EsunConfig,
    chips: Vec<StatusChip>,
    overall: String,
}

async fn page_data(state: &AppState, session: &Session) -> PageData {
    let config = state.bankpay_config.lock().await.clone();
    let can_manage = state
        .permissions
        .has_permission_db(&session.user_id, "ROLE_MANAGE")
        .await;
    let card = dashboard_card(&state.pool, config.esun_mock_mode).await;
    let batches: Vec<BatchSummaryRow> = sqlx::query_as::<_, BatchSummaryRow>(
        "SELECT batch_id, esun_ref, amount_minor, currency, status, created_at \
         FROM bankpay_batches ORDER BY created_at DESC LIMIT 20",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let (chips, overall) = crate::api::status_chips(state).await;
    PageData {
        can_manage,
        mock_mode: config.esun_mock_mode,
        configured: config.source_system_id.is_some() && config.customer_id.is_some(),
        card,
        batches,
        chips,
        overall,
        config,
    }
}

async fn focus_batch_row(pool: &SqlitePool, batch_id: &str) -> Option<BatchSummaryRow> {
    sqlx::query_as::<_, BatchSummaryRow>(
        "SELECT batch_id, esun_ref, amount_minor, currency, status, created_at \
         FROM bankpay_batches WHERE batch_id = ?",
    )
    .bind(batch_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_default()
}

fn render_page_error(template: BankPayPageTemplate, error: String, status: StatusCode) -> Response {
    (status, render_or_500_with(template, Some(error))).into_response()
}

fn render_or_500_with(mut template: BankPayPageTemplate, error: Option<String>) -> String {
    template.error = error;
    template
        .render()
        .unwrap_or_else(|e| format!("模板渲染失敗：{e}"))
}

/// GET /bankpay — 銀行支付作業頁（含查詢/PRG 橫幅）。
pub async fn bankpay_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    axum::extract::Query(query): axum::extract::Query<BankPayPageQuery>,
) -> Response {
    let data = page_data(&state, &session).await;

    // ?batch= → 從本地批次帳載入聚焦批次（PRG 後的結果呈現）。
    let focus_batch = match query.batch.as_deref() {
        Some(id) => focus_batch_row(&state.pool, id).await,
        None => None,
    };

    // ?imported=m:n → 匯入成功橫幅。
    let (imported_matched, imported_mismatched) = match query.imported.as_deref() {
        Some(raw) => raw
            .split_once(':')
            .and_then(|(m, n)| Some((m.parse::<usize>().ok()?, n.parse::<usize>().ok()?)))
            .map_or((None, None), |(m, n)| (Some(m), Some(n))),
        None => (None, None),
    };

    // ?q= → 執行查詢（LEDGER_READ；GET 讀取語義）。
    let mut query_result: Option<QueryView> = None;
    let mut error: Option<String> = None;
    if let Some(esun_ref) = query.q.as_deref().filter(|s| !s.trim().is_empty()) {
        let req = QueryRequest {
            esun_ref: esun_ref.trim().to_string(),
        };
        match query_inner(&state, &session, &req).await {
            Ok(resp) => {
                let amount_display = resp
                    .amount_minor
                    .map(|m| flaps_core::format_minor(m, 2))
                    .unwrap_or_else(|| "—".to_string());
                query_result = Some(QueryView {
                    esun_ref: req.esun_ref.clone(),
                    status: resp.status,
                    amount_display,
                    currency: resp.currency.unwrap_or_else(|| "—".to_string()),
                });
            }
            Err(e) => error = Some(e.1),
        }
    }

    let template = BankPayPageTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        can_manage: data.can_manage,
        mock_mode: data.mock_mode,
        configured: data.configured,
        card: data.card,
        batches: data.batches,
        config: data.config,
        focus_batch,
        imported_matched,
        imported_mismatched,
        saved_banner: query.saved.as_deref() == Some("1"),
        query_result,
        status_chips: data.chips,
        sys_status_overall: data.overall,
        error,
    };
    match template.render() {
        Ok(html) => axum::response::Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("模板渲染失敗：{e}"),
        )
            .into_response(),
    }
}

/// POST /bankpay/export — 表單匯出（成功 PRG；失敗回顯錯誤）。
pub async fn bankpay_export_form(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(form): Form<ExportForm>,
) -> Response {
    // 金額：小數字串 → 精確整數分（INV-01；禁浮點）。
    let amount_minor = match flaps_core::parse_decimal_to_minor(form.amount.trim(), 2) {
        Ok(v) => v,
        Err(e) => {
            let data = page_data(&state, &session).await;
            return render_page_error(
                BankPayPageTemplate {
                    acting_user: session.user_id.clone(),
                    csrf_token: session.csrf_token.clone(),
                    can_manage: data.can_manage,
                    mock_mode: data.mock_mode,
                    configured: data.configured,
                    card: data.card,
                    batches: data.batches,
                    config: data.config,
                    focus_batch: None,
                    imported_matched: None,
                    imported_mismatched: None,
                    saved_banner: false,
                    query_result: None,
                    status_chips: data.chips,
                    sys_status_overall: data.overall,
                    error: None,
                },
                format!("金額驗證失敗：{e}。問題：金額「{}」無法轉為整數分；處理方式：以小數輸入（如 500.00，即 50000 分）後重試。", form.amount.trim()),
                StatusCode::BAD_REQUEST,
            );
        }
    };
    let req = ExportRequest {
        batch_id: form.batch_id.trim().to_string(),
        amount_minor,
        currency: form.currency.trim().to_string(),
    };
    match export_inner(&state, &session, &form.csrf_token, &req).await {
        // 成功 → PRG（同批次重放安全：冪等回既有狀態）。
        Ok(_) => Redirect::to(&format!("/bankpay?batch={}", req.batch_id)).into_response(),
        Err(e) => {
            let data = page_data(&state, &session).await;
            render_page_error(
                BankPayPageTemplate {
                    acting_user: session.user_id.clone(),
                    csrf_token: session.csrf_token.clone(),
                    can_manage: data.can_manage,
                    mock_mode: data.mock_mode,
                    configured: data.configured,
                    card: data.card,
                    batches: data.batches,
                    config: data.config,
                    focus_batch: None,
                    imported_matched: None,
                    imported_mismatched: None,
                    saved_banner: false,
                    query_result: None,
                    status_chips: data.chips,
                    sys_status_overall: data.overall,
                    error: None,
                },
                e.1,
                e.0,
            )
        }
    }
}

/// POST /bankpay/import — 表單匯入（成功 PRG 帶筆數橫幅；失敗回顯錯誤）。
pub async fn bankpay_import_form(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(form): Form<ImportForm>,
) -> Response {
    let req = ImportRequest {
        file_path: form.file_name.trim().to_string(),
    };
    match import_inner(&state, &session, &form.csrf_token, &req).await {
        Ok(resp) => Redirect::to(&format!(
            "/bankpay?imported={}:{}#batches",
            resp.matched, resp.mismatched
        ))
        .into_response(),
        Err(e) => {
            let data = page_data(&state, &session).await;
            render_page_error(
                BankPayPageTemplate {
                    acting_user: session.user_id.clone(),
                    csrf_token: session.csrf_token.clone(),
                    can_manage: data.can_manage,
                    mock_mode: data.mock_mode,
                    configured: data.configured,
                    card: data.card,
                    batches: data.batches,
                    config: data.config,
                    focus_batch: None,
                    imported_matched: None,
                    imported_mismatched: None,
                    saved_banner: false,
                    query_result: None,
                    status_chips: data.chips,
                    sys_status_overall: data.overall,
                    error: None,
                },
                e.1,
                e.0,
            )
        }
    }
}

/// POST /bankpay/config — 表單設定（成功 PRG 帶已儲存橫幅；ROLE_MANAGE）。
pub async fn bankpay_config_form(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(form): Form<ConfigForm>,
) -> Response {
    // 部分更新語義：空欄位 = 保留現值；checkbox 未勾選 = 關閉。
    let req = ConfigUpdateRequest {
        esun_mock_mode: Some(form.esun_mock_mode.as_deref() != Some("on")),
        esun_url: form
            .esun_url
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        source_system_id: form
            .source_system_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        customer_id: form
            .customer_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        payer_account: form
            .payer_account
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
    };
    match config_update_inner(&state, &session, &form.csrf_token, req).await {
        Ok(_) => Redirect::to("/bankpay?saved=1#config").into_response(),
        Err(e) => {
            let data = page_data(&state, &session).await;
            render_page_error(
                BankPayPageTemplate {
                    acting_user: session.user_id.clone(),
                    csrf_token: session.csrf_token.clone(),
                    can_manage: data.can_manage,
                    mock_mode: data.mock_mode,
                    configured: data.configured,
                    card: data.card,
                    batches: data.batches,
                    config: data.config,
                    focus_batch: None,
                    imported_matched: None,
                    imported_mismatched: None,
                    saved_banner: false,
                    query_result: None,
                    status_chips: data.chips,
                    sys_status_overall: data.overall,
                    error: None,
                },
                e.1,
                e.0,
            )
        }
    }
}
