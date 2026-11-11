// Flaps ERP — 更新 feed 清單產生器（latest.json）
//
// 用途（計畫 1793033568961 §W10）：把 NSIS 產物與其 minisign 簽章組裝成
// tauri-plugin-updater 2.14.x 期望的 latest.json，避免人工編輯 JSON 造成格式錯誤
// 或版本不一致（requireSignedVersion 會直接拒裝）。
//
// 格式與驗證語意對齊 tauri-plugin-updater（src/updater.rs::verify_signature）：
//   1. plugins.updater.pubkey 為 base64(minisign 公鑰文字)；本工具會比對產物簽章。
//   2. platforms.<target>.signature 為 base64(.sig 檔全文)。
//   3. platforms.<target>.url 指向產物本體（檔名空白需 percent-encode）。
//   4. requireSignedVersion = true 時，簽章 trusted comment 內含 `version:<v>`；
//      與本檔 `version` 欄位不一致即拒裝（防 feed 版本號遭竄改）。
//
// 用法：
//   cargo run --bin make_latest_json -- [--output <path>] [--notes <text>]
//                                                [--pub-date <rfc3339>]
//                                                [--endpoint <base-url>]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use minisign_verify::{PublicKey, Signature};
use serde_json::json;

/// 產物目錄（相對於 crate 根）。
const BUNDLE_DIR: &str = "target/release/bundle/nsis";

fn main() -> ExitCode {
    match run() {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("錯誤：{err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = crate_dir.join("tauri.conf.json");
    let config_text = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("讀取 {} 失敗：{e}", config_path.display()))?;
    let config: serde_json::Value = serde_json::from_str(&config_text)
        .map_err(|e| format!("解析 {} 失敗：{e}", config_path.display()))?;

    let version = config
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "tauri.conf.json 缺少 version".to_string())?
        .to_string();
    let updater = config
        .get("plugins")
        .and_then(|p| p.get("updater"))
        .ok_or_else(|| "tauri.conf.json 缺少 plugins.updater".to_string())?;
    let pubkey = updater
        .get("pubkey")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "plugins.updater 缺少 pubkey".to_string())?;
    let require_signed_version = updater
        .get("requireSignedVersion")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !require_signed_version {
        return Err(
            "plugins.updater.requireSignedVersion 必須為 true（停用即失去版本綁定保護）"
                .to_string(),
        );
    }
    let endpoint = match arg_value("--endpoint") {
        Some(value) => value,
        None => default_endpoint(updater)?,
    };

    let bundle_dir = crate_dir.join(BUNDLE_DIR);
    let installer = find_installer(&bundle_dir, &version)?;
    let signature_path = installer.with_extension("exe.sig");
    // Tauri CLI 產出的 .sig 檔為「base64(minisign 四行格式)」的單行文字；
    // latest.json 的 signature 欄位即為此 base64 字串（外掛解碼後才解析簽章）。
    let signature_b64 = std::fs::read_to_string(&signature_path)
        .map_err(|e| format!("讀取簽章 {} 失敗：{e}", signature_path.display()))?;
    let signature_b64 = signature_b64.trim().to_string();
    let signature_text = BASE64
        .decode(&signature_b64)
        .map_err(|e| format!("簽章 {} 非合法 base64：{e}", signature_path.display()))?;
    let signature_text = String::from_utf8(signature_text)
        .map_err(|_| format!("簽章 {} base64 解碼後非 UTF-8", signature_path.display()))?;

    // 以設定檔公鑰實際驗簽（與 tauri-plugin-updater 相同語意），確保產出的
    // latest.json 不會指向無法安裝的簽章；同時檢查版本綁定欄位。
    let pubkey_text = BASE64
        .decode(pubkey)
        .map_err(|e| format!("plugins.updater.pubkey 非合法 base64：{e}"))?;
    let pubkey_text = String::from_utf8(pubkey_text)
        .map_err(|_| "plugins.updater.pubkey base64 解碼後非 UTF-8".to_string())?;
    let public_key = PublicKey::decode(&pubkey_text)
        .map_err(|e| format!("無法解析 plugins.updater.pubkey：{e:?}"))?;
    let signature = Signature::decode(&signature_text)
        .map_err(|e| format!("無法解析簽章 {}：{e:?}", signature_path.display()))?;
    let installer_bytes = std::fs::read(&installer)
        .map_err(|e| format!("讀取產物 {} 失敗：{e}", installer.display()))?;
    public_key
        .verify(&installer_bytes, &signature, true)
        .map_err(|e| format!("簽章驗證失敗（公鑰與產物不符）：{e:?}"))?;
    let signed_version = signature
        .trusted_comment()
        .split('\t')
        .find_map(|field| field.strip_prefix("version:"));
    match signed_version {
        Some(v) if v == version => {}
        Some(v) => return Err(format!("簽章綁定版本為 {v}，與設定版本 {version} 不符")),
        None => {
            return Err(format!(
                "簽章 trusted comment 未含 version:{version}（requireSignedVersion 將拒裝）"
            ))
        }
    }

    let file_name = installer
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "產物檔名非 UTF-8".to_string())?;
    let url = format!(
        "{}/{}",
        endpoint.trim_end_matches('/'),
        percent_encode(file_name)
    );

    let notes = arg_value("--notes")
        .unwrap_or_else(|| format!("Flaps ERP {version}（自動產生 latest.json）"));
    let pub_date = arg_value("--pub-date")
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));

    let manifest = json!({
        "version": version,
        "notes": notes,
        "pub_date": pub_date,
        "platforms": {
            "windows-x86_64": {
                "signature": signature_b64,
                "url": url,
            }
        }
    });

    let output = match arg_value("--output") {
        Some(path) => PathBuf::from(path),
        None => bundle_dir.join("latest.json"),
    };
    let body = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("序列化 latest.json 失敗：{e}"))?;
    std::fs::write(&output, format!("{body}\n"))
        .map_err(|e| format!("寫入 {} 失敗：{e}", output.display()))?;

    Ok(format!(
        "已產生 {}（version={version}，產物={}）",
        output.display(),
        installer.display()
    ))
}

