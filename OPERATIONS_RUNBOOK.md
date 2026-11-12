# 營運手冊（Operations Runbook）— Dev 根工作區

- 版本：v1.0｜生效日期：2026-11-12｜適用範圍：開發機與安裝版之日常營運｜狀態：Active
- 詳細發行/更新流程 SSOT：`docs/release/UPDATER_RUNBOOK.md`（本檔不重複其内容）

## 1. 啟動與停止

| 情境 | 命令 | 說明 |
|---|---|---|
| 開發伺服器 | `cargo run --bin auxbridge-server` | 預設 `:8090`；DB 由 `AUXBRIDGE_DB_URL` 指定 |
| 桌面殼（開發） | `cargo tauri dev`（於 `web/src-tauri`） | 會自動拉起 platform |
| 安裝版 | 開始功能表 → AuxBridge | 殼啟動同目錄 `auxbridge-server.exe` |
| 停止 | 關閉視窗／結束 `auxbridge-server` 行程 | Windows 重建前務必先結束殘留 `.exe`（避免档案占用） |

## 2. 日誌

| 元件 | 位置 | 等級控制 |
|---|---|---|
| platform | 標準輸出／`AUXBRIDGE_LOG` | `AUXBRIDGE_LOG=debug` |
| 桌面殼 | `%LOCALAPPDATA%\AuxBridge\logs\auxbridge-web.log` | 同上 |

## 3. 備份與還原（依 `SHARED_STANDARDS` §8.1）

- 備份內容：`data/`（SQLite DB）＋ `knowledge/` ＋ `docs/` 之版本庫。
- 建議頻率：每日；保留 30 日；離線保存並驗證。
- **還原演練**：每月一次；本 repo 目前以人工演練為之（`scripts/backup.ps1` 尚未提供 → 見 `TROUBLESHOOTING.md` 之已知缺口）。

## 4. 更新（安裝版）

1. 產出簽章安裝包（見 `docs/release/UPDATER_RUNBOOK.md` §1–§3）。
2. 更新 `latest.json` 並發布至 `updates.auxbridge.internal`（**尚未建置 → 更新鏈端到端仍 BLOCKED**）。
3. 被動更新流程：外掛驗簽（minisign）→ 下載 → 執行 installer（passive）→ 重啟。

## 5. 健康檢查

| 檢查 | 方式 |
|---|---|
| 伺服器存活 | `GET /login` 回 200 |
| 資料庫 | `GET /status`（platform 狀態頁） |
| 更新通道 | `GET https://updates.auxbridge.internal/erp/latest.json`（未建置時回 DNS 失敗） |

## 6. 事故處理原則

1. 先保值證（日誌、DB 備份），再修復。
2. 變更一律走 ADR／計畫登記（`docs/PLAN_REGISTER.md`）。
3. 未量測不得宣稱已修復（`UNKNOWN ≠ PASS`）。

## 7. 變更紀錄

| 日期 | 版本 | 說明 |
|---|---|---|
| 2026-11-12 | v1.0 | 依 SHARED_STANDARDS §4.1 建立；內容與現況一致（含已知缺口）|