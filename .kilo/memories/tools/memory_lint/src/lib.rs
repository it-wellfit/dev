#![forbid(unsafe_code)]
//! memory_lint — 專案記憶庫結構與引用完整性檢查（記憶庫 v2.0）
//!
//! 規格來源：`.kilo/plans/1792461065184-project-memory-lessons.md` §9.1
//! 範圍：僅掃描 `.kilo/memories/` 四檔（corrections.md／project.md／decisions.md／README.md）；
//! 驗證 §-引用之存在性（主計畫 1792865422970、本計畫 1792461065184、ADR 檔）。
//! 特性：純讀、離線、零依賴（std only）、確定性輸出。
//! 輸出：`PASS|WARN|FAIL [L#] <file>:<line> <message>`；結尾 summary。
//! 退出碼：0＝無 FAIL（WARN 允許）；1＝有 FAIL；2＝環境錯誤（目標檔缺失／解析失敗）。

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const FILES: [&str; 4] = ["corrections.md", "project.md", "decisions.md", "README.md"];
const CHECKS: [&str; 8] = ["L1", "L2", "L3", "L4", "L5", "L5b", "L6", "L7"];
const MAIN_ID: &str = "1792865422970";
const THIS_ID: &str = "1792461065184";
const SIZE_CORRECTIONS_MAX: u64 = 8 * 1024;
const SIZE_ALWAYS_LOAD_MAX: u64 = 12 * 1024;
const STALE_DAYS: i64 = 30;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Fail,
    Warn,
}

pub struct Issue {
    pub level: Level,
    pub check: &'static str,
    pub loc: String,
    pub msg: String,
}

#[derive(Debug)]
pub struct LintError(pub String);

impl std::fmt::Display for LintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for LintError {}

pub struct Outcome {
    pub lines: Vec<String>,
    pub fail_count: usize,
}

type Issues = HashMap<&'static str, Vec<Issue>>;

enum Target {
    Main,
    This,
    Adr(String),
    Info,
}

enum Cell {
    Md(String),
    Adr(String),
    Main,
    This,
    Info,
}

fn add(issues: &mut Issues, level: Level, check: &'static str, loc: String, msg: String) {
    issues.entry(check).or_default().push(Issue {
        level,
        check,
        loc,
        msg,
    });
}

fn read_text(p: &Path) -> Result<String, LintError> {
    let bytes = fs::read(p).map_err(|e| LintError(format!("{}: {e}", p.display())))?;
    let bytes = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        &bytes[..]
    };
    String::from_utf8(bytes.to_vec())
        .map_err(|e| LintError(format!("{}: 非 UTF-8（{e}）", p.display())))
}

fn cells_of(line: &str) -> Vec<String> {
    let s = line.trim();
    let s = s.strip_prefix('|').unwrap_or(s);
    let s = s.strip_suffix('|').unwrap_or(s);
    s.split('|').map(|c| c.trim().to_string()).collect()
}

fn is_separator(s: &str) -> bool {
    let t = s.trim();
    if !(t.starts_with('|') && t.ends_with('|') && t.len() >= 3) {
        return false;
    }
    let mut has_dash = false;
    for c in t.chars() {
        match c {
            '|' | ':' => {}
            '-' => has_dash = true,
            ' ' | '\t' => {}
            _ => return false,
        }
    }
    has_dash
}

fn cid_num(s: &str) -> Option<u32> {
    let rest = s.strip_prefix("C-")?;
    if rest.len() == 2 && rest.bytes().all(|b| b.is_ascii_digit()) {
        rest.parse().ok()
    } else {
        None
    }
}

fn did_num(s: &str) -> Option<u32> {
    let rest = s.strip_prefix("D-")?;
    if rest.len() == 2 && rest.bytes().all(|b| b.is_ascii_digit()) {
        rest.parse().ok()
    } else {
        None
    }
}

// ---------- §-引用掃描 ----------

