# E.Sun BankPay 整合深化與界面狀態區塊調整 — 詳細設計方案

**計畫編號**：1793782615513  
**版本**：v1.0  
**狀態**：待實作  
**相關計畫**：`1793782615513-wellfit-unified-plan-v2.md`（單一事實來源）

---

## 一、執行摘要

本計畫回應使用者反饋的兩個核心優化需求：

1. **E.Sun BankPay 整合深化**：將 E.Sun 銀行支付功能從 Wellfit 工作區的相對獨立狀態，整合進平台主工作台，成為原生體驗的一部分。
2. **界面狀態區塊調整**：將「介面狀態」（載入中、錯誤、部分不可用等）從功能導航菜單中移除，重新定位為獨立的系統狀態指示器區域，作為全域清晰的狀態指標，而非功能入口。

本方案採用「漸進式整合」策略，不推翻既有成果，僅在現有基礎上進行結構性重構與擴充。

---

## 二、現狀深入分析

### 2.1 E.Sun BankPay 整合現狀

| 維度 | 現狀描述 | 問題 |
|------|---------|------|
| 代碼位置 | `crates/wellfit_bankpay/`（Wellfit 工作區獨立 crate） | 與平台工作台 (`platform/`) 分離，無法直接透過平台 UI 存取 |
| API 路由 | Wellfit CLI 子命令 (`wellfit bankpay export/import/query`) | 無 HTTP API 端點，無法從平台 Web UI 呼叫 |
| 認證機制 | CLI 環境變數 `.env` (`ESUN_MOCK_MODE`, `ESUN_URL` 等) | 與平台 RBAC/Session 體系隔離，無法復用平台登入狀態 |
| 資料流 | BankPay 獨立資料庫/檔案 | 與平台決策帳本、稽核軌跡未串接，無法產生跨系統審計記錄 |
| UI 呈現 | 無平台頁面 | 使用者必須切換到 CLI 環境才能執行銀行支付作業 |

### 2.2 界面狀態區塊現狀

| 項目 | 現狀 | 問題 |
|------|------|------|
| 位置 | `platform/templates/index.html` 導航欄中的 `<a href="#states">介面狀態</a>` | 作為功能選單項目出現，使用者誤以為可點擊進入「功能頁面」 |
| 內容 | 純靜態 HTML 預覽（Loading/失敗/部分不可用/表單驗證錯誤樣式） | 非即時系統狀態，僅為設計樣式展示，無實際監控價值 |
| 互動 | 點擊後跳轉到 `#states` 錨點 | 模糊了「狀態指示」與「功能導航」的界限，違反單一職責原則 |
| 視覺 | 與主功能選單同層級顯示 | 狀態指示器應該是全域常駐、非導向性元素，不應佔用導航欄寶貴空間 |

---

## 三、設計目標

### 3.1 E.Sun BankPay 整合目標

| 目標 | 具體指標 | 驗證方式 |
|------|---------|---------|
| 原生化整合 | 平台主工作台新增「銀行支付」模組區塊，無需離開平台 UI | 用戶可從平台儀表板直接發起 BankPay 作業 |
| API 統一 | 平台新增 `POST /api/bankpay/export`, `POST /api/bankpay/import`, `GET /api/bankpay/query` | 透過瀏覽器或 curl 可直接呼叫平台端點 |
| 認證復用 | 復用平台 `flaps_session` Cookie 與 RBAC 權限檢查 | 未登入用戶存取 BankPay API 返回 401 |
| 稽核串接 | BankPay 作業自動寫入平台 `audit_log` 表 | 每筆支付作業產生對應 `event_type='BANKPAY_*'` 記錄 |
| 雙模式支援 | 模擬模式（Mock）與生產模式並存，透過平台設定切換 | 管理員可在平台設定頁面切換 `ESUN_MOCK_MODE` |

### 3.2 界面狀態區塊調整目標

| 目標 | 具體指標 | 驗證方式 |
|------|---------|---------|
| 導航清理 | 導航欄移除 `#states` 錨點連結 | 導航欄僅保留功能性連結 |
| 狀態儀表板 | 新增 `/api/status` JSON 端點，回傳系統各模組即時狀態 | `curl /api/status` 返回 `{"db":"ok","search":"ok","sync":"ok","com":"ok","ai":"ok","overall":"healthy"}` |
| 全域狀態欄 | 頁面頂部新增狀態指示條（狀態圖標 + 文字 + 快速操作） | 所有頁面共用，滾動時固定在頂部 |
| 錯誤處理 | 各狀態（成功/失敗/進行中/無權限）有明確視覺區分與處理建議 | 點擊狀態欄可展開詳細錯誤與處理步驟 |

