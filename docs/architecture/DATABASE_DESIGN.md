# DATABASE DESIGN — 資料庫設計

> SSOT：`platform/migrations/`（0001-0003）。SQLite WAL。本文件為設計摘要。

## Migration 清單

| Migration | 內容 |
|---|---|
| 0001_baseline_schema | accounts、transactions、transaction_items（**amount_minor INTEGER**、exchange_rate INTEGER）、audit_log、erp_config、config_history、roles、permissions、role_permissions、user_roles＋索引 |
| 0002_users_sessions | users（Argon2id）、sessions（csrf_token、expires_at） |
| 0003_import_and_drafts | import_batches、import_rows（SHA-256＋逐列狀態）、drafts（依據卡 JSON） |

## ERD（文字）

```
users 1──n sessions
users 1──n user_roles n──1 roles 1──n role_permissions n──1 permissions
transactions 1──n transaction_items n──1 accounts
transactions 1──n audit_log
erp_config 1──n config_history
import_batches 1──n import_rows
drafts（獨立）
```

## 設計決策

| 決策 | 理由 |
|------|------|
| 金額 INTEGER（整數分） | INV-01 零浮點（P0-1） |
| 交易＋分錄＋稽核同一事務 | INV-04 冪等＋原子 |
| sessions 伺服器端存放 | 可撤銷（T-01） |
| drafts 存依據卡 JSON | 可解釋（Layer C） |
| 索引：transactions(date desc)、items(transaction_id)、audit(transaction_id) | 報表查詢熱路徑（FR-007） |

## 連線與交易安全（INV-03）

- WAL＋busy_timeout 5000ms＋synchronous NORMAL
- 登帳事務（header＋items＋audit）單一 commit ≤ 50ms（設計目標）
- 迴歸：migrations 重放測試＋FK 約束測試

## 備份與還原

- 檔案式 DB：備份 = 複製 .db（WAL checkpoint 後）——**還原演練未執行（Stage 2）**
- 示範資料庫（demo.db）與正式（flaps.db）檔案獨立
