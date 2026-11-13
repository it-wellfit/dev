//! V3-B 映射驗收：中文標題＋亂序欄位的 CSV → 同義詞自動對應 → 正確草稿。

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const ADMIN_PASSWORD: &str = "map-admin-pass";

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
async fn csv_chinese_headers_auto_mapped() {
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

    let session_id = cookie
        .strip_prefix("auxbridge_session=")
        .expect("cookie shape");
    let (csrf,): (String,) = sqlx::query_as("SELECT csrf_token FROM sessions WHERE session_id = ?")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .expect("session row");

    // 中文標題＋亂序：價稅合計/摘要/交易日期/幣別/借方/貸方。
    let csv = "價稅合計,摘要,交易日期,幣別,借方科目,貸方科目\r\n\
               1500.5,測試,2026-11-09,TWD,ACC-A,ACC-B\r\n\
               99.99,測試二,2026-11-09,TWD,ACC-C,ACC-D\r\n";

    let boundary = "MAPTESTBOUNDARY";
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{csrf}\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"ch.csv\"\r\nContent-Type: text/csv\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(csv.as_bytes());
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
        "chinese-header csv must auto-map and PRG"
    );
    let batch_id = response
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|l| l.rsplit("batch=").next())
        .unwrap_or("")
        .to_string();
    assert!(!batch_id.is_empty(), "PRG must carry batch id");

    let (row_total, row_ok): (i64, i64) =
        sqlx::query_as("SELECT row_total, row_ok FROM import_batches WHERE batch_id = ?")
            .bind(&batch_id)
            .fetch_one(&pool)
            .await
            .expect("batch row");
    assert_eq!(row_total, 2, "two data rows");
    assert_eq!(row_ok, 2, "both rows valid after mapping");

    // 映射後欄位落位：date 第一欄、amount 第四欄（D-MAP1 重排）。
    let first: (String, i64) = sqlx::query_as(
        "SELECT json_extract(draft_json, '$.date'), json_extract(draft_json, '$.amount_minor') \
         FROM import_rows WHERE batch_id = ? AND status = 'ok' ORDER BY row_no LIMIT 1",
    )
    .bind(&batch_id)
    .fetch_one(&pool)
    .await
    .expect("first draft");
    assert_eq!(first.0, "2026-11-09", "date mapped");
    assert_eq!(first.1, 150_050, "1500.5 → 150050 minor");

    // profile 已按簽名自動儲存（erp_config）。
    let profile: (String,) = sqlx::query_as(
        "SELECT config_value FROM erp_config WHERE config_category = 'ingest_profile' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("profile saved");
    assert!(
        profile.0.contains('0'),
        "profile json carries field index: {}",
        profile.0
    );
}
