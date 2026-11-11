# FlapsErp 最佳設定基線 v1.0

> 文件編號：OPS-2026-1111-001｜版本：v1.0｜日期：2026-11-11｜狀態：Active
> 範圍：FlapsErp v0.0.4 桌面產品（`C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp`）＋共用 Windows 宿主＋與 FlapsErp 直接相接的 Wellfit 批次。
> 紅線：**不得寫入 `C:\SYSTEM_PROGRAM_DIRECTORY` 的原廠元件**（config.ini 由 IT 變更管制）；不得關閉 AV／UAC／WDAC；不得以 FlapsErp 效能為由拖垮其他程式。
> 事實標記：✔ 2026-11-11 唯讀實測｜△ 待實機驗證｜⚠ 需 IT／原廠確認。
> **軟體邊界**：FlapsErp（原廠）｜Wellfit（自建）｜Windows 宿主（微軟/IT）。本文件 §1–§3 為原廠 FlapsErp／Windows；§5 工作台為自建 Wellfit；`FlapsErpData` 僅唯讀。

---

## 1. 產品層設定（原廠 FlapsErp；`.conf\config.ini`）

**實機路徑**：`C:\SYSTEM_PROGRAM_DIRECTORY\.conf\config.ini`（✔）。
**完整內容（2026-11-11 實測）**：

```ini
IGNORE_CERT_ERR=1
KEEP_MACHINE_ID=1
DISABLE_HARDWARE_ACCELERATION=0
DEBUG_FASTREPORT=0
FORCE_DEVICE_SACLE_FACTOR=0
LANG=zh_TW
KEEP_MACHINE_ID_MIGRATED=1
```

| 鍵 | 現值 | 目標 | 說明 |
|---|---|---|---|
| `IGNORE_CERT_ERR` | 1 | 1（條件式→0） | 預設 AP 為 HTTP，此鍵對生產無作用；測試 AP TLS 已實測 CERT-VALID；其他 HTTPS 端點盤點後才可 0 |
| `KEEP_MACHINE_ID` | 1 | 1 | 授權綁定 |
| `DISABLE_HARDWARE_ACCELERATION` | 0 | 0（A/B 後定案） | 關閉 GPU 可能把渲染推回 CPU，影響其他程式；需系統級 CPU A/B |
| `DEBUG_FASTREPORT` | 0 | 0 | 報表除錯用 |
| `FORCE_DEVICE_SACLE_FACTOR` | 0 | 0 | 原廠拼字鍵；UI 裁切才評估 |
| `LANG` | zh_TW | zh_TW | 語言 |
| `KEEP_MACHINE_ID_MIGRATED` | 1 | 1 | 機器 ID 遷移 |

> 只有 `DISABLE_HARDWARE_ACCELERATION` 與 `FORCE_DEVICE_SACLE_FACTOR` 與效能/渲染直接相關；其餘不要為了「看起來有調過」而改。

## 2. AP／網路／TLS

| 項目 | 實測/目標 | 證據 |
|---|---|---|
| 預設 AP | `8E2EDBE64B3F0747B90EC071E4548BE8`（大前集團 WELLFIT）HTTP `210.64.124.59:10578` | `-Probe settings` ✔ |
| 備援 AP | `54B36E18…`（WELLFIT-1）HTTP；不設預設 | `-Probe settings` ✔ |
| 測試 AP | `241C394A…` HTTPS `test2.flaps.tw:10113`；**CERT-VALID**（Let's Encrypt、TLS 1.2、到期 2027-01-21） | `-Probe tls` ✔ |
| `IGNORE_CERT_ERR` | HTTP 下 1；HTTPS 端點盤點完成後可 0 | §1 |
| AP 轉 HTTPS | 要求原廠正式憑證；必要時 IT 安裝內部 Root CA | RSK-E2/E10 |
| TLS 1.0/1.1 | 不主動停用 | 舊 OpenSSL 相依 ⚠ |
| 防火牆 | 白名單 `210.64.124.59:10578`；測試 AP 生產不可達 | IT |

> `app.asar` 掃描：無其他 `https://...flaps/wellfit...` literal 端點；更新/授權若由 config/變數提供，需 runtime 監控。

## 3. Windows 宿主設定（微軟/IT；系統級；先做 FlapsErp-only）

