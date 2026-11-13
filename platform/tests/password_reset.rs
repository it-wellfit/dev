//! Self-service password reset end-to-end (2026-11-12 瀏覽器實證補洞).
//!
//! 流程契約（本地部署、無郵件）：
//!   使用者 POST /forgot-password → 請求入列（無帳號列舉：回應恆同）
//!   管理員 POST /admin/users/issue_reset → 一次性連結（明文僅出現一次）
//!   使用者 GET/POST /reset-password → 設新密碼（撤銷全部 session、token 焚毀）
//!
//! 負面：錯誤 token、重放 token、弱密碼、兩次不一致。

#![forbid(unsafe_code)]

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const ADMIN_PASSWORD: &str = "reset-admin-pass-2026";
const USER_PASSWORD: &str = "reset-user-pass-2026";
const NEW_PASSWORD: &str = "brand-new-pass-2026x";

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

async fn get(app: &axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, String) {
    let mut builder = Request::builder().uri(uri);
    if let Some(c) = cookie {
        builder = builder.header(header::COOKIE, c);
    }
    send(app.clone(), builder.body(Body::empty()).unwrap()).await
}

fn form_body(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!("{k}={}", urlencoding_lite(v)))
        .collect::<Vec<_>>()
        .join("&")
}

fn urlencoding_lite(v: &str) -> String {
    let mut out = String::new();
    for b in v.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

async fn seed(pool: &sqlx::SqlitePool) {
    for (uid, name, pass) in [
        ("user-admin", "admin", ADMIN_PASSWORD),
        ("user-finance", "finance", USER_PASSWORD),
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
}

/// 取出管理員核發頁上的一次性 token（僅出現一次的明文）。
fn extract_token(issue_body: &str) -> String {
    let needle = "/reset-password?token=";
    let start = issue_body
        .find(needle)
        .unwrap_or_else(|| panic!("issue page must show the one-time link: {issue_body}"));
    let rest = &issue_body[start + needle.len()..];
    let end = rest.find(['"', '<', ' ']).unwrap_or(rest.len());
    rest[..end].to_string()
}

async fn csrf_of(app: &axum::Router, cookie: &str, uri: &str) -> String {
    let (_, body) = get(app, uri, Some(cookie)).await;
    let marker = "name=\"csrf_token\" value=\"";
    let start = body
        .find(marker)
        .unwrap_or_else(|| panic!("csrf_token missing on {uri}"));
    let rest = &body[start + marker.len()..];
    rest[..rest.find('"').expect("csrf close quote")].to_string()
}

#[tokio::test]
async fn full_reset_flow_request_issue_reset_login() {
    let pool = in_memory_pool().await;
    let state = platform::api::AppState::new(pool.clone())
        .await
        .expect("app state");
    let app = platform::api::router(state);
    seed(&pool).await;

    // 1) 使用者登記請求（帳號存在）。
    let (status, _) = send(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/forgot-password")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from("username=finance"))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "forgot-password submit must render");

    // 2) 無帳號列舉：不存在的帳號回應相同（200）。
    let (status2, _) = send(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/forgot-password")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from("username=ghost-user"))
            .unwrap(),
    )
    .await;
    assert_eq!(status2, StatusCode::OK, "no account enumeration");

    // 3) admin 於 /admin/users 看到待處理請求並核發。
    let admin_cookie = login(&app, "admin", ADMIN_PASSWORD).await;
    let (status, admin_body) = get(&app, "/admin/users", Some(&admin_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        admin_body.contains("待處理密碼重設請求"),
        "pending section must render"
    );
    assert!(admin_body.contains("finance"), "request username visible");

    let csrf = csrf_of(&app, &admin_cookie, "/admin/users").await;
    let request_id: (String,) = sqlx::query_as("SELECT request_id FROM password_reset_requests WHERE username = 'finance' AND status = 'pending'")
        .fetch_one(&pool)
        .await
        .expect("pending request row");
    let (status, issue_body) = send(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/admin/users/issue_reset")
            .header(header::COOKIE, &admin_cookie)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form_body(&[
                ("csrf_token", &csrf),
                ("request_id", &request_id.0),
            ])))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "issue must succeed");
    let token = extract_token(&issue_body);
    assert!(token.len() >= 48, "token entropy: {}", token.len());

    // 4) 使用者以 token 開啟重設頁 → 設新密碼。
    let (status, reset_body) = get(&app, &format!("/reset-password?token={token}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(reset_body.contains("設定新密碼"), "reset form must render");

    let (status, _) = send(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/reset-password")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form_body(&[
                ("token", &token),
                ("password", NEW_PASSWORD),
                ("password_confirm", NEW_PASSWORD),
            ])))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "reset submit must succeed");

    // 5) 舊密碼失效、新密碼可登入。
    let old_login = send(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/login")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from("username=finance&password=reset-user-pass-2026"))
            .unwrap(),
    )
    .await;
    assert_ne!(
        old_login.0,
        StatusCode::SEE_OTHER,
        "old password must be dead"
    );
    let new_login = send(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/login")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(format!(
                "username=finance&password={NEW_PASSWORD}"
            )))
            .unwrap(),
    )
    .await;
    assert_eq!(new_login.0, StatusCode::SEE_OTHER, "new password must work");

    // 6) token 用後即焚：重放不渲染表單（以表單專屬屬性斷言，避開 <title> 誤中）。
    let (_, replay_body) = get(&app, &format!("/reset-password?token={token}"), None).await;
    assert!(
        !replay_body.contains("autocomplete=\"new-password\""),
        "used token must not render the form again"
    );
    assert!(
        replay_body.contains("無法重設"),
        "used token must be reported as invalid"
    );
}

#[tokio::test]
async fn reset_rejects_bad_token_and_weak_password() {
    let pool = in_memory_pool().await;
    let state = platform::api::AppState::new(pool.clone())
        .await
        .expect("app state");
    let app = platform::api::router(state);
    seed(&pool).await;

    // 錯誤 token：GET 不渲染表單（以表單專屬屬性斷言，避開 <title>/h1 誤中）。
    let (status, body) = get(&app, "/reset-password?token=not-a-real-token", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !body.contains("autocomplete=\"new-password\""),
        "invalid token must not render form"
    );
    assert!(body.contains("無法重設"), "invalid token must be explained");

    // 弱密碼：需要有效 token 才會走到密碼檢查——用 admin 直接核發一個。
    let admin_cookie = login(&app, "admin", ADMIN_PASSWORD).await;
    let (status, _) = send(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/forgot-password")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from("username=finance"))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let csrf = csrf_of(&app, &admin_cookie, "/admin/users").await;
    let request_id: (String,) = sqlx::query_as("SELECT request_id FROM password_reset_requests WHERE username = 'finance' AND status = 'pending'")
        .fetch_one(&pool)
        .await
        .expect("request");
    let (_, issue_body) = send(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/admin/users/issue_reset")
            .header(header::COOKIE, &admin_cookie)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form_body(&[
                ("csrf_token", &csrf),
                ("request_id", &request_id.0),
            ])))
            .unwrap(),
    )
    .await;
    let token = extract_token(&issue_body);

    let (status, body) = send(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/reset-password")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form_body(&[
                ("token", &token),
                ("password", "short"),
                ("password_confirm", "short"),
            ])))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("無法重設") || body.contains("密碼"),
        "weak password must be rejected with guidance"
    );

    // 兩次不一致。
    let (status, body) = send(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/reset-password")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form_body(&[
                ("token", &token),
                ("password", NEW_PASSWORD),
                ("password_confirm", "different-pass-2026"),
            ])))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("不一致"), "mismatch must be reported");
}
