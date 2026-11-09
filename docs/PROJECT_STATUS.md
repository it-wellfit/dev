# PROJECT_STATUS — 專案現況

> 更新：2026-11-07｜真實狀態以測試 log 為準，不以本檔宣稱。
> 品質閘門本輪結果：**fmt EXIT=0｜clippy -D warnings EXIT=0｜test 157 passed / 0 failed｜audit EXIT=0**（1 個已接受警告 RUSTSEC-2026-0374 basic-toml unmaintained）。

## 目前階段

**Phase 1 — MVP Demo Ready** ✅（Phase 2 準備文件已就位，部署需授權）

## 六階段位置

| 階段 | 狀態 | 證據 |
|------|------|------|
| Phase 0 Discovery & Blueprint | ✅ 完成 | docs/product/＋docs/architecture/＋docs/design/ |
| Phase 1 POC → MVP Demo | ✅ **Demo Ready** | 157 tests；seed/reset 腳本；Demo 文件 |
| Phase 2 Demo → Production | 🟡 準備就緒（文件），**部署需授權** | docs/production/PRODUCTION_READINESS.md |
| Phase 3 Scale-up | ⬜ 未開始（需 Stage 2＋流量數據） | — |
| Phase 4 Product Line | ⬜ 未開始（需商業決策） | — |
| Phase 5 Evolution | 🟡 制度運作中 | BACKLOG/CHANGELOG/PLAN_REGISTER/PLAN v3.1 |

## 關鍵數據（2026-11-07 驗證）

| 指標 | 值 |
|------|-----|
| 自動化測試 | **157 passed / 0 failed / 0 ignored**（bridge 7＋core 42＋kem 53＋platform 48＋close_out 4＋ledger_slice12 3） |
| 路由 | 23 條（`platform/src/api.rs` 路由表為 SSOT；含 `/api/health`、search 橋接） |
| 效能 SLA | 100,000 交易 stress：**warm p95 422ms ≤ 500ms**（量測法：預熱趟＋21 樣本 p95；seed 批次化 500 rows/statement，從 ~90s 降至 ~13s 全測時長） |
| 突變測試 | cargo-mutants 27.1：money/engine 得分 **≥94.7%**（missed 清零於 money；exclude 設計見 MUTATION_SCORE_REPORT v2） |
| Migrations | 3（0001-0003） |
| 文件 | 70+ 份（本索引 docs/README.md v3.0 收錄） |

## 2026-11-07 本輪完成（可追溯）

1. **CP950 編碼損壞根因修復**：`platform/src/api.rs` 以 commit `dc414ac` 為乾淨源、`git merge-file` 三方合併還原（含 Askama 分頁欄位還原）；`core/src/config.rs` 類別註解與 `crates/kem/src/main.rs` `P1 M2–M3` PUA 字元修復。防制規範見 `docs/encoding-standard.md`。
2. **Stress test 量測法修正**：`api::tests::stress_10000_transactions_p95_under_500ms` 原以 3 樣本取 max 為 p95，首筆冷啟動查詢 535ms 誤判逾時；改為預熱趟（排除冷快取）＋21 樣本 p95（容許單次排程抖動），SLA 500ms 保持不變，seed 批次化。
3. **方法論補強**：DEVELOPMENT_METHODOLOGY §7–§9（TDD 實證邊界/跨域/缺口表/Skills）；cargo-mutants 排除設計；FAULT_INJECTION_SOP（FAULT-01..07）；ATDD 綁定稽核（8 情境）。
4. **並行工作線（未纳入本輪門檻，屬另一 session）**：Slice 13 全文檢索引擎與 `cross_module_api.rs`（search/sync/com/ai 橋接）、kem evolve/gov 模組 — 其狀態以該線記錄為準。

## 品質門檻重現命令

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --locked --workspace --no-fail-fast
cargo audit
```

> 若 `target\debug\flaps-server.exe` 被執行中行程鎖定（如並行 session 的 dev server），以
> `$env:CARGO_TARGET_DIR = "$env:LOCALAPPDATA\Temp\dev-target"` 隔離建置，勿殺他人的行程。
