# 更新鏈 Runbook（W10／計畫 1793033568961）

> 狀態：v1.1（2026-11-10）｜產物已可**實際建置並通過 minisign 驗簽**（正例＋負例自動化測試）；
> **Authenticode（Azure Trusted Signing／OV-EV）尚未購證**——購證前所有產物仍標注 unsigned (test-only)，禁止生產分發聲明。

## 1. 金鑰保管（minisign／Ed25519）

| 項目 | 位置 | 紀律 |
|---|---|---|
| 私鑰 | `%USERPROFILE%\.tauri\flaps-erp\flaps-erp.key` | **repo 外**；遺失＝無法向已裝機推送更新（公鑰燒錄於已分發安裝包） |
| 公鑰 | 同目錄 `flaps-erp.key.pub`（內容已錄 `tauri.conf.json` plugins.updater.pubkey） | 可公開；變更＝全用戶重裝 |
| 密碼 | 同目錄 custody 記錄（本 runbook 不錄值） | **生產前必須輪換**（現值語義：flaps-updater-2026-rotate-before-production） |

備份：私鑰＋密碼離線備份至組織密碼管理器；本機遺失即輪換。

## 2. 構建與簽章（2026-11-10 實測校正）

> **重要校正（實測發現）**：`cargo tauri build`／`bundle` 只讀取環境變數 **`TAURI_SIGNING_PRIVATE_KEY`（私鑰「內容」）**；
> `TAURI_SIGNING_PRIVATE_KEY_PATH`（檔案路徑）**僅** `cargo tauri signer sign` 子命令支援。
> 誤用路徑變數會得到：`Error A public key has been found, but no private key. Make sure to set TAURI_SIGNING_PRIVATE_KEY environment variable.`

```powershell
# 1) 構建（產出 -setup.exe 與 .exe.sig；createUpdaterArtifacts=true；tauri build 自動帶 --app-version）
$env:TAURI_SIGNING_PRIVATE_KEY = [IO.File]::ReadAllText("$env:USERPROFILE\.tauri\flaps-erp\flaps-erp.key")
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<custody 密碼>"
cargo tauri build   # 於 web/src-tauri

# 2) 產物三件：
#    target/release/bundle/nsis/AuxBridge_<v>_x64-setup.exe
#    target/release/bundle/nsis/AuxBridge_<v>_x64-setup.exe.sig
#    target/release/bundle/nsis/latest.json（由本 repo 的 Rust 工具產生，見 §3）
#
# 3) 簽章（Authenticode）：購證後以 bundle.windows.signCommand 接 Azure Trusted Signing；
#    未購證前所有產物標注 unsigned (test-only)，禁止對外分發。
```

手動簽章（僅產物已存在、需重簽時；`--app-version` 必填，否則 `requireSignedVersion` 會拒裝）：

```powershell
cargo tauri signer sign -f "$env:USERPROFILE\.tauri\flaps-erp\flaps-erp.key" -p "<custody 密碼>" `
  --app-version 0.2.0 "web\src-tauri\target\release\bundle\nsis\AuxBridge_0.2.0_x64-setup.exe"
```

## 3. Feed（latest.json）產生（Rust 工具，零人工編輯）

工具：`web/src-tauri/src/bin/make_latest_json.rs`（Rust；base64／serde_json／minisign-verify，無新增第三方風險）。
它會 **以設定檔公鑰實際驗簽** 產物、檢查 trusted comment 的 `version:<v>` 綁定，再寫出 feed；任何不一致即拒絕產出。

```powershell
cargo run --manifest-path web/src-tauri/Cargo.toml --bin make_latest_json -- `
  [--output <path>] [--notes <text>] [--pub-date <rfc3339>] [--endpoint <base-url>]
```

格式（與 tauri-plugin-updater 2.14.x `verify_signature` 語意一致）：

```json
{ "version": "0.2.0", "notes": "...", "pub_date": "2026-11-10T18:15:03Z",
  "platforms": { "windows-x86_64": {
      "signature": "<.sig 檔全文（其本身即 base64(minisign 四行格式)）>",
      "url": "https://updates.auxbridge.internal/erp/Flaps%20ERP_0.2.0_x64-setup.exe" } } }
```

