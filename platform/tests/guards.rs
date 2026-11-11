#![forbid(unsafe_code)]
//! Corruption & zero-JS guards (R5).
//!
//! Born from the 2026-11-07 api.rs mojibake incident: these tests scan
//! workspace sources for encoding damage (UTF-8 BOM, NUL bytes, PUA /
//! replacement characters) and enforce the zero-JavaScript invariant on
//! templates (`<script>` elements and inline `on*=` handlers). Incident
//! record: `.kilo/sessions/20261107-dev-flaps-completion/`.

use std::fs;
use std::path::{Path, PathBuf};

/// Directories scanned relative to the repository root (the platform crate's
/// manifest dir is inside the repo root; the root is its parent).
const SCAN_DIRS: [&str; 8] = [
    "core/src",
    "platform/src",
    "platform/templates",
    "platform/migrations",
    "web/assets",
    "web/src-tauri/src",
    "crates/kem/src",
    "knowledge/console",
];

const TEXT_EXTENSIONS: [&str; 6] = ["rs", "html", "css", "sql", "toml", "md"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("platform crate must live inside the repo root")
        .to_path_buf()
}

fn collect_text_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_text_files(&path, out);
        } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if TEXT_EXTENSIONS.contains(&ext) {
                out.push(path);
            }
        }
    }
}

/// Returns the damage kind when the byte stream shows corruption markers.
fn damage_kind(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Some("UTF-8 BOM present");
    }
    if bytes.contains(&0u8) {
        return Some("NUL byte present");
    }
    let text = String::from_utf8_lossy(bytes);
    for ch in text.chars() {
        let c = ch as u32;
        if c == 0xFFFD {
            return Some("U+FFFD replacement character");
        }
        if (0xE000..=0xF8FF).contains(&c) || c == 0xFEFF {
            return Some("Private-Use-Area character (mojibake damage)");
        }
    }
    None
}

#[test]
fn sources_are_free_of_encoding_damage() {
    let root = repo_root();
    let mut files = Vec::new();
    for dir in SCAN_DIRS {
        collect_text_files(&root.join(dir), &mut files);
    }
    assert!(
        files.len() >= 40,
        "guard must scan the real tree (found only {} files)",
        files.len()
    );
    let mut problems = Vec::new();
    for file in &files {
        let bytes = match fs::read(file) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if let Some(kind) = damage_kind(&bytes) {
            problems.push(format!("{}: {kind}", file.display()));
        }
    }
    assert!(
        problems.is_empty(),
        "encoding damage found (repair from the last clean revision; see .kilo/sessions/):\n{}",
        problems.join("\n")
    );
}

#[test]
fn templates_have_zero_javascript() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_text_files(&root.join("platform/templates"), &mut files);
    assert!(!files.is_empty(), "template directory must not be empty");
    const INLINE_HANDLERS: [&str; 12] = [
        "onclick=",
        "onload=",
        "onsubmit=",
        "onchange=",
        "onerror=",
        "oninput=",
        "onfocus=",
        "onblur=",
        "onmouseover=",
        "onmouseenter=",
        "onkeyup=",
        "onkeydown=",
    ];
    let mut problems = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).unwrap_or_default();
        if text.contains("<script") {
            problems.push(format!("{}: <script> element", file.display()));
        }
        for needle in INLINE_HANDLERS {
            if text.contains(needle) {
                problems.push(format!("{}: inline handler {needle}", file.display()));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "zero-JS invariant violated (Charter §2.3):\n{}",
        problems.join("\n")
    );
}

#[test]
fn templates_have_no_inline_styles() {
    // 平台 CSP 為 `style-src 'self'`（無 unsafe-inline）：模板內聯樣式會被
    // 瀏覽器直接阻擋而失效，故於建置期攔截（與零 JS 同屬呈現層契約）。
    let root = repo_root();
    let mut files = Vec::new();
    collect_text_files(&root.join("platform/templates"), &mut files);
    assert!(!files.is_empty(), "template directory must not be empty");
    let mut problems = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).unwrap_or_default();
        if text.contains("style=") {
            problems.push(format!("{}: inline style attribute", file.display()));
        }
    }
    assert!(
        problems.is_empty(),
        "inline style violates CSP style-src 'self':\n{}",
        problems.join("\n")
    );
}

// ---------------------------------------------------------------------------
// W6/W7（計畫 1793033568961）：KEM 控制台生成物護欄＋設計 token 收斂檢查。
// ---------------------------------------------------------------------------

