# FlapsErp 案全項目完成計畫（母計畫剩餘收斂＋軌道 B／C）

> 計畫編號：1792463086698｜日期：2026-11-11｜**修訂 v1.2（2026-11-12）**｜狀態：**Active（規劃完成，待執行）**
> 修訂紀錄：
> - **v1.1（2026-11-12）**：P-08 排程錯峰治理（變更窗／量測／soak 當日不得與 02:00 批次重疊；批次排程註冊為「停用」，啟用閘門 C-4）；P-09 DPI 臂裁定「暫不 A/B」；新增 I-0 前置檢查（`wellfit.exe` 可執行性／WDAC）；§5 命令輸出路徑一律 artifacts（不用 `$env:TEMP`）；W0.1 驗收改 dry-run（W0.12 才是正式 Arm A 5×）；W0.9 檔名不變、檔內 v1.1、僅增量（C-01）；soak／系統級 A/B 驗收補「其他程式無 >10% 退步」與「保持喚醒不變更電源計畫」。
> - **v1.2（2026-11-12）**：**量測歸因修正**——Wave 1 內 I-3（T-05 A/B）必須先於 I-2（宿主變更）執行，並新增歸因規則與基線新鮮度備案（否則 config 效應與宿主效應混淆）；**soak 改工作日時段執行**（T-12 每小時 UI 循環需操作者在場，夜間無人不可行；併發負載即真實使用）；**T-07 `IGNORE_CERT_ERR` 決策閉環**掛接 soak 期 runtime 端點盤點；W0.1 COM 標記為對照／地板指標（非 GPU 臂判別指標）；M-2 補「混合→保留原值並記錄」（母計畫 §6.5）；操作者角色（＝使用者）與低干擾時段註明；W0.8/S-4 權責措辭精確化。
> 母計畫（SSOT，規格不重複）：`.kilo/plans/1794395575776-flaps-erp-product-system-settings.md`
> 關聯：`1794413131000`（Windmill Stage 0）、`1794365729846`（C-13 制度本體）
> 執行邊界（不變）：**代理唯讀**（量測／腳本／文件／驗收；不得重啟 FlapsErp）；任何寫入 `C:\SYSTEM_PROGRAM_DIRECTORY` 或 OS 由 IT 於變更管制執行（RSK-E5 不變）。
> 判定：五級（PASS／CONDITIONAL PASS／BLOCKED／REJECTED／NOT VERIFIED）；**UNKNOWN ≠ PASS ≠ FAIL**；每個數字附 artifact 路徑與日期。
> 腳本規範：PowerShell 5.1、ASCII-only 無 BOM 字面量（C-09）、既有 `.ps1` 工具鏈延伸（C-11）、只寫 `Wellfit\artifacts\`。

---

## 0. 目的與完成定義（DoD）

一輪內把母計畫**所有未完成項目**收斂到終態，並完成兩個相鄰軌道：

1. 母計畫 T-04..T-16 全部有終態（五級之一）且附 artifact；代理可執行者全數完成。
2. IT 依賴項做成 **turnkey 變更包**（runbook v1.1 §7）並於變更窗執行；未執行項明確標 NOT VERIFIED，不得虛稱。
3. 軌道 B：Windmill Stage 0（T-S0.1 腳本、T-S0.2 文件更正、T-S0.3 決策單）。
4. 軌道 C：C-13 vs 憲章／標準之**差異報告**（裁定權在使用者）。
5. 每一波次收尾即落 artifacts（必要時附該波 A–L 小結）；最終合併於 T-11 v2.0（A–L＋逐項與總判定）＋ `PLAN_REGISTER` addendum＋文件同步。

---

## 1. 現況（已核實 2026-11-11）

### 1.1 已完成（不再重做；證據見 AB 報告 v1.0 §B.8）
- T-00／T-00b（宿主＋全系統基線）、T-01（凍結基線）、T-02（`flaps-perf.ps1` 全 probe）、T-03（TLS/PE/endpoints）、T-09（文件更正）、T-10（基線文件）、T-15 fixture（`Wellfit/artifacts/erp-batch/20261111-194835/`）。
- 交付物已存在：`FlapsErp最佳設定基線_v1.0.md`、`FlapsErp設定變更runbook_v1.0.md`、`FlapsErp設定AB實測報告_v1.0.md`（NOT VERIFIED）、`PLAN_REGISTER` #33。
- 工具已存在：`Wellfit/scripts/perf/flaps-perf.ps1`、`Wellfit/scripts/erp-batch/run-mtrix-pipeline.ps1`。
- 相鄰軌道（範圍外）：Wellfit QG-01/02 已於後續實作——CLI Export 參數已由 `_` 改為綁定（傳遞鏈未深查，非本計畫範圍）；`PinnedCertVerifier` 在 `Wellfit/crates/wellfit_bankpay/src/http.rs`。

### 1.2 未完成清單（＝本計畫全部工作項）

| # | 項目（來源） | 執行者 | 波次 |
|---|---|---|---|
| 1 | T-04 Tier 0 宿主套用＋前後量測（本機僅適用：AV 排除、AP 白名單、時間） | IT＋代理 | W1/W2 |
| 2 | T-05 硬體加速 A/B（Arm A 先測；Arm B flip；**兩臂須同宿主狀態——W1 內 I-3 先於 I-2**）；**DPI 臂暫不 A/B（P-09）** | 代理＋IT | W0/W1/W2 |
| 3 | T-06 OS 進階（條件式；無抖動實證則結案「不適用」） | IT | W2 |
| 4 | T-07 AP／網路／TLS（白名單、測試 AP 可達性裁定、`IGNORE_CERT_ERR` 決策；**決策閉環含 soak 期 runtime 盤點**） | IT | W1/W3 |
| 5 | T-08 排程／監控／告警（腳本由代理建；註冊由 IT；批次排程啟用閘門在 W4，見 P-08） | 代理＋IT | W0/W1/W4 |
| 6 | T-11 A/B 報告最終版（v2.0，A–L＋五級） | 代理 | W4 |
| 7 | T-12 8h soak | IT＋代理＋**使用者**（operator-in-loop） | W3 |
| 8 | T-13 分頁／程序衛生（腳本＋rollout） | 代理＋IT／使用者 | W0/W3 |
| 9 | T-14 夜間重啟（條件式，依 soak 結果；與批次時段錯開） | IT | W3/W4 |
| 10 | T-15 A1／Wellfit／報表外移（生產批次、報表外移、深層連結、bench） | 開發＋IT＋使用者 | W3 |
| 11 | T-16 備份／更新演練 | IT | W1/W3 |
| 12 | §12 未決收斂（FORCE 鍵註記、批次主機決策輸入、runtime 端點監控） | 代理 | W0 |
| 13 | Windmill S0.1 包裝腳本 `run-erp-batch-scheduled.ps1` | 代理 | W0 |
| 14 | Windmill S0.2 A1 手冊 §1.2 更正＋母計畫 §12 同步 | 代理 | W0 |
| 15 | Windmill S0.3 EE 詢價／決策單（草稿＋提交） | 代理＋使用者/IT | W0 |
| 16 | C-13 vs 憲章／標準差異報告 | 代理 | W0 |
| 17 | 冷啟動系列（需重開機；無法由代理執行） | IT＋代理 | W1（選配） |

---

## 2. 新增決策（本計畫）

| ID | 決策 | 理由 |
|---|---|---|
| P-01 | 首波只做 FlapsErp-only＋安全項（AV 最小排除／防火牆白名單／時間）；系統級 S/C 預設**不做**（本機為行動平台） | 母計畫 §4.3／§0.5 |
| P-02 | 量測一律以既有 `.ps1` 工具鏈擴充；不新增 Python（C-11） | C-11 |
| P-03 | IT 變更包＝runbook **檔名不變、檔內版本升 v1.1**，新增 §7「本機封閉式執行包」＋§8 表單；僅增量、不刪改既有節（C-01） | SSOT＋C-01 |
| P-04 | 8h soak 為整體 PASS 必要條件；未跑只能 CONDITIONAL PASS／NOT VERIFIED | 母計畫 D-15 |
| P-05 | Windmill 只做 Stage 0 三項；EE 決策為後續唯一閘門 | `1794413131000` §5 |
| P-06 | C-13 對齊只出差異報告，不擅自裁定效力順位 | 制度條文 |
| P-07 | Arm A 量測（現值 `DISABLE_HARDWARE_ACCELERATION=0`）在 Wave 0 先跑完，變更窗只需 Arm B | 效率；A 臂零寫入 |
| P-08 | **排程錯峰治理**：IT 變更窗、A/B 量測、8h soak 的**當日**不得與 02:00 批次重疊；批次排程（I-5）註冊為「**存在但停用**」，啟用閘門＝T-12 soak PASS＋資源守門演練（C-4） | 母計畫 §6.1（量測期禁其他批次）＋§4.5.2 |
| P-09 | **DPI 臂暫不 A/B**：現況無 UI 裁切實證（1536×776／1536×864）＋`FORCE_DEVICE_SACLE_FACTOR` 行為 UNKNOWN（原廠未確認）；列入追蹤，僅在出現裁切且原廠確認鍵行為後才評估 | 母計畫 D-03／§12 |
| P-10 | A/B／soak 進行時段**避開監控排程**（09:00/14:00/18:00）或由 IT 暫停監控排程、量測後恢復 | 母計畫 §6.1 量測紀律 |
| P-11 | **量測歸因隔離**：Wave 1 內 T-05 A/B（I-3）先於宿主變更（I-2）執行；T-04 前後對照之基線歸屬見 Wave 1 歸因規則 | 避免config×宿主效應混淆（母計畫 §6.1） |
| P-12 | **soak 於工作日時段執行**（操作者在場執行每小時 UI 循環；併發負載即真實使用），非夜間 | T-12 協議需人工循環；夜間無操作者不可行 |

---

## 3. 執行波次（有序；無時間估計）

### Wave 0 — 代理（現在；零 IT、零原廠寫入）

| ID | 任務 | 交付物 | 驗收 |
|---|---|---|---|
| W0.1 | A/B harness | `Wellfit/scripts/perf/flaps-ab.ps1`（新）：`-ArmLabel A|B -Runs 5 [-Cold]`；operator-in-loop（**操作者＝使用者**；提示重啟 app；代理不得重啟）；指標定義沿用母計畫 §6.4（module→menu、RAM／行程含 `--type` 角色、COM）；**COM 為對照／地板指標，非 GPU 臂判別指標**；彙總 median/min/max；輸出 `perf-result.json` 系列＋`ab-decision.md` | **乾跑 1 筆**成功產出；路徑全在 artifacts |
| W0.2 | 系統級 A/B harness | `Wellfit/scripts/perf/flaps-system-ab.ps1`（新）：`Get-Counter` 系統 CPU/RAM/磁碟佇列；代表程式回應以人工欄位記錄；輸出前後對照 | 與母計畫 §6.9／§6.10 欄位一致 |
| W0.3 | soak 儀器 | `Wellfit/scripts/perf/flaps-soak.ps1`（新）：8h；每 5 min RAM/行程、每 10 min app log、每小時 Event Log＋操作者 UI 循環提示（母計畫 T-12 循環）；守門（RAM>85% 或佇列>2.0 持續 5 min 即中止） | 1h 試跑（**dry＝僅驗證儀器與取樣；UI 循環可略過**）成功；輸出 `soak-report.md` |
| W0.4 | 監控／漂移／告警 | `Wellfit/scripts/perf/flaps-monitor.ps1`（新）：`-Check all|perf|drift|alerts`；drift＝7 鍵／`.ver`／AP JSON vs 快照 manifest；告警寫 `artifacts/perf/alerts.log`＋exit code；內含 runtime HTTPS 端點監控 hook（接 W0.10／S-1） | `-Check drift` 正確辨識**已知漂移**（`8E2EDBE…version.txt`；AB 報告 §B.1）且其餘檔案判定一致 |
| W0.5 | 單一實例／衛生 | `Wellfit/scripts/flaps-single-instance.ps1`（新）：偵測重複行程→提示、**不 kill**、exit 3；行程數報告 | 實測：1 實例 exit 0；模擬重複 exit 3 |
| W0.6 | Windmill S0.1 腳本 | `Wellfit/scripts/erp-batch/run-erp-batch-scheduled.ps1`（新）：依 `1794413131000` §5 T-S0.1 ①–⑦（單一實例守門 exit 3、低優先、只寫 artifacts、exit code 傳遞、run log、ASCII-only、絕對路徑 exe）；**AC 條件以排程條件為主（IT）、腳本內 guard 為輔** | 手動觸發 exit 0；artifacts 產出；不寫原廠目錄 |
| W0.7 | Windmill S0.2 文件 | 更正 `docs/04-operations/A1檔案通道端到端操作手冊_v1.0.md` §1.2（SQLite→Postgres、EE/Windows 限制、PW7 前置）；母計畫 §12 Windmill 列同步 | 與 `1794413131000` §2 三項更正一致 |
| W0.8 | Windmill S0.3 決策單 | `docs/04-operations/Windmill導入_EE詢價與決策單_v1.0.md`（詢價問題、試用請求、A/B/C 決策欄） | 文件就緒並**交付使用者送出**（送出權在使用者） |
| W0.9 | IT 執行包 | runbook 檔內升 **v1.1**：§7 本機封閉式執行包（僅適用項、精確命令、前後擷取命令、T-05 flip 程序、判定表、回滾演練）；§8 表單；**僅增量不刪改既有節（C-01）** | 每命令可複製直用；與母計畫 §4.8／§6 一致 |
| W0.10 | 盤點加深（§12） | (a) runtime 端點監控 hook 說明（併 W0.4／S-1）；(b) `FORCE_DEVICE_SACLE_FACTOR` 保持 UNKNOWN，落點＝T-11 v2.0 §K＋併入既有 OQ 追蹤（不另設制度）；(c) 批次主機決策輸入表（另機 vs 同機對照）→ 併 T-11 v2.0 §K | 三項皆有書面落點 |
| W0.11 | C-13 對齊 | `docs/standards/evidence-driven-doctrine-alignment.md`（新）：逐條差異＋建議（不裁定） | 差異清單可逐條追溯 |
| W0.12 | Arm A＋基線強化 | `flaps-perf.ps1 -Probe full` 5×（暖）；Arm A 5×（W0.1 harness）；`-Probe com` 補測；`-Probe app` 解析最新 log（menu ready 樣本累計）；結果先入 artifacts，彙整於 v2.0（**操作者＝使用者；低干擾時段、關閉其他應用**） | 新增 timestamped artifacts；median/min/max |
| W0.13 | T-11 v2.0 骨架 | `docs/research/FlapsErp設定AB實測報告_v2.0.md` 骨架（A–L＋判定表模板，資料待填） | 結構完整、無預填假數字 |

### Wave 1 — IT 變更窗（A；依 runbook v1.1 §7）

> **執行順序：I-0 → I-1 → I-3 → I-2 → I-4 → I-5 → I-6 → I-7。** I-3 必須先於 I-2：T-05 兩臂須在**同宿主狀態**下量測，否則 config 效應與宿主效應混淆（P-11；母計畫 §6.1 量測隔離）。

| ID | 任務 | 執行 | 驗收 |
|---|---|---|---|
| I-0 | **前置檢查**：`%LOCALAPPDATA%\Temp\wellfit-target\debug\wellfit.exe` 存在且可執行（若需重建：`build.ps1` 後**重驗執行**——WDAC 可能封鎖新產物）；`Wellfit\artifacts\` 可寫；2026-11-03 config 快照 manifest 在位（drift 比對基準） | 代理（唯讀檢查）＋IT（如需重建） | 三項檢查有輸出紀錄；任一不成立先補齊再進窗 |
| I-1 | 備份（7 檔群＋SHA256 manifest）；公告／回滾窗口；**當日無批次（P-08：批次排程尚停用）** | IT | `SHA256_MANIFEST.txt` 齊 |
| I-3 | **（先於 I-2 執行）** T-05 Arm B flip（`DISABLE_HARDWARE_ACCELERATION=1`）→ 代理測 5× → 判定 → 保留勝者（IT 僅 2 次寫入：flip 與定案） | IT＋代理 | 逐行替換、CRLF、UTF-8 無 BOM；`-Probe settings` 逐字；**兩臂同宿主狀態** |
| I-2 | 僅適用主機項：Defender 最小排除（`FlapsErp`／`Module`／`FlapsErpData`）、AP 防火牆白名單（`210.64.124.59:10578`）、時間/NTP 同步；**測試 AP Block 為獨立裁定項**——本機目前**可達**測試 AP（CERT-VALID 探測成功，§B.3），Block 會改變此行為，需使用者/IT 明示裁定 | IT | `Get-MpPreference`、`Test-NetConnection`、`Get-TimeZone` 前後輸出；測試 AP 裁定紀錄 |
| I-4 | 冷啟動（選配）：**每次冷啟前重開機**，共 3×（使用者同意；否則冷啟動標 UNKNOWN，不阻判定） | IT＋操作者（使用者） | 冷/暖分開記錄 |
| I-5 | Task Scheduler 註冊：**每日監控**（`flaps-monitor.ps1`；09:00/14:00/18:00）＋**批次排程（02:00）註冊為「存在但停用」**（啟用閘門＝C-4，P-08） | IT | `Get-ScheduledTask` 顯示兩項；批次項狀態＝Disabled；監控項手動 run 成功 |
| I-6 | T-16 預演：config 還原演練（複製回原檔＋SHA256 對照；必要時才真還原） | IT | 還原演練紀錄 |
| I-7 | app smoke 六步（母計畫 §6.8）＋任一失敗即回滾 | 使用者＋代理 | 六步紀錄 |

> **歸因規則（P-11）**：T-05＝Arm A（W0.12）vs Arm B（窗內、I-2 前）＝**純 config 效應**。T-04＝M-1（新宿主＋定案 config）vs「**定案 config 下之窗前基線**」——勝者為 Arm A（config 回 0）時基線＝W0.12；勝者為 Arm B（config 留 1）時基線＝Arm B 5 筆。**基線新鮮度備案**：若 W0.12 與變更窗間隔過長（環境漂移疑慮），於窗內 I-2 前補測 Arm A′ 5×（config=0、同宿主）與 W0.12 對照校驗；一致則採 W0.12，不一致則以 Arm A′ 為 T-05 Arm A 與 T-04 基線（並記錄差異原因）。

### Wave 2 — 代理量測（窗後；時段遵守 P-08/P-10）

| ID | 任務 | 驗收 |
|---|---|---|
| M-1 | 窗後 `-Probe full` 5×（暖）＋與歸因規則指定之基線對照；量測時段避開監控排程或請 IT 暫停後恢復 | 前後對照表；任一退步 >10% 記錄 |
| M-2 | T-05 判定（A vs B）：median/min/max；任一惡化 >10% 保留原值；**混合結果→保留原值並記錄**（母計畫 §6.5）；DPI 臂記「暫不評估（P-09）」 | `ab-decision.md`；無崩潰 |
| M-3 | 系統級 A/B（代表程式＋併發負載；母計畫 §6.9／§6.10） | 任一程式退步 >10% 或降頻→回滾 |
| M-4 | T-06 決策：僅在有抖動實證時提 C 項變更；否則結案「不適用（無實證）」 | 決策紀錄（含證據或 UNKNOWN） |

### Wave 3 — 穩定性與工作台

| ID | 任務 | 驗收 |
|---|---|---|
| S-1 | T-12 8h soak（**工作日時段執行（P-12）：操作者在場執行每小時 UI 循環；併發負載＝真實使用（瀏覽器／Excel／Outlook）**；AC；人工保持喚醒、不變更電源計畫（P-01）；當日無批次（P-08）、避開 ledger 備份（`daily-backup.ps1`）與 Update；代理儀器；**額外產出：soak 期 app log／AP 連線紀錄＝runtime HTTPS 端點盤點資料，餵給 T-07 `IGNORE_CERT_ERR` 決策閉環**） | RAM 增長 <50 MB；無崩潰；Event Log 無錯誤；**其他程式無 >10% 退步**；runtime 端點清單產出 |
| S-2 | T-13 rollout：單一實例部署（W0.5 腳本）＋衛生；行程數對照 11 基線 | 行程數／游離 RAM 下降；無 DLL 重複註冊 |
| S-3 | T-14：僅 soak 洩漏→排程夜間重啟（IT；**與（未來）批次時段錯開**） | 8h 增長 <50 MB；不影響批次 |
| S-4 | T-15：生產批次 `wellfit.exe mtrix --dir <唯讀 delivered> --report/--db <artifacts>`＋冪等重跑；報表匯出（**由使用者於 FlapsErp 操作**）→Wellfit 解析（≤60 s／10k 列）；`flapserp://` 深連結驗證；`bench.ps1` 每檔耗時 | 合成/真實分開標記；exit code 與工時記錄 |
| S-5 | T-16：ledger 還原（`restore.ps1`）＋config 復原演練；更新後基線重跑（併 W0.4 drift） | 演練成功；無漂移殘留 |

