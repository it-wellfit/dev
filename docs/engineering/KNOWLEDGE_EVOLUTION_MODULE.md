# 知識管理演進模組（KEM）— 研究比較與最佳演進架構

> **文件路徑**: `docs/engineering/KNOWLEDGE_EVOLUTION_MODULE.md`
> **狀態**: Active v1.0｜**日期**: 2026-11-07｜**維護者**: Autonomous Engineering Coordinator
> **定位**: 完整研究 JEV 及所有候選方案後提出的知識演進架構——涵蓋關鍵維度、判斷準則、功能模組、整合方式、實施路線圖與驗證方法。
> **證據標籤**: `[FACT:DOC]`＝研究來源可查；`[INFER]`＝推論；`[PROP]`＝本文件提案；`[LIMIT]`＝已知限制；`[GAP]`＝待決事項。

---

## 0. 研究範圍與方法

**研究日期**: 2026-11-07。**方法**: 網路檢索 7 組主題（JEV 生態、知識圖譜生命週期、代理記憶系統、GraphRAG 家族、KM 成熟度模型、PKM 方法論、知識衰減/退役治理），交叉比對 2024–2026 來源（VLDB 2025、arXiv、官方文件、實測 benchmark）。

**結論先講**（§2 詳細比較）:
- 最佳完整參考實作＝**pi-jev-wiki**（JEV 驅動的知識演進管線）`[FACT:DOC]`
- 但 JEV＝TypeSafe 商用 API（需金鑰）`[FACT:DOC]`，違反本工作區零成本/本地優先約束（`FREE_AI_ROUTING.md`）
- **建議**: 自建 KEM，採 pi-jev-wiki 的管線與資料格式，裁決層設計為可插拔（規則→本地 LLM→JEV），並補上 JEV wiki 缺少的雙時間軸與兩軌治理 `[PROP]`

---

## 1. 名詞界定

| 術語 | 定義 |
|------|------|
| **JEV** | TypeSafe AI 的「System One」決策模型：輸入一段文字＋一個有界問題，回傳型別化機率。三原語：**Noul**（是/否）、**Choice**（單選）、**Score**（有序量表）`[FACT:DOC]` jevwiki.ai |
| **Claim（主張）** | 知識的最小單位：一句可驗證的陳述＋逐字引證＋證據連結 `[FACT:DOC]` pi-jev-wiki |
| **雙時間軸（bi-temporal）** | valid time（事實於現實中何時為真）與 transaction time（系統何時記錄）分離 `[FACT:DOC]` Graphiti/VLDB |
| **Supersession（取代不刪除）** | 新主張取代舊主張時，舊主張降級保留並連結（supersedes/superseded_by），不靜默刪除 `[FACT:DOC]` eli.ai/Moonpool |
| **兩軌治理** | Track A＝結構級變更走審批；Track B＝實例級寫入即時生效、事後審查提升（Wikidata 模式）`[FACT:DOC]` eli.ai |

---

## 2. 候選方案全面比較

### 2.1 候選清單（七大類）

| # | 候選 | 類型 | 授權/成本 | 核心強項 | 核心弱點（對本工作區） |
|---|------|------|-----------|----------|------------------------|
| A | **JEV + pi-jev-wiki** | 裁決層＋wiki 管線 | JEV 商用（金鑰）；wiki 工具開源 | 完整生命週期：raw→claims→裁決→歸檔→審查→sync→lint→取代不刪除；每主張帶證據與狀態 | JEV 需金鑰；工具非 Rust |
| B | **Zep / Graphiti** | 雙時間軸知識圖記憶 | Graphiti Apache-2.0；Zep 雲端商業 | 時間正確性最強（邊帶有效期，失效不刪）；point-in-time 查詢 | Python＋Neo4j 依賴；每 episode 多次 LLM 呼叫；非演進管線（只管記憶） |
| C | **Cognee** | KG 管線（ECL） | Apache-2.0 | 38+ 資料來源、14 種檢索模式、MCP 原生 | Python；偏檢索管線，生命週期治理薄 |
| D | **Mem0** | 向量記憶層 | Apache-2.0 | 採用最快、社群最大（~63K★） | 對話個人化導向；機構知識弱；無演進治理 |
| E | **Letta (MemGPT)** | 代理運行時＋記憶 | Apache-2.0 | 代理自管理記憶、git-backed | 整套運行時太重；非知識治理 |
| F | **GraphRAG 家族**（MS GraphRAG / LightRAG / HippoRAG2） | 檢索層 | 開源 | 多跳推理：MS GraphRAG 0.211 vs 向量 RAG 0.125 `[FACT:DOC]` 實測 | 是檢索不是演進；索引 LLM 成本 5–10×；MS 版索引 31 分鐘 |
| G | **PKM 方法論**（Zettelkasten / PARA / Evergreen / CODE） | 方法論 | 無 | Evergreen 原則（原子、概念導向、密集連結）最適合長期累積 | 全手動，無自動化驗證與治理 |

