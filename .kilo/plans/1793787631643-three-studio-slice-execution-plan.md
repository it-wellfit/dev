# 三大工作室 SLICE 執行計畫（PHASE-G1..G4）— v8.2

> 計畫編號：1793787631643｜日期：2026-11-10（v8 初版）；2026-11-12（v8.1 SPEC-GAP 缺口補強；v8.2 深度稽核＋SOTA 採用雷達＋LEAN＋跨裝置＋營運級別＋API 契約＋資料治理＋測試策略）｜狀態：implementation-ready
> v8 變更：RP-6 全域盤點（Dev 根目錄未登錄文件群＋Kilo 快照區 ~70 份計畫＋web/legacy-1420 並存事實與零 JS 適應度驗證）；RP-7 雙報告 HTML 渲染對稱化；RP-8 懸空引用修復。
> v8.2 變更：SPEC-GAP-2 深度稽核（RRF 融合／嵌入斷路器／批次寫入／角色 seed 事實校正／api.rs 5,950 行實測）；**PHASE-G4 執行切片 G4.1–G4.8**；TECH-SOTA 免費 SOTA 技術雷達；LEAN 精簡預算；UX-XDEVICE 跨平台跨裝置；OPS-SLA 營運級別（含 STRIDE）；API-CONTRACT 契約；DATA-GOV 資料治理；PERF-BUDGET 效能預算；TEST-STRATEGY 測試策略；FLAG 旗標；A11Y 檢查表。
> 上位架構 SSOT：`.kilo/plans/1792865422970-three-studio-governance-platform.md`（v3.1 §7/§17/§20/§22）
> 執行者注意：衝突以上位為準；每切片先做 §0 樹態核查再立 RED。

---

## 0.1 計畫目的與範圍

本計畫為「三大工作室（部門業務工作台 S1、API 中樞與資料管理 S2、治理稽核平台 S3）」的執行計畫，並包含下列軌道：

- **PHASE-G1～G3（三大工作室執行）**：本計畫**直接執行**，包含 12 個切片。
- **PHASE-G4（深度稽核修復，D2）**：本計畫**直接執行**，處理 SPEC-GAP-2 之程式級缺口（測試/角色/單位/文件）。
- **PHASE-K（知識管理系統架構藍圖）**：本計畫**僅產出架構藍圖**，執行由「專案品質檢查與文件管理優化專案」負責。
- **PHASE-RP（輔翼ERP效能評估進度報告整合與演進）**：本計畫**僅產出架構藍圖與執行順序**，由「專案品質檢查與文件管理優化專案」承接實作。
- **跨切面規格（TECH-SOTA／LEAN／UX-XDEVICE／OPS-SLA／API-CONTRACT／DATA-GOV／PERF-BUDGET／TEST-STRATEGY／FLAG／A11Y）**：本計畫**定義並適用**於 G1～G4；PHASE-K 承接時一併沿用。

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
| F11 | 檢索融合已升級為 **RRF（k=60）**：`RRF_K=60.0`（與 `kem::search::RRF_K` 共用）、`REFUSAL_FLOOR=0.55`；加權 `fuse` 僅存 `#[cfg(test)]` 作對照 | `platform/src/search.rs:36-40,88-95,176-178,425-432` | RAG-S2 規格已改寫；`fuse` 保留為測試 oracle（LEAN：非死碼） |
| F12 | 嵌入呼叫具 **2s 上限＋60s 斷路器**（`embed_with_breaker`）；寫入後 `invalidate()` 立即失效；`add_documents_bulk` 批次重建（消除 O(n²) 與逐檔 60s） | `platform/src/search.rs:259-275,292-301` | RAG-S2 精實化三件套納入；效能預算見 PERF-BUDGET |
| F13 | UDM 除 `MoneyCents` 外另有 `Decimal4(i64)`（10^4 固定小數） | `platform-hub/crates/udm/src/lib.rs:26-28` | 單位契約須同時覆蓋 Decimal4（D2-04） |
| F14 | 角色 seed 在 `core/src/permission.rs::initialize_default()`（ADMIN 等）；示範帳號由 `scripts/seed-demo.ps1` 建（demo-finance→FINANCE、demo-viewer→STORE）；`VIEWER` 僅見於模板 | `core/src/permission.rs:224-266`；`scripts/seed-demo.ps1:113-114`；`platform/templates/index.html:112` | VIEWER 無 seed/測試 → 缺口 D2-05 |
| F15 | API 目錄仍寫「全文檢索（Tantivy…）」——與現行 BM25+RRF 實作不符 | `platform/src/apihub.rs:212-213` | 文件/程式降級修正（D2-06） |

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
| `api.rs` 模組化（R1） | **5,950 行（non-test）／9,059 行（含 tests；2026-11-12 實測，`#[cfg(test)]` 自 L5951）**；**78 路由**（production 段）；14 個 POST 表單結構各自內聯 CSRF 檢查（`csrf_token != ` ×14） | **Connascence-of-algorithm**：同演算法複製 14 處、局部性極低（跨 6000 行）→ 高強度低局部性=拆分候選；但**扇入=78 路由共用 AppState**，屬單向門 | **最後排序**：先 PRE-R1 PIN 測試；GAP-08 分歧解鎖後以「extractor/middleware 化」食譜執行（見 PRE-R1b） |
| `web/index.html` 靜態基線 | Superseded 頭註（2026-11-04；**306 行**，2026-11-12 實測）；並行線另增 `web/legacy-1420.html`（**179 行**，2026-11-12 實測，1420 原版 UI 重現）＋`legacy-1420.css`——兩檔並存，零 JS 適應度待 RP-6⑤ 驗證 | 資訊價值>維護成本 | 兩檔均保留不刪；legacy-1420 納入零 JS 適應度檢查 |
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

> **完成狀態總覽（2026-11-12 更新）**：PHASE-G1/G2/G3 共 12 切片＋PRE-R1a 全數實作完成並驗證（G1.1＋G2.x 見 addendum #12；G1.2–G1.4＋G3.x 見 #13；全工作區 246 passed / 0 failed，含並行線新增）。未執行：G3.4（僅規格，OQ-GOV-04 阻塞）、PRE-R1b（可選）、**PHASE-G4 G4.1–G4.8（D2 深度稽核修復，本輪新增）**。

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

### PHASE-G4 — 深度稽核修復（D2，2026-11-12 新增；單切片單提交）
- [ ] **G4.1 RAG-S3 整合測試**（D2-02）：新增 `search_api_*` 4 例（fixture 語料排序、CJK 查詢、未知詞回 `[]`、無 embedder 時 `via="bm25"`）；DoD：4 測試綠＋四關。
- [ ] **G4.2 VIEWER 角色 seed/測試**（D2-05）：`seed-demo.ps1` 或 migration 增 VIEWER；landing 測試覆蓋 `VIEWER→報表入口`；DoD：測試綠＋DEMO 文件更新。
- [ ] **G4.3 Decimal4 單位契約**（D2-04）：定義 `Decimal4`（10^4）與 `MoneyCents`（10^2）之關係——禁止直接互轉；如需換算走明確 `TryFrom`＋scale 標註；DoD：round-trip＋非法轉換測試。
- [ ] **G4.4 API 目錄 stale 修正**（D2-06）：`apihub.rs:212-213`「Tantivy」改為「BM25＋RRF」；加文件↔路由一致性測試；DoD：測試綠。
- [ ] **G4.5 G1.4 測試逐名核**（D2-02b）：列出批量部分失敗/自我裁決 403 之實際測試名並回填 S.2.1；DoD：表更新＋無懸空名。
- [ ] **G4.6 A11Y 檢查表落地**（D2-16）：A11Y 10 項轉為 `docs/standards/A11Y_CHECKLIST.md`；可機械檢查項（label/landmark/標題層級）納入測試；DoD：文件＋檢查綠。
- [ ] **G4.7 PERF-BUDGET 基線量測**（D2-11）：為 `/api/search`（向量/ BM25-only）與 TTFB 建 benchmark；記錄基線與方法；DoD：基線入冊（未量測標 UNKNOWN）。
- [ ] **G4.8 OPS-SLA runbook**（D2-08）：SLO/SLI、備份還原、事件管理（P1/P2）runbook；DoD：文件入 `docs/04-operations/` 且 README 登錄。

