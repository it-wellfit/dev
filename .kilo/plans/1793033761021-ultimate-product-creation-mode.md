# ULTIMATE PRODUCT CREATION MODE — 主計畫（定案，待執行）

> 計畫編號：1793033761021｜日期：2026-11-10｜狀態：**implementation-ready（Q1–Q8 全數定案；Product Brief 已確認；執行起點 B-1）**
> 方法論 SSOT：使用者提供之《ULTIMATE PRODUCT CREATION MASTER PROMPT》（PHASE 0–12）
> 上位執行基線：`.kilo/plans/1793787631643-three-studio-slice-execution-plan.md`（v7，G1–G3 全數完成）
> 衝突規則：本計畫不得牴觸上位基線之硬性約束；每階段產出先過 §5 約束再落地。

---

## 0. 模式宣告與當前階段

- 已啟動：ULTIMATE PRODUCT CREATION MODE（CPO/CTO/Principal Designer/UX/工程/QA 多角色協作）。
- 當前階段：**PHASE 1 Grill-me 已完成（Q1–Q8 逐題落盤於 §8）；PHASE 2–12 依 §9 任務序列推進**。
- 不得跳關：B-1 量測基線未完成前，不得進行任何 V2 動刀（C-03 先量測後動刀）。

## 1. 現況盤點（2026-11-10 證據基礎）

| # | 產品線／資產 | 現況 | 證據 |
|---|---|---|---|
| A1 | 三大工作室治理平台（根工作區 S1 工作台） | G1–G3 共 12 切片＋PRE-R1a **完成**；全工作區 246 passed / 0 failed；適應度全綠（零 JS script=0、PUA=0、MT 100%、warm p95 422ms≤500ms、CSRF 10/10、clippy allow=0、金額 f64=0） | three-studio-slice-execution-plan v7 §2 完成總覽 |
| A2 | ekpi-platform（OEKAP，ADR-0001 主產品） | T1–T10 完成（hybrid retrieval、OCR、multimodal、形式/執行期驗證、local auth/RBAC、backup、κ、CI）；fmt/clippy/build 綠；**測試執行被 Windows 應用程式控制原則阻擋（os error 4551）** | 記憶體 corrections：ekpi_platform_demo_readiness |
| A3 | Wellfit（獨立子專案） | wellfit_shopee／mtrix／bankpay 已具獨立冪等帳本＋重試熔斷＋SHA-256 審計鏈，**不再依賴 FlapsErp**；DEMO waves 已交付 | 記憶體：vendor_independence_flapserp |
| A4 | FlapsErp 逆向素材 | app.asar 讀取受 Bash 權限阻擋；`AppData\Roaming\flaps-erp` 僅空白偏好與日誌——**無可萃取現場設定** | 本 session 偵察紀錄 |
| A5 | 專案文件 | `project-document.docx`（16 章＋附錄、19,501 bytes、30+ 表格）已交付；**PDF 轉換工具鏈已定（Q4：LibreOffice headless），待安裝執行**（無 pandoc/wkhtmltopdf；python-docx 不可用；Bun 1.4.4＋docx@9.14.0 可用） | 本 session 交付紀錄 |
| A6 | 方法論收斂 | methodology-adoption 計畫 2 個未決問題已於 Q7 定案（proptest 條件採用、CI 併入 B-4） | 1793959524-methodology-adoption.md |
| A7 | 品質約束記憶 | KEM G4 需 ≥500 筆人工複核帳本（未達不得虛造）；zero_paid_ai（L0＋L1 免費鏈，L1.5 僅明示升級）；Strict Zero-JS First | project.md / corrections.md |

## 2. 多軌展開工作區（回應「多家展開」要求）

