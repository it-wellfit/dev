# 輔翼ERP效能優化操作手冊

**文件編號**：OPS-PERF-001  
**版本**：v1.0（2026-11-10）  
**狀態**：Active（現行）  
**目標讀者**：IT 管理人員、系統維運工程師、桌面自動化維運人員  
**適用範疇**：(A) 原廠 FlapsErp 桌面產品（`C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp`）；(B) 根工作區 Rust 平台（`core/`、`platform/`、`web/`、`crates/flaps_bridge`、`crates/kem`；自建）。兩者不得混淆。  
> **軟體邊界**：原廠 FlapsErp 的設定/二進位/資料在 `C:\SYSTEM_PROGRAM_DIRECTORY`（唯讀；config 僅 IT 變更）；自建 Rust 平台/Wellfit 在 `Dev\`（可改）。§3.2 為原廠 config；§2.2/§3.3 的 `platform/data/flaps.db`、`flaps-server`、Axum/Tokio 為自建平台，不得當成 FlapsErp。  
**相依文件**：  
- `docs/research/輔翼ERP效能基線報告_v1.0.md`（產品效能基線，2026-11-03）  
- `docs/research/FlapsErp平台線效能評估進度報告_v2.0.md`（平台線效能現況，2026-11-09）  
- `docs/research/FlapsErp產品線效能進度報告_v2.0.md`（產品線進度報告；附錄C＝config.ini 調優核對表，§3.2 之操作源）  
- `docs/research/輔翼ERP逆向分析與最佳化方案_v1.0.md`（逆向分析與優化方案，2026-11-03）  
- `docs/architecture/效能架構.md`（軟硬體槓桿與效能預算，2026-11-06）  
- `docs/architecture/DATABASE_DESIGN.md`（資料庫設計）  
- `docs/architecture/API_DESIGN.md`（API 路由契約，2026-11-09）  
- `docs/04-operations/troubleshooting-runbook.md`（緊急故障排除劇本）  
- `docs/engineering/FAULT_INJECTION_SOP.md`（故障注入 SOP）  
- `docs/encoding-standard.md`（編碼標準）  
- `docs/PLAN_REGISTER.md`（計畫登記冊，效力位階 SSOT）  

---

## 1. 前言與概述

### 1.1 撰寫目的

本手冊為「輔翼ERP」（FlapsErp）系統的 IT 管理人員與系統維運工程師編寫，目標是統一**效能優化操作**的執行流程與判斷標準，確保系統在生產環境下維持可預測的回應時間與資源使用率。

本手冊僅涉及**預防性維護與例行優化**，不包含功能開發或架構變更（後者請見 `docs/research/輔翼ERP優化多階段執行藍圖_v1.0.md`）。所有操作均已在根工作區（`Dev/`）實際驗證，並與 `docs/PLAN_REGISTER.md` 及各 ADR 決策保持同步。

### 1.2 系統技術基線

輔翼ERP 為基於 **Rust 2021 Workspace** 建構的企業級自動化平台，根工作區成員為 `core/`、`platform/`、`web/`、`crates/flaps_bridge`、`crates/kem`。主要技術棧與效能特性如下：

| 層級 | 技術 | 版本 | 用途 | 效能特性 |
|------|------|------|------|----------|
| 後端服務 | Axum | 0.7.x | REST API 與伺服器端渲染 | 非同步請求處理，適合高併發輕量請求 |
| 非同步運行時 | Tokio | 1.x | 異步 I/O 與執行緒池 | 多執行緒併發，需注意上下文切換開銷 |
| 持久層 | SQLx / SQLite | 0.9.x，WAL 模式 | 關聯式資料儲存 | 單一檔案資料庫，適合桌面環境；寫入順序化 |
| 桌面自動化 | Windows COM | 0.58 | 驅動輔翼 ERP 桌面介面（RPA） | **同步 COM 呼叫為效能瓶頸主因** |
| 表現層 | HTML / CSS / SVG | 零 JavaScript | Askama 0.16.x 伺服器端模板 | 無前端建置鏈，首次載入較快 |
| 設計系統 | Design Token | `web/assets/design-system.css` | 樣式集中管理 | 禁止內聯硬編碼色彩／間距／字體 |
| 知識證據 | KEM（crates/kem） | 內建 | SHA-256 哈希鏈證據鏈 | 寫入開銷較高，僅於關鍵交易啟用 |

### 1.3 何謂「效能優化」

在本系統中，**效能優化**是指在**不變更業務邏輯、不變更對外契約（API 路由與輸出格式）**的前提下，透過以下手段降低資源消耗並改善回應時間：

1. **資料庫層**：索引優化、WAL 檢查點排程、歷史資料封存、查詢計畫檢查。
2. **應用層**：熱路徑（Hot Path）程式碼優化、非同步併發度調整、靜態資源快取。
3. **桌面自動化層**：COM 呼叫批次化、減少同步阻塞、桌面焦點管理。
4. **作業系統層**：記憶體配置、磁碟 I/O 模式、併發連線數限制、桌面保活。
5. **監控與預警**：建立關鍵效能指標（KPI）的連續量測與異常偵測。

所有操作必須符合「可重現、可回滾、可驗證」三原則：變更前必須有基準測試結果，變更後必須重新驗證，且必須保留回滾路徑。

---

## 2. 效能監控與問題診斷

### 2.1 關鍵效能指標（KPI）

建議每日自動採集並儲存以下指標，以建立效能基準線（Baseline）：

| 指標類別 | 指標名稱 | 採集方式 | 建議閾值（警告） | 建議閾值（危險） |
|----------|----------|----------|------------------|------------------|
| 資料庫 | `transactions` 筆數 | `SELECT COUNT(*) FROM transactions` | — | > 100 萬筆需評估封存 |
| 資料庫 | WAL 檔案大小 | `Get-Item platform/data/flaps.db-wal` | > 500 MB | > 2 GB |
| 應用程式 | 月報查詢 P95 | `/reports/monthly` 端點計時 | > 500 ms | > 1 秒 |
| 應用程式 | 種子建置（100k 交易） | `cargo test -p platform --lib stress_10000` | > 5 秒 | > 10 秒 |
| 作業系統 | CPU 使用率 | Windows 效能監視器 / `Get-Process` | > 70% 持續 5 分鐘 | > 90% |
| 作業系統 | 記憶體使用率 | `Get-Process` | > 80% | > 95% |
| 作業系統 | 磁碟空閒 | `Get-Volume` | > 80% 已用 | > 95% 已用 |
| 自動化 | COM 呼叫延遲 | Win32 COM 呼叫計時 | > 1 秒/次 | > 5 秒/次 |

### 2.2 監控命令（PowerShell）

#### 2.2.1 資料庫與檔案層級（自建 Rust 平台；非 FlapsErp）

```powershell
# 檢查 SQLite 資料庫與 WAL 大小
Get-Item "platform/data/flaps.db" | Select-Object Name, Length
Get-Item "platform/data/flaps.db-wal" | Select-Object Name, Length

