# ADR-0014：unsafe 邊界、供應鏈政策與 SBOM 閘（平台側）

- 狀態：Accepted
- 日期：2026-11-11
- 關聯：憲章 §2.2（`#![forbid(unsafe_code)]`）、§2.5（證據紀律）、`AGENTS.md` §3、`docs/release/UPDATER_RUNBOOK.md` §4、計畫 `1793033568961`（W10 收尾）
- 前置：ADR-0001（技術基線）、ADR-0012（bankpay 邊界）、ADR-0013（跨平台策略）

## 背景

2026-11-11 的全域審查發現三個「政策存在但基層未落實」的缺口：

1. **unsafe 邊界名存實亡**：`platform/src/com/{bridge,pool,batch,async_ipc}.rs` 共 6 處
   `unsafe { ... }`（`CoCreateInstance`、`IDispatch::Invoke`、`VARIANT::default`），
   且**無任何 `CoInitializeEx`／`CoUninitialize`**（COM apartment 未初始化即呼叫），
   同時 `com.rs` 未宣告任何子模組 → 這 4 個檔案從未參與編譯（死碼）。
   另一方面，73 個 `.rs` 中有 47 個（含全部 crate 根）沒有 `#![forbid(unsafe_code)]`，
   憲章 §2.2 的「編譯器強制」實際上未生效。
2. **供應鏈政策跨 workspace 斷裂**：`web/src-tauri` 為獨立 workspace，`cargo audit`
   以 workspace 根為政策搜尋起點，因此不繼承 `Dev/.cargo/audit.toml`；桌面套件
   （實際交付物）的弱點政策等於沒有。
3. **SBOM 無新鮮度閘**：SBOM 內含 timestamp／serialNumber，無法位元組比對；既有 7 份
   SBOM 產生方式不一致（`--all` 與否混用、`flaps_core` 舊名殘留），且無任何機制阻止
   「依賴已升級、SBOM 未重產」。

## 決策

### D1：unsafe 全數移除，並以 crate 根編譯器強制

- **刪除** `platform/src/com/{async_ipc,batch,bridge,pool}.rs`（未參與編譯之死碼；其 unsafe
  呼叫缺 COM apartment 初始化，若日後被接上將是立即的未定義行為風險）。
- 全部 crate 根補 `#![forbid(unsafe_code)]`：`core/src/lib.rs`、`platform/src/lib.rs`、
  `web/src/lib.rs`、`web/src-tauri/src/{lib,main}.rs`（其餘 `crates/*/src/lib.rs` 原本已具）。
- 新增防回歸閘 `platform/tests/guards.rs::unsafe_code_is_absent_and_forbidden_at_crate_roots`：
  掃描工作區 Rust 原始碼不得出現 unsafe 語法，且每個 crate 根必須帶 forbid 屬性。
- 實測（2026-11-11）：全工作區 unsafe 語法 **0 處**；8 個 crate 根全部帶 forbid。

### D2：真實 COM 通道留在 `flaps_bridge`，平台層不再自行 FFI

- 真實 COM 通道為 `crates/flaps_bridge`（`real-com` feature 門控，crate 根已 forbid，實作零 unsafe）。
- 未來若需在平台層直接呼叫 COM，必須：(a) 獨立 crate；(b) 每個使用執行緒明確
  `CoInitializeEx`／`CoUninitialize`（並在型別或 guard 上綁定生命週期）；(c) 每個
  `unsafe` 區塊附 `// SAFETY:` 說明；(d) 具正／負例測試。上述條件未滿足前不得開啟。

### D3：供應鏈政策按 workspace 配置，共用檔僅限可共用者

- `cargo audit`：每個 workspace 自有 `.cargo/audit.toml`（新增 `web/src-tauri/.cargo/audit.toml`，
  與根政策一致並補桌面特有暴露評估）；CI 之桌面 audit 以 `working-directory: web/src-tauri` 執行。
- `cargo deny`：共用根 `deny.toml`（cargo-deny 由 CWD 向上尋找，兩 workspace 皆命中）；
  共用政策在兩圖上會產生**非失敗**之裝飾性警告（`advisory-not-encountered`／
  `license-not-encountered`），已記錄為預期行為。
- `deny.toml` 圖範圍 `all-features = true`（掃描可交付最大範圍）；**禁止**以
  `cargo deny --config` 覆寫（實測會使本檔 ignore／allow 策略全部失效）。

### D4：`cargo auditable` 用於伺服器二進位；`cargo vet` 暫緩

- **cargo-auditable**：於發佈流程以 `cargo auditable build --release` 產出平台伺服器
  （`auxbridge-server`）二進位，使二進位內嵌相依清單（可離線稽核）。桌面安裝包由
  `cargo tauri build` 產生，未經 auditable —— 此為已知限制，列入後續評估。
- **cargo-vet**：暫不採用。理由：需對 400+ 相依逐筆人工稽核，與目前規模與階段不符；
  觸發條件：對外散布前、或引入新的關鍵第三方（加密／傳輸／DB）時再啟動。

### D5：SBOM 以「元件版本比對」作為新鮮度閘

- 放棄位元組比對（含 timestamp／serialNumber，必然漂移），改為**元件集比對**：
  `platform/tests/sbom_freshness.rs` 以 `cargo metadata --filter-platform x86_64-pc-windows-msvc`
  計算每個 crate 的可達套件集（排除 dev-dependencies），與 `docs/release/sbom/*.cdx.json`
  的 `metadata.component` ＋ `components[]` 比對，缺漏或冗餘即失敗。
- SBOM 一律以「該 crate 目錄為 CWD」＋`--all --spec-version 1.5` 產生；`--manifest-path`
  從 repo 根執行會產生錯誤範圍（實測 6 份全部落到 `core/`）。
- 檔名隨 crate 名（`auxbridge_core.cdx.json`、`auxbridge-web.cdx.json`）；舊名
  `flaps_core.cdx.json`／`flaps-erp-web.cdx.json` 已刪除。

## 後果

- 正面：unsafe 由編譯器在每個 crate 強制（非文件約定）；桌面交付物首次納入完整弱點／
  授權／來源政策；SBOM 可持續驗證；COM 死碼與其未定義行為風險移除。
- 成本：新增 2 個閘（unsafe guard、SBOM freshness），CI 增加 2 個 audit 執行點；
  每次 `cargo update` 後必須重產 7 份 SBOM（否則 `cargo test` 即失敗，屬預期）。
- 承擔：桌面安裝包二進位未經 cargo-auditable（D4）；`multiple-versions` 重複版本屬
  生態常態（不同 major 需求），維持 `warn`，不以人為方式強迫收斂。

## 複核觸發

1. 任何 `.rs` 需要 unsafe（COM／FFI／其他）→ 依 D2 條件重新評估，並更新本 ADR。
2. 新增 workspace 或 crate 根 → 必須帶 `#![forbid(unsafe_code)]` 與 SBOM。
3. 對外散布（外部客戶）前 → 啟動 cargo-vet 評估（D4）。
4. 新增 Linux／macOS 建置目標 → 重審 glib／gtk 相關公告處置（RUSTSEC-2024-0370／0429）。