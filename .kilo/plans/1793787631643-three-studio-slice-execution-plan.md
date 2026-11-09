# 三大工作室 SLICE 執行計畫（PHASE-G1..G3）— v4 定稿

> 計畫編號：1793787631643｜日期：2026-11-09（v4：樹態核查完成，擴充點全部落點到檔案/函式級）｜狀態：implementation-ready
> 上位架構 SSOT：`.kilo/plans/1792865422970-three-studio-governance-platform.md`（v3.1 §7/§17/§20/§22）
> 執行者注意：衝突以上位為準；每切片先做 §0 樹態核查再立 RED。

---

## 0. 樹態核查（2026-11-09 全數完成；擴充點已落到檔案級）

| # | 事實 | 證據 | 對切片的落點 |
|---|---|---|---|
| F1 | 平行線已交付 `transactions.html`：GET 搜尋（`q`）、分頁（`prev_page`/`next_page` 為完整 URL）、零結果文案與空庫文案已互異、「清除搜尋」連結 | platform/templates/transactions.html L28-43、L61-70、L87-99 | G1.2 縮小為：驗證 handler 組分頁 URL 時**保留 `q`**；detail 返回連結保留 `q`+page；勿重做搜尋/分頁 |
| F2 | `states.html`＝互動態標本頁（E3.7：Hover/Focus/Invalid），非資料八態 | states.html L17、L34-60 | G1.1 在此頁增「資料八態」標本區＋四真實頁矩陣測試 |
| F3 | UDM 已有 `UnifiedOrder`/`InventoryRecord`/`ProductRecord`＋`MoneyCents(i64)`＋`Provenance(platform,contract_version,field_source)` | platform-hub/crates/udm/src/lib.rs | G2.3 延伸型別，補 Customer/Payment/Invoice |
| F4 | eventbus 已有 outbox/inbox：`OutboxEvent.attempt`、`InboxEvent(consumer_id,event_id)` 去重、`EventStore` trait、`KafkaEventStore` 為保留 stub | platform-hub/crates/eventbus/src/lib.rs | G2.1 加 `deliver_with_backoff`＋DLQ；stub 不動 |
| F5 | channels 已有 `ChannelAdapter`/`PendingContractAdapter`（契约未到一律 NotImplemented，不猜事實）/`ColumnContract.validate_header` | platform-hub/crates/channels/src/lib.rs | G2.2 以 decorator 包 adapter；語義不變 |
| F6 | **agent-policy 受 DEC-15 管制**：`AutoApprovalGate::advisory_only()` fail-closed；High/Critical/paid/Confidential/Restricted 一律 RequireHuman | platform-hub/crates/agent-policy/src/lib.rs | G2.5 維持 fail-closed，僅擴充速率/草稿/審計標記 |
| F7 | **audit_log 欄位**：log_id/transaction_id/event_type/event_data(JSON)/performed_by/performed_at | migrations/0001_baseline_schema.sql | G3.1 四要件映射：期間=performed_at 區間、控制=event_type→控制碼、完整性=kem 鏈引用、來源=event_data |
| F8 | **kem gov.rs＝知識主張治理**（lint L1..L5：無證據發布/無後繼/信心過期/標題近似/未標 superseded；`Finding{rule,claim_id,detail}`；STALE_THRESHOLD=0.45 為信心分數非金額） | crates/kem/src/gov.rs | G3.2 控制庫與其**不衝突**（SOA 控制≠知識主張）；但孤立控制檢查**復用 Finding/lint() 模式** |
| F9 | 角色為 DB 動態（roles/permissions/user_roles；role_code 驅動；demo-finance/demo-viewer 存在） | core/src/permission.rs；migrations/0002 | G1.3 以 role_code 對映＋fallback 測試 |
| F10 | 平行線已新增 `api_catalog/config/logs/test.html` 與 `transaction_detail.html` | platform/templates/ | S2 UI 演進掛該線；G1.2 只補 detail 返回語義 |

## 0.5 重構判定框架（不亂覆蓋亂刪的運作機制）

