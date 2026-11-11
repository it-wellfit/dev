//! V3-C DOCX 匯入轉接器驗收：
//! ZIP 容器（PK magic）→ word/document.xml → OOXML 表格抽取（w:tbl→w:tr→w:tc→w:t）
//! → 中文標題映射（同義詞）→ 2 列草稿（浮點金額正規化）。

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const ADMIN_PASSWORD: &str = "docx-admin-pass";
const SAMPLE_DOCX: &[u8] = include_bytes!("fixtures/sample-import.docx");

async fn in_memory_pool() -> sqlx::SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory pool")
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

#[tokio::test]
async fn docx_table_import_end_to_end() {
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

    let session_id = cookie.strip_prefix("flaps_session=").expect("cookie shape");
    let (csrf,): (String,) = sqlx::query_as("SELECT csrf_token FROM sessions WHERE session_id = ?")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .expect("session row");

    let boundary = "DOCXTESTBOUNDARY";
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{csrf}\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"sample-import.docx\"\r\nContent-Type: application/vnd.openxmlformats-officedocument.wordprocessingml.document\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(SAMPLE_DOCX);
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
    let response = app.clone().oneshot(request).await.expect("import response");
    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "docx import preview must PRG"
    );
    let batch_id = response
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|l| l.split("batch=").nth(1))
        .and_then(|s| s.split('&').next())
        .unwrap_or("")
        .to_string();
    assert!(!batch_id.is_empty(), "PRG must carry batch id");

    // 表格 3 列（1 標題＋2 資料）；映射（中文同義詞）後 2 列全合格。
    let (row_total, row_ok, status): (i64, i64, String) =
        sqlx::query_as("SELECT row_total, row_ok, status FROM import_batches WHERE batch_id = ?")
            .bind(&batch_id)
            .fetch_one(&pool)
            .await
            .expect("batch row");
    eprintln!("DEBUG batch: total={row_total} ok={row_ok} status={status}");
    let err_rows: Vec<(String,)> =
        sqlx::query_as("SELECT COALESCE(error_msg, '') FROM import_rows WHERE batch_id = ?")
            .bind(&batch_id)
            .fetch_all(&pool)
            .await
            .expect("error rows query");
    for (msg,) in &err_rows {
        eprintln!("DEBUG row: {msg}");
    }
    assert_eq!(row_total, 3, "header + two data rows");
    assert_eq!(row_ok, 2, "two valid rows after mapping");
    assert_eq!(status, "draft", "preview must not touch the ledger");

    // 映射後欄位落位：date 第一欄、amount 第四欄（重排正確）。
    let first: (String, i64) = sqlx::query_as(
        "SELECT json_extract(draft_json, '$.date'), json_extract(draft_json, '$.amount_minor') \
         FROM import_rows WHERE batch_id = ? AND status = 'ok' ORDER BY row_no LIMIT 1",
    )
    .bind(&batch_id)
    .fetch_one(&pool)
    .await
    .expect("first draft");
    assert_eq!(first.0, "2026-11-09", "date mapped from Chinese header");
    assert_eq!(first.1, 150_050, "1500.5 → 150050 minor");

    // 報表頁渲染（零 JS）。
    let request = Request::builder()
        .uri(format!("/import?batch={batch_id}"))
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .unwrap();
    let (status, page) = send(app, request).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("合格"), "page shows ok rows");
    assert!(!page.contains("<script"), "zero-JS contract");
}

async fn send(app: axum::Router, request: Request<Body>) -> (StatusCode, String) {
    let response = app.oneshot(request).await.expect("router response");
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 4 * 1_048_576)
        .await
        .expect("body bytes");
    (status, String::from_utf8_lossy(&body).to_string())
}
