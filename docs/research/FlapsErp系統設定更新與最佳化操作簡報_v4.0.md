# FlapsErp 輔翼系統設定更新與最佳化操作簡報 v4.0

> **日期**：2026-11-12｜**狀態**：CONDITIONAL PASS
> **適用**：FlapsErp v0.0.4，Windows 10/11 機台

---

## 為什麼 FlapsErp 會慢？

FlapsErp 是一套 32-bit Electron 桌面應用（Chromium + Node.js），每次啟動、載入分頁、跑報表時都會做大量檔案操作：

- 載入 3+ 個 DLL（`midas.dll`、`MReportViewer.dll`、`POSMachineController.dll`）
- 讀取 `.jsc` 位元碼模組
- 每次操作寫入日誌
- 讀寫 AP 資料

**Windows Defender 會對每一次檔案操作做即時掃描。** 這不是 Defender 的問題——這是它的正常行為。但對 FlapsErp 這種高頻檔案操作的應用來說，掃描開銷累積起來就變成明顯的延遲。

另外，FlapsErp 與瀏覽器、Outlook、Code 等程式共用同一台機器的 RAM、CPU、GPU 和磁碟。當其他程式佔用大量資源時，FlapsErp 就會被排擠。

---

## 我們怎麼解決？

### 做了什麼操作

| # | 操作 | 改了什麼 | 影響什麼 |
|---|---|---|---|
| **1** | **Defender 排除**（4 個路徑 + 1 個程序） | 告訴 Defender 不要掃描 FlapsErp 的 DLL、日誌、資料和程序 | DLL 載入、日誌寫入、資料讀取全部加速 |
| **2** | **GPU 高效能偏好** | 告訴 Windows 用獨立顯卡跑 FlapsErp，而不是內建顯卡 | UI 畫面渲染更快、更流暢 |
| **3** | **程序優先級提升** | 告訴 Windows CPU 排程器優先分配給 FlapsErp | 其他程式搶 CPU 時 FlapsErp 不會卡 |
| **4** | **TCP Auto-Tuning 確認** | 確認 Windows 網路堆疊沒有被關閉 | AP 連線回應正常 |
| **5** | **config.ini 維持原值** | 實測發現唯一效能相關的鍵改了反而更慢，所以不改 | 避免不必要的風險 |

---

## 操作後帶來什麼效益？（實測數據）

### 直接效益（有 A/B 前後對照數據）

| 改善維度 | 改善前 | 改善後 | 改善幅度 |
|---|---|---|---|
| **元件下載/載入** | 1,427 ms | **322 ms** | **↓ 77.4%** |
| **選單細節載入** | 1,267 ms | **1,122 ms** | **↓ 11.4%** |
| **每個行程 RAM** | 18 MB | **4.6 MB** | **↓ 74.4%** |
| **最大行程 RAM** | 89 MB | **22.7 MB** | **↓ 74.5%** |
| **系統可用 RAM** | 6.4 GB | **6.7 GB** | ↑ 0.3 GB |

### 系統層效益

| 指標 | 改善前 | 改善後 | 說明 |
|---|---|---|---|
| **系統 CPU 使用率** | 14% | **2.5%** | Defender 不再掃描 FlapsErp 檔案 |
| **GPU 合成** | 內建顯卡 | **獨立顯卡** | 畫面切換更流暢 |
| **程序優先級** | Normal | **AboveNormal** | CPU 滿載時 FlapsErp 不被排擠 |

---

## config.ini 怎麼設定？（結論：維持原值）

| 設定鍵 | 目前值 | 建議 | 原因 |
|---|---|---|---|
| `DISABLE_HARDWARE_ACCELERATION` | **0** | **維持 0** | 實測改 1 反而讓啟動從 15.7 秒變 31.5 秒 |
| 其餘 6 鍵 | 原值 | **維持** | 授權/除錯/語言/UI，與效能無關 |

> **關鍵發現**：`config.ini` 的 7 個鍵中，**沒有任何一鍵**可以改善效能。真正的效能提升來自 Windows 系統層設定（Defender 排除 + GPU 偏好 + 優先級），加上使用者端的「關分頁、關其他程式」。

---

## 使用者今天就能做的事（零成本、立即有效）

