# 統一營運控制台：單一 Console Manifest 雙渲染器交付計畫

> 計畫編號：1793033568961-unified-ops-console-manifest｜版本 v1.5｜日期：2026-11-10｜狀態：**implementation-ready（grill-me 全分支走完：A/發行鏈/B/C/D/E 已裁定）**
> v1.5 變更：grill-E 裁定 token **全收斂（名稱＋值）**——W7 昇格為完整映射表＋色差記錄；grill-B 裁定 NavModel middleware 注入（W2）；grill-C 消歧 POST 頁口徑（§3.2 分工原則）；grill-D 增 bootstrap admin 閘校準（W0）；W10 發行鏈現代化（v1.4）。
> v1.4 變更：上網實證 2026 發行鏈（B19/B20）；新增 W10 發行鏈現代化（updater＋簽署配置位＋NSIS 捆綁）；§7 增簽署採購與 feed 托管開放點。
> v1.3 變更：KEM 擴展律已裁決（讀畢 generate() 逐檔計數：**pages＝24＋claims＋⌈ledger/200⌉**，實測 45 回傳/44 HTML；文檔「≈21」過時）；新增 §3.1 NAV_GROUPS、§3.2 PAGES 登記種子表（41 筆）、§3.3 KEM 靜態清單；W2 導覽批次具體化；W3 補 accounts.html 無詳情連結實測。
> v1.2 變更：KEM 伺服模式已裁定（platform 受保護靜態路由 `/console/*path`）；新增 B18（KEM 生成器內聯樣式 vs 平台 CSP 衝突）；W6 擴充伺服設計與內聯歸零；§7.1 已決議。
> v1.1 變更：依 PLAN_REGISTER addendum #19（平行流 2026-11-10 交付）重測基線——F6 已閉環、建置已綠（145 測試 0 失敗）、page_matrix.rs 已存在、NavFlags 導覽機制已落地；本計畫收斂為「剩餘缺口＋manifest 統一層」。
> 上位 SSOT（**不重複、必先讀**）：
> 1. 範圍與決策 D1–D8：`.kilo/plans/1793034142477-apihub-tauri-uiux-integration-plan.md` v1.3（§3.5 非協商約束、§3.6 角色×工作流×情境矩陣、§9 P0/P1 已由 addendum #19 執行）
> 2. 設計系統與畫面登記：`.kilo/plans/1792865422970-three-studio-governance-platform.md` v3.1（§11、§26、§27）
> 3. 平台收斂缺口：`.kilo/plans/1793708660255-ops-console-27page-delivery.md`（本檔吸收其未竟項；該檔不刪除，實作時於檔頭標「已整合」）
> 欽定約束：零 JS（HTMX/Datastar/WASM 不算）；禁用耗費效能的 JS 工具鏈；Electron 不採用；效能／安全／穩定／易維護／易持續演進。
> 使用者裁定（2026-11-10）：採「**單一 Console Manifest 雙渲染器**」——一份 registry 為 SSOT，`renderer: Dynamic|Static`；platform 動態渲染營運頁、KEM 靜態渲染高基數頁；共用導覽與 design token。

---

## §0 現況基準（2026-11-10 深夜重測；含 addendum #19 已交付項；實作前 W0 仍須複測）

> 控制台＝`Dev/platform/`（根 workspace 成員：core/platform/web/crates/flaps_bridge/crates/kem）；`ekpi-platform/` 為獨立產品，不在本範圍。

