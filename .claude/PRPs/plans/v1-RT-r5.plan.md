# Plan: v1-RT-r5 — instance-wide reputation rollup cron + admin rollup endpoint

## 1. Summary

This sub-phase materialises an **instance-wide reputation rollup** for every person who has at least one per-community `reputation_snapshot` row. A new weekly `reputation_rollup_cron` (in `scheduled_tasks.rs`) computes, per person, the equal-weighted average of that person's per-community snapshot dimensions — **excluding communities the person is banned from** — and writes the result as a `reputation_snapshot` row with `community_id IS NULL`. Each recomputation emits `ENTRY_KIND_ROLLUP_RECOMPUTED` (system pseudonym) into the governance log, and instance-wide capability flips emit `ENTRY_KIND_CAPABILITY_CHANGED` with `snapshot_community_id: null`. A new admin-only `GET /api/v4/governance/admin/reputation/rollup?person_id=N` endpoint returns `{rollup: Option<ReputationSnapshot>, contributing: Vec<ReputationSnapshot>}`. **No migration** (the table already supports `community_id IS NULL`) and **no new entry-kind const** (count stays 55; RT-r5 is the first emission site of the RT-r1-landed `ROLLUP_RECOMPUTED` const). Headline acceptance: after per-community data exists, one cron tick materialises a `community_id IS NULL` row whose dimensions equal the integer mean of the non-banned contributing snapshots, and the governance log carries the `rollup_recomputed` entry.

## 2. Source

