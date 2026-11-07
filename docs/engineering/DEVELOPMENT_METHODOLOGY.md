# DEVELOPMENT METHODOLOGY — TDD/BDD/SDD/DDD/ADD 執行體系

> 編號：ENG-METHOD-1.0｜日期：2026-11-06｜狀態：Active
> 原則：TDD 紅綠循環為基線；BDD 場景為需求與測試的橋；SDD 以規格為 SSOT；
> DDD 保護領域模型與泛用語言；ADD 以品質屬性驅動架構戰術。
> 證據規則：每項主張綁「測試名稱＋檔案＋行號」或「指令＋退出碼」；UNKNOWN 不等於 PASS。

---

## 1. TDD — 紅綠循環（基線，強制）

循環：RED（先寫失敗測試）→ GREEN（最小實作）→ REFACTOR（fmt/clippy 清潔）。
本輪實例（可複製的示範）：

| 階段 | 證據 |
|---|---|
| RED | `pending_review_kpi_shows_real_count_instead_of_placeholder` 先寫並執行 → FAILED（api.rs:3276 斷言「—」未消失） |
| GREEN | IndexTemplate 加 pending_review_count＋COUNT 查詢＋模板渲染 → 54 passed / 0 failed |
| REFACTOR | cargo fmt exit=0、clippy -D warnings exit=0 |

規則：任何新功能未見「先紅後綠」記錄即視同未完成。flaky 測試視同缺陷（P1-4 先例）。

## 2. BDD — 場景驅動（需求↔測試橋樑）

SSOT 場景註冊表：docs/testing/BDD_SCENARIOS.feature（Gherkin）。
每個 Scenario 綁定一個存在且通過的 Rust 測試；**不引入 cucumber crate**
（精選原則：既有 axum oneshot 整合測試已是真實 router 端到端，步驟解耦層屬純開銷）。
已註冊 8 個 Scenario／10 個綁定測試（登入限流、登帳端到端、分頁×3、待覆核 KPI、
健康降級、RBAC 403、會話撤銷、CSRF 護欄）。
新增旅程時：先在 feature 檔寫 Scenario → 再走 TDD 循環 → 更新綁定測試名。

## 3. SDD — 規格驅動（規格為 SSOT，先於程式碼）

追溯鏈（traceability_chain）：**需求 ID → 規格章節 → 契約/ADR → 測試 → 證據**。
註冊表（現況，全部已驗證）：

| 需求 ID | 規格來源 | 實作 | 測試 | 狀態 |
|---|---|---|---|---|
| FR-001/002 零 JS | PRD；憲章 §2.3 | templates/ 全集 | grep 護欄（script=0） | PASS |
| FR-004 錯誤三段式 | PRD | api.rs 錯誤路徑文案 | 整合測試（400/403 文案） | PASS |
| FR-005 分頁 | REDESIGN_BLUEPRINT M1 | engine paged＋api | pagination_*×3＋parse_page×7 | PASS |
| FR-006 KPI 真實計數 | REDESIGN_BLUEPRINT M3 | build_index COUNT | pending_review_kpi_* | PASS |
| INV-01 零浮點貨幣 | ADR-0001 | flaps_core::money i64 分 | money 6 案＋roundtrip | PASS |
| INV-02 借貸平衡 | ADR-0003；P1-5 | engine validate_journal_entry | engine 負向 8 案 | PASS |
| R-07 Secure cookie | THREAT_MODEL | auth.rs cookie_secure flag | 組態驅動（本機 HTTP 預設關） | PASS |
| T-08 CSRF | THREAT_MODEL | 同步器權杖全 POST | 護欄 10/10 | PASS |
| RPT-001 報表預算 | MVP_ACCEPTANCE | SQL 聚合 | monthly_report_stays_under_500ms | PASS |
| OP-002 健康探測 | NEXT_ACTIONS #2 | /api/health 強化 | health_reports_*×2 | PASS |

維護規則：新增需求先入表（規格未寫不寫碼）；測試名變更時同步本表與 BDD 檔。

## 4. DDD — 領域驅動（聚合、界限、泛用語言）

### 4.1 界限上下文圖

```
┌─────────────── flaps_core（領域層，零 HTTP/零 IO 依賴方向）───────────────┐
│ Ledger 上下文：Transaction 聚合（根）＋TransactionItem（實體）＋Money 值物件 │
│   聚合不變量：ΣDEBIT=ΣCREDIT 且 >0（i128 累加）；金額整數分；日期非未來      │
│ Permission 上下文：Role/Permission 值物件（RBAC 決策）                     │
├─────────────── platform（應用/表現層）────────────────────────────────────┤
│ Identity 上下文：User/Session/CSRF（auth.rs；Argon2id）                    │
│ Import 上下文：Batch/Draft（CSV 暫存→確認）                                │
│ Reporting 上下文：Monthly/Pivot/Parallel 只讀投影（SQL 聚合）               │
├─────────────── crates/flaps_bridge（基礎設施層）──────────────────────────┤
│ COM 上下文：FlapsErp 橋接（批次/池）                                      │
└──────────────────────────────────────────────────────────────────────────┘
```

