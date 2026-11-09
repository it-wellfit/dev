//! Slice 13: Cross-module API endpoints (Search, Sync, COM, AI).
//!
//! These endpoints expose the internal platform modules (search, sync, com, ai)
//! as RESTful HTTP APIs, enabling external systems and the web UI to interact
//! with the platform's core services.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use crate::api::AppState;

// ═════════════════════════════════════════════════════════════════════════════
// Audit Trail API (Governance & Audit Studio)
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct AuditQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub user_id: Option<String>,
    pub action: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Serialize)]
pub struct AuditEntry {
    pub log_id: String,
    pub user_id: String,
    pub username: String,
    pub action: String,
    pub resource: String,
    pub details: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct AuditListResponse {
    pub items: Vec<AuditEntry>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

pub type AuditRow = (
    String,
    String,
    Option<String>,
    String,
    String,
    String,
    String,
);

pub async fn audit_list(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<AuditListResponse>, (axum::http::StatusCode, String)> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 500);
    let offset = (page - 1) * limit;

    let mut sql = "SELECT al.log_id, al.performed_by, u.username, al.event_type, COALESCE(al.transaction_id, ''), COALESCE(al.event_data, ''), al.performed_at FROM audit_log al LEFT JOIN users u ON al.performed_by = u.user_id WHERE 1=1".to_string();
    let mut params: Vec<String> = vec![];

    if let Some(ref uid) = query.user_id {
        sql.push_str(" AND al.performed_by = ?");
        params.push(uid.clone());
    }
    if let Some(ref action) = query.action {
        sql.push_str(" AND al.event_type LIKE ?");
        params.push(format!("%{}%", action));
    }
    if let Some(ref sd) = query.start_date {
        sql.push_str(" AND al.performed_at >= ?");
        params.push(sd.clone());
    }
    if let Some(ref ed) = query.end_date {
        sql.push_str(" AND al.performed_at <= ?");
        params.push(ed.clone());
    }

    sql.push_str(" ORDER BY al.performed_at DESC LIMIT ? OFFSET ?");
    params.push(limit.to_string());
    params.push(offset.to_string());

    // total 必須以「相同 WHERE 條件」的 COUNT(*) 取得（修正：原實作以
    // 分頁後的 items.len() 充當 total，導致 total/total_pages 恆錯）。
    // 分頁除法以整數天花板除法完成，不引入浮點（INV-01 精確計算紀律）。
    let mut count_sql =
        "SELECT COUNT(*) FROM audit_log al LEFT JOIN users u ON al.performed_by = u.user_id WHERE 1=1"
            .to_string();
    let mut count_params: Vec<String> = vec![];
    if let Some(ref uid) = query.user_id {
        count_sql.push_str(" AND al.performed_by = ?");
        count_params.push(uid.clone());
    }
    if let Some(ref action) = query.action {
        count_sql.push_str(" AND al.event_type LIKE ?");
        count_params.push(format!("%{}%", action));
    }
    if let Some(ref sd) = query.start_date {
        count_sql.push_str(" AND al.performed_at >= ?");
        count_params.push(sd.clone());
    }
    if let Some(ref ed) = query.end_date {
        count_sql.push_str(" AND al.performed_at <= ?");
        count_params.push(ed.clone());
    }
    let mut count_builder = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(count_sql.as_str()));
    for p in &count_params {
        count_builder = count_builder.bind(p);
    }
    let total: u64 = count_builder
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("計數失敗：{}", e),
            )
        })?
        .max(0) as u64;

    let mut query_builder = sqlx::query_as(sqlx::AssertSqlSafe(sql.as_str()));
    for p in &params {
        query_builder = query_builder.bind(p);
    }

    let rows: Vec<AuditRow> = match query_builder.fetch_all(&state.pool).await {
        Ok(rows) => rows,
        Err(e) => {
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("查詢失敗：{}", e),
            ))
        }
    };

    let items: Vec<AuditEntry> = rows
        .into_iter()
        .map(
            |(
                log_id,
                performed_by,
                username,
                event_type,
                transaction_id,
                event_data,
                performed_at,
            )| {
                AuditEntry {
                    log_id,
                    user_id: performed_by,
                    username: username.unwrap_or_else(|| "unknown".to_string()),
                    action: event_type,
                    resource: transaction_id,
                    details: event_data,
                    created_at: performed_at,
                }
            },
        )
        .collect();

    // 整數天花板除法：total_pages = ceil(total / limit)，零浮點。
    let total_pages = total.div_ceil(limit as u64) as u32;

    Ok(Json(AuditListResponse {
        items,
        total,
        page,
        limit,
        total_pages,
    }))
}

