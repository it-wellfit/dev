# 零 JS 營運控制台「27+ 頁一次交付」收斂執行計畫

> 計畫編號：1793708660255-ops-console-27page-delivery｜版本 v1.0｜日期：2026-11-09｜狀態：**implementation-ready**
> 上位 SSOT（**不重複、必先讀**）：
> 1. 範圍與已拍板決策 D1–D8：`.kilo/plans/1793034142477-apihub-tauri-uiux-integration-plan.md` v1.3（§3.5 非協商約束、§3.6 角色×工作流×情境矩陣、D7 二十七頁清單、§5 T1–T8、§8 決策記錄）。
> 2. 設計系統與畫面登記：`.kilo/plans/1792865422970-three-studio-governance-platform.md` v3.1（§11、§26、§27 SCR-01..24）。
> 3. 執行現況：`docs/PLAN_REGISTER.md` addendum #18（受眾分層 IA、治理四頁、重複路由前例）。
> 本檔＝「現狀校正＋收斂缺口＋矩陣驗收」的可執行收斂計畫；範圍/決策/約束以上位為準，本檔不推翻、不複製。
> 欽定約束（使用者 2026-11-09）：零 JS（HTMX/Datastar/WASM 不算）；**禁用耗費效能的 JS 工具鏈**（Node/npm/Vite/Webpack/esbuild）；**Electron 不採用**；效能/安全/穩定/易維護/易持續演進與優構。
> KEM 控制台（`knowledge/console/`，44 頁、擴展律 pages=21+claims+ceil(ledger/200)、19,683 級可達）**已交付且不在本範圍**，僅為架構先例。

---

## §0 現狀核查（2026-11-09 實測；實作前先重新量測，本節為基線）

| # | 事實 | 證據（命令/檔案:行） |
|---|---|---|
| F1 | **建置紅**：`cargo check -p platform` EXIT=101，17 errors＝E0609×12／E0599×4／E0308×1，全為 Askama 模板↔結構漂移。落點：`apihub.rs` L399（TestParam.description）、L413×2（AuditLogRow.event_type/time）、L420（ApiConfigTemplate.config）；`pages.rs` L182（AuditView.username）、L432×3（BatchRowFull.amount_display/status_class、auto_escape）、L456×3（Option.is_empty、AuditView.action/created_at）、L494×2、L519×2、L786（E0308 &String vs &&Concat）。警告 2：`pages.rs:9` unused HeaderMap、`workbench.rs:17` unused Deserialize | 本會話驗證代理：cargo check -p platform, exit=101 |
| F2 | **路由重複註冊 ×4**：`/accounts`、`/accounts/toggle`、`/ai-decisions`、`/ai-decisions/reject`（`api.rs` L409–412 與 L444–447 各一份）。axum 同路徑同方法二次註冊＝啟動 panic（前例：addendum #18 修復 `/bankpay/exceptions` 等三路徑） | api.rs:409–412, 444–447 |
| F3 | **`GET /bankpay` 缺路由**：handler `bankpay/api.rs:989 bankpay_page`＋模板 `bankpay.html`＋測試 `bankpay_api.rs:555` 均在，api.rs 無註冊 → 測試與導覽（index.html L47）將 404 | grep 全 src：`"/bankpay"` 僅測試與 handler 定義 |
| F4 | **`/accounts/:id` 缺路由與 handler**（D7 第 5 頁「科目詳情＋餘額」）；`account_detail.html` 為孤兒模板（零 derive 引用）；現行近似為 `/reports/ledger`（account_ledger.html） | grep：無 AccountDetail 匹配 |
| F5 | **模組路由雙來源**：`apihub::router()`（apihub.rs L893–907）、`workbench::router()`（workbench.rs L81–83）為 public 且註記「require_session 由 api.rs 統一掛載」，但 api.rs 以 inline 另行註冊（L463–475）＝維護雙點（未 merge，暫不致 panic） | 兩檔+api.rs 對照 |
| F6 | **`/audit` 未補 Tier 2 閘**：audit_page 無 ROLE_MANAGE 檢查；導覽 L48 對全體顯示（addendum #18 建議未落地） | pages.rs:208–254；index.html L48 |
| F7 | **工作樹 78 項變更**（46 M＋32 ??）；`platform/src/pages.rs`、`workbench.rs` 與多數新模板未入版控；HEAD=`2e18204`（message「okk」為並行流雜訊提交） | git status／git log |
| F8 | 既有護欄可用：R5 guards（eb522b2：編碼損傷＋script/inline 掃描）、PRE-R1a 特徵化 PIN（92cc577：27 endpoints）、`bankpay_persistence.rs`／`bankpay_api.rs`／`close_out.rs`／`ledger_slice12.rs` | git log |
| F9 | **導覽未達全頁可達**：T1 群組缺 `/new`、`/reports/trial-balance`、`/reports/ledger`、`/bankpay/batches`、`/bankpay/exceptions`、`/import/history`、`/profile`；管理群組缺 `/admin/roles`、`/admin/sessions`、`/admin/system`、`/admin/parse_coverage`、records/codes/release/net 入口 | index.html L40–78 對照路由表 |

