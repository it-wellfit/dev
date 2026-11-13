#![forbid(unsafe_code)]
//! E.Sun BankPay 與系統狀態 API 整合測試（Slice 13／計畫 1793782615513）。
//!
//! 驗收對照（計畫 §9.1）：
//! - 未登入存取 BankPay API → 401（`require_session_api`）。
//! - 無 LEDGER_WRITE → 403；變更類 POST 缺 X-CSRF-Token → 403。
//! - 匯出成功寫入 `bankpay_batches`（SUBMITTED）與 `audit_log`
//!   （BANKPAY_EXPORT_SUCCESS）；同 batch_id 重放為冪等（不重複呼叫銀行）。
//! - 輸入驗證（金額/幣別/識別碼）→ 400；匯入路徑穿越 → 400。
//! - /api/status 回應形狀與模擬模式警示。

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
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

/// 以真實 RBAC 資料列授權（role → role_permissions → user_roles）。
/// 註：`PermissionManager::initialize_default` 會自動種子 role-admin 並指派
/// user-admin；本函式確保種子存在（冪等），供 200 路徑測試使用。
async fn grant_admin_permissions(pool: &SqlitePool) {
    sqlx::query(
        "INSERT OR IGNORE INTO roles (role_id, role_name, role_code, is_system) \
         VALUES ('role-admin', '管理員', 'ADMIN', 1)",
    )
    .execute(pool)
    .await
    .expect("seed role");
    for (pid, code) in [
        ("perm-ledger-write", "LEDGER_WRITE"),
        ("perm-ledger-read", "LEDGER_READ"),
        ("perm-role-manage", "ROLE_MANAGE"),
    ] {
        sqlx::query(
            "INSERT OR IGNORE INTO permissions \
             (permission_id, permission_name, permission_code, permission_type, resource, action, is_active) \
             VALUES (?, ?, ?, 'api', 'bankpay', 'execute', 1)",
        )
        .bind(pid)
        .bind(code)
        .bind(code)
        .execute(pool)
        .await
        .expect("seed permission");
        sqlx::query("INSERT OR IGNORE INTO role_permissions (role_id, permission_id, granted_by) VALUES ('role-admin', ?, 'system')")
            .bind(pid)
            .execute(pool)
            .await
            .expect("grant to role");
    }
    sqlx::query("INSERT OR IGNORE INTO user_roles (user_id, role_id, assigned_by) VALUES ('user-admin', 'role-admin', 'system')")
        .execute(pool)
        .await
        .expect("assign role to admin");
}

/// 種子一個「沒有任何角色」的使用者（無 user_roles 列），供 RBAC 403 測試。
async fn seed_user_without_roles(pool: &SqlitePool) {
    let hash = auth::hash_password(TEST_ADMIN_PASSWORD).expect("hash password");
    sqlx::query(
        "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
         VALUES ('user-nobody', 'nobody', ?, 1)",
    )
    .bind(&hash)
    .execute(pool)
    .await
    .expect("seed roleless user");
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

/// JSON POST（可帶 X-CSRF-Token）。
fn post_json(uri: &str, cookie: &str, csrf: Option<&str>, body: &str) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, cookie);
    if let Some(t) = csrf {
        builder = builder.header("x-csrf-token", t);
    }
    builder
        .body(Body::from(body.to_string()))
        .expect("build request")
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
    let (status, _, cookie) = send(app.clone(), request).await;
    assert_eq!(status, StatusCode::SEE_OTHER, "login must redirect (PRG)");
    cookie.expect("login must set a session cookie")
}

/// 從首頁 HTML 取出本 session 的 CSRF 權杖（同步器權杖模式的測試對應）。
async fn fetch_csrf(app: &Router, cookie: &str) -> String {
    let (status, body, _) = send(app.clone(), get("/", Some(cookie))).await;
    assert_eq!(status, StatusCode::OK);
    let marker = "name=\"csrf_token\" value=\"";
    let start = body.find(marker).expect("csrf token marker in index") + marker.len();
    let rest = &body[start..];
    let end = rest.find('"').expect("closing quote of csrf token");
    rest[..end].to_string()
}

// ── 401：未登入一律拒絕（含先前未受保護的 cross_module 路由） ──

