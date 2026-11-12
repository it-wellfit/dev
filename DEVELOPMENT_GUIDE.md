# 開發指南（Development Guide）— Dev 根工作區

- 版本：v1.0｜生效日期：2026-11-12｜適用範圍：根 workspace｜狀態：Active
- 入口規則：`AGENTS.md`（最短接手程序）；本檔為命令與環境之對照。

## 1. 環境

| 項目 | 值 |
|---|---|
| Toolchain | `rust-toolchain.toml`（channel 1.99.1、rustfmt、clippy） |
| 目標 | `x86_64-pc-windows-msvc`（桌上安裝包）；CI 另跑 Ubuntu 作為可攜性檢查 |
| 資料庫 | SQLite（開發/安裝版）；策略上 PostgreSQL 18 為目標（ADR-0001） |
| 變數 | `AUXBRIDGE_DB_URL`、`AUXBRIDGE_ADMIN_PASSWORD`、`AUXBRIDGE_SERVER_EXE`、`AUXBRIDGE_LOG` |

## 2. 常用命令（PowerShell；串接用 `;`）

```powershell
cargo fmt -- --check                                   # 格式（提交前必過）
cargo clippy --all-targets --locked -- -D warnings     # 零警告
cargo test --locked                                    # 全部測試（0 失敗）
cargo audit                                            # 弱點（政策：.cargo/audit.toml）
cargo deny check                                       # 政策：deny.toml
cargo machete --manifest-path platform/Cargo.toml      # 未用依賴（逐成員執行）
cargo run --bin auxbridge-server                       #  Platform 伺服器
cargo tauri build                                      #  桌面安裝包（簽章金鑰見發行手冊）
```

## 3. 品質門檻（提交前）

1. 基線六項（`SHARED_STANDARDS` §6.2）。
2. 供應鏈：audit／deny／machete 三閘通過。
3. SBOM：任何依賴變更後，7 份 SBOM 必須重產（`docs/release/UPDATER_RUNBOOK.md` §4）；否則
   `cargo test -p platform --test sbom_freshness` 會失敗。
4. 文件與代碼同一提交更新（`SHARED_STANDARDS` §4.3）。

## 4. 目錄導覽

| 路徑 | 內容 |
|---|---|
| `core/` | 領域層（金額、帳本、權限、草稿） |
| `platform/` | 應用服務＋表現層（axum/Askama） |
| `web/` | 零 JS 前端資產（`index.html`、`assets/`） |
| `crates/console_manifest` | 頁面/導覽 SSOT |
| `crates/kem` | 知識演化模組 |
| `crates/flaps_bridge` | 原廠串接（real-com 門控） |
| `web/src-tauri/` | 桌面殼與更新鏈 |
| `docs/` | 標準、ADR、發行、研究、計畫登記 |
| `scripts/` | 既有 PowerShell 工具鏈（session 管理、示範資料等） |

## 5. 疑難排解入口

見 `TROUBLESHOOTING.md`；發行相關見 `docs/release/UPDATER_RUNBOOK.md`。

## 6. 變更紀錄

| 日期 | 版本 | 說明 |
|---|---|---|
| 2026-11-12 | v1.0 | 依 SHARED_STANDARDS §4.1 建立；命令與現行 CI 一致 |