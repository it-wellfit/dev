# AuxBridge 完整工具包與最佳 UI/UX 體驗總計畫（C→B→A × 體驗室 UX-E）

> 計畫編號：1793708660255（沿用主計畫檔）｜版本：**v2.2**｜日期：2026-11-13｜狀態：**implementation-ready（待使用者審閱；§15 待裁定點未回覆時依建議執行並記錄理由）**
>
> **v2.1 refine 重點（回應使用者 2026-11-13 指令）**：
> 1. 把「全端 UI/UX 觀念」定義為 **10 條體驗契約（EC-1..EC-10）× 9 個表面（S1–S9）** 的驗證矩陣，不是「多做幾頁」。
> 2. 把「拓展所有可能、處理所有問題」變成 **問題來源清單 → 分級 → 衍生細節五問 → 覆蓋證明** 的可稽核流程。
> 3. 把「完整工具包＋最佳文件」變成 **T1–T9 工件 + 文件質量 rubric + 工具組體驗契約（TC-1..TC-5；與工程互動 TT-1..TT-8 分名）**。
> 4. 把「可追溯、可查證、可解釋、可推導、可驗證」變成 **REQ→DEC→IMPL→TEST→EVID 追溯矩陣 + 自動校驗測試**。
> 5. **v2.2 再深化**：全端介面鏈 L1–L6、失敗/邊界情境清單、UI 細節檢查表、文件清單與驗收、推導鏈範例——把「所有情境／所有細節」化為可逐項打勾的清單。
>
> **檔案沿革**：v1.0＝「零 JS 營運控制台 27+ 頁收斂計畫」（未竟項已由 `1793033568961-unified-ops-console-manifest` 吸收並執行）；v2.0＝C→B→A＋UX-E 總計畫；本版 v2.1 為深度 refine。v1.0 原文保留於附錄 A（依 C-01 不刪除）；v2.0→v2.1 修訂見 §16。
>
> **使用者指令（2026-11-12／11-13，效力最高）**：
> 1. 主軸順序必須走 **C → B → A**（C＝覆核軸、B＝經辦軸、A＝整合軸）。
> 2. 工具包不精緻、缺全端介面 UI/UX 觀念——必須再深度優化，產出**最佳 UI/UX 產品體驗（體驗室）**。
> 3. 拓展所有可能與實際處理所有問題——專業實務之所有情境、大大小小問題與衍生細節必須最佳化處理完善；交付＝**完整工具包＋最佳文件**；禁脫鉤、禁背離所有目標與條件及需求。
> 4. 杜絕所有錯誤與幻覺——**可追溯、可查證、可解釋、可推導、可驗證完全**。
>
> **效力與上位**：WORKSPACE_CHARTER ＞ GOVERNANCE_STANDARD ＞ SHARED_STANDARDS ＞ ADR ＞ 本檔。運作制度＝《MASTER PROMPT：Evidence-Driven Global Engineering…》（`1794365729846`／C-13）：迭代交付 A–L 節、發布判定五級（PASS／CONDITIONAL PASS／BLOCKED／REJECTED／NOT VERIFIED）。
> **上位計畫（不重複，必先讀）**：`1793033761021`（覆核 Brief＋D-1..D-7）｜`1793782615513-ui-v2-sidebar-redesign`（7/7 裁決＋R0–R9 協議）｜`1793033568961`（manifest SSOT＋W1–W10）｜`1793034142477` §3.5（非協商約束）｜`1792865422970` §26/§27。
> **相鄰線（協作不重疊）**：`1793033952894` 電商 OMS v4（T0=2026-11-17）；FlapsErp 原廠優化線（`docs/04-operations/`）。
> **標籤消歧**：本檔 Phase C／B／A＝三軸；ultimate 計畫之任務 B-1..B-8 一律寫作 **uB-1..uB-8**。

---

## §0 指令對照（mandate traceability；每條指令都有落點與驗證）

| 使用者指令（2026-11-13） | 對應章節 | 驗證 |
|---|---|---|
| 精煉工具包、UI/UX、全端介面 | §2.1、§2.5、§3、§8、§9.1 | E-1..E-8；T1–T9；TC-1..5 |
| 所有可能情境 | §5（角色×任務×情境＋失敗/邊界） | 情境矩陣；P0 全測＋pairwise ≥95% |
| 所有問題與細節 | §6（來源/分級/五問/覆蓋/細節表） | 問題登記；覆蓋率；零空格 |
| 完整工具包＋最佳文件 | §9、§13 | T1–T9 質量門；文件 rubric 六維 |
| 不偏離目標/條件/需求 | §2.4、§10、§15 | 追溯矩陣；N/A 需理由；五級判定 |
| 可追溯/可查證/可解釋/可推導/可驗證 | §10 | REQ→DEC→IMPL→TEST→EVID；自動校驗 |

---

## §1 現況基線（2026-11-12 複測；本版未重測，實作前 W0 必須複測）