---

## §1 交付定義（DoD）

1. **頁面總表 40 條 GET**（39 受保護＋`/login`）＋2 條 POST 呈現頁（bulk_review、ai_suggestion）全數存在且通過角色矩陣；F3/F4 缺口關閉。
2. **角色矩陣**（測試 SSOT）：訪客→303 `/login`；ALL 頁→任一登入者 200；READ 頁→store 200；WRITE 頁→store 403；MANAGE 頁→finance/store 403；admin 全 200（allow/deny 一律 server-side）。
3. **零 JS**：全部模板與全部 HTTP 回應 body `<script`=0、inline `on*=`=0；每頁回應斷言安全標頭（CSP `default-src 'none'`、no-store、nosniff、DENY、no-referrer）。
4. **編碼乾淨**：全模板與 src 無 BOM／NUL／PUA（UTF-8 無 BOM 寫檔）。
5. **八態掛點**：registry 每頁含 empty/denied/error 三錨點（共享 partial 提供）；`states.html` 為樣板 SSOT。
6. **四關 EXIT=0**：fmt／clippy `-D warnings`／test `--locked`／audit；警告 0。
7. **版控與文件**：分批提交；PLAN_REGISTER addendum #19＋CHANGELOG＋PROJECT_STATUS/NEXT_ACTIONS＋追溯矩陣更新。
8. **證據紀律**：每聲明＝命令＋exit code＋（可行時）修訂版；UNKNOWN≠PASS；不得虛報。

---

## §2 頁面登記總表（registry 候選；W5 入碼為 SSOT）

分層：T1 操作者／T2 管理者／T3 技術人員。閘別為矩陣期望（W6 逐頁對齊，差異需記錄）。

| # | 路由 | 模板 | 分層 | 閘別 | 現況 |
|---|---|---|---|---|---|
| 1 | `/login` | login | 公開 | 訪客200/已登入導離 | 既有 |
| 2 | `/` | index | T1 | ALL | 既有 |
| 3 | `/transactions` | transactions | T1 | READ | 既有 |
| 4 | `/transactions/:id` | transaction_detail | T1 | READ | 既有 |
| 5 | `/new` | wizard | T1 | WRITE | 既有 |
| 6 | `/import` | import | T1 | WRITE | 既有 |
| 7 | `/import/history` | import_history | T1 | READ | 既有（F9 導覽缺） |
| 8 | `/drafts` | drafts | T1 | READ | 既有 |
| 9 | `/accounts` | accounts | T1 | READ | 既有 |
| 10 | `/accounts/:id` | account_detail | T1 | READ | **缺口 F4** |
| 11 | `/search` | search | T1 | ALL | 既有 |
| 12 | `/ai-decisions` | ai_decisions | T1 | READ | 既有 |
| 13 | `/reports/monthly` | report_monthly | T1 | READ | 既有 |
| 14 | `/reports/pivot` | report_pivot | T1 | READ | 既有 |
| 15 | `/reports/parallel` | report_parallel | T1 | READ | 既有 |
| 16 | `/reports/trial-balance` | trial_balance | T1 | READ | 既有（F9 導覽缺） |
| 17 | `/reports/ledger` | account_ledger | T1 | READ | 既有（F9 導覽缺） |
| 18 | `/bankpay` | bankpay | T1 | READ | **缺口 F3** |
| 19 | `/bankpay/batches` | bankpay_batches | T1 | READ | 編譯阻塞（F1） |
| 20 | `/bankpay/batches/:batch_id` | bankpay_batch_detail | T1 | READ | 編譯阻塞（F1） |
| 21 | `/bankpay/exceptions` | bankpay_exceptions | T1 | READ | 編譯阻塞（F1） |
| 22 | `/audit` | audit_page | T2 | **MANAGE** | 待補閘 F6 |
| 23 | `/help` | help | T1 | ALL | 既有 |
| 24 | `/status` | status_page | T1 | ALL | 編譯阻塞（F1） |
| 25 | `/profile` | profile | ALL | ALL | 既有（F9 導覽缺） |
| 26 | `/states` | states | T3 | ALL | 既有（樣板 SSOT） |
| 27 | `/admin/users` | admin_users | T2 | MANAGE | 既有 |
| 28 | `/admin/roles` | admin_roles | T2 | MANAGE | 既有（F9 導覽缺） |
| 29 | `/admin/sessions` | admin_sessions | T2 | MANAGE | 既有（F9 導覽缺） |
| 30 | `/admin/system` | admin_system | T2 | MANAGE | 既有（F9 導覽缺） |
| 31 | `/admin/parse_coverage` | parse_coverage | T2 | MANAGE | 既有（F9 導覽缺） |
| 32 | `/admin/api/catalog` | api_catalog | T3 | MANAGE | 既有 |
| 33 | `/admin/api/test` | api_test | T3 | MANAGE | 既有 |
| 34 | `/admin/api/logs` | api_logs | T3 | MANAGE | 既有 |
| 35 | `/admin/api/config` | api_config | T3 | MANAGE | 既有 |
| 36 | `/admin/api/records` | api_records | T3 | MANAGE | 既有（T2 新頁） |
| 37 | `/admin/api/codes` | api_codes | T3 | MANAGE | 既有（T2 新頁） |
| 38 | `/admin/api/release` | api_release | T3 | MANAGE | 既有（T2 新頁） |
| 39 | `/admin/api/net` | api_net | T3 | MANAGE | 既有（T2 新頁） |
| 40 | `/tauri-preview` | （桌面對照） | T3 | MANAGE | §6-1 待拍板 |
| P1 | POST `/reviews/bulk` 呈現 | bulk_review | T1 | WRITE | 既有 |
| P2 | POST `/drafts/ai_suggest` 呈現 | ai_suggestion | T1 | WRITE | 既有 |

