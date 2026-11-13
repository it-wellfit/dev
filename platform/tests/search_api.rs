#![forbid(unsafe_code)]
//! G4.1 (D2-02) — `/api/search` end-to-end integration tests (PHASE-R S3).
//!
//! Audit gap closed: the route was wired (`cross_module_api.rs`) but never
//! exercised end-to-end. Four cases: ranking order, CJK query, unknown term
//! -> `[]`, and honest `via` provenance (ranked vs direct fetch).

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use platform::api::{AppState, router};
use platform::auth;
use serde_json::Value;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const TEST_ADMIN_PASSWORD: &str = "Admin-Test-Passw0rd!";

async fn in_memory_app() -> (Router, SqlitePool) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory SQLite pool");
    let state = AppState::new(pool.clone())
        .await
        .expect("app state with migrations applied");
    (router(state), pool)
}

async fn seed_admin(pool: &SqlitePool) {
    let hash = auth::hash_password(TEST_ADMIN_PASSWORD).expect("hash admin password");
    sqlx::query(
        "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
         VALUES ('user-admin', 'admin', ?, 1)",
    )
    .bind(&hash)
    .execute(pool)
    .await
    .expect("seed admin user");
}

async fn login(app: &Router) -> String {
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "username=admin&password={TEST_ADMIN_PASSWORD}"
        )))
        .expect("build login request");
    let response = app.clone().oneshot(request).await.expect("login response");
    assert_eq!(response.status(), StatusCode::SEE_OTHER, "login PRG");
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|v| v.to_str().ok())
        .map(|v| v.split(';').next().unwrap_or("").to_string())
        .expect("session cookie")
}

/// Percent-encode a query for the URL (UTF-8 bytes; no external crate).
fn enc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

async fn index(app: &Router, cookie: &str, doc_id: &str, content: &str) {
    let body = serde_json::json!({ "doc_id": doc_id, "content": content }).to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/api/search")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, cookie)
        .body(Body::from(body))
        .expect("build index request");
    let response = app.clone().oneshot(request).await.expect("index response");
    assert_eq!(response.status(), StatusCode::OK, "index must succeed");
}

async fn search(app: &Router, cookie: &str, q: &str) -> Vec<Value> {
    let request = Request::builder()
        .uri(format!("/api/search?q={}", enc(q)))
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .expect("build search request");
    let response = app.clone().oneshot(request).await.expect("search response");
    assert_eq!(response.status(), StatusCode::OK, "search must succeed");
    let bytes = axum::body::to_bytes(response.into_body(), 1_048_576)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("JSON array")
}

async fn get_json(app: &Router, cookie: &str, uri: &str) -> Value {
    let request = Request::builder()
        .uri(uri)
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .expect("build GET request");
    let response = app.clone().oneshot(request).await.expect("GET response");
    assert_eq!(response.status(), StatusCode::OK, "GET {uri} must succeed");
    let bytes = axum::body::to_bytes(response.into_body(), 1_048_576)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("JSON value")
}

#[tokio::test]
async fn search_api_ranks_documents_by_query() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;

    index(&app, &cookie, "d1", "alpha beta alpha alpha").await;
    index(&app, &cookie, "d2", "beta beta").await;
    index(&app, &cookie, "d3", "gamma delta").await;

    let hits = search(&app, &cookie, "alpha").await;
    assert!(!hits.is_empty(), "query must match indexed documents");
    assert_eq!(
        hits[0]["doc_id"], "d1",
        "highest term frequency ranks first"
    );

    let scores: Vec<f64> = hits
        .iter()
        .map(|h| h["score"].as_f64().expect("score is a number"))
        .collect();
    for pair in scores.windows(2) {
        assert!(
            pair[0] >= pair[1],
            "scores must be non-increasing: {scores:?}"
        );
    }
}

#[tokio::test]
async fn search_api_cjk_query_matches_chinese_content() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;

    index(&app, &cookie, "c1", "財務報表分析與月結流程").await;
    index(&app, &cookie, "c2", "庫存管理與進貨單").await;

    let hits = search(&app, &cookie, "財務").await;
    assert_eq!(hits.len(), 1, "CJK query must match exactly one document");
    assert_eq!(hits[0]["doc_id"], "c1", "matched document identity");
}

#[tokio::test]
async fn search_api_unknown_term_returns_empty_array() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;

    index(&app, &cookie, "d1", "alpha beta").await;
    index(&app, &cookie, "d2", "gamma delta").await;

    let hits = search(&app, &cookie, "zzz-nonexistent-token").await;
    assert!(
        hits.is_empty(),
        "unknown term must return [] (no fabricated scores)"
    );
}

#[tokio::test]
async fn search_api_reports_via_provenance_and_direct_fetch_is_neutral() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;

    index(&app, &cookie, "d1", "alpha beta alpha").await;
    index(&app, &cookie, "d2", "gamma delta").await;

    let hits = search(&app, &cookie, "alpha").await;
    assert!(!hits.is_empty(), "ranked query must return hits");
    for hit in &hits {
        let via = hit["via"].as_str().expect("via is a string");
        assert!(
            via == "bm25" || via == "hybrid",
            "ranked via must be bm25 or hybrid, got {via}"
        );
        let score = hit["score"].as_f64().expect("score is a number");
        assert!(
            (score - 1.0).abs() > f64::EPSILON,
            "ranked score must not be the constant 1.0 placeholder"
        );
    }

    let direct = get_json(&app, &cookie, "/api/search/d1").await;
    assert_eq!(direct["via"], "direct", "direct fetch provenance");
    assert_eq!(
        direct["score"].as_f64(),
        Some(1.0),
        "direct fetch uses a neutral score"
    );
}
