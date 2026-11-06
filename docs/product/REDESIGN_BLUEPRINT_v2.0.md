# REDESIGN BLUEPRINT v2.0 — Flaps ERP 工作台全面深化設計

> 編號：PRD-RD-2.0｜日期：2026-11-06｜狀態：Active（執行中，逐項綁驗收證據）
> 上位文件：PRD.md、ADR-0001（技術基線）、ADR-0002（API 層）、ADR-0003（複式記帳授權）、MVP_SCOPE
> 原則：**深化，不是推翻**。ADR-0001/0002/0003 的決策（零 JavaScript、整數分貨幣、PRG、RBAC、SQLite、Askama SSR）全部保留；本計畫在已驗證骨架上補足專業級細節。所有改進項完成時必須附「指令＋退出碼」證據，UNKNOWN 不等於 PASS 也不等於 FAIL。

---

## 0. 現況誠實盤點（2026-11-06）

| 維度 | 現況 | 證據 |
|---|---|---|
| 測試 | 96+ 通過 0 失敗（core 30＋platform lib 47＋bridge 7＋close_out 4＋slice12） | cargo test --locked |
| 安全 | Argon2id＋session cookie＋CSRF 10/10＋登入速率限制＋停用即撤銷會話＋安全標頭 | BACKLOG P1-6..P1-9 |
| 資料正確性 | 整數分（i64）＋複式平衡驗證＋migrations SSOT | P0-1/P1-2/P1-5 |
| **結構債** | platform/src/api.rs 約 160 KB／4200 行單體（40+ handler 同檔） | 檔案實測 |
| **功能缺口** | 交易清單全量渲染（無分頁）；/api/health 無 DB 版本資訊；session cookie 無 Secure 屬性（R-07 殘餘） | 原始碼檢視 |
| **互動缺口** | 清單無分頁導覽；KPI「待覆核」永遠顯示「—」（無資料來源） | 模板檢視 |
| 視覺 | Design tokens 已建（16 KB CSS）；雙檔同步護欄已入 | P3-2 |

---

## 1. 架構改進（精良／精密）

### R1 — api.rs 模組化拆分（最大結構債，分四步執行、每步全綠）

現況 4200 行單體違反單一職責與可維護性。拆分地圖（每步後 fmt/clippy/test 全綠才進下一步）：

| 步驟 | 新模組 | 內容（原 api.rs 函式） | 風險 |
|---|---|---|---|
| R1.1 | platform/src/web/mod.rs | router()、AppState、共用型別、render_with_status、render_or_500 | 低（移動＋re-export） |
| R1.2 | web/dashboard.rs | index、build_index、monthly_chart_svg、IndexTemplate、分頁（新） | 中（本次同時加分頁） |
| R1.3 | web/transactions.rs＋web/import.rs＋web/drafts.rs＋web/wizard.rs | create/validate_and_post；import_*；drafts_*、ai_suggest；wizard_* | 中 |
| R1.4 | web/reports.rs＋web/admin.rs | report_monthly/pivot/parallel/CSV；admin_*、parse_coverage | 中 |

驗收：每步 `cargo fmt -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test --locked` 全 EXIT=0；router() 對外路徑不變（22 條路由逐一存在）；api.rs 最終歸零或僅存 re-export。

### R2 — 健康檢查強化（本輪實作）

/api/health 由「SELECT 1」升級為：DB 可達＋schema/migration 版本＋交易總數；DB 不可達時回 503 與 status=degraded（機器可讀，供部署層探測）。

### R3 — 層次邊界不變量（強化既有）

- 金額只在 flaps_core::money（整數分）進出，web 層僅 format_minor 顯示（已有測試護欄）。
- HTTP 內部型別（SendOutcome 等）不得洩漏至 core：R1 拆分時以 pub(crate) 可見性強制。

---

## 2. 模組功能改進（精巧／精準）

