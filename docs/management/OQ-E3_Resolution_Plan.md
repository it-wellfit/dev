# OQ-E3 Bank Sandbox Credentials — Resolution Plan

**Decision ID**: D-2026-11-03-002
**Date**: 2026-11-03
**Decision maker**: PMT/ARCH (single-owner governance)
**Status**: Action plan defined

---

## 1. Decision

**PMT contacts E.Sun account manager within 48 hours (by 2026-11-05), requesting scoped sandbox credentials with a 30-day trial period. In parallel, build the mock security component (ADR-BP-005) so development continues regardless of sandbox availability.**

## 2. Rationale (Benefit/Cost Analysis)

| Option | Benefit | Cost | Risk | Decision |
|---|---|---|---|---|
| A: Wait passively for sandbox | Zero effort | Indefinite delay; P1 exit blocked; BankPay E2E unverified | High — no timeline, no leverage | ❌ Rejected |
| B: PMT contacts account manager with scoped request + mock fallback | Fastest path to real validation; scoped request reduces approval friction; mock unblocks dev | 2-4 hours PMT effort; mock development effort | Medium — depends on bank responsiveness | ✅ Selected |
| C: Switch to alternative bank | Eliminates single-vendor dependency | Massive rework; new API integration; new spec | High — sunk cost, timeline reset | ❌ Rejected |

**Key insight**: Scoped request (TWD payment + status query only, not all 8 APIs) reduces bank's compliance review burden, accelerating approval. 30-day trial limits financial commitment. Mock component ensures BankPay development never stalls.

## 3. Action Plan

### Immediate (2026-11-03 ~ 2026-11-05)

| # | Action | Owner | Due | Output |
|---|---|---|---|---|
| 1 | Draft sandbox request letter (scoped: TWD payment + status query, 30-day trial, 1-2 test accounts) | PMT | 11-04 | Request letter draft |
| 2 | Send request to E.Sun account manager via official channel (email + follow-up call within 24h) | PMT | 11-05 | Sent confirmation |
| 3 | Begin mock security component implementation (ADR-BP-005) | DEV | 11-05 | Mock component crate module |
| 4 | Document mock contract spec from V2.8 §1 (endpoints, request/response shapes, result codes) | DEV | 11-06 | Mock spec doc |

### Follow-up (2026-11-06 ~ 2026-11-14)

| # | Action | Owner | Due | Output |
|---|---|---|---|---|
| 5 | Follow up with account manager (call if no response within 5 business days) | PMT | 11-12 | Follow-up log |
| 6 | Mock component: implement `/auth` + `/payment/createTwdPaymentInstruction` + `/payment/retrieveTwdPaymentInstructionResult` | DEV | 11-14 | Working mock server |
| 7 | Mock component: integration tests exercising full payment lifecycle | DEV | 11-14 | Test suite |
| 8 | Update traceability matrix: mock tests marked as "mock-validated" (not bank-validated) | PMT | 11-14 | Updated matrix |

### Escalation (if no response by 2026-11-19)

| # | Action | Owner | Trigger |
|---|---|---|---|
| 9 | Escalate to management: request senior contact at E.Sun | PMT | 14 days no response |
| 10 | Evaluate alternative: negotiate paid sandbox access (if free trial unavailable) | PMT | Cost-benefit review |
| 11 | Consider: mock-only validation for P1 exit, defer real E2E to P2 | ARCH | If sandbox unavailable |

## 4. Cost-Down Strategies

| Strategy | Savings | Implementation |
|---|---|---|
| Scoped request (TWD only, not 8 APIs) | Reduces bank compliance review from 2-4 weeks to 1-2 weeks | Request letter specifies only TWD payment + status query |
| 30-day trial period | Limits financial commitment; allows evaluation | Request trial period explicitly |
| Mock component (ADR-BP-005) | Unblocks all development without sandbox; zero external cost | Build from V2.8 spec (already documented) |
| Reuse existing TLS/pinning infrastructure | No new code for cert handling | `PinnedCertVerifier` already implemented in `http.rs` |
| Batch API calls in test fixtures | Reduces mock server complexity | Design mock to handle batch requests |

## 5. Success Criteria

- [ ] Sandbox credentials received and verified (P0 exit: token acquisition + health check succeed)
- [ ] Mock security component operational in CI (P1 partial exit: local validation)
- [ ] TWD payment + status query E2E against real sandbox (P1 full exit)
- [ ] Traceability matrix updated with mock vs. bank validation distinction

## 6. Related

- `docs/bankpay/08-open-questions-and-decisions.md` OQ-E3 (pending)
- `docs/bankpay/03-technical-design.md` §1 (architecture with security component)
- `docs/TRACEABILITY_MATRIX.md` (FR-BP-01, FR-BP-02 status)
- `docs/bankpay/06-test-and-acceptance.md` §10 (exit criteria per phase)
- ADR-BP-005 (mock component for CI) — proposed in OQ doc
- MOP §11.5 (external dependencies & vendor management)
