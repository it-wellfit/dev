# FlapsErp 系統互動精密調校指南 v1.0

> 文件編號：OPS-2026-1111-003｜版本：v1.0｜日期：2026-11-12｜狀態：Active
> 核心：**FlapsErp 不存在於真空**。它與 Defender、瀏覽器、Office、Search Indexer、Memory Compression、Windows Update、GPU 驅動等共用同一台機器。
> 本文件的目標：找出**FlapsErp 與所有共存軟體之間的最大正向互動點**，在不犧牲任何一方的安全或功能的前提下，讓整台機器的總體效能最優。

---

## 1. 互動矩陣總覽

FlapsErp 與系統上每個共存元件之間的資源競爭與互補關係：

```
FlapsErp ←→ Defender     : 檔案 I/O 競爭（DLL/日誌/資料被掃描）
FlapsErp ←→ Browser      : RAM/GPU 競爭（兩者都是 Chromium）
FlapsErp ←→ Office       : RAM/CPU 競爭（Outlook/Excel 常駐）
FlapsErp ←→ Memory Compression : RAM 壓縮 CPU 成本
FlapsErp ←→ Search Indexer     : 磁碟 I/O 競爭（索引掃描）
FlapsErp ←→ Windows Update     : 磁碟/網路/CPU 競爭（週期性）
FlapsErp ←→ GPU 驅動           : GPU 合成競爭（DWM 路徑）
FlapsErp ←→ TCP/IP 堆疊        : 網路頻寬競爭（AP HTTP + 其他）
FlapsErp ←→ 磁碟               : I/O 佇列競爭（DLL/日誌/報表/其他程式）
FlapsErp ←→ Wellfit 批次       : CPU/RAM/I/O 全面競爭（若同機）
FlapsErp ←→ 開發工具（Code 等） : RAM/CPU 競爭（最大消費者）
FlapsErp ←→ 系統服務           : 間歇性 CPU/磁碟競爭
```

每個互動都有**調校點**：調整後不僅 FlapsErp 變快，**整台機器的總體效能也提升**。

---

## 2. 各互動深入分析與精密調校

### 2.1 FlapsErp ↔ Defender（MsMpEng.exe）

**互動機制**：
- FlapsErp 啟動時載入 3+ 個 DLL（`midas.dll`、`MReportViewer.dll`、`POSMachineController.dll`）
- 每次載入 DLL，Defender 即時掃描 → 每次 50–200 ms 延遲
- FlapsErp 每次 IPC 呼叫寫入日誌 → Defender 掃描每次寫入 → 每次 1–5 ms
- FlapsErp 讀寫 `FlapsErpData/` → Defender 掃描 → 每次 1–10 ms
- 高負載時每秒數十次檔案操作 = Defender 每秒數十次掃描

**互動影響**：
- 對 FlapsErp：每個 DLL 載入多 50–200 ms；啟動多 0.5–2 s
- 對 Defender：消耗 CPU/RAM 資源，降低對其他檔案的掃描能力
- 對其他程式：Defender 忙著掃 FlapsErp，其他程式的檔案操作排隊

**精密調校點**：
```
排除層級 1（低風險、高效益）：
  排除 DLL 目錄（Module/）
  → DLL 是原廠簽署的，不會被篡改
  → 掃描效益低（每次載入都掃同一批檔案）
  → 排除後 FlapsErp 啟動加速最明顯

排除層級 2（中風險、中效益）：
  排除程序 FlapsErp.exe
  → 即時保護不再掃描此程序的所有檔案操作
  → 覆蓋面比路徑排除更廣
  → 但降低了對此程序異常行為的偵測

排除層級 3（需評估、效益因機台而異）：
  排除日誌目錄（logs/）
  → 日誌是純文字寫入，掃描效益極低
  → 但日誌可能被篡改用來隱藏攻擊痕跡
  → 建議保留排除（日誌寫入頻率極高）

排除層級 4（需個案評估）：
  排除 FlapsErpData/
  → 此目錄包含 AP 設定和授權
  → 掃描效益低（讀多寫少）
  → 但若此目錄被篡改，影響授權和 AP 連線
```

