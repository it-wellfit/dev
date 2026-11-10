# 專案記憶庫與迭代教訓落實計畫

> 編號：1792461065184｜日期：2026-11-09｜狀態：v1.0 已實作（§4 步驟 1-5、7 落地；步驟 6 可選未執行）＋§8 演進建議與 §9 v2.0 任務清單（待執行）
> 來源：2026-11-07～11-09 對 `1792865422970-three-studio-governance-platform.md` 之 v3.1→v3.1-r2→v3.1-r3 精細迭代過程提煉。
> 唯一權威：本檔。
> **2026-11-09 更新**：v1.0 已交付（`.kilo/memories/` 四檔＋`.kilo/kilo.jsonc` `instructions` 自動加載＋`AGENTS.md` §7 第 4 步）。§8 為經 14 項線上來源查證之演進建議（v1.0→v2.0）。

## 1. 目標

將本次迭代提煉出 10 條工程教訓固化為 repo 內的**版本控管記憶庫**，使任何後續 session（尤其並行 session）能直接讀到並遵守，避免重蹈以下覆轍：截斷檔案、誤覆蓋並行編輯、引用語義錯位、未經驗證的覆蓋聲明。

## 2. 現況證據（調查結論）

> **修正備註（2026-11-09 全盤覆核）**：本節第 1 行「專案內目前不存在 memory 檔」之結論為**部分誤判**——`glob` 僅掃描了 repo 內（`Dev/`），未及 repo 外的同級目錄 `Knowledge/`（2026 年同級目錄）與 workspace-root 層。實際存在之記憶系統如下，本節其餘各行請以本修正為準：

| 事實 | 證據 | 意義 |
|---|---|---|
| `Dev/knowledge/`：surgical 自動繼續規則、Camino-1 對話機制、R2 review verdict | `ls Dev/knowledge/`（working-tree 未版本控管） | 可能承載 AGENTS.md 級規則但未進版本控管；實作前須逐檔查閱，不得重複書寫其領域規則 |
| `Knowledge/README.md`：知識庫索引 | `ls Knowledge/`；`grep project.md` 命中 `docs/adr/ADR-0001-tech-stack-baseline.md:16` 指出「記憶（project.md）宣稱與程式碼不一致」 | 既有記憶與程式碼矛盾已有 ADR 處置；新記憶須分 workspace 標籤（見 §3.3），禁止跨 workspace 誤用 |
| `Dev/` 根 workspace 非 git repo | ADR-0001/2026-11-03「根目錄非 git repo」 | repo 內 `.kilo/memories/` 無版本控管背書；需誠實標註（見 §6） |
| `Dev/.kilo/sessions/` 已存在且有制度 | `index.md` 規則：「每 session 目錄須有 manifest.md；session 結束時更新 index」；現況 4 個 session 目錄 | 記憶庫沿用此制度（README 即索引） |
| `Dev/.kilo/templates/session-manifest.md` 已存在 | `ls Dev/.kilo/templates/` | 記憶檔案格式可參照 manifest 慣例 |
| 迭代已寫入正式記錄 | plan 檔 §24（v3.1/v3.1-r2/v3.1-r3）＋§25.8 結構優化分析；另 `PLAN_REGISTER.md:174` 登載「plan stub 缺正文 GAP-06」相關歷史 | 記憶庫只紀錄**教訓與連結**，不複寫 plan 內容（SSOT 紀律） |

## 3. 要寫入的記憶庫內容（資料結構）

### 3.1 `.kilo/memories/README.md`（索引與維護規則）

- 記憶庫定位：只收「跨 session 才值得知道的事」——教訓、事實索引、決策記錄；**禁止複寫 SSOT**（AGENTS.md、plan 檔、KB）；只寫連結＋一句話。
- 維護規則（仿 `sessions/index.md` Rules）：每次 session 結束時，若有新教訓，append 到 `corrections.md` 並更新本索引；決策事項記入 `decisions.md`；**規則總數上限 15 條，逾限須合併/歸檔**（保持反射效率，避免稀釋）。
- 讀取順序規範：新 session 開始時先讀 `corrections.md`（自動加載見 §4 步驟 7，手動查閱為備援），再讀 `project.md`。
- **演進方向（供後續參考）**：記憶庫係數化紀錄（每條規則之命中次數、覆核日期、SUPERSEDED 狀態欄位）以支持衰減管理；規則命中統計可由 session manifest 的「規則應用記錄」彙總。

### 3.2 `.kilo/memories/corrections.md`（強制規則，10 條）

每條以**單行六欄表（ID＋五內容欄）**固定格式（值域已定，避免實作者格式分歧，並使自動檢查可行）：

| 欄 | 值域 |
|---|---|
| 規則 ID | `C-01` 起編（`C-` = correction；與 plan C-01..23 不同命名空間，避免混淆，見 ID 前綴避碰） |
| 規則 | 一句可執行命令式陳述（動詞開頭，如「先量測後動刀」） |
| 為什麼 | 背後事件名稱（如「截斷事件 2026-11-07」或「r2 覆蓋誤報」） |
| 證據 | 指向 plan 章節或檔路徑（如 `1792865422970-* §25.8-B`） |
| 驗證 | 可重算的命令或檢查步驟（如「`grep` 回驗測試名命中 10/11」） |
| 狀態 | `ACTIVE` / `SUPERSEDED`（被替代者就地標註，仿 claim superseded_by） |

**範例行**（格式示意；**v1.0 實作已完成**，真實內容見 `.kilo/memories/corrections.md`；本節自此保留為格式 SSOT，使用佔位 ID 避免與實檔混淆）：

| C-NN | 〈規則動詞句〉 | 〈事件名〉 | `1792865422970-* §…` | 〈可重算檢查〉 | ACTIVE |

共 10 條（下列來源逐條編號 C-01..C-10），**維持順序穩定**（編號一經賦予不得改序；作廢者標 SUPERSEDED 不刪號，仿 claim log seq 慣例）。

