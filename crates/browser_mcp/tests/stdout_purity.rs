#![forbid(unsafe_code)]
//! stdout 純淨冒煙測試（D-10）：實際執行 `browser-mcp` 二進位，確認
//! stdout 僅承載 JSON-RPC、無任何雜訊（日誌必須走 stderr）。

use std::io::{BufRead, BufReader, Write};
use std::process::{ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// 以 watchdog 讀取一行 stdout。
fn read_line_with_timeout(stdout: ChildStdout, timeout: Duration) -> String {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let result = reader.read_line(&mut line).map(|_| line);
        let _ = tx.send(result);
    });
    match rx.recv_timeout(timeout) {
        Ok(Ok(line)) => line,
        Ok(Err(error)) => panic!("讀取 stdout 失敗：{error}"),
        Err(_) => panic!("等待 stdout 逾時：伺服器可能未回應 initialize"),
    }
}

#[test]
fn binary_stdout_is_pure_json_rpc() {
    let exe = env!("CARGO_BIN_EXE_browser-mcp");
    let mut child = Command::new(exe)
        .arg("--headless")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("應可啟動 browser-mcp 二進位");

    let mut stdin = child.stdin.take().expect("stdin 應可用");
    let initialize = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","#,
        r#""capabilities":{},"clientInfo":{"name":"stdout-purity-test","version":"0.0.1"}}}"#,
        "\n"
    );
    stdin
        .write_all(initialize.as_bytes())
        .expect("送出 initialize 失敗");
    stdin.flush().expect("flush 失敗");

    let stdout = child.stdout.take().expect("stdout 應可用");
    let line = read_line_with_timeout(stdout, Duration::from_secs(20));
    let trimmed = line.trim();
    assert!(
        !trimmed.is_empty(),
        "stdout 第一行不得為空：可能有雜訊或未回應"
    );

    let value: serde_json::Value = serde_json::from_str(trimmed).unwrap_or_else(|error| {
        panic!("stdout 必須是純 JSON-RPC，實際內容：{trimmed:?}（{error}）")
    });
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["id"], 1);
    assert_eq!(
        value["result"]["serverInfo"]["name"], "browser",
        "伺服器名稱應為 browser：{value}"
    );

    // 關閉 stdin 觸發 EOF 結束，並確保不殘留子進程。
    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
}
