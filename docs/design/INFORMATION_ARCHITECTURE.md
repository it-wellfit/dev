# INFORMATION ARCHITECTURE — 資訊架構

> SSOT：`platform/src/api.rs`（48 條）＋ `platform/src/cross_module_api.rs`（12 條）＝**60 條路由**（2026-11-09 計數）。本文件為路由的使用者視角組織；新增路由必同步本文件（見各分組）。

## Sitemap（頁面層級，2026-11-09 現況）

```
/login                       登入（公開）
/                            工作台首頁（儀表板＋KPI 定義註＋角色起始頁＋批量覆核表單＋SVG 圖）
├── #role-landing            角色起始頁（G1.3：FINANCE→覆核佇列／STORE→報表／預設總覽）
├── #dashboard               總覽（KPI 四卡＋定義註＋最近交易＋月度圖）
├── #bulk-review             批量裁決表單（G1.4：POST /reviews/bulk）
├── #transactions            交易管理（面板）
├── #accounts                會計科目
├── #ai-decisions            AI 決策支援（降級說明）
├── #modules                 跨模組狀態（Slice 13：search/sync/com/ai）
├── #audit                   稽核軌跡（最近 10 筆）
├── #new-transaction         新增交易（表單）
├── #states                  介面狀態樣本（E3.7 互動態＋八資料態 G1.1）
/transactions                交易清單（搜尋 q＋分頁；零結果≠空庫）
/transactions/:id            交易詳情（借貸平衡檢查；返回連結保留 q+page）
/reviews/bulk                批量覆核結果（POST；逐列審計＋規則名）
/reports/monthly[.csv]       月報表（HTML/CSV）
/reports/pivot               樞紐分析（白名單維度矩陣）
/reports/parallel            平行座標多維投影
/import                      快速匯入（上傳＋預檢＋確認；SHA-256 防重放）
/drafts                      草稿（規則解析＋AI 建議表單＋清單）
/states                      設計系統標本頁（E3.7 互動態＋八資料態）
/admin/users                 使用者管理（建立/停用/重設密碼）
/admin/users/toggle|reset_password
/admin/parse_coverage        解析覆蓋率（OQ-8-2）
/admin/api/catalog|config|logs|test   API 資料管理主控台（並行線 Slice 13/E2E）
/admin/audit_evidence.csv    審計證據導出（G3.1 四要件標頭；ROLE_MANAGE）
/admin/control_coverage.json 控制庫跨框架映射＋lint（G3.2；ROLE_MANAGE）
/admin/ccm.json              CCM 三指標（G3.3；ROLE_MANAGE）
/api/health                  健康探針（JSON；DB 不可達→503 degraded）
/api/search                  混合檢索（PHASE-R：真實融合分數＋via 溯源）
/api/bankpay/*               BankPay 設定/匯出/操作（並行線 E2E；ROLE_MANAGE）
/logout                      登出（POST）
```

## 導航規則

| 規則 | 實作 |
|------|------|
| 面板切換 | CSS `:target`（零 JS） |
| 導航高亮 | `:has()` |
| 保護頁 | 未認證 → 303 /login |
| 唯讀頁 | RBAC LEDGER_READ（報表）；ROLE_MANAGE（管理/覆蓋率/治理導出） |
| 寫入 | 全部表單 POST＋CSRF＋303 PRG（例外：/reviews/bulk 結果頁 200——逐列重檢 PENDING_REVIEW，重送安全） |
| 檢索 | /api/search 回真實融合分數＋via（"hybrid"/"bm25"降級/"direct"） |

## 使用者旅程 ↔ 頁面對映

| 旅程（USER_JOURNEYS.md） | 頁面 |
|---|---|
| 登帳（US-5） | /new?step=1..3 → POST /transactions → 303 → /#transactions |
| 查詢覆核（US-11/12） | /#transactions、/transactions/:id（返回保留上下文）、批量覆核、/reports/monthly、/reports/pivot |
| 匯入（US-8） | /import → preview → confirm → 303 |
| 草稿解析（US-9/10） | /drafts → /drafts/parse|ai_suggest → /new?step=2 |
| 管理（US-15/16） | /admin/users、/admin/parse_coverage、/admin/api/* |
| 稽核取證（US-稽核） | /admin/audit_evidence.csv、/admin/control_coverage.json、/admin/ccm.json |