### 2.2 評分矩陣（判斷準則見 §3；1–5 分，加權總分 100）

| 判斷準則（權重） | A: JEV+pi-jev-wiki | B: Graphiti | C: Cognee | D: Mem0 | F: GraphRAG 家族 | G: 純 PKM | **H: 自建 KEM（提案）** |
|---|---|---|---|---|---|---|---|
| 可追溯性（15） | 5 | 4 | 3 | 3 | 3 | 2 | **5** |
| 時間正確性（10） | 3 | 5 | 3 | 2 | 2 | 1 | **5** |
| 演進安全性（15） | 5 | 3 | 2 | 1 | 1 | 2 | **5** |
| 檢索品質（10） | 4 | 3 | 4 | 3 | 5 | 2 | **4** |
| 治理與衝突可見（10） | 4 | 2 | 2 | 1 | 1 | 1 | **5** |
| 零成本（10） | 1（JEV 商用） | 3 | 3 | 4 | 2 | 5 | **5** |
| 本地優先/隱私（10） | 2 | 3 | 3 | 3 | 2 | 5 | **5** |
| 代理可整合（10） | 5 | 3 | 4 | 4 | 3 | 1 | **5** |
| 技術棧契合（Rust-first）（5） | 1 | 1 | 1 | 1 | 1 | 3 | **5** |
| 實作風險低（5） | 3 | 2 | 3 | 4 | 2 | 5 | **3** |
| **加權總分** | 70 | 62 | 61 | 55 | 49 | 51 | **92** |

**結論**（`[PROP]`）:
1. **採用 A 的管線設計**作為藍圖（生命週期最完整、審計性最強）。
2. **裁決層可插拔**：預設＝確定性規則（程式碼擁有所有門檻，pi-jev-wiki 哲學）→ 第二層＝本地 LLM（qwen3.5:4b）→ 選配＝JEV（有金鑰時）。這正是 `FREE_AI_ROUTING.md` §3.2 fallback 鏈的知識版。
3. **補上 A 缺少的兩項**：雙時間軸（取自 B/Graphiti）與兩軌治理（取自 eli.ai/Wikidata）。
4. **檢索**：BM25（tantivy，工作區既有技術）＋向量（Ollama embeddings）為主；PPR 多跳列為 Phase 5 選配（GraphRAG 社群摘要索引成本不合本地優先）。

---

## 3. 關鍵維度與判斷準則

### 3.1 十大關鍵維度（評分矩陣的列）

| 維度 | 定義 | 可檢驗判準（範例） |
|------|------|--------------------|
| 1. 可追溯性 | 每主張可回溯源檔＋版本＋逐字引證 | 任取 1 主張，5 分鐘內重現其證據鏈 |
| 2. 時間正確性 | 能回答「我們在日期 X 時相信什麼」 | 雙時間軸測試通過（§8 V3） |
| 3. 演進安全性 | 取代不刪除；退役傳播到所有衍生物 | 刪除傳播測試；衍生物無孤兒 |
| 4. 檢索品質 | 混合檢索；可拒答 | 金標查詢集 top-1 ≥ 20/20；錯誤期望必敗 |
| 5. 治理 | 衝突可見（成為 dispute 不被靜默覆寫）；審查佇列 | 矛盾偵測測試；佇列处置記錄 |
| 6. 成本 | 每主張裁決 token/呼叫數有預算 | 每主張 ≤ 1 次本地 LLM 呼叫（規則層 0 次） |
| 7. 本地優先 | 無金鑰可完整運作 | 斷網 smoke test 全綠 |
| 8. 代理可整合 | MCP 工具面；永不注入 context | 工具清單 ≥ 8 個；TOC 以 skill 形式暴露 |
| 9. 技術棧契合 | Rust 實作；SQLite/tantivy；四關全綠 | cargo fmt/clippy/test/audit EXIT=0 |
| 10. 可攜性 | 純 Markdown＋git；衍生物可丟棄重建 | 刪除 `.kem/` 目錄後 rebuild 全部索引 |

