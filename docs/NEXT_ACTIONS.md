# NEXT_ACTIONS — 下一個實際開發任務

> 更新：2026-11-09｜依價值排序。每項完成後更新 PROJECT_STATUS.md 與本文件。
> （v3：十二切片已完成並登錄 addendum #12/#13；剩餘項目如下。）

## 已完成（2026-11-07 → 2026-11-09）

| # | 任務 | 證據 |
|---|------|------|
| ✅ | 三大工作室十二切片（G1.1–G1.4／G2.1–G2.5／G3.1–G3.3） | PLAN_REGISTER addendum #12/#13；追溯矩陣 master §22 全綠 |
| ✅ | Stress test 量測法修正（預熱＋21 樣本 p95；SLA 500ms 不變） | warm p95 422ms；100k tx |
| ✅ | CP950 編碼損壞根因修復（api.rs 三方合併；config.rs/kem main.rs PUA） | docs/encoding-standard.md |
| ✅ | 驗證方法論體系（TDD/BDD 8 情境/SDD/DDD/ADD；MT 100%；FUZZING.md） | DEVELOPMENT_METHODOLOGY §7–§9 |
| ✅ | docs/README v3.0 索引重整＋PROJECT_STATUS/NEXT_ACTIONS/GAPS 刷新 | 2026-11-07/09 |
| ✅ | PRE-R1a 特徵化測試（並行 session 交付 characterization.rs） | 6 tests PASS |

## 最高價值（排序）

| # | 任務 | 階段 | 前置 |
|---|------|------|------|
| 1 | **kem 線穩定後整工作區四關重跑**（含 kem in-flight evolve/gov/search） | 收口 | 並行線穩定點 |
| 2 | **Backup/Restore 演練**（data\flaps.db 複製→還原→查詢驗證）＋BACKUP_RECOVERY.md | Phase 2 | 無 |
| 3 | **fuzz 執行解鎖**：Windows ASan DLL（0xc0000135）或 libFuzzer-only／Linux CI | Phase 1→2 | 工具鏈 |
| 4 | **對帳報告渲染**：G2.4 計算層接 api_logs.html 演進（掛平行線） | G2 收口 | 平行線 |
| 5 | **部署環境決策**（TLS／網域／主機）——**需擁有者授權** | Phase 2 | 授權 |
| 6 | OpenTelemetry 導入評估 | Phase 2/PHASE-G4 | 部署環境 |
| 7 | R1 api.rs 模組化（23 路由）：特徵化測試已備，待 GAP-08 分歧解鎖 | PHASE-G4 | 分歧裁決＋ADR |

## 高風險問題

| # | 問題 | 處理 |
|---|------|------|
| 1 | f64 遺留？——**無**（全整數 minor 分已驗證；P0-1 待 BACKLOG 覆核關閉） | — |
| 2 | 速率限制 in-process（多實例不共用） | 多實例需求出現時遷移 DB/Redis |
| 3 | 編碼損壞（CP950 mojibake）復發風險 | 已由 docs/encoding-standard.md 規範＋PowerShell UTF8 無 BOM 寫檔慣例防制 |
| 4 | 並行 session 同檔案寫入衝突（本週 api.rs/kem 兩起瞬時編譯錯誤） | 以 board 協調分工；門檻重跑前先 `cargo check` 確認對方線穩定 |

## 產品改善候選（完整清單見 IMPROVEMENT_BACKLOG.md）

| # | 項目 | 狀態 |
|---|------|------|
| 1 | 交易分頁 | ✅ 已完成（分頁欄位還原並過測） |
| 2 | 多幣別匯率換算 | 候選 |
| 3 | 正式會計傳票格式 | 候選 |
| 4 | parse 規則表管理 UI（erp_config parser 關鍵字） | 候選 |

## 制度化候選

| # | 項目 |
|---|------|
| 1 | `.kilo/rules/`（原 .kilocode/rules）沉澱本 workspace 規則（依 Kilo 版本確認格式） |
| 2 | GitHub Actions workflow 驗證（存在未執行；補 BDD binding audit step） |
