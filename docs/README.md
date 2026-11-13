# FlapsErp / Wellfit 智慧自動化平台 — 工程知識體系導覽總綱
## Engineering Knowledge Hub & Diátaxis Navigation Map

> **文件路徑**: `docs/README.md`｜**版本**: v3.0（2026-11-07 索引重整：修死鏈、收錄新文件、標記 superseded）
> **所屬專案**: FlapsErp / Wellfit 輔翼 ERP 智慧自動化與微服務平台
> **遵循標準**: Diátaxis Documentation Framework ⨁ 認知標籤體系
> **維護規則**: 新增文件必須同時登錄本索引；文件被取代時保留原檔並加 superseded 標記（不刪除）。
> **認知身分**: `[FACT:DOC]`

---

## 🗺️ 1. 知識拓撲導航地圖 (Diátaxis Matrix)

```
                     【知識導覽四象限】

         學習導向 (Learning)                目標導向 (Problem-Solving)
       ┌─────────────────────────────────┬─────────────────────────────────┐
 實作  │ 🚀 快速入門：本節＋下方索引      │ 🛠️ [04-operations/              │
(How-to)  │ 環境、命令與品質閘門速查        │  troubleshooting-runbook.md]    │
       │ [encoding-standard.md]          │ 凌晨三點故障急救、鎖死排除       │
       ├─────────────────────────────────┼─────────────────────────────────┤
 理論  │ 🏛️ [ARCHITECTURE.md]            │ 📖 [03-reference/               │
(What/ │ C4、STPA 控制論與設計決策       │  domain-glossary.md]            │
 Why)  │ [adr/] ADR 決策紀錄             │ 型別規格字典、代碼型別對照       │
       └─────────────────────────────────┴─────────────────────────────────┘
```

> 舊索引之 `docs/01-onboarding/`、`docs/02-architecture/` 路徑從未存在（死鏈已移除）；
> 快速入門內容由 §4 速覽、`encoding-standard.md`、`ARES_CARGO_WORKSPACE.md` 與 `adr/` 承接。

---

## 📚 2. 全域文檔索引 (Global Documentation Index)

### 2.1 根層控制文件（每日對接必讀）

| 檔案路徑 | 定位 | 目標讀者 | 摘要 |
| :--- | :--- | :--- | :--- |
| [`PROJECT_STATUS.md`](PROJECT_STATUS.md) | **專案現況快照** | 所有人 | 當前基線、品質門檻結果、版本事實 |
| [`NEXT_ACTIONS.md`](NEXT_ACTIONS.md) | **下一步行動清單** | 所有人 | 已排序之待辦與負責歸屬 |
| [`GAPS_AND_RISKS.md`](GAPS_AND_RISKS.md) | **存證與風險清單** | PM / 審查者 | `[FACT:CODE]` 實相依據、`[GAP]` 缺口與技術債 |
| [`PLAN_REGISTER.md`](PLAN_REGISTER.md) | **計畫登記冊（SSOT）** | PM / 架構師 | 所有計畫文件之 ID、狀態與效力位階 |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | **系統架構總綱** | 工程師 / 架構師 | C4、STAMP/STPA、COM STA 隔離設計 |
| [`encoding-standard.md`](encoding-standard.md) | **編碼標準** | 工程師 | UTF-8 無 BOM、CP950 禁令、PowerShell 寫檔規範（根因防制 mojibake） |
| [`knowledge-index.md`](knowledge-index.md) | **Knowledge KB 導引** | 所有人 | 外部知識庫（`../Knowledge/`）釘錨與 ADR-0004 |
| [`product-solution.md`](product-solution.md) | **產品方案說明** | 業務 / PM | 平台方案與價值主張 |

### 2.2 工程方法論（engineering/）