#[test]
fn generated_console_has_zero_javascript_and_no_inline_styles() {
    let root = repo_root();
    let console = root.join("knowledge/console");
    assert!(
        console.join("index.html").is_file(),
        "knowledge/console/index.html must exist (run `kem --root knowledge console`)"
    );
    let mut files = Vec::new();
    collect_text_files(&console, &mut files);
    assert!(
        files.len() >= 40,
        "console pages missing (found {})",
        files.len()
    );
    let mut problems = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).unwrap_or_default();
        if text.contains("<script") {
            problems.push(format!("{}: <script> element", file.display()));
        }
        // W6：生成頁內聯樣式歸零（平台 CSP style-src 'self' 會阻擋）。
        if text.contains("style=") {
            problems.push(format!("{}: inline style attribute", file.display()));
        }
    }
    assert!(
        problems.is_empty(),
        "KEM console zero-JS / de-inline invariant violated:\n{}",
        problems.join("\n")
    );
}

#[test]
fn console_css_tokens_are_canonical_design_system_values() {
    // W7（grill-E 全收斂）：console.css 的自造色值必須逐字存在於
    // web/assets/design-system.css canonical 調色（漂移即攔截）。
    let root = repo_root();
    let console_css = fs::read_to_string(root.join("knowledge/console/assets/console.css"))
        .expect("console.css must exist");
    let design = fs::read_to_string(root.join("web/assets/design-system.css"))
        .expect("design-system.css must exist");
    // 抽取 console.css 的色值字面值（#rgb/#rrggbb）。
    let mut problems = Vec::new();
    let mut in_def = false;
    for line in console_css.lines() {
        let l = line.trim();
        if l.starts_with(":root{") || l.starts_with("--color-") {
            in_def = true;
        }
        if l.starts_with('}') {
            in_def = false;
        }
        // 僅檢查 :root 定義區的色值（規則區的 color-mix 引用不受限）。
        if !in_def {
            continue;
        }
        for tok in l.split([';', '{', '}']) {
            let tok = tok.trim();
            if let Some(hash) = tok.find('#') {
                let hex = &tok[hash..];
                let hex = hex.split(' ').next().unwrap_or(hex);
                let valid = hex.len() == 4 || hex.len() == 7;
                if !valid {
                    continue;
                }
                if !design.to_uppercase().contains(&hex.to_uppercase()) {
                    problems.push(format!("non-canonical color {hex} (line: {tok})"));
                }
            }
        }
    }
    assert!(
        problems.is_empty(),
        "console.css token drift (grill-E 全收斂違反):\n{}",
        problems.join("\n")
    );
    // 舊詞彙 token 不得殘留（W7 收斂完成後的防回歸）。
    for legacy in [
        "--bg:",
        "--accent:",
        "--ink:",
        "--muted:",
        "--s1:",
        "--font:",
    ] {
        assert!(
            !console_css.contains(legacy),
            "legacy token {legacy} survived convergence"
        );
    }
}

/// 憲章 §2.2（每個 .rs 第一行 `#![forbid(unsafe_code)]`）與 ADR-0014：
/// 全工作區 Rust 原始碼不得出現 unsafe 語法，且每個 crate 根必須以
/// `#![forbid(unsafe_code)]` 讓編譯器強制。
#[test]
fn unsafe_code_is_absent_and_forbidden_at_crate_roots() {
    let root = repo_root();
    let roots = [
        "core/src/lib.rs",
        "platform/src/lib.rs",
        "web/src/lib.rs",
        "crates/console_manifest/src/lib.rs",
        "crates/kem/src/lib.rs",
        "crates/flaps_bridge/src/lib.rs",
        "web/src-tauri/src/lib.rs",
        "web/src-tauri/src/main.rs",
    ];
    let mut problems = Vec::new();
    for rel in roots {
        let path = root.join(rel);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        if !text.contains("#![forbid(unsafe_code)]") {
            problems.push(format!("{rel}: missing #![forbid(unsafe_code)]"));
        }
    }

    let mut files = Vec::new();
    for dir in [
        "core/src",
        "platform/src",
        "crates",
        "web/src",
        "web/src-tauri/src",
    ] {
        collect_text_files(&root.join(dir), &mut files);
    }
    for file in &files {
        if file.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("read {}: {e}", file.display()));
        for (idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue; // 註解中的散文不計
            }
            let is_unsafe = trimmed == "unsafe"
                || trimmed.starts_with("unsafe {")
                || trimmed.starts_with("unsafe fn")
                || trimmed.starts_with("unsafe impl")
                || trimmed.starts_with("unsafe trait")
                || trimmed.starts_with("unsafe extern");
            if is_unsafe {
                problems.push(format!(
                    "{}:{} unsafe syntax",
                    file.strip_prefix(&root).unwrap_or(file).display(),
                    idx + 1
                ));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "unsafe 邊界違反（ADR-0014）：\n{}",
        problems.join("\n")
    );
}