### 平行安排（非切片，只加測試不改結構）
- [x] **PRE-R1a**（✅ 並行線交付 2026-11-09：`platform/tests/characterization.rs`，6 tests PASS）：api.rs 路由特徵化測試集（請求→狀態碼+關鍵 DOM 斷言），作未來模組化安全網。
- [ ] **PRE-R1b（可選，框架首次應用示範）**：CSRF 統一 extractor——量化證據：同一檢查複製 **14 處**（2026-11-12 實測；connascence-of-algorithm）。食譜：以 axum extractor/middleware 收斂為單一實作（行為不變）；前置=PRE-R1a PIN 全綠；驗證=PIN 不變＋四關＋`post_rejects_wrong_csrf_token` 等既有測試不改一字通過。雙向門（可回滾），architect 核可即做；不做亦不阻塞任何切片。

## 3. 執行順序

G1.1 → G1.2 → G1.3 → G1.4 → G2.1 → G2.2 → G2.3 → G3.1 → G2.4 → G2.5 → G3.2 → G3.3 → **G4.1 → G4.2 → G4.3 → G4.4 → G4.5 → G4.6 → G4.7 → G4.8**（D2 修復）。單切片單提交；每切片完成即更新上位 §22 追溯矩陣（零孤兒）＋PLAN_REGISTER addendum，不批量補帳。

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
| D2-02 測試缺口（RAG-S3/G1.4 無逐名證據） | G4.1/G4.5 修復；修復前追溯矩陣標「未驗證」 |
| D2-05 VIEWER 無 seed | G4.2；未修前 landing 表標「未啟用」 |
| D2-11 效能預算未量測 | G4.7 建基線；未量測標 UNKNOWN（C-10） |
| SOTA 新依賴（proptest/cargo-mutants/tracing 等） | TECH-SOTA 五閘門＋POC＋ADR；未過不進 Cargo.toml |

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
- [x] **RAG-S2 混合引擎**（✅ 2026-11-09；2026-11-12 SOTA 精實化）：`platform/Cargo.toml` 增 `kem = { path = "../crates/kem" }`；`SearchEngine` 增 `bm25: RwLock<BM25Index>`＋`vectors: RwLock<HashMap<doc_id, (sha256, Vec<f32>)>>`；`add_document` 內嵌 `kem::embed::embed_batch(&kem::embed::default_pool(), ...)`（Err→無向量，照常索引——誠實降級）；新增 `search_ranked(query) -> Vec<RankedHit{document, score, via}>`：**融合預設為 RRF（Reciprocal Rank Fusion，k=60，Cormack et al. 2009；`RRF_K` 與 `kem::search::RRF_K` 共用單一標準）**——融合**排名**而非正規化分數，`score(d)=Σ_path 1/(k+rank_path(d))`，尺度不變（不需分數校準、單一路徑分數範圍無法壓制另一路）；加權 `fuse`（α=0.5）保留為**測試對照**（`#[cfg(test)]`，證明 RRF 尺度不變而加權不是）；語義拒絕地板 `REFUSAL_FLOOR=0.55`（對齊 kem 校準 2026-11-09）逐字保留，零詞法支持且 cosine<地板→丟棄；舊 `search()` 委託 ranked 丟分數（相容不變）；`delete_document` 重建 BM25＋刪向量；向量帶內容 SHA-256（kem §8 新鮮度規則），失配視同缺失。
  - **2026-11-12 精實化三件套**：①`embed_with_breaker`——嵌入呼叫 2 秒上限＋斷路器（不可達即停用 60 秒；把離線情境每次搜尋 180s 攤成 ≤2s）；②`invalidate()`——寫入後主動失效，下次搜尋立刻重建（關閉「新交易最多 60 秒搜不到」一致性窗口）；③`add_documents_bulk`——一次 BM25 重建＋一次批次嵌入（消除逐檔 rebuild 的 O(n²) 與逐檔 60s 網路等待）。
  - 測試（`platform/src/search.rs`，9 項）：`bm25_only_degradation_ranks_by_relevance_and_marks_via`、`unknown_term_returns_empty_and_query_ordering_holds`、`fusion_blends_lexical_and_vector_with_via_provenance`、`refusal_floor_drops_weak_vector_only_hits`、`stale_vector_is_treated_as_missing`、`delete_rebuilds_index_and_document_count`、`rrf_fuse_preserves_refusal_floor`、`rrf_fuse_is_scale_invariant_where_weighted_is_not`、`search_ranked_uses_rrf_by_default`。DoD：全綠（2026-11-12 實證）。
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

RAG-S1～S4 已全數實作完成並驗證（詳見 addendum #17——原 #15 因與並行線撞號重編）：`platform/src/bm25.rs`（5 tests）、`search.rs` 混合引擎（9 tests，含拒絕地板/stale 向量/降級/RRF 尺度不變）、`/api/search` 真分數＋via、跨線 stale fixture 修復（bankpay 鍵 5→10）。2026-11-12 SOTA 精實化（RRF k=60 預設、嵌入斷路器 2s/60s、寫入失效、批次寫入）已落地並有測試支撐。

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
| ①**產品線**進度報告 | `docs/research/FlapsErp產品線效能進度報告_v2.0.md`（35,577 B，**工作區內**）＋`docs/research/FlapsErp產品線效能進度報告_v2.0.html` | 2026-11-05 撰寫／11-09 入冊 | **Electron 產品線**（FlapsErp v0.0.4 桌面） | FLAPS-PERF-PRG-2026-1105-003「效能問題評估與解決方案—專案進度暨改善追蹤報告」：config.ini 調優核對表（附錄C）、A/B 雙軌、P1-P4 藍圖、RSK-E1~E9、五閘門前身（G0-G4 雛形） |
| ②**平台線**進度報告 | `docs/research/FlapsErp平台線效能評估進度報告_v2.0.md`（10,950 B，**工作區內**）＋`docs/research/FlapsErp平台線效能評估進度報告_v2.0.html` | 2026-11-09 | **Rust 平台線** | 132 行：246 tests、422ms SLA、MT 100%、RAG、十二切片 |
| ③**RPA-ECOM 專案主文件 v2.1** | `FlapsErp_RPA_電商系統對接_專案文件與開發前準備報告_v2.1.html`（**Dev 根目錄**，863 行；未登錄 docs 索引） | 2026-10-27 撰寫／11-03 Gate1 複審修訂 | **RPA-ECOM 對接線**（Electron→電商） | 專案代碼 FLAPS-RPA-ECOM-2026：G1–G6 KPI（G4＝斷路器：連續失敗 5 次觸發、重試 ≤3、恢復 ≤5min——即 G2.2 之驗收準則源）、OQ-01~08、交付物規格 DEL-M/D/C/T、WBS 101 人日、甘特（至 2027-02-08 結案）、RACI、風險登記冊、§10 部署維運、§11 最佳化（「文件即程式碼」原則自述）、ADR-001/002 草案清單 |
| ④**Dev 根目錄未登錄文件群**（8+ 份 .md，全未入 docs 索引） | 根目錄 glob 實測 | 2026-10-26~11-05 | 產品線＋跨線 | **除 DOC-01/02/03 由 ① 搬移處理外**：`PHASE1_IMPLEMENTATION_README.md`（311 行，Phase 1 實作記錄）、`FREE_AI_SYSTEM_UPDATE.md`（459 行，免費 AI 研究——FREE_AI_ROUTING 之前身/補充）、`DOCUMENTATION_MASTER_SPEC.md`（205 行，**文檔治理憲政規格**：SSOT/反模擬/漸進式揭露/八角色驗收）、`WORKTREE_WORKFLOW.md`（209 行，八環境協作，Active）、`GEMINI.md`（75 行，WEB3 工作區規則，跨區）、`README.md`（根目錄 README，待 audit 與 docs/README.md 區分） |
| ⑤**Kilo 快照區完整計畫庫** | `~/.local/share/kilo/plans/`（**~70 份 .md**，工作區外） | 2026-11-01~09 | 跨專案 | **效能家族演化鏈完整現蹤**：progress v1（`1793787631643-flaps-erp-performance-progress-report.md`，與本計畫同 session 前綴——Kilo 計畫 ID＝session 時間戳前綴，非唯一計畫 ID）→v2.0（1793788660000）；install guide **v1→v2.0→v3.0→v4.0→v5.0→v6.0**（1793788300000/9000000/9500000——版本鏈已核實無 v8）；deep-verification report（1793789700000）。另有其他專案計畫 ~50 份（ISMS/ekpi/MTRIX/Wellfit/ARES/KNB 系列）——歸各專案，不複製 |
| ⑥**knowledge/ 目錄（KEM 治理實體）** | `knowledge/`（工作區內） | 2026-11-07~11-09 | S3 治理稽核 | **KEM 知識演進模組**：`dashboard.html`（儀表板）、`index.md`（索引）、`log.md`（操作日誌）、`claims/*.json`（14 則知識主張）、`.kem/ledger.jsonl`（哈希鏈帳本）、`.kem/review-queue.jsonl`（審查佇列）、`.kem/vectors.json`（向量索引）——與 PLAN_REGISTER addendum 記錄的 evolve/gov 模組對應，未登錄 docs 索引（屬生成性/運行態產物，非手寫文檔） |

