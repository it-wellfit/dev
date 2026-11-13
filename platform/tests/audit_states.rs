#![forbid(unsafe_code)]
//! 稽核頁面八態與量測基礎設施整合測試（計畫 1793033761021 §16 T-P0.1~T-P0.3、A-2、D-1）。
//!
//! 驗收對照：
//! - §15 C-2：DB 故障時 /audit 與 /my-decisions 渲染**錯誤態**（八態互異；
//!   empty≠error），並提供重試連結。
//! - §15 C-1（migration 0008）：/my-decisions 的 performed_by 查詢走
//!   idx_audit_log_performed_by（EXPLAIN QUERY PLAN 適應度斷言）。
//! - §15 A-2：雙重認領行為文件化——SQLite 單寫者下後到者逐列失敗
//!   （NOT_PENDING），不產生雙寫；摘要事件如實記錄部分失敗。
//! - §15 D-1：稽核時間以 Asia/Taipei（UTC+08:00）呈現（伺服器端格式化）。

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use platform::api::{AppState, router};
use platform::auth;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Row, SqlitePool};
use tower::ServiceExt;

const TEST_ADMIN_PASSWORD: &str = "Admin-Test-Passw0rd!";
const CLERK_B_PASSWORD: &str = "Clerk-B-Passw0rd!";

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

fn post_form(uri: &str, cookie: &str, body: String) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, cookie)
        .body(Body::from(body))
        .expect("build form request")
}

async fn login_as(app: &Router, username: &str, password: &str) -> String {
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "username={username}&password={password}"
        )))
        .expect("login request");
    let (status, _, cookie) = send(app.clone(), request).await;
    assert_eq!(status, StatusCode::SEE_OTHER, "login must redirect (PRG)");
    cookie.expect("login must set a session cookie")
}

async fn login_admin(app: &Router) -> String {
    login_as(app, "admin", TEST_ADMIN_PASSWORD).await
}

/// 從頁面 HTML 取出本 session 的 CSRF 權杖。
async fn fetch_csrf_from(app: &Router, uri: &str, cookie: &str) -> String {
    let (status, body, _) = send(app.clone(), get(uri, Some(cookie))).await;
    assert_eq!(status, StatusCode::OK);
    let marker = "name=\"csrf_token\" value=\"";
    let start = body
        .find(marker)
        .unwrap_or_else(|| panic!("csrf token marker missing on {uri}"))
        + marker.len();
    let rest = &body[start..];
    let end = rest.find('"').expect("closing quote of csrf token");
    rest[..end].to_string()
}

// ── §15 C-2：DB 故障 → 錯誤態（empty≠error 八態互異） ──

#[tokio::test]
async fn my_decisions_renders_error_state_on_db_failure() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login_admin(&app).await;
    // 故障注入：稽核表不存在 → 查詢必然失敗（session/roles 表不受影響）。
    sqlx::query("DROP TABLE audit_log")
        .execute(&pool)
        .await
        .expect("inject db failure");

    let (status, body, _) = send(app.clone(), get("/my-decisions", Some(&cookie))).await;
    assert_eq!(status, StatusCode::OK, "degraded page still renders");
    assert!(
        body.contains("data-state-error"),
        "error state marker must render"
    );
    assert!(
        body.contains("暫時無法讀取"),
        "error copy must be present (eight-state vocabulary)"
    );
    assert!(
        body.contains("重試"),
        "error state must offer a next step (retry link)"
    );
    assert!(
        !body.contains("尚無您的稽核紀錄"),
        "empty-state copy must NOT appear on failure (states are mutually exclusive)"
    );
}

#[tokio::test]
async fn audit_page_renders_error_state_on_db_failure() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login_admin(&app).await;
    sqlx::query("DROP TABLE audit_log")
        .execute(&pool)
        .await
        .expect("inject db failure");

    let (status, body, _) = send(app.clone(), get("/audit", Some(&cookie))).await;
    assert_eq!(status, StatusCode::OK, "degraded page still renders");
    assert!(body.contains("data-state-error"), "error state marker");
    assert!(body.contains("暫時無法讀取"), "error copy");
    assert!(
        !body.contains("尚無稽核紀錄"),
        "empty-state copy must NOT appear on failure"
    );
}

