# 輔翼ERP 優化多階段執行藍圖 v1.0

> 文件編號：RSH-2026-1103-002｜版本：v1.0｜日期：2026-11-03｜狀態：Active（不等原廠回應，我方軌道全速執行）
> 上游依據：`docs/research/輔翼ERP逆向分析與最佳化方案_v1.0.md`（v1.0，SHA256 DA35DB8A…98AB）
> 原則：**每項變更可量測（有基線數字）、可回滾（有還原程序）、可驗收（有明確退出條件）**
> 證據標記：✔ 已觀測｜△ 推導｜⚠ 未驗證（需現場確認）

---

## 0. 階段總覽與依賴圖

```
P1 量測與軌道建置（本週，立即執行）
 ├─ T-101 C-03 效能基線量測工具 + 首測 ──────────┐
 ├─ T-102 COM 批次化 PoC（flaps_bridge）─────────┼─→ P2 批次上線試行
 ├─ T-103 Wellfit 冪等批次外移（腳本+實測）──────┤
 ├─ T-104 CDP/DOM 軌道探測 + Playbook ──────────┘
 ├─ T-105 ADR-003 草案（Layer 3 修正）
 └─ T-106 合規風險追蹤表
P2 批次上線試行（11/10–11/16）
 ├─ T-201 偵察 session（11/10，新議程）→ 通道可行性結論
 ├─ T-202 A1 檔案通道端到端（Windmill 排程 → Wellfit → flapserp:// → 匯入回執）
 └─ T-203 月結對帳試行（1 個真實月結）
P3 橋接與看板（11/17–12/05，依賴 P1/P2 驗收）
 ├─ T-301 flaps-bridge 生產化（依賴 T-102 Go + EULA 書面確認 ⚠）
 ├─ T-302 C-06 零 JS 報表看板（依賴 T-203 數據）
 └─ T-303 AI 裁決閉環（ekpi 規則接 Wellfit 隔離列）
P4 原廠通道升級（依原廠回應，不被阻塞）
 ├─ T-401 RDA API 規格落地（依賴 OQ-F1 回覆 ⚠）
 ├─ T-402 64-bit / 憑證 / 同步IPC 需求驗收（依賴 C-05 提案包回覆 ⚠）
 └─ T-403 CDP 正式化（依賴 T-104 現場驗證 + 原廠政策）
```

---

## 1. 分階段任務清單（含優先序、依賴、驗收、量測、回滾）

### Phase 1 — 量測與軌道建置（本週執行，本藍圖簽發即開工）

| ID | 任務 | 優先 | 依賴 | 驗收準則 | 量測 | 回滾 |
|---|---|---|---|---|---|---|
| T-101 | C-03 效能基線量測工具（flaps-perf.ps1：baseline/com/cdp 三探測器）+ 首次採樣 | **P0** | 無 | 工具 exit 0；產出 JSON+MD 基線於 `Wellfit/artifacts/perf/<ts>/`； FlapsErp 未執行時明確標 `APP-NOT-RUNNING` 並提供排程指引 | 啟動時間、RAM 峰值/趨勢、COM 每呼叫延遲（1000 迴路 P50/P95）、CDP 埠探測結果 | 純讀取式測量，無系統變更；刪除 artifacts 目錄即回滾 |
| T-102 | COM 批次化 PoC：`crates/flaps_bridge`（根工作區，R 型研究 crate） | **P0** | 無 | `cargo test -p flaps_bridge` 全綠（mock 傳輸）；`clippy -p flaps_bridge -- -D warnings` 0 警告；fmt 0 差異；批次 vs 逐筆對比數字產出 | mock 引擎：批次(1000) vs 逐筆(1000) 呼叫耗時對比，寫入 PoC 報告 | 移除 crates/flaps_bridge + 還原根 Cargo.toml members（gitless 工作區：保留 .bak 或刪除即淨） |
| T-103 | Wellfit 冪等批次外移：`scripts/erp-batch/run-mtrix-pipeline.ps1` + 實測 | **P0** | 無（Wellfit 基線已綠） | 腳本以真實交付 fixtures 跑通：exit 0；產出 report.json + ledger + 執行計時；重跑偵測到冪等批次（duplicate content）正確處理 | 真實 MTRIX 五檔處理耗時（ms）、列數、Finding 數 | ledger 為 SQLite 檔，刪除該執行 db + 輸出目錄即回滾；腳本不寫入 ERP 端任何資料 |
| T-104 | CDP/DOM 軌道探測 + 自動化 Playbook 模板 | P1 | T-101 工具 | cdp 探測器完成（埠 9222 /json 探測、未執行時提供帶參啟動指引 ⚠）；Playbook 模板含 5 步驗證流程 | 探測結果（FOUND/NOT-FOUND）| 純讀取；無回滾需求 |
| T-105 | ADR-003 草案（Layer 3：FlaUI/UIA → RDA/CDP/COM 三軌修正） | **P0** | T-101~T-104 初步結果 | 草案落盤（T-04 模板：Options/Decision/Consequences/驗收）；狀態 Draft 待會簽 | — | 文件，標 Superseded 即回滾 |
| T-106 | 合規風險追蹤表（RSK-E1~E4） | **P0** | 無 | 落盤 + 4 風險項（EULA 邊界、憑證忽略、32-bit、元件漂移）各附緩解與觸發條件 | — | 文件 |
| T-107 | config.ini + .conf 快照（含 SHA256 manifest，供 P1 調優前回滾基準） | P1 | 無 | `Wellfit/artifacts/perf/<ts>/config-snapshot/` 含 config.ini、*.txt、.ver 全集合 + manifest | — | 還原 manifest 內檔案即回滾 |

