# 企業知識與流程智慧平台設計（OEKAP）— 深度挖掘與脈絡拓展計畫

> 主題：**企業知識與流程智慧平台**（Enterprise Knowledge & Process Intelligence Platform）
> 日期：2026-11-12｜狀態：**Implementation-ready**｜治理：C-01/C-03/C-04/C-10/C-13
> 事實邊界：Phase A **未核准**、Phase B **阻塞**（不因本計畫改變）；本計畫只做設計/盤點/接線契約，不宣稱驗證、不改閘門。
> 擁有者：Product＋Engineering｜審閱：架構/QA（獨立檢核）｜單寫者執行。
> 檔名沿用既有 plan 檔（內容為主題計畫；前序 FlapsErp 設定收口已併入 git 歷史，現由母計畫 `1794395575776` v4.1 承接，見 §7）。

---

## 0. 主題定義（一句話）

把「**文件版本＋條款錨點＋生效規則＋流程證據＋審批紀錄**」綁成**一條可重播、可稽核、證據不足即拒答或升級**的決策鏈——這是本平台有別於「文件搜尋」或「流程圖工具」的核心。

## 1. 脈絡地圖（三資產＋兩支援線＋治理）

| 資產 | 角色 | 現況（實證） | 權威文件 |
|---|---|---|---|
| `ekpi-platform/` | **判定核心原型**（local deterministic） | JSONL hash 鏈帳本；PBKDF2 本地 session；445+ 測試；`api.rs` 為未接線佔位；多模組 library-only | `docs/product/README.md`、`docs/architecture/README.md`、`docs/quality/phase-gates.md` |
| `oekap/` | **目標企業全端**（部署級） | Rust 2024＋Axum 0.8＋Askama 0.16＋PostgreSQL 18（SQLx 0.9 編譯期、`.sqlx`、`migrations/`）；`apps/web`＋`crates/{domain,application,adapters}`；Zero-JS／CSP `script-src 'none'`／ASVS 5.0／WCAG 2.2 AA | `oekap/README.md` |
| `knowledge/` | **知識/主張層**（KEM） | `index.md` 20 條 published claims（Fact／Decision／Invariant；confidence 0.79；schema 1）；`claims/`、`.kem/`、`console/`、`raw/` | `knowledge/index.md` |
| 支援線 A：`Wellfit/` | 批次與外部證據產生者 | MTRIX／Shopee／bankpay 冪等帳本＋SHA-256 投入產出 | `Wellfit/AGENTS.md` |
| 支援線 B：根 `platform/`＋`core/`＋`web/` | FlapsErp RPA 支援（非本平台） | Axum 0.7／SQLite／zero-JS | 根 `AGENTS.md` |
| 治理 | 閘門與追溯 | Phase A/B 矩陣、traceability register、assurance、C-13 五級判定 | `docs/quality/phase-gates.md`、`docs/traceability/register.md` |

## 2. 六面能力矩陣（深度挖掘主體）

