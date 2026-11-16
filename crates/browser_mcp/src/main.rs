#![forbid(unsafe_code)]
//! browser-mcp 進入點：解析參數、初始化 stderr 日誌、啟動 stdio MCP 伺服器。
//!
//! D-10：stdout 僅承載 JSON-RPC；所有日誌一律寫入 stderr。

use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use browser_mcp::cli::{self, Cli};
use browser_mcp::server::BrowserServer;
use browser_mcp::session::{BrowserSession, SessionConfig};
use browser_mcp::snapshot::Limits;
use rmcp::ServiceExt;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    init_logging();

    let args = match cli::parse(std::env::args().skip(1)) {
        Ok(Cli::Help) => {
            print!("{}", cli::HELP);
            return ExitCode::SUCCESS;
        }
        Ok(Cli::Run(args)) => args,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    let session = Arc::new(BrowserSession::new(SessionConfig {
        headless: args.headless,
        chrome: args.chrome,
        timeout: Duration::from_millis(args.timeout_ms),
        limits: Limits::default(),
    }));
    let server = BrowserServer::new(session.clone());

    let running = match server.serve(rmcp::transport::stdio()).await {
        Ok(running) => running,
        Err(error) => {
            eprintln!("MCP 伺服器啟動失敗：{error}。處理方式：檢查 Kilo 客戶端日誌或回報問題。");
            return ExitCode::from(1);
        }
    };

    let _ = running.waiting().await;
    session.shutdown().await;
    ExitCode::SUCCESS
}

/// 初始化 tracing（一律輸出到 stderr，避免污染 stdout 的 JSON-RPC 串流）。
fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("browser_mcp=info,warn"));
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .try_init();
}