# 檢查服務程序
Get-Process flaps-server -ErrorAction SilentlyContinue | Select-Object Id, CPU, WorkingSet, StartTime

# 檢查磁碟空間
Get-Volume | Select-Object DriveLetter, FileSystemLabel, Size, FreeSpace
```

#### 2.2.2 執行品質閘門（自建 Rust 平台；本機）

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --locked --workspace --no-fail-fast
cargo audit
```

### 2.3 常見效能瓶頸診斷流程（自建 Rust 平台；非 FlapsErp）

當使用者回報「系統變慢」或監控觸發告警時，請依下列流程診斷：

```
開始：使用者回報效能異常 / 監控觸發告警
        │
        ▼
[步驟 1] 檢查系統資源（CPU / 記憶體 / 磁碟）
        │
        ├─ 是否超過閾值？ ──是──► [步驟 2] 檢查單一程序獨佔資源
        │                         │
        │                         ├─ 是 ──► [步驟 3] 確認是否為 flaps-server.exe
        │                         │                         │
        │                         │                         ├─ 是 ──► [步驟 4] 檢查資料庫查詢效能
        │                         │                         │
        │                         │                         └─ 否 ─► 終止無關程序，重新監控
        │                         │
        │                         否 ──► [步驟 5] 檢查網路與客戶端瀏覽器
        │
        └─ 否 ──► [步驟 6] 檢查特定業務流程（月報表、匯入批次）的回應時間
```

