# Fuzzing — 模糊測試操作手冊（FBT）

> 級別：ENG-FUZZ-1.0｜日期：2026-11-07｜定位：方法論 §8 FBT 的執行文件（承接 FAULT_INJECTION_SOP FAULT-04）
> 工具：cargo-fuzz 0.13.2 ＋ nightly 1.101.0（libFuzzer）

## 1. 範圍與目標

| 項目 | 內容 |
| :--- | :--- |
| 已有 target | `fuzz/fuzz_targets/money_parser.rs` — `parse_decimal_to_minor_i128` 與 `format_minor_i128` round-trip |
| 不變量 | INV1 解析結果非負；INV2 `format(parse(x)) == x`（正規形式往返一致） |
| 未來 target（P2） | CSV parser 欄位數不變量、日期邊界、report pivot 總額守恆 |

## 2. 目錄與設定事實 `[FACT:CODE]`

- `fuzz/Cargo.toml`：獨立 `[workspace]`（避免污染根 workspace 鎖）；依賴 `flaps_core = { path = "../core" }`。
- target 名稱：`money_parser`（`fuzz_targets/money_parser.rs`）。
- nightly 工具鏈：`cargo +nightly fuzz build money_parser --release` → **build PASS**。

## 3. 執行（目前受阻 `[LIMIT]`）

| 步驟 | 命令 | 狀態 |
| :--- | :--- | :--- |
| 建置 | `cargo +nightly fuzz build money_parser --release` | ✅ PASS |
| 執行 | `cargo +nightly fuzz run money_parser --release -- -max_total_time=60` | ⛔ Windows `STATUS_DLL_NOT_FOUND`（0xc0000135）— ASan runtime DLL 未隨 target 部署 |

解鎖路徑（擇一）：
1. 安裝 LLVM 工具鏈並確保 `clang_rt.asan-*.dll` 位於 target 產出物同目錄或 PATH。
2. 於 `fuzz/Cargo.toml` 移除 ASan（`-Zsanitizer=address`）僅用 libFuzzer 覆蓋率模式（失去記憶體錯誤偵測，仍具輸入解析健壯性價值）。
3. 於 Linux CI 執行（GitHub Actions `ubuntu-latest` 免 DLL 問題）— 建議與制度化候選 #2 合併處理。

## 4. 最小執行迴圈（解鎖後）

```powershell
cargo +nightly fuzz run money_parser --release -- -max_total_time=300 -print_final_stats=1
# crash artifact 位於 fuzz/artifacts/money_parser/
```

任何 crash → 以最小化輸入轉為 `#[test]` 回歸案例（對齊 FAULT_INJECTION_SOP §7 證據留存）。

## 5. 與其他技術的關係

- **MT（突變測試）**：證明測試殺變異能力；**FBT**：探索未預期輸入空間。互補不可互替。
- **PBT**：proptest 在 `flaps_core` 已有 8 性質測試；fuzz 走 nightly 工具鏈與隨機語料，二者執行層不同。
- **ATDD/BDD**：情境綁定見 `ATDD_BDD_BIND_AUDIT.md`。
