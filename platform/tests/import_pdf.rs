//! V2 PDF 文字層匯入轉接器驗收（全能進料門戶）：
//! 1. 格式偵測：%PDF magic → PDF 路徑（pdf-extract 記憶體擷取）
//! 2. TAB 分隔行 → 同一 6 欄契約（1 合格＋1 幣別錯誤列）
//! 3. 兩階段批次語意：draft、未登帳；錯誤訊息指名幣別

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const ADMIN_PASSWORD: &str = "pdf-admin-pass";
const SAMPLE_PDF: &[u8] = include_bytes!("fixtures/sample-import.pdf");

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
    let location = response
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let body = axum::body::to_bytes(response.into_body(), 4 * 1_048_576)
        .await
        .expect("body bytes");
    (status, String::from_utf8_lossy(&body).to_string(), location)
}

async fn login(app: &axum::Router) -> String {
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "username=admin&password={ADMIN_PASSWORD}"
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
        .expect("session cookie")
}

async fn get_csrf(pool: &sqlx::SqlitePool, cookie: &str) -> String {
    let session_id = cookie.strip_prefix("flaps_session=").expect("cookie shape");
    let (csrf,): (String,) = sqlx::query_as("SELECT csrf_token FROM sessions WHERE session_id = ?")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("session row");
    csrf
}

#[tokio::test]
async fn pdf_import_end_to_end() {
    let pool = in_memory_pool().await;
    let state = platform::api::AppState::new(pool.clone())
        .await
        .expect("app state");
    let app = platform::api::router(state);

    let admin_hash = platform::auth::hash_password(ADMIN_PASSWORD).expect("hash");
    sqlx::query(
        "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
         VALUES ('user-admin', 'admin', ?, 1)",
    )
    .bind(&admin_hash)
    .execute(&pool)
    .await
    .expect("seed admin");
    let cookie = login(&app).await;
    let csrf = get_csrf(&pool, &cookie).await;

    let boundary = "PDFTESTBOUNDARY";
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{csrf}\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"sample-import.pdf\"\r\nContent-Type: application/pdf\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(SAMPLE_PDF);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let request = Request::builder()
        .method("POST")
        .uri("/import/preview")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header(header::COOKIE, cookie.clone())
        .body(Body::from(body))
        .unwrap();
    let (status, resp_body, location) = send(app.clone(), request).await;
    assert_eq!(
        status,
        StatusCode::SEE_OTHER,
        "pdf import preview must PRG: {resp_body}"
    );
    let batch_id = location
        .as_deref()
        .and_then(|l| l.rsplit("batch=").next())
        .unwrap_or("")
        .to_string();
    assert!(!batch_id.is_empty(), "PRG must carry batch id");

    let (row_total, row_ok, status): (i64, i64, String) =
        sqlx::query_as("SELECT row_total, row_ok, status FROM import_batches WHERE batch_id = ?")
            .bind(&batch_id)
            .fetch_one(&pool)
            .await
            .expect("batch row");
    assert_eq!(row_total, 2, "header skipped, two data rows");
    assert_eq!(row_ok, 1, "one valid row (tab-separated fields)");
    assert_eq!(status, "draft", "preview must not touch the ledger");

    let bad: Vec<(String,)> =
        sqlx::query_as("SELECT error_msg FROM import_rows WHERE batch_id = ? AND status = 'error'")
            .bind(&batch_id)
            .fetch_all(&pool)
            .await
            .expect("error rows");
    assert_eq!(bad.len(), 1, "exactly one error row");
    assert!(
        bad[0].0.contains("幣別不支援：XXX"),
        "error must name the unsupported currency: {}",
        bad[0].0
    );

    let amounts: Vec<(i64,)> = sqlx::query_as(
        "SELECT json_extract(draft_json, '$.amount_minor') FROM import_rows \
         WHERE batch_id = ? AND status = 'ok'",
    )
    .bind(&batch_id)
    .fetch_all(&pool)
    .await
    .expect("draft amounts");
    assert_eq!(amounts.len(), 1);
    assert_eq!(amounts[0].0, 150_050, "1500.5 → 150050 minor");

    let request = Request::builder()
        .uri(format!("/import?batch={batch_id}"))
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .unwrap();
    let (status, page, _) = send(app, request).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("合格"), "page shows ok rows");
    assert!(!page.contains("<script"), "zero-JS contract");
}
