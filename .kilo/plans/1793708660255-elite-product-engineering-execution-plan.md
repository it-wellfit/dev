# ELITE PRODUCT ENGINEERING MASTER EXECUTION PLAN

> 依據《ELITE PRODUCT ENGINEERING MASTER DIRECTIVE》v1.0 制定。狀態：**v3（2026-11-04）**——Slices 1–9 已完成（66 tests 全綠，git cd66388→40f46f9），本版新增 §15「Slice 10：多維洞察進階與方法論硬化」（市場與技術研究佐證＋TDD/BDD/SDD/DDD 貫穿）。驗證以 curl.exe／Rust 測試為主；Python 至多一次性小工具。

## 1. 執行上下文（Environment Reconnaissance 已完成）

- 工作空間：`C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev`；治理文件齊備（WORKSPACE_CHARTER、SHARED_STANDARDS、GOVERNANCE_STANDARD、PLAN_REGISTER Reviewed 2026-11-02）。
- 工具鏈：Rust 1.99.1、Git 2.56、gh 2.103（未驗證）；**Node.js/npm 未安裝**、codex CLI 未安裝。
- PLAN_REGISTER 判定：ARES 計畫為 proposal、ekpi-platform Phase A 未核准／Phase B blocked、Wellfit 證據為歷史快照、SyncMesh 為 prototype 且安全宣稱與代碼不符。

### 1.1 子專案現況

| 子專案 | 實際技術棧 | 狀態 |
|---|---|---|
| ARES（根 workspace：`core/`、`platform/`、`web/`、`crates/flaps_bridge`） | edition 2021、resolver 2、axum 0.7、tokio 1.0、sqlx 0.8 SQLite、tauri 2.0、windows 0.58 | Planning（未核准） |
| ekpi-platform | Rust 2024、resolver 3、Axum 0.8.9、Askama 0.16.1、SQLx 0.9.x、PostgreSQL 18、Argon2id、Strict Zero-JS、Clean Architecture | Phase A 未核准、Phase B blocked；T1–T10 完成 |
| Wellfit | Rust 7-crate workspace、E.Sun V2.8、無前端 | Draft；工作樹有未提交變更 |
| SyncMesh | Rust 4-crate、TCP-only（無 TLS） | Prototype；TLS/mutual-auth 宣稱不實 |
| oekap | 僅工作名稱與 `docs/product-solution.md` 藍圖 | 需裁定歸屬 |

### 1.2 記憶 vs 實際程式碼差異（首要裁定事項）

project.md 記憶宣稱 Rust 2024／Resolver 3／Axum 0.8／Askama 0.16／SQLx 0.9／PostgreSQL 18／Strict Zero-JS；**根 workspace 實際**為 edition 2021／resolver 2／axum 0.7／sqlx 0.8 SQLite／tauri 2.0，且 `web/index.html` 含約 500 行內嵌 JavaScript（**違反 Strict Zero-JS 與 WORKSPACE_CHARTER §2.3**）。後續所有技術決策以實際程式碼為基線；ekpi-platform 的 2024/PG18 棧僅在選定其為主產品後適用。

## 2. 能力盤點（Skill & Tool Discovery 已完成）

- **內建 Skills**：kilo-config（已用於組態定位）、agent-md-refactor（適用於建立根 AGENTS.md）、skill-creator（備用）、microsoft-docs/code-reference（無 .NET 需求，不載入）；其餘 Skills 與本工作無關，不載入。
- **.kilo 代理（primary mode）**：architect（本規劃角色）、code-reviewer（唯讀審查）、code-skeptic（批判驗證＋md 編輯）、test-engineer（測試檔編輯）。
- **MCP（kilo.jsonc 全部 enabled）**：sequentialthinking、playwright（E2E／視覺／可存取性）、crates-io（crate 版本與健康度）、rust-analyzer（工作區診斷）、narsil、firecrawl、context7（Axum/SQLx/Askama/Tauri/WCAG 官方文件）、grep-app、jina。
- **缺口**：無 Node.js（強化零 JS 純 HTML/CSS 路線）；無 Product Discovery 專屬 Skill（以文件模板＋code-skeptic 消歧替代）；workspace 根目錄無 AGENTS.md。

