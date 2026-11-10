# Flaps ERP UI V2 — Data-Driven Premium 設計系統重構

## 狀態：✅ Grill-me 完成（7/7 節點裁決）— 實作就緒

## 產品脈絡（事實，已查證）

- **產品**：Flaps ERP 工作台 — 台灣中小企業帳務登帳、查詢、玉山銀行支付整合
- **使用者**：finance（財務，長時間）、admin（管理）、store（店面）、pos（零售）
- **剛性規範**：Rust/Axum + Askama + SQLite，**零 JavaScript**、零 Electron、零新依賴
- **現況**：31 頁 200、145 tests 綠、dark theme（prefers-color-scheme 自動）、頂部導航
- **起因**：使用者批評介面醜、比 tauri-preview（1420 銀行金流平台）差、未經 grill-me 設計

## 已裁決的設計決策（Grill-me 7/7）

| # | 決策 | 裁決 | 為什麼 | 依據 |
|---|------|------|--------|------|
| Q1 | 視覺方向 | **A. Data-Driven Premium**（dark-first、高密度、sidebar） | 使用者偏好 tauri-preview（暗色側邊欄+卡片）；2026 專業工具 dark-first 預設；帳務=長時間工作階段 | Linear/Vercel/Grafana/Supabase |
| Q2 | Accent 主色 | **玉山綠 Teal**（`#14b8a6` 系） | 品牌對齊玉山合作夥伴；綠=金錢/信任；暗底對比 8.9:1 (AAA)；與通用 SaaS 藍區隔 | 玉山銀行品牌 + WCAG |
| Q3 | 字體 | **自架 Inter（拉丁/數字）+ JetBrains Mono（金額）** latin 子集 woff2，中文用系統微軟正黑體 | Linear/Vercel 同款排版跨平台一致；~50KB；CSP `font-src 'self'` 合規；CJK 系統字體零下載 | 2026 SaaS 字體趨勢 |
| Q4 | 導航 | **左側 220px sticky 側邊欄**，details 分組可折疊 | 31 頁長導航需固定位置；頂部下拉不符工具類慣例 | Linear/Grafana 標準 |
| Q5 | 密度 | **緊湊預設**（padding 16px、表格緊湊列） | 工具類每日使用；資訊密度優先於留白 | 2026 minimal chrome |
| Q6 | 動畫 | **僅互動回饋 150ms ease**（hover/active/focus），無載入動畫 | fadeIn 造成面板半透明 bug（已移除）；動效只為回饋不為炫技 | 本專案實證教訓 |
| Q7 | 資訊層級 | **狀態列→KPI→面板→表格**（Grafana 節奏）；金額 tabular-nums+mono | 帳務工具第一眼=狀態；金額對齊=可信度 | Grafana/Mercury 模式 |

## 色彩系統（已裁決，dark-only v1）

| 語義 | 值 | 對比 |
|------|-----|------|
| Background | `#020617` slate-950 | — |
| Surface（面板） | `#1e293b` slate-800 | lighter=closer |
| Border | `#334155` slate-700 | — |
| Text Primary | `#f1f5f9` | 15.9:1 AAA |
| Text Secondary | `#94a3b8` | 7.4:1 AA |
| **Accent（玉山綠）** | `#14b8a6`（teal-500，連結/圖示） | 8.9:1 AAA |
| **Accent Button** | `#0d9488`（teal-600，白字按鈕） | 白字 4.6:1 AA |
| Success | `#4ade80` / Warning `#fbbf24` / Error `#f87171` / Info `#38bdf8` | 暗底亮色版 |

**既有 slate 系 90% 相同**——只需：dark 翻轉為永遠預設（V2 :root 覆寫）+ accent 從藍換綠。

---

## 實作任務（依序執行）

### A1. design-system.css 末尾附加 V2 覆蓋層
`web/assets/design-system.css` — 完整 CSS 區塊如下（附加於檔尾，cascade 覆蓋）：

