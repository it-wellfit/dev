# FlapsErp 設定變更 Runbook v2.0（機台無關版）

> 文件編號：OPS-2026-1111-002｜版本：v2.0｜日期：2026-11-12｜狀態：Active
> 適用：FlapsErp v0.0.4＋**任何 Windows 10/11 機台**（不限 CPU/GPU/RAM/筆電桌機）。
> 紅線：**不得修改原廠二進位**；不得關閉 AV／UAC／WDAC；不得降低其他程式效能。
> 設計原則：**先偵測硬體 → 分支建議 → 量測驗證 → 才套用**。不假設任何硬體規格。

---

## 1. 適用範圍與硬體分支

| 元件 | 平台限制 | 說明 |
|---|---|---|
| FlapsErp | Windows only | COM/ActiveX/DLL；無 macOS/Linux |
| 本 Runbook | Windows 10/11 | 家用版/專業版/企業版均適用 |
| 硬體 | 不限 | 筆電/桌機/一體機；Intel/AMD/任何 GPU |

---

## 2. Phase 0：硬體偵測（每次新機台必做）

執行以下命令，記錄結果到 `Wellfit/artifacts/host-inventory/<ts>/host.json`。

```powershell
# === 系統 ===
Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,BuildNumber,OSArchitecture,TotalVisibleMemorySize,FreePhysicalMemory
Get-CimInstance Win32_ComputerSystem | Select-Object Manufacturer,Model,PCSystemType

# === CPU ===
Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors,MaxClockSpeed

# === RAM ===
Get-CimInstance Win32_PhysicalMemory | Select-Object Capacity,Speed,Manufacturer

# === 磁碟 ===
Get-PhysicalDisk | Select-Object FriendlyName,MediaType,Size,HealthStatus
Get-Volume | Select-Object DriveLetter,FileSystemLabel,Size,SizeRemaining

# === GPU ===
Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion,AdapterRAM,Status

# === 網路 ===
Get-NetAdapter | Where-Object Status -eq 'Up' | Select-Object Name,LinkSpeed,MediaType

# === 電源（判斷筆電/桌機） ===
powercfg /getactivescheme
powercfg /a

# === 防毒狀態 ===
Get-MpComputerStatus | Select-Object AMServiceEnabled,RealTimeProtectionEnabled,AntivirusSignatureLastUpdated

# === UAC / WDAC ===
Get-ExecutionPolicy -List
Get-AppLockerPolicy -Effective -ErrorAction SilentlyContinue

# === FlapsErp ===
Get-Process FlapsErp -ErrorAction SilentlyContinue
Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion" | Select-Object DisplayVersion
reg query "HKCR\flapserp\shell\open\command" 2>$null
```

---

## 3. Phase 1：硬體分支決策表

根據 Phase 0 偵測結果，選擇適用的建議分支。

### 3.1 RAM 分支

| 偵測結果 | 分支 | 建議 |
|---|---|---|
| ≤ 8 GB | **LOW_RAM** | 關閉不用的其他程式；不跑高記憶體程式與 FlapsErp 同時；考慮加 RAM |
| 8–16 GB | **MID_RAM** | FlapsErp + 2–3 個程式可並行；Defender 排除仍有幫助 |
| ≥ 16 GB | **HIGH_RAM** | FlapsErp + 多程式可並行；Defender 排除 + GPU 偏好 + 優先級 |

### 3.2 GPU 分支

| 偵測結果 | 分支 | 建議 |
|---|---|---|
| 僅內顯（Intel UHD/Iris、AMD Radeon Graphics） | **IGPU_ONLY** | 不建議改 GPU 偏好；`DISABLE_HARDWARE_ACCELERATION` 需 A/B（部分內顯 `=1` 反而穩定） |
| 內顯 + 獨顯（hybrid） | **HYBRID_GPU** | 建議設 FlapsErp 為高效能 GPU（Windows 設定 → 顯示卡 → 瀏覽 → 選 FlapsErp.exe → 高效能） |
| 僅獨顯 | **DGPU_ONLY** | GPU 偏好已預設高效能；不需改 |

