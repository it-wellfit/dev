# 知識手冊（A–F 章節規格）

> 版本：v1.0｜日期：2026-11-06｜狀態：草案（供檢視）
> 適用範圍：Dev 根工作區（FlapsErp：core/platform/web/crates/flaps_bridge）＋ Wellfit 子專案（crates/wellfit_bankpay 等）
> 來源事實：僅來自已驗證程式碼、既存 docs/ 文件、ADR 決策記錄；不得臆測

---

## A. 專案總覽（新人先建立全貌）

### A1 產品定位

Flaps ERP（輔翼 ERP）是單機、零 JavaScript 的企業帳務自動化工作台。核心價值：「30 秒完成一筆可稽核、可解釋、平衡的複式分錄」。資料不出機器，零訂閱鎖功能，審計人員可逐行驗證表現層。

### A2 四大核心假設（已驗證）

| ID | 假設 | 本產品的答案 |
|----|------|--------------|
| H1 | 帳務資訊取得成本高 | 儀表板＋月報＋CSV 一次到位 |
| H2 | 流程斷裂（記帳與核對分離） | 借貸平衡強制＋稽核軌跡一體 |
| H3 | 對 AI 信任不足 | AI 僅建議、需人工覆核；每個預填標示來源規則 |
| H4 | 整合維護成本高 | 單一執行檔＋SQLite 檔案式 DB；零 JS 無前端建置鏈 |

### A3 目標使用者與角色

| 角色 | 主要需求 | 對應頁面 |
|------|----------|----------|
| 記帳人員 | 快速登帳、錯誤即時提示 | `/` 交易表單、`/new` 精靈 |
| 財務審核員 | 平衡驗證、稽核軌跡、報表 | `/reports/monthly`、`/reports/pivot` |
| 企業主 | 資料在本機、無訂閱依賴 | `/` 儀表板 |
| IT/稽核 | 零 JS 可審計、WAL 備份簡單 | `/import/preview`、原始碼審查 |

### A4 目錄結構（根工作區）

```
Dev/
├── web/                 # Tauri 2.0 桌面殼 + 靜態零 JS 介面
├── core/                # flaps_core：MDM、SQLite 分類帳、ReleaseGate、ScaledDecimal
├── platform/            # Axum 伺服器：API、Auth、會話、CSRF、匯入、報表
├── crates/flaps_bridge  # FlapsErp RPA 支援線（Windmill + PowerShell）
├── docs/                # 文件（本手冊所在）
├── scripts/             # Windmill 相容整合/測試腳本（PowerShell）
├── config/              # WDAC 原則、環境變數範本
└── .kilo/plans/         # 專案計畫與執行紀錄
```

### A5 不變量（INV-01…INV-08）

| ID | 不變量 | 執行點 |
|----|--------|--------|
| INV-01 | 金額／數量禁用浮點（整數 minor units） | `core/src/money.rs`、`core/src/value_objects.rs` |
| INV-02 | 複式分錄借貸必須平衡 | `core/src/ledger/engine.rs::validate_journal_entry` |
| INV-03 | WAL＋busy_timeout；寫入交易 ≤ 50 ms | `core/src/ledger/engine.rs::apply_pragmas` |
| INV-04 | 同 txSeq 冪等；重複 POST 不重複記帳 | `core/src/ledger/engine.rs::post_journal_entry` |
| INV-05 | `#![forbid(unsafe_code)]` | 每個 `.rs` 第一行 |
| INV-06 | clippy 零警告；fmt 檢查通過 | CI 執行門檻 |
| INV-07 | GET 僅讀取；建立／修改／刪除用 POST；成功後 PRG | `platform/src/api.rs` 路由 |
| INV-08 | 表現層零 JavaScript；`<script>` 數 = 0 | `web/index.html` CSP |

---

## B. 需求與產品行為

### B1 功能需求（FR-001…FR-008）

