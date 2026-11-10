# Cross-Project Plan and Decision Register

**Reviewed:** 2026-11-02  
**Addendum:** 2026-11-03 (root-workspace baseline restoration; see "2026-11-03 addendum" below)  
**Scope:** Document authority, plan overlap, decision provenance, and current evidence across ARES, Wellfit, ekpi-platform, and SyncMesh.  
**Status:** Review register only. It does not approve product scope, implementation, certification, or release.

This register is the navigation point for planning documents. Each project retains its own requirements and acceptance evidence. Do not copy project-specific plans into this file or infer implementation approval from a plan's existence.

## Project authority and disposition

| Project | Keep as primary navigation / evidence | Current disposition | Required next decision or evidence |
|---|---|---|---|
| ARES (workspace root) | [ARES master spec](ARES_MASTER_SPEC.md), [implementation plan](ARES_IMPLEMENTATION_PLAN.md); the root `Cargo.toml` and source are the factual workspace baseline | Both planning documents remain proposals. The current workspace uses `resolver = "2"` and `core/`, `platform/`, `web/`; the proposed Cargo design describes a different crate layout and has an inconsistent member/API inventory. No approved executable plan is evidenced. | The owner must decide whether ARES replaces, coexists with, or is separate from the current root workspace; define the REA fork/provider boundary; then reconcile the Cargo proposal and acceptance evidence. |
| Wellfit | [Wellfit plan register](../Wellfit/PLAN.md), [BankPay decisions](../Wellfit/docs/bankpay/08-open-questions-and-decisions.md), [project evidence snapshot](../Wellfit/docs/project/CURRENT_STATE.md) | The plan register is current navigation; the project evidence snapshot is historical. The local E.Sun V2.8 PDF supports API-first integration; real-account/sandbox and end-to-end validation remain blocked. The BankPay quality review records that CLI export currently does not dispatch a network request; API protocol selection is not evidence of working integration. ISMS scope and applicability remain owner decisions. | Approve the ISMS boundary and accountable owner. Re-run project checks on a named revision before updating any current-state or completion claim. |
| ekpi-platform | [Documentation map](../ekpi-platform/docs/README.md), [phase gates](../ekpi-platform/docs/quality/phase-gates.md), and revision-bound [assurance evidence](../ekpi-platform/docs/assurance/README.md) | Phase A is not approved and Phase B is blocked. Existing assurance applies to revision `279c919`; the reviewed checkout is dirty at `c0c0e33`, so prior test evidence does not establish current-tree status. No approved delivery roadmap was found. | Reconcile README/charter claims to the canonical assurance record. Obtain named product, pilot, security, and data owners; rerun evidence on the current revision before claiming any gate passed. |
| SyncMesh | [Project charter](../SyncMesh/PROJECT_CHARTER.md) for intent; [`Cargo.toml`](../SyncMesh/Cargo.toml) and source for implementation facts; [traceability ledger](../SyncMesh/docs/TRACEABILITY.md) for acceptance status | Prototype status only. The manifest declares version `0.1.0` and four crates; charter/spec completion and version claims conflict. The 14 target acceptance cases are `NOT RUN`; no release sign-off is evidenced. **Security documents claim TLS/mutual authentication, but the inspected transport is TCP-only with no TLS implementation.** | Resolve intended release/version and feature scope; correct security claims before any deployment; confirm ADRs and name acceptance owners. Record actual test commands, results, and supported-platform evidence before updating release claims. |

- **2026-11-05 Unified integration plan** — `.kilo/plans/1792461065184-unified-approval-channel-integration.md` (v1.1). Defines ekpi-platform as governance/approval hub, platform-hub as integration layer, momo-first ecommerce skeleton, Knowledge base as standalone git repo. Phase 0 (KB version control) and Phase A (ADR-0002/0003) in progress.

## Decisions supported by current evidence

| Topic | Recommended decision | Evidence and limit |
|---|---|---|
| E.Sun integration mode | Use the bank security-component API as the canonical boundary; default to the JSON payload (`formatType=2`). Use the fixed-width file payload (`formatType=1`) only for the one-debit/one-credit case specified by the bank document. Do not design a file-drop-only integration. | The local [E.Sun V2.8 technical document](../Wellfit/docs/bankpay/references/【玉山銀行】企業ERP銀企直連串接_技術文件V2.8_v1.pdf) documents API rules, both payload types, and a separate media-file specification. This resolves the architecture-description conflict, not bank account access or live acceptance. |
| Wellfit ISMS documents | Recommended hierarchy: `KNB-MASTER.md` is a short governance/index document; `KNB-ISO-MASTER.md` is the sole detailed ISO-control reference; `KNB-INDUSTRY-MASTER.md` is an applicability proposal. Keep `001`–`010` as historical inputs, not active plans. | This is a document-management recommendation only. The responsible owner must approve organizational scope, standards, identity assumptions, and the execution roadmap. Version/date claims are not verified; the official ISO catalogue returned HTTP 403 during review, which means `UNKNOWN`, not pass or fail. |
| ERP/AI data boundary | Keep inference local and prohibit cloud fallback for ERP data until the data owner approves an explicit exception with data classification, provider, retention, access, and audit controls. | The Flaps architecture proposal states local-only data handling, while the Phase 2 plan proposes cloud fallback. The local-only constraint is the consistent, lower-risk default; changing it requires owner authorization. |
| “Latest technology” plans | Treat version tables and “latest” claims as dated observations, not architecture decisions. Adopt a dependency or platform only when the project owner approves the need and the exact version is verified against the project's manifest and an authoritative release source. | The EKPI migration proposal's proposed stack differs from current project manifests. Wellfit's encoding plans also contain speculative Base-N/envelope/analytics formats. No such adoption is implied by this review. |

## 2026-11-03 addendum (root workspace baseline + ADR-0001)

Executed under the ELITE PRODUCT ENGINEERING MASTER DIRECTIVE plan (`.kilo/plans/1793708660255-elite-product-engineering-execution-plan.md`), session `.kilo/sessions/20261103-dev-elite-impl/`. Root workspace is **not a git repository**, so evidence is command+exit-code bound, not revision bound (known gap, see ADR-0001 §5.3).

