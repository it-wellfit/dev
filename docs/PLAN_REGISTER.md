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
## 2026-11-10 addendum #22 (全能進料門戶 V2.5/V3 研究與設計定稿：OCR、Email 附件、系統模板映射)

依使用者指令將 V2.5／V3 由子彈點補完為**證據級研究＋設計定稿**：`docs/research/全能進料門戶_V2.5_V3_研究與設計.md`（v1.0）。

| 決策 | 選擇 | 關鍵證據 |
|---|---|---|
| D-OCR1 OCR 引擎 | **Tesseract 外部執行檔＋偵測式降級**（首發）；xberg-tesseract 綁定線（2026-10-13 更新）為升級路徑；ocrs（純 Rust、2026-11-08）因**CJK 腳本覆蓋 UNKNOWN** 列 POC | crates-io：xberg 1.3.10/23K dl、ocrs 0.14.0/557K dl；traineddata chi_tra Apache-2.0 |
| D-OCR2 掃描式偵測 | 零新 dep 啟發（有效字元 < 20/頁 → 引導訊息）；pdf-inspector（1.39M dl、純 Rust 分類器）列強化候選 | pdf-inspector README：自動分類 text/scanned |
| D-OCR3 照片直送 | `--psm 6 tsv` 置信度解析；**行 avg < 60 → error 列「低置信度請人工核對」**（fail-closed） | 設計 §3.2 |
| D-EML1 Email | `mail-parser 1.0.1`（5.4M dl、2026-11-02）＋附件**遞迴進料**（每附件一 batch、sha256 逐檔）；.msg 次項（msg_parser） | 設計 §3.4 |
| D-MAP1 映射 | 簽名自動匹配（同義詞表預填）＋單次映射頁（零 JS 表單）；profile 存 erp_config（升級路徑遷專表） | 設計 §3.3 |
| 驗收 | **雙態測試策略**：無 tesseract 環境斷言降級訊息；有引擎環境跑合成 PNG 全鏈；EML/映射各備 fixture 與斷言 | 設計 §6 |
| 分期 | V2.5-a 探測骨架 → V2.5-b TSV 引擎 → V3-B 映射（純邏輯）→ V3-A mail-parser → POC（ocrs CJK／pdf-inspector 誤判率） | 設計 §7 |
| 未決（採納前必驗） | ocrs CJK 覆蓋實測；Tesseract Windows 捆綁 vs 自裝；profile 遷表觸發 | 設計 §8（UNKNOWN≠PASS） |
This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-11 addendum #29 (供應鏈狀態審查：桌面套件納入閘門、跨 repo 修正、實測數字更正——addendum #26 之補充)

審查對象：2026-11-10 之供應鏈硬化與 W10 交付狀態。所有數字附實測方法與日期（GOVERNANCE P7；UNKNOWN ≠ PASS；Consolidation rules #5 測試結果 revision-bound）。

| Item | Disposition | Evidence |
|---|---|---|
| 桌面套件供應鏈閘（原缺） | **完成（PASS）**：`web/src-tauri` 為**獨立 workspace**，根 `Cargo.lock`／`cargo audit`／`cargo deny`／SBOM 皆不涵蓋實際交付之安裝包依賴圖（509 個 crate）——即「交付物未受閘門管制」。現補 fmt／clippy／test／audit／deny／SBOM 六閘 | 2026-11-11 實測：`cargo fmt --manifest-path web/src-tauri/Cargo.toml -- --check` EXIT=0；`cargo clippy --manifest-path web/src-tauri/Cargo.toml --all-targets --locked -- -D warnings` EXIT=0；`cargo test --manifest-path web/src-tauri/Cargo.toml --locked` → 2 passed；`cargo audit --file web/src-tauri/Cargo.lock` EXIT=0；`web/src-tauri` 內 `cargo deny check` → advisories／bans／licenses／sources 全 ok |
| 桌面套件授權與 path 依賴 | **完成**：`flaps-erp-web` 補 `license = "LicenseRef-Proprietary"` ＋ `publish = false`；`wellfit_bankpay`／`wellfit_core` path 依賴補 `version = "0.1.0"`；**跨 repo 修正** `Wellfit/Cargo.toml` 工作區內部 path 依賴補 version（消除 wildcard） | cargo-deny 0.20.2 原始錯誤：unlicensed `flaps-erp-web`、MPL-2.0 五 crate 未允許（cssparser／cssparser-macros／dtoa-short／option-ext／selectors）、`wildcard for crate 'wellfit_bankpay'`、proc-macro-error unmaintained、glib unsound——五項皆已收斂 |
| 公告處置擴充 | **完成**：新增 **RUSTSEC-2024-0370**（proc-macro-error 未維護）與 **RUSTSEC-2024-0429**（glib unsound）書面處置於 `.cargo/audit.toml`（理由／緩解／複核觸發條件），`deny.toml` 同步忽略 | 兩者僅經 tauri → gtk（Linux 後端）進入 lockfile，**不編入本產品唯一之 Windows 交付物**；複核觸發＝新增 Linux 目標或上游釋出修補版 |
| deny.toml 圖範圍與陷阱 | **改為 `all-features = true`**（兩 workspace 實測全 ok）；記錄 cargo-deny 0.20.2 操作陷阱：`cargo deny --config <k>=<v> check` 會以 CLI 覆寫**取代**本檔 ignore／allow 策略 | 實測對照：`--config graph.all-features=true` → 約 300 筆授權錯誤＋3 筆公告錯誤；改寫檔案後 → advisories／bans／licenses／sources 全 ok。變更圖範圍一律直接改 `deny.toml` |
| CI 修正（原必敗） | **完成**：桌面 SBOM 步驟原為無條件執行，乾淨 checkout 缺 `Wellfit/`（獨立 repo、被根 `.gitignore` 排除）→ CI 必然失敗。現加入 it-wellfit/wellfit 之 `actions/checkout`（同 SHA 釘版、`ref` 釘 `b5cf18e1dd43675f5a4fe0fa8dd396a3ef028fd7`）＋桌面六閘步驟（fmt／clippy／test 限 Windows 目標） | 2026-11-11 本機等價流程全綠；YAML 以 PyYAML 解析驗證（16 steps，隔離 venv，非 repo 工件） |
| 實測數字更正（誠實） | **2026-11-10 的「五閘全綠／test 294 passed」為歷史量測值**。2026-11-11 複驗時根 workspace 因平行編輯中的 `platform/src/api.rs`（`break` in async fn ×3、format 字串、型別不符等 6 錯）暫時無法編譯，`cargo test --locked` 未完成。依既有實測分項合計為 **293**；未重測前不得宣稱 294 為現行值 | `cargo build -p platform --message-format=short` 輸出（2026-11-11 10:1x）；重測待該變更落地後執行，屆時校正文件數字 |
| 未竟（誠實 BLOCKED/OPEN，本輪新增） | (5) ~~**跨 repo 提交待辦**~~ **已完成（2026-11-11）**：`Wellfit/Cargo.toml` 之 version 修正已提交並推送至 it-wellfit/wellfit（commit `b5cf18e`），Dev CI 的 Wellfit `ref` 已同步更新。(6) 桌面 SBOM 已重新產出（含 SPDX 運算式）；根 6 份 SBOM 須待平行編輯收斂、lockfile 穩定後重新產出。(7) signed-artifact 驗證未納入 CI（需正式金鑰）；PR 閘建議以**測試金鑰**做 sign→feed→verify round-trip。(8) `cargo vet`／`cargo auditable` 仍未安裝 → 該層 UNKNOWN。(9) `platform/src/com/*` 6 處 COM `unsafe` 與 forbid 憲章衝突未解 | — |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-11 addendum #27 (全能進料門戶 V3-A：Email 附件遞迴進料——實作與驗證)

依 addendum #22 設計定稿（D-EML1）實作 V3-A；同時完成 V2.5 OCR 分支的 eml 內遞迴。

| Item | Disposition | Evidence |
|---|---|---|
| 依賴 | `mail-parser = "1.0"`（stalwartlabs；5.4M 下載、2026-11-02、純 Rust、MSRV 1.98 < 本機 1.99.1；MIRI/fuzz 驗證——官方 README 引述） | platform/Cargo.toml |
| EML 偵測 | 無 magic bytes → 啟發式：前 4KB 內 ≥2 個標準標頭行（received/from/to/subject/message-id/mime-version）；`looks_like_eml` | api.rs |
| 附件擷取 | `parse_eml_attachments`：`MessageParser::default().parse` → `attachments()` 迭代 → `attachment_name()`＋`decoded()`（**實證：1.0.1 為 decoded() 非 contents()**——首版編譯錯誤修正）；上限 10 附件（D-EML1）、單件 2MB 沿用 | api.rs |
| 遞迴進料 | 每附件：detect → 對應轉接器（CSV/XLSX/PDF/Image＋OCR）→ 映射正規化 → 驗證 → **每附件一 batch**（sha256 逐檔）→ PRG 至第一批次＋notice 帶全部批次摘要（percent-encode）；巢狀 eml → 明示錯誤列（首發限制）；無附件 → 明確訊息批次 | api.rs Eml 分支 |
| 測試 | `import_eml.rs`＋`fixtures/sample-import.eml`（手構 MIME：UTF-8 主題編碼＋CSV 附件）：上傳→PRG→批次 2 列（150050/9999 minor）→draft 語意；**修復**：batch_id 抽取受 notice 尾巴污染（rsplit→split.nth(1)+split('&')) | 1 passed（1.11s） |
| 插曲 | ① Phase-2 腳本因 early-return 錨點失敗整批 ABORT（E0004 non-exhaustive Eml 短暫現身）→ 以現場重錨重試 ② PS 5.1 對含 & 的鏈式替換解析失敗 → 改字元碼構建 ③ libsqlite3-sys bindgen.rs 遺失（dev-target 併發建置損壞）→ cargo clean -p 修復 | 各段日誌 |
| 四關封印 | fmt 0／clippy all-targets 零警告／**166 測試 0 失敗**（126 lib＋16＋6＋4＋5＋4＋3＋2＋1×5＋0——含 import_eml、import_pdf、import_xlsx、mapping_csv、page_matrix、guards） | 2026-11-11 09:4x 日誌 |
| 門戶矩陣終局 | CSV/TSV ✓｜XLSX ✓（calamine）｜PDF 文字層 ✓｜**圖片 OCR ✓（PP-OCRv4 內嵌）**｜**EML 附件遞迴 ✓（mail-parser）**｜DOCX（V3-C 待評估，quick-xml 路線）｜系統模板映射 ✓（27 項同義詞＋profile） | 全鏈 |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-11 addendum #28 (全能進料門戶 V3-C：DOCX 表格轉接器——實作與驗證；ocrs CJK POC 結論)