**五步程序**（每個候選必走）：①特徵化（PIN）測試先行 → ②公理檢查（AX-1..AX-7） → ③定理量測（下表） → ④風險定級（單向門=ADR+擁有者；雙向門=architect 核＋預寫回滾） → ⑤驗證閉環（四關＋PIN 不變＋新性質測試＋追溯更新）。

| 原則/定理 | 量測方式 | 觸發門檻 |
|---|---|---|
| Connascence | 跨檔聯生盤點 | 高強度＋低局部性→拆分 |
| 認知複雜度 | 函式級審計 | >25 且無法表格驅動 |
| LCOM/扇入扇出 | 公開面盤點 | 扇入>5 且職責>1 |
| Big-O 實證 | EXPLAIN QUERY PLAN＋stress | 報表非 O(log n + k) |
| Little's Law | 批次/佇列參數 | 與延遲預算失配 |
| 冪等代數 | at-least-once＋冪等鍵≡exactly-once | 消費路徑無鍵 |
| CAP 定位 | 每跨系統流標注一致性 | 未標注=設計債 |
| STPA 控制論 | 證據鏈=回饋控制器 | 控制無回饋量測 |
| 適應度函數 | CI：零 JS/PUA/MT/p95/CSRF | 回退即擋 |
| 語義漂移 | 同概念多型別對照 | 邊界無單位契約 |

**已裁決候選**（結論固化，勿再議；每項附計算證據）：

| 候選 | 量化證據（2026-11-09 實測） | 判定定理應用 | 裁決 |
|---|---|---|---|
| `api.rs` 模組化（R1） | ≈4200 行；23 路由；14 個 POST 表單結構各自內聯 CSRF 檢查（`pub csrf_token` ×14；`if form.csrf_token != session.csrf_token` 重複 ≥11 處） | **Connascence-of-algorithm**：同演算法複製 11+ 處、局部性極低（跨 4000 行）→ 高強度低局部性=拆分候選；但**扇入=23 路由共用 AppState**，屬單向門 | **最後排序**：先 PRE-R1 PIN 測試；GAP-08 分歧解鎖後以「extractor/middleware 化」食譜執行（見 PRE-R1b） |
| `web/index.html` 靜態基線 | 已有 Superseded 頭註 | 資訊價值>維護成本 | 保留不刪 |
| hub `MoneyCents(i64)` vs 根線 minor(i128)+rate(10^4) | 兩型別並存；UDM 序列化往返測試在 | **語義漂移偵測**：同概念（錢）兩單位、邊界無契約 | **不合併**（單向門、無對帳數據支持）；G2.3 補單位契約＋explicit 轉換＋防混用適應度 |
| `KafkaEventStore` stub | 無流量證據 | 無觸發條件 | 不動（未來插槽） |
| `agent-policy` DEC-15 | `advisory_only()` fail-closed 已驗（3 測試） | 決策權在擁有者（ESC-4 級） | 維持；G2.5 僅擴充 |
| engine 五個 domain stub | MT exclude 5 項 | 屬並行線工作面 | 由 search/sync 線接管補測 |
| gov.rs `f64`（STALE_THRESHOLD=0.45） | 信心分數，非金額路徑 | INV-01 範圍=金額 | 不違規；約束措辭精確化 |