---

## 四、架構設計

### 4.1 E.Sun BankPay 整合架構圖

```
┌─────────────────────────────────────────────────────────────┐
│                    平台主工作台 (platform)                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ 儀表板模組   │  │ 交易模組     │  │ 銀行支付模組  │    │
│  │ (/api/*)     │  │ (/api/*)     │  │ (/api/bankpay*)│    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│           │                │                │              │
│           └────────────────┴────────────────┘              │
│                    Axum Router + RBAC Middleware            │
│           │                │                │              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ wellfit_bankpay│  │ audit_log   │  │ erp_config    │    │
│  │ (內部 crate)  │  │ (資料表)     │  │ (設定表)      │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│           │                │                │              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ E.Sun API    │  │ SQLite DB    │  │ 風險控制模組   │    │
│  │ (TLS 1.2+)   │  │ (SQLitePool)│  │ (風險評分)    │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 界面狀態區塊架構圖

```
┌─────────────────────────────────────────────────────────────┐
│  頁面頂部全域狀態列 (Global Status Bar)                      │
│  [🟢 系統正常] 資料庫: 正常 │ 搜尋: 正常 │ 同步: 閒置 │ AI: 可用   │
│                                                  [查看詳情]  │
├─────────────────────────────────────────────────────────────┤
│  導航欄 (僅功能連結)                                          │
│  [總覽] [交易] [會計科目] [AI 決策] [新增交易] [報表] [使用者管理]│
├─────────────────────────────────────────────────────────────┤
│  主內容區                                                      │
│  ...                                                          │
└─────────────────────────────────────────────────────────────┘
```

---

## 五、詳細組件設計

### 5.1 E.Sun BankPay 模組設計

#### 5.1.1 API 端點定義

| 端點 | 方法 | 說明 | 權限 | 請求範例 | 回應範例 |
|------|------|------|------|---------|---------|
| `/api/bankpay/export` | POST | 匯出付款檔案至 E.Sun | `LEDGER_WRITE` | `{"batch_id":"BATCH-001","amount_minor":50000,"currency":"TWD"}` | `{"status":"submitted","batch_id":"BATCH-001","esun_ref":"ESUN-20261109-001"}` |
| `/api/bankpay/import` | POST | 匯入銀行回應檔 | `LEDGER_WRITE` | `{"file_path":"/path/to/response.csv"}` | `{"status":"processed","matched":5,"mismatched":0}` |
| `/api/bankpay/query` | GET | 查詢交易狀態 | `LEDGER_READ` | `?esun_ref=ESUN-20261109-001` | `{"status":"completed","amount":50000,"currency":"TWD"}` |
| `/api/bankpay/config` | GET/POST | 銀行連線設定 | `ROLE_MANAGE` | `{"esun_mock_mode":true,"esun_url":"https://..."}` | `{"esun_mock_mode":true,"connected":true}` |

#### 5.1.2 資料模型

```rust
// 銀行支付批次記錄
pub struct BankPayBatch {
    pub batch_id: String,           // 內部批次 ID
    pub esun_ref: Option<String>,   // E.Sun 參考編號
    pub amount_minor: i64,          // 金額（整數分）
    pub currency: String,           // 幣別
    pub status: BankPayStatus,      // PENDING/SUBMITTED/COMPLETED/FAILED/REJECTED
    pub created_by: String,         # 操作者 user_id
    pub created_at: DateTime,       # 建立時間
    pub completed_at: Option<DateTime>,
}

// 銀行支付稽核記錄
pub struct BankPayAudit {
    pub log_id: String,
    pub batch_id: String,
    pub event_type: String,         // BANKPAY_EXPORT / BANKPAY_IMPORT / BANKPAY_QUERY
    pub performed_by: String,
    pub performed_at: DateTime,
    pub request_payload: Option<String>,
    pub response_payload: Option<String>,
    pub status_code: u16,
}
```

#### 5.1.3 狀態機設計

```
[PENDING] ──(export API 呼叫成功)──> [SUBMITTED] ──(E.Sun 回應成功)──> [COMPLETED]
   │                                      │                                      │
   │                                      │                                      │
   └──(export 失敗/超時)──> [FAILED]      └──(E.Sun 回應失敗)──> [REJECTED]
                                                            │
                                                            └──(人工複核後重試)──> [PENDING]