#[tokio::test]
async fn bankpay_api_requires_session() {
    let (app, _pool) = in_memory_app().await;
    for (method, uri) in [
        ("GET", "/api/bankpay/health"),
        ("GET", "/api/bankpay/query?esun_ref=X"),
        ("GET", "/api/status"),
        ("GET", "/api/audit"),
        ("GET", "/api/modules/health"),
    ] {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .expect("req");
        let (status, _, _) = send(app.clone(), request).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri} must 401");
    }
    let (status, body, _) = send(
        app.clone(),
        post_json(
            "/api/bankpay/export",
            "auxbridge_session=forged",
            None,
            "{}",
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "export must 401");
    assert!(body.contains("未授權"), "401 body explains the problem");
}

// ── 403：RBAC 與 CSRF ──

#[tokio::test]
async fn bankpay_export_requires_ledger_write() {
    let (app, pool) = in_memory_app().await;
    seed_user_without_roles(&pool).await;
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "username=nobody&password={TEST_ADMIN_PASSWORD}"
        )))
        .expect("login request");
    let (login_status, _, cookie) = send(app.clone(), request).await;
    assert_eq!(login_status, StatusCode::SEE_OTHER);
    let cookie = cookie.expect("session cookie");
    let csrf = fetch_csrf(&app, &cookie).await;
    let (status, body, _) = send(
        app.clone(),
        post_json(
            "/api/bankpay/export",
            &cookie,
            Some(&csrf),
            r#"{"batch_id":"B-1","amount_minor":50000,"currency":"TWD"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "no LEDGER_WRITE must 403");
    assert!(body.contains("LEDGER_WRITE"));
}

#[tokio::test]
async fn bankpay_export_requires_csrf_token() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    grant_admin_permissions(&pool).await;
    let cookie = login(&app).await;
    let (status, body, _) = send(
        app.clone(),
        post_json(
            "/api/bankpay/export",
            &cookie,
            None,
            r#"{"batch_id":"B-1","amount_minor":50000,"currency":"TWD"}"#,
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "missing X-CSRF-Token must 403"
    );
    assert!(body.contains("CSRF"));
    // 防禦縱深：CSRF 拒絕的請求絕不可留下任何批次或稽核痕跡。
    let batches: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bankpay_batches")
        .fetch_one(&pool)
        .await
        .expect("count batches");
    assert_eq!(batches, 0, "no batch row may be created without CSRF");
}

// ── 200 路徑：驗證 → 冪等匯出 → 稽核 → 查詢推進 ──

#[tokio::test]
async fn bankpay_export_validates_input() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    grant_admin_permissions(&pool).await;
    let cookie = login(&app).await;
    let csrf = fetch_csrf(&app, &cookie).await;

    for (payload, why) in [
        (
            r#"{"batch_id":"B-1","amount_minor":0,"currency":"TWD"}"#,
            "zero amount",
        ),
        (
            r#"{"batch_id":"B-1","amount_minor":-5,"currency":"TWD"}"#,
            "negative amount",
        ),
        (
            r#"{"batch_id":"B-1","amount_minor":100,"currency":"tw"}"#,
            "bad currency",
        ),
        (
            r#"{"batch_id":"../evil","amount_minor":100,"currency":"TWD"}"#,
            "bad id",
        ),
    ] {
        let (status, body, _) = send(
            app.clone(),
            post_json("/api/bankpay/export", &cookie, Some(&csrf), payload),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{why} must 400");
        assert!(
            body.contains("驗證失敗"),
            "{why} message must be actionable"
        );
    }
}

#[tokio::test]
async fn bankpay_export_is_idempotent_and_audited() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    grant_admin_permissions(&pool).await;
    let cookie = login(&app).await;
    let csrf = fetch_csrf(&app, &cookie).await;
    let payload = r#"{"batch_id":"B-1","amount_minor":50000,"currency":"TWD"}"#;

    let (status1, body1, _) = send(
        app.clone(),
        post_json("/api/bankpay/export", &cookie, Some(&csrf), payload),
    )
    .await;
    assert_eq!(status1, StatusCode::OK, "first export: {body1}");
    assert!(body1.contains("SUBMITTED"), "status submitted");
    assert!(body1.contains("ESUN-MOCK-B-1"), "deterministic esun_ref");

    let (status2, body2, _) = send(
        app.clone(),
        post_json("/api/bankpay/export", &cookie, Some(&csrf), payload),
    )
    .await;
    assert_eq!(status2, StatusCode::OK, "replay export: {body2}");
    assert_eq!(body1, body2, "replay must return the stored state");

    let batches: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bankpay_batches WHERE batch_id = 'B-1'")
            .fetch_one(&pool)
            .await
            .expect("count batches");
    assert_eq!(batches, 1, "unique batch_id: no duplicate rows");

    let status_in_db: String =
        sqlx::query_scalar("SELECT status FROM bankpay_batches WHERE batch_id = 'B-1'")
            .fetch_one(&pool)
            .await
            .expect("batch status");
    assert_eq!(status_in_db, "SUBMITTED");

    let audits: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE event_type = 'BANKPAY_EXPORT_SUCCESS'",
    )
    .fetch_one(&pool)
    .await
    .expect("count audits");
    assert!(
        audits >= 1,
        "audit rows must be written (was silently lost before)"
    );

    let performed_by: String = sqlx::query_scalar(
        "SELECT performed_by FROM audit_log WHERE event_type = 'BANKPAY_EXPORT_SUCCESS' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("audit actor");
    assert_eq!(performed_by, "user-admin", "audit is actor-signed");
}