| # | 事實 | 證據 |
|---|---|---|
| 1 | 建置與測試綠：`cargo check -p platform` EXIT=0；`cargo test -p platform`（隔離 target）全整合套件 0 failed＋lib 146 passed | 本線驗證代理 2026-11-12 實錄 |
| 2 | 建置鎖：dev-target 有 orphan `auxbridge-server`（PID 25016、127.0.0.1:8095）持有 debug exe → 建置前釋放 | 進程表；PLAN_REGISTER #31 |
| 3 | 版控：HEAD=`bb4f10d`；11-10 以來 14 提交；工作樹 93 項未收斂 → 每切片重測（C-05） | git log／status |
| 4 | 品牌：AuxBridge 輔翼（`auxbridge-server`／`auxbridge_core`／`AUXBRIDGE_*`／`auxbridge.db`） | NAMING_AND_BOUNDARY_DECREE；naming_guard |
| 5 | 進料門戶全格式：CSV/XLSX/PDF/OCR（PP-OCRv4）/EML/DOCX/映射 profile | PLAN_REGISTER #27/#28/#30 |
| 6 | 覆核軸：uB-1 量測（M-1 機器下限 11ms）、uB-5.0 `/my-decisions`、uB-5.1 二段確認、uB-5.2 AA 修復、P0 批已落地；未竟：uB-5.1 筆數預覽、uB-5.3、uB-6、uB-7、A-1/A-4/E-1 固化、uB-8、Q9、D-7(a) | `1793033761021` §12–§17 |
| 7 | 體驗層：`ui-v2.css`（451 行，dark-first＋玉山綠 token）已由 `api.rs:683 include_str!` 串接；**側欄未落地**（模板零 `sidebar`）；逐頁 QA 7 區＋R0–R9 未見完成證據 | api.rs:683；grep；`partials/studio_nav.html` |
| 8 | `states.html`（239 行）已定義八態：`empty/zero/loading/partial/stale/denied/error/conflict`，含按鈕／表單／徽章／錯誤／降級示範 | `platform/templates/states.html` |
| 9 | manifest 契約欄位：`PageSpec{id,path,method,title,nav_group,nav_href,renderer,tier,gate,workflows}`；`Tier=Operator/Admin/Technical`；`Gate=Public/All/Read/Write/Manage` | `crates/console_manifest/src/lib.rs` |
| 10 | guards 已掃 8 個目錄：編碼損傷、`<script`=0、`style=`=0、console 零 JS／零內聯、token canonical、unsafe=0 | `platform/tests/guards.rs` |
| 11 | 問題來源已存在：`IMPROVEMENT_BACKLOG`（P0–P3）、`GAPS_AND_RISKS`（GAP-01..08＋風險矩陣）、`MUTATION_SCORE_REPORT`、`NEXT_ACTIONS` | 各檔 |
| 12 | `docs/design/` 現有 `INFORMATION_ARCHITECTURE.md`、`UI_DESIGN_DECONSTRUCTION_v1.0.md`；**無 EXPERIENCE_SPEC／體驗矩陣** | 目錄實測 |
| 13 | 相鄰線：OMS v4 Native-First（T0=11-17；S4 三方差異報表＋人工複核佇列）；FlapsErp 原廠優化線 | `1793033952894` v4 |

---

## §2 名詞與範圍裁定（消歧；禁模糊）

### 2.1 「工具包」的定義（T1–T9）

| ID | 工件 | 內容 | 質量門 |
|---|---|---|---|
| T1 | 平台本體 | `auxbridge-server`＋`web/`（動態頁、零 JS） | 可啟動、四關綠、頁面矩陣綠 |
| T2 | 桌面殼 | Tauri v0.2、NSIS、updater（unsigned=test-only） | 安裝/啟動/更新/錯誤四態可驗證 |
| T3 | KEM 控制台 | `/console/*` 靜態頁（65 檔） | 受保護、零 JS、與平台一致 |
| T4 | 進料門戶 | CSV/XLSX/PDF/OCR/EML/DOCX/映射 | 全格式端到端測試綠 |
| T5 | 自動化接線 | `wellfit` CLI/crates＋`flaps_bridge` | 冪等、重試、審計鏈可驗證 |
| T6 | 演示資料包 | seed 腳本、demo DB、操作手冊 | 可重現、可還原、禁生產 |
| T7 | 驗證包 | 測試、證據 log、追溯矩陣 | 每需求有 TEST＋EVID |
| T8 | 運維工具 | 備份 CLI、效能探針、健康檢查 | 破壞性預設關閉、輸出可讀 |
| T9 | 文件包 | docs＋runbooks＋release notes＋決策 log | 受眾標籤、零死鏈、可執行 |

### 2.2 「全端」的定義（S1–S9；不是「所有頁面」）

> 全端＝**同一體驗契約在多個表面的實現一致性**，加上每個表面依其媒介的最佳化。不是把同一頁縮放到處。

| ID | 表面 | 載體／限制 | 主要輸入 | 體驗重點 |
|---|---|---|---|---|
| S1 | Web 工作台 | 瀏覽器、≤900px 折疊、無 JS | 滑鼠／鍵盤／觸控 | 密度、掃視、鍵盤流、八態 |
| S2 | 桌面殼 | Tauri WebView2、視窗、離線 | 滑鼠／鍵盤 | 首啟、視窗標題、更新、錯誤 |
| S3 | KEM 控制台 | 靜態 HTML、無伺服器互動 | 滑鼠／鍵盤 | 導覽一致、可讀、可列印 |
| S4 | 列印 | 紙張／PDF、黑白 | 無 | 頁首頁尾、分頁、對比、可讀 |
| S5 | 匯出 | CSV／JSON、Excel 可開 | 無 | 命名、欄序、編碼、穩定 |
| S6 | 文件體系 | Markdown／HTML、GitHub/本地 | 滑鼠／鍵盤 | 受眾分層、可搜尋、零死鏈 |
| S7 | API／DX | JSON、HTTP 狀態、錯誤碼 | 程式 | 一致封套、可預測、可測試 |
| S8 | 通知／警報 | Email／Teams、批次管線 | 無 | 事件語義、嚴重度、可行動 |
| S9 | 工具組 CLI | 終端機、管線、CI | 鍵盤 | help、退出碼、JSON、無破壞預設 |

**裝置矩陣（S1 必測）**：desktop ≥1280、laptop 1024、tablet 768、phone 390；輸入：滑鼠／鍵盤／觸控；輔助：螢幕閱讀器、reduced-motion、200% 縮放、高對比。

### 2.3 角色／人物映射

| 平台角色（程式碼） | 產品人物 | 主要任務 |
|---|---|---|
| visitor | 訪客 | 登入、錯誤頁 |
| store | P1 一線經辦（門市/營運） | 匯入、草稿、登帳、查詢、對帳、付款 |
| finance | P1 經辦＋P2 覆核 | 批量裁決、例外、報表 |
| admin | P3 IT 管理員 | 帳號、角色、會話、系統、API |
| （技術） | P3 技術人員 | API、日誌、設定、部署、體驗室 |
| （稽核） | P5 外部/內部稽核 | 唯讀軌跡、證據匯出 |

