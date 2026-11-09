# Session: 20261107-dev-flaps-completion

- Project(s): root workspace (Flaps ERP 工作台 — 部門業務工作台平台：core/platform/web)
- Goal: Excute REDESIGN_BLUEPRINT v2.0 completion plan (S0–S7): incident repair, P1–P5 pages, M4–M6 modules, quality gates, delivery.
- Created: 2026-11-07T12:50:00Z
- Owner: dev-flaps-completion
- git_head_start: 9704933 (working tree with repaired api.rs, uncommitted)
- dirty_start: true (parallel streams' WIP: kem/knowledge; api.rs repair staged)

## Three-platform frame (user directive 2026-11-07)

1. 部門業務工作台（本 session 主責）— `platform/` + `web/` + `core/`
2. API 中樞與資料管理平台（含資安）— `platform-hub/` + API/整合層（並行流）
3. 治理與稽核平台 — `crates/kem/` + `knowledge/` + 審計/治理（並行流）

## Incident ledger (api.rs encoding corruption)

| Time (+0800) | Event | Evidence |
|---|---|---|
| 18:20 | `e82a49e` corrupts api.rs (UTF-8→CP950 re-encode; BOM) | blob df3b20cc, 157,455 B, PUA=712 |
| 18:46 | `c6c1d54` stress seed 100k→1k (transient) + blank lines | git show |
| 18:59–19:01 | `c0790da`/`2d6411b` blank-line appends | git show |
| 19:02 | `8b8b92b` stress restored 100k + assert (deliberate final) | git show |
| 19:12 | `bca0ab7` whole-file rewrite accident (CRLF 4,201 CRs, BOM) | blob 9e9fcd9, 161,029 B, PUA=713 |
| 19:12 | HEAD: 28 rustfmt parse errors → platform unbuildable | rustfmt --check |
| 19:20–20:07 | Repair: byte-restore from `dc414ac`/ffc4036 + rustfmt; guards vs races | PUA=0 / parse=0 / fmt=0 |
| 20:15 | `cargo check -p platform` EXIT=0 (16.04s) | tests/20261107-api-rs-repair.log |
| 20:40 | `3e0db70` (parallel stream) reverts CRLF layer 9e9fcd9→7f81419 | git show |
| 20:4x | Commit `d8429af` (scoped: api.rs only); HEAD blob == ffc4036 | git rev-parse |

## Notes

- 10 commits (e82a49e..9704933) individually inspected: corruption lineage contained no other semantic deltas besides stress-value churn + blank lines + CRLF + BOM.
- Preserved: `dc414ac`/`8b8b92b` deliberate calibration (100k + BEGIN/COMMIT + assert 100k).
- Next: P1–P5 pages (start `/states`), R5 encoding guards (PUA/BOM scan test), M4–M6, S5 tests, S6 ops.

## M1–M3 re-implementation (2026-11-07)

- Blueprint §5 claimed M1 (pagination) / M2 (health) / M3 (KPI) done 2026-11-06 with tests.
- **Verified ABSENT in main**: `git log -S 'parse_page' --all` empty; `-S 'degraded'` empty; all worktrees scanned (local-develop/release/test, booming-radon) — no matches; no pagination/health tests existed. Recorded as spec-vs-reality divergence (plan §12.3 rule: 現況＋套件本文為準); re-implemented to blueprint spec §M1/§R2/§M3.
- Implemented: M1 fail-closed page parsing (`parse_page_param`/`clamp_page`) + bound LIMIT/OFFSET + `第 X / Y 頁・共 Z 筆` + prev/next with disabled states + dashboard 5-row overview split; M2 health probe (`db` + `migrations_applied` + `transactions`; 503 `status=degraded`); M3 `pending_review` = `COUNT(status='PENDING_REVIEW')` rendered as a real number.
- Evidence: `cargo test -p platform --locked` → **54 passed / 0 failed** (incl. `stress_10000` @100k rows, 14.2s suite); `cargo clippy -p platform --all-targets -- -D warnings` EXIT=0; rustfmt clean; api.rs normalized to LF (0 CR).
- New tests: `parse_page_param_is_fail_closed` (8 assertions), `pagination_splits_25_rows_into_20_plus_5`, `pagination_clamps_out_of_range_page_to_last`, `pagination_falls_back_to_page_1_on_garbage`, `health_degrades_to_503_when_db_unreachable`, `pending_review_kpi_shows_real_count`; `health_reports_database_status` extended.

## P1/P2 — /transactions list + detail (2026-11-09)

- Added `GET /transactions` (bound LIKE search `?q=`, paged `?page=` reusing M1 helpers, status badges colour+text, search-preserving paging links via `percent_encode`) and `GET /transactions/:id` (entry legs, i128 totals, debit=credit badge, three-part 404) per UI-UX §E3.4; nav links to the new page; two new templates (`transactions.html`, `transaction_detail.html`).
- Verification (isolated): temp worktree at `b736444` + hunk-level patch of this session's changes only → `cargo clippy -p platform --all-targets -- -D warnings` EXIT=0 (7.24s); `cargo test -p platform --locked` → **58 passed / 0 failed** (incl. `stress_10000`) + 4 + 3 integration.
- Coexistence: the parallel stream's BankPay/audit WIP lives in the same working files (api.rs, index.html, lib.rs; see commit `b736444`) and was excluded from this commit via hunk-level staging (`git apply --cached` of an isolated patch); their working-tree state was not modified. Their BankPay module currently needs `time` + `reqwest/multipart` deps before the crate build is green again.

## BankPay/status/audit integration + R5 guards + encoding repairs (2026-11-09)

- **Integration (feature-gated)**: parallel-stream BankPay module + status engine + audit wiring integrated so the DEFAULT build compiles with zero new deps. `bankpay = ["dep:time", "reqwest/multipart"]`; lib.rs/api.rs imports, AppState fields, and routes are cfg-gated (routes moved to a cfg'd Router segment so auth/body-limit layering still wraps them when enabled). Dashboard audit query fixed to the live audit_log schema. Partner WIP backup: `partners-wip-backup-20261109/`.
- **Isolated verification** (worktree @ c0a015b + integration files only): `cargo check` EXIT=0; clippy -D warnings EXIT=0; tests 58/0 + 4 + 3.
- **Repairs (R5 findings)**: 4 `api_*.html` transcoded losslessly CP950→UTF-8 (content intact, wrong encoding); engine.rs PUA-damaged test messages repaired; 11 BOMs stripped (com/db/search/sync + migration 0005).
- **R5 guards added** (`platform/tests/guards.rs`): encoding-damage scan (BOM/NUL/FFFD/PUA) + zero-JS scan (`<script>` / inline handlers) across core/src, platform/src, templates, migrations, web/assets. Guards green post-repair.
- **Partner live overlay**: ~12:56 the partner stream resumed editing api.rs (TDD red tests `detail_back_link_preserves_list_context`, `pagination_links_preserve_search_term`, plus a test fn awaiting `#[test]`); their live edits were NOT committed and remain untouched on disk for their cycle.
- Residual: `--features bankpay` build still carries the partner's in-flight gaps (response-type bindings, config binding, BankPayStatus visibility) — theirs to finish; default build unaffected.
- Metrics: api.rs 5,161 lines (awaiting R1 split) | bankpay/ 918 | status.rs 253 | templates 18 | platform tests 58+4+3 green.

## PRE-R1a — route characterization (PIN) suite (2026-11-09)

- New `platform/tests/characterization.rs` (6 tests, 27 endpoints): public contract (login/health/css); anonymous protected GETs → 303 /login (12 routes); authenticated GET markers (7 pages) + report pages + CSV (`?ym=YYYY-MM`); detail 404; anonymous POSTs → 303 (10 routes); bad-CSRF behavior (exact 403 for /transactions with a complete form; 4xx for the other 9); logout never-5xx.
- Calibrated fail-closed contracts recorded: monthly report/CSV require `?ym=` (400 otherwise); the Form extractor rejects incomplete bodies (422) before the handler-level CSRF check.
- Role: safety net required by the partner plan §PRE-R1a (`1793787631643`) before any api.rs module split; the refactor must keep these assertions unchanged.
- Evidence (isolated worktree @ eb522b2 + this file only): clippy -D warnings EXIT=0; characterization 6/6 passed. Main-tree full suite also green at capture: 65 lib + 6 char + 4 + 2 guards + 3.
- Observed, not ours, not touched: partner's live edit at `api.rs:1703` carried a clippy `manual_pattern_char_comparison` violation at capture time (their batch-adjudication work); partner gates will surface it.
## BankPay 整合收斂與 E2E 驗證（2026-11-09 下午，review/verification 流）

- 修復 BankPay/status 模組 37 編譯錯誤至綠燈（`Serialize` 推導、const 轉 match 靜態切片、`StatusEngine: Clone`、bool 誤用 unwrap 移除、`permissions` 可見性）；該輪 `--features bankpay` 測試 13/13 通過（含新增 `erp_config` 持久化 3 測試）。
- `erp_config` 設定持久化落地：`bankpay::{load,save}_config`（冪等 UPSERT，`config_id`=`bankpay:{key}`）＋ POST 先持久化後記憶體（fail-closed）＋啟動載入；runtime smoke（登入→`/api/status`→config POST/GET→erp_config 5 列）通過（14:26-14:27）。
- 稽核 NULL 毒化修復：`api.rs`（儀表板最近稽核）＋ `cross_module_api.rs`（audit_list／audit_export）三處 `COALESCE`。
- 分頁語義確認：儀表板頁尺寸=`LIST_PAGE_SIZE`(20)（非 5；5 僅供未分頁 recent 概觀）；`clamp_page` 夾限以 runtime smoke 驗證。
- 新增 `platform/tests/bankpay_persistence.rs`（21 筆→`第 2 / 2 頁`、超頁夾限、config 持久化跨「重啟」、session 跨重啟皆斷言；已適配 `X-CSRF-Token` 契約）；**PASS**：`cargo test -p platform --test bankpay_persistence` 1 passed / 0 failed（1.16s，15:44）；同輪全平台 97+16 測試 0 失敗、fmt EXIT=0。clippy 12 警告全在並行流收斂中新碼（bankpay/api.rs 10×`result_large_err`＋2×`div_ceil`），建議以 `ApiError{status,message}` 小結構＋`impl IntoResponse` 修復，由該串流重跑。
- 觀察：並行流架構收斂 — `time`→`chrono`、移除 `bankpay` feature 閘控（零新增依賴）、bankpay JSON API 加 `X-CSRF-Token`、audit 匯出加 CWE-1236 公式注入防護。