1. **不亂覆蓋亂刪**（C-01）——任何修訂先保留被取代內容，標 `SUPERSEDED` 就地歸檔，不得刪除（`1792865422970-*` §0 變更管制；§25.6-archived 即範例）。
2. **修訂記於 §24**（C-02）——所有計畫檔修訂必須在 §24 增列；歷史編號（SLICE-G18、QG-19）保留於軌跡之用，掃描器須排除 §24 區塊。
3. **先量測後動刀**（C-03）——編輯前先以機器掃描全檔 REGISTER（ID 普查、引用圖、語義覆蓋重算），未量測之聲明不得提出（§25.7 方法論）。
4. **斷言式補丁**（C-04）——每次檔改先斷言目標字串存在且唯一，斷言失敗即中止；禁止正則大範圍替換。
5. **並行編輯先定基準**（C-05）——動手前重新量測行數/大小/`mtime` 並與已知狀態對帳，發現並行改動先重讀（如 §22 被並行 session 新增 3 列實證）；實證宣稱一律 `grep` 回驗（`platform/src/api.rs` 測試名 10/11 命中）。
6. **引用完整性稽核**（C-06）——每個 `§N(.M)` 與 ID 引用須解析到定義；懸空先查出處（甄別跨文件引用如「憲章 §2.2」）；子節標題樣式須全檔統一。
7. **語義覆蓋≠ID 存在**（C-07）——數「是否被展開/引用」時，斜線簡寫（`RIG-06/07/11/12/13`）與範圍簡寫（`PSN-01..08`）須展開後計數（教訓：誤報 21/24，實為 23/24）。
8. **偽影甄別**（C-08）——掃描器給出的失敗先區分「掃描器假陽性」（字串多一空格、歷史記錄區塊）與「真錯誤」，再決定修文件或修掃描器。
9. **寫入通道**（C-09）——PowerShell 5.1：heredoc 不支援；中文/`$`/反引號/雙引號經 `python3 -c` 可完整傳遞；寫含引號檔案先用 `$c = @' ... '@` here-string 寫 temp 檔，再由 Python append（`WriteAllText` 需 `[Text.UTF8Encoding]::new($false)` 防 BOM）。
10. **誠實註記**（C-10）——未量測標 `UNKNOWN`，`UNKNOWN ≠ PASS ≠ FAIL`；數字聲明附量測方法與日期（`1792865422970-*` §0 證據紀律）。

### 3.2a 元素分層與重構工序（用户方法論落實——大/中/小/微/毫元素序列）

本計畫之方法論落實為**六階工序序列**（來自使用者「精準精細精密分析→解剖→解構→分解→重組→重構→優構」之鏈條）：

| 階 | 工序 | 產出物 | 完成標誌 |
|---|---|---|---|
| 1 | 精準精細分析 | 全檔量化普查（REGISTER/引用圖/語義覆蓋重算） | 數字有測量（§25.7） |
| 2 | 解剖 | 深層診斷（語義覆蓋 vs 宣稱之差異定位） | 差異清單（真錯誤 vs 偽影） |
| 3 | 解構 | 元素清冊（命名空間×數量×關係） | 登記表（C-01..23、WP/HP/GP、RIG/FLX…） |
| 4 | 分解 | 不可分割元素判定（§3.2b 層級表） | 每元素有層級與所屬集合 |
| 5 | 重組 | 集合／關係重配（補缺列、跨層閉環映射） | 零孤兒（§22=28/28） |
| 6 | 重構→優構 | 執行＋驗證（斷言式補丁＋deep3/deep4） | 全綠驗證紀錄＋§24 修訂 |

**元素分層尺度**（分解到不可分割為止）：

| 層 | 元素 | 例 | 不可分割判準 |
|---|---|---|---|
| 大 | 命名空間（`C-`／`WP`／`HP`／`GP`／`RIG`／`FLX`／`SCR`／`QG`） | 15 個命名空間 | 命名空間內 ID 唯一且連續；跨命名空間禁互換 |
| 中 | 登記表（`RIG-01..24` 類之表） | §22 追溯矩阵（28 列）、§25.5（30 列） | 每列有主（至少一條 RIG/切片對應）；零孤兒 |
| 小 | 單條登記（單一 RIG／PNT／SCR） | RIG-01 | 單條具備六欄制完整性（ID＋規則/為什麼/證據/驗證/狀態） |
| 微 | 單一欄位（六欄制之欄） | 「驗證」欄 | 欄位值域內（ACTIVE/SUPERSEDED 等） |
| 毫 | 最小原子（單字元／單字串） | `@` 字符 | 字符級可重算（grep/正則可定位） |

**重組規則**：每階產出物必須回到上一階驗證（閉環）；任何重組後的元素若無法證明屬於既有集合或新增登記，退回階 4 重新分解。**不亂改亂寫**作為 meta-規則管轄全鏈：階 1 未完成者不得進入階 5（先量測後動刀，C-03）；階 6 必須重跑階 1 的掃描（C-04 斷言式補丁即閉環驗證之具體形式）。

### 3.2b 元素分層普查（實測值，2026-11-09）

| 層 | 元素種類 | 實測數量 | 登記位置 |
|---|---|---|---|
| 大（命名空間） | `C-`／`WP`／`HP`／`GP`／`VS`／`HO`／`ESC`／`RB`／`RSK-G`／`OQ-GOV`／`QG`／`RIG`／`FLX`／`PSN`／`PNT`／`SCR`／`CM`／`DS`／`A11Y`／`FEED` | 20 個命名空間（15 新＋5 既有） | `1792865422970-* §0.3` |
| 中（登記表） | 追溯矩阵、閉環總表、品質閘門總表、需求登記、元件庫 | 5 張核心表 | §20／§22／§25.3–25.5／§26.4 |
| 小（單條登記） | RIG×24、FLX×18、PNT×30、PSN×14、SCR×24、CM×30 | 140 條 | §25／§26／§27 |
| 微（欄位） | 六欄制之欄（ID＋規則/為什麼/證據/驗證/狀態） | 每條 6 欄 | §3.2 格式 |
| 毫（原子） | 字符／最小字串（`@`、引號、中文標點） | 計數器級 | C-09 寫入通道預警 |

### 3.3 `.kilo/memories/project.md`（事實索引，不複寫）

- 一律以連結指向 SSOT：AGENTS.md（基線與硬性規則）、`Dev/WORKSPACE_CHARTER.md`（憲政）、`Dev/docs/standards/GOVERNANCE_STANDARD.md`（管理體系）＋`Dev/docs/standards/SHARED_STANDARDS.md`（工程基線）、`1792865422970-three-studio-governance-platform.md`（計畫 SSOT）、`Knowledge/`（合規 SSOT）。
- 僅額外記錄「跨 session 故意的現況」（如三大工作室對應 repo 清單、ID 前綴避碰清單）——每條必附來源連結，無來源者不記。
- **workspace 標籤規則**：每條事實標 `[根]`／`[ekpi]`／`[KB]` 前綴其來源 workspace；跨 workspace 通用者標 `[全]`。未標籤的事實視同未驗證（ADR-0001:16 記憶vs程式碼矛盾之防護）。
- **格式**：單行表 `事實 | 標籤 | 來源連結 | 覆核日期`；覆核日期為最近一次以 `grep`/量測命令回驗之日期，逾 30 日未覆核者標 `STALE`。

### 3.4 `.kilo/memories/decisions.md`（決策記錄）

- **格式 v1.1**：`決策 ID | 決策 | 備選方案 | 為何勝出 | 連結 | 日期 | 狀態`（7 欄；對應 plan §7.3 決策總表慣例；日期為 Ship/採用日期）。
  - 差異說明（v1.1 vs v1.0 實檔）：v1.0 實檔為 6 欄無 ID（`決策|備選|勝出|連結|日期|狀態`）；v1.1 於首欄增 `決策 ID`（`D-01` 起編，與 C-/S-/R- 分屬不同命名空間），狀態值域 `NEW`／`VERIFIED`／`SUPERSEDED`。**v1.0→v1.1 為欄位追加，既有列內容零更動**（符 C-01）；README 既有索引宣稱 `D-01..D-04` 於 v1.1 起為真。
