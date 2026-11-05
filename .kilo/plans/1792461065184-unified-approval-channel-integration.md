# 統一專案管理簽核系統與電商平台整合 — 執行計畫

## 1. 現況基線（2026-11-05 實測核對，非假設）

本節為「現況事實」，實作時不得推翻，除非另立 ADR。

| 系統 | 路徑 | 現況事實 |
|------|------|----------|
| Wellfit | `Wellfit/` | **9-crate** Rust 工作區（非 5-crate）：`wellfit_core` / `wellfit_shopee` / `wellfit_reports` / `wellfit_mtrix` / `wellfit_cli` / `wellfit_sync` / `wellfit_bankpay` / `wellfit_assist` / `wellfit_integration`。CLI 已有 10 子命令（status/shopee/reports/mtrix/audit/spec/access-control/jit/bankpay/shadow）。235+ tests passed |
| EKPI | `ekpi-platform/` | 2 crate（`platform` / `eval`），**無根 Cargo workspace**。已實作：確定性裁決引擎、治理狀態機（12 階段/5 關卡 G1–G5）、`governance_web` 工作台（Viewer/Editor/Approver 分權、PBKDF2 600k、CSRF、same-origin）、SHA-256 追加式稽核、WORM、`governance.rs:365` Ledger。`api.rs:49` 僅 3 條路由 |
| 根工作區 | `Dev/` | FlapsErp RPA 線：`core`/`platform`/`web`/`crates/flaps_bridge`（edition 2021／resolver 2／axum 0.7／sqlx 0.8 SQLite／Tauri 2.0）。**現已是 git repo**（remote `it-wellfit/dev.git`），已有 develop/test/release/main 四分支 + `ci.yml` 多環境管線。`platform/src/ai/decision_gating.rs` 有 AI 簽核閘（`requires_human_approval`、`Action::EscalateToHuman`、`auto_approval_confidence: 0.95`）但 **`web/src-tauri/src-tauri/commands.rs:156` 的 `approve_transaction` 是 stub（`TODO: Implement actual transaction approval logic`）** |
| SyncMesh | `SyncMesh/` | 4-crate（core/net/storage/cli），Ed25519 + CRDT version vector + SQLite content-addressed store。**已被 `wellfit_sync` 取代**，本計畫不納入 |
| 既有簽核機制 | `Wellfit/crates/wellfit_core/src/` | `access_control.rs`（RBAC：PermissionLevel/Classification/SodConstraint/AccessController）與 `jit_permission.rs`（JIT：request/approve/reject/revoke/expire_stale_requests）**已存在且可重用** |
| 知識庫 | `Projects/2026/Knowledge/` | 跨專案 SSOT，17 個子目錄（adr/sources/specs/reconciliation/issues/traceability/compliance/tools/...），含 12 份 ADR、87 項議題登錄、ISO 27001 合規套件。**位置已由 ADR-0004（Approved）裁定，引用方向單向**（專案→KB）。**但不在 git 版控內**（實測 `fatal: not a git repository`） |
| 金額整數化 | `Dev/core/src/money.rs`、`Dev/platform/src/api.rs` | ADR-0002 已落地：`amount_minor: i64`（整數分）＋ `exchange_rate_scaled: i64`（10^4），`parse_decimal_to_minor` 純字串運算。**殘留 f64 僅 2 處**：`core/src/ledger/account.rs:57` `get_balance() -> f64`、`platform/src/ai/decision_gating.rs:32,34` `TransactionItem.amount/exchange_rate` |

### 已確認缺口（grep／實測）

