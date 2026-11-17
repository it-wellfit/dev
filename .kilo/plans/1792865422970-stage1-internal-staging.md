# EKPI Platform Stage 1 內網單機 Staging 上線計畫

- **日期**：2026-11-17
- **範圍裁定**：Stage 1 內網單機 staging（PR-02 方向：單一內網 Windows 主機、原生二進位、手動啟動、僅授權非敏感資料）
- **服務託管裁定**：Stage 1 **不註冊 Windows 服務**，採「登入主機後前台手動啟動＋Ctrl+C 停機」；服務化列 Stage 2（D1）
- **基準**：register 最新 **R31**（2026-11-13）：canonical `scripts/check.ps1` exit 0、workspace **582 passed / 0 failed / 0 ignored**、docs 53 required/866 links/141 files、secrets 356 files
- **SSOT**：`ekpi-platform/docs/operations/staging-checklist-draft.md`（Stage 1 段）、`docs/operations/production-readiness.md`（PR-02）、`docs/quality/phase-gates.md`
- **性質**：只涵蓋 repo 內可執行、可本機驗證的工作；主機指派、服務帳號、防火牆、排程器註冊、TLS/LAN、簽章均為外部 ops/安全動作。

## 目標

讓 Operations 能照 runbook 在指定內網 Windows 主機完成 Stage 1 staging：修正部署事實與執行期設定缺陷、補齊最小設定面、產出手動啟動版 runbook 與證據模板，並通過 canonical 閘門。

**非目標**：正式生產（Phase B：IdP/SSO/MFA、WORM、多租戶、off-host、TLS/LAN）；A1 真實 pilot（DEC-01）；Windows 服務化實作；PostgreSQL 接線實作；程式碼簽章。

## 已查證事實（含對先前摘要與過時文件的修正）

1. `ekpi-platform/Dockerfile` **存在**，但建的是 `ekp_web`（`apps/web` 骨架），不是 `governance_web`；先前「無 Dockerfile」的說法錯誤，實際是「建錯目標」。無 `.dockerignore`。
2. `governance_web`（`platform/src/bin/governance_web.rs:79-90`）埠由 `EKPI_WEB_PORT` 控制，但**解析失敗靜默退回 3000**；位址寫死 `127.0.0.1`。
3. **執行期資料路徑是編譯期絕對路徑**：`governance_web.rs:3664-3666` 的 `platform_data_dir()` 用 `env!("CARGO_MANIFEST_DIR")/data`；換主機後 `/search` 會「文件庫載入失敗」。這是 staging 實質阻塞點。
4. `governance_web` 用 JSONL ledger（`EKP_GOVERNANCE_DATA`）＋ JSONL approval store（`EKP_APPROVAL_DATA`，`jsonl_approval.rs:595-600`）；PostgreSQL adapters 存在但未接線；`EKPI_STORAGE=postgres` 對工作台無效。
5. **Windows 服務化不可直接用 `sc.exe create`**：console 二進位無 SCM `ServiceMain`，註冊後啟動會 error 1053。草案的「sc.exe create；手動啟動」自相矛盾。
6. CI workflow **不在 HEAD**（`.github/` 只有 dependabot.yml、ISSUE_TEMPLATE、PULL_REQUEST_TEMPLATE；`a697f47` 移除）；B4 Blocked。
7. `governance_web` 已有 `/healthz`（liveness-only）、`/ready`（只檢查 ledger＋auth，不含資料目錄）、`/metrics`（`governance_web.rs:146-148, 538-562`）；但 `docs/deployment/monitoring.md:13-15` 仍寫「No metrics endpoint / No health/readiness endpoint」，**過時**。
8. **CSRF 已實作**（`governance_web.rs:748-842, 961-1007, 1052-1057`：session 用 `X-CSRF-Token`／`ekpi_csrf` cookie 雙重提交；登入用一次性 hidden field＋cookie）；`docs/deployment/security.md:41` 的「No dedicated CSRF token」**過時**。
9. `EKPI_COOKIE_SECURE=1` 會加 `Secure`；Stage 1 為 http://127.0.0.1 明文，部分瀏覽器不回傳 Secure cookie → 登入可能失敗；Stage 1 保持 unset，TLS 列 Stage 2。
10. `scripts/drill-runtime-data.ps1` **已存在**（S14/IMP-021）：真實 server、雙 JSONL 鏈、真 kill、真檔案遺失、真還原、HTTP 觀察鏈徽章；T6 直接 reuse。它寫死 `target\debug\governance_web.exe`，可加 `-Exe` 參數支援 release。`docs/operations/local-backup.md:77` 與 `backup-log.md:3` 仍寫「No runtime-data restore drill / IMP-021 未執行」，**過時**。
11. `scripts/backup.ps1` 只產 git bundle＋還原鑽演（bundle 落 `%USERPROFILE%\EKPI-backup\bundles`），**不含** ledger/approvals JSONL；`scripts/schedule-backup.ps1` 用 `Register-ScheduledTask`（需管理員）。
12. **C6 已關閉**：R27（2026-11-13）/`docs/adr/0004` 已把 governance payload 型別整併進 `ekp_domain`；Postgres 接線不再受 C6 阻擋。
13. `governance_web` bin 在 `platform/Cargo.toml` 標記 `test = false`（迴避 host Application Control 對新 test harness 的攔阻）→ **bin 內 unit test 不會執行**；T2 的設定解析邏輯必須抽到 lib 才能測試。
14. `rust-toolchain.toml`：channel `1.99.1`、profile minimal、components clippy/rustfmt。
15. `AGENTS.md:39` 的 env 清單缺 `EKP_APPROVAL_DATA`／`EKPI_WEB_PORT`，**過時**；`docs/architecture/threat-model.md:11,36,49` 的 bind/CSRF 敘述需隨 T2 更新。
16. Phase A：A1 `Not met`（DEC-01）、A2 `Partial`（AT-01–AT-31 `NOT RUN`）、A6/A10 `Partial`；Phase B `Blocked`。

