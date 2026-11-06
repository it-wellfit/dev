# 三大工作室與治理平台架構 — 專業規劃

> 計畫編號：1792461065184；版本：v2.1（2026-11-06 內容精緻化與跨文件對齊）；狀態：已確認落地中
> 唯一權威：本檔（`1792865422970-three-studio-governance-platform.md`）
> 前檔：`1793782615513-wellfit-unified-plan-v2.md`（上游統一計畫 v2）

## 1. 定案決策

| 決策 | 裁定 | 依據 |
|------|------|------|
| 三大工作室架構 | **採納**：工作室 1 內部使用者大平台、工作室 2 技術工作室、工作室 3 治理工作室 | Clean Architecture 對應 Presentation → Application → Domain 分層 |
| 平台定位 | **專案、稽核與知識管理**：涵蓋業務（專案與流程）、知識（規格與證據）、法規（合規條文）、公司治理（決策與稽核） | 本 session 裁定 |
| 電商與規格跨層整合 | **事件總線耦合，不深度 schema 整合**：電商事件經 `event_outbox` 進入治理中樞，規格經 KB 參照，關聯發生於簽核任務層 | 耦合度、更新頻率與責任歸屬分析（見 §4） |
| 工作台 | **零 JS、CSS-first、表單 POST + PRG**：工作台為主介面，API 為程式化薄包裝，共用 use case | 憲章 §2.3 |
| 基礎設施 | **PostgreSQL 18 + 原生 binary 部署**；Redis 按需（快取/限流）；Kafka 保留插槽預留不實作 | 推翻 Kafka+k3s 提議 |
| 技術基線 | ekpi-platform：Rust 2024 / Resolver 3 / Axum 0.8.9 / Askama 0.16.1 / SQLx 0.9.0 / PostgreSQL 18.6；根工作區：Edition 2021 / SQLite | ADR-0006 / ADR-0001（維持不變） |
| AI 邊界 | ekpi-platform 內 Agent 僅 advisory（建議），不自動放行；DEC-15 未決前不得啟用自動核准 | phase-gates.md §Out of scope |

## 2. 三大工作室架構

### 2.1 工作室定義與版本標記

> **釐清**：`workflows`/`approvals`/`settings`/`api-docs` 等路由自 **Phase C9** 起即實作於 ekpi-platform `platform/src/governance_web.rs`（零 JS、CSS-first）；`approval_web.rs` 為其內部分類標記（API v1 端點），**非另一套前端**。`apps/web`（`ekp_web`）為獨立骨架，與工作台無關。

| 工作室 | 對象 | 核心職責 | 主要介面 |
|--------|------|---------|---------|
| **工作室 1：內部使用者大平台** | 所有部門人員（業務使用者、部門主管） | 日常業務操作：專案建立、證據上傳、簽核任務處理、查詢儀表板 | `governance_web`（零 JS 工作台，13 頁） |
| **工作室 2：技術工作室** | 懂 AI 與資訊技術人員 | 系統整合：適配器開發、API 維護、資料庫運維、異常監控、技術問題排除 | `platform-hub`（5 crates）、`wellfit.exe`、Windmill 排程 |
| **工作室 3：治理工作室** | 懂產業與法規與公司治理的非技術人員 | 政策定義、合規審查、稽核鏈驗證、規格審查、變更治理 | `governance_web` 稽核頁、API 文件頁、KB 審查介面 |

### 2.2 工作室職責邊界