## 3. 現況品質評估（Baseline Assessment 摘要）

| 面向 | 已確認問題（證據） | 嚴重度 | 優先級 |
|---|---|---|---|
| UX/UI | `web/index.html` 內嵌 JS、未使用 `docs/design-system.css` Token、漸層／製式 AI Dashboard、無 Loading/Empty/Error 狀態設計 | High | High |
| 架構 | 記憶與實際技術棧不符（§1.2）；core/platform/web 無分層契約；platform 未依賴 axum（API 層缺失） | High | High |
| 安全 | SyncMesh TLS 宣稱不實（PLAN_REGISTER）；內嵌 JS 擴大 XSS 面；無 THREAT_MODEL | High | High |
| 治理 | 根目錄無 AGENTS.md；產品方向分散五個子專案無單一負責人 | High | High |
| 品質 | 根 workspace 無測試執行證據；無 cargo audit/deny；根 Cargo.toml clippy 設 `all="allow"`（違反憲章 §2.2 零警告不變量） | High | High |
| 資料 | SQLite 有 WAL/冪等設計（INV-03/04）但無版本化 migration 目錄、無備份還原演練證據 | Medium | Medium |
| 效能/維運 | 無效能預算與測量；CI 工作流存在但未驗證執行 | Medium | Medium |

**Top 問題**：P1 零 JS 違反、P2 技術棧版本不一致、P3 產品方向分散、P4 SyncMesh 虛假安全宣稱、P5 根 AGENTS.md 缺失、P6 品質證據缺失（含 clippy allow 與憲章矛盾）。

## 4. 產品方向（Product Blueprint 核心決策）

- **主產品**：ekpi-platform / OEKAP——唯一已具備 Clean Architecture＋Strict Zero-JS＋PostgreSQL 棧與 T1–T10 完成紀錄的候選；根 workspace（ARES）定位為 FlapsErp RPA 支援線（flaps_bridge 維持 feature-gated）。
- **願景**：證據導向企業知識與流程智慧平台（30 秒找到正確答案＋可執行動作＋完整稽核軌跡）；核心假設 H1 資訊取得成本、H2 流程斷裂、H3 AI 信任、H4 整合維護成本。
- **需求編號體系**：PRD-FR/NFR、UX-FLOW、API、DATA、SEC、TEST 前綴建立追溯矩陣。

### 核心驗收條件（Given/When/Then 摘錄）

| ID | 驗收條件 |
|---|---|
| FR-001 | Given 任意頁面 When 檢視原始碼 Then `<script>` 數 = 0（零 JS；HTMX/WASM 亦不允許） |
| FR-002 | Given 頁面渲染 When 檢查樣式 Then 色彩/間距/字體全來自 Design Token |
| FR-003 | Given 表單提交 When 成功 Then 302 Redirect（PRG）；GET 永不修改狀態 |
| FR-004 | Given 資料載入/空/失敗 Then 對應 Loading/Empty/Error(+復原動作)/Success 狀態 |
| FR-005 | Given Tab 導航 When 移動焦點 Then 焦點環可見、順序合理（WCAG 2.2 AA） |
| FR-006 | Given 所有 `.rs` When 檢查首行 Then `#![forbid(unsafe_code)]`；clippy `-D warnings` 退出碼 0 |
| FR-007 | Given `cargo test --locked` Then 0 failed、0 ignored |
| FR-008 | Given Schema 變更 When 部署 Then 經版本化 migration，乾淨庫可重放 |

## 5. 改善計畫

### 5.1 UI/UX（零 JS 重構，針對根 workspace `web/index.html`）

1. 移除全部 `<script>`（fetch/WebSocket/localStorage），改語義 HTML＋表單 POST/PRG；互動回饋以 CSS（`:hover/:focus-visible/:checked`、`prefers-reduced-motion`）達成。
2. 樣式全面改用 `docs/design-system.css` Token；移除漸層、無目的圓角卡片與玻璃擬態。
3. 補齊 Dashboard/Form/Table 六態（首次進入/有資料/無資料/載入中/載入失敗/部分不可用）。
4. WCAG 2.2 AA：鍵盤導航、可見焦點、aria-label、對比、非顏色單一傳達；Playwright 掃描＋人工鍵盤走查。
5. 產出 `docs/design/DESIGN_SYSTEM.md`、`docs/design/ACCESSIBILITY.md`；UX Writing 採繁中台灣情境，錯誤訊息含「問題＋原因＋下一步」。

