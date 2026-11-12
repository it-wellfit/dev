#![forbid(unsafe_code)]

//! 機密掃描閘（`SHARED_STANDARDS` §7.1／§6.2 第 6 項）。
//!
//! 掃描版本控制內之文字檔，找出高訊號機密模式：私鑰、常見憑證、硬編碼高熵密碼。
//! 以 `cargo test` 實作（C-11：新工具一律 Rust），避免新增 .ps1/.py 工具鏈。
//!
//! 設計取捨：**高訊號優先**，寧可少報也不誤報（誤報會侵蝕閘門可信度）。
//! 允許清單：明確之佔位符／假資料（`demo-admin-2026` 為本機示範種子，已登錄）。

use std::path::{Path, PathBuf};

const SCAN_EXTENSIONS: [&str; 9] = [
    "rs", "toml", "json", "md", "yml", "yaml", "ps1", "html", "css",
];
const SKIP_DIRS: [&str; 15] = [
    "target",
    ".git",
    "node_modules",
    ".kilo",
    "Wellfit",
    "worktrees",
    "mutants.out",
    "archive",
    "fuzz",
    // 技能文件（.opencode/skills）內含「憑證處理」範例程式碼（如
    // client.create_secret(...)），為教學示例非真實憑證——排除以免誤報。
    ".opencode",
    // 技能設定歸檔（大量範例/備份文件，非執行源碼）。
    "kilo-settings-skills-archive",
    // ekpi-platform 為獨立產品線（自有 AGENTS.md、scripts/check.ps1、
    // gate.ps1 品質閘門）；根掃描之 scope 為根 workspace，不越界。
    "ekpi-platform",
    // SyncMesh 為獨立子專案（根 AGENTS.md §1：各依其 AGENTS.md 操作）。
    "SyncMesh",
    // vendored 上游（ghidra-rust；第三方碼非本庫源碼）。
    "rea-rust-upstream",
    // 測試目錄：整合測試夹具有假憑證（*-admin-pass 等），非生產秘密；
    // 生產掃描之 scope 為執行源碼，測試夹具不入。
    "tests",
];
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;

/// 允許清單：已知之非秘密字串（佔位符或本機示範種子），附理由。
const ALLOWLIST: [(&str, &str); 8] = [
    ("<custody 密碼>", "手冊中之佔位符"),
    ("demo-admin-2026", "本機示範資料之種子密碼（非生產）"),
    (
        "flaps-updater-2026-rotate-before-production",
        "簽章金鑰之語義值（手冊已載明；實際金鑰檔案不在版控）",
    ),
    ("test-only-password", "測試用佔位字串"),
    ("test-admin-pass", "整合測試之管理員夹具密碼（非生產）"),
    ("test-store-pass", "整合測試之門市夹具密碼（非生產）"),
    (
        "Admin-Test-Passw0rd!",
        "稽核測試之夹具密碼（非生產；僅用於測試 DB）",
    ),
    ("Clerk-B-Passw0rd!", "稽核測試之職員夹具密碼（非生產）"),
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("platform crate 應位於 repo 根之下")
        .to_path_buf()
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if SKIP_DIRS.contains(&name.as_str()) || name.starts_with("skills.backup") {
                continue;
            }
            collect_files(&path, out);
            continue;
        }
        let ok_ext = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| SCAN_EXTENSIONS.contains(&e));
        if !ok_ext {
            continue;
        }
        if entry.metadata().map(|m| m.len()).unwrap_or(0) > MAX_FILE_BYTES {
            continue;
        }
        out.push(path);
    }
}

#[test]
fn no_hardcoded_secrets_in_tracked_sources() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_files(&root, &mut files);
    assert!(
        files.len() >= 200,
        "掃描器必須掃到真實樹（僅 {} 檔）",
        files.len()
    );

    let mut findings: Vec<String> = Vec::new();
    for file in &files {
        // 掃描器自身（含 -----BEGIN/PRIVATE KEY 偵測模式）→ 跳過自獵。
        if file.ends_with("secret_scan.rs") {
            continue;
        }
        let Ok(bytes) = std::fs::read(file) else {
            continue;
        };
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        for (idx, line) in text.lines().enumerate() {
            if line.len() > 2000 {
                continue; // 壓縮資料/長字串不檢查
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("<!--")
            {
                // 註解仍檢查（秘密常藏在註解），但不檢查 Markdown 表格分隔線等雜訊。
            }
            if let Some(reason) = classify(trimmed) {
                if ALLOWLIST.iter().any(|(token, _)| trimmed.contains(token)) {
                    continue;
                }
                findings.push(format!(
                    "{}:{} [{}] {}",
                    file.strip_prefix(&root).unwrap_or(file).display(),
                    idx + 1,
                    reason,
                    truncate(trimmed, 120)
                ));
            }
        }
    }

    assert!(
        findings.is_empty(),
        "疑似硬編碼機密（{} 筆）：\n{}",
        findings.len(),
        findings.join("\n")
    );
}

