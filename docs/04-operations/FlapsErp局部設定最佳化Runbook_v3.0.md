# FlapsErp 局部設定最佳化 Runbook v3.0
# 範圍：C:\SYSTEM_PROGRAM_DIRECTORY 內所有 FlapsErp 相關檔案的微調
# 原則：不碰原廠二進位；不碰原廠資料；只微調設定/暫存/日誌/快取/程序/註冊
# 上游：本文件為「局部最優」；全域最優見「FlapsErp系統互動精密調校指南_v1.0.md」

---

## 第一部分：config.ini 精密設定

### 1.1 七鍵逐一說明與最佳值決策樹

```
IGNORE_CERT_ERR
├─ 預設 AP 是 HTTPS 且憑證可信？ → 0（安全最佳）
├─ 預設 AP 是 HTTP？            → 1（對 HTTP 無作用，維持不變）
└─ 混合/不確定？                → 1（保守；待 HTTPS 端點盤點完成後再改 0）

KEEP_MACHINE_ID
└─ 永遠 = 1（授權綁定；改 0 會導致授權失效）

DISABLE_HARDWARE_ACCELERATION
├─ GPU 類型？
│   ├─ 僅內顯（Intel UHD/Iris、AMD Radeon Graphics）
│   │   ├─ A/B 實測 =1 比 =0 快？ → 採用實測結果
│   │   ├─ A/B 實測 =0 比 =1 快？ → 維持 0
│   │   └─ 尚未 A/B？            → 維持 0（預設；逐機 A/B 後再改）
│   ├─ 內顯 + 獨顯（hybrid）     → 0（獨顯硬體加速幾乎總是更快）
│   └─ 僅獨顯                    → 0（硬體加速必須開）
│
└─ 本機已 A/B 實測？ =1 更慢 → 0；=1 更快 → 1

DEBUG_FASTREPORT
└─ 永遠 = 0（除非原廠支援人員要求開啟除錯；開啟會增加日誌量與延遲）

FORCE_DEVICE_SACLE_FACTOR
├─ UI 有裁切/模糊？ → 嘗試 1（強制 100% 縮放）
├─ UI 正常？       → 維持 0
└─ 注意：此鍵名拼字是原廠的（SACLE，非 SCALE）；不可改名

LANG
└─ 依使用者需求；zh_TW/en/jp

KEEP_MACHINE_ID_MIGRATED
└─ 永遠 = 1（原廠管理）
```

### 1.2 config.ini 修改程序（每次必做）

```
1. 讀取目前 7 鍵值 → 記錄
2. 備份 → <備份根>/config-backup/<ts>/config.ini + SHA256
3. 用 [IO.File]::ReadAllText + -replace 只改目標鍵 → [IO.File]::WriteAllText UTF-8 無 BOM
4. 驗證：讀回 7 鍵逐字比對（只有目標鍵變了，其他 6 鍵不變）
5. 重啟 FlapsErp
6. -Probe settings 確認
7. -Probe app 量測
```

---

## 第二部分：日誌管理（logs/）

### 2.1 現況

| 路徑 | 內容 | 原廠行為 |
|---|---|---|
| `logs/application-WELLFIT-*.log` | 當日日誌（每次 IPC 寫入） | 原廠每日輪替 |
| `logs/archived/*.zip` | 歷史日誌（已壓縮） | 原廠自動歸檔 |

### 2.2 可微調項目

| 項目 | 做法 | 效益 |
|---|---|---|
| 日誌目錄排除 Defender 掃描 | `Add-MpPreference -ExclusionPath '<安裝根>\logs'` | 每次日誌寫入 -1~5 ms |
| 日誌大小監控 | `Get-ChildItem <安裝根>\logs -File | Measure-Object Length -Sum` | 異常增長告警 |
| archived zip 保留期 | IT 政策（建議 90 天） | 磁碟空間 |

### 2.3 不做的事

- 不刪除當日日誌（原廠正在寫入）
- 不刪除 archived zip（除超過保留期）
- 不修改日誌格式/內容（原廠管理）

---

## 第三部分：報表暫存管理（ReportTemp/）

### 3.1 現況

| 路徑 | 內容 | 說明 |
|---|---|---|
| `ReportTemp/Cache/` | `.fr3` 報表模板快取 | **保留**：刪了會重新下載，反而慢 |
| `ReportTemp/Task/` | `.result` 報表工作結果 | **可清理**：已完成/過期的報表結果 |

