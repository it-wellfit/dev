#![forbid(unsafe_code)]
//! browser_mcp — 純 Rust 的 MCP 瀏覽器自動化伺服器。
//!
//! 以 `rmcp`（官方 Rust MCP SDK）提供 stdio MCP 介面，並以 `chromiumoxide`
//! 透過 Chrome DevTools Protocol 驅動系統 Chrome；全程不依賴 Node.js／npm。
//!
//! 設計約束（見 plan D-10／D-11）：stdout 僅承載 JSON-RPC，日誌一律 stderr；
//! 工具路徑不得 panic，錯誤一律映射為 MCP tool error。

pub mod cli;
pub mod error;
pub mod server;
pub mod session;
pub mod snapshot;
