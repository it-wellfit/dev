# Threat Model — FlapsErp RPA 支援線（根 workspace）

> 版本：v1.0｜日期：2026-11-03｜狀態：Active
> 方法：STRIDE（參考 OWASP ASVS 5.0／OWASP Top 10:2025 摘要）。範圍：根 workspace 的 `web/` 表現層、`platform/` 服務層、`core/` 領域層、SQLite 本地儲存。ekpi-platform、Wellfit、SyncMesh 另有各自安全文件。
> 證據規則：本文件僅記錄「已實作（附代碼路徑）」與「未實作（列為缺口）」，禁止將目標態寫成現狀（P7 反模擬）。

## 1. 資產（Assets）

| 資產 | 說明 | 價值 |
|------|------|------|
| 帳務資料 | transactions／transaction_items／accounts（SQLite） | 高：財務正確性 |
| 稽核紀錄 | audit_log（事件級） | 高：不可抵賴 |
| RBAC 對應 | roles／permissions／role_permissions／user_roles | 高：授權邊界 |
| ERP 設定 | erp_config／config_history（含 is_sensitive 欄位） | 中高 |
| COM 通道 | flaps_bridge（real-com，EULA-gated） | 高：可操作真實 ERP |
| 機密 | 設定中的金鑰／憑證（is_sensitive=1） | 高 |

## 2. 信任邊界（Trust Boundaries）

1. **瀏覽器 ↔ Axum API 層**（API 層尚未建置，列為缺口 G-1；建置時此邊界為首要防線）。
2. **服務層 ↔ SQLite**（本機檔案）。
3. **服務層 ↔ 外部 AI API**（reqwest；資料本機優先原則 C-08，雲端回退需資料擁有者書面例外）。
4. **服務層 ↔ 輔翼 ERP**（COM；僅在 real-com 啟用且取得書面授權後成立）。

## 3. STRIDE 分析與現狀控制

| # | 威脅（STRIDE） | 攻擊面 | 現狀控制（已實作） | 缺口（未實作） |
|---|----------------|--------|--------------------|----------------|
| T-01 | Spoofing：偽造使用者呼叫 API | 邊界 1 | **已緩解（2026-11-04，ADR-0003）**：Argon2id 登入＋伺服器端會話（可撤銷、24h 過期）＋cookie HttpOnly/SameSite；acting user 一律來自會話，RBAC 伺服器端強制，登帳寫入 audit_log | **G-1 殘餘**：登入速率限制未實作（對外開放前必須；BACKLOG）；`Secure` cookie 待 TLS 前置（R-07） |
| T-02 | Tampering：SQL 注入竄改帳務 | 查詢建構 | `query_transactions` 使用 QueryBuilder push_bind（參數化）；餘為 bind 參數 | 契約測試補齊（TEST_STRATEGY TC-SQL-01） |
| T-03 | Repudiation：否認操作 | audit_log | 登帳寫入 TRANSACTION_POSTED 事件（engine.rs）；操作者欄位存在 | 讀取／刪除操作未寫 audit（config delete 有 history；ledger 讀取無） |
| T-04 | Information Disclosure：敏感設定外洩 | erp_config.is_sensitive | 欄位存在 | 未實作遮罩／加密存取；log 未統一遮罩機制 |
| T-05 | Denial of Service：鎖死／資源耗盡 | SQLite | WAL＋busy_timeout 5000ms＋事務 ≤50ms 設計（INV-03） | 連線池上限未設文件化；API 限流隨 G-1 |
| T-06 | Elevation of Privilege：越權（ID→code 混淆） | PermissionChecker | 已修正為 permission_code 判定＋回歸測試（permission.rs tests） | 物件級授權（row-level）未實作；零 JS 介面不含授權邏輯（伺服器強制） |
| T-07 | XSS：注入腳本於表現層 | web/index.html | 零 JavaScript（FR-001）＋無內聯事件；伺服器渲染經 Askama 自動 escaping（ADR-0002，OWASP 建議：跳脫 `< > & " '`） | —（靜態頁無動態插值；G-1 落地時引入 CSP） |
| T-08 | CSRF：跨站偽造表單 | 邊界 1 | **已緩解（2026-11-04，ADR-0003）**：同步器 token（session 綁定）寫入寫入表單並於 POST 驗證（403）＋SameSite=Lax cookie | —（整合測試覆蓋偽造 token 拒絕） |
| T-09 | 供應鏈：惡意依賴 | Cargo.lock | `cargo audit` EXIT=0（.cargo/audit.toml 記錄 rsa 接受理由）；鎖定版本 | cargo-deny 授權／來源白名單未建 |
| T-10 | 機密外洩至版控 | repo | SHARED_STANDARDS §7.1 禁令＋check-secrets（專案腳本） | 根 workspace 未初始化 git，掃描排程隨之（P0-2） |

## 4. AI 專屬控制（對應 FR-008）

1. AI 禁區（金額修改、權限變更、交易作廢、科目變更、結帳）由規則引擎強制，不經 AI 決策路徑。
2. 決策閘門三段（<0.3 自動通過／0.3–0.7 人工覆核／≥0.7 拒絕＋告警）必須記錄至 audit_log（含模型版本、分數、理由）。
3. 資料本機優先（C-08）：除經資料擁有者書面例外，ERP 資料不得送雲端推論。

## 5. 殘餘風險（Residual Risks）

| 編號 | 風險 | 處置 | 狀態 |
|------|------|------|------|
| R-01 | f64 貨幣（T-02 變體：精度竄改） | IMPROVEMENT_BACKLOG P0-1；修復前登帳不得用於真實會計 | Open |
| R-02 | rsa RUSTSEC-2023-0071（經 sqlx-mysql，未編入） | `.cargo/audit.toml` 文件化接受 | Accepted |
| R-03 | 無 git 版控（無法綁修訂版審計） | Backlog P0-2 | Open |
| R-04 | COM 通道未經 EULA 授權即啟用 | feature-gated＋測試 `real_com_not_active_by_default` 防呆 | Controlled |
| R-05 | 匯率 NUMERIC affinity 歧義（< 0.1 無小數 integer 值無法區分是否已縮放） | migrations 0001 註解＋engine 拒非正匯率；本工作區無生產資料 | Accepted |
| ~~R-06~~ | ~~X-Acting-User header 未經認證~~ | **Resolved（ADR-0003）**：身分一律來自伺服器端會話，header 已移除 | Resolved |
| R-07 | Cookie 缺 `Secure` 屬性（本機 HTTP 部署） | 前置 TLS 時必須加 `Secure`；列入部署檢查清單 | Open |
| R-06 | `X-Acting-User` header 未經認證（G-1） | RBAC 仍伺服器強制＋audit_log 記錄宣告身分；**對外開放前必須落地認證**（T-01） | Open |

## 6. 審查節奏

每季或重大變更（G-1 API 層落地、real-com 啟用、AI 介接）後 7 日內複審；複審記錄加於本文件版本歷史。
