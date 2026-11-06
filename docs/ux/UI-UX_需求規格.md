# UI/UX 需求規格（精美、智慧、易用、智能化）

> 版本：v1.0｜日期：2026-11-06｜狀態：草案
> 適用：Flaps ERP 表現層（web/、platform/templates/）
> 相依：docs/knowledge-hub/手冊-A-F.md §E；docs/product/PRD.md FR-001…FR-004、NFR-002／NFR-005

---

## E1 設計原則（零 JavaScript First）

| 原則 | 說明 | 執行標準 |
|------|------|----------|
| 零 JavaScript | 瀏覽器只接收 HTML、CSS、SVG、圖片與字體；不載入應用程式 JavaScript | `<script>` 數 = 0；無內聯事件處理器；HTMX／Datastar／WASM 不算零 JS |
| Design Token | 樣式一律使用設計 Token；禁止內聯硬編碼色彩／間距／字體 | `web/assets/design-system.css` 唯一來源 |
| 分區切換 | 用 `:target` 實作頁面區塊切換 | 無 JS 路由；URL `#` 錨點驅動 |
| 導航高亮 | 用 `:has()` 實作目前區塊高亮 | CSS 選擇器；無 JS 監聽 |
| 互動回饋 | 只用 CSS 實作（`:hover/:focus-visible/:user-invalid`） | 無 JavaScript 事件監聽 |
| 動畫 | 優先 `transform`/`opacity`；支援 `prefers-reduced-motion` | 無佈局抖動（layout shift） |
| 語言 | 介面文字預設繁體中文（台灣情境） | 所有使用者可見文字皆為繁體中文 |

---

## E2 元件狀態矩陣

每個互動元件必須具備以下 7 種狀態：

| 狀態 | 觸發條件 | 視覺表徵 | 鍵盤行為 |
|------|----------|----------|----------|
| Default | 未互動 | 設計系統預設色彩與間距 | 可 Tab 焦點 |
| Hover | 滑鼠懸停 | `transform`/`opacity` 微動（非佈局變動） | N/A |
| Focus | 鍵盤聚焦（Tab 到達） | 可見焦點環（2px，對比度 ≥ 3:1） | Enter/Space 觸發 |
| Disabled | 權限不足或欄位鎖定 | 灰階；`pointer-events: none`；無焦點 | 被跳過 |
| Loading | 資料載入中（>300ms） | 骨架畫面或 CSS 動畫轉圈 | 無互動 |
| Empty | 查詢結果為空 | 「—」或空狀態圖示；文字說明 | 可嘗試新搜尋 |
| Error | 操作失敗 | 問題＋原因＋處理方式；可重試按鈕 | Enter 重試 |

### E2.1 錯誤訊息三要素

每個錯誤訊息必須包含：

1. **問題**：什麼出了問題（簡潔，不超過 20 字）。
2. **可能原因**：為什麼會發生（技術或業務原因）。
3. **處理方式**：使用者應採取的具體動作。

範例：
- ❌ 「錯誤代碼 910071」
- ✅ 「登入失敗（可能原因：帳號或密碼不正確、帳號已停用）；請重新輸入，或聯絡系統管理員。」

範例：
- ❌ 「金額格式錯誤」
- ✅ 「金額格式不正確（可能原因：含非法字元或多次小數點）；請輸入純數字，例如 1234.50」

---

## E3 頁面規格

### E3.1 登入頁（/login）

| 元件 | 狀態 | 內容 |
|------|------|------|
| 帳號輸入框 | Default／Hover／Focus／Disabled／Error | placeholder「請輸入帳號」 |
| 密碼輸入框 | Default／Hover／Focus／Disabled／Error | placeholder「請輸入密碼」；type=password |
| 登入按鈕 | Default／Hover／Focus／Disabled／Loading | 文字「登入」；Disabled 時顯示「處理中…」 |
| 錯誤訊息 | Error | 問題＋原因＋處理方式 |
| 連結 | Default／Hover／Focus | 「忘記密碼？聯絡系統管理員」 |

### E3.2 儀表板（/）

| 元件 | 狀態 | 內容 |
|------|------|------|
| KPI 卡片（4 個） | Default／Loading／Empty／Error | 交易總筆數／交易總金額／啟用科目數／待覆核交易；無資料顯示「—」 |
| 最近交易表單 | Default／Loading／Empty | 表單：交易編號／摘要／日期／金額／狀態；無資料顯示「尚無交易紀錄」 |
| 月報長條圖 | Default／Loading／Error | 零 JS SVG；借方／貸方並列；軸標籤可見 |

### E3.3 新增交易（/new）