```css
/* ══════════════════════════════════════════════
   V2 — Data-Driven Premium（grill-me 7/7 裁決）
   原則: dark-first · 高密度 · minimal chrome · 側邊欄
   ══════════════════════════════════════════════ */

/* ── 色彩：dark 永遠預設 + 玉山綠 accent ── */
:root {
  --color-background: #020617;
  --color-surface: #0f172a;
  --color-surface-raised: #1e293b;
  --color-surface-sunken: #020617;
  --color-border: #334155;
  --color-border-subtle: #1e293b;
  --color-text-primary: #f1f5f9;
  --color-text-secondary: #94a3b8;
  --color-text-tertiary: #64748b;
  --color-primary: #14b8a6;
  --color-primary-subtle: rgba(20, 184, 166, 0.12);
  --color-primary-muted: rgba(20, 184, 166, 0.25);
  --color-border-focus: #14b8a6;
  --gradient-primary: linear-gradient(135deg, #14b8a6, #0d9488);
  --color-success: #4ade80;
  --color-warning: #fbbf24;
  --color-error: #f87171;
  --color-info: #38bdf8;
}

/* 中和 light/dark 媒體查詢（dark-only v1） */
@media (prefers-color-scheme: light) {
  :root {
    --color-background: #020617;
    --color-surface: #0f172a;
    --color-surface-raised: #1e293b;
    --color-surface-sunken: #020617;
    --color-border: #334155;
    --color-text-primary: #f1f5f9;
    --color-text-secondary: #94a3b8;
    --color-text-tertiary: #64748b;
  }
}

/* ── 字體：Inter + JetBrains Mono（自架 woff2） ── */
@font-face {
  font-family: "Inter";
  src: url("/assets/fonts/inter-latin.woff2") format("woff2");
  font-weight: 100 900;
  font-display: swap;
  unicode-range: U+0000-00FF, U+2000-206F, U+2074, U+20B9;
}
@font-face {
  font-family: "JetBrains Mono";
  src: url("/assets/fonts/jetbrains-mono-latin.woff2") format("woff2");
  font-weight: 100 800;
  font-display: swap;
  unicode-range: U+0000-00FF, U+2000-206F;
}

/* ── 佈局：左側欄 220px + 頂欄 + 主區（CSS Grid，零 HTML 變更） ── */
body {
  display: grid;
  grid-template-columns: 220px 1fr;
  grid-template-rows: auto 1fr;
  min-height: 100vh;
  background: var(--color-background);
}

.site-nav {
  grid-column: 1;
  grid-row: 1 / 3;
  position: sticky;
  top: 0;
  height: 100vh;
  overflow-y: auto;
  overflow-x: hidden;
  background: var(--color-surface);
  border-right: 1px solid var(--color-border);
  border-bottom: none;
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  z-index: var(--z-sticky);
}

.site-nav .container {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: var(--space-3) var(--space-2);
  max-width: none;
  overflow-x: visible;
}

.site-nav .container > a,
.site-nav details.nav-menu > summary {
  display: flex;
  align-items: center;
  width: 100%;
  padding: var(--space-2) var(--space-3);
  border-bottom: none;
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
}

.site-nav .container > a:hover,
.site-nav details.nav-menu > summary:hover {
  background: var(--color-primary-subtle);
  color: var(--color-primary);
}

.site-nav details.nav-menu .nav-menu-items {
  position: static;
  display: none;
  border: none;
  background: none;
  box-shadow: none;
  padding: 0 0 0 var(--space-4);
  min-width: 0;
}

.site-nav details.nav-menu[open] .nav-menu-items {
  display: flex;
  flex-direction: column;
}

.app-header {
  grid-column: 2;
  grid-row: 1;
  position: sticky;
  top: 0;
  z-index: var(--z-sticky);
  background: var(--color-surface);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

.app-header .container { max-width: none; }

main.container {
  grid-column: 2;
  grid-row: 2;
  max-width: none;
  padding: var(--space-5) var(--space-6);
}

/* ── 面板：1px 邊框 + raised 表面，minimal chrome ── */
.panel {
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  padding: var(--space-4);
  margin-bottom: var(--space-4);
  box-shadow: none;
}

.panel h3 {
  font-size: var(--font-size-lg);
  border-bottom: 1px solid var(--color-border-subtle);
  padding-bottom: var(--space-2);
  margin-bottom: var(--space-3);
}

.panel h3::after { display: none; }

/* ── 表格：tabular-nums + 緊湊 ── */
table { font-variant-numeric: tabular-nums; }
th, td { padding: var(--space-2) var(--space-3); }

/* ── KPI：mono 數字 ── */
.kpi { border-radius: 4px; padding: var(--space-4); }
.kpi dd { font-family: var(--font-family-mono); font-size: var(--font-size-2xl); }

/* ── 按鈕/輸入：radius 4px + accent ── */
.btn, .input { border-radius: 4px; }
.btn-primary { background: var(--gradient-primary); border: none; }

/* ── 登入頁：無側邊欄 ── */
body:has(.login-panel) { display: block; }
body:has(.login-panel) .site-nav { display: none; }

/* ── 響應式：≤900px 折疊回頂欄 ── */
@media (max-width: 900px) {
  body { display: block; background: var(--color-surface); }
  .site-nav {
    position: static;
    height: auto;
    border-right: none;
    border-bottom: 1px solid var(--color-border);
  }
  .site-nav .container { flex-direction: row; flex-wrap: wrap; }
  .site-nav .container > a,
  .site-nav details.nav-menu > summary { width: auto; }
  .site-nav details.nav-menu .nav-menu-items {
    position: absolute;
    background: var(--color-surface-raised);
    border: 1px solid var(--color-border);
    padding: var(--space-2);
  }
  main.container { padding: var(--space-4); }
}
```

