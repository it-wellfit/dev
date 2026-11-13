#![forbid(unsafe_code)]
//! 體驗矩陣自動校驗（計畫 1793708660255 v2.2 §10.2；2026-11-13）。
//!
//! 解析 `docs/design` 體驗五件套＋稽核，使「文件與實作脫鉤＝紅燈」：
//! (a) 追溯矩陣：REQ-UX 列七欄非空、判定五級、NOT TESTED／UNKNOWN 必附解鎖、
//!     反引號 token 解析（路徑存在／測試名或同名檔案存在）；
//! (b) 表面矩陣：S1–S9 九列 × 10 契約格零空格、格值 ∈ {●,◐,—,？}、出現 ？ 必附解鎖；
//! (c) 問題登錄：XP 列八欄非空、ID 唯一、級別集合、UNKNOWN 必附解鎖；
//! (d) 情境矩陣：CP 列六欄非空、CP-01..CP-12 具備；
//! (e) 稽核引用之 XP-IDs 全數已登錄；規格含 EC-1..EC-10 與 TC-1..TC-5。
//!
//! 與 `trace_guard.rs`（標準需求矩陣）分工：本檔只掃體驗文件，互不干擾。

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const VALID_VERDICTS: [&str; 6] = [
    "PASS",
    "PARTIAL",
    "NOT TESTED",
    "BLOCKED",
    "FAIL",
    "NOT VERIFIED",
];
const SURFACE_CELLS: [&str; 4] = ["●", "◐", "—", "？"];
const LEVELS: [&str; 5] = ["P0", "P1", "P2", "P3", "—"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("platform crate 應位於 repo 根之下")
        .to_path_buf()
}

fn read_doc(root: &Path, rel: &str) -> String {
    let path = root.join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} 應可讀取：{e}"))
}

/// 表格列 → 去掉頭尾空欄的 cells。
fn cells(line: &str) -> Vec<String> {
    let mut out: Vec<String> = line.split('|').map(|c| c.trim().to_string()).collect();
    if out.first().is_some_and(|c| c.is_empty()) {
        out.remove(0);
    }
    if out.last().is_some_and(|c| c.is_empty()) {
        out.pop();
    }
    out
}

/// 收集 .rs 原始碼（內容串接）與檔名集合，供測試名/同名檔解析。
fn collect_rs(dir: &Path, texts: &mut String, names: &mut HashSet<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs(&path, texts, names);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    names.insert(name.to_string());
                }
                if let Ok(text) = fs::read_to_string(&path) {
                    texts.push_str(&text);
                    texts.push('\n');
                }
            }
        }
    }
}

/// 反引號 token 解析：含 `/` 或 .md/.rs 結尾＝路徑；純小寫函式名 ≥10＝原始碼/同名檔。
fn resolve_backticks(
    row: &str,
    root: &Path,
    sources: &str,
    names: &HashSet<String>,
    problems: &mut Vec<String>,
) {
    let mut rest = row;
    while let Some(open) = rest.find('`') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('`') else { break };
        let token = &after[..close];
        rest = &after[close + 1..];

        if token.contains('/') || token.ends_with(".md") || token.ends_with(".rs") {
            if token.contains('/') {
                if !root.join(token).exists() {
                    problems.push(format!("token 路徑不存在：{token}"));
                }
            } else if !names.contains(token) {
                problems.push(format!("token 檔名不存在：{token}"));
            }
            continue;
        }
        let pure_name = token.len() >= 10
            && token
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if pure_name && !sources.contains(token) && !names.contains(&format!("{token}.rs")) {
            problems.push(format!("token 測試名/函式名不存在：{token}"));
        }
    }
}

#[test]
fn traceability_rows_are_complete_and_resolvable() {
    let root = repo_root();
    let text = read_doc(&root, "docs/design/EXPERIENCE_TRACEABILITY.md");
    let mut sources = String::new();
    let mut names = HashSet::new();
    collect_rs(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut sources,
        &mut names,
    );
    collect_rs(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests"),
        &mut sources,
        &mut names,
    );
    collect_rs(&root.join("core/src"), &mut sources, &mut names);
    collect_rs(&root.join("crates"), &mut sources, &mut names);

    let mut problems: Vec<String> = Vec::new();
    let mut rows = 0usize;
    let mut ids: HashSet<String> = HashSet::new();

    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("| REQ-UX-") {
            continue;
        }
        rows += 1;
        let c = cells(line);
        if c.len() != 7 {
            problems.push(format!("{line}：欄數 {}≠7", c.len()));
            continue;
        }
        for (i, cell) in c.iter().enumerate() {
            if cell.is_empty() {
                problems.push(format!("{}：第 {i} 欄為空", c[0]));
            }
        }
        if !ids.insert(c[0].clone()) {
            problems.push(format!("{}：ID 重複", c[0]));
        }
        let verdict = c[6].replace('*', "");
        if !VALID_VERDICTS.iter().any(|v| verdict.starts_with(v)) {
            problems.push(format!("{}：判定「{}」不在五級用語", c[0], c[6]));
        }
        if (verdict.contains("NOT TESTED") || line.contains("UNKNOWN")) && !line.contains("解鎖")
        {
            problems.push(format!("{}：NOT TESTED／UNKNOWN 未附解鎖條件", c[0]));
        }
        resolve_backticks(line, &root, &sources, &names, &mut problems);
    }

    assert!(rows >= 20, "追溯列數不足（{rows}），預期 REQ-UX-01..20");
    assert!(
        problems.is_empty(),
        "追溯矩陣問題：\n{}",
        problems.join("\n")
    );
}