| # | 事實 | 證據 |
|---|---|---|
| B1 | 控制台＝`Dev/platform/`：axum 0.7＋Askama 0.16＋sqlx 0.9(SQLite)，edition 2021 | `platform/Cargo.toml` |
| B2 | **建置已綠**（addendum #19 P0-3）：cargo check 0 錯；**145 測試 0 失敗**；fmt EXIT=0 | docs/PLAN_REGISTER.md addendum #19 |
| B3 | 模板 42 個＋`partials/studio_nav.html`；`#[template(path)]` derive 40 個；孤兒模板：`account_detail.html`（**已完整可用**，待綁路由）、`audit.html`（被 audit_page.html 取代） | templates 清單＋src 全掃 |
| B4 | **F6 已閉環**：`audit_page` 具 ROLE_MANAGE 閘＋`denied_page` 共用 403 輔助（Denied 八態語彙） | pages.rs L227–250；addendum #19 P0-2 |
| B5 | **CSP 內聯樣式已歸零**：`grep 'style="' platform/templates`＝0；index 內聯移入 `.audit-filter-form` 類別 | addendum #19 P0-1a；平台 CSP=`style-src 'self'`（auth.rs L265） |
| B6 | **F9 部分改善**：index 導覽角色過濾（`NavFlags`）＋補試算表/總分類帳/roles/sessions/system 連結＋`.nav-role-badge` | index.html L40–90；api.rs L881 |
| B7 | **F9 剩餘**：(a) `/audit` 連結在 `nav.ledger_read` 條件下渲染，但處理器閘為 ROLE_MANAGE——僅具 LEDGER_READ 者點擊得 403＝**死連結**（違反 NavFlags 自身註解「低角色永不見死連結」）；(b) `bankpay.html` 仍有自帶內聯導覽（未角色過濾）；(c) `partials/studio_nav.html` 仍為孤兒（零 import）；(d) 其餘 ~38 模板僅 header「返回工作台」、無站內導覽 | index.html diff；grep `site-nav`＝3 檔 |
| B8 | **F4 未修**：`/accounts/:id` 無路由、無 handler；`account_detail.html` 欄位契約已備妥：`found, account_code, account_name, account_type_label, currency, is_active, debit_display, credit_display, items[transaction_code,transaction_date,description,item_type,amount_display,currency], acting_user` | api.rs 路由表；account_detail.html |
| B9 | **F5 未修**：`apihub::router()`（apihub.rs:893）、`workbench::router()`（workbench.rs:80）、`auth::login_router()`（auth.rs:524）為**死碼**（零呼叫者；api.rs 重排後復測無引用）；api.rs inline 註冊為唯一活源 | grep api.rs 無 `apihub::router` 等 |
| B10 | **T4 矩陣已部分交付**：`platform/tests/page_matrix.rs`（未入版控）——訪客 303；admin 10 頁 200＋零 JS；store1 操作者頁 200、治理頁（audit/catalog/users）403。**缺**：finance 身份、安全標頭斷言、八態掛點斷言、導覽可達斷言、POST 呈現頁 | page_matrix.rs 全文 |
| B11 | 護欄：`guards.rs` SCAN_DIRS 已 5→7（＋`web/src-tauri/src`、＋`crates/kem/src`）；**`knowledge/console`（生成物）仍不在零 JS 掃描範圍**；token 雙副本漂移測試在位 | guards.rs；addendum #19 P1-3 |
| B12 | **KEM 控制台**：`crates/kem/src/console.rs` 生成。**擴展律已裁決（v1.3 讀畢 generate()）**：`pages += 1` 共 10 處（console.css 亦計 1），公式＝**24＋claims＋⌈ledger/200⌉**（claims=20、ledger=103 → 回傳 45；HTML 檔 44＝23+20+1）；文檔律「≈21+claims+⌈ledger/200⌉」**過時**（console.rs L8 與 index.html L425 之文案需同步更正） | console.rs L362/428/463/477/506/528/572/598/635/664 |
| B13 | **KEM console.css 自帶 token 詞彙**（`--bg/--ink/--accent…`，值異於 design-system：如 accent #0b5cab vs primary #2563eb）＝雙設計系統；design-system.css 有 dark mode 區塊 | console.css L1–9；design-system.css L228 |
| B14 | Tauri D1/D5/D8/T5 **已完成**：NSIS 3.82MiB 實證＋smoke（login 200＋CSP＋零 JS）；`scripts/tauri-shell-smoke.ps1`；white-screen 修復 | addendum #19 P1-1..P1-4 |
| B15 | 頁面統計：platform 動態 GET HTML 頁 **39**（含 /login）＋POST 呈現頁 ≥2（bulk_review、ai_suggestion；另有 /import/preview、/new/step2 重渲染 GET 模板——W0 精確盤點）；KEM 靜態 44；合計 **≈85 頁 ≥ 27 門檻** | api.rs 路由表＋console 清單 |
| B16 | PLAN_REGISTER 最新為 **addendum #19**（2026-11-10）；無 CI，四關本地跑並綁 session 證據 | docs/PLAN_REGISTER.md |
| B17 | `page_registry.rs` 不存在；八態共享 partial 不存在（partials/ 僅 studio_nav.html）；僅 5 模板帶 `data-state-*` 錨點 | glob＋grep |
| B18 | **KEM 生成器輸出含內聯樣式**（4 模式：`h2 style="margin-top:16px"`、`p.mono style="margin-top:12px"`、`div.mono style="font-size:1.5rem…"`、`pre style="white-space:pre-wrap"`；生成檔 21 處）——平台 CSP `style-src 'self'` 會阻擋，**伺服模式下視覺破損**；且違反設計 token 規範 | grep `style="` knowledge/console＝21；console.rs L324/326/340/385/655 |
| B19 | **發行鏈現代化查證（2026-11-10 上網實證）**：Tauri v2 官方 Windows 目標僅 NSIS/WiX MSI 兩種，**MSIX 無原生支援**（商店路徑＝EXE/MSI 提交）；NSIS 支援 `/S` 靜默、跨編譯、`installMode: portable`；現況 `tauri.conf.json`＝僅 `targets:["nsis"]`，**無 updater、無簽署、無第二目標** | v2.tauri.app/distribute/windows-installer（2026-11 更新）；tauri.conf.json L1 |
| B20 | **2026 專業發行鏈標準**：(1) 自動更新＝tauri-plugin-updater（minisign/Ed25519 簽章、不可停用）＋`latest.json` 靜態 feed＋`createUpdaterArtifacts`，Windows 端**重用 NSIS 安裝器**（passive 靜默）；(2) 簽署＝`bundle.windows.signCommand`（Azure Trusted Signing，微軟代管信譽）或 `certificateThumbprint`（本機憑證），**payload 每個 PE 都須簽**；未簽署＝非生產工件（SmartScreen）；(3) 更新簽章與 Authenticode 為兩條獨立鏈 | v2.tauri.app/plugin/updater；windows-production-release-guide（2026-06）；Release-Signing-Guide 範例 |

