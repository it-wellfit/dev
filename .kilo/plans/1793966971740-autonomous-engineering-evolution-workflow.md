# Autonomous Engineering Evolution Workflow — Master Plan

**Plan file:** `.kilo/plans/1793966971740-autonomous-engineering-evolution-workflow.md`
**Date:** 2026-11-06
**Status:** Implementation-ready (all open questions resolved; ready for an implementation-capable agent)
**Scope:** Dev root workspace `C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev` (all subprojects)

---

## 1. Goal

Establish a repeatable autonomous development evolution workflow combining:
- **Loop Engineering** (recurrence: Goal → Action → Observation → Adjustment, bounded stop condition)
- **Graph Engineering** (topology: nodes = bounded units, edges = data dependencies, independent verifier)
- **Kitchen Loop** (safe self-evolution: spec surface + unbeatable tests + drift control + pause gate)

All using **zero-cost/free** AI resources only. No source-code changes by this agent (plan only).

---

## 2. Constraints (non-negotiable)

| Constraint | Source |
|------------|--------|
| `#![forbid(unsafe_code)]` on every `.rs` first line | Workspace AGENTS.md §2.2 |
| `cargo clippy --all-targets -- -D warnings` EXIT=0 (no `allow`) | Workspace AGENTS.md §2.2 |
| `cargo fmt -- --check` EXIT=0 | Workspace AGENTS.md §2.2 |
| `cargo test --locked`: 0 failed / 0 ignored | Workspace AGENTS.md §2 |
| Zero-JS: `<script>` count = 0, no inline `on*=` | Workspace AGENTS.md §2.3 |
| Integer amounts only (no f64 for money) | SHARED_STANDARDS §1.1 |
| GET read / POST write / PRG after success | Project constraint |
| Evidence rule: PASS only if exact command ran on named revision and exited 0 | GOVERNANCE §2.5 |
| ekpi-platform: Phase A **not approved**, Phase B **blocked**, not production-ready | PROJECT_CHARTER §8 |
| Plan Mode: no source edits, no mutating commands | Plan constraint |

---

## 3. Affected Boundaries

- **In scope:** `.kilo/sessions/`, `.kilo/plans/`, `ekpi-platform/` (evidence collection only, no deployment), `core/`/`platform/`/`web/` (baseline validation), `Wellfit/` (review unified plan v2), free AI endpoint smoke tests.
- **Out of scope:** Production deployment, TLS/hosting decisions, Git remote connection without authorization, `api.rs` production implementation (design only), any source-code edits.
- **Single-writer rule:** `platform/src/api.rs` and `web/*` — one writer at a time (Flaps plan §7).

---

## 4. Data Flow (workflow graph)

```
TRIGGER (schedule or event: issue, failing build, user request)
    ↓
PLAN (agent.md + program.md: bounded tasks, spec surface)
    ↓
GRAPH DECOMPOS (nodes = bounded units; edges = data dependencies)
    ↓
EXECUTE (isolated worktree; small model; read-only default)
    ↓
VERIFY (independent verifier ≠ implementer; tests read-only; clippy/fmt)
    ├─ fail → bounded repair loop (max N attempts) → back to EXECUTE
    └─ pass → HUMAN GATE (consequential actions: merge, deploy, state change)
    ↓ (approve)
COMMIT + ARTIFACT (evidence: command + exit code + result; UNKNOWN≠PASS)
    ↓
LOOP CLOSE (record trace to session manifest; update specs/knowledge; drift control)
    ↓ (drift detected → PAUSE gate; escalate to human)
```

---

## 5. Failure Modes & Mitigations

| Failure mode | Mitigation |
|--------------|------------|
| Token exhaustion on free tiers | Per-loop budget; kill switch; small model for verify/guard; local Ollama primary |
| Rate limits (20 RPM / 200/day Kilo Gateway) | Batch independent graph nodes; cache results; rotate across free providers |
| Single-writer conflicts on `api.rs` / `web/*` | S0 determines R1 ownership; coordinate via session manifest |
| Template/encoding regression (UTF-8, NUL, `<script>`) | R5 guards test; scan after every template edit |
| Windows app-control blocking builds | `CARGO_TARGET_DIR=%LOCALAPPDATA%\Temp\dev-target`; `;` chaining (no `&&`) |
| No Node → Lighthouse unavailable | Playwright/axe or manual checklist + honest "alternative" marking |