| 軌道 | 內容 | 依賴 | 狀態 |
|---|---|---|---|
| T-A 產品主軸深化 | Product Brief（§9）→ Design System → V1/V2/V3 迭代 | 無（Q1/Q2/Q3/Q8 已定案） | 🟢 **定案待執行** |
| T-B 文件工程 | DOCX→PDF（LibreOffice headless）；範本套用 Wellfit 先、SyncMesh 後 | 無（Q4/Q5 已定案） | 🟢 定案，隨 B-8 並行 |
| T-C FlapsErp 對接 | app.asar 逆向／設定介面確認→簽核矩陣落地（對照已存 `flaps-erp-ap-approval-plan` 邏輯） | 阻塞：需廠商原始碼或設定介面 | 🔴 阻塞（誠實標記） |
| T-D 方法論收斂 | proptest（僅當已在 Cargo.lock）＋CI workflow 併入 B-4 | 無（Q7 已定案） | 🟢 定案，併入 B-4 |
| T-E G 收尾 | G3.4 維持僅規格；PRE-R1b 維持可選不阻塞 | 無（Q6 已定案） | 🟢 定案 |

**防落後原則**：T-A 為主軸；T-B/T-D 於 B-1~B-7 等待點並行；T-C 維持阻塞誠實標記，不虛構進度。

## 3. PHASE 映射（主提示詞 → 本工作區具體產物）

| Master PHASE | 本工作區落點 | 硬約束掛鉤 |
|---|---|---|
| P1 Grill-me | §4 問題佇列（Q1–Q8 全數定案）＋§8 決策紀錄 | C-10 誠實註記（UNKNOWN≠PASS≠FAIL） |
| P2 研究 | B-2：官方文件優先（context7）；結論附來源＋日期＋成熟度 | 禁捏造數據 |
| P3 策略 | §9 Product Brief（已確認）→更新上位 §22 追溯 | ADR-0001 產品裁定 |
| P4 UI/UX | B-3：Design Token（`web/assets/design-system.css` 體系） | **Strict Zero-JS**（script=0、:target/:has()、八態、WCAG 2.2 AA、繁中） |
| P5 元件互動 | 每元件 Default/Hover/Focus/Disabled/Loading/Empty/Error 八態＋錯誤三要素 | web/index.html 現行基準 |
| P6 視覺素材 | SVG 無損、WebP/AVIF 預算（文件快照規範） | 禁無意義裝飾 |
| P7 工程 | Rust-first（C-11）；金額 i64/i128 minor 禁浮點；GET/POST/PRG；`#![forbid(unsafe_code)]` | 四關 EXIT=0 |
| P8 無障礙 | WCAG 2.2 AA、鍵盤、prefers-reduced-motion | AGENTS.md §4 |
| P9 審查 | 多角色 Review（CPO/UX/UI/Principal/QA）→記錄於本計畫 addendum | 並行編輯 C-05 基準對帳 |
| P10 迭代 | B-4 V1 地基→B-5 V2 體驗精煉→B-6 V3 頂級品質；每輪問題/原因/改善/驗證/殘留 | 適應度函數回退即擋 |
| P11 決策文件 | B-7 Decision Log：需求/選擇/原因/替代/優劣/證據/驗證 | §24 修訂軌跡慣例 |
| P12 交付 | 與實際完成度一致；未測不得稱通過；未作不得稱完成 | GOVERNANCE P7 |

## 4. Grill-me 問題佇列（全數定案）

| # | 問題 | 狀態 |
|---|---|---|
| Q1 | 主攻產品線與首位真實使用者＋首要核心任務 | ✅ D-1 |
| Q2 | 成功指標基準值：現值量測與目標值 | ✅ D-2 |
| Q3 | 設計方向三案取捨 | ✅ D-3 |
| Q4 | PDF 工具鏈 | ✅ D-4（LibreOffice headless） |
| Q5 | 文件範本套用順位 | ✅ D-4（Wellfit 先） |
| Q6 | G3.4/PRE-R1b 處置 | ✅ D-4 |
| Q7 | methodology-adoption 兩決策併入 | ✅ D-4 |
| Q8 | M-5 稽核查詢權限矛盾 | ✅ D-6（新增自我稽核視圖） |

## 5. 硬性約束（每階段產出必過）

