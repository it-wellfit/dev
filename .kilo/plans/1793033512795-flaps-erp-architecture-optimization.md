# Flaps ERP 系統架構全面優化計畫 — Rust + AI 混合架構 (第一性原理設計)

> 文件編號：FLAPS-ARCH-OPT-003
> 版本：v3.0
> 日期：2026-10-27
> 對象：主管 / 專案利害關係人 / 架構設計團隊
> 狀態：Draft for Review
> 前置文件：FLAPS-PERF-PRJ-001 (v1.0), FLAPS-ARCH-OPT-001 (v1.0), FLAPS-ARCH-OPT-002 (v2.0)
> 工作目錄：C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev

> **Disposition (2026-11-02): Unapproved architecture proposal; not implementation authority.** The claimed FlapsErp baseline and performance figures require reproducible source/measurement evidence. Preserve the explicit local-only data constraint (C-08); the separate cloud-fallback proposal is blocked unless the data owner approves a documented exception, data classification, provider, and retention controls. Do not select or deploy an AI model from this proposal alone.

---

## 0. 第一性原理與系統性思維框架

### 0.1 方法論

本文件採用第一性原理 (First Principles) 與系統性思維 (Systems Thinking) 進行架構優化設計。所有設計決策均從「本質問題」出發，不套用既有模式或慣例，而是從物理/邏輯本質推導最優解。本版本 (v3.0) 在 v2.0 基礎上，將技術棧升級為 **Rust 核心 + 免費 AI 模型 (JEV)** 的混合架構，並引入 4 層系統架構、62 項 ERP 設定盤點、AI 模型推薦與 JEV 整合策略。

### 0.2 事實 (Facts) — 已確認的客觀事實

| 編號 | 事實 | 來源 |
|------|------|------|
| F-01 | FlapsErp v0.0.4 為 Electron 桌面應用，使用 winax (ActiveX/COM) 與後端 ERP 通訊 | 逆向工程 app.asar |
| F-02 | 所有 COM 呼叫透過 sendSync 同步 IPC 執行，阻塞 UI 執行緒 | mainApi.js 原始碼 |
| F-03 | COM 物件無執行緒隔離，直接在主程序主執行緒運行 | mainApi.js 原始碼 |
| F-04 | 無快取機制，每次表單載入都重新透過 COM 取得資料 | loadSaveForm.js 原始碼 |
| F-05 | 無 COM 物件池，物件生命週期管理粗放 | mainApi.js 原始碼 |
| F-06 | 無批次 COM 呼叫機制 | mainApi.js 原始碼 |
| F-07 | 同步檔案 I/O 分散於更新與設定流程 | mainApi.js 原始碼 |
| F-08 | 硬體加速可透過組態檔關閉 | main.js 原始碼 |
| F-09 | 應用程式包含 23 個頂層資源項目 | app.asar 解構 |
| F-10 | Wellfit 專案使用 Rust + Axum + SQLite 架構，已有 COM 介接經驗 | Wellfit/AGENTS.md |
| F-11 | 專案團隊具備 Rust、Electron、COM/ActiveX 開發能力 | 專案文件 |
| F-12 | Rust 生態系提供 windows-rs crate，可直接呼叫 Windows COM API | crates.io |
| F-13 | 免費 AI 模型 (JEV/DeepSeek) 提供 OpenAI 相容 API，可本機部署 | 公開文件 |
| F-14 | Tantivy 全文搜尋引擎為 Rust 原生，效能優於 SQLite FTS5 | crates.io |
| F-15 | Tauri 2.0 支援 Rust 後端 + Web 前端，體積 < 10MB | Tauri 官方文件 |

### 0.3 假設 (Assumptions) — 待驗證的推論

| 編號 | 假設 | 驗證方式 | 風險等級 |
|------|------|---------|---------|
| A-01 | winax 可於 Electron Utility Process 中穩定執行 | POC 驗證 | 高 |
| A-02 | ERP COM 介面在專案期間不發生重大版本變動 | ERP 廠商確認 | 中 |
| A-03 | 開發系統可獨立於 Flaps ERP 部署 | 架構評估 | 低 |
| A-04 | 使用者可接受漸進式遷移（舊系統並行） | 使用者調查 | 中 |
| A-05 | Rust sidecar 可替代 Electron 主程序的 COM 橋接職責 | 技術評估 | 中 |
| A-06 | JEV 模型可在本機 GPU 上穩定運行 | POC 驗證 | 中 |
| A-07 | Tauri 2.0 的 webview 在目標環境相容 | 相容性測試 | 低 |

### 0.4 限制 (Constraints) — 不可違反的邊界條件

| 編號 | 限制 | 影響 |
|------|------|------|
| C-01 | 目標運行環境為 Windows 10/11 64-bit | 僅支援 Windows 平台 |
| C-02 | 既有 ERP COM 介面不可修改（第三方系統） | 必須保留 COM 相容層 |
| C-03 | 專案團隊規模 4 人（1 PM + 2 Dev + 1 QA + 1 兼職專家） | 資源有限，需分階段實施 |
| C-04 | 預算有限，優先使用開源工具 | 技術選型受限 |
| C-05 | 必須保持業務連續性，不可中斷現有 ERP 操作 | 需漸進式部署 |
| C-06 | OS Application Control policy 要求建構產物置於 %LOCALAPPDATA%\Temp | 建構環境限制 |
| C-07 | AI 模型必須免費/開源，不可使用付費 API | 成本控制 |
| C-08 | 所有資料必須本機優先，不可外傳至雲端 | 安全/合規 |

### 0.5 成功指標 (Success Metrics)

| 編號 | 指標 | 現況 (估) | 目標 | 測量方式 |
|------|------|----------|------|---------|
| SM-01 | 帳務登帳延遲 | 100-500ms | < 10ms | 主程序日誌 |
| SM-02 | 帳務查詢延遲 | 100-500ms | < 50ms | 主程序日誌 |
| SM-03 | 系統啟動時間 | 8-12s | < 2s | 冷啟動計時 |
| SM-04 | 記憶體 (8h) | +200MB | < +20MB | 程序記憶體監控 |
| SM-05 | 並發使用者 | 1 | 50+ | 壓力測試 |
| SM-06 | 設定同步延遲 | N/A | < 1s | 同步引擎日誌 |
| SM-07 | UI 凍結頻率 | 高 | 0 | 自動化監控 |
| SM-08 | COM 物件數量 | 無限制 | ≤ 50 | 物件池監控 |
| SM-09 | AI 決策延遲 | N/A | < 500ms | AI 引擎日誌 |
| SM-10 | AI 決策準確率 | N/A | > 95% | 測試集驗證 |

### 0.6 非功能需求 (Non-Functional Requirements)

| 編號 | 類別 | 需求描述 | 量化目標 |
|------|------|---------|---------|
| NFR-01 | 效能 | 系統整體效能必須達到 SM-01~SM-10 | 見成功指標表 |
| NFR-02 | 穩定性 | 8 小時連續運行無崩潰、無凍結 | 0 崩潰、0 凍結 |
| NFR-03 | 相容性 | 支援 Windows 10/11，既有 ERP COM 介面版本 | 與既有 ERP 版本相容 |
| NFR-04 | 可維護性 | Rust Lint 零警告，模組職責單一 | Clippy 零警告 |
| NFR-05 | 安全性 | COM 物件不洩漏、無記憶體洩漏、無未釋放資源 | 8 小時 soak 測試記憶體增長 < +20MB |
| NFR-06 | 可追蹤性 | 每一需求、任務、風險與驗收標準皆須有唯一識別碼與負責人 | 100% 需求對應設計與測試 |
| NFR-07 | 可擴展性 | 支援水平擴展，新增門市/POS 不需修改核心 | 新增門市 < 1 天 |
| NFR-08 | 可觀測性 | 所有關鍵指標可被監控與告警 | 100% KPI 有監控 |
| NFR-09 | 成本 | 全部使用免費/開源工具 | $0/月 |
| NFR-10 | 隱私 | 所有 AI 推理本機執行，資料不外傳 | 100% 本機 |

---

## 15. EKPI Platform Product Excellence Plan (A-J Framework)

> This section contains the EKPI Platform product excellence plan, appended per user request.
> Plan file: `.kilo/plans/1792914505665-ekpi-platform-product-excellence.md`

### 15.1 Phase A: 0 → 1 MVP Completion (7 items)

| Item | Status | Evidence |
|------|--------|----------|
| A-01 Select pilot workflow, owner, data | `Not met` | No named product owner |
| A-02 Complete acceptance-test matrix | `Partial` | 67-case corpus passes; AT-01–AT-31 not mapped |
| A-03 Real-workflow dataset + independent review | `Partial` | Synthetic only; no independent reviewer |
| A-04 Open decisions registered | `Partial` | Decision register exists; DEC-01–DEC-20 not all assigned |
| A-05 Complete backup log + off-host proposal | `Partial` | Local restore drills PASS; no off-host copy |
| A-06 Close stale claims + handoff dry run | `Partial` | Pilot runbook exists; no handoff dry run recorded |
| A-07 Retrospective + improvement register | `Partial` | No retro report or improvement register |

