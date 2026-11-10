#![forbid(unsafe_code)]
//! memory_lint CLI — 規格見 `src/lib.rs` 模組說明（計畫 §9.1）。

use std::path::{Path, PathBuf};
use std::process::exit;

fn main() {
    let mut root: Option<String> = None;
    let mut out: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => match args.next() {
                Some(v) => root = Some(v),
                None => {
                    eprintln!("--root 需要值");
                    exit(2);
                }
            },
            "--out" => match args.next() {
                Some(v) => out = Some(v),
                None => {
                    eprintln!("--out 需要值");
                    exit(2);
                }
            },
            "-h" | "--help" => {
                println!("用法：memory_lint [--root <Dev 工作區根>] [--out <報告檔>]");
                exit(0);
            }
            other => {
                eprintln!("未知參數：{other}");
                exit(2);
            }
        }
    }

    let root_path = match root {
        Some(r) => PathBuf::from(r),
        None => Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(4)
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    match memory_lint::lint(&root_path) {
        Ok(o) => {
            for l in &o.lines {
                println!("{l}");
            }
            if let Some(p) = out {
                let _ = std::fs::write(p, format!("{}\n", o.lines.join("\n")));
            }
            exit(if o.fail_count > 0 { 1 } else { 0 });
        }
        Err(e) => {
            let msg = format!("FAIL [ENV] {}", e.0);
            println!("{msg}");
            if let Some(p) = out {
                let _ = std::fs::write(p, format!("{msg}\n"));
            }
            exit(2);
        }
    }
}