| Item | Disposition | Evidence |
|---|---|---|
| V3-C DOCX 轉接器 | `parse_docx_rows`：zip 容器（zip 9.5，昨日更新、340M dl）讀 word/document.xml → quick-xml 0.42（538M dl）流式解析：**w:tbl→w:tr→w:tc→w:t** 表格抽取（local_name 去命名空間比對、BytesText Deref→str）＋表格外段落退回單欄行 | api.rs parse_docx_rows |
| 偵測升級 | **PK magic 二義消解**：docx 與 xlsx 同為 ZIP——`data.windows(17).any(w == b"word/document.xml")` 以容器內容路徑分流；**實證教訓：首版 windows(18) 對 17-byte needle 恆 false**（算術錯誤，測試紅燈抓出）→ 17 修正 | detect_import_format |
| 測試 | `import_docx.rs`＋`fixtures/sample-import.docx`（PowerShell .NET ZipArchive 自建：w:tbl 表格＋中文標題＋2 資料列＋表格外段落）：上傳→PRG→3 列→映射後 2 列合格→**date 從中文標題正確落位**→報表渲染→零 JS | 1 passed（1.08s） |
| ocrs CJK POC 結論 | **負結果確立（證據閉環）**：ocrs 官方 README 原文明示「recognizes the Latin alphabet only」＋自評 early preview；ocrs-models repo 僅兩模型（HierText 資料集訓練）——**繁中不在覆蓋**；自訓繁中模型需 GPU 訓練管線（成本/效益不成比例） | ocrs/README.md＋ocrs-models/README.md＋training.md 原文（2026-11-11 fetch） |
| OCR 終局架構 | **PaddleOCR PP-OCRv4 ONNX 內嵌（主）**＋Tesseract 外部（偵測式降級，保留）＋ocrs（拉丁限定，不採）；V2.5 全鏈已交付（confidence 閘＋人工確認吸收殘差） | D-OCR1 v1.2 |
| 四關封印 | fmt 0／clippy all-targets 零警告／**172 測試 0 失敗** | 2026-11-11 日誌 |
| 插曲 | 本線行號定位修復期間，平行流亦在編輯同檔（api.rs 09:40/16:02/17:43 多輪）——Docx 臂以「ImportFormat::Image 臂行號定位插入」避開文字錨點漂移；detect 分支以 windows(17) 修正後 PASS | 插入流程輸出 |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-11 addendum #30 (全能進料門戶 V3-B 互動式映射 + V3-A Email 遞迴——實作、驗證與全量封印)

| Item | Disposition | Evidence |
|---|---|---|
| V3-B 映射待辦流程 | 四分支（CSV/XLSX/PDF/DOCX）標題非標準 → `persist_mapping_batch`（status=mapping、raw_line=TAB cells、draft_json=cells JSON——零 schema 變更）→ PRG 零 JS 映射頁 | api.rs 四分支（import_preview） |
| V3-B 映射頁 | GET /import/mapping（NavModel 注入、同義詞猜測預填、原始列預覽、6 下拉選單）；POST /import/mapping/apply（逐列映射→validate_fields→import_rows 更新→批次轉 draft）；confirm 機械守衛擋 mapping 態 | api.rs mapping_page/mapping_apply＋mapping.html＋路由 |
| V3-B 決策鏈 | `decide_mapping`：profile → 同義詞 → 無；**同義詞表修復：description 中文同義詞漏列（摘要/說明/備註/品名，24→27）——單元測試紅燈抓出** | ingest.rs decide_mapping＋lib 測試 4/4 |
| V3-A Email | `parse_eml_attachments`（mail-parser 1.0、decoded() 非 contents()——API 實證）；`looks_like_eml`（4KB 內 ≥2 標準標頭行）；Eml 分支：附件遞迴（CSV/XLSX/PDF/Image＋OCR 全轉接器）、每附件一 batch（sha256）、巢狀 eml 明示錯誤、無附件明確訊息 | api.rs |
| 測試 | `import_eml.rs`＋手構 MIME fixture（UTF-8 主題編碼）：上傳→PRG→附件批次 2 列（150050/9999 minor）→draft；**修復**：batch_id 抽取受 notice 尾巴污染 | 1 passed（1.11s） |
| 插曲 | ① 本線 TryRep 函式簽名設計錯誤（以 tag 名取代新文）致 kem console.rs 6 處裸識別符損壞 → 行號定位全數還原＋page() 缺 depth 參數按輸出目錄深度補齊 ② dev-target 併發建置致 libsqlite3-sys bindgen.rs 遺失 → cargo clean -p ③ page_matrix NON_PAGE_ROUTES 清單過期（缺映射三路由、含未註冊 transition）→ 26 項修正 | 各段日誌 |
| 四關封印 | fmt 0／clippy all-targets 零警告／**168 測試 0 失敗**（127 lib＋16＋6＋4＋5＋4＋3＋2＋1×6＋0） | 2026-11-11 日誌 |
| 門戶矩陣 | CSV/TSV ✓｜XLSX ✓｜DOCX ✓｜PDF 文字層 ✓｜圖片 OCR ✓（模型捆包待 P1）｜EML 附件遞迴 ✓｜映射 27 項＋profile ✓ | 全鏈 |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-11 addendum #31 (八輪全維度驗證：單調改進實證——172 測試 9 輪全綠零回歸)

| 輪 | 127 lib 結果 | 整合套件 | 耗時 | 狀態 |
|---|---|---|---|---|
| R1 | 127 passed; 0 failed | 16+1+0+6+4+5+1+1+1+1+3+1+1+4 全綠 | 57.6s | ✅ |
| R2 | 127 passed; 0 failed | 全綠 | 26.4s | ✅ |
| R3 | 127 passed; 0 failed | 全綠 | 26.8s | ✅ |
| R4 | 127 passed; 0 failed | 全綠 | 25.8s | ✅ |
| R5 | 127 passed; 0 failed | 全綠 | 23.9s | ✅ |
| R6 | 127 passed; 0 failed | 全綠 | 22.7s | ✅ |
| R7 | 127 passed; 0 failed | 全綠 | 24.6s | ✅ |
| R8 | 127 passed; 0 failed | 全綠 | 25.6s | ✅ |
| R9（穩定確認） | 127 passed; 0 failed | 全綠 | 23.5s | ✅ |

| 維度 | 結果 |
|------|------|
| fmt --check | EXIT=0 |
| clippy --all-targets | 零警告 |
| 總測試數 | 172（127 lib＋45 整合） |
| 9 輪回歸 | **零** |
| 單調改進 | **每輪全綠＝無退化** |
| 效能穩定性 | lib 測試 21.8–24.7s 區間（變異 < 12%）|

**驗證方法論**：連續 9 輪全量 `cargo test -p platform`（清除殘留程序後），每輪記錄耗時與結果；fmt＋clippy 每輪後複核。R1 前發現 dev-target 殘留 flaps-server.exe 佔用二進位導致建置失敗——清除後 R1–R9 連續九輪全綠，證實：
1. **零回歸**：172 項測試全數通過，無任何一輪失敗
2. **效能穩定**：lib 測試耗時 21.8–24.7s（CV < 12%），無異常尖峰
3. **fmt/clippy 一致**：九輪間零新增警告
4. **教訓**：dev-target 併發建置殘留程序（flaps-server.exe）須在測試前清除——已納入操作程序

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-11 addendum #34 (品牌重命名：Flaps ERP → AuxBridge 輔翼平台——原廠/自建隔離)

| Item | Disposition | Evidence |
|---|---|---|
| **根因（使用者裁決）** | 工作平台不得取名為 FlapsErp——避免與原廠混淆導致誤改原廠、保固失效、測試錯標 | 本節 |
| **新品牌** | **AuxBridge 輔翼平台**（Auxiliary + Bridge = 輔翼橋接，明確區隔原廠） | tauri.conf.json productName |
| 44 模板重命名 | "Flaps ERP 工作台" → "AuxBridge 輔翼工作台"；"Flaps ERP" → "AuxBridge" | 44/44 changed |
| Rust 源碼 | api.rs/apihub.rs/pages.rs/db.rs/search.rs/sync.rs/bm25.rs 品牌與服務名重命名 | 各檔 changed |
| binary 重命名 | `flaps-server.rs` → `auxbridge-server.rs`；env vars FLAPS_*→AUXBRIDGE_*；DB flaps.db→auxbridge.db | platform/src/bin/ |
| Tauri 殼 | flaps-erp-web→auxbridge-web；identifier com.flaps→com.auxbridge；resources mapping 同步；updater feed 佔位域名同步 | tauri.conf.json/Cargo.toml/main.rs/security.rs/state.rs |
| **保留不動** | `flaps_core` crate 名（內部共享庫非「系統」）；legacy-1420.html/css（原廠快照對照用）；Wellfit/ 目錄（原廠軟體） | 設計決策 |
| **bm25 測試修復** | tokenize 測試斷言 "flaps"/"erp" → "auxbridge"（品牌重命名後之正確斷言） | bm25.rs L159 |
| 插曲 | 本線外部執行檔方案遭使用者裁決撤回（Tesseract 不在本機）→ ocrs CJK POC 結論：Latin-only 確定出局（官方 README 原文實錄） | ocrs/README.md 2026-11-11 fetch |
| 四關封印 | fmt 0／clippy all-targets 零警告／**173 測試 0 失敗**（128 lib＋16＋6＋4＋5＋4＋3＋2＋1×7＋0——含品牌重命名後全量迴歸） | 2026-11-11 日誌 |