### 3.2 可微調項目

| 項目 | 做法 | 效益 |
|---|---|---|
| Task/ 過期清理 | `Get-ChildItem <安裝根>\ReportTemp\Cache\Task -File | Where-Object LastWriteTime -lt (Get-Date).AddDays(-7) | Remove-Item` | 磁碟空間；減少檔案數 |
| Cache/ **保留** | 不刪除 .fr3 模板 | 刪了反而慢（重新下載） |
| 排除 Defender 掃描 | `Add-MpPreference -ExclusionPath '<安裝根>\ReportTemp'` | 減少報表 I/O 延遲 |

### 3.3 不做的事

- 不刪除 Cache/ 的 .fr3 模板（快取，刪了反而慢）
- 不修改 .result 檔案內容（原廠格式）

---

## 第四部分：POS 日誌管理（Pos2Log/）

### 4.1 現況

| 路徑 | 內容 | 說明 |
|---|---|---|
| `Pos2Log/ECD/<日期>/` | POS 交易日誌（按日分目錄） | 實測多數日期目錄為空 |
| `Pos2Log/archived/` | 已壓縮歷史 | 原廠自動歸檔 |

### 4.2 可微調項目

| 項目 | 做法 | 效益 |
|---|---|---|
| 空目錄清理 | `Get-ChildItem <安裝根>\Pos2Log\ECD -Directory | Where-Object { (Get-ChildItem $_.FullName -File).Count -eq 0 } | Remove-Item` | 減少目錄數；微小 |
| ECD 目錄排除 Defender | `Add-MpPreference -ExclusionPath '<安裝根>\Pos2Log'` | 減少 POS 日誌 I/O 延遲 |

### 4.3 不做的事

- 不刪除非空日期目錄（可能含交易紀錄）
- 不修改 archived zip

---

## 第五部分：COM 元件管理（Module/ + script/）

### 5.1 現況

| 檔案 | 說明 | 原廠行為 |
|---|---|---|
| `Module/midas.dll` | Borland 資料庫引擎 | regsvr32 註冊 |
| `Module/MReportViewer.dll` | FastReport 報表引擎 | regsvr32 註冊 |
| `Module/POSMachineController.dll` | POS 控制器 | regsvr32 註冊 |
| `Module/*.jsc` | bytenode 位元碼模組 | Node.js require |
| `script/registerDLL.ps1` | DLL 註冊腳本 | 原廠執行 |
| `.conf/processed.ini` | 註冊狀態標記 | 原廠管理 |

### 5.2 可微調項目

| 項目 | 做法 | 效益 |
|---|---|---|
| Module/ 排除 Defender | `Add-MpPreference -ExclusionPath '<安裝根>\Module'` | DLL 載入 -50~200 ms/次 |
| Module/ 排除 Search Index | GUI 索引選項取消勾選 | 減少背景掃描 |
| 監控 .ver 變更 | `-Probe settings` 比對 manifest | 原廠更新通知 |
| 確認 DLL 註冊不重複 | `-Probe hygiene` 比對 processed.ini 啟動前後 | DLL 不重複註冊 |

### 5.3 不做的事

- 不重新註冊 DLL（除非原廠更新後）
- 不刪除/移動 DLL
- 不修改 registerDLL.ps1
- 不修改 .jsc 檔案

---

## 第六部分：程序管理

### 6.1 FlapsErp 程序結構

```
FlapsErp.exe（main）— Node.js 主程序
├── FlapsErp.exe（renderer）× N — 每個分頁一個 BrowserView
├── FlapsErp.exe（gpu-process）— GPU 合成
└── FlapsErp.exe（utility）— 網路/音訊
```

### 6.2 可微調項目

| 項目 | 做法 | 效益 |
|---|---|---|
| 程序優先級 AboveNormal | `Get-Process FlapsErp | ForEach { $_.PriorityClass = 'AboveNormal' }` | 高負載時不凍結 |
| 單一實例 | 啟動前 `Get-Process FlapsErp`；重複則提示 | 不浪費資源 |
| 關閉不用的分頁 | 每個分頁 = 40–100 MB renderer | RAM 直接下降 |
| 排除程序掃描 | `Add-MpPreference -ExclusionProcess 'FlapsErp.exe'` | 所有檔案操作免掃描 |

### 6.3 不做的事