### 3.3 電源分支

| 偵測結果 | 分支 | 建議 |
|---|---|---|
| `powercfg /a` 有 S3/S4（有電池） | **LAPTOP** | 不建議高效能電源；不建議停用睡眠/休眠；建議 AC 時用「最佳效能」，DC 用「平衡」 |
| 無電池（桌機/一體機） | **DESKTOP** | 建議「最佳效能」電源；可停用睡眠；仍不停用 Defender/UAC |

### 3.4 磁碟分支

| 偵測結果 | 分支 | 建議 |
|---|---|---|
| SSD/NVMe | **SSD** | 不需要 Defrag；定期 `Optimize-Volume -ReTrim`；NTFS Last Access 保持預設 |
| HDD | **HDD** | 定期 Defrag；考慮升級 SSD（效益最大的硬體投資） |

### 3.5 作業系統分支

| 偵測結果 | 分支 | 建議 |
|---|---|---|
| Windows 11 家用版 | **HOME** | AppLocker/WDAC 可能不可用；Defender 排除可用 |
| Windows 10/11 專業版/企業版 | **PRO/ENT** | 全部功能可用 |

---

## 4. Phase 2：依分支套用設定（僅套用適用分支的項目）

### 4.1 所有分支通用（安全、零風險）

#### 4.1.1 Defender 最小排除（效益最大）

```powershell
# 需管理員；不關閉 Defender；只排除原廠已安裝的路徑
Add-MpPreference -ExclusionPath '<FlapsErp安裝目錄>'
Add-MpPreference -ExclusionPath '<FlapsErp安裝目錄>\Module'
Add-MpPreference -ExclusionPath '<FlapsErp安裝目錄>\..\FlapsErpData'  # 若存在
Add-MpPreference -ExclusionProcess 'FlapsErp.exe'
```

> `<FlapsErp安裝目錄>` 依實機路徑替換，例如 `C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp`。

**前提**：先確認 AV 即時掃描確實是瓶頸（用 `-Probe baseline` 量測啟動時間前後對照）；IT 資安核准；記錄核准人與期限。

#### 4.1.2 Windows Search 排除

透過 GUI：設定 → 搜尋 Windows → 進階索引選項 → 修改 → 取消勾選 FlapsErp 安裝目錄。

#### 4.1.3 TCP Auto-Tuning 確認

```powershell
netsh int tcp show global
# 確認 autotuninglevel = normal（非 disabled）
# 若 disabled：netsh int tcp set global autotuninglevel=normal
```

#### 4.1.4 AP 網路延遲量測

```powershell
# 量測 AP 登入頁回應時間（建立網路基準）
Measure-Command { Invoke-WebRequest -Uri "<AP_URL>/login/index.jsp" -UseBasicParsing -TimeoutSec 10 }
```

#### 4.1.5 程序優先級（FlapsErp 啟動後執行）

```powershell
Get-Process FlapsErp -ErrorAction SilentlyContinue | ForEach-Object { $_.PriorityClass = 'AboveNormal' }
```

> **風險**：其他程式在 CPU 滿載時略慢。適度（AboveNormal，非 High/Realtime）。

---

### 4.2 HYBRID_GPU 分支適用（有獨顯的機台）

#### 4.2.1 GPU 偏好設定

```powershell
# 方法 A：透過 Windows 設定
# 設定 → 系統 → 顯示器 → 顯示卡 → 瀏覽 → 選 FlapsErp.exe → 高效能

# 方法 B：透過登錄檔
$gpuKey = "HKCU\SOFTWARE\Microsoft\DirectX\UserGpuPreferences"
Set-ItemProperty -Path $gpuKey -Name "<FlapsErp.exe完整路徑>" -Value "GpuPreference=2;"
```