#[test]
fn surface_matrix_has_no_empty_cells() {
    let root = repo_root();
    let text = read_doc(&root, "docs/design/EXPERIENCE_SURFACE_MATRIX.md");
    let mut problems: Vec<String> = Vec::new();
    let mut rows = 0usize;
    let mut has_unknown = false;

    for line in text.lines() {
        let line = line.trim();
        let is_row = line
            .strip_prefix("| S")
            .is_some_and(|r| r.chars().next().is_some_and(|c| c.is_ascii_digit()));
        if !is_row {
            continue;
        }
        rows += 1;
        let c = cells(line);
        if c.len() != 11 {
            problems.push(format!("{line}：欄數 {}≠11（表面＋10 契約）", c.len()));
            continue;
        }
        for cell in &c[1..] {
            if !SURFACE_CELLS.contains(&cell.as_str()) {
                problems.push(format!("{}：格值「{cell}」不在 {{●,◐,—,？}}", c[0]));
            }
            if cell == "？" {
                has_unknown = true;
            }
        }
    }
    if rows != 9 {
        problems.push(format!("表面列數 {rows}≠9"));
    }
    if has_unknown && !text.contains("解鎖") {
        problems.push("出現 ？ 但文件未載解鎖條件".to_string());
    }
    assert!(
        problems.is_empty(),
        "表面矩陣問題：\n{}",
        problems.join("\n")
    );
}

#[test]
fn problem_register_rows_are_complete() {
    let root = repo_root();
    let text = read_doc(&root, "docs/design/EXPERIENCE_PROBLEM_REGISTER.md");
    let mut problems: Vec<String> = Vec::new();
    let mut ids: HashSet<String> = HashSet::new();

    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("| XP-") {
            continue;
        }
        let c = cells(line);
        if c.len() != 8 {
            problems.push(format!("{line}：欄數 {}≠8", c.len()));
            continue;
        }
        for (i, cell) in c.iter().enumerate() {
            if cell.is_empty() {
                problems.push(format!("{}：第 {i} 欄為空", c[0]));
            }
        }
        if !ids.insert(c[0].clone()) {
            problems.push(format!("{}：ID 重複", c[0]));
        }
        if !LEVELS.contains(&c[3].as_str()) {
            problems.push(format!("{}：級別「{}」不在集合", c[0], c[3]));
        }
        if c[6].contains("UNKNOWN") && !c[5].contains("解鎖") {
            problems.push(format!("{}：UNKNOWN 未附解鎖條件", c[0]));
        }
    }
    assert!(!ids.is_empty(), "問題登錄為空");
    assert!(
        problems.is_empty(),
        "問題登錄問題：\n{}",
        problems.join("\n")
    );
}

#[test]
fn scenario_key_paths_are_complete() {
    let root = repo_root();
    let text = read_doc(&root, "docs/design/EXPERIENCE_SCENARIO_MATRIX.md");
    let mut problems: Vec<String> = Vec::new();
    let mut rows = 0usize;

    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("| CP-") {
            continue;
        }
        rows += 1;
        let c = cells(line);
        if c.len() != 6 {
            problems.push(format!("{line}：欄數 {}≠6", c.len()));
            continue;
        }
        for (i, cell) in c.iter().enumerate() {
            if cell.is_empty() {
                problems.push(format!("{}：第 {i} 欄為空", c[0]));
            }
        }
    }
    if rows < 12 {
        problems.push(format!("關鍵路徑列數 {rows}<12"));
    }
    for marker in ["CP-01", "CP-12"] {
        if !text.contains(marker) {
            problems.push(format!("缺少 {marker}"));
        }
    }
    assert!(
        problems.is_empty(),
        "情境矩陣問題：\n{}",
        problems.join("\n")
    );
}

#[test]
fn audit_xp_ids_are_registered_and_spec_has_contracts() {
    let root = repo_root();
    let audit = read_doc(&root, "docs/design/EXPERIENCE_AUDIT_v1.md");
    let register = read_doc(&root, "docs/design/EXPERIENCE_PROBLEM_REGISTER.md");
    let spec = read_doc(&root, "docs/design/EXPERIENCE_SPEC.md");
    let mut problems: Vec<String> = Vec::new();

    // 稽核文內 XP-### 全數已登錄。
    let mut rest = audit.as_str();
    while let Some(pos) = rest.find("XP-") {
        let tail = &rest[pos + 3..];
        let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.len() == 3 {
            let id = format!("XP-{digits}");
            if !register.contains(&id) {
                problems.push(format!("稽核引用 {id} 未登錄"));
            }
        }
        rest = &rest[pos + 3..];
    }
    // 規格必載契約全集。
    for ec in 1..=10 {
        let marker = format!("EC-{ec}");
        if !spec.contains(&marker) {
            problems.push(format!("規格缺少 {marker}"));
        }
    }
    for tc in 1..=5 {
        let marker = format!("TC-{tc}");
        if !spec.contains(&marker) {
            problems.push(format!("規格缺少 {marker}"));
        }
    }
    assert!(
        problems.is_empty(),
        "引用/契約檢查問題：\n{}",
        problems.join("\n")
    );
}
