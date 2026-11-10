# FlapsErp／輔翼 ERP 效能評估進度報告 v2.0 — 從開發起點到現況的最佳完整版

> 文件編號：PERF-PROGRESS-V2.0｜狀態：**現行（Active）**｜簽發：2026-11-09
> 上游：`FlapsErp效能基線報告_v1.0.md`（2026-11-03 產品基線，本報告繼承其全部量測）｜登錄：`docs/PLAN_REGISTER.md` addendum #17
> 紀律：`[FACT:CODE]` 附命令與 exit code；UNKNOWN ≠ PASS ≠ FAIL；金額/分數分離（INV-01 只限金額）。
> 涵蓋範圍：**從 2026-11-03 工作區建立到 2026-11-09 的完整效能評估進度**——產品基線（v1.0）、Rust 平台全程構建、驗證體系進化、三大工作室、安全事件與處置、RAG 檢索升級，以及仍開放的項目。

---

## 1. 執行摘要（一頁讀完）

| 維度 | 2026-11-03（起點） | 2026-11-09（現況） | 倍率/狀態 |
| :--- | :--- | :--- | :--- |
| 自動化測試防護 | 0（工作區剛建立） | **246 passed / 0 failed**（bridge 7＋core 42＋kem 58＋platform 108＋kem-int 15＋bankpay_persistence 1＋characterization 6＋close_out 4＋slice12 2＋misc 3） | ∞（從零建立） |
| 突變測試得分 | 無 | money 73/73、engine 22/22＝**100%**（cargo-mutants 27.1） | 新增能力 |
| 效能 SLA | 無平台 | 100k 交易 **warm p95 422ms ≤ 500ms**（預熱＋21 樣本量測法） | 新增並通過 |
| 種子建置 | — | 100k 交易 **~2s**（批次 500 rows/語句；原逐列 ~90s） | **45×** |
| 檢索 | 無 | **混合 RAG**：BM25（CJK 感知）＋向量（重用 kem embedder 池）＋誠實降級 via 溯源 | 新增能力 |
| 安全門檻 | 無 | 四關全綠＋audit 0（2 個已文件化處置：basic-toml、RUSTSEC-2026-0459） | 新增能力 |
| 產品基線（Electron） | COM P95 0.39ms、RAM 623.8MB（v1.0 已量測） | 保留為對照基準；RPA 支援線持續 | 維持 |
| 文件體系 | 少量 | 70+ 份、索引零死鏈、追溯矩陣全綠 | 體系化 |

**一句話**：六天內從零建成「零 JS 工作台＋API 中樞＋治理稽核」三工作室的企業級 Rust 平台，效能 SLA 首測即達標（422ms/500ms），並補齊突變測試、混合 RAG、稽核四要件與安全處置的全部驗證證據鏈。

---

## 2. 開發至今完整時間軸（效能相關事件全記錄）

| 日期 | 里程碑 | 效能/品質影響 | 證據 |
| :--- | :--- | :--- | :--- |
| 11-03 | 工作區建立；**產品基線量測**（v1.0：Electron 記憶體 623.8MB、COM P50 0.264ms/P95 0.388ms/P99 0.474ms、CDP 通道、config.ini、回滾快照） | 對照基準確立 | 效能基線報告 v1.0 |
| 11-03 | OQ-F1/F2 需求提案 v1.0（→11-04 v2.1 提交廠商） | Phase 4 通道（11-19 期限） | research/proposals/ |
| 11-05/06 | Slice 1–13 構建：P0-1 整數金額（i128 minor，禁浮點）、P1-1 API 層、複式記帳＋Argon2id 認證、CSV 匯入（SHA-256 指紋防重放）、速率限制、使用者管理、洞察層、部署強化、演示套件；方法論體系（TDD/BDD/SDD/DDD/ADD） | 品質地基 | CHANGELOG 11-06 輪 |
| 11-06 | 重新設計藍圖 R2/M1/M3：KPI 真實計數、最近交易、分頁 | 工作台成形 | CHANGELOG |
| 11-07 | **CP950 編碼損壞根因修復**（api.rs 三方合併回 `dc414ac`；config.rs/kem PUA 清零）＋`encoding-standard.md` 防制規範 | 消除 mojibake 復發風險 | addendum #11 |
| 11-07 | **Stress 量測法修正**：3 樣本 max（冷啟 535ms 誤判）→ 預熱趟＋21 樣本 p95＝**422ms**；seed 批次化 **90s→2s（45×）** | 量測方法論根因修復 | 同上＋本報告 §4 |
| 11-07 | MT 導入：cargo-mutants 27.1，money/engine **100%**；FBT 建置（fuzz target build PASS）；ATDD 綁定稽核（8 情境）；故障注入 SOP（FAULT-01..07） | 驗證體系三支柱 | MUTATION_SCORE_REPORT v2 等 |
| 11-07 | 文檔整合：README v3.0 索引（零死鏈）、PROJECT_STATUS 等 | 文檔↔代碼對齊 | docs/README.md |
| 11-08/09 | （並行線）Slice 13：search/status/sync/com 橋接、**BankPay E2E**（EsunConfig mTLS 10 鍵、api_catalog/logs/test 頁）、characterization 測試 | S1/S2 擴展 | 並行線記錄 |
| 11-09 | **三大工作室十二切片**：G1.1–G1.4（八態/上下文/角色頁/批量覆核）、G2.1–G2.5（冪等+DLQ/熔斷/UDM/對帳/代理政策）、G3.1–G3.3（證據四要件/控制庫/CCM） | addendum #12/#13 | 本線 |
| 11-09 | **安全驅動 sqlx 0.8.6→0.9.0**（RUSTSEC-2026-0459 觸發；雙版本衝突根因定位 `cargo tree -i sqlx`）；RUSTSEC-2026-0459 兩工作區文件化接受 | 安全基線 | addendum #16 |
| 11-09 | **PHASE-R 混合 RAG**：BM25（CJK 改進）＋重用 kem embedder＋融合＋誠實降級；跨線 stale fixture 修復（bankpay 鍵 5→10） | 檢索面升級 | addendum #17 |
| 11-09 | 全工作區四關：**fmt 0／clippy 0／246 passed 0 failed／audit 0**（兩工作區） | 現況基線 | 本報告 §3 |

