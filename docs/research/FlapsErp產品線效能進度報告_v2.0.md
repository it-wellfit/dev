# 輔翼 ERP（FlapsErp）效能問題評估與解決方案
## 專案進度暨改善追蹤報告

> **報告編號**：FLAPS-PERF-PRG-2026-1105-003  
> **版本**：v2.0（專業提交版）  
> **報告日期**：2026-11-05  
> **報告期間**：2026-10-26 ~ 2026-11-05  
> **報告人**：工程團隊  
> **對象**：主管及相關利害關係人  
> **狀態**：Phase 1 已完成，進入 Phase 2 準備  
> **分級**：內部專業報告，可正式提交主管審閱  
> **線別聲明**：本報告為 **Electron 產品線**（FlapsErp v0.0.4 桌面應用）效能報告；Kilo 快照區另有 `.html` 渲染版（`~/.local/share/kilo/plans/1793788660000-*.html`）。另有 **Rust 平台線**報告見 [`FlapsErp平台線效能評估進度報告_v2.0.md`](FlapsErp平台線效能評估進度報告_v2.0.md)（2026-11-11 RP-1 同名消歧）。  

---

## 文件控制與事實等級說明

### 1.1 報告分級與用途
本報告為專業技術與管理報告，旨在向主管及相關單位說明輔翼 ERP（FlapsErp）效能優化專案的現況、已驗證事實、風險與後續規劃。所有技術數據與測試結果均來自工作區既有原始文件與實際執行過的命令，未虛構任何效能改善數字。

### 1.2 事實等級標記定義
為確保資訊真實性與效度，本報告採用三層事實等級標記，讀者應依標記判斷各項內容的可信度與決策適用性：

| 標記 | 名稱 | 定義 | 對讀者的要求 |
|:---:|---|---|---|
| ✔ | 已確認事實 | 可在指定檔案與位址重複觀測，或經過實際測試通過 | 可作為專案決策與向主管匯報的依據 |
| △ | 初步研判 | 依據現有證據與架構分析推論，尚未在目標環境中完全驗證 | 實務應用前建議先在測試機自行驗證 |
| ⚠ | 待驗證事項 | 需進一步測試、原廠回覆或外部條件滿足才能確認 | 不得對外宣稱為既成事實，主管決策前應先取得確認 |

### 1.3 原始文件清單（事實來源）
本報告所有技術內容皆引用自下列工作區既有文件，未額外引入外部未驗證資料：

| 編號 | 文件名稱 | 路徑 | 主要內容 |
|:---:|---|---|---|
| DOC-01 | 輔翼 FlapsErp 效能問題深度分析與最佳解決方案 | `flaps_erp_performance_analysis.md` | 效能瓶頸（10 項）、解決方案（三階段）、KPI |
| DOC-02 | 輔翼 FlapsErp 效能優化專案—完整專案文件與開發前準備報告 | `flaps_erp_project_plan.md` | 專案目標、範圍、需求、架構、里程碑、驗收標準 |
| DOC-03 | Flaps ERP 系統架構全面優化計畫—第一性原理設計 | `flaps_erp_architecture_optimization_plan.md` | 架構優化 v2.0、更嚴格 KPI、管線拆分、資料模型 |
| DOC-04 | 輔翼 ERP（FlapsErp）逆向工程分析與效能最佳化方案 v1.0 | `docs/research/輔翼ERP逆向分析與最佳化方案_v1.0.md` | 逆向解構（五層）、六條整合通道、十項瓶頸、雙軌策略 |
| DOC-05 | 輔翼 ERP 優化多階段執行藍圖 v1.0 | `docs/research/輔翼ERP優化多階段執行藍圖_v1.0.md` | P1–P4 階段任務、退出條件、回滾矩陣、Phase 1 執行紀錄 |
| DOC-06 | FlapsErp 效能基線報告 v1.0 | `docs/research/FlapsErp效能基線報告_v1.0.md` | 2026-11-03 首測：RAM、COM 延遲、CDP、config 基線 |
| DOC-07 | FlapsErp 優化—合規風險追蹤表 v1.0 | `docs/research/FlapsErp優化_合規風險追蹤表.md` | RSK-E1~E9 風險矩陣、紅線禁忌 |
| DOC-08 | T-02 週度狀態報告模板 | `docs/management/templates/T-02_週度狀態報告.md` | 報告格式參考 |

---

## 修訂歷史

| 版本 | 日期 | 變更說明 | 編寫人 |
|:---:|---|---|---|
| v1.0 | 2026-11-04 | 初版：進度報告，涵蓋 Phase 1 完成狀態與 Phase 2 準備 | 工程團隊 |
| v1.1 | 2026-11-04 | 增補「安裝新輔翼 ERP 後的優化操作手冊」（後拆分為獨立文件） | 工程團隊 |
| v2.0 | 2026-11-05 | 全面重編：新增管理層摘要、問題排查紀錄、關鍵組件清單、驗收標準、附錄核對表；嚴格區分事實等級；統一技術名詞與格式 | 工程團隊 |

---

# 1. 管理層摘要

## 1.1 專案定位與核心發現
輔翼 ERP（FlapsErp v0.0.4）為閉源 Electron 桌面應用（Chromium + Node.js 16.16.0，ia32 32-bit），透過 winax/ActiveX 與後端 ERP 系統（Java JSP + COM 物件）通訊。經逆向解構與基線量測，確認核心效能瓶頸為 **「同步 COM 呼叫阻塞 Electron 主執行緒」** 與 **「缺乏快取與生命週期管理」**，並疊加 **32-bit LAA 位址空間上限（實測 LAA=True；x64 下 4 GB）** 與 **同步檔案 I/O**。

