# Flaps ERP 工作台 — 全面重新設計與開發：完成執行計畫

> **Plan file:** `.kilo/plans/1793033512795-flaps-erp-workbench-completion.md`
> **日期:** 2026-11-06｜**狀態:** Implementation-ready（可交實作代理執行）｜**修訂:** v1.1（增 §11 追溯矩陣／§12 來源登錄與查證／§13 決策推導／§14 計畫自我驗證）
> **觸發:** 使用者要求「依據完整規格計畫，對本產品進行全面重新設計與開發」——所有頁面、功能與實作必須完整，十二工藝維度（精緻/精美/精密/精良/精巧/精雕/精準/精湛/精品/精益/精研/精選）＋延伸維度（無障礙/效能/安全/耐用/可維護）達專業級，快速高效解決所有問題。
> **產品:** Flaps ERP 工作台（輔翼ERP 工作平台）＝Dev 根工作區 `core/`＋`platform/`＋`web/`（Rust Axum 0.7＋Askama＋SQLite，Strict Zero-JS）。
> **本計畫邊界:** 以 v2.0 規格套件為「完整規格計畫」權威版本；本檔為將其**執行完成**的主計畫（S0–S7），並補齊套件尚未涵蓋的完整性缺口（頁面齊備、防護測試、維運交付）。

---

## 0. 上位規格（完整規格計畫 v2.0 套件 — 交付物本文）

| # | 交付物 | 文件（本文） | 現狀 |
|---|---|---|---|
| ① | 改進方案（架構／模組／介面／互動設計） | `docs/product/REDESIGN_BLUEPRINT_v2.0.md`（PRD-RD-2.0） | Active；部分完成（見 §1） |
| ① 補 | UI/UX 規格 | `docs/ux/UI-UX_需求規格.md`＋`docs/knowledge-hub/手冊-A-F.md` §E | 草案→本計畫轉為驗收 |
| ② | 品質標準與驗收準則 | 藍圖 §4＋UI-UX §E5＋本檔 §3 | 已定義；待全數達成 |
| ③ | 測試驗證計畫 | `docs/testing/REDESIGN_TEST_PLAN_v2.0.md`（TEST-RD-2.0） | 已定義；Pending 項併入本計畫 §4 |
| ④ | 交付清單 | `docs/delivery/DELIVERY_MANIFEST_v2.0.md`（DEL-RD-2.0） | 已交付；完成版勾稽於本檔 §5 |
| 輔 | 治理與 SLO | `docs/management/營運治理機制.md`（G1 查詢 SLO 表） | 草案；SLO 納入 §3.4 |

> 規則：不重複四份交付物全文；實作代理**逐項完成時、同一提交更新其狀態表**（狀態＋指令＋退出碼），不得只在對話宣稱完成。

---

## 1. 現況基線（2026-11-06 盤點；S0 必須重測定案）

### 1.1 已驗證
- 測試：96 passed / 0 failed（REDESIGN 執行記錄）；R1 併行流回報 97 綠。→ **S0 重測為準**。
- 四關：`fmt` / `clippy -D warnings` / `test --locked` / `audit` 最近一次全 EXIT=0。
- 路由：記載 22 條（`PROJECT_STATUS.md`）＋`api.rs:230–254` 觀察 23 個註冊、`auth.rs:483–484` 子路由 → **S0 清點定案**；模板 11 個（glob 實測）；migrations 0001–0004（實察）；CSS 雙檔同步護欄測試綠（記載）。
- 已完成（藍圖 §5）：M1 分頁、M2 健康檢查、M3 待覆核 KPI、R-07 Secure cookie。
- 既有護欄：零 JS 掃描、`:root` token 同步測試、migrations 重放測試、CSRF 10/10、安全標頭 middleware、整數金額測試。

### 1.2 進行中（併行工作流）
- **R1 api.rs 模組化**：`platform/src/` 已出現 `com/`、`db/`、`search/`、`sync/` 子模組（對應單檔並存）；`web/` 尚未出現，api.rs 仍為主體。**單一寫者原則：禁止兩流同時編輯 api.rs**（見 §7）。

### 1.3 未完成（本計畫範圍）
| 類別 | 缺口 | 來源 |
|---|---|---|
| 頁面 | `/transactions`（清單＋搜尋＋分頁＋詳情）、`/accounts`、`/ai-decisions`、`/states` | UI-UX §E3、手冊 §E5、治理 G1.1 三處共同定義 |
| 模組 | M4 多幣別匯率、M5 傳票格式、M6 parse 規則表管理 UI | 藍圖 §2 |
| 介面 | §3.1 token 擴充、§3.2 每頁 7 狀態矩陣補齊 | 藍圖 §3 |
| 測試 | 無障礙/Lighthouse 基線、E2E 腳本、效能新案例 | TEST-RD-2.0 §7 Pending |
| 維運 | 備份/還原演練＋`BACKUP_RECOVERY.md`、CI workflow 驗證、（OTel 評估＝延後） | NEXT_ACTIONS、DEL-RD-2.0 §G |

---

## 2. 交付物①完成版：改進方案（架構、模組、介面與互動設計）

### 2.1 架構（精良／精密）

**R1 — api.rs 模組化（四步、每步四關全綠才進下一步）**（依藍圖 §1 地圖）：

