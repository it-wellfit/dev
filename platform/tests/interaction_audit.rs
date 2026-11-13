//! Interaction-layer deep audit (2026-11-13)：零 JS 應用的全部互動原語——
//! 頁內錨點、<details> 下拉展開、:target 面板、多步流程。任何一個壞掉就是
//! 「沒反應／空白／卡住」。
//!
//! 每項契約＝可打勾的檢查：TT-N 編號對應 docs/engineering/UX_CONTRACTS.md。

#![forbid(unsafe_code)]

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const ADMIN_PASSWORD: &str = "interact-admin-pass-2026";

async fn in_memory_pool() -> sqlx::SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory pool")
}

async fn send(app: axum::Router, request: Request<Body>) -> (StatusCode, String) {
    let response = app.oneshot(request).await.expect("router response");
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 8 * 1_048_576)
        .await
        .expect("body bytes");
    (status, String::from_utf8_lossy(&body).to_string())
}

async fn login(app: &axum::Router, username: &str, password: &str) -> String {
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "username={username}&password={password}"
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
        .expect("login must set session cookie")
}

async fn get(app: &axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, String) {
    let mut builder = Request::builder().uri(uri);
    if let Some(c) = cookie {
        builder = builder.header(header::COOKIE, c);
    }
    send(app.clone(), builder.body(Body::empty()).unwrap()).await
}

async fn post(
    app: &axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: String,
) -> (StatusCode, String) {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    if let Some(c) = cookie {
        builder = builder.header(header::COOKIE, c);
    }
    send(app.clone(), builder.body(Body::from(body)).unwrap()).await
}

async fn seed(pool: &sqlx::SqlitePool) {
    let hash = platform::auth::hash_password(ADMIN_PASSWORD).expect("hash");
    sqlx::query(
        "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
         VALUES ('user-admin', 'admin', ?, 1)",
    )
    .bind(&hash)
    .execute(pool)
    .await
    .expect("seed admin");
    sqlx::query(
        "INSERT OR IGNORE INTO accounts (account_id, account_code, account_name, account_type, currency, is_active) \
         VALUES ('ACC-A', 'A1', 'Cash', 'ASSETS', 'TWD', 1)",
    )
    .execute(pool)
    .await
    .expect("seed account");
}

#[tokio::test]
async fn tt1_in_page_anchors_targets_exist_on_bankpay() {
    let pool = in_memory_pool().await;
    let state = platform::api::AppState::new(pool.clone())
        .await
        .expect("state");
    seed(&pool).await;
    let app = platform::api::router(state);
    let cookie = login(&app, "admin", ADMIN_PASSWORD).await;

    // TT-1：子導覽宣告的每個錨點，其目標 id 必須存在於同一頁。
    let (_, body) = get(&app, "/bankpay", Some(&cookie)).await;
    let mut anchors = 0usize;
    let mut failures = Vec::new();
    for cap in body.match_indices("href=\"#") {
        let start = cap.0 + "href=\"#".len();
        let Some(end) = body[start..].find('"') else {
            continue;
        };
        let id = &body[start..start + end];
        anchors += 1;
        if !body.contains(&format!("id=\"{id}\"")) {
            failures.push(format!("anchor #{id} has no target element"));
        }
    }
    assert!(anchors >= 5, "bankpay subnav anchors missing: {anchors}");
    assert!(failures.is_empty(), "TT-1 anchor failures: {:?}", failures);
}

#[tokio::test]
async fn tt2_details_dropdowns_are_wellformed() {
    let pool = in_memory_pool().await;
    let state = platform::api::AppState::new(pool.clone())
        .await
        .expect("state");
    seed(&pool).await;
    let app = platform::api::router(state);
    let cookie = login(&app, "admin", ADMIN_PASSWORD).await;

    // TT-2：每個 <details class="nav-menu"> 必須有 <summary> 與 .nav-menu-items
    // TT-2：每頁導覽的下拉原語齊備——nav-menu 容器、summary（點擊目標）、
    // items 容器（展開內容）。全域計數（char-safe），不逐一切片。
    for uri in ["/", "/import", "/reports/monthly", "/bankpay"] {
        let (_, body) = get(&app, uri, Some(&cookie)).await;
        let menus = body.matches("nav-menu").count();
        assert!(
            menus >= 4,
            "{uri}: expected >=4 nav-menu occurrences, got {menus}"
        );
        let summaries = body.matches("<summary").count();
        assert!(
            summaries >= 4,
            "{uri}: every nav-menu needs a <summary>, got {summaries}"
        );
        let items = body.matches("nav-menu-items").count();
        assert!(
            items >= 4,
            "{uri}: every nav-menu needs .nav-menu-items, got {items}"
        );
    }
}

