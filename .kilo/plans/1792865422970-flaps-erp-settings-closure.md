# FlapsErp 設定優化專案 — 自主可完成項全數收口計畫（Stage 0 收口版）

> 計畫編號：1792865422970（本次 session）｜日期：2026-11-11｜狀態：**Implementation-ready**
> 上游母計畫：`.kilo/plans/1794395575776-flaps-erp-product-system-settings.md`（Active，待 IT 變更窗）
> 上游子計畫：`.kilo/plans/1794413131000-windmill-install-plan.md`（Active v3；Stage 0 立即）
> 對應報告：`docs/research/FlapsErp設定AB實測報告_v1.0.md`（**NOT VERIFIED**；唯讀證據完成）
> 治理：C-01/C-03/C-04（先量測、斷言式編輯）、C-10（UNKNOWN ≠ PASS ≠ FAIL）、C-13（五級判定）。
> 紅線不變：代理唯讀 `C:\SYSTEM_PROGRAM_DIRECTORY` 與 OS；任何原廠目錄／系統設定寫入由 IT 於變更管制下執行。

---

## 0. 目標

完成**所有不依賴 IT／原廠**的剩餘項目，並把 IT 執行面精確交棒；最終以誠實五級判定收口。

**非目標**：IT 變更（T-04/05/06/07/08/12/13/14/16）、系統級 A/B、8h soak、Windmill Stage 1、EE 採購、原廠 OQ-F1/F2、`15.4 s / -10%` 等成效宣稱（未量測不得宣稱）。

---

## 1. 現況基線（2026-11-11 實證）

| 項目 | 狀態 | 證據 |
|---|---|---|
| 母計畫 T-00..T-03 唯讀證據 | 完成 | A/B 報告 v1.0 §B（124 行） |
| 已入版控交付物 | 乾淨（`38bdd59`） | 最佳設定基線、設定變更 runbook、A/B 報告、操作手冊 §3.2 更正 banner、A1 手冊 §更正 banner、風險表 v1.2、PLAN_REGISTER #32/#33、docs/README 索引列（L121-140） |
| perf artifacts | 存在（25 個時戳執行，含報告引用 4 筆＋`20261111-200314`） | `Wellfit/artifacts/perf/` 目錄實讀 |
| 計畫檔版控 | `1794395575776`、`1794413131000`、`1793033952894` 均 **untracked** | explore 實測 |
| 他 session in-flight（**禁碰**） | `crates/console_manifest/src/lib.rs`、`platform/src/{api,nav}.rs`、`platform/templates/*`（4）、`platform/tests/page_matrix.rs`、`web/assets/*.css`（2） | `git status --short` |

### 1.1 未竟項清單（本次全收）

1. **A1 手冊殘餘錯誤**（母計畫 T-09 未竟＋Windmill T-S0.2）：§1.2（L24-41）Windmill 安裝段仍寫 `windmill db init`＋SQLite＋任意二進位（與 Windmill 計畫 §2 三項推翻直接衝突）；L70-71 `run-shopee-pipeline.ps1` 指向 `C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp\...\`（錯誤路徑）且該腳本**不存在**（`Wellfit/scripts/erp-batch/` 只有 `run-mtrix-pipeline.ps1`）；L77 `flapserp://` import 參數指向原廠目錄 `FlapsErpData\import\latest.csv`（違反 RSK-E5 寫入）；L236 清理指令 `rm -rf C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErpData\*\import.csv`（原廠目錄刪除）。
2. **母計畫 §12 Windmill 列**：仍寫「待 T-00」；T-00 已完成（實測未安裝）＋路線已裁定。
3. **藍圖 v1.0**（`docs/research/輔翼ERP優化多階段執行藍圖_v1.0.md`）：§0 依賴圖與 T-202 行仍以「Windmill(原生)排程」為前提（舊假設），未指向 v3 更正。
4. **Windmill T-S0.1 未實作**：包裝腳本 `Wellfit/scripts/erp-batch/run-erp-batch-scheduled.ps1` 不存在（Stage 0 核心交付）。
5. **A/B 報告 §B.3/§K「其他 HTTPS 端點盤點」**：僅完成 app.asar 靜態面；runtime 面（程序連線／log 盤點）未收。
6. **A/B 報告 v1.1 未出**：T1–T5 結果未摺入；§J 未連 windmill 計畫／藍圖。
7. **登記未收**：PLAN_REGISTER addendum #34 未建；兩份本工作流計畫檔（settings、windmill）未入版控。

