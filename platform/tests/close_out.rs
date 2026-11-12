//! Slice 9 close-out tests (P3-2 design-token sync, P3-1 report budget,
//! P1-9 session revocation, P1-8 residual password reset).

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use sqlx::SqlitePool;
use tower::ServiceExt;

use platform::api::{router, AppState};

const TEST_ADMIN_PASSWORD: &str = "test-admin-pass";
const TEST_STORE_PASSWORD: &str = "test-store-pass";

async fn in_memory_pool() -> SqlitePool {
    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory SQLite pool")
}

/// Build (router, pool) with admin + store1 credentials seeded.
async fn seeded_app() -> (Router, SqlitePool) {
    let pool = in_memory_pool().await;
    let state = AppState::new(pool.clone())
        .await
        .expect("app state with migrations applied");
    let app = router(state.clone());

    let admin_hash = platform::auth::hash_password(TEST_ADMIN_PASSWORD).expect("hash");
    sqlx::query(
        "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
         VALUES ('user-admin', 'admin', ?, 1)",
    )
    .bind(&admin_hash)
    .execute(&pool)
    .await
    .expect("seed admin");

    let store_hash = platform::auth::hash_password(TEST_STORE_PASSWORD).expect("hash");
    sqlx::query(
        "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
         VALUES ('user-store1', 'store1', ?, 1)",
    )
    .bind(&store_hash)
    .execute(&pool)
    .await
    .expect("seed store");

    (app, pool)
}

/// Login without assertions; returns (cookie, csrf) on success, None on
/// generic failure (deliberately distinguishes "wrong credentials" from
/// panics so deactivated accounts can be tested honestly).
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
    let session_id = cookie
        .strip_prefix("auxbridge_session=")
        .expect("cookie shape");
    let (csrf,): (String,) = sqlx::query_as("SELECT csrf_token FROM sessions WHERE session_id = ?")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("session row");
    Some((cookie, csrf))
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

// ---------------------------------------------------------------------------
// P3-2: design tokens must not drift between the deployment stylesheet and
// the documentation snapshot (single source of truth rule, SHARED_STANDARDS).
// ---------------------------------------------------------------------------

fn root_block(css: &str) -> String {
    let start = css
        .find(":root")
        .unwrap_or_else(|| panic!(":root block missing"));
    let open = css[start..].find('{').unwrap() + start;
    let close = css[open..].find('}').unwrap() + open;
    css[open..=close]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn css_design_tokens_are_in_sync_between_docs_and_deployment() {
    let deployed = include_str!("../../web/assets/design-system.css");
    let documented = include_str!("../../docs/design-system.css");
    assert_eq!(
        root_block(deployed),
        root_block(documented),
        "design tokens drifted: update web/assets and docs/design-system.css in the same commit"
    );
}

// ---------------------------------------------------------------------------
// P3-1: the monthly report must stay under its 500 ms budget with a
// realistically sized ledger (RPT-001 throughput clause).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn monthly_report_stays_under_500ms_with_2000_transactions() {
    let (app, pool, cookie, _csrf) = {
        let (app, pool) = seeded_app().await;
        let (cookie, csrf) = try_login(&app, &pool, "admin", TEST_ADMIN_PASSWORD)
            .await
            .expect("admin login");
        (app, pool, cookie, csrf)
    };

    // Seed 20 accounts first (items carry FKs to accounts), then 2,000
    // balanced transactions (2 items each) with recursive CTEs: set-based,
    // deterministic, fast to prepare.
    sqlx::query(
        "WITH RECURSIVE seq(n) AS (SELECT 0 UNION ALL SELECT n + 1 FROM seq WHERE n < 19) \
         INSERT INTO accounts (account_id, account_code, account_name, account_type, currency, is_active) \
         SELECT 'ACC-D-' || n, 'AD' || n, 'Perf Debit ' || n, 'ASSETS', 'TWD', 1 FROM seq",
    )
    .execute(&pool)
    .await
    .expect("seed debit accounts");

    sqlx::query(
        "WITH RECURSIVE seq(n) AS (SELECT 0 UNION ALL SELECT n + 1 FROM seq WHERE n < 19) \
         INSERT INTO accounts (account_id, account_code, account_name, account_type, currency, is_active) \
         SELECT 'ACC-C-' || n, 'AC' || n, 'Perf Credit ' || n, 'LIABILITIES', 'TWD', 1 FROM seq",
    )
    .execute(&pool)
    .await
    .expect("seed credit accounts");

    sqlx::query(
        "WITH RECURSIVE seq(n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM seq WHERE n < 2000) \
         INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, status, created_by) \
         SELECT 'tx-perf-' || n, 'TX-P-' || n, date('2026-11-01', '+' || (n % 28) || ' days'), \
                'perf row', 'POSTED', 'tester' FROM seq",
    )
    .execute(&pool)
    .await
    .expect("seed transactions");

    sqlx::query(
        "WITH RECURSIVE seq(n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM seq WHERE n < 2000) \
         INSERT INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency, exchange_rate) \
         SELECT 'it-p-d-' || n, 'tx-perf-' || n, 'ACC-D-' || (n % 20), 'DEBIT', 500 + n, 'TWD', 10000 FROM seq",
    )
    .execute(&pool)
    .await
    .expect("seed debit legs");

    sqlx::query(
        "WITH RECURSIVE seq(n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM seq WHERE n < 2000) \
         INSERT INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency, exchange_rate) \
         SELECT 'it-p-c-' || n, 'tx-perf-' || n, 'ACC-C-' || (n % 20), 'CREDIT', 500 + n, 'TWD', 10000 FROM seq",
    )
    .execute(&pool)
    .await
    .expect("seed credit legs");

    let started = std::time::Instant::now();
    let (status, body, _) = get_with_location(&app, "/reports/monthly?ym=2026-11", &cookie).await;
    let elapsed = started.elapsed();

    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("ACC-D-1"),
        "report must aggregate seeded data"
    );
    assert!(
        elapsed < std::time::Duration::from_millis(500),
        "RPT-001 budget: report took {elapsed:?} (budget 500 ms)"
    );
}

