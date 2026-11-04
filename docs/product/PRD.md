# PRD — FlapsErp RPA 支援線（根 workspace）產品需求文件

> 版本：v1.0｜日期：2026-11-03｜狀態：Active（適用於根 workspace 的 `web/` 表現層與 `platform/` 服務層）
> 主產品裁定見 `docs/adr/ADR-0001-tech-stack-baseline.md`：主產品為 `ekpi-platform/`（OEKAP），本 PRD 範圍僅涵蓋根 workspace（FlapsErp RPA 支援線）。
> 追溯規則：每項需求 ID 需對應到使用者流程（`USER_JOURNEYS.md`）→ UI 元件（`web/index.html`）→ 程式模組 → 測試案例 → 驗收證據。

## 1. 產品概述

- **Product Vision**：讓輔翼 ERP 的帳務登帳、查詢與 AI 決策輔助在單一零 JavaScript 介面完成，提供可稽核、可驗證的操作流程。
- **Business Goals**：降低帳務操作延遲（目標見 `docs/ARCHITECTURE.md` INV 系列）；消除 UI 凍結；以證據與稽核軌跡建立 AI 信任（H3）。
- **Target Users**：營運人員（登帳／查詢）、財務審計員（覆核）、IT 管理員（設定與權限）。
- **Out of Scope（本 PRD）**：ekpi-platform 的知識搜尋與治理功能；Wellfit 銀行介接；SyncMesh 裝置同步。

## 2. 功能需求（Functional Requirements）

| ID | 需求 | 優先級 | 驗收條件（Given/When/Then） | 對應模組 |
|----|------|--------|------------------------------|----------|
| FR-001 | 表現層零 JavaScript | Must | Given 任意頁面 When 檢視原始碼 Then `<script>` 標籤數 = 0 且無內聯事件處理器 | `web/index.html` |
| FR-002 | 設計系統一致性 | Must | Given 頁面渲染 When 檢查樣式 Then 色彩／間距／字體皆來自 Design Token（`web/assets/design-system.css`） | `web/assets/design-system.css` |
| FR-003 | 表單 PRG | Must | Given 使用者提交「新增交易」表單 When 伺服器處理成功 Then 回應 302 導向交易清單；重新整理不重複送出 | `platform/`（API 層待建）、`web/index.html` |
| FR-004 | 完整元件狀態 | Must | Given 資料載入中／空／失敗／部分不可用 When 介面渲染 Then 對應 Loading／Empty／Error（含復原動作）／Degraded 狀態 | `web/index.html` §states |
| FR-005 | 帳務登帳驗證 | Must | Given 交易含空欄位／負數金額／未來日期／非 DEBIT/CREDIT 借貸別 When 提交 Then 伺服器拒絕並回傳欄位級錯誤訊息 | `core/src/ledger/engine.rs::validate_journal_entry` |
| FR-006 | RBAC 權限檢查 | Must | Given 使用者屬於某角色 When 呼叫受控操作 Then 以 permission_code 判定（非 permission_id），無權限回傳明確拒絕 | `core/src/permission.rs::PermissionChecker` |
| FR-007 | 交易查詢分頁 | Should | Given 查詢含起訖日期／狀態／limit/offset When 執行 Then 回傳排序結果且參數化（無字串拼接） | `core/src/ledger/engine.rs::query_transactions` |
| FR-008 | AI 決策閘門 | Could | Given AI 風險評分 score<0.3 / 0.3–0.7 / ≥0.7 When 決策 Then 自動通過／人工覆核／自動拒絕＋告警；金額修改、權限變更、作廢、科目變更、結帳為 AI 禁區 | `platform/src/ai/decision_gating.rs` |

## 3. 非功能需求（Non-functional Requirements）

| ID | 類別 | 目標 | 測量方式 |
|----|------|------|----------|
| NFR-001 | 安全 | 輸入驗證 fail-closed；SQL 一律參數化；輸出編碼防注入 | 負向測試＋`cargo audit`（基線見 ADR-0001 §4） |
| NFR-002 | 無障礙 | WCAG 2.2 AA 方向：鍵盤導航、可見焦點、語義 HTML、對比 | Playwright 掃描＋人工鍵盤走查（`docs/testing/TEST_STRATEGY.md`） |
| NFR-003 | 效能 | 靜態頁面首次載入 < 1s（本地）；登帳 API P95 < 50ms（SQLite WAL） | 瀏覽器 DevTools；criterion 基準（待建，backlog） |
| NFR-004 | 可維護性 | 所有 public API 有 doc comment；clippy 零警告 | `cargo clippy --all-targets -- -D warnings` |
| NFR-005 | 反模擬 | 介面不得以範例數字冒充真實資料；未知值顯示「—」 | UI 審查（FR-004 證據） |

## 4. 商業規則（Business Rules）

1. 金額／數量計算禁用浮點（INV-01）；現有 f64 技術債見 IMPROVEMENT_BACKLOG P0-1，修復前登帳結果不得用於真實會計。
2. 讀取 GET；建立／修改／刪除 POST；成功後 PRG（憲章 §3 前專案約束）。
3. COM 真實呼叫（`flaps_bridge` `real-com`）需 ERP 廠商書面授權（RSK-E1），未授權前僅 stub。
4. AI 禁區：金額修改、權限變更、交易作廢、會計科目變更、結帳／年度結轉（`platform/src/ai/decision_gating.rs`）。

## 5. 資料需求（Data Requirements）

- Schema SSOT：`platform/migrations/`（版本化遷移）；現有動態建表（`core/ledger/engine.rs::ensure_tables`、`platform/src/db.rs::initialize`）收斂計畫見 IMPROVEMENT_BACKLOG。
- 既有資料相容：遷移一律 `IF NOT EXISTS` 冪等起手；破壞性變更採 Expand-and-Contract。
- 備份／還原：依 SHARED_STANDARDS §8.1（git bundle＋每月還原演練；根 workspace 未初始化 git，列為 blocker）。

## 6. 風險與依賴

| 風險 | 影響 | 緩解 |
|------|------|------|
| f64 貨幣技術債 | 會計累積誤差 | Backlog P0-1；修復前標記登帳為非會計用途 |
| 根 workspace 無 git | 證據無法綁修訂版 | Backlog P0-2；session 記錄暫代 |
| flaps_bridge EULA 未決 | COM 通道不可用 | feature-gated＋stub 測試（7 passed） |
| 後端 API 層未接線 | 表單 PRG 無法端到端 | ADR-0001 §6；API 層列為下一切片候選 |

## 7. 成功指標（Success Metrics）

| 指標 | 現況（2026-11-03 基線） | 目標 |
|------|--------------------------|------|
| clippy 零警告 | PASS（EXIT=0） | 持續保持 |
| 測試 | 9 passed / 0 failed | 覆蓋 FR-005~008 的測試補齊 |
| 依賴漏洞 | 0 高危（rsa 文件化接受） | 持續 0 高危 |
| 零 JS 合規 | PASS（script=0、內聯事件=0） | 持續 100% |
