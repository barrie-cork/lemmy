# Brief: impl-task 2 — v1-RT-r2 flag wiring + resolver + clamp + unit tests

## 1. Role + dispatch

`[role:impl-task] v1-RT-r2 task 2 — flag wiring + per-event half-life resolver + bounds clamp + 11 unit tests — see .claude/PRPs/briefs/v1-RT-r2-impl-2.md`

## 2. Scope

Wire the feature flag, per-event half-life resolution, and bounds clamp into
`recompute_snapshot` in `crates/api/api/src/governance/reputation_snapshot.rs`.
Add 11 new unit tests. Add `resolve_half_life_for_event` and `clamp_dimension_i32`
private helpers. Update the module docstring.

**Task 1 must already be on `phase-v1-RT-r2`** before this task starts (it adds
the `v1_enabled` parameter to `compute_applied_delta` + the `chained_halve` helper).

**Exactly 1 file: `crates/api/api/src/governance/reputation_snapshot.rs`**

### 2.1 Seven implementation steps

**Step 1 — `recompute_snapshot` config-read block (lines ~240–275 currently):**

Replace the existing single `decay_half_life_days` read with the block from
plan §10.3. Read `feature.reputation_v1_decay_enabled` FIRST; then read
`decay.positive_half_life_days` as `legacy_half_life`. Keep the 3 existing
threshold reads (`thresholds.reporting_accuracy`, `thresholds.jury_reliability`,
`thresholds.participation_consistency`) and the `jury_age_requirement_days` read
unchanged. Verbatim block from plan §10.3:

```rust
let v1_decay_enabled = config::get_bool(
  cache,
  &mut (&mut *conn).into(),
  Scope::Instance,
  "feature.reputation_v1_decay_enabled",
)
.await?;
let legacy_half_life_days = config::get_int(
  cache,
  &mut (&mut *conn).into(),
  Scope::Instance,
  "decay.positive_half_life_days",
)
.await?;
let legacy_half_life = Duration::days(legacy_half_life_days);
```

**Step 2 — per-event summation loop (lines ~286–294 currently):**

Replace with the block from plan §10.3:

```rust
let now = Utc::now();
let mut reporting_accuracy = 0_i32;
let mut jury_reliability = 0_i32;
let mut participation_consistency = 0_i32;
let mut endorsement_strength = 0_i32;

for event in &events {
  let half_life = if v1_decay_enabled {
    resolve_half_life_for_event(cache, conn, event).await?
  } else {
    legacy_half_life
  };
  let applied_delta = compute_applied_delta(event, now, half_life, v1_decay_enabled);
  match event.dimension {
    ReputationDimension::ReportingAccuracy => reporting_accuracy += applied_delta,
    ReputationDimension::JuryReliability => jury_reliability += applied_delta,
    ReputationDimension::ParticipationConsistency => participation_consistency += applied_delta,
    ReputationDimension::EndorsementStrength => endorsement_strength += applied_delta,
  }
}
```

**Step 3 — bounds clamp block (after loop, before `account_age_days` at ~line 297):**

Insert the block from plan §10.3:

```rust
if v1_decay_enabled {
  reporting_accuracy = clamp_dimension_i32(
    reporting_accuracy,
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance,
                    "bounds.reporting_accuracy.floor").await?,
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance,
                    "bounds.reporting_accuracy.ceiling").await?,
  );
  jury_reliability = clamp_dimension_i32(
    jury_reliability,
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance,
                    "bounds.jury_reliability.floor").await?,
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance,
                    "bounds.jury_reliability.ceiling").await?,
  );
  participation_consistency = clamp_dimension_i32(
    participation_consistency,
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance,
                    "bounds.participation_consistency.floor").await?,
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance,
                    "bounds.participation_consistency.ceiling").await?,
  );
  endorsement_strength = clamp_dimension_i32(
    endorsement_strength,
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance,
                    "bounds.endorsement_strength.floor").await?,
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance,
                    "bounds.endorsement_strength.ceiling").await?,
  );
}
```