**定理計算實例（框架運作示範）**：
- **Little's Law（G2.4/G1.4 批次參數）**：實測每列寫入 ≈0.3ms（90s/300k 列，2026-11-07 stress seed）；批次 500 列/語句 → 600 語句 × ~3ms ≈ 2s，實測 seed ≈2s ✓ 模型吻合。派生：待覆核批量裁決分批 ≤500 列/POST，避免請求級 W 超出 SLA。
- **冪等代數（G2.1）**：eventbus 已有 `InboxEvent(consumer_id,event_id)` 去重＋`Duplicate` 錯誤 ⇒ 消費側 at-least-once+dedup ≡ exactly-once 效果**已成立**；缺口在 channels 下游（fetch/push 無冪等鍵）與 DLQ——故 G2.1 只補「退避排程＋DLQ＋重放」，不重做去重。
- **STPA 控制論（G3.1/G3.3）**：audit_log 有回饋量（event_type/performed_at/event_data）但**無防篡改連結**（無鏈雜湊欄）→ 控制器量測點與執行點斷鏈=控制損失風險 → G3.1 以 kem 鏈引用補上、G3.3 將「鏈完整性」設為連續回饋指標。
- **CAP 定位表（跨系統流標注，防靜默降級）**：VS-01 帳務=CP（單庫事務）；VS-05 同步=AP- eventual（重試+對帳收斂）；VS-06 事件=AP（outbox 至少一次）；VS-07 證據=CP（哈希鏈不可變）；VS-09 應變=AP（可重建優先）。

**適應度函數現值（全綠基線，回退即擋）**：零 JS script=0 ✅｜PUA=0（已修復檔）✅｜MT 得分 money/engine 100% ✅｜warm p95 422ms≤500ms ✅｜CSRF 10/10（14 表單全覆蓋，`post_rejects_wrong_csrf_token` 等 PIN 在）✅｜clippy allow=0 ✅｜金額 f64=0（money.rs 僅文件註提及 f64）✅。

## 1. 硬性約束（每切片）

Zero-JS（script=0）；**金額** i64/i128 minor 禁浮點；GET 讀/POST 寫（CSRF 10/10 維持）/PRG；`#![forbid(unsafe_code)]`；clippy 零警告不新增 allow；UTF-8 無 BOM（PowerShell 用 `[System.IO.File]::WriteAllText`，禁 `>`）；UNKNOWN≠PASS≠FAIL；切片完成即 PLAN_REGISTER addendum＋上位 §22 追溯更新；四關（fmt/clippy -D warnings/test --locked/audit）EXIT=0。

## 2. 任務清單（依序；每項含擴充點與 DoD）

### PHASE-G1 — S1 工作台卓越化
- [ ] **G1.1 八資料態全覆蓋**（WP-08）
  - 擴充點：`states.html` 增「資料八態」標本區（空/零結果/載入中/部分/過期/無權限/錯誤/衝突——文案互異且各含下一步連結）。
  - 測試：index/transactions/import/drafts 四頁八態矩陣（註：transactions 之零結果態平行線已做，測試直接覆蓋即可）。
  - DoD：矩陣測試綠＋states.html 對照區 DOM 斷言。
- [ ] **G1.2 篩選持久化驗證＋detail 返回語義**（WP-10）
  - 擴充點：F1——只補缺口。
  - 測試：①帶 `q` 搜尋後翻頁，`next_page` URL 含 `q`（若 handler 已保留→測試即 PIN；未保留→RED 補齊）；②`transaction_detail` 返回連結保留 `q`+page。
  - DoD：兩測試綠；不重寫既有搜尋/分頁。
- [ ] **G1.3 角色起始頁＋KPI 就地解釋**（WP-05/04/02）
  - 作法：role_code→首屏區塊（finance=待覆核隊列；viewer=報表入口；未知=預設總覽）；KPI 卡附定義/時點/來源註（內容接 ubiquitous_language）。
  - DoD：demo-finance/demo-viewer/ADMIN 三測試＋未知角色 fallback 測試。
- [ ] **G1.4 批量裁決＋差異 highlight**（WP-07）
  - 作法：待覆核列表多選批量批/駁——單一 POST、逐列審計（沿用 audit_log event_type 約定）、署名、部分失敗語義明確（回報逐列結果）；異常列 highlight；自我裁決禁止。
  - DoD：批量部分失敗測試＋自我裁決 403 測試。

### PHASE-G2 — S2 中樞韌性化（platform-hub；於其目錄內獨立驗證）
- [ ] **G2.1 冪等排程＋DLQ**（HP-05）
  - 擴充點：`EventStore` trait＋`attempt` 欄位。
  - 作法：`deliver_with_backoff`（指數退避+抖動；上限後入 DLQ：event_id/錯誤/時間）；DLQ 檢視與重放（重放需署名）。Kafka stub 不動。
  - DoD：重複事件不重放（驗證既有 `Duplicate` 語義）、超限入 DLQ、署名重放成功三測試。
