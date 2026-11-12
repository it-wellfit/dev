# FlapsErp 產品優化設定與系統最佳設定計畫（成效導向版）

> 計畫編號：1794395575776｜日期：2026-11-11｜狀態：**Active（待 IT 變更窗）**
> 範圍：FlapsErp v0.0.4 桌面產品（`C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp`）＋ Windows 宿主＋與 FlapsErp 直接相接的 Wellfit 批次排程。
> 事實標記：✔ 2026-11-11 唯讀觀測｜△ 依證據推導，待實機驗證｜⚠ 需原廠／IT 確認。
> 治理：C-13 Evidence-Driven；發布判定五級（PASS／CONDITIONAL PASS／BLOCKED／REJECTED／NOT VERIFIED）。
> **執行邊界（使用者 2026-11-12 更新）**：代理可套用 `config.ini`（備份＋SHA256＋回滾；使用者明示授權）；OS／二進位仍由 IT；不得關閉 AV／UAC／WDAC；不得拖垮其他程式。

---

## 0. 執行摘要（先看這裡）

### 0.1 核心結論
**不能改原始碼，能改的只有五層**：
1. **產品設定**（`.conf\config.ini` 7 鍵；只有 `DISABLE_HARDWARE_ACCELERATION` 與 `FORCE_DEVICE_SACLE_FACTOR` 與效能直接相關）。
2. **Windows 宿主**（電源、AV 排除、SSD／RAM／分頁檔、網路省電、時間、鎖定）。
3. **啟動／分頁／程序衛生**（單一實例、關閉多餘分頁、夜間重啟、DLL 一次性註冊）。
4. **外部工作台（自建）**（A1 檔案通道、`flapserp://` 深層連結、Wellfit 批次外移、報表外移）。
5. **監控與回滾**（基線、A/B、漂移告警、備份）。

**真正的效益主要在第 3–4 層，不是第 1 層。** 只調 7 鍵無法解決 `sendSync`、32-bit LAA 4 GB（實測）、`backgroundThrottling:false`、`console.log`、舊 OpenSSL 這些原始碼層瓶頸；這些只能「降低觸發頻率與影響」，不能消除。

> **Scope 提醒**：這台 Windows 是共用環境；優化 FlapsErp 不得以降低其他程式效能為代價。先做 FlapsErp-only；系統級變更見 §0.5／§4.3／§6.9。

### 0.2 可達成的成效（全部待實測，不得先宣稱 PASS）
| 槓桿 | 基線 | 目標（假設） | 證據等級 | 任務 |
|---|---|---|---|---|
| AV 排除＋分頁/程序衛生（本機為行動平台、推測筆電；電源維持平衡） | module load→menu ready ≈17.1 s | 改善 ≥10%（≤15.4 s）；-20% 理想值 | △ | T-04/T-06/T-13 |
| GPU 硬體加速 A/B | 11 行程 RAM median 46.2 MB／總 623.8 MB | median ≤41.6 MB／總 ≤561 MB（-10%） | △ | T-05 |
| 分頁／程序衛生 | 11 行程；多餘 readonly tabs | 行程數↓、游離 RAM↓、UI 凍結 -50% | △ | T-13 |
| 夜間重啟 | 8h RAM 增長未量測 | 8h 增長 <50 MB、無崩潰 | △ | T-14 |
| A1 檔案通道＋Wellfit 外移（自建） | 逐筆人工作業 | 批次工時 -80%（研究 KPI） | △ | T-15 |
| 報表外移（不客端渲染） | 大報表佔 client CPU／RAM | 報表處理 ≤60 s／10k 列；client RAM 不爆 | △ | T-15 |

> 以上全部為**待實測假設**；只有 T-11 報告＋T-12 soak 通過才可標 PASS，否則 CONDITIONAL PASS／REJECTED。
> 採用門檻：改善 ≥10% 才採用；-20% 為理想值，非承諾；未達門檻記錄但不改。

### 0.3 不能達成的事（誠實）
- `sendSync` 同步 IPC：原始碼層，設定無法消除；只能**不在渲染進程做高頻 COM**（改外部批次／檔案通道）。
- 32-bit LAA 4 GB 位址空間：設定無法放寬；實測 `LAA=True`（x64 下可用 4 GB，不是 2 GB）；仍有限，只能**降低記憶體壓力**（關閉分頁、報表外移、夜間重啟）＋ A 線要求 64-bit。
- `backgroundThrottling:false`：原廠寫死；只能**減少背景分頁**。
- 每次 IPC `console.log(args)`：原廠寫死；只能**降低 IPC 頻率**。
- 舊 OpenSSL／Vue 2 EOL：原廠問題；只能**網路隔離＋要求原廠升級**。
- `IGNORE_CERT_ERR`：不是效能設定；且預設 AP 是 HTTP，改 0 對 HTTP 無作用。

### 0.4 五個優先動作
1. **T-00/T-00b 宿主＋全系統盤點、T-01 凍結基線＋備份**（沒有基線不得改）。
2. **T-04/T-06 Tier 0 宿主與衛生（含系統級 A/B）**。
3. **T-05 GPU／DPI A/B**（唯一兩個產品層效能鍵）。
4. **T-15 A1／自建 Wellfit／報表外移**（最大效益，屬外部工作台）。
5. **T-08/T-14 監控、備份、夜間重啟、回滾**。

### 0.5 系統級不退化原則（這台 Windows 不是 FlapsErp 專用）
1. **FlapsErp-only 變更優先**：分頁／程序衛生、單一實例、夜間重啟、config 7 鍵——不影響其他程式。
2. **系統級變更需另證**：電源、睡眠、快速啟動、USB、NIC、GPU 驅動、分頁檔、搜尋索引、視覺效果——必須 IT 核准，且以「系統級 A/B」證明其他程式不退化（不只是 FlapsErp 變快）。
3. **不得以降低其他程式效能為代價**：若某變更使其他程式 CPU／RAM／DPI／電池／安全性變差，即使 FlapsErp 變快也不採用。
4. **批次外移不是免費的**：Wellfit／MTRIX／報表處理若與 FlapsErp 同機，會搶 CPU／RAM／I/O；必須另機、離峰、低優先、可中止，否則不算優化。
5. **系統級驗收**：變更後量測其他代表性程式（瀏覽器／Office／檔案總管）與系統 CPU／RAM／磁碟；任一退步 >10% 即回滾。

| 變更 | 影響範圍 | 對其他程式的風險 | 決策 |
|---|---|---|---|
| 分頁／程序衛生、單一實例、夜間重啟 | FlapsErp-only | 無 | 採用 |
| config 7 鍵 | FlapsErp-only | 無（`IGNORE_CERT_ERR` 影響連線，不影響其他程式） | 採用（A/B） |
| AV 排除 | 系統（安全） | 安全面下降（不影響效能） | IT 核准；最小排除 |
| 電源計畫／睡眠／USB／NIC | 系統 | 電力／裝置行為改變；不應降低其他程式效能 | 桌機/RPA 站；A/B |
| 快速啟動 | 系統 | 開機變慢（所有使用者） | 僅在 DLL/驅動狀態問題時 |
| 分頁檔固定 | 系統 | 其他程式虛擬記憶體受限 | 預設 system-managed；實測才固定 |
| GPU 驅動 | 系統 | 其他 GPU 程式可能回歸 | 僅在現有驅動有問題時 |
| 搜尋索引／視覺效果 | 系統 | 其他程式搜尋／外觀改變 | 非必要不做 |
| Wellfit 批次 | 同機/另機 | 搶 CPU／RAM／I/O | 另機或離峰＋低優先＋可中止 |