### 4.2 泛用語言（ubiquitous language）

| 詞彙 | 定義 | 誤用警示 |
|---|---|---|
| 分（minor unit） | 金額以 i64 整數分儲存 | 不得稱「元」用於儲存欄位 |
| 借方合計 | 清單金額＝DEBIT 腿總和（P1-5 語義） | 非「總金額」 |
| 權杖（token） | CSRF 同步器權杖 | 非 session cookie id |
| 待覆核 | status=PENDING_REVIEW | 與「已登帳 POSTED」互斥 |

### 4.3 聚合不變量測試審計

| 不變量 | 測試 |
|---|---|
| ΣDEBIT=ΣCREDIT 且 >0 | engine：unbalanced_entry_is_rejected、one_sided_entry_is_rejected、zero_amount_item_is_rejected、empty_items_are_rejected |
| 狀態機轉移（POSTED↔PENDING_REVIEW） | engine：status_transition_posted_to_pending_review |
| 非法日期/金額/匯率/型別 | engine：future_date_is_rejected、negative_amount_is_rejected、amount_over_cap_is_rejected、nonpositive_exchange_rate_is_rejected、invalid_item_type_is_rejected |
| 同科目借貸拒絕 | same-account 400 整合測試 |
| 會話撤銷 | logout_revokes/deactivated_user_cannot_login |

狀態機轉移（POSTED↔PENDING_REVIEW）：engine.rs 內 `status_transition_posted_to_pending_review`（TDD 先紅後綠，與全工作品牌 98 測試同步全綠）。

## 5. ADD — 屬性驅動設計（品質屬性場景→戰術→證據）

| 屬性場景 | 戰術 | 證據 |
|---|---|---|
| 安全：未認證者不可寫 | RBAC middleware＋session | require_session＋403 測試 |
| 安全：跨站偽造 | CSRF 同步器權杖 | 護欄 10/10 |
| 安全：注入 | bind 參數＋Askama 自動轉義 | 分頁 bind＋T-07 |
| 可用性：DB 斷線可探測 | 503 degraded 探測端點 | health_reports_degraded |
| 可用性：AI 退化不阻斷 | 規則引擎降級閘 | decision_gating 測試 |
| 效能：報表 < 500ms@2k | SQL 聚合＋索引 | close_out 效能測試 |
| 效能：清單常態延遲 | 分頁 LIMIT/OFFSET | pagination 測試（20 筆/頁） |
| 可維護：單體風險 | 模組化拆分地圖 | 藍圖 R1（進行中，併行） |
| 可測：旅程可重放 | BDD 場景註冊表 | 本文件 §2 |
| 耐用：資料可回復 | migrations 可重放＋備份程序 | 遷移測試×3；演練列 NEXT_ACTIONS |
| 可觀測性：單節點 SQLite，無 SLO 目標 | 以 /api/health degraded 狀態為代理監控；部署需擁有者授權 | health_reports_degraded |

## 6. 編開發編測試的檢查清單（每個功能切片必答）

1. 規格先寫了嗎？（SDD 表入列）
2. 場景先寫了嗎？（BDD feature 檔）
3. 測試先紅後綠了嗎？（TDD 證據）
4. 領域語彙一致嗎？不變量有測試嗎？（DDD）
5. 觸碰了哪些屬性戰術？證據更新了嗎？（ADD）
6. 四關（fmt/clippy/test/audit）全綠了嗎？

## 7. 決策登記

| 決策 | 內容 | 狀態 |
|---|---|---|
| PBT 引入 | proptest 1.12.0 已加入 core dev-dependencies；8 個性質測試（parse/format round-trip i64+i128、非負、overflow、TooManyDecimalPlaces、empty、negative）全部通過；Cargo.lock 已更新 | 決定：採用（已執行） |
| TDD 實證邊界(2025-11-06) | 2025 tertiary SLR：8 份 SLR 僅 3% primary studies 被全部納入，重疊 36–54%，結論受選文標準影響；不可把文獻結論直接合併。經典 SR：內部品質 +76%、外部品質 +88% 研究正向，生產力學界正/產業負。家族實驗：TDD 對新手品質反低於 test-last。⇒ TDD 為「條件式工具」：以品質/迴歸/可重演為目標，不可宣稱全面提升生產力；需用本工作區 fmt/clippy/test/audit+證據鏈自我度量 | 已納入基線 |
| 跨領域 TDD 延伸 | Firmware：host mock/HAL/QEMU + ISS 週期精確 + fault injection(TDD) 支援 ISO 26262；FPGA：RC-Unity 跨抽象層同套 unit test；SoC：unit-level fault injection + 靜態 bit-level 分析加速數個數量級。⇒ 本工作區應補：fault injection、mutation testing、HIL/ISS 脫鉤介面 | 缺口（見 §8） |
| PBT+M/BDD 增益證據 | Python 426 corpus(N=40)：PBT 每測 kill 約 unit test 50 倍，76% mutants 20 inputs 內命中；TDD+M 實驗：mutation coverage 64.9% vs 37.5%；BDD+STPA：溝通效益提升，fault detection 無顯著差異。⇒ 本工作區：PBT 已啟用但未擴至 engine/API；mutation 未導入；BDD 未接 STPA/FMEA | 缺口（見 §8） |
| CI/CD 執行 | 工作流檔案 .github/workflows/flexible-ci.yml 已提交於本機 (fd3199f)；推送至 origin 需 `workflow` scope（OAuth token 限制），gh auth refresh -s workflow 需瀏覽器互動；本機已模擬 CI 四關全綠（fmt/clippy/test/audit EXIT=0） | 阻塞：需 workflow scope |
| SRE/SLO | 單節點 SQLite + 本機部署，不訂 SLO；以 /api/health degraded 狀態為代理監控；部署需擁有者授權（Phase 2 閘門 NO-GO） | 決定：不設 SLO |
| TDD 核心主張修正 | 本工作區將 TDD 定位為「測試先行 + 快速回饋迴圈」教學/規範工具，不作生產力宣稱；效益以 mutation score / PBT kill rate / 迴歸逮捕率量測 | 規範修正 |