- 初始收錄：§25.5/§22 粒度互補（互引取代合併）、22 子標題去 § 前綴、§22 對並行實證列優先（⬜ 列可改寫）原則。
- 決策增加後，同步在 README 索引中將該行標 `NEW`（未經 session-end 覆核）／`VERIFIED`（已覆核），仿 `knowledge/index.md` Coverage 表慣例。

## 4. 執行順序（有序清單，可交付給實作代理）

> **狀態：v1.0 已完成**（2026-11-09；步驟 1-5、7 落地，步驟 6 可選未執行）。本清單自此保留為**稽核基準與格式 SSOT**；v2.0 新任務見 §9，勿重複執行本節。

1. 建立 `.kilo/memories/` 目錄；寫 `README.md`（定位＋讀取順序＋session-end 維護規則）。
2. 寫 `corrections.md`（§3.2 之 10 條，每條含證據連結與驗證方式）。**前置步驟**：先 `ls Dev/knowledge/` 逐檔查閱既有自動繼續規則與機制（§2 備註），其領域規則只連結不重寫；查閱結果記錄於當次 session manifest。
3. 寫 `project.md`（§3.3 純連結式事實索引）。**前置步驟**：分 workspace 標籤每條事實來源（根 workspace／ekpi-platform／Knowledge），並引用 `docs/adr/ADR-0001-tech-stack-baseline.md:16` 之既知矛盾警語——禁止跨 workspace 誤用記憶宣稱。
4. 寫 `decisions.md`（§3.4 決策記錄）。
5. 人工檢核：確認每條連結目標檔案存在（以 `glob`/`read` 抽查，不寫入）；確認無與 AGENTS.md 重複複寫。
6. （可選）AGENTS.md §5 技能與工具政策條目末尾加一行：「跨 session 記憶見 `.kilo/memories/`」。需與既有 §5 風格一致。
7. **自動加載（使用者已決策）**：
   - `.kilo/kilo.jsonc` 頂層新增 `"instructions": [".kilo/memories/corrections.md", ".kilo/memories/project.md"]`（glob patterns，相對 project root；決策依據：教訓成為 agent 系統指令而非參考書，使演進反射化）。現況 `.kilo/kilo.jsonc` 無 `instructions` 欄位（67 行，僅 mcp/indexing），實作時直接增列。
   - `AGENTS.md`「7. 接手程序」末尾加一行：`4. 讀 \`.kilo/memories/corrections.md\`（強制規則）→ \`.kilo/memories/project.md\`（事實索引）→ \`.kilo/memories/decisions.md\`（決策記錄）。`（接手程序為 AGENTS.md §7，L57-62；行號以實作時為準）。
   - **效果與邊界**：新 session 啟動即有教訓作系統指令；規則稀釋風險以 **15 條總上限**控制（現 10 條；session-end 審查逾 15 條時啟動合併/歸檔程序——與 §3.1 一致）。`instructions` 欄位屬 config，實作後以文字向使用者確認加載生效（重啟 session 驗證）。


## 5. 驗證計畫

- 每檔第一行：版本與更新日期；內容以讀取驗證（不以作者宣稱）。
- 所有內部連結（`§N` 式引用、檔案路徑）以 grep 對照目標存在性；懸空即修補（引用完整性稽核慣例）。**（v2.0 起由 T1 `memory_lint.py` 之 L5／L5b 自動化與強化——見 §9.1；v1.0 人工版曾遺漏 8 處，實證見 §9 引言）**
- 無重複內容檢查：以關鍵字交叉比對 `AGENTS.md`、`1792865422970-*`（禁止大段複寫 SSOT）。

## 6. 風險與邊界

- **記憶與 SSOT 重複**：以純連結＋來源標註防止；`project.md` 禁止複寫 AGENTS.md 條文（憲章「一條規則一個 canonical 來源」）。
- **記憶過時**：以 session-end 維護規則緩解；過時事實須標 `SUPERSEDED`（規則 1）。
- **寫檔權限**：plan agent 無寫檔權限——實作須切換至可寫檔 agent 執行。
- **超出範圍**：記憶系統自動注入至 session 啟動（Kilo config `instructions` 欄位機制）是否啟用，見 §4 步驟 7（使用者已決策為自動加載，理由：「教訓成為 agent 的反射而非文檔」）。
- **Dev 根非 git repo 之風險**：記憶庫無 commit 背書，過時風險以 session-end 規則緩解；初始化 git 後（AGENTS.md §6）建議將 `.kilo/memories/` 納入版控。
- **與 `Dev/knowledge/` claims 之關係**：claims 為 claim-registry（Fact/Invariant/Decision 三類，有 confidence／evidence／supersede 欄位，由 `crates/kem` 演進模組管理）；`.kilo/memories/` 為 agent 作業規則（強制執行）。兩者分工：**claim 是事實宣稱，memory 是行為規則**；規則若需升級為不變量，走 kem claim 流程（見 `knowledge/index.md` 索引與 `knowledge/log.md` 決策日誌），不直接把 memory 升格為 claim。
- **覆核範圍聲明**：本計畫調查範圍為 workspace-root `Dev/` 及其同級 `Knowledge/`（2026 年同級目錄）與 `~/projects/`（`.config/kilo`、`.kilo`、`.kilocode` 已由 kilo-config skill 確認 Config File Locations）。

## 7. 完成定義（DoD）

1. `.kilo/memories/` 四檔存在、互不重複、每條附證據連結。2. `AGENTS.md` §7 第 4 步已存在且與 §4 步驟 7 一致（§4 步驟 6「AGENTS.md §5 一行」為選配、已決策不執行）。3. 本計畫 §5 驗證全綠。4. `1792865422970-*` §24 增列 v3.1-r4 修訂記錄說明本次記憶落實。

---

## 8. 研究驅動的演進建議（v1.0 → v2.0）

> 方法：本章所有建議皆由 2026-11-09 之線上研究驅動；每條建議標注（i）研究依據（可查證來源 S-xx）、（ii）對映現況（記憶庫 v1.0 實測狀態）、（iii）具體行動、（iv）驗證方式、（v）優缺點。研究結論互相衝突或僅來自二級來源者，明確標注不確定性（符 C-10 誠實註記）。
> ID 範圍：`S-`（來源）、`R-`（建議）、`A-`（反向建議）；僅本節範圍使用，未來升格走 §6 分工（memory→claim/ADR）。

### 8.1 來源清單（S-01..S-14，2026-11-09 線上查證，URL 可複查）

