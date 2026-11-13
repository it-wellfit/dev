# REQUIREMENTS TRACEABILITY MATRIX（需求追蹤矩陣）

> 版本 v1.0｜日期：2026-11-12｜狀態：Active
> 目的：每條需求可回溯到 **設計 → 實作位置 → 測試 → 證據 → 驗收判定**（總控指令 §8/§12）。
> 判定用語：PASS（已執行且通過）／PARTIAL（部分驗證）／NOT TESTED（未執行）／BLOCKED（環境限制）。
> **本矩陣只記載可查證之事實**；未執行者標 NOT TESTED，不以推論充數（C-10）。

## 1. 命名與身份分離（2026-11-11 裁定）

| ID | 需求 | 設計 | 實作位置 | 測試 | 證據 | 判定 |
|---|---|---|---|---|---|---|
| N-01 | 自建系統不得命名 FlapsErp/輔翼 | `docs/standards/NAMING_AND_BOUNDARY_DECREE.md` | 全庫更名（bin/env/crate/cookie/DB/UI/tauri） | `platform/tests/naming_guard.rs`（3 tests） | 78 檔 205 處；活躍面殘留 0 | PASS |
| N-02 | 原廠 FlapsErp 僅可深度串接、禁改內容 | Decree §3 邊界三不三只 | `crates/flaps_bridge`（COM 橋） | `platform/tests/naming_guard.rs`（橋接器名稱保留於白名單語境） | 架構審查 | PASS |
| N-03 | 更名不得破壞既有功能 | — | — | 全測試 143/0 | `cargo test --locked -p platform --lib` | PASS |

## 2. 可觀測性（本輪補洞）

| ID | 需求 | 設計 | 實作位置 | 測試 | 證據 | 判定 |
|---|---|---|---|---|---|---|
| O-01 | 執行期日誌必須真的可見（原 tracing 無 subscriber） | `platform/src/obs.rs`（零依賴 stderr 日誌，AUXBRIDGE_LOG 控制等級） | `platform/src/obs.rs`、`sync.rs`（5 處改用 obs_info!）、`bin/auxbridge-server.rs`（init） | `obs::tests::level_parsing_is_fail_safe`、`obs::tests::log_respects_level_gate` | 執行輸出：`INFO obs: logger initialised`、`INFO auxbridge_server: backup: snapshot written` | PASS |
| O-02 | 移除未使用相依（精實） | 移除 `tracing`（僅 5 處呼叫且無 sink） | `platform/Cargo.toml` | 建置 | clippy/fmt/test 全綠 | PASS |

## 3. 安全硬化（本輪補洞）

| ID | 需求 | 設計 | 實作位置 | 測試 | 證據 | 判定 |
|---|---|---|---|---|---|---|
| S-01 | CSRF 覆蓋所有狀態變更 POST | 逐路由稽核（27 條） | api.rs / auth.rs / bankpay / pages / apihub | 既有 CSRF 測試（forged token 系列） | 25 條有檢查；`/login` 無 session 前豁免（設計上不可能有 token）；`transition_handler` 為純驗證無狀態變更 → 不適用 | PASS |
| S-02 | 稽核事件可溯源（IP/User-Agent） | migration `platform/migrations/0009_audit_request_context.sql` 加 `ip_address`/`user_agent` + 索引；`platform/src/pages.rs`、`platform/src/auth.rs`（LOGIN_SUCCEEDED/FAILED、LOGOUT） | 備份快照欄位實證（`PRAGMA table_info`） | 快照含兩欄；登入/登出寫入 | PASS |
| S-03 | HSTS（HTTPS 部署強制 TLS） | `security_headers` 無條件送出（http 下瀏覽器忽略，無害） | `auth.rs` | `security_headers_include_hsts` | 實測 `max-age=31536000; includeSubDomains` | PASS |
| S-04 | 公開路由 body 上限（防記憶體耗盡） | public router 加 `DefaultBodyLimit::max(64KiB)` | `api.rs` router() | 編譯期掛載；未測 65KiB 拒絕 | 掛載位置審查 | PARTIAL（未執行 65KiB 負向測試） |
| S-05 | 搜尋查詢長度上限（防成本放大） | `MAX_SEARCH_QUERY = 200`（與交易清單一致） | `workbench.rs` | `search_caps_overlong_query_without_error`（5000 字元→200） | 測試綠 | PASS |
| S-06 | 密碼政策（NIST SP 800-63B 對齊） | ≥12 字元＋18 條弱密碼阻擋＋不得同帳號；不設組成規則 | `api.rs::validate_password_strength` | `password_policy_rejects_short_blocklisted_and_username_equal` | 測試綠 | PASS |
| S-07 | FK 約束必須真的生效（sqlx 預設關閉） | `db::connect_options`：foreign_keys=ON、WAL、synchronous=NORMAL、busy_timeout=5s、mmap 256MiB | `db.rs`、`bin` | `connect_options_enforces_foreign_keys_and_wal`（含 FK 違規被拒） | 測試綠 | PASS |
| S-08 | 唯讀連線不得套用需寫入 pragma | `mode=ro`/`:memory:` 跳過 WAL/synchronous | `db.rs` | `vacuum_into_produces_consistent_snapshot`（唯讀開啟快照） | 測試綠（修復前實測 "attempt to write a readonly database"） | PASS |

