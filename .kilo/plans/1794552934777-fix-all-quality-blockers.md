# 修復所有品質閘道阻礙 — 實作計畫 (v4.1 已完成)
> 2026-11-17 Reviewed（整合完成計畫 1794810000000 I-4）：掃描無未勾選任務（\- [ ]\=0）；無未關閉程式任務。證據：PLAN_REGISTER addendum #52。

## 狀態：代碼層品質閘門全部綠；UX/P2 手動項以誠實 UNKNOWN/PARTIAL＋解鎖條件結案

## 目標（達成情況）
將 EXPERIENCE_PROBLEM_REGISTER / EXPERIENCE_AUDIT_v1 中未關閉 XP (P1/P2)，以及 REQ-UX 追溯矩陣 PARTIAL/NOT TESTED，全部閉環為 PASS 或「誠實 UNKNOWN/PARTIAL＋解鎖條件」。

- P0 = 0（既有 P0 全數已修，見 UX_CONTRACTS §問題清單）
- P1 = 0 遺留（XP-001／XP-005 已閉環並產出證據）
- P2：XP-002／XP-003 程式層已驗，手動走查標記 PARTIAL＋解鎖；XP-006 維持 P2 路線；XP-007 R6–R8 標記 NOT TESTED／PARTIAL＋解鎖條件
- REQ-UX：REQ-UX-13 → PASS；其餘手動項保留 PARTIAL／NOT TESTED 並附解鎖條件（符合 C-10 誠實註記，UNKNOWN ≠ PASS）

## 環境前提（已解鎖，code 代理可執行）
- `cargo 1.99.1` 可用；`rg` 15.2.0、`jq` 1.8.2 已安裝（消除「無法辨識 'rg'/'jq'」）。
- 四關均可執行並已驗證（下方證據）。

## 已完成（代碼層修復，四關皆綠）

| # | 問題 | 修復位置 | 驗收結果 |
|---|---|---|---|
| 1 | `ollama_client.rs` `"think": false,` 縮排錯誤（8→12 空格） | platform/src/ai/ollama_client.rs:75 | fmt/clippy 通過 |
| 2 | `admin_system.html` 含 CRLF（違反 LF／無 BOM） | platform/templates/admin_system.html | 轉 LF／無 BOM，guards 通過 |
| 3 | `mapping_apply` 欄位索引 off-by-one（>4 應 >5） | platform/src/api.rs:5150-5160 | mapping_flow 1/1 綠 |
| 4 | `docs/design-system.css` `--shadow-glow` 與 web 漂移 | docs/design-system.css:151 | close_out 4/4 綠 |

四關最終驗證（全部綠）：fmt EXIT=0；clippy 零警告；`cargo check -p platform` EXIT=0；`cargo test --locked` 全綠 0 failed／0 ignored；壓力 SLA p95=508ms ≤ 650ms PASS；`cargo audit` 歷史 EXIT=0（`.cargo/audit.toml` 6 筆 disposition，環境網路卡索引更新）。

## XP-005 文件受眾標籤（P1）→ REQ-UX-13 PARTIAL→PASS（已閉環）
為 9 份設計文件補 audience metadata（文件 ID／受眾／版本／最後驗證日期／證據連結／owner），格式為 blockquote 單行，插入於 header 最後一行 `>` 區塊之後、第一個 `##` 章節標題之前：

| 文件 | 文件 ID | 受眾 | owner |
|---|---|---|---|
| docs/design/EXPERIENCE_SPEC.md | DOC-EXP-SPEC-001 | 操作者／管理者／技術者／稽核 | 產品線 |
| docs/design/EXPERIENCE_AUDIT_v1.md | DOC-EXP-AUDIT-001 | 管理者／技術者／稽核 | 驗證線 |
| docs/design/EXPERIENCE_PROBLEM_REGISTER.md | DOC-EXP-REG-001 | 管理者／技術者 | 產品線 |
| docs/design/EXPERIENCE_SURFACE_MATRIX.md | DOC-EXP-SURF-001 | 管理者／技術者 | 設計線 |
| docs/design/EXPERIENCE_SCENARIO_MATRIX.md | DOC-EXP-SCEN-001 | 管理者／技術者 | 產品線 |
| docs/design/EXPERIENCE_TRACEABILITY.md | DOC-EXP-TRACE-001 | 管理者／技術者／稽核 | 產品線 |
| docs/engineering/UX_CONTRACTS.md | DOC-UX-CONTRACT-001 | 管理者／技術者 | 設計線 |
| docs/design/INFORMATION_ARCHITECTURE.md | DOC-IA-001 | 管理者／技術者 | 架構線 |
| docs/design/UI_DESIGN_DECONSTRUCTION_v1.0.md | DOC-UI-DECONSTR-001 | 設計者／技術者 | 設計線 |

驗收：`platform/tests/experience_traceability.rs` 5/5 綠（含 `traceability_rows_are_complete_and_resolvable`）；`docs_governance.rs` 3/3 綠（含 `required_project_docs_exist_with_metadata`）；REQ-UX-13 判定改為 PASS。

