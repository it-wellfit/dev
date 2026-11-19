# 組合與分層設計 — ERP 效能診斷工具鏈（Pilot, 2026-11-19）
（Composition & Layering Design）

> 版本：v1.0｜日期：2026-11-19｜狀態：Draft（Pilot）
> 方法論依據：`docs/standards/TOOLCHAIN_METHODOLOGY.md` §3
> 決策依據：`docs/toolchain/scoring.md`（Sysinternals 4.30 直接入選；WPT 3.45 冷備援候選；TaskMgr 3.65 既有基準）

---

## 3.1 分層設計（Layering）

| 工具 | 層級 | 理由 |
|---|---|---|
| Sysinternals Suite（Process Monitor/Explorer/Handle 等） | **Satellite（衛星層）** | 低頻診斷／臨時用途；淺整合（人工作業級）；不寫入自動化流程 |
| Windows Performance Toolkit (WPT) | **Satellite（衛星層，冷備援）** | 僅在 Sysinternals 無法覆蓋時啟動 POC；目前不啟用 |
| Task Manager + Resource Monitor | **Satellite（既有基準）** | 所有 Windows 使用者已熟悉；作為最低門檻 fallback |

**分層紀律**：Satellite 工具禁止進入 CI／自動化管線。若未來診斷流程自動化（例如排程擷取），Sysinternals 必須升級為 Adjacent 並補足相應治理（ADR + 介面標準化）。

## 3.2 冗餘備援設計（Redundancy）

| 備援等級 | 適用對象 | 本案例 |
|---|---|---|
| 熱備援（Hot Standby） | Core 層中斷即營運中斷者 | 不適用（無 Core 層工具） |
| 冷備援（Cold Standby） | Core 層可容忍數日中斷者 | WPT 作為 Sysinternals 的冷備援（替代路徑文件化於 Runbook） |
| 免備援 | Satellite 層、唯讀診斷工具 | Sysinternals、TaskMgr（中斷不造成損失） |

**備援成本控制**：備援僅要求「路徑存在且演練過」，不要求雙工具並行維運。

## 3.3 介面標準化（Interface Standardization）

| 介面類型 | 標準要求（本案例） |
|---|---|
| **API** | 不適用（Sysinternals 無 REST API） |
| **CLI** | Process Monitor：`procmon /AcceptEula /Minimized /BackingFile <path> /Quiet`；Process Explorer：`procexp /t`（樹狀）；Handle：`handle <filename>`；退出碼語意：0＝成功、1＝失敗（依工具宣告，不假設一致）；stdout 資料、stderr 診斷；支援 `/SaveAs` 匯出 CSV |
| **事件匯流排** | 不適用（Satellite 層不接入事件匯流排） |

## 3.4 決策樹走查（Build / Buy / Open Source / Hybrid）

```
新工具需求：ERP 效能診斷
  ├─ 屬核心差異化能力？ → 否（診斷非產品競爭優勢）
  ├─ 市場有成熟解？ → 是（Sysinternals、WPT、PerfView 等 ≥3 個可驗證產品）
  ├─ 開源授權可接受？ → 是（Microsoft 免費授權，非開源但免費且可再散布條件明確）
  └─ 資料/合規要求本地部署？ → 是（診斷資料含行程/檔案路徑，須本地處理）
  → 結論：採用（Buy/Free）+ 本地部署（Hybrid 之本地部分）
```

**決策樹使用規則檢核**
1. 每個分支裁定附證據：「市場有成熟解」列出 3 個可驗證產品（Sysinternals、WPT、PerfView）。
2. 「自建」不適用（非自建）。
3. 「採購」附：退出條款（工具免費、可隨時移除，無合約）；資料匯出驗證（ProcMon 匯出 CSV，已驗證）；鎖定風險評級 **低**（資料格式標準 CSV，可攜）。
4. ADR 記錄決策與替代方案（見 `docs/adr/ADR-0015-erp-diagnosis-toolchain.md`）。

## 3.5 工具生命週期（Tool Lifecycle）

- **入選**：Sysinternals Suite（2026-11-19，評分 4.30）。
- **複審觸發**：Microsoft Store 自動更新、FlapsErp 廠商端問題型態變更、每季 governance。
- **退役條件**：走退役清單（替代路徑上線＋資料遷移＋依賴清點＋Runbook 更新＋ADR 記錄）。

## 證據（Evidence）

- Microsoft Store 套件頁：2026-11-18 擷取。
- ProcMon CLI 參數：官方文件（docs.microsoft.com/sysinternals）。
- WPT：Windows SDK 官方文件。
- PerfView：Microsoft 開源（GitHub），作為第三候選參考。

## 未量測項（UNKNOWN 清單）

- UNK-07：ProcMon CSV 匯出與現有報告產生器的實際相容性（未實測）。
- UNK-08：WPT 作為冷備援的切換演練（未執行）。
