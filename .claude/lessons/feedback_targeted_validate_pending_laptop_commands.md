# Targeted validate-pending-laptop command selection

## Rule

When authoring a `validate-pending-laptop` DQ entry (pre-Shape-G plans), select
the minimum set of cargo commands that provide meaningful signal for the change
class. Do NOT default to the full 4-command sequence for every task.

## Change-class heuristic

| Change class | Commands |
|---|---|
| Production code + tests (new fn, new const, new imports) | check + clippy + lib-test + e2e `--no-run` |
| Test-only (new test module or new test fn, no production change) | clippy + lib-test |
| Test assertion fix only (logic fix, no new imports/types) | clippy + lib-test |
| `.claude/` meta only (brief, DQ, lesson, runlog) | none |

**Why:**
- `cargo-check` is redundant when the prior task's check already passed AND the
  new change adds no new types, imports, or function signatures. Test assertion
  text doesn't affect compilation.
- `e2e --no-run` (link check) is only valuable when the changed file has
  cross-crate consumers exercised by e2e. A governance-module test file with no
  direct e2e fixture coverage adds ~2-3 min warm for near-zero signal.
- `lib-test` is always warranted for any `.rs` change — it's the fastest
  correctness signal (~1 min warm for a single crate) and catches runtime
  assertion failures that check + clippy cannot.

## How to apply

At brief-authoring time, classify the change against the table above. Encode
the selected commands verbatim in the DQ entry's `commands` array. Add a
one-line comment in the brief explaining which class applies and why any
commands were omitted.

## Evidence

- v1-federation-inbound-d Task 1 (production code + test): full 4-command
  sequence correct — new const, new fn block, new imports warranted check +
  clippy + lib-test + e2e `--no-run`.
- v1-federation-inbound-d Task 2 fix-impl-1 (test assertion fix): check
  omitted (prior check passed, no new types/imports); e2e `--no-run` omitted
  (`publish_trust_attestation.rs` has no direct e2e coverage path). clippy +
  lib-test sufficient. Saved ~3-4 min per validate cycle.

## Related lessons

- `feedback_features_full_workspace_only.md` — `--workspace --features full`
  for check + clippy; lib-test uses `-p <crate> --lib` without `--features full`
- `feedback_clippy_test_style.md` — workspace lint discipline