Zero-JS（script=0）｜金額禁浮點（i64/i128 minor）｜GET 讀/POST 寫+CSRF/PRG｜`#![forbid(unsafe_code)]`｜clippy 零警告不新增 allow｜UTF-8 無 BOM（PowerShell 用 `[System.IO.File]::WriteAllText`，禁 `>`）｜UNKNOWN≠PASS≠FAIL｜不亂覆蓋（C-01..C-04：斷言式補丁、先量測後動刀）｜Rust-first（C-11）｜四關（fmt/clippy -D warnings/test --locked/audit）EXIT=0｜文件與代碼同一提交｜**量測永遠 fail-open（D-5 原則）**。

## 6. 驗收與驗證（本計畫層級）

1. ~~Grill-me 完成：Product Brief 獲使用者確認~~ ✅ 已達成（D-4，2026-11-10）。
2. ~~設計方向：≥3 案各有 WHY/WHAT/HOW/TRADE-OFFS/EVIDENCE，擇一案獲確認~~ ✅ 已達成（D-3）。
3. 每輪迭代（V1/V2/V3）：適應度函數不回退＋新增可驗證改善＋殘留問題如實記錄。
4. 交付（P12）：Working Product＋測試證據＋Known Limitations；數字皆附量測方法與日期。
5. 實作完成條件：B-1~B-7 全數完成＋M-1 基線與改善後對照已固化＋Q8 視圖之跨用戶隔離 PIN 測試綠。

## 7. 開放決策

- **無未決設計問題**（Q1–Q8 全數定案，見 §4/§8）。
- 執行期預期決策點（非開放問題）：B-3 Design Token 具體色值、B-5 撤銷語義細節——均由 architect 依 D-3 方向核可，屬雙向門。
- T-C 阻塞解除路徑待使用者提供：FlapsErp 原始碼 / 設定介面截圖 / 廠商文件（不阻塞主軸）。

## 8. 決策紀錄（逐題落盤）

### D-1（2026-11-10，Q1 已答）
| 欄位 | 內容 |
|---|---|
| 需求 | ULTIMATE 模式需先鎖定主攻產品線，否則設計與驗收發散 |
| 選擇 | **治理平台 S1 工作台**（使用者確認建議案） |
| 首位使用者 | 財務覆核人員（demo-finance 角色為其代理 persona） |
| 核心任務 | 「30 秒內完成一批待覆核憑單的正確裁決，且每步留稽核軌跡」 |
| 為何 | 唯一「功能完成＋測試全綠（246/0）＋適應度可量測」之線；本 session 原始訴求（應付憑單二階覆核）正落在其批量裁決流程 |
| 影響範圍 | T-A 主軸指向根工作區 `platform/templates/`＋G1.x 既有切片的 V2/V3 精煉；OEKAP/Wellfit 降為 T-B/T-E 軌道 |
| 排除 | 「全組合均衡推進」（碎片化風險）；OEKAP 需先解 os error 4551 才可為主軸 |

### D-2（2026-11-10，Q2 已答）
| 欄位 | 內容 |
|---|---|
| 需求 | 「30 秒」為未量測敘述；V2 動刀前需真實基線（C-03/C-10） |
| 選擇 | **假設目標＋立即量測**（使用者確認建議案） |
| KPI 假設集 | M-1 批量裁決任務時間 ≤30s｜M-2 裁決正確率｜M-3 異常列個案處理時間｜M-4 SUS ≥80｜M-5 稽核查詢 ≤2 步——**五項全標 HYPOTHESIS** |
| 量測方法 | 伺服器端零侵入＋demo-finance 角色任務式測試（機制細節經程式碼查證後修訂於 D-5） |
| 驗證 | 基線實測值出爐後於本計畫 addendum 固化，取代假設；V2 前後對照以同一量測腳本重跑 |
| 排除 | 「純量測先行」（Q3 會懸空）、「沿用適應度」（p95≠任務時間，漏可用性維度） |