```

各狀態的用戶引導：
- **PENDING**：顯示「等待提交」，提供「立即提交」按鈕
- **SUBMITTED**：顯示「處理中」，顯示預估完成時間，提供「查詢狀態」按鈕
- **COMPLETED**：顯示「已完成」，顯示交易明細，提供下載憑證
- **FAILED**：顯示「提交失敗」，顯示錯誤碼與建議處理方式，提供「重新提交」按鈕
- **REJECTED**：顯示「銀行退件」，顯示退件原因，要求人工複核後重新建立批次

#### 5.1.4 風險控制設計

| 風險點 | 緩解措施 |
|--------|---------|
| 重複提交 | 批次 ID 唯一性約束，資料庫唯一索引 |
| 金額竄改 | 伺服器端重新計算總額，與前端傳入值比對 |
| 權限越權 | 每個 API 端點檢查 `LEDGER_WRITE` 或 `LEDGER_READ` 權限 |
| 資料外洩 | 回應中不包含完整銀行帳號，僅顯示後四碼 |
| 網路中斷 | 指數退避重試（1s, 2s, 4s, 8s），最多 5 次 |

### 5.2 界面狀態區塊設計

#### 5.2.1 全域狀態列 (Global Status Bar)

```
┌──────────────────────────────────────────────────────────────────────┐
│ [● 系統正常]  資料庫: 正常  │  搜尋: 正常  │  同步: 閒置  │  AI: 可用  │  [查看詳情 ▾]  │
└──────────────────────────────────────────────────────────────────────┘
```

設計說明：
- **位置**：頁面最上方，導航欄之上，全寬橫條
- **高度**：48px
- **背景色**：正常時 `#e7f3e4`（淺綠），異常時 `#fdecea`（淺紅），警告時 `#fff4e0`（淺黃）
- **圖標**：正常時綠色圓點，異常時紅色感叹號，警告時黃色警示圓
- **文字**：狀態標題 + 各模組狀態（用 `|` 分隔）
- **右側操作**：「查看詳情」按鈕，點擊展開狀態詳情面板
- **行為**：滾動時固定頂部（`position: sticky; top: 0`）

#### 5.2.2 狀態詳情面板 (Status Detail Panel)

```
┌──────────────────────────────────────────────────────────────────────┐
│  系統狀態詳情                                                  [✕ 關閉]│
├──────────────────────────────────────────────────────────────────────┤
│  資料庫        ● 正常        最後檢查: 2026-11-09 02:15:30          │
│  搜尋模組      ● 正常        最後檢查: 2026-11-09 02:15:28          │
│  同步模組      ● 閒置        最後同步: 2026-11-09 01:30:00          │
│  AI 模組       ● 可用        模型: ollama:llama3, codellama         │
│  銀行支付      ● 模擬模式    E.Sun 連線: 未連線（模擬中）            │
│                                                                        │
│  最近異常:                                                              │
│  2026-11-09 01:45:12  銀行支付 API 超時 (ETIMEDOUT)                   │
│             處理建議: 檢查網路連線，確認 E.Sun 伺服器狀態               │
│                                                                        │
│  [全部標記為已讀]  [匯出狀態報告]                                      │
└──────────────────────────────────────────────────────────────────────┘
```

設計說明：
- **觸發方式**：點擊頂部狀態列「查看詳情」按鈕
- **樣式**：淺灰色背景（#f8f9fa），邊框 1px（#e0e0e0），圓角 8px
- **位置**：從頂部狀態列下方彈出，覆蓋主內容區上方
- **內容**：各模組狀態圖標、文字、最後檢查時間
- **異常區**：獨立區塊，紅色左邊框，顯示錯誤訊息與處理建議
- **操作**：全部標記為已讀、匯出狀態報告（CSV）

#### 5.2.3 狀態圖標規範

