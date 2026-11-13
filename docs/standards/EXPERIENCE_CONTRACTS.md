# EXPERIENCE CONTRACTS — 全端 UI/UX 體驗契約與驗證矩陣 v1.0

> 日期：2026-11-13｜狀態：Active
> 定義：「全端 UI/UX」＝**完全的多層體驗契約＋驗證矩陣**，不是多做幾頁應付。
> 配套：`REQUIREMENTS_TRACEABILITY.md`（REQ→DEC→IMPL→TEST→EVID）＋ `trace_guard` 自動校驗測試。

## 0. 契約分層模型（每頁皆適用，缺一即違約）

| 層 | 契約 | 驗證方式 |
|---|---|---|
| L0 頁面殼 | 200、title、lang、viewport、品牌、無錯誤痕跡 | `all_manifest_pages_render_under_contract` |
| L1 導覽 | site-nav 存在、studio 標頭、aria-current、導覽隨權限收縮 | 同上＋RBAC 矩陣 |
| L2 內容 | 主要資料呈現、空態/載入/錯誤八態掛點 | E2E D5/各頁內容斷言 |
| L3 元件 | 表單有 label、欄位契約正確、表格 tabular-nums、按鈕 type | harness 元件檢查 |
| L4 互動 | 每個 POST 有 CSRF、成功 PRG、失敗帶原因、冪等 | E2E D 系列 |
| L5 邊界 | 空值/超長/特殊字元/越權/404/併發 | 二十五問掃描 |

## 1. 功能組體驗契約 TT-1..TT-5

每列 = 一條體驗契約：**觸發 → 正常結果 → 錯誤路徑 → 證據**。證據編號 E-xx 對應下表測試。

### TT-1 會話與身份
| 操作 | 正常結果 | 錯誤路徑 | 證據 |
|---|---|---|---|
| 登入成功 | 303→角色落地頁＋`auxbridge_session`（HttpOnly/SameSite=Lax/Path=/） | 錯誤→200＋通用訊息（不洩帳號存在性） | E-1 `login_sets_secure_cookie_when_configured`、`login_failure_shows_generic_error` |
| 連續失敗 | — | 5 次鎖定 900s；**鎖定訊息對正確密碼同樣拒絕** | E-2 `login_lockout_survives_correct_password` |
| 登出 | 303→/login＋cookie 清除＋稽核 LOGOUT（IP/UA） | CSRF 錯→session 不撤 | E-3 `close_out` 測試＋稽核實測 |
| 改密碼 | 303＋新密碼可登入 | <12 字/弱密碼/同帳號→400 帶原因 | E-4 `password_policy_*`＋E2E |
| 稽核溯源 | LOGIN/LOGOUT 帶 IP+UA | 無代理→ConnectInfo；皆無→NULL（不捏造） | E-5 快照欄位實證 |

### TT-2 帳務與交易
| 操作 | 正常結果 | 錯誤路徑 | 證據 |
|---|---|---|---|
| 建立交易 | 303→清單→詳情（金額/描述）→試算表 | 同科目/金額 0/負→4xx 帶原因 | E-6 E2E D1–D3 |
| 批量覆核 | approve→離開 PENDING_REVIEW＋逐列稽核 | 缺 WRITE→403 | E-7 E2E D4＋`bulk_review_*` |
| 搜尋 | 46ms 命中（BM25）；空態正常 | >200 字元截斷；離線嵌入≤2s | E-8 E2E D5＋`search_caps_*` |
| 分頁 | page=2 內容不同 | page=abc→夾回合法頁 | E-9 E2E D6 |
| 不存在 ID | 404 頁 | — | E-10 E2E D7 |
| XSS | 描述轉義 | 原始 `<script>` 禁出現 | E-11 E2E D8 |

### TT-3 匯入與草稿
| 操作 | 正常結果 | 錯誤路徑 | 證據 |
|---|---|---|---|
| CSV 上傳預覽 | 200 預覽頁（不動帳） | 無 WRITE→403 | E-12 E2E D10＋`import_preview_*` |
| 確認匯入 | 303＋部分成功訊息（失敗列可修） | 冪等重放→跳過 | E-13 `import_confirm_is_replay_guarded` |
| 草稿採用 | 決不直接過帳 | — | E-14 `ai_suggestion_adopt_never_posts_to_ledger` |

### TT-4 銀行支付（Mock 模式，誠實標示）
| 操作 | 正常結果 | 錯誤路徑 | 證據 |
|---|---|---|---|
| 建批匯出 | 303＋批次 SUBMITTED | 缺 READ→403 | E-15 E2E（本輪新增） |
| 匯入回應 | 303＋對帳 | 檔案不存在→400 | E-16 表單契約 |
| 狀態轉換驗證 | 200 JSON（純驗證，無寫入→免 CSRF） | 非法轉換→valid=false | E-17 `bankpay_api` 測試 |

### TT-5 治理與系統
| 操作 | 正常結果 | 錯誤路徑 | 證據 |
|---|---|---|---|
| 管理頁（users/roles/sessions/system） | ROLE_MANAGE→200 | 無權→403 帶原因（**伺服器端**，非僅導覽隱藏） | E-18 `rbac_matrix_server_side_gates`＋RBAC 矩陣 264 組 |
| 建使用者→登入→停用 | 全流程可逆 | 自我停用→拒 | E-19 `admin_creates_user_who_can_then_login`、`admin_cannot_deactivate_self` |
| 全站連結 | 77+ 內部連結全可達 | 死連結→建置期紅 | E-20 `internal_links_all_resolve` |
| 頁面殼 | 全 manifest 頁符合 L0–L1 契約 | — | E-21 `all_manifest_pages_render_under_contract` |

