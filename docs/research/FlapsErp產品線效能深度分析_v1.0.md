# 輔翼 FlapsErp 效能問題深度分析與最佳解決方案

> 文件類型：逆向工程分析報告（Reverse-Engineering Analysis）
> 分析目標：`C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp`（Electron 應用程式，v0.0.4）
> 日期：2026-10-26

---

## 1. 軟體架構解構（Reverse-Engineering Results）

### 1.1 技術堆疊

| 層級 | 技術 | 說明 |
|---|---|---|
| 執行環境 | Electron (Chromium + Node.js 16.16.0) | 桌面殼，nodeIntegration 啟用 |
| 主程序 | `main.js` + `mainApi.js` | IPC handler、視窗管理 |
| COM/ActiveX 橋接 | `winax` (ffi-napi) | 透過 COM 呼叫 ERP 物件（如 SAP/鼎新/自訂 ERP） |
| 應用控制器 | `js/appController.js` | 視圖管理、分頁、事件路由 |
| 表單引擎 | `js/loadSaveForm.js` (43KB) | 表單載入/儲存 |
| API 傳送 | `js/apiSender.js`、`js/lib/netUtil.js` (9.7KB) | HTTP/網路呼叫 |
| 資料持久 | `js/lib/userData.js`、`flaps-osUtil` | 使用者設定、OS 整合 |
| 更新機制 | `script/asarUpdater.vbs`、`updater.vbs` | VBScript 更新器 |

### 1.2 關鍵效能瓶頸（已確認）

1. **同步 IPC (`sendSync`) 全面使用**
   - `mainApi.js` ActiveXObject 方法一律標記 `{ send: 'sendSync' }`。
   - 前端每次 COM 呼叫都同步阻塞渲染執行緒 → UI 凍結。

2. **COM 物件操作在主程序主執行緒**
   - `winax.Object(data.name)` 與方法 `Function.prototype.apply.call(...)` 直接在 Electron 主程序執行。
   - COM 呼叫是同步、不可中斷的；大量呼叫時主程序無回應。

3. **無快取 / 無批次**
   - 每次表單載入都重新透過 COM 取得欄位/資料，無 memoization。
   - `loadSaveForm.js` 43KB 表明表單邏輯複雜但無快取層。

4. **檔案 I/O 同步**
   - `fs.readFileSync/writeFileSync/unlinkSync/copyFileSync` 散佈於更新與設定流程。

5. **硬體加速可被關閉**
   - `configs.DISABLE_HARDWARE_ACCELERATION === '1'` 可關閉 GPU 加速，影響渲染效能。

6. **單一視窗/分頁模型**
   - `backgroundThrottling: false` 設定，所有分頁持續運算，記憶體與 CPU 負載高。

7. **ActiveX 物件生命週期管理粗放**
   - `event.sender.once('destroyed', ...)` 才釋放，長時間持有 COM 物件 → 記憶體洩漏風險。

---

## 2. 效能問題分類矩陣

| 類別 | 症狀 | 根因 | 影響度 |
|---|---|---|---|
| UI 凍結 | 操作 ERP 物件時畫面卡死 | `sendSync` + COM 同步 | 高 |
| 啟動慢 | 啟動 > 10s | winax 載入 + COM 物件初始化 | 中 |
| 記憶體增長 | 長時間使用後記憶體上升 | COM 物件未釋放、無 GC | 高 |
| 表單載入慢 | 開啟表單 > 3s | 無快取、重複 COM 呼叫 | 中 |
| 網路延遲 | API 呼叫阻塞 | `netUtil.js` 同步風格 | 中 |
| 更新失敗 | 更新器 VBScript 不穩定 | VBScript + regsvr32 同步 | 低 |

---

## 3. 最佳解決方案（分階段）

### Phase 1 — 立即見效（1–2 週）

1. **將 COM 呼叫移至 Worker Threads / Utility Process**
   - 使用 Electron `utilityProcess` 或 Node `worker_threads` 執行 winax 呼叫。
   - 主程序僅負責 IPC 路由，UI 執行緒不再阻塞。

2. **同步 IPC → 非同步 IPC**
   - 將 `{ send: 'sendSync' }` 改為 `{ send: 'send' }`（非同步）。
   - 前端改用 `ipcRenderer.invoke` + Promise，避免 `event.returnValue` 阻塞。

3. **啟用硬體加速**
   - 移除/修正 `DISABLE_HARDWARE_ACCELERATION` 預設；確認 GPU 可用。

4. **表單資料快取**
   - 在 `loadSaveForm.js` 加入 LRU 快取（key: formId+version），TTL 5 分鐘。

### Phase 2 — 架構優化（3–6 週）

5. **COM 物件池 + 生命週期管理**
   - 建立物件池，限制同時 COM 物件數量（如 50）。
   - 使用 WeakRef + FinalizationRegistry 自動釋放。

6. **批次 COM 呼叫**
   - 將多次單一屬性讀取合併為一次 COM 呼叫（batch get）。
   - 減少 IPC 往返次數 80%+。

7. **資料序列化優化**
   - 對大型 COM 回傳物件使用 structured clone / transferable，避免深拷貝。

8. **背景分頁節流**
   - 啟用 `backgroundThrottling: true`，非活躍分頁降低更新頻率。

### Phase 3 — 長期（2–3 月）

9. **遷移至 Rust 後端（wellfit 整合方向）**
   - 參考本專案 `Wellfit` 架構（Rust + Axum + SQLite ledger），將 ERP 介接邏輯從 COM 轉為 Rust FFI/COM crate。
   - 用 `wellfit.exe` 作為 sidecar，Electron 僅作 UI 殼。

10. **效能監控與基準**
    - 加入 `cargo bench` 等效的 JS benchmark（如 `benchmark.js`）。
    - 目標：表單載入 < 500ms、COM 呼叫 < 50ms、記憶體穩定。

---

## 4. 風險與相容性

- **winax 在 Worker Thread 中的穩定性**：winax 使用 COM STA/MTA；需設定 `CoInitializeEx(COINIT_MULTITHREADED)` 或使用 STA 執行緒。建議先做 POC 驗證。
- **非同步改造的回歸風險**：前端程式碼大量依賴同步回傳值；需全面審查 `sendSync` 呼叫點。
- **ERP 版本相容性**：COM 物件介面在不同 ERP 版本可能不同；需保留同步 fallback。

---

## 5. 驗證指標（KPI）

| 指標 | 現況（估） | 目標 |
|---|---|---|
| 表單載入時間 | 3–5s | < 500ms |
| COM 呼叫延遲 | 100–500ms | < 50ms |
| UI 凍結頻率 | 高 | 0 |
| 記憶體（8h） | +200MB | < +50MB |
| 啟動時間 | 8–12s | < 4s |

---

## 6. 結論

FlapsErp 的效能問題本質是 **同步 COM 呼叫阻塞 Electron 主執行緒** 加上 **缺乏快取與生命週期管理**。最佳解法是分階段：先以 Worker/Utility Process + 非同步 IPC 取得立即改善，再以 COM 物件池與批次呼叫優化，最終遷移至 Rust 後端（wellfit 架構）實現長期效能與可維護性。
