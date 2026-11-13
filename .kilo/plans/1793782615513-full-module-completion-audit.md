# 全模組完工計畫 — 17 模組 × 業務流程閉環 × 元件契約

> 日期：2026-11-13｜狀態：Implementation-Ready（待 1 項範圍裁定）｜執行代理：需可編輯檔案之代理
> 基線（實測）：66 manifest 頁｜149 lib tests / 0 failed｜E2E 25/25｜RBAC 264 組 0 越權｜CSV 32→34、XLSX 34→36 範本閉環

## 1. 既有測試 → 不得重複（盤點）

| 測試檔 | 已覆蓋 | 本計畫不重做 |
|---|---|---|
| `page_matrix.rs` | 角色矩陣（訪客/admin/store）、零 `<script>`、路由↔manifest 雙向對帳 | 頁面 200/權限矩陣 |
| `interaction_audit.rs` | 頁內錨點、`<details>` 下拉、`:target` 面板、多步流程 | 互動原語存在性 |
| `design_a11y.rs` | WCAG 2.2 AA 對比（token 數值化） | 色彩對比 |
| `naming_guard.rs`、`guards.rs` | 命名分離、原廠標記禁用 | 命名 |
| `close_out.rs`、`password_reset.rs` | 工作階段撤銷、重設流程 | 登入/登出/重設 |
| `import_{docx,pdf,eml,xlsx}.rs`、`mapping_{csv,flow}.rs` | 各格式解析器、映射流程 | 解析器單元測試 |
| `bankpay_api.rs`、`bankpay_persistence.rs`、`money_props.rs`、`ledger_slice12.rs` | 銀行支付契約/持久化、金額 proptest、總帳 | 該等單元 |
| `trace_guard.rs`、`experience_traceability.rs`、`docs_governance.rs` | 文件↔程式碼、TT 契約、文件治理 | 文件校驗 |
| `backup_tool.rs`、`sbom_freshness.rs`、`secret_scan.rs`、`web_crawl.rs` | 備份、SBOM、秘密掃描、爬取 | 該等 |

**缺口＝「渲染層已驗、業務流程未驗」**：操作後的資料變化、跨頁一致性、元件契約（inline style/label/狀態掛點）、錯誤自癒。

## 2. 工作項目（有序；每項＝改動 → 測試 → 驗證）

### W1（P0）BOM 容錯
- 改：`platform/src/api.rs` `import_preview` 解析前 strip UTF-8 BOM；`validate_fields` 各欄 `trim_start_matches('\u{feff}')`
- 測：`platform/tests/import_bom.rs`：BOM＋表頭＋1 列 → `row_ok=1`
- 驗：`cargo test --locked -p platform --test import_bom`；curl 上傳 BOM 檔 → 303 入帳

### W2（P0）錯誤訊息可自癒
- 改：`validate_fields`（api.rs:3982）amount/debit/credit/description 訊息加 `{value:?}`（date/currency 已有值，統一）
- 測：單測 `validate_fields_errors_include_received_value`
- 驗：上傳壞資料 → 訊息含實際值

### W3（P0）0 列成功不得回成功
- 改：`import_confirm`：`row_ok == 0` → 422＋失敗清單（HTML）
- 測：`import_confirm_zero_rows_is_422`
- 驗：全錯批次 → 422（非 303）

### W4（P0）元件契約機器化
- 改：`platform/tests/component_contract.rs`：全 manifest GET 頁斷言 (a) 無 `style="` (b) 無 `on*=` (c) `<input>` 有 label/aria-label (d) 表單 POST＋csrf (e) 每頁 ≥1 狀態掛點（`/states`、`/tauri-preview`、`/console/*` 除外）
- 測：本測試；失敗頁修模板
- 驗：測試綠；此後新頁不合約＝紅燈