> 角色閘以程式碼 `NavFlags`／`Gate` 為準；文件角色僅供 IA 分層，不新增後端角色。

### 2.4 明確不在範圍（防止脫鉤）

- 原生手機 App、微信小程序、Electron（零 JS／憲政衝突；如需另立 ADR）。
- 重寫 FlapsErp 原廠產品本體（僅原廠通道；`flaps_bridge` 例外）。
- OEKAP／Wellfit 功能開發（僅介面/文件對接）。
- 以「增加頁面數」冒充品質；以裝飾取代資訊密度。
- 任何會破壞零 JS、金額整數、CSRF/PRG、四關的「優化」。

### 2.5 全端介面鏈（full-stack interface chain）

> 「全端」不只 S1–S9 表面，也包括後端介面；每層都有契約與驗證，禁止跨層假設。

| 層 | 範圍 | 契約要點 | 驗證 |
|---|---|---|---|
| L1 呈現 | 模板/CSS/瀏覽器 | EC-1..EC-10、零 JS、token、八態 | guards、page_matrix、a11y |
| L2 HTTP | 路由、狀態碼、標頭、CSRF | 動詞語義、錯誤碼、安全標頭、快取政策 | API 契約測試、curl |
| L3 應用 | handler→use case | 授權、輸入驗證、冪等、交易邊界 | 整合測試、fault injection |
| L4 領域 | 金額、分錄、狀態機 | i64 minor、借貸平衡、狀態轉移 | 單元/property tests |
| L5 資料 | schema、索引、WAL | 遷移、約束、索引、備份/還原 | migration test、EXPLAIN、演練 |
| L6 運維 | 日誌、健康、備份、更新 | 可觀測、退出碼、復原 | health check、runbook |

規則：任一層介面變更，必須同步契約文件＋測試；上層不得假設下層已驗證（defense in depth）。

---

## §3 體驗契約 EC-1..EC-10（全端 UI/UX 的核心定義）

> 每條契約＝可驗收規則；E-1 逐表面判定「符合／部分／N/A（附理由）／UNKNOWN」。

| ID | 契約 | 規則（可驗收） | 適用表面 | 驗證方法 |
|---|---|---|---|---|
| EC-1 | 識別與導覽 | 每頁有 `title`＋單一 `h1`；導覽由 manifest 驅動、角色過濾、`aria-current`；skip-link；無死連結 | S1/S2/S3/S6 | page_matrix、nav 快照、死鏈掃描 |
| EC-2 | 資訊架構 | studio＋section 兩層；頁分 Operator/Admin/Technical；操作者不得見開發文件連結 | S1/S2/S3/S6 | IA 文件、nav 角色矩陣 |
| EC-3 | 狀態語彙 | 八態文案互異、各含下一步；元件至少 Default/Hover/Focus/Disabled/Loading/Error | S1/S2/S3 | states.html、每頁 data-state 錨點、文案斷言 |
| EC-4 | 動作語義 | GET 讀／POST 寫＋CSRF＋PRG；危險動作二段確認；撤銷＝反向事件不刪原紀錄 | S1/S2/S7 | CSRF 矩陣、PIN、HTTP 掃描 |
| EC-5 | 內容與語氣 | zh-Hant-TW；錯誤＝問題＋原因＋處理；數字千分位／minor／+08:00；術語表一致 | 全部 | 文案 lint、ubiquitous language、內容斷言 |
| EC-6 | 視覺語言 | 只用 token（原始/語義/元件三層）；無模板硬編碼色；對比 AA；dark-first | S1/S2/S3/S4 | token guard、raw-hex 掃描、對比腳本 |
| EC-7 | 回饋與動效 | CSS-only；150ms ease；`prefers-reduced-motion`；focus-visible | S1/S2/S3 | guards、CSS 審查、鍵盤走查 |
| EC-8 | 無障礙 | WCAG 2.2 AA：語義、標籤、替代文字、鍵盤、不以顏色唯一、200% 縮放 | S1/S2/S3/S4 | a11y 檢查表、手動走查、自動掃描 |
| EC-9 | 效能與感知 | 清單 <300ms、詳情 <150ms、報表 <500ms（2k 交易）；分頁；無明顯位移 | S1/S2/S3 | 預熱＋p95、curl timing、served bytes |
| EC-10 | 韌性與恢復 | error／degraded／timeout／conflict／retry／empty；金額與權限 fail-closed、量測 fail-open | S1/S2/S3/S7/S8 | 故障注入、fault tests、PIN |

**契約使用規則**：
1. 新頁面不得在 `EXPERIENCE_SPEC` 缺條目下新增；先補規格。
2. 每頁的 PR 必須附「本頁適用契約與驗證」表。
3. N/A 必須寫理由（例：靜態頁無 POST → EC-4 N/A）。

---

## §4 表面 × 契約矩陣（S1–S9 × EC-1..EC-10）

- 產出：`docs/design/EXPERIENCE_SURFACE_MATRIX.md`（E-1 建立；每格＝狀態＋證據＋缺口 ID）。
- 規則：**零空格**；每格只能填 `符合`／`部分（GAP-ID）`／`N/A（理由）`／`UNKNOWN（解鎖條件）`。
- 已知 N/A 範例：S3 靜態頁 EC-4 N/A（無 POST）；S4 列印 EC-7 N/A（無互動）；S9 CLI EC-3 以 exit code/JSON 表達（非八態）。
- 驗收：矩陣完整性測試（`experience_traceability.rs`）檢查每格非空、每個缺口有 ID、每個 UNKNOWN 有解鎖條件。

---

## §5 角色 × 任務 × 情境矩陣（「所有情境」的可執行定義）

### 5.1 維度

