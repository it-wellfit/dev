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