### D-3（2026-11-10，Q3 已答）
| 欄位 | 內容 |
|---|---|
| 需求 | 高利害裁決場景需要與 30 秒任務契合的設計語言（master prompt PHASE 4.1 ≥3 案探索） |
| 選擇 | **Human-centered Data Workbench 主方向＋Premium SaaS 元件質感次層**（使用者確認建議案） |
| 為何 | 核心任務重複、高利害、不容錯——第一義是降低認知負擔與誤觸率；零 JS 的 CSS 八態天然契合；states.html 標本頁已是此語境，改造成本最低 |
| 落點 | Design Token 層（`web/assets/design-system.css` 體系）：等寬數字右對齊、差異列高對比、鍵盤優先 focus 設計、密度優先排版；次層僅升級按鈕/卡片質感，不減資訊密度 |
| 排除 | Minimal Editorial（留白拉長掃視路徑）；Premium SaaS 為主（裝飾層擠壓密度） |

### D-4（2026-11-10，Product Brief 確認）
| 欄位 | 內容 |
|---|---|
| 需求 | PHASE 1 退出閘門：Brief 須經使用者確認方可進入工程實作 |
| 選擇 | **確認 §9 Brief＋全部建議預設**（使用者確認） |
| 效果 | 計畫升級 implementation-ready；Q4=LibreOffice headless 轉 PDF｜Q5=範本 Wellfit 先行｜Q6=G3.4 維持僅規格、PRE-R1b 維持可選｜Q7=proptest 僅在已於 Cargo.lock 時採用、CI workflow 併入 B-4 |
| 修訂規則 | Brief 條列如需變更，依 C-01/C-02 就地標 SUPERSEDED 並記於本節，不刪原文 |

### D-5（2026-11-10，程式碼查證後的量測設計修訂）

查證證據：`api.rs:2654-2665`（REVIEW_BULK 逐列 INSERT，`performed_at` 為 DB DEFAULT→單 POST 各列僅差毫秒，**任務時間不可由現有稽核推導**）；`index.html:163-166`（批量表單宿主，已有 csrf_token hidden 欄）；`pages.rs:229`＋`nav.rs:127`（`/audit` 存在但 Manage 閘控，LEDGER_READ 角色不可見）；`api.rs:2605`（BULK_REVIEW_CAP=100）。

| KPI | 量測設計（修訂後） | 狀態 |
|---|---|---|
| M-1 批量裁決任務時間 | hidden `queue_opened_at`＋`REVIEW_BULK_SUMMARY` 事件（設計見 B-1）；語義＝隊列頁渲染→提交送達之伺服器時間差 | 機制已定，基線 UNKNOWN 待 B-1 實測 |
| M-2 裁決正確率 | 需 ground truth（二次複核抽樣）；現無雙人複核流 | **UNKNOWN**——解鎖條件：V2 定義抽樣協議；與 KEM G4 ≥500 帳本門檻無關（不虛造關聯） |
| M-3 異常列個案處理時間 | 現狀無「異常補救→重新提交」獨立事件流 | **UNKNOWN**——解鎖條件：V2 增個案處理流方可量測 |
| M-4 SUS | 無真實使用者（demo-finance 為代理 persona） | **延後**——V2 以認知走查＋啟發式評估替代，SUS 留待真實使用者導入 |
| M-5 稽核查詢 ≤2 步 | 權限矛盾經 Q8 裁決：新增自我稽核視圖 | ✅ 機制已定（D-6），基線待 B-1 實測 |

**原則（約束 B-1~B-7）**：量測是觀測面，永遠 fail-open——缺量測欄位、解析失敗、時間戳異常，一律記 UNKNOWN 並放行業務操作；禁止為量測而阻擋裁決。

### D-6（2026-11-10，Q8 已答：M-5 權限矛盾裁決）
| 欄位 | 內容 |
|---|---|
| 需求 | Brief 承諾「每步留稽核軌跡」且首位使用者須能 ≤2 步自查；但 `/audit` 為 Manage 閘控，矛盾經程式碼查證證實 |
| 選擇 | **方案 (a)：新增「我的裁決紀錄」自我稽核視圖**（使用者確認建議案） |
| 為何 | ①核心任務的稽核軌跡要可自證才有信任價值；②scoped self-view 可見資訊等冪於覆核者已看過的 bulk_review 結果頁之持久化版本，未新增他人資訊面，安全擴張極小；③屬雙向門，依上位 §0.5 由 architect 核可＋預寫回滾 |
| 實作條款 | 伺服器端強制 `performed_by = session.user_id` 過濾、**不接受任何查詢參數覆寫**；新路由（建議 `/my-decisions`）登入即可見、不動 Manage 閘；nav 增對應入口；跨用戶隔離 PIN 測試（A 登入不可見 B 之紀錄）為 DoD 必備 |
| 排除 | (b) M-5 僅 Manage 量測（與核心任務敘事脫節）；(c) 開放 finance 存取全量 `/audit`（暴露他人動作、單向門、違最小權限） |

