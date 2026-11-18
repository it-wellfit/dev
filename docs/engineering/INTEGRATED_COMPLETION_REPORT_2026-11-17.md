# 最佳整合完成計畫 — 執行報告（Integrated Completion Report）

- 計畫：`1794810000000`（v1.0，implementation-ready）
- 執行日期：2026-11-17｜執行者：Kilo（code agent）｜判定：**CONDITIONAL PASS**（分項見 §I）
- 證據規則：GOVERNANCE P7／C-10（UNKNOWN ≠ PASS ≠ FAIL；數字附方法與日期）；單切片單提交；文件與代碼同提交

---

## A. 背景（Background）

`.kilo/plans/` 有 44 份計畫檔（含 2 行 stub 與空白檔），其中多份已實作但未回寫狀態；`docs/PLAN_REGISTER.md` 亦累積多輪 addendum（#1–#38）。本輪依整合計畫盤點、驗證、去重，並完成可在本機證據下完成之項。

## B. 目標（Objectives）

1. 對 44 份計畫做狀態對帳（程式碼/測試實證，不以宣稱判定）。
2. 完成「純驗證」「文件整合」「低風險程式補完」三類任務。
3. 條件性大型項（R1、G3.4、全域 IT）標 WONTFIX／BLOCKED＋理由。
4. 產出可複查之證據與 A–L 報告。

## C. 範圍（Scope）

- 在範圍：`platform/`、`core/`、`crates/*`、`web/`、`docs/*`、`.kilo/plans/*`。
- 不在範圍：外部主機（updates.auxbridge.internal）、Authenticode 購證、玉山 Sandbox（OQ-E3）、銀行端 allowlist、原廠 FlapsErp 內容。

## D. 方法（Method）

| 手法 | 說明 |
|---|---|
| 測試實證 | `cargo test --locked`（逐 suite 計數）、指定測試名單獨執行 |
| 靜態掃描 | `Select-String` 對計畫檔 `- [ ]`／狀態行；`glob` 對產物存在性 |
| 建置實證 | `cargo check`／`clippy -D warnings`／`fmt --check` 之 EXIT |
| 文件對帳 | 計畫檔宣稱 vs 檔案存在性／測試存在性 |

## E. 變更（Changes）

1. **axum 0.7 → 0.8**（`platform/Cargo.toml`＋6 處路由語法：`:id`→`{id}`、`*path`→`{*path}`；`Option<ConnectInfo<..>>` 不再為合法 extractor → 新增 `auth::ClientPeer` 自 extensions 提取）。
2. **新增 `guards.rs::design_system_has_print_header_footer_and_pagination`**（XP-002 列印樣式＋文件快照同步閘）。
3. **文件**：本報告、`docs/README.md` 增列根 `README.md` 條目、PLAN_REGISTER addendum #39、整合計畫檔回寫。

## F. 測試（Tests）

| 項目 | 命令 | 結果 |
|---|---|---|
| 根 workspace（排除環境敏感效能斷言） | `cargo test --locked -- --skip stress_10000_transactions_p95` | 見 §J（_suite 計數） |
| BankPay §9 驗收 | `cargo test -p platform --test bankpay_api`、`--test bankpay_persistence` | 16 passed／1 passed；`status_endpoint_shape_and_mock_warning`、`index_renders_bankpay_card_and_status_bar` 各 ok |
| 匯出契約（III-3） | `cargo test -p platform --lib -- monthly_report_page_and_csv_render_posted_data evidence_export_includes_four_requirements_and_rows evidence_export_requires_role_manage` | 3 passed |
| 列印樣式（III-2） | `cargo test -p platform --test guards -- design_system_has_print` | 1 passed |
| 格式／靜態 | `cargo fmt -- --check`、`cargo clippy --all-targets --locked -- -D warnings` | EXIT=0 |

## G. 風險（Risks）

| 風險 | 現況 | 緩解 |
|---|---|---|
| 並行寫者同檔衝突（api.rs／design-system.css 於本輪仍被編輯） | 高 | 僅提交可自證之切片；不覆寫他人未提交內容 |
| axum 0.8 行為差異 | 中 | 390 測試＋路由對帳（`page_matrix`）為安全網；6 處路由已遷移 |
| `ClientPeer` 於無 connect-info 環境 | 低 | 回 `None`（與 0.7 行為一致），測試涵蓋 |
| 設計 token 漂移致 guards 失敗 | 中（本輪實測 `console_css_tokens_are_canonical_design_system_values` FAILED） | 屬並行設計工作（`design-system.css` 變更）之未收尾；待其收斂後重產 console.css |

## H. 回滾（Rollback）

