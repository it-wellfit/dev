# 效能基線 v1.0（G4.7 / D2-11）

> 日期：2026-11-13｜方法：`cargo test --locked -p platform --test perf_baseline -- --nocapture`（in-memory SQLite、200 份索引文件、1 次暖機＋50 樣本、p50/p95）。
> 證據紀律（C-10）：未量測項目標 **UNKNOWN**，不得視為 PASS。

## 量測結果（本機，2026-11-13）

| 指標 | p50 | p95 | 預算（PERF-BUDGET） | 判定 |
|---|---|---|---|---|
| `/api/search`（BM25-only；embedder 不可達） | 2.55 ms | 3.20 ms | ≤200 ms（BM25-only） | ✅ 遠低於預算 |
| 工作台 TTFB（`/`） | 3.61 ms | 4.83 ms | ≤200 ms | ✅ 遠低於預算 |

## 未量測（UNKNOWN）

| 指標 | 原因 | 後續 |
|---|---|---|
| `/api/search`（向量/RRF 混合） | 此環境無 embedder（OVH bge-m3 不可達）；向量腿需 `via="hybrid"` 才有意義 | 於有 embedder 之環境重跑，補紀錄 |
| 1 萬份索引重建（`add_documents_bulk`） | 需大量文件與向量；本次僅 200 份 | 後續以 1 萬份重測 |
| 增量建置時間 / 二進位大小 | 非本測試範圍 | 由 CI 記錄 |

## 重跑方式

```powershell
cargo test --locked -p platform --test perf_baseline -- --nocapture
```

## 注意

- 測試以 `assert p95 < 2_000ms` 作為 CI 防呆（寬鬆）；真正的預算判定以本表為準。
- 數值受機器負載影響；每次發布前重跑並更新本表（附日期）。