### 15.2 Phase B: MVP → Production-grade (8 items)

| Item | Status | Blocker |
|------|--------|---------|
| B-01 Assign owners + approve scope | `Blocked` | No product/security/ops/data owner |
| B-02 Served identity + tenant isolation | `Blocked` | No IdP approved; tenant module library-only |
| B-03 Observability + SLO | `Blocked` | No deployment target or ops owner |
| B-04 Delivery governance (CI) | `Blocked` | No remote, branch protection, or CI execution |
| B-05 AI/agent quality controls | `Blocked` | AI not implemented; no provider/data policy |
| B-06 Off-host backup + RPO/RTO | `Partial` | Local bundle restore PASS; no off-host copy |
| B-07 Security review + vuln disposition | `Partial` | cargo audit 0 vulns; no security owner |
| B-08 Performance + capacity model | `Partial` | One local benchmark; no approved workload |

### 15.3 Constraints (non-negotiable)

- `#![forbid(unsafe_code)]` on every Rust file
- Clippy `-D warnings` zero warnings
- `cargo fmt -- --check` clean
- No secrets in repo; `.env.example` only
- No fabricated integrations, permissions, approvals, scan results
- Governance web prototype local-only; no enterprise auth
- Library-only modules must not be described as product capabilities

### 15.4 Validation per work item

Each item: write failing test first (TDD) → implement → run affected checks → code review → rerun full baseline → record evidence in traceability register.

### 15.5 Open decisions (require external input)

- Product owner name (`DEC-01`); security owner (`DEC-12`); operations owner (`DEC-16`); data owner (`DEC-13`); deployment target (`PR-02`)
- Single-tenant vs multi-tenant (`DEC-12`); off-host backup location/RPO/RTO (`DEC-14`); AI in-scope or out (`DEC-15`); compliance scope (`DEC-09`)
- Remote collaboration upgrade only when criteria in `docs/remote-collaboration-criteria.md` are all met

### 15.6 Out of scope

- Enterprise SSO/MFA (IdP not approved)
- Live Microsoft 365/Google connectors; Kubernetes; message queue; relational DB; public network exposure; 24×7 support; regulatory certification
- AI approval or autonomous gate release
- Converting `local_auth` into an enterprise identity system

### 15.7 Section J — Immediate Execution (next 3 slices)

**Slice 1:** Pilot workflow selection + traceability (A-01, A-02, E-01)
- Select pilot workflow, owner, dataset
- Complete acceptance-test matrix for pilot scope
- Complete traceability register rows for pilot scope

**Slice 2:** Governance web edge cases + UX polish (B-01, B-02, C-01, C-02)
- Form validation, input retention, flash messages
- Empty states with next actions
- Error message audit (no technical leakage)
- Role-based 403, expired-session redirect, rate-limit Retry-After

**Slice 3:** Observability + contracts + stale-claims closure (D-01, D-03, E-02, E-03)
- `/ready` endpoint, verdict distribution, auth counters
- Contracts documentation updated
- Stale claims audited and corrected; handoff dry run recorded
- Retro report + improvement register

**After Section J:** Phase A gate rows A2/A3/A5/A6/A7/A10 move toward `Met`; A1 remains `Not met` until a real operator executes the pilot. Phase B remains `Blocked` until external owners/decisions exist.

### 1.1 核心目標

將 Flaps ERP 系統從「多功能耦合的 Electron 桌面應用」收斂為「純帳務登帳與修改功能」，同時將所有 ERP 設定項目（62 項，11 類別）移至獨立的「開發系統 (Dev System)」管理，透過 Rust 核心引擎 + 免費 AI 模型 (JEV) 的混合架構，實現智能化、自動化、高效穩定的整體系統。

### 1.2 目標分解

| 層級 | 目標 | 衡量方式 |
|------|------|---------|
| L1: 效能 | 消除 UI 凍結，降低延遲 10x | KPI 達標 |
| L2: 架構 | 簡化 Flaps ERP 為純帳務功能 | 模組數量減少 50%+ |
| L3: 管理 | 所有 ERP 設定項目集中管理 | 開發系統覆蓋 100% 設定 (62/62) |
| L4: 同步 | 開發系統與 Flaps ERP 即時同步 | 同步延遲 < 1s |
| L5: 智能化 | AI 輔助、自動化、預測分析 | 自動化率 > 80% |
| L6: 穩定性 | 8h 無崩潰、無記憶體洩漏 | Soak 測試通過 |
| L7: 成本 | 全部使用免費/開源工具 | $0/月 |

---

## 2. Rust vs AI 模型分工 (Division of Labor)

### 2.1 分工原則

**核心哲學**：Rust 負責「確定性」，AI 負責「不確定性」。

| 特性 | Rust 處理 | AI 模型 (JEV) 處理 |
|------|----------|-------------------|
| 決定性 | 固定流程、規則明確 | 複雜判斷、語意理解 |
| 效能敏感 | 高頻呼叫、低延遲要求 | 延遲容忍度較高 |
| 穩定性關鍵 | 資料一致性、交易完整性 | 輔助建議、非關鍵路徑 |
| 創造性 | 無 | 報表生成、異常分類、決策建議 |
| 動態性 | 靜態配置 | 動態適應、學習優化 |

### 2.2 分工矩陣

| 功能模組 | 處理方式 | 理由 | 延遲要求 |
|---------|---------|------|---------|
| 帳務登帳 (Ledger Write) | **Rust** | 交易完整性、ACID 保證 | < 10ms |
| 帳務修改 (Ledger Modify) | **Rust** | 版本控制、審計追蹤 | < 50ms |
| 帳務查詢 (Ledger Read) | **Rust** | 高頻、低延遲 | < 50ms |
| COM 橋接 | **Rust** | 執行緒安全、物件池 | < 5ms |
| 資料同步 | **Rust** | 可靠性、衝突解決規則 | < 1s |
| 全文搜尋 | **Rust (Tantivy)** | 效能敏感 | < 50ms |
| 快取管理 | **Rust** | 效能敏感 | < 1ms |
| 會計科目驗證 | **Rust** | 規則明確 | < 5ms |
| 權限檢查 | **Rust** | 安全關鍵 | < 1ms |
| 審計日誌 | **Rust** | 資料完整性 | < 10ms |
| 異常交易分類 | **AI (JEV)** | 語意理解、模式辨識 | < 500ms |
| 設定異常偵測 | **AI (JEV)** | 複雜判斷 | < 500ms |
| 交易風險評分 | **AI (JEV)** | 多因子評估 | < 500ms |
| 報表內容生成 | **AI (JEV)** | 創造性、自然語言 | < 2s |
| 設定建議 | **AI (JEV)** | 動態適應 | < 1s |
| 異常根因分析 | **AI (JEV)** | 複雜推理 | < 2s |
| 使用者意圖理解 | **AI (JEV)** | 語意解析 | < 500ms |
| 資料品質檢查 | **AI (JEV)** | 模式辨識 | < 1s |

### 2.3 AI 決策閘門 (Decision Gating)

```
┌─────────────────────────────────────────────────────────────┐
│              AI 決策閘門 (Decision Gating)                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  交易請求 → Rust 預驗證 → [通過] → AI 風險評分            │
│                              │                              │
│                              ▼                              │
│                    ┌──────────────────┐                     │
│                    │  AI 評分結果     │                     │
│                    │  score < 0.3     │ → 自動通過          │
│                    │  0.3 ≤ score < 0.7│ → 人工審核          │
│                    │  score ≥ 0.7    │ → 自動拒絕 + 告警    │
│                    └──────────────────┘                     │
│                                                             │
│  所有 AI 決策均記錄於 AuditLog，可追溯、可回滾            │
└─────────────────────────────────────────────────────────────┘
```

### 2.4 AI 不可介入的領域 (AI Exclusion Zones)

以下領域 **嚴禁** AI 自動決策，必須由 Rust 規則引擎處理：

1. 帳務金額修改（任何涉及金額的變更）
2. 權限變更（角色、功能、資料權限）
3. 交易刪除/作廢
4. 會計科目變更
5. 結帳/年度結轉操作

---

## 3. 系統架構設計 (4-Layer Architecture)

