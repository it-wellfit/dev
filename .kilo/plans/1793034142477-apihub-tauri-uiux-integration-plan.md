# 雙作品 UI/UX 比較、互補演進與最佳整合計畫（Tauri 玉山平台 ↔ Web 工作台）

- 版本：v1.4｜日期：2026-11-10｜狀態：定稿＋P0/P1 執行切片（§9）；D1/D3/D5/D7/D8 全數拍板
- 對應計畫 SSOT：ADR-0012（E.Sun 整合邊界）、`1793782615513-esun-bankpay-ui-optimization.md`、`.kilo/plans/1793787631643-three-studio-slice-execution-plan.md`
- 營運方式：本檔為主計畫；實作後依規範更新 PLAN_REGISTER addendum 與 §22 追溯矩陣。

## 0. 先行事實校正（對來文「Electron 更正」之直陳）

**「玉山 API 平台」不是 Electron 的誤拼或同義詞。**

- 玉山＝ E.Sun Bank（玉山銀行）。本工作區的 Tauri 應用（`web/src-tauri/`、產物 `flaps-erp-web.exe`、窗口標題「Flaps ERP - 企業資源規劃系統」）就是 E.Sun V2.8 API 管理平台：`bankpay_get_api_status()` 返回 **8 筆 API 紀錄**（endpoint／purpose／environment／result=Mock Pass／response_time_ms），另含錯誤碼、測試紀錄、Release checklist、連線檢查、資安檢查。
- **Electron 在本 codebase 零引用**；Rust(edition 2024) 是憲政基線，引入 Node/Electron 與憲政衝突。來文「已更正為 Tauri vs Electron 比較」在事實與工作範圍上都不成立。
- 本計畫仍然回應來文的分析綱要（第 1–4 節：架構渲染／設計系統／DX 生態／持續迭代；第 5 節：工作室與跨平台整合策略），但其對象是**本工作區已有的兩件作品**（見 §1），框架選型只作為「跨平台決策」一章。

## 1. 兩件作品盤點（證據級）

### 作品 A：Tauri 桌面版玉山 API 平台
- **位置**：`web/src-tauri/`（`main.rs` 182 行、`state.rs` 249 行、`commands.rs`、`security.rs`）；產物 `web/src-tauri/target/debug/flaps-erp-web.exe`（12,472,832 B，2026-11-05 建置）。獨立 workspace（`Cargo.toml` 有 `[workspace]` 標記）。
- **8 API 紀錄**：`/payment/createTwdPaymentInstruction`、`/payment/retrieveTwdPaymentInstructionResult`、`/account/retrieveStatements`、`/payment/validateFxPaymentInstruction`、`/payment/createFxPaymentInstruction`、`/payment/retrieveFxPaymentInstructionResult`、`/account/retrieveAccountBalance`、`/payment/retrieveTwdPaymentInstructionPrintResult`。
- **治理面板獨有**：測試紀錄（id/api_name/test_type/result/timestamp/duration_ms）、錯誤碼（910000）、Release checklist（RC-01…）、連線檢查（TLS 1.2、雙向 SSL、8443/9443、security_component、device_binding）、資安檢查（cert pinning、WDAC、masking、CSRF/XSS）。
- **前端（經查證）**：`web/index.html`（313 行）**零 `invoke` 調用**；`web/src/*.rs` 僅資料型別；全倉唯一 `.html` 即該基線。**因此 Tauri 前端現況只是載入該靜態頁，無 commands 接線、無 Vite/build。**
- **硬傷**：
  1. 全資料為硬編碼常量 JSON（`total_transactions:150` 等虛報）——依 P7 不得作為產品証據，必須以真實探測取代或明示 mock。
  2. `f64` 金額/餘額/總量（`total_volume: f64`、`balance: f64`）——違反 INV-01（金額禁浮點），與 `platform` 側 i64 minor 基準不一致。
  3. `main.rs:137` 單行壓縮損壞（錯誤碼/狀態函式擠成一行）——可讀性與審計性差。
  4. `tauri.conf.json` CSP 含 `'unsafe-inline'`（style+script）——與零-JS 憲政衝突；`bundle.active: false`（無法打包分發）。