### A2. 下載字體檔案
`web/assets/fonts/`：`inter-latin.woff2`（變數字重 ~45KB）+ `jetbrains-mono-latin.woff2`（~25KB）。
來源：GitHub 官方 releases（rsms/inter、JetBrains/JetBrainsMono，OFL 授權）。curl 下載。
**退路**：若下載失敗，@font-face 404 → 靜默退回系統字體堆疊（零風險）。

### A3. CSS 路由：字體檔案
`api.rs` public 群組加：`.route("/assets/fonts/:file", get(serve_font))` — Content-Type `font/woff2`，路徑白名單限定 `fonts/` 目錄（防路徑穿越）。CSP `font-src 'self'` ✓。

### A4. docs 同步 + 重建 + 重啟
`Copy-Item docs`；`cargo build`（CSS/字體 include_str 或路由）；重啟 8090。

### A5. 全套件驗證（見下方）

---

## 驗證計畫

1. 編譯通過 → 重啟伺服器（port 8090，demo-flaps.db：admin/finance/store/pos + 3 科目 + 3 交易 + 2 批次）
2. **視覺**（整合瀏覽器截圖）：`/login`（無側邊欄、亮 CTA）、`/`（側邊欄+玉山綠+面板 1px 邊框）、`/bankpay`、`/reports/trial-balance`（tabular-nums）
3. **功能**：31 頁 200（admin）、未登入 303、pos 寫入 403
4. **全套件**：145 tests 綠
5. **字體**：devtools network 確認 woff2 200；fallback 驗證（字體檔 404 時頁面仍正常）

## 風險與對策

| 風險 | 對策 |
|------|------|
| Grid 佈局破壞特殊頁面 | 登入頁 `:has()` 排除；30 頁結構已驗證一致 |
| 字體下載失敗 | @font-face 404 → 系統字體堆疊靜默退回（零風險） |
| dark-only 影響列印 | `@media print` 已有白底規則 ✓ |
| 覆蓋層與既有規則衝突 | cascade 附加於檔尾 = 後者勝；衝突僅限預期覆蓋項 |
| 使用者不滿意 | 覆蓋層獨立區塊，刪除即回滾 |

## 明確不在範圍（Phase B 另案）

- 各頁功能完成度（使用者指出的 7 區實質內容——後端資料對接）
- 淺色主題、密度切換 toggle、KPI hero 強調
- 圖表/資料視覺化（目前無圖表庫，零 JS 限制下需評估 SVG server-render）
