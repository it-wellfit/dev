#![forbid(unsafe_code)]
//! 極簡命令列解析（避免額外依賴；對齊 plan D-12／D-14）。
//!
//! 支援：`--headless`、`--chrome <path>`、`--timeout-ms <n>`、`--help`。

use std::path::PathBuf;

/// 預設單次工具呼叫逾時（毫秒）。涵蓋 Chrome 啟動（chromiumoxide 預設 20s 上限）。
pub const DEFAULT_TIMEOUT_MS: u64 = 60_000;

/// 解析結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cli {
    /// 以指定參數啟動伺服器。
    Run(Args),
    /// 使用者要求說明。
    Help,
}

/// 伺服器啟動參數。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    /// 是否以 headless 模式啟動 Chrome（預設 headed，對齊 Kilo 內建預設）。
    pub headless: bool,
    /// Chrome 執行檔路徑（優先於 `CHROME` 環境變數；D-14）。
    pub chrome: Option<PathBuf>,
    /// 單次工具呼叫逾時（毫秒）。
    pub timeout_ms: u64,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            headless: false,
            chrome: None,
            timeout_ms: DEFAULT_TIMEOUT_MS,
        }
    }
}

/// 使用說明。
pub const HELP: &str = "\
browser-mcp — 純 Rust MCP 瀏覽器自動化伺服器（stdio）

用法：
  browser-mcp [--headless] [--chrome <path>] [--timeout-ms <ms>]

選項：
  --headless           以 headless 模式啟動 Chrome（預設為可見視窗）
  --chrome <path>      指定 Chrome 執行檔（未指定時依序使用 CHROME 環境變數與自動偵測）
  --timeout-ms <ms>    單次工具呼叫逾時，預設 60000
  --help, -h           顯示本說明
";

/// 解析命令列參數（不含程式名稱本身）。
pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Cli, String> {
    let mut out = Args::default();
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--headless" => out.headless = true,
            "--chrome" => {
                let value = iter.next().ok_or_else(|| {
                    "--chrome 缺少參數值：處理方式：改用 `--chrome <path>`。".to_string()
                })?;
                out.chrome = Some(PathBuf::from(value));
            }
            "--timeout-ms" => {
                let value = iter.next().ok_or_else(|| {
                    "--timeout-ms 缺少參數值：處理方式：改用 `--timeout-ms <毫秒>`。".to_string()
                })?;
                let parsed: u64 = value.parse().map_err(|_| {
                    format!(
                        "--timeout-ms 的值 `{value}` 不是有效數字：處理方式：提供正整數毫秒數，例如 60000。"
                    )
                })?;
                if parsed == 0 {
                    return Err(
                        "--timeout-ms 不可為 0：可能原因：輸入為零。處理方式：提供正整數毫秒數，例如 60000。"
                            .to_string(),
                    );
                }
                out.timeout_ms = parsed;
            }
            "--help" | "-h" => return Ok(Cli::Help),
            other => {
                return Err(format!(
                    "未知參數 `{other}`：處理方式：執行 `browser-mcp --help` 查看可用參數。"
                ));
            }
        }
    }
    Ok(Cli::Run(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(args: &[&str]) -> Result<Cli, String> {
        parse(args.iter().map(|s| (*s).to_string()))
    }

    #[test]
    fn defaults_are_headed_with_system_chrome() {
        let Cli::Run(args) = parse_str(&[]).expect("空參數應成功") else {
            panic!("應為 Run");
        };
        assert!(!args.headless);
        assert_eq!(args.chrome, None);
        assert_eq!(args.timeout_ms, DEFAULT_TIMEOUT_MS);
    }

    #[test]
    fn parses_all_flags() {
        let Cli::Run(args) = parse_str(&[
            "--headless",
            "--chrome",
            "C:\\chrome.exe",
            "--timeout-ms",
            "15000",
        ])
        .expect("完整參數應成功") else {
            panic!("應為 Run");
        };
        assert!(args.headless);
        assert_eq!(args.chrome, Some(PathBuf::from("C:\\chrome.exe")));
        assert_eq!(args.timeout_ms, 15000);
    }

    #[test]
    fn help_is_reported() {
        assert_eq!(parse_str(&["--help"]).expect("--help 應成功"), Cli::Help);
        assert_eq!(parse_str(&["-h"]).expect("-h 應成功"), Cli::Help);
    }

    #[test]
    fn rejects_unknown_flag() {
        let err = parse_str(&["--nope"]).expect_err("未知參數應失敗");
        assert!(err.contains("--nope"));
    }

    #[test]
    fn rejects_missing_and_invalid_values() {
        assert!(parse_str(&["--chrome"]).is_err());
        assert!(parse_str(&["--timeout-ms"]).is_err());
        assert!(parse_str(&["--timeout-ms", "abc"]).is_err());
        assert!(parse_str(&["--timeout-ms", "0"]).is_err());
    }
}
