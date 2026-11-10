# 企業級智能研發工作台與平台演進計畫 v2.0（ekpi-platform）

- **Plan file:** `C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\.kilo\plans\1792865422970-quality-restoration-plan.md`
- **Date:** 2026-11-09 | **Status:** Ready for implementation（承接已執行完成之品質修復計畫 S1–S8）
- **淵源聲明（誠實記錄）：** 本路徑原存放「品質修復計畫」，於 2026-11-07 前後被並行 session 覆寫為指向 `.kilo/plans/1793782615513-wellfit-unified-plan-v2.md`（Wellfit 專案）的指標。本 session 的工作對象是 **ekpi-platform**，與該 Wellfit 文件無關；故本檔以 v2 演進計畫重建，並保留此記錄以免 provenance 斷裂。
- **承接基線（全部可查證；2026-11-09 更新）：** S1–S10 已執行並逐片以 canonical `scripts/check.ps1` 收口。最新鏈：`9df339a`（S9 證據包匯出，CTR-EXP-003）→ `97b3f4c`（重構決策準則 v1.0＋C1/C2）→ `680aacf`（S10 冪等寫入 API＋C4/C5；**503 測試 0 失敗**、canonical `CANONICAL_EXITCODE=0`）→ `b2786e4`（S11–S15 詳細執行計畫）。倉內權威路線圖：`docs/product/evolution-roadmap-v1.md`；詳細執行計畫：`docs/operations/execution-plan-s11-s15.md`。
- **輸入需求：** 使用者 2026-11-09 改寫版《企業級智能研發工作台與平台演進計畫》（四項成功準則＋三階段路徑＋三項設計原則）。本計畫將其**逐條映射到既有已驗證資產**，以「演進而非重寫」為第一設計原則。

## 0. 現行執行佇列（任何實作 agent 從此讀起；2026-11-09 排定）

| # | 任務 | 規格位置 | 收口 |
|---|---|---|---|
| Q1 | QA 演進流程文件落地（T0–T4，含防覆寫前置檢查） | 本檔 §9 | check-docs PASS＋單一提交 |
| Q2–Q6 | S11 資料地圖 → S12 行事曆 → S13 合規報告 → S14 稽核包＋IMP-021 drill → S15 量測複核 | `docs/operations/execution-plan-s11-s15.md`（每片交付物/DoD/canonical 配方/相依/順序） | 每片 canonical exit 0＋register 證據列 |
| Q7 | PG-A 接線、IMP 收口、決策解鎖 | 本檔 §5 | 同上 |

- **單寫者紀律：** 佇列嚴格按序；不做投機並行（重構準則 §3：B>10 ∨ H≥10 禁止並行期大改）。
- **本檔防護：** 此 plan 檔**不在 git 追蹤內**（`.kilo` 未納入版控）——若遭並行覆寫，還原來源為本對話記錄與 `docs/operations/execution-plan-s11-s15.md`；實作 agent 的 T0 已含儲存庫側檢查。

---

## 1. 現況資產盤點（全部有證據，非宣稱）

| 資產 | 狀態 | 證據 |
|---|---|---|
| 確定性核心（engine/rules/processes/retrieval） | 完成 | selftest 49、schema 20、eval gate 28、H-1…H-6 |
| 治理工作台（12 頁面系統） | 完成＋S7 十維躍升 | governance web 28 測試；KPI/甜甜圈/SLA 計/階段點/時間軸/chips |
| 檢索頁 `/search` | 完成 | 引用 `doc@version§section`、權限過濾、誠實空結果 |
| 核准工作流（domain→application→JSONL adapter→web） | 完成 | BDD 26 場景、integration 20 測試、雙鏈 verify |
| 佇列視圖＋SLA 計＋送出者溯源 | S8 完成 | `queue_views_filter_and_adapt_to_role` |
| API 中樞（唯讀契約） | CTR-API-001 v1 | 2 端點真資料、其餘誠實 501 |
| 決策匯出 v2 | CTR-EXP-002 | 逐位元組重放鎖定 |
| 測試/門檻 | 503 測試全綠；clippy/fmt 零警告 | 每次 canonical |
| PostgreSQL 目標態 | fail-closed 骨架＋遷移檔 | `migrations/20261106000000_approval_workflow.sql` |
| 未完項目 | IMP-019 剩 8 行註解亂碼、IMP-020(b)、IMP-021 runtime 資料演練 | `docs/reviews/README.md` |

