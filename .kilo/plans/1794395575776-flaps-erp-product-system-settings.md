# FlapsErp 產品優化與系統最佳化：完整整合計畫 v4.1
# 細節 → 局部 → 全域 → 擴展性（跨機台、跨維度、跨尺度）

> 計畫編號：1794395575776｜日期：2026-11-12｜狀態：**NOT VERIFIED（局部已量測，全域待 IT）**
> 範圍：FlapsErp v0.0.4 原廠產品（`C:\SYSTEM_PROGRAM_DIRECTORY`）＋ Windows 10/11 宿主＋自建 Wellfit 外部工作台；**適用任何 Windows 機台**。
> 設計原則：**每一層的輸出是下一層的輸入**；局部不是孤立設定，而是全域最優的基礎；跨機台分支；隨 scope/scalability 擴大而**不降級**。
> 事實標記：✔ 2026-11-11/12 唯讀實測｜△ 推導待驗｜⚠ 需 IT/原廠。

---

## 0. 執行摘要

### 0.1 核心結論（已實測）

| 發現 | 證據 | 結論 |
|---|---|---|
| `config.ini` 7 鍵中**沒有任何一鍵**可帶來多維度改善 | `=1` startup 31.473 s vs `=0` 15.674 s | **不修改 config.ini** |
| FlapsErp RAM 佔比**狀態相依**：active 8 行程 5,913 MB（約 37%）；2026-11-03 閒置 623.8 MB（約 4%） | `-Probe baseline` 實測 | active 時是主要消費者之一；與其他程式 top 20（約 6.5 GB）合計才是系統壓力 |
| 真正的瓶頸是**原廠原始碼層**（sendSync／32-bit LAA 4 GB／console.log） | 逆向分析 ✔ | 設定無法消除；原廠才能根治 |
| 系統上其他程式（Code/ChatGPT/kilo/瀏覽器）top 20 約 6.5 GB | `-Probe host` 實測 | 關閉不用程式是最立即有效的操作 |
| Defender 即時掃描 DLL/日誌/資料是最大的**可調整 I/O 瓶頸** | 逆向分析 E-04 | **AV 排除是效益最大的系統設定** |
| 已實際生效：GPU 高效能偏好、程序優先級 AboveNormal | HKCU 登錄檔 + 程序狀態 | 持久；重啟後需重設優先級 |

### 0.2 四層結構（細節→局部→全域→擴展性）

```
L1 細節層｜單一檔案/鍵      config 7 鍵、.ver、processed.ini、logs、ReportTemp、Pos2Log
    ↓（局部輸出）
L2 局部層｜安裝目錄最佳化   Defender 排除、Search 排除、DLL 註冊衛生、日誌/暫存 retention
    ↓（局部輸出 + 系統狀態）
L3 全域層｜系統互動        12 個互動對（Defender/瀏覽器/Memory Compression/Update/GPU/磁碟/Wellfit…）
    ↓（跨機台分支 + 負載模型）
L4 擴展層｜多機台/多使用者  硬體分支、容量模型、閾值、不降級原則
```

### 0.3 8 程序完整量測（2026-11-12 11:21 UTC；`20261112-112150`）

| 指標 | 值 | 說明 |
|---|---|
| FlapsErp 行程數 | **8** | main + gpu + 5 renderer + utility |
| FlapsErp 總 RAM | **5,913 MB** | active（GPU 偏好已生效） |
| FlapsErp RAM median/true | 71.5 / 66.9 MB | 與 2026-11-03 的 46.2/46.2 相比增加（獨顯合成） |
| COM P50/P95/P99 | 0.1874 / 0.2454 / 0.311 ms | 優於 2026-11-03 的 0.264/0.388/0.474 |
| host free RAM | **4,870 MB** | 15.79 GB 總量 |
| host CPU load | **11.4%** | 8C/16T 閒置 |
| 磁碟佇列 | 0 | 無 I/O 瓶頸 |
| 溫度 | UNKNOWN | 筆電無 ACPI 熱區/權限 |
| contended | False | 無 cargo build；GPU+優先級已生效 |
| manifest drift | 1/9 | 8E2EDBE…version.txt（AP 版本更新） |
| LAA | True（0x0122） | x64 下 4 GB 可用 |

