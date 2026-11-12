//! E2E 執行期語義驗證（BankPay 為無條件編譯模組；原 feature 閘控已依
//! 架構收斂移除，`time` 已遷移至 `chrono`，無新增依賴）：
//! 1) 分頁：21 筆交易 → 儀表板「第 2 / 2 頁・共 21 筆」（M1 頁尺寸 20）
//! 2) 超頁夾限：?page=999 → 最後一頁（clamp_page）
//! 3) 設定持久化：POST /api/bankpay/config → erp_config 5 列（冪等 UPSERT）
//!    → 新 AppState（重啟等價）後由 erp_config 載入仍生效
//! 4) Session 跨「重啟」仍有效（DB-backed；未過期不被 purge 清除）

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const ADMIN_PASSWORD: &str = "it-persist-admin-pass";

async fn in_memory_pool() -> sqlx::SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory pool")
}

async fn send(app: axum::Router, request: Request<Body>) -> (StatusCode, String, Option<String>) {
    let response = app.oneshot(request).await.expect("router response");
    let status = response.status();
    let set_cookie = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|v| v.to_str().ok())
        .map(|v| v.split(';').next().unwrap_or("").to_string());
    let body = axum::body::to_bytes(response.into_body(), 4 * 1_048_576)
        .await
        .expect("body bytes");
    (
        status,
        String::from_utf8_lossy(&body).to_string(),
        set_cookie,
    )
}

async fn login(app: &axum::Router, pool: &sqlx::SqlitePool) -> (String, String) {
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "username=admin&password={ADMIN_PASSWORD}"
        )))
        .unwrap();
    let (status, _, cookie) = send(app.clone(), request).await;
    assert_eq!(status, StatusCode::SEE_OTHER, "login must PRG (303)");
    let cookie = cookie.expect("login must set a session cookie");
    let session_id = cookie
        .strip_prefix("auxbridge_session=")
        .expect("session cookie shape");
    let (csrf,): (String,) = sqlx::query_as("SELECT csrf_token FROM sessions WHERE session_id = ?")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("session row");
    (cookie, csrf)
}

