# PRODUCTION READINESS — 正式營運就緒檢查（Go/No-Go）

> 更新：2026-11-04｜結論：**NO-GO（尚未授權部署）**——程式面就緒度高，部署/營運面需授權與環境。

## 使用規則

- 每項必須有**可驗證證據**（測試/log/指令輸出），`UNKNOWN ≠ PASS`。
- 標示 🔒 的項目需要**明確授權**（部署/採購/環境）才能執行。
- 全部 ✅ 且無重大未處理風險時，方可改判 **GO**。

## 就緒矩陣

| # | 類別 | 項目 | 狀態 | 證據/備註 |
|---|------|------|------|-----------|
| 1 | 應用程式 | Build 成功（--locked） | ✅ | cargo build EXIT=0 |
| 2 | 應用程式 | 測試 87→90 passed / 0 failed | ✅ | session tests log |
| 3 | 應用程式 | clippy -D warnings / fmt | ✅ | EXIT=0；allow 歸零 |
| 4 | 應用程式 | 零 JS（CSP 無 script 來源） | ✅ | security_headers 測試 |
| 5 | 認證 | Argon2id＋會話（撤銷/清理/Secure 旗標） | ✅ | 對應測試 |
| 6 | 授權 | RBAC 即時 DB 查詢（server-side） | ✅ | RBAC 403 測試 |
| 7 | 授權 | 多租戶資料隔離 | ⬜ N/A | 單租戶現況；多租戶屬 Phase 4 |
| 8 | 資料 | Migration 可重放（乾淨 DB） | ✅ | 遷移測試 |
| 9 | 資料 | Backup 機制 | 🟡 檔案複製（手動） | 🔒 需備份策略授權 |
| 10 | 資料 | **Restore 演練** | ❌ | 未執行（Stage 2 必做） |
| 11 | 資料 | 資料保留/刪除政策 | ❌ | 未制定 |
| 12 | 安全 | OWASP 標頭＋CSRF＋速率鎖定＋參數化 SQL | ✅ | 對應測試 |
| 13 | 安全 | DAST／外部滲透測試 | ❌ | 部署後（🔒） |
| 14 | 安全 | Secrets 管理（無秘密入碼 ✓；正式秘密保管） | 🟡 | 環境變數；無 vault |
| 15 | 效能 | 報表 <500ms（2k 交易） | ✅ | close_out 效能測試 |
| 16 | 效能 | 負載測試（k6） | ❌ | 需流量模型（Stage 3） |
| 17 | 營運 | Monitoring／Alerting（OpenTelemetry） | ❌ | 🔒 需部署環境 |
| 18 | 營運 | Health check | ✅ | /api/health |
| 19 | 營運 | Incident Response／Runbook | 🟡 | README Troubleshooting 部分 |
| 20 | 營運 | Rollback Plan | 🟡 | git revert＋DB 備份還原（未演練） |
| 21 | 營運 | SLA 宣稱 | ⬜ 不宣稱 | 無正式營運數據 |
| 22 | 合規 | 會計法規遵循 | ❌ | **需專業意見**（NEEDS CONFIRMATION） |
| 23 | 部署 | Production 環境（TLS／主機） | ❌ | 🔒 需授權＋採購 |
| 24 | 部署 | CI/CD pipeline（驗證執行） | 🟡 | .github/workflows 存在未驗證 |
| 25 | 授權 | 正式部署授權 | ❌ | **EXPLICIT_AUTHORIZATION_REQUIRED** |

## Go/No-Go 判定

**NO-GO** —— 阻斷項：#10（還原演練）、#17（監控）、#23（環境）、#25（授權）。

## 進入 GO 的最短路徑

1. 🔒 授權部署環境（TLS 主機）→ #23。
2. 完成 #10 還原演練＋#20 Rollback 演練。
3. 導入 #17 監控（OTel 最小集：logs＋/api/health 探測）。
4. #22 會計法規遵循確認（專業意見）。
5. 全部 ✅ 後重新本檢查並改判 GO。
