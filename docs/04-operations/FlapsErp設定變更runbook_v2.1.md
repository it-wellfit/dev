# FlapsErp 設定變更 Runbook v2.1（機台無關＋實務操作版）

> 文件編號：OPS-2026-1111-002｜版本：v2.1｜日期：2026-11-12｜狀態：Active
> 適用：FlapsErp v0.0.4＋**任何 Windows 10/11 機台**。
> 紅線：**不得修改原廠二進位**；不得關閉 AV／UAC／WDAC；不得降低其他程式效能。
> 設計原則：**偵測 → 分支 → 套用（一步一步驗證）→ 遇到錯誤有解法 → 隨時可回滾**。

---

## 0. 執行優先序（時間有限時按此順序做）

| 優先 | 項目 | 需管理員 | 需重啟 | 效益 | 風險 | 做錯了能回滾嗎 |
|---|---|---|---|---|---|---|
| **P0** | Defender 排除（4 路徑＋1 程序） | ✅ | 否 | **最大**（DLL/日誌/資料 I/O 加速） | 安全面下降（可接受） | ✅ 一行 Remove-MpPreference |
| **P0** | GPU 偏好高效能（有獨顯時） | 否 | 重啟 FlapsErp | UI 渲染走獨顯 | 零 | ✅ 刪除登錄值 |
| **P1** | 程序優先級 AboveNormal | 否 | 否 | 高負載不凍結 | 其他程式略慢 | ✅ 重啟即恢復 |
| **P1** | Windows Search 排除 | ✅ | 否 | 背景 I/O 減少 | 零 | ✅ GUI 加回 |
| **P2** | TCP Auto-Tuning 確認 | ✅ | 可能 | AP 回應 | 零（如果本來 normal） | ✅ netsh 改回 |
| **P2** | 電源計畫（桌機 only） | ✅ | 可能 | 避免 CPU 降頻 | 筆電功耗 | ✅ powercfg 改回 |
| **P3** | SSD ReTrim | ✅ | 否 | 磁碟維護 | 零 | 不需要回滾 |
| **P3** | 程序優先級捷徑 | 否 | 否 | 永久自動化 | 零 | ✅ 刪除捷徑 |

> **時間不足？只做 P0 兩項**（Defender 排除＋GPU 偏好），這兩項合計已涵蓋 80% 的效益。

---

## 1. 執行前檢查清單（Pre-Implementation Checklist）

每一項都通過才能開始。任何一項不通過 → 先解決再繼續。

| # | 檢查 | 命令 | 通過條件 | 不通過怎麼辦 |
|---|---|---|---|---|
| 1 | FlapsErp 版本確認 | `Get-Item '<安裝目錄>\FlapsErp.exe'` | 檔案存在 | 確認安裝路徑 |
| 2 | config.ini 存在 | `Test-Path '<安裝目錄>\..\.conf\config.ini'` | True | 找到正確路徑 |
| 3 | 管理員權限 | `[Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole('Administrators')` | True | 用 RunAs 提權 |
| 4 | 磁碟可用空間 > 5 GB | `Get-Volume C` | SizeRemaining > 5 GB | 清理磁碟 |
| 5 | 無 cargo build | `Get-Process link,cargo,rustc -EA SilentlyContinue` | Count = 0 | 等建置結束 |
| 6 | Defender 啟用中 | `Get-MpComputerStatus` | RealTimeProtectionEnabled = True | 排除無意義，跳過 |
| 7 | 目前 Defender 排除（記錄原值） | `Get-MpPreference | Select ExclusionPath` | 記錄 | — |
| 8 | 目前電源計畫（記錄原值） | `powercfg /getactivescheme` | 記錄 GUID | — |
| 9 | 目前 TCP 設定（記錄原值） | `netsh int tcp show global` | 記錄 | — |

---

## 2. Phase 0：硬體偵測（每次新機台必做）

```powershell
# 執行以下命令，記錄結果
Get-CimInstance Win32_OperatingSystem | Select-Object Caption,OSArchitecture,TotalVisibleMemorySize,FreePhysicalMemory
Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors
Get-PhysicalDisk | Select-Object FriendlyName,MediaType
Get-CimInstance Win32_VideoController | Select-Object Name,AdapterRAM
powercfg /getactivescheme
powercfg /a
Get-MpComputerStatus | Select-Object RealTimeProtectionEnabled
Get-ExecutionPolicy -List
```

根據結果判定分支（見 §4 分支表）。