### 0.6 軟體邊界（原廠 vs 自建；避免設錯/測錯）
| 元件 | 所有者 | 路徑 | 可否修改 | 本計畫用途 |
|---|---|---|---|---|
| FlapsErp 外殼/執行檔 | 原廠 | `C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp\FlapsErp.exe`、`resources\app.asar` | **不可**（紅線） | 唯讀 pe/settings/app/endpoints |
| FlapsErp 原廠設定 | 原廠 | `C:\SYSTEM_PROGRAM_DIRECTORY\.conf\config.ini` | 僅 IT 變更管制 | settings probe；7 鍵 |
| FlapsErp 原廠元件 | 原廠 | `Module\*.jsc/.node`、`*.dll`、`.conf\*.version.txt` | **不可** | 漂移監控 |
| FlapsErp 原廠資料 | 原廠 | `FlapsErpData\AP\*.json`、`logs/`、`ReportTemp/`、`Pos2Log/` | **不可寫**；唯讀/IT 清理 | settings/app probes |
| Wellfit 自建軟體 | 自建 | `Wellfit\crates`、`%LOCALAPPDATA%\Temp\wellfit-target\debug\wellfit.exe`、`Wellfit\scripts`、`Wellfit\spec`、`Wellfit\data\wellfit_ledger.sqlite` | 可（依 Wellfit AGENTS） | MTRIX fixture、daily-backup、bench |
| Wellfit 量測工具 | 自建 | `Wellfit\scripts\perf\flaps-perf.ps1`、`Wellfit\artifacts\perf` | 可 | 唯讀探測/證據 |
| Rust 平台（非本計畫） | 自建 | `platform\`、`core\`、`web\`、`crates\` | 可（另計畫） | 不在本計畫 |
| Windows 宿主 | 微軟/IT | OS 設定 | IT | host probe |

**規則**：不得把 Wellfit/自建工具的效能或設定當成 FlapsErp 原廠效能；不得把 FlapsErp 原廠設定寫到 Wellfit；不得把自建平台的 DB 當 FlapsErp 資料；`FlapsErpData` 僅唯讀。

### 0.7 平台邊界（Windows / Linux / macOS）
| 元件 | 平台 | 說明 |
|---|---|---|
| FlapsErp | **Windows only** | COM/ActiveX/DLL；無 macOS/Linux 版本；本計畫的 Windows 設定僅適用 Windows |
| Wellfit 批次 | Windows / Linux | Rust 可跨平台；**優先 Linux 批次主機**（native binary；無 Docker）；若同 Windows 主機，套用 §4.5.2 |
| `flaps-perf.ps1` | Windows only | PowerShell；Linux 不適用；Wellfit Linux 用 `wellfit` CLI＋`/usr/bin/time` 或 Rust harness |
| Rust 平台 | 跨平台 | 非本計畫 |
| 量測/證據 | 平台特定 | Windows 用 `.ps1`/artifacts；Linux 用 shell/Rust harness；**不得混用** |

---

## 1. 目標與非目標

### 1.1 目標
1. 產出 **可執行的完整設定基線**：產品 7 鍵、AP／網路、Windows 宿主、啟動／分頁／程序、外部工作台。
2. 每項變更都有 **基線、A/B、驗收、回滾**；未量測不得宣稱改善。
3. 把「原始碼層瓶頸」轉為 **外部可達的降低影響方案**（檔案通道、批次外移、報表外移、分頁衛生）。
4. 修正既有文件與實機不符的路徑／項目。

### 1.2 非目標
- 修改 `app.asar`／`FlapsErp.exe`／任何二進位（閉源＋bytenode；紅線）。
- ERP 伺服器端（`RDAPortal.jsp`／AP 主機）設定變更。
- COM／CDP／RDA 自動化正式化（Phase 3–4，另計畫）。
- Rust 平台線（`platform/`／`core/`／`ekpi-platform`）內部 DB／API 設定。
- 64-bit 重新打包（A 線 OQ-F1/F2）。
- 投機的 Chromium 旗標（無證據不採用）。

---

## 2. 已驗證現況（2026-11-11 唯讀）

### 2.1 安裝與產品設定（原廠 FlapsErp）
| 項目 | 實測值 | 證據 |
|---|---|---|
| 安裝目錄 | `C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp`（20 項） | 目錄讀取 ✔ |
| 外殼 | Electron（Chromium＋Node.js），ia32 32-bit，exe 134.4 MB（134,413,000 bytes） | 既有逆向 ✔（2026-11-03）＋`(Get-Item).Length` ✔（2026-11-11） |
| `config.ini` 路徑 | `C:\SYSTEM_PROGRAM_DIRECTORY\.conf\config.ini` | 目錄讀取 ✔ |
| `config.ini` 內容 | 7 鍵：`IGNORE_CERT_ERR=1`、`KEEP_MACHINE_ID=1`、`DISABLE_HARDWARE_ACCELERATION=0`、`DEBUG_FASTREPORT=0`、`FORCE_DEVICE_SACLE_FACTOR=0`、`LANG=zh_TW`、`KEEP_MACHINE_ID_MIGRATED=1` | 檔案讀取 ✔ |
| `Module` | `Winax.node`（ver 0.0.4）、`POSMachineController.CD8240.jsc`（ver 0.0.4） | 目錄＋`.ver` ✔ |
| 執行期元件 | `midas.dll`、`MReportViewer.dll`、`POSMachineController.dll`、`libeay32.dll`、`ssleay32.dll` 等 | 目錄 ✔ |
| PE 位址空間 | `FlapsErp.exe` i386、**LAA=True**（`Characteristics=0x0122`） | `flaps-perf.ps1 -Probe pe` ✔（2026-11-11） |

### 2.2 AP 伺服器（原廠；`FlapsErpData\AP`）
| AP ID | 名稱 | URL | 預設 | 用途 |
|---|---|---|---|---|
| `8E2EDBE64B3F0747B90EC071E4548BE8` | 大前集團(WELLFIT) | `http://210.64.124.59:10578/WELLFIT/` | **是** | 生產 |
| `54B36E184BE06A838721E3D567A1203F` | 大前集團(WELLFIT-1) | `http://210.64.124.59:10578/WELLFIT-1/` | 否 | 備援 |
| `241C394AA7CE983F4DB1B162B5113D95` | *FL測試(WELLFIT)(0.1:1433) | `https://test2.flaps.tw:10113/WELLFIT/` | 否 | 測試 |

**關鍵事實**：預設生產 AP 走 **HTTP 明文**。`IGNORE_CERT_ERR` 對預設 AP 無作用；TLS 風險只在測試 AP／未來 HTTPS。真正修正＝AP 轉 HTTPS＋正式憑證。✔/△
**測試 AP TLS 實測（2026-11-11）**：`CERT-VALID`；CN=test2.flaps.tw；Let's Encrypt；TLS 1.2；到期 2027-01-21；hostname/鏈/撤銷 OK。因此 `IGNORE_CERT_ERR=0` 對測試 AP 不會破壞連線；但其他 HTTPS 端點（更新/授權）仍須 T-03 盤點。✔

### 2.3 今日執行日誌基線（原廠 FlapsErp log；`logs\application-WELLFIT-20261111.log`，單一日志樣本）
| 事件 | 時間 | 推得基線 |
|---|---|---|
| AP 登入頁載入 | 16:20:40.310 | — |
| 選單就緒（`menu.prepareColorMask`） | 16:20:57.397 | **module load→menu ready ≈17.1 s** |
| 元件下載完成（`menu.finishDownloadRequires`） | 日誌標記 `+1465 ms` | 元件檢查約 1.5 s |
| 選單細節（`menu.prepareDetails`） | 日誌標記 `+1673 ms` | 細節渲染約 1.7 s |
| 視窗／螢幕 | 1536×776／1536×864 | ≈125% 縮放 ✔ |
| 報表工作 | 17:43:48.306 `non delay ReportAddTaskSuccess` | 僅事件時間；延遲 UNKNOWN |

> 單次日誌僅供基線；T-01/T-02 需 ≥5 次重複取中位數。

### 2.4 既有更高效能基線（FlapsErp 原廠 vs Wellfit 自建；不可互比）
| 指標 | 2026-11-03 | 2026-11-06 | 來源 |
|---|---|---|---|
| 行程數／RAM median／max | 11／46.2 MB／103.9 MB | — | `20261103-160315` ✔ |
| 總 RAM | 623.8 MB | — | 同上 ✔ |
| COM P50／P95／P99 | 0.2636／0.3884／0.4740 ms | 0.1869／0.2634／0.3394 ms | 同上＋`20261106-115317` ✔ |
| CDP 9222 | NOT-FOUND | — | 同上 ✔ |
| config 快照 | 10 檔＋SHA256 manifest | — | `SHA256_MANIFEST.txt` ✔ |

> 2026-11-12 09:17 全量（app 執行中）：8 行程；RAM median 67.2（true 62.4）；max 84；host free 3.1 GB；CPU 45%；COM 0.2332/0.3988/0.478 ms；hygiene 8 行程／flapserp 已註冊／windmill 未安裝。