/// 預設更新端點基底（取 endpoints[0] 去掉檔名 latest.json）。
fn default_endpoint(updater: &serde_json::Value) -> Result<String, String> {
    let endpoints = updater
        .get("endpoints")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "plugins.updater 缺少 endpoints".to_string())?;
    let first = endpoints
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| "plugins.updater.endpoints 為空".to_string())?;
    Ok(first
        .rsplit_once('/')
        .map(|(base, _)| base.to_string())
        .unwrap_or_else(|| first.to_string()))
}

/// 於 bundle 目錄找出對應版本的 NSIS 安裝檔。
fn find_installer(bundle_dir: &Path, version: &str) -> Result<PathBuf, String> {
    if !bundle_dir.is_dir() {
        return Err(format!(
            "找不到產物目錄 {}（先執行 cargo tauri build）",
            bundle_dir.display()
        ));
    }
    let entries = std::fs::read_dir(bundle_dir)
        .map_err(|e| format!("讀取 {} 失敗：{e}", bundle_dir.display()))?;
    let mut candidates: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains(version) && n.ends_with("-setup.exe"))
        })
        .collect();
    candidates.sort();
    candidates.pop().ok_or_else(|| {
        format!(
            "{} 中找不到版本 {version} 的 -setup.exe",
            bundle_dir.display()
        )
    })
}

/// 極簡 percent-encode（僅處理檔名會出現的保留字元）。
fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// 讀取 `--flag value` 形式的第一個出現值。
fn arg_value(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        if arg == flag {
            return iter.next().cloned();
        }
        if let Some(rest) = arg.strip_prefix(&format!("{flag}=")) {
            return Some(rest.to_string());
        }
    }
    None
}