| 步驟 | 新模組 | 內容 | 驗收 |
|---|---|---|---|
| R1.1 | `platform/src/web/mod.rs` | `router()`、`AppState`、共用型別、render 輔助 | 既有路由全數逐一存在（S0 清點為計數基線） |
| R1.2 | `web/dashboard.rs` | index、build_index、monthly_chart_svg、IndexTemplate、分頁 | 分頁測試續綠 |
| R1.3 | `web/{transactions,import,drafts,wizard}.rs` | create/驗證、import_*、drafts_*、ai_suggest、wizard_* | CSRF 矩陣不減 |
| R1.4 | `web/{reports,admin}.rs` | report_*、CSV、admin_*、parse_coverage | 全部測試綠 |

- 完成判準：api.rs 歸零或僅 re-export；`pub(crate)` 可見性強制（HTTP 型別不外洩至 core）；每 commit 附四關證據。

**R2 — 健康檢查強化**：✅ Done（維持；migrations 欄位＋503 degraded 語義）。

**R3 — 層次邊界不變量**：金額僅在 `flaps_core::money`（整數分）進出；web 層僅 `format_minor` 顯示（既有護欄延續）。

**R4（新增）— 統一錯誤頁**：400／403／404／500 統一模板（三段式訊息＋回首頁連結），API 路徑回 JSON；整合測試逐狀態碼驗形狀。

**R5（新增）— 防護測試 `platform/tests/guards.rs`**：
1. 全部模板：合法 UTF-8、無 NUL、`<script>` 計數=0、無內聯 `on*=`（防範模板污染回歸）。
2. 路由清單凍結測試（22＋新增；防止意外增刪）。
3. 每頁渲染守衛：種子資料＋已登入 session 逐頁 GET，斷言 200 與關鍵標記（見 §4.2）。

### 2.2 模組功能（精巧／精準）— 完成矩陣

**A. 既有 M 系（規格摘錄）**

| # | 模組 | 驗收準則 |
|---|---|---|
| M4 | 多幣別匯率 | `exchange_rate_scaled: i64`（10^4）延伸；報表以 TWD 換算欄（i128 中間值）；100 USD @31250 → 3,125.00；四捨五入規則文件化；來源與時間標註 |
| M5 | 正式傳票格式 | 單一交易傳票檢視（借貸分錄表＋編號＋摘要＋簽核欄留白、列印友好）；借=貸斷言 |
| M6 | parse 規則表管理 UI | `erp_config` parser 關鍵字 CRUD（ROLE_MANAGE；CSRF；變更寫 audit）；CRUD＋403＋CSRF 負向測試 |

**B. 頁面完整化（UI-UX §E3／手冊 §E5 → 驗收）**

| # | 頁面 | 路由（建議） | 必要功能 | 驗收準則 |
|---|---|---|---|---|
| P1 | 交易清單 | `GET /transactions`（`?q=`、`?page=`） | 搜尋（編號/摘要關鍵字，GET 唯讀）、分頁（上/下頁、首/末頁、Disabled 態）、列點入詳情、狀態標籤（0000/1000–1008/4000/9999 色＋文字並用） | 25 筆種子：第 1 頁 20、第 2 頁 5；越界 clamp；q 過濾正確 |
| P2 | 交易詳情 | `GET /transactions/:id` | 分錄明細（借貸）＋摘要＋日期＋狀態 | 存在 200／不存在 404 統一錯誤頁；< 200ms |
| P3 | 會計科目 | `GET /accounts`＋`POST /accounts`＋`POST /accounts/toggle` | 科目樹（縮排＋`details/summary`）；啟停切換（POST→PRG）；新增表單；Default/Empty 態 | 樹結構正確；403＋CSRF 負向；SLO 見 §3.4 |
| P4 | AI 決策 | `GET /ai-decisions`＋`POST /ai-decisions/decide` | 風險分 0–1 進度條（<0.3 綠/0.3–0.7 黃/≥0.7 紅＋文字）；建議卡＋來源規則可見；人工覆核；採納/拒絕（PRG） | 來源不可隱藏；採納走既有草稿路徑（永不直接落地）；audit 齊備 |
| P5 | 介面狀態 | `GET /states` | 全互動元件 7 狀態展示；連結設計系統 CSS | `script`=0；作為 DoD 目視樣板 |

> RBAC：檢視最低 `LEDGER_READ`；變更類沿用管理權（依 `permission.rs` 現有常數選擇並補 403 負向）。新 POST 一律 CSRF＋audit；讀取 GET、寫入 POST、成功 PRG。

### 2.3 介面與互動設計（精緻／精美／精湛）

1. **Token 擴充（§3.1）**：`--color-{success,warning,danger}-bg`、`--radius-{card,input,badge}`、`--shadow-card{,-hover}`、`--space-section`、`--focus-ring`；`prefers-reduced-motion` 全域；`web/assets/design-system.css` 與 `docs/design-system.css` 同步（P3-2 護欄把關）。
2. **狀態矩陣（§3.2）**：逐頁補齊 7 狀態（既有 11 頁＋新增 P1–P5）；`/states` 為可視化樣板；Playwright 選擇器存在性檢查。
3. **零 JS 互動法**：分頁/搜尋＝純連結＋GET 表單；上傳＝submit 後按鈕屬性＋CSS；錯誤＝三段式文案 100% 覆蓋；表格 hover；金額右對齊＋`tabular-nums`；傳票列印 CSS。
4. **無障礙**：語義 HTML、可見焦點（2px，≥3:1）、鍵盤全可達、非顏色單一傳達、標題無跳級、nav/main/footer landmarks。

