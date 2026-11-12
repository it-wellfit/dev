# FlapsErp 最佳設定基線 v2.0（機台無關版）

> 文件編號：OPS-2026-1111-001｜版本：v2.0｜日期：2026-11-12｜狀態：Active
> 適用：FlapsErp v0.0.4＋**任何 Windows 10/11 機台**。
> 紅線：**不得修改原廠二進位**；不得關閉 AV／UAC／WDAC。
> 設計原則：**先偵測 → 分支 → 量測 → 套用**；不假設任何硬體規格。

---

## 1. 產品層設定（`.conf\config.ini`）

實機路徑：`C:\SYSTEM_PROGRAM_DIRECTORY\.conf\config.ini`（或依部署而定）。

7 鍵中**只有 1 鍵與效能直接相關**，其餘 6 鍵為授權/除錯/語言/UI，**不需要修改**。

| 鍵 | 預設值 | 建議 | 說明 |
|---|---|---|---|
| `IGNORE_CERT_ERR` | 1 | 維持 1（條件式→0） | 預設 AP 若為 HTTP 則此鍵無作用；若 AP 轉 HTTPS 且憑證可信後可改 0 |
| `KEEP_MACHINE_ID` | 1 | 維持 | 授權綁定 |
| `DISABLE_HARDWARE_ACCELERATION` | 0 | 維持 0（**需 A/B**） | 實測 `=1` 在部分機台 startup 更慢；部分內顯機台 `=1` 反而穩定；需逐機 A/B |
| `DEBUG_FASTREPORT` | 0 | 維持 | 報表除錯用 |
| `FORCE_DEVICE_SACLE_FACTOR` | 0 | 維持 | UI 縮放（原廠拼字）；僅 UI 裁切時才評估 |
| `LANG` | zh_TW | 維持 | 語言 |
| `KEEP_MACHINE_ID_MIGRATED` | 1 | 維持 | 授權遷移 |

> **結論：config.ini 沒有任何一鍵可單獨帶來多維度效能改善。** 效能改善來自 Windows 系統層（§3）與操作面（§5），不是 config 值。

---

## 2. AP／網路／TLS（原廠）

| 項目 | 建議 |
|---|---|
| 預設 AP | 維持原廠設定；不變更 |
| AP 轉 HTTPS | 向原廠提出需求；過渡期受控 LAN |
| TLS 1.0/1.1 | 不主動停用（舊 OpenSSL 相依） |
| 防火牆 | 最小白名單（目的 IP/Port/Protocol/方向） |
| 網路延遲 | `Measure-Command` 量測 AP 回應；> 200 ms 為網路瓶頸 |

---

## 3. Windows 宿主層（機台無關；先偵測→分支→套用）

### 3.1 硬體偵測分支

| 維度 | 分支 | 適用建議 |
|---|---|---|
| RAM ≤ 8 GB | LOW_RAM | 關閉不用的其他程式；不建議排除（效益有限）；考慮加 RAM |
| RAM 8–16 GB | MID_RAM | Defender 排除 + GPU 偏好 + 優先級 |
| RAM ≥ 16 GB | HIGH_RAM | Defender 排除 + GPU 偏好 + 優先級 + Search 排除 |
| 僅內顯 | IGPU_ONLY | 不建議 GPU 偏好；`DISABLE_HARDWARE_ACCELERATION` 需逐機 A/B |
| 內顯+獨顯 | HYBRID_GPU | GPU 偏好設高效能 |
| 僅獨顯 | DGPU_ONLY | GPU 已預設高效能 |
| 有電池 | LAPTOP | 不建議高效能電源；不建議停用睡眠 |
| 無電池 | DESKTOP | 高效能電源；可停用睡眠 |
| SSD/NVMe | SSD | 定期 ReTrim；不停用 Last Access |
| HDD | HDD | 定期 Defrag；考慮升級 SSD |
| Windows 家用版 | HOME | AppLocker/WDAC 可能不可用 |
| 專業版/企業版 | PRO/ENT | 全部可用 |

### 3.2 所有分支通用（安全、零風險、效益最大）

| 項目 | 命令（管理員） | 預期改善 |
|---|---|---|
| **Defender 最小排除** | `Add-MpPreference -ExclusionPath '<安裝目錄>'` 等 | 啟動 -10~30%；操作 -5~15% |
| **Search 排除** | GUI：進階索引選項 → 取消勾選 | 背景 I/O -5~10% |
| **TCP Auto-Tuning 確認** | `netsh int tcp show global`（normal 即可） | 若 disabled → AP 回應 -20~50% |
| **程序優先級** | `Get-Process FlapsErp | ForEach { $_.PriorityClass = 'AboveNormal' }` | 高負載凍結 -30~50% |
| **AP 延遲量測** | `Measure-Command { Invoke-WebRequest ... }` | 建立網路基準 |