- [ ] **G2.2 熔斷＋半開**（HP-05）
  - 作法：`CircuitBreakerAdapter` decorator 包 `ChannelAdapter`（成功率窗→Open→HalfOpen 探測→Closed）；不改 trait、不觸 `PendingContractAdapter` 語義。
  - DoD：故障注入測試（FAULT-07 落地）。
- [ ] **G2.3 UDM 補實體＋單位契約**（HP-03/04）
  - 作法：新增 `CustomerRecord`/`PaymentRecord`/`InvoiceRecord`（沿用 Provenance/MoneyCents）；權威源＋衝突裁決欄位；補 MoneyCents↔根線 minor 單位契約文件＋explicit 轉換函式＋防混用適應度測試。
  - DoD：round-trip×3＋映射契約（缺欄硬失敗，沿用 `ColumnContract` 語義）＋單位契約測試。
- [ ] **G2.4 對帳報告**（HP-06）
  - 作法：每同步流產出件數/金額總計/差異樣本（JSON＋零 JS 頁渲染；UI 掛 api_logs.html 演進由該線定，本切片只交付資料結構與計算）。
  - DoD：fixture 同步流→數字可重算驗證。
- [ ] **G2.5 agent-policy 擴充**（HP-09）
  - 作法：維持 DEC-15 fail-closed；代理寫入一律產草稿；速率/資料類別上限（沿用 `RiskLevel`/`DataClassification` 枚舉）；請求帶 agent 標記入稽核。
  - DoD：越權拒絕＋超速 429＋審計標記三測試。

### PHASE-G3 — S3 治理稽核就緒
- [ ] **G3.1 證據四要件導出**（GP-04）
  - 擴充點：F7 audit_log 既有欄位＋kem 鏈（`capture_source` 之 SHA-256）。
  - 作法：導出（CSV/頁）附「控制（event_type→控制碼映射）/期間（performed_at）/完整性（鏈驗證結果）/來源（event_data）」區塊。
  - DoD：渲染測試＋「一控制全鏈追溯」演練記錄。
- [ ] **G3.2 控制庫跨框架映射**（GP-02）
  - 作法：一控制↔多框架（ISO/內控/銀行合規）映射表；與 kem capture 雙向連結；孤兒檢查**復用 gov.rs 之 `Finding`/`lint()` 模式**（rule/claim_id/detail → rule/control_id/detail）。
  - DoD：映射完整性測試（孤兒控制=0）。
- [ ] **G3.3 CCM 三指標**（GP-05）
  - 作法：鏈完整性（日驗證）/登入異常率/金額不變量違反=0——零 JS SVG 儀表。
  - DoD：fixture 事件流→數值重放斷言。
- [ ] **G3.4 外部唯讀門戶**：僅規格（OQ-GOV-04 憑證模型未決；阻塞）。

### 平行安排（非切片，只加測試不改結構）
- [ ] **PRE-R1a**：api.rs 23 路由特徵化測試集（請求→狀態碼+關鍵 DOM 斷言），作未來模組化安全網；可與 G1 並行。
- [ ] **PRE-R1b（可選，框架首次應用示範）**：CSRF 統一 extractor——量化證據：同一檢查複製 ≥11 處（connascence-of-algorithm）。食譜：以 axum extractor/middleware 收斂為單一實作（行為不變）；前置=PRE-R1a PIN 全綠；驗證=PIN 不變＋四關＋`post_rejects_wrong_csrf_token` 等既有測試不改一字通過。雙向門（可回滾），architect 核可即做；不做亦不阻塞任何切片。

## 3. 執行順序

G1.1 → G1.2 → G1.3 → G1.4 → G2.1 → G2.2 → G2.3 → G3.1 → G2.4 → G2.5 → G3.2 → G3.3。單切片單提交；每切片完成即更新上位 §22 追溯矩陣（零孤兒）＋PLAN_REGISTER addendum，不批量補帳。