---

## 2. 決策

| ID | 決策 | 理由 |
|---|---|---|
| D-1 | 本次 commit 僅含本工作流檔案；**嚴禁** stage 他 session 的 9 個 in-flight 檔 | 單寫者；避免 provenance 污染 |
| D-2 | 計畫檔入版控＝`1794395575776`＋`1794413131000` 兩個；`1793033952894`（OMS 工作流）**不**入，留待其所有者 | 範圍紀律；OMS 為獨立 16 週工作流 |
| D-3 | 不新增並行文件；一律「更正延續既有文件」＋（必要時）小幅修訂紀錄 | 非冗餘（母計畫 §10；使用者主軸） |
| D-4 | A/B 報告判定**維持 NOT VERIFIED**（IT 未執行）；僅更新證據/未決，不得升級 | C-10/C-13 |
| D-5 | Stage 0 包裝腳本依 Windmill 計畫 §5 T-S0.1 七點規格＋PS 5.1 規範（ASCII-only、無 BOM） | 既存 .ps1 工具鏈例外（C-11） |
| D-6 | 不為 `run-shopee-pipeline.ps1` 新建腳本；A1 手冊改為誠實標註「尚未建立＋現行替代＝`wellfit.exe shopee`」 | 不虛構；避免未裁定擴建 |

---

## 3. 任務（有序；實作代理單寫者執行）