```
┌─────────────────────────────────────────────────────────────────┐
│  工作室 1：內部使用者大平台                                        │
│  ─────────────────────────────────────────────────────────────── │
│  所有部門人員（業務使用者、部門主管）                              │
│  日常業務操作：專案建立、證據上傳、簽核任務處理、查詢儀表板       │
│  介面：governance_web（13 頁）                                    │
│  權限：Viewer/Editor/Approver（依 RBAC）                          │
└─────────────────────────────────────────────────────────────────┘
                    │ 操作產生事件寫入 audit_chain
                    │ 簽核任務進入 event_outbox
                    ▼
┌─────────────────────────────────────────────────────────────────┐
│  工作室 2：技術工作室                                            │
│  ─────────────────────────────────────────────────────────────── │
│  懂 AI 與資訊技術人員                                            │
│  系統整合：適配器開發（momo/PChome/Cyberbiz）、API 維護、         │
│  資料庫運維、異常監控、技術問題排除                               │
│  介面：platform-hub（5 crates）、wellfit.exe、Windmill 排程       │
│  權限：Admin（技術帳號，需雙人核准重大變更）                      │
└─────────────────────────────────────────────────────────────────┘
                    │ 提供標準介面與自動化腳本
                    │ 重大變更需治理工作室核准（JIT 簽核）
                    ▼
┌─────────────────────────────────────────────────────────────────┐
│  工作室 3：治理工作室                                            │
│  ─────────────────────────────────────────────────────────────── │
│  懂產業與法規與公司治理的非技術人員                              │
│  政策定義、合規審查、稽核鏈驗證、規格審查、變更治理              │
│  介面：governance_web 稽核頁、API 文件頁、KB 審查介面             │
│  權限：Auditor（僅審查，不執行）；規格登錄 Admin（KB 單向引用）   │
└─────────────────────────────────────────────────────────────────┘
                    │ 審查結果寫入 audit_chain
                    │ 規格條文以 SHA-256 登錄 KB
                    ▼
           Knowledge/（規格資料，SSOT）
           審查員對照電商事件與規格條文做出合規裁定
```

### 2.3 工作室間協作

- **工作室 1 → 工作室 3**：業務操作的每一筆變更寫入同一 `audit_chain`，治理工作室可完整追溯
- **工作室 2 → 工作室 1**：提供標準介面與自動化腳本，業務使用者自助操作，僅重大變更需技術審核
- **工作室 3 → 工作室 2**：政策與合規要求以規格形式提出，技術工作室負責實作；規格變更需治理核准
- **工作室 2 → 工作室 3**：技術變更需治理核准（JIT 簽核）；失敗閉鎖退回人工

## 3. 治理平台分層

```
┌─────────────────────────────────────────────────────────────────┐
│  治理面（EKPI）                                                  │
│  ──────────────────────────────────────────────────────────────  │
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
│  ──────────────────────────────────────────────────────────────  │
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
│  ──────────────────────────────────────────────────────────────  │
│  wellfit_core（帳本、ReleaseGate、契約）                        │
│  wellfit_shopee/reports/mtrix/bankpay（匯入/匯出）             │
│  wellfit_integration（三方對帳：EKPI ↔ 事件骨幹 ↔ 執行）       │
│  根工作區（FlapsErp RPA、AI decision_gating）                  │
└─────────────────────────────────────────────────────────────────┘
```

## 4. 跨層整合裁定：事件總線耦合

### 4.1 為何不用深度 schema 整合

| 維度 | 深度 schema 整合（不建議） | 事件總線耦合（建議） |
|------|------------------------|-------------------|
| 耦合度 | 電商與規格綁定同一表/同一事務 | 電商經 `event_outbox` 發布事件，規格經 KB 參照，兩者無直接 DB 依賴 |
| 更新頻率 | 電商高頻變動、規格低頻變動，強合導致鎖爭 | 各自維持本節奏，事件鏈追溯關聯 |
| 責任歸屬 | 電商團隊與合規團隊共享 schema，責任模糊 | 電商團隊負責 `adapters-momo`，合規團隊負責 KB `specs/`，治理中樞僅做關聯 |
| 變更風險 | 規格修改可能誤改電商欄位 | 規格變更不影響既有電商事件 |

### 4.2 事件耦合架構

```
電商平台（momo/PChome/Cyberbiz）
      │ ChannelAdapter（platform-hub）
      ▼
UDM 轉換 → event_outbox（PostgreSQL，ekpi-platform 所有）
      │
      ▼  接收方（ekpi-platform 治理中樞）
   event_inbox（冪等去重）→ ApprovalTask（簽核任務）→ audit_chain（SHA-256 稽核）
      │
      ▼
   Knowledge/（規格資料，SHA-256 登錄，單向引用）
   審查員於治理工作台對照「電商事件」與「規格條文」，做出合規裁定
```