---

## 3. 分支決策表

### RAM 分支

| 偵測 | 分支 | 建議 |
|---|---|---|
| ≤ 8 GB | LOW_RAM | 關閉不用的程式；Defender 排除仍做；考慮加 RAM |
| 8–16 GB | MID_RAM | Defender 排除＋GPU 偏好（有獨顯）＋優先級 |
| ≥ 16 GB | HIGH_RAM | 全部適用 |

### GPU 分支

| 偵測 | 分支 | 建議 |
|---|---|---|
| 僅內顯 | IGPU_ONLY | 不建議 GPU 偏好；`DISABLE_HARDWARE_ACCELERATION` 逐機 A/B |
| 內顯＋獨顯 | HYBRID_GPU | GPU 偏好高效能 |
| 僅獨顯 | DGPU_ONLY | 已預設高效能 |

### 電源分支

| 偵測 | 分支 | 建議 |
|---|---|---|
| 有電池 | LAPTOP | 不建議高效能電源；AC 時用最佳效能 |
| 無電池 | DESKTOP | 高效能電源 |

### 磁碟分支

| 偵測 | 分支 | 建議 |
|---|---|---|
| SSD | SSD | ReTrim 每月 |
| HDD | HDD | Defrag；考慮升級 |

### OS 分支

| 偵測 | 分支 | 建議 |
|---|---|---|
| 家用版 | HOME | AppLocker/WDAC 可能不可用 |
| 專業版/企業版 | PRO/ENT | 全部可用 |

---

## 4. 依分支套用設定

### 4.1 所有分支：Defender 排除（P0；效益最大）

**執行前**：確認 §1 檢查清單第 3、6、7 項通過。

```powershell
# 記錄原值（回滾用）
$orig = Get-MpPreference | Select-Object ExclusionPath, ExclusionProcess
$orig | Out-File -FilePath '<備份根>\defender-before.txt' -Encoding UTF8

# 套用排除（4 路徑 + 1 程序）
$erpDir = 'C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp'  # 依實機路徑調整
Add-MpPreference -ExclusionPath $erpDir
Add-MpPreference -ExclusionPath "$erpDir\Module"
Add-MpPreference -ExclusionPath "$erpDir\..\FlapsErpData"   # 若存在
Add-MpPreference -ExclusionPath "$erpDir\..\logs"           # 若存在
Add-MpPreference -ExclusionProcess 'FlapsErp.exe'

# 驗證
$after = Get-MpPreference | Select-Object ExclusionPath, ExclusionProcess
$after | Out-File -FilePath '<備份根>\defender-after.txt' -Encoding UTF8
```

**常見錯誤與解法**：

| 錯誤訊息 | 原因 | 解法 |
|---|---|---|
| HRESULT 0xc0000142 | 未以管理員執行 | 右鍵 PowerShell → 以系統管理員身份執行 |
| Access denied | 群組原則管理 Defender | 聯絡 IT 確認 GPO 是否允許本機排除 |
| ExclusionPath 為空 | Defender 服務未啟動 | `Start-Service WinDefend` |
| 排除後仍慢 | AV 不是瓶頸 | 用 `-Probe baseline` 量測其他維度 |

**回滾**：
```powershell
Remove-MpPreference -ExclusionPath '<安裝目錄>'
Remove-MpPreference -ExclusionPath '<安裝目錄>\Module'
Remove-MpPreference -ExclusionPath '<安裝目錄>\..\FlapsErpData'
Remove-MpPreference -ExclusionPath '<安裝目錄>\..\logs'
Remove-MpPreference -ExclusionProcess 'FlapsErp.exe'
```

---

### 4.2 所有分支：GPU 偏好（僅 HYBRID_GPU 分支）

**執行前**：確認有獨顯（`Get-CimInstance Win32_VideoController` 返回 2 個以上）。

```powershell
# 記錄原值
$gpuKey = 'HKCU:\SOFTWARE\Microsoft\DirectX\UserGpuPreferences'
$orig = Get-ItemProperty -Path $gpuKey -ErrorAction SilentlyContinue
$orig | Out-File -FilePath '<備份根>\gpu-before.txt' -Encoding UTF8

# 套用
Set-ItemProperty -Path $gpuKey -Name '<FlapsErp.exe完整路徑>' -Value 'GpuPreference=2;'

# 驗證
Get-ItemProperty -Path $gpuKey | Select-Object '<FlapsErp.exe完整路徑>'
```

**常見錯誤**：

