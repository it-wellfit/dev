#![forbid(unsafe_code)]
//! MCP 交握測試：以記憶體內雙向串流驗證 `initialize` 與 `tools/list`。
//!
//! 本測試不需要 Chrome（工具清單不觸發瀏覽器啟動）。

use std::sync::Arc;
use std::time::Duration;

use browser_mcp::server::BrowserServer;
use browser_mcp::session::{BrowserSession, SessionConfig};
use browser_mcp::snapshot::Limits;
use rmcp::ServiceExt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

fn test_session() -> Arc<BrowserSession> {
    Arc::new(BrowserSession::new(SessionConfig {
        headless: true,
        chrome: None,
        timeout: Duration::from_millis(1_000),
        limits: Limits::default(),
    }))
}

async fn read_json_line<R>(reader: &mut BufReader<R>) -> serde_json::Value
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut line = String::new();
    let read = tokio::time::timeout(Duration::from_secs(10), reader.read_line(&mut line)).await;
    let read = read.expect("等待伺服器回應逾時").expect("讀取回應失敗");
    assert!(read > 0, "連線在收到回應前關閉");
    serde_json::from_str(line.trim()).expect("回應必須是純 JSON-RPC")
}

#[tokio::test]
async fn initialize_and_tools_list_over_duplex() {
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    let (server_read, server_write) = tokio::io::split(server_io);
    let (client_read, client_write) = tokio::io::split(client_io);

    let server = BrowserServer::new(test_session());
    let serve_task = tokio::spawn(async move {
        if let Ok(running) = server.serve((server_read, server_write)).await {
            let _ = running.waiting().await;
        }
    });

    let mut writer = client_write;
    let mut reader = BufReader::new(client_read);

    let initialize = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": { "name": "browser_mcp-test", "version": "0.0.1" }
        }
    });
    writer
        .write_all(format!("{initialize}\n").as_bytes())
        .await
        .expect("送出 initialize 失敗");
    writer.flush().await.expect("flush 失敗");

    let response = read_json_line(&mut reader).await;
    assert_eq!(response["id"], 1);
    assert_eq!(
        response["result"]["serverInfo"]["name"], "browser",
        "伺服器名稱應為 browser：{response}"
    );
    assert!(
        response["result"]["capabilities"]["tools"].is_object(),
        "應宣告 tools 能力：{response}"
    );

    // 舊式生命週期：initialize 後送 initialized 通知。
    let initialized = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    writer
        .write_all(format!("{initialized}\n").as_bytes())
        .await
        .expect("送出 initialized 失敗");
    writer.flush().await.expect("flush 失敗");

    let list = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });
    writer
        .write_all(format!("{list}\n").as_bytes())
        .await
        .expect("送出 tools/list 失敗");
    writer.flush().await.expect("flush 失敗");

    let tools_response = read_json_line(&mut reader).await;
    assert_eq!(tools_response["id"], 2);
    let tools = tools_response["result"]["tools"]
        .as_array()
        .expect("tools 應為陣列");
    assert_eq!(tools.len(), 11, "應有 11 個工具：{tools_response}");
    let names: Vec<&str> = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();
    for expected in [
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
    ] {
        assert!(names.contains(&expected), "缺少工具 {expected}：{names:?}");
    }

    drop(writer);
    serve_task.abort();
}