| 狀態 | 圖標 | 顏色 | 動畫 | 適用場景 |
|------|------|------|------|---------|
| 正常 | ● 圓點 | #177245（綠） | 靜態 | 所有模組正常 |
| 警告 | ⚠ 三角形 | #9a5b00（黃） | 慢閃 (2s) | 模組可用但效能降低 |
| 錯誤 | ✕ 感叹號 | #b3261e（紅） | 快閃 (1s) | 模組不可用 |
| 進行中 | ⟳ 圓弧 | #0b5cab（藍） | 連續旋轉 | API 呼叫進行中 |
| 無權限 | 🔒 鎖形 | #5c6b7a（灰） | 靜態 | 使用者無存取權限 |

#### 5.2.4 錯誤處理機制

| 錯誤類型 | 使用者見到的訊息 | 處理建議 | 自動恢復 |
|---------|----------------|---------|---------|
| 資料庫連線失敗 | 「資料庫連線中斷」 | 「請確認資料庫服務是否運行，或聯絡系統管理員」 | 每 30s 重試連線 |
| API 超時 | 「請求逾時」 | 「請稍後重試，或檢查網路連線」 | 指數退避重試 3 次 |
| 權限不足 | 「無權限執行此操作」 | 「請聯絡管理員授權 LEDGER_WRITE 權限」 | 無 |
| 銀行回應錯誤 | 「銀行交易失敗：{錯誤碼}」 | 「請確認付款資訊正確，或聯絡銀行客服」 | 無，需人工介入 |
| 驗證失敗 | 「資料驗證失敗：{欄位名稱}」 | 「請修正標紅欄位後重新提交」 | 無 |

---

## 六、實施計畫

### 6.1 階段一：基礎建設（第 1-2 週）

| 任務 | 說明 | 交付物 | 驗收標準 |
|------|------|--------|---------|
| T1 | 建立 `platform/src/bankpay/` 模組，包含資料模型與狀態機 | `bankpay.rs`, `bankpay_state.rs` | cargo build 通過 |
| T2 | 實作 `audit_log` 表的 BankPay 事件寫入 | 修改 `audit_log` 寫入邏輯 | 每筆 BankPay 作業產生 audit_log 記錄 |
| T3 | 實作 `/api/bankpay/export` 端點 | API 端點 + 測試 | 200 OK 返回批次資訊 |
| T4 | 實作 `/api/bankpay/import` 端點 | API 端點 + 測試 | 200 OK 返回匹配結果 |
| T5 | 實作 `/api/bankpay/query` 端點 | API 端點 + 測試 | 200 OK 返回交易狀態 |
| T6 | 實作 `/api/bankpay/config` 端點 | API 端點 + 測試 | 可讀寫 E.Sun 連線設定 |
| T7 | 添加銀行支付測試資料（種子資料） | `seed_data.py` 腳本 | 4 個測試用戶，5 筆交易 |

### 6.2 階段二：界面優化（第 3-4 週）

| 任務 | 說明 | 交付物 | 驗收標準 |
|------|------|--------|---------|
| T8 | 從導航欄移除 `#states` 錨點連結 | `index.html` 修改 | 導航欄不再顯示「介面狀態」 |
| T9 | 實作 `/api/status` JSON 端點 | API 端點 + 測試 | 返回 JSON 狀態物件 |
| T10 | 新增全域狀態列組件 | `index.html` 模板修改 | 所有頁面頂部顯示狀態列 |
| T11 | 新增狀態詳情面板 | `index.html` 模板 + JS（或純 CSS :target） | 點擊「查看詳情」展開面板 |
| T12 | 實作各狀態的視覺規範與錯誤處理 | CSS 設計系統更新 | 各狀態有明確視覺區分 |
| T13 | 銀行支付 UI 區塊（儀表板卡片） | `index.html` 修改 | 儀表板顯示銀行支付狀態卡片 |

### 6.3 階段三：整合驗證（第 5 週）

| 任務 | 說明 | 交付物 | 驗收標準 |
|------|------|--------|---------|
| T14 | 全端整合測試（平台 UI + BankPay API + 資料庫） | 整合測試腳本 | 所有端點 200 OK，資料一致 |
| T15 | 安全性審查（RBAC、CSRF、輸入驗證） | 安全檢查清單 | 未授權存取返回 401/403 |
| T16 | 效能測試（10,000 筆交易 P95 < 500ms） | 效能報告 | P95 < 500ms SLA 達成 |
| T17 | 使用者驗收測試（4 種角色各測試一次）| UTS 報告 | 4 種角色皆可正常操作 |
| T18 | 文件更新（API 文件、使用者手冊） | 文件檔案 | 文件與實作一致 |