#[tokio::test]
async fn bankpay_query_advances_batch_status() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    grant_admin_permissions(&pool).await;
    let cookie = login(&app).await;
    let csrf = fetch_csrf(&app, &cookie).await;

    let (status, _, _) = send(
        app.clone(),
        post_json(
            "/api/bankpay/export",
            &cookie,
            Some(&csrf),
            r#"{"batch_id":"B-9","amount_minor":12345,"currency":"TWD"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (qstatus, qbody, _) = send(
        app.clone(),
        get("/api/bankpay/query?esun_ref=ESUN-MOCK-B-9", Some(&cookie)),
    )
    .await;
    assert_eq!(qstatus, StatusCode::OK, "query: {qbody}");
    assert!(qbody.contains("COMPLETED"), "bank status completed");
    assert!(
        qbody.contains("12345"),
        "amount overlaid from local batch ledger"
    );
    assert!(
        qbody.contains("TWD"),
        "currency overlaid from local batch ledger"
    );

    let status_in_db: String =
        sqlx::query_scalar("SELECT status FROM bankpay_batches WHERE batch_id = 'B-9'")
            .fetch_one(&pool)
            .await
            .expect("batch status");
    assert_eq!(
        status_in_db, "COMPLETED",
        "SUBMITTED → COMPLETED transition persisted"
    );
}

// ── 匯入：路徑白名單 ──

