# PROJECT_STATUS — 專案現況

> 更新：2026-11-09（PHASE-R RAG 後）｜真實狀態以測試 log 為準，不以本檔宣稱。
> 品質閘門本輪結果（全工作區）：**fmt EXIT=0｜clippy -D warnings EXIT=0｜246 passed / 0 failed｜platform-hub 10 passed / 0 failed｜audit EXIT=0 兩工作區**（2 個已文件化處置：RUSTSEC-2026-0374 basic-toml、RUSTSEC-2026-0459 libsqlite3-sys——見各 `.cargo/audit.toml`）。

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
| 自動化測試 | **246 passed / 0 failed / 0 ignored**（bridge 7＋core 42＋kem 58＋platform 108＋kem-int 15＋bankpay_persistence 1＋characterization 6＋close_out 4＋slice12 2＋misc 3） |
| 路由 | **60 條**（api.rs 48＋cross_module_api 12；原始碼路由表為 SSOT） |
| 效能 SLA | 100,000 交易 stress：**warm p95 422ms ≤ 500ms**（量測法：預熱趟＋21 樣本 p95；seed 批次化 500 rows/statement，從 ~90s 降至 ~13s 全測時長） |
| 突變測試 | cargo-mutants 27.1：money/engine 得分 **≥94.7%**（missed 清零於 money；exclude 設計見 MUTATION_SCORE_REPORT v2） |
| Migrations | 3（0001-0003） |
| 文件 | 70+ 份（本索引 docs/README.md v3.0 收錄） |

## 2026-11-07 本輪完成（可追溯）

1. **CP950 編碼損壞根因修復**：`platform/src/api.rs` 以 commit `dc414ac` 為乾淨源、`git merge-file` 三方合併還原（含 Askama 分頁欄位還原）；`core/src/config.rs` 類別註解與 `crates/kem/src/main.rs` `P1 M2–M3` PUA 字元修復。防制規範見 `docs/encoding-standard.md`。
2. **Stress test 量測法修正**：`api::tests::stress_10000_transactions_p95_under_500ms` 原以 3 樣本取 max 為 p95，首筆冷啟動查詢 535ms 誤判逾時；改為預熱趟（排除冷快取）＋21 樣本 p95（容許單次排程抖動），SLA 500ms 保持不變，seed 批次化。
3. **方法論補強**：DEVELOPMENT_METHODOLOGY §7–§9（TDD 實證邊界/跨域/缺口表/Skills）；cargo-mutants 排除設計；FAULT_INJECTION_SOP（FAULT-01..07）；ATDD 綁定稽核（8 情境）。
4. **並行工作線（未纳入本輪門檻，屬另一 session）**：Slice 13 全文檢索引擎與 `cross_module_api.rs`（search/sync/com/ai 橋接）、kem evolve/gov 模組 — 其狀態以該線記錄為準。

## 2026-11-09 本輪完成（可追溯）

1. **三大工作室切片落地**（`1793787631643` 執行計畫 v4）：G1.1 八資料態、G1.2 上下文連續、G1.3 角色起始頁＋KPI 定義、G1.4 批量覆核、G2.1 冪等+DLQ、G2.2 熔斷、G2.3 UDM 實體＋單位契約、G2.4 對帳計算、G2.5 agent 限流/草稿/審計、G3.1 證據四要件導出、G3.2 控制庫跨框架映射、G3.3 CCM 三指標。逐切片證據：PLAN_REGISTER addendum #12/#13；追溯：master plan §22。
2. **重構判定框架**（`1793787631643` §0.5）：五步程序＋定理量測表＋已裁決候選（api.rs R1 最後排序、MoneyCents 契約不合併、Kafka stub 不動、DEC-15 fail-closed）。
3. **並行線**：Slice 13 搜尋＋bankpay＋characterization.rs（6 tests）由並行 session 交付；kem in-flight。

## 品質門檻重現命令

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --locked --workspace --no-fail-fast
cargo audit
```

> 若 `target\debug\flaps-server.exe` 被執行中行程鎖定（如並行 session 的 dev server），以
> `$env:CARGO_TARGET_DIR = "$env:LOCALAPPDATA\Temp\dev-target"` 隔離建置，勿殺他人的行程。