| # | 模組 | 設計 | 驗收準則 |
|---|---|---|---|
| M1 | 交易分頁（本輪實作） | GET /?page=N（N≥1，非數字→1、<1→1、>末頁→末頁）；每頁 20 筆；伺服器端 LIMIT ? OFFSET ?；模板渲染「第 X / Y 頁・共 Z 筆」＋上一頁/下一頁連結；頁碼參數永遠以 bind 綁定，不拼接 SQL | 建立 25 筆 → 第 1 頁 20 筆、第 2 頁 5 筆；page=999 回末頁；page=abc 回第 1 頁；整合測試 3 條全過 |
| M2 | 健康檢查（本輪實作） | 見 R2 | health 測試：DB 可達回 ok＋migrations 欄位；斷線回 503 degraded |
| M3 | 待覆核 KPI 落地 | transactions.status 已有欄位；「待覆核」＝status='PENDING_REVIEW' 計數；KPI 顯示真實數字，無資料顯示 0 而非「—」 | 整合測試：種子 2 筆待覆核 → KPI 顯示 2 |
| M4 | 多幣別匯率 | 既有 exchange_rate_scaled: i64（10^4）延伸：報表以 TWD 換算欄呈現（i128 中間值）；換算率來源與時間標註 | 換算測試：100 USD @31250 → 3,125.00；四捨五入規則文件化 |
| M5 | 正式傳票格式 | 報表新增「傳票檢視」（單一交易：借貸分錄表＋編號＋摘要＋簽核欄留白列印友好） | 傳票渲染含全部分錄且借=貸 |
| M6 | parse 規則表管理 UI | erp_config parser 關鍵字 CRUD 頁（ROLE_MANAGE；CSRF；變更寫 audit） | CRUD＋403＋CSRF 負向測試 |

---

## 3. 介面與互動設計（精緻／精美／精湛）

### 3.1 設計 Token 擴充（web/assets/design-system.css；同步 docs/design-system.css）

新增語意 token（現有基礎上）：

- --color-success-bg / --color-warning-bg / --color-danger-bg（狀態底色）
- --radius-card / --radius-input / --radius-badge（三級圓角）
- --shadow-card / --shadow-card-hover（兩級陰影）
- --space-section（區塊節奏）
- --focus-ring（:focus-visible 統一）

約束：prefers-reduced-motion 全域尊重（既有）；全部經 cascade layers；新 token 同一提交雙檔同步（P3-2 護欄測試把關）。

### 3.2 每頁元件狀態完備矩陣（驗收對照表）

| 頁面 | Default | Hover | Focus | Disabled | Loading | Empty | Error |
|---|---|---|---|---|---|---|---|
| 總覽（index） | 有 | 有 | 有 | — | 有（既有樣式） | 有（無交易顯示建立入口） | 有（DB 斷線「—」＋degraded） |
| 交易清單 | 有 | 有 | 有 | — | — | 有 | 有 |
| 匯入 | 有 | 有 | 有 | 有（上傳中按鈕 disabled） | 有 | 有 | 有 |
| 草稿/精靈 | 有 | 有 | 有 | — | — | 有 | 有 |
| 報表×4 | 有 | — | 有 | — | — | 有（無資料月份提示） | 有（非法參數 400 頁） |
| 管理頁 | 有 | 有 | 有 | 有（停用自己 disabled） | — | 有 | 有 |

驗收：每頁由 template 審查清單＋Playwright 選擇器存在性檢查（測試計畫 §U）。

### 3.3 互動流程設計（零 JS 達成法）

- 分頁導覽：純連結 ?page=N（GET 唯讀，符合 form_http_rules）。
- 上傳回饋：submit 後按鈕 disabled 屬性＋CSS :active；required＋:user-invalid 提交前回饋。
- 錯誤訊息三段式（問題＋可能原因＋處理方式）延續 FR-004，新錯誤文案比照。
- 表格列 hover 高亮；金額右對齊＋font-variant-numeric: tabular-nums（會計專業細節）。

---

## 4. 品質標準與驗收準則（精品／精益／精選）

### 4.1 十二維度 → 可驗證準則對照

