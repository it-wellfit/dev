# 多維度正向演進 — 專業品質提升計畫

## 目標
將 FlapsERp 平台從「功能可用」提升至「專業產品品質」，追求精緻、精美、精密、精良、精巧、精雕、精準、精湛、精品、精益、精研、精選的十二個工藝維度。

## 執行前提
- 工作目錄：`C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev`
- 建置產物必須導向 `%LOCALAPPDATA%\Temp\dev-target`（桌面路徑受應用程式控制原則阻擋）
- 所有 Schema 變更必須透過 Migration（Migration 規則）
- 零 JavaScript 政策（`<script>` 數 = 0）
- 整數金額禁用浮點（INV-01）
- clippy 零警告（`-D warnings`，不得加 allow）
- fmt 檢查必過（`cargo fmt -- --check`）
- 全部測試必須通過（`cargo test --locked`，0 failed / 0 ignored）

## 影響範圍
- `platform/src/api.rs`：移除除錯語句、修正 unwrap、新增查詢參數驗證
- `platform/src/bin/flaps-server.rs`：啟動時 Session 清理
- `platform/migrations/0004_query_indexes.sql`：新增查詢索引（已建立）
- `platform/templates/*.html`：修正模板損壞（已修正）、改善錯誤頁面
- `web/assets/design-system.css`：深色模式、微互動、焦點環（已改善）
- `scripts/seed-demo.ps1`、`scripts/reset-demo.ps1`：Demo 種子與重置（已存在）

## 五大演進維度與具體任務

### 1. 視覺精緻化（精緻 / 精美 / 精雕）
**目標**：提升介面質感與視覺一致性，呈現細膩、賞心悅目的設計。

| 任務 | 說明 | 優先級 |
|------|------|--------|
| V1 | Deep Dark Mode：補充所有暗色色票（陰影、邊框、按鈕、表單、徽章、表格） | 高 |
| V2 | 動畫過渡：淡入淡出、滑入、縮放，受 `prefers-reduced-motion` 控制 | 中 |
| V3 | 微互動：按鈕載入中、表單欄位聚焦、卡片懸浮、表格列選中 | 中 |
| V4 | 色票一致性審查：確保 Design Token 在明/暗模式對比度 ≥ 4.5:1（WCAG AA） | 高 |

### 2. 無障礙審查（WCAG 2.2 AA / 精良 / 精選）
**目標**：確保所有互動元件可鍵盤導航、焦點可見、不以色彩為唯一傳達。

| 任務 | 說明 | 優先級 |
|------|------|--------|
| A1 | WCAG 2.2 AA 對比度審查：所有文字與背景組合 ≥ 4.5:1，大文字 ≥ 3:1 | 高 |
| A2 | 鍵盤導航順序：Tab 鍵順序合理、焦點不被困住 | 高 |
| A3 | 焦點可見性：`:focus-visible` 環寬、顏色、偏移一致 | 高 |
| A4 | 語義 HTML：標題層級、地標區域、表單標籤關聯正確 | 中 |
| A5 | 非色彩傳達：錯誤/成功狀態同時使用圖示與文字 | 中 |

### 3. 效能優化（精密 / 精準）
**目標**：支援 10,000 筆交易壓力測試，查詢延遲 ≤ 500ms。

| 任務 | 說明 | 優先級 |
|------|------|--------|
| P1 | 查詢索引優化（Migration 0004 已建立，驗證執行計畫） | 高 |
| P2 | 10,000 筆交易壓力測試：建立 benchmark 測試，量測 P95 響應時間 | 高 |
| P3 | 分頁與排序：大資料集不重新載入整頁 | 中 |
| P4 | 資料庫連線池優化：根據並發需求調整 max_connections | 中 |

### 4. 功能擴展（精密 / 精巧 / 精良）
**目標**：擴充企業級功能，支援多幣別、批次處理、自訂報表。

| 任務 | 說明 | 優先級 |
|------|------|--------|
| F1 | 多幣別匯率管理：匯率表、換算、報表幣別切換 | 中 |
| F2 | 批次交易：CSV/Excel 批次匯入與驗證（已有 import，擴充為批次作業） | 中 |
| F3 | 自訂報表：使用者定義欄位、篩選、匯出 | 低 |
| F4 | 交易作廢與沖銷：保留稽核軌跡 | 中 |

### 5. 部署自動化（精良 / 精益 / 精研）
**目標**：支援 CI/CD、備份還原、監控觀測。

| 任務 | 說明 | 優先級 |
|------|------|--------|
| D1 | CI/CD pipeline：fmt、clippy、test、build、deploy | 高 |
| D2 | 備份還原演練：SQLite 資料庫備份、還原驗證 | 高 |
| D3 | OpenTelemetry 觀測：請求延遲、錯誤率、資源使用 | 中 |
| D4 | Windmill 原生二進位部署：Windows/Linux/macOS/NAS Task Scheduler | 中 |

## 品質門檻（執行前後必過）
| 檢查項目 | 命令 | 預期結果 |
|----------|------|----------|
| 格式檢查 | `cargo fmt -- --check` | 零差異 |
| Clippy | `cargo clippy --all-targets -- -D warnings` | 零警告 |
| 測試 | `cargo test --locked` | 0 failed / 0 ignored |
| 安全 | `cargo audit` | 無高危漏洞 |
| 建置 | `cargo build --locked` | 0 error |

## 驗收標準
1. 所有五大演進維度的「高」優先級任務完成
2. 品質門檻全部 EXIT=0
3. Demo 環境可重複種子與重置
4. UI/UX 符合 WCAG 2.2 AA 與零 JavaScript 政策
5. 沒有 `unsafe_code`、`f64` 金額計算、內聯樣式、Git 提交前 fmt/clippy 錯誤

## 開發約定（必遵守）
- 每個 `.rs` 第一行 `#![forbid(unsafe_code)]`
- 讀取 GET；建立／修改／刪除 POST；成功後 PRG
- PowerShell 串接用 `；`，禁止 `&&`
- 不將 Secrets 提交版控
- 文件與代碼同一提交更新
- 所有 Schema 變更必須透過 Migration，可在乾淨資料庫成功執行
