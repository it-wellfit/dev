# Plan: EKPI Platform — Product Excellence Engineering (A–J Framework)

> **Disposition (2026-11-02): Historical proposal, not an approved current roadmap.** This file is stored in the workspace-level `.kilo/plans/` but describes EKPI. Its 2026-10-27 verification snapshot predates current uncommitted changes in `ekpi-platform/`; revalidate before relying on any status claim. Canonical project context: [EKPI charter](../../ekpi-platform/PROJECT_CHARTER.md).

**Plan file:** `.kilo/plans/1792914505665-ekpi-platform-product-excellence.md`

**Framework:** PRODUCT EXCELLENCE ENGINE (A–J): Product Discovery → Feature Deepening → UX → System Engineering → Documentation → Implementation & Verification → Product Evaluation → Iteration → Final Completion Standard → Immediate Execution.

**Context:** The EKPI platform is a tested local deterministic prototype (Rust, two independent crates `eval/` and `platform/`). Phase A (0→1 MVP) is **not approved**; Phase B (production-grade) is **blocked**. Per AGENTS.md and phase-gates.md, this repository must not be described as production-ready, secure, tenant-isolated, compliant or validated without linked evidence.

## Current verified state (2026-10-27)

| Dimension | Status | Evidence |
|-----------|--------|----------|
| Code compiles | `PASS` | `cargo build --locked` in both crates |
| Clippy (-D warnings) | `PASS`, zero warnings | `cargo clippy --locked --all-targets -- -D warnings` |
| Format | `PASS` | `cargo fmt -- --check` both crates |
| Tests compile | `PASS` | `cargo test --locked --no-run` both crates |
| Platform unit tests | 242 PASS | `cargo test --release --locked` |
| CLI selftest | 49 assertions PASS | `--selftest` on release build |
| Declared-rules schema | 20 records clean | `--declared-rules-schema` |
| Search evaluation | 28 cases: 20/20 top-1, 8/8 refusal, 0 violations | `--eval --test-set data/eval/search_eval.json` |
| Backup/restore drill | PASS (multiple revisions) | `scripts/backup.ps1`, fresh clone + `check.ps1` exit 0 |
| Phase A gate | `Not approved` | A1 `Not met`; A2/A3/A5/A6/A7/A10/A11 `Partial` |
| Phase B gate | `Blocked` | B1/B4/B5/B6/B8 blocked on owners/decisions |

---

## Section A — Product Discovery (user problem → spec)

### A-01: Select pilot workflow, owner, and data (`DEC-01`, gate A1 `Not met`)
- **Skills:** `/grill-with-docs`, `/research`, `/to-spec`
- **Spec:** Choose one representative workflow from MVP scope (governance/product/README.md §4.3): a single project manager creates a project, links one evidence item with a versioned clause anchor, records one gate decision, and replays the snapshot. One named product owner. One authorized non-sensitive synthetic dataset (already exists in `platform/data/`).
- **Artifact:** `docs/governance/09-pilot-workflow.md` (scope, owner, dataset, success criteria, expected verdict per case).
- **Definition of done:** Owner named in governance decision register; dataset path recorded; success criteria written; A1 row moves from `Not met` to `Partial` (requires real execution for `Met`).
- **Verification:** Reviewer confirms owner and dataset are real and authorized; no secrets or real customer data in repo.

### A-02: Complete acceptance-test matrix for pilot scope (gate A2 `Partial`)
- **Skills:** `/to-spec`, `/to-tickets`, `/tdd`
- **Spec:** Map the 67-case corpus (9 seed + 46 casebook + 12 edge) and the 28-case retrieval evaluation set to acceptance IDs AT-01–AT-31. Each case must state: expected verdict, expected citation set, abstain semantics, hard-gate expectation (H-1..H-6).
- **Artifact:** Updated `docs/governance/04-acceptance-test-matrix.md` with per-case expected results and traceability IDs.
- **Definition of done:** Every in-scope case has an AT ID, expected verdict, citation expectation, and hard-gate mapping; synthetic test results linked per case.
- **Verification:** Traceability register rows reference AT IDs; no case lacks an expected verdict.