### 3.1 四層架構總覽

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Flaps ERP 混合架構 (4-Layer)                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Layer 1: 呈現層 (Presentation Layer)                       │   │
│  │  - Tauri 2.0 Webview (Rust 後端 + HTML/CSS/JS 前端)        │   │
│  │  - 帳務登帳 UI / 帳務查詢 UI / 儀表板                      │   │
│  │  - 開發系統 Web UI (設定管理、權限管理)                    │   │
│  └────────────────────────────┬────────────────────────────────┘   │
│                               │ Tauri IPC (非同步)                  │
│  ┌────────────────────────────┴────────────────────────────────┐   │
│  │  Layer 2: API 閘道層 (API Gateway Layer)                    │   │
│  │  - Axum HTTP Server (REST + WebSocket)                     │   │
│  │  - 認證/授權 (JWT + RBAC)                                   │   │
│  │  - 速率限制 / 請求路由 / CORS                               │   │
│  │  - COM 橋接閘道 (windows-rs)                                │   │
│  └────────────────────────────┬────────────────────────────────┘   │
│                               │                                     │
│  ┌────────────────────────────┴────────────────────────────────┐   │
│  │  Layer 3: 應用服務層 (Application Services Layer)           │   │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐        │   │
│  │  │  帳務核心     │ │  開發系統     │ │  AI 引擎     │        │   │
│  │  │  (Rust)      │ │  (Rust)      │ │  (JEV)       │        │   │
│  │  │  - 登帳      │ │  - 設定 CRUD │ │  - 風險評分  │        │   │
│  │  │  - 修改      │ │  - 權限管理  │ │  - 異常分類  │        │   │
│  │  │  - 查詢      │ │  - 版本控制  │ │  - 決策建議  │        │   │
│  │  │  - 審計      │ │  - 同步推送  │ │  - 報表生成  │        │   │
│  │  └──────────────┘ └──────────────┘ └──────────────┘        │   │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐        │   │
│  │  │  同步引擎     │ │  監控服務     │ │  快取層      │        │   │
│  │  │  (Rust)      │ │  (Rust)      │ │  (Rust)      │        │   │
│  │  └──────────────┘ └──────────────┘ └──────────────┘        │   │
│  └────────────────────────────┬────────────────────────────────┘   │
│                               │                                     │
│  ┌────────────────────────────┴────────────────────────────────┐   │
│  │  Layer 4: 基礎設施層 (Infrastructure Layer)                 │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐       │   │
│  │  │ SQLite   │ │  Redis   │ │ Tantivy  │ │ JEV AI   │       │   │
│  │  │ (WAL)    │ │ (Cache)  │ │ (Search) │ │ (Model)  │       │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘       │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐                   │   │
│  │  │  COM     │ │  File    │ │  Network │                   │   │
│  │  │  Bridge  │ │  System  │ │  (TCP)   │                   │   │
│  │  └──────────┘ └──────────┘ └──────────┘                   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 各層職責

| 層級 | 職責 | 技術 | 延遲目標 |
|------|------|------|---------|
| Layer 1 呈現層 | UI 渲染、使用者互動 | Tauri 2.0 + HTML/CSS/JS | < 16ms (60fps) |
| Layer 2 API 閘道層 | 請求路由、認證、限流、COM 橋接 | Axum + windows-rs | < 5ms |
| Layer 3 應用服務層 | 業務邏輯、資料處理、AI 推理 | Rust crates + JEV | < 50ms |
| Layer 4 基礎設施層 | 資料持久化、快取、搜尋、AI 模型 | SQLite/Redis/Tantivy/JEV | < 10ms |

### 3.3 資料流方向

```
使用者操作 → Tauri IPC → Axum Router → Service Layer → Repository → SQLite/Redis/Tantivy
                                    ↓
                              AI Engine (JEV) → 決策建議 → AuditLog
```

---

## 4. 技術選型表 (Technology Selection)

| 類別 | 技術 | 版本 | 用途 | 選擇理由 | 替代方案 | 成本 |
|------|------|------|------|---------|---------|------|
| 語言 | Rust | 1.80+ | 核心引擎 | 記憶體安全、效能、並發 | C++ | $0 |
| Web 框架 | Axum | 0.7+ | HTTP API | Tower 生態、型別安全 | Actix-web | $0 |
| 桌面框架 | Tauri | 2.0 | 呈現層 | 輕量、Rust 後端 | Electron | $0 |
| 全文搜尋 | Tantivy | 0.22+ | 搜尋引擎 | Rust 原生、高效能 | SQLite FTS5 | $0 |
| 資料庫 | SQLite | 3.45+ | 資料持久化 | 嵌入式、WAL 模式 | PostgreSQL | $0 |
| 快取 | Redis | 7.2+ | 快取/佇列 | 高效能、Pub/Sub | Memcached | $0 |
| COM 介接 | windows-rs | 0.58+ | Windows COM | 官方 crate、型別安全 | winax | $0 |
| AI 模型 | JEV (JEV-7B) | latest | 主要 AI 引擎 | 免費、本機部署、中文優化 | - | $0 |
| AI 模型 | DeepSeek-V3 | latest | 複雜推理 | 免費 API、強推理 | - | $0 |
| AI 模型 | Codestral | latest | 程式碼生成 | 免費、程式碼專精 | - | $0 |
| AI 模型 | Llama 3.1 | 8B/70B | 通用推理 | 開源、生態豐富 | - | $0 |
| AI 模型 | Qwen 2.5 | 7B/14B | 中文理解 | 中文優化 | - | $0 |
| AI 模型 | Phi-4 | 14B | 輕量推理 | 小模型高效能 | - | $0 |
| 序列化 | serde + serde_json | 1.0 | 資料序列化 | Rust 標準 | - | $0 |
| 非同步 | tokio | 1.38+ | 非同步執行時 | Rust 標準 | - | $0 |
| ORM | sqlx | 0.8+ | 資料庫存取 | 編譯期檢查 | diesel | $0 |
| 日誌 | tracing | 0.1 | 結構化日誌 | Rust 標準 | - | $0 |
| 測試 | proptest | 1.5+ | 屬性測試 | Rust 屬性測試 | - | $0 |
| 監控 | Prometheus | latest | 指標收集 | 開源標準 | - | $0 |
| 儀表板 | Grafana | latest | 視覺化 | 開源標準 | - | $0 |

---

## 5. 模組分解 (Module Breakdown)

### 5.1 專案目錄結構

```
flaps-erp/
├── Cargo.toml                 # Workspace 根設定
├── core/                      # 核心領域模組
│   ├── src/
│   │   ├── ledger/            # 帳務引擎
│   │   │   ├── mod.rs
│   │   │   ├── transaction.rs # 交易記錄
│   │   │   ├── account.rs     # 會計科目
│   │   │   ├── audit.rs       # 審計日誌
│   │   │   └── engine.rs      # 帳務引擎核心
│   │   ├── config/            # 設定管理
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs      # 設定 Schema
│   │   │   ├── version.rs     # 版本控制
│   │   │   └── sync.rs        # 同步邏輯
│   │   ├── permission/        # 權限管理
│   │   │   ├── mod.rs
│   │   │   ├── rbac.rs        # RBAC 模型
│   │   │   ├── checker.rs     # 權限檢查
│   │   │   └── cache.rs       # 權限快取
│   │   └── domain/            # 領域模型
│   │       ├── mod.rs
│   │       ├── organization.rs
│   │       ├── store.rs
│   │       └── product.rs
│   └── tests/                 # 單元測試
├── platform/                  # 平台介接模組
│   ├── src/
│   │   ├── com/               # COM 橋接
│   │   │   ├── mod.rs
│   │   │   ├── bridge.rs      # COM 橋接服務
│   │   │   ├── pool.rs        # 物件池
│   │   │   ├── batch.rs       # 批次執行器
│   │   │   └── async_ipc.rs   # 非同步 IPC
│   │   ├── db/                # 資料庫存取
│   │   │   ├── mod.rs
│   │   │   ├── sqlite.rs      # SQLite 存取
│   │   │   ├── redis.rs       # Redis 快取
│   │   │   └── migration.rs   # 資料庫遷移
│   │   ├── search/            # 全文搜尋
│   │   │   ├── mod.rs
│   │   │   └── tantivy.rs     # Tantivy 索引
│   │   └── sync/              # 資料同步
│   │       ├── mod.rs
│   │       ├── detector.rs    # 變更偵測
│   │       ├── resolver.rs    # 衝突解決
│   │       └── engine.rs      # 同步引擎
│   └── tests/
├── intelligence/              # AI 智能模組
│   ├── src/
│   │   ├── mod.rs
│   │   ├── client.rs          # AI 模型客戶端
│   │   ├── jev.rs             # JEV 整合
│   │   ├── gating.rs          # 決策閘門
│   │   ├── anomaly.rs         # 異常分類
│   │   ├── scoring.rs         # 交易評分
│   │   └── report.rs          # 報表生成
│   └── tests/
├── web/                       # 呈現層 (Tauri)
│   ├── src-tauri/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands.rs    # Tauri commands
│   │   │   └── state.rs       # 應用狀態
│   │   └── tauri.conf.json
│   └── src/                   # 前端 (HTML/CSS/JS)
│       ├── index.html
│       ├── ledger/            # 帳務 UI
│       ├── devsys/            # 開發系統 UI
│       └── dashboard/         # 儀表板
├── tools/                     # 工具與腳本
│   ├── migration/             # 遷移工具
│   ├── benchmark/             # 效能基準測試
│   └── seed/                  # 種子資料
└── tests/                     # 整合/E2E 測試
    ├── integration/
    ├── performance/
    ├── soak/
    └── e2e/
```

