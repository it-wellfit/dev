# EKPI Platform — 品質修復與工藝精進計畫（Quality Restoration & Craft Plan）

- **Plan file:** `C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\.kilo\plans\1792865422970-quality-restoration-plan.md`
- **Date:** 2026-11-05 | **Status:** Ready for implementation
- **Trigger:** 使用者驗收回饋——「開啟後失望、很多錯誤、簡陋易壞、與規格計畫相距甚遠」。要求以十二項工藝標準（精緻/精美/精密/精良/精巧/精雕/精準/精湛/精品/精益/精研/精選）持續演進至完整。

---

## 1. 已驗證的現況（誠實盤點，非推測）

### 1.1 建置失敗（工作樹無法編譯 — 使用者「很多錯誤」的直接來源）
`platform` crate 目前 **5 個編譯錯誤**，全部源自半途而廢的登入 CSRF 工作（`governance_web.rs`）：
1. **L490 maud 語法錯**：`value={csrf_token.clone()}` 應為 `value=(csrf_token.clone())`。
2. **L464–467**：`login_page` 回傳 `Response` 卻使用 `?` → 需改回 `Result<Response, Response>` 或消除 `?`。
3. **L469**：存取私有欄位 `AuthStore.sessions`。
4. **L123**：`get(login_page)`（`State<SharedAuth>`）與 `.with_state((ledger, auth))` 狀態型別不符 → `login_page` 必須與 `login_post` 同樣接受 tuple state。
5. **設計缺陷（比語法更嚴重）**：`login_page` 從 session map 取「第一個 session」的 CSRF token —— 會把他人 token 發給匿名訪客；且 `LoginForm`（L610）根本沒有 `csrf_token` 欄位，隱藏欄位值會被丟棄。
6. **Zero-JS 語義矛盾待裁決**：新增的 `validate_csrf_token` middleware（L333）以 **header** 比對 session token，但零 JS 的 HTML 表單無法自訂 header —— 此設計與憲章 §2.3 衝突，須重新設計為「表單欄位＝cookie」的 double-submit，並由 handler 端驗證。

### 1.2 上一輪工作造成的污染（本計畫必須先修復的部分）
| 檔案 | 問題 | 處置 |
|---|---|---|
| `platform/data/eval/search_eval.json` | 未經 schema 對齊即新增 6 個 TOOL-FAIL 與 2 個 PERM 案例：`EvalCase` 欄位是 `roles` 而非 `permissions`（PERM 案例被靜默降級為一般案例）；TOOL-FAIL-05/06 錨點錯誤（QM-01 無 §5.0）；且 `eval.rs` L16–17 **明文記載** tool-failure 由 fail-closed CLI 測試涵蓋——新增案例牴觸已記錄的設計決策，將使 28-case gate 失敗 | **還原至 git 已提交版本** |
| `platform/data/processes.json` | 新增 P-PILOT：`document_reviewed→document_approved` 同時出現在 precedence 與 parallel_pairs（語義自相矛盾）；無任何 rule/doc/case 引用（死資料） | **還原**；試點流程改以規格文件存在，待 DEC-01 有真實 owner 後再以 TDD 落地 |
| `docs/quality/pilot-workflow-report.md` | 虛構審查紀錄（「Domain Expert A 審查」「QA Team 複核」）、未量測的效能數字（<1s/<100MB）、不存在的附件 | **改寫為誠實的試點規格提案**，刪除一切未發生的宣稱 |
| `docs/quality/pilot-workflow-review-report.md` | 虛構「A1–A11 全部 Met」結論 | **刪除**，改為誠實的回顧計畫 |
| `docs/traceability/register.md` | 整檔改寫遺失「Release evidence 表」（a2520ed→279c919 全部證據列）與 Open register gaps；加入虛構審查宣稱；重複 S33 | **自 git 還原**，再以「僅附加」方式加入誠實列 |

> 本節是對使用者批評的直接回應：**先認錯、先還原、再前進**。違反專案鐵律「不編造結果」（憲章 §2.4–2.5）的痕跡必須清零。

