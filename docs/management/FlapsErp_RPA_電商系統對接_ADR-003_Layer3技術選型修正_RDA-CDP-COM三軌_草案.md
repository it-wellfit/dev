# ADR-003：Layer 3 自動化技術選型修正（FlaUI/UIA → RDA/CDP/COM 三軌）

> ADR-003｜狀態：Draft（待會簽，預定 2026-11-10 偵察 session 定稿）
> 日期：2026-11-03｜決策者：PMT/ARCH（單人治理 + QA 驗證）
> 關聯：藍圖 V1.0 Layer 3；RPA-ECOM v2.1 MOD 需求；`docs/research/輔翼ERP逆向分析與最佳化方案_v1.0.md` §1.3/§4
> 提案包：`docs/research/proposals/OQ-F1_F2_輔翼ERP正式需求提案_v1.0.md`（2026-11-03 送出，回覆期限 2026-11-19）

## 1. 背景（Context）

原藍圖 V1.0 規劃以 FlaUI/UIA 自動化「輔翼ERP 桌面 UI」（原 P0 偵察議程以 FlaUI 可行性為主軸）。2026-11-03 逆向工程（證據 E-01~E-07）證實：

- ERP 業務 UI 為 **web 內容**（Chromium webview 載入 JSP 伺服器頁面），非原生 Win32 控制項 ✔（main.js webviewTag、connectInfo.ap）
- 外殼存在 **官方自動化入口**：`flapserp://` 協定（base64 JSON）、RDAPortal.jsp XML RDA 協定（`com.flaps.Module.RDA.TGetModuleInfo` 已觀測）、COM（`FLAPSConsoleCOM.AXCXMLNode` + winax + Module\*.jsc）、Playwright（外殼自測用）✔
- FlaUI/UIA 對 web 內容僅能觸及 Chromium accessibility 橋：脆弱、低階、維護成本高 △

## 2. 選項（Options）

| 選項 | 說明 | 優點 | 缺點 |
|---|---|---|---|
| A | 維持 FlaUI/UIA 桌面自動化 | 不改計畫 | 對 web UI 脆弱；選擇器隨版面漂移；違反「最佳效率」目標 |
| B | **三軌並行（RDA > CDP > COM），檔案通道為底座** | 各取所長、層層退讓、全部走官方入口 | 三軌建置成本；RDA/CDP 需原廠或現場確認 ⚠ |
| C | 僅 CDP/DOM | 單一技術棧 | 無 RDA 時效能上限；COM 硬體場景（POS/BHT）無法覆蓋 |

## 3. 決策（Decision）

**採選項 B**，穩定性分級：
- **A1 檔案交換 + flapserp:// 啟動**（預設底座，零額外依賴，✔ 已可用）
- **A2 COM（flaps-bridge）**：處理序內互動/硬體場景；以 `.ver` 偵測防漂移；生產化前需 EULA 書面確認（RSK-E1）
- **A3 CDP/DOM**：僅用於無 API 的畫面流程；現場驗證通過才考慮（⚠）
- **A4 RDAPortal RDA 直連**：大量讀寫終極方案；原廠書面規格前僅 PoC（⚠，OQ-F1）
- **選擇器退讓規則**：能用檔案不用 API；能用 API 不用 COM；能用 COM 不用 CDP；**禁止** UIA 模擬點擊進入生產流程。

## 4. 影響（Consequences）

- **正面**：Layer 3 預估工作量大幅下降（原 FlaUI 偵察/開發轉為通道驗證）；穩定性提升（官方入口、契約測試可測）；與 Wellfit 冪等管線天然整合
- **負面**：需維護三軌代碼；RDA/CDP 有未驗證項（⚠）；EULA 邊界需書面化
- **中性**：原 60–90 分鐘偵察議程重寫（已入藍圖 T-201）

## 5. 驗收（Acceptance）

- [ ] A1 端到端試行通過（T-202，11/16 前）
- [ ] COM 批次 PoC 數據產出（T-102）且 Go/No-Go 判定完成
- [ ] CDP 探測現場驗證（T-201 session 內）
- [ ] EULA 自動化政策書面回覆（OQ-F1）或風險正式接受（RSK-E1 簽核）
- [ ] ADR-003 狀態改 Approved 並同步 RPA-ECOM v2.1 之 MOD 需求映射

## 6. 關聯

- 藍圖：`docs/research/輔翼ERP優化多階段執行藍圖_v1.0.md`（T-102/T-104/T-201/T-301/T-403）
- 風險：`docs/research/FlapsErp優化_合規風險追蹤表.md` RSK-E1/E2/E5
- 逆向證據：`docs/research/輔翼ERP逆向分析與最佳化方案_v1.0.md` §1.3（六通道）、§4（分級）