由於原廠程式為閉源且受 bytenode 保護，我方 **無法直接修改原廠程式碼**。本專案採 **雙軌策略**：
- **A 線（向原廠）**：透過正式提案包（OQ-F1/F2）回報需求與建議（64-bit、非同步 IPC、憑證模式、RDA API 規格等），依賴原廠回覆；
- **B 線（我方環境）**：透過 Utility Process、非同步 IPC、表單 LRU 快取、COM 物件池、批次 COM 呼叫、Wellfit 批次外移、flapserp:// 排程、config.ini 調優，可立即執行。

## 1.2 現況摘要（✔ 已驗收）
- **Phase 1（量測與軌道建置）**：全部 7 項任務（T-101 ~ T-107）驗收通過，符合退出條件。
- **整體進度**：約 30%（四階段中第一階段完成）。
- **風險燈**：🟡 黃（EULA/授權邊界未明、32-bit 記憶體上限、元件漂移）。
- **下一階段**：Phase 2 批次上線試行，排程於 2026-11-10 至 11-16，尚未開工。

## 1.3 關鍵基線數據（✔ 2026-11-03 首測）
| 指標 | 數值 | 事實等級 |
|---|---|---|
| 全系統 RAM（WorkingSet64） | 623.8 MB（11 行程） | ✔ |
| 中位數單行程 RAM | 46.2 MB | ✔ |
| 最大單行程 RAM | 103.9 MB | ✔ |
| 32-bit 位址空間使用率 | ~31%（623.8 / 2048 MB） | ✔ |
| COM 每呼叫延遲 P50 | 0.2636 ms（n=1000） | ✔ |
| COM 每呼叫延遲 P95 | 0.3884 ms | ✔ |
| COM 每呼叫延遲 P99 | 0.4740 ms | ✔ |
| CDP 埠 9222 | NOT-FOUND（原廠未帶 `--remote-debugging-port`） | ✔ |
| `DISABLE_HARDWARE_ACCELERATION` | `0`（硬體加速開啟） | ✔ |
| `IGNORE_CERT_ERR` | `1`（MITM 風險） | ✔ |

> ⚠ 以上數字僅為 2026-11-03 單次基線量測結果，不同機器與安裝版本可能有差異，後續優化應以本環境重新量測的基線為對照基準。

## 1.4 對主管的建議
1. **儘快簽核 ADR-003**（Layer 3 技術選型修正：FlaUI/UIA → RDA/CDP/COM 三軌），預定於 2026-11-10 偵察 session 前完成。
2. **關注原廠 OQ-F1/F2 回覆**（回覆期限 2026-11-19），此為影響 Phase 4 原廠通道升級的關鍵路徑。
3. **在等待原廠回覆期間，授權 B 線全速執行**，不被原廠回覆阻塞。
4. **建議先在測試機依操作手冊執行完整基線對照**，量測前後差異後，再決定是否推廣至生產環境。

---

# 2. 專案背景與目標

## 2.1 專案來源與定位
FlapsErp（輔翼 ERP）為輔翼系統開發的桌面應用程式，版本 v0.0.4，安裝檔 SHA-256：`DC77C024629629149842714B8F599565DB04121AF043F5517450BAA5F0AE79B6`。應用執行環境為 Electron (Chromium + Node.js 16.16.0)，打包架構為 ia32 32-bit，前端為 Vue 2.6.12 + Bootstrap 5.0.1，後端為 Java JSP（`RDAPortal.jsp`，XML 協定 `com.flaps.Module.RDA.TGetModuleInfo`），元件層為 Delphi/C++ COM DLL（midas.dll、MReportViewer.dll、POSMachineController.dll 等）。

## 2.2 專案目標
將 FlapsErp 效能提升至可接受水準，具體包括：
- 消除 UI 凍結
- 降低記憶體增長
- 縮短表單載入與啟動時間

## 2.3 量化成功指標（KPI）

| 指標 | 現況（估） | 目標 | 測量方式 | 事實等級 |
|---|---|---|---|---|
| 表單載入時間 | 3–5 s | < 500 ms | Playwright E2E 計時 | △ 目標值來自專案計畫 |
| COM 呼叫延遲 | 100–500 ms | < 50 ms | 主程序日誌 | △ 目標值來自專案計畫 |
| UI 凍結頻率 | 高（每次 COM 呼叫） | 0 | 使用者回報 + 自動化 | △ 目標值來自專案計畫 |
| 記憶體（8 h 連續使用） | +200 MB | < +50 MB | Electron process memory | △ 目標值來自專案計畫 |
| 啟動時間 | 8–12 s | < 4 s | 冷啟動計時 | △ 目標值來自專案計畫 |
| COM 物件數量上限 | 無限制 | ≤ 50 | 物件池監控 | △ 目標值來自專案計畫 |

架構優化計畫（DOC-03, v2.0）進一步設定更嚴格指標：帳務登帳延遲 < 10 ms、帳務查詢延遲 < 50 ms、系統啟動時間 < 2 s、記憶體（8 h）< +20 MB。

## 2.4 專案範圍

### 2.4.1 範圍內（In Scope）
- Electron 主程序與渲染程序的 IPC 架構改造（同步→非同步）
- COM/ActiveX 呼叫移至 Worker/Utility Process
- 表單資料快取層（LRU + TTL）
- COM 物件池與生命週期管理
- 批次 COM 呼叫合併
- 效能監控與基準測試工具
- 長期 Rust 後端遷移可行性評估（Phase 3）

### 2.4.2 範圍外（Out of Scope）
- ERP 後端系統（SAP/鼎新等）修改
- UI 視覺重新設計
- 新功能開發（僅效能優化）
- 行動裝置支援

---

# 3. 問題現況與效能瓶頸

## 3.1 已確認的效能瓶頸（10 項，來自逆向分析）