// ---------------------------------------------------------------------------
// P1-9: deactivating an account must revoke its live sessions immediately.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn deactivating_a_user_revokes_their_live_session() {
    let (app, pool, admin_cookie, admin_csrf) = seeded_app_with_admin().await;

    // store1 logs in and holds a live session.
    let (store_cookie, _store_csrf) = try_login(&app, &pool, "store1", TEST_STORE_PASSWORD)
        .await
        .expect("store login");
    let (status, _, _) = get_with_location(&app, "/", &store_cookie).await;
    assert_eq!(status, StatusCode::OK, "store1 starts active");

    // Admin deactivates store1.
    let request = Request::builder()
        .method("POST")
        .uri("/admin/users/toggle")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, admin_cookie)
        .body(Body::from(format!(
            "csrf_token={admin_csrf}&user_id=user-store1"
        )))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("toggle");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    // P1-9: the old session must be dead immediately.
    let (status, _, location) = get_with_location(&app, "/", &store_cookie).await;
    assert_eq!(
        status,
        StatusCode::SEE_OTHER,
        "revoked session must be rejected"
    );
    assert_eq!(location.as_deref(), Some("/login"));
}

async fn seeded_app_with_admin() -> (Router, SqlitePool, String, String) {
    let (app, pool) = seeded_app().await;
    let (cookie, csrf) = try_login(&app, &pool, "admin", TEST_ADMIN_PASSWORD)
        .await
        .expect("admin login");
    (app, pool, cookie, csrf)
}

// ---------------------------------------------------------------------------
// P1-8 residual: administrator password reset.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn admin_password_reset_revokes_sessions_and_rotates_credentials() {
    let (app, pool, admin_cookie, admin_csrf) = seeded_app_with_admin().await;

    // store1 has a live session.
    let (store_cookie, _store_csrf) = try_login(&app, &pool, "store1", TEST_STORE_PASSWORD)
        .await
        .expect("store login");

    // Admin resets store1's password.
    let request = Request::builder()
        .method("POST")
        .uri("/admin/users/reset_password")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, admin_cookie)
        .body(Body::from(format!(
            "csrf_token={admin_csrf}&user_id=user-store1&new_password=new-store-pass-9"
        )))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("reset");
    assert_eq!(response.status(), StatusCode::SEE_OTHER, "reset must PRG");

    // Old password no longer authenticates.
    assert!(
        try_login(&app, &pool, "store1", TEST_STORE_PASSWORD)
            .await
            .is_none(),
        "old password must fail after reset"
    );

    // New password works.
    assert!(
        try_login(&app, &pool, "store1", "new-store-pass-9")
            .await
            .is_some(),
        "new password must authenticate"
    );

    // The pre-reset store session was revoked.
    let (status, _, location) = get_with_location(&app, "/", &store_cookie).await;
    assert_eq!(status, StatusCode::SEE_OTHER, "reset revokes sessions");
    assert_eq!(location.as_deref(), Some("/login"));

    // Reset is audited.
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_log WHERE event_type = 'PASSWORD_RESET' AND performed_by = 'user-admin'",
    )
    .fetch_one(&pool)
    .await
    .expect("audit query");
    assert_eq!(count, 1, "password reset must be audited");
}
