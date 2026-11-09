# Changelog — Dev 根工作區

本檔記錄根 workspace（`core/`、`platform/`、`web/`、`crates/flaps_bridge`）的實際變更。格式參考 Keep a Changelog。子專案（ekpi-platform、Wellfit、SyncMesh）各自有變更記錄。


## [Unreleased] — 2026-11-07（第十一輪：編碼根因修復＋驗證方法論補全＋效能量測法修正）

### Fixed
- **CP950 編碼損壞根因修復**：`platform/src/api.rs` 以最後乾淨提交 `dc414ac` 為源、`git merge-file` 三方合併還原（損壞自 `e82a49e` 起；Askama 分頁欄位 `page/total_pages/prev_page/next_page/total_transactions/pending_review_count` 隨合併復位）；`core/src/config.rs` 類別註解（ORG 組織／PERM 權限／ACT 動作／STORE 門市／POS 終端／INV 存貨）與 `crates/kem/src/main.rs` `P1 M2–M3`（U+E62F → en dash）PUA 字元清零。防制規範：`docs/encoding-standard.md`（PowerShell 禁 `>` 重導向寫碼，改 `[System.IO.File]::WriteAllText`＋UTF-8 無 BOM）。
- **Stress test 量測法修正**：`stress_10000_transactions_p95_under_500ms` 原 3 樣本 p95 退化為 max-of-3，首筆冷啟動查詢（SQLite 頁快取 miss，535ms）誤判 SLA 違規；改為**預熱趟（不计時）＋21 樣本 p95**（`ceil(21×0.95)-1=19`，容許單次排程抖動），**SLA 500ms 不變**；seed 改批次多列 INSERT（500 rows/語句，100k 交易由 ~90s 降至 ~2s）。修正後 warm p95 **422ms**。
- `NEXT_ACTIONS.md` 字面 `` `n `` 序列損壞（表格斷行遺留）重寫為合法表格。

### Added
- **突變測試（MT）**：cargo-mutants 27.1.0；`.cargo/mutants.toml` 官方 schema（examine_globs/exclude_re/additional_cargo_test_args）；補測 `amount_at_cap_posts_successfully`、`parse_i128_overflow_boundary_kills_boundary_mutants`、`money_error_display_is_not_empty`——money 73/73、engine 22/22 全殺（**≥98%→100%**，Missed 清零）。報告：`docs/engineering/MUTATION_SCORE_REPORT.md`（v2）。
- **模糊測試（FBT）**：cargo-fuzz 0.13.2＋nightly 1.101.0；`fuzz/fuzz_targets/money_parser.rs`（INV 非負＋round-trip）build PASS；run 受阻 Windows ASan DLL（`STATUS_DLL_NOT_FOUND` 0xc0000135）——`docs/engineering/FUZZING.md` §3 記錄三條解鎖路徑。
- **故障注入 SOP**：`docs/engineering/FAULT_INJECTION_SOP.md`（§10 追蹤表 FAULT-01..07；FAULT-01/02 DONE、04/05 IN PROGRESS）。
- **ATDD 綁定稽核**：`docs/engineering/ATDD_BDD_BIND_AUDIT.md`（8 情境 → Rust 測試；2 MISSING＋1 BLOCKED 列管）。
- **方法論補全**：DEVELOPMENT_METHODOLOGY §7（TDD 實證邊界＋跨域韌體/FPGA/SoC）、§8（32 項缺口表動態更新）、§9（Skills 修補）。
- **索引重整**：`docs/README.md` v3.0——死鏈（`01-onboarding/`、`02-architecture/` 從未存在）移除、新文件全收錄、superseded 標記規則入檔。
- **計畫登記**：`docs/PLAN_REGISTER.md` 2026-11-07 addendum #11（本輪執行事實）。

### Verified
- 四關全綠（2026-11-07）：fmt EXIT=0／clippy -D warnings EXIT=0／test --locked **157 passed 0 failed**（bridge 7＋core 42＋kem 53＋platform 48＋close_out 4＋slice12 3）／audit EXIT=0（1 個已接受警告 RUSTSEC-2026-0374）。
- 零 JS 護欄：templates script=0、內聯事件=0（未變動模板行為）。
- 並行工作線相容：Slice 13 搜尋引擎（`search.rs`＋`cross_module_api.rs`）與 kem evolve/gov 屬並行 session，本輪門檻在雙方穩定點量測；`target\debug\flaps-server.exe` 檔案鎖以隔離 `CARGO_TARGET_DIR` 迴避。

## [Unreleased] — 2026-11-06（第十輪：重新設計藍圖 R2/M1/M3＋TDD/BDD/SDD/DDD/ADD 體系）

### Added
- **REDESIGN_BLUEPRINT_v2.0**（docs/product/）：全面深化設計——架構 R1-R3、模組 M1-M6、介面互動設計、十二維度品質準則、硬性不變量。深化而非推翻：ADR-0001/0002/0003 決策全數保留。
- **PBT（proptest 1.12.0）**：core dev-dependencies 新增 proptest；8 個性質測試（parse/format round-trip i64+i128、非負、overflow、TooManyDecimalPlaces、empty、negative）全部通過；INV-01 零浮點護欄強化。
- **DDD 狀態機測試**：engine `status_transition_posted_to_pending_review`（POSTED→PENDING_REVIEW），core 31→39 測試。
- **CI 工作流**：.github/workflows/flexible-ci.yml 已在本機提交；推送至 origin 需 `workflow` scope（OAuth token 限制），本機已模擬 CI 四關全綠。

- **測試驗證計畫**（docs/testing/REDESIGN_TEST_PLAN_v2.0.md）與**交付清單**（docs/delivery/DELIVERY_MANIFEST_v2.0.md）。
- **開發方法論體系**（docs/engineering/DEVELOPMENT_METHODOLOGY.md）：TDD 紅綠循環（強制基線）＋BDD 場景註冊表（docs/testing/BDD_SCENARIOS.feature，8 Scenario／10 綁定測試，全部驗證存在）＋SDD 追溯表（需求→規格→測試→證據）＋DDD 界限上下文圖與聚合不變量審計＋ADD 品質屬性戰術表。

### Changed
- **M1 交易分頁**（TDD，RED→GREEN）：RED `pagination_shows_twenty_rows_per_page_with_page_two`（首頁曾全量列出）→ GREEN `engine::query_transaction_summaries_paged`（LIMIT/OFFSET 全 bind）＋`post_json` `?page=N`（非法→第 1 頁、越界→末頁）＋模板分頁導覽（`{{ page }}/{{ total_pages }}`，首末頁禁用態）。測試：3 整合＋`parse_page` 7 斷言。
- **M2/R2 健康檢查強化**（TDD，RED→GREEN）：RED `health_reports_degraded_when_database_unreachable`（預期 200 ok）→ GREEN `/api/health` 回報 migration 版本＋交易總數；DB 不可達時 503＋status=degraded（部署探測可用）。測試：`health_reports_*`×2。
- **M3 待覆核 KPI**（TDD，RED→GREEN）：RED `pending_review_kpi_shows_real_count_instead_of_placeholder`（KPI 顯示「—」）→ GREEN `build_index` 加 `COUNT(*) WHERE status='PENDING_REVIEW'` ＋`IndexTemplate.pending_review_count` ＋模板 `{{ pending_review_count }}`（0 起跳，斷線時仍為「—」）。測試：`pending_review_kpi_shows_real_count_instead_of_placeholder`。
- `.pagination` 元件樣式（tabular-nums 頁碼、禁用態）。

### Verified
- 四關全綠：fmt EXIT=0／clippy -D warnings EXIT=0／test --locked **98 passed 0 failed**（90→98）／audit 1 allowed warning（既有）。
- 零 JS 護欄：templates script=0、內聯事件=0。
- 併行工作流相容：governance-hub（Phase D3）與模組化重組之變更與本輪全部共存且測試通過。
## [Unreleased] — 2026-11-05（第九輪：Slice 12 Demo Kit 完成＋Phase 0 藍圖＋Phase 2 閘門）

### Added
- Demo Kit：`scripts/seed-demo.ps1`＋`reset-demo.ps1`（可重複展示；[DEMO] 資料經正式 API 種入，10/10 交易；demo-finance/demo-viewer 帳號）。
- Demo 文件：DEMO_GUIDE／DEMO_SCENARIOS／DEMO_DATA／DEMO_LIMITATIONS（誠實揭露限制）。
- Phase 0 藍圖文件：PRODUCT_VISION／USER_STORIES（US-1..16）／PRODUCT_ROADMAP（六階段閘門）／BUSINESS_MODEL（假設標示）／INFORMATION_ARCHITECTURE（22 路由 sitemap）／SYSTEM_ARCHITECTURE／DATABASE_DESIGN／API_DESIGN／TECH_STACK／TECHNICAL_DECISIONS（D-1..D-12）。
- 進度制度：`docs/PROJECT_STATUS.md`＋`docs/NEXT_ACTIONS.md`。
- Phase 2 閘門：`docs/production/PRODUCTION_READINESS.md`（Go/No-Go 矩陣 25 項；部署/監控/還原演練標示需授權——**NO-GO**）。
- 啟動時過期會話清理（`AppState::new` → `purge_expired_sessions`）。
- 遷移 0004：查詢索引（平行工作階段交付，已稽核入版控）。

### Fixed
- 桌面路徑檔案寫入回滾事故（測試檔合併）——以位元組級腳本＋讀回驗證修復。

### Tests
- 90 passed, 0 failed（含遷移 0004 重放）；基線四項 EXIT=0 @ 07b1d9c。
## [Unreleased] — 2026-11-04（第八輪：完成度稽核收尾）

### Added
- 值物件 Money／AccountId（DDD 戰術模式，建構即驗證）＋測試（core/src/value_objects.rs）。
- 樞紐列上限（OQ-10-2）：top-20 依合計，其餘摺疊至 OTHER 列＋截斷提示。
- AI 建議請求稽核（OQ-10-1）：AI_SUGGEST_REQUESTED（記錄 text_len/prompt_version/schema 結果；原文遮罩）。
- 管理頁密碼重設表單（P1-8 殘餘 UI）；草稿頁 AI 建議表單（OQ-8-2 UI）。

### Fixed
- CSRF 覆蓋缺口：import_preview（multipart）與 ai_suggest 補同步器驗證——POST 路由覆蓋 10/10。
- persist_batch 改 ImportBatchStaging（最後一處 too_many_arguments 豁免移除）。
- PIV-003 斷言修正為整數分語義。

### Tests
- 73→87 passed；基線四項 EXIT=0 @ c932753。
## [Unreleased] — 2026-11-04（第七輪：Slice 11 部署硬化）

### Added
- 安全標頭 middleware：CSP（default-src 'none'、style-src 'self'、img-src 'self' data:、無任何 script 來源）、nosniff、DENY、no-referrer、no-store（全回應）。
- 過期會話清理：登入邊界（新會話建立時刪除過期列）。
- Secure cookie 旗標（erp_config security/cookie_secure；預設關，TLS 部署時啟用——R-07 準備）。
- 根 README.md（Directive 13.1 全欄位）。

### Changed
- 模板 9 處內聯 style 屬性全數改為設計系統 utility classes（嚴格 CSP 成立）。

### Fixed
- 停用帳號安全缺口：停用後舊會話仍有效至自然過期（Slice 9 已閉，本輪補測試）。

### Tests
- 73 passed；基線四項 EXIT=0 @ 26bff52。
## [Unreleased] — 2026-11-04（第六輪：Slice 10 多維洞察進階與方法論硬化）

### Added
- 樞紐分析：GET /reports/pivot（白名單維度 R-PIV-WHITELIST，矩陣＋合計；PIV-003 上限摺疊 OTHER）。
- 平行座標投影：core/src/parallel.rs（整數 min-max 正規化，零浮點）＋GET /reports/parallel（每科目一折線＋無障礙 title）。
- AI 建議草稿（特性旗標預設關）：POST /drafts/ai_suggest（schema 閘門 parse_ai_suggestion，畸形輸出拒絕）＋建議卡（需人工覆核）＋POST /drafts/adopt（唯一入徑，永不自動落地）。
- Builder：TransactionBuilder／TransactionItemBuilder（P2-1 清償；兩處 too_many_arguments 豁免移除）。
- 解析覆蓋率：GET /admin/parse_coverage（ROLE_MANAGE；COV-001）。
- 方法論文件：docs/quality/bdd_acceptance.md、docs/testing/TRACEABILITY_MATRIX.md、docs/quality/ubiquitous_language.md。

### Changed
- Transaction/TransactionItem 建構改用 Builder（create_transaction、import_confirm、engine 測試；語義不變）。

### Removed
- 兩處 #[allow(clippy::too_many_arguments)]（allow 歸零）。

### Tests
- 70→73 passed；基線四項 EXIT=0 @ 7eb5c1d。
## [Unreleased] — 2026-11-04（第五輪：Slice 9 硬化收尾）

### Added
- 帳號停用即撤銷會話（P1-9）：admin/users/toggle 停用時刪除該帳號全部工作階段。
- 管理員密碼重設（P1-8 殘餘）：POST /admin/users/reset_password（ROLE_MANAGE、CSRF、≥8 字元）——Argon2id 輪換＋目標會話撤銷＋PASSWORD_RESET 稽核。
- 報表效能預算量化（P3-1/RPT-001）：close_out 測試以 recursive CTE 播種 2,000 筆平衡交易，斷言 /reports/monthly < 500ms。
- 設計 Token 同步迴歸測試（P3-2）：web/assets 與 docs 的 :root 區塊逐字一致。

### Fixed
- 停用帳號安全缺口：此前停用僅改旗標，舊會話仍有效至自然過期。

### Tests
- 66→70 passed；基線四項 EXIT=0 @ bc68b7e。
## [Unreleased] — 2026-11-04（第四輪：Slice 8 資料獲取與洞察層）

### Added
- **快速匯入（Layer A）**：兩段式 CSV/TSV 管線——`POST /import/preview`（multipart ≤2MB／5,000 列，逐列規則驗證、乾跑報告、SHA-256 指紋，**零帳務副作用**）→ `POST /import/confirm`（CSRF、狀態機重放防護 409、僅合格列登帳、批次稽核 `IMPORT_CONFIRMED`）。
- **引導式登帳精靈（Layer B）**：`GET /new` 三步流程（模板選擇→借貸填寫→確認摘要）；錯誤停留原步並顯示**規則名稱**（WIZ-001）；支援 `from_draft`／`from_batch` 預填並標示來源。
- **草稿解析（Layer C）**：`flaps_core::parser`——規則表驅動的非結構化文字解析（日期三格式、金額、借貸箭號、科目關鍵字對映），每欄位附**依據卡**（規則＋命中文字）；`POST /drafts/parse` 儲存草稿，**人工確認必經**，AI 不直接落地（H3 模型不變）。
- **洞察層（Layer D）**：dashboard 伺服器端 **SVG 月度借貸長條圖**（零 JS，僅整數聚合生成，safe filter 已審核）；`GET /reports/monthly`（科目借貸彙計）與 `.csv` 匯出（LEDGER_READ、整數分）；`@media print` 列印樣式。
- `migrations/0003_import_and_drafts.sql`（import_batches/import_rows/drafts）＋重放測試。
- Slice 8 驗收測試：IMP-001~003、WIZ-001、DRAFT-001、RPT-001、ZJS-001（`<script>` 計數 = 0 含 SVG 頁）。

### Security
- 匯入預檢無副作用、確認有重放防護、批次級稽核事件；報表與匯出受 LEDGER_READ 把關；解析器純規則（AI 建議與落地分離）。

### Notes
- 互動式 3D 視覺化與 Strict Zero-JS 憲章衝突，裁定為 Open Question OQ-8-1（零 JS 內以 SVG＋CSV 出場替代；豁免需另立 ADR）。

## [Unreleased] — 2026-11-04（第三輪：治理、防爆破、帳號管理）

### Added
- **根 workspace 版控（P0-2）**：`git init -b main`＋初始提交 `cd66388`——.gitignore 裁定範圍（排除 6 個嵌套獨立 repo、計畫目錄、target 7GB、本機 DB）；staged 內容安全掃描（無真實秘密）；此後證據綁修訂版。
- **登入速率限制（P1-7）**：`LoginRateLimiter`——每帳號 5 次連續失敗鎖定 15 分鐘（成功重置；鎖定訊息含剩餘秒數；大小寫不敏感；檢查先於憑證驗證）。
- **帳號管理 UI（P1-8）**：`/admin/users`（清單＋建立＋啟停），ROLE_MANAGE **即時 DB 查詢**把關；使用者名稱規範（3–32、小寫/數字/_/-）、密碼 ≥8、角色白名單；自我停用拒絕；全表單 CSRF。

### Changed
- RBAC 檢查改為**即時 DB 查詢**（`has_permission_db`）——角色指派於啟動後即時生效（原 in-memory 快照不一致）。
- 移除無人使用的 tauri workspace 依賴（P2-2；Cargo.lock tauri 條目歸零）。

### Security
- 暴力破解防護（P1-7）＋帳號停用（停用後登入被拒，測試覆蓋）。

## [Unreleased] — 2026-11-04（第二輪：複式記帳＋認證）

### Added
- **借貸平衡強制（P1-5，INV-02）**：`validate_journal_entry` 強制 ΣDEBIT = ΣCREDIT 且 >0（i128 累加）；表單升級為借方/貸方雙欄（同科目拒絕）；交易清單金額改為借方合計語義。
- **認證與會話（G-1，ADR-0003）**：`platform/src/auth.rs`——Argon2id 密碼雜湊、伺服器端會話（24h、可撤銷）、`require_session` middleware、登入/登出（PRG）、同步器 CSRF token（登出＋交易表單）、首次啟動管理員播種（`FLAPS_ADMIN_PASSWORD` 或隨機一次性列印）。
- `migrations/0002_users_sessions.sql`（users＋sessions 表）。
- 登入頁 `platform/templates/login.html`（零 JS，錯誤泛化防枚舉，密碼永不回顯）。
- 引擎負向測試 8 條＋整合測試 13 條（未認證重導、CSRF 403、登出撤銷、RBAC 403、平衡端到端）。

### Changed
- 身分來源改為伺服器端會話（`Extension<Session>`）；**移除可偽造的 `X-Acting-User` header**。
- `AppState`/`LedgerEngine`/`PermissionManager` 加 `Clone`（router state 型別需求）。
- 路由拆分 public/protected（`require_session` middleware）。
- `web/index.html` 表單同步升級為借貸雙欄（靜態基線與模板一致）。

### Security
- THREAT_MODEL：T-01 已緩解（認證落地）、T-08 已緩解（CSRF token）、R-06 Resolved；新增 R-07（Secure cookie 待 TLS）。

## [Unreleased] — 2026-11-04（第一輪：整數貨幣＋API 層）

### Added
- **零 JS API 層（ADR-0002）**：`platform/src/bin/flaps-server.rs`（Axum 0.7 伺服器）與 `platform/src/api.rs`（路由與處理器）：
  - `GET /`：Askama 0.16 伺服器渲染全頁（模板 SSOT：`platform/templates/index.html`，自動 HTML escaping）。
  - `GET /assets/design-system.css`：編譯期嵌入（零路徑遍歷面）。
  - `GET /api/health`：JSON 探針（db 連線狀態＋版本）。
  - `POST /transactions`：表單 → RBAC（`LEDGER_WRITE`）→ 驗證 → 冪等登帳 → 303 PRG；失敗回完整頁面並保留表單值（400/403/500 分級）。
- **整數貨幣模組（P0-1，INV-01）**：`core/src/money.rs`——`parse_decimal_to_minor`（精確字串解析，拒 >2 位小數／負數／溢位，繁中錯誤訊息）與 `format_minor`（純整數千分位格式化）。
- `LedgerEngine::with_pool`：不建表的推薦建構（schema SSOT 為 migrations）；`query_recent_transaction_summaries`（JOIN 彙總金額）。
- `TransactionSummary`（header＋items 彙總，供清單渲染）。
- 平台整合測試 8 條（PRG 303、RBAC 403、驗證 400 ×3、零 JS 渲染、health、CSS）＋遷移測試 4 條（建表/重放冪等/舊庫相容/整數 roundtrip）。
- `docs/adr/ADR-0001-tech-stack-baseline.md`、`docs/adr/ADR-0002-api-layer.md`。
- 治理文件：根 `AGENTS.md`、`docs/product/PRD.md`、`docs/product/USER_JOURNEYS.md`、`docs/product/IMPROVEMENT_BACKLOG.md`、`docs/security/THREAT_MODEL.md`、`docs/testing/TEST_STRATEGY.md`、`docs/engineering/SKILL_INVENTORY.md`、本檔。

### Changed
- **金額整數化（P0-1）**：`TransactionItem.amount_minor: i64`（原 `amount: f64`）、`exchange_rate_scaled: i64`（原 `exchange_rate: f64`）；migrations 0001 schema 改為 `amount_minor INTEGER NOT NULL`、`exchange_rate INTEGER DEFAULT 10000`；`core/src/ledger/engine.rs::ensure_tables` DDL 同步。
- 依賴安全：`reqwest` 0.11 → 0.12.28（h2 0.3.27 → 0.4.20，修復 RUSTSEC-2026-0258；移除 unmaintained 的 rustls-pemfile）。
- 根 `Cargo.toml`：移除 `[workspace.lints.clippy] all/pedantic = "allow"`（憲章 §2.2 零警告不變量）。
- `web/index.html`：標頭註記 Superseded——伺服器渲染 SSOT 為 `platform/templates/index.html`；本檔保留為離線設計基線。
- `docs/design-system.css`：頭註部署副本關係（SSOT：`web/assets/design-system.css`）。
- `SyncMesh/docs/SECURITY.md` v1.1.0：反模擬更正（TCP 明文現狀；TLS 1.3 為目標、AT-008 未驗證）。
- `docs/PLAN_REGISTER.md`：2026-11-03 addendum（基線恢復與 ADR-0001）。

### Fixed
- `core/src/permission.rs`：`initialize_default` 缺失閉合括號導致 workspace 無法編譯（語法損壞）。
- 19 個 clippy 錯誤（redundant_pattern_matching、needless_borrows_for_generic_args、too_many_arguments（帶理由豁免）、new_without_default、unnecessary_cast、for_kv_map、collapsible_match、manual_div_ceil、is_multiple_of）。
- flaky 測試 `batching_beats_per_call_under_measured_envelope`：牆鐘比較改為確定性往返計數（MockTransport 新增 `round_trips()`）。

### Security
- `cargo audit` EXIT=0：h2 升級修復 RUSTSEC-2026-0258；rsa RUSTSEC-2023-0071 經 `.cargo/audit.toml` 文件化接受（sqlx-mysql 未啟用、未編入產物）。
- 伺服器端 RBAC 強制＋輸入驗證 fail-closed（THREAT_MODEL T-01 部分緩解、T-02/T-06 控制）。

### Known Limitations（誠實記錄）
- 根 workspace 非 git repo（P0-2）：證據為指令＋退出碼綁定，非修訂版綁定。
- `X-Acting-User` header 未經認證（G-1/R-06）：對外開放前必須落地認證會話。
- 表單建立單筆分錄，未強制借貸平衡（P1-5）。