| Item | Disposition | Evidence |
|---|---|---|
| Baseline restoration | `cargo fmt -- --check` / `cargo clippy --all-targets -- -D warnings` / `cargo test --locked` / `cargo audit` all exit 0 (after fixing `core/src/permission.rs` syntax damage, 19 clippy errors, clippy-allow removal, reqwest 0.11→0.12 for RUSTSEC-2026-0258) | session `runs/` and `tests/` logs; 12 tests passed / 0 failed |
| ADR-0001 accepted | D-01: primary product = `ekpi-platform/` (OEKAP); D-02: root workspace keeps edition 2021 / axum 0.7 / SQLite; D-03: zero-JS HTML/CSS + PRG; D-05: SyncMesh claims corrected | `docs/adr/ADR-0001-tech-stack-baseline.md` |
| Zero-JS presentation layer | `web/index.html` rewritten: `<script>` count = 0, inline handlers = 0, design tokens in `web/assets/design-system.css`; live-checked in browser | session runs log; PRD FR-001/002 |
| Schema SSOT started | `platform/migrations/0001_baseline_schema.sql` + `sqlx::migrate!()` wiring in `platform/src/db.rs` with 3 replay/compat tests | `cargo test -p platform` 3 passed |
| Follow-up #1 (SyncMesh claims) | **Partially addressed**: `SyncMesh/docs/SECURITY.md` v1.1.0 corrects §1/§2/§5.1 to "TCP plaintext today; TLS 1.3 target, unverified (AT-008)". The `NOT RUN` acceptance ledger (AT-001..014) and release-scope decision remain open. | `SyncMesh/docs/SECURITY.md` §9 changelog |
| New governance documents | `AGENTS.md` (root), `docs/product/PRD.md`, `docs/product/USER_JOURNEYS.md`, `docs/product/IMPROVEMENT_BACKLOG.md`, `docs/security/THREAT_MODEL.md`, `docs/testing/TEST_STRATEGY.md`, `docs/engineering/SKILL_INVENTORY.md` | in place |

This addendum records execution facts only; it does not approve product scope, certification, or release.



## 2026-11-04 addendum #2 (Slice 5: P0-1 integer money + P1-1 API layer)

Executed under the ELITE plan §9, session `.kilo/sessions/20261103-dev-elite-impl/`. Evidence: command+exit-code bound (root workspace still not a git repo).

| Item | Disposition | Evidence |
|---|---|---|
| P0-1 integer money | `TransactionItem.amount_minor/exchange_rate_scaled` (i64), `flaps_core::money` exact parsing, migrations 0001 INTEGER schema, `format_minor` display | money tests 6, integer roundtrip test, full baseline EXIT=0 |
| P1-1 API layer | `platform/src/bin/flaps-server.rs` + `src/api.rs` (ADR-0002): GET / server-rendered (Askama), POST /transactions RBAC→validate→ledger→303 PRG, GET /api/health | curl smoke: 303 + Location /#transactions, GET renders `1,234.50` real data, 403 for STORE role, 400 for 0.005, `<script>` count 0; integration tests 8 |
| ADR-0002 accepted | Zero-JS API decisions incl. honest G-1 residual (X-Acting-User unauthenticated, RBAC still enforced) | `docs/adr/ADR-0002-api-layer.md` |
| Full baseline 2026-11-04 | fmt/clippy -D warnings/test --locked/audit all EXIT=0; 21 tests passed 0 failed | session tests/20261104-0200-slice5-test.log |

This addendum records execution facts only; it does not approve product scope, certification, or release.






## 2026-11-05 addendum #10 (Slice 12: demo kit + Phase 0 blueprint + Phase 2 readiness)

| Item | Disposition | Evidence |
|---|---|---|
| Demo kit | seed-demo/reset-demo scripts (repeatable, 10/10 [DEMO] transactions via real API); demo docs (GUIDE/SCENARIOS/DATA/LIMITATIONS) | scripts/seed-demo.ps1; docs/demo/ |
| MVP docs | MVP_SCOPE (MoSCoW) / MVP_ACCEPTANCE / MVP_TEST_PLAN / MVP_TEST_REPORT (90 passed, Not Run honestly listed) | docs/product/ + docs/testing/ |
| Phase 0 blueprint | 10 docs (vision/stories/roadmap/business-model/IA/system-arch/db-design/api-design/tech-stack/decisions) — consolidated, linked to existing ADRs | docs/product/ docs/design/ docs/architecture/ docs/engineering/ |
| Progress system | PROJECT_STATUS.md + NEXT_ACTIONS.md | docs/ |
| Phase 2 gate | PRODUCTION_READINESS.md Go/No-Go (NO-GO: env/TLS/restore-drill/monitoring/authorization) | docs/production/ |
| Session hygiene | Expired-session purge at startup | api.rs AppState::new |
| Baseline @ d6cf480 | fmt/clippy/test/audit all EXIT=0; 90 passed, 0 failed | session tests/20261104-2200-slice12-final.log |
| Plan v4 | Sections 16 (phase mapping) + 17 (Slice 12 record) + 18 (Slice 13 candidates) appended | .kilo/plans/1793708660255-...md |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-04 addendum #9 (completion audit close-out)

| Item | Disposition | Evidence |
|---|---|---|
| CSRF coverage 10/10 | import_preview (multipart csrf field) and ai_suggest gated; verified by per-handler audit | CSRF matrix scan |
| Value objects (DDD) | Money/AccountId in core/src/value_objects.rs with validation tests | core 30 tests passed |
| Pivot row cap (OQ-10-2) | top-20 by total, overflow folded into OTHER with truncation notice | pivot_caps_rows_and_folds_into_other_bucket |
| AI request audit (OQ-10-1) | AI_SUGGEST_REQUESTED with masked fields | api.rs ai_suggest |
| P1-8 residual UI | Admin password-reset form on admin page | admin_users.html |
| OQ-8-2 UI | AI suggestion form on drafts page | drafts.html |
| Baseline @ c932753 | fmt/clippy/test/audit all EXIT=0; 87 tests passed, 0 failed | session tests/20261104-2100-final-audit.log |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-04 addendum #8 (Slice 11: deployment hardening)

| Item | Disposition | Evidence |
|---|---|---|
| Security headers | CSP/nosniff/DENY/no-referrer/no-store middleware on every response (OWASP checklist) | security_headers_present_on_public_and_protected_responses |
| Session hygiene | Expired sessions purged at login (table bounded) | expired_sessions_are_purged_at_login |
| R-07 preparation | Config-driven Secure cookie flag (default OFF, tested) | login_sets_secure_cookie_when_configured |
| CSP-safe templates | 9 inline style attributes replaced with utility classes in both stylesheets | template grep = 0; :root sync test green |
| Baseline @ 26bff52 | fmt/clippy/test/audit all EXIT=0; 79 tests passed, 0 failed | session tests/20261104-1800-slice11-final.log |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-04 addendum #7 (Slice 10: multidimensional insight + methodology hardening)