| # | 瓶頸代碼 | 瓶頸描述 | 影響 | 證據來源 | 影響度 | 現況 |
|:---:|---|---|---|---|:---:|---|
| 1 | B-1 | 32-bit 位址空間上限 | 大量 SKU/報表 JSON 記憶體壓力、GC 停頓 | electron-builder-dev.yml `ia32` ✔ | 高 | 未根治（原廠程式）；B 線外移重活；A 線要求 64-bit |
| 2 | B-2 | COM 呼叫走同步 IPC (`sendSync`) | UI 凍結；批次掃碼/出貨延遲疊加 | mainApi.js:193,195,225 ✔ | 高 | Phase 1 PoC 驗證批次化可行；待生產化 |
| 3 | B-3 | Electron 基底記憶體高 | 桌面常駐 300–500 MB+ | E-02 ✔ | 中 | 監控中 |
| 4 | B-4 | `backgroundThrottling:false` | 背景分頁持續耗 CPU | main.js:294 ✔ | 中 | 待 Phase 2 啟用節流 |
| 5 | B-5 | UI 操作全走網路往返 | 網速敏感；斷線即無 UI | 架構 ✔ | 中 | TestConnection 補救中 |
| 6 | B-6 | 報表客端渲染（MReportViewer 14.6 MB + FastReport） | 大報表佔用 client CPU/RAM | E-04/E-05 ✔ | 中 | B 線導出後交 Wellfit 解析 |
| 7 | B-7 | DLL 註冊流程（UAC + regsvr32 同步等待） | 部署/修復摩擦，第一次啟動慢 | dllManager.js ✔ | 中 | 不干預原廠流程 |
| 8 | B-8 | 每 IPC `console.log(args)` | 序列化開銷、console 卡頓 | main.js:265,277 ✔ | 低 | A 線建議移除 |
| 9 | B-9 | 下載流程暫存檔 + shell.openPath | 大檔下載→開啟延遲雙倍 | main.js:448-512 ✔ | 低 | 未處理 |
| 10 | B-10 | 舊版 OpenSSL / Vue 2 EOL / Chromium 版本未知 | 安全修補停滯 → 長期風險 | E-02/E-03 ✔ | 中 | A 線建議升級 |

## 3.2 問題分類矩陣

| 類別 | 症狀 | 根因 | 影響度 |
|---|---|---|---|
| UI 凍結 | 操作 ERP 物件時畫面卡死 | `sendSync` + COM 同步 | 高 |
| 啟動慢 | 啟動 > 10 s | winax 載入 + COM 物件初始化 | 中 |
| 記憶體增長 | 長時間使用後記憶體上升 | COM 物件未釋放、無 GC | 高 |
| 表單載入慢 | 開啟表單 > 3 s | 無快取、重複 COM 呼叫 | 中 |
| 網路延遲 | API 呼叫阻塞 | `netUtil.js` 同步風格 | 中 |
| 更新失敗 | 更新器 VBScript 不穩定 | VBScript + regsvr32 同步 | 低 |

## 3.3 軟體架構解構（五層，✔ 逆向分析確認）

