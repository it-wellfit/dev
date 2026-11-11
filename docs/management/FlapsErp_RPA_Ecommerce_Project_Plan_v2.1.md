# FlapsErp RPA 電商整合專案計畫 v2.1

> ⚠️ **SUPERSEDED by [`FlapsErp_RPA_電商系統對接_專案文件與開發前準備報告_v2.1.md`](FlapsErp_RPA_電商系統對接_專案文件與開發前準備報告_v2.1.md)** — 原始專案主文件（Dev 根目錄 HTML v2.1，863 行，charter 級）已轉錄為 .md 版本入冊 docs/management/。本檔（2026-11-10 撰寫之摘要版）之獨有「現況對齊」段已併入主文件 §0 附錄。本檔保留供追溯，不得作為對外計畫依據。
> **修正記錄**：前版頭註宣稱「v2.1 前檔為 0 bytes 空殼，本版首次補實全文」對家族歷史不實——.md 檔本身 0 bytes 屬實，但家族 v2.1 內容一直存在於根目錄 HTML（2026-10-27 撰寫、11-03 Gate1 複審修訂）。依誠實紀律（GOVERNANCE P7）修正。

> 文件編號：PLAN-FLAPSERP-V2.1｜狀態：**SUPERSEDED**｜簽發：2026-11-07
> 上游：`docs/PLAN_REGISTER.md`（效力位階登記處）
> 定位：~~FlapsErp（輔翼 ERP）RPA 支援線之執行計畫 SSOT~~（已由主文件取代——本檔降級為執行狀態附錄）
> 事實紀律：`[FACT:CODE]` 已驗證代碼事實／`[FACT:DOC]` 已確認規範／`[PROP]` 提案未實作／`[LIMIT]` 已知限制。

---

## 1. 目標與範圍

| 項目 | 內容 |
| :--- | :--- |
| 業務目標 | 以 RPA＋橋接線取代輔翼 ERP 高頻人工操作（進銷存、庫存報表、應付憑單），支撐多通路 OMO、E.Sun 銀企直連、MTRIX 五檔對接。 |
| 技術範圍 | 根 workspace `core/`（領域與引擎）、`platform/`（API 中樞）、`web/`（零 JS 基準）、`crates/flaps_bridge`（COM STA 橋接）。 |
| 不在範圍 | 輔翼 ERP 產品本身之修改（僅能以原廠通道提案，見 OQ-F1/F2）；ekpi-platform 主產品線；Wellfit/SyncMesh 子專案（各有計畫）。 |
| 成功定義 | Phase 4 驗收：批次作業夜間無人值守通過 8h soak；對帳差異率歸零由 MTRIX 五檔勾稽證明。 |

## 2. 現況基線（2026-11-07 事實）

| 項目 | 事實 | 證據 |
| :--- | :--- | :--- |
| 品質門檻 | fmt/clippy/test/audit 四關 EXIT=0；**246 passed / 0 failed**（2026-11-09 log；含 RAG 混合檢索與安全升級後） | `cargo test --locked --workspace` |
| 效能 SLA | 100k 交易 warm p95 **422ms ≤ 500ms**（預熱趟＋21 樣本量測法） | `platform/src/api.rs` stress 測試 |
| 路由 | 60 條（api.rs 48＋cross_module_api 12；SSOT：原始碼路由表） | `[FACT:CODE]` 2026-11-09 計數 |
| COM 橋接 | windows 0.58（Win32_Foundation＋Win32_System_Com），STA 線程隔離 | 根 Cargo.toml |
| 效能基線報告 | `docs/research/FlapsErp效能基線報告_v1.0.md`；深度驗證報告與安裝最佳化指南 v6.0 已交付 | research/ |
| 廠商通道 | OQ-F1/F2 正式提案 **v2.1 已提交**（v1.0 superseded）；廠商回覆期限 2026-11-19；回覆未到 → 相關項目狀態一律 **UNKNOWN（非 PASS）** | research/proposals/ |

## 3. 分階段計畫（對齊多階段執行藍圖 v1.0）

| 階段 | 期間 | 內容 | 驗收證據 | 狀態 |
| :--- | :--- | :--- | :--- | :--- |
| Phase 1 量測與軌道建置 | 已執行 | 效能基線、壓力測試軌道、品質門檻制度化 | 效能基線報告 v1.0；stress 100k p95 422ms | ✅ 完成 |
| Phase 2 批次上線試行 | 11/10–11/16 | A1 檔案通道端到端（`research/A1檔案通道端到端操作手冊_v1.0.md`）、CDP 自動化 A3 軌道 | 操作手冊演練記錄；合規風險追蹤表同步 | 🟡 進行中 |
| Phase 3 橋接與看板 | 11/17–12/05 | flaps_bridge 測試規格全案落地（`research/flaps_bridge測試規格_v1.0.md`）；報表看板 | bridge 測試綠；KPI 看板上線 | ⬜ |
| Phase 4 原廠通道升級 | 不受我方阻塞 | OQ-F1/F2 廠商回覆後之 config.ini A/B、Winax COM 通道、8h soak | 廠商書面規格＋soak 證據 | ⬜ 阻塞於廠商（UNKNOWN） |

## 4. 硬性約束（全階段適用）

1. **零 JS 表現層**：瀏覽器只收 HTML/CSS/SVG；`<script>` 數 = 0。
2. **金額禁浮點**：i128 minor 單位（INV-01）；借貸平衡 ΣDEBIT=ΣCREDIT>0（INV-02）。
3. **HTTP 語義**：讀取 GET；建立/修改/刪除 POST；成功後 PRG。
4. **每 `.rs` 首行** `#![forbid(unsafe_code)]`；clippy 零警告；不得新增 allow。
5. **編碼**：一律 UTF-8 無 BOM（`docs/encoding-standard.md`）；PowerShell 寫檔禁 `>`。
6. **證據紀律**：UNKNOWN ≠ PASS ≠ FAIL；廠商/環境不可得之事實不得轉為已驗證。

## 5. 驗收與回滾

- 每階段閘門 = 四關全綠＋該階段驗收證據入 `docs/PLAN_REGISTER.md` addendum。
- 回滾矩陣：`docs/research/輔翼ERP優化多階段執行藍圖_v1.0.md` §3（全藍圖共用，不重複維護）。
- 風險與缺口：`docs/GAPS_AND_RISKS.md`（GAP-01 COM 無頭操作、GAP-02 蝦皮授權、GAP-07 廠商 OQ-F1/F2）。

## 6. 修訂紀錄

| 版本 | 日期 | 修訂 |
| :--- | :--- | :--- |
| v2.1 | 2026-11-07 | 首次補實全文（原檔 0 bytes）；對齊藍圖 v1.0 四階段；收錄 2026-11-07 門檻事實與 stress 量測法修正。 |
| v2.0 及之前 | — | 歷史草案，未保留下列之外的事實主張；以本版為準。 |