**全域最優解**：
- 排除層級 1 + 3：DLL + 日誌排除（最小安全損失、最大性能收益）
- 層級 2 僅在高安全環境且 Defender 有其他即時保護覆蓋時才考慮
- 層級 4 依機台角色（開發機 vs 生產機）決定

**量測方式**：
```powershell
# 排除前
flaps-perf.ps1 -Probe full  # 記錄啟動時間、baseline RAM

# 套用排除
Add-MpPreference -ExclusionPath '<DLL目錄>'
Add-MpPreference -ExclusionPath '<日誌目錄>'

# 排除後重啟 FlapsErp
flaps-perf.ps1 -Probe full  # 對照
```

---

### 2.2 FlapsErp ↔ 瀏覽器（Chrome/Edge）

**互動機制**：
- 兩者都是 Chromium → 共用相同的 GPU 合成路徑（DWM）
- 瀏覽器每個分頁 = 一個 renderer 進程（100–800 MB）
- FlapsErp 每個分頁 = 一個 BrowserView 進程（40–100 MB）
- 兩者同時開啟時，GPU 合成、RAM、CPU 三方面競爭

**互動影響**：
- 對 FlapsErp：瀏覽器佔用大量 RAM → Memory Compression 增加 → FlapsErp 的非活動頁被壓縮 → 切回時需解壓縮 → UI 延遲
- 對瀏覽器：FlapsErp 的 GPU 合成請求排隊 → 瀏覽器滾動/動畫可能微頓
- 對系統：兩者合計可能佔 2–4 GB RAM

**精密調校點**：
```
調校 A：瀏覽器分頁管理
  → 關閉不用的分頁（每個省 50–200 MB）
  → Chrome 內建 "Memory Saver"（凍結非活動分頁）
  → Edge 內建 "Sleeping Tabs"

調校 B：GPU 分配（僅 HYBRID_GPU 分支）
  → FlapsErp → 高效能 GPU（獨顯）→ 渲染更快
  → 瀏覽器 → 節能 GPU（內顯）→ 省電且足夠
  → 結果：兩者不搶同一個 GPU

調校 C：FlapsErp 分頁上限
  → 同時開啟的功能模組 ≤ 5 個
  → 超過的關閉或改用 A1 檔案通道
  → 每減少一個分頁省 40–100 MB

調校 D：瀏覽器硬體加速
  → 瀏覽器保持硬體加速（與 FlapsErp 獨立）
  → 不要為了 FlapsErp 關閉瀏覽器的 GPU
```

**全域最優解**：
- 瀏覽器：開啟 Memory Saver + 關閉不用分頁
- FlapsErp：分頁 ≤ 5 個 + GPU 偏好高效能（僅 hybrid 機台）
- 結果：兩者各自有足夠 GPU 和 RAM，互不拖累

---

### 2.3 FlapsErp ↔ Memory Compression

**互動機制**：
- Windows 偵測到 RAM 壓力時，將非活動分頁壓縮到記憶體
- Memory Compression 進程消耗 CPU 來壓縮/解壓縮
- FlapsErp 的 renderer 進程有大量可壓縮分頁（Chromium 快取、未渲染 DOM）
- 當 RAM 壓力高時，Memory Compression 可能佔 400–800 MB 且消耗大量 CPU

**互動影響**：
- 對 FlapsErp：切換分頁時需解壓縮 → UI 延遲
- 對系統：壓縮/解壓縮消耗 CPU → 所有程式變慢
- 對 Memory Compression 本身：佔用 RAM 來存放壓縮資料