## 2. 問題來源清單 → 分級

| 來源 | 例 | 預設分級 |
|---|---|---|
| S-01 需求側 | 需求矛盾、漏列角色 | P1 |
| S-02 規格側 | gate 未定義、路徑模板 | P1 |
| S-03 實作側 | 閘漏掛、env 錯置 | **P0** |
| S-04 資料側 | 重複帳戶、借貸反轉 | **P0** |
| S-05 併發/狀態 | 遷移版本碰撞、冪等 | P1 |
| S-06 環境側 | 埠/exe 共用、LF/CRLF | P1 |
| S-07 安全側 | CSRF、XSS、越權 | **P0** |
| S-08 人的因素 | 誤導訊息、文案不清 | P2 |

分級規則：P0＝安全/正確性/資料完整（**先修**）；P1＝功能完整性/可靠性；P2＝體驗；P3＝潤飾。本輪已修 P0：RBAC 伺服器端閘、稽核溯源、FK 硬化、借貸平衡；P1：日誌、備份、斷路器。

## 3. 細節二十五問（每個功能組的強制掃描清單）

| # | 問題 | TT-1 | TT-2 | TT-3 | TT-4 | TT-5 | 覆蓋證明 |
|---|---|---|---|---|---|---|---|
| 1 | 空值/NULL | ✓ | ✓ | ✓ | ✓ | ✓ | 快照欄位 NULL 語義；E2E |
| 2 | 空字串/空集合 | ✓ | ✓ | ✓ | ✓ | ✓ | E-8 空態；`accounts_page_lists_groups_and_empty_state` |
| 3 | 單一元素 | — | ✓ | ✓ | ✓ | ✓ | 分頁邊界測試 |
| 4 | 超大輸入 | ✓ | ✓ | ✓ | ✓ | ✓ | 64KiB body cap、5000 字元查詢 |
| 5 | 超長字串 | ✓ | ✓ | ✓ | — | ✓ | 512 UA 截斷、200 查詢上限 |
| 6 | 特殊字元/Unicode | ✓ | ✓ | ✓ | ✓ | ✓ | E2E 中文+CRLF CSV |
| 7 | 型別錯置 | — | ✓ | ✓ | ✓ | — | sqlx 型別＋CAST；測試 |
| 8 | 負數/零 | — | ✓ | ✓ | ✓ | — | E2E D3（min=0 + 伺服器拒） |
| 9 | 溢位 | — | ✓ | — | ✓ | — | amount_minor i64；`money_props` proptest |
| 10 | 小數精度 | — | ✓ | ✓ | ✓ | — | INV-01 整數分；`money_parser` |
| 11 | 日期邊界 | — | ✓ | ✓ | — | — | DATE CAST；月末報表測試 |
| 12 | 編碼 | ✓ | ✓ | ✓ | ✓ | ✓ | C-09 UTF-8 無 BOM；CSV 實測 |
| 13 | 路徑注入 | — | — | ✓ | — | ✓ | include_str! 零檔案路徑輸入 |
| 14 | 重複提交/冪等 | — | — | ✓ | ✓ | — | `import_confirm_is_replay_guarded` |
| 15 | 併發競爭 | — | ✓ | ✓ | — | ✓ | busy_timeout；WAL；遷移鎖 |
| 16 | 部分失敗 | — | ✓ | ✓ | ✓ | ✓ | 匯入部分成功訊息；bulk mixed outcomes |
| 17 | 逾時/重試 | ✓ | — | — | ✓ | ✓ | 嵌入 2s＋斷路器；busy_timeout |
| 18 | 權限不足 | ✓ | ✓ | ✓ | ✓ | ✓ | RBAC 矩陣 264 組 |
| 19 | 未登入 | ✓ | ✓ | ✓ | ✓ | ✓ | 303/401 測試；矩陣 anon 檢查 |
| 20 | 會話過期中操作 | ✓ | ✓ | ✓ | ✓ | ✓ | `revoked session must be rejected` |
| 21 | 資源耗盡 | — | — | — | — | ✓ | body cap；連線池 5 |
| 22 | 依賴失效 | ✓ | — | ✓ | ✓ | ✓ | 嵌入斷路器；BM25 降級 |
| 23 | 亂序操作 | — | ✓ | ✓ | ✓ | — | replay guard；狀態機拒非法轉換 |
| 24 | 非法狀態轉換 | — | ✓ | ✓ | ✓ | — | `validate_transition` 測試 |
| 25 | 可觀測性 | ✓ | ✓ | ✓ | ✓ | ✓ | obs 日誌；稽核事件 |

**覆蓋證明規則**：每問至少一項 E-xx 或 Rust 測試對應；「—」＝該組無此風險面（需在 PR 審查時說明而非空白）。NOT TESTED 者必須列於追蹤矩陣 §7。

## 4. 文件質量 rubric（文件驗收五問）

1. 每個主張有來源或測試編號？2. 數字附量測方法＋日期？3. 未完成標 NOT TESTED？4. 與程式碼同步（trace_guard 自動驗）？5. 讀者能照做（命令可複製）？

## 5. 自動校驗

`platform/tests/trace_guard.rs`：解析 `REQUIREMENTS_TRACEABILITY.md` → 逐列驗證 (a) 實作/測試欄引用的檔案存在 (b) 測試名在程式碼中存在 (c) 判定 ∈ 五級。**文件與實作脫鉤 = 建置紅燈**（總控指令 §13「不得讓文件、規格、架構與實作互相脫鉤」的機械化）。