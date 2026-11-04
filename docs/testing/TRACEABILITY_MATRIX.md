# 需求追溯矩陣 — Flaps ERP 工作區

> 規則：每個規格 ID 追溯至 **使用者場景 → 整合測試 → 程式模組 → 提交**。測試名稱即 BDD 場景名（見 `docs/quality/bdd_acceptance.md`）。證據規則：測試結果綁 session log（`.kilo/sessions/20261103-dev-elite-impl/tests/`）與 git 修訂版。

## Slice 8（資料獲取與洞察層）

| 規格 ID | 需求 | 場景/測試 | 模組 | 提交 |
|---|---|---|---|---|
| IMP-001 | 匯入預檢無帳務副作用 | import_preview_reports_and_persists_no_ledger_effects | api.rs import_preview／persist_batch | 662ff39 |
| IMP-002 | 確認僅登帳合格列＋批次稽核 | import_confirm_posts_only_valid_rows | api.rs import_confirm | 662ff39 |
| IMP-003 | 重複確認機械防護 409 | import_confirm_is_replay_guarded | api.rs import_confirm（status UPDATE guard） | 662ff39 |
| DRAFT-001 | 非結構化→草稿＋依據卡 | drafts_parse_creates_draft_and_wizard_prefills＋parser 單元 5 條 | core parser.rs／api.rs drafts_parse | 662ff39 |
| WIZ-001 | 精靈錯誤停留原步＋規則名稱 | wizard_step2_rejects_same_account_and_preserves_values | api.rs wizard_step2 | 662ff39 |
| RPT-001 | 月報＋CSV（含 500ms 預算） | monthly_report_page_and_csv_render_posted_data＋close_out 效能測試 | core engine.monthly_by_account／api.rs report_monthly_* | 662ff39／bc68b7e |
| ZJS-001 | 全頁 `<script>`=0（含 SVG 頁） | dashboard_renders_chart_svg_zero_js | core parallel.rs／api.rs build_index | 662ff39 |

## Slice 9（硬化收尾）

| 規格 ID | 需求 | 場景/測試 | 模組 | 提交 |
|---|---|---|---|---|
| P1-9 | 停用即撤銷會話 | deactivating_a_user_revokes_their_live_session | api.rs admin_toggle_user（sessions DELETE） | bc68b7e |
| P1-8-R | 管理員密碼重設（輪換＋撤銷＋稽核） | admin_password_reset_revokes_sessions_and_rotates_credentials | api.rs admin_reset_password | bc68b7e |
| P3-2 | 設計 Token 同步 | css_design_tokens_are_in_sync_between_docs_and_deployment | platform/tests/close_out.rs | bc68b7e |
| P3-1 | 報表效能預算 <500ms | monthly_report_stays_under_500ms_with_2000_transactions | platform/tests/close_out.rs | bc68b7e |

## Slice 10（多維洞察進階與方法論硬化）

| 規格 ID | 需求 | 場景/測試 | 模組 | 提交 |
|---|---|---|---|---|
| S10-FR-1 | 白名單樞紐矩陣 | pivot_matrix_matches_hand_computed_values／pivot_rejects_unknown_dimensions_with_rule_name | api.rs report_pivot_page／assemble_pivot／dim_expr | 本切片 |
| S10-FR-2 | 平行座標多維投影（零 JS） | parallel_projection_renders_one_line_per_account_zero_js＋core parallel 單元 5 條 | core parallel.rs／api.rs report_parallel_page | 本切片 |
| S10-FR-3 | AI 建議草稿（旗標＋schema 閘門＋永不自動落地） | ai_suggest_disabled_by_default_returns_503／ai_suggestion_adopt_never_posts_to_ledger／ai_suggestion_schema_validation_rejects_malformed_output | core parser.parse_ai_suggestion／api.rs ai_suggest／drafts_adopt | 本切片 |
| S10-FR-4 | Builder 重構（allow 歸零） | transaction_builder_constructs_domain_objects＋全測試語義不變 | core ledger transaction.rs（TransactionBuilder/TransactionItemBuilder） | 本切片 |
| S10-FR-5 | 解析覆蓋率量測（OQ-8-2 數據） | parse_coverage_computes_hit_ratio | api.rs parse_coverage_page | 本切片 |
| H-001 | clippy allow 歸零（基線四項 EXIT=0） | cargo clippy --all-targets -- -D warnings | 全 workspace | 本切片 |

## 對映到國際標準

| 標準 | 條款/實踐 | 本工作區落實 | 證據 |
|---|---|---|---|
| ISO/IEC 25010（效能效率） | 時間特性 | RPT-001 <500ms 斷言（2k 交易） | close_out 效能測試 |
| ISO/IEC 25010（可靠性） | 容錯性 | 匯入重放防護 409、批次部分失敗誠實回報 | IMP-003 測試 |
| ISO/IEC 25010（安全性） | 可追究性 | audit_log（TRANSACTION_POSTED/IMPORT_CONFIRMED/PASSWORD_RESET） | THREAT_MODEL.md |
| WCAG 2.2 | 非文字內容（1.1.1）、鍵盤（2.1.1） | SVG title/desc/aria-label；skip-link；全頁零 JS | PAR-001 測試 |
| OWASP ASVS（注入） | 參數化＋白名單 | 全 SQL bind；樞紐白名單映射（R-PIV-WHITELIST） | PIV-002 測試 |
| OWASP（會話管理） | 會話撤銷 | 停用/重設即撤銷（P1-9/重設） | 對應測試 |
| DORA | 測試自動化 | 66→70+ 測試、提交粒度（每切片一 commit） | git log |