| ID | 來源 | 日期 | 關鍵發現（引用點） |
|---|---|---|---|
| S-01 | Anthropic, "Effective context engineering for AI agents"（anthropic.com/engineering/effective-context-engineering-for-ai-agents） | 2025-09-29 | 上下文為有限資源（attention budget）與 context rot；目標＝「最小高訊號 token 集」；JIT 檢索＋progressive disclosure（輕量指標：路徑/查詢/連結）；長時任務三槓桿：compaction／結構化筆記（agentic memory）／子代理 |
| S-02 | Böckeler (Thoughtworks), "Context Engineering for Coding Agents"（martinfowler.com/articles/exploring-gen-ai/context-engineering-coding-agents.html） | 2026-02-05 | 指令（instructions）vs 指引（guidance）；載入決策三主體（LLM／人／代理軟體）；「逐步建構、勿一次灌入」；context engineering 無單元測試；「控制的幻覺」——只能提高機率，仍需人為監督層級 |
| S-03 | Huang et al., "A Survey of Agent Memory in the Second Half"（arXiv:2602.06052，TMLR） | 2026-01（v4 2026-08） | 記憶三軸分類：substrate（內部參數/外部檢索）／mechanism（sensory, working, episodic, semantic, procedural）／subject（user/agent-centric）；記憶為自我演化基底：短期篩選經驗、長期整合為可重用知識與技能；記憶管理本身正成為可訓練能力 |
| S-04 | Cline 官方文件 "Memory Bank"（docs.cline.bot/prompting/cline-memory-bank） | 檢索 2026-11-09 | 六檔分層（projectbrief/productContext/activeContext/systemPatterns/techContext/progress）；「MUST read ALL at start of EVERY task」；activeContext 最常更新；里程碑後更新；與 README 之差異＝為 AI 上下文管理而設計 |
| S-05 | Hong et al. (Chroma), "Context Rot"（research.trychroma.com/context-rot） | 2025-07-14 | 18 個模型輸入變長時效能非均勻下降（即使簡單任務）；干擾物放大退化且非均勻；LongMemEval：聚焦輸入（~300 tokens）大幅優於全量（~113k）——「相關資訊的呈現方式」比「是否在上下文」更關鍵 |
| S-06 | agents.md 官方站 | 檢索 2026-11-09 | AGENTS.md 為開放格式、60k+ 專案採用、2025-12-09 起由 Linux Foundation 之 Agentic AI Foundation 託管；無必填欄位；最近檔案優先；「explicit user chat prompts override everything」 |
| S-07 | Inference Notes, "What is your agent's instruction budget?"（inferencenotes.co） | 2026-08-01 | Codex 將注入之 AGENTS.md 上限設為 32,768 bytes（DEFAULT_PROJECT_DOC_MAX_BYTES）且**靜默截斷**（自尾端）；引 arXiv:2606.15828：100 個熱門 repo 中 42% 有上下文膨脹、35% 有 skill leakage；「被截掉的是最新加入的規則」→非可協商項置頂 |
| S-08 | Sharma, "ContextCov: Deriving and Enforcing Executable Constraints from Agent Instruction Files"（arXiv:2603.00822，經 S-11 引述） | 2026-03 | 將自然語言指令轉為可執行驗證檢查，合規率 67.0%→88.3% |
| S-09 | "Guardrails Beat Guidance: A Large-Scale Study of Rules, Skills, and Persistent Configuration for Coding Agents"（arXiv:2604.11088） | 2026-04 | 效能對規則數具韌性（≤50 條無退化）；明確禁令（negative constraints／prohibitions）優於正向指令；引 Arize 2025：迭代優化規則於 SWE-bench Lite 提升 >15% |
| S-10 | McMillan, "Instruction Adherence in Coding Agent Configuration Files"（arXiv:2605.10039） | 2026-05-11 | 1,650 場受控 session：檔案大小/位置/架構/跨檔衝突**無統計顯著效應**；合規率隨任務與 session 內序列波動；「更複雜指令（多步程序、條件規則）之衰減可能不同——留待後續」 |
| S-11 | "Lessons Outlive Their Failures: A Case Study of Rules for AI Coding Agents"（arXiv:2610.32578） | 2026-10 | 規則檔平均隨生命週期成長 226%（1,867 repo）；證據混合（有研究稱 AGENTS.md 降時省 token；有研究稱只有禁令類有效）；建議：規則即活碼（living code artifacts） |
| S-12 | dosu.dev "Score Documentation Freshness in CI"；os-tack/docfresh；Microsoft HVE Core 文件維護頁 | 2026-05～08 | 文件新鮮度自動化三模式：manifest 映射（doc↔source）＋SHA/日期比對＋CI 退出碼；MS 模式：frontmatter 日期＋每週掃描＋冪等 issue；docfresh：每頁釘住 `verified_at` SHA 與來源 HEAD 比對 |
| S-13 | three.ws/IBM 社群貼文（Doc freshness，github.com/nirholas/three.ws） | 2026 | 「人類把過時句子當提示，代理把它當**指令**」；三輪清掃 339 個被判 stale 文件，293 個（86%）確需修正；stale 偵測僅測「動了」，非「錯了」（誠實限制） |
| S-14 | dev.to/alexefimenko（引 ETH Zurich 2026-02 研究）；dylanconlin.com "Harness Engineering" | 2026-03～ | **（二級來源，標注）** LLM 生成規則檔：-3% 成功率/+20% 成本；人手寫：+4%/+19%；行為約束（MUST/NEVER）在 10+ 條共存時稀釋為惰性；「硬護欄」（pre-commit、編譯、結構測試）比軟指令穩定 |

### 8.2 建議清單（R-01..R-12）

**R-01 把可檢查的規則執行化（軟指令→硬護欄）**（NOW）
- 依據：S-08（可執行化 67%→88.3%）、S-09（禁令＋可執行勝）、S-14（硬護欄穩定）
- 現況：C-01..C-10 皆為散文；其中 C-02／C-04／C-07 其實可機器檢查；已有先例（本專案 deep3/deep4 掃描器）
- 行動：新增 `.kilo/memories/tools/memory_lint.py`（環境已備 python3 3.14，無新依賴）：檢查 (a) corrections 六欄完整、ID 連續唯一、狀態值域 {ACTIVE, SUPERSEDED}；(b) project 每列 workspace 標籤＋日期可解析＋>30 天 WARN 補標 STALE；(c) 交叉引用解析（目標判定與解析規則見 §9.1 L5——含主計畫 §N/§N.M、本計畫 `## N.`、ADR §N.M 三種慣例；`憲章` 前綴白名單）；(d) 預算警示（corrections>8KB 或常駐合計>12KB WARN）。於 session-end 執行，輸出退出碼（**精確契約：§9.1**）
- 驗證：對 v1.0 現檔跑 lint 全綠；注入壞列（缺欄/重號/壞日期）須轉紅（兩個案例）
- 優缺點：（＋）可執行＝可驗證，抗稀釋；有先例證明合規大幅提升。（－）多一個維護腳本；格式變更需同步腳本

