#![forbid(unsafe_code)]
//! G4.7 (D2-11) — performance baseline for `/api/search` (BM25-only) and TTFB.
//!
//! Method: in-memory SQLite, 200 indexed documents, 1 warm-up + 50 samples,
//! report p50/p95. The vector leg is NOT measured here: no embedder is
//! reachable in this environment, so the vector path is recorded as UNKNOWN
//! (C-10: UNKNOWN != PASS). The BM25-only path is deterministic.
//!
//! Run with `cargo test --locked -p platform --test perf_baseline -- --nocapture`.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use platform::api::{AppState, router};
use platform::auth;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const TEST_ADMIN_PASSWORD: &str = "Admin-Test-Passw0rd!";
const DOCS: usize = 200;
/// 20 samples + 1 warm-up = 21 requests, below the 30/min API rate limit.
const SAMPLES: usize = 20;

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

async fn login_with_pool(app: &Router) -> String {
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

fn pct(mut samples: Vec<f64>, p: f64) -> f64 {
    samples.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    let idx = ((samples.len() as f64) * p).ceil() as usize;
    samples[idx.saturating_sub(1).min(samples.len() - 1)]
}

async fn timed(app: &Router, uri: &str, cookie: &str) -> f64 {
    let start = std::time::Instant::now();
    let request = Request::builder()
        .uri(uri)
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK, "GET {uri}");
    let _ = axum::body::to_bytes(response.into_body(), 8_388_608)
        .await
        .expect("body");
    start.elapsed().as_secs_f64() * 1000.0
}

#[tokio::test]
async fn search_bm25_only_and_ttfb_baseline() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login_with_pool(&app).await;

    for i in 0..DOCS {
        let body = serde_json::json!({
            "doc_id": format!("doc-{i}"),
            "content": format!("alpha beta gamma {i} 財務報表 庫存 交易明細")
        })
        .to_string();
        let request = Request::builder()
            .method("POST")
            .uri("/api/search")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, &cookie)
            .body(Body::from(body))
            .expect("index request");
        let response = app.clone().oneshot(request).await.expect("index response");
        assert_eq!(response.status(), StatusCode::OK);
    }

    // Warm-up (JIT-free but warms caches / migrations).
    let _ = timed(&app, "/api/search?q=alpha", &cookie).await;

    let mut search = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        search.push(timed(&app, "/api/search?q=alpha", &cookie).await);
    }
    let mut ttfb = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        ttfb.push(timed(&app, "/", &cookie).await);
    }

    println!(
        "search(bm25-only) p50={:.2}ms p95={:.2}ms | workbench TTFB p50={:.2}ms p95={:.2}ms",
        pct(search.clone(), 0.50),
        pct(search.clone(), 0.95),
        pct(ttfb.clone(), 0.50),
        pct(ttfb.clone(), 0.95)
    );

    // Generous CI bounds; the recorded baseline lives in the operations doc.
    assert!(pct(search.clone(), 0.95) < 2_000.0, "search p95 budget");
    assert!(pct(ttfb, 0.95) < 2_000.0, "TTFB p95 budget");
}
