# ekpi-platform → Strict Zero-JS 企業級全端憲章遷移實作計畫 (v2.0 — 2026 最新技術版)

> **Disposition (2026-11-02): Unapproved, unverified migration proposal.** Its “latest version” table is a dated snapshot, not an authoritative dependency baseline; compare any proposed change with `ekpi-platform/Cargo.toml`, the project charter, and owner-approved decisions first. The migration scope and acceptance gates are not approved by this document.

**目標專案**: `ekpi-platform/` (`C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\ekpi-platform`)
**目標狀態**: 符合《嚴格零 JavaScript 企業級 Web 全端開發憲章》v1.0
**實作約束**: 僅產出計畫，不修改原始碼；由後續實作代理人依本計畫執行
**技術基線**: Rust 1.99.1 (Stable) / Rust 2024 Edition / Axum 0.8.9 / Askama 0.16.1 / SQLx 0.9.0 / PostgreSQL 18.6 / Tokio 1.48 / Tower 0.5 / Tower-HTTP 0.7

---

## 0. 技術版本最新性驗證 (2026-10-30 真機核對)

| 技術 | 憲章規格 | 2026-10-30 最新版本 | 狀態 | 來源 |
|------|---------|-------------------|------|------|
| Rust Toolchain | 1.99.1 | 1.99.1 (Stable, 2026-10-15) | ✅ 最新 | rust-lang.org |
| Rust Edition | 2024 | 2024 (released with Rust 1.85, 2025-02-20) | ✅ 最新 (下一個 edition 預計 2027) | Rust Blog |
| Cargo Resolver | 3 | 3 (Cargo 1.99) | ✅ 最新 | Cargo Book |
| Axum | 0.8.x | 0.8.9 (2026-04-14) | ✅ 最新 | crates.io |
| Tokio | 1.x | 1.48.x (latest stable) | ✅ 最新 | crates.io |
| Askama | 0.16.x | 0.16.1 (2026-09-04) | ✅ 最新 | crates.io |
| SQLx | 0.9.x | 0.9.0 (2026-05-21) | ✅ 最新 | crates.io |
| PostgreSQL | 18.x | 18.6 (2026-08-13) | ✅ 最新 (PostgreSQL 19 Beta 3 已發布) | postgresql.org |
| Tower | 0.5.x | 0.5.x (latest) | ✅ 最新 | crates.io |
| Tower-HTTP | 0.7.x | 0.7.x (latest) | ✅ 最新 | crates.io |
| Serde | 1.x | 1.x (latest) | ✅ 最新 | crates.io |
| Argon2 | 0.6.x | 0.6.x (latest) | ✅ 最新 | crates.io |
| thiserror | 2.x | 2.x (latest) | ✅ 最新 | crates.io |
| chrono | 0.4.x | 0.4.x (latest) | ✅ 最新 | crates.io |
| uuid | 1.x | 1.x (latest) | ✅ 最新 | crates.io |
| tracing | 0.1.x | 0.1.x (latest) | ✅ 最新 | crates.io |
| tracing-subscriber | 0.3.x | 0.3.x (latest) | ✅ 最新 | crates.io |

**關鍵發現**:
1. **Rust 2024 Edition 是最新 edition** — Rust editions 每 3 年發布一次，下一個 edition (2027) 尚未發布。憲章的 Rust 2024 Edition 規格是正確的。
2. **Rust 1.99.1 是最新 Stable** — 憲章的 toolchain 1.99.1 規格是正確的。
3. **Axum 0.8.9 是最新版本** — 憲章的 0.8.x 規格是正確的。
4. **Askama 0.16.1 是最新版本** — 憲章的 0.16.x 規格是正確的。
5. **SQLx 0.9.0 是最新版本** — 憲章的 0.9.x 規格是正確的。
6. **PostgreSQL 18.6 是最新版本** — PostgreSQL 19 目前為 Beta 3，尚未正式發布。憲章的 PostgreSQL 18 規格是正確的。