| 元件 | 狀態 | 內容 |
|------|------|------|
| 借方科目 | Default／Hover／Focus／Disabled／Error | 下拉選單；預設第一個啟用科目 |
| 貸方科目 | Default／Hover／Focus／Disabled／Error | 下拉選單；預設不同於借方 |
| 金額 | Default／Hover／Focus／Disabled／Error | 數字輸入；小數點最多 2 位；負數拒絕 |
| 摘要 | Default／Hover／Focus／Disabled | 文字輸入；最多 200 字 |
| 交易日期 | Default／Hover／Focus／Disabled | 日期選擇器；預設今日 |
| 提交按鈕 | Default／Hover／Focus／Disabled／Loading | 文字「建立憑單」；CSRF 權杖隱藏欄位 |
| 錯誤訊息 | Error | 欄位級：欄位名＋問題＋處理方式 |

### E3.4 交易清單（/transactions）

| 元件 | 狀態 | 內容 |
|------|------|------|
| 搜尋欄位 | Default／Focus | 交易編號／摘要關鍵字 |
| 分頁控制 | Default／Hover／Focus／Disabled | 上一頁／下一頁；首頁／末頁；Disabled 時不可點 |
| 表單列 | Default／Hover／Focus | 每列可點擊查看詳情 |
| 狀態標籤 | Default／Loading／Error | 0000 綠色／1000–1008 黃色／4000 紅色／9999 灰色 |

### E3.5 會計科目（/accounts）

| 元件 | 狀態 | 內容 |
|------|------|------|
| 科目樹 | Default／Loading／Error | 層級縮進；展開／摺疊（`details/summary`） |
| 啟用／停用切換 | Default／Hover／Focus／Disabled | 切換後 PRG 重載頁面 |
| 新增科目按鈕 | Default／Hover／Focus／Disabled | 開啟新增表單 |

### E3.6 AI 決策（/ai-decisions）

| 元件 | 狀態 | 內容 |
|------|------|------|
| 風險分顯示 | Default／Loading／Error | 0–1 進度條；<0.3 綠色／0.3–0.7 黃色／≥0.7 紅色 |
| 覆核按鈕 | Default／Hover／Focus／Disabled | 「人工覆核」；AI 禁區操作顯示「不可自動化」 |
| 建議卡片 | Default／Empty | 建議內容＋來源規則；來源不可隱藏 |
| 接受／拒絕 | Default／Hover／Focus／Disabled | 雙按鈕；PRG 後重載頁面 |

### E3.7 介面狀態（/states）

| 元件 | 狀態 | 內容 |
|------|------|------|
| 元件狀態展示 | Default／Hover／Focus／Disabled／Loading／Empty／Error | 所有互動元件的狀態範例 |
| 設計系統連結 | Default／Hover／Focus | 指向 `web/assets/design-system.css` |

---

## E4 無障礙（WCAG 2.2 AA）

| 準則 | 要求 | 驗證方法 |
|------|------|----------|
| 語義 HTML | 使用 `<button>`／`<a>`／`<form>`／`<label>` | 程式碼審查 |
| 可見焦點 | 所有互動元件有可見焦點環（2px，對比度 ≥ 3:1） | 鍵盤導航測試 |
| 鍵盤導航 | 所有功能可僅用鍵盤完成 | Tab 順序測試 |
| 不以顏色為唯一傳達 | 錯誤狀態需配合圖示或文字 | 灰階模式檢查 |
| prefers-reduced-motion | 動畫在偏好減少動態時停用 | 瀏覽器設定測試 |
| 對比度 | 文字對比度 ≥ 4.5:1；大文字 ≥ 3:1 | 色對比分析工具 |

---

## E5 驗收標準（DoD）

| 項目 | 驗收條件 | 驗證方式 |
|------|----------|----------|
| 零 JavaScript | `<script>` 數 = 0；無內聯事件 | 瀏覽器原始碼檢查；Playwright |
| Design Token 一致性 | 所有色彩／間距／字體來自 Design Token | 程式碼審查；CSS 審計 |
| 元件狀態完整 | 每個互動元件具備 7 種狀態 | UI 元件清單核對 |
| 錯誤訊息三要素 | 每個錯誤訊息含問題＋原因＋處理方式 | 使用者測試 |
| WCAG 2.2 AA | 鍵盤導航；可見焦點；語義 HTML；不以顏色為唯一傳達 | Playwright 無障礙掃描；人工鍵盤走查 |
| 無假資料 | 未知值顯示「—」；不以範例數字冒充 | UI 審查（FR-005 證據） |

---

*本規格與 `docs/knowledge-hub/手冊-A-F.md` §E 保持同步。*