#### 常見瓶頸對照表

| 現象 | 可能原因 | 初步檢查方式 | 建議處置 |
|------|----------|--------------|----------|
| 全系統回應變慢 | CPU 飽和 / 記憶體不足 | 工作管理員 / 效能監視器 | 關閉無關程序；必要時擴充記憶體 |
| 報表查詢延遲 | SQLite 未建立適當索引 / WAL 膨脹 | `EXPLAIN QUERY PLAN` 檢查查詢計畫 | 建立索引；執行 `PRAGMA wal_checkpoint(TRUNCATE)` |
| 全系統無回應 | Win32 COM 線程鎖死（RPA 自動化） | 工作管理員檢查 `flaps-server.exe` | 重啟服務；檢查桌面鎖定狀態 |
| 僅特定報表慢 | 該 SQL 查詢未建立索引 | `EXPLAIN QUERY PLAN` 檢查該查詢 | 建立索引；優化查詢語句 |
| 記憶體持續增長 | 可能存在記憶體洩漏 | 長時間監控記憶體使用趨勢 | 重啟服務暫時緩解；提交開發團隊調查 |
| COM 呼叫延遲過高 | 逐筆同步調用 / 桌面鎖定 | `Get-Process` 檢查 `flaps-server.exe` | 改為批次化 COM 呼叫；確保桌面未鎖定 |

---

## 3. 核心優化操作步驟

### 3.1 資料庫優化（SQLite）

輔翼ERP 使用 SQLite WAL（Write-Ahead Logging）模式作為主要持久層。以下操作須在**維護時段**執行，並事先完成資料庫備份（見第 4 節）。

#### 3.1.1 檢查現有索引

```sql
-- 執行於 platform/data/flaps.db
.schema transactions
.schema transaction_items
.schema audit_log
```

#### 3.1.2 建立高頻查詢欄位索引

根據 `docs/architecture/DATABASE_DESIGN.md`，建議對高頻查詢欄位建立索引：

```sql
-- 依交易日期與狀態篩選（月報、報表常用）
CREATE INDEX IF NOT EXISTS idx_transactions_date_status 
    ON transactions(transaction_date, status);

-- 依交易代號快速查找
CREATE INDEX IF NOT EXISTS idx_transactions_code 
    ON transactions(transaction_code);

-- 依貸項目交易編號快速查找（借貸平衡檢查常用）
CREATE INDEX IF NOT EXISTS idx_transaction_items_transaction_id 
    ON transaction_items(transaction_id);
```

#### 3.1.3 驗證索引效果

使用 `EXPLAIN QUERY PLAN` 確認查詢使用索引而非全表掃描：

```sql
EXPLAIN QUERY PLAN 
SELECT * FROM transactions 
WHERE transaction_date = '2026-11-09' 
  AND status = 'POSTED';
```

預期結果應包含 `USE INDEX`，而非 `SCAN TABLE`。

#### 3.1.4 資料表統計（ANALYZE）

SQLite 依賴統計資訊選擇最佳查詢計畫。建議於重大資料異動後執行：

```sql
ANALYZE;
```

#### 3.1.5 WAL 檔案管理（關鍵）

SQLite 使用 WAL（Write-Ahead Logging）模式。WAL 檔案過大會影響讀取效能。

**檢查 WAL 大小：**
```powershell
Get-Item "platform/data/flaps.db-wal" | Select-Object Name, Length
```

**強制檢查點（Checkpoint）：**
```sql
PRAGMA wal_checkpoint(TRUNCATE);
```

> **[建議]** 於每日凌晨低峰期自動執行 `PRAGMA wal_checkpoint(TRUNCATE)`，避免 WAL 檔案無限增長。可透過 Windows 排程工作自動化此步驟。

#### 3.1.6 歷史資料封存策略

當 `transactions` 表超過 100 萬筆資料時，建議將較舊資料封存至獨立資料庫，僅保留最近 24 個月於主資料庫：

