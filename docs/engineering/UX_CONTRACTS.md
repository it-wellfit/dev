# UX 體驗契約與驗證矩陣（TT-1..TT-8）

> 版本：v1.0｜日期：2026-11-13｜狀態：Active
> 定義「全端 UI/UX 觀念」為**可稽核的體驗契約**，不是「多做幾頁應付了事」。
> 每條契約：REQ（需求）→ DEC（決策）→ IMPL（實作位置）→ TEST（測試）→ EVID（證據）。
> 自動校驗：`platform/tests/interaction_audit.rs`（TT-1..TT-5）＋ `web_crawl.rs`（6 閘門）＋ `password_reset.rs`（重設全鏈）。

## 體驗契約（Experience Contracts）

| ID | 契約 | 交互原語 | REQ | DEC | IMPL | TEST | EVID（2026-11-13 實測） |
|----|------|----------|-----|-----|------|------|--------------------------|
| TT-1 | 頁內錨點：子導覽宣告的每個 `#id`，其目標元素必須存在同頁 | 錨點跳轉 | 子導覽不可死鏈 | 6 錨點對應 6 區塊 id | `bankpay.html`＋`interaction_audit::tt1` | tt1 | anchors 6/6 目標存在 |
| TT-2 | `<details class="nav-menu">` 下拉：summary（點擊目標）＋items（展開內容）齊備 | `<details>` 原生展開 | 每頁導覽可展開 | 原生元素、零 JS | `partials/studio_nav.html` | tt2 | ≥4 menus/summary/items 每頁 |
| TT-3 | `:target` 面板：狀態詳情以 hash 開關（CSS display 切換） | `:target` 偽類 | 詳情面板零 JS 開關 | `#status-panel:target` 規則 | `design-system.css` L1001+ | tt3 | panel id＋:target 規則＋toggle 連結齊 |
| TT-4 | 多步流程：精靈三步（模板→細節→確認），錯誤帶規則標籤回渲染 | Form POST 鏈 | 引導式登帳可用且錯誤可修 | stage=choose 分支＋R-BALANCE 回饋 | `api.rs::wizard_step2` | tt4 | 三步全通；借貸同科目→R-BALANCE 顯示 |
| TT-5 | 搜尋回填：GET 查詢後輸入值保留 | GET form | 送出後輸入不消失 | value 屬性回填 | `search.html` | tt5 | q=cash → value="cash" |
| TT-6 | 忘記密碼全鏈：請求（無帳號列舉）→管理員核發一次性 token（僅存 SHA-256、30 分鐘、用後即焚）→自設新密碼（撤銷全 session） | 多步公開流程 | 無自助重設＝登入死路 | 本地部署無郵件→token 交付制 | `0010_password_reset.sql`＋`api.rs` 3 路由＋2 模板 | password_reset 2 測試 | 全鏈 curl 實證：請求→核發（64 字元 token）→重設→新密碼登入 303 |
| TT-7 | 死連結免疫：全站內部連結逐一可達（nav/頁尾/麵包屑任一錯字即 FAIL） | 全站連結圖 | 任何連結點開必須有頁 | BFS 爬取≥30 連結 | `web_crawl::every_internal_link_resolves` | crawl | 0 死連結 |
| TT-8 | 404 契約：未知路徑＝品牌頁（識別＋問題/原因/處理方式＋返回連結＋零 JS） | fallback handler | 空白 404＝體驗不良 | 品牌 fallback 頁 | `api.rs::not_found_page` | web_crawl unknown_path | 404＋返回連結＋CSS＋scripts=0 |

## 問題來源清單 → 分級（2026-11-12..13 瀏覽器實證輪）

| 來源 | 問題 | 級別 | 狀態 |
|------|------|------|------|
| 使用者回報 | 登入缺忘記密碼/重設 | P1 | ✅ 修復（TT-6 全鏈） |
| 使用者回報 | 草稿→登帳資訊漏傳 | P0（財務數據） | ✅ 修復（amount plain 格式＋「元：」清理＋description 帶證據） |
| 使用者回報 | AI 建議畫面錯誤無法開啟 | P1 | ✅ 修復（旗標治理 UI＋llama3→qwen3.5:4b＋think:false＋混合解析金額 88,500.00 ✓＋幣別正規化） |
| 使用者回報 | 缺 AI 操作教學 | P2 | ⏳ 設計中（help 頁已有 AI 段落；互動教學列 P2 路線） |
| 使用者回報 | 自訂工作流程治理介面 | P2 | ⏳ 第一片落地（AI 旗標治理 UI）；完整工作流設計器列 P2 路線 |
| 瀏覽器實證 | 子導覽被側欄蓋住不可點擊 | P0 | ✅ 修復（ui-v2 grid col 2） |
| 瀏覽器實證 | 精靈第一步 422 | P0 | ✅ 修復（stage 分支） |
| 瀏覽器實證 | ConnectInfo 登入回歸 | P0 | ✅ 修復（Option<ConnectInfo>） |
| 爬取實證 | 404 空白頁 | P2 | ✅ 修復（品牌 fallback，TT-8） |
| curl 實證 | AI 模型 llama3 未安裝 | P1 | ✅ 修復（qwen3.5:4b） |
| curl 實證 | thinking model 回應空 | P1 | ✅ 修復（think:false） |
| curl 實證 | 模型金額截斷（88,500→885.00） | P0（財務紅線） | ✅ 修復（混合解析：規則提示權威值） |
| curl 實證 | 幣別「元」未過 schema | P1 | ✅ 修復（正規化＋prompt 強化） |
| 爬取實證 | ai/suggest_enabled 無 UI 可開 | P1 | ✅ 修復（治理 UI toggle＋稽核） |

## 已知限制（誠實揭露）

- OQ-E3：玉山 Sandbox 憑證未得——BankPay 正式模式阻塞（外部）。
- perf.rs 延遲為機器相依（非閘門，僅趨勢證據）。
- rerank/HyDE 需 Ollama 運行（不可用→誠實降級，已測）。
- 互動教學（TT-9 候選）與工作流設計器（TT-10 候選）列 P2 演進路線，尚未實作。
- 瀏覽器實證（Playwright）為人工/半自動；自動化瀏覽器閘門列 P2。

## 驗收判定

**PASS（條件式）**：TT-1..TT-8 全部有 TEST＋EVID；已知限制如上揭露；P0/P1 全數關閉；P2 兩項（教學、工作流設計器）列路線未實作——不影響現有功能正確性，但屬使用者明示需求，列為下一切片。
