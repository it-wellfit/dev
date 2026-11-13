# EXPERIENCE_SPEC — 全端體驗規格（SSOT）v1.0

> 版本 v1.0｜生效日期 2026-11-13｜狀態 Active｜適用範圍：S1–S9 全表面、EC-1..EC-10 契約、TC-1..TC-5 工具組契約
> 上位：使用者指令（零 JS／效能／安全／穩定／易維護；全端 UI/UX；可追溯可驗證）→ 本規格 → 體驗室（`/design`，產品內標本館）→ 逐頁驗收（R0–R9）。
> 文件關係：互動層 TT-1..TT-8 見 `docs/engineering/UX_CONTRACTS.md`；需求追溯見 `docs/design/EXPERIENCE_TRACEABILITY.md`；稽核見 `docs/design/EXPERIENCE_AUDIT_v1.md`；矩陣見 `docs/design/EXPERIENCE_SURFACE_MATRIX.md` 與 `docs/design/EXPERIENCE_SCENARIO_MATRIX.md`。

## §1 目的與範圍

把散落的體驗知識（ui-v2 視覺、D-3 互動、token、八態、受眾分層）收斂為單一規格；任何新頁面/新表面**先補本規格**再實作。不適用（N/A）必須附理由；UNKNOWN 必須附解鎖條件（C-10）。

## §2 體驗契約（EC-1..EC-10；詳細規則與驗證）

| ID | 契約 | 規則（可驗收） | 驗證方法 | 現況（2026-11-13） |
|---|---|---|---|---|
| EC-1 | 識別與導覽 | 每頁 title＋單一 h1；manifest 驅動導覽、角色過濾、aria-current；skip-link（有重複區塊者）；無死連結 | page_matrix／web_crawl／設計 lab 內容標記 | PASS（pre-auth N/A 附理由＝XP-004） |
| EC-2 | 資訊架構 | studio＋section 兩層；Operator／Admin／Technical 分層；操作者不得見開發文件 | manifest 對帳＋導覽矩陣 | PASS |
| EC-3 | 狀態語彙 | 資料八態文案互異、各含下一步；元件六態齊備 | states 頁 SSOT＋每頁 data-state 錨點＋page_matrix | PASS |
| EC-4 | 動作語義 | GET 讀／POST 寫＋CSRF＋PRG；危險動作二段確認；撤銷＝反向事件 | CSRF 矩陣（27 路由）＋characterization | PASS |
| EC-5 | 內容與語氣 | zh-Hant-TW；錯誤＝問題＋原因＋處理；數字千分位／minor 兩位／+08:00；術語一致 | audit_states／文案審查／本規格 §7 | PASS |
| EC-6 | 視覺語言 | 三層 token（原始/語義/元件）；無硬編碼色；dark-first；對比 AA | close_out token 同步＋guards canonical＋design_a11y | PASS |
| EC-7 | 回饋與動效 | CSS-only；約 150ms；reduced-motion；focus-visible | guards＋互動稽核＋人工 | PASS（部分＝focus 全站走查列 R5 續輪） |
| EC-8 | 無障礙 | WCAG 2.2 AA：語義、標籤、替代文字、鍵盤、不以顏色唯一、200% 縮放 | design_a11y（對比）＋檢查表＋走查 | PARTIAL（鍵盤全站走查未執行；解鎖：R5 續輪） |
| EC-9 | 效能與感知 | 清單 <300ms、詳情 <150ms、報表 <500ms（2k）；分頁；無明顯位移 | close_out（月報 500ms）＋REQ 矩陣 P-03 | PARTIAL（頁面級未全量；解鎖：R8） |
| EC-10 | 韌性與恢復 | error／degraded／timeout／conflict／retry／empty；金額與權限 fail-closed、量測 fail-open | audit_states（錯誤態/衝突/時區）＋故障測試 | PASS |

## §3 表面矩陣（S1–S9）

見 `docs/design/EXPERIENCE_SURFACE_MATRIX.md`（零空格；逐列證據）。摘要：S1 全 ●；S2/S4 有 P1/P2 未驗（XP-001/002）；S3/S6/S7/S8/S9 以 ◐ 標示部分並附理由。

## §4 資訊架構與導覽

