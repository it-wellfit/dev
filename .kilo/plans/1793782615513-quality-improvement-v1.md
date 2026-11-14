# AUXBRIDGE 全面品質提升方案 v1.0

> 日期：2026-11-13｜狀態：Active｜依據：576 問矩陣＋TT-1..5＋REQ→DEC→IMPL→TEST→EVID 追溯體系（`docs/standards/REQUIREMENTS_TRACEABILITY.md`）

## 0. 量測基線（2026-11-13 實證）

| 維度 | 現況 | 證據 |
|---|---|---|
| 測試 | 146 lib / 0 failed | `cargo test --locked -p platform --lib` |
| 頁面 | 66 manifest 頁、32 頁掃描 200、77+ 連結全通 | E2E 報告 |
| RBAC | 66 頁×4 角色 264 組零越權 | `rbac_matrix` 工具 |
| 圖標 | **10 emoji 於 manifest（📒📥🏦📈🔍🛡️⚙️🔌🖥️📚）**；模板 9 個 ✓/✕ 文本符號（合法，非圖標） | 本輪實測 |
| 空態 CTA | index.html:212 已 1 處（✓ 前往新增交易按鈕）；其餘空態無 CTA | 本輪實測 |
| 麵包屑 | 已實作（nav.rs SectionLabel＋studio_nav.html＋CSS），**已部署驗證**（總覽 › 業務工作室 › 帳務作業） | 8090 實測 True |
| 樣式債 | design-system.css 5 處舊藍 rgba(37,99,235)→玉山綠 ✓ 已清 | 殘留 0 |
| 漢堡抽屜 | ≤900px ☰ 選單（checkbox hack）✓ 已部署 | 8090 實測 |
| 焦點雙環 | 玉山綠雙環 ✓；::placeholder token 化 ✓ | 8090 實測 |
| 效能 | 149→146 tests；home 56KB；CSS 74KB；搜尋 46ms | curl 實測 |

## 2. 全面品質提升計劃（四維度）

### 維度 1：工作流程與操作細節

| # | 項目 | 狀態 | 下一步 |
|---|---|---|---|
| W-1 | **採納流程 100% 斷鏈**（P0，使用者實測） | 根因＝parse_decimal_to_minor 拒絕千分位；**已修復**（strip 逗號，E2E 12,000.00→入帳） | ✅ 完成；補 D12 永久回歸 |
| W-2 | **匯入範本閉環**（P0） | CSV/TSV/XLSX 三格式範本下載＋零映射上傳入帳 | ✅ 完成（CSV 32→34、XLSX 34→36） |
| W-3 | **零筆匯入裝成功**（P0） | 0 rows → 422＋失敗清單 | ✅ 完成（import_confirm） |
| W-4 | **BOM 容錯**（P0） | Excel UTF-8 BOM → 首欄解析失敗 | ✅ 完成（parse 前 strip EF BB BF；BOM 38→39） |
| W-5 | **錯誤訊息自癒**（P1） | 金額/科目錯誤不帶實際值 | ✅ 完成（validate_fields 帶收到值） |
| W-6 | **空態 CTA 化**（P1，Vercel 模式） | 4 空態補 CTA（前往新增交易/前往匯入/前往 AI 決策） | ✅ 完成（index.html） |
| W-7 | **漢堡抽屜導覽**（P1） | ≤900px ☰ 選單（checkbox hack） | ✅ 完成 |
| W-8 | **麵包屑**（P1） | studio→group 兩級＋頁 | ✅ 完成（nav.rs SectionLabel） |
| W-9 | **焦點環品牌化＋placeholder**（P1） | 玉山綠雙環；placeholder token 化 | ✅ 完成 |
| W-10 | **表格排序/日期範圍標準元件**（P1） | 無排序/日期元件 | 下輪：transactions/audit 加 :target+GET 排序 |
| W-11 | **命令面板 :target**（P2） | 無 ⌘K 近似 | 下輪：#command 面板 |
| W-12 | **首頁任務佇列**（P2） | KPI 優先 | 下輪：待放行 N 筆入口卡 |
| W-13 | **emoji → Lucide stroke 圖標**（P0，最高） | 10 emoji 在 manifest | 本輪：icons.rs＋manifest 改 key＋模板渲染 SVG |

