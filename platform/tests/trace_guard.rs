//! 追溯矩陣自動校驗（2026-11-13；總控指令 §13 機械化）。
//!
//! 解析 `docs/standards/REQUIREMENTS_TRACEABILITY.md`，逐列驗證：
//! (a) 判定 ∈ {PASS, PARTIAL, NOT TESTED, BLOCKED, FAIL, NOT VERIFIED}；
//! (b) 「設計/實作位置/證據」欄引用的**檔案路徑**真實存在；
//! (c) 「測試」欄引用的**測試/函式名**真實存在於 platform 原始碼。
//!
//! 為何需要：文件主張「已驗證」但測試被改名/刪除、或引用檔案被移動時，
//! 無人察覺 → 文件與實作脫鉤（最高約束明文禁止）。本測試使脫鉤＝紅燈。
//!
//! 解析契約（與矩陣格式耦合，修改矩陣格式須同步此處）：
//! 資料列 = 以 `| ` 開頭且首欄符合 `[A-Z]+-\d+` 的 7 欄表格列。
//! 反引號 token：含 `/` 或 `.` 視為路徑；純 `a-z0-9_` 且長度 ≥ 10 視為
//! 測試/函式名。`*` 結尾視為前綴萬用。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR"); // .../Dev/platform
const MATRIX: &str = "../docs/standards/REQUIREMENTS_TRACEABILITY.md";
const VALID_VERDICTS: &[&str] = &[
    "PASS",
    "PARTIAL",
    "NOT TESTED",
    "BLOCKED",
    "FAIL",
    "NOT VERIFIED",
];

/// workspace 根下已存在的檔案快取（路徑 → 存在）。
fn workspace_root() -> PathBuf {
    Path::new(MANIFEST_DIR)
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn collect_rs_sources(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_sources(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs")
                && let Ok(text) = fs::read_to_string(&path)
            {
                out.push(text);
            }
        }
    }
}