### Wave 4 — 收斂（代理＋IT）

| ID | 任務 | 驗收 |
|---|---|---|
| C-1 | T-11 v2.0 完成（A–L；逐項＋總判定五級；v1.0 頭註 SUPERSEDED by v2.0） | 每個數字附 artifact；無 UNKNOWN 冒充 |
| C-2 | `PLAN_REGISTER` addendum（下一號）＋`docs/README` 索引＋風險表 v1.3＋母計畫 §12 結案列 | 索引無死鏈 |
| C-3 | 母計畫狀態更新（依判定：Closed／Conditional／BLOCKED） | 與 C-1 判定一致 |
| C-4 | **批次排程啟用閘門（IT）**：條件＝T-12 soak PASS＋資源守門演練（CPU>80%/RAM>85% 持續 5 min 中止機制實測）＋與維護窗錯峰裁定→`Enable-ScheduledTask` | 啟用後首跑 exit 0＋run log；守門紀錄 |

---

## 4. 逐項終態判定矩陣（收斂條件）

| 項目 | 終態判定條件 | 未達時 |
|---|---|---|
| T-04 | 前後命令輸出（歸因規則指定基線）＋系統級 A/B 無 >10% 退步 | NOT VERIFIED（IT 未執行） |
| T-05（硬體加速） | 兩臂各 5 筆（同宿主狀態）＋判定表＋無崩潰 | CONDITIONAL（樣本不足）／NOT VERIFIED |
| T-05（DPI） | 決策紀錄「暫不評估（P-09）」即終態；裁切出現＋原廠確認後重開 | —（追蹤項） |
| T-06 | 決策紀錄（變更或「不適用」皆為終態） | NOT VERIFIED |
| T-07 | 白名單連通＋測試 AP 裁定紀錄＋`IGNORE_CERT_ERR` 決策（**依據：T-03 靜態掃描＋soak 期 runtime 端點盤點**） | NOT VERIFIED |
| T-08 | 監控排程存在＋手動 run 成功＋告警可觸發；批次排程=已註冊（啟用依 C-4） | CONDITIONAL（未註冊） |
| T-11 | A–L 完整＋五級判定 | — |
| T-12 | RAM 增長 <50 MB、無崩潰、Event Log 無錯誤、其他程式無退步 | NOT VERIFIED（未跑） |
| T-13 | 行程數下降＋腳本部署 | CONDITIONAL |
| T-14 | soak 無洩漏→「不啟用」即終態；有洩漏→排程完成且與批次錯開 | NOT VERIFIED |
| T-15 | 生產批次 exit 0＋報表 ≤60 s／10k＋深連結可用 | CONDITIONAL |
| T-16 | 還原演練成功＋更新後無漂移 | NOT VERIFIED |
| 批次排程啟用 | C-4 條件全滿＋啟用後首跑 exit 0 | 停用狀態（不計 FAIL） |
| S0.1–S0.3 | 腳本手動 exit 0；文件更正；決策單送出 | — |
| C-13 對齊 | 差異報告完成（裁定另候使用者） | — |