| 檔案路徑 | 定位 | 摘要 |
| :--- | :--- | :--- |
| [`engineering/DEVELOPMENT_METHODOLOGY.md`](engineering/DEVELOPMENT_METHODOLOGY.md) | **開發方法論 SSOT** | TDD 實證邊界、跨領域（韌體/FPGA/SoC）TDD、32 項缺口表（§8）、Skills 修補（§9） |
| [`engineering/THINKING_OPERATING_SYSTEM.md`](engineering/THINKING_OPERATING_SYSTEM.md) | **思考作業系統** | 四維決策程序（第一性分解/系統思考/紅隊/量化）、觸發表、決策記錄格式 |
| [`engineering/MUTATION_SCORE_REPORT.md`](engineering/MUTATION_SCORE_REPORT.md) | **突變測試報告 v2** | cargo-mutants 27.1：money/engine 排除設計、得分 ≥94.7%、Missed 分析 |
| [`engineering/FAULT_INJECTION_SOP.md`](engineering/FAULT_INJECTION_SOP.md) | **故障注入 SOP** | FAULT-01..07 追蹤表、注入點與驗證程序 |
| [`engineering/ATDD_BDD_BIND_AUDIT.md`](engineering/ATDD_BDD_BIND_AUDIT.md) | **ATDD 綁定稽核** | 8 情境 → Rust 測試綁定表、2 缺口 + 1 BLOCKED 追蹤 |
| [`engineering/SKILL_INVENTORY.md`](engineering/SKILL_INVENTORY.md) | **技能/MCP 盤點** | 可用 Skills、MCP、代理清單與實際使用記錄（未用須記原因） |
| [`engineering/TECHNICAL_DECISIONS.md`](engineering/TECHNICAL_DECISIONS.md) | **技術決策快查** | 高頻技術選型結論摘要 |
| [`engineering/TECH_STACK.md`](engineering/TECH_STACK.md) | **技術棧清單** | 根 workspace 依賴與版本 |
| [`engineering/FREE_AI_ROUTING.md`](engineering/FREE_AI_ROUTING.md) | **AI 路由設計** | 免费/低成本 AI 供應商路由策略（並行工作線） |
| [`engineering/KNOWLEDGE_EVOLUTION_MODULE.md`](engineering/KNOWLEDGE_EVOLUTION_MODULE.md) | **知識演進模組** | JEV、十大判斷準則、KEM 分層架構（裁決可插拔/雙時間軸/兩軌治理） |

### 2.3 架構與決策（architecture/、adr/）

| 檔案路徑 | 定位 | 摘要 |
| :--- | :--- | :--- |
| [`adr/ADR-0001-tech-stack-baseline.md`](adr/ADR-0001-tech-stack-baseline.md) | **ADR-0001** | 技術棧基線（edition 2021 / axum 0.7 / SQLite） |
| [`adr/ADR-0002-api-layer.md`](adr/ADR-0002-api-layer.md) | **ADR-0002** | API 層身分來源、RBAC 檢查、錯誤路徑 |
| [`adr/ADR-0003-double-entry-auth.md`](adr/ADR-0003-double-entry-auth.md) | **ADR-0003** | 複式記帳與授權設計 |
| [`adr/ADR-0011-语言分工与多语言整合.md`](adr/ADR-0011-语言分工与多语言整合.md) | **ADR-0011** | 語言分工與多語言整合 |
| [`architecture/SYSTEM_ARCHITECTURE.md`](architecture/SYSTEM_ARCHITECTURE.md) | 系統架構細節 | 模組關係與邊界 |
| [`architecture/API_DESIGN.md`](architecture/API_DESIGN.md) | API 設計 | 路由與契約慣例 |
| [`architecture/DATABASE_DESIGN.md`](architecture/DATABASE_DESIGN.md) | 資料庫設計 | SQLite schema 與索引 |
| [`architecture/效能架構.md`](architecture/效能架構.md) | 效能架構 | 效能預算、基線與 SLA（100k tx p95 ≤500ms） |
| [`design/INFORMATION_ARCHITECTURE.md`](design/INFORMATION_ARCHITECTURE.md) | 資訊架構 | 22+ 路由使用者視角組織（SSOT 為 api.rs 路由表） |

### 2.4 測試與品質（testing/、quality/）

