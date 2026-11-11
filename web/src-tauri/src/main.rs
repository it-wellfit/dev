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
use tauri::Manager;

/// 平台伺服器位址（殼與伺服器的唯一繫結點）。
#[allow(dead_code)] // redundant by design: URL enforced via tauri.conf.json window.url; kept as documented single bind point
const PLATFORM_URL: &str = "http://127.0.0.1:8090/";

fn main() {
    tauri::Builder::default()
        // W10（計畫 1793033568961）：自動更新鏈——minisign 簽章驗證由
        // tauri.conf.json plugins.updater.pubkey 鎖定（不可停用）；
        // Windows 走 NSIS passive 靜默更新；endpoint 指向內部 feed
        // （https://updates.flaps.internal/erp/latest.json）。
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            ensure_platform_server();
            // R4（2026-11-10）：就緒後才顯示主視窗（visible:false 於 conf）；
            // 失敗路徑仍 show（WebView 顯示連線指引，不困死隱形視窗）。
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            } else {
                tracing::error!("找不到主視窗（label=main）；請檢查 tauri.conf.json");
            }
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

    // 解析序（計畫 §9 P1-1；smoke 2026-11-10 證實相對路徑會沿 PATH 誤抓 dev 二進位）：
    // FLAPS_SERVER_EXE → exe 同目錄（NSIS 落位）→ exe 目錄 binaries/（dev sidecar）。
    let path = resolve_server_exe();
    let Some(path) = path else {
        tracing::warn!(
            "platform 伺服器執行檔未找到（已查 FLAPS_SERVER_EXE、exe 同目錄與 binaries/）；\
             請先建置（cargo build -p platform --bin flaps-server）或設定 FLAPS_SERVER_EXE"
        );
        return;
    };

    // DB 位置不隨 CWD 漂移（smoke 2026-11-10 證實 CWD 相對路徑會撞上工作區舊 DB 的
    // VersionMismatch）：以伺服器所在目錄為 CWD，並預建 data/（fail-soft；不可寫時
    // 伺服器自身 fail-closed 回報）。
    let mut cmd = std::process::Command::new(&path);
    cmd.env("FLAPS_PORT", "8090");
    if let Some(dir) = path.parent() {
        cmd.current_dir(dir);
        let _ = std::fs::create_dir_all(dir.join("data"));
    }
    match cmd.spawn() {
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

/// 伺服器執行檔解析（絕對路徑優先；杜絕 PATH 誤抓）。
fn resolve_server_exe() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("FLAPS_SERVER_EXE") {
        let path = std::path::PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
        tracing::warn!("FLAPS_SERVER_EXE 指向的檔案不存在，續查預設位置");
    }
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(std::path::Path::to_path_buf));
    if let Some(dir) = exe_dir {
        let installed = dir.join("flaps-server.exe");
        if installed.exists() {
            return Some(installed);
        }
        let dev_sidecar = dir
            .join("binaries")
            .join("flaps-server-x86_64-pc-windows-msvc.exe");
        if dev_sidecar.exists() {
            return Some(dev_sidecar);
        }
    }
    None
}