### 作品 B：Web 工作台（`platform/`）
- **強項**：零 JS、登入+RBAC+CSRF、SQLite 真實資料、`erp_config` 持久化（SSOT）、fail-closed 稽核、`/api/status` 即時探測（不快取，模擬模式 `Warning` 誠實標示）、12 端點 registry（`apihub.rs`）、參數化測試台、124 測試綠、ADR-0012 已驗證（401/403/冪等/白名單/稽核寫入）。
- **缺口**：無桌面殼；治理面板（測試紀錄/Release checklist/錯誤碼/連線&資安檢查）不存在；E.Sun V2.8 8 端點語意未全部映射（現僅 export/import/query/config/health/transition 6 個）。

### 權威契約基準（不可動搖）
- V2.8 協議層（TLS/mTLS、CP950 媒體編碼、ReleaseGate、安全元件 API 信封 `rqHeader`/`rqData`）：`Wellfit/crates/wellfit_bankpay`（8 端點、32 測試）。
- 文件：`Wellfit/docs/bankpay/04-interface-specification.md`；邊界裁決：ADR-0012。

## 2. UI/UX 比較矩陣（鑑別式盤點，非估觀印象）

| 維度 | 作品 A（Tauri 桌面／玉山平台） | 作品 B（Web 工作台） | 互相借鑑方向 |
|------|-------------------------------|------------------------|-----------------|
| 資訊架構 | 桌面窗口一覽（1400×900）＋命令驅動；導航在命令頁 | 主導覽列＋狀態列 sticky＋三工作室入口；資訊密度高 | B 吸收 A 的工作室分區；A 採用 B 的主導覽語彙 |
| 狀態誠實性 | 加載時 status 恆為「Mock Pass」；全常量——**違 P7** | 即時 probe；mock 標 `Warning`＋文字說明；`overall: healthy` 不虛報 | A 必須停用虛報（以 probe 取代）；B 沿用 |
| 測試治理 | 測試紀錄表（有 response_time_ms／環境／結果）＋ Release checklist＋錯誤碼——**這是 B 完全缺席的治理面** | 無任何治理面板 | B 新增治理四頁（見 §4 D7／§5 T2） |
| 連線/資安可驗證性 | 連線檢查（TLS／雙向SSL／8443／9443）＋資安檢查（pinning/WDAC/mask/CSRF/XSS）——**B 缺席** | 無對等面板 | B 新增（以本地探測＋設定核實為準，不宣稱遠端連通性） |
| 表單/互動 | Tauri invoke（需 JS，已安裝殼）；現況前端未接線（載入靜態基線，無 invoke 調用） | 零 JS 表單＋PRG＋參數 GET 探測 | 桌面殼保留 invoke 路徑作為作者面工具；**使用者面維持零 JS**（雙通道誠實分工） |
| 可及性 | JS 渲染，鍵盤/焦點不透明 | 語義 HTML＋skip-link＋aria＋`:focus-visible`（WCAG 2.2 AA 方向） | A 遵循 B 語意與焦點規範 |
| 效能/足跡 | 常駐 WebView + OS 級 WebView2，Rust 二進位 12MB | axum server 啟動成本＋當次渲染 O(1) | 桌面殼載入 localhost server 時，WebView 資源成本可測 |
| 資料真實性 | 虛報數值（150/500000/45/12） | 全來自 DB/設定/探測；測試數據標本機 | A 停用虛報，全面改為 B 的探測值或明示 mock |
| 安全 | CSP `unsafe-inline`；無認證/CSRF/稽核 | `auth::require_session_api` 401、XFER、CSRF 同步器、稽核 fail-closed | A 對齊 B（CSP 收緊；命令需 session/RBAC） |
| 金額數值 | `f64`（INV-01 違規） | i64 minor＋`ScaledDecimal` | A 改 i64 minor（`parse_decimal_to_minor` 綁定） |