**19,683 的表述紀律**：19,683＝3⁹ 為「角色×工作流×情境」組合空間級數，不是已交付實體頁數。本計畫承諾：manifest 化頁面與測試矩陣覆蓋其可驗證子集；擴展律讓實體頁數隨資料有機成長至該量級而不需架構變更。

---

## §1 交付定義（DoD）

1. **單一 Manifest SSOT**：新 crate `crates/console_manifest`（零外部依賴或僅 serde），`PAGES`＋靜態族＋擴展律；platform 與 kem 皆依賴。
2. **導覽統一**：三處導覽定義（index 內聯、bankpay 內聯、孤兒 partial）收斂為 manifest 驅動之單一 macro（消費既有 `NavFlags`）；**死連結問題消除**（無權限入口不渲染）；所有認證頁具一致導覽與 `aria-current="page"`。
3. **缺口全關**：F4（`/accounts/:id`）、F5（死碼清除）、F9 剩餘（死連結＋bankpay 導覽＋孤兒 partial）。
4. **矩陣擴充**：`page_matrix.rs` 就地擴充（不另建檔）——＋finance 身份、＋安全標頭斷言、＋八態掛點、＋導覽可達、＋POST 呈現頁；既有斷言全數保持綠。
5. **KEM 接入**：kem 靜態渲染器由 manifest 取導覽/標題/擴展律；擴展律常數以實測重校準；`knowledge/console` 納入零 JS 護欄；**platform 以受保護靜態路由伺服之**（`GET /console/*path`，防路徑穿越＋副檔名白名單），生成頁內聯樣式歸零（CSP 相容）。
6. **設計單一身份**：console.css 採 canonical token（名稱＋值）；映射表逐項記錄；對比度維持 AA。
7. **四關 EXIT=0**：fmt／clippy `-D warnings`／test `--locked`（0 failed/0 ignored）／audit；警告 0。
8. **證據紀律**：命令＋exit code＋日期；UNKNOWN≠PASS；19,683 表述依 §0 紀律；PIN（characterization/page_matrix）變更逐條附理由。
9. **文件**：PLAN_REGISTER addendum #20＋CHANGELOG＋PROJECT_STATUS/NEXT_ACTIONS＋舊計畫檔頭標「已整合」。
10. **發行鏈現代化（W10）**：tauri-plugin-updater 接通（minisign 簽章驗證、latest.json feed、passive 靜默更新）；簽署配置位就緒（未購證前所有產物誠實標注 unsigned＝test-only，禁生產分發聲明）；NSIS resources 捆綁 knowledge/console（與 W6 接軌）。

---

## §2 架構決策（已拍板＋本輪補強）

**單一 Console Manifest、雙渲染器**（使用者 2026-11-10 裁定）：

```
crates/console_manifest（SSOT：id/path/method/title/group/renderer/tier/gate/workflows/靜態族/擴展律）
        │
        ├── platform（Dynamic）── nav::model(state,session) 消費 NavFlags 過濾 → 單一 nav macro
        └── kem（Static）─────── generate() 導覽/標題/頁數同源；擴展律可審計
```

**本輪補強（證據驅動）**：
- 導覽過濾**復用**平行流已交付的 `NavFlags`（ledger_read/ledger_write/manage_users/role_code）作為閘解析輸入——不另造權限查詢路徑（單一寫者禮讓＋最小 chum）。
- Gate 對映：`Public`→免登入；`All`→任一登入者；`Read`→LEDGER_READ；`Write`→LEDGER_WRITE；`Manage`→ROLE_MANAGE。**導覽可見性＝gate 過濾**，根治 B7(a) 死連結。
- 路由對帳測試採**源碼掃描**（沿 guards.rs 慣例：掃 api.rs 之 `.route("…")` 字串 ↔ manifest 動態 path 集合雙向相等），不新增 build.rs。
- **KEM 伺服模式（使用者 2026-11-10 裁定：由 platform 伺服）**：platform 新增 `GET /console/*path` 受保護靜態路由掛載 `knowledge/console/`——同源、導覽真正互通、安全標頭自動套用；設計細節見 W6。

---

## §3 Manifest 資料模型（實作契約）

```rust
// crates/console_manifest/src/lib.rs
pub enum Renderer { Dynamic, Static }
pub enum Tier { Operator, Admin, Technical }
pub enum Gate { Public, All, Read, Write, Manage }
pub enum Method { Get, Post }

pub struct PageSpec {
    pub id: &'static str,           // 穩定 slug："tx.list"、"kb.claim"
    pub path: &'static str,         // 動態："/transactions"；靜態：相對路徑樣式
    pub method: Method,             // POST 僅用於呈現型輸出（bulk_review/ai_suggestion）
    pub title: &'static str,        // zh-Hant
    pub nav_group: Option<&'static str>, // None＝不入導覽（詳情頁/login/states/tauri-preview）
    pub nav_href: Option<&'static str>,  // 導覽用（可帶查詢參數預設）；None＝用 path
    pub renderer: Renderer,
    pub tier: Tier,
    pub gate: Gate,
    pub workflows: &'static [&'static str],
}

/// 靜態高基數族（每筆主張一頁、帳本分頁）——manifest 不枚舉實例，只登記族與律。
pub struct PageFamily {
    pub id: &'static str,           // "kb.claim"、"kb.ledger-chunk"
    pub title: &'static str,
    pub cardinality: fn(claims: usize, ledger_entries: usize) -> usize,
}

pub const PAGES: &[PageSpec] = &[ /* 動態 41＋靜態固定頁（index/roles×5/workflows×13/queue/vectors/maintenance/claims-index） */ ];
pub const FAMILIES: &[PageFamily] = &[ /* kb.claim（=claims）、kb.ledger-chunk（=⌈ledger/200⌉） */ ];
pub fn static_console_pages(claims: usize, ledger: usize) -> usize; // 擴展律（W6 以 generate() 實測校準常數）
```

