# ADR-0002: 零 JS API 層（Axum 0.7 伺服器渲染）

| 欄位 | 內容 |
|------|------|
| ADR 編號 | ADR-0002 |
| 標題 | 零 JS API 層：Axum 0.7 伺服器渲染＋PRG＋伺服器端 RBAC |
| 狀態 | Accepted |
| 決策者 | 工作區擁有者（依《ELITE PRODUCT ENGINEERING MASTER DIRECTIVE》執行計畫 Slice 5／P1-1） |
| 日期 | 2026-11-04 |
| 關聯 | ADR-0001（技術棧基線）、THREAT_MODEL T-01/G-1、PRD FR-003~007 |

## 1. 背景與問題

根 workspace 的零 JS 介面（ADR-0001 D-03/D-04）當時為靜態基線：表單 POST `/transactions` 無處理者、清單為空狀態佔位——核心登帳流程（FR-003）無法端到端。需要最小 API 層讓產品可用，同時不得違反：零 JavaScript（憲章 §2.3）、GET 不修改狀態、金額整數化（INV-01，P0-1）、伺服器端授權（FR-006）。

## 2. 考慮的選項

| 決策點 | 選項 A（採用） | 選項 B（不採用） | 理由 |
|--------|---------------|------------------|------|
| 伺服器落點 | `platform/src/bin/flaps-server.rs`（bin 掛在既有 crate） | 新建獨立 `server/` crate | 最小複雜度；platform 已依賴 core＋sqlx，避免 crate 膨脹；未來可再抽離 |
| 模板引擎 | Askama 0.16（編譯期型別檢查＋自動 HTML escaping） | `format!` 手工拼接（需自寫 escaping，XSS 風險）／引入前端框架（違反零 JS） | 型別安全＋OWASP escaping 開箱即用（T-07）；ekpi-platform 同版本已驗證可行 |
| 靜態資產 | `include_str!` 編譯期嵌入 `web/assets/design-system.css` | `tower-http` ServeDir 讀檔 | 零路徑遍歷面、零執行期檔案 I/O、部署確定性 |
| PRG 動詞 | `Redirect::to`（axum 303 See Other） | 302 Found | 303 是 POST→GET 語意正確動詞；axum 預設即 303 |
| 表單值保留 | 失敗時伺服器重渲染並回填欄位 | 導回空白表單 | UX（FR-004）：使用者不重打已輸入內容；Askama 跳脫保證安全 |
| 身分來源 | `X-Acting-User` header（預設 user-admin）→ RBAC 檢查 | 完整認證會話 | 認證屬 G-1 缺口（THREAT_MODEL T-01）；header 值仍過 RBAC——偽造不會取得角色表以外權限；限制誠實記錄於 §5 |

## 3. 決策

1. **路由面**：`GET /`（伺服器渲染全頁）、`GET /assets/design-system.css`（嵌入）、`GET /api/health`（JSON 探針）、`POST /transactions`（表單→驗證→RBAC→登帳→303 PRG）。
2. **驗證鏈（POST）**：RBAC（`LEDGER_WRITE`，依 permission_code）→ 科目代碼（trim、≤64）→ 金額（`flaps_core::money::parse_decimal_to_minor`，整數分 i64，最多 2 位小數，>0，≤DECIMAL(15,2) 上限）→ 借貸別白名單 → 幣別白名單 → 摘要（≤200）→ 日期（YYYY-MM-DD、不得未來）→ `post_journal_entry`（冪等 UUID＋audit_log＋原子事務）。
3. **模板 SSOT**：`platform/templates/index.html` 為伺服器渲染 SSOT；`web/index.html` 標頭註記 Superseded（保留為離線設計基線，不新增功能）；Design Token SSOT 維持 `web/assets/design-system.css`（嵌入提供）。
4. **金額整數化（P0-1 一併落地）**：`TransactionItem.amount_minor: i64`（整數分）＋ `exchange_rate_scaled: i64`（10^4 縮放）；schema 為 `amount_minor INTEGER NOT NULL`、`exchange_rate INTEGER DEFAULT 10000`（migrations 0001 修正，見 §6）；顯示用 `format_minor` 純整數還原（千分位＋兩位小數）。
5. **失败路徑**：400（欄位驗證，訊息含問題＋原因＋處理方式）、403（RBAC，指名缺少權限碼）、500（資料庫層，含脫敏細節）；一律回完整頁面＋保留表單值。

## 4. 實施與正面結果（2026-11-04 實測）

| 驗證 | 結果 |
|------|------|
| `cargo test --locked`（全 workspace） | EXIT=0；21 passed / 0 failed（platform 12：API 端到端 8＋遷移 4；flaps_bridge 7；flaps_core 6，含 money 邊界與 roundtrip） |
| `cargo clippy --all-targets -- -D warnings` / `cargo fmt -- --check` / `cargo audit` | 全部 EXIT=0 |
| 煙霧測試（curl.exe，無 Python 依賴） | health 200/db:true；POST → **303 + Location /#transactions**；GET / 渲染真實資料（`TX-…`、`測試交易`、**`1,234.50`**）；`X-Acting-User: user-store1` → **403**；`0.005` → **400**；頁面 `<script>` 計數 = **0** |
| 實機檢視（Integrated Browser） | 交易清單渲染 POSTED badge、導航高亮、深色主題正常 |

## 5. 負面結果與風險

1. **G-1 未完成**：`X-Acting-User` 未經認證（任何人可自稱 user-admin）——RBAC 仍強制，但**授權邊界暫由客戶端宣告**。上線前必須落地認證會話（SHARED_STANDARDS §7.2）；THREAT_MODEL T-01 維持 Open。
2. **Askama 0.16 parser 邊界**：開始標籤屬性區內的 `{% %}` block 不支援（編譯期報誤導性錯誤）；模板以「完整 option 重複」規避。已記入開發指南注意事項。
3. **PowerShell 5.1 編碼陷阱**：`Get-Content/Set-Content`（ANSI 預設）會污染含中文的模板；本輪以工具鏈寫盤繞過。模板維護一律使用 UTF-8 安全工具。
4. **單筆分錄**：表單目前建立單一 item（借或貸），未強制借貸平衡——帳務完整性缺口列 IMPROVEMENT_BACKLOG P1-5。

## 6. Follow-up Actions

1. P0-1 收尾：0001 遷移直接修正為整數 schema（無已遷移資料庫，未發布遷移允許原地修正——註解已記錄裁決）；`Contract` 階段不再需要。
2. P1-5：借貸平衡強制（multi-item 表單或伺服器組合分錄）。
3. G-1：認證會話＋CSRF token（T-01/T-08）。
4. 模板維護規則寫入 SHARED_STANDARDS 增量（屬性區禁 `{% %}`；模板只經 UTF-8 安全工具編輯）。