> Scope：`F`=FlapsErp-only；`S`=system-wide；`C`=conditional。任何 `S`/`C` 變更需 IT 核准＋系統級 A/B（§6.4），不得降低其他程式效能。
> **本機實測（2026-11-11）**：Windows 11 Home、Ryzen 7 4800HS 8C/16T、15.79 GB RAM（3.4–5.0 GB free，取樣）、CPU 7–73%（取樣；受其他程式/本工具影響）、磁碟佇列 0–1（取樣）、溫度 UNKNOWN；同時跑 Code/ChatGPT/Chrome/Outlook。因此本機預設：Balanced 電源、system-managed 分頁檔、保留快速啟動/USB/NIC/GPU/視覺/搜尋；只做 FlapsErp-only＋AV 排除（IT 核准）＋AP 白名單。

| 類別 | 設定 | Scope | 目標 | 對其他程式的影響 |
|---|---|---|---|---|
| 電源 | 電源計畫 | S/C | 桌機/RPA 站 High performance；筆電平衡＋AC 不睡 | 電力/熱/風扇；監控溫度 |
| 電源 | 睡眠/休眠/關閉顯示器 | S/C | RPA 站 AC 全部 0 | 全機電源 |
| 電源 | 快速啟動 | C | 保留預設；僅 DLL/驅動狀態問題時停用 | 所有使用者開機變慢 |
| 電源 | USB 選擇性暫停 | C | 僅裝置斷線時停用 | 全機 USB 供電 |
| 桌面 | 螢幕保護/鎖定 | S | 依 IT 安全政策；RPA 站不鎖定 | 安全 vs 自動化 |
| 記憶體 | RAM | S | ≥16 GB（若低於才加） | 全機受益 |
| 記憶體 | 分頁檔 | C | 預設 system-managed；實測抖動才固定 | 固定大小可能限制其他程式 |
| 儲存 | 磁碟 | S | SSD/NVMe；可用 ≥15% | 全機 I/O 受益 |
| 儲存 | Defrag/Trim | S | SSD Defrag 關；定期 ReTrim | 全機受益 |
| 儲存 | NTFS Last Access | C | 預設；僅 I/O 壓力實測時關 | 影響備份/完整性工具 |
| 安全 | Defender 排除 | S | 最小排除 `FlapsErp`/`Module`/`FlapsErpData` | 安全面下降；不影響效能 |
| 安全 | WDAC/AppLocker | S | 確認允許；不改政策 | 安全/政策 |
| 安全 | UAC | S | 保持預設 | 不可關閉 |
| 安全 | PowerShell 執行原則 | S | `Restricted`/`AllSigned` | 不可 Unrestricted |
| 時間 | 時間/時區 | S | NTP/`Taipei Standard Time` | 全機受益 |
| 網路 | NIC 省電 | C | 桌機關閉；筆電保留 | 全機網路/電力 |
| 驅動 | GPU 驅動 | C | 現有驅動有問題才更新 | 其他 GPU 程式可能回歸 |
| 索引 | Windows Search | C | 非必要不做；若做只排除安裝目錄 | 其他程式搜尋 |
| 視覺 | 視覺效果 | C | 非必要不做；保留字型平滑 | 全機外觀 |
| 記憶體 | `IncreaseUserVA`/PE LAA | C | **LAA 已實測 True**；x64 下 4 GB，不需 `IncreaseUserVA` | 32-bit 位址空間 |

## 4. 啟動／分頁／程序衛生（FlapsErp-only，不影響其他程式）

| 項目 | 做法 | 證據 |
|---|---|---|
| 單一實例 | 啟動前檢查 `Get-Process FlapsErp`；重複啟動先提示，不自動 kill（除 IT 腳本） | △ |
| 多餘分頁 | 收工前關閉未使用 readonly tabs；單一主視窗 | △ |
| DLL 註冊 | 一次性註冊；保持 `.ver` 穩定；AV 排除避免隔離 | △ |
| 夜間重啟 | 若 8h soak 顯示洩漏才排程；02:00 或批次後 | △ |
| `processed.ini`/`.ver` | 啟動前後比對；若每次變動→DLL 重註冊 | `-Probe settings` |
| 下載資料夾 | 指向 SSD（若目前非 SSD） | △ |

## 5. 外部工作台（自建 Wellfit；最大效益；繞開原始碼瓶頸）

| 工作台 | 做法 | 繞開 |
|---|---|---|
| A1 檔案通道 | `flapserp://` 深層連結；CSV 匯入/匯出 | `sendSync` UI 操作 |
| Wellfit 批次 | 清洗/驗證/修復/彙總在 Wellfit CLI | 客端 CPU/記憶體 |
| 報表外移 | 匯出成檔→Wellfit 解析→零 JS 看板 | 客端報表渲染 |
| 大量 COM | 外部 Rust COM bridge 批次（EULA/Phase 3） | `sendSync` |