**R-02 常駐載入預算與分層載入觸發**（NOW）
- 依據：S-01（attention budget）、S-05（context rot）、S-07（靜默截斷、尾端先死）
- 現況：`instructions` 常駐載入 corrections.md（3.3KB）＋project.md（1.6KB）≈ 5KB；decisions.md 為手動查閱
- 行動：README 明訂「常駐預算 ≤12KB；corrections.md 上限 8KB；逾限時 project.md 移出常駐，改由 corrections.md 頂部唯一指針行觸發按需載入」；維持「非可協商規則置頂」原則（S-07）
- 驗證：R-01 lint 之預算檢查；人工核對 `kilo.jsonc` instructions 欄位與現況一致
- 優缺點：（＋）防靜默截斷、護 attention budget。（－）有過早最佳化之嫌——故以「觸發式」設計、現階段不改動（5KB 遠低於上限）

**R-03 規則撰寫標準：禁令優先、非可推論原則**（NOW）
- 依據：S-09（negative constraints 勝）、S-14（ETH：只留 non-inferable；LLM 生成檔有害）、S-11（膨脹 226%）、S-02（重複/矛盾指令警告）
- 現況：C-01..C-10 混合句型；project.md 已為純指標（符合）
- 行動：README 增「規則撰寫 4 準則」：(1) 行為約束盡量以「不得/禁止」表述；(2) 只收 agent 無法自程式碼推得（non-inferable）或跨檔聚合（cross-cutting）之事；(3) 每條必附可重算驗證；(4) 新增前先查重（防重複與矛盾）
- 驗證：新規則審查清單；lint 檢查欄位格式（語義由人審）
- 優缺點：（＋）從源頭抗稀釋/膨脹。（－）純禁令不涵蓋流程型規則——保留流程類但明確分類（蒸餾時標 PROC）

**R-04 三層記憶分工正式化（episodic／semantic／procedural）**（NOW）
- 依據：S-03（五機制分類；記憶=自我演化基底）、S-04（Cline 分層範式）、S-12
- 現況：procedural＝corrections；semantic＝project/decisions；episodic＝`.kilo/sessions/`＋平台原生記憶（session digest）——存在但**未正式化介面**
- 行動：README 增分工表與讀取時機：常駐只放 procedural＋索引；semantic 按需；episodic 於「任務匹配過去事件」時才查；明訂「情節細節留 sessions/，只把可重用教訓蒸餾進 corrections」
- 驗證：抽查 3 個 session manifest 皆可被指到；README 表格完整
- 優缺點：（＋）對齊研究分類，避免 episodic 灌入常駐。（－）多一層導航成本

**R-05 事實列升級：來源指紋（sha256）＋覆核狀態機**（NEXT）
- 依據：S-12（docfresh：`verified_at` SHA 釘住）、S-13（86% stale 確需修正；「動了≠錯了」）
- 現況：project.md 僅有日期；`Dev/` 非 git（ADR-0001），git SHA 不可用
- 行動：project.md 格式升級為「事實｜標籤｜來源連結｜來源 sha256（前 12 碼）｜覆核日期」；lint 重算 sha256 比對，不符→標 `REVERIFY`；每季覆核更新
- 驗證：lint 對未變更來源全綠；改一來源檔→該列轉 REVERIFY
- 優缺點：（＋）偵測「來源已動」而非只靠時間；不需 git。（－）每列維護成本上升；多來源事實需多指紋

**R-06 蒸餾循環正式化（事故→教訓→規則）**（NOW）
- 依據：S-03（長期記憶整合迴圈）、S-11（lessons outlive failures）、S-04（update triggers）
- 現況：C-01..C-10 已由 r2/r3 事故蒸餾一次（隱含流程）；README 有 session-end 規則但無蒸餾步驟
- 行動：README 增三步蒸餾程序：(1) session 內事故→當場記入 manifest；(2) session 結束→問「可重用嗎？」→併入既有 C 或新增；(3) 新 C 必帶事件級「為什麼」與可重算「驗證」
- 驗證：lint 檢查新 C 之「為什麼/驗證」非空；抽查 2 個案例
- 優缺點：（＋）制度化學習迴圈。（－）濫蒸餾會膨脹——由 R-03/R-07 抵消

**R-07 規則修剪機制（零命中退場）**（NOW）
- 依據：S-02（模型半年一變、有些規則不再需要）、S-11（只增不減之病）、S-07（最新規則反最易受害）
- 現況：README 有 15 條上限；無命中統計或修剪程序
- 行動：定義「每 10 個 session 或每季」修剪：零命中且非禁令者→候選降級（標 SUPERSEDED 不刪號，移入附錄區）
- 驗證：修剪紀錄寫入 decisions.md；lint 檢查 SUPERSEDED 保留編號
- 優缺點：（＋）對抗膨脹。（－）命中統計目前靠人工彙總（成本低）

**R-08 衝突解決權威序（載入記憶 vs 現況）**（NOW）
- 依據：S-13（過時指令被當指令）、S-02（複製他人 setup 之矛盾風險）、S-05（干擾物／誤導）
- 現況：平台層已有「repo 檔與工具輸出勝於記憶」原則；本記憶庫未明寫
- 行動：README 增權威序：**現行 repo 檔與工具輸出＞decisions.md＞project.md 事實＞corrections 指引**（禁令不因衝突失效——衝突時採更嚴者）；遇衝突→開清單修記憶
- 驗證：README 條文；抽查一衝突案例演練
- 優缺點：（＋）防止「記憶幻覺」污染行動。（－）幾乎無

**R-09 規則升格路徑：memory→claim/ADR**（NEXT）
- 依據：S-03（記憶操作學習化）、§6 既有分工（memory=行為規則、claim=事實宣稱）
- 現況：§6 已定「升格走 kem claim 流程」；無具體觸發條件
- 行動：定義升格觸發：當一條規則成為「不變量」級（跨專案、需正式審計）→提 ADR/claim；候選：C-01（SUPERSEDED 保留）、C-10（誠實註記）
- 驗證：升格後 claim evidence 指向本檔
- 優缺點：（＋）正式治理通道。（－）claim 流程較重——僅高價值者走

**R-10 檢索式記憶（vector/hybrid）暫緩**（LATER＋明確觸發）
- 依據：S-03（外部檢索 store）、S-01（JIT 檢索）、S-05（聚焦輸入勝全量）
- 現況：檔案＋目錄名即索引（S-01 之 lightweight identifiers 模式）；規模小（<10KB）
- 行動：**明確不採用向量庫**，直至觸發：常駐＋按需合計>50KB 或規則>40 條或記憶庫檔案>15。屆時先加 `index.md` 檢索表，再評估 hybrid
- 驗證：觸發條件寫入 README；屆時決策走 decisions.md
- 優缺點：（＋）避免過早複雜化（S-02 逐步建構原則）。（－）大規模時檢索效率待補

