# Plan: v1-RT-r2 — per-dimension chained-halving decay + bounds clamping

## 1. Summary

v1-RT-r2 ships the per-dimension/per-direction chained-halving decay calculator
inside `recompute_snapshot` and a per-dimension bounds clamp on the dimension
sums, both gated by `feature.reputation_v1_decay_enabled`. When the flag is
`true`, organic positive events are halved once per complete half-life elapsed
(age=2x half-life -> delta/4, age=3x half-life -> delta/8) using the relevant
`decay.<dim>.<direction>_half_life_days` knob; per-dimension sums are clamped
to `[bounds.<dim>.floor, bounds.<dim>.ceiling]`. When the flag is `false`, the
v0 single-halving against `decay.positive_half_life_days` is preserved
verbatim and no clamp applies. Founder-cliff events
(`expires_at.is_some()`) and penalty events (`original <= 0`) skip decay
entirely, unchanged from v0. Acceptance: unit tests demonstrate
age={0, 1, 2, 3}x half-life on positive organic events; the legacy v0 single
half-life behaviour for `feature.reputation_v1_decay_enabled = false`;
bounds clamping when accumulated sums exceed ceiling or floor.

## 2. Source

- `.claude/PRPs/prds/v1-reputation-tuning.prd.md` section 5.2 (per-dimension/per-direction decay + chained halving + bounds), section 6 (acceptance), section 8 (defaults matrix), section 9 (backwards compat + 9.1 feature-flag posture), section 11 row 2 (this sub-phase scope) @ `57b1ade60`
- `.claude/PRPs/briefs/v1-RT-r2-planning-1.md` @ `57b1ade60`
- DQ `a3d0e9941441-009` (bounds clamp must read per-dim config; `[0, i32::MAX]` rejected; resolved)
- DQ `a3d0e9941441-010` (full per-dimension/per-direction half-life reads, 8 keys; resolved)
- `.claude/lessons/feedback_clippy_test_style.md` — Lemmy test-style legal patterns
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — error-shape uniformity across e2e modules (referenced; r2 unit tests live in `#[cfg(test)] mod tests` inside `reputation_snapshot.rs` and do NOT touch `crates/server/tests/e2e.rs`)
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — FILES YAML discipline
- `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` — pre-enumerate callsites; `compute_applied_delta` is private (`fn compute_applied_delta(...)`), 1 caller in `recompute_snapshot`, 4 test-callsites in same file. `recompute_snapshot` is `pub`; signature is preserved — 5 external callsites enumerated below in section 11 stay untouched.
- `.claude/lessons/feedback_plan_dod_dry_run_at_write.md` — every section 15 command dry-runnable
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — `--features full` requires `--workspace`, never `-p <crate> --features full`
- ADR-005 (multi-dimensional reputation; decay tuning at snapshot level not event level — unchanged), ADR-008 (append-only event log; r2 touches snapshot logic, not events — unchanged), ADR-010 (v1 milestone targeting "production-grade governance" — this PRD is chartered v1)
- `.claude/PRPs/reports/v1-RT-r1-retro.md` — r1 phase context (schema foundation; 26 config keys; ENTRY_KIND_DECAY_KNOB_CHANGED const pre-landed for r2 emit site)
- `.claude/PRPs/handovers/v1-RT-r2-bootstrap.md` — phase bootstrap brief (watchpoints section 4, scope-violation tripwires section 7)

## 3. Problem statement

The v0 `compute_applied_delta` (`crates/api/api/src/governance/reputation_snapshot.rs:354-387`) implements a single-halving decay against a single instance-wide `decay.positive_half_life_days` knob. A user with a `+100 endorsement_strength` event from 270 days ago still contributes `+50` (halved exactly once at age > 90d), then continues to contribute `+50` to the snapshot forever — there is no second halving at age > 180d. The TODO at line 375 names exactly this gap: "v1 — switch to chained halving per half-life elapsed and add a regression test for age > 2x half-life."

The v0 `recompute_snapshot` (line 213-345) also performs no bounds clamping: the per-dimension sums are written to `reputation_snapshot.<dim>` rows directly. A user with 50 +5 endorsement_strength events accumulates +250 even though `bounds.endorsement_strength.ceiling = 200`. Without clamping, the [01 section 1 principle 5] "no permanent elites" intent is unsupported.

Both gaps are gated by `feature.reputation_v1_decay_enabled` (seeded by r1, default `false`). r2 wires the new behaviour behind that flag without touching the v0 code path's default semantics.

## 4. Solution statement

Replace the body of `compute_applied_delta` with a per-event-direction chained-halving routine that takes the resolved `Duration` for this event's `(dimension, direction)` half-life as an extra argument. `recompute_snapshot` performs the (dimension, direction) -> half-life resolution once per event in its existing summation loop. When `feature.reputation_v1_decay_enabled = false`, `recompute_snapshot` short-circuits the resolution: it reads only the legacy `decay.positive_half_life_days` key and passes the same `Duration` to every positive event (matching v0); negative events and founder seeds still skip decay.

The watchpoints to **preserve**:
- **Watch 2 / Watch 8 (penalty + cliff guard):** the existing `if event.expires_at.is_none() { if original <= 0 { return original; } ... }` shape stays. The new chained-halving only runs in the positive-organic branch; penalties + founder seeds bypass it entirely.
- **`recompute_snapshot` external signature unchanged:** `pub async fn recompute_snapshot(conn, person_id, community_id, cache) -> LemmyResult<ReputationSnapshot>`. The 5 existing callsites do not edit. `compute_applied_delta`'s signature MAY change internally (it's `fn`, not `pub`).

After the per-event summation loop, `recompute_snapshot` reads the 8 `bounds.<dim>.<floor|ceiling>` keys (only when the flag is `true`) and applies clamp to each dimension sum before constructing the `ReputationSnapshotInsertForm`. When the flag is `false`, no clamp applies (v0 has no bounds — preserve exactly).

Unit tests in the existing `#[cfg(test)] mod tests` at the bottom of `reputation_snapshot.rs` cover the chained-halving math + flag arms + clamp behaviour. No new e2e tests; r2 is compute-logic only. No new migrations; r1 owns the schema. No new config keys; r1 seeded all 17 keys r2 reads.

## 5. Metadata