/// CSV 欄位跳脫＋試算表公式注入防護（CWE-1236）：
/// 內嵌 `"` 逐字加倍；以 `=`、`+`、`-`、`@`、Tab 開頭的欄位前置單引號，
/// 防止匯入 Excel/Sheets 時被當成公式執行。
fn csv_escape(field: &str) -> String {
    let needs_quote = field.contains('"') || field.contains(',') || field.contains('\n');
    let guarded = if matches!(
        field.chars().next(),
        Some('=') | Some('+') | Some('-') | Some('@') | Some('\t')
    ) {
        format!("'{field}")
    } else {
        field.to_string()
    };
    if needs_quote || guarded != field {
        format!("\"{}\"", guarded.replace('"', "\"\""))
    } else {
        format!("\"{guarded}\"")
    }
}

pub async fn audit_export(
    State(state): State<AppState>,
) -> Result<axum::response::Response, (axum::http::StatusCode, String)> {
    let rows: Vec<AuditRow> = sqlx::query_as(
        "SELECT al.log_id, al.performed_by, u.username, al.event_type, COALESCE(al.transaction_id, ''), COALESCE(al.event_data, ''), al.performed_at FROM audit_log al LEFT JOIN users u ON al.performed_by = u.user_id ORDER BY al.performed_at DESC LIMIT 10000"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("匯出失敗：{}", e)))?;

    let mut csv = String::from("log_id,user_id,username,action,resource,details,created_at\n");
    for (log_id, performed_by, username, event_type, transaction_id, event_data, performed_at) in
        &rows
    {
        let fields = [
            log_id.as_str(),
            performed_by.as_str(),
            username.as_deref().unwrap_or("unknown"),
            event_type.as_str(),
            transaction_id.as_str(),
            event_data.as_str(),
            performed_at.as_str(),
        ];
        let row: Vec<String> = fields.iter().map(|f| csv_escape(f)).collect();
        csv.push_str(&row.join(","));
        csv.push('\n');
    }

    axum::response::Response::builder()
        .header(axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(
            axum::http::header::CONTENT_DISPOSITION,
            r#"attachment; filename="audit_log.csv""#,
        )
        .body(axum::body::Body::from(csv))
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("建立回應失敗：{}", e),
            )
        })
}

// ══════════════════════════════════════════════════════════════════════════════
// Search API
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub doc_id: String,
    pub content: String,
    pub score: f64,
}