## 4. 驗證計畫

- 每切片：四關 EXIT=0＋RED→GREEN 證據＋（涉既有行為者）特徵化不變。
- 階段閘門（G1/G2/G3 各一次）：FL-01..04 端到端演練各一、FAULT 表更新、追溯零孤兒、適應度函數全綠（零 JS/PUA/MT/p95/CSRF）。
- platform-hub 於 `platform-hub/` 內獨立執行同標準（workspace 隔離）。

## 5. 風險與阻塞

| 項目 | 處置 |
|---|---|
| 平行線同檔衝突（templates/、kem、search） | 樹態核查先行＋board 協調；門檻量測避開對方不穩定點（隔離 CARGO_TARGET_DIR） |
| R1 模組化（GAP-08 分歧） | 最後排序＋PRE-R1 前置＋ADR；本計畫不含結構重構 |
| DEC-15 / OQ-GOV-01..04 / DEC-21 | 擁有者決策；本計畫一律 fail-closed 或僅規格 |
| GAP-04 fuzz（ASan DLL）、OQ-F1/F2（11-19） | 不阻塞；UNKNOWN 紀律 |

## 6. 明示不在範圍

KafkaEventStore 實作、SSE（OQ-GOV-02）、PostgreSQL 遷移（OQ-GOV-01）、HA（OQ-GOV-03）、api.rs 結構重構（僅 PRE-R1 測試）、輔翼 ERP 本體修改、授權/合約行為、MoneyCents↔minor 型別合併（僅契約）、外部門戶實作（G3.4 僅規格）。

---

# PHASE-R — S1 檢索面 RAG 升級（BM25＋混合向量，2026-11-09 增補）

> 觸發：擁有者指出「沒用到 RAG 相關技術」。核查結論：S3（kem）已有完整混合檢索（`crates/kem/src/search.rs` BM25、`embed.rs` embedding 池、`main.rs:452` hybrid_fuse＋BM25-only 誠實降級，58 lib tests 綠）；**缺口在 S1**——`platform/src/search.rs`（81 行）僅子字串比對，`/api/search` 的 `score` 恆 1.0（`cross_module_api.rs:265-280`）。
> 定義界線：本階段的 "RAG"＝**檢索排序增強**（BM25＋向量混合、誠實降級、via 溯源）；**不含 LLM 生成**（與 kem `ask` 一致：只回參照不注入生成；LLM 生成屬 FREE_AI_ROUTING 線的未來決策）。

## R.0 已核實設計事實（2026-11-09，皆附檔案行號）

| 事實 | 證據 | 對設計的影響 |
|---|---|---|
| `kem::embed::embed_batch(&[EmbedEntry], &[String])` 為**文字泛用**（非 Claim 綁定）；`cosine` 泛用 | kem/src/embed.rs:71、:46 | 向量腿**直接重用**，不重寫 embedder |
| `kem::search::Index`/`hybrid_fuse` **Claim 綁定**（吃 `&[Claim]`、產 `Hit{status,confidence}`） | kem/src/search.rs:61、:136、:172 | BM25＋融合須平台自寫**泛用版**，語義對齊 kem（K1/B、IDF、α 正規化、拒絕地板、via 誠實標記） |
| kem tokenizer `w.len() > 1` **丟棄單一 CJK 字** | kem/src/search.rs:53-58 | 平台版 tokenizer 支援 CJK 單字 token（中文檢索改進點） |
| `platform → kem` 路徑依賴**零新增外部 crate**（kem 依賴＝serde/serde_json/thiserror/chrono/sha2/tokio/reqwest，platform 全已具備） | crates/kem/Cargo.toml vs platform/Cargo.toml | 採用路徑依賴重用 embedder；耦合成本≈0 |
| embedder 池＝OVH bge-m3（免金鑰、批次、1024 維）→ Ollama 本地 fallback；全不可達＝誠實 Err | kem/src/embed.rs:27-43 | 測試環境（無 embedder）走 BM25-only＝確定性測試路徑 |
| 平行線擁有 `search.rs`/`cross_module_api.rs`（Slice 13），目前穩定（kem 58 tests 綠） | 2026-11-09 樹態 | 實作前樹態核查＋board 宣告 |

