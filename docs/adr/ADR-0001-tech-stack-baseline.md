# ADR-0001: 技術棧基線裁定與品質基線恢復

| 欄位 | 內容 |
|------|------|
| ADR 編號 | ADR-0001 |
| 標題 | 技術棧基線裁定與品質基線恢復（Dev 根 workspace） |
| 狀態 | Accepted |
| 決策者 | 工作區擁有者（依《ELITE PRODUCT ENGINEERING MASTER DIRECTIVE》執行計畫 D-01~D-05 建議值，用戶未提出異議） |
| 日期 | 2026-11-03 |
| 驗證證據 | 本 workspace 根目錄非 git repo，證據綁定：指令＋工作目錄＋退出碼＋session 記錄（`.kilo/sessions/20261103-dev-elite-impl/`）；無 git 修訂版可引用，此為已知治理缺口（見 §7） |

---

## 1. 背景與問題（Context & Problem）

1. **記憶與程式碼不一致**：專案記憶（project.md）宣稱 Rust 2024／Resolver 3／Axum 0.8／Askama 0.16／SQLx 0.9／PostgreSQL 18／Strict Zero-JS；實際根 workspace 為 edition 2021／resolver 2／axum 0.7／sqlx 0.8 SQLite／tauri 2.0，且 `web/index.html` 含約 500 行內嵌 JavaScript。若以記憶版本為準新增依賴或架構決策，會導致建置失敗與不相容。
2. **品質基線失效**：`cargo fmt --check` 因 `core/src/permission.rs` 語法損壞（`initialize_default` 缺少閉合 `}`）無法執行；`cargo clippy -D warnings` 有 19 個錯誤；根 `Cargo.toml` 含 `[workspace.lints.clippy] all = "allow"`，與憲章 §2.2 零警告不變量矛盾。先前「all pass」宣稱（debugging-summary）不成立。
3. **依賴安全**：`cargo audit` 報告 RUSTSEC-2026-0258（h2）與 RUSTSEC-2023-0071（rsa）。
4. **產品方向分散**：五個子專案無單一主產品裁定。

## 2. 考慮的選項（Considered Alternatives）

| 決策點 | 選項 A（採用） | 選項 B（不採用） | 理由 |
|--------|---------------|------------------|------|
| 主產品（D-01） | ekpi-platform/OEKAP 為主產品；根 workspace 定位 FlapsErp RPA 支援線 | 以根 workspace（ARES）為主產品 | ekpi-platform 是唯一具備 Clean Architecture＋Strict Zero-JS＋PostgreSQL 棧與 T1–T10 完成紀錄的候選；PLAN_REGISTER 顯示根 workspace 計畫僅為 proposal |
| 根 workspace 技術棧（D-02） | 維持 edition 2021／resolver 2／axum 0.7／SQLite；升級列為後續 POC | 立即遷移 Rust 2024／Axum 0.8／PostgreSQL | 無明確收益的遷移引入建置與相容性風險；違反「最少合理複雜度」原則 |
| 前端路線（D-03） | 純 HTML/CSS 零 JS＋表單 PRG；Tauri 殼保留 | 保留 JavaScript／引入 HTMX/WASM | 憲章 §2.3 表現層 100% JavaScript-Free 為剛性不變量；Node.js 未安裝，無前端建構工具鏈 |
| `web/index.html` 範圍（D-04） | 先重構單頁為零 JS 語義頁面 | 新增多頁面路由 | 最小可逆變更，快速驗證零 JS 可行性 |
| SyncMesh 宣稱（D-05） | 文件更正為「TCP-only」，TLS 列為技術債 | 補 TLS 實作 | 反模擬原則（P7）：虛假安全宣稱比無 TLS 更危險 |
| h2 漏洞 | 升級 reqwest 0.11→0.12（h2 0.3.27→0.4.20） | 接受風險 | 修復版存在且 reqwest 0.12 API 相容；一併移除 unmaintained 的 rustls-pemfile |
| rsa 漏洞 | 文件化接受（`.cargo/audit.toml` ignore＋理由） | 移除依賴 | rsa 僅經由 sqlx-mysql 引入；本 workspace 僅啟用 SQLite feature，未編入任何產物，暴露面為零；上游無修復版 |

## 3. 決策（Decision）

1. **主產品**：ekpi-platform/OEKAP。根 workspace 的 `core/`、`platform/`、`web/`、`crates/flaps_bridge` 定位為 FlapsErp RPA 支援線；`flaps_bridge` 的 `real-com` feature 維持 EULA-gated，未取得 ERP 廠商書面授權（RSK-E1）前不得啟用。
2. **根 workspace 技術棧基線**（不變）：Rust edition 2021、Cargo resolver 2、axum 0.7、tokio 1.0、sqlx 0.8（SQLite，WAL）、tauri 2.0、windows 0.58。升級至 2024/Axum 0.8/PostgreSQL 列為獨立 POC，需先有明確收益證據。
3. **表現層**：`web/index.html` 重構為零 `<script>` 的語義 HTML＋CSS Design Token 頁面；狀態修改僅經 POST＋PRG。
4. **品質基線恢復**：移除根 `Cargo.toml` 的 `[workspace.lints.clippy] all/pedantic = "allow"`；以 `cargo clippy --all-targets -- -D warnings` 為門檻。**注意**：目前各 crate 未透過 `[lints] workspace = true` 接入 workspace lints，零警告強制依靠命令列參數（CI 需沿用）；未來接入 workspace lints 時不得恢復 allow。
5. **依賴安全處置**：reqwest 升級 0.12.28（h2 0.4.20）；rsa 經 `.cargo/audit.toml` 記錄為接受風險（理由：未編入）。

