# Flaps ERP 工作台 — Dev 根工作區

> 證據導向的企業帳務工作台：零 JavaScript、整數分貨幣、複式記帳強制、Argon2id 認證、完整稽核軌跡。

## Project Overview

Flaps ERP 工作台是根 workspace（ARES）的表現層產品：以 **Strict Zero-JS**（瀏覽器零應用程式 JavaScript）方式提供帳務登帳、查詢、匯入、報表與 AI 輔助（建議僅供人工覆核）。後端為 Rust（Axum 0.7 + SQLx/SQLite），前端為 Askama 伺服器端渲染＋CSS Design Token。

核心能力：引導式登帳精靈、複式記帳強制平衡、CSV 兩段式匯入（乾跑→確認）、非結構化文字解析草稿（規則表＋依據卡）、AI 建議（特性旗標＋schema 閘門，永不自動落地）、多維樞紐與平行座標報表、月報 CSV 匯出。

## Features

- 帳務登帳：借貸雙欄、引擎強制平衡（Σ借=Σ貸>0）、冪等 UUID、稽核事件
- 快速匯入：兩段式 CSV/TSV（乾跑預檢→確認，≤2MB/5,000 列，重放防護 409）
- 引導式精靈：模板預填、欄位級規則提示、from_draft/from_batch 修正入口
- 草稿解析：規則表驅動（依據卡）；AI 建議（特性旗標，schema 閘門，需人工覆核）
- 報表：月報（HTML＋CSV）、樞紐矩陣（白名單維度）、平行座標投影、SVG 月度圖
- 安全：Argon2id、伺服器端會話（可撤銷）、CSRF 同步器 token、登入速率鎖定、安全標頭（CSP/nosniff/DENY/no-referrer/no-store）、會話即時撤銷（停用/重設）
- 管理：使用者建立/停用/密碼重設（ROLE_MANAGE）

## Architecture Overview

```
瀏覽器（零 JS）──HTML/CSS──> Axum 0.7（api.rs 路由層）
  │ middleware: require_session（會話）＋ security_headers（OWASP 標頭）
  ▼
platform（db／auth／api／ai）──> flaps_core（領域：ledger／money／parser／permission）
  ▼
SQLite（WAL；migrations 0001-0003）
```

分層：Presentation（api.rs handlers）→ Application（auth/session、import 流程）→ Domain（flaps_core：ledger engine、money、parser、permission）。決策記錄見 `docs/adr/`（ADR-0001 技術棧、ADR-0002 API 層、ADR-0003 認證）。

## Technology Stack

- Rust 1.99（edition 2021）、Axum 0.7、Askama 0.16、SQLx 0.8（SQLite）
- argon2 0.5（密碼雜湊）、sha2 0.10（檔案指紋）、csv 1.3（匯入）
- 零前端依賴：無 Node/npm；CSS Design Token（`web/assets/design-system.css`）

## Requirements

- Windows（開發環境）：PowerShell；Rust 1.99 toolchain（rust-toolchain.toml）
- 本機 SQLite（檔案式，WAL）
- （選用）Ollama 127.0.0.1:11434 供 AI 建議

## Installation

```powershell
git clone <repo>; cd Dev
cargo build --locked -p platform
```

## Environment Variables

| 變數 | 預設 | 說明 |
|------|------|------|
| `FLAPS_DB_URL` | `sqlite://data/flaps.db?mode=rwc` | SQLite 連線（父目錄自動建立） |
| `FLAPS_PORT` | 8090 | 監聽埠（僅 127.0.0.1） |
| `FLAPS_ADMIN_PASSWORD` | （隨機，一次性列印） | 首次啟動管理員密碼 |

## Development Setup

```powershell
cargo fmt -- --check; cargo clippy --all-targets -- -D warnings; cargo test --locked
```

品質門檻四項皆須 EXIT=0（見 `docs/standards/SHARED_STANDARDS.md`）。測試證據入 `.kilo/sessions/`。

## Testing

```powershell
cargo test --locked                       # 全部（66+ 測試，0 failed）
cargo test --locked -p platform --test close_out   # 硬化收尾（效能預算/Token 同步/撤銷/重設）
```

## Build / Deployment

```powershell
cargo build --locked -p platform --bin flaps-server
# 啟動：target\debug\flaps-server.exe（127.0.0.1:8090）
```

部署要點：前置 TLS 後於 erp_config（security/cookie_secure=1）啟用 Secure cookie（R-07）；多實例部署需將速率限制遷移至共享存放（P1-7 notes）。

## Troubleshooting

| 症狀 | 處理 |
|------|------|
| 首次登入無帳號 | 啟動時管理員密碼列印於 stdout（僅一次），或設 `FLAPS_ADMIN_PASSWORD` |
| AI 建議 503 | 特性旗標未開或 Ollama 離線——改用規則解析（/drafts） |
| 匯入 409 | 該批次已確認；不可重複匯入 |
| 資料庫忙碌 | WAL＋busy_timeout 已設；持續失敗見 `docs/operations/` |

## Documentation

- 治理：`WORKSPACE_CHARTER.md`、`docs/standards/`、`docs/PLAN_REGISTER.md`
- 產品：`docs/product/PRD.md`、`docs/quality/bdd_acceptance.md`、`docs/testing/TRACEABILITY_MATRIX.md`
- 設計：`docs/design/`（Token、無障礙）、`docs/adr/`
- 安全：`docs/security/THREAT_MODEL.md`、`.cargo/audit.toml`
