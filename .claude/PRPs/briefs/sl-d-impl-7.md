---
phase: v1-SL-d
role: impl-task
task: 7
brief_n: 1
authored: 2026-05-10
---

# [role:impl-task] v1-SL-d task 7 unit tests — compute idempotency + grace_window_for_severity mapping — see .claude/PRPs/briefs/sl-d-impl-7.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-d task 7 unit tests compute idempotency grace window severity mapping`

## §2 Scope

In `crates/api/api/src/governance/sponsor_liability.rs`, append a
`#[cfg(test)] mod tests { ... }` block at end-of-file (after the closing `}`
of `apply_sponsor_liability`) with 2 unit tests:

1. `compute_sponsor_liability_idempotent` — calls `compute_sponsor_liability(...)`
   twice with identical inputs; asserts returned `Vec<SponsorDelta>` is byte-equal
   both times; asserts no `reputation_event` or `governance_log` rows written by
   either call.

2. `grace_window_for_severity_reads_correct_config_key` — for each severity tier
   (Minor / Moderate / Severe), invoke `grace_window_for_severity` and assert the
   duration matches the SL-a-seeded defaults (24h / 72h / 168h).

**Test fn signatures (Case A — mandatory):**
```rust
#[tokio::test]
async fn compute_sponsor_liability_idempotent() -> LemmyResult<()>

#[tokio::test]
async fn grace_window_for_severity_reads_correct_config_key() -> LemmyResult<()>
```

**Pre-flight GOTCHA (testcontainers in crates/api/api):** at task-start, read
existing `#[cfg(test)] mod tests` blocks in
`crates/api/api/src/governance/*.rs`. If NO in-crate testcontainers pattern
exists there, **file a `kind: "blocker"` DQ entry** asking the advisor whether
to (a) add testcontainers dev-dependency to `crates/api/api/Cargo.toml` or
(b) relocate both tests to `crates/server/tests/e2e.rs` (inside
`mod v1_sl_d_fixtures`). Per DQ #180 (LOCKED 2026-05-10): default lean is
to keep in `sponsor_liability.rs::tests`; relocate only if infrastructure
friction requires it. Do NOT guess — DQ and wait.

If in-crate testcontainers IS already present, proceed with the skeleton from
§10.7.

**Anchor identification at task-start (if relocating to e2e.rs):**
```bash
grep -n 'apply_sponsor_liability_wrapper_preserves_v0_outputs' crates/server/tests/e2e.rs | tail -1
```
Insert the new tests after the closing `}` of that fn — but only if the
advisor DQ resolves to option (b).

**FILES:**
```yaml
creates: []
modifies:
  - crates/api/api/src/governance/sponsor_liability.rs   # append #[cfg(test)] mod tests block
```
(or `crates/server/tests/e2e.rs` if advisor DQ resolves to option (b))

### 2.1 §G4 CANONICAL CASE OVERRIDE — MANDATORY

**Case A (LemmyResult<()>) for all test fns.** All helper signatures
`-> LemmyResult<T>`. All test fn signatures `-> LemmyResult<()>`. Bare `?`
propagation. NO `Box<dyn Error>`. NO `.map_err` bridges.

## §3 Required reading

- `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` §10.7 (unit test pattern
  skeleton — read the full `#[cfg(test)] mod tests { ... }` skeleton), §13 Task 7
  (IMPLEMENT steps + all GOTCHAs), §10.4 (config-key namespaces for
  grace_window assertions)
- `crates/api/api/src/governance/sponsor_liability.rs` — full file (currently
  ~358 lines post-Task-1): `compute_sponsor_liability` signature, `SponsorDelta`
  struct (verify `PartialEq + Debug` derives from Task 1), `grace_window_for_severity`
  signature, `apply_sponsor_liability` closing `}` (anchor for append)
- `crates/api/api/src/governance/*.rs` — scan for existing `#[cfg(test)] mod tests`
  blocks to determine if testcontainers is already in-crate; this is the pre-flight
  gate described above
- `crates/db_schema_file/src/enums.rs` — read `CaseSeverity` variant set (Minor /
  Moderate / Severe) to ensure test loop uses correct variant names
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A mandatory;
  `LemmyResult<()>` test fn, `LemmyResult<T>` helpers, bare `?`
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish`
  requires `use diesel_async::AsyncConnection;`
- `.claude/lessons/feedback_clippy_test_style.md` — no `unwrap`/`expect` in new code

## §3a Handover from prior cohort

Task 6 final clean commit: `test(v1-SL-d): e2e test #4 — wrapper preserves v0 outputs (compose-equivalence) (task 6)`.
DQ #198 result: `pass` — workspace-check clean.
`mod v1_sl_d_fixtures` now contains Test #1 + Test #2 + Test #3 + Test #4 + helpers (COMPLETE — this is the LAST test in that mod).

Key notes:
- `SponsorDelta` derives `PartialEq + Debug` (confirmed in Task 1 — verify by reading sponsor_liability.rs before writing)
- `compute_sponsor_liability` and `grace_window_for_severity` are both `pub(crate)` — callable from `sponsor_liability.rs::tests` via `use super::*;`
- The testcontainers pre-flight check is mandatory before writing any test code

## §4 Constraints

- **Files:** only `crates/api/api/src/governance/sponsor_liability.rs` (nominal).
  If pre-flight DQ resolves to option (b), only `crates/server/tests/e2e.rs`.
  No other files.
- **ONE append** — append `#[cfg(test)] mod tests { ... }` at end-of-file. No
  other edits.
- **Case A canonical shape** — `LemmyResult<()>` test fn, bare `?`. HARD constraint.
- **No new imports in non-test code** — all imports go inside `#[cfg(test)] mod tests`.
- **DQ first if infrastructure absent** — file blocker DQ before writing a line of
  test code if testcontainers is not already in-crate.
- **Shape G:** after committing, push the worker branch; write a `kind: "validate-pending"`
  DQ entry with `workflow_run_id` from the triggered `cargo-validate-workspace.yml` run.
- **DQ mid-task push:** if a DQ blocker is filed, commit + push immediately per
  decision-queue.md.
- **COMMIT MESSAGE:** `test(v1-SL-d): unit tests for compute_sponsor_liability idempotency + grace_window_for_severity mapping (task 7)`
