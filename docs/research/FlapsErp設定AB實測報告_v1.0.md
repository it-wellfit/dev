# FlapsErp 設定 A/B 實測報告 v1.0

> 報告編號：RPT-2026-1111-001｜日期：2026-11-11｜狀態：**NOT VERIFIED**（僅完成唯讀證據；IT 變更/系統級 A/B/8h soak 未執行）
> 範圍：FlapsErp v0.0.4＋共用 Windows 宿主；對應計畫 `1794395575776`。
> 紅線：代理唯讀；不得寫入 `C:\SYSTEM_PROGRAM_DIRECTORY`；不得降低其他程式效能。

---

## A. 目標與範圍
- 在不改原廠原始碼前提下，建立 FlapsErp 產品/宿主/衛生/工作台的完整設定基線，並以 A/B 證明可量測改善。
- 非目標：原廠二進位修改、ERP 伺服器端、COM/CDP 正式化、Rust 平台線、64-bit 重打包。
- 軟體邊界：原廠 FlapsErp（`C:\SYSTEM_PROGRAM_DIRECTORY`，唯讀）／自建 Wellfit（`Dev\Wellfit`，可執行/產出）／Windows 宿主（IT）；三者不得互相比較或互相量測。
- 平台：FlapsErp 僅 Windows；Wellfit 批次可 Linux；Windows artifacts 不適用於 Linux。

## B. 現況證據（2026-11-11 唯讀）

### B.1 `config.ini`（原廠 FlapsErp；`-Probe settings`）
- 7 鍵：`IGNORE_CERT_ERR=1`、`KEEP_MACHINE_ID=1`、`DISABLE_HARDWARE_ACCELERATION=0`、`DEBUG_FASTREPORT=0`、`FORCE_DEVICE_SACLE_FACTOR=0`、`LANG=zh_TW`、`KEEP_MACHINE_ID_MIGRATED=1`。
- AP：3 個；預設 `8E2EDBE64B3F0747B90EC071E4548BE8`（大前集團 WELLFIT）HTTP `210.64.124.59:10578`；測試 AP HTTPS `test2.flaps.tw:10113`。
- Module：`POSMachineController.CD8240.jsc.ver=0.0.4`、`Winax.node.ver=0.0.4`。
- Manifest drift：**1/9**；`8E2EDBE64B3F0747B90EC071E4548BE8.version.txt` 目前 `7A4C6E…` vs snapshot `B82297…`（AP 版本更新）。
- `softwareId.txt`：目前＝manifest `D5E6B4350C1EBC0F049B0D69AD71668EFFFB716D525B42090C4C1D38567CA453`（基線報告原 `…4010EE…` 為繕寫錯誤，已更正）。

### B.2 App log（原廠 FlapsErp log；`-Probe app`）
- Log：`application-WELLFIT-20261111.log`；`erpRunning=False`（探測時未啟動）。
- `moduleToMenuMs=17087`（單次日誌；16:20:40.310→16:20:57.397）。
- 2026-11-12 log（`application-WELLFIT-20261112.log`）：`moduleToMenuMs=15674`、`downloadDoneRelMs=1427`、`detailsRelMs=1267`（單次日誌；不同日/負載，**不是 A/B**）。
- `downloadDoneRelMs=1465`；`detailsRelMs=1673`。
- `menuToFirstTabMs=UNKNOWN`（無 tab ready 標記）；`firstTabAfterMenuMs=300308`（事件差，非載入延遲）。
- `window=1536x776`、`screen=1536x864`（≈125%）。

### B.3 TLS（原廠 AP；`-Probe tls`）
- `test2.flaps.tw:10113`：**CERT-VALID**；CN=test2.flaps.tw；Let's Encrypt；TLS 1.2；到期 2027-01-21；hostname/鏈/撤銷 OK。
- 預設生產 AP 為 HTTP；`IGNORE_CERT_ERR` 對生產無作用。
- `app.asar` 掃描：無其他 `https://...flaps/wellfit...` literal 端點（`flapsWellfit=0`；artifact `Wellfit/artifacts/perf/20261111-195857/endpoints.json`，asar 11,832,082 bytes）；更新/授權端點若存在可能由 config/變數提供，需 runtime 監控。

### B.4 PE（原廠 FlapsErp.exe；`-Probe pe`）
- `FlapsErp.exe`：i386；**LAA=True**；`Characteristics=0x0122`；x64 下 32-bit 可用 4 GB（非 2 GB）。