## 2. 四項成功準則 → 映射與量化驗收

| 使用者準則 | 既有機制（已達成部分） | 本計畫新增（差距） | 量化驗收 |
|---|---|---|---|
| 1 功能完備與精致體驗 | 12 頁面＋檢索＋佇列視圖＋七狀態審計 | S12 SLA 行事曆、S8 延伸（篩選/排序）、可用性走查 | 每頁七狀態審計表全勾；`ui-state-audit.md` |
| 2 架構先進與深度優化 | 演進式模組化單體＋服務接縫（bounded-context 子路由）；port/adapter 分層 | S10 冪等寫入 API；PostgreSQL 接線；OTel（Phase 2，需 ADR） | canonical exit 0；benchmark 前後並列；微服務接縫以 router 邊界證明 |
| 3 全維度整合與價值交付 | 治理↔品質↔證據↔核准已以共同識別碼串連 | S11 資料地圖（欄位→事件→頁面血緣）；S13 合規報告 | 血緣與程式掃描一致；每功能附 Q 指標 |
| 4 可追溯可驗證 | traceability register、phase-gates、backup-log、BDD 場景、紅隊報告 | 持續（已制度化） | 每切片 register 證據列＋bundle SHA |

## 3. 平台架構設計（演進式，不推翻）

### 3.1 總體形態：模組化單體 → 微服務就緒接縫 →（DEC 後）服務化

**第一性原理：** 先把 N1 可歸因／N2 可回溯／N3 可防錯／N4 可演進做深，再談拓撲。微服務是部署決策，不是品質來源；在試點階段拆服務只會把 fail-closed 邊界拆散。因此：

```
        ┌────────── 橫切：資安(session/CSRF/rate-limit/timeout) · 可觀測(metrics/log)
        │           備份還原(bundle+drill) · 契約治理(CTR-*/gate) ──────────┐
┌───────┴──────┐   ┌───────────────┐   ┌───────────────┐
│ A 部門業務    │   │ B API 中樞與   │   │ C 治理稽核     │   ← axum 子路由＝服務接縫
│ 工作室        │   │ 資料管理       │   │ 工作室         │     （已完成合併模式）
└───────┬──────┘   └───────┬───────┘   └───────┬───────┘
        └──── application ports（typed contracts）────┘
        ┌──── domain（workflow/governance/auth/hashing：純領域）────┐
        ┌──── adapters：jsonl 雙鏈（現行真相）｜postgres（目標態）────┐
```

- **服務化路徑：** 每個 bounded context 已是獨立子路由＋獨立狀態＋獨立事件鏈；未來抽離＝把子路由升級為獨立 bin＋port 換遠端實作（adapter 已隔離 I/O）。
- **韌性：** 已有 timeout/concurrency/panic-recovery/fail-closed 讀取＋原子寫入；Phase 2 補：重試退避（出站）、斷路器（webhook）、健康探針彙總。
- **彈性/客製：** 工作流定義（stages/conditions/urgency）與規則庫皆資料驅動——客製＝改資料＋驗證，非改碼。

### 3.2 資料管理（「資料湖」的誠實落地）

- 現行：JSONL 雙鏈（治理 ledger＋核准 store）＝append-only、hash-chained、檔案即契約。
- 目標：PostgreSQL 18 為企業態（骨架＋遷移已在）；JSONL 保留為離線/可攜真相（bundle 可還原）。
- 湖倉：Phase 2 以 decisions export v2/v3＋資料地圖對接分析環境；**不入庫前不宣稱資料湖**。

### 3.3 資安（現有→演進）

- 現有：PBKDF2 本機 session、CSRF 雙重提交、same-origin、rate-limit、request-id、結構化錯誤、fail-closed。
- Phase 1 補：機密環境變數治理（已無預設帳號）、runtime 資料還原演練（IMP-021）。
- Phase 2/3：IdP/SSO/MFA、WORM、金鑰生命週期、滲透測試——**全部 DEC-12 起，未決前不實作不宣稱**。

### 3.4 AI 能力（advisory 邊界不變）