1. **momo / PChome / Cyberbiz 零實作**：全 repo 僅出現於 `docs/research/輔翼ERP逆向分析與最佳化方案_v1.0.md:79,194`、`docs/research/proposals/OQ-F1_F2_輔翼ERP正式需求提案_v1.0.md:70`（列為 P2 待索取項）與 MDM 渠道碼 `995E`(MOMO)/`995F`(PCHOME)，後者明確標為 `UNVERIFIED_CHANNELS`（須對接方確認）。
2. **Kafka / PostgreSQL / Redis / NATS 全未導入**：全 repo 唯一命中為 `.cargo/audit.toml:12` 註解。現況為 SQLite（`rusqlite` bundled）+ JSONL ledger。
3. **文件漂移**：`ekpi-platform/docs/project-management-optimization.md:20` 宣稱「6 個 crate (apps/web, crates/domain, crates/application, crates/adapters)」— 該工作區實際不存在（僅 `platform`/`eval`）；同檔 §4 建議 Python 處理非 ASCII（違反 Rust-first 修正）、§8 建議 Docker 部署（違反原生部署決策）。
4. **知識庫無版控且 CI 抓不到**：`Knowledge/` 非 git repo；`ci.yml` 全檔 grep `Knowledge` → **0 命中**（僅 `defaults.run.shell: pwsh`）。四環境 CI 在**缺少規格事實**下建置測試；ADR-0004 宣稱「可版本控管」不成立。
5. **三套平行 ADR／治理儲存處**：`Dev/docs/adr/`（ADR-0001~0011，根工作區範圍）、`Knowledge/adr/`（KNB-ADR-001~012，跨專案範圍）、`ekpi-platform/docs/adr/`（ekpi 範圍）。三者互不交叉引用（`ekpi-platform` grep `Knowledge` → 0 命中）。

## 2. 已定案決策

| 決策 | 裁定 | 依據 |
|------|------|------|
| 基礎設施 | **PostgreSQL 18 為共享事實來源 + 事件表（outbox/inbox）作可重播骨幹；Redis 僅用於快取與分散式限流；全部原生 binary 部署（sc.exe/systemd/launchd）；Kafka 降為未來可插拔適配器，預留不實作** | 本 session 裁定；推翻早前 Kafka+k3s 提議以符合「不使用 Docker」決策 |
| 電商整合邊界 | **通用骨架（ChannelAdapter trait + UDM + 規則引擎）先行，momo 為第一個完整實作（須先取得其 API 規格）；PChome/Cyberbiz 待規格取得後套同一骨架** | 本 session 裁定；遵守 `Knowledge/sources/research-protocol.md` §1.1（契約事實禁止網路推測） |
| 三系統整合 | Wellfit（執行面/資料面）、EKPI（治理面/簽核面）、根工作區 FlapsErp（ERP 執行面）**各自獨立演進**，以 PostgreSQL + 事件表 + 窄 API 契約互通；禁止跨 repo 直接依賴 | 本 session 裁定；`cargo_workspace_architecture` 決策要求依賴方向 Presentation→Application→Domain |
| 簽核狀態機 | 動態多級（>5）、多急、條件分支、超時升級、定期提醒/清理、Agent 判斷免簽核、付費動作權限控管 | 本 session 裁定 |
| 技術基線 | **新增程式碼一律沿用根工作區現行基線**：Rust edition 2021、Cargo resolver 2、axum 0.7、sqlx 0.8、Askama 0.16、tokio 1.0；**Rust-first，禁止 Python** | ADR-0001（Accepted，2026-11-03）D-02 明定「維持 edition 2021／resolver 2／axum 0.7／SQLite；升級至 2024/Axum 0.8/PostgreSQL 列為獨立 POC，需先有明確收益證據」。本計畫**不得**擅自升級 |
| PostgreSQL 採用方式 | 本計畫新增的跨系統整合層（`platform-hub`）以 PostgreSQL 18 為共享事實來源，屬**新系統的技術選型**，不構成對根工作區既有 SQLite 基線的變更；根工作區維持 SQLite（ADR-0001 D-02 不變） | ADR-0001 §3.2；ADR-0001 §6 要求升級須獨立 POC |
| 知識庫版控與多環境共用 | **位置不動（ADR-0004 已 Approved，不重開）；`Knowledge/` 單獨初始化 git（私有 remote）；CI 以 SHA 釘選消費，確保 develop/test/release/main 取得與程式碼同一 commit 的規格事實；敏感檔（實體交付位元組）排除版控，改以 SHA-256 登錄引用** | 本 session 裁定；ADR-0004 §2；KB README §6.5 機敏資料規約 |
| 硬性不變量 | 每個 `.rs` 第一行 `#![forbid(unsafe_code)]`；`clippy --all-targets -- -D warnings` 零警告（不得加 allow）；金額/數量禁用浮點（根工作區用 `flaps_core::money`、Wellfit 用 `wellfit_core::scaled::ScaledDecimal`）；展示層零 JS；欄位以名稱解析禁止序位退讓 | `Wellfit/AGENTS.md` §4、憲章 §2.2–2.3、ADR-0001 §3.4、ADR-0002 §3.4 |

