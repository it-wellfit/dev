#![forbid(unsafe_code)]
//! 工具層錯誤型別。
//!
//! 所有訊息遵循「問題＋可能原因＋處理方式」格式，並以 MCP tool error
//! （`isError: true` 的 tool result）回報，而非 JSON-RPC 協定錯誤。

/// 工具執行錯誤。
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ToolError(String);

impl ToolError {
    /// 以完整訊息建立錯誤（呼叫端須確保含問題＋原因＋處理）。
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    /// 取出訊息文字。
    pub fn message(&self) -> &str {
        &self.0
    }
}

/// 工具層結果別名。
pub type ToolResult<T> = Result<T, ToolError>;
