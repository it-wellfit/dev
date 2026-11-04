# 共享開發標準 (Shared Development Standards)

**版本**: v1.0.0  
**生效日期**: 2026-10-25  
**適用範圍**: 全工作區所有子專案 (ekpi-platform, Wellfit, SyncMesh)  
**狀態**: Active

> **管轄範圍釐清 (2026-11-02)**：本文件為**工程基線**（編碼、建置、測試、安全、操作）的 owner。
> 檔案／文件／知識／工程**管理體系**（session 產物、文件生命週期、知識治理、營運節奏、演進機制）以
> [GOVERNANCE_STANDARD.md](GOVERNANCE_STANDARD.md) 為唯一 canonical 標準，本文件不重複其內容。

---

## 1. 編碼標準 (Coding Standards)

### 1.1 Rust 通用規則
- 所有 Rust 原始碼檔案第一行強制宣告 `#![forbid(unsafe_code)]`，零 `unsafe` 區塊容忍。
- `cargo clippy --all-targets -- -D warnings` 必須編譯期零警告。
- 所有 `public` API 必須有文件註解 (`///` 或 `//!`)，包含參數、回傳值、錯誤條件與範例。
- 禁止 `assert!(true)` 偽斷言，斷言必須綁定具名變數並包含有意義的訊息。
- 使用 `serde` 進行序列化時，必須顯式指定 `serde(rename_all = ...)` 與 `serde(skip_serializing_if = ...)`，避免隱藏欄位。
- 錯誤處理使用 `thiserror` 與 `anyhow`，區分可恢復錯誤 (`Result`) 與不可恢復錯誤 (`panic!` 僅限程式錯誤)。
- 所有數字貨幣或財務欄位使用整數類型 (如 `i64` 代表分)，嚴禁浮點數。

### 1.2 命名規則
- Crate 名稱：kebab-case (例如 `wellfit_core`, `syncmesh_net`)。
- 模組、檔案：snake_case (例如 `mtrix_engine.rs`, `conflict.rs`)。
- 類型、結構體、列舉：PascalCase (例如 `MtrixRow`, `ConflictResolution`)。
- 函數、方法、變數：snake_case (例如 `parse_mtrix_row`, `validate_signature`)。
- 常數：SCREAMING_SNAKE_CASE (例如 `MAX_BATCH_SIZE`, `DEFAULT_TIMEOUT_SEC`)。
- Trait：PascalCase，描述能力而非實作 (例如 `Syncable`, `Validatable`, `Exportable`)。

### 1.3 註解標準
- 公開項目必須有文件註解，說明目的、參數、回傳、錯誤、範例。
- 內部邏輯使用 `//` 行註解，說明「為什麼」而非「做什麼」。
- 所有 `TODO`、`FIXME`、`HACK` 必須附帶 GitHub issue 編號或負責人與預計日期。
- 公開 API 的 breaking change 必須在文件註解中標記 `[Breaking]` 與移除版本。

---

## 2. 建置標準 (Build Standards)

### 2.1 Cargo 工作區
- 每個子專案獨立管理 Cargo.toml 與 Cargo.lock，不假設根目錄 Cargo workspace (ekpi-platform 明確無根 workspace)。
- `rust-toolchain.toml` 固定 toolchain channel，ekpi-platform 為 1.99.1。
- 建置前必須先 `cargo fmt -- --check`，格式化無誤後才可提交。
- 建置命令：`cargo build --locked` (ekpi-platform) 或 `cargo build --workspace --locked` (Wellfit, SyncMesh)。
- 發布建置：`cargo build --release --locked`，Windows 執行前必須終止殘留 `.exe` 守護程序。

### 2.2 依賴管理
- 所有依賴必須鎖定版本 (Cargo.lock 提交至版本控制)。
- 新增依賴前必須執行 `cargo deny check` (若專案有 `deny.toml`) 或等效的安全審查。
- 嚴禁引入未審查的 transitive 依賴導致供應鏈風險。
- 生產建置不得包含僅用於開發的依賴 (dev-dependencies 不得洩漏至 release 建置)。