**風險**：①②同名（FlapsErp效能評估進度報告_v2.0）、同 v2.0、不同線（Electron 產品 vs Rust 平台）——正是 §0.5 語義漂移定理命中案例：同概念、邊界無契約。**擁有者記憶正確**：①內容含 config.ini 調優（附錄C：備份→DISABLE_HARDWARE_ACCELERATION→IGNORE_CERT_ERR→重跑量測→回滾），即「調整哪些設定改善效能」之文件。③之發現另揭：`docs/management/FlapsErp_RPA_Ecommerce_Project_Plan_v2.1.md` 頭註宣稱「v2.1 前檔為 0 bytes 空殼，本版首次補實全文」**對家族歷史不實**——.md 檔本身 0 bytes 屬實，但家族 v2.1 內容一直存在於根目錄 HTML（見 RP-5 修正）。

**RP-1 執行狀態更新（2026-11-11 18:42 實證）**：兩份報告之 `.md` 與 `.html` **均已入工作區**——`docs/research/FlapsErp平台線效能評估進度報告_v2.0.md`（10,950 B）＋`docs/research/FlapsErp平台線效能評估進度報告_v2.0.html`＋`docs/research/FlapsErp產品線效能進度報告_v2.0.md`（35,577 B）＋`docs/research/FlapsErp產品線效能進度報告_v2.0.html`。**RP-1 完成**（對稱改名＋入冊＋HTML 渲染完成）。

## RP.1 切片（消解同名衝突，可查證可追溯）

- [x] **RP-1 命名區分＋入冊（已裁決：對稱改名，雙向門）**：①產品線報告入工作區為 `docs/research/FlapsErp產品線效能進度報告_v2.0.md`（內容自 Kilo 快照 .md 複製，頭註聲明「Electron 產品線；Kilo 快照區另有 .html 渲染版」）；②平台線改名 `docs/research/FlapsErp平台線效能評估進度報告_v2.0.md`——同一提交更新活引用（README §2.7、效能優化操作手冊相依清單；歷史 addendum #14/#17 為點時記錄不回改，由 #18 記錄改名）；③兩者互鏈＋README §2.7 索引登錄＋PLAN_REGISTER addendum #18。**裁決依據**：擁有者核心顧慮＝同名混淆已實際發生三次（Kilo HTML 誤認、config.ini 文件遍尋不著、本輪重複疑慮）；對稱命名永久消歧；改名為雙向門（revert＝改回）。**活引用實證（2026-11-10 grep）＝3 處**：效能優化操作手冊 L10、README L121、PLAN_REGISTER addendum #17（點時記錄不回改）——全部同一提交更新，估算精確。
  - DoD：兩報告可區分（檔名＋頭註）、零內容重複、README＋PLAN_REGISTER 已登錄、addendum #18 落盤。
- [ ] **RP-2 四方整合指引（零內容重複）**：config.ini 調優之**操作源**＝產品線報告附錄C 核對表；**整合手冊**＝`docs/04-operations/FlapsErp效能優化操作手冊_v1.0.md` §3.2；**量測法**＝效能架構 P4；**驗收指標源**＝RPA-ECOM 主文件 §1.1 G4（斷路器：連續失敗 5 次觸發、重試 ≤3、恢復 ≤5min——即 G2.2 之驗收準則）與 §10.2（監控/備份/回滾）。四方互鏈不重寫。另：**操作手冊家族互鏈**——效能優化操作手冊 ↔ A1 檔案通道手冊（RP-6⑥ 遷移後同目錄 `docs/04-operations/`），附錄交叉引用。**（四方來源已確認存在；實際互鏈寫入待落實——RP.4 待辦表標 🟡 部分。）**
- [ ] **RP-3 演進程序**：平台線報告——每次四關全綠後 7 日內刷新（測試數/SLA/新切片）；產品線報告——隨 P1-P4 藍圖階段更新（歸並行線/承接專案）。**（程序已定義；實際刷新週期未啟動——RP.4 待辦表標 ⬜。）**
- [x] **RP-4 誠實標註維持**：RAM 未量測、M2 登入稽核、fuzz run 等維持 UNKNOWN；拒絕地板/事實等級（✔△⚠）標記沿用。**（持續實踐中——RP.4 待辦表標 ✅ 持續中。）**
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
| RP-5 RPA-ECOM 家族調和 | ✅ 完成（2026-11-11 實證） | 主文件 .md 入冊 docs/management/；摘要版 SUPERSEDED 標記（頭註 L3）；ADR 四檔＋Gate1 報告搬入；README §2.6 索引登錄（L111-117） |
| RP-6 全域盤點登錄 | ✅ 大半完成 | DOC-01/02/03、A1 手冊搬移、四份待分類已處理或裁定；根目錄 README.md 待 audit |
| RP-7 雙報告 HTML 渲染對稱化 | ⬜ 新增 | pandoc 渲染兩份 .md → 同目錄 .html（寫入操作，交 code agent） |
| RP-8 懸空引用修復 | ⬜ 新增 | `1794395575776` 計畫不存在——擁有者裁決：重建該計畫文件或修正兩處引用（建議：於 .kilo/plans/ 補建該計畫 stub 並標「設定變更計畫（已併入 1793787631643）」或修正 A/B 報告 J 節引用） |

---

## RP.5 明示不在範圍（PHASE-RP）

- KafkaEventStore 實作、SSE（OQ-GOV-02）、PostgreSQL 遷移（OQ-GOV-01）、HA（OQ-GOV-03）、api.rs 結構重構（僅 PRE-R1 測試）、輔翼 ERP 本體修改、授權/合約行為、MoneyCents↔minor 型別合併（僅契約）、外部門戶實作（G3.4 僅規格）。
- **知識管理系統（D-OPS＋RAG-Transcendence）建置**：PHASE-K 為移交件，由「專案品質檢查與文件管理優化專案」承接執行，本計畫不負責建置。

---

# SPEC-GAP — 規格缺口審查與補強（2026-11-12）

> 觸發：擁有者指示「必須深度挖掘與拓展規格，規格一定充滿漏洞與不足需要你最佳化解決所有相關問題完善所有細節」。本節為對本計畫（v8）的規格級審查，逐項列出缺口並補強。

## S.0 審查方法論

1. **對照法**：與上位架構 SSOT（`1792865422970` v3.1 §7/§17/§20/§22）逐條對照，找出未覆蓋的承諾。
2. **證據法**：檢查每個 `[x]` 切片的 DoD 是否附可驗證證據（測試名/addendum），未附者列為追溯缺口。
3. **定理法**：對每個設計決策套用 §0.5 的定理（Connascence/Little's Law/冪等代數/CAP/STPA），檢查是否有未量化即宣稱完成的項目。
4. **邊界法**：對每個「明示不在範圍」項目，檢查是否真的不影響本計畫的驗收（而非規避）。