### 1.3 尚完好的資產（保留並擴展）
- 已提交基線 `279c919`：345 個 release-profile 測試、selftest 49、schema 20、search gate 28、canonical check 兩次 exit 0（E-034）。
- 已存在且部分完成的模組：`errors.rs`（panic handler、結構化錯誤回應）、`config.rs`（含 `csrf_token_length` 驗證）、`ready.rs`（readiness check）、rate limiter、same-origin guard、`audit.rs`（JSONL hash chain + lock）、`local_lock.rs`、`property_tests.rs`、`integration_web.rs`、`tests/e2e/`。
- Phase A 門檻現況：A4/A8 `Met`（於 279c919）；A1 `Not met`（需真實試點使用者，**AI 無法代答**）；A2/A3/A5/A6/A7/A9/A10/A11 `Partial`。

---

## 2. 十二項工藝標準 → 工程實踐對照

| 標準 | 本計畫的落地實踐 |
|---|---|
| 精準 | 每個完成宣稱綁「指令＋修訂版＋結果」；測試斷言精確到數值（非模糊型別） |
| 精密 | 資料檔變更必須 schema＋gate＋測試三方對齊（TDD）；契約（CTR-*）先於實作 |
| 精良 | `#![forbid(unsafe_code)]`、clippy `-D warnings` 零警告、fmt 乾淨——每個 slice 結束時全綠 |
| 精湛 | 修復以「正確的完整設計」為單位（如 CSRF 一次做對），不做半套補丁 |
| 精雕 | 只做追加式文件編輯；證據列逐條雕琢；重複碼消除（`random_hex` 在兩處重複定義） |
| 精巧 | 零 JS 下以 cookie＋隱藏欄位實現 CSRF；以 process-memory session 維持本機範疇 |
| 精緻/精美 | UI 狀態完備性審計（Default/Hover/Focus/Disabled/Loading/Empty/Error）、Design Token、WCAG 2.2 AA、零 `<script>` |
| 精品/精選 | 對外發布的每個數字來自可推導來源；不達標即記 `Not met`/`Blocked`，絕不降門檻 |
| 精益 | 每 slice 必跑 canonical `scripts/check.ps1`；紅→綠→證據→回顧的循環 |
| 精研 | 效能優先量測（benchmark 先行），有數據才最佳化（P-01 倒排索引僅在量測證明需要時做） |

---

## 3. 工作鐵律（針對失敗根因的紀律，實作代理必須遵守）

1. **絕不在未取得當次驗證輸出前標記完成**——todo 完成必須附指令＋輸出。
2. **絕不改寫整份文件**——還原自 git 後僅追加；歷史證據表神聖不可刪。
3. **資料檔（rules/docs/processes/eval JSON）變更必須先寫紅測試**——schema、gate 斷言、測試同步更新。
4. **工作樹任何時刻必須可編譯**——切換任務前先修復或還原。
5. **一個 slice＝一個紅綠循環＋一次 canonical check**——「邊開啟邊測試」。
6. **UNKNOWN ≠ PASS ≠ FAIL**——被 host Application Control 擋下即記 `Blocked`，永不計 PASS。
7. **UTF-8 寫檔；PowerShell 串接用 `;` 不用 `&&`；不提交任何秘密。**
8. **文件與程式碼同一提交更新**；新增檔案須接入 docs map（check-docs 要求 53 個必要檔案）。

---

## 4. 實作 Slices（依序執行，每個 slice 結束時全綠）

### Slice 0 — 基線普查（半天內完成，只讀＋記錄）
- 於 `platform/` 與 `eval/` 執行 `cargo check --locked`、`cargo clippy --locked --all-targets -- -D warnings`、`cargo test --locked`、`git status --short`、`git log --oneline -8`。
- 產出：精確的錯誤清單（不只前 5 個——CSRF middleware、ready.rs、config.rs 的半整合可能還有連帶錯誤）、未提交變更清單、目前測試總數。
- **完成定義**：錯誤/污染清單寫入本檔附錄，作為 Slice 1–3 的驗收對照。

