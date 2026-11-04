# FlapsErp / Wellfit 智慧自動化平台 — 工程知識體系導覽總綱
## Engineering Knowledge Hub & Diátaxis Navigation Map

> **文件路徑**: `docs/README.md`  
> **所屬專案**: FlapsErp / Wellfit 輔翼 ERP 智慧自動化與微服務平台  
> **遵循標準**: Antigravity 2.0 Sovereign Specification ⨁ Diátaxis Documentation Framework  
> **維護者**: 核心架構智囊團 | **認知身分**: `[FACT:DOC]`

---

## 🗺️ 1. 知識拓撲導航地圖 (Diátaxis Matrix)

整套工程知識體系依照 **Diátaxis 四維框架** 組織，請依據您的當前目標選擇閱讀路徑：

```
                             【知識導覽四象限】
             
                 學習導向 (Learning)               目標導向 (Problem-Solving)
               ┌─────────────────────────────────┬─────────────────────────────────┐
    實作導向   │ 🚀 [01-onboarding/quickstart.md] │ 🛠️ [04-operations/runbook.md]   │
  (Practical)  │ 新手 5 分鐘跑通環境與本地驗收    │ 凌晨三點故障急救、鎖死排除與回滾│
               ├─────────────────────────────────┼─────────────────────────────────┤
    理論導向   │ 🏛️ [02-architecture/c4-models]  │ 📖 [03-reference/glossary.md]   │
  (Theoretical)│ C4 架構圖、STPA 控制論與設計決策 │ 型別規格字典、COM 介面與資料模型│
               └─────────────────────────────────┴─────────────────────────────────┘
```

---

## 📚 2. 全域文檔索引目錄 (Global Documentation Index)

| 檔案路徑 | 模組定位 | 目標讀者 | 核心內容摘要 |
| :--- | :--- | :--- | :--- |
| [`docs/ARCHITECTURE.md`](file:///c:/Users/stephanie.huang/Desktop/新人訓練/Projects/2026/Dev/docs/ARCHITECTURE.md) | **系統架構總綱** | 所有工程師 / 架構師 | C4 架構模型、STAMP/STPA 階層控制、Rust 與 Windows COM STA 隔離設計 |
| [`docs/GAPS_AND_RISKS.md`](file:///c:/Users/stephanie.huang/Desktop/新人訓練/Projects/2026/Dev/docs/GAPS_AND_RISKS.md) | **存證與風險清單** | 專案經理 / 審查者 | `[FACT:CODE]` 實相依據、`[GAP]` 阻塞缺口與技術債演進清單 |
| [`docs/01-onboarding/quickstart.md`](file:///c:/Users/stephanie.huang/Desktop/新人訓練/Projects/2026/Dev/docs/01-onboarding/quickstart.md) | **五分鐘快速入門** | 新進開發者 | Rust 依賴、Windows SDK 配置、編譯除錯與第一個探針驗證 |
| [`docs/02-architecture/decisions/`](file:///c:/Users/stephanie.huang/Desktop/新人訓練/Projects/2026/Dev/docs/02-architecture/decisions/) | **ADR 決策專區** | 架構師 / 資深工程師 | 歷史架構決策紀錄（含單向門/雙向門權衡與不可變約束） |
| [`docs/03-reference/domain-glossary.md`](file:///c:/Users/stephanie.huang/Desktop/新人訓練/Projects/2026/Dev/docs/03-reference/domain-glossary.md) | **領域術語字典** | 全體團隊 | 輔翼 ERP、哈瓦士 MTRIX、蝦皮報表欄位與代碼型別對照表 (SSOT) |
| [`docs/04-operations/troubleshooting-runbook.md`](file:///c:/Users/stephanie.huang/Desktop/新人訓練/Projects/2026/Dev/docs/04-operations/troubleshooting-runbook.md) | **SRE 急救手冊** | 維運 / On-call | 視窗找不到、Windows COM STA 跨線程崩潰、SQLite 鎖爭用快速處置 |

---

## 🏷️ 3. 認知標籤快速查閱指南 (Epistemic Tags)

閱讀本專案文檔時，所有關鍵陳述皆帶有以下標準標籤：
- `[FACT:CODE]`：**已驗證代碼事實**（附帶原始碼路徑，如 `crates/core/src/lib.rs`）。
- `[FACT:DOC]`：**已確認規範事實**（法律法規、正式合約或經核准之設計）。
- `[INFER]`：**合理推論**（由架構模式推導，尚未直接驗證）。
- `[PROP]`：**改善提案**（屬於未來建議方案，尚未實作）。
- `[LIMIT]`：**已知限制**（系統邊界、硬體極限或已知技術債）。
- `[GAP]`：**關鍵缺口**（阻礙正確實作的未知問題，需由專案負責人解答）。

---

## ⚡ 4. 核心系統一句話速覽

本系統為基於 **Rust 2021 Workspace** 建構之高效能、高可靠性企業級整合平台：
*   **前端展示**：純 CSS + `web-sys` ⨁ Tauri 2.0 桌面 GUI。
*   **後端服務**：Axum 0.7 REST API + Tokio 異步運行時。
*   **持久化層**：Sqlx 0.8 (SQLite WAL 模式，具備冪等事務)。
*   **自動化核心**：Win32 / Windows COM 0.58 專屬 STA 線程隔離架構，無縫驅動輔翼 ERP、哈瓦士 MTRIX 與蝦皮訂單流。