- `AgentAdvisor` port 已存在；`AutoApprove` 已被定義驗證**結構性拒絕**（IMP-020a）。
- Phase 2：advisory 建議接入核准頁（建議＋理由＋信心），決策仍全人工；可解釋性以 notes/decisions export 承載。

## 4. 技術選型建議（現行＋評估項，每項含理由/時點/替代）

| 層 | 選型 | 為何 | 替代案 | 決策點 |
|---|---|---|---|---|
| 語言/框架 | Rust 2024 + axum 0.8 | 型別即狀態機、零 unsafe、單二進位部署 | Go（生態快）| 維持（既有 503 測試資產） |
| 呈現 | Askama/maud 零 JS | 憲章 §2.3 硬規則 | HTMX（明文不計零 JS，禁用） | 不變 |
| 儲存 | JSONL（試點）＋ PostgreSQL 18（企業） | 檔案可攜＋SQLx 編譯期檢查 | SQLite（單檔） | Phase 1 末接線驗證 |
| 會話 | 行程內 process-memory（試點） | 本機範疇最簡 | Redis/DB session | Phase B（DEC-12） |
| 觀測 | 自建 /metrics＋JSON 存取日誌 | 零依賴 | OpenTelemetry | Phase 2 ADR |
| 供應鏈 | cargo-audit/deny＋SBOM 腳本 | 已制度化 | — | 週期執行 |
| 容器/編排 | 未採用 | 部署目標未決（DEC-17） | Docker/K8s | DEC-17 後 |
| AI | advisory port＋規則式引擎 | advisory 邊界＋可解釋 | LLM API | DEC-15 後 |

## 5. 分階段路線圖（對應使用者三階段；每片 canonical 收口）

### Phase 1 — 基座完成（承接現行；S9–S15＋PG 接線；詳細執行計畫見 `docs/operations/execution-plan-s11-s15.md`）
| 切片 | 內容 | 出口準則 | 狀態 |
|---|---|---|---|
| S9 | 證據包匯出 `--export-evidence-pack`（manifest＋雙鏈摘要＋decisions） | 重放逐位元組一致＋schema 鎖 | ✅ `9df339a`（canonical exit 0；501 測試） |
| S10 | API 寫入契約 v2（冪等鍵）＋POST approvals 真接線 | 契約測試＋重放防護 | ✅ `680aacf`（canonical exit 0；503 測試；重放不追加事件） |
| S11 | 資料地圖頁（欄位→事件→頁面血緣，掃描生成） | 血緣與程式一致 | ⏳ 詳細規格見執行計畫 |
| S12 | SLA 行事曆檢視（逾期紅點） | 逾期資料驅動正確 | ⏳ |
| S13 | 合規報告產生器（控制矩陣＋證據連結＋缺口） | 報告可由 register 推導 | ⏳ |
| S14 | 稽核包＋**runtime 資料還原演練**（IMP-021） | 雙鏈 verify 演練 PASS | ⏳ |
| S15 | 效能/可用性複核（量測制） | benchmark 前後並列 | ⏳ |
| PG-A | PostgreSQL approval adapter 接線＋雙軌行為共同測試 | 整合測試綠＋遷移可重放 | ⏳ |
| IMP 收口 | IMP-019 剩餘 8 行、IMP-020(b)、IMP-021 | register 狀態更新 | ⏳ |
| 決策解鎖 | DEC-01/12…20 逐項推進 | 每項 ADR＋owner | ⏳ |

### Phase 2 — 能力增強與整合（AI、分析、自動化）
- OpenTelemetry（ADR）、排程器兌現 on_timeout（IMP-020b）、AI advisory 接入、多租戶 ADR、湖倉對接、webhook 出站＋斷路器。
- 出口：試點量測基線（時間/交接/錯誤率/舉證工時）建立並公示。

### Phase 3 — 智能與韌性
- 預測性維護＝以逾期率/品質事項趨勢做預警（資料需 Phase 2 累積）；動態資源調度**僅在雲端部署（DEC-17）後適用**——本機試點階段明確標注不適用，不作空頭宣稱。

## 6. 量測語言（取代「打敗市面產品」的宣稱）

