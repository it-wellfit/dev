// Flaps ERP — 桌面殼（Tauri 2.0）
//
// D1（計畫 1793034142477）：殼載入平台伺服器 —— UI 單一事實來源為
// platform 伺服器（http://127.0.0.1:8090；零 JS 工作台）。本檔不虛構任何
// API 資料（治理 P7：原硬編碼常量已全數移除；契約目錄請見
// platform /admin/api/catalog，由 apihub registry 驅動）。
//
// 啟動行為：若 8090 埠無人監聽，嘗試啟動本機 platform 伺服器
// （執行檔路徑可用 FLAPS_SERVER_EXE 指定；預設與殼同目錄的 flaps-server.exe）。
// 伺服器 env（FLAPS_DB_URL／FLAPS_ADMIN_PASSWORD）由啟動者繼承。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;

use state::AppState;

/// 平台伺服器位址（殼與伺服器的唯一繫結點）。
const PLATFORM_URL: &str = "http://127.0.0.1:8090/";

fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            ensure_platform_server();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 冪等啟動：已監聽則不重複啟動；未監聽則 spawn 並等候埠就緒（最長約 6 秒）。
///
/// 失敗不恐慌（fail-open 於殼層）：WebView 會顯示連線錯誤頁，使用者可依
/// 提示先啟動 platform 伺服器。此為 v0.2 權宜；v0.3 評估殼內嵌 axum。
fn ensure_platform_server() {
    let host_port = "127.0.0.1:8090";

    if std::net::TcpStream::connect(host_port).is_ok() {
        tracing::info!("platform 伺服器已在線（{host_port}）");
        return;
    }

    let exe = std::env::var("FLAPS_SERVER_EXE")
        .unwrap_or_else(|_| "flaps-server.exe".to_string());
    let path = std::path::PathBuf::from(&exe);
    if !path.exists() {
        tracing::warn!(
            "platform 伺服器執行檔未找到：{exe}；請先建置（cargo build -p platform --bin flaps-server）或設定 FLAPS_SERVER_EXE"
        );
        return;
    }

    match std::process::Command::new(&path).env("FLAPS_PORT", "8090").spawn() {
        Ok(child) => {
            tracing::info!("platform 伺服器已啟動（pid {:?}），等候埠就緒…", child.id());
        }
        Err(e) => {
            tracing::error!("platform 伺服器啟動失敗：{e}");
            return;
        }
    }

    for _ in 0..60 {
        if std::net::TcpStream::connect(host_port).is_ok() {
            tracing::info!("platform 伺服器埠就緒（{host_port}）");
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    tracing::warn!("platform 伺服器埠未於 6 秒內就緒；WebView 可能顯示連線錯誤，請稍後重新整理");
}

/// 保留：殼端狀態鏡像（V2.8 型別參照；金額一律 i64 minor，INV-01）。
#[allow(dead_code)]
fn _state_mirror(_s: &AppState) {}