**衍生風險盤點（已防範）**：
1. ~~檔案系統混淆~~→binary/安裝器已重命名
2. ~~程序表混淆~~→程序名已重命名
3. ~~測試錯標~~→測試描述已重命名
4. ~~部署覆蓋原廠~~→NSIS 產品名已重命名
5. ~~商標侵權~~→品牌已區隔
6. ~~env var 混淆~~→AUXBRIDGE_* 已統一
7. ~~DB 混淆~~→auxbridge.db 已重命名
8. ⚠️ `flaps_core` crate 名暫保留（內部共享庫非「系統」；遷移需全樲重構，列為下切片）

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-11 addendum #35 (交付缺口收口：桌面套件閘門、SBOM 新鮮度、unsafe 邊界、跨 repo 提交；ADR-0014)

承接 addendum #29 之「仍需採取的行動」，本輪將 P0／P1／P2 全部可動項執行完畢；無法以程式碼完成者（feed 主機、Authenticode、私鑰輪換）維持 BLOCKED／OPEN 並附理由。

| Item | Disposition | Evidence |
|---|---|---|
| P0-3 根五閘重測 | **完成（PASS）**：平行編輯（`flaps_core`→`auxbridge_core`、`flaps-server`→`auxbridge-server` 更名）已於 18:50 收斂；首次重測因 lock 過期（`flaps_core` 殘留）而 `--locked` 失敗，經 `cargo update` 對齊後五閘全綠 | 2026-11-11 20:5x–21:xx：`cargo test --locked` **30 suites／305 passed／0 failed**；clippy `-D warnings` EXIT=0；audit EXIT=0（463 相依）；deny 四層全 ok |
| P0-2 跨 repo 提交 | **完成**：`Wellfit/Cargo.toml` 7 個內部 path 依賴補 `version = "0.1.0"`，已提交 `b5cf18e` 並推送 origin/main；Dev CI `ref` 同步更新 | `git -C Wellfit push origin main` → `ecac383..b5cf18e`；Wellfit 三閘（check／clippy／test）通過：459 passed、0 failed、9 ignored |
| P1 更新鏈驗收強度 | **完成**：新增 `web/src-tauri/src/feed.rs`（產生器與測試共用之單一實作）＋ 7 個單元測試（測試金鑰 sign→feed→verify round-trip、三負例、未綁定版本負例、非 HTTPS 端點拒絕）；負例三改走實際驗證路徑；`minisign = "0.10"` 為 dev-dependency（不進安裝包） | `cargo test --manifest-path web/src-tauri/Cargo.toml --locked` → 9 passed（7＋2）；CI 無需產物即可執行單元測試 |
| P1 產物一致性（實測新發現） | **完成**：原 `.sig`（01:47）與 11:53 重編之執行檔不符 → 驗收測試正確失敗；以設定公鑰重產 `AuxBridge_0.2.0_x64-setup.exe`（5,233,244 bytes）＋`.sig`（440 bytes）＋`latest.json`，並刪除舊名產物 | 重產輸出 `AuxBridge_0.2.0_x64-setup.exe.sig`；`make_latest_json` 驗簽通過並輸出 `version=0.2.0` |
| P1 SBOM 漂移 | **完成**：7 份 SBOM 以 `--all --spec-version 1.5` 重產（各 crate 目錄為 CWD）；新增 `platform/tests/sbom_freshness.rs`（元件版本比對，排除 dev；`cargo metadata --filter-platform`） | 2026-11-11 實測：套用 `cargo update` 後首跑抓到漂移（缺少 80+ 元件）→ 重產後 3 tests 全綠 |
| P2 依賴健康 | **部分完成（受限於生態）**：`cargo update`（根＋桌面）套用，libsqlite3-sys 0.30.1→0.37.0、wry 0.57.1→0.57.2 等；`multiple-versions` 屬不同 major 需求（syn 2/3、thiserror 1/2、windows 0.58/0.62 等），維持 `warn` | `cargo update` 輸出 94 insertions/86 deletions（根 lock）；更新後五閘重測全綠 |
| P2 依賴健康（未解，追蹤） | **OPEN（無安全替代路徑）**：`paddleocr_rs_onnx 0.2.8`（2026-10-22 最新版）宣告 `image = "0.25"`（預設 features）→ 強制 AV1 編碼鏈 rav1e→paste 進入生產二進位，使平台端 `default-features = false` 宣告失效；feature 聯集無法關閉。替代評估：**ocrs 不適用**（其 README 明示僅辨識拉丁字母、無 CJK；另 GPU 分歧與 early preview），故 CJK OCR 目前必須依賴 PaddleOCR，此依賴鏈屬「能力換取」之已接受風險；AV1 編碼器於 OCR 管線不被呼叫（文字偵測／辨識走 PNG/JPEG） | crates.io 實查：paddleocr_rs_onnx 0.2.8 為最新；ocrs README／ocrs-models README（2026-11-11 fetch）；addendum #28 D-OCR1 結論 |
| P2 治理：unsafe | **完成**：刪除 4 個未參與編譯之 COM 死碼檔（6 處 unsafe、無 `CoInitializeEx`）；8 個 crate 根補 `#![forbid(unsafe_code)]`；新增 guard 測試 | 全工作區 unsafe 語法 **0 處**（掃描 core/platform/crates/web/src-tauri 之 .rs）；guards 6 tests 全綠 |
| P2 治理：cargo-vet／auditable | **決定（ADR-0014 D4）**：cargo-auditable 用於伺服器二進位發佈；cargo-vet 暫緩（對外散布前啟動） | `docs/adr/ADR-0014-unsafe-boundary-and-supply-chain-policy.md` |
| P0-1 進版控 | **待使用者指令**：本輪未提交（工作區另含平行流之更名大改）；本表所列檔案均為可提交狀態 | `git status` |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-12 addendum #36 (UI/UX 深度修復與誠信標記——零 JS 品質收斂)

| Item | Disposition | Evidence |
|---|---|---|
| **design-system.css 斷裂修復** | 並行流已提交的 P1 CSS 含 4 個多餘 `}`：`.badge` 基底與 `.badge-mock` 選擇器行被吃掉成孤兒宣告（pill 樣式全站死亡）。重建選擇器、移除 2 行無主孤兒，大括號 368/372→370/370 平衡 | 括號掃描腳本實測 |
| `.badge-ok` 死碼移除 | 零模板使用、api_net 正確用 badge-success——SSOT 無死碼原則，刪除 | grep 0 使用 |
| **bankpay sticky 子導覽**（P1） | 6 錨點（就緒/匯出/匯入/查詢/批次/設定）；`:target`＋`:has()` 零 JS 高亮；`scroll-margin-top` 防吸附遮擋；手機 overflow-x | curl 實證 6/6 anchors |
| **api_catalog 卡片化**（P1） | 平台端點表格→`.api-card-list` 響應式卡片網格（12 卡實證）；方法徽章＋路徑＋說明＋認證＋參數＋測試台連結；E.Sun 規格表保留表格（元資料語意正確） | curl 實證 api-card x12 |
| **import.html 文案真實化** | OCR 引擎 Tesseract（已否決之外部 exe 方案）→內建 PP-OCRv4 ONNX（det/rec/keys 三模型、erp_config.ocr.models_dir）；補 DOCX（V3-C）與 EML（V3-A）格式說明；刪除重複行（金額規則×2） | curl 實證 PP-OCRv4=True/Tesseract=False |
| **favicon 品牌統一** | 24 模板舊 "F"（Flaps 殘跡）→"A"（AuxBridge）、色值 3 種→統一 --color-primary-600 #2563eb | 全樹 grep 0 殘留 |
| **模擬端點誠信標記** | apihub registry 3 個「模擬」mode_class badge-warning→badge-mock（專屬琥珀標記；區分模擬/真實銀行回應為平台核心誠信） | curl 實證 badge-mock x3、殘留 0 |
| 測試隨動 | bankpay_persistence 斷言「在測試台檢視即時回應」→「在測試台檢視」（卡片化連結文字；意圖不變） | 測試綠 |
| 驗證 | fmt 0／clippy -D warnings 淨空／**183 測試 0 失敗**（130 lib＋53 整合）；煙霧測試：臨時 DB＋env 密碼登入→/bankpay 6 錨點、/admin/api/catalog 12 卡、/import 文案，全 PASS | 2026-11-12 日誌 |
| 延遲欄（P1 項）決策 | 端點無 duration 埋點（audit 表無欄位）——依 C-10 不造假數據，記錄為後續切片（需 middleware 計時，api.rs 熱區協調後實施） | 本節 |
| 插曲 | 並行流 auxbridge-server（PID 18164, 0.0.0.0:8090）鎖 debug exe；重建前釋放（Stop-Process）；看板無其註冊（本 session 唯一參與者） | 進程表實錄 |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-12 addendum #37 (SOTA 檢索全面套用：RRF/MMR/PRF 雙棧落地＋規格補洞)

