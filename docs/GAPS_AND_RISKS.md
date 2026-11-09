# 系統認知存證、資訊缺口與演進風險報告
## Epistemic Evidence, Information Gaps & Risk Matrix

> **文件路徑**: `docs/GAPS_AND_RISKS.md`
> **所屬體系**: Evidence Assurance（認知標籤見 docs/README.md §3）
> **當前狀態**: Active｜**更新時間**: 2026-11-07

---

## 🔍 1. 已驗證代碼實相清單 `[FACT:CODE]`

| 實相項目 | 具體代碼/檔案依據 | 驗證結論與影響 |
| :--- | :--- | :--- |
| **Rust 2021 Workspace 結構** | `Cargo.toml#L1-L8` | members：`core`, `platform`, `web`, `crates/flaps_bridge`, `crates/kem`。 |
| **Tokio 異步與 Axum 服務** | `Cargo.toml#L12-L13` | `axum 0.7` + `tokio 1.0 (full)` 微服務底座。 |
| **Sqlx SQLite 持久化** | `Cargo.toml#L16` | `sqlx 0.8`（runtime-tokio, macros, sqlite）。 |
| **Windows COM RPA 支援** | `Cargo.toml#L28` | `windows 0.58`（Win32_Foundation & Win32_System_Com）原生桌面自動化。 |
| **Tauri 2.0 桌面集成** | `Cargo.toml#L19` | 客戶端跨平台桌面殼。 |
| **發布編譯最佳化** | `Cargo.toml#L36-L42` | `opt-level = 3`, `lto = true`。 |
| **測試基線 157/0** | `cargo test --locked --workspace`（2026-11-07） | bridge 7＋core 42＋kem 53＋platform 48＋close_out 4＋slice12 3，全綠。 |
| **效能 SLA 成立** | `platform/src/api.rs` stress 測試 | 100k 交易 warm p95 422ms ≤ 500ms（預熱趟＋21 樣本量測法）。 |
| **路由 SSOT 23 條** | `platform/src/api.rs` `.route(` 計數 | 新增路由必同步 docs/design/INFORMATION_ARCHITECTURE.md。 |
| **編碼根因已修** | `dc414ac` 三方合併＋`encoding-standard.md` | api.rs/config.rs/kem main.rs PUA 清零；防制規範就位。 |
| **突變測試就位** | `.cargo/mutants.toml`＋MUTATION_SCORE_REPORT v2 | money 73/engine 22 viable，得分 ≥94.7%，mutants.out/ 已 gitignore。 |

---

## 🧩 2. 關鍵資訊缺口與阻塞清單 `[GAP]`

| ID | 缺口 | 問題 | 解鎖所需證據 | 狀態 |
| :--- | :--- | :--- | :--- | :--- |
| GAP-01 | 輔翼 ERP 版本與 Session 復用 | 背景無頭操作或必須桌面解鎖？ | 輔翼 COM GUID / WindowClass 調用片段 | OPEN（環境受限） |
| GAP-02 | 蝦皮授權機制 | 官方 Open API（OAuth2）或報表下載？ | 蝦皮認證模組或設定檔範本 | OPEN（由 Wellfit 線承接） |
| GAP-03 | MTRIX 56 欄勾稽法定定義 | 勾稽失敗容錯與人工介入流程 | 勾稽演算法規格或測試案例 | OPEN |
| GAP-04 | fuzz 執行環境 | Windows ASan DLL 缺失（STATUS_DLL_NOT_FOUND 0xc0000135）；build 成功、run 受阻 | clang_rt.asan DLL 部署或 libFuzzer-only 模式 | OPEN（工具鏈） |
| GAP-05 | ATDD 綁定缺口 ×2 | rate_limiter 鎖定與 health degraded 未有直接綁定測試 | 補 2 個 Rust 測試並更新 ATDD_BDD_BIND_AUDIT | OPEN（本週可解） |
| GAP-06 | 三大工作室計畫 stub | PLAN 1792865422970 僅目錄無內文 | §0–§24 補實 | IN PROGRESS |
| GAP-07 | 廠商 OQ-F1/F2 回覆 | config.ini A/B、COM 通道、8h soak 不可驗證 | 廠商環境 | OPEN（UNKNOWN≠PASS） |
| GAP-08 | worktree 分歧 | core/Cargo.toml、money.rs、Cargo.lock 與主線分歧，阻 R1 模組化 | 分歧比對與併入決策 | OPEN |

---

## ⚠️ 3. 系統級潛在風險矩陣 (Risk Matrix)

| 風險項目 | 觸發條件 | 潛在影響 | 建議防禦策略 (SOTA) | 狀態 |
| :--- | :--- | :--- | :--- | :--- |
| **Windows 螢幕保護/鎖定阻斷** | 凌晨排程執行時鎖屏 | Win32 GUI 失焦，RPA 全面失敗 | Session 0 隔離（Windows Service）或桌面保活守護 | 已知，維運手冊 |
| **SQLite WAL 膨脹** | 長時間讀寫未 checkpoint | 查詢延遲飆升 | 排程 `PRAGMA wal_checkpoint(TRUNCATE)` | 已知，Runbook |
| **COM 循環引用洩漏** | 未正確 Drop | GDI/User 句柄洩漏 → OS Error 5 | RAII 封裝＋句柄計數測試 | 已知 |
| **CP950 編碼損壞復發** | 工具以非 UTF-8 讀寫 | 原始碼/文件 mojibake（本週根因修復 3 檔） | `encoding-standard.md` 強制 UTF-8 無 BOM；PowerShell 用 `[System.IO.File]::WriteAllText`+UTF8NoBOM | ✅ 已防制 |
| **建置鎖衝突** | 並行 session 的 flaps-server.exe 佔用 target | test/cargo 檔案鎖失敗 | `CARGO_TARGET_DIR` 隔離建置；禁止殺他人行程 | ✅ 已防制 |
| **p95 量測法誤判** | 冷啟動計入小樣本 p95 | 誤報 SLA 違規（本週 535ms 誤判） | 預熱趟＋≥21 樣本；SLA 只規範 warm 穩態 | ✅ 已修正 |
| **並行線同檔衝突** | 兩 session 同改 api.rs/kem | 瞬時編譯錯誤 | board 分工宣告；門檻重跑前 `cargo check` 對齊 | 🟡 監控中 |
