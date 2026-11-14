#![forbid(unsafe_code)]
//! G4.6 (D2-16) — static accessibility gate for server-rendered templates.
//!
//! Mechanical subset of `docs/standards/A11Y_CHECKLIST.md`: doctype/lang,
//! viewport, `main#main` landmark, an `<h1>`, label↔control binding, and
//! `img` alternatives. Contrast (item 7) is covered by `design_a11y.rs`.

use std::path::{Path, PathBuf};

fn templates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn template_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(templates_dir()).expect("read templates dir");
    for entry in rd.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("html") {
            out.push(path);
        }
    }
    out.sort();
    out
}

/// Extract the target of every `<label for="X">` in the source.
fn label_targets(src: &str) -> Vec<String> {
    let needle = "<label for=\"";
    let mut out = Vec::new();
    let mut rest = src;
    while let Some(pos) = rest.find(needle) {
        let after = &rest[pos + needle.len()..];
        let Some(end) = after.find('"') else { break };
        out.push(after[..end].to_string());
        rest = &after[end..];
    }
    out
}

#[test]
fn templates_declare_language_and_viewport() {
    let files = template_files();
    assert!(!files.is_empty(), "no templates found");
    for path in &files {
        let src = std::fs::read_to_string(path).expect("read template");
        let name = path.display();
        assert!(
            src.contains("<!DOCTYPE html>"),
            "{name}: missing <!DOCTYPE html>"
        );
        assert!(
            src.contains("lang=\"zh-Hant"),
            "{name}: missing zh-Hant language declaration"
        );
        assert!(
            src.contains("name=\"viewport\"") && src.contains("width=device-width"),
            "{name}: missing responsive viewport meta"
        );
    }
}

#[test]
fn landmarks_present() {
    for path in template_files() {
        let src = std::fs::read_to_string(&path).expect("read template");
        assert!(
            src.contains("<main id=\"main\""),
            "{}: missing <main id=\"main\"> landmark",
            path.display()
        );
    }
}

#[test]
fn h1_present() {
    for path in template_files() {
        let src = std::fs::read_to_string(&path).expect("read template");
        assert!(
            src.contains("<h1"),
            "{}: page must have exactly one h1 (at least one required)",
            path.display()
        );
    }
}

#[test]
fn labels_bind_controls() {
    for path in template_files() {
        let src = std::fs::read_to_string(&path).expect("read template");
        for target in label_targets(&src) {
            assert!(
                src.contains(&format!("id=\"{target}\"")),
                "{}: label for=\"{target}\" has no matching control id",
                path.display()
            );
        }
    }
}

#[test]
fn images_have_alt() {
    for path in template_files() {
        let src = std::fs::read_to_string(&path).expect("read template");
        let mut rest: &str = &src;
        while let Some(pos) = rest.find("<img") {
            let after = &rest[pos..];
            let end = after.find('>').map(|i| pos + i + 1).unwrap_or(src.len());
            let tag = &src[pos..end];
            assert!(
                tag.contains("alt="),
                "{}: <img> without alt: {tag}",
                path.display()
            );
            rest = &src[end..];
        }
    }
}