---

## 七、程式碼變更清單

### 7.1 新增檔案

| 路徑 | 說明 |
|------|------|
| `platform/src/bankpay/mod.rs` | BankPay 模組入口 |
| `platform/src/bankpay/model.rs` | 資料模型（BankPayBatch, BankPayAudit） |
| `platform/src/bankpay/state.rs` | 狀態機邏輯 |
| `platform/src/bankpay/client.rs` | E.Sun API 客戶端 |
| `platform/src/bankpay/api.rs` | API 端點定義 |
| `platform/src/status.rs` | 系統狀態監控模組 |
| `platform/templates/bankpay.html` | 銀行支付頁面模板 |
| `platform/templates/status.html` | 狀態詳情面板模板（純 CSS :target 版本） |

### 7.2 修改檔案

| 路徑 | 說明 |
|------|------|
| `platform/src/api.rs` | 新增 BankPay 路由註冊、Status 路由註冊、AppState 新增 status 欄位 |
| `platform/src/lib.rs` | 新增 `pub mod bankpay;` 與 `pub mod status;` |
| `platform/templates/index.html` | 移除 `#states` 導航連結、新增全域狀態列、新增銀行支付卡片、新增審計軌跡區塊 |
| `platform/src/auth.rs` | 新增 `perm-bankpay-*` 權限檢查邏輯 |
| `platform/migrations/0006_bankpay.sql` | 新增 `bankpay_batches` 與 `bankpay_audits` 資料表 |
| `platform/migrations/0007_status_indexes.sql` | 新增狀態查詢索引 |

### 7.3 刪除檔案

| 路徑 | 說明 |
|------|------|
|（無）| 不刪除任何既有檔案，僅新增與修改 |

---

## 八、測試計畫

### 8.1 單元測試

| 測試案例 | 說明 | 預期結果 |
|---------|------|---------|
| `bankpay_state_transition` | 狀態機轉換測試 | PENDING→SUBMITTED→COMPLETED 正確 |
| `bankpay_export_validation` | 匯出請求驗證 | 欠缺欄位返回 400 |
| `bankpay_rbac_check` | 權限檢查 | 無 LEDGER_WRITE 返回 403 |
| `status_endpoint` | /api/status 端點測試 | 返回 JSON，所有欄位正確 |
| `status_bar_render` | 狀態列渲染測試 | 異常狀態顯示紅色圖標 |

### 8.2 整合測試

| 測試案例 | 說明 | 預期結果 |
|---------|------|---------|
| `bankpay_full_flow` | 匯出→查詢→匯入完整流程 | 批次狀態正確遞轉 |
| `audit_log_integrity` | 稽核記錄完整性 | 每筆 BankPay 作業對應 1 筆 audit_log |
| `status_bar_persistence` | 頁面切換後狀態列保持 | 全域狀態列在所有頁面顯示一致 |
| `unauthorized_access` | 未授權存取 BankPay API | 返回 401 |

### 8.3 效能測試

| 測試案例 | 說明 | 預期結果 |
|---------|------|---------|
| `bankpay_export_10k` | 10,000 筆交易匯出 | P95 < 500ms |
| `status_endpoint_concurrent` | 並發 100 次 /api/status | 所有回應 < 100ms |
| `dashboard_load` | 儀表板頁面載入 | 首屏渲染 < 200ms |

---

## 九、驗收標準

### 9.1 功能驗收

- [ ] 管理員可從平台儀表板查看銀行支付狀態卡片
- [ ] 管理員可透過平台 UI 發起 BankPay 匯出作業
- [ ] 管理員可透過平台 UI 匯入銀行回應檔
- [ ] 管理員可透過平台 UI 查詢銀行交易狀態
- [ ] 所有 BankPay 作業均寫入 `audit_log` 稽核表
- [ ] 未登入用戶無法存取任何 BankPay API 端點
- [ ] 無 `LEDGER_WRITE` 權限用戶無法執行匯出/匯入

### 9.2 界面驗收

