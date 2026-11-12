# UI 設計解剖報告 v1.0 — legacy-1420 × 整合工作台 V2 × 一流產品

> 日期：2026-11-12｜狀態：Active｜用途：未來 UI 演進的重要參考依據（使用者指示持久化）
> 對照三方：`web/legacy-1420.html`（歷史快照）｜`platform/templates` + `web/assets/ui-v2.css`（現行）｜Linear / Vercel / Stripe / Notion（校準基準，記憶 product_calibration_flaps_erp）

## 0. 方法論：五層解剖模型

```
L5 微細節（像素級）── 複利效應所在
L4 互動與狀態 ─── 品質感受的來源
L3 結構工程 ──── 演化速度的決定者
L2 視覺語言 ──── 一致性的基礎設施
L1 資訊架構 ──── 產品世界觀（最高槓桿）
```

判斷原則：L1 錯了，L2–L5 全部白費；L3 弱了，每次迭代都在還債。

## 1. L1 資訊架構：組織邏輯 = 產品的世界觀

| | legacy-1420 | V2 整合工作台 | 一流產品 |
|---|---|---|---|
| 分類軸 | 銀行 API 領域（系統/台幣/外幣/帳戶/工具） | 角色工作室（biz/bank/gov/ops + home 置頂） | Stripe＝資金流；Linear＝狀態流 |
| 層數 | 2 層 | 3 層（工作室標頭為路標非節點） | 2–3 層 + 錨點 |
| 過濾 | 全顯示 | RBAC 資料層過濾（NavModel） | 同 V2 |

演化軌跡：legacy＝工程師視角的 API 清單 → V2＝企業角色分工 → 一流產品＝使用者的下一個動作（任務佇列）。

已兌現的巧思：home 獨立錨點（`nav.home`）、層次增加而認知負擔不增（工作室標頭不可點）、RBAC 不渲染即無權限、降維響應式（≤900px 工作室層折疊，ui-v2.css `.nav-studio-header{display:none}`）。

## 2. L2 視覺語言：Token 代差

- legacy：17+ 處硬編碼 hex、primary 為 Tailwind sky-500 預設值（未經品牌決策）。
- V2：三層語義 token（surface 4 階/text 4 階/border 3 階）＋玉山綠完整互動階梯（base→hover→active＋subtle 10%/muted 18% alpha）＋5 級 elevation＋glass 浮層。
- **暗色主題 elevation 語言學**：陰影在暗色失效 → 亮度階（#020617→#0f172a→#1e293b）承擔平面層次，shadow 專責浮層——GitHub Primer dark 手法，兩種訊號各司其職。
- **alpha 疊層規則**：狀態底色用色相降飽和（Linear badge 手法）——滿版實色徽章互相打架，alpha 底可並列掃描。

## 3. L3 結構工程：Design as Code

```
console_manifest（Rust 常數，型別化 SSOT）
  → NavModel（RBAC 過濾 + 語義測試）
  → studio_nav.html macro（40+ 模板單點生效）
  → CSS token 層（design-system.css）→ 佈局層（ui-v2.css）
```

比 design-tokens.json 更進一步：tokens 在 Rust 裡有型別、有編譯檢查、有測試。`:has()` 佈局重塑（login 頁零 HTML 改動去側欄，ui-v2.css:300-306）。

## 4. L4 互動與狀態

V2 已收割的零 JS 規格紅利：`<details>` disclosure、`:target` 路由、`:has()` 佈局判斷、`:user-invalid`、`aria-current`、`prefers-reduced-motion`、八態 DOM 掛點。legacy 全無。

## 5. L5 微細節現況（2026-11-12 實測）

| 項目 | 狀態 |
|---|---|
| `tabular-nums`（金融數字對齊） | ✅ 已到位（ui-v2.css:277 `table{...}`） |
| `:focus-visible` | ✅ design-system.css 五處（383/616/996/1751/2030） |
| `::selection` 品牌化 | ✅ 2026-11-12 補（ui-v2.css 尾段，primary-muted alpha） |
| emoji → SVG stroke icons | ⏳ 未做（跨平台渲染不一致、無 currentColor；經 SSOT 驅動改一處全站生效） |
| 骨架屏 / focus ring 雙環 | ⏳ backlog |

## 6. 一流產品隱形巧思（按零 JS 可學性）

1. Vercel 空狀態即引導（CTA 化）— 完全可學（八態掛點已就位）
2. Stripe 表格即產品 — `:target`+GET 表單可近似
3. Linear URL 即狀態 — 大部分可學（現行模式已相容）
4. Notion 麵包屑 — 伺服器端渲染完全可學
5. Linear ⌘K / 樂觀 UI — 零 JS 邊界，對應解法＝伺服器回應速度為信心來源

## 7. 演進參考點（裁定優先級）

- **P0**：emoji→Lucide inline SVG（經 console_manifest SSOT）｜focus ring 雙環｜空狀態 CTA 化
- **P1**：表格視圖化（欄位/日期範圍標準元件）｜服務端麵包屑（工作室→群組→頁）
- **P2**：首頁「任務佇列」化（待放行 N 筆直入工作流；KPI 退居次要）｜`:target` 命令面板｜亮色主題手動切換（`<details>`+cookie POST 零 JS）
- **根基勿動**：SSOT 鏈、三層 token、亮度階 elevation、alpha 疊層、八態掛點、降維響應式。

## 8. 一句話定性

legacy＝「工程師視角的 API 清單，披著暗色皮膚」；V2＝「有世界觀、有基礎設施的產品，已跨過能演化門檻」；一流產品＝同一套基礎設施上，把「使用者的下一個動作」做成組織原則，再用微細節複利堆出質感。差距已不在架構，在紀律與最後一哩——而 SSOT 鏈正是兌現這一哩的成本放大器。
