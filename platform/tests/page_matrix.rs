//! T4 角色矩陣驗收（2026-11-10）：訪客／admin／store1 × 關鍵頁面。
//!
//! 期望：訪客（未登入）HTML 頁 → 303 導向 /login；store1（唯讀操作者）
//! 操作者頁 → 200、治理/技術頁 → 403（含 P0-2 稽核閘）；admin 全通 → 200；
//! 所有回應零 `<script`（零 JS 契約）。

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const ADMIN_PASSWORD: &str = "matrix-admin-pass";
const STORE_PASSWORD: &str = "matrix-store-pass";

async fn in_memory_pool() -> sqlx::SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory pool")
}

async fn send(app: axum::Router, request: Request<Body>) -> (StatusCode, String) {
    let response = app.oneshot(request).await.expect("router response");
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 4 * 1_048_576)
        .await
        .expect("body bytes");
    (status, String::from_utf8_lossy(&body).to_string())
}

async fn login(app: &axum::Router, username: &str, password: &str) -> String {
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "username={username}&password={password}"
        )))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("login response");
    assert_eq!(response.status(), StatusCode::SEE_OTHER, "login must PRG");
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|v| v.to_str().ok())
        .map(|v| v.split(';').next().unwrap_or("").to_string())
        .expect("login must set session cookie")
}

async fn get(app: axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, String) {
    let mut builder = Request::builder().uri(uri);
    if let Some(c) = cookie {
        builder = builder.header(header::COOKIE, c);
    }
    let request = builder.body(Body::empty()).unwrap();
    send(app, request).await
}

#[tokio::test]
async fn role_matrix_visitor_store_admin() {
    let pool = in_memory_pool().await;
    let state = platform::api::AppState::new(pool.clone())
        .await
        .expect("app state");
    let app = platform::api::router(state);

    let admin_hash = platform::auth::hash_password(ADMIN_PASSWORD).expect("hash admin");
    sqlx::query(
        "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
         VALUES ('user-admin', 'admin', ?, 1)",
    )
    .bind(&admin_hash)
    .execute(&pool)
    .await
    .expect("seed admin");
    let store_hash = platform::auth::hash_password(STORE_PASSWORD).expect("hash store");
    sqlx::query(
        "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
         VALUES ('user-store1', 'store1', ?, 1)",
    )
    .bind(&store_hash)
    .execute(&pool)
    .await
    .expect("seed store1");

    // 訪客：一律 303 → /login（HTML 群組）。
    for uri in ["/", "/audit", "/transactions", "/admin/api/catalog"] {
        let (status, _) = get(app.clone(), uri, None).await;
        assert_eq!(
            status,
            StatusCode::SEE_OTHER,
            "visitor {uri} must redirect to login"
        );
    }

    let admin_cookie = login(&app, "admin", ADMIN_PASSWORD).await;
    let store_cookie = login(&app, "store1", STORE_PASSWORD).await;

    // admin：全通 + 零 JS。
    for uri in [
        "/",
        "/audit",
        "/transactions",
        "/search",
        "/states",
        "/bankpay",
        "/reports/monthly",
        "/admin/api/catalog",
        "/admin/api/records",
        "/admin/users",
    ] {
        let (status, body) = get(app.clone(), uri, Some(&admin_cookie)).await;
        assert_eq!(status, StatusCode::OK, "admin {uri} must be 200");
        assert!(
            !body.contains("<script"),
            "zero-JS contract violated on {uri}"
        );
    }

    // store1（唯讀操作者）：操作者頁 200。
    for uri in ["/", "/transactions", "/search", "/states"] {
        let (status, _) = get(app.clone(), uri, Some(&store_cookie)).await;
        assert_eq!(status, StatusCode::OK, "store {uri} must be 200");
    }

    // store1：治理／技術頁 403（含 P0-2 稽核閘）。
    for uri in ["/audit", "/admin/api/catalog", "/admin/users"] {
        let (status, _) = get(app.clone(), uri, Some(&store_cookie)).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "store {uri} must be 403 (governance gate)"
        );
    }
}

// ---------------------------------------------------------------------------
// W5（計畫 1793033568961）：finance 身份、安全標頭、八態掛點、導覽可達、
// /console 靜態伺服、死連結回歸防護（B7a）。
// ---------------------------------------------------------------------------