## 9. Product Brief（已於 2026-11-10 確認）

| 項目 | 內容 |
|---|---|
| 產品 | 三大工作室治理平台——S1 財務覆核工作台（V2/V3 體驗精煉） |
| 願景 | 讓覆核人員以最低認知負擔完成高利害裁決，每步可稽核、可信任 |
| 首位使用者 | 財務覆核人員（demo-finance 為代理 persona） |
| 核心任務 | 30 秒內完成一批待覆核憑單的正確裁決，且每步留稽核軌跡 |
| 範圍 In | 待覆核隊列頁、批量裁決流程（多選/鍵盤/進度/確認）、差異 highlight 掃視性、稽核查詢 ≤2 步（＝D-6 自我稽核視圖）、八資料態文案與下一步連結精修、角色首屏 |
| 範圍 Out | import/drafts 大改（G1.1 已覆蓋，僅一致性檢查）；platform-hub S2/S3（G2.x/G3.x 已完成，僅零 JS 一致性檢查）；OEKAP/Wellfit 功能開發 |
| KPI | M-1~M-5（假設集見 D-2；量測機制與誠實處置見 D-5；M-5 機制見 D-6） |
| 設計方向 | Human-centered Data Workbench＋Premium SaaS 次層（見 D-3） |
| 硬約束 | §5 全部適用（Zero-JS、金額禁浮點、四關、C-01..C-12、量測 fail-open） |

### 執行任務序列（每步先過 §5 約束；每項先立 RED）

1. **B-1 量測基線**（機制見 D-5）：
   - M-1：`index.html` 批量表單（L163）新增 hidden 欄 `queue_opened_at`（伺服器渲染時間戳，零 JS）；`reviews_bulk` handler 計算 `elapsed_ms`；每 POST 寫一列 `REVIEW_BULK_SUMMARY`（event_data JSON：`batch_size`/`ok_count`/`fail_count`/`elapsed_ms`）；不改逐列事件、不改 schema；**fail-open**。
   - M-5 基線：量測現狀「覆核者無法自查」（步驟數=∞ 的事實陳述）作為 D-6 視圖的前後對照起點。
   - M-2/M-3/M-4 依 D-5 標 UNKNOWN/延後（註明解鎖條件）。
   - 實測 demo-finance 任務（10 筆批次）；結果固化 addendum（附日期與方法）。
2. **B-2 研究（PHASE 2）**：官方來源調查資料密集工作台模式（NN/g 表單與批次操作、WCAG 2.2 AA 焦點/對比、設計 Token 實務）；結論附來源與日期，禁捏造。
3. **B-3 Design System（PHASE 4）**：盤點 `web/assets/design-system.css` 現有 Token→依 D-3 補齊色階/字級/間距/密度/對比（AA 驗證）→淺深色一致性；具體色值由 architect 核可（雙向門）。
4. **B-4 V1 地基確認（PHASE 10 V1）**：246 測試＋適應度全綠即 V1 基線，不重作；補八態×四頁矩陣 PIN 缺口（若 B-1 發現）；Q7 併入：proptest 僅當已在 Cargo.lock；CI workflow 一併執行並記錄證據。
5. **B-5 V2 體驗精煉**（每項先立 RED）：
   - **B-5.0 自我稽核視圖**（D-6）：新路由 `/my-decisions`＋伺服器端 `performed_by=self` 強制過濾（拒絕任何參數覆寫）＋nav 入口＋跨用戶隔離 PIN＋八態渲染；預寫回滾（還原即刪路由與模板，無 schema 變更）。
   - **B-5.1 批量裁決流程**：鍵盤優先（全選/反選/提交快捷）、進度回饋（部分失敗逐列語義已備，強化掃視）、誤觸確認與撤銷語義（撤銷＝新的反向 REVIEW_BULK 事件，不刪原紀錄）。
   - **B-5.2 差異掃視性**：金額等寬右對齊、異常列 badge 對比、`data-bulk-review` 表單分區層級。
   - **B-5.3 八態文案**：states.html 對照區＋四頁矩陣文案與「下一步連結」精修。