```sql
-- 建立封存資料庫
ATTACH DATABASE 'archive_2025.db' AS archive;

-- 將較舊資料移至封存資料庫
INSERT INTO archive.transactions SELECT * FROM transactions 
WHERE transaction_date < '2025-01-01';

DELETE FROM transactions 
WHERE transaction_date < '2025-01-01';
```

> **[注意]** 執行封存前必須先完整備份主資料庫（見第 4 節）。

---

### 3.2 應用程式配置調整（原廠 FlapsErp；config.ini）

> **更正（2026-11-11）**：本節原指向 `Dev\platform\config.ini` 與 `ORG/PERM/...` 項目，**與實機不符**。實機設定檔為 `C:\SYSTEM_PROGRAM_DIRECTORY\.conf\config.ini`，只有 7 個鍵；SSOT 見 `docs/04-operations/FlapsErp最佳設定基線_v1.0.md`。變更由 IT 於變更管制下執行，代理唯讀。
>
> **四方整合指引（RP-2；互鏈不重寫）**：①**操作源**＝產品線報告附錄C「config.ini 調優核對表」（`docs/research/FlapsErp產品線效能進度報告_v2.0.md`）；②**整合手冊**＝本節（§3.2）；③**量測法**＝效能架構 P4（`docs/architecture/效能架構.md`）；④**驗收指標源**＝RPA-ECOM 主文件 §1.1 G4 斷路器（連續失敗 5 次觸發、重試 ≤3、恢復 ≤5min）與 §10.2 監控/備份/回滾（`docs/management/FlapsErp_RPA_電商系統對接_專案文件與開發前準備報告_v2.1.md`）。另見 A1 檔案通道手冊（同目錄 `docs/04-operations/`）。

#### 3.2.1 修改前的必做步驟

1. **完整備份原始檔案：**
   ```powershell
   $date = Get-Date -Format "yyyyMMdd_HHmmss"
   $src = "C:\SYSTEM_PROGRAM_DIRECTORY\.conf\config.ini"
   $dst = "Wellfit\artifacts\config-backup\$date\config.ini"
   New-Item -ItemType Directory -Path (Split-Path $dst) -Force | Out-Null
   Copy-Item -LiteralPath $src -Destination $dst -Force
   ```
2. **計算 SHA256 雜湊並記錄：**
   ```powershell
   Get-FileHash -LiteralPath $src -Algorithm SHA256 | Select-Object Hash
   ```
3. **僅在備份完成後，才進行編輯（逐行替換、保留 CRLF、UTF-8 無 BOM）：**
   ```powershell
   $lines = [System.IO.File]::ReadAllLines($src)
   $newContent = ($lines -join "`r`n") + "`r`n"
   [System.IO.File]::WriteAllText($src, $newContent, (New-Object System.Text.UTF8Encoding($false)))
   ```

> **[強制規則]** 所有編輯必須使用 UTF-8 無 BOM；PowerShell 寫檔禁用 `>` 重導向；config 變更由 IT 變更管制執行（備份＋SHA256＋回滾），代理不得寫入 `C:\SYSTEM_PROGRAM_DIRECTORY`。
> **位元組層校正**：先檢查原始位元組／換行／BOM／最後換行；只替換指定鍵；變更前後 diff／hex compare。`ReadAllLines`＋`WriteAllText` 會統一 CRLF 並補尾換行，若原檔格式不同即非逐位元保留。
> 完整備份（`config.ini`／`processed.ini`／`*.version.txt`／`Module\*.ver`／`AP\*.json`）與還原命令見 `FlapsErp設定變更runbook_v1.0.md` §2.2／§7。

#### 3.2.2 可調整項目（實機 7 鍵）

| 設定項 | 現值 | 說明 | 建議操作 |
|--------|------|------|----------|
| `IGNORE_CERT_ERR` | 1 | TLS 憑證錯誤處理 | 預設 1；測試 AP 已實測 CERT-VALID；其他 HTTPS 端點盤點後可 0 |
| `KEEP_MACHINE_ID` | 1 | 保持機器 ID | 保持 1 |
| `DISABLE_HARDWARE_ACCELERATION` | 0 | 硬體加速 | 0；A/B 後定案 |
| `DEBUG_FASTREPORT` | 0 | 報表除錯 | 0；僅原廠指示時 1 |
| `FORCE_DEVICE_SACLE_FACTOR` | 0 | 縮放（原廠拼字） | 0；UI 裁切才評估 |
| `LANG` | zh_TW | 語言 | 保持 |
| `KEEP_MACHINE_ID_MIGRATED` | 1 | 機器 ID 遷移 | 保持 1 |