---

## 1. 軟體邊界與文件治理

### 1.1 軟體邊界（原廠 vs 自建 vs 宿主）

| 元件 | 所有者 | 路徑 | 可改性 |
|---|---|---|---|
| FlapsErp 外殼/執行檔 | 原廠 | `C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp\FlapsErp.exe`、`resources\app.asar` | **不可**（紅線） |
| FlapsErp 原廠設定 | 原廠 | `.conf\config.ini` | 僅 IT 變更管制 |
| FlapsErp 原廠元件 | 原廠 | `Module\*.jsc/.node`、`*.dll`、`.conf\*.version.txt` | **不可** |
| FlapsErp 原廠資料 | 原廠 | `FlapsErpData\AP\*.json`、`logs/`、`ReportTemp/`、`Pos2Log/` | **不可寫**；唯讀/IT 清理 |
| Wellfit 自建軟體 | 自建 | `Wellfit\crates`、`wellfit.exe`、`Wellfit\scripts`、`Wellfit\spec`、`Wellfit\data` | 可（依 Wellfit AGENTS） |
| Wellfit 量測工具 | 自建 | `Wellfit\scripts\perf\flaps-perf.ps1`、`Wellfit\artifacts\perf` | 可 |
| Rust 平台（非本計畫） | 自建 | `platform\`、`core\`、`web\`、`crates\` | 可（另計畫） |
| Windows 宿主 | 微軟/IT | OS 設定 | IT |

### 1.2 文件層級（SSOT／執行／全域／量測／基線／風險／歸檔／其他線）

| 層級 | 文件 | 版本 | 角色 |
|---|---|---|---|
| **SSOT** | 本計畫（`1794395575776`） | v4.1 | 總計畫；連接所有層 |
| 執行（機台無關） | `FlapsErp設定變更runbook_v2.0.md` | v2.0 | 跨機台 runbook |
| 執行（局部細節） | `FlapsErp局部設定最佳化Runbook_v3.0.md` | v3.0 | 安裝目錄檔案微調 |
| 全域 | `FlapsErp系統互動精密調校指南_v1.0.md` | v1.0 | 12 個互動對 |
| 量測 | `FlapsErp設定AB實測報告_v1.0.md` | v1.0 | A/B 實測與證據索引 |
| 基線 | `FlapsErp效能基線報告_v1.0.md` | v1.0 | 2026-11-03 歷史基線 |
| 風險 | `FlapsErp優化_合規風險追蹤表.md` | v1.2 | 風險登記 |
| 其他操作 | `A1檔案通道端到端操作手冊_v1.0.md`、`troubleshooting-runbook.md` | v1.0 | 外部工作台/急救 |
| 其他研究 | `FlapsErp產品線效能進度報告_v2.0.md`、`FlapsErp平台線效能評估進度報告_v2.0.md`、`輔翼ERP逆向分析與最佳化方案_v1.0.md` | v2.0/v1.0 | 產品線/平台線/逆向 |
| 歸檔 | `*_v1.0_ARCHIVED.md` | — | **不得使用** |

### 1.3 版本治理規則

1. **一個用途只有一個 SSOT**：本計畫為總計畫；執行/全域/量測/基線各有單一 SSOT。
2. 舊版一律標 `*_ARCHIVED.md`；不得刪除（C-01 保留軌跡）；不得引用。
3. 新增文件先註冊 `PLAN_REGISTER`／`docs/README.md`；無索引不得存在。
4. 數字/命令/路徑必須與實機一致；每個聲明附證據（artifacts/命令）。

---

## 2. L1 細節→L2 局部：FlapsErp 所有檔案設定全表

### 2.1 設定面全表（每個檔案/目錄）

| 檔案/目錄 | 所有者 | 可改性 | 量測方式 | 最佳化槓桿 | 證據 |
|---|---|---|---|---|
| `.conf\config.ini` | 原廠 | 僅 IT | `-Probe settings` | **無效能槓桿**（7 鍵已測完） | ✔ 7 鍵 |
| `.conf\processed.ini` | 原廠 | 不可 | `-Probe hygiene` | 監控是否每次啟動變動（DLL 重註冊） | ✔ 值 |
| `.conf\*.version.txt` | 原廠 | 不可 | `-Probe settings` | 漂移監控（AP 版本更新） | ✔ 3 檔 |
| `config\EXT024.txt` | 原廠 | 不可 | `Get-Content` | 授權/擴充；不修改 | ✔ 1 檔 |
| `Module\*.jsc/.node/.dll` | 原廠 | 不可 | `-Probe hygiene` | **Defender 排除**（載入加速） | ✔ 2 `.ver` |
| `resources\app.asar` | 原廠 | 不可 | `-Probe endpoints` | 唯讀端點掃描；不修改 | ✔ 11.8 MB |
| `FlapsErp.exe` | 原廠 | 不可 | `-Probe pe` | LAA 監控；不修改 | ✔ 134.4 MB |
| `logs\` | 原廠 | 不可寫 | `-Probe app` | 原廠自動輪替；Defender 排除；retention | ✔ 6 zip |
| `ReportTemp\Cache` | 原廠 | 不可寫 | `Get-ChildItem` | **保留**（快取；刪了反而慢） | ✔ 21 `.fr3` |
| `ReportTemp\Task` | 原廠 | IT 清理 | `Get-ChildItem` | 過期 `.result` 清理 | ✔ 1 檔 |
| `Pos2Log\ECD` | 原廠 | 不可寫 | `Get-ChildItem` | 空目錄清理；Defender 排除 | ✔ 22 目錄 |
| `Pos2Log\archived` | 原廠 | IT 清理 | `Get-ChildItem` | 壓縮保留 | ✔ 1 zip |
| `FlapsErpData\AP\*.json` | 原廠 | 不可 | `-Probe settings` | 預設 AP；HTTP；不修改 | ✔ 3 AP |
| `script\registerDLL.ps1` | 原廠 | 不可 | `-Probe hygiene` | 監控重複註冊；不修改 | ✔ 1 檔 |

### 2.2 config.ini 7 鍵（已測完；無效槓桿）

| 鍵 | 最佳值 | 證據 |
|---|---|---|
| `DISABLE_HARDWARE_ACCELERATION` | **0** | `=1`：31.473 s vs `=0`：15.674 s |
| `IGNORE_CERT_ERR` | **1** | 預設 AP HTTP，無作用 |
| `FORCE_DEVICE_SACLE_FACTOR` | **0** | UI 縮放（原廠拼字） |
| `KEEP_MACHINE_ID` | **1** | 授權綁定 |
| `DEBUG_FASTREPORT` | **0** | 除錯用 |
| `LANG` | **zh_TW** | 語言 |
| `KEEP_MACHINE_ID_MIGRATED` | **1** | 授權遷移 |

### 2.3 日誌/暫存/快取（local levers）

| 項目 | 最佳做法 | 效益 |
|---|---|---|
| `logs/` | 原廠自動輪替；IT 依 retention 清理 archived；Defender 排除 | 日誌寫入 -1~5 ms/次 |
| `ReportTemp/Cache` | **保留**（`.fr3` 快取；刪了重新下載更慢） | 避免重新下載 |
| `ReportTemp/Task` | IT 清理 > 7 天 `.result` | 磁碟/檔案數 |
| `Pos2Log/ECD` | IT 清理空目錄；非空保留 | 目錄數 |
| `Pos2Log/archived` | 原廠/IT retention | 磁碟 |

### 2.4 Module／原廠元件（不可改；只能監控與排除）

| 項目 | 做法 | 效益 |
|---|---|---|
| `Module/` | **Defender 排除**（IT） | DLL 載入 -50~200 ms/次 |
| `.conf\*.version.txt` | `-Probe settings` 比對 manifest | 原廠更新通知 |
| `.conf\processed.ini` | `-Probe hygiene` 啟動前後比對 | DLL 不重複註冊 |
| `script/registerDLL.ps1` | 不修改；IT 一次性安裝 | 避免重複 UAC/regsvr32 |

### 2.5 程序／註冊

| 項目 | 做法 | 效益 |
|---|---|---|
| FlapsErp 8 行程 | 單一實例；關閉不用分頁 | RAM -40~100 MB/分頁 |
| 程序優先級 AboveNormal | 已設定；重啟後重設 | 高負載不凍結 |
| GPU 高效能偏好 | 已設定（HKCU） | 獨顯合成 |
| 下載資料夾 | SSD | I/O |

### 2.6 AP／網路／TLS

| 項目 | 最佳做法 | 證據 |
|---|---|---|
| 預設 AP | 維持原廠 `8E2EDBE…` HTTP | `-Probe settings` |
| 防火牆 | 最小白名單（目的 IP/Port） | IT |
| TLS 1.0/1.1 | 不主動停用 | 舊 OpenSSL |
| `IGNORE_CERT_ERR` | HTTP 下 1；HTTPS 可信後 0 | `-Probe tls` |

### 2.7 局部層輸出（給全域層的輸入）

| 輸出 | 來源 | 用途 |
|---|---|---|
| config 7 鍵 | `.conf\config.ini` | 確認無效能槓桿 |
| Module `.ver`／`processed.ini` | `Module/`、`.conf/` | DLL 漂移/重註冊偵測 |
| logs/ReportTemp/Pos2Log 大小 | 目錄 | retention/清理決策 |
| FlapsErp 行程數/角色 | `-Probe hygiene` | 記憶體/分頁壓力 |
| AP 預設/協定 | `-Probe settings` | 網路/安全 |
| LAA/PE | `-Probe pe` | 位址空間 |

---

## 3. L2→L3：系統互動精密調校（11 個互動對；config.ini 另列 §2.2）

> 完整分析見 `FlapsErp系統互動精密調校指南_v1.0.md`。

| # | 互動對 | 調校點 | 全域效益 |
|---|---|---|---|
| 1 | Defender | 排除 DLL/日誌/資料 | FlapsErp 加速 + Defender 資源釋放 |
| 2 | 瀏覽器 | Memory Saver + 分頁管理 + GPU 分配 | 兩者不搶 GPU/RAM |
| 3 | Memory Compression | 減少 RAM 壓力 → 壓縮 CPU 下降 | 全機 CPU 效率提升 |
| 4 | Search Indexer | 排除索引 | 磁碟 I/O 釋放 |
| 5 | Windows Update | 活動時間 + 頻寬限制 | 不被中斷 |
| 6 | GPU 驅動/DWM | HYBRID：FlapsErp→獨顯，瀏覽器→內顯 | GPU 不過載 |
| 7 | TCP/IP | Auto-Tuning normal | AP 回應最優 |
| 8 | 磁碟 | SSD ReTrim；Code watcher 排除 | I/O 釋放 |
| 9 | Wellfit 批次 | 優先另機/Linux；同機→離峰+低優先 | FlapsErp 不受批次影響 |
| 10 | 開發工具 | Code watcher 排除；不同時高負載 | 磁碟/CPU 釋放 |
| 11 | 系統服務 | 排程避開工作時段 | 不被干擾 |

### 3.1 全域最優原則

1. **網路效益**：每個調校同時考慮 FlapsErp + 被調整元件 + 所有其他軟體。
2. **不犧牲安全**：不關閉 AV/UAC/WDAC；AV 排除最小範圍 + 核准 + 期限。
3. **不犧牲其他程式**：任一其他程式退步 > 10% → 回滾。
4. **不犧牲穩定性**：8h soak 通過才生產。
5. **不犧牲可維護性**：文件/命令/證據可追溯。

### 3.2 全域決策樹（symptom → local → global）

```
FlapsErp 慢？
├─ 啟動慢？
│   ├─ Defender 排除了？→ 沒有 → 加排除（互動 #1）
│   ├─ AP 回應 > 200 ms？→ 是 → 網路瓶頸
│   └─ DLL 重複註冊？→ 是 → processed.ini 異常
├─ 操作慢？
│   ├─ 可用 RAM < 2 GB？→ 關其他程式（互動 #3）
│   ├─ COM P95 > 0.5 ms？→ I/O 壓力（互動 #8）
│   └─ 磁碟佇列 > 2？→ 磁碟瓶頸
├─ UI 凍結？
│   ├─ 大量資料？→ sendSync（原廠層）
│   └─ 無大量資料？→ Memory Compression 或 GPU
└─ 間歇卡頓？
    ├─ Windows Update？→ 活動時間（互動 #5）
    ├─ Search Indexer？→ 排除索引（互動 #4）
    └─ Defender 掃描高峰？→ 排除（互動 #1）