- [ ] 導航欄不再顯示「介面狀態」連結
- [ ] 所有頁面頂部顯示全域狀態列
- [ ] 狀態列顯示資料庫、搜尋、同步、AI、銀行支付模組狀態
- [ ] 點擊「查看詳情」展開狀態詳情面板
- [ ] 異常狀態顯示紅色圖標與處理建議
- [ ] 狀態列在頁面滾動時固定頂部

### 9.3 效能驗收

- [ ] `/api/status` 端點回應時間 < 100ms
- [ ] BankPay 匯出 API 回應時間 P95 < 500ms
- [ ] 儀表板首屏渲染時間 < 200ms
- [ ] 10,000 筆交易壓力測試 P95 < 500ms

---

## 十、風險與緩解

| 風險 | 發生機率 | 影響程度 | 緩解措施 |
|------|---------|---------|---------|
| E.Sun API 變更導致整合失敗 | 中 | 高 | 抽象化銀行客戶端介面，支援多版本適配器 |
| 銀行支付資料與平台帳本不一致 | 低 | 高 | 每筆 BankPay 作業雙重寫入（平台 DB + 銀行回應），定期對帳 |
| 狀態列影響頁面載入效能 | 低 | 低 | 狀態列採用非同步載入，首屏不阻塞 |
| 使用者混淆狀態列與功能選單 | 中 | 中 | 狀態列使用明顯區別的視覺樣式（橫條 vs 選單項目），文字標記為「系統狀態」而非功能名稱 |

---

## 十一、後續擴展建議

1. **多銀行支援**：將 E.Sun BankPay 抽象化為 `BankPayProvider` trait，未來可擴展至其他銀行（玉山、台新、凱基等）
2. **即時通知**：整合 WebSocket 或 Server-Sent Events，銀行交易狀態變更即時推送至前端
3. **自動對帳**：定期自動比對平台帳本與銀行對帳單，自動標記異常項目
4. **行動端介面**：為銀行支付模組建立行動端友善的介面（響應式設計）

---

## 十二、參考文件

| 檔案 | 說明 |
|------|------|
| `.kilo/plans/1793782615513-wellfit-unified-plan-v2.md` | Wellfit 統一整合計畫（單一事實來源） |
| `platform/src/auth.rs` | 認證與授權實作（RBAC、Session） |
| `platform/migrations/0001_baseline_schema.sql` | 基礎資料庫 schema（audit_log 表定義） |
| `platform/templates/index.html` | 主工作台 HTML 模板 |
| `platform/templates/design-system.css` | 設計系統樣式 |

---

## 附錄 A：實作裁決記錄（2026-11-09，依 §0.5 判定框架）

> 本附錄為實作期決策的 SSOT 更新（依本文件末尾「變更須先更新本文件」條款）。詳細論證見 `docs/adr/ADR-0012-bankpay-integration-boundary.md`。

