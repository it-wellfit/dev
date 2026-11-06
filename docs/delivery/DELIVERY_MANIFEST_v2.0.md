# DELIVERY MANIFEST v2.0 — 交付清單

> 編號：DEL-RD-2.0｜日期：2026-11-06｜對應：REDESIGN_BLUEPRINT_v2.0.md

## A. 文件

| 文件 | 位置 | 狀態 |
|---|---|---|
| 改進方案（架構/模組/介面/互動）＋品質標準與驗收準則 | docs/product/REDESIGN_BLUEPRINT_v2.0.md | 交付 |
| 測試驗證計畫 | docs/testing/REDESIGN_TEST_PLAN_v2.0.md | 交付 |
| 本交付清單 | docs/delivery/DELIVERY_MANIFEST_v2.0.md | 交付 |
| 上位文件（沿用） | PRD.md、ADR-0001/0002/0003、MVP_*、THREAT_MODEL、TEST_STRATEGY | 已存在 |
| 部署與維運 | docs/production/PRODUCTION_READINESS.md（Go/No-Go 現況 NO-GO，授權項目列明） | 已存在，強化後待重審 |

## B. 程式碼（本輪變更）

| 檔案 | 內容 |
|---|---|
| platform/src/api.rs | 分頁（parse_page/LIMIT-OFFSET bind/模板欄位）、健康檢查強化、Secure cookie |
| platform/templates/index.html | 分頁導覽列與頁碼文字 |
| core/src/ledger/engine.rs | query_transaction_summaries_paged |
| platform/src/api.rs mod tests | 分頁 3＋health 2＋cookie 1 |

## C. 測試報告

全域關卡（fmt/clippy/test/audit）與整合測試結果見本檔下方執行記錄與 session log。

## D. 部署與維運指南（現況）

- 啟動：cargo run -p platform --bin flaps-server（FLAPS_PORT 預設 8090，綁 127.0.0.1）
- 健康探測：GET /api/health（本輪強化 degraded 語義，供監控使用）
- 備份：data/flaps.db 單檔複製；還原演練列 NEXT_ACTIONS #1（待執行）
- Session 清理：啟動時過期清除（既有）

## F. 執行記錄（2026-11-06）

| 項目 | 指令 | 結果 |
|---|---|---|
| fmt | cargo fmt -- --check | EXIT=0 |
| clippy | cargo clippy --all-targets -- -D warnings | EXIT=0 |
| 測試 | cargo test --locked | **96 passed / 0 failed** |
| audit | cargo audit | 1 allowed warning（既有接受 RUSTSEC-2026-0374） |
| 零 JS | grep templates | script=0、內聯事件=0 |

## G. 已知限制（誠實清單）

- api.rs 模組化（R1 四步）尚未執行（藍圖 §1 地圖已列，為下一輪首要）
- Lighthouse 基線、E2E 腳本延伸、M3-M6 未執行
- 分頁排序為固定（日期新到舊）；互動式排序屬後續切片
- 部署需擁有者授權（Phase 2 Gate NO-GO 現況有效）