```

---

## 4. L4 擴展層：跨機台分支與 scalability

### 4.1 硬體偵測（Phase 0；新機台必做）

| 維度 | 命令 |
|---|---|
| OS/記憶體 | `Get-CimInstance Win32_OperatingSystem` |
| CPU | `Get-CimInstance Win32_Processor` |
| RAM | `Get-CimInstance Win32_PhysicalMemory` |
| 磁碟 | `Get-PhysicalDisk`、`Get-Volume` |
| GPU | `Get-CimInstance Win32_VideoController` |
| 電源 | `powercfg /list`、`powercfg /a` |
| AV | `Get-MpComputerStatus` |
| UAC/WDAC | `Get-ExecutionPolicy -List`、`Get-AppLockerPolicy -Effective` |
| FlapsErp | `Get-Process FlapsErp`、`-Probe settings/pe/tls` |

### 4.2 分支決策表（局部 → 全域）

| 分支 | 局部設定 | 全域設定 |
|---|---|
| LOW_RAM（≤ 8 GB） | 關閉不用的程式；Defender 排除效益有限 | 不建議高效能電源；優先關閉其他程式 |
| MID_RAM（8–16 GB） | Defender 排除 + GPU 偏好 + 優先級 | 瀏覽器 Memory Saver；批次離峰 |
| HIGH_RAM（≥ 16 GB） | 全部局部 + Search 排除 | 全部全域；可併發更多 |
| IGPU_ONLY | `=1` 逐機 A/B | 不建議 GPU 偏好 |
| HYBRID_GPU | GPU 偏好高效能 | FlapsErp→獨顯；瀏覽器→內顯 |
| LAPTOP | 保留省電；不建議高效能電源 | 不用 USB/NIC 省電變更 |
| DESKTOP | 高效能電源；可停用睡眠 | 全部全域 |
| SSD | ReTrim | 不需要 Defrag |
| HDD | Defrag；考慮升級 | I/O 守門更嚴格 |

### 4.3 擴展性（Scalability）與不降級原則

| 擴展維度 | 對局部的影響 | 對全域的影響 | 應對 |
|---|---|---|
| +1 分頁 | +1 renderer（40–100 MB） | Memory Compression 增加 | 關閉不用分頁；分頁上限 |
| +1 DLL 更新 | `Module/`/`.ver` 變更 | Defender 排除仍有效 | `.ver` 比對 + 更新後重測 |
| +1 報表模板 | `ReportTemp/Cache` 增加 | 不影響全域（快取有益） | 保留；監控大小 |
| +1 日誌天數 | `logs/` 增加 | 原廠自動輪替 | retention |
| +1 使用者 | RAM/CPU 競爭加劇 | 全域互動複雜度增加 | 重新跑 Phase 0 + 互動分析 |
| +1 AP 伺服器 | `FlapsErpData/AP` 增加 | 網路/安全 | 白名單/CA |
| +1 批次工作 | Wellfit 資源增加 | 搶 CPU/RAM/I/O | 離峰/低優先/可中止 |

**不降級原則**：任一變更（局部/全域/擴展）若使 FlapsErp 或其他程式退步 > 10%，或使 8h soak 失敗 → 回滾。

---

## 5. 驗證測試計畫

### 5.1 每次變更後必做

| 步驟 | 內容 | 工具 |
|---|---|---|
| 1 | Pre-flight：非 CONTENDED | `-Probe full` → `contended=False` |
| 2 | 量測 | `-Probe full` → 記錄 `OUT_DIR` |
| 3 | App smoke | 登入→選單→模組→報表→重啟→再登入 |
| 4 | 系統級 A/B | FlapsErp + 瀏覽器 10 tabs + Office + 1 GB 複製 |
| 5 | 判定 | FlapsErp ≥ 10% 改善；其他 ≤ 10% 退步 |

### 5.2 8h soak（生產必要）

| 項目 | 條件 |
|---|---|
| 時長 | 8 小時連續 |
| 工作負載 | 開/關 10 tabs + 5 報表 + 瀏覽器 + Office（併發） |
| 採樣 | 每 5 分鐘 RAM；每 10 分鐘 app log |
| 通過條件 | RAM 增長 < 50 MB；無崩潰；其他程式 ≤ 基線 +10% |
| 狀態 | **未執行** |

### 5.3 跨機台驗證清單

| # | 檢查 | 工具 |
|---|---|---|
| 1 | 硬體偵測 | §4.1 命令 |
| 2 | 分支判定 | §4.2 表 |
| 3 | config 7 鍵 | `-Probe settings` |
| 4 | LAA | `-Probe pe` |
| 5 | TLS | `-Probe tls` |
| 6 | AP 連線 | `Test-NetConnection` |
| 7 | Defender 排除 | `Get-MpPreference` |
| 8 | GPU 偏好 | `Get-ItemProperty HKCU:\SOFTWARE\Microsoft\DirectX\UserGpuPreferences` |
| 9 | 基線 | `-Probe full` |
| 10 | App smoke | 手動 5 步 |

---

## 6. 任務清單（有序；代理 vs IT）

| ID | 任務 | 執行 | 輸入/命令 | 驗收 | 證據 |
|---|---|---|---|---|
| T-01 | 文件治理/版本歸檔 | 代理 | 標 `*_ARCHIVED.md`；更新 `PLAN_REGISTER` #27、`docs/README.md` | 無重複/死鏈 | 文件清單 |
| T-02 | 局部設定全表 | 代理 | 本計畫 §2.1 | 每個檔案/目錄有 owner/量測/槓桿 | 本計畫 |
| T-03 | config 7 鍵驗證 | 代理 | `-Probe settings` | 7 鍵與實機一致 | `195906` |
| T-04 | Defender 排除 | IT | `Add-MpPreference -ExclusionPath ...` | `Get-MpPreference` 前後 | IT runbook |
| T-05 | Search 索引排除 | IT | GUI/索引選項 | 排除生效 | IT runbook |
| T-06 | GPU 偏好 | IT/代理 | HKCU 登錄檔 | 已設定 | 登錄檔 |
| T-07 | 程序優先級 | IT/代理 | `PriorityClass = AboveNormal` | 已設定 | 程序狀態 |
| T-08 | 日誌/暫存 retention | IT | 清理 `ReportTemp/Task`、空 ECD 目錄 | 大小/目錄數 | artifacts |
| T-09 | Module/.ver 監控 | 代理 | `-Probe hygiene` | 漂移告警 | artifacts |
| T-10 | AP/網路/TLS | IT | 白名單/CA | `Test-NetConnection` | runbook |
| T-11 | Wellfit 批次外移 | 開發/IT | `run-mtrix-pipeline.ps1` | fixture exit 0/1 | `194835` |
| T-12 | 系統級 A/B | IT/代理 | 併發負載 | 其他 ≤ 10% | artifacts |
| T-13 | 8h soak | IT/代理 | 8 小時腳本 | RAM < 50 MB | artifacts |
| T-14 | 跨機台盤點 | 代理 | §4.1 命令 | `host-inventory` | artifacts |
| T-15 | 擴展性測試 | 代理/IT | 多分頁/多 AP/多用戶 | 不降級 | artifacts |
| T-16 | A/B 報告更新 | 代理 | §5 結果 | §F/G/L 更新 | 報告 |
| T-17 | 風險表更新 | 代理 | §9 | v1.3 | 風險表 |
| T-18 | 文件索引更新 | 代理 | §1.2 | `docs/README.md` | 索引 |

---

## 7. 風險與回滾

| 風險 | 等級 | 緩解 |
|---|---|---|
| 寫入原廠目錄 | 極高 | 代理唯讀；IT 變更管制 |
| AP HTTP 明文 | 極高 | 受控 LAN；要求 HTTPS |
| 32-bit LAA 4 GB | 高 | 分頁衛生/報表外移/夜間重啟 |
| 原廠元件漂移 | 高 | `.ver`/config 每季比對 |
| AV 排除降低安全 | 中 | 最小範圍/IT 核准/期限/複查 |
| 為 FlapsErp 拖垮其他程式 | 極高 | 系統級 A/B/批次守門/另機 |
| 夜間重啟影響批次 | 中 | 02:00 或批次後 |
| 文件版本散亂 | 中 | §1.2 文件層級/歸檔 |

**回滾**：config 從 `Wellfit/artifacts/config-backup/<date>/`＋SHA256；OS 依 runbook 原值；AP 還原；排程 `Unregister-ScheduledTask`；文件 `SUPERSEDED`+archive。

---

## 8. A–L 證據包

| 節 | 內容 |
|---|---|
| A | 目標、範圍、非目標 |
| B | 2026-11-11/12 唯讀現況（§0.3/§2） |
| C | 決策 D-01..D-15 |
| D | 變更內容（§2/§4；僅 IT 套用） |
| E | 量測方法（§5；命令、環境、5 次） |
| F | 實測結果（artifacts） |
| G | 前後對照與判定（§7） |
| H | 風險與回滾（§7） |
| I | 合規與紅線 |
| J | 文件與追溯（§1.2） |
| K | 未決事項（§9） |
| L | 發布判定（NOT VERIFIED） |

---

## 9. 未決

| 項目 | 狀態 | 解除條件 |
|---|---|---|
| IT 變更所有者/測試機 | 待指派 | IT 指定 |
| Defender 排除 | 待 IT | IT 管理員執行 |
| 8h soak | 未執行 | T-13 |
| 其他 HTTPS 端點盤點 | 部分（app.asar 無 flaps/wellfit literal；runtime 待監控） | T-10 |
| 生產 AP 轉 HTTPS | 待原廠 | OQ-F1/F2 |
| COM/CDP 正式化 | 範圍外 | Phase 3–4 |
| 文件版本歸檔 | 待 T-01 | 標 ARCHIVED + 索引 |

---

## 10. 修訂紀錄

| 版本 | 日期 | 變更 |
|---|---|---|
| v4.1 | 2026-11-12 | 修正表格格式（所有表格列統一 `|`）；文件治理；設定面全表；跨機台分支；8 程序量測；任務清單；A–L；不降級原則 |
| v4.0 | 2026-11-12 | 四層結構整合 |
| v3.0 | 2026-11-12 | 三層架構整合 |
| v2.0 | 2026-11-11 | 成效導向版 |
| v1.0 | 2026-11-11 | 初版 |

---

*本計畫為 FlapsErp 產品優化與系統最佳化的**總計畫（SSOT）**。細節見 §2；局部見 `FlapsErp局部設定最佳化Runbook_v3.0.md`；全域見 `FlapsErp系統互動精密調校指南_v1.0.md`；量測見 `FlapsErp設定AB實測報告_v1.0.md`。每一層的輸出是下一層的輸入；未量測不得標 PASS；UNKNOWN ≠ PASS ≠ FAIL。*