fn scan_refs(line: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut it = line.char_indices().peekable();
    while let Some((idx, c)) = it.next() {
        if c != '§' {
            continue;
        }
        let mut num = String::new();
        while let Some(&(_, d)) = it.peek() {
            if d.is_ascii_digit() {
                num.push(d);
                it.next();
            } else {
                break;
            }
        }
        if num.is_empty() {
            continue;
        }
        loop {
            let mut look = it.clone();
            let dot = matches!(look.next(), Some((_, '.')));
            if dot {
                let digit = matches!(look.next(), Some((_, d)) if d.is_ascii_digit());
                if digit {
                    num.push('.');
                    it.next();
                    while let Some(&(_, d2)) = it.peek() {
                        if d2.is_ascii_digit() {
                            num.push(d2);
                            it.next();
                        } else {
                            break;
                        }
                    }
                    continue;
                }
            }
            break;
        }
        out.push((idx, num));
    }
    out
}

fn scan_hyphen(line: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut it = line.char_indices().peekable();
    while let Some((idx, c)) = it.next() {
        if c != '§' {
            continue;
        }
        let mut d1 = String::new();
        while let Some(&(_, d)) = it.peek() {
            if d.is_ascii_digit() {
                d1.push(d);
                it.next();
            } else {
                break;
            }
        }
        if d1.is_empty() {
            continue;
        }
        match it.peek() {
            Some(&(_, '-')) => {
                it.next();
            }
            _ => continue,
        }
        let mut d2 = String::new();
        while let Some(&(_, d)) = it.peek() {
            if d.is_ascii_digit() {
                d2.push(d);
                it.next();
            } else {
                break;
            }
        }
        if d2.is_empty() {
            continue;
        }
        out.push((idx, format!("§{d1}-{d2}")));
    }
    out
}

// ---------- 目標判定（逐引用取「該引用之前最近的目標標記」） ----------

fn nearest_adjacent_md(prefix: &str) -> Option<(usize, &str)> {
    let mut end = prefix.len();
    while end > 0 {
        let c = prefix[..end].chars().next_back()?;
        if matches!(c, ' ' | '\t' | '`' | '）' | '」' | '》' | '\r') {
            end -= c.len_utf8();
        } else {
            break;
        }
    }
    if end < 3 || !prefix[..end].ends_with(".md") {
        return None;
    }
    let mut start = end - 3;
    while start > 0 {
        let c = match prefix[..start].chars().next_back() {
            Some(c) => c,
            None => break,
        };
        if c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '\\' | '-') {
            start -= c.len_utf8();
        } else {
            break;
        }
    }
    Some((end, &prefix[start..end]))
}

fn last_adr(s: &str) -> Option<(usize, String)> {
    let mut res = None;
    for (pos, _) in s.match_indices("ADR-") {
        let after = pos + 4;
        let num: String = s[after..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .take(4)
            .collect();
        if num.len() == 4 {
            res = Some((after + 4, num));
        }
    }
    res
}

fn adr_num_in(s: &str) -> Option<String> {
    last_adr(s).map(|(_, n)| n)
}

fn target_for(prefix: &str) -> Target {
    let mut best_end: Option<usize> = None;
    let mut best: Option<Cell> = None;

    fn consider(best_end: &mut Option<usize>, best: &mut Option<Cell>, end: usize, cell: Cell) {
        let take = match *best_end {
            None => true,
            Some(e) => end > e,
        };
        if take {
            *best_end = Some(end);
            *best = Some(cell);
        }
    }

    if let Some((end, path)) = nearest_adjacent_md(prefix) {
        consider(&mut best_end, &mut best, end, Cell::Md(path.to_string()));
    }
    if let Some((end, num)) = last_adr(prefix) {
        consider(&mut best_end, &mut best, end, Cell::Adr(num));
    }
    if let Some(p) = prefix.rfind(MAIN_ID) {
        consider(&mut best_end, &mut best, p + MAIN_ID.len(), Cell::Main);
    }
    if let Some(p) = prefix.rfind(THIS_ID) {
        consider(&mut best_end, &mut best, p + THIS_ID.len(), Cell::This);
    }
    if let Some(p) = prefix.rfind("憲章") {
        consider(&mut best_end, &mut best, p + "憲章".len(), Cell::Info);
    }

    match best {
        None => Target::Main,
        Some(Cell::Md(path)) => {
            if let Some(num) = adr_num_in(&path) {
                Target::Adr(num)
            } else if path.contains(MAIN_ID) {
                Target::Main
            } else if path.contains(THIS_ID) {
                Target::This
            } else {
                Target::Info
            }
        }
        Some(Cell::Adr(num)) => Target::Adr(num),
        Some(Cell::Main) => Target::Main,
        Some(Cell::This) => Target::This,
        Some(Cell::Info) => Target::Info,
    }
}

// ---------- 標題解析 ----------

fn take_num_dot(s: &str) -> String {
    let mut out = String::new();
    let mut prev_digit = false;
    for c in s.chars() {
        if c.is_ascii_digit() {
            out.push(c);
            prev_digit = true;
        } else if c == '.' && prev_digit {
            out.push(c);
            prev_digit = false;
        } else {
            break;
        }
    }
    while out.ends_with('.') {
        out.pop();
    }
    out
}

fn leading_chapter(s: &str) -> Option<String> {
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    if s[digits.len()..].starts_with('.') {
        Some(digits)
    } else {
        None
    }
}

fn parse_main_headings(text: &str) -> (HashSet<String>, HashSet<String>) {
    let mut top = HashSet::new();
    let mut sub = HashSet::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## §") {
            let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !num.is_empty() {
                top.insert(num);
                continue;
            }
        }
        let hashes = line.chars().take_while(|&c| c == '#').count();
        if (3..=6).contains(&hashes) {
            let after = line[hashes..].trim_start();
            let after = after.strip_prefix('§').unwrap_or(after);
            let num = take_num_dot(after);
            if !num.is_empty() {
                sub.insert(num);
            }
        }
    }
    (top, sub)
}

