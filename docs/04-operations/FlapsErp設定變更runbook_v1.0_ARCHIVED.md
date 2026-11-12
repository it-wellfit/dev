# FlapsErp 設定變更 Runbook v1.0

> 文件編號：OPS-2026-1111-002｜版本：v1.0｜日期：2026-11-11｜狀態：Active
> 適用：FlapsErp v0.0.4（`C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp`）＋共用 Windows 宿主。
> 紅線：**代理唯讀 by default；使用者明示授權時可套用 config.ini（備份＋SHA256＋回滾）**；OS／二進位仍由 IT；不得關閉 AV／UAC／WDAC；不得降低其他程式效能。
> **軟體邊界**：`config.ini` 為原廠 FlapsErp（僅 IT 變更）；`FlapsErpData` 為原廠資料（唯讀）；`Wellfit\artifacts` 為自建產出（可寫）；OS 設定為 Windows 宿主（IT）。
> **平台**：FlapsErp 僅 Windows；本 runbook 僅適用 Windows；Wellfit Linux 批次另循 Linux runbook。

---

## 0. 快速 A/B（最常用：硬體加速）

> 只做 FlapsErp-only 的 `DISABLE_HARDWARE_ACCELERATION` A/B；不改其他設定。前提：無 cargo build（`link`/`cargo`/`rustc`）、app 執行中、備份完成、系統級 A/B 另行安排。

1. **Pre-flight**：`powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\perf\flaps-perf.ps1" -Probe full`；確認 `baseline.contended=False`、`host.contended=False`；若 `contended=True`，等建置結束。
2. **備份**：依 §2.2。
3. **Arm A**（目前 0）：重啟 FlapsErp；等 `menu.prepareColorMask`；5 次 `-Probe baseline`＋`-Probe app`；記錄 `OUT_DIR` 為 `<A>`。
4. **Arm B**：IT 逐行改 `DISABLE_HARDWARE_ACCELERATION=1`（§3）；重啟；5 次 `-Probe baseline`＋`-Probe app`；記錄 `OUT_DIR` 為 `<B>`。
5. **比較**：`powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\perf\flaps-perf.ps1" -Probe diff -BaselineDir <A> -CurrentDir <B>`。
6. **判定**：FlapsErp RAM/CPU/時間改善 ≥10% 且其他程式/系統指標 ≤10% → 可採 B；否則保留 A；溫度 >85°C/降頻 → 保留 A。
7. **還原/記錄**：採 B 則保留並記錄；否則 §3 還原 A；結果寫入 A/B 報告 §F/§G。

> 本機 2026-11-12 實測：`=1` 可啟動，但登入後 startup 31.473 s（vs `=0` 舊 session 15.674 s）；`=0` 新鮮登入後未測（login 頁；代理無法登入）；`=1` 未採用；config 已還原 0。`=1` 未證實有效，不得宣稱改善。

## 1. 角色

| 角色 | 可做 | 不可做 |
|---|---|---|
| 代理（本工具） | 唯讀量測、設定規格、文件、驗收 | 寫入原廠目錄、改 OS、重啟 FlapsErp |
| IT | 備份、套用、驗證、回滾 | 跳過備份／A/B |
| 使用者 | 回報症狀、確認 app smoke | 自行改 config |

## 2. 變更前（每次必做）

1. **公告**：變更時間、範圍、影響、回滾窗口。
2. **備份**：
   ```powershell
   $date = Get-Date -Format "yyyyMMdd_HHmmss"
   $src = "C:\SYSTEM_PROGRAM_DIRECTORY\.conf\config.ini"
   $dst = "Wellfit\artifacts\config-backup\$date"
   New-Item -ItemType Directory -Path $dst -Force | Out-Null
   Copy-Item -LiteralPath $src -Destination "$dst\config.ini" -Force
   Copy-Item -LiteralPath "C:\SYSTEM_PROGRAM_DIRECTORY\.conf\processed.ini" -Destination "$dst\processed.ini" -Force
   Copy-Item -Path "C:\SYSTEM_PROGRAM_DIRECTORY\.conf\*.version.txt" -Destination $dst -Force
   Copy-Item -Path "C:\SYSTEM_PROGRAM_DIRECTORY\Module\*.ver" -Destination $dst -Force
   Copy-Item -Path "C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErpData\AP\*.json" -Destination $dst -Force
   Get-ChildItem -LiteralPath $dst -File | ForEach-Object {
  $h = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash
  "$h  $($_.Name)"
} | Set-Content -LiteralPath "$dst\SHA256_MANIFEST.txt" -Encoding UTF8
   ```
   > 備份品質：`$dst` 用絕對路徑（或固定 repo 根為 workdir）；記錄檔案清單／大小／可讀／ACL；重要備份複製到另一受控位置；回滾前重算 SHA256。