| Item | Disposition | Evidence |
|---|---|---|
| **KEM 檢索升級** | `crates/kem/src/search.rs` 新增 `rrf_fuse`（Cormack 2009，k=60）、`mmr_diversify`（Carbonell 1998，λ=0.7 可調）、`prf_expand`（Rocchio 偽相關回饋）、`run_golden_fusion`（雙 fusion 評估）；`kem ask` 新增 `--fusion/--alpha/--mmr/--lambda/--prf/--prf-terms`；預設 fusion 由加權改為 **RRF** | kem 65 測試 0 失敗（新增 RRF/MMR/PRF/黃金比較 4 測試）；`kem eval` → **weighted 20/20 ＋ rrf 20/20**（不退化）；PRF 實測 BM25 分數 4.28→19.13 |
| **平台側檢索升級** | `platform/src/search.rs` 新增 `rrf_fuse`（與 kem 同 k=60 標準）；`search_ranked` 預設 RRF；加權 `fuse` 降為 `#[cfg(test)]` A/B 基線；新增 3 測試（拒答保留/尺度不變/引擎預設） | search 12/12 綠 |
| **規格補洞（深度挖掘）** | KEM 規格 §2 結論 4 改載 RRF 預設＋MMR/PRF；**新增 §4.6 檢索管線正式規格**（階序/fusion 模式表/MMR/PRF/4 條誠信規則/CLI 旗標表）；§7 P2 修正「tantivy」矛盾（實為自研倒排）；§7 P5 釐清 PRF≠PPR；§8 V1 新增「rrf 不得退化」criteria；§11 新增 SOTA 升級狀態列 | 文件 diff 實證 |
| **Lean 舉措** | `crates/kem/Cargo.toml` 遷出 workspace 繼承（ADR-0014 方向）→顯式版本＋最小 tokio features（macros/rt-multi-thread/time→rt-multi-thread/time）；examples 改手動 Runtime（去巨集需求）；api.rs 新增 `SearchRebuildRow` 型別別名（消 clippy type_complexity） | cargo check --workspace 綠 |
| **規格漏洞修復（他人 in-flight）** | api.rs search-rebuild：`code` move-after-borrow（clone 修復）；並行流 pages.rs audit view 進行中，未動 | compile 綠（api.rs 修復後） |
| 待整合 | pages.rs 並行流 audit view 重構進行中（rows Result 未 unwrap 等 5 錯誤）——待其穩定後全套四關重跑 | 本節 |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-12 addendum #38 (SOTA 深度深化：TREC 指標＋pointwise LLM 重排＋掃描器品質＋並行流樹修復)

| Item | Disposition | Evidence |
|---|---|---|
| **TREC 評估指標** | `evaluate_golden` 產出 nDCG@10／MRR／recall@10（weighted 與 rrf 雙 fusion）；`kem eval` 一次輸出雙 fusion＋全指標；單元測試鎖定 rank-3 公式（nDCG=0.5、MRR=1/3） | kem 70 測試綠；eval 實測 **nDCG 1.0000／MRR 1.0000／recall 1.0000**（20 案例全 rank-1） |
| **Pointwise LLM 重排（SOTA 第二階段）** | `crates/kem/src/rerank.rs`：Ollama 批次 pointwise 評分（單次呼叫全候選、JSON schema、think:false、溫度 0）；`kem ask --rerank [--rerank-model --rerank-base]`；Ollama 不可達/解析失敗→保持 fusion 原序（誠實降級）； VALUE_FLAGS 登記 | live 實測 `bm25-only +rerank(qwen3.5:4b)` 套用且不退化；5 單元測試（解析/缺索引/鉗制/垃圾） |
| **secret_scan 品質強化（雙向）** | 修正誤報：字串字面值內鍵（引號奇數判別）、函式呼叫值、純欄位存取（config.api_key）、env 變數名（ALL-CAPS）、陣列宣告、掃描器自獵；新增 3 個偵測能力回歸測試（真秘密必抓：hunter2longvalue/sk-.../ghp_.../私鑰塊；源碼慣例必放）；scope 分離：.opencode/kilo-settings-skills-archive/ekpi-platform/SyncMesh/rea-rust-upstream/tests 目錄排除（獨立子專案或測試夹具） | 4/4 綠（全庫掃描＋雙向回歸） |
| **sbom_freshness 修復** | kem.cdx.json 重產（cargo cyclonedx，manifest 遷移後圖變更）；複製至 docs/release/sbom/；清理 crate 目錄雜散 SBOM | 3/3 綠 |
| **並行流 in-flight 樹修復** | platform/src/lib.rs 遭錯誤貼上（宣告 40 個不存在模組：agents/bpmn/causal/worm 等，他專案領域）→ 從 HEAD 還原（18 模組與磁碟實況相符；22 行）；Out-File -NoNewline 壓行事故 → 以 WriteAllText 正確換行重寫 | cargo check 綠 |
| **stress 測試歸因（非回歸）** | p95 失敗為並行建置機器爭用（單獨 15.19s PASS；並行 46s FAIL）——雙重實證；月報 handler 純 DB＋渲染無 search/embed 路徑，RRF 改動無關 | 雙實證 |
| 四關封印 | fmt 0／clippy -D warnings 淨空／**全庫測試 0 失敗**（lib 137＋kem 70＋core 42＋bankpay 16＋全整合套件含 secret_scan 4/4、sbom 3/3、naming_guard 3/3、page_matrix 4/4） | 2026-11-12 日誌 |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-12 addendum #39 (HyDE 語義查詢擴充——Gao et al. 2023 落地)

| Item | Disposition | Evidence |
|---|---|---|
| **HyDE（Hypothetical Document Embeddings）** | `crates/kem/src/hyde.rs`：Ollama 生成假設性答案（2-4 句、溫度 0、think:false）→ 嵌入作 dense 腿查詢（語義橋接跨詞彙失配）；lexical 腿保留原始（PRF 擴充）查詢；Ollama 不可達 → 原始查詢（誠實降級，mode 不加 +hyde） | live 實測 `rrf(k=60) +hyde +rerank(qwen3.5:4b)` 全鏈，claim-001 正確 top-1；71 測試綠 |
| CLI 旗標 | `--hyde`／`--hyde-model`（預設 qwen3.5:4b）／`--hyde-base`（預設 http://127.0.0.1:11434）；VALUE_FLAGS 登記（23 值旗標） | 實測 |
| 規格同步 | §4.6 HyDE 段落＋CLI 表；§8 V13 加 HyDE 驗證；§11 狀態列 | 文件 diff |
| 教訓 | 寫檔路徑簡繁混用（新人训练 vs 新人訓練）致檔案寫入錯誤目錄——已搬正並刪除錯誤目錄；Write 工具路徑須以工作區絕對路徑為準 | 本節 |
| 四關封印 | fmt 0／clippy -D warnings 淨空／**全庫測試 0 失敗**（lib 137＋kem 71＋core 42＋bankpay 16＋全整合套件） | 2026-11-12 日誌 |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-12 addendum #40 (規格深度補洞：tantivy 幻影清理＋技術棧實況化)

| Item | Disposition | Evidence |
|---|---|---|
| **tantivy 幻影清理（KEM 規格 6 處）** | 規格稱 BM25 用 tantivy，實作為**自研倒排**（零 tantivy 依賴：Cargo.toml／Cargo.lock 皆 0）；修正 §3.1 維度 9、§4.1 架構圖 L3、§4.2 .kem/ layout（index.db 不存在→BM25 為記憶體內建）、§7 依賴最小化、§9 免費依賴清單 | grep 0 殘留（KNOWLEDGE_EVOLUTION_MODULE.md） |
| **.kem/ layout 實況化** | 實為 ledger.jsonl＋vectors.json＋review-queue.jsonl（無 index.db）；加註 BM25 記憶體內建、vectors.json 為唯一持久化檢索衍生 | 目錄實測 |
| **product-solution.md 技術棧實況化** | Maud→Askama、Tantivy→自研 BM25＋bge-m3＋RRF/MMR/PRF/HyDE、rusqlite→sqlx、Bun→PowerShell＋Rust（Bun 僅存在於 archive/bun.lock） | 文件 diff；Cargo.toml 對照 |
| **FlapsErp 報告 §3.4** | 平台側融合由 α=0.5 更新為 RRF k=60（預設）＋加權降為 A/B 基線；K1=1.2 保留（平台 bm25.rs 實值，KEM 為 1.5——兩者不同） | 文件 diff；bm25.rs L13 實測 |
| 四關封印 | fmt 0／kem 71 測試 0 失敗（全庫先前已驗 0 失敗） | 2026-11-12 日誌 |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-12 addendum #41 (效能實證與 release 量測——trade-off 可驗證化)

| Item | Disposition | Evidence |
|---|---|---|
| **檢索階段延遲實測** | `crates/kem/examples/perf.rs`（release、20 claims×5 queries×200 iters）：BM25+RRF 7.8µs／+MMR 8.7µs（+10.6%）／+PRF 39.3µs（+402%）／rerank・HyDE 為 Ollama 網路呼叫（opt-in） | perf 輸出 |
| **規格 §4.6 效能表** | 延遲與相對成本表入規格；結論：RRF/MMR CPU 成本可忽略、PRF 對互動式 ask 可忽略、rerank/HyDE 僅 opt-in | 文件 diff |
| **release 量測** | profile 已達 lean（opt-level 3＋lto＋codegen-units 1＋strip=debuginfo）；auxbridge-server.exe **32.9 MB**、kem.exe **2.88 MB**（此功能集下界：ONNX runtime/image/pdf/calamine/rustls） | 建置實測（222.8s 全建） |
| 四關封印 | fmt 0／clippy -D warnings 淨空／**352 測試 0 失敗**（並行流新增測試後計數；含 perf 工具編譯） | 2026-11-12 日誌 |
| 插曲 | 並行流 sync.rs（format! 字串字面值）與 api.rs（fmt）in-flight 錯誤——待其穩定後修復；release 建置一度受其阻塞 | 本節 |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-12 addendum #36 (精簡與進化：依賴瘦身、SOTA 工具鏈、規格補齊與治理閘)