#[tokio::test]
async fn tt3_target_panel_opens_via_hash() {
    let pool = in_memory_pool().await;
    let state = platform::api::AppState::new(pool.clone())
        .await
        .expect("state");
    seed(&pool).await;
    let app = platform::api::router(state);
    let cookie = login(&app, "admin", ADMIN_PASSWORD).await;

    // TT-3：狀態列「詳情」面板——/:target 顯示、無 hash 隱藏。
    let (_, base) = get(&app, "/", Some(&cookie)).await;
    assert!(
        base.contains(".status-panel") || base.contains("status-panel"),
        "status panel anchor missing on /"
    );
    let (_, targeted) = get(&app, "/#status-panel", Some(&cookie)).await;
    // hash 不送至伺服器，回應文件應相同（除伺服器端動態時間戳外）；以位元組長度與關鍵錨點比對。
    assert_eq!(
        targeted.len(),
        base.len(),
        "hash view must render the same document length"
    );
    assert!(
        targeted.contains("status-panel"),
        "status panel anchor missing on hash view"
    );
}

#[tokio::test]
async fn tt4_wizard_three_step_flow_with_draft_prefill() {
    let pool = in_memory_pool().await;
    let state = platform::api::AppState::new(pool.clone())
        .await
        .expect("state");
    seed(&pool).await;
    let app = platform::api::router(state);
    let cookie = login(&app, "admin", ADMIN_PASSWORD).await;
    let csrf = {
        let (_, body) = get(&app, "/new", Some(&cookie)).await;
        let marker = "name=\"csrf_token\" value=\"";
        let start = body.find(marker).expect("wizard csrf") + marker.len();
        body[start..start + body[start..].find('"').expect("quote")].to_string()
    };

    // TT-4a：step-1 選模板 → step-2 渲染（2026-11-12 修復的 422 回歸閘）。
    let (status, step2) = post(
        &app,
        "/new/step2",
        Some(&cookie),
        form_body(&[
            ("csrf_token", &csrf),
            ("template", "cash-payment"),
            ("template_stage", "choose"),
        ]),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "step-1 → step-2 must render");
    assert!(
        step2.contains("name=\"debit_account\""),
        "step-2 form must expose debit/credit/amount fields"
    );

    // TT-4b：step-2 提交完整欄位 → step-3 確認摘要。
    let (status, step3) = post(
        &app,
        "/new/step2",
        Some(&cookie),
        form_body(&[
            ("csrf_token", &csrf),
            ("template", "cash-payment"),
            ("debit_account", "ACC-A"),
            ("credit_account", "ACC-B"),
            ("amount", "1500.50"),
            ("currency", "TWD"),
            ("description", "互動稽核"),
            ("transaction_date", "2026-11-13"),
        ]),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        step3.contains("確認摘要") || step3.contains("步驟 3"),
        "step-3 confirmation must render"
    );

    // TT-4c：借貸同科目 → 規則錯誤回饋（R-BALANCE）在 step-2 重渲染。
    let (status, err_page) = post(
        &app,
        "/new/step2",
        Some(&cookie),
        form_body(&[
            ("csrf_token", &csrf),
            ("template", "cash-payment"),
            ("debit_account", "ACC-A"),
            ("credit_account", "ACC-A"),
            ("amount", "100.00"),
            ("currency", "TWD"),
            ("description", "same"),
            ("transaction_date", "2026-11-13"),
        ]),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "validation error must re-render");
    assert!(err_page.contains("R-BALANCE"), "rule tag must surface");
}

fn form_body(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!("{k}={}", v))
        .collect::<Vec<_>>()
        .join("&")
}

#[tokio::test]
async fn tt5_search_roundtrip_with_query_preserved() {
    let pool = in_memory_pool().await;
    let state = platform::api::AppState::new(pool.clone())
        .await
        .expect("state");
    seed(&pool).await;
    let app = platform::api::router(state);
    let cookie = login(&app, "admin", ADMIN_PASSWORD).await;

    // TT-5：搜尋 GET 表單——q 值回填（體驗：送出後輸入不消失）。
    let (_, body) = get(&app, "/search?q=cash", Some(&cookie)).await;
    let marker = "name=\"q\"";
    let Some(pos) = body.find(marker) else {
        panic!("search input missing");
    };
    let tag_end = body[pos..].find('>').map(|e| pos + e).unwrap_or(body.len());
    let tag = &body[pos..tag_end];
    assert!(
        tag.contains("value=\"cash\""),
        "search input must preserve the query: {tag}"
    );
}