超越＝在同任務基準線上量測：完成時間、人工交接次數、錯誤率、舉證工時、逾期率。未量測前一律使用「本機示範可用」。此為誠實鐵律之直接推論，亦是使用者「可追溯可驗證」準則的落實。

## 7. 執行紀律（承襲，防止回退）

1. 每切片＝紅→綠→canonical→register 證據列；門檻只升不降。
2. 不推翻既有成果：所有變更為增量；資料檔變更必先 schema＋測試對齊。
3. 雜湊/計數/行號先算後寫（IMP-019 教訓）；gate 輸出讀完才提交。
4. 並行 session 紀律：單寫者或板面協調；禁止 `DONE`/`OKK` 式訊息。

## 8. 明確不做／外部決策

A1 真實試點（DEC-01）、IdP/WORM（DEC-12）、資料政策（DEC-13）、部署目標（DEC-17）、RPO/RTO（DEC-18）等未決前：不實作、不宣稱；本計畫僅產出可執行環境與規格。

## 9. ✅ 已落地：QA 演進流程 v1.0 文件（T0–T4 完成，提交 `d2e3f0c`）

**任務清單（code agent 執行；依序）：**
0. **T0 前置檢查（防覆寫，必做）：** (a) `git log --oneline -5` 確認無並行 session 新提交且本 §9 仍完整（本檔曾兩度遭並行覆寫：Wellfit 指標、`DONE`/`OKK`）；(b) 確認 `docs/quality/qa-evolution-flow.md` **不存在**——若已存在，**停止並比對內容後人工裁決**，禁止直接覆蓋；(c) 確認 `docs/quality/test-verification-plan-v2.md` 尚無 `qa-evolution-flow` 連結（避免重複插入）。
1. **T1** 建立 `docs/quality/qa-evolution-flow.md`，內容＝下方圍欄全文（原樣複製，僅移除外圍四反引號欄）。
2. **T2** 於 `docs/quality/test-verification-plan-v2.md` 的 `## 0. 原則` 段後新增：`- **執行流程（三工作流）**：[QA 演進流程](qa-evolution-flow.md)（閉環/對抗深審/混沌韌性）。`
3. **T3** 於 `docs/traceability/register.md` 的 R10 列之前插入 R13 列（原文見下方 R13 片段）。
4. **T4** 執行 `scripts/check-docs.ps1`（預期 PASS、連結數 ≥758）→ 提交 `docs/quality/qa-evolution-flow.md`＋`docs/quality/test-verification-plan-v2.md`＋`docs/traceability/register.md`，訊息：`docs: QA evolution flow v1.0 — three workflows grounded in research and measured evidence`。
5. **T5** 完成標記：將本 §9 標題改為「✅ 已落地：QA 演進流程 v1.0 文件（T0–T4 完成，提交 `<hash>`）」，任務清單保留為歷史；並確認 `docs/operations/execution-plan-s11-s15.md` 為 S11–S15 的下一站規格（依 §0 佇列 Q2–Q6 執行）。
6. **驗收**：check-docs PASS；文件含 §1–§9 全部章節；register 與 roadmap 連結可解析。

**研究來源（2026-11-09 檢索）：** OOPSLA 2025（DOI 10.1145/3764068）、SBQS 2024（DOI 10.1145/3701625.3701629）、ICSE 2024 PBT in Practice、DORA、Stanek 2025（Capers Jones 200× 引述）。

#### 文件全文（T1 內容）

````markdown
# QA 演進流程 v1.0（三工作流：閉環／對抗深審／混沌韌性）

- **狀態：** Active（依使用者 2026-11-09 指示建立；基於外部研究＋本倉實測證據）
- **Owner：** QA + Engineering
- **最後審閱：** 2026-11-09
- **上位文件：** [測試驗證計畫](test-verification-plan-v2.md)、[品質標準](quality-standards-v2.md)、[演化藍圖](../product/evolution-roadmap-v1.md)、[重構決策準則](../development/refactor-criteria.md)
- **配套審查：** [AUDIT-0003 紅隊報告](../reviews/2026-11-06-s1-s6-red-team.md)
- **誠實聲明：** 研究結論來自公開文獻（§2，附 DOI）；「本倉實證」來自 register/git 可查記錄。任何流程本身不改變 canonical 為唯一收口的鐵律。

## 1. 為何「Reviewer → Skeptic → Debug → Test」線性序列失敗

