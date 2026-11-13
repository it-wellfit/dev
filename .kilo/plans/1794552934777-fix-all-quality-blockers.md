# 修復所有品質閘道阻礙 — 實作計畫 (v2)

## 目標
將 EXPERIENCE_PROBLEM_REGISTER / EXPERIENCE_AUDIT_v1 中未關閉 XP (P1/P2)，以及 REQ-UX 追溯矩陣 PARTIAL/NOT TESTED，全部閉環為 PASS 或「誠實 UNKNOWN＋解鎖條件」。

## 環境前提（非原始碼，必先由使用者解鎖）
`cargo fmt / clippy --all-targets -- -D warnings / test --locked / audit` 被本機權限系統封鎖。實作代理無法重跑四關，所有「綠」僅為歷史證據。
- 前置：使用者開放 `cargo` 執行權，或設定隔離目標 `CARGO_TARGET_DIR=$env:LOCALAPPDATA\Temp\dev-target`，否則驗收只能記為「歷史證據，未複驗」。

## 開放決策（實作前需使用者裁定）
| # | 決策 | 建議 |
|---|---|---|
| D1 | cargo 執行權未開放 | 由使用者開放；若不可開，四關驗收標記「僅歷史證據」 |
| D2 | XP-006 TT-9/TT-10 互動教學／工作流設計器 | 建議維持 P2 路線，本輪不實作，不阻發布 |
| D3 | R6–R8 驗證窗口排程 | 由使用者確認時間，實作代理僅負責執行與留痕 |

## 阻礙與任務（依 P 級；單一寫者原則；不得加 #[allow]）

### P1（必關閉，產出證據）
**XP-001 桌面殼演練 (S2，A-2 窗口)**
- 首啟 smoke：啟動 tauri 殼、登入頁 200、CSP 標頭、零 JS；留 exit code＋stderr。
- passive 更新 smoke：檢查 updater 設定、版本比對、更新提示。
- 錯誤態演練：DB 損毀/啟動失敗時錯誤頁顯示與恢復路徑。
- 證據寫入 `.kilo/sessions/`；EXPERIENCE_SPEC §12 補 S2 規格條目。

**XP-005 文件受眾標籤 (四層：操作者/管理者/技術者/稽核)**
- 掃描 `docs/` 全檔，找出缺少受眾 metadata 的文件。
- 逐檔補 metadata：文件 ID、受眾、版本、適用範圍、最後驗證日期、證據連結、owner。
- EXPERIENCE_SPEC §12/S6 補「受眾四層＋metadata」規則。
- 驗收：`grep` 受眾標籤覆蓋率，或新增 `platform/tests/docs_governance.rs`（實作代理擇一；新增測試檔屬原始碼改動）。

### P2（需證據或誠實 UNKNOWN＋解鎖，不得留空）
**XP-002 報表列印 (S4，R7 窗口)**
- 確認 `web/assets/design-system.css` `@media print`：頁首（公司/報表名/期間/產出時間）、頁尾（頁碼/版本/內部使用）、分頁不截斷、黑白可讀。
- 列印預覽手動走查，截圖證據。
- EXPERIENCE_AUDIT_v1 §3/S4 更新現況；EXPERIENCE_SURFACE_MATRIX S4 EC-1 ◐→●（若通過）。

**XP-003 匯出抽驗 (S5，E-4)**
- 抽驗 3 類匯出檔（CSV/JSON/報表匯出）：檔名含報表名＋日期、欄序穩定、UTF-8、金額原樣（未知「—」）。
- 結果寫入驗收 log；EXPERIENCE_SPEC §12/S5 已訂規則，此處補實作證據。

**XP-006 TT-9/TT-10 路線確認**
- 確認 help 頁現有 AI 段落（已存在，不新增代碼）。
- 標註為 P2 路線，不阻發布；EXPERIENCE_PROBLEM_REGISTER 已登載，無需新增代碼。

**XP-007 R6–R8 輪次 (R7 窗口)**
- R6：E2E 全走查（登入→匯入→草稿→登帳→查詢→稽核，含失敗注入）。
- R7：跨裝置走查（tablet/phone/列印）；列印部分與 XP-002 合併。
- R8：頁面級效能 p95／bytes／位移（清單 <300ms、詳情 <150ms、報表 <500ms，2k 交易）。
- 結果回填 EXPERIENCE_SCENARIO_MATRIX 抽樣紀錄小節。

### 追溯與自動校驗（最後執行）
- 執行 `platform/tests/experience_traceability.rs`，確認零問題（需 cargo 可用）。
- EXPERIENCE_TRACEABILITY.md：PARTIAL/NOT TESTED 依對應 XP 更新為 PASS 或 UNKNOWN（附解鎖），無 PARTIAL/NOT TESTED 遺留。
- PLAN_REGISTER 新增 addendum（#45），記錄所有 XP 處置與證據路徑（命令＋exit code＋日期＋修訂版）。

## 驗收門檻
- 四關 EXIT=0（若 cargo 可用），否則標記「僅歷史證據」。
- `experience_traceability.rs` 零問題。
- XP-001、XP-005 (P1) 必須關閉並產出證據。
- XP-002/003/006/007 (P2) 必須有證據或誠實 UNKNOWN＋解鎖條件。
- 所有 REQ-UX 判定為 PASS 或 UNKNOWN（附解鎖）。
- PLAN_REGISTER addendum 紀錄完整且可追溯。

## 強制規則（參考，不複製全文）
- C-01 不亂覆蓋亂刪；C-03 先量測後動刀；C-04 斷言式補丁；C-05 並行編輯先定基準；C-10 UNKNOWN ≠ PASS ≠ FAIL；C-11 Rust-first（不新增 Python）；C-13 Evidence-Driven 五級判定。