### 5.2 模組依賴關係

```
web/ ──► core/ ──► platform/
  │         │          │
  │         ▼          ▼
  │    intelligence/  platform/sync
  │         │
  │         ▼
  └──► platform/db, platform/search, platform/com
```

---

## 6. 資料流設計 (Data Flow Design)

### 6.1 帳務登帳流程 (Ledger Write)

```
使用者輸入交易資料
    │
    ▼
┌───────────────┐
│  前端驗證     │ → 必填欄位、格式驗證 (Tauri Webview)
└───────┬───────┘
        │ Tauri IPC (非同步)
        ▼
┌───────────────┐
│  Axum Router  │ → 路由分派、認證檢查
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  Rust 預驗證  │ → 業務規則、餘額檢查、權限檢查
└───────┬───────┘
        │ [通過]
        ▼
┌───────────────┐
│  AI 風險評分  │ → JEV 評分 (非同步，不阻塞主流程)
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  帳本寫入     │ → SQLite WAL 模式 (事務)
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  全文索引更新  │ → Tantivy 索引 (非同步)
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  審計日誌     │ → 記錄操作者、時間、變更、AI 評分
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  快取失效     │ → Redis invalidate
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  通知系統     │ → WebSocket 推送 (非同步)
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  回傳結果     │ → Tauri IPC 回傳
└───────────────┘
```

### 6.2 設定同步流程 (Config Sync)

```
開發系統設定變更
    │
    ▼
┌───────────────┐
│  變更偵測     │ → 記錄變更類型、時間戳記、操作者
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  版本控制     │ → 建立新版本、保留舊版本
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  AI 異常偵測  │ → JEV 檢查設定合理性
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  審批流程     │ → 若需審批，等待審批通過 (PERM-04)
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  同步引擎     │ → 增量同步至 Flaps ERP (批次處理)
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  衝突解決     │ → 開發系統優先策略
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  確認回寫     │ → Flaps ERP 確認接收，回寫狀態
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  快取更新     │ → Redis 更新 + 通知
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  通知系統     │ → 通知相關人員同步完成
└───────────────┘
```

### 6.3 AI 決策流程 (AI Decision)

```
交易/設定事件觸發
    │
    ▼
┌───────────────┐
│  事件分類     │ → Rust 判斷是否需要 AI 介入
└───────┬───────┘
        │ [需要 AI]
        ▼
┌───────────────┐
│  上下文組裝   │ → 收集相關資料 (交易歷史、設定、使用者行為)
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  JEV 推理     │ → 本機模型推理 (< 500ms)
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  決策閘門     │ → score < 0.3: 自動通過
│               │   0.3-0.7: 人工審核
│               │   ≥ 0.7: 自動拒絕 + 告警
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  結果記錄     │ → AuditLog (含 AI 評分、理由、模型版本)
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  回饋迴路     │ → 人工審核結果回饋至模型 (持續優化)
└───────────────┘
```

---

## 7. 整合策略 (Integration Strategy)

### 7.1 COM 橋接 (via windows-rs)

```rust
// platform/com/bridge.rs
use windows::Win32::System::Com::*;

pub struct ComBridge {
    pool: ObjectPool<IDispatch>,
    batch_executor: BatchExecutor,
    async_runtime: tokio::runtime::Handle,
}

impl ComBridge {
    pub fn connect(&self, prog_id: &str) -> Result<ComObject, ComError> { ... }
    pub fn invoke(&self, obj: &ComObject, method: &str, args: &[Variant]) -> Result<Variant, ComError> { ... }
    pub fn batch_get(&self, requests: &[BatchRequest]) -> Result<Vec<Variant>, ComError> { ... }
}
```

| 元件 | 說明 | 效能目標 |
|------|------|---------|
| ObjectPool | COM 物件池，最多 50 個物件 | 取得 < 1ms |
| BatchExecutor | 批次 COM 呼叫合併 | 合併率 > 80% |
| AsyncIPC | 非同步 IPC，不阻塞 UI | 延遲 < 5ms |
| SyncFallback | 同步 fallback（相容模式） | 延遲 < 50ms |

### 7.2 資料同步策略

| 策略 | 說明 | 適用場景 |
|------|------|---------|
| 增量同步 | 僅同步變更的記錄 | 日常同步 |
| 批次同步 | 批次處理多筆變更 | 初始同步、大量變更 |
| 事件驅動 | 變更事件觸發同步 | 即時同步 |
| 定時同步 | 定期全量同步 | 資料一致性校驗 |

### 7.3 衝突解決策略

| 衝突類型 | 解決策略 | 說明 |
|---------|---------|------|
| 設定衝突 | 開發系統優先 | 開發系統為唯一設定來源 |
| 資料衝突 | 時間戳記優先 | 最新修改勝出 |
| 版本衝突 | 版本號優先 | 高版本號勝出 |
| 人工介入 | 通知 + 人工決策 | 複雜衝突人工處理 |

---

## 8. 測試與遷移計畫 (Testing & Migration)

### 8.1 測試策略

| 測試類型 | 工具 | 覆蓋率目標 | 說明 |
|---------|------|----------|------|
| 單元測試 | cargo test | > 80% | Rust 函數/模組測試 |
| 屬性測試 | proptest | 關鍵路徑 | 不變量驗證 |
| 整合測試 | cargo test (integration) | 核心流程 | 跨模組整合 |
| 效能測試 | criterion | KPI 達標 | 延遲/吞吐量基準 |
| Soak 測試 | 自訂腳本 | 8h 穩定 | 記憶體洩漏/崩潰 |
| E2E 測試 | Playwright + Tauri | 主要流程 | 端對端使用者流程 |
| AI 決策測試 | 自訂測試集 | > 95% 準確率 | AI 模型驗證 |
| COM 測試 | Mock COM 物件 | 介接正確性 | COM 橋接驗證 |

### 8.2 AI 決策測試細節

| 測試項目 | 方法 | 通過標準 |
|---------|------|---------|
| 風險評分準確率 | 測試集 1000 筆交易 | > 95% |
| 異常分類準確率 | 測試集 500 筆異常 | > 90% |
| 誤判率 (False Positive) | 正常交易被誤判為異常 | < 5% |
| 漏判率 (False Negative) | 異常交易被漏判 | < 2% |
| 推理延遲 | P99 延遲 | < 500ms |
| 模型一致性 | 相同輸入相同輸出 | 100% |

### 8.3 遷移策略

| 階段 | 策略 | 說明 |
|------|------|------|
| Phase 1 | 並行運行 | 新舊系統並行，資料雙寫 |
| Phase 2 | 漸進切換 | 逐模組切換至新系統 |
| Phase 3 | 全量切換 | 所有功能切換至新系統 |
| Phase 4 | 舊系統退役 | 舊系統下線，保留資料備份 |

### 8.4 回滾計畫

| 場景 | 回滾方式 | RTO |
|------|---------|-----|
| 效能未達標 | 切回舊系統 | < 1h |
| 資料不一致 | 從備份恢復 | < 4h |
| COM 橋接失敗 | 啟用同步 fallback | < 5min |
| AI 模型失效 | 停用 AI，使用規則引擎 | < 1min |

---

## 9. 風險控制 (R-01~R-07)

| 風險代碼 | 風險描述 | 可能性 | 影響 | 因應措施 | 負責人 |
|----------|---------|--------|------|---------|--------|
| R-01 | COM 橋接在新架構中不穩定 | 中 | 高 | 保留同步 fallback，逐步遷移，POC 驗證 | Dev B |
| R-02 | 資料同步衝突 | 中 | 中 | 開發系統優先策略，衝突通知人工處理 | Dev A |
| R-03 | 效能目標未達標 | 中 | 高 | 分階段驗證，每階段 KPI 檢查，criterion 基準 | QA |
| R-04 | 使用者適應新系統 | 高 | 中 | 漸進式部署，保留舊系統並行，培訓 | PM |
| R-05 | AI 模型準確率不足 | 中 | 中 | 人工審核機制，逐步提高自動化比例，回饋迴路 | AI 工程師 |
| R-06 | Rust 學習曲線導致延遲 | 中 | 中 | 程式碼審查、結對程式設計、Wellfit 經驗複用 | Dev A/B |
| R-07 | 本機 AI 模型硬體需求過高 | 中 | 高 | 提供多模型選項 (Phi-4 輕量)、雲端 fallback (DeepSeek API) | Dev A |

---

## 10. 成本評估 (Cost Assessment)

### 10.1 成本明細

