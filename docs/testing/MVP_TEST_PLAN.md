# MVP TEST PLAN — Phase 1 測試計畫

> 依風險選擇測試層級（SHARED_STANDARDS §3）。測試即 BDD 場景（見 docs/quality/bdd_acceptance.md）。

## 1. 測試層級與範圍

| 層級 | 範圍 | 工具 | 位置 |
|------|------|------|------|
| 單元測試 | money 精確解析、parser 依據卡、parallel 整數正規化、value objects、builder | cargo test --lib | core/src（各模組 #[cfg(test)]） |
| 引擎測試 | 借貸平衡負向 8 條、聚合、稽核 | cargo test --lib | core/src/ledger/engine.rs |
| 整合測試（HTTP） | 全部 22 路由：認證、CSRF、RBAC、匯入、精靈、報表 | axum oneshot | platform/src/api.rs（tests mod） |
| 硬化收尾 | Token 同步、效能預算、會話撤銷、密碼重設 | cargo test --test close_out | platform/tests/close_out.rs |
| 煙霧（手動/腳本） | demo 資料種入＋啟動＋頁面可達 | curl.exe＋seed-demo.ps1 | scripts/ |

## 2. 測試原則

1. 新功能先寫失敗測試（TDD red→green→refactor，紅波 log 存證）。
2. Bug 修復必附迴歸測試（例：flaky 效能測試改確定性斷言）。
3. 不得為通過測試而刪除測試或放寬斷言。
4. 測試結果綁 session log＋git 修訂版。

## 3. 風險導向覆蓋

| 風險 | 覆蓋測試 |
|------|----------|
| 金額計算錯誤（INV-01） | money 6 條（精度/負數/溢位/幣別）＋roundtrip |
| 借貸不平衡（INV-02） | 引擎負向 8 條＋PIV-003 摺疊 |
| 未授權存取（T-01/T-06） | RBAC 403、CSRF 403、未認證 303 |
| 會話生命週期 | 停用撤銷、重設撤銷、過期清理 |
| 匯入資料品質 | 逐列驗證、重放防護、部分失敗回報 |
| 效能（RPT-001） | 2k 交易 <500ms |

## 4. 未涵蓋（誠實揭露）

| 項目 | 理由 | 後續 |
|------|------|------|
| 瀏覽器 E2E（真實瀏覽器自動化） | 零 JS 伺服器渲染＋本機驗證已由煙霧涵蓋；瀏覽器自動化工具（Playwright MCP）於 Phase 2 導入 | Stage 2 |
| 負載測試（k6） | 無真實流量數據；效能預算已由 2k 交易測試涵蓋 | Stage 3 |
| DAST | 部署後（Stage 2） | Stage 2 |
