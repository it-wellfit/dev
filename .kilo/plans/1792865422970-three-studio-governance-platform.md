# 三大工作室與治理平台架構 v3.0 — 企業級完整架構計畫

> 計畫編號：1792865422970 | 版本：v3.0（2026-11-07 全面事實核對版）
> 狀態：**計畫文件（未來式）**；實作狀態以各專案證據為準，不以本檔宣稱。
> 唯一權威：本檔（`1792865422970-three-studio-governance-platform.md`）
> 前檔：`1793782615513-wellfit-unified-plan-v2.md`（上游統一計畫 v2）| v2.1（本檔前版，已被本版取代）
> 適用範圍：`Dev/ekpi-platform/`（主產品 OEKAP）、`Dev/platform-hub/`、`Dev/Wellfit/`、根 `Dev/` workspace（FlapsErp RPA 支援線）、`Knowledge/`（合規 SSOT）

## 目錄

- [§0 文件管制與閱讀契約](#0-文件管制與閱讀契約)
- [§1 願景與範圍邊界](#1-願景與範圍邊界)
- [§2 現況基線與事實核對（C-01..C-23）](#2-現況基線與事實核對c-01c-23)
- [§3 企業級價值流全圖（VS-01..VS-09）](#3-企業級價值流全圖vs-01vs-09)
- [§4 三大工作室架構（3 核心＋規模觸發擴編）](#4-三大工作室架構3-核心規模觸發擴編)
- [§5 職責、決策權與交接契約（HO-01..HO-08）](#5-職責決策權與交接契約ho-01ho-08)
- [§6 協作機制與升級路徑（ESC-1..ESC-4）](#6-協作機制與升級路徑esc-1esc-4)
- [§7 目標架構](#7-目標架構)
- [§8 資料與整合模型](#8-資料與整合模型)
- [§9 端到端流程規格](#9-端到端流程規格)
- [§10 簽核引擎規格](#10-簽核引擎規格)
- [§11 工作台與 API 契約](#11-工作台與-api-契約)
- [§12 稽核與合規對照](#12-稽核與合規對照)
- [§13 權限模型](#13-權限模型)
- [§14 資安基線](#14-資安基線)
- [§15 可靠性與維運 Runbook（RB-01..RB-10）](#15-可靠性與維運-runbookrb-01rb-10)
- [§16 資料治理與持續改善](#16-資料治理與持續改善)
- [§17 演進路徑與路線圖（PHASE-G*/SLICE-G*）](#17-演進路徑與路線圖phase-gsliceg)
- [§18 風險登記冊（RSK-G*）](#18-風險登記冊rsk-g)
- [§19 未決決策與阻塞（DEC-21+／OQ-GOV-*）](#19-未決決策與阻塞dec-21oq-gov-)
- [§20 品質閘門總表（QG-*）](#20-品質閘門總表qg-)
- [§21 驗證計畫](#21-驗證計畫)
- [§22 追溯矩陣](#22-追溯矩陣)
- [§23 參考文件](#23-參考文件)
- [§24 變更紀錄與執行順序](#24-變更紀錄與執行順序)

---

## §0 文件管制與閱讀契約

### §0.1 目的與規範力

- 本檔為「三大工作室治理平台」企業級架構與演進計畫的唯一權威計畫文件。
- 本檔用**未來式**描述尚未實作的目標；所有「已實作／已接線／已驗證」陳述必須能回指至 §2 或 §22 的具體檔案行號；不能回指者，一律視為未來目標，不得當作成果引用。
- 本檔不取代各專案的 canonical 文件：需求（`ekpi-platform/docs/governance/02-requirements-v0.1.md`）、資料模型（`06-data-and-state-model-v1.md`）、決策登記（`08-decision-register.md`）、Gate（`docs/quality/phase-gates.md`）、KB（`Knowledge/`）。本檔只能**引用**它們，不得複寫其條文；複寫即產生第二來源，違反 `ekpi-platform/docs/README.md`「一條規則一個 canonical 來源」。

### §0.2 閱讀一點通（依讀者身份）

| 讀者 | 必讀 | 選讀 |
|---|---|---|
| 專案擁有者／決策者 | §1、§19、§17、§24 | §18 |
| 工作室 1 業務人員 | §4、§5、§9（VS-01..VS-09 相關列）、§11 | §13 |
| 工作室 2 技術人員 | §2、§7、§8、§10、§11、§15、§20、§21 | §22 |
| 工作室 3 治理／稽核人員 | §2、§12、§13、§16、§20 | §22 |
| 實作代理（將計畫轉為程式） | 本檔全部＋§24 執行順序 | — |

### §0.3 ID 命名空間

- 沿用既有命名空間：`FR-`、`NFR-`、`QC-`、`AT-`、`DEC-`、`BASE-`、`ADR-`、`PR-`、`CTR-`、`G1–G5`（ekpi-platform 領域關卡）、`G0–G7`（IMS）。
- 本檔新增命名空間（**已逐一 grep 驗證無碰撞**；理由見 §24 C-23）：
  - `VS-01..VS-09` — 價值流域（D1–D9 原草稿因 `D1/D2/D3` 已被 KB `depth_tier` 與 `wellfit-unified-plan-v2` 佔用，改為 VS）
  - `IND-1..IND-6` — 產業疊層
  - `SCALE-1..SCALE-4` — 營運規模層級
  - `HO-01..HO-08` — 交接契約
  - `ESC-1..ESC-4` — 升級路徑
  - `RB-01..RB-10` — 維運 Runbook
  - `QG-01..QG-16` — 品質閘門
  - `RSK-G01..RSK-G14` — 治理平台風險（`RSK-E*` 已被 FlapsErp RSK 表佔用）
  - `PHASE-G1..PHASE-G6`、`SLICE-G01..SLICE-G16` — 路徑階段與切片（`S1..S6` 已被 redesign-v2-master-plan 佔用；`S1..S11` 已被 traceability register 佔用；`SC-01..SC-23` 已被 PHASE1_IMPLEMENTATION_README／FREE_AI_SYSTEM_UPDATE 佔用）
- 新決策沿用 `DEC-21+`（`DEC-01..DEC-20` 已被 `08-decision-register.md` 佔用）→ 須併入 canonical `08-decision-register.md`，不得另外建決策表。
- 新開放問題用 `OQ-GOV-01..OQ-GOV-14` → 須併入 `08-decision-register.md`，不得自建 OQ 表。

### §0.4 誠實性鐵律（全檔適用）

1. 不宣稱未實作能力（見 §2 C-04 孤兒目錄、C-16 偽造安全宣稱）。
2. 拒絕與未接線必須顯式（`501 NOT_IMPLEMENTED`、fail-closed；見 §10、§11）。
3. 金額／數量禁用浮點（`i64` 最小單位；見 §8）。
4. 欄位以名稱解析，禁止序位退讓（`ColumnContract`；見 §8）。
5. 跨 repo 禁止 path dependency（唯一既存例外：`Dev/web/src-tauri/Cargo.toml` 對 `wellfit_bankpay`/`wellfit_core` 的 path dep，列為技術債，見 C-11）。
6. 契約事實禁網路推測（KB `research-protocol.md` §1.1）。

---

## §1 願景與範圍邊界

### §1.1 願景

為百貨、零售、製造、生產、銷售與行銷的企業提供一條**端到端可追溯、可稽核、可持續演進**的價值鏈：
上游採購 → 商品與規格主資料 → 生產／加值 → 倉儲物流 → 零售通路／門市 → 銷售與訂單 → 行銷與內容合規 → 售後服務與客訴 → 合規、稽核、持續改善。

每個環節的業務事件、決策、證據與結果，都寫入同一可驗證鏈條；治理不是事後補文件，而是每個交接處的內建控制。

### §1.2 四個核心假設（H1–H4，待 DEC-01 以真實試點驗證）

| 代號 | 假設 | 驗證法 |
|---|---|---|
| H1 | 資訊取得成本高：經營者看不懂哪個數字可信 | 試點以儀表板決策時間／退件率變化量測（FR-07） |
| H2 | 工作流程斷裂：記帳與核對分離、證據散落 | 試點以交接等待時間、退回率量測（§20 QG-08） |
| H3 | AI 信任不足：AI 建議必須可追溯、可否決 | advisor-only 決策留痕；人工覆核率（§10） |
| H4 | 整合維護成本高：每家平台欄位不同 | 8 欄契約化＋契約差異報告（§8）；維運工時趨勢 |

> H1–H4 僅為假設，不得當作已成立的價值宣稱。`FR-07` 要求指標有定義／來源／期間／樣本數／核准目標；未核定者儀表板顯示 N/A（見 `08-decision-register.md` DEC-09）。

### §1.3 範圍內（IN_SCOPE）

- 三大工作室協作制度（§4–§6）：職責邊界、交接契約、升級路徑、共同問題解決。
- 企業級價值流 VS-01..VS-09（§3）與其端到端流程規格（§9）。
- 架構：治理面（ekpi-platform）、整合面（platform-hub）、執行面（Wellfit）三層耦合（§7）。
- 治理介面：零 JS 工作台與 API v1（§11）、稽核鏈（§12）、權限（§13）、安全（§14）、維運（§15）。
- 演進框架：版本、feedback loops、指標、roadmap、風險、治理控制（§16、§17、§18、§20）。
- 品質閘門與驗證（§20、§21）、追溯矩陣（§22）。

### §1.4 範圍外（OUT_OF_SCOPE，明確不做）

- 不把 `postgres/` 適配器宣稱為已實作（除真實 SQL 的 `identity.rs` 外，`governance.rs`／`approval.rs`／`channels.rs` 仍為 stub，見 C-06）。
- 不把 `/api/v1/*` 宣稱為已接線（2/13 已接線，11/13 仍為 501，見 C-01）。
- 不宣稱 Beta 版或正式版安全性（KB ADR-0010/0012 要求實測控制值域，見 §12）。
- 不做線下實體（PHY）與人事（HR）領域的系統實作（僅作流程與控制登記）。
- 不做多租戶服務承諾（DEC-12 指派的平台未決，見 §13）。

---

## §2 現況基線與事實核對（C-01..C-23）

本章為 v3.0 的「誠實性根基」。每條皆以 `檔案:行號` 為據，可用 `grep`  reproducer 驗證。

### §2.1 技術基線（四套，2026-11-07 逐一查證）

| # | Workspace | 位置 | Rust | Web | 資料庫 | 備註 |
|---|---|---|---|---|---|---|
| B1 | ekpi-platform | `Dev/ekpi-platform/Cargo.toml` | 2024 / Resolver 3 / 1.99 | Axum 0.8.9 (form), Askama 0.16.1 | SQLx 0.9.0 + PostgreSQL 18.6 | 主產品 |
| B2 | platform-hub | `Dev/platform-hub/Cargo.toml` | 2024 / Resolver 3 / 1.99 | —（無 web） | SQLx 0.9.0 + PostgreSQL | 整合面，5 crates |
| B3 | 根 Dev | `Dev/Cargo.toml` | 2021 / Resolver 2 | Axum 0.7, Askama 0.16 | SQLx 0.8 + SQLite 8.0.1 | FlapsErp RPA 支援線；含 `web`（lib）與 `src-tauri`（Tauri） |
| B4 | Wellfit | `Dev/Wellfit/Cargo.toml` | 2021 / Resolver 2 | —（`wellfit.exe` CLI） | rusqlite 0.32 (SQLite) | 9 個 crate（見 §2.2） |

> **基線紀律**：四套 workspace 並存，**不得**在本計畫內宣稱已統一。統一與否屬 ADR-0001 決策事項，見 §19 DEC-22／OQ-GOV-01。

### §2.2 事實更正表（C-01..C-23）

| 代號 | 錯誤宣稱（舊檔位置） | 查證事實與證據 | 本計畫處理 |
|---|---|---|---|
| **C-01** | v2.1 §6.2「API 端點（11 端點，已實作）」表列 9 端點 | `platform/src/governance_web.rs:141-194` 註冊 **13 條 API v1 method-routes**；**2 條已接線回真實資料**（`GET /api/v1/workflows`、`GET /api/v1/approvals`，見 L183-193 的 `api_workflows_list`／`api_approvals_list`）；**11 條回誠實 501**（`POST /api/v1/workflows`、`POST /api/v1/workflows/{id}/publish`、`POST /api/v1/approvals`、`POST /api/v1/approvals/{id}/decide`、`POST /api/v1/approvals/{id}/escalate`、`POST /api/v1/events/replay`、`GET /api/v1/channels`、`GET /api/v1/channels/health`、`GET /api/v1/users`、`GET /api/v1/settings`、`GET /api/v1/search`） | §11.2 正表：13 條、2 接線、11 × 501；§17 規畫逐條收斂 |
| **C-02** | v2.1 §5.1「程式化呼叫｜Bearer API key（設定頁產生）」附 curl 範例 | `docs/contracts/approval-api.md`（CTR-API-001）明載認證為**本機 session cookie**，「**無** Bearer/API key（不存在的能力不得宣稱）」；`/api-docs` 頁要求同 session | §11.1：刪除 Bearer 宣稱；程式化存取一律以 session cookie 為準；`curl` 範例一律以 cookie 傳遞 |
| **C-03** | v2.1 §7.1「Phase D3 \| /api/health 擴充（outbox、hub_reachable）\| 3 health 測試通過」 | 全 `Dev/` grep `EKPI_HUB_URL\|hub_reachable\|outbox_provisioned` 僅命中計畫檔本身；`Dev/platform/src/api.rs:287` 的 `health()` 只回 `{status, db, service:"flaps-server", version}`（4 欄位）；`ekpi-platform/platform/src/api.rs` 僅有 `/health`、`/api/v1/cases/:id/decide`、`/api/v1/cases/:id/snapshot`（固定回 `comply`／`not_found`），且 `docs/backend/README.md:72-78` 明載 **never mounted** | §7.1 回退為「未做」；`ekpi-platform/platform/src/api.rs` 永遠不得掛載（QG-01 鎖定） |
| **C-04** | v2.1 §7.1「Phase D1 \| approve_transaction stub 移除 TODO」 | stub 位於**未編譯的孤兒目錄** `Dev/web/src-tauri/src-tauri/commands.rs:156`（`approve_transaction`）、L177（`reject_transaction`）；`Dev/web/src-tauri/Cargo.toml`（package `flaps-erp-web`、自帶 `[workspace]`）**無 `[[bin]]`** → 預設 `src/main.rs` 為實際編譯入口；`src/main.rs:147` 的 `invoke_handler` 僅列 `approve_release_item`（無 `approve_transaction`） | §7.1 回退為「未做」；孤兒目錄列為清理工作項（§17 SLICE-G07） |
| **C-05** | v2.1 §2.1「approval_web.rs 為其內部分類標記（API v1 端點），非另一套前端」—表述含混，易誤讀為獨立檔 | `governance_web.rs` 約 3027 行；路由：核心 `Router::new()`（L140-171）＋ `approval_routes` 子 Router（L173-194），`core.merge(approval_routes)`（L196）；middleware 鏈：`require_session` → `validate_csrf_token` → `rate_limit_guard` → `same_origin_guard` → `request_context`（L197-207） | §11.1 正確描述路由拓樸；任何文件不得將其拆解為「兩套前端」 |
| **C-06** | v2.1 §7.1「Phase C1–C5 \| …postgres 適配器… \| 全通過」暗示 Postgres 已落地 | `crates/adapters/src/postgres/governance.rs` 有 **9 個 `todo!()`**（L163/177/187/194/203/216/226/233/248，runtime panic 風險）；`postgres/approval.rs` 有 **8 處 `Err(GovernanceError::Storage("not yet implemented"))`**（L42/51/60/71/75/79/83/87，fail-closed）；`postgres/channels.rs` 有 **1 處**同樣（L36）；**真實 SQL 僅 `postgres/identity.rs`**（帳號／session／throttle 皆真實） | §7.1 回退為「部分」；postgres 落地列 PHASE-G4（§17）；在完成前一律 JSONL（SLICE-G05）；`todo!()` 列 QG-02 閘門 |
| **C-07** | 「ChannelAdapter trait 唯一」暗示 | `ChannelAdapter` **重複定義**：`ekpi-platform/crates/domain/src/channels.rs`（`serde_json::Value` 型）vs `platform-hub/crates/channels/src/lib.rs`（typed `UnifiedOrder`／`InventoryRecord`） | §19 DEC-21 提案 ADR-0004；未裁決前以 `platform-hub` 為準（typed），ekpi-platform 側標註為 adapter 骨架 |
| **C-08** | `Dev/ekpi-platform/AGENTS.md`「root workspace 既存在…There is no root Cargo workspace」自相矛盾；「Edition 2021」與實際 2024 矛盾；`DATABASE_URL` 未被使用宣稱與 `crates/adapters/src/configuration.rs` 矛盾 | `ekpi-platform/Cargo.toml` 確為 workspace（resolver 3、edition 2024、rust-version 1.99，6 members）、`platform/src/lib.rs` 為庫；`configuration.rs` 確實使用 `EKPI_STORAGE`＋`DATABASE_URL` | 以 manifest 為真；此 AGENTS.md 段落列清理工作項（§17 SLICE-G08）；本計畫 §2.1 表取代其敘述 |
| **C-09** | `Dev/ekpi-platform/docs/quality/phase-gates.md` §SC-01「No database」；§constraints「Edition 2021」 | `crates/adapters/src/postgres/`（`identity.rs` 真實 SQL、`pool.rs` PoolSettings、`governance.rs` 含 9 `todo!()`）；`migrations/` 有 3 檔 5 表；`ekpi-platform/Cargo.toml` edition 2024 | Phase-gate §SC-01 與 §constraints 以本計畫 C-09 為準，屬未同步文件 |
| **C-10** | v2.1 §0／memory「platform-hub `apps/cli` binary」 | `platform-hub/Cargo.toml` members **僅 5 crates**（udm/channels/eventbus/adapters-momo/agent-policy），**無 `apps/cli`** | §7.1 刪除 `apps/cli` 陳述；CLI 註冊列 OQ-GOV-02（DEC-21） |
| **C-11** | 「跨 repo 無 path dependency」宣稱 | `Dev/web/src-tauri/Cargo.toml:17-18` 有 `wellfit_bankpay = { path = "../../Wellfit/crates/wellfit_bankpay" }` 與 `wellfit_core` path dep（根 workspace `web` member） | §7 架構決策：保留現行 path dep 並登記例外；收斂方向見 DEC-22；不在本計畫內強制消除 |
| **C-12** | `/api-docs` 頁（`governance_web.rs` L2031-2053）宣稱「所有 API 端點目前一律回 501，不提供任何資料」且「工作台為主介面；API 端點已註冊路由但尚未接線」；表格只列 8 行，把已接線的 `GET /api/v1/workflows`、`GET /api/v1/approvals` 也標為「未實作（一律 501）」 | 6 條被漏列：`GET /api/v1/channels/health`、`GET /api/v1/search`、`POST /api/v1/workflows/{id}/publish`、`POST /api/v1/approvals`、`POST /api/v1/approvals/{id}/escalate`；另 `docs/contracts/approval-api.md`（CTR-API-001）正表寫明 2 條已接線，與頁面矛盾 | §11.4：`/api-docs` 頁內容由契約檔單一來源產生（或標註為契約快照日期）；未修前頁面不得被引用為契約事實 |
| **C-13** | `/settings` 頁宣稱「PostgreSQL：僅在 EKPI_STORAGE=postgres + DATABASE_URL 啟用」；`AGENTS.md` 宣稱 `DATABASE_URL` 未被使用 | `crates/adapters/src/configuration.rs` 實作 `AppConfig::from_env()`（L67）讀取 `EKPI_BIND_ADDRESS`、`EKPI_STORAGE`、`DATABASE_URL` 等；但**全 workspace 無任何呼叫 `AppConfig::from_env`**（grep 僅命中定義與測試）；供應的 binary `platform/src/bin/governance_web.rs` 寫死 `default_ledger_path()`（L12）與 `AuthStore::new(3600)`（L24），**不讀取 AppConfig**，故 `EKPI_STORAGE=postgres` 在已發布 binary 無效 | §11.5：`/settings` 頁標註「PostgreSQL 尚未在供應 binary 啟用」；供應 binary 接線列 PHASE-G3（§17）；同時修正 C-08 兩邊矛盾以 AppConfig 存在但未接線為準 |
| **C-14** | 「ledger 驗證通過」暗示 Postgres 完整性檢查可用 | `crates/adapters/src/postgres/governance.rs:251-253` 的 `verify_integrity()` **無條件回 `Ok(())`**（永遠「完整」）；`snapshot()`（L148-152）回 `GovernanceSnapshot::default()`（空快照） | §12 列為稽核完整性 P0 風險；PG 適配器未完成前，稽核頁一律以 JSONL 帳鏈為據，不得以 PG 為據 |
| **C-15** | 「登入角色」單一語意 | `Role` **重複定義**：`ekp_domain::auth::Role`（crates/domain/src/auth.rs:24，附 `satisfies()`、`Permission` 4 種＋`minimum_role()`）與 `ekp_platform::local_auth::Role`（platform/src/local_auth.rs:41，PBKDF2 600,000 iter）；`governance_web` 用 local_auth，`adapters/configuration.rs` 用 domain auth | §13：登入用 local_auth；authorization 以 domain `Permission::minimum_role()` 為準；雙 Role 合併列入 PHASE-G3（DEC-23），未合併前不得以「角色」一詞跨層互換 |
| **C-16** | `Dev/web/src-tauri/src/main.rs` 宣稱資安檢查通過 | `run_security_check()`（L73-74）回**固定 JSON** `cert_pinning:true, wdac_enabled:true, csrf_protection:true, xss_protection:true`；`run_connection_check()`（L68-69）回 `two_way_ssl:true, cert_status:"Valid"`；bankpay 全回 `MOCK123`／`SUCCESS (Mock)`；`get_account_balance()`（L133）回固定 `500000.0` | §14 禁止引用 `src-tauri` mock 作資安證據；§17 列 mock 移除（或標註 `mock_mode:true` 已在 JSON 內） |
| **C-17** | v2.1 §7.1「Phase D2 \| f64 殘留收斂 \| 全通過」 | `Dev/web/src/ledger.rs:10`（`balance: f64`）、L18（`amount: f64`）；`Dev/web/src/dashboard.rs:6`（`total_volume: f64`）、L9（`avg_processing_time: f64`）；`Dev/web/src-tauri/src/state.rs` DashboardData f64；`Dev/web/src-tauri/src/main.rs:133` `get_account_balance() -> f64`。故 D2 僅「core＋platform」完成，`web` crate 未收斂 | §8 標明 `web` crate 為活碼殘留；§17 SLICE-G07 收斂 `web` crate f64 |
| **C-18** | `Dev/web/src-tauri/tauri.conf.json` 建置設定可用 | `"frontendDist": "../src"`（L9）指向 Rust 原始碼目錄（誤設定，無前端靜態產物） | §17 列為清理工作項；不作部署保證 |
| **C-19** | `Dev/ekpi-platform/docs/architecture/README.md`「no server-side database」 | `crates/adapters/src/postgres/`、`migrations/`、`configuration.rs` 皆存在（見 C-06） | 以程式碼為準；該頁 stale，列文件清理（§17 SLICE-G08） |
| **C-20** | 「NFR-08/09/10 未定義」（DEC-19 引用 NFR-08/09、DEC-20 引用 NFR-10） | `02-requirements-v0.1.md` 僅列 NFR-01..NFR-07（bullet）；**expanded 定義在** `05-functional-spec-v1.md` §6（NFR-01..NFR-11）；`10-traceability-baseline.md` 亦列 NFR-01..NFR-11；`04-acceptance-test-matrix.md` 以 NFR-08/09/10 為 AT-29/30 之據 | 本計畫不另立 NFR 意義；追溯矩陣（§22）引用 `05-functional-spec-v1.md` §6 為 NFR-01..NFR-11 之正據 |
| **C-21** | `Dev/ekpi-platform/apps/web` 命名誤導（「apps/web 最薄弱層」被誤認為工作台） | `apps/web` 為獨立 crate `ekp_web`（自有 router：`/`、`/health`、`/submit`；templates 僅 `home.html`），與 `governance_web` 工作台無關 | §11.6 明確區隔；`apps/web` 列為未來 either 移除或整合決策（OQ-GOV-09） |
| **C-22** | 「Wellfit CLI 9 個子命令」 | `crates/wellfit_cli/src/main.rs:92` `enum Commands` 實為 **10** 個：`status`、`shopee`、`reports`、`mtrix`、`audit`、`spec`、`access-control`、`jit`、`bankpay`、`shadow` | 本計畫以 10 個為準；AGENTS.md 文件列同步清理 |
| **C-23** | 早期計畫草稿的 ID 命名空間（`D1–D9`、`I1–I6`、`S1–S4`、`H1–H8`、`RSK-*`）與現有 ID 並存即安全 | 已 grep 實證碰撞：`D1/D2/D3` 已被 `Knowledge/compliance/iso-27001/iso-27001-control-library-method.md` 的 `depth_tier` 佔用；`D1..D9` 已被 `1793782615513-wellfit-unified-plan-v2.md` 的 UI 維度佔用；`S1..S4` 與 `S1..S6` 切片、`S1..S11` 追溯列、`S1..S4` 嚴重度皆碰撞；`H1..H4` 已被 `手冊-A-F.md` 的 H1/H2/H3 與 `H-1..H-6` 硬門檻佔用；`SC-01..` 已被根 `PHASE1_IMPLEMENTATION_README.md` 佔用；`RSK-E1..E9`、`RSK-01..07` 已被 FlapsErp／管理文件佔用；`RP-01..07` 已被管理文件佔用 | 本計畫改用已驗證無碰撞的前綴：`VS-*`、`IND-*`、`SCALE-*`、`HO-*`、`ESC-*`、`RB-*`、`QG-*`、`RSK-G*`、`PHASE-G*`、`SLICE-G*`、`DEC-21+`、`OQ-GOV-*` |

### §2.3 誠實能力清單（截至 2026-11-07）

**已驗證可用（可引用）**
- `ekpi-platform`：CLI（`ekp_platform`）與 `governance_web` binary；`governance_web.rs` 3027 行；`/ready`（`ready.rs`）、`/healthz`、`/metrics`；13 HTML 頁（含 `/approvals` 等）＋ 13 API v1 僅 2 條已接線；CSRF 雙提交（`governance_web.rs:583-649`、`validate_csrf_token`）；本機帳密 PBKDF2-HMAC-SHA256 600,000 iter（`local_auth.rs:26`）＋ CSPRNG token（`TOKEN_BYTES=32`，`SALT_BYTES=16`，L28-29）；`;fallback` 的 session 3600s；同源守衛；rate limit；`same_origin_guard`；要求的 dive：`is_public_path` 僅 `/healthz`、`/metrics`、`/login`、`/assets/` 公開（`governance_web.rs:667`）。
- `crates/domain`：`workflow.rs`（Urgency 4 階＋`sla_seconds()`、ConditionExpr 6 變體＋And/Or/Not、Stage、TimeoutAction{Escalate/AutoReject/AutoApprove/Cancel}、WorkflowDefinition::validate、TaskState 6 態＋`is_terminal`/`transition`、TaskAction 5 種、WorkflowError{InvalidTransition/InvalidDefinition/InvalidSubmission/NoEscalationPath}）；`auth.rs`（Role 4 階＋`satisfies()`、Permission 4 種＋`minimum_role()`）；`governance.rs`（`GENESIS_HASH="GENESIS"`、EventPayload 9 變體、`QUALITY_CHECKLIST: [QualityCheckDefinition; 29]`）；`hashing.rs`（SHA-256）。
- `crates/application`：`ports.rs`（`GovernanceError{Validation/NotFound/Conflict/Storage}`；`ApprovalRepository`／`ChannelRepository`／`EventStore`／`AgentAdvisor`）、`approval.rs`（8 個 use case：create/publish workflow、submit/decide task、expire overdue、list workflows/tasks、agent advice）。
- `crates/adapters`：`jsonl_ledger.rs`（`Ledger` 附 SHA-256 帳鏈與 advisory lock）；`jsonl_approval.rs`（`ApprovalStore` 附獨立 hash chain `ekpi-approval-v1`）；`postgres/identity.rs` 真實 SQL；`postgres/pool.rs`（`PoolSettings` 預設 max 10 / acquire 10s / idle 600s / lifetime 1800s，`migrate()` 用 `sqlx::migrate!("../../migrations")`，`is_reachable`）；`identity.rs` 記憶體 `UserStore`／`SessionStore`（`MAX_FAILED_LOGINS=5`、`LOCKOUT_WINDOW_SECS=300`）。
- `platform-hub`：5 crates；`udm`（`MoneyCents(i64)`、`Decimal4(i64)`、`Provenance`、`OrderItem`、`UnifiedOrder`、`InventoryRecord`、`ProductRecord`，`MoneyCents(600)="6.00"`）；`channels`（`ChannelAdapter`、`FormatDeriver`、`ColumnContract::validate_header`、`ChannelError`）；`eventbus`（`OutboxEvent`、`InboxEvent`、`EventStore` trait、`KafkaEventStore` 空槽、`EventBusError`）；`adapters-momo`（`momo_order_contract()`、`PendingContractAdapter` skeleton）；`agent-policy`（`RiskLevel`、`ApprovalDecision`、`DataClassification`、`AgentRecommendation`、`AutoApprovalGate::advisory_only()`（enabled=false、confidence_threshold=1.0）、`decide()` 失敗閉鎖）。
- `Wellfit`：9 crates（core/sync/bankpay/cli/integration/shopee/mtrix/reports/assist）；三方對帳 `three_way.rs`（`EkpiDecision`／`EventBackboneRecord`／`WellfitExecution`／`ThreeWayStatus{Consistent/Mismatch/Missing}`／`IsolationEntry`／`ThreeWayReport`／`ThreeWayReconciler::reconcile()`；規則：EKPI=事件骨幹；approved⇒executed；rejected⇒rejected|isolated；違規或缺側一律 quarantine，永不自動修正）；`wellfit_integration` lib（coordination／reconciliation／switching／switching_drill／three_way 五模組）；`wellfit_core` 的 `contract.rs`（`ColumnContract`／`ColumnMap`）、`ledger.rs`（`IdempotencyLedger`）、`jit_permission.rs`（`JitPermissionManager`）、`release_gate.rs`（`ReleaseState` 7 態，含 `DispatchUncertain`）、`access_control.rs`（`PermissionLevel` 4 階、`Classification` 4 階、`Permission`、`SodConstraint`、`Role`、`AccessContext`、`AccessRequest`、`AccessRecord`、`AccessController`）；CLI 10 子命令（C-22）。
- `Knowledge`：ADR-0009..0012（0012 IMS 權威：G0–G7、9 角色、crosswalk 18 欄）；9 角色（roles.csv：EXEC_SPONSOR、ISMS_OWNER、PROCESS_OWNER、ASSET_OWNER、CONTROL_OWNER、LEGAL_PRIVACY、SEC_IT_ENG、INTERNAL_AUDIT、PMO）；12 process areas（process-areas.csv：FIN/OPS/HR/IT/PHY/LOG/RET/ECS/SVC/MKT/SUP/GOV）；65 項 compliance templates（`soa-93-controls`、`crosswalk`、`control-library`、`risk-register`、`evidence-index` 等計 32 組 schema+csv＋README）；12 vocab CSV。
- 根 `Dev/`：`platform/src/api.rs` 完全接線零 JS Axum 0.7 伺服器（`/`、`/transactions`、`/admin/users*`、`/import*`、`/drafts*`、`/new*`、`/reports/monthly*`、`/reports/pivot`、`/reports/parallel`、`/drafts/ai_suggest`、`/drafts/adopt`、`/admin/parse_coverage`、`/api/health`；§C-08 同名不同物）；`core/src/ledger/account.rs:57`（`get_balance()->i64`）；`platform/src/ai/decision_gating.rs`（amount_minor／exchange_rate_scaled i64）。

**未完成／不可宣稱（禁引用為能力）**
- Postgres 治理／核准／通道適配器（C-06、C-14）。
- API v1 11 條未接線端點（C-01）。
- `/api-docs` 頁內容（C-12）。
- `/settings` 頁 PostgreSQL 宣稱（C-13）。
- `AppConfig::from_env` 未接線（C-13）→ 供應 binary 不讀取 EKPI_STORAGE/DATABASE_URL。
- `Dev/web` crate 內 f64（C-17）。
- `Dev/web/src-tauri/src/main.rs` 全部 mock JSON（C-16）。
- `Dev/web/src-tauri/src-tauri/` 孤兒目錄（C-04）。
- `Dev/web/src-tauri/tauri.conf.json` `frontendDist: "../src"`（C-18）。
- `Architecture/README`「no server-side database」（C-19）。
- 4 套 workspace 未統一（B1-B4）。

---

## §3 企業級價值流全圖（VS-01..VS-09）

### §3.1 價值流域定義（9 域）

| 代號 | 價值流域 | 範圍／活動 | 直接產出 | 治理交點 |
|---|---|---|---|---|
| VS-01 | 上游採購與供應商管理 | 供應商准入、價格核定、採購單生成、到貨驗收、對帳 | 購貨單、驗收紀錄 | SUP 程序區；需簽核（§10） |
| VS-02 | 商品與規格主資料（MDM） | SKU 命名、條碼、標示法欄位、規格版本 | 主資料版本＋checksum | GOV 受控；需審查 |
| VS-03 | 生產與製程（BOM／工單／批次追溯） | BOM 維護、工單下達、批次與原料追溯 | 批次紀錄 | 需電子簽章 |
| VS-04 | 倉儲與物流（效期／FEFO） | 入庫、效期分層、FEFO 出貨、盤點 | 庫存變動事件 | 需複驗 |
| VS-05 | 零售與通路營運 | 門市 POS、百貨專櫃、通路價量、退換貨 | 銷售事件 | 需稽核 |
| VS-06 | 銷售與訂單 | 訂單擷取（Shopee／momo／PChome／Cyberbiz）、轉單、結算 | 訂單事件、結算單 | 需三方對帳 |
| VS-07 | 行銷與商品內容（廣告法／宣稱佐證） | 訴求文案、宣稱佐證連結、上架審查 | 內容版本＋佐證清單 | 需法遵審查 |
| VS-08 | 售後服務與客訴 | 客訴受理、RMA、CAPA、賠償核定 | 客訴單、CAPA | 需 CAPA 閉環驗證 |
| VS-09 | 合規、稽核與持續改善 | 定期稽核、控制自評、差異管理、改善 | SoA、風險登錄、KPI 基線 | 需獨立覆核 |

### §3.2 端到端主線（上游 → 下游）

```
VS-01 採購
  │ (購貨單 + 供應商資格)
  ▼
VS-02 商品主資料 (SKU/條碼/標示法版本)
  │ (版本化主資料, SHA-256 登錄)
  ▼
VS-03 生產 (BOM/工單/批次追溯)
  │ (批次 + 原料來源)
  ▼
VS-04 倉儲物流 (效期/FEFO)
  │ (庫存事件)
  ▼
VS-05 零售通路
  │ (銷售事件)
  ▼
VS-06 銷售訂單 + 結算 (三方對帳閉環)
  │ (已對帳結算)
  ▼
VS-07 行銷內容合規 (宣稱佐證)
  │ (內容版本 + 佐證)
  ▼
VS-08 售後/客訴/CAPA
  │ (客訴 + CAPA 閉環)
  ▼
VS-09 合規稽核與持續改善
  └─→ 回饋至 VS-01..VS-08 (5 條 feedback loops, §16)
```

每條流的「交點」即為潛在的簽核任務（§10），由 `event_outbox` 送出事件，而不是跨表 join。

### §3.3 產業疊層 IND-1..IND-6

| 代號 | 產業 | 疊層參數（資料，不 fork 程式） | 必須啟用的價值流 |
|---|---|---|---|
| IND-1 | 百貨量販 | 專櫃品牌方清單、退換貨折讓規則集 | VS-05, VS-08 |
| IND-2 | 零售電商 | 通道清單（Shopee/momo/PChome/Cyberbiz）、欄位契約 | VS-05, VS-06, VS-07 |
| IND-3 | 製造業 | BOM 層級、批次追溯深度、檢驗標準集 | VS-02, VS-03, VS-04 |
| IND-4 | 通路經銷 | 經銷價表、跨店調撥、回款條件 | VS-01, VS-05, VS-06 |
| IND-5 | 受管制品（食藥粧） | 法規清單（標示法條文、廠登、藥證）、效期規則 | VS-02, VS-03, VS-07 |
| IND-6 | 服務業 | 服務規格集、SLA 承諾值 | VS-08 |

**原則**：產業差異 = policy pack（資料）+ adapter set（程式）+ KB rule set（事實）；不得 fork 三大工作室或簽核引擎（見 §17 DEC-26）。

### §3.4 營運規模層級 SCALE-1..SCALE-4

| 代號 | 規模 | 工作室數 | 關卡 | SLO／人力基準（未核定，見 PR-03） | 範圍外 |
|---|---|---|---|---|---|
| SCALE-1 | 單點試點 | 1（僅治理面） | G1–G5 全，PIA 流程一個 | 單機、loopback、1 名治理人員 | 跨機、多租戶、信箱自動化 |
| SCALE-2 | 部門級 | 2（+ 整合面） | + 雙簽核 | 每日備份、雙人覆核 | 跨部門聯邦 |
| SCALE-3 | 多部門多通路 | 3（二＋執行面） | + 自動放行閘 | SLO/RPO/RTO 待定 | 多國法規 |
| SCALE-4 | 集團級 | 3 + 子工作室 | + 連續稽核 | 多據點、災難復原 | 無 |

層級提升以**觸發事件**（流量、稽核發現、法規變更、合併）驅動，不得以時間軸強推。

---

## §4 三大工作室架構（3 核心＋規模觸發擴編）

### §4.1 工作室定義與版本標記

> **釐清**：`workflows`／`approvals`／`settings`／`api-docs` 路由實作於 ekpi-platform `platform/src/governance_web.rs`（零 JS、CSS-first）；`apps/web`（`ekp_web`）為獨立骨架，與工作台無關。

| 工作室 | 主責 | 核心帳號 | 主要介面 | 不做 |
|---|---|---|---|---|
| **工作室 1：內部使用者大平台** | 日常業務操作：專案建立、證據上傳、簽核任務處理、查詢儀表板 | Viewer/Editor/Approver（依 RBAC，§13） | `governance_web` 13 HTML 頁（見 §11.1） | 不得自建或修改簽核狀態機 |
| **工作室 2：技術工作室** | 系統整合：適配器（momo/PChome/Cyberbiz）、API 維護、資料庫運維、異常監控、排程 | Admin（技術帳號，重大變更需雙人核准） | `platform-hub`（5 crates）、`wellfit.exe`、Windmill 排程 | 不得自行改法規解釋 |
| **工作室 3：治理工作室** | 政策定義、合規審查、稽核鏈驗證、規格審查、變更治理 | Auditor（僅審查，不執行）＋ KB 規格登錄 | `governance_web` 稽核頁、API 文件頁、KB 審查介面 | 不得執行業務交易 |

### §4.2 職責邊界

```
┌─────────────────────────────────────────────────────────────────┐
│  工作室 1：內部使用者大平台                                        │
│  日常業務操作：專案建立、證據上傳、簽核任務處理、查詢儀表板        │
│  介面：governance_web（13 HTML 頁）；權限：Viewer/Editor/Approver  │
└─────────────────────────────────────────────────────────────────┘
                    │ 操作產生事件寫入 audit_chain（SHA-256）
                    │ 簽核任務進入 event_outbox
                    ▼
┌─────────────────────────────────────────────────────────────────┐
│  工作室 2：技術工作室                                             │
│  適配器開發（momo/PChome/Cyberbiz）、API 維護、資料庫運維、        │
│  異常監控；介面：platform-hub + wellfit.exe + Windmill             │
│  權限：Admin（重大變更雙人核准）                                   │
└─────────────────────────────────────────────────────────────────┘
                    │ 標準介面與自動化腳本（已有）
                    │ 重大變更需治理工作室核准（JIT 簽核）
                    ▼
┌─────────────────────────────────────────────────────────────────┐
│  工作室 3：治理工作室                                             │
│  政策定義、合規審查、稽核鏈驗證、規格審查、變更治理               │
│  權限：Auditor（僅審查）；規格登錄 Admin（KB，單向引用）           │
└─────────────────────────────────────────────────────────────────┘
                    │ 審查結果寫入 audit_chain
                    │ 規格條文以 SHA-256 登錄 KB
                    ▼
            Knowledge/（規格資料，SSOT）
            審查員對照電商事件與規格條文做出合規裁定
```

- **工作室 1 → 工作室 3**：每筆業務變更寫入同一 `audit_chain`，治理可完整追溯。
- **工作室 2 → 工作室 1**：提供標準介面與自動化腳本，僅重大變更需覆核。
- **工作室 3 → 工作室 2**：政策與合規要求以規格形式提出，規格變更需治理核准（KB 單向寫入）。
- **工作室 2 → 工作室 3**：技術變更需治理核准（JIT 簽核）；失敗閉鎖退回人工。

### §4.3 規模觸發擴編

| 觸發事件 | 擴編動作 | 決策權 | 證據 |
|---|---|---|---|
| 工作室 1 任務量週期性超過當日可處理量 2× | 增設子工作室 1.x（以產業 IND-* 為界） | 流程擁有者 | 佇列日誌 |
| 新增通路達 3 家 | 增設適配器隊列（platform-hub adapter set） | 工作室 2 主管 | 新契約註冊 |
| 稽核發現控制失效率 > 5% | 增設獨立覆核員（INTERNAL_AUDIT） | 執行贊助人 | 稽核報告 |
| 跨國法規啟用 | 增設司法管轄區 policy pack | 法遵＋執行贊助人 | KNB-CMP 增補 |

新工作室必須繼承 HO-01..HO-08 交接契約並註冊於治理註冊表；不得以「新部門」繞過稽核鏈。

---

## §5 職責、決策權與交接契約（HO-01..HO-08）

### §5.1 決策權矩陣（RACI 摘要）

| 決策 | 工作室 1 | 工作室 2 | 工作室 3 | 執行層 |
|---|---|---|---|---|
| 資料輸入／編輯 | R | C | I | A（流程擁有者） |
| 簽核／駁回（金額分級） | R（≤ 等級） | I | I | A（金額上限） |
| 適配器／頻道管理 | I | R | C | A |
| 規格／合規條文登錄 | I | I | R | A（法遵） |
| 稽核鏈查看 | R | R | R | I |
| 系統設定／儲存模式 | I | R | C | A |
| 使用者／角色管理 | I | R | C | A |
| 事件重播／除錯 | I | R | R | A |

> **不可取代決策**（對應 KB 9 角色）：EXEC_SPONSOR（風險接受）、ISMS_OWNER（ISMS 範圍）、PROCESS_OWNER（業務風險）、ASSET_OWNER（資料分類）、CONTROL_OWNER（控制可用性，不可獨立稽核自己）、LEGAL_PRIVACY（法律結論）、SEC_IT_ENG（技術證據，不得核准風險接受）、INTERNAL_AUDIT（獨立性）、PMO（不得將文件交付等同有效性判定）。

### §5.2 交接契約 HO-01..HO-08

每張交接契約含四要素：**交接包、受理條件、SLA、所有權轉移點**。**未受理不得交接**——交付方不得假定已移交。

| 代號 | 交點 | 交接包 | 受理條件 | SLA（未核定，見 DEC-19） | 所有權轉移 |
|---|---|---|---|---|---|
| HO-01 | VS-01 採購 → VS-02 主資料 | 購貨單＋供應商資格＋JitPermission 證明 | 供應商級別 ≥ D2；合約未過期 | 1 個工作日 | 購貨單簽字 |
| HO-02 | VS-02 主資料 → VS-03 生產 | SKU 版本＋標示法欄位＋checksum | 欄位契約 8 欄全驗證通過；版本已發布 | 即時 | 版本發布事件 |
| HO-03 | VS-03 生產 → VS-04 倉儲 | 批次＋工單＋原料來源紀錄 | 五檔契約全通過（56 欄字典、15 VR、12 ANOM） | 1 個工作日 | 入庫驗收 |
| HO-04 | VS-04 倉儲 → VS-05 零售 | 庫存可用量＋效期清單（FEFO 排序） | 盤點差異 = 0 或已隔離 | 4 小時 | 出貨單 |
| HO-05 | VS-05 零售 → VS-06 銷售訂單 | 銷售事件（含 order_id、金額、扣庫結果） | 事件已寫入 outbox 且未重投 | 即時 | 訂單建立 |
| HO-06 | VS-06 訂單 → VS-07 行銷內容 | 訂單＋商品內容版本＋宣稱佐證清單 | 行銷內容已過法遵審查 | 1 個工作日 | 內容發布 |
| HO-07 | VS-07 內容 → VS-08 售後 | 內容版本＋客訴原文＋訂單追溯號 | 客訴分類已完成；時限 SLA 已啟動 | 24 小時 | 客訴單開啟 |
| HO-08 | VS-08 客訴 → VS-09 稽核／改善 | CAPA 報告＋驗證證據＋剩餘風險接受書 | 驗證證據完整；不可豁免項無一 waived | 72 小時 | CAPA 結案 |

### §5.3 保障

- **SoD（職責分離）**：同一人不得同時為「簽核人」與「執行人」（`platform-hub` 的 `agent-policy` 與 `wellfit_core::access_control::SodConstraint`）。
- **JIT 權限**：跨機部署、資料庫備份還原需治理工作室核准；執行上限由 `JitPermissionManager` 控制，過期即失效。
- **失敗閉鎖**：自動化遇異常預設退回人工簽核，不可靜默繼續。

---

## §6 協作機制與升級路徑（ESC-1..ESC-4）

### §6.1 例行節奏

| 機制 | 頻率 | 主持 | 輸出 |
|---|---|---|---|
| Studio Council（三大工作室例會） | 每週 | PMO | 交接積壓清單、BLOCKED 清單 |
| 變更審查（微變更：當日；重大：雙簽） | 每日 | 流程擁有者＋治理 | 變更事件（寫入 audit_chain） |
| 稽核鏈驗證 | 每月 | INTERNAL_AUDIT | 驗證報告 |
| 控制自評（Control Self Assessment） | 季 | CONTROL_OWNER | 評分表；不替代獨立稽核 |
| 改善回顧（Retro） | 每里程碑 | PMO | 改善動作登記（不降低閘門） |

### §6.2 共同問題解決（聯合事故）

1. 觸發事件 → 差距總監指派 → 24h 內產出隔離清單（不自動修正）。
2. 指派：三方（業務／技術／治理）各 1 名代表；產出「三方比對表」。
3. 比對結果三種：一致 → 通過；不一致或缺漏 → 進入隔離清單（§15 RB-06）；結構性缺陷 → 發出 CAPA（VS-09）。
4. 結論必須回寫 `event_outbox`（append-only），事件 ID 可被 `HO-03`、`HO-05`、`HO-08` 引用。

### §6.3 升級路徑 ESC-1..ESC-4

| 層級 | 觸發 | 負責人 | 最大停留 | 升級訊號 |
|---|---|---|---|---|
| ESC-1 | 工作階段內爭議 | 工作階段主管 | 1 個工作日 | 未解 |
| ESC-2 | 跨工作室爭議 | Studio Council | 3 個工作日 | 未解 |
| ESC-3 | 流程擁有者 | EXEC_SPONSOR（風險接受） | 5 個工作日 | 未解 |
| ESC-4 | 重大違規／監管通知 | LEGAL_PRIVACY＋EXEC_SPONSOR 緊急會議 | 24 小時 | 立即停用相關自動化 |

---

## §7 目標架構

### §7.1 三層耦合（與 v2.1 一致，證據為准）

```
┌─────────────────────────────────────────────────────────────────┐
│  治理面（EKPI）                                                  │
│  apps/web（Axum 0.8.9 + Askama 0.16.1，CSS-first 零JS）          │
│  crates/domain（governance/tenant/auth + workflow/channels）     │
│  crates/application（ports.rs + approval.rs；AgentAdvisor 僅 advise()）│
│  crates/adapters（jsonl_ledger + postgres/{governance,identity,approval}）│
│  platform/（裁決引擎、governance_web、api.rs）                  │
│  【NEW migration】approval_workflows / approval_tasks /         │
│    event_outbox（出站事件骨幹，共享事實來源）                     │
└──────────────────┬──────────────────────────────────────────────┘
                   │ 窄 API 契約 + PostgreSQL 事件表（outbox/inbox）
                   ▼
┌─────────────────────────────────────────────────────────────────┐
│  整合面（platform-hub）                                          │
│  crates/udm（統一資料模型，金額 i64 cents + Decimal4）           │
│  crates/channels（ChannelAdapter + FormatDeriver 實作面）       │
│  crates/eventbus（EventStore，consumer 側 inbox + publisher）   │
│  crates/adapters-momo（條件式實作，pending_contract）            │
│  crates/agent-policy（低風險自動核准閘，DEC-15 閘控）          │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────────┐
│  執行面（Wellfit + 根工作區）                                    │
│  wellfit_core（帳本、ReleaseGate、契約）                        │
│  wellfit_shopee/reports/mtrix/bankpay（匯入/匯出）             │
│  wellfit_integration（三方對帳：EKPI ↔ 事件骨幹 ↔ 執行）       │
│  根工作區（FlapsErp RPA、AI decision_gating）                  │
└─────────────────────────────────────────────────────────────────┘
```

### §7.2 落地狀態更正（回退 v2.1 §7.1 錯誤宣稱）

| 階段 | v2.1 錯誤宣稱 | 實際狀態（以 §2 為準） |
|---|---|---|
| Phase C1–C5 | 「postgres 適配器 \| 全通過」 | domain/application/adapters（jsonl）全通過；**postgres 僅 identity.rs 真實 SQL**，governance.rs 9×todo!()、approval.rs 8×stub、channels.rs 1×stub → 狀態 **Partial** |
| Phase C6 | 「API v1 路由（11 端點）」 | **13 method-routes**；2 已接線；11×501（見 §11） |
| Phase C9–C11 | 「工作台頁面」 | `/approvals`、`settings`、`api-docs` 已上線但內容有誤（C-12、C-13 未修前不得引用） |
| Phase D1 | 「approve_transaction stub 移除」 | **未做**（C-04，孤兒目錄） |
| Phase D2 | 「f64 殘留收斂」 | **部分**（C-17，`web` crate 未收斂） |
| Phase D3 | 「/api/health 擴充」 | **不存在**（C-03） |
| Phase C8 | `three_way` | 真實（§2.3） |
| Phase E2–E6 | 電商適配器 | **Blocked**（OQ-F2-5，未取得規格，期限 2026-11-19） |
| Phase F | 部署 | **Blocked**（PR-02） |

### §7.3 架構決策總表

| 決策 | 裁定 | 依據 |
|---|---|---|
| 三大工作室架構 | **採納**（§4） | Presentation → Application → Domain（Clean Architecture 對應） |
| 平台定位 | **專案、稽核與知識管理** | 本計畫裁定 |
| 電商與規格跨層整合 | **事件總線耦合，不深度 schema 整合** | 耦合度／更新頻率／責任歸屬（§8） |
| 工作台 | **零 JS、CSS-first、表單 POST + PRG** | 憲章 §2.3 |
| 基礎設施 | **PostgreSQL 18 + 原生 binary**；Redis 按需；Kafka 僅 trait 空槽 | 推翻 Kafka+k3s 提議 |
| 技術基線 | ekpi-platform：Rust 2024 / Resolver 3 / Axum 0.8.9 / Askama 0.16.1 / SQLx 0.9.0 / PostgreSQL 18.6；根工作區：Edition 2021 / SQLite | ADR-0006／ADR-0001（維持不變） |
| AI 邊界 | ekpi-platform 內 Agent 僅 advisory；DEC-15 未決前不得啟用自動核准；`AutoApprovalGate::advisory_only()` | phase-gates §Out of scope；§10 |
| 四套基線統一 | **未決**（DEC-22） | 本計畫不宣稱已統一 |

---

## §8 資料與整合模型

### §8.1 資料三分類（所有權不可混淆）

| 類別 | 內容 | 寫入方 | 不可被誰覆寫 |
|---|---|---|---|
| 事實（Facts） | 訂單號、金額、庫存、批次 | 業務事件（event_outbox） | 規格資料不得覆寫 |
| 規則（Rules） | 驗證條文、對照表、閾值 | KB（`Knowledge/specs/` SHA-256 登錄） | 電商事件不得修改 |
| 決策（Decisions） | 簽核結果、Gate 決議、例外接受 | approval API／audit event | 任何一方不得靜默改寫 |

> 關聯發生於「簽核任務」層：`ApprovalTask` 可同時參照 `event_outbox.id` 與 `Knowledge/specs/<id>`，由核准者裁定。

### §8.2 金額／數值

- `platform-hub/crates/udm`：`MoneyCents(i64)` + `Decimal4(i64)`（10^4 縮放）；`MoneyCents(600)="6.00"`。
- `Wellfit`：`ScaledDecimal`（10^4）、`i64` 分運算；`web` crate 殘留 f64 須收斂（C-17）。
- **貸款／價格分級跳躍**：金額形態以 `i64` 最小單位為唯一合法載體；任何 `f64` 進入金額路徑即缺陷（QG-05）。

### §8.3 欄位契約（名稱解析，禁序位退讓）

- `wellfit_core::contract`（`ColumnContract`／`ColumnMap`）；`platform-hub/crates/channels`（`ColumnContract::validate_header`）。
- 必填欄缺漏 → `Err`，**不得**退回硬編碼序位；多餘欄位 → 警告並隔離，不靜默丟棄。
- Shopee 8 欄契約、MTRIX 56 欄字典 + 15 條 VR + 12 條 ANOM（§2.3）為先行實證。

### §8.4 事件骨幹（outbox/inbox）

```
電商平台（momo/PChome/Cyberbiz）
      │ ChannelAdapter（platform-hub）
      ▼
UDM 轉換 → event_outbox（PostgreSQL，ekpi-platform 所有）
      │
      ▼
event_inbox（冪等去重）→ ApprovalTask（簽核任務）→ audit_chain（SHA-256 稽核）
      │
      ▼
Knowledge/（規格資料，SHA-256 登錄，單向引用）
```

- `eventbus` 的 `EventStore` trait（`publish`／`consume`／`pending_outbox`／`mark_published`）為日後 Kafka 的唯一替換點（`KafkaEventStore` 空槽）。
- 跨 repo 一律以 JSON 契約交換（`three_way` 註解明載，無 path dep）。

### §8.5 三方對帳閉環（Phase C8）

```
EKPI 決策（approval API/audit event）
    ╲
     ╲ 三方比對（wellfit_integration::three_way::ThreeWayReconciler）
      ╲
事件骨幹（event_outbox/inbox)
        ╲
         ╲
Wellfit 執行（IdempotencyLedger）
    ↘
     一致（Consistent） → 通過
     不一致（Mismatch）/缺漏（Missing） → IsolationEntry（隔離清單，絕不靜默修正）
```

一致性規則：EKPI = 事件骨幹；`approved⇒executed`；`rejected⇒rejected|isolated`。違規或缺側一律 quarantine。

### §8.6 稽核鏈

- `platform/src/governance.rs`：append-only JSONL 事件（`GENESIS_HASH="GENESIS"`、9 變體 `EventPayload`、SHA-256 帳鏈）；`default_ledger_path()`（`%LOCALAPPDATA%\EKPI\governance-ledger.jsonl` 或 `EKP_GOVERNANCE_DATA`）；`Ledger::open` 持 advisory lock、重放並逐筆驗證 sequence／previous_sha256／sha256，截斷末行／空白行／重複 ID／懸空參照／非法流轉一律 fail-closed 不附加。
- **不視為** WORM（`worm` 模組屬 library-only，見 `docs/backend/README.md`）。

---

## §9 端到端流程規格

每條價值流以 HO 交接為界，逐流定義觸發、參與者、產出、證據與例外。

### §9.1 主線總圖

```
                        ┌─ 治理面 ────────────────────────┐
  VS-01 採購 ─▶ VS-02 主資料 ─▶ VS-03 生產 ─▶ VS-04 倉儲 ─▶ VS-05 零售
       │                │                │               │           │
       ▼                ▼                ▼               ▼           ▼
  [簽核：購貨單]  [簽核：主資料版本]  [簽核：工單]   [簽核：盤點]  [簽核：價量]
                        │                │               │           │
       └────────────────┴────────────────┴───────────────┴───────────┘
                                    │
                                    ▼
                        VS-06 銷售訂單 + 三方對帳 (event_outbox)
                                    │
                                    ▼
                        VS-07 行銷內容 (宣稱佐證 + 法遵)
                                    │
                                    ▼
                        VS-08 售後/客訴 → CAPA → 驗證
                                    │
                                    ▼
                        VS-09 合規稽核 → SoA → 風險登錄 → 改善
                                    │ (5 feedback loops §16)
                                    └──→ 回流至 VS-01..VS-08
```

### §9.2 各流規格（觸發／產出／證據／例外）

| 流 | 觸發 | 產出（事件） | 證據（稽核） | 例外處理 |
|---|---|---|---|---|
| VS-01 | 需求單提交 | 購貨單、驗收紀錄 | 供應商資格＋核准事件 | 驗收不一致 → 隔離單 |
| VS-02 | 新品上市／法規變更 | 主資料版本（SHA-256） | 欄位契約驗證紀錄 | 標示法缺欄 → blocked（不得發布） |
| VS-03 | 工單下達 | 批次紀錄、原料追溯 | 生產參數簽名 | 超規 → 停線＋CAPA |
| VS-04 | 盤點／效期到期 | 庫存變動事件 | 盤點單、FEFO 排序清單 | 效期不足 → 報廢核准 |
| VS-05 | 門市結帳 | 銷售事件 | 收銀紀錄 | 折讓超限 → 主管覆核 |
| VS-06 | 訂單匯入 | 訂單事件、結算單 | 8 欄契約驗證、VR-01..VR-15、ANOM-01..12 | 缺欄／溢出 → 隔離（VR-01 整檔拒絕、VR-05 重複彙總） |
| VS-07 | 訴求文案提交 | 內容版本＋佐證清單 | 法遵審查意見 | 佐證不足 → 退回（不發布） |
| VS-08 | 客訴受理 | 客訴單、賠償單 | 處理紀錄、RMA | 超時 → ESC-1 |
| VS-09 | 季度／年稽核 | SoA、風險登錄、KPI 基線 | 複驗證據、accept記錄 | 未閉環 → 升級 ESC-2 |

---

## §10 簽核引擎規格

### §10.1 已實作（`crates/domain/src/workflow.rs`，經查證）

- `Urgency::{Low, Normal, High, Critical}` 含 `sla_seconds()`。
- `ConditionExpr` 6 變體＋`And/Or/Not`；`Stage`；`TimeoutAction::{Escalate, AutoReject, AutoApprove, Cancel}`。
- `WorkflowDefinition::validate()`（空階段／重複 id／零 SLA／未知升級路徑皆拒）。
- `TaskState::{Pending, Approved, Rejected, Escalated, Expired, Cancelled}` 含 `is_terminal()`／`transition()`。
- `TaskAction::{Approve, Reject, Escalate, Expire, Cancel}`。
- `WorkflowError::{InvalidTransition, InvalidDefinition, InvalidSubmission, NoEscalationPath}`。
- `AgentDecision` 含 `auto_approved` 保留欄位，**無路徑設為 true**（須以測試鎖定，見 QG-10）。

### §10.2 未完成（明確標註）

| 項目 | 現況 | 處理 |
|---|---|---|
| `postgres/approval.rs` | 8 × `Err(Storage("not yet implemented"))`（fail-closed，非 panic） | PHASE-G4 實作 PostgreSQL 版；完成前一律 JSONL（SLICE-G05） |
| `postgres/governance.rs` | 9 × `todo!()`（runtime panic） | PHASE-G4 高順位；先將 `todo!()` 改為 fail-closed `Err`（SLICE-G05），再接真實 SQL |
| `postgres/channels.rs` | 1 × 同上 | PHASE-G4 |
| API v1 寫入端點（11 條） | 全 501 | §11.3 目標契約逐條定義 |
| 多層條件分支 UI | ConditionExpr 已在 domain 支援，UI 未做 | 非目標（BDD §3） |

### §10.3 AI／Agent 邊界（硬性）

- ekpi-platform 內 `AgentAdvisor` 僅 `advise()`，**無 `approve()`**。
- `platform-hub/agent-policy` 的 `AutoApprovalGate::advisory_only()`（enabled=false、threshold=1.0）為 DEC-15 解鎖前的唯一合法狀態；`decide()` 失敗閉鎖。
- 任何新增自動放行必須先關閉 DEC-15 並經本計畫 §20 QG-10 機械化測試（`grep -r auto_approve crates/` 必須 0 命中）。

---

## §11 工作台與 API 契約

### §11.1 已接線版面（`platform/src/governance_web.rs`，3027 行；路由註冊 L141-194）

> 注：C-24 — v2.1「13 頁」漏列營運端點。以下為 13 個**業務頁**，另含 3 個營運端點（`/`、`/healthz`、`/metrics`、`/ready` 共 4 頁；`/ready` 未在 v2.1 表內）。

| 路徑 | 頁面 | 狀態 |
|---|---|---|
| `/ready` | 就緒檢查（`ready.rs`） | 已實作（v2.1 未列） |
| `/healthz` | 存活 | 已實作 |
| `/metrics` | Prometheus text | 已實作 |
| `/` | 工作總覽 | 已實作 |
| `/login` | 登入 | 已實作 |
| `/projects` | 專案管理 | 已實作 |
| `/evidence` | 文件與證據 | 已實作 |
| `/inspections` | 品質檢查（29 項） | 已實作 |
| `/workflows` | 流程範本 | 已實作 |
| `/controls` | 風險與品質 | 已實作 |
| `/lifecycle` | 階段與關卡 | 已實作 |
| `/audit` | 稽核紀錄 | 已實作 |
| `/settings` | 系統設定（見 §11.5） | 已實作（內容誤導） |
| `/api-docs` | API 文件（見 §11.4） | 已實作（內容誤導） |
| `/approvals` | **核准任務佇列** | Phase C9 已實作（真實資料） |
| `/assets/governance.css` | 樣式（公開） | 已實作 |

### §11.2 API v1 端點（13 method-routes；2 已接線，11 × 501）

| 方法 | 端點 | 說明 | 狀態 |
|---|---|---|---|
| GET | `/api/v1/workflows` | 列出簽核工作流定義 | ✅ 已接線（真實資料） |
| POST | `/api/v1/workflows` | 建立簽核工作流定義 | 501 |
| POST | `/api/v1/workflows/{id}/publish` | 發布定義 | 501 |
| GET | `/api/v1/approvals` | 列出簽核任務 | ✅ 已接線（真實資料） |
| POST | `/api/v1/approvals` | 送審 | 501 |
| POST | `/api/v1/approvals/{id}/decide` | 核准/駁回 | 501 |
| POST | `/api/v1/approvals/{id}/escalate` | 升級 | 501 |
| POST | `/api/v1/events/replay` | 重播指定事件 | 501 |
| GET | `/api/v1/channels` | 列出通道適配器 | 501 |
| GET | `/api/v1/channels/health` | 通道健康 | 501 |
| GET | `/api/v1/users` | 列出使用者 | 501 |
| GET | `/api/v1/settings` | 系統設定 | 501 |
| GET | `/api/v1/search` | 全域搜尋 | 501 |

### §11.3 API 認證說明（回退 v2.1 §5.1 錯誤）

| 場景 | 認證方式 |
|------|----------|
| 工作台操作 | session cookie（`ekpi_session`）＋ CSRF（`ekpi_csrf` cookie 或 `X-CSRF-Token` header） |
| 程式化呼叫 | **同一本機 session cookie**；未登入一律 `401` 結構化錯誤。**無 Bearer／API key（不存在的能力不得宣稱）** |

- 依據：`docs/contracts/approval-api.md`（CTR-API-001）§1「認證：與工作台同一本機 session（cookie）；未登入一律 401 結構化錯誤；**無** Bearer/API key（不存在的能力不得宣稱）」。
- CSRF：`governance_web.rs` 的 `validate_csrf_token` middleware（L201-204）＋ `X-CSRF-Token` 接受 header（程式化）或 cookie（表單）；登入頁雙提交（hidden field + `ekpi_csrf` cookie）。
- `is_public_path` 僅含 `/healthz`、`/metrics`、`/login`、`/assets/`（L667）；**`/ready` 須登入**（受 `require_session` 保護）。
- 501 的結構化錯誤：`{"error":{code,type,message,timestamp,request_id,...}}`（見 `api_not_implemented`）。

### §11.4 `/api-docs` 頁待修項（C-12）

修復要點（SLICE-G06，詳見 §17）：
1. 表格必須列全 **13 條**（含「GET /api/v1/channels/health」與「GET /api/v1/search」）。
2. 把 `GET /api/v1/workflows`、`GET /api/v1/approvals` 正確標為「已接線（唯讀）」。
3. 移除或改寫「所有 API 端點目前一律回 501」聲明；改為「13 條中 2 條已接線（唯讀），其餘 11 條回 501」。
4. 頁面內容不得與 `docs/contracts/approval-api.md` 矛盾；長遠改為由契約檔產生（QG-07 鎖定）。
5. 在修復完成前，本頁不得作為契約事實引用。

### §11.5 `/settings` 頁待修項（C-13）

現況：頁面宣稱 PostgreSQL 模式僅在 `EKPI_STORAGE=postgres`＋`DATABASE_URL` 啟用；但 `AppConfig::from_env` 無呼叫、供應 binary 不讀取（L12／L24 寫死） → **頁面宣稱與供應 binary 行為不一致**。

修復要點（SLICE-G06）：
1. `/settings` 頁加註「PostgreSQL 尚未在供應 binary 啟用；設定需另行接線 AppConfig」。
2. 或在供應 binary 真正接線 `AppConfig::from_env` 後（PHASE-G3）再宣稱。

### §11.6 `apps/web` 區隔（C-21）

- `apps/web`（`ekp_web`，自有 router `／／health／/submit`，僅 `home.html`）與 `governance_web` 工作台**完全獨立**，不得混同。
- 長遠二選一（OQ-GOV-09）：合併為一或移除 `apps/web`。

### §11.7 API v1 收斂目標（每端點先契約後實作）

| 批次 | 端點 | 理由／注意 |
|---|---|---|
| B-A 唯讀（已） | GET /api/v1/workflows、GET /api/v1/approvals | 已落地（真實資料） |
| B-B 寫入 | POST /api/v1/workflows、…/publish、POST /api/v1/approvals、…/decide、…/escalate | 需冪等鍵＋重放防護（CTR-API-001 §3） |
| B-C 事件 | POST /api/v1/events/replay | 需重放契約定義 |
| B-D 治理 | GET /api/v1/channels、/health、GET /api/v1/users、GET /api/v1/settings | 通道註冊表、身分治理邊界（DEC-12）、設定安全審查 |
| B-E 檢索 | GET /api/v1/search | 搜尋 API 參數／權限契約（S19 權限語義先定） |

**原則**：「任何端點從 501 轉為 200 前，必須先有契約、測試與誠實資料來源；禁止先上線後補文檔」（CTR-API-001 §4）。

---

## §12 稽核與合規對照

### §12.1 稽核鏈（audit_chain）

- `platform/src/governance.rs`：SHA-256 帳鏈（`GENESIS_HASH="GENESIS"`、9 變體 `EventPayload`、`QUALITY_CHECKLIST` 29 項）。
- 完整性檢查：`Ledger::open` 時逐筆驗證；截斷／空白／重複 ID／懸空參照／非法流轉 → fail-closed。
- 限制：目前為 JSONL 單機帳鏈，**非 WORM**（`worm` 僅 library-only）；Postgres 版不可視為生產稽核庫（DEC-14 未決）。

### §12.2 KB（Knowledge）權威分工（ADR-0012）

| 事實 | 權威來源 | 本計畫處理 |
|---|---|---|
| G0–G7 關卡定義、決策權、通過／條件通過／阻擋條件 | IMS 套件（`49_Phase0-1_驗收關卡.csv`） | 引用，不另立 |
| 9 種 IMS 角色 | IMS 手冊 §2 | 引用，不另立 |
| 12 process areas | `tools/compliance/vocab/process-areas.csv` | 引用，不另立 |
| SoA 93 控制母表 | `soa-93-controls.csv`（ISO 27001:2022 Annex A，A.5.1–A.5.37／A.6.1–A.6.8／A.7.1–A.7.14／A.8.1–A.8.34） | 引用，不另立 |
| Crosswalk 18 欄 schema；主鍵 `control_id + standard_ref + std_clause` | ADR-0011 §2.3（欄位：`control_id`、`control_title`、`applicability`、`exclude_reason`、`verify_status`、`owner_role`、`review_cycle`、`evidence_id`、`evidence_ref`、`standard_ref`、`std_clause`、`gap`、`disposition`） | 引用，不另立 |
| 權限治理垂直切片 | `templates/wbs-workpackages.csv`、`templates/wg-workgroups.csv` | 引用 |
| KB 舊 G-0..G-4（別名） | G-0→G1、G-1→G2、G-2→G3、G-3→G6、G-4→G7；G0、G4、G5 為 IMS 新增 | 引用時須標明 IMS 對應（ADR-0012 §4） |

### §12.3 ekpi-platform G1–G5 與 IMS G0–G7 之對應（必須顯式，禁止靜默重編號）

ekpi-platform 的 G1–G5 是**專案生命週期關卡**（`docs/governance/06-data-and-state-model-v1.md` §5 起改狀態機；`platform/src/governance.rs:735` 與 `:1122` 的 `["G1","G2","G3","G4","G5"]` whitelist；`governance_web.rs:2522-2526` 的說明）；IMS 的 G0–G7 是 **ISMS 實施關卡**（ADR-0012）。兩者**不可混用**，以下為**對應表（不是同一概念）**：

| ekpi-platform 專案關卡 | 語義（`governance_web.rs`） | IMS 對應（參考） | KB 舊別名 |
|---|---|---|---|
| G1 立案 | 目的、贊助人、範圍與風險分級已確認 | G1 範圍（IMS）近似 | G-0 → G1 |
| G2 基準 | 需求、驗收標準、計畫及責任已確認 | G2 基線（IMS）近似 | G-1 → G2 |
| G3 執行 | 關鍵品質檢查通過，或缺失已正式登錄 | G3 風險與 SoA（IMS）近似 | G-2 → G3 |
| G4 交付 | 必要證據、適用要求與驗收條件已滿足 | G4 控制實施／G5 測試（IMS）跨兩階 | —（IMS 新增） |
| G5 結案 | 移交、未結責任、封存及改善計畫已完成 | G6 內稽與管理評審／G7 維運移交（IMS）跨兩階 | G-3 → G6、G-4 → G7 |

> **ADR-0004 提案（DEC-21）**：必須保留已發佈帳鏈中 `G1..G5` 字串（帳鏈相容），同時在治理層新增 `gate_namespace: project | ims` 分別儲存關卡決定；UI 呈現兩個命名空間但不得混寫。未裁決前，任何文件引用 `G1..G5` 時須標註「ekpi-platform 專案關卡」，引用 `G0..G7` 時須標註「IMS」。

### §12.4 9 IMS 角色 × 3 工作室 × 4 應用角色 對照

| IMS 角色 | 工作室歸屬 | 應用 Role（local_auth） | 決策禁令 |
|---|---|---|---|
| EXEC_SPONSOR | 3 | —（管理層） | 唯一風險接受者 |
| ISMS_OWNER | 3 | Admin | 不可代替業務風險擁有人接受風險 |
| PROCESS_OWNER | 1 | Approver | 業務風險唯一接受者 |
| ASSET_OWNER | 1/3 | Editor | 不可以技術管理代替業務所有權 |
| CONTROL_OWNER | 2 | Editor | 不可獨立稽核自己設計的控制 |
| LEGAL_PRIVACY | 3 | —（外部職能） | 法律結論須保留法域/日期/事實前提/覆核人 |
| SEC_IT_ENG | 2 | Admin | 不得核准業務風險接受 |
| INTERNAL_AUDIT | 3 | Viewer（稽核帳號） | 不得稽核自己執行的工作 |
| PMO | 1/3 | Approver | 不得將文件交付等同控制有效性判定 |

### §12.5 12 專案階段（AT-02）對應

`AT-02` 列 12 階段：立案 → 需求分析 → 計畫 → 文件系統設定 → 執行 → 品質驗證 → 變更分析 → 缺失風險閉環 → 階段放行 → 驗收移交 → 結案封存 → 效益改善。
對應 VS 域：立案（VS-01 前置）、需求分析（VS-01）、計畫（VS-02）、文件系統設定（VS-02）、執行（VS-03/VS-04）、品質驗證（VS-09）、變更分析（VS-09）、缺失風險閉環（VS-09）、階段放行（VS-05/VS-06）、驗收移交（VS-06）、結案封存（VS-09）、效益改善（VS-09 回饋 §16）。

每階段必須具備「進入條件／活動／交付物／角色／完成條件／前後階段」欄位（AT-02），不可只剩名稱。

---

## §13 權限模型

### §13.1 雙 Role 並存（C-15）— 必須收斂

| 型別 | 位置 | 規則 |
|---|---|---|
| `ekp_domain::auth::Role` | `crates/domain/src/auth.rs:24` | Viewer < Editor < Approver < Admin（`satisfies()`）；`Permission` 4 種＋`minimum_role()` |
| `ekp_platform::local_auth::Role` | `platform/src/local_auth.rs:41` | 同名 4 階；`authenticate` + `has_role`；實際被 `governance_web` 用 |

**目標**（PHASE-G3，DEC-23）：登入層保留 `local_auth::Role`；authorization 統一以 domain `Permission::minimum_role()` 為準；禁止跨層互換「角色」一詞。未收斂前，程式碼必須 `use` 完整路徑，不得僅寫 `Role`。

### §13.2 權限矩陣（擴展 v2.1 §5）

| 能力 | 工作室 1（業務） | 工作室 2（技術） | 工作室 3（治理） |
|---|---|---|---|
| 專案／任務 CRUD | Editor（自有部門） | Viewer | Viewer |
| 簽核／駁回／升級 | Approver（金額分級） | 無（除非兼任主管） | Auditor（僅審查，不執行） |
| 適配器／頻道管理 | 無 | Admin | Viewer |
| 規格／合規條文登錄 | 無 | 無 | Admin（KB，單向引用） |
| 稽核鏈查看 | Viewer | Viewer | Admin |
| 系統設定／儲存模式 | 無 | Admin | 無 |
| 使用者／角色管理 | 無 | Admin（技術帳號） | 無（需工作室 2 協作） |
| 事件重播／除錯 | 無 | Admin | Viewer |

### §13.3 關鍵控制

- **SoD**：不得同時擔任簽核與執行（`SodConstraint`、`PermissionLevel` 4 階、`Classification` 4 階）。
- **JIT 權限**：敏感操作（跨機部署、資料庫備份還原、銀行付款發送）須治理核准；由 `JitPermissionManager` 控制上限與到期。
- **失敗閉鎖**：異常退回人工，不可靜默繼續。
- **限制**：目前無企業 IdP/SSO/MFA/帳號復原/持久化身分資料庫/存取審查/租戶隔離（DEC-12，B1 Blocked）。

---

## §14 資安基線

### §14.1 現況（已驗證）

| 控制 | 現況 | 證據 |
|---|---|---|
| 本機帳密 | PBKDF2-HMAC-SHA256 600,000 iter＋16 byte salt；帳號驗證以常數時間比對 | `local_auth.rs:26, 28, 95-100` |
| Session | 32 byte CSPRNG token（`TOKEN_BYTES=32`）＋獨立 CSRF token（`TOKEN_BYTES`）；3600s；進程記憶體；`HttpOnly`＋`SameSite=Strict` cookie `ekpi_session`；`Secure` 僅當 `EKPI_COOKIE_SECURE=1` | `local_auth.rs:28-29, 208-212`; `web-routes.md` §Server boundary |
| CSRF | 雙提交（`X-CSRF-Token` header 或 `ekpi_csrf` cookie）；登入頁 hidden field + cookie；session 需要 | `governance_web.rs:201-204, 583-649, 848-853` |
| 登入保護 | 失敗回 generic 訊息；lockout 5 次／300s（`MAX_FAILED_LOGINS=5`、`LOCKOUT_WINDOW_SECS=300` 在 `adapters/identity.rs`）；未知使用者亦付 dummy hash 成本 | `web-routes.md`; `backend/README.md` |
| 同源／Host | `same_origin_guard`；`Host` 須為單一 loopback authority；重複／空白／非 UTF-8／非 loopback 拒絕；session cookie 重複／空／非 UTF-8 視為未認證 | `governance_web.rs`; `integration_web.rs` |
| 標頭 | CSP、`X-Content-Type-Options`、`Referrer-Policy`、`X-Frame-Options`、`Cache-Control: no-store`、`Permissions-Policy`、`Cross-Origin-Opener-Policy`、`Cross-Origin-Resource-Policy` | `web-routes.md` |
| Form 限制 | 64 KiB 上限；超量或格式錯誤回 `400` 且不附加事件 | `web-routes.md` |
| 表單 actor 不可信 | mutation 一律用 session 使用者為 ledger actor，忽略表單 `actor` | `governance_web.rs` session_identity |
| 公開路徑 | 僅 `/healthz`、`/metrics`、`/login`、`/assets/`（`is_public_path`，L667）；`/ready` 須登入 | `governance_web.rs:667` |

### §14.2 不可引用為資安證據（C-16）

- `Dev/web/src-tauri/src/main.rs` 的 `run_security_check()`／`run_connection_check()` 為硬編碼 JSON（含 `cert_pinning:true`、`two_way_ssl:true`），**禁止**引用為資安證據。
- `get_account_balance()` 回固定值；bankpay 全回 `MOCK123`。
- `tauri.conf.json` `frontendDist: "../src"` 為誤設定，不得作為部署保證。

### §14.3 落地硬性要求（QG-03）

- `ekpi-platform/platform/src/api.rs`（固定回 `comply`）**永遠不得掛載**；`docs/backend/README.md` 明載 "must not be exposed"。
- 不得將 `local_auth` 說成企業認證；不得將 session 說成租戶隔離。
- 明確區隔：`Dev/platform/src/api.rs`（根 FlapsErp，完全接線）≠ `ekpi-platform/platform/src/api.rs`（未接線 placeholder）；兩者同名不同物，必須以全路徑引用。

---

## §15 可靠性與維運 Runbook（RB-01..RB-10）

| 代號 | 場景 | 自動程序 | 人工覆核 | 證據 |
|---|---|---|---|---|
| RB-01 | 啟停與升級 | 原生 binary 部署；異步備份；版本回滾 | 雙人覆核 | 部署清單、回滾紀錄 |
| RB-02 | 備份與還原 | `scripts/backup.ps1`；還原演練 | 還原演練簽核 | backup bundle 雜湊、還原演練紀錄 |
| RB-03 | 帳鏈竄改應變 | `verify_integrity()` 失敗即停 | INTERNAL_AUDIT 複核 | 驗證報告 |
| RB-04 | outbox 積壓 | `pending_outbox` 監控；重投 | 技術值班 | 監控截圖、重投紀錄 |
| RB-05 | 冪等／重播防護 | `IdempotencyLedger`；`event_id` 去重 | 稽核確認 | 去重紀錄 |
| RB-06 | 隔離清單分流 | `IsolationEntry` 列表；關鍵字分流 | VS-09 覆核 | 分流工作單 |
| RB-07 | 三方對帳差異 | `ThreeWayReconciler`；自動 quarantine | 業務覆核 | 對帳報告（見 §8.5） |
| RB-08 | 事件／事故應變 | 告警 → ESC-1..ESC-4；隔離區間 | 事故指揮 | Incident timeline |
| RB-09 | 憑證與密文輪替 | 過期前 30 日預警 | 雙人操作 | 輪替紀錄 |
| RB-10 | 適配器上線 | 契約測試全綠 → 灰度 → 全量 | 治理簽字 | 契約註冊、簽字單 |

每支 Runbook 必須有：觸發事件、自動程序、人工覆核、證據產出、回滾路徑。未備妥者不得宣稱營運就緒。

---

## §16 資料治理與持續改善

### §16.1 五條 Feedback Loops

| Loop | 來源 | 消費端 | 閉合條件 |
|---|---|---|---|
| F1 使用者 → 需求 | 工作室 1 裏意見 | 工作室 3（政策）＋工作室 2（實作） | 需求 ID 入 `DEC-` 登記 |
| F2 稽核 → 控制 | VS-09 稽核發現 | CONTROL_OWNER | 控制修復＋複驗 |
| F3 資料品質 → 模型 | 隔離清單（RB-06）+ 契約違反 | 業務擁有者 | 主資料修正＋版本發布 |
| F4 效能 → 容量 | 監控（/metrics） | 工作室 2 | 容量模型更新（DEC-19） |
| F5 Retro → Roadmap | 里程碑回顧 | PMO | Roadmap 更新（§17） |

### §16.2 治理控制

- SoA（93 控制）為 `verify_status` 唯一真相來源（ADR-0011 §2.4）；crosswalk 只存差距與處置（`gap`、`disposition`），不得另立實作狀態欄。
- `applicability` ≠ 適用者必填 `exclude_reason`（ADR-0007：不可驗證來源一律標記）。
- 多標準接軌經 crosswalk 18 欄；主鍵 `control_id + standard_ref + std_clause`。
- 指標先有定義與核准目標（BASE-07）；試點負責人簽認基線與結果。

---

## §17 演進路徑與路線圖（PHASE-G*/SLICE-G*）

### §17.1 階段總覽

| 階段 | 主題 | 退出條件（Exit Gate） | 阻塞依賴 |
|---|---|---|---|
| PHASE-G1 | 誠實化收斂（C-12、C-13、§7.1 回退） | QG-07、QG-08 全綠 | 無 |
| PHASE-G2 | Postgres 治理適配器（fail-closed 先行） | QG-02、QG-03 全綠 | 無（需 PG 18.6 環境） |
| PHASE-G3 | 供應 binary 接線（AppConfig、Role 收斂） | QG-04、QG-11 全綠 | 無 |
| PHASE-G4 | API v1 批次 B-B..B-E | QG-09 全綠（逐端點證據） | 契約先行（CTR-API-001 §4） |
| PHASE-G5 | 三工作室制度化（HO/ESC 證據鏈） | QG-12、QG-13 全綠 | 試點（DEC-01） |
| PHASE-G6 | 產業疊層與規模擴展（IND-*、SCALE-2+） | QG-14、QG-15 全綠 | DEC-26 arg、PR-01..PR-13 |

### §17.2 切片（有序，互不並行阻塞外的依賴）

| 切片 | 內容 | 影響邊界 | 驗證 |
|---|---|---|---|
| SLICE-G01 | `/api-docs` 頁重寫（C-12：13 條全列、移除錯誤 501 標籤、單一來源） | `platform/src/governance_web.rs`（api-docs 頁）＋測試 | QG-07 |
| SLICE-G02 | `/settings` 頁誠實化（C-13：Postgres 標註未接線） | 同上（settings 頁） | QG-08 |
| SLICE-G03 | C-08 AGENTS.md 清理（workspace/edition/DATABASE_URL） | `ekpi-platform/AGENTS.md` | QG-07 |
| SLICE-G04 | C-19 architecture README 清理（no server-side database） | `docs/architecture/README.md` | QG-07 |
| SLICE-G05 | Postgres fail-closed 化：`todo!()` → `Err(Storage("not implemented in postgres adapter"))`；`verify_integrity()` 不得無條件 `Ok(())` | `crates/adapters/src/postgres/governance.rs` | QG-02 |
| SLICE-G06 | `AppConfig::from_env` 死碼處理（呼叫或刪除） | `crates/adapters/src/configuration.rs`＋`platform/src/bin/governance_web.rs` | QG-04 |
| SLICE-G07 | `web` crate 金額 i64（C-17）＋孤兒目錄清理（C-04）＋`tauri.conf.json` 修正（C-18） | `Dev/web/**` | QG-05、QG-06 |
| SLICE-G08 | C-22 AGENTS.md 同步（9→10 子命令）＋ C-09 phase-gates 同步 | `Wellfit/AGENTS.md`、`phase-gates.md` | QG-07 |
| SLICE-G09 | 雙 Role 收斂（C-15，DEC-23） | `crates/domain/src/auth.rs`、`platform/src/local_auth.rs` | QG-11 |
| SLICE-G10 | ChannelAdapter trait 統一（C-07，ADR-0004，DEC-21） | `crates/domain/src/channels.rs`、`platform-hub/crates/channels` | QG-14 |
| SLICE-G11 | `apps/web` 去留（C-21，OQ-GOV-09） | `apps/web` | QG-13 |
| SLICE-G12 | API v1 寫入端點（B-B） | `platform/src/governance_web.rs`、`crates/application` | QG-09 |
| SLICE-G13 | API v1 事件／治理／檢索（B-C..B-E） | 同上 | QG-09 |
| SLICE-G14 | Postgres 治理適配器真實 SQL（PHASE-G4） | `crates/adapters/src/postgres/`、`migrations/` | QG-02、QG-03 |
| SLICE-G15 | KB crosswalk 落地（VS→process_area、IND→policy pack） | 對接 KB `crosswalk.csv` | QG-14 |
| SLICE-G16 | 試點與驗證（DEC-01、A1） | `docs.traceability/register.md` | QG-12、QG-13、QG-15 |

### §17.3 失敗模式（每切片必考）

- Postgres 未啟用 → 一律 JSONL（不得假裝已接線）。
- `verify_integrity()` 回 `Ok(())` 在修復前不得出現在稽核頁。
- 契約未定 → 保持 501；禁止先上線後補契約（CTR-API-001 §4）。
- 任一 f64 金額 → QG-05 阻擋。

---

## §18 風險登記冊（RSK-G01..RSK-G14）

> 前綴 `RSK-G` 與 FlapsErp `RSK-E*`／管理文件 `RSK-01..07` 區隔，避免命名空間碰撞。

| 代號 | 風險 | 等級 | 緩解（本計畫內） | 關閉條件 |
|---|---|---|---|---|
| RSK-G01 | Studio 2 成為技術瓶頸 | 高 | 自助＋例外審核；HQ 例外僅重大變更需核准 | 佇列工時未超閾值 |
| RSK-G02 | Studio 3 誤判合規條文 | 中 | advisory-only＋雙人複核 | 複核一致率 ≥ 99% |
| RSK-G03 | 電商與規格邏輯衝突 | 高 | 僅於 ApprovalTask 層關聯；衝突進隔離清單 | 隔離清單週期清理 |
| RSK-G04 | 權限集中於 Studio 2 Admin | 高 | Admin 雙人核准；全操作寫入 audit_chain | 持續有效 |
| RSK-G05 | Agent 誤判（R-05） | 高 | 高風險強制人工，fail-closed，完整稽核 | DEC-15 解鎖前須有 QG-10 |
| RSK-G06 | momo 規格未取得（R-01） | 高 | Phase E 標條件式；B/C/D 先行；掛 OQ-F2-5 | 2026-11-19 前未回覆即維持條件式 |
| RSK-G07 | 部署目標未核准（R-15） | 高 | 僅 loopback；跨機另立 Phase G | PR-02 |
| RSK-G08 | `todo!()` runtime panic（C-06） | 高 | SLICE-G05 先 fail-closed 化 | QG-02 |
| RSK-G09 | `verify_integrity()` 偽通過（C-14） | 高 | SLICE-G05 修復 | QG-02 |
| RSK-G10 | `AppConfig` 死碼（C-13） | 中 | SLICE-G06 | QG-04 |
| RSK-G11 | 雙 Role 語義漂移（C-15） | 中 | SLICE-G09 | QG-11 |
| RSK-G12 | `web` crate f64（C-17） | 中 | SLICE-G07 | QG-05 |
| RSK-G13 | Mock 安全宣稱（C-16） | 高 | 標註 mock；不得引用 | QG-06 |
| RSK-G14 | 四套 workspace 技術棧漂移 | 中 | ADR-0001 基線維持；DEC-22 追蹤 | DEC-22 裁決 |

---

## §19 未決決策與阻塞（DEC-21+／OQ-GOV-*）

### §19.1 既有未決（canonical：`docs/governance/08-decision-register.md`）

| 編號 | 主題 | 裁定負責人 | 未決處置 | 影響 |
|---|---|---|---|---|
| DEC-01 | 試點部門／流程／贊助人 | 業務贊助人 | 只做樣例建模 | FR-01/04/07 |
| DEC-02 | 目標族群與角色分工 | 業務主管／HR／PMO | 角色基線草稿；權限測試 BLOCKED | FR-03..06, NFR-03 |
| DEC-03 | 風險分類與例外層級 | 風險主管／QA／法遵 | 風險欄位僅記錄 | FR-01/05/11 |
| DEC-04 | 專案欄位必填與基線簽署 | PMO／資料擁有者 | 維持草稿；不得進 G1 | FR-01/07 |
| DEC-05 | 文件來源／版本／保存 | 文件控制者／法遵 | 人工確認並標記 | FR-02/03/08, NFR-04 |
| DEC-06 | 交付物與 G1–G5 證據 | 流程擁有者／PMO／QA | 通用模板原型 | FR-04/05 |
| DEC-07 | 29 項 QC 適用範圍／N/A | QA／流程擁有者 | N/A 須理由 | FR-11 |
| DEC-08 | 風險／變更／CAPA SLA 與門檻 | QA／風險負責人 | 只記錄 | FR-06 |
| DEC-09 | 基線期／指標目標／驗收者 | 贊助人／PMO | 儀表板 N/A | FR-07 |
| DEC-10 | MS/Google SKU／API | IT／資安 | 僅人工來源參照 | FR-08, NFR-06 |
| DEC-11 | 優先接入工具排序 | IT／PMO | 不開發未排序連接器 | FR-08 |
| DEC-12 | IdP／MFA／租戶 | IT／資安 | 不得部署多使用者 | NFR-03 |
| DEC-13 | 資料分類／保存／個資 | 資料治理／法遵／法務 | 不輸入敏感真實資料 | NFR-04/05/07 |
| DEC-14 | 稽核防竄改／RPO/RTO | IT 營運／資安 | 本機 hash 僅原型 | FR-10, NFR-07 |
| DEC-15 | AI 供應者／模型／資料層級 | AI 負責人／資安／法遵 | AI 關閉 | FR-09, NFR-05 |
| DEC-16 | SLO／支援時段／監控 | 系統擁有者／IT | 不作營運承諾 | NFR-07 |
| DEC-17 | 無障礙義務／瀏覽器矩陣 | UX／IT／無障礙負責人 | WCAG 2.2 AA 基線 | NFR-01/02 |
| DEC-18 | 法規適用產業／司法管轄區 | 法遵／法務／業務 | 只用通用基線 | FR-05/06/11 |
| DEC-19 | 容量模型（使用者／併發／匯出） | 系統擁有者／IT／PMO | 不宣稱容量 | NFR-08/09, FR-10 |
| DEC-20 | 威脅模型／ASVS 等級 | 資安架構師 | 不作安全宣稱 | NFR-03/10 |

### §19.2 本計畫新增 — 待併入 canonical 登記

| 提案編號 | 主題 | 所需裁定 | 未決處置 |
|---|---|---|---|
| DEC-21（提案 ADR-0004） | platform-hub 落點（是否留 `Dev/platform-hub/`）與 `ChannelAdapter` trait 統一（C-07、C-10） | 架構決策 | 以 `platform-hub` typed `UnifiedOrder` 為準；ekpi-platform 側標骨架 |
| DEC-22 | 四套 Cargo workspace 是否統一；長遠架構（B3/B4 是否併入 B1/B2） | 工作區擁有者 | 維持 ADR-0001 基線並存；ADR-0001 本身不作改 |
| DEC-23 | 雙 Role 收斂（C-15）— 登入層與授權層分工 | 安全架構 | 登入用 local_auth；授權以 domain Permission 為準 |
| DEC-24 | Postgres 適配器 go-live 時機與 fail-closed 政策（C-06、C-14） | 系統擁有者／資安 | PG 未上線前一律 JSONL |
| DEC-25 | KB 跨標準對照表 18 欄在 ekpi-platform 的落地（VS→process_area、IND→policy pack，見 §12.2） | 法遵／資料治理 | 僅參照，不自動寫入 SoA |
| DEC-26 | 產業疊層（IND-*）以 policy pack 參數化而非 fork | 架構決策 | 禁止按產業 fork 程式 |
| DEC-27 | `/api-docs`、`/settings` 頁面由單一來源產生（見 §11.4／§11.5） | 工程決策 | 誠實化頁面；不得與契約矛盾 |
| DEC-28 | VS-01..VS-09 與既有 12 階段／G1–G5 的映射基準 | 流程擁有者／PMO | 以 §12.5 映射為準 |

### §19.3 本計畫新增開放問題（OQ-GOV-*）

| 提案編號 | 問題 |
|---|---|
| OQ-GOV-01 | 部署目標（呼應 PR-02） |
| OQ-GOV-02 | `platform-hub/apps/cli` 是否建 |
| OQ-GOV-03 | Redis/Fly-`TRACTION` 的 Windows 落點（Memurai vs PG 快取，DEC-14） |
| OQ-GOV-04 | Kafka 啟用門檻（>5,000/s 或 backlog >100k） |
| OQ-GOV-05 | PChome／Cyberbiz 規格未取得 |
| OQ-GOV-06 | 備援 RPO／RTO 核定（DEC-14） |
| OQ-GOV-07 | 本機 Postgres 初始資料遷移 |
| OQ-GOV-08 | KB 私有 remote 位置 |
| OQ-GOV-09 | `apps/web` 去留（C-21） |
| OQ-GOV-10 | 12 階段模版自訂與 SOI 映射 |
| OQ-GOV-11 | SQLite（B3/B4）與 PostgreSQL（B1/B2）長期並存與否 |
| OQ-GOV-12 | `sqlx` compile-time check 在離線 CI 的處理 |
| OQ-GOV-13 | `check-docs` 覆蓋率 |
| OQ-GOV-14 | `crates/adapters/src/postgres/identity.rs` 與 `local_auth` 的長期關係 |

### §19.4 PR（Production Readiness）映射

`PR-01..PR-13`（見 `docs/operations/production-readiness.md`）為生產就緒的 canonical 登記。PR-01（範圍與完成定義）、PR-02（部署目標）、PR-03（身分與授權）為前置，解鎖 PHASE-G5／G6。其餘 PR（PR-04..PR-13）沿用既有登記，不在此複寫。

---

## §20 品質閘門總表（QG-*）

| 閘門 | 判定指令／證據 | 通過標準 | 關閉條件（未達即阻擋） |
|---|---|---|---|
| QG-01 | `grep -r "adjudicate_case" platform/src/api.rs` 期望或其掛載 | `platform/src/api.rs` 未掛載 | 任一掛載即阻擋 |
| QG-02 | `grep -rn "todo!" crates/adapters/src/postgres/` | **0 命中** | 任何命中即阻擋（RSK-G08） |
| QG-03 | `postgres/approval.rs`、`postgres/channels.rs` 不含 `Err(Storage("not yet implemented"))` | **0 命中** | 任何命中即阻擋 |
| QG-04 | `grep -rn "AppConfig::from_env" crates/ platform/` 有呼叫者（或已刪除 AppConfig）；供應 binary 印出「EKPI_STORAGE=postgres」啟用訊息 | 以程式碼為據 | 無呼叫者即阻擋（C-13） |
| QG-05 | `grep -rn "f64" src/ src-tauri/src/ --include="*.rs"`（`Dev/web`）＋金額路徑 f64 | **0 命中**（金額／數量欄位） | 任何命中即阻擋（C-17） |
| QG-06 | `grep -rn "cert_pinning\|MOCK123\|two_way_ssl" src-tauri/src/` | **0 命中**（或標 `mock_mode:true`） | 任何命中即阻擋（C-16） |
| QG-07 | 契約對齊：`/api-docs` 頁表格 row count = 13；兩個已接線端點標「已接線（唯讀）」；AGENTS.md workspace/edition/DATABASE_URL 與 manifest 一致；`architecture/README` 無「no server-side database」 | 4 項全綠 | 任一未綠即阻擋 |
| QG-08 | `/settings` 頁「PostgreSQL／EKPI_STORAGE」字串在未接線前不會出現未標註的啟用宣稱 | 以頁面檢查為據 | 未標註即阻擋 |
| QG-09 | API v1 合約測試：每端點由 501 轉 200 前，契約、冪等、重放防護、錯誤結構全覆蓋 | 一端點一證據 | 缺任一即阻擋 |
| QG-10 | `grep -ri "auto_approve" ekpi-platform/crates/`（大小寫不敏感） | **0 命中** | 任何命中即阻擋（DEC-15 未決） |
| QG-11 | 雙 Role 收斂完成前：所有授權判定經過 `Permission::minimum_role()`；無散見 `role >= Role::X` 手寫比對 | grep 審查 | 手寫比對即阻擋 |
| QG-12 | 試點（A1）證據：真實使用者完成工作流並記錄（DEC-01） | 證據連結 | 未有即阻擋（A1 仍 `Not met`） |
| QG-13 | `apps/web` 去留決策（OQ-GOV-09）或已併入工作台 | 二選一完成 | 未決即阻擋 |
| QG-14 | `ChannelAdapter` trait 統一（ADR-0004，DEC-21） | 一個正據 | 未統一前 ekpi-platform 側必須標骨架 |
| QG-15 | 三方對帳週期性執行（§8.5）且隔離清單屬實 | 報告 | 未執行即阻擋 |
| QG-16 | 每 `*.rs` 首行 `#![forbid(unsafe_code)]` | 全掃描 | 缺行即阻擋 |

---

## §21 驗證計畫

### §21.1 指令矩陣（每 workspace）

| Workspace | 指令 | 通過標準 |
|---|---|---|
| ekpi-platform | `scripts/check.ps1` | 0 failed / 0 ignored |
| ekpi-platform | `cargo run -p ekp_platform -- --selftest` | 49 assertions PASS |
| ekpi-platform | `cargo run -p ekp_platform -- --declared-rules-schema` | schema OK |
| ekpi-platform | `cargo run -p ekp_platform -- --eval --test-set data/eval/search_eval.json` | 28 案例通過（20/20、8/8、4 role、0 violations） |
| platform-hub | `cargo clippy --workspace --all-targets -- -D warnings` ＋ `cargo test --locked` | 0 警告；全通過 |
| Wellfit | `build.ps1 -Command "clippy --workspace --all-targets -- -D warnings"` ＋ `build.ps1 -Command "test --workspace --no-fail-fast"` | 0 警告；全通過 |
| 根 Dev | `cargo fmt -- --check` ＋ `cargo clippy --all-targets -- -D warnings` ＋ `cargo test --locked` ＋ `cargo audit` | 全綠 |
| Knowledge | `kb-verify.ps1` | 全綠 |

### §21.2 不變量掃描（QG-05、QG-16 等）

- `#![forbid(unsafe_code)]`：每 `.rs` 首行。
- clippy 零警告；不得用 `#[allow(...)]` 壓縮。
- `<script>` 計數 = 0（工作台）。
- 金額不使用 `f64`（QG-05）。
- 不出現 `Bearer`（QG-01 衍生）。
- 不出現 `auto_approve`（QG-10）。

### §21.3 合同測試節點

| 契約 | 測試 | 覆蓋點 |
|---|---|---|
| `ColumnContract` | wellfit 單元＋整合 | 必填缺漏硬失敗 |
| 冪等 | event replay | 同 event_id 重投僅生效一次 |
| Agent 邊界 | `grep auto_approve` | 0 命中（QG-10） |
| AP 501 誠實 | `unwired_api_endpoints_return_honest_501_never_fabricated_data`（`integration_web.rs:438`）；`wired_api_list_endpoints_serve_real_contracted_data`（:793） | 未接線 501、401、200 形狀 |
| 莫偽造 | `cargo test` 內備有攤位禁止（`cargo test` 全綠） | — |

### §21.4 執行環境

- 每個 workspace 獨立 `cargo test`；不要跨 workspace 的併行 test 依賴未定義行為。
- Windows：`cargo fmt -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`。
- ekpi-platform：`cargo build` 需 PG 環境支援 migrations；若 PG 不可用，以 JSONL 為據。

---

## §22 追溯矩陣

### §22.1 需求 → 計畫章節 → 證據

| 需求 | 本計畫章節 | 程式碼／契約／文件證據 | 驗證 | 狀態 |
|---|---|---|---|---|
| FR-01 | §3 VS-01、§9.2 | `platform/src/governance.rs` `Project`；`governance_web.rs` `/projects` | — | Partial |
| FR-02 | §3 VS-02、§9.2 | `Evidence`；`docs/contracts/data-formats.md` CTR-DATA-001..004 | — | Partial |
| FR-03 | §12 | `EventPayload` 9 變體；`EvidenceReviewed` | — | Partial |
| FR-04 | §10 組 A（5 場景） | `WorkflowDefinition::validate` | `valid_definition_passes_validation` 等 | Partial |
| FR-05 | §12.3、§12.5 | `governance.rs` G1-G5 whitelist；`WorkflowStage`；12 階段（AT-02） | AT-02..05 | Partial |
| FR-06 | §9.2 VS-08、§16 | `QualityCase`、`QualityInspection` 29 項 | AT-15..18 | Partial |
| FR-07 | §16、§20 QG-12 | — | 儀表板 N/A | Not started |
| FR-08 | §8、§11 | 對接器契約；`docs/contracts/api.md`、CTR-WEB-001 | AT-06/07/21 | Partial |
| FR-09 | §10.3、§14 | `AgentAdvisor` 僅 advise；`agent-policy` advisory_only | advisory-only 測試 | Partial |
| FR-10 | §8.6 | `Ledger` SHA-256 帳鏈；JSONL 封存山形 | — | Partial |
| FR-11 | §6.2 | `QualityCheckDefinition` 29 項 | AT-23.. | Partial |
| NFR-01..NFR-11 | §13、§14、§15、§21 | `05-functional-spec-v1.md` §6；本計畫 §22 為 NFR-01..11之正據（見 C-20） | — | Partial/Planned |

### §22.2 本計畫新 ID → 章節索引

| ID | 定義章節 | 落地（程式／契約） | 驗證 |
|---|---|---|---|
| VS-01..VS-09 | §3 | `platform-hub/udm`、`wellfit_core` | QG-15 流程試跑 |
| IND-1..IND-6 | §3.3 | policy pack（資料） | QG-14 |
| SCALE-1..SCALE-4 | §3.4 | —（制度） | QG-13 |
| HO-01..HO-08 | §5.2 | `ApprovalTask` 參照 | QG-12 |
| ESC-1..ESC-4 | §6.3 | `TimeoutAction`、`TaskAction::Escalate` | QG-12 |
| RB-01..RB-10 | §15 | script＋演練 | QG-12 |
| QG-01..QG-16 | §20 | —（閘門） | — |
| RSK-G01..RSK-G14 | §18 | —（風險） | — |
| PHASE-G1..G6 | §17.1 | — | — |
| SLICE-G01..G16 | §17.2 | 見 §17.2 表 | 見 §20 |
| DEC-21..DEC-28 | §19.2 | 須併入 `08-decision-register.md` | — |
| OQ-GOV-01..14 | §19.3 | 須併入 `08-decision-register.md` | — |

> **規定**：每條需求須經 requirement ID/PRD → system design/ADR → contract → ticket → implementation/Git commit → automated test → release evidence → retro（見 `project.md` traceability_chain）。缺鏈者不得標 Verified；「測試通過」不等於「使用者價值已驗證」（Phase A 穩健性 未滿足）。

---

## §23 參考文件

- `1793782615513-wellfit-unified-plan-v2.md`（上游統一計畫 v2）
- `ekpi-platform/AGENTS.md`、`Wellfit/AGENTS.md`
- `Knowledge/adr/ADR-0009-spec-derivation-vs-arbitration-separation.md`
- `Knowledge/adr/ADR-0010-iso-27001-isms-baseline-single-control-library.md`
- `Knowledge/adr/ADR-0011-compliance-naming-and-crosswalk-schema.md`（crosswalk 18 欄、主鍵）
- `Knowledge/adr/ADR-0012-isms-authority-precedence.md`（IMS 權威、G0–G7、9 角色、別名）
- `Knowledge/sources/research-protocol.md`（§1.1：契約事實禁網路推測）
- `ekpi-platform/docs/adr/0002-unified-integration-architecture.md`、`0003-ecommerce-integration-boundary.md`
- `ekpi-platform/docs/contracts/approval-api.md`（CTR-API-001）、`web-routes.md`（CTR-WEB-001）、`data-formats.md`（CTR-DATA-*）、`README.md`（契約清單）
- `ekpi-platform/docs/governance/02-requirements-v0.1.md`（FR-01..11、NFR-01..07 bullet、QC-01..29）
- `ekpi-platform/docs/governance/05-functional-spec-v1.md` §6（NFR-01..11 expanded）
- `ekpi-platform/docs/governance/06-data-and-state-model-v1.md`（12 階段、專案狀態機、G1–G5）
- `ekpi-platform/docs/governance/08-decision-register.md`（BASE-01..08、DEC-01..20）
- `ekpi-platform/docs/governance/10-traceability-baseline.md`（SN 對應、NFR-01..11）
- `ekpi-platform/docs/quality/phase-gates.md`（Phase A 未核准、Phase B Blocked、A1..A11、B1..B8）
- `ekpi-platform/docs/quality/bdd-scenarios-approval.md`（26 場景 A1–D7）
- `ekpi-platform/docs/product/redesign-v2-master-plan.md`（S1–S6 切片）
- `Dev/docs/adr/ADR-0001-tech-stack-baseline.md`（D-01..D-05；已知 cargo audit 風險：RUSTSEC-2026-0258 h2 / RUSTSEC-2023-0071 rsa）
- `Dev/docs/PLAN_REGISTER.md`
- `Dev/.github/workflows/flexible-ci.yml`（pwsh；RUSTFLAGS -D warnings；os 矩陣 ubuntu+windows；branches main/develop/test/release；env choices develop/test/release/production；config `.github/config/ci-config.yml`）
- `Dev/.github/CODEOWNERS`（docs→architecture、core/platform→backend、web→frontend、crates→bridge、scripts→ops）

---

## §24 變更紀錄與執行順序

### §24.1 v2.1 → v3.0 變更摘要

| 項目 | v2.1 | v3.0 |
|---|---|---|
| API v1 端點數 | 11（9 條表）已實作 | **13 method-routes；2 已接線；11 × 501**（C-01）；正表見 §11.2 |
| 程式化認證 | Bearer API key | **session cookie only**（C-02；見 §11.3） |
| Phase D3 | 已完成 | **不存在**（C-03），回退 |
| Phase D1 | 已完成 | **未做**（C-04，孤兒目錄） |
| Phase C1–C5 | 全通過 | **部分**（C-06：9×todo!()、8×stub、1×stub；真實 SQL 僅 identity.rs） |
| Phase D2 | 全通過 | **部分**（C-17：web crate 未收斂） |
| api-docs | 已實作 | **內容誤導**（C-12），列 SLICE-G01 |
| settings | 已實作 | **內容誤導**（C-13），列 SLICE-G02 |
| §5.1 契約面 | Bearer 敘述 | session cookie（見 §11.3） |
| §6.2 端點數 | 11 已實作 | 13／2／11（見 §11.2） |
| §7.1 落地路徑 | 寬鬆表述 | 逐項回退（見 §7.2） |
| 新 ID | — | VS/IND/SCALE/HO/ESC/RB/QG/RSK-G/PHASE-G/SLICE-G/DEC-21+/OQ-GOV（見 §0.3、§24.1 C-23） |
| 章節數 | 10 | 24（見目錄） |

### §24.2 新增 C-23（ID 命名空間碰撞校正）

本計畫原草稿新增 ID（`D1–D9`、`I1–I6`、`S1–S4`、`H1–H8`、`RSK-*`）經 grep 實證與現有 ID **碰撞**（詳見 §2 C-23）。為遵守憲章「ID 唯一性」與「禁止平行命名空間」（每個概念只能有一個正據名稱），本計畫改用已驗證無碰撞的前綴（見 §0.3）；原草稿的 ID 前綴**不得**被引用為有效 ID。

### §24.3 執行順序（有序清單，可交付給實作代理）

> 規劃文件不會自動執行；必須依序實作並在每步後加陰影驗證。以下順序已依「先誠實、後實作、先 fail-closed、後放量」排序。

1. **誠實化頁面**（SLICE-G01、G02、G03、G04、G08）：`/api-docs` 全 13 條＋正標已接線；`/settings` 加註 PG 未接線；清理 AGENTS.md 與 architecture README 誤導段落；v2.1 本身即以本檔取代。
2. **Fail-closed 化**（SLICE-G05）：postgres `todo!()` → `Err`；`verify_integrity()` 不再無條件 `Ok(())`。
3. **接線封測**（SLICE-G06）：`AppConfig::from_env` 呼叫或刪除；供應 binary 讀取 EKPI_STORAGE/DATABASE_URL。
4. **金額收斂**（SLICE-G07）：`web` crate f64 → i64；孤兒目錄清理；`tauri.conf.json` 修正。
5. **API v1 收斂**（SLICE-G12、G13）：批次 B-B..B-E，每端點一契約。
6. **Postgres 治理適配器**（SLICE-G14）：governance／approval／channels 真實 SQL（保留 JSONL fallback）。
7. **制度化**（SLICE-G15、G16）：KB crosswalk 落地（VS → process_area、IND → policy pack）、試點與驗證（A1）、更新 traceability register。

完成順序即終端狀態：Phase A 仍未核准，必須以絕對誠實的狀態推進，不得包裝。