### 5.2 後端／資料庫／安全

- 裁定 API 層（Axum 0.7 或維持無 API）並記錄 ADR；flaps_bridge `real-com` 維持 feature-gated（EULA 未決前不得啟用）。
- 建立 `migrations/` 版本化遷移（乾淨庫重放驗證）；恢復 clippy 零警告（移除根 `Cargo.toml` 的 `all="allow"`，逐項修復或以模組屬性限縮豁免）。
- `cargo audit`／`cargo deny`（如可用）記錄依賴風險；更正 SyncMesh 文件為「TCP-only，TLS 列為技術債」；建立 `docs/security/THREAT_MODEL.md`（STRIDE，含本機資料邊界與 ledger 模型）。

## 6. 路線圖（POC → MVP → Production）

| 階段 | 範圍 | 退出閘門 |
|---|---|---|
| POC | 零 JS 前端可行性（PRG＋CSS 狀態）＋技術棧基線驗證＋主產品裁定 | Gate 1：假設有證據、Proceed/Pivot/Stop 有結論 |
| MVP | OEKAP 核心垂直切片：搜尋（證據鏈）→ 詳情 → 動作 → 稽核，端到端零 JS；基礎安全（輸入驗證、fail-closed）＋測試 | Gate 2：核心流程可操作、驗收案例通過、UI/UX 實機檢查、文件同步 |
| Production | 環境分離、CI/CD 品質門檻、備份還原演練、OpenTelemetry 觀測、SLO、回滾 | Gate 3：安全/可存取性/效能審查通過、部署驗證、文件完整 |

## 7. 文件計畫（依需求裁剪，不製造空文件）

| 文件 | 路徑 | 狀態 |
|---|---|---|
| 根 AGENTS.md（執行入口，增量整合既有憲章，不覆蓋） | `AGENTS.md` | 新建 |
| PRD＋追溯矩陣 | `docs/product/PRD.md` | 新建 |
| 使用者旅程（含失敗情境與恢復） | `docs/product/USER_JOURNEYS.md` | 新建 |
| 設計系統／無障礙 | `docs/design/DESIGN_SYSTEM.md`、`docs/design/ACCESSIBILITY.md` | 新建 |
| ADR-0001 技術棧基線裁定 | `docs/adr/ADR-0001-tech-stack-baseline.md` | 新建 |
| 威脅模型 | `docs/security/THREAT_MODEL.md` | 新建 |
| 測試策略 | `docs/testing/TEST_STRATEGY.md` | 新建 |
| 技術研究（Verified Facts/Assumptions/Decisions/Open Questions） | `docs/research/TECHNOLOGY_RESEARCH.md` | 新建 |
| 技能盤點 | `docs/engineering/SKILL_INVENTORY.md` | 新建 |
| 改善 Backlog | `docs/product/IMPROVEMENT_BACKLOG.md` | 新建（納入 P1–P6） |

## 8. 風險與緩解

- **記憶污染決策**：一律以 `Cargo.toml`/程式碼為準；研究結論寫入 TECHNOLOGY_RESEARCH 並標記版本。
- **Scope 蔓延**：每切片以 FR-001~008 為門檻，未過不進下一片。
- **Windows 建置限制**：桌面路徑建置受阻（應用程式控制）→ 沿用 `build.ps1` 導向 `%LOCALAPPDATA%\Temp` target；PowerShell 串接用 `;`。
- **flaps_bridge 合規**：COM 真實呼叫需 ERP 廠商書面授權（RSK-E1），未決前僅 stub。

## 9. 實作任務清單（交給實作代理，依序執行）