## XP-001 桌面殼體驗（P1）→ 程式層 smoke 已驗（已閉環）
EXPERIENCE_SPEC §12 S2 已補具體規格：
- 首啟：whitelisted URL http://127.0.0.1:8090/，登入頁 200，CSP `default-src 'none'`／`style-src 'self'`／`img-src 'self' data:`（零 JS 指令）
- passive 更新：`installMode=passive`、`requireSignedVersion=true`，endpoint `https://updates.auxbridge.internal/erp/latest.json`
- 錯誤態：`not_found_page()` 樣板採 problem／原因／處理方式格式，不空白

Live smoke 證據（已執行）：
- 啟動 auxbridge-server.exe，登入頁 http://127.0.0.1:8090/login 回 200 OK
- CSP header：`default-src 'none'; style-src 'self'; img-src 'self' data:; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'`（零 JS）
- 安全頭：x-content-type-options、x-frame-options: DENY、strict-transport-security、referrer-policy、cache-control: no-store
- 伺服器已正常停止

REQ-UX-17 判定：NOT TESTED（解鎖：A-2 窗口，live 錯誤態 smoke；UNKNOWN 見證據欄）——完整 DB-損壞錯誤態 live smoke 未執行，樣板與邏輯已存在。

## XP-002 報表列印（P2）→ 程式層 CSS 已驗，手動走查標 PARTIAL
- `web/assets/design-system.css` `@media print`（行 1590、2151）：隱藏 chrome、body `#ffffff`/`#000000`（黑白可讀）、`break-inside: avoid`（分頁不截斷卡片／表格）、隱藏分頁／導覽
- 缺口：無 `@page` 頁首（公司／報表名／期間／產出時間）／頁尾（頁碼／版本／內部使用）規則；手動列印預覽走查未執行（需瀏覽器）

REQ-UX-18 判定：PARTIAL（基礎規則在；@page 頁首頁腳＋報表列印走查未做；解鎖：R7 窗口）。

## XP-003 匯出抽驗（P2）→ 程式層已驗（bankpay_export 16/16 綠）
- 匯出端點回 200 OK，Content-Type `text/csv; charset=utf-8`（UTF-8）及 `application/json`
- 冪等匯出（重複請求結果一致）
- 缺口：檔名含報表名＋日期、欄序穩定、金額原樣的實際檔案抽驗未執行（需手動取樣）

REQ-UX-18 匯出層面：已驗 UTF-8／格式；檔名＋欄序抽驗標 PARTIAL＋解鎖（E-4）。

## XP-006 TT-9/TT-10（P2）→ 維持路線，不阻發布
help 頁現有 AI 段落（已存在，不新增代碼）。標註為 P2 路線；EXPERIENCE_PROBLEM_REGISTER 已登載，無需新增代碼。

## XP-007 R6–R8（P2）→ 手動驗證窗口，誠實標註
- R6：E2E 全走查（登入→匯入→草稿→登帳→查詢→稽核，含失敗注入）→ NOT TESTED（解鎖：排定 R6 窗口）
- R7：跨裝置走查（tablet／phone／列印）→ NOT TESTED（解鎖：排定 R7 窗口；列印部分與 XP-002 合併）
- R8：頁面級效能 p95／bytes／位移（清單 <300ms、詳情 <150ms、報表 <500ms，2k 交易）→ PARTIAL（壓力 p95 439ms 既有；頁面級預算見 SPEC §11；解鎖：排定 R8 窗口）

REQ-UX-02／06／19 保留 PARTIAL（R6–R8 未執行；解鎖：排定 R6–R8）；REQ-UX-16／17 保留 NOT TESTED／UNKNOWN（解鎖：R7／A-2 窗口）。

## 追溯與自動校驗（已執行，全部綠）
- `platform/tests/experience_traceability.rs`：5/5 綠
- `platform/tests/docs_governance.rs`：3/3 綠
- `platform/tests/guards.rs`：6/6 綠
- EXPERIENCE_TRACEABILITY.md：REQ-UX-13 → PASS；REQ-UX-17／18 更新證據與判定；其餘 PARTIAL／NOT TESTED 附解鎖條件，無遺留
- PLAN_REGISTER：未新增 addendum（手動 R6–R8 項目另立階段時再登載 #45）

## 驗收門檻（全部滿足）
- 四關 EXIT=0（fmt／clippy -D warnings／test --locked）；audit 歷史 disposition 維持。
- `experience_traceability.rs` 零問題；`docs_governance.rs` 3/3 綠；`guards.rs` 6/6 綠。
- XP-001、XP-005 (P1) 已閉環並產出證據。
- XP-002／003／007 (P2) 已有程式層證據或誠實 UNKNOWN/PARTIAL＋解鎖條件，無空格遺留。
- 所有 REQ-UX 判定為 PASS 或 PARTIAL/NOT TESTED（附解鎖），無空白。
- C-01／C-03／C-04／C-05／C-10／C-11／C-13 遵守。

## 版控狀態
工作樹有未提交變更（見 `git status`）。**未執行 `git commit`**——僅於使用者明確要求時提交。如需發布，請下達 commit 指令。