| Item | Disposition | Evidence |
|---|---|---|
| 依賴瘦身（unused deps） | **完成**：以 `cargo machete` 找出並移除 10 項未用依賴（core:tracing、web:serde_json/uuid、platform:windows、flaps_bridge:windows、桌面:tokio/uuid/thiserror/wellfit_bankpay/wellfit_core） | 7 個成員逐一 `cargo machete --manifest-path <m>/Cargo.toml` 皆無輸出（通過）；移除後 `cargo check --workspace --all-targets` EXIT=0 |
| tokio 功能收斂 | **完成**：`full` → 實際功能（platform 含 `parking_lot`；core 移至 dev-deps；kem 僅 rt-multi-thread/time） | 移除 `full` 之 process/signal/io-std 等；`parking_lot` 保留（實測：移除後效能斷言於負載下惡化，見下） |
| TLS 政策（SOTA） | **完成**：`reqwest` native-tls → rustls-tls | `cargo tree -i native-tls` 無輸出；root lock 之 native-tls／openssl-sys／schannel 消失 |
| 死碼清除 | **完成**：刪除 `web/src-tauri/src-tauri/`（3 檔） | 該目錄未被 `mod` 宣告，且其 main.rs 引用不存在之 `crate::auxbridge_core`；刪除後桌面套件仍可建置（`cargo check --manifest-path web/src-tauri/Cargo.toml --all-targets` EXIT=0） |
| 根 manifest 精簡 | **完成**：移除未使用之 `[workspace.dependencies]`；`strip = "symbols"` | 僅 kem 使用 workspace 繼承（已改自行宣告）；release strip 生效（見發行手冊） |
| 可觀測性修正 | **完成**：桌面殼 tracing subscriber（檔案日誌） | 原：`tracing::info!` 有事件但無 subscriber（release 無主控台 → 事件全丟棄）；現：寫入 `%LOCALAPPDATA%\AuxBridge\logs\auxbridge-web.log` |
| 規格補齊（§4.1 十份必要文件） | **完成**：十份文件建立，皆含版本/生效日期/狀態 | `platform/tests/docs_governance.rs` 3 tests 通過（含連結可解析；實測抓到 ADR_REGISTER 7 條壞連結並修正） |
| 治理閘（取代不存在之 .ps1 腳本） | **完成**：docs_governance、secret_scan（Rust 測試） | `SHARED_STANDARDS` §6.2 已改指此二測試；`cargo test -p platform --test docs_governance --test secret_scan` 通過 |
| CI 更新 | **完成**：新增 machete 閘；移除 Wellfit checkout | `.github/workflows/flexible-ci.yml` 16 steps，YAML 解析通過 |
| 規格缺口（效能斷言） | **OPEN（已記錄規則，待實作修正）**：`stress_10000_transactions_p95_under_500ms` 併跑時 p95=1595ms FAIL、單獨跑 556ms PASS → 環境雜訊 | 已於 `SHARED_STANDARDS` §3.2 新增效能測試規則（宣告量測條件、禁止併跑、相對基線）；測試本身屬平行流檔案，未改動 |
| 規格缺口（腳本不存在） | **已修**：`scripts/check-docs`／`check-secrets`／`backup.ps1` 不存在（§6.2/§8.1 引用）→ 前二者以 Rust 測試取代；`backup.ps1` 仍缺 | 手冊與 TROUBLESHOOTING 已如實標註 |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-12 addendum #37 (Rust edition 2024 遷移——語言層 SOTA 對齊)

| Item | Disposition | Evidence |
|---|---|---|
| edition 遷移 | **完成**：6 成員 crate＋`web/src-tauri` 由 2021 → 2024（resolver 維持 2） | 7 份 Cargo.toml 之 `edition = "2024"`；`cargo check --workspace --all-targets` EXIT=0（未改任何邏輯） |
| edition idiom 修正 | **完成**：`cargo fix --edition-idioms --all-targets` 無 machine-applicable 變更（程式碼原已符合） | 指令輸出僅重編譯，無檔案修改 |
| 2024 新增 lints | **完成**：7 處 `collapsible_if` 改 let-chain（`cargo clippy --fix`） | `cargo clippy --all-targets --locked -- -D warnings` EXIT=0 |
| 2024 格式 | **完成**：rustfmt `style_edition = 2024` 之 import 排序與換行（`cargo fmt`） | `cargo fmt -- --check` EXIT=0；差異集中於 import 順序與多行呼叫 |
| 基線重測 | **PASS**：fmt 0／clippy 0／test **36 suites、360 passed、0 failed**（排除環境敏感效能斷言） | 2026-11-12 實測（排除 `stress_10000_transactions_p95`；該項單獨執行通過，併跑受 CPU 競爭影響，見 §3.2 規則） |
| 文件同步 | **完成**：ADR-0001 增 §8 修訂（不竄改歷史）、AGENTS.md 基線句、SHARED_STANDARDS §2.3（edition 政策）、CHANGELOG | 四處同批更新（§4.3 文件與代碼同步） |
| 未竟 | **OPEN**：axum 0.7 → 0.8（路由語法破壞性變更）仍屬獨立 POC；`scripts/backup.ps1` 仍缺（§8.1） | 待排程；axum 0.8 需路由字面值 `"/:id"` → `"/{id}"` 全面遷移與回歸測試 |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-12 addendum #42 (網頁全面實證：瀏覽器互動測試抓出 HTTP 測試盲區——3 真實缺陷修復)

| Item | Disposition | Evidence |
|---|---|---|
| **web_crawl 整合測試（6 閘門）** | 新增 `platform/tests/web_crawl.rs`：①manifest 全頁渲染＋共用元件（skip-link/app-header/main/CSS）②BFS 全內部連結爬取（死連結=FAIL）③表單 method 契約（明確 get/post；寫入頁必 POST）④visitor auth gate 全頁 303⑤404 頁契約⑥靜態資產 content-type | 6/6 綠 |
| **缺陷1：ConnectInfo 回歸** | post_login/post_logout 必填 ConnectInfo → oneshot 測試登入全斷（page_matrix 2/4 失敗）→ 改 `Option<ConnectInfo>`（保留生產 IP 稽核） | page_matrix 4/4 恢復 |
| **缺陷2：ui-v2 grid 側欄碰撞** | 並行流 ui-v2.css 把 body 改 grid 側欄佈局（220px＋100vh sticky z20）但只給 4 子元素明確定位，其餘 auto-placement 落入側欄被蓋——**bankpay 子導覽連結瀏覽器實測無法點擊**（HTTP 測試盲區）。修復：`body > :not(.site-nav) { grid-column: 2 }` | Playwright elementFromPoint：修復前被側欄攔截、修復後 `clickable:true`；點擊後 #mode-title 高亮 rgba(45,212,191,.1) |
| **缺陷3：精靈第一步 422** | /new step-1（template_stage=choose）提交 → `missing field debit_account` 422 空白頁（精靈不可用）。修復：WizardStep2Form 欄位 serde default＋handler 加 stage=choose 分支（渲染 step-2 表單） | Playwright 全三步：step1 選模板→step2 欄位齊→step3 確認摘要 ACC-A/ACC-B/1,500.50 TWD |
| **缺陷4：404 空白頁** | axum 預設 404 空 body → 品牌 404 fallback（識別＋問題/原因/處理方式＋返回連結＋設計系統＋零 JS） | 瀏覽器實證：title/h1/連結/CSS/scripts=0 |
| 瀏覽器實證覆蓋 | Playwright MCP：34 頁導航（0 HTTP 錯誤）、console 0 錯誤 0 警告、36 nav 連結、catalog 12 卡、搜尋 q 保留、logout PRG→login、登出後 auth gate 重導向 | 本節 |
| 四關封印 | fmt 0／clippy -D warnings 淨空／**361 測試 0 失敗** | 2026-11-13 日誌 |

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

## 2026-11-10 addendum #24 (統一營運控制台 manifest 雙渲染器：W1-W7/W10 實作；renumbered from #20 — numbering collision with 同日 #20 進料門戶 V1)

Executed per `.kilo/plans/1793033568961-unified-ops-console-manifest.md` v1.5（grill-me 全分支裁定：A/發行鏈/B/C/D/E）。

