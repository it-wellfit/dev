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

工具：`cargo run --release --bin auxbridge-backup`（原始碼 `platform/src/bin/auxbridge-backup.rs`）

| 作業 | 命令 | 說明 |
|---|---|---|
| 建立備份 | `cargo run --release --bin auxbridge-backup -- --db data/auxbridge.db --out backups --keep 14` | 唯讀連線 `VACUUM INTO` 一致性快照＋`integrity_check`／`foreign_key_check`＋SHA-256＋JSONL 清單；保留最近 14 份 |
| 驗證備份 | `cargo run --release --bin auxbridge-backup -- --verify backups/auxbridge-<ts>.db` | 校驗碼比對＋完整性檢查；PASS 才可用於還原 |
| 還原演練 | 停止服務 → 以驗證通過之備份覆蓋 DB（含 `-wal`／`-shm` 清除）→ 啟動並確認 `/status` 與交易筆數 | 每月至少一次並記錄 |

- 內容：SQLite 資料庫快照（含 WAL 內容）；`knowledge/` 與 `docs/` 另以版本庫／檔案備份保存。
- 保存：離線保存、保留 30 日；每月還原演練並留存紀錄。
- 目前限制：git 層之分支／標籤備份（`git bundle`）尚未自動化（仍為人工），列為追蹤項。

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