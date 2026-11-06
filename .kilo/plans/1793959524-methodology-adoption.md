# Development Methodology Adoption Plan

Scope: Dev root workspace (Flaps ERP / ekpi-platform root workspace: core/, platform/, web/, crates/flaps_bridge).
Current state: 98 tests passed / 0 failed; fmt/clippy/test/audit all EXIT=0; zero-JS invariant (script=0, inline-on=0) verified; audit has 1 pre-accepted warning (RUSTSEC-2026-0374).
Concurrent work: governance-hub (Phase D3) and moduleization submodules (db/, sync/, search/, com/) are being edited in parallel; this plan avoids touching api.rs directly to prevent conflicts.

## Decisions (made)

- SKILLS adopted only where they reduce execution risk: agent-md-refactor for restructuring AGENTS.md/docs when needed; microsoft-code-reference only if touching Microsoft SDKs (not applicable now); skill-creator not needed now.
- PBT (proptest) adopted ONLY if already in Cargo.lock; otherwise new dependency requires explicit approval before adding.
- CI/CD workflow exists but is unexecuted (per NEXT_ACTIONS): enable/run it and capture results.
- Deferred (not in scope): FM/Kani (codebase forbids unsafe_code, low value), chaos engineering, digital twin, MBSE/MDD/MDA, mutation testing unless CI proves stable first.
- DDD gap to fill: POSTED to PENDING_REVIEW state-transition test in core ledger engine.
- BDD gap to fill: end-to-end journey scenario login to post transaction to paginate to logout with binding test.
- TDD evidence gap: strengthen RED to GREEN documentation for completed features (M1/M2/M3).

## Ordered task list

1. Inventory Cargo.lock: check whether proptest / cargo-fuzz / cargo-mutant are already present or available. Decide adopt-or-reject for PBT money-arithmetic property tests (round-trip parse/format, non-negativity). Record decision in docs/engineering/DEVELOPMENT_METHODOLOGY.md.
2. Run/enable the existing GitHub Actions CI workflow (unexecuted per NEXT_ACTIONS); capture pass/fail evidence in PROJECT_STATUS.md.
3. Strengthen TDD evidence: rewrite CHANGELOG M1/M2/M3 entries to explicitly show RED to GREEN (test name that failed first, then passing count).
4. Confirm BDD scenario to test bindings all point to existing passing tests (docs/testing/BDD_SCENARIOS.feature); add missing end-to-end journey scenario + its binding test if a gap remains.
5. Add DDD state-transition test: POSTED to PENDING_REVIEW in core/src/ledger/engine.rs.
6. ADD gap: record CI/CD execution result and SRE/SLO decision (skip SLO for single-node SQLite; document why).
7. Run full four gates (fmt/clippy/test/audit) after changes; verify 0 failures; update PROJECT_STATUS.md and NEXT_ACTIONS.md.

## Open questions (need user answer before proceeding)

- Approve adding proptest as a dependency for PBT money tests? (Reject = keep property testing out of scope.)
- Approve running/enabling the GitHub Actions CI workflow now?

## Risks

- Concurrent edits to api.rs by another workstream: this plan deliberately avoids editing api.rs; only core/ engine and docs/tests are touched.
- Adding dependencies changes Cargo.lock and requires CI re-run; do not add without approval.

## Validation (completion criteria)

- cargo fmt -- --check EXIT=0
- cargo clippy --all-targets -- -D warnings EXIT=0
- cargo test --locked: 0 failed
- cargo audit: only pre-accepted warnings
- BDD feature file: every Scenario bound to an existing passing test name
- New PBT/state-transition tests pass before plan_exit