規則：`id`/動態 `path` 唯一；`nav_group` 封閉集合見 §3.1；每頁 ≥1 workflow；`Method::Post` 僅 2 筆；導覽順序＝`PAGES` 宣告順序。

### §3.1 NAV_GROUPS（導覽結構 SSOT；順序即渲染順序）

| # | group id | 顯示名 | 渲染 | 需求閘 |
|---|---|---|---|---|
| 1 | `overview` | 總覽 | Flat（單連結） | All |
| 2 | `ledger` | 帳務作業 | Menu（details 下拉） | Read |
| 3 | `capture` | 匯入與草稿 | Menu | Write |
| 4 | `bankpay` | 銀行支付 | Menu | Read |
| 5 | `reports` | 報表分析 | Menu | Read |
| 6 | `query` | 檢索 | Flat | All |
| 7 | `governance` | 治理稽核 | Menu | Manage |
| 8 | `admin` | 管理 | Menu | Manage |
| 9 | `apihub` | API 中樞 | Menu | Manage |
| 10 | `system` | 系統 | Flat（status/help/profile 三連結） | All |
| 11 | `knowledge` | 知識治理 | Flat（單連結 → /console/index.html） | All |

規則：Menu 群組＝`<details class="nav-menu">`（沿現行零 JS 下拉與 CSS）；群組內項目各自 gate 過濾（例如 bankpay 群組對 Read 者顯示）；**群組內所有項目皆被過濾掉時整組不渲染**（連空 dropdown 都不出現）。

### §3.2 PAGES 登記種子表（動態 **42** 筆＝Public 1＋GET 39（38 實測＋W3 前瞻之 accounts.detail；含 bankpay.batch-detail）＋POST 2；gate 標注：✓＝已量測、?＝W0 對 handler 實測校準）

> 修正紀錄（實作期 W1）：原表漏列 bankpay.batch-detail 且未計 accounts.detail 前瞻登記——manifest 以程式碼為準，動態總數 42。

**Public（1）**

| id | path | method | group | gate | tier |
|---|---|---|---|---|---|
| auth.login | /login | GET+POST | — | Public ✓ | T1 |

**動態 GET（38）**

| id | path | group | gate（證據） | tier |
|---|---|---|---|---|
| home.index | / | overview | All ✓（page_matrix：admin/store 200） | T1 |
| tx.list | /transactions | ledger | Read ✓（store 200） | T1 |
| tx.detail | /transactions/:id | —（詳情） | Read ? | T1 |
| tx.wizard | /new | ledger | Write ? | T1 |
| accounts.list | /accounts | ledger | Read ✓（store 建立被擋＝Manage 僅 POST） | T1 |
| accounts.detail | /accounts/:id | —（詳情；W3 新增） | Read ?（隨 W3 落閘） | T1 |
| ai.decisions | /ai-decisions | ledger | Read ? | T1 |
| import.upload | /import | capture | Write ? | T1 |
| import.history | /import/history | capture | Read ? | T1 |
| drafts.list | /drafts | capture | Write ? | T1 |
| bankpay.main | /bankpay | bankpay | Read ? | T1 |
| bankpay.batches | /bankpay/batches | bankpay | Read ? | T1 |
| bankpay.exceptions | /bankpay/exceptions | bankpay | Read ? | T1 |
| reports.monthly | /reports/monthly | reports | Read ? | T1 |
| reports.trial-balance | /reports/trial-balance | reports | Read ? | T1 |
| reports.ledger | /reports/ledger | reports | Read ? | T1 |
| reports.pivot | /reports/pivot | reports | Read ? | T1 |
| reports.parallel | /reports/parallel | reports | Read ? | T1 |
| search.page | /search | query | All ✓（store 200） | T1 |
| audit.page | /audit | governance | **Manage ✓**（store 403；page_matrix） | T2 |
| status.page | /status | system | All ? | T1 |
| help.page | /help | system | All ? | T1 |
| profile.page | /profile | system | All ? | T1 |
| states.specimen | /states | —（§7.5：不入導覽） | All ✓ | T3 |
| tauri.preview | /tauri-preview | —（§7.4：不入導覽） | All ?（session 保護） | T3 |
| admin.users | /admin/users | admin | **Manage ✓**（store 403） | T2 |
| admin.roles | /admin/roles | admin | Manage ✓（handler 內 ROLE_MANAGE） | T2 |
| admin.sessions | /admin/sessions | admin | Manage ✓ | T2 |
| admin.system | /admin/system | admin | Manage ✓ | T2 |
| admin.parse-coverage | /admin/parse_coverage | admin | Manage ✓ | T2 |
| apihub.catalog | /admin/api/catalog | apihub | **Manage ✓**（store 403） | T2 |
| apihub.test | /admin/api/test | apihub | Manage ✓（模組註解：處理器內再驗） | T2 |
| apihub.logs | /admin/api/logs | apihub | Manage ✓ | T2 |
| apihub.config | /admin/api/config | apihub | Manage ✓ | T2 |
| apihub.records | /admin/api/records | apihub | Manage ✓ | T2 |
| apihub.codes | /admin/api/codes | apihub | Manage ✓ | T2 |
| apihub.release | /admin/api/release | apihub | Manage ✓ | T2 |
| apihub.net | /admin/api/net | apihub | Manage ✓ | T2 |

