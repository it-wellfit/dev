# ADR-0012: E.Sun BankPay 平台整合邊界與誠實模擬策略

- 狀態：Accepted
- 日期：2026-11-09
- 決策脈絡：計畫 `1793782615513-esun-bankpay-ui-optimization.md`（平台原生整合）、治理 P6（最小複製）/P7（反模擬）、§0.5 重構判定框架
- 實作證據：`platform/src/bankpay/*`、`platform/src/status.rs`、`platform/migrations/0006_bankpay.sql`、`platform/tests/bankpay_api.rs`（11 tests PASS）

## 背景

平台需將 E.Sun 銀行支付整合進主工作台（匯出/匯入/查詢/設定）。權威 V2.8 契約實作位於 `Wellfit/crates/wellfit_bankpay`（8 端點、`rqHeader`/`rqData` 信封、mTLS、CP950 媒體編碼、冪等帳本、ReleaseGate），與平台分屬兩個 Cargo workspace。初步實作（並行流 WIP）出現三類問題：

1. **捏造契約**：平台 client 自創 `/v1/payment/export|import|query` 端點與 `source_system_id/customer_id/payer_account` 欄位，與 V2.8 權威契約語意漂移（HP-04）；預設 `esun_url` 猜測為 `https://esunbank.com.tw/bankpay`（不可驗證來源）。
2. **合成資料冒充真實**：mock 匯入寫死 `matched:5`、mock 查詢寫死金額 `50000`（P7 違規）。
3. **功能未交付**：整個模組掛在非預設 feature gate 之後（`default = []`），CI 的 `cargo test --locked`（預設 features）完全不編譯、不測試、不註冊路由——功能名義存在、實際未上線。

## 決策

### D1：運輸層採「誠實模擬器 + fail-closed 生產閘門」，不自創銀行契約

- 模擬模式（預設）：確定性模擬——`esun_ref = ESUN-MOCK-{batch_id}` 決定性推導（可重放、可測試）；匯入仍**真實讀取並解析** CSV（matched/mismatched 為實際列數）；查詢金額/幣別由本地批次帳（`bankpay_batches`）補入，客戶端不虛構任何財務數值。
- 生產模式：明確回 500 與完整說明（問題＋原因＋處理方式），指向權威實作 `Wellfit/crates/wellfit_bankpay` 與 `wellfit bankpay` CLI。**平台在取得經驗證端點規格與 mTLS 憑證前，不猜測端點或信封**（Wellfit 不變量 #6：不可驗證來源標記，不推測）。
- 契約單一事實來源不變：V2.8 協議層（TLS/重試/編碼/ReleaseGate）留在 Wellfit；平台僅持批次帳、狀態機、RBAC、稽核、UI。

### D2：移除 feature gate，零新增依賴交付

- `bankpay` feature（`dep:time` + `reqwest/multipart`）移除：`time` → 既有 `chrono`；`reqwest::multipart` → 不需要（匯入走伺服器端白名單檔案，見 D4）。平台依賴集零新增。
- 模組改為無條件編譯與註冊路由，納入 CI 預設測試面（`cargo test --locked`），消除「功能名義存在、實際未上線」的交付缺口。
- 理由：feature gate 保留了「預設建置零新增依賴」的原始意圖（達成），但犧牲交付完整性；兩者可同時滿足，故不保留閘門。

### D3：冪等與稽核 fail-closed（金額安全）

- `bankpay_batches.batch_id` 主鍵唯一（遷移 0006）；匯出先 `INSERT … ON CONFLICT DO NOTHING` 再讀回現況；已 `SUBMITTED/COMPLETED` 的批次重放時**不重複呼叫銀行**，直接回傳既有狀態（回應逐位元組一致）。
- 稽核寫入修正原實作的 bind 數錯誤（4 binds vs 3 placeholders——原實作 6 個稽核寫入點中 5 個在執行期必敗且被 `let _ =` 靜默吞掉）：統一 `write_audit` 輔助（3 `?` 對 3 bind、`event_data` 為 `{"request","response"}` 單一 JSON 物件）；稽核失敗 → 500（fail-closed），配合冪等重放，重試安全不會重複支付。

### D4：安全控制

- **CSRF（T-08）**：變更類 POST（export/import/config POST）驗證 `X-CSRF-Token` 標頭與 session token 一致（JSON API 的同步器權杖模式）。
- **API 認證**：新增 `auth::require_session_api`（未登入回 401 JSON，符合計畫驗收「未登入存取 API 返回 401」）；同時修復 `cross_module_api` 在 `route_layer(require_session)` 之後 `merge` 導致的未認證存取漏洞（稽核軌跡/CSV 匯出/COM/AI 端點先前完全無需登入）。
- **路徑白名單（CWE-22）**：匯入檔僅接受 `data/bankpay/inbound/` 內的 `.csv`（canonicalize 後前綴比對，拒絕 `..` 與目錄外絕對路徑）。
- **輸入驗證**：`batch_id`/`esun_ref` 限 1–64 個 `[A-Za-z0-9_-]`；金額必須為正整數分（INV-01）；幣別 3 個大寫字母；`esun_url` 必須 https://。
- RBAC 沿用既有種子權限碼（`LEDGER_WRITE`/`LEDGER_READ`/`ROLE_MANAGE`），不新增權限規則（P6）。

### D5：狀態探測 derive 而非快取

- `/api/status` 與 index 全域狀態列共用 `status::probe_modules`（即時探測，無 RwLock 快取）——狀態永不過期、永不殘留 Unknown（原實作 `overall_status` 恆為 `degraded`）。
- 誠實標示：sync/com/ai 標 `Idle` 並說明未接入原因；bankpay 模擬模式標 `Warning`（模擬資料必須讓使用者看見）。儀表板「AI 可用模型數」由虛構的 `3` 改為真實的 `0`。

## 後果

- 正面：功能實際交付並受 CI 保護；金額操作具冪等與完整稽核；未認證漏洞關閉；平台與 Wellfit 契約邊界清晰（單一事實來源）。
- 取捨：生產模式在此切片為明確 fail-closed（未達「可直接打真銀行」）；啟用真實通道需後續切片（設定憑證 + 契約一致性測試 pin 住 V2.8 欄位）。
- 逆轉成本：模擬器 API 形狀即生產形狀；未來以 adapter 實作替換 `EsunClient` 內部，路由/狀態機/稽核不變。

## 驗證證據（revision-bound）

- `cargo test --locked -p platform`：124 passed / 0 failed（含 `bankpay_api.rs` 11 項：401/403/CSRF/冪等/驗證/路徑白名單/狀態推進/稽核寫入/儀表板渲染）。
- `cargo clippy -p platform --all-targets -- -D warnings`：0 警告（引入小型 `ApiError` 型別替代大型 `Response` Err，`div_ceil` 取代手寫天花板除法）。
- `cargo fmt -p platform -- --check`：通過。
- Live smoke（127.0.0.1:8091）：未登入 401；CSRF 缺漏/錯誤皆 403 且**不產生批次列**；匯出 200 → 冪等重放回應一致；查詢推進 SUBMITTED→COMPLETED 並回報本地帳金額；白名單外路徑 400；匯入實測 `matched:2/mismatched:1`；`/api/audit` 需登入（漏洞關閉證據）。