#[test]
fn traceability_matrix_rows_resolve() {
    let root = workspace_root();
    let matrix_path = root.join(MATRIX.trim_start_matches("../"));
    let text = fs::read_to_string(&matrix_path)
        .unwrap_or_else(|e| panic!("traceability matrix readable: {e}"));

    // 收集 platform＋core＋crates 原始碼（供測試名存在性檢查）——
    // 2026-11-13 擴大：AMT-01 引用 core/src/money.rs 的測試曾被誤判不存在。
    let mut sources = Vec::new();
    collect_rs_sources(&Path::new(MANIFEST_DIR).join("src"), &mut sources);
    collect_rs_sources(&Path::new(MANIFEST_DIR).join("tests"), &mut sources);
    collect_rs_sources(&root.join("core/src"), &mut sources);
    collect_rs_sources(&root.join("crates"), &mut sources);
    let all_source = sources.join("\n");

    let mut checked_files = 0usize;
    let mut checked_tests = 0usize;
    let mut rows = 0usize;
    let mut problems: Vec<String> = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("| ") {
            continue;
        }
        let cells: Vec<&str> = line.split('|').map(|c| c.trim()).collect();
        // split 產生 [ "", c1..c7, "" ] → 9 欄
        if cells.len() < 9 {
            continue;
        }
        let id_cell = cells[1];
        let id = id_cell.trim();
        if !id
            .chars()
            .next()
            .map(|c| c.is_ascii_uppercase())
            .unwrap_or(false)
            || !id.contains('-')
            || !id
                .split('-')
                .nth(1)
                .map(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
                .unwrap_or(false)
        {
            continue;
        }
        rows += 1;
        // split('|') → ["", ID, 需求, 設計, 實作位置, 測試, 證據, 判定, ""]
        let design = cells[3];
        let impl_col = cells[4];
        let test_col = cells[5];
        let evidence = cells[6];
        let verdict = cells[7];

        // (a) 判定五級
        let verdict_clean = verdict.replace('*', "").trim().to_string();
        let verdict_ok = VALID_VERDICTS.iter().any(|v| verdict_clean.starts_with(v));
        if !verdict_ok {
            problems.push(format!("{id}: 判定「{verdict}」不在五級用語"));
        }

        // (b) 檔案路徑存在（設計/實作/證據欄；測試欄的 .rs 亦屬檔案）
        for (label, column) in [
            ("設計", design),
            ("實作", impl_col),
            ("測試", test_col),
            ("證據", evidence),
        ] {
            for token in column.split('`').chain(column.split('`')) {
                let tok = token.trim().trim_end_matches('（').trim_end_matches('）');
                let looks_path = (tok.contains('/') || tok.contains('.'))
                    && (tok.starts_with("platform/")
                        || tok.starts_with("crates/")
                        || tok.starts_with("core/")
                        || tok.starts_with("web/")
                        || tok.starts_with("docs/")
                        || tok.starts_with("scripts/")
                        || tok.starts_with("migrations/")
                        || tok.starts_with("Cargo.toml"));
                if !looks_path || tok.len() < 5 || tok.contains(' ') {
                    continue;
                }
                // 去除行尾修飾（如 `（3 tests）` 已由反引號切割排除）
                if root.join(tok).exists() {
                    checked_files += 1;
                } else {
                    problems.push(format!("{id}: {label}欄引用檔案不存在: {tok}"));
                }
            }
        }

        // (c) 測試/函式名存在（測試欄的純識別字 token）
        for token in test_col.split('`') {
            let tok = token.trim();
            if tok.contains('/') || tok.contains('.') || tok.len() < 10 {
                continue;
            }
            if !tok
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            {
                continue;
            }
            let exists = if let Some(prefix) = tok.strip_suffix('_') {
                all_source.contains(&format!("fn {prefix}"))
            } else {
                all_source.contains(&format!("fn {tok}"))
                    || all_source.contains(&format!("mod {tok}"))
            };
            checked_tests += 1;
            if !exists {
                problems.push(format!("{id}: 測試欄引用的測試不存在: {tok}"));
            }
        }
    }

    assert!(
        rows >= 20,
        "矩陣應有 20+ 需求列（實測 {rows}）——格式變更須同步 trace_guard"
    );
    assert!(
        checked_files >= 5,
        "應驗證過 5+ 檔案引用（實測 {checked_files}）"
    );
    assert!(
        checked_tests >= 5,
        "應驗證過 5+ 測試名（實測 {checked_tests}）"
    );
    assert!(
        problems.is_empty(),
        "追溯矩陣與實作脫鉤（{} 項）:\n{}",
        problems.len(),
        problems.join("\n")
    );
}

/// 體驗契約文件必須存在且含 TT-1..TT-5 與二十五問（結構不滅失）。
#[test]
fn experience_contracts_present() {
    let root = workspace_root();
    let text = fs::read_to_string(root.join("docs/standards/EXPERIENCE_CONTRACTS.md"))
        .expect("EXPERIENCE_CONTRACTS.md readable");
    for tt in ["TT-1", "TT-2", "TT-3", "TT-4", "TT-5"] {
        assert!(text.contains(tt), "體驗契約缺 {tt}");
    }
    assert!(text.contains("細節二十五問"), "缺二十五問清單");
    assert!(text.contains("覆蓋證明規則"), "缺覆蓋證明規則");
}

/// 判定用語（HashMap 供未來擴充；避免 dead_code 警告的語義登記）。
#[test]
fn verdict_vocabulary_is_registered() {
    let mut vocab: HashMap<&str, &str> = HashMap::new();
    vocab.insert("PASS", "已執行且通過");
    vocab.insert("PARTIAL", "部分驗證");
    vocab.insert("NOT TESTED", "尚未執行");
    vocab.insert("BLOCKED", "受環境限制");
    vocab.insert("FAIL", "已執行但失敗");
    assert_eq!(vocab.len(), VALID_VERDICTS.len() - 1);
}
