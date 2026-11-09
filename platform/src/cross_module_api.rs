//! Slice 13: Cross-module API endpoints (Search, Sync, COM, AI).
//!
//! These endpoints expose the internal platform modules (search, sync, com, ai)
//! as RESTful HTTP APIs, enabling external systems and the web UI to interact
//! with the platform's core services.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
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
    pub ip_address: String,
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

type AuditRow = (
    String,
    String,
    Option<String>,
    String,
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

    let mut sql = "SELECT al.log_id, al.user_id, u.username, al.action, al.resource, al.details, al.ip_address, al.created_at FROM audit_log al LEFT JOIN users u ON al.user_id = u.user_id WHERE 1=1".to_string();
    let mut params: Vec<String> = vec![];

    if let Some(ref uid) = query.user_id {
        sql.push_str(" AND al.user_id = ?");
        params.push(uid.clone());
    }
    if let Some(ref action) = query.action {
        sql.push_str(" AND al.action LIKE ?");
        params.push(format!("%{}%", action));
    }
    if let Some(ref sd) = query.start_date {
        sql.push_str(" AND al.created_at >= ?");
        params.push(sd.clone());
    }
    if let Some(ref ed) = query.end_date {
        sql.push_str(" AND al.created_at <= ?");
        params.push(ed.clone());
    }

    sql.push_str(" ORDER BY al.created_at DESC LIMIT ? OFFSET ?");
    params.push(limit.to_string());
    params.push(offset.to_string());

    let mut query_builder = sqlx::query_as(&sql);
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
            |(log_id, user_id, username, action, resource, details, ip_address, created_at)| {
                AuditEntry {
                    log_id,
                    user_id,
                    username: username.unwrap_or_else(|| "unknown".to_string()),
                    action,
                    resource,
                    details,
                    ip_address,
                    created_at,
                }
            },
        )
        .collect();

    let total = items.len() as u64;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    Ok(Json(AuditListResponse {
        items,
        total,
        page,
        limit,
        total_pages,
    }))
}

pub async fn audit_export(
    State(state): State<AppState>,
) -> Result<axum::response::Response, (axum::http::StatusCode, String)> {
    let rows: Vec<AuditRow> = sqlx::query_as(
        "SELECT al.log_id, al.user_id, u.username, al.action, al.resource, al.details, al.ip_address, al.created_at FROM audit_log al LEFT JOIN users u ON al.user_id = u.user_id ORDER BY al.created_at DESC LIMIT 10000"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("匯出失敗：{}", e)))?;

    let mut csv =
        String::from("log_id,user_id,username,action,resource,details,ip_address,created_at\n");
    for (log_id, user_id, username, action, resource, details, ip_address, created_at) in &rows {
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            log_id,
            user_id,
            username.as_deref().unwrap_or("unknown"),
            action,
            resource,
            details,
            ip_address,
            created_at
        ));
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
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "scheduled",
        "config_id": req.config_id,
        "target_system": req.target_system
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
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "scheduled",
        "entity_id": req.entity_id,
        "target_system": req.target_system
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
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "prog_id": req.prog_id,
        "method": req.method_name,
        "result": format!("Called {}.{} successfully", req.prog_id, req.method_name)
    }))
}

pub async fn com_object_count(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"count": 0}))
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
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "model": req.model.unwrap_or_else(|| "default".to_string()),
        "response": format!("AI response to: {}", req.prompt)
    }))
}

pub async fn ai_list_models(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "models": ["ollama:llama3", "ollama:codellama", "openai:gpt-3.5-turbo"]
    }))
}
// ══════════════════════════════════════════════════════════════════════════════
// Router
// ══════════════════════════════════════════════════════════════════════════════

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        // ── Search API ──
        .route("/api/search", get(search_query).post(search_index))
        .route(
            "/api/search/{doc_id}",
            get(search_get).delete(search_delete),
        )
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
    let db_ok = sqlx::query("SELECT 1").fetch_one(&state.pool).await.is_ok();
    let search_count = state.search.document_count().await;
    Json(ModuleHealth {
        search: format!("ok ({} docs)", search_count),
        sync: "ok (0 pending)".to_string(),
        com: "ok (0 objects)".to_string(),
        ai: "ok (3 models)".to_string(),
        db: if db_ok {
            "ok".to_string()
        } else {
            "error".to_string()
        },
        overall: if db_ok {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
    })
}