### 3.3 HYBRID_GPU 分支適用

| 項目 | 命令 | 預期改善 |
|---|---|---|
| GPU 偏好（高效能） | 設定 → 顯示卡 → 選 FlapsErp.exe → 高效能 | UI 渲染 -10~20% |

### 3.4 DESKTOP 分支適用

| 項目 | 命令 |
|---|---|
| 高效能電源 | `powercfg /setactive SCHEME_MIN` |
| 停用睡眠 | `powercfg /change standby-timeout-ac 0` |

### 3.5 SSD 分支適用

| 項目 | 命令 |
|---|---|
| ReTrim | `Optimize-Volume -DriveLetter <系統碟> -ReTrim`（每月） |

### 3.6 不建議做的事（所有分支）

| 項目 | 原因 |
|---|---|
| 修改原廠二進位 | 違約 |
| 關閉 Defender/UAC/WDAC | 安全 |
| `--single-process`/`--no-sandbox` | 穩定性退步 |
| RAM booster | 無實證 |
| 高效能電源（筆電分支） | 功耗/熱 |
| `DISABLE_HARDWARE_ACCELERATION=1`（未 A/B 前） | 部分機台更慢 |

---

## 4. 啟動／分頁／程序衛生（所有分支通用；FlapsErp-only）

| 項目 | 做法 |
|---|---|
| 關閉不用的分頁 | 每個分頁 = 一個 renderer 進程（40–100 MB） |
| 單一實例 | 啟動前檢查 `Get-Process FlapsErp`；不自動 kill |
| 關閉多餘 readonly tabs | 收工前檢查 |
| DLL 註冊 | 一次性；保持 `.ver` 穩定 |
| 夜間重啟 | 條件式；若 8h soak 無洩漏則不啟用 |

---

## 5. 外部工作台（自建 Wellfit；所有分支通用）

| 工作台 | 說明 |
|---|---|
| A1 檔案通道 | `flapserp://` 深層連結；CSV 匯入/匯出 |
| Wellfit 批次 | 清洗/驗證/修復在 Wellfit CLI；ERP 只確認/匯入 |
| 報表外移 | 匯出成檔 → Wellfit 解析 |
| 批次守門 | 離峰；低優先；可中止；不與 ERP 爭資源 |

---

## 6. 監控（所有分支通用）

| 頻率 | 工作 |
|---|---|
| 每日 | `flaps-perf.ps1 -Probe all`；app log 解析 |
| 每週 | `-Probe com` |
| 每月/更新後 | `-Probe cdp`；`-Probe settings`；`-Probe pe` |
| 每季 | 漂移比對 |
| 每次變更 | `-Probe full`；`-Probe diff` |
| 每次 A/B | 確認 `contended=False` |

---

## 7. 驗收（所有分支通用）

| 碼 | 項目 | 條件 |
|---|---|---|
| AC-S01 | 7 鍵一致 | `-Probe settings` |
| AC-S02 | Defender 排除有證據 | `Get-MpPreference` 前後對照 |
| AC-S03 | GPU 偏好有證據 | 主觀 + baseline |
| AC-S04 | 程序優先級有證據 | 高負載時操作 |
| AC-S05 | 啟動改善 | ≥ 10% |
| AC-S06 | 記憶體改善 | ≥ 10% |
| AC-S07 | 無退步 | 所有指標 ≤ 基線 +10% |
| AC-S08 | 8h 穩定 | RAM 增長 < 50 MB；無崩潰 |
| AC-S09 | App smoke | 全過 |
| AC-S10 | 外部工作台 | 端到端 |
| AC-S11 | 系統級不退化 | 其他程式 ≤ 基線 +10% |

---

## 8. 回滾

| 變更 | 回滾 |
|---|---|
| Defender 排除 | `Remove-MpPreference` |
| GPU 偏好 | 刪除登錄值 |
| 優先級 | 重啟即恢復 |
| 電源 | `powercfg /setactive <原GUID>` |
| config | 從備份還原 |

---

## 9. 變更紀錄

| 版本 | 日期 | 變更 |
|---|---|---|
| v1.0 | 2026-11-11 | 初版；含本機特定值 |
| v2.0 | 2026-11-12 | **重寫為機台無關版**：移除硬體特定值；新增硬體偵測分支表（RAM/GPU/電源/磁碟/OS）；所有建議改為「偵測 → 分支 → 套用」 |

---

*本文件適用於任何 Windows 10/11 機台；不含硬體特定值；所有建議以硬體偵測結果為前提。*