**POST 呈現頁（2）**

| id | path | method | group | gate | tier |
|---|---|---|---|---|---|
| review.bulk-result | /reviews/bulk | POST | —（表單輸出） | Write ✓（測試 bulk_review_requires_ledger_write） | T1 |
| ai.suggestion | /drafts/ai_suggest | POST | —（表單輸出） | Write ✓ | T1 |

詳情頁（tx.detail、accounts.detail、bankpay.batch-detail）不入導覽但**必須**出現在 PAGES（矩陣測試覆蓋＋路由對帳）；其可達性由父列表頁連結斷言（W5）。`?` 項由 W0 以 page_matrix 擴充實測後回寫 manifest（**禁止虛構閘**——以 handler 程式碼為基線）。

**POST 頁口徑（grill-C 自省消歧）**：`/import/preview`、`/new/step2` 亦直接渲染 HTML，但屬**表單流程中間態**（前者依賴 multipart 檔案、後者依賴 step1 表單狀態），矩陣無法參數化其前置條件——故**不入 PAGES**（Method::Post 維持 2 筆），改入路由對帳**排除清單並附具名理由**，其行為由既有專項測試覆蓋（`import_preview_reports_and_persists_no_ledger_effects`、`wizard_step2_rejects_same_account_and_preserves_values`）。此為「矩陣測頁面不變量，專項測流程」的分工原則。

### §3.3 KEM 靜態清單（renderer=Static）

固定頁（**24**，含 console.css 計數口徑；HTML 檔 23）：`console.index`（/console/index.html）、`claims.index`、`queue`、`vectors`、`maintenance`（5）、roles×5（developer/reviewer/operator/pm/auditor）、workflows×13（capture/ingest/ask/sync/supersede/embed/eval/vet2/review/lint/doctor/status/finalize）＋`console.css`（generate() 計 1 頁）＋族 2（`kb.claim`＝claims 數、`kb.ledger-chunk`＝⌈ledger/200⌉）。

**擴展律（已裁決）**：`static_console_pages(claims, ledger) = 24 + claims + max(ledger,1).div_ceil(200)`——與 `generate()` 回傳值逐次相等（斷言測試）；文檔「≈21」更正為此式（console.rs L8、index.html L425 同步）。KEM 工作流 id 對映 13 命令；workflows 屬 `knowledge` 群。

---

## §4 工作項（依序；單一寫者；每項含 DoD）

**W0 基線複測與在途對帳（必做第一步）**
- 四關現況複測（`$env:CARGO_TARGET_DIR="$env:LOCALAPPDATA\Temp\dev-target"`）；`git status` 盤點在途（addendum #19 已交付 P0/P1——不得重做）。
- 精確盤點：動態 GET/POST 呈現頁全表（含 /import/preview、/new/step2 渲染行為）；derive↔模板對帳（重算 B3/B15）。
- **bootstrap admin 閘校準（grill-D 自省項）**：驗證生產 bootstrap 流程是否授予 admin `LEDGER_READ`（migrations/seed 路徑）；若無，middleware NavFlags 會使 admin 導覽缺帳務群組——修 bootstrap 或 NavFlags 對 admin 特例，以實測裁決。
- 確認 `page_matrix.rs`、design-system.css 變更已入版控或屬在途——**開工前與平行流確認工作面靜止**（api.rs/pages.rs/templates 剛被重排）。
- DoD：量測命令＋exit code 入 `.kilo/sessions/<id>/tests/`；§0 數字更新；bootstrap 閘結論記錄。

**W1 `crates/console_manifest` 新 crate**
- 建 crate（workspace members 增列；platform、kem 加 path 依賴）；實作 §3 型別＋`NAV_GROUPS`（§3.1）＋`PAGES`（§3.2 種子表，`?` 項以 W0 實測回寫）＋`FAMILIES`＋擴展律（§3.3 已裁決公式）。
- 單元測試：id/path 唯一、group 封閉、Public 僅 login、Post 僅 2 筆、擴展律單調、每頁 workflows 非空。
- DoD：`cargo test -p console_manifest` 綠。