## 已裁定決策

| ID | 決策 | 理由 |
|---|---|---|
| D1 | Stage 1 手動啟動，不註冊服務 | 避免 1053；符合「手動啟動、不 auto-start」；服務化需 `windows-service`＋SCM 握手＋安全審查，列 Stage 2 |
| D2 | Stage 1 一律 JSONL storage | hash chain 已測試；staging 僅合成/授權非敏感資料；Postgres 列 Stage 2 |
| D3 | 新增 `EKPI_BIND_IP`（**僅接受 loopback IP**）＋沿用 `EKPI_WEB_PORT`；非 loopback **直接拒絕啟動**，Stage 1 無 opt-in | 防止誤暴露；`EKPI_BIND_ADDRESS` 在 adapters `AppConfig` 已是「IP:port」語意，platform 另用 `EKPI_BIND_IP` 避免同名歧義 |
| D4 | 新增 `EKPI_DATA_DIR` 執行期解析；未設時沿用編譯期路徑 | 修正 `CARGO_MANIFEST_DIR` 換主機失效；保留本機開發行為 |
| D5 | CI workflow 檔案先進 repo，執行驗證標 `Not run` | 無 push 授權；檔案存在 ≠ CI 通過 |
| D6 | Dockerfile 只做正確性修復，**不是 Stage 1 部署產物** | PR-02 為 Windows 原生二進位；loopback-only 設計下容器對外不可用，需 Stage 2 非 loopback 核准 |

## 任務清單（依序執行）

### T1. `Dockerfile` 正確性修復（非 Stage 1 部署路徑）
- `cargo build --release --locked --bin governance_web`；runtime 複製 binary＋`platform/data`；`ENV EKPI_DATA_DIR=/app/data`、`EKPI_BIND_IP=127.0.0.1`、`EKPI_WEB_PORT=3000`；移除 `apps/web/static`、`templates` 依賴（CSS 由 `include_str!` 嵌入、maud 編譯）。
- 新增 `.dockerignore`（`target/`、`.git/`、`artifacts/`、`backups/`、`*.bundle`），避免 `COPY . .` 把建置產物塞進 build context。
- 註記：此映像在 loopback-only 設計下僅容器內可達；Stage 1 不採用；對外需 Stage 2 核准與程式碼 opt-in（目前無）。
- 驗證：`docker build`（若可用）；被 host Application Control 阻擋（`os error 4551` 有前例）記 `Blocked`。

