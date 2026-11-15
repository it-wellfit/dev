#![forbid(unsafe_code)]
//! 互動教學（TT-9，2026-11-15）：引導式新手教學頁 `/help/tutorial`。
//!
//! 契約（`docs/engineering/UX_CONTRACTS.md` TT-9）：每一步驟必須帶
//! 「目標／怎麼做／成功判準／常見錯誤」四欄結構，並以真實頁面深連結收尾——
//! 每個深連結都由 `web_crawl` 全站連結圖（TT-7）與
//! `all_manifest_pages_render_under_contract` 自動驗證，教學連結不得指向 404。
//! 互動原語：步驟折疊用原生 `<details>`（TT-2 同族；零 JS），頁內快速跳轉
//! 用錨點（TT-1 同律：錨點必須命中同頁目標 id）。
//!
//! 為何獨立模組：`pages.rs` 為高頻併編輯區（多條並行工作線），
//! 本頁自成一檔以降低合併衝突面；handler 形狀與 `pages::help_page` 一致
//! （session/nav 由 `require_session` middleware 注入，無 I/O）。

use askama::Template;
use axum::Extension;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use crate::auth::Session;

#[derive(Template)]
#[template(path = "help_tutorial.html")]
pub struct TutorialTemplate {
    pub acting_user: String,
    pub nav: crate::nav::NavModel,
}

/// GET /help/tutorial — 互動教學（引導式新手七步，零 JavaScript）。
pub async fn tutorial_page(
    Extension(session): Extension<Session>,
    nav: Extension<crate::nav::NavModel>,
) -> Response {
    let template = TutorialTemplate {
        acting_user: session.username.clone(),
        nav: nav.0.clone(),
    };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("模板渲染失敗：{e}"),
        )
            .into_response(),
    }
}
