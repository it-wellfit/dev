//! V3-B 互動式映射頁 E2E：未知標題 CSV → 映射待辦 → 映射頁 → 套用 → 草稿正確。

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const ADMIN_PASSWORD: &str = "map2-admin-pass";

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

async fn get_csrf(pool: &sqlx::SqlitePool, cookie: &str) -> String {
    let session_id = cookie
        .strip_prefix("auxbridge_session=")
        .expect("cookie shape");
    let (csrf,): (String,) = sqlx::query_as("SELECT csrf_token FROM sessions WHERE session_id = ?")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("session row");
    csrf
}

#[tokio::test]
async fn unknown_headers_route_to_mapping_page_then_apply() {
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

    // 未知標題（無任何同義詞命中）＋亂序。
    let csv = "欄位A,欄位B,欄位C,欄位D,欄位E,欄位F\r\n\
               2026-11-09,ACC-A,ACC-B,1500.5,TWD,測試列\r\n";

    let boundary = "UNKBOUNDARY";
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{csrf}\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"unk.csv\"\r\nContent-Type: text/csv\r\n\r\n"
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
        "unknown headers must route to the mapping page"
    );
    let loc = response
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    assert!(
        loc.starts_with("/import/mapping?batch="),
        "must redirect to the mapping page: {loc}"
    );
    let batch_id = loc
        .split("batch=")
        .nth(1)
        .unwrap_or("")
        .split('&')
        .next()
        .unwrap_or("")
        .to_string();

    // 映射頁：狀態 mapping、同義詞零命中（全「忽略」預選）。
    let request = Request::builder()
        .uri(format!("/import/mapping?batch={batch_id}"))
        .header(header::COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let (status, page) = {
        let r = app.clone().oneshot(request).await.expect("mapping page");
        let s = r.status();
        let b = axum::body::to_bytes(r.into_body(), 4 * 1_048_576)
            .await
            .expect("body");
        (s, String::from_utf8_lossy(&b).to_string())
    };
    assert_eq!(status, StatusCode::OK, "mapping page must render");
    assert!(page.contains("欄位映射"), "mapping page title");
    assert!(!page.contains("<script"), "zero-JS contract");

    // 套用映射：欄位A→date(0)、欄位B→debit(1)、欄位C→credit(2)、欄位D→amount(3)、
    // 欄位E→currency(4)、欄位F→description(5)。
    let apply_body = format!("csrf_token={csrf}&batch_id={batch_id}&f0=0&f1=1&f2=2&f3=3&f4=4&f5=5");
    let request = Request::builder()
        .method("POST")
        .uri("/import/mapping/apply")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, cookie.clone())
        .body(Body::from(apply_body))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("apply response");
    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "apply must PRG to the batch view"
    );

    // 批次已轉 draft：1 列合格、映射後欄位正確。
    let (row_total, row_ok, status): (i64, i64, String) =
        sqlx::query_as("SELECT row_total, row_ok, status FROM import_batches WHERE batch_id = ?")
            .bind(&batch_id)
            .fetch_one(&pool)
            .await
            .expect("batch row");
    assert_eq!(row_total, 2, "header + one data row");
    assert_eq!(row_ok, 1, "one valid row");
    assert_eq!(status, "draft", "batch promoted to draft");

    let (date, _amount): (String, i64) = sqlx::query_as(
        "SELECT json_extract(draft_json, '$.date'), json_extract(draft_json, '$.amount_minor') \
         FROM import_rows WHERE batch_id = ? AND status = 'ok' LIMIT 1",
    )
    .bind(&batch_id)
    .fetch_one(&pool)
    .await
    .expect("draft row");
    assert_eq!(date, "2026-11-09", "date mapped from 欄位A");
}