fn parse_this_headings(text: &str) -> (HashSet<String>, HashSet<String>) {
    let mut top = HashSet::new();
    let mut sub = HashSet::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(num) = leading_chapter(rest) {
                top.insert(num);
            }
        } else if let Some(rest) = line.strip_prefix("### ") {
            let num = take_num_dot(rest);
            if !num.is_empty() {
                sub.insert(num);
            }
        }
    }
    (top, sub)
}

fn parse_adr_chapters(text: &str) -> BTreeMap<String, Vec<u32>> {
    let mut chapters: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    let mut cur: Option<String> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            cur = leading_chapter(rest);
            if let Some(c) = &cur {
                chapters.entry(c.clone()).or_default();
            }
            continue;
        }
        if let Some(c) = &cur {
            let digits: String = line.chars().take_while(|ch| ch.is_ascii_digit()).collect();
            if !digits.is_empty() && line[digits.len()..].starts_with(". ") {
                if let Ok(n) = digits.parse::<u32>() {
                    if let Some(v) = chapters.get_mut(c) {
                        v.push(n);
                    }
                }
            }
        }
    }
    chapters
}

fn pick_plan(
    dir: &Path,
    id: &str,
    marker: &dyn Fn(&str) -> bool,
) -> Result<Option<PathBuf>, LintError> {
    let rd = fs::read_dir(dir).map_err(|e| LintError(format!("{}: {e}", dir.display())))?;
    let mut marked: Vec<(PathBuf, u64)> = Vec::new();
    for ent in rd {
        let ent = ent.map_err(|e| LintError(format!("{}: {e}", dir.display())))?;
        let name = ent.file_name().to_string_lossy().to_string();
        if name.starts_with(id) && name.ends_with(".md") {
            let p = ent.path();
            let text = read_text(&p)?;
            if marker(&text) {
                let sz = fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                marked.push((p, sz));
            }
        }
    }
    Ok(marked.into_iter().max_by_key(|(_, sz)| *sz).map(|(p, _)| p))
}

// ---------- 日期 ----------

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn parse_date(s: &str) -> Option<(i32, u32, u32)> {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let y: i32 = s[0..4].parse().ok()?;
    let m: u32 = s[5..7].parse().ok()?;
    let d: u32 = s[8..10].parse().ok()?;
    if !(1..=12).contains(&m) || d < 1 || d > days_in_month(y, m) {
        return None;
    }
    Some((y, m, d))
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y: i64 = if month <= 2 {
        (year - 1) as i64
    } else {
        year as i64
    };
    let era: i64 = if y >= 0 { y } else { y - 399 } / 400;
    let yoe: i64 = y - era * 400;
    let mp: i64 = if month > 2 {
        (month - 3) as i64
    } else {
        (month + 9) as i64
    };
    let doy: i64 = (153 * mp + 2) / 5 + (day as i64) - 1;
    let doe: i64 = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn today_days() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64 / 86400)
        .unwrap_or(0)
}

// ---------- 檢查 ----------