### 2.5 已發現的文件事實錯誤（必須更正）
1. `docs/04-operations/FlapsErp效能優化操作手冊_v1.0.md` §3.2 指向 `Dev\platform\config.ini`（**錯誤路徑**；實機在 `C:\SYSTEM_PROGRAM_DIRECTORY\.conf\config.ini`）。
2. 同節「可調整項目」寫 `ORG/PERM/ACT/STORE/POS/INV`——**實機無這些鍵**，屬虛構；實際只有 §2.1 的 7 鍵。
3. 同節 §3.2.3 要求「修改後跑 `cargo test --locked --workspace`」——**config 變更與 Rust 測試無關**；正確驗證是 `flaps-perf.ps1`＋app smoke。
4. `FlapsErp效能基線報告_v1.0.md` §7.1 的 `softwareId.txt` SHA256 與 manifest 後段不同——**已重算：目前＝manifest `D5E6…42090C4C1D38567CA453`；報告的 `…4010EE21977F8990F6D8` 為繕寫錯誤**。
5. `docs/04-operations/A1檔案通道端到端操作手冊_v1.0.md` Step 2/4 把 `ledger.sqlite`、`report.json`、`import.csv` 寫入 `C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErpData\`——**違反 RSK-E5 紅線**；應改寫 `%TEMP%` 或 `Wellfit\artifacts\erp-batch\<ts>\`。
6. 同手冊 §1.2 把 Windmill 安裝到 `C:\SYSTEM_PROGRAM_DIRECTORY\windmill\`——**不可寫入原廠目錄**；應裝到 `C:\ProgramData\Windmill` 或 `%LOCALAPPDATA%\Windmill`（IT 核准）。
7. 同手冊 Step 2 指向 `C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp\...\run-mtrix-pipeline.ps1`——**路徑錯誤**；實際在 `Wellfit\scripts\erp-batch\run-mtrix-pipeline.ps1`（只寫 `OutRoot`，預設 artifacts）。
8. config 快照漂移：`8E2EDBE64B3F0747B90EC071E4548BE8.version.txt` 目前 hash（`7A4C6E…`）與 2026-11-03 manifest（`B82297…`）不同——AP 版本已更新；屬正常更新但需記錄（T-01/T-16）。

### 2.6 設定面清單（原廠 vs 自建；可改 vs 不可改）
| 設定面 | 可改性 | 變更者 | 備註 |
|---|---|---|---|
| `.conf\config.ini` | 可改（7 鍵） | IT | 只有 2 鍵與效能直接相關 |
| `FlapsErpData\AP\*.json` | 可改（僅透過 UI） | IT／使用者 | 預設 AP；避免誤連 |
| `Module\*.ver`、`.conf\*.version.txt` | **不可改** | 原廠 | 監控漂移 |
| `.conf\processed.ini` | **不可改** | 原廠 | 安裝／註冊狀態 |
| `config\EXT024.txt` | **不可改** | 原廠 | 授權／擴充 |
| `resources\app.asar` | **不可改** | 原廠 | 紅線 |
| `logs\`、`ReportTemp\`、`Pos2Log\` | **不可改**（IT 清理） | 原廠／IT | 監控／retention |
| Windows 宿主 | 可改 | IT | 電源／AV／SSD／RAM／網路 |
| 啟動／分頁／程序 | 可改（作業方式） | 使用者／IT | 單一實例、關閉分頁、夜間重啟 |
| 外部工作台（自建） | 可改（我方） | 開發／IT | A1／Wellfit／報表外移 |

### 2.7 全系統交互作用分析（共用 Windows 環境；原廠+自建+其他程式）
> 使用者同時跑 FlapsErp、瀏覽器、Office、檔案總管、可能還有視訊/備份/防毒；下列互動必須納入。
| 資源 | FlapsErp | 其他程式 | 互動風險 | 對變更的意義 |
|---|---|---|---|---|
| CPU | 11 行程；Chromium 渲染 | 瀏覽器/Office/防毒/索引 | 任何 CPU 吃重會拖慢全部 | GPU 關閉可能把渲染推回 CPU；需系統級 CPU A/B |
| RAM | 623.8 MB（11 行程） | 瀏覽器/Office 動輒 GB | 記憶體壓力→pagefile→全機變慢 | 先降 FlapsErp 自己用量；分頁檔勿固定 |
| 磁碟 I/O | DLL/報表/log | Office/瀏覽器/防毒/備份/Update | 批次 I/O 會拖慢其他程式 | 批次離峰/低優先；AV 排除；監控佇列 |
| 網路 | AP HTTP | 瀏覽器/Office/Update/備份 | 批次上傳會搶頻寬 | 離峰；限制批次 |
| GPU | Chromium 合成 | 瀏覽器/Office/視訊 | 驅動更新/硬體加速 A/B 影響全機 | GPU 驅動條件式；A/B 看系統 GPU/CPU |
| 電源/熱 | 11 行程 | 全部 | 高效能計畫→熱→全部降頻 | 桌機/RPA 站；監控溫度 |
| 背景服務 | — | Search/Update/Backup/Defender | 與 FlapsErp 搶 I/O/CPU | 排程避開；監控 |
| 安全 | COM/DLL | 全部 | 關閉 AV/WDAC 會降低全機安全 | 不關閉；只最小排除 |

**結論：FlapsErp 的設定必須以「自身用量下降、其他程式不退化」為雙目標；只讓 FlapsErp 變快但拖慢 Office／瀏覽器，不算優化。**

### 2.8 實測宿主基線（2026-11-11；`-Probe host`）
| 項目 | 實測 | 意義 |
|---|---|---|
| OS | Windows 11 Home build 26300（64-bit） | 家用版；AppLocker/WDAC 可能不可用 |
| CPU | AMD Ryzen 7 4800HS（8C/16T）；load 7–73%（取樣；受其他程式/本工具影響） | 筆電；高效能計畫會犧牲電池/熱 |
| RAM | 15.79 GB total；4.0–7.2 GB free（非 CONTENDED 取樣；會隨其他程式變動） | 16 GB 已足；但同時跑 Code/ChatGPT/Chrome/Outlook 只剩約 4.0–7.2 GB |
| 磁碟佇列 | 0–1（取樣） | 目前無明顯 I/O 瓶頸 |
| 溫度 | UNKNOWN（無 ACPI 熱區/權限） | 筆電熱降頻風險需監控 |
| 前 20 行程（代表值） | Code 991/623/233/149 MB；Memory Compression 796；kilo 744；MsMpEng 308；ChatGPT 287/189/178；explorer 273；msedgewebview2 253/206；chrome 215/172/170/143；powershell 163；msedge 132；OUTLOOK 129 | 其他程式已佔大量 RAM；FlapsErp 11 行程會再擠 |

**結論：本機為行動平台（推測筆電），不是桌機/RPA 站。** 預設維持 **Balanced 電源、系統管理分頁檔、保留快速啟動/USB/NIC/GPU/視覺/搜尋**；只做 FlapsErp-only（分頁/程序衛生、config A/B）與 IT 核准的 AV 排除/AP 白名單。系統級 A/B 必須包含 Code/ChatGPT/Chrome/Outlook 的反應；FlapsErp 的 11 行程與批次不得在別人忙時搶 RAM/CPU。

---

## 3. 決策（已裁定，含理由）

| ID | 決策 | 理由 |
|---|---|---|
| D-01 | 代理可套用 `config.ini`（使用者 2026-11-12 明示授權）；OS/二進位仍由 IT | 備份／SHA256／回滾；不得關閉 AV/UAC/WDAC；不得拖垮其他程式 |
| D-02 | 產品設定以 **7 個既有鍵**為準；不新增未證實的鍵 | 原廠只認既有鍵；新增鍵無效且製造假設定 |
| D-03 | 7 鍵中只 A/B `DISABLE_HARDWARE_ACCELERATION`；`FORCE_DEVICE_SACLE_FACTOR` 僅在 UI 裁切時評估 | 只有這兩個與效能／渲染直接相關 |
| D-04 | `IGNORE_CERT_ERR` 維持 1，直到「HTTPS＋憑證可信或內部 CA 已安裝」；HTTP AP 下無作用 | 直接改 0 可能破壞測試 AP；真正修正需 HTTPS |
| D-05 | 生產 AP 固定 `8E2EDBE…`；測試 AP 僅測試網段；備援不設預設 | 避免誤連 |
| D-06 | **外部工作台（自建 Wellfit）是主要效益來源**：A1 檔案通道、Wellfit 批次外移、報表外移、`flapserp://` 深層連結 | 直接繞開 `sendSync`／客端報表渲染 |
| D-07 | 分頁／程序衛生：單一實例、關閉多餘 readonly tabs、夜間重啟 | 對抗 `backgroundThrottling:false` 與 32-bit 記憶體壓力 |
| D-08 | 宿主設定只做「可量測、可回滾、不降低安全」的變更 | AV／UAC／WDAC 不可為效能關閉 |
| D-09 | 量測工具沿用既有 `flaps-perf.ps1` 擴充；不新增 Python | C-11；既有 `.ps1` 為例外 |
| D-10 | 交付採「新設定基線＋更正手冊＋A/B 報告＋PLAN_REGISTER」 | 單一事實來源 |
| D-11 | 不採用無證據的 Chromium 旗標（`--single-process`、`--no-sandbox`、`--disable-site-isolation` 等） | 安全／穩定性風險大於效益 |
| D-12 | 夜間重啟是「記憶體壓力」緩解，不是修復；若 8h soak 無洩漏則不啟用 | 避免不必要的中斷 |
| D-13 | 報表一律「匯出成檔→Wellfit 解析」，不在 client 大量渲染 | 直接降低 B-6 與 B-1 壓力 |
| D-14 | `IGNORE_CERT_ERR=0` 前必須先盤點所有 HTTPS 端點（更新／授權／AP） | 避免只修 AP 卻弄壞更新 |
| D-15 | 8h soak 為生產放行必要條件；測試機可先不跑 | 防止「設定看起來有效」但未驗證穩定性 |

