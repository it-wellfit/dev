# DEMO GUIDE — Flaps ERP 工作台展示操作指南

> 對象：展示者／評審。展示版本：`data\demo.db`（全部資料以 `[DEMO]` 標示）。
> 重建展示資料：`powershell -File scripts\reset-demo.ps1`（可重複執行）。

## 1. 啟動 Demo

```powershell
# 1) 種入展示資料（可重複執行，冪等）
powershell -File scripts\seed-demo.ps1

# 2) 以展示資料庫啟動伺服器
$env:FLAPS_DB_URL = "sqlite://<專案根>/data/demo.db?mode=rwc"
$env:FLAPS_PORT = "8090"
target\debug\flaps-server.exe

# 3) 開瀏覽器（零 JavaScript，任何現代瀏覽器）
http://127.0.0.1:8090
```

## 2. 展示帳號

| 帳號 | 密碼 | 角色 | 權限 |
|------|------|------|------|
| admin | demo-admin-2026 | ADMIN | 全部（含使用者管理） |
| demo-finance | demo-finance-2026 | FINANCE | 可登帳、可查報表 |
| demo-viewer | demo-viewer-2026 | STORE | 唯讀（登帳會被 403 拒絕——可展示 RBAC） |

## 3. 核心展示流程（建議順序）

| # | 流程 | 路徑 | 展示重點 |
|---|------|------|----------|
| 1 | 登入與權限 | `/login` | Argon2id；demo-viewer 登入後導航無管理項 |
| 2 | 儀表板總覽 | `/` | KPI、月度借貸 SVG 圖、[DEMO] 交易清單 |
| 3 | 引導式登帳 | `/new` | 三步精靈：模板→借貸→確認；錯誤停留原步並顯示規則名稱 |
| 4 | 快速匯入 | `/import` | 上傳 CSV→預檢報告（合格/錯誤逐列）→確認登帳 |
| 5 | 草稿解析 | `/drafts` | 貼文字→依據卡→採用進精靈；AI 建議（實驗性） |
| 6 | 報表 | `/reports/monthly`、`/reports/pivot`、`/reports/parallel` | 月報＋CSV 下載、樞紐矩陣、平行座標多維投影 |
| 7 | 管理 | `/admin/users` | 建立帳號、停用（會話即時撤銷）、密碼重設 |

## 4. 安全展示亮點

- 整頁**零 JavaScript**（檢視原始碼無 `<script>`）——所有互動為表單 POST＋CSS
- 帳號停用即**撤銷全部會話**（停用 demo-viewer 後其頁面立即 303 → /login）
- 密碼以 **Argon2id** 雜湊儲存；登入失敗 5 次鎖定 15 分鐘
- 每筆登帳寫入 **audit_log**（可追蹤操作者與時間）

## 5. 展示後重置

```powershell
powershell -File scripts\reset-demo.ps1
```

重置會刪除展示資料庫並重新種入相同的 [DEMO] 資料——**不會**影響 `data\flaps.db`（正式資料庫檔案獨立）。
