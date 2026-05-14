---
phase: chore/refactor-seed-tests
role: impl-task
task: refactor
brief_n: 1
authored: 2026-05-14
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_finding: 3.E.20 (rank 17)
parent_phase_tip: <set by bm-cut — branch tip is governance-v0 HEAD at bm-cut time>
---

# [role:impl-task] chore/refactor-seed-tests — add unit tests for parse_founder_spec — see .claude/PRPs/briefs/refactor-seed-tests-impl.md

## §1 Role + dispatch

`[role:impl-task] chore/refactor-seed-tests — add unit tests for parse_founder_spec in seed_founders per audit §3.E.20`

## §2 Scope

### §2.1 Driving audit finding

Audit §3.E.20 (rank 17, severity MED, effort S, frequency 1):

> `seed_founders/src/main.rs:75-130` `parse_founder_spec` validation function no unit tests · Lens 3 · Axis-4 quality-fail · **[MED]** function is the main entry point for CLI parsing; bugs here break the tool; no edge-case coverage (negative values, out-of-range, parse errors, wrong segment count) · add `#[cfg(test)] mod tests` block with `test_parse_founder_spec_valid`, `_negative_delta`, `_exceeds_max`, `_wrong_segment_count`, `_non_numeric` cases · **S** · 1 file, 5 tests.

### §2.2 Verified function shape (pre-edit; from `crates/tools/seed_founders/src/main.rs` lines 75-122)

```rust
fn parse_founder_spec(raw: &str, max_seed_delta: i32) -> LemmyResult<FounderSpec> {
  let parts: Vec<&str> = raw.split(':').collect();
  if parts.len() != 4 {
    return Err(LemmyErrorType::Unknown(format!(
      "founder spec `{raw}` must be PERSON_ID:JUR:RPT:END (4 colon-separated ints)"
    ))
    .into());
  }
  let parse_i32 = |idx: usize, label: &str| -> LemmyResult<i32> {
    let raw_part = parts.get(idx).copied().unwrap_or("");
    raw_part.parse::<i32>().map_err(|_e| {
      LemmyErrorType::Unknown(format!("founder spec `{raw}` {label} not a valid i32")).into()
    })
  };
  let person_id_raw = parse_i32(0, "person-id")?;
  let jury_reliability = parse_i32(1, "jury_reliability")?;
  let reporting_accuracy = parse_i32(2, "reporting_accuracy")?;
  let endorsement_strength = parse_i32(3, "endorsement_strength")?;
  // Each of jury/reporting/endorsement must be > 0 AND <= max_seed_delta.
  for (label, value) in [
    ("jury_reliability", jury_reliability),
    ("reporting_accuracy", reporting_accuracy),
    ("endorsement_strength", endorsement_strength),
  ] {
    if value <= 0 { /* error */ }
    if i64::from(value) > i64::from(max_seed_delta) { /* error */ }
  }
  Ok(FounderSpec { ... })
}
```

### §2.3 The edit — add `#[cfg(test)] mod tests`

At the END of `crates/tools/seed_founders/src/main.rs` (after `async fn main`, before EOF), add:

```rust
#[cfg(test)]
mod tests {
  use super::*;

  /// Happy path: well-formed input parses with all values within range.
  #[test]
  fn parse_founder_spec_valid() {
    let spec = parse_founder_spec("42:5:10:15", 100).expect("valid spec parses");
    assert_eq!(spec.person_id.0, 42);
    assert_eq!(spec.jury_reliability, 5);
    assert_eq!(spec.reporting_accuracy, 10);
    assert_eq!(spec.endorsement_strength, 15);
  }

  /// Zero or negative reputation values reject (each dimension must be > 0).
  #[test]
  fn parse_founder_spec_negative_delta() {
    let err = parse_founder_spec("1:0:5:5", 100).unwrap_err();
    assert!(err.to_string().contains("jury_reliability must be > 0"),
            "expected jury_reliability > 0 error, got: {err}");

    let err = parse_founder_spec("1:5:-3:5", 100).unwrap_err();
    assert!(err.to_string().contains("reporting_accuracy must be > 0"),
            "expected reporting_accuracy > 0 error, got: {err}");
  }

  /// Values exceeding max_seed_delta reject.
  #[test]
  fn parse_founder_spec_exceeds_max() {
    let err = parse_founder_spec("1:101:5:5", 100).unwrap_err();
    assert!(err.to_string().contains("jury_reliability=101 exceeds founder.max_seed_delta=100"),
            "expected exceeds-max error, got: {err}");
  }

  /// Wrong segment count (not exactly 4 colons-separated) rejects.
  #[test]
  fn parse_founder_spec_wrong_segment_count() {
    let err = parse_founder_spec("1:5:5", 100).unwrap_err();   // 3 segments
    assert!(err.to_string().contains("must be PERSON_ID:JUR:RPT:END"),
            "expected segment-count error, got: {err}");

    let err = parse_founder_spec("1:5:5:5:5", 100).unwrap_err(); // 5 segments
    assert!(err.to_string().contains("must be PERSON_ID:JUR:RPT:END"),
            "expected segment-count error, got: {err}");
  }

  /// Non-numeric segments reject with parse-error message.
  #[test]
  fn parse_founder_spec_non_numeric() {
    let err = parse_founder_spec("not_a_num:5:5:5", 100).unwrap_err();
    assert!(err.to_string().contains("person-id not a valid i32"),
            "expected person-id parse error, got: {err}");

    let err = parse_founder_spec("1:abc:5:5", 100).unwrap_err();
    assert!(err.to_string().contains("jury_reliability not a valid i32"),
            "expected jury_reliability parse error, got: {err}");
  }
}
```

