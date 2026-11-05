# NEXT_ACTIONS — 下一個實際開發任務

> 依價值排序。每項完成後更新 PROJECT_STATUS.md 與本文件。

## 最高價值（Stage 1 → Stage 2 銜接）

| # | 任務 | 階段 | 前置 |
|---|------|------|------|
| 1 | **Backup/Restore 演練**（data\flaps.db 複製→還原→查詢驗證）＋BACKUP_RECOVERY.md | Phase 2 | 無（可本機執行） |
| 2 | **Health check 強化**（DB 查詢＋migration 版本納入 /api/health） | Phase 2 | 無 |
| 3 | **部署環境決策**（TLS／網域／主機）——**需擁有者授權** | Phase 2 | 授權 |
| 4 | OpenTelemetry 導入評估 | Phase 2 | 部署環境 |

## 高風險問題

| # | 問題 | 處理 |
|---|------|------|
| 1 | f64 遺留？——**無**（全整數分已驗證） | — |
| 2 | 速率限制 in-process（多實例不共用） | 多實例需求出現時遷移 DB/Redis |

## 產品改善候選（BACKLOG 完整清單見 IMPROVEMENT_BACKLOG.md）

| # | 項目 |
|---|------|
| 1 | 交易分頁（目前全量渲染） |
| 2 | 多幣別匯率換算 |
| 3 | 正式會計傳票格式 |
| 4 | parse 規則表管理 UI（erp_config parser 關鍵字） |

## 制度化候選

| # | 項目 |
|---|------|
| 1 | `.kilocode/rules/` 沉澱本 workspace 規則（依 Kilo 版本確認格式） |
| 2 | GitHub Actions workflow 驗證（存在未執行） |