**結論性判斷**：作品 A 的「治理面板＋8 API 纪录表」是第二工作室（API 中樞）的現成內容原型；作品 B 的「真實探測＋持久化＋安全＋零 JS」是唯一可採取的產品基底。**不是二選一，是 A 的資訊架構長在 B 的真實基底上。**

## 3. Tauri vs Electron（框架選型一章——回應來文綱要）

| 維度 | Tauri（本專案採用） | Electron（本 codebase 未採用） |
|------|---------------------|-------------------------------|
| UI 渲染 | OS 級 WebView（Win→WebView2、macOS→WKWebView、Linux→WebKitGTK），無 Chromium 捆綁 | 捆綁整顆 Chromium，足跡通常 100MB+/一顆電池與記憶體成本高一個量級 |
| 記憶體/效能 | Rust 二進位 12MB 級；無 Node 進程常駐 | 雙進程（main+renders）；常駐記憶體通常 200MB+ |
| 設計語言 | 需自建 token 層（本專案 `design-system.css` 已有 SSOT） | 需自建；但 JS 生態設計系統多 |
| DX/生態 | Rust 生態；`tauri dev/build` 級且引腳少；除錯需 Rust 側 | Node 生態成熟、除錯工具链豐富；但 Node/JS 依賴樹巨大 |
| 持續迭代 | 跨平台（Windows/Linux/macOS）；行動版需 Tauri Mobile 另線 | 桌面主流成熟；行動版不覆蓋 |
| 與本專案憲政之契合度 | **Rust 2024 Edition workspace、零 JS、單一 binary 部署** —— 完全契合 | 與「零 JS＋Rust 基線」憲政衝突 |

**裁決**：本專案**不引入 Electron**。桌面面採用 Tauri（現有殼），行動面（若啟動）以 Tauri Mobile 或直接響應式 Web（本平台零 JS 已是響應式）評估。框架比較到此為止；之後的整合計畫以這兩件作品為主體。

## 3.5 非協商約束（使用者欽定 2026-11-09）

1. **零 JavaScript 優先**（憲章 §2.3）：使用者面 `<script>` 數 = 0、無內聯事件；HTMX／Datastar／WASM **不算**零 JS。
2. **禁用耗費效能的 JS 相關工具鏈**：不引入 Node／npm／Vite／Webpack／esbuild 等建置鏈；靜態資產以 `include_str!` **編譯期內嵌**（零執行期檔案 I/O）。Tauri `devUrl: http://localhost:1420`（Vite 殘留）**必須清除或改指平台伺服器**。
3. **提防 Electron**：不引入（§3 已裁決）；任何「UI 遷往 JS 生態」之提案預設拒絕，例外需 ADR。
4. **效能**：伺服端渲染、當次渲染 O(1)、SQL 綁定參數（非拼接）、分頁夾限、狀態探測 derive 非快取。
5. **安全**：RBAC＋CSRF 同步器＋稽核 fail-closed＋路徑白名單（CWE-22）＋CSP 收緊（去 `unsafe-inline`）。
6. **穩定／可靠**：fail-closed、冪等重放、UNKNOWN≠PASS（不得以常數冒充探測）；錯誤訊息含「問題＋可能原因＋處理方式」。
7. **易維護／易規模化／易演進**：registry SSOT、design token SSOT、八態樣板、§0.5 重構判定框架＋跡效函式（fitness functions）持續護欄；擴充以「加入 registry、複用元件、複用探測」為路徑（規模化不重造）。

## 3.6 角色 × 工作流 × 情境覆蓋矩陣（系統性覆蓋，非枚舉）