## S.1 已識別缺口清單（含補強方案）

| 缺口 ID | 缺口描述 | 影響 | 補強方案 | 嚴重度 |
|---|---|---|---|---|
| GAP-RAG-01 | RAG-S1~S4 標記完成但缺測試名證據 | 追溯矩陣（§22）無法機械驗證 | 補上測試名：`bm25::tests::rarer_terms_score_higher_than_common_terms` 等 5 項 + `search::tests::bm25_only_degradation_*` 等 6 項 + 整合測試 `search_api_*` | 高 |
| GAP-G-01 | G1.1 八資料態缺具體測試名 | DoD 無可驗證證據 | 補上：`states_page_covers_eight_data_states`、`import_zero_result_vs_empty` 等 | 中 |
| GAP-G-02 | G1.3 角色起始頁缺 role_code 對映表 | 實作時需自行推導 | 補上角色對映表（FINANCE/STORE/ADMIN/UNKNOWN → 區塊） | 中 |
| GAP-G-03 | G2.1 DLQ 缺 schema 定義 | 實作時 schema 未定 | 補上 `DeadLetter {event_id, aggregate_type, error, attempt, failed_at}` 結構（eventbus 已有，platform 需對映） | 中 |
| GAP-G-04 | G3.1 證據四要件缺控制碼映射表 | 控制碼為範例性質，未定案 | 補上 `CONTROL_LIBRARY` 對映範例（CTRL-POST→ISO-27001-A.8.2 等） | 中 |
| GAP-RP-01 | RP-7 pandoc 工具可用性未驗證 | 渲染路徑可能受阻 | 先驗證 `pandoc --version`，不可用則改 `grip` 或線上渲染 | 中 |
| GAP-RP-02 | RP-8 `1794395575776` 懸空引用未解 | 兩份文件引用不存在的計畫 | 補建 stub 或修正引用（擁有者裁決） | 中 |
| GAP-K-01 | PHASE-K 缺 API 端點契約 | 承接專案無實作依據 | 補上 RESTful 端點清單（/api/kb/search, /api/kb/answer 等） | 中 |
| GAP-K-02 | PHASE-K 缺向量庫 schema | 承接專案無資料模型 | 補上 `VectorDoc {doc_id, content_hash, embedding: Vec<f32>, model, updated_at}` | 中 |
| GAP-K-03 | PHASE-K 缺成本模型 | 無法評估營運成本 | 補上：OVH bge-m3 免金鑰→$0；Ollama 本地→$0；P1-P4 階段性評估 | 低 |
| GAP-K-04 | PHASE-K 缺無障礙要求 | L1 使用者層無 WCAG 規範 | 補上 WCAG 2.2 AA 對齊（與工作台一致） | 中 |
| GAP-K-05 | PHASE-K 缺遷移路徑 | 從 in-memory 到向量庫無路線 | 補上 Phase 1 in-memory 增強→Phase 2 向量庫評估→Phase 3 持久化 | 中 |
| GAP-RP-03 | 全計畫缺 rollback plan | 失敗時無回滾依據 | 補上：每切片的 feature-flag 回滾 + 資料庫還原 | 中 |
| GAP-RP-04 | 全計畫缺 timeline/milestones | 無法追蹤進度 | 補上：PHASE-G1～G3 已完成的實際日期；PHASE-RP 待辦的預估日期 | 中 |
| GAP-RP-05 | 全計畫缺 communication plan | 跨線協作無規範 | 補上：board 協調＋每週 status update＋escalate 路徑 | 中 |
| GAP-RP-06 | 全計畫缺 quality metrics（文檔層） | 文檔品質無量化指標 | 補上：可讀性（Flesch-Kincaid）、搜尋命中率、連結有效性 | 中 |
| GAP-RP-07 | 全計畫缺 i18n 規範 | 僅繁中，無多語擴展 | 補上：UI 文字繁中預設；未來 i18n 走 `rust-i18n` 或 `fluent`（非本計畫範圍） | 低 |
| GAP-RP-08 | 全計畫缺 security 規範（知識系統） | K 系統無資安基線 | 補上：RBAC（沿用現有 roles）、CSRF（POST）、rate limit、audit_log（沿用） | 中 |

## S.2 補強後的規格增補

## S.2.1 測試證據補強（GAP-RAG-01/G-01；2026-11-12 逐名實證）

| 切片 | 測試名（實際；行號為 2026-11-12 實證） | 證據位置 | 核驗 |
|---|---|---|---|
| RAG-S1 | `rarer_terms_score_higher_than_common_terms`, `tokenize_handles_mixed_ascii_and_cjk`, `unknown_term_scores_zero_and_empty_index_is_safe`, `higher_term_frequency_ranks_higher`, `empty_and_punctuation_only_texts_tokenize_to_nothing` | `platform/src/bm25.rs` tests | ✅ 5/5 命中 |
| RAG-S2 | `bm25_only_degradation_ranks_by_relevance_and_marks_via`, `unknown_term_returns_empty_and_query_ordering_holds`, `fusion_blends_lexical_and_vector_with_via_provenance`, `refusal_floor_drops_weak_vector_only_hits`, `stale_vector_is_treated_as_missing`, `delete_rebuilds_index_and_document_count`, `rrf_fuse_preserves_refusal_floor`, `rrf_fuse_is_scale_invariant_where_weighted_is_not`, `search_ranked_uses_rrf_by_default` | `platform/src/search.rs` tests | ✅ 9/9 命中（原表 5 名中 `hybrid_path_with_injected_vectors` 不存在） |
| RAG-S3 | **無專屬整合測試**：`/api/search` 僅路由接線（`search_query`/`search_get`，via="direct"） | `platform/src/cross_module_api.rs:435-436` | ❌ 缺口 D2-02：需補 `search_api_*` 整合測試 |
| G1.1 | `states_page_covers_eight_data_states` | `platform/src/api.rs:8204` | ✅（原表示例 `import_zero_result_vs_empty` 不存在，已刪） |
| G1.2 | `pagination_links_preserve_search_term`, `detail_back_link_preserves_list_context` | `platform/src/api.rs:8248,8283` | ✅ |
| G1.3 | `finance_landing_leads_with_review_queue`, `store_landing_leads_with_reports`, `admin_landing_defaults_to_overview`；VIEWER 無示範帳號 → 未覆蓋 | `platform/src/api.rs:8370,8392,8411` | 🟡 3/4（缺口 D2-05） |
| G1.4 | 批量部分失敗 + 自我裁決 403（api.rs 測試群，未逐一核名） | `platform/src/api.rs` | 🟡 待逐名核（D2-02） |
| G2.1 | `duplicate_consume_never_reapplies`, `exhausted_attempts_dead_letter_and_backoff_is_exponential_bounded`, `dead_letter_record_then_signed_replay` | `platform-hub/crates/eventbus/src/lib.rs:306,317,350` | ✅（原表第 2 名不完整） |
| G2.2 | `threshold_failures_open_and_fast_fail`, `open_recloses_via_half_open_probes`, `half_open_failure_reopens_immediately` | `platform-hub/crates/channels/src/lib.rs:499,521,547` | ✅ |
| G2.3 | `customer_record_roundtrip_serde`, `invoice_record_roundtrip_serde`, `payment_record_roundtrip_serde`, `minor_cents_boundary_is_explicit_and_lossless` | `platform-hub/crates/udm/src/lib.rs:309,326,345,366` | ✅（原名缺 `_serde`／`_and_lossless`） |
| G2.4 | `reconciliation_report_matches_hand_computation`, `reconciliation_lists_are_deterministic_regardless_of_input_order`, `reconciliation_sample_cap_holds` | `platform-hub/crates/udm/src/lib.rs:397,434,444` | ✅ |
| G2.5 | `rate_limiter_blocks_burst_and_recovers_after_window`, `agent_writes_are_draft_only`, `audit_marker_renders_agent_identity_and_classification` | `platform-hub/crates/agent-policy/src/lib.rs:227,254,271` | ✅（原表 3 名均不精確） |
| G3.1 | `evidence_export_includes_four_requirements_and_rows`, `evidence_export_requires_role_manage` | `platform/src/api.rs:8833,8891` | ✅ |
| G3.2 | `every_library_control_maps_at_least_one_event_and_framework`, `full_observation_yields_no_findings` | `platform/src/controls.rs:123,162` | ✅（無 `control_mapping_*`；實際名如上） |
| G3.3 | `ccm_indicators_detect_invariant_and_signature_violations` | `platform/src/api.rs:8944` | ✅ |

