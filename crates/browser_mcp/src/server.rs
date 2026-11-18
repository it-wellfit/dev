#![forbid(unsafe_code)]
//! MCP 伺服器：以 `rmcp` 暴露瀏覽器工具（MCP 伺服器名 `browser`）。
//!
//! 錯誤語義（plan D-16）：工具執行失敗一律回 `isError: true` 的 tool result
//! （含「問題＋可能原因＋處理方式」），JSON-RPC 協定錯誤僅用於無法路由的要求。

use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ServerHandler, schemars, tool, tool_handler, tool_router};

use crate::error::ToolError;
use crate::session::BrowserSession;

/// `browser_navigate` 參數。
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NavigateArgs {
    /// 目標網址（僅支援 http／https／about／data 開頭）。
    pub url: String,
}

/// `browser_click` 參數。
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClickArgs {
    /// 來自最近一次 `browser_snapshot` 的元素 ref（例如 `e3`）。
    pub r#ref: String,
}

/// `browser_type_text` 參數。
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TypeArgs {
    /// 來自最近一次 `browser_snapshot` 的輸入元素 ref。
    pub r#ref: String,
    /// 要輸入的文字（不會清空既有內容；如需清空請先全選或改用表單重設）。
    pub text: String,
}

/// `browser_press_key` 參數。
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PressKeyArgs {
    /// 具名按鍵：Enter／Tab／Escape／Backspace／ArrowUp／ArrowDown／ArrowLeft／ArrowRight／Home／End／PageUp／PageDown／Space。
    pub key: String,
}

/// `browser_select_option` 參數。
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SelectOptionArgs {
    /// 來自最近一次 `browser_snapshot` 的 select 元素 ref。
    pub r#ref: String,
    /// 要設定的選項值（value 屬性）。
    pub value: String,
}

/// `browser_hover` 參數。
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HoverArgs {
    /// 來自最近一次 `browser_snapshot` 的元素 ref。
    pub r#ref: String,
}

/// `browser_wait_for` 參數（三者恰一）。
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WaitForArgs {
    /// 等待固定毫秒數（與 text／ref 三者恰一）。
    pub time_ms: Option<u64>,
    /// 等待頁面內文出現指定文字（與 time_ms／ref 三者恰一）。
    pub text: Option<String>,
    /// 等待指定 ref 存在於目前快照（與 time_ms／text 三者恰一）。
    pub r#ref: Option<String>,
}

/// 瀏覽器 MCP 伺服器。
#[derive(Clone)]
pub struct BrowserServer {
    session: Arc<BrowserSession>,
}

#[tool_router]
impl BrowserServer {
    /// 建立伺服器（共用同一瀏覽器工作階段）。
    pub fn new(session: Arc<BrowserSession>) -> Self {
        Self { session }
    }