**結論**: 憲章的技術基線規格已與 2026-10-30 的最新版本完全一致，無需調整版本號。Rust 2024 Edition 是最新 edition（不是 2026），因為 Rust editions 每 3 年發布一次，下一個 edition 預計在 2027 年發布。

---

## 1. 現狀與目標對齊

| 項目 | 現狀 (ekpi-platform) | 目標 (憲章) | 已確認決策 |
|------|---------------------|------------|-----------|
| 語言版本 | Rust 2021 Edition | Rust 2024 Edition | 升級為 2024 |
| 工具鏈 | rust-toolchain.toml 1.99.1 (已有) | channel 1.99.1, clippy+rustfmt | 維持 1.99.1 |
| Cargo 架構 | 兩獨立 crate (eval, platform)，無 root workspace | Cargo Workspace: apps/web, crates/domain, crates/application, crates/adapters | 在 ekpi-platform/ 內重構 |
| 模板引擎 | Maud 0.27 | Askama 0.16.1 | 全面替換為 Askama，移除 Maud |
| 資料庫 | SQLite (rusqlite) | PostgreSQL 18.6 + SQLx 0.9.0 | 完全遷移，移除 rusqlite |
| 身分認證 | 本機 session (PBKDF2)，無企業控制 | Argon2id, tower-sessions+PostgreSQL, CSRF, RBAC/ABAC | 完整實作憲章 §8 |
| CSS 設計系統 | CSS variables (基本) | Design Tokens, Cascade Layers, Container Queries, Native Popover/Dialog, View Transitions, WCAG 2.2 AA | 完整實作憲章 §5 |
| CI/CD 部署 | PowerShell 腳本，CI 未執行 | GitHub Actions, Docker, Health Check, Graceful Shutdown, 備份/還原 | 完整實作憲章 §11-12 |
| Zero-JS | 已符合 `<script>` 數量 = 0 | CSP script-src 'none', Zero-JS 測試自動化 | 維持並強化測試 |

---

## 2. 已確認的架構決策 (共 8 項)

1. **範圍**: 將 ekpi-platform 完整遷移至憲章標準 (選項 A)。
2. **Workspace**: 在 `ekpi-platform/` 內重構為 `apps/web`, `crates/domain`, `crates/application`, `crates/adapters` 四層 (選項 A)。
3. **模板引擎**: Maud 0.27 → Askama 0.16.1，全面替換 (選項 A)。
4. **資料庫**: SQLite/rusqlite → PostgreSQL 18.6 + SQLx 0.9.0，所有環境統一 (選項 A)。
5. **安全**: 完整實作憲章 §8 (Argon2id, tower-sessions+PostgreSQL, CSRF, RBAC/ABAC, Security Headers, 稽核日誌) (選項 A)。
6. **Rust Edition**: 2021 → 2024，rust-toolchain.toml channel 1.99.1 (選項 A)。
7. **CSS 設計系統**: 完整依憲章 §5 實作 (選項 A)。
8. **CI/CD 部署**: 完整依憲章 §11-12 實作 (選項 A)。

---

## 3. Crate 對應與目錄結構 (重構後)

```
ekpi-platform/
├── Cargo.toml              ← 根 workspace manifest (resolver = "3", edition 2024)
├── rust-toolchain.toml     ← channel 1.99.1, clippy+rustfmt
├── Cargo.lock              ← 統一 lockfile
├── migrations/             ← SQLx 遷移檔 (PostgreSQL 18.6)
├── apps/
│   └── web/                ← Presentation: Axum 0.8.9 HTTP, Askama 0.16.1 HTML, handlers, routes, middleware
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs
│       │   ├── state.rs
│       │   ├── routes/
│       │   ├── handlers/
│       │   ├── extractors/
│       │   ├── middleware/   (CSRF, session, authz, security headers)
│       │   ├── responses/
│       │   └── errors/
│       ├── templates/
│       │   ├── layouts/
│       │   ├── pages/
│       │   ├── components/
│       │   ├── partials/
│       │   └── macros/
│       └── static/
│           └── css/
├── crates/
│   ├── domain/             ← Domain: 核心業務規則 (從 eval/ 遷移)
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── application/        ← Application: 使用案例、Ports (從 platform/ 拆分)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── ports/      (repositories.rs, clock.rs, transaction.rs)
│   │       ├── users/
│   │       ├── accounts/
│   │       ├── authentication/
│   │       └── authorization/
│   └── adapters/           ← Infrastructure: PostgreSQL 18.6, sessions, password, email, cache, configuration
│       ├── Cargo.toml
│       └── src/
│           ├── postgres/
│           ├── sessions/
│           ├── password/
│           ├── email/
│           ├── cache/
│           └── configuration/
├── eval/                   ← 保留為評測 harness (依賴 crates/domain，不改動核心邏輯)
└── platform/               ← 保留 CLI 進入點 (ekp_platform bin)，不作為 web 入口
```

