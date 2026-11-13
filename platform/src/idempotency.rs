#![forbid(unsafe_code)]
//! G4.11 (D4-02) — `Idempotency-Key` replay for POST `/api/*`.
//!
//! First request with a key executes normally; its response (status + body) is
//! stored keyed by (key, user_id). Replays return the stored response verbatim
//! without re-executing. The key is scoped per user, so different users can use
//! the same key string safely.

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;

use crate::api::AppState;
use crate::auth::Session;

pub const IDEMPOTENCY_HEADER: &str = "idempotency-key";
const MAX_BODY: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StoredResponse {
    pub status: i64,
    pub body: String,
}

pub async fn lookup(pool: &sqlx::SqlitePool, key: &str, user_id: &str) -> Option<StoredResponse> {
    sqlx::query_as("SELECT status, body FROM idempotency_records WHERE key = ? AND user_id = ?")
        .bind(key)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

#[allow(clippy::too_many_arguments)]
pub async fn store(
    pool: &sqlx::SqlitePool,
    key: &str,
    user_id: &str,
    method: &str,
    path: &str,
    status: u16,
    body: &str,
) {
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO idempotency_records \
         (key, user_id, method, path, status, body) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(key)
    .bind(user_id)
    .bind(method)
    .bind(path)
    .bind(i64::from(status))
    .bind(body)
    .execute(pool)
    .await;
}

/// POST-only replay middleware. Runs after `require_session_api` (session
/// extension available) and before the handler.
pub async fn middleware(State(state): State<AppState>, request: Request, next: Next) -> Response {
    if request.method() != Method::POST {
        return next.run(request).await;
    }
    let Some(key) = request
        .headers()
        .get(IDEMPOTENCY_HEADER)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
    else {
        return next.run(request).await;
    };
    let Some(session) = request.extensions().get::<Session>().cloned() else {
        return next.run(request).await;
    };
    let path = request.uri().path().to_string();
    if let Some(stored) = lookup(&state.pool, &key, &session.user_id).await {
        let status = StatusCode::from_u16(stored.status as u16).unwrap_or(StatusCode::OK);
        let mut response = Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "application/json")
            .header("idempotency-replay", "true")
            .body(Body::from(stored.body))
            .expect("replay response");
        if let Ok(value) = HeaderValue::from_str(&key) {
            response.headers_mut().insert("idempotency-key", value);
        }
        return response;
    }

    let method = request.method().to_string();
    let response = next.run(request).await;
    let (parts, body) = response.into_parts();
    let bytes = match axum::body::to_bytes(body, MAX_BODY).await {
        Ok(bytes) => bytes,
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };
    let body_str = String::from_utf8_lossy(&bytes).to_string();
    store(
        &state.pool,
        &key,
        &session.user_id,
        &method,
        &path,
        parts.status.as_u16(),
        &body_str,
    )
    .await;
    Response::from_parts(parts, Body::from(bytes))
}