#### 3.2.3 修改後驗證

```powershell
powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\perf\flaps-perf.ps1" -Probe full
```

並執行 app smoke：① 啟動→登入→選單就緒；② FAS 應付憑單→列表載入；③ HRS 員工→列表載入；④ 報表→匯出 CSV；⑤ 關閉→重啟→再登入。任一失敗即回滾。

---

### 3.3 應用程式配置調整（自建 Rust 平台；非 FlapsErp）

#### 3.3.1 併發連線數

Axum/Tokio 採非同步模型，不建議過度提升同步執行緒數。於 `platform/src/main.rs` 或對應伺服器建立區段檢查 Tokio 執行緒池設定：

```rust
// 範例：Tokio 執行緒池設定
let runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)  // 建議等於 CPU 實體核心數
    .enable_all()
    .build()?;
```

> **[注意]** 過多執行緒會增加上下文切換開銷，未必提升效能。建議先以實體 CPU 核心數為基準，再逐步調整並量測。

#### 3.3.2 靜態資源快取

本系統採用伺服器端渲染（SSR），瀏覽器端無 JavaScript 快取。建議：
- 確認 Web 伺服器對靜態資源（CSS、圖片、SVG）啟用適當的 `Cache-Control` 標頭。
- 對於高頻讀取的靜態參考資料（如設計系統 `web/assets/design-system.css`），評估於應用層記憶體快取。

---

### 3.3 使用者與權限管理

過多啟用使用者帳戶與過於寬鬆的權限不只影響安全，也會增加登入驗證與權限檢查開銷。

**建議作業：**
1. 每季檢視一次使用者帳戶，停用超過 90 天未登入的帳戶。
2. 權限採用「最小權限原則」：僅授予該角色所需的最小權限，避免不必要地授予 `ROLE_MANAGE` 或系統管理權限。
3. 定期檢視 `roles` 與 `user_roles` 資料表，移除無效的關聯記錄。

```sql
-- 檢查未關聯任何使用者的角色
SELECT r.role_id, r.role_code 
FROM roles r 
LEFT JOIN user_roles ur ON r.role_id = ur.role_id 
WHERE ur.role_id IS NULL;
```

---

### 3.4 硬體與基礎架構建議（原廠 FlapsErp＋自建平台）

| 項目 | 建議規格 | 說明 |
|------|----------|------|
| CPU | 4 核心以上 | Axum/Tokio 非同步模型對單核心效能較敏感 |
| 記憶體 | 16 GB 以上（共用工作站） | FlapsErp 11 行程＋瀏覽器／Office；8 GB 僅能勉強，需關閉其他程式 |
| 儲存裝置 | SSD (NVMe) | SQLite 為檔案型資料庫，SSD 可顯著改善寫入與查詢效能 |
| 作業系統 | Windows 10/11 專業版或伺服器版 | 支援 Win32 COM 自動化（RPA 必需） |
| 備份 | 每日自動備份至獨立磁碟 | 防止資料遺失 |

> **[注意]** 輔翼ERP 的 RPA 功能依賴 Windows COM 元件，因此伺服器必須為 Windows 桌面環境，且需確保桌面不被自動鎖定（否則 RPA 操作會失敗）。
> **[共用環境]** 這台 Windows 同時跑瀏覽器／Office／其他程式；不得為了 FlapsErp 關閉 AV／UAC／WDAC，也不得讓批次/報表在營業時間搶 CPU／I/O。系統級變更見 `docs/04-operations/FlapsErp最佳設定基線_v1.0.md` §4.3／§4.10。

---

## 4. 安全與維護注意事項

### 4.1 備份先於任何變更

在執行以下任何操作前，必須先完成完整資料庫備份：