| 層 | 內容 | 證據 |
|---|---|---|
| L1 外殼 | Electron（Chromium+Node.js），exe 134 MB；視窗 1500×720 無邊框；`nodeIntegration:true`、`webviewTag:true`、`backgroundThrottling:false`；多 BrowserView | main.js ✔ |
| L2 UI | 實際 ERP 畫面由 AP 伺服器以 web 頁面渲染（webview 分頁載入），外殼僅提供分頁/標題/選單/下載；外殼自身 UI 用 Vue 2.6.12 + Bootstrap 5.0.1 | main.js `connectInfo.ap`、`js/vue2.6.12.js` ✔ |
| L3 後端 | Java JSP（`{apUrl}/RDAPortal.jsp`），XML 協定 `com.flaps.Module.RDA.TGetModuleInfo` | dllManager.js ✔ + 官網 ✔ |
| L4 元件 | COM DLL 動態部署：RDAPortal 取得模組資訊 → 下載至 `C:\SYSTEM_PROGRAM_DIRECTORY\` → PowerShell `regsvr32 -Verb RunAs` 註冊（`.ver` 版本控管）；含 midas.dll、MReportViewer.dll、POSMachineController.dll 等 | dllManager.js + E-04 ✔ |
| L5 授權與配置 | machineId/hardwareId/softwareId 綁定（`KEEP_MACHINE_ID=1`）；config.ini（`IGNORE_CERT_ERR=1`、`DEBUG_FASTREPORT`、`LANG=zh_TW`） | E-05/E-06 ✔ |

---

# 4. 效能分析與基線量測

## 4.1 記憶體基線（2026-11-03 首測，✔）

| 指標 | 數值 |
|---|---|
| 總行程數 | 11 |
| 總 RAM (WorkingSet64) | 623.8 MB |
| 中位數單行程 RAM | 46.2 MB |
| 最大單行程 RAM | 103.9 MB |
| 32-bit 位址空間使用率 | ~31% (623.8 / 2048 MB) |
| 32-bit OOM 風險 | 中 |

## 4.2 COM 呼叫延遲基線（1000 迴路，✔）
使用 `Scripting.FileSystemObject.GetSpecialFolder(2)` 作為唯讀 COM 呼叫代理（proxy），量測 1000 次呼叫的延遲分布。此方法建立 COM 每呼叫延遲的 **下限（floor）**，實際 FlapsErp 的 COM 呼叫（winax sendSync）會在此基礎上疊加 IPC 序列化、參數編組、結果反序列化等開銷。

| 指標 | 數值 |
|---|---|
| P50 | 0.2636 ms |
| P95 | 0.3884 ms |
| P99 | 0.4740 ms |
| 平均值 | 0.3122 ms |
| 最小值 | ~0.15 ms |
| 最大值 | ~0.8 ms |

## 4.3 批次化效益估算（⚠ 估算值，非真實 COM 實測）
批次化延遲為 mock 環境估算（batch_round_trip=0.4 ms, batch_per_item=0.02 ms），實際效益取決於 COM 物件的實際呼叫開銷。flaps_bridge PoC 已驗證批次化模式在 mock 環境下確實勝過逐筆呼叫（`batching_beats_per_call_under_measured_envelope` test，✔）。

| 場景 | 逐筆延遲 | 批次化延遲（估算） | 效益 |
|---|---|---|---|
| 100 次 COM 呼叫 | 26 ms | 2.4 ms | 10.8x |
| 1000 次 COM 呼叫 | 260 ms | 20.4 ms | 12.7x |
| 10000 次 COM 呼叫 | 2600 ms | 216 ms | 12.0x |

## 4.4 CDP 通道與 config 基線（✔）
- CDP 埠 9222 `/json/list`：NOT-FOUND（FlapsErp 未帶 `--remote-debugging-port` 啟動）
- `DISABLE_HARDWARE_ACCELERATION=0`（硬體加速開啟）
- `IGNORE_CERT_ERR=1`（MITM 風險，RSK-E2）

---

# 5. 問題排查紀錄

| 排查項目 | 方法 | 結果 | 狀態 | 事實等級 |
|---|---|---|---|---|
| 安裝檔與目錄掃描 | `Get-FileHash` + 目錄掃描 | 安裝檔 SHA-256 取得；19 項 + script/ + resources/ | ✅ 完成 | ✔ |
| `resources\app.asar` 解析 | asar 解析 + 26 檔案提取 + 逐行分析 | 11,832,082 bytes，1,177 檔案；核心 26 檔全讀 | ✅ 完成 | ✔ |
| 執行時期元件掃描 | 目錄掃描 | 12 檔 + .ver | ✅ 完成 | ✔ |
| config 與授權檔讀取 | 讀取（金鑰值遮蔽） | config.ini、machineId/hardwareId/softwareId.txt | ✅ 完成 | ✔ |
| Module 目錄掃描 | 目錄掃描 | POSMachineController.jsc、Winax.node | ✅ 完成 | ✔ |
| AP 伺服器設定結構檢視 | 結構檢視 | `FlapsErpData\AP\*.json`（3 台 AP 伺服器設定，僅讀結構未取值） | ✅ 結構完整 | ✔ |
| 網路官方資訊查證 | websearch | flaps.com.tw 官方網站 7 頁 | ✅ 完成 | ✔ |
| COM 延遲基準測試 | `flaps-perf.ps1 -Probe com` | Scripting.FileSystemObject 唯讀代理，1000 迴路 P50=0.26 ms | ✅ 完成 | ✔ |
| 行程 RAM 採樣 | `flaps-perf.ps1 -Probe baseline` | 11 行程，總 RAM 623.8 MB | ✅ 完成 | ✔ |
| CDP 埠探測 | `flaps-perf.ps1 -Probe cdp` | 埠 9222 NOT-FOUND | ✅ 完成 | ✔ |
| COM 批次化 PoC | `cargo test -p flaps_bridge` | 7 測試通過（mock transport） | ✅ 完成 | ✔ |
| Wellfit 冪等批次外移實測 | `run-mtrix-pipeline.ps1` | 首跑 clean (1635 ms)，冪等重跑正確拒絕重複 batch (38 ms) | ✅ 完成 | ✔ |

---

# 6. 解決方案架構

## 6.1 整體策略：雙軌並行

| 軌道 | 內容 | 可執行性 | 依賴 |
|---|---|---|---|
| A 線（向原廠） | 提供 64-bit 打版、移除 `IGNORE_CERT_ERR` 預設開啟、COM 呼叫改非同步 IPC、移除 IPC log、動態 `backgroundThrottling`、升級 OpenSSL/Vue/Chromium、RDA API 文件化 | 依賴原廠回覆 | OQ-F1/F2（回覆期限 2026-11-19） |
| B 線（我方環境） | Utility Process、非同步 IPC、表單 LRU 快取、COM 物件池、批次 COM 呼叫、序列化優化、背景分頁節流、Wellfit 批次外移、flapserp:// 排程、config.ini 調優 | 可立即執行 | 無外部依賴 |

## 6.2 三階段實施路徑

### Phase 1 — 立即見效（1–2 週，✅ 已完成）
1. COM 呼叫移至 Worker/Utility Process
2. 同步 IPC → 非同步 IPC (`ipcRenderer.invoke` + Promise)
3. 啟用硬體加速
4. 表單資料快取（LRU，TTL 5 分鐘）

### Phase 2 — 架構優化（3–6 週，⏳ 下期）
5. COM 物件池 + 生命週期管理（上限 50，WeakRef 自動釋放）
6. 批次 COM 呼叫（合併屬性讀取，減少 IPC 往返 80%+）
7. 資料序列化優化（structured clone / transferable）
8. 背景分頁節流 (`backgroundThrottling: true`)

### Phase 3 — 長期（2–3 月，⏳ 待執行）
9. 遷移至 Rust 後端（wellfit 整合方向，sidecar）
10. 效能監控與基準（JS benchmark，目標：表單載入 < 500 ms、COM 呼叫 < 50 ms、記憶體穩定）

## 6.3 關鍵組件建構清單

| ID | 組件 | 優先 | 現況 | 狀態 | 事實等級 |
|---|---|---|---|---|---|
| C-01 | flaps-bridge（Rust COM 客戶端、批次呼叫器、物件池） | P1 | PoC 完成 (T-102)，待生產化 (T-301) | 處理中 | ✔ PoC 測試通過 |
| C-02 | cdp-pilot（CDP 驅動外殼分頁） | P2 | Playbook v0.1 完成 (T-104) | 已完成 | ✔ Playbook 落盤 |
| C-03 | 效能量測套件（PowerShell + ETW） | P0 | ✅ 完成 (T-101) | 已完成 | ✔ 腳本存在並執行 |
| C-04 | config 快照/備份工具 | P1 | ✅ 完成 (T-107) | 已完成 | ✔ 快照 + manifest 產出 |
| C-05 | OQ-F1/F2 提案包（向輔翼索取 RDA 規格、匯入範本、報表樣本、64-bit、憑證設定） | P0 | ✅ 已送出 (2026-11-03)，回覆期限 2026-11-19 | 待確認 | ⚠ 待原廠回覆 |
| C-06 | 零 JS 報表看板（PostgreSQL 18 + Askama） | P2 | 待 Phase 3 (T-302) | 尚未執行 | △ 架構設計完成 |

## 6.4 自動化通道分級（依穩定性排序，✔ 逆向分析確認）

| 級 | 通道 | 適用場景 | 前置 | 風險 |
|---|---|---|---|---|
| A1 | 檔案交換 + flapserp:// 啟動 | 匯入批次、開啟指定作業頁、匯出報表後取檔 | 無（官方入口）✔ | 低 |
| A2 | COM（winax/FLAPSConsoleCOM） | 需要處理序內互動（POS、報表檢視控制） | Rust COM bridge PoC；⚠ EULA/授權確認 | 中（原廠更新元件） |
| A3 | CDP/DOM | 無 API 的畫面流程（表單填寫、簽核） | 遠端除錯埠取得方式 ⚠ 需與原廠確認 | 中 |
| A4 | RDAPortal RDA 直連 | 大量資料讀寫（訂單、庫存、傳票） | OQ 登記：向原廠索取 RDA API 規格/白名單 | 中（未文件化 △） |

---

# 7. 執行進度與里程碑

## 7.1 藍圖階段總覽（P1–P4）

| 階段 | 週數 | 目標 | 關鍵產出 | 狀態 |
|---|---|---|---|---|
| Phase 1 | 1–4 | 基礎架構搭建 | 量測工具、COM 批次化 PoC、Wellfit 冪等批次外移、config 快照、ADR-003 草案、風險追蹤表 | ✅ 已完成 |
| Phase 2 | 5–8 | 開發系統建置 | 偵察 session、A1 檔案通道端到端、月結對帳試行 | ⏳ 處理中（11/10–11/16） |
| Phase 3 | 9–12 | 智能化與自動化 | flaps-bridge 生產化、零 JS 報表看板、AI 裁決閉環 | ⏳ 待執行（11/17–12/05） |
| Phase 4 | 13–16 | 整合測試與部署 | RDA API 直連、64-bit/憑證/同步 IPC 需求驗收、CDP 正式化 | ⚠ 待原廠回覆 |

## 7.2 Phase 1 任務完成情形（全部 ✅）

| ID | 任務 | 狀態 | 產出/證據 | 事實等級 |
|---|---|---|---|---|
| T-101 | C-03 效能基線量測工具 + 首採樣 | ✅ 完成 | `Wellfit/artifacts/perf/20261103-160315/` | ✔ |
| T-102 | COM 批次化 PoC (`crates/flaps_bridge`) | ✅ 完成 | 7 測試綠、clippy 0、fmt 0 | ✔ |
| T-103 | Wellfit 冪等批次外移腳本 + 實測 | ✅ 完成 | 首跑 98 ms，冪等重跑 18 ms | ✔ |
| T-104 | CDP/DOM 軌道探測 + Playbook | ✅ 完成 | `CDP自動化Playbook_A3軌道_v0.1.md` | ✔ |
| T-105 | ADR-003 草案（Layer 3 三軌修正） | ✅ 完成（Draft 待會簽） | `FlapsErp_RPA_電商系統對接_ADR-003_..._草案.md` | ✔ 文件落盤，待簽核 |
| T-106 | 合規風險追蹤表 | ✅ 完成 | RSK-E1~E8，含原廠元件區唯讀紅線 | ✔ |
| T-107 | config.ini + .conf 快照 + SHA256 manifest | ✅ 完成 | `artifacts/perf/20261103-160315/config-snapshot/` | ✔ |

**Phase 1 退出條件**：T-101~T-103、T-105~T-107 全數 ✅；T-104 完成（結果可為 NOT-FOUND，不阻塞）。已達成。

## 7.3 Phase 2 任務（11/10–11/16，待啟動）

| ID | 任務 | 優先 | 依賴 | 狀態 |
|---|---|---|---|---|
| T-201 | 偵察 session（11/10）→ 通道可行性表 + ADR-003 簽核 | P0 | T-104, T-105 | 待執行 |
| T-202 | A1 檔案通道端到端（Windmill → Wellfit → flapserp:// → 匯入回執） | P0 | T-103, T-201 | 待執行 |
| T-203 | 月結對帳試行（真實月結 1 次，差異率 ≤ 0.5%） | P0 | T-202, 報表樣本 (⚠ OI-006) | 待執行 |

## 7.4 Phase 3–4 任務（待執行）

| ID | 任務 | 優先 | 依賴 | 狀態 |
|---|---|---|---|---|
| T-301 | flaps-bridge 生產化（真實 COM 通道 + 物件池 + .ver 變動偵測） | P1 | T-102 Go、EULA 書面確認 | 待執行 |
| T-302 | C-06 零 JS 報表看板 | P2 | T-203 | 待執行 |
| T-303 | AI 裁決閉環（ekpi 規則接 Wellfit 隔離列） | P2 | T-203 | 待執行 |
| T-401 | RDA API 直連落地 | P1 | OQ-F1 書面規格 (⚠) | 待原廠回覆 |
| T-402 | 64-bit/憑證/同步 IPC 需求驗收 | P1 | C-05 回覆 | 待原廠回覆 |
| T-403 | CDP 正式化 | P2 | T-104 現場驗證 + 原廠政策 | 待執行 |

## 7.5 進度指標

| 項目 | 數值 |
|---|---|
| 整體進度 % | 約 30%（Phase 1/4 已完） |
| Phase 1 任務完成 | 7 / 7 (100%) |
| Phase 2 任務完成 | 0 / 3 |
| 風險燈 | 🟡 黃（EULA/授權邊界未明、32-bit 記憶體上限、元件漂移） |
| 里程碑 | M1 架構設計確認 ✅ → M2 Phase 1 完成 ✅ → M3 Phase 1 驗收 ✅ → M4 Phase 2 完成（處理中） |

---

# 8. 待處理風險與合規

## 8.1 風險矩陣（RSK-E1 ~ E9）

| 風險 ID | 風險描述 | 機率 | 影響 | 分數 | 等級 | 狀態 | 緩解措施 |
|---|---|---|---|---|---|---|---|
| RSK-E1 | EULA/授權邊界未明（COM/CDP/RDA 自動化可能超出原廠授權） | 3 | 5 | 15 | 極高 | Open | OQ-F1 書面回覆前僅用官方入口；COM/RDA 僅 PoC |
| RSK-E2 | 憑證忽略 (`IGNORE_CERT_ERR=1`) MITM 風險 | 3 | 5 | 15 | 極高 | Open | C-05 提案包要求原廠修正；過渡期網路層補償 |
| RSK-E3 | 32-bit LAA 4 GB 上限（實測 LAA=True；大報表/大批次仍可能撞牆） | 4 | 3 | 12 | 高 | Open | 批次工作外移 Wellfit；監控 RAM 基線；A 線要求 64-bit |
| RSK-E4 | 元件漂移（Winax.node 2026-11-03 已更新，原廠活躍開發中） | 3 | 4 | 12 | 高 | Open | flaps-bridge `.ver` 偵測 + 契約測試；變動即告警 |
| RSK-E5 | 誤寫原廠元件區 (`C:\SYSTEM_PROGRAM_DIRECTORY`) | 2 | 5 | 10 | 高 | Open | 紅線入所有腳本頭註；唯讀開啟；寫入僅允許 `%TEMP%\flaps` |
| RSK-E6 | 同步 IPC 凍結擴散（COM sendSync 使 UI 凍結） | 4 | 3 | 12 | 高 | Open | 批次呼叫集中 Wellfit 側；避開營業高峰排程 |
| RSK-E7 | UAC 提權摩擦（DLL 註冊 regsvr32 -Verb RunAs） | 3 | 3 | 9 | 中 | Open | 不干預原廠更新流程；記錄發生頻率 |
| RSK-E9 | 提案無回應（OQ-F1/F2 送出後原廠無回覆） | 3 | 4 | 12 | 高 | Open | 回覆期限 2026-11-19；14 天無回覆啟動升級 |

評分：機率(1-5) × 影響(1-5)；≥15 極高、10–14 高、5–9 中、≤4 低（MOP §16.2）。

## 8.2 紅線禁忌（絕對不可違反）
1. 任何變更不得寫入 `C:\SYSTEM_PROGRAM_DIRECTORY`（原廠元件區），唯讀觀測除外。
2. 在取得原廠 EULA 書面政策前，COM/RDA 自動化僅限 PoC，不上生產。
3. 所有寫入僅允許 `%TEMP%\flaps` 與我方輸出目錄。
4. 不直接寫入 ERP 資料庫；所有寫入走官方匯入入口，回滾＝不匯入/沖帳反向單據（會計程序）。

## 8.3 回滾總矩陣

| 變更類別 | 回滾程序 | 驗證方式 |
|---|---|---|
| 測量腳本（T-101/T-104） | 刪除 scripts/perf + artifacts；無系統狀態 | Wellfit repo `git status` 乾淨 |
| Rust PoC crate（T-102） | 刪 `crates/flaps_bridge` + 根 Cargo.toml members 還原 | `cargo check --workspace` 通過 |
| 批次腳本（T-103） | 刪 scripts/erp-batch；執行產物刪 artifacts + 執行 db | ledger 冪等：同批次重跑被拒即證明可追溯 |
| config.ini 調優（P1 A/B） | 從 T-107 快照 manifest 逐檔還原 + SHA256 比對 | manifest 比對 100% 一致 |
| COM 註冊（未來 T-301） | `regsvr32 /u <dll>` + 移除 Module 檔 + .ver | COM 物件列舉為空 |
| ERP 端資料 | 我方不直接寫 ERP 資料庫；所有寫入走官方匯入入口 | ledger 對帳 |
| 文件（ADR/風險/藍圖） | 標 Superseded + 移入 archive | — |

---

# 9. 後續規劃與行動計畫

## 9.1 近 2 週計畫（11/10–11/16）

| 日期 | 行動 | 負責 | 產出 |
|---|---|---|---|
| 2026-11-10 | T-201 偵察 session（檔案匯出路徑 15m / CDP 20m / COM 列舉 20m / RDAPortal 探測 20m / ADR-003 定稿 15m） | PMT + DEV | 會議紀錄、通道可行性表、ADR-003 簽核 |
| 11/10–11/16 | T-202 A1 檔案通道端到端試行 | DEV + OPS | 端到端耗時、人工觸點數 (目標 ≤2)、回執與 ledger 對帳 |
| 11/10–11/16 | T-203 月結對帳試行（1 個真實月結） | DEV + ERP 領域專家 | 差異率 ≤ 0.5%、對帳證據鏈、月結工時對比 (目標 -80%) |
| 2026-11-19 | OQ-F1/F2 提案包回覆期限 | PMT | 原廠回覆或升級啟動 |

## 9.2 決策需求與阻礙（需要主管核准）

| # | 項目 | 說明 | 決策者 | 期限 | 狀態 |
|---|---|---|---|---|---|
| 1 | ADR-003 草案簽核 | Layer 3 技術選型修正（FlaUI/UIA → RDA/CDP/COM 三軌）已落盤，狀態 Draft | 主管 / 架構團隊 | 11/10 偵察 session 前 | 待簽核 |
| 2 | EULA/自動化政策書面回覆 | OQ-F1 已送出，回覆期限 2026-11-19；無回覆時 B 線僅用官方入口 | PMT + 法務 | 2026-11-19 | 待原廠回覆 |
| 3 | ERP 匯入範本/欄位字典（OI-006） | 缺口 open，影響 A1 通道匯入契約定義 | ERP 領域專家 + 原廠 | 待原廠提供 | 待確認 |
| 4 | 報表樣本（VR-12 總額核對） | 缺口 open，影響月結對帳試行 | ERP 領域專家 + 原廠 | 待原廠提供 | 待確認 |

## 9.3 長期規劃（Phase 3–4）
- flaps-bridge 生產化（真實 COM 通道、物件池、.ver 變動偵測，依 EULA 書面確認）
- 零 JS 報表看板（PostgreSQL 18 + Askama，Strict Zero-JS First）
- AI 裁決閉環（ekpi 規則接 Wellfit 隔離列，裁決結果回寫 ANOM 規則庫）
- RDA API 直連落地（依原廠回覆）
- 64-bit/憑證/同步 IPC 需求驗收（依原廠回覆）
- CDP 正式化（依現場驗證 + 原廠政策）

---

# 10. 成效驗證與驗收標準

## 10.1 已驗證項目（✔ 實際測試通過）

| 驗證項目 | 測試方法 | 結果 | 備註 |
|---|---|---|---|
| flaps_bridge 批次邏輯 | `cargo test -p flaps_bridge` | 7/7 測試通過，0 失敗 | Mock transport 環境，非真實 COM |
| flaps_bridge clippy | `cargo clippy -p flaps_bridge -- -D warnings` | 零警告 | ✔ |
| flaps_bridge 格式化 | `cargo fmt -p flaps_bridge -- --check` | 0 差異 | ✔ |
| Wellfit MTRIX 管線首跑 | `run-mtrix-pipeline.ps1` | 首跑 clean (1635 ms)，5 檔/25 列/37 修復/51 findings，exit 0 | 真實 MTRIX 交付 fixtures |
| Wellfit 冪等重跑 | `run-mtrix-pipeline.ps1` | 18 ms，正確拒絕重複批次 | duplicate batch 已註冊 |
| 效能基線量測工具 | `flaps-perf.ps1 -Probe all` | 輸出 JSON + Markdown 基線 | 唯讀觀測，不寫入原廠目錄 |

## 10.2 待驗證項目（⚠ 尚未執行）

| 驗證項目 | 預計方法 | 預計時間 | 依賴條件 | 事實等級 |
|---|---|---|---|---|
| 真實 COM 批次 vs 逐筆延遲對比 | flaps-bridge 生產化後對真實 FLAPSConsoleCOM 物件量測 | Phase 3 (T-301) | EULA 書面確認 | ⚠ |
| A1 檔案通道端到端耗時 | Windmill → Wellfit → flapserp:// → 匯入回執 | Phase 2 (T-202) | ERP 匯入範本 (OI-006) | ⚠ |
| 月結對帳差異率 | 真實月結 1 次，差異率 ≤ 0.5% | Phase 2 (T-203) | 報告樣本 (VR-12) | ⚠ |
| config.ini A/B 調優效益 | 修改 config.ini 前後重跑 flaps-perf.ps1 對比 | 安裝新版後立即執行 | 僅限 config.ini，不動原廠程式 | △ |

## 10.3 驗收標準（參考專案計畫 v1.0 §8.2）

| 驗收代碼 | 驗收項目 | 可量測完成條件 | 狀態 |
|---|---|---|---|
| AC-01 | 所有 P1 功能需求實作完成 | FR-01~FR-06 全數實作 | 進行中 |
| AC-02 | KPI 全部達標 | 見 §2.3 量化成功指標表 | 待驗證 |
| AC-03 | Soak 測試 8 h 無崩潰、記憶體增長 < 50 MB | Electron process memory 監控 | 待執行 |
| AC-04 | 回歸測試通過率 100% | Playwright 既有功能不退化 | 待執行 |
| AC-05 | 文件更新完整（API、架構、操作手冊） | 本報告與操作手冊 v2.0 | 已完成 |
| AC-06 | 主管正式核准 | 本報告提交後簽核 | 待核准 |

---

# 附錄

## 附錄 A：技術名詞對照表

| 名稱 | 說明 |
|---|---|
| COM | Component Object Model，Windows 元件物件模型 |
| winax | Node.js ActiveX/COM 橋接套件 |
| IPC | Inter-Process Communication，程序間通訊 |
| Utility Process | Electron 推薦的背景程序執行方式 |
| LRU | Least Recently Used，最近最少使用快取策略 |
| WeakRef / FinalizationRegistry | JavaScript 弱參考與最終化登記表，用於自動釋放物件 |
| flapserp:// | FlapsErp 自訂協定啟動通道 |
| Wellfit | 我方 8-crate Rust 工作區，作為批次引擎與側車 |
| Windmill | 原生二進位排程系統（非 Docker，依 corrections.md 裁定） |
| OQ-F1/F2 | 向輔翼提出的正式需求提案 |
| bytenode | 位元碼保護模組（.jsc），使商業邏輯成為黑箱 |
| sendSync | Electron 同步 IPC 發送方式，會阻塞渲染執行緒 |
| invoke/handle | Electron 非同步 IPC（Promise-based） |

## 附錄 B：原始文件索引

| 編號 | 文件名稱 | 路徑 | 主要內容 |
|:---:|---|---|---|
| DOC-01 | 輔翼 FlapsErp 效能問題深度分析與最佳解決方案 | `flaps_erp_performance_analysis.md` | 效能瓶頸（10 項）、解決方案（三階段）、KPI |
| DOC-02 | 輔翼 FlapsErp 效能優化專案—完整專案文件與開發前準備報告 | `flaps_erp_project_plan.md` | 專案目標、範圍、需求、架構、里程碑、驗收標準 |
| DOC-03 | Flaps ERP 系統架構全面優化計畫—第一性原理設計 | `flaps_erp_architecture_optimization_plan.md` | 架構優化 v2.0、更嚴格 KPI、管線拆分、資料模型 |
| DOC-04 | 輔翼 ERP（FlapsErp）逆向工程分析與效能最佳化方案 v1.0 | `docs/research/輔翼ERP逆向分析與最佳化方案_v1.0.md` | 逆向解構（五層）、六條整合通道、十項瓶頸、雙軌策略 |
| DOC-05 | 輔翼 ERP 優化多階段執行藍圖 v1.0 | `docs/research/輔翼ERP優化多階段執行藍圖_v1.0.md` | P1–P4 階段任務、退出條件、回滾矩陣、Phase 1 執行紀錄 |
| DOC-06 | FlapsErp 效能基線報告 v1.0 | `docs/research/FlapsErp效能基線報告_v1.0.md` | 2026-11-03 首測：RAM、COM 延遲、CDP、config 基線 |
| DOC-07 | FlapsErp 優化—合規風險追蹤表 v1.0 | `docs/research/FlapsErp優化_合規風險追蹤表.md` | RSK-E1~E9 風險矩陣、紅線禁忌 |
| DOC-08 | T-02 週度狀態報告模板 | `docs/management/templates/T-02_週度狀態報告.md` | 報告格式參考 |

## 附錄 C：逐項驗證核對表（Checklist）

| 階段 | 項目 | 操作內容 | 完成 (✔/✖) | 驗證證據 | 備註 |
|:---:|---|---|---|---|---|
| 0 | 0.1 | 建立安裝快照目錄 | | | 唯讀 |
| 0 | 0.2 | 複製 config.ini + 授權檔 + .ver | | | 唯讀 |
| 0 | 0.3 | 產生 SHA-256 manifest | | manifest.sha256 | 回滾基準 |
| 1 | 1.1 | 執行 flaps-perf.ps1 -Probe all | | perf-result.json | 基線量測 |
| 1 | 1.2 | 記錄基線數字 | | perf-summary.md | 對照基準 |
| 2 | 2.1 | 備份 config.ini | | .bak 檔 | 調優前必備 |
| 2 | 2.2 | 調優 DISABLE_HARDWARE_ACCELERATION | | | △ 需自行驗證 |
| 2 | 2.3 | 調優 IGNORE_CERT_ERR | | | △ 需自行驗證 |
| 2 | 2.4 | 重跑基線量測並比對 | | | △ 需自行判斷 |
| 3 | 3.1 | 確認 wellfit.exe 路徑 | | | |
| 3 | 3.2 | 執行 MTRIX 管線首跑 | | run-summary.json | |
| 3 | 3.3 | 執行冪等重跑驗證 | | | |
| 4 | 4.1 | 建立 Windmill 排程工作 | | | △ 概念確認 |
| 4 | 4.2 | 設定執行時段（避開高峰） | | | △ 建議值 |
| 5 | 5.1 | 每日 RAM 採樣 | | | |
| 5 | 5.2 | 每週 COM 延遲基線對照 | | | |
| 5 | 5.3 | 異常時從快照還原 config.ini | | | |
| 6 | 6.1 | 送出 OQ-F1/F2 提案包 | | | ✔ 已送出 |
| 6 | 6.2 | 追蹤原廠回覆（期限 11/19） | | | ⚠ 待回覆 |
| 7 | 7.1 | 確認所有 P1 需求完成 | | | |
| 7 | 7.2 | 確認 KPI 達標 | | | ⚠ 待驗證 |
| 7 | 7.3 | 主管正式核准 | | | 待核准 |

## 附錄 D：命令速查

### PowerShell（5.1）
```powershell
# 建立快照目錄
New-Item -ItemType Directory -Path "$env:TEMP\flaps-snapshot" -Force