6. **B-6 V3 頂級品質**：元件質感次層（SaaS borrow）、AA 對比全檢、`prefers-reduced-motion`、跨視窗尺寸檢查。
7. **B-7 每輪驗證**：四關 EXIT=0＋適應度不回退＋M-1/M-5 重測對照（M-2/M-3/M-4 維持標記）＋Decision Log 更新（PHASE 11 格式：需求/選擇/原因/替代/優劣/證據/驗證）。
8. **B-8 並行軌道**（於 B-1~B-7 等待點執行，不阻塞主軸）：Q4 winget 安裝 LibreOffice→`project-document.docx` 轉 PDF；Q5 產出 Wellfit 專案文件（引用其既有 traceability/DEMO 事實），SyncMesh 續後。

## 10. 風險登記（執行期）

| 風險 | 觸發點 | 因應 |
|---|---|---|
| hidden 時間戳被竄改污染 M-1 | B-1 | 量測欄位僅供觀測、fail-open；異常值（負數/超 24h）記 UNKNOWN 不入統計；不承擔安全職能 |
| self-view 過濾被參數繞過 | B-5.0 | 伺服器端強制過濾＋跨用戶隔離 PIN 為 DoD 必備；滲透式測試（`?user_id=` 覆寫嘗試須無效） |
| 並行線同時編輯 index.html/api.rs | 全程 | C-05：動手前重測行數/mtime 對帳；斷言式補丁（C-04）；衝突以上位計畫為準 |
| LibreOffice 安裝受阻（應用程式控制原則） | B-8 | 退路：維持 DOCX＋同步發布 HTML 版；不虛稱 PDF 已產 |
| 測試執行環境 os error 4551 | B-4/B-7 | 根工作區實測可跑（246 passed 為證）；若惡化改以 CI（Linux）跑並如實標記 |

## 11. Known Limitations（交付時必須如實聲明）

1. M-2/M-3 無基線（解鎖條件見 D-5）；M-4 SUS 延後至真實使用者。
2. demo-finance 為代理 persona，非真實使用者訪談結果。
3. T-C（FlapsErp 對接）阻塞中，簽核矩陣實際欄位名待廠商資料。
4. 「30 秒」為假設目標，B-1 基線出爐前不得宣稱達標。

## 12. 實作紀錄 addendum（2026-11-10，B-1＋B-5.0 完成）

### 已落地（B-1 量測機制＋B-5.0 自我稽核視圖）

| 項目 | 檔案 | 內容 |
|---|---|---|
| M-1 測點 | `platform/templates/index.html` | 批量表單新增 hidden `queue_opened_at`（伺服器渲染 epoch 毫秒） |
| M-1 測點 | `platform/src/api.rs` | `IndexTemplate`/`build_index` 渲染時間戳；`BulkReviewForm.queue_opened_at: Option<String>`；`reviews_bulk` 計算 elapsed_ms（負數/>24h→unknown）並寫入每批次一列 `REVIEW_BULK_SUMMARY`（transaction_id=NULL、actor 簽名、event_data 含 batch_size/ok_count/fail_count/elapsed_ms；**fail-open**） |
| 顯示映射 | `platform/src/api.rs` | `audit_event_display` 新增「批量覆核裁決」「批量覆核摘要」中文標籤 |
| 控制映射 | `platform/src/controls.rs` | CTRL-REVIEW event_types 加 `REVIEW_BULK_SUMMARY`（防 UNMAPPED_EVENT lint） |
| D-6 視圖 | `platform/src/pages.rs`＋`templates/my_decisions.html` | `GET /my-decisions`：伺服器端強制 `performed_by=session.user_id`；查詢字串僅分頁，無任何範圍參數；八態渲染＋≤2 步自查 |
| D-6 註冊 | `crates/console_manifest/src/lib.rs`＋`platform/src/api.rs` | `my.decisions` PageSpec（overview 群、Gate::All）＋路由；manifest dynamic 計數測試 42→43（修訂註記依 C-01） |
| 跨線修復 | `platform/tests/guards.rs:201` | clippy `manual_pattern_char_comparison`（並行線新增測試觸發；機械式等義修復） |