### 3.2 判斷準則的操作化（引 THINKING_OPERATING_SYSTEM §4）

每個維度的分數必須附證據型別：量測（指令＋退出碼）／估算（方法＋誤差）／GUESS（不得作為閘門）。矩陣中 H 欄的自評分屬 **GUESS/估算**——驗證方法（§8）負責把它們升級為量測。

---

## 4. 最佳演進架構（KEM）

### 4.1 分層架構圖

```
┌─────────────────────────────────────────────────────────────┐
│ L5 消費層   人類閱讀（Obsidian 相容）＋ MCP 工具面＋ CI 閘門  │
├─────────────────────────────────────────────────────────────┤
│ L4 演進層   sync 重驗證｜時間衰減＋再確認｜取代/退役│lint    │
├─────────────────────────────────────────────────────────────┤
│ L3 檢索層   BM25(tantivy)＋向量(Ollama embed)＋圖遍歷(選配)  │
├─────────────────────────────────────────────────────────────┤
│ L2 儲存層   wiki/*.md（SSOT）＋ .kem/ 衍生索引（可丟棄）     │
├─────────────────────────────────────────────────────────────┤
│ L1 裁決層   規則（預設）→ 本地 LLM → JEV（選配，可插拔）     │
├─────────────────────────────────────────────────────────────┤
│ L0 擷取層   raw/ 不可變源檔＋會話擷取＋逐字引證原子化        │
└─────────────────────────────────────────────────────────────┘
治理貫穿層：決策帳本（JSONL）＋審查佇列＋兩軌規則＋雙時間軸
```

### 4.2 資料模型（主張 frontmatter schema v1）

```yaml
---
id: claim-20261107-001            # 不可重用 ID
type: concept|decision|invariant|gotcha|glossary|fact
status: draft|published|superseded|deprecated|retired   # 見 §4.4
track: a|b                        # 兩軌：a=結構級(需審批) b=實例級(即時)
confidence: 0.87                  # 派生值：源數量＋再確認新近度−矛盾
valid_from: 2026-11-07            # 雙時間軸：事實為真起點
valid_to: null                    # null＝現行有效
recorded_at: 2026-11-07T10:00:00Z # 雙時間軸：系統記錄時間
supersedes: []                    # 被本主張取代的舊 ID
superseded_by: null
evidence:                         # ≥1 筆；無證據不得 published
  - path: ekpi-platform/docs/quality/phase-gates.md
    rev: a268319
    quote: "Phase A is not approved"
topics: [ekpi-platform, phase-gates]
source: raw/2026-11-07-phase-gates.md   # 指向 L0 不可變源檔
---
```

**儲存佈局**（`[PROP]`，pi-jev-wiki 佈局的本地化版）:

```
knowledge/
├── raw/<topic>/YYYY-MM-DD-slug.md     # L0 不可變（只增不改）
├── wiki/
│   ├── index.md                       # 生成 TOC
│   ├── log.md                         # 生成 append-only 日誌
│   ├── decisions/ decision-*.md       # ADR 式
│   ├── invariants/ invariant-*.md
│   ├── architecture/ module-*.md
│   └── topics/ gotcha-*/glossary-*/concept-*.md
└── .kem/                              # 衍生狀態（gitignore；可隨時重建）
    ├── ledger.jsonl                   # 裁決決策帳本（append-only）
    ├── index.db                       # tantivy/SQLite 派生索引
    └── review-queue.jsonl
```

### 4.3 裁決層（可插拔三級）

