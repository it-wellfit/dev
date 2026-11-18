# RP-2 四方整合指引（零內容重複）

> 版本：v1.0（2026-11-17）｜計畫：`.kilo/plans/1793787631643-three-studio-slice-execution-plan.md` RP-2
> 目的：config.ini 調優之四方來源互鏈，不重寫內容，僅建立導航。

## 四方來源

| # | 角色 | 文件 | 關鍵章節 |
|---|---|---|---|
| 1 | **操作源**（config.ini 調優核對表） | `docs/research/FlapsErp產品線效能進度報告_v2.0.md` | 附錄 C（備份→DISABLE_HARDWARE_ACCELERATION→IGNORE_CERT_ERR→重跑量測→回滾） |
| 2 | **整合手冊**（操作程序） | `docs/04-operations/FlapsErp效能優化操作手冊_v1.0.md` | §3.2（config.ini 調優程序） |
| 3 | **量測法**（效能架構） | `docs/research/FlapsErp產品線架構優化計畫_v2.0.md` | P4（量測方法與 SLA） |
| 4 | **驗收指標源**（斷路器/監控/回滾） | `docs/management/FlapsErp_RPA_電商系統對接_專案文件與開發前準備報告_v2.1.md` | §1.1 G4（連續失敗 5 次觸發、重試 ≤3、恢復 ≤5min = G2.2 驗收準則）；§10.2（監控/備份/回滾） |

## 操作手冊家族互鏈

| 手冊 | 路徑 | 關係 |
|---|---|---|
| 效能優化操作手冊 | `docs/04-operations/FlapsErp效能優化操作手冊_v1.0.md` | config.ini 調優主程序 |
| A1 檔案通道手冊 | `docs/04-operations/A1檔案通道端到端操作手冊_v1.0.md` | 檔案通道操作（同目錄） |
| 設定變更 runbook | `docs/04-operations/FlapsErp設定變更runbook_v1.0.md` | 設定變更 SOP |
| 設定 A/B 實測報告 | `docs/04-operations/FlapsErp設定AB實測報告_v1.0.md` | A/B 實測結果（NOT VERIFIED） |
| 效能基線 | `docs/04-operations/PERF_BASELINE_v1.0.md` | 量測基線（2026-11-13） |
| OPS-SLA runbook | `docs/04-operations/OPS_SLA_RUNBOOK_v1.0.md` | 營運級別與 SLO |

## 使用方式

1. **config.ini 調優**：先讀 ① 附錄 C 核對表 → 依 ② §3.2 執行程序 → 用 ③ P4 量測 → 以 ④ §1.1 G4 驗收。
2. **回滾**：依 ② §3.2 回滾程序 + ④ §10.2 回滾條件。
3. **監控**：依 ④ §10.2 監控指標 + OPS-SLA runbook。

## 互鏈驗證

- 本文件與四方來源均為相對路徑，無內容重複。
- 更新任一來源時，本指引不需修改（僅連結）。