- **角色（4）**：未登入（訪客）、admin（`ROLE_MANAGE`）、finance（`LEDGER_WRITE`／`LEDGER_READ`）、store（唯讀）。
- **工作流（6）**：憑單作業（建立／覆核）、銀行支付（匯出／匯入／查詢／設定）、匯入解析（上傳／草稿／AI）、報表分析（月報／樞紐／平行座標）、帳務查詢（交易／科目／全文檢索）、治理稽核（使用者／覆蓋率／稽核／API 中樞）。
- **情境（9）**：資料八態（empty／zero／loading／partial／stale／denied／error／conflict）＋權限邊界（401／403）。組合空間上界 ~3^9＝**19,683+**（使用者語：不誇張；承認全枚舉不可行）。
- **覆蓋機制**（收斂組合、單切片可驗證）：
  1. **共用元件**：八態樣板、badge、table shell、panel、pager——新頁一律引用，不重造。
  2. **角色矩陣測試**：3 個登入身份 × 27 頁 smoke（200／303／403 期望斷言），未登入一律 303→/login（HTML）或 401（API）。
  3. **八態 DOM 斷言**：`states.html` 為樣板 SSOT；每頁至少渲染 Empty／Denied／Error 三態資料掛點（可由共享 partial 提供）。
  4. **跡效函式**（持續護欄）：zero-JS 掃描、編碼損傷掃描、design token 漂移測試、registry 對帳測試（目錄 vs 路由）。

## 4. 最佳整合架構（D1–D8 全數拍板 ✅）

### D1：殼載入平台伺服器（已拍板 ✅ 建議案 a）
- Tauri 應用改為「**原生殼**」：WebView 指向 `http://127.0.0.1:{port}` 的 platform 伺服器；Tauri 後端只負責啟停本機 platform 二進位（或同進程 axum）、檔案選擇、深連結。
- 好處：UI SSOT 唯一（作品 B）；桌面殼不重寫介面、不複製 API 語意。
- 命令顆粒度：保留作品 A 的 24 個 commands 為「作者面工具」（本機探測器／證書工具），**不作使用者流程入口**。
- 驗證：殼啟動→登入→儀表板渲染→/bankpay＝與瀏覽器一致（既有 E2E 覆蓋）。

### D2：API Registry 單一事實來源（消滅三處手寫清單）
- 擴充 `apihub::registry()`：12 → **19**（現有 12 + V2.8 玉山 8 端點語意映射：`createTwdPaymentInstruction`、`retrieveTwdPaymentInstructionResult`、`retrieveStatements`、`validateFxPaymentInstruction`、`createFxPaymentInstruction`、`retrieveFxPaymentInstructionResult`、`retrieveAccountBalance`、`retrieveTwdPaymentInstructionPrintResult`，auth/RBAC 按 ADR-0012 規則）。
- `bankpay_get_api_status()` 改由**同一 registry** 生成（Tauri 側純量資料），消除三處清單漂移（apihub registry、Tauri Vec、Wellfit 8 端點 wrapper）。

### D3：治理面板移植（已拍板 ✅ 併入 D7 二十七頁批量）
- 治理四頁（`records`／`codes`／`release`／`net`）與其餘三新頁一次交付，共用 probe 元件；詳見 D7 清單。
- `/admin/api/records`：API 測試紀錄表（源=`bankpay_batches`＋audit_log；欄：api_name／test_type／result／timestamp／duration_ms；response_time_ms 由 probe 實測，不存常數）。
- `/admin/api/release`：Release checklist（暫以 migration/erp_config 設定＋靜態規則表起步，標示「待 Wellfit ReleaseGate 串接」）。
- `/admin/api/codes`：錯誤碼（源=Wellfit error codes 表；起步靜態表，標示來源）。
- `/admin/api/net`：連線與資安檢查（TLS、8443/9443 本地探測、cert pinning、mock、遮罩、CSRF/XSS——回報本地探測結果，不宣稱遠端保證）。

### D4：金額 i64 全線貫徹
- 作品 A 所有 `f64` 金額/餘額/總量 → i64 minor＋`parse_decimal_to_minor`；跨平台的 JSON 顯示用已驗證的 `format_minor`。

