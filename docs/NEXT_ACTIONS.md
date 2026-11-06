# NEXT_ACTIONS — 下一個實際開發任務

> 依價值排序。每項完成後更新 PROJECT_STATUS.md 與本文件。

## 最高價值（Stage 1 → Stage 2 銜接）

| # | 任務 | 階段 | 前置 |
|---|------|------|------|
| 1 | **Backup/Restore 演練**（data\flaps.db 複製→還原→查詢驗證）＋BACKUP_RECOVERY.md | Phase 2 | 無（可本機執行） |
| ~~2. **Health check 強化**~~（已完成：/api/health 回報 migration 版本＋交易總數；DB 不可達時 503 degraded；測試 health_reports_*×2 通過） | Phase 2 | 無（已完成） |
| 2.5 | **驗證方法論體系**：TDD 紅綠循環（M3 待覆核 KPI 先紅後綠）、BDD 場景註冊表 docs/testing/BDD_SCENARIOS.feature（8 Scenario 全綁定現有通過測試）、SDD 追溯表（docs/engineering/DEVELOPMENT_METHODOLOGY.md §3）、DDD 聚合不變量審計（新增 status_transition_posted_to_pending_review）、ADD 品質屬性戰術表（§5） | Phase 1→2 銜接 | 無 |` + "`n`| 2.6 | **DDD 狀態機測試**：POSTED→PENDING_REVIEW（core/src/ledger/engine.rs，31 passed / 0 failed） | Phase 1 | 無 |` + "`n`| 3 | **部署環境決策**（TLS／網域／主機）——**需擁有者授權** | Phase 2 | 授權 |
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