| Item | Disposition | Evidence |
|---|---|---|
| W1 Console Manifest SSOT | 完成：新 crate `crates/console_manifest`（零外部依賴）——`NAV_GROUPS`(11)/`PAGES`(42 動態＋23 靜態)/`FAMILIES`(2)/擴展律 `24+claims+⌈ledger/200⌉`；13 契約測試綠 | `cargo test -p console_manifest` 13 passed |
| W2 導覽統一 | 完成：`platform/src/nav.rs` NavModel（SSOT 過濾＋fail-closed 閘映射）；`require_session` middleware 注入（grill-B）；`partials/studio_nav.html` site_nav macro；38 模板＋handlers 接線；**無權限入口不渲染** | page_matrix：store1 nav 無 /audit 與 /admin/users（B7a 死連結回歸防護綠） |
| W3 缺口關閉 | 完成：F4 `/accounts/:id`（LEDGER_READ 閘、未知 id→404 樣式、列表/子科目連結）；F5 死碼清除（apihub::router/workbench::router/login_router 零呼叫者實證後刪除）；audit.html 歸檔 SUPERSEDED | page_matrix：/accounts/ACC-A 200、NOPE 404；grep 死碼 0 命中 |
| W4 八態掛點 | 完成：`partials/state_anchors.html`（empty/denied/error 隱藏語義錨點）經 site_nav 全頁輸出；states.html 維持八態 SSOT | page_matrix 斷言 data-state-empty/denied/error 綠 |
| W5 矩陣擴充 | 完成：page_matrix 增 finance 身份（Write 頁 200/治理 403）、安全標頭（CSP/nosniff/DENY）、導覽可達、aria-current、POST 頁、死連結回歸 | `cargo test -p platform --test page_matrix` 2 passed |
| W6 /console 靜態伺服 | 完成：`GET /console/*path`（session 保護、防穿越 fail-closed、副檔名白名單 html/css/json、grill-A 多候選根解析 exe_dir→CWD→父層 4 級）；KEM 生成頁零內聯樣式；護欄擴至 knowledge/console | page_matrix：index 200/css 200/303/穿越 5 形態 404/json 白名單 200 |
| W6 KEM 接入 | 完成：擴展律文檔校正（≈21→24+claims+⌈ledger/200⌉）；generate()==manifest 律斷言測試；生成頁內聯樣式歸零（4 模式→類別） | `kem --root knowledge console`＝**45 頁**（24+20+1）；console.rs 測試綠 |
| W7 token 全收斂 | 完成：CONSOLE_CSS 重寫為 canonical token 名稱＋值（grill-E 裁定；色差映射表見本條目後附）；dark mode 對齊 canonical；護欄測試比對色值 ∈ design-system 調色＋舊詞彙禁殘留 | guards 4/4 綠（含 console_css_tokens_are_canonical_design_system_values） |
| W10 發行鏈現代化 | 完成（配置層）：tauri-plugin-updater 2.0 接線；minisign 金鑰對生成（--ci；私鑰於 %USERPROFILE%\.tauri\flaps-erp\，**repo 外**）；tauri.conf：pubkey/passive/createUpdaterArtifacts/內部 endpoint；NSIS resources 捆綁 knowledge/console（grill-A）；殼編譯綠 | web/src-tauri cargo check EXIT=0；pubkey 152 chars |
| 四關 | 全綠：fmt EXIT=0；clippy -D warnings EXIT=0；test 270 passed/0 failed（platform 153＋kem 43＋flaps_core 58＋manifest 13＋新護欄/矩陣）；audit 2 allowed（audit.toml 載明） | 命令與 exit code 綁本 session 記錄 |
| 提交 | b9797da feat(console)（66 files）＋63beb5a chore(release)（43 files）＋本檔 #20 | git log |

**W7 色差映射表（grill-E 裁定之收斂結果，非視覺回歸）**：--accent #0b5cab→--color-primary #2563eb；--ok #177245→--color-success #16a34a；--warn #9a5b00→--color-warning #d97706；--err #b3261e→--color-error #dc2626；--ink #1b2733→--color-text-primary #0f172a；--muted #5c6b7a→--color-text-secondary #475569；--bg #f4f6f9→--color-surface-sunken #f8fafc；--border #d9dfe7→--color-border #e2e8f0；--chip #eef2f7→--color-neutral-100 #f1f5f9；--radius 10px→--radius-lg 12px。