1. **裁定開立決策**（§10 D-01~D-05；若用戶無異議即採建議值）。
2. **Slice 1 基線**：根 workspace 跑 `cargo fmt -- --check`、`cargo clippy --all-targets -- -D warnings`（先還原 allow）、`cargo test --locked`、`cargo audit`；記錄修訂版＋退出碼；寫 ADR-0001。
3. **Slice 2 零 JS 重構**：`web/index.html` 去 `<script>`、套 Token、補六態；以 Playwright 實機驗證（1280/768/375px）＋`<script>` 計數 = 0 證據。
4. **Slice 3 治理文件**：根 `AGENTS.md`（增量整合憲章，不覆蓋）、`PRD.md`、`USER_JOURNEYS.md`、`SKILL_INVENTORY.md`、`IMPROVEMENT_BACKLOG.md`。
5. **Slice 4 資料與安全**：migrations 目錄、THREAT_MODEL.md、SyncMesh 宣稱更正、測試策略文件。
6. **每片完成後**：code-reviewer 審查＋code-skeptic 複核（「show me the logs」），更新 CHANGELOG 與受影響文件（同一提交）。

## 10. 開立決策（Open Decisions，含建議值）

| # | 決策 | 建議 | 理由 |
|---|---|---|---|
| D-01 | 主產品 | ekpi-platform/OEKAP | 唯一具備完整規範與 T1–T10 基礎 |
| D-02 | 根 workspace 技術棧 | 維持 edition 2021/axum 0.7/SQLite，升級列 POC | 避免無收益遷移風險 |
| D-03 | 前端路線 | 純 HTML/CSS 零 JS＋PRG（Tauri 殼保留） | Node.js 缺失＋政策一致 |
| D-04 | `web/index.html` 範圍 | 先重構單頁，不新增路由 | 最小可逆 |
| D-05 | SyncMesh 宣稱 | 文件改「TCP-only」，TLS 列技術債 | 反模擬原則 |

## 11. 實作紀錄（Slices 1–7 已完成，2026-11-03～04）

| 提交 | 主題 | 測試證據 |
|---|---|---|
| `cd66388` | P0-2 版控初始化＋治理基線（此前已修復編譯、19 個 clippy、零 JS 靜態頁、reqwest 升級） | 34 tests / 基線四項 EXIT=0 |
| `8a79adf` | P0-1 整數貨幣、P1-1 零 JS API（Axum 0.7＋Askama＋PRG）、P1-5 借貸平衡強制、G-1 認證（Argon2id＋sessions＋CSRF） | 50 tests / 基線四項 EXIT=0 |
| `4e94d7b` | 文件證據閉環（BACKLOG/CHANGELOG/PLAN_REGISTER #3/#4） | git status clean |

Backlog 已解：P0-1、P0-2、P1-1～P1-8、P2-2。Open：P1-7 多實例遷移 note、P1-8 密碼重設流程、R-07 Secure cookie、P3 系列。

## 12. Slice 8：資料獲取與洞察層（本版新增，多層展開）

使用者需求：快速匯入、多維度表格的主動詢問結構化輸入、非結構化資訊取得、儀表板/報表升級、多維資訊呈現（含 3D 提案）。以下四層皆受既有不變量約束：零 JavaScript（憲章 §2.3）、複式記帳（INV-02）、整數分（INV-01）、RBAC＋CSRF＋audit（FR-006/ADR-0003）、AI 禁區（金額修改等不由 AI 決策）。

### 12.1 Layer A — 快速匯入管線（兩段式、可回復、可追溯）

- **A1 預檢（dry-run）**：`POST /import/preview`（multipart CSV/TSV ≤2MB，欄位：日期、借方科目、貸方科目、金額、幣別、摘要）→ 伺服器逐列以 `flaps_core::money` 與規則驗證 → 渲染**預檢報告**（合格/錯誤逐列明細＋原因）→ 批次暫存 `import_batches`（status=draft）＋原始檔 SHA-256，**不落地交易**。
- **A2 確認**：`POST /import/confirm`（csrf＋batch_id）→ 僅合格列逐筆登帳（冪等 UUID）→ audit 事件 `IMPORT_CONFIRMED`（含 batch_id、row 數）→ 303 報告頁；錯誤列保留報告可下載。
- **不變量**：預檢無副作用；確認可重放防護（batch status 機械擋重複 confirm）；全部動作 audit。
- **遷移 0003**：`import_batches`（batch_id PK、file_sha256、row_total、row_ok、status、created_by、created_at）＋`import_rows`（batch_id、row_no、raw_line、status、error_msg、draft_json）。