---

## 4. 目標設定基線（分層）

### 4.0 槓桿優先序（依預期效益／風險）
| 順位 | 槓桿 | 層 | 預期效益 | 風險 | 任務 |
|---|---|---|---|---|---|
| 1 | A1 檔案通道＋Wellfit 外移＋報表外移 | 外部工作台 | 高（繞開 sendSync／客端報表） | 低 | T-15 |
| 2 | 分頁／程序衛生（單一實例、關閉分頁、夜間重啟） | 作業 | 中高（記憶體／凍結） | 低 | T-13/T-14 |
| 3 | AV 排除＋電源計畫＋SSD／RAM | 宿主 | 中（啟動／I/O） | 中（需 IT 核准） | T-04/T-06 |
| 4 | GPU／DPI A/B | 產品 | 中（RAM／渲染） | 低 | T-05 |
| 5 | 網路／AP／TLS | 網路 | 低（效能）／高（安全） | 中 | T-07 |
| 6 | 監視／備份／漂移 | 維運 | 預防性 | 低 | T-08/T-16 |

### 4.1 產品層：`config.ini`（7 鍵）
| 鍵 | 現值 | 目標 | 動作 | 證據等級 |
|---|---|---|---|---|
| `IGNORE_CERT_ERR` | 1 | **1**（條件式→0） | 見 §4.2；HTTP 下維持 1 | ✔ 現值；⚠ 條件式 |
| `KEEP_MACHINE_ID` | 1 | 1 | 保持（授權綁定） | ✔ |
| `DISABLE_HARDWARE_ACCELERATION` | 0 | **0（預設）**；A/B 勝出則改 1 | T-05 A/B | ✔ 現值；△ 目標 |
| `DEBUG_FASTREPORT` | 0 | 0 | 保持；僅原廠指示時暫時 1 | ✔ |
| `FORCE_DEVICE_SACLE_FACTOR` | 0 | 0 | 保持；UI 裁切才評估 1 | ✔ 現值；△ |
| `LANG` | zh_TW | zh_TW | 保持 | ✔ |
| `KEEP_MACHINE_ID_MIGRATED` | 1 | 1 | 保持 | ✔ |

> 其餘 5 鍵與效能無直接關係；不要為了「看起來有調過」而改。

本機目前完整內容（2026-11-11 實測；這就是「產品層完整設定」）：
```ini
IGNORE_CERT_ERR=1
KEEP_MACHINE_ID=1
DISABLE_HARDWARE_ACCELERATION=0
DEBUG_FASTREPORT=0
FORCE_DEVICE_SACLE_FACTOR=0
LANG=zh_TW
KEEP_MACHINE_ID_MIGRATED=1
```
> `DISABLE_HARDWARE_ACCELERATION` 待 T-05 A/B 定案；`IGNORE_CERT_ERR` 依 §4.2 條件式；其餘保持不變。

### 4.2 AP／網路／TLS
| 項目 | 目標 | 說明 | 所有者 |
|---|---|---|---|
| 預設 AP | `8E2EDBE…`（HTTP） | 僅受控 LAN；防火牆白名單 `210.64.124.59:10578` | IT |
| 測試 AP | 僅測試網段；生產不可達 | 避免誤用；必要時先做 T-03 | IT |
| HTTPS 端點盤點 | 列出更新／授權／AP 全部 `https://` | 唯讀掃 app log／AP JSON／`app.asar` 字串；決定 `IGNORE_CERT_ERR` | 代理盤點／IT |
| `IGNORE_CERT_ERR` | HTTP 下 1；測試 AP 已實測 CERT-VALID；其他 HTTPS 端點盤點完成後可 0 | 不以關閉驗證取代 HTTPS | IT＋PMT |
| AP 轉 HTTPS | 原廠正式憑證；必要時 IT 安裝內部 Root CA | 根治 MITM（RSK-E2） | 原廠＋IT |
| TLS 1.0/1.1 | **不主動停用** | 舊 OpenSSL 可能相依；需原廠確認 | IT（⚠） |

### 4.3 Windows 宿主層（系統級；先做 FlapsErp-only，再考慮系統級）
> Scope：`F`=FlapsErp-only；`S`=system-wide；`C`=conditional。任何 `S`/`C` 變更需 IT 核准＋系統級 A/B（§6.9）。T-00 盤點後逐項標「已符合／需變更／不適用」。
| 類別 | 設定 | Scope | 目標 | 理由／風險（對其他程式） | 所有者 |
|---|---|---|---|---|---|
| 電源 | 電源計畫 | S/C | **本機為行動平台（推測筆電）：Balanced**；AC 不睡；不強制 High performance | 避免犧牲電池/熱；效能改善有限 | IT |
| 電源 | 睡眠／休眠／關閉顯示器 | S/C | **筆電保留省電**；RPA 站才 AC 全部 0 | 影響全機電源 | IT |
| 電源 | 快速啟動 | C | **保留預設**（本機為行動平台（推測筆電）） | 停用會使所有使用者開機變慢 | IT |
| 電源 | USB 選擇性暫停 | C | **保留預設**（本機為行動平台（推測筆電））；僅裝置斷線時 | 影響全機 USB 供電 | IT |
| 桌面 | 螢幕保護／鎖定 | S | 依 IT 安全政策；RPA 站不鎖定 | 安全 vs 自動化 | IT（GPO） |
| 記憶體 | RAM | S | 本機 15.79 GB 已足；不加 | 32-bit LAA 4 GB；多 Chromium 仍吃系統 RAM | IT |
| 記憶體 | 分頁檔 | C | **system-managed**（本機） | 固定大小可能限制其他程式 | IT |
| 儲存 | 磁碟 | S | SSD／NVMe；可用 ≥15%（T-00 確認） | 全機 I/O 受益 | IT |
| 儲存 | Defrag／Trim | S | SSD Defrag 關；定期 ReTrim | 全機受益 | IT |
| 儲存 | NTFS Last Access | C | **不做**（本機；0=user-enabled、1=user-disabled、2=system-enabled/default、3=system-disabled） | 影響備份/完整性工具 | IT |
| 安全 | Defender 排除 | S | 最小排除 `FlapsErp`／`Module`／`FlapsErpData` | 安全面下降；不影響效能 | IT（核准） |
| 安全 | WDAC／AppLocker | S | 確認允許；不改政策 | 安全/政策 | IT |
| 安全 | UAC | S | 保持預設 | 不可關閉 | IT |
| 安全 | PowerShell 執行原則 | S | 保持 `Restricted`／`AllSigned` | 不可 Unrestricted | IT |
| 時間 | 時間／時區 | S | NTP／`Taipei Standard Time` | 全機受益 | IT |
| 網路 | NIC 省電 | C | **保留預設**（本機為行動平台（推測筆電）） | 影響全機網路/電力 | IT |
| 驅動 | GPU 驅動 | C | **不更新**（除非現有驅動有問題） | 可能影響其他 GPU 程式 | IT |
| 索引 | Windows Search | C | **不做**（本機） | 影響其他程式搜尋 | IT |
| 視覺 | 視覺效果 | C | **不做**（本機） | 影響全機外觀 | IT |
| 記憶體 | `IncreaseUserVA`／PE LAA | C | **LAA 已實測 True**；x64 下 32-bit 可用 4 GB，不需 `IncreaseUserVA`；僅 32-bit Windows 才評估 | 位址空間；不修改二進位 | 代理／IT |

### 4.4 啟動／分頁／程序衛生（不用改程式）
| 項目 | 目標 | 做法 | 證據等級 |
|---|---|---|---|
| 單一實例 | 只允許一個 FlapsErp | 啟動前檢查 `Get-Process FlapsErp`；重複啟動先提示，不自動 kill（除 IT 腳本） | △ |
| readonly tabs | 關閉未使用分頁 | 使用者手動關閉；每日收工前檢查 | △ |
| 視窗數 | 單一主視窗 | 避免多開；`flapserp://` 深層連結直接開頁 | △ |
| DLL 註冊 | 一次性註冊，不重複 `regsvr32` | 安裝時完成；保持 `.ver` 穩定；AV 排除避免隔離 | △ |
| 夜間重啟 | 每日收工後重啟 FlapsErp | Task Scheduler 排程；若 8h soak 無洩漏則不啟用 | △ |
| 下載資料夾 | 指向 SSD | Windows  Downloads 路徑移到 SSD（若目前非 SSD） | △ |
| 日誌／暫存 | 控制大小 | `logs\archived`、`ReportTemp\Task`、`Pos2Log\archived` 由 IT 依 retention 清理 | △ |

