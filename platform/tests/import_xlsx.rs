//! V1 XLSX 匯入轉接器驗收（全能進料門戶）：
//! 1. 格式偵測：ZIP magic bytes → XLSX 路徑
//! 2. calamine 解析第一張工作表（inlineStr 中文＋浮點金額）
//! 3. 同一欄位契約驗證：1 錯誤列（幣別 XXX）＋ 2 合格列
//! 4. 兩階段批次語意：draft 狀態、未登帳

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const ADMIN_PASSWORD: &str = "xlsx-admin-pass";
const SAMPLE_XLSX: &[u8] = include_bytes!("fixtures/sample-import.xlsx");

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

async fn login(app: &axum::Router, _pool: &sqlx::SqlitePool) -> String {
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

async fn get_csrf(app: &axum::Router, cookie: &str, pool: &sqlx::SqlitePool) -> String {
    let request = Request::builder()
        .uri("/")
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .unwrap();
    let (status, body, _) = send(app.clone(), request).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("name=\"csrf_token\" value=\""),
        "dashboard must expose csrf token"
    );
    let session_id = cookie.strip_prefix("flaps_session=").expect("cookie shape");
    let (csrf,): (String,) = sqlx::query_as("SELECT csrf_token FROM sessions WHERE session_id = ?")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("session row");
    csrf
}

#[tokio::test]
async fn xlsx_import_end_to_end() {
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
    let cookie = login(&app, &pool).await;
    let csrf = get_csrf(&app, &cookie, &pool).await;

    // 手工 multipart（邊界固定；檔案以原始位元組內嵌）。
    let boundary = "XLSXTESTBOUNDARY";
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{csrf}\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"sample-import.xlsx\"\r\nContent-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(SAMPLE_XLSX);
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
        "xlsx import preview must PRG: {resp_body}"
    );
    let batch_id = location
        .as_deref()
        .and_then(|l| l.rsplit("batch=").next())
        .unwrap_or("")
        .to_string();
    assert!(!batch_id.is_empty(), "PRG must carry batch id");

    // 批次統計：3 列（1 錯誤＋2 合格）；狀態 draft（未登帳）。
    let (row_total, row_ok, status): (i64, i64, String) =
        sqlx::query_as("SELECT row_total, row_ok, status FROM import_batches WHERE batch_id = ?")
            .bind(&batch_id)
            .fetch_one(&pool)
            .await
            .expect("batch row");
    assert_eq!(row_total, 3, "header skipped, three data rows");
    assert_eq!(row_ok, 2, "two valid rows (float amounts normalized)");
    assert_eq!(status, "draft", "preview must not touch the ledger");

    // 錯誤列＝幣別 XXX（同一欄位契約 fail-closed）。
    let bad: Vec<(String,)> =
        sqlx::query_as("SELECT error_msg FROM import_rows WHERE batch_id = ? AND status = 'error'")
            .bind(&batch_id)
            .fetch_all(&pool)
            .await
            .expect("error rows");
    assert_eq!(bad.len(), 1, "exactly one error row");
    assert!(
        bad[0].0.contains("幣別不支援：XXX"),
        "error must name the unsupported currency"
    );

    // 合格列含浮點金額正規化（1500.5 → 150050 minor；99.99 → 9999 minor）。
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

    // 報表頁渲染批次檢視（含合格／錯誤標示）。
    let request = Request::builder()
        .uri(format!("/import?batch={batch_id}"))
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .unwrap();
    let (status, page, _) = send(app, request).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("合格"), "page shows ok rows");
    assert!(page.contains("錯誤"), "page shows error rows");
    assert!(!page.contains("<script"), "zero-JS contract");
}
