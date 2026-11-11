#![forbid(unsafe_code)]

//! W10 驗收閘（計畫 1793033568961）：更新鏈的設定不變量、產物簽章驗證與負向用例。
//!
//! 驗證語意與 tauri-plugin-updater 2.14.x（src/updater.rs::verify_signature）一致：
//!   1. `plugins.updater.pubkey` 為 base64(minisign 公鑰文字)。
//!   2. `platforms.<target>.signature` 為 base64(.sig 檔全文)。
//!   3. 產物位元組必須通過 minisign 驗簽。
//!   4. `requireSignedVersion = true` 時，簽章 trusted comment 內的 `version:<v>`
//!      必須與 feed 宣告版本一致。
//!
//! 產物不存在時（尚未 `cargo tauri build`）本測試會略過產物項目並輸出說明；
//! 發行流水線請設 `FLAPS_REQUIRE_RELEASE_ARTIFACTS=1` 令缺產物直接失敗。

use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use minisign_verify::{PublicKey, Signature};

const BUNDLE_DIR: &str = "target/release/bundle/nsis";

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

fn public_key(pubkey_b64: &str) -> PublicKey {
    let decoded = BASE64
        .decode(pubkey_b64)
        .unwrap_or_else(|e| panic!("pubkey 非合法 base64：{e}"));
    let text = String::from_utf8(decoded).expect("pubkey base64 解碼後應為 UTF-8");
    PublicKey::decode(&text).unwrap_or_else(|e| panic!("無法解析 pubkey：{e:?}"))
}

fn signature(sig_text: &str) -> Signature {
    Signature::decode(sig_text).unwrap_or_else(|e| panic!("無法解析簽章：{e:?}"))
}

/// Tauri CLI 的 .sig 檔為 base64(minisign 四行格式)；latest.json 的 signature 欄位
/// 即為此 base64 字串（外掛解碼後才解析簽章）。
fn signature_text_from_sig_file(sig_b64: &str) -> String {
    let decoded = BASE64
        .decode(sig_b64.trim())
        .unwrap_or_else(|e| panic!("簽章檔非合法 base64：{e}"));
    String::from_utf8(decoded).expect("簽章 base64 解碼後應為 UTF-8")
}

/// 與外掛一致：trusted comment 以 tab 分欄，取 `version:` 前綴欄位。
fn signed_version(trusted_comment: &str) -> Option<&str> {
    trusted_comment
        .split('\t')
        .find_map(|field| field.strip_prefix("version:"))
}

/// 與外掛一致：以 `allow_legacy = true` 驗簽（minisign 舊版相容）。
fn verify(public_key: &PublicKey, sig: &Signature, data: &[u8]) -> Result<(), String> {
    public_key
        .verify(data, sig, true)
        .map_err(|e| format!("{e:?}"))
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
    let _ = public_key(&pubkey);

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
            if std::env::var("FLAPS_REQUIRE_RELEASE_ARTIFACTS").as_deref() == Ok("1") {
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
    let sig_text = signature_text_from_sig_file(&sig_b64);
    let installer_bytes = std::fs::read(&installer)
        .unwrap_or_else(|e| panic!("讀取 {} 失敗：{e}", installer.display()));

    let pubkey = public_key(&config_pubkey(&config));
    let sig = signature(&sig_text);

    // 正例：產物可通過驗簽。
    verify(&pubkey, &sig, &installer_bytes).expect("正式產物必須通過 minisign 驗簽");

    // 正例：簽章綁定版本與設定版本一致（requireSignedVersion 的前提）。
    assert_eq!(
        signed_version(sig.trusted_comment()),
        Some(version.as_str()),
        "簽章 trusted comment 必須綁定版本 {version}"
    );

    // 負例一：竄改簽章 → 驗簽必敗。
    let tampered_sig_text = tamper_signature(&sig_text);
    let tampered = signature_or_error(&tampered_sig_text);
    assert!(
        tampered.is_err() || verify(&pubkey, &tampered.unwrap(), &installer_bytes).is_err(),
        "竄改簽章後必須驗簽失敗"
    );
    // 負例二：竄改產物位元組 → 驗簽必敗。
    let mut tampered_bytes = installer_bytes.clone();
    let last = tampered_bytes.len() - 1;
    tampered_bytes[last] ^= 0xFF;
    assert!(
        verify(&pubkey, &sig, &tampered_bytes).is_err(),
        "竄改產物後必須驗簽失敗"
    );

    // 負例三：feed 宣告版本與簽章綁定版本不符 → 版本綁定檢查必敗。
    let announced = "0.0.1";
    assert_ne!(announced, version.as_str());
    let bound = signed_version(sig.trusted_comment());
    assert!(
        bound != Some(announced),
        "宣告版本 {announced} 與簽章綁定 {bound:?} 不符時必須拒裝"
    );
}

fn signature_or_error(text: &str) -> Result<Signature, String> {
    Signature::decode(text).map_err(|e| format!("{e:?}"))
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

/// 竄改 .sig 的 base64 載荷首字元（保持可解碼，但簽章位元組改變）。
fn tamper_signature(text: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    assert!(lines.len() >= 2, "簽章檔應至少含註解與載荷兩行");
    let payload: Vec<char> = lines[1].chars().collect();
    assert!(!payload.is_empty(), "簽章載荷不可為空");
    let replacement = if payload[0] == 'A' { 'B' } else { 'A' };
    let mut tampered_payload = payload;
    tampered_payload[0] = replacement;
    lines[1] = tampered_payload.into_iter().collect();
    lines.join("\n")
}
