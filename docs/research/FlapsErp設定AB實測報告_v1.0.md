# FlapsErp 設定 A/B 實測報告 v1.0

> 報告編號：RPT-2026-1111-001｜日期：2026-11-11｜狀態：**NOT VERIFIED**（僅完成唯讀證據；IT 變更/系統級 A/B/8h soak 未執行）
> 範圍：FlapsErp v0.0.4＋共用 Windows 宿主；對應計畫 `1794395575776`。
> 紅線：代理唯讀；不得寫入 `C:\SYSTEM_PROGRAM_DIRECTORY`；不得降低其他程式效能。

---

## A. 目標與範圍
- 在不改原廠原始碼前提下，建立 FlapsErp 產品/宿主/衛生/工作台的完整設定基線，並以 A/B 證明可量測改善。
- 非目標：原廠二進位修改、ERP 伺服器端、COM/CDP 正式化、Rust 平台線、64-bit 重打包。

## B. 現況證據（2026-11-11 唯讀）

### B.1 `config.ini`（`-Probe settings`）
- 7 鍵：`IGNORE_CERT_ERR=1`、`KEEP_MACHINE_ID=1`、`DISABLE_HARDWARE_ACCELERATION=0`、`DEBUG_FASTREPORT=0`、`FORCE_DEVICE_SACLE_FACTOR=0`、`LANG=zh_TW`、`KEEP_MACHINE_ID_MIGRATED=1`。
- AP：3 個；預設 `8E2EDBE64B3F0747B90EC071E4548BE8`（大前集團 WELLFIT）HTTP `210.64.124.59:10578`；測試 AP HTTPS `test2.flaps.tw:10113`。
- Module：`POSMachineController.CD8240.jsc.ver=0.0.4`、`Winax.node.ver=0.0.4`。
- Manifest drift：**1/9**；`8E2EDBE64B3F0747B90EC071E4548BE8.version.txt` 目前 `7A4C6E…` vs snapshot `B82297…`（AP 版本更新）。
- `softwareId.txt`：目前＝manifest `D5E6B4350C1EBC0F049B0D69AD71668EFFFB716D525B42090C4C1D38567CA453`（基線報告原 `…4010EE…` 為繕寫錯誤，已更正）。

### B.2 App log（`-Probe app`）
- Log：`application-WELLFIT-20261111.log`；`erpRunning=False`（探測時未啟動）。
- `moduleToMenuMs=17087`（單次日誌；16:20:40.310→16:20:57.397）。
- `downloadDoneRelMs=1465`；`detailsRelMs=1673`。
- `menuToFirstTabMs=UNKNOWN`（無 tab ready 標記）；`firstTabAfterMenuMs=300308`（事件差，非載入延遲）。
- `window=1536x776`、`screen=1536x864`（≈125%）。

### B.3 TLS（`-Probe tls`）
- `test2.flaps.tw:10113`：**CERT-VALID**；CN=test2.flaps.tw；Let's Encrypt；TLS 1.2；到期 2027-01-21；hostname/鏈/撤銷 OK。
- 預設生產 AP 為 HTTP；`IGNORE_CERT_ERR` 對生產無作用。
- `app.asar` 掃描：無其他 `https://...flaps/wellfit...` literal 端點（`flapsWellfit=0`；artifact `Wellfit/artifacts/perf/20261111-195857/endpoints.json`，asar 11,832,082 bytes）；更新/授權端點若存在可能由 config/變數提供，需 runtime 監控。

### B.4 PE（`-Probe pe`）
- `FlapsErp.exe`：i386；**LAA=True**；`Characteristics=0x0122`；x64 下 32-bit 可用 4 GB（非 2 GB）。

### B.5 既有基線
- 2026-11-03：11 行程／RAM median 46.2 MB／max 103.9 MB／總 623.8 MB；COM P50/P95/P99 0.2636/0.3884/0.4740 ms；CDP NOT-FOUND。
- 2026-11-06：COM 0.1869/0.2634/0.3394 ms。
- 2026-11-11 全量（`Wellfit/artifacts/perf/20261111-194436/`、`/194739/`、`/195713/`、`/195906/`）：COM P50/P95/P99 `0.1736–0.1882/0.2069–0.3171/0.2405–0.4467` ms（四次取樣；P95 約 53%、P99 約 86% 差距）；CDP NOT-FOUND；baseline APP-NOT-RUNNING；`-Probe host` 併入 `system-baseline.json`。

