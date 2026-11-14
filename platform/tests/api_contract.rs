#![forbid(unsafe_code)]
//! G4.11 (D4-01..D4-04) — API contract: JSON error envelope, `/api/search`
//! rate limit (429 + Retry-After), `Idempotency-Key` replay, `X-Request-Id`.

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode, header};
use platform::api::{AppState, router};
use platform::auth;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const TEST_ADMIN_PASSWORD: &str = "Admin-Test-Passw0rd!";

async fn in_memory_app() -> (Router, SqlitePool) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("pool");
    let state = AppState::new(pool.clone()).await.expect("state");
    (router(state), pool)
}

async fn seed_admin(pool: &SqlitePool) {
    let hash = auth::hash_password(TEST_ADMIN_PASSWORD).expect("hash");
    sqlx::query(
        "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
         VALUES ('user-admin', 'admin', ?, 1)",
    )
    .bind(&hash)
    .execute(pool)
    .await
    .expect("seed admin");
}

async fn login(app: &Router) -> String {
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "username=admin&password={TEST_ADMIN_PASSWORD}"
        )))
        .expect("login request");
    let response = app.clone().oneshot(request).await.expect("login response");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|v| v.to_str().ok())
        .map(|v| v.split(';').next().unwrap_or("").to_string())
        .expect("session cookie")
}

struct Resp {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

async fn call(app: &Router, request: Request<Body>) -> Resp {
    let response = app.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), 4_194_304)
        .await
        .expect("body");
    Resp {
        status,
        headers,
        body: String::from_utf8_lossy(&bytes).to_string(),
    }
}

fn get(app: &Router, uri: &str, cookie: &str) -> Request<Body> {
    let _ = app;
    Request::builder()
        .uri(uri)
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .expect("GET request")
}

fn post_index(
    app: &Router,
    cookie: &str,
    key: Option<&str>,
    doc_id: &str,
    content: &str,
) -> Request<Body> {
    let _ = app;
    let body = serde_json::json!({"doc_id": doc_id, "content": content}).to_string();
    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/search")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, cookie);
    if let Some(key) = key {
        builder = builder.header("idempotency-key", key);
    }
    builder.body(Body::from(body)).expect("POST request")
}

fn delete(app: &Router, cookie: &str, uri: &str) -> Request<Body> {
    let _ = app;
    Request::builder()
        .method("DELETE")
        .uri(uri)
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .expect("DELETE request")
}

#[tokio::test]
async fn unauthenticated_api_returns_json_envelope() {
    let (app, _pool) = in_memory_app().await;
    let response = call(&app, get(&app, "/api/search?q=x", "")).await;
    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    let json: serde_json::Value = serde_json::from_str(&response.body).expect("json envelope");
    assert_eq!(json["error"]["code"], "UNAUTHENTICATED");
    assert!(json["error"]["request_id"].as_str().is_some());
    assert!(
        json["error"]["hint"]
            .as_str()
            .is_some_and(|h| h.contains("問題"))
    );
}

#[tokio::test]
async fn missing_document_returns_not_found_envelope() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;
    let response = call(&app, get(&app, "/api/search/nope", &cookie)).await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    let json: serde_json::Value = serde_json::from_str(&response.body).expect("json envelope");
    assert_eq!(json["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn search_is_rate_limited_after_30_per_minute() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;
    for _ in 0..30 {
        let response = call(&app, get(&app, "/api/search?q=alpha", &cookie)).await;
        assert_eq!(response.status, StatusCode::OK);
    }
    let response = call(&app, get(&app, "/api/search?q=alpha", &cookie)).await;
    assert_eq!(response.status, StatusCode::TOO_MANY_REQUESTS);
    assert!(
        response.headers.contains_key(header::RETRY_AFTER),
        "429 must carry Retry-After"
    );
    let json: serde_json::Value = serde_json::from_str(&response.body).expect("json envelope");
    assert_eq!(json["error"]["code"], "RATE_LIMITED");
}

#[tokio::test]
async fn idempotency_key_replays_without_re_execution() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;
    const KEY: &str = "idem-g4-11";

    let first = call(&app, post_index(&app, &cookie, Some(KEY), "d1", "alpha")).await;
    assert_eq!(first.status, StatusCode::OK);
    assert!(
        first.headers.get("idempotency-replay").is_none(),
        "first call executes"
    );

    // Remove the side effect the first call produced.
    let removed = call(&app, delete(&app, &cookie, "/api/search/d1")).await;
    assert_eq!(removed.status, StatusCode::OK);

    // Replay: stored response, no re-execution → doc must stay absent.
    let replay = call(&app, post_index(&app, &cookie, Some(KEY), "d1", "alpha")).await;
    assert_eq!(replay.status, StatusCode::OK);
    assert_eq!(
        replay
            .headers
            .get("idempotency-replay")
            .and_then(|v| v.to_str().ok()),
        Some("true")
    );
    let after = call(&app, get(&app, "/api/search/d1", &cookie)).await;
    assert_eq!(
        after.status,
        StatusCode::NOT_FOUND,
        "replay must not re-execute the write"
    );
}

#[tokio::test]
async fn different_idempotency_keys_execute_independently() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;

    let a = call(
        &app,
        post_index(&app, &cookie, Some("key-a"), "d1", "alpha"),
    )
    .await;
    let b = call(&app, post_index(&app, &cookie, Some("key-b"), "d2", "beta")).await;
    assert_eq!(a.status, StatusCode::OK);
    assert_eq!(b.status, StatusCode::OK);
    assert!(a.headers.get("idempotency-replay").is_none());
    assert!(b.headers.get("idempotency-replay").is_none());
}

#[tokio::test]
async fn request_id_is_echoed_on_api_responses() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;
    let request = Request::builder()
        .uri("/api/search?q=alpha")
        .header(header::COOKIE, &cookie)
        .header("x-request-id", "trace-123")
        .body(Body::empty())
        .expect("request");
    let response = call(&app, request).await;
    assert_eq!(
        response
            .headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok()),
        Some("trace-123")
    );
}