## 8. 先進技術缺口(對照 32 項總表)

| # | 技術 | 本工作區現況 | 缺口/行動 |
|---|---|---|---|
| 15 | PBT | proptest 僅在 money | 擴至 ledger engine 不變量、parallel 歸一、pivot/pagination |
| 18 | MT mutation | **已導入** cargo-mutants 27.1.0；money 73: 69 caught / 4 missed = **94.5 %**；engine 22 (excluded 5 stubs): 21 caught / 0 missed after fix = **95.5 %**；TOTAL 94.7 % ≥80 % PASS。剩 4 money missed (Display×1 + 3 format/parse) 追蹤於 FAULT-02 | 已完成(剩 P1) |
| 19 | MPT 變形測試 | 未導入 | 報表金額守恆、欄序不變 relationships |
| 17 | FBT 模糊 | 未導入 | cargo fuzz for CSV parser/money parser(限高價值路徑) |
| 6 | ATDD | BDD Scenario 已註冊 8 個，但隨 code 演進不必然同步 | feature→測試綁定自動稽核(CI job) |
| 20/21 | FM 形式化/DbC | 未導入 | 優先 engine 借貸平衡，以 proptest+模糊+smt(BITBY) 渐升 |
| 24 | FMEA/FTA + STPA | THREAT_MODEL 已有表，但未接 BDD/STPA | FMEA→BDD 綁定 |
| 23 | DVT 設計驗證 | 以 BDD+整合測試替代 | 明示為「近似」 |
| 25–32 | CI/CD, DevSecOps, SRE, Chaos | CI workflow 本機 ready、push blocked；SRE 不設 SLO | 先解 workflow scope；Chaos 導入 fault injection |

> 【執行紀錄 2026-11-06】cargo-mutants 已安裝並跑完 money(73) / engine(33) 兩輪，產物 `mutants.out/{caught.txt,missed.txt,unviable.txt,outcomes.json}`。
> Money 13 missed 幾皆為 Display fmt 文字 mutants（已有 grep guard）、i128 `<→>` 邊界 — 需補 i128 邊界測才可達 100%；Engine 剩 1 真 missed 為 `amount > MAX_AMOUNT_MINOR` 的 `>→>=` 等值邊界（缺 amount==MAX 通過案），其餘 missed 為 pragma/query stub 域外變異，已列入 `.cargo/mutants.toml` exclude。
> Fault Injection SOP 已落地至 `docs/engineering/FAULT_INJECTION_SOP.md` (§10 追蹤表 FAULT-01~06)；TDD 定位不變。

> 【執行紀錄 2026-11-07】修復 `.cargo/mutants.toml` (schema: examine_globs / exclude_re / additional_cargo_test_args)，補 `amount_at_cap_posts_successfully` (engine.rs) + `parse_i128_overflow_boundary_kills_boundary_mutants` (money.rs)，重跑 money 69/73=94.5 %、engine 21/22=95.5 %，TOTAL 94.7 % ≥80 % PASS。報告見 `docs/engineering/MUTATION_SCORE_REPORT.md`。TDD 僅作回饋迴圈，不作生產力宣稱。

## 9. Skills 使用修補(你指出的問題)

| Skill | 最近一個月使用率 | 行動 |
|---|---|---|
| kilo-config | 已使用 | 續用 |
| sequentialthinking | 未使用（僅盤點） | 複雜決策/多約束場景必載入並留痕跡 |
| firecrawl/context7 | 未呼叫 | 每次方法論變更前必須呼叫查證，不得僅依記憶 |
| playwright | UI 場景有透用 | 續用(零JS驗證證據) |
| rust-analyzer | 未呼叫 | 改 record why；clippy/test 不取代 IDE 診斷 |
| skill-creator | 未使用 | 專案專屬 TDD skill：紅綠證據檢查表+PBT/mutation SOP 候選 |

所有 future sessions：先讀 SKILL_INVENTORY.md；未載入技能不得假裝已執行；新 skill 需求 → skill-creator 立項。