| # | 結構性缺陷 | 證據（文獻＋本倉） |
|---|---|---|
| 1 | **錨定效應**：reviewer 先背書，skeptic 後到要花社交成本反駁——skeptic 的職責是**證偽主張**，必須先於認可 | 紅隊方法論；本倉 AUDIT-0003 以「假設→實證否證」記錄（預設路徑假設被執行輸出否證） |
| 2 | **缺陷成本倒置**：debug 在 test 之前＝無可重現票證的猜測；產品後修復成本可達開發期 ~200×（Capers Jones，經 Stanek 2025 引述） | 本倉：空案例 vacuous-comply 在 345 個通過測試下潛伏，僅 property 紅測試抓到——先有失敗測試才有 debug 的對象 |
| 3 | **線性無法表達遞迴**：canonical 失敗 → debug → 新紅測試 → 重過門，是真實動態 | 本倉：`52b4e6c` fmt 疏失 → `ae3dcee` 修復重審；S2 資料污染 → 還原＋重審 |
| 4 | **資安不能晚**：安全語義（CSRF 需零 JS 相容）在實作後才審將迫使中途改寫 | 本倉：S1 登入 CSRF 由 header 語義重設計為 cookie 雙重提交 |

**結論：** 流程不是一條線，是帶門檻出口的閉環；三種鏡頭在不同時點介入。

## 2. 支撐研究（公開文獻，附出處）

| 發現 | 來源 | 對本專案的推論 |
|---|---|---|
| Property-based tests 變異殺滅能力遠高於手寫單元測試（≈30×；精確倍數以論文全文為準） | Ravi & Coblenz, OOPSLA 2025, DOI 10.1145/3764068 | PBT 為最高槓桿層：18 條 property 已實證抓到 H-4 缺陷；每切片新增不變式 |
| 行覆蓋 >80% 僅 54% 變異殺滅、65% 測試強度 → 覆蓋≠正確；設立 mutation 門檻 | SBQS 2024, DOI 10.1145/3701625.3701629 | 「全綠」≠「測得好」；mutation 門檻（cargo-mutants）列評估項 |
| 高績效組織以結果指標自評（CFR/lead time/MTTR/部署頻率） | DORA State of DevOps | 流程指標一律自 register/git 實測，不得發明 |
| 產品後修復 ~200×；測試左移須風險驅動 | Capers Jones（Stanek 2025） | 測試映射風險：高風險路徑前置；可見功能右移 |
| PBT 實務：differential/model-based 屬性最有效；以 mutation 自證套件 | ICSE 2024 | 本倉屬性優先決定論/重放/等價類（已符合） |

## 3. Workflow A — 風險驅動閉環（日常作業系統；立即採用）

```
SPEC/BDD ──▶ Skeptic₀（攻擊規格）──▶ RED 測試 ──▶ IMPLEMENT ──▶ 雙鏡頭審查 ──▶ CANONICAL GATE ──▶ 證據/回顧
   ▲                                                                                    │ exit 0
   └──────────────── Debug 模式（僅在門檻失敗時進入）◀──────────────────────────────────┘
```

| 階段 | 入口 | 活動 | 出口 | 自動化 |
|---|---|---|---|---|
| 0 SPEC | 切片排程 | DDD/SDD＋BDD 場景 | Skeptic₀：反演攻擊規格 | 場景↔測試掃描（待建） |
| 1 RED | 場景列表 | 測試必須失敗 | 紅＝可重現證據 | ✅ |
| 2 IMPLEMENT | 紅已存在 | 實作；debug 是模式 | 綠＋永久回歸鎖 | 部分 |
| 3 雙鏡頭 | 同一 diff | Reviewer（符合度）∥ Skeptic（證偽）並行 | 兩鏡頭皆過 | 判斷類不自動化（Goodhart） |
| 4 GATE | 雙鏡頭通過 | canonical `scripts/check.ps1` | exit 0（Blocked≠PASS） | ✅ |
| 5 證據/回顧 | gate 輸出已讀 | register＋回顧可改規則 | 規則變更入文檔/ADR | 指標自 git/register |

**Debug 紀律：** 無紅不修；修復必留永久測試；根因假設連續兩次失敗 → 升級架構師，禁止反覆亂試。

