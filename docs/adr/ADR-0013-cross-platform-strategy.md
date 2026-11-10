# ADR-0013: 跨平台交付策略（桌面殼、響應式 Web 與行動面）

- 狀態：Accepted
- 日期：2026-11-09
- 決策脈絡：計畫 `1793034142477-apihub-tauri-uiux-integration-plan.md`（T5–T7）、ADR-0012（E.Sun 邊界）、使用者欽定約束（零 JS 優先；提防 Electron 與耗效能 JS 工具鏈；效能／安全／可靠／穩定／易維護／易規模化／易演進）
- 實作證據：`web/src-tauri/tauri.conf.json`（殼載入平台伺服器＋NSIS）、`web/src-tauri/src/main.rs`（冪等啟動）、`platform/templates/*`（27+ 頁零 JS）

## 背景

工作區有兩件歷史作品：Tauri 桌面版玉山 API 平台（`web/src-tauri/`，24 個硬編碼 mock commands）與 Web 工作台（`platform/`，零 JS＋真實資料＋RBAC）。計畫 D1 已拍板「殼載入平台伺服器」；本 ADR 記錄跨平台交付的完整策略與框架選型（含 Tauri vs Electron 評估），以及行動面的誠實結論。

## 決策

### D1：桌面＝Tauri 殼載入平台伺服器（單一 UI 事實來源）
- WebView 指向 `http://127.0.0.1:8090/`（platform 伺服器）；殼負責冪等啟停（埠探測→spawn→就緒等待）與後續原生整合（檔案對話、深連結、自動更新）。
- 原 24 個 mock commands 全數移除（虛報資料違治理 P7）；V2.8 型別鏡射保留於 `state.rs`（金額 i64 minor，INV-01）。
- 理由：UI 不重寫、API 語意不複製；桌面與瀏覽器體驗逐位元組一致（同伺服器）。

### D2：Web＝響應式零 JS（現行基線即跨平台）
- axum＋Askama＋`design-system.css`（`include_str!` 內嵌）；任何現代瀏覽器可用，行動瀏覽器已由響應式佈局覆蓋。
- 不引入任何 JS 建置鏈（Node/npm/Vite）；Tauri `devUrl:1420`（Vite 殘留）已移除。

### D3：框架選型＝Tauri，不採 Electron
- Tauri：OS 級 WebView＋Rust 核心（12MB 級二進位）、無 Node 常駐、與 Rust 2024 workspace 憲政契合。
- Electron：捆綁 Chromium＋Node（100MB+ 足跡、JS 生態依賴樹），與「零 JS＋Rust 基線」衝突——**不採用**；任何 UI 遷往 JS 生態之提案需新 ADR 證明必要性。

### D4：行動面＝本輪不承諾，誠實評估
- 零 JS 響應式 Web 已可在行動瀏覽器使用（無需任何新工作）。
- 原生行動（Tauri Mobile iOS/Android）待產品需求明確後另啟 POC；本輪不寫程式碼、不開分支。
- 打包：Windows NSIS 先行（本輪）；Linux（AppImage/deb）／macOS（dmg）依需求擴張。

## 後果

- 正面：單一 UI 事實來源；桌面/瀏覽器/行動瀏覽器三面同源；足跡最小；憲政一致（零 JS、Rust、無新依賴）。
- 取捨：桌面離線能力受限（伺服器未啟動時殼顯示連線錯誤——v0.3 評估殼內嵌 axum 解離線）；行動原生體驗延後。
- 逆轉成本：低——殼僅是 URL 指向；伺服器遷移或換殼不影響 UI/API。

## 驗證證據（revision-bound）

- `tauri.conf.json`：`url=http://127.0.0.1:8090/`、`bundle.active=true`、`targets=["nsis"]`、CSP 無 `script-src 'unsafe-inline'`。
- `main.rs`：冪等啟動（埠探測→spawn→就緒等待 6s 上限）；無任何 mock 常量。
- `state.rs`：金額欄位 i64 minor（4 處 f64 修正）。
- （待辦）`tauri build` NSIS 產物驗證——列入下一切片（需 Rust Windows 工具鏈完整建置）。
