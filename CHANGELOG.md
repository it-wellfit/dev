# Changelog — Dev 根工作區

本檔記錄根 workspace（`core/`、`platform/`、`web/`、`crates/flaps_bridge`）的實際變更。格式參考 Keep a Changelog。子專案（ekpi-platform、Wellfit、SyncMesh）各自有變更記錄。

## [Unreleased] — 2026-11-04（第二輪：複式記帳＋認證）

### Added
- **借貸平衡強制（P1-5，INV-02）**：`validate_journal_entry` 強制 ΣDEBIT = ΣCREDIT 且 >0（i128 累加）；表單升級為借方/貸方雙欄（同科目拒絕）；交易清單金額改為借方合計語義。
- **認證與會話（G-1，ADR-0003）**：`platform/src/auth.rs`——Argon2id 密碼雜湊、伺服器端會話（24h、可撤銷）、`require_session` middleware、登入/登出（PRG）、同步器 CSRF token（登出＋交易表單）、首次啟動管理員播種（`FLAPS_ADMIN_PASSWORD` 或隨機一次性列印）。
- `migrations/0002_users_sessions.sql`（users＋sessions 表）。
- 登入頁 `platform/templates/login.html`（零 JS，錯誤泛化防枚舉，密碼永不回顯）。
- 引擎負向測試 8 條＋整合測試 13 條（未認證重導、CSRF 403、登出撤銷、RBAC 403、平衡端到端）。

### Changed
- 身分來源改為伺服器端會話（`Extension<Session>`）；**移除可偽造的 `X-Acting-User` header**。
- `AppState`/`LedgerEngine`/`PermissionManager` 加 `Clone`（router state 型別需求）。
- 路由拆分 public/protected（`require_session` middleware）。
- `web/index.html` 表單同步升級為借貸雙欄（靜態基線與模板一致）。

### Security
- THREAT_MODEL：T-01 已緩解（認證落地）、T-08 已緩解（CSRF token）、R-06 Resolved；新增 R-07（Secure cookie 待 TLS）。

## [Unreleased] — 2026-11-04（第一輪：整數貨幣＋API 層）

### Added
- **零 JS API 層（ADR-0002）**：`platform/src/bin/flaps-server.rs`（Axum 0.7 伺服器）與 `platform/src/api.rs`（路由與處理器）：
  - `GET /`：Askama 0.16 伺服器渲染全頁（模板 SSOT：`platform/templates/index.html`，自動 HTML escaping）。
  - `GET /assets/design-system.css`：編譯期嵌入（零路徑遍歷面）。
  - `GET /api/health`：JSON 探針（db 連線狀態＋版本）。
  - `POST /transactions`：表單 → RBAC（`LEDGER_WRITE`）→ 驗證 → 冪等登帳 → 303 PRG；失敗回完整頁面並保留表單值（400/403/500 分級）。
- **整數貨幣模組（P0-1，INV-01）**：`core/src/money.rs`——`parse_decimal_to_minor`（精確字串解析，拒 >2 位小數／負數／溢位，繁中錯誤訊息）與 `format_minor`（純整數千分位格式化）。
- `LedgerEngine::with_pool`：不建表的推薦建構（schema SSOT 為 migrations）；`query_recent_transaction_summaries`（JOIN 彙總金額）。
- `TransactionSummary`（header＋items 彙總，供清單渲染）。
- 平台整合測試 8 條（PRG 303、RBAC 403、驗證 400 ×3、零 JS 渲染、health、CSS）＋遷移測試 4 條（建表/重放冪等/舊庫相容/整數 roundtrip）。
- `docs/adr/ADR-0001-tech-stack-baseline.md`、`docs/adr/ADR-0002-api-layer.md`。
- 治理文件：根 `AGENTS.md`、`docs/product/PRD.md`、`docs/product/USER_JOURNEYS.md`、`docs/product/IMPROVEMENT_BACKLOG.md`、`docs/security/THREAT_MODEL.md`、`docs/testing/TEST_STRATEGY.md`、`docs/engineering/SKILL_INVENTORY.md`、本檔。

### Changed
- **金額整數化（P0-1）**：`TransactionItem.amount_minor: i64`（原 `amount: f64`）、`exchange_rate_scaled: i64`（原 `exchange_rate: f64`）；migrations 0001 schema 改為 `amount_minor INTEGER NOT NULL`、`exchange_rate INTEGER DEFAULT 10000`；`core/src/ledger/engine.rs::ensure_tables` DDL 同步。
- 依賴安全：`reqwest` 0.11 → 0.12.28（h2 0.3.27 → 0.4.20，修復 RUSTSEC-2026-0258；移除 unmaintained 的 rustls-pemfile）。
- 根 `Cargo.toml`：移除 `[workspace.lints.clippy] all/pedantic = "allow"`（憲章 §2.2 零警告不變量）。
- `web/index.html`：標頭註記 Superseded——伺服器渲染 SSOT 為 `platform/templates/index.html`；本檔保留為離線設計基線。
- `docs/design-system.css`：頭註部署副本關係（SSOT：`web/assets/design-system.css`）。
- `SyncMesh/docs/SECURITY.md` v1.1.0：反模擬更正（TCP 明文現狀；TLS 1.3 為目標、AT-008 未驗證）。
- `docs/PLAN_REGISTER.md`：2026-11-03 addendum（基線恢復與 ADR-0001）。

### Fixed
- `core/src/permission.rs`：`initialize_default` 缺失閉合括號導致 workspace 無法編譯（語法損壞）。
- 19 個 clippy 錯誤（redundant_pattern_matching、needless_borrows_for_generic_args、too_many_arguments（帶理由豁免）、new_without_default、unnecessary_cast、for_kv_map、collapsible_match、manual_div_ceil、is_multiple_of）。
- flaky 測試 `batching_beats_per_call_under_measured_envelope`：牆鐘比較改為確定性往返計數（MockTransport 新增 `round_trips()`）。

### Security
- `cargo audit` EXIT=0：h2 升級修復 RUSTSEC-2026-0258；rsa RUSTSEC-2023-0071 經 `.cargo/audit.toml` 文件化接受（sqlx-mysql 未啟用、未編入產物）。
- 伺服器端 RBAC 強制＋輸入驗證 fail-closed（THREAT_MODEL T-01 部分緩解、T-02/T-06 控制）。

### Known Limitations（誠實記錄）
- 根 workspace 非 git repo（P0-2）：證據為指令＋退出碼綁定，非修訂版綁定。
- `X-Acting-User` header 未經認證（G-1/R-06）：對外開放前必須落地認證會話。
- 表單建立單筆分錄，未強制借貸平衡（P1-5）。