### §2.4 Test assertion notes (avoid clippy-test-style anti-patterns)

Per `.claude/lessons/feedback_clippy_test_style.md`:

- **`.expect(...)` for the happy path is fine** in test code — the workspace clippy denies `unwrap` in non-test code but allows annotated assertions in tests.
- **`.unwrap_err()` followed by `.to_string()` then substring check** is the canonical pattern (sibling fixtures in `e2e.rs` use this; matches Brehon test style).
- **`#[test]` (not `#[tokio::test]`)** — `parse_founder_spec` is sync; no async runtime needed.
- **No use of `#[allow(...)]`** — none of the new tests should need attribute escape hatches.
- **No floating-point comparisons** — all values are i32; equality checks are safe.

### §2.5 Validation gate (Shape G)

Push the worker branch. Triggers `cargo-validate-workspace.yml`. The workspace check runs `cargo test --no-run` which will compile (but not execute) the new tests. **The tests do NOT auto-execute via Shape G workflow** — they only compile.

To actually run the tests locally before push:

```bash
bash scripts/brehon/cargo-test.sh --workspace --features full -p seed_founders --tests > .claude/PRPs/debug/refactor-seed-tests-runtests.log 2>&1
status=$?
tail -20 .claude/PRPs/debug/refactor-seed-tests-runtests.log
[ $status -eq 0 ] || exit $status
```

**Worker MUST run the tests locally before push** — Junior dispatch runs on EliteDesk where Docker + Postgres are available, but `parse_founder_spec` is a pure parsing function (no DB, no testcontainers). The 5 tests run in <1 second. Failing to run them locally first risks shipping broken tests.

Per `.claude/rules/cargo-output-capture.md` — capture-then-tail pattern.

Capture `workflow_run_id` after push. Raise `kind: "validate-pending"` DQ entry per Recipe 1.

## §3 Required reading

- `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` §3.E.20 — driving finding
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` — PR-6 of 6 context
- `crates/tools/seed_founders/src/main.rs` lines 75-122 — function being tested
- `.claude/agents/impl-task.md` — subagent contract
- `.claude/lessons/feedback_clippy_test_style.md` — test-style invariants (LemmyResult, no unwrap-in-non-test, etc.)
- `.claude/rules/decision-queue.md` Recipe 1

## §4 Constraints

- **Files:** ONLY `crates/tools/seed_founders/src/main.rs`. NO other files.
- **Edits:** ONE addition only — a `#[cfg(test)] mod tests { ... }` block appended at the end of the file. No edits to existing code in the file.
- **Test count:** exactly 5 test functions as named in §2.3. Not more (out of scope). Not fewer (audit specifically called these 5 out).
- **Test isolation:** each test is independent; no shared state; no setup/teardown.
- **Branch:** `chore/refactor-seed-tests`.
- **Pre-push cargo-check + cargo-test (mandatory per `feedback_fix_impl_pre_push_cargo_check.md`):**

  ```bash
  bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/refactor-seed-tests-precheck.log 2>&1
  status=$?
  tail -20 .claude/PRPs/debug/refactor-seed-tests-precheck.log
  [ $status -eq 0 ] || exit $status

  # AND run the new tests locally to confirm they pass:
  bash scripts/brehon/cargo-test.sh --workspace --features full -p seed_founders --tests > .claude/PRPs/debug/refactor-seed-tests-runtests.log 2>&1
  status=$?
  tail -20 .claude/PRPs/debug/refactor-seed-tests-runtests.log
  [ $status -eq 0 ] || exit $status
  ```

  Per `.claude/rules/cargo-output-capture.md` + `no-cargo-output-paste.md`.

- **Shape G:** push triggers workspace check. Raise 1 validate-pending DQ entry.
- **DQ discipline:** atomic raise + ensure_ascii=False + next_id-spans-archives.
- **COMMIT MESSAGE:** `test(seed_founders): unit tests for parse_founder_spec validation (audit 3.E.20)`

## §5 Out of scope

- Other untested functions in seed_founders (`resolve_admin`, `count_active_founders`, `seed_one_founder`, `run`, `main`) — audit only flagged `parse_founder_spec`. Defer.
- Integration tests requiring a DB (`testcontainers`) — out of scope; pure-parse unit tests only.
- Adding `tracing` setup or test logging.
- Refactoring `parse_founder_spec` itself (e.g. audit §3.E.19's separate finding on `unwrap_or("")`) — that's a different finding, separate refactor.
- Changing `Cargo.toml` to add `dev-dependencies` (not needed — `LemmyErrorType` is already available via existing imports).

## §6 HANDOVER trailer

Parallel-lane refactor; no cohort handover. Trailer optional. If added: note that this is the first test surface in `seed_founders/`; future refactors can extend the `mod tests` block without restructuring.