### 驗證證據（2026-11-10）

| 關 | 結果 | 時點／證據 |
|---|---|---|
| fmt --check | EXIT=0 | 20:2x 最新樹 |
| test --locked | **EXIT=0，287 passed / 0 failed** | 20:24 樹；四新測試具名通過：`bulk_review_summary_event_carries_batch_and_elapsed`、`bulk_review_summary_fails_open_without_timestamp`、`my_decisions_is_self_scoped_and_ignores_scope_params`、`my_decisions_nav_link_renders_for_all_authenticated` |
| audit | EXIT=0（3 個已接受警告，RUSTSEC 前案） | 20:2x |
| clippy 全 workspace | EXIT=0 @ ~20:05（我的完整樹）；**現被並行線 kem 進行中編輯阻擋**（kem/console.rs 20:31 仍在寫，E0061/E0308/E0425——皆非本計畫觸碰檔） | 依 C-05/C-10：不與進行中重構競賽；並行線收斂後重跑即證 |

### 待辦（下一步）

1. ~~B-1 尾段：M-1/M-5 基線實測~~ ✅ 完成（見 §13）。
2. ~~並行線（模組化）收斂後：重跑全 workspace clippy~~ ✅ 完成（2026-11-11 09:44 全四關 EXIT=0）。
3. B-2~B-3（研究＋Design Token）、B-5.1~B-5.3（V2 精煉）依 §9 序列接續。

## 13. 基線實測固化（2026-11-11，B-1 尾段完成）

| KPI | 實測值 | 方法與日期 |
|---|---|---|
| M-1 機器下限 | **11ms**（batch=10、ok=10） | 整合測試 `api::tests::m1_baseline_machine_floor_ten_row_batch`（--nocapture）；經真實路由 GET /（渲染 queue_opened_at）→POST /reviews/bulk→audit_log 提取；2026-11-11 09:4x |
| M-1 人類基線 | 待真人實測 | 指引：以 demo-finance 帳號登入→開啟工作台（隊列渲染＝t0）→擇 10 筆→送出；事後查詢 `SELECT event_data FROM audit_log WHERE event_type='REVIEW_BULK_SUMMARY' ORDER BY performed_at DESC LIMIT 1`；數值出爐後謄入本節取代「待實測」 |
| M-5 步數 | 修正前：**無路徑**（/audit 為 Manage 閘，覆核者不可達）；修正後：**1 步**（nav 連結即達，PIN `my_decisions_nav_link_renders_for_all_authenticated` 證明） | 2026-11-11 |
| 四關（最新樹，含並行線 EML 變更） | fmt=0｜clippy=0（全 workspace）｜test=0（**295 passed**，較基線 246＋49）｜audit=0 | 2026-11-11 09:44-09:47 |

### 誠實發現（需產品裁量，超出 B-1 範圍）

**待覆核佇列目前無生產上游**：全生產程式碼路徑（Web 表單 `validate_and_post` L915、匯入確認 `import_confirm` L3923）皆以 `status="POSTED"` 建立交易；`PENDING_REVIEW` 僅存在於批量覆核的讀取/更新與測試種子。G1.4 的裁決機制已完備，但其上游「匯入→待覆核」管線尚缺（進入佇列的生產者）。影響：M-1/M-2 的真實人類基線需先有此管線（或以 DB 種子運維流程替代）。建議列為後續切片（PHASE-G 上游補全或產品裁量：Web 表單交易是否應進待覆核佇列）。

