# 回滾演練記錄（DRILL_LOG）

> 依據：計畫 1793782615513 §9.4（季度演練）｜通過條件：RTO ≤ 5 分鐘、資料完整性 100%

---

## 演練 #1 — 2026-11-17

| 欄位 | 值 |
|---|---|
| 環境 | staging（埠 8092，data/staging.db） |
| 演練者 | AI 代理（code） |
| 演練類型 | 資料損壞 → 備份還原 → 服務恢復 |

### 時間線（UTC+8）

| 時刻 | 動作 | 結果 |
|---|---|---|
| T+0 | 停止 staging server（計畫 §9.2 步驟 1） | pid 31044 正常退出 |
| T+1s | 備份現況 staging.db 至 rollback-drill-20261117160855（430,080 B） | 成功 |
| T+15s | **注入損壞**：DELETE 全部 transactions（39→0 筆，模擬資料災難） | 注入成功 |
| T+40s | **執行還原**：從備份 Copy-Item 覆寫 staging.db（430,080 B） | 檔案恢復 |
| T+55s | 重啟 staging server（persistent） | 監聽 8092 |
| T+62s | `/health` 驗證：`{"status":"ok","transactions":39,"db":true,"migrations_applied":11}` | **資料完整恢復** |

### 量測結果

- **RTO（損壞注入 → /health 200 且資料完整）**：約 62 秒 ≈ **1 分鐘**（目標 ≤ 5 分鐘 ✅，12% 消耗）
- **RPO（資料遺失）**：**0**（39 筆交易全數恢復 ✅）
- 工具：檔案層 Copy-Item（SQLite 單檔語義）；輔以 `auxbridge-server --backup`（VACUUM INTO 一致性快照）為正式備份路徑

### 發現與改善

1. 回滾腳本 rollback.ps1 已就位但本次演練以手動步驟執行（步驟 = 腳本內容逐步鏡射）——下次演練改為直接呼叫 `tools/rollback.ps1 -TargetTag -Port` 全自動路徑。
2. 備份目錄慣例（`backup\<tag>\`）與 deploy.ps1 的自動備份（`backup\pre-deploy-*`）一致，無縫銜接。
3. staging 資料為匿名化副本，演練零風險；正式環境演練前須先執行 `auxbridge-server --backup`。

### 結論

**PASS**——RTO 與 RPO 均遠優於目標。制度生效：本記錄即為 §10.1 錯誤預算制度之第一期量測基礎。