| 維度 | 取值 |
|---|---|
| 角色 | V 訪客／C 經辦／R 覆核／M 主管／A 管理員／T 技術／U 稽核 |
| 任務 | 每角色 5–8 個高頻任務（E-1 盤點；範例：C＝匯入、草稿、登帳、查詢、對帳、付款） |
| 狀態 | empty／zero／loading／partial／stale／denied／error／conflict |
| 裝置 | desktop／laptop／tablet／phone |
| 網路 | LAN／WAN 高延遲／離線（殼） |
| 資料量 | 0／20／2k／100k |
| 併發 | 1／10／50 |
| 輸入 | 滑鼠／鍵盤／觸控 |
| 輔助 | 預設／reduced-motion／200%／高對比／螢幕閱讀器 |

### 5.2 覆蓋規則（避免 19,683 種組合的假承諾）

1. **關鍵路徑全測**：每角色 × 每任務 × 八態 × 主要裝置（P0 組合）必須有測試或 E2E 證據。
2. **Pairwise 抽樣**：其餘組合以 pairwise（每對維度值至少一次）抽樣；結果記錄於 `EXPERIENCE_SCENARIO_MATRIX.md`。
3. **禁止「想到才測」**：矩陣未覆蓋格必須標 `UNKNOWN`＋解鎖條件；不得留空。
4. **規模承認**：組合空間是級數，不是已交付頁數；本計畫承諾「可驗證子集」＋「擴展律」——新增頁面沿用同一契約與矩陣，不需重設計。

### 5.3 產出

- `docs/design/EXPERIENCE_SCENARIO_MATRIX.md`：維度、組合、狀態、證據、缺口。
- 驗收：P0 組合 100% 有證據；pairwise 覆蓋率 ≥ 95%；UNKNOWN 全部有 owner/日期。

### 5.4 失敗與邊界情境清單（每項須有預期/訊息/恢復/測試）

| 類 | 情境（至少） |
|---|---|
| 輸入 | 空值、超長、CJK/中英混排、emoji、HTML/SQL 字元、超大檔、0 元、負數、溢位、非法日期、時區邊界、重複鍵 |
| 狀態 | 0 筆、1 筆、2k、100k、部分成功、部分失敗、併發衝突、過期資料、權限撤銷、會話過期 |
| 環境 | DB 鎖/損毀、磁碟滿、記憶體不足、網路中斷/高延遲、時鐘漂移、編碼損壞、字型缺失、列印缺紙/黑白 |
| 操作 | 重複提交、雙擊、瀏覽器返回、深連結、多標籤、撤銷、批次部分失敗、檔案被占用 |

每格：預期行為、使用者訊息（三要素）、恢復/下一步、驗證方式；未覆蓋→UNKNOWN（解鎖條件）。

---

## §6 全問題覆蓋方法論（「大大小小問題與衍生細節」）

### 6.1 問題來源（必須逐個盤點，不得憑印象）

| 來源 | 檔案／方法 | 產出 |
|---|---|---|
| 產品 backlog | `docs/product/IMPROVEMENT_BACKLOG.md`（P0–P3） | 問題 ID、狀態、驗收 |
| 缺口/風險 | `docs/GAPS_AND_RISKS.md`（GAP-01..08、風險矩陣） | GAP／RSK ID |
| 測試 | `cargo test`（失敗/ignored/flaky）、`platform/tests/*` | TEST 缺口 |
| 靜態掃描 | guards、clippy、encoding、dead-link、token | SCAN 缺口 |
| 設計審查 | ui-v2 7 區 QA、八態、a11y、對比 | UX 缺口 |
| 使用者回饋 | UI/UX 批評、文件受眾分層、操作痛點 | UX 缺口 |
| 營運事件 | 備份/還原、鎖、OOM、時鐘、網路 | OPS 缺口 |
| 情境推演 | §5 矩陣未覆蓋格 | GAP |

### 6.2 分級與驗收

| 級 | 定義 | 驗收門檻 |
|---|---|---|
| P0 | 正確性、安全、資料完整性、阻斷 | 必須修復＋回歸測試＋證據；否則不得發布 |
| P1 | 核心流程、韌性、可維護性 | 修復或明確接受（附理由/期限） |
| P2 | 體驗、一致性、可讀性 | 排入 E-4 backlog；同輪或下輪 |
| P3 | 精緻度、觀察項 | 記錄；不阻塞發布 |

### 6.3 衍生細節五問（每個問題必答）

1. **觸發條件**：什麼輸入/狀態/時序會發生？
2. **影響面**：哪些頁面/角色/表面/資料？
3. **相鄰元件**：同頁/同元件/共用 CSS/共用 API 是否受牽連？
4. **回滾/降級**：失敗時如何回到安全狀態？
5. **驗證**：用哪個測試/命令/證據證明已修？修復是否製造新問題（交互效應）？

### 6.4 覆蓋證明與 UNKNOWN 規則

- 問題登記：`docs/design/EXPERIENCE_PROBLEM_REGISTER.md`（ID、來源、證據、分級、處置、驗證、狀態、owner/日期）。
- 指標：覆蓋率＝已處置/已識別；**未識別不是 0**——以探索性測試、設計走查、角色旅程補足，並記錄「搜尋方法」。
- 禁止以「沒想到」結案；只能以「已搜尋（方法）」＋「UNKNOWN（解鎖條件）」結案。

### 6.5 與 R0–R9 的映射

| 輪 | 透鏡 | 主要抓的問題類別 |
|---|---|---|
| R0 | 基準量測 | 效能、頁面可達、CSS bytes |
| R1 | 編譯靜態 | 型別、警告、token |
| R2 | 測試套件 | 邏輯、回歸 |
| R3 | 路由完整性 | 404、死鏈、manifest |
| R4 | 權限矩陣 | 403、導覽洩漏 |
| R5 | 視覺/無障礙 | 對比、零 JS、focus |
| R6 | 端到端流程 | 任務鏈、錯誤恢復 |
| R7 | 靜態控制台/跨裝置 | S3、響應式、列印 |
| R8 | 效能資源 | p95、bytes、位移 |
| R9 | 閉環 | 零新缺陷＝驗證完全 |