要點：
- `signature` 欄位＝`.sig` 檔全文（**不可再額外 base64**，外掛會再解一次）。
- `url` 檔名空白須 percent-encode（工具已處理）。
- `requireSignedVersion = true`（`tauri.conf.json`）時，外掛會比對簽章 trusted comment 的 `version:<v>` 與 feed `version`；
  不一致即拒裝（防 feed 版本號遭竄改）。CLI 於 2026-11-10 起支援 `--app-version`。

endpoint（tauri.conf）：`https://updates.auxbridge.internal/erp/latest.json`——**內部 HTTPS 主機尚未建置**（未竟項）。

## 4. 供應鏈閘（提交前＋CI 皆執行）

| 閘 | 命令 | 政策 SSOT |
|---|---|---|
| 格式 | `cargo fmt -- --check` | — |
| 靜態分析 | `cargo clippy --all-targets --locked -- -D warnings` | — |
| 測試 | `cargo test --locked` | — |
| 弱點 | `cargo audit` | `.cargo/audit.toml`（逐條書面處置＋複核觸發條件） |
| 政策 | `cargo deny check` | `deny.toml`（公告／版本／授權／來源四層） |
| SBOM | 每個 crate 以**自身目錄為 CWD** 執行 `cargo cyclonedx --format json --spec-version 1.5 --all --override-filename <crate>.cdx` | `docs/release/sbom/`（7 份：platform／auxbridge_core／kem／console_manifest／flaps_bridge／web／auxbridge-web） |
| SBOM 新鮮度 | `cargo test -p platform --test sbom_freshness` | `platform/tests/sbom_freshness.rs`（元件版本比對，非位元組比對） |
| 工具鏈 | `rust-toolchain.toml`（channel = 1.99.1，含 rustfmt／clippy） | 根目錄 |
| 桌面套件格式／靜態／測試 | `cargo fmt\|clippy\|test --manifest-path web/src-tauri/Cargo.toml` | 獨立 workspace（根 workspace 不含） |
| 桌面套件弱點 | `cargo audit --file Cargo.lock`（工作目錄 `web/src-tauri`） | `web/src-tauri/.cargo/audit.toml`（獨立 workspace 不繼承根政策） |
| 桌面套件政策 | `cargo deny check`（工作目錄 `web/src-tauri`） | `deny.toml`（自上層目錄自動載入，同一份政策 SSOT） |
| unsafe 邊界 | `cargo test -p platform --test guards`（`unsafe_code_is_absent_and_forbidden_at_crate_roots`） | `docs/adr/ADR-0014`（crate 根 forbid ＋ 全區掃描） |

其他硬化（2026-11-10）：
- CI（`.github/workflows/flexible-ci.yml`）第三方 action 一律 **commit SHA 釘版**（actions/checkout v4、actions/cache v4、actions/upload-artifact v4、dtolnay/rust-toolchain master）；工具安裝一律 `cargo install <tool> --locked`。
- 內部 crate 授權改用合法 SPDX 自訂項 `LicenseRef-Proprietary` ＋ `publish = false`（`UNLICENSED` 非合法 SPDX 運算式，會使 SBOM 產生警告）。
- path 依賴補 `version`（cargo-deny `wildcards = deny` 可重現解析）。

其他硬化（2026-11-11 審查補齊）：
- **桌面套件（`web/src-tauri` 為獨立 workspace）原本不在任何供應鏈閘內**：根 `Cargo.lock`／`cargo audit`／`cargo deny`／SBOM 皆不涵蓋實際交付的安裝包依賴圖。現已補齊 fmt／clippy／test／audit／deny／SBOM 六項閘。
- CI 以 `actions/checkout`（同一 SHA 釘版）取 **it-wellfit/wellfit** 至 `Wellfit/`（`ref` 釘 `b5cf18e1dd43675f5a4fe0fa8dd396a3ef028fd7`，即「內部 path 依賴補 version」修正之提交），原因：桌面套件 path 依賴 `Wellfit/crates/{wellfit_core,wellfit_bankpay}`，而 `Wellfit/` 為獨立 repo 且被根 `.gitignore` 排除。
- **跨 repo 修正（已完成）**：Wellfit 工作區的內部 path 依賴補 `version = "0.1.0"`（`Wellfit/Cargo.toml`），否則 cargo-deny 於桌面套件圖中報 `wildcard for crate 'wellfit_bankpay'`。該修正已於 2026-11-11 提交並推送（commit `b5cf18e`），Dev CI 的 `ref` 已同步更新。
- `deny.toml` 圖範圍改為 `all-features = true`（含 optional 依賴與各 backend），兩 workspace 實測皆 `advisories ok / bans ok / licenses ok / sources ok`。
- **cargo-deny 操作陷阱**：`cargo deny --config <key>=<value> check` 會以 CLI 覆寫取代 `deny.toml` 其餘策略（ignore／allow 清單全部失效，實測出現約 300 筆授權錯誤與 3 筆公告錯誤）。變更圖範圍一律直接改 `deny.toml`，不可用 CLI 覆寫。
- 新增兩則桌面套件公告書面處置：**RUSTSEC-2024-0370**（proc-macro-error 未維護，Linux GTK/glib 建置期依賴）與 **RUSTSEC-2024-0429**（glib unsound，Linux-only 後端）；兩者皆不編入本產品唯一的 Windows 交付物（複核觸發條件見 `.cargo/audit.toml`）。
- 授權白名單新增 **MPL-2.0**（僅 Tauri css 解析鏈：cssparser／cssparser-macros／dtoa-short／option-ext／selectors；未修改、未獨立散布）。
- 共用單一政策檔於兩個 workspace 會產生**非失敗**之裝飾性警告（`advisory-not-encountered`／`license-not-encountered`），屬預期行為，不影響閘門結果。

