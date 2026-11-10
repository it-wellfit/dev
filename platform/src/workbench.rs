#![forbid(unsafe_code)]
//! 工作流頁面（T3）：全文檢索。
//!
//! 受眾分層（IA 修正 2026-11-09）：本模組頁面屬操作者層（所有部門使用者）。
//! 稽核軌跡頁與科目分錄頁由並行流交付（`pages.rs` 的 /audit 與 /reports/ledger），
//! 本模組不重複實作（避免雙入口 IA 混亂）；技術層頁面見 `apihub.rs`。
//! 本模組不做網路探測、不虛構資料（治理 P7）。

use std::collections::HashMap;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Router};

use crate::api::AppState;
use crate::auth::Session;

// ---------------------------------------------------------------------------
// 全文檢索（Tier 1 操作者）
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchTemplate {
    pub acting_user: String,
    pub q: String,
    pub has_q: bool,
    pub results: Vec<SearchResultRow>,
}

pub struct SearchResultRow {
    pub doc_id: String,
    pub content: String,
}

/// GET /search?q=… — 全文檢索（Tantivy；空索引誠實回空態，不虛構結果）。
pub async fn search_page(
    State(state): State<AppState>,
    session: Extension<Session>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let q = query.get("q").cloned().unwrap_or_default();
    let trimmed = q.trim().to_string();
    let has_q = !trimmed.is_empty();
    let mut results: Vec<SearchResultRow> = Vec::new();
    if has_q {
        let docs = state.search.search(&trimmed).await;
        results = docs
            .into_iter()
            .take(50)
            .map(|r| SearchResultRow {
                doc_id: r.doc_id,
                content: r.content,
            })
            .collect();
    }
    render_or_500(SearchTemplate {
        acting_user: session.user_id.clone(),
        q,
        has_q,
        results,
    })
}

fn render_or_500<T: Template>(template: T) -> Response {
    match template.render() {
        Ok(body) => Html(body).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("頁面渲染失敗（可能原因：模板錯誤）；請聯絡管理員。詳情：{e}"),
        )
            .into_response(),
    }
}

/// HTML 群組路由（require_session 由 api.rs 統一掛載）。
pub fn router() -> Router<AppState> {
    Router::new().route("/search", get(search_page))
}