### 5.1 批次資源守門（避免拖垮其他程式）
- **優先另機**：Wellfit/MTRIX/報表處理在獨立批次機。
- **同機時**：02:00–05:00 離峰；`BelowNormal`；必要時 CPU affinity；RAM 上限；可中止。
- **守門**：CPU>80% 或 RAM>85% 持續 5 min → 中止；磁碟佇列>2.0 持續 5 min → 延後。
- **其他程式**：批次前後跑代表程式（瀏覽器/Office/檔案總管）；任一退步 >10% → 中止。

## 6. 監控／備份／更新（自建工具＋IT）

| 項目 | 頻率 | 工具 |
|---|---|---|
| `flaps-perf.ps1 -Probe all` | 每日 09:00/14:00/18:00 | 既有腳本 |
| `-Probe app`（menu/tab） | 每日 | 既有腳本 |
| `-Probe com` | 每週 | 既有腳本 |
| `-Probe cdp` | 每月/更新後 | 既有腳本 |
| `-Probe settings`/`tls`/`pe` | 每季/更新後 | 既有腳本 |
| 系統 CPU/RAM/磁碟/溫度 | 每日/批次期間 | `Get-Counter`/`MSAcpi_ThermalZoneTemperature` |
| ledger 備份 | 每日 | `Wellfit/scripts/daily-backup.ps1` |
| config 備份 | 每週/變更前 | `Wellfit/artifacts/config-backup/<date>/`＋SHA256 |
| 更新 | 每次原廠更新 | 離峰；更新後重跑 `-Probe full`＋`.ver` 比對 |

## 7. 驗收

| 碼 | 項目 | 條件 |
|---|---|---|
| AC-S01 | 7 鍵與實機一致 | `-Probe settings` 逐字比對 |
| AC-S02 | GPU/DPI 決策有證據 | 兩臂各 5 筆；無崩潰 |
| AC-S03 | TLS 決策有證據 | `-Probe tls`＋HTTPS 端點盤點 |
| AC-S04 | OS 設定可驗證 | `powercfg`/`Get-MpPreference`/`fsutil`/`Get-ScheduledTask` 前後輸出 |
| AC-S05 | 啟動/選單改善 | menu ready 改善 ≥10%（≤15.4 s）；-20% 為理想值 |
| AC-S06 | 記憶體改善 | RAM median ≤41.6 MB（-10%） |
| AC-S07 | 無效能退步 | 所有指標 ≤ 基線 +10% |
| AC-S08 | 8h 穩定 | RAM 增長 <50 MB；無崩潰 |
| AC-S09 | 文件與實機一致 | 手冊/報告爭議已結 |
| AC-S10 | 外部工作台驗證 | A1/Wellfit/報表端到端 |
| AC-S11 | 系統級不退化 | 代表程式/CPU/RAM/磁碟/溫度 ≤ 基線 +10%；批次守門有效 |

## 8. 回滾

| 變更 | 回滾 |
|---|---|
| `config.ini` | 從 `Wellfit/artifacts/config-backup/<date>/` 還原＋SHA256；重啟 |
| OS 設定 | IT runbook 記錄原值；逐項還原 |
| AP 設定 | 還原 `managerLayout.json`/AP JSON（若變更）；預設不變 |
| 排程 | `Unregister-ScheduledTask` |
| 外部工作台 | 停用排程/feature flag；保留 ledger |
| 文件 | 標 `SUPERSEDED`＋archive |

## 9. 變更紀錄

| 版本 | 日期 | 變更 |
|---|---|---|
| v1.0 | 2026-11-11 | 初版：7 鍵、AP/TLS、宿主、衛生、工作台、監控、驗收、回滾；實測 LAA=True、測試 AP CERT-VALID、AP 版本漂移 |

## 10. 證據與未決

- 實測命令：`powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\perf\flaps-perf.ps1" -Probe settings`（另 `app`／`tls`／`pe`／`host`／`endpoints`／`full`）。
- Artifacts：`Wellfit/artifacts/perf/<ts>/perf-result.json`、`perf-summary.md`、`system-baseline.json`、`endpoints.json`。
- 未決：IT 變更所有者/測試機；其他 HTTPS 端點盤點；批次主機（另機/同機）；8h soak；`FORCE_DEVICE_SACLE_FACTOR` 正確鍵行為。
- 本基線僅規劃/記錄；任何寫入 `C:\SYSTEM_PROGRAM_DIRECTORY` 由 IT 變更管制執行；UNKNOWN ≠ PASS ≠ FAIL。