#[tokio::test]
async fn dashboard_pagination_and_bankpay_config_survive_restart() {
    let pool = in_memory_pool().await;
    let state = platform::api::AppState::new(pool.clone())
        .await
        .expect("app state");
    let app = platform::api::router(state);

    let hash = platform::auth::hash_password(ADMIN_PASSWORD).expect("hash");
    sqlx::query(
        "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
         VALUES ('user-admin', 'admin', ?, 1)",
    )
    .bind(&hash)
    .execute(&pool)
    .await
    .expect("seed admin");
    let (cookie, csrf) = login(&app, &pool).await;

    // Seed 21 transactions through the real form endpoint (CSRF enforced).
    let today = chrono::Utc::now().date_naive();
    for i in 1..=21 {
        let body = format!(
            "csrf_token={csrf}&debit_account=ACC-A&credit_account=ACC-B&amount=100.{i:02}&currency=TWD&description=it-{i}&transaction_date={today}"
        );
        let request = Request::builder()
            .method("POST")
            .uri("/transactions")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::COOKIE, cookie.clone())
            .body(Body::from(body))
            .unwrap();
        let (status, resp, _) = send(app.clone(), request).await;
        assert_eq!(
            status,
            StatusCode::SEE_OTHER,
            "seed tx {i} must PRG: {resp}"
        );
    }

    // 1) Dashboard page 2 of 2 (M1: LIST_PAGE_SIZE = 20).
    let request = Request::builder()
        .uri("/?page=2")
        .header(header::COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let (status, body, _) = send(app.clone(), request).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("第 2 / 2 頁"),
        "dashboard must render page 2 / 2"
    );
    assert!(body.contains("共 21 筆"), "dashboard must count 21 entries");

    // 2) Past-the-end request clamps to the last page.
    let request = Request::builder()
        .uri("/?page=999")
        .header(header::COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let (status, body, _) = send(app.clone(), request).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("第 2 / 2 頁"), "page=999 must clamp to 2 / 2");

    // 3) Persist BankPay config through the API (ROLE_MANAGE enforced).
    let request = Request::builder()
        .method("POST")
        .uri("/api/bankpay/config")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, cookie.clone())
        .header("x-csrf-token", csrf.clone())
        .body(Body::from(
            r#"{"esun_mock_mode":false,"esun_url":"https://it.example/v2","customer_id":"CUST-IT","source_system_id":"SRC-IT"}"#,
        ))
        .unwrap();
    let (status, body, _) = send(app.clone(), request).await;
    assert_eq!(status, StatusCode::OK, "config POST must succeed: {body}");
    assert!(body.contains("https://it.example/v2"));

    let rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM erp_config WHERE config_category = 'bankpay'")
            .fetch_one(&pool)
            .await
            .expect("count persisted keys");
    // 2026-11-09: EsunConfig grew from 5 to 10 persisted keys (mTLS cert
    // fields: ca_cert_path, client_cert_path, client_key_path, + 2 more).
    // Cross-line stale-fixture fix (PHASE-R session) — count follows the
    // model, never the other way around.
    assert_eq!(rows, 10, "all ten bankpay keys must be persisted");

    // 3.5) API 資料管理平台（零 JS 管理主控台；目錄由 registry 驅動）。
    let catalog_req = Request::builder()
        .uri("/admin/api/catalog")
        .header(header::COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let (status, catalog_body, _) = send(app.clone(), catalog_req).await;
    assert_eq!(status, StatusCode::OK, "catalog must render");
    assert!(
        catalog_body.contains("/api/bankpay/export"),
        "catalog must list registry endpoints"
    );
    assert!(
        catalog_body.contains("在測試台檢視"),
        "catalog must link to the test bench"
    );

    let test_req = Request::builder()
        .uri("/admin/api/test?endpoint=/api/status")
        .header(header::COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let (status, test_body, _) = send(app.clone(), test_req).await;
    assert_eq!(status, StatusCode::OK, "test bench must render");
    assert!(test_body.contains("即時回應"), "live probe panel rendered");
    assert!(
        test_body.contains("modules"),
        "status probe payload embedded"
    );

    let form = format!(
        "csrf_token={csrf}&esun_mock_mode=1&esun_url=https%3A%2F%2Fit.example%2Fv3&source_system_id=SRC-FORM&customer_id=CUST-FORM&payer_account=PAY-FORM"
    );
    let update_req = Request::builder()
        .method("POST")
        .uri("/admin/api/config")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, cookie.clone())
        .body(Body::from(form))
        .unwrap();
    let (status, update_body, _) = send(app.clone(), update_req).await;
    assert_eq!(
        status,
        StatusCode::SEE_OTHER,
        "config form must PRG: {update_body}"
    );

    let logs_req = Request::builder()
        .uri("/admin/api/logs")
        .header(header::COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let (status, logs_body, _) = send(app.clone(), logs_req).await;
    assert_eq!(status, StatusCode::OK, "logs must render");
    assert!(
        logs_body.contains("BANKPAY_CONFIG_UPDATE"),
        "form update must be audited (fail-closed)"
    );

    // 4) "Restart" equivalence: a fresh AppState over the same store must
    //    authenticate the old cookie and load the persisted config.
    let state2 = platform::api::AppState::new(pool.clone())
        .await
        .expect("restarted app state");
    let app2 = platform::api::router(state2);
    let request = Request::builder()
        .uri("/api/bankpay/config")
        .header(header::COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let (status, body, _) = send(app2, request).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "session must survive restart (DB-backed): {body}"
    );
    assert!(
        body.contains("https://it.example/v3") && body.contains("CUST-FORM"),
        "form-updated config must be loaded after restart: {body}"
    );
    assert!(!body.contains("<script"), "zero-JS invariant holds");
}