### B.5 既有基線（FlapsErp 原廠）
- 2026-11-03：11 行程／RAM median 46.2 MB／max 103.9 MB／總 623.8 MB；COM P50/P95/P99 0.2636/0.3884/0.4740 ms；CDP NOT-FOUND。
- 2026-11-06：COM 0.1869/0.2634/0.3394 ms。
- 2026-11-11/12 非 CONTENDED 全量（`Wellfit/artifacts/perf/20261111-194436/`、`/194739/`、`/195713/`、`/195906/`、`20261112-100341/`）：COM P50/P95/P99 `0.1736–0.1906/0.2069–0.3171/0.2405–0.4467` ms（五次取樣；P95 約 53%、P99 約 86% 差距）；CDP NOT-FOUND；`195906` baseline APP-NOT-RUNNING；`100341` baseline 8 行程/IDLE（5.7/true 4.8/max 15.8/total 510；free 7.158 GB；CPU 12.5%）；`-Probe host` 併入 `system-baseline.json`。`100341` 的 `app` `3,805.923 s` 為多 session 誤配；`100506` 修正後 15.674 s。
- 2026-11-03 RAM median 46.2 MB 為舊版 upper-median；新版 tool 另輸出 `wsMB_median_true`（true median），舊 artifact 無此欄，比較時標 UNKNOWN。
- 2026-11-12 09:17 為 **CONTENDED**（30 個 `link`；free 3.1 GB；diskQueue 0；CPU 45%；COM P95 0.3988/P99 0.478；top 清單 30 link）；**不列入基線**、A/B 無效。
- 2026-11-12 09:19/09:25 為 **CONTENDED**（14–30 個 `link`；free 1–5 MB；diskQueue 15–17；COM P99 5.58–9.14 ms）；09:25（`092528`）由工具標 `contended=True`；09:19（`091941`）為同類事件；**不列入基線**、A/B 無效。
- 2026-11-12 10:23 Arm B（`=1`，登入後單次）：8 行程；median 73.7/true 72；max 106.3；total 5,872.5；COM 0.192/0.3841/0.466；host free 7.462 GB；CPU 23.2%；manifest drift 2/9（config.ini 已改）；`moduleToMenu=31.473 s`；**=0 新鮮登入後 UNKNOWN（app 在 login 頁；代理無法登入）**；A/B 未完成。

### B.6 宿主基線（Windows；非原廠/自建；`-Probe host`，2026-11-11/12）
- OS：Windows 11 Home build 26300（64-bit）。
- CPU：AMD Ryzen 7 4800HS（8C/16T）；load 7–73%（取樣；受其他程式/本工具影響）。
- RAM：15.79 GB total；4.0–7.2 GB free（非 CONTENDED 取樣；`100341` 7.158、`194543` 5.022、`194739` 4.646、`195713` 4.253、`195906` 4.005 GB）。
- 磁碟佇列 0–1（非 CONTENDED 取樣）；溫度 UNKNOWN。
- Top（代表值，會隨取樣變動）：Code 991/623/233/149 MB、Memory Compression 796、kilo 744、MsMpEng 308、ChatGPT 287/189/178、explorer 273、msedgewebview2 253/206、chrome 215/172/170/143、powershell 163、msedge 132、Outlook 129；2026-11-12 取樣另含 WhatsApp.Root 190、MSPCManager 165.3、POWERPNT 304.7。
- 意義：本機為行動平台（推測筆電）、記憶體只剩約 4.0–7.2 GB free（非 CONTENDED）；FlapsErp 11 行程與批次會再擠；不套用高效能電源/快速啟動/USB/NIC/GPU/視覺/搜尋變更；FlapsErp-only 為主。

### B.7 Wellfit MTRIX fixture（自建 Wellfit；非 FlapsErp 效能；T-15 驗證）
- 命令：`powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\erp-batch\run-mtrix-pipeline.ps1"`。
- 首跑：exit 0；2085 ms；5 檔、25 列、37 修復、22 errors/29 warnings（partial-accepted）。
- 冪等重跑：exit 1（duplicate batch 正確拒絕）。
- RUN_DIR：`Wellfit/artifacts/erp-batch/20261111-194835/`。
- 意義：外部工作台流程可重現；**此為自建 Wellfit 的 fixture，不是 FlapsErp 效能；不可作為 FlapsErp 產品效能證據；fixture 非生產 SLA**。

