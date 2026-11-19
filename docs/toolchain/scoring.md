# 工具評分矩陣 — ERP 效能診斷（Pilot, 2026-11-19）
（Tool Scoring Matrix — ERP Performance Diagnosis）

> 版本：v1.0｜日期：2026-11-19｜狀態：Draft（Pilot）
> 方法論依據：`docs/standards/TOOLCHAIN_METHODOLOGY.md` §2
> 候選工具：A=Sysinternals Suite（Process Monitor/Explorer/Handle 等）、B=Windows Performance Toolkit (WPT/xperf)、C=內建 Task Manager + Resource Monitor（現有基準）
> 權重版本：v1（30/25/15/15/15），受監管產業調整不適用（本案例為內部 IT 診斷）

---

## 評分矩陣

| 維度 | 權重 | A Sysinternals | B WPT | C TaskMgr/ResMon |
|---|---|---|---|---|
| D1 功能覆蓋率 | 30% | 5 | 4 | 2 |
| D2 整合相容性 | 25% | 4 | 3 | 5 |
| D3 學習曲線 | 15% | 3 | 2 | 5 |
| D4 授權／TCO | 15% | 5 | 5 | 5 |
| D5 擴充性生態 | 15% | 4 | 3 | 2 |
| **加權總分（滿分 5.0）** | | **4.30** | **3.45** | **3.65** |

### 計算過程

- A：5×0.30 + 4×0.25 + 3×0.15 + 5×0.15 + 4×0.15 = 1.50+1.00+0.45+0.75+0.60 = **4.30**
- B：4×0.30 + 3×0.25 + 2×0.15 + 5×0.15 + 3×0.15 = 1.20+0.75+0.30+0.75+0.45 = **3.45**
- C：2×0.30 + 5×0.25 + 5×0.15 + 5×0.15 + 2×0.15 = 0.60+1.25+0.75+0.75+0.30 = **3.65**

### 裁定（依入選門檻）

| 工具 | 加權總分 | 裁定 |
|---|---|---|
| A Sysinternals | 4.30 | ≥4.0 → **直接入選**（進入模組三組合設計） |
| C TaskMgr/ResMon | 3.65 | 3.0–3.9 → 進入 POC 驗證（本案例為既有基準，免 POC，保留為 fallback） |
| B WPT | 3.45 | 3.0–3.9 → 進入 POC 驗證（列為冷備援候選，視需求啟動 POC） |

## 證據附錄（每格分數必須能指到至少一條證據）

| 工具 | 維度 | 分數 | 證據（量測方法＋日期＋連結） |
|---|---|---|---|
| A | D1=5 | 5 | 官方功能清單：Process Monitor（檔案／登錄／行程／網路／執行緒）、Process Explorer、Handle、RAMMap、VMMap、TCPView、ClockRes、Contig 共 8 項；對應 TSK-01/02/03 全覆蓋（2026-11-19 實測 Microsoft Store 套件頁） |
| A | D2=4 | 4 | Windows 原生（x64）；Process Monitor/Handle/Process Explorer 提供 CLI（`procmon /AcceptEula /Minimized /BackingFile ...`）；無正式 REST API；輸出 CSV/CSV (ProcMon) |
| A | D3=3 | 3 | TTFV **UNKNOWN**（未實測新人）；官方文件完整（Sysinternals docs + Mark's blog）；社群規模大（Microsoft 維護） |
| A | D4=5 | 5 | 免費（Microsoft Sysinternals License）；無授權費；維運近零（Store 自動更新） |
| A | D5=4 | 4 | CLI 腳本化能力存在；第三方腳本生態（PowerShell）；資料可匯出 CSV；供應商路線圖由 Microsoft 維護（透明度中） |
| B | D1=4 | 4 | WPT 提供 ETW 深度追蹤（CPU sampling、disk I/O、file I/O、registry），覆蓋 TSK-01/02；Handle 覆蓋弱（需另工具） |
| B | D2=3 | 3 | Windows 原生；CLI（xperf/wpr/wpa）；輸出 ETL 格式，需 WPA 視覺化；與現有工具鏈無官方整合 |
| B | D3=2 | 2 | TTFV **UNKNOWN**（預估 ≥1 週，ETW 概念陡峭）；文件完整但分散 |
| B | D4=5 | 5 | 免費（Windows SDK 內含） |
| B | D5=3 | 3 | 腳本化能力（wpr/wpa CLI）；生態較小；資料可匯出 CSV/ETL |
| C | D1=2 | 2 | 僅基本 CPU/記憶體/磁碟概覽；無檔案／登錄／控制代碼追蹤 |
| C | D2=5 | 5 | Windows 內建，零整合成本 |
| C | D3=5 | 5 | TTFV ≤ 10 分鐘（所有 Windows 使用者已熟悉） |
| C | D4=5 | 5 | 免費，零成本 |
| C | D5=2 | 2 | 無外掛／擴充 API；資料不可匯出（僅螢幕顯示） |

## 否決權檢核（一票否決，不計分）

| 檢核項 | A Sysinternals | B WPT | C TaskMgr/ResMon |
|---|---|---|---|
| 資安／合規硬性要求 | PASS（Microsoft 官方，數位簽章） | PASS | PASS |
| 授權法務風險 | PASS（免費，無商業限制） | PASS | PASS |
| 不可逆鎖定（lock-in） | PASS（資料可匯出 CSV，工具可移除） | PASS | PASS（基準工具，無鎖定） |

## 平手裁決

不適用（無平手）。

## 複審觸發

- 候選工具版本更新（Microsoft Store 自動更新）。
- FlapsErp 廠商端效能問題型態變更（如新增網路通訊瓶頸）。
- 每季治理節奏全量重審（§4.3）。

## 未量測項（UNKNOWN 清單）

- UNK-03：D3 TTFV（Sysinternals 與 WPT 均未實測新人，估算分數）。
- UNK-04：D2 實際串接測試（CLI 輸出格式與現有報告產生器的相容性未驗證）。
- UNK-05：D4 三年 TCO 之維運工時估算（目前以「近零」估算，未計工時）。
- UNK-06：D5 第三方外掛數量（未統計）。

## 證據（Evidence）

- Microsoft Store 套件頁（Sysinternals Suite）：2026-11-18 擷取。
- 官方文件：Sysinternals 官方文件（docs.microsoft.com/sysinternals）。
- WPT：Windows SDK 10/11 內含，官方文件。
- Task Manager/ResMon：Windows 內建，無需來源。
