# Windmill 導入計畫（修訂 v3：證據導向、測試／正式分離、CI/CD 對齊、資源守門）

> 計畫編號：1794413131000｜日期：2026-11-11｜狀態：**Active（路線已裁定 2026-11-11：階段式 C 先行＋評估 EE）**
> 治理：C-01（不亂覆蓋）、C-10（UNKNOWN ≠ PASS ≠ FAIL）、C-13（Evidence-Driven 五級判定）。
> 執行邊界：與母計畫 1794395575776 一致——代理只做量測、規格、文件、驗證；任何 OS 層安裝由 IT 於變更管制（備份＋SHA256＋回滾）下執行。

---

## 1. 現狀與已驗證事實（附來源與日期）

| # | 事實 | 證據 | 日期 |
|---|---|---|---|
| F1 | Windmill **完全未安裝**：全工作區掃描 14 處提及皆為文件/CI flag；`Get-Command windmill` 無輸出 | 工作區 grep 全檔掃描；母計畫 §12「待 T-00」 | 2026-11-11 |
| F2 | Windmill 伺服器**必須 PostgreSQL**（含 job queue 的全部狀態）；無 SQLite 部署模式 | windmill.dev/docs/advanced/self_host（Architecture） | 2026-11-11 |
| F3 | **Windows 原生 worker＝Enterprise 付費功能**；Windows 二進位為 `windmill-ee.exe`，服務化用 `sc.exe`（MODE=worker/server/agent） | windmill.dev/docs/misc/windows_workers | 2026-11-11 |
| F4 | 免費 CE 自託管官方路徑＝Docker／K8s；本環境**禁止 Docker** | 同 F2（Docker/Helm 為唯二主要路徑）＋ corrections `windmill_deployment_requirement` | 2026-11-11 |
| F5 | 最新 release：v1.867.0（2026-11-10）；版本迭代極快（每數日一版） | github.com/windmill-labs/windmill/releases | 2026-11-11 |
| F6 | 工作區 CI 已有四環境模型：develop／test／release／main(production)；production 需 PR＋審批 | `.github/config/ci-config.yml:4-36` | 2026-11-11 |
| F7 | 專案資料庫基線＝PostgreSQL 18 | `.kilo/memories/project.md` database_strategy | 2026-11-11 |
| F8 | 本機 shell 為 Windows PowerShell **5.1**；Windmill PowerShell executor 官方要求 **PowerShell 7+**（`POWERSHELL_PATH`）→ PW7 須由 IT 安裝，非既有 | shell 環境實測（PS 5.1）；corrections C-09；windmill.dev/docs/misc/windows_workers（PowerShell executor 節） | 2026-11-11 |
| F9 | `.github/workflows/` **不存在**（glob 實測零檔）；ci-config.yml 自述「動態驅動 workflow」為意圖配置，`windmill: true` 目前無實際消費者 | glob `.github/workflows/*` → 0；`.github/config/ci-config.yml:1-2` | 2026-11-11 |
| F10 | Wellfit 批次腳本與產物目錄**實在**：`Wellfit\scripts\erp-batch\run-mtrix-pipeline.ps1` 存在；`Wellfit\artifacts\erp-batch\` 有 7 個時戳執行（20261103–20261111） | 目錄 read 實測（先前 glob 對中文路徑零命中為工具佯謬，read 已排除） | 2026-11-11 |
| F11 | Wellfit 建置**必須**走 `Knowledge\tools\dev-env\build.ps1`，產物在 `%LOCALAPPDATA%\Temp\wellfit-target`；Wellfit 帳本為 SQLite；既有 `Wellfit\scripts\daily-backup.ps1`（ISO 27001 A.8.13 慣例：時戳＋完整性檢查＋失敗 exit 1） | `Wellfit\AGENTS.md` §3；`scripts\daily-backup.ps1:1-40` | 2026-11-11 |

## 2. 重大更正（推翻 A1 手冊 §1.2 三處假設）

1. `windmill serve --database-url "sqlite://…"` → **錯誤**。正確：環境變數 `MODE`＋`DATABASE_URL=postgres://…`，服務化用 `sc.exe`（F2/F3）。
2. `windmill db init` → **語法未驗證**；Windmill 以 Postgres migration 啟動時自動處理（`SKIP_MIGRATION` 控制），無 SQLite init 概念（F2）。
3. 「下載 Windows x64 原生二進位即可自建完整實例」→ **CE 無此路徑**；Windows 原生＝EE 付費功能（F3/F4）。A1 手冊 §1.2 需依本計畫修訂（T-A1）。

