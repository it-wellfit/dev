# UI 迭代計畫 v2 — 體驗契約 × 一流產品化（P0–P2）

> 日期：2026-11-13｜狀態：Implementation-Ready（切換可編輯代理執行）
> **⚠️ 執行順序（2026-11-13 更新）：W11 緊急回滾必須第一個執行**——使用者實測所有頁面主體被壓成 ~60px 垂直細條、每字一行（漢堡抽屜 CSS 副作用，畫面崩壞＝P0）。W1–W10 在 W11 驗證通過前**暫停**。
> 依據：`docs/standards/EXPERIENCE_CONTRACTS.md`、`docs/design/UI_DESIGN_DECONSTRUCTION_v1.0.md`、`docs/standards/NAMING_AND_BOUNDARY_DECREE.md`、`docs/standards/REQUIREMENTS_TRACEABILITY.md`

## 0. 已交付基線（2026-11-13 實證）

| 項目 | 證據 |
|---|---|
| 漢堡抽屜（≤900px checkbox hack）、焦點雙環（品牌玉山綠）、::placeholder token | ui-v2.css＋studio_nav.html；CSS 實測 |
| W1 BOM 容錯、W2 錯誤訊息帶值、W3 零筆 PRG notice | w123_regression 4/4；BOM 檔 38→39 實證 |
| W4 元件契約、W5 模組流程 E2E、W6 console/help、W7 格式指引、W8 映射樣本、W9 空態 CTA、W10 搜尋 p95 | `.kilo/plans/1793782615513-full-module-completion-audit.md` |
| trace_guard（文件↔程式碼）、module_flows、web_crawl | 146 tests / 0 failed |

**本計畫＝UI 體驗深化層**（P0 圖標/空態 → P1 表格/麵包屑/源清理 → P2 佇列/命令面板）。

## 1. 量測基準（2026-11-13 證據）

| 項目 | 實測 | 缺口 |
|---|---|---|
| 圖標 | NAV_GROUPS label 內嵌 emoji（`crates/console_manifest/src/lib.rs:162/169` 等 11 群組）；STUDIOS 純文字（lib.rs:132-149） | 跨平台渲染不一致、無 currentColor、無法調線寬 |
| 空態 | `.empty-state`＋`<strong>尚無…` 無 CTA（`platform/templates/index.html:209/444/499/541`） | 流程終點而非入口 |
| 表格 | ~50 個模板裸 `<table>`（transactions.html:50、audit.html:56 等） | 無排序/日期標準元件 |
| 麵包屑 | 全庫零 `breadcrumb`（grep 實證） | 深層無逃生通道 |
| 樣式債 | `web/assets/design-system.css:632` 硬編碼 `rgba(37,99,235,0.2)`（ui-v2 已覆蓋但源頭未清） | 未來視覺回歸風險 |
| 搜尋 | `platform/src/workbench.rs:40-74`（q 參數、200 字上限、top50） | 命令面板骨架可掛 `:target` |
| 導航 | `platform/src/nav.rs:48` NavModel{home,top,studios} | 麵包屑需 group 層資料 |

## 2. UI-1（P0）Lucide stroke 圖標系統（SSOT）

- **範圍**：`crates/console_manifest/src/lib.rs`（NAV_GROUPS 11 label 的 emoji）→ 圖標 key；新增 `platform/src/icons.rs`（path 表，零依賴）；`platform/templates/partials/studio_nav.html` 渲染；index 品牌區
- **做法**：`NavGroup` 加 `icon: &'static str`（key）；`icons.rs` 映射 key→SVG path（24×24、stroke 1.5、fill none）；NavGroupView/StudioView 攜帶 key；Askama 渲染 `<svg …><path d=…></svg>`，`aria-hidden="true"`，顏色 `currentColor`
- **契約**：key 封閉集合、path 非空、模板零 emoji 殘留
- **測試**：`icons_contract`（key 唯一/path 非空/每群組有 key）＋`zero_emoji_in_nav`（模板 grep）
- **驗證**：cargo test＋66 頁掃描＋人工視覺
- **風險**：Askama 對 `&'static str` match 限制 → 退回模板內聯 SVG include（已驗證可行）