### T2. `governance_web` 執行期設定硬化（核心程式變更）
- 新增 `platform/src/runtime_config.rs`（`#![forbid(unsafe_code)]`）：
  - `pub struct RuntimeConfig { pub bind_ip: IpAddr, pub port: u16, pub data_dir: PathBuf }`
  - `pub fn resolve(bind_ip: Option<&str>, port: Option<&str>, data_dir: Option<&str>) -> Result<RuntimeConfig, String>`（純函式，便于測試；不做 env 讀取）
  - `pub fn from_env() -> Result<RuntimeConfig, String>`（讀 `EKPI_BIND_IP`／`EKPI_WEB_PORT`／`EKPI_DATA_DIR`，呼叫 `resolve`）
  - 規則：bind 預設 `127.0.0.1`；只接受 loopback（`IpAddr::is_loopback`）；傳入 `IP:port` 給出可執行錯誤（「僅接受 IP，埠請用 EKPI_WEB_PORT」）；port 預設 3000、拒 `0`、非數字 fail-closed；`EKPI_DATA_DIR` 明確設定但不存在 → `Err`；未設且預設路徑不存在 → 回傳 warning（不阻啟動，`/search` honest error）。
- `platform/src/lib.rs` 加 `pub mod runtime_config;`；`platform/src/bin/governance_web.rs` 改為呼叫 `from_env()`，失敗印 `啟動設定錯誤：…` 並 exit 2；啟動成功印 `bind=… port=… data_dir=… ledger=… approvals=… storage=jsonl`（不含密碼）。
- 測試（放 `runtime_config.rs` 的 `#[cfg(test)]`，因 bin `test=false`）：預設值、IPv4/IPv6 loopback、非 loopback 拒絕、`IP:port` 拒絕、非法/空 port、port 0 拒絕、明確 data dir 不存在、明確 data dir 存在、預設 data dir 缺失 warning。

### T3. CI workflow（檔案先進 repo）
- 新增 `.github/workflows/ci.yml`：`windows-latest`；`dtolnay/rust-toolchain@master`（`toolchain: 1.99.1`、`components: clippy, rustfmt`）或 `rustup show` 讀 `rust-toolchain.toml`；`Swatinem/rust-cache@v2`；步驟：`cargo fmt --all -- --check`、`cargo clippy --locked --workspace --all-targets -- -D warnings`、`powershell -ExecutionPolicy Bypass -File scripts/check.ps1`（含 test/selftest/schema/search）。
- 記錄：本機無法驗證（無 push 授權、`gh` 認證未確認）→ `Not run`；B4 仍 Blocked，但「無 workflow 檔案」子項關閉。

### T4. Postgres 接線決策（只記錄，不實作）
- `staging-checklist-draft.md` Stage 1 加「Storage decision」：Stage 1 一律 JSONL；`EKPI_STORAGE=postgres` 列 Stage 2，前置：`governance_web` 改用 `GovernanceRepository` port＋approval port 抽象、真 PG 18 整合測試、migration 帳齊備、ledger/approval parity 測試。C6 已關閉（R27/ADR-0004），不再列為前置。
- 不把 `EKPI_STORAGE` 寫進 Stage 1 runbook 設定步驟。