### T1 — A1 手冊更正收口（`docs/04-operations/A1檔案通道端到端操作手冊_v1.0.md`）
1. **T1.1 §1.2 重寫（L24-41）**：刪 `windmill db init`／`serve --database-url "sqlite://…"`／「Windows x64 原生二進位」；改為事實版：①路線 C（Task Scheduler）先行；②Windows 原生＝EE 付費（`windmill-ee.exe`＋`sc.exe`）；③自託管必 PostgreSQL 18（無 SQLite 模式）；④EE 若獲准：`C:\ProgramData\Windmill\versions\<ver>\`＋`MODE=server/worker`＋PW 7 前置（`POWERSHELL_PATH`）；引用 Windmill 計畫 §2/§4。保留「本機 2026-11-11 未安裝」註記。
2. **T1.2 Flow YAML（L65-79）**：`run-mtrix` 步保持（L65 路徑正確）；`run-shopee` 步改 `& "Wellfit\scripts\erp-batch\run-shopee-pipeline.ps1"` ＋ 行內註「【尚未建立；現行替代：`wellfit.exe shopee`，見 Step 2；待建任務另立】」；`open-flapserp` 的 `file` 參數改為 `"$env:TEMP\flaps-import-latest.csv"`（或 artifacts run 目錄）。
3. **T1.3 清理指令（L236 區）**：`rm -rf C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErpData\*\import.csv` → 改為 artifacts 範圍清理（如 `Remove-Item -Recurse "$run"`）。先讀 L131-252 全段確認無其他原廠目錄寫入殘餘。
4. **T1.4 頭註/修訂紀錄**：狀態行更新（保留 Draft＋加「2026-11-11 更正：Windmill 段與路徑依計畫 1794413131000 §2 修訂」）；檔尾加修訂紀錄表列 v1.1。
5. **驗收**：全文 `SYSTEM_PROGRAM_DIRECTORY` 命中僅剩「唯讀輸入／檢查（Test-Path）」＋更正註記；**零**寫入型指令（`New-Item`/`Set-Content`/`Copy-Item`/安裝段）指向原廠目錄。

### T2 — 交叉文件同步（母計畫／藍圖／操作手冊）
1. **母計畫 §12**：「Windmill 是否已安裝」列更新＝「未安裝（F1 實測 2026-11-11；`Get-Command windmill` 無輸出）；路線已裁定＝階段式 C 先行＋評估 EE（計畫 `1794413131000` v3）；Stage 0 執行中」。
2. **藍圖 v1.0**：§0 依賴圖 P2 註記＋T-202 行加更正：「2026-11-11：Windmill 原生排程假設已由計畫 `1794413131000` v3 更正（EE 付費／無 SQLite 模式）；先以 Task Scheduler 落地（Stage 0）」；修訂紀錄加 v1.1 列。
3. **操作手冊**（`docs/04-operations/FlapsErp效能優化操作手冊_v1.0.md`）：§相依文件（L9-20）增列三新檔（`FlapsErp最佳設定基線_v1.0.md`、`FlapsErp設定變更runbook_v1.0.md`、`FlapsErp設定AB實測報告_v1.0.md`）；實讀 §3.2-§3.4（L220-280）確認僅存更正 banner、無殘餘錯誤指令。
4. **驗收**：三檔名出現在相依列；`grep -n "Windmill(原生)"`（藍圖）與「待 T-00」（母計畫）零未標註命中。

### T3 — Windmill Stage 0 包裝腳本（T-S0.1）
1. 前置斷言：讀 `Wellfit/AGENTS.md`（PS 規範）；`Test-Path "$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe"` — 不存在則依 F11 以 `Knowledge\tools\dev-env\build.ps1` 建置（若建置不可行 → 記錄並降級驗收）。
2. 新增 `Wellfit/scripts/erp-batch/run-erp-batch-scheduled.ps1`：①單一實例守門（偵測未結束前次 → exit 3）；②`BelowNormal` 啟動子程序；③產出只寫 `Wellfit\artifacts\erp-batch\<ts>\`；④逐步 exit code 傳遞，任一失敗 exit 1；⑤run log 寫入同 run 目錄；⑥PS 5.1：ASCII-only、無 BOM、訊息用 ASCII；⑦以絕對路徑呼叫 `wellfit.exe`；⑧編排呼叫既有 `run-mtrix-pipeline.ps1`（不重複實作批次邏輯）。註：dup 批次時 run-mtrix exit 1（冪等拒絕）為預期語義，wrapper 原樣傳遞。
3. 實跑證據：手動執行 → exit 0＋artifacts＋log；併發第二實例 → exit 3。
4. 產出 IT 用排程註冊命令（`Register-ScheduledTask`，每日 02:00、僅 AC、與 `Unregister-ScheduledTask` 回滾），寫入腳本頭註＋Windmill 計畫 §5 Stage 0。
5. **驗收**：腳本存在；編碼掃描（無 PUA、無 BOM）；實跑 exit 0 記錄（artifacts 路徑＋log）；註冊命令文件化。

### T4 — HTTPS 端點 runtime 盤點（唯讀；母計畫 T-03/§K 收尾）
1. 若 FlapsErp 執行中：`Get-NetTCPConnection -State Established` 過濾 FlapsErp PID 之 remote endpoint（解析 host）；掃 `C:\SYSTEM_PROGRAM_DIRECTORY\logs\*.log`（唯讀）取 `https://`／`http://` host 清單；與既有 app.asar 靜態清單聯集。
2. 若未執行：**不啟動**（不屬代理權限）；runtime 面標 `UNKNOWN`＋方法文件化（人工於下次運行時可重跑）。
3. 產出：`Wellfit/artifacts/perf/<ts>/https-endpoints.json`（比照既有 probe 產物慣例）；更新 A/B 報告 §B.3/§K。
4. **驗收**：JSON 存在或 UNKNOWN 註記；全程唯讀（紅線）。

### T5 — A/B 報告 v1.1（摺入 T1–T4）
1. `docs/research/FlapsErp設定AB實測報告_v1.0.md`：修訂紀錄加 v1.1；§B.3 端點盤點結果；§F 增 Stage 0 wrapper 實測列；§J 連結加 Windmill 計畫／藍圖；§K 移除已結項、保留 IT/原廠項；§L **維持 NOT VERIFIED**＋明列解除條件（T-04/05/07/12＋系統級 A/B＋排程註冊）。
2. **驗收**：與 T1–T4 證據一致；引用路徑實際存在。