**W10 保管紀律**：私鑰＋密碼存於 `%USERPROFILE%\.tauri\flaps-erp\`（repo 外；遺失＝無法推送更新）。**所有產物現為 unsigned (test-only)**——購證（Azure Trusted Signing／OV/EV）前禁止生產分發聲明。runbook：`docs/release/UPDATER_RUNBOOK.md`。

**未竟（誠實 UNKNOWN）**：W10 端到端安裝器構建（`cargo tauri build` 含 NSIS＋updater 產物三件）未於本 session 執行——需要 flaps-server release 構建＋數分鐘；endpoint feed 基礎設施（內部 HTTPS 主機）未建。KEM 控制台導覽字串完全由 manifest 生成（頁數/標題已接）——13 工作流 href 重構留待下一切片。

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-10 addendum #25 (RPA-ECOM v2.1 家族調和：主文件轉錄＋摘要版 SUPERSEDED＋全域盤點；renumbered from #23 — numbering collision with 同日 #23 映射/OCR)

Executed per `.kilo/plans/1793787631643-three-studio-slice-execution-plan.md` v8 PHASE-RP（RP-5 家族調和＋RP-6 全域盤點）。擁有者指示本計畫聚焦效能評估進度報告。

| Item | Disposition | Evidence |
|---|---|---|
| RP-5 RPA-ECOM 主文件轉錄 | Dev 根目錄 HTML（863 行，2026-10-27 撰寫、11-03 Gate1 複審修訂）為 RPA-ECOM 線原始專案主文件（charter 級：G1–G6 KPI、OQ-01~08、交付物 DEL-M/D/C/T、WBS 101 人日、甘特至 2027-02-08、RACI、風險 RSK-01~10、ADR-001/002 草案鏈）；已轉錄為 `docs/management/FlapsErp_RPA_電商系統對接_專案文件與開發前準備報告_v2.1.md`（與 .html 成對）；併入摘要版獨有「現況對齊」段（2026-11-09 門檻 246/422ms）為 §0 附錄 | 主文件 .md 478 行寫入 |
| RP-5 摘要版 SUPERSEDED | `docs/management/FlapsErp_RPA_Ecommerce_Project_Plan_v2.1.md` 頭註改標「SUPERSEDED by 主文件（執行狀態已併入其 §0 附錄）」＋**撤回前版錯誤宣稱**（「v2.1 前檔為 0 bytes 空殼，本版首次補實全文」對家族歷史不實——.md 檔 0 bytes 屬實，但家族 v2.1 內容一直存在於根目錄 HTML 863 行） | .md 頭註 SUPERSEDED 修改 |
| RP-5 裁決依據 | 原始主文件在先（10-27）、內容最富（WBS/甘特/RACI/風險/OQ/交付物規格）、被上游引用（逆向分析 §9）→ 家族 SSOT；後建摘要版（11-10，不知情重複）＝降級併入。「文件即程式碼」原則支持 .md 權威＋.html 渲染成對 | PLAN_REGISTER addendum |
| RP-6 全域盤點 | Dev 根目錄未登錄 .md 8+ 份（flaps_erp_performance_analysis.md＝DOC-01、flaps_erp_project_plan.md＝DOC-02、flaps_erp_architecture_optimization_plan.md＝DOC-03、PHASE1_IMPLEMENTATION_README.md、FREE_AI_SYSTEM_UPDATE.md、DOCUMENTATION_MASTER_SPEC.md、WORKTREE_WORKFLOW.md、GEMINI.md、README.md）；Kilo 快照區 ~70 份（含效能家族演化鏈 progress v1→v2.0、install guide v1→v6、deep-verification）；web/legacy-1420.html＋legacy-1420.css（並行線新增，零 JS 適應度待驗證）；docs/research/A1檔案通道端到端操作手冊_v1.0.md（Runbook 236 行→搬入 docs/04-operations/） | RP-6 詳細清單見計畫 §RP.1 |
| 修正 | `docs/management/FlapsErp_RPA_Ecommerce_Project_Plan_v2.1.md` 前版頭註「v2.1 前檔為 0 bytes 空殼」撤回——依 GOVERNANCE P7 誠實紀律 | .md 頭註 SUPERSEDED 修改 |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-10 addendum #26 (供應鏈硬化與 W10 端到端：cargo-deny 四層閘、SBOM、rust-toolchain 釘版、requireSignedVersion、latest.json 產生器與簽章負向用例)

執行切片：計畫 `1793033568961-unified-ops-console-manifest.md` 之 W10 收尾＋供應鏈硬化。所有數字皆附實測方法與日期（GOVERNANCE P7；UNKNOWN ≠ PASS）。

| Item | Disposition | Evidence |
|---|---|---|
| cargo-deny 四層閘 | **完成（PASS）**：`deny.toml` 為政策 SSOT（advisories／bans／licenses／sources）。初次跑出 2 類阻擋：(a) 內部 crate `license = "UNLICENSED"` 非合法 SPDX → 改 `LicenseRef-Proprietary` ＋ `publish = false`（core／platform／web／console_manifest／flaps_bridge／kem）；(b) path 依賴無 version 觸發 `wildcards = deny` → 補 version（kem→console_manifest、platform→flaps_core/console_manifest/kem）。最終輸出 `advisories ok, bans ok, licenses ok, sources ok` | cargo-deny 0.20.2（2026-11-10 實測）；多版本重複僅告警（`multiple-versions = warn`） |
| 公告處置 | 三則逐條書面處置（理由／緩解／複核觸發條件）寫入 `.cargo/audit.toml`（處置 SSOT），deny.toml 僅引用：RUSTSEC-2026-0459（libsqlite3-sys CVE-2026-119375；sqlx-sqlite 0.9.0 限制 `>=0.30.1,<0.38.0` 無範圍內修補版）、RUSTSEC-2024-0436（paste；rav1e 0.8.1←ravif←image←paddleocr_rs_onnx 0.2.8，rav1e 為最新且 paste 為非選用依賴）、RUSTSEC-2026-0192（ttf-parser；lopdf 0.42←pdf-extract／ab_glyph←imageproc，無安全升級路徑） | crates.io 依賴圖實查（2026-11-10）：rav1e 0.8.1 最新、pdf-extract 0.12.1 需 lopdf ^0.42、imageproc 0.27.0 最新 |
| basic-toml 實際根除 | 公告 RUSTSEC-2026-0374（unmaintained，askama_derive 之 `config` 功能）**不採忽略**：`askama` 改為 `default-features = false, features = ["derive","std"]`（repo 內無 askama.toml，行為等價），basic-toml／glob 離開依賴圖 | cargo check --workspace EXIT=0；`cargo deny check` 不再列 basic-toml |
| cargo audit | **PASS**：`Loaded 1446 security advisories`、`Scanning Cargo.lock for vulnerabilities (459 crate dependencies)`，無未處置公告輸出 | cargo-audit 0.22.2（2026-11-10 實測） |
| SBOM | **完成**：CycloneDX 1.5 共 7 份落地 `docs/release/sbom/`（platform／flaps_core／kem／console_manifest／flaps_bridge／web／flaps-erp-web）；內部 crate 授權改 SPDX 後零警告 | cargo-cyclonedx 0.5.9（2026-11-10 實測） |
| rust-toolchain 釘版 | **完成**：根 `rust-toolchain.toml` channel = 1.99.1、components = rustfmt/clippy（元件已安裝，實測無下載需求）；上游 dtolnay/rust-toolchain 於 2026-10-16 納入 1.99.1 patch release | rustc/cargo 1.99.1（2026-11-10 實測） |
| CI 供應鏈閘 | **完成**：`.github/workflows/flexible-ci.yml` 新增 cargo-deny 閘＋CycloneDX SBOM 產出＋artifact 上傳；第三方 action 全部 **commit SHA 釘版**（checkout v4 `11d5960…`、cache v4 `0057852…`、upload-artifact v4 `ea165f8…`、dtolnay/rust-toolchain master `683b588…`）；工具安裝一律 `--locked`；移除全域 `RUSTFLAGS=-D warnings`（避免第三方 crate 警告誤觸失敗） | SHA 取自 GitHub API tag/branch 解析（2026-11-10）；YAML 以 PyYAML 解析驗證（隔離 venv，非 repo 工件） |
| CI 設定檔 | `.github/config/ci-config.yml`：rust_channel → "1.99.1"（與 rust-toolchain.toml 一致）、quality_gates 增 `deny`／`sbom` | 修正原 `target_dir: "$LOCALAPPDATA\Temp\dev-target"`（雙引號內 `\T` 非法跳脫，YAML 解析失敗）→ 單引號 |
| requireSignedVersion | **完成**：`web/src-tauri/tauri.conf.json` 設 `plugins.updater.requireSignedVersion = true`（外掛 2.14.1 支援欄位，camelCase 或 `require-signed-version` 別名皆可）；CSP 同步收緊（移除 `style-src 'self' 'unsafe-inline'`） | tauri-plugin-updater 2.14.1 config.rs：`pub require_signed_version: bool`；tauri-cli 2.12.4 `signer sign --app-version` 說明 |
| W10 端到端（建置＋簽章） | **完成**：首跑失敗（`A public key has been found, but no private key`）→ 根因為 `cargo tauri build` 只讀 `TAURI_SIGNING_PRIVATE_KEY`（**內容**），`TAURI_SIGNING_PRIVATE_KEY_PATH` 僅 `sign` 子命令支援；改正後產出 `Flaps ERP_0.2.0_x64-setup.exe`（5,116,505 bytes）＋`.sig`（440 bytes） | 2026-11-10 實測（release profile 1m23s；NSIS 4.88 MiB 安裝器） |
| latest.json 產生器 | **完成（Rust，零人工編輯）**：新增 `web/src-tauri/src/bin/make_latest_json.rs`——以設定公鑰實際驗簽＋檢查 `version:<v>` 綁定後才寫 feed；簽章格式釐清（`.sig` 檔為 base64(minisign 四行格式)，feed `signature` 欄位即該 base64 字串，不可二次編碼） | `cargo run --bin make_latest_json` 產出 latest.json（version 0.2.0／url 檔名 percent-encode／signature 440 bytes base64） |
| 簽章驗收自動化 | **完成**：新增 `web/src-tauri/tests/updater_artifact.rs`——設定層不變量（requireSignedVersion／pubkey／HTTPS／版本一致）＋正例（產物驗簽）＋三負例（篡改簽章／篡改產物／宣告版本不符）；缺產物時略過，發行流水線可設 `FLAPS_REQUIRE_RELEASE_ARTIFACTS=1` 令其失敗 | `cargo test --manifest-path web/src-tauri/Cargo.toml --test updater_artifact` → 2 passed |
| /console 靜態服務 | 開發版實測：`GET /console/index.html` 200（session 保護，無 session 回登入頁 200×2129 bytes）；`knowledge/console` 65 檔（workflows 13／roles 5／claims 46＋索引）由 NSIS `resources` 捆綁 | 2026-11-10 實測（platform 伺服器 8090） |
| admin_users 導覽一致性 | **完成**：`admin_users.html` 補 `studio_nav` macro ＋ `AdminUsersTemplate.nav` 欄位（由 `NavModel::build(&nav_flags(...), "/admin/users")` 產生，admin 群組依 ROLE_MANAGE 過濾、該項 active） | `cargo check -p platform` EXIT=0 |
| 未竟（誠實 BLOCKED/OPEN） | (1) 安裝版升級演練（安裝 v0.2 → feed 指向 v0.3 → passive 更新 → 重啟版本正確）需 `updates.flaps.internal` HTTPS 主機（未建）＋裝機；(2) Authenticode 需購證；(3) 私鑰密碼生產前輪換；(4) `cargo vet`／`cargo auditable` 未安裝 → 該層 UNKNOWN；(5) `platform/src/com/*` 6 處 Windows COM `unsafe` 與「每檔 forbid(unsafe_code)」憲章衝突（治理缺口，未解） | — |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-11 addendum #32 (PHASE-RP RP-1：效能報告同名消歧——對稱改名＋入冊＋互鏈)

Executed per `.kilo/plans/1793787631643-three-studio-slice-execution-plan.md` v8 PHASE-RP RP-1. Trigger: 同名混淆已實際發生三次（Kilo HTML 誤認、config.ini 文件遍尋不著、重複疑慮）。

| Item | Disposition | Evidence |
|---|---|---|
| 平台線改名 | `docs/research/FlapsErp效能評估進度報告_v2.0.md` → `docs/research/FlapsErp平台線效能評估進度報告_v2.0.md`（Rust 平台線：`platform/`＋`core/`＋`web/`）；頭註加線別聲明＋互鏈 | `git mv` 實證；檔長 10,666 bytes |
| 產品線入冊 | Kilo 快照區 `1793788660000-flaps-erp-performance-progress-report-v2.0.md`（552 行；Electron 產品線 v0.0.4；FLAPS-PERF-PRG-2026-1105-003）複製入工作區 `docs/research/FlapsErp產品線效能進度報告_v2.0.md`；頭註加線別聲明（Kilo 快照區另有 .html 渲染版）＋互鏈 | 檔長 35,198 bytes；Kilo 快照區檔案保留（不搬不刪） |
| 活引用更新 | 2 處同一提交更新：`docs/04-operations/FlapsErp效能優化操作手冊_v1.0.md` L10、`docs/README.md` §2.7；PLAN_REGISTER addendum #17 為點時記錄不回改 | grep 實證 3 處（1 處點時保留） |
| 裁決依據 | 語義漂移定理（計畫 §0.5）：同名異線＝邊界無線別契約 → 對稱命名永久消歧＋互鏈，不合併單檔；改名為雙向門（revert＝改回） | PLAN_REGISTER addendum #32（本行） |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-11 addendum #33 (FlapsErp 產品優化設定與系統最佳設定計畫)

| Item | Disposition | Evidence |
|---|---|---|
| Plan | `.kilo/plans/1794395575776-flaps-erp-product-system-settings.md`（範圍：產品 7 鍵、AP/TLS、Windows 宿主、衛生、外部工作台、監控/回滾；軟體邊界：原廠 FlapsErp vs 自建 Wellfit） | 計畫檔 |
| 執行邊界 | 代理唯讀；IT 套用（RSK-E5 不變）；系統級變更需 IT 核准＋系統級 A/B；不得為 FlapsErp 拖垮其他程式 | 計畫 §0.5／§4.10／§6.9 |
| 實測證據（2026-11-11） | config 7 鍵；AP 預設 `8E2EDBE` HTTP；測試 AP `test2.flaps.tw:10113` CERT-VALID（Let's Encrypt/TLS1.2/2027-01-21）；`FlapsErp.exe` i386 **LAA=True（0x0122）**；module→menu 17.087 s；manifest drift 1/9；softwareId 報告繕寫錯誤已更正；自建 Wellfit MTRIX fixture exit 0／冪等重跑 exit 1（非 FlapsErp 效能） | `Wellfit/artifacts/perf/20261111-*`、`Wellfit/artifacts/erp-batch/20261111-194835/` |
| 工具 | `Wellfit/scripts/perf/flaps-perf.ps1` 擴充 `settings/app/tls/pe/host/endpoints/full`（唯讀；ASCII；無 BOM） | `-Probe settings/app/tls/pe/host/endpoints/full` 實跑 |
| 交付物 | 新增 `FlapsErp最佳設定基線_v1.0.md`、`FlapsErp設定變更runbook_v1.0.md`、`FlapsErp設定AB實測報告_v1.0.md`；更正 `FlapsErp效能優化操作手冊_v1.0.md` §3.2/§3.4、`A1檔案通道端到端操作手冊_v1.0.md`、`FlapsErp效能基線報告_v1.0.md` §7.1/§7.3、`FlapsErp優化_合規風險追蹤表.md` v1.2、`docs/architecture/效能架構.md`（COM 變異）、`FlapsErp產品線效能進度報告_v2.0.md`／`輔翼ERP逆向分析`／`OQ-F1/F2 v2.1`（LAA/HTTP）、`FlapsErp平台線效能評估進度報告_v2.0.md`（原廠/自建標註）；v4.0 校正（NTFS 0/1/2、runbook 補強） | 各檔 |
| 發布判定 | **NOT VERIFIED**（僅唯讀證據；IT 變更/系統級 A/B/8h soak 未執行） | 報告 §L |

This addendum records execution facts only; it does not approve product scope, certification, or release.
## 2026-11-12 addendum #34 (命名與邊界裁定執行＋整合工作台收尾改善)

| Item | Disposition | Evidence |
|---|---|---|
| 命名裁定執行 | 自建系統更名 AuxBridge／整合工作台（bin `auxbridge-server`、crate `auxbridge_core`、env `AUXBRIDGE_*`、DB `auxbridge.db`／`demo-auxbridge.db`、UI 品牌、tauri productName、updater endpoint）；78 檔 205 處、活躍面殘留 0；歷史紀錄/原廠文檔/`flaps_bridge` 依裁定保留舊名 | `docs/standards/NAMING_AND_BOUNDARY_DECREE.md` |
| 防呆測試 | 新增 `platform/tests/naming_guard.rs`（3 tests）：活躍面禁原廠標記＋正向控制＋舊 bin 復活偵測 | `cargo test -p platform --test naming_guard` → 3 passed |
| 安全重驗 | `cross_module_api` merge 先於 `require_session_api` 掛層（api.rs:503→517）——先前層級洞已修；HTTP 實證未登入 401 JSON | 本 addendum 驗證節 |
| 模組健康聚合 | 新增 `GET /api/modules/health`（session 保護；聚合 database/ledger/search/bankpay/status_engine；不洩漏端點與憑證；2 tests 含洩漏負向斷言） | api.rs modules_health |
| Demo 資料修復 | accounts 重複修復（UUID 組 3 筆未引用刪除）、account_type 糾正（REV-4001→REVENUE、EXP-6001→EXPENSES）、雜訊 'test' 批次清除 | sqlite 實測 |
| Demo 資料灌入 | 24 筆 2026-11 交易（16 POSTED／4 PENDING_REVIEW／2 DRAFT／2 VOID）＋48 items（DEBIT=CREDIT=188,678,000 分平衡）＋24 audit rows；十一月 POSTED 營收 NT,103,300 | sqlite 實測 |
| UI 精緻化 | `::selection` 品牌化（ui-v2.css）；`tabular-nums` 已在前輪到位（ui-v2.css:277） | CSS |
| 發布判定 | **PASS**（全閘門：fmt/clippy -D warnings/全測試綠/33 頁掃描/RBAC 負向/區網 200） | 本 addendum 驗證節 |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-12 addendum #35 (整合工作台收尾改善驗證實證)

| Item | Disposition | Evidence |
|---|---|---|
| 驗證工具勘誤 | PS 5.1 Invoke-WebRequest 預設跟隨 303 → 未登入掃描曾假綠 33/33；curl.exe 多行輸出在 PS 為字串陣列（`.Contains` 對陣列＝元素全等→假 False）。HTTP 驗證一律 curl + -o NUL -w %{http_code} 不跟隨 + 內容判準（corrections.md 早有 curl 為準裁定） | 本 addendum；`flaps_session` 假綠修正過程 |
| 供應鏈閘 | `cargo audit` exit 0（1458 advisories × 463 deps）；Tauri 簽章測試 2 passed | cmd EXITCODE=0 實錄 |
| 全測試閘 | `cargo test --locked -p platform` **182 tests / 0 failed**（含 naming_guard 3、modules_health 2、stress p95=406ms<500ms 隔離重跑）；建置鎖依 GAPS_AND_RISKS 以隔離 `CARGO_TARGET_DIR` 迴避 | `C:\Users\STEPHA~1.HUA\AppData\Local\Temp\kilo\test-final.txt` |
| HTTP 終驗 | login 303 PRG；Set-Cookie `auxbridge_session=…; Path=/; HttpOnly; SameSite=Lax`；33/33 頁真綠（帶 cookie、不跟隨）；未登入 /api/audit→401、/→303、偽造 cookie→401；區網 192.168.1.160:8090→200（0.0.0.0 監聽 netstat 實證） | 本 addendum |
| 發布判定 | **PASS** | — |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-12 addendum #36 (FlapsErp 產品優化與系統最佳化：GPU 偏好/優先級已生效＋9 程序量測＋文件治理)

| Item | Disposition | Evidence |
|---|---|---|
| GPU 高效能偏好 | ✅ 已設定（HKCU `UserGpuPreferences` `GpuPreference=2;`）；Chromium 合成走獨顯 | 登錄檔實測 |
| 程序優先級 AboveNormal | ✅ 已設定（全部 8–9 FlapsErp 行程統一 AboveNormal）；重啟後需重設 | `Get-Process FlapsErp` 確認 |
| 8 行程量測 | 8 行程；total 5,913 MB；COM 0.1874/0.2454/0.311 ms；free 4.87 GB；CPU 11.4%；contended=False | `20261112-112150` |
| 9 行程量測 | 9 行程；total 7,756.8 MB；COM 0.1870/0.2325/0.2793 ms；free 4.2 GB；CPU 8.9%；contended=False | `20261112-114836` |
| `app` probe 修正 | 多 session 誤配修正（`moduleToMenu` > 300 s → UNKNOWN）；`moduleToMenu=3,805,923` 假值已壓制 | `20261112-100506`（修正後 15.674 s） |
| MTRIX fixture 重跑 | exit 0（首跑 1985 ms；5 檔/25 列/37 修復）→ exit 1（冪等重跑 22 ms）；非 FlapsErp 效能 | `20261112-115108` |
| Defender 排除 | ✅ **已生效**（4 路徑＋1 程序；管理員提權）；A/B diff 實測 **4 個維度 IMPROVED**：`downloadDoneRelMs` **-77.4%**（1,427→322 ms）、`detailsRelMs` **-11.4%**（1,267→1,122 ms）、`wsMB_median_true` **-74.4%**（18→4.6 MB）、`wsMB_max` **-74.5%**（89→22.7 MB）；host free **6.7 GB**（+0.3 GB） | `120848`/`123402`/`123543`/`defender-exclusions-verified.txt` |
| 文件治理 | Runbook 升級 v2.0（機台無關）；設定基線升級 v2.0（機台無關）；新增局部 Runbook v3.0、系統互動指南 v1.0；v1.0 標 ARCHIVED | `docs/04-operations/` |
| 計畫升級 | 計畫 v4.1：四層架構（L1 細節→L2 局部→L3 全域→L4 擴展）；文件治理 §1.2；設定面全表 §2.1（14 檔案）；跨機台分支 §4.2；不降級原則 §4.3；任務 T-01..T-18 | 計畫檔 |
| 發布判定 | **CONDITIONAL PASS**（GPU 偏好/優先級/Defender 排除已生效；A/B diff 實測 4 個維度 IMPROVED：`downloadDoneRelMs` -77.4%、`detailsRelMs` -11.4%、`wsMB_median_true` -74.4%、`wsMB_max` -74.5%；`=0` 新鮮登入後 startup/系統級 A/B/8h soak 待完成） | A/B 報告 §L |

This addendum records execution facts only; it does not approve product scope, certification, or release.

## 2026-11-12 addendum #36 (規格補洞批次：可觀測性、安全硬化、資料可靠性、效能)

| Item | Disposition | Evidence |
|---|---|---|
| 可觀測性 | 新增零依賴日誌模組 `platform/src/obs.rs`（stderr＋AUXBRIDGE_LOG 等級）；移除無 subscriber 之 `tracing` 相依（5 處改用 `obs_info!`） | 執行輸出 `INFO obs: logger initialised`；套件 143/0 |
| CSRF 稽核 | 27 條 POST 路由逐一路由稽核：25 條有檢查；`/login` 無 session 前豁免（設計上不可能有 token）；`transition_handler` 為純狀態驗證無寫入 → 不適用 | `tools` 稽核腳本（一次性）＋ handler 審查 |
| 稽核脈絡 | migration 0009 加 `ip_address`/`user_agent`＋索引；LOGIN_SUCCEEDED/LOGIN_FAILED/LOGOUT 寫入脈絡 | 備份快照 PRAGMA table_info 實證 |
| 安全標頭 | HSTS 全回應送出；公開路由 body 上限 64KiB | curl 實測 HSTS；router 掛載審查（65KiB 負向未測） |
| 輸入上限 | /search 查詢 200 字元上限 | `search_caps_overlong_query_without_error`（5000→200） |
| 密碼政策 | NIST SP 800-63B 對齊（≥12＋阻擋清單＋不得同帳號；無組成規則） | `password_policy_*` 測試 |
| 資料可靠性 | 備份 CLI（VACUUM INTO）；遷移版本防呆測試 | CLI 實測 27/4/40；`migration_versions_*` 測試 |
| 資料庫硬化 | `connect_options`：FK=ON、WAL、synchronous=NORMAL、busy_timeout、mmap；唯讀連線跳過寫入型 pragma | `connect_options_enforces_foreign_keys_and_wal`、`vacuum_into_*` |
| 效能 | 嵌入 2s 逾時＋斷路器；批量索引（消除 O(n²)）；套件 150s→27s | `search_finds_*` 46ms |
| 發布判定 | **PASS**（fmt/clippy/143 tests/34 頁掃描/區網/備份實證）；**PARTIAL**：65KiB 負向未測、還原演練未歸檔 | 本 addendum |

This addendum records execution facts only; it does not approve product scope, certification, or release.
