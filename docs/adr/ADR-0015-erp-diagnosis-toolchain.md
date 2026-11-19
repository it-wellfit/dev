# ADR-0015: ERP 效能診斷工具鏈採用 Sysinternals Suite（Satellite 層）

| 欄位 | 內容 |
|------|------|
| ADR 編號 | ADR-0015 |
| 標題 | ERP 效能診斷工具鏈採用 Sysinternals Suite 作為 Satellite 層診斷工具 |
| 狀態 | Accepted |
| 決策者 | 方法論負責人（依《TOOLCHAIN_METHODOLOGY.md》模組二/三流程） |
| 日期 | 2026-11-19 |
| 關聯 | TOOLCHAIN_METHODOLOGY.md §2/§3；docs/toolchain/requirements.md、scoring.md、composition.md；ADR-0014（衍生檔治理） |

## 1. 背景與問題

- ERP 慢速問題平均定位時間約 4 小時（2026-11 支援工程師手動排查，估算值，標 `MANUAL`）。
- 現有手動流程缺乏行程級與 IO 級可見性（Runbook v1.0 §3）。
- 需要標準化診斷流程以縮短定位時間至 15 分鐘（OBJ-01）。

## 2. 考慮的選項

| 決策點 | 選項 A（採用） | 選項 B（不採用） | 理由 |
|--------|---------------|------------------|------|
| 工具選擇 | Sysinternals Suite（Process Monitor/Explorer/Handle 等） | Windows Performance Toolkit (WPT) / PerfView | Sysinternals 評分 4.30（直接入選）；WPT 3.45（冷備援候選）；TaskMgr 3.65（既有基準 fallback） |
| 層級 | Satellite（衛星層） | Adjacent / Core | 低頻診斷／臨時用途，淺整合；Satellite 禁止進入 CI／自動化管線 |
| 備援 | 免備援（Satellite）+ WPT 冷備援 | 熱備援 | Satellite 中斷不造成損失；WPT 作為替代路徑文件化於 Runbook |
| 部署 | 本地部署（診斷資料含行程/檔案路徑） | 雲端 | 資料/合規要求本地處理 |

## 3. 決策

1. **納入 Sysinternals Suite**（Process Monitor、Process Explorer、Handle、RAMMap、VMMap、TCPView、ClockRes、Contig）為 Satellite 層診斷工具。
2. **僅診斷 session 啟用**，禁止常駐生產環境（Satellite 層紀律）。
3. **使用方式寫入 Runbook** §診斷流程（含標準 ProcMon 過濾器範本）。
4. **WPT 列為冷備援候選**，視需求啟動 POC（POC-20261119-02 預留）。
5. **Task Manager + Resource Monitor 保留為既有基準 fallback**。

## 4. 實施與正面結果（2026-11-19 文件層級）

| 驗證 | 結果 |
|------|------|
| 評分矩陣 | Sysinternals 4.30（直接入選）、WPT 3.45、TaskMgr 3.65 |
| 否決權檢核 | 全部 PASS（資安／法務／鎖定） |
| 分層設計 | Satellite 層，免備援 + WPT 冷備援 |
| 風險清單 | 6 條（2 條高風險 RSK-01/RSK-02 需簽核） |

## 5. 替代方案（已否決）

| 方案 | 否決理由 | 證據 |
|---|---|---|
| 自建 ETW 追蹤器 | 開發成本高且非核心差異化（決策樹 D→E→F 路徑） | 決策樹 §3.4 |
| 僅使用 WPT | 評分 3.45（POC 範圍），TTFV 預估 ≥1 週，ETW 概念陡峭 | scoring.md D3 |
| 僅使用 Task Manager | D1=2（僅基本概覽，無檔案/登錄/控制代碼追蹤） | scoring.md D1 |

## 6. 關聯（References）

- 情境 ID：SCN-01、SCN-02
- 任務 ID：TSK-01、TSK-02、TSK-03
- 驗收標準：AC-01、AC-02、AC-03
- 風險清單：RSK-01 ~ RSK-06（docs/toolchain/risks.md）
- 技術債：TD-01 ~ TD-04（docs/toolchain/debt-register.md）
- POC：POC-20261119-01（docs/toolchain/validation.md）

## 7. 後續工作

- [ ] 啟動 POC-20261119-01（Sysinternals 診斷流程驗證）
- [ ] 完成 TTFV 實測（TD-02）
- [ ] 完成 ProcMon CSV 匯出相容性實測（TD-04）
- [ ] 撰寫 Runbook §診斷流程（含標準過濾器範本）
- [ ] WPT 冷備援切換演練（TD-03）
