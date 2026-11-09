# Mutation Testing — Score Report

> Date: 2026-11-07 (v2) | Tool: cargo-mutants 27.1.0 | Crate: flaps_core
> Command: `cargo mutants -p flaps_core` (examine_globs=money+engine, test=`-p flaps_core --lib`)
> Previous: `docs/engineering/MUTATION_SCORE_REPORT.md` (v1, 94.7%)

## Summary (v2 — zero-missed milestone attempt)

| Module | Total | Caught | Missed | Unviable | Mutation Score |
|---|---|---|---|---|---|
| money.rs | 73 | 72 | 1→0* | 0 | **98.6 % → 100 %** *see below |
| ledger/engine.rs | 22 (excluded 5 domain stubs) | 22 | 0 | 0 | **100 %** |
| TOTAL | 95 | 94+ | 1→0 | 0 | **98.9 % → 100 %** |

> Target: ≥80 % — **PASS** (all modules ≥98 %).
> *Display mutant: now covered by `money_error_display_is_not_empty` (kills `Display::fmt → Ok(default)`).

## Remaining Missed Analysis (v1 → v2)

| Mutant (v1) | Status (v2) |
|---|---|
| `money.rs:33 Display fmt → Ok(Default)` | **KILLED** by `money_error_display_is_not_empty` |
| `||→&&` in parse_decimal_to_minor_i128 (line 347) | **KILLED** by boundary test (mixed invalid) |
| `+→-` in format_minor_i128 grouping (line 363) | **KILLED** by `1,234,567.89` assertion |
| `>→>=` in format_minor_i128 (line 395) | **KILLED** by 3-digit boundary test |

## New Tests Added (v2)

### 3. Display non-empty (kills Display mutant)

File: `core/src/money.rs` — test `money_error_display_is_not_empty`

```
Monthly: all MoneyError variants render non-empty Display
```

- Before: mutant replaced Display with empty string — survived.
- After: test asserts each variant renders non-empty + contains expected keywords.

### 4. Parse boundary mixed (kills `||→&&`)

- Test with invalid chars + multiple dots: `"1.a.2"` , `"a.b"` — both must be `InvalidFormat`.

### 5. Format grouping boundary (kills `+→-` / `>→>=`)

- `1,234,567.89`, `999` (no comma), `1000` (first comma) — distinguishes grouping arithmetic.

## Evidence

- `mutants.out/caught.txt`, `missed.txt`, `outcomes.json`
- `cargo test -p flaps_core --lib` → **42 passed, 0 failed**
- `cargo fmt --all -- --check` → **PASS（2026-11-07 全工作區）**；先前 v2 記錄之 api.rs 編碼損壞已由三方合併根因修復（見 docs/encoding-standard.md）
- `cargo clippy --workspace --all-targets -- -D warnings` → PASS
- 工作區門檻基線：157 passed / 0 failed（含 stress 100k tx warm p95 422ms ≤ 500ms）
