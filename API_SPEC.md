# API 規格（API Specification）— Dev 根工作區

- 版本：v1.0｜生效日期：2026-11-12｜適用範圍：platform HTTP API｜狀態：Active
- **SSOT**：路由與契約目錄由 `apihub` registry 驅動，執行期可查 `GET /admin/api/catalog`（不得在本檔複寫完整路由表，避免雙份事實來源）。

## 1. 傳輸與格式

| 項目 | 規則 |
|---|---|
| 通訊協定 | HTTP/1.1（本機 `:8090`；對外由反向代理終結 TLS） |
| 內容類型 | HTML（伺服器渲染）為主；資料端點 `application/json` |
| 編碼 | UTF-8（禁止 BOM；`platform/tests/guards.rs` 掃描） |
| 金额 | JSON 整數（最小貨幣單位；禁浮點） |

## 2. 方法語義（專案約束）

| 方法 | 用途 | 成功後 |
|---|---|---|
| GET | 讀取（冪等、可快取） | 200 |
| POST | 建立／修改／刪除 | **PRG**：303 轉址至結果頁 |
| 其他 | 不支援（除非另有登錄） | 405 |

## 3. 認證與授權

- 工作階段 cookie：`auxbridge_session`（`Path=/`、`HttpOnly`、`SameSite=Lax`）。
- 未登入存取受保護頁 → 303 至登入頁（**不得**以 403 取代；見 `.kilo/memories/corrections.md` 之 HTTP 驗證結論）。
- 授權決策 fail-closed：預設拒絕，僅明確允許之角色可存取（`core/src/permission/`）。
- 高風險操作（付款、匯入確認、權限變更）需二次確認與審計記錄。

## 4. 錯誤契約

每個使用者可見錯誤須含三段：**問題＋可能原因＋處理方式**（憲章 §4／`SHARED_STANDARDS` §8.2）。
伺服器端錯誤以 `thiserror` 型別化錯誤回傳，禁止洩漏堆疊或秘密。

## 5. 版本與相容

- 以 `apihub` registry 之 `version` 欄位標示契約版本；破壞性變更需新版本號與遷移說明。
- 契約變更屬 High 風險（`SHARED_STANDARDS` §6.1）→ 需契約測試與文件同步（同一提交）。

## 6. 變更紀錄

| 日期 | 版本 | 說明 |
|---|---|---|
| 2026-11-12 | v1.0 | 依 SHARED_STANDARDS §4.1 建立；路由 SSOT 指向 apihub registry |