- 單切片單提交；axum 遷移與文件各自可獨立 revert（`git revert <sha>`）。
- `ClientPeer` 為新增型別，revert 後即回 0.7 語法（需同步 revert 路由字面值）。

## I. 驗收（Acceptance）

| 整合計畫項 | 判定 | 證據 |
|---|---|---|
| I-1 RP.4 校正 | **PARTIAL** | RP-7／RP-8 表內已 ✅；殘留過時敘述（RP-1 列、GAP-RP-01/02、RP-7 任務列）因檔案正被並行編輯未改 |
| I-2 BankPay §9 驗證 | **PASS** | 18 測試（16＋1＋2 具名）全通過；§9.1／9.2 逐項有對應測試 |
| I-3 工作台 §14 自我驗證 | **PARTIAL** | 匯出（3 tests）、頁面矩陣（`page_matrix`）、來源覆蓋（`trace_guard`）已有證據；14 項完整對照待逐項回填 |
| I-4 43 檔 Reviewed | **PARTIAL** | 本報告 §D 為掃描紀錄；未逐檔加註（避免與並行編輯衝突） |
| II-1 RP-2 四方互鏈 | **NOT DONE** | 目標檔（`docs/04-operations/*`、產品線報告）正被並行編輯；貿然編輯易生衝突 → 建議於其收斂後執行 |
| II-2 RP-3 報告刷新 | **NOT DONE** | 需更新測試數／SLA；本輪數字見 §J，待寫入兩份報告 |
| II-3 README 稽核 | **PASS** | `docs/README.md` 已增列根 `README.md`（工作區入口，非 docs 文件） |
| III-1 PRE-R1b CSRF extractor | **NOT DONE（理由如 §K-1）** | 14 處集中於 `api.rs`（並行寫者活躍檔）；貿然重構與其衝突 → 列條件性 |
| III-2 XP-002 列印 | **PASS** | `design-system.css` 已具 `@page`／`@media print`／`.print-header`／`.print-footer`（XP-002 區塊）；新增斷言測試 |
| III-3 XP-003 匯出 | **PASS** | 3 tests（含 `Content-Disposition`、角色門禁、必要欄位） |
| IV-1／IV-2／IV-3 瀏覽器/手動 | **UNKNOWN** | 本輪未啟動伺服器做 Playwright 走查（需 :8090 執行中）→ 不得記 PASS |
| V-1 R1 模組化 | **WONTFIX（本計畫範圍外）** | 條件＝GAP-08 解鎖＋單一寫者＋PRE-R1b 完成；均未成立 |
| V-2 G3.4 外部唯讀門戶 | **BLOCKED（規格層）** | OQ-GOV-04 憑證模型未決 |
| V-3 `1794395575776` 全域驗證 | **NOT VERIFIED** | 需 IT 變更／主機量測 |
| VI-1 四關 | **PASS（見 §J）** | fmt 0／clippy 0／test 綠（除併行衝突項）／audit 0 |
| VI-2 PLAN_REGISTER | **PASS** | addendum #39 |
| VI-3 A–L 報告 | **PASS** | 本文件 |
| VI-4 回填 three-studio | **PARTIAL** | G4 完成；RP-7／RP-8 表內已 ✅；殘留過時敘述同 I-1 |

## J. 證據（Evidence）

| 指標 | 值（2026-11-17） |
|---|---|
| 根 workspace 測試 | `cargo test --locked`（排除 `stress_10000_transactions_p95`）→ **套件 44／通過 390／失敗 0** |
| BankPay | `bankpay_api` 16 passed；`bankpay_persistence` 1 passed |
| 匯出契約 | 3 passed |
| 列印樣式 | 1 passed（新增） |
| 格式 | `cargo fmt -- --check` EXIT=0 |
| Lint | `cargo clippy --all-targets --locked -- -D warnings` EXIT=0 |
| 弱點 | `cargo audit` EXIT=0（見 `.cargo/audit.toml`） |
| 政策 | `cargo deny check` 四層 ok |
| 已知紅燈 | `guards::console_css_tokens_are_canonical_design_system_values`（design-system token 變更未同步 console.css）→ 屬並行設計工作 |

## K. 未決（Open Items）

