# 三大工作室 SLICE 執行計畫（PHASE-G1..G3）— v8

> 計畫編號：1793787631643｜日期：2026-11-10（v8：RP-6 全域盤點——Dev 根目錄未登錄文件群＋Kilo 快照區 ~70 份計畫（效能家族演化鏈完整現蹤）＋web/legacy-1420 並存事實；RP-7：Kilo 快照區 ~70 份計畫文件清點與登錄；RP-8：Dev 根目錄未登錄文件盤點；RP-9：web/legacy-1420.html 零 JS 適應度驗證）｜狀態：implementation-ready
> 上位架構 SSOT：`.kilo/plans/1792865422970-three-studio-governance-platform.md`（v3.1 §7/§17/§20/§22）
> 執行者注意：衝突以上位為準；每切片先做 §0 樹態核查再立 RED。

---

## 0.1 計畫目的與範圍

本計畫為「三大工作室（部門業務工作台 S1、API 中樞與資料管理 S2、治理稽核平台 S3）」的執行計畫，並包含兩個平行軌道：

- **PHASE-G1～G3（三大工作室執行）**：本計畫**直接執行**，包含 12 個切片。
- **PHASE-K（知識管理系統架構藍圖）**：本計畫**僅產出架構藍圖**，執行由「專案品質檢查與文件管理優化專案」負責。
- **PHASE-RP（輔翼ERP效能評估進度報告整合與演進）**：本計畫**僅產出架構藍圖與執行順序**，由「專案品質檢查與文件管理優化專案」承接實作。

**執行者請注意**：PHASE-K 與 PHASE-RP 不屬於本計畫的直接執行範圍，僅作為移交件（handoff）與參考。本計畫的執行範圍嚴格限於 PHASE-G1 至 G3。

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
| `web/index.html` 靜態基線 | Superseded 頭註（2026-11-04，314 行）；並行線另增 `web/legacy-1420.html`（180 行，1420 原版 UI 重現）＋`legacy-1420.css`——兩檔並存，零 JS 適應度待 RP-6⑤ 驗證 | 資訊價值>維護成本 | 兩檔均保留不刪；legacy-1420 納入零 JS 適應度檢查 |
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

> **完成狀態總覽（2026-11-10）**：PHASE-G1/G2/G3 共 12 切片＋PRE-R1a 全數實作完成並驗證（G1.1＋G2.x 見 addendum #12；G1.2–G1.4＋G3.x 見 #13；全工作區 246 passed / 0 failed）。僅 G3.4（僅規格，阻塞）與 PRE-R1b（可選）未執行。

### PHASE-G1 — S1 工作台卓越化
- [x] **G1.1 八資料態全覆蓋**（WP-08）
  - 擴充點：`states.html` 增「資料八態」標本區（空/零結果/載入中/部分/過期/無權限/錯誤/衝突——文案互異且各含下一步連結）。
  - 測試：index/transactions/import/drafts 四頁八態矩陣（註：transactions 之零結果態平行線已做，測試直接覆蓋即可）。
  - DoD：矩陣測試綠＋states.html 對照區 DOM 斷言。
- [x] **G1.2 篩選持久化驗證＋detail 返回語義**（WP-10）
  - 擴充點：F1——只補缺口。
  - 測試：①帶 `q` 搜尋後翻頁，`next_page` URL 含 `q`（若 handler 已保留→測試即 PIN；未保留→RED 補齊）；②`transaction_detail` 返回連結保留 `q`+page。
  - DoD：兩測試綠；不重寫既有搜尋/分頁。
- [x] **G1.3 角色起始頁＋KPI 就地解釋**（WP-05/04/02）
  - 作法：role_code→首屏區塊（finance=待覆核隊列；viewer=報表入口；未知=預設總覽）；KPI 卡附定義/時點/來源註（內容接 ubiquitous_language）。
  - DoD：demo-finance/demo-viewer/ADMIN 三測試＋未知角色 fallback 測試。
- [x] **G1.4 批量裁決＋差異 highlight**（WP-07）
  - 作法：待覆核列表多選批量批/駁——單一 POST、逐列審計（沿用 audit_log event_type 約定）、署名、部分失敗語義明確（回報逐列結果）；異常列 highlight；自我裁決禁止。
  - DoD：批量部分失敗測試＋自我裁決 403 測試。

### PHASE-G2 — S2 中樞韌性化（platform-hub；於其目錄內獨立驗證）
- [x] **G2.1 冪等排程＋DLQ**（HP-05）
  - 擴充點：`EventStore` trait＋`attempt` 欄位。
  - 作法：`deliver_with_backoff`（指數退避+抖動；上限後入 DLQ：event_id/錯誤/時間）；DLQ 檢視與重放（重放需署名）。Kafka stub 不動。
  - DoD：重複事件不重放（驗證既有 `Duplicate` 語義）、超限入 DLQ、署名重放成功三測試。