#[tokio::test]
async fn bankpay_import_rejects_path_traversal() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    grant_admin_permissions(&pool).await;
    let cookie = login(&app).await;
    let csrf = fetch_csrf(&app, &cookie).await;

    for evil in ["../../Cargo.toml", "C:/Windows/win.ini", "a/../../b.csv"] {
        let (status, body, _) = send(
            app.clone(),
            post_json(
                "/api/bankpay/import",
                &cookie,
                Some(&csrf),
                &format!(r#"{{"file_path":"{evil}"}}"#),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{evil} must 400");
        assert!(
            body.contains("白名單") || body.contains("驗證失敗"),
            "{evil} message"
        );
    }
}

#[tokio::test]
async fn bankpay_import_parses_whitelisted_file() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    grant_admin_permissions(&pool).await;
    let cookie = login(&app).await;
    let csrf = fetch_csrf(&app, &cookie).await;

    // 測試執行目錄為 platform/（package root），白名單目錄相對於此。
    let dir = std::path::Path::new("data/bankpay/inbound");
    std::fs::create_dir_all(dir).expect("create import dir");
    let file = dir.join("it_responses.csv");
    std::fs::write(
        &file,
        "ref,status,amount,currency\nR1,OK,100,TWD\nR2,OK,200,TWD\nBAD\n",
    )
    .expect("write csv");

    let (status, body, _) = send(
        app.clone(),
        post_json(
            "/api/bankpay/import",
            &cookie,
            Some(&csrf),
            r#"{"file_path":"it_responses.csv"}"#,
        ),
    )
    .await;
    let _ = std::fs::remove_file(&file);
    assert_eq!(status, StatusCode::OK, "import: {body}");
    assert!(body.contains(r#""matched":2"#), "real parsed count: {body}");
    assert!(
        body.contains(r#""mismatched":1"#),
        "malformed row counted: {body}"
    );

    let audits: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE event_type = 'BANKPAY_IMPORT_SUCCESS'",
    )
    .fetch_one(&pool)
    .await
    .expect("count audits");
    assert_eq!(audits, 1);
}

// ── /api/status：回應形狀與誠實狀態 ──

#[tokio::test]
async fn status_endpoint_shape_and_mock_warning() {
    let (app, _pool) = in_memory_app().await;
    seed_admin(&_pool).await;
    let cookie = login(&app).await;

    let (status, body, _) = send(app.clone(), get("/api/status", Some(&cookie))).await;
    assert_eq!(status, StatusCode::OK);
    for key in [
        "\"database\"",
        "\"search\"",
        "\"sync\"",
        "\"com\"",
        "\"ai\"",
        "\"bankpay\"",
        "\"modules\"",
        "\"status\"",
    ] {
        assert!(body.contains(key), "status must expose {key}");
    }
    // 模擬模式 = Warning：整體降級且模組訊息明示「模擬」（不虛構 healthy）。
    assert!(body.contains("\"warning\""), "mock-mode bankpay must warn");
    assert!(
        body.contains("degraded"),
        "mock mode degrades overall status"
    );
}

// ── 狀態機除錯端點 ──

#[tokio::test]
async fn transition_endpoint_validates_state_machine() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;

    let (status, body, _) = send(
        app.clone(),
        post_json(
            "/api/bankpay/transition",
            &cookie,
            None,
            r#"{"from":"Pending","to":"Completed"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("\"valid\":false"),
        "Pending→Completed is illegal"
    );

    let (status, body, _) = send(
        app.clone(),
        post_json(
            "/api/bankpay/transition",
            &cookie,
            None,
            r#"{"from":"Pending","to":"Submitted"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("\"valid\":true"),
        "Pending→Submitted is legal"
    );
}

// ── 作業頁（零 JS 表單） ──

#[tokio::test]
async fn bankpay_page_renders_operations_and_requires_session() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    // 未登入 → HTML 群組 303 轉址（與 API 群組的 401 JSON 區分）。
    let (status, _, _) = send(app.clone(), get("/bankpay", None)).await;
    assert_eq!(status, StatusCode::SEE_OTHER, "unauth page must redirect");

    let cookie = login(&app).await;
    let (status, body, _) = send(app.clone(), get("/bankpay", Some(&cookie))).await;
    assert_eq!(status, StatusCode::OK, "page renders");
    for marker in [
        "匯出付款批次",
        "匯入銀行回應檔",
        "查詢交易狀態",
        "name=\"csrf_token\"",
        "action=\"/bankpay/export\"",
        "action=\"/bankpay/import\"",
        "id=\"status-bar\"",
        "模擬模式",
    ] {
        assert!(body.contains(marker), "page must contain {marker}");
    }
}

#[tokio::test]
async fn bankpay_export_form_creates_batch_and_prgs() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;
    let csrf = fetch_csrf(&app, &cookie).await;

    let form = format!(
        "csrf_token={}&batch_id=FORM-1&amount=500.00&currency=TWD",
        urlencode(&csrf)
    );
    let request = Request::builder()
        .method("POST")
        .uri("/bankpay/export")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &cookie)
        .body(Body::from(form))
        .expect("form request");
    let (status, _, location) = send(app.clone(), request).await;
    assert_eq!(status, StatusCode::SEE_OTHER, "success must PRG");
    let loc = location.expect("PRG location");
    assert!(
        loc.contains("batch=FORM-1"),
        "PRG targets focused batch: {loc}"
    );

    // PRG 目標頁顯示批次結果與銀行參照。
    let (status, body, _) = send(app.clone(), get("/bankpay?batch=FORM-1", Some(&cookie))).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("ESUN-MOCK-FORM-1"),
        "focused batch esun_ref: {body}"
    );
    assert!(body.contains("處理中"), "SUBMITTED renders Chinese badge");

    let amount: i64 =
        sqlx::query_scalar("SELECT amount_minor FROM bankpay_batches WHERE batch_id = 'FORM-1'")
            .fetch_one(&pool)
            .await
            .expect("batch row");
    assert_eq!(
        amount, 50000,
        "decimal form input converted exactly to minor units"
    );
}

#[tokio::test]
async fn bankpay_import_form_parses_and_prgs_with_counts() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login(&app).await;
    let csrf = fetch_csrf(&app, &cookie).await;

    let dir = std::path::Path::new("data/bankpay/inbound");
    std::fs::create_dir_all(dir).expect("mkdir");
    std::fs::write(
        dir.join("form.csv"),
        "ref,status,amount,currency\nR1,OK,1,TWD\nR2,OK,2,TWD\nBAD\n",
    )
    .expect("write csv");

    let form = format!("csrf_token={}&file_name=form.csv", urlencode(&csrf));
    let request = Request::builder()
        .method("POST")
        .uri("/bankpay/import")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &cookie)
        .body(Body::from(form))
        .expect("form request");
    let (status, _, location) = send(app.clone(), request).await;
    let _ = std::fs::remove_file(dir.join("form.csv"));
    assert_eq!(status, StatusCode::SEE_OTHER);
    let loc = location.expect("PRG location");
    assert!(
        loc.contains("imported=2:1"),
        "PRG carries parsed counts: {loc}"
    );
}

