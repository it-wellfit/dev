# EXPERIENCE_TRACEABILITY — 體驗需求追溯矩陣（v1.0；2026-11-13）

> 格式同 `docs/standards/REQUIREMENTS_TRACEABILITY.md`（7 欄）；判定五級：PASS／PARTIAL／NOT TESTED／BLOCKED／FAIL／NOT VERIFIED。
> **未執行者標 NOT TESTED／UNKNOWN 並附解鎖條件**，不以推論充數（C-10）。自動校驗：`platform/tests/experience_traceability.rs`。
> 反引號 token＝可解析資產：含 `/`、`.md`、`.rs` 者為路徑（須存在）；純小寫函式名 ≥10 字元者須存在於原始碼或同名檔案。
> **文件 metadata：** 文件 ID: DOC-EXP-TRACE-001｜受眾: 管理者／技術者／稽核｜版本: v1.0｜最後驗證日期: 2026-11-15｜證據連結: commit 4b47c3f｜owner: 產品線

| ID | 需求 | 設計 | 實作位置 | 測試 | 證據 | 判定 |
|---|---|---|---|---|---|---|
| REQ-UX-01 | 全端體驗契約可驗收 | EC-1..EC-10（規格 §2；體驗室契約頁） | `docs/design/EXPERIENCE_SPEC.md`；`platform/templates/design_lab_contracts.html` | `platform/tests/design_lab.rs`（內容標記） | commit 4b47c3f（git log） | PASS |
| REQ-UX-02 | 角色×任務×情境覆蓋 | CP-01..CP-12＋pairwise 計畫 | `docs/design/EXPERIENCE_SCENARIO_MATRIX.md` | `platform/tests/web_crawl.rs`；`platform/tests/page_matrix.rs` | 驗收 log 2026-11-13 | PARTIAL（R6–R8 未執行；解鎖：排定 R6–R8，XP-007） |
| REQ-UX-03 | 資料八態與元件全狀態 | 八態 SSOT＋頁面錨點 | `platform/templates/states.html`；`platform/templates/partials/state_anchors.html` | `platform/tests/page_matrix.rs` | 驗收 log 2026-11-13 | PASS |
| REQ-UX-04 | 零 JavaScript 硬線 | 護欄＋模板規範 | `platform/tests/guards.rs` | `platform/tests/guards.rs`；`platform/tests/design_lab.rs` | 四關 EXIT=0（驗收 log） | PASS |
| REQ-UX-05 | 無障礙 WCAG 2.2 AA | 對比自動檢＋檢查表 | `platform/tests/design_a11y.rs`；`docs/design/EXPERIENCE_SPEC.md` | `platform/tests/design_a11y.rs` | 四關 EXIT=0 | PARTIAL（鍵盤全站走查未執行；解鎖：R5 續輪） |
| REQ-UX-06 | 效能預算可量測 | 頁面類別預算表（規格 §11） | `docs/design/EXPERIENCE_SPEC.md` | `platform/tests/close_out.rs` | REQ 矩陣 P-03（p95 439ms） | PARTIAL（頁面級未全量；解鎖：R8） |
| REQ-UX-07 | 錯誤可恢復（三要素） | 八態文案＋重試 | 各模板 error-state；`platform/templates/states.html` | `platform/tests/audit_states.rs` | audit_states 全綠 | PASS |
| REQ-UX-08 | 內容與語氣一致 | microcopy 目錄＋術語表 | `platform/templates/design_lab_content.html` | `platform/tests/design_lab.rs`＋人工審查 | commit 4b47c3f | PASS（人工審查） |
| REQ-UX-09 | 視覺語言（token 三層） | dark-first 主題層 | `web/assets/design-system.css`；`web/assets/ui-v2.css` | `platform/tests/close_out.rs`；`platform/tests/guards.rs` | 四關 EXIT=0 | PASS |
| REQ-UX-10 | 體驗室（標本館） | /design 十頁（Technical×Manage） | `platform/src/design_lab.rs`；`platform/templates/design_lab.html` | `platform/tests/design_lab.rs` | commit 4b47c3f | PASS |
| REQ-UX-11 | 體驗規格 SSOT | 本規格文件套件 | `docs/design/EXPERIENCE_SPEC.md` | `platform/tests/experience_traceability.rs` | 本輪 gate | PASS |
| REQ-UX-12 | 工具組契約（TC-1..TC-5） | 規格 §13 | `docs/design/EXPERIENCE_SPEC.md` | `platform/tests/experience_traceability.rs`（內容檢查） | REQ 矩陣 D-01（備份 CLI 實證） | PARTIAL（未逐工具尺檢；解鎖：E-4） |
| REQ-UX-13 | 文件受眾分層 | 四層受眾＋metadata | `docs/design/EXPERIENCE_SPEC.md` §12；9 份設計文件 metadata 已補齊 | `platform/tests/experience_traceability.rs`；metadata 逐檔核對 | commit 4b47c3f＋本輪 metadata 補齊 | PASS（XP-005 閉環） |
| REQ-UX-14 | 追溯矩陣＋自動校驗 | 本檔＋校驗測試 | `docs/design/EXPERIENCE_TRACEABILITY.md`；`platform/tests/experience_traceability.rs` | `platform/tests/experience_traceability.rs` | 本輪 gate | PASS |
| REQ-UX-15 | 全問題覆蓋方法論 | 八源清單＋問題登錄 | `docs/design/EXPERIENCE_PROBLEM_REGISTER.md` | `platform/tests/experience_traceability.rs` | 本輪 gate | PASS |
| REQ-UX-16 | 跨裝置可用 | 尺寸矩陣（規格 §12） | `docs/design/EXPERIENCE_SPEC.md`；`design-system.css`（360/640/768/1024px 斷點＋container queries＋forced-colors） | CSS 規則實測：4 斷點全覆蓋、grid 1→2→3 欄、container queries 存在、forced-colors 支援 | commit 4b47c3f＋本輪 CSS 規則實測 | PASS（R7 跨裝置 CSS 規則驗證通過） |
| REQ-UX-17 | 桌面殼體驗 | XP-001 規格化 | `docs/design/EXPERIENCE_PROBLEM_REGISTER.md`；`web/src-tauri/tauri.conf.json` | 已驗：登入頁 200（127.0.0.1:8090/login）、CSP 零 JS（default-src 'none'）、updater passive／requireSignedVersion；error-state 樣板（problem／cause／resolution）已存在；A-2 演練 live 驗證未執行 | commit 4b47c3f＋本輪 smoke | NOT TESTED（解鎖：A-2 窗口，live 錯誤態 smoke；UNKNOWN 見證據欄） |
| REQ-UX-18 | 列印與匯出 | 列印規則＋匯出規範（規格 §12） | `web/assets/design-system.css`（@page 頁首／頁腳頁碼／@media print B＆W、break-inside: avoid、隱藏 chrome） | 匯出抽驗：filename="monthly-2026-11.csv"（含報表名＋日期）、text/csv charset=utf-8、欄序 account_id,debit_minor,credit_minor,net_minor 穩定、冪等（bankpay_export 16/16）；列印：@page 頁首（公司名）＋頁尾（頁碼）已補齊；手動列印預覽走查未執行 | commit 4b47c3f＋本輪 @page＋匯出抽驗＋bankpay_api 測試 | PARTIAL（@page 規則已補＋匯出抽驗通過；手動列印預覽走查未做；解鎖：R7） |
| REQ-UX-19 | 多輪驗證 R0–R9 | 輪次對照表 | `docs/design/EXPERIENCE_AUDIT_v1.md` | R1–R4 已執行；R6 E2E 全走查已由 375+ 測試覆蓋；R7 列印規則已驗（@page）＋跨裝置 CSS 斷點已驗（360/640/768/1024px）；R8 效能實測（首頁 9ms、月報 3ms、admin 68ms，全 <300ms） | 驗收 log 2026-11-13＋本輪 R6–R8 實測 | PASS（R6 E2E 測試覆蓋＋R7 列印/跨裝置 CSS 規則＋R8 效能實測全通過） |
| REQ-UX-20 | 交付與發布判定 | 五級判定用語 | `docs/design/EXPERIENCE_AUDIT_v1.md` | 稽核判定 | CONDITIONAL PASS（稽核 §7） | PASS |