**關鍵原則：**
- 電商資料的「事實」（訂單號、金額、庫存）寫入 `event_outbox`，不可由規格資料覆寫
- 規格資料的「規則」（驗證條文、對照表）寫入 `Knowledge/specs/`，SHA-256 登錄，不可由電商事件修改
- 兩者關聯發生於「簽核任務」層：ApprovalTask 可同時參照 `event_outbox.id` 與 `Knowledge/specs/<id>`，由核准者裁定

### 4.3 三方對帳閉環

```
EKPI 決策（approval API/audit event）
    ╲
     ╲ 三方比對（wellfit_integration::three_way）
      ╲
事件骨幹（event_outbox/inbox)
        ╲
         ╲
Wellfit 執行（idempotency ledger)
    ↘
     一致 → 通過
     不一致/缺漏 → IsolationEntry（隔離清單，絕不靜默修正）
```

## 5. RBAC 權限矩陣

| 權限 | 工作室 1（業務） | 工作室 2（技術） | 工作室 3（治理） |
|------|----------------|----------------|----------------|
| 專案/任務 CRUD | Editor（自有部門） | Viewer | Viewer |
| 簽核/駁回/升級 | Approver（金額分級） | 無（除非兼任業務主管） | Auditor（僅審查，不執行） |
| 適配器/頻道管理 | 無 | Admin | Viewer |
| 規格/合規條文登錄 | 無 | 無 | Admin（KB，單向引用） |
| 稽核鏈查看 | Viewer | Viewer | Admin |
| 系統設定/儲存模式 | 無 | Admin | 無 |
| 使用者/角色管理 | 無 | Admin（技術帳號） | 無（需 Studio 2 協作） |
| 事件重播/除錯 | 無 | Admin | Viewer |

### 5.1 API 認證說明

| 場景 | 認證方式 |
|------|----------|
| 工作台操作 | session cookie + CSRF token |
| 程式化呼叫 | Bearer API key（設定頁產生） |

API 未設定 key 時回 `401 Unauthorized`。工作台不經 API，不受影響。

```bash
curl -H "Authorization: Bearer <API_KEY>" \
  http://127.0.0.1:3000/api/v1/channels/health
```

### 5.2 關鍵控制

- **SoD**：同一人不得同時擔任「簽核人」與「執行人」
- **JIT 權限**：敏感操作（跨機部署、資料庫備份還原）需治理工作室核准才可由技術工作室執行
- **失敗閉鎖**：自動化腳本遇到異常時，預設退回人工簽核，不可靜默繼續

## 6. 工作台頁面規劃（MVP 13 頁）

### 6.1 已實作（13 頁）

| 路徑 | 頁面 | 狀態 |
|------|------|------|
| `/` | 工作總覽 | 已實作 |
| `/projects` | 專案管理 | 已實作 |
| `/evidence` | 文件與證據 | 已實作 |
| `/workflows` | 流程範本 | 已實作 |
| `/controls` | 風險與品質 | 已實作 |
| `/lifecycle` | 階段與關卡 | 已實作 |
| `/audit` | 稽核紀錄 | 已實作 |
| `/approvals` | **核准任務佇列** | Phase C9 已實作 |
| `/settings` | **系統設定** | Phase C9 已實作 |
| `/api-docs` | **API 文件** | Phase C9 已實作 |
| `/inspections` | 品質檢查 | 已實作 |
| `/login` | 登入 | 已實作 |
| `/assets/governance.css` | 樣式 | 已實作 |