---

## 5. 驗證指令集（可直接複製）

```powershell
# 前置（I-0；代理唯讀檢查）
Test-Path "$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe"
Test-Path "Wellfit\artifacts"

# 量測（代理；唯讀；時段遵守 P-08/P-10；操作者＝使用者配合重啟 app；低干擾時段）
powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\perf\flaps-perf.ps1" -Probe full
powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\perf\flaps-perf.ps1" -Probe com
powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\perf\flaps-ab.ps1" -ArmLabel A -Runs 5
powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\perf\flaps-soak.ps1" -Hours 1   # dry 試跑（儀器驗證；UI 循環可略過）
powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\perf\flaps-soak.ps1" -Hours 8
powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\perf\flaps-monitor.ps1" -Check drift

# 變更窗（IT；依 runbook v1.1；順序 I-0→I-1→I-3→I-2→I-4→I-5→I-6→I-7）
Get-MpPreference | Select-Object -ExpandProperty ExclusionPath
Test-NetConnection 210.64.124.59 -Port 10578
Get-ScheduledTask | Where-Object { $_.TaskName -like '*Flaps*' -or $_.TaskName -like '*Wellfit*' } | Select-Object TaskName,State
powercfg /getactivescheme

# 工作台（開發；自建；輸出一律 artifacts）
$ts = Get-Date -Format "yyyyMMdd-HHmmss"
$out = "Wellfit\artifacts\erp-batch\$ts"
powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\erp-batch\run-erp-batch-scheduled.ps1"
& "$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe" mtrix --dir "spec\fixtures\mtrix\delivered" --report "$out\report.json" --db "$out\ledger.sqlite"
```