#[tokio::test]
async fn audit_pages_keep_empty_state_on_success() {
    // 回歸防護：正常路徑（表在）渲染紀錄區且錯誤態不誤現。
    // 註：2026-11-12 認證事件入帳後（規格補洞），登入即產生稽核事件，
    // 「登入後空態」實務上不可達——空態保留給歷史/預事件資料庫。
    // 互斥性由 db_error 分支結構保證（同區塊 if/else）。
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login_admin(&app).await;

    let (status, body, _) = send(app.clone(), get("/my-decisions", Some(&cookie))).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("我的稽核紀錄"),
        "success path renders the records section"
    );
    // 判別用文案（data-state-error 標記亦存在於 studio_nav partial，字串過寬）。
    assert!(
        !body.contains("暫時無法讀取"),
        "no error-state copy on success"
    );

    let (status, body, _) = send(app, get("/audit", Some(&cookie))).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("稽核紀錄"), "success path renders records");
    assert!(
        !body.contains("暫時無法讀取"),
        "no error-state copy on success"
    );
}

// ── §15 C-1（migration 0008）：自查查詢的索引適應度 ──

#[tokio::test]
async fn self_query_is_covered_by_performed_by_index() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let _ = login_admin(&app).await;

    let rows = sqlx::query(
        "EXPLAIN QUERY PLAN SELECT log_id FROM audit_log \
         WHERE performed_by = ? ORDER BY performed_at DESC LIMIT 25 OFFSET 0",
    )
    .bind("user-admin")
    .fetch_all(&pool)
    .await
    .expect("explain query plan");
    let detail: String = rows
        .iter()
        .map(|r| r.try_get::<String, _>("detail").unwrap_or_default())
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        detail.contains("idx_audit_log_performed_by"),
        "self-audit query must be index-covered (plan: {detail})"
    );
}

// ── §15 A-2：雙重認領文件化（SQLite 單寫者序列化下的安全語義） ──

#[tokio::test]
async fn double_claim_fails_cleanly_and_audits_partial_outcome() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let admin_cookie = login_admin(&app).await;

    // 一筆待覆核列，created_by='seed'（與兩位裁決者皆不同 → 無自我裁決干擾）。
    sqlx::query(
        "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, status, created_by) \
         VALUES ('tx-claim-1', 'TX-CLAIM-1', '2026-11-01', 'double claim row', 'PENDING_REVIEW', 'seed')",
    )
    .execute(&pool)
    .await
    .expect("seed pending row");

    // 建立第二位裁決者（ADMIN 角色；與 admin 不同 user_id）。
    let admin_csrf = fetch_csrf_from(&app, "/", &admin_cookie).await;
    let form = format!(
        "csrf_token={admin_csrf}&username=clerk-b&password={CLERK_B_PASSWORD}&role_code=ADMIN"
    );
    let (status, _, _) = send(app.clone(), post_form("/admin/users", &admin_cookie, form)).await;
    assert_eq!(
        status,
        StatusCode::SEE_OTHER,
        "admin user creation must redirect (PRG)"
    );
    let clerk_cookie = login_as(&app, "clerk-b", CLERK_B_PASSWORD).await;

    // 第一位（admin）認領 → 成功。
    let admin_csrf = fetch_csrf_from(&app, "/", &admin_cookie).await;
    let (status, body1, _) = send(
        app.clone(),
        post_form(
            "/reviews/bulk",
            &admin_cookie,
            format!("csrf_token={admin_csrf}&decision=approve&tx_ids=tx-claim-1"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body1.contains("成功套用"), "first claim applies the row");

    // 第二位（clerk-b）後到認領 → 逐列失敗（NOT_PENDING），不雙寫。
    // clerk-b 的 csrf 取自 /（index.html 含批量表單；/my-decisions 唯讀無表單）。
    let clerk_csrf = fetch_csrf_from(&app, "/", &clerk_cookie).await;
    let (status, body2, _) = send(
        app.clone(),
        post_form(
            "/reviews/bulk",
            &clerk_cookie,
            format!("csrf_token={clerk_csrf}&decision=approve&tx_ids=tx-claim-1"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body2.contains("NOT_PENDING"),
        "second claim must fail with the explicit rule name"
    );

    // DB：列恰好前進一次；兩列摘要事件如實記錄（ok=1 與 fail=1 各一）。
    let posted: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM transactions WHERE transaction_id = 'tx-claim-1' AND status = 'POSTED'",
    )
    .fetch_one(&pool)
    .await
    .expect("count posted");
    assert_eq!(posted.0, 1, "row advances exactly once (no double-write)");

    let summaries: Vec<(String,)> =
        sqlx::query_as("SELECT event_data FROM audit_log WHERE event_type = 'REVIEW_BULK_SUMMARY'")
            .fetch_all(&pool)
            .await
            .expect("fetch summaries");
    assert_eq!(summaries.len(), 2, "one summary per batch POST");
    let mut ok_total = 0;
    let mut fail_total = 0;
    for (data,) in &summaries {
        let v: serde_json::Value = serde_json::from_str(data).expect("summary JSON");
        ok_total += v["ok_count"].as_i64().unwrap_or(0);
        fail_total += v["fail_count"].as_i64().unwrap_or(0);
    }
    assert_eq!(
        (ok_total, fail_total),
        (1, 1),
        "summaries record the partial outcome honestly (A-2/R-3)"
    );
}

// ── §15 D-1：稽核時間 Asia/Taipei 呈現 ──

#[tokio::test]
async fn audit_time_rendered_in_taipei_offset() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login_admin(&app).await;
    // UTC 16:30 → 台北時間為次日 00:30（跨日驗證格式化正確性）。
    sqlx::query(
        "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by, performed_at) \
         VALUES ('ev-tz-1', NULL, 'REVIEW_BULK', '{\"decision\":\"approve\"}', 'user-admin', '2026-11-11 16:30:00')",
    )
    .execute(&pool)
    .await
    .expect("seed audit row with explicit UTC time");

    let (status, body, _) = send(app, get("/my-decisions", Some(&cookie))).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("2026-11-12 00:30:00"),
        "time must render in Asia/Taipei (+08:00), crossing midnight correctly"
    );
    assert!(
        !body.contains("16:30:00"),
        "raw UTC time must not leak into the rendered view"
    );
}