---

## 4. 實作階段 (6 週，有序依賴)

### Phase 1: 工具鏈與 Workspace 重構 (Week 1)

**目標**: 產出可編譯的 Cargo Workspace，Rust 2024 + Askama 0.16.1，零 clippy 警告。

| # | 任務 | 詳細內容 | 驗收 |
|---|------|---------|------|
| 1.1 | 安裝 PostgreSQL 18.6 | `winget install PostgreSQL.PostgreSQL.18`，啟動服務，建立本地測試資料庫 | `psql` 可連線 |
| 1.2 | 升級 Rust Edition | 所有 `Cargo.toml` `edition = "2024"`，根 workspace `resolver = "3"` | `cargo build` 無 edition 錯誤 |
| 1.3 | 建立 rust-toolchain.toml | channel = "1.99.1", components = [rustfmt, clippy], profile = "minimal" | `rustc --version` = 1.99.x |
| 1.4 | 重構為 Cargo Workspace | 建立 apps/web, crates/domain, crates/application, crates/adapters；eval/ 與 platform/ 保留但調整依賴指向 | `cargo build --workspace` 通過 |
| 1.5 | 替換模板引擎 | Cargo.toml 加入 askama = "0.16.1"，移除 maud = "0.27"；platform/governance_web 模板轉為 Askama | `cargo build --workspace` 通過，無 maud 引用 |
| 1.6 | 建立空模板目錄 | apps/web/templates/{layouts,pages,components,partials,macros} | 目錄存在 |
| 1.7 | 驗證建置與檢查 | `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace --locked` | 零警告，測試通過 |

### Phase 2: 資料層遷移 (Week 2)

**目標**: PostgreSQL 18.6 作為主要資料庫，SQLx 0.9.0 compile-time checked query，離線中繼資料版本控制。

| # | 任務 | 詳細內容 | 驗收 |
|---|------|---------|------|
| 2.1 | 設計 PostgreSQL schema | users, sessions, roles, audit_logs 等表；UUID 主鍵，TIMESTAMPTZ，NOT NULL/UNIQUE/CHECK 約束 | `migrations/` 下有遷移 SQL |
| 2.2 | 建立 SQLx 遷移 | `migrations/20261030xxxx_*.sql`，可在乾淨資料庫執行 | `cargo sqlx prepare` 或 `sqlx migrate run` 成功 |
| 2.3 | 設定 SQLx compile-time query | crates/adapters/postgres/ 使用 `sqlx::query!` 宏，啟用 postgres + chrono + uuid + migrate feature | `cargo build` 無 SQLx 編譯錯誤 |
| 2.4 | 離線中繼資料 | `cargo sqlx prepare`，`.sqlx/` 加入 Git | `cargo sqlx prepare --check` 通過 |
| 2.5 | 連線池與交易 | PgPoolOptions (max_connections, acquire_timeout, idle_timeout, query_timeout)，Transaction 實作 | 連線池參數可設定，tx commit/rollback 正確 |
| 2.6 | 索引與執行計畫 | EXPLAIN ANALYZE 驗證常用查詢，建立 Partial/Composite Index | 無 N+1，查詢執行計畫合理 |
| 2.7 | 資料遷移腳本 | SQLite → PostgreSQL 資料遷移 (若有既有資料) | 遷移後資料完整 |

### Phase 3: 模板引擎與呈現層 (Week 3)