### T6 — 登記與提交
1. PLAN_REGISTER **addendum #34**（Item/Disposition/Evidence 三欄式）：T-09 收口、T-S0.2、Stage 0 包裝腳本＋實跑、端點盤點、A/B v1.1、判定 NOT VERIFIED 不變。
2. 慣例斷言：`git ls-files .kilo/plans`（計畫檔入版控前例？）與 `git ls-files Wellfit/artifacts | head`（artifacts 是否入版控）→ 依前例決定是否 `git add`（預設：計畫檔依 D-2；新 artifacts 不入版控除非有前例）。
3. 提交（單寫者；訊息風格仿 `38bdd59`）：
   - 檔案清單：`docs/04-operations/A1檔案通道端到端操作手冊_v1.0.md`、`docs/04-operations/FlapsErp效能優化操作手冊_v1.0.md`、`docs/research/輔翼ERP優化多階段執行藍圖_v1.0.md`、`docs/research/FlapsErp設定AB實測報告_v1.0.md`、`docs/PLAN_REGISTER.md`、`.kilo/plans/1794395575776-*.md`、`.kilo/plans/1794413131000-*.md`、`Wellfit/scripts/erp-batch/run-erp-batch-scheduled.ps1`（＋條件式：docs/README 若有變更）。
   - **禁碰**：§1 表列他 session 9 檔＋`1793033952894`。
4. **驗收**：`git status` 顯示禁碰檔案仍為原狀態；提交後工作樹僅剩他 session 變更。

### T7 — 最終判定與 IT 交棒
1. 判定表（誠實五級）：
   - 設定採用（config/OS）：**NOT VERIFIED**（IT 未執行）。
   - Stage 0 腳本：**PASS**（實跑綠）｜排程註冊：**CONDITIONAL**（IT 建立後驗收）。
   - 文件更正：**PASS**｜端點盤點：PASS 或 UNKNOWN（依 T4）。
2. 交棒單：引用 runbook §2-§7＋T3 註冊命令＋EE 詢價決策單（Windmill T-S0.3 指標）＋母計畫 §12 剩餘 IT 列。

---

## 4. 驗證（收口時執行）

1. 編碼：`scripts/encoding-tools.ps1`（或等價 PUA 掃描）覆蓋本次改動檔（含新 .ps1）。
2. 引用完整性：本次新增/修改的相對連結逐一 `Test-Path`／read 實證存在。
3. `git diff --stat` 對照 §3.6 檔案清單；禁碰清單零命中。
4. T3 實跑證據（exit 0＋artifacts＋log）與 T4 JSON 路徑實際存在（或 UNKNOWN 註記）。
5. 五級判定與 §3.7 一致；UNKNOWN 不偽裝 PASS。

---

## 5. 風險

| 風險 | 緩解 |
|---|---|
| 誤入他 session 檔案（並行污染） | 精確 stage 清單；提交前 `git status` 對照禁碰表 |
| 中文路徑工具佯謬（glob 零命中） | 一律以 read 實證；不用 glob 判定中文路徑存在性 |
| PS 5.1 編碼陷阱（C-09） | 新 .ps1 ASCII-only、無 BOM；寫入用 `[IO.File]::WriteAllText`＋UTF8 no BOM |
| `wellfit.exe` 未建置 | T3 前置斷言＋依 F11 建置；不可行則降級記錄 |
| FlapsErp 未執行致 T4 降級 | 標 UNKNOWN＋方法文件化；不越權啟動 |
| artifacts 版控慣例 UNKNOWN | T6 先 `git ls-files` 斷言，再決定 |

## 6. 未決（非阻塞）

- OMS 計畫 `1793033952894` 之入冊與執行：另一工作流，本次範圍外（D-2）。
- Windmill Stage 1（EE）與 T-Doc 終局：依 EE 詢價裁定（外部）。
