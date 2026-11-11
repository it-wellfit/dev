# FlapsErp 效能基線報告 v1.0

> 報告編號：PERF-2026-1103-001｜日期：2026-11-03｜狀態：Baseline（首次量測）
> 量測工具：`Wellfit/scripts/perf/flaps-perf.ps1`（T-101）
> 量測環境：Windows 11, FlapsErp v0.0.4 (ia32), 11 行程常駐
> 原始資料：`Wellfit/artifacts/perf/20261103-160315/`

---

## 1. 量測環境

| 項目 | 值 |
|---|---|
| OS | Windows 11 (Build 26100) |
| FlapsErp 版本 | v0.0.4 (2025-04-29) |
| 打包架構 | ia32 (32-bit) |
| 安裝檔 SHA256 | DC77C024629629149842714B8F599565DB04121AF043F5517450BAA5F0AE79B6 |
| Electron/Chromium | 未確認（v0.0.4 2025-04 建置） |
| Node.js | 未確認 |
| 量測時間 | 2026-11-03 16:03 |
| FlapsErp 狀態 | 執行中（11 行程） |

---

## 2. 記憶體基線

### 2.1 全系統 RAM

| 指標 | 數值 |
|---|---|
| 總行程數 | 11 |
| 總 RAM (WorkingSet64) | **623.8 MB** |
| 中位數單行程 RAM | 46.2 MB |
| 最小單行程 RAM | 15.3 MB |
| 最大單行程 RAM | 103.9 MB |
| 32-bit 位址空間使用率 | ~31% (623.8 / 2048 MB) |

### 2.2 行程分布（推導）

| 行程類型 | 推估數量 | 推估 RAM | 說明 |
|---|---|---|---|
| Main process (Electron) | 1 | ~100 MB | Node.js + Chromium 主進程 |
| Renderer (webview) | 3-5 | 40-100 MB each | 每個分頁一個渲染進程 |
| GPU process | 1 | ~30 MB | Chromium GPU 加速 |
| Utility processes | 2-3 | 15-30 MB each | 網路、音訊等 |
| COM/DLL host | 1-2 | 15-40 MB each | POS/BHT 元件 |

### 2.3 記憶體風險評估

| 風險 | 評估 | 證據 |
|---|---|---|
| 32-bit OOM | **中風險** | 623.8MB / 2048MB = 31% 使用率；大報表/大批次可能撞牆 |
| 記憶體洩漏 | **未量測** | 需 24h 長時間採樣（T-101 後續） |
| GC 停頓 | **未量測** | 需 ETW 追蹤（T-101 後續） |

---

## 3. COM 呼叫延遲基線

### 3.1 量測方法

使用 `Scripting.FileSystemObject.GetSpecialFolder(2)` 作為唯讀 COM 呼叫的代理（proxy），量測 1000 次呼叫的延遲分布。此方法建立 COM 每呼叫延遲的**下限（floor）**，實際 FlapsErp 的 COM 呼叫（winax sendSync）會在此基礎上疊加 IPC 序列化、參數編組、結果反序列化等開銷。

### 3.2 結果

| 指標 | 數值 |
|---|---|
| 迴路數 | 1000 |
| P50 | **0.2636 ms** |
| P95 | **0.3884 ms** |
| P99 | **0.4740 ms** |
| 平均值 | **0.3122 ms** |
| 最小值 | ~0.15 ms |
| 最大值 | ~0.8 ms |

### 3.3 批次化效益估算

| 場景 | 逐筆延遲 | 批次化延遲（估算） | 效益 |
|---|---|---|---|
| 100 次 COM 呼叫 | 100 × 0.26 = 26 ms | 1 × 0.4 + 100 × 0.02 = 2.4 ms | **10.8x** |
| 1000 次 COM 呼叫 | 1000 × 0.26 = 260 ms | 1 × 0.4 + 1000 × 0.02 = 20.4 ms | **12.7x** |
| 10000 次 COM 呼叫 | 10000 × 0.26 = 2600 ms | 40 × 0.4 + 10000 × 0.02 = 216 ms | **12.0x** |

> 註：批次化延遲為 mock 估算（batch_round_trip=0.4ms, batch_per_item=0.02ms），實際效益取決於 COM 物件的實際呼叫開銷。flaps_bridge PoC 已驗證批次化模式在 mock 環境下確實勝過逐筆呼叫（`batching_beats_per_call_under_measured_envelope` test）。

### 3.4 風險評估

| 風險 | 評估 | 證據 |
|---|---|---|
| UI 凍結（sendSync） | **高風險** | 1000 次 COM 呼叫 = 260ms 阻塞；批次化可降至 20ms |
| 記憶體壓力 | **中風險** | 批次化減少 IPC 緩衝區分配 |
| 執行緒競爭 | **未量測** | 需 async_ipc 實作後量測 |

---

## 4. CDP 通道基線

### 4.1 量測結果

| 項目 | 數值 |
|---|---|
| FlapsErp 執行中 | True（11 行程） |
| 埠 9222 `/json/list` | **NOT-FOUND** |
| 原因 | FlapsErp 未帶 `--remote-debugging-port` 啟動 |

### 4.2 結論

CDP 通道在預設啟動下不可用。需在 11/10 偵察 session 中驗證帶埠啟動的可行性（T-201 §15-35 min）。

**退讓鏈**：CDP NOT-FOUND → COM（A2）→ 檔案通道（A1，永不阻塞）。

---

## 5. config.ini 基線

### 5.1 當前配置