### 4.5 外部工作台（自建 Wellfit；最大效益；繞開原始碼瓶頸）
| 工作台 | 做法 | 繞開的瓶頸 | 證據等級 |
|---|---|---|---|
| A1 檔案通道 | `flapserp://` 深層連結開指定作業；CSV 匯入／匯出 | `sendSync` UI 操作、網路往返 | △（官方入口 ✔） |
| 自建 Wellfit 批次 | 清洗／驗證／修復／彙總在 Wellfit CLI；ERP 只做確認／匯入 | `sendSync`、客端 CPU | ✔（既有管線） |
| 報表外移 | 報表匯出成檔→Wellfit 解析→零 JS 看板 | B-6 客端渲染、B-1 記憶體 | △ |
| 月結對帳 | Wellfit 對帳證據鏈；ERP 只匯入 | 大量資料在 client | △ |
| 大量 COM | 外部 Rust COM bridge 批次；不在渲染進程逐筆 | B-2 `sendSync` | ⚠（EULA／Phase 3） |

> 這一層才是「不能改原始碼」下的真正優化；設定只能輔助。

#### 4.5.1 既有工具（自建；不要重造）
| 工具 | 用途 | 注意 |
|---|---|---|
| `Wellfit/scripts/perf/flaps-perf.ps1` | baseline／com／cdp／app／settings／tls／pe／host／endpoints／hygiene／diff 探測 | 唯讀；輸出 artifacts |
| `Wellfit/scripts/erp-batch/run-mtrix-pipeline.ps1` | MTRIX 流程／冪等驗證（fixture） | 只寫 `OutRoot`（預設 artifacts）；**不寫原廠目錄**；生產批次另用 `wellfit.exe mtrix --dir <唯讀 delivered> --report/--db <artifacts>` |
| `Wellfit/scripts/bench.ps1` | MTRIX 每檔耗時（target <5 s/檔） | 需先建 `wellfit.exe` |
| `Wellfit/scripts/benchmark_synthetic_ingestion.ps1` | Shopee／Reports 合成容量 | **合成資料，非生產 SLA** |
| `Wellfit/scripts/daily-backup.ps1` | Wellfit SQLite ledger 備份（7 天） | 非 `.conf` 備份 |
| `Wellfit/scripts/restore.ps1` | Wellfit ledger 還原 | 還原前會再備份當前 DB |
| `docs/04-operations/A1檔案通道端到端操作手冊_v1.0.md` | A1 五步流程 | **路徑需先修正（§2.5 5–7）** |

#### 4.5.2 批次資源守門（自建 Wellfit；避免拖垮其他程式）
- **優先另機**：Wellfit／MTRIX／報表處理在獨立批次機或伺服器；不要與 FlapsErp 同機。
- **同機時**：只排 02:00–05:00 離峰；`BelowNormal` 優先；必要時 CPU affinity；RAM 上限；可中止。
- **系統級監控**：批次期間每 1 min 取 CPU／RAM／磁碟佇列；若 CPU >80% 或 RAM >85% 持續 5 min，中止批次。
- **其他程式守門**：批次前後各跑一次「代表程式」檢查（瀏覽器開頁、Office 開檔、檔案總管）；任一 >10% 退步即中止。
- **不得**：在營業時間、與 ERP 尖峰同時、或與其他批次同時跑。
- **PowerShell 範例**：`Start-Process` 後 `(Get-Process -Id $p.Id).PriorityClass = 'BelowNormal'`；必要時 `$p.ProcessorAffinity = [IntPtr]0x0F`；RAM 上限用 Job Object（IT）。

### 4.6 監控／備份／更新
| 項目 | 做法 | 頻率 | 所有者 |
|---|---|---|---|
| 基線 | `flaps-perf.ps1 -Probe all` | 每日 09:00／14:00／18:00 | IT |
| app log | 解析 module load→menu ready、tab、report | 每日 | IT |
| COM | `-Probe com` 1000 迴路 | 每週 | IT |
| CDP | `-Probe cdp` | 每月／更新後 | IT |
| 漂移 | config／`.ver` vs manifest | 每季／更新後 | IT |
| 備份（ledger） | `Wellfit/scripts/daily-backup.ps1`（SQLite；7 天 retention） | 每日 | IT |
| 備份（config） | `.conf`＋`FlapsErpData\AP`→`Wellfit\artifacts\config-backup\<date>\`（含授權檔，僅 IT）＋SHA256 | 每週／變更前 | IT |
| 更新 | 原廠更新排離峰；更新後重跑基線＋`.ver` 比對 | 每次更新 | IT |
| 告警 | `FAIL`／`DRIFT`／門檻超標寫 Windows 事件記錄或 `alerts.log` | 即時 | IT |
| 系統 CPU／RAM／磁碟 | `Get-Counter`／`Get-CimInstance`；每 5 min | 每日／批次期間 | IT |
| 其他程式守門 | 代表程式開啟/回應時間；批次前後 | 每次批次 | IT |
| 批次守門 | CPU>80% 或 RAM>85% 持續 5 min → 中止 | 批次期間 | IT |

### 4.7 設定卡（給 IT 的一頁）
> Scope：先做 FlapsErp-only；系統級項目見 §4.3 的 `S`/`C` 標記與 §6.9 系統級 A/B。
| 類別 | 最終目標值 |
|---|---|
| `config.ini` | 7 鍵：維持 6；`DISABLE_HARDWARE_ACCELERATION` A/B 後定案；`IGNORE_CERT_ERR` 依 §4.2 條件式 |
| AP | 預設 `8E2EDBE…` HTTP；測試 AP 生產不可達；防火牆白名單 `210.64.124.59:10578` |
| 電源 | High performance；睡眠／休眠／關閉顯示器 = 0；快速啟動 = 0；USB 選擇性暫停 = 0 |
| 桌面 | 螢幕保護／鎖定 = 關 |
| 記憶體 | RAM ≥16 GB；分頁檔固定 8 GB（SSD） |
| 儲存 | SSD／NVMe；可用 ≥15%；Defrag 關；Trim 定期；NTFS last access 關 |
| 安全 | Defender 排除 `FlapsErp`／`Module`／`FlapsErpData`；WDAC 允許；UAC 預設；PowerShell `Restricted`／`AllSigned` |
| 時間 | NTP／網域；`Taipei Standard Time` |
| 網路 | NIC 省電關閉；AP 直連、無 proxy |
| 驅動 | GPU 最新 WHQL |
| 索引／視覺 | 排除安裝目錄；自訂視覺（保留字型平滑、關動畫） |
| 衛生 | 單一實例；關閉多餘分頁；單一主視窗；夜間重啟（條件式） |
| 工作台（自建） | A1 檔案通道；Wellfit 批次；報表匯出→Wellfit；`flapserp://` 深層連結 |
| 監控 | 每日 perf／app log；每週 COM；每月 CDP；每季漂移；每週備份 |

### 4.8 命令卡（IT 用；A=唯讀盤點、B=系統級條件式）
> **本機為行動平台（推測筆電；Ryzen 7 4800HS / 16 GB / Win11 Home）**：B 段的電源/快速啟動/USB/NTFS/視覺/搜尋/GPU 命令**預設不套用**；只做 A 段盤點＋FlapsErp-only＋IT 核准的 AV 排除/AP 白名單。
```powershell
# A) read-only inventory
# 0) 盤點（唯讀）
powercfg /list; powercfg /a
Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors
Get-CimInstance Win32_PhysicalMemory | Select-Object Capacity,Speed,Manufacturer
Get-PhysicalDisk | Select-Object FriendlyName,MediaType,Size
Get-Volume | Select-Object DriveLetter,FileSystemLabel,Size,SizeRemaining
Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion
Get-MpPreference | Select-Object -ExpandProperty ExclusionPath
Get-ExecutionPolicy -List
Get-TimeZone; w32tm /query /status
Get-ScheduledTask | Where-Object TaskName -like '*Flaps*'
Get-Process FlapsErp -ErrorAction SilentlyContinue
Get-Command windmill -ErrorAction SilentlyContinue
reg query HKCR\flapserp\shell\open\command

# B) system-wide conditional: requires IT approval + system A/B (see 4.3 S/C)
# 1) 電源（GUID 以 powercfg /list 實際值為準；桌機/RPA 站；監控溫度）
powercfg /change standby-timeout-ac 0
powercfg /change hibernate-timeout-ac 0
powercfg /change monitor-timeout-ac 0
powercfg /setacvalueindex SCHEME_CURRENT 2a737441-1930-4402-8d77-b2bebba308a3 48e6b7a6-50f5-4782-a5d4-53bb8f07e226 0
powercfg /setactive SCHEME_CURRENT

# 2) 快速啟動（完整開機；需管理員）
reg add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Power" /v HiberbootEnabled /t REG_DWORD /d 0 /f

# 3) NTFS last access（0=user-enabled,1=user-disabled,2=system-enabled/default,3=system-disabled；本機不套用）
fsutil behavior set disablelastaccess 1

# 4) Defender 最小排除（不關閉 AV）
Add-MpPreference -ExclusionPath 'C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp'
Add-MpPreference -ExclusionPath 'C:\SYSTEM_PROGRAM_DIRECTORY\Module'
Add-MpPreference -ExclusionPath 'C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErpData'

# 5) 時間
w32tm /resync
Set-TimeZone -Id "Taipei Standard Time"

# 6) AP 白名單（outbound）
New-NetFirewallRule -DisplayName "FlapsErp AP HTTP" -Direction Outbound -RemoteAddress 210.64.124.59 -RemotePort 10578 -Action Allow -Profile Any
# 7) 測試 AP 生產不可達（先 Resolve-DnsName test2.flaps.tw 取 IP；若需 Block）
# New-NetFirewallRule -DisplayName "Block FlapsErp Test AP" -Direction Outbound -RemoteAddress <test2-ip> -Action Block -Profile Any
# 註：USB selective suspend 的 setting GUID 以 powercfg /q SCHEME_CURRENT 確認後再套用
```
> `reg add`、`Add-MpPreference`、`New-NetFirewallRule` 需管理員；`powercfg` GUID 以 `powercfg /list` 的實際值為準；分頁檔用 `Win32_PageFileSetting` 或 GUI（避免 `wmic`）。