| 面 | 現況（模組/證據） | 目標能力 | 缺口（誠實標註） | 驗證方式 |
|---|---|---|---|---|
| **K 知識層** | 文件 7 份/10 版本＋條款錨點＋SHA-256；KEM 20 claims（Fact/Decision/Invariant；confidence 0.79） | 版本化文件→條款→主張→證據的可解析鏈 | `knowledge_graph`／`ocr`／`multimodal` **library-only**；合規控制集（SoA）未在 `knowledge/` 直接可見（**UNKNOWN**，D2 定位）；KEM 與判定層未接線 | 錨點解析測試；claim→citation 正/負例 |
| **R 規則層** | 19 active＋1 draft；schema 驗證 fail-closed；LegalRuleML 匯出 | 規則版本化、衝突偵測、生效/失效邊界 | 規則變更遷移與衝突檢測未實作 | `--declared-rules-schema`；draft 不可消費測試 |
| **P 流程層** | process replay：禁止配對/期限/迴圈；`violate`／`insufficient_evidence`／`escalate` | BPMN 2.0／OCEL 匯入＋conformance（fitness/precision） | `bpmn`／`xes_import` **library-only**；無 conformance 指標 | 種子＋邊界案例；BPMN 往返測試（待建） |
| **E 證據層** | lexical retrieval（threshold 0.42；28 案；4 角色案）；引用或拒答；hash 鏈帳本 | 混合檢索（向量/全文/rerank）＋角色/租戶邊界 | `retrieval_v2`／`fulltext`／`reranker`／`vector_search` **未接線**；角色是 library 證據非強制 | `--eval` 硬閘 H-1..H-6；28 案不退化 |
| **D 決策層** | 決定性快照＋重播；decisions export v2；審批工作流（7 態稽核、冪等） | 跨資產決策鏈：claim→evidence→decision→approval 可重播 | 跨資產（KEM↔判定↔批次）未接線 | snapshot 重播一致；export 結構驗證 |
| **G 治理/評測層** | eval harness（67 評測案；eval 自檢 12；平台 self-test 49；硬閘 H-1..H-6）；traceability；phase gates | 需求→測試→證據→判定 全鏈可查 | A1 真實試點未做；B 系（B1/B4/B5/B6/B8）需外部決策 | register 完整性；閘門證據標準 |

## 3. 脈絡拓展（接線工作；只設計不實作）

| ID | 接線 | 現況 → 目標 | 驗收 |
|---|---|---|---|
| X1 | 原型 → `oekap` 對映 | JSONL 帳本→PostgreSQL 18；PBKDF2→Argon2id；`platform` 模組→Clean Arch 三層（已起步：`crates/adapters/src/postgres/approval.rs`） | 對映表（模組/契約/資料）逐項有去向 |
| X2 | KEM claims → 決策快照 | claim（Fact/Decision/Invariant）→ 快照引用 → 可重播 | 引用鏈正/負例（無 claim 即拒答） |
| X3 | Wellfit 批次 → 流程事件源 | 批次 ledger/產出 → process replay 事件 | 事件對映契約＋範例 |
| X4 | 合規控制集（SoA；**位置待 D2 定位**）→ 規則/評測 | 控制項 → 規則條款 → 評測案 | 控制覆蓋矩陣（可重跑）；SoA 定位 UNKNOWN 則本項標 BLOCKED |
| X5 | 治理閘門 → 證據標準 | Phase A/B 七態＋A–L＋五級判定 → 每輸出的判定欄位 | 判定模板與檢核命令 |

**X1 對映草案（原型 → `oekap`；D4 驗證後才可被實作）**

| 原型模組 | 目標位置 | 不變量 |
|---|---|---|
| `platform/src/{docs,rules,process}` | `oekap/crates/domain`（聚合＋規則＋流程） | 金額/數量禁浮點；draft 規則不可消費；禁止配對/期限/迴圈語義不變 |
| `retrieval`／`evidence` | `oekap/crates/application`（port）＋`adapters`（先 lexical） | 證據不足→`insufficient_evidence`；引用必帶錨點 |
| `snapshot`／`export` | `application` 服務＋`adapters`（LegalRuleML） | 同輸入快照重播一致 |
| `governance`／`governance_web`／`local_auth` | `oekap/apps/web`＋`adapters`（JSONL→PostgreSQL 18） | 帳本 append-only＋hash 鏈；角色 fail-closed |
| `eval`（67 案） | `oekap` 評測 crate（**待建；UNKNOWN**） | 硬閘 H-1..H-6 不退化 |

## 4. 任務（有序；文件/設計類，可單寫者執行）