## 3. 目標架構

```
┌──────────────────── EKPI 治理面 (ekpi-platform/platform) ────────────────────┐
│ governance_web (CSS-first, 零JS)  │ 裁決引擎  │ audit.rs (SHA-256 追加式)   │
│ 12階段/5關卡 → 擴充為動態多級簽核工作流引擎                                │
│ 新增: workflow.rs (動態簽核圖) / escalation.rs / agent_policy.rs            │
└───────────────┬──────────────────────────────────────┬──────────────────────┘
                │ 窄 API 契約 + PostgreSQL 事件表         │
                │ (outbox/inbox, 可重播)                  │
┌───────────────▼──────────────────────────────────────▼──────────────────────┐
│            共享整合層 (新: Dev/platform-hub 或獨立 crate)                    │
│ crates/channels/  ── ChannelAdapter trait + UDM + 規則引擎 + momo 適配器     │
│ crates/eventbus/  ── Outbox/Inbox 事件骨幹（PostgreSQL）+ Redis 快取適配    │
│ crates/workflow/  ── 跨系統簽核工作流（動態多級/多急/升級/Agent 判斷）       │
└───────────────┬─────────────────────────────────────────────────────────────┘
                │
┌───────────────▼─────────────┐  ┌──────────────────────┐  ┌────────────────┐
│ Wellfit 執行面 (9 crates)     │  │ 根工作區 FlapsErp    │  │ Windmill (原生) │
│ core(帳本/ReleaseGate/合約)   │  │ core/platform/web    │  │ + PowerShell    │
│ shopee/reports/mtrix/bankpay  │  │ AI decision_gating   │  │ + wellfit.exe   │
│ assist(多格式/異常/影子)       │  │ stubs → 真實簽核      │  │ 排程觸發        │
│ integration(對帳/切換)         │  │                       │  │                │
└───────────────────────────────┘  └──────────────────────┘  └────────────────┘
```

### 職責邊界（不可越界）

| 層 | 擁有 | 不做 |
|----|------|------|
| EKPI 治理面 | 簽核工作流、證據審查、風險/變更/CAPA、稽核鏈、裁決 | 不碰電商 API、不碰 ERP 資料格式 |
| 共享整合層 | ChannelAdapter/UDM/規則引擎、事件骨幹、跨系統簽核工作流 | 不重複 EKPI 裁決邏輯、不重複 Wellfit 帳本 |
| Wellfit 執行面 | 資料契約、冪等帳本、Release Gate、ERP 匯入/匯出、多格式轉換 | 不做簽核政策、不做平台 API 認證 |
| 根工作區 | FlapsErp RPA、ERP 業務查詢 | 簽核邏輯實作（目前是 stub） |

## 4. 執行工作清單

### Phase 0 — 知識庫版控與多環境 SHA 釘選消費（先行，阻擋一切可追溯宣稱）

> 直接回應「知識庫是否拉出來、test/dev/prod 共用」：**位置已拉出（ADR-0004），共用機制缺版控**。本階段補齊，位置不動。