| ID | 需求 | 驗收條件 | 對應測試 |
|----|------|----------|----------|
| FR-001 | 表現層零 JavaScript | `<script>` 數 = 0；無內聯事件 | `platform/tests/...` |
| FR-002 | 設計系統一致性 | 色彩／間距／字體皆來自 Design Token | UI 審查 |
| FR-003 | 表單 PRG | 成功後 303 Redirect；重新整理不重複送出 | `api::tests::post_then_get...` |
| FR-004 | 完整元件狀態 | Loading／Empty／Error／Default／Disabled | `web/index.html` §states |
| FR-005 | 帳務登帳驗證 | 負數／未來日期／非 DEBIT/CREDIT 拒絕 | `engine` 負向測試 8 條 |
| FR-006 | RBAC 權限檢查 | permission_code 判定；無權限回 403 | `api::tests::post_requires_ledger_write_permission` |
| FR-007 | 交易查詢分頁 | 參數化查詢；無字串拼接 | `engine::query_transactions` |
| FR-008 | AI 決策閘門 | score<0.3 自動通過；0.3–0.7 人工覆核；≥0.7 拒絕 | `platform/src/ai/decision_gating.rs` |

### B2 非功能需求（NFR-001…NFR-005）

| ID | 類別 | 目標 | 測量方式 |
|----|------|------|----------|
| NFR-001 | 安全 | 輸入驗證 fail-closed；SQL 參數化 | 負向測試＋`cargo audit` |
| NFR-002 | 無障礙 | WCAG 2.2 AA：鍵盤導航、可見焦點 | Playwright 掃描 |
| NFR-003 | 效能 | 首次載入 < 1s；登帳 API P95 < 50ms | DevTools；criterion 基準 |
| NFR-004 | 可維護性 | clippy 零警告；fmt 檢查通過 | `cargo clippy --all-targets -- -D warnings` |
| NFR-005 | 反模擬 | 未知值顯示「—」；不以假資料冒充 | UI 審查 |

---

## C. 系統架構與設計理念

### C1 背景與目標

- 背景：中小企業會計登帳從「手動記帳、事後核對」變為「30 秒完成一筆可稽核分錄」。
- 目標：單機安全工作台，證據導向的帳務自動化。
- 重要取捨：單機 SQLite（無 PostgreSQL 依賴）；零 JavaScript（審計友善）；AI 僅建議（人工覆核）。

### C2 系統上下文

```
使用者 ──(零 JS 介面)──> Axum 伺服器 (platform/)
      │                    │
      │                    ▼
      │              SQLite (WAL 模式)
      │                    │
      └─(TLS 1.2, 2-Way SSL)─> 玉山銀行安控元件 (Wellfit/bankpay)
```

### C3 主要元件與依賴方向

```
Presentation (web/, platform/templates)
         ↓ (Axum HTTP)
Application (platform/src/api.rs, auth.rs)
         ↓ (flaps_core ports)
Domain (core/src/ledger/, money/, parser/, permission/)
         ↓ (SQLx ports)
Infrastructure (platform/migrations/, SQLite)
```

依賴方向：Presentation → Application → Domain → Infrastructure（Clean Architecture）。

### C4 各模組責任與邊界

| 模組 | 責任 | 輸入 | 輸出 | 依賴 |
|------|------|------|------|------|
| `web/` | 零 JS 介面、設計系統 | HTTP GET | HTML/CSS/SVG | 無 |
| `platform/src/api.rs` | 表單處理、PRG、CSRF 驗證 | HTTP POST | HTML/Redirect | `flaps_core`、`platform/src/auth` |
| `platform/src/auth.rs` | Argon2id、會話、CSRF、速率鎖定 | 認證請求 | Session、Cookie | SQLite (`users`、`sessions`) |
| `core/src/ledger/engine.rs` | 複式分錄驗證、原子寫入 | Transaction＋Items | PostgreSQL/SQLite 寫入 | SQLite |
| `core/src/money.rs` | 精確小數轉整數 minor units | 字串金額 | `i64` minor units | 無 |
| `core/src/parser.rs` | 非結構化文字解析為草稿 | 自由文字 | `DraftEntry`＋證據卡 | 無 |
| `core/src/parallel.rs` | 平行座標 SVG 投影 | 整數序列 | SVG 字串 | 無 |
| `platform/src/ai/` | AI 決策閘門（建議不執行） | 交易＋風險分 | 通過／覆核／拒絕 | 無外部 AI 服務 |

