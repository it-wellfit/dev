# API DESIGN — API 設計

> SSOT：`platform/src/api.rs` 路由表。零 JS：全部頁面為伺服器渲染 HTML；寫入為表單 POST＋CSRF＋303 PRG。

## 路由表（22 條）

| 方法 | 路徑 | 權限 | CSRF | 說明 |
|------|------|------|------|------|
| GET | /login | 公開 | — | 登入頁 |
| POST | /login | 公開（速率鎖定 5/15min） | — | 認證→會話 Cookie |
| POST | /logout | 會話 | ✓ | 撤銷會話＋清 Cookie |
| GET | / | 會話 | — | 工作台首頁（KPI＋SVG＋清單） |
| GET | /assets/design-system.css | 會話 | — | 設計系統（嵌入） |
| GET | /api/health | 公開 | — | JSON 探針 |
| POST | /transactions | 會話＋LEDGER_WRITE | ✓ | 登帳（借貸雙欄→引擎平衡→稽核）→303 |
| GET | /import | 會話＋LEDGER_WRITE | — | 匯入頁＋批次報告 |
| POST | /import/preview | 會話＋LEDGER_WRITE | ✓（multipart） | 乾跑預檢（零帳務副作用） |
| POST | /import/confirm | 會話＋LEDGER_WRITE | ✓ | 批次登帳（重放 409）＋批次稽核 |
| GET | /drafts | 會話 | — | 草稿頁＋AI 建議表單 |
| POST | /drafts/parse | 會話 | ✓ | 規則解析→草稿→303 精靈 |
| POST | /drafts/ai_suggest | 會話 | ✓ | AI 建議（旗標預設關→503；schema 閘門） |
| POST | /drafts/adopt | 會話 | ✓ | 採用建議→草稿（永不直接登帳） |
| GET | /new | 會話 | — | 三步精靈（step 1..3） |
| POST | /new/step2 | 會話 | ✓ | 步驟 2 驗證→步驟 3 確認 |
| GET | /reports/monthly | 會話＋LEDGER_READ | — | 月報（HTML） |
| GET | /reports/monthly.csv | 會話＋LEDGER_READ | — | 月報 CSV |
| GET | /reports/pivot | 會話＋LEDGER_READ | — | 樞紐矩陣（白名單維度） |
| GET | /reports/parallel | 會話＋LEDGER_READ | — | 平行座標 SVG |
| GET | /admin/users | 會話＋ROLE_MANAGE | — | 使用者管理 |
| POST | /admin/users | 會話＋ROLE_MANAGE | ✓ | 建立使用者 |
| POST | /admin/users/toggle | 會話＋ROLE_MANAGE | ✓ | 停用/啟用（撤銷會話） |
| POST | /admin/users/reset_password | 會話＋ROLE_MANAGE | ✓ | 密碼重設（撤銷＋稽核） |
| GET | /admin/parse_coverage | 會話＋ROLE_MANAGE | — | 解析覆蓋率 |

## 錯誤模型

| 狀態 | 情境 | 訊息規範 |
|------|------|----------|
| 303 | PRG 成功／未認證→/login | — |
| 400 | 表單驗證失敗 | 問題＋原因＋處理方式（繁中） |
| 403 | RBAC／CSRF | 指名缺少權限碼 |
| 409 | 匯入重放 | 機械防護 |
| 500 | 資料庫／建構 | 詳情＋重試指引 |

## 契約測試

全部路由的行為由整合測試鎖定（見 docs/testing/TRACEABILITY_MATRIX.md）——路由表變更必須同步測試與本文件。