### 12.2 Layer B — 引導式結構化輸入（主動詢問精靈）

- **B1 多步精靈**：`GET /new?step=1|2|3`（伺服器端逐步驗證，狀態經 hidden 欄位攜帶）：步驟 1 選**交易模板**（現金支出/收入/轉帳/開帳——模板預設借貸方向與提示）→ 步驟 2 填借貸科目＋金額＋日期（欄位級規則提示：科目存在性候選、借貸須不同、日期不可未來）→ 步驟 3 確認（平衡預檢＋摘要卡）→ 提交至既有 `POST /transactions`。
- **B2 智慧預填**：精靈接受 `?from_draft=<draft_id>`（Layer C 產出）或 `?from_batch=<batch_id>`（匯入錯誤列修正入口）——每個預填值在頁面標註**規則來源**（可解釋）。
- 指標：可解釋（每個限制標註來源規則）、貼心（錯誤前置到輸入當步）、精準（沿用 money 模組）。

### 12.3 Layer C — 非結構化 → 結構化草稿（規則解析，100% 可推導）

- **C1 解析**：`POST /drafts/parse`（textarea ≤10KB）→ core 新模組 `flaps_core::parser`（規則表驅動：日期格式集、金額模式、科目關鍵字對映存 `erp_config`，管理員可增修）→ 產出**草稿分錄**＋**依據卡**（哪條規則命中哪段文字、confidence=rule）
- **C2 人工確認必經**：草稿頁（借貸/金額/日期可改）→ 確認才登帳；規則未命中時 AI（Ollama，decision_gating）可**建議**草稿——信心 <0.7 一律人工覆核，AI 永不直接落地（H3 信任模型＋禁區不變）。
- 指標：可解釋（依據卡）、可推導（同輸入同輸出）、安全（建議與落地分離）。

### 12.4 Layer D — 洞察層：零 JS 圖表與報表（含 3D 憲政裁定）

- **D0 憲政裁定（重要）**：互動式 3D 拖拉需要 JS/WebGL，與 Strict Zero-JS（憲章 §2.3——HTMX/WASM 亦禁）**直接衝突，不得靜默違反**。零 JS 內的多維洞察替代方案（按價值排序）：
  1. 伺服器渲染 **SVG 圖表**（長條/折線/堆疊；維度=期間×科目×借貸，經查詢參數切換；含 `<title>/<desc>` 無障礙）。
  2. **:target 維度切換**＋排序參數（零 JS 互動）。
  3. **CSV 匯出**（`GET /reports/balance.csv` 等，`Content-Disposition: attachment`，RBAC=LEDGER_READ）——資料出場給 Excel/BI/3D 工具，而非 JS 進場。
  4. 列印友好月報（`@media print`）。
  - 若未來確需互動 3D：另立「分析工作台」獨立模組＋**憲政豁免 ADR**（本計畫 Open Question OQ-8-1，需擁有者裁定）。
- **D1 儀表板升級**：KPI 下方加 SVG 月度收支圖＋近 30 天趨勢（伺服器端聚合 SQL，走既有索引）。
- **D2 月報**：`GET /reports/monthly?ym=YYYY-MM`——科目餘額表（借/貸兩欄＋淨額）、列印 CSS；`GET /reports/monthly.csv` 匯出。
- 指標：效率（10k 交易報表 P95 < 500ms）、易讀（語義表格＋Token）、高吞吐（聚合走 idx_transactions_date）。

### 12.5 驗收條件（Given/When/Then，節錄）

| ID | 驗收條件 |
|---|---|
| IMP-001 | Given 合法 CSV 100 列 When preview Then 報告 100 OK 且 transactions 表零新增 |
| IMP-002 | Given 90 好 10 壞列 When confirm Then 僅 90 筆登帳＋audit IMPORT_CONFIRMED，錯誤報告列 10 筆原因 |
| IMP-003 | Given 同 batch_id 重複 confirm Then 第二次被拒（status 機械擋） |
| WIZ-001 | Given 步驟 2 缺貸方科目 When 下一步 Then 停留本步並欄位級標示錯誤 |
| DRAFT-001 | Given 文字「2026-11-04 支付租金 12,000 元 現金→租金費用」When parse Then 草稿含日期/金額 1,200,000 minor/借租金費用貸現金＋依據卡標示命中規則 |
| RPT-001 | Given 10k 交易 When 月報 Then P95 < 500ms 且 CSV 欄位完整 |
| ZJS-001 | Given 全部新頁面 When 檢視原始碼 Then `<script>` 計數 = 0 |