### C5 核心演算法與商業規則

1. **複式分錄平衡**：借方總額 = 貸方總額，且總額 > 0（`engine.rs::validate_journal_entry`）。
2. **整數金額**：所有金額以 `i64` minor units 儲存；解析使用精確字串運算，無 `f64`（`money.rs`）。
3. **冪等登帳**：相同 `transaction_id` 重複 POST 時，第二次失敗（主鍵衝突），不重複記帳（`engine.rs` + `bankpay.rs::ReleaseGate`）。
4. **AI 決策閘門**：風險分 < 0.3 自動通過；0.3–0.7 人工覆核；≥ 0.7 自動拒絕＋告警；金額修改、權限變更、作廢為 AI 禁區（`decision_gating.rs`）。

### C6 錯誤處理與例外傳播

- **Business envelopes**：`910000` 成功、`910071` 欄位驗證錯誤（`moreInfo` 含逐筆說明）、`999999` 未知錯誤。
- **HTTP 狀態**：429/500–504 可重試（idempotent 僅限）；4xx 不重試。
- **TLS 故障**：client_cert/ca_cert 載入失敗時封鎖請求，不降級（`http.rs`）。
- **錯誤訊息格式**：「問題＋可能原因＋處理方式」（零 JS 介面必須遵循）。

### C7 重試/逾時/取消

- `RetryConfig`：max_retries=3，base_delay_ms=200，backoff_multiplier=2.0，max_delay_ms=2000（本地 security component 優化）。
- 全抖動（full jitter）避免雷群。
- `Retry-After` 標頭被尊重，上限為 `max_delay_ms`。
- 非冪等 POST（create_twd_payment、create_fx_payment_instruction）永不重試（double 風險）。

### C8 新增功能／修改／重構指引

- 新增 API 端點：先在 `api.rs` 新增模型（camelCase serde），再在 `http.rs` 新增 wrapper，最後補測試（先紅後綠）。
- 修改 schema：`IF NOT EXISTS` 冪等起手；破壞性變更採 Expand-and-Contract。
- 不複製檔案內容進文件；文件引用程式碼路徑（`file.rs:line`）。

---

## D. 元件、類別、函式、元件及屬性大全

### D1 核心型別（core/src/value_objects.rs）

| 名稱 | 類型 | 前置條件 | 後置條件 | 不變條件 |
|------|------|----------|----------|----------|
| `Money` | struct | `minor > 0`；currency 為 3 碼 ASCII 大寫 | `minor` 與 `currency` 不變 | 不可變；無浮點 |
| `AccountId` | struct | 非空白；≤ 64 字元；無前後空白 | `as_str()` 回傳修正後字串 | 不可變 |

### D2 頭單模型（core/src/ledger/transaction.rs）

| 名稱 | 類型 | 關鍵欄位 | 建構方式 |
|------|------|----------|----------|
| `Transaction` | struct | transaction_id, txDate, txAmt, txSeq, detailList... | `Transaction::builder()` |
| `TransactionItem` | struct | item_id, account_id, item_type, amount_minor, currency | `TransactionItem::builder()` |

### D3 核心服務（core/src/ledger/engine.rs）

| 名稱 | 方法 | 前置條件 | 後置條件 | 副作用 |
|------|------|----------|----------|--------|
| `LedgerEngine` | `post_journal_entry` | items 不空；借貸平衡；日期不未來 | 交易寫入；audit_log 記錄 | SQLite 寫入 |
| `LedgerEngine` | `query_transactions` | 參數化查詢（無字串拼接） | 回傳交易清單 | 僅讀取 |
| `LedgerEngine` | `validate_journal_entry` | 單一交易＋項目清單 | 錯誤訊息（問題＋原因＋處理） | 無副作用 |