**Step 4 — `resolve_half_life_for_event` helper (add after `acquire_advisory_xact_lock`):**

Verbatim from plan §10.4:

```rust
async fn resolve_half_life_for_event(
  cache: &mut ConfigCache,
  conn: &mut AsyncPgConnection,
  event: &ReputationEvent,
) -> LemmyResult<Duration> {
  let dim_segment = match event.dimension {
    ReputationDimension::ReportingAccuracy => "reporting_accuracy",
    ReputationDimension::JuryReliability => "jury_reliability",
    ReputationDimension::ParticipationConsistency => "participation_consistency",
    ReputationDimension::EndorsementStrength => "endorsement_strength",
  };
  let direction_segment = if event.delta >= 0 { "positive" } else { "negative" };
  let key = format!("decay.{dim_segment}.{direction_segment}_half_life_days");
  let days = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    &key,
  )
  .await?;
  Ok(Duration::days(days))
}
```

**Step 5 — `clamp_dimension_i32` helper (add near `resolve_half_life_for_event`):**

Verbatim from plan §10.5:

```rust
fn clamp_dimension_i32(value: i32, floor: i64, ceiling: i64) -> i32 {
  let widened = i64::from(value);
  let clamped = widened.clamp(floor, ceiling);
  i32::try_from(clamped).unwrap_or_else(|_| {
    if clamped > 0 { i32::MAX } else { i32::MIN }
  })
}
```

Note: `unwrap_or_else` is allowed under workspace clippy (deny list targets
`unwrap_used`/`expect_used`, not `unwrap_or_else`). The error branch is
unreachable given PRD §8 ranges; the `else` arm is defence-in-depth.

**Step 6 — 11 new unit tests (add after existing test at line ~958):**

Each is `#[test] fn name() { assert_eq!(...); }` — synchronous, no `unwrap`,
no `LemmyResult`, no `tokio::test`. MIRROR: existing `decay_applies_only_to_organic_events_past_half_life` test at line ~917.

Use `make_event` helper to construct `ReputationEvent` values (same as the existing test).
For `expires_at` tests, use `Utc::now() + Duration::days(30)`.

Tests to add:

1. `chained_halving_at_2x_half_life_quarters_delta` — organic +100, age=180d, hl=90d, v1=true → 25
2. `chained_halving_at_3x_half_life_eighths_delta` — organic +100, age=270d, hl=90d, v1=true → 12
3. `chained_halving_at_age_zero_no_decay` — organic +100, age=0d, hl=90d, v1=true → 100
4. `chained_halving_negative_delta_skipped` — penalty -20, age=180d, hl=90d, v1=true → -20
5. `chained_halving_founder_cliff_skipped` — organic +100, expires_at=Some(now+30d), age=180d, hl=90d, v1=true → 100
6. `v0_arm_at_2x_half_life_single_halving` — organic +100, age=180d, hl=90d, v1=false → 50
7. `chained_halve_helper_saturates_at_very_old_event` — `chained_halve(100, Duration::days(365*100), Duration::days(90)) == 0`
8. `clamp_dimension_i32_within_bounds` — `clamp_dimension_i32(50, -100, 100) == 50`
9. `clamp_dimension_i32_above_ceiling` — `clamp_dimension_i32(250, -100, 100) == 100`
10. `clamp_dimension_i32_below_floor` — `clamp_dimension_i32(-250, -100, 100) == -100`
11. `clamp_dimension_i32_negative_below_zero_floor` — `clamp_dimension_i32(-5, 0, 200) == 0`

**Step 7 — module docstring (insert after current line 46):**

```rust
//! ## v1 feature flag (RT-r2)
//!
//! `feature.reputation_v1_decay_enabled` (governance_config, default `false`)
//! gates the per-(dimension, direction) chained-halving decay + per-dimension
//! bounds clamp introduced in v1-RT-r2. When `false`, the v0 single-halving
//! against `decay.positive_half_life_days` is preserved verbatim and no clamp
//! applies. The 16 per-(dim, direction) + bounds keys are read only when the
//! flag is `true`; otherwise the legacy single key is read.
```