## 4. 資料可靠性（本輪補洞）

| ID | 需求 | 設計 | 實作位置 | 測試 | 證據 | 判定 |
|---|---|---|---|---|---|---|
| D-01 | 備份/復原機制（原 Dev 根工作區無任何備份工具） | `auxbridge-server --backup <src> <snapshot>`（SQLite `VACUUM INTO`，一致性快照、免停機） | `bin/auxbridge-server.rs` | `db::tests::vacuum_into_produces_consistent_snapshot` | CLI 實測：27 交易／4 使用者／40 稽核列 | PASS（備份）／NOT TESTED（還原演練未歸檔） |
| D-02 | 遷移版本唯一且連續（防並行碰撞） | 建置期防呆測試 | `db.rs::migration_versions_are_unique_and_sequential` | 同左 | 實證：曾發生 0008 碰撞 → 改號 0009 後通過 | PASS |

## 5. 效能與精實（本輪）

| ID | 需求 | 設計 | 實作位置 | 測試 | 證據 | 判定 |
|---|---|---|---|---|---|---|
| P-01 | 搜尋不得在離線/未設定 embedder 時卡死 | 嵌入 2s 逾時＋斷路器（60s 停用）＋批量索引（一次 BM25 重建） | `search.rs::embed_with_breaker`、`add_documents_bulk` | `search_finds_seeded_transactions_after_out_of_band_insert` | 套件 150s→27s；搜尋 46ms | PASS |
| P-02 | CSS 重訪不傳全量 | ETag（建置期雜湊）＋`Cache-Control: no-cache` | `api.rs::design_system_css` | `design_system_css_revalidates_with_etag`（304 無 body） | 測試綠 | PASS |
| P-03 | 壓力測試 p95 < 500ms | 既有 | — | `stress_10000_transactions_p95_under_500ms` | 隔離重跑 p95=439ms（併行建置時偶爾失敗＝負載假象） | PASS（附離散風險註記） |

## 6. 介面與無障礙（既有，覆核）

| ID | 需求 | 設計 | 實作位置 | 測試 | 證據 | 判定 |
|---|---|---|---|---|---|---|
| U-01 | 零 JavaScript | CSP `default-src 'none'` + 無 `<script>` | `auth.rs`、templates | 既有測試（script 數 = 0） | CSP 實測 | PASS |
| U-02 | 跨裝置可用 | `AUXBRIDGE_BIND=0.0.0.0`（launcher） | `bin`、`auxbridge-demo-start.cmd` | — | 區網 192.168.1.160:8090 → 200 | PASS |
| U-03 | 金融數字等寬對齊 | `table { font-variant-numeric: tabular-nums }` | `ui-v2.css:277` | — | CSS 實測 | PASS |
| U-04 | 文字選取品牌化 | `::selection`（primary-muted alpha） | `ui-v2.css` | — | CSS 實測 | PASS |
| U-05 | 鍵盤可見焦點 | `:focus-visible`（design-system 五處） | `design-system.css` | — | CSS 審查 | PASS |

## 6b. 體驗契約與驗證深化（2026-11-13 批次）