**Phase 1 退出條件**：T-101~T-103、T-105~T-107 全數驗收 ✔；T-104 完成（結果可為 NOT-FOUND，不阻塞）。

### Phase 2 — 批次上線試行（11/10–11/16）

| ID | 任務 | 優先 | 依賴 | 驗收準則 | 量測 | 回滾 |
|---|---|---|---|---|---|---|
| T-201 | 偵察 session（11/10 09:00–10:30，新議程：檔案匯出路徑 15m / CDP 20m / COM 列舉 20m / RDAPortal 探測 20m / ADR-003 定稿 15m） | **P0** | T-104、T-105 | 會議紀錄 + 通道可行性表（A1/A2/A3/A4 各 Go/No-Go/Conditional）+ ADR-003 簽核 | 各通道探測結果 | ADR-003 標 Superseded |
| T-202 | A1 檔案通道端到端：Windmill(原生) 排程 → Wellfit 批次 → flapserp:// 開頁 → 人工確認 → 匯入回執檔回收 | **P0** | T-103、T-201 | 試行 ≥1 條真實資料流全程走通；每步耗時記錄；回執與 ledger 對帳一致 | 端到端耗時、人工觸點數（目標 ≤2） | 停用 Windmill 排程 + 刪 ledger 批次（冪等設計天然支援重跑） |
| T-203 | 月結對帳試行（真實月結 1 次） | **P0** | T-202、報表樣本（⚠ OI-006） | 差異率 ≤ 0.5%（藍圖 KPI）；對帳證據鏈（claims-vs-evidence 式）產出 | 月結工時對比（目標 -80%） | ledger 唯讀保留 + 標記試行批次 |

### Phase 3 — 橋接與看板（11/17–12/05）

| ID | 任務 | 優先 | 依賴 | 驗收準則 | 量測 | 回滾 |
|---|---|---|---|---|---|---|
| T-301 | flaps-bridge 生產化（COM 真實通道 + 物件池 + .ver 變動偵測） | P1 | T-102 Go、EULA 書面確認（RSK-E1 緩解） | clippy/test 全綠；.ver 偵測測試；1000 呼叫批次 PoC 數據於真實 COM 物件複驗 | 真實 COM 批次 vs 逐筆延遲 | feature flag 停用 + 移除註冊 COM（regsvr32 /u 程序） |
| T-302 | C-06 零 JS 報表看板（PostgreSQL 18 + Askama） | P2 | T-203 | Strict Zero-JS（0 JS 位元組）；GET 讀取；對帳看板 ≥1 頁 | 頁面載入 < 1s（本機） | 停服務 + 刪 db schema（Expand-and-Contract 反向） |
| T-303 | AI 裁決閉環（ekpi 規則引擎接 Wellfit 隔離列） | P2 | T-203 | 隔離列 → 規則裁決 → 低信心人工 → 規則庫回寫；裁決率量測 | 自動裁決率、誤裁率（目標誤裁 0） | 關閉裁決開關，回退純人工 |

### Phase 4 — 原廠通道升級（不被我方阻塞）