---

## 3. 交付物②：品質標準與驗收準則

### 3.1 十二維度＋延伸 → 可驗證準則

| 維度 | 驗收準則（可執行證據） |
|---|---|
| 精緻 | 每頁 7 狀態矩陣齊備；字級/間距僅用 token |
| 精美 | 無障礙基線量測記錄（Lighthouse ≥95 或替代證據，見 OD-7）；對比 AA |
| 精密 | 金額 i64 分進出、i128 中間累加；tabular-nums 右對齊；日期 YYYY-MM-DD |
| 精良 | fmt/clippy/test/audit 四關全 EXIT=0（每 slice、每 commit） |
| 精巧 | 分頁/搜尋零 JS；健康檢查機器可讀 JSON |
| 精雕 | 錯誤三段式覆蓋 100%；模板編碼/零 JS 防護測試在案 |
| 精準 | 斷言具體值（筆數、金額字串、狀態碼、Location） |
| 精湛 | clippy allow 數=0；無 unsafe（每 .rs 首行 `#![forbid(unsafe_code)]`） |
| 精品 | 交付清單（§5）全稽；P0/P1 缺陷全結案 |
| 精益 | 缺陷入 BACKLOG 並結案；紅→綠→證據循環 |
| 精研 | 新做法附來源；效能先量測後最佳化 |
| 精選 | 依賴最小化；本計畫原則零新 crate |
| ＋無障礙 | WCAG 2.2 AA：鍵盤、焦點、語義、對比、非顏色、reduced-motion |
| ＋效能 | §3.4 SLO 全達標附量測 |
| ＋安全 | CSRF 全 POST、標頭、RBAC 負向、會話撤銷 |
| ＋耐用 | 冪等/PRG/重放防護；備份還原演練完成；DB 斷線 degraded |
| ＋可維護 | R1 完成；模組邊界 `pub(crate)`；文件與代碼同提交 |

### 3.2 硬性不變量（違反即缺陷）
1. `<script>` 數=0、內聯 `on*=`=0（模板變更後掃描）。
2. 金額零浮點。3. 讀取 GET、寫入 POST、成功 PRG。4. CSRF 覆蓋所有 POST。5. 每 `.rs` 首行 `#![forbid(unsafe_code)]`。6. 未知值以「—」呈現。

### 3.3 頁面 DoD
7 狀態齊備 ➜ 錯誤三段式 ➜ 鍵盤可達＋焦點可見 ➜ 對比 AA ➜ token-only ➜ 零 JS ➜ RBAC＋CSRF（如有寫入） ➜ 測試在案。

### 3.4 SLO（量測於 `platform/tests/performance/mod.rs`）
| 端點 | 目標 |
|---|---|
| `/transactions`（分頁） | P99 < 50ms |
| `/accounts` | P99 < 50ms |
| `/api/health` | < 100ms |
| `/reports/monthly`（2,000 筆） | < 500ms |
| `/transactions/:id` | < 200ms |
| 首頁（2,000 筆含分頁） | < 500ms |

### 3.5 證據規則
每項完成＝「指令＋退出碼＋結果」；UNKNOWN ≠ PASS ≠ FAIL；flaky 視同缺陷；環境阻擋記 Blocked；不編造數字。證據存 `.kilo/sessions/20261106-platform-completion/`。

---

## 4. 交付物③完成版：測試驗證計畫（TEST-RD-2.0 全量＋補齊）

> 測試案例 ID 對照見 §11.2；實作測試命名須對齊 ID（或於測試註解標註），以利機器／人工溯源。

### 4.1 單元（Unit）
parse_page（0/負/非數字/巨大值/正常→合法頁碼）；format_minor 邊界；M4 換算（i128 中間值、四捨五入）；健康檢查 payload 組裝；科目樹組裝；風險分區間判定。

### 4.2 整合（Integration，router oneshot）
| 目標 | 斷言 |
|---|---|
| 既有回歸 | 96–97＋全數通過，0 failed（每 slice 重跑） |
| 分頁（M1） | 25 筆→第 1 頁 20、第 2 頁 5；越界回末頁；非數字回第 1 頁不 500 |
| 健康檢查（M2） | 200 ok＋migrations 欄位；DB 關閉→503 degraded |
| Secure cookie | HTTPS 綁定時 Set-Cookie 含 Secure；HttpOnly＋SameSite=Lax 恆在 |
| **渲染守衛（R5-3）** | 種子＋登入 session 逐頁 GET：全部 200；關鍵標記（頁名/KPI 數字/分頁列）存在；`script`=0 |
| **路由凍結（R5-2）** | 既有路由（S0 清點基線）＋新增 P1–P5 路由全數存在（清單比對） |
| **CSRF 補齊** | 新增 POST（accounts create/toggle、ai-decisions/decide、m6 CRUD）逐一：缺 token→403；含量測計數 |
| **RBAC 負向** | 無權限角色存取新管理動作→403；停用即撤銷會話（既有） |
| **錯誤頁（R4）** | 400/403/404/500 形狀：三段式文案＋回首頁；API 路徑 JSON |
| **M4/M5/M6** | 換算值精確（312,500 分）；傳票借=貸；規則 CRUD 生效＋audit |