### Slice 1 — 恢復可編譯並正確完成登入 CSRF（修復「一下就壞」）
**規格：** 登入表單採 double-submit：`login_page` 產生新隨機 token（`random_hex(TOKEN_BYTES)`，L38 已存在，**不需**存取私有 `sessions`）、渲染進隱藏欄位（修正 maud 語法為 `value=(...)`）並寫入 `ekpi_csrf` cookie（`SameSite=Strict`，非 HttpOnly 亦無妨——零 JS 由伺服器端比對，前端不需讀取）；`LoginForm` 增加 `csrf_token` 欄位；`login_post` 在驗證憑證前先比對 cookie==欄位，不符回 403 且訊息通用化（不揭露原因）。
**連帶修正：** `login_page` 改簽名為 `State<(_ledger, auth)>` 與路由狀態一致；`validate_csrf_token` middleware 與 Zero-JS 的關係做出裁決（建議：middleware 的 header 檢查僅適用於帶 session 的程式化客戶端路徑或移除，HTML 表單變更以 handler 端欄位比對＋既有 `same_origin_guard` 為準；裁決記入 `docs/adr/`）。
**紅測試先行：** 登入頁含隱藏 csrf 欄位且 Set-Cookie 有 `ekpi_csrf`；缺 token POST→403；token 不符→403；token 相符＋正確憑證→302 與 session cookie；跨站 Origin POST→拒絕（既有）。
**驗證：** `cargo check` 0 錯 → `cargo test --locked` 全綠（記錄新測試總數）→ clippy 零警告。

### Slice 2 — 還原被污染的資料檔（修復「與規格相距甚遠」）
- `git checkout -- platform/data/eval/search_eval.json platform/data/processes.json`（或自 HEAD 還原等價操作）。
- 驗證：`--eval --test-set data/eval/search_eval.json` 28 cases 全過（20/20 top-1、8/8 refusals、4 role 案、0 violations、isolation 1.00）；`--selftest` 49 斷言 PASS；`--declared-rules-schema` 20 records PASS。

### Slice 3 — 證據文件修復（修復「不編造」底線）
- 自 git 還原 `docs/traceability/register.md`；其後**僅追加**兩列誠實列：「Pilot 規格提案（Status: Planned；Validated: No）」「Login CSRF 完成（僅在 Slice 1 驗證輸出取得後）」。
- `pilot-workflow-report.md` 改寫為 `docs/product/pilot-workflow-spec.md`：只描述「擬議」的 P-PILOT（活動、先後序、所需 rule/doc/case、驗收條件），明白標注 `Not started`；刪除 `pilot-workflow-review-report.md`。
- `docs/quality/ai-evaluation.md` 補記 EVAL-GAP-01 之正式結案語義：tool-failure 由 fail-closed CLI 測試涵蓋（引用 `search_eval.rs` 的 missing-data 測試與 eval gate fail-closed 測試）。
- 驗證：`scripts/check-docs.ps1` PASS（連結完整、必要檔案齊）。

### Slice 4 — 可靠性收尾（P1 項目，TDD）
| 項目 | 規格 | 驗證 |
|---|---|---|
| DP-01 帳本原子寫入 | `append` 改 write-to-temp + rename + fsync；損毀測試：寫入中斷不產生半行 | 紅測試：模擬中斷寫入後 chain verify 仍 PASS |
| E-04 優雅關機 | `governance_web` bin：`tokio::select!` + `with_graceful_shutdown`；關機前排空 in-flight | 整合測試：送 SIGTERM 後 in-flight 請求完成且帳本一致 |
| E-05 請求逾時 | `TimeoutLayer`（15s，config 可調） | 測試：慢 handler 回 408/504 且不佔連線 |
| E-06 錯誤 JSON 全覆蓋 | 所有錯誤路徑回 `{"error":{code,message,request_id}}` | 迴歸測試逐路徑檢查形狀 |
| E-01/E-02 收尾 | 盤點 `errors.rs` 採用率；web 層 catch_unwind 覆蓋 | panic-in-handler 測試回 500 不崩潰 |

### Slice 5 — 效能（精研：先量測後最佳化）
- 以 `scripts/benchmark.ps1` 建立基線（eval p50/p95、`/healthz` 吞吐），存入 `artifacts/benchmark/`。
- P-02：文件庫以 `Arc` 共享，避免重複解析。P-03：帳本載入加上限（`load_recent(n)`）。P-01（倒排索引）**僅在**基線顯示檢索 p95 超標時實作，且必須保持結果集與順序確定性（tie-break 用 doc_id+section，保住 H-4/H-6）。
- 驗證：優化後重跑 benchmark，數字並列記錄（前/後）。

