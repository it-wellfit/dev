# DEMO GUIDE — AuxBridge 整合工作台展示操作指南

> 對象：展示者／評審。展示版本：`data\demo.db`（全部資料以 `[DEMO]` 標示）。
> 重建展示資料：`powershell -File scripts\reset-demo.ps1`（可重複執行）。

## 1. 啟動 Demo

```powershell
# 1) 種入展示資料（可重複執行，冪等）
powershell -File scripts\seed-demo.ps1

# 2) 以展示資料庫啟動伺服器
$env:AUXBRIDGE_DB_URL = "sqlite://<專案根>/data/demo.db?mode=rwc"
$env:AUXBRIDGE_PORT = "8090"
target\debug\auxbridge-server.exe

# 3) 開瀏覽器（零 JavaScript，任何現代瀏覽器）
http://127.0.0.1:8090
```

## 2. 展示帳號

| 帳號 | 密碼 | 角色 | 權限 |
|------|------|------|------|
| admin | demo-admin-2026 | ADMIN | 全部（含使用者管理） |
| demo-finance | demo-finance-2026 | FINANCE | 可登帳、可查報表 |
| demo-viewer | demo-viewer-2026 | STORE | 唯讀（登帳會被 403 拒絕——可展示 RBAC） |

> 角色 seed（`core/src/permission.rs::initialize_default`）含 ADMIN／FINANCE／STORE／POS／**VIEWER**（唯讀報表；登入後導向 `/reports/monthly`，與 STORE 同為唯讀入口）。示範帳號 demo-viewer 仍指派 STORE 以保留 RBAC 差異展示。

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

重置會刪除展示資料庫並重新種入相同的 [DEMO] 資料——**不會**影響 `data\auxbridge.db`（正式資料庫檔案獨立）。

## 6. 本機單機 Demo（8096）與登入疑難排除

> 2026-11-15 實證（見 `docs/PLAN_REGISTER.md` addendum #49）。此節描述「單機演示模式」：
> 直接啟動 release 執行檔、使用正式資料庫 `data\auxbridge.db`（repo 根相對）、僅在本機 127.0.0.1 提供服務。

### 6.1 啟動

| 步驟 | 做法 | 成功判準 |
|------|------|----------|
| 1 | 雙擊桌面上的 `start_auxbridge.bat`（ASCII-only 啟動器：cmd 無法可靠解析 UTF-8 中文路徑，故於執行期解析 repo 路徑） | 日誌出現 `AuxBridge server listening on http://127.0.0.1:8096 (zero-JS UI)` |
| 2 | 瀏覽器開啟 http://127.0.0.1:8096 | 導向 `/login`，標題「登入 — AuxBridge 整合工作台」 |
| 3 | 以 `admin` 登入 | 進入總覽 `/`，右上顯示 `使用者：admin`、角色 `ADMIN` |

| 帳號 | 密碼 | 說明 |
|------|------|------|
| admin | Admin123456 | 僅限本機單機演示；密碼由啟動器的 `AUXBRIDGE_ADMIN_PASSWORD` 決定，服務啟動時會自動與資料庫同步（故改過密碼後重啟即恢復） |

### 6.2 疑難排除（症狀 → 可能原因 → 處理方式）

| 症狀 | 可能原因 | 處理方式 |
|------|----------|----------|
| 瀏覽器顯示無法連線 | 服務未啟動，或舊行程仍佔用 8096 | 關閉舊 `auxbridge-server.exe` 行程後重新執行啟動器；以 `Test-NetConnection 127.0.0.1 -Port 8096` 確認 |
| 登入頁可開、但登入一直失敗 | 資料庫內 admin 密碼與環境變數不一致 | 環境變數為部署端權威：重啟服務即自動同步（並撤銷舊 session）；此為刻意取捨，見 UX_CONTRACTS「已知限制」 |
| 某頁 404（例如 `/forgot-password`） | 執行檔版本落後於原始碼（源碼已加路由、二進位未重建） | `cargo build --release -p platform --bin auxbridge-server --locked --target-dir web/src-tauri/target` 後重新啟動服務 |
| 忘了密碼 | 需要自助重設 | 登入頁「忘記密碼？」→ 登記請求 → 管理員於 `/admin/users` 核發一次性連結（30 分鐘有效、用後即焚；重設後所有 session 失效） |
| 登入後立刻被登出 | 密碼剛被同步或重設，舊 session 已撤銷 | 重新登入即可 |
| 頁面樣式錯亂 | 舊版 CSS 快取 | 強制重新整理（Ctrl+F5）；樣式 SSOT 為 `web/assets/design-system.css` |

### 6.3 命令列驗證（不需瀏覽器）

```powershell
# 登入（PRG：預期 303，並取得 session cookie）
curl.exe -s -c "$env:TEMP\cookies.txt" -b "$env:TEMP\cookies.txt" -X POST "http://127.0.0.1:8096/login" `
  -H "Content-Type: application/x-www-form-urlencoded" --data "username=admin&password=Admin123456" -o NUL -w "%{http_code}`n"

# 免登入頁可達（預期 200）
curl.exe -s -o NUL -w "%{http_code}`n" "http://127.0.0.1:8096/forgot-password"
```