### 4.3 端對端（E2E，真實伺服器）
- 新增 `scripts/e2e-smoke.ps1`（延伸 seed/reset）：seed → 登入 → 逐頁瀏覽（含 P1–P5）→ 建立交易 → 分頁 → 搜尋 → 詳情 → 報表×4＋CSV → 管理頁 → 科目 toggle → reset；全程斷言 200/303、無 5xx、`script`=0。
- Playwright 工作階段：7 狀態選擇器存在性、鍵盤走查、焦點可見、reduced-motion 模擬。
- PowerShell 規則：`;` 串接、無 `&&`；`.ps1` 無 BOM 不含非 ASCII。

### 4.4 效能（Performance）
| 案例 | 預算 | 方法 |
|---|---|---|
| 月報表 2,000 筆 | < 500ms | 既有 close_out（RPT-001） |
| 首頁分頁 2,000 筆 | < 500ms | close_out 新案例：GET /?page=1 計時 |
| /transactions P99 | < 50ms | performance/mod.rs 新案例（2,000 筆種子，多輪取 P99） |
| /accounts P99 | < 50ms | 同上 |
| /transactions/:id | < 200ms | 同上 |
| /api/health | < 100ms | 同上 |
- 量測結果入 `artifacts/perf/`（日期戳記）＋報告文件；「先量測、後最佳化」，未達標即開 BACKLOG 缺陷。

### 4.5 安全（Security）
分頁/搜尋參數注入（`1%27OR` 等→非法回預設，bind 綁定）；CSRF 全 POST 計數只增不減；安全標頭回歸；帳號邊界（STORE→403）；會話撤銷；`cargo audit`（允許清單外零容忍）。

### 4.6 可用性（Usability）
| 案例 | 通過準則 |
|---|---|
| 鍵盤導覽 | skip-link→nav→main 序；全功能可鍵盤完成 |
| 焦點可見 | `--focus-ring` 對比 ≥3:1、全互動元件 |
| 語意 | h1→h2→h3 無跳級；landmarks 齊備 |
| 對比 | 文字 ≥4.5:1、大文字 ≥3:1（記量測表） |
| reduced-motion | 動畫停用 |
| 無障礙基線 | Lighthouse ≥95；若環境無 Node：Playwright＋axe 注入／人工清單替代並如實標記（OD-7） |

### 4.7 Soak／耐久
既有 soak（ignored）維持；備份還原演練（§6 S6）；DB 斷線 degraded 已驗。

### 4.8 回歸協議（每 slice 強制）
紅（先寫失敗測試→存證據）→ 綠（最小實作）→ 重構＋四關全 EXIT=0 → 提交（單一提交含測試、實作、文件更新）→ 更新狀態表。flaky 視同缺陷；UNKNOWN ≠ PASS。

---

## 5. 交付物④完成版：交付清單（對應 DEL-RD-2.0 並補齊）

### 5.1 A. 文件
| 文件 | 狀態 |
|---|---|
| REDESIGN_BLUEPRINT_v2.0（本文①） | 完成時：§5 狀態表全 Done |
| REDESIGN_TEST_PLAN_v2.0（③） | 完成時：§7 Pending 全數結案 |
| DELIVERY_MANIFEST_v2.0（④） | 完成時：§G 已知限制清零或列明殘餘 |
| UI-UX_需求規格 | 草案→v1.0 定稿（補實作狀態註記） |
| BACKUP_RECOVERY.md（新增） | 演練後交付（步驟＋演練證據） |
| CHANGELOG / PLAN_REGISTER addendum / PROJECT_STATUS / NEXT_ACTIONS | 每 slice 同步；收口時終版 |
| 營運治理機制 G2.2（測試缺口表） | 收口更新（41 缺口狀態） |

### 5.2 B. 程式碼
R1 web/ 模組（mod/dashboard/transactions/import/drafts/wizard/reports/admin）；R4 錯誤頁模板＋handler；R5 guards 測試；P1–P5 模板＋handlers；M4/M5/M6；`web/assets/design-system.css`＋`docs/design-system.css` token；`scripts/e2e-smoke.ps1`。

### 5.3 C. 測試報告
每 slice session log（`.kilo/sessions/20261106-platform-completion/`）；最終基線 log；效能量測（artifacts/perf）；無障礙/可用性紀錄；E2E 記錄；guards 記錄。

### 5.4 D. 部署與維運
啟動（FLAPS_PORT=8090、127.0.0.1）；健康探測 `/api/health`（degraded 語義）；備份/還原程序文件化＋演練；Session 清理（既有）；CI workflow 驗證；PRODUCTION_READINESS 重審（部署授權＝擁有者，NO-GO 由授權解除）。

### 5.5 完成判準（PA，全過即「全面重新設計與開發」完成）
- PA-01 所有規格頁面存在且驗收通過（/login、/、/new、/transactions＋詳情、/accounts、/ai-decisions、/states）。
- PA-02 R1 完成（api.rs 歸零/re-export；路由不變）。
- PA-03 M4/M5/M6 完成附測試。
- PA-04 Token 擴充＋7 狀態矩陣補齊。
- PA-05 測試計畫全執行（含 E2E、效能 SLO、無障礙基線或誠實替代）。
- PA-06 備份還原演練＋文件；CI 驗證；健康強化（已）。
- PA-07 交付清單全稽；P0/P1 缺陷 0。
- PA-08 最終修訂版四關 EXIT=0、零 JS、整數金額、CSRF 全覆蓋。