> 僅適用 HYBRID_GPU 分支（有獨顯）；IGPU_ONLY 分支不適用。

---

### 4.3 DESKTOP 分支適用（桌機/一體機）

#### 4.3.1 電源計畫

```powershell
powercfg /setactive SCHEME_MIN  # High Performance
# 或：powercfg /setactive SCHEME_BALANCED  # Balanced
```

#### 4.3.2 睡眠/休眠

```powershell
powercfg /change standby-timeout-ac 0
powercfg /change monitor-timeout-ac 0
```

> 僅桌機；筆電不建議。

---

### 4.4 SSD 分支適用

```powershell
# 定期（每月）
Optimize-Volume -DriveLetter C -ReTrim -Verbose
```

---

### 4.5 HIGH_RAM + DESKTOP + SSD 組合（理想環境）

以上 4.1 + 4.2 + 4.3 + 4.4 全部適用。

---

## 5. Phase 3：量測驗證（每次變更後必做）

```powershell
powershell -ExecutionPolicy Bypass -File "<Wellfit根目錄>\scripts\perf\flaps-perf.ps1" -Probe full
```

### 5.1 確認非 CONTENDED

```
baseline.contended = False
com.contended = False
host.contended = False
```

若任一 `contended=True`（有 `link`/`cargo`/`rustc` 程序、可用 RAM < 1 GB、磁碟佇列 > 5），樣本無效，等建置/負載結束後重測。

### 5.2 App smoke（每次必過）

1. 啟動 → 登入 → 選單就緒
2. 開一個功能模組 → 列表載入
3. 匯出 CSV → 檔案可讀
4. 關閉 → 重啟 → 再登入
5. 若改憑證相關設定，加測 HTTPS AP

### 5.3 系統級 A/B（系統變更必做）

同時開啟：FlapsErp + 瀏覽器數分頁 + Office 文件 + 檔案總管。

| 指標 | 判定 |
|---|---|
| FlapsErp 改善 | ≥ 10% 為改善 |
| 其他程式退步 | ≤ 10% 為可接受 |
| 溫度 | > 85°C 或降頻 → 回滾 |
| 溫度 UNKNOWN | **不可視為通過** |

---

## 6. Phase 4：回滾

| 變更 | 回滾 |
|---|---|
| Defender 排除 | `Remove-MpPreference -ExclusionPath ...` |
| GPU 偏好 | 刪除登錄值 `Remove-ItemProperty` |
| 程序優先級 | 重啟 FlapsErp 即恢復 Normal |
| 電源計畫 | `powercfg /setactive <原GUID>` |
| TCP | `netsh int tcp set global autotuninglevel=<原值>` |
| config.ini | 從備份還原＋SHA256 比對 |

---

## 7. 簽核

| 角色 | 名稱 | 日期 | 結果 |
|---|---|---|---|
| 硬體偵測 | | | |
| Defender 排除核准 | | | |
| 設定套用 | | | |
| 量測驗證 | | | |
| 系統級 A/B | | | |
| 批准 | | | |

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
| RAM booster | 無實證；可能更慢 |
| 在營業時間跑批次 | 搶其他程式資源 |
| 直接寫 ERP 資料庫 | 違反資料契約 |

---

## 9. 變更紀錄

| 版本 | 日期 | 變更 |
|---|---|---|
| v1.0 | 2026-11-11 | 初版；硬體特定 |
| v2.0 | 2026-11-12 | **重寫為機台無關版**：Phase 0 硬體偵測 → Phase 1 分支決策 → Phase 2 依分支套用 → Phase 3 量測 → Phase 4 回滾；不假設任何硬體規格；新增硬體分支決策表（RAM/GPU/電源/磁碟/OS 五個維度） |

---

*本 Runbook 適用於任何 Windows 10/11 機台；不含硬體特定值；所有建議均以 Phase 0 偵測結果為前提；未量測不得標 PASS。*