### 6.6 UI 細節檢查表（「大大小小」逐項）

| 面 | 檢查項 |
|---|---|
| 排版 | 對齊、間距節奏、行高、CJK 換行/標點、數字等寬、欄寬、長字截斷/換行、捲動 |
| 元件 | 6 態（default/hover/focus/disabled/loading/error）、focus 可見、empty |
| 表單 | label、必填、驗證時機、錯誤位置、修復指引、Tab 順序、Enter/Esc |
| 導覽 | 目前位置、上一頁/下一步、返回、深連結、麵包屑 |
| 狀態 | 八態文案、下一步、重試、空態視覺（CSS/SVG） |
| 內容 | 標題層級、術語、數字/日期/時區/單位、錯誤三要素 |
| 跨頁 | 一致性、位置可預測、無重複、無死鏈 |
| 列印 | 分頁、頁首頁尾、黑白、避免截斷 |
| 效能 | DOM/bytes/timing、無版面位移、快取 |

---

## §7 體驗規格 SSOT（`docs/design/EXPERIENCE_SPEC.md`）

**章節目錄（E-2 必須產出）**：

1. 目的、範圍、名詞
2. 體驗契約 EC-1..EC-10（逐條規則、適用、驗證）
3. 表面矩陣 S1–S9（引用 §4 檔）
4. 資訊架構與導覽（studio/section、tier、role gate）
5. 視覺語言：token 三層（原始/語義/元件）、色彩、字型（Inter/JetBrains Mono woff2）、間距、密度、圓角、陰影、dark-first
5.1 元件 token（新增）：按鈕/輸入/卡片/表格/徽章/導覽/狀態/列印
6. 元件庫（≥24）：每個含用途、解剖、狀態、Do/Don't、可訪問性、程式碼對應
7. 內容與語氣：microcopy 目錄（成功/錯誤/空/警告/確認/不可逆）、數字/日期/時區格式、術語表
8. 狀態與錯誤：八態文案、三要素、重試/下一步
9. 互動與動效：CSS-only、時序、reduced-motion
10. 無障礙：WCAG 2.2 AA 檢查表、鍵盤、焦點、替代文字
11. 效能預算：頁面類別、p95、bytes、位移
12. 跨表面一致性：S1–S9 各自規則與 N/A
13. 驗收與追溯：REQ→DEC→TEST→EVID
14. 決策記錄（格式：需求／選擇／原因／替代／優勢／缺點／證據／驗證）

**每個重要決策必須回答 8 欄**；不回答者不得進入實作。

---

## §8 體驗室（`/design`）規格

### 8.1 路由與權限

| 路由 | 內容 | Gate | 說明 |
|---|---|---|---|
| `/design` | 總覽：契約、矩陣、元件、狀態、內容、藍圖入口 | Manage | 入口卡 + 使用說明 |
| `/design/contracts` | EC-1..EC-10 條文 + 正反例 | Manage | 每條至少 1 正例 1 反例 |
| `/design/components` | ≥24 元件 × 狀態矩陣 | Manage | 每元件 6 態 |
| `/design/states` | 元件級八態 + 連結 `/states`（全頁八態 SSOT） | Manage | 不重複全頁八態 |
| `/design/content` | microcopy 目錄（依情境） | Manage | 每情境含三要素 |
| `/design/blueprints` | 頁面藍圖（wireframe 級） | Manage | 每頁族至少 1 張 |
| `/design/surfaces` | S1–S9 規則/限制/N/A 展示 | Manage | 引用表面矩陣 |
| `/design/matrices` | 表面×契約、角色×任務×情境（由 SSOT 檔渲染） | Manage | 與 docs 檔一致 |
| `/design/tokens` | token 值、對比表、dark/light | Manage | 可驗算 |
| `/design/decisions` | 體驗決策 log（8 欄） | Manage | 與 §7.14 一致 |

> 首版可先以單頁 `/design`＋`:target` 分區；當內容超過 3 頁份量再拆分（避免過早膨脹）。`/states` 保留為全頁八態 SSOT，`/design/states` 只做元件級矩陣並互鏈。

### 8.2 零 JS 與 token 約束

- 只用 `details`/`:target`/`:has()`/`:checked`/`:hover`/`:focus-visible`；不得 `<script>`、不得 `style=`。
- 顏色/間距/字體一律 `var(--token)`；不得 raw hex（token 定義區除外）。
- 列印樣式：A4、黑白可讀、頁首頁尾、避免分頁截斷。

### 8.3 註冊與驗收

- 在 `console_manifest` 加 `PageSpec`（`renderer=Dynamic`、`tier=Technical`、`gate=Manage`、`workflows=["design"]`）；更新動態頁計數測試（C-01 註記）。
- guards 掃描目錄納入新模板。
- 驗收：每路由 200；AA；`<script`=0；`style=`=0；900px 折疊；列印預覽；每元件至少 1 個真實頁面使用（或標 `planned`＋Issue）。

---

## §9 工具包與最佳文件

### 9.1 工具組體驗契約（TC-1..TC-5）

> 「工具包不精緻」的另一半：CLI/腳本/管線也是介面（與工程互動契約 TT-1..TT-8 分名，避免混淆）。

| ID | 契約 | 規則 |
|---|---|---|
| TC-1 | 一致 help | 每個子命令有 `--help`、用法、範例、退出碼說明 |
| TC-2 | 退出碼語義 | 0 成功；非 0 失敗；`UNKNOWN` 明確（不得 0 冒充） |
| TC-3 | 雙模式輸出 | 人類可讀 + `--json` 機器可讀；欄位穩定 |
| TC-4 | 無破壞預設 | 破壞性動作需 `--yes`／dry-run；預設不覆蓋 |
| TC-5 | 每台機器通用 | 偵測環境（CPU/GPU/路徑/相依）；禁寫死本機規格 |

### 9.2 文件體系（受眾 × 類型）