### D5：安全與分發基線收斂（已拍板 ✅ 本輪啟用 NSIS）
- Tauri CSP：移除 `'unsafe-inline'`（script）並以 hash/傳入式配置取代；`style-src 'unsafe-inline'` 僅當設計系統 inline 存在時保留明示。
- Tauri `bundle.active: true`＋`targets: ["nsis"]`（Windows 打包可分發，v0.2 可分發版）；平台三端打包另議（T7 行動面不承諾）。

### D6：可及性與設計系統同源
- 作品 A 前端（如 invoke 桌面面）遵循作品 B 語意/Focus/Reduced-motion 規範；`design-system.css` 為兩面唯一 token SSOT。

### D7：二十七頁全工作流批量交付（已拍板 ✅ 一次交付）

**盤點基線（證據）**：`platform/templates/*.html` 現有 **20 頁**；`grep` 確認 `/search`、`/accounts/:id`、`/admin/audit` **無 UI**。目標＝20 現有（收斂合格）＋**7 新頁**＝**27 頁**。

| 工作室 | # | 路由 | 模板 | 新/舊 |
|--------|---|------|------|-------|
| 一（部門業務） | 1 | `/login` | login | 舊 |
| | 2 | `/` | index（總覽） | 舊 |
| | 3 | `/transactions` | transactions | 舊 |
| | 4 | `/transactions/:code` | transaction_detail | 舊 |
| | 5 | `/accounts/:id` | account_detail | **新**（科目詳情＋餘額） |
| | 6 | `/new`（＋step2） | wizard | 舊 |
| | 7 | `/import` | import | 舊 |
| | 8 | `/drafts` | drafts | 舊 |
| | 9 | `/drafts/ai` | ai_suggestion | 舊 |
| | 10 | `/reviews/bulk` | bulk_review | 舊 |
| | 11 | `/search` | search | **新**（全文檢索＋空態） |
| | 12 | `/reports/monthly` | report_monthly | 舊 |
| | 13 | `/reports/pivot` | report_pivot | 舊 |
| | 14 | `/reports/parallel` | report_parallel | 舊 |
| 二（API 中樞） | 15 | `/bankpay` | bankpay | 舊 |
| | 16 | `/admin/api/catalog` | api_catalog | 舊 |
| | 17 | `/admin/api/test` | api_test | 舊 |
| | 18 | `/admin/api/logs` | api_logs | 舊 |
| | 19 | `/admin/api/config` | api_config | 舊 |
| | 20 | `/admin/api/records` | api_records | **新**（測試紀錄） |
| | 21 | `/admin/api/codes` | api_codes | **新**（錯誤碼） |
| | 22 | `/admin/api/release` | api_release | **新**（Release checklist） |
| | 23 | `/admin/api/net` | api_net | **新**（連線/資安檢查） |
| 三（治理稽核） | 24 | `/admin/users` | admin_users | 舊 |
| | 25 | `/admin/parse_coverage` | parse_coverage | 舊 |
| | 26 | `/states` | states | 舊 |
| | 27 | `/admin/audit` | audit | **新**（稽核軌跡完整頁：分頁＋篩選＋匯出） |

**批量原則**：新頁一律「registry／探測／元件複用」；導覽列 27 頁可達（HTML 群組 `require_session`＋頁內 RBAC 區分）；一次交付、四關一次過。

### D8：桌面殼無 JS 建置鏈（已拍板 ✅）
- `tauri.conf.json`：移除 `devUrl: http://localhost:1420`（Vite 殘留）或改指 `http://127.0.0.1:8090`；`beforeDevCommand` 維持空白（**無 npm／Vite**）。
- 前端資產＝平台伺服器輸出（Askama HTML＋內嵌 CSS），Tauri 殼零 JS 建置鏈；24 commands 為 Rust 側作者面工具（無 JS 依賴）。

## 5. 工作項（依賴順序、逐項可驗收）