3. **基線**：
   ```powershell
   powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\perf\flaps-perf.ps1" -Probe full
   ```
4. **系統基線**：記錄 CPU/RAM/磁碟/溫度、代表程式（瀏覽器/Office/檔案總管）回應、背景服務（Search/Update/Backup/Defender）。
5. **變更窗**：離峰（02:00–05:00）或維護窗；避免與備份/Windows Update/其他批次重疊。

## 3. 變更 A：`config.ini`（7 鍵）

1. 確認備份與 SHA256（§2.2）。
2. 逐行替換；保留 CRLF；UTF-8 無 BOM：
   ```powershell
   $lines = [System.IO.File]::ReadAllLines($src)
   # 只改指定鍵；不得重排其他行
   $newContent = ($lines -join "`r`n") + "`r`n"
   [System.IO.File]::WriteAllText($src, $newContent, (New-Object System.Text.UTF8Encoding($false)))
   ```
   > 先檢查原始位元組／換行／BOM／最後換行；只替換指定鍵；變更前後 diff／hex compare；不因重寫造成其他差異。`ReadAllLines`＋`WriteAllText` 會統一為 CRLF 並補尾換行，若原檔格式不同即非逐位元保留。
3. **A/B 硬體加速**：Arm A `DISABLE_HARDWARE_ACCELERATION=0` vs Arm B `=1`；每臂 5 次重啟；比較 FlapsErp RAM/CPU **與系統 CPU**；任一其他程式退步 >10% → 保留 0。
4. **`IGNORE_CERT_ERR`**：預設 1；測試 AP 已實測 CERT-VALID；其他 HTTPS 端點盤點完成後才可 0；生產 AP 為 HTTP，改 0 無作用。
5. **驗證**：`-Probe settings` 7 鍵逐字；`-Probe app`；app smoke（§6）。

## 4. 變更 B：Windows 宿主（僅 S/C；需 IT 核准＋系統級 A/B）

> **本機為行動平台（推測筆電）（Ryzen 7 4800HS / 16 GB / Win11 Home）**：預設不套用電源/快速啟動/USB/NTFS/視覺/搜尋/GPU 命令；只做 AV 排除（IT 核准）＋AP 白名單＋時間。若未來換桌機/RPA 站，才依表套用。

| 設定 | 命令（管理員） | 回滾 |
|---|---|---|
| 電源計畫 | `powercfg /list`；`powercfg /getactivescheme`；`powercfg /query`；`powercfg /setactive <GUID>` | `powercfg /setactive <原GUID>` |
| 睡眠/休眠/顯示器 | `powercfg /change standby-timeout-ac 0`；`hibernate-timeout-ac 0`；`monitor-timeout-ac 0`（AC/DC 分開記錄） | 記錄原值後還原 |
| 快速啟動 | 預設保留；僅必要時 `reg add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Power" /v HiberbootEnabled /t REG_DWORD /d 0 /f` | `reg add ... /d 1 /f` |
| USB 選擇性暫停 | `powercfg /setacvalueindex SCHEME_CURRENT 2a737441-1930-4402-8d77-b2bebba308a3 48e6b7a6-50f5-4782-a5d4-53bb8f07e226 0`；`powercfg /setactive SCHEME_CURRENT` | 原值 |
| NTFS last access | `fsutil behavior set disablelastaccess 1`（0=user-enabled,1=user-disabled,2=system-enabled/default,3=system-disabled；本機不套用） | `fsutil behavior set disablelastaccess 2`（預設；原值 0/3 則回 0/3） |
| Defender 排除 | `Add-MpPreference -ExclusionPath 'C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp'`（`Module`、`FlapsErpData` 同） | `Remove-MpPreference -ExclusionPath ...` |
| 時間 | `w32tm /query /status`；`/source`；`/peers`；`w32tm /resync`；`Set-TimeZone -Id "Taipei Standard Time"` | 原時區 |
| NIC 省電 | `Get-NetAdapterPowerManagement`；GUI 取消「允許電腦關閉此裝置」 | 原值 |
| 分頁檔 | 預設 system-managed；實測抖動才固定（`Win32_PageFileSetting`/GUI） | 原設定 |

