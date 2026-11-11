# 輔翼ERP（FlapsErp）逆向工程分析與效能最佳化方案 v1.0

> 文件編號：RSH-2026-1103-001｜版本：v1.0｜日期：2026-11-03｜狀態：Draft（待主管評審）
> 分析者：工程分析（逆向工程 + 網路查證 + 知識庫核對）
> 證據標記：✔ 已觀測（直接逆向證據）｜△ 推導（依證據推斷）｜⚠ 未驗證（需確認）
> 本文件遵循 claims-vs-evidence 紀律：每項宣稱指向證據來源；推導與猜測分離標記。

---

## 0. 證據基礎（Evidence Base）

| # | 證據來源 | 取得方式 | 完整性 |
|---|---|---|---|
| E-01 | 安裝檔 `C:\Users\stephanie.huang\Downloads\FlapsErp.Installer (1).exe` | 中繼資料 + SHA256 | ✔ 完整 |
| E-02 | 安裝目錄 `C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp\`（19 項 + script/ + resources/） | 目錄掃描 | ✔ 完整 |
| E-03 | `resources\app.asar`（11,832,082 bytes，1,177 檔案） | asar 解析 + 26 檔案提取 + 逐行分析 | ✔ 核心 26 檔全讀 |
| E-04 | 執行時期元件 `C:\SYSTEM_PROGRAM_DIRECTORY\`（12 檔 + .ver） | 目錄掃描 | ✔ 完整 |
| E-05 | `C:\SYSTEM_PROGRAM_DIRECTORY\.conf\`（config.ini、machineId/hardwareId/softwareId.txt） | 讀取（金鑰值遮蔽） | ✔ 完整 |
| E-06 | `C:\SYSTEM_PROGRAM_DIRECTORY\Module\`（POSMachineController.jsc、Winax.node） | 目錄掃描 | ✔ 完整 |
| E-07 | `FlapsErpData\AP\*.json`（3 台 AP 伺服器設定，僅讀結構未取值） | 結構檢視 | ✔ 結構完整 |
| E-08 | 網路：flaps.com.tw 官方網站（about/products/erp/ecommerce-integration 等 7 頁） | websearch | ✔ 官方公開資訊 |
| E-09 | 知識庫：`LOCAL-018`、`OI-006`、`claims-vs-evidence.md` C-17/C-22、`erp-report-contract.md` | 核對 | ✔ |
| E-10 | 安裝檔 SHA256：`DC77C024629629149842714B8F599565DB04121AF043F5517450BAA5F0AE79B6` | Get-FileHash | ✔ |

**覆蓋矩陣**：使用者要求 8 項 → ①專案文件/安裝套件掃描 §1；②網路資訊 §1.5；③專業分析+逆向解構 §1-2；④分層優缺點 §2；⑤效能優化 §3；⑥流程自動化 §4；⑦內外資料串接 §5；⑧大量批次處理 §6；⑨應建構系統/組件/架構 §7；⑩重構/優化/創新 §8。全數覆蓋。

---

## 1. 系統解構（逆向工程確認）

### 1.1 一句話架構

> **輔翼ERP = 「Web 雲端 ERP（Java/JSP 後端）」 + 「Electron 桌面外殼（32-bit）」 + 「COM/ActiveX 硬體與報表元件層（Delphi 時代 DLL）」 + 「協力批次管線（我方 Wellfit）」** ✔

### 1.2 五層解構

| 層 | 內容 | 證據 |
|---|---|---|
| L1 外殼 | Electron（Chromium+Node.js），exe 134MB；視窗 1500×720 無邊框；`nodeIntegration:true`、`webviewTag:true`、`backgroundThrottling:false`；多 BrowserView（title/menu/connect/tabs/readonlyTabs） | main.js ✔ |
| L2 UI | 實際 ERP 畫面由 **AP 伺服器以 web 頁面渲染**（webview 分頁載入），外殼僅提供分頁/標題/選單/下載；外殼自身 UI 用 Vue 2.6.12 + Bootstrap 5.0.1 | main.js `connectInfo.ap`、`js/vue2.6.12.js` ✔ |
| L3 後端 | Java JSP（`{apUrl}/RDAPortal.jsp`），XML 協定 `com.flaps.Module.RDA.TGetModuleInfo`；官方網站證實雲端 ERP（erp.flaps.com.tw / oms.flaps.com.tw / dashboard.flaps.com.tw）、FLAPS.AI Text-to-SQL | dllManager.js ✔ + 官網 ✔ |
| L4 元件 | COM DLL 動態部署：RDAPortal 取得模組資訊 → 下載至 `C:\SYSTEM_PROGRAM_DIRECTORY\` → PowerShell `regsvr32 -Verb RunAs` 註冊（`.ver` 版本控管）；含 midas.dll（Delphi DataSnap）、MReportViewer.dll（14.6MB 報表）、POSMachineController.dll（4.5MB POS）、iPOSCustDisplay.dll、BHT.exe（手持終端，IrDA/Ymodem 序列傳輸）、LRDASProcesser.exe、libeay32/ssleay32（舊版 OpenSSL） | dllManager.js + E-04 ✔ |
| L5 授權與配置 | machineId/hardwareId/softwareId 綁定（`KEEP_MACHINE_ID=1`）；config.ini（`IGNORE_CERT_ERR=1`、`DEBUG_FASTREPORT`、`LANG=zh_TW`）；`.jsc` bytenode 位元碼模組 | E-05/E-06 ✔ |

### 1.3 六條整合通道（自動化的官方入口）✔

| 通道 | 機制 | 自動化潛力 |
|---|---|---|
| CH-1 | `flapserp://` 自訂協定：`flapserp://{base64(JSON)}` 啟動參數（含 multi 多開協商、test 模式） | **官方支援的外部啟動通道** — 可由任何程式/排程器帶參數開啟指定 AP 作業 |
| CH-2 | `RDAPortal.jsp` XML RDA 協定（POST，Content-Type application/xml） | **伺服器級直連** — 已觀測 TGetModuleInfo；△推斷同協定應有業務 API，⚠ 需協議探索與原廠同意 |
| CH-3 | COM/ActiveX：`FLAPSConsoleCOM.AXCXMLNode`（active_patch.js 提供 JS polyfill 證明其存在）+ winax + `Module\*.jsc` 模組 | **處理序間橋接** — 我方 Rust COM bridge（platform/src/com）設計方向獲驗證 |
| CH-4 | 檔案交換：下載（saveFile/downloadFile API）+ CSV 匯入匯出 | 我方 Wellfit 主戰場（8 欄契約、冪等帳本）✔ |
| CH-5 | 序列埠/硬體：serialport 10.4 + BHT（IrDA/Ymodem）+ iPOS 客顯 + POS 機 | 門市硬體整合 △ |
| CH-6 | Playwright（dev 測試；`option.test===true` 開空白窗配合 launch） | **官方自我測試即用 CDP** — △推導外殼可被 Playwright/CDP 驅動 |