/// 回傳命中理由；僅辨識高訊號樣式。
fn classify(line: &str) -> Option<&'static str> {
    if line.contains("-----BEGIN") && line.contains("PRIVATE KEY") {
        return Some("私鑰");
    }
    let lower = line.to_ascii_lowercase();
    let assignment = ['=', ':'];
    for key in [
        "password",
        "passwd",
        "secret",
        "api_key",
        "apikey",
        "token",
        "client_secret",
        "private_key",
    ] {
        let Some(pos) = lower.find(key) else { continue };
        // 鍵若位於字串字面值內（鍵前引號數為奇數），則此為偵測模式/測試
        // 資料/錯誤訊息（如 "password="、"hash password: {e}"），非賦值。
        if line[..pos].matches('"').count() % 2 == 1 {
            continue;
        }
        let rest = &line[pos + key.len()..];
        let Some(trimmed) = rest.trim_start().chars().next() else {
            continue;
        };
        if !assignment.contains(&trimmed) {
            continue;
        }
        // 值抽取：有引號時取引號內文（可穿透型別標註如 &str = "..."），
        // 無引號時取賦值後的裸值（欄位存取/函式呼叫等由下方啟發式過濾）。
        let after_assign = rest
            .trim_start()
            .trim_start_matches(['=', ':'])
            .trim()
            .trim_end_matches(|c: char| c == '{' || c.is_whitespace());
        let value = match after_assign.find('"').or_else(|| after_assign.find('\'')) {
            Some(q) => {
                let quote = after_assign.as_bytes()[q] as char;
                let inner = &after_assign[q + 1..];
                match inner.find(quote) {
                    Some(end) => inner[..end].trim(),
                    None => inner.trim(),
                }
            }
            None => after_assign
                .trim_matches(|c| c == '"' || c == '\'' || c == ',')
                .trim(),
        };
        if value.is_empty() {
            continue;
        }
        // 排除明顯非秘密：佔位符、變數參照、空字串、型別宣告、函式呼叫、
        // 純欄位存取（config.api_key、form.csrf_token 等）、陣列宣告。
        let field_access = value.len() <= 40
            && value.split('.').count() == 2
            && value
                .split('.')
                .all(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
            && value.starts_with(|c: char| c.is_ascii_lowercase() || c == '_');
        let looks_secret = value.len() >= 8
            && !value.starts_with('$')
            && !value.starts_with("${{")
            && !value.starts_with('<') // 佔位符：<password>、<local-password>
            && !value.starts_with("choose-") // 佔位符慣例：choose-a-long-...
            && !value.starts_with('[') // 陣列宣告（[0u8; 32]）
            && !value.chars().all(|c| c.is_ascii_uppercase() || c == '_') // env 變數名（AUXBRIDGE_ADMIN_PASSWORD）
            && !value.contains("env::")
            && !value.contains("env!(")
            && !value.contains("$env:") // 值內的環境變數參照（非硬編碼秘密）
            && !value.contains('(') // 函式呼叫（trim_punct(token)、generate_...）
            && !field_access
            && !value.starts_with("Option")
            && !value.starts_with("Some(")
            && !value.starts_with("String")
            && !value.starts_with('&')
            && !value.contains("example")
            && !value.contains("your_")
            && !value.contains("xxxx");
        if looks_secret {
            return Some("硬編碼憑證樣式");
        }
    }
    None
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    text.chars().take(max).collect::<String>() + "…"
}

// 偵測能力回歸（2026-11-12）：SED 強化放過源碼慣例的同時，真秘密必須
// 仍被抓到——雙向鎖定，防止「修誤報」變成「瞎掉」。
#[test]
fn catches_real_hardcoded_secrets() {
    for line in [
        r#"let password = "hunter2longvalue";"#,
        r#"let api_key = "sk-abc123456789xyz";"#,
        r#"const TOKEN: &str = "ghp_abcdefghijklmnop";"#,
        r#"client_secret: "topsecretvalue123","#,
    ] {
        assert!(classify(line).is_some(), "真秘密必須被抓到：{line}");
    }
}

#[test]
fn ignores_source_idioms() {
    for line in [
        "let token = trim_punct(token);",
        "if session.csrf_token == form.csrf_token {",
        r#"let csrf_token = generate_csrf_token();"#,
        "api_key: config.api_key,",
        r#"["-----BEGIN", "AKIA", "sk-", "ghp_", "password=", "api_key="];"#,
        r#"let mut secret = [0u8; 32];"#,
        r#"format!("hash password: {error}")"#,
        r#"claim_with_body("store api_key=abc123 here")"#,
        r#"$env:TAURI_SIGNING_PRIVATE_KEY = [IO.File]::ReadAllText("x")"#,
        r#"password = "<local-password>""#,
    ] {
        assert!(classify(line).is_none(), "源碼慣例不應誤報：{line}");
    }
}

#[test]
fn catches_private_key_blocks() {
    assert!(classify("-----BEGIN RSA PRIVATE KEY-----").is_some());
}
