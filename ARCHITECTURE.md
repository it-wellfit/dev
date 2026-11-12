# 架構（Architecture）— Dev 根工作區

- 版本：v1.0｜生效日期：2026-11-12｜適用範圍：根 workspace 全體 crate｜狀態：Active
- 決策 SSOT：`docs/adr/`（技術基線 ADR-0001、API 分層 ADR-0002、雙一分錄 ADR-0003、跨平台 ADR-0013、unsafe 與供應鏈 ADR-0014）

## 1. 系統上下文

```
瀏覽器（零 JS）  ──HTTP──▶  platform（axum + Askama，:8090）  ──▶  SQLite（auxbridge.db）
        ▲                        │
        │                        ├──▶ KEM（知識演化模組，BM25＋claims＋hash 鏈）
        │                        ├──▶ OCR／匯入（CSV/XLSX/PDF/DOCX/EML）
        │                        └──▶ 銀行支付／同步（Wellfit 契約）
        │
   Tauri 桌面殼（auxbridge-web）──啟動/監看──▶ auxbridge-server（platform 的 bin）
```

## 2. Crate 拓撲與依賴方向

| Crate | 角色 | 依賴 |
|---|---|---|
| `core`（auxbridge_core） | 領域：金額（i64 分）、帳本引擎、權限、草稿 | 無（最底層） |
| `crates/console_manifest` | 頁面/導覽 SSOT（靜態資料） | 無 |
| `crates/kem` | 知識演化（claims、BM25、hashing） | console_manifest |
| `crates/flaps_bridge` | 原廠串接（COM 批次橋接，real-com 門控） | core 契約 |
| `platform` | 應用服務＋表現層（axum/Askama） | core、console_manifest、kem |
| `web` | 零 JS 前端資產（HTML/CSS） | — |
| `web/src-tauri` | 桌面殼（Tauri 2）＋更新鏈 | 無 crate 相依（僅殼邏輯） |

規則：依賴方向單向（`platform → core`），不得反向；`core` 不得依賴任何 I/O 框架。

## 3. 表現層不變量

- 零 JavaScript：`<script>` 數 = 0、無內聯事件（`platform/tests/guards.rs` 強制）。
- 樣式只用 Design Token（`web/assets/design-system.css`），禁內聯硬編碼色彩。
- 表單：GET 讀取、POST 變更、成功後 PRG。

## 4. 資料與一致性

- 金額一律 `i64` 最小貨幣單位（禁用 f64；技術債見 `docs/IMPROVEMENT_BACKLOG.md`）。
- 雙一分錄：每筆交易至少借貸各一，帳本引擎驗證平衡（`core/src/ledger/`）。
- 冪等：匯入以 SHA-256 指紋去重；外部同步以冪等鍵防重。

## 5. 可觀測性

- 後端：`tracing`＋`tracing-subscriber`（`AUXBRIDGE_LOG` 控制等級）。
- 桌面殼：日誌寫入 `%LOCALAPPDATA%\AuxBridge\logs\auxbridge-web.log`（無主控台）。
- 健康檢查：`platform` 之 status 端點與 `platform/src/status.rs`。

## 6. 部署拓撲

| 環境 | 組成 | 說明 |
|---|---|---|
| 開發 | platform :8090 + Tauri dev | `AUXBRIDGE_DB_URL` 指向開發 DB |
| 桌面安裝版 | NSIS 安裝包（殼＋server sidecar＋knowledge/console） | 更新走 minisign 簽名 feed |

## 7. 變更紀錄

| 日期 | 版本 | 說明 |
|---|---|---|
| 2026-11-12 | v1.0 | 依 SHARED_STANDARDS §4.1 建立；拓撲與現況一致（含 ADR-0014 之 unsafe 邊界）|