- **A0-1** `Projects/2026/Knowledge/` 初始化獨立 git repo（`git init` + `.gitignore` + 初始 commit），建立**私有** remote（不得推公開 repo；KB README §6.5 對接樣本含客戶代碼與價格屬機敏）。
- **A0-2** `.gitignore` 排除機敏與不可再生物：`sources/snapshots/**`（含實體交付位元組與抽取快照，改以 `source-register.json` 的 SHA-256 + bytes 登錄引用）、任何含客戶代碼/價格/個資的樣本檔、`rag-index/` 快取索引、`*.sqlite`、`data/` 衍生產物。**保留** `sources/source-register.json`、ADR、`specs/`、`reconciliation/`、`issues/`、`traceability/`、`compliance/`、`tools/`、`ops/`。
- **A0-3** 新增 `Knowledge/tools/kb-verify.ps1`：機械式驗證器 — 重算 `source-register.json` 內每筆 SHA-256（KB README §6.6「不可信任先前的記錄值」）、驗證 `specs/mtrix-field-dictionary.md` 與 `reconciliation/field-diff.md` 與 `Knowledge/tools/reconciliation/build-field-diff.ps1` 產生結果一致、不一致即 exit 1。
- **A0-4** `Dev/.github/workflows/ci.yml` 新增 `knowledge` job：淺層 checkout Knowledge remote 至 `../Knowledge`（或 `${{ runner.temp }}/Knowledge`），以**檔案內釘選的 SHA**（新增 `Knowledge/PINNED_SHA` 或 repo variable `vars.KNOWLEDGE_SHA`）checkout 特定 commit，而非 `main` 浮動版；執行 `kb-verify.ps1`；失敗即阻擋四環境全部 deploy job。**禁止**使用 `ref: main`（會造成程式碼與規格事實不同步）。
- **A0-5** `Dev/` 與 `Wellfit/` 各新增 `docs/knowledge-index.md` 指向已釘選 SHA 的 KB（`Wellfit/docs/knowledge-index.md` 已有，需更新為含 SHA 欄位）。`ekpi-platform/docs/` 增設 KB 交叉引用（目前 `ekpi-platform` grep `Knowledge` → 0 命中，屬治理缺口）。
- **A0-6** KB README 補「環境消費規約」章節：dev 自由取最新 main；test/release/main 僅取釘選 SHA；**KB 為唯讀契約事實來源，實作側不得回寫**（維持 ADR-0004 §2 單向引用）。

**驗收**：
- `git -C Projects/2026/Knowledge log --oneline` 有初始 commit；`git ls-files` 不含任何 `sources/snapshots/` 實體樣本
- `kb-verify.ps1` 對未竄改 KB → exit 0；竄改任一受控產物 → exit 1（負向測試）
- `ci.yml` 中 `Knowledge` 命中數 > 0，且 deploy job 依賴 `knowledge` job 成功
- `grep -rn "ref: main" .github/workflows/ci.yml` 於 Knowledge checkout 步驟 → 0 命中

### Phase A — 文件與決策校正（Phase 0 之後）

- **A1** 修正 `ekpi-platform/docs/project-management-optimization.md`：刪除不存在的 6-crate 架構敘述，改為實際 `platform`+`eval`；移除 Python 建議（Rust-first）；移除 Docker 建議（原生部署）。新增「本文件為歷史草案」標頭與 ADR 連結。
- **A2** 新增 ADR：統一整合架構（PostgreSQL 事件骨幹 + 原生部署 + Kafka 保留插槽 + 動態簽核工作流），置於 `Dev/docs/adr/`，編號接續現有 `ADR-0011` → **ADR-0012**。明確記載「推翻 Kafka+k3s 提議」的理由與推翻時點，並聲明根工作區技術基線維持 ADR-0001 不變。
- **A3** 新增 ADR：電商平台整合邊界（通用骨架 + momo 首個完整實作；契約事實禁止網路推測）→ **ADR-0013**。
- **A4** 更新 `Dev/docs/PROJECT_STATUS.md`、`Dev/docs/NEXT_ACTIONS.md`、`Dev/docs/PLAN_REGISTER.md`，納入本計畫。
- **A5** 在 `Projects/2026/Knowledge/issues/mtx-issue-register.md` 登錄（**注意：知識庫不在 `Dev/` 之下**）：momo/PChome/Cyberbiz API 規格待取得（掛 OQ-F2-5 對接方索取項）；PChome 渠道碼 995F 待確認；KB 版控化（Phase 0）。
- **A6** 登錄三套平行 ADR 儲存處的權威邊界（`Dev/docs/adr/`＝根工作區範圍、`Knowledge/adr/`＝跨專案介接事實、`ekpi-platform/docs/adr/`＝ekpi 範圍），並於三處各加交叉索引，避免日後誤引。

**驗收**：`grep -rn "apps/web\|crates/domain" Dev/ekpi-platform/docs/project-management-optimization.md` → 僅出現在「歷史草案已廢止」段落；ADR-0012/0013 存在且含決策理由；A5/A6 條目可於 KB 議題登錄簿查得。