| # | 工作 | 驗收（指令級） |
|---|------|----------------|
| T1 | Registry 擴充 12→19（V2.8 8 語意映射）＋Tauri `bankpay_get_api_status` 同源 | `/admin/api/catalog` 19 項；`cargo test -p platform` 斷言含 8 玉山 endpoint path（新增一測） |
| T2 | 治理四頁一次交付（records／codes／release／net） | 4 頁零 JS 200；records 以 BATCH-DEMO-001 匯出後含該筆；net 本地探測且不宣稱遠端連通 |
| T3 | 工作流補口三頁（`/accounts/:id`、`/search`、`/admin/audit`） | 3 頁 200；search 有 q 參數與空態；audit 分頁＋篩選＋匯出連結；accounts 餘額 i64 minor |
| T4 | 27 頁矩陣合格化（八態掛點＋角色矩陣＋zero-JS 掃描） | 新測試：訪客／admin／finance／store × 27 頁 smoke（303／200／403 期望）；27 頁 `<script>`＝0 |
| T5 | Tauri 殼載入平台（D1＋D8） | 殼 smoke：登入→工作室→catalog；grep 無 `1420`；`bundle.active=true`＋NSIS 產物 |
| T6 | 作品 A 硬傷修復（虛報停用；i64；L137 拆擠；CSP；bundle） | grep 無 `500000`／裸 `f64` 金額；CSP 無 `script 'unsafe-inline'` |
| T7 | 行動面評估（響應式現況 vs Tauri Mobile）＋跨平台矩陣備忘 | 決策記 ADR-0013（本輪 NSIS 為 Windows 先行） |
| T8 | §22 追溯矩陣＋PLAN_REGISTER addendum＋本計畫同提交 | 每切片單提交（既有慣例） |

## 6. 驗證計畫

- 四關：`cargo fmt -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test --locked`（0 failed/0 ignored）、`cargo audit`。
- **27 頁矩陣**：角色矩陣測試（訪客／admin／finance／store × 27 頁）＋ zero-JS 掃描（27 模板 `<script>`＝0）＋編碼防護掃描（既有 R5 guards 擴至全 templates）。
- **八態掛點**：每頁至少 Empty／Denied／Error 三態資料掛點（共享 partial）；`states.html` 樣板 SSOT 引用檢查。
- E2E：`cargo test -p platform --test bankpay_persistence` 擴充（19 項 registry 斷言、records 頁筆數、audit 頁分頁）。
- 桌面殼 smoke：手動操作證據（殼啟動→登入→工作室→catalog 一致）；`grep` 確認 conf 無 `1420`。
- 警示：本工作區無 CI（客觀狀況）；四關須在本機證據 bind session 記錄。

## 7. 風險與已知限制

1. **Tauri 殼載入 localhost**：需處理 port 競用、子進程生命週期、深連結——殼實作若延期，D2–D4 不受影響（Web 面獨立交付）。
2. **V2.8 語意映射**：8玉山端點 mapping 到平台 6 端點（export/import/query/config/health/transition）語意細節繁多——以 Wellfit `04-interface-specification.md` 為準，未驗證來源標記 UNKNOWN，不推測。
3. **Release checklist 來源**：靜態起步需與 Wellfit ReleaseGate 規則定期對帳（記入 follow-up）。
4. **並行串流衝突**：本工作樹有多串流即時寫入同檔；執行順序依單切片原則逐項提交，git worktree 隔離。
5. **OQ-E3**（玉山 Sandbox 憑證）仍待——正式模式 E2E 未驗證，mock 為到手前唯一可驗證路徑（ADR-0012）。

## 8. 已拍板決策記錄（2026-11-09，全數裁決完成）

