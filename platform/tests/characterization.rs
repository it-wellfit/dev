#![forbid(unsafe_code)]
//! PRE-R1a — route characterization (PIN) suite (plan 1787631643 §PRE-R1a).
//!
//! Every route must answer with its contracted status code and a stable DOM
//! marker. This file is the safety net for the future api.rs module split:
//! the refactor must keep every assertion below unchanged and green.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use platform::api::{router, AppState};
use platform::auth;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
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

fn get(uri: &str, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().uri(uri);
    if let Some(c) = cookie {
        builder = builder.header(header::COOKIE, c);
    }
    builder.body(Body::empty()).expect("build request")
}

fn post(uri: &str, cookie: Option<&str>, body: &str) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    if let Some(c) = cookie {
        builder = builder.header(header::COOKIE, c);
    }
    builder
        .body(Body::from(body.to_string()))
        .expect("build request")
}

async fn login(app: &Router) -> String {
    let request = post(
        "/login",
        None,
        &format!("username=admin&password={TEST_ADMIN_PASSWORD}"),
    );
    let (status, _, cookie) = send(app.clone(), request).await;
    assert_eq!(status, StatusCode::SEE_OTHER, "login must redirect (PRG)");
    cookie.expect("login must set a session cookie")
}

/// Public surface: no session required.
#[tokio::test]
async fn public_routes_contract() {
    let (app, _pool) = in_memory_app().await;
    let (status, body, _) = send(app.clone(), get("/login", None)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("登入"), "login page marker");

    let (status, body, _) = send(app.clone(), get("/api/health", None)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("\"status\":\"ok\""), "health marker");

    let (status, body, _) = send(app, get("/assets/design-system.css", None)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("--color-primary"), "design tokens marker");
}

/// Anonymous access to every protected GET redirects to /login.
#[tokio::test]
async fn anonymous_protected_gets_redirect_to_login() {
    let (app, _pool) = in_memory_app().await;
    const GETS: [&str; 12] = [
        "/",
        "/transactions",
        "/admin/users",
        "/import",
        "/drafts",
        "/new",
        "/reports/monthly",
        "/reports/monthly.csv",
        "/reports/pivot",
        "/reports/parallel",
        "/admin/parse_coverage",
        "/states",
    ];
    for uri in GETS {
        let (status, _, location) = send(app.clone(), get(uri, None)).await;
        assert_eq!(
            status,
            StatusCode::SEE_OTHER,
            "GET {uri} must redirect when anonymous"
        );
        assert_eq!(
            location.as_deref(),
            Some("/login"),
            "GET {uri} redirect target"
        );
    }
}

/// Authenticated GETs render their page markers (or a non-empty body for
/// template-owned report pages).
#[tokio::test]
async fn authenticated_gets_render_markers() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;

    const PAGES: [(&str, &str); 7] = [
        ("/", "交易管理"),
        ("/transactions", "交易清單"),
        ("/admin/users", "使用者"),
        ("/import", "匯入"),
        ("/drafts", "草稿"),
        ("/states", "介面狀態"),
        ("/admin/parse_coverage", "解析覆蓋率"),
    ];
    for (uri, marker) in PAGES {
        let (status, body, _) = send(app.clone(), get(uri, Some(&cookie))).await;
        assert_eq!(status, StatusCode::OK, "GET {uri} must render");
        assert!(body.contains(marker), "GET {uri} marker {marker:?} absent");
    }

    // Monthly report fails closed (400) without `ym`; the PIN sends a valid
    // month so the rendered page is exercised.
    for uri in [
        "/reports/monthly?ym=2026-11",
        "/reports/pivot",
        "/reports/parallel",
        "/new",
    ] {
        let (status, body, _) = send(app.clone(), get(uri, Some(&cookie))).await;
        assert_eq!(status, StatusCode::OK, "GET {uri} must render");
        assert!(body.len() > 200, "GET {uri} body must not be empty");
    }

    let (status, body, _) = send(
        app.clone(),
        get("/reports/monthly.csv?ym=2026-11", Some(&cookie)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "csv export must render");
    assert!(!body.is_empty(), "csv body non-empty");
}

/// Missing transaction detail renders the 404 state (not a 500).
#[tokio::test]
async fn missing_transaction_detail_is_404() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;
    let (status, _, _) = send(app, get("/transactions/tx-pin-missing", Some(&cookie))).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "missing detail must 404");
}

/// Anonymous POSTs are stopped by the auth gate before any handler runs.
#[tokio::test]
async fn anonymous_posts_redirect_to_login() {
    let (app, _pool) = in_memory_app().await;
    const POSTS: [&str; 10] = [
        "/transactions",
        "/admin/users",
        "/admin/users/toggle",
        "/admin/users/reset_password",
        "/import/preview",
        "/import/confirm",
        "/drafts/parse",
        "/new/step2",
        "/drafts/ai_suggest",
        "/drafts/adopt",
    ];
    for uri in POSTS {
        let (status, _, location) = send(app.clone(), post(uri, None, "")).await;
        assert_eq!(
            status,
            StatusCode::SEE_OTHER,
            "POST {uri} must redirect when anonymous"
        );
        assert_eq!(
            location.as_deref(),
            Some("/login"),
            "POST {uri} redirect target"
        );
    }
}

/// Authenticated POSTs with a wrong CSRF token are rejected: never a redirect
/// (no state change), never a 5xx.
#[tokio::test]
async fn authenticated_posts_reject_bad_csrf() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;

    // A complete form body is required so the extractor succeeds and the
    // handler-level CSRF check is what rejects the request (403).
    let (status, _, _) = send(
        app.clone(),
        post(
            "/transactions",
            Some(&cookie),
            "csrf_token=wrong&debit_account=ACC-DEBIT&credit_account=ACC-CREDIT\
             &amount=1.00&currency=TWD&description=pin&transaction_date=2026-11-01",
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "POST /transactions with wrong CSRF must 403"
    );

    const OTHERS: [&str; 9] = [
        "/admin/users",
        "/admin/users/toggle",
        "/admin/users/reset_password",
        "/import/preview",
        "/import/confirm",
        "/drafts/parse",
        "/new/step2",
        "/drafts/ai_suggest",
        "/drafts/adopt",
    ];
    for uri in OTHERS {
        let (status, _, _) = send(app.clone(), post(uri, Some(&cookie), "csrf_token=wrong")).await;
        assert!(
            status.is_client_error(),
            "POST {uri} with wrong CSRF must be a 4xx, got {status}"
        );
    }

    // Logout endpoint must never 5xx, with or without a session.
    let (status, _, _) = send(app, post("/logout", None, "")).await;
    assert!(status.as_u16() < 500, "POST /logout must not 5xx");
}