### B.6 宿主基線（`-Probe host`，2026-11-11）
- OS：Windows 11 Home build 26300（64-bit）。
- CPU：AMD Ryzen 7 4800HS（8C/16T）；load 7–73%（取樣；受其他程式/本工具影響）。
- RAM：15.79 GB total；3.4–5.0 GB free（取樣）。
- 磁碟佇列 0–1（取樣）；溫度 UNKNOWN。
- Top（代表值，會隨取樣變動）：Code 991/623/233/149 MB、Memory Compression 796、kilo 744、MsMpEng 308、ChatGPT 287/189/178、explorer 273、msedgewebview2 253/206、chrome 215/172/170/143、powershell 163、msedge 132、OUTLOOK 129。
- 意義：本機為行動平台（推測筆電）、記憶體只剩約 3.4–5.0 GB free；FlapsErp 11 行程與批次會再擠；不套用高效能電源/快速啟動/USB/NIC/GPU/視覺/搜尋變更；FlapsErp-only 為主。

### B.7 Wellfit MTRIX fixture（T-15 驗證）
- 命令：`powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\erp-batch\run-mtrix-pipeline.ps1"`。
- 首跑：exit 0；2085 ms；5 檔、25 列、37 修復、22 errors/29 warnings（partial-accepted）。
- 冪等重跑：exit 1（duplicate batch 正確拒絕）。
- RUN_DIR：`Wellfit/artifacts/erp-batch/20261111-194835/`。
- 意義：外部工作台流程可重現；**fixture 非生產 SLA**。

### B.8 證據索引（可追溯）
| 主張 | 命令 | Artifact |
|---|---|---|
| config 7 鍵 | `flaps-perf.ps1 -Probe settings` | `Wellfit/artifacts/perf/20261111-195906/perf-result.json` |
| AP 預設 HTTP／default `8E2EDBE` | `-Probe settings` | 同上 |
| 測試 AP CERT-VALID | `-Probe tls` | 同上 |
| app.asar 無其他 flaps/wellfit HTTPS | `-Probe endpoints` | `Wellfit/artifacts/perf/20261111-195857/endpoints.json`（`flapsWellfit=0`） |
| module→menu 17.087 s | `-Probe app` | `20261111-195906/perf-summary.md` |
| LAA=True（0x0122） | `-Probe pe` | 同上 |
| COM 取樣 | `-Probe com` | `20261111-194436/`、`194739/`、`195713/`、`195906/` |
| 宿主基線 | `-Probe host` | `20261111-200314/system-baseline.json`（含 netStats；全量 `195906`） |
| MTRIX fixture | `run-mtrix-pipeline.ps1` | `Wellfit/artifacts/erp-batch/20261111-194835/` |
| softwareId 更正／manifest drift 1/9 | `-Probe settings` | `20261111-195906/perf-result.json` |

## C. 決策
- 代理唯讀；IT 套用（RSK-E5）。
- 產品設定只動 7 鍵；只 A/B `DISABLE_HARDWARE_ACCELERATION`；`FORCE_DEVICE_SACLE_FACTOR` 僅 UI 裁切時評估。
- `IGNORE_CERT_ERR` 維持 1，直到 HTTPS 端點盤點完成。
- 外部工作台（A1/Wellfit/報表外移）為主要效益；批次必須另機/離峰/低優先/可中止。
- 系統級變更需 IT 核准＋系統級 A/B；不得以 FlapsErp 效能為由拖垮其他程式。

## D. 變更內容（尚未套用）
- 產品：7 鍵（見 B.1）；`DISABLE_HARDWARE_ACCELERATION` A/B；`IGNORE_CERT_ERR` 條件式。
- 宿主：電源/睡眠/快速啟動/USB/分頁檔/NTFS/AV/WDAC/UAC/PowerShell/時間/NIC/GPU/搜尋/視覺——均標 S/C，見基線文件 §3。
- AP/網路/TLS：預設 AP 不變；防火牆白名單；測試 AP 生產不可達；HTTPS/CA 條件式。
- 衛生/工作台：單一實例、關閉多餘分頁、夜間重啟（條件式）、A1/Wellfit/報表外移。