| 鍵 | 值 | 說明 |
|---|---|---|
| IGNORE_CERT_ERR | 1 | ⚠ TLS 憑證忽略（MITM 風險） |
| KEEP_MACHINE_ID | 1 | 授權綁定 |
| DISABLE_HARDWARE_ACCELERATION | 0 | 硬體加速開啟 |
| DEBUG_FASTREPORT | 0 | FastReport 除錯關閉 |
| FORCE_DEVICE_SACLE_FACTOR | 0 | 裝置縮放因子（拼字錯誤） |
| LANG | zh_TW | 繁體中文 |
| KEEP_MACHINE_ID_MIGRATED | 1 | 授權遷移完成 |

### 5.2 A/B 調優計畫（P1）

| 參數 | 實驗 | 目標 |
|---|---|---|
| DISABLE_HARDWARE_ACCELERATION | 0 vs 1（A/B） | 測量 RAM/CPU 差異（含系統級 CPU） |
| IGNORE_CERT_ERR | 1 → 0（測試 AP 已實測 CERT-VALID；生產 AP 為 HTTP） | 安全性提升；先盤點其他 HTTPS 端點 |
| FORCE_DEVICE_SACLE_FACTOR | 0 vs 1（高 DPI 環境） | UI 渲染正確性 |

> 2026-11-11 複測：測試 AP `test2.flaps.tw:10113` 憑證有效（Let's Encrypt；TLS 1.2；到期 2027-01-21）；預設生產 AP 為 HTTP，故 `IGNORE_CERT_ERR` 對生產無作用。

---

## 6. Wellfit 批次處理基線

### 6.1 MTRIX 管線實測

| 指標 | 數值 |
|---|---|
| 首跑耗時 | 98 ms |
| 冪等重跑耗時 | 18 ms |
| 檔案數 | 5 |
| 列數 | 25（5 檔 × 5 列） |
| 修復數 | 37 |
| Finding 數 | 51（22 errors + 29 warnings） |
| 退出碼 | 0（clean） |
| 冪等重跑退出碼 | 1（duplicate batch 正確拒絕） |

### 6.2 容量目標（建議，待基準實測）

| 指標 | 目標 | 狀態 |
|---|---|---|
| 單批次 ≥ 10,000 列 | < 60s 驗證+修復 | ⚠ 待合成容量基準實測 |
| 日處理 ≥ 200 筆付款 | < 5 min | ⚠ 待 Wellfit BankPay 實測 |
| 月結對帳差異率 | ≤ 0.5% | ⚠ 待 T-203 月結試行 |

---

## 7. 回滾基準

### 7.1 Config 快照

| 檔案 | SHA256 |
|---|---|
| config.ini | 24D07ABF7458C1ABB8F85C645C7CFA96C5C675DC16E36573F2C42A3C0EE0C1C6 |
| hardwareId.txt | 52D909BE3B69E56197F58BCFE5EE0622A157D6258296723BC9469279ACE8EDDC |
| machineId.txt | AAF20E774E5FEC202744B574E75ED63C6C376814169DDD19ECF281E3FFBF390A |
| softwareId.txt | D5E6B4350C1EBC0F049B0D69AD71668EFFFB716D525B42090C4C1D38567CA453 |
| Winax.node.ver | 1D463FE1351BA63B2A073D98F63B919609C7C0E82718A010EE21977F8990F6D8 |
| POSMachineController.CD8240.jsc.ver | 1D463FE1351BA63B2A073D98F63B919609C7C0E82718A010EE21977F8990F6D8 |

### 7.2 回滾程序

1. 從 `Wellfit/artifacts/perf/20261103-160315/config-snapshot/` 還原所有檔案
2. 逐檔 SHA256 比對（manifest 驗證）
3. 重啟 FlapsErp 驗證功能正常

### 7.3 2026-11-11 複測補充

| 項目 | 實測 | 工具 |
|---|---|---|
| AP 預設 | `8E2EDBE…`（大前集團 WELLFIT）HTTP `210.64.124.59:10578` | `-Probe settings` |
| 測試 AP TLS | `test2.flaps.tw:10113` CERT-VALID（Let's Encrypt；TLS 1.2；到期 2027-01-21） | `-Probe tls` |
| 螢幕／視窗 | 1536×864（≈125%）／1536×776 | `-Probe app` |
| PE LAA | i386；LAA=True（`0x0122`） | `-Probe pe` |
| module→menu | 17.087 s（單次日誌） | `-Probe app` |
| config 漂移 | `8E2EDBE…version.txt` 與 2026-11-03 manifest 不同（AP 版本更新） | `-Probe settings` |

---

## 8. 後續量測計畫

| 量測 | 頻率 | 工具 | 目標 |
|---|---|---|---|
| RAM 採樣 | 每日 3 次（09:00/14:00/18:00） | flaps-perf.ps1 -Probe baseline | 趨勢追蹤 |
| COM 延遲 | 每週 1 次 | flaps-perf.ps1 -Probe com | 批次化前後對比 |
| CDP 埠 | 每次 FlapsErp 更新後 | flaps-perf.ps1 -Probe cdp | 通道可用性 |
| app log | 每日 | flaps-perf.ps1 -Probe app | menu ready／tab 事件 |
| settings/TLS/PE | 每季／更新後 | flaps-perf.ps1 -Probe settings/tls/pe | 漂移／憑證／LAA |
| 全量 | 每次基線 | flaps-perf.ps1 -Probe full | 一鍵證據 |
| config 變更 | 每次修改前 | SHA256 manifest | 回滾基準 |

---

## 9. 修訂紀錄

| 版本 | 日期 | 變更 |
|---|---|---|
| v1.0 | 2026-11-03 | 初版：記憶體基線、COM 延遲基線、CDP 通道、config 基線、Wellfit 批次基線、回滾基準、後續計畫 |