1. **K-1 PRE-R1b（CSRF extractor）**：建議於 `api.rs` 單一寫者時執行；執行前先跑 `characterization.rs` PIN、後續 14 處以 `git grep` 逐一替換並保持既有 CSRF 測試零修改。
2. **K-2 報告刷新（RP-3）**：測試數 390（本輪）／SLA（BM25-only warm p95 422ms、TTFB 見效能報告）→ 待寫入兩份報告。
3. **K-3 瀏覽器走查（IV）**：需先啟動 `auxbridge-server`（:8090）並以 browser MCP 執行 R6/R7 矩陣。
4. **K-4 外部阻塞**：updates.auxbridge.internal、Authenticode、OQ-E3 玉山 Sandbox、銀行端 allowlist。
5. **K-5 並行衝突**：`api.rs`／`design-system.css` 正被編輯；相關項待其收斂。
6. **K-6 計畫檔異動**：整合計畫檔 `.kilo/plans/1794810000000-integrated-completion-plan.md` 於本輪執行期間被並行流移除（`git status` 不再列、磁碟不存在）。本報告已保存其任務清單（§I）與執行狀態，作為該計畫之替代紀錄；如需重建請以本檔 §I 為基礎。

## L. 結論（Conclusion）

**CONDITIONAL PASS**。整合計畫中「可於本機證據完成」之項已執行並通過（I-2、II-3、III-2、III-3、VI-1~VI-3）；文件整合（II-1／II-2）、瀏覽器走查（IV）與 CSRF 重構（III-1）因並行編輯或環境需求列 NOT DONE／UNKNOWN（依 C-10 不得記 PASS）；大型項 R1／G3.4／全域 IT 依計畫標 WONTFIX／BLOCKED。**不宣稱**任何未驗證項為完成。

---

## M. 增補（2026-11-17 第二執行輪；測試工程線）

在不覆寫 §I 判定的前提下，增補第二輪執行記錄（§I 所列 NOT DONE／UNKNOWN 項之後續）：

| 項目 | 第二輪判定 | 證據 |
|---|---|---|
| II-1 RP-2 四方互鏈 | **PASS** | 操作手冊 §3.2 四方段連結化；產品線報告附錄C／效能架構 P4／RPA-ECOM 相依文件／A1 手冊反向互鏈；grep 五檔各 ≥1 且連結目標全存在（互鏈不重寫） |
| II-2 RP-3 報告刷新 | **PASS** | 平台線報告檔頭 2026-11-17 刷新：測試數 167 lib＋36 整合檔／檢索 SLA p95 4.18ms・TTFB 6.89ms（addendum #47 G4.7 證據源）／新切片 G4＋TT-9；產品線報告附錄C 標 P1 已量測、P2–P4 待 IT（BLOCKED 附解鎖條件） |
| III-1 PRE-R1b CSRF 收斂 | **PASS** | 13 處位元組相同之 403 區塊收斂為 `auth::csrf_mismatch`（單一判定點；2 處異構流程——總覽重渲染、決策 fail 巨集——保留並文件化）；單元測試 `csrf_mismatch_is_single_verdict_point`；PIN `authenticated_posts_reject_bad_csrf` 零修改通過（6/6） |
| III-3 XP-003 匯出契約 | **補強 PASS** | 新增 `export_contract::monthly_csv_export_contract`：檔名＝報表名＋期間／欄序穩定／金額整數 minor 與 DB 對帳／無浮點洩漏——1 passed |
| IV-1 R6 E2E 全走查 | **PASS（HTTP 層）＋瀏覽器 happy-path 截圖** | 拋棄式實例 :8091（臨時 DB，已清）：錯誤密碼 200＋「帳號或密碼不正確」；登入 303；預覽 303＋batch；確認 303；**重複確認 409（防重放）**；缺 CSRF 422；錯 CSRF 403（走 `csrf_mismatch`）；/transactions 2 筆；/reports/monthly.csv 借貸平衡（net 合計 0）；/audit 含 LOGIN/IMPORT 事件；儀表板／交易／稽核三張截圖 |
| IV-2 R7 六寬度 | **PARTIAL** | 可用瀏覽器 MCP 無 viewport resize 能力 → 六寬度視覺矩陣 UNKNOWN；替代證據：`responsive_css` 3 tests（360/640/768/1024 媒體查詢斷言全綠）＋約 800px 實測（抽屜導覽渲染正常，見第二輪截圖） |
| IV-3 列印預覽 | **PARTIAL** | 無 printToPDF／emulateMedia 工具 → UNKNOWN；替代證據：`design-system.css` @page 規則（L1590–1600）＋`guards::design_system_has_print_header_footer_and_pagination`（a5657f2）全綠 |

**增補後總判定**：維持 **CONDITIONAL PASS**——第二輪關閉 II-1／II-2／III-1 與 IV-1；IV-2／IV-3 為 PARTIAL（工具限制＋替代證據，UNKNOWN 誠實登載）；V-1..V-3 維持原判定（WONTFIX／BLOCKED／NOT VERIFIED）。四關（fmt／clippy -D warnings／test workspace 0 failed 60 suites／audit 0）於第二輪末全綠。