## 3. UI-2（P0）空態 CTA 化（Vercel 模式）

- **範圍**：index.html:209/444/499/541 及 transactions/accounts/drafts/audit/bankpay/search 空態
- **做法**：`.empty-state` 補 `<p class="empty-cta">`＝原因＋下一步＋**真實 CTA 連結**（建立交易/建立科目/前往匯入/清除篩選）；`.empty-cta .btn` token 樣式
- **測試**：`empty_state_cta`（每空態模板含 `.empty-cta`）；空 DB fixture 断言 CTA 可達
- **驗證**：單元＋curl（清空資料 fixture）

## 4. UI-3（P1）表格標準元件（`:target`＋GET 白名單排序）

- **範圍**：transactions.html（排序＋日期範圍）、audit.html（日期範圍）
- **做法**：表頭 `<a href="?sort=amount&dir=desc">` GET 連結；handler 白名單 `ORDER BY`；日期 GET 表單；`.data-table` 樣式（aria-sort 樣式箭頭）
- **契約**：排序/篩選狀態全在 URL（可分享/回退）；非法欄位忽略（fail-safe）
- **測試**：`table_sort_contract`（非法 sort→忽略；排序後首列符合；URL 往返）
- **驗證**：Rust 測試＋curl

## 5. UI-4（P1）麵包屑

- **改**：`nav.rs` NavModel 加 `section: Option<(studio_label, group_label)>`；`studio_nav.html` 頂部渲染「總覽 / 工作室 / 群組 / 頁」（最後項 aria-current）
- **測試**：`breadcrumb_contract`（每頁渲染、三級可點、位置正確）
- **驗證**：單元＋人工

## 6. UI-5（P1）樣式債清理

- **改**：`design-system.css:632` 刪除 `rgba(37,99,235,0.2)` glow（ui-v2 雙環已接管）
- **驗證**：grep `rgba(37, 99, 235`＝0；design_a11y 綠

## 7. UI-6（P2）首頁任務佇列

- **改**：index.html top 區「待放行 N｜待覆核 M｜草稿 K」佇列卡（DB 計數同源 `/api/health`）；KPI 退居次要；佇列卡點入 `/transactions?status=PENDING_REVIEW`
- **測試**：`task_queue_counts`（DB vs 頁面數字一致）
- **驗證**：單元＋curl

## 8. UI-7（P2）`:target` 命令面板（零 JS ⌘K）

- **改**：`/search#command` 全屏 `:target` 覆層（搜尋表單 GET 即搜尋）；側欄/頂欄「搜尋」入口；URL 即狀態、瀏覽器返回＝關閉面板
- **測試**：`command_palette_target`（`:target` 覆層契約、無 JS）
- **驗證**：curl＋人工

## 5. 驗證總計劃（每輪必跑）

1. `cargo fmt --check`；`cargo clippy -D warnings`；`cargo test --locked -p platform`（≥150 含新契約測試）
2. 全 66 頁掃描＋連結完整性（`internal_links_all_resolve`、`web_crawl`）
3. **人工瀏覽器視覺清單**（永久缺口，無自動替代）：窄窗漢堡展開、圖標一致性、空態 CTA、麵包屑、雙環焦點、佇列卡
4. 需求追蹤矩陣＋CHANGELOG 同批同步

## 6. 風險與約束（吸取教訓）

- 零 JS 憲政：CSS/GET/`:target` only；樂觀 UI/延遲 ⌘K 需 JS → 不引入
- 視覺回歸無自動化（已知缺口）：改動後人工驗證（§5.3）
- 平行工作線：動工前重測 mtime/行數（C-05）；斷言式補丁（C-04：oldString 必須存在且唯一）
- 編碼：UTF-8 無 BOM；禁 PS Get-Content/Set-Content 往返（C-09）；量測須多方法交叉驗證（C-08：本輪 Contains 誤判 4 次皆 PS 轉義所致）
- 未驗證不宣稱（C-10/C-13）；UNKNOWN ≠ PASS

## 7. 範圍外