- [x] **G2.2 熔斷＋半開**（HP-05）
  - 作法：`CircuitBreakerAdapter` decorator 包 `ChannelAdapter`（成功率窗→Open→HalfOpen 探測→Closed）；不改 trait、不觸 `PendingContractAdapter` 語義。
  - DoD：故障注入測試（FAULT-07 落地）。
- [x] **G2.3 UDM 補實體＋單位契約**（HP-03/04）
  - 作法：新增 `CustomerRecord`/`PaymentRecord`/`InvoiceRecord`（沿用 Provenance/MoneyCents）；權威源＋衝突裁決欄位；補 MoneyCents↔根線 minor 單位契約文件＋explicit 轉換函式＋防混用適應度測試。
  - DoD：round-trip×3＋映射契約（缺欄硬失敗，沿用 `ColumnContract` 語義）＋單位契約測試。
- [x] **G2.4 對帳報告**（HP-06）
  - 作法：每同步流產出件數/金額總計/差異樣本（JSON＋零 JS 頁渲染；UI 掛 api_logs.html 演進由該線定，本切片只交付資料結構與計算）。
  - DoD：fixture 同步流→數字可重算驗證。
- [x] **G2.5 agent-policy 擴充**（HP-09）
  - 作法：維持 DEC-15 fail-closed；代理寫入一律產草稿；速率/資料類別上限（沿用 `RiskLevel`/`DataClassification` 枚舉）；請求帶 agent 標記入稽核。
  - DoD：越權拒絕＋超速 429＋審計標記三測試。

### PHASE-G3 — S3 治理稽核就緒
- [x] **G3.1 證據四要件導出**（GP-04）
  - 擴充點：F7 audit_log 既有欄位＋kem 鏈（`capture_source` 之 SHA-256）。
  - 作法：導出（CSV/頁）附「控制（event_type→控制碼映射）/期間（performed_at）/完整性（鏈驗證結果）/來源（event_data）」區塊。
  - DoD：渲染測試＋「一控制全鏈追溯」演練記錄。
- [x] **G3.2 控制庫跨框架映射**（GP-02）
  - 作法：一控制↔多框架（ISO/內控/銀行合規）映射表；與 kem capture 雙向連結；孤兒檢查**復用 gov.rs 之 `Finding`/`lint()` 模式**（rule/claim_id/detail → rule/control_id/detail）。
  - DoD：映射完整性測試（孤兒控制=0）。
- [x] **G3.3 CCM 三指標**（GP-05）
  - 作法：鏈完整性（日驗證）/登入異常率/金額不變量違反=0——零 JS SVG 儀表。
  - DoD：fixture 事件流→數值重放斷言。
- [ ] **G3.4 外部唯讀門戶**：僅規格（OQ-GOV-04 憑證模型未決；阻塞）。

### 平行安排（非切片，只加測試不改結構）
- [x] **PRE-R1a**（✅ 並行線交付 2026-11-09：`platform/tests/characterization.rs`，6 tests PASS）：api.rs 路由特徵化測試集（請求→狀態碼+關鍵 DOM 斷言），作未來模組化安全網。
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

KafkaEventStore 實作、SSE（OQ-GOV-02）、PostgreSQL 遷移（OQ-GOV-01）、HA（OQ-GOV-03）、api.rs 結構重構（僅 PRE-R1 測試）、輔翼 ERP 本體修改、授權/合約行為、MoneyCents↔minor 型別合併（僅契約）、外部門戶實作（G3.4 僅規格）、**知識管理系統（D-OPS＋RAG-Transcendence）建置**（PHASE-K 為移交件，由「專案品質檢查與文件管理優化專案」承接執行）。

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