- 單一 manifest（`crates/console_manifest`）驅動：studio（導覽群組）＋section（頁面）兩層；tier＝Operator／Admin／Technical；gate＝Public／All／Read／Write／Manage。
- 受眾分層：操作者層不得渲染開發/治理入口（無權限入口不渲染；handler 端再驗＝縱深防禦）。
- 體驗室（/design 十頁）＝Technical×Manage：展示＋回歸基準＋入門；僅管理者可見。

## §5 視覺語言

- 三層 token：原始（色階/間距/圓角/陰影，`web/assets/design-system.css`）→ 語義（surface/text/border/semantic）→ 元件（按鈕/卡片/表格等以語義組裝）。
- 主題層：`web/assets/ui-v2.css`（dark-first；玉山綠 accent；完整語義覆蓋）——由伺服器串接於設計系統之後。
- 硬規則：模板不出現硬編碼色；雙副本（`web/assets/design-system.css` 與 `docs/design-system.css`）token 區逐字同步（測試把關）；間距 4px 網格；密度優先（每日高頻工具）。
- 數值呈現：金額 tabular-nums、右對齊；未知「—」；時間 +08:00 伺服器端格式化。

## §6 元件庫（≥24；逐一示範於 `/design/components`）

| # | 元件 | 用途 | 實作類別 | 狀態覆蓋 |
|---|---|---|---|---|
| 1 | 主要按鈕 | 主動作 | btn btn-primary | 六態 |
| 2 | 次要按鈕 | 輔助動作 | btn btn-secondary | 六態 |
| 3 | 幽靈按鈕 | 低干擾 | btn btn-ghost | 六態 |
| 4 | 危險按鈕 | 破壞性（二段確認） | btn btn-danger | 六態 |
| 5 | 成功按鈕 | 正向路徑 | btn btn-success | 六態 |
| 6 | 尺寸變體 | 密度調整 | btn-sm／btn-lg／btn-xl | — |
| 7 | 文字輸入 | 表單 | input（＋input-sm/lg） | 六態（含 user-invalid） |
| 8 | 表單列 | 標籤＋欄位＋提示 | form-row | — |
| 9 | 欄位錯誤 | 問題＋修復指引 | field-error＋aria-invalid | — |
| 10 | 提示文字 | 說明 | hint／form-hint | — |
| 11 | 動作列 | 送出/取消 | form-actions | — |
| 12 | 徽章 | 狀態（必附文字） | badge＋六色＋badge-mock | — |
| 13 | 狀態點 | 輕量標示 | badge-dot | — |
| 14 | 卡片 | 入口/摘要/清單 | card＋card-header/body/footer | hover |
| 15 | 面板 | 頁內分區 | panel | — |
| 16 | KPI 卡 | 指標 | kpi-grid／kpi | — |
| 17 | 表格 | 清單 | table-wrap＋caption/scope | 列 hover |
| 18 | 分頁 | 清單導覽 | pagination（is-disabled＋aria-disabled） | 首末態 |
| 19 | 披露 | 零 JS 展開 | details／summary | — |
| 20 | 空狀態 | 空 | empty-state | — |
| 21 | 零結果 | 篩選無相符 | zero-state | — |
| 22 | 載入 | 進行中 | loading-state＋spinner | reduced-motion |
| 23 | 錯誤 | 失敗＋重試 | error-state（role=alert） | — |
| 24 | 降級 | 部分不可用 | degraded-state（role=status） | — |
| 25 | 過期 | 最後更新時間 | stale-state＋time | — |
| 26 | 無權限 | 指名權限碼 | denied-state | — |
| 27 | 衝突 | 併發處理 | conflict-state（409 語義） | — |
| 28 | 跳至內容 | 無障礙 | skip-link | focus |

## §7 內容與語氣

- 錯誤三要素（問題＋可能原因＋處理方式）＋下一步連結；成功必含結果與去向。
- microcopy 目錄與術語表（ubiquitous language）示範於 `/design/content`；數字/日期/時區格式見該頁。
- 誠實原則：示意資料必標示；模擬模式掛 badge-mock；未知以「—」。

## §8 狀態語彙（八態 SSOT）

全頁樣板＝`platform/templates/states.html`（八態）；每頁經 `platform/templates/partials/state_anchors.html` 掛 empty/denied/error 錨點；元件級矩陣於 `/design/states`。

## §9 互動與動效（零 JS 原語）