| # | 任務 | 動作 | 驗收 |
|---|---|---|---|
| D1 | 名詞與脈絡 SSOT | 產出 `ekpi-platform/docs/research/oekap-脈絡地圖_v1.0.md`（產品自有 docs 樹；`docs/README.md` 已列 `research/` 為研究證據區）：三資產、名詞表（文件/條款/規則/主張/證據/流程/快照）、邊界 | 每個名詞有單一定義與指向 |
| D2 | 六面盤點（覆核） | 逐面覆核 §2 表：錨點存在性、served／library-only 標註、UNKNOWN 收斂（含 SoA 位置） | 每格有檔案錨點或 UNKNOWN，無高估 |
| D3 | 缺口與優先序 | 依「阻斷 Phase A/B」與「可驗證性」排序；外部決策標 BLOCKED（DEC-01/12–20） | 缺口表含負責面/驗證/阻塞 |
| D4 | 接線契約 | X1–X5 各出「輸入/輸出/不變量/失敗模式」；不寫實作 | 五份契約可被實作代理直接執行 |
| D5 | 驗證計畫 | 每項能力給可重跑命令與判定欄位：① `ekpi-platform/scripts/check.ps1`（canonical，須 `CHECK_EXITCODE=0`）② `cargo run --bin ekp_platform -- --selftest` ③ `--declared-rules-schema`（20 筆）④ `--search`／`--eval --test-set data/eval/search_eval.json`（28 案、H-1..H-6）⑤ `--export-legalruleml` 結構檢查 ⑥ KEM：`knowledge/index.md` 之產生器（重產不出現 diff） | 命令可重跑；輸出欄位含「命令/修訂/結果」；不宣稱 PASS |
| D6 | 登記與提交 | `ekpi-platform/docs/traceability/register.md`（產品追溯 SSOT）＋根 `docs/PLAN_REGISTER.md` #37（跨資產事實）；`ekpi-platform/docs/README.md` 索引；單一提交（僅本次檔案） | register 三欄式；禁碰清單零命中 |

## 5. 驗證
1. 文件連結 `Test-Path` 全數存在；`ekpi-platform/scripts/check-docs.ps1` 通過（若適用）。
2. §2 每格有錨點；UNKNOWN 不得寫成 PASS。
3. 判定：本計畫產出＝**設計/盤點文件**；平台能力判定不變（Phase A 未核准／Phase B 阻塞）。
4. 提交前 `git status` 對照：僅本次新增/修改檔。

## 6. 風險
| 風險 | 緩解 |
|---|---|
| 名詞漂移（文件/條款/規則/主張混用） | D1 名詞表為 SSOT；文件僅引用 |
| 把 library-only 當能力 | §2 逐項標 served/library-only；`api.rs` 永不稱契約 |
| 越界宣稱（production-ready/compliant） | 沿用 phase gates 七態與 C-13 五級；未驗證即 UNKNOWN |
| 與並行 session 衝突 | 單寫者；只動本次檔案；提交前重測 `git status` |

## 7. 非目標（不節外生枝）
- FlapsErp 設定優化收口（另檔：母計畫 `1794395575776` v4.1；A/B 報告 `CONDITIONAL PASS`）。
- OMS 工作流（`1793033952894`）、Windmill Stage 0（`1794413131000`）。
- 任何 Phase A/B 決策、部署、身分/租戶實作、AI 模型導入。

## 8. 交付物與完成條件（DoD）

| 交付物 | 位置 | 完成條件 |
|---|---|---|
| 脈絡地圖＋名詞表 | `ekpi-platform/docs/research/oekap-脈絡地圖_v1.0.md` | 三資產＋名詞＋邊界齊；每項有指向 |
| 六面能力矩陣 | 同上（§2 表落地） | 六面×五欄皆有錨點或 UNKNOWN |
| 缺口與優先序 | 同上 | 含 BLOCKED（DEC-01/12–20）標註 |
| X1–X5 接線契約 | 同上 | 每份含輸入/輸出/不變量/失敗模式 |
| 驗證計畫 | 同上 | 命令可重跑；判定欄位齊 |
| 登記 | `ekpi-platform/docs/traceability/register.md`（若適用）或根 `docs/PLAN_REGISTER.md` #37 | 三欄式；證據可回溯 |

**證據基線（實證，不得自行放大）**：`ekpi-platform` 445+ 測試、67 評測案、28 檢索案（20 答/8 拒）、19 active＋1 draft 規則、7 文件/10 版本、KEM 20 claims（confidence 0.79）、Phase A 未核准／Phase B 阻塞。