- [x] **RAG-S1 詞法腿**（✅ 2026-11-09）：新增 `platform/src/bm25.rs`（僅 std，零新依賴）——`tokenize`（ASCII 詞組＋CJK 單字 token，文檔註明無字典分詞 tradeoff）、`BM25Index`（K1=1.2、B=0.75、Robertson IDF 與 kem 同式——初版 0 地板曾致全共現詞零分，已修）、`scores_all` 降序、`add_text` 維護 df/長度；測試：稀有詞>常見詞、詞頻有序、未知詞 0 分、混合中英 tokenize。DoD：5 測試綠，現行行為零變更。
- [x] **RAG-S2 混合引擎**（✅ 2026-11-09）：`platform/Cargo.toml` 增 `kem = { path = "../crates/kem" }`；`SearchEngine` 增 `bm25: RwLock<BM25Index>`＋`vectors: RwLock<HashMap<doc_id, (sha256, Vec<f32>)>>`；`add_document` 內嵌 `kem::embed::embed_batch(&kem::embed::default_pool(), ...)`（Err→無向量，照常索引——誠實降級）；新增 `search_ranked(query) -> Vec<(SearchDocument, f64, String via)>`：α·norm(BM25)+(1−α)·norm(cosine)（α=0.5、語義拒絕地板 0.55 對齊 kem 校準 2026-11-09），零詞法支持且 cosine<地板→丟棄；舊 `search()` 委託 ranked 丟分數（相容不變）；`delete_document` 重建 BM25＋刪向量；向量帶內容 SHA-256（kem §8 新鮮度規則），失配視同缺失。測試：注入向量 → via="hybrid"；弱向量（餘弦<地板、零詞法）→ 拒絕；空池注入（embedder 不可達）→ via="bm25" 確定性降級。DoD：6 單元測試全綠。
- [x] **RAG-S3 端點接線**（✅ 2026-11-09）：`/api/search` 回真實融合分數；`SearchResult` 增 `via` 欄位（加法相容，不破壞既有消費端；直取 via="direct"）；整合測試——fixture 語料排序、CJK 查詢、未知詞回 `[]`（非假分數）、無 embedder 環境 via="bm25"。DoD：全測試綠＋四關。
- [x] **RAG-S4 文檔收口**（✅ 2026-11-09，addendum #17——原 #15 因與並行線撞號重編）：PLAN_REGISTER addendum #17、CHANGELOG 第十四輪、master §22 增 RAG 列（可檢索性/語義查詢）、跨線 stale fixture 修復（bankpay 鍵 5→10）。

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

## R.5 完成標記（2026-11-10 更新）

RAG-S1～S4 已全數實作完成並驗證（詳見 addendum #17——原 #15 因與並行線撞號重編）：`platform/src/bm25.rs`（5 tests）、`search.rs` 混合引擎（6 tests，含拒絕地板/stale 向量/降級）、`/api/search` 真分數＋via、跨線 stale fixture 修復（bankpay 鍵 5→10）。

---

# PHASE-K — 產品開發專案知識管理系統架構藍圖（D-OPS＋RAG-Transcendence）【移交件】

> 擁有者指示（2026-11-10）：先產出最佳架構計畫，交由「**專案品質檢查與文件管理優化專案**」承接執行，避免本計畫失焦。本節＝移交件全文；**本計畫不執行其建置**。

## K.0 定義界線

- **D-OPS**（動態文檔查詢）：元資料驅動的文檔檢索/導航/雙向鏈結——**給人用**。
- **RAG-Transcendence**：向量語義搜索（替代關鍵詞）＋AI 自動閱讀綜答＋自動嵌入聚類——**給 AI 用**。
- 兩者合流＝「文檔＋知識＋產品生命週期管理（PLM）」一體化系統；本藍圖定義其架構與上線管線，執行歸承接專案。

## K.1 工作室定位裁決（回應擁有者問題：稽核管理？技術？跨兩者？）

**裁決：跨工作室，主導權單一——S3 治理稽核平台為系統擁有者（治理主導），S2 API 中樞為技術底座，S1 為使用者端。**

| 候選方案 | 優點 | 缺點 | 裁決 |
|---|---|---|---|
| 僅 S3（稽核管理） | 審批流/版控政策/稽核鏈天然內建（kem gov.rs＋capture/vet/verdict 全在此語義域） | 缺向量庫/嵌入/CI-CD 技術棧 | ❌ 單獨不足 |
| 僅技術工作室 | 向量/檢索/管線技術成熟 | 無治理→無審批、無稽核、無版控政策＝亂文件倉庫 | ❌ |
| **跨 S3 主導＋S2 底座＋S1 端** | 治理與技術各司其職；**資產重用實證**：embedder 池/BM25（PHASE-R 已證 platform→kem 零成本）、kem 哈希鏈、gov.rs Finding/lint 模式、PLAN_REGISTER 雙登記慣例 | 跨界契約需先定義（K.5） | ✅ **採用** |

## K.2 分層架構（RAG-Transcendence and Integration）

