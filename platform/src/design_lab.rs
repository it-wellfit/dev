#![forbid(unsafe_code)]
//! 體驗室（Design Lab）——計畫 1793708660255 v2.2 §8 實作。
//!
//! 目的（回應「工具包不精緻、缺全端介面 UI/UX 觀念」）：把體驗知識從
//! 4+ 份文件收斂為**產品內零 JS 標本館**，同時作為驗收對照（R0–R9）、
//! 回歸基準與新人入門。規格 SSOT 文件為 `docs/design/EXPERIENCE_SPEC.md`。
//!
//! 十頁（全數 Gate=ROLE_MANAGE；操作者層不得見開發文件——受眾分層 IA）：
//! - `/design`             總覽：入口卡、驗收協議、使用規則
//! - `/design/contracts`   體驗契約 EC-1..EC-10＋全端介面鏈 L1–L6＋TT-1..TT-5
//! - `/design/components`  元件庫（≥24 元件 × 狀態）
//! - `/design/states`      元件級八態矩陣（全頁樣板 SSOT 仍在 `/states`）
//! - `/design/content`     microcopy 目錄（成功／錯誤／空／警告／確認）
//! - `/design/blueprints`  頁面藍圖（ASCII wireframe，零 JS）
//! - `/design/surfaces`    全端表面 S1–S9 與跨裝置矩陣
//! - `/design/matrices`    表面×契約、角色×任務×情境
//! - `/design/tokens`      設計 Token 數值與對比規則
//! - `/design/decisions`   體驗決策 log（八欄格式）
//!
//! 硬規則：`<script>`=0、無 inline style、僅 CSS 互動（`:hover/:focus-visible/
//! :target/details`）；全部使用既有 Design Token 類別（不發明新 class）。

use askama::Template;
use axum::Extension;
use axum::extract::State;
use axum::response::{IntoResponse, Response};

use crate::api::AppState;
use crate::auth::Session;

/// 模板渲染（渲染失敗＝程式錯誤；回 500，不吞錯）。
fn render<T: Template>(t: T) -> Response {
    match t.render() {
        Ok(html) => axum::response::Html(html).into_response(),
        Err(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "template render error",
        )
            .into_response(),
    }
}

/// 10 頁共用：ROLE_MANAGE 閘（Defense in Depth；導覽隱藏≠保護）。
async fn gate(state: &AppState, user_id: &str) -> Option<Response> {
    crate::pages::require_perm(state, user_id, "ROLE_MANAGE", "無權檢視體驗室").await
}

macro_rules! lab_page {
    ($tpl:literal, $struct_name:ident, $fn_name:ident) => {
        #[derive(Template)]
        #[template(path = $tpl)]
        pub struct $struct_name {
            pub acting_user: String,
            pub nav: crate::nav::NavModel,
            pub active: &'static str,
        }

        pub async fn $fn_name(
            State(state): State<AppState>,
            Extension(session): Extension<Session>,
            nav: Extension<crate::nav::NavModel>,
        ) -> Response {
            if let Some(denied) = gate(&state, &session.user_id).await {
                return denied;
            }
            render($struct_name {
                acting_user: session.user_id.clone(),
                nav: nav.0.clone(),
                active: stringify!($fn_name),
            })
        }
    };
}

lab_page!("design_lab.html", LabOverview, overview);
lab_page!("design_lab_contracts.html", LabContracts, contracts);
lab_page!("design_lab_components.html", LabComponents, components);
lab_page!("design_lab_states.html", LabStates, states);
lab_page!("design_lab_content.html", LabContent, content);
lab_page!("design_lab_blueprints.html", LabBlueprints, blueprints);
lab_page!("design_lab_surfaces.html", LabSurfaces, surfaces);
lab_page!("design_lab_matrices.html", LabMatrices, matrices);
lab_page!("design_lab_tokens.html", LabTokens, tokens);
lab_page!("design_lab_decisions.html", LabDecisions, decisions);