### 4.9 不要做的事（常見迷思）
| 迷思 | 為什麼不要 |
|---|---|
| 關閉 Windows Defender | 降低安全；只做最小排除 |
| `--single-process`／`--no-sandbox`／`--disable-site-isolation` | Electron 穩定性／安全大退步；無證據效益 |
| 完全關閉視覺效果 | 字型平滑／可讀性變差；只關動畫／陰影 |
| 加未證實的 config 鍵 | 原廠不認；製造假設定 |
| `IncreaseUserVA` 但 PE 未設 LAA | 無效；先 T-03 |
| `IGNORE_CERT_ERR=0` 但無 HTTPS／CA | 可能破壞連線；先 T-03/T-07 |
| RAM booster／記憶體清理工具 | 無實證；可能把工作集推到 pagefile，更慢 |
| 夜間刪 `ReportTemp` | 可能刪到進行中報表；由 IT 依 retention |

### 4.10 全系統互動守門（每次變更都要過）
1. **FlapsErp-only 優先**：config 7 鍵、分頁/程序衛生、單一實例、夜間重啟——不影響其他程式。
2. **系統級變更需 IT 核准＋系統級 A/B**：§4.3 標 `S`/`C` 的項目；未過 A/B 不得宣稱改善。
3. **批次守門**：Wellfit／MTRIX／報表處理優先另機；同機只離峰（02:00–05:00）＋`BelowNormal`＋必要時 CPU affinity＋RAM 上限＋可中止。
4. **併發工作負載**：A/B 時同時跑 FlapsErp＋瀏覽器（10 tabs）＋Excel（10 MB 開/存）＋Outlook（收發）＋1 GB 檔案複製；量測每個程式的回應與系統 CPU／RAM／磁碟佇列／溫度。
5. **溫度／熱降頻**：CPU 溫度 >85°C 或偵測到降頻 → 收回高效能電源計畫，改用平衡。
6. **記憶體壓力**：系統 RAM >85% 持續 5 min → 關閉 FlapsErp 多餘分頁／延後批次；不得讓其他程式進入 pagefile 抖動。
7. **磁碟佇列**：磁碟佇列 >2.0 持續 5 min → 延後批次／報表匯出。
8. **其他程式守門**：代表程式（瀏覽器/Office/檔案總管）回應時間退步 >10% → 回滾該變更。
9. **安全不可交易**：不得關閉 AV／UAC／WDAC；AV 排除只限 FlapsErp 安裝目錄。
10. **記錄**：每次變更存系統基線、A/B 數據、決策；未量測不得宣稱 PASS。

---

## 5. 執行任務（有序；代理 vs IT）