### Phase B — 共享整合層骨架（新增 crate）

- **B1** 建立 `Dev/platform-hub/` 獨立 Cargo workspace，member：`channels`、`eventbus`、`workflow`、`udm`、`adapters-momo`。技術基線依 ADR-0001：**edition 2021 / resolver 2**（新 crate 沿用現行基線，升級需獨立 POC）；若需要 HTTP 端點則用 axum 0.7 + Askama 0.16。**不得**以 path dependency 引用 Wellfit/ekpi-platform（避免跨 repo 耦合）；共用型別以本 workspace 內 `udm` crate 為 SSOT。
- **B2** `udm` crate：定義統一資料模型。必含：訂單（order_id/platform_order_id/platform/customer/items/total/currency/status/timestamps）、庫存、商品、金額一律用整數分（`i64` cents）+ `Decimal4`（10^4 縮放），**禁用 f64**（比照 `Dev/core/src/money.rs` 的 `parse_decimal_to_minor` 純字串運算做法）。型別須 `#[derive(Serialize, Deserialize)]` 且附 `provenance`（來源平台 + 契約版本 + 欄位來源）。
- **B3** `channels` crate：`ChannelAdapter` trait（`platform_id()` / `capabilities()` / `fetch_orders()` / `push_inventory()` / `verify_webhook_signature()` / `health()`）。另定義 `FormatDeriver` trait 支援雙向格式衍生（UDM → 平台格式 / 平台格式 → UDM），每個轉換器必須附 `ColumnContract`（以名稱解析，必填缺漏硬失敗，**禁止序位退讓**，比照 `wellfit_core::contract` 設計原則但在此 crate 獨立實作並加交叉一致性測試）。
- **B4** `eventbus` crate：PostgreSQL 為主的 outbox/inbox 事件骨幹。表：`event_outbox`（id/aggregate_type/aggregate_id/event_type/payload_jsonb/created_at/published_at/attempt）、`event_inbox`（consumer_id/event_id 唯一鍵，冪等去重）、`workflow_state`、`approval_task`。提供 `EventStore` trait 抽象，並留 `KafkaEventStore` 空槽（`unimplemented!()` + 明確文件說明啟用條件），**不引入 Kafka 依賴**（Kafka 啟用門檻見 §7）。索引依 aggregate 與時間建立。
- **B5** `workflow` crate：動態簽核工作流引擎。資料模型：`WorkflowDefinition`（可版本化、stage 數量不限）、`Stage`（approver_rule / sla / urgency / conditions / escalation_path / on_timeout）、`ApprovalTask`（state machine：Pending → Approved/Rejected/Escalated/Expired/Cancelled）、`ConditionExpr`（金額門檻、平台、資料分類、風險分數、異常旗標的布林組合，**確定性求值，無模型推論**）。含：多急簽核（Urgency: Low/Normal/High/Critical，各自有 SLA 與升級路徑）、條件分支、超時升級（可多跳且防環）、定期提醒、閒置清理（retention policy）。
- **B6** `workflow` crate：Agent 判斷層 `agent_policy.rs`。**明確邊界**：Agent 僅能「建議」或在預授權低風險範圍內「自動核准」，不可核准高風險/付費/敏感分類。規則：決定性風險分數 + 分類矩陣（`Classification` 概念沿用 `wellfit_core::access_control`），高風險強制人工；所有 Agent 決策（含自動核准）寫入 append-only 稽核並附信心分數與理由。失敗時 fail-closed（退回人工）。
- **B7** 權限控管：付費動作與敏感分類的核准需具體角色權限，沿用 RBAC + SoD 概念（分離職責），並支援金額分級核准權限（例如 >NT$X 需財務主管）。Agent 不能繞過。
- **B8** `adapters-momo` crate：**條件式實作** — 僅在取得 momo 官方 API 規格後才填入端點/欄位/簽章；在此之前只建立骨架、`ChannelAdapter` 實作骨架、與以 `pending_contract` 狀態標記的 stub，所有未取得事實標 `Unverified`（KB `research-protocol.md` §1.1 第 4 點：原文不可讀時標 `Unverified`、保留原值不做決策、登錄 `issues/`），**不得**填入猜測值。
- **B9** Redis 適配：僅實作快取（查詢結果）與分散式限流計數器；設定為可選，未安裝 Redis 時自動降級為 in-process（不影響核心流程）。
- **B10** CLI：新增 `platform-hub` 單一 binary，統一入口子命令：`channels`（列出/啟停適配器）、`momo`（同步訂單/庫存）、`workflow`（建立/發布流程定義、查詢任務、核准、駁回、升級）、`events`（重播指定事件、檢視 outbox/inbox 積壓）、`health`（DB/Redis/適配器狀態）。