### B.8 證據索引（可追溯）
| 主張 | 所有者 | 命令 | Artifact |
|---|---|---|---|
| config 7 鍵 | 原廠 FlapsErp | `flaps-perf.ps1 -Probe settings` | `Wellfit/artifacts/perf/20261111-195906/perf-result.json` |
| AP 預設 HTTP／default `8E2EDBE` | 原廠 FlapsErp | `-Probe settings` | 同上 |
| 測試 AP CERT-VALID | 原廠 AP | `-Probe tls` | 同上 |
| app.asar 無其他 flaps/wellfit HTTPS | 原廠 FlapsErp | `-Probe endpoints` | `Wellfit/artifacts/perf/20261111-195857/endpoints.json`（`flapsWellfit=0`） |
| module→menu 17.087 s | 原廠 FlapsErp log | `-Probe app` | `20261111-195906/perf-summary.md` |
| LAA=True（0x0122） | 原廠 FlapsErp.exe | `-Probe pe` | 同上 |
| COM 取樣 | Windows COM（非 FlapsErp） | `-Probe com` | `20261111-194436/`、`194739/`、`195713/`、`195906/`、`20261112-100341/`（IDLE，非 CONTENDED；`091710`/`091941`/`092528` CONTENDED，不列入） |
| 宿主基線 | Windows 宿主 | `-Probe host` | `20261111-200314/system-baseline.json`（含 netStats；另全量 `195906`） |
| MTRIX fixture | 自建 Wellfit（非 FlapsErp） | `run-mtrix-pipeline.ps1` | `Wellfit/artifacts/erp-batch/20261111-194835/` |
| softwareId 更正／manifest drift 1/9 | 原廠 FlapsErp | `-Probe settings` | `20261111-195906/perf-result.json` |
| 程序/模組/協定 hygiene | 自建工具 | `-Probe hygiene` | `20261112-091230/hygiene.json` |
| A/B 差異 | 自建工具 | `-Probe diff -BaselineDir <before> -CurrentDir <after>` | `20261112-091548/diff.json` |
| 2026-11-12 09:17 全量（CONTENDED） | 自建工具 | `-Probe full` | `20261112-091710/perf-summary.md` |

### B.9 v4.0 校正紀錄（2026-11-12）
| 項目 | v4.0 內容 | 校正／處理 |
|---|---|---|
| NTFS `disablelastaccess` | 稱有效值只有 0/1，2 無效 | **錯誤**：實際 0=user-managed enabled、1=user-managed disabled、2=system-managed enabled（預設）、3=system-managed disabled；runbook 回滾 `2` 正確，保留；已加註 0–3 對照 |
| config 編輯 | `ReadAllLines`+`WriteAllText` 會統一 CRLF/補尾換行 | **正確**：已納入 runbook/手冊；先檢查位元組/換行/BOM/最後換行，只替換指定鍵 |
| `ExecutionPolicy Bypass` | 不是安全邊界 | **正確**：已納入 runbook |
| Defender 排除 | 提高風險等級、最小範圍 | **正確**：已納入 runbook/settings baseline（核准/期限/複查） |
| 時間同步 | 補 status/source/peers | **正確**：已納入 runbook |
| 防火牆 | 補 TCP/Profile/rollback/限制 | **正確**：已納入 runbook |
| 溫度 UNKNOWN | 不可視為通過 | **正確**：已納入 runbook/plan |
| MTRIX fixture | 標為自建 Wellfit、非 FlapsErp | **正確**：本報告 §B.7/§B.8/§F 已標註 |
| 其他 v4.0 數字 | 15.79 GB/3.1–5.0 GB/7–73%/17.087 s/COM 範圍/LAA/endpoint | 與實測一致（非 CONTENDED 取樣） |

> NTFS 值來源：Microsoft Learn `fsutil behavior` 語法列 `{1|0}`（不完整）；Windows 實際 `fsutil behavior set disablelastaccess` help 列 0–3（0/1=user-managed、2/3=system-managed）；本機預設 2。v4.0 的「0/1 only、2 無效」不成立。