### D4 認證與授權（platform/src/auth.rs）

| 名稱 | 方法 | 前置條件 | 後置條件 | 副作用 |
|------|------|----------|----------|--------|
| `LoginRateLimiter` | `check` | username 不空 | `Allowed` 或 `Locked { remaining_secs }` | 讀取記憶體 |
| `LoginRateLimiter` | `record_failure` | 連續失敗達閾值 | 鎖定帳號 `LOGIN_LOCKOUT_SECS` | 寫入記憶體 |
| `authenticate` | 用戶名＋密碼 | 使用者存在且 is_active=1 | 回傳 `user_id`；否則 `None` | 讀取 SQLite |
| `create_session` | user_id | 使用者已驗證 | 產生 session_id + csrf_token；寫入 `sessions` | 寫入 SQLite |
| `security_headers` | 應答標頭 | 每個 HTTP 回應 | CSP／nosniff／DENY／no-referrer／no-store | 修改回應標頭 |

### D5 前端元件（web/index.html）

| 元件 | 狀態 | WCAG 2.2 AA |
|------|------|-------------|
| 交易表單 | Default／Hover／Focus／Disabled／Loading／Empty／Error | 鍵盤導航；可見焦點 |
| 儀表板 KPI | Default／Loading／Error | 對比檢查；不以顏色為唯一傳達 |
| 會計科目下拉 | Default／Focus／Disabled | 鍵盤可操作 |
| 錯誤訊息 | 問題＋可能原因＋處理方式 | 文字傳達；非僅顏色 |

### D6 注意事項

- 不複製任何檔案內容進本章節；所有型別與函式名稱請直接參照原始碼（`core/src/`、`platform/src/`）。
- 私有變數不獨立寫成章節；僅記錄影響跨模組互動或維護決策的公開項目。

---

## E. UI/UX 需求（精美、智慧、易用、智能化）

### E1 設計原則

- **零 JavaScript**：`<script>` 數 = 0；互動僅用 CSS（`:hover/:focus-visible/:user-invalid`）。
- **Design Token**：樣式一律使用 `web/assets/design-system.css`；禁止內聯硬編碼色彩／間距／字體。
- **分區切換**：`:target`；導航高亮：`:has()`。
- **動畫**：僅 `transform`/`opacity`；支援 `prefers-reduced-motion`。
- **語言**：介面文字預設繁體中文（台灣情境）。

### E2 元件狀態矩陣

每個互動元件必須具備以下狀態：

| 狀態 | 說明 | 視覺表徵 |
|------|------|----------|
| Default | 未互動 | 設計系統預設樣式 |
| Hover | 滑鼠懸停 | `transform`/`opacity` 微動 |
| Focus | 鍵盤聚焦 | 可見焦點環（WCAG） |
| Disabled | 不可操作 | 灰階；`pointer-events: none` |
| Loading | 資料載入中 | 骨架畫面或轉圈（CSS 動畫） |
| Empty | 無資料 | 「—」或空狀態圖示；不以假資料冒充 |
| Error | 操作失敗 | 問題＋原因＋處理方式；可重試 |

### E3 錯誤訊息格式

每個錯誤訊息必須包含：

1. **問題**：什麼出了問題（簡潔）。
2. **可能原因**：為什麼會發生。
3. **處理方式**：使用者該做什麼。

範例：「登入失敗（可能原因：帳號或密碼不正確、帳號已停用）；請重新輸入，或聯絡系統管理員。」

### E4 無障礙（WCAG 2.2 AA）

- 語義 HTML；可見焦點；鍵盤導航。
- 不以顏色為唯一傳達資訊。
- `prefers-reduced-motion` 支援。
- 對比度符合 AA 標準。

### E5 頁面清單

