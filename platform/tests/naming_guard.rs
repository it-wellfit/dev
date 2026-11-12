//! 命名守衛（2026-11-11 使用者裁定，docs/standards/NAMING_AND_BOUNDARY_DECREE.md）。
//!
//! 原廠產品 FlapsErp（中文名「輔翼ERP」）與自建平台 AuxBridge（整合工作台）
//! 必須身份分離：自建活躍面（模板、原始碼、資產、組態）不得出現原廠標記，
//! 防止設錯、測錯、誤改原廠內容失去保固。
//!
//! 範圍（活躍面 only）：`platform/templates`、`platform/src`、`platform/tests`、
//! `web/index.html`、`web/assets/*.css`、`web/src-tauri/tauri.conf.json`、
//! `platform/Cargo.toml`、`core/Cargo.toml`。
//! 刻意不在範圍（歷史紀錄／串接目標語境，見 decree）：`.kilo/`、`knowledge/`、
//! `web/legacy-1420.html`（歷史快照）、`docs/research/FlapsErp*`（原廠文檔）、
//! `crates/flaps_bridge`（橋接 crate 以串接目標命名——正確語義）、CHANGELOG。

use std::fs;
use std::path::{Path, PathBuf};

/// 自建活躍面禁用之原廠／舊名標記（bytes-level 檢查，避免編碼誤差）。
const BANNED: &[&str] = &[
    "Flaps ERP",              // 舊品牌（自我標記）
    "輔翼工作台",             // 原廠中文名「輔翼」污染自建品牌
    "flaps-server",           // 舊 bin 名
    "flaps_core",             // 舊 crate 名（已更名 auxbridge_core）
    "flaps_session",          // 舊 cookie 名（已更名 auxbridge_session）
    "FLAPS_",                 // 舊 env 前綴（已統一 AUXBRIDGE_）
    "demo-flaps.db",          // 舊 demo DB 名
    "updates.flaps.internal", // 舊 updater endpoint
];

const PLATFORM_DIR: &str = env!("CARGO_MANIFEST_DIR"); // .../Dev/platform

fn collect(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, exts, out);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if exts.contains(&ext) {
                out.push(path);
            }
        }
    }
}

fn scan(tokens_banned: &[&str], label: &str) {
    let root = Path::new(PLATFORM_DIR);
    let mut files: Vec<PathBuf> = Vec::new();
    collect(&root.join("templates"), &["html"], &mut files);
    collect(&root.join("src"), &["rs"], &mut files);
    collect(&root.join("tests"), &["rs"], &mut files);
    // web 側（相對 platform/）
    let ws = root.parent().expect("workspace root");
    files.push(ws.join("web").join("index.html"));
    collect(&ws.join("web").join("assets"), &["css"], &mut files);
    files.push(ws.join("web").join("src-tauri").join("tauri.conf.json"));
    files.push(ws.join("platform").join("Cargo.toml"));
    files.push(ws.join("core").join("Cargo.toml"));

    let mut violations: Vec<String> = Vec::new();
    let guard_self = root.join("tests").join("naming_guard.rs");
    for f in &files {
        if *f == guard_self {
            continue; // 守衛檔本身含禁用字串清單（定義處），排除自指。
        }
        let Ok(text) = fs::read(f) else { continue };
        for tok in tokens_banned {
            if text.windows(tok.len()).any(|w| w == tok.as_bytes()) {
                violations.push(format!("{} contains banned token {:?}", f.display(), tok));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "{label}: 自建活躍面不得含原廠標記（見 NAMING_AND_BOUNDARY_DECREE）:\n{}",
        violations.join("\n")
    );
}

/// 舊 bin 檔不得復活（平行工作線防呆：新 bin 為 auxbridge-server）。
#[test]
fn no_legacy_flaps_server_bin() {
    let bins = Path::new(PLATFORM_DIR).join("src").join("bin");
    let Ok(entries) = fs::read_dir(&bins) else {
        panic!("bin dir missing: {}", bins.display());
    };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        assert!(
            !name.contains("flaps"),
            "legacy bin file must not return: {}",
            name
        );
    }
}

#[test]
fn naming_guard_living_surfaces_clean() {
    scan(BANNED, "naming-guard");
}

/// 正向控制：自建品牌字串必須實際渲染（防守衛被「全刪字串」規避）。
#[test]
fn self_brand_present_in_index() {
    let index = Path::new(PLATFORM_DIR)
        .parent()
        .expect("workspace root")
        .join("web")
        .join("index.html");
    let text = fs::read_to_string(&index).expect("web/index.html readable");
    assert!(
        text.contains("AuxBridge 整合工作台"),
        "self brand 'AuxBridge 整合工作台' must be present in web/index.html"
    );
}