## S.2.2 角色對映表（GAP-G-02）

| role_code | 首屏區塊 | 可見 KPI |
|---|---|---|
| FINANCE | 待覆核隊列 | 待覆核交易、月報金額 |
| STORE | 報表入口 | 月報、樞紐分析 |
| VIEWER | 報表入口（唯讀） | 月報、樞紐分析 |
| ADMIN | 總覽 | 全部 KPI |
| UNKNOWN/預設 | 總覽 | 交易總筆數、啟用科目數 |
| （無角色） | 總覽 | 同上 |

## S.2.3 DLQ Schema（GAP-G-03）

```rust
// eventbus/src/lib.rs 已有；platform 映射時沿用
pub struct DeadLetter {
    pub event_id: Uuid,
    pub aggregate_type: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub error: String,
    pub attempt: i32,
    pub failed_at: DateTime<Utc>,
}
```
- 映射：`OutboxEvent → DeadLetter`（event_id = id, aggregate_type 保留, attempt 保留, failed_at = created_at）

## S.2.4 控制庫對映範例（GAP-G-04）

| 控制 ID | 事件型別 | 框架對映 |
|---|---|---|
| CTRL-POST | TRANSACTION_POSTED | ISO-27001-A.8.2, INTERNAL-IC-01, BANK-FSC-3 |
| CTRL-IMPORT | IMPORT_CONFIRMED | ISO-27001-A.8.9, INTERNAL-IC-02, BANK-FSC-3 |
| CTRL-REVIEW | REVIEW_BULK, REVIEW_BULK_SUMMARY（批次摘要，D-5/B-1） | ISO-27001-A.5.3, INTERNAL-IC-03 |
| CTRL-CRED | PASSWORD_RESET | ISO-27001-A.5.17, INTERNAL-IC-04 |
| CTRL-AI | AI_SUGGEST_REQUESTED | INTERNAL-IC-05 |

> 核驗：上表與 `platform/src/controls.rs:25-59` 之 `CONTROL_LIBRARY` 逐筆一致（2026-11-12 實證）；`control_for_event`／`lint_control_coverage` 為其查詢與 lint 介面。

## S.2.5 PHASE-K API 端點契約（GAP-K-01）

| 端點 | 方法 | 權限 | 說明 |
|---|---|---|---|
| `/api/kb/search` | GET | 會話 | 混合檢索（BM25+vector），回 `Vec<{doc_id, score, via}>` |
| `/api/kb/answer` | POST | 會話＋LEDGER_READ | RAG 綜答（回 `{answer, citations: [{doc_id, version, section}]}`） |
| `/api/kb/index` | POST | 會話＋ROLE_MANAGE | 重建索引（觸發全量 embed） |
| `/api/kb/doc/:id` | GET | 會話 | 單文檔檢索 |
| `/api/kb/stats` | GET | 會話＋ROLE_MANAGE | 索引統計（文檔數、向量維度、embedder 狀態） |

## S.2.6 向量庫 Schema（GAP-K-02）

```rust
// 承接專案實作；本計畫僅定義結構
pub struct VectorDoc {
    pub doc_id: String,
    pub content_hash: String,   // SHA-256 of content
    pub embedding: Vec<f32>,    // 1024-dim (bge-m3)
    pub model: String,          // embedder label (provenance)
    pub updated_at: DateTime<Utc>,
}
```
- 儲存：pgvector（PostgreSQL 擴展）或 Qdrant（獨立服務，POC 後決定）
- 索引：HNSW（近似最近鄰）
- 維度：1024（bge-m3）或 768（Ollama 本地 fallback）

## S.2.7 成本模型（GAP-K-03）

| 項目 | 成本 | 觸發條件 |
|---|---|---|
| OVH bge-m3 | $0（免金鑰、免費 tier） | 預設（embed_batch） |
| Ollama 本地 | $0（本地執行） | OVH 不可達時 fallback |
| 伺服器記憶體 | 8GB 以上 | 1024 維 × 1 萬文檔 ≈ 40MB（可接受） |
| 儲存 | SSD | 向量庫索引 < 1GB |

## S.2.8 無障礙要求（GAP-K-04）

- **WCAG 2.2 AA**：語義 HTML、可見焦點、鍵盤導航、`prefers-reduced-motion`
- **零 JS**：`<script>` 計數 = 0（與工作台一致）
- **繁中預設**：UI 文字繁體中文（台灣情境）
- **顏色對比**：≥4.5:1（AA 標準）
- **表單標籤**：每個 `<input>` 有對應 `<label>`

## S.2.9 遷移路徑（GAP-K-05）

| 階段 | 內容 | 前置條件 |
|---|---|---|
| Phase 1 | 現有 in-memory `SearchEngine` 增強（BM25+vector 混合） | ✅ 已完成（RAG-S1~S3） |
| Phase 2 | 評估向量庫（pgvector/Qdrant） | 效能基準（1 萬文檔） |
| Phase 3 | 持久化（重啟不遺失） | Phase 2 通過 |

## S.2.10 Rollback Plan（GAP-RP-03）

| 切片 | 回滾策略 |
|---|---|
| G1.1~G1.4 | Git revert + 重跑四關（PIN 不變） |
| G2.1~G2.5 | platform-hub 獨立回滾（feature flag） |
| G3.1~G3.3 | DB migration 還原（备份還原） |
| RAG-S1~S4 | 移除 `bm25.rs` + `search.rs` 回退到舊版 |
| PHASE-K/RP | 移交件——承接專案自行 rollback |

## S.2.11 Timeline（GAP-RP-04）

| 里程碑 | 預計日期 | 狀態 |
|---|---|---|
| PHASE-G1 完成 | 2026-11-09 | ✅ 完成 |
| PHASE-G2 完成 | 2026-11-09 | ✅ 完成 |
| PHASE-G3 完成 | 2026-11-09 | ✅ 完成 |
| RAG-S1~S4 完成 | 2026-11-09 | ✅ 完成 |
| RP-1 完成 | 2026-11-11 | ✅ 完成 |
| RP-2 四方整合指引 | 2026-11-12 | ⬜ 待執行 |
| RP-3 演進程序 | 2026-11-12 | ⬜ 待執行 |
| RP-5 RPA-ECOM 家族調和 | 2026-11-12 | ⬜ 待執行 |
| RP-6 全域盤點登錄 | 2026-11-12 | 🟡 大半完成 |
| RP-7 雙報告 HTML 渲染 | 2026-11-13 | ⬜ 待執行 |
| RP-8 懸空引用修復 | 2026-11-13 | ⬜ 待執行 |

## S.2.12 溝通計劃（GAP-RP-05）

| 項目 | 頻率 | 參與者 | 產出 |
|---|---|---|---|
| 進度更新 | 每週 | architect + code agent | PLAN_REGISTER addendum |
| 契約仲裁 | 需要時（ESC-2） | architect 主持 | ADR 或計畫修訂 |
| 阻塞性升級 | >48h | board 宣告 | addendum + 擁有者通知 |
| 憲章級衝突 | 立即 | 擁有者 | ESC-4 停線 |

## S.2.13 文檔質量指標（GAP-RP-06）

| 指標 | 目標 | 量測方式 |
|---|---|---|
| 可讀性（Flesch-Kincaid） | ≥ 60（大學程度） | `textstat` 或線上工具 |
| 搜尋命中率（top-5） | ≥ 80% | golden queries 測試 |
| 連結有效性（零死鏈） | 100% | `linkcheck` 或 `markdownlint` |
| 文件覆蓋率（route ↔ doc） | 100% | 機械比對 |
| 最新更新（days） | ≤ 7 天 | `git log --since` |