---

## 3. 測試標準 (Testing Standards)

### 3.1 測試分類
| 類型 | 範圍 | 工具 | 最小覆蓋 |
|------|------|------|----------|
| 單元測試 | 單一模組/函數 | `cargo test --lib` | 所有 public 函數 |
| 整合測試 | 跨模組路徑 | `cargo test --test` | 主要使用者旅程 |
| E2E 測試 | 完整使用者旅程 | CLI 整合測試 | 關鍵業務流程 |
| 契約測試 | API 契約驗證 | schema/route 測試 | 所有公開契約 |
| 安全測試 | 授權、輸入驗證 | 針對性負向測試 | 高風險變更 |

### 3.2 測試規則
- 測試必須可重複執行，不依賴時間、隨機數或外部網路 (除非明確標記為 integration)。
- 測試必須驗證成功路徑與失敗/拒絕路徑 (反向驗證)。
- 每個 bug 修正必須先加入回歸測試，再修正程式碼 (TDD 原則)。
- 測試必須具名，使用描述性名稱 (例如 `test_mtrix_row_rejects_invalid_column_count`)。
- 禁止斷言恆真 (`assert!(true)`)，必須驗證具體期望值。
- 合成測試永遠不得使目標驗收案例 (AT-*) 通過；`Implemented` 列不等於交付能力。

### 3.3 執行標準
- 執行前必須 `cargo fmt -- --check` 與 `cargo clippy --all-targets -- -D warnings`。
- 執行 `cargo test --locked`，所有測試必須通過 (0 個失敗、0 個忽略、0 個被忽略的測試)。
- CI 必須在 Ubuntu 與 Windows 兩平台執行相同檢查。

---

## 4. 文件標準 (Documentation Standards)

### 4.1 文件結構
每個專案必須包含以下文件 (見 `WORKSPACE_CHARTER.md` §4)：
1. `PROJECT_CHARTER.md` — 專案概覽、目標、範圍、非目標
2. `ARCHITECTURE.md` — 系統架構、資料流、元件關係
3. `API_SPEC.md` — API/CLI 契約與介面
4. `DEVELOPMENT_GUIDE.md` — 建置、測試、部署、本機檢查入口
5. `QUALITY_STANDARDS.md` — 風險分級、品質門檻、定義完成
6. `OPERATIONS_RUNBOOK.md` — 備份、還原、監控、疑難排解
7. `SECURITY.md` — 安全強化、機密管理、權限模型
8. `ADR_REGISTER.md` — 架構決策記錄
9. `TRACEABILITY.md` — 需求追溯與驗證證據
10. `TROUBLESHOOTING.md` — 常見問題與解決方案

