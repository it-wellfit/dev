//! V3-A Email 附件遞迴進料驗收（D-EML1）：
//! .eml 上傳 → mail-parser 解析 → CSV 附件遞迴 → 獨立批次（sha256 逐檔）
//! → 映射 → 驗證 → PRG；無附件 → 明確訊息。

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const ADMIN_PASSWORD: &str = "eml-admin-pass";
const SAMPLE_EML: &[u8] = include_bytes!("fixtures/sample-import.eml");

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
async fn eml_csv_attachment_imported() {
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

    let boundary = "EMLUPLOADBOUNDARY";
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{csrf}\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"sample-import.eml\"\r\nContent-Type: message/rfc822\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(SAMPLE_EML);
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
        "eml import must PRG"
    );
    let batch_id = response
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|l| l.split("batch=").nth(1))
        .and_then(|s| s.split("&").next())
        .unwrap_or("")
        .to_string();
    assert!(!batch_id.is_empty(), "PRG must carry the first batch id");

    // 附件 CSV 經遞迴進料：2 列全合格（映射直通——標準標題）。
    let (row_total, row_ok, status): (i64, i64, String) =
        sqlx::query_as("SELECT row_total, row_ok, status FROM import_batches WHERE batch_id = ?")
            .bind(&batch_id)
            .fetch_one(&pool)
            .await
            .expect("batch row");
    assert_eq!(row_total, 2, "two data rows from the attachment");
    assert_eq!(row_ok, 2, "both rows valid");
    assert_eq!(status, "draft", "preview must not touch the ledger");

    // 附件內容正確（浮點金額正規化）。
    let amounts: Vec<(i64,)> = sqlx::query_as(
        "SELECT json_extract(draft_json, '$.amount_minor') FROM import_rows \
         WHERE batch_id = ? AND status = 'ok' ORDER BY row_no",
    )
    .bind(&batch_id)
    .fetch_all(&pool)
    .await
    .expect("draft amounts");
    assert_eq!(amounts.len(), 2);
    assert_eq!(amounts[0].0, 150_050, "1500.5 → 150050 minor");
    assert_eq!(amounts[1].0, 9_999, "99.99 → 9999 minor");
}
