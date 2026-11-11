# 輔翼 FlapsErp 效能優化專案 — 完整專案文件與開發前準備報告

> 文件編號：FLAPS-PERF-PRJ-001
> 版本：v1.0
> 日期：2026-10-26
> 對象：主管 / 專案利害關係人
> 狀態：Draft for Approval

---

## 1. 專案目標

### 1.1 主要目標
將 FlapsErp（Electron 桌面應用，v0.0.4）的效能提升至可接受水準，消除 UI 凍結、降低記憶體增長、縮短表單載入與啟動時間。

### 1.2 量化成功指標（KPI）

| 指標 | 現況（估） | 目標 | 測量方式 |
|---|---|---|---|
| 表單載入時間 | 3–5s | < 500ms | Playwright E2E 計時 |
| COM 呼叫延遲 | 100–500ms | < 50ms | 主程序日誌 |
| UI 凍結頻率 | 高（每次 COM 呼叫） | 0 | 使用者回報 + 自動化 |
| 記憶體（8h 連續使用） | +200MB | < +50MB | Electron process memory |
| 啟動時間 | 8–12s | < 4s | 冷啟動計時 |
| COM 物件數量上限 | 無限制 | ≤ 50 | 物件池監控 |

---

## 2. 專案範圍

### 2.1 範圍內（In Scope）
- Electron 主程序與渲染程序的 IPC 架構改造（同步→非同步）。
- COM/ActiveX 呼叫移至 Worker/Utility Process。
- 表單資料快取層（LRU + TTL）。
- COM 物件池與生命週期管理。
- 批次 COM 呼叫合併。
- 效能監控與基準測試工具。
- 長期 Rust 後端遷移可行性評估（Phase 3）。

### 2.2 範圍外（Out of Scope）
- ERP 後端系統（SAP/鼎新等）修改。
- UI 視覺重新設計。
- 新功能開發（僅效能優化）。
- 行動裝置支援。

---

## 3. 需求分析

### 3.1 利害關係人

| 角色 | 需求 | 優先級 |
|---|---|---|
| 終端使用者（ERP 操作員） | 操作不卡頓、快速開啟表單 | 高 |
| IT 管理員 | 穩定、低記憶體、易部署 | 高 |
| 開發團隊 | 可維護、可測試的架構 | 中 |
| 主管 | 可量化成果、可控風險 | 高 |

### 3.2 功能需求（FR）

| 編號 | 需求 | 優先級 |
|---|---|---|
| FR-01 | COM 呼叫不得阻塞 UI 執行緒 | P1 |
| FR-02 | 表單資料支援快取（可設定 TTL） | P1 |
| FR-03 | COM 物件數量受控（物件池） | P2 |
| FR-04 | 多個 COM 屬性讀取支援批次 | P2 |
| FR-05 | 效能指標可被監控與匯出 | P3 |
| FR-06 | 保留同步 fallback 相容性 | P2 |

### 3.3 非功能需求（NFR）

| 編號 | 需求 | 目標 |
|---|---|---|
| NFR-01 | 效能 | 見 KPI 表 |
| NFR-02 | 穩定性 | 8h 連續運行無崩潰 |
| NFR-03 | 相容性 | Windows 10/11、既有 ERP 版本 |
| NFR-04 | 可維護性 | Clippy 零警告等效（JS lint 零警告） |
| NFR-05 | 安全性 | COM 物件不洩漏、無記憶體洩漏 |

---

## 4. 系統分析與設計

### 4.1 系統架構（目標架構）

```
┌─────────────────────────────────────────────┐
│  Renderer Process (UI)                      │
│  - Vue 2.6 / HTML                           │
│  - 非同步 IPC (invoke/handle)               │
└──────────────┬──────────────────────────────┘
               │ IPC (async)
┌──────────────▼──────────────────────────────┐
│  Main Process (Electron)                    │
│  - IPC Router                               │
│  - 物件池管理                               │
│  - 快取層 (LRU)                             │
└──────────────┬──────────────────────────────┘
               │ MessagePort
┌──────────────▼──────────────────────────────┐
│  Utility Process / Worker Thread            │
│  - winax COM 呼叫                           │
│  - 批次執行                                 │
└──────────────┬──────────────────────────────┘
               │ COM/ActiveX
┌──────────────▼──────────────────────────────┐
│  ERP 系統 (COM 物件)                        │
└─────────────────────────────────────────────┘
```

