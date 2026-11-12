# 安全（Security）— Dev 根工作區

- 版本：v1.0｜生效日期：2026-11-12｜適用範圍：根 workspace 與桌面安裝包｜狀態：Active
- 規則 SSOT：`docs/standards/SHARED_STANDARDS.md` §7；決策 SSOT：`docs/adr/ADR-0014-*.md`

## 1. 範圍與信任邊界

| 邊界 | 說明 |
|---|---|
| 瀏覽器 ↔ platform | 本機 HTTP；工作階段 cookie（`auxbridge_session`，HttpOnly、SameSite=Lax） |
| 平台 ↔ 原廠 ERP | 僅 config／資料串接；不改原廠內容（`NAMING_AND_BOUNDARY_DECREE`） |
| 平台 ↔ 更新伺服器 | HTTPS + minisign 簽章驗證（`requireSignedVersion` 不可停用） |
| 更新伺服器 | **尚未建置**（`updates.auxbridge.internal`）→ 現階段無遠端更新通道 |

## 2. 密鑰與簽章

| 項目 | 規則 |
|---|---|
| 更新簽章私鑰 | 存於 `%USERPROFILE%\.tauri\flaps-erp\flaps-erp.key`（不得進版控） |
| 私鑰密碼 | 生產前必須輪換（語義值見 `docs/release/UPDATER_RUNBOOK.md` §1） |
| 工作階段秘密 | `AUXBRIDGE_ADMIN_PASSWORD` 由環境注入；種子密碼僅供開發 |
| 資料庫 | 開發/安裝版為本機 SQLite（單機單使用者） |

## 3. 平台安全控制

- 認證：Argon2id 密碼雜湊、工作階段過期與撤銷、連續失敗鎖定（`platform/src/auth.rs`）。
- 授權：RBAC（`core/src/permission/`）、fail-closed、最小權限。
- 輸入：邊界驗證（fail-fast）、輸出編碼（HTML/XML/SQL）。
- 傳輸：對外 TLS 1.2+；本機開發例外需記錄。

## 4. 供應鏈

- 弱點：`cargo audit`（政策 `.cargo/audit.toml`、`web/src-tauri/.cargo/audit.toml`）。
- 授權/來源：`cargo deny check`（`deny.toml`）。
- SBOM：7 份 CycloneDX 1.5（`docs/release/sbom/`），依賴變更後必須重產。
- 未用依賴：`cargo machete`（逐成員）於 CI 執行。

## 5. 已知缺口（誠實揭露）

| 缺口 | 影響 | 狀態 |
|---|---|---|
| 更新 feed 主機未建置 | 無法端到端驗證自動更新 | BLOCKED |
| Authenticode 未購證 | 安裝包無發行簽章（SmartScreen 警告） | OPEN |
| 私鑰密碼未輪換 | 值以語義記錄於手冊 | OPEN |
| `cargo vet` 未採用 | 無供應鏈人工稽核層 | 依 ADR-0014 D4 暫緩 |

## 6. 變更紀錄

| 日期 | 版本 | 說明 |
|---|---|---|
| 2026-11-12 | v1.0 | 依 SHARED_STANDARDS §4.1 建立；同步 ADR-0014 決定 |