| 頁面 | 路由 | 主要元件 | 狀態 |
|------|------|----------|------|
| 登入 | `/login` | 帳號／密碼／CSRF | Default／Loading／Error |
| 儀表板 | `/` | KPI 卡片／最近交易／科目總覽 | Default／Loading |
| 新增交易 | `/new` | 交易表單（借貸方） | Default／Hover／Focus／Disabled／Loading／Empty／Error |
| 交易清單 | `/transactions` | 分頁表單 | Default／Empty |
| 會計科目 | `/accounts` | 科目樹／啟用／停用 | Default／Empty |
| AI 決策 | `/ai-decisions` | 風險分／覆核按鈕 | Default／Loading／Error |
| 介面狀態 | `/states` | 元件狀態展示 | Default |

---

## F. 專業名詞與領域知識庫

### F1 核心術語

| 標準名稱 | 英文對應 | 定義 | 本專案意義 | 出現位置 |
|----------|----------|------|------------|----------|
| 複式分錄 | Double-entry bookkeeping | 每筆交易必須有借方與貸方，且總額相等 | 核心不變量 INV-02 | `engine.rs` |
| 小數單位 | Minor units | 金額以 10^-dp 整數儲存（dp=2 為分） | INV-01；禁止浮點 | `money.rs` |
| Idempotency | 冪等 | 相同輸入重複執行結果相同 | txSeq 重複 POST 拒絕 | `bankpay.rs` |
| PRG | Post/Redirect/Get | 建立／修改／刪除用 POST；成功後重導向 | 表單提交模式 | `api.rs` |
| 2-Way SSL | Mutual TLS | 客戶端與伺服器互驗證 | E.Sun 安控元件 | `http.rs` |
| Certificate Pinning | 憑證釘固 | 僅信任特定 SHA-256 憑證 | 防止中間人攻擊 | `http.rs` |
| Argon2id | Argon2id | 記憶體硬雜湊演算法 | 密碼雜湊 | `auth.rs` |
| CSRF | Cross-Site Request Forgery | 跨站請求偽造；同步化權杖防護 | 表單 `csrf_token` | `auth.rs` |
| RBAC | Role-Based Access Control | 角色基礎存取控制 | `user_roles` → `LEDGER_WRITE` | `permission.rs` |
| WAL | Write-Ahead Logging | SQLite 預寫日誌模式 | 效能與崩潰恢復 | `engine.rs` |

### F2 常見錯誤代碼

| 代碼 | 名稱 | 意義 | 處理方式 |
|------|------|------|----------|
| 910000 | 成功 | API 正常處理 | 顯示 rsData |
| 910071 | 欄位驗證錯誤 | 欄位不符合規格 | 顯示 moreInfo 逐筆說明 |
| 999999 | 未知錯誤 | 伺服器內部錯誤 | 記錄並提示聯絡管理員 |
| 0000 | 交易成功 | 銀行端交易完成 | 顯示「已完成」 |
| 1000–1008 | 待處理 | 交易中途狀態 | 顯示「覆核中」；輪詢查詢 |
| 4000 | 交易失敗 | 銀行端拒絕 | 顯示失敗原因；建議重試 |
| 9999 | 無資料 | 查詢結果為空 | 顯示「—」；不以假資料冒充 |

### F3 領域知識摘要

- **會計科目類型**：ASSETS（資產）、LIABILITIES（負債）、EQUITY（權益）、REVENUE（收入）、EXPENSE（支出）。
- **借貸方向**：資產／支出類借方增加；負債／權益／收入類貸方增加。
- **交易狀態**：0000 成功；1000–1008 待處理；1030–1032 外匯相關；3000/3001 外匯預檢；4000 失敗；9999 無資料。
- **銀行環境**：DEV/Test `http://IP:8080`；UAT `eospu.testesunbank.com.tw:9443`；Prod `eosp.esunbank.com.tw:9443`；內部安全元件 `:8443`（TLS 1.2+，2-Way SSL）。

---

*本手冊為即時文件；任何程式碼變更必須同步更新對應章節（SHARED_STANDARDS §4.3）。*