其他硬化（2026-11-11 第二輪，ADR-0014）：
- **unsafe 全數移除**：刪除未參與編譯之 COM 死碼 `platform/src/com/{async_ipc,batch,bridge,pool}.rs`（6 處 unsafe，且無 `CoInitializeEx`，若接上即未定義行為）；8 個 crate 根全部補 `#![forbid(unsafe_code)]`；新增 guard 測試掃描全區 unsafe 語法（實測 0 處）。真實 COM 通道留在 `flaps_bridge`（`real-com` feature，crate 根已 forbid）。
- **每個 workspace 自有弱點政策**：`cargo audit` 以 workspace 根為政策搜尋起點，故新增 `web/src-tauri/.cargo/audit.toml`（三則處置：RUSTSEC-2026-0459／2024-0370／2024-0429，含桌面特有暴露評估）。
- **SBOM 產生方式統一**：`--manifest-path` 從 repo 根執行會產生錯誤範圍（實測 6 份全部落到 `core/`），必須以 crate 目錄為 CWD；檔名隨 crate（`auxbridge_core`／`auxbridge-web`），舊名 `flaps_core`／`flaps-erp-web` 已刪除。
- **SBOM 新鮮度閘**：`platform/tests/sbom_freshness.rs` 以 `cargo metadata --filter-platform x86_64-pc-windows-msvc` 的可達套件集（排除 dev）與 SBOM 元件集比對；`cargo update` 後未重產 SBOM 會即時失敗（2026-11-11 實測：更新後首跑即抓到漂移）。
- **依賴更新**：`cargo update`（根與桌面）已套用；libsqlite3-sys 0.30.1 → 0.37.0（仍在 RUSTSEC-2026-0459 範圍 0.6.0–0.38.2 內，處置續行）。
- **cargo-auditable／cargo-vet 決定**（ADR-0014 D4）：auditable 用於伺服器二進位發佈；cargo-vet 暫緩（對外散布前再啟動）。

## 5. 端到端驗證清單（2026-11-10 實測結果）