pub async fn search_query(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<Vec<SearchResult>> {
    let results = state
        .search
        .search(&query.q)
        .await
        .into_iter()
        .map(|doc| SearchResult {
            doc_id: doc.doc_id,
            content: doc.content,
            score: 1.0,
        })
        .collect::<Vec<_>>();
    Json(results)
}

#[derive(Deserialize)]
pub struct SearchIndexRequest {
    pub doc_id: String,
    pub content: String,
}

pub async fn search_index(
    State(state): State<AppState>,
    Json(req): Json<SearchIndexRequest>,
) -> Json<serde_json::Value> {
    state
        .search
        .add_document(req.doc_id, req.content, HashMap::new())
        .await;
    Json(serde_json::json!({"status": "indexed"}))
}

pub async fn search_get(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Json<SearchResult>, (StatusCode, String)> {
    match state.search.get_document(&doc_id).await {
        Some(doc) => Ok(Json(SearchResult {
            doc_id: doc.doc_id,
            content: doc.content,
            score: 1.0,
        })),
        None => Err((StatusCode::NOT_FOUND, "document not found".to_string())),
    }
}

pub async fn search_delete(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Json<serde_json::Value> {
    state.search.delete_document(&doc_id).await;
    Json(serde_json::json!({"status": "deleted"}))
}
// ══════════════════════════════════════════════════════════════════════════════
// Sync API
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct SyncConfigRequest {
    pub config_id: String,
    pub target_system: String,
}

#[derive(Serialize)]
pub struct SyncStatusResponse {
    pub pending: usize,
    pub completed: usize,
    pub failed: usize,
}

pub async fn sync_status(State(_state): State<AppState>) -> Json<SyncStatusResponse> {
    Json(SyncStatusResponse {
        pending: 0,
        completed: 0,
        failed: 0,
    })
}

pub async fn sync_config(
    State(_state): State<AppState>,
    Json(req): Json<SyncConfigRequest>,
) -> axum::response::Response {
    not_implemented(json!({
        "received": { "config_id": req.config_id, "target_system": req.target_system },
        "reason": "sync::SyncEngine 未於 Web 層組裝；本端點不虛構「已排程」回應（治理 P7）。"
    }))
}

#[derive(Deserialize)]
pub struct SyncPermissionRequest {
    pub entity_id: String,
    pub target_system: String,
}

pub async fn sync_permission(
    State(_state): State<AppState>,
    Json(req): Json<SyncPermissionRequest>,
) -> axum::response::Response {
    not_implemented(json!({
        "received": { "entity_id": req.entity_id, "target_system": req.target_system },
        "reason": "sync::SyncEngine 未於 Web 層組裝；本端點不虛構「已排程」回應（治理 P7）。"
    }))
}
// ══════════════════════════════════════════════════════════════════════════════
// COM API
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct ComInvokeRequest {
    pub prog_id: String,
    pub method_name: String,
    pub args: Option<Vec<String>>,
}

pub async fn com_invoke(
    State(_state): State<AppState>,
    Json(req): Json<ComInvokeRequest>,
) -> axum::response::Response {
    not_implemented(json!({
        "received": { "prog_id": req.prog_id, "method": req.method_name },
        "reason": "com::ComBridge::invoke 為佔位實作，未連接真實 COM；本端點不虛構「呼叫成功」回應（治理 P7）。真實通道請使用 flaps_bridge（real-com feature）。"
    }))
}

pub async fn com_object_count(State(_state): State<AppState>) -> Json<serde_json::Value> {
    // 誠實值：ComBridge 物件池未於 Web 層組裝，可列舉物件數為 0。
    Json(serde_json::json!({ "count": 0, "mode": "placeholder" }))
}
// ══════════════════════════════════════════════════════════════════════════════
// AI API
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct AiGenerateRequest {
    pub prompt: String,
    pub model: Option<String>,
}

pub async fn ai_generate(
    State(_state): State<AppState>,
    Json(req): Json<AiGenerateRequest>,
) -> axum::response::Response {
    not_implemented(json!({
        "received": { "prompt": req.prompt, "model": req.model },
        "reason": "AI 推論端點未設定（ai 模組客戶端待接 ollama/chutes 設定）；本端點不虛構 AI 回應（治理 P7）。"
    }))
}

pub async fn ai_list_models(State(_state): State<AppState>) -> Json<serde_json::Value> {
    // 誠實值：未設定任何推論端點，可用模型數為 0（絕不虛構模型清單）。
    Json(serde_json::json!({ "models": [], "mode": "not_configured" }))
}
// ══════════════════════════════════════════════════════════════════════════════
// Router
// ══════════════════════════════════════════════════════════════════════════════

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        // ── Search API ──
        .route("/api/search", get(search_query).post(search_index))
        .route("/api/search/:doc_id", get(search_get).delete(search_delete))
        // ── Sync API ──
        .route("/api/sync/status", get(sync_status))
        .route("/api/sync/config", post(sync_config))
        .route("/api/sync/permission", post(sync_permission))
        // ── COM API ──
        .route("/api/com/invoke", post(com_invoke))
        .route("/api/com/objects", get(com_object_count))
        // ── AI API ──
        .route("/api/ai/generate", post(ai_generate))
        .route("/api/ai/models", get(ai_list_models))
        // ── Module Health ──
        .route("/api/modules/health", get(modules_health))
        // ── Audit Trail API ──
        .route("/api/audit", get(audit_list))
        .route("/api/audit/export", get(audit_export))
        .with_state(state)
}

// ── Module Health Check ──
#[derive(Serialize)]
pub struct ModuleHealth {
    pub search: String,
    pub sync: String,
    pub com: String,
    pub ai: String,
    pub db: String,
    pub overall: String,
}

pub async fn modules_health(State(state): State<AppState>) -> Json<ModuleHealth> {
    // 與 /api/status 同源即時探測（單一事實來源，絕不硬編碼「ok (3 models)」）。
    let modules = crate::status::probe_modules(&state).await;
    let describe = |key: &str| -> String {
        modules.get(key).map_or_else(
            || "unknown".to_string(),
            |m| match &m.message {
                Some(msg) => format!("{} — {}", m.status, msg),
                None => format!("{}", m.status),
            },
        )
    };
    Json(ModuleHealth {
        search: describe("search"),
        sync: describe("sync"),
        com: describe("com"),
        ai: describe("ai"),
        db: modules
            .get("database")
            .map(|m| m.status.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        overall: crate::status::overall_status(&modules),
    })
}

/// 501 回應（誠實標示未串接的端點；含問題說明與處理建議）。
fn not_implemented(detail: serde_json::Value) -> axum::response::Response {
    (
        axum::http::StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "NOT_IMPLEMENTED",
            "message": "此端點尚未串接真實後端；平台選擇如實回報，而非回傳模擬成功（治理 P7 反模擬）。",
            "detail": detail,
        })),
    )
        .into_response()
}
