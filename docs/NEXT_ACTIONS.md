# NEXT_ACTIONS — 下一個實際開發任務

> 更新：2026-11-07｜依價值排序。每項完成後更新 PROJECT_STATUS.md 與本文件。
> （v2 修復：原檔第 11 行含字面 `` `n `` 序列損壞，本版以合法表格重寫。）

## 已完成（2026-11-06 → 2026-11-07）

| # | 任務 | 證據 |
|---|------|------|
| ✅ | Health check 強化（migration 版本＋交易總數；DB 不可達 503 degraded） | `health_reports_*` ×2 測試通過 |
| ✅ | 驗證方法論體系（TDD 紅綠、BDD 8 情境綁定稽核、SDD 追溯、DDD 狀態機、ADD 戰術表） | DEVELOPMENT_METHODOLOGY §3/§5/§7–§9；ATDD_BDD_BIND_AUDIT |
| ✅ | DDD 狀態機測試 POSTED→PENDING_REVIEW | core/src/ledger/engine.rs（42 tests） |
| ✅ | 交易分頁（模板欄位 page/total_pages/prev/next 還原） | platform/templates/index.html；IndexTemplate |
| ✅ | Stress test 量測法修正（預熱＋21 樣本 p95；SLA 500ms 不變） | warm p95 422ms；100k tx |
| ✅ | CP950 編碼損壞根因修復（api.rs 三方合併；config.rs/kem main.rs PUA） | docs/encoding-standard.md |

## 最高價值（排序）

| # | 任務 | 階段 | 前置 |
|---|------|------|------|
| 1 | **Backup/Restore 演練**（data\flaps.db 複製→還原→查詢驗證）＋BACKUP_RECOVERY.md | Phase 2 | 無（可本機執行） |
| 2 | **三大工作室計畫 v3.1 落地**：S1 部門工作台、S2 API 中樞與資料管理（platform-hub：UDM/eventbus/channels/agent-policy）、S3 治理稽核（kem 證據鏈）— 計畫文件 §0–§24 補實後逐項實作 | 跨線 | 計畫文件 |
| 3 | **fuzz 執行解鎖**：Windows ASan DLL（STATUS_DLL_NOT_FOUND 0xc0000135）— 部署 clang_rt.asan 或改用 `-Zsanitizer=none`＋libFuzzer only | Phase 1→2 | 工具鏈 |
| 4 | **ATDD 缺口補測**：`rate_limiter_locks_after_threshold`、`health_reports_degraded_when_database_unreachable` 綁定補齊 | Phase 1 | 無 |
| 5 | **部署環境決策**（TLS／網域／主機）——**需擁有者授權** | Phase 2 | 授權 |
| 6 | OpenTelemetry 導入評估 | Phase 2 | 部署環境 |

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