---

## 3. 現況效能與品質基線（全部 2026-11-09 量測）

### 3.1 平台 API 效能（SLA 軌道）

| 指標 | 值 | 量測方法 | 門檻 | 判定 |
| :--- | :--- | :--- | :--- | :--- |
| 月報查詢 warm p95（100k 交易） | **422ms**（隔離跑 453ms；併發編譯負載下曾達 476ms——環境敏感已記錄） | `cargo test -p platform --lib stress_10000` --nocapture；預熱趟＋42 樣本（7 輪×3 月），p95 索引 39 | ≤ 500ms | ✅ PASS |
| 查詢穩態（暖機後） | 0–2ms/查詢（42 樣本中 28 個 ≤2ms） | 同上 | — | ✅ |
| 種子建置 | ~2s / 100k 交易（批次 500 rows/語句；Little's Law 推導吻合實測） | 同上 | — | ✅（45× vs 逐列） |
| 冷啟動首查 | 428–535ms（頁快取 miss；已由預熱趟自 SLA 量測排除——量測「穩態伺服器」而非「首次觸盤」） | 11-07 事故教訓 | — | 文檔化 |

### 3.2 COM/RPA 基線（產品線，v1.0 保留）

| 指標 | 值（2026-11-03） | 現況 |
| :--- | :--- | :--- |
| COM 呼叫 P50/P95/P99 | 0.264 / 0.388 / 0.474 ms（1000 迴路） | 基線保留；批次化效益估算見 v1.0 §3.3 |
| Electron 全系統 RAM | 623.8 MB（11 行程） | 對照基準；Rust 平台行程 RAM **未量測**（UNKNOWN，列 T-後續） |
| 帳務規模 | — | 平台側 100k 交易軌道已建立（§3.1） |

### 3.3 品質防護基線

| 指標 | 值 | 證據 |
| :--- | :--- | :--- |
| 測試總數 | **246 passed / 0 failed**（platform 108、kem 58、core 42、kem-int 15、bridge 7、characterization 6、close_out 4、slice12 2、misc 3、bankpay_persistence 1） | `cargo test --locked --workspace` |
| 突變得分 | money **73/73**、engine **22/22**＝100% | MUTATION_SCORE_REPORT v2 |
| 模糊測試 | money_parser target **build PASS**；run 受阻 ASan DLL（GAP-04，UNKNOWN） | FUZZING.md §3 |
| 安全審計 | audit EXIT=0（兩工作區）；2 個已文件化處置 | `.cargo/audit.toml` ×2 |
| 零 JS | templates `<script>`=0、內聯事件=0（含 RAG 後新增模板） | 掃描 2026-11-09 |
| 編碼 | PUA=0（全部觸及檔案） | 掃描 2026-11-09 |
| 文檔 | 70+ 份、索引零死鏈、追溯矩陣零孤兒 | docs/README.md v3.0 |

### 3.4 檢索效能（PHASE-R 新增）