### T5. Stage 1 runbook 具體化（手動啟動版）
更新 `docs/operations/staging-checklist-draft.md` Stage 1 段：
- Preconditions：clean revision、bundle＋restore drill PASS、僅合成/授權非敏感資料、主機欄位經核准（staging scope）、以**專用非管理員操作者帳號**登入。
- Build：`cargo build --release --locked`；記錄 binary SHA-256、`rust-toolchain.toml` 版本。
- 產物移轉：以核准的檔案通道傳送 `governance_web.exe`＋`platform/data/{docs.json,rules.json,processes.json}`；**兩端各算 SHA-256 並比對**；記錄 data manifest。
- 環境變數（範例）：
  - `EKPI_LOCAL_AUTH_USER`（ASCII；無預設帳號）、`EKPI_LOCAL_AUTH_PASSWORD`（8–128；**不要直接輸入 PowerShell 主控台**，用 `Read-Host -AsSecureString`＋`Marshal` 轉 plain，或先 `Set-PSReadLineOption -HistorySaveStyle SaveNothing`；結束後 `Remove-Item Env:EKPI_LOCAL_AUTH_PASSWORD`）、`EKPI_LOCAL_AUTH_ROLE=approver`
  - `EKP_GOVERNANCE_DATA`、`EKP_APPROVAL_DATA`、`EKPI_DATA_DIR`、`EKPI_BIND_IP=127.0.0.1`、`EKPI_WEB_PORT=3000`
  - **`EKPI_COOKIE_SECURE` 保持 unset**（明文 HTTP）；**不設**任何非 loopback bind。
- 資料目錄：必須為**本機固定磁碟**（不可用 SMB/網路磁碟機，檔案鎖不可靠）；目錄權限限操作者帳號可讀寫。
- 啟動：前台手動啟動，例如 `.\governance_web.exe *> D:\EKPI\stage1\logs\stage1.log`；**不註冊服務、不 auto-start、不開防火牆埠**；只能有一個實例（第二個會因 ledger/approval 鎖 refused，屬預期）；log 會持續成長，Stage 1 以手動輪替為主。
- Smoke（逐項記錄命令＋輸出）：
  - `curl.exe -s -o NUL -w "%{http_code}" http://127.0.0.1:3000/healthz` → 200（注意：PS 5.1 的 `curl` 是 `Invoke-WebRequest` 別名，必須寫 `curl.exe`）
  - `/ready` → 200 且 body `"status":"ready"`（`/ready` 不含資料目錄）
  - `/metrics` → 200
  - 瀏覽器登入一次（synthetic creds）→ dashboard
  - `/search`：`curl.exe -s -G --data-urlencode "q=變更執行前，須完成風險評估並取得核准紀錄" http://127.0.0.1:3000/search` → 有結果、無「文件庫載入失敗」
  - `/audit/chains` → 雙鏈 verified
  - 一條端到端：建立專案 → 連結證據 → 審查 → 提交核准 → 決策 → `/events` 事件數增加
  - `cargo run --release --bin ekp_platform -- --selftest`（或原生 CLI）→ 49 assertions PASS；**不要與工作台同時對同一 ledger 跑 `--audit-log`**（會搶鎖）
  - 記錄主機時間（`Get-Date`）與 skew；approval due_at 以 unix 時間計，skew 過大需記錄。
- 停機：在前景 console 按 **Ctrl+C**（不要直接關視窗；關視窗可能等於 hard kill）。
- Rollback：停程序 → 還原上一版 binary＋data＋ledger/approvals → 重跑 smoke → 記錄 keep/rollback。
- 停止條件：真實客戶資料、缺證據、健康檢查失敗、`/search` 載入失敗、ledger 鎖拒絕、無法解釋的日誌錯誤。
- 所有 checkbox 註明「checked box without evidence is not done」；證據寫入 `docs/operations/stage1-staging-record.md`；失敗/資料異常記 `docs/operations/incident-log.md`。

### T6. 備份與還原（本機可做的部分）
- **reuse** `scripts/drill-runtime-data.ps1`（可加 `-Exe` 參數支援 release）：跑一次，記錄 `DRILL PASS`／exit 0；它涵蓋真 kill、檔案遺失、還原、雙鏈 HTTP 徽章。
- `scripts/backup.ps1`：跑一次 bundle＋還原鑽演；記錄 bundle 路徑、SHA-256、`RESTORE DRILL PASSED` 到 `docs/operations/backup-log.md`。
- JSONL 備份：**先 Ctrl+C 停機、等程序完全退出**，再把 `EKP_GOVERNANCE_DATA`＋`EKP_APPROVAL_DATA` **成對**複製到受保護 ACL 的備份目錄，各記 SHA-256；還原鑽演：複製到 temp、以另一 port 啟動、`/audit/chains` 與 approvals 徽章驗證、停機。
- `scripts/schedule-backup.ps1`：記錄參數（`-BackupDir`、`-DataDir`、`-Time`）與「需管理員」；實際註冊 = ops，標 `Not run`。
- RPO 說明：Stage 1 為「上次備份」還原，無 PITR；off-host/加密/RPO/RTO 屬 DEC-14。