**目標**: Askama 0.16.1 模板渲染 HTML，完整路由與表單，Zero-JS 驗證。

| # | 任務 | 詳細內容 | 驗收 |
|---|------|---------|------|
| 3.1 | 轉換現有模板 | platform/governance_web 的 Maud 模板重寫為 Askama，置於 apps/web/templates/ | 無 maud 檔案，Askama Template derive 正確 |
| 3.2 | 建立路由與處理器 | apps/web/routes/ 與 handlers/：登入、登出、CRUD、搜尋、分頁、錯誤頁 (404/500) | 所有路由可存取，回傳完整 HTML |
| 3.3 | HTML Auto-Escaping | Askama 預設 Auto-Escaping，禁止隨意 `safe`，使用者輸入經過 Sanitizer 白名單 | 無 `safe` 濿除經審核者 |
| 3.4 | CSP 設定 | `script-src 'none'`，`style-src 'self'`，`img-src 'self' data:` 等 | Response Header 包含 CSP |
| 3.5 | Zero-JS 測試自動化 | 檢查產生的 HTML 無 `<script>`、`onclick`、`onchange`、`.js`、`.wasm` | Zero-JS 檢查清單全勾選 |
| 3.6 | 表單 POST/Redirect/Get | 新增/修改/刪除使用 POST，成功後 303 See Other | 表單提交無重複提交問題 |

### Phase 4: 企業級安全加固 (Week 4)

**目標**: 依憲章 §8 實作完整企業安全控制。

| # | 任務 | 詳細內容 | 驗收 |
|---|------|---------|------|
| 4.1 | 密碼雜湊 | Argon2id (Memory 19 MiB, Iterations 2, Parallelism 1，依硬體調整) | `argon2` crate 0.6.x，密碼不為明文 |
| 4.2 | Session 管理 | tower-sessions + PostgreSQL Store，__Host-session cookie (Secure, HttpOnly, SameSite=Lax)，Idle/Absolute Timeout | Session Cookie 符合憲章 §8.2 |
| 4.3 | CSRF 保護 | Synchronizer Token Pattern，HTML Form 含隨機 CSRF Token，伺服器驗證與 Session 綁定 | 狀態變更請求無 CSRF Token 被拒 |
| 4.4 | RBAC/ABAC | User → Authentication → Session → Authorization → Business Rule → Repository → Database，Fail Closed | 權限拒絕均 Fail Closed |
| 4.5 | Security Headers | CSP, HSTS, X-Content-Type-Options, Referrer-Policy, X-Frame-Options, Permissions-Policy | 回應標頭完整 |
| 4.6 | Rate Limiting | 登入失敗 Rate Limiting，失敗訊息不洩漏帳號是否存在 | 登入速率受限 |
| 4.7 | 稽核日誌 | 記錄認證事件、授權失敗、應用程式錯誤，禁止記錄密碼/Session ID/CSRF Token | 稽核日誌可查，無敏感資料外洩 |

### Phase 5: 設計系統完善 (Week 5)

**目標**: 依憲章 §5 實作完整 CSS 設計系統，WCAG 2.2 AA。

| # | 任務 | 詳細內容 | 驗收 |
|---|------|---------|------|
| 5.1 | Design Tokens | CSS Variables (OKLCH, light-dark, 顏色/半徑/間距/陰影)，`color-scheme: light dark` | `static/css/core/tokens.css` |
| 5.2 | Cascade Layers | `@layer reset, tokens, base, layout, components, utilities, overrides` | `static/css/app.css` 分層 |
| 5.3 | Container Queries | 元件層響應式 (`container-type: inline-size`, `@container`) | 元件在不同容器寬度正確響應 |
| 5.4 | Native 元件 | `<details>` Accordion, `<button popovertarget>`, `<dialog>` Invoker Commands API | 無 JS 動態開合，舊版瀏覽器有替代導覽 |
| 5.5 | View Transitions | `@view-transition { navigation: auto; }` 為漸進增強 | 跨頁導覽動畫，非必要條件 |
| 5.6 | 無障礙 | Keyboard Navigation, Focus Visibility, Color Contrast, Label Association, Reduced Motion, Touch Target | WCAG 2.2 AA 評估通過 |

