# 品質標準（Quality Standards）— Dev 根工作區

- 版本：v1.0｜生效日期：2026-11-12｜適用範圍：根 workspace 全體變更｜狀態：Active
- 規則 SSOT：`docs/standards/SHARED_STANDARDS.md` §6（品質門檻）、`docs/standards/GOVERNANCE_STANDARD.md`（治理與證據）
- 本檔只摘錄「如何判定通過」之可操作版本，條文以 SSOT 為準。

## 1. 風險分級與所需驗證

| 等級 | 例子 | 額外要求 |
|---|---|---|
| Low | 文件、字串、內部重構 | 基線六項 |
| Medium | UI、一般工作流、資料格式 | 基線＋針對性驗收測試＋契約檢查 |
| High | 認證/授權、付款、稽核、移轉、更新鏈 | 基線＋負向測試＋冪等/重試＋移轉回滾證據＋獨立審查 |

## 2. 基線檢查（每個變更）

1. `cargo fmt -- --check`
2. `cargo clippy --all-targets --locked -- -D warnings`
3. `cargo test --locked`（0 失敗）
4. 契約／架構檢查（`platform/tests/page_matrix.rs`、`naming_guard.rs`）
5. 文件完整性（`platform/tests/docs_governance.rs`）
6. 機密掃描（`platform/tests/secret_scan.rs`）
7. 供應鏈：`cargo audit`、`cargo deny check`、`cargo machete`（逐成員）

## 3. 證據規則

- 數字聲明必須附量測方法與日期；未量測者標 `UNKNOWN`（**UNKNOWN ≠ PASS**）。
- 測試結果與修訂版綁定：舊的綠燈不代表目前工作樹通過。
- 合成測試不得使目標驗收案例通過；`Implemented` 不等於交付能力。

## 4. 發佈判定（五級）

`PASS`｜`CONDITIONAL PASS`｜`BLOCKED`｜`REJECTED`｜`NOT VERIFIED`。
無證據不得勾選 PASS；未驗證者一律 `NOT VERIFIED`。

## 5. 效能與品質預算（可量測）

| 指標 | 目標 | 量測方式 |
|---|---|---|
| 建置時間（release，變更後增量） | 記錄基線，不得無理由變差 | `Measure-Command { cargo build --release }` |
| 二進位大小（auxbridge-server／auxbridge-web） | 記錄基線，成長需說明 | `Get-Item target/release/*.exe` |
| 相依 crate 數（root／desktop lock） | 只減不增（除非有理由） | `Select-String '^\[\[package\]\]' Cargo.lock` |
| 重複版本數 | 維持 `warn` 並逐季檢視 | `cargo tree -d` |

## 6. 變更紀錄

| 日期 | 版本 | 說明 |
|---|---|---|
| 2026-11-12 | v1.0 | 依 SHARED_STANDARDS §4.1 建立；基線對齊 CI 實際步驟 |