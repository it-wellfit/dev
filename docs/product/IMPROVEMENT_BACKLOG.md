# Improvement Backlog — Dev 根工作區

> 版本：v1.0｜日期：2026-11-03｜狀態：Active
> 優先順序依 Directive Part 14.3：安全 → 資料正確性 → 核心流程 → 高影響 Bug → UX → 產品改善 → 架構效能 → 視覺細節。
> 每項完成時更新 Status 並附驗收證據（指令＋退出碼）。

| Issue ID | Problem | Evidence | User Impact | Business Impact | Priority | Effort | Risk | Proposed Solution | Acceptance Criteria | Status |
|----------|---------|----------|-------------|-----------------|----------|--------|------|-------------------|---------------------|--------|
| P0-1 | ~~金額／匯率使用 `f64`，違反 INV-01 零浮點貨幣不變量~~ **整數化完成** | 修復後：`TransactionItem.amount_minor: i64`（整數分）、`exchange_rate_scaled: i64`（10^4 縮放）；migrations 0001 INTEGER schema；`flaps_core::money` 精確字串解析（拒 >2 位小數／負數／溢位）＋`format_minor` 純整數格式化 | 會計累積誤差消除 | 高：登帳結果可用於真實會計 | **P0（資料正確性）** | M | 已完成 | — | money 邊界 6 測試＋整數 roundtrip 測試＋clippy/test/audit 全綠（2026-11-04 session log） | **Resolved（2026-11-04）** |
| ~~P0-2~~ | ~~根 workspace 非 git repo，證據無法綁修訂版~~ **已初始化** | 修復後：根 repo `main`（提交 cd66388 初始、8a79adf 功能）；.gitignore 排除 6 個嵌套獨立 repo 與計畫目錄、target 7GB、本機 DB；staged 內容安全掃描通過（無真實秘密） | — | 治理證據鏈閉合（P2：修訂版＋指令＋退出碼） | **P0（治理）** | S | 已完成 | — | `git log` 可引用；後續變更皆綁修訂版 | **Resolved（2026-11-04）** |
| P1-1 | ~~後端 API 層未接線~~ **端到端完成** | 修復後：`platform/src/bin/flaps-server.rs`（Axum 0.7）＋`src/api.rs`：GET /（Askama 伺服器渲染）、GET /assets/design-system.css（嵌入）、GET /api/health、POST /transactions（RBAC→驗證→登帳→303 PRG） | 使用者可實際登帳（303→清單渲染真實資料） | 核心流程解鎖 | **P1（核心流程）** | M | 認證層仍屬 G-1（X-Acting-User header 未認證，見 THREAT_MODEL T-01） | 詳 ADR-0002 | curl 煙霧：POST 303→GET 渲染 1,234.50；403/400 負向全過；整合測試 8 條 | **Resolved（2026-11-04）**（殘餘：G-1 認證） |
| P1-2 | sqlx 雙建表路徑（`ensure_tables` 動態建表 vs migrations） | `core/src/ledger/engine.rs::ensure_tables`、`platform/src/db.rs::initialize` | — | schema 漂移風險 | P1 | S | 低 | `platform/migrations/` 為 SSOT（已建＋`initialize()` 已接線 `sqlx::migrate!()`）；`ensure_tables` DDL 已同步整數 schema，`LedgerEngine::with_pool` 為推薦建構（不建表） | 乾淨庫重放遷移成功；兩路徑一致 | In Progress（收斂完成 90%；`ensure_tables` 退役待 P0-2 git 後一併） |
| P1-3 | ~~測試覆蓋不足~~ **已補齊** | 最終：flaps_bridge 7、flaps_core 22（含引擎負向 8：不平衡/單邊/零額/未來日/負數/上限/匯率/非法型別）、platform 37（認證/CSRF/匯入/精靈/報表/效能/Token 同步） | 回歸防護完整 | 品質門檻滿足 | P1 | M | 已完成 | — | 66 tests passed / 0 failed（slice9-final log） | **Resolved（2026-11-04）** |
| P1-5 | ~~單筆分錄不強制借貸平衡~~ **複式記帳落地** | 修復後：engine `validate_journal_entry` 強制 ΣDEBIT = ΣCREDIT 且 >0（i128 累加防溢位）；表單升級借方/貸方雙欄（同科目拒絕）；清單金額改借方合計語義 | 帳目完整性保證 | 高：真實會計可用 | P1 | M | 已完成 | — | engine 負向測試 8 條（不平衡/單邊/零額/未來日/負數/上限/匯率/非法型別）＋same-account 400 整合測試 | **Resolved（2026-11-04）** |
| P1-6 | G-1 認證缺口：`X-Acting-User` 可偽造 | `api.rs` 依 header 判定身分 | 任何人可自稱 admin（RBAC 仍擋權限） | 高：安全邊界 | **P1（安全）** | M | 已完成 | ADR-0003：Argon2id＋sessions 表＋HttpOnly/SameSite cookie＋CSRF 同步器 token＋`require_session` middleware＋首次啟動管理員播種 | 登入/登出/CSRF/RBAC 整合測試 13 條＋煙霧全綠 | **Resolved（2026-11-04）**；殘餘：登入速率限制、Secure cookie（R-07）、帳號管理 UI |
| P1-4 | flaky 效能測試：`batching_beats_per_call_under_measured_envelope` 以實際時鐘比較，差距 < 1ms 即翻盤 | 2026-11-03 三次觀察兩次 FAILED（session tests logs）；根因：時鐘斷言受排程抖動影響且效益未經確定性證明 | CI 不可靠（DORA Test Reliability） | 偽失敗造成門檻噪音 | P1 | S | 低 | 改為確定性往返計數斷言：MockTransport 新增 `round_trips()`；斷言 `round_trips == n + n.div_ceil(256)`，牆鐘僅供觀察輸出 | 連續 3 次全綠＋clippy/fmt 綠 | **Resolved（2026-11-03）**：連續 3 次通過，全 workspace 基線四項 EXIT=0 |
| P1-7 | 登入速率限制 | `LoginRateLimiter`（每帳號 5 次連續失敗 → 鎖定 15 分鐘，成功即重置；鎖定訊息含剩餘秒數；大小寫不敏感） | 爆破面收斂 | 高：認證邊界完整 | P2 | S | 已完成（單機 in-process；多實例需遷移共享存放——記錄於 P1-7 notes） | — | 鎖定/解鎖/重置/跨帳號隔離 4 測試＋正確密碼被鎖整合測試 | **Resolved（2026-11-04）** |
| P1-8 | 帳號管理 UI | `/admin/users`（清單＋建立＋啟停），ROLE_MANAGE 即時 DB 查詢把關，CSRF 全覆蓋，自我停用拒絕 | 營運可自助 | 中 | P2 | M | 已完成 | — | 建立→登入→登帳端到端＋403＋停用拒登入＋自停拒 4 測試 | **Resolved（2026-11-04）** |