### 1.4 知識庫缺口關閉宣告（重大成果）

知識庫 `LOCAL-018`／`OI-006` 原記載：**「尚未證實：ERP 的開發語言、UI 框架、資料庫種類與位置」**。本次逆向已回答：

| 原缺口 | 現在答案 | 證據等級 |
|---|---|---|
| 開發語言 | 外殼 JavaScript（Electron/Node）；後端 Java（JSP）；元件層 Delphi/C++（midas、regsvr32 COM） | ✔ E-03/E-04 |
| UI 框架 | **Web UI（伺服器渲染，webview 呈現）**，非原生 Win32；外殼 UI Vue 2.6.12 + Bootstrap 5 | ✔ E-03 |
| 資料庫 | 未直接觀測（在 AP 伺服器端）；△ 推導：JSP+midas(DataSnap)+FastReport 指向關聯式 DB | △ |
| 報表引擎 | **FastReport**（config.ini `DEBUG_FASTREPORT`）+ MReportViewer.dll 客端檢視 | ✔ E-05/E-04 |

**連帶重大修正**：藍圖 V1.0「Layer 3 FlaUI/UIA 桌面自動化」**方向錯誤** — ERP UI 是 web 內容（Chromium），正確工具為 **CDP/DOM 自動化**或**直接呼叫 RDAPortal**（§4）。此修正大幅降低 Layer 3 工作量與脆弱度。

