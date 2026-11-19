# 風險與緩解清單 — ERP 效能診斷工具鏈（Pilot, 2026-11-19）
（Risk & Mitigation Register — ERP Performance Diagnosis）

> 版本：v1.0｜日期：2026-11-19｜狀態：Draft（Pilot）
> 方法論依據：`docs/standards/TOOLCHAIN_METHODOLOGY.md` §5.4
> 等級門檻：≥ 12 高風險（須負責人簽核緩解計畫）；8–11 中（列入季審）；< 8 低（觀察）

---

## 風險清單

| ID | 風險描述 | 可能性(1-5) | 衝擊(1-5) | 等級(=可能性×衝擊) | 緩解措施 | 應變措施 | 負責人 | 狀態 |
|---|---|---|---|---|---|---|---|---|
| RSK-01 | ProcMon 未設過濾器產生 GB 級資料拖慢生產機 | 4 | 4 | 16 | 僅診斷 session 啟用＋標準過濾器範本（排除系統雜訊） | 立即停止擷取＋還原環境 | 支援工程師 | OPEN |
| RSK-02 | 診斷工具常駐生產環境造成效能劣化 | 3 | 4 | 12 | Satellite 層紀律：禁止進入 CI／自動化管線；僅診斷 session 啟用 | 停用工具＋審查流程 | 支援工程師 | OPEN |
| RSK-03 | Microsoft Store 自動更新改變工具行為 | 2 | 3 | 6 | 鎖定版本（記錄版本號於 Runbook）＋每季評估活躍度 | 回退至已驗證版本 | 方法論負責人 | OPEN |
| RSK-04 | ProcMon CSV 與報告產生器不相容阻礙自動化 | 3 | 3 | 9 | 完成匯出格式實測（TD-04） | 改採人工整理或更換匯出格式 | 支援工程師 | OPEN |
| RSK-05 | WPT 冷備援不可用（未演練） | 2 | 3 | 6 | 完成切換演練（TD-03） | 啟用 Task Manager fallback | 方法論負責人 | OPEN |
| RSK-06 | 定位時間基準值不精確導致評分失真 | 4 | 2 | 8 | 完成 TTFV 實測（TD-02） | 以 MANUAL 標註並列入補測 | 方法論負責人 | OPEN |

## 高風險項（≥12）需負責人簽核緩解計畫

- RSK-01（16）：須簽核標準過濾器範本。
- RSK-02（12）：須簽核 Satellite 層紀律與診斷 session 管控流程。

## 證據（Evidence）

- Microsoft Store 套件頁：2026-11-18 擷取。
- ProcMon 官方文件：CLI 參數與 CSV 匯出。
- 負責人：待指派（本 pilot 為文件層級，無實際人員）。
