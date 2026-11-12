//! 零依賴結構化日誌（2026-11-12 規格補洞：可觀測性）。
//!
//! 為什麼需要：原設計使用 `tracing` facade 但**未安裝任何 subscriber**
//! → 5 處 `tracing::info!` 全數蒸發， production 無任何可診斷輸出
//! （SOTA 的反例：有 facade 沒有 sink）。新增 `tracing-subscriber` 需
//! 通過新依賴審查（AGENTS.md §3）且拉高建置成本；本模組以 std＋chrono
//! 實作同等可觀測性，並移除未使用的 `tracing` 相依（淨精實）。
//!
//! 契約：輸出至 stderr（不污染 HTTP body）；`AUXBRIDGE_LOG` 控制最低
//! 等級（error|warn|info|debug，預設 info）；每行格式
//! `2026-11-12T09:40:04.123+08:00 LEVEL target: message`。

use std::io::Write;
use std::sync::atomic::{AtomicU8, Ordering};

pub const ERROR: u8 = 1;
pub const WARN: u8 = 2;
pub const INFO: u8 = 3;
pub const DEBUG: u8 = 4;

static LEVEL: AtomicU8 = AtomicU8::new(INFO);

/// 由環境變數設定最低日誌等級（bin 啟動時呼叫一次）。
pub fn init() {
    let raw = std::env::var("AUXBRIDGE_LOG").unwrap_or_else(|_| "info".to_string());
    LEVEL.store(parse_level(&raw), Ordering::Relaxed);
    log(
        INFO,
        "obs",
        &format!(
            "logger initialised (level={})",
            level_name(parse_level(&raw))
        ),
    );
}

fn parse_level(raw: &str) -> u8 {
    match raw.trim().to_ascii_lowercase().as_str() {
        "error" => ERROR,
        "warn" | "warning" => WARN,
        "debug" | "trace" => DEBUG,
        _ => INFO,
    }
}

fn level_name(level: u8) -> &'static str {
    match level {
        ERROR => "ERROR",
        WARN => "WARN",
        DEBUG => "DEBUG",
        _ => "INFO",
    }
}

/// 是否應輸出該等級（供呼叫端跳過昂貴的訊息格式化）。
pub fn enabled(level: u8) -> bool {
    level <= LEVEL.load(Ordering::Relaxed)
}

/// 寫出一行日誌（stderr；寫入失敗靜默——日誌不得反噬主流程）。
pub fn log(level: u8, target: &str, message: &str) {
    if !enabled(level) {
        return;
    }
    let ts = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z");
    let mut err = std::io::stderr().lock();
    let _ = writeln!(err, "{ts} {} {target}: {message}", level_name(level));
}

/// 巨集：格式字串優先（與 `tracing::info!` 呼叫形狀相容），target 自動取
/// `module_path!()`（呼叫端零負擔、仍可分層過濾）。
#[macro_export]
macro_rules! obs_error {
    ($($arg:tt)*) => {
        $crate::obs::log($crate::obs::ERROR, module_path!(), &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! obs_warn {
    ($($arg:tt)*) => {
        $crate::obs::log($crate::obs::WARN, module_path!(), &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! obs_info {
    ($($arg:tt)*) => {
        $crate::obs::log($crate::obs::INFO, module_path!(), &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! obs_debug {
    ($($arg:tt)*) => {
        $crate::obs::log($crate::obs::DEBUG, module_path!(), &format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_parsing_is_fail_safe() {
        assert_eq!(parse_level("error"), ERROR);
        assert_eq!(parse_level("WARN"), WARN);
        assert_eq!(parse_level("debug"), DEBUG);
        assert_eq!(parse_level("trace"), DEBUG);
        assert_eq!(parse_level("nonsense"), INFO, "未知值回退 info");
        assert_eq!(parse_level(""), INFO);
    }

    #[test]
    fn log_respects_level_gate() {
        LEVEL.store(ERROR, Ordering::Relaxed);
        assert!(!enabled(WARN));
        assert!(enabled(ERROR));
        LEVEL.store(INFO, Ordering::Relaxed);
        assert!(enabled(INFO));
        assert!(!enabled(DEBUG));
    }
}