### A-03: Real-workflow dataset and independent review (gate A3 `Partial`)
- **Skills:** `/to-spec`, `/diagnosing-bugs`, `/code-review`
- **Spec:** Curate one pilot workflow dataset (not the synthetic corpus) representing a real governed change: project, evidence link with doc@version§section, 29-item quality checklist, one gate decision. Independent reviewer verifies the engine verdict matches the manual determination.
- **Artifact:** `docs/governance/pilot-workflow-dataset.jsonl` (synthetic, no real customer data); `docs/reviews/pilot-independent-review.md`.
- **Definition of done:** Dataset loads without shape errors; engine decides each case; independent reviewer signs the review record; discrepancies logged as defects.
- **Verification:** `cargo test --locked` passes; reviewer signs off; defects have linked tickets.

### A-04: Open decisions registered (`DEC-01` through `DEC-20`)
- **Skills:** `/grill-with-docs`, `/wayfinder`
- **Spec:** Record every open product decision in `docs/governance/08-decision-register.md`: pilot scope, roles, risk classification, data policy, identity, retention, service levels, AI use. Each decision has an accountable role (named or explicitly "unassigned — blocker").
- **Artifact:** Updated decision register with all open DEC items and owners.
- **Definition of done:** Every open DEC item has an ID, question, options considered, current status, accountable role, and review date. No placeholder names.
- **Verification:** Cross-check against phase-gates.md blocking decisions list.

---

## Section B — Feature Deepening (complete, handle edge cases)

### B-01: Governance web — normal use flows (J1–J3)
- **Skills:** `/to-spec`, `/implement`, `/tdd`
- **Spec:** Three journeys from product brief:
  - J1: Adjudicate a case via CLI (`--eval --test-set`) — already works; add a `--pilot-workflow` flag that runs a single case end-to-end and prints verdict/citations/snapshot-hash.
  - J2: Link and review evidence via `/evidence` and `/evidence/{id}/review` — exists; add form validation (non-empty title/source/actor), flash messages on success/failure, and preserved form data on validation error.
  - J3: Record gate/quality decision via `/lifecycle` and `/controls` — exists; add role-gating (Editor/approver only for publish/gate), preview-before-submit, and cancellation.
- **Artifact:** Updated `platform/src/governance_web.rs` handlers + templates; `platform/tests/governance_web.rs` new edge-case tests.
- **Definition of done:** All three journeys complete end-to-end in a manual walkthrough; validation errors retain input; success/failure feedback is visible; unauthorized roles receive 403 with a plain-language message.
- **Verification:** Governance web unit tests pass; clippy/fmt clean; manual walkthrough recorded in `docs/reviews/`.

### B-02: Edge cases — empty data, invalid input, permissions, concurrency
- **Skills:** `/to-tickets`, `/tdd`, `/diagnosing-bugs`
- **Spec:**
  - Empty docs/rules/processes: CLI returns a clear "no data" message, not a stack trace.
  - Invalid form input: server returns 400 with field-level error messages; input retained.
  - Permission denied: 403 with plain-language message, not a stack trace.
  - Expired session: redirect to `/login` with "session expired" message.
  - Concurrent requests: semaphore bound (already 64); rate-limit returns 429 with Retry-After.
- **Artifact:** New edge-case tests in `platform/tests/governance_web.rs` and `platform/tests/edge_cases.rs`; updated handler code.
- **Definition of done:** Every edge case returns a user-understandable response; no unhandled panics; no stack traces in UI.
- **Verification:** `cargo test --locked` passes; clippy/fmt clean.

### B-03: Recovery flows — logout, session expiry, retry
- **Skills:** `/to-spec`, `/implement`
- **Spec:**
  - Logout clears session cookie and redirects to `/login`.
  - Expired session on any mutating route returns 401 + redirect to `/login` with `session_expired` param.
  - Retry-after on 429/503 includes `Retry-After` header.
- **Artifact:** Updated `logout_post`, `check_session`/`require_session` middleware, `request_context` timeout handling.
- **Definition of done:** Logout invalidates session; expired session test passes; 429 includes Retry-After.
- **Verification:** Existing logout/expired-session tests pass; new rate-limit test added.

---

## Section C — User Experience (clear, fluent, trustworthy)

### C-01: CSS-first UI polish (zero JavaScript)
- **Skills:** `/to-spec`, `/implement`
- **Spec:** Existing `governance.css` is included via `include_str!`. Improve:
  - Clear page titles and breadcrumb navigation on every page.
  - Form labels with associated inputs; required fields marked.
  - Success/error/info banners with distinct colors.
  - Empty states show a clear next action (e.g., "No projects yet — create your first project").
  - Keyboard-navigable forms and buttons; focus styles visible.
  - No `<script>` tags; no inline JS event handlers.
