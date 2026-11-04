# BDD 驗收場景 — Flaps ERP 工作區

> 方法論：BDD 以 **Feature/Scenario（Gherkin 風格）** 文件化驗收，每個 Scenario 對映一個**可執行整合測試**（不引入 cucumber 依賴——測試名即場景名，單一事實來源為測試本身）。規格 ID 對映見 `docs/testing/TRACEABILITY_MATRIX.md`。

## Feature: 多維樞紐分析（S10-FR-1，PIV-001/002）

```gherkin
Scenario: 已知種子資料的樞紐矩陣與手算一致
  Given 已登入的管理員與 2 帳戶 × 2 月的已知交易（ACC-A 借方 10 月 10.00、11 月 20.00）
  When 請求 GET /reports/pivot?rows=account&cols=month&measure=debit
  Then 回應 200，矩陣含 >10.00<、>20.00<，ACC-A 列合計 >30.00<

Scenario: 非白名單維度被規則名稱拒絕
  Given 已登入的管理員
  When 請求 GET /reports/pivot?rows=account&cols=hacked&measure=debit
  Then 回應 400 且含 R-PIV-WHITELIST
```

**測試**：`pivot_matrix_matches_hand_computed_values`、`pivot_rejects_unknown_dimensions_with_rule_name`

## Feature: 平行座標多維投影（S10-FR-2，PAR-001/002）

```gherkin
Scenario: 每科目一條折線且可無障礙讀取
  Given 已登入的管理員與一筆 ACC-A 交易
  When 請求 GET /reports/parallel?months=2
  Then 回應 200，含 <polyline> 與 <title>ACC-A</title>，且 <script> 計數為 0
```

**測試**：`parallel_projection_renders_one_line_per_account_zero_js`、core：`each_account_gets_one_polyline_with_accessible_title` 等 5 條

## Feature: AI 建議草稿（S10-FR-3，AIS-001..003）

```gherkin
Scenario: 特性旗標預設關閉，關閉時 503 並說明啟用方式
  Given ai/suggest_enabled 旗標未設定
  When POST /drafts/ai_suggest
  Then 回應 503 且頁面含「AI 建議」啟用指引

Scenario: AI 建議永不自動落地
  Given 管理員採用一筆 AI 建議（POST /drafts/adopt）
  When 檢查資料庫
  Then drafts 增加 1 筆且 transactions 仍為 0

Scenario: 畸形模型輸出被 schema 閘門拒絕
  Given 模型輸出「非 JSON」「日期格式錯誤」「金額非數字」
  When parse_ai_suggestion
  Then 全部 Err 且訊息含規則名稱
```

**測試**：`ai_suggest_disabled_by_default_returns_503`、`ai_suggestion_adopt_never_posts_to_ledger`、`ai_suggestion_schema_validation_rejects_malformed_output`

## Feature: 帳號停用即撤銷會話（P1-9）

```gherkin
Scenario: 停用的操作員不得以舊會話繼續操作
  Given store1 持有有效會話
  When 管理員停用 user-store1
  Then store1 的舊會話立即失效（請求 303 → /login）
```

**測試**：`deactivating_a_user_revokes_their_live_session`

## Feature: 管理員密碼重設（P1-8 殘餘）

```gherkin
Scenario: 重設輪換憑證並撤銷會話且稽核
  Given store1 持有舊密碼與有效會話
  When 管理員重設其密碼
  Then 舊密碼登入失敗、新密碼成功、舊會話撤銷、PASSWORD_RESET 稽核 1 筆
```

**測試**：`admin_password_reset_revokes_sessions_and_rotates_credentials`

## Feature: 匯入管線（Slice 8，IMP-001..003，摘錄）

```gherkin
Scenario: 預檢無帳務副作用
  Given 合法 CSV 3 列
  When POST /import/preview
  Then 回應 303 且 transactions 仍為 0、合格列暫存 3 筆

Scenario: 確認僅登帳合格列且批次稽核
  Given 2 好 1 壞列的草稿批次
  When POST /import/confirm
  Then 2 筆登帳、IMPORT_CONFIRMED 稽核 1 筆

Scenario: 重複確認被機械防護拒絕
  Given 已確認的批次
  When 再次 POST /import/confirm
  Then 回應 409
```

**測試**：`import_preview_reports_and_persists_no_ledger_effects`、`import_confirm_posts_only_valid_rows`、`import_confirm_is_replay_guarded`
