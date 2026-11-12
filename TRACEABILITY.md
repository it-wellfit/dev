# 追溯（Traceability）— Dev 根工作區

- 版本：v1.0｜生效日期：2026-11-12｜狀態：Active
- 追溯規則 SSOT：`docs/standards/GOVERNANCE_STANDARD.md`（證據與聲明規則）

## 1. 追溯鏈

```
需求/計畫 → 變更登記（PLAN_REGISTER addendum） → 實作（commit） → 驗證證據（測試/量測） → 聲明（CHANGELOG）
```

## 2. 登錄節點

| 節點 | 位置 | 用途 |
|---|---|---|
| 計畫 | `.kilo/plans/*.md` | 工作分解與驗收命令 |
| 變更登記 | `docs/PLAN_REGISTER.md`（addendum 編號） | 每輪交付之項目/證據/未竟 |
| 決策 | `docs/adr/`＋`ADR_REGISTER.md` | 持久工程決策 |
| 聲明 | `CHANGELOG.md` | 使用者可見變更 |
| 風險/缺口 | `docs/GAPS_AND_RISKS.md` | 已知風險與未決事項 |
| 產品聲明 vs 證據 | `Knowledge/traceability/claims-vs-evidence.md` | 知識庫聲明對照 |

## 3. 證據類型與保存

| 類型 | 範例 | 保存位置 |
|---|---|---|
| 測試 | `cargo test --locked` 之套件/通過數 | 變更登記（addendum）＋CI 記錄 |
| 量測 | 二進位大小、相依數、建置時間 | 變更登記（含日期與方法） |
| 供應鏈 | SBOM、audit/deny/machete 結果 | `docs/release/sbom/`、`.cargo/audit.toml`、`deny.toml` |
| 發行 | 簽章、latest.json、安裝包 | `docs/release/UPDATER_RUNBOOK.md` §5 |

## 4. 規則

1. 未量測者標 `UNKNOWN`；不得將 `UNKNOWN` 記為 `PASS`。
2. 數字聲明須附方法與日期；跨修訂版之結果須重新量測。
3. 合成/示範資料不得作為真實交付證據。
4. 變更登記之「未竟」欄必須如實保留（不得刪除歷史，只疊加）。

## 5. 變更紀錄

| 日期 | 版本 | 說明 |
|---|---|---|
| 2026-11-12 | v1.0 | 依 SHARED_STANDARDS §4.1 建立；節點對應現行檔案 |