**R-11 子專案路徑化規則（nested AGENTS.md）**（LATER）
- 依據：S-06（最近檔案優先、嵌套支援）、S-02（path-scoped rules）
- 現況：根 `AGENTS.md` 統管；Wellfit/SyncMesh 各有自檔（既有事實）
- 行動：當子專案專屬規則>5 條時，評估下沉至 `<sub>/AGENTS.md`；根檔僅留指標
- 驗證：nested 載入行為實測
- 優缺點：（＋）減根檔負擔。（－）載入不確定性（S-02：LLM 是否真載入有非確定性）
- 觸發：LATER

**R-12 量測：規則命中與新鮮度報告**（NEXT）
- 依據：S-02（context engineering 無單元測試→靠觀察量測）、S-09（Arize 迭代優化+15%＝量測驅動）、S-10（合規率隨任務波動→需本地數據）
- 現況：README 已展望「命中次數」；未有機制
- 行動：session manifest 增「本次引用之 C-xx 清單」欄（人工 5 秒）；每季彙總成報告（併入 R-07 修剪輸入）
- 驗證：連續 3 個 session manifest 有該欄
- 優缺點：（＋）本地證據取代猜測。（－）人工填寫依賴紀律

### 8.3 研究不確定性與衝突（誠實標注；符 C-10）

1. **規則數量的效應衝突**：S-09 稱 ≤50 條無退化；S-14 稱 10+ 條行為約束稀釋為惰性；S-10 稱大小/結構無顯著效應。**處置**：維持 15 條上限（保守）；以本地量測（R-12）為最終仲裁，不押注單一研究。
2. **規則檔總體效果衝突**：S-11 綜合——有人測出降時省 token（Lulla 等），有人測出 LLM 生成檔有害、人手寫小幅有益（ETH，二級來源）。**處置**：只收 non-inferable（R-03）＋可執行化（R-01）＋量測（R-12）。
3. **二級來源標記**：S-14 之 ETH 數字、S-07 之 42%/35% 統計為二級引述（未取得原始論文全文）；僅作方向性依據。
4. **適用域外推警示**：多數研究為公開 benchmark 與通用代理，非本 repo 情境；模型版本效應快——每季重檢 S 清單。
5. **平台差異（待測）**：Codex 之 32KB 靜默截斷（S-07）為特定 harness 行為；**Kilo `instructions` 之截斷行為 UNKNOWN**——列為待測項（由 R-01 lint 之注入測試或人工實驗驗證），不假設安全。

### 8.4 反向建議（明確不做）

- **A-01 不引入外部託管記憶服務**（mem0/Zep 等）：與「零付費 AI 硬性約束」（`knowledge/claims/claim-20261107-016`）衝突；現規模檔案方案已足（S-01/S-03 之 lightweight identifiers）。觸發再評估（R-10）。
- **A-02 不採用 LLM 自動生成規則全文**（S-14）：方向性證據為負；蒸餾仍須人工把關語義與驗證。
- **A-03 不把 `sessions/` 全量注入**（S-05：聚焦勝全量）：情節僅按需檢索。
- **A-04 不複製外部規則包**（S-02）：context 相似性要求高、易矛盾重複。

### 8.5 執行順序（增量；不動 v1.0 已交付物之結構）

1. **NOW**（可執行任務清單：**§9**；依 T1→T5 順序）：R-03／R-04／R-06／R-07／R-08 併入 README；R-02 預算與觸發寫入 README；R-01 lint 腳本實作＋跑綠；§8.6 驗證。
2. **NEXT**（前置或觸發即辦）：R-05（R-01 完成後併入）；R-12（T4 建欄後起填）；R-09（首個候選出現時）。
3. **LATER**（條件觸發）：R-10（規模觸發）、R-11（規則數觸發）。
4. **本清單自身維護**：每季重檢 S 清單並把新研究併入 §8；修訂記於 decisions.md。

### 8.6 驗證計畫（§8 自身）

- R-01 lint 於 v1.0 現檔全綠；注入壞資料（缺欄/重號/壞日期）轉紅——兩案例皆須重現。
- README 新增條文（R-02/03/04/06/07/08）逐條可在檔內指到。
- 抽查：任一 C 規則可由「為什麼」追到 `1792865422970-* §25.8` 或 §24 之原始事件。
- 來源可複查性抽樣：§8.1 至少 3 條 S-xx 之 URL 以 webfetch 複查回 200 且引文存在。

---

## 9. v2.0 實作任務清單（交付給實作代理）

> **執行者**：可寫檔代理（plan agent 無寫檔權限）。**前置**：完整閱讀 §8；全程遵守 C-01..C-10（斷言式補丁、量測先行、誠實回報）。
> **範圍邊界**：只動 `.kilo/memories/` 四檔＋新增 `tools/` 目錄＋`.kilo/templates/session-manifest.md`＋`scripts/session-start.ps1`（單行，T5b）。**session 工具鏈已全數實測（非假設）**：`session-validate.ps1` 僅驗 `git_head_start`／`dirty_start`／`Created:`／`Project(s):` 四欄（L25）→ 無需改動，且**新欄不得設為必填**（否則既存 4 份 manifest 將驗證失敗）；`session-end.ps1` 僅以 regex 替換 `(pending)` 欄（L37-38）→ 新欄沿用 `(pending)` 慣例即自動相容；`session-cleanup.ps1` 僅清除 `runs\`／`tests\*.log`／`io\input|output` 之過期產物（L33-35）→ 與 manifest 欄位零互動。**不屬本次範圍**：`AGENTS.md`、`.kilo/kilo.jsonc`（v1.0 已完成）、`Dev/knowledge/`、`Knowledge/`（只連結）。
> **今日精確化稽核發現（2026-11-09，四檔全量 `§` 掃描驅動 T2a 之實證）**：corrections.md C-01/C-10 之 `§0.4`/`§0.2` 與 README L17 之 `§0.4` 為懸空（主計畫 §0 為編號清單；r3 已建立「§0 條名」慣例並廢除 `§0.N` 形式）；C-06 之歷史示例 token（`§12.1`、`§6.3`）以 `§` 前綴書寫，會被掃描器誤判為現行引用；project.md 對 ADR-0001 之 `§3-1/§3-2/§3-3` 為非慣例 hyphen 形式（lint 主 regex 盲區→以 L5b 偵測；ADR 慣例正解 `§3.1-3.3`）且 `§7` 語義不精確（正解 `§5.3`——read 實證 ADR `## 5.` 第 3 條＝「無 git 版控」）；decisions.md 無 ID 欄但 README 索引宣稱 `D-01..D-04`（三方不一致）。**四檔 `§` token 全量已枚舉（共 19 匹配：corrections 9 行／C-09 無 token；decisions 4 行；README 1 行；project 5 行），除 T2a 8 處外其餘皆可解析**（如 `§24`、`§25.7`、`§25.8-*`、`§25.5`、`§22`、`§21.2`、`§4` 等——已對主計畫/本計畫標題實測）。

