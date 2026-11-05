# DEMO DATA — 展示資料說明

> 資料來源：`scripts\seed-demo.ps1`（經正式 HTTP API 種入，非直接寫 DB）。
> 資料庫：`data\demo.db`（獨立檔案；正式資料庫 `data\flaps.db` 不受影響）。

## 1. 識別規則

1. 所有種入的描述以 **`[DEMO]`** 開頭。
2. 所有示範科目代碼使用語義前綴（`OFFICE-EXP`、`REVENUE-001`、`BANK-001`、`CASH-001`、`OPENING-EQUITY`、`SUPPLIES-EXP`、`PAYROLL-EXP`、`UTILITIES-EXP`、`TRAVEL-EXP`）。
3. 示範帳號：`demo-finance`、`demo-viewer`（密碼見 DEMO_GUIDE）。

## 2. 種入清單（10 筆交易，全部借貸平衡）

| # | 日期 | 借方 | 貸方 | 金額（TWD） | 描述 |
|---|------|------|------|------|------|
| 1 | 2026-09-05 | OFFICE-EXP | BANK-001 | 25,000 | [DEMO] 9月辦公室租金 |
| 2 | 2026-09-15 | SUPPLIES-EXP | CASH-001 | 3,200 | [DEMO] 文具採購 |
| 3 | 2026-10-05 | OFFICE-EXP | BANK-001 | 25,000 | [DEMO] 10月辦公室租金 |
| 4 | 2026-10-12 | BANK-001 | REVENUE-001 | 88,000 | [DEMO] 專案收入（收款） |
| 5 | 2026-10-20 | PAYROLL-EXP | BANK-001 | 65,000 | [DEMO] 10月薪資 |
| 6 | 2026-11-01 | BANK-001 | OPENING-EQUITY | 500,000 | [DEMO] 期初資本（開帳） |
| 7 | 2026-11-05 | SUPPLIES-EXP | CASH-001 | 1,850 | [DEMO] 雜項採購 |
| 8 | 2026-11-08 | CASH-001 | REVENUE-001 | 42,000 | [DEMO] 零售收入（收款） |
| 9 | 2026-11-10 | UTILITIES-EXP | BANK-001 | 6,800 | [DEMO] 水電費 |
| 10 | 2026-11-12 | TRAVEL-EXP | CASH-001 | 5,600 | [DEMO] 出差差旅 |

合計：借方 = 貸方 = 777,450 TWD（平衡不變量）。

## 3. 示範帳號

| user_id | 帳號 | 角色 | 密碼 |
|---|---|---|---|
| user-admin | admin | ADMIN | demo-admin-2026（或種入時指定） |
| user-demo-finance | demo-finance | FINANCE | demo-finance-2026 |
| user-demo-viewer | demo-viewer | STORE | demo-viewer-2026 |

## 4. 重置

```powershell
powershell -File scripts\reset-demo.ps1
```

刪除 `data\demo.db*` 後重新種入——**可重複執行**（已驗證兩次執行結果一致：10/10 交易）。

## 5. 禁止事項

- 不得將 `data\demo.db` 與 `data\flaps.db` 混用（檔案獨立）。
- 不得移除 `[DEMO]` 前綴後宣稱為正式資料。
- Demo 資料不得作為任何正式財務報告的依據。