> USB selective suspend 的 setting GUID 以 `powercfg /q SCHEME_CURRENT` 確認後再套用；`powercfg` GUID 一律以 `powercfg /list`／`/q` 實機值為準。
> Defender 排除：先量測是否為瓶頸；IT 資安核准；最小範圍；記錄核准人／期限；定期複查；**不得關閉 Defender**。
> 電源變更前 `powercfg /export "Wellfit\artifacts\power-before.pow" <原GUID>`；AC／DC 分開記錄。

> 電源/快速啟動/USB/NIC 影響全機；變更前後必須跑系統級 A/B（§6.3）；溫度 >85°C 或降頻即回滾。

## 5. 變更 C：AP／網路／TLS

1. 預設 AP 不變（`8E2EDBE…` HTTP）。
2. 防火牆白名單：
   ```powershell
   New-NetFirewallRule -DisplayName "FlapsErp AP HTTP" -Direction Outbound -RemoteAddress 210.64.124.59 -RemotePort 10578 -Action Allow -Profile Any
   ```
3. 測試 AP 生產不可達：`Resolve-DnsName test2.flaps.tw`；必要時 Block。
4. HTTPS/CA：測試 AP 憑證已有效；其他 HTTPS 端點盤點後再決定 `IGNORE_CERT_ERR`；必要時 `Import-Certificate -FilePath <ca.crt> -CertStoreLocation Cert:\LocalMachine\Root`（IT）。
5. 驗證：`Test-NetConnection 210.64.124.59 -Port 10578`；`-Probe tls`。
> 防火牆規則先確認 TCP／Profile／既有規則／rollback；`Test-NetConnection` 只證明 TCP 可達，不證明 HTTP／app login。

## 6. 驗證（自建工具 + 原廠 app）

### 6.1 FlapsErp
```powershell
powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\perf\flaps-perf.ps1" -Probe full
```
> `powershell -ExecutionPolicy Bypass` 不是安全邊界；執行前確認腳本來源／雜湊／權限／企業政策。
- `-Probe settings`：7 鍵、AP、manifest drift。
- `-Probe app`：module→menu、download/details、window/screen。
- `-Probe tls`：測試 AP 憑證。
- `-Probe pe`：LAA。
- `-Probe baseline`：11 行程 RAM。
- `-Probe hygiene`：FlapsErp 程序/角色/processed.ini/Module `.ver`/flapserp/windmill/Defender/ExecutionPolicy/AppLocker。
- `-Probe diff -BaselineDir <before> -CurrentDir <after>`：config/app/baseline/com/host 差異；環境指標標 `INFO`（非 FlapsErp 改善）。
- **CONTENDED 樣本**：`baseline.contended`／`com.contended`／`host.contended` 任一為 true（`link`/`cargo`/`rustc`、free<1 GB、diskQueue>5）→ 樣本無效；先結束建置或等離峰；不得用於 A/B。

### 6.2 App smoke（每次必過）
① 啟動→登入→選單就緒；② FAS 應付憑單→列表載入；③ HRS 員工→列表載入；④ 報表→匯出 CSV；⑤ 關閉→重啟→再登入；⑥ 若改 `IGNORE_CERT_ERR`/CA，加測 HTTPS AP 登入＋更新檢查。

### 6.3 系統級 A/B（S/C 必做）
- 併發負載：FlapsErp＋瀏覽器 10 tabs＋Excel 10 MB 開/存＋Outlook 收發＋1 GB 檔案複製。
- 指標：每個程式回應、系統 CPU/RAM/磁碟佇列/GPU/溫度。
- 判定：任一程式退步 >10% 或溫度降頻 → 回滾；溫度 UNKNOWN 不可視為通過；以 `-Probe diff` 比較前後 artifacts。

## 7. 回滾

| 變更 | 程序 | 驗證 |
|---|---|---|
| config | 從 `config-backup/<date>/` 還原＋SHA256；重啟 | 7 鍵；登入 |
| OS | 依 §4 表回滾原值 | `powercfg`/`Get-MpPreference`/`fsutil` 對照 |
| AP/防火牆 | 還原 `managerLayout.json`/AP JSON；`Remove-NetFirewallRule` | `Test-NetConnection` |
| 排程 | `Unregister-ScheduledTask` | 無殘留 |

## 8. 簽核

| 角色 | 名稱 | 日期 | 結果 |
|---|---|---|---|
| 備份/基線 |  |  |  |
| 變更套用 |  |  |  |
| 驗證 |  |  |  |
| 系統級 A/B |  |  |  |
| IT 資安 |  |  |  |
| 主管 |  |  |  |
| 批准 |  |  |  |

> 未量測不得標 PASS；`UNKNOWN ≠ PASS ≠ FAIL`。