## E. 量測方法
- 工具：`Wellfit/scripts/perf/flaps-perf.ps1`（`-Probe settings/app/tls/pe/host/endpoints/baseline/com/cdp/all/full`）。
- 每臂 5 次；冷/暖啟動分開；同一台機器/使用者/AP/時段；不跑 `cargo build`。
- 系統級 A/B：FlapsErp＋瀏覽器 10 tabs＋Excel 10 MB＋Outlook＋1 GB 檔案複製；量測每個程式回應/系統 CPU/RAM/磁碟佇列/GPU/溫度。

## F. 實測結果（目前）
| 項目 | 結果 | 狀態 |
|---|---|---|
| config 7 鍵 | 與實機一致 | ✔ |
| AP/TLS | 預設 HTTP；測試 AP CERT-VALID | ✔ |
| PE LAA | True（0x0122） | ✔ |
| module→menu | 17.087 s（單次日誌） | ✔（基線） |
| RAM/COM | 既有 2026-11-03/06 基線；2026-11-11 COM 取樣 | ✔（歷史/取樣） |
| 硬體加速 A/B | 未執行 | UNKNOWN |
| DPI A/B | 未執行 | UNKNOWN |
| OS 變更前後 | 未執行 | UNKNOWN |
| 系統級 A/B | 未執行 | UNKNOWN |
| Wellfit MTRIX fixture | exit 0；冪等重跑 exit 1 | ✔（fixture） |
| 生產批次 | 未執行 | UNKNOWN |
| 8h soak | 未執行 | UNKNOWN |

## G. 對照與判定（待 IT）
- 採用門檻：FlapsErp 指標改善 ≥10%；其他程式/系統指標退步 ≤10%；溫度無降頻。
- 未達門檻記錄但不改；安全變更採另一把尺（安全提升且無功能破壞可接受，需記錄）。

## H. 風險與回滾
- RSK-E5 寫入原廠目錄：代理唯讀；IT 變更管制。
- RSK-E2/E10 AP HTTP/憑證：受控 LAN；要求 HTTPS。
- RSK-E3 32-bit LAA 4 GB：分頁衛生/報表外移/夜間重啟。
- RSK-S1..S6 系統級互動：FlapsErp-only 優先；系統級 A/B；批次守門。
- 回滾：config 從 `Wellfit/artifacts/config-backup/<date>/`＋SHA256；OS 依 runbook 原值；排程 `Unregister-ScheduledTask`。

## I. 合規與紅線
- 未修改 `FlapsErp.exe`／`app.asar`／任何原廠二進位。
- 未寫入 `C:\SYSTEM_PROGRAM_DIRECTORY`。
- 未關閉 AV／UAC／WDAC；未新增未證實 config 鍵。

## J. 文件與追溯
- 計畫：`.kilo/plans/1794395575776-flaps-erp-product-system-settings.md`。
- 基線：`docs/04-operations/FlapsErp最佳設定基線_v1.0.md`。
- Runbook：`docs/04-operations/FlapsErp設定變更runbook_v1.0.md`。
- 手冊更正：`docs/04-operations/FlapsErp效能優化操作手冊_v1.0.md` §3.2/§3.4。
- A1 手冊更正：`docs/04-operations/A1檔案通道端到端操作手冊_v1.0.md`。
- 基線報告補充：`docs/research/FlapsErp效能基線報告_v1.0.md` §7.3。
- 風險表：`docs/research/FlapsErp優化_合規風險追蹤表.md` v1.2。

## K. 未決
- IT 變更所有者/測試機；其他 HTTPS 端點盤點；批次主機（另機/同機）；8h soak；`FORCE_DEVICE_SACLE_FACTOR` 正確鍵行為；AP 轉 HTTPS（原廠）。

## L. 發布判定
**NOT VERIFIED**。唯讀證據（T-00/T-01/T-02/T-03/T-15 fixture）已完成；但 IT 變更、系統級 A/B、8h soak 未執行，不能標 PASS 或 CONDITIONAL PASS。解除條件：T-04/T-05/T-07 完成＋系統級 A/B 無退步＋8h soak 通過。

---

*本報告只引用 artifacts；未量測標 UNKNOWN；UNKNOWN ≠ PASS ≠ FAIL。*