| 檔案路徑 | 定位 | 摘要 |
| :--- | :--- | :--- |
| [`testing/TEST_STRATEGY.md`](testing/TEST_STRATEGY.md) | 測試策略 | 測試金字塔與範圍 |
| [`testing/TRACEABILITY_MATRIX.md`](testing/TRACEABILITY_MATRIX.md) | 追溯矩陣 | 需求 ↔ 測試對照 |
| [`testing/BDD_SCENARIOS.feature`](testing/BDD_SCENARIOS.feature) | BDD 情境源 | `驗證:` 標記對應 Rust 測試綁定 |
| [`quality/bdd_acceptance.md`](quality/bdd_acceptance.md) | BDD 驗收 | 情境驗收紀錄 |
| [`quality/ubiquitous_language.md`](quality/ubiquitous_language.md) | 共用語言 | 術語與代碼對齊 |
| [`testing/MVP_TEST_PLAN.md`](testing/MVP_TEST_PLAN.md)、[`testing/MVP_TEST_REPORT.md`](testing/MVP_TEST_REPORT.md) | MVP 測試 | 計畫與報告 |
| [`testing/REDESIGN_TEST_PLAN_v2.0.md`](testing/REDESIGN_TEST_PLAN_v2.0.md) | v2.0 測試 | 重設計測試計畫 |

### 2.5 產品與 UX（product/、ux/）

| 檔案路徑 | 定位 |
| :--- | :--- |
| [`product/PRD.md`](product/PRD.md)、[`product/MVP_SCOPE.md`](product/MVP_SCOPE.md)、[`product/PRODUCT_ROADMAP.md`](product/PRODUCT_ROADMAP.md) | 需求、範圍與路線圖 |
| [`product/REDESIGN_BLUEPRINT_v2.0.md`](product/REDESIGN_BLUEPRINT_v2.0.md)、[`product/USER_JOURNEYS.md`](product/USER_JOURNEYS.md)、[`product/USER_STORIES.md`](product/USER_STORIES.md) | 重設計藍圖、旅程與故事 |
| [`product/IMPROVEMENT_BACKLOG.md`](product/IMPROVEMENT_BACKLOG.md) | 改善待辦（P0-1 f64 技術債等） |
| [`product/MVP_ACCEPTANCE.md`](product/MVP_ACCEPTANCE.md) | MVP 驗收標準 |
| [`ux/UI-UX_需求規格.md`](ux/UI-UX_需求規格.md) | UI/UX 需求規格（Design Token、8 狀態、WCAG 2.2 AA） |

### 2.6 管理與治理（management/、standards/）

