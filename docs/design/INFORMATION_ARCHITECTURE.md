# INFORMATION ARCHITECTURE — 資訊架構

> SSOT：`platform/src/api.rs` 路由表（22 條路由）。本文件為路由的使用者視角組織。

## Sitemap（頁面層級）

```
/login                      登入（公開）
/                           工作台首頁（儀表板＋KPI＋SVG 圖）
├── #dashboard              總覽（面板 1：KPI＋最近交易＋月度圖）
├── #transactions           交易管理（面板 2）
├── #accounts               會計科目（面板 3）
├── #ai-decisions           AI 決策支援（面板 4，降級說明）
├── #new-transaction        新增交易（面板 5：表單）
├── #states                 介面狀態樣式（面板 6：設計審查用）
/reports/monthly            月報表（HTML）
/reports/monthly.csv        月報表 CSV 下載
/reports/pivot              樞紐分析（白名單維度矩陣）
/reports/parallel           平行座標多維投影
/import                     快速匯入（上傳＋預檢＋確認）
/drafts                     草稿（規則解析＋AI 建議表單＋清單）
/admin/users                使用者管理（建立/停用/重設密碼）
/admin/parse_coverage       解析覆蓋率（OQ-8-2 數據）
/api/health                 健康探針（JSON）
/logout                     登出（POST）
```

## 導航規則

| 規則 | 實作 |
|------|------|
| 面板切換 | CSS `:target`（零 JS） |
| 導航高亮 | `:has()` |
| 保護頁 | 未認證 → 303 /login |
| 唯讀頁 | RBAC LEDGER_READ（報表）；ROLE_MANAGE（管理/覆蓋率） |
| 寫入 | 全部表單 POST＋CSRF＋303 PRG |

## 使用者旅程 ↔ 頁面對映

| 旅程（USER_JOURNEYS.md） | 頁面 |
|---|---|
| 登帳（US-5） | /new?step=1..3 → POST /transactions → 303 → /#transactions |
| 查詢覆核（US-11/12） | /#transactions、/reports/monthly、/reports/pivot |
| 匯入（US-8） | /import → preview → confirm → 303 |
| 草稿解析（US-9/10） | /drafts → /drafts/parse|ai_suggest → /new?step=2 |
| 管理（US-15/16） | /admin/users、/admin/parse_coverage |