### 並行線協作紀錄（C-05）

| 時點 | 事件 | 處置 |
|---|---|---|
| 11-10 18:47 | 並行線改 guards.rs 觸發 clippy lint | 機械式等義修復（split 陣列型樣） |
| 11-10 20:03-20:33 | 並行線模組化（lib.rs 重寫、ingest.rs 新增、PageFamily.path_template、kem 迭代） | 等待靜默；我方樹先行驗證（287 passed），不競賽 |
| 11-11 02:31-09:40 | 並行線 api.rs EML 匯入（ImportFormat::Eml） | 對帳：我方 8 處 REVIEW_BULK_SUMMARY 引用完好；靜默後編譯重試成功 |

## 14. B-2 研究紀錄（2026-11-11，PHASE 2 定點研究）＋B-5.1 落地

### 研究結論（附來源）

| # | 結論 | 來源 | 對本系統的對照 |
|---|---|---|---|
| R-1 | 批次操作三原則：Select All、情境動作列、明確回饋＋**撤銷選項** | NN/g《Bulk Actions: 3 Design Guidelines》2025-03-17 | 我方批量表單已具選取+動作+逐列回饋；撤銷未實作（見 R-4） |
| R-2 | 確認對話「必須具名動作與精確筆數，否則是裝飾」；確認疲勞使確認失效，**可逆動作以撤銷優於確認**（Nielsen 注意力論證） | Phoenix Studio 2026-09-19（引 NN/g）；137Foundry 2026-09-10 | 我方 B-5.1 二段式確認具名動作與方向，**但無筆數顯示**——零 JS 限制：textarea 行數無法前端計數。零 JS 等價解＝「預覽後確認」二次 POST（伺服器渲染筆數），列為 B-5.1 後續選項 |
| R-3 | **部分失敗是常態**：結果須以請求同項回報、具名未竟列並保留供重試 | vocab.design bulk actions 模式 | 我方 BulkReviewOutcome 逐列 ok/rule＋失敗列 highlight **已符合**；「保留選取供重試」受零 JS 限制（重試=複製失敗 ID 回表單） |
| R-4 | 撤銷需**真實反轉契約**：須處理他人後續變更、權限變化、不可逆列 | Mockitt 2026-10-05 | 撤銷＝POSTED→PENDING_REVIEW 狀態轉移，觸及 ledger 引擎狀態機與審計語義——**單向門，需 ADR＋擁有者裁量**（列入開放決策 Q9） |
| R-5 | 稽核軌跡是批次功能的一部分，非事後補丁 | 137Foundry 2026-09-10 | 我方逐列 REVIEW_BULK＋批次 REVIEW_BULK_SUMMARY＋D-6 自我稽核視圖已符合 |

### B-5.1 已落地（2026-11-11）

| 項目 | 內容 | 證據 |
|---|---|---|
| 二段式確認 | 批量送出鈕收合於 `<details class="confirm-submit" data-bulk-confirm>`；展開顯示 confirm-hint（具名「不可改寫、撤銷須另發反向裁決」）與真實送出鈕 | `platform/templates/index.html`；PIN `bulk_review_requires_two_step_confirmation` 綠 |
| 零 JS 進度回饋 | `form[data-bulk-review]:has(#bulk-ids:not(:placeholder-shown)) .bulk-hint` → 顯示「清單已填入」提示 | `web/assets/design-system.css`（token：--color-warning 等，全用既有 Token 無新硬編碼色） |
| 鍵盤可達 | `<details>/<summary>` 原生鍵盤互動＋`:focus-visible` 描邊（沿用 nav-menu 既有先例） | design-system.css 元件區 |

### 開放決策（新增）

- **Q9（單向門）**：是否實作「撤銷裁決」（POSTED→PENDING_REVIEW 反向轉移＋真實反轉契約：逐列檢查他人後續變更、部分不可逆、全程審計）。依 R-4 需 ADR＋產品擁有者裁量；未裁前，B-5.1 的確認文案已誠實聲明「撤銷須另發反向裁決」。