## S.2.14 資安基線（GAP-RP-08）

- **認證**：沿用現有 Argon2id + 伺服器端會話（24h 可撤銷）
- **授權**：沿用 RBAC（roles/permissions/user_roles 即時查詢）
- **CSRF**：所有 POST（sync token）
- **速率限制**：登入 5 次/15 分、API 限流
- **Audit**：所有 POST 寫入 audit_log（沿用現有 event_type 約定）
- **Secrets**：不進版控（沿用 .cargo/audit.toml）
- **傳輸**：HTTPS（TLS 1.2+）

## S.2.15 i18n 規範（GAP-RP-07）

- **預設語言**：繁體中文（台灣）
- **未來擴展**：`rust-i18n` 或 `fluent`（需獨立評估）
- **UI 文字**：全部在模板中，易於替換
- **日期/數字**：台灣格式化（YYYY-MM-DD、千分位逗號）

## S.2.16 缺口修復優先順序

| 優先 | 缺口 ID | 理由 |
|---|---|---|
| P0 | GAP-RAG-01, GAP-G-01, GAP-K-04 | 追溯性與可驗證性——直接影響驗收 |
| P1 | GAP-G-02, GAP-G-03, GAP-G-04, GAP-K-01, GAP-K-02, GAP-RP-03, GAP-RP-04, GAP-RP-05 | 實作可執行性 |
| P2 | GAP-K-03, GAP-K-05, GAP-RP-06, GAP-RP-07, GAP-RP-08 | 長期維護性 |

> 註：GAP-RP-01（pandoc 可用性）與 GAP-RP-02（`1794395575776` 懸空引用）屬**執行步驟類**缺口——分別於 RP-7（渲染前先驗證 `pandoc --version`）與 RP-8（補建 stub 或修正引用，擁有者裁決）執行時處理，不另立規格節（見 RP.4 待辦登記）。

---

# SPEC-GAP-2 — 深度稽核（第二輪，2026-11-12）

> 觸發：擁有者指示「必須深度挖掘與拓展規格，規格一定充滿漏洞與不足」。本輪不以文件對文件，而以**程式碼實證**逐項核對 v8.1 之規格主張，列出 16 項缺口、修復與新增規格（下節 TECH-SOTA～A11Y 為補全文）。

## D2.0 稽核方法

1. **規格↔程式對照**：每個 `[x]` 切片與 S.2 表之測試名、struct、常量逐一 grep 核對（本輪已核：bm25/search/eventbus/channels/udm/agent-policy/controls/api）。
2. **事實↔證據對照**：以行號、struct 欄位、常量值為準；不得引用未核實之測試名。
3. **失效模式推演（FMEA/STPA）**：每個新設計點問「若 X 失效，誰發現、多快、如何回滾」。
4. **邊界壓力**：零/空/巨大/並發/離線/降級六情境。

## D2.1 缺口清單（含修復與證據）

| ID | 缺口 | 證據 | 修復 | 嚴重度 |
|---|---|---|---|---|
| D2-01 | RAG-S2 規格陳述「α=0.5 加權融合」與實作不符（實為 **RRF k=60 預設**；加權僅 `#[cfg(test)]` 對照） | `search.rs:36-40,88-95,176-178,425-432` | ✅ 已修（R.1／R.5 改寫） | 高 |
| D2-02 | RAG-S3 無整合測試（`/api/search` 僅路由接線）；G1.4 測試未逐名核 | `cross_module_api.rs:435-436` | 補 `search_api_*` 4 例；G1.4 逐名核（交 code agent） | 高 |
| D2-03 | S.2.1 多個測試名不精確/不存在（agent-policy 3、udm 4、eventbus 1、search 1、bm25 0） | 本輪 grep 實證 | ✅ 已修（S.2.1 表逐名校正） | 中 |
| D2-04 | `Decimal4`（10^4 固定小數）未納入單位契約 | `udm:26-28` | 新增單位契約條文（MoneyCents vs Decimal4 禁止互轉） | 中 |
| D2-05 | `VIEWER` 角色無 seed 帳號、無測試 | `seed-demo.ps1:113-114`；`index.html:112` | 新增 VIEWER seed/測試或標「未啟用」 | 中 |
| D2-06 | API 目錄敘述 stale（「Tantivy」）與現行 BM25+RRF 不符 | `apihub.rs:212-213` | 改寫＋文件↔路由一致性測試 | 中 |
| D2-07 | 無跨裝置/響應式規格（擁有者硬需求：跨平台跨裝置） | 全計畫缺口 | 新增 **UX-XDEVICE** | 高 |
| D2-08 | 無 SLO/SLI、可觀測性、備份/DR | 全計畫缺口 | 新增 **OPS-SLA** | 高 |
| D2-09 | 無 API 錯誤契約/分頁/冪等/版本化 | 全計畫缺口 | 新增 **API-CONTRACT** | 高 |
| D2-10 | 無資料保留/PII/匯出/刪除 | 全計畫缺口 | 新增 **DATA-GOV** | 中 |
| D2-11 | 無效能預算（每頁 p95/TTFB/傳輸量） | 全計畫缺口 | 新增 **PERF-BUDGET** | 高 |
| D2-12 | 無測試策略（金字塔/屬性/變異/模糊/混沌） | 全計畫缺口 | 新增 **TEST-STRATEGY** | 中 |
| D2-13 | 無 feature flag 機制（S.2.10 rollback 依賴） | `S.2.10` | 新增 **FLAG** | 中 |
| D2-14 | 無 SOTA 採用準則（何時可加依賴） | 全計畫缺口 | 新增 **TECH-SOTA**（五閘門） | 中 |
| D2-15 | 無複雜度/刪除預算（LEAN） | 全計畫缺口 | 新增 **LEAN** | 中 |
| D2-16 | 無無障礙測試工具鏈與驗收腳本（S.2.8 僅原則） | `S.2.8` | 新增 **A11Y** 檢查表 | 中 |

## D2.2 新增規格摘要（全文見下列各節）

- **單位契約**：`MoneyCents`（1/100）與 `Decimal4`（10^4）不得互轉；跨型別一律 `TryFrom` 並在型別名標示 scale。
- **角色矩陣**：FINANCE/STORE/VIEWER/ADMIN/UNKNOWN 五態；VIEWER 須 seed 或明確標 disabled。
- **錯誤契約**：`{code, message, hint, retryable, request_id}`；hint 含「問題＋可能原因＋處理方式」。
- **分頁**：cursor 優先（`next_cursor`）；offset 分頁須保留 `q` 與 filters。
- **冪等**：POST 寫入接受 `Idempotency-Key`；同鍵重放回原結果。
- **效能預算**：工作台首屏 p95 ≤500ms（既有 422ms）；`/api/search` p95 ≤800ms（向量）/≤200ms（BM25-only）；TTFB ≤200ms；HTML ≤100KB（未壓縮）。
- **跨裝置**：320px 無橫向捲動；點擊 ≥44px；200% 縮放可用；鍵盤全通；列印樣式。
- **可觀測性**：`tracing` 結構化日誌＋`/health`＋`/ready`＋請求 ID；無付費 APM 依賴。
- **備份**：SQLite 每日 `VACUUM INTO`（RPO ≤24h、RTO ≤1h）；PostgreSQL 線 `pg_dump`（OQ-GOV-01 後）。
- **測試**：單元 ≥70%；金額/搜尋/控制/事件 ≥90%；屬性＋變異＋模糊＋混沌依 TEST-STRATEGY。
- **SOTA 採用**：四環雷達＋五閘門；預設不加依賴。

---

# TECH-SOTA — 免費 SOTA 技術雷達（2026-11-12）

> 原則：**免費（FOSS）優先、量測後採用、可回滾、零 JS 不變**。每項須通過五閘門：①授權（MIT/Apache-2.0）②維護（近 12 月有更新）③安全（`cargo audit`／OSV 無已知 CVE）④量測（POC 前後數據）⑤回滾（feature flag／可移除）。未過即不得進入 `Cargo.toml`。