fn check_l1(fname: &str, text: &str, issues: &mut Issues) {
    let mut ids: Vec<u32> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let ln = i + 1;
        let s = line.trim();
        if !s.starts_with('|') || is_separator(s) {
            continue;
        }
        let cells = cells_of(s);
        if cells.is_empty() || cells[0] == "規則 ID" {
            continue;
        }
        if cells.len() != 6 {
            add(
                issues,
                Level::Fail,
                "L1",
                format!("{fname}:{ln}"),
                format!("欄數 {} ≠ 6", cells.len()),
            );
            continue;
        }
        match cid_num(&cells[0]) {
            None => {
                add(
                    issues,
                    Level::Fail,
                    "L1",
                    format!("{fname}:{ln}"),
                    format!("ID 格式錯誤：{:?}（應為 C-NN）", cells[0]),
                );
                continue;
            }
            Some(n) => ids.push(n),
        }
        if cells[2].is_empty() {
            add(
                issues,
                Level::Fail,
                "L1",
                format!("{fname}:{ln}"),
                "「為什麼」欄為空".to_string(),
            );
        }
        if cells[4].is_empty() {
            add(
                issues,
                Level::Fail,
                "L1",
                format!("{fname}:{ln}"),
                "「驗證」欄為空".to_string(),
            );
        }
        if !matches!(cells[5].as_str(), "ACTIVE" | "SUPERSEDED") {
            add(
                issues,
                Level::Fail,
                "L1",
                format!("{fname}:{ln}"),
                format!("狀態值域錯誤：{:?}", cells[5]),
            );
        }
    }
    let expected: Vec<u32> = (1..=ids.len() as u32).collect();
    if ids != expected {
        add(
            issues,
            Level::Fail,
            "L1",
            fname.to_string(),
            format!("ID 需自 C-01 連續且唯一；實得 {ids:?}"),
        );
    }
}

fn project_rows(text: &str) -> Vec<(usize, String, Vec<String>)> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let s = line.trim();
        if !s.starts_with('|') || is_separator(s) {
            continue;
        }
        let cells = cells_of(s);
        if cells.is_empty() || cells[0] == "事實（一句話）" {
            continue;
        }
        out.push((i + 1, line.to_string(), cells));
    }
    out
}

fn check_l2(fname: &str, text: &str, issues: &mut Issues) {
    for (ln, _line, cells) in project_rows(text) {
        if cells.len() != 4 {
            add(
                issues,
                Level::Fail,
                "L2",
                format!("{fname}:{ln}"),
                format!("欄數 {} ≠ 4", cells.len()),
            );
            continue;
        }
        if !matches!(cells[1].as_str(), "[根]" | "[ekpi]" | "[KB]" | "[全]") {
            add(
                issues,
                Level::Fail,
                "L2",
                format!("{fname}:{ln}"),
                format!("workspace 標籤錯誤：{:?}", cells[1]),
            );
        }
        if parse_date(&cells[3]).is_none() {
            add(
                issues,
                Level::Fail,
                "L2",
                format!("{fname}:{ln}"),
                format!("覆核日期格式錯誤：{:?}（YYYY-MM-DD）", cells[3]),
            );
        }
    }
}

fn check_l3(fname: &str, text: &str, issues: &mut Issues) {
    let today = today_days();
    for (ln, line, cells) in project_rows(text) {
        if cells.len() != 4 {
            continue;
        }
        let (y, m, d) = match parse_date(&cells[3]) {
            Some(v) => v,
            None => continue,
        };
        let age = today - days_from_civil(y, m, d);
        if age > STALE_DAYS && !line.contains("STALE") {
            add(
                issues,
                Level::Warn,
                "L3",
                format!("{fname}:{ln}"),
                format!("覆核日期已逾 {STALE_DAYS} 天（{}）且未標 STALE", cells[3]),
            );
        }
    }
}