- **Artifact:** Updated `platform/src/assets/governance.css` and `platform/src/bin/governance_web.rs` templates.
- **Definition of done:** Zero `<script>` tags; every page has a title and navigation; empty states guide the user; forms validate client-side (HTML5 attributes) and server-side.
- **Verification:** Manual review; `scripts/check-docs.sh` not affected; clippy/fmt clean.

### C-02: Error messages are actionable
- **Skills:** `/to-spec`, `/implement`, `/diagnosing-bugs`
- **Spec:** Audit all user-facing error strings in CLI and web:
  - Replace technical messages ("serde_json::Error", "mutex poisoned") with plain-language messages ("could not read the data file", "server temporarily unavailable").
  - Every error includes what happened and what to do next.
  - No stack traces leaked to the UI or CLI output (except `--selftest` debug mode, which is intentional).
- **Artifact:** Updated error strings across `main.rs`, `governance_web.rs`, `engine.rs`, `docs.rs`, `rules.rs`, `process.rs`.
- **Definition of done:** No technical exception text reaches the user; every error has an action; error strings are tested where feasible.
- **Verification:** Manual audit; clippy/fmt clean.

---

## Section D — System Engineering (contracts, boundaries, verification)

### D-01: Document CLI/web/data contracts
- **Skills:** `/to-spec`, `/code-review`
- **Spec:** Update `docs/contracts/README.md` with:
  - CLI flags: accepted values, dependencies (e.g., `--test-set` requires `--eval`), exit codes.
  - Web routes: methods, auth requirements, request/response shapes, error codes.
  - Data formats: `docs.json`, `rules.json`, `processes.json`, `cases/*.jsonl` schema.
  - LegalRuleML export shape.
- **Artifact:** `docs/contracts/README.md` updated; contract test expectations documented.
- **Definition of done:** Every consumer-visible interface has a documented contract; no undocumented behavior is relied upon by tests.
- **Verification:** `scripts/check-docs.sh` passes; contracts cross-referenced from traceability register.

### D-02: AuthZ boundary — local only, no enterprise claims
- **Skills:** `/to-spec`, `/security-review`
- **Spec:** `local_auth` is wired into `governance_web` (session cookie, PBKDF2-HMAC-SHA256 600k, CSPRNG tokens, lockout after 5 failures, role checks Viewer/Editor/Approver/Admin). Explicitly document:
  - Not an IdP, SSO, MFA, account recovery, or tenant isolation.
  - Sessions live in process memory only; restart clears them.
  - No default account; credentials come from `EKPI_LOCAL_AUTH_USER/PASSWORD` env vars only.
- **Artifact:** `docs/architecture/threat-model.md` updated; `AGENTS.md` auth boundary confirmed accurate.
- **Definition of done:** No claim of enterprise authentication appears in code or docs; threat model documents the boundary clearly.
- **Verification:** Manual audit of docs and code comments; no "enterprise auth" claims found via search.

### D-03: Observability — metrics, logs, health
- **Skills:** `/to-spec`, `/implement`
- **Spec:** Existing `/metrics` (Prometheus text), `/healthz` (liveness), JSON access log, correlation IDs, request duration, in-flight gauge, rate-limit counter. Add:
  - Readiness probe (`/ready`) distinct from liveness.
  - Decision latency histogram (buckets) in addition to average.
  - Verdict distribution counter (comply/violate/insufficient_evidence/escalate).
  - Auth events counter (login success/failure, logout).
- **Artifact:** Updated `governance_web.rs` metrics endpoint; new metrics tests.
- **Definition of done:** `/ready` returns 200 when the ledger is open and auth store is initialized; verdict distribution counter increments per decision; auth counter increments per login attempt.
- **Verification:** `cargo test --locked` passes; metrics endpoint returns valid Prometheus text.

---

## Section E — Documentation (traceable, current, consistent)

### E-01: Traceability register — complete pilot scope rows
- **Skills:** `/to-spec`, `/implement-spec`
- **Spec:** Every accepted requirement in the pilot scope gets a row in `docs/traceability/register.md`: requirement ID → spec → code change(s) → test evidence → PR/decision. Distinguish verified (synthetic test) vs validated (real user).
- **Artifact:** Updated `docs/traceability/register.md` with all pilot-scope rows.
- **Definition of done:** All A1–A11 in-scope rows have code+test+evidence links; no unverified claim is marked `Met`.
- **Verification:** Cross-check against phase-gates.md; traceability script (`scripts/check-docs.sh` link check) passes.

