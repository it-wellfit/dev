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
const SCAN_DIRS: [&str; 7] = [
    "core/src",
    "platform/src",
    "platform/templates",
    "platform/migrations",
    "web/assets",
    "web/src-tauri/src",
    "crates/kem/src",
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