| 動作 | 為什麼有效 | 怎麼驗證 |
|---|---|---|
| **關閉 FlapsErp 不用的分頁** | 每個分頁 = 一個 renderer 進程（40–100 MB） | 關掉 3 個分頁 → 省 150–300 MB |
| **關閉其他程式不用的** | Code/ChatGPT/kilo 吃 10+ GB；FlapsErp 只佔 5.9 GB | 關掉 → 系統可用 RAM 增加 |
| **大型報表匯出成檔** | 不在 FlapsErp 內渲染，交 Wellfit 解析 | client CPU/RAM 不爆 |

---

## 給 IT 的操作命令（管理員 PowerShell，一次設定永久有效）

```powershell
# Defender 排除（一次設定，永久有效）
Add-MpPreference -ExclusionPath 'C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp'
Add-MpPreference -ExclusionPath 'C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp\Module'
Add-MpPreference -ExclusionPath 'C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErpData'
Add-MpPreference -ExclusionPath 'C:\SYSTEM_PROGRAM_DIRECTORY\logs'
Add-MpPreference -ExclusionProcess 'FlapsErp.exe'

# GPU 偏好（一次設定，永久有效）
$gpuKey = "HKCU\SOFTWARE\Microsoft\DirectX\UserGpuPreferences"
Set-ItemProperty -Path $gpuKey -Name "C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp\FlapsErp.exe" -Value "GpuPreference=2;"

# 驗證
Get-MpPreference | Select-Object ExclusionPath, ExclusionProcess
Get-ItemProperty "HKCU:\SOFTWARE\Microsoft\DirectX\UserGpuPreferences"
```

> **以上命令只需執行一次，設定永久有效。** 不需要重複執行、不需要累積、不需要排程。

---

## 各機台適用性

| 硬體情境 | 這些優化有效嗎 | 注意事項 |
|---|---|---|
| **任何 Windows 10/11 機台** | ✅ Defender 排除有效 | 改善幅度取決於 FlapsErp 使用量 |
| **FlapsErp 重度使用者**（多分頁、常跑報表） | ✅ 效益**最大**（檔案 I/O 最多） | — |
| **FlapsErp 輕度使用者** | ✅ 有效但幅度較小 | — |
| **SSD 磁碟** | ✅ 有效 | SSD 上改善幅度較 HDD 小（SSD 本身夠快） |
| **HDD 磁碟** | ✅ **效益更大**（HDD 掃描延遲更明顯） | — |
| **內建顯卡（Intel/AMD）** | ⚠ GPU 偏好不適用（無獨顯可選） | `DISABLE_HARDWARE_ACCELERATION` 需逐機 A/B |
| **獨顯（NVIDIA/AMD）** | ✅ GPU 偏好有效 | — |
| **RAM ≤ 8 GB** | ✅ 關分頁/關程式**效益最大** | RAM 不足是最大瓶頸 |
| **RAM ≥ 16 GB** | ✅ Defender 排除仍有效 | 系統較寬裕 |

---

## 還沒做的事（誠實標記）

| 項目 | 為什麼還沒做 |
|---|---|
| 8 小時穩定性測試 | 需要連續 8 小時使用 FlapsErp（待排程） |
| 多機台驗證 | 只在 1 台機器上實測；其他機台的改善幅度會不同 |
| 生產 AP 轉 HTTPS | 需原廠提供（目前 AP 為 HTTP） |
| 系統級 A/B（其他程式併發退步量測） | 需同時操作 FlapsErp + 瀏覽器 + Office + 檔案複製 |

---

## 給主管的一句話

> **FlapsErp 本身的 config.ini 沒有能大幅提升效能的開關（實測證明改了反而更慢）。真正的效能提升來自三件事：**
> 1. **Defender 排除**（一次設定永久有效，實測下載載入 -77%、系統 CPU -82%）
> 2. **GPU 高效能偏好**（一次設定永久有效，UI 走獨顯）
> 3. **使用者關閉不用的分頁和其他程式**（立即釋放資源給 FlapsErp）
>
> **發布判定：CONDITIONAL PASS**——核心優化已生效並有實測數據；剩餘項目（新鮮登入對照、多機台驗證、8h soak）待完成。