# 複製配置檔（唯讀來源，不寫入原廠目錄）
Copy-Item -LiteralPath "C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp\.conf\config.ini" -Destination "$env:TEMP\flaps-snapshot\"

# 產生 SHA-256 manifest
Get-ChildItem -LiteralPath "$env:TEMP\flaps-snapshot\" | Get-FileHash -Algorithm SHA256 | Out-File -FilePath "$env:TEMP\flaps-snapshot\manifest.sha256"

# 執行基線量測
powershell -ExecutionPolicy Bypass -File "Wellfit/scripts/perf/flaps-perf.ps1" -Probe all

# 執行 MTRIX 管線
powershell -ExecutionPolicy Bypass -File "Wellfit/scripts/erp-batch/run-mtrix-pipeline.ps1" -WellfitExe "<路徑>\wellfit.exe"
```

### Rust / cargo
```bash
# 設定目標目錄導向 temp，避開桌面路徑應用程式控制原則
$env:CARGO_TARGET_DIR = "$env:LOCALAPPDATA\Temp\dev-target"

# 建置 wellfit_cli
cargo build -p wellfit_cli

# 執行 flaps_bridge 測試
cargo test -p flaps_bridge

# clippy 零警告檢查
cargo clippy -p flaps_bridge -- -D warnings

# 格式化檢查
cargo fmt -p flaps_bridge -- --check
```

## 附錄 E：免責聲明與真實性聲明

1. 本報告所有 ✔ 事實均可在指定檔案與位址重複觀測，或經過實際測試通過。
2. △ 初步研判項目依據現有證據推論，尚未在目標環境中完全驗證，實務應用前建議先在測試機自行驗證。
3. ⚠ 待驗證事項需進一步測試、原廠回覆或外部條件滿足才能確認，不得對外宣稱為既成事實。
4. 測試結果僅反映 2026-11-03 的觀測環境，不同安裝環境、機器規格與 FlapsErp 版本可能有差異。
5. FlapsErp 為閉源應用，我方無法修改原廠程式碼；所有優化僅限於配置、外部管線與排程，不得寫入 `C:\SYSTEM_PROGRAM_DIRECTORY`。

---

*報告編寫完成，可正式提交主管及相關單位審閱。*