### 12.6 關鍵指標對映

精準=money 模組；效率=報表 P95/預檢吞吐；穩定=兩段式冪等；安全=RBAC+CSRF+AI 禁區；高吞吐=聚合 SQL+索引；高擴展=規則表(erp_config)驅動；可靠/韌性=預檢無副作用＋可回復草稿；可追溯=batch_id+SHA-256+audit；可查證=測試證據；可解釋=依據卡；可推導=規則表驅動解析。

## 13. Slice 8 執行任務清單（交給實作代理，依序）

1. **S8-1 遷移 0003**：import_batches/import_rows＋重放測試。
2. **S8-2 匯入管線**：/import/preview＋/import/confirm＋預檢報告頁＋測試（IMP-001~003）。
3. **S8-3 精靈**：/new 三步＋模板規則＋預填（from_draft/from_batch）＋測試（WIZ-001）。
4. **S8-4 解析器**：`flaps_core::parser`（規則表＋依據卡）＋/drafts/parse＋草稿確認頁＋測試（DRAFT-001）。
5. **S8-5 SVG 圖表**：dashboard 月度收支 SVG＋維度參數切換＋測試（ZJS-001）。
6. **S8-6 報表與 CSV**：/reports/monthly(+.csv)＋print CSS＋測試（RPT-001）。
7. **S8-7 收尾**：文檔同步（PRD/CHANGELOG/PLAN_REGISTER #5）＋基線四項＋煙霧（curl.exe）。

## 14. Slice 8 開立問題

| # | 問題 | 建議 |
|---|---|---|
| OQ-8-1 | 互動式 3D 是否憲政豁免？ | 先以零 JS SVG＋CSV 出場交付；豁免另立 ADR |
| OQ-8-2 | Ollama 建議何時介接？ | 規則解析器上線且 C1 覆蓋率量測後（先規則後 AI） |
| OQ-8-3 | 匯入單檔上限？ | 2MB/5,000 列起，量測後調整 |

## 15. Slice 10：多維洞察進階與方法論硬化（v3 新增）

研究佐證（2026-11-04）：市場面——2026 會計軟體核心需求為匯入、AI 分類（46% 會計師日用 AI）、月結報表、稽核軌跡、多幣別（Float/Booke AI/Articsledge 市場調查）；技術面——平行座標為 Apache ECharts/Inselberg 認可的多維標準視覺化（Fortune 500 財務分析實用），**伺服器渲染 SVG 可在零 JS 內達成同等多維分析**，優於互動 3D（OQ-8-1 維持擱置）；生態面——ollama-rs 0.3.6 為主流 Rust 用戶端，但 platform 已有自建 `src/ai/`（reqwest＋decision_gating），**復用優於新增依賴**。

方法論貫穿（本切片起强制）：**TDD**——每個驗收條件先寫失敗測試（red→green→refactor，提交含測試先行證據）；**BDD**——`docs/quality/bdd_acceptance.md` 以 Feature/Scenario 對映整合測試名（不引入 cucumber 依賴）；**SDD**——規格 ID（S10-FR-x）→測試→模組追溯矩陣；**DDD**——core 新增值物件 `Money(i64 minor, Currency)` 與 `AccountId` newtype（建構即驗證），統一語言詞彙表入 `docs/quality/ubiquitous_language.md`。

### 15.1 規格表（S10-FR，含驗收）