---

## 2. 分層評估：優點 / 缺點 / 缺陷

### 2.1 產品層

| 優點 ✔ | 缺點 | 缺陷 |
|---|---|---|
| 零售業深度客製（進銷存/財會/POS/EC/稅務/簽核/加盟 一條龍，官網 E-08） | 闭源黑箱（bytenode 保護、無公開 API 文件 ⚠） | 對帳/拋轉仍有斷點（官網自述客戶月底對帳需數週 → 官網亦承認痛點存在） |
| 500+ 品牌、19 家銀行/電支串接（官網） | 外掛式元件多、安裝面積大（安裝 64MB + 元件 20MB+ + 執行期下載 DLL） | — |
| OMO 全通路（91APP/SHOPLINE/Shopify/蝦皮/MOMO/CYBERBIZ API + Excel） | — | — |
| 外殼具官方自動化入口（flapserp://、RDA、COM、Playwright 自測） | — | — |

### 2.2 架構層

| 優點 ✔ | 缺點 ✔ | 缺陷 ✔ |
|---|---|---|
| 殼分離：商業邏輯在伺服器，client 薄殼升級容易（UpdateAsar 熱更新機制） | Electron 殼承重過多：COM、FFI、序列埠全塞進 GUI 處理序 | **32-bit 打包（ia32）**：LAA 實測 True（x64 下 4 GB），大報表/大批次仍可能撞牆（electron-builder-dev.yml `arch: ia32`） |
| 元件版本控管（.ver + 動態下載註冊） | DLL 註冊需 UAC 提權 + `Set-ExecutionPolicy Unrestricted`（dllManager.js:225） | 元件層 DLL 與殼耦合：殼更新不會同步修 DLL（各別 .ver 漂移風險） |
| 多 AP 伺服器設定（AP/*.json 3 台）+ 連線測試器（netUtil.ConnectionTester） | 同步 IPC 承載 COM 呼叫（見 §3 B-2） | `GetThumbnail` 硬編碼 'AnyDesk' 寫 3222.png（mainApi.js:766-788，死碼/開發殘留） |
| 冪等式單一實例鎖 + 多開協商設計細膩 | 配置鍵有拼字錯誤且需雙寫相容：`FORCE_DEVICE_SACLE_FACTOR`（main.js:186 同時檢查正確鍵） | 空檔案殘留：dllManager.html 0 bytes、apManagerRenderer.js 0 bytes ✔ |
| 下載互動（開啟/儲存/取消三分流）設計完整 | `executeCommand`/`excuteCommand` 拼字別名共存（mainApi.js:1393-1394） | `console.log(args)` 每次 IPC 全量輸出（main.js:265）— 效能噪音 + 敏感資料可能進 console |

### 2.3 效能層（10 項瓶頸，附證據）

| # | 瓶頸 | 證據 | 影響 |
|---|---|---|---|
| B-1 | 32-bit 位址空間上限 | electron-builder-dev.yml `ia32` ✔ | 大量 SKU/報表 JSON → 記憶體壓力、GC 停頓 |
| B-2 | **COM 呼叫走同步 IPC**：每個方法/屬性標記 `send:'sendSync'`，渲染進程逐呼叫阻塞 | mainApi.js:193,195,225 ✔ | 自動化/POS 作業時 **UI 凍結**；批次掃碼/出貨時延遲疊加 |
| B-3 | Electron 基底記憶體（Chromium+Node+V8，exe 134MB） | E-02 ✔ | 桌面常駐 300-500MB+ △ |
| B-4 | `backgroundThrottling:false` | main.js:294 ✔ | 背景分頁持續耗 CPU，多分頁開啟時整機變慢 |
| B-5 | UI 操作全走網路往返（webview → JSP） | 架構 ✔ | 網速敏感；斷線即無 UI（有 TestConnection 補救）✔ |
| B-6 | 報表客端渲染：MReportViewer.dll 14.6MB + FastReport | E-04/E-05 ✔ | 大報表佔用 client CPU/RAM；與 B-1 疊加 |
| B-7 | DLL 註冊流程：每次下載+UAC+regsvr32（同步等待 PowerShell 行程） | dllManager.js ✔ | 部署/修復摩擦，第一次啟動慢 |
| B-8 | 每 IPC `console.log(args)` + handler.result | main.js:265,277 ✔ | 高頻操作時序列化開銷、console 卡頓 |
| B-9 | 下載流程：暫存檔 + shell.openPath（非串流直接開） | main.js:448-512 ✔ | 大檔下載→開啟延遲雙倍 |
| B-10 | 舊版 OpenSSL（libeay32/ssleay32）+ Vue 2.6.12（EOL） | E-04/E-03 ✔ | 非直接效能，但安全修補停滯 → 長期風險成本 |

### 2.4 安全層

| # | 事項 | 證據 | 等級 |
|---|---|---|---|
| S-1 | `IGNORE_CERT_ERR=1` 預設生效（`!== '0'` 即忽略憑證錯誤） | main.js:190-192 + E-05 ✔ | 高 — MITM 風險 |
| S-2 | `nodeIntegration:true` + `ExecuteCommand`/`ExecuteFile` 任意行程執行 API | mainApi.js:570-598 ✔ | 高 — 若 webview 內容被注入即可指揮本機 |
| S-3 | `Set-ExecutionPolicy Unrestricted` + regsvr32 -Verb RunAs | dllManager.js:225 ✔ | 中 — 提權面擴大 |
| S-4 | 舊 OpenSSL、Vue 2 EOL、Chromium 版本未知（v0.0.4 2025-04 建置） | E-02 ✔ | 中 |
| S-5 | 正面：本機未發現明文憑證（AP json 僅 url/testUrl）✔；machineId 授權綁定 ✔ | E-07/E-05 | — |

### 2.5 可維護性層

| 優點 | 缺點 |
|---|---|
| bytenode 保護商業邏輯（.jsc）| 對整合方（我方）即黑箱：無 API 文件、無契約（OI-006 痛點根源）|
| i18n 模組化（flaps-i18n、langs/）| 原始碼內 TDOO/拼字錯誤多（多語 TDOO、SACLE、excuteCommand）|
| Playwright 測試存在（有測試文化 △）| 0 bytes 死檔、死碼（AnyDesk）未清 |

---

## 3. 效能優化方案（P0→P2，可執行）

> 原則：我方無法改原廠程式碼（閉源 + bytenode）。**分兩線**：A 線=對原廠的需求/回報（透過客服/顧問通道，附本報告證據）；B 線=我方環境與管線側優化（可立即執行）。

### A 線 — 對原廠建議（附證據編號，可轉交）

| 優先 | 建議 | 依據 |
|---|---|---|
| P0 | 提供 **64-bit (x64) 打包**（至少雙架構並行） | B-1 |
| P0 | 移除 `IGNORE_CERT_ERR` 預設開啟，改為明確設定 + 憑證釘選 | S-1 |
| P1 | COM 呼叫改 **非同步 IPC（ipcRenderer.invoke + 佇列）**，或提供官方批次 API | B-2 |
| P1 | 移除每次 IPC 的 `console.log(args)`，改條件式 debug log | B-8 |
| P1 | `backgroundThrottling` 依分頁可見性動態開啟 | B-4 |
| P2 | 升級 OpenSSL / Vue 3 / Chromium；清理死碼（GetThumbnail、空檔案） | B-10、§2.2 |

### B 線 — 我方環境側（Wellfit + 配置 + 流程）

| 優先 | 行動 | 具體做法 | 驗收 |
|---|---|---|---|
| P0 | **批次工作全部外移 Wellfit**：ERP 只做「人機確認 + 匯入/匯出」，重活（清洗、驗證、比對、大檔）由 Wellfit CLI 執行 | 現行 7-crate 管線已具備：ColumnContract 名稱解析、冪等帳本、ReleaseGate、ScaledDecimal 整數分 | `build.ps1 test --workspace` 全綠（現行 235+ 通過）✔ 基線已達 |
| P0 | **效能基線量測**（先測再優化）：記錄啟動時間、分頁開啟時間、報表渲染時間、RAM 峰值、COM 呼叫延遲 | PowerShell 計時 + `Get-Process FlapsErp` 採樣 + ETW（Windows Performance Recorder） | 產出 `docs/research/efficiency-baseline-YYYYMMDD.md`，含 5 欄基線表 |
| P1 | config.ini 環境調優：`DISABLE_HARDWARE_ACCELERATION` A/B 實測（部分內顯環境開啟反而更穩）、關閉非必要 readonly 分頁、`FORCE_DEVICE_SCALE_FACTOR` 高 DPI 環境確認 | 編輯 `C:\SYSTEM_PROGRAM_DIRECTORY\.conf\config.ini`（先備份 + 記錄 SHA256） | A/B 對照數據 ≥ 3 次取中位數 |
| P1 | **COM 批次化包裝**：我方需要大量 COM 互動時，不在渲染進程逐筆 sendSync；改由 Wellfit 側 Rust COM bridge 以單一處理序批次執行 | `platform/src/com/bridge.rs`（重構為 FLAPSConsoleCOM 目標）+ 物件池 pool.rs | PoC：1000 次呼叫延遲對比（sendSync 逐筆 vs Rust 批次） |
| P1 | 報表/大檔避免客端開啟：導出為檔後交 Wellfit 解析（8 欄契約、零 OCR） | 既有 wellfit_reports 管線 | 月結對帳自動化 KPI（藍圖 100% reconciliation）|
| P2 | 靜態資源快取與預熱：固定 AP 伺服器、排程 TestConnection 預檢（netUtil 已有 ConnectionTester） | flapserp:// + 排程工作 | 斷線預警 < 60s |

**量化目標**（建議，接入 MOP §15 八效指標）：
- 啟動→可操作時間：基線建立後 -30%（P1 後）
- 大批次（≥500 筆）匯入處理：Wellfit 批次 vs ERP 逐筆人工作業，工時 -80%（藍圖 KPI 對齊）
- UI 凍結事件（>1s）：量測後 -90%（COM 批次化後）

---

## 4. 流程自動化方案（含 Layer 3 重大修正）

### 4.1 修正決策： FlaUI/UIA → CDP/RDA/COM 三軌

| 原藍圖 | 逆向事實 | 修正後方案 |
|---|---|---|
| Layer 3 以 FlaUI/UIA 自動化「ERP 桌面 UI」 | ERP UI 是 **web 內容**（Chromium webview），UIA 樹僅能觸及 Chromium accessibility 橋，脆弱且低階 ✔ | **主軸改三軌**：<br>①**RDA 直連**（伺服器級，最穩，需原廠）<br>②**CDP/DOM**（`remote-debugging-port` 或 Playwright connect；外殼自家用 Playwright 測試佐證可行性 △）<br>③**COM/協定檔**（flapserp:// 啟動 + FLAPSConsoleCOM 事件） |

### 4.2 自動化分級（依穩定性排序）

| 級 | 通道 | 適用場景 | 前置 | 風險 |
|---|---|---|---|---|
| A1 | 檔案交換 + flapserp:// 啟動 | 匯入批次、開啟指定作業頁、匯出報表後取檔 | 無（官方入口）✔ | 低 |
| A2 | COM（winax/FLAPSConsoleCOM） | 需要處理序內互動（POS、報表檢視控制） | Rust COM bridge PoC；⚠ EULA/授權確認 | 中（原廠更新元件） |
| A3 | CDP/DOM | 無 API 的畫面流程（表單填寫、簽核） | 遠端除錯埠取得方式 ⚠ 需與原廠確認；loadSaveForm.js 顯示官方自己也做表單自動化 ✔ | 中 |
| A4 | RDAPortal RDA 直連 | 大量資料讀寫（訂單、庫存、傳票） | **OQ 登記：向原廠索取 RDA API 規格/白名單**（TGetModuleInfo 已證實協定形狀） | 中（未文件化 △） |

### 4.3 立即可執行（本週）

1. **重寫 2026-11-10 偵察 session 議程**（原 60-90min FlaUI 偵察改為）：
   - 15min：確認目標作業的「有無檔案/報表匯出」路徑（A1 最優先）
   - 20min：CDP 可行性（外殼是否可帶 `--remote-debugging-port`；Electron 預設支援 ⚠）
   - 20min：COM 物件列舉（FLAPSConsoleCOM.ProgID 清單、Module\*.jsc 方法面）
   - 20min：RDAPortal 探測（僅 TGetModuleInfo 合法呼叫，記錄回應形狀）
   - 15min：產出 ADR-003 草案（Layer 3 修正決策）
2. 登記 **OQ-F1：向輔翼索取 RDA API 規格與自動化政策**（EULA 允許範圍 ⚠）。

---

## 5. 內外部系統資料串接（最佳效率方案）

### 5.1 目標拓撲

```
[蝦皮/MOMO/官網 CSV] ─→ Wellfit(wellfit_shopee) ─→ 8欄匯入CSV ─→ 輔翼ERP(A1檔案通道)
[MTRIX 五檔]        ─→ Wellfit(wellfit_mtrix) ─→ 驗證/修復/彙總 ─→ BI/對帳
[輔翼ERP 報表匯出]  ─→ Wellfit(wellfit_reports) ─→ 零OCR解析+發布閘門 ─→ 總額核對(VR-12)
[E.Sun BankPay]     ─→ Wellfit(wellfit_bankpay) ─→ 憑單媒體/API ─→ ERP 應付憑單
[POS/BHT/iPOS]      ─→ FlapsErp(COM/serialport，原廠通道，不重造)
[RDAPortal RDA]     ─→ (A4，待原廠) ─→ 直接讀寫，取代畫面操作
```

### 5.2 串接原則

1. **每條通道單一契約檔**（ColumnContract 名稱解析、必填硬失敗、禁序位退讓）— 已在 Wellfit 落地 ✔
2. **金額一律整數分**（ScaledDecimal，禁浮點）✔
3. **冪等**：批次 ID + 內容雜湊 + 憑單鎖（`VOUCHER_NO:AMOUNT:BATCH_DATE`）✔
4. **兩向門**：verify_payload → dispatch 單向門，DispatchUncertain 對帳 ✔
5. **禁止序位退讓與靜默改寫**（ADR-0005、ANOM 規則）✔

### 5.3 缺口（誠實標記）

| 缺口 | 狀態 | 解除條件 |
|---|---|---|
| ERP 匯入範本/欄位字典（OI-006） | open | 向輔翼取得正式範本（OQ-F2，與 OQ-F1 同批提出） |
| ERP 管理報表樣本（VR-12 總額核對） | open | 取得報表樣本 + 欄位定義 |
| RDA 業務 API 可用性 | ⚠ 未驗證 | 原廠確認（OQ-F1） |

---

## 6. 大量資料批次處理（最佳效率方案）

| 層 | 方案 | 現況 |
|---|---|---|
| 讀取 | 上限保護讀（MAX_INPUT_BYTES capped read）+ 編碼自動偵測（BOM/CP950/Big5/UTF-8） | bankpay/encoding 已實作 ✔ |
| 驗證 | 逐列名稱解析契約 + 錯誤列隔離（quarantine）+ Finding 稽軌 | 全 crates 一致 ✔ |
| 修復 | 編號化規則（ANOM-01..12）+ 保留原值出 Warning | mtrix ✔ |
| 冪等 | SQLite 帳本：批次註冊（內容雜湊去重）+ 順序鎖 + bulk 鎖定（保序、失敗回滾） | wellfit_core ledger ✔（76 tests） |
| 擴載 | 合成容量基準（synthetic_capacity.rs，manual ignored） | 已入庫 ✔（本週 commit 145865f） |
| 編排 | Windmill + PowerShell 原生二進位（**無 Docker**，corrections.md 裁定） | 排程接入待做 |
| 出口 | Post/Redirect/Get 式檔案落地 + flapserp:// 帶參開頁供人工確認 | A1 通道 ✔ |

**容量目標**（建議，待基準實測）：單批次 ≥ 10,000 列、< 60s 驗證+修復；日處理 ≥ 200 筆付款/10 萬列報表（藍圖 KPI-量測後定案）。

---

## 7. 應開啟/建構的系統、組件與架構

### 7.1 建構清單（新組件）

| ID | 組件 | 內容 | 依賴 | 優先 |
|---|---|---|---|---|
| C-01 | `flaps-bridge`（Rust crate） | COM client 指向 FLAPSConsoleCOM.*；批次呼叫器；物件池；重構 platform/src/com/{bridge,pool,batch,async_ipc}.rs 孤兒碼為正式模組 | windows-rs；EULA 確認 | P1 |
| C-02 | `cdp-pilot`（腳本/Node 或 Rust） | 透過 CDP 驅動外殼分頁：登入表單（參考 loadSaveForm.js 模式）、匯出觸發、下載收尾 | 遠端除錯埠 ⚠ | P2 |
| C-03 | 效能量測套件（PowerShell + ETW） | §3 B 線 P0 基線腳本 | 無 | **P0** |
| C-04 | config 快照/備份工具 | .conf + Module .ver + AP json（遮蔽）快照，SHA256 manifest | 無 | P1 |
| C-05 | OQ-F1/F2 提案包 | 對輔翼的正式需求：RDA 規格、匯入範本、報表樣本、64-bit、憑證設定 | 本報告 | **P0** |
| C-06 | 零 JS 報表看板（藍圖 H1 試點） | Wellfit 解析結果 → PostgreSQL → Askama 純 HTML/CSS 看板（Strict Zero-JS First 決策） | Wellfit+PG18 | P2 |

### 7.2 開啟/配置現有系統

| 項目 | 動作 |
|---|---|
| config.ini | 快照後 A/B 調優（§3 B 線 P1）；**勿**長期維持 IGNORE_CERT_ERR=1 — 對我方 AP 連線要求原廠提供憑證正規化解法 |
| flapserp:// | 註冊為我方排程器出口（Windmill → flapserp:// 啟動指定作業）|
| Playwright | 以「外殼測試」思維建立我方 E2E（先驗證 CDP 可連性）|
| Wellfit | 作為預設批次引擎接入日常（§6）|

---

## 8. 重構、優化與創新建議

### 8.1 我方重構（Wellfit/Dev 工作區）

| ID | 重構 | 動機 |
|---|---|---|
| R-01 | platform/src/com 孤兒碼（bridge/pool/batch/async_ipc）→ 重生為 C-01 flaps-bridge | 逆向證實 COM 通道為官方機制；原孤兒碼方向正確但目標不明 |
| R-02 | platform/src/db/{sqlite,redis,migration}.rs、search/tantivy.rs、sync/* → 依 MOP §10.1 幽靈產物規則處置（刪或入 pending） | 避免虛假資產 |
| R-03 | core ledger engine SQL 注入修補（status 參數化）+ post_journal_entry 交易原子性 | 既知缺陷（先前的 review 記錄）|
| R-04 | root workspace clippy 18 錯/fmt 192 → 修至零 | MOP O3 |

### 8.2 對原廠優化建議（轉交用，§3 A 線彙總）

64-bit；非同步 COM IPC；憑證模式重構；OpenSSL/Vue/Chromium 升級；死碼清理；IPC log 節流；RDA API 文件化 + 自動化政策公開。

### 8.3 創新設計（差異化機會）

1. **「原廠通道 + 我方管線」雙軌模式**：別人做 RPA 都在模擬點擊；我方走官方入口（flapserp:// + COM + RDA）+ 零 OCR 契約管線 → 穩定性與可維護性代差（藍圖 H1/H3 假設的驗證路徑）。
2. **AI 裁決（ekpi rule-based adjudication）接在 Wellfit 隔離列之後**：隔離列 → 規則裁決 → 低信心才人工 → 裁決結果回寫 ANOM 規則庫（閉環學習，純本地零成本）。
3. **月結自動化 KPI 儀表**：Wellfit 產出對帳證據鏈（claims-vs-evidence 式）→ 零 JS HTML 報表給管理層（效度可審計）。

---

## 9. 與既有準備之銜接

| 既有資產 | 本分析帶來的更新 |
|---|---|
| 藍圖 V1.0 KPI（0% 錯漏、≥95% 零OCR、100% 對帳、≥80% 省工） | 維持；Layer 3 手段修正（§4） |
| RPA-ECOM v2.1（Gate 1 已過、ADR-001/002 草案） | 新增 **ADR-003（Layer 3 技術選型修正：FlaUI/UIA → RDA/CDP/COM 三軌）** 草案待簽；M1 偵察議程更新 |
| Wellfit 7-crate | 批次引擎角色確立；flaps-bridge 新模組排入 |
| 知識庫 LOCAL-018/OI-006 | **部分關閉**：開發語言/UI 框架/報表引擎已答（§1.4）；資料庫種類仍 △；建議 source-register 新增 E-01~E-07 逆向證據條目（附 SHA256） |
| 2026-11-10 偵察 session | 議程重寫（§4.3），時間不變 |

---

## 10. 執行路線圖（2 週衝刺 + 後續）

| 週 | 行動 | 產出 |
|---|---|---|
| W1（本週） | C-03 效能基線量測；C-04 config 快照；C-05 OQ-F1/F2 提案包送出；R-03/R-04 修補 | baseline 報告、快照、提案包、測試全綠 |
| W2（11/10） | 偵察 session（新議程 §4.3）→ ADR-003 | ADR-003 草案 + 通道可行性結論 |
| W3 | C-01 flaps-bridge PoC（COM 批次 1000 呼叫對比） | PoC 數據 + Go/No-Go |
| W4 | A1 通道端到端：Windmill 排程 → Wellfit 批次 → flapserp:// 開頁人工確認 → 匯入回執 | 月結對帳試行（1 個真實月結） |
| W5+ | 依原廠回覆推進 A4（RDA）與 C-06 看板 | 逐 KPI 驗收 |

---

## 11. 風險與合規

| 風險 | 等級 | 緩解 |
|---|---|---|
| 原廠元件/協定變動（Winax.node 2026-11-03 剛更新 — 原廠活躍開發中 ✔） | 中 | flaps-bridge 以 .ver 偵測 + 契約測試；變動即告警 |
| EULA/授權：自動化邊界未明 | 中 | OQ-F1 正式取得書面政策前，僅用官方入口（flapserp://、檔案交換）；CDP/RDA 先 PoC 不上生產 |
| 同步 IPC/32-bit 為原廠程式，我方無法根治 | 高 | A 線需求 + B 線外移重活（雙軌並行）|
| 憑證忽略（S-1）在客戶環境的資安責任 | 高 | 提案包要求正式解法；過渡期以網路層補償（內網隔離/白名單）⚠ 需 IT 評估 |

---

## 12. 修訂紀錄

| 版本 | 日期 | 變更 | 證據 |
|---|---|---|---|
| v1.0 | 2026-11-03 | 初版：完整逆向（E-01~E-07）+ 網路查證（E-08）+ 知識庫核對（E-09）；10 瓶頸、4 通道分級、6 新組件、雙軌優化路線 | 本文件 §0 |

*本報告所有 ✔ 事實均可在指定位址重現；△/⚠ 項目已標明驗證路徑。不得將 △/⚠ 項目對外宣稱為既成事實。*