| 層 | 職責 | 所屬工作室 | 關鍵組件（重用優先） |
|---|---|---|---|
| L1 使用者層 | 自然語言查詢、AI 答案閱讀、文檔瀏覽、回饋標記 | S1 | 問答介面＋引用溯源卡（文檔ID＋版本＋區段） |
| L2 檢索層 | **向量語義搜索替代關鍵詞**＋BM25 混合＋rerank | S2 | 重用 `platform/bm25.rs`＋kem embedder 池（bge-m3→Ollama）＋向量庫（pgvector/Tantivy/Qdrant 由承接專案依 K.5 評估集裁決） |
| L3 生成層 | AI 自動閱讀多篇文檔並綜合回答；**無引用不輸出**（每答案必附文檔ID＋版本＋行區段） | S2 | LLM 閘門（沿用 FREE_AI_ROUTING 池；本地優先 C-08） |
| L4 治理層 | 自動嵌入/聚類（僅導航輔助，不替代治理分類）、審批工作流、沙盒、灰度、監控、版控、稽核鏈 | S3 | gov.rs `Finding`/`lint()` 模式、kem SHA-256 鏈、PLAN_REGISTER 雙登記 |

## K.3 五閘門上線管線（沙盒→驗證→試點→灰度→監控）

| Gate | 名稱 | 內容 | 退出條件 |
|---|---|---|---|
| G0 | 沙盒模擬 | 隔離環境重放真實查詢集；不觸生產索引/文檔 | 沙盒報告產出；零生產寫入 |
| G1 | 精密實驗 | eval 金測：檢索命中率、回答正確性、**引用可解析**（每答案→文檔+版本+區段）、幻覺審計 | 指標 ≥ 預先釘選門檻（未達=拒絕，如拒絕地板概念） |
| G2 | 真實情境影子試點 | AI 答案與人工查閱**並行**（shadow mode）；差異逐條登記 | 影子對比報告 ≥ n 天零重大分歧 |
| G3 | 灰度替換 | 10%→50%→100% 查詢流；**回滾=feature flag 切回人工/舊流程** | 每階段指標不退化 |
| G4 | 上線監控 | 稽核鏈連續性、漂移偵測（embedder 模型版本變更→全量重嵌評估）、品質回歸警報 | 常態營運＋季度重審 |

## K.4 版控與稽核（重用既有資產，不重造）

| 機制 | 重用來源 | 說明 |
|---|---|---|
| 文檔版本化 | kem 哈希鏈＋audit_log 模式 | 文檔=版本化物件（content+sha256+version+prev_id）；變更=diff＋審批；回滾=revert |
| 自動嵌入 | kem::embed::embed_batch（PHASE-R 已證零成本重用） | add 時嵌入；模型版本變更→stale（sha256 失配語義沿用） |
| 聚類導航 | gov.rs `Finding`/`lint()` 模式 | 聚類僅輔助檢索；治理分類仍以 CONTROL_LIBRARY/框架映射為準 |
| 登記整合 | PLAN_REGISTER 雙登記慣例 | 文檔登記＝計畫登記子集；同一提交雙登記 |

## K.5 移交清單（交「專案品質檢查與文件管理優化專案」）

1. 本節 K.0–K.4 全文（架構藍圖＋五閘門＋版控稽核）。
2. 元資料規範：front-matter（tags/status/owner/last-updated/related-docs），全文檔強制。
3. 評估集：golden queries＋引用金測（G1 門檻釘選由承接專案執行，本藍圖僅定方法）。
4. SSOT 紀律：`docs/` 權威文件**不搬移**，系統僅索引＋嵌入＋治理。
5. 承接方驗收：G0–G4 各自 DoD；完成後回寫 PLAN_REGISTER addendum。
6. **`DOCUMENTATION_MASTER_SPEC.md`**（Dev 根目錄，205 行，v2.0 Apex）：文檔治理憲政規格——SSOT/絕對反模擬/漸進式揭露/剛性邊界/八角色驗收邊界——與本藍圖 L4 治理層直接互補，承接專案必讀並併入其治理章程（RP-6④ 一併移交）。

## K.6 釐清：1793788660000 HTML 檔案之謎（擁有者先前查詢）

擁有者提及的 `~/.local/share/kilo/plans/1793788660000-flaps-erp-performance-progress-report-v2.0.html` **確實存在**——位於 Kilo 自身計畫快照區（使用者 profile，工作區外），故工作區掃描找不到。該檔為 2026-11-05 產品線進度報告之 Kilo 快照（.md＋.html 渲染版）。**工作區 SSOT** 仍為 `docs/research/` 下文件；Kilo 快照區由 Kilo 管理，不入工作區版控（已入 PLAN_REGISTER 追溯，見 RP-1）。

---

# PHASE-RP — 輔翼ERP效能評估進度報告 整合與演進（本計畫聚焦重定向）

> 擁有者指示（2026-11-10）：本計畫聚焦 輔翼ERP效能評估進度報告。

## RP.0 重大發現：同名報告 ×2（重複風險實錘，語義漂移定理命中）