| ID | 任務 | 優先 | 依賴 | 驗收準則 | 回滾 |
|---|---|---|---|---|---|
| T-401 | RDA API 直連落地 | P1 | OQ-F1 書面規格 ⚠ | 契約測試 + 冪等寫入 + 原廠確認範圍 | feature flag 停用 |
| T-402 | 64-bit/憑證/同步IPC 需求驗收 | P1 | C-05 回覆 | 安裝 64-bit 版重測基線；RAM/延遲對比 | 保留 32-bit 安裝並存切換 |
| T-403 | CDP 正式化 | P2 | T-403 現場驗證 + 原廠政策 | Playwright E2E ≥ 5 條核心流程綠 | 停用腳本 |

---

## 2. Phase 1 執行紀錄（即時更新）

| 任務 | 狀態 | 證據/產出 |
|---|---|---|
| T-101 | ✅ 完成 | `Wellfit/scripts/perf/flaps-perf.ps1`；首測 `Wellfit/artifacts/perf/20261103-160315/`：FlapsErp 執行中 11 行程、**全系統 RAM 623.8MB**（RSK-E3 實證）、COM 每呼叫 P50=0.26ms/P95=0.39ms（n=1000）、CDP 9222 NOT-FOUND（FlapsErp 執行中） |
| T-102 | ✅ 完成 | `crates/flaps_bridge`（根工作區 R 型 PoC）：6 tests 綠、clippy -D warnings 0、fmt 0；mock 封包對齊實測地板（260µs）；原子回滾語義（失敗回隊不丟失） |
| T-103 | ✅ 完成 | `Wellfit/scripts/erp-batch/run-mtrix-pipeline.ps1`；真實交付 fixtures 實測：**首跑 98ms**（5 檔/25 列/37 修復/51 findings，exit 0）、**冪等重跑 18ms 正確拒絕**（duplicate batch 已註冊） |
| T-104 | ✅ 完成 | `docs/research/CDP自動化Playbook_A3軌道_v0.1.md`：五步驗證流程 + Go 骨架 + 合規閘門 + 退讓鏈 |
| T-105 | ✅ 完成（Draft 待會簽） | `FlapsErp_RPA_電商系統對接_ADR-003_Layer3技術選型修正_RDA-CDP-COM三軌_草案.md` |
| T-106 | ✅ 完成 | `docs/research/FlapsErp優化_合規風險追蹤表.md`：RSK-E1~E8，含紅線（不寫 SYSTEM_PROGRAM_DIRECTORY） |
| T-107 | ✅ 完成 | config 快照 10 檔 + SHA256 manifest（`artifacts/perf/20261103-160315/config-snapshot/`） |

---

## 3. 回滾總矩陣（全藍圖）

| 變更類別 | 回滾程序 | 驗證方式 |
|---|---|---|
| 測量腳本（T-101/T-104） | 刪除 scripts/perf + artifacts；無系統狀態 | `git status`（Wellfit repo）乾淨 |
| Rust PoC crate（T-102） | 刪 `crates/flaps_bridge` + 根 Cargo.toml members 還原（編輯前自動留 `.bak`） | `cargo check --workspace` 通過 |
| 批次腳本（T-103） | 刪 scripts/erp-batch；執行產物刪 artifacts + 執行 db | ledger 冪等：同批次重跑被拒即證明可追溯 |
| config.ini 調優（P1 A/B） | 從 T-107 快照 manifest 逐檔還原 + SHA256 比對 | manifest 比對 100% 一致 |
| COM 註冊（未來 T-301） | `regsvr32 /u <dll>` + 移除 Module 檔 + .ver | COM 物件列舉為空 |
| ERP 端資料 | **我方不直接寫 ERP 資料庫**；所有寫入走官方匯入入口，回滾=不匯入/沖帳反向單據（會計程序） | ledger 對帳 |
| 文件（ADR/風險/藍圖） | 標 Superseded + 移入 archive | — |

**紅線**：任何變更不得寫入 `C:\SYSTEM_PROGRAM_DIRECTORY`（原廠元件區）——唯讀觀測除外。此紅線寫入 T-106 風險表 RSK-E5。

---

## 4. 修訂紀錄

| 版本 | 日期 | 變更 |
|---|---|---|
| v1.0 | 2026-11-03 | 初版：P0–P4 五階段、24 任務、依賴圖、驗收準則、回滾總矩陣；Phase 1 立即開工 |