### 9.1 T1：memory_lint.py（R-01 精確契約）

- 路徑：`.kilo/memories/tools/memory_lint.py`；純讀、離線、Python stdlib only；從 `Dev/` 根執行（`--root` 可覆寫）。
- 輸出：每項 `PASS|WARN|FAIL [L#] <file>:<line> <message>`；結尾 summary。退出碼：`0`＝無 FAIL（WARN 允許）、`1`＝有 FAIL、`2`＝環境錯誤（目標檔缺失／解析失敗）。
- 檢查項：
  - **L1（FAIL）** corrections.md：每資料列 6 欄；ID `C-\d{2}` 自 C-01 連續遞增、唯一；狀態 ∈ {ACTIVE, SUPERSEDED}；「為什麼」「驗證」欄非空。
  - **L2（FAIL）** project.md：每資料列 4 欄；標籤 ∈ {`[根]`,`[ekpi]`,`[KB]`,`[全]`}；覆核日期可解析為 `YYYY-MM-DD`。
  - **L3（WARN）** project.md：覆核日期距執行日 >30 日且該列未含 `STALE` → 提示補標。
  - **L4（FAIL）** decisions.md：每資料列 7 欄（v1.1）；ID `D-\d{2}` 連續、唯一；狀態 ∈ {NEW, VERIFIED, SUPERSEDED}。
  - **L5（FAIL）** 交叉引用：§-引用解析（規則如下）；`憲章` 前綴引用為跨文件白名單（INFO，不檢）。
  - **L5b（WARN）** 非慣例形式：`§\d+-\d+`（hyphen 形式，如 `§3-1`）→ 提示改用點號或條名形式。**已知盲區聲明**：主 regex `§(\d+(?:\.\d+)*)` 對 hyphen 形式只會誤抓 `§3`（假陰性），故須獨立偵測此模式（C-10：盲區明示，不假設安全）。
  - **L6（WARN）** 預算：corrections.md > 8KB；或 corrections.md＋project.md > 12KB。
  - **L7（WARN）** README 索引一致：以 regex `C-01\.\.C-(\d{2})`／`D-01\.\.D-(\d{2})` 取索引表宣告值，與實檔資料列 ID 數比對；pattern 缺失→WARN「索引列未登載」。
  - **範圍聲明**：本 lint 僅掃描四檔（引用**存在性**驗證）；不對主計畫內文做「現行主張掃描」（§24／archived 排除等約定屬 QG-09 職責；避免雙重職責）。
- **L5 解析規則**（寫入腳本，供 QG-09 沿用）：
  - 蒐集 regex：`§(\d+(?:\.\d+)*)`；掃描四檔全部行。
  - 目標判定（**逐引用取「該引用之前最近的目標標記」**，非整行判定——因單行可含多目標，如 project.md L13「ADR…§7；AGENTS.md §6」）：標記集＝{`ADR-\d{4}` → 該 ADR 檔；`憲章` → INFO 白名單；`1792865422970` → 主計畫；`1792461065184` → 本計畫；其他 `*.md` 路徑 → INFO 略過（未納入目標表）}；**無任何標記 → 預設主計畫**。
  - 主計畫解析：`N` → 存在 `## §N` 標題；`N.M`（含更深）→ 存在 `### N.M…`／`#### N.M…` 標題（後綴如 `-archived` 忽略）；`§0`＋中文註記（如「§0 證據紀律」）→ 以 `§0` 判定。**`§0.N` 形式 → FAIL**（該慣例已由 r3 廢除）。
  - 本計畫解析：`N` → `## N.` 標題；`N.M` → `### N.M` 標題。
  - ADR 解析：`N` → `## N.` 章存在；`N.M` → 章 N 存在且該章內編號列（行首 `N'\.`）數 ≥ M（例：ADR-0001 §3 有 5 條 → `§3.3` 合法；§5 有 4 條 → `§5.3` 合法）。
  - 無法解析 → FAIL（附 file:line 與解析結果）。

### 9.2 T2a：引用精確化（8 處，斷言式；修正依據見 §9 引言實證與四檔全量 `§` 掃描）

| # | 檔案 | 欄位 | 現值 | 修正為 |
|---|---|---|---|---|
| 1 | corrections.md | C-01 證據 | `` `1792865422970-* §0.4` `` | `` `1792865422970-* §0 變更管制（第 4 條）` `` |
| 2 | corrections.md | C-10 證據 | `` `1792865422970-* §0.2` 證據紀律 `` | `` `1792865422970-* §0 證據紀律（第 2 條）` `` |
| 3 | corrections.md | C-06 為什麼 | 「（§12.1-12.3、§6.3）語義錯位」 | 「（12.1-12.3、6.3 等舊子節號，已修）語義錯位」（歷史 token 去 `§` 前綴） |
| 4 | project.md | 列 1 來源 | `§3-1` | `§3.1` |
| 5 | project.md | 列 2 來源 | `§3-2` | `§3.2` |
| 6 | project.md | 列 3 來源 | `§3-3` | `§3.3` |
| 7 | project.md | 列 7 來源 | `…ADR-0001…` §7 | `…ADR-0001…` §5.3（ADR §5 第 3 條＝「無 git 版控」，read 實證） |
| 8 | README.md | L17 維護規則 | 「（仿 plan §0.4）」 | 「（仿 `1792865422970-*` §0 變更管制）」 |

### 9.3 T2：README.md 增補（精確插入規格；防重複為第一原則——研究 S-02/S-11 之膨脹警示）

README 現行節序：`讀取順序` → `維護規則` → `與既有系統的分工` → `索引` → `演進方向`。逐項規格：

| # | 對應 | 插入位置 | 內容要點 |
|---|---|---|---|
| 1 | R-08 | 新節「衝突權威序」，緊接「讀取順序」之後 | 現行 repo 檔與工具輸出＞decisions＞project＞corrections 指引；**禁令不因衝突失效——衝突採更嚴者**；遇衝突→開清單修記憶 |
| 2 | R-06 | 併入「維護規則」節，追加 bullet | 蒸餾三步：事故→(session 內)記 manifest→(session 末)問「可重用？」→併入既有 C 或新增（新 C 必帶事件級「為什麼」與可重算「驗證」） |
| 3 | R-07 | 併入「維護規則」節，追加 bullet | 修剪：每 10 session 或每季一次；零命中且非禁令→標 SUPERSEDED 不刪號；剪修建議與結果記入 decisions.md |
| 4 | R-02 | 新節「常駐載入預算」，置於「維護規則」與「與既有系統的分工」之間 | ≤12KB；corrections.md ≤8KB；逾限時 project.md 移出 `instructions`、由 corrections.md 頂部唯一指針行按需觸發；非可協商規則置頂（S-07） |
| 5 | R-03 | 新節「規則撰寫 4 準則」，緊接第 4 節 | (1) 禁令優先；(2) 只收 non-inferable／cross-cutting；(3) 每條必附可重算驗證；(4) 新增前先查重 |
| 6 | R-04 | **擴充既有「與既有系統的分工」節**（同節加子表；**不新開平行節**） | 記憶三層子表：procedural＝corrections｜semantic＝project＋decisions｜episodic＝`.kilo/sessions/`＋平台原生記憶；含「讀取時機」欄（常駐／按需／任務匹配） |
| 7 | R-04 補 | 「讀取順序」節，追加第 4 點 | 情節（episodic）僅於「任務與過去事件相關」時檢索；常駐不載入（S-05） |
| 8 | 精確化 | 「索引」節 | 「`.kilo/sessions/`（4 個現行 session）」→「（現行 session，數目以目錄為準）」；**D 條數欄（`D-01..D-04`）於 T5c 統一更新為 `D-01..D-06`**（T5a 完成後；此為 L7 WARN 之消解點）；狀態欄於 T5c 依覆核結果更新 |
| 9 | 精確化 | 首行 | 版本 `v1.0` → `v1.1` |