**驗收**：
- `cargo clippy --workspace --all-targets -- -D warnings` 零警告；`cargo test --workspace` 全通過
- 整合測試覆蓋：UDM 雙向轉換 round-trip、必填欄缺漏硬失敗、金額全程整數分無 f64（以 clippy + 型別約束驗證）、事件冪等去重（同 event_id 重投僅處理一次）、多級簽核全流程、超時升級、Agent 自動核准僅限低風險、fail-closed 行為
- `grep -rn "f64" Dev/platform-hub/crates/udm/src/` → 僅允許出現在統計/評分欄位（且需註解說明不參與金額運算）
- `grep -rn "kafka" Dev/platform-hub/Cargo.toml` → 0 命中（Kafka 僅為 trait 空槽）

### Phase C — EKPI 治理面接軌

- **C1** `ekpi-platform/platform/src/` 新增 `workflow.rs`：以 `platform-hub` 的 `workflow` crate 為模型來源（**透過 API/事件而非 crate 依賴**），提供 EKPI 側的簽核任務檢視與操作端點。
- **C2** `api.rs` 擴充路由（既有僅 `/health`、`/api/v1/cases/:id/decide`、`/api/v1/cases/:id/snapshot`）：新增 `/api/v1/workflows`、`/api/v1/approvals`、`/api/v1/approvals/:id/decide`、`/api/v1/events/replay`（僅本機、需角色權限）。沿用既有 `local_auth.rs` session 與分權（Viewer/Editor/Approver）。
- **C3** `governance_web` 工作台新增簽核任務頁（CSS-first、零 JS、表單 POST + PRG，讀取用 GET）。含多急分級顯示、逾時升級提示、稽核鏈連結。
- **C4** 稽核對接：`audit.rs` 的 SHA-256 追加式鏈納入平台側簽核事件（事件以 `source = "platform-hub"` 進入同一鏈格式），確保跨系統事件可驗證完整性。
- **C5** 對帳：`wellfit_integration`（既有對帳/切換模組）擴充為三方對帳（EKPI 決策 ↔ 事件骨幹 ↔ Wellfit 執行結果），差異產出隔離清單而非靜默修正。

**驗收**：EKPI `cargo test --locked` 全通過；既有 270 release-profile 測試數不得下降（只增不減）；`governance_web` 零 JS 合規測試通過；跨系統事件完整性鏈可驗證。

### Phase D — 根工作區 stub 收斂

- **D1** `web/src-tauri/src-tauri/commands.rs:156` `approve_transaction` 移除 `TODO`，改為呼叫 `platform-hub` 簽核 API；`platform/src/ai/decision_gating.rs` 的 `AIDecision` 映射到統一簽核任務（保留 `requires_human_approval` 與 `Action::EscalateToHuman` 語意），移除 `f32 confidence` → 金額語境（僅保留非金額欄位）。
- **D2** 根工作區 `platform` 對 f64 金額全面收斂為整數分/定點十進位（`core`/`platform`/`web` 三 crate 範圍內），以 `scaled.rs` 等價實作落地。
- **D3** 補齊 `/api/health`：DB 查詢 + migration 版本 + 事件積壓 + 適配器健康（既有 `docs/NEXT_ACTIONS.md` 高價值項 #2）。

**驗收**：根工作區 `cargo clippy --workspace --all-targets -- -D warnings` 零警告；`grep -rn "TODO: Implement actual transaction approval logic"` → 0 命中；健康檢查端點回傳 DB 版本與積壓數。

### Phase E — momo 首個完整實作（解除外部阻塞後）