## 3. 路線決策（已裁定 2026-11-11）

| 路線 | 內容 | 成本 | 與約束的相容性 |
|---|---|---|---|
| **A：EE 完整版（原生 Windows）** | 採購 EE 授權；`windmill-ee.exe` 以 sc.exe 註冊 Server＋Worker 服務；PostgreSQL 18 為後端；workspace `dev`/`prod` 分離 | **付費授權**（Windows native workers 計 1–2 Compute Unit；價格待查詢） | ✔ 無 Docker；✔ Windows 原生；✔ 最佳完整版；△ 付費需裁定 |
| **B：CE 免費版（IT 另供 Linux 批次主機）** | Windmill server＋worker 跑在 IT 提供的 Linux 主機（母計畫 §12「批次主機另機」待定項）；Windows 端僅保留 Task Scheduler 觸發 | 免費；需 IT 另機 | ✔ 免費；✘ Windows 本機原生執行不可得（Windows worker 亦為 EE）；△ 依賴另機裁定 |
| **C：暫不導入（Task Scheduler 先行）** | A1 手冊已載明：「可先用 Task Scheduler 直接跑 run-mtrix-pipeline.ps1」；Windmill 標記範圍外，修正文件消除「已規劃未裝」落差 | 零成本零風險 | ✔ 全相容；✘ 無排程 UI／重試／告警平台 |

**建議：階段式 A/C 混合**——立即執行 C（Stage 0，不等決策）；同時向 IT/使用者提出 A 的授權詢價。若 A 獲准 → 進 Stage 1（下節架構）；若否 → 正式裁定 C 為終局，執行 T-Doc。

> **✅ 已裁定（2026-11-11，使用者）**：採階段式 C 先行＋評估 EE。Stage 0 立即執行；EE 詢價結果出來後二選一：獲准 → Stage 1；未獲准 → C 為終局（T-Doc）。

## 4. 目標架構（路線 A 成立時）

### 4.1 環境分離（測試區／正式區）
- **單一實例、雙 workspace**：`dev`（測試區）與 `prod`（正式區）；workspace 為 Windmill 原生隔離單位（腳本/流程/排程/變數互不可見）。
- **排程隔離**：每日 02:00 ERP 批次排程只存在於 `prod` workspace；`dev` 僅手動觸發驗證。
- **對齊 CI 四環境**：`feature/…→develop` 對映 `dev` workspace；`main`（production，需 PR＋審批）對映 `prod` workspace（F6）。

