#![forbid(unsafe_code)]
//! T4d 負向測試（計畫 §9.5）：僅在 temp 複本注入，不得動真檔。
//! 每個案例：注入 → 期望對應檢查 FAIL；其中 (i)(ii)(v) 追加「還原 → 回綠」全程重現。

use std::fs;
use std::path::{Path, PathBuf};

use memory_lint::{lint, Outcome};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .expect("repo root")
        .to_path_buf()
}

fn build_fixture(tag: &str) -> PathBuf {
    let root = repo_root();
    let tmp = std::env::temp_dir().join(format!("kilo-memory-lint-neg-{tag}"));
    if tmp.exists() {
        fs::remove_dir_all(&tmp).expect("clean tmp");
    }
    let mem = tmp.join(".kilo").join("memories");
    let plans = tmp.join(".kilo").join("plans");
    let adr = tmp.join("docs").join("adr");
    fs::create_dir_all(&mem).expect("mkdir mem");
    fs::create_dir_all(&plans).expect("mkdir plans");
    fs::create_dir_all(&adr).expect("mkdir adr");
    for f in ["corrections.md", "project.md", "decisions.md", "README.md"] {
        fs::copy(root.join(".kilo").join("memories").join(f), mem.join(f)).expect("copy mem");
    }
    for ent in fs::read_dir(root.join(".kilo").join("plans")).expect("plans dir") {
        let ent = ent.expect("ent");
        let name = ent.file_name().to_string_lossy().to_string();
        if (name.starts_with("1792865422970-") || name.starts_with("1792461065184-"))
            && name.ends_with(".md")
        {
            fs::copy(ent.path(), plans.join(&name)).expect("copy plan");
        }
    }
    for ent in fs::read_dir(root.join("docs").join("adr")).expect("adr dir") {
        let ent = ent.expect("ent");
        let name = ent.file_name().to_string_lossy().to_string();
        if name.starts_with("ADR-") && name.ends_with(".md") {
            fs::copy(ent.path(), adr.join(&name)).expect("copy adr");
        }
    }
    tmp
}

fn run(tmp: &Path) -> Outcome {
    lint(tmp).expect("lint 不應環境錯誤")
}

fn has_fail(o: &Outcome, check: &str, needle: &str) -> bool {
    o.lines
        .iter()
        .any(|l| l.starts_with(&format!("FAIL [{check}]")) && l.contains(needle))
}

fn inject(tmp: &Path, file: &str, old: &str, new: &str) {
    let p = tmp.join(".kilo").join("memories").join(file);
    let t = fs::read_to_string(&p).expect("read");
    let n = t.matches(old).count();
    assert_eq!(n, 1, "anchor 需唯一於 {file}：{old:?}（實得 {n}）");
    fs::write(&p, t.replacen(old, new, 1)).expect("write");
}

fn restore(tmp: &Path, file: &str) {
    let root = repo_root();
    fs::copy(
        root.join(".kilo").join("memories").join(file),
        tmp.join(".kilo").join("memories").join(file),
    )
    .expect("restore");
}

fn dump(o: &Outcome) -> String {
    o.lines.join("\n")
}

#[test]
fn baseline_fresh_copy_green() {
    let tmp = build_fixture("baseline");
    let o = run(&tmp);
    assert_eq!(o.fail_count, 0, "baseline 應全綠：\n{}", dump(&o));
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn case_i_duplicate_c_id_fails_l1_and_restores_green() {
    let tmp = build_fixture("case-i");
    inject(&tmp, "corrections.md", "| C-02 |", "| C-01 |");
    let o = run(&tmp);
    assert!(
        has_fail(&o, "L1", "連續且唯一"),
        "重號應觸發 L1 FAIL：\n{}",
        dump(&o)
    );
    restore(&tmp, "corrections.md");
    let o2 = run(&tmp);
    assert_eq!(o2.fail_count, 0, "還原後應回綠：\n{}", dump(&o2));
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn case_ii_bad_date_fails_l2_and_restores_green() {
    let tmp = build_fixture("case-ii");
    inject(&tmp, "project.md", "§3.1 | 2026-11-09", "§3.1 | 2026/11/09");
    let o = run(&tmp);
    assert!(
        has_fail(&o, "L2", "覆核日期格式錯誤"),
        "壞日期應觸發 L2 FAIL：\n{}",
        dump(&o)
    );
    restore(&tmp, "project.md");
    let o2 = run(&tmp);
    assert_eq!(o2.fail_count, 0, "還原後應回綠：\n{}", dump(&o2));
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn case_iii_dangling_ref_fails_l5() {
    let tmp = build_fixture("case-iii");
    inject(
        &tmp,
        "corrections.md",
        "`§24` 修訂紀錄完整且舊文仍存",
        "`§24`、`§99` 修訂紀錄完整且舊文仍存",
    );
    let o = run(&tmp);
    assert!(
        has_fail(&o, "L5", "§99"),
        "懸空 §99 應觸發 L5 FAIL：\n{}",
        dump(&o)
    );
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn case_iv_decisions_column_error_fails_l4() {
    let tmp = build_fixture("case-iv");
    inject(&tmp, "decisions.md", "| 兩表合併為一 |", "|");
    let o = run(&tmp);
    assert!(
        has_fail(&o, "L4", "欄數 6 ≠ 7"),
        "缺欄應觸發 L4 FAIL：\n{}",
        dump(&o)
    );
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn case_v_s0n_form_fails_l5_and_restores_green() {
    let tmp = build_fixture("case-v");
    inject(
        &tmp,
        "corrections.md",
        "`§24` 修訂紀錄完整且舊文仍存",
        "`§24`、`§0.9` 修訂紀錄完整且舊文仍存",
    );
    let o = run(&tmp);
    assert!(
        has_fail(&o, "L5", "§0.N 形式已廢除"),
        "§0.N 形式應觸發 L5 FAIL：\n{}",
        dump(&o)
    );
    restore(&tmp, "corrections.md");
    let o2 = run(&tmp);
    assert_eq!(o2.fail_count, 0, "還原後應回綠：\n{}", dump(&o2));
    let _ = fs::remove_dir_all(&tmp);
}