### 4.2 模組劃分

| 模組 | 職責 | 檔案 |
|---|---|---|
| IPC Router | 路由前端請求至對應 handler | `main.js` |
| ActiveX Bridge | COM 物件建立/呼叫/釋放 | `mainApi.js` (ActiveXObject) |
| Object Pool | COM 物件池管理 | 新增 `js/lib/objectPool.js` |
| Cache Layer | 表單資料 LRU 快取 | 新增 `js/lib/formCache.js` |
| Batch Executor | 批次 COM 呼叫合併 | 新增 `js/lib/batchExecutor.js` |
| Performance Monitor | 指標收集與匯出 | 新增 `js/lib/perfMonitor.js` |
| Form Loader | 表單載入/儲存 | `js/loadSaveForm.js` |

### 4.3 資料流程（表單載入範例）

```
使用者開啟表單
  → Renderer 發送 IPC invoke('loadForm', formId)
  → Main Process 檢查 Cache Layer
     → 命中：直接回傳 (< 10ms)
     → 未命中：送至 Utility Process
  → Utility Process 執行 COM 呼叫（批次取得欄位+資料）
  → 回傳結果 → Main Process 寫入快取 → 回傳 Renderer
  → Renderer 渲染表單
```

### 4.4 介面設計

- **IPC 介面**：統一使用 `ipcMain.handle` / `ipcRenderer.invoke`（Promise-based）。
- **COM 介面**：保留既有 `__methods` 探索機制，新增 `batchGet(properties[])` 方法。
- **快取介面**：`cache.get(key)`, `cache.set(key, value, ttl)`, `cache.invalidate(key)`。

### 4.5 技術選型

| 項目 | 選擇 | 理由 |
|---|---|---|
| COM 執行環境 | Electron Utility Process | 官方推薦、隔離、不阻塞主執行緒 |
| 快取 | LRU-Cache (npm) | 輕量、TTL 支援 |
| 監控 | 自訂 perfMonitor + JSON 匯出 | 無需外部依賴 |
| 測試 | Playwright + Jest | 既有 playwright.config.js |
| 長期後端 | Rust (wellfit 架構) | 本專案既有 Rust 能力 |

---

## 5. 技術方案

### 5.1 Phase 1（1–2 週）：立即見效
1. Utility Process 建立（`com-worker.js`），接收 COM 請求。
2. `mainApi.js` ActiveXObject 改為 async，移除 `sendSync`。
3. 前端 `loadSaveForm.js` 改用 `invoke` + Promise。
4. 啟用硬體加速（移除預設關閉）。
5. 表單資料 LRU 快取（TTL 5 min）。

### 5.2 Phase 2（3–6 週）：架構優化
1. COM 物件池（上限 50，WeakRef 自動釋放）。
2. 批次 COM 呼叫（合併屬性讀取）。
3. 結構化複製（transferable）。
4. 背景分頁節流啟用。

### 5.3 Phase 3（2–3 月）：長期遷移
1. Rust COM crate 評估（windows-rs）。
2. wellfit sidecar 整合。
3. 完整效能基準與監控儀表板。

---

## 6. 開發計畫

### 6.1 里程碑

| 里程碑 | 時程 | 交付物 |
|---|---|---|
| M1：架構設計確認 | Week 1 | 本文件核准、POC 計畫 |
| M2：Phase 1 完成 | Week 3 | Utility Process + 非同步 IPC + 快取 |
| M3：Phase 1 驗收 | Week 4 | KPI 達標驗證報告 |
| M4：Phase 2 完成 | Week 8 | 物件池 + 批次 + 節流 |
| M5：Phase 2 驗收 | Week 9 | 全面效能測試報告 |
| M6：Phase 3 評估 | Week 12 | Rust 遷移可行性報告 |

### 6.2 工作分解結構（WBS）

```
1. 需求與設計 (Week 1)
   1.1 需求確認與 KPI 定義
   1.2 架構設計
   1.3 POC 驗證（Utility Process + winax）
2. Phase 1 開發 (Week 2–3)
   2.1 Utility Process 建立
   2.2 IPC 非同步改造
   2.3 快取層實作
   2.4 硬體加速修正
3. Phase 1 測試與驗收 (Week 4)
4. Phase 2 開發 (Week 5–8)
   4.1 物件池
   4.2 批次呼叫
   4.3 節流與序列化
5. Phase 2 測試與驗收 (Week 9)
6. Phase 3 評估 (Week 10–12)
```