| 原語 | 用途 | 出處 |
|---|---|---|
| 表單 POST＋PRG | 所有寫入 | EC-4 |
| `:target` 面板 | 詳情開關 | TT-3 |
| `details/summary` | 選單與披露 | TT-2 |
| `:hover/:focus-visible/:user-invalid` | 回饋 | EC-7 |
| prefers-reduced-motion | 動效停用 | EC-7 |

（互動層完整契約 TT-1..TT-8 見 `docs/engineering/UX_CONTRACTS.md`。）

## §10 無障礙（WCAG 2.2 AA 檢查表）

- 對比：內文 ≥4.5:1、大字 ≥3:1、焦點環 ≥3:1（自動：`platform/tests/design_a11y.rs`）。
- 鍵盤：全互動可達；focus 可見；skip-link（有重複區塊處）。
- 語義：landmark、單一 h1、表單 label、表格 caption/scope、按鈕語義。
- 不以顏色為唯一傳達；200% 縮放不破版（R7 排程）。

## §11 效能預算

| 頁面類別 | 預算（p95） | 機制 | 驗證 |
|---|---|---|---|
| 清單（/transactions 等） | <300ms | 分頁 20/頁、索引 | 預熱＋p95（R8 全量） |
| 詳情（/transactions/:id 等） | <150ms | 單列查詢 | 同上 |
| 報表（月報等，2k 交易） | <500ms | 聚合快取/索引 | close_out 月報測試 |
| 頁面重量 | CSS 單檔＋字型自託管；ETag 304 | 快取重驗 | REQ 矩陣 P-02 |

## §12 跨表面規則（S1–S9）

- S1：響應式 4 尺寸（1280/1024/768/390）；900px 折疊；觸控目標 ≥44px。
- S2：首啟/更新/錯誤態規格化（XP-001；解鎖：A-2 演練）。
- S3：靜態頁與平台同 token/密度；唯讀。
- S4：列印＝頁首（公司/報表名/期間/產出時間）＋頁尾（頁碼/版本/內部使用）；分頁不截斷；黑白可讀（XP-002）。
- S5：匯出＝檔名含報表名＋日期；欄序穩定；UTF-8；金額原樣（未知「—」）。
- S6：文件受眾四層（操作者/管理者/技術者/稽核）＋metadata（XP-005）。
- S7：錯誤碼字典（解鎖：A-3）；錯誤封套一致。
- S8：事件語義、嚴重度、可行動；範本走查（未執行，解鎖：R7）。
- S9：TC-1..TC-5（見 §13）。

## §13 工具組契約（TC-1..TC-5；與互動 TT 系列分名）

| ID | 契約 | 規則 |
|---|---|---|
| TC-1 | 一致 help | 子命令皆有 --help、用法、範例、退出碼 |
| TC-2 | 退出碼語義 | 0 成功；非 0 失敗；UNKNOWN 明確（不得 0 冒充） |
| TC-3 | 雙模式輸出 | 人類可讀＋--json；欄位穩定 |
| TC-4 | 無破壞預設 | 破壞性需 --yes 或 dry-run；預設不覆蓋 |
| TC-5 | 每台機器通用 | 偵測環境；禁寫死本機規格 |

## §14 驗收與追溯

- 追溯矩陣：`docs/design/EXPERIENCE_TRACEABILITY.md`（REQ-UX-01..20）。
- 自動校驗：`platform/tests/experience_traceability.rs`（零空格/五級判定/解鎖條件/路徑解析）。
- 驗收輪次：R0–R9（本輪對照見 `docs/design/EXPERIENCE_AUDIT_v1.md` §6）。

## §15 決策記錄（8 欄格式範例；完整三則於 `/design/decisions`）

| 決策 | 選擇 | 原因（摘） | 證據 |
|---|---|---|---|
| DEC-DL-01 暗色優先分層 | dark-first 主題層覆蓋全語義 token | 專業工具主流；集中修正殘留淺色 | `web/assets/ui-v2.css` |
| DEC-DL-02 零 JS 互動 | CSS 原語＋POST/PRG | 無啟動成本、無 XSS 面、易測 | guards／web_crawl |
| DEC-DL-03 體驗室 Gate=Manage | 十頁 ROLE_MANAGE | 規格維護者為管理者；操作者不見開發文件 | design_lab 閘矩陣 |
