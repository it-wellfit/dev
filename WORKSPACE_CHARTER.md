# 工作區憲章與導航 (Workspace Charter & Navigation)

**版本**: v1.0.0  
**生效日期**: 2026-10-25  
**適用範圍**: `C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev` 及所有子專案  
**狀態**: Active

---

## 1. 宗旨與範圍

本工作區包含多個獨立產品與工程／研究目錄。以下只列出已由目錄與入口文件確認的主要產品；表內狀態是文件記載，不等於本次已驗收：

| 子專案 | 路徑 | 產品定位 | 技術棧 | 狀態 |
|--------|------|----------|--------|------|
| ARES（工作區根目錄） | `core/`、`platform/`、`web/` | 根目錄 Cargo workspace；參見 `docs/ARES_MASTER_SPEC.md` | Rust workspace | 規格／計畫標示 Planning；須以程式碼與測試驗證進度 |
| ekpi-platform | `ekpi-platform/` | 可追溯、可驗證的企業知識與流程智慧平台 | Rust | 憲章記載 Phase A 未核准、Phase B 受阻 |
| Wellfit | `Wellfit/` | 輔翼 ERP 整合與資料自動化 | Rust (7-crate workspace) | `docs/project/CURRENT_STATE.md` 僅為 2026-10-25 驗證快照；目前工作樹有未提交變更 |
| SyncMesh | `SyncMesh/` | 跨平台裝置同步方案 | Rust workspace | 憲章中的完成／效能宣稱尚未由本次審核獨立驗證 |

其他目錄包括 `agentre-bench/`、`oekap/`、`re-toolchain/` 與 `rea-rust-upstream/`，須按各自入口文件辨識用途；`archive/`、`kilo-settings-skills-archive/` 與 `skills.backup-*/` 是歷史／封存內容，不是現行產品計畫來源。不要由目錄名稱推定產品狀態或跨專案依賴。

---

## 2. 工作區剛性不變量 (Rigid Invariants)

以下守則適用於所有子專案、所有程式語言、所有代理人操作，絕對不可逾越：

### 2.1 反過早固化
嚴禁在缺乏數學形式化證明或物理剛性界限下，將啟發式門檻硬編碼為固化不變量。僅宇宙大定理（李雅普諾夫收斂、熱力學矽結溫極限、密碼學單向雜湊）與局部封閉問題的可決定性最佳解始得固化。

### 2.2 Rust 記憶體安全與編譯零容忍
- 所有 Rust 原始碼與測試代碼第一行強制宣告 `#![forbid(unsafe_code)]`，零 `unsafe` 區塊容忍。
- `cargo clippy --all-targets -- -D warnings` 編譯期絕對零警告。
- 禁止 `assert!(true)` 偽斷言，必須綁定具名變數。

### 2.3 表現層 100% JavaScript-Free
客戶端展示層優先採用語義化 HTML5 與現代 CSS，全站 `<script>` 數量恆等於 0。

### 2.4 Windows PowerShell 執行邊界
- 多指令串接必須使用分號 (`;`)，嚴禁 `&&`。
- 執行 `cargo build --release` 前必須先終止殘留之後台守護程序。
- 優先調用 Windows 原生 `curl.exe`，避免未加 `-UseBasicParsing` 的 `Invoke-WebRequest`。

### 2.5 絕對反模擬
嚴禁偽造或硬編碼掃描結果、漏洞證明或固定金額損失。探針失敗必須如實回報，`UNKNOWN != PASS` 且 `UNKNOWN != FAIL`。

### 2.6 規則包容壓縮協議 (≤ 20 條)
本工作區規章條款總數嚴格限制 ≤ 20 條。新增規則時必須強制執行包容性合併，向上抽象為更高階的正向模式。

---

## 3. 開發工作流 (Universal Development Workflow)

所有工作項目均遵循以下標準生命週期：

1. **任務探索與評估** — 使用檢視文件與既有規格評估複雜度與影響半徑。
2. **需求拷問與消歧** — 消除規格歧義，形成決策共識，記錄未解決項目而非臆測。
3. **規格合成與工單切分** — 合成規格，切分 DAG 工單 (1 Ticket = 1 Context Window)。
4. **實作與紅綠重構** — 循序落實代碼，保持 100% 測試通過與零警告。
5. **雙軸代碼審查與驗證 PR** — 行為與架構雙盲審查，附帶形式化回滾證明。
6. **大一統全生命週期** — 重大里程碑啟用 8 大主門禁綠燈閉環。