### 4.2 文件格式
- 所有文件使用 Markdown，編碼 UTF-8。
- 標題層級遵循：H1 (標題) → H2 (章節) → H3 (子章節)。
- 所有表格必須有標題行，欄位對齊。
- 程式碼區塊必須標註語言 (例如 ```powershell, ```rust, ```bash)。
- 路徑引用使用相對路徑，避免絕對路徑。
- 文件須包含：版本、生效日期、適用範圍、狀態 (Active/Deprecated/Archived)。

### 4.3 文件與代碼同步
- 任何影響公眾契約、API、資料模型或配置的代碼變更，必須在同一提交中更新對應文件。
- 過期文件標記為 `[Deprecated]`，註明替代文件與日期。
- 文件變更必須經過與代碼變更相同的審查流程。

---

## 5. 版本控制標準 (Version Control Standards)

### 5.1 分支策略
- `main` 分支為生產分支，受保護，禁止直接推送。
- 功能分支命名：`feature/<issue-id>-<short-description>`。
- 修復分支命名：`fix/<issue-id>-<short-description>`。
- 發布分支命名：`release/v<major>.<minor>.<patch>`。
- 熱修復分支命名：`hotfix/<issue-id>-<short-description>`。

### 5.2 提交規則
- 提交訊息格式：`<type>: <subject>` (例如 `feat: add mtrix row validation`)。
- 類型：`feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `security`, `chore`。
- 每個提交必須關聯 GitHub Issue 編號 (在訊息正文中引用 `Fixes #<issue>` 或 `Part of #<issue>`)。
- 提交前必須 `git status` 確認工作目錄乾淨，無未提交檔案。
- 提交訊息必須包含：問題描述、變更摘要、測試證據、回滾方式。

### 5.3 標籤與發布
- 發布使用語義化版本 (SemVer)：`v<major>.<minor>.<patch>`。
- 標籤必須附註 (annotated tag)，包含發布說明與變更清單。
- 每個發布必須對應一個 GitHub Release，包含 release notes 與建置產物。

---

## 6. 品質門檻 (Quality Gates)

### 6.1 風險分級
| 等級 | 典型變更 | 所需驗證 |
|------|----------|----------|
| Low | 文件、措辭、無可觀察行為變更的內部重構 | 基線檢查，文件連結與準確性審查 |
| Medium | 使用者可見 UI、一般工作流行為、資料格式/API 契約變更、現有整合變更 | 基線檢查 + 針對性驗收行為測試、契約/架構檢查、回歸覆蓋 |
| High | 認證/授權、敏感資料、證據或版本完整性、審計歷史、核准/門禁、保留、資料移轉、外部連接器、AI 產生工作流草稿 | 基線檢查 + 負向/失敗測試、授權與 fail-closed 檢查、冪等性/重試/版本測試、移轉與回滾證據、安全/依賴/機密掃描、獨立人工審查 |

### 6.2 基線檢查 (Baseline)
每個變更必須通過：
1. `cargo fmt -- --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test --locked` (所有測試通過)
4. 專案特定的 selftest / 契約驗證命令
5. 文件連結檢查 (`scripts/check-docs` 或等效)
6. 機密模式掃描 (`scripts/check-secrets` 或等效)

### 6.3 定義完成 (Definition of Done)
工作單位完成時必須滿足所有適用條件：
1. 需求、使用者問題與成功指標已確認；開放的產品/控制決策已記錄而非臆測。
2. 接受規格與任何持久工程決策 (ADR) 已納入 Git 並從 Issue 連結。
3. 每個在範圍內的需求都有測試或明確接受的審查驗證；無未測試行為。
4. 主要案例與高風險/拒絕/失敗案例在指定修訂版中通過。
5. 基線檢查通過；風險適當的額外檢查通過；失敗/跳過檢查有負責人記錄。
6. 主要審查發現已修正並重跑受影響檢查；其餘發現有接受風險負責人。
7. 版本已提交，發布時標記標籤；已建立備份並通過還原演練 (或發布明確封鎖)。
8. 交付、操作、恢復、已知限制與移交文件皆為最新。
9. MVP 聲稱需由授權使用者執行並驗證真實工作流，不僅限於合成測試。
10. `/retro` 已執行且接受的改進已追蹤。

---

## 7. 安全標準 (Security Standards)

### 7.1 機密管理
- 絕不將密碼、API 金鑰、憑證或任何秘密提交至版本控制。
- 使用 `.env.example` 提供範本，實際 `.env` 必須列入 `.gitignore`。
- 生產環境機密透過環境變數或秘密管理系統註入，絕不硬編碼。
- 執行高信號機密模式掃描 (`scripts/check-secrets`) 作為基線檢查一部分。

### 7.2 存取控制
- 採用最小權限原則：每個元件僅存取其完成任務所需的最小資源。
- 所有存取決策必須 fail-closed (預設拒絕，明確允許)。
- 認證與授權邏輯必須獨立於業務邏輯，可單元測試。
- 會話與令牌必須有過期機制，並支援撤銷。

### 7.3 資料保護
- 輸入驗證必須在邊界執行 (fail-fast)，拒絕不符合規格的輸入。
- 輸出必須適當編碼，防止注入攻擊 (SQL, XML, HTML, 命令注入)。
- 資料傳輸必須使用 TLS 1.2+，本地原型例外需記錄。
- 儲存的敏感資料必須加密 (靜態加密)，金鑰輪換週期不超過 90 天。

---

## 8. 操作標準 (Operations Standards)

### 8.1 備份與還原
- 每個專案必須有備份程序 (`scripts/backup.ps1` 或等效)。
- 備份必須包含所有分支與標籤 (`git bundle create ... --branches --tags`)。
- 還原演練必須每月執行一次，並記錄結果。
- 備份須離線儲存，並定期驗證完整性 (`git bundle verify`)。

### 8.2 監控與可觀測性
- 所有服務必須暴露健康檢查端點 (或 CLI `status` 子命令)。
- 關鍵操作必須寫入審計日誌，包含時間戳、操作者、動作、結果。
- 錯誤必須記錄足夠上下文以支援疑難排解，但不得洩漏機密。

### 8.3 部署標準
- 部署必須可回滾，每個發布必須有明確的回滾程序。
- 生產部署前必須在預備環境通過完整測試。
- 部署必須自動化，人工介入僅限核准與緊急情況。

---

## 9. 架構決策記錄 (ADR) 標準

### 9.1 ADR 格式
每個 ADR 必須包含：
- 標題：`ADR-<NNNN>-<short-title>.md`
- 狀態：Proposed / Accepted / Deprecated / Superseded
- 決策者與日期
- 背景與問題陳述
- 考慮的選項與比較
- 決策與理由
- 結果與影響 (正面、負面、風險)
- 關聯需求與文件

### 9.2 ADR 規則
- 每個持久工程決策必須有一個 ADR。
- ADR 一經 Accepted 不得刪除，僅能標記為 Deprecated 或 Superseded，並保留歷史記錄。
- 所有 ADR 必須在 `docs/adr/` 或 `plans/adr/` 目錄中。
- ADR 必須從相關 Issue/PR 連結，保持可追溯性。

---

## 10. 工作區不變量總計 (≤ 20 條)

本節為 2.6 規則包容壓縮協議的實施，條款總數嚴格 ≤ 20：

1. Rust `#![forbid(unsafe_code)]`，零 unsafe。
2. `cargo clippy --all-targets -- -D warnings` 零警告。
3. 禁止 `assert!(true)` 偽斷言。
4. 展示層 100% JavaScript-Free。
5. PowerShell 串接使用 `;`，嚴禁 `&&`。
6. 建置前終止殘留 `.exe` 守護程序。
7. 優先 `curl.exe`，避免未加 `-UseBasicParsing` 的 `Invoke-WebRequest`。
8. 絕對反模擬，`UNKNOWN != PASS` 且 `UNKNOWN != FAIL`。
9. 規章條款總數 ≤ 20。
10. 所有 public API 必須有文件註解。
11. 貨幣/財務欄位使用整數類型，嚴禁浮點數。
12. 依賴版本鎖定，Cargo.lock 提交版本控制。
13. 新增依賴前必須安全審查。
14. 測試必須可重複，驗證成功與失敗路徑。
15. 每個 bug 修正先加回歸測試。
16. 文件與代碼同一提交更新。
17. `main` 分支受保護，禁止直接推送。
18. 提交必須關聯 GitHub Issue 編號。
19. 機密絕不提交至版本控制。
20. 備份必須包含所有分支與標籤，並定期還原演練。

---

*本標準與 `WORKSPACE_CHARTER.md` 互補，子專案 AGENTS.md 為執行入口，本文件為跨專案基準。*