| Item | Disposition | Evidence |
|---|---|---|
| S10-FR-1 pivot | /reports/pivot whitelisted dimensions (R-PIV-WHITELIST), matrix with totals | pivot_matrix_matches_hand_computed_values + pivot_rejects_unknown_dimensions_with_rule_name |
| S10-FR-2 parallel coordinates | core/src/parallel.rs integer min-max normalization; zero-JS SVG superior alternative to interactive 3D (OQ-8-1 deferred) | parallel_projection_renders_one_line_per_account_zero_js + core parallel tests |
| S10-FR-3 AI suggestion | erp_config flag (default OFF -> 503), Ollama via existing ai module, strict schema gate (parse_ai_suggestion), suggestion-only rendering, /drafts/adopt human path (ledger untouched) | ai_suggest_disabled_by_default_returns_503 + ai_suggestion_adopt_never_posts_to_ledger + ai_suggestion_schema_validation_rejects_malformed_output |
| S10-FR-4 builder | TransactionBuilder/TransactionItemBuilder; both clippy too_many_arguments exemptions removed (allow count = 0) | transaction_builder_constructs_domain_objects + clippy green |
| S10-FR-5 coverage | /admin/parse_coverage (ROLE_MANAGE) rule-hit ratio over drafts | parse_coverage_computes_hit_ratio |
| Methodology | BDD acceptance doc, traceability matrix, ubiquitous language glossary | docs/quality/bdd_acceptance.md, docs/testing/TRACEABILITY_MATRIX.md, docs/quality/ubiquitous_language.md |
| Baseline @ 7eb5c1d | fmt/clippy -D warnings/test --locked/audit all EXIT=0; **73 tests passed, 0 failed** | session tests/20261104-1400-slice10-final.log |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-04 addendum #6 (Slice 9: hardening close-out)

| Item | Disposition | Evidence |
|---|---|---|
| P1-9 session revocation | Deactivation now deletes all live sessions of the target account | deactivating_a_user_revokes_their_live_session |
| P1-8 password reset | POST /admin/users/reset_password (ROLE_MANAGE, CSRF) rotates hash, revokes sessions, audited PASSWORD_RESET | admin_password_reset_revokes_sessions_and_rotates_credentials |
| P3-1 report budget | 2,000-transaction ledger seeded via recursive CTEs; /reports/monthly asserted under 500 ms | monthly_report_stays_under_500ms_with_2000_transactions |
| P3-2 token sync | Regression test keeps :root token blocks of deployment and docs CSS identical | css_design_tokens_are_in_sync_between_docs_and_deployment |
| Baseline @ bc68b7e | fmt/clippy -D warnings/test --locked/audit all EXIT=0; **66 tests passed, 0 failed** | session tests/20261104-0700-slice9-final.log |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-04 addendum #5 (Slice 8: data acquisition and insight layer)

| Item | Disposition | Evidence |
|---|---|---|
| Layer A import | Two-phase CSV pipeline: preview (dry-run, SHA-256, no ledger effects) + confirm (replay-guarded 409, valid rows only, batch audit) | IMP-001..003 integration tests; curl smoke 303/409 |
| Layer B wizard | /new three-step guided posting with rule-name errors and from_draft/from_batch prefill | WIZ-001 test |
| Layer C drafts | flaps_core::parser (rule-table, evidence cards, deterministic) + /drafts/parse, human confirmation required | DRAFT-001 test; parser unit tests |
| Layer D insight | Server-rendered SVG chart (zero-JS), /reports/monthly + .csv (LEDGER_READ), print CSS | RPT-001 + ZJS-001 tests; curl smoke |
| 3D ruling | Interactive 3D conflicts with zero-JS charter; SVG+CSV-export alternative delivered; constitutional exemption deferred (OQ-8-1) | plan v2 section 12.4 |
| Baseline | fmt/clippy/test/audit all EXIT=0; 62 tests passed 0 failed | session tests/20261104-0800-slice8-final.log @ 662ff39 |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-04 addendum #4 (Slice 7: P0-2 git + P1-7 rate limit + P1-8 user admin)

| Item | Disposition | Evidence |
|---|---|---|
| P0-2 git repository | Root repo `main`: initial commit `cd66388` (ARES workspace + shared governance; 6 nested independent repos and plan dirs excluded via .gitignore; staged content scanned — no real secrets), feature commit `8a79adf`. All subsequent claims are revision-bound. | `git log --oneline` |
| P1-7 login rate limiting | `LoginRateLimiter` (5 consecutive failures -> 15 min lockout, success resets, case-insensitive, checked before credential work) | 5 tests + locked-account-with-correct-password integration test |
| P1-8 user administration | `/admin/users` list/create/toggle, ROLE_MANAGE via **live DB query** (`has_permission_db` — role assignments effective immediately), CSRF everywhere, self-deactivation rejected, created user verified login+post e2e | 4 integration tests |
| P2-2 dependency cleanup | tauri removed from workspace deps; `TAURI_IN_LOCK=0` | build/test green |
| Full baseline | fmt/clippy -D warnings/test --locked/audit all EXIT=0; **50 tests passed, 0 failed** | session tests/20261104-0500-slice7.log @ 8a79adf |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-04 addendum #3 (Slice 6: balanced double-entry + authentication)

| Item | Disposition | Evidence |
|---|---|---|
| P1-5 double-entry | Engine enforces ΣDEBIT=ΣCREDIT>0 (i128); form upgraded to debit/credit legs (same-account rejected); list amount = debit-side total | engine negative tests 8, integration same-account 400, 17 platform tests passed |
| G-1 authentication (ADR-0003) | Argon2id + server-side sessions (revocable, 24h) + synchronizer CSRF tokens + require_session middleware + one-time admin bootstrap; spoofable X-Acting-User header removed | 13 integration tests (unauth redirect, login failure genericity, forged CSRF 403, logout revocation, store-role RBAC 403, balanced e2e); curl smoke all green |
| Full baseline | fmt/clippy -D warnings/test --locked/audit all EXIT=0 | session tests log 2026-11-04 |
| Known residuals | Login rate limiting absent (P1-7), Secure cookie pending TLS (R-07), user-management UI (P1-8) | BACKLOG |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-07 addendum #11 (encoding root-cause fix + verification methodology + stress SLA)

Executed in session lineage of `.kilo/plans/1793789700000-flaps-erp-performance-report.md` (performance report line) and the three-studio plan stub `.kilo/plans/1792865422970-three-studio-governance-platform.md`. Evidence: command+exit-code bound.