### E-02: Close stale operational claims
- **Skills:** `/implement-spec`, `/code-review`
- **Spec:** Audit `docs/operations/` for stale claims (e.g., dated final-validation-report.md). Update `docs/operations/production-readiness.md` to reflect actual state (not production ready). Add `docs/operations/handoff-dry-run.md` recording a fresh-clone install/configure/diagnose walkthrough.
- **Artifact:** Updated ops docs; new handoff-dry-run record.
- **Definition of done:** No false production claims remain; a second operator can run the full local check from docs alone.
- **Verification:** `scripts/check-docs.sh` passes; manual dry run recorded.

### E-03: Retrospective and improvement register
- **Skills:** `/retro`, `/improve-codebase-architecture`
- **Spec:** Complete a retro on the current state; record accepted improvements in `docs/reviews/improvement-register.md` with owner and due date.
- **Artifact:** Retro report + improvement register entries.
- **Definition of done:** Each improvement has an owner and a linked ticket/issue; no improvement is ownerless.
- **Verification:** Retro report exists; register rows are complete.

---

## Section F — Implementation & Verification (TDD, checks, review)

### F-01: Fixed local check scripts
- **Skills:** `/implement-spec`
- **Spec:** `scripts/check.ps1` and `scripts/check.sh` are the fixed entry point. They must run: docs link check, secret scan, supply-chain check, fmt, clippy (`-D warnings`), tests, selftest, schema validation. Each check names command, revision, date, result.
- **Artifact:** Both scripts updated; `gate.ps1` with `-IncludeSmoke` for report/export/coverage/HTML changes.
- **Definition of done:** Scripts run end-to-end on a fresh clone; every check exits 0 or records `Blocked` with reason; no check is silently skipped.
- **Verification:** Manual run on clean clone; output captured as evidence.

### F-02: Test every new slice before implementation (TDD)
- **Skills:** `/tdd`
- **Spec:** For every new behavior: write a failing test first, implement the minimum to pass, rerun affected checks, review, rerun full baseline.
- **Artifact:** New tests accompany every new feature; no feature ships without a test.
- **Definition of done:** No committed code lacks test coverage for its primary behavior; clippy/fmt clean on every commit.
- **Verification:** `cargo test --locked --no-run` compiles all tests; clippy zero warnings.

---

## Section G — Product Evaluation (measurable value)

### G-01: Define success metrics with baselines
- **Skills:** `/to-spec`, `/research`
- **Spec:** Update `docs/metrics/targets.md` with:
  - Task completion rate (cases decided correctly / total cases).
  - Error rate (4xx/5xx responses / total requests).
  - Manual handoff rate (escalate verdicts / total cases).
  - Processing time per case (p50/p99 latency from `/metrics`).
  - Completion rate (cases reaching terminal verdict / total cases).
  - Each metric has: definition, measurement method, target value (locked/proposed/deferred), owner, review date.
- **Artifact:** `docs/metrics/targets.md` with locked targets (H-1..H-6 are locked at 0 wrongful-allow, 0 released violations, 100% completion).
- **Definition of done:** Every public number in reports is derivable from `engine_snapshots.json` or `engine_hard_gates.json`; no invented numbers.
- **Verification:** Traceability register links each metric to its measurement command.

### G-02: Honest about unvalidated claims
- **Skills:** `/to-spec`, `/code-review`
- **Spec:** Any claim about user value, compliance, security, or performance must name the evidence. Where evidence is absent (real-user pilot, enterprise auth, off-host backup), the document states `Not met` or `Blocked` with a blocker reason. No placeholder owners.
- **Artifact:** All docs audited for unvalidated claims; phase-gates.md remains the source of truth.
- **Definition of done:** No document claims production readiness, compliance, or certification without linked evidence; `AGENTS.md` warning is current.
- **Verification:** `grep` for "production-ready", "compliant", "certified", "secure" outside AGENTS.md caveats returns only documented caveats.

---

## Section H — Iteration Strategy (improve the most important problem)

### H-01: Retro-driven improvement backlog
- **Skills:** `/retro`, `/improve-codebase-architecture`
- **Spec:** After each work slice, run a retro. Identify: hardest task, largest defect, most error-prone operation, trust/safety/data issue, performance bottleneck, documentation friction. Convert to improvement tickets with owner and due date.
- **Artifact:** `docs/reviews/improvement-register.md` entries.
- **Definition of done:** Each improvement is traceable to a retro finding; no improvement is ownerless.
- **Verification:** Register reviewed at each retro; stale items are closed or re-scoped.