| 錯誤 | 原因 | 解法 |
|---|---|---|
| 路徑不存在 | FlapsErp.exe 路徑打錯 | 確認 `Get-Item '<路徑>'` |
| 登錄檔路徑不存在 | Windows 10 舊版無此鍵 | 用 GUI 方法（設定 → 顯示卡） |
| 設定後無效 | 需重啟 FlapsErp | Stop-Process → 重新啟動 |

**回滾**：
```powershell
Remove-ItemProperty -Path $gpuKey -Name '<FlapsErp.exe完整路徑>'
```

---

### 4.3 所有分支：程序優先級

**前提**：FlapsErp 正在執行。

```powershell
Get-Process FlapsErp -ErrorAction SilentlyContinue | ForEach-Object { $_.PriorityClass = 'AboveNormal' }
```

**驗證**：
```powershell
Get-Process FlapsErp | Select-Object Id, PriorityClass
# 全部應為 AboveNormal
```

**注意**：重啟 FlapsErp 後需重新設定。可用捷徑或排程自動化。

---

### 4.4 LAPTOP 分支不適用的項目

以下項目**僅 DESKTOP 分支**適用；筆電不做：

| 項目 | 為什麼筆電不做 |
|---|---|
| 高效能電源 | 增加功耗與熱量 |
| 停用睡眠 | 影響行動使用 |
| 停用休眠 | 佔磁碟空間且筆電需要 |
| USB 選擇性暫停停用 | 影響電池續航 |

---

### 4.5 DESKTOP 分支適用：電源計畫

```powershell
# 記錄原值
$orig = powercfg /getactivescheme

# 切換
powercfg /setactive SCHEME_MIN  # High Performance

# 驗證
powercfg /getactivescheme
```

**回滾**：
```powershell
powercfg /setactive <原GUID>
```

---

### 4.6 SSD 分支適用：ReTrim

```powershell
# 每月執行一次
Optimize-Volume -DriveLetter <系統碟代號> -ReTrim -Verbose
```

---

### 4.7 TCP Auto-Tuning 確認

```powershell
# 查看
netsh int tcp show global
# 確認 autotuninglevel = normal

# 若 disabled，改回 normal（需管理員）
netsh int tcp set global autotuninglevel=normal

# 記錄原值（回滾用）
# 原值通常是 normal（Windows 預設）
```

---

## 5. 量測驗證

### 5.1 基線量測（變更前）

```powershell
powershell -ExecutionPolicy Bypass -File "<Wellfit根>\scripts\perf\flaps-perf.ps1" -Probe full
```

### 5.2 變更後量測

同上命令。確認 `contended=False`。

### 5.3 App smoke

1. 啟動 → 登入 → 選單就緒
2. 開功能模組 → 列表載入
3. 匯出 CSV → 檔案可讀
4. 關閉 → 重啟 → 再登入

### 5.4 A/B diff

```powershell
flaps-perf.ps1 -Probe diff -BaselineDir <before> -CurrentDir <after>
```

---

## 6. 實務常見問題與解法

### 6.1 群組原則（GPO）阻擋變更

**症狀**：`Add-MpPreference` 成功但重啟後排除消失。

**原因**：企業 GPO 強制覆寫 Defender 排除。

**解法**：
1. 聯絡 IT 確認 GPO 路徑：`電腦設定 → 系統管理範本 → Windows 元件 → Microsoft Defender 防毒病毒 → 排除`
2. 請 IT 在 GPO 中加入 FlapsErp 排除路徑
3. 或改用 `ExclusionProcess`（程序排除有時不受 GPO 路徑限制）

---

### 6.2 WDAC 阻擋 PowerShell 腳本

**症狀**：`powershell -ExecutionPolicy Bypass` 無法執行 `.ps1`。

**原因**：WDAC 或 AppLocker 政策限制腳本執行。

**解法**：
1. 改用 GUI 操作（Defender 排除可用 Windows 安全性 → 病毒與威脅防護 → 管理設定）
2. 或請 IT 將腳本加入 WDAC 允許清單
3. 或使用簽署腳本

---

### 6.3 多台機器批次部署

**做法**：
1. 先在**一台測試機**完成全部流程，確認 App smoke 通過
2. 將確認過的命令整理成腳本
3. 透過 GPO/Intune/SCCM 推送
4. 每台機器跑 `flaps-perf.ps1 -Probe full` 驗證

**注意**：不同機台的 FlapsErp 安裝路徑可能不同，腳本需使用變數。