### 4.2 不亂覆蓋機制（C-01 對齊）
1. **Flows as code**：流程定義放 `Dev/windmill/`（進 git），以 `wmill sync push` 部署；禁 UI 手改為唯一來源。測試→正式一律經 git promotion，永不直接改 `prod`。
2. **二進位版本化安裝**：`C:\ProgramData\Windmill\versions\<ver>\windmill-ee.exe`，永不原地覆蓋；以 `sc.exe` binPath 切換；舊版保留 ≥1 個供回滾。
3. **DB 保護**：升級前 `pg_dump` 備份；`SKIP_MIGRATION=true`（worker）避免 worker 搶跑 migration；僅 server 執行 migration。
4. **SHA256 manifest**：每個安裝的二進位記錄版本＋SHA256（`Wellfit\artifacts\windmill\`），供漂移比對（沿用既有 config 快照制度）。

### 4.3 長遠演進政策
- **版本釘選**：不追最新（F5 迭代快）；候選釘選版＝**v1.867.0**（2026-11-10 發布，截至今日最新穩定），實際安裝日重驗並記錄 tag；每季評估升級一次，升級走 A/B（dev workspace 先行驗證）。
- **升級 runbook**：備份 → 新版本目錄 → sc 切換 → 冒煙（`sc query`＋`dev` workspace 測跑）→ 觀察 24h → 刪舊版。
- **漂移告警**：二進位 SHA256 與 service binPath 每日檢查（併入既有 T-08 監控）。
- **DB 生命週期**：每日 02:30 `pg_dump` 備份至 `Wellfit\artifacts\windmill\<ts>\`（沿用 daily-backup.ps1 慣例：時戳＋`pg_restore --list` 完整性驗證＋失敗 exit 1）；備份保留 30 日輪替；runs/logs retention 依版本設定調整，避免 DB 無限膨脹。

### 4.4 元件與路徑
| 元件 | 位置 | 備註 |
|---|---|---|
| PostgreSQL 18 | IT 既有或另裝（依 database_strategy） | Windmill 專用 DB `windmill`；連線僅本機 |
| `windmill-ee.exe` | `C:\ProgramData\Windmill\versions\<ver>\` | 來源：官方 releases；SHA256 入 manifest |
| 服務 | `WindmillServer`（MODE=server）、`WindmillWorker`（MODE=worker, SKIP_MIGRATION=true） | sc.exe＋failure recovery（官方文件）；server 綁 `127.0.0.1:8000` 僅本機，無需防火牆變更 |
| Flow 程式碼 | `Dev/windmill/`（git 受控） | 對映 A1 手冊 erp-batch-daily（路徑已依母計畫 §2.5-7 修正） |
| PowerShell 執行器 | PowerShell 7+（`POWERSHELL_PATH`） | 本機僅 PS 5.1（F8）；PW7 安裝列 T-1.0 前置 |

### 4.5 資源守門與安全（共享機器紅線）
1. **不影響其他程式**（母計畫 §0.5-4）：worker 併發設低（起始 1）；排程 02:00 離峰；工作可中止；採用前須過「系統級 A/B——其他代表程式 CPU/RAM 退步 >10% 即回滾」。
2. **憑證**：`DATABASE_URL` 僅存於服務 registry Environment（HKLM），**不入 git、不入文件**；DB 帳號最小權限（僅 `windmill` DB）。
3. **服務執行帳號**：LocalSystem 或專用低權帳號，由 IT 於變更管制裁定；寫入限 `C:\ProgramData\Windmill`、`C:\tmp\windmill\logs\`、`Wellfit\artifacts\`。

## 5. 分階段任務

**Stage 0（C，立即，零成本）**
- T-S0.1：Task Scheduler 排程落地（IT 於變更管制下建立）：
  - 動作：每日 02:00 執行 `Wellfit\scripts\erp-batch\run-erp-batch-scheduled.ps1`（新包裝腳本，進 git；**編排既有 `run-mtrix-pipeline.ps1`，不重複實作批次邏輯**——C-11 既存 .ps1 工具鏈之延伸）。
  - 包裝腳本規格：①單一實例守門（偵測前次執行未結束即退出 exit 3）；②以低優先啟動 Wellfit 子程序；③產出只寫 `Wellfit\artifacts\erp-batch\<ts>\`（RSK-E5）；④每步 exit code 傳遞，任一失敗即停並 exit 1；⑤寫入簡易 run log 供對帳；⑥遵守 Wellfit PS 5.1 規範：腳本 ASCII-only（無 BOM 非 ASCII 字面量；訊息以 ASCII 或 UTF-8 資料檔 `Get-Content -Encoding UTF8` 讀取）；⑦以**絕對路徑**呼叫 `%LOCALAPPDATA%\Temp\wellfit-target\debug\wellfit.exe`（排程以服務帳號執行，env 變數不保證展開）。
  - 條件：僅接 AC 電源時啟動（筆電情境）；不含 flapserp:// 自動開頁（人工觸點保留，待 T-15）。
- T-S0.2：A1 手冊 §1.2 依 §2 更正（SQLite→Postgres；EE 限制註記；PowerShell 7 前置）；母計畫 §12 Windmill 列同步更新。
- T-S0.3：向 IT/使用者提交路線 A 詢價與決策單（本計畫 §3），含 EE trial license 請求（供 POC）。
- **Stage 0 驗收**：排程存在（`Get-ScheduledTask`）；手動觸發首次跑 exit 0、artifacts 產出；執行期間瀏覽器/檔案總管無可感知退化（人工觀測記錄）；UNKNOWN 項標記不虛稱。

**Stage 1（A 獲准後，IT 執行）**
- T-1.0：前置盤點（全 UNKNOWN 開局，實測後轉 PASS/FAIL）：PostgreSQL 18 是否已存在；PowerShell 7 安裝；磁碟空間；pwsh/dotnet 路徑；`%LOCALAPPDATA%\Temp\wellfit-target\debug\wellfit.exe` 存在或可經 build.ps1 重建（F11）。
- T-1.1：PostgreSQL 18 備妥（新建 `windmill` DB＋最小權限專用帳號）。
- T-1.2：EE **試用**金鑰就位（僅試用，此階段不採購）→ 下載釘選版 `windmill-ee.exe`（候選釘選 v1.867.0；安裝日以當日最新穩定為準並記錄 tag）→ versions 目錄 → SHA256 manifest。**正式採購決策點在 T-1.3 POC 通過之後**（避免未驗證先花費）。
- T-1.3：**POC（閘門，最先做）**：單機 dev 驗證 MODE=server 於 Windows 原生可啟動（含驗證 EE 二進位無授權/試用授權的啟動行為——未記載，實測判定）；POC 失敗 → BLOCKED，回退 C，不浪費後續工項。
- T-1.4：sc.exe 註冊 Server＋Worker（含 failure recovery、環境變數登錄、§4.5 守門）。
- T-1.5：建立 `dev`/`prod` workspace；`wmill sync` 綁定 git repo。
- T-1.6：erp-batch-daily flow 以 code 部署至 `dev` → 手動測跑（exit 0）→ git promotion 至 `prod` → 冒煙。
- T-1.7：告警通道（A1 手冊 §Step 5 表）與 T-08 監控併接。
- T-1.8：升級 runbook 落地（§4.3）＋ 24h soak。
- T-1.9（註記，F9）：ci-config `windmill: true` 的 CI 自動佈線依賴 workflow 產生機制（現無 `.github/workflows/`）；機制就緒前，promotion 以「本地 `wmill sync push`＋PR 審批紀錄」人工流程執行，**不虛構自動化**。

**T-Doc（路線 C 為終局時）**：ci-config.yml `windmill: true` 移除或註記「改用 Task Scheduler」；A1/母計畫同步收斂，消除規劃落差。

## 6. 失敗模式與回滾
- 服務起不來 → Event Viewer（來源 WindmillServer/Worker）；`C:\tmp\windmill\logs\`；檢查 DATABASE_URL/執行器路徑環境變數。
- migration 失敗 → 回滾 binPath 至舊 versions 目錄＋`pg_restore` 備份。
- Windows 原生 server 為非主流記載路徑（文件主軸＝worker）→ **Stage 1 前置 POC（T-1.3）**：①驗證 EE 二進位在無授權／試用授權下的啟動行為（未記載，UNKNOWN）；②驗證 MODE=server 於 Windows 可用。POC 失敗 → 回退路線 C，視為 BLOCKED 證據。
- 授權未獲准 → 自動落 C，不阻塞 ERP 批次（Stage 0 已在跑）。
- Stage 0 回滾：`Unregister-ScheduledTask` 刪排程＋移除包裝腳本；無 DB 無服務，零殘留。

## 7. 驗收標準（五級）
- **PASS**：`sc query WindmillServer/Worker`=RUNNING；`dev`/`prod` workspace 存在；flow 由 git sync 部署；`prod` 排程 02:00 觸發 exit 0；SHA256 manifest 齊。
- **CONDITIONAL PASS**：核心可用但告警/升級 runbook 未備。
- **BLOCKED**：POC 證實 Windows server 不可用／EE 授權未准且 B 不可行 → 落 C。
- **REJECTED**：POC 或 A/B 實測顯示對其他程式退化（母計畫 §0.5 原則）。
- **NOT VERIFIED**：任何未量測之宣稱。

## 8. 決策紀錄（已解）
- 2026-11-11：路線裁定＝**階段式 C 先行＋評估 EE**（使用者確認）。Stage 0 不阻塞；EE 詢價為 Stage 1 閘門。
- 後續唯一閘門：EE 授權可否編列 → 准＝Stage 1；否＝T-Doc 終局收斂。