#[tokio::test]
async fn bankpay_config_form_requires_manage_and_saves() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    seed_user_without_roles(&pool).await;
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "username=nobody&password={TEST_ADMIN_PASSWORD}"
        )))
        .expect("login request");
    let (_, _, cookie) = send(app.clone(), request).await;
    let cookie = cookie.expect("session");
    let csrf = fetch_csrf(&app, &cookie).await;
    let form = format!("csrf_token={}&esun_mock_mode=on", urlencode(&csrf));
    let request = Request::builder()
        .method("POST")
        .uri("/bankpay/config")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &cookie)
        .body(Body::from(form))
        .expect("form request");
    let (status, _, _) = send(app.clone(), request).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "config needs ROLE_MANAGE");

    // 管理者：儲存成功 → PRG + erp_config 持久化。
    grant_admin_permissions(&pool).await;
    let cookie = login(&app).await;
    let csrf = fetch_csrf(&app, &cookie).await;
    let form = format!(
        "csrf_token={}&esun_url=https://esun.example.test&source_system_id=SRC-ERP&customer_id=CUST-001",
        urlencode(&csrf)
    );
    let request = Request::builder()
        .method("POST")
        .uri("/bankpay/config")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &cookie)
        .body(Body::from(form))
        .expect("form request");
    let (status, _, location) = send(app.clone(), request).await;
    assert_eq!(status, StatusCode::SEE_OTHER, "config save must PRG");
    assert!(location.expect("loc").contains("saved=1"));
    let url_in_db: String = sqlx::query_scalar(
        "SELECT config_value FROM erp_config WHERE config_id = 'bankpay:esun_url'",
    )
    .fetch_one(&pool)
    .await
    .expect("persisted config");
    assert_eq!(url_in_db, "https://esun.example.test");
}

// ── 桌面版（Tauri）UI 整合 ──

#[tokio::test]
async fn tauri_preview_serves_desktop_ui_with_cross_nav() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    // 公開路由（純靜態設計文件，無任何資料）：未登入亦 200。
    let (status, _, _) = send(app.clone(), get("/tauri-preview", None)).await;
    assert_eq!(status, StatusCode::OK);

    let cookie = login(&app).await;
    let (status, body, _) = send(app.clone(), get("/tauri-preview", Some(&cookie))).await;
    assert_eq!(status, StatusCode::OK, "desktop UI served verbatim");
    assert!(
        body.contains("銀行金流整合平台"),
        "legacy 1420 page title present"
    );
    // 雙向整合連結：舊版頁內可回伺服器工作台。
    assert!(
        body.contains("工作台（伺服器版）"),
        "cross-link back to platform must exist"
    );
    // 平台導航「不得」出現開發紀錄入口（使用者優先；路由仍存在供開發）。
    let (_, index_body, _) = send(app.clone(), get("/", Some(&cookie))).await;
    assert!(
        !index_body.contains(r#"href="/tauri-preview""#),
        "dev artifact must not appear in user-facing nav"
    );
}

/// 極簡百分號編碼（測試用；權杖字元集為 URL 安全字元，僅防禦性處理）。
fn urlencode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[tokio::test]
async fn index_renders_bankpay_card_and_status_bar() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    grant_admin_permissions(&pool).await;
    let cookie = login(&app).await;
    let csrf = fetch_csrf(&app, &cookie).await;

    let (status, _, _) = send(
        app.clone(),
        post_json(
            "/api/bankpay/export",
            &cookie,
            Some(&csrf),
            r#"{"batch_id":"B-CARD","amount_minor":99000,"currency":"TWD"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body, _) = send(app.clone(), get("/", Some(&cookie))).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("id=\"status-bar\""),
        "global status bar rendered"
    );
    assert!(
        body.contains("id=\"status-panel\""),
        "status detail panel rendered"
    );
    assert!(body.contains("id=\"bankpay\""), "bankpay card rendered");
    assert!(body.contains("B-CARD"), "recent batch listed in card");
    assert!(body.contains("模擬模式"), "mock mode badge must be visible");
    // 計畫 T8：導航不再出現「介面狀態」連結（/states 路由仍存在）。
    assert!(
        !body.contains("href=\"/states\">介面狀態"),
        "nav must not link #states"
    );
}