### 維度 2：網頁結構與串聯

| # | 項目 | 狀態 |
|---|---|---|
| S-1 | 導覽 IA（home 置頂＋4 工作室 11 群組 66 頁） | ✅ |
| S-2 | RBAC 伺服器端閘（66 頁×4 角色） | ✅（W4 補齊 7 頁 403） |
| S-3 | 麵包屑（總覽 › 工作室 › 群組） | ✅ |
| S-4 | 深層連結（/transactions/:id 等模板） | ✅ 404 測試 |
| S-5 | 全站連結完整性 | ✅ 77+ 連結 |
| S-5 | 漢堡抽屜（窄窗 IA 不壓平） | ✅ |

### 維度 3：數據流

| # | 項目 | 狀態 |
|---|---|---|
| D-1 | 匯入閉環（範本→上傳→預覽→映射→確認→入帳） | ✅ 三格式 |
| D-2 | 採納閉環（草稿→過帳） | ✅（含千分位） |
| D-3 | RBAC 過濾（NavModel 資料層） | ✅ |
| D-4 | 稽核軌跡（IP/UA/使用者/事件） | ✅ |
| D-5 | 交易→報表→試算表一致 | ✅ E2E |
| D-6 | 搜尋索引（BM25＋向量，自愈 TTL） | ✅ |

### 維度 4：整體設計

| # | 項目 | 狀態 |
|---|---|---|
| V-1 | 設計 token（玉山綠、暗色優先、8 態掛點） | ✅ |
| V-2 | 響應式（漢堡抽屜、≤900px 降維） | ✅ |
| V-3 | 無障礙（focus-visible、aria-current、prefers-reduced-motion） | ✅ |
| V-4 | 圖標統一（Lucide） | **待做（本輪 P0）** |
| ADR-01 | 表格標準元件 | 下輪 |
| ADR-02 | 命令面板 :target | 下輪 |

## 4. 執行優先級與批次

### 批次 A（本輪必做）：UI-1 Lucide 圖標系統
- `platform/src/icons.rs`（新，path 表，零依賴）
- `crates/console_manifest/src/lib.rs`：10 個 NavGroup label 去 emoji → 加 `icon: &'static str` key（ledger/capture/bank/reports/query/governance/admin/api/system/knowledge）
- `platform/src/nav.rs`：NavGroupView 加 `icon: &'static str`（從 g.icon 取）
- `platform/templates/partials/studio_nav.html`：渲染 `<svg>`（由 icon key 映射）
- 防呆：`platform/tests/icons_guard.rs`（manifest 零 emoji、key 唯一、每個 NavGroup 有 icon）

### 批次 B（下輪）：UI-3 表格標準元件＋UI-7 命令面板＋UI-6 任務佇列

### 驗證程序（每輪）
1. cargo fmt/clippy/test（≥146）
2. 66 頁掃描＋連結完整性
5. 人工瀏覽器視覺清單（漢堡展開/圖標/空態 CTA/麵包屑/雙環）
5. 追蹤矩陣＋CHANGELOG 同步

## 5. 風險與約束

- 平行工作線：動工前重測 mtime/行數（C-05）；斷言式補丁（C-04）
- 編碼：UTF-8 無 BOM；禁 PS Get-Content/Set-Content 往返（C-09）
- 零 JS 憲政：CSS/GET/:target only
- 未驗證不宣稱（C-10/C-13）

## 6. 完成條件

批次 A（圖標）完成＋146 tests 綠＋視覺清單過＝本輪 PASS；B 批次列入下輪。未測項標 NOT TESTED。