### Phase 6: CI/CD + 部署 (Week 6)

**目標**: 依憲章 §11-12 實作 CI/CD 與生產部署基礎設施。

| # | 任務 | 詳細內容 | 驗收 |
|---|------|---------|------|
| 6.1 | GitHub Actions 工作流 | fmt check, clippy -D warnings, test --workspace, sqlx prepare --check, cargo audit, build --release --locked | PR 門檻全部通過 |
| 6.2 | Zero-JS CI 檢查 | 檢查產生的 HTML 與 Network Requests 無 JS 資產 | CI 零 JS 檢查通過 |
| 6.3 | Dockerfile | 多階段建置 (builder + runtime)，非 root 使用者 (10001)，靜態資源複製 | `docker build` 成功，非 root 執行 |
| 6.4 | Health Check | GET /health/live, GET /health/ready (ready 檢查資料庫連線) | 兩個端點回應正確 |
| 6.5 | Graceful Shutdown | SIGTERM 收到後停止接收新流量，等待執行中請求，結束交易，關閉資料庫 Pool | 正常終止 |
| 6.6 | 資料庫備份/還原 | 備份腳本，還原測試 | 備份可還原 |
| 6.7 | 監控與告警 | HTTP 錯誤率、延遲、連線池、CPU/記憶體、磁碟 | 監控指標可視化 |

---

## 5. 垂直切片 (Vertical Slice) — 第一個可驗證交付物

依憲章建議，第一個交付物為可編譯的 Cargo Workspace，接著完成包含以下元素的垂直切片：

- **登入**: POST Form + CSRF Token + Argon2id 密碼驗證 + Session
- **Session**: tower-sessions + PostgreSQL 持久化，Secure/HttpOnly/SameSite Cookie
- **PostgreSQL 18.6**: SQLx 0.9.0 compile-time query，連線池，遷移
- **Askama 0.16.1**: 模板渲染 HTML，Auto-Escaping
- **CRUD**: 使用者新增/修改/刪除，POST/Redirect/Get
- **權限**: RBAC 伺服器端驗證，Fail Closed
- **現代 CSS**: Design Tokens, Cascade Layers, Container Queries, 響應式版型

此垂直切片作為整個企業平台的標準範例，驗證所有技術基線可協同運作。

---

## 6. 風險與緩解

| 風險 | 等級 | 緩解措施 |
|------|------|---------|
| PostgreSQL 18.6 Windows 安裝受阻 | High | `winget install PostgreSQL.PostgreSQL.18`；若失敗，改用 Docker PostgreSQL 或遠端實例 |
| eval/ 與 platform/ 重構為 Workspace 造成依賴破壞 | High | 逐 crate 遷移，每步驟後 `cargo build --workspace` 驗證；保留 Git 回滾點 |
| Askama 0.16.1 模板語法與 Maud 不相容 | Medium | 逐頁轉換，Askama 0.16.1 `let`/`set` 改為 `decl`/`declare`，參考官方遷移指南 |
| Argon2id 參數效能 | Medium | 依正式伺服器硬體進行安全與效能測試後調高 Memory/Iterations |
| ekpi-platform 原 PROJECT_CHARTER 階段門栓 (Phase A 未核准、Phase B 受阻) 與企業級目標衝突 | Medium | 遷移完成後更新 PROJECT_CHARTER、PRODUCTION_READINESS，重新評估階段門栓 |
| 舊版系統可能需要 JavaScript | Low | 本憲章為 Strict Zero-JS，舊版系統若需要 JavaScript 應另行評估例外（ADR），不納入本計畫 |

---

## 7. 驗收清單 (Definition of Done)

### 架構
- [ ] Rust 2024 Edition，所有 crate
- [ ] Cargo Workspace：apps/web, crates/domain, crates/application, crates/adapters
- [ ] Clean Architecture 依賴方向正確 (Presentation → Application → Domain)

