# 版本演進評分卡 — v0.2.0（2026-11-17 快照）

> 依據：計畫 1793782615513 §10.4｜基準：2026-11-13 [0.2.0] 發布（commit 4c29a1e）｜受評：2026-11-17 工作樹（HEAD 560e3ce）
> 規則：每維度 ≥ 前版（單調進化）；任何倒退須 ADR 量化說明。

| 維度 | 前版（11-13） | 本版（11-17） | 證據 |
|---|---|---|---|
| 功能完整性 | 動態頁 43＋靜態 console；導覽兩級 | ＋帳簿日期區間過濾、互動教學、9 個頂層頁圖示、自動展開導覽 | contract_nav 3 快照；瀏覽器實證截圖 |
| 效能（p95） | stress p95 未入冊（僅 SLA 斷言） | **504ms / SLA 650**（42 samples 入 baseline.json） | docs/benchmarks/baseline.json |
| 安全（audit 高危） | 0 vulnerabilities | 0（audit/deny 閘於 CI） | .github/workflows/ci.yml security job |
| 可維護性（clippy） | 零警告 | 零警告（＋visual-diff crate 亦零警告） | 最終閘門 EXIT=0 |
| 使用者體驗 | 側欄 60px 直條缺陷、行動溢出、麵包屑重複 | **全數修復**：main 48→1210px、scrollW 365==clientW、麵包屑去重、圖示一致性、精品化 W12 | 截圖 w12b-final-sidebar-icons-dedup.png 等 |
| 可靠性 | 無回滾機制、無環境隔離 | 回滾演練 PASS（RTO 1 分/RPO 0）、三環境隔離、/health /ready 探針 | docs/ops/DRILL_LOG.md |
| 可觀測性 | 文字日誌 | JSON 日誌＋BUILD_INFO（commit/build_time/uptime_s）＋DEPLOY_LOG | /health 實測輸出 |
| 治理 | 無 SLO、無埠登記 | SLO 99.5%/216 分預算、PORTS.toml SSOT、DRILL_LOG、BRANCH_PROTECTION 指引 | docs/standards/、docs/ops/ |

**總評**：8/8 維度單調進化，零倒退。唯一移除項＝NSIS updater 簽名工件（createUpdaterArtifacts false）——理由：updater 端點為內部佔位、無簽名金鑰，工件本不可用；移除使建置可重現（ADR 類別：配置修正，非功能退步）。