- **E1** 前置條件：取得 momo 官方 API 規格（認證方式、端點、欄位、簽章/驗證、錯誤碼、速率限制），登錄至 `Dev/Knowledge/sources/source-register.json` 含 SHA-256 與頁碼。未取得前 **E2–E6 不得執行**（禁止猜測）。
- **E2** `adapters-momo` 實作真實端點：認證（token 取得/刷新）、分頁抓取、錯誤重試（指數退避 + jitter）、速率限制遵守。
- **E3** momo 欄位 → UDM 轉換器，以 `ColumnContract` 名稱解析；反向 UDM → momo 格式（推播庫存）同樣具契約。
- **E4** Webhook 接收端點（含簽章驗證）與重放保護（nonce/timestamp window）。
- **E5** 冪等：momo 訂單/事件以平台 ID + 更新時間戳作冪等鍵，寫入 `event_inbox` 去重。
- **E6** 端到端測試：以合成的 momo 格式 fixture（附 PROVENANCE 與 SHA-256）驗證 fetch → UDM → ERP 匯入 CSV 全鏈路；**fixture 必須標示為合成資料**（`Wellfit/AGENTS.md` §4 不變量 5）。
- **E7** PChome / Cyberbiz：建立同一骨架下的 adapter stub（`pending_contract` 狀態），待各自規格取得後依 E2–E6 流程填入；每個平台一個待辦議題。

**驗收**：momo fixture 端到端可跑且輸出金額全為整數分；簽章驗證測試涵蓋有效/無效/過期/重放四情境；缺欄位檔案硬失敗。

### Phase F — 排程、監控與效能最佳化

- **F1** 排程：依 `automation_integration_architecture` 決策，採 **Windmill（原生二進位）+ PowerShell + `wellfit.exe`** 為自動化觸發層；`platform-hub` 暴露 idempotent 的排程型 CLI 供 Windmill 呼叫（`workflow tick`、`events drain`、`channels sync --platform=momo`）。不引入容器。
- **F2** 監控：健康檢查 + 指標（事件積壓、簽核逾時數、Agent 自動核准率、拒答率、API 錯誤率、DB 連線數）；指數退避重試、斷路器、死信佇列。
- **F3** 調校：依 `docs/metrics/targets.md` 與 `docs/risk-model.md` 既有門檻設定 SLO；先量測基線再優化（禁止無量測的「最佳化」）。
- **F4** 復原演練：outbox 重播、inbox 去重驗證、資料庫備份/還原、簽核狀態機復原，記錄 RTO/RPO 實測值。
- **F5** 安全：TLS 憑證釘選（僅接受指定 CA）、秘密管理（.env 不進版控）、審計日誌防竄改驗證、供應鏈 `cargo audit`、最小權限檢查。

**驗收**：Windmill 排程可在無人工介入下完成一次完整 momo 同步；重播同一事件不產生重複副作用；備份還原演練 PASS 且 RTO 有實測數據。

## 5. 驗證總表

| 閘門 | 指令 | 通過標準 |
|------|------|----------|
| 格式 | `cargo fmt --all -- --check` | 0 差異 |
| 零警告 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 警告（不得加 `allow`） |
| 測試 | `cargo test --workspace` | 0 failed / 0 ignored |
| 不變量 | 掃描腳本 | 每個 `.rs` 首行 `#![forbid(unsafe_code)]`；金額路徑無 f64；`<script>` 計數 = 0 |
| 契約 | `ColumnContract` 測試 | 必填欄缺漏必硬失敗；無序位退讓 |
| 冪等 | 重播測試 | 同 event_id 重投僅生效一次 |
| 完整性 | SHA-256 鏈驗證 | 竄改任一事件即驗證失敗 |
| 溯源 | claims-vs-evidence | 每項數量對應到測試 ID / 指令 / 來源快照 |
| 安全 | `cargo audit` | 0 未接受風險 |

> 執行注意：`Wellfit` 建置須用 `Knowledge/tools/dev-env/build.ps1`（工作站 Application Control 阻擋桌面路徑產物），且 `-TargetDir` 須各不相同以免 lock 競爭。

## 6. 風險登記

