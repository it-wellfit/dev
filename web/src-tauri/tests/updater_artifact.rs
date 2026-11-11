#![forbid(unsafe_code)]

//! W10 驗收閘（計畫 1793033568961）：更新鏈的設定不變量、產物簽章驗證與負向用例。
//!
//! 驗證與版本綁定語意**共用** `flaps_erp_web::feed`（與產生器
//! `src/bin/make_latest_json.rs` 同一份實作），避免「產生」與「驗收」各說各話。
//! `feed` 的單元測試以測試金鑰完成 sign→feed→verify round-trip，可在無產物之
//! CI 環境執行；本檔案的產物層測試則需 `cargo tauri build` 之實際產物。
//!
//! 產物不存在時（尚未 `cargo tauri build`）本測試會略過產物項目並輸出說明；
//! 發行流水線請設 `AUXBRIDGE_REQUIRE_RELEASE_ARTIFACTS=1` 令缺產物直接失敗。

use std::path::{Path, PathBuf};

use auxbridge_web::feed;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

const BUNDLE_DIR: &str = feed::BUNDLE_DIR;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_config() -> serde_json::Value {
    let path = crate_dir().join("tauri.conf.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("讀取 {} 失敗：{e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("解析 {} 失敗：{e}", path.display()))
}

fn config_pubkey(config: &serde_json::Value) -> String {
    config["plugins"]["updater"]["pubkey"]
        .as_str()
        .unwrap_or_else(|| panic!("tauri.conf.json 缺少 plugins.updater.pubkey"))
        .to_string()
}

/// 設定層不變量：簽章驗證不可停用、公鑰可解析、端點為 HTTPS、產物需簽章、版本一致。
#[test]
fn config_pins_signed_version_updates() {
    let config = read_config();

    assert_eq!(
        config["plugins"]["updater"]["requireSignedVersion"].as_bool(),
        Some(true),
        "plugins.updater.requireSignedVersion 必須為 true（否則失去版本綁定保護）"
    );
    assert_eq!(
        config["bundle"]["createUpdaterArtifacts"].as_bool(),
        Some(true),
        "bundle.createUpdaterArtifacts 必須為 true（否則不產生 .sig）"
    );
    assert_eq!(
        config["version"].as_str(),
        Some(env!("CARGO_PKG_VERSION")),
        "tauri.conf.json 版本必須與 Cargo.toml 一致"
    );

    let pubkey = config_pubkey(&config);
    feed::decode_public_key(&pubkey).expect("plugins.updater.pubkey 應可解析為 minisign 公鑰");

    let endpoints = config["plugins"]["updater"]["endpoints"]
        .as_array()
        .unwrap_or_else(|| panic!("plugins.updater.endpoints 應為陣列"));
    assert!(!endpoints.is_empty(), "更新端點不可為空");
    for endpoint in endpoints {
        let url = endpoint.as_str().expect("端點應為字串");
        assert!(url.starts_with("https://"), "更新端點必須為 HTTPS：{url}");
    }
}

/// 產物層驗收：正例（驗簽成功＋版本綁定）與負例（改簽章／改產物／改版本皆必敗）。
#[test]
fn release_artifacts_are_signed_and_version_bound() {
    let config = read_config();
    let version = config["version"]
        .as_str()
        .expect("tauri.conf.json 應有 version")
        .to_string();
    let bundle_dir = crate_dir().join(BUNDLE_DIR);

    let installer = match find_installer(&bundle_dir, &version) {
        Some(path) => path,
        None => {
            if std::env::var("AUXBRIDGE_REQUIRE_RELEASE_ARTIFACTS").as_deref() == Ok("1") {
                panic!(
                    "{} 找不到版本 {version} 的安裝產物（發行流水線要求產物存在）",
                    bundle_dir.display()
                );
            }
            eprintln!(
                "略過產物驗證：{} 尚無版本 {version} 的安裝產物（先執行 cargo tauri build）",
                bundle_dir.display()
            );
            return;
        }
    };
    let sig_path = installer.with_extension("exe.sig");
    let sig_b64 = std::fs::read_to_string(&sig_path)
        .unwrap_or_else(|e| panic!("讀取 {} 失敗：{e}", sig_path.display()));
    let installer_bytes = std::fs::read(&installer)
        .unwrap_or_else(|e| panic!("讀取 {} 失敗：{e}", installer.display()));
    let pubkey_b64 = config_pubkey(&config);

    // 正例：產物可通過驗簽，且簽章綁定版本與設定版本一致（requireSignedVersion 的前提）。
    feed::verify_signature_for_version(&pubkey_b64, sig_b64.trim(), &installer_bytes, &version)
        .expect("正式產物必須通過 minisign 驗簽且版本綁定成立");

    // 負例一：竄改簽章 → 驗簽必敗。
    let tampered_sig = tamper_signature(sig_b64.trim());
    assert!(
        feed::verify_signature_for_version(&pubkey_b64, &tampered_sig, &installer_bytes, &version)
            .is_err(),
        "竄改簽章後必須驗簽失敗"
    );

    // 負例二：竄改產物位元組 → 驗簽必敗。
    let mut tampered_bytes = installer_bytes.clone();
    let last = tampered_bytes.len() - 1;
    tampered_bytes[last] ^= 0xFF;
    assert!(
        feed::verify_signature_for_version(&pubkey_b64, sig_b64.trim(), &tampered_bytes, &version)
            .is_err(),
        "竄改產物後必須驗簽失敗"
    );

    // 負例三：feed 宣告版本與簽章綁定版本不符 → 版本綁定檢查必敗（與外掛同語意）。
    let announced = "0.0.1";
    assert_ne!(announced, version.as_str());
    let err = feed::verify_signature_for_version(
        &pubkey_b64,
        sig_b64.trim(),
        &installer_bytes,
        announced,
    )
    .expect_err("宣告版本與簽章綁定不符時必須拒裝");
    assert!(err.contains("不符"), "錯誤訊息應說明版本不符：{err}");
}

/// 於 bundle 目錄找出對應版本的 NSIS 安裝檔。
fn find_installer(bundle_dir: &Path, version: &str) -> Option<PathBuf> {
    if !bundle_dir.is_dir() {
        return None;
    }
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(bundle_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains(version) && n.ends_with("-setup.exe"))
        })
        .collect();
    candidates.sort();
    candidates.pop()
}

/// 竄改 .sig 的「簽章載荷行」（minisign 簽章文字第 2 行）後重新 base64 編碼。
///
/// 第 1 行是未簽章之 untrusted comment，竄改它不會改變驗簽結果，故不作為負例。
fn tamper_signature(text: &str) -> String {
    let decoded = BASE64.decode(text).expect("簽章應為合法 base64");
    let decoded = String::from_utf8(decoded).expect("簽章 base64 解碼後應為 UTF-8");
    let mut lines: Vec<String> = decoded.lines().map(|l| l.to_string()).collect();
    assert!(lines.len() >= 2, "簽章文字應至少含兩行");
    let mut payload: Vec<char> = lines[1].chars().collect();
    assert!(!payload.is_empty(), "簽章載荷不可為空");
    payload[0] = if payload[0] == 'A' { 'B' } else { 'A' };
    lines[1] = payload.into_iter().collect();
    BASE64.encode(lines.join("\n").as_bytes())
}