| 檔案路徑 | 定位 | 摘要 |
| :--- | :--- | :--- |
| [`standards/GOVERNANCE_STANDARD.md`](standards/GOVERNANCE_STANDARD.md) | **治理標準** | 效力順位、接手六步、證據綁定 |
| [`standards/SHARED_STANDARDS.md`](standards/SHARED_STANDARDS.md) | **工程基線** | forbid(unsafe_code)、零警告、金額禁浮點、PRG |
| [`management/營運治理機制.md`](management/營運治理機制.md) | 營運治理 | 日常治理運作 |
| [`management/專案管理最佳化方案_v1.0.md`](management/專案管理最佳化方案_v1.0.md) | 管理方案 | 專案管理最佳化 |
| [`management/OEaAP_Platform_Design_and_Development_Plan_v1.0.md`](management/OEaAP_Platform_Design_and_Development_Plan_v1.0.md) | OEKAP 計畫 | 主產品設計開發計畫 |
| [`management/OQ-E3_Resolution_Plan.md`](management/OQ-E3_Resolution_Plan.md) | OQ-E3 | 未決問題解法計畫 |
| [`management/ERP_Reconnaissance_Session_Decision.md`](management/ERP_Reconnaissance_Session_Decision.md) | 偵察決策 | ERP 偵察 session 裁定 |
| [`management/FlapsErp_RPA_電商系統對接_專案文件與開發前準備報告_v2.1.md`](management/FlapsErp_RPA_電商系統對接_專案文件與開發前準備報告_v2.1.md) | **FlapsErp RPA-ECOM 專案主文件 v2.1（charter 級）** | 現行 |
| [`management/FlapsErp_RPA_電商系統對接_專案文件與開發前準備報告_v2.1.html`](management/FlapsErp_RPA_電商系統對接_專案文件與開發前準備報告_v2.1.html) | 主文件 HTML 渲染版（A4 列印） | 現行 |
| [`management/FlapsErp_RPA_電商系統對接_ADR-001_既有資產關係與複用決策_草案.md`](management/FlapsErp_RPA_電商系統對接_ADR-001_既有資產關係與複用決策_草案.md) | RPA-ECOM ADR-001（既有資產複用；M1 前會簽） | 草案 |
| [`management/FlapsErp_RPA_電商系統對接_ADR-002_對接工具選型決策_草案.md`](management/FlapsErp_RPA_電商系統對接_ADR-002_對接工具選型決策_草案.md) | RPA-ECOM ADR-002（對接工具選型；M2 前會簽） | 草案 |
| [`management/FlapsErp_RPA_電商系統對接_ADR-003_Layer3技術選型修正_RDA-CDP-COM三軌_草案.md`](management/FlapsErp_RPA_電商系統對接_ADR-003_Layer3技術選型修正_RDA-CDP-COM三軌_草案.md) | RPA-ECOM ADR-003（Layer 3 技術選型修正；RDA/CDP/COM 三軌） | 草案 |
| [`management/FlapsErp_RPA_電商系統對接_Gate1複審報告_v1.md`](management/FlapsErp_RPA_電商系統對接_Gate1複審報告_v1.md) | RPA-ECOM Gate 1 複審與放行紀錄 | 現行 |
| [`management/FlapsErp_RPA_Ecommerce_Project_Plan_v2.1.md`](management/FlapsErp_RPA_Ecommerce_Project_Plan_v2.1.md) | ~~FlapsErp 計畫 v2.1（執行摘要版）~~ | ⚠️ **SUPERSEDED by 主文件** |
| [`management/templates/`](management/templates/) | 範本 | 週報 T-02、閘門 T-03、ADR T-04、研究 T-08 |

### 2.7 研究與交付（research/、delivery/、demo/、production/）

| 檔案路徑 | 定位 | 狀態 |
| :--- | :--- | :--- |
| [`research/輔翼ERP逆向分析與最佳化方案_v1.0.md`](research/輔翼ERP逆向分析與最佳化方案_v1.0.md) | 逆向分析主報告 | 現行 |
| [`research/輔翼ERP優化多階段執行藍圖_v1.0.md`](research/輔翼ERP優化多階段執行藍圖_v1.0.md) | 多階段藍圖 | 現行 |
| [`research/FlapsErp效能基線報告_v1.0.md`](research/FlapsErp效能基線報告_v1.0.md) | 產品效能基線（2026-11-03） | 現行（對照基準） |
| [`research/FlapsErp平台線效能評估進度報告_v2.0.md`](research/FlapsErp平台線效能評估進度報告_v2.0.md) | **平台線全程進度報告（2026-11-09）**：開發至今時間軸、246 測試、SLA 422ms、RAG、安全處置 | 現行（平台線） |
| [`research/FlapsErp產品線效能進度報告_v2.0.md`](research/FlapsErp產品線效能進度報告_v2.0.md) | **產品線進度報告（2026-11-05，Electron）**：config.ini 調優核對表、A/B 雙軌、P1–P4 藍圖、RSK-E1~E9 | 現行（產品線） |
| [`research/FlapsErp產品線效能深度分析_v1.0.md`](research/FlapsErp產品線效能深度分析_v1.0.md) | 產品線效能深度分析（DOC-01；2026-11-11 自根目錄遷入） | 現行 |
| [`research/FlapsErp產品線專案章程_v1.0.md`](research/FlapsErp產品線專案章程_v1.0.md) | 產品線專案章程（DOC-02；2026-11-11 自根目錄遷入） | 現行 |
| [`research/FlapsErp產品線架構優化計畫_v2.0.md`](research/FlapsErp產品線架構優化計畫_v2.0.md) | 產品線架構優化計畫（DOC-03；2026-11-11 自根目錄遷入） | 現行 |
| [`research/FlapsErp優化_合規風險追蹤表.md`](research/FlapsErp優化_合規風險追蹤表.md) | 合規風險 | 現行 |
| [`04-operations/A1檔案通道端到端操作手冊_v1.0.md`](04-operations/A1檔案通道端到端操作手冊_v1.0.md)、[`research/CDP自動化Playbook_A3軌道_v0.1.md`](research/CDP自動化Playbook_A3軌道_v0.1.md) | 操作手冊 | 現行 |
| [`research/flaps_bridge測試規格_v1.0.md`](research/flaps_bridge測試規格_v1.0.md) | bridge 測試規格 | 現行 |
| [`research/proposals/OQ-F1_F2_輔翼ERP正式需求提案_v2.1.md`](research/proposals/OQ-F1_F2_輔翼ERP正式需求提案_v2.1.md) | **OQ-F1/F2 正式提案（現行）** | v2.1 |
| [`research/proposals/OQ-F1_F2_輔翼ERP正式需求提案_v1.0.md`](research/proposals/OQ-F1_F2_輔翼ERP正式需求提案_v1.0.md) | OQ-F1/F2 提案 | ⚠️ **SUPERSEDED by v2.1**（保留供追溯） |
| [`delivery/DELIVERY_MANIFEST_v2.0.md`](delivery/DELIVERY_MANIFEST_v2.0.md)、[`demo/`](demo/)、[`production/PRODUCTION_READINESS.md`](production/PRODUCTION_READINESS.md) | 交付/演示/量產 | 現行 |

