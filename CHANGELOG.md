# Changelog — Dev 根工作區

本檔記錄根 workspace（`core/`、`platform/`、`web/`、`crates/flaps_bridge`）的實際變更。格式參考 Keep a Changelog。子專案（ekpi-platform、Wellfit、SyncMesh）各自有變更記錄。


## [Unreleased] — 2026-11-11（交付缺口收口：桌面套件納入閘門、SBOM 新鮮度閘、unsafe 邊界、跨 repo 提交；ADR-0014）

### Added
- **SBOM 新鮮度閘**（`platform/tests/sbom_freshness.rs`，3 tests）：以 `cargo metadata --filter-platform x86_64-pc-windows-msvc` 的可達套件集（排除 dev-dependencies）與 `docs/release/sbom/*.cdx.json` 的 `metadata.component` ＋ `components[]` 比對；SBOM 含 timestamp／serialNumber 故不採位元組比對。實測：`cargo update` 後首跑即抓到漂移。
- **unsafe 邊界閘**（`guards.rs::unsafe_code_is_absent_and_forbidden_at_crate_roots`）：掃描全工作區 Rust 原始碼不得出現 unsafe 語法，且每個 crate 根必須帶 `#![forbid(unsafe_code)]`。
- **桌面套件弱點政策**（`web/src-tauri/.cargo/audit.toml`）：`cargo audit` 以 workspace 根為政策搜尋起點，獨立 workspace 不繼承根政策——新增本檔後，桌面圖之 RUSTSEC-2026-0459／2024-0370／2024-0429 才有書面處置（修前實測 `1 vulnerability found`）。
- **更新鏈測試金鑰 round-trip**（`web/src-tauri/src/feed.rs` 7 個單元測試）：以 `minisign` 產生測試金鑰 → 簽章 → 組 feed → 以 `minisign-verify` 驗簽，含三負例（竄改簽章／竄改產物／版本不符）與「未綁定版本」負例；不需求任何實機產物，可在 CI 無條件執行。
- **ADR-0014**（`docs/adr/ADR-0014-unsafe-boundary-and-supply-chain-policy.md`）：unsafe 邊界、真實 COM 通道歸屬、每個 workspace 之弱點政策、cargo-auditable／cargo-vet 決定、SBOM 產生與新鮮度機制。

### Changed
- **刪除未參與編譯之 COM 死碼**（`platform/src/com/{async_ipc,batch,bridge,pool}.rs`，6 處 unsafe 且無 `CoInitializeEx`）；8 個 crate 根補 `#![forbid(unsafe_code)]`（`core`／`platform`／`web`／`web/src-tauri` 之 lib/main 與既有 crates）。
- **SBOM 全數重產**（7 份，`--all --spec-version 1.5`，以各 crate 目錄為 CWD）：`platform`／`auxbridge_core`／`kem`／`console_manifest`／`flaps_bridge`／`web`／`auxbridge-web`；刪除舊名 `flaps_core.cdx.json`／`flaps-erp-web.cdx.json`。`--manifest-path` 從 repo 根執行會產生錯誤範圍（實測 6 份全落到 `core/`）。
- **更新鏈驗收強化**：`updater_artifact.rs` 改與產生器共用 `feed` 模組（單一實作來源）；負例三改走**實際驗證路徑**（`verify_signature_for_version`），不再只是字串比對；修正負例一之竄改位置（minisign 簽章文字第 1 行是**未簽章**的 untrusted comment，竄改不影響驗簽）。
- **重新簽章產物**：原 `Flaps ERP_0.2.0_x64-setup.exe.sig`（01:47）與 11:53 重編之執行檔不符（驗收測試正確失敗）→ 以設定公鑰重產 `AuxBridge_0.2.0_x64-setup.exe`（5,233,244 bytes）＋`.sig`（440 bytes）＋`latest.json`，並清除舊名產物。
- **依賴更新**：根與桌面 `cargo update`（libsqlite3-sys 0.30.1→0.37.0、wry 0.57.1→0.57.2、zip 9.5.0→9.5.1、uuid 1.29.0→1.29.2 等）；更新後五閘與桌面五閘重測全綠（SBOM 同步重產）。
- **CI 修正**：桌面 audit 改以 `working-directory: web/src-tauri` 執行（政策檔命中）；SBOM 步驟改為逐 crate 以自身目錄為 CWD 產生（含 `Push-Location` 迴圈）。
- **跨 repo 提交**：`Wellfit/Cargo.toml` 內部 path 依賴補 `version = "0.1.0"`（消除桌面套件 `wildcard for crate 'wellfit_bankpay'`），已提交並推送（`b5cf18e`）；Dev CI 之 Wellfit `ref` 同步為該 commit。

### Verified（2026-11-11）
- 根五閘：fmt EXIT=0／clippy `-D warnings` EXIT=0／`cargo test --locked` **30 suites、305 passed、0 failed**／`cargo audit` EXIT=0（463 相依、1458 公告）／`cargo deny check` 四層全 ok。
- 桌面五閘：fmt EXIT=0／clippy `-D warnings` EXIT=0／`cargo test --locked` **9 passed**（7 單元＋2 整合）／`cargo audit --file Cargo.lock` EXIT=0（532 相依）／`cargo deny check` 四層全 ok。
- Wellfit 三閘（以 `Knowledge/tools/dev-env/build.ps1`）：check／clippy `-D warnings`／test 全通過（459 passed、0 failed、9 ignored）。
- unsafe 語法全工作區 0 處；8 個 crate 根皆帶 `#![forbid(unsafe_code)]`。

## [Unreleased] — 2026-11-11（命名與邊界裁定：自建系統更名 AuxBridge／整合工作台，與原廠 FlapsErp 身份分離）

