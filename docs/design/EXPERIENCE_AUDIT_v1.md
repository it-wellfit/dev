# EXPERIENCE_AUDIT_v1 — 全端體驗稽核（2026-11-13）

> 版本 v1.0｜方法：C-03「先量測後動刀」；證據綁定（命令＋exit＋日期＋修訂版）｜範圍：S1–S9 表面 × EC-1..EC-10 契約 × 資料八態 × 問題來源清單
> 關聯文件：`docs/design/EXPERIENCE_SPEC.md`（規格 SSOT）、`docs/design/EXPERIENCE_SURFACE_MATRIX.md`、`docs/design/EXPERIENCE_SCENARIO_MATRIX.md`、`docs/design/EXPERIENCE_PROBLEM_REGISTER.md`、`docs/engineering/UX_CONTRACTS.md`（互動層 TT-1..TT-8）、`docs/standards/REQUIREMENTS_TRACEABILITY.md`
> 判定用語：符合／部分／N/A（附理由）／UNKNOWN（解鎖條件）；**UNKNOWN ≠ PASS**。

---

## §1 方法與問題來源（逐源盤點，不得憑印象）

| # | 來源 | 搜尋方法（本輪實際執行） | 本輪結果 |
|---|---|---|---|
| 1 | 產品 backlog | 讀 `docs/product/IMPROVEMENT_BACKLOG.md`（P0–P3） | 既有項目已被 #36/#44 批次吸收；未見體驗層未閉項 |
| 2 | 缺口/風險 | 讀 `docs/GAPS_AND_RISKS.md`（GAP-01..08） | 無體驗層新增缺口 |
| 3 | 測試 | `cargo test --locked`（S1 驗收，單次全綠 EXIT=0）；聚焦套件 5 套 | 0 failed（詳 §2） |
| 4 | 靜態掃描 | guards（零 JS／零內聯樣式／編碼）＋複驗 `platform/tests/design_a11y.rs`（3 對比測試） | 對比閘門存在（badge strong light／dark composite／body text）；無新缺口 |
| 5 | 設計審查 | 體驗室十頁＋`UX_CONTRACTS.md` TT-1..TT-8 對照；ui-v2 落地盤點 | ui-v2 token＋grid 側欄已落地；碰撞 P0 已修（#44） |
| 6 | 使用者回饋 | #44 回報清單（忘記密碼／草稿漏傳／AI 不可用／治理 UI） | 13 項全修（見 UX_CONTRACTS §問題清單） |
| 7 | 營運事件 | 備份 CLI、遷移防呆、斷路器（REQ 矩陣 S/D/P 系列） | 還原演練未歸檔（既有項，非體驗層） |
| 8 | 情境推演 | S×EC 矩陣逐格＋角色×任務關鍵路徑（CP-01..CP-12） | 缺口入 XP 清單 |

> 未識別風險不是 0：本輪以「八源清單＋S×EC 逐格＋關鍵路徑」補足搜尋方法；結論以「已搜尋（方法）」＋「UNKNOWN（解鎖條件）」表述。

## §2 實測基線（2026-11-13）

| 項目 | 值 | 證據 |
|---|---|---|
| HEAD（稽核起點） | `e76794b`（E-3 證據回填） | git log |
| 建置 | `cargo check -p platform` EXIT=0 | 本輪實測 |
| 四關 | fmt EXIT=0；clippy（workspace・all-targets・-D warnings）EXIT=0；audit EXIT=0；`cargo test --locked` 單次全綠 EXIT=0 | `.kilo/sessions/20261113-design-lab/tests/20261113-e3-acceptance.log` |
| 測試總量 | 375 測試 0 失敗（lib 146＋kem 71＋core 44＋bridge 7＋bankpay 16＋整合） | PLAN_REGISTER #44 |
| 聚焦套件 | console_manifest lib 16/0；page_matrix 4/0；web_crawl 6/0；design_lab 2/0；password_reset 2/0 | 同上驗收 log |
| 體驗室 | `/design` 十頁上線（Gate=Manage；零 JS；設計 lab 測試 2/2） | commit `4b47c3f` |
| UI V2 | token（暗色優先＋玉山綠）＋grid 側欄已落地；子導覽被蓋 P0 已修 | #44；`web/assets/ui-v2.css` |
| 互動層 | TT-1..TT-8 有 TEST＋EVID（TT-6 密碼重設全鏈、TT-7 死鏈免疫、TT-8 品牌 404） | `UX_CONTRACTS.md`；interaction_audit 5/5 |
| 列印 | 基礎 `@media print` 規則 2 段（隱藏 chrome） | `design-system.css` 實測 |

## §3 表面稽核摘要（S1–S9；逐格矩陣見 SURFACE_MATRIX）

