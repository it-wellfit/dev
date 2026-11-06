# Desktop Cleanup Plan

## Goal
Organize `C:\Users\stephanie.huang\Desktop` into clean category folders, remove duplicate dev artifacts and Office temp lock files, preserve active work (`新人訓練/`) and existing `Archive/`.

## Inventory (desktop root)

**Keep untouched (already organized):**
- `Archive/` — existing archive, do not touch
- `新人訓練/` — active workspace, do not touch

**Folders to relocate into category folders:**
- `flaps_extracted/` → `05_軟體開發/`
- `ner/` → `05_軟體開發/`
- `vscode/` (empty) → delete or move to `05_軟體開發/`

**Files → `00_刪除候選/` (Office temp lock files + PST temp):**
- `~$GoodsDetails.xlsx`
- `~$促銷活動.xlsx`
- `~$八月份工作計畫.xlsx`
- `~$工作計畫.xlsx`
- `~$百貨收入差異2026.08.xlsx`
- `~custom.pst.tmp`
- `~leo.pst.tmp`
- `desktop.ini`

**Files → `00_刪除候選/` (duplicate/obsolete dev archives — keep newest only):**
- `kilo-settings-skills-archive-20261030.zip` — older duplicate
- `kilo-settings-skills-archive-20261106.bundle` — bundle format, verify redundancy with .zip
- Keep: `kilo-settings-skills-archive-20261106.zip` (newest, complete)

**Files → `02_備份封存/`:**
- `backup-log-20261103.txt`
- `backup-log-20261104.txt`
- `backup-log-20261106.txt`
- `custom.pst` (143 MB — Outlook data file, large archive)
- `leo.pst` (1.5 GB — Outlook data file, large archive)
- `asar_header.json` — dev artifact, move to `05_軟體開發/`

**Files → `03_快捷方式/`:**
- `Flaps.url`
- `GitHub Copilot.lnk`
- `LINE.lnk`
- `mspaint.lnk`
- `NAS.lnk`
- `notepad.lnk`
- `OpenCode.lnk`
- `Outlook.lnk`

**Files → `04_圖片/`:**
- `輔翼連線清單.png`

**Files → `01_工作文件/`:**
- `報告內容.txt`
- `展威庫存差異-1029.xls`
- `新增 文字文件.txt`
- `簡報1.pptx`
- `輔翼ERP的店櫃資料_20261020 更新.xlsx`
- `輔翼ERP的店櫃資料_20261022 更新.xlsx`
- `NAS 登入權限的人員名單.csv`

## Folder structure to create on desktop
```
00_刪除候選/       # temp files, duplicates, pending deletion review
01_工作文件/        # active work documents
02_備份封存/        # backups, PST archives, logs
03_快捷方式/        # application shortcuts
04_圖片/            # image files
05_軟體開發/        # dev artifacts (flaps_extracted, ner, asar_header.json)
Archive/            # existing, untouched
新人訓練/           # active workspace, untouched
```

## Execution steps (implementation agent)
1. Create the 6 category folders on desktop root.
2. Move files per category mapping above.
3. Review `00_刪除候選/` with user before permanent deletion.
4. Confirm `新人訓練/` and `Archive/` remain untouched.
5. Verify desktop root contains only category folders + `Archive/` + `新人訓練/`.

## Validation
- `ls desktop` shows only: `00_刪除候選/`, `01_工作文件/`, `02_備份封存/`, `03_快捷方式/`, `04_圖片/`, `05_軟體開發/`, `Archive/`, `新人訓練/`
- No `~$*` temp files remain on desktop root
- No duplicate `.zip`/`.bundle` archives remain on desktop root