| 級 | 引擎 | 適用 | 門檻所有權 |
|----|------|------|------------|
| T1 規則（預設） | Rust 確定性檢查：證據存在、引證逐字匹配、重複偵測（hash＋相似度）、機密掃描、有效連結 | 結構性可判定項 | 程式碼（常數＋測試鎖定） |
| T2 本地 LLM | qwen3.5:4b，結構化輸出 7 維評分：groundedness/derivability/durability/importance/duplication/sensitivity/placement | 語義判斷（如「這是決策還是事實？」） | 程式碼把分數映射為決策；信心 0.30–0.70 → 審查佇列（JEV cookbook 不確定帶模式 `[FACT:DOC]`） |
| T3 JEV（選配） | TypeSafe System One（Noul/Choice/Score） | 有金鑰且需低延遲高一致性 | 同 T2 門檻映射；JEV 只出機率，**決策權恆在程式碼** |

規則: 每個裁決決策（接受/審查/拒絕＋分數＋理由）寫入 `ledger.jsonl`；覆寫（override）允許但必須記帳 `[FACT:DOC]` pi-jev-wiki 模式。

### 4.4 生命週期狀態機（含雙時間軸）

```
draft → pending_review → published ──→ superseded ──┐
   │            │                └→ deprecated ──┤→ retired（軟刪除，出庫）
   └── rejected（變更請求狀態，非實體狀態）────────┘
```

- `draft`/`pending_review`：**不進檢索**（不出現在 wiki_ask、TOC、自動完成）`[FACT:DOC]` eli.ai 模式
- `superseded`/`deprecated`：**可檢索但降級＋必標籤**；superseded 必連 superseded_by（繼任者必須自身是 published；不得自我取代）
- `retired`：完全出庫；歷史保留於 raw/ 與帳本，供稽核
- 衍生物（索引、向量、快取、摘要）各自帶 **freshness stamp**（來源 rev＋重建時間）；退役必須傳播到所有衍生物，重建不得假設退役已發生 `[FACT:DOC]` Inferama/IF4IT

### 4.5 時間衰減與再確認（Moonpool 模式本地化 `[FACT:DOC]`）

| 證據年齡 | 信心貢獻 |
|----------|----------|
| ≤ 30 天 | 100% |
| 1–6 個月 | 線性遞減 |
| > 6 個月 | 大幅衰減，**除非再確認**（reaffirmation 重置曲線） |

- 衰減率按主張類別配置：架構決策慢衰、暫時性 bug 快衰 `[FACT:DOC]` chancery 模式
- 衰減只**降權**，永不刪除；最低衰減到「需再確認才可引用」

---

## 5. 功能模組（8 個，對應工具面）

