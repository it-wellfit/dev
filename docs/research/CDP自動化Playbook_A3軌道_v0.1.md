# CDP/DOM 自動化 Playbook（A3 軌道模板）v0.1

> 對應藍圖 T-104 / ADR-003 A3 軌｜狀態：模板（現場驗證前不得用於生產）
> 紅線：EULA 政策書面確認（RSK-E1）前，本軌道僅 PoC；不寫入 `C:\SYSTEM_PROGRAM_DIRECTORY`（RSK-E5）

## 1. 探測結果（2026-11-03 首測）

| 項目 | 結果 |
|---|---|
| FlapsErp 執行中 | True（11 行程，總 RAM 623.8MB）|
| 埠 9222 `/json/list` | NOT-FOUND（預期：外殼未帶除錯埠啟動）|
| 探測工具 | `Wellfit/scripts/perf/flaps-perf.ps1 -Probe cdp`（可重跑）|

## 2. 五步驗證流程（11/10 偵察 session 執行）

1. **啟動帶埠測試**：關閉 FlapsErp → 以 `--remote-debugging-port=9222` 參數啟動（Electron 標準支援 ⚠）→ 重跑 cdp 探測器 → 記錄 FOUND/NOT-FOUND
2. **目標列舉**：若 FOUND，記錄 `/json/list` 回傳的頁面類型與標題（webview 分頁應以 `page`/`webview` 型別出現）
3. **唯讀觀察**：對目標頁開 CDP WebSocket → `Runtime.evaluate` 唯讀取 `document.title` 與 `location.host`（證明 DOM 可達）
4. **協定面確認**：記錄 connect 頁與 tabs 頁的 `preload` 腳本暴露面（`js/commonPreload.js` 2337 bytes，已提取於逆向分析）
5. **判定**：產出 Go/No-Go/Conditional → 回寫 ADR-003 與藍圖 T-104 狀態

## 3. 若 Go：自動化腳本骨架（Playwright connectOverCDP）

```javascript
// 僅 PoC；需 EULA 確認後才得納入生產排程
const { chromium } = require('playwright');
(async () => {
  const browser = await chromium.connectOverCDP('http://127.0.0.1:9222');
  const contexts = browser.contexts();
  // 列舉頁面 → 依 title/route 導航 → 表單填寫（參考 loadSaveForm.js 官方模式）
  // 完成後 disconnect，絕不關閉使用者視窗
  await browser.close(); // 僅斷開 CDP 連線
})();
```

## 4. 安全與合規閘門

| 閘門 | 條件 |
|---|---|
| G1 | RSK-E1（EULA）書面確認或正式接受 |
| G2 | 除錯埠僅綁 127.0.0.1（預設即如此 ✔）；不對外開放 |
| G3 | 腳本不得讀寫使用者工作區外路徑；輸出僅入 artifacts |
| G4 | 每次執行附 run-summary（時間、頁面、動作），可審計 |

## 5. 退讓鏈（若 No-Go）

CDP 不可用 → 退 A2 COM（flaps-bridge）→ 退 A1 檔案通道（永不阻塞）。