| 環 | 技術 | 現況 | 裁決 | 理由/條件 |
|---|---|---|---|---|
| Adopt | Rust std BM25＋RRF（k=60） | ✅ 已用 | 維持 | 零依賴、尺度不變、已驗證（F11） |
| Adopt | `kem` embedder 池（bge-m3→Ollama） | ✅ 已用 | 維持 | 免金鑰、$0、斷路器保護（F12） |
| Adopt | Argon2id 認證 | ✅ 已用 | 維持 | OWASP 建議 |
| Adopt | SQLx＋SQLite（本機）／PostgreSQL 18（OQ-GOV-01） | ✅ | 維持 | 型別安全、編譯期檢查 |
| Adopt | Askama 模板（型別安全 HTML） | ✅ 已用 | 維持 | 零 JS、編譯期 escaping |
| Adopt | `tracing`＋`tracing-subscriber` | 評估 | 建議 Adopt | 結構化日誌標準；輕量 |
| Trial | `proptest`（屬性測試） | 評估 | 建議 Trial | 金額/日期/分頁不變量；純測試依賴 |
| Trial | `cargo-mutants`（變異測試） | 評估 | 建議 Trial | 驗證測試強度；僅 CI |
| Trial | `cargo deny`（授權/公告/來源） | 評估 | 建議 Trial | 供應鏈治理；CI 執行 |
| Trial | `cargo-fuzz`（libFuzzer/AFL） | 已部分 | 維持 Trial | GAP-04 ASan DLL 阻塞；Linux CI 可跑 |
| Trial | `insta`（快照測試） | 評估 | 建議 Trial | HTML 快照回歸；開發期 |
| Trial | `tokio-console` | 評估 | 建議 Trial | 本地診斷，不入生產 |
| Assess | `mimalloc`／`jemallocator` | 評估 | Assess | 須量測配置器收益；可能無需 |
| Assess | `tower-http`（trace/compression） | 評估 | Assess | 僅在需要 gzip/trace 時；不影響零 JS |
| Assess | `axum-extra`（multipart 等） | 評估 | Assess | 僅在需要時；避免膨脹 |
| Assess | `rust-i18n`／`fluent` | 評估 | Assess | 僅多語需求出現時 |
| Hold | Tantivy／Qdrant／pgvector | 否決/評估 | Hold | Tantivy 編譯事故；向量庫待 K.5 評估 |
| Hold | 前端框架（React/Vue） | 否決 | Hold | 違憲章零 JS |
| Hold | Electron/Tauri 重寫本計畫 | 不適用 | Hold | 既有 Tauri 殼由獨立線維護 |

**採用程序**：新依賴 → ADR（五閘門證據）→ 隔離 `CARGO_TARGET_DIR` POC → 四關量測（含建置時間/二進位大小）→ rollback 條件 → 合併。`cargo tree -d`（重複依賴）與 `cargo machete`（未使用）列入每季檢查。

---

# LEAN — 精簡與持續演化

| 項目 | 現況（2026-11-12 實證/待量測） | 預算/目標 | 機制 |
|---|---|---|---|
| `api.rs` 行數 | **5,950（non-test）／9,059（含 tests；2026-11-12 實測）** | 單檔 ≤1500（R1 解鎖後） | PRE-R1b extractor 化；R1 模組化 |
| 重複 CSRF 檢查 | 14 處（2026-11-12 實測） | 1 處 | PRE-R1b（axum extractor） |
| 路由數（api.rs production） | 78（2026-11-12 實測） | 不變（重構不改路由） | PRE-R1a PIN 覆蓋 |
| 依賴數（platform） | 待 `cargo tree` | 不增加（除 POC 通過） | 每季盤點 |
| 未使用依賴 | 待 `cargo machete` | 0 | CI 加入 |
| 測試/程式比 | 待量測 | ≥0.7 | coverage 報告 |
| 認知複雜度 | 待掃描 | 每函式 ≤25 | clippy `too_many_lines`／人工審 |
| 增量建置 | 待量測 | ≤60s | `sccache`（免費） |
| 二進位大小 | 待量測 | 記錄基線 | `cargo bloat`（僅分析） |
| 刪除優先 | `Decimal4` 用途未定；`fuse` 為測試對照 | 每季刪/併 ≥1 項 | 「使用或刪除」規則 |
| 文件冗餘 | 同名報告已消歧；Kilo 快照 ~70 份 | 索引化不重複 | PLAN_REGISTER 雙登記 |

**LEAN 三律**：①「未被測試或文件引用之程式＝刪除候選」；②「新功能先問能否以既有元件組合」；③「每次迭代至少刪除或合併一處」。

---

# UX-XDEVICE — 跨平台跨裝置體驗（彌補 D2-07）

> 擁有者硬需求：跨平台跨裝置可用。本節為驗收規格（非框架改造）；與零 JS、WCAG 2.2 AA 並行。

| 維度 | 規格 | 驗證 |
|---|---|---|
| 視窗寬度 | 320 / 375 / 768 / 1024 / 1440 / 1920 px 六斷點 | media/container query 檢查；320px 無橫向捲動 |
| 點擊目標 | ≥44×44 CSS px（WCAG 2.5.8 AAA 建議；AA 為 24） | 靜態樣式檢查 |
| 縮放 | 200% 文字縮放不破版、不遺失內容 | 瀏覽器縮放實測 |
| 鍵盤 | 全部互動可 Tab 到達；焦點可見（`:focus-visible`） | 鍵盤走查 |
| 減少動態 | `prefers-reduced-motion` 停用動畫 | CSS 檢查 |
| 色彩偏好 | `prefers-color-scheme` 淺/深色（Token 化） | 樣式檢查 |
| 列印 | 報表/對帳/證據匯出有 `@media print` | 預覽列印 |
| 瀏覽器 | Chromium／Firefox／Safari（WebKit）最新兩版 | Playwright（可用時）或手動矩陣 |
| 裝置 | 桌面／平板／手機（無原生 app 需求；Tauri 殼另線） | 響應式模式 |
| 離線 | 不承諾離線；`/health` 明示降級 | 產品邊界 |
| 低階效能 | 4× CPU 節流首屏 ≤2s | Lighthouse／手動節流 |
| 無障礙 | WCAG 2.2 AA；零 JS；語義 HTML；表單 label | axe-core（**測試專用**，離線注入；不進 production HTML）或手動檢查表 |

---

# OPS-SLA — 營運級別與可觀測性（彌補 D2-08）

| SLI | SLO | 量測 |
|---|---|---|
| 可用性（月） | ≥99.5%（內部工具情境） | `/health` 探測紀錄 |
| 工作台首屏 p95 | ≤500ms（基線 422ms） | stress 腳本（warm 21 樣本） |
| `/api/search` p95 | ≤800ms（向量）；≤200ms（BM25-only） | 新增 benchmark |
| 錯誤率 | <0.5%（5xx/總請求） | 結構化日誌 |
| 稽核鏈完整率 | 100% | CCM 指標（G3.3） |
| 備份成功率 | 100% | 備份腳本 exit code |

- **可觀測性**：`tracing`＋`tracing-subscriber`（JSON 到 stderr）；`X-Request-Id` 貫穿；`/health`（liveness）與 `/ready`（DB/索引就緒）。
- **備份/DR**：SQLite `VACUUM INTO` 每日、保留 7 日；RPO ≤24h、RTO ≤1h；還原演練每季（記錄於 runbook）。
- **機密**：環境變數；`.env` 不進版控；工作階段金鑰/密碼每 90 天或事件驅動輪替。
- **事件管理**：P1（不可用）30 分鐘內回報擁有者；P2（降級）1 個工作日；每件附時間線與根因（FMEA/STPA）。

## OPS-SLA 附：STRIDE 威脅建模（摘要）

