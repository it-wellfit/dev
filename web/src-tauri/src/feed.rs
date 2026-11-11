#![forbid(unsafe_code)]
//! 更新 feed（latest.json）之簽章驗證與清單組裝。
//!
//! 語意與 `tauri-plugin-updater` 2.14.x（`src/updater.rs::verify_signature`）一致：
//!   1. `plugins.updater.pubkey` 為 base64(minisign 公鑰文字)。
//!   2. `platforms.<target>.signature` 為 base64(.sig 檔全文)。
//!   3. `requireSignedVersion = true` 時，簽章 trusted comment 內的 `version:<v>`
//!      必須與 feed 宣告版本一致，否則外掛拒裝。
//!
//! 本模組同時是產生器（`src/bin/make_latest_json.rs`）與驗收測試
//! （`tests/updater_artifact.rs`）的單一實作來源。

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use minisign_verify::{PublicKey, Signature};
use serde_json::json;

/// 產物目錄（相對於 crate 根）。
pub const BUNDLE_DIR: &str = "target/release/bundle/nsis";

/// 解碼 `plugins.updater.pubkey`（base64 編碼之 minisign 公鑰文字）。
pub fn decode_public_key(pubkey_b64: &str) -> Result<PublicKey, String> {
    let decoded = BASE64
        .decode(pubkey_b64.trim())
        .map_err(|e| format!("pubkey 非合法 base64：{e}"))?;
    let text =
        String::from_utf8(decoded).map_err(|_| "pubkey base64 解碼後非 UTF-8".to_string())?;
    PublicKey::decode(&text).map_err(|e| format!("無法解析 pubkey：{e:?}"))
}

/// 解碼 `.sig` 檔內容（base64 編碼之 minisign 四行格式）為簽章文字。
pub fn decode_signature_text(signature_b64: &str) -> Result<String, String> {
    let decoded = BASE64
        .decode(signature_b64.trim())
        .map_err(|e| format!("簽章非合法 base64：{e}"))?;
    String::from_utf8(decoded).map_err(|_| "簽章 base64 解碼後非 UTF-8".to_string())
}

/// 與外掛一致：trusted comment 以 tab 分欄，取 `version:` 前綴欄位。
pub fn signed_version(trusted_comment: &str) -> Option<&str> {
    trusted_comment
        .split('\t')
        .find_map(|field| field.strip_prefix("version:"))
}

/// 驗簽並強制版本綁定（`requireSignedVersion` 語意）。
///
/// 任何一步失敗即回傳 `Err`：公鑰/簽章無法解析、產物位元組遭竄改、
/// 或簽章綁定版本與 `expected_version` 不一致。
pub fn verify_signature_for_version(
    pubkey_b64: &str,
    signature_b64: &str,
    artifact: &[u8],
    expected_version: &str,
) -> Result<(), String> {
    let public_key = decode_public_key(pubkey_b64)?;
    let signature_text = decode_signature_text(signature_b64)?;
    let signature =
        Signature::decode(&signature_text).map_err(|e| format!("無法解析簽章：{e:?}"))?;
    public_key
        .verify(artifact, &signature, true)
        .map_err(|e| format!("簽章驗證失敗（公鑰與產物不符）：{e:?}"))?;
    match signed_version(signature.trusted_comment()) {
        Some(v) if v == expected_version => Ok(()),
        Some(v) => Err(format!(
            "簽章綁定版本為 {v}，與期望版本 {expected_version} 不符"
        )),
        None => Err(format!(
            "簽章 trusted comment 未含 version:{expected_version}（requireSignedVersion 將拒裝）"
        )),
    }
}

/// 組裝 `latest.json` 主體（不含寫檔）。
pub fn build_manifest(
    version: &str,
    notes: &str,
    pub_date: &str,
    signature_b64: &str,
    endpoint: &str,
    file_name: &str,
) -> Result<serde_json::Value, String> {
    if version.trim().is_empty() {
        return Err("version 不可為空".to_string());
    }
    if signature_b64.trim().is_empty() {
        return Err("signature 不可為空".to_string());
    }
    if !endpoint.starts_with("https://") {
        return Err(format!("更新端點必須為 HTTPS：{endpoint}"));
    }
    let url = format!(
        "{}/{}",
        endpoint.trim_end_matches('/'),
        percent_encode(file_name)
    );
    Ok(json!({
        "version": version,
        "notes": notes,
        "pub_date": pub_date,
        "platforms": {
            "windows-x86_64": {
                "signature": signature_b64,
                "url": url,
            }
        }
    }))
}