| ID | 規格 | 驗收（Given/When/Then） |
|---|---|---|
| S10-FR-1（Layer E 樞紐） | `GET /reports/pivot?rows=account|month&cols=month|account|direction&measure=debit|credit|net&ym=YYYY-MM`——維度**白名單映射**（SHARED_STANDARDS 動態欄位規則），伺服器渲染矩陣表＋合計行列 | PIV-001：Given 已知種子 When 3×3 樞紐 Then 與手算一致；PIV-002：Given 非法維度 When 請求 Then 400＋規則名稱 |
| S10-FR-2（Layer F 平行座標） | `GET /reports/parallel?months=6&top=10`——每科目一條 polyline，軸＝近 N 月正規化（**整數 min-max**，零浮點），SVG 含 title/desc/逐線 title | PAR-001：每帳戶一線＋a11y 標籤；PAR-002：頁面 `<script>`=0 |
| S10-FR-3（Layer G AI 建議） | `POST /drafts/ai_suggest`（textarea）——特性旗標 erp_config(category='ai', key='suggest_enabled', 預設 '0')；呼叫**既有** `src/ai/` 模組（Ollama，5s timeout）；輸出 JSON schema 驗證（date/debit/credit/amount/currency）；**一律渲染為建議卡（source=AI 需人工覆核）**，使用者採用才經規則路徑建草稿——**永不自動落地**；Ollama 離線/逾時 → 降級訊息（韌性） | AIS-001：旗標關閉 → 503＋說明；AIS-002：建議永不自動落地（測試斷言 drafts 無 AI 直接寫入）；AIS-003：schema 驗證拒絕畸形輸出 |
| S10-FR-4（Layer H builder） | `Transaction::new`/`TransactionItem::new` 重構為 Builder（P2-1），移除兩處 `#[allow(too_many_arguments)]` | H-001：clippy `-D warnings` 全綠且 allow 歸零；既有 66 測試不改語義全過 |
| S10-FR-5（Layer H 量測） | `GET /admin/parse_coverage`（ROLE_MANAGE）——近 N 草稿的規則命中率（is_complete 比例），作為 OQ-8-2 決策數據 | COV-001：N 筆草稿 → 命中率正確計算 |

### 15.2 標準對映

ISO/IEC 25010：效能效率（PIV/PAR/報表預算測試）、易用性（精靈＋依據卡）、可靠性（重放防護）；WCAG 2.2：SVG title/desc、鍵盤、對比；OWASP：白名單 SQL、CSRF、速率限制、會話；DORA：測試覆蓋與提交粒度持續追蹤。

### 15.3 TDD 執行順序（每任務）

1. 依驗收條件寫失敗整合測試（red，`cargo test` 錯誤輸出存 session log）→ 2. 最小實作（green）→ 3. 重構＋clippy/fmt（refactor）→ 4. 提交（單一提交含測試與實作，訊息 `test:`/`feat:` 對應）。

### 15.4 任務清單（交給實作代理，依序）

1. **S10-1**：PIV-001/002 失敗測試 → 樞紐 handler＋白名單映射 → 綠。
2. **S10-2**：PAR-001/002 失敗測試 → 平行座標 SVG 產生器（core 純函式＋整數正規化）→ 綠。
3. **S10-3**：AIS-001~003 失敗測試 → 旗標＋Ollama 呼叫（既有 ai 模組）＋schema 驗證＋建議卡 → 綠。
4. **S10-4**：H-001 失敗（clippy 對新 builder 仍報 too_many_arguments 的反向驗證）→ Builder 重構 → allow 歸零 → 綠。
5. **S10-5**：COV-001 失敗測試 → parse_coverage 端點 → 綠。
6. **S10-6**：BDD 場景文件＋追溯矩陣＋統一語言詞彙表。
7. **S10-7**：基線四項＋煙霧（curl.exe）＋CHANGELOG/PLAN_REGISTER #7 提交。

### 15.5 Slice 10 開立問題

| # | 問題 | 建議 |
|---|---|---|
| OQ-10-1 | AI 建議的模型與提示詞治理（版本追蹤、離線行為） | 模型名入 erp_config；提示詞入版控常數；回應留存 audit |
| OQ-10-2 | 樞紐資料上限（帳戶×月份矩陣爆炸） | 預設 top-20 帳戶＋單月，量測後調整 |
| OQ-10-3 | 多實例速率限制遷移（P1-7 notes） | 維持 in-process 至多實例部署需求出現 |