- **Phase:** `v1-RT-r2`
- **Branch:** `phase-v1-RT-r2` (cut by `bm-cut` before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 4 (Task 0 pre-flight + Tasks 1-2 impl + Task 3 retro)
- **Estimated cargo budget:** `~6 GB peak` (single `cargo check --workspace --features full` per task; no e2e edits; no migrations)
- **Forbidden-window applicability:** standard (per `.claude/rules/advisor-orchestrator.md` section 5.1). Shape G suspended per DQ #229 -> validate-pending-laptop pathway active; cargo runs on laptop. Forbidden-window check applies.
- **Complexity score:** `1/10`

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| section 13 impl tasks above 5 | +1 each | 0 | 2 impl tasks (1, 2); Task 0 + retro excluded |
| Migrations touched | +2 each | 0 | r2 is compute-logic only; PRD section 11 row 2 + brief section 2 |
| Crates touched | +1 each | 1 | only `crates/api/api/src/governance/reputation_snapshot.rs` |
| `crates/server/tests/e2e.rs` edits | +3 each | 0 | r2 unit tests stay in `reputation_snapshot.rs` `#[cfg(test)] mod tests` |
| New ADR-affecting decisions | +2 each | 0 | r2 inherits PRD's ADR posture; no new OQ deltas |
| Cargo budget peak above 6 GB | +1 per GB | 0 | ~6 GB peak; pre-Shape-G plan running validate-pending-laptop |
| **Total** | — | **1** | Threshold for split-DQ: `>8` (Sonnet target); no split |

### 5.2 Per-task complexity ceiling (Sonnet target)

Sonnet ceiling: `<=4 files per task` and `<=2 distinct crates per task`. Task 1 modifies 1 file (1 crate). Task 2 modifies 1 file (same crate). Both well under ceiling. No e2e file appears in either task's `modifies:`.

## 6. Relationship to other v1-RT-* sub-phases

- **Depends on:** v1-RT-r1 (PR #126, merged) — schema foundation (`dedupe_key`, `source_event_type` columns; `ReputationEventSourceType` enum; 26 governance_config seed keys; `ENTRY_KIND_*` consts). All 17 keys r2 reads (8 decay + 8 bounds + 1 feature flag) are present on `governance-v0` HEAD `ece760ae6`.
- **Followed by:** v1-RT-r3 (multi-source participation events: weekly active cron, dormancy cron, vote-outcome emitter, evidence-quality emitter). r3 produces new `reputation_event` rows; r2's per-dimension calculator + bounds clamp absorb them transparently.
- **Sibling parallel:** v1-RT-r4 (sponsor-gate strategies — independent of r2), v1-RT-r5 (instance-wide rollup — consumes r2 snapshots), v1-RT-r6 (CR carry-forward bundle — independent).
- v1 planning-queue id: `RT-r2` (per PRD section 11).

## 7. Preflight guardrails inherited from prior phases

- **R1 (clippy):** every `i32 <-> i64` comparison uses `i64::from(...)`, never `as` cast (per `feedback_clippy_test_style.md`). Bounds clamp reads `i64` from config; cast back to `i32` for the snapshot sum **only after** the `clamp` is performed on `i64`.
- **R2 (config read):** every governance_config read goes through `config::get_int` / `config::get_bool` with the `ConfigCache` already in scope inside `recompute_snapshot` (line 240, 247, 254, 261, 268). Never call `fetch_value` directly.
- **R3 (decay branch order):** the existing `if event.expires_at.is_none()` outer + `if original <= 0` inner gate stays as the **first two checks** in `compute_applied_delta` regardless of flag arm. Penalty + cliff bypass is invariant per ADR-005 + Watch 8.
- **R4 (test style):** unit tests use `#[test]` (synchronous; existing `compute_applied_delta` tests are synchronous). No `unwrap`/`expect`; `assert_eq!` is fine in `#[cfg(test)]` modules.
- **R5 (Task 0 audit):** enumerate ALL probes explicitly (per `.claude/rules/pre-phase-harness-audit.md`).
- **R6 (clippy invocations):** all clippy commands use `--no-deps` uniformly.
- **R7 (test-target compile gate):** any task that edits a struct or re-export runs `cargo test --no-run -p lemmy_server --test e2e`. r2 tasks do not edit structs or re-exports -> R7 is **not invoked**.
- **R8 (features full):** all cargo commands use `--features full --workspace`, NEVER `-p <crate> --features full` (per `feedback_features_full_p_crate_incompatible.md`).

## 8. Flow design

### Before (v0, flag = false default)

```
recompute_snapshot(conn, person_id, community_id, cache):
  events = load_live_events(...)
  half_life = Duration::days(get_int("decay.positive_half_life_days"))
  for event in events:
    applied = compute_applied_delta(event, now, half_life)
    sum[event.dimension] += applied
  return upsert(sum)

compute_applied_delta(event, now, half_life) -> i32:
  if expires_at.is_some(): return original   # founder cliff
  if original <= 0: return original          # penalty
  if (now - created_at) > half_life: return original / 2  # single halving
  else: return original
```

### After (r2, flag arms)

```
recompute_snapshot(conn, person_id, community_id, cache):
  events = load_live_events(...)
  v1_enabled = get_bool("feature.reputation_v1_decay_enabled")
  legacy_half_life = Duration::days(get_int("decay.positive_half_life_days"))
  ...
  for event in events:
    half_life = resolve_half_life(event, v1_enabled, legacy_half_life, cache, conn)
    applied = compute_applied_delta(event, now, half_life, v1_enabled)
    sum[event.dimension] += applied
  if v1_enabled:
    sum[ReportingAccuracy] = clamp(sum[ReportingAccuracy], floor, ceiling)
    ... (3 more dimensions)
  return upsert(sum)

compute_applied_delta(event, now, half_life, v1_enabled) -> i32:
  if expires_at.is_some(): return original
  if original <= 0: return original
  if v1_enabled:
    n = floor((now - created_at).num_days() / half_life.num_days())
    return original >> n      # chained halving via right-shift, saturating
  else:
    if (now - created_at) > half_life: return original / 2
    else: return original

resolve_half_life(event, v1_enabled, legacy, cache, conn) -> Duration:
  if !v1_enabled: return legacy
  direction = if event.delta >= 0 then "positive" else "negative"
  key = format!("decay.{dim}.{direction}_half_life_days")
  Duration::days(get_int(key))
```

**Box -> task map:**

- `compute_applied_delta` body rewrite -> Task 1
- `recompute_snapshot` flag read + per-event half-life resolution + bounds clamp -> Task 2
- Unit tests for the four-cases x two arms x clamp -> Task 2

## 9. Mandatory reading

The impl-task subagent MUST Read these before its first edit on each task:

**Schema / type definitions:**
- `crates/db_schema_file/src/enums.rs:609-615` — `ReputationDimension` (4 variants)
- `crates/db_schema/src/source/governance/reputation_event.rs:20-36` — `ReputationEvent` struct (note `delta: i32`, `expires_at: Option<DateTime<Utc>>`, `dimension: ReputationDimension`)

**Existing patterns:**
- `crates/api/api/src/governance/reputation_snapshot.rs:213-345` — full `recompute_snapshot` body (the function r2 edits)
- `crates/api/api/src/governance/reputation_snapshot.rs:354-387` — `compute_applied_delta` (the function whose body r2 replaces)
- `crates/api/api/src/governance/reputation_snapshot.rs:286-294` — the per-event summation loop where the new half-life resolution is wired
- `crates/api/api/src/governance/config.rs:227-261` — `get_int` signature + cache semantics
- `crates/api/api/src/governance/config.rs:299-348` — `get_bool` signature
- `crates/api/api/src/governance/config.rs:992-1009` — the 8 decay + 8 bounds DEFAULT_* consts (key spellings + types)
- `crates/api/api/src/governance/config.rs:1033` — `DEFAULT_FEATURE_REPUTATION_V1_DECAY_ENABLED: bool = false`
- `crates/api/api/src/governance/config.rs:1137-1172` — 16 match-arm key strings that r2 reads (verify the exact lowercase + dot spelling per key)

**Adjacent test fixtures (MIRROR refs for Task 2's new tests):**
- `crates/api/api/src/governance/reputation_snapshot.rs:917-958` — the existing `decay_applies_only_to_organic_events_past_half_life` test (canonical shape for new tests)
- `crates/api/api/src/governance/reputation_snapshot.rs:857-875` — `make_snapshot` fixture (kept; not edited)

**Lessons (per section 2.4 mandatory file-class lesson injection per `.claude/rules/advisor-orchestrator.md` section 2.4):**
- `.claude/lessons/feedback_clippy_test_style.md` (Task 2 adds tests to `reputation_snapshot.rs` `#[cfg(test)] mod tests` — within `crates/api/api/src`, NOT `crates/server/tests/e2e.rs`. The Lemmy clippy `unwrap_used` / `expect_used` denies still apply; new tests use `assert_eq!` only, no `unwrap`)
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` (Tasks 1+2 section VALIDATE commands use `--workspace --features full`, NEVER `-p lemmy_api --features full`)
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` (planner asserted `union(creates, modifies) == IMPLEMENT files` for both impl tasks)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` (Case A vs Case B vs Case C — r2 tests are **synchronous** `#[test]` blocks returning `()`, NOT `LemmyResult<()>` or `Box<dyn Error>` — neither Case A nor Case B applies; the existing `decay_applies_only_to_organic_events_past_half_life` test at line 917 is the literal MIRROR and returns `()`)

## 10. Patterns to mirror

### 10.1 `compute_applied_delta` — penalty + cliff guards stay first

**Mirror:** `crates/api/api/src/governance/reputation_snapshot.rs:354-387`

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

### 10.2 Chained-halving helper — saturating right-shift

**Mirror:** new private fn introduced in Task 1; verbatim body below.

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

### 10.3 `recompute_snapshot` — flag read + per-event half-life resolution + bounds clamp

**Mirror:** `crates/api/api/src/governance/reputation_snapshot.rs:240-294` (the config-read block + the per-event summation loop). The block below replaces the existing `decay_half_life_days` read and the loop body.

```rust
// 4. Read the config thresholds, feature flag, decay half-life, and bounds
//    via the cache. NOTE: all reads flow through a single ConfigCache so
//    repeated lookups in one recompute don't hit the DB multiple times.
let v1_decay_enabled = config::get_bool(
  cache,
  &mut (&mut *conn).into(),
  Scope::Instance,
  "feature.reputation_v1_decay_enabled",
)
.await?;
// Always read the legacy key so the v1=false arm uses the v0 single
// half-life behaviour verbatim.
let legacy_half_life_days = config::get_int(
  cache,
  &mut (&mut *conn).into(),
  Scope::Instance,
  "decay.positive_half_life_days",
)
.await?;
let legacy_half_life = Duration::days(legacy_half_life_days);
let threshold_jury_reliability = config::get_int(
  cache,
  &mut (&mut *conn).into(),
  Scope::Instance,
  "thresholds.jury_reliability",
)
.await?;
// ... (other threshold reads stay unchanged: reporting_accuracy,
//      endorsement_strength, jury_age_requirement_days)

// 5. Sum deltas by dimension, applying decay only when expires_at is
//    None (Watch 8 — double-decay guard). The per-event half-life is
//    resolved per-(dimension, direction) when v1 decay is enabled, or
//    the legacy single half-life when disabled.
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

// 5a. (r2) Bounds clamping — only when v1 decay is enabled. v0 has no
//     bounds and must not change behaviour when the flag is false.
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

### 10.4 `resolve_half_life_for_event` — per-(dimension, direction) read

**Mirror:** new private async fn introduced in Task 2.

```rust
/// Resolve the half-life for a given event under v1 decay. Reads the
/// `decay.<dim>.<positive|negative>_half_life_days` key matching the
/// event's `(dimension, direction)` from the ConfigCache.
///
/// Direction:
/// * `delta >= 0` -> `positive` (organic positive events; the only path
///   that actually invokes decay because the penalty guard in
///   `compute_applied_delta` returns early for `delta <= 0`)
/// * `delta < 0` -> `negative` (read for future-completeness; the negative
///   half-life is wired but never consumed in r2 because penalties
///   short-circuit before the half-life is consulted)
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

### 10.5 `clamp_dimension_i32` — i64 clamp + i32 narrowing

**Mirror:** new private fn introduced in Task 2.

```rust
/// Clamp a per-dimension i32 sum to the (floor, ceiling) bounds read as
/// i64 from governance_config. The clamp is performed on i64 to avoid
/// any chance of the sum overflowing i32 before the clamp narrows it.
fn clamp_dimension_i32(value: i32, floor: i64, ceiling: i64) -> i32 {
  let widened = i64::from(value);
  let clamped = widened.clamp(floor, ceiling);
  // After clamp, the value is guaranteed to be within [floor, ceiling],
  // both of which fit in i32 per the PRD section 8 ranges
  // (-10000 to 10000). Casting back via i32::try_from is therefore safe;
  // we use saturating semantics defensively.
  i32::try_from(clamped).unwrap_or_else(|_| {
    if clamped > 0 { i32::MAX } else { i32::MIN }
  })
}
```

Note: `unwrap_or_else` is allowed under workspace clippy because the deny list targets `unwrap_used` / `expect_used`, not `unwrap_or_else`. The error branch is unreachable given PRD section 8 ranges; the `else` arm exists for defence-in-depth.

## 11. Files to change

### `crates/api/api` (1 crate, 1 file)

- `crates/api/api/src/governance/reputation_snapshot.rs` — replace `compute_applied_delta` body + add `chained_halve` private fn (Task 1); read feature flag + per-event half-life resolver + bounds clamp in `recompute_snapshot` (Task 2); add unit tests for the new behaviour and the flag=false arm (Task 2).

### Caller crates (no edits — signatures preserved)

`recompute_snapshot`'s 5 external callsites stay untouched (signature: `pub async fn recompute_snapshot(conn: &mut AsyncPgConnection, person_id: PersonId, community_id: Option<CommunityId>, cache: &mut ConfigCache) -> LemmyResult<ReputationSnapshot>`):

- `crates/api/api_crud/src/governance/create_endorsement.rs:300` — sponsor recompute
- `crates/api/api_crud/src/governance/create_endorsement.rs:301` — sponsee recompute
- `crates/api/api_crud/src/governance/revoke_endorsement.rs:335` — revoking sponsor recompute
- `crates/api/api_crud/src/governance/revoke_endorsement.rs:337` — sponsored-person recompute
- `crates/server/tests/e2e.rs:3278, 3778, 3779` — e2e test imports (the import statement at 3278 plus two call sites in the existing test at 3778/3779; no edits)

The `compute_applied_delta` private fn signature changes (adds a `v1_enabled: bool` parameter) but is `fn` (not `pub`); its only non-test caller is `recompute_snapshot` itself at line 287 (edited in Task 2). The 4 existing test-callsites at lines 936, 943, 950, 957 (inside `#[cfg(test)] mod tests` in the same file) are updated in Task 2 alongside the new tests.

**`rg compute_applied_delta crates/` confirms** (planner-side enumeration per `feedback_fix_impl_enumerate_all_callsites.md`):
1 caller in `recompute_snapshot` + 4 test-callsites + 2 declarations (the fn definition + its docstring reference at line 287) — all in the same file. No cross-crate impact. Zero risk of E0063 cascade.

### Struct-field add: none

r2 does not add fields to any struct. `ReputationSnapshot` and `ReputationSnapshotInsertForm` shapes are unchanged. The form is constructed from the same four `i32` per-dimension sums; only the values change.

## 12. NOT building in v1-RT-r2

- **New migrations** — r2 is compute-logic only per PRD section 11 row 2 + brief section 2.4. r1 owns the schema. A migration in r2 = scope violation; advisor catch-fire tripwire per bootstrap section 7.
- **New governance_config keys** — all 18 keys r2 reads (1 feature flag + 1 legacy decay + 8 per-(dim, direction) decay + 8 bounds) already shipped by r1 (or pre-r1 in the case of `decay.positive_half_life_days`). Total keys r2 *introduces*: 0.
- **New cron / scheduler entries** — r3 owns participation + dormancy crons; r5 owns rollup cron. r2's behaviour is invoked transparently by the existing 15-min `run_snapshot_batch` tick at `crates/routes/src/utils/scheduled_tasks.rs:177`.
- **New REST endpoints** — r2 introduces no new HTTP handlers. The flag is flipped via the v0 admin-config-write CLI wrapper or the v1-AD-b admin-config endpoint (when it ships); r2 just reads the value.
- **`ENTRY_KIND_DECAY_KNOB_CHANGED` emit** — the const is pre-landed by r1 in the entry-kind registry; the emit site is v1-AD-b's admin-config write handler per the pre-landed-const exemption. **Deferred** to v1-AD-b. r2 reads decay knobs; it does not write them.
- **e2e tests** — r2 unit tests live in `#[cfg(test)] mod tests` in `reputation_snapshot.rs`. No e2e edits (avoids `feedback_junior_worker_e2e_edit_hang.md` risk).
- **`load_or_compute_snapshot` / `run_snapshot_batch` / `check_snapshot_staleness` edits** — r2 doesn't touch these; they call `recompute_snapshot` transitively and inherit r2's new behaviour for free.
- **Negative-direction half-life consumed in the body** — the resolver reads the negative key (defence-in-depth, future-completeness for r3+'s negative emitters), but `compute_applied_delta`'s penalty guard short-circuits before the half-life is consulted for `delta < 0`. The negative key is therefore "wired but inert" in r2; r3+ may introduce negative-direction decay when sources emit `-2` dormancy events. r2 keeps the v0 "penalties never decay" invariant exactly.

---

## 13. Step-by-step tasks

> **Cohort dispatch:** Tasks 1 and 2 share the same file (`reputation_snapshot.rs`). They are therefore **serial** (no `[P]`). Task 0 is the pre-flight barrier; Task 3 (retro) is the post-impl barrier.
>
> **Shape G:** SUSPENDED per DQ #229 until 2026-06-01. r2 is a pre-Shape-G plan: cargo runs on laptop via the `validate-pending-laptop` handler. impl-task subagents on the EliteDesk write the `kind: "validate-pending-laptop"` DQ entry after push; the advisor laptop session runs the section 15 commands and mutates the entry.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready for v1-RT-r2; confirm branch is `phase-v1-RT-r2`; confirm prior phase's r1 deliverables are intact on the base (`governance-v0` HEAD `ece760ae6` per handover); confirm pre-existing clippy baseline is clean.

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes explicitly):**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — wrapper sanity (cargo-check honors -p)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/audit-cargo-check-p.log 2>&1"
tail -20 .claude/audit-cargo-check-p.log
# EXPECT: only lemmy_utils compiles

# Probe 2 — feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/audit-cargo-check-features.log 2>&1"
echo "exit: $?"
tail -20 .claude/audit-cargo-check-features.log
# EXPECT: workspace compiles with --features full; exit 0

# Probe 3 — cargo-test wrapper honors target selection
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --no-run --features full > .claude/audit-cargo-test.log 2>&1"
tail -20 .claude/audit-cargo-test.log
# EXPECT: only e2e test target compiles

# Probe 4 — wrappers fail loud on cargo errors (exit-code propagation)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --no-run --features nonexistent_xyz > .claude/audit-cargo-test-negative.log 2>&1"
echo "cargo-test.bat exit on bogus feature: $?"
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"
# EXPECT: both lines print non-zero (typically 101)

# Probe 5 — clippy baseline against prior-phase HEAD (governance-v0 + r1 merge)
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/audit-clippy-baseline.log 2>&1"
echo "exit: $?"
tail -40 .claude/audit-clippy-baseline.log
# EXPECT: exit 0 (clippy baseline must be green before Task 1)

# Probe 6 — current branch is phase-v1-RT-r2 (cut by bm-cut)
git branch --show-current
# EXPECT: phase-v1-RT-r2

# Probe 7 — r1 schema effects landed on the base
git log governance-v0 --oneline | head -25
# EXPECT: r1 merge commit visible in log; PR #126 / its merge sha referenced

# Probe 8 — 17 governance_config keys r2 reads are present in const_default_int /
#           const_default_bool (mechanical check; no DB call)
rg "decay\.(reporting_accuracy|jury_reliability|participation_consistency|endorsement_strength)\.(positive|negative)_half_life_days" crates/api/api/src/governance/config.rs | wc -l
# EXPECT: 8 lines (8 per-(dim, direction) decay keys)
rg "bounds\.(reporting_accuracy|jury_reliability|participation_consistency|endorsement_strength)\.(floor|ceiling)" crates/api/api/src/governance/config.rs | wc -l
# EXPECT: 8 lines (8 per-dim bounds keys)
rg 'feature\.reputation_v1_decay_enabled' crates/api/api/src/governance/config.rs | wc -l
# EXPECT: at least 1 (in const_default_bool match arm)

# Probe 9 — concurrent-PR check (no other PR touches reputation_snapshot.rs)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | contains("reputation_snapshot.rs")) | {number, title, headRefName}'
# EXPECT: empty output; if v1-ship-2 or another lane is touching this file, STOP
```

**EXPECT block:**
- Probes 0..3, 5..9 exit 0 (or as documented per probe)
- Probe 4 exits NON-ZERO (negative test confirms exit-code propagation)
- Probe 6 returns `phase-v1-RT-r2`
- Probe 8 confirms key strings are in the match arms (mechanical, not behavioural)
- Probe 9 returns empty (no concurrent PR overlap)

**No commit at Task 0** — this is verification only. If any probe fails, file a `kind: "blocker"` DQ pending entry and stop.

### Task 1: Replace `compute_applied_delta` body — add chained halving + v1_enabled gate

**ACTION:** rewrite `compute_applied_delta` to accept a `v1_enabled: bool` parameter and apply the per-event chained-halving path when `true` (against the `half_life` argument the caller resolves); preserve the v0 single-halving path verbatim when `false`. Add the `chained_halve` private helper.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/reputation_snapshot.rs   # compute_applied_delta body rewrite + new chained_halve fn
```

**IMPLEMENT (file 1 of 1):**
- Replace lines `354-387` of `crates/api/api/src/governance/reputation_snapshot.rs` with the verbatim body from section 10.1 (the new `compute_applied_delta`).
- Insert the new `chained_halve` helper from section 10.2 immediately after `compute_applied_delta`'s closing brace (so both functions live side-by-side; the helper is `fn` private to the module).
- Update the single caller at line 287 in `recompute_snapshot` to pass a temporary placeholder `false` for the new `v1_enabled` argument (Task 2 wires the actual flag read).
- Update the 4 existing test-callsites at lines 936, 943, 950, 957 (`assert_eq!(compute_applied_delta(...), N)`) to pass `false` as the new `v1_enabled` argument. Task 2 ADDS new tests for the `true` arm + `chained_halve` math; Task 1's edits are limited to keeping the existing tests green.
- Preserve the docstring at the top of `compute_applied_delta` verbatim from section 10.1.

**MIRROR:** `crates/api/api/src/governance/reputation_snapshot.rs:354-387` for the shape; `:917-958` for the existing test layout.

**GOTCHA:**
- The PRD section 5.2 says "chained halving per half-life elapsed" — this means **floor division** of the age by the half-life, then right-shift the delta by `n`. At age = 2 x half-life exactly, `n = 2`, so `original >> 2 = original / 4`. The existing test at line 936 uses `age = 180 days, half_life = 90 days` (age = 2 x half-life), and the v0 assertion is `compute_applied_delta(...) == 50` (a single halving). Under v1, the same input should produce `25` (two halvings). Task 1's edits to that assertion **pass `false`** so the v0 path is preserved; Task 2 adds the v1-arm assertion that yields `25`.
- `Duration::num_days()` returns `i64`. The division `age_days / hl_days` is i64; cast to `u32` via `.min(31) as u32` before the right-shift to avoid overflow on truly ancient events.
- `i32::wrapping_shr` is NOT needed here — for positive operands within `[0, i32::MAX]` and shift amount within `[0, 31]`, `>>` is well-defined arithmetic right-shift yielding non-negative results. The penalty guard ensures `original > 0` before the shift.

**VALIDATE:**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r2-task1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r2-task1-check.log
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r2-task1-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-RT-r2-task1-clippy.log
# EXPECT: exit 0
```

```bash
# Re-run the existing unit test to confirm v0 semantics preserved
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --lib -- reputation_snapshot::tests::decay_applies_only_to_organic_events_past_half_life > .claude/PRPs/debug/v1-RT-r2-task1-test.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r2-task1-test.log
# EXPECT: exit 0, "1 passed; 0 failed"
```

**Post-validate:** impl-task writes `kind: "validate-pending-laptop"` DQ entry per `advisor-orchestrator.md` section 5.2. Required fields: `commands` (the three VALIDATE commands above), `branch: "phase-v1-RT-r2"`, `phase_task: 1`. Pushes worker branch. Advisor laptop session mutates the entry.

### Task 2: Wire feature flag, per-event half-life resolution, bounds clamp, and new unit tests

**ACTION:** in `recompute_snapshot`, read `feature.reputation_v1_decay_enabled`, resolve per-(dimension, direction) half-life per event when the flag is `true`, apply bounds clamp to the 4 dimension sums when the flag is `true`. Preserve v0 behaviour verbatim when the flag is `false`. Add unit tests for the new behaviour and the flag=false arm.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/reputation_snapshot.rs   # flag read + per-event resolver + bounds clamp in recompute_snapshot, + resolve_half_life_for_event + clamp_dimension_i32 helpers, + new unit tests
requires:
  - task: 1
    reason: "compute_applied_delta signature change (added v1_enabled arg) must be in place before Task 2's recompute_snapshot edits pass the live flag value through."
```

**IMPLEMENT (file 1 of 1):**

1. In `recompute_snapshot` (lines `240-275` currently), replace the existing `decay_half_life_days` read with the block from section 10.3 (read `feature.reputation_v1_decay_enabled` first; read `decay.positive_half_life_days` as `legacy_half_life`; keep the existing 3 threshold reads + `jury_age_requirement_days` read unchanged).
2. Replace the per-event summation loop (lines `286-294` currently) with the block from section 10.3 (resolve per-event half-life via `resolve_half_life_for_event` when `v1_enabled`, else use `legacy_half_life`; pass `v1_enabled` to `compute_applied_delta`).
3. Insert the bounds clamp block from section 10.3 between the loop end and the `account_age_days` computation (existing line 297). The clamp runs only when `v1_decay_enabled = true`; v0 path leaves the sums unclamped.
4. Add `resolve_half_life_for_event` from section 10.4 as a private async fn in the helpers section (after `acquire_advisory_xact_lock`).
5. Add `clamp_dimension_i32` from section 10.5 as a private fn in the helpers section.
6. Update the test-callsites in `#[cfg(test)] mod tests` (lines 936, 943, 950, 957) — Task 1 changed them to pass `false` for `v1_enabled`; Task 2 ADDS new tests after the existing test (line 958) covering:
   - **`chained_halving_at_2x_half_life_quarters_delta`**: organic +100, age = 180d, half_life = 90d -> `compute_applied_delta(..., v1_enabled = true) == 25`.
   - **`chained_halving_at_3x_half_life_eighths_delta`**: organic +100, age = 270d, half_life = 90d -> `compute_applied_delta(..., v1_enabled = true) == 12` (100 >> 3 = 12).
   - **`chained_halving_at_age_zero_no_decay`**: organic +100, age = 0d, half_life = 90d -> `compute_applied_delta(..., v1_enabled = true) == 100`.
   - **`chained_halving_negative_delta_skipped`**: penalty -20, age = 180d, half_life = 90d -> `compute_applied_delta(..., v1_enabled = true) == -20` (penalty guard fires regardless of flag).
   - **`chained_halving_founder_cliff_skipped`**: organic +100, expires_at = Some(now + 30d), age = 180d -> `compute_applied_delta(..., v1_enabled = true) == 100` (cliff guard fires regardless of flag).
   - **`v0_arm_at_2x_half_life_single_halving`**: organic +100, age = 180d, half_life = 90d -> `compute_applied_delta(..., v1_enabled = false) == 50` (v0 single halving preserved).
   - **`chained_halve_helper_saturates_at_very_old_event`**: `chained_halve(100, Duration::days(365 * 100), Duration::days(90)) == 0` (n = ~405 -> clamped to 31 -> 100 >> 31 = 0).
   - **`clamp_dimension_i32_within_bounds`**: `clamp_dimension_i32(50, -100, 100) == 50`.
   - **`clamp_dimension_i32_above_ceiling`**: `clamp_dimension_i32(250, -100, 100) == 100`.
   - **`clamp_dimension_i32_below_floor`**: `clamp_dimension_i32(-250, -100, 100) == -100`.
   - **`clamp_dimension_i32_negative_below_zero_floor`**: `clamp_dimension_i32(-5, 0, 200) == 0` (per `bounds.endorsement_strength.floor = 0`; OQ-024 zero-floor preserved).

   Each test is a `#[test]` (synchronous) `fn name() { assert_eq!(...); }` block. NO `unwrap` / `expect`; NO `LemmyResult` (per `feedback_clippy_test_style.md` — the existing `decay_applies_only_to_organic_events_past_half_life` test is the literal MIRROR and uses `assert_eq!` only). The chained-halving + clamp logic is pure (no DB, no async); these tests do NOT need `tokio::test` or `LemmyContext`.

7. Module docstring update: add a one-line `## v1 feature flag` note to the `//!` doc block at the top of the file (insert between current line 46 and the empty line at 47) explaining the flag gates the new behaviour. Verbatim text:

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

**MIRROR:** `crates/api/api/src/governance/reputation_snapshot.rs:240-294` (existing config-read + summation loop); `:577-690` (existing private helper layout for `resolve_half_life_for_event` placement); `:917-958` (existing test layout for new test block; same `make_snapshot` fixture stays).

**GOTCHA:**
- The `cache: &mut ConfigCache` borrow inside `recompute_snapshot` is **mutably borrowed for the entire loop body** when read from `resolve_half_life_for_event(cache, conn, event)`. Each event triggers up to 1 `get_int` call (cached after the first event of each (dim, direction) pair); the 4-dim x 2-direction = 8 possible keys are at most read once each per recompute. The `cache: &mut ConfigCache` is structured to interleave reads inside loops by design (see existing usage at line 240, 247, 254, 261, 268). Pass `cache` AND `conn` by `&mut` reborrow to the resolver fn; do NOT clone or take the cache by value.
- After the per-event loop ends, the borrow on `cache` is dropped — so the 8 bounds reads in section 5a are sequential `await?` calls and re-borrow `cache` fresh.
- `Duration::days(get_int(...).await?)` — `get_int` returns `LemmyResult<i64>`, and `Duration::days(i64)` is the existing pattern at line 280. Don't introduce intermediate `i32` casting on the half-life days value (PRD section 8 range is `1-3650`, but the key type in config is i64; preserve i64 throughout the duration construction).
- The bounds-clamp block reads 8 config keys serially (no parallelism). At ~0.1ms per cache-hit read, total overhead is <1ms — within the per-recompute budget per `feedback_resource_budget_pre_queue.md`.
- The module-level docstring edit at step 7 is a comment-only change and does NOT count toward the section 5.2 per-task complexity ceiling.

**VALIDATE:**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r2-task2-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r2-task2-check.log
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r2-task2-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-RT-r2-task2-clippy.log
# EXPECT: exit 0
```

```bash
# Run new unit tests
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --lib -- reputation_snapshot::tests > .claude/PRPs/debug/v1-RT-r2-task2-unit-tests.log 2>&1"
echo "exit: $?"
tail -30 .claude/PRPs/debug/v1-RT-r2-task2-unit-tests.log
# EXPECT: exit 0, all tests (existing 1 + new 11 = 12) passed
```

```bash
# Re-run e2e to catch any regression in upstream callers (reputation_snapshot::recompute_snapshot
# is used by create_endorsement.rs, revoke_endorsement.rs, and several e2e tests)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-RT-r2-task2-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-RT-r2-task2-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-RT-r2-task2-e2e.log"
tail -50 .claude/PRPs/debug/v1-RT-r2-task2-e2e.log
# EXPECT: E2E_EXIT_0 marker present; all pre-existing e2e tests still pass.
# (r2 default flag = false -> flag-arm unchanged from v0; e2e behaviour preserved.)
```

**Post-validate:** impl-task writes `kind: "validate-pending-laptop-e2e"` DQ entry for the e2e command (separate kind per `decision-queue.md` "validate-pending-laptop-e2e"). Required fields: `commands` (the four VALIDATE commands above), `branch: "phase-v1-RT-r2"`, `phase_task: 2`. Pushes worker branch. Advisor laptop session mutates the entry.

### Task 3: Retro

**Goal:** author retro at `.claude/PRPs/reports/v1-RT-r2-retro.md` per `.claude/lessons/feedback_retro_not_report.md` and `.claude/lessons/feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM) with signals + lessons. Promote any new lessons to `.claude/lessons/feedback_*.md` in the same retro commit (per `feedback_one_system_memory_in_repo.md`). Update per-task complexity score per `feedback_retro_task_complexity_score.md` (per-task `<files>/<commits>/<runtime-min>/<max-log-silence-min>`).

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-RT-r2-retro.md
modifies: []
requires:
  - task: 2
    reason: "Retro reads the Task-1 + Task-2 impl commits + validate-pending-laptop DQ entries to populate the per-role signals."
```

**No new code edits in retro task.** Retro is meta-work only.

---

## 14. Testing strategy

- **Unit (compile-time, per task):** `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full"`. Exit 0.
- **Lint (per task, uniform R6):** `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"`. Exit 0.
- **Unit tests (per task touching `compute_applied_delta` or `recompute_snapshot`):**
  - Task 1: `cargo test --workspace --features full --lib -- reputation_snapshot::tests::decay_applies_only_to_organic_events_past_half_life` (regression — v0 path preserved).
  - Task 2: `cargo test --workspace --features full --lib -- reputation_snapshot::tests` (full new + existing test set).
- **e2e execution (Task 2 only):** `cargo test --workspace --test e2e --features full` (no flag = default `false` -> v0 path -> regression check).
- **No migration round-trip** — r2 has no migrations.

## 15. Validation commands (DoD)

> **Planner-side dry-run gate (per `feedback_plan_dod_dry_run_at_write.md`):** every command below MUST be dry-run by the advisor against current HEAD before plan approval. The advisor laptop session runs the section 3.4 DoD smoke test as gate 1's pre-condition.

### 15.1 Static analysis (per task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r2-<task>-check.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.2 Lint (per task — uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r2-<task>-clippy.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.3 Unit tests (Tasks 1 + 2)

```bash
# Task 1 — re-run existing test to confirm v0 path preserved
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --lib -- reputation_snapshot::tests::decay_applies_only_to_organic_events_past_half_life > .claude/PRPs/debug/v1-RT-r2-task1-test.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0, "1 passed; 0 failed"

# Task 2 — run full new + existing test set
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --lib -- reputation_snapshot::tests > .claude/PRPs/debug/v1-RT-r2-task2-tests.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0; all tests passed (existing 4 + new 11 = 15 total)
```

The cargo-test command uses `--workspace --features full --lib` per `feedback_features_full_p_crate_incompatible.md` — never `-p lemmy_api --features full` because the workspace owns the `full` feature.

### 15.4 e2e regression (Task 2)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-RT-r2-task2-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-RT-r2-task2-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-RT-r2-task2-e2e.log"
tail -50 .claude/PRPs/debug/v1-RT-r2-task2-e2e.log
# EXPECT: E2E_EXIT_0 marker present; pre-existing e2e tests pass.
```

### 15.5 Cross-cutting verification

- [ ] R1: every `i32 <-> i64` comparison uses `i64::from(...)`, never `as` cast — `clamp_dimension_i32` uses `i64::from(value)` per section 10.5.
- [ ] R2: every governance_config read flows through the `ConfigCache` already in scope inside `recompute_snapshot`. No direct `fetch_value`.
- [ ] R3: the `if event.expires_at.is_some()` cliff guard + `if original <= 0` penalty guard remain the first two checks in `compute_applied_delta` regardless of `v1_enabled` value (Watch 8 + Watch 2 preserved).
- [ ] R4: new unit tests use `assert_eq!` only; no `.unwrap()`, no `.expect()`, no `dbg!`.
- [ ] R5: Task 0 enumerated all 10 probes explicitly.
- [ ] R6: all clippy invocations use `--workspace --features full --no-deps -- -D warnings` uniformly.
- [ ] R8: no `-p <crate> --features full` invocation in any section 13 task body or section 15 DoD command (only `--workspace --features full` per the lesson).
- [ ] Watchpoint W1 (section 18 risk row 1): `recompute_snapshot` external signature unchanged — the 5 callsites at `create_endorsement.rs:300-301`, `revoke_endorsement.rs:335-337`, `e2e.rs:3278/3778/3779` do not require edits.
- [ ] Watchpoint W2: `feature.reputation_v1_decay_enabled = false` arm preserves v0 behaviour exactly — the existing `decay_applies_only_to_organic_events_past_half_life` test passes unchanged.
- [ ] Watchpoint W3: 8 bounds keys are read only when `v1_decay_enabled = true` — the v0 arm produces snapshot sums that are byte-identical to pre-r2 sums for identical inputs.
- [ ] Watchpoint W4: `chained_halve` is symmetric across the 4 dimensions and 2 directions (positive and negative reads pick separate keys; both produce the same shift math).
- [ ] No `crates/server/tests/e2e.rs` edits in any section 13 task's `modifies:` array (avoids `feedback_junior_worker_e2e_edit_hang.md` risk).
- [ ] No new ENTRY_KIND_* consts added in r2 (`ENTRY_KIND_DECAY_KNOB_CHANGED` is pre-landed by r1 + deferred-emit per `governance-log-entry-kind-registry.md` pre-landed-const exemption; r2 does not emit it — that's v1-AD-b admin-config write handler's responsibility).
- [ ] PRD section 6 acceptance: bullet "Feature flag ... defaults `false` at v1 release; flipping `true` activates per-dimension decay; flipping `false` reverts to v0 single-half-life behaviour without restart" — confirmed by Task 2's `v0_arm_at_2x_half_life_single_halving` test against the same input the v1-arm test (`chained_halving_at_2x_half_life_quarters_delta`) uses with `v1_enabled = true`.

### 15.6 DoD per workflow (Shape G plans — not applicable)

Shape G is **SUSPENDED** per DQ #229 until 2026-06-01. v1-RT-r2 runs under the pre-Shape-G validate-pending-laptop pathway. Each task's VALIDATE block lists the explicit cargo commands run by the advisor laptop session per `advisor-orchestrator.md` section 5.2.

---

## 16. Acceptance criteria

- [ ] All 3 tasks (Task 0 + Tasks 1-2 impl + Task 3 retro) completed in dependency order. **Task 0 is verification-only (no commit).** Tasks 1-2 each produce one commit with subject `feat(rep-tuning): <description> (task N)` per brief section 4. Task 3 produces one commit with subject `docs(retro): v1-RT-r2 retro`.
- [ ] section 15.1 (cargo check `--workspace --features full`) exit 0 after every task.
- [ ] section 15.2 (cargo clippy `--workspace --features full --no-deps -- -D warnings`) exit 0 after every task.
- [ ] section 15.3 (cargo test `--lib -- reputation_snapshot::tests`) exit 0 after Tasks 1 + 2.
- [ ] section 15.4 (cargo test e2e workspace) exit 0 after Task 2 — pre-existing e2e tests pass; no regressions.
- [ ] section 15.5 (cross-cutting verification) — all 12 boxes ticked.
- [ ] section 16a stories — all stories `[done]`.
- [ ] No edits to files outside section 11 list (only `crates/api/api/src/governance/reputation_snapshot.rs` + Task 3's retro file at `.claude/PRPs/reports/v1-RT-r2-retro.md`).
- [ ] Retro committed per section 13 Task 3.
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy` flag (per `gh-pr-fork-target.md`).

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: Chained-halving decay correct under v1 flag

- **Composing tasks:** Task 1, Task 2.
- **Checkpoint command:**
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --lib -- reputation_snapshot::tests::chained > .claude/PRPs/debug/v1-RT-r2-story1-checkpoint.log 2>&1"
  echo "exit: $?"
  tail -30 .claude/PRPs/debug/v1-RT-r2-story1-checkpoint.log
  ```
- **Expected output:** 6 tests passed (5 `chained_halving_*` tests + `chained_halve_helper_saturates_at_very_old_event`; all matched by substring `chained`).
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `crates/api/api/src/governance/reputation_snapshot.rs` contains `fn chained_halve(` declaration.
  - `crates/api/api/src/governance/reputation_snapshot.rs` contains `compute_applied_delta(event: &ReputationEvent, now: DateTime<Utc>, half_life: Duration, v1_enabled: bool)` signature (with the new `v1_enabled` arg).
  - `crates/api/api/src/governance/reputation_snapshot.rs` contains `fn resolve_half_life_for_event(` declaration.

### Story 2: Feature-flag false path preserves prior behaviour

- **Composing tasks:** Task 1, Task 2.
- **Checkpoint command:**
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --lib -- reputation_snapshot::tests::v0_arm_at_2x_half_life_single_halving > .claude/PRPs/debug/v1-RT-r2-story2-checkpoint.log 2>&1"
  echo "exit: $?"
  tail -20 .claude/PRPs/debug/v1-RT-r2-story2-checkpoint.log
  ```
- **Expected output:** `1 passed; 0 failed` (the dedicated `v0_arm_at_2x_half_life_single_halving` test asserts `compute_applied_delta(organic +100, age=180d, half_life=90d, v1_enabled=false) == 50`).
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/reputation_snapshot.rs` contains both the v0 single-halving branch (`if age > half_life { original / 2 } else { original }`) AND the v1 chained-halving branch (`chained_halve(original, ...)`) — both inside the `if v1_enabled { ... } else { ... }` structure inside `compute_applied_delta`.
  - The existing test `decay_applies_only_to_organic_events_past_half_life` still passes (its 4 assertions are the v0 arm; Task 1 modified the call sites to pass `false` explicitly).

### Story 3: Bounds clamping applies only under v1 flag

- **Composing tasks:** Task 2.
- **Checkpoint command:**
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --lib -- reputation_snapshot::tests::clamp_dimension > .claude/PRPs/debug/v1-RT-r2-story3-checkpoint.log 2>&1"
  echo "exit: $?"
  tail -20 .claude/PRPs/debug/v1-RT-r2-story3-checkpoint.log
  ```
- **Expected output:** `4 passed; 0 failed` (the 4 `clamp_dimension_i32_*` tests).
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/reputation_snapshot.rs` contains `fn clamp_dimension_i32(value: i32, floor: i64, ceiling: i64) -> i32` declaration.
  - The body of `recompute_snapshot` reads `bounds.reporting_accuracy.floor`, `bounds.reporting_accuracy.ceiling`, `bounds.jury_reliability.floor`, `bounds.jury_reliability.ceiling`, `bounds.participation_consistency.floor`, `bounds.participation_consistency.ceiling`, `bounds.endorsement_strength.floor`, `bounds.endorsement_strength.ceiling` — all 8 keys, all inside a single `if v1_decay_enabled { ... }` block.

> **Verification mapping:** the advisor's `/brehon-verify` step iterates this section, runs each Story's Checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Phantoms (task complete but output absent or empty) trigger the catch-fire procedure in advisor-orchestrator.md section 5.6.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all 10 probes confirmed).
- [ ] Task 1 committed: `feat(rep-tuning): replace compute_applied_delta with chained halving (task 1)`.
- [ ] Task 2 committed: `feat(rep-tuning): wire feature flag, per-event half-life resolver, bounds clamp (task 2)`.
- [ ] section 15 validation green at every gate.
- [ ] section 16a 3 stories all `[done]`.
- [ ] Task 3 retro committed: `docs(retro): v1-RT-r2 — per-dimension chained-halving decay`.
- [ ] PR opened by BM session against `governance-v0` via `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-RT-r2`.
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-RT-r2-verify.md` shows all 3 stories ok.
- [ ] Post-merge phase branch retained for retro reads.

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| W1 — `compute_applied_delta` body change breaks one of the 5 external `recompute_snapshot` callsites | LOW | MED | Signature of `recompute_snapshot` is preserved by section 11 design; `compute_applied_delta` is private. section 15.5 box 7 verifies no external callsite edit needed. Task 0 Probe 9 confirms no concurrent PR overlaps this file. |
| W2 — Flag=false arm diverges from v0 behaviour (regression in default deployments) | LOW | HIGH | Story 2 checkpoint runs `v0_arm_at_2x_half_life_single_halving` test specifically; existing test `decay_applies_only_to_organic_events_past_half_life` re-runs verbatim. section 15.4 e2e regression catches downstream callers. |
| W3 — Bounds clamp produces dimension sum outside `[i32::MIN, i32::MAX]` (overflow) | VERY LOW | MED | `clamp_dimension_i32` performs the clamp on i64 first per section 10.5; PRD section 8 ranges (-10000, 10000) x `i64::from(i32)` cannot overflow i64. The `i32::try_from` defensive branch yields `i32::MAX` / `i32::MIN` on the unreachable overflow path. |
| W4 — `chained_halve` shift overflow on truly ancient event (age > 31 x half-life) | VERY LOW | LOW | `n.min(31) as u32` clamps the shift; for positive i32, the result saturates at 0 (correct semantics — the event has effectively decayed completely). Test `chained_halve_helper_saturates_at_very_old_event` asserts this. |
| W5 — `ConfigCache` mutable-borrow conflict between `resolve_half_life_for_event` (inside loop) and bounds-clamp (after loop) | LOW | MED | After the per-event loop scope ends, `cache` is no longer borrowed; the 8 bounds-clamp reads re-borrow `cache` fresh. The pattern matches the existing `recompute_snapshot` body (lines 240, 247, 254, 261, 268) which makes 5 sequential `get_int` calls already. Task 2 GOTCHA documents the borrow flow. |
| W6 — `cargo test --workspace --features full --lib -- reputation_snapshot::tests` runs the test from the wrong crate context | LOW | LOW | The `reputation_snapshot::tests` path resolves uniquely (only one `reputation_snapshot.rs` defines a `tests` mod) within the workspace. The `--workspace --features full` combination is the canonical form per `feedback_features_full_p_crate_incompatible.md`. If cargo picks an unintended target, Task 0 Probe 5 (clippy baseline) catches the inconsistency. |
| W7 — Parallel `v1-ship-2` lane edits `reputation_snapshot.rs` concurrently | VERY LOW | MED | Task 0 Probe 9 (gh pr list with file filter) confirms no overlap before phase start. The phase-v1-ship-2 branch (per handover) covers cohort-3 merge work, not `reputation_snapshot.rs`. |
| W8 — `feature.reputation_v1_decay_enabled` was not seeded by r1 on `governance-v0` HEAD | VERY LOW | HIGH | Task 0 Probe 8 explicitly greps the key in `config.rs` match arms. If absent, STOP and file a `kind: "blocker"` DQ before any impl. The const at line 1033 (`DEFAULT_FEATURE_REPUTATION_V1_DECAY_ENABLED: bool = false`) and the match arm at line 1279 confirm presence as of HEAD `57b1ade60`. |
| W9 — Cycle-count meta-rule (>=3 fails same `(error_class, file_basename)`) triggers on `(E0277, reputation_snapshot.rs)` because both Task 1 and Task 2 edit the same file with closely-related compile-error classes | VERY LOW | MED | Task 1's signature change is isolated to `compute_applied_delta` body + 4 test-callsite arg updates (5 sites total). Task 2's edits are additive (new tests, new helper fns, new bounds clamp block). Both pass `cargo check --workspace --features full` independently per VALIDATE. If a Task 1 fix-impl cycle fires, the section G4 classifier reads the log slice and applies the allowlist recipe; if a Task 2 fix-impl cycle fires on the same `(error_class, file_basename)` tuple, hard refusal per `advisor-orchestrator.md` section 5.3 "Cycle-count meta-rule" — re-plan required. |

---

## 19. Notes

- **PRD section 9.1 weaker-compliance posture** — the feature flag is the entirety of r2's ADR-010 compliance gate. PRD section 9.1 documents the deliberate weaker posture vs jury-mechanics-style snapshot columns. r2 ships the flag; pilot operators flip when ready.
- **Negative-direction half-life "wired but inert"** — `resolve_half_life_for_event` reads `decay.<dim>.negative_half_life_days` for `event.delta < 0`, but `compute_applied_delta`'s penalty guard `if original <= 0 { return original; }` fires BEFORE the half-life is consulted. This is correct per PRD section 5.2's preservation of "penalties never decay" in r2; a future v1+ PRD that introduces negative-decay event sources (e.g. dormancy-decay) will remove the penalty guard or split it into "penalty vs slow-negative" branches. r2 deliberately preserves the v0 invariant.
- **`ENTRY_KIND_DECAY_KNOB_CHANGED` emit deferred** — the const is in `crates/db_schema/src/source/governance/governance_log.rs` (landed by r1 task 9); the registry entry at `.claude/rules/governance-log-entry-kind-registry.md` v1-RT-r1 section names v1-RT-r2 as the emitter (`admin_config.rs`). However, r2 reads decay knobs; it does not write them. The admin-config write handler is v1-AD-b's territory. The pre-landed-const exemption in the registry's "Acceptance invariants" allows this. r2's retro should flag this for the registry maintenance pass at v1-AD-b ship-time.
- **`config.rs` ConfigCache 17-key read per recompute** — when `v1_decay_enabled = true`, each `recompute_snapshot` call reads 17 keys (1 flag + 1 legacy + 1 jury_reliability_threshold + 1 reporting_accuracy_threshold + 1 endorsement_strength_threshold + 1 jury_age_requirement + 8 bounds + N per-(dim, direction) decay keys where N <= 8 depending on which dimensions have events). The `ConfigCache` is per-recompute so the cost is at most 17 DB reads x O(1) cached lookups. Within the 15-min snapshot batch tick budget per `feedback_resource_budget_pre_queue.md`.
- **Reputation event-source classification** — r1 introduced `ReputationEventSourceType` enum on `reputation_event`; r2 does NOT branch on `source_event_type` (decay is per-(dimension, direction) only, not per-source). r3+ may add source-conditioned decay; r2 keeps source orthogonal to decay.
- **DQ entries pre-resolved:** `a3d0e9941441-009` (bounds clamp) and `a3d0e9941441-010` (per-dimension keys) both resolved at commit `57b1ade60`. No clarify-pending DQs remain on this brief.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — the algorithm is well-specified by PRD section 5.2; the bounds clamp + feature-flag arms are mechanical; the existing test suite at line 917 is the literal MIRROR for new tests. One point reserved for the cargo-test invocation form: the `--workspace --features full --lib -- reputation_snapshot::tests` path was chosen per `feedback_features_full_p_crate_incompatible.md` but the test name resolution depends on cargo's libtest selecting `lemmy_api` as the target; impl-task verifies at Task 0 Probe 5 (clippy baseline workspace).
- **Cargo budget:** 9/10 — `cargo check --workspace --features full` is the dominant cost (~5-6 GB peak), well within the 10 GB EliteDesk cap. r2 has no migrations and no e2e edits, so no compound cost; ~6 GB total peak across both impl tasks (serial).
- **Test coverage:** 8/10 — 11 new unit tests cover the chained-halving math (4 cases x 2 arms = 8) + bounds clamp (4 cases) + cliff/penalty guards (2 inherited). The e2e regression catches caller-path regression. One point reserved for testing the live config-cache borrow flow (W5) — the new `resolve_half_life_for_event` helper takes `&mut cache` and `&mut conn`; if the Rust borrow checker rejects the interleaved use, Task 2 GOTCHA documents the rework path. The borrow flow is symmetric with existing patterns at lines 240-275 so the risk is low but non-zero.