- 索引重建或資料表結構變更
- 歷史資料封存或刪除
- 應用程式版本升級
- 系統參數調整

**備份範例（PowerShell）：**
```powershell
$date = Get-Date -Format "yyyyMMdd_HHmmss"
Copy-Item "platform/data/flaps.db" "backup/flaps_db_$date.db"
Copy-Item "platform/data/flaps.db-wal" "backup/flaps_db_$date.db-wal"
Copy-Item "platform/data/flaps.db-shm" "backup/flaps_db_$date.db-shm"
```

### 4.2 常見風險與回復計畫

| 操作 | 風險 | 預防措施 | 回復計畫 |
|------|------|----------|----------|
| 索引建立 | 暫時鎖定資料表，寫入變慢 | 於低峰期執行 | 中斷索引建立，資料表仍可用 |
| 歷史資料封存 | 誤刪近期資料 | 備份後執行；先在測試環境驗證 | 從備份還原資料庫 |
| 應用程式升級 | 新版本導致效能退化 | 在預佈環境先行測試 | 回滾至前一版本；還原資料庫備份 |
| 硬體升級 | 資料遷移錯誤 | 完整備份；離線遷移 | 還原備份；檢查資料完整性 |

### 4.3 變更記錄

所有效能優化操作必須記錄於維護日誌，包含：
- 變更日期與執行人員
- 變更內容（例如：新增索引 `idx_transactions_date_status`）
- 變更前後的效能量測結果
- 發生問題與處置方式

---

## 5. 附錄

### 5.1 常用監控命令（PowerShell）

| 用途 | 指令 |
|------|------|
| 檢查 SQLite 資料庫大小 | `Get-Item "platform/data/flaps.db" \| Select-Object Name, Length` |
| 檢查 WAL 檔案大小 | `Get-Item "platform/data/flaps.db-wal" \| Select-Object Name, Length` |
| 檢查服務程序 | `Get-Process flaps-server -ErrorAction SilentlyContinue` |
| 檢查磁碟空間 | `Get-Volume \| Select-Object DriveLetter, FileSystemLabel, Size, FreeSpace` |
| 執行資料庫檢查點 | `sqlite3 platform/data/flaps.db "PRAGMA wal_checkpoint(TRUNCATE);"` |
| 執行全工作區品質閘門 | `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --locked --workspace --no-fail-fast`、`cargo audit` |

### 5.2 系統設定檔路徑

| 項目 | 路徑 |
|------|------|
| 主資料庫 | `platform/data/flaps.db` |
| 後端原始碼 | `platform/src/api.rs` |
| 設計系統樣式 | `web/assets/design-system.css` |
| 資料庫遷移腳本 | `platform/migrations/` |
| 品質閘門腳本 | `Knowledge/tools/dev-env/build.ps1` |
| 依賴安全記錄 | `.cargo/audit.toml` |

### 5.3 技術支援聯絡

| 項目 | 內容 |
|------|------|
| 系統開發團隊 | 內部開發團隊（見 `docs/PLAN_REGISTER.md` 負責人欄位） |
| 基礎設施支援 | 內部 IT 部門 |
| 廠商支援 | 輔翼 ERP 廠商（見 `docs/research/proposals/OQ-F1_F2_輔翼ERP正式需求提案_v2.1.md`） |

---

## 7. 修訂紀錄

| 版本 | 日期 | 變更內容 | 作者 |
|------|------|----------|------|
| v1.0 | 2026-11-10 | 初版發布：涵蓋效能監控、資料庫優化、應用程式配置（含 config.ini 環境調優）、使用者與權限管理、硬體建議、安全維護與附錄；內容與既有 `troubleshooting-runbook.md`、`效能評估報告 v2.0`、`效能架構.md`、`逆向分析與最佳化方案_v1.0.md` 互補，不重複。 | 系統工程團隊 |

---

> **重要提醒**：本手冊僅針對輔翼ERP系統（Rust/Axum/SQLite 架構）。如系統架構發生重大變更（例如遷移至 PostgreSQL 或新增微服務），必須同步更新本手冊，以免產生操作風險。所有操作必須先在非生產環境驗證，再應用於生產環境。