fn check_l4(fname: &str, text: &str, issues: &mut Issues) {
    let mut ids: Vec<u32> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let ln = i + 1;
        let s = line.trim();
        if !s.starts_with('|') || is_separator(s) {
            continue;
        }
        let cells = cells_of(s);
        if cells.is_empty() || cells[0] == "決策" || cells[0] == "決策 ID" {
            continue;
        }
        if cells.len() != 7 {
            add(
                issues,
                Level::Fail,
                "L4",
                format!("{fname}:{ln}"),
                format!("欄數 {} ≠ 7（v1.1）", cells.len()),
            );
            continue;
        }
        match did_num(&cells[0]) {
            None => {
                add(
                    issues,
                    Level::Fail,
                    "L4",
                    format!("{fname}:{ln}"),
                    format!("ID 格式錯誤：{:?}（應為 D-NN）", cells[0]),
                );
                continue;
            }
            Some(n) => ids.push(n),
        }
        if !matches!(cells[6].as_str(), "NEW" | "VERIFIED" | "SUPERSEDED") {
            add(
                issues,
                Level::Fail,
                "L4",
                format!("{fname}:{ln}"),
                format!("狀態值域錯誤：{:?}", cells[6]),
            );
        }
    }
    let expected: Vec<u32> = (1..=ids.len() as u32).collect();
    if ids != expected {
        add(
            issues,
            Level::Fail,
            "L4",
            fname.to_string(),
            format!("ID 需自 D-01 連續且唯一；實得 {ids:?}"),
        );
    }
}

struct Ctx {
    checked: usize,
    whitelisted: usize,
    main_top: HashSet<String>,
    main_sub: HashSet<String>,
    this_top: HashSet<String>,
    this_sub: HashSet<String>,
    adr_chapters: BTreeMap<String, BTreeMap<String, Vec<u32>>>,
}

fn check_ref(r: &str, target: Target, ctx: &mut Ctx, fname: &str, ln: usize, issues: &mut Issues) {
    let loc = format!("{fname}:{ln}");
    match target {
        Target::Info => {
            ctx.whitelisted += 1;
        }
        Target::Main => {
            ctx.checked += 1;
            if r.starts_with("0.") {
                add(
                    issues,
                    Level::Fail,
                    "L5",
                    loc,
                    format!("§{r}：§0.N 形式已廢除（r3 慣例「§0 條名」）"),
                );
                return;
            }
            if r.contains('.') {
                if !ctx.main_sub.contains(r) {
                    add(
                        issues,
                        Level::Fail,
                        "L5",
                        loc,
                        format!("§{r}：主計畫無此子節標題"),
                    );
                }
            } else if !ctx.main_top.contains(r) {
                add(
                    issues,
                    Level::Fail,
                    "L5",
                    loc,
                    format!("§{r}：主計畫無此章節標題"),
                );
            }
        }
        Target::This => {
            ctx.checked += 1;
            if r.contains('.') {
                if !ctx.this_sub.contains(r) {
                    add(
                        issues,
                        Level::Fail,
                        "L5",
                        loc,
                        format!("§{r}：本計畫無此子節標題"),
                    );
                }
            } else if !ctx.this_top.contains(r) {
                add(
                    issues,
                    Level::Fail,
                    "L5",
                    loc,
                    format!("§{r}：本計畫無此章節標題"),
                );
            }
        }
        Target::Adr(num) => {
            ctx.checked += 1;
            match ctx.adr_chapters.get(&num) {
                None => {
                    add(
                        issues,
                        Level::Fail,
                        "L5",
                        loc,
                        format!("§{r}：ADR-{num} 檔不存在（docs/adr/）"),
                    );
                }
                Some(chapters) => {
                    if r.contains('.') {
                        let (chap, rest) = r.split_once('.').unwrap_or((r, ""));
                        match chapters.get(chap) {
                            None => {
                                add(
                                    issues,
                                    Level::Fail,
                                    "L5",
                                    loc,
                                    format!("§{r}：ADR-{num} 無此章節（## {chap}.）"),
                                );
                            }
                            Some(items) => {
                                let need: Option<u32> = rest.parse().ok();
                                let fail = match need {
                                    Some(n) => !items.contains(&n),
                                    None => true,
                                };
                                if fail {
                                    add(
                                        issues,
                                        Level::Fail,
                                        "L5",
                                        loc,
                                        format!(
                                            "§{r}：ADR-{num} §{chap} 無第 {rest} 條（章內編號列 {}）",
                                            items.len()
                                        ),
                                    );
                                }
                            }
                        }
                    } else if !chapters.contains_key(r) {
                        add(
                            issues,
                            Level::Fail,
                            "L5",
                            loc,
                            format!("§{r}：ADR-{num} 無此章節"),
                        );
                    }
                }
            }
        }
    }
}

