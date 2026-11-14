# A11Y 檢查表（WCAG 2.2 AA，零 JS）

> 版本：v1.0（2026-11-13）｜適用：所有 server-rendered 模板（`platform/templates/*.html`）與 `web/assets/design-system.css`。
> 目的：把「無障礙」從文件承諾變成可檢查的項目。機械可檢項由 `platform/tests/a11y_static.rs` 守門；其餘為人工/輔助工具檢查。

## 十項檢查

| # | 項目 | 機械檢查 | 人工/工具 |
|---|---|---|---|
| 1 | 語義 landmark：`header/nav/main/aside/footer` 齊備 | ✅ `a11y_static::landmarks_present` | 螢幕閱讀器走查 |
| 2 | 標題層級不重跳（h1→h2→h3） | ✅ 每頁有 `<h1>`（`h1_present`） | 逐頁 heading 順序 |
| 3 | 每個表單控制有對應 `<label>` | ✅ `<label for>` 均有匹配 `id`（`labels_bind_controls`） | 包裹式 label 人工確認 |
| 4 | 焦點順序合理、`:focus-visible` 可見 | CSS 檢查（`:focus-visible` 存在） | 鍵盤 Tab 走查 |
| 5 | 全部互動可鍵盤完成（無 mouse-only） | 零 JS＋CSS 互動（`:hover/:focus-visible/:target/:has()`） | 鍵盤走查 |
| 6 | 非文字內容有替代文字或 `aria-hidden` | ✅ `<img>` 必有 `alt`（`images_have_alt`） | 裝飾性 SVG 人工確認 |
| 7 | 顏色對比 ≥4.5:1（本文/背景） | ✅ `platform/tests/design_a11y.rs`（相對亮度公式） | 深色/淺色實測 |
| 8 | 不以顏色為唯一傳達（錯誤/狀態另有文字/圖示） | — | 狀態徽章人工確認 |
| 9 | `prefers-reduced-motion` 停用非必要動畫 | CSS 檢查（`@media (prefers-reduced-motion: reduce)` 存在） | 系統開啟「減少動態」實測 |
| 10 | 錯誤訊息含「問題＋可能原因＋處理方式」且與欄位關聯 | — | 表單錯誤人工走查（`aria-describedby`） |

## 機械檢查涵蓋

`platform/tests/a11y_static.rs` 對每個 `platform/templates/*.html` 驗證：

- `<!DOCTYPE html>` 與 `<html lang="zh-Hant…">`（語言宣告）。
- `<meta name="viewport" content="width=device-width, initial-scale=1.0">`（響應式；對齊 UX-XDEVICE）。
- `<main id="main"`（landmark；對應 item 1）。
- 至少一個 `<h1`（item 2）。
- 每個 `<label for="X">` 都有同檔 `id="X"`（item 3）。
- 每個 `<img` 都有 `alt=`（item 6）。

## 未涵蓋（人工/工具）

- item 4/5/8/10：需鍵盤走查、螢幕閱讀器（NVDA/VoiceOver）或輔助工具（axe DevTools）。
- item 7：由 `design_a11y.rs` 覆蓋 Token 對比；實際元件狀態（hover/disabled）需人工抽查。

## 變更規則

- 新增模板：必須通過 `a11y_static` 全部檢查；新增互動元件需補人工檢查紀錄。
- 新增顏色 Token：必須通過 `design_a11y` 對比檢查（≥4.5:1）。
- 任何 a11y 例外：於本表加註理由、範圍與追蹤 ID（不得無記錄放行）。