| P2-1 | `#[allow(too_many_arguments)]` 兩處 | `core/src/ledger/transaction.rs` | 可讀性 | 維護成本 | P2 | S | 低 | Builder 或參數物件（struct）重構 | allow 移除且 clippy 綠 | **Resolved（2026-11-04，c932753）** |
| P2-2 | 根 `Cargo.toml` 含 tauri 2.0 workspace 依賴但無人使用 | 移除後 Cargo.lock 中 tauri 條目歸零；`src-tauri/` 為獨立 crate 自行宣告依賴不受影響 | 建置時間 | 依賴面縮減 | P2 | S | 已完成 | — | `TAURI_IN_LOCK=0`＋build/test 綠 | **Resolved（2026-11-04）** |
| P3-1 | ~~介面效能預算未量化~~ **報表預算已量化** | close_out 測試：2,000 交易（recursive CTE seed）→ /reports/monthly 斷言 < 500ms | — | RPT-001 吞吐條款可驗收 | P3 | S | 已完成 | — | monthly_report_stays_under_500ms_with_2000_transactions | **Resolved（2026-11-04，首頁 Lighthouse 仍待量測）** |
| P3-2 | `docs/design-system.css` 與 `web/assets/design-system.css` 雙檔同步 | 兩檔 :root Token 需逐字一致 | — | SSOT 漂移風險 | P3 | S | 低 | 短期：變更同一提交雙向同步（頭註已標）；長期：build 時由單一來源產生 | 同步檢查腳本或單一來源 | **Resolved（2026-11-04，同步迴歸測試護欄已入）** |
| P3-3 | session 遺留：`.kilo/plans/debugging-summary.md` 先前規劃輪嘗試佔用未遂 | 該檔現況 | — | 治理整潔 | P3 | S | 低 | 確認檔案內容後歸位 | plans 目錄無遺留狀態 | **Resolved（2026-11-04，確認兩檔並存、主計畫為 SSOT）** |