### T7. 文件與證據收口

| 檔案 | 修改 |
|---|---|
| `docs/operations/stage1-staging-record.md`（新增） | Stage 1 證據範本：revision、binary/data SHA-256、env（不含密碼）、smoke 輸出、drill 結果、bundle、keep/rollback 決策；狀態 `Not run` |
| `docs/operations/staging-checklist-draft.md` | Stage 1 手動啟動步驟、Storage decision、證據指向 record；狀態仍為 draft（未核准、未執行） |
| `docs/operations/monitoring.md` | 更正 current state：`/healthz`（liveness-only）、`/ready`（ledger＋auth）、`/metrics`（aggregate）、request ID、access log；仍無 SLO/tracing/alerting |
| `docs/operations/local-backup.md` | 更正 IMP-021 敘述：runtime-data drill 已於 2026-11-09 PASS（合成資料）；補「先停機、雙檔成對複製」 |
| `docs/operations/backup-log.md` | 新增 T6 一列；狀態列移除「runtime JSONL 無演練」的過時句 |
| `docs/deployment/install.md` | staging 差異（env、bind、data dir、手動啟動、無 TLS） |
| `docs/deployment/config.md` | env 表新增 `EKPI_BIND_IP`、`EKPI_WEB_PORT`、`EKPI_DATA_DIR`、`EKP_APPROVAL_DATA`；「Local web configuration」更新 |
| `docs/deployment/security.md` | 更正 CSRF 敘述（double-submit＋header）、env 清單、bind 邊界（預設 loopback、非 loopback 拒絕、無 TLS） |
| `docs/deployment/monitoring.md`（若與 operations 版重複則以 operations 為準） | 可只留連結，避免兩處漂移 |
| `docs/deployment/troubleshooting.md` | 新增 staging 除錯：port in use、data dir 不存在、ledger 鎖拒絕、torn final line、Secure cookie 登入失敗 |
| `docs/architecture/threat-model.md` | bind 改為 `EKPI_BIND_IP` loopback-only 驗證；TH-03 改寫為「已有多重提交 token，仍非企業 CSRF/身分」 |
| `AGENTS.md` | env 清單補 `EKP_APPROVAL_DATA`／`EKPI_WEB_PORT`／`EKPI_BIND_IP`／`EKPI_DATA_DIR`；註明 `DATABASE_URL` 只被未接線的 adapters `AppConfig` 讀取 |
| `.env.example` | 補新變數佔位（不得放真實密碼） |
| `README.md` | run 段補 `EKPI_BIND_IP`／`EKPI_DATA_DIR`／`EKP_APPROVAL_DATA` |
| `docs/traceability/register.md` | 新增 **R32**（T1–T7 證據；未驗證項標 `Not run`/`Blocked`），更新頂部「Current release evidence」 |
| `docs/quality/phase-gates.md` | A6/A10 的 next action 指向新 runbook/record；**不得**把任何列改為 `Met`（除非有連結證據） |
| `docs/README.md` | 新增 `stage1-staging-record.md` 的文件地圖條目 |
| `docs/operations/release-evidence-bundle.md` | 只加交叉連結到 Stage 1 record，不把 staging 證據混進 Phase A pack |

## 驗證計畫（canonical 閘門，全部必過才提交）

1. `cargo fmt --all -- --check`
2. `cargo clippy --locked --workspace --all-targets -- -D warnings`
3. `cargo test --locked --workspace`（0 failed / 0 ignored；基準 R31 為 582，新增測試後只要求全綠，不寫死數字）
4. `powershell -ExecutionPolicy Bypass -File scripts/check.ps1` → exit 0 且印 `ALL CHECKS PASSED`
5. `powershell -ExecutionPolicy Bypass -File scripts/check-docs.ps1` → PASS
6. T2 新測試：runtime config resolver 全綠
7. `powershell -ExecutionPolicy Bypass -File scripts/drill-runtime-data.ps1` → `DRILL PASS`
8. `powershell -ExecutionPolicy Bypass -File scripts/backup.ps1` → `RESTORE DRILL PASSED`
9. 本機手動 smoke：以 T5 環境變數啟動 → health/ready/metrics → 登入 → `/search` 有結果 → `/audit/chains` verified
10. T1 `docker build`（若可用；被阻擋記 `Blocked`）
11. 提交訊息依 repo 慣例（`<type>: <subject>`）；不 push、不建 PR 除非使用者明示。