| 表面 | 現況 | 缺口 |
|---|---|---|
| S1 Web 工作台 | 契約齊備；零 JS 護欄；八態掛點；44+ 頁爬網 0 死鏈 | 無 P0/P1 |
| S2 桌面殼 | NSIS＋updater 配置在；shell 可編譯 | XP-001（首啟/更新/錯誤演練未執行） |
| S3 KEM 控制台 | 受保護靜態 65 檔；零 JS；token 收斂 | 無 |
| S4 列印 | 基礎規則 2 段 | XP-002（報表分頁/頁眉腳未驗） |
| S5 匯出 | CSV/JSON 可用 | XP-003（規範未成文；本規格 §12 已補） |
| S6 文件體系 | docs 受眾軸在；本輪補體驗文件套件 | XP-005（全面受眾標籤化＝A-3） |
| S7 API/DX | 401/403 語義；錯誤封套既有 | 無新缺口 |
| S8 通知/警報 | 批次管線告警在（runbook 家族） | 無新缺口 |
| S9 工具組 CLI | 備份/探針可用；TC 契約已訂（本規格 §13） | TT/TC 分名完成（本輪） |

## §4 缺口清單（XP）

| ID | 級別 | 缺口 | 解鎖/處置 |
|---|---|---|---|
| XP-001 | P1 | S2 桌面殼體驗未規格化：首啟、更新（passive）、錯誤態演練未執行 | A-2 桌面殼演練窗口；排程中 |
| XP-002 | P2 | S4 報表列印：分頁不截斷/頁眉頁腳/黑白可讀未驗（基礎規則已在） | E-4 backlog；排程中 |
| XP-003 | P2 | S5 匯出規範未成文 | E-2 §12 已訂規則；實作抽驗待做（部分） |
| XP-004 | P3 | （接受，非缺口）pre-auth 家族（login/forgot/reset）skip-link 與 app-header：N/A——無重複區塊可跳、獨立品牌版型為刻意設計 | 已接受（N/A 附理由） |
| XP-005 | P1 | 文件受眾標籤未全面（操作者/管理者/技術者/稽核） | A-3/E-6；排程中 |
| XP-006 | P2 | 互動教學（TT-9 候選）與工作流設計器（TT-10 候選）未實作（使用者明示需求） | P2 路線（UX_CONTRACTS 已知限制） |
| XP-007 | P2 | R6–R8 輪次（E2E 全走查/跨裝置/效能資源）本輪未執行 | UNKNOWN（解鎖：排定 R6–R8 驗證窗口） |
| XP-008 | P3 | 證據 raw log 之 CJK 字元在部分主控台碼頁顯示為 ?（不影響 exit/ASCII 摘要） | 已緩解（ASCII 摘要為權威） |
| XP-009 | — | （已撤）初稿誤報「對比閘門缺席」——複驗更正：`platform/tests/design_a11y.rs` 存在 3 測試；誤報原因：初檢只搜檔案內容、未搜檔名 | 誤報已撤（保留列供追溯；不計入缺口數） |

## §5 覆蓋證明

- 八個來源逐源盤點（§1）；疊加「表面×契約零空格矩陣」與「角色×任務關鍵路徑 CP-01..CP-12」。
- 已修復（本輪前後，引用不自建）：UX_CONTRACTS §問題清單 13 項（含財務紅線 P0 兩項：草稿金額漏傳、AI 金額截斷）；ui-v2 子導覽碰撞 P0；404 品牌頁 P2。
- 未關閉：XP-001..003、005..007（詳 REGISTER；P1 全數有解鎖條件）；XP-009 為誤報、已撤。

## §6 R0–R9 本輪執行對照（誠實標註）

| 輪 | 內容 | 本輪狀態 | 證據 |
|---|---|---|---|
| R0 | 基準量測快照 | 部分（HEAD/套件數字；CSS bytes 未快照） | §2 |
| R1 | 編譯靜態（fmt/clippy/guards） | 執行（EXIT=0） | 驗收 log |
| R2 | 測試套件 | 執行（375/0） | #44 |
| R3 | 路由完整性（404/死鏈/manifest 對帳） | 執行（page_matrix 4/0、web_crawl 6/0） | 同上 |
| R4 | 權限矩陣 | 執行（design_lab 閘矩陣＋page_matrix 角色矩陣） | 同上 |
| R5 | 視覺/無障礙（對比/零 JS/focus） | 部分（零 JS 全掃＝guards；對比自動化＝design_a11y 3 測試綠；focus 全站走查未執行） | guards；design_a11y |
| R6 | 端到端全走查 | NOT TESTED（解鎖：排定窗口） | — |
| R7 | 跨裝置/列印 | NOT TESTED（解鎖同上；列印部分＝XP-002） | — |
| R8 | 效能資源（p95/bytes/位移） | 部分（壓力 p95 439ms 既有；頁面級預算見 SPEC §11） | REQ 矩陣 P-03 |

## §7 結論

體驗層由「四份文件分治」收斂為「規格 SSOT＋體驗室＋矩陣＋登錄＋自動校驗」五件套（本輪交付）；P0 全數為 0（既有 P0 皆已修），P1 缺口 2 項（XP-001／XP-005）均有解鎖條件與排程（XP-009 為誤報、已撤）；R6–R8 為下輪驗證主體。本稽核為 **CONDITIONAL PASS**：不得視為發布認證（五級判定用語依 C-13）。
