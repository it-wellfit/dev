# SYSTEM ARCHITECTURE — 系統架構

> 詳細決策見 `docs/ARCHITECTURE.md`（C4、不變量 INV-01..04）與 `docs/adr/`。本文件為 Phase 0 交付的架構總覽。

## 分層架構（Clean Architecture）

```
┌────────────────────────────────────────────────────────────┐
│  Presentation — platform/src/api.rs（Axum 0.7 handlers）    │
│  22 路由；Askama 伺服器端渲染（零 JS）；CSRF/RBAC/標頭        │
├────────────────────────────────────────────────────────────┤
│  Application — platform/src/auth.rs（會話/Argon2id/速率）    │
│  platform/src/db.rs（migrations 0001-0003）                 │
├────────────────────────────────────────────────────────────┤
│  Domain — core/src（flaps_core）                             │
│  ledger（Transaction/Item/Engine：平衡、冪等、稽核）          │
│  money（整數分）、parser（依據卡）、parallel（SVG）           │
│  value_objects（Money/AccountId）、permission（RBAC）        │
├────────────────────────────────────────────────────────────┤
│  Infrastructure — SQLite（WAL；migrations 0001-0003）        │
│  flaps_bridge（COM 整合，feature-gated，EULA-gated）         │
└────────────────────────────────────────────────────────────┘
```

## 依賴規則

- **api.rs → auth/db → flaps_core →（無）**：內層不依賴外層。
- `#![forbid(unsafe_code)]` 全 workspace（憲章 §2.2）。
- clippy `-D warnings`、零 allow（too_many）。

## 關鍵設計決策（ADR 索引）

| ADR | 決策 |
|-----|------|
| ADR-0001 | 技術棧基線（edition 2021、axum 0.7、SQLite；記憶版差異見文件） |
| ADR-0002 | 零 JS API 層（Axum 伺服器渲染＋PRG＋白名單樞紐） |
| ADR-0003 | 認證會話（Argon2id＋伺服器端會話＋同步器 CSRF） |

## 安全模型摘要

| 控制 | 實作 | 測試 |
|------|------|------|
| 認證 | Argon2id＋伺服器端會話（24h、可撤銷、登入清理） | logout/reset 測試 |
| 授權 | RBAC 即時 DB 查詢（has_permission_db） | RBAC 403 測試 |
| CSRF | 同步器 token（10/10 POST 路由） | 偽造 token 403 測試 |
| 速率 | 5 次失敗鎖定 15 分（in-process） | 鎖定測試 |
| 標頭 | CSP/nosniff/DENY/no-referrer/no-store | security_headers 測試 |
| XSS | 零 JS＋Askama 自動跳脫＋CSP | ZJS-001 |
