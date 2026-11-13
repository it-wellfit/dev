# 疑難排解（Troubleshooting）— Dev 根工作區

- 版本：v1.0｜生效日期：2026-11-12｜狀態：Active
- 格式：每條含 **症狀 → 原因 → 處理**（與 `SHARED_STANDARDS` §8.2 之錯誤訊息要求一致）

## 1. 建置/測試

| 症狀 | 原因 | 處理 |
|---|---|---|
| `cargo test --locked` 失敗於 lock 檔 | manifest 變更後未更新 `Cargo.lock` | 於該 workspace 執行 `cargo metadata`（或 `cargo check`）更新 lock，再重跑 |
| clippy 大量 `needless_borrow` 等 | 新 MSRV 之 lints 生效 | 以 `cargo clippy --fix --allow-dirty` 修正；不得以 `#[allow]` 規避（憲章 §2.2） |
| `cargo machete` 報未用依賴 | 依賴宣告但未引用 | 移除，或於該 Cargo.toml 加 `[package.metadata.cargo-machete] ignored=[...]` 並註明理由 |
| 建置被桌面原則阻擋 | 產物控制 | `$env:CARGO_TARGET_DIR = "$env:LOCALAPPDATA\Temp\dev-target"` |

## 2. 執行/連線

| 症狀 | 原因 | 處理 |
|---|---|---|
| 瀏覽器顯示連線失敗 | platform 未啟動/埠不對 | 先起 `auxbridge-server`；確認 `:8090`；殼會於 6 秒後提示 |
| 收到 403 而非登入頁 | 以 PowerShell 5.1 之 `MaximumRedirection` 量測造成假象 | 以 `curl` 驗證 HTTP 行為（見 corrections 之驗證結論） |
| 桌面殼無日誌 | 無主控台且未設 subscriber | 已修：日誌寫入 `%LOCALAPPDATA%\AuxBridge\logs\auxbridge-web.log`（2026-11-12） |

## 3. 更新鏈

| 症狀 | 原因 | 處理 |
|---|---|---|
| 更新驗簽失敗 `InvalidSignature` | 簽章與執行檔不一致（重編未重簽） | 重新 `cargo tauri build`（帶簽章金鑰）並重產 `latest.json` |
| `A public key has been found, but no private key` | 只設了 `..._PATH` | 改用 `TAURI_SIGNING_PRIVATE_KEY`（金鑰**內容**）＋密碼 |
| feed 404 | 主機未建置 | 目前 BLOCKED：需建 `updates.auxbridge.internal` |

## 4. SBOM/供應鏈

| 症狀 | 原因 | 處理 |
|---|---|---|
| `sbom_freshness` 失敗 | 依賴變更未重產 SBOM | 逐 crate 重產 7 份（見發行手冊 §4）並重跑 |
| `cargo audit` 失敗 | 新公告 | 先評估：可升級即升級；否則於 `.cargo/audit.toml` 寫入理由/緩解/複核條件 |
| SBOM 範圍錯誤（內容相同、檔名不同） | 以 `--manifest-path` 自 repo 根執行 | 以各 crate 目錄為 CWD 執行（`Push-Location`） |

## 5. 已知缺口

- `scripts/backup.ps1`、`scripts/check-docs`、`scripts/check-secrets` 尚未提供（`SHARED_STANDARDS` §8.1／§6.2 引用之腳本）。
  現階段替代：資料庫備份／驗證以 **`cargo run --release --bin auxbridge-backup`** 取代（一致性快照＋SHA-256＋保留策略＋`--verify`）；文件完整性由 `platform/tests/docs_governance.rs`、機密掃描由 `platform/tests/secret_scan.rs` 於 `cargo test` 執行。git 層之 `git bundle` 備份仍為人工。

## 6. 變更紀錄

| 日期 | 版本 | 說明 |
|---|---|---|
| 2026-11-12 | v1.0 | 依 SHARED_STANDARDS §4.1 建立；收錄本輪實測之症狀 |