---

## 6. Rollout / Migration Path (phases)

| Phase | Task | Exit condition |
|-------|------|----------------|
| A | Validate baseline: four gates (fmt/clippy/test/audit) in `ekpi-platform/` each crate + root workspace; record exit codes | All EXIT=0 or blocked documented |
| A | Confirm Git status: remotes, dirty state, `git log --oneline -10` | Status recorded |
| A | Create session `.kilo/sessions/<YYYYMMDD>-autonomous-evolution-init/` with `manifest.md` | Manifest written |
| B | Smoke-test free AI endpoints reachable from this machine (Kilo Gateway nemotron-3.5-super-vl:free, Cloudflare Workers AI, OpenRouter :free, Z.ai GLM-4.5-Flash, local Ollama) | Reachability recorded |
| B | Draft `docs/engineering/FREE_AI_ROUTING.md`: providers, limits, routing rules, fallback chain, per-loop token budget | Doc published |
| C | Read ekpi-platform `docs/quality/phase-gates.md` + `docs/assurance/README.md`; list exact evidence for Phase A approval | Evidence list |
| C | Design production API contract for `api.rs` (read governance spec first; design only, no impl) | Contract design |
| C | Enable/run GitHub Actions CI workflow; capture pass/fail | CI result recorded |
| D | Execute Flaps ERP S0 (baseline reconciliation + freeze baseline) per `1793033512795-flaps-erp-workbench-completion.md` | S0 baseline log |
| D | Execute S1–S7 in order; each slice: four gates full green + evidence before advancing | PA-01..08 |
| E | Resolve 2 open questions in `1793959524-methodology-adoption.md` (proptest, CI) or mark out of scope | Questions resolved |
| E | Execute its 6 ordered tasks | Tasks complete |
| F | Execute desktop cleanup `1793965687748-desktop-cleanup-plan.md` steps 1–5 | Desktop organized |
| G | Review Wellfit `1793782615513-wellfit-unified-plan-v2.md` requirements table; add unmet items to backlog | Backlog updated |
| H | Recurring session cadence: run workflow graph against highest-priority backlog item; record trace; pause on drift | Ongoing |

---

## 7. Validation / Completion Criteria

- Session manifest created; inventory recorded as artifacts under `<project>/artifacts/<session-id>/`.
- Four gates EXIT=0 on current baseline (or blocked explicitly documented, never fabricated).
- Free AI routing doc published with verified reachable endpoints and fallback chain.
- Flaps ERP S0 baseline log produced; gap list confirmed against plan §1.3.
- All open questions answered by user or explicitly marked out of scope.
- `.kilo/sessions/index.md` updated at session end.

---

## 8. Open Questions (need user decision — one at a time)

**Q-1:** Approve using free remote LLM endpoints (Kilo Gateway, Cloudflare Workers AI, OpenRouter :free, Z.ai GLM-4.5-Flash) plus local Ollama for the autonomous evolution workflow? → **RESOLVED: Approved** (zero cost, redundant fallback chain, no vendor lock-in).

**Q-2:** Approve creating sessions under `.kilo/sessions/` and artifacts under `<project>/artifacts/<session-id>/` for ongoing autonomous work? → **RESOLVED: Approved** (matches existing convention).

**Q-3 (current):** Approve proceeding with ekpi-platform Phase A evidence collection and API contract design (not production deployment)? → **RESOLVED: Approved** (evidence toward approval only; no deployment).

**Q-4:** Should the implementation agent run `cargo test --locked --workspace` and the four gates (fmt/clippy/test/audit) to confirm baseline before starting Phase B? → **RESOLVED: Yes** (establishes current truth before any evolution work).

---

## 9. Status

All open questions resolved. Plan is implementation-ready.

**Plan location:** `C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\.kilo\plans\1793966971740-autonomous-engineering-evolution-workflow.md`

This plan is implementation-ready. An implementation-capable agent should execute Phase A onward, recording evidence (command + exit code + result) for every check, and updating session manifests at the end of each session.