// ── D-5 fail-open 邊界：不可信時間戳 → elapsed 記 unknown，裁決不受影響 ──

#[tokio::test]
async fn bulk_summary_elapsed_fails_open_on_implausible_timestamps() {
    let (app, pool) = in_memory_app().await;
    seed_admin(&pool).await;
    let cookie = login_admin(&app).await;

    let now_ms = chrono::Utc::now().timestamp_millis();
    let cases = [
        ("future-clock", now_ms + 3_600_000), // 未來時鐘（ delta<0 → unknown）
        ("ancient-page", now_ms - 90 * 86_400_000), // 頁面停留 90 天（>24h → unknown）
    ];
    for (i, (label, opened)) in cases.iter().enumerate() {
        let tx = format!("tx-open-{i}");
        sqlx::query(
            "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, status, created_by) \
             VALUES (?, ?, '2026-11-01', 'fail-open boundary row', 'PENDING_REVIEW', 'seed')",
        )
        .bind(&tx)
        .bind(format!("TX-OPEN-{i}"))
        .execute(&pool)
        .await
        .expect("seed pending row");
        let csrf = fetch_csrf_from(&app, "/", &cookie).await;
        let (status, _, _) = send(
            app.clone(),
            post_form(
                "/reviews/bulk",
                &cookie,
                format!("csrf_token={csrf}&decision=approve&tx_ids={tx}&queue_opened_at={opened}"),
            ),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "fail-open: implausible {label} timestamp must not block adjudication"
        );
    }

    let summaries: Vec<(String,)> =
        sqlx::query_as("SELECT event_data FROM audit_log WHERE event_type = 'REVIEW_BULK_SUMMARY'")
            .fetch_all(&pool)
            .await
            .expect("fetch summaries");
    assert_eq!(summaries.len(), 2, "one summary per POST");
    for (data,) in &summaries {
        let v: serde_json::Value = serde_json::from_str(data).expect("summary JSON");
        assert_eq!(
            v["elapsed_ms"], "unknown",
            "implausible timestamps must record unknown (fail-open bounds)"
        );
        assert_eq!(
            v["ok_count"], 1,
            "business outcome unaffected by measurement quality"
        );
    }
}