## 風險與誠實邊界

- **1053**：已改手動啟動；runbook 明確禁 `sc.exe`；服務化列 Stage 2。
- **`CARGO_MANIFEST_DIR` 換主機失效**：T2 以 `EKPI_DATA_DIR` 修正；staging 必設。
- **Cookie Secure 與明文 HTTP**：Stage 1 unset；誤設會登入失敗，runbook 標註。
- **Docker 非 Stage 1 產物**：loopback-only 設計下容器對外不可用；不得用 `--network host` 或 `0.0.0.0` 繞過。
- **Application Control**：debug/release 二進位與 docker build 可能被阻擋（`os error 4551`）→ 記 `Blocked`，非 PASS。
- **CI 無法本機驗證**：workflow 檔案存在 ≠ CI 通過；B4 仍 Blocked。
- **PSReadLine 密碼外洩**：直接輸入 `$env:...PASSWORD="..."` 會進 history；runbook 要求 `Read-Host -AsSecureString`＋停用 history。
- **Hard kill / 關視窗**：可能留下 torn final line，下次啟動 fail-closed；runbook 要求 Ctrl+C，異常後先驗鏈、必要時還原備份並記 incident。
- **網路磁碟／SMB**：檔案鎖不可靠，runbook 要求本機固定磁碟。
- **單一實例**：第二個實例會被 ledger/approval 鎖拒絕；這是預期，不是故障。
- **未簽章二進位 / AV 掃描**：Stage 1 以 SHA-256 對帳；簽章列 Stage 2；AV 鎖檔時先排除或記錄。
- **時鐘 skew**：approval SLA 以 unix 時間計；記錄主機時間，偏差過大需處理。
- **日誌內容**：access log 可能含 query 字串；僅 synthetic 資料，且日誌目錄需 ACL 保護。
- **主機欄位、排程器註冊、防火牆、TLS、服務帳號** = 外部 ops/安全動作，保持 `<blank>` 與 `Blocked`。
- **不得宣稱**：staging 已存在、生產就緒、企業身分驗證、多租戶、WORM、off-host 備份完成、TLS 已建。

## 外部阻擋項（本計畫不執行，僅記錄）

| 項目 | 決策 ID | 狀態 |
|---|---|---|
| 產品 owner / pilot workflow 指派 | DEC-01 | 未指派 |
| IdP/SSO/MFA、租戶隔離 | DEC-12 | 未決 |
| 資料治理/保留 | DEC-13 | 未決 |
| off-host 備份/RPO/RTO | DEC-14 | 未決 |
| 運維 owner/服務等級 | DEC-16 | 未指派 |
| 負載模型/容量 | DEC-19 | 未決 |
| 資安 owner/滲透測試 | DEC-20 | 未指派 |
| 部署主機/網路邊界/TLS | PR-02 | 方向已定、欄位空白 |
| Windows 服務化設計（SCM ServiceMain 或核准 wrapper） | Stage 2 | 未設計 |
| 程式碼簽章 / 二進位信任 | Stage 2 | 未設計 |

## 完成定義（本計畫的 DoD）

- T1–T7 的 repo 內可執行部分完成；canonical 閘門全綠、docs check PASS、register R32 有證據列。
- Stage 1 runbook 可讓 Operations 在不改程式的前提下完成「build → 設 env → 手動啟動 → smoke → 停機 → rollback」。
- 所有未執行/未核准項均標 `Not run`/`Blocked`；無任何 `PASS`／`Met` 的無證據宣稱。
- 實際主機部署、排程器註冊、服務化、Postgres 接線、簽章、Phase B 不在本計畫完成範圍。