**W2 導覽統一（消滅三處導覽定義與死連結）**
- 新 `platform/src/nav.rs`：`NavModel`＝由 `PAGES`＋`NAV_GROUPS`（過濾 renderer=Dynamic 且 nav_group 有值）依 `NavFlags` 過濾生成；順序＝§3.1 宣告序；群組全空則整組不渲染。
- **計算位置（grill-B 裁定：middleware 注入）**：`require_session` middleware 內計算 `NavFlags`（+1 次 SQLite 權限查詢，本機 <1ms）並建構 `NavModel`（純記憶體過濾）→ 注入 Extension；全部 handler（GET＋POST 呈現頁）一行複製 `nav: nav_model` 欄位——單點實作、GET/POST 統一、API 路徑成本由 W5 效能斷言驗證（P99 50ms 預算內）。**不做**「權限快照取代 RBAC 檢查」（PRE-R1b 範疇，雙向門另案）。
- 重寫 `partials/studio_nav.html` 為 `{% macro site_nav(nav) %}`（渲染 aria-current、details 分組、nav-role-badge 沿用現行類別）。
- index.html：以 macro 取代內聯導覽（保留 NavFlags 語義；**`/audit` 移入 governance 群組＝根治死連結**）；bankpay.html：同樣以 macro 取代其內聯導覽。
- 分批推廣（每批獨立可驗收、可提交）：
  - **批次 A（本項內）**：index.html、bankpay.html。
  - **批次 B（T2/T3 頁，14 模板）**：audit_page、admin_users、admin_roles、admin_sessions、admin_system、parse_coverage、api_catalog、api_test、api_logs、api_config、api_records、api_codes、api_release、api_net。
  - **批次 C（T1 頁，23 模板）**：transactions、transaction_detail、accounts、account_detail（W3 後）、ai_decisions、import、import_history、drafts、wizard、bulk_review、ai_suggestion、search、report_monthly、report_pivot、report_parallel、trial_balance、account_ledger、bankpay_batches、bankpay_batch_detail、bankpay_exceptions、status_page、profile、help。login、states、tauri-preview 不套用（2＋14＋23＝39＝42−3 不適用，對帳閉合）。
  - 每頁 handler 增 `nav: NavModel` 欄位（機械式，~40 處；Askama 結構欄位＋一行呼叫）。