| 項目 | 技術 | 成本 | 說明 |
|------|------|------|------|
| 開發語言 | Rust | $0 | 開源 |
| Web 框架 | Axum | $0 | 開源 (MIT) |
| 桌面框架 | Tauri | $0 | 開源 (MIT/Apache-2.0) |
| 資料庫 | SQLite | $0 | 公共領域 |
| 快取 | Redis | $0 | 開源 (BSD) |
| 搜尋引擎 | Tantivy | $0 | 開源 (MIT) |
| COM 介接 | windows-rs | $0 | 開源 (MIT) |
| AI 模型 (JEV) | JEV-7B | $0 | 開源 (Apache-2.0) |
| AI 模型 (DeepSeek) | DeepSeek-V3 | $0 | 免費 API |
| AI 模型 (Codestral) | Codestral | $0 | 免費 API |
| 監控 | Prometheus + Grafana | $0 | 開源 |
| CI/CD | GitHub Actions | $0 | 免費額度 |
| **總計** | | **$0/月** | 全部免費/開源 |

### 10.2 硬體需求

| 元件 | 最低需求 | 建議需求 |
|------|---------|---------|
| CPU | 4 核心 | 8 核心 |
| RAM | 8 GB | 16 GB |
| GPU | 無 (CPU 推理) | NVIDIA GPU 8GB VRAM (加速 AI) |
| 磁碟 | 10 GB | 50 GB SSD |
| OS | Windows 10 64-bit | Windows 11 64-bit |

---

## 11. 成功標準 (SC-01~SC-14)

| 編號 | 成功標準 | 可量測完成條件 | 驗證方式 |
|------|---------|---------------|---------|
| SC-01 | 帳務登帳延遲 < 10ms | 主程序日誌 P99 < 10ms | 效能測試 |
| SC-02 | 帳務查詢延遲 < 50ms | 主程序日誌 P99 < 50ms | 效能測試 |
| SC-03 | 系統啟動時間 < 2s | 冷啟動計時 < 2s | 啟動測試 |
| SC-04 | 記憶體 (8h) < +20MB | 程序記憶體監控 | Soak 測試 |
| SC-05 | 並發使用者 50+ | 壓力測試通過 | 壓力測試 |
| SC-06 | 設定同步延遲 < 1s | 同步引擎日誌 P99 < 1s | 整合測試 |
| SC-07 | UI 凍結頻率 = 0 | 8h 監控零凍結 | Soak 測試 |
| SC-08 | COM 物件數量 ≤ 50 | 物件池監控 | 單元測試 |
| SC-09 | 所有 62 項 ERP 設定可管理 | 開發系統功能測試 62/62 | E2E 測試 |
| SC-10 | AI 決策準確率 > 95% | 測試集驗證 | AI 測試 |
| SC-11 | AI 推理延遲 < 500ms | P99 < 500ms | 效能測試 |
| SC-12 | Rust Clippy 零警告 | CI 檢查通過 | CI/CD |
| SC-13 | 8h Soak 測試零崩潰 | Soak 測試報告 | Soak 測試 |
| SC-14 | 成本 = $0/月 | 成本審計 | 財務審查 |

---

## 12. 階段路線圖 (Phased Roadmap, 16 週)

### 12.1 階段概覽

| 階段 | 週數 | 目標 | 關鍵產出 | KPI 檢查點 |
|------|------|------|---------|-----------|
| Phase 1 | 1-4 | 基礎架構搭建 | Rust workspace、COM 橋接、帳務核心、Tauri shell | SM-01, SM-03, SM-08 |
| Phase 2 | 5-8 | 開發系統建置 | 62 項設定 CRUD、權限管理、同步引擎 | SM-06, SC-09 |
| Phase 3 | 9-12 | 智能化與自動化 | JEV 整合、AI 決策閘門、監控儀表板 | SM-09, SM-10, SC-10 |
| Phase 4 | 13-16 | 整合測試與部署 | E2E、Soak、效能測試、漸進式部署 | SC-01~SC-14 |

### 12.2 詳細任務

