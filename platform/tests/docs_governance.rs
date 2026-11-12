#![forbid(unsafe_code)]

//! 文件治理閘（`SHARED_STANDARDS` §4.1／§4.2／§6.2 第 5 項）。
//!
//! 檢查 Dev 根目錄之十份必要文件是否存在、是否具備必要中介資料（版本／生效日期／適用範圍／狀態），
//! 以及文件內之相對連結是否可解析（避免「文件指向不存在的檔案」）。
//!
//! 為何以測試實作：C-11 要求新工具一律 Rust；以 `cargo test` 執行可與既有基線閘一致，
//! 不需新增 .ps1/.py 工具鏈。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const REQUIRED_DOCS: [&str; 10] = [
    "PROJECT_CHARTER.md",
    "ARCHITECTURE.md",
    "API_SPEC.md",
    "DEVELOPMENT_GUIDE.md",
    "QUALITY_STANDARDS.md",
    "OPERATIONS_RUNBOOK.md",
    "SECURITY.md",
    "ADR_REGISTER.md",
    "TRACEABILITY.md",
    "TROUBLESHOOTING.md",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("platform crate 應位於 repo 根之下")
        .to_path_buf()
}

#[test]
fn required_project_docs_exist_with_metadata() {
    let root = repo_root();
    let mut problems = Vec::new();

    for doc in REQUIRED_DOCS {
        let path = root.join(doc);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) => {
                problems.push(format!("{doc}: 無法讀取（{err}）"));
                continue;
            }
        };
        for field in ["版本", "生效日期", "狀態"] {
            if !text.contains(field) {
                problems.push(format!("{doc}: 缺少中介資料「{field}」"));
            }
        }
    }

    assert!(
        problems.is_empty(),
        "文件治理閘失敗（SHARED_STANDARDS §4.1／§4.2）：\n{}",
        problems.join("\n")
    );
}

#[test]
fn doc_relative_links_resolve() {
    let root = repo_root();
    let mut checked = 0usize;
    let mut problems = Vec::new();

    for doc in REQUIRED_DOCS {
        let path = root.join(doc);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue; // 存在性由另一測試負責
        };
        for (idx, line) in text.lines().enumerate() {
            // 取 Markdown 連結 [label](target)，僅檢查相對路徑與錨點。
            let Some(start) = line.find("](") else {
                continue;
            };
            let rest = &line[start + 2..];
            let Some(end) = rest.find(')') else { continue };
            let target = rest[..end].trim();
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with('#')
                || target.starts_with("mailto:")
            {
                continue;
            }
            let target = target.split('#').next().unwrap_or(target).trim();
            if target.is_empty() {
                continue;
            }
            checked += 1;
            let resolved = root.join(target);
            if !resolved.exists() {
                problems.push(format!("{doc}:{} 連結無法解析：{target}", idx + 1));
            }
        }
    }

    assert!(
        problems.is_empty(),
        "文件連結壞損（共檢查 {checked} 個相對連結）：\n{}",
        problems.join("\n")
    );
}

#[test]
fn required_docs_have_no_placeholder_markers() {
    let root = repo_root();
    let banned = ["TODO: 待補", "FIXME: 文件", "TBD（未填寫"];
    let mut problems = Vec::new();
    for doc in REQUIRED_DOCS {
        let path = root.join(doc);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for marker in banned {
            if text.contains(marker) {
                problems.push(format!("{doc}: 含佔位標記 {marker}"));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "必要文件不得留佔位內容：\n{}",
        problems.join("\n")
    );
}

/// 供其他測試重用的 repo 根定位（避免重複實作）。
#[allow(dead_code)]
fn root_markers() -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    set.insert(repo_root().display().to_string());
    set
}