---

## 6. 執行序（Slices S0–S7；每 slice 四關全綠＋證據後才前進）

### S0 — 對賬與凍結基線（唯讀；半日）
- 跑四關並記修訂版＋退出碼；零 JS 掃描；模板編碼檢查；檢查 R1 併行流狀態（`web/` 是否出現、api.rs 行數、最新測試數）。
- 產出：baseline log＋差距清單（對照 §1.3）＋R1 交接判定（見 §7）。
- 驗收：log 入 session 目錄；本檔 §1 數字更新。

### S1 — 架構完成（R1＋R4＋R5）
- R1 四步逐步落地（協調見 §7）；R4 錯誤頁；R5 guards 測試。
- 驗收：每步四關 EXIT=0；路由凍結測試綠；api.rs 歸零/re-export。

### S2 — 頁面完整化（P1→P2→P3→P4→P5，逐一 TDD）
- 每頁：紅測試（渲染＋功能＋403/CSRF 負向）→ 實作 → 模板狀態齊備 → 綠 ＋ 四關。
- 驗收：§2.2-B 各頁驗收準則全過；渲染守衛覆蓋新頁。

### S3 — 模組 M4→M5→M6（逐一 TDD）
- M4 換算＋報表欄；M5 傳票＋列印 CSS；M6 CRUD＋audit。
- 驗收：§2.2-A 準則全過。

### S4 — 介面與互動（可與 S1/S2 並行；純 CSS/文件不觸 api.rs）
- Token 擴充＋雙檔同步；7 狀態矩陣逐頁補齊；`/states` 樣板；對比量測。
- 驗收：P3-2 護欄綠；矩陣審查表 100%。

### S5 — 測試深化（依賴 S2/S3 完成）
- `scripts/e2e-smoke.ps1`；效能新案例；安全案例；可用性＋無障礙基線（或替代）。
- 驗收：TEST-RD-2.0 §7 Pending 清零；SLO 達標並記錄。

### S6 — 維運（可與 S1–S5 並行）
- 備份/還原演練＋`BACKUP_RECOVERY.md`；CI workflow 驗證（阻擋則文件化）；PRODUCTION_READINESS 更新。
- 驗收：演練 log＋文件；CI 狀態如實記錄。

### S7 — 收口
- 文件同步（§5.1）；交付清單勾稽；最終四關＋全頁掃描＋零 JS；PA-01..08 核對。
- 驗收：全部 PA 達成；殘餘項以 OD/BACKLOG 明列（不暗藏）。

---

## 7. 並行工作協調（單一寫者原則）