| 受眾 | 類型 | 範例 |
|---|---|---|
| 操作者（P1） | 快速上手、操作手冊、疑難排解 | 登入、匯入、草稿、對帳、付款 |
| 管理者（P2） | 治理、審批、報表解讀 | 覆核、例外、KPI |
| 技術者（P3） | 架構、API、部署、runbook | ADR、API、備份、更新 |
| 稽核（P5） | 證據、軌跡、控制 | 稽核軌跡、控制矩陣 |

**每份文件必備 metadata**：文件 ID、受眾、版本、適用範圍、最後驗證日期、證據連結、owner。
**硬規則**：命令可複製；不得寫死本機規格；不得有死鏈；數字附日期與方法；未驗證標 `UNKNOWN`。

### 9.3 文件質量 rubric（最佳文件）

| 維度 | 問題 | 通過標準 |
|---|---|---|
| 可執行 | 照做能否成功？ | 命令可複製、路徑通用、前置檢查 |
| 可追溯 | 數字/結論哪來？ | 每個主張有來源/命令/日期 |
| 可解釋 | 為什麼這樣做？ | 有理由、替代方案、trade-off |
| 可推導 | 從需求到實作是否一氣？ | 需求→決策→實作→測試鏈完整 |
| 可驗證 | 如何證明有效？ | 驗收命令＋預期輸出 |
| 可維護 | 誰更新、何時更新？ | owner、更新觸發、版本 |

### 9.4 文件清單與驗收（現況盤點 → 處置）

| 文件 | 受眾 | 現況 | 處置 |
|---|---|---|---|
| `docs/README.md` | 全部 | v3.0 索引 | 更新索引、加受眾入口 |
| `PROJECT_STATUS`／`NEXT_ACTIONS` | 全部 | 現況快照 | 每輪更新 |
| `GAPS_AND_RISKS` | PM/審查 | GAP/RSK | 每輪對帳 |
| `PLAN_REGISTER` | PM/架構 | 計畫 SSOT | 新增本計畫 addendum |
| `ARCHITECTURE`／`API_DESIGN`／`DATABASE_DESIGN` | 技術 | 架構 | 與 L1–L6 契約對齊 |
| `design/INFORMATION_ARCHITECTURE` | 設計/技術 | IA | 與 manifest 對帳 |
| `design/UI_DESIGN_DECONSTRUCTION` | 設計 | UI 拆解 | 併入 `EXPERIENCE_SPEC` |
| `testing/TRACEABILITY_MATRIX`／`MVP_TEST_REPORT` | QA | 測試 | 與 REQ/TEST 對帳 |
| `04-operations/*` | 操作/技術 | runbook 家族 | metadata、零死鏈、通用化 |
| `adr/*` | 技術 | 決策 | 新體驗決策用 8 欄 |

驗收：每份文件有 metadata；命令可複製且實測；零死鏈；數字附日期/方法；每個受眾有入口。

---

## §10 追溯與驗證（反幻覺的核心）

### 10.1 ID 與矩陣

| ID 類 | 範例 | 規則 |
|---|---|---|
| REQ-UX-xx | REQ-UX-01 全端體驗契約 | 每條需求至少 1 個 DEC、1 個 TEST、1 個 EVID |
| DEC-UX-xx | DEC-UX-01 dark-first 主題層 | 必填 8 欄決策格式 |
| IMPL | `platform/templates/index.html:42` | 檔案:行（可變更，需複查） |
| TEST | `page_matrix::...` | 測試全名 |
| EVID | commit／log 路徑 | 綁修訂版與日期 |

**需求集（E-2 細化）**：
REQ-UX-01 全端契約｜02 角色×任務×情境覆蓋｜03 八態與元件全狀態｜04 零 JS｜05 無障礙 AA｜06 效能預算｜07 錯誤可恢復｜08 內容與語氣｜09 視覺語言/token｜10 體驗室｜11 EXPERIENCE_SPEC｜12 工具組 TC（分名）｜13 文件體系｜14 追溯矩陣｜15 問題覆蓋｜16 跨裝置｜17 桌面殼｜18 列印/匯出｜19 多輪驗證｜20 交付/發布。

### 10.2 自動校驗

- 產出：`platform/tests/experience_traceability.rs`（純 Rust、零新依賴）。
- 檢查：
  1. 每個 REQ 至少 1 DEC／1 TEST／1 EVID；不得懸空。
  2. 每個 DEC 有 8 欄；每個 TEST 存在；每個 EVID 路徑存在。
  3. 表面×契約矩陣零空格；每個缺口有 ID；每個 UNKNOWN 有解鎖條件。
  4. 問題登記每筆有來源、分級、處置、驗證、狀態。
- 失敗即 CI 紅；不得以「文件問題」忽略。

### 10.3 驗證層級與命令

| 層 | 內容 | 命令/方法 |
|---|---|---|
| L1 靜態 | fmt、clippy、guards、編碼、token | 四關＋guards |
| L2 單元 | core/platform 邏輯 | `cargo test --locked` |
| L3 整合 | 路由、DB、CSRF、RBAC | `cargo test -p platform` |
| L4 頁面矩陣 | 角色×頁×八態 | `page_matrix` |
| L5 端到端 | 任務鏈（登入→…→稽核） | curl E2E、瀏覽器走查 |
| L6 無障礙 | AA、鍵盤、對比 | a11y 檢查表＋掃描 |
| L7 效能 | p95、bytes、位移 | 預熱＋p95、curl |
| L8 跨裝置 | 4 尺寸×3 輸入 | 走查＋截圖 |
| L9 文件/追溯 | 死鏈、metadata、矩陣 | 連結檢查＋`experience_traceability` |

### 10.4 反幻覺協議（每個人也適用）

1. 每個聲明＝命令＋exit code＋日期＋修訂版；無證據不得寫「已通過」。
2. `UNKNOWN ≠ PASS ≠ FAIL`；不得以沉默代替 UNKNOWN。
3. 不得做無數據支撐的趨勢判斷（「變快了」需前後量測）。
4. 修復後必須重跑受影響的下游驗證（交互效應）。
5. 文件和程式碼同一提交更新；數字附方法與日期。