| 報告 | 位置 | 日期 | 覆蓋線 | 內容 |
|---|---|---|---|---|
| ①**產品線**進度報告 | `~/.local/share/kilo/plans/1793788660000-*`（.md 552 行＋.html；Kilo 快照區，**工作區外**） | 2026-11-05 | **Electron 產品線**（FlapsErp v0.0.4 桌面） | FLAPS-PERF-PRG-2026-1105-003「效能問題評估與解決方案—專案進度暨改善追蹤報告」：config.ini 調優核對表（附錄C）、A/B 雙軌、P1-P4 藍圖、RSK-E1~E9、五閘門前身（G0-G4 雛形） |
| ②**平台線**進度報告 | `docs/research/FlapsErp效能評估進度報告_v2.0.md`（**工作區內 SSOT**） | 2026-11-09 | **Rust 平台線** | 132 行：246 tests、422ms SLA、MT 100%、RAG、十二切片 |
| ③**RPA-ECOM 專案主文件 v2.1** | `FlapsErp_RPA_電商系統對接_專案文件與開發前準備報告_v2.1.html`（**Dev 根目錄**，863 行；未登錄 docs 索引） | 2026-10-27 撰寫／11-03 Gate1 複審修訂 | **RPA-ECOM 對接線**（Electron→電商） | 專案代碼 FLAPS-RPA-ECOM-2026：G1–G6 KPI（G4＝斷路器：連續失敗 5 次觸發、重試 ≤3、恢復 ≤5min——即 G2.2 之驗收準則源）、OQ-01~08、交付物規格 DEL-M/D/C/T、WBS 101 人日、甘特（至 2027-02-08 結案）、RACI、風險登記冊、§10 部署維運、§11 最佳化（「文件即程式碼」原則自述）、ADR-001/002 草案清單 |
| ④**Dev 根目錄未登錄文件群**（8+ 份 .md，全未入 docs 索引） | 根目錄 glob 實測 | 2026-10-26~11-05 | 產品線＋跨線 | **除 DOC-01/02/03 由 ① 搬移處理外**：`PHASE1_IMPLEMENTATION_README.md`（311 行，Phase 1 實作記錄）、`FREE_AI_SYSTEM_UPDATE.md`（459 行，免費 AI 研究——FREE_AI_ROUTING 之前身/補充）、`DOCUMENTATION_MASTER_SPEC.md`（205 行，**文檔治理憲政規格**：SSOT/反模擬/漸進式揭露/八角色驗收）、`WORKTREE_WORKFLOW.md`（209 行，八環境協作，Active）、`GEMINI.md`（75 行，WEB3 工作區規則，跨區）、`README.md`（根目錄 README，待 audit 與 docs/README.md 區分） |
| ⑤**Kilo 快照區完整計畫庫** | `~/.local/share/kilo/plans/`（**~70 份 .md**，工作區外） | 2026-11-01~09 | 跨專案 | **效能家族演化鏈完整現蹤**：progress v1（`1793787631643-flaps-erp-performance-progress-report.md`，與本計畫同 session 前綴——Kilo 計畫 ID＝session 時間戳前綴，非唯一計畫 ID）→v2.0（1793788660000）；install guide **v1→v2.0→v3.0→v4.0→v5.0→v6.0**（1793788300000/9000000/9500000——版本鏈已核實無 v8）；deep-verification report（1793789700000）。另有其他專案計畫 ~50 份（ISMS/ekpi/MTRIX/Wellfit/ARES/KNB 系列）——歸各專案，不複製 |

**風險**：①②同名（FlapsErp效能評估進度報告_v2.0）、同 v2.0、不同線（Electron 產品 vs Rust 平台）——正是 §0.5 語義漂移定理命中案例：同概念、邊界無契約。**擁有者記憶正確**：①內容含 config.ini 調優（附錄C：備份→DISABLE_HARDWARE_ACCELERATION→IGNORE_CERT_ERR→重跑量測→回滾），即「調整哪些設定改善效能」之文件。③之發現另揭：`docs/management/FlapsErp_RPA_Ecommerce_Project_Plan_v2.1.md` 頭註宣稱「v2.1 前檔為 0 bytes 空殼，本版首次補實全文」**對家族歷史不實**——.md 檔本身 0 bytes 屬實，但家族 v2.1 內容一直存在於根目錄 HTML（見 RP-5 修正）。

**RP-1 執行狀態更新（2026-11-11 18:42 實證）**：兩份報告之 `.md` **均已在工作區**——`docs/research/FlapsErp平台線效能評估進度報告_v2.0.md`（10,950 B，平台線改名完成）＋`docs/research/FlapsErp產品線效能進度報告_v2.0.md`（35,577 B，產品線入冊完成，內容自 Kilo 快照 552 行複製）。**惟 HTML 渲染狀態不對稱**：產品線 `.html`（66,298 B）僅存在於 Kilo 快照區；平台線 `.html` **不存在**（僅 .md）——待 RP-7 以 pandoc 渲染補齊（置於各報告同目錄；產出後兩線對稱：.md 權威＋.html 渲染成對）。