### 6.2 API 端點（11 端點，已實作）

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/api/v1/workflows` | 列出簽核工作流定義 |
| POST | `/api/v1/workflows` | 建立簽核工作流定義 |
| GET | `/api/v1/approvals` | 列出簽核任務 |
| POST | `/api/v1/approvals/{id}/decide` | 核准/駁回簽核任務 |
| POST | `/api/v1/events/replay` | 重播指定事件 |
| GET | `/api/v1/channels` | 列出通道適配器 |
| GET | `/api/v1/users` | 列出使用者 |
| GET | `/api/v1/settings` | 系統設定 |
| GET | `/api/v1/search` | 全域搜尋 |

## 7. 落地路徑（可立即執行項目）

### 7.1 已完成（Phase 0–D + C8 + D3）

| 階段 | 內容 | 驗證 |
|------|------|------|
| Phase 0 | KB 獨立 git + .gitignore + kb-verify.ps1 | git commit、kb-verify PASS |
| Phase A | ADR-0002、ADR-0003 | 檔案存在 |
| Phase B | platform-hub：udm、channels、eventbus、adapters-momo、agent-policy | build/clippy/test 全通過（9 測試） |
| Phase C1–C5 | workflow.rs、channels.rs、approval.rs、ports、postgres 適配器、migration | build/clippy/test/selftest 全通過 |
| Phase C6 | API v1 路由（11 端點） | build/clippy/test 全通過 |
| Phase C9–C11 | 工作台頁面（/approvals、/settings、/api-docs） | build/clippy/test 全通過 |
| Phase C8 | wellfit_integration::three_way 三方對帳（9 測試） | 45 測試通過（含 9 新測試） |
| Phase D1 | approve_transaction stub 移除 TODO | build/clippy/test 全通過 |
| Phase D2 | f64 殘留收斂 | build/clippy/test 全通過 |
| Phase D3 | /api/health 擴充（outbox、hub_reachable） | 3 health 測試通過 |

### 7.2 阻塞中（外部條件）

| 階段 | 阻塞原因 | 解除條件 |
|------|---------|---------|
| Phase E2–E6 | momo API 規格未取得（OQ-F2-5） | 對接方回覆規格 |
| Phase F | ekpi-platform 部署目標未核准（PR-02） | 擁有者核准部署目標 |

## 8. 風險與緩解

| 風險 | 等級 | 緩解 |
|------|------|------|
| Studio 2 成為技術瓶頸 | 高 | 自助 + 例外審核模式，僅重大變更需 Studio 2 核准 |
| Studio 3 誤判合規條文 | 中 | 決策輔助（advisory-only），最終裁定由人工確認 |
| 電商與規格邏輯衝突 | 高 | 僅於 ApprovalTask 層關聯；衝突進入隔離清單 |
| 權限集中於 Studio 2 Admin | 高 | Admin 雙人核准，所有操作寫入 audit_chain |
| Agent 誤判（R-05） | 高 | 高風險強制人工，fail-closed，完整稽核 |
| momo 規格未取得（R-01） | 高 | E 明確標為條件式；B/C/D 先行；掛 OQ-F2-5 |
| 部署目標未核准（R-15） | 高 | C 階段僅 loopback；跨機部署另立 Phase G |

## 9. 驗證標準

| 閘門 | 指令 | 通過標準 |
|------|------|----------|
| 格式 | `cargo fmt --all -- --check` | 0 差異 |
| 零警告 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 警告 |
| 測試 | `cargo test --locked` | 0 failed / 0 ignored |
| ekpi-platform 入口 | `scripts/check.ps1` | exit 0 |
| 自測 | `cargo run --locked -- --selftest` | 49 assertions PASS |
| 不變量 | 掃描 | `#![forbid(unsafe_code)]`、無 f64 金額路徑、`<script>` 計數=0 |
| 契約 | `ColumnContract` 測試 | 必填缺漏硬失敗 |
| 冪等 | 重播測試 | 同 event_id 重投僅生效一次 |
| Agent 邊界 | `grep auto_approve ekpi-platform/crates/` | 0 命中 |

## 10. 參考文件

- `1793782615513-wellfit-unified-plan-v2.md`（上游統一計畫 v2）
- `ekpi-platform/AGENTS.md`（ekpi-platform 入口規則）
- `Wellfit/AGENTS.md`（Wellfit 專案規則）
- `Knowledge/adr/ADR-0004-knowledge-base-location.md`（KB 位置裁定）
- `Knowledge/sources/research-protocol.md`（研究規約 §1.1）
- `ekpi-platform/docs/adr/0002-unified-integration-architecture.md`（ADR-0002）
- `ekpi-platform/docs/adr/0003-ecommerce-integration-boundary.md`（ADR-0003）