### 10.5 推導鏈範例（可解釋/可推導）

```
REQ-UX-03 八態與元件全狀態
  → DEC-UX-03 採 CSS-only data-state 錨點（不引入 JS）
  → IMPL states.html、partials/state_anchors.html、各頁 data-state
  → TEST page_matrix::every_page_has_state_anchors、guards::templates_have_zero_javascript
  → EVID 20261113-*.log、commit <hash>
```

驗證矩陣：每個 REQ 至少覆蓋 L1–L9 中的三層（靜態/整合/端到端/文件）；缺口標 UNKNOWN。

---

## §11 C→B→A 任務（深化；每軸含體驗對齊）

### 11.1 Phase C：覆核軸收尾

| # | 任務 | 內容 | DoD |
|---|---|---|---|
| C-1 | uB-5.1 筆數預覽 | 零 JS 二段確認（伺服器渲染精確筆數） | PIN＋R4/R6 |
| C-2 | uB-5.3 八態文案 | 四頁矩陣＋下一步 | 文案表＋PIN |
| C-3 | uB-6 V3 品質 | 元件質感、AA、reduced-motion、跨視窗 | 對比表＋R5 |
| C-4 | 固化批 | A-1 聚合、A-4 保留、E-1 固化 | 文件＋測試 |
| C-5 | 單向門 | Q9 ADR（建議暫緩）；D-7(a) 上游管線小切片 | ADR×2＋PIN |
| C-6 | uB-8 並行軌 | PDF、文件範本（條件式） | 產物或 UNKNOWN |
| C-7 | uB-7 完整一輪 | 四關＋適應度＋M-1/M-5 | A–L 報告 |
| C-8 | 出口閘 | PIN＋R1–R9 | 閉環報告 |
| **C-9** | **覆核台體驗對齊** | 以 EC-1..EC-10 驗收批量裁決、`/my-decisions`、差異掃視 | 契約矩陣 C 軸全綠 |

### 11.2 Phase B：經辦軸旗艦（對帳與例外）

| # | 任務 | 內容 | DoD |
|---|---|---|---|
| B-0 | 全鏈掃描 | 段×現況×缺口×歸屬 | 表 |
| B-1 | 研究 | 對帳 UX、例外工單、撥款對帳（來源＋日期） | 研究檔 |
| B-2 | 設計 ≥3 案 | WHY/WHAT/HOW/TRADE-OFFS/EVIDENCE | 裁定 |
| B-3 | Brief | KPI：偵出率、單件時間、追溯完備、帳齡 | 確認 |
| B-4 | 實作 | 對帳工作檯、例外工單、OMS 介面 | RED→GREEN |
| B-5 | 迭代驗證 | R0–R9＋A–L | 報告 |
| **B-6** | **對帳體驗對齊** | EC-3/4/5/8/10 全驗收；例外必須可恢復 | 契約矩陣 B 軸全綠 |

### 11.3 Phase A：整合軸

| # | 任務 | 內容 | DoD |
|---|---|---|---|
| A-1 | 統一藍圖封版 | EXPERIENCE_SPEC v1.0；頁面/表面/情境對帳 | 封版 |
| A-2 | 完整工具包 | T1–T9 全部達質量門 | 清單＋證據 |
| A-3 | 文件體系 | 受眾×類型、metadata、零死鏈 | 稽核報告 |
| A-4 | 全維驗證 | L1–L9；R0–R9 | 全綠 |
| A-5 | 發布判定 | 五級；unsigned 聲明；Known Limitations | 判定 |
| **A-6** | **全端封版** | 表面×契約矩陣全綠；無 P0/P1 缺口；追溯矩陣零懸空 | 封版證據 |

---

## §12 UX-E 執行序列（E-0..E-8）

| # | 任務 | 產出 | DoD |
|---|---|---|---|
| E-0 | W0 基線複測 | 建置/測試/頁面/效能快照 | 與 §1 對帳 |
| E-1 | 全端體驗稽核 | `EXPERIENCE_AUDIT_v1.md`、`EXPERIENCE_SURFACE_MATRIX.md`、`EXPERIENCE_SCENARIO_MATRIX.md`、`EXPERIENCE_PROBLEM_REGISTER.md` | 零空格；每項證據 |
| E-2 | EXPERIENCE_SPEC | §7 全章＋REQ/DEC 表 | 核可（雙向門） |
| E-3 | 體驗室 | §8 路由與內容 | 200/AA/零 JS/無內聯 |
| E-4 | 精修 backlog | P0/P1 修復；P2 排程 | 清單歸零或接受 |
| E-5 | 全端一致性 | S1–S9 走查；跨裝置 | 矩陣全綠 |
| E-6 | 文件對齊 | 受眾標籤、metadata、死鏈 | 稽核通過 |
| E-7 | 多輪驗證 | R0–R9＋趨勢表 | 閉環零新缺陷 |
| E-8 | 封版追溯 | `EXPERIENCE_TRACEABILITY.md`＋自動校驗 | 零懸空 |

---

## §13 交付物與 DoD（完整工具包＋最佳文件）

| 類 | 交付物 | DoD |
|---|---|---|
| 程式 | T1–T5 可執行 | 四關綠、頁面/情境矩陣綠 |
| 體驗 | EXPERIENCE_SPEC、體驗室、表面/情境矩陣 | §7/§8 驗收 |
| 工具 | T8 CLI/腳本 | TC-1..TC-5 |
| 文件 | T9 docs＋runbooks＋release notes | rubric 六維；零死鏈 |
| 證據 | 測試 log、追溯矩陣、決策 log | 每 REQ 有 TEST+EVID |
| 發布 | 安裝器/更新/已知限制 | 五級判定；unsigned 誠實 |

---

## §14 風險登記

