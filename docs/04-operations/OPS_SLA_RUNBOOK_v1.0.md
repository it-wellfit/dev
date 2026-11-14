# OPS-SLA 營運手冊 v1.0（G4.8 / D2-08）

> 日期：2026-11-13｜適用：AuxBridge 整合工作台（platform，SQLite 本機）與 platform-hub（PostgreSQL 18，OQ-GOV-01 後）。
> 這是**營運規格與程序**，不是已達成的實測聲明；未量測者標 UNKNOWN（C-10）。

## 1. SLI / SLO

| SLI | SLO | 量測方式 | 現況 |
|---|---|---|---|
| 可用性（月） | ≥99.5%（內部工具情境） | `/api/health` 探測紀錄 | UNKNOWN（尚無探測紀錄） |
| 工作台首屏 p95 | ≤500 ms | `perf_baseline`（目前 p95 4.83 ms，TTFB） | ✅ 遠低於預算（2026-11-13） |
| `/api/search` p95（BM25-only） | ≤200 ms | `perf_baseline`（p95 3.20 ms） | ✅ 遠低於預算（2026-11-13） |
| `/api/search` p95（向量/RRF） | ≤800 ms | 需 embedder 環境 | UNKNOWN |
| 錯誤率（5xx/總請求） | <0.5% | 結構化日誌（tracing；platform 待導入） | UNKNOWN |
| 稽核鏈完整率 | 100% | CCM 鏈驗證（G3.3） | 待機制定期執行 |
| 備份成功率 | 100% | 備份腳本 exit code | UNKNOWN |

## 2. 健康探測

| 端點 | 用途 | 認證 |
|---|---|---|
| `GET /api/health` | liveness＋readiness（回報 DB 狀態；DB 不可達→503） | 公開 |
| `GET /api/modules/health` | 跨模組健康摘要 | session |
| `GET /api/bankpay/health` | BankPay 模組健康 | session |

`/ready` 目前未實作；`/api/health` 已同時表達 liveness/readiness 語義（G4.9 選項）。

## 3. 備份與還原

### 3.1 SQLite（platform，本機）

```powershell
# 備份（一致性快照；不需要停機）
sqlite3 target\data\auxbridge.db ".backup 'D:\backups\auxbridge-$(Get-Date -Format yyyyMMdd-HHmm).db'"

# 驗證
sqlite3 D:\backups\auxbridge-<timestamp>.db "PRAGMA integrity_check;"

# 還原（停機後覆蓋；先備份現檔）
Copy-Item target\data\auxbridge.db target\data\auxbridge.db.bak
Copy-Item D:\backups\auxbridge-<timestamp>.db target\data\auxbridge.db
```

- 頻率：每日；保留 7 日。
- RPO ≤24 h、RTO ≤1 h。
- 還原演練：每季一次，記錄於本檔附錄。

### 3.2 PostgreSQL（platform-hub，OQ-GOV-01 後）

```bash
pg_dump "$AUXBRIDGE_PG_URL" -Fc -f backup.dump
pg_restore --clean --if-exists -d "$AUXBRIDGE_PG_URL" backup.dump
```

## 4. 事件管理

| 等級 | 定義 | 回報時限 | 處理 |
|---|---|---|---|
| P1 | 服務不可用（`/api/health` 503 或無法登入） | 30 分鐘內回報擁有者 | 依 §3 還原或 rollback；記錄時間線與根因 |
| P2 | 降級（部分功能失效、效能超預算） | 1 個工作日 | 開缺口單；以 profiling/`EXPLAIN QUERY PLAN` 定位 |
| P3 | 單一使用者問題、文件錯誤 | 3 個工作日 | 排入下個迭代 |

每件事件須記錄：時間線、影響範圍、根因（FMEA/STPA）、矯正與預防、回滾點。

## 5. 變更與回滾

- 任何設定/旗標變更須有 rollback 條件與「關閉後正確行為」測試（FLAG）。
- DB 遷移須可回滾（down migration）或提供還原備份。
- 發布前四關：fmt／clippy -D warnings／test --locked／audit。

## 6. 訓練與演練

- 每季：還原演練、P1 桌面演練（安靜模式）、FAULT-01..07 混沌項目（TEST-STRATEGY）。
- 紀錄：本檔附錄（日期、參與者、結果、缺口）。