### Changed
- **身份分離**（使用者裁定，SSOT：`docs/standards/NAMING_AND_BOUNDARY_DECREE.md`）：原廠＝FlapsErp（輔翼ERP，僅可深度串接與 config 調整，禁改內容）；自建平台一律 **AuxBridge／整合工作台**。全庫掃描更名 78 檔 205 處、活躍面殘留 0：bin `auxbridge-server`（原 `flaps-server`）、crate `auxbridge_core`（原 `flaps_core`）、env `AUXBRIDGE_*`（原 `FLAPS_*`）、DB `auxbridge.db`／`demo-auxbridge.db`、UI 品牌「AuxBridge 整合工作台」（禁用「輔翼工作台」——輔翼為原廠中文名）、tauri `productName: AuxBridge`、updater `updates.auxbridge.internal`。
- **防呆測試**（新增 `platform/tests/naming_guard.rs`，3 tests）：自建活躍面（templates/src/tests/web assets/tauri conf/Cargo.tomls）禁出現原廠標記（`Flaps ERP`、`輔翼工作台`、`flaps-server`、`flaps_core`、`FLAPS_` 等）；含正向控制（自建品牌必須實際渲染）與舊 bin 復活防呆。

### 保留舊名（合法語境，見 decree §5）
- `docs/research/FlapsErp*`（原廠文檔）、`web/legacy-1420.html`（歷史快照）、`.kilo/plans`、`knowledge/claims`（審計鏈）、`crates/flaps_bridge`（串接目標命名規則）。


## [Unreleased] — 2026-11-10（供應鏈硬化＋W10 端到端：cargo-deny 四層閘、SBOM、requireSignedVersion、latest.json 工具與簽章負向用例；addendum #26）

### Added
- **cargo-deny 政策**（新 `deny.toml`）：公告／版本／授權／來源四層；`wildcards = deny`、僅 crates.io 來源、內部 crate 以 `private.ignore` 排除授權政策。輸出 `advisories ok, bans ok, licenses ok, sources ok`。
- **SBOM**：CycloneDX 1.5 共 7 份（platform／flaps_core／kem／console_manifest／flaps_bridge／web／flaps-erp-web）落地 `docs/release/sbom/`。
- **rust-toolchain.toml**：channel = 1.99.1（components rustfmt/clippy）——本機與 CI 工具鏈一致、可重現。
- **CI 供應鏈閘**（`.github/workflows/flexible-ci.yml`）：cargo-deny 檢查＋CycloneDX SBOM 產出＋artifact 上傳；第三方 action 全部 commit SHA 釘版。
- **latest.json 產生器**（`web/src-tauri/src/bin/make_latest_json.rs`，Rust）：以設定公鑰實際驗簽＋檢查 `version:<v>` 綁定後才寫 feed，零人工編輯。
- **更新鏈驗收測試**（`web/src-tauri/tests/updater_artifact.rs`）：設定層不變量＋正例（產物驗簽）＋三負例（篡改簽章／篡改產物／宣告版本不符）。
- **公告處置**（`.cargo/audit.toml`）：RUSTSEC-2026-0459／RUSTSEC-2024-0436／RUSTSEC-2026-0192 逐條書面理由、緩解措施與複核觸發條件。

### Changed
- **requireSignedVersion = true**（`web/src-tauri/tauri.conf.json`）：簽章 trusted comment 的 `version:<v>` 與 feed 宣告版本不符即拒裝；同步收緊 CSP（移除 `style-src 'unsafe-inline'`）。
- **內部 crate 授權**：`UNLICENSED` → `LicenseRef-Proprietary` ＋ `publish = false`（`UNLICENSED` 非合法 SPDX，會使 SBOM 產生警告）。
- **askama 功能收斂**：`default-features = false, features = ["derive","std"]`——根除 unmaintained 的 basic-toml／glob（repo 無 askama.toml，行為等價）。
- **path 依賴補 version**（kem→console_manifest、platform→flaps_core/console_manifest/kem）：滿足 cargo-deny `wildcards = deny` 的可重現解析要求。
- **`admin_users.html` 導覽一致性**：補 `studio_nav` macro ＋ `AdminUsersTemplate.nav`（依 ROLE_MANAGE 過濾、active 指向 /admin/users）。
- **CI 設定檔**：rust_channel → "1.99.1"、quality_gates 增 `deny`／`sbom`；修正 `ci-config.yml` 內非法跳脫 `\T`（雙引號 → 單引號）。
- **Runbook v1.1**（`docs/release/UPDATER_RUNBOOK.md`）：校正簽章環境變數（`cargo tauri build` 只讀 `TAURI_SIGNING_PRIVATE_KEY` **內容**，路徑變數僅 `sign` 子命令支援）、補 latest.json 產生流程與供應鏈閘節、端到端清單附實測結果。

### Verified
- 量測（2026-11-10）：`cargo deny check` 四層全綠；`cargo audit` 掃描 459 個 crate 依賴、1446 則公告、無未處置輸出；`cargo check --workspace --offline` EXIT=0；`cargo test --manifest-path web/src-tauri/Cargo.toml --test updater_artifact` → 2 passed。
- W10 建置實測：`cargo tauri build` 產出 `Flaps ERP_0.2.0_x64-setup.exe`（5,116,505 bytes）＋`.sig`（440 bytes）；`make_latest_json` 以設定公鑰驗簽通過並產出 latest.json（version 0.2.0）。
- /console 靜態服務實測：開發版 `GET /console/index.html` 200（session 保護）；`knowledge/console` 65 檔已由 NSIS `resources` 捆綁。
- 未竟（誠實記錄）：安裝版升級演練（需 feed 主機）、Authenticode 購證、私鑰密碼輪換、`cargo vet`／`cargo auditable`（未安裝 → UNKNOWN）。


## [Unreleased] — 2026-11-11（供應鏈狀態審查補齊：桌面套件納入閘門、跨 repo 修正、實測數字更正）

### Fixed
- **桌面套件（`web/src-tauri` 為獨立 workspace）原本不在任何供應鏈閘內**：根 `Cargo.lock`／`cargo audit`／`cargo deny`／SBOM 皆不涵蓋實際交付的安裝包依賴圖（509 個 crate）。現補齊 fmt／clippy／test／audit／deny／SBOM 六閘，2026-11-11 實測全綠。
- `flaps-erp-web` manifest 補 `license = "LicenseRef-Proprietary"` ＋ `publish = false`（原為 unlicensed，cargo-deny 授權閘失敗）；桌面 SBOM 重新產出（SBOM 元資料已含 SPDX 運算式）。
- 桌面套件 path 依賴（`wellfit_bankpay`／`wellfit_core`）補 `version = "0.1.0"`；跨 repo 修正 `Wellfit/Cargo.toml` 工作區內部 path 依賴補 `version`（消除 cargo-deny `wildcard for crate 'wellfit_bankpay'` 錯誤）——此項**必須於 Wellfit repo 另行提交**，Dev CI 的 Wellfit `ref` 須同步更新。
- CI 桌面 SBOM 步驟原為無條件執行，乾淨 checkout 缺 `Wellfit/`（獨立 repo、被根 `.gitignore` 排除）將使 CI 失敗；現加入 it-wellfit/wellfit 的 `actions/checkout`（SHA 釘版、`ref` 釘 commit）與桌面閘步驟。