/// 極簡 percent-encode（僅處理檔名會出現的保留字元）。
pub fn percent_encode(input: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// 以測試金鑰簽署 `artifact`，回傳 (base64 公鑰, base64 簽章, 公鑰) —— 與
    /// `tauri.conf.json` 的 pubkey 欄位、`.sig` 檔的編碼方式完全相同。
    /// 測試用金鑰不加密（`minisign::sign` 不接受加密私鑰）；金鑰僅存在於測試行程記憶體。
    fn sign_with_test_key(
        trusted_comment: &str,
        artifact: &[u8],
    ) -> (String, String, minisign::PublicKey) {
        let key_pair = minisign::KeyPair::generate_unencrypted_keypair().expect("可產生測試金鑰對");
        let public_key_text = key_pair.pk.to_box().expect("公鑰可轉為 box").to_string();
        let signature_box = minisign::sign(
            None,
            &key_pair.sk,
            Cursor::new(artifact),
            Some(trusted_comment),
            None,
        )
        .expect("可完成簽章");
        let signature_text = signature_box.into_string();
        (
            BASE64.encode(public_key_text.as_bytes()),
            BASE64.encode(signature_text.as_bytes()),
            key_pair.pk,
        )
    }

    /// 正例：測試金鑰簽章 → 驗簽通過、版本綁定成立、feed 欄位正確。
    #[test]
    fn feed_round_trip_accepts_signed_artifact() {
        let artifact = b"fake-installer-bytes";
        let (pubkey_b64, signature_b64, _) = sign_with_test_key("version:0.2.0", artifact);

        verify_signature_for_version(&pubkey_b64, &signature_b64, artifact, "0.2.0")
            .expect("測試金鑰簽章必須通過驗證與版本綁定");

        let manifest = build_manifest(
            "0.2.0",
            "測試備註",
            "2026-11-11T00:00:00Z",
            &signature_b64,
            "https://updates.example.internal/erp",
            "Flaps ERP_0.2.0_x64-setup.exe",
        )
        .expect("可組裝 feed");

        assert_eq!(manifest["version"], "0.2.0");
        assert_eq!(manifest["pub_date"], "2026-11-11T00:00:00Z");
        assert_eq!(
            manifest["platforms"]["windows-x86_64"]["signature"],
            signature_b64
        );
        assert_eq!(
            manifest["platforms"]["windows-x86_64"]["url"],
            "https://updates.example.internal/erp/Flaps%20ERP_0.2.0_x64-setup.exe"
        );
    }

    /// 負例一：竄改簽章 → 驗簽必敗。
    ///
    /// 注意：minisign 簽章文字第 1 行是**未簽章**的 untrusted comment，竄改它
    /// 不會影響驗簽結果（不是有效負例）；必須竄改第 2 行的簽章載荷。
    #[test]
    fn tampered_signature_is_rejected() {
        let artifact = b"fake-installer-bytes";
        let (pubkey_b64, signature_b64, _) = sign_with_test_key("version:0.2.0", artifact);
        let tampered = tamper_signature_payload(&signature_b64);

        assert!(tampered != signature_b64, "竄改後的簽章文字應與原始不同");
        assert!(
            verify_signature_for_version(&pubkey_b64, &tampered, artifact, "0.2.0").is_err(),
            "竄改簽章後必須驗簽失敗"
        );
    }

    /// 竄改 minisign 簽章文字的「簽章載荷行」（第 2 行）後重新 base64 編碼。
    fn tamper_signature_payload(signature_b64: &str) -> String {
        let text = String::from_utf8(
            BASE64
                .decode(signature_b64.trim())
                .expect("簽章應為合法 base64"),
        )
        .expect("簽章 base64 解碼後應為 UTF-8");
        let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
        assert!(lines.len() >= 2, "簽章文字應至少含兩行");
        let mut payload: Vec<char> = lines[1].chars().collect();
        assert!(!payload.is_empty(), "簽章載荷不可為空");
        payload[0] = if payload[0] == 'A' { 'B' } else { 'A' };
        lines[1] = payload.into_iter().collect();
        BASE64.encode(lines.join("\n").as_bytes())
    }

    /// 負例二：竄改產物位元組 → 驗簽必敗。
    #[test]
    fn tampered_artifact_is_rejected() {
        let artifact = b"fake-installer-bytes";
        let (pubkey_b64, signature_b64, _) = sign_with_test_key("version:0.2.0", artifact);
        let mut tampered = artifact.to_vec();
        let last = tampered.len() - 1;
        tampered[last] ^= 0xFF;

        assert!(
            verify_signature_for_version(&pubkey_b64, &signature_b64, &tampered, "0.2.0").is_err(),
            "竄改產物後必須驗簽失敗"
        );
    }

    /// 負例三：簽章綁定版本與期望版本不符（feed 版本遭竄改）→ 必敗。
    #[test]
    fn version_mismatch_is_rejected() {
        let artifact = b"fake-installer-bytes";
        let (pubkey_b64, signature_b64, _) = sign_with_test_key("version:0.9.0", artifact);

        let err = verify_signature_for_version(&pubkey_b64, &signature_b64, artifact, "0.2.0")
            .expect_err("版本不符必須拒絕");
        assert!(err.contains("不符"), "錯誤訊息應說明版本不符：{err}");
    }

    /// 負例四：簽章未綁定版本（trusted comment 無 version: 欄位）→ 必敗。
    #[test]
    fn missing_version_binding_is_rejected() {
        let artifact = b"fake-installer-bytes";
        let (pubkey_b64, signature_b64, _) = sign_with_test_key("timestamp:1", artifact);

        let err = verify_signature_for_version(&pubkey_b64, &signature_b64, artifact, "0.2.0")
            .expect_err("缺少版本綁定必須拒絕");
        assert!(
            err.contains("version:0.2.0"),
            "錯誤訊息應說明缺少綁定：{err}"
        );
    }

    /// 端點非 HTTPS → 拒絕組裝（避免產生可被中間人替換的 feed）。
    #[test]
    fn build_manifest_rejects_non_https_endpoint() {
        let err = build_manifest(
            "0.2.0",
            "notes",
            "2026-11-11T00:00:00Z",
            "c2ln",
            "http://updates.example.internal/erp/latest.json",
            "Flaps ERP_0.2.0_x64-setup.exe",
        )
        .expect_err("非 HTTPS 端點必須拒絕");
        assert!(err.contains("HTTPS"), "錯誤訊息應說明需 HTTPS：{err}");
    }

    /// percent-encode 僅保留未保留字元，空白編為 %20。
    #[test]
    fn percent_encode_encodes_reserved_characters() {
        assert_eq!(
            percent_encode("Flaps ERP_0.2.0_x64-setup.exe"),
            "Flaps%20ERP_0.2.0_x64-setup.exe"
        );
        assert_eq!(percent_encode("a~b-c_d.e"), "a~b-c_d.e");
    }
}