### B.10 工具強化（2026-11-12）
| 強化 | 說明 |
|---|---|
| `hygiene` probe | FlapsErp 程序/角色/processed.ini/Module `.ver`/flapserp/windmill/Defender/ExecutionPolicy/AppLocker；2026-11-12 實測 8 行程、flapserp 已註冊、windmill 未安裝 |
| `diff` probe | 兩 artifacts 比較；config/app/baseline/com/host；環境指標標 `INFO`（非 FlapsErp 改善）；缺 settings 不誤報 |
| `wsMB_median_true` | 真正的中位數（偶數取兩中間平均）；保留舊 upper-median 向後相容 |
| `wsMB_total` | 行程總 RAM；2026-11-12 09:17 為 **490.5 MB**（8 行程：68.8+55.8+78.3+67.2+38.7+84+57.5+40.2；**09:17 CONTENDED；僅供參，不作基線**；`100341` IDLE 為 510 MB；舊 artifact 無此欄，由 rows 加總） |
| `app` probe session-aware | 以最後一次 `module load` 後的第一個 `menu` 計算；修正 2026-11-12 `3,805.923 s` 多 session 誤配；`100506` 修正後 15.674 s |
| `contended` | `link`/`cargo`/`rustc`、free<1 GB、diskQueue>5 → 樣本無效；09:17/09:19/09:25 為同類事件（30/14/30 link；free 3.1/1/5 MB；disk 0/15/17）；09:25（`092528`）由工具標 `contended=True`；09:17/09:19 早於旗標，事件相同 |

### B.11 A/B 執行狀態（2026-11-12）
| 項目 | 狀態 | 證據 |
|---|---|---|
| Arm A（`DISABLE_HARDWARE_ACCELERATION=0`，目前） | **準備取樣**（混合 active/idle/not-running；非 CONTENDED） | `194436`/`194739`/`195713`/`195906`；`100341` IDLE |
| Arm B（`=1`） | **部分實測**：app 可啟動；登入後 startup 31.473 s（單次）；RAM/COM 已測；**=0 新鮮登入後 UNKNOWN（login 頁）** | `102314`；`102609`/`102725` |
| 系統級 A/B | **未執行（IT）** | UNKNOWN |
| 8h soak | **未執行（IT）** | UNKNOWN |
| App smoke | **未執行（IT/使用者）** | UNKNOWN |
| 其他 HTTPS 端點盤點 | **部分**：app.asar 無 flaps/wellfit literal；runtime 待監控 | `endpoints.json` |

### B.12 完成 A/B 的步驟（IT）
1. Pre-flight：`link`/`cargo`/`rustc` 為 0；app 執行中；`-Probe full` 非 CONTENDED。
2. 備份 config＋SHA256（runbook §2.2）。
3. Arm A 5 次（`baseline`＋`app`）；記 `OUT_DIR` 為 `<A>`。
4. IT 改 `DISABLE_HARDWARE_ACCELERATION=1`；重啟；5 次；記 `OUT_DIR` 為 `<B>`。
5. `-Probe diff -BaselineDir <A> -CurrentDir <B>`。
6. 判定（§G）：FlapsErp 改善 ≥10%、其他程式/系統 ≤10%、無熱降頻 → 採 B；否則 A。
7. 還原/記錄；更新 §F/§G/§L。

## C. 決策
- 代理可套用 `config.ini`（使用者 2026-11-12 明示授權；備份/回滾）；OS/二進位仍由 IT。
- 原廠 FlapsErp 產品設定只動 7 鍵；只 A/B `DISABLE_HARDWARE_ACCELERATION`；`FORCE_DEVICE_SACLE_FACTOR` 僅 UI 裁切時評估。
- `IGNORE_CERT_ERR` 維持 1，直到 HTTPS 端點盤點完成。
- 外部工作台（自建 Wellfit；A1/報表外移）為主要效益；批次必須另機/離峰/低優先/可中止。
- 系統級變更需 IT 核准＋系統級 A/B；不得以 FlapsErp 效能為由拖垮其他程式。

## D. 變更內容（尚未套用）
- 原廠 FlapsErp：7 鍵（見 B.1）；`DISABLE_HARDWARE_ACCELERATION` A/B；`IGNORE_CERT_ERR` 條件式。
- Windows 宿主：電源/睡眠/快速啟動/USB/分頁檔/NTFS/AV/WDAC/UAC/PowerShell/時間/NIC/GPU/搜尋/視覺——均標 S/C，見基線文件 §3。
- AP/網路/TLS：原廠 AP 預設不變；防火牆白名單；測試 AP 生產不可達；HTTPS/CA 條件式。
- 自建 Wellfit／衛生：單一實例、關閉多餘分頁、夜間重啟（條件式）、A1/Wellfit/報表外移。

