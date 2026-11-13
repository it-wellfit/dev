#![forbid(unsafe_code)]
//! 體驗室（Design Lab）驗收測試（計畫 1793708660255 v2.2 §8；2026-11-13）。
//!
//! 契約：
//! 1. 十頁存在且僅 ROLE_MANAGE 可達（admin 200／store、finance 403／visitor 303）。
//! 2. 零 JS：`<script`=0、inline `style=`=0；共用元件（skip-link／app-header／main）。
//! 3. 內容標記：各分區核心語彙存在；子導覽 aria-current 存在。
//! 4. 導覽入口：admin 首頁導覽出現「體驗室」連結；store 導覽不出現（受眾分層）。

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const ADMIN_PASSWORD: &str = "lab-admin-pass";
const STORE_PASSWORD: &str = "lab-store-pass";
const FINANCE_PASSWORD: &str = "lab-finance-pass";

const LAB_PAGES: [&str; 10] = [
    "/design",
    "/design/contracts",
    "/design/components",
    "/design/states",
    "/design/content",
    "/design/blueprints",
    "/design/surfaces",
    "/design/matrices",
    "/design/tokens",
    "/design/decisions",
];

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

async fn seed_users(pool: &sqlx::SqlitePool) {
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
        .execute(pool)
        .await
        .expect("seed user");
    }
    sqlx::query(
        "INSERT OR IGNORE INTO user_roles (user_id, role_id, assigned_by) \
         VALUES ('user-finance', 'role-finance', 'system')",
    )
    .execute(pool)
    .await
    .expect("assign finance role");
}

#[tokio::test]
async fn design_lab_gate_matrix_and_zero_js() {
    let pool = in_memory_pool().await;
    let state = platform::api::AppState::new(pool.clone())
        .await
        .expect("app state");
    let app = platform::api::router(state);
    seed_users(&pool).await;

    // 訪客：全數 303 → /login。
    for uri in LAB_PAGES {
        let (status, _) = get(app.clone(), uri, None).await;
        assert_eq!(status, StatusCode::SEE_OTHER, "visitor {uri} must 303");
    }

    let admin_cookie = login(&app, "admin", ADMIN_PASSWORD).await;
    let store_cookie = login(&app, "store1", STORE_PASSWORD).await;
    let finance_cookie = login(&app, "finance", FINANCE_PASSWORD).await;

    // admin：十頁 200＋零 JS＋共用元件。
    for uri in LAB_PAGES {
        let (status, body) = get(app.clone(), uri, Some(&admin_cookie)).await;
        assert_eq!(status, StatusCode::OK, "admin {uri} must be 200");
        assert!(!body.contains("<script"), "zero-JS violated on {uri}");
        assert!(!body.contains("style="), "inline style violated on {uri}");
        for marker in [
            "<!DOCTYPE html>",
            "/assets/design-system.css",
            "class=\"skip-link\"",
            "class=\"app-header\"",
            "<main",
            "aria-current=\"page\"",
        ] {
            assert!(body.contains(marker), "{uri} missing marker {marker:?}");
        }
    }

    // store／finance：體驗室屬技術面 → 403（ROLE_MANAGE 閘）。
    for uri in LAB_PAGES {
        let (status, _) = get(app.clone(), uri, Some(&store_cookie)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "store {uri} must 403");
        let (status, _) = get(app.clone(), uri, Some(&finance_cookie)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "finance {uri} must 403");
    }

    // 導覽入口（受眾分層）：admin 首頁有 /design；store 首頁沒有。
    let (_, admin_home) = get(app.clone(), "/", Some(&admin_cookie)).await;
    assert!(
        admin_home.contains("href=\"/design\""),
        "admin nav must link design lab"
    );
    let (_, store_home) = get(app.clone(), "/", Some(&store_cookie)).await;
    assert!(
        !store_home.contains("href=\"/design\""),
        "store nav must NOT link design lab (audience layering)"
    );
}

#[tokio::test]
async fn design_lab_content_markers() {
    let pool = in_memory_pool().await;
    let state = platform::api::AppState::new(pool.clone())
        .await
        .expect("app state");
    let app = platform::api::router(state);
    seed_users(&pool).await;
    let admin_cookie = login(&app, "admin", ADMIN_PASSWORD).await;

    let markers: [(&str, &[&str]); 10] = [
        ("/design", &["分區入口", "R0", "EXPERIENCE_SPEC"]),
        ("/design/contracts", &["EC-1", "EC-10", "TC-1", "L1 呈現"]),
        ("/design/components", &["按鈕", "徽章", "分頁", "KPI"]),
        (
            "/design/states",
            &["data-state-empty", "data-state-conflict"],
        ),
        ("/design/content", &["問題＋原因＋處理", "術語"]),
        ("/design/blueprints", &["<pre", "對帳工作檯"]),
        ("/design/surfaces", &["S9", "列印規則", "不適用"]),
        ("/design/matrices", &["pairwise", "UNKNOWN", "EC-10"]),
        ("/design/tokens", &["#2dd4bf", "--space-4", "4.5:1"]),
        ("/design/decisions", &["DEC-DL-01", "DEC-DL-03", "證據"]),
    ];
    for (uri, expected) in markers {
        let (status, body) = get(app.clone(), uri, Some(&admin_cookie)).await;
        assert_eq!(status, StatusCode::OK, "{uri} must be 200");
        for marker in expected {
            assert!(body.contains(marker), "{uri} missing marker {marker:?}");
        }
        // 子導覽 10 分區連結齊備。
        for sub in LAB_PAGES {
            assert!(
                body.contains(&format!("href=\"{sub}\"")),
                "{uri} missing lab sub-nav link {sub}"
            );
        }
    }
}