| 模組 | 職責 | 輸入 → 輸出 | 對應 MCP 工具 |
|------|------|-------------|---------------|
| M1 Capture 擷取 | 會話/文件/決策進 raw/，不可變 | 文件/會話 → raw 檔＋hash | `kem_capture` |
| M2 Claim 原子化 | 源檔切原子主張＋逐字引證 | raw 檔 → 候選 claims | （內嵌於 ingest） |
| M3 Vetting 裁決 | T1→T2→T3 三級評分與決策＋記帳 | claims → 決策＋placement brief | （內嵌於 ingest） |
| M4 Store 歸檔 | 寫入 topic 頁＋TOC/log 更新 | 決策 → wiki/*.md | `kem_finalize` |
| M5 Retrieval 檢索 | 混合查詢：BM25＋向量＋(選配)PPR；永不注入 | 查詢 → 頁面/摘錄＋信心 | `kem_ask` |
| M6 Evolution 演進 | 對 commit 重驗證 file-linked 主張；衰減評估；矛盾/重複偵測 | git diff → sync 報告 | `kem_sync` |
| M7 Governance 治理 | 審查佇列处置（accept/reject/supersede/defer）；lint 健檢；doctor | 佇列 → 处置記錄 | `kem_review`、`kem_lint`、`kem_doctor` |
| M8 Status 觀測 | 頁數/源檔/帳本/裁決用量/索引健康 | — → 狀態報告 | `kem_status` |

---

## 6. 整合方式（與本工作區現有機制）

| 現有機制 | 整合點 |
|----------|--------|
| `Projects/2026/Knowledge/`（KB，ADR-0004 單向引用） | KEM 落在 **Dev 工作區** `knowledge/`，維持單向 project→KB；不寫入 KB（對實作者唯讀 `[FACT:DOC]` knowledge-index.md） |
| MCP（kilo.jsonc 已有 9 個 server） | 新增 `kem` MCP server（Rust，stdio），註冊於 `.kilo/kilo.jsonc` |
| 證據規則（UNKNOWN≠PASS） | published 主張的 evidence 必含 path+rev+quote；lint 對「無證據主張」報錯 |
| 四關品質閘 | `kem` crate 納入 cargo fmt/clippy/test/audit；lint 融入 `scripts/check.ps1` |
| THINKING_OS 決策記錄（DEC- 格式） | wiki/decisions/ 的 decision 頁引用 DEC 條目；KEM 自身裁決寫 ledger |
| FREE_AI_ROUTING fallback 鏈 | 裁決層 T1→T2→T3 即其知識域實例；T2 走 Ollama |
| 會話 manifest 制度 | M1 capture 的會話擷取寫入 session manifest 的 artifacts 記錄 |

**明確不整合**（範圍控制）: 不導入 Neo4j（外部服務違本地優先）；不做社群摘要式 GraphRAG（索引成本）；不自建 OpenAPI（無對外 API 承諾）。

---

## 7. 實施路線圖（對齊 APQC 成熟度：先標準化再最佳化 `[FACT:DOC]`）

| 階段 | 內容 | 退出判準 | 對應 APQC 層級 |
|------|------|----------|----------------|
| **P0 契約**（1 天） | schema v1 冻結；raw/ 佈局；ledger 格式；`kem` crate 骨架 | schema 驗證測試綠（20 筆種子全過，對標 `--declared-rules-schema` 模式） | L1→L2 |
| **P1 擷取歸檔**（3 天） | M1–M4：capture→原子化→T1 規則裁決→歸檔 | 10 份真實文件走完全程；每主張 5 分鐘可溯證 | L2 |
| **P2 檢索**（3 天） | M5：tantivy BM25＋Ollama 向量＋MCP `kem_ask` | 金標查詢集 §8-V1 全綠；斷網 smoke 全綠 | L3 |
| **P3 演進**（4 天） | M6：sync 對 commit 重驗證；衰減＋再確認；取代流程 | §8-V2/V3/V4 全綠（演化/雙時間軸/退役傳播） | L3 |
| **P4 治理**（3 天） | M7–M8：審查佇列、lint、doctor、status；併入 check.ps1 | 四關 EXIT=0；lint 對種子缺陷集 100% 命中 | L4 |
| **P5 選配**（視需求） | T3 JEV 後端；PPR 多跳；跨專案多 wiki 目錄 | 有金鑰後 14 天試點＋成本報告 | L5 |

**路線圖風險控制**: 每階段結束跑四關＋證據寫入 session manifest；不綠不前進（flaps 計畫 §6 慣例）。

---

## 8. 驗證方法（把 §3.2 的 GUESS 升級為量測）

| # | 驗證 | 方法 | 通過門檻 |
|---|------|------|----------|
| V1 | 檢索品質 | 金標查詢集 20 查詢（含 4 個權限/邊界案例），對標工作區 search_eval.json 模式 `[FACT:DOC]` ekpi CLI contract | top-1 ≥ 20/20；錯誤期望必敗（fail-closed）；缺資料源必拒 |
| V2 | 演化正確性 | 取代測試：新主張否證舊主張 → 舊者 status=superseded＋連結＋降級可檢索 | 狀態機轉移 100% 符合 §4.4 |
| V3 | 雙時間軸 | 查詢「日期 X 時的信念」→ 回應 valid_from≤X<valid_to 的主張版本 | 點時查詢 3 組情境全對 |
| V4 | 退役傳播 | retire 一主張 → 檢查索引/向量/TOC/log 四處衍生物 | 衍生物零殘留；重建路徑可用 |
| V5 | 可追溯性 | 隨機抽 10 主張 → 重現證據鏈（path+rev+quote 逐字匹配） | 10/10 於 5 分鐘內 |
| V6 | 成本預算 | 每主張裁決呼叫數統計（帳本） | T1=0 次 LLM；T2 ≤1 次/主張 |
| V7 | 衰減/再確認 | 構造 6 個月舊證據 → 信心衰減；再確認 → 回升 | 衰減曲線符合 §4.5 配置 |
| V8 | 矛盾可見性 | 注入互斥主張 → 成為 dispute 進審查佇列，不靜默覆寫 | 佇列捕獲 100% |
| V9 | 四關回歸 | kem crate＋工作區 fmt/clippy/test/audit | 全 EXIT=0 |
| V10 | 可攜性 | 刪除 `.kem/` → rebuild → V1–V8 重跑全綠 | 衍生物完全可再生 |

**失敗模式與緩解**（研究來源的已知坑）:

| 失敗模式 | 來源證據 | 緩解 |
|----------|----------|------|
| 知識庫變傾倒場（無裁決） | pi-jev-wiki 設計動機 `[FACT:DOC]` | M3 裁決為強制閘；duplication → 強化既有主張而非新增 |
| 靜默覆寫歷史 | eli.ai/Moonpool | 取代不刪除＋帳本 append-only |
| 衍生物殘留（索引比源新/舊） | Inferama「retirement must propagate」 | V4＋每衍生物帶 freshness stamp |
| 小型本地模型結構化輸出不可靠 | Graphiti README 警告 `[FACT:DOC]` | T2 僅作評分不作文法輸出；門檻映射在 Rust 端；0.30–0.70 入人工佇列 |
| 檢索層越權（繞過生命週期過濾） | turntakingreview「filter before ranking」 | 檢索前強制 status/validity 過濾（SQL/tantivy filter，非 prompt 層） |
| OSS 依賴棄坑/供應鏈 | SHARED_STANDARDS §2.2 | 依賴最小化（tantivy+serde+reqwest 級別）；deny.toml；鎖定 |

---

## 9. 決策記錄

| 決策 | 結論 | 依據 |
|------|------|------|
| 自建 vs 現成品 | **自建 KEM**（Rust crate），採 pi-jev-wiki 藍圖 | 評分矩陣 92 vs 70；零成本/本地優先/Rust 契約 |
| 裁決引擎 | 規則為主、本地 LLM 為輔、JEV 選配 | FREE_AI_ROUTING 無金鑰現狀（2026-11-06 實測 401/403） |
| 時間模型 | 雙時間軸（valid + recorded） | Graphiti/VLDB bitemporal 為時間正確性最優解 |
| 治理模型 | 兩軌（A 結構審批 / B 實例即時） | Wikidata 規模實證：實例級預審批會殺死吞吐 `[FACT:DOC]` eli.ai |
| 檢索 | BM25＋向量混合；PPR 列選配 | 實測：單一事實向量贏、多跳圖贏、混合最佳 `[FACT:DOC]` benchmark §2.1-F |

**開放問題**（`[GAP]`，需決策）:
1. KEM 落點：根工作區 `knowledge/` 或每子專案自帶 wiki？（建議：根工作區單庫，topics 前綴分專案——避免多 wiki 同步複雜度）
2. P5 是否值得為 JEV 付費？（建議：除非 T2 信心帶人工審查量 >50/週，否則不付費）
3. 金標查詢集 20 查詢由誰標註？（建議：先由 agent 產候選、人工複核 30 分鐘，與 ekpi annotation κ 模式一致）

---

## 10. 相關文件

- `docs/engineering/FREE_AI_ROUTING.md` — 裁決層 T1→T2→T3 的資源基礎
- `docs/engineering/THINKING_OPERATING_SYSTEM.md` — §4 量化方法論被本文件 §3.2、§8 引用
- `docs/knowledge-index.md` — 既有 KB 消費契約（ADR-0004）
- `.kilo/plans/1793966971740-autonomous-engineering-evolution-workflow.md` — KEM 是其知識層落地
- 來源: jevwiki.ai（JEV 概念/cookbook）、pi.dev/packages/pi-jev-wiki、VLDB vol18 p1390（KG 生命週期）、eli.ai concept-governance（兩軌）、Moonpool docs（衰減）、Graphiti/Zep/Mem0/Cognee 官方文件與 2026 比較、APQC KM CAT、GraphRAG benchmark（wonlab 2026-08、arXiv 2506.05690）
