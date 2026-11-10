# API DESIGN — API 設計

> SSOT：`platform/src/api.rs`（48 條）＋ `platform/src/cross_module_api.rs`（12 條）＝60 條路由（2026-11-09）。零 JS：全部頁面為伺服器渲染 HTML；寫入為表單 POST＋CSRF（例外見下）＋303 PRG。
> 本表為**核心契約節錄**（建立時 22 條；2026-11-09 增補至現況分組）。完整清單以原始碼路由表為準，使用者視角組織見 `docs/design/INFORMATION_ARCHITECTURE.md`。

## 路由表（核心契約＋2026-11-09 增補群組）

| 方法 | 路徑 | 權限 | CSRF | 說明 |
|------|------|------|------|------|
| GET | /login | 公開 | — | 登入頁 |
| POST | /login | 公開（速率鎖定 5/15min） | — | 認證→會話 Cookie |
| POST | /logout | 會話 | ✓ | 撤銷會話＋清 Cookie |
| GET | / | 會話 | — | 工作台首頁（KPI＋定義註＋角色起始頁＋批量覆核表單＋SVG） |
| GET | /assets/design-system.css | 會話 | — | 設計系統（嵌入；docs 快照同步） |
| GET | /api/health | 公開 | — | JSON 探針（遷移版本＋交易數；DB 不可達 503） |
| POST | /transactions | 會話＋LEDGER_WRITE | ✓ | 登帳（借貸雙欄→引擎平衡→稽核）→303 |
| GET | /transactions | 會話 | — | 交易清單（搜尋 q＋分頁；零結果≠空庫） |
| GET | /transactions/:id | 會話 | — | 詳情（借貸平衡檢查；返回連結保留 q+page） |
| POST | /reviews/bulk | 會話＋LEDGER_WRITE | ✓ | 批量覆核（逐列審計＋規則名；重送安全；結果頁 200——逐列重檢即防重放） |
| POST | /import/preview | 會話＋LEDGER_WRITE | ✓（multipart） | 乾跑預檢（零帳務副作用） |
| POST | /import/confirm | 會話＋LEDGER_WRITE | ✓ | 批次登帳（重放 409）＋批次稽核 |
| GET/POST | /drafts、/drafts/parse|ai_suggest|adopt | 會話（adopt 需 LEDGER_WRITE） | ✓ | 草稿＋AI 建議（schema 閘門；永不直接登帳） |
| GET/POST | /new、/new/step2 | 會話 | ✓ | 三步精靈 |
| GET | /reports/monthly[.csv]、/reports/pivot、/reports/parallel | 會話＋LEDGER_READ | — | 月報/樞紐（白名單維度）/平行座標 |
| GET/POST | /admin/users（+toggle、reset_password） | 會話＋ROLE_MANAGE | ✓(POST) | 使用者管理（撤銷會話＋稽核） |
| GET | /admin/parse_coverage | 會話＋ROLE_MANAGE | — | 解析覆蓋率 |
| GET | /admin/audit_evidence.csv | 會話＋ROLE_MANAGE | — | 審計證據導出（G3.1 四要件標頭＋sha256 摘要） |
| GET | /admin/control_coverage.json | 會話＋ROLE_MANAGE | — | 控制庫跨框架映射＋lint（G3.2） |
| GET | /admin/ccm.json | 會話＋ROLE_MANAGE | — | CCM 三指標（G3.3；M2 誠實 null） |
| GET/POST | /api/search | 公開 | —（讀/索引） | **混合檢索**（PHASE-R）：真實融合分數＋via（"hybrid"/"bm25"/"direct"）；索引 POST doc_id+content |
| GET/POST | /api/bankpay/*（catalog/config/logs/test/export…） | 會話＋ROLE_MANAGE | ✓(寫) | BankPay 設定/匯出/操作（並行線 E2E；erp_config 持久化 10 鍵） |
| GET | /admin/api/catalog|config|logs|test | 會話 | — | API 資料管理主控台（零 JS 頁面群） |

> 2026-11-09 增補群組來源：G1.2/G1.4（詳情/批量覆核）、G3.1–G3.3（治理導出）、PHASE-R（混合檢索）、並行線 Slice 13/E2E（cross-module＋BankPay）。

## 錯誤模型

| 狀態 | 情境 | 訊息規範 |
|------|------|----------|
| 303 | PRG 成功／未認證→/login | — |
| 400 | 表單驗證失敗 | 問題＋原因＋處理方式（繁中） |
| 403 | RBAC／CSRF | 指名缺少權限碼 |
| 409 | 匯入重放 | 機械防護 |
| 503 | DB 不可達（health degraded） | 狀態誠實化 |
| 500 | 資料庫／建構 | 詳情＋重試指引 |

## 契約測試

全部路由的行為由整合測試鎖定（見 docs/testing/TRACEABILITY_MATRIX.md）——路由表變更必須同步測試與本文件（SHARED_STANDARDS §4.3）。
