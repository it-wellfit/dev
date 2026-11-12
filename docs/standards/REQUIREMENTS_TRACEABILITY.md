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
| S-02 | 稽核事件可溯源（IP/User-Agent） | migration 0009 加 `ip_address`/`user_agent` + 索引；`pages::write_event_ctx` | `migrations/0009_audit_request_context.sql`、`pages.rs`、`auth.rs`（LOGIN_SUCCEEDED/FAILED、LOGOUT） | 備份快照欄位實證（`PRAGMA table_info`） | 快照含兩欄；登入/登出寫入 | PASS |
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

## 7. 未測／已知限制（誠實揭露）

| 項目 | 狀態 | 說明 |
|---|---|---|
| 65KiB 以上 POST 被拒（S-04） | NOT TESTED | 掛載已確認；未執行負向測試 |
| 還原演練（restore drill）歸檔（D-01） | NOT TESTED | 備份 CLI 已實證；未建立標準還原程序文件與演練紀錄 |
| 並行建置下壓力測試 | PARTIAL | 併行 cargo 建置時偶爾超 500ms（負載假象）；隔離重跑 439ms |
| 多因素認證（MFA） | NOT IMPLEMENTED | 密碼政策已對齊 NIST，MFA 未上路（密碼政策門檻 12 為此情境之務實下界） |
| 外部 embedder 實際可用時的 hybrid 效果 | NOT TESTED | 端點未設定（斷路器確保不卡）；BM25-only 已驗證 |