## RP.1 切片（消解同名衝突，可查證可追溯）

- [x] **RP-1 命名區分＋入冊（已裁決：對稱改名，雙向門）**：①產品線報告入工作區為 `docs/research/FlapsErp產品線效能進度報告_v2.0.md`（內容自 Kilo 快照 .md 複製，頭註聲明「Electron 產品線；Kilo 快照區另有 .html 渲染版」）；②平台線改名 `docs/research/FlapsErp平台線效能評估進度報告_v2.0.md`——同一提交更新活引用（README §2.7、效能優化操作手冊相依清單；歷史 addendum #14/#17 為點時記錄不回改，由 #18 記錄改名）；③兩者互鏈＋README §2.7 索引登錄＋PLAN_REGISTER addendum #18。**裁決依據**：擁有者核心顧慮＝同名混淆已實際發生三次（Kilo HTML 誤認、config.ini 文件遍尋不著、本輪重複疑慮）；對稱命名永久消歧；改名為雙向門（revert＝改回）。**活引用實證（2026-11-10 grep）＝3 處**：效能優化操作手冊 L10、README L121、PLAN_REGISTER addendum #17（點時記錄不回改）——全部同一提交更新，估算精確。
- [x] **RP-2 四方整合指引（零內容重複）**：config.ini 調優之**操作源**＝產品線報告附錄C 核對表；**整合手冊**＝`docs/04-operations/FlapsErp效能優化操作手冊_v1.0.md` §3.2；**量測法**＝效能架構 P4；**驗收指標源**＝RPA-ECOM 主文件 §1.1 G4（斷路器：連續失敗 5 次觸發、重試 ≤3、恢復 ≤5min——即 G2.2 之驗收準則）與 §10.2（監控/備份/回滾）。四方互鏈不重寫。另：**操作手冊家族互鏈**——效能優化操作手冊 ↔ A1 檔案通道手冊（RP-6⑥ 遷移後同目錄 `docs/04-operations/`），附錄交叉引用。
- [x] **RP-3 演進程序**：平台線報告——每次四關全綠後 7 日內刷新（測試數/SLA/新切片）；產品線報告——隨 P1-P4 藍圖階段更新（歸並行線/承接專案）。
- [x] **RP-4 誠實標註維持**：RAM 未量測、M2 登入稽核、fuzz run 等維持 UNKNOWN；拒絕地板/事實等級（✔△⚠）標記沿用。
- [x] **RP-5 RPA-ECOM v2.1 家族調和（新發現，撤回錯誤宣稱）**：
  - 事實：Dev 根目錄 `FlapsErp_RPA_電商系統對接_專案文件與開發前準備報告_v2.1.html`（863 行）為 RPA-ECOM 線**原始專案主文件**（2026-10-27 撰寫、11-03 Gate1 複審修訂、被逆向分析 §9 引用、有 ADR-001/002 鏈）；`docs/management/FlapsErp_RPA_Ecommerce_Project_Plan_v2.1.md` 頭註宣稱「v2.1 前檔為 0 bytes 空殼，本版首次補實全文」**對家族歷史不實**（.md 檔本身 0 bytes 屬實，但家族 v2.1 內容一直存在於根目錄 HTML）——依誠實紀律（GOVERNANCE P7）必須修正。
  - 作法：①將 HTML 內容轉錄為 `docs/management/FlapsErp_RPA_電商系統對接_專案文件與開發前準備報告_v2.1.md`（與 .html 成對——沿用 Kilo 快照 .md/.html 成對慣例），並併入摘要版之獨有「現況對齊」段（四階段藍圖對齊＋2026-11-09 門檻 246/422ms）為 §0 附錄；②摘要版 `FlapsErp_RPA_Ecommerce_Project_Plan_v2.1.md` 頭註改標「SUPERSEDED by 主文件（執行狀態已併入其 §0 附錄）」；③根目錄 .html 與 ADR 四檔（ADR-001/002/003 草案＋Gate1 複審報告 v1）搬入 `docs/management/`（**不搬 docs/adr/**——避免與 ADR-0001..0013 撞號；RPA-ECOM 家族檔名自帶前綴消歧）；④README §2.6 索引登錄＋addendum #18。
  - 裁決依據：原始主文件在先（10-27）、內容最富（WBS/甘特/RACI/風險/OQ/交付物規格）、被上游引用 → 為家族 SSOT；後建摘要版（11-10，不知情重複）＝降級併入。「文件即程式碼」原則（主文件 §11 自述）支持 .md 權威＋.html 渲染成對。
  - DoD：主文件 .md/.html 成對入冊；錯誤宣稱修正有記錄；ADR 四檔入冊；摘要版 superseded 標記；docs 索引零死鏈。
- [ ] **RP-6 全域盤點登錄（Dev 根目錄＋Kilo 快照區——消除盤點方法缺口）**：
  - [x] ①**DOC-01/02/03 搬入 `docs/research/`**（產品線前綴命名：`FlapsErp產品線效能深度分析_v1.0.md`／`FlapsErp產品線專案章程_v1.0.md`／`FlapsErp產品線架構優化計畫_v2.0.md`），同一提交更新產品線報告之來源清單路徑（DOC-01~03 原引無 docs/ 前綴）。**（2026-11-11 完成：git mv 三檔；產品線報告來源清單（2 處）＋專案章程 §11.1 引用更新；README §2.7 索引新增 3 列。）**
  - ②**Kilo 快照區追溯表**落盤於 README 索引：效能家族演化鏈（progress v1→v2.0、install guide v1→v6、deep-verification，含 Kilo 內路徑與版本鏈）——**不搬移**（Kilo 管理），僅登錄位置與版本鏈供追溯。
  - ③其他專案計畫 ~50 份（ISMS/ekpi/MTRIX/Wellfit/ARES/KNB）＝README 索引單一指引行（「其他專案計畫見 Kilo 快照區」），**不複製不搬移**（歸各專案 SSOT）。
  - ④四份待分類裁決：`PHASE1_IMPLEMENTATION_README.md`（audit 後 register 或 superseded）、`FREE_AI_SYSTEM_UPDATE.md`（與 `docs/engineering/FREE_AI_ROUTING.md` 關係裁定：合併或標 superseded——同一主題兩份，語義漂移風險）、`DOCUMENTATION_MASTER_SPEC.md`（→ **併入 PHASE-K.5 移交件**：文檔治理憲政規格與 L4 治理層直接互補）、`WORKTREE_WORKFLOW.md`（Active → 入 `docs/standards/` 登錄）；`GEMINI.md`（WEB3 工作區規則，非本工作區管轄）→ 標記待擁有者裁決去留，本計畫不動。
  - ⑤`web/legacy-1420.html`＋`web/assets/legacy-1420.css`（並行線新增，1420 原版 UI 重現）：納入零 JS 適應度驗證（script=0 斷言）＋登錄 web/ 說明。另：並行線已產出 `docs/research/legacy-1420_vs_platform_深度比較分析.md`（13.8KB，2026-11-10）——已存在於 research/（自動納入索引，無需搬移）。
  - ⑥`docs/research/A1檔案通道端到端操作手冊_v1.0.md`（236 行 Runbook，RUN-2026-1103-001）→ ✅ **已完成搬移**（2026-11-10 實證：glob 顯示僅存 `docs/04-operations/A1檔案通道端到端操作手冊_v1.0.md`，research/ 無殘留；README §2.7→§2.6 索引同步已由先前回合完成）。
  - DoD：Dev 根目錄 .md 零未登錄（README 索引全覆蓋）；Kilo 追溯表落盤；每檔有去向（register/supersede/移交/待裁決）；操作手冊家族三冊同目錄互鏈；addendum #18 記錄。
- DoD：兩報告可區分（檔名＋頭註）、零內容重複、README＋PLAN_REGISTER 已登錄、addendum #18 落盤。

## RP.2 裁決依據

- 語義漂移定理（§0.5）：同名異線＝邊界無單位/線別契約 → 命名區分＋互鏈，不合併為單檔（單向門：合併需擁有者 ADR，且兩線讀者/節奏不同）。
- SSOT 紀律：Kilo 快照區（工作區外）非 SSOT；`docs/research/` 為 SSOT；快照檔由 Kilo 管理**不搬移不刪除**，僅入冊索引（RP-1）。
- RPA-ECOM 家族（RP-5）：原始主文件在先（10-27）、內容最富、被上游引用（逆向分析 §9）→ 為家族 SSOT；後建摘要版（11-10，不知情重複）＝降級併入。「文件即程式碼」原則（RPA-ECOM 主文件 §11 自述：架構圖/介面矩陣/工作流/腳本納入版控）支持 .md 權威＋.html 渲染成對。
- 命名裁決（RP-1）：對稱改名為雙向門（revert＝改回原名），引用更新 ≤3 處同一提交、addendum #18 留痕——符合「該刪就刪該保留就保留該發展就發展」與零静默變更紀律。**活引用實證（2026-11-10 grep）＝3 處**：效能優化操作手冊 L10、README L121、PLAN_REGISTER addendum #17（點時記錄不回改）——全部同一提交更新。

## RP.3 文檔一致性核查（2026-11-09 執行，本輪新增）

對 2026-11-09 新增的設定變更文件進行引用完整性（C-06）與事實一致性核查：

| 核查項 | 結果 | 證據 |
|---|---|---|
| `docs/04-operations/FlapsErp設定AB實測報告_v1.0.md` | ✅ 存在；NOT VERIFIED 標記正確（唯讀證據；IT 變更/A/B/soak 未執行）；引用的 `flaps-perf.ps1`、Wellfit artifacts、`FlapsErp效能基線報告_v1.0.md` §7.3、操作手冊 §3.2/§3.4 全部存在且數據一致 | 本回合 read 實測 |
| `docs/04-operations/FlapsErp設定變更runbook_v1.0.md` | ✅ 存在；SOP 結構完整（角色/變更前/變更 A-C/驗證/回滾/簽核）；紅線明確（代理唯讀、IT 變更管制） | 本回合 read 實測 |
| README 索引 | ✅ 新文件已登錄（§2.8 L137-140：最佳設定基線、設定變更 runbook、效能優化操作手冊、設定 A/B 實測報告） | README L137-140 |
| 鍵名拼字一致性 | ✅ **無需修正**：原廠鍵名確為 `FORCE_DEVICE_SACLE_FACTOR`（廠商拼字錯誤），main.js:186 同時檢查正確鍵 `FORCE_DEVICE_SCALE_FACTOR`；逆向分析方案 L89 已記錄此雙寫事實、操作手冊 L261 採原廠鍵名——三處一致 | 逆向分析 L89/L122/L148；操作手冊 L261 |
| ❌ **懸空引用（C-06 違例）** | `1794395575776-flaps-erp-product-system-settings.md` 被設定 A/B 實測報告（J 節）與設定變更 runbook 引用為來源計畫，但**工作區 `.kilo/plans/` 與 Kilo 快照區皆不存在** | 本回合 glob 雙區實測 |
| RP-1 執行狀態 | ✅ 兩份報告 .md 均已在工作區（平台線 10,950 B／產品線 35,577 B，2026-11-11 18:42 寫入）；**惟 HTML 渲染不對稱**（產品線 .html 僅 Kilo 快照區；平台線無 HTML）→ RP-7 待執行 | 本回合 Get-ChildItem 實測 |

## RP.4 待辦登記（2026-11-09 更新）

| 切片 | 狀態 | 說明 |
|---|---|---|
| RP-1 命名區分＋入冊 | ✅ 已執行（並行線 2026-11-11 18:42） | 平台線改名＋產品線入冊完成；惟 HTML 渲染待 RP-7 |
| RP-2 四方整合指引 | 🟡 部分 | 操作源（產品線報告附錄C）與驗收指標源（RPA-ECOM G4）已確認存在；四方互鏈待落實 |
| RP-3 演進程序 | ⬜ | 平台線 7 日刷新／產品線隨 P1-P4 更新 |
| RP-4 誠實標註維持 | ✅ 持續中 | A/B 報告 NOT VERIFIED 標記正確；CCM M2 誠實 null 沿用 |
| RP-5 RPA-ECOM 家族調和 | 🟡 部分 | HTML→md 成對待執行；摘要版 SUPERSEDED 待標記 |
| RP-6 全域盤點登錄 | ✅ 大半完成 | DOC-01/02/03、A1 手冊搬移、四份待分類已處理或裁定；根目錄 README.md 待 audit |
| RP-7 雙報告 HTML 渲染對稱化 | ⬜ 新增 | pandoc 渲染兩份 .md → 同目錄 .html（寫入操作，交 code agent） |
| RP-8 懸空引用修復 | ⬜ 新增 | `1794395575776` 計畫不存在——擁有者裁決：重建該計畫文件或修正兩處引用（建議：於 .kilo/plans/ 補建該計畫 stub 並標「設定變更計畫（已併入 1793787631643）」或修正 A/B 報告 J 節引用） |

---

# PHASE-RP.10 明示不在範圍（PHASE-RP）

- KafkaEventStore 實作、SSE（OQ-GOV-02）、PostgreSQL 遷移（OQ-GOV-01）、HA（OQ-GOV-03）、api.rs 結構重構（僅 PRE-R1 測試）、輔翼 ERP 本體修改、授權/合約行為、MoneyCents↔minor 型別合併（僅契約）、外部門戶實作（G3.4 僅規格）。
- **知識管理系統（D-OPS＋RAG-Transcendence）建置**：PHASE-K 為移交件，由「專案品質檢查與文件管理優化專案」承接執行，本計畫不負責建置。

---

**本計畫結束。** 後續執行由 code agent 依切片順序進行；每切片完成後更新 PLAN_REGISTER addendum。所有「完成」聲明以證據為準；本檔不構成部署、認證或合約授權。