const FINANCE_PASSWORD: &str = "matrix-finance-pass";

async fn get_full(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, axum::http::HeaderMap, String) {
    let mut builder = Request::builder().uri(uri);
    if let Some(c) = cookie {
        builder = builder.header(header::COOKIE, c);
    }
    let request = builder.body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.expect("router response");
    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), 4 * 1_048_576)
        .await
        .expect("body bytes");
    (status, headers, String::from_utf8_lossy(&body).to_string())
}

#[tokio::test]
async fn w2_w6_nav_matrix_headers_anchors_console() {
    let pool = in_memory_pool().await;
    let state = platform::api::AppState::new(pool.clone())
        .await
        .expect("app state");
    let app = platform::api::router(state);

    // 三身份種子（admin/store1 由 initialize_default 自動授權；finance 補指派）。
    for (uid, name, pass) in [
        ("user-admin", "admin", ADMIN_PASSWORD),
        ("user-store1", "store1", STORE_PASSWORD),
        ("user-finance", "finance", FINANCE_PASSWORD),
    ] {
        let hash = platform::auth::hash_password(pass).expect("hash");
        sqlx::query(
            "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
             VALUES (?, ?, ?, 1)",
        )
        .bind(uid)
        .bind(name)
        .bind(&hash)
        .execute(&pool)
        .await
        .expect("seed user");
    }
    // finance → role-finance（LEDGER_WRITE＋LEDGER_READ；initialize_default 已種角色）。
    sqlx::query(
        "INSERT OR IGNORE INTO user_roles (user_id, role_id, assigned_by) \
         VALUES ('user-finance', 'role-finance', 'system')",
    )
    .execute(&pool)
    .await
    .expect("assign finance role");
    // 科目種子（F4 詳情頁 200 路徑）。
    sqlx::query(
        "INSERT OR IGNORE INTO accounts (account_id, account_code, account_name, account_type, currency, is_active) \
         VALUES ('ACC-A', 'A1', 'Account A', 'ASSETS', 'TWD', 1)",
    )
    .execute(&pool)
    .await
    .expect("seed account");

    let admin_cookie = login(&app, "admin", ADMIN_PASSWORD).await;
    let store_cookie = login(&app, "store1", STORE_PASSWORD).await;
    let finance_cookie = login(&app, "finance", FINANCE_PASSWORD).await;

    // 1) finance：Write 頁 200（/new /import /drafts /ai-decisions）。
    for uri in ["/new", "/import", "/drafts", "/ai-decisions"] {
        let (status, _) = get(app.clone(), uri, Some(&finance_cookie)).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "finance {uri} (LEDGER_WRITE) must be 200"
        );
    }
    // 2) finance：治理頁 403（ROLE_MANAGE 閘）。
    for uri in ["/audit", "/admin/users", "/admin/api/catalog"] {
        let (status, _) = get(app.clone(), uri, Some(&finance_cookie)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "finance {uri} must be 403");
    }
    // 2b) finance：科目詳情（LEDGER_READ）200。
    let (status, _) = get(app.clone(), "/accounts/ACC-A", Some(&finance_cookie)).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "finance /accounts/ACC-A must be 200"
    );
    // 2c) 未知科目 → 404（不洩漏存在性）。
    let (status, _) = get(app.clone(), "/accounts/NOPE", Some(&finance_cookie)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "unknown account must be 404");

    // 3) admin：安全標頭（CSP/no-store/nosniff/frame-deny）。
    let (status, headers, _) = get_full(app.clone(), "/", Some(&admin_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    let csp = headers
        .get(header::CONTENT_SECURITY_POLICY)
        .map(|v| v.to_str().unwrap_or("").to_string())
        .unwrap_or_default();
    assert!(csp.contains("default-src 'none'"), "CSP missing: {csp}");
    assert!(
        csp.contains("frame-ancestors 'none'"),
        "frame-ancestors missing"
    );
    assert_eq!(
        headers
            .get(header::X_CONTENT_TYPE_OPTIONS)
            .and_then(|v| v.to_str().ok()),
        Some("nosniff")
    );
    assert_eq!(
        headers
            .get(header::X_FRAME_OPTIONS)
            .and_then(|v| v.to_str().ok()),
        Some("DENY")
    );

    // 4) admin GET /：導覽可達——manifest 宣告之 nav 頁 href 全數出現。
    let (_, _, body) = get_full(app.clone(), "/", Some(&admin_cookie)).await;
    for href in [
        "href=\"/\"",
        "href=\"/transactions\"",
        "href=\"/new\"",
        "href=\"/accounts\"",
        "href=\"/ai-decisions\"",
        "href=\"/import\"",
        "href=\"/drafts\"",
        "href=\"/bankpay\"",
        "href=\"/reports/monthly\"",
        "href=\"/reports/trial-balance\"",
        "href=\"/reports/ledger\"",
        "href=\"/search\"",
        "href=\"/audit\"",
        "href=\"/admin/users\"",
        "href=\"/admin/api/catalog\"",
        "href=\"/status\"",
        "href=\"/help\"",
        "href=\"/profile\"",
        "href=\"/console/index.html\"",
    ] {
        assert!(body.contains(href), "nav reachability missing {href}");
    }
    // 5) aria-current 命中目前頁（首頁）。
    assert!(
        body.contains("aria-current=\"page\""),
        "aria-current missing on /"
    );
    // 6) 八態掛點（W4）。
    assert!(
        body.contains("data-state-empty"),
        "state anchor empty missing"
    );
    assert!(
        body.contains("data-state-denied"),
        "state anchor denied missing"
    );
    assert!(
        body.contains("data-state-error"),
        "state anchor error missing"
    );

    // 7) store1 死連結回歸（B7a）：無 Manage 權 → 導覽不得出現 /audit 與 /admin。
    let (_, _, store_body) = get_full(app.clone(), "/", Some(&store_cookie)).await;
    assert!(
        !store_body.contains("href=\"/audit\""),
        "store1 nav must NOT link /audit (dead link regression)"
    );
    assert!(
        !store_body.contains("href=\"/admin/users\""),
        "store1 nav must NOT link /admin/users"
    );
    // store1 仍見 All 頁與知識群組。
    assert!(store_body.contains("href=\"/console/index.html\""));

    // 8) /console 靜態伺服：admin 200＋零 JS；store1 200（All 閘）；visitor 303。
    let (status, headers, console_body) =
        get_full(app.clone(), "/console/index.html", Some(&admin_cookie)).await;
    assert_eq!(status, StatusCode::OK, "console index must be 200");
    assert!(
        !console_body.contains("<script"),
        "console page zero-JS violated"
    );
    assert_eq!(
        headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("text/html; charset=utf-8")
    );
    let (status, _) = get(app.clone(), "/console/index.html", Some(&store_cookie)).await;
    assert_eq!(status, StatusCode::OK, "console index (All gate) store 200");
    let (status, _) = get(app.clone(), "/console/index.html", None).await;
    assert_eq!(status, StatusCode::SEE_OTHER, "visitor console must 303");

    // 8b) /console 入口轉址與 CSS。
    let (status, _, _) = get_full(app.clone(), "/console", Some(&admin_cookie)).await;
    assert_eq!(
        status,
        StatusCode::SEE_OTHER,
        "/console must redirect to index"
    );
    let (status, headers, _) = get_full(
        app.clone(),
        "/console/assets/console.css",
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "console.css must be 200");
    assert_eq!(
        headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("text/css; charset=utf-8")
    );

    // 9) 穿越防護：fail-closed 404（任一形態皆不得外洩 repo 檔案）。
    for evil in [
        "/console/../Cargo.toml",
        "/console/..%2FCargo.toml",
        "/console/a/../../Cargo.toml",
        "/console/.git/config",
        "/console/C:/Windows/win.ini",
    ] {
        let (status, _, _) = get_full(app.clone(), evil, Some(&admin_cookie)).await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "traversal {evil} must be 404"
        );
    }

    // 10) /console/*json 白名單：主張機器 SSOT 頁可得。
    let (status, _, _) = get_full(
        app.clone(),
        "/console/claims/claim-20261107-001.json",
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "claims json whitelist must be 200");
}
