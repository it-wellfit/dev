# NAMING_AND_BOUNDARY_DECREE — 命名與邊界裁定 v1.0

> 日期：2026-11-11｜裁定人：使用者（原話）｜狀態：Active
> 「工作平台與開發的相關系統不能取名為 FlapsErp，避免搞混、搞雜、混用發生錯誤，我們是要深度串接而不是亂改原廠內容造成無法登入使用原廠軟體，亂改反而沒保固。」

## 1. 身份分離表（最高優先裁定）

| | 原廠軟體（vendor） | 自建平台（self-built） |
|---|---|---|
| 名稱 | **FlapsErp**（中文名 **輔翼ERP**） | **AuxBridge**（中文顯示名 **整合工作台**） |
| 形態 | 第三方 Electron 產品（閉源、逆向分析對象） | Rust/Axum 零 JS 工作台（本 workspace `platform/` 等） |
| 安裝/資料 | `C:\SYSTEM_PROGRAM_DIRECTORY`（唯讀） | `Dev\` workspace（可改）；使用者機器 `%LOCALAPPDATA%` |
| 可否修改 | **否**——僅可完整設定/組態調整（config-only）；原始碼、二進位、登錄檔、安裝目錄一律禁改 | 是——本 workspace 內全權修改 |
| 串接方式 | 深度串接：API、config、唯讀觀察、`crates/flaps_bridge`（COM 批次橋） | — |
| 保固 | 誤改即失保固、無法登入使用 | 自負 |

## 2. 命名規則（2026-11-11 生效）

1. **串接/轉接器 crate 以串接目標命名**（正確語義，保留）：`flaps_bridge`（橋接 FlapsErp COM）、`wellfit_bankpay`（玉山）、`wellfit_shopee`、`wellfit_mtrix`。
2. **領域/基礎設施 crate 以自建身份命名**：`auxbridge_core`（原 `flaps_core`）、`platform`、`console_manifest`、`kem`。
3. **bin／env／程序**：`auxbridge-server`（原 `flaps-server`）；env 前綴 `AUXBRIDGE_`（`AUXBRIDGE_DB_URL`/`AUXBRIDGE_PORT`/`AUXBRIDGE_BIND`/`AUXBRIDGE_ADMIN_PASSWORD`/`AUXBRIDGE_SERVER_EXE`/`AUXBRIDGE_REQUIRE_RELEASE_ARTIFACTS`；原 `FLAPS_*` 全數廢止）。
4. **產品/殼**：Tauri `productName: AuxBridge`、identifier `com.auxbridge.erp`、updater `updates.auxbridge.internal`；視窗標題與 UI 品牌一律「AuxBridge 整合工作台」。
5. **資料庫**：`auxbridge.db`（預設）、`demo-auxbridge.db`（示範，root `data/`；原 `demo-flaps.db`）。埠 8090（1420 為歷史/原廠生態埠，自建勿綁）。

## 3. 邊界三不三只

- **三不**：不改原廠原始碼/安裝目錄/登錄檔；不刪「測試資料」時動到原廠 DB；不對原廠環境跑自建 E2E（反之亦然）。
- **三只**：對原廠只經（a）文件化 config、（b）官方/逆向得证的 API 介面、（c）唯讀觀察路徑；自建 E2E 只打 `127.0.0.1:8090`；憑證只用自己的（admin/Admin1234 等 demo 帳）。

## 4. 可執行防呆

- `platform/tests/naming_guard.rs`：活躍面（templates/src/tests/web assets/tauri conf/Cargo.tomls）禁用 `Flaps ERP`、`輔翼工作台`、`flaps-server`、`flaps_core`、`FLAPS_`、`demo-flaps.db`、`updates.flaps.internal`；`cargo test -p platform --test naming_guard` 綠為閘門。
- 舊→新映射即本表；歷史紀錄（`.kilo/plans`、CHANGELOG 舊條目、`knowledge/claims`、ADR 決策表）**保留舊名不改**（C-01），以本文件為對照 SSOT。

## 5. 刻意保留舊名之處（合法語境）

| 位置 | 理由 |
|---|---|
| `docs/research/FlapsErp*`、`docs/04-operations/FlapsErp*` | 原廠產品線文檔，主語即原廠 |
| `web/legacy-1420.html` | 歷史快照「原樣重現」，改了就失真 |
| `.kilo/plans/*`、CHANGELOG 舊條目、`knowledge/claims/*` | 歷史紀錄/知識庫審計鏈，不可回寫 |
| `crates/flaps_bridge`、root `Cargo.toml` members | 串接目標命名規則（§2.1） |
| 根目錄 `flaps_erp_*.md` 三檔 | 歷史管理報告 |

## 6. 待辦（需擁有者動作）

- 已安裝的 NSIS 產品 `%LOCALAPPDATA%\Flaps ERP\`（0.2.0，自建但品牌錯誤）：經「設定→應用程式」解除安裝後，以新名重建安裝（`cargo tauri build`，productName 已改 AuxBridge）。**勿與原廠 FlapsErp 安裝混淆——它們是兩回事。**