## 3. Required reading (read before first edit)

- `crates/api/api/src/governance/reputation_snapshot.rs:213-345` — full `recompute_snapshot` body (locate exact current line numbers for the decay_half_life_days read, the summation loop, and account_age_days)
- `crates/api/api/src/governance/reputation_snapshot.rs:286-294` — per-event loop to replace
- `crates/api/api/src/governance/reputation_snapshot.rs:577-690` — existing private helpers section (placement for new helpers)
- `crates/api/api/src/governance/reputation_snapshot.rs:857-875` — `make_snapshot` + `make_event` fixture (use for new tests)
- `crates/api/api/src/governance/reputation_snapshot.rs:917-958` — existing test (MIRROR shape)
- `crates/api/api/src/governance/config.rs:227-261` — `get_bool` + `get_int` signatures
- `crates/api/api/src/governance/config.rs:1137-1172` — verify exact key spellings for decay + bounds match arms
- `.claude/PRPs/plans/v1-RT-r2.plan.md` §10.3 + §10.4 + §10.5 + §13 Task 2 — full spec with verbatim code blocks
- `.claude/lessons/feedback_clippy_test_style.md` — no `unwrap`/`expect`; `assert_eq!` only
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — VALIDATE uses `--workspace --features full`

## 4. Constraints

- **Only 1 file:** `crates/api/api/src/governance/reputation_snapshot.rs`
- **No new migrations, no new config keys, no new public API.**
- **Borrow discipline:** pass `cache` and `conn` by `&mut` reborrow to `resolve_half_life_for_event`; do NOT clone or take by value (plan §13 Task 2 GOTCHA).
- **`clamp_dimension_i32` uses i64 clamp then narrows to i32** via `i32::try_from` — cast only AFTER the clamp (plan §7 preflight R1).
- **`get_int` returns `LemmyResult<i64>`** — pass directly to `Duration::days(i64)`; no intermediate `i32` cast on the half-life value.
- **Tests are synchronous `#[test]`** — no `tokio::test`, no `LemmyContext`, no DB.
- **`v1_decay_enabled = false`** was a placeholder in Task 1's line ~287 call. Task 2 replaces it with the live flag read.
- After edits run VALIDATE commands; write `kind: "validate-pending-laptop-e2e"` DQ entry with all four VALIDATE commands, `branch: "phase-v1-RT-r2"`, `phase_task: 2`. Push worker branch.

### VALIDATE commands (run all four before writing DQ entry)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r2-task2-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r2-task2-check.log
```

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r2-task2-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-RT-r2-task2-clippy.log
```

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --lib -- reputation_snapshot::tests > .claude/PRPs/debug/v1-RT-r2-task2-unit-tests.log 2>&1"
echo "exit: $?"
tail -30 .claude/PRPs/debug/v1-RT-r2-task2-unit-tests.log
```

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-RT-r2-task2-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-RT-r2-task2-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-RT-r2-task2-e2e.log"
tail -50 .claude/PRPs/debug/v1-RT-r2-task2-e2e.log
```

After all four pass: write DQ entry (`kind: "validate-pending-laptop-e2e"`), commit, push.

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1
    commit: c963c9066
    filesCreated: []
    filesModified:
      - crates/api/api/src/governance/reputation_snapshot.rs
    keyDecisions:
      - compute_applied_delta signature extended with v1_enabled bool parameter
      - chained_halve helper added immediately after compute_applied_delta
      - existing 4 test-callsites updated to pass false for v1_enabled
      - line ~287 production caller updated to pass false (placeholder for Task 2)
    notes: "Task 1 DoD passed laptop (cargo-check/clippy/unit-test exit 0). Submodule init required in worktree (feedback_worktree_submodules_not_auto_init). DQ 8aca794fb044-001 mutated result:pass."
```