---

## 6. 風險與回滾

- **IT 未指派（關鍵路徑）**：IT 項全部 NOT VERIFIED；代理項照做不阻塞。
- **量測噪音**（共用筆電）：依母計畫 §6 紀律（同機／同時段／5×／冷暖分離／量測期不跑 cargo build／P-10 避開監控）。
- **歸因混淆**：config 效應 vs 宿主效應——P-11 順序強制＋Wave 1 歸因規則＋基線新鮮度備案。
- **撞期風險**：變更窗／量測／soak／02:00 批次互相污染——P-08 錯峰＋批次停用至 C-4。
- **soak 時段/中斷/睡眠**：soak 必須於**工作日時段**執行（每小時 UI 循環需操作者在場；夜間無人不可行）；AC 供電＋人工保持喚醒（不變更電源計畫）＋避開 Update／ledger 備份／批次（P-08）；中斷即重跑，不得拼接。
- **flip 遺留**：T-05 結束必確認 `-Probe settings` 與判定一致；回滾＝備份還原＋SHA256。
- **腳本越界**：所有新腳本只寫 `Wellfit\artifacts\`；違反即缺陷（RSK-E5）。
- **WDAC 重建風險**：I-0 若重建 `wellfit.exe`，新產物可能被應用程式控制原則封鎖（os error 4551）；重建後必先重驗執行，不可逕行排程。
- **判定紀律**：無 artifact 不得宣稱；UNKNOWN 保留；P-04 不放寬。

---

## 7. 阻礙與未決（需使用者/IT 裁定；附建議）

| 項目 | 建議 |
|---|---|
| IT 變更所有者／窗口 | 指派既有 IT 窗口＋指定本機為受測機，維護窗 02:00–05:00；否則本計畫 IT 項維持 NOT VERIFIED |
| 批次主機（另機 vs 同機） | 優先另機；否則離峰 02:00–05:00＋BelowNormal＋守門（母計畫 §0.5-4／§4.5.2） |
| 批次排程啟用時機 | 依 C-4：soak PASS＋守門演練後（P-08） |
| soak 時段 | 工作日時段（P-12；操作者在場）；無需 IT 窗，但該日避開批次/備份/Update |
| 測試 AP 可達性 | 本機目前可達（§B.3）；是否 Block 由使用者/IT 裁定（I-2） |
| Windmill EE 詢價 | Wave 0 即送出（並行；不阻塞 Stage 0） |
| 原廠項（`FORCE_DEVICE_SACLE_FACTOR` 行為、AP 轉 HTTPS） | 保持 UNKNOWN＋問題單追蹤（OQ-F1/F2） |

---

## 8. 範圍外（明確）

COM/CDP 自動化正式化（Phase 3–4）；64-bit 重打包（OQ-F1/F2）；Rust 平台線（`platform/`／`core/`／`ekpi-platform`）；Wellfit 線（QG-01/02 已完成；`reconcile` 實作另排）；EE 採購決策本身；非 FlapsErp 專案。

---

*本計畫僅規劃；所有寫入原廠目錄或 OS 之變更由 IT 於變更管制下執行。未量測標 UNKNOWN；UNKNOWN ≠ PASS ≠ FAIL。*