- 不設 High/Realtime（會餓死其他程式）
- 不強制 kill（除非當機；先提示使用者儲存）
- 不限制記憶體（Electron 需要彈性）

---

## 第七部分：AP 連線管理（網路層微調）

### 7.1 現況

| 項目 | 值 |
|---|---|
| 預設 AP | `http://<AP_IP>:<AP_Port>/<路徑>/` |
| 協定 | HTTP（非 HTTPS） |
| 認證 | JSP session |

### 7.2 可微調項目

| 項目 | 做法 | 效益 |
|---|---|---|
| 防火牆白名單 | `New-NetFirewallRule` 允許 outbound 到 AP IP:Port | 減少防火牆規則評估延遲 |
| AP 回應量測 | `Measure-Command { Invoke-WebRequest <AP_URL> }` | 建立網路基準；> 200 ms 為網路瓶頸 |
| DNS | AP 用 IP 直連，無 DNS 延遲 | 無需調整 |
| TCP Auto-Tuning | `netsh int tcp show global` 確認 normal | 若 disabled 則改回 normal |

### 7.3 不做的事

- 不修改 AP URL（原廠管理）
- 不加 proxy（除非企業政策）
- 不停用 IPv6（不影響 IPv4 連線）

---

## 第八部分：整體微調彙總（依效益排序）

| 優先 | 微調 | 位置 | 效益 | 風險 | 需管理員 |
|---|---|---|---|---|---|
| 1 | Module/ 排除 Defender | `Module/` | DLL 載入加速 | 低 | ✓ |
| 2 | logs/ 排除 Defender | `logs/` | 日誌寫入加速 | 低 | ✓ |
| 3 | 程序優先級 AboveNormal | 系統程序 | 凍結減少 | 低 | ✗ |
| 4 | FlapsErpData/ 排除 Defender | `FlapsErpData/` | AP 資料讀取加速 | 中 | ✓ |
| 5 | ReportTemp/ 排除 Defender | `ReportTemp/` | 報表 I/O 加速 | 低 | ✓ |
| 6 | Pos2Log/ 排除 Defender | `Pos2Log/` | POS 日誌加速 | 低 | ✓ |
| 7 | 排除程序掃描 | 系統 | 所有檔案操作免掃描 | 中 | ✓ |
| 8 | Search Index 排除 | 系統 | 背景 I/O 減少 | 零 | ✓ |
| 9 | TCP Auto-Tuning 確認 | 系統 | AP 回應 | 零 | ✓ |
| 10 | Task/ 過期清理 | `ReportTemp/Task/` | 磁碟空間 | 零 | ✗ |

---

## 第九部分：全域最優的連接

本文件完成**局部最優**（C:\SYSTEM_PROGRAM_DIRECTORY 內的微調）。

**下一步**：將局部最優結果帶入「FlapsErp系統互動精密調校指南_v1.0.md」，
與 Defender/瀏覽器/Office/Search/GPU/磁碟/網路等**全域互動**一起分析，
找出**全域最優解**——不是 FlapsErp 最快，而是**整台機器**最優。

### 擴展性（Scalability）原則

隨著 FlapsErp 功能模組增加（更多分頁、更多報表、更多 COM 呼叫）：

| 擴展 | 對局部設定的影響 | 對全域互動的影響 | 應對 |
|---|---|---|---|
| +1 分頁 | +1 renderer（40–100 MB） | Memory Compression 增加影響其他程式 | 關閉不用的分頁 |
| +1 DLL 更新 | Module/ 大小增加 | Defender 排除範圍仍有效 | .ver 比對確認更新 |
| +1 報表模板 | Cache/ 增加 | 不影響全域（快取有助效能） | 保留 |
| +1 日誌天數 | logs/ 增加 | Defender 排除仍有效 | 原廠自動輪替 |
| +1 使用者 | RAM 競爭加劇 | 全域互動複雜度增加 | 回到系統互動指南重新分析 |

**目標**：FlapsErp 的 scope 擴大時，效能不下降、其他程式不受影響——這靠的不是「改更多 config」，而是**維持全域互動的正向平衡**。

---

*本文件範圍：C:\SYSTEM_PROGRAM_DIRECTORY 內的局部微調。
全域最優見「FlapsErp系統互動精密調校指南_v1.0.md」。
兩份文件互相關聯：局部最優是全域最優的基礎；全域最優驗證局部最優的有效性。*