---

## 4. 文件位置索引 (Documentation Index)

### 工作區層級
| 文件 | 路徑 | 用途 |
|------|------|------|
| 工作區憲章與導航 | `WORKSPACE_CHARTER.md` | 本文件，全工作區憲政與導航 |
| 共享開發標準 | `docs/standards/SHARED_STANDARDS.md` | 跨專案共享編碼、建置、測試標準 |
| 工作區治理準則 | `docs/standards/GOVERNANCE_STANDARD.md` | 檔案／文件／知識／工程四大管理體系的唯一 canonical 標準；含演進機制與合規鏈 |
| 跨專案計畫與決策登錄 | [docs/PLAN_REGISTER.md](docs/PLAN_REGISTER.md) | 專案文件權威、計畫重複、決策來源與未決阻塞的導航；不是產品核准或驗收證據 |
| ARES 主規格／實施計畫 | `docs/ARES_MASTER_SPEC.md`、`docs/ARES_IMPLEMENTATION_PLAN.md` | 根目錄 Cargo workspace 的規格與計畫 |

### ekpi-platform
| 文件 | 路徑 | 用途 |
|------|------|------|
| 專案憲章 | `ekpi-platform/PROJECT_CHARTER.md` | 專案概覽、目標、範圍、非目標 |
| 系統架構 | `ekpi-platform/docs/ARCHITECTURE.md` | 系統架構、資料流、元件關係 |
| API 規格 | `ekpi-platform/docs/API_SPEC.md` | CLI/HTML/JSON/XML 契約與介面 |
| 開發指南 | `ekpi-platform/docs/DEVELOPMENT_GUIDE.md` | 建置、測試、部署、本機檢查入口 |
| 品質標準 | `ekpi-platform/docs/QUALITY_STANDARDS.md` | 風險分級、品質門檻、定義完成 |
| 運維手冊 | `ekpi-platform/docs/OPERATIONS_RUNBOOK.md` | 備份、還原、監控、疑難排解 |
| 安全文件 | `ekpi-platform/docs/SECURITY.md` | 安全強化、機密管理、權限模型 |
| ADR 登錄 | `ekpi-platform/docs/ADR_REGISTER.md` | 架構決策記錄 |
| 追溯矩陣 | `ekpi-platform/docs/TRACEABILITY.md` | 需求追溯與驗證證據 |
| 疑難排解 | `ekpi-platform/docs/TROUBLESHOOTING.md` | 常見問題與解決方案 |

### Wellfit
| 文件 | 路徑 | 用途 |
|------|------|------|
| 專案入口 | `Wellfit/README.md` | 實際 crate、產品邊界、快速開始與限制 |
| 專案概覽 | `Wellfit/docs/project/PROJECT_OVERVIEW.md` | 產品與 crate 概覽（需與 README 同步） |
| 最後驗證快照 | `Wellfit/docs/project/CURRENT_STATE.md` | 2026-10-25 基線；不是即時工作樹狀態 |
| 文件地圖 | `Wellfit/docs/DOCUMENTATION_MAP.md` | 文件入口與分類；銀行介接模式存在未決衝突 |
| BankPay 專案文件 | `Wellfit/docs/bankpay/`、`Wellfit/docs/esun-bank-flaps-erp-integration/` | 兩組文件所描述的介接模式尚須裁定，不可任選一份視為已核准 |
| 系統架構 | `Wellfit/docs/ARCHITECTURE.md` | 系統架構、資料流、元件關係 |
| CLI 契約 | `Wellfit/docs/CLI.md` | CLI 子命令與使用契約 |
| 測試／追溯／品質文件 | `Wellfit/docs/TESTING_QUALITY.md`、`Wellfit/docs/TRACEABILITY_MATRIX.md`、`Wellfit/docs/QUALITY_REVIEW_REPORT.md` | 品質與證據入口 |

