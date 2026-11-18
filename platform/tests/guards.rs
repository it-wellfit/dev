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
        } else if let Some(ext) = path.extension().and_then(|s| s.to_str())
            && TEXT_EXTENSIONS.contains(&ext)
        {
            out.push(path);
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

/// 盲區補洞（2026-11-15 瀏覽器實證）：`templates_have_no_inline_styles` 只掃
/// platform/templates，但品牌 404 頁等「內聯 HTML 字串」住在 platform/src 的
/// .rs 常數裡（not_found_page 的 `style="text-align:center"` 曾漏網，瀏覽器
/// 以 CSP 錯誤實證）。本測試把同一契約延伸到 .rs 內聯 HTML：
/// 排除註解行與測試斷言行（`contains(` 之 needle 非屬性）。
#[test]
fn rust_inline_html_has_no_inline_styles() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_text_files(&root.join("platform/src"), &mut files);
    collect_text_files(&root.join("web/src-tauri/src"), &mut files);
    assert!(
        files.len() >= 20,
        "guard must scan the real tree (found {} files)",
        files.len()
    );
    let mut problems = Vec::new();
    for file in &files {
        if file.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(file).unwrap_or_default();
        for (idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("///") {
                continue; // 註解散文不計
            }
            if trimmed.contains("contains(") {
                continue; // 測試斷言之 needle，非 HTML 屬性
            }
            if trimmed.contains("style=\"") {
                problems.push(format!(
                    "{}:{} inline style attribute (use a design-system class)",
                    file.strip_prefix(&root).unwrap_or(file).display(),
                    idx + 1
                ));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "inline style in .rs HTML strings violates CSP style-src 'self':\n{}",
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
fn console_css_is_byte_identical_to_living_ledger_ssot() {
    // W7（2026-11-17 修訂）：CSS 單一來源護欄——console.css 必須與
    // docs/design/living-ledger.css 逐字一致（SSOT；取代舊 token 名稱
    // 啟發式與 canonical 色值比對——Living Ledger 設計語言已將
    // --bg/--accent/--ink 列為合法 token）。
    // 〔C-01 舊規則歷史〕grill-E 全收斂的 token 名稱啟發式
    // （console_css_tokens_are_canonical_design_system_values：自造色值
    // 逐字存在於 design-system.css＋legacy 黑名單）於 2026-11-12 前有效，
    // 由本規則（逐字 SSOT，強度更高）取代。
    let root = repo_root();
    let console_css = fs::read_to_string(root.join("knowledge/console/assets/console.css"))
        .expect("console.css must exist");
    let canonical = fs::read_to_string(root.join("docs/design/living-ledger.css"))
        .expect("living-ledger.css must exist");
    assert_eq!(
        console_css, canonical,
        "console.css 與 living-ledger.css 不一致（SSOT 漂移）——\
         請執行 cargo run -p kem -- --root knowledge console 重新生成"
    );
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

/// XP-002（G4.12）列印樣式閘：`design-system.css` 必須具備列印頁首頁尾與分頁樣式
/// （`@page`、列印頁首/頁尾、`.print-header`/`.print-footer`、`@media print`），
/// 否則報表列印會缺頁首頁尾或分頁資訊（整合計畫 III-2）。
#[test]
fn design_system_has_print_header_footer_and_pagination() {
    let root = repo_root();
    let css_path = root.join("web/assets/design-system.css");
    let text = std::fs::read_to_string(&css_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", css_path.display()));
    let mut problems = Vec::new();
    for needle in ["@page", "@media print", ".print-header", ".print-footer"] {
        if !text.contains(needle) {
            problems.push(format!("缺少 {needle}"));
        }
    }
    // 文件快照必須與正式資產同步（single source of truth）。
    let doc_snapshot = root.join("docs/design-system.css");
    if doc_snapshot.is_file() {
        let doc = std::fs::read_to_string(&doc_snapshot).expect("read docs snapshot");
        if doc != text {
            problems
                .push("docs/design-system.css 與 web/assets/design-system.css 未同步".to_string());
        }
    }
    assert!(
        problems.is_empty(),
        "列印樣式閘失敗：{}",
        problems.join("；")
    );
}
