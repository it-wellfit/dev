# 專案記憶庫（跨 session 工程記憶）

> 建立日期：2026-11-09 ｜ 版本：v1.1 ｜ 定位：**跨 session 才值得知道的事**——教訓、事實索引、決策記錄。**禁止複寫 SSOT**（AGENTS.md、plan 檔、KB）；本目錄只寫連結＋一句話。
> 完整落實計畫：`.kilo/plans/1792461065184-project-memory-lessons.md`（唯一權威設計文件）。
> 演進來源：2026-11-07～11-09 `1792865422970-three-studio-governance-platform.md` v3.1→v3.1-r2→v3.1-r3 精細迭代（特別是 r3 機器稽核發現 r2 五項聲明不實並修正的全程）。

## 讀取順序（新 session 標準動作）

1. `corrections.md`（強制規則，`C-01..C-13`，以 `instructions` 自動加載，手動查閱為備援）。
2. `project.md`（事實索引，workspace 標籤制）。
3. `decisions.md`（決策記錄），再回本 README 查索引狀態。
4. 情節（episodic）僅於「任務與過去事件相關」時檢索 `.kilo/sessions/`；常駐不載入（S-05：聚焦勝全量）。

## 衝突權威序（讀到矛盾時）

現行 repo 檔與工具輸出 ＞ `decisions.md` ＞ `project.md` 事實 ＞ `corrections.md` 指引；**禁令不因衝突失效——衝突採更嚴者**。遇衝突 → 開清單修正記憶（記錄於 `decisions.md`）。

## 維護規則（仿 `.kilo/sessions/index.md` Rules）

- 每次 session 結束時：若有新教訓，append 到 `corrections.md` 並更新本索引；決策事項記入 `decisions.md`；新事實記入 `project.md`。
- **規則總數上限 15 條**，逾限須啟動合併/歸檔程序（保持反射效率，避免稀釋）。
- 作廢不刪除：改標 `SUPERSEDED`（`corrections.md`/`project.md`/`decisions.md` 皆同），保留軌跡（仿 `1792865422970-*` §0 變更管制）。
- 每條必附來源連結；無來源者不記。
- **蒸餾三步**：事故 →（session 內）記 manifest →（session 末）問「可重用？」→ 併入既有 C 或新增（新 C 必帶事件級「為什麼」與可重算「驗證」）。
- **修剪**：每 10 session 或每季一次；零命中且非禁令 → 標 `SUPERSEDED` 不刪號；剪修建議與結果記入 `decisions.md`。

## 常駐載入預算

- 常駐（`instructions`）合計 ≤12KB；`corrections.md` ≤8KB（`tools/memory_lint`（Rust）L6 監測）。
- 逾限時：`project.md` 移出 `instructions`，由 `corrections.md` 頂部唯一指針行按需觸發。
- 非可協商規則置頂（S-07：harness 自尾端截斷，尾端規則最先犧牲）。

## 規則撰寫 4 準則

1. **禁令優先**：「不得／禁止」表述優先於正向指令（S-09：negative constraints 勝）。
2. **只收非可推得**：agent 無法自 config／程式碼推得（non-inferable）或跨檔聚合（cross-cutting）之事才收。
3. **必附可重算驗證**：`tools/memory_lint`（Rust）可自動檢查者尤佳。
4. **新增前先查重**：防重複與矛盾（S-02）。

## 與既有系統的分工

| 系統 | 定位 | 關係 |
|---|---|---|
| `.kilo/memories/`（本目錄） | agent 作業規則（行為） | 強制執行 |
| `Dev/knowledge/claims/`（kem claim registry，20 筆） | 事實宣稱（Fact/Invariant/Decision，有 confidence／evidence／supersede 欄位） | 規則若需升級為不變量，走 kem claim 流程，見 `knowledge/index.md` 索引與 `knowledge/log.md` 決策日誌 |
| `Knowledge/`（2026 同級目錄） | 合規 SSOT（ADR、SoA 93 控制、crosswalk） | 本目錄只連結，不複寫 |
| `.kilo/sessions/`（現行 session，數目以目錄為準） | session 執行紀錄（manifest.md） | 記憶庫維護紀錄回寫當次 session manifest（查閱結果與規則應用記錄） |

**記憶三層（procedural／semantic／episodic）**：

| 層 | 載體 | 讀取時機 |
|---|---|---|
| procedural（規則） | `corrections.md` | 常駐（`instructions` 自動加載） |
| semantic（事實／決策） | `project.md`、`decisions.md` | 按需（接手程序第 4 步） |
| episodic（情節） | `.kilo/sessions/`、平台原生記憶 | 僅任務與過去事件相關時檢索 |

## 索引（NEW = 未經 session-end 覆核；VERIFIED = 已覆核）

| 檔案 | 條目數 | 狀態 |
|---|---|---|
| `corrections.md` | C-01..C-13（13 條） | VERIFIED |
| `project.md` | 7 條（跨 session 現況） | VERIFIED |
| `decisions.md` | D-01..D-07（7 條） | VERIFIED |

## 演進方向（供後續參考）

係數化紀錄：每條規則增 `命中次數／覆核日期／SUPERSEDED 狀態` 欄位以支持衰減管理；命中統計由各 session manifest 的「規則應用記錄」彙總。
