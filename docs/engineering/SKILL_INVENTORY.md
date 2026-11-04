# Skill Inventory — Dev 根工作區技能盤點

> 版本：v1.0｜日期：2026-11-03｜狀態：Active
> 依《ELITE PRODUCT ENGINEERING MASTER DIRECTIVE》Part 01 與本工作區 AGENTS.md §5 維護。
> 更新時機：每個使用 Skills 的 session 結束時更新「實際使用」欄。

## 1. 內建 Skills（Kilo builtin）

| Skill Name | Description | Availability | Domain | Applicable Tasks | Priority | Load Status | Actual Usage | Deliverables | Reason |
|------------|-------------|--------------|--------|------------------|----------|-------------|--------------|--------------|--------|
| kilo-config | Kilo 組態、代理、plan 位置、MCP 設定 | 可用 | Delivery & Documentation | 定位 .kilo 結構、代理權限、plan 路徑 | High | 已載入（本次 session） | 讀取 kilo.jsonc／agent-manager.json／.kilo/agents/* 完成盤點 | 本檔、plan 檔定位 | 組態與代理清單的權威來源 |
| agent-md-refactor | 重構膨脹的 AGENTS.md 為漸進揭露結構 | 可用 | Delivery & Documentation | 根 AGENTS.md 結構設計 | Medium | 未載入（指引已知） | 根 AGENTS.md 採「入口＋連結 owner」模式（P1 不複製規則） | `AGENTS.md` | 模式簡單，無需完整載入 |
| skill-creator | 建立新 Skill 的指南 | 可用 | Delivery & Documentation | 未來設計專案專屬 Skill | Low | 未載入 | 未使用 | — | 現有技能足夠 |
| microsoft-docs / microsoft-code-reference | Microsoft 官方文件與 SDK 範例 | 可用 | Software Engineering | .NET／Azure 相關 | Low | 未載入 | 未使用 | — | 本工作區為 Rust，無 Microsoft SDK 需求 |
| 其餘（beam-concepts、canvas-design、connecting-to-data-source、content-research-writer、cortex-code、dynamic-tables-tutorial、figma-implement-design、purview-data-catalog） | 與本工作區領域無關 | 可用但不在範圍 | — | — | — | 未載入 | 未使用 | — | 領域不符，載入浪費 context |

## 2. 工作區代理（.kilo/agents，primary mode）

| Agent | 權限摘要 | 實際使用（2026-11-03 session） | Reason |
|-------|----------|-------------------------------|--------|
| architect | read＋edit `.kilo/plans/*.md`＋plan_exit | 產出執行計畫（前一輪）；本輪由 code 代理接手實作 | 規劃／實作分離 |
| code | 完整實作權限 | 本輪執行 Slices 1–4 | — |
| code-reviewer | read＋bash，唯讀審查 | 待執行（最終審查階段以 subagent 代替） | 詳見 §4 |
| code-skeptic | read＋bash＋md 編輯 | 待執行 | 詳見 §4 |
| test-engineer | read＋bash＋測試檔編輯 | 待執行（migrations 測試） | 詳見 §4 |

## 3. MCP 伺服器（.kilo/kilo.jsonc，全部 enabled）

| Server | 類型 | 本次實際使用 | 產出 | Reason |
|--------|------|--------------|------|--------|
| playwright | local（bunx） | 是：`browser_open` 實機檢視（總覽／介面狀態／新增交易三分區） | 截圖證據（深色主題、:target 切換、狀態樣式、表單） | UI 驗證不經原始碼宣稱完美 |
| crates-io | local（cratesio-mcp.exe） | 未直接呼叫 | — | 版本確認由 cargo update/audit 覆蓋 |
| rust-analyzer | local | 未呼叫 | — | clippy/test 已覆蓋診斷需求 |
| narsil | local | 未呼叫 | — | 無語意搜尋需求 |
| firecrawl / jina / context7 / grep-app | remote | 未呼叫 | — | 本輪無需外部研究（ADR-0001 基於實測） |
| sequentialthinking | local | 未呼叫 | — | 計畫已定，執行不需額外推理工具 |

## 4. 缺口與改善方案

1. **code-reviewer／code-skeptic 為獨立 primary 代理**，在同一 session 內不可直接呼叫 → 以 `task` subagent（explore/general 型）執行等效獨立審查，並在交付報告記錄差異。
2. **無 Node.js/npm**：前端建構工具鏈（含 Playwright CLI）不可用 → 以零 JS 純 HTML/CSS 路線迴避（ADR-0001 D-03），介面驗證走 Integrated Browser。
3. **無 Product Discovery 專屬 Skill** → 以 PRD／User Journeys 模板＋code-skeptic 式需求消歧替代。