| 決策 | 拍板結果 | 備註 |
|------|----------|------|
| D1 殼策略 | **a：殼載入平台伺服器** | WebView 指向 `http://127.0.0.1:{port}`；24 commands 降級為作者面工具 |
| D3 治理面板 | **併入 D7：二十七頁一次交付** | records／codes／release／net＋工作流補口三頁 |
| D5 打包 | **本輪啟用 NSIS** | `bundle.active=true`、`targets: ["nsis"]`；三平台擴張另議（T7） |
| D7 交付批量 | **27 頁全工作流（20 舊＋7 新）** | 清單見 D7 表；一次交付、四關一次過 |
| D8 建置鏈 | **無 JS 建置鏈** | 移除 `devUrl:1420`；無 npm／Vite；資產 `include_str!` 內嵌 |
| 欽定約束 | **零 JS／提防 Electron／效能・安全・可靠・穩定・易維護・易規模化・易演進** | 見 §3.5；任何例外需 ADR |
| CSS SSOT | **維持 `include_str!` 現狀**（`platform/src/api.rs:505`） | 編譯期內嵌、零執行期 I/O；`close_out.rs` 漂移測試強制同 commit 與 `docs/design-system.css` 同步 |

**實作移交**：本計畫已 implementation-ready；請切換至可寫入原始碼的代理（如 code 代理）依 §5 T1–T8 執行；plan 代理不執行原始碼變更。

## 9. P0/P1 執行切片（2026-11-10 評估後定稿；交由 code 代理執行）

> 前置：先跑 `cargo check -p platform` 取即時錯誤數。熱檔紀律：`api.rs`／`pages.rs` 若 30 分鐘內有他流寫入——讀取驗證後以**單點守衛替換**（出現次數 ≠ 1 即中止），否則等他流存檔。

### P0-1 安全回應標頭（G-SH1）
- **檔案**：`platform/src/api.rs`。
- **步驟**：
  1. 新增 `async fn security_headers`（`axum::middleware::from_fn` 形式）：對**每個**回應插入 `Content-Security-Policy: default-src 'self'; style-src 'self'; img-src 'self' data:; script-src 'none'`、`X-Content-Type-Options: nosniff`、`X-Frame-Options: DENY`、`Referrer-Policy: no-referrer`。
  2. 掛載於最外層 router 建構尾端（`.layer(axum::middleware::from_fn(security_headers))`），確保 public／protected／protected_api 三群全覆蓋。
- **P0-1a（CSP 前提，先做）**：`platform/templates/index.html` 僅 2 處內聯樣式——L404（`<form class="form-row" … style="display:flex; …">`）、L446（`<div id="audit-pagination" style="…">`）→ 移入 `web/assets/design-system.css` 新類別（`.audit-filter-form`、`.audit-pagination`，值沿用 token var）；同步 `docs/design-system.css`（`close_out.rs` 漂移測試強制同 commit）。L446 空 div 執行時檢查：若為模板條件殘留 → 補內容或移除。
- **驗收**：登入後 `Invoke-WebRequest` 首頁回應含四標頭；`/`、`/states`、`/admin/api/catalog`、`/login` 渲染不變；`grep 'style="'` 於 `platform/templates` = 0。
- **風險**：極低（純加成）。CSP 對 JSON API 無害。

### P0-2 /audit RBAC（R2）
- **檔案**：`platform/src/pages.rs` `audit_page`（現無閘——session 即可見全公司稽核）。
- **步驟**：`Extension(session)` 後、任何查詢前插入 ROLE_MANAGE 閘：`state.permissions.has_permission_db(&session.user_id, "ROLE_MANAGE").await`；false → 403 denied 頁（語彙同 `apihub.rs::forbidden_page`，可直接複用其 HTML 字串樣式）。
- **驗收**：store1 `GET /audit` → 403；admin → 200；T4 矩陣測試加入此斷言。
- **風險**：低；若平行流同時改此 fn → 守衛替換保護。

### P0-3 pages.rs 收斂（R1；擁有權另流，先查後補）
- **步驟**：`cargo check -p platform`；綠 → 跳過。非綠且 `pages.rs` mtime > 30 分鐘未動 → 依錯誤修復（01:11 時 13 錯：`AuditView` 缺 template 所用 `username/action/created_at`、`BatchRowFull` 缺 `amount_display/status_class`、askama 對 `Option<String>` 之 `.is_empty()`（改 `{% if let Some(x) = … %}` 或欄位改 String）、`:821` E0308 型別）。原則：以**模板為契約**對齊 struct，不得改語意。
- **驗收**：`cargo check -p platform` 0 錯 → `cargo test -p platform` 全綠 → 補 T4（訪客／admin／store1 × 頁面矩陣 smoke＋zero-JS 掃描）。