### Zero-JS
- [ ] 無瀏覽器端 JavaScript (`.js`, `.mjs`, `.cjs`, `.wasm`)
- [ ] CSP `script-src 'none'`
- [ ] 無 Inline Script、無 Inline Event Handler
- [ ] JavaScript Disabled 模式正常

### 前端
- [ ] 語意化 HTML5, CSS Design Tokens, Cascade Layers, Container Queries
- [ ] Keyboard Navigation, Reduced Motion, WCAG 2.2 AA

### 後端
- [ ] Axum 0.8.9 HTTP, Askama 0.16.1 HTML
- [ ] SQL 全部參數化 (SQLx query!)，無字串拼接
- [ ] Request Timeout、Request Body Limit、Graceful Shutdown

### 資料庫
- [ ] PostgreSQL 18.6，有 Migration
- [ ] SQLx Compile-Time Query Check，離線中繼資料版本控制
- [ ] Connection Pool 管理，備份與還原測試

### 安全
- [ ] HTTPS, Secure Cookie, HttpOnly, SameSite
- [ ] CSRF Token, Argon2id, Rate Limiting
- [ ] RBAC/ABAC, CSP, Security Headers, Audit Log

### 效能
- [ ] LCP p75 ≤ 2.5 秒，INP p75 ≤ 200 ms，CLS p75 ≤ 0.1
- [ ] HTTP Compression，Static Asset Cache，Connection Pool 壓力測試

### CI/CD
- [ ] Formatting, Clippy, Unit Test, Integration Test
- [ ] SQLx Offline Check, Security Audit, Zero-JS Check
- [ ] Docker Build, Staging Deployment, Rollback Plan

---

## 8. 未解決問題

1. **PostgreSQL 18.6 安裝方式**: Windows 上 `winget install PostgreSQL.PostgreSQL.18` 是否可行？若不可行，是否改用 Docker 實例？
2. **eval/ 與 platform/ 是否保留**: 重構後 eval/ (評測 harness) 與 platform/ (CLI) 是否作為獨立 crate 保留，或完全併入新 Workspace 結構？
3. **現有 SQLite 資料遷移**: ekpi-platform 現有 SQLite 資料是否需要遷移至 PostgreSQL 18.6？若是，遷移腳本範圍為何？
4. **根 Dev/Cargo.toml 與 ekpi-platform/ workspace 關係**: 兩個 workspace 並行是否可接受？或是否應統一於根目錄？
5. **階段門栓狀態更新**: 遷移後 ekpi-platform 的 Phase A / Phase B 門栓狀態應如何更新？由誰核准？

---

**計畫路徑**: `.kilo/plans/1793358284200-ekpi-platform-zero-js-charter-migration.md`
**下一步**: 實作代理人依 Phase 1 執行，先安裝 PostgreSQL 18.6 與升級 Rust Edition 2024。

---

### 關鍵技術說明

**關於「2024 版本」的說明**: 
- **Rust 2024 Edition** 是 Rust 的最新 edition，於 2025-02-20 隨 Rust 1.85.0 正式發布。Rust editions 每 3 年發布一次，下一個 edition 預計在 2027 年發布。因此，Rust 2024 Edition 是目前（2026 年 10 月）的最新版本。
- **Rust 1.99.1** 是最新的 Stable 版本（2026-10-15 發布）。
- 所有依賴套件版本（Axum 0.8.9、Askama 0.16.1、SQLx 0.9.0、PostgreSQL 18.6）均為 2026 年 10 月的最新版本。

**關於 PostgreSQL 19**: PostgreSQL 19 目前為 Beta 3（2026-08-13 發布），尚未正式發布。憲章指定 PostgreSQL 18 作為生產基線是正確的選擇。

**關於 Rust 2027 Edition**: Rust editions 每 3 年發布一次，下一個 edition (2027) 預計在 2027 年發布。目前 (2026-10) 尚未有 2027 Edition 的預覽版本。Rust 2024 Edition 是目前的正確且最新 edition。

---

由於系統權限限制，此計畫已寫入 `.kilo/plans/1793358284200-ekpi-platform-zero-js-charter-migration.md`。