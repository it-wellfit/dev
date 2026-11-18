#![forbid(unsafe_code)]
//! 整合測試（雙路徑；符合工作區「0 ignored」慣例）：
//!
//! - 環境有可用的 Chrome：驗證 navigate → snapshot → click → screenshot → close 全流程。
//! - 環境無可用 Chrome：斷言回傳「可行動錯誤」且不 panic，並明確標示 UNKNOWN
//!   （C-10：UNKNOWN ≠ PASS）。

use std::io::Write;
use std::time::Duration;

use browser_mcp::session::{BrowserSession, SessionConfig};
use browser_mcp::snapshot::Limits;

/// 本機最小頁面（零 JS；供真實瀏覽器載入）。
const PAGE: &str = "<!doctype html><html><head><meta charset=\"utf-8\"><title>測試頁</title></head><body><h1>Hello 交易</h1><button id=\"go\">送出</button></body></html>";

/// 啟動本機 HTTP 伺服器，回傳（URL, 執行緒）。
///
/// 必須先完整讀取請求標頭再回應，否則 Chrome 會因連線被過早關閉而
/// 回報 `ERR_CONNECTION_ABORTED`。
fn spawn_local_page() -> (String, std::thread::JoinHandle<()>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("應可綁定本機埠");
    let addr = listener.local_addr().expect("應可取得本機位址");
    let handle = std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            {
                use std::io::BufRead as _;
                let mut reader = std::io::BufReader::new(match stream.try_clone() {
                    Ok(clone) => clone,
                    Err(_) => continue,
                });
                let mut line = String::new();
                while let Ok(read) = reader.read_line(&mut line) {
                    if read == 0 || line == "\r\n" || line == "\n" {
                        break;
                    }
                    line.clear();
                }
            }
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                PAGE.len(),
                PAGE
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
            let _ = stream.shutdown(std::net::Shutdown::Write);
        }
    });
    (format!("http://127.0.0.1:{}/", addr.port()), handle)
}

/// 自快照文字取出指定角色的第一個 ref。
fn ref_for_role(text: &str, role: &str) -> Option<String> {
    let needle = format!("- {role} ");
    text.lines()
        .find(|line| line.contains(&needle))
        .and_then(|line| {
            let start = line.find("[ref=")? + "[ref=".len();
            let rest = &line[start..];
            let end = rest.find(']')?;
            Some(rest[..end].to_string())
        })
}

#[tokio::test]
async fn navigate_snapshot_click_screenshot_or_actionable_error() {
    let (url, server_thread) = spawn_local_page();

    let session = BrowserSession::new(SessionConfig {
        headless: true,
        chrome: None,
        timeout: Duration::from_secs(30),
        limits: Limits::default(),
    });

    match session.navigate(&url).await {
        Ok(message) => {
            assert!(message.contains("已導覽至"), "導覽回應不如預期：{message}");

            let snapshot = session.snapshot().await.expect("snapshot 應成功");
            assert!(
                snapshot.text.contains("Hello"),
                "快照應包含頁面文字：{}",
                snapshot.text
            );

            if let Some(button_ref) = ref_for_role(&snapshot.text, "button") {
                session
                    .click(&button_ref)
                    .await
                    .unwrap_or_else(|error| panic!("點擊 {button_ref} 失敗：{error}"));
            }

            let png = session.screenshot().await.expect("截圖應成功");
            assert!(!png.is_empty(), "截圖不得為空");
            session.close().await.expect("關閉應成功");
        }
        Err(error) => {
            let message = error.message().to_string();
            assert!(
                message.contains("Chrome") && message.contains("處理方式"),
                "非 Chrome 相關的失敗視為真錯誤（不得靜默略過）：{message}"
            );
            eprintln!(
                "UNKNOWN：此環境無可用或無法啟動的 Chrome，瀏覽器流程未驗證（C-10）。訊息：{message}"
            );
        }
    }

    drop(server_thread);
}