合計 **42 頁**（40 GET＋2 POST 呈現）≥ 27 門檻；交付標準由「合格化」定義（§1）。

---

## §3 工作項（依序；單一寫者原則；每項含 DoD）

**W1 編譯紅轉綠（F1）**——`pages.rs`／`apihub.rs` 模板↔結構對齊。
- 修法：優先對齊**結構欄位與模板既有引用**（資料可得者補欄位；模板端僅在結構不該有該欄時改模板）。
- Askama 0.16 陷阱（既有教訓）：屬性區禁 `{% %}`；`Option` 需 `if let`；禁對 `Option` 直呼 `.is_empty()`；模板變更後必要時 `cargo clean -p platform`（proc-macro 未追蹤）；寫檔 UTF-8 無 BOM。
- 一併清 2 警告（unused imports）。
- DoD：`cargo check -p platform` EXIT=0、警告 0。

**W2 路由單一化與缺口（F2/F3/F5）**
- 刪除 L444–447 重複塊（保留 L409–412）；確認全表無其他重複（`/api/*` 群組一併掃）。
- 補 `.route("/bankpay", get(crate::bankpay::api::bankpay_page))`＋import。
- api.rs 改 `.merge(crate::apihub::router())`＋`.merge(crate::workbench::router())` 取代 inline 註冊（單一來源；二選一之另一解為刪除模組 router，需同步其模組測試）。
- DoD：router 建構不 panic；`bankpay_api.rs:555` 測試綠；PRE-R1a PIN 復綠（PIN 變更須逐條記錄理由，禁盲改）。

**W3 `/accounts/:id` 新頁（F4，D7 第 5 頁）**
- 新 handler（mirror `account_ledger_page` 模式：session＋READ 閘、參數 fail-closed）；`account_detail.html` 收斂＝餘額摘要＋近期分錄＋「查看完整分類帳」連結至 `/reports/ledger?account=…`；與 `account_ledger.html` 重複部分擇一（建議保留兩頁分工：detail＝快照、ledger＝流水）。
- `/accounts` 每列加 detail 連結；未知 id→404 樣式頁（問題＋原因＋處理）。
- DoD：矩陣 200；餘額走 `format_minor`（i64 minor，禁浮點）。

**W4 `/audit` Tier 2（F6）**——handler 補 `ROLE_MANAGE` 閘（403 樣式頁）；導覽移入管理群組。DoD：finance/store 403、admin 200、導覽一致。

**W5 registry 入碼**——`platform/src/page_registry.rs`：`PageSpec{ path, template, tier, gate, title }`＋`pub const PAGES`；矩陣測試與導覽測試共用（供未來 nav 自動生成與規模化）。DoD：與 api.rs 路由表對帳通過（由 W8 測試證明）。

**W6 RBAC 對齊**——逐頁核 handler 閘與 §2 矩陣一致；不符者修正或（有理由時）回寫矩陣；**禁止虛構閘**，一切以程式碼為基線。DoD：矩陣測試全綠或差異全數具名記錄。

**W7 導覽完整化（F9）**——T1 補 `/new`、`/reports/trial-balance`、`/reports/ledger`、`/bankpay/batches`、`/bankpay/exceptions`、`/import/history`、`/profile`；管理群組補 `/admin/roles`、`/admin/sessions`、`/admin/system`、`/admin/parse_coverage`＋API 中樞子連結（records/codes/release/net）；維持 `can_manage_users` 閘控與操作者層不含開發文件連結。DoD：導覽可達測試綠。