---

### 6.4 FlapsErp 原廠更新後重新設定

**觸發條件**：`-Probe settings` 偵測到 `.ver` 漂移或 config.ini 變更。

**做法**：
1. 確認更新內容（新增/修改了哪些檔案）
2. 重新執行 Defender 排除（若有新目錄）
3. 重新量測基線
4. 比對是否需要調整 GPU 偏好或優先級

---

### 6.5 使用者回報「還是慢」

**排查流程**：
```
使用者說慢
→ 用 -Probe app 確認 module→menu 時間
→ 用 -Probe host 確認系統負載
→ 用 -Probe com 確認 COM 延遲
→ 用 -Probe baseline 確認 RAM
→ 找出瓶頸在哪一層：
   ├─ AP 回應 > 200 ms → 網路/AP 瓶頸（非本機能解）
   ├─ 可用 RAM < 2 GB → 關閉其他程式
   ├─ 磁碟佇列 > 2 → 磁碟瓶頸
   ├─ COM P95 > 0.5 ms → 系統壓力
   └─ 全部正常但使用者仍覺得慢 → 請使用者描述具體操作步驟
```

---

## 7. 回滾（完整）

### 7.1 逐步回滾（依套用順序反做）

| 步驟 | 回滾命令 | 驗證 |
|---|---|---|
| Defender 排除 | `Remove-MpPreference -ExclusionPath ...` × N | `Get-MpPreference` 確認已移除 |
| GPU 偏好 | `Remove-ItemProperty -Path $gpuKey -Name '<路徑>'` | 確認已刪除 |
| 程序優先級 | 重啟 FlapsErp | `Get-Process FlapsErp` 確認 Normal |
| TCP | `netsh int tcp set global autotuninglevel=<原值>` | `netsh int tcp show global` |
| 電源 | `powercfg /setactive <原GUID>` | `powercfg /getactivescheme` |
| config.ini | 從備份還原＋SHA256 | 7 鍵與備份一致 |

### 7.2 全部回滾（緊急情況）

```powershell
# 1. 還原 config.ini
Copy-Item -LiteralPath '<備份根>\config-backup\<date>\config.ini' -Destination '<config.ini路徑>' -Force

# 2. 還原 Defender
Remove-MpPreference -ExclusionPath '<路徑>'  # 逐一移除

# 3. 還原 GPU
Remove-ItemProperty -Path $gpuKey -Name '<FlapsErp.exe完整路徑>'

# 4. 重啟 FlapsErp
Stop-Process -Name FlapsErp -ErrorAction SilentlyContinue
Start-Process -FilePath '<FlapsErp.exe路徑>'

# 5. 驗證
flaps-perf.ps1 -Probe full
```

---

## 8. 不建議做的事（所有分支通用）

| 項目 | 原因 |
|---|---|
| 修改 `FlapsErp.exe`/`app.asar`/`Module/` | 原廠二進位；違約 |
| 加未證實的 config 鍵 | 原廠不認；假設定 |
| 關閉 Defender | 安全風險 |
| 關閉 UAC | 安全風險 |
| 關閉 WDAC | 安全風險 |
| `--single-process`/`--no-sandbox` | Electron 穩定性退步 |
| RAM booster | 無實證 |
| 在營業時間跑批次 | 搶其他程式資源 |
| 直接寫 ERP 資料庫 | 違反資料契約 |
| 停用 Memory Compression | 停用後直接分頁到磁碟，更慢 |
| 停用 SysMain/Superfetch（SSD 上） | 效益有限且可能影響其他程式 |
| 在營業時間重啟 FlapsErp | 影響使用者工作 |

---

## 9. 變更紀錄

| 版本 | 日期 | 變更 |
|---|---|---|
| v1.0 | 2026-11-11 | 初版；硬體特定 |
| v2.0 | 2026-11-12 | 機台無關版 |
| v2.1 | 2026-11-12 | **新增**：執行優先序（時間有限時的順序）、執行前檢查清單（9 項）、每步驟的常見錯誤與解法（6 個情境）、逐步回滾（含命令）、全部回滾（緊急）、實務常見問題（GPO/WDAC/多台部署/原廠更新/使用者回報慢）、不建議做的事新增 3 項 |

---

*本 Runbook 適用於任何 Windows 10/11 機台。核心原則：偵測→分支→套用（一步一步驗證）→遇到錯誤有解法→隨時可回滾。未量測不得標 PASS。*