# Brief: impl-task 1 — v1-RT-r2 compute_applied_delta rewrite

## 1. Role + dispatch

`[role:impl-task] v1-RT-r2 task 1 — compute_applied_delta rewrite + chained_halve helper — see .claude/PRPs/briefs/v1-RT-r2-impl-1.md`

## 2. Scope

Rewrite the body of `compute_applied_delta` in
`crates/api/api/src/governance/reputation_snapshot.rs` to accept a new
`v1_enabled: bool` parameter and apply per-event chained-halving when
`true`, preserving the v0 single-halving path verbatim when `false`.
Add the `chained_halve` private helper immediately after the function.

**Exactly 5 edits in 1 file:**

1. Replace `crates/api/api/src/governance/reputation_snapshot.rs` lines
   **354–387** with the verbatim body from plan §10.1 (new
   `compute_applied_delta` signature + body including docstring).
2. Insert the `chained_halve` helper from plan §10.2 immediately after
   `compute_applied_delta`'s closing brace.
3. Update **line 287** in `recompute_snapshot` — the single production
   caller of `compute_applied_delta` — to pass a temporary `false` for
   the new `v1_enabled` argument (Task 2 wires the live flag; Task 1
   just keeps compilation green).
4. Update the **4 existing test-callsites** at lines **936, 943, 950,
   957** (`assert_eq!(compute_applied_delta(...), N)`) to pass `false`
   as the new last argument. No new tests in this task.
5. Preserve the docstring at the top of `compute_applied_delta` verbatim
   from plan §10.1.

**No other file edits. No new migrations. No new config keys.**

### 2.1 Verbatim plan §10.1 body (Task 1 implements this exactly)

```rust
/// Compute the decayed delta applied to the running sum for a single
/// event (Watch 8). Organic positive events get halved once per complete
/// half-life elapsed when v1 decay is enabled; founders keep their full
/// delta until their cliff fires; penalties (delta <= 0) persist at full
/// value.
///
/// The `half_life` argument carries the resolved per-(dimension, direction)
/// half-life when `v1_enabled = true`, OR the legacy
/// `decay.positive_half_life_days` value when `v1_enabled = false`.
///
/// Watch 8 — decay half-life must NOT apply to events with expires_at set
/// (founder cliffs).
fn compute_applied_delta(
  event: &ReputationEvent,
  now: DateTime<Utc>,
  half_life: Duration,
  v1_enabled: bool,
) -> i32 {
  let original = event.delta;
  // Cliff guard FIRST: founder seeds skip decay entirely.
  if event.expires_at.is_some() {
    return original;
  }
  // Penalty guard SECOND: negative + zero deltas never decay.
  if original <= 0 {
    return original;
  }
  // Positive organic event — apply the v1 or v0 decay path.
  if v1_enabled {
    // Chained halving per complete half-life elapsed.
    chained_halve(original, now - event.created_at, half_life)
  } else {
    // v0 path: single halving past one half-life. PRESERVE EXACTLY.
    let age = now - event.created_at;
    if age > half_life {
      original / 2
    } else {
      original
    }
  }
}
```

### 2.2 Verbatim plan §10.2 helper (Task 1 inserts this after §10.1)

```rust
/// Apply chained halving: `original >> floor(age_days / half_life_days)`,
/// saturating at 0 for very large `n`. Returns `original` when
/// `half_life_days <= 0` (defensive against misconfigured DB rows; the
/// PRD section 8 range floor is 1, so this branch is defence-in-depth).
fn chained_halve(original: i32, age: Duration, half_life: Duration) -> i32 {
  let hl_days = half_life.num_days();
  if hl_days <= 0 {
    return original;
  }
  let age_days = age.num_days();
  if age_days < hl_days {
    return original;
  }
  // Number of complete half-lives elapsed; n >= 1 here.
  // i64 division floors toward zero for positive operands.
  let n = age_days / hl_days;
  // Saturate at 31 to avoid undefined-behaviour shift overflow;
  // beyond that the result is 0 (any non-zero i32 shifted past 31 is 0
  // for positive operands).
  let shift = n.min(31) as u32;
  // For positive i32, right-shift is arithmetic AND logical (both yield
  // 0 in the limit). Penalty guard above ensures original > 0 here.
  original >> shift
}
```

## 3. Required reading

- `crates/api/api/src/governance/reputation_snapshot.rs:354-387` — current `compute_applied_delta` body to replace
- `crates/api/api/src/governance/reputation_snapshot.rs:280-295` — the single production caller at line ~287 that needs `false` added
- `crates/api/api/src/governance/reputation_snapshot.rs:917-958` — existing test callsites (4 callsites to update + the MIRROR shape)
- `crates/db_schema_file/src/enums.rs:609-615` — `ReputationDimension` (4 variants)
- `crates/db_schema/src/source/governance/reputation_event.rs:20-36` — `ReputationEvent` struct (`delta: i32`, `expires_at: Option<DateTime<Utc>>`)
- `.claude/PRPs/plans/v1-RT-r2.plan.md` §10.1 + §10.2 + §13 Task 1 — full spec
- `.claude/lessons/feedback_clippy_test_style.md` — no `unwrap`/`expect`; `assert_eq!` only in tests
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — VALIDATE commands use `--workspace --features full`, NEVER `-p lemmy_api --features full`

## 4. Constraints

- **Only 1 file touched:** `crates/api/api/src/governance/reputation_snapshot.rs`
- **No new tests in Task 1** — existing 4 test-callsites updated to pass `false`; new tests are Task 2's scope.
- **`v1_enabled = false` placeholder** at line ~287 (the recompute_snapshot caller) — do NOT read the config flag yet; Task 2 does that.
- **Cliff guard FIRST, penalty guard SECOND** — the guard order is invariant per ADR-005 + Watch 8 in plan §7. The new `v1_enabled` branch comes AFTER both guards.
- **`chained_halve` is a private `fn`** — not `pub`. Inserted immediately after `compute_applied_delta`'s closing brace.
- **Shift safety:** `n.min(31) as u32` before the right-shift — see plan §13 Task 1 GOTCHA.
- **No migrations, no config keys, no new public API.**
- After all edits, run VALIDATE commands and write `kind: "validate-pending-laptop"` DQ entry with all three VALIDATE commands, `branch: "phase-v1-RT-r2"`, `phase_task: 1`. Push worker branch.

### VALIDATE commands (run before writing DQ entry)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r2-task1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r2-task1-check.log
```

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r2-task1-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-RT-r2-task1-clippy.log
```

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --lib -- reputation_snapshot::tests::decay_applies_only_to_organic_events_past_half_life > .claude/PRPs/debug/v1-RT-r2-task1-test.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r2-task1-test.log
```

After all three exit 0: write DQ entry, commit, push.