### 9.4 T3：decisions.md 格式 v1.1＋corrections.md 術語修正

- decisions.md：表頭改 7 欄（`決策 ID | 決策 | 備選方案 | 為何勝出 | 連結 | 日期 | 狀態`）；既有 4 列依序補 `D-01..D-04`（**欄位追加，內容零更動**）。
- corrections.md：前言「單行五欄表」→「單行六欄表（ID＋五內容欄）」（術語與本計畫 §3.2 統一）。

### 9.5 T4→T5：驗證、回寫與收尾（執行序）

1. **T4a（sensitivity 基準）**：實作 T1 後先跑**真檔**——預期初始：**L4 FAIL×4**（decisions.md 尚為 6 欄，待 T3 升級）＋**L5 FAIL×5**（C-01 `§0.4`；C-10 `§0.2`；C-06 之歷史示例 token `§12.1`、`§6.3`；README L17 `§0.4`——5 處皆為 T2a 標的，其中 C-06 兩處屬「歷史 token 誤判為現行引用」型，其餘屬懸空型）＋**WARN×4**（L5b×3：project.md hyphen 形式；L7×1：README 宣稱 `D-01..D-04` vs 實檔無 ID——T3 後消解）。project.md `§7` 之語義不合格屬目視稽核（lint 只驗存在性，不可判語義——盲區誠實標注）。記錄為 **sensitivity 證據**（證明檢查器不是空轉）。
2. **T4b（修正與增補）**：依序執行 T2a（8 處斷言式）→ T3（decisions v1.1＋corrections 術語）→ T2（README 依 §9.3 之 9 項插入規格：3 新節＋1 擴充＋3 增補＋2 精確化）。
3. **T4c（全綠重跑）**：重跑 lint → **真檔全 PASS（WARN 允許；L3／L6 預期 PASS——覆核日期為當日、檔案遠小於預算）**。
4. **T4d（負向測試）**：**僅在 temp 複本**（不得動真檔）：注入 (i) C-ID 跳號/重號 (ii) project 壞日期 (iii) `§99` 懸空引用 (iv) decisions 欄數錯 (v) `§0.N` 形式 → 各須 FAIL；修正後各須回綠（≥2 案例全程重現）。
5. **T5a**：decisions.md 增列 `D-05`（採用 memory_lint 執行化，連結 §9.1）與 `D-06`（引用精確化 8 處，連結 §9.2）。
6. **T5b（雙檔同步；已實測生成器＝`session-start.ps1` 之硬編 `$lines` L74-101，模板僅為格式文件——兩者必須同步改，否則新欄漂移）**：(a) `.kilo/templates/session-manifest.md` 於 `- dirty_end: <true|false>` 行後增 `- Rules referenced: <C-xx, ... | none>`；(b) `scripts/session-start.ps1` 於 `$lines += "- dirty_end: (pending)"`（現 L84）後增 `$lines += "- Rules referenced: (pending)"`（沿用 `(pending)` 慣例即與 `session-end.ps1` 相容）。**不做**：`session-validate.ps1` 不增必填；`session-end.ps1` 不改（人工於 session 末填實）。
7. **T5c**：更新 README 索引（D 條數欄 `D-01..D-04` → `D-01..D-06`；狀態欄依覆核結果）→ 重跑 lint 確認 **L7 WARN 消解、維持全 PASS**；回寫 session manifest（證據：命令＋輸出）；`1792865422970-*` §24 若有需要補註 r4 之延續（依 C-02）。
8. **範圍外提醒**：R-05（sha256）、R-09、R-10、R-11 保持 NEXT/LATER，本次**不執行**。

### 9.6 v2.0 完成定義（DoD）

1. lint 真檔全 PASS＋負向 5 注入全 FAIL 重現（證據入 manifest）。
2. 引用精確化 8 處完成，L5 驗證通過。
3. README 依 §9.3 九項插入規格完成（3 新節：衝突權威序／常駐載入預算／規則撰寫 4 準則；1 擴充：與既有系統的分工＋記憶三層；3 增補：讀取順序第 4 點＋維護規則蒸餾／修剪 bullets；2 精確化：索引去脆化數＋版本 v1.1）。
4. decisions.md 7 欄、`D-01..D-06` 齊備；corrections.md 術語修正完成。
5. session-manifest 模板與 `scripts/session-start.ps1` **雙檔同步**含「Rules referenced」欄；該欄未列入 `session-validate.ps1` 必填。驗證方式：`read` 核對 `session-start.ps1` 四必填欄仍在且新欄格式正確；選擇性附證：`session-validate.ps1` 實跑之錯誤數**不增加**（前後對比法；既有 warnings 不計）。
6. 全程斷言式、零內容刪除、diff 僅限 `.kilo/memories/`（四檔＋`tools/`）＋`.kilo/templates/session-manifest.md`＋`scripts/session-start.ps1`（單行）。

### 9.7 觀察項（不在本次範圍；供後續）

- `AGENTS.md` §6 之「ADR-0001 §5.3」以 ADR 慣例解析**合法**（§5 第 3 條），無需修正——但 AGENTS.md 本身不在 lint 掃描範圍；若未來擴展掃描，須為其加入 ADR/憲章目標判定。
- ADR-0001 頁首（L10）自指「見 §7」；§7 為關聯文件、未重述非 git 事實——ADR 自身文內引用精度屬 ADR 維護範疇，本文僅記錄。
- **結構性漂移風險（本次稽核發現）**：`.kilo/templates/session-manifest.md`（格式文件）與 `scripts/session-start.ps1`（`$lines` 硬編生成器）為**雙份手寫結構**，天生易漂移（本次 T5b 已同步一輪）。建議另案：`session-start.ps1` 改為讀取模板檔生成（單一來源），並可考慮納入 `memory_lint` 結構檢查。
