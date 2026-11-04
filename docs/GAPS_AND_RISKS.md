# 系統認知存證、資訊缺口與演進風險報告
## Epistemic Evidence, Information Gaps & Risk Matrix

> **文件路徑**: `docs/GAPS_AND_RISKS.md`  
> **所屬體系**: Antigravity 2.0 Sovereign Evidence Assurance  
> **當前狀態**: 進行中 (Active) | **更新時間**: 2026-10-29

---

## 🔍 1. 已驗證代碼實相清單 `[FACT:CODE]`

以下內容已經由實際讀取磁碟代碼與設定檔確認無誤：

| 實相項目 | 具體代碼/檔案依據 | 驗證結論與影響 |
| :--- | :--- | :--- |
| **Rust 2021 Workspace 結構** | `Cargo.toml#L1-L8` | 確認分為 `core`, `platform`, `web` 三大 member，架構層次明確。 |
| **Tokio 異步與 Axum 服務** | `Cargo.toml#L12-L13` | 採用 `axum 0.7` 與 `tokio 1.0 (full)` 作為高併發微服務底座。 |
| **Sqlx SQLite 持久化** | `Cargo.toml#L16` | 採用 `sqlx 0.8` (SQLite 驅動，具備 runtime-tokio 與 macros)。 |
| **Windows COM RPA 支援** | `Cargo.toml#L28` | 引入 `windows 0.58` (Win32_Foundation & Win32_System_Com)，證明具備原生 Windows 桌面自動化能力。 |
| **Tauri 2.0 桌面集成** | `Cargo.toml#L19` | 客戶端支援跨平台桌面原生視窗。 |
| **發布編譯最佳化** | `Cargo.toml#L36-L42` | `opt-level = 3`, `lto = true`, `panic = "abort"`，追求極限二進位效能與小體積。 |

---

## 🧩 2. 關鍵資訊缺口與阻塞清單 `[GAP]`

以下項目在現有儲存庫中尚未找到直接的代碼或規格證據，需由團隊進一步對焦確認：

1. **`[GAP-01]` 輔翼 ERP 確切版本與登入 Session 復用機制**：
   - *問題*：當前代碼是否支援背景無頭（Headless）操作輔翼，還是必須依賴使用者桌面處於登入解鎖狀態？
   - *解鎖所需證據*：`core/` 或 `platform/` 中調用輔翼 COM GUID 或視窗類名（WindowClass）的原始碼片段。
2. **`[GAP-02]` 蝦皮 Shopee API 授權與 Token 刷新機制**：
   - *問題*：蝦皮是透過官方 Open API（OAuth2 Token）串接，還是透過爬蟲/下載報表方式處理？
   - *解鎖所需證據*：蝦皮認證模組的實作檔案或設定檔範本。
3. **`[GAP-03]` 哈瓦士 MTRIX 56 欄勾稽規則之法定定義**：
   - *問題*：MTRIX 勾稽失敗時的容錯與人工介入審計流程為何？
   - *解鎖所需證據*：勾稽演算法規格文件或相關單元測試案例。

---

## ⚠️ 3. 系統級潛在風險矩陣 (Risk Matrix)

| 風險項目 | 觸發條件 | 潛在影響 | 建議防禦策略 (SOTA) |
| :--- | :--- | :--- | :--- |
| **Windows 螢幕保護/鎖定阻斷** | 凌晨排程執行時，Windows 處於螢幕鎖定 (LockScreen) | Win32 GUI 找不到視窗焦點，RPA 作業全面失敗 | 透過 Windows Service 專屬 Session 0 隔離或桌面保活守護進程。 |
| **SQLite WAL 檔案膨脹** | 長時間讀寫未觸發 Checkpoint | `.db-wal` 檔案急劇增大，查詢延遲飆升 | 在 SRE 維運定時排程中加入 `PRAGMA wal_checkpoint(TRUNCATE)`。 |
| **COM 物件循環引用洩漏** | Rust 未正確實現 `Drop` 釋放 COM 引用計數 | 記憶體與 Windows GDI/User 句柄洩漏，最終導致 `OS Error 5` | 強制以 RAII 模式封裝 COM 智慧指針，單元測試覆蓋句柄計數檢查。 |