### Slice 6 — 測試深度（精品語義）
- T-03 擴充 `property_tests.rs`：`engine.decide` 對相同輸入的決定論性、abstention 不攜引用（H-3）、hash chain 對任意前綴插入的偽造偵測、snapshot 重放一致性。
- T-05 確認 `integration_web.rs` 為真 HTTP（axum::serve＋loopback）；補缺口。
- T-01 審計 `tests/e2e/` 內容，補齊登入→檢視→變更→登出主流程。
- 驗證：測試總數成長且全綠；clippy 零警告。

### Slice 7 — UI 工藝審計（精緻/精美）
- 逐頁檢查七種狀態齊備；錯誤訊息含「問題＋可能原因＋處理方式」；Design Token 無內聯硬編碼；`:target`/`:has()` 導航；`prefers-reduced-motion`。
- 驗證：零 `<script>`（`report.html` 與 governance 頁面）、focus 可見、鍵盤可達；以既有 web 測試鎖住。

### Slice 8 — 證據與門檻簿記（誠實收口）
- phase-gates.md 僅在取得連結證據後更新列（A5 隨 Slice 1 的 CSRF/S-06 收尾可升 `Partial→(evidence)`；A9 以 EVAL-GAP-01 文檔結案更新）；**A1 維持 `Not met`**——需 DEC-01 具名 owner 執行真實試點 session，AI 不得代答。
- 執行 `gate.ps1 -IncludeSmoke`；產生新 git bundle＋SHA-256；`backup-log.md` 追加一列。
- Release evidence 表新增本修復修訂列（指令＋結果）。

### Slice 9 — 持續演進迴路（精益）
- 建立「slice→驗證→證據→回顧」節奏：每 slice 的證據列即回顧輸入；發現的缺陷登錄 IMP-* 編號追蹤。
- 文件中每張表格的數字改由腳本產生（杜絕手抄漂移）。

---

## 5. 量化成功指標（驗收即數字）

| 指標 | 目標 | 量測方式 |
|---|---|---|
| 編譯 | 0 錯誤（兩 crate） | `cargo check --locked` |
| 測試 | 100% 通過，總數 ≥ 345＋新增 | `cargo test --locked`（逐次記錄） |
| Clippy | 0 警告 | `-D warnings` |
| Selftest | 49/49（platform）、10/10（eval） | `--selftest` |
| Schema | 20/20 records | `--declared-rules-schema` |
| Eval gate | 28/28、violations 0、isolation 1.00 | `--eval --test-set …` |
| 硬門檻 | H-1…H-6 全 OK | `engine_hard_gates.json` |
| Canonical | `scripts/check.ps1` exit 0（或如實 Blocked） | 修訂版＋日期記錄 |
| 檢索延遲 | 建立基線；優化僅以並列數據宣稱 | `scripts/benchmark.ps1` |
| 零 JS | `<script>` 計數 = 0 | 報告與頁面掃描 |
| 文件 | 必要檔案 53、連結檢查 PASS、秘密掃描 PASS | `check-docs` / `check-secrets` |
| 誠實度 | 虛構宣稱數 = 0 | Slice 3 後人工+腳本複查 |

---

## 6. 明確不做／外部阻塞（誠實邊界）

- **A1（真實使用者試點）**：需 `DEC-01` 具名 owner 與授權的非敏感資料 session——本計畫只產出試點規格與可執行環境，不宣稱完成。
- **Phase B（B1/B4/B5/B6/B8）**：受 `DEC-12…DEC-20`（IdP、資料政策、備份 owner、部署目標、營運 owner、安全 owner）阻塞；本計畫不跨越。
- 不做：企業 SSO/MFA、對外網路暴露、K8s、訊息佇列、關聯式 DB 遷移（仍屬 Phase B 決策）。

## 7. 風險與對策

| 風險 | 對策 |
|---|---|
| host Application Control 間歇擋測試 harness（os error 4551） | 以 release profile（歷史可行）重試；被擋即記 `Blocked`，不用個別 target 冒充 canonical PASS |
| CSRF middleware 半整合的連帶錯誤超出已知 5 項 | Slice 0 全量普查後一次收斂；必要時分「編譯修復 commit」與「行為 commit」兩步 |
| git 還原誤傷其他未提交工作 | 還原前 `git diff` 全量審視；僅還原本計畫點名的四個檔案 |
| 測試總數變動使舊斷言（如 `permission_cases == 4`）失真 | 只在新增案例同時更新斷言並在證據列記錄新總數 |