| Item | Disposition | Evidence |
|---|---|---|
| CP950 encoding corruption root-cause fix | `platform/src/api.rs` restored via 3-way `git merge-file` against clean oracle `dc414ac` (corruption introduced at/after `e82a49e`); Askama pagination fields restored; `core/src/config.rs` category comment and `crates/kem/src/main.rs` `P1 M2–M3` PUA chars fixed | `cargo check --workspace` PASS; docs/encoding-standard.md; PUA scan = 0 in repaired files |
| Stress test measurement methodology | p95 degenerated to max-of-3 with cold-start sample (535ms false violation); fixed to warm-up pass + 21 samples; SLA 500ms unchanged; seed batched (500 rows/statement) cutting test wall time ~90s→~13s | warm p95 422ms; `stress_10000_transactions_p95_under_500ms` PASS |
| Workspace gates | fmt EXIT=0 / clippy -D warnings EXIT=0 / **157 passed 0 failed** / audit EXIT=0 (1 accepted RUSTSEC-2026-0374) | `cargo test --locked --workspace` 2026-11-07 log (isolated CARGO_TARGET_DIR to avoid parallel-session server file lock) |
| Verification methodology | MUTATION_SCORE_REPORT v2 (money/engine 100% after Display mutant kill); FAULT_INJECTION_SOP §10 (FAULT-02 DONE, 04/05 IN PROGRESS); ATDD_BDD_BIND_AUDIT (8 scenarios, 2 MISSING + 1 BLOCKED); FUZZING.md (cargo-fuzz build PASS, run blocked by ASan DLL — GAP-04) | docs/engineering/* |
| Docs integration | docs/README.md v3.0 index rebuilt (dead links removed, new docs indexed, superseded marked); PROJECT_STATUS/NEXT_ACTIONS/GAPS_AND_RISKS refreshed; NEXT_ACTIONS literal backtick-n corruption fixed | docs/README.md |
| Three-studio plan stub | PLAN 1792865422970 exists as TOC-only stub (§0–§24 promised, body missing; trailing "\n# Test Append" corruption) — body completion is a tracked gap (GAP-06) | .kilo/plans/1792865422970-three-studio-governance-platform.md |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-09 addendum #12 (SLICE-G1.1 + G2.1–G2.5: three-studio execution plan v4)

Executed under `.kilo/plans/1793787631643-three-studio-slice-execution-plan.md` (v4, refactoring-judgment framework included). Evidence: command+exit-code bound.

| Item | Disposition | Evidence |
|---|---|---|
| SLICE-G1.1 (S1, WP-08) | `states.html` extended to the full eight data states (empty/zero/loading/partial/stale/denied/error/conflict) with `data-state-*` anchors and next-step affordances; `design-system.css` + docs snapshot gained `.zero-state/.stale-state/.denied-state/.conflict-state` (token-only, dark-mode safe); RED→GREEN matrix test | `states_page_covers_eight_data_states` PASS; platform lib **59 passed / 0 failed** |
| SLICE-G2.1 (S2, HP-05) | eventbus: `RetryPolicy` (exponential backoff, ±20% bounded deterministic jitter), `DeliveryOutcome`, `DeadLetterStore` (record/list/signed replay with actor audit) + in-memory impl; Kafka stub untouched | platform-hub-eventbus **4 passed / 0 failed** (duplicate-consume idempotency, exhaustion→DLQ, signed replay) |
| SLICE-G2.2 (S2, HP-05) | channels: `CircuitBreakerAdapter` decorator (Closed→Open→HalfOpen→Closed; immediate reopen on failed probe), `CircuitOpen` error variant, `CircuitStats` observability; injectable clock for deterministic fault injection (FAULT-07) | platform-hub-channels **5 passed / 0 failed** (threshold→open+fast-fail; half-open probes re-close; probe failure reopens) |
| SLICE-G2.3 (S2, HP-03/04) | udm: `CustomerRecord`/`InvoiceRecord`/`PaymentRecord` each with `authoritative_system`; unit contract for UDM MoneyCents ↔ root minor (explicit, checked, refuse-truncation conversions) — merge remains forbidden per §0.5 ruling | platform-hub-udm **9 passed / 0 failed** (3 roundtrips + boundary fitness) |
| SLICE-G2.4 (S2, HP-06) | udm: `reconcile_orders` → `ReconciliationReport` (matched/missing/mismatch, deterministic sorted lists, 10-entry sample cap, i128 cent totals, JSON-serializable) | 3 tests incl. hand-computation fixture and determinism/cap checks |
| SLICE-G2.5 (S2, HP-09) | agent-policy: `AgentRateLimiter` (sliding window, caller-supplied clock), `WriteDisposition::DraftRequired` fail-closed for ALL risk levels (DEC-15 preserved), `AgentAuditMarker.audit_event()` | platform-hub-agent-policy **6 passed / 0 failed** |
| platform-hub gates | fmt 0 / clippy -D warnings 0 / **22 passed / 0 failed** / (audit via root lock) | `cargo test --locked --workspace` in platform-hub |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-09 addendum #13 (SLICE-G1.2–G1.4 + G3.1–G3.3: three-studio execution completion)

Continuation of addendum #12 under `.kilo/plans/1793787631643-three-studio-slice-execution-plan.md` (v4). Evidence: command+exit-code bound. kem in-flight refactor by the parallel line blocks whole-workspace runs; gates are run per-lane (platform/core/bridge + platform-hub), matching the plan's isolation rule.

| Item | Disposition | Evidence |
|---|---|---|
| SLICE-G1.2 (S1, WP-10) | Detail links carry `from_page`+`q` context; detail page back link restores the exact filtered view (CJK percent-encoded, hostile input truncated/degraded, never error); pagination q-preservation PINned (Askama escapes `&` as `&#38;` — assertion matches wire format) | `pagination_links_preserve_search_term`, `detail_back_link_preserves_list_context` PASS |
| SLICE-G1.3 (S1, WP-05/04/02) | Role-based landing panel on `/` (FINANCE→review queue; STORE/VIEWER→reports; ADMIN/others→overview; `data-role` anchor), role fetched live from user_roles; KPI cards carry definition/timepoint/source titles + visible `data-kpi-definitions` note | `finance_landing_leads_with_review_queue`, `store_landing_leads_with_reports`, `admin_landing_defaults_to_overview`, `kpi_definitions_are_inline` PASS |
| SLICE-G1.4 (S1, WP-07) | `POST /reviews/bulk`: single POST, per-row outcomes with rule names (SELF_ADJUDICATION_FORBIDDEN / NOT_PENDING / NOT_FOUND / STATE_CHANGED), per-row REVIEW_BULK audit events with actor signature, failed-row highlight (`tr.outcome-failed`), replay-safe by design (re-submission fails per-row, never double-posts), 100-row cap, forged-CSRF and RBAC gates | `bulk_review_reports_mixed_outcomes_and_audits_each_row`, `bulk_review_reject_marks_rejected`, `bulk_review_requires_ledger_write`, `bulk_review_rejects_forged_csrf` PASS |
| SLICE-G3.1 (S3, GP-04) | `GET /admin/audit_evidence.csv` (ROLE_MANAGE): four audit-readiness requirements as header block — 控制相關 (control library ids), 涵蓋期間 (validated from..to, fail-closed), 完整性 (SHA-256 row digest at export time), 來源 (platform audit_log); RFC 4180 escaping | `evidence_export_includes_four_requirements_and_rows`, `evidence_export_requires_role_manage` PASS |
| SLICE-G3.2 (S3, GP-02) | `platform/src/controls.rs`: one control library (CTRL-POST/IMPORT/REVIEW/CRED/AI) each mapping audit event types × framework obligations (ISO-27001 annex / INTERNAL-IC / BANK-FSC); `lint_control_coverage` reuses kem gov.rs Finding shape (UNMAPPED_EVENT / INACTIVE_CONTROL); `GET /admin/control_coverage.json` | `platform/src/controls.rs` 4 unit tests + `control_coverage_maps_observed_and_flags_unmapped` PASS |
| SLICE-G3.3 (S3, GP-05) | `GET /admin/ccm.json`: three indicators with definition/timepoint/source — M1 金額不變量違反 (unbalanced transactions), M2 登入異常 (honest null + gap note: login auditing not yet implemented — a fake zero would fake a green control), M3 簽名完整性違反 (unsigned audit rows) | `ccm_indicators_detect_invariant_and_signature_violations` PASS |
| PRE-R1a | Delivered by the parallel line (not duplicated here): `platform/tests/characterization.rs` | 6 tests PASS in combined run |
| Gates (this lane) | fmt 0 / clippy -D warnings 0 / **platform 77 passed 0 failed** (lib) / combined lanes **141 passed 0 failed** (bridge 7 + core 42 + platform 77 + characterization 6 + close_out 4 + slice12 2 + misc 3) / audit EXIT=0 (1 accepted warning) | 2026-11-09 logs |
| G3.4 | Spec-only per plan (OQ-GOV-04 credential model unresolved) | plan §2 PHASE-G3 |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-09 addendum #14 (SLICE-E2E: BankPay 整合收斂 + 稽核 NULL 安全 + 設定持久化)

Executed by the review/verification lane on the shared working tree (S1/S2/S3 streams were concurrently active on the same files; converged artifacts noted). Evidence: command+exit-code bound.

| Item | Disposition | Evidence |
|---|---|---|
| BankPay 模組建置修復 | 37 編譯錯誤→0（缺失回應型別匯入、請求結構補 `Serialize`、`const` 中 `vec!` 改 `match` 靜態切片、`StatusEngine: Clone`、`has_permission_db` 回傳 `bool` 誤用 `.unwrap_or(false)` 移除、`permissions` 欄位可見性） | 該輪 `cargo check -p platform --features bankpay` EXIT=0（14:0x）；其後架構收斂移除 feature 閘控（`time`→`chrono`，零新增依賴），修復內容併入無條件編譯模組 |
| 設定持久化（erp_config SSOT） | `bankpay::load_config`/`save_config`（決定性 `config_id`=`bankpay:{key}`、冪等 UPSERT）；POST 先持久化後記憶體（fail-closed）；啟動時載入 | 單元測試 3 項 PASS（roundtrip／UPSERT 不重複列／新 AppState 重啟載入）；runtime smoke：POST→GET 生效（14:27，erp_config 5 列） |
| 稽核 NULL 安全 | 三處查詢改 `COALESCE(al.transaction_id,'')`／`COALESCE(al.event_data,'')`（NULL 列不再毒化整批查詢） | platform/src/api.rs L3731；cross_module_api.rs audit_list／audit_export |
| 分頁語義確認 | 儀表板頁尺寸=`LIST_PAGE_SIZE`(20)；`?page=999` 以 `clamp_page` 夾限至末頁；非法 `page` 回第 1 頁（fail-closed） | runtime smoke：`/transactions?page=999`→`第 1 / 1 頁`（6 筆）；E2E 測試斷言 21 筆→`第 2 / 2 頁` 已就位待綠樹重跑 |
| E2E 測試交付 | `platform/tests/bankpay_persistence.rs`：21 筆分頁＋超頁夾限＋config 持久化（適配 `X-CSRF-Token`）＋session 跨「重啟」（DB-backed） | **PASS**：`cargo test -p platform --test bankpay_persistence` 1 passed / 0 failed（1.16s）；同輪全平台 97+16 測試 0 失敗；`cargo fmt -p platform -- --check` EXIT=0（15:44） |
| clippy 閘門觀察 | `cargo clippy -p platform --all-targets -- -D warnings` 共 12 警告，全部位於並行流收斂中之新碼：10× `result_large_err`（platform/src/bankpay/api.rs:50/167/222/275/426/488/562/592/674/700）、2× `div_ceil`（cross_module_api.rs:178、bankpay/api.rs:701） | 15:45 clippy 全目標輸出。建議修法（不新增 allow）：錯誤以小結構承載（如 `ApiError{status,message}` + `impl IntoResponse`）取代 `Result<_, Response>`；除法改 `u64::div_ceil`／`u32::div_ceil`。由該串流修復後重跑為綠 |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-09 addendum #15 (SLICE-E2E收斂後修復: BankPay 交付完整性 + 稽核/CSRF/冪等 + 狀態誠實化)

Executed on the shared working tree after addendum #14. Scope: `platform/src/bankpay/*`, `platform/src/status.rs`, `platform/src/auth.rs`, `platform/src/api.rs`, `platform/src/cross_module_api.rs`, `platform/migrations/000{6,7}_*.sql`, `platform/templates/index.html`, `web/assets/design-system.css` (+docs snapshot), `platform/tests/bankpay_api.rs`. Parallel-stream files (`crates/kem/*`, `platform-hub/*`, `platform/src/apihub.rs`, `knowledge/*`) untouched. Decision rationale: `docs/adr/ADR-0012-bankpay-integration-boundary.md`; plan-level disposition table: `.kilo/plans/1793782615513-esun-bankpay-ui-optimization.md` 附錄 A.

| Item | Disposition | Evidence |
|---|---|---|
| Feature gate 移除（交付缺口關閉） | `bankpay`/`status` 改無條件編譯+註冊；`time`/`reqwest/multipart` 不再需要 → 依賴零新增 | `cargo test --locked -p platform`（預設 features）124 passed / 0 failed |
| 稽核寫入修復 | 6 寫入點中 5 個 bind 數錯誤（4 binds vs 3 placeholders，執行期必敗且被 `let _ =` 吞掉）→ 統一 `write_audit`；fail-closed（稽核失敗→500，配合冪等重放安全） | 整合測試 `bankpay_export_is_idempotent_and_audited`（audit 列存在、actor 簽署） |
| 冪等批次帳 | 遷移 0006：`bankpay_batches`（`batch_id` PK、`esun_ref` UNIQUE、金額 CHECK、狀態 CHECK）+ `bankpay_audits`；匯出 `ON CONFLICT DO NOTHING` → 重放回傳既有狀態不重複呼叫銀行 | 遷移 0006/0007；整合測試重放回應逐位元組一致 |
| CSRF（T-08） | 變更類 POST 驗 `X-CSRF-Token`；拒絕時不留批次/稽核痕跡 | 整合測試 `bankpay_export_requires_csrf_token`；live smoke：缺漏/錯誤 token 皆 403 且 DB 無新增批次 |
| API 401 + 未認證漏洞關閉 | `auth::require_session_api`（401 JSON）；`cross_module_api` 原於 `route_layer` 後 merge → 稽核/CSV 匯出/COM/AI 無需登入 → 改先 merge 後掛層 | 整合測試 `bankpay_api_requires_session`（含 `/api/audit` 401） |
| 路徑白名單（CWE-22） | 匯入僅接受 `data/bankpay/inbound/*.csv`（canonicalize 前綴比對） | 整合測試 `bankpay_import_rejects_path_traversal`；live smoke 400 |
| 模擬誠實化（P7） | 捏造端點/欄位移除；mock 匯入真實解析 CSV；查詢金額由本地批次帳補入；生產模式 fail-closed 500（權威實作指向 `Wellfit/crates/wellfit_bankpay`） | `client.rs` 單元測試；live smoke `matched:2/mismatched:1` |
| 狀態誠實化 | `/api/status` 即時探測（移除 Unknown 快取→`degraded` 恆真 bug）；sync/com/ai 標 Idle 並說明；bankpay 模擬模式標 Warning；儀表板 AI 模型數 3→0 | 整合測試 `status_endpoint_shape_and_mock_warning`；live smoke JSON |
| UI（T8/T10/T11/T13） | 導航移除 `/states`（路由保留）；sticky 狀態列 + `:target` 詳情面板（Design Token，`web/assets/design-system.css` + docs 快照同步）；BankPay 卡片；稽核篩選死控制改真實 GET 表單 | 整合測試 `index_renders_bankpay_card_and_status_bar`；live smoke 渲染斷言 |
| clippy/fmt | `ApiError(StatusCode,String)` 小型錯誤型別（`result_large_err` 合規，禁用 allow）；`div_ceil`；`cargo clippy -p platform --all-targets -- -D warnings` = 0；`cargo fmt -p platform -- --check` = 通過 | clippy/fmt 輸出 |
| Live smoke（:8091, 獨立 smoke DB, 已清理） | 401（未登入）/303（PRG 登入）/匯出 200→冪等重放一致→查詢推進 COMPLETED＋本地帳金額/白名單外 400/`/api/audit` 需登入 | 本 addendum 執行記錄 |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-09 addendum #16 (security-driven sqlx 0.9 upgrade + RUSTSEC-2026-0459 disposition; renumbered from #14 — numbering collision with the parallel BankPay line)

Trigger: `cargo audit` began reporting RUSTSEC-2026-0459 (CVE-2026-119375, severity 6.8 medium) after the advisory DB refresh — libsqlite3-sys 0.30.1 bundles SQLite < 3.54.0 (uninitialized page-cache memory disclosure via crafted .db + mismatched-page-size -wal pair; LOCAL vector). Evidence: audit EXIT=1 log 2026-11-09; advisory https://rustsec.org/advisories/RUSTSEC-2026-0459.

| Item | Disposition | Evidence |
|---|---|---|
| sqlx 0.8.6 → 0.9.0 | Upgraded across the workspace (root workspace deps, core/Cargo.toml, platform/Cargo.toml). First attempt left dual sqlx versions in the graph (core pinned 0.8 → E0308 two-Pool types + fallout FromRow/dynamic-SQL-audit errors); root-caused via `cargo tree -i sqlx` and fixed by unifying all three manifests to 0.9 | `cargo check --workspace` EXIT=0 after unification |
| libsqlite3-sys 0.39.0 | NOT reachable: sqlx-sqlite 0.9.0 (latest crates.io release) pins `>=0.30.1, <0.38.0` | `cargo update -p libsqlite3-sys --precise 0.39.0` failure log |
| Acceptance disposition | Documented in `.cargo/audit.toml` per GOVERNANCE P2/P7: local-vector-only exposure; app opens only operator-created files under data\; single-user desktop deployment; daily backup bounds exposure. Revisit triggers: sqlx widens the range, or the app ever opens third-party DB files | `cargo audit` EXIT=0 (2 documented dispositions) |
| Four gates after upgrade | fmt EXIT=0 / clippy -D warnings EXIT=0 / **235 passed 0 failed** whole workspace (kem line recovered by parallel session: 58 lib tests green) / audit EXIT=0 | 2026-11-09 logs |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-09 addendum #17 (PHASE-R: S1 檢索面 RAG 升級 — BM25＋混合向量; renumbered from #15 — numbering collision)

Trigger: owner directive「必須用到所有先進相關技術（RAG）」. Tree-state audit found the gap precisely: S3 (kem) already ships full hybrid retrieval (BM25 + embeddings + fusion, 58 lib tests), but S1's `/api/search` was substring-only with constant `score: 1.0` (`platform/src/search.rs:48-56`, `cross_module_api.rs:265-280`). Executed per PHASE-R of `.kilo/plans/1793787631643-three-studio-slice-execution-plan.md`.

| Item | Disposition | Evidence |
|---|---|---|
| RAG-S1 詞法腿 | New `platform/src/bm25.rs` (std-only, zero new deps — Tantivy lesson): CJK-aware tokenizer (single-char CJK tokens — improvement over kem's `w.len()>1` filter which drops them), `BM25Index` with Robertson IDF identical to kem (kem/search.rs:128; the +1.0 inside ln keeps IDF positive — a 0-floor variant was tried and rejected: it zeroed ubiquitous terms) | 5 unit tests PASS (rare-vs-common IDF, tf ordering, unknown→empty, mixed CJK tokenize, empty-safety) |
| RAG-S2 混合引擎 | `platform/Cargo.toml` += `kem = { path = "../crates/kem" }` (zero new external crates — kem deps ⊆ platform deps). `SearchEngine` gains BM25 index (rebuilt on mutation, sorted doc_id positions) + freshness-stamped vectors (`sha256(content)` rule, kem §8) + `search_ranked()` with α-normalized fusion (α=0.5, refusal floor 0.55 mirroring kem calibration) and `via` provenance ("hybrid"/"bm25"); embedder reuse via `kem::embed::embed_batch` + `default_pool` (OVH bge-m3 → Ollama local); unreachable embedder → deterministic BM25-only degradation | 6 unit tests PASS (degradation marks via, ordering, pure-fusion blend at α=0.3, refusal floor, stale-vector-missing, delete rebuild) |
| RAG-S3 端點接線 | `/api/search` returns real fused scores + `via` (additive field); direct fetch marked `via="direct"` (never a fake fused score) | compiles; degradation path deterministic in CI (no embedder) |
| Cross-line stale fixture | `platform/tests/bankpay_persistence.rs:150` expected 5 persisted bankpay keys while the parallel line's EsunConfig refactor grew it to 10 (mTLS cert fields) — one-line stale-fixture fix with rationale comment | full workspace suite green after fix |
| Four gates | fmt 0 / clippy -D warnings 0 / **246 passed 0 failed** whole workspace (bridge 7 + core 42 + kem 58 + platform 108 + kem-int 15 + bankpay_persistence 1 + characterization 6 + close_out 4 + slice12 2 + misc 3) / audit EXIT=0 both workspaces (platform-hub gained its own `.cargo/audit.toml` with the mirrored RUSTSEC-2026-0459 disposition — hub lock resolves libsqlite3-sys 0.37.0, same range block) | 2026-11-09 logs |
| Out of scope (explicit) | LLM generation (kem `ask` discipline: references only), Tantivy/Qdrant (POC separate), index persistence, kem search decoupling refactor | plan §R.4 |
| 效能評估進度報告 v2.0 | `docs/research/FlapsErp效能評估進度報告_v2.0.md` — 全程進度版（11-03→11-09 時間軸、246 測試、SLA 422ms、方法論進化表、開放項目誠實清單），登錄 docs/README.md 索引 | 本行 |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-09 addendum #18 (雙作品整合：治理四頁＋檢索頁＋桌面殼 D1/D8 + 受眾分層 IA)

Executed by the plan/code lane per `.kilo/plans/1793034142477-apihub-tauri-uiux-integration-plan.md`（D1/D3/D5/D7/D8 全數拍板）。Concurrent-stream convergence on `pages.rs`／`apihub.rs` is owned by that stream（見「進行中」列）。

| Item | Disposition | Evidence |
|---|---|---|
| 受眾分層 IA（使用者裁決） | 操作者／管理者／技術人員三層；主導覽 `can_manage_users` 閘控管理與技術入口（並行流已重構為下拉分組）；操作者頁尾不帶開發文件連結 | `platform/templates/index.html` nav（20:59 版：管理下拉含 API 中樞/日誌/設定） |
| E.Sun V2.8 契約目錄 | 8 契約端點入 catalog（`esun_endpoints()`：path/name/contract_ref §47–§299/平台對應/誠實標示「模擬對應3·未串接5」）；KPI 卡契約 8｜原生 12 | `apihub.rs::esun_endpoints()`＋`api_catalog.html`（含 KPI 定義 tooltip） |
| 治理四頁（T2） | `/admin/api/records`（源 bankpay_audits＋bankpay_batches，HTTP 狀態實錄）、`/admin/api/codes`（僅列已驗證 910000＋rsStatus 語義）、`/admin/api/release`（RC-01..06 誠實狀態：3 已驗證/2 待憑證/1 未串接）、`/admin/api/net`（本地設定與程式面檢查；不做網路探測） | `apihub.rs` 處理器＋4 模板＋api.rs 路由；catalog 頁首入口連結已加 |
| 檢索頁（T3） | `/search?q=`（Tantivy；空索引誠實回空態；RAG 升級後呈現融合結果） | `workbench.rs::search_page`＋`search.html`＋index 導覽「檢索」連結 |
| 重複路由修復 | `/bankpay/exceptions`、`/import/history`、`/help` 各註冊兩次（axum 啟動 panic）→ 移除第二份 | api.rs 編輯（21:07） |
| 桌面殼 D1/D8（T5/T6） | `tauri.conf.json`：devUrl:1420 移除、`url=http://127.0.0.1:8090/`、CSP 去 `script 'unsafe-inline'`、`bundle.active=true`+NSIS、v0.2.0；`main.rs`：24 個 mock commands 全數移除（P7）、冪等啟動（埠探測→spawn→6s 就緒等待）；`state.rs`：金額 f64→i64 minor（4 處，INV-01） | `web/src-tauri/*`（10/30 以來首次收斂；NSIS 產物建置列入下一切片） |
| ADR-0013（T7） | 跨平台策略：桌面=Tauri 殼、Web=響應式零 JS、行動面本輪不承諾（誠實評估）；Electron 不採用（憲政衝突） | `docs/adr/ADR-0013-cross-platform-strategy.md` |
| 進行中（他串流擁有） | `pages.rs`（14 頁：audit/trial-balance/ledger/batches/detail/exceptions/import-history/help/status/profile/roles/sessions/system）與 `apihub.rs` 模板結構重構收斂中——21:21 時 21 errors（E0609/E0308/E0599）為其過渡態；`/audit` 建議補 ROLE_MANAGE 閘（IA Tier 2） | cargo check 21:21 輸出；pages.rs mtime 21:06+ |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-10 addendum #19 (P0 安全收口 + P1 桌面 v0.2 可分發：NSIS 實證)

Executed per `.kilo/plans/1793034142477-*` §9（P0/P1 執行切片）。並行流協作：security headers 由其先落地（Slice 11），pages.rs 收斂由其完成；本線完成 P0-1a/P0-2 與全部 P1。

| Item | Disposition | Evidence |
|---|---|---|
| P0-1 安全標頭 | 已在位且嚴於計畫：`auth::security_headers` 掛全回應——`default-src 'none'; style-src 'self'; img-src 'self' data:; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'`＋`nosniff`＋`X-Frame-Options: DENY`＋`no-referrer`＋`no-store` | api.rs router 尾 `.layer(map_response(...))`；smoke 實收 CSP 標頭 |
| P0-1a 內聯歸零 | `index.html` 2 處內聯樣式（稽核篩選 form、死元素空 div）→ 新類別 `.audit-filter-form` 入 `web/assets`＋`docs/design-system.css`（兩檔同步 byte-equal 實測）；死 div 移除（零 JS 下無引用） | `grep 'style="' platform/templates` = 0 |
| P0-2 稽核 RBAC | `/audit` 補 `ROLE_MANAGE` 閘＋`denied_page` 共用輔助（Denied 八態語彙） | `page_matrix` 斷言 store1 `/audit` → 403 PASS |
| P0-3 pages.rs 收斂 | 並行流完成（09:19）；本線 10:0x 複驗 0 錯、145 測試綠 | `cargo check`/`cargo test` |
| P0-4 catalog KPI | 已動態（`esun_total`/`native_total`）；`grep '<dd>[0-9]'` 零命中 | 模板複驗 |
| T4 角色矩陣 | 新 `platform/tests/page_matrix.rs`：訪客 303；admin 10 頁 200＋零 JS；store1 操作者頁 200、治理頁（audit/catalog/users）403 | 1 passed（2.35s） |
| P1-1 圖示＋打包 | `bundle.icon` 接 `icons/icon.ico`；**NSIS 實證 externalBin 單獨不入包**（1.9MiB 安裝缺 sidecar）→ 追加 `resources` 映射後 3.82MiB 入包（安裝目錄 3 檔齊）；`resolve_server_exe` 絕對路徑序（FLAPS_SERVER_EXE→exe 目錄→binaries/）——**smoke 抓出 PATH 相對路徑誤抓 dev 二進位之真 bug** | 安裝清單＋`target/release/flaps-server.exe` 拷貝鏈 |
| P1-2 無白屏啟動 | `label:main`＋`visible:false`；setup 就緒後 `window.show()`（失敗路徑亦 show 防困死）；**smoke 抓出 CWD 相對 DB 路徑 bug**——舊版 DB 遇（BOM 正規化後）0005 checksum 不符之 `VersionMismatch(5)` fail-closed 崩潰 → spawn 強制 `current_dir(exe_dir)`＋預建 `data/` | 兩輪 smoke 前後對照：崩潰（stderr 實錄）→ login 200 |
| P1-3 守衛擴充 | `SCAN_DIRS` 5→7（＋`web/src-tauri/src`、＋`crates/kem/src`）；guards 2/2 PASS（新目錄零損傷） | guards test |
| P1-4 NSIS 實證 | `Flaps ERP_0.2.0_x64-setup.exe`（3.82MiB）；/S 安裝→shell（安裝目錄）→spawn 安裝目錄 flaps-server（路徑實錄）→**login 200＋CSP＋零 JS**；teardown 0 殘留；/S 反安裝 exit 0（`data/flaps.db` 依設計保留使用者資料） | 本節全部命令輸出 |
| 過渡態事件 | release 建置窗口撞上並行流瞬時重複路由 `GET /tauri-preview`（axum 執行期 panic；編譯不擋）——其 11:54 修復後本線重建清除 | stderr 實錄＋重建後 smoke 綠 |
| 四關封印 | `fmt` EXIT=0；**145 測試 0 失敗**（112 lib＋16＋1＋6＋4＋2＋3＋1）；clippy 本線範圍零新增 | 2026-11-10 日誌 |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-10 addendum #20 (全能進料門戶 V1：XLSX 匯入轉接器——Grill-me 收斂後的首個實作)

Grill-me 兩問收斂：第一位使用者＝會計/財務承辦；殺手級應用＝消滅重複鍵入；進料＝全格式（Excel/CSV＋Email PDF＋紙本＋系統匯出）。產品答案：**單一進料門戶＋多格式轉接器管線**（開放封閉：新格式＝新 adapter，主幹不動）。免費技術實證：calamine 0.38.0（crates-io：17.8M 下載、2026-11-02 更新、純 Rust）。

| Item | Disposition | Evidence |
|---|---|---|
| 欄位驗證器萃取 | `validate_import_row` 重構為薄殼＋新增 `validate_fields`（6 欄格式中立契約）——CSV 與 XLSX 共用同一驗證 | api.rs L3103 |
| 格式偵測 | `detect_import_format`（magic bytes：PK\x03\x04→XLSX，其餘 CSV）；`parse_xlsx_rows`（calamine 第一張工作表→(row_no, cells)） | api.rs L3178/L3189 |
| preview 分流 | `push_import_row`（列數上限 fail-closed＋欄位不足明確訊息）；CSV 路徑零改動、XLSX 路徑跳過標題列＋row_no+1 對齊 Excel 顯示 | api.rs import_preview |
| calamine dates | `features=["dates"]`（as_datetime 被閘；**chrono 為既有依賴，零新增外部 crate**） | platform/Cargo.toml |
| Excel 浮點正規化 | Data::Float 整數值去小數、其餘最短表示→既有 `parse_decimal_to_minor` fail-closed | 測試實錄：1500.5→150050 minor、99.99→9999 minor |
| 端到端驗收 | 新 `platform/tests/import_xlsx.rs`＋`fixtures/sample-import.xlsx`（PowerShell .NET ZipArchive 自建最小 OOXML：inlineStr 中文＋浮點金額＋幣別錯誤列）：multipart 上傳→PRG→批次 draft→3 列（1 錯 2 合格）→錯誤訊息含「幣別不支援：XXX」→報表渲染→零 JS | 1 passed（1.30s） |
| 模板 | `import.html` accept 加 `.xlsx`＋提示（支援格式/日期文字格式 V1 限制） | import.html L36-42 |
| V2/V2.5 路線 | PDF 文字層（pdf-extract 0.12.1，crates-io 實證）＝V2 首項；OCR（Tesseract）依紙本佔比評估＝V2.5；Email 附件＝V3 | 本 Brief 分期表 |
| 過渡態聲明 | clippy 複跑時並行流帳戶分頁重構進行中（api.rs:1655/pages.rs:426 兩錯，15:29-15:31 mtime）——**他串流擁有收斂權**；本線 XLSX 測試於其前已 PASS（revision-bound） | 15:33 cargo 輸出 |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-10 addendum #21 (全能進料門戶 V2：PDF 文字層轉接器 + NavModel 平台重構收斂封印)

V2 交付（另一串流同期完成 NavModel 統一導覽大重構——本線觀察其收斂並封印 fmt）。

| Item | Disposition | Evidence |
|---|---|---|
| PDF 轉接器 | `detect_import_format` 加 `%PDF` magic；`parse_pdf_rows`（pdf-extract 0.12.1 記憶體擷取，crates-io 實證：7.9M 下載、2026-09-16 更新）；分隔啟發鏈 TAB → 「2+ 連續空白」→「\|」；掃描式 PDF → 明確錯誤訊息（OCR 屬 V2.5） | api.rs（enum/detect/parse_pdf_rows/preview Pdf 分支） |
| 重大教訓（實證） | 首版 fixture 以 PDF 字串逃逸 `\011`（TAB）分隔——擷取器未還原逃逸 → 測試紅（right:1）；改**管線分隔**（普通字元必存）→ 綠。真實世界 PDF 文字層的分隔符不確定性，正是啟發鏈存在的理由 | import_pdf 首次紅→修 fixture 後 PASS（1.10s） |
| 測試 | `import_pdf.rs`：multipart→PRG→draft→2 列（1 合格 1 幣別錯誤）→150050 minor→報表渲染→零 JS；fixture＝PowerShell 手組最小 PDF（正確 xref、795B） | 1 passed |
| NavModel 重構（他串流） | `crate::nav::NavModel`＋Extension 注入席捲全模板（含本線 apihub/workbench 之結構——其完成掃描）；本線觀察 33→0 收斂全程並於其靜置後以 rustfmt 封印餘留縮排（api.rs/apihub.rs/nav.rs，語義零改動） | cargo check 逐輪輸出；rustfmt EXIT=0 |
| 四關封印 | fmt 0／clippy 零警告（all-targets）／**152 測試 0 失敗**（117 lib＋16＋6＋4＋3＋2＋1×4；含 import_xlsx、import_pdf、page_matrix、guards）／本線範圍零新增警告 | 2026-11-10 17:5x 日誌 |
| 門戶矩陣現況 | CSV✓（既有）｜XLSX✓（V1）｜PDF 文字層✓（V2）｜OCR＝V2.5｜Email 附件＝V3｜系統模板映射＝V3 | 分期表 |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## Consolidation rules

1. Keep one current navigation register and one acceptance/evidence ledger per project. Detailed requirements belong in their project documents.
2. Keep proposals only when they have a named owner, explicit status, dependencies/blockers, measurable acceptance criteria, and dated evidence. Otherwise mark them as historical, draft, or blocked.
3. Use in-place supersession notices and links; do not delete or bulk-move plans until unique requirements and inbound links are reconciled.
4. Treat an ADR as accepted only when its decision owner, rationale, date, and implementation evidence can be identified. An index entry alone is not approval.
5. Test results are revision-bound. A prior green run does not establish that a dirty or newer checkout passes.
6. External-source failures must remain explicit `UNKNOWN`; never convert HTTP errors or inaccessible sources into verified claims.

## Ordered follow-up

1. Correct SyncMesh's TLS/mutual-authentication and release-completion claims against the actual code and `NOT RUN` acceptance ledger; no deployment/security claim should rely on the current prose.
2. Resolve the ARES workspace boundary and reconcile its target Cargo design before any implementation roadmap is executed.
3. Reconcile EKPI status summaries and legacy flat documentation against `docs/README.md`, `docs/quality/phase-gates.md`, and revision-bound assurance records.
4. Ratify the Wellfit ISMS document hierarchy and scope; independently verify each standards edition and applicability from a primary source before using it as compliance evidence.
5. Preserve every unresolved owner, bank, or data-governance decision as blocked; do not silently select a vendor, certification scope, cloud provider, or release target.