fn check_l5(files: &HashMap<&'static str, String>, ctx: &mut Ctx, issues: &mut Issues) {
    for fname in FILES {
        let text = &files[fname];
        for (i, line) in text.lines().enumerate() {
            let ln = i + 1;
            for (idx, r) in scan_refs(line) {
                let target = target_for(&line[..idx]);
                check_ref(&r, target, ctx, fname, ln, issues);
            }
            for (_, tok) in scan_hyphen(line) {
                add(
                    issues,
                    Level::Warn,
                    "L5b",
                    format!("{fname}:{ln}"),
                    format!("非慣例 hyphen 形式 {tok}（改用點號或條名）"),
                );
            }
        }
    }
}

fn check_l6(mem_dir: &Path, issues: &mut Issues) {
    let s_corr = fs::metadata(mem_dir.join("corrections.md"))
        .map(|m| m.len())
        .unwrap_or(0);
    let s_proj = fs::metadata(mem_dir.join("project.md"))
        .map(|m| m.len())
        .unwrap_or(0);
    if s_corr > SIZE_CORRECTIONS_MAX {
        add(
            issues,
            Level::Warn,
            "L6",
            "corrections.md".to_string(),
            format!("大小 {s_corr}B > {SIZE_CORRECTIONS_MAX}B（8KB）——啟動分層載入程序"),
        );
    }
    if s_corr + s_proj > SIZE_ALWAYS_LOAD_MAX {
        add(
            issues,
            Level::Warn,
            "L6",
            "corrections.md+project.md".to_string(),
            format!(
                "常駐合計 {}B > {SIZE_ALWAYS_LOAD_MAX}B（12KB）——project.md 移出 instructions",
                s_corr + s_proj
            ),
        );
    }
}

fn declared_count(text: &str, prefix: &str) -> Option<u32> {
    let pos = text.find(prefix)?;
    let rest = &text[pos + prefix.len()..];
    let digits: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .take(2)
        .collect();
    if digits.len() == 2 {
        digits.parse().ok()
    } else {
        None
    }
}

fn check_l7(files: &HashMap<&'static str, String>, issues: &mut Issues) {
    let readme = &files["README.md"];
    let pairs: [(&str, &str, &'static str); 2] = [
        ("C", "C-01..C-", "corrections.md"),
        ("D", "D-01..D-", "decisions.md"),
    ];
    for (label, prefix, body_name) in pairs {
        let body = &files[body_name];
        let actual = body
            .lines()
            .filter(|l| {
                let s = l.trim();
                if !s.starts_with('|') || is_separator(s) {
                    return false;
                }
                let c = cells_of(s);
                if c.is_empty() {
                    return false;
                }
                if label == "C" {
                    cid_num(&c[0]).is_some()
                } else {
                    did_num(&c[0]).is_some()
                }
            })
            .count();
        match declared_count(readme, prefix) {
            None => add(
                issues,
                Level::Warn,
                "L7",
                "README.md".to_string(),
                format!("{label} 索引列未登載"),
            ),
            Some(d) if d as usize != actual => add(
                issues,
                Level::Warn,
                "L7",
                "README.md".to_string(),
                format!("{label} 條數不符：索引宣告 {d}，實檔 {actual}"),
            ),
            Some(_) => {}
        }
    }
}

fn scope_desc(check: &str) -> &'static str {
    match check {
        "L1" => "corrections.md 結構（6 欄／ID 自 C-01 連續唯一／狀態值域／必填非空）",
        "L2" => "project.md 結構（4 欄／workspace 標籤值域／日期 YYYY-MM-DD）",
        "L3" => "project.md 新鮮度（覆核日期 >30 天且未標 STALE）",
        "L4" => "decisions.md 結構（7 欄／ID 自 D-01 連續唯一／狀態值域）",
        "L5" => "四檔 §-引用解析（存在性；主計畫／本計畫／ADR）",
        "L5b" => "四檔 hyphen 形式偵測（§N-M，非慣例）",
        "L6" => "預算（corrections ≤8KB；corrections+project ≤12KB）",
        "L7" => "README 索引條數一致（C／D 宣告值 vs 實檔）",
        _ => "未知檢查",
    }
}

// ---------- 主流程 ----------