| 維度 | 驗收準則（可執行證據） |
|---|---|
| 精緻 | 每頁 7 狀態矩陣齊備（3.2）；字級/間距僅用 token |
| 精美 | Lighthouse Accessibility ≥ 95（首次基線量測並記錄）；色彩對比 AA |
| 精密 | 金額 i64 分進出、i128 中間累加；tabular-nums 右對齊；日期統一 YYYY-MM-DD |
| 精良 | fmt/clippy -D warnings/test --locked/audit 四關全 EXIT=0（R1 每步都跑） |
| 精巧 | 分頁零 JS（純連結）；健康檢查機器可讀 JSON |
| 精雕 | 錯誤文案三段式覆蓋率 100%（新錯誤全數遵守） |
| 精準 | 整合測試斷言具體值（筆數、金額字串、狀態碼、Location） |
| 精湛 | clippy allow 數 = 0；無 unsafe（每 .rs 首行 forbid） |
| 精品 | Phase 2 驗收清單全過＋交付清單齊備 |
| 精益 | 每 defect 入 BACKLOG 並結案（Status＋證據） |
| 精研 | 每一新做法附來源（AWS/MDN/RFC 引用） |
| 精選 | 依賴面最小化：本計畫全部項目零新 crate |

### 4.2 硬性不變量（違反即缺陷）

1. script 標籤數 = 0、內聯 on* 事件 = 0（憲章 §2.3）——模板變更後 grep 驗證。
2. 金額零浮點（clippy＋專用測試）。
3. 讀取 GET、寫入 POST、成功後 PRG（路由審查）。
4. CSRF 覆蓋所有 POST（既有 10/10 護欄＋新增 POST 同步補）。
5. 每個 .rs 首行 #![forbid(unsafe_code)]。
---

## 5. 執行順序與狀態

| 順序 | 項目 | 狀態 |
|---|---|---|
| 1 | M1 分頁＋M2 健康檢查＋R-07 Secure cookie（本輪，零新依賴） | **Done（2026-11-06）**：分頁 3 整合測試＋parse_page 7 單元斷言＋health degraded 測試通過；R-07 經查已於先前切片完成（auth.rs `security_flag("cookie_secure")` config 驅動） |
| 2 | R1 api.rs 模組化 | **進行中（併行工作流）**：db/、sync/、search/、com/ 子模組已出現並與本輪改動相容（97 tests 綠）；本藍圖執行方暫停觸碰 api.rs 以免衝突 |
| 3 | M3 待覆核 KPI | **Done（2026-11-06，TDD 先紅後綠）**：RED=pending_review_kpi 測試失敗（佔位符未消失）；GREEN=COUNT 查詢＋模板渲染→54 platform tests 綠；與併行 R1/D3 工作共存（97 全工作區綠） |
| 4 | 3.1 token 擴充＋3.2 矩陣補齊 | Pending（.pagination 樣式已入 web/assets；docs 版為文件快照，:root 同步護欄綠） |
| 5 | M4 多幣別 → M5 傳票 → M6 規則 UI | Pending |
| 6 | Playwright 可用性腳本＋Lighthouse 基線 | Pending |

### 本輪驗收證據（2026-11-06，指令＋結果）

| 項目 | 指令 | 結果 |
|---|---|---|
| 全域四關 | cargo fmt --check / clippy --all-targets -D warnings / test --locked / audit（CARGO_TARGET_DIR 導向 %LOCALAPPDATA%） | **全 EXIT=0；96 passed / 0 failed**（原 90＋6） |
| 零 JS 護欄 | grep templates/index.html | script=0、內聯 on*=0 |
| 分頁 | cargo test -p platform pagination | 3 整合測試通過（20+5 分頁、999 clamp 末頁、abc/0/-3 回第 1 頁不 500） |
| 健康檢查 | cargo test -p platform health | ok 時 200＋migrations/transactions 欄位；關閉 DB 後 503 degraded |
| CSS 同步 | P3-2 護欄測試（:root 逐字） | 綠 |

---

## 6. 耐用性設計（耐用）

- 冪等：PRG 已防重複提交（既有）；匯入 confirm 冪等（批次 id 鎖）。
- 退化：DB 斷線 → 全頁「—」＋degraded 樣式＋/api/health 503（機器可測）；AI 引擎不可用 → 規則引擎照常（既有降級閘）。
- 回復：SQLite 單檔備份＋還原演練程序（NEXT_ACTIONS #1，Phase 2）；migrations 可重放（既有 3 測試）。
- 演進：migrations SSOT＋ BACKLOG 制度；本藍圖逐項結案制。