    /// 導覽至指定網址並等待載入完成。
    #[tool(
        description = "以系統 Chrome 導覽至指定網址並等待載入完成。導覽後請重新呼叫 browser_snapshot 取得元素 ref。"
    )]
    async fn navigate(
        &self,
        Parameters(args): Parameters<NavigateArgs>,
    ) -> Result<CallToolResult, McpError> {
        match self.session.navigate(&args.url).await {
            Ok(message) => Ok(text_ok(message)),
            Err(error) => Ok(text_err(error)),
        }
    }

    /// 取得目前頁面的無障礙樹快照（含元素 ref）。
    #[tool(
        description = "取得目前頁面的無障礙樹文字快照；每個可操作元素附有 ref（如 e3），供 click／type_text／select_option／hover 使用。導覽後舊 ref 會失效。"
    )]
    async fn snapshot(&self) -> Result<CallToolResult, McpError> {
        match self.session.snapshot().await {
            Ok(snapshot) => Ok(text_ok(snapshot.text)),
            Err(error) => Ok(text_err(error)),
        }
    }

    /// 點擊指定 ref 的元素。
    #[tool(description = "點擊指定 ref 的元素（會先捲入視野）。")]
    async fn click(
        &self,
        Parameters(args): Parameters<ClickArgs>,
    ) -> Result<CallToolResult, McpError> {
        match self.session.click(&args.r#ref).await {
            Ok(message) => Ok(text_ok(message)),
            Err(error) => Ok(text_err(error)),
        }
    }

    /// 於指定 ref 的輸入元素輸入文字。
    #[tool(
        description = "於指定 ref 的輸入元素輸入文字（不會清空既有內容）。輸入後常需搭配 browser_press_key（如 Enter）提交。"
    )]
    async fn type_text(
        &self,
        Parameters(args): Parameters<TypeArgs>,
    ) -> Result<CallToolResult, McpError> {
        match self.session.type_text(&args.r#ref, &args.text).await {
            Ok(message) => Ok(text_ok(message)),
            Err(error) => Ok(text_err(error)),
        }
    }

    /// 發送具名按鍵。
    #[tool(description = "發送具名按鍵（Enter／Tab／Escape／方向鍵等）。")]
    async fn press_key(
        &self,
        Parameters(args): Parameters<PressKeyArgs>,
    ) -> Result<CallToolResult, McpError> {
        match self.session.press_key(&args.key).await {
            Ok(message) => Ok(text_ok(message)),
            Err(error) => Ok(text_err(error)),
        }
    }

    /// 設定下拉選項值。
    #[tool(description = "設定 select 元素的值並派發 change 事件。")]
    async fn select_option(
        &self,
        Parameters(args): Parameters<SelectOptionArgs>,
    ) -> Result<CallToolResult, McpError> {
        match self.session.select_option(&args.r#ref, &args.value).await {
            Ok(message) => Ok(text_ok(message)),
            Err(error) => Ok(text_err(error)),
        }
    }

    /// 將滑鼠移到指定 ref 元素上方。
    #[tool(description = "將滑鼠移動到指定 ref 元素的中心（觸發 hover 效果）。")]
    async fn hover(
        &self,
        Parameters(args): Parameters<HoverArgs>,
    ) -> Result<CallToolResult, McpError> {
        match self.session.hover(&args.r#ref).await {
            Ok(message) => Ok(text_ok(message)),
            Err(error) => Ok(text_err(error)),
        }
    }

    /// 截取目前畫面。
    #[tool(description = "截取目前頁面畫面並回傳 PNG 影像。")]
    async fn screenshot(&self) -> Result<CallToolResult, McpError> {
        match self.session.screenshot().await {
            Ok(bytes) => {
                let encoded = BASE64.encode(&bytes);
                let note = format!("畫面截圖（PNG，{} bytes）。", bytes.len());
                Ok(CallToolResult::success(vec![
                    ContentBlock::text(note),
                    ContentBlock::image(encoded, "image/png"),
                ]))
            }
            Err(error) => Ok(text_err(error)),
        }
    }

    /// 回到上一頁。
    #[tool(description = "返回瀏覽歷史的上一頁。返回後請重新呼叫 browser_snapshot 取得元素 ref。")]
    async fn go_back(&self) -> Result<CallToolResult, McpError> {
        match self.session.go_back().await {
            Ok(message) => Ok(text_ok(message)),
            Err(error) => Ok(text_err(error)),
        }
    }

    /// 顯式等待（無自動等待機制）。
    #[tool(
        description = "顯式等待：等待固定時間、等待頁面出現指定文字，或確認指定 ref 存在（三者恰一）。本伺服器無自動等待，操作後請視需要呼叫。"
    )]
    async fn wait_for(
        &self,
        Parameters(args): Parameters<WaitForArgs>,
    ) -> Result<CallToolResult, McpError> {
        match self
            .session
            .wait_for(args.time_ms, args.text.as_deref(), args.r#ref.as_deref())
            .await
        {
            Ok(message) => Ok(text_ok(message)),
            Err(error) => Ok(text_err(error)),
        }
    }

    /// 關閉瀏覽器並釋放資源。
    #[tool(description = "關閉瀏覽器並清除暫存 profile；之後再次呼叫任何工具會自動重新啟動。")]
    async fn close(&self) -> Result<CallToolResult, McpError> {
        match self.session.close().await {
            Ok(()) => Ok(text_ok("已關閉瀏覽器並清除暫存 profile。")),
            Err(error) => Ok(text_err(error)),
        }
    }
}

#[tool_handler(
    name = "browser",
    version = "0.2.0",
    instructions = "純 Rust 瀏覽器自動化（Chrome DevTools Protocol）：navigate → snapshot → click／type_text／press_key／select_option／hover → screenshot。元素以 snapshot 回傳的 ref（如 e3）操作；導覽或返回後 ref 會失效，需重新 snapshot。無自動等待，時序需要時請用 wait_for。"
)]
impl ServerHandler for BrowserServer {}

/// 成功結果（文字）。
fn text_ok(message: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(message.into())])
}

/// 工具層錯誤（D-16：`isError: true` 且訊息可見）。
fn text_err(error: ToolError) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(error.message().to_string())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::SessionConfig;
    use crate::snapshot::Limits;
    use std::time::Duration;

    fn server() -> BrowserServer {
        BrowserServer::new(Arc::new(BrowserSession::new(SessionConfig {
            headless: true,
            chrome: None,
            timeout: Duration::from_millis(1_000),
            limits: Limits::default(),
        })))
    }

    #[tokio::test]
    async fn exposes_eleven_tools_with_expected_names() {
        let tools = BrowserServer::tool_router()
            .list_all()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect::<Vec<_>>();
        let expected = [
            "navigate",
            "snapshot",
            "click",
            "type_text",
            "press_key",
            "select_option",
            "hover",
            "screenshot",
            "go_back",
            "wait_for",
            "close",
        ];
        assert_eq!(tools.len(), expected.len(), "工具數量應為 11：{tools:?}");
        for name in expected {
            assert!(tools.contains(&name.to_string()), "缺少工具 {name}");
        }
    }

    #[tokio::test]
    async fn wait_for_requires_exactly_one_condition() {
        let result = server()
            .wait_for(Parameters(WaitForArgs {
                time_ms: None,
                text: None,
                r#ref: None,
            }))
            .await
            .expect("工具呼叫本身應成功（錯誤以 isError 呈現）");
        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn click_with_unknown_ref_reports_actionable_error() {
        let result = server()
            .click(Parameters(ClickArgs {
                r#ref: "e999".to_string(),
            }))
            .await
            .expect("工具呼叫本身應成功");
        assert_eq!(result.is_error, Some(true));
        let text = result
            .content
            .iter()
            .filter_map(|block| block.as_text().map(|t| t.text.clone()))
            .collect::<String>();
        assert!(
            text.contains("snapshot"),
            "錯誤訊息應指引重新 snapshot：{text}"
        );
    }
}
