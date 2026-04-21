# Task 0 pre-phase audit — v1-AD-c

**Branch**: `phase-v1-AD-c`
**Base SHA**: `f03ed1cba` (post-PR-#76 merge, v1-AD-b tip)
**Date**: 2026-04-21
**Plan**: `.claude/PRPs/plans/v1-admin-dashboard-c.plan.md`
**Rule**: `.claude/rules/pre-phase-harness-audit.md`

## Summary

Pre-phase harness audit GREEN on all four probes + metadata parity. Clippy ratchet narrowed from `--workspace --all-targets --features full --no-deps` (draft plan) to `--workspace --features full --no-deps` (v1-AD-b precedent) per runlog Q1 decision. `--all-targets` superset baseline recorded at **N = 135 errors**, all in test code, for Task 8's advisory delta check. v1-AD-c is cleared to proceed to Task 1.

## Probe results

| # | Probe | Command | Exit | Log | Notes |
|---|---|---|---|---|---|
| 1 | `-p` crate scoping | `cargo-check.bat -p lemmy_api` | 0 (5m 51s) | `.claude/audit-cargo-check-p.log` | Wrapper honors `-p`; per-crate scope correct |
| 2 | `--features full` activation | `cargo-check.bat -p lemmy_api --features full` | 0 (29s) | `.claude/audit-cargo-check-features.log` | Feature flag flips cleanly; incremental cache warm |
| 3 | e2e test target scoping | `cargo-test.bat --test e2e --no-run -p lemmy_server` | 0 (2m 10s) | `.claude/audit-cargo-test.log` | Only `e2e` test target built; executable compiled |
| 4 | Negative feature (Issue #8 regression) | `cargo-check.bat -p lemmy_server --features nonexistent_xyz` | 101 | `.claude/audit-cargo-check-negative.log` | Wrapper correctly propagates non-zero exit code; `error: the package 'lemmy_server' does not contain this feature: nonexistent_xyz` in tail |

Exit-code capture per `.claude/rules/cargo-output-capture.md`: every probe redirected to file via `> file 2>&1`, exit code captured separately via `echo "exit: $?"`. No pipes.

## Clippy baseline — v1-AD-b ratchet (narrowed, production DoD)

**Command**: `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings`
**Log**: `.claude/audit-clippy-narrowed.log` (also re-confirmed at `.claude/audit-clippy-baseline-narrowed.log`)
**Exit**: 0 (5m 41s)
**Errors**: 0

Matches v1-AD-b's Level 6 ratchet (commit `3ca80e736` "chore(lint): clear pre-existing clippy debt on admin_config.rs + config.rs"). Production code lint-clean at v1-AD-c base.

## Clippy `--all-targets` superset — baseline for Task 8 delta (advisory, NOT DoD)

**Command**: `cargo-clippy.bat --workspace --all-targets --features full --no-deps -- -D warnings`
**Log**: `.claude/audit-clippy-baseline.log` (copy at `.claude/audit-clippy-alltargets.log`)
**Exit**: 101
**Errors**: **135** (baseline **N = 135** for Task 8 advisory delta gate)

All 135 errors are pre-existing governance-fork test-code debt. Zero errors in production code.

### Files affected

| File | Errors | Notes |
|---|---|---|
| `crates/server/tests/e2e.rs` | 126 | Integration test binary crate; flagged by `tests_outside_test_module` because `tests/e2e.rs` is NOT inside a `#[cfg(test)] mod` — this is a **known false-positive class** for Rust integration-test binaries (the file IS the test binary by convention) |
| `crates/api/api/src/governance/reputation_snapshot.rs` | 5 | Lines 838–856, all inside `#[cfg(test)] mod tests` — `indexing_slicing` on `changes[0]`/`changes[1]` in test assertions |
| `crates/api/api/src/governance/admin_config.rs` | 2 | Lines 1356 / 1375 — `.unwrap()` on `payload.as_object()` in the `governance_log_payload_shell_parity` parity helper |

### Lint distribution

| Count | Lint | Notes |
|---|---|---|
| 56 | `tests_outside_test_module` | False-positive class for `tests/e2e.rs`; `#[test]` functions in integration-test binary crates are not inside `mod tests { #[cfg(test)] ... }` by convention |
| 34 | `indexing_slicing` | `[0]`/`[1]` indexing in test assertions |
| 27 | `items_after_statements` | Inline `use` declarations inside test-function bodies after statements |
| 5 | `expect_used` (Option) | `.expect()` on `Option` in test code |
| 3 | `map_err_ignore` | Wildcard pattern in `.map_err(|_| ...)` |
| 2 | `unwrap_used` (Option) | `.unwrap()` on `Option` in parity helper |
| 1 | `expect_used` (Result) | `.expect()` on `Result` in test |
| 1 | `unreachable_macro` | Test-only `unreachable!()` |
| 1 | `too_many_arguments` | 9-arg function signature in test helper |
| 1 | `redundant_type_annotations` | Redundant type annotation in test |
| 1 | `needless_return` (Ok + `?`) | `Ok(...)?` unwrap redundancy |
| 1 | `needless_collect` | `iter().copied().collect()` vs `to_vec()` |
| 2 | (rollup summary lines) | `could not compile lemmy_server (test "e2e")` + `could not compile lemmy_api (lib test)` |
| **135** | **Total `^error:` lines** | 133 actionable lints + 2 rollup summaries |

### Root cause

Plan §15 Level 4 as originally drafted used `--workspace --all-targets --features full --no-deps -- -D warnings` — stricter than v1-AD-b's ratchet. The `--all-targets` flag pulls in `tests/e2e.rs` and `lemmy_api lib test` surfaces where pre-existing upstream-inherited + governance-fork-test debt lives. Upstream Lemmy's own `.woodpecker.yml` runs `--all-targets --all-features` and passes because Lemmy's integration tests are TypeScript/Jest, not Rust — so the Rust test-discipline lint surface at Brehon is novel.

### Decision (runlog Q1)

Option C hybrid: narrow plan §15 Level 4 to drop `--all-targets` (match v1-AD-b ratchet); Task 8 captures `--all-targets` delta vs baseline 135 as advisory guardrail against regression by new tests. Full rationale in `.claude/PRPs/v1-AD-c-runlog/02-task0-narrative.md` §Q1.

## CONFIG_KEY_METADATA parity (plan §7.1 trigger A check)

`crates/api/api/src/governance/config.rs` — **7** `requires_re_jury: true` rows verified at lines 967, 979, 1340, 1352, 1364, 1376, 1388. Key names byte-identical to plan §13 task 5 `REQUIRES_RE_JURY_KEYS` constant. No drift; DQ-V1-AD-C-01 does not fire.

## v1-AD-b surfaces (plan §13 task 0 step 3 check)

All 6 surfaces present on `f03ed1cba`:

- ✅ `pub async fn get_int_opt` — `crates/api/api/src/governance/config.rs:333`
- ✅ `pub async fn admin_set_config` — `crates/api/api/src/governance/admin_config.rs:408`
- ✅ `applied_config_snapshot` column on `ModerationCase` + `InsertForm` — `crates/db_schema/src/source/governance/moderation_case.rs:41,66`
- ✅ `rule_set_version_id` column on `ModerationCase` + `InsertForm` — `crates/db_schema/src/source/governance/moderation_case.rs:45,67`
- ✅ `pub struct RuleSetVersion` — `crates/db_schema/src/source/governance/rule_set_version.rs:23`
- ✅ `pub struct RuleSetVersionInsertForm` — `crates/db_schema/src/source/governance/rule_set_version.rs:40`

## Phase-close follow-up (filed after v1-AD-c PR merges, NOT now)

**Issue title**: `chore(lint): clear e2e.rs + governance test-target clippy debt (135 errors at v1-AD-c base)`

**Body**: link this audit report; attach the lint distribution table above; mark `--workspace --all-targets --features full --no-deps -- -D warnings` as the eventual ratchet target. Candidate `chore(lint)` mini-phase between v1-AD-c and v1-AD-d.

## Verdict

**GREEN — proceed to Task 1.** All six audit points pass under the narrowed clippy DoD. The `--all-targets` superset red is expected and tracked; it does not block Task 1.
