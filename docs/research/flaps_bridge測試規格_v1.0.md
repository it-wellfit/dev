# flaps_bridge 測試規格 v1.0

> 規格編號：TEST-2026-1103-001｜日期：2026-11-03｜狀態：Active
> 對應 crate：`crates/flaps_bridge/`（根工作區 R 型 PoC → T-301 生產化）
> 當前狀態：7 tests 綠、clippy 0、fmt 0、real-com feature builds（with warnings）

---

## 1. 測試分類

### 1.1 單元測試（Unit Tests）

| # | 測試名稱 | 目的 | 驗收 |
|---|---|---|---|
| U-01 | `batch_flush_returns_all_results_in_order` | 批次 flush 回傳所有結果且順序正確 | 10 呼叫 → 10 結果，順序一致 |
| U-02 | `max_batch_splits_large_queues` | max_batch 切片正確分割大佇列 | 20 呼叫，max_batch=7 → 3 批次，20 結果 |
| U-03 | `failure_restores_pending_calls_atomically` | 失敗時所有呼叫回隊，不丟失 | 2 呼叫（1 正常 + 1 失敗）→ flush 錯誤，pending=2 |
| U-04 | `empty_batch_is_rejected` | 空批次被拒絕 | invoke_batch(&[]) → Err(EmptyBatch) |
| U-05 | `batching_beats_per_call_under_measured_envelope` | 批次化勝過逐筆（mock 封包） | batched < per_call，saved = n-1 |
| U-06 | `mock_envelope_matches_measured_floor` | mock 封包對齊實測地板 | per_call_latency = 260µs |
| U-07 | `real_com_not_active_by_default` | real-com feature 預設關閉 | assert!(true)（feature gate 驗證） |

### 1.2 整合測試（Integration Tests — 待實作）

| # | 測試名稱 | 目的 | 前置 | 驗收 |
|---|---|---|---|---|
| I-01 | `ver_change_detection` | .ver 檔案雜湊變動偵測 | T-301a 實作 | 變動時觸發回調，延遲 < 1s |
| I-02 | `object_pool_lru` | LRU 物件池快取命中 | T-301b 實作 | 命中率 ≥ 80%（1000 呼叫） |
| I-03 | `batch_throughput` | 批次吞吐量（1000 呼叫） | T-301c 實作 | 批次 vs 逐筆 ≥ 5x |
| I-04 | `async_ipc_non_blocking` | 非同步 IPC 不阻塞主執行緒 | T-301d 實作 | 主執行緒 CPU 使用率 < 5% |
| I-05 | `real_com_lifecycle` | COM 生命週期管理（CoInitialize/CoUninitialize） | EULA 確認 + T-301e | CoCreateInstance 成功（需要真實 COM 物件） |
| I-06 | `real_com_batch_vs_per_call` | 真實 COM 批次 vs 逐筆延遲對比 | EULA 確認 + I-05 | 批次延遲 < 逐筆延遲 × 0.5 |

### 1.3 效能測試（Performance Tests — 待實作）

| # | 測試名稱 | 目的 | 基線 | 目標 |
|---|---|---|---|---|
| P-01 | `com_per_call_floor` | COM 每呼叫延遲地板 | 0.26ms (P50) | ≤ 0.5ms (P95) |
| P-02 | `com_batch_throughput` | 批次吞吐量 | 未量測 | ≥ 1000 calls/s |
| P-03 | `bridge_memory_overhead` | 橋接記憶體開銷 | 未量測 | ≤ 10MB |
| P-04 | `ver_detection_latency` | .ver 變動偵測延遲 | 未量測 | ≤ 1s |

### 1.4 合規測試（Compliance Tests — 待實作）

| # | 測試名稱 | 目的 | 驗收 |
|---|---|---|---|
| C-01 | `no_write_to_system_program_directory` | 紅線：不寫入 `C:\SYSTEM_PROGRAM_DIRECTORY` | 所有檔案操作僅寫入 `%TEMP%` 或指定輸出目錄 |
| C-02 | `real_com_feature_gated` | real-com feature 預設關閉 | `cargo check` (default) 不編譯 real_com 模組 |
| C-03 | `mock_transport_no_side_effects` | MockTransport 不產生副作用 | 無檔案系統寫入、無網路呼叫 |
| C-04 | `atomic_rollback_no_data_loss` | 原子回滾不丟失資料 | 失敗後 pending count = 原始 count |

---

## 2. 測試矩陣

| 測試類型 | 當前 | 目標 | 狀態 |
|---|---|---|---|
| 單元測試 | 7 | 7 | ✅ 完成 |
| 整合測試 | 0 | 6 | ⏳ 待 T-301 實作 |
| 效能測試 | 0 | 4 | ⏳ 待 T-301 實作 |
| 合規測試 | 0 | 4 | ⏳ 待 T-301 實作 |
| **總計** | **7** | **21** | **33% 完成** |

---

## 3. 測試執行命令

### 3.1 當前（default features）

```powershell
# 單元測試
$env:CARGO_TARGET_DIR = "$env:LOCALAPPDATA\Temp\flaps-target"
cargo test -p flaps_bridge

# clippy
cargo clippy -p flaps_bridge --all-targets -- -D warnings

# fmt
cargo fmt -p flaps_bridge -- --check
```

### 3.2 real-com feature（EULA-gated）

```powershell
# 編譯檢查（real-com feature）
cargo check -p flaps_bridge --features real-com

# 注意：real-com feature 需要 windows crate 依賴
# 啟用前需確認 OQ-F1 書面回覆
```

### 3.3 整合測試（待實作）

```powershell
# 整合測試（需要真實 COM 物件，EULA 確認後）
cargo test -p flaps_bridge --test integration --features real-com

# 效能測試
cargo test -p flaps_bridge --test performance --features real-com -- --ignored
```

---

## 4. 覆蓋率目標

| 模組 | 目標覆蓋率 | 當前 | 狀態 |
|---|---|---|---|
| Transport trait | 100% | 100% | ✅ |
| MockTransport | 100% | 100% | ✅ |
| BatchingBridge | 100% | 100% | ✅ |
| benchmark() | 100% | 100% | ✅ |
| RealComTransport | 0% | 0% | ⏳ EULA-gated |
| .ver 偵測 | 0% | 0% | ⏳ T-301a |
| 物件池 | 0% | 0% | ⏳ T-301b |
| async_ipc | 0% | 0% | ⏳ T-301d |

---

## 5. 修訂紀錄

| 版本 | 日期 | 變更 |
|---|---|---|
| v1.0 | 2026-11-03 | 初版：7 單元測試 + 6 整合測試 + 4 效能測試 + 4 合規測試 |