| 區域 | 寫者 | 規則 |
|---|---|---|
| `platform/src/api.rs`、`web/*` | R1 流 → 本計畫接手 | **同一時間僅一方持有**；S0 判定：R1 已落地→直接採用；進行中→本計畫先做 S4/S6/文件等非 api.rs 任務，取得交接窗口後執行 S1 + 依賴項 |
| `platform/src/{com,db,search,sync}*` | 他流 | 只讀整合；未經協調不編輯 |
| templates、`web/assets`、docs、scripts | 本計畫 | 不衝突區，可先行 |
| 測試檔（platform/tests/*） | 遇到他流修改→先 rebase/合併再寫 | 衝突即停下協調，不合併半成品 |

---

## 8. 風險與緩解

| 風險 | 緩解 |
|---|---|
| R1 兩流衝突 | §7 單一寫者＋S0 判定＋交接窗口 |
| 模板污染/編碼回歸重演 | R5 guards（UTF-8/NUL/script/on* 掃描）＋每模板編輯後掃描 |
| Windows 應用程式控制阻擋建置 | `CARGO_TARGET_DIR=%LOCALAPPDATA%\Temp\dev-target`；`;` 串接；UTF-8 寫檔 |
| 無 Node→Lighthouse 不可用 | OD-7：Playwright/axe 或人工替代＋如實標記，不虛報 |
| 範圍蔓延 | 邊界＝v2.0 套件；延伸項（Dark mode、F2–F4）列 OD-5/OD-8 後續 |
| 證據疲勞/虛報 | §3.5 規則；每 slice 一次 canonical；UNKNOWN ≠ PASS |
| 效能未達標 | 先量測後最佳化；未達標開 BACKLOG 缺陷不放行 PA-05 |

---

## 9. 開立決策（含建議值；實作代理如無異議即採建議）

| # | 決策 | 建議 | 理由 |
|---|---|---|---|
| OD-1 | R1 歸屬與窗口 | S0 對賬：已落地即採用；進行中取交接窗口 | 避免雙寫者衝突 |
| OD-2 | `/transactions` 獨立頁 | 實作（index 保留「最近交易」摘要） | 規格三處共同定義；治理 SLO 明列 |
| OD-3 | `/ai-decisions` 動作範圍 | 採納/拒絕經既有 drafts 路徑＋audit；永不直接落地 | 沿用 AI 禁區不變量 |
| OD-4 | `/states` 展示頁 | 實作（QA/無障礙樣板） | DoD 目視驗收與長期維護價值 |
| OD-5 | Dark mode/進階動畫 | 延後（不在 v2.0 套件） | 範圍控制；列後續品質切片 |
| OD-6 | 部署授權（TLS/主機） | 維持 NO-GO，待擁有者 | 治理要求 |
| OD-7 | 無障礙量測方法 | 優先 Lighthouse；無 Node 則 Playwright/axe＋人工清單並標記替代 | 誠實證據原則 |
| OD-8 | F2 批次/F3 自訂報表/F4 作廢沖銷 | 延後，列 BACKLOG | 無驗收規格於套件；先完成定義內範圍 |

---

## 10. 附錄

### 10.1 指令速查（PowerShell；`;` 串接）
```powershell
$env:CARGO_TARGET_DIR = "$env:LOCALAPPDATA\Temp\dev-target"
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
cargo audit
```
- 零 JS 掃描：對 `platform/templates/*.html` 計數 `<script` 與 `on*=` 皆為 0。
- 啟動：`cargo run -p platform --bin flaps-server`（127.0.0.1:8090）。

### 10.2 關鍵路徑
`platform/src/api.rs`（R1 母體）、`platform/templates/`（11＋新 5 頁）、`platform/tests/{close_out,performance,e2e,integration,guards}`、`core/src/{money,ledger}`、`web/assets/design-system.css`＋`docs/design-system.css`、`scripts/{seed-demo,reset-demo,e2e-smoke}.ps1`、`docs/product/REDESIGN_BLUEPRINT_v2.0.md`。

### 10.3 詞彙
M1–M6＝藍圖模組項；R1–R5＝架構項；P1–P5＝新頁面；S0–S7＝本計畫 slices；PA＝完成判準；OD＝開立決策。

---

## 11. 追溯矩陣（可追溯）

### 11.1 使用者需求 → 計畫對策（原文級追溯）

| 使用者原話（歷次訊息） | 解讀 | 計畫對策（章節） | 驗收 |
|---|---|---|---|
| 「現有版本存在錯誤」 | 執行期錯誤與頁面破損風險 | S0 重測＋R4 統一錯誤頁＋R5 防護測試 | PA-08／R4／R5 |
| 「功能不穩」 | flaky／回歸風險 | §4.8 回歸協議（flaky 視同缺陷）＋每 slice 四關 | PA-08 |
| 「外觀粗糙」 | 視覺與狀態不完備 | S4：Token 擴充＋7 狀態矩陣＋`/states` 樣板 | PA-04 |
| 「耐用性不足」 | 資料與運維韌性 | S6：備份還原演練＋degraded（既有）＋冪等（既有） | PA-06 |
| 「與規格計畫相距甚遠」 | 規格未落實 | §0 套件逐項完成＋狀態表同步（不宣稱未做） | PA-01／PA-05 |
| 「所有頁面、功能與實作必須完整」 | 頁面齊備 | S2（P1–P5）＋S3（M4–M6） | PA-01／PA-03 |
| 「十二維度…專業級品質」 | 品質標準 | §2.3＋§3.1（維度→可驗證準則） | PA-04／PA-07 |
| 「快速高效解決所有問題」 | 執行效率 | §6 序列＋§7 並行安全區（衝突面最小化） | 方法性（過程紀錄） |
| 「可追溯可查證可解釋／可推導／驗證完全」 | 計畫自身品質 | §11／§12／§13／§14 | §14 全項 |

### 11.2 規格項 → 計畫項 → 測試 ID → 證據（完整矩陣）

| 來源規格 | 計畫項 | 測試 ID（命名對齊） | 證據位置 |
|---|---|---|---|
| 藍圖 R1.1–R1.4 | S1 | `r1_step1_router` … `r1_step4_reports_admin` | 每步四關 log |
| 藍圖 R2 | 已完成 | `health_ok_payload`、`health_degraded_503`（既有） | 基線 log |
| 藍圖 R3 | 沿既有 | 金額邊界測試（既有） | 基線 log |
| 本計畫 R4 | S1 | `errpage_400/403/404/500`、`errpage_api_json` | 整合測試 log |
| 本計畫 R5 | S1 | `guard_templates_utf8_zerojs`、`guard_routes_frozen`、`guard_render_all_pages` | guards log |
| 藍圖 M1/M2/M3/R-07 | 已完成 | `pagination_*`、`health_*`、`pending_review_kpi`、`secure_cookie_*`（既有） | 96 基線 |
| 藍圖 M4 | S3 | `mc_convert_exact_312500`、`mc_report_twd_column` | 單元＋整合 |
| 藍圖 M5 | S3 | `voucher_debit_credit_balance`、print-CSS 審查 | 整合＋目視 |
| 藍圖 M6 | S3 | `rules_crud_roundtrip`、`rules_403`、`rules_csrf_negative` | 整合 |
| UI-UX E3.4／手冊 E5／治理 G1.1 | S2-P1 | `txl_pagination_25_rows`、`txl_clamp_bounds`、`txl_search_q`、`txl_status_label_text` | 整合＋SLO 量測 |
| UI-UX E3.4 | S2-P2 | `txd_detail_ok`、`txd_detail_404` | 整合 |
| UI-UX E3.5／治理 G1.1 | S2-P3 | `acc_tree_structure`、`acc_toggle_prg`、`acc_403_csrf_negative` | 整合＋SLO 量測 |
| UI-UX E3.6 | S2-P4 | `aid_risk_bands`、`aid_source_visible`、`aid_adopt_via_drafts` | 整合 |
| UI-UX E3.7 | S2-P5 | `states_page_zero_js` | 整合＋目視 |
| TEST-RD §3 | S5 | `e2e_smoke_all_pages` | 腳本 log |
| TEST-RD §4／治理 G1.1 | S5 | `perf_index_paged_2000`、`perf_txl_p99`、`perf_acc_p99`、`perf_health`、`perf_txd_200ms` | artifacts/perf |
| TEST-RD §5 | S5 | `sec_page_param_injection`、`csrf_count_no_regression` | 整合 |
| TEST-RD §6 | S5 | `a11y_keyboard`、`a11y_focus_ring`、`a11y_contrast`、`a11y_reduced_motion`、`a11y_baseline_*` | 量測表＋log |
| NEXT_ACTIONS #1 | S6 | `bak_restore_drill` | 演練 log＋BACKUP_RECOVERY.md |
| NEXT_ACTIONS #4 | S6 | `ci_workflow_verify`（或 Blocked 記錄） | CI log |
| §0 交付物本文狀態 | S7 | `docs_status_sync`（三份狀態表） | 勾稽表 |

### 11.3 PA → 驗證方法（每個完成判準怎麼驗）

| PA | 驗證方法 | 證據 |
|---|---|---|
| PA-01 | guards 渲染測試＋Playwright／人工逐頁走查（7 頁） | 測試 log＋走查表 |
| PA-02 | 四關全綠＋路由凍結測試＋api.rs 歸零／re-export 檢查 | 四關 log |
| PA-03 | M4/M5/M6 測試案例全過 | 測試 log |
| PA-04 | token 同步護欄測試＋矩陣逐頁簽核表 | 測試 log＋審查表 |
| PA-05 | S5 全部案例＋SLO 量測達標 | artifacts/perf＋量測表 |
| PA-06 | 演練 log＋CI 紀錄＋health 測試 | 演練 log |
| PA-07 | DEL-RD-2.0 勾稽＋BACKLOG 掃描（0 P0/P1 open） | 勾稽表 |
| PA-08 | 四關＋`script` 掃描＋金額測試＋CSRF 計數 | 四關 log |

## 12. 來源登錄與查證（可查證）

### 12.1 來源登錄（本計畫所有依據）

| 文件 | 版本／日期 | 本計畫用途 | 查證方式與等級 |
|---|---|---|---|
| `docs/product/REDESIGN_BLUEPRINT_v2.0.md` | PRD-RD-2.0 Active 2026-11-06 | M／R／token／矩陣／品質／§5 狀態 | 全文已讀（154 行）；2026-11-06 12:16 複查 §5 未變 |
| `docs/testing/REDESIGN_TEST_PLAN_v2.0.md` | TEST-RD-2.0 2026-11-06 | 測試全量＋§7 Pending | 全文已讀（61 行） |
| `docs/delivery/DELIVERY_MANIFEST_v2.0.md` | DEL-RD-2.0 2026-11-06 | 交付清單＋§G 限制 | 全文已讀（50 行） |
| `docs/ux/UI-UX_需求規格.md` | v1.0 草案 2026-11-06 | 頁面 E3／狀態／無障礙 E4／DoD E5 | 全文已讀（148 行） |
| `docs/knowledge-hub/手冊-A-F.md` | §E 區段 | 頁面清單 E5／錯誤三要素／狀態矩陣 | 行 225–307 已讀 |
| `docs/management/營運治理機制.md` | v1.0 草案 | G1 查詢 SLO／G2.2 測試缺口 | 行 1–80 已讀 |
| `docs/PROJECT_STATUS.md` | 2026-11-04 | 期程／90 測試／22 路由記載 | 已讀 |
| `docs/PLAN_REGISTER.md` | addenda 1–10 | 沿革／四關證據史 | 行 1–150 已讀 |
| `docs/NEXT_ACTIONS.md` | — | 維運項 #1–#4 | 已讀（35 行） |
| `platform/src/`、`migrations/`、`templates/` | 2026-11-06 12:16 實察 | 現況：`web/` 不存在、11 模板、0001–0004 | 目錄／glob 實察 |
| root `.git/logs/HEAD` | — | 提交沿革（含 R1 相關） | 已讀（截斷不影響結論） |
| 根 `AGENTS.md` | v1.0 | 建置規範（CARGO_TARGET_DIR、`;` 等） | 已讀 |

### 12.2 關鍵數字推導鏈（每個數字可追出處與重驗方式）

| 數字 | 出處 | 等級 | S0 重驗方式 |
|---|---|---|---|
| 96 passed／0 failed | DEL-RD-2.0 §F（執行記錄） | 記載（規劃期未重跑） | `cargo test --locked` 計數 |
| 97 綠 | 藍圖 §5（R1 註記） | 記載 | 同上 |
| 22 路由 | PROJECT_STATUS 記載；實察 api.rs 23 註冊 | **待清點** | 路由凍結測試＋清點 |
| 11 模板 | glob 實測（本次） | 已驗 | 再 glob |
| 0001–0004 migrations | 目錄實察 | 已驗 | 目錄檢視 |
| CSRF 10/10 | PLAN_REGISTER／PROJECT_STATUS | 記載 | 測試計數 |
| SLO 50／100／200／500ms | 營運治理 G1.1（草案） | 草案值（量測後定案；允許依量測調整並註記） | 效能測試量測 |
| 41 測試缺口 | 營運治理 G2.2 | 草案記載 | 收口時更新 |

### 12.3 規劃效力邊界（誠實聲明；UNKNOWN ≠ PASS）

- 本規劃**未執行任何測試、建置或變更**（唯讀盤點）；所有測試數字為「記載值」，S0 必須重測後才可引用為事實。
- 未與 R1 併行流直接通訊；R1 狀態為「文件＋目錄觀察」之判讀（`web/` 未出現 ≠ 必然未進行）；以 S0 交接判定為準。
- 本計畫不修改 v2.0 套件本文；套件狀態表由實作代理逐項更新（同一提交）。
- 若實作期發現本計畫與現況牴觸（例如 R1 已落地且結構不同），以**現況＋套件本文**為準，修訂本計畫並記差異（不得默默偏離）。

## 13. 決策推導（可解釋／可推導）

### 13.1 排序推導（為什麼 S0→S1→S2→S3；S4/S6 可並行；S5 最後）
- **S0 先行**：所有後續決策依賴可信基線；記載 ≠ 事實（§12.2），先重測凍結。
- **S1 先於 S2/S3**：新頁面（P1–P5）與 M6 都會新增路由、進入 `api.rs` 領地；R1 正在搬移同一檔案——先把領地整理完成（或確認其已落地），避免「同一路由被搬兩次」的雙重工作與衝突。
- **S4 可並行**：token／CSS／文件不觸 `api.rs`（衝突面＝0）；且狀態矩陣與 `/states` 樣板是 S2 各頁的驗收依據，可預先備好。
- **S6 可並行**：備份演練／CI 與程式碼面無交集。
- **S5 最後**：E2E／效能／無障礙驗證對象是 S2/S3 的成品；提前執行會重測、證據失效。
- 推導原則：**衝突面最小、依賴面單向**（對所有任務逐一檢核後排序）。

### 13.2 範圍推導（為什麼 P1–P5 在內、F2–F4 在外）
- **P1–P5 在內**：三份獨立文件共同定義（UI-UX §E3＋手冊 §E5＋治理 G1.1）→ 多源一致＝高信度，屬「完整規格計畫」範圍。
- **M4–M6 在內**：藍圖 §2 明列且 §5 標 Pending（Active 文件）。
- **F2 批次交易／F3 自訂報表／F4 作廢沖銷在外的**：僅見於較舊的多維演進計畫；v2.0 套件無對應驗收規格 → 依「邊界＝權威套件」推導為後續（OD-8）。
- **Dark mode／進階動畫在外**：僅舊計畫；v2.0 §3.1 token 清單不含 → 後續品質切片（OD-5）。

### 13.3 協調推導（為什麼單一寫者）
- 證據：藍圖 §5 自記「本藍圖執行方暫停觸碰 api.rs 以免衝突」→ 雙流碰撞風險**已實際發生並被記錄**；推導出 §7 單一寫者＋S0 交接判定＋不衝突區先行。

### 13.4 證據制度推導（為什麼「指令＋退出碼」）
- 沿革證據：PLAN_REGISTER addenda 1–10 全以「指令＋EXIT=0＋修訂版」記載；歷史上有模板損壞、未驗證宣稱等事故 → 制度性防呆（§3.5）而非個人細心。

## 14. 計畫自我驗證（驗證完全）

S0 執行時（確認計畫與現況一致）與 S7 執行時（確認交付與計畫一致），逐項核對並記錄：

- [ ] 14.1 **來源覆蓋掃描**：逐份讀 §12.1 全部來源；凡標 `Pending`／`未完成`／`Not Run` 處，皆能對應到本計畫某項，或有 OD／BACKLOG 記錄（無遺漏、無暗項）。
- [ ] 14.2 **頁面覆蓋**：UI-UX E3.1–E3.7＋手冊 E5 全 7 項 → 每項有計畫項＋測試 ID＋證據（§11.2 逐列核對）。
- [ ] 14.3 **模組覆蓋**：藍圖 R1–R3、M1–M6 全數「已完成或已排程」。
- [ ] 14.4 **測試覆蓋**：TEST-RD-2.0 全部案例 → 有 ID、有 slice、有證據路徑；§7 Pending 清零。
- [ ] 14.5 **交付覆蓋**：DEL-RD-2.0 §A–§D 全項＋§G 限制清零或明列。
- [ ] 14.6 **PA 覆蓋**：PA-01..08 每項有可執行驗證方法（§11.3）。
- [ ] 14.7 **不變量覆蓋**：§3.2 六條不變量各有自動檢測（掃描／測試）。
- [ ] 14.8 **反面檢查**：專案中任一「已知問題」清單（IMPROVEMENT_BACKLOG、NEXT_ACTIONS、G2.2 缺口表）不含未映射項。
- [ ] 14.9 **證據存在性**（S7）：§5.3 所列每份證據檔案在 `.kilo/sessions/…`／`artifacts/` 真實存在且非空。
- [ ] 14.10 **狀態表同步**（S7）：藍圖／測試計畫／交付清單三份狀態表與實況一致（無虛標 Done）。

> 執行方式：14.1–14.8 於 S0 全跑一次並記錄；14.9–14.10 於 S7 全跑一次並記錄。未過項即修正計畫或補做，不得跳過。

*本計畫為執行主計畫；完成時以 PA-01..08 全數達成且四關全綠結案。歷史紀錄不刪改，狀態表疊加更新。*