---

## Section I — Final Completion Standard

**Every feature must satisfy all of the following before it is considered complete:**

1. **Clear user value** — names who benefits and what problem is solved.
2. **Requirements + acceptance criteria defined** — linked in traceability register.
3. **Normal flow operable** — demonstrated end-to-end (CLI or web).
4. **Important failure cases handled** — empty data, invalid input, permission denied, expired session, concurrency limit, timeout.
5. **Frontend/backend/data contracts consistent** — documented in `docs/contracts/`.
6. **Tests actually executed** — `cargo test --locked` compiles; clippy/fmt clean; results recorded with revision/date.
7. **Review findings fixed or recorded** — code review findings addressed or explicitly deferred with owner.
8. **Documentation consistent with code** — no stale claims; `scripts/check-docs.sh` passes.
9. **Security/performance/accessibility checks** — threat model boundary respected; metrics/health endpoints present; zero `<script>` tags; keyboard-navigable forms.
10. **Known limits disclosed** — phase-gates.md status is accurate; no feature is described beyond its verified scope.

**Product version completion** additionally requires: deployment, backup, monitoring, recovery and rollback conditions met per phase-gates.md Phase B rows.

---

## Section J — Immediate Execution (next 3 slices)

### Slice 1: Pilot workflow selection + traceability (A-01, A-02, E-01)
- Select pilot workflow, owner, dataset.
- Complete acceptance-test matrix for pilot scope.
- Complete traceability register rows for pilot scope.
- **Evidence:** `docs/governance/09-pilot-workflow.md`, updated acceptance matrix, updated traceability register.

### Slice 2: Governance web edge cases + UX polish (B-01, B-02, C-01, C-02)
- Form validation, input retention, flash messages.
- Empty states with next actions.
- Error message audit (no technical leakage).
- Role-based 403, expired-session redirect, rate-limit Retry-After.
- **Evidence:** Updated handlers/templates; new tests; clippy/fmt/test pass.

### Slice 3: Observability + contracts + stale-claims closure (D-01, D-03, E-02, E-03)
- `/ready` endpoint, verdict distribution, auth counters.
- Contracts documentation updated.
- Stale claims audited and corrected; handoff dry run recorded.
- Retro report + improvement register.
- **Evidence:** Updated `governance_web.rs` metrics; `docs/contracts/README.md`; `docs/operations/handoff-dry-run.md`; retro report.

**After Section J:** Phase A gate rows A2/A3/A5/A6/A7/A10 move toward `Met`; A1 remains `Not met` until a real operator executes the pilot. Phase B remains `Blocked` until external owners/decisions exist.

---

## Constraints (non-negotiable)

- `#![forbid(unsafe_code)]` on every Rust file; clippy `-D warnings` zero warnings; `cargo fmt -- --check` clean. Edition 2021.
- PowerShell boundary: multiple commands stitched with `;`, never `&&`/`|`/`&` in a single shell line.
- UTF-8 only for file writes; never PowerShell `Set-Content` without `-Encoding UTF8` (use Write tool / Python).
- No secrets, credentials, tokens, or real customer/org facts in the repo. `.env.example` only.
- No fabricated integrations, permissions, approvals, scan results, or compliance claims. Governance web prototype is local-only; no enterprise authentication.
- Library-only modules (`tenant`, `audit`, `worm`, `agents`, `formal_verification`, `runtime_verification`) must not be described as product capabilities until wired with spec + integration tests.
- A check result is evidence only when it names command, revision, date, and result.

## Open decisions (require external input — not plan-agent)

- Product owner name (`DEC-01`); security owner (`DEC-12`); operations owner (`DEC-16`); data owner (`DEC-13`); deployment target (`PR-02`).
- Single-tenant vs multi-tenant decision (`DEC-12`); off-host backup location/RPO/RTO (`DEC-14`); AI in-scope or out-of-scope (`DEC-15`); compliance scope (`DEC-09`).
- Remote collaboration upgrade only when criteria in `docs/remote-collaboration-criteria.md` are all met.

## Out of scope for this plan

- Enterprise SSO/MFA (IdP not approved).
- Live Microsoft 365/Google connectors; Kubernetes; message queue; relational DB; public network exposure; 24×7 support; regulatory certification.
- AI approval or autonomous gate release.
- Converting `local_auth` into an enterprise identity system.
