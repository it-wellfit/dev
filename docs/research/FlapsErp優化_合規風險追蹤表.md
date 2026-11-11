# FlapsErp 優化 — 合規風險追蹤表 v1.0

> 文件編號：RSH-2026-1103-003｜日期：2026-11-03｜狀態：Active｜複審頻率：每週五（隨狀態帳本）
> 評分：機率(1-5) × 影響(1-5)；≥15 極高、10–14 高、5–9 中、≤4 低（MOP §16.2）
> 紅線：**任何變更不得寫入 `C:\SYSTEM_PROGRAM_DIRECTORY`（原廠元件區），唯讀觀測除外。**

| 風險 ID | 風險描述 | 機率 | 影響 | 分數 | 等級 | 緩解措施 | 觸發條件 | 負責 | 狀態 |
|---|---|---|---|---|---|---|---|---|---|
| RSK-E1 | **EULA/授權邊界未明**：以 COM/CDP/RDA 自動化輔翼ERP 可能超出原廠授權範圍 | 3 | 5 | 15 | 極高 | OQ-F1 正式索取自動化政策書面回覆；取得前僅用官方入口（flapserp://、檔案交換）；COM/RDA 僅 PoC 不上生產 | 原廠明示禁止或無回覆逾 14 天 | PMT | Open（Phase 1 提出） |
| RSK-E2 | **憑證忽略／AP 明文**：`IGNORE_CERT_ERR=1`；預設生產 AP 為 **HTTP 明文** ✔；測試 AP TLS CERT-VALID ✔ | 3 | 5 | 15 | 極高 | 要求原廠 AP 轉 HTTPS＋正式憑證；過渡期受控 LAN/白名單；`IGNORE_CERT_ERR=0` 僅在 HTTPS 端點盤點完成後；我方 Wellfit 強制 TLS 驗證 | 任何對外 AP 連線 | PMT/IT | Open |
| RSK-E3 | **32-bit LAA 4 GB 上限（實測 LAA=True）**：大報表/大批次仍可能觸及 4 GB | 4 | 3 | 12 | 高 | 批次外移 Wellfit（獨立處理序）；A 線要求 64-bit；監控 RAM；分頁/程序衛生 | RAM 峰值 > 3 GB 或 GC 停頓 | DEV | Open |
| RSK-E4 | **元件漂移**：原廠活躍更新（Winax.node 2026-11-03 更新 ✔），COM/RDA 介面可能變動 | 3 | 4 | 12 | 高 | flaps-bridge 內建 `.ver` 偵測 + 契約測試；變動即告警停止；.ver 快照比對 | 任何 .ver 雜湊變化 | DEV | Open（Phase 3 生效） |
| RSK-E5 | **誤寫原廠元件區**：自動化流程意外寫入 `C:\SYSTEM_PROGRAM_DIRECTORY` | 2 | 5 | 10 | 高 | 紅線入所有腳本頭註；腳本一律唯讀開啟該目錄；寫入僅允許 `%TEMP%\flaps` 與我方輸出目錄 | 任何寫入嘗試路徑檢查失敗 | DEV | Open（常駐） |
| RSK-E6 | **同步 IPC 凍結擴散**：COM sendSync 使 ERP UI 凍結，自動化高峰期影響使用者作業 ✔ | 4 | 3 | 12 | 高 | 我方批次呼叫集中於 Wellfit 側（flaps-bridge 批次化）；避開營業高峰時段排程（02:00–06:00 或週末）；基線量測凍結事件 | 使用者回報凍結或量測 > 1s 事件 | DEV | Open（Phase 1 量測） |
| RSK-E7 | **UAC 提權摩擦**：DLL 註冊流程 regsvr32 -Verb RunAs + Set-ExecutionPolicy Unrestricted ✔ | 3 | 3 | 9 | 中 | 不干預原廠更新流程；我方腳本不採用 Unrestricted；IT 以一次性安裝/註冊處理；記錄發生頻率 | 註冊失敗或重複 UAC 彈窗 | OPS | Open |
| RSK-E9 | **提案無回應**：OQ-F1/F2 提案包送出後原廠無回覆，自動化政策與資料契約缺口持續 | 3 | 4 | 12 | 高 | 提案包已送出（2026-11-03，回覆期限 2026-11-19）；14 天無回覆啟動升級（RSK-E1 緩解）；我方軌道（A1/A2/A3）不依賴原廠回覆 | 2026-11-19 無回覆 | PMT | Open（Phase 1 提出） |
| RSK-E10 | **預設 AP HTTP 明文**：帳密/資料可能明文 | 4 | 5 | 20 | 極高 | 受控 LAN；要求原廠 HTTPS＋正式憑證；網段隔離；不以關閉驗證取代 | 任何 AP 連線 | PMT/IT | Open |
| RSK-S1 | **為 FlapsErp 優化而拖垮其他程式** | 3 | 5 | 15 | 極高 | FlapsErp-only 優先；系統級 A/B；批次守門；另機/離峰 | 其他程式退步 >10% | DEV/IT | Open |
| RSK-S2 | **Wellfit 批次與其他程式搶資源** | 4 | 4 | 16 | 極高 | 離峰/BelowNormal/CPU/RAM 上限/可中止；優先另機 | 批次期間系統 CPU>80% 或 RAM>85% | DEV/IT | Open |
| RSK-S3 | **系統級設定（電源/驅動/分頁檔）影響全機** | 3 | 3 | 9 | 中 | 條件式；IT 核准；系統級 A/B；可回滾 | 任何 S/C 變更 | IT | Open |
| RSK-S4 | **熱降頻** | 3 | 4 | 12 | 高 | 溫度監控；>85°C 收回高效能；桌機/RPA 站 | 溫度 >85°C 或降頻 | IT | Open |
| RSK-S5 | **記憶體壓力/pagefile 抖動** | 3 | 4 | 12 | 高 | 分頁衛生；RAM 監控；>85% 延後批次 | 系統 RAM >85% 5 min | IT | Open |
| RSK-S6 | **批次/報表/備份 I/O 拖慢其他程式** | 3 | 4 | 12 | 高 | 離峰/低優先/限制；磁碟佇列守門 | 佇列 >2.0 5 min | IT | Open |

## 變更紀錄

| 版本 | 日期 | 變更 |
|---|---|---|
| v1.1 | 2026-11-03 | 新增 RSK-E9（提案無回應）；更新 RSK-E1 關聯提案包 PROP-2026-1103-001 |
| v1.2 | 2026-11-11 | 新增 RSK-E10（AP HTTP 明文）、RSK-S1..S6（系統級互動）；更新 RSK-E2/E3/E7；實測 LAA=True、測試 AP CERT-VALID |