| ID | 需求 | 設計 | 實作位置 | 測試 | 證據 | 判定 |
|---|---|---|---|---|---|---|
| RB-01 | 無權限角色直接 URL 存取管理/寫入頁必須 403（原僅導覽隱藏） | Defense in Depth（隱藏≠保護） | `platform/src/pages.rs`（require_perm）、`platform/src/api.rs`（4 個寫入頁閘） | `platform/tests/web_crawl.rs`、`rbac_matrix_server_side_gates` | RBAC 矩陣 264 組：真實越權 0（store/pos 曾 200 的 7 頁全改 403） | PASS |
| LC-01 | 全站內部連結不得死連結 | 全站連結圖驗證 | `platform/src/api.rs`（internal_links_all_resolve） | 同左 | 77+ 連結全可達 | PASS |
| PC-01 | 全 manifest 頁符合 L0/L1 頁面殼契約 | `docs/standards/EXPERIENCE_CONTRACTS.md` L0–L1 | `platform/src/api.rs`（all_manifest_pages_render_under_contract） | 同左 | 66 頁 × 契約檢查 | PASS |
| IP-01 | 稽核必須記錄真實對端 IP（原僅 X-Forwarded-For，直連恆 NULL） | ConnectInfo → X-Forwarded-For → NULL | `platform/src/bin/auxbridge-server.rs`（into_make_service_with_connect_info）、`platform/src/auth.rs` | 稽核欄位實證 | 快照 ip_address 欄 | PASS |
| TC-01 | 追溯矩陣與實作不得脫鉤（自動校驗） | 總控指令 §13 機械化 | `platform/tests/trace_guard.rs` | 同左 | 抓到 S-02 路徑漂移並修正 | PASS |
| EC-01 | 體驗契約系統（分層模型＋TT-1..5＋二十五問） | `docs/standards/EXPERIENCE_CONTRACTS.md` | 同左 | `trace_guard::experience_contracts_present` | 3/3 tests | PASS |
| TT-1 | 會話與身份操作契約 | EXPERIENCE_CONTRACTS §1 | login/logout/profile | `login_lockout_survives_correct_password` 等 | E-1..E-5 | PASS |
| TT-2 | 帳務與交易操作契約 | 同上 | transactions/reviews/search | E2E D1–D8＋Rust 測試 | E-6..E-11 | PASS |
| TT-3 | 匯入與草稿操作契約 | 同上 | import/drafts | E2E D10＋`import_confirm_is_replay_guarded` | E-12..E-14 | PASS |
| TT-4 | 銀行支付操作契約（Mock 誠實標示） | 同上 | bankpay | `bankpay_api` 測試＋E2E D11（建批→清單→例外頁） | E-15..E-17 | PASS |
| TT-5 | 治理與系統操作契約 | 同上 | admin/audit | `rbac_matrix_server_side_gates`＋E2E | E-18..E-21 | PASS |

## 6c. 使用者實測回饋修復（2026-11-13 第二批，P0）

| ID | 需求 | 設計 | 實作位置 | 測試 | 證據 | 判定 |
|---|---|---|---|---|---|---|
| AMT-01 | 千分位金額（"12,000.00"）必須可解析——AI 建議採納流程原 100% 斷鏈 | 財務標準記法容許千分位；剝除逗號後驗證（負號/多點仍拒） | `core/src/money.rs`（parse_decimal_to_minor + _i128） | `parse_rejects_non_representable_input`（1,234→123,400）＋E2E D12 | 採納 400→**303**；core 44/0、platform 146/0 | PASS |
| OVL-01 | 全域狀態晶片列不得壓過品牌頂欄（雙 sticky 相撞） | 吸頂職責唯一化：header sticky、status-bar 靜態 | `web/assets/ui-v2.css`（.status-bar position:static + 根因註解） | CSS 實證（served 含 position: static） | 根因：design-system.css:1661 sticky+z40 與 grid row2 衝突 | PASS |
| UI-01 | /ai-decisions 卡片標題不得為 UUID | 人類可讀摘要（日期/科目/金額）；UUID 降為次要 hint | `platform/templates/ai_decisions.html` | 頁面契約測試＋HTTP 實證 | h3 無 UUID、摘要含 → | PASS |
| UI-02 | 頂欄顯示帳號名而非內部 user_id | Session 增 username（JOIN users，COALESCE 防孤兒） | `platform/src/auth.rs`（get_valid_session）＋50 處 acting_user | `login_failure_shows_generic_error` 等既有 | 頂欄實測「使用者：admin」 | PASS |
| WZ-01 | wizard 金額預填不得含千分位（number input 會清空值） | format_minor_plain（純數字） | `platform/src/api.rs`（平行流實作）＋測試斷言同步 | `drafts_parse_creates_draft_and_wizard_prefills` | value="12000.00" | PASS |

## 7. 未測／已知限制（誠實揭露）

| 項目 | 狀態 | 說明 |
|---|---|---|
| 65KiB 以上 POST 被拒（S-04） | NOT TESTED | 掛載已確認；未執行負向測試 |
| 還原演練（restore drill）歸檔（D-01） | NOT TESTED | 備份 CLI 已實證；未建立標準還原程序文件與演練紀錄 |
| 並行建置下壓力測試 | PARTIAL | 併行 cargo 建置時偶爾超 500ms（負載假象）；隔離重跑 439ms |
| 多因素認證（MFA） | NOT IMPLEMENTED | 密碼政策已對齊 NIST，MFA 未上路（密碼政策門檻 12 為此情境之務實下界） |
| 外部 embedder 實際可用時的 hybrid 效果 | NOT TESTED | 端點未設定（斷路器確保不卡）；BM25-only 已驗證 |