## 4. Workflow B — 對抗深審（里程碑邊界；建議於 S14 邊界執行）

- **時點：** 里程碑交界（S14 稽核包完成後、S15 效能宣稱前），半天為單位。
- **手法：** 反演提示（每項宣稱問「什麼會使它為假？」）；重放/雜湊鑑識；並行壓力（模擬並行 session——本倉實測發生過 `DONE`/`OKK` junk commits 與 plan 覆寫）；偽造事件演練（亂序/斷行/雜湊不符注入）。
- **出口：** 發現登錄 IMP-*；否證清單三態（已測試/已修復/已標注）。

## 5. Workflow C — 混沌與韌性（Shift-Right；建議 S14 後加入夜間演練）

- **現況：** 混沌情境僅達函式庫層（竄改/斷行/獨佔鎖測試已存在）。
- **升級：** 夜間演練腳本批次執行（本機排程；CI 屬 DEC 後）——竄改→拒載、寫入中斷→原子性、鎖爭用→單實例、時鐘回撥→決定論。
- **輸出：** chaos pass rate 進 register；FAIL 即 IMP-*。
- **誠實聲明：** 無 CI 前為本機約定，不宣稱 24/7。

## 6. 三工作流選擇準則

| 情境 | 用哪個 | 頻率 |
|---|---|---|
| 日常切片 | A | 每切片 |
| 里程碑宣稱將對外 | A＋B | 里程碑邊界 |
| 基礎設施/儲存層變更後 | A＋C | 變更後一週內建立 nightly |
| 新測試層導入 | A＋研究複核 | 依研究更新 |

## 7. 自動化升級積壓（每項≈一日；皆為既有能力延伸）

| # | 項目 | 接線點 | 狀態 |
|---|---|---|---|
| AB-1 | BDD 場景↔測試名可追溯掃描 | check-docs 衛星 | 待建（~40 行） |
| AB-2 | `????` 佔位字串掃描（operator-facing 零容忍）—IMP-019 機械鎖 | canonical 衛星 | 待建 |
| AB-3 | 效能退步閘（benchmark JSON 比對，>10% fail） | benchmark 後處理 | 待建 |
| AB-4 | register 列 presence 檢查 | CI 前僅警告 | 待建（低優先） |
| AB-5 | cargo-audit/deny 併 canonical | 先離線可行性 spike | 評估中 |

## 8. 度量（自 register/git 實測；不發明）

| 指標 | 定義 | 現況基線 |
|---|---|---|
| 逃逸率 | 合併後才發現的缺陷/切片 | 2026-11-06 紅隊：3 項 |
| 證偽命中率 | skeptic 每輪否決數 | AUDIT-0003：1 項 |
| 紅→綠延遲 | 紅測試建立到轉綠的提交數 | 例：空案例 1 提交內 |
| 混沌通過率 | nightly PASS/總數 | C 導入後起始蒐集 |
| 測試總數 | workspace 合計 | 503（@`680aacf`） |

## 9. 導入建議（結論）

1. 立即：Workflow A 為日常作業系統（S1–S10 事實做法的制度化）。
2. S14 邊界：執行 Workflow B（稽核包後、效能宣稱前），登錄 register。
3. S14 後：建立 Workflow C 夜間混沌演練。
4. 同批落地 AB-1/AB-2/AB-3。
````

#### T2 片段（test-verification-plan-v2.md）

於 `## 0. 原則` 段後新增：
`- **執行流程（三工作流）**：[QA 演進流程](qa-evolution-flow.md)（閉環/對抗深審/混沌韌性）。`

#### T3 片段（register.md，插於 R10 列之前）

`| R13 QA evolution flow v1.0 (2026-11-09) | [QA evolution flow](../quality/qa-evolution-flow.md); [test plan](../quality/test-verification-plan-v2.md) | Documentation only (docs/quality/qa-evolution-flow.md, test-plan pointer) | — | Three workflows (Risk-Driven Closed Loop daily OS; Adversarial Deep Audit at milestone boundaries; Chaos & Resilience nightly) grounded in cited research (OOPSLA 2025 DOI 10.1145/3764068; SBQS 2024 DOI 10.1145/3701625.3701629; ICSE 2024; DORA; Capers Jones via Stanek 2025) and measured repo evidence (503 tests; 345-tests-missed vacuous-comply caught by property test; 3 red-team findings) | Yes (docs check) | No | commit follows | Implemented (documentation) |`