| # | 裁決 | 觸發證據 | 處置 | 驗證 |
|---|------|---------|------|------|
| R1 | 移除 `bankpay` feature gate（原 `default=[]` 使功能在 CI 完全未編譯/未測試/未註冊） | `cargo test --locked`（預設 features）不含 bankpay 模組；路由在預設建置不存在 | 模組無條件編譯；`time`→既有 `chrono`、`reqwest/multipart` 不再需要 → **依賴零新增**，保留原閘門意圖 | `cargo test --locked -p platform` 124 passed |
| R2 | 生產模式 fail-closed：不自創 `/v1/payment/*` 端點 | 原實作捏造端點/欄位，與 `Wellfit/crates/wellfit_bankpay` V2.8 契約語意漂移（HP-04）；`esun_url` 預設值為未經驗證猜測 | 模擬器確定性化（`ESUN-MOCK-{batch_id}`）；生產模式回 500＋權威實作指引 | 單元測試 `test_production_mode_fails_closed` |
| R3 | mock 不虛構財務數值 | 原實作寫死 `matched:5`、`amount_minor:50000`（P7） | 匯入真實解析 CSV；查詢金額/幣別由 `bankpay_batches` 補入 | 整合測試 `matched:2/mismatched:1` |
| R4 | 冪等：`batch_id` 主鍵唯一 + 重放不重複呼叫銀行 | 原實作無任何持久化；重複提交=重複付款風險（§5.1.4 未實作） | 遷移 0006 `bankpay_batches`/`bankpay_audits`；`ON CONFLICT DO NOTHING` + 已提交批次回傳既有狀態 | 整合測試 `bankpay_export_is_idempotent_and_audited` |
| R5 | 稽核 fail-closed | 原實作 6 個稽核寫入點中 5 個 bind 數錯誤（執行期必敗、被 `let _ =` 吞掉） | 統一 `write_audit`（3 `?` 對 3 bind）；稽核失敗回 500（配合冪等重放安全） | 整合測試斷言 audit 列存在且 actor 簽署 |
| R6 | CSRF（T-08）：JSON POST 驗 `X-CSRF-Token` | 原實作對資金端點無 CSRF；threat model 明定每個變更 POST 須驗證 | `verify_csrf` 輔助（同步器權杖）＋測試斷言拒絕時不留批次列 | 整合測試 `bankpay_export_requires_csrf_token` |
| R7 | API 401 中間件 + 修復 cross_module 未認證漏洞 | `cross_module_api` 在 `route_layer(require_session)` 後 `merge` → 稽核/匯出/COM/AI 端點無需登入 | 新增 `auth::require_session_api`；`protected_api` 群組先 merge 後掛層 | 整合測試 `bankpay_api_requires_session`（含 `/api/audit`） |
| R8 | 匯入路徑白名單（CWE-22） | 原實作讀取任意伺服器 `file_path`（任意檔讀→外洩） | 僅接受 `data/bankpay/inbound/*.csv`（canonicalize 前綴比對） | 整合測試 `bankpay_import_rejects_path_traversal` |
| R9 | 狀態探測 derive 不快取 | 原實作快取 Unknown → `overall` 恆 `degraded`；儀表板硬編碼「AI 3 模型」 | `status::probe_modules` 即時探測；sync/com/ai 誠實標 Idle；AI 模型數 0 | 整合測試 `status_endpoint_shape_and_mock_warning` |
| R10 | UI（T8/T10/T11/T13） | 計畫 §六 | 導航移除 `/states`（路由保留）；sticky 全域狀態列 + `:target` 詳情面板；BankPay 卡片（模擬模式徽章）；稽核篩選改真實 GET 表單（原為需 JS 的死控制） | 整合測試 `index_renders_bankpay_card_and_status_bar` |
| R11 | 雙作品比較與整合：Tauri 桌面版（`web/src-tauri`，Enterprise Bank Integration Frontend）↔ 平台 Web 伺服器 | 桌面版 `bankpay_get_api_status()` 含 8 個 V2.8 API 紀錄、連線設定模型（mTLS 三件套/外部沙盒 URL/逾時）、連線檢查/安全檢查/Release 檢查清單——但**全部為寫死 mock 字串**（無實際行為，使用者實測「沒什麼反應」）；伺服器版為真實實作（P7 反模擬） | 判定：**伺服器版為單一事實來源與主力**；萃取桌面版資產入伺服器版——(1) `EsunConfig` 擴充 5 連線欄位（`esun_external_url`/`client_cert_path`/`client_key_path`/`ca_cert_path`/`timeout_secs`，erp_config 10 鍵持久化）；(2) `/bankpay` 新增**誠實連線就緒檢查卡**（V2.8 信封契約、910000、Source-System 標頭、msgId 格式、mTLS 要件——由實際設定狀態推導，非捏造檢查結果）；(3) API 中樞目錄收錄 8 個 V2.8 官方端點＋對應狀態（模擬對應 3/未串接 5）。桌面版未來定位：**平台的桌面殼**（對接 `http://127.0.0.1:8090` API），其 UI 概念持續反向萃取 | Live smoke：就緒卡渲染✓、10 鍵持久化✓、設定 PRG✓；`cargo test -p platform` bankpay 17 unit + 15 integration ✓ |

**後續切片（本計畫未完結項）**：T14 效能基準（10k P95）、T16 UAT 四角色、真實通道啟用（mTLS 憑證 + V2.8 契約一致性測試 pin，使用就緒檢查卡為啟用閘門）、`bankpay_audits` 寫入接點（目前稽核主寫 `audit_log`；`bankpay_audits` 表已建，供對帳明細用）、桌面殼（Tauri）對接平台 API 的整合切片。

---

**本計畫文件為單一事實來源（SSOT）。實作過程中如需變更設計，必須先更新本文件並取得使用者確認。**
