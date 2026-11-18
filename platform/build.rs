//! Build metadata injection（計畫 1793782615513 §4.4.2）。
//!
//! 注入 `BUILD_COMMIT` 與 `BUILD_TIME`（unix epoch 秒）環境變數，供
//! `/api/health` 與 `/health`、`/ready` 端點回報部署溯源資訊（可觀測性 §8.2）。
//!
//! 設計：
//! - 零額外依賴（std + git CLI；不引入 chrono 至 build-dependencies）。
//! - **不輸出 `rerun-if-changed`**：cargo 預設在 package 內任一檔案變更時
//!   重跑本腳本，確保 commit SHA 不會因 HEAD 檔案未變（同 branch 新 commit
//!   改的是 refs/heads/<branch>）而過時——正確性優先於重建速度（git 查詢
//!   僅 ~10ms）。
//! - git 不在 PATH 或非 git 環境（CI tarball）→ 回退 "unknown"，永不失敗。

use std::process::Command;

fn main() {
    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let build_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    println!("cargo:rustc-env=BUILD_COMMIT={commit}");
    println!("cargo:rustc-env=BUILD_TIME={build_time}");
}