| ID | 風險 | 等級 | 緩解 | 擁有者 |
|----|------|------|------|--------|
| R-01 | momo API 規格未取得 → E 階段全數阻塞 | 高 | Phase E 明確標為條件式；先完成 B/C/D；對接方索取項掛 OQ-F2-5 | 整合負責人 |
| R-02 | 規格事實被推測補值（違反研究規約 §1.1） | 高 | `VerifyStatus::Unverified` 強制標記；來源登錄 SHA-256 缺一不得實作；裁決與規格分離（沿用 ADR-0009 精神） | 知識庫維護者 |
| R-03 | 跨 repo 耦合破壞獨立演進 | 中 | 禁止 path dependency；僅以 API + PostgreSQL 事件契約互通；契約變更走版本欄位 | 架構負責人 |
| R-04 | 跨系統交易性不一致 | 高 | outbox/inbox + 冪等鍵 + 三方對帳；不確定狀態進隔離清單 | 整合負責人 |
| R-05 | Agent 誤判導致未簽核執行 | 高 | 決定性風險分數優先；高風險/付費/敏感強制人工；fail-closed；完整稽核 | 安全負責人 |
| R-06 | Agent 建議品質不足致人工負荷 | 中 | 先規則後模型；信心門檻可調；自動核准率與否決率納入監控；低信心自動轉人工 | 治理負責人 |
| R-07 | 原始碼 `f64` 金額殘留（根工作區現況） | 高 | Phase D2 收斂；型別約束 + clippy + grep 雙重把關 | 根工作區負責人 |
| R-08 | 文件漂移持續擴大 | 中 | Phase A 先校正；文件更新與代碼同提交（SHARED_STANDARDS §4.3） | 知識庫維護者 |
| R-09 | 事件骨幹隨流量成長受限 | 中 | PostgreSQL 分區表 + outbox 索引；預留 Kafka 適配器插槽；壓測後再決 | 架構負責人 |
| R-10 | 簽核流程過長致業務停滯 | 中 | 多急 SLA + 逾時升級 + 低風險自動核准 + 逾時監控告警 | 業務負責人 |

## 7. 未決事項（明確標記為待決，不阻擋 A–D 執行）

1. **momo 規格取得時程** — 由對接方回覆時程決定 E 階段起點；在此之前 E 階段全部掛起。
2. **Kafka 啟用條件** — 建議門檻：單表事件寫入持續 >5,000/s 或 outbox 積壓 >100k 且 Postgres 分區仍無法滿足 P99 <100ms。屆時評估 `KafkaEventStore`。
3. **Redis 必要性** — 僅在多實例部署或分散式限流需求出現時啟用；單機部署以 in-process 降級。
4. **PChome/Cyberbiz 規格** — 各自獨立索取，骨架完成後依 E2–E6 流程填入。
5. **EKPI 是否併入 Wellfit workspace** — 維持獨立（尊重既有 `no root Cargo workspace` 決策）；若未來合併需另立 ADR。

## 8. 參考文件

- `Wellfit/AGENTS.md` §3–§4（建置入口、不變量）、`Wellfit/crates/wellfit_core/src/contract.rs`（名稱解析契約）、`.../access_control.rs`（RBAC）、`.../jit_permission.rs`（JIT 簽核）、`.../scaled.rs`（定點十進位）、`.../ledger.rs`（冪等帳本）、`.../release_gate.rs`（Release Gate）
- `Wellfit/crates/wellfit_assist/src/data_transformer.rs`（多格式/編碼轉換）、`.../anomaly.rs`（異常偵測）、`.../shadow.rs`（影子模式）
- `Wellfit/crates/wellfit_integration/src/`（對帳、切換）
- `ekpi-platform/platform/src/governance.rs`（Ledger/GateDecision/Workflow）、`.../api.rs`（路由）、`.../audit.rs`、`.../local_auth.rs`、`.../governance_web.rs`
- `Dev/platform/src/ai/decision_gating.rs`（AI 簽核閘）、`Dev/web/src-tauri/src-tauri/commands.rs:156`（待收斂 stub）
- `Dev/docs/research/proposals/OQ-F1_F2_輔翼ERP正式需求提案_v1.0.md`（對接方索取項）
- `Dev/docs/NEXT_ACTIONS.md`、`Dev/docs/PROJECT_STATUS.md`、`Dev/docs/PLAN_REGISTER.md`