| ID | 任務 | 執行 | 輸入／命令 | 驗收 |
|---|---|---|---|---|
| T-00 | 宿主與環境盤點（唯讀） | 代理 | `Get-ComputerInfo`；`Win32_Processor`；`Win32_PhysicalMemory`；`Get-PhysicalDisk`／`Get-Volume`；`Win32_VideoController`；`Get-NetAdapter`／`Get-NetAdapterPowerManagement`；`Get-MpPreference`；`Get-AppLockerPolicy -Effective`；`Get-ExecutionPolicy -List`；`powercfg /list`、`/a`、`HiberbootEnabled`；`Get-TimeZone`；`w32tm /query /status`；`Get-ScheduledTask`；`Get-Process FlapsErp`；`Get-Command windmill`；`reg query HKCR\flapserp\shell\open\command` | `Wellfit\artifacts\perf\<ts>\system-baseline.json`（host probe；AppLocker/ExecutionPolicy/MpPreference 由 IT 另附）；標出缺口（HDD／RAM／AV 排除／Fast Startup／flapserp 未註冊等）；不寫入原廠目錄 |
| T-00b | 全系統基線（唯讀） | 代理 | 前 20 名 CPU/RAM 行程；`Get-Counter` CPU/可用 RAM/磁碟佇列/延遲；`MSAcpi_ThermalZoneTemperature`；`Get-Service` Search/Update/Backup/Defender；`Get-ScheduledTask`；`Get-MpComputerStatus`；`Get-NetAdapterStatistics` | `Wellfit\artifacts\perf\<ts>\system-baseline.json`；標出其他程式/背景服務/熱/佇列；不寫入原廠目錄 |
| T-01 | 凍結唯讀基線 | 代理 | `flaps-perf.ps1 -Probe all`；`Get-FileHash` 比對 manifest；保存最新 app log 摘要 | timestamped artifact；7 鍵與 manifest 一致或列差異（含 softwareId） |
| T-02 | 擴充 `flaps-perf.ps1`（既有 .ps1；唯讀） | 代理 | `baseline` 加 `Get-CimInstance Win32_Process` 命令列角色標註（`--type=gpu-process`／`renderer`／`utility`）；新增 `-Probe app`（參數 `-LogDir` 預設 `C:\SYSTEM_PROGRAM_DIRECTORY\logs`）：解析最新 `application-WELLFIT-*.log`（依 LastWriteTime、FileShare.ReadWrite），輸出 `module_load→menu_ready_ms`、`menu→first_tab_ms`（無 ready 標記則 UNKNOWN）、`report_task`、視窗／螢幕；新增 `-Probe settings`（參數 `-ConfigPath`、`-ApDir`、`-Manifest`）：7 鍵、AP ID／scheme／host／port（其餘遮罩）、Module `.ver`、manifest hash 比對；`all` 保持 baseline+com+cdp（向後相容），新增 `full` 含 app+settings+tls+pe+host+endpoints+hygiene；新增 `hygiene`（程序/角色/processed.ini/Module `.ver`/flapserp/windmill/Defender/ExecutionPolicy/AppLocker）與 `diff`（兩 artifacts 比較；環境指標 INFO；true median）；baseline/com/host 輸出 `contended`（`link`/`cargo`/`rustc`、free<1 GB、diskQueue>5） | 未執行時 `APP-NOT-RUNNING`；不寫入 `C:\SYSTEM_PROGRAM_DIRECTORY`；`.ps1` 無 BOM、ASCII 字面量 |
| T-03 | TLS／二進位唯讀探測 | 代理 | `SslStream` 檢查 `test2.flaps.tw:10113`（主體／發行者／到期／鏈／主機名／TLS 版本／撤銷狀態；探測用 callback 僅在 probe 內接受）；讀 `FlapsErp.exe` PE `IMAGE_FILE_LARGE_ADDRESS_AWARE`；唯讀掃 `app.asar`／log／AP JSON 的 `https://` 端點清單（決定 `IGNORE_CERT_ERR` 影響面） | JSON：`CERT-VALID`／`CERT-UNTRUSTED`／`CERT-EXPIRED`／`UNREACHABLE`；LAA `SET`／`NOT-SET`；HTTPS 端點清單 |
| T-04 | Tier 0 宿主套用＋前後量測 | IT 套用；代理量測 | 電源計畫／睡眠／休眠／快速啟動／USB／顯示器；AV 排除；NTFS last access；時間／時區；搜尋排除；config.ini 先備份、`Format-Hex` 檢查 BOM、**逐行替換不得重排**、保留 CRLF、`[IO.File]::WriteAllText` UTF-8 no BOM、記 SHA256 | `powercfg /getactivescheme`、`powercfg /a`、`Get-MpPreference`、`fsutil behavior query disablelastaccess`、`MSAcpi_ThermalZoneTemperature`、`Get-TimeZone` 前後輸出；config hash；T-01 重跑對照；無熱降頻 |
| T-05 | 硬體加速／DPI A/B | 代理量測；IT 套用 | Arm A `DISABLE_HARDWARE_ACCELERATION=0` vs Arm B `=1`；每臂 **5 次**重啟；比較 RAM median／max、GPU 程序 RAM、app log 時間、主觀凍結；DPI 0 vs 1（若裁切） | 每臂 5 筆 `perf-result.json`＋app log；判定表；任一指標惡化 >10% 保留原值 |
| T-06 | OS 進階（分頁檔／網路／視覺） | IT | 分頁檔（預設 system-managed；實測抖動才固定；`Win32_PageFileSetting` 或 GUI）；NIC 省電（桌機/RPA 站）；視覺效果（非必要不做）；`Optimize-Volume -ReTrim`；溫度/降頻檢查 | `Get-CimInstance Win32_PageFileUsage`、`Get-NetAdapterPowerManagement`、`MSAcpi_ThermalZoneTemperature` 前後輸出；無熱降頻 |
| T-07 | AP／網路／TLS | IT | 預設 AP 不變；防火牆白名單（`New-NetFirewallRule ... -RemoteAddress 210.64.124.59 -RemotePort 10578 -Action Allow`）；測試 AP 生產不可達（必要時 Block 規則）；必要時安裝 CA；`IGNORE_CERT_ERR` 決策 | `Test-NetConnection 210.64.124.59 -Port 10578` 成功；測試 AP 生產不可達；憑證證據 |
| T-08 | 排程／監控／告警 | IT／代理 | 註冊每日／每週／每月；app log 門檻；`FAIL`／`DRIFT` 告警；temp 大小監控；系統 CPU/RAM/磁碟佇列；其他程式守門 | `Get-ScheduledTask`；手動 run 成功；artifact＋alert；系統指標；其他程式無退步 |
| T-09 | 文件更正 | 代理 | 修正 §2.5 七項（含 A1 手冊紅線路徑）；軟體邊界標註（原廠 FlapsErp vs 自建 Wellfit）；LAA/HTTP/COM 更正；v4.0 校正（NTFS 0/1/2、runbook 補強）；`PLAN_REGISTER.md` addendum #33；`docs/README.md` 索引 | 路徑／鍵與實機一致；無懸空引用；A1 手冊不再寫入 `C:\SYSTEM_PROGRAM_DIRECTORY` |
| T-10 | 最佳設定基線文件 | 代理 | 新增 `docs/04-operations/FlapsErp最佳設定基線_v1.0.md`（產品／AP／宿主／衛生／工作台／監控／驗收／回滾／變更紀錄） | 與 T-00..T-16 證據一致；未量測標 UNKNOWN |
| T-11 | A/B 報告與發布判定 | 代理 | 彙整 T-00..T-07、T-13、T-15；以 `-Probe diff` 產出差異（contended 樣本不列入）；A–L；五級判定 | 報告含命令、exit code、日期、環境；預期 **CONDITIONAL PASS**（8h soak 前） |
| T-12 | 8h soak（生產必要） | IT／代理 | 連續 8h；每小時循環：0–10 min 開/關 10 tabs（FAS/HRS/INV/POS）、10–20 min 匯出 5 報表、20–60 min idle；同時跑瀏覽器 10 tabs／Excel／Outlook（併發）；每 5 min RAM；每 10 min 檢查 app log | RAM 增長 <50 MB；無崩潰；Event Log 無錯誤；其他程式無 >10% 退步 |
| T-13 | 分頁／程序衛生 | IT／使用者 | 單一實例檢查腳本（重複啟動先提示，不自動 kill，除 IT 腳本）；關閉多餘 readonly tabs；單一主視窗；下載資料夾 SSD；以 `-Probe hygiene` 與 T-02 角色清單為對照（基線 11 行程）；比對 `.conf\processed.ini`／`.ver` 啟動前後是否變動（若每次變動→DLL 重註冊） | 行程數／游離 RAM 下降；UI 凍結 -50%（△）；DLL 不重複註冊 |
| T-14 | 夜間重啟（條件式） | IT | 若 T-12 顯示洩漏，才排程每日收工後重啟；排 02:00 或批次後，先確認無夜間使用者 | 8h RAM 增長 <50 MB；不影響夜間批次 |
| T-15 | A1／自建 Wellfit／報表外移 | 開發／IT | 先修 A1 手冊路徑（T-09）；先以 `run-mtrix-pipeline.ps1`（fixture）驗證流程；生產批次用 `wellfit.exe mtrix --dir <唯讀 delivered> --report/--db <artifacts>`（不寫原廠目錄）；`bench.ps1` 取每檔基線；`benchmark_synthetic_ingestion.ps1` 取合成容量（標明非生產 SLA）；`flapserp://` 深層連結；報表匯出→Wellfit 解析；`reconcile` 未實作，以 ledger＋report 對帳 | 批次工時 -80%（△）；報表 ≤60 s／10k 列（△）；client RAM 不爆；合成與真實數據分開標記 |
| T-16 | 備份／更新演練 | IT | ledger 用 `daily-backup.ps1`（7 天）；config 每週備份 `config.ini`／`processed.ini`／`*.version.txt`／`Module\*.ver`／`AP\*.json` 至 `Wellfit\artifacts\config-backup\<date>\`（含授權檔，僅 IT）＋SHA256；更新後重跑基線＋`.ver` 比對 | 還原演練成功（`restore.ps1`＋config manifest）；更新後無漂移 |

---

## 6. A/B 量測協議（避免假改善）

1. 同一台機器、同一使用者、同一 AP、同一時段；關閉其他應用程式；量測期間不跑 `cargo build`。
2. 每臂開始前重啟 FlapsErp，等待 app log `menu.prepareColorMask`。**冷啟動（重開機後）與暖啟動（僅重啟 app）分開記錄，不得混算。**
3. **每臂 5 次**（app log 變異大；COM 取樣變異大，P95 0.21–0.40、P99 0.24–0.48 ms）；取中位數，記錄 min／max。
4. 指標：
   - app log：`module_load→menu_ready`、`menu.finishDownloadRequires`、`menu.prepareDetails`、可辨識 tab／report 事件。
   - 記憶體：`flaps-perf.ps1 -Probe baseline`（11 行程 median／max／總量；含 `--type` 角色：gpu／renderer／utility）。
   - COM proxy：`-Probe com` P95（僅地板參考）。
   - GPU 程序 RAM（若可取得；否則 UNKNOWN）。
   - 主觀 UI 凍結：同一操作 5 次，記錄 >1 s 次數。
   - 報表：匯出＋Wellfit 解析耗時（T-15）。
5. 判定：效能專案任一指標惡化 >10% → 保留原值；改善 >10% 且無其他退步 → 可採用；混合 → 保留原值並記錄。**安全變更（TLS／CA／WDAC）採另一把尺：只要安全提升且無功能破壞，即使效能小幅退步也可接受，但必須記錄。**
6. 禁止：只跑 1 次、冷啟動混入穩態、同時多重背景工作、以估算代替實測。
7. 每筆結果附原始 `perf-result.json`／app log 路徑；報告只引用 artifacts，不引用記憶。
8. **每次變更後的 app smoke（必過，任一失敗即回滾）**：① 啟動→登入→選單就緒；② 開 FAS 應付憑單→列表載入；③ 開 HRS 員工→列表載入；④ 報表→匯出 CSV；⑤ 關閉→重啟→再登入；⑥ 若改 `IGNORE_CERT_ERR`／CA，加測 HTTPS AP 登入＋更新檢查。
9. **系統級 A/B（所有 S/C 變更必做）**：除 FlapsErp 指標外，量測代表程式（瀏覽器開頁、Office 開檔、檔案總管）與系統 CPU／RAM／磁碟；任一退步 >10% 即回滾。§4.3 標 `S`/`C` 者適用。
10. **併發工作負載 A/B（系統級變更必做）**：
    - 負載：FlapsErp 登入＋10 tabs＋5 報表匯出；瀏覽器 10 tabs；Excel 10 MB 開/存；Outlook 收發；1 GB 檔案複製。
    - 指標：每個程式回應時間、系統 CPU／RAM／磁碟佇列／GPU／溫度。
    - 判定：任一程式退步 >10% 或溫度降頻 → 回滾。
11. **CONTENDED 樣本**：`baseline.contended`／`com.contended`／`host.contended` 任一為 true（`link`/`cargo`/`rustc`、free<1 GB、diskQueue>5）→ 樣本無效；先結束建置或等離峰；不得用於 A/B。

---

## 7. 驗收矩陣（含成效目標）

| 驗收碼 | 項目 | 可量測條件 | 狀態 |
|---|---|---|---|
| AC-S01 | 7 鍵與實機一致 | `settings` probe 逐字比對 | 待執行 |
| AC-S02 | GPU／DPI 決策有證據 | 兩臂各 5 筆；判定表；無崩潰 | 待執行 |
| AC-S03 | TLS 決策有證據 | T-03 JSON＋T-05/T-07 決策記錄 | 待執行 |
| AC-S04 | OS 設定可驗證 | `powercfg`／`Get-MpPreference`／`fsutil`／`Get-ScheduledTask` 前後輸出 | 待執行 |
| AC-S05 | 啟動／選單改善 | menu ready 改善 ≥10%（≤15.4 s）；-20%（≤13.7 s）為理想值（△） | 待執行 |
| AC-S06 | 記憶體改善 | RAM median ≤41.6 MB（-10%，△） | 待執行 |
| AC-S07 | 無效能退步 | 所有指標 ≤ 基線 +10% | 待執行 |
| AC-S08 | 8h 穩定 | RAM 增長 <50 MB；無崩潰 | 待執行 |
| AC-S09 | 文件與實機一致 | 手冊 §3.2／基線報告爭議已結 | 待執行 |
| AC-S10 | 外部工作台驗證 | A1／Wellfit／報表外移端到端；工時／耗時記錄 | 待執行 |
| AC-S11 | 系統級不退化 | 代表程式（瀏覽器/Office/檔案總管）回應 ≤ 基線 +10%；系統 CPU/RAM/磁碟佇列/溫度 ≤ 基線 +10%；批次守門有效；無熱降頻 | 待執行 |

---

## 8. 回滾

| 變更 | 回滾程序 | 驗證 |
|---|---|---|
| `config.ini` | 從 `config-snapshot/` 還原＋SHA256 manifest；重啟 | 7 鍵還原；可登入 |
| OS 設定 | IT runbook 記錄原值；逐項還原（`powercfg`、`HiberbootEnabled`、分頁檔、AV 排除、`fsutil`、NIC、排程） | 前後輸出對照 |
| AP 設定 | 還原 `managerLayout.json`／AP JSON（若變更）；預設不變 | AP 可連；default 正確 |
| 排程 | `Unregister-ScheduledTask` | 無殘留 |
| 外部工作台 | 停用排程／feature flag；保留 ledger | 可回到人工流程 |
| 文件 | 標 `SUPERSEDED`＋archive | 索引無死鏈 |

---

## 9. 風險與紅線

| ID | 風險 | 等級 | 緩解 |
|---|---|---|---|
| RSK-E5 | 寫入 `C:\SYSTEM_PROGRAM_DIRECTORY` | 極高 | 代理唯讀；IT 變更管制 |
| RSK-E2 | AP HTTP 明文；帳密可能明文 | 極高 | 受控 LAN；要求 HTTPS＋正式憑證；不以關閉驗證取代 |
| RSK-E3 | 32-bit LAA 4 GB 上限（實測） | 高 | 設定無法根治；分頁衛生／報表外移／夜間重啟；A 線 64-bit |
| RSK-E4 | 原廠元件漂移 | 高 | `.ver`／config 每季＋更新後比對 |
| NEW-01 | AV 排除／WDAC 調整降低安全 | 中 | 最小排除；IT 核准；不關閉 AV／UAC |
| NEW-02 | DPI／縮放變更改版 UI | 中 | A/B；可回退 |
| NEW-03 | 快速啟動停用／分頁檔變更需重開機 | 低 | 變更窗 |
| NEW-04 | 夜間重啟影響夜間批次 | 中 | 先看 T-12；必要才啟用；避開批次時段 |
| NEW-05 | 外部工作台繞過原廠 UI 流程導致資料不一致 | 中 | 只用官方匯入入口；ledger 對帳；不直寫 DB |
| RSK-S1 | 為 FlapsErp 優化而拖垮其他程式 | 極高 | FlapsErp-only 優先；系統級 A/B；批次守門；另機/離峰 |
| RSK-S2 | Wellfit 批次與其他程式搶資源 | 高 | 離峰、BelowNormal、CPU/RAM 上限、可中止 |
| RSK-S3 | 系統級設定（電源/驅動/分頁檔）影響全機 | 中 | 條件式；IT 核准；系統級 A/B；可回滾 |
| RSK-S4 | 高效能電源計畫/長時間高載造成熱降頻 | 高 | 監控溫度；>85°C 或降頻即收回平衡；桌機/RPA 站 |
| RSK-S5 | FlapsErp＋瀏覽器＋Office 記憶體壓力→pagefile 抖動 | 高 | 分頁衛生；監控 RAM；>85% 延後批次 |
| RSK-S6 | 批次/報表/備份 I/O 拖慢其他程式 | 高 | 離峰/低優先/限制；磁碟佇列守門 |

**紅線**：不得修改 `FlapsErp.exe`／`app.asar`／任何原廠二進位；不得直接寫 ERP 資料庫；所有寫入走官方匯入入口；不得為效能關閉 AV／UAC／WDAC；不得新增未證實的 config 鍵。

---

## 10. 交付物

1. `docs/04-operations/FlapsErp最佳設定基線_v1.0.md`（新增；SSOT）。
2. `docs/04-operations/FlapsErp設定變更runbook_v1.0.md`（新增；IT 備份／變更／回滾）。
3. `docs/research/FlapsErp設定AB實測報告_v1.0.md`（新增；A–L＋五級判定）。
4. `docs/04-operations/FlapsErp效能優化操作手冊_v1.0.md`（更正 §3.2／§3.4；連新基線）。
5. `docs/04-operations/A1檔案通道端到端操作手冊_v1.0.md`（更正 §1.2／Step 2／Step 4 路徑；移除對 `C:\SYSTEM_PROGRAM_DIRECTORY` 的寫入）。
6. `docs/research/FlapsErp效能基線報告_v1.0.md`（補 AP HTTP、screen、softwareId）。
7. `docs/research/FlapsErp優化_合規風險追蹤表.md`（RSK-E2 更新；新增 HTTP AP）。
8. `docs/PLAN_REGISTER.md` addendum #33；`docs/README.md` 索引。
9. 證據 artifacts：`Wellfit/artifacts/perf/<ts>/`（含 `system-baseline.json`、`endpoints.json`、`hygiene.json`、`diff.json`）、`artifacts/erp-batch/<ts>/`。

---

## 11. A–L 證據包（C-13）

| 節 | 內容 |
|---|---|
| A | 目標、範圍、非目標 |
| B | 2026-11-11 唯讀現況（§2） |
| C | 決策 D-01..D-15 |
| D | 變更內容（§4；僅 IT 套用） |
| E | 量測方法（§6；命令、環境、5 次） |
| F | 實測結果（T-00..T-16 artifacts） |
| G | 前後對照與判定（§7） |
| H | 風險與回滾（§8、§9） |
| I | 合規與紅線（RSK-E5、EULA、OSP） |
| J | 文件與追溯（§10） |
| K | 未決事項（§12） |
| L | 發布判定（五級；預期 CONDITIONAL PASS） |

---

## 12. 未決／範圍外

| 項目 | 狀態 | 解除條件 |
|---|---|---|
| IT 變更所有者／測試機 | 待指派 | IT 指定；測試機或維護窗 |
| `softwareId.txt` SHA256 爭議 | 已結 | 目前＝manifest `D5E6…42090C4C1D38567CA453`；報告 `…4010EE21977F8990F6D8` 為繕寫錯誤 |
| `FORCE_DEVICE_SACLE_FACTOR` 正確鍵行為 | △ | 原廠確認（不可取原始碼） |
| PE LAA 旗標／`IncreaseUserVA` | 已實測 LAA=True | x64 下 32-bit 可用 4 GB；`IncreaseUserVA` 不適用於 x64 |
| AP 版本漂移 | 已觀測 | `8E2EDBE…version.txt` 與 2026-11-03 manifest 不同；T-16 更新後比對 |
| 生產 AP 轉 HTTPS | 待原廠 | OQ-F1/F2（11-19） |
| 8h soak | 生產必要 | T-12 |
| Windmill 是否已安裝 | 待 T-00 | `Get-Command windmill`；未裝由 IT 裝到 `C:\ProgramData\Windmill` |
| A1 `reconcile` 未實作 | 已知 | 以 ledger＋report 對帳；實作另排 |
| A1 手冊路徑修正 | 待 T-09 | 不再寫入原廠目錄 |
| 批次主機（另機/同機） | 待定 | 若同機，需離峰＋守門；若另機，需 IT 提供 |
| 系統級 A/B（其他程式） | 待 T-04/T-06 | 代表程式回應/CPU/RAM/磁碟前後對照 |
| COM／CDP 自動化正式化 | 範圍外 | Phase 3–4 |
| Rust 平台線 DB／API 設定 | 範圍外 | 平台線報告 |

---

*本計畫僅規劃；任何寫入 `C:\SYSTEM_PROGRAM_DIRECTORY` 或 OS 的變更由 IT 於變更管制下執行。所有數字附量測方法與日期；UNKNOWN ≠ PASS ≠ FAIL。*