#### 風險

- 本路徑 plan 檔曾遭並行覆寫：執行前先 `git log`／讀檔確認本節仍在；若被覆寫，以本節為還原來源。
- 文獻倍數（≈30×）以論文全文為準，引用時保留出處 DOI。

## 11. 全面未完成項目盤點（2026-11-10；以唯讀查證為準，HEAD `b2b601c`）

### 11.1 已完成且已驗證（無需動作）

S9–S15 全部切片、S11/S12 頁面批次、IMP-019 修復（`f60dc8a`）、IMP-020(b)（`6581933`）、IMP-021 演練（DRILL PASS）、AUDIT-0003/0004、QA 演進流程文件、519 測試基線、雙 benchmark 基線。

### 11.2 未完成——工程可單獨完成（下一輪 code agent 佇列）

| # | 項目 | 現況 | 前置 |
|---|---|---|---|
| P1 | **IMP-019 register 列修正**——修復已落地（`f60dc8a`）但 register 列仍寫 In progress（前一輪 .Replace 靜默失敗：backtick 跳脫字串未匹配）。追溯說謊必須修正 | register 列與事實不符 | 無 |
| P2 | **IMP-020 register 列修正**——同上，(b) 已落地（`6581933`）但列仍寫 (b) pending | 同上 | 無 |
| P3 | **Roadmap ticks 補齊**——S10/S12/S13/S14/S15 五列未勾（僅 S9/S11 有 ✅） | roadmap-v1.md | 無 |
| P4 | **ui-state-audit 六列補齊**——/events、/audit/chains、/role-matrix、/glossary、/help、/about 六頁審計列未寫 | ui-state-audit.md | 無 |
| P5 | **C6 同位性紅報告**——雙 EventPayload 的 serde 逐位元組同位性測試（預期紅：欄位差異實測），作為合併前置證據 | 測試未寫 | 無（紅報告本身即交付物） |
| P6 | **PG-A PostgreSQL 接線**——PgApprovalRepository 以 runtime SQL 實作（對應 migration schema），無 DB 環境誠實標注 | fail-closed 骨架 | P5（合併後單一型別）建議但非必須 |
| P7 | **頁面擴展至 27+**——現註冊 21 頁；補 workflows/{id} 詳細、events/{seq} 詳細、audit/verify 等 ≥6 頁 | 21 頁 | P1–P4（註冊表驅動測試就緒） |

### 11.3 未完成——需外部決策或資源（誠實邊界，不可單方面關閉）

| # | 項目 | 阻塞來源 |
|---|---|---|
| E1 | A1 真實試點（Phase A approval） | DEC-01 owner 未指派 |
| E2 | IdP/SSO/MFA、WORM、多租戶 | DEC-12…20 未決（Phase B） |
| E3 | CI/跨主機證據 | 無 CI 環境（IMP-017 host policy） |
| E4 | off-host 備份目標 | IMP-012 需核准的異地目標 |
| E5 | Phase 2/3（OTel、排程器、AI advisory、湖倉、預測性維護） | 依 Phase 1 試點結果與 DEC |

### 11.4 執行順序（下一輪 code agent）

```
P1+P2+P3+P4（文檔修正批，單一提交）→ P5（C6 紅報告）→ P6（PG-A）
→ P7（頁面擴展批次）→ 每片 canonical 收口
```

### 11.5 驗收

每項完成後：canonical exit 0＋register 證據列＋（文件變更時）check-docs PASS。P5 的紅報告為**預期失敗的測試**——本身即交付物，不需轉綠；合併實作時才轉綠。

#### 不變式（T1–T4 期間）

- 僅觸碰三個檔案：`docs/quality/qa-evolution-flow.md`（新建）、`docs/quality/test-verification-plan-v2.md`（一行）、`docs/traceability/register.md`（一列）——**零原始碼變更、零其他文件變更**。
- 不得覆蓋或重寫任何既有段落；T2/T3 為**純插入**。
- 完成後本 §9 標注「已落地＋提交雜湊」，任務清單保留為歷史。