**W8 測試矩陣（T4 完整化）**——`platform/tests/pages_matrix.rs`：
- 4 身份（visitor/admin/finance/store 種子）× 40 頁：303/200/403 期望斷言。
- 每回應：安全標頭（CSP/no-store/nosniff/DENY/no-referrer）、`<script`=0、UTF-8 無 NUL。
- 每頁 body 含 3 態錨點；導覽可達（GET `/` 掃 href 覆蓋 T1/T2 目標）。
- 效能：沿用「預熱＋p95」法（P3-1 樣式），新增 2 條——`/transactions` 與 `/audit` w/ 2k 交易 <500ms（SLA 不變）。
- DoD：矩陣全綠；證據 log 入 session tests/。

**W9 八態共享 partial**——`templates/partials/state_anchors.html`（empty/denied/error 掛點，`data-state-*`）；registry 模板 include；`states.html` 對照區維持樣板 SSOT；W8 斷言掛點。DoD：斷言綠。

**W10 護欄擴充**——R5 guard 掃描擴至**全 templates**（script=0、inline on*=0）；編碼掃描擴至全 templates＋src（BOM/NUL/PUA=0）；design token 雙副本漂移測試維持綠。DoD：護欄測試全綠。

**W11 四關＋證據**——隔離 `CARGO_TARGET_DIR`（避並行 server 檔鎖）；kem 並行線若阻塞 workspace 級測試，改泳道級執行並誠實記錄。DoD：fmt／clippy `-D warnings`／test `--locked`（0 failed/0 ignored）／audit 全 EXIT=0。

**W12 版控與文件**——分批提交（fix→feat→test→docs）；`.gitignore` 處置 `platform/data/*.db-shm|-wal`；PLAN_REGISTER addendum #19＋CHANGELOG＋PROJECT_STATUS＋NEXT_ACTIONS＋追溯矩陣。DoD：工作樹收斂且提交訊息具語義（非「okk」類）。

**W13（條件式）桌面面殘項**——`grep` conf 無 `1420`、CSP 無 script `unsafe-inline`、無虛報常數（`500000` 類）；NSIS 需工具鏈，缺工具鏈＝**UNKNOWN＋follow-up**（不虛報）。DoD：證據或誠實 UNKNOWN。

---

## §4 驗收命令（實作代理逐項執行並留痕）

```powershell
$env:CARGO_TARGET_DIR = "$env:LOCALAPPDATA\Temp\dev-target"   # 隔離，避檔鎖
cargo fmt -- --check                                          # EXIT=0
cargo clippy --all-targets -- -D warnings                     # EXIT=0（警告 0）
cargo test --locked                                           # 0 failed / 0 ignored（受阻時分泳道記錄）
cargo audit                                                   # EXIT=0（既有已記錄處置維持）
cargo test -p platform --test pages_matrix                    # 角色矩陣全綠
```

證據 log 依慣例入 `.kilo/sessions/<id>/tests/`；寫檔一律 UTF-8 無 BOM（`[IO.File]::WriteAllText`），寫後讀回驗證（桌面同步回滾風險之既有教訓）。

---

## §5 風險與界線

1. **並行流**：`pages.rs`／`apihub.rs` 為「雙作品」串流工作面（addendum #18）；開工前先確認其靜止或接管所有權（單一寫者）；本計畫即其收斂清單。
2. **F1 會漂移**：並行寫入可能改變錯誤清單；W1 以**重新量測**為準（本節為基線快照）。
3. **Askama 陷阱**：屬性區禁 `{%%}`、Option if-let、proc-macro 快取需 cargo clean；改模板後建議跑 W8 全矩陣。
4. **桌面路徑同步回滾**：單次原子寫入＋讀回；禁 `>` 重定向（編碼污染前例）。
5. **DB 執行期檔案**（flaps.db-shm/-wal）在 git status：以 .gitignore 或替換為測試路徑處置，禁提交。
6. **PIN 紀律**：characterization（92cc577）與既有斷言的變更須逐條說明理由；UNKNOWN≠PASS≠FAIL。

## §6 低風險開放點（實作時拍板並記錄於 addendum）

1. `/tauri-preview`（#40）：保留標內部（建議）或移除（若與 D8 無關聯需求）。
2. `account_detail` vs `account_ledger` 命名與分工（建議 detail=快照／ledger=流水）。
3. `/audit` 是否對 finance 開放（現依 addendum #18 建議 Tier 2＝不開放；如業務需要可再拍板）。
4. NSIS 工具鏈可用性（W13）。