### 2.8 安全與維運（security/、04-operations/）

| 檔案路徑 | 定位 |
| :--- | :--- |
| [`security/THREAT_MODEL.md`](security/THREAT_MODEL.md) | 威脅模型 |
| [`04-operations/troubleshooting-runbook.md`](04-operations/troubleshooting-runbook.md) | SRE 急救手冊 |
| [`04-operations/FlapsErp設定變更runbook_v2.0.md`](04-operations/FlapsErp設定變更runbook_v2.0.md) | FlapsErp 設定變更 runbook（機台無關；Phase 0 偵測→分支→套用→量測→回滾） |
| [`04-operations/FlapsErp最佳設定基線_v2.0.md`](04-operations/FlapsErp最佳設定基線_v2.0.md) | FlapsErp 最佳設定基線（機台無關；config 7 鍵/宿主/衛生/工作台） |
| [`04-operations/FlapsErp局部設定最佳化Runbook_v3.0.md`](04-operations/FlapsErp局部設定最佳化Runbook_v3.0.md) | FlapsErp 局部設定最佳化（安裝目錄 14 檔案/目錄微調） |
| [`04-operations/FlapsErp系統互動精密調校指南_v1.0.md`](04-operations/FlapsErp系統互動精密調校指南_v1.0.md) | FlapsErp 與共存軟體的 12 個互動對精密調校 |
| [`04-operations/FlapsErp效能優化操作手冊_v1.0.md`](04-operations/FlapsErp效能優化操作手冊_v1.0.md) | 效能優化操作手冊（config §3.2 已更正；平台線 §2.2 標自建） |
| [`04-operations/OPS_SLA_RUNBOOK_v1.0.md`](04-operations/OPS_SLA_RUNBOOK_v1.0.md) | OPS-SLA 營運手冊（SLO/SLI、備份還原、P1/P2 事件管理） |
| [`04-operations/PERF_BASELINE_v1.0.md`](04-operations/PERF_BASELINE_v1.0.md) | 效能基線（`/api/search` BM25-only 與工作台 TTFB；2026-11-13 實測） |
| [`research/FlapsErp設定AB實測報告_v1.0.md`](research/FlapsErp設定AB實測報告_v1.0.md) | 設定 A/B 實測報告（NOT VERIFIED；GPU 偏好+優先級已生效） |
| [`03-reference/domain-glossary.md`](03-reference/domain-glossary.md) | 領域術語字典（SSOT） |