### W5（P1）17 模組業務流程 E2E
- 改：`platform/tests/module_flows.rs`（每模組一條「操作→資料變化→可見結果」）
  1. 總覽 KPI 與 `/api/health` 交易數一致
  2. 帳務：建立→清單→詳情→試算表金額一致
  3. 帳戶：`/accounts/:id` 詳情＋期初/本期/期末
  4. 匯入：範本 CSV 原樣上傳→入帳（32→34 實證可重現）
  5. 草稿：解析→草稿→採納→過帳
  6. 報表：pivot 三種 measure 參數、parallel months=6/12
  7. 檢索：命中／空態／200 字元截斷
  8. 銀行支付：建批→批次詳情→例外頁→（mock）查詢
  9. 治理稽核：LOGIN/LOGOUT 事件帶 IP/UA
  10. 管理：建號→該號登入→停用→登入失敗
  11. API 中樞：catalog/logs/config 200；`/admin/api/test` POST 契約
  12. 個人設定：改密碼→舊密碼失敗→新密碼成功
  13. 系統狀態/操作指引/體驗室/知識控制台：內容與連結存在性
- 測：`module_flows.rs`（≥ 20 斷言）
- 驗：`cargo test --locked -p platform --test module_flows`

### W6（P1）知識控制台與操作指引一致性
- 改：`/console/*` 23 頁納入 `web_crawl.rs`/`internal_links_all_resolve` 範圍；`/help` 內容與實際功能對照（移除不存在功能之敘述）
- 測：連結 0 死；help 關鍵字對照
- 驗：測試綠

### W7（P1）匯入其餘格式：指引＋錯誤處理（**使用者裁定：不產其他格式範本**）
- 改：`/import` 為 docx/pdf/eml/OCR 各列「格式要求＋最小可行結構」說明（docx：表格列；pdf：文字層、無欄位時前後對齊規則；eml：附件逐檔批次；OCR：需 det/rec/keys 三模型檔）
- 改：OCR 缺模型 → **明確錯誤訊息**（含缺檔名與部署路徑），不得靜默 0 列
- 測：缺模型情境 → 訊息含「模型檔缺失」；`/import` 頁含四種格式說明
- 驗：curl 實證＋頁面斷言（`import_format_guidance`）
- 註：CSV/TSV/XLSX 範本已交付（`platform/templates/import_templates/`）；其餘格式不提供範本（結構不適用）

### W8（P2）映射頁樣本值
- 改：`mapping.html` 每個 select 顯示該來源欄目前 3 筆樣本（資料驅動、零 JS）
- 測：頁面含樣本值
- 驗：curl 檢查

### W9（P2）空態 CTA
- 改：transactions/accounts/drafts/audit/bankpay 空態加下一步 CTA
- 測：空 DB 情境每頁含 CTA
- 驗：測試綠

### W10（P2）搜尋效能預算
- 改：`search` p95 < 500ms 斷言（離線 BM25-only）
- 測：`search_p95_budget`
- 驗：測試綠

## 3. 驗證程序（每輪）

1. `cargo fmt --check`；`cargo clippy --locked -p platform --all-targets -- -D warnings`
2. `cargo test --locked -p platform`（lib＋全部 tests，0 failed）
3. 隔離 target 啟動（8091＋DB 快照）→ 34 頁掃描＋E2E＋三範本閉環（CSV/TSV/XLSX）
5. `docs/standards/REQUIREMENTS_TRACEABILITY.md` 新增 W1–W10 對應列；`PLAN_REGISTER` addendum

## 4. 風險與回滾

- 並行工作線編輯 `api.rs`/`pages.rs`：動工前重測 mtime/行數（C-05）、斷言式補丁（C-04）
- 每項獨立可回退；BOM strip 冪等（無 BOM 檔不受影響）；錯誤訊息變更僅文字
- 8090 由平行線佔用時，驗證一律走 8091＋快照（不互搶）

## 5. 範圍外

- MFA/WebAuthn、Redis、PostgreSQL（基線 SQLite；變更需 POC）
- 桌面 Excel 實際開啟（無 GUI）
- 外觀重設計（單調改進原則；不得推翻 V2）

## 6. 完成條件

W1–W10 全數 PASS；測試 ≥ 160 且 0 failed；34 頁＋連結＋三範本閉環；追蹤矩陣同步；未測項標 NOT TESTED。