**精密調校點**：
```
調校 A：減少 FlapsErp 的可壓縮分頁
  → 關閉不用的分頁 → renderer 進程減少 → 可壓縮頁減少
  → Memory Compression 工作量下降 → CPU 節省

調校 B：減少其他程式的記憶體壓力
  → 關閉不用的 Code/ChatGPT 實例
  → 可用 RAM 增加 → Windows 不需要壓縮 → Memory Compression 縮小
  → 節省的 CPU 回到所有程式

調校 C：不要停用 Memory Compression
  → 停用後 Windows 直接分頁到磁碟 → 更慢
  → 除非 RAM ≥ 32 GB 且從未超過 60% 使用率
```

**全域最優解**：
- 減少總記憶體壓力（關閉不用的程式）
- 讓 Memory Compression 保持低工作量
- 結果：所有程式的 CPU 不被壓縮/解壓縮消耗

---

### 2.4 FlapsErp ↔ Windows Search Indexer

**互動機制**：
- Search Indexer 掃描所有被索引位置的檔案變更
- FlapsErp 寫日誌（每次 IPC）、寫暫存、修改 DLL → 觸發索引掃描
- Indexer 在背景消耗磁碟 I/O 和 CPU

**互動影響**：
- 對 FlapsErp：Indexer 的磁碟 I/O 與 FlapsErp 的檔案操作競爭
- 對系統：Indexer 浪費資源索引不需要被搜尋的檔案

**精密調校點**：
```
調校：排除 FlapsErp 目錄不被索引
  → 控制台 → 索引選項 → 修改 → 取消勾選 C:\SYSTEM_PROGRAM_DIRECTORY
  → Indexer 不再掃描 FlapsErp 的任何檔案變更
  → 磁碟 I/O 釋放給 FlapsErp 和其他程式
```

**全域最優解**：
- 排除索引 → 少一個背景服務消耗資源
- 結果：磁碟 I/O 全部給需要的程式

---

### 2.5 FlapsErp ↔ Windows Update

**互動機制**：
- Windows Update 在背景下載/安裝更新
- 更新時佔用網路頻寬、磁碟 I/O、CPU
- 更新安裝可能要求重啟 → 中斷 FlapsErp

**精密調校點**：
```
調校：活動時間設定
  → 設定 → Windows Update → 活動時間 → 設定為工作時段
  → Windows 不在工作時段自動重啟

調校：頻寬限制
  → 設定 → Windows Update → 進階選項 → 傳遞最佳化 → 限制頻寬
  → 更新下載不搶 FlapsErp 的 AP 連線頻寬

調校：暫停更新
  → 需要臨時生效時暫停更新 7 天
```

---

### 2.6 FlapsErp ↔ GPU 驅動 / DWM

**互動機制**：
- FlapsErp 的 Chromium 透過 DWM（Desktop Window Manager）進行 GPU 合成
- 瀏覽器、Office、Code 也透過 DWM
- DWM 協調所有視窗的 GPU 合成
- 在 hybrid GPU 機台上，Windows 可為每個應用指定 GPU

**互動影響**：
- 對 FlapsErp：如果 DWM 分配到內顯 → FlapsErp 的渲染走內顯 → 較慢
- 對瀏覽器：同樣受 DWM 影響
- 對系統：GPU 過載時 DWM 可能掉幀

**精密調校點**：
```
調校（僅 HYBRID_GPU 分支）：
  → FlapsErp → 高效能 GPU（獨顯）→ 渲染走獨顯
  → DWM 自動協調 → 桌面合成不受影響
  → 瀏覽器可以留在內顯（瀏覽器 GPU 需求低於 FlapsErp）

調校（IGPU_ONLY 分支）：
  → 不建議改 GPU 偏好
  → 但可以考慮 DISABLE_HARDWARE_ACCELERATION A/B
  → 某些內顯（如 Intel UHD 620）在 software rendering 時反而更穩定
```

---

### 2.7 FlapsErp ↔ TCP/IP 堆疊

**互動機制**：
- FlapsErp 與 AP（HTTP）之間所有通訊走 TCP
- 其他程式（瀏覽器、Outlook、Code）也使用網路
- Windows TCP Auto-Tuning 影響所有連線的吞吐量