- JS 框架/Islands 架構（違憲）
- 亮色主題完整重設計（單調改進；token override 已具）
- 後端 API 結構變更（僅 handler 白名單小改）

## 9. W11（P0 緊急）：響應式漢堡回滾＋選擇器範圍修復（使用者實測畫面崩壞）

- **症狀（2026-11-13 截圖實證）**：頂部晶片列正常滿版，但頁面主體（您的角色：ADMIN、工作總覽面板）被壓成 **~60px 垂直細條、每字一行、max-height 70vh 可捲動**。另：**元件內容少很多**（≤900px 時 `.nav-studio-header` 被隱藏、側欄壓成 chips）。
- **根因（由症狀特徵反推，高置信）**：我加的選擇器 `.nav-toggle:checked ~ .container { display:flex; flex-direction:column; max-height:70vh; overflow-y:auto }` **未加 `.site-nav` 範圍**——`~` 後位兄弟選擇器會命中**任何**位於 checkbox 之後的同層 `.container`。細條內容（角色徽章＋工作總覽面板）證明被命中的是**頁面級容器**而非 nav 的容器；flex column＋無寬度約束＋CJK 逐字換行＝60px 直條。
- **執行序（精確）**：
  1. **DOM 舉證（修復前）**：curl `/status` 與 `/`，dump `body` 直接子元素、checkbox 父層、被命中的 `.container` 所在層級（證據存檔；確認 `~` 跨層命中的確切 DOM 條件）
  2. **範圍化選擇器（首選，保留漢堡）**：所有漢堡規則加 `.site-nav` 前綴——`.site-nav .nav-toggle:checked ~ .container`、`.site-nav .container { display:none }` 等（共 3 條）；頁面級容器即不可能命中
  5. **驗證**：重建→三寬度（375/800/1440）curl＋人工——頁面主體滿版正常、抽屜僅在 nav 內展開、**工作室標頭顯示**
  5. **後備回滾**：若範圍化後仍崩壞 → 整段移除 ≤900px 漢堡區塊，還原先前的單欄堆疊版本；**同時移除 ≤900px 的 `.nav-studio-header { display: none }`**（修復「元件內容少很多」——工作室標頭必須顯示）；checkbox/label 基底規則本為 `display:none`，回滾後自動隱藏
- **新增防呆**：`component_contract` 測試加兩條——(a) 漢堡選擇器必須以 `.site-nav` 開頭（防未來再寫出未範圍化的全域規則）(b) ≤900px 區塊不得含 `.nav-studio-header { display: none }`（IA 三層不壓平）
- **DOM 舉證步驟（修復前）**：curl `/status` 與 `/`，dump `body` 直接子元素與 checkbox 父層，確認 `.container` 與 checkbox 的實際兄弟關係（證據存檔）
- **W11b 追加診斷（2026-11-14 截圖）**：崩壞畫面＝`/status#status-panel`（:target 面板展開）。CSS 合併已驗證正確（伺服 77,419B = ds 61,378 + ui-v2 15,571；status-bar static、漢堡、麵包屑、雙環、.ico 全在、順序正確）→ **崩壞源頭縮小至「狀態詳情面板層」**：`.status-panel:target { display: block }` 展開時，面板/角色徽章被壓成中央直條（推導：面板或其父在 block 流中缺寬度約束，或 app-header `overflow:hidden`＋內部 flex 崩潰）。修復步驟：(a) dump `/status#status-panel` 的面板 DOM 與 `.status-panel` 相關全部規則 (b) 給面板 `width: 100%`＋`max-width` 約束 (c) 驗證角色徽章不在直條內 (d) 三寬度實測。

## 8. 完成條件（更新）

W11 回滾**先於一切**（畫面崩壞＝P0）；完成後回歸 W1–W3 全綠＋三寬度人工驗證，才可續行 W4–W10。

UI-1/2（P0）＋UI-5 全 PASS；UI-3/4 主流程完成；UI-6/7 骨架完成；測試 ≥150＋新契約測試全綠；人工視覺清單全過；文檔同步。未測項標 NOT TESTED。