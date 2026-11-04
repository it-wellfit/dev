# ADR-0003: 認證會話與 CSRF 防護（G-1 落地）

| 欄位 | 內容 |
|------|------|
| ADR 編號 | ADR-0003 |
| 標題 | 最小正確認證：Argon2id＋伺服器端會話＋同步器 CSRF token |
| 狀態 | Accepted |
| 決策者 | 工作區擁有者（依《ELITE PRODUCT ENGINEERING MASTER DIRECTIVE》用戶授權「繼續完善所有」，本 session 前輪已建議 G-1 為首要安全缺口） |
| 日期 | 2026-11-04 |
| 關聯 | ADR-0002、THREAT_MODEL T-01／T-08／R-06、PRD FR-006、SHARED_STANDARDS §7.2 |

## 1. 背景與問題

ADR-0002 的 `X-Acting-User` header **未經認證**（R-06）：任何人可自稱 admin。THREAT_MODEL T-01 維持 Open，R-06 明列「對外開放前必須落地認證」。需要零 JS 相容（表單 POST＋PRG）、可撤銷、可稽核的最小認證層。

## 2. 考慮的選項

| 決策點 | 選項 A（採用） | 選項 B（不採用） | 理由 |
|--------|---------------|------------------|------|
| 密碼雜湊 | Argon2id（`argon2` crate 0.5.3 預設參數，PHC 格式） | bcrypt／SHA-256 | OWASP 首選；ekpi-platform 同選擇；抗 GPU |
| 會話存放 | 伺服器端 `sessions` 表（可撤銷、過期檢查） | JWT 自包含 token | 撤銷（登出即失效）與稽核需要伺服器狀態；JWT 無法單方面撤銷 |
| Cookie | `flaps_session` 僅含隨機 opaque id；`HttpOnly; SameSite=Lax; Path=/; Max-Age=24h` | JWT／明文 user id | 最小資訊暴露；JS 不可讀（降低 XSS 竊取面） |
| CSRF | 同步器 token（session 綁定，寫入表單 hidden 欄位，POST 驗證） | 僅 SameSite=Lax | Lax 已擋跨站 POST，token 提供縱深防禦（T-08） |
| 首次管理員 | 啟動時無使用者才建立；密碼來自 `FLAPS_ADMIN_PASSWORD` 或**隨機生成僅列印一次** | 硬編碼預設密碼 | 預設憑證是經典漏洞（CWE-798） |
| 登入失敗訊息 | 泛化訊息（不區分帳號/密碼錯誤） | 精確提示 | 防帳號枚舉（測試涵蓋「密碼不回顯」） |

## 3. 決策

1. **路由拆分**：public（`/login` GET+POST、嵌入 CSS、`/api/health`）與 protected（`/`、`POST /transactions`、`/logout`），protected 由 `require_session` middleware 守衛（未認證 → 303 `/login`）。
2. **身分來源**：acting user 一律來自會話（`Extension<Session>`）——**`X-Acting-User` header 已移除**，偽造面消除。
3. **CSRF**：`TransactionForm`/`LogoutForm` 均含 `csrf_token`，驗證失敗 → 403＋說明頁（含「重新整理後再送出」指引）。
4. **會話生命週期**：24 小時固定；登出即刪除（伺服器端撤銷）；過期查詢於每次請求。
5. **登出**：POST（CSRF-checked）→ 刪會話＋清 cookie＋303（PRG 語意一致）。

## 4. 實施與正面結果（2026-11-04 實測）

| 驗證 | 結果 |
|------|------|
| `cargo test --locked` | EXIT=0；**platform 17 passed**（新增：未認證重導、登入失敗泛化＋密碼不回顯、CSRF 偽造 403、登出撤銷後拒絕、store 帳號 RBAC 403、端到端平衡登帳） |
| clippy/fmt/audit | 全部 EXIT=0 |
| 煙霧（curl.exe） | 未認證 303→/login；登入 303→/＋Cookie；頁面含 user-admin＋CSRF 欄位 ×2；偽造 token 403；正確 token 平衡登帳 303→/#transactions；登出後舊 cookie 被拒；登入頁不含密碼字串 |

## 5. 負面結果與風險

1. **R-07（新）**：Cookie 未加 `Secure`——本機 HTTP 部署所需；前置 TLS 後必須加上（部署檢查清單項目）。
2. **無登入速率限制**：暴力破解防護未實作（本機單機部署風險低；對外開放前加 per-username/IP 失敗計數）——列 backlog。
3. **會話固定**：登入成功即**新建** session id（不含 cookie 固定向量），風險已由流程設計規避；登入後 rotation 不再額外執行。
4. **帳號管理 UI 未建**：使用者新增/停用目前需直接操作 DB（管理員功能列 backlog）。

## 6. Follow-up Actions

1. THREAT_MODEL：T-01 → 已緩解（認證落地）、T-08 → 已緩解（CSRF token）、R-06 → Resolved、新增 R-07（Secure cookie）。
2. 部署檢查清單：前置 TLS＋`Secure` 屬性＋登入速率限制。
3. 使用者管理介面（ROLE_MANAGE 權限已存在）。