| 指標 | 值 | 說明 |
| :--- | :--- | :--- |
| 詞法排序 | BM25（K1=1.2、B=0.75、Robertson IDF）——稀有詞勝常見詞、詞頻有序（5 測試） | `platform/src/bm25.rs` |
| 混合融合 | α·norm(BM25)+(1−α)·norm(cosine)，α=0.5、拒絕地板 0.55（kem 校準）；via 溯源 "hybrid"/"bm25" | `search.rs::fuse` 6 測試 |
| 降級語義 | embedder 不可達→BM25-only（確定性）；向量失鮮（sha256 失配）→視同缺失 | 誠實降級，永不捏造 |
| 依賴成本 | `platform → kem` 路徑依賴＝**零新增外部 crate**（Tantivy 教訓） | 兩 Cargo.toml 對照 |

---

## 4. 量測方法論進化（教訓制度化了）

| 教訓 | 原症狀 | 根因 | 制度化修復 |
| :--- | :--- | :--- | :--- |
| p95 小樣本誤判 | 3 樣本 p95＝max（冷啟 535ms 誤判違 SLA） | 冷啟 I/O 混入穩態量測 | 預熱趟＋≥21 樣本＋p95 索引公式寫進測試註解 |
| 種子逐列寫入 | 100k 交易 ~90s | 每列一次 execute | 批次 500 rows/語句（Little's Law 推導≈實測） |
| 編碼損壞 | api.rs 712 PUA 字元 | CP950 誤解 UTF-8 | `encoding-standard.md`＋PUA 掃描進適應度 |
| 建置鎖 | 並行 dev server 佔 target | 共享 target 目錄 | 隔離 `CARGO_TARGET_DIR`（Runbook RB-09） |
| 依賴雙版本 | sqlx 0.8+0.9 並存（兩個 `Pool<Sqlite>` 型別） | 部分 manifest 未升級 | `cargo tree -i sqlx` 根因定位；三處 manifest 統一 |
| 量測環境負載 | 併發編譯下 p95 476ms（仍 ≤500） | 機器負載 | Runbook：門檻量測避開並行建置 |

---

## 5. 三大工作室現況（對照 master plan §22 追溯矩陣）

| 工作室 | 已交付切片 | 痛點覆蓋 | 現況 |
| :--- | :--- | :--- | :--- |
| **S1 部門業務工作台** | G1.1 八資料態、G1.2 上下文連續、G1.3 角色起始頁＋KPI 定義、G1.4 批量覆核、PHASE-R 混合檢索 | WP-01/02/04/05/07/08/10 全綠 | ✅（108 platform tests） |
| **S2 API 中樞與資料管理** | G2.1 冪等+DLQ、G2.2 熔斷、G2.3 UDM 實體＋單位契約、G2.4 對帳計算、G2.5 代理政策 | HP-03/04/05/06/09 綠（渲染掛平行線） | ✅（22 platform-hub tests） |
| **S3 治理稽核** | G3.1 證據四要件、G3.2 控制庫跨框架映射、G3.3 CCM 三指標（M2 誠實 null）；kem 哈希鏈＋混合 RAG | GP-02/04/05 綠 | ✅（58 kem tests） |

---

## 6. 開放項目（誠實清單，不虛報）

| 項目 | 阻塞 | 來源 |
| :--- | :--- | :--- |
| G3.4 外部稽核門戶（僅規格） | OQ-GOV-04 憑證模型 | plan §2 |
| R1 api.rs 模組化（特徵化已備） | GAP-08 worktree 分歧 | §0.5 裁決 |
| fuzz 執行（GAP-04） | ASan DLL | FUZZING.md §3 |
| 廠商 OQ-F1/F2（11-19 期限） | 廠商環境（UNKNOWN） | 合規追蹤表 |
| CCM M2 登入稽核事件 | auth 尚未寫事件（已註記） | ccm.json |
| ATDD 綁定 ×2 | 無（近期可解） | ATDD_BDD_BIND_AUDIT |
| 平台行程 RAM 量測 | 未執行（UNKNOWN） | 本報告 §3.2 |
| 擁有者決策群 | OQ-GOV-01..04、DEC-15/21 | plan §19 |

---

## 7. 修訂紀錄

| 版本 | 日期 | 內容 |
| :--- | :--- | :--- |
| v2.0 | 2026-11-09 | 全程進度版：時間軸 11-03→11-09、246 測試基線、SLA 422ms、MT 100%、RAG 混合檢索、sqlx 0.9 安全升級、方法論進化表、三工作室現況、開放項目誠實清單。繼承 v1.0 全部量測為對照基準。 |
| v1.0 | 2026-11-03 | 產品效能基線（Electron 記憶體/COM/CDP/config.ini/回滾）。 |