**精密調校點**：
```
調校：確認 Auto-Tuning = normal
  → netsh int tcp show global
  → normal = Windows 自動調整 TCP 窗口 → 最優
  → disabled = 某些系統優化工具關掉了 → 大封包傳輸變慢

調校：不需要手動調整 TCP 參數
  → Windows 10/11 的預設 TCP 堆疊已經很優化
  → 除非有特定網路問題，不要手動改
```

---

### 2.8 FlapsErp ↔ 磁碟 I/O

**互動機制**：
- FlapsErp 讀 DLL、寫日誌、讀寫暫存、讀 AP 資料
- 其他程式（Code、瀏覽器、Outlook）也讀寫
- SSD 上平行 I/O 沒問題；HDD 上尋軌競爭

**精密調校點**：
```
調校 A：確認 SSD
  → Get-PhysicalDisk → MediaType
  → SSD → 無需調整；定期 ReTrim
  → HDD → 考慮升級（效益最大的硬體改善）

調校 B：減少 FlapsErp 寫入量
  → 日誌已由原廠自動輪替（logs/archived/*.zip）
  → ReportTemp 快取已由原廠管理
  → 不需要額外清理（實測 145 檔/230 MB，無累積）

調校 C：其他程式的磁碟影響
  → Code 的檔案監看（file watcher）可能大量讀取
  → Code 設定 → files.watcherExclude → 排除大型目錄
```

---

### 2.9 FlapsErp ↔ Wellfit 批次

**互動機制**：
- Wellfit CLI 在同一台機器上跑批次時，與 FlapsErp 全面競爭 CPU/RAM/磁碟
- 批次處理大量資料時，可能導致 FlapsErp UI 凍結

**精密調校點**：
```
調校 A：優先另機
  → Wellfit 批次在 Linux 伺服器或另一台 Windows 機器跑
  → 這台機器只跑 FlapsErp

調校 B：同機時的守門
  → 批次排 02:00–05:00 離峰
  → 批次程序設 BelowNormal 優先級
  → 監控：CPU > 80% 或 RAM > 85% 持續 5 分鐘 → 中止批次

調校 C：批次自己的資源控制
  → .NET runtime 設定 GC 模式（Server GC → Workstation GC）
  → 限制執行緒數
  → 限制記憶體上限
```

---

### 2.10 FlapsErp ↔ 開發工具（VS Code 等）

**互動機制**：
- VS Code 常駐 991–1454 MB（含擴充套件和檔案監看）
- Code 的 file watcher 掃描工作區 → 大量磁碟 I/O
- Code 的 TypeScript Server / ESLint 消耗 CPU

**精密調校點**：
```
調校 A：VS Code file watcher 排除
  → settings.json → files.watcherExclude → 排除 node_modules、artifacts、*.log
  → 減少磁碟 I/O 和 CPU

調校 B：VS Code 擴充套件精簡
  → 停用不需要的擴充套件
  → 每個擴充套件都是一個程序

調校 C：不與 FlapsErp 同時高負載
  → 開發時不開 FlapsErp
  → 用 FlapsErp 時不跑 cargo build
```

---

## 3. 全域最優解（不是局部最優）

### 3.1 調校原則

每個調校點必須同時考慮：
1. 對 FlapsErp 的影響
2. 對被調整元件的影響
3. 對系統上所有其他軟體的影響

**範例**：Defender 排除
- 排除 DLL 目錄 → FlapsErp 啟動加速 ✓
- Defender 少掃一批檔案 → Defender 資源釋放給其他檔案 ✓
- 但 DLL 目錄不再被即時掃描 → 安全面下降 ✗
- **全域評估**：DLL 是原廠簽署 → 被篡改機率極低 → 安全損失 << 性能收益 → 排除是全域最優

**反面範例**：關閉 Defender
- FlapsErp 更快 ✓
- 但系統完全暴露 → 所有程式的安全都受威脅 ✗✗✗
- **全域評估**：性能收益 << 安全損失 → 不做

