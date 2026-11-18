#![forbid(unsafe_code)]
//! XP-003 匯出抽驗契約（2026-11-17；整合完成計畫 1794810000000 III-3）。
//!
//! 三個契約：
//! 1. `Content-Disposition` 檔名＝報表名＋期間（`monthly-YYYY-MM.csv`）。
//! 2. CSV 欄序穩定（`account_id,debit_minor,credit_minor,net_minor`）。
//! 3. 金額原樣＝整數 minor（與 DB 一致；禁止浮點格式化——INV-01 同族）。
//!
//! 資料以產品 API 自身播種（POST /transactions 正確 CSRF → PRG），
//! 使測試同時覆蓋「登帳→匯出」的真實鏈路。

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

async fn send(
    router: Router,
    request: Request<Body>,
) -> (StatusCode, axum::http::HeaderMap, String) {
    let response = router.oneshot(request).await.expect("router response");
    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), 1_048_576)
        .await
        .expect("body bytes");
    (status, headers, String::from_utf8_lossy(&body).to_string())
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
    let (status, headers, _) = send(app.clone(), request).await;
    assert_eq!(status, StatusCode::SEE_OTHER, "login must redirect (PRG)");
    headers
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|v| v.to_str().ok())
        .map(|v| v.split(';').next().unwrap_or("").to_string())
        .expect("login must set session cookie")
}

async fn csrf_of(app: &Router, cookie: &str, uri: &str) -> String {
    let (status, _, body) = send(app.clone(), get(uri, Some(cookie))).await;
    assert_eq!(status, StatusCode::OK, "{uri} must render");
    let marker = "name=\"csrf_token\" value=\"";
    let start = body
        .find(marker)
        .unwrap_or_else(|| panic!("csrf_token missing on {uri}"));
    let rest = &body[start + marker.len()..];
    rest[..rest.find('"').expect("csrf close quote")].to_string()
}

#[tokio::test]
async fn monthly_csv_export_contract() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    for (id, code, name, kind) in [
        ("ACC-DEBIT", "D1", "現金", "ASSETS"),
        ("ACC-CREDIT", "C1", "收入", "REVENUE"),
    ] {
        sqlx::query(
            "INSERT OR IGNORE INTO accounts (account_id, account_code, account_name, account_type, currency, is_active) \
             VALUES (?, ?, ?, ?, 'TWD', 1)",
        )
        .bind(id)
        .bind(code)
        .bind(name)
        .bind(kind)
        .execute(&pool)
        .await
        .expect("seed account");
    }
    let cookie = login(&app).await;
    let csrf = csrf_of(&app, &cookie, "/new").await;

    // 登帳：1,500.50 TWD @ 2026-10-15（金額含小數，驗證 minor 整數化）。
    let form = format!(
        "csrf_token={csrf}&debit_account=ACC-DEBIT&credit_account=ACC-CREDIT\
         &amount=1500.50&currency=TWD&description=export-pin&transaction_date=2026-10-15"
    );
    let (status, _, _) = send(app.clone(), post("/transactions", Some(&cookie), &form)).await;
    assert_eq!(status, StatusCode::SEE_OTHER, "posting must PRG");

    // 匯出：檔名（報表名＋期間）＋欄序＋金額原樣。
    let (status, headers, body) =
        send(app, get("/reports/monthly.csv?ym=2026-10", Some(&cookie))).await;
    assert_eq!(status, StatusCode::OK, "export must render");

    let cd = headers
        .get(header::CONTENT_DISPOSITION)
        .and_then(|v| v.to_str().ok())
        .expect("Content-Disposition must exist")
        .to_string();
    assert_eq!(
        cd, "attachment; filename=\"monthly-2026-10.csv\"",
        "filename must carry report name + period (XP-003 #1)"
    );

    let ct = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(ct.contains("text/csv"), "content-type {ct}");

    assert!(
        body.starts_with("account_id,debit_minor,credit_minor,net_minor\n"),
        "column order must be stable (XP-003 #2); got: {body}"
    );
    assert!(
        body.contains("ACC-DEBIT,150050,0,150050"),
        "debit row must carry integer minor as-is; got: {body}"
    );
    assert!(
        body.contains("ACC-CREDIT,0,150050,-150050"),
        "credit row must carry signed integer minor; got: {body}"
    );
    assert!(
        !body.contains("1500.5") && !body.contains("1500.50,"),
        "no float formatting may leak into amounts (XP-003 #3); got: {body}"
    );

    // 對帳 DB：CSV 值必須等於 transaction_items 聚合（金額原樣＝與 DB 一致）。
    let (debit_minor,): (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(amount_minor), 0) FROM transaction_items \
         WHERE account_id = 'ACC-DEBIT' AND item_type = 'DEBIT'",
    )
    .fetch_one(&pool)
    .await
    .expect("query debit total");
    assert_eq!(debit_minor, 150_050, "DB debit minor must match CSV");
    let (credit_minor,): (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(amount_minor), 0) FROM transaction_items \
         WHERE account_id = 'ACC-CREDIT' AND item_type = 'CREDIT'",
    )
    .fetch_one(&pool)
    .await
    .expect("query credit total");
    assert_eq!(credit_minor, 150_050, "DB credit minor must match CSV");
}