### 6.3 資源配置

| 角色 | 人數 | 職責 |
|---|---|---|
| 專案經理 | 1 | 排程、溝通、風險管理 |
| 資深開發工程師 | 2 | 架構設計、核心改造 |
| 測試工程師 | 1 | 效能測試、回歸測試 |
| ERP 領域專家 | 1（兼） | COM 介面驗證 |

### 6.4 溝通機制

- **每週例會**：進度、阻礙、風險。
- **每日 standup**：15 分鐘（開發期間）。
- **里程碑審查**：每個里程碑結束後正式審查。
- **文件更新**：每次變更更新本文件與 CHANGELOG。

---

## 7. 風險評估與因應措施

| 風險 | 可能性 | 影響 | 因應措施 |
|---|---|---|---|
| winax 在 Utility Process 不穩定 | 中 | 高 | 先做 POC；保留同步 fallback |
| 非同步改造回歸 | 高 | 高 | 全面審查 sendSync 呼叫點；自動化測試 |
| ERP 版本相容性 | 中 | 中 | 保留同步 fallback；版本矩陣測試 |
| 時程延遲 | 中 | 中 | Phase 切分、每週檢查點 |
| 記憶體洩漏未解決 | 低 | 高 | WeakRef + FinalizationRegistry；長時間 soak test |

---

## 8. 測試與品質保證策略

### 8.1 測試層級

| 層級 | 工具 | 範圍 |
|---|---|---|
| 單元測試 | Jest | 快取、物件池、批次邏輯 |
| 整合測試 | Playwright | IPC 流程、表單載入 |
| 效能測試 | Playwright + 自訂計時 | KPI 驗證 |
| Soak 測試 | 自動化腳本 | 8h 連續運行記憶體監控 |
| 回歸測試 | Playwright | 既有功能不退化 |

### 8.2 驗收標準（Acceptance Criteria）

- [ ] 所有 P1 功能需求實作完成。
- [ ] KPI 全部達標（見 §1.2）。
- [ ] Soak 測試 8h 無崩潰、記憶體增長 < 50MB。
- [ ] 回歸測試通過率 100%。
- [ ] 文件更新完整（API、架構、操作手冊）。
- [ ] 主管正式核准。

---

## 9. 部署與維運計畫

### 9.1 部署策略
- **漸進式部署**：先內部測試環境 → 少數使用者 → 全量。
- **回滾機制**：保留 v0.0.4 安裝包，問題時一鍵回滾。
- **更新機制**：沿用既有 `asarUpdater.vbs`，但改為非同步下載。

### 9.2 維運監控
- 效能指標匯出至 JSON，定期收集分析。
- 崩潰日誌自動收集（`app.on('render-process-gone')`）。
- 版本追蹤：每次部署記錄版本與日期。

### 9.3 維護計畫
- 每月效能檢視。
- 每季架構審查。
- Phase 3 Rust 遷移評估報告後決定是否執行。

---

## 10. 系統流程最佳化建議

1. **COM 呼叫批次化**：將 `get`/`set` 多屬性合併為單一 COM 呼叫，減少 IPC 往返。
2. **快取失效策略**：表單資料變更時主動 invalidate，避免過期資料。
3. **物件池預熱**：啟動時預建立常用 COM 物件，減少首次呼叫延遲。
4. **記憶體監控告警**：超過閾值（如 500MB）時自動觸發 GC 與物件釋放。
5. **使用者回饋迴圈**：每次版本發布後收集效能回饋，持續迭代。

---

## 11. 附錄

### 11.1 參考文件
- `docs/research/FlapsErp產品線效能深度分析_v1.0.md`（逆向工程分析報告；2026-11-11 RP-6① 遷入）
- `Wellfit/AGENTS.md`（Rust 專案規範）
- `docs/product-solution.md`（產品解決方案）

### 11.2 術語表
- **COM**：Component Object Model，Windows 元件物件模型。
- **winax**：Node.js ActiveX/COM 橋接套件。
- **IPC**：Inter-Process Communication，程序間通訊。
- **LRU**：Least Recently Used，最近最少使用快取策略。
- **Utility Process**：Electron 推薦的背景程序執行方式。

---

*本文件為專案啟動前的完整準備報告，需主管核准後方可進入開發階段。*