### P0-4 catalog KPI derive（G-T1a；**疑似已解，僅核對**）
- **步驟**：現行 `api_catalog.html`（L75–112）已改 `{{ esun_total }}`/`{{ native_total }}` 動態；舊硬編碼 `<dd>3</dd>/<dd>5</dd>` 已不存在。核對 handler `apihub_catalog` 傳值正確、模板無殘餘固定計數（`grep '<dd>[0-9]'`）。
- **驗收**：無固定計數；如發現殘餘 → handler 計算 `esun_endpoints().iter().filter(…mode 統計…)` 傳入。

### P1-1 Tauri 圖示與打包（R3）
- **現況**：`web/src-tauri/icons/icon.ico` **已存在**；`bundle.icon: []` 未接；server 未入包。
- **步驟**：
  1. `tauri.conf.json` `bundle.icon: ["icons/icon.ico"]`。
  2. sidecar：`cargo build --release -p platform --bin flaps-server` → 複製為 `web/src-tauri/binaries/flaps-server-x86_64-pc-windows-msvc.exe`；conf 加 `"externalBin": ["binaries/flaps-server"]`。
  3. `main.rs` 解析序：`FLAPS_SERVER_EXE` → `exe_dir/flaps-server.exe`（bundle 後 sidecar 去 triple 同目錄）→ `exe_dir/binaries/flaps-server-*.exe`（dev）。
  4. spawn 傳 `FLAPS_PORT=8090`；不強設 `FLAPS_DB_URL`（NSIS 預設 per-user 安裝、安裝目錄可寫 → `data/flaps.db` 可用；文件註明）。
- **驗收**：`cargo tauri build` 通過並含 sidecar。

### P1-2 啟動序無白屏（R4）
- `tauri.conf.json`：視窗加 `"label": "main"`、`"visible": false`。
- `main.rs`：`use tauri::Manager;`；setup 內 `ensure_platform_server()` 後 `app.get_webview_window("main").map(|w| w.show())`；失敗路徑**仍 show**（顯示連線指引，不困死在隱形視窗）。
- **驗收**：server 關閉冷啟 → 視窗於 spawn 嘗試後才現；server 在線 → 直達儀表板。

### P1-3 守衛掃描擴充（R7）
- `platform/tests/guards.rs`：`SCAN_DIRS` 5 → 7：＋`"web/src-tauri/src"`、＋`"crates/kem/src"`（擴充後 `files.len() >= 40` 下限不變即自然滿足）。
- **驗收**：`cargo test -p platform --test guards` 綠（新目錄零損傷）。

### P1-4 NSIS 建置＋安裝 smoke
- 前置：`cargo tauri --version`；缺 → `cargo install tauri-cli --locked`（一次性、需網路）。
- 步驟：於 `web/src-tauri` 執行 `cargo tauri build --bundles nsis` → 產物 `target/release/bundle/nsis/Flaps ERP_0.2.0_x64-setup.exe`；`/S` 靜默安裝 → 啟動 → 驗證（視窗載入儀表板；`tasklist` 見 flaps-server.exe 被 spawn；登入後開 `/admin/api/catalog`）→ `/S` 反安裝 → 程序清空。
- **證據**：產物路徑、tasklist 輸出、smoke 記錄 → PLAN_REGISTER addendum #19。
- **風險**：NSIS bundler 首次需下載工具鏈（需網路）；WebView2 runtime 開發機已具。

### 執行順序與依賴
`P0-1(+1a) → P0-2 → P0-3(先查) → P0-4(核對) → 四關全綠 → P1-1 → P1-2 → P1-3 → P1-4 → addendum #19`。
T4 矩陣測試（4 身份 × 頁面）於 P0-3 綠後立即補，作為 P1-4 smoke 前的自動化防護。