---

## 🏷️ 3. 認知標籤快速查閱指南 (Epistemic Tags)

- `[FACT:CODE]`：**已驗證代碼事實**（附原始碼路徑）。
- `[FACT:DOC]`：**已確認規範事實**（法規、合約或經核准設計）。
- `[INFER]`：**合理推論**（由架構模式推導，尚未直接驗證）。
- `[PROP]`：**改善提案**（未來建議，尚未實作）。
- `[LIMIT]`：**已知限制**（系統邊界、硬體極限或已知技術債）。
- `[GAP]`：**關鍵缺口**（阻礙正確實作的未知問題）。

---

## ⚡ 4. 核心系統一句話速覽

本系統為基於 **Rust 2021 Workspace** 建構之高效能、高可靠性企業級整合平台：
* **前端展示**：**Strict Zero-JS** — 純 HTML/CSS/SVG（無 `<script>`、無內聯事件）⨁ Tauri 2.0 桌面殼；Design Token 體系（`web/assets/design-system.css`）。
* **後端服務**：Axum 0.7 REST API + Tokio 異步運行時；讀取 GET／寫入 POST／成功後 PRG。
* **持久化層**：SQLx 0.8（SQLite WAL，冪等事務；金額一律 i128 minor 單位，禁浮點）。
* **自動化核心**：Win32 / Windows COM 0.58 STA 線程隔離，驅動輔翼 ERP、哈瓦士 MTRIX 與蝦皮訂單流。
* **品質閘門**（每次交付必過）：`cargo fmt -- --check`；`cargo clippy --all-targets -- -D warnings`；`cargo test --locked`（當前 **246 passed / 0 failed**，含 100k 交易 stress warm p95 422ms ≤500ms）；`cargo audit`（2 個已文件化處置）。

---

## 🔄 5. 索引維護規則

1. 新增文件 → 同一提交登錄本索引（SHARED_STANDARDS §4.3：文件與代碼同一提交更新）。
2. 取代文件 → 原檔開頭加 `> ⚠️ SUPERSEDED by <新路徑>（保留供追溯）`，索引同步改註。
3. 死鏈零容忍：本索引所有相對路徑以 `docs/` 為根；發現死鏈即修。
4. 並行工作線之文件（如 FREE_AI_ROUTING、KNOWLEDGE_EVOLUTION_MODULE）收錄時註明所屬線，避免誤認為主線 SSOT。

---

## 6. Kilo 快照區追溯（工作區外；RP-6②）

> Kilo 計畫快照區位於使用者 profile（`~/.local/share/kilo/plans/`，**工作區外**），由 Kilo 管理——**不搬移、不刪除**；此處僅登錄位置與版本鏈供追溯。SSOT 為 `docs/` 內文件。

### 6.1 效能家族演化鏈（已核實）

| 家族 | 版本鏈 | Kilo 內路徑（`~/.local/share/kilo/plans/`） |
| :--- | :--- | :--- |
| 效能進度報告（產品線） | v1 → **v2.0** | `1793787631643-flaps-erp-performance-progress-report.md` → `1793788660000-flaps-erp-performance-progress-report-v2.0.md`（35,198 B）＋ `.html`（66,298 B 渲染版） |
| 安裝優化指南 | v1 → v2.0 → v3.0 → v4.0 → v5.0 → v6.0 | `1793787631643-...install-optimization-guide.md` → `1793788300000-...-v2.0.md` → `1793789000000-...-v3.0.md` → … |
| 深度驗證報告 | — | `1793789700000-*`（deep-verification report） |

> v2.0 產品線報告已入工作區為 [`research/FlapsErp產品線效能進度報告_v2.0.md`](research/FlapsErp產品線效能進度報告_v2.0.md)（RP-1 入冊）；Kilo 快照副本保留。

### 6.2 其他專案計畫

其他專案計畫（ISMS／ekpi-platform／MTRIX／Wellfit／ARES／KNB 等 ~50 份）見 Kilo 快照區；**歸各專案 SSOT，不複製不搬移**。