| 任務代碼 | 任務名稱 | 負責人 | 週數 | 依賴 | 產出 |
|----------|---------|--------|------|------|------|
| T-01 | 建立 Rust Workspace + Cargo 設定 | Dev A | W1 | - | Cargo.toml, 目錄結構 |
| T-02 | 建立 COM 橋接 (windows-rs) | Dev B | W1-2 | - | platform/com/* |
| T-03 | 建立帳務核心 (Ledger Engine) | Dev A | W1-3 | T-01 | core/ledger/* |
| T-04 | 建立 SQLite + sqlx 存取層 | Dev A | W2 | T-01 | platform/db/* |
| T-05 | 建立 Tauri 2.0 Shell | Dev B | W2-3 | T-01 | web/src-tauri/* |
| T-06 | 建立 Axum API 閘道 | Dev A | W3 | T-01 | platform/api/* |
| T-07 | 建立 Redis 快取層 | Dev B | W3 | T-01 | platform/db/redis.rs |
| T-08 | 建立 Tantivy 搜尋引擎 | Dev B | W4 | T-01 | platform/search/* |
| T-09 | 建立開發系統 (62 項設定) | Dev B | W5-7 | T-03, T-04 | core/config/* |
| T-10 | 建立權限管理 (RBAC) | Dev A | W5-6 | T-03 | core/permission/* |
| T-11 | 建立同步引擎 | Dev A | W7-8 | T-09, T-10 | platform/sync/* |
| T-12 | 建立 JEV AI 整合 | Dev A + AI | W9-10 | T-03 | intelligence/* |
| T-13 | 建立 AI 決策閘門 | Dev A + AI | W9-10 | T-12 | intelligence/gating.rs |
| T-14 | 建立監控系統 (Prometheus) | QA | W11 | T-03 | 監控設定 |
| T-15 | 建立 E2E 測試 | QA | W12-13 | T-01~T-14 | tests/e2e/* |
| T-16 | 建立 Soak 測試 | QA | W13-14 | T-15 | tests/soak/* |
| T-17 | 效能測試與優化 | Dev A/B | W14-15 | T-16 | criterion 報告 |
| T-18 | 漸進式部署 | 全體 | W15-16 | T-17 | 部署腳本 |

---

## 13. 文件系列與圖表規劃

### 13.1 文件系列 (10 份)

| 編號 | 文件名稱 | 類型 | 內容 | 優先級 |
|------|---------|------|------|--------|
| DOC-01 | 系統架構總覽文件 | 架構文件 | 整體架構、模組劃分、資料流程 | P1 |
| DOC-02 | Flaps ERP 帳務核心設計文件 | 設計文件 | 帳務登帳、修改、查詢、審計 | P1 |
| DOC-03 | 開發系統設計文件 | 設計文件 | 設定管理、權限管理、版本控制 | P1 |
| DOC-04 | 資料對接同步設計文件 | 設計文件 | 同步策略、衝突解決、版本控制 | P1 |
| DOC-05 | ERP 設定項目完整盤點 | 配置文件 | 62 項 ERP 設定的完整清單 | P1 |
| DOC-06 | 分流處理設計文件 | 設計文件 | 讀/寫/查詢三路分流架構 | P2 |
| DOC-07 | 智能化與自動化設計文件 | 設計文件 | AI 輔助、自動化流程、JEV 整合 | P2 |
| DOC-08 | 效能優化設計文件 | 設計文件 | 效能策略、KPI、監控 | P2 |
| DOC-09 | 部署與維運計畫 | 運維文件 | 部署策略、監控、維護 | P3 |
| DOC-10 | 測試與品質保證計畫 | 測試文件 | 測試策略、驗收標準、AI 測試 | P3 |

### 13.2 圖表系列 (12 個)

| 編號 | 圖表名稱 | 類型 | 用途 | 製作工具 |
|------|---------|------|------|---------|
| FIG-01 | 系統架構總覽圖 | 架構圖 | 展示整體 4 層架構 | draw.io |
| FIG-02 | 分流處理流程圖 | 流程圖 | 展示讀/寫/查詢三路分流 | Mermaid |
| FIG-03 | 開發系統架構圖 | 架構圖 | 展示開發系統內部結構 | draw.io |
| FIG-04 | 資料同步流程圖 | 流程圖 | 展示資料同步機制 | Mermaid |
| FIG-05 | ERP 設定項目關係圖 | ER 圖 | 展示 62 項設定間的依賴關係 | PlantUML |
| FIG-06 | 帳務登帳流程圖 | 流程圖 | 展示帳務登帳流程 | Mermaid |
| FIG-07 | 權限管理模型圖 | 模型圖 | 展示 RBAC 權限模型 | draw.io |
| FIG-08 | 組織架構圖 | 樹狀圖 | 展示組織結構 | draw.io |
| FIG-09 | POS 系統架構圖 | 架構圖 | 展示 POS 與 ERP 整合 | draw.io |
| FIG-10 | 盤點流程圖 | 流程圖 | 展示盤點流程 | Mermaid |
| FIG-11 | 3D 系統架構動畫 | 3D 動畫 | 互動式架構展示 | Three.js/WebGL |
| FIG-12 | 效能監控儀表板 | 儀表板 | KPI 即時監控 | D3.js |

### 13.3 3D 動畫設計建議 (FIG-11)

3D 系統架構動畫的設計要點：
- 使用 Three.js 建立互動式 3D 場景
- 展示系統架構的四層結構（呈現層、API 閘道層、應用服務層、基礎設施層）
- 動畫展示資料流動方向（箭頭動畫）
- 可點擊查看各模組詳情
- 支援旋轉、縮放、平移操作
- 色彩編碼：Rust 核心=藍色、AI 引擎=紫色、COM 橋接=橙色、資料庫=綠色
- JEV 推理過程以粒子動畫展示

---

## 14. ERP 設定項目盤點 (62 項, 11 類別)

### 14.1 盤點方法論

每個設定項目按以下維度分析：
- **目的 (Purpose)**：該設定項目的業務目的
- **依賴 (Dependencies)**：依賴的其他設定項目
- **資料流向 (Data Flow)**：資料的流向（開發系統 → Flaps ERP → POS 等）
- **風險 (Risk)**：設定錯誤或同步失敗的影響
- **自動化程度 (Automation)**：可自動化的程度
- **效能瓶頸 (Bottleneck)**：該項目對效能的影響
- **AI 輔助**：是否適用 AI 輔助管理

### 14.2 完整盤點矩陣

#### 14.2.1 組織管理 (ORG) — 6 項

| 編號 | 設定項目 | 目的 | 依賴 | 資料流向 | 風險 | 自動化 | 效能瓶頸 | 優化方案 | AI 輔助 |
|------|---------|------|------|---------|------|--------|---------|---------|---------|
| ORG-01 | 公司/組織架構 | 定義組織結構 | 無 | Dev→ERP→POS | 高 | 低 | 中 | 樹狀快取，增量同步 | 無 |
| ORG-02 | 部門/事業單位 | 部門編碼與負責人 | ORG-01 | Dev→ERP | 中 | 中 | 低 | 批次同步 | 無 |
| ORG-03 | 職位/職稱 | 職位定義與職等 | ORG-01 | Dev→ERP | 低 | 高 | 低 | 自動同步 | 無 |
| ORG-04 | 員工/人員 | 員工資訊與權限 | ORG-01~03, PERM-* | Dev→ERP→POS | 高 | 中 | 中 | 事件驅動同步 | 無 |
| ORG-05 | 班別/排班 | 排班規則 | ORG-04 | Dev→ERP→POS | 中 | 高 | 低 | 自動排班 | JEV 排班建議 |
| ORG-06 | 工作日曆 | 工作日與假日 | 無 | Dev→ERP | 低 | 高 | 低 | 預載快取 | 無 |

#### 14.2.2 權限管理 (PERM) — 6 項

| 編號 | 設定項目 | 目的 | 依賴 | 資料流向 | 風險 | 自動化 | 效能瓶頸 | 優化方案 | AI 輔助 |
|------|---------|------|------|---------|------|--------|---------|---------|---------|
| PERM-01 | 角色定義 (RBAC) | 角色建立與繼承 | ORG-04 | Dev→ERP | 高 | 中 | 中 | Redis 快取，角色預計算 | 無 |
| PERM-02 | 功能權限 | 選單/按鈕/資料權限 | PERM-01 | Dev→ERP→POS | 高 | 中 | 高 | 權限位元圖，O(1) 查詢 | 無 |
| PERM-03 | 資料權限 | 列/欄位級別權限 | PERM-01, PERM-02 | Dev→ERP | 高 | 低 | 高 | 預計算權限矩陣 | 無 |
| PERM-04 | 審批權限 | 審批流程與層級 | PERM-01 | Dev→ERP | 中 | 中 | 中 | 工作流引擎 | JEV 流程建議 |
| PERM-05 | 操作權限 | CRUD/批次操作權限 | PERM-01, PERM-02 | Dev→ERP | 中 | 高 | 低 | 規則引擎 | 無 |
| PERM-06 | 時間權限 | 操作時間限制 | PERM-01 | Dev→ERP | 低 | 高 | 低 | 快取 + 定時更新 | 無 |

#### 14.2.3 活動管理 (ACT) — 6 項

| 編號 | 設定項目 | 目的 | 依賴 | 資料流向 | 風險 | 自動化 | 效能瓶頸 | 優化方案 | AI 輔助 |
|------|---------|------|------|---------|------|--------|---------|---------|---------|
| ACT-01 | 促銷活動 | 促銷規則與折扣 | PROD-01~03, STORE-01 | Dev→ERP→POS | 高 | 高 | 中 | 規則預編譯，快取結果 | JEV 促銷建議 |
| ACT-02 | 優惠券 | 優惠券類型與條件 | ACT-01, CRM-01 | Dev→ERP→POS | 中 | 高 | 低 | 批次驗證 | 無 |
| ACT-03 | 會員活動 | 會員等級與積分 | CRM-01~02 | Dev→ERP→POS | 中 | 高 | 低 | 事件驅動更新 | JEV 個人化建議 |
| ACT-04 | 節慶活動 | 節慶設定與特殊定價 | ACT-01, STORE-01 | Dev→ERP→POS | 中 | 高 | 低 | 定時觸發 | JEV 節慶預測 |
| ACT-05 | 工作流程 | 審批流程與通知 | PERM-04 | Dev→ERP | 中 | 中 | 中 | 工作流引擎 | JEV 流程優化 |
| ACT-06 | 事件通知 | 郵件/簡訊/系統通知 | ACT-05 | Dev→ERP | 低 | 高 | 低 | 非同步佇列 | 無 |

#### 14.2.4 門市管理 (STORE) — 6 項

| 編號 | 設定項目 | 目的 | 依賴 | 資料流向 | 風險 | 自動化 | 效能瓶頸 | 優化方案 | AI 輔助 |
|------|---------|------|------|---------|------|--------|---------|---------|---------|
| STORE-01 | 門市基本資料 | 門市編號、名稱、地址 | ORG-01 | Dev→ERP→POS | 高 | 中 | 低 | 批次同步 | 無 |
| STORE-02 | 門市類型 | 門市分類與營業模式 | STORE-01 | Dev→ERP | 低 | 高 | 低 | 自動同步 | 無 |
| STORE-03 | 門市營業時間 | 營業時間與特殊日 | STORE-01 | Dev→ERP→POS | 中 | 高 | 低 | 快取 + 定時更新 | 無 |
| STORE-04 | 門市倉庫 | 門市倉庫設定 | STORE-01, INV-01 | Dev→ERP | 中 | 中 | 低 | 批次同步 | 無 |
| STORE-05 | 門市定價 | 門市價格策略 | STORE-01, PROD-03 | Dev→ERP→POS | 高 | 中 | 中 | 增量同步 | JEV 定價建議 |
| STORE-06 | 門市人員 | 門市員工分配 | STORE-01, ORG-04 | Dev→ERP | 中 | 中 | 低 | 事件驅動 | 無 |

#### 14.2.5 POS 機與收銀機 (POS) — 7 項

| 編號 | 設定項目 | 目的 | 依賴 | 資料流向 | 風險 | 自動化 | 效能瓶頸 | 優化方案 | AI 輔助 |
|------|---------|------|------|---------|------|--------|---------|---------|---------|
| POS-01 | POS 機基本資料 | POS 機編號、型號、IP、MAC | STORE-01 | Dev→ERP→POS | 高 | 中 | 低 | 自動註冊 | 無 |
| POS-02 | 收銀機設定 | 收銀機參數、列印設定 | POS-01 | Dev→ERP→POS | 中 | 高 | 低 | 配置檔案推送 | 無 |
| POS-03 | 支付方式 | 現金、信用卡、行動支付 | POS-01 | Dev→ERP→POS | 高 | 中 | 低 | 批次同步 | 無 |
| POS-04 | 發票設定 | 發票格式、字軌、稅率 | POS-01, ACCT-04 | Dev→ERP→POS | 高 | 中 | 低 | 自動更新 | 無 |
| POS-05 | 收銀員 | 收銀員分配、權限、班別 | POS-01, ORG-04, PERM-01 | Dev→ERP→POS | 高 | 中 | 中 | 事件驅動 | 無 |
| POS-06 | 錢箱設定 | 錢箱金額、找零、盤點規則 | POS-01 | Dev→ERP→POS | 中 | 高 | 低 | 自動同步 | 無 |
| POS-07 | 連線設定 | POS 與 ERP 連線、同步頻率 | POS-01 | Dev→ERP→POS | 高 | 中 | 高 | 自動重連 + 心跳 | 無 |

#### 14.2.6 盤點管理 (INV) — 6 項

| 編號 | 設定項目 | 目的 | 依賴 | 資料流向 | 風險 | 自動化 | 效能瓶頸 | 優化方案 | AI 輔助 |
|------|---------|------|------|---------|------|--------|---------|---------|---------|
| INV-01 | 盤點類型 | 全盤、抽盤、循環盤點 | PROD-01, STORE-04 | Dev→ERP | 中 | 高 | 低 | 自動排程 | 無 |
| INV-02 | 盤點頻率 | 盤點週期、排程 | INV-01 | Dev→ERP | 低 | 高 | 低 | 定時觸發 | 無 |
| INV-03 | 盤點範圍 | 盤點商品/倉庫範圍 | INV-01, PROD-01 | Dev→ERP | 中 | 高 | 低 | 自動生成 | 無 |
| INV-04 | 盤點人員 | 盤點人員分配、複核 | ORG-04, PERM-01 | Dev→ERP | 中 | 中 | 低 | 自動分配 | 無 |
| INV-05 | 盤點差異 | 差異容許範圍、自動調整 | INV-01 | Dev→ERP | 高 | 中 | 中 | 規則引擎 | JEV 差異分析 |
| INV-06 | 盤點報表 | 盤點報表格式、匯出 | RPT-01 | Dev→ERP | 低 | 高 | 低 | 自動產生 | 無 |

#### 14.2.7 商品管理 (PROD) — 6 項

| 編號 | 設定項目 | 目的 | 依賴 | 資料流向 | 風險 | 自動化 | 效能瓶頸 | 優化方案 | AI 輔助 |
|------|---------|------|------|---------|------|--------|---------|---------|---------|
| PROD-01 | 商品基本資料 | 商品編號、名稱、規格 | 無 | Dev→ERP→POS | 高 | 中 | 中 | 增量同步 | 無 |
| PROD-02 | 商品分類 | 商品分類樹、屬性 | PROD-01 | Dev→ERP | 低 | 高 | 低 | 樹狀快取 | 無 |
| PROD-03 | 商品定價 | 售價、成本、促銷價 | PROD-01, ACT-01 | Dev→ERP→POS | 高 | 中 | 高 | 增量同步 + 快取 | JEV 定價建議 |
| PROD-04 | 商品條碼 | 條碼類型、多條碼 | PROD-01 | Dev→ERP→POS | 中 | 高 | 低 | 批次同步 | 無 |
| PROD-05 | 商品供應商 | 供應商分配、採購價 | PROD-01, CRM-03 | Dev→ERP | 中 | 中 | 低 | 批次同步 | 無 |
| PROD-06 | 商品庫存 | 安全庫存、補貨點 | PROD-01, STORE-04 | Dev→ERP | 中 | 高 | 中 | 自動計算 | JEV 補貨建議 |

#### 14.2.8 帳務管理 (ACCT) — 6 項

| 編號 | 設定項目 | 目的 | 依賴 | 資料流向 | 風險 | 自動化 | 效能瓶頸 | 優化方案 | AI 輔助 |
|------|---------|------|------|---------|------|--------|---------|---------|---------|
| ACCT-01 | 會計科目 | 科目編號、名稱、類型 | 無 | Dev→ERP | 高 | 低 | 低 | 版本控制 | 無 |
| ACCT-02 | 帳簿設定 | 帳簿類型、記帳規則 | ACCT-01 | Dev→ERP | 高 | 低 | 低 | 版本控制 | 無 |
| ACCT-03 | 幣別設定 | 幣別、匯率、轉換 | 無 | Dev→ERP | 中 | 高 | 低 | 自動更新匯率 | 無 |
| ACCT-04 | 稅務設定 | 稅率、稅務代碼 | 無 | Dev→ERP→POS | 高 | 中 | 低 | 自動同步 | 無 |
| ACCT-05 | 傳票設定 | 傳票類型、編號、審批 | ACCT-01, PERM-04 | Dev→ERP | 高 | 中 | 中 | 規則引擎 | 無 |
| ACCT-06 | 結帳設定 | 結帳週期、年度結轉 | ACCT-01 | Dev→ERP | 高 | 低 | 低 | 手動審批 | 無 |

#### 14.2.9 客戶與供應商 (CRM) — 4 項

| 編號 | 設定項目 | 目的 | 依賴 | 資料流向 | 風險 | 自動化 | 效能瓶頸 | 優化方案 | AI 輔助 |
|------|---------|------|------|---------|------|--------|---------|---------|---------|
| CRM-01 | 客戶基本資料 | 客戶編號、名稱、聯絡 | 無 | Dev→ERP→POS | 高 | 中 | 中 | 增量同步 | 無 |
| CRM-02 | 客戶等級 | 會員等級、折扣、積分 | CRM-01, ACT-03 | Dev→ERP→POS | 中 | 高 | 低 | 事件驅動 | JEV 個人化 |
| CRM-03 | 供應商基本資料 | 供應商編號、名稱 | 無 | Dev→ERP | 中 | 中 | 低 | 批次同步 | 無 |
| CRM-04 | 供應商評級 | 供應商評級、採購條件 | CRM-03 | Dev→ERP | 低 | 中 | 低 | 自動評分 | JEV 評級建議 |

#### 14.2.10 報表與分析 (RPT) — 4 項

| 編號 | 設定項目 | 目的 | 依賴 | 資料流向 | 風險 | 自動化 | 效能瓶頸 | 優化方案 | AI 輔助 |
|------|---------|------|------|---------|------|--------|---------|---------|---------|
| RPT-01 | 報表範本 | 報表格式、欄位設定 | 無 | Dev→ERP | 低 | 高 | 低 | 模板引擎 | 無 |
| RPT-02 | 排程報表 | 自動產生報表 | RPT-01 | Dev→ERP | 低 | 高 | 中 | 背景任務 | JEV 報表生成 |
| RPT-03 | 儀表板 | KPI 儀表板 | RPT-01 | Dev→ERP | 低 | 高 | 中 | 快取 + 增量更新 | 無 |
| RPT-04 | 資料匯出 | 匯出格式、排程 | RPT-01 | Dev→ERP | 低 | 高 | 低 | 非同步匯出 | 無 |

#### 14.2.11 系統管理 (SYS) — 5 項

| 編號 | 設定項目 | 目的 | 依賴 | 資料流向 | 風險 | 自動化 | 效能瓶頸 | 優化方案 | AI 輔助 |
|------|---------|------|------|---------|------|--------|---------|---------|---------|
| SYS-01 | 系統參數 | 全域參數設定 | 無 | Dev→ERP | 中 | 高 | 低 | 快取 + 熱更新 | 無 |
| SYS-02 | 資料備份 | 備份排程、保留政策 | 無 | Dev→ERP | 高 | 高 | 中 | 自動備份 + 驗證 | 無 |
| SYS-03 | 日誌設定 | 日誌等級、保留期限 | 無 | Dev→ERP | 低 | 高 | 低 | 動態調整 | 無 |
| SYS-04 | 整合設定 | 外部系統整合、API 金鑰 | 無 | Dev→ERP | 高 | 中 | 低 | 安全儲存 | 無 |
| SYS-05 | 安全設定 | 密碼政策、Session 設定 | 無 | Dev→ERP | 高 | 中 | 低 | 安全審計 | 無 |

### 14.3 統計摘要

| 類別 | 項目數 | 高風險數 | AI 輔助數 |
|------|--------|---------|----------|
| ORG | 6 | 2 | 1 |
| PERM | 6 | 3 | 1 |
| ACT | 6 | 1 | 3 |
| STORE | 6 | 2 | 1 |
| POS | 7 | 4 | 0 |
| INV | 6 | 1 | 1 |
| PROD | 6 | 2 | 2 |
| ACCT | 6 | 4 | 0 |
| CRM | 4 | 1 | 2 |
| RPT | 4 | 0 | 1 |
| SYS | 5 | 3 | 0 |
| **合計** | **62** | **23** | **12** |

---

## 15. AI 模型推薦 (AI Model Recommendations)

### 15.1 模型清單

| 編號 | 模型 | 參數量 | 類型 | 主要用途 | 部署方式 | 成本 |
|------|------|--------|------|---------|---------|------|
| AI-01 | JEV (JEV-7B) | 7B | 本機模型 | 主要 AI 引擎：異常分類、風險評分、決策建議 | 本機 GPU/CPU | $0 |
| AI-02 | DeepSeek-V3 | 671B (MoE) | API | 複雜推理、根因分析、報表生成 | 免費 API | $0 |
| AI-03 | Codestral | 22B | API | 程式碼生成、SQL 生成、設定腳本 | 免費 API | $0 |
| AI-04 | Llama 3.1 | 8B/70B | 本機/API | 通用推理、對話、文件摘要 | 本機/API | $0 |
| AI-05 | Qwen 2.5 | 7B/14B | 本機/API | 中文理解、中文報表生成 | 本機/API | $0 |
| AI-06 | Phi-4 | 14B | 本機模型 | 輕量推理、快速回應、資源受限環境 | 本機 CPU | $0 |

### 15.2 模型選擇策略

```
┌─────────────────────────────────────────────────────────────┐
│              AI 模型選擇策略 (Model Selection)               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  任務分類 → 複雜度評估 → 模型選擇                          │
│                                                             │
│  ┌─────────────┐                                           │
│  │ 簡單任務     │ → Phi-4 (CPU, < 200ms)                  │
│  │ (分類/標籤)  │                                           │
│  └─────────────┘                                           │
│                                                             │
│  ┌─────────────┐                                           │
│  │ 中等任務     │ → JEV-7B (GPU, < 500ms)                 │
│  │ (風險評分)   │                                           │
│  └─────────────┘                                           │
│                                                             │
│  ┌─────────────┐                                           │
│  │ 複雜任務     │ → DeepSeek-V3 (API, < 2s)               │
│  │ (根因分析)   │                                           │
│  └─────────────┘                                           │
│                                                             │
│  ┌─────────────┐                                           │
│  │ 程式碼任務   │ → Codestral (API, < 2s)                 │
│  │ (SQL/腳本)   │                                           │
│  └─────────────┘                                           │
│                                                             │
│  Fallback: 若本機模型不可用 → DeepSeek API                 │
└─────────────────────────────────────────────────────────────┘
```

### 15.3 模型效能比較

| 模型 | 推理速度 (tokens/s) | 記憶體需求 | 中文能力 | 程式碼能力 | 推薦場景 |
|------|-------------------|-----------|---------|-----------|---------|
| JEV-7B | 30-50 (GPU) | 8GB VRAM | 優秀 | 良好 | 主要引擎 |
| Phi-4 | 15-25 (CPU) | 8GB RAM | 良好 | 良好 | 輕量/快速 |
| DeepSeek-V3 | API | 無 (雲端) | 優秀 | 優秀 | 複雜推理 |
| Codestral | API | 無 (雲端) | 一般 | 優秀 | 程式碼生成 |
| Qwen 2.5 7B | 25-40 (GPU) | 6GB VRAM | 優秀 | 良好 | 中文報表 |
| Llama 3.1 8B | 25-40 (GPU) | 6GB VRAM | 一般 | 良好 | 通用 |

---

## 16. JEV 整合策略 (JEV Integration Strategy)

### 16.1 整合架構

```
┌─────────────────────────────────────────────────────────────┐
│              JEV 整合架構 (JEV Integration)                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Rust 應用服務層                                     │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐         │   │
│  │  │ 事件收集  │  │ 上下文   │  │ 結果處理  │         │   │
│  │  │          │  │ 組裝     │  │          │         │   │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘         │   │
│  │       │             │             │                │   │
│  └───────┼─────────────┼─────────────┼────────────────┘   │
│          │             │             │                     │
│          ▼             ▼             ▼                     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  JEV 客戶端 (HTTP API to 本機模型服務)              │   │
│  │  - POST /v1/chat/completions (OpenAI 相容)          │   │
│  │  - 超時: 5s, 重試: 2 次                            │   │
│  │  - 快取: 相同輸入快取 5 分鐘                        │   │
│  └──────────────────────┬──────────────────────────────┘   │
│                         │                                   │
│                         ▼                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  JEV 模型服務 (本機)                                │   │
│  │  - llama.cpp / Ollama 後端                          │   │
│  │  - GPU 加速 (CUDA)                                  │   │
│  │  - 模型: JEV-7B-Q4_K_M                            │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 16.2 三大整合策略

#### 16.2.1 決策閘門 (Decision Gating)

**目的**：AI 輔助交易決策，降低風險

```
交易請求 → Rust 預驗證 → AI 風險評分 → 決策閘門 → 執行/審核/拒絕
```

| 評分範圍 | 動作 | 說明 |
|---------|------|------|
| score < 0.3 | 自動通過 | 低風險，自動執行 |
| 0.3 ≤ score < 0.7 | 人工審核 | 中風險，需主管審核 |
| score ≥ 0.7 | 自動拒絕 + 告警 | 高風險，拒絕並告警 |

**JEV Prompt 模板**：
```
你是一個 ERP 帳務風險評估專家。請評估以下交易的風險等級 (0.0-1.0)：
交易類型: {txn_type}
金額: {amount} {currency}
帳戶: {account_code} - {account_name}
操作者: {operator}
歷史異常次數: {anomaly_count}
最近 24h 同類交易次數: {recent_count}

請回傳 JSON: {"score": 0.0-1.0, "reason": "評估理由", "factors": ["因子1", "因子2"]}
```

#### 16.2.2 異常分類 (Anomaly Classification)

**目的**：自動識別異常交易與設定變更

| 異常類型 | 說明 | JEV 處理 | 告警等級 |
|---------|------|---------|---------|
| 金額異常 | 交易金額超出歷史範圍 | JEV 分類 + 原因分析 | 高 |
| 頻率異常 | 短時間大量交易 | JEV 分類 | 中 |
| 時間異常 | 非營業時間操作 | JEV 分類 | 低 |
| 權限異常 | 超越權限的操作 | Rust 規則 (非 AI) | 高 |
| 設定異常 | 不合理的設定變更 | JEV 分類 + 建議 | 中 |
| 資料不一致 | 跨系統資料不一致 | JEV 根因分析 | 高 |

**JEV Prompt 模板**：
```
你是一個 ERP 異常偵測專家。請分析以下事件是否為異常，並分類：
事件類型: {event_type}
事件資料: {event_data}
歷史正常範圍: {normal_range}
相關設定: {related_config}

請回傳 JSON: {"is_anomaly": true/false, "category": "分類", "severity": "high/medium/low", "reason": "原因", "suggestion": "建議處理方式"}
```

#### 16.2.3 交易評分 (Transaction Scoring)

**目的**：多因子交易評分，輔助決策

| 評分因子 | 權重 | 說明 | 資料來源 |
|---------|------|------|---------|
| 金額因子 | 25% | 交易金額 vs 歷史平均 | SQLite |
| 頻率因子 | 20% | 近期同類交易頻率 | SQLite |
| 時間因子 | 15% | 操作時間 vs 營業時間 | SYS-01 |
| 操作者因子 | 15% | 操作者歷史異常率 | AuditLog |
| 帳戶因子 | 15% | 帳戶風險等級 | ACCT-* |
| 設定因子 | 10% | 相關設定變更頻率 | Config |

**JEV 綜合評分 Prompt**：
```
你是一個 ERP 交易評分專家。請根據以下多因子資訊，計算綜合風險評分 (0.0-1.0)：
金額因子: {amount_factor} (權重 25%)
頻率因子: {frequency_factor} (權重 20%)
時間因子: {time_factor} (權重 15%)
操作者因子: {operator_factor} (權重 15%)
帳戶因子: {account_factor} (權重 15%)
設定因子: {config_factor} (權重 10%)

請回傳 JSON: {"final_score": 0.0-1.0, "breakdown": {"amount": x, "frequency": x, "time": x, "operator": x, "account": x, "config": x}, "recommendation": "approve/review/reject", "reason": "綜合評估理由"}
```

### 16.3 JEV 部署配置

| 項目 | 配置 | 說明 |
|------|------|------|
| 後端框架 | Ollama / llama.cpp | 本機模型服務 |
| 模型檔案 | JEV-7B-Q4_K_M.gguf | 量化版本，~4GB |
| GPU 加速 | CUDA (NVIDIA) | 可選，加速推理 |
| API 端點 | http://localhost:11434 | OpenAI 相容 API |
| 超時設定 | 5s (簡單) / 10s (複雜) | 避免阻塞 |
| 快取策略 | 相同輸入快取 5 分鐘 | 減少重複推理 |
| Fallback | DeepSeek API | 本機模型不可用時 |
| 監控 | Prometheus metrics | 推理延遲/成功率 |

### 16.4 JEV 與 Rust 的通訊協議

```rust
// intelligence/jev.rs
pub struct JevClient {
    base_url: String,      // http://localhost:11434
    http_client: reqwest::Client,
    cache: LruCache<String, JevResponse>,
    timeout: Duration,
}

impl JevClient {
    pub async fn evaluate_risk(&self, txn: &Transaction) -> Result<RiskScore, JevError> {
        let prompt = format_risk_prompt(txn);
        let cache_key = hash(&prompt);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached);
        }
        let response = self.http_client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&JevRequest { model: "jev-7b", messages: vec![...], max_tokens: 256 })
            .timeout(self.timeout)
            .send().await?;
        // 解析 + 快取 + 回傳
    }
}
```

---

## 17. 結論

本計畫 (v3.0) 以第一性原理為基礎，將 Flaps ERP 簡化為純帳務登帳與修改功能，所有 ERP 設定項目（62 項，11 類別）移至獨立的開發系統管理。採用 **Rust 核心 + 免費 AI 模型 (JEV)** 的混合架構，Rust 負責確定性/效能敏感/穩定性關鍵的固定流程，AI 負責複雜判斷/創造性/動態適應。透過 4 層系統架構、分流處理、COM 橋接 (windows-rs)、資料同步、AI 決策閘門，全面實現智能化、自動化、高效穩定的整體系統。所有技術皆為免費/開源，成本 $0/月。本計畫將產出 10 份專業文件與 12 個圖表（含 3D 動畫），為架構設計與溝通提供完整支援。

---

*本計畫為架構優化規劃文件，需主管核准後方可進入實作階段。*
*文件版本：v3.0 | 日期：2026-10-27 | 編號：FLAPS-ARCH-OPT-003*