| 類別 | 威脅 | 控制 |
|---|---|---|
| Spoofing | 工作階段劫持/偽冒 | `HttpOnly`＋`SameSite=Lax`＋`Secure`；Argon2id；24h 可撤銷 |
| Tampering | 稽核鏈竄改 | kem SHA-256 鏈；append-only；CCM 鏈驗證（G3.3） |
| Repudiation | 否認操作 | audit_log（who/what/when）；DLQ 簽名重放 |
| Information disclosure | PII/交易外洩 | RBAC 最小權限；向量僅存內容 hash；TLS 1.2+ |
| Denial of service | 嵌入/查詢 flood | 斷路器 2s/60s；速率限制 30 次/分；查詢長度上限 |
| Elevation of privilege | 角色提升 | 伺服器端 `role_code`；POST 不得自帶 role；CSRF token |

---

# API-CONTRACT — API 契約與錯誤分類（彌補 D2-09）

- **錯誤格式（JSON）**：`{"error": {"code": "…", "message": "…", "hint": "問題＋可能原因＋處理方式", "retryable": false, "request_id": "…"}}`。
- **代碼表**：`VALIDATION`／`UNAUTHENTICATED`／`FORBIDDEN`／`NOT_FOUND`／`CONFLICT`／`RATE_LIMITED`／`UPSTREAM_UNAVAILABLE`／`INTERNAL`。
- **HTML 頁錯誤**：沿用「問題＋可能原因＋處理方式」三段式（憲章 §4）。
- **分頁**：cursor 優先（`next_cursor`）；既有 offset 分頁須保留 `q` 與 filters（G1.2 已驗證）。
- **冪等**：POST 寫入接受 `Idempotency-Key`；同鍵重放回第一次結果（不重播副作用）。
- **版本化**：`/api/v1/*` 明確版本；破壞性變更需新版本＋90 天重疊（內部 API 可例外，需 ADR）。
- **速率限制**：登入 5 次/15 分；`/api/search` 30 次/分/工作階段；超限 429＋`Retry-After`。
- **CSRF/工作階段**：所有 POST 同步 token（14 表單）；工作階段 24h 可撤銷；Cookie `HttpOnly`＋`SameSite=Lax`＋`Secure`（TLS 環境）。

---

# DATA-GOV — 資料治理與隱私（彌補 D2-10）

| 類別 | 範例 | 保留 | 處置 |
|---|---|---|---|
| 交易/金額 | transactions, audit_log | 依會計法規（建議 7 年；擁有者裁決） | 封存不刪；可匯出 CSV |
| 個人資料 | 使用者名稱、角色 | 帳號存续期＋離職後 90 天 | 匿名化/刪除；可匯出 |
| 工作階段 | sessions | 24h TTL | 自動過期 |
| 索引/向量 | search index, vectors | 可重建 | 可刪；向量僅存內容 hash，不存 PII |
| 稽核鏈 | kem ledger | 永久 | 只增不刪（append-only） |

- **匯出**：使用者請求 → JSON＋CSV（交易、稽核、個人資料）。
- **刪除**：帳號停用即撤銷工作階段（已實作）；硬刪除需 DBA 程序＋紀錄。
- **遷移**：SQLite→PostgreSQL（OQ-GOV-01）以 SQLx 遷移；每步可回滾（down migration）。

---

# PERF-BUDGET — 效能預算（彌補 D2-11）

| 指標 | 預算 | 量測 |
|---|---|---|
| 工作台首屏 p95 | ≤500ms（現況 422ms） | stress（warm 21 樣本） |
| `/api/search` p95（向量） | ≤800ms | 新增 benchmark |
| `/api/search` p95（BM25-only） | ≤200ms | 斷網情境 |
| TTFB | ≤200ms | 本機/內網 |
| 每頁 HTML（未壓縮） | ≤100KB | `wc -c` 或測試斷言 |
| 嵌入逾時 | ≤2s（斷路器 60s） | `embed_with_breaker` |
| 索引重建（1 萬文件） | ≤5s（批次） | `add_documents_bulk` |
| 建置（增量） | ≤60s | CI 計時 |

> 超預算即視為回歸：擋合併、開 PERF 缺口、以 `EXPLAIN QUERY PLAN`／profiler 定位後才放行。
> 證據紀律（C-10）：上表記「預算」為**目標值**，非實測；未量測者標「待量測」，不得視為 PASS。現況僅「工作台首屏 p95 422ms（2026-11-07 stress）」與「嵌入 ≤2s／斷路器 60s」有實證。

---

# TEST-STRATEGY — 測試策略與品質閘門（彌補 D2-12）

| 層 | 工具 | 範圍 | 頻率 |
|---|---|---|---|
| 單元 | `#[test]`／`tokio::test` | 純函式/狀態機 | 每次提交 |
| 整合 | axum `oneshot`／TestServer | 路由×DB×模板 | 每次提交 |
| 屬性 | `proptest`（Trial） | 金額/日期/分頁/單位轉換不變量 | CI |
| 變異 | `cargo-mutants`（Trial） | money/search/controls/eventbus | 每週/每月 |
| 模糊 | `cargo-fuzz` | 解析器/模板/查詢 | 每季（Linux CI） |
| 混沌 | FAULT-01..07 | 斷網/DB 鎖/嵌入不可達/磁碟滿 | 每季 |
| 特徵化 | `characterization.rs` | api.rs 路由 PIN | 重構前必跑 |
| 效能 | stress 腳本 | p95/SLA | 發布前 |
| 無障礙 | 手動檢查表＋axe-core（離線） | 關鍵頁 | 每次 UI 變更 |

- **覆蓋率**：整體 ≥70%；money/search/controls/eventbus ≥90%。
- **測試資料**：fixture 種子（`seed-demo.ps1`）；不得用真實 PII。
- **失敗處理**：任一層失敗即擋合併；`#[ignore]` 僅限環境性（須註記理由與追蹤 ID）。

---

# FLAG — Feature Flag 與回滾機制（彌補 D2-13）

| 旗標 | 預設 | 作用 | 回滾 |
|---|---|---|---|
| `search_hybrid` | on | BM25＋向量融合（off＝BM25-only） | 環境變數/設定；即時 |
| `search_rrf` | on | RRF 融合（off＝加權 `fuse`） | 同上 |
| `ccm_chain_verify` | on | CCM 鏈驗證 | 同上 |
| `bankpay_esun_mtls` | off（OQ-F1/F2） | 玉山 mTLS 通道 | 同上 |
| `external_portal` | off（OQ-GOV-04） | 外部唯讀門戶 | 同上 |

- 旗標由環境變數/設定檔驅動（不硬編碼）；變更須記錄於 audit_log 與 runbook。
- 每個旗標須有「關閉後的正確行為」測試（雙向門）。

---

# A11Y — 無障礙驗收檢查表（彌補 D2-16）

1. 語義 landmark（`header/nav/main/aside/footer`）齊備。
2. 標題層級不重跳（h1→h2→h3）。
3. 每個表單控制有對應 `<label>`。
4. 焦點順序合理；`:focus-visible` 可見。
5. 全部互動可鍵盤完成（無 mouse-only）。
6. 非文字內容有替代文字或 `aria-hidden`。
7. 顏色對比 ≥4.5:1（本文/背景）。
8. 不以顏色為唯一傳達（錯誤/狀態另有文字/圖示）。
9. `prefers-reduced-motion` 停用非必要動畫。
10. 錯誤訊息含「問題＋可能原因＋處理方式」且與欄位關聯（`aria-describedby`）。

---

# 附錄：第二輪缺口修復優先序

| 優先 | 缺口 ID | 理由 |
|---|---|---|
| P0 | D2-01, D2-02, D2-07, D2-08, D2-09, D2-11 | 驗收/契約/跨裝置/效能——直接影響可用性 |
| P1 | D2-04, D2-05, D2-06, D2-12, D2-13, D2-16 | 實作可執行性與品質閘門 |
| P2 | D2-03, D2-10, D2-14, D2-15 | 長期維護性與精簡 |

---

**本計畫結束（v8.2）。** 後續執行由 code agent 依切片順序進行；每切片完成後更新 PLAN_REGISTER addendum。所有「完成」聲明以證據為準；本檔不構成部署、認證或合約授權。