## 4. 實施與正面結果（Positive Consequences）

以下均為 2026-11-03 實測（工作目錄：workspace 根；無 git 修訂版，綁定 session 記錄）：

| 檢查 | 指令 | 結果 |
|------|------|------|
| 格式 | `cargo fmt -- --check` | 退出碼 0（先以 `cargo fmt` 修復格式＋`permission.rs` 語法錯誤） |
| Lint | `cargo clippy --all-targets -- -D warnings` | 退出碼 0（修復 19 個錯誤：config.rs 3＋2、engine.rs 4、transaction.rs 2（`#[allow(too_many_arguments)]` 帶理由）、permission.rs 7、search.rs 1、model_orchestrator.rs 5） |
| 測試 | `cargo test --locked` | 退出碼 0；9 passed / 0 failed（flaps_bridge 7、flaps_core 2） |
| 安全 | `cargo audit` | 退出碼 0（h2 修復；rsa 文件化接受，證據 `.cargo/audit.toml`） |

## 5. 負面結果與風險（Negative Consequences & Risks）

1. **f64 貨幣技術債（高）**：`TransactionItem.amount: f64`／`exchange_rate: f64` 違反 INV-01（零浮點貨幣）與 SHARED_STANDARDS §1.1（財務欄位禁浮點）。本 ADR 不修復（型別變更牽動 schema 與所有呼叫端），已列 `docs/product/IMPROVEMENT_BACKLOG.md` 首位；在修復前，任何金額運算結果不得用於真實會計用途。
2. **`#[allow(too_many_arguments)]`**：兩處建構子使用帶理由的豁免；builder 重構列為 backlog。
3. **無 git 版控**：根 workspace 非 git repo，P2 修訂綁定無法完全滿足；建議初始化 git（列為 backlog，屬不可逆性低的改變但需擁有者確認）。
4. **sqlx 雙建表路徑**：`core/ledger/engine.rs::ensure_tables` 與（後續 Slice 4 引入的）`platform/migrations/` 並存；遷移檔以 `IF NOT EXISTS` 保持冪等，兩路徑收斂列為 backlog。

## 6. 後續行動（Follow-up Actions）

1. Slice 2：`web/index.html` 零 JS 重構＋實機驗證（本 ADR D-03/D-04 的執行）。
2. Slice 4：`platform/migrations/` 版本化遷移＋`DatabaseManager::initialize()` 接線。
3. CI 工作流沿用 `cargo clippy --all-targets -- -D warnings`（不得加 allow）。
4. 將本 ADR 登錄至 `docs/PLAN_REGISTER.md` 之 ADR 索引（若存在）。

## 7. 關聯文件

- `WORKSPACE_CHARTER.md` §2.2／§2.3（不變量來源）
- `docs/standards/SHARED_STANDARDS.md` §6（品質門檻）、§9（ADR 格式）
- `.kilo/plans/1793708660255-elite-product-engineering-execution-plan.md`（執行計畫）
- `.cargo/audit.toml`（rsa 漏洞處置）
- `.kilo/sessions/20261103-dev-elite-impl/runs/`（clippy/audit 前後對照日誌）

## 8. 2026-11-12 修訂（D-02 之 edition 基線已更新）

- **變更**：根 workspace 之 Rust edition 由 **2021 → 2024**（6 個成員 crate 全部；`web/src-tauri` 亦同）。Cargo resolver 維持 `2`（2024 edition 預設即 resolver 2）。
- **理由**：2024 為當前穩定 edition（SOTA），可取得 let-chains、`gen` 關鍵字保留、RPIT 生命週期捕捉修正等語言改進；`ekpi-platform` 已於 2024。
- **成本/風險**：實測**零程式碼變更需求**（`cargo check --workspace --all-targets` 直接通過）；新增 7 處 `clippy::collapsible_if`（let-chain 建議）與 rustfmt `style_edition = 2024` 之 import 排序差異，已以 `cargo clippy --fix`＋`cargo fmt` 修正。
- **驗證（2026-11-12 實測）**：fmt EXIT=0；clippy `--all-targets --locked -D warnings` EXIT=0；`cargo test --locked` **36 suites／360 passed／0 failed**（排除環境敏感之效能斷言；該斷言單獨跑通過）。
- **不變**：axum 0.7、SQLite、resolver 2、tokio 1.x 之基線維持（升 axum 0.8 仍屬獨立 POC，見 §3 D-02）。