| # | 風險 | 因應 |
|---|---|---|
| R1 | 並行衝突（93 項） | C-05、單一寫者、斷言式補丁 |
| R2 | ui-v2 落地不明 | E-1 首解；補執行或記錄放棄 |
| R3 | orphan 鎖 | 先釋放；隔離 target |
| R4 | OMS 介面漂移 | 契約文件化；差異報表格式先凍結 |
| R5 | 體驗室範圍蔓延 | 首版 10 路由；新增走 manifest |
| R6 | 矩陣過大/形式化 | 關鍵路徑全測＋pairwise；自動校驗 |
| R7 | 文件/程式漂移 | 同提交；追溯測試 |
| R8 | 工具組被忽略 | TT 契約納入 T8 驗收 |
| R9 | 無障礙只靠自動 | 自動＋手動走查；AA 檢查表 |

---

## §15 待確認裁定點（附建議；未回覆依建議執行）

- **D-1 體驗室路由**：建議 `/design` 系列（10 路由）＋`Gate=Manage`。
- **D-2 視覺方向**：維持 Data-Driven Premium（已入 token）× D-3 互動；側欄依 ui-v2 執行。
- **D-3 Phase B 與 OMS 分工**：B＝工作檯/佇列/工單；OMS＝偵測/沖銷；接口＝差異報表。
- **D-4 Q9/Q10**：Q9 暫緩；Q10(a) 列入 C 軸小切片。
- **D-5 文件受眾**：加入「稽核」層（操作者/管理者/技術者/稽核）。
- **D-6 `/states` 定位**：保留為全頁八態 SSOT；`/design/states` 做元件級矩陣並互鏈。
- **D-7 表面優先序**：S1→S2→S3→S4→S5→S6→S7→S8→S9。
- **D-8 工具組範圍**：T8 納入 TT；Wellfit 原廠腳本僅建議套用，不納入本計畫交付。

---

## §16 修訂紀錄

| 版本 | 日期 | 修訂 |
|---|---|---|
| v1.0 | 2026-11-09 | 27+ 頁收斂計畫（已整合；附錄 A） |
| v2.0 | 2026-11-12 | C→B→A＋UX-E 總計畫；體驗室概念 |
| v2.1 | 2026-11-13 | 深化：EC-1..EC-10 體驗契約、S1–S9 表面矩陣、角色×任務×情境覆蓋、全問題覆蓋方法論、TT-1..TT-5 工具組契約、REQ/DEC/IMPL/TEST/EVID 追溯與自動校驗、T1–T9 工具包、文件 rubric、C-9/B-6/A-6 體驗對齊 |
| v2.2 | 2026-11-13 | 再深化：§0 指令對照、§2.5 全端介面鏈 L1–L6、§5.4 失敗/邊界情境、§6.6 UI 細節檢查表、§9.4 文件清單與驗收、§10.5 推導鏈範例 |
| v2.2.1 | 2026-11-13 | §17 執行現況回填：E-0 基線、E-3 體驗室十頁落地、並行整合收斂、四關複驗（證據連結） |
| v2.2.2 | 2026-11-13 | E-1／E-2／E-8 落地：稽核＋規格＋表面/情境矩陣＋問題登錄＋追溯與自動校驗（5/5）；TC/TT 分名；XP-009 誤報撤回 |

## §17 執行現況（2026-11-13 回填；append-only；以證據為限）

| 項目 | 狀態＋證據 |
|---|---|
| E-0 基線複測 | 隔離 target 建置綠；本日實作前後均複驗（C-05） |
| E-3 體驗室（10 頁） | **已落地**：`platform/src/design_lab.rs`（10 handlers；ROLE_MANAGE 閘；macro 生成）、`platform/templates/design_lab*.html` ×10＋`partials/lab_nav.html`、`platform/tests/design_lab.rs`（閘矩陣＋零 JS＋內容標記）；10 路由於 `api.rs`；`console_manifest`：`WF_DESIGN`＋10 PageSpec |
| manifest 計數 | 動態 43→53（體驗室）→55（並行整合兩頁）；測試修訂註記依 C-01 |
| 並行整合（密碼重設） | `/forgot-password`、`/reset-password` 補登 Public；`/admin/users/issue_reset` 入 `NON_PAGE_ROUTES`（27→28）；`web_crawl` pre-auth skip（共用版型缺口列 E-1）；`public_gate_only_for_login` 白名單化（3 頁，附理由） |
| 聚焦套件 | design_lab 2/2；page_matrix 4/4；web_crawl 6/6；password_reset 2/2；console_manifest lib 16/16 |
| 四關 | fmt EXIT=0；clippy（workspace・all-targets・-D warnings）EXIT=0；audit EXIT=0；全庫 `cargo test --locked` 結果與 commit hash 見 `docs/PLAN_REGISTER.md` addendum #43 與 `.kilo/sessions/20261113-design-lab/tests/20261113-e3-acceptance.log`；commit=4b47c3f |
| E-1／E-2／E-8 | **已落地（2026-11-13）**：`docs/design/EXPERIENCE_AUDIT_v1.md`、`docs/design/EXPERIENCE_SPEC.md`、`docs/design/EXPERIENCE_SURFACE_MATRIX.md`、`docs/design/EXPERIENCE_SCENARIO_MATRIX.md`、`docs/design/EXPERIENCE_PROBLEM_REGISTER.md`、`docs/design/EXPERIENCE_TRACEABILITY.md`＋`platform/tests/experience_traceability.rs`（5/5：零空格/五級判定/解鎖條件/token 路徑解析/XP 引用閉環） | PLAN_REGISTER #45 |
| 未完（下一步） | E-4 精修 backlog（XP-001/002/003/005）、E-5 全端一致性＋pairwise 抽樣、E-7 R6–R8 驗證窗口、E-8 封版（追溯歸檔）；C/B/A 軸依 §11 續行 | 同上 |

---

## 附錄 A：v1.0 原文（SUPERSEDED／已整合——僅供追溯，禁止作為現行指引）

> 以下為本檔 v1.0（2026-11-09「零 JS 營運控制台 27+ 頁收斂計畫」）原文保留。其未竟項已由 `1793033568961-unified-ops-console-manifest` 吸收並執行；保留依 C-01。

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