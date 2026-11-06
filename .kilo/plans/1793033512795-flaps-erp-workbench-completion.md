# Flaps ERP 工作台 — 全面重新設計與開發：完成執行計畫

> **Plan file:** `.kilo/plans/1793033512795-flaps-erp-workbench-completion.md`
> **日期:** 2026-11-06｜**狀態:** Implementation-ready（可交實作代理執行）
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
- 路由 22 條（`platform/src/api.rs:230–254`）；模板 11 個；migrations 0001–0004；CSS 雙檔同步護欄測試綠。
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
| R1.1 | `platform/src/web/mod.rs` | `router()`、`AppState`、共用型別、render 輔助 | 路由 22 條逐一存在 |
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
| **路由凍結（R5-2）** | 既有 22 條＋新增 P1–P5 路由全數存在（清單比對） |
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

*本計畫為執行主計畫；完成時以 PA-01..08 全數達成且四關全綠結案。歷史紀錄不刪改，狀態表疊加更新。*