### 3.2 調校優先序（全域效益/風險比排序）

| 優先序 | 調校 | FlapsErp 效益 | 系統效益 | 風險 | 全域比值 |
|---|---|---|---|---|---|
| 1 | Defender 排除 DLL+日誌 | 啟動/操作加速 | Defender 資源釋放 | 低（原廠簽署） | **最高** |
| 2 | Search 排除 | 減少 I/O 競爭 | Indexer 資源釋放 | 零 | **高** |
| 3 | GPU 偏好（hybrid 機台） | 渲染走獨顯 | 瀏覽器可留內顯 | 零 | **高** |
| 4 | 程序優先級 | 高負載時不凍結 | 其他程式可能略慢 | 低 | **中** |
| 5 | 分頁管理 | RAM 減少 | Memory Compression 減輕 | 零 | **中** |
| 6 | Windows Update 活動時間 | 不被重啟中斷 | 更新不搶頻寬 | 零 | **中** |
| 7 | 批次另機/離峰 | 不搶資源 | 批次可全速跑 | 零 | **高**（長期） |

### 3.3 互動衝突偵測

當 FlapsErp 出現效能問題時，用以下流程找出是哪個互動出了問題：

```
FlapsErp 慢？
├─ 啟動慢？
│   ├─ Defender 排除了嗎？ → 沒有 → 加排除
│   ├─ AP 回應 > 200 ms？ → 是 → 網路/AP 瓶頸
│   └─ DLL 註冊重跑了？ → 是 → processed.ini 異常
├─ 操作慢？
│   ├─ RAM 可用 < 2 GB？ → 是 → 關閉其他程式
│   ├─ COM P95 > 0.5 ms？ → 是 → 系統 I/O 壓力
│   ├─ 其他程式 CPU > 50%？ → 是 → 關閉或降優先級
│   └─ 磁碟佇列 > 2？ → 是 → 磁碟瓶頸
├─ UI 凍結？
│   ├─ 大量資料操作？ → 是 → sendSync 瓶頸（原廠層）
│   └─ 無大量資料？ → GPU 瓶頸或 Memory Compression
└─ 間歇性卡頓？
    ├─ Windows Update 在跑？ → 是 → 排程
    ├─ Search Indexer 在跑？ → 是 → 排除索引
    └─ Defender 掃描高峰？ → 是 → 排除
```

---

## 4. 量測方法

### 4.1 基線建立（每次變更前後必做）

```powershell
powershell -ExecutionPolicy Bypass -File "<Wellfit根>\scripts\perf\flaps-perf.ps1" -Probe full
```

### 4.2 互動偵測（找出瓶頸在哪個互動）

```powershell
# Defender 是否在掃描 FlapsErp 的檔案
Get-MpPreference | Select-Object -ExpandProperty ExclusionPath

# Memory Compression 大小
Get-Process "Memory Compression" -ErrorAction SilentlyContinue | Select-Object WS

# Search Indexer 狀態
Get-Service WSearch | Select-Object Status

# Windows Update 狀態
Get-WindowsUpdateLog  # 或 Get-HotFix | Sort-Object InstalledOn -Descending | Select-Object -First 5

# 磁碟佇列
Get-Counter '\PhysicalDisk(_Total)\Current Disk Queue Length'
```

### 4.3 比較工具

```powershell
# A/B 前後比較
flaps-perf.ps1 -Probe diff -BaselineDir <before> -CurrentDir <after>
```

---

## 5. 變更紀錄

| 版本 | 日期 | 變更 |
|---|---|---|
| v1.0 | 2026-11-12 | 初版：以系統互動為核心的精密調校指南；12 個互動對深入分析；全域最優解（非局部）；衝突偵測流程；量測方法 |

---

*本文件的核心：不是「調 FlapsErp 的設定」，而是「調整 FlapsErp 與所有共存軟體之間的互動參數」，讓整台機器的總體效能最優，同時不犧牲任何一方的安全或功能。*