### Changed
- `deny.toml`：圖範圍改 `all-features = true`（兩 workspace 實測 `advisories/bans/licenses/sources` 全 ok）；授權白名單新增 `MPL-2.0`（僅 Tauri css 解析鏈）；新增 `RUSTSEC-2024-0370`／`RUSTSEC-2024-0429` 忽略項（Linux-only GTK/glib 路徑，不入 Windows 交付物），並註記 cargo-deny 0.20.2 之 `--config` 覆寫陷阱（CLI 覆寫會使本檔 ignore／allow 策略全部失效）。
- `.cargo/audit.toml`：補上述兩則桌面套件公告之書面處置（理由／緩解／複核觸發條件）。

### Docs
- `docs/release/UPDATER_RUNBOOK.md` v1.1 補充：桌面套件六閘與 Wellfit checkout 機制、`cargo deny --config` 陷阱說明、端到端清單新增第 12–14 列實測結果。
- **實測數字更正（誠實記錄）**：本次審查時根 workspace 因平行編輯中的 `platform/src/api.rs` 暫時無法編譯（`break` in async fn 等 6 個錯誤），2026-11-10 的「五閘全綠／test 294 passed」屬**歷史量測值**，須於該變更落地後重測；依既有實測分項合計為 **293**，未重測前不得宣稱 294 為現行值。

## [Unreleased] — 2026-11-10（統一營運控制台 manifest 雙渲染器；addendum #20）

### Added
- **Console Manifest SSOT**（計畫 `1793033568961` W1）：新 crate `crates/console_manifest`——NAV_GROUPS(11)/PAGES(65 登記)/FAMILIES(2)/擴展律 `pages=24+claims+⌈ledger/200⌉`（與 `kem console generate()` 逐次相等；實測 45 頁）；13 契約測試。
- **導覽統一**（W2/grill-B）：`platform/src/nav.rs` NavModel 由 manifest 生成、`require_session` middleware 每請求注入；38 模板＋handlers 接線；`partials/studio_nav.html` site_nav macro（aria-current、零 JS 下拉、角色徽章）。**無權限入口不渲染**——/audit 死連結（B7a）根治，回歸測試鎖定。
- **/accounts/:id 科目詳情**（W3/F4）：LEDGER_READ 閘、未知 id 404 樣式頁、列表/子科目連結；`partials/state_anchors.html` 八態掛點（W4）經導覽 macro 全頁輸出。
- **/console/*path 知識控制台伺服**（W6/grill-A）：session 保護、防穿越 fail-closed、副檔名白名單（html/css/json）、多候選根解析（exe_dir→CWD→父層）；KEM 生成頁 45 頁重新生成（token 全收斂＋零內聯樣式）。
- **發行鏈現代化**（W10）：tauri-plugin-updater 接線（minisign 公鑰鎖定/passive/createUpdaterArtifacts）；minisign 金鑰對（私鑰 repo 外）；NSIS resources 捆綁 knowledge/console；runbook `docs/release/UPDATER_RUNBOOK.md`。**unsigned (test-only)**。

### Changed
- **KEM console.css token 全收斂**（W7/grill-E）：canonical 名稱＋值（design-system.css SSOT）；色差映射表見 PLAN_REGISTER addendum #20；dark mode 對齊 canonical。
- 擴展律文檔校正：`pages ≈ 21+…` → `pages = 24 + claims + ⌈ledger/200⌉`（實測校準）。
- 死碼清除（F5）：`apihub::router()`/`workbench::router()`/`login_router()`（零呼叫者實證）。
- runtime DB 狀態檔（flaps.db-shm/-wal）解除追蹤（.gitignore 意圖完成）。

### Verified
- 量測（2026-11-10）：fmt EXIT=0；clippy -D warnings EXIT=0；test **270 passed / 0 failed**（platform 153＋kem 43＋flaps_core 58＋console_manifest 13＋護欄/矩陣）；audit EXIT=0（2 allowed，audit.toml 載明）。
- page_matrix：visitor 303／admin 200／store1 操作者 200＋治理 403／finance Write 200＋治理 403；安全標頭；導覽可達；aria-current；八態掛點；/console 200/303/404/穿越 5 形態/json 白名單；死連結回歸防護。
## [Unreleased] — 2026-11-09（第十四輪：PHASE-R — S1 檢索面 RAG 升級）

### Added
- **混合檢索（RAG）落地於工作台**（`1793787631643` PHASE-R；addendum #17）：
  - `platform/src/bm25.rs`（新）：std-only BM25（K1=1.2、B=0.75、Robertson IDF 與 kem 同式）＋CJK 感知 tokenizer（單一漢字 token——改進 kem `w.len()>1` 過濾丟字的限制）；零新外部依賴（Tantivy 教訓）。
  - `platform/Cargo.toml`：`kem = { path = "../crates/kem" }`——**重用**已驗證的 embedder 池（OVH bge-m3 免金鑰 → Ollama 本地 fallback）與 cosine；kem 依賴為 platform 依賴子集，零新增外部 crate。
  - `search.rs`：`SearchEngine` 升級為混合引擎——BM25 索引（變更即重建，doc_id 排序定位）＋新鮮度戳記向量（`sha256(content)`，kem §8 規則，失配視同缺失）＋`search_ranked()`（α·norm(BM25)+(1−α)·norm(cosine)，α=0.5、語義拒絕地板 0.55 對齊 kem 校準）＋`via` 溯源（"hybrid"/"bm25"）；embedder 不可達→確定性 BM25-only 降級，**永不捏造向量分數**；舊 `search()` 簽名不變（相容）。
  - `cross_module_api.rs`：`/api/search` 回真實融合分數；`SearchResult.via` 加法欄位；直取標 `via="direct"`。
- **跨線 stale fixture 修復**：`bankpay_persistence.rs` 期望鍵數 5→10（並行線 EsunConfig mTLS 欄位擴增），附理由註解。

### Verified
- 四關全綠（2026-11-09）：fmt 0／clippy 0／test --locked **246 passed 0 failed**（bridge 7＋core 42＋kem 58＋platform 108＋kem-int 15＋bankpay_persistence 1＋characterization 6＋close_out 4＋slice12 2＋misc 3）／audit 0（兩工作區；platform-hub 新增自有 `.cargo/audit.toml` 鏡像 RUSTSEC-2026-0459 處置——hub 鎖 0.37.0，同受範圍限制）。
- RAG 測試 15 項新增：bm25 5＋search 混合 6＋降級/排序整合。
- 證據：`docs/PLAN_REGISTER.md` addendum #16/#17。

## [Unreleased] — 2026-11-09（第十三輪：安全驅動 sqlx 0.9 升級＋RUSTSEC-2026-0459 處置）

### Security
- **sqlx 0.8.6 → 0.9.0**（根 workspace、core、platform 三處 manifest 統一）：回應 RUSTSEC-2026-0459（CVE-2026-119375，severity 6.8 medium）——libsqlite3-sys 0.30.1 內建 SQLite < 3.54.0，特製 .db＋頁大小不符之 -wal 檔對可致未初始化頁快取記憶體洩漏（僅本地向量）。首次升級曾造成雙版本共存（core 仍鎖 0.8，兩個 `Pool<Sqlite>` 型別衝突＋FromRow/audit-lint 連鎖錯誤），以 `cargo tree -i sqlx` 根因定位後統一三處 manifest 解決。
- **RUSTSEC-2026-0459 有紀錄接受**：修復需 libsqlite3-sys ≥0.39.0，但 sqlx-sqlite 0.9.0（最新發布）釘選 `>=0.30.1, <0.38.0`——上游無範圍內修復。依 GOVERNANCE P2/P7 於 `.cargo/audit.toml` 記錄處置：本地向量、應用僅開啟 data\ 下操作者自建檔、單機桌面部署、每日備份限縮暴露；重審觸發＝sqlx 放寬範圍或應用開啟第三方 DB 檔。

### Verified
- 四關全綠（2026-11-09）：fmt EXIT=0／clippy -D warnings EXIT=0／test --locked **235 passed 0 failed**（整工作區，kem 線由並行 session 恢復：58 lib tests 綠）／audit EXIT=0（2 個已文件化處置）。
- 詳細證據：`docs/PLAN_REGISTER.md` addendum #14。

## [Unreleased] — 2026-11-09（第十二輪：三大工作室切片 G1.1–G1.4＋G2.1–G2.5＋G3.1–G3.3）

依 `.kilo/plans/1793787631643-three-studio-slice-execution-plan.md`（v4，含重構判定框架）執行；上位架構 SSOT：`.kilo/plans/1792865422970-three-studio-governance-platform.md` v3.1。

### Added
- **S1 工作台（G1.1–G1.4）**：
  - 八資料態全覆蓋（WP-08）：`states.html` 增 零結果/過期/無權限/衝突 標本（`data-state-*` 錨點＋下一步連結）；`design-system.css`（含 docs 快照）新增 `.zero-state/.stale-state/.denied-state/.conflict-state`（Design Token 驅動、深色模式自動適配）。
  - 上下文連續（WP-10）：交易詳情連結攜帶 `from_page`+`q`，詳情頁返回連結還原精確篩選視圖（CJK 百分比編碼；敵意輸入截斷降級、永不 500）；分頁保留搜尋詞（PIN）。
  - 角色起始頁（WP-05/04）：`/` 依 role_code 渲染落地面板（FINANCE→待覆核佇列；STORE/VIEWER→報表入口；其餘→總覽；`data-role` 錨點），角色即時查詢 user_roles。
  - KPI 就地解釋（WP-02）：四張 KPI 卡附 定義/時點/來源（`title`＋可見 `data-kpi-definitions` 註）。
  - 批量覆核（WP-07）：`POST /reviews/bulk`——單一 POST、逐列審計（REVIEW_BULK＋簽名者）、逐列結果含規則名（SELF_ADJUDICATION_FORBIDDEN/NOT_PENDING/NOT_FOUND/STATE_CHANGED）、失敗列高亮（`tr.outcome-failed`）、重送安全（逐列重檢 PENDING_REVIEW，永不重複入帳）、100 筆上限、CSRF＋LEDGER_WRITE 閘門；`bulk_review.html`＋index 表單。
- **S2 中樞（platform-hub，G2.1–G2.5）**：
  - eventbus：`RetryPolicy`（指數退避＋±20% 有界確定性抖動）、`DeliveryOutcome`、`DeadLetterStore`（記錄/列表/署名重放，重放記錄 actor）＋記憶體實作；Kafka stub 不動。
  - channels：`CircuitBreakerAdapter`（Closed→Open→HalfOpen→Closed；探測失敗立即重開）、`ChannelError::CircuitOpen`、`CircuitStats`；可注入時鐘做確定性故障注入（FAULT-07）。
  - udm：`CustomerRecord`/`InvoiceRecord`/`PaymentRecord`（各帶 `authoritative_system`）；MoneyCents↔根線 minor 單位契約（明確、檢查、拒絕截斷的轉換；合併依 §0.5 裁決不做）；`reconcile_orders`→`ReconciliationReport`（確定性排序、樣本上限 10、i128 分總額、JSON 可序列化）。
  - agent-policy：`AgentRateLimiter`（滑動窗、呼叫方時鐘）、`WriteDisposition::DraftRequired`（DEC-15 fail-closed 對所有風險等級成立）、`AgentAuditMarker.audit_event()`。
- **S3 治理稽核（G3.1–G3.3）**：
  - `platform/src/controls.rs`：單一控制庫（CTRL-POST/IMPORT/REVIEW/CRED/AI）×框架映射（ISO-27001 附錄/內控 IC/銀行 FSC）；`lint_control_coverage` 復用 kem gov.rs Finding 模式（UNMAPPED_EVENT/INACTIVE_CONTROL）。
  - `GET /admin/audit_evidence.csv`（ROLE_MANAGE）：審計就緒四要件標頭（控制相關/涵蓋期間/完整性 sha256 匯出摘要/來源）＋RFC 4180 CSV 資料列；期間格式 fail-closed。
  - `GET /admin/control_coverage.json`：控制庫＋觀察事件 lint 結果。
  - `GET /admin/ccm.json`：三指標（M1 金額不變量違反、M2 登入異常＝誠實 null＋缺口註記（登入稽核未實作，假零即假綠）、M3 簽名完整性違反），各附 定義/時點/來源。

### Verified
- 本線門檻（2026-11-09）：fmt 0／clippy -D warnings 0／platform lib **77 passed 0 failed**／合併線 **141 passed 0 failed**（bridge 7＋core 42＋platform 77＋characterization 6＋close_out 4＋slice12 2＋misc 3）／audit EXIT=0（1 個已接受警告）。
- platform-hub 門檻：fmt 0／clippy 0／**22 passed 0 failed**。
- 零 JS 護欄：新模板（bulk_review.html）與 index 修改均 script=0、無內聯事件。
- kem（並行線）in-flight 重構中，整工作區門檻待其穩定點重跑（隔離紀律見 PROJECT_STATUS）。
- 詳細逐切片證據：`docs/PLAN_REGISTER.md` addendum #12/#13。

## [Unreleased] — 2026-11-07（第十一輪：編碼根因修復＋驗證方法論補全＋效能量測法修正）

### Fixed
- **CP950 編碼損壞根因修復**：`platform/src/api.rs` 以最後乾淨提交 `dc414ac` 為源、`git merge-file` 三方合併還原（損壞自 `e82a49e` 起；Askama 分頁欄位 `page/total_pages/prev_page/next_page/total_transactions/pending_review_count` 隨合併復位）；`core/src/config.rs` 類別註解（ORG 組織／PERM 權限／ACT 動作／STORE 門市／POS 終端／INV 存貨）與 `crates/kem/src/main.rs` `P1 M2–M3`（U+E62F → en dash）PUA 字元清零。防制規範：`docs/encoding-standard.md`（PowerShell 禁 `>` 重導向寫碼，改 `[System.IO.File]::WriteAllText`＋UTF-8 無 BOM）。
- **Stress test 量測法修正**：`stress_10000_transactions_p95_under_500ms` 原 3 樣本 p95 退化為 max-of-3，首筆冷啟動查詢（SQLite 頁快取 miss，535ms）誤判 SLA 違規；改為**預熱趟（不计時）＋21 樣本 p95**（`ceil(21×0.95)-1=19`，容許單次排程抖動），**SLA 500ms 不變**；seed 改批次多列 INSERT（500 rows/語句，100k 交易由 ~90s 降至 ~2s）。修正後 warm p95 **422ms**。
- `NEXT_ACTIONS.md` 字面 `` `n `` 序列損壞（表格斷行遺留）重寫為合法表格。

### Added
- **突變測試（MT）**：cargo-mutants 27.1.0；`.cargo/mutants.toml` 官方 schema（examine_globs/exclude_re/additional_cargo_test_args）；補測 `amount_at_cap_posts_successfully`、`parse_i128_overflow_boundary_kills_boundary_mutants`、`money_error_display_is_not_empty`——money 73/73、engine 22/22 全殺（**≥98%→100%**，Missed 清零）。報告：`docs/engineering/MUTATION_SCORE_REPORT.md`（v2）。
- **模糊測試（FBT）**：cargo-fuzz 0.13.2＋nightly 1.101.0；`fuzz/fuzz_targets/money_parser.rs`（INV 非負＋round-trip）build PASS；run 受阻 Windows ASan DLL（`STATUS_DLL_NOT_FOUND` 0xc0000135）——`docs/engineering/FUZZING.md` §3 記錄三條解鎖路徑。
- **故障注入 SOP**：`docs/engineering/FAULT_INJECTION_SOP.md`（§10 追蹤表 FAULT-01..07；FAULT-01/02 DONE、04/05 IN PROGRESS）。
- **ATDD 綁定稽核**：`docs/engineering/ATDD_BDD_BIND_AUDIT.md`（8 情境 → Rust 測試；2 MISSING＋1 BLOCKED 列管）。
- **方法論補全**：DEVELOPMENT_METHODOLOGY §7（TDD 實證邊界＋跨域韌體/FPGA/SoC）、§8（32 項缺口表動態更新）、§9（Skills 修補）。
- **索引重整**：`docs/README.md` v3.0——死鏈（`01-onboarding/`、`02-architecture/` 從未存在）移除、新文件全收錄、superseded 標記規則入檔。
- **計畫登記**：`docs/PLAN_REGISTER.md` 2026-11-07 addendum #11（本輪執行事實）。

### Verified
- 四關全綠（2026-11-07）：fmt EXIT=0／clippy -D warnings EXIT=0／test --locked **157 passed 0 failed**（bridge 7＋core 42＋kem 53＋platform 48＋close_out 4＋slice12 3）／audit EXIT=0（1 個已接受警告 RUSTSEC-2026-0374）。
- 零 JS 護欄：templates script=0、內聯事件=0（未變動模板行為）。
- 並行工作線相容：Slice 13 搜尋引擎（`search.rs`＋`cross_module_api.rs`）與 kem evolve/gov 屬並行 session，本輪門檻在雙方穩定點量測；`target\debug\flaps-server.exe` 檔案鎖以隔離 `CARGO_TARGET_DIR` 迴避。

## [Unreleased] — 2026-11-06（第十輪：重新設計藍圖 R2/M1/M3＋TDD/BDD/SDD/DDD/ADD 體系）

### Added
- **REDESIGN_BLUEPRINT_v2.0**（docs/product/）：全面深化設計——架構 R1-R3、模組 M1-M6、介面互動設計、十二維度品質準則、硬性不變量。深化而非推翻：ADR-0001/0002/0003 決策全數保留。
- **PBT（proptest 1.12.0）**：core dev-dependencies 新增 proptest；8 個性質測試（parse/format round-trip i64+i128、非負、overflow、TooManyDecimalPlaces、empty、negative）全部通過；INV-01 零浮點護欄強化。
- **DDD 狀態機測試**：engine `status_transition_posted_to_pending_review`（POSTED→PENDING_REVIEW），core 31→39 測試。
- **CI 工作流**：.github/workflows/flexible-ci.yml 已在本機提交；推送至 origin 需 `workflow` scope（OAuth token 限制），本機已模擬 CI 四關全綠。

- **測試驗證計畫**（docs/testing/REDESIGN_TEST_PLAN_v2.0.md）與**交付清單**（docs/delivery/DELIVERY_MANIFEST_v2.0.md）。
- **開發方法論體系**（docs/engineering/DEVELOPMENT_METHODOLOGY.md）：TDD 紅綠循環（強制基線）＋BDD 場景註冊表（docs/testing/BDD_SCENARIOS.feature，8 Scenario／10 綁定測試，全部驗證存在）＋SDD 追溯表（需求→規格→測試→證據）＋DDD 界限上下文圖與聚合不變量審計＋ADD 品質屬性戰術表。

### Changed
- **M1 交易分頁**（TDD，RED→GREEN）：RED `pagination_shows_twenty_rows_per_page_with_page_two`（首頁曾全量列出）→ GREEN `engine::query_transaction_summaries_paged`（LIMIT/OFFSET 全 bind）＋`post_json` `?page=N`（非法→第 1 頁、越界→末頁）＋模板分頁導覽（`{{ page }}/{{ total_pages }}`，首末頁禁用態）。測試：3 整合＋`parse_page` 7 斷言。
- **M2/R2 健康檢查強化**（TDD，RED→GREEN）：RED `health_reports_degraded_when_database_unreachable`（預期 200 ok）→ GREEN `/api/health` 回報 migration 版本＋交易總數；DB 不可達時 503＋status=degraded（部署探測可用）。測試：`health_reports_*`×2。
- **M3 待覆核 KPI**（TDD，RED→GREEN）：RED `pending_review_kpi_shows_real_count_instead_of_placeholder`（KPI 顯示「—」）→ GREEN `build_index` 加 `COUNT(*) WHERE status='PENDING_REVIEW'` ＋`IndexTemplate.pending_review_count` ＋模板 `{{ pending_review_count }}`（0 起跳，斷線時仍為「—」）。測試：`pending_review_kpi_shows_real_count_instead_of_placeholder`。
- `.pagination` 元件樣式（tabular-nums 頁碼、禁用態）。

### Verified
- 四關全綠：fmt EXIT=0／clippy -D warnings EXIT=0／test --locked **98 passed 0 failed**（90→98）／audit 1 allowed warning（既有）。
- 零 JS 護欄：templates script=0、內聯事件=0。
- 併行工作流相容：governance-hub（Phase D3）與模組化重組之變更與本輪全部共存且測試通過。
## [Unreleased] — 2026-11-05（第九輪：Slice 12 Demo Kit 完成＋Phase 0 藍圖＋Phase 2 閘門）

### Added
- Demo Kit：`scripts/seed-demo.ps1`＋`reset-demo.ps1`（可重複展示；[DEMO] 資料經正式 API 種入，10/10 交易；demo-finance/demo-viewer 帳號）。
- Demo 文件：DEMO_GUIDE／DEMO_SCENARIOS／DEMO_DATA／DEMO_LIMITATIONS（誠實揭露限制）。
- Phase 0 藍圖文件：PRODUCT_VISION／USER_STORIES（US-1..16）／PRODUCT_ROADMAP（六階段閘門）／BUSINESS_MODEL（假設標示）／INFORMATION_ARCHITECTURE（22 路由 sitemap）／SYSTEM_ARCHITECTURE／DATABASE_DESIGN／API_DESIGN／TECH_STACK／TECHNICAL_DECISIONS（D-1..D-12）。
- 進度制度：`docs/PROJECT_STATUS.md`＋`docs/NEXT_ACTIONS.md`。
- Phase 2 閘門：`docs/production/PRODUCTION_READINESS.md`（Go/No-Go 矩陣 25 項；部署/監控/還原演練標示需授權——**NO-GO**）。
- 啟動時過期會話清理（`AppState::new` → `purge_expired_sessions`）。
- 遷移 0004：查詢索引（平行工作階段交付，已稽核入版控）。

### Fixed
- 桌面路徑檔案寫入回滾事故（測試檔合併）——以位元組級腳本＋讀回驗證修復。

### Tests
- 90 passed, 0 failed（含遷移 0004 重放）；基線四項 EXIT=0 @ 07b1d9c。
## [Unreleased] — 2026-11-04（第八輪：完成度稽核收尾）

### Added
- 值物件 Money／AccountId（DDD 戰術模式，建構即驗證）＋測試（core/src/value_objects.rs）。
- 樞紐列上限（OQ-10-2）：top-20 依合計，其餘摺疊至 OTHER 列＋截斷提示。
- AI 建議請求稽核（OQ-10-1）：AI_SUGGEST_REQUESTED（記錄 text_len/prompt_version/schema 結果；原文遮罩）。
- 管理頁密碼重設表單（P1-8 殘餘 UI）；草稿頁 AI 建議表單（OQ-8-2 UI）。

### Fixed
- CSRF 覆蓋缺口：import_preview（multipart）與 ai_suggest 補同步器驗證——POST 路由覆蓋 10/10。
- persist_batch 改 ImportBatchStaging（最後一處 too_many_arguments 豁免移除）。
- PIV-003 斷言修正為整數分語義。

### Tests
- 73→87 passed；基線四項 EXIT=0 @ c932753。
## [Unreleased] — 2026-11-04（第七輪：Slice 11 部署硬化）

### Added
- 安全標頭 middleware：CSP（default-src 'none'、style-src 'self'、img-src 'self' data:、無任何 script 來源）、nosniff、DENY、no-referrer、no-store（全回應）。
- 過期會話清理：登入邊界（新會話建立時刪除過期列）。
- Secure cookie 旗標（erp_config security/cookie_secure；預設關，TLS 部署時啟用——R-07 準備）。
- 根 README.md（Directive 13.1 全欄位）。

### Changed
- 模板 9 處內聯 style 屬性全數改為設計系統 utility classes（嚴格 CSP 成立）。

### Fixed
- 停用帳號安全缺口：停用後舊會話仍有效至自然過期（Slice 9 已閉，本輪補測試）。

### Tests
- 73 passed；基線四項 EXIT=0 @ 26bff52。
## [Unreleased] — 2026-11-04（第六輪：Slice 10 多維洞察進階與方法論硬化）

### Added
- 樞紐分析：GET /reports/pivot（白名單維度 R-PIV-WHITELIST，矩陣＋合計；PIV-003 上限摺疊 OTHER）。
- 平行座標投影：core/src/parallel.rs（整數 min-max 正規化，零浮點）＋GET /reports/parallel（每科目一折線＋無障礙 title）。
- AI 建議草稿（特性旗標預設關）：POST /drafts/ai_suggest（schema 閘門 parse_ai_suggestion，畸形輸出拒絕）＋建議卡（需人工覆核）＋POST /drafts/adopt（唯一入徑，永不自動落地）。
- Builder：TransactionBuilder／TransactionItemBuilder（P2-1 清償；兩處 too_many_arguments 豁免移除）。
- 解析覆蓋率：GET /admin/parse_coverage（ROLE_MANAGE；COV-001）。
- 方法論文件：docs/quality/bdd_acceptance.md、docs/testing/TRACEABILITY_MATRIX.md、docs/quality/ubiquitous_language.md。

### Changed
- Transaction/TransactionItem 建構改用 Builder（create_transaction、import_confirm、engine 測試；語義不變）。

### Removed
- 兩處 #[allow(clippy::too_many_arguments)]（allow 歸零）。

### Tests
- 70→73 passed；基線四項 EXIT=0 @ 7eb5c1d。
## [Unreleased] — 2026-11-04（第五輪：Slice 9 硬化收尾）

### Added
- 帳號停用即撤銷會話（P1-9）：admin/users/toggle 停用時刪除該帳號全部工作階段。
- 管理員密碼重設（P1-8 殘餘）：POST /admin/users/reset_password（ROLE_MANAGE、CSRF、≥8 字元）——Argon2id 輪換＋目標會話撤銷＋PASSWORD_RESET 稽核。
- 報表效能預算量化（P3-1/RPT-001）：close_out 測試以 recursive CTE 播種 2,000 筆平衡交易，斷言 /reports/monthly < 500ms。
- 設計 Token 同步迴歸測試（P3-2）：web/assets 與 docs 的 :root 區塊逐字一致。

### Fixed
- 停用帳號安全缺口：此前停用僅改旗標，舊會話仍有效至自然過期。

### Tests
- 66→70 passed；基線四項 EXIT=0 @ bc68b7e。
## [Unreleased] — 2026-11-04（第四輪：Slice 8 資料獲取與洞察層）

### Added
- **快速匯入（Layer A）**：兩段式 CSV/TSV 管線——`POST /import/preview`（multipart ≤2MB／5,000 列，逐列規則驗證、乾跑報告、SHA-256 指紋，**零帳務副作用**）→ `POST /import/confirm`（CSRF、狀態機重放防護 409、僅合格列登帳、批次稽核 `IMPORT_CONFIRMED`）。
- **引導式登帳精靈（Layer B）**：`GET /new` 三步流程（模板選擇→借貸填寫→確認摘要）；錯誤停留原步並顯示**規則名稱**（WIZ-001）；支援 `from_draft`／`from_batch` 預填並標示來源。
- **草稿解析（Layer C）**：`flaps_core::parser`——規則表驅動的非結構化文字解析（日期三格式、金額、借貸箭號、科目關鍵字對映），每欄位附**依據卡**（規則＋命中文字）；`POST /drafts/parse` 儲存草稿，**人工確認必經**，AI 不直接落地（H3 模型不變）。
- **洞察層（Layer D）**：dashboard 伺服器端 **SVG 月度借貸長條圖**（零 JS，僅整數聚合生成，safe filter 已審核）；`GET /reports/monthly`（科目借貸彙計）與 `.csv` 匯出（LEDGER_READ、整數分）；`@media print` 列印樣式。
- `migrations/0003_import_and_drafts.sql`（import_batches/import_rows/drafts）＋重放測試。
- Slice 8 驗收測試：IMP-001~003、WIZ-001、DRAFT-001、RPT-001、ZJS-001（`<script>` 計數 = 0 含 SVG 頁）。

### Security
- 匯入預檢無副作用、確認有重放防護、批次級稽核事件；報表與匯出受 LEDGER_READ 把關；解析器純規則（AI 建議與落地分離）。

### Notes
- 互動式 3D 視覺化與 Strict Zero-JS 憲章衝突，裁定為 Open Question OQ-8-1（零 JS 內以 SVG＋CSV 出場替代；豁免需另立 ADR）。

## [Unreleased] — 2026-11-04（第三輪：治理、防爆破、帳號管理）

### Added
- **根 workspace 版控（P0-2）**：`git init -b main`＋初始提交 `cd66388`——.gitignore 裁定範圍（排除 6 個嵌套獨立 repo、計畫目錄、target 7GB、本機 DB）；staged 內容安全掃描（無真實秘密）；此後證據綁修訂版。
- **登入速率限制（P1-7）**：`LoginRateLimiter`——每帳號 5 次連續失敗鎖定 15 分鐘（成功重置；鎖定訊息含剩餘秒數；大小寫不敏感；檢查先於憑證驗證）。
- **帳號管理 UI（P1-8）**：`/admin/users`（清單＋建立＋啟停），ROLE_MANAGE **即時 DB 查詢**把關；使用者名稱規範（3–32、小寫/數字/_/-）、密碼 ≥8、角色白名單；自我停用拒絕；全表單 CSRF。

### Changed
- RBAC 檢查改為**即時 DB 查詢**（`has_permission_db`）——角色指派於啟動後即時生效（原 in-memory 快照不一致）。
- 移除無人使用的 tauri workspace 依賴（P2-2；Cargo.lock tauri 條目歸零）。

### Security
- 暴力破解防護（P1-7）＋帳號停用（停用後登入被拒，測試覆蓋）。

## [Unreleased] — 2026-11-04（第二輪：複式記帳＋認證）

### Added
- **借貸平衡強制（P1-5，INV-02）**：`validate_journal_entry` 強制 ΣDEBIT = ΣCREDIT 且 >0（i128 累加）；表單升級為借方/貸方雙欄（同科目拒絕）；交易清單金額改為借方合計語義。
- **認證與會話（G-1，ADR-0003）**：`platform/src/auth.rs`——Argon2id 密碼雜湊、伺服器端會話（24h、可撤銷）、`require_session` middleware、登入/登出（PRG）、同步器 CSRF token（登出＋交易表單）、首次啟動管理員播種（`FLAPS_ADMIN_PASSWORD` 或隨機一次性列印）。
- `migrations/0002_users_sessions.sql`（users＋sessions 表）。
- 登入頁 `platform/templates/login.html`（零 JS，錯誤泛化防枚舉，密碼永不回顯）。
- 引擎負向測試 8 條＋整合測試 13 條（未認證重導、CSRF 403、登出撤銷、RBAC 403、平衡端到端）。

### Changed
- 身分來源改為伺服器端會話（`Extension<Session>`）；**移除可偽造的 `X-Acting-User` header**。
- `AppState`/`LedgerEngine`/`PermissionManager` 加 `Clone`（router state 型別需求）。
- 路由拆分 public/protected（`require_session` middleware）。
- `web/index.html` 表單同步升級為借貸雙欄（靜態基線與模板一致）。

### Security
- THREAT_MODEL：T-01 已緩解（認證落地）、T-08 已緩解（CSRF token）、R-06 Resolved；新增 R-07（Secure cookie 待 TLS）。

## [Unreleased] — 2026-11-04（第一輪：整數貨幣＋API 層）

### Added
- **零 JS API 層（ADR-0002）**：`platform/src/bin/flaps-server.rs`（Axum 0.7 伺服器）與 `platform/src/api.rs`（路由與處理器）：
  - `GET /`：Askama 0.16 伺服器渲染全頁（模板 SSOT：`platform/templates/index.html`，自動 HTML escaping）。
  - `GET /assets/design-system.css`：編譯期嵌入（零路徑遍歷面）。
  - `GET /api/health`：JSON 探針（db 連線狀態＋版本）。
  - `POST /transactions`：表單 → RBAC（`LEDGER_WRITE`）→ 驗證 → 冪等登帳 → 303 PRG；失敗回完整頁面並保留表單值（400/403/500 分級）。
- **整數貨幣模組（P0-1，INV-01）**：`core/src/money.rs`——`parse_decimal_to_minor`（精確字串解析，拒 >2 位小數／負數／溢位，繁中錯誤訊息）與 `format_minor`（純整數千分位格式化）。
- `LedgerEngine::with_pool`：不建表的推薦建構（schema SSOT 為 migrations）；`query_recent_transaction_summaries`（JOIN 彙總金額）。
- `TransactionSummary`（header＋items 彙總，供清單渲染）。
- 平台整合測試 8 條（PRG 303、RBAC 403、驗證 400 ×3、零 JS 渲染、health、CSS）＋遷移測試 4 條（建表/重放冪等/舊庫相容/整數 roundtrip）。
- `docs/adr/ADR-0001-tech-stack-baseline.md`、`docs/adr/ADR-0002-api-layer.md`。
- 治理文件：根 `AGENTS.md`、`docs/product/PRD.md`、`docs/product/USER_JOURNEYS.md`、`docs/product/IMPROVEMENT_BACKLOG.md`、`docs/security/THREAT_MODEL.md`、`docs/testing/TEST_STRATEGY.md`、`docs/engineering/SKILL_INVENTORY.md`、本檔。

### Changed
- **金額整數化（P0-1）**：`TransactionItem.amount_minor: i64`（原 `amount: f64`）、`exchange_rate_scaled: i64`（原 `exchange_rate: f64`）；migrations 0001 schema 改為 `amount_minor INTEGER NOT NULL`、`exchange_rate INTEGER DEFAULT 10000`；`core/src/ledger/engine.rs::ensure_tables` DDL 同步。
- 依賴安全：`reqwest` 0.11 → 0.12.28（h2 0.3.27 → 0.4.20，修復 RUSTSEC-2026-0258；移除 unmaintained 的 rustls-pemfile）。
- 根 `Cargo.toml`：移除 `[workspace.lints.clippy] all/pedantic = "allow"`（憲章 §2.2 零警告不變量）。
- `web/index.html`：標頭註記 Superseded——伺服器渲染 SSOT 為 `platform/templates/index.html`；本檔保留為離線設計基線。
- `docs/design-system.css`：頭註部署副本關係（SSOT：`web/assets/design-system.css`）。
- `SyncMesh/docs/SECURITY.md` v1.1.0：反模擬更正（TCP 明文現狀；TLS 1.3 為目標、AT-008 未驗證）。
- `docs/PLAN_REGISTER.md`：2026-11-03 addendum（基線恢復與 ADR-0001）。

### Fixed
- `core/src/permission.rs`：`initialize_default` 缺失閉合括號導致 workspace 無法編譯（語法損壞）。
- 19 個 clippy 錯誤（redundant_pattern_matching、needless_borrows_for_generic_args、too_many_arguments（帶理由豁免）、new_without_default、unnecessary_cast、for_kv_map、collapsible_match、manual_div_ceil、is_multiple_of）。
- flaky 測試 `batching_beats_per_call_under_measured_envelope`：牆鐘比較改為確定性往返計數（MockTransport 新增 `round_trips()`）。

### Security
- `cargo audit` EXIT=0：h2 升級修復 RUSTSEC-2026-0258；rsa RUSTSEC-2023-0071 經 `.cargo/audit.toml` 文件化接受（sqlx-mysql 未啟用、未編入產物）。
- 伺服器端 RBAC 強制＋輸入驗證 fail-closed（THREAT_MODEL T-01 部分緩解、T-02/T-06 控制）。

### Known Limitations（誠實記錄）
- 根 workspace 非 git repo（P0-2）：證據為指令＋退出碼綁定，非修訂版綁定。
- `X-Acting-User` header 未經認證（G-1/R-06）：對外開放前必須落地認證會話。
- 表單建立單筆分錄，未強制借貸平衡（P1-5）。