### SyncMesh
| 文件 | 路徑 | 用途 |
|------|------|------|
| 專案憲章 | `SyncMesh/PROJECT_CHARTER.md` | 專案概覽、目標、範圍、非目標 |
| 系統架構 | `SyncMesh/docs/ARCHITECTURE.md` | 系統架構、資料流、元件關係 |
| API 規格 | `SyncMesh/docs/API_SPEC.md` | CLI 契約與介面 |
| 開發指南 | `SyncMesh/docs/DEVELOPMENT_GUIDE.md` | 建置、測試、部署、本機檢查入口 |
| 品質標準 | `SyncMesh/docs/QUALITY_STANDARDS.md` | 風險分級、品質門檻、定義完成 |
| 運維手冊 | `SyncMesh/docs/OPERATIONS_RUNBOOK.md` | 備份、還原、監控、疑難排解 |
| 安全文件 | `SyncMesh/docs/SECURITY.md` | 安全強化、機密管理、權限模型 |
| ADR 登錄 | `SyncMesh/docs/ADR_REGISTER.md` | 架構決策記錄 |
| 追溯矩陣 | `SyncMesh/docs/TRACEABILITY.md` | 需求追溯與驗證證據 |
| 疑難排解 | `SyncMesh/docs/TROUBLESHOOTING.md` | 常見問題與解決方案 |

---

## 5. 環境與工具鏈

| 工具 | 版本 | 用途 | 狀態 |
|------|------|------|------|
| Rust / cargo | 1.99.1 | 主要建置工具鏈 | 已安裝 |
| Git | 2.56.0.windows.3 | 版本控制 | 已安裝 |
| GitHub CLI (gh) | 2.103.2 | Issue/PR 操作 | 已安裝，未驗證 |
| Node.js / npm | — | 未安裝 | 缺失 |
| codex CLI | — | 未安裝 | 缺失 |

Rust 工具鏈固定於 `rust-toolchain.toml` (channel 1.99.1, minimal, clippy+rustfmt)。

---

## 6. Git 與驗證狀態

Git 分支、遠端、工作樹是否乾淨及測試結果都是即時狀態，不在本憲章中固化。開始工作前，應在相關 repository 執行 `git status`，並以當次測試輸出更新專案狀態文件。不得把 `SyncMesh/PROJECT_CHARTER.md` 的版本宣告或舊計畫中的測試數字當作已驗證結果。

---

## 7. 計畫文件管理

- 每個工作範圍最多一份經核准的執行計畫；規格、ADR、測試策略與狀態報告是不同文件類型，不因減少數量而合併。
- 執行計畫必須標明負責人、狀態、最後驗證日期／證據、依賴與阻塞、驗收條件及取代關係。缺少核准或當前證據者視為提案，不可直接執行或宣稱完成。
- 被取代文件保留追溯，但需在文件開頭標明「Superseded」及唯一替代文件；歷史／備份目錄不納入現行計畫索引。
- 若兩份文件對業務模式、範圍或驗收條件互相矛盾，先標為 Blocked 並取得責任人裁定，不以合併文字掩蓋衝突。

---

## 8. 文件更新規則

- 任何跨專案變更必須同步更新 `WORKSPACE_CHARTER.md` 與 `docs/standards/SHARED_STANDARDS.md`。
- 管理體系（檔案、文件、知識、工程管理）的治理規則以 `docs/standards/GOVERNANCE_STANDARD.md` 為唯一 owner；本憲章僅定效力順位，不重複其內容。
- 子專案專屬變更僅更新該專案對應文件，並記錄於 ADR 登錄。
- 文件變更必須與代碼變更在同一提交中完成，保持文件與代碼同步。
- 過期文件標記為 `[Deprecated]`，並註明替代文件與日期。

---

## 9. Session 檔案管理

所有對話產出、測試、執行與匯入／匯出檔案必須置於有組織的專案目錄，不得散亂於桌面或工作區根目錄。

- **Canonical 標準**：`docs/standards/GOVERNANCE_STANDARD.md` §2（檔案管理體系）。
- 實作規格（歷史提案）：`.kilo/plans/1793585665396-session-file-management.md`。
- Session registry：`.kilo/sessions/<session-id>/`（manifest + 小型摘要）。
- 專案產物：`<project>/artifacts/<session-id>/{conversation,tests,runs,io/}`。
- 輔助腳本：`scripts/session-start.ps1`、`session-end.ps1`、`session-validate.ps1`、`session-cleanup.ps1`。

---

*本文件為工作區最高導航與憲政文件，與子專案 AGENTS.md 互補，不取代子專案專屬規範。*