pub fn lint(root: &Path) -> Result<Outcome, LintError> {
    let mem_dir = root.join(".kilo").join("memories");
    let mut files: HashMap<&'static str, String> = HashMap::new();
    for f in FILES {
        let p = mem_dir.join(f);
        if !p.exists() {
            return Err(LintError(format!("{} 檔案缺失", p.display())));
        }
        files.insert(f, read_text(&p)?);
    }

    let plans_dir = root.join(".kilo").join("plans");
    let main_marker = |t: &str| t.lines().any(|l| l.starts_with("## §"));
    let this_marker = |t: &str| {
        t.lines().any(|l| match l.strip_prefix("## ") {
            Some(r) => leading_chapter(r).is_some(),
            None => false,
        })
    };
    let main_plan = match pick_plan(&plans_dir, MAIN_ID, &main_marker)? {
        Some(p) => p,
        None => {
            return Err(LintError(format!(
                "主計畫檔缺失或格式異常：.kilo/plans/{MAIN_ID}-*.md（需含 `## §N` 章節標題）"
            )));
        }
    };
    let this_plan = match pick_plan(&plans_dir, THIS_ID, &this_marker)? {
        Some(p) => p,
        None => {
            return Err(LintError(format!(
                "本計畫檔缺失或格式異常：.kilo/plans/{THIS_ID}-*.md（需含 `## N.` 章節標題）"
            )));
        }
    };

    let (main_top, main_sub) = parse_main_headings(&read_text(&main_plan)?);
    if main_top.is_empty() {
        return Err(LintError("主計畫解析不到章節標題（格式異常）".to_string()));
    }
    let (this_top, this_sub) = parse_this_headings(&read_text(&this_plan)?);
    if this_top.is_empty() {
        return Err(LintError("本計畫解析不到章節標題（格式異常）".to_string()));
    }

    let mut adr_chapters: BTreeMap<String, BTreeMap<String, Vec<u32>>> = BTreeMap::new();
    let adr_dir = root.join("docs").join("adr");
    if adr_dir.is_dir() {
        let rd =
            fs::read_dir(&adr_dir).map_err(|e| LintError(format!("{}: {e}", adr_dir.display())))?;
        for ent in rd {
            let ent = ent.map_err(|e| LintError(e.to_string()))?;
            let name = ent.file_name().to_string_lossy().to_string();
            if name.starts_with("ADR-") && name.ends_with(".md") {
                let text = read_text(&ent.path())?;
                if let Some(num) = adr_num_in(&name) {
                    adr_chapters
                        .entry(num)
                        .or_insert_with(|| parse_adr_chapters(&text));
                }
            }
        }
    }

    let mut ctx = Ctx {
        checked: 0,
        whitelisted: 0,
        main_top,
        main_sub,
        this_top,
        this_sub,
        adr_chapters,
    };
    let mut issues: Issues = HashMap::new();

    check_l1("corrections.md", &files["corrections.md"], &mut issues);
    check_l2("project.md", &files["project.md"], &mut issues);
    check_l3("project.md", &files["project.md"], &mut issues);
    check_l4("decisions.md", &files["decisions.md"], &mut issues);
    check_l5(&files, &mut ctx, &mut issues);
    check_l6(&mem_dir, &mut issues);
    check_l7(&files, &mut issues);

    let mut pass_count = 0usize;
    let mut warn_count = 0usize;
    let mut fail_count = 0usize;
    let mut lines: Vec<String> = Vec::new();
    for check in CHECKS {
        match issues.get(check) {
            None => {
                pass_count += 1;
                if check == "L5" {
                    lines.push(format!(
                        "PASS [L5] {}：checked={}, whitelisted={}",
                        scope_desc(check),
                        ctx.checked,
                        ctx.whitelisted
                    ));
                } else {
                    lines.push(format!("PASS [{}] {}", check, scope_desc(check)));
                }
            }
            Some(v) => {
                for iss in v {
                    let lv = match iss.level {
                        Level::Fail => {
                            fail_count += 1;
                            "FAIL"
                        }
                        Level::Warn => {
                            warn_count += 1;
                            "WARN"
                        }
                    };
                    lines.push(format!("{lv} [{}] {} {}", iss.check, iss.loc, iss.msg));
                }
            }
        }
    }
    lines.push(format!(
        "SUMMARY: FAIL={fail_count} WARN={warn_count} PASS={pass_count} (refs checked={}, whitelisted={})",
        ctx.checked, ctx.whitelisted
    ));

    Ok(Outcome { lines, fail_count })
}