| # | 項目 | 狀態 | 證據 |
|---|---|---|---|
| 1 | `cargo tauri build` 全綠 → 安裝器＋.sig 產出 | **PASS（2026-11-11 重產）** | `AuxBridge_0.2.0_x64-setup.exe`（5,233,244 bytes）＋`.sig`（440 bytes）；首跑發現舊產物 `.sig`（01:47）與 11:53 重編之執行檔不符 → 驗收測試正確失敗，重新簽章後通過 |
| 2 | latest.json 三件一致（version／url／signature） | **PASS** | `make_latest_json` 產出並以設定公鑰驗簽通過；`version=0.2.0`；url 為 `…/erp/AuxBridge_0.2.0_x64-setup.exe` |
| 3 | 正例：產物通過 minisign 驗簽 | **PASS（自動化）** | `web/src-tauri/tests/updater_artifact.rs::release_artifacts_are_signed_and_version_bound` |
| 4 | 負例一：竄改簽章 → 驗簽必敗 | **PASS（自動化）** | 同上測試（竄改簽章文字**第 2 行載荷**；第 1 行為未簽章之 untrusted comment，竄改不影響驗簽——2026-11-11 修正） |
| 5 | 負例二：竄改產物位元組 → 驗簽必敗 | **PASS（自動化）** | 同上測試（翻轉末位元組 → Err） |
| 6 | 負例三：feed 宣告版本與簽章綁定不符 → 拒裝 | **PASS（自動化）** | 同上測試（以 `feed::verify_signature_for_version` 走**實際驗證路徑**，非字串比對；另含「未綁定版本」負例） |
| 7 | 設定層不變量（requireSignedVersion／pubkey／HTTPS／版本一致） | **PASS（自動化）** | `updater_artifact.rs::config_pins_signed_version_updates` |
| 8 | /console/index.html 於安裝版 200 | **PASS（開發版實測）／安裝版待裝機驗證** | 開發版：platform 伺服器 `GET /console/index.html` 200（session 保護，無 session 回登入頁）；`knowledge/console` 65 檔（workflows 13／roles 5／claims 46＋索引）已由 NSIS `resources` 捆綁 |
| 9 | 安裝 v0.2 → feed 指向 v0.3 → passive 更新 → 重啟版本正確 | **BLOCKED** | 需 `updates.auxbridge.internal` HTTPS 主機（未建）＋實際安裝升級演練；需授權與主機 |
| 10 | Authenticode 簽章後 SmartScreen 無黃條 | **BLOCKED** | 需購證（Azure Trusted Signing／OV-EV）＋憑證 |
| 11 | 私鑰密碼輪換（生產前） | **OPEN** | custody 記錄待輪換（現值語義見 §1） |
| 12 | 桌面套件供應鏈閘（fmt／clippy／test／audit／deny） | **PASS（2026-11-11 實測）** | fmt EXIT=0；clippy `--all-targets --locked -D warnings` EXIT=0；`cargo test --manifest-path web/src-tauri/Cargo.toml --locked` → 9 passed（7 單元＋2 整合）；`cargo audit --file Cargo.lock`（於 `web/src-tauri`）EXIT=0（532 相依）；`cargo deny check` → advisories／bans／licenses／sources 全 ok |
| 13 | 根 workspace 五閘（fmt／clippy／test／audit／deny） | **PASS（2026-11-11 收斂後重測）** | fmt EXIT=0；clippy `-D warnings` EXIT=0；`cargo test --locked` → **30 suites／305 passed／0 failed**（含 sbom_freshness 3 項與 guards 6 項）；`cargo audit` EXIT=0（463 相依／1458 公告）；`cargo deny check` 四層全 ok |
| 14 | CI 於乾淨 checkout 可執行（桌面套件 path 依賴具備） | **PASS（本機等價驗證）／CI 實跑待觀察** | 本機以等價流程驗證：Wellfit 已推送 `b5cf18e` 且 CI `ref` 同步；桌面 audit 以 `working-directory` 執行（政策檔可命中）；SBOM 以 crate 目錄為 CWD 產生 |
| 15 | SBOM 新鮮度（元件版本比對） | **PASS（自動化）** | `platform/tests/sbom_freshness.rs` 3 項；2026-11-11 實測：套用 `cargo update` 後首跑即抓到漂移（SBOM 未重產）→ 重產 7 份後通過 |
| 16 | unsafe 邊界（crate 根 forbid＋全區掃描） | **PASS（自動化）** | `guards.rs::unsafe_code_is_absent_and_forbidden_at_crate_roots`；實測 unsafe 語法 0 處、8 個 crate 根皆帶 forbid（ADR-0014） |
| 17 | 依賴更新（根與桌面） | **PASS** | `cargo update`：libsqlite3-sys 0.30.1→0.37.0、wry 0.57.1→0.57.2、zip 9.5.0→9.5.1 等；更新後五閘與桌面五閘重測全綠 |
| 18 | 每個 workspace 自有弱點政策 | **PASS** | 新增 `web/src-tauri/.cargo/audit.toml`（RUSTSEC-2026-0459／2024-0370／2024-0429 書面處置）；於 `web/src-tauri` 執行 `cargo audit --file Cargo.lock` EXIT=0（修前為 1 vulnerability found） |

驗收測試執行方式：

```powershell
cargo test --manifest-path web/src-tauri/Cargo.toml --test updater_artifact
# 產物未建時會略過產物項；發行流水線請設 AUXBRIDGE_REQUIRE_RELEASE_ARTIFACTS=1 令缺產物直接失敗
```