- 路由對帳測試：掃 api.rs `.route("…")` ↔ `PAGES` 動態 path 雙向相等（沿 guards 源碼掃描慣例）；**非頁面路由明示排除清單**（/logout、/assets/*、/api/*、CSV/JSON 端點、/console/*、/tauri-preview 等），排除亦須窮舉防漏。
- DoD：對帳測試綠；W5 之導覽可達斷言綠；死連結零（無權限入口不渲染，以測試證明）。

**W3 缺口關閉**
- F4：`GET /accounts/:id` handler（mirror `account_ledger_page`：session＋LEDGER_READ 閘、fail-closed 參數、未知 id→404 樣式頁）＋新 `AccountDetailTemplate` 綁定 `account_detail.html`（欄位契約見 B8）＋**`/accounts` 列加詳情連結（已實測現無：accounts.html 列內僅 toggle 表單，無 detail href）**。
- F5：刪除死碼 `apihub::router()`、`workbench::router()`、`auth::login_router()`（零呼叫者、零行為變更；保留 api.rs 為唯一註冊源）。
- `audit.html` 孤兒：歸檔標 SUPERSEDED（不刪；`audit_page.html` 為現行版）。
- PIN 紀律：`characterization.rs`（GETS/POSTS/CSRF 陣列）與 page_matrix 新增斷言須附理由註解。
- DoD：四關綠；`/accounts/:id` 矩陣斷言（admin/finance 200、store 視閘、訪客 303）。

**W4 八態共享 partial**
- `templates/partials/state_anchors.html`：empty/denied/error 錨點 macro（`data-state-*`）；全模板 include；`states.html` 維持八態 SSOT。
- DoD：模板存在；W5 掛點斷言綠。

**W5 矩陣擴充（就地擴充 page_matrix.rs，不另建檔）**
- ＋finance 身份（LEDGER_WRITE/READ：Read 頁 200、Manage 頁 403）；＋每回應安全標頭斷言（CSP/no-store/nosniff/DENY/no-referrer，沿 api.rs 既有測試樣式）；＋每頁 empty/denied/error 掛點；＋導覽可達（GET `/` 掃 href 覆蓋 manifest 宣告之 nav 頁）；＋POST 呈現頁 2 筆（CSRF＋cookie）。
- **閘校準回寫**：對 §3.2 標 `?` 之頁，以 store/finance/admin 三身份實測期望碼，回寫 manifest `gate` 欄（以 handler 程式碼為基線；禁止虛構閘；差異須具名記錄）。
- 效能：沿「預熱＋p95」法，`/transactions`、`/audit` w/ 2k 交易 <500ms。
- DoD：矩陣全綠；manifest `?` 歸零（全數實測或具名 UNKNOWN）；既有 145 測試 0 失敗不回退；證據入 session tests/。

**W6 KEM 靜態渲染器接入 manifest＋受保護伺服**
- `crates/kem/src/console.rs`：導覽/角色/工作流/標題由 manifest 生成（href 生成沿現行 `href_fix` 相對路徑模式）；**擴展律落地（§3.3 已裁決）**：`static_console_pages(claims, ledger) = 24 + claims + max(ledger,1).div_ceil(200)`，`generate()` 回傳值斷言相等；同步更正兩處過時文案（console.rs L8 文檔註解、index.html L425「擴展律：頁數 ≈ 21…」）。
- **生成頁內聯樣式歸零**（B18；CSP 相容）：4 模式改類別——`h2.mt-4`（margin-top:16px→space-4）、`p.mono.mt-3`（12px→space-3）、`div.kpi-num`（font-size/weight）、`pre.log-tail`（white-space:pre-wrap）入 CONSOLE_CSS；生成檔 `grep 'style="'`＝0（納入護欄斷言）。
- **platform 伺服路由**（新增 `platform/src/console_files.rs` 或置 api.rs）：
  - `GET /console/*path` 掛載 `knowledge/console/`；掛入 protected HTML 群組（訪客 303）；僅 GET。
  - **安裝版現實（grill-A 裁定：NSIS 捆綁＋多候選解析）**：console 根以候選序列解析——`exe_dir/knowledge/console`（NSIS resources 捆綁；tauri.conf `resources` 沿 P1-1 sidecar 先例加入）→ `CWD/knowledge/console`（開發版）；兩者皆無 → `/console/*` 一律 404 樣式頁（誠實降級，絕不 panic）。捆綁清單與 smoke 驗證（安裝目錄含 knowledge/console 且 `/console/index.html` 200）入 addendum。
  - 防穿越：拒絕 `..` 區段、反斜線、NUL、絕對路徑、點檔；`std::fs::canonicalize` 後驗證前綴仍在 console 根目錄內（fail-closed）。
  - 副檔名白名單：`.html`／`.css`／`.json`（claims 機器 SSOT 頁有 .json 連結）；其餘 404。Content-Type：text/html; charset=utf-8／text/css; charset=utf-8／application/json。
  - `/console` 與 `/console/` → `index.html`；未知路徑→404 樣式頁（data-state-error 語彙）。
  - 安全標頭沿既有 middleware 自動套用（no-store 可接受；如需快取另行拍板）。
- 跨台連結：platform 導覽「知識治理」群組 → `/console/index.html`；KEM 頁頭導覽末端加「營運工作台」→ `/`（同源相對根；獨立以 file:// 開啟時此連結不適用，頁尾誠實標注）。
- 護欄：零 JS 掃描擴至 `knowledge/console`（生成物）；編碼掃描沿 SCAN_DIRS=7 已含 kem/src。
- 測試落點：伺服行為（200/303/404/穿越/白名單/Content-Type）入 `page_matrix.rs`（或併入 guards 風格之新測試，二擇一記錄）。
- DoD：kem 測試綠；生成頁數＝擴展律（45 現值斷言）；穿越/白名單/303/404 測試綠；護欄含 knowledge/console；`/console/index.html` 於 admin 200。

**W7 設計單一身份（token 收斂——grill-E 裁定：全收斂，名稱＋值）**
- `CONSOLE_CSS` 重寫：採 canonical token 名稱**與值**；**逐項映射表**（舊 token→canonical token→新值）隨 addendum 提交，已知色差：`--accent #0b5cab→--color-primary #2563eb`、`--ok #177245→--color-success #16a34a`、`--warn #9a5b00→--color-warning #d97706`、`--err #b3261e→--color-error #dc2626`、`--ink #1b2733→--color-text-primary #0f172a`、`--muted #5c6b7a→--color-text-secondary #475569`、`--bg #f4f6f9→--color-surface-sunken #f8fafc`、`--border #d9dfe7→--color-border #e2e8f0`、`--chip #eef2f7→--color-neutral-100 #f1f5f9`、`--radius→--radius-lg`、`--s1..4→--space-1/2/4/6`、字體→`--font-family-sans/mono`。**色差屬使用者裁定之收斂結果，非視覺回歸**（addendum 記錄此聲明）。
- 護欄測試：console.css 僅引用 canonical token 名稱（禁自造色彩字面值）；維持既有 `web/assets`↔`docs/` 雙副本漂移測試綠。
- 視覺驗證：對比度維持 AA（canonical 調色已為 AA 設計）；dark mode 對齊 design-system dark 區塊。
- DoD：token 測試綠；KEM 頁零 JS 不變；映射表＋色差聲明入 addendum。

**W8 端到端與擴展律演示**
- 以現行 claims/ledger 值輸出 `static_console_pages` 對照表與成長曲線（不虛造頁）；產出「角色×工作流×情境」覆蓋摘要（矩陣＋八態＋導覽可達＝可驗證子集；19,683 依 §0 紀律表述）。
- DoD：演示輸出綁證據。

**W9 版控與文件**
- 分批提交（feat: manifest crate → nav 統一 → 缺口 → tests → kem/token → docs）；`page_matrix.rs` 與其他未追蹤交付一併入版控；舊計畫檔頭標「已整合」；PLAN_REGISTER addendum #20＋CHANGELOG＋PROJECT_STATUS/NEXT_ACTIONS。
- DoD：工作樹收斂；提交訊息具語義。

**W10 發行鏈現代化（grill-A/發行鏈裁定；2026 實證基準）**
- **自動更新**：`tauri-plugin-updater` 接入 web/src-tauri——`cargo tauri signer generate` 產生 minisign 金鑰對（私鑰 gitignore＋離線備份；公鑰入 `plugins.updater.pubkey`）；`bundle.createUpdaterArtifacts: true`；endpoint＝`latest.json` 靜態 feed（內部 HTTPS 資源；開發期 `dangerousInsecureTransportProtocol` 僅限測試構建、不得提交）；`plugins.updater.windows.installMode: "passive"`。更新驗證為硬門檻：簽章不符即拒裝（負向用例：改一字符必敗，沿 2026 業界測試法）。
- **簽署配置位**：tauri.conf 預留 `bundle.windows.signCommand`（Azure Trusted Signing / trusted-signing-cli 形態）註解塊；**未購證前不啟用**——所有產物於 CHANGELOG/release notes 明確標注 `unsigned (test-only)`；不得宣稱生產可分發。購證與否為使用者預算決策（§7 開放點）。
- **NSIS resources 捆綁**（與 W6 接軌）：`bundle.resources` 增 `knowledge/console` → 安裝目錄；W6 之多候選解析據此命中。
- **更新鏈路本地驗證**（沿 2026 業界流程）：新版構建→feed→舊版 shell 觸發 check→下載→驗章→passive 安裝→重啟；teardown 零殘留（沿 P1-4 smoke 慣例，擴充 `scripts/tauri-shell-smoke.ps1`）。
- DoD：更新鏈路端到端 green（含簽章負向用例）；`tauri build` 產出 `-setup.exe`＋`.sig`＋`latest.json` 三件；捆綁後安裝版 `/console/index.html` 200；四關不回退；未簽署聲明落文件。

---

## §5 驗收命令（實作代理逐項執行並留痕）

```powershell
$env:CARGO_TARGET_DIR = "$env:LOCALAPPDATA\Temp\dev-target"   # 隔離，避並行檔鎖
cargo fmt -- --check                                          # EXIT=0
cargo clippy --all-targets -- -D warnings                     # EXIT=0（警告 0）
cargo test --locked                                           # 0 failed / 0 ignored（基線 145，擴充後遞增）
cargo audit                                                   # EXIT=0
cargo test -p console_manifest                                # manifest 契約
cargo test -p platform --test page_matrix                     # 角色矩陣（擴充後）
cargo test -p platform --test guards                          # 零 JS／編碼護欄
cargo test -p kem                                             # KEM 生成器＋擴展律
```

寫檔一律 UTF-8 無 BOM；寫後讀回驗證；禁 `>` 重定向。證據入 `.kilo/sessions/<id>/tests/`。

---

## §6 風險與界線

1. **平行流收斂**：addendum #19 剛交付（api.rs 重排 14k 行、templates 多檔修改、page_matrix.rs 未入版控）——W0 先確認工作面靜止或接管所有權；`page_matrix.rs`/design-system 變更須隨本計畫入版控。
2. **Askama 0.16 陷阱**：屬性區禁 `{% %}`、Option 需 if-let、改模板後 `cargo clean -p platform`。
3. **NavFlags 為既有投資**：導覽統一必須消費而非取代 NavFlags（避免推翻平行流成果；C-01）。
4. **kem 並行線**：workspace 級測試若受阻，泳道級執行並誠實記錄（addendum #13 前例）。
5. **勿重做已閉環項**：F6 稽核閘、內聯樣式歸零、Tauri NSIS/smoke、guards 7 目錄（B4/B5/B11/B14）——不得回退。
6. **19,683 表述**：僅作組合空間級數與擴展律目標；禁宣稱實體頁已交付。
7. **C-01 保留原則**：孤兒檔歸檔不刪；`studio_nav.html` 換代保留檔名。
8. **KEM 擴展律常數**：文檔 21 與實測 44 不符——W6 以 generate() 實測校準，禁沿用未核對常數。
9. **未簽署產物紀律**（B19/B20）：unsigned 僅供內部測試；對外分發聲明（SMARTSCREEN 警告、信譽累積）禁虛構；W10 完成前 CHANGELOG 每版標注 `unsigned (test-only)`。
10. **minisign 私鑰保管**：gitignore＋離線備份；私鑰遺失＝無法向已裝機用戶推送更新（公鑰燒錄於已分發安裝包）——2026 業界共識，入 W10 DoD。

## §7 開放點

1. ~~KEM 控制台伺服模式~~ **已裁定（2026-11-10）**：由 platform 伺服——設計併入 W6。
2. F5 解法：建議刪除死碼（零行為變更）；若要模組化可改 merge，二擇一記錄於 addendum。
3. `account_detail` vs `account_ledger` 分工：建議 detail＝快照／ledger＝流水。
4. `/tauri-preview`：建議保留標內部。
5. `/states` 是否入導覽：建議否（技術樣本頁，經 /help 可達）。
6. `/console/*` 靜態頁是否需快取標頭（脫離 no-store）：建議維持全域 no-store（一致性優先），有實測需求再拍板。
7. **程式碼簽署採購**（W10 預留接口）：Azure Trusted Signing（微軟代管，年費制）vs OV/EV 憑證 vs 維持 unsigned＝test-only。屬預算決策，購證前所有 release 誠實標注 unsigned。
8. 更新 feed 托管位置：內部 HTTPS（哈瓦士/內網伺服）vs GitHub Releases——建議內部（本工作區產品不假設公網）。
