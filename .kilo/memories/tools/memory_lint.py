#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""memory_lint.py — 專案記憶庫結構與引用完整性檢查（記憶庫 v2.0）

規格來源：.kilo/plans/1792461065184-project-memory-lessons.md §9.1
範圍：僅掃描 .kilo/memories/ 四檔（corrections.md／project.md／decisions.md／README.md）；
      驗證 §-引用之存在性（主計畫 1792865422970、本計畫 1792461065184、ADR 檔）。
      不對主計畫內文做「現行主張掃描」（§24／archived 排除等約定屬 QG-09 職責）。
特性：純讀、離線、Python stdlib only、確定性輸出。
輸出：PASS|WARN|FAIL [L#] <file>:<line> <message>；結尾 summary。
退出碼：0＝無 FAIL（WARN 允許）；1＝有 FAIL；2＝環境錯誤（目標檔缺失／解析失敗）。
擴充旗標：--out <file> 另存報告（供 session 證據；不改變 stdout 契約）。
"""

from __future__ import annotations

import argparse
import datetime as _dt
import re
import sys
from pathlib import Path

CHECKS = ("L1", "L2", "L3", "L4", "L5", "L5b", "L6", "L7")

_FILES = ("corrections.md", "project.md", "decisions.md", "README.md")
_MAIN_ID = "1792865422970"
_THIS_ID = "1792461065184"

_SIZE_CORRECTIONS_MAX = 8 * 1024      # L6: corrections.md ≤ 8KB
_SIZE_ALWAYS_LOAD_MAX = 12 * 1024     # L6: corrections + project ≤ 12KB
_STALE_DAYS = 30                      # L3

_TAG_RE = re.compile(r"^\[(根|ekpi|KB|全)\]$")
_DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
_CID_RE = re.compile(r"^C-(\d{2})$")
_DID_RE = re.compile(r"^D-(\d{2})$")
_REF_RE = re.compile(r"§(\d+(?:\.\d+)*)")
_HYPHEN_RE = re.compile(r"§\d+-\d+")
_ADR_IN_TEXT_RE = re.compile(r"ADR-(\d{4})")
_ADR_FILENAME_RE = re.compile(r"ADR-(\d{4})")
_MDPATH_RE = re.compile(r"[A-Za-z0-9_./\\\-]+\.md")
_S0N_RE = re.compile(r"^0\.")

_VALID_STATUS_CORR = {"ACTIVE", "SUPERSEDED"}
_VALID_STATUS_DEC = {"NEW", "VERIFIED", "SUPERSEDED"}

_SCOPE_DESC = {
    "L1": "corrections.md 結構（6 欄／ID 自 C-01 連續唯一／狀態值域／必填非空）",
    "L2": "project.md 結構（4 欄／workspace 標籤值域／日期 YYYY-MM-DD）",
    "L3": "project.md 新鮮度（覆核日期 >30 天且未標 STALE）",
    "L4": "decisions.md 結構（7 欄／ID 自 D-01 連續唯一／狀態值域）",
    "L5": "四檔 §-引用解析（存在性；主計畫／本計畫／ADR）",
    "L5b": "四檔 hyphen 形式偵測（§N-M，非慣例）",
    "L6": "預算（corrections ≤8KB；corrections+project ≤12KB）",
    "L7": "README 索引條數一致（C／D 宣告值 vs 實檔）",
}


def add(issues, level, check, loc, msg):
    issues.setdefault(check, []).append((level, loc, msg))


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8-sig")


def cells_of(line: str):
    s = line.strip()
    if s.startswith("|"):
        s = s[1:]
    if s.endswith("|"):
        s = s[:-1]
    return [c.strip() for c in s.split("|")]


def is_separator(line: str) -> bool:
    return bool(re.fullmatch(r"\|[\s\-:|]+\|", line.strip()))


def parse_main_plan_headings(text: str):
    top, sub = set(), set()
    for line in text.splitlines():
        m = re.match(r"^## §(\d+)\b", line)
        if m:
            top.add(m.group(1))
            continue
        m = re.match(r"^#{3,6}\s*§?(\d+(?:\.\d+)*)", line)
        if m:
            sub.add(m.group(1))
    return top, sub


def parse_this_plan_headings(text: str):
    top, sub = set(), set()
    for line in text.splitlines():
        m = re.match(r"^## (\d+)\.", line)
        if m:
            top.add(m.group(1))
            continue
        m = re.match(r"^### (\d+(?:\.\d+)*)", line)
        if m:
            sub.add(m.group(1))
    return top, sub


def parse_adr_chapters(text: str):
    chapters = {}
    cur = None
    for line in text.splitlines():
        m = re.match(r"^## (\d+)\.", line)
        if m:
            cur = m.group(1)
            chapters.setdefault(cur, [])
            continue
        if cur is not None:
            mi = re.match(r"^(\d+)\.\s", line)
            if mi:
                chapters[cur].append(int(mi.group(1)))
    return chapters


def target_for(prefix: str):
    """逐引用取「該引用之前最近的目標標記」；無標記 → 預設主計畫。"""
    best = None  # (end, kind, text)
    candidates = []
    for m in _MDPATH_RE.finditer(prefix):
        candidates.append((m.end(), "mdpath", m.group(0)))
    for m in re.finditer(_MAIN_ID, prefix):
        candidates.append((m.end(), "main", _MAIN_ID))
    for m in re.finditer(_THIS_ID, prefix):
        candidates.append((m.end(), "this", _THIS_ID))
    for m in re.finditer("憲章", prefix):
        candidates.append((m.end(), "charter", "憲章"))
    for cand in candidates:
        if best is None or cand[0] > best[0]:
            best = cand
    if best is None:
        return ("main", "")
    _, kind, text = best
    if kind == "mdpath":
        am = _ADR_IN_TEXT_RE.search(text)
        if am:
            return ("adr", am.group(1))
        if _MAIN_ID in text:
            return ("main", "")
        if _THIS_ID in text:
            return ("this", "")
        return ("info", text)
    return (kind, text)


def check_ref(ref, target, ctx, fname, ln, issues):
    kind, extra = target
    if kind == "info":
        ctx["whitelisted"] += 1
        return
    ctx["checked"] += 1
    if kind == "main":
        if _S0N_RE.match(ref):
            add(issues, "FAIL", "L5", f"{fname}:{ln}",
                f"§{ref}：§0.N 形式已廢除（r3 慣例「§0 條名」）")
            return
        if "." in ref:
            if ref not in ctx["main_sub"]:
                add(issues, "FAIL", "L5", f"{fname}:{ln}", f"§{ref}：主計畫無此子節標題")
        else:
            if ref not in ctx["main_top"]:
                add(issues, "FAIL", "L5", f"{fname}:{ln}", f"§{ref}：主計畫無此章節標題")
    elif kind == "this":
        if "." in ref:
            if ref not in ctx["this_sub"]:
                add(issues, "FAIL", "L5", f"{fname}:{ln}", f"§{ref}：本計畫無此子節標題")
        else:
            if ref not in ctx["this_top"]:
                add(issues, "FAIL", "L5", f"{fname}:{ln}", f"§{ref}：本計畫無此章節標題")
    elif kind == "adr":
        num = extra
        chapters = ctx["adr_chapters"].get(num)
        if chapters is None:
            add(issues, "FAIL", "L5", f"{fname}:{ln}",
                f"§{ref}：ADR-{num} 檔不存在（docs/adr/）")
            return
        if "." in ref:
            chap, _, rest = ref.partition(".")
            if chap not in chapters:
                add(issues, "FAIL", "L5", f"{fname}:{ln}",
                    f"§{ref}：ADR-{num} 無此章節（## {chap}.）")
                return
            items = chapters[chap]
            need = int(rest) if rest.isdigit() else None
            if need is None or (max(items) if items else 0) < need:
                add(issues, "FAIL", "L5", f"{fname}:{ln}",
                    f"§{ref}：ADR-{num} §{chap} 無第 {rest} 條（章內編號列 {len(items)}）")
        else:
            if ref not in chapters:
                add(issues, "FAIL", "L5", f"{fname}:{ln}",
                    f"§{ref}：ADR-{num} 無此章節")


def check_l1(fname, text, issues):
    ids = []
    for i, line in enumerate(text.splitlines(), 1):
        s = line.strip()
        if not s.startswith("|") or is_separator(s):
            continue
        cells = cells_of(line)
        if not cells or cells[0] == "規則 ID":
            continue
        if len(cells) != 6:
            add(issues, "FAIL", "L1", f"{fname}:{i}", f"欄數 {len(cells)} ≠ 6")
            continue
        m = _CID_RE.match(cells[0])
        if not m:
            add(issues, "FAIL", "L1", f"{fname}:{i}",
                f"ID 格式錯誤：{cells[0]!r}（應為 C-NN）")
            continue
        ids.append(int(m.group(1)))
        if not cells[2]:
            add(issues, "FAIL", "L1", f"{fname}:{i}", "「為什麼」欄為空")
        if not cells[4]:
            add(issues, "FAIL", "L1", f"{fname}:{i}", "「驗證」欄為空")
        if cells[5] not in _VALID_STATUS_CORR:
            add(issues, "FAIL", "L1", f"{fname}:{i}", f"狀態值域錯誤：{cells[5]!r}")
    if ids != list(range(1, len(ids) + 1)):
        add(issues, "FAIL", "L1", fname, f"ID 需自 C-01 連續且唯一；實得 {ids}")


def _project_rows(text):
    for i, line in enumerate(text.splitlines(), 1):
        s = line.strip()
        if not s.startswith("|") or is_separator(s):
            continue
        cells = cells_of(line)
        if not cells or cells[0] == "事實（一句話）":
            continue
        yield i, line, cells


def check_l2(fname, text, issues):
    for i, line, cells in _project_rows(text):
        if len(cells) != 4:
            add(issues, "FAIL", "L2", f"{fname}:{i}", f"欄數 {len(cells)} ≠ 4")
            continue
        if not _TAG_RE.match(cells[1]):
            add(issues, "FAIL", "L2", f"{fname}:{i}",
                f"workspace 標籤錯誤：{cells[1]!r}")
        if not _DATE_RE.match(cells[3]):
            add(issues, "FAIL", "L2", f"{fname}:{i}",
                f"覆核日期格式錯誤：{cells[3]!r}（YYYY-MM-DD）")


def check_l3(fname, text, today, issues):
    for i, line, cells in _project_rows(text):
        if len(cells) != 4 or not _DATE_RE.match(cells[3]):
            continue
        d = _dt.date.fromisoformat(cells[3])
        if (today - d).days > _STALE_DAYS and "STALE" not in line:
            add(issues, "WARN", "L3", f"{fname}:{i}",
                f"覆核日期已逾 {_STALE_DAYS} 天（{cells[3]}）且未標 STALE")


def check_l4(fname, text, issues):
    ids = []
    for i, line in enumerate(text.splitlines(), 1):
        s = line.strip()
        if not s.startswith("|") or is_separator(s):
            continue
        cells = cells_of(line)
        if not cells or cells[0] in ("決策", "決策 ID"):
            continue
        if len(cells) != 7:
            add(issues, "FAIL", "L4", f"{fname}:{i}", f"欄數 {len(cells)} ≠ 7（v1.1）")
            continue
        m = _DID_RE.match(cells[0])
        if not m:
            add(issues, "FAIL", "L4", f"{fname}:{i}",
                f"ID 格式錯誤：{cells[0]!r}（應為 D-NN）")
            continue
        ids.append(int(m.group(1)))
        if cells[6] not in _VALID_STATUS_DEC:
            add(issues, "FAIL", "L4", f"{fname}:{i}", f"狀態值域錯誤：{cells[6]!r}")
    if ids != list(range(1, len(ids) + 1)):
        add(issues, "FAIL", "L4", fname, f"ID 需自 D-01 連續且唯一；實得 {ids}")


def check_l5(files, ctx, issues):
    for fname in _FILES:
        text = files[fname]
        for i, line in enumerate(text.splitlines(), 1):
            for m in _REF_RE.finditer(line):
                target = target_for(line[: m.start()])
                check_ref(m.group(1), target, ctx, fname, i, issues)
            for m in _HYPHEN_RE.finditer(line):
                add(issues, "WARN", "L5b", f"{fname}:{i}",
                    f"非慣例 hyphen 形式 {m.group(0)}（改用點號或條名）")


def check_l6(root, issues):
    p_corr = root / ".kilo" / "memories" / "corrections.md"
    p_proj = root / ".kilo" / "memories" / "project.md"
    s_corr = p_corr.stat().st_size
    s_proj = p_proj.stat().st_size
    if s_corr > _SIZE_CORRECTIONS_MAX:
        add(issues, "WARN", "L6", "corrections.md",
            f"大小 {s_corr}B > {_SIZE_CORRECTIONS_MAX}B（8KB）——啟動分層載入程序")
    if s_corr + s_proj > _SIZE_ALWAYS_LOAD_MAX:
        add(issues, "WARN", "L6", "corrections.md+project.md",
            f"常駐合計 {s_corr + s_proj}B > {_SIZE_ALWAYS_LOAD_MAX}B（12KB）——project.md 移出 instructions")


def check_l7(files, issues):
    readme = files["README.md"]
    corr = files["corrections.md"]
    dec = files["decisions.md"]
    pairs = (
        ("C", r"C-01\.\.C-(\d{2})", corr, r"^\|\s*C-\d{2}\s*\|"),
        ("D", r"D-01\.\.D-(\d{2})", dec, r"^\|\s*D-\d{2}\s*\|"),
    )
    for label, decl_re, body, row_re in pairs:
        m = re.search(decl_re, readme)
        actual = len(re.findall(row_re, body, re.M))
        if m is None:
            add(issues, "WARN", "L7", "README.md", f"{label} 索引列未登載")
            continue
        declared = int(m.group(1))
        if declared != actual:
            add(issues, "WARN", "L7", "README.md",
                f"{label} 條數不符：索引宣告 {declared}，實檔 {actual}")


def main() -> int:
    ap = argparse.ArgumentParser(description="專案記憶庫結構與引用完整性檢查（v2.0）")
    ap.add_argument("--root", default=None,
                    help="Dev 工作區根（預設：本腳本往上三層）")
    ap.add_argument("--out", default=None,
                    help="另存報告檔（供 session 證據；不影響 stdout）")
    args = ap.parse_args()

    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

    root = Path(args.root).resolve() if args.root else Path(__file__).resolve().parents[3]
    captured = []

    def out(s: str):
        print(s)
        captured.append(s)

    def finish(code: int):
        if args.out:
            try:
                Path(args.out).write_text("\n".join(captured) + "\n", encoding="utf-8")
            except OSError:
                pass
        return code

    # ---- 載入（環境錯誤 → exit 2） ----
    mem_dir = root / ".kilo" / "memories"
    files = {}
    for fname in _FILES:
        p = mem_dir / fname
        if not p.exists():
            out(f"FAIL [ENV] {p} 檔案缺失")
            return finish(2)
        files[fname] = read_text(p)

    main_plan_files = sorted((root / ".kilo" / "plans").glob(f"{_MAIN_ID}-*.md"))
    this_plan_files = sorted((root / ".kilo" / "plans").glob(f"{_THIS_ID}-*.md"))
    if not main_plan_files:
        out(f"FAIL [ENV] 主計畫檔缺失：.kilo/plans/{_MAIN_ID}-*.md")
        return finish(2)
    if not this_plan_files:
        out(f"FAIL [ENV] 本計畫檔缺失：.kilo/plans/{_THIS_ID}-*.md")
        return finish(2)

    try:
        main_top, main_sub = parse_main_plan_headings(read_text(main_plan_files[0]))
        this_top, this_sub = parse_this_plan_headings(read_text(this_plan_files[0]))
        adr_chapters = {}
        adr_dir = root / "docs" / "adr"
        for p in sorted(adr_dir.glob("ADR-*.md")):
            m = _ADR_FILENAME_RE.search(p.name)
            if m and m.group(1) not in adr_chapters:
                adr_chapters[m.group(1)] = parse_adr_chapters(read_text(p))
    except OSError as exc:
        out(f"FAIL [ENV] 解析失敗：{exc}")
        return finish(2)

    ctx = {
        "checked": 0,
        "whitelisted": 0,
        "main_top": main_top,
        "main_sub": main_sub,
        "this_top": this_top,
        "this_sub": this_sub,
        "adr_chapters": adr_chapters,
    }
    issues = {}

    # ---- 檢查 ----
    check_l1("corrections.md", files["corrections.md"], issues)
    check_l2("project.md", files["project.md"], issues)
    check_l3("project.md", files["project.md"], _dt.date.today(), issues)
    check_l4("decisions.md", files["decisions.md"], issues)
    check_l5(files, ctx, issues)
    check_l6(root, issues)
    check_l7(files, issues)

    # ---- 輸出 ----
    counts = {"PASS": 0, "WARN": 0, "FAIL": 0}
    for check in CHECKS:
        lst = issues.get(check, [])
        if not lst:
            counts["PASS"] += 1
            if check == "L5":
                out(f"PASS [L5] {_SCOPE_DESC[check]}：checked={ctx['checked']}, "
                    f"whitelisted={ctx['whitelisted']}")
            else:
                out(f"PASS [{check}] {_SCOPE_DESC[check]}")
            continue
        for level, loc, msg in lst:
            counts[level] += 1
            out(f"{level} [{check}] {loc} {msg}")

    out(f"SUMMARY: FAIL={counts['FAIL']} WARN={counts['WARN']} PASS={counts['PASS']} "
        f"(refs checked={ctx['checked']}, whitelisted={ctx['whitelisted']})")
    return finish(1 if counts["FAIL"] else 0)


if __name__ == "__main__":
    sys.exit(main())