- `.claude/PRPs/prds/v1-reputation-tuning.prd.md` §5.5 (instance-wide reputation rollup — materialised weekly cron, equal weights, banned-community exclusion, capability-flip on rollup changes), §5.2 (cron ordering — rollup after participation), §7 (`ENTRY_KIND_ROLLUP_RECOMPUTED` payload), §10 (Security — cron signs via instance key, `system` pseudonym), §11 row 5 (phase definition; dependency v1.r2 decay live — satisfied PR #150 `3b36b4e61`).
- `.claude/rules/governance-log-entry-kind-registry.md:188` — `ENTRY_KIND_ROLLUP_RECOMPUTED` row (`(pending)` marker → live this phase) + payload shape.
- ADRs: **ADR-008** (governance_log append-only hash chain — every rollup recomputation emits via `governance_log::append`), **ADR-015** (pseudonymised actor IDs — cron-batch entries use `system` pseudonym / `None`).
- Open questions: **OQ-001** (instance-wide vs per-community reputation — resolved here: rollup is the derived `reputation_snapshot WHERE community_id IS NULL` row), **OQ-V1-02** (banned-from-community rollup behaviour — lean: exclude banned communities from numerator AND denominator AND contributing count).
- Clarify DQ `81ae24440317-001` (RESOLVED, advisor-mode 2026-05-31): rollup compute path MUST be a dedicated weighted-average helper, NOT `recompute_snapshot(None)` reuse.
- Lessons: `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md`, `feedback_fix_impl_pre_locate_e2e_anchors.md`, `feedback_features_full_workspace_only.md`, `feedback_features_full_p_crate_incompatible.md`, `feedback_clippy_test_style.md`.
- Prior retros / preflight guardrails: §7 below.
- Canonical sibling plan: `.claude/PRPs/plans/v1-RT-r3.plan.md` (participation cron — closest "add a cron to scheduled_tasks.rs" structural sibling).

## 3. Problem statement

1. **No instance-wide rollup exists.** The `reputation_snapshot` table supports `community_id IS NULL` rows, but no code path writes them. PRD §5.5 requires a materialised weekly rollup. → §13 Task 1 (`compute_rollup_snapshot` + `run_rollup_batch`), Task 3 (cron registration).
2. **`ENTRY_KIND_ROLLUP_RECOMPUTED` is declared but never emitted.** The const shipped in RT-r1 and the registry names RT-r5 as its fire site, but it has no emission call site. → §13 Task 1 (emit) + Task 6 (registry flip).
3. **No admin read path for rollup data.** Operators cannot inspect a person's instance-wide rollup or the per-community snapshots that compose it. → §13 Task 2 (DTOs), Task 4 (handler + route).
4. **No e2e coverage** for the rollup cron, banned-community exclusion, the admin endpoint, or the governance-log emit. → §13 Task 5.

## 4. Solution statement

```
WEEKLY SCHEDULER TICK (scheduled_tasks.rs, AFTER participation cron, BEFORE run loop)
  └─ reputation_rollup_cron  [Task 3: ROLLUP_CRON_RUNNING guard + clokwerk registration]
       └─ run_rollup_batch(context)  [Task 1; mirrors run_snapshot_batch :481-530]
            ├─ load_rollup_candidates(pool)   → persons with ≥1 per-community snapshot
            └─ for each person:
                 ├─ compute_rollup_snapshot(conn, person_id, cache)  [Task 1; rollup-specific COMPUTE]
                 │    ├─ load per-community snapshots WHERE person_id=$1 AND community_id IS NOT NULL
                 │    ├─ exclude banned communities (sanction filter, mirrors load_person_context :736-768)
                 │    ├─ per dim: sum(dim×1)/sum(1)  [integer division, truncates toward zero]
                 │    └─ denominator 0 (all banned / none) → skip (no row, no emit)
                 ├─ upsert_snapshot(conn, &form{community_id:None,...}, now)  [SHARED write path :770]
                 ├─ governance_log::append(ROLLUP_RECOMPUTED, payload{raw person_id,...}, None=system)
                 └─ detect_capability_changes(old, new) → CAPABILITY_CHANGED(snapshot_community_id:null)

ADMIN READ (Query GET)
  GET /api/v4/governance/admin/reputation/rollup?person_id=N
   └─ admin_reputation_rollup  [Task 4; mirrors admin_reputation_stats.rs:71-82]
        ├─ is_admin(&local_user_view)?
        ├─ rollup     = reputation_snapshot WHERE person_id=$1 AND community_id IS NULL  (Option)
        └─ contributing = reputation_snapshot WHERE person_id=$1 AND community_id IS NOT NULL (Vec)
   → AdminReputationRollupResponse { rollup: Option<ReputationSnapshot>, contributing: Vec<ReputationSnapshot> }
```

## 5. Metadata

- **Phase:** `v1-RT-r5`
- **Branch:** `phase-v1-RT-r5` (cut by BM-task before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 8 (Task 0 pre-flight + Tasks 1–5 impl/e2e + Task 6 registry flip [advisor-executed] + Task 7 retro)
- **Estimated cargo budget:** ~5–6 GB peak (workspace check + e2e with `--features full`; ≤ 6 GB)
- **Forbidden-window applicability:** **BINDING** — RT-r5 is **pre-Shape-G** (Shape G suspended until 2026-06-01; today 2026-05-31). Local cargo via `validate-pending-laptop` / `validate-pending-laptop-e2e` DQ handler; forbidden windows bind any local cargo invocation.
- **Complexity score:** `8/10` — see breakdown below.

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 1 | 6 impl/registry tasks (Tasks 1–6); 6 − 5 = 1 |
| Migrations touched | +2 each | 0 | No migration (table supports `community_id IS NULL`) |
| Crates touched | +1 each | 4 | `lemmy_api`, `lemmy_routes`, `lemmy_api_common`, `lemmy_api_routes` |
| `crates/server/tests/e2e.rs` edits | +3 each | 3 | One e2e task (Task 5), 4 tests in one file |
| New ADR-affecting decisions | +2 each | 0 | OQ-001/OQ-V1-02 already leaned; no ADR supersede |
| Cargo budget peak above 6 GB | +1 per GB | 0 | Peak ~5–6 GB ≤ 6 GB |
| **Total** | — | **8** | Threshold for split-DQ: `>8` (Sonnet). 8 is NOT > 8 → no split. |

Target model is Sonnet (`sonnet-4-6`) → split-DQ fires only if score `> 8`. Score is exactly 8 → **proceed as one sub-phase, no split**.

### 5.2 Per-task complexity ceiling (Sonnet ≤ 4 files / ≤ 2 crates)

| Task | Files | Crates | Within Sonnet ceiling? |
|---|---|---|---|
| Task 1 | 1 (`reputation_snapshot.rs`) | 1 (`lemmy_api`) | ✅ 1/1 |
| Task 2 | 1 (`api_common/governance.rs`) | 1 (`lemmy_api_common`) | ✅ 1/1 |
| Task 3 | 1 (`scheduled_tasks.rs`) | 1 (`lemmy_routes`) | ✅ 1/1 |
| Task 4 | 3 (`admin_reputation_rollup.rs`, `mod.rs`, `lib.rs`) | 2 (`lemmy_api`, `lemmy_api_routes`) | ✅ 3/2 |
| Task 5 | 1 (`e2e.rs`) | 1 (`lemmy_server` test target) | ✅ 1/1 |
| Task 6 | 1 (registry md) | 0 | ✅ advisor-executed |

The endpoint was split into Task 2 (DTOs, 1 file / 1 crate) + Task 4 (handler + mod + route, 3 files / 2 crates) precisely so each task stays inside the Sonnet 4/2 ceiling. Task 4's 2-crate span (`lemmy_api` for handler+mod, `lemmy_api_routes` for the route) is irreducible — the handler and its route registration live in different crates.

## 6. Relationship to other v1-RT sub-phases

- **Depends on:** RT-r1 (schema + `ROLLUP_RECOMPUTED` const + `reputation_snapshot.community_id` Option) and RT-r2 (v1 decay live — PRD §11 row 5 dependency, PR #150 `3b36b4e61`). Both shipped.
- **Sibling pattern:** RT-r3 (participation cron — same `AtomicBool` + `RunningGuard` + clokwerk registration + governance_log emit; PR #155).
- **Followed by:** RT-r6 (carry-forward CodeRabbit fixes #19–#22, #31 — NOT in scope here).

## 7. Preflight guardrails inherited from prior phases

- **R1:** every `i32 ↔ i64` comparison uses `i64::from(...)`, never `as` cast (per `feedback_clippy_test_style.md`). The capability-bit thresholds compare `i64::from(dim) >= threshold` (mirror :350–399).
- **R5:** Task 0 enumerates ALL probes explicitly (Probes 0–11), no implicit inheritance.
- **R6:** all clippy invocations use `--no-deps` uniformly.
- **R7:** test-target compile via `cargo test --no-run ... --test e2e` after any struct/re-export change.
- **Linux `.sh` wrapper form** — Junior daemon is Ubuntu Server; use `./scripts/brehon/cargo-<verb>.sh <args>` (NOT the Windows `.bat` form).
- **`--features full` discipline** — governance code is behind `#[cfg(feature = "full")]`; always `--workspace --features full` (NEVER `-p <crate> --features full` per `feedback_features_full_p_crate_incompatible.md`).
- **e2e anchor uniqueness** — for every `old_string` Edit anchor in Task 5, confirm `grep -c '<anchor>' crates/server/tests/e2e.rs` == 1 before authoring (per `feedback_fix_impl_pre_locate_e2e_anchors.md`).
- **System pseudonym** — cron-batch `ROLLUP_RECOMPUTED` emit passes `None` as the `actor_pseudonym` arg to `governance_log::append` (ADR-015 / PRD §10).

## 8. Flow design

**Before:** `scheduled_tasks.rs` registers 5 crons (snapshot, appeal-window, grace-check, fed-replay-cleanup, participation), then the `loop { scheduler.run_pending().await; sleep(1000ms) }` run loop. No rollup row is ever written. `ENTRY_KIND_ROLLUP_RECOMPUTED` has zero call sites. No `admin/reputation/rollup` route.

**After (weekly tick):**
```
scheduler tick
  → participation_cron (existing, ends ~:516)
  → reputation_rollup_cron (NEW, Task 3)
       → run_rollup_batch (NEW, Task 1)
            → load_rollup_candidates → [person_ids]
            → compute_rollup_snapshot (NEW, Task 1) → ReputationSnapshotInsertForm{community_id:None}
            → upsert_snapshot (EXISTING :770)
            → governance_log::append(ROLLUP_RECOMPUTED, None)  (EXISTING :256)
            → detect_capability_changes (EXISTING :142) → CAPABILITY_CHANGED(null)
  → run loop (unchanged, registration MUST precede it)
```

**After (admin read):**
```
GET /admin/reputation/rollup?person_id=N
  → admin_reputation_rollup (NEW, Task 4)
       → is_admin? → load rollup row (community_id IS NULL) + contributing (community_id IS NOT NULL)
       → AdminReputationRollupResponse (NEW DTO, Task 2)
```

§11 (files to change) follows directly: `reputation_snapshot.rs` (compute+batch), `scheduled_tasks.rs` (cron), `api_common/governance.rs` (DTOs), `admin_reputation_rollup.rs`+`mod.rs`+`lib.rs` (endpoint), `e2e.rs` (tests), registry md (flip).

## 9. Mandatory reading

### Schema / type definitions
- `crates/db_schema/src/source/governance/reputation_snapshot.rs:1-47` — `ReputationSnapshot` (NO `Copy`/`Default`/`Hash`; HAS `PartialEq`/`Eq` + ts-rs derive) and `ReputationSnapshotInsertForm` (`Clone, Default` + Insertable/AsChangeset). Confirms response-DTO derive set.
- `crates/db_schema/src/source/governance/governance_log.rs:121,215,256` — `ENTRY_KIND_CAPABILITY_CHANGED`, `ENTRY_KIND_ROLLUP_RECOMPUTED`, `append(pool, entry_kind, payload, actor_pseudonym)` signature.

### Existing patterns (MIRROR refs — see §10)
- `crates/api/api/src/governance/reputation_snapshot.rs:481-530` — `run_snapshot_batch` (control-flow mirror for `run_rollup_batch`).
- `crates/api/api/src/governance/reputation_snapshot.rs:350-399` — capability-bit computation + flip emit.
- `crates/api/api/src/governance/reputation_snapshot.rs:736-768` — `load_person_context` (canonical banned-detection sanction filter).
- `crates/api/api/src/governance/reputation_snapshot.rs:770` — `upsert_snapshot` (SHARED write path).
- `crates/routes/src/utils/scheduled_tasks.rs:105-114,471-516,518` — participation guard + registration body + run-loop position.
- `crates/api/api/src/governance/admin_reputation_stats.rs:24-25,71-82` — admin-read handler head (imports + `is_admin` first line + `Query<...>`).
- `crates/api/api_common/src/governance.rs:34,369,423` — `AdminReputationStats` request + `AdminReputationStatsResponse` derive stacks.
- `crates/api/routes/src/lib.rs:40,502` — admin-endpoint import + route registration.

### ANTI-MIRROR (do NOT copy)
- `crates/api/api/src/governance/reputation_snapshot.rs:222` — `recompute_snapshot`; and `:236`/`:682` — `load_live_events` event-sum (filters `community_id IS NULL`). This is **event-sourced instance recompute**, categorically different from §5.5's weighted average of per-community snapshot VALUES. **Must NOT be reused for the rollup compute** (clarify DQ `81ae24440317-001`).

### Adjacent test fixtures
- `crates/server/tests/e2e.rs:17592-17648` — `participation_activity_cron_emits_plus_one_per_active_user` (the canonical cron-e2e fixture: `EnvVarGuard::set`, `boot_context`, `seed_named_community`, `run_activity_batch`, governance_log count assertion, ISO-week-boundary xfail guard). Mirror its error-shape and seed helpers verbatim.

### Config knobs
- `crates/api/api/src/governance/config.rs:1027,1030,1184,1278` — `job.rollup_interval_days` (default 7), `job.rollup_equal_weights` (default true) + lookup arms.

### Lessons (gate §13 tasks)
- `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` (Task 5 e2e).
- `feedback_fix_impl_pre_locate_e2e_anchors.md` (Task 5 — ≥2 e2e edits).
- `feedback_features_full_workspace_only.md` + `feedback_features_full_p_crate_incompatible.md` (any `#[cfg(feature = "full")]` gate).
- `feedback_clippy_test_style.md` (R1 i64::from).

## 10. Patterns to mirror

### 10.1 `run_rollup_batch` mirrors `run_snapshot_batch`; `compute_rollup_snapshot` is rollup-specific

**Mirror (batch control flow):** `crates/api/api/src/governance/reputation_snapshot.rs:481-530`

```rust
// run_snapshot_batch shape (mirror for run_rollup_batch):
let pool = &mut context.pool();
let mut cache = ConfigCache::new();
let chunk_size = config::get_int(&mut cache, pool, Scope::Instance, "job.snapshot_batch_chunk_size").await?;
let chunk_size_usize = usize::try_from(chunk_size).unwrap_or(/*default*/);
let candidates = load_rollup_candidates(pool).await?;   // persons w/ ≥1 per-community snapshot
let mut outcome = RollupBatchOutcome::default();
if candidates.is_empty() { info!("rollup batch: no candidates"); return Ok(outcome); }
for chunk in candidates.chunks(chunk_size_usize) {
    let conn = &mut get_conn(pool).await?;
    for &person_id in chunk { /* compute_rollup_snapshot + upsert + emit */ }
}
info!("rollup batch complete: {} rows", outcome.rows_written);
Ok(outcome)
```

**ANTI-MIRROR:** do NOT call `recompute_snapshot(conn, person_id, None, cache)` — it event-sums `reputation_event` rows (`load_live_events` :682), not per-community snapshot VALUES.

**`compute_rollup_snapshot(conn, person_id, cache)` rollup-specific COMPUTE (NEW):**
```rust
// 1. load per-community snapshots
let per_community: Vec<ReputationSnapshot> = reputation_snapshot::table
    .filter(reputation_snapshot::person_id.eq(person_id))
    .filter(reputation_snapshot::community_id.is_not_null())
    .load(conn).await?;
// 2. exclude banned communities (mirror load_person_context :736-768 sanction filter)
//    a community is excluded if an active sanction has target_community_id == that community.
// 3. equal weights (job.rollup_equal_weights default true): weight = 1 per non-banned community.
//    denominator = count(non-banned contributing snapshots).
//    if denominator == 0 → skip (no rollup row, no emit; avoids divide-by-zero).
// 4. per dimension: sum(dim * 1) / denominator   — INTEGER division (i32), truncates toward zero.
//    e2e MUST compute the expected mean with the SAME integer truncation.
```

### 10.2 Capability bits + flip emit (`:350-399`) — rollup variant

**Mirror:** `crates/api/api/src/governance/reputation_snapshot.rs:350-399`

```rust
let jury_eligible = i64::from(jury_reliability) >= threshold_jury_reliability
    && account_age_days >= jury_age_requirement_days
    && active_sanctions == 0;
let trusted_reporter = i64::from(reporting_accuracy) >= threshold_reporting_accuracy;
let can_sponsor = i64::from(endorsement_strength) >= threshold_endorsement_strength;
let form = ReputationSnapshotInsertForm {
    person_id, community_id, /* 4 dims */, jury_eligible, trusted_reporter, can_sponsor };
let new_snapshot = upsert_snapshot(conn, &form, now).await?;
// flip detection:
let changes = detect_capability_changes(old_snapshot.as_ref(), &new_snapshot);
// CAPABILITY_CHANGED payload — snapshot_community_id falls out as null naturally for rollup:
json!({ "dimension_flipped": change.dimension.as_str(), "direction": change.direction.as_str(),
        "snapshot_community_id": community_id.map(|c| c.0) });  // community_id=None → null
```

**Rollup variant:** `community_id = None` (so `snapshot_community_id` serialises to `null`). `active_sanctions` is the **instance-wide** count: `load_person_context(conn, person_id, None).await?.1` (the `None` community-filter arm counts ALL active sanctions on the person).

### 10.3 Cron guard + registration (`:105-114`, `:471-516`)

**Mirror:** `crates/routes/src/utils/scheduled_tasks.rs:105-114` (guard) + `:471-516` (registration)

```rust
// module scope (mirror PARTICIPATION_CRON_RUNNING :108-114):
static ROLLUP_CRON_RUNNING: AtomicBool = AtomicBool::new(false);
struct RollupCronRunningGuard;
impl Drop for RollupCronRunningGuard {
    fn drop(&mut self) { ROLLUP_CRON_RUNNING.store(false, Ordering::Release); }
}
// registration (mirror participation :471-516) — AFTER participation block, BEFORE run loop :518:
let context_rollup = context.reset_request_count();
let rollup_pool = &mut context.pool();
let rollup_interval = get_int(&mut ConfigCache::new(), rollup_pool, Scope::Instance,
    "job.rollup_interval_days").await.unwrap_or(7).max(1);
let rollup_days = u32::try_from(rollup_interval).unwrap_or(7);
scheduler.every(CTimeUnits::days(rollup_days)).run(move || {
    let context = context_rollup.reset_request_count();
    async move {
        if std::env::var("BREHON_DISABLE_ROLLUP_JOB").as_deref() == Ok("1") { return; }
        if ROLLUP_CRON_RUNNING.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
            warn!("rollup cron already running; skipping tick"); return;
        }
        let _guard = RollupCronRunningGuard;
        lemmy_api::governance::reputation_snapshot::run_rollup_batch(&context).await
            .inspect_err(|e| warn!("rollup batch error: {e}")).ok();
    }
});
```

**GOTCHA:** the registration MUST be placed **before** the `loop { scheduler.run_pending().await; ... }` at `:518`. A cron registered after the run loop never fires.

### 10.4 `ROLLUP_RECOMPUTED` emit (system pseudonym, raw person_id)

```rust
governance_log::append(
    &mut (&mut *conn).into(),
    ENTRY_KIND_ROLLUP_RECOMPUTED,
    json!({
        "person_id": person_id.0,                  // RAW person id (not pseudonym) per registry payload
        "contributing_community_count": non_banned_count,  // non-banned only (OQ-V1-02)
        "rollup_dimensions": { "reporting_accuracy": ra, "jury_reliability": jr,
                               "participation_consistency": pc, "endorsement_strength": es },
        "recomputed_at": now.to_rfc3339(),
    }),
    None,   // system pseudonym (cron-batch, ADR-015 / PRD §10)
).await?;
```

### 10.5 Admin handler head (`admin_reputation_stats.rs:71-82`)

**Mirror:** `crates/api/api/src/governance/admin_reputation_stats.rs:24-25,71-82`

```rust
use crate::governance::config::{ConfigCache, Scope, get_int};   // (rollup may not need get_int)
use actix_web::web::{Data, Json, Query};

pub async fn admin_reputation_rollup(
    Query(data): Query<AdminReputationRollup>,
    context: Data<LemmyContext>,
    local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminReputationRollupResponse>> {
    is_admin(&local_user_view)?;              // FIRST line
    let mut pool = context.pool();
    let conn = &mut get_conn(&mut pool).await?;
    let person_id = data.person_id;
    // rollup = reputation_snapshot WHERE person_id=$1 AND community_id IS NULL (Option, first())
    // contributing = reputation_snapshot WHERE person_id=$1 AND community_id IS NOT NULL (Vec, load())
    Ok(Json(AdminReputationRollupResponse { rollup, contributing }))
}
```

GET endpoint takes `person_id` via `Query<...>` (query parameter), NOT a JSON body.

### 10.6 DTO derive stacks (`api_common/governance.rs`)

**Mirror:** `crates/api/api_common/src/governance.rs:34,369,423`

```rust
// NEW import required (api_common/governance.rs has NO existing ReputationSnapshot import):
use lemmy_db_schema::source::governance::reputation_snapshot::ReputationSnapshot;

// Request — has a Copy field (PersonId), no Option → KEEP full stack (mirror AdminReputationStats :34/:369):
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminReputationRollup { pub person_id: PersonId }

// Response — contains ReputationSnapshot (NOT Copy/Default/Hash) → DROP Copy/Default/Hash:
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminReputationRollupResponse {
    pub rollup: Option<ReputationSnapshot>,
    pub contributing: Vec<ReputationSnapshot>,
}
```

`ReputationSnapshot` HAS the ts-rs derive (so the response DTO's `ts_rs::TS` compiles) but lacks `Copy`/`Default`/`Hash` — hence the response drops them. `skip_serializing_none` is harmless on the response (`rollup` is `Option`).

## 11. Files to change

**`lemmy_api` (`crates/api/api`):**
- `src/governance/reputation_snapshot.rs` — add `compute_rollup_snapshot`, `run_rollup_batch`, `load_rollup_candidates`, `RollupBatchOutcome` (Task 1).
- `src/governance/admin_reputation_rollup.rs` — NEW admin handler (Task 4).
- `src/governance/mod.rs` — add `pub mod admin_reputation_rollup;` (Task 4).

**`lemmy_routes` (`crates/routes`):**
- `src/utils/scheduled_tasks.rs` — `ROLLUP_CRON_RUNNING` guard + `reputation_rollup_cron` registration after participation, before run loop (Task 3).

**`lemmy_api_common` (`crates/api/api_common`):**
- `src/governance.rs` — `AdminReputationRollup` + `AdminReputationRollupResponse` DTOs + `ReputationSnapshot` import (Task 2).

**`lemmy_api_routes` (`crates/api/routes`):**
- `src/lib.rs` — `admin_reputation_rollup` import (near :40) + `.route("/reputation/rollup", get().to(admin_reputation_rollup))` in admin scope (near :502) (Task 4).

**e2e:**
- `crates/server/tests/e2e.rs` — 4 tests (Task 5).

**Meta (advisor-executed):**
- `.claude/rules/governance-log-entry-kind-registry.md:188` — drop `(pending)`, name `reputation_rollup_cron` (Task 6).

**Struct-field add: enumerate all callsites** — N/A. No field is added to any existing public struct (`ReputationSnapshotInsertForm` is constructed fresh in Task 1; the two new DTOs are new types with no external callers until their handler).

## 12. NOT building in v1-RT-r5

- **Per-community weighted rollup** — deferred to v1+; `job.rollup_equal_weights` defaults `true` (equal weights only).
- **New migration / index on `community_id IS NULL`** — deferred; table already supports NULL rows. If a planner-time read suggested an index is needed, that would be a `kind:"blocker"` DQ — none filed (the cron is weekly batch, not a hot read path).
- **Cross-instance reputation portability** — v2/v3 (PRD §7 federation).
- **Admin override of reputation events** — v2 (PRD §10).
- **New entry-kind const** — `ROLLUP_RECOMPUTED` shipped in RT-r1; count stays 55.
- **RT-r2/r3/r4 edits** — no touching `compute_applied_delta`, `participation_cron`, `create_endorsement.rs`, or `admin_sponsor_allowlist.rs`.

---

## 13. Step-by-step tasks

> **Cohort sequencing (advisor-side):** Cohort A = {Task 1, Task 2} (both `requires:[]`, no file overlap → dispatch together). Cohort B = {Task 3, Task 4} (Task 3 `requires` Task 1; Task 4 `requires` Task 2 → dispatch together AFTER cohort A merges). Tasks 0, 5, 6, 7 are barriers. Pre-Shape-G: each impl task ends with edit + commit + `validate-pending-laptop` DQ entry; advisor runs the §15 cargo locally.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment ready for `v1-RT-r5`; confirm branch is `phase-v1-RT-r5`; confirm RT-r1/r2/r3 deliverables intact; confirm clippy baseline clean; confirm no concurrent PR touches §11 files.

**Probes (Linux `.sh` form — R5: enumerate ALL):**

```bash
# Probe 0 — Docker daemon (e2e needs testcontainers)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }
# Probe 1 — wrapper honors -p
./scripts/brehon/cargo-check.sh -p lemmy_utils > .claude/audit-cargo-check-p.log 2>&1; tail -20 .claude/audit-cargo-check-p.log
# Probe 2 — feature flag activation
./scripts/brehon/cargo-check.sh -p lemmy_db_schema --features full > .claude/audit-cargo-check-features.log 2>&1; tail -20 .claude/audit-cargo-check-features.log
# Probe 3 — cargo-test wrapper honors target selection
./scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server > .claude/audit-cargo-test.log 2>&1; tail -20 .claude/audit-cargo-test.log
# Probe 4 — wrappers fail loud on bogus feature (exit-code propagation; NEGATIVE)
./scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-test-negative.log 2>&1; echo "cargo-test.sh exit: $?"
./scripts/brehon/cargo-check.sh -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1; echo "cargo-check.sh exit: $?"
# Probe 5 — clippy baseline clean on workspace (expectation: exit 0)
./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/audit-clippy-baseline.log 2>&1; echo "exit: $?"; tail -20 .claude/audit-clippy-baseline.log
# Probe 6 — ROLLUP_RECOMPUTED const present
grep -n 'ENTRY_KIND_ROLLUP_RECOMPUTED' crates/db_schema/src/source/governance/governance_log.rs
# Probe 7 — const count is 55 (no new const this phase)
grep -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs   # EXPECT 55
# Probe 8 — reputation_snapshot.community_id is Option
grep -n 'community_id: Option<CommunityId>' crates/db_schema/src/source/governance/reputation_snapshot.rs
# Probe 9 — config knobs seeded
grep -n 'job.rollup_interval_days\|job.rollup_equal_weights' crates/api/api/src/governance/config.rs
# Probe 10 — participation guard present (cron sibling intact)
grep -n 'PARTICIPATION_CRON_RUNNING\|ParticipationCronRunningGuard' crates/routes/src/utils/scheduled_tasks.rs
# Probe 11 — concurrent-PR check (no open PR touches §11 files)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("scheduled_tasks.rs|reputation_snapshot.rs|api_common/src/governance.rs|api/routes/src/lib.rs|governance/mod.rs|tests/e2e.rs")) | {number, title, headRefName}'
# EXPECT: empty
```

**EXPECT block:**
- Probes 0–3, 5–10 exit/behave per inline comment (Probe 5 exit 0; Probe 7 == 55).
- Probe 4 BOTH lines NON-ZERO (negative — exit-code propagation).
- Probe 11 empty output.

**No commit at Task 0.**

### Task 1 [P]: rollup compute + batch in `reputation_snapshot.rs`

**ACTION:** add `compute_rollup_snapshot`, `run_rollup_batch`, `load_rollup_candidates`, and `RollupBatchOutcome` to `reputation_snapshot.rs`; compute equal-weighted per-dimension integer mean of non-banned per-community snapshots, write via shared `upsert_snapshot` with `community_id=None`, emit `ROLLUP_RECOMPUTED` (None pseudonym) + `CAPABILITY_CHANGED`(null).

**FILES:**
```yaml
creates: []
modifies:
  - crates/api/api/src/governance/reputation_snapshot.rs   # +compute_rollup_snapshot, +run_rollup_batch, +load_rollup_candidates, +RollupBatchOutcome
requires: []
```

**IMPLEMENT (file 1 of 1):** in `crates/api/api/src/governance/reputation_snapshot.rs`: add `RollupBatchOutcome` (mirror `SnapshotBatchOutcome` :127); `load_rollup_candidates(pool) -> Vec<PersonId>` (distinct `person_id` where `community_id IS NOT NULL`); `compute_rollup_snapshot(conn, person_id, cache)` per §10.1 (load per-community, exclude banned via §10.2 sanction filter, integer mean, denominator-0 skip, capability bits per §10.2, `upsert_snapshot` with `community_id=None`, emit §10.4 + flip §10.2); `run_rollup_batch(context)` per §10.1 control flow. **Instance-wide `active_sanctions` via `load_person_context(conn, person_id, None).await?.1`.**

**MIRROR:** `reputation_snapshot.rs:481-530` (batch), `:350-399` (capability+flip), `:736-768` (sanction filter), `:770` (upsert).

**ANTI-MIRROR:** do NOT call `recompute_snapshot` (:222) — event-sum, wrong computation.

**GOTCHA:** integer division (`i32`) truncates toward zero — the e2e mean assertion MUST compute identically. Denominator 0 (all communities banned, or none) → return early, write no row, emit nothing (no divide-by-zero, no orphan rollup row). Silent-wrong-number risk: no compile error catches a wrong mean — Story 1's e2e assertion is the catch.

**VALIDATE:** impl writes `validate-pending-laptop` DQ entry with `commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`; advisor runs locally; EXPECT exit 0.

### Task 2 [P]: rollup DTOs in `api_common/governance.rs`

**ACTION:** add `AdminReputationRollup` (request) + `AdminReputationRollupResponse` (response) DTOs and the `ReputationSnapshot` import.

**FILES:**
```yaml
creates: []
modifies:
  - crates/api/api_common/src/governance.rs   # +2 DTOs, +ReputationSnapshot import
requires: []
```

**IMPLEMENT (file 1 of 1):** in `crates/api/api_common/src/governance.rs`: add `use lemmy_db_schema::source::governance::reputation_snapshot::ReputationSnapshot;`; add the two DTOs with the derive stacks from §10.6 (request keeps `Copy`/`Hash`; response DROPS `Copy`/`Default`/`Hash`).

**MIRROR:** `api_common/governance.rs:34,369` (request stack), `:423` (response — but DROP Copy/Default/Hash since `ReputationSnapshot` lacks them).

**GOTCHA:** `api_common/governance.rs` currently has NO `ReputationSnapshot` import — must add it. `ReputationSnapshot` is NOT `Copy`/`Default`/`Hash` → a response derive copying `AdminReputationStatsResponse`'s `Copy` will fail to compile.

**VALIDATE:** `validate-pending-laptop` `["./scripts/brehon/cargo-check.sh --workspace --features full"]`; EXPECT exit 0.

### Task 3 [P]: rollup cron registration in `scheduled_tasks.rs`

**ACTION:** add `ROLLUP_CRON_RUNNING` AtomicBool + `RollupCronRunningGuard` + `reputation_rollup_cron` clokwerk registration, after participation, before the run loop.

**FILES:**
```yaml
creates: []
modifies:
  - crates/routes/src/utils/scheduled_tasks.rs   # +ROLLUP_CRON_RUNNING guard, +reputation_rollup_cron registration
requires:
  - task: 1
    reason: registration calls lemmy_api::governance::reputation_snapshot::run_rollup_batch (created in Task 1)
```

**IMPLEMENT (file 1 of 1):** in `crates/routes/src/utils/scheduled_tasks.rs`: add the module-scope guard (mirror `:108-114`); add the registration block (mirror participation `:471-516`) AFTER the participation block and BEFORE the run loop `:518`. Env-disable var `BREHON_DISABLE_ROLLUP_JOB`. Cadence from `job.rollup_interval_days` (default 7, `.max(1)`).

**MIRROR:** `scheduled_tasks.rs:108-114` (guard), `:471-516` (registration body), per §10.3.

**GOTCHA:** registration MUST precede the run loop at `:518` — a cron registered after never fires.

**VALIDATE:** `validate-pending-laptop` `["./scripts/brehon/cargo-check.sh --workspace --features full"]`; EXPECT exit 0.

### Task 4 [P]: admin rollup endpoint (handler + mod + route)

**ACTION:** create `admin_reputation_rollup.rs` handler; register the module in `mod.rs`; register the route in `lib.rs`.

**FILES:**
```yaml
creates:
  - crates/api/api/src/governance/admin_reputation_rollup.rs
modifies:
  - crates/api/api/src/governance/mod.rs        # +pub mod admin_reputation_rollup;
  - crates/api/routes/src/lib.rs                # +import, +.route("/reputation/rollup", ...)
requires:
  - task: 2
    reason: handler returns AdminReputationRollupResponse and takes AdminReputationRollup (created in Task 2)
```

**IMPLEMENT (file 1 of 3):** create `crates/api/api/src/governance/admin_reputation_rollup.rs` per §10.5 (Query GET, `is_admin` first, load rollup `community_id IS NULL` via `.first()` Option + contributing `community_id IS NOT NULL` via `.load()` Vec).
**IMPLEMENT (file 2 of 3):** in `crates/api/api/src/governance/mod.rs`, add `pub mod admin_reputation_rollup;` (alphabetical, before `pub mod admin_reputation_stats;` :23).
**IMPLEMENT (file 3 of 3):** in `crates/api/routes/src/lib.rs`, add `admin_reputation_rollup::admin_reputation_rollup,` import near :40 and `.route("/reputation/rollup", get().to(admin_reputation_rollup))` in the admin scope near :502.

**MIRROR:** `admin_reputation_stats.rs:24-25,71-82` (handler), `mod.rs:23` (module decl), `lib.rs:40,502` (import+route).

**GOTCHA:** route path is **PRD-literal slash** `/reputation/rollup` (PRD §5.5), which diverges from the sibling's hyphen form `/reputation-stats` (lib.rs:502). This is intentional — file a `kind:"log"` DQ noting the convention divergence (see §19). `crates/api/routes/src/lib.rs` (`lemmy_api_routes`) is DISTINCT from `crates/routes` (`lemmy_routes`) — do not edit the wrong lib.

**VALIDATE:** `validate-pending-laptop` `["./scripts/brehon/cargo-check.sh --workspace --features full"]`; EXPECT exit 0.

### Task 5: e2e coverage (barrier)

**ACTION:** add 4 e2e tests to `crates/server/tests/e2e.rs` per §16a stories.

**FILES:**
```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # +4 tests: rollup mean, banned exclusion, admin endpoint, ROLLUP_RECOMPUTED emit
requires:
  - task: 1
    reason: tests invoke run_rollup_batch
  - task: 2
    reason: admin-endpoint test deserialises AdminReputationRollupResponse
  - task: 4
    reason: admin-endpoint test hits GET /admin/reputation/rollup
```

**IMPLEMENT (file 1 of 1):** in `crates/server/tests/e2e.rs`, add 4 `#[tokio::test(flavor = "multi_thread")]` tests mirroring the participation-cron fixture `:17592-17648` (`EnvVarGuard::set("BREHON_DISABLE_ROLLUP_JOB", "1")` where appropriate to control timing, `boot_context`, `seed_named_community`, seed per-community snapshots, invoke `run_rollup_batch`, assert on `reputation_snapshot WHERE community_id IS NULL` + `governance_log`). Compute expected mean with integer truncation matching the helper. Mirror the sibling module's error-shape case verbatim (per `feedback_lemmy_error_no_std_error.md`). Pre-locate all `old_string` anchors; confirm `grep -c '<anchor>' crates/server/tests/e2e.rs` == 1 each.

**MIRROR:** `crates/server/tests/e2e.rs:17592-17648`.

**GOTCHA:** `LemmyError` does NOT implement `std::error::Error` — use `LemmyResult<()>` return + `?`, never `Box<dyn Error>` (per `feedback_lemmy_error_no_std_error.md`). ISO-week-boundary xfail guard if any test depends on week-relative seeding.

**VALIDATE:** `validate-pending-laptop-e2e` `["./scripts/brehon/cargo-test.sh --workspace --features full --test e2e"]` (and a `--no-run` compile gate first); advisor picks local-vs-dispatch per user gate 4; EXPECT 4 passed.

### Task 6: Registry-row flip (ADVISOR-EXECUTED)

**ACTION:** in `.claude/rules/governance-log-entry-kind-registry.md:188`, drop the `(pending)` marker and name the real fire site `scheduled_tasks.rs::reputation_rollup_cron`.

**FILES:**
```yaml
creates: []
modifies:
  - .claude/rules/governance-log-entry-kind-registry.md   # :188 — drop (pending)
requires:
  - task: 1
    reason: confirms emit call site exists
  - task: 3
    reason: confirms cron registration exists
```

**EXECUTED BY ADVISOR** — Junior planner/impl are file-ownership-blocked from `.claude/rules/**`. Const count stays **55** (this only flips an existing row from pending to live; no new const).

### Task 7: Retro

**Goal:** author retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM) with signals + lessons. Promote any new lessons to `.claude/lessons/feedback_*.md` in the same retro commit. Include per-task complexity scores (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`).

---

## 14. Testing strategy

- **Unit (compile-time):** `./scripts/brehon/cargo-check.sh --workspace --features full`
- **Lint:** `./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings`
- **Test target compile:** `./scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e` (R7 — Tasks 1, 2, 4 touch structs/re-exports)
- **e2e execution:** `./scripts/brehon/cargo-test.sh --workspace --features full --test e2e <test_name>` (Task 5)
- **Migration round-trip:** N/A (no migration)

---

## 15. Validation commands (DoD)

> Pre-Shape-G: impl-task writes `validate-pending-laptop` / `validate-pending-laptop-e2e` DQ entries; advisor runs these locally (forbidden-window BINDING). Linux `.sh` form.

### 15.1 Static analysis (per task)
```bash
./scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-RT-r5-<task>-check.log 2>&1
echo "exit: $?"   # EXPECT 0
```

### 15.2 Lint (per task — uniform R6)
```bash
./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r5-<task>-clippy.log 2>&1
echo "exit: $?"   # EXPECT 0
```

### 15.3 Test-target compile (R7 — Tasks 1, 2, 4)
```bash
./scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e > .claude/PRPs/debug/v1-RT-r5-<task>-testcompile.log 2>&1
echo "exit: $?"   # EXPECT 0
```

### 15.4 e2e execution (Task 5)
```bash
./scripts/brehon/cargo-test.sh --workspace --features full --test e2e > .claude/PRPs/debug/v1-RT-r5-e2e.log 2>&1
echo "exit: $?"   # EXPECT 0; 4 new tests pass, pre-existing pass
```

### 15.5 Cross-cutting verification
- [ ] Every rollup emit calls `governance_log::append(...)` (not direct INSERT) — ADR-008
- [ ] R1: every i32↔i64 comparison uses `i64::from(...)`, never `as` cast
- [ ] R5: Task 0 enumerated all probes (0–11)
- [ ] R6: all clippy invocations use `--no-deps` uniformly
- [ ] Rollup compute uses dedicated `compute_rollup_snapshot`, NOT `recompute_snapshot` (clarify DQ `81ae24440317-001`)
- [ ] Rollup writes via SHARED `upsert_snapshot` with `community_id=None`
- [ ] Banned communities excluded from numerator AND denominator AND `contributing_community_count` (OQ-V1-02)
- [ ] Denominator-0 → skip emit (no rollup row, no divide-by-zero)
- [ ] `ROLLUP_RECOMPUTED` emit passes `None` (system pseudonym, ADR-015) and RAW `person_id`
- [ ] `CAPABILITY_CHANGED` for rollup carries `snapshot_community_id: null`
- [ ] Cron registered AFTER participation, BEFORE the run loop
- [ ] Response DTO drops `Copy`/`Default`/`Hash`; request keeps them
- [ ] No new entry-kind const (count stays 55); no migration
- [ ] Route path is PRD-literal `/reputation/rollup`

---

## 16. Acceptance criteria

- [ ] All 8 tasks completed in dependency order
- [ ] §15.1 (cargo check) exit 0 after every task
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after every task
- [ ] §15.3 (cargo test --no-run) exit 0 after Tasks 1, 2, 4
- [ ] §15.4 (e2e) — 4 new tests pass; pre-existing pass
- [ ] §15.5 (cross-cutting) — all boxes ticked
- [ ] §16a stories — all `[done]`
- [ ] No edits to files outside §11 list
- [ ] Retro committed per §13 Task 7
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: Weekly cron materialises an instance-wide rollup row as the integer mean of contributing snapshots
- **Composing tasks:** Task 1, Task 3
- **Checkpoint command:** `./scripts/brehon/cargo-test.sh --workspace --features full --test e2e rollup_cron_materialises_instance_wide_mean`
- **Expected output:** `1 passed; 0 failed`
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/reputation_snapshot.rs` contains `compute_rollup_snapshot` + `run_rollup_batch`
  - `crates/routes/src/utils/scheduled_tasks.rs` contains `ROLLUP_CRON_RUNNING` + `reputation_rollup_cron`
  - after seeding per-community snapshots [1,3,5] for a dim, the `community_id IS NULL` row's dim == 3 (integer mean)

### Story 2: Banned-from communities are excluded from the rollup
- **Composing tasks:** Task 1, Task 5
- **Checkpoint command:** `./scripts/brehon/cargo-test.sh --workspace --features full --test e2e rollup_excludes_banned_communities`
- **Expected output:** `1 passed; 0 failed`
- **Brief-Scope outputs to verify:**
  - with snapshots [1,5] where the person is banned from the community holding 5, the rollup dim == 1 and `contributing_community_count` == 1

### Story 3: Admin endpoint returns rollup + contributing; rejects non-admin
- **Composing tasks:** Task 2, Task 4, Task 5
- **Checkpoint command:** `./scripts/brehon/cargo-test.sh --workspace --features full --test e2e admin_reputation_rollup_endpoint`
- **Expected output:** `1 passed; 0 failed`
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/admin_reputation_rollup.rs` exists, non-empty, `is_admin` first line
  - `crates/api/api_common/src/governance.rs` contains `AdminReputationRollup` + `AdminReputationRollupResponse`
  - `crates/api/routes/src/lib.rs` contains `.route("/reputation/rollup", ...)`
  - admin caller gets `{rollup, contributing}`; non-admin gets rejection

### Story 4: `ROLLUP_RECOMPUTED` governance-log entry emitted after cron
- **Composing tasks:** Task 1, Task 3, Task 5
- **Checkpoint command:** `./scripts/brehon/cargo-test.sh --workspace --features full --test e2e rollup_emits_governance_log_entry`
- **Expected output:** `1 passed; 0 failed`
- **Brief-Scope outputs to verify:**
  - after `run_rollup_batch`, `governance_log` has a `rollup_recomputed` entry with raw `person_id` + `contributing_community_count` + `rollup_dimensions` + `recomputed_at`

> **Verification mapping:** `/brehon-verify` iterates this section, runs each Checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0–11 confirmed)
- [ ] Tasks 1–6 committed (Task 6 advisor-executed)
- [ ] §15 validation green at every gate
- [ ] §16a stories all `[done]`
- [ ] Retro committed (Task 7)
- [ ] PR opened by BM session against `governance-v0`
- [ ] CodeRabbit review complete, findings triaged
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-RT-r5-verify.md` shows all stories ✓
- [ ] Post-merge phase branch retained for retro reads

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Impl reuses `recompute_snapshot(None)` (event-sum) instead of weighted average | MED | HIGH | §10.1 ANTI-MIRROR callout + clarify DQ `81ae24440317-001` + §15.5 checklist box; Story 1 mean assertion catches wrong number |
| e2e anchor collision (>1 match for `old_string`) | MED | MED | §7 + Task 5 uniqueness gate (`grep -c == 1`); `feedback_fix_impl_pre_locate_e2e_anchors.md` |
| Cron registered after the run loop (never fires) | LOW | HIGH | §10.3 GOTCHA + §15.5 checklist box; Story 1 cron-fires assertion |
| Editing wrong `lib.rs` (`lemmy_routes` vs `lemmy_api_routes`) | MED | MED | §11 crate-grouping + Task 4 GOTCHA names the distinct crates |
| Response DTO copies `Copy`/`Default`/`Hash` from sibling → compile error | MED | LOW | §10.6 explicit DROP + Task 2 GOTCHA |
| Integer-division rounding mismatch between helper and e2e | MED | MED | §10.1 + Task 1 GOTCHA: both compute integer mean; Story 1 asserts exact truncated value |
| Banned-exclusion off-by-one (denominator includes banned) | MED | MED | §10.2 sanction filter mirror :736-768; Story 2 asserts dim==1 + count==1 |
| All communities banned → divide-by-zero | LOW | HIGH | §10.1 denominator-0 → skip (no row, no emit); §15.5 box |

---

## 19. Notes

- **Route-path convention divergence:** PRD §5.5 specifies `/admin/reputation/rollup` (slash), but the sibling endpoint registers `/reputation-stats` (hyphen, lib.rs:502). The plan follows the **PRD-literal slash** form. File a `kind:"log"` planner DQ documenting the divergence so a future convention-normalisation pass can decide (slash-vs-hyphen for admin reputation routes). This is non-blocking — both forms route correctly.
- **`ROLLUP_RECOMPUTED` carries RAW `person_id`** (not pseudonym) in its payload per the registry payload shape, even though the emit's `actor_pseudonym` arg is `None` (system). The payload person_id is data; the actor pseudonym is the signer.
- **Task 4's 2-crate span is irreducible** — handler+mod live in `lemmy_api`, route in `lemmy_api_routes`. Within the Sonnet 3-file/2-crate ceiling.
- **No new config knob** — both `job.rollup_interval_days` and `job.rollup_equal_weights` shipped in RT-r1.
- **First-rollup baseline** — for a person with no prior rollup row, `old_snapshot` is `None`; capability flips compute against the all-false baseline (mirror the unit-test pattern at `reputation_snapshot.rs:1001-1022`).
- **All-banned edge** — a person banned from every community they have a snapshot in produces denominator 0 → no rollup row written, no emit. This is correct (they have no instance-wide standing to roll up).

---

## 20. Confidence score

- **Plan correctness:** 9/10 — every anchor re-verified fresh against `phase-v1-RT-r5`; the one residual risk is the integer-mean exactness between helper and e2e (mitigated by Story 1).
- **Cargo budget:** 9/10 — ~5–6 GB peak ≤ 6 GB; pre-Shape-G local validation with forbidden-window discipline.
- **Test coverage:** 8/10 — 4 stories cover mean, banned exclusion, endpoint admin/non-admin, and log emit; capability-flip-on-rollup is exercised indirectly (not a dedicated story).