## R.1 切片（每片樹態核查先行＋四關）

- [ ] **RAG-S1 詞法腿**：新增 `platform/src/bm25.rs`（僅 std，零新依賴）——`tokenize`（ASCII 詞組＋CJK 單字 token，文檔註明無字典分詞 tradeoff）、`BM25Index`（K1=1.2、B=0.75、Robertson IDF +0.5 平滑下限 0、`scores_all` 降序、`add_text` 維護 df/長度）；測試：稀有詞>常見詞、詞頻有序、未知詞 0 分、混合中英 tokenize。DoD：4 測試綠，現行行為零變更。
- [ ] **RAG-S2 混合引擎**：`platform/Cargo.toml` 增 `kem = { path = "../crates/kem" }`；`SearchEngine` 增 `bm25: RwLock<BM25Index>`＋`vectors: RwLock<HashMap<doc_id, (sha256, Vec<f32>)>>`；`add_document` 內嵌 `kem::embed::embed_batch(&kem::embed::default_pool(), ...)`（Err→無向量，照常索引——誠實降級）；新增 `search_ranked(query) -> Vec<(SearchDocument, f64, String via)>`：α·norm(BM25)+(1−α)·norm(cosine)（α=0.5、語義拒絕地板 0.55 對齊 kem 校準 2026-11-09），零詞法支持且 cosine<地板→丟棄；舊 `search()` 委託 ranked 丟分數（相容不變）；`delete_document` 重建 BM25＋刪向量；向量帶內容 SHA-256（kem §8 新鮮度規則），失配視同缺失。測試：注入向量 → via="hybrid"；弱向量（餘弦<地板、零詞法）→ 拒絕；空池注入（embedder 不可達）→ via="bm25" 確定性降級。DoD：單元測試全綠。
- [ ] **RAG-S3 端點接線**：`/api/search` 回真實融合分數；`SearchResult` 增 `via` 欄位（加法相容，不破壞既有消費端）；整合測試——fixture 語料排序、CJK 查詢、未知詞回 `[]`（非假分數）、無 embedder 環境 via="bm25"。DoD：全測試綠＋四關。
- [ ] **RAG-S4 文檔收口**：PLAN_REGISTER addendum #15、CHANGELOG 條目、master §22 增 RAG 列（WP 對照：可檢索性/語義查詢）、eval fixture 記錄（BM25 top-1 金測，沿用 kem `run_golden` 思路）。

## R.2 風險與約束

| 項目 | 處置 |
|---|---|
| 平行線擁有 `search.rs`/`cross_module_api.rs`（Slice 13） | 樹態核查＋board 宣告先行；量測避開不穩定點（隔離 `CARGO_TARGET_DIR`） |
| embedder 於 CI/本機不可達 | 降級即產品：BM25-only 為確定性預設路徑；hybrid 為增益（測試以注入向量覆蓋，如 kem refusal 測試模式） |
| 重啟遺失（in-memory 索引/向量） | 既有限制（現行 search 亦然），標註 `[LIMIT]`；持久化屬 PHASE-G4 評估 |
| Tantivy 編譯事故教訓 | 不引入新重依賴；kem 路徑依賴零外部成本 |
| 大語料效能 | BM25 O(語料×查詢詞)，工作台演示規模足夠；重規模屬 PHASE-G4 評估 |
| f64 分數 | 純排序分數非金額，不違 INV-01（與 kem 一致） |

## R.4 明示不在範圍（PHASE-R）

LLM 生成（`ask` 僅回參照的既有紀律）、Tantivy/Qdrant 重型索引（POC 另議）、索引/向量持久化、kem search.rs 去耦合重構（共享 crate 議題，待 kem 線穩定另案）。