## E. 量測方法
- 工具：`Wellfit/scripts/perf/flaps-perf.ps1`（`-Probe settings/app/tls/pe/host/endpoints/hygiene/diff/baseline/com/cdp/all/full`）。
- 每臂 5 次；冷/暖啟動分開；同一台機器/使用者/AP/時段；不跑 `cargo build`。
- 系統級 A/B：FlapsErp＋瀏覽器 10 tabs＋Excel 10 MB＋Outlook＋1 GB 檔案複製；量測每個程式回應/系統 CPU/RAM/磁碟佇列/GPU/溫度。

## F. 實測結果（目前）
| 項目 | 結果 | 狀態 |
|---|---|---|
| config 7 鍵 | 與實機一致 | ✔ |
| AP/TLS | 預設 HTTP；測試 AP CERT-VALID | ✔ |
| PE LAA | True（0x0122） | ✔ |
| module→menu | 17.087 s（舊 session）；`=1` 新鮮登入後 31.473 s（單次） | ✔/部分 |
| RAM/COM | 既有 2026-11-03/06 基線；2026-11-11 COM 取樣；`=1` 102314：median 73.7/true 72/total 5,872.5（active；不可與 IDLE 比較） | ✔（歷史/取樣） |
| config.ini 已改/已還原 | `=1` 已套用→已還原 `=0`；備份 `20261112-1013` | ✔ |
| 硬體加速 A/B | `=1` 登入後 startup 31.473 s（單次）；RAM 8/median 73.7/true 72；COM 0.192/0.3841/0.466；`=0` 新鮮登入後 UNKNOWN | 部分/未驗證 |
| DPI A/B | 未執行 | UNKNOWN |
| OS 變更前後 | 未執行 | UNKNOWN |
| 系統級 A/B | 未執行 | UNKNOWN |
| 自建 Wellfit MTRIX fixture | exit 0；冪等重跑 exit 1 | ✔（fixture；非 FlapsErp） |
| 自建 Wellfit hygiene | 8 行程；roles；processed；flapserp 已註冊；windmill 未安裝 | ✔（唯讀） |
| 2026-11-12 09:17 full（CONTENDED） | 30 link；free 3.1 GB；disk 0；COM P95 0.3988/P99 0.478 | 無效（cargo build 併發） |
| 2026-11-12 09:25 full（CONTENDED） | 30 個 link；free 5 MB；diskQueue 17；COM P99 5.58 ms | 無效（cargo build 併發） |
| 自建 Wellfit A/B diff | 工具可輸出差異；環境指標標 INFO | ✔（工具） |
| 自建 Wellfit 生產批次 | 未執行 | UNKNOWN |
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
- LAA/COM 更正：`docs/research/FlapsErp產品線效能進度報告_v2.0.md`、`docs/research/輔翼ERP逆向分析與最佳化方案_v1.0.md`、`docs/research/proposals/OQ-F1_F2_輔翼ERP正式需求提案_v2.1.md`、`docs/architecture/效能架構.md`。

## K. 未決
- IT 變更所有者/測試機；其他 HTTPS 端點盤點；批次主機（另機/同機）；8h soak；`FORCE_DEVICE_SACLE_FACTOR` 正確鍵行為；AP 轉 HTTPS（原廠）。

## L. 發布判定
**NOT VERIFIED**。唯讀證據（T-00/T-01/T-02/T-03/T-15 fixture）已完成；`DISABLE_HARDWARE_ACCELERATION=1` 已套用並還原（備份 `20261112-1013`）；但 `=1` 登入後 startup 31.473 s（單次）、`=0` 新鮮登入後 UNKNOWN（app 在 login 頁；代理無法登入），**A/B 未完成**，不能標 PASS 或 CONDITIONAL PASS。解除條件：使用者登入後完成 fresh `=0`/`=1` 各 5 次＋系統級 A/B 無退步＋8h soak 通過。

---

*本報告只引用 artifacts；未量測標 UNKNOWN；UNKNOWN ≠ PASS ≠ FAIL。*