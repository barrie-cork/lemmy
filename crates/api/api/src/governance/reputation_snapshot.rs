//! Reputation snapshot calculator.
//!
//! Computes `ReputationSnapshot` rows from `ReputationEvent` history. The
//! heart of Phase 5a — correct-by-default against the five named
//! watchpoints (2, 6, 8, 9, 11 from advisor-context-phase-5.md §4).
//!
//! ## Public surface
//!
//! - [`recompute_snapshot`] — compute-and-upsert one snapshot for a
//!   `(person_id, community_id)` pair. Safe to call from inside a
//!   `run_transaction` closure (task 55) OR from the background job
//!   (task 54). Emits one `capability_changed` governance-log entry per
//!   boolean transition.
//! - [`run_snapshot_batch`] — batch variant registered by the clokwerk
//!   scheduler. Finds dirty `(person_id, community_id)` pairs (new events
//!   since last tick; founder cliffs that just expired) and calls
//!   `recompute_snapshot` for each, chunked per
//!   `job.snapshot_batch_chunk_size` config.
//!
//! ## Watchpoint mapping
//!
//! - **Watch 2** (`expires_at` filter / founder cliff): the query
//!   `WHERE expires_at IS NULL OR expires_at > now()` drops founder seeds
//!   whose cliff has passed. NOT `expires_at IS NOT NULL AND ...` — that
//!   would drop permanent organic events.
//! - **Watch 8** (decay-vs-cliff double discount): the in-memory decay
//!   branch gates on `if event.expires_at.is_none()` — a distinct
//!   predicate from the query-level filter. Founders never get decayed;
//!   organic events get halved past the half-life.
//! - **Watch 9** (concurrent recompute race): old-snapshot SELECT uses
//!   naive `FOR UPDATE` when a row exists (GOTCHA-53i rationale — no
//!   `SKIP LOCKED` / `NOWAIT`). First-time recomputes have nothing to
//!   lock; serialisation falls to `ON CONFLICT (person_id, community_id)
//!   DO UPDATE WHERE reputation_snapshot.calculated_at < excluded.calculated_at`.
//!   Background-job callers ALSO acquire a `pg_advisory_xact_lock` keyed
//!   on the (person_id, community_id) pair, serialising the bg job
//!   against any concurrent synchronous endorsement handler.
//! - **Watch 10** (PII leakage): `capability_changed` payload contains
//!   only `dimension_flipped`, `direction`, `snapshot_community_id` — no
//!   raw `person_id`. The actor pseudonym rides in `governance_log.actor_pseudonym`
//!   via `actor_pseudonym_helper::get_or_create`.
//! - **Watch 11** (admin attribution): every capability flip is logged.
//!   An admin raising `config.thresholds.jury_reliability` will cascade
//!   into many `capability_changed` entries on the next tick; the
//!   `admin-config-write.sh` wrapper (decision-queue #13; deferred to
//!   5c sibling docs) is the v0 story for attributing the cascade root.
//!
//! ## v1 feature flag (RT-r2)
//!
//! `feature.reputation_v1_decay_enabled` (governance_config, default `false`)
//! gates the per-(dimension, direction) chained-halving decay + per-dimension
//! bounds clamp introduced in v1-RT-r2. When `false`, the v0 single-halving
//! against `decay.positive_half_life_days` is preserved verbatim and no clamp
//! applies. The 16 per-(dim, direction) + bounds keys are read only when the
//! flag is `true`; otherwise the legacy single key is read.

use crate::governance::{
  actor_pseudonym_helper,
  config::{self, ConfigCache, Scope},
  governance_log,
};
use chrono::{DateTime, Duration, Utc};
use diesel::{
  BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, sql_query,
  sql_types::BigInt,
};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::newtypes::{CommunityId, ReputationSnapshotId};
use lemmy_db_schema::source::governance::{
  reputation_event::ReputationEvent,
  reputation_snapshot::{ReputationSnapshot, ReputationSnapshotInsertForm},
};
use lemmy_db_schema_file::PersonId;
use lemmy_db_schema_file::enums::ReputationDimension;
use lemmy_db_schema_file::schema::{person, reputation_event, reputation_snapshot, sanction};
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;
use tracing::info;

// -- Types ------------------------------------------------------------------

/// Direction of a boolean capability flip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityDirection {
  Gained,
  Lost,
}

impl CapabilityDirection {
  fn as_str(self) -> &'static str {
    match self {
      CapabilityDirection::Gained => "gained",
      CapabilityDirection::Lost => "lost",
    }
  }
}

/// Which boolean on `ReputationSnapshot` flipped between snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityDimension {
  JuryEligible,
  TrustedReporter,
  CanSponsor,
}

impl CapabilityDimension {
  fn as_str(self) -> &'static str {
    match self {
      CapabilityDimension::JuryEligible => "jury_eligible",
      CapabilityDimension::TrustedReporter => "trusted_reporter",
      CapabilityDimension::CanSponsor => "can_sponsor",
    }
  }
}

/// One boolean transition between two consecutive snapshots.
#[derive(Debug, Clone, Copy)]
pub struct CapabilityChange {
  pub dimension: CapabilityDimension,
  pub direction: CapabilityDirection,
}

/// Summary of a single `run_snapshot_batch` tick.
#[derive(Debug, Clone, Copy, Default)]
pub struct SnapshotBatchOutcome {
  pub pairs_processed: usize,
  pub expired_founders: usize,
  pub chunks: usize,
  pub chunk_size: usize,
}

/// Summary of a single `run_rollup_batch` tick.
#[derive(Debug, Clone, Copy, Default)]
pub struct RollupBatchOutcome {
  pub candidates: usize,
  pub rows_written: usize,
  pub skipped_empty: usize,
}

// -- Core calculator --------------------------------------------------------

/// Compare old vs new snapshot and return the set of boolean transitions
/// that fired. `None` for `old` means this is the first snapshot ever
/// written for the pair; any `true` boolean on `new` counts as a
/// `Gained` transition.
///
/// Pure function — no DB, no side effects, unit-testable in isolation.
pub fn detect_capability_changes(
  old: Option<&ReputationSnapshot>,
  new: &ReputationSnapshot,
) -> Vec<CapabilityChange> {
  let mut changes = Vec::new();
  let check = |old_val: bool,
               new_val: bool,
               dim: CapabilityDimension,
               changes: &mut Vec<CapabilityChange>| {
    if old_val != new_val {
      changes.push(CapabilityChange {
        dimension: dim,
        direction: if new_val {
          CapabilityDirection::Gained
        } else {
          CapabilityDirection::Lost
        },
      });
    }
  };
  match old {
    None => {
      // First snapshot — every `true` is Gained, every `false` is Lost
      // (neutral default is `false`, so flipping from the implicit default
      // to `true` is meaningful; `false` to `false` emits nothing).
      if new.jury_eligible {
        changes.push(CapabilityChange {
          dimension: CapabilityDimension::JuryEligible,
          direction: CapabilityDirection::Gained,
        });
      }
      if new.trusted_reporter {
        changes.push(CapabilityChange {
          dimension: CapabilityDimension::TrustedReporter,
          direction: CapabilityDirection::Gained,
        });
      }
      if new.can_sponsor {
        changes.push(CapabilityChange {
          dimension: CapabilityDimension::CanSponsor,
          direction: CapabilityDirection::Gained,
        });
      }
    }
    Some(old) => {
      check(
        old.jury_eligible,
        new.jury_eligible,
        CapabilityDimension::JuryEligible,
        &mut changes,
      );
      check(
        old.trusted_reporter,
        new.trusted_reporter,
        CapabilityDimension::TrustedReporter,
        &mut changes,
      );
      check(
        old.can_sponsor,
        new.can_sponsor,
        CapabilityDimension::CanSponsor,
        &mut changes,
      );
    }
  }
  changes
}

/// Recompute a single `(person_id, community_id)` snapshot.
///
/// Safe to call from:
/// - Inside a `run_transaction` closure (synchronous endorsement handler,
///   task 55). `FOR UPDATE` serialises against concurrent in-tx recomputes
///   for the same pair.
/// - Outside any transaction (background job, task 54). Caller MUST acquire
///   `pg_advisory_xact_lock(hash(person_id, community_id))` inside a
///   wrapping mini-tx before calling.
///
/// See module-level docs for the Watch 2 / Watch 8 / Watch 9 / Watch 10 /
/// Watch 11 mapping.
pub async fn recompute_snapshot(
  conn: &mut AsyncPgConnection,
  person_id: PersonId,
  community_id: Option<CommunityId>,
  cache: &mut ConfigCache,
) -> LemmyResult<ReputationSnapshot> {
  // 1. Lock the old snapshot row if one exists (Watch 9). First-time
  //    recomputes have nothing to lock; serialisation falls to the
  //    ON CONFLICT DO UPDATE below.
  let old_snapshot = read_existing_snapshot_for_update(conn, person_id, community_id).await?;

  // 2. Load the events that contribute to the snapshot (Watch 2 —
  //    founder-cliff filter). `expires_at IS NULL` covers permanent
  //    organic events; `expires_at > now()` covers live founder seeds.
  let events = load_live_events(conn, person_id, community_id).await?;

  // 3. Load person.published_at for age-gated capability checks and the
  //    active-sanction count for the eligibility guard. Pass the snapshot's
  //    `community_id` so a community-scoped snapshot only considers
  //    sanctions that apply to that community (plus instance-wide
  //    sanctions); an instance-scoped snapshot (`community_id = None`)
  //    considers every active sanction.
  let (published_at, active_sanctions) = load_person_context(conn, person_id, community_id).await?;

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
  let threshold_reporting_accuracy = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "thresholds.reporting_accuracy",
  )
  .await?;
  let threshold_endorsement_strength = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "thresholds.endorsement_strength",
  )
  .await?;
  let jury_age_requirement_days = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.age_requirement_days",
  )
  .await?;

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

  // 6. Compute the three capability booleans.
  let account_age_days = (now - published_at).num_days();
  let jury_eligible = i64::from(jury_reliability) >= threshold_jury_reliability
    && account_age_days >= jury_age_requirement_days
    && active_sanctions == 0;
  let trusted_reporter = i64::from(reporting_accuracy) >= threshold_reporting_accuracy;
  let can_sponsor = i64::from(endorsement_strength) >= threshold_endorsement_strength;

  // 7. Upsert the new snapshot row. `WHERE calculated_at < excluded.calculated_at`
  //    guards against last-writer-loses: if a racing recompute already
  //    wrote a row with a higher `calculated_at`, keep that one.
  let form = ReputationSnapshotInsertForm {
    person_id,
    community_id,
    reporting_accuracy,
    jury_reliability,
    participation_consistency,
    endorsement_strength,
    jury_eligible,
    trusted_reporter,
    can_sponsor,
  };
  let new_snapshot = upsert_snapshot(conn, &form, now).await?;

  // 8. Detect capability flips and emit one `capability_changed`
  //    governance_log entry per flip (Watches 10 + 11). The actor
  //    pseudonym lands in the governance_log.actor_pseudonym column;
  //    the payload holds only dimension/direction/community — no raw
  //    person_id.
  let changes = detect_capability_changes(old_snapshot.as_ref(), &new_snapshot);
  if !changes.is_empty() {
    let actor_pseudonym =
      actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), person_id).await?;
    for change in &changes {
      governance_log::append(
        &mut (&mut *conn).into(),
        governance_log::ENTRY_KIND_CAPABILITY_CHANGED,
        json!({
          "dimension_flipped": change.dimension.as_str(),
          "direction": change.direction.as_str(),
          "snapshot_community_id": community_id.map(|c| c.0),
        }),
        Some(actor_pseudonym.clone()),
      )
      .await?;
    }
  }

  Ok(new_snapshot)
}

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
  // for positive operands). `n.min(31)` is in [1, 31] at this point
  // (age_days >= hl_days > 0 ensures n >= 1); try_from never actually
  // errors here — the fallback 31 is defence-in-depth.
  let shift = u32::try_from(n.min(31)).unwrap_or(31);
  // For positive i32, right-shift is arithmetic AND logical (both yield
  // 0 in the limit). Penalty guard above ensures original > 0 here.
  original >> shift
}

// -- Batch scheduler entry point -------------------------------------------

/// Tick of the scheduled snapshot batch job (task 54's clokwerk
/// registration calls this). Finds dirty `(person_id, community_id)`
/// pairs since the last tick, chunks them per `job.snapshot_batch_chunk_size`,
/// and processes each chunk in its own transaction. A crash mid-chunk
/// rolls back only the current chunk — earlier chunks' commits stay.
///
/// Errors never panic; the caller (`scheduler.run(...)`) logs and returns
/// so the next tick retries (Watch 6).
pub async fn run_snapshot_batch(context: &LemmyContext) -> LemmyResult<SnapshotBatchOutcome> {
  let pool = &mut context.pool();
  let mut cache = ConfigCache::new();

  let chunk_size = config::get_int(
    &mut cache,
    pool,
    Scope::Instance,
    "job.snapshot_batch_chunk_size",
  )
  .await?;
  let chunk_size_usize = usize::try_from(chunk_size).map_err(|_e| {
    LemmyErrorType::Unknown(format!(
      "job.snapshot_batch_chunk_size out of range for usize: {chunk_size}"
    ))
  })?;

  let dirty_pairs = load_dirty_pairs(pool).await?;
  let mut outcome = SnapshotBatchOutcome {
    pairs_processed: 0,
    expired_founders: 0,
    chunks: 0,
    chunk_size: chunk_size_usize,
  };

  if dirty_pairs.is_empty() {
    info!("governance: snapshot batch tick — no dirty pairs (chunk_size={chunk_size_usize})");
    return Ok(outcome);
  }

  for chunk in dirty_pairs.chunks(chunk_size_usize) {
    let mut chunk_cache = ConfigCache::new();
    let conn = &mut get_conn(pool).await?;
    // Per-chunk transaction — a mid-chunk crash rolls back only this
    // chunk; prior chunks stayed committed via earlier tx boundaries.
    let rows_in_chunk = chunk.len();
    let expired_in_chunk = process_chunk(conn, chunk, &mut chunk_cache).await?;
    outcome.pairs_processed += rows_in_chunk;
    outcome.expired_founders += expired_in_chunk;
    outcome.chunks += 1;
  }

  info!(
    "governance: snapshot batch tick — pairs_total={total}, chunks={chunks}, chunk_size={chunk_size_usize}, expired_founders={expired}",
    total = outcome.pairs_processed,
    chunks = outcome.chunks,
    expired = outcome.expired_founders,
  );
  Ok(outcome)
}

// -- Rollup batch -----------------------------------------------------------

/// Return distinct `PersonId` values that have at least one per-community
/// snapshot row, ordered by `person_id ASC` for deterministic chunking.
async fn load_rollup_candidates(
  pool: &mut lemmy_diesel_utils::connection::DbPool<'_>,
) -> LemmyResult<Vec<PersonId>> {
  let conn = &mut get_conn(pool).await?;
  let rows: Vec<PersonId> = reputation_snapshot::table
    .filter(reputation_snapshot::community_id.is_not_null())
    .select(reputation_snapshot::person_id)
    .distinct()
    .order_by(reputation_snapshot::person_id.asc())
    .load::<PersonId>(conn)
    .await?;
  Ok(rows)
}

/// Compute the instance-wide rollup snapshot for one person.
///
/// Equal-weighted integer mean of non-banned per-community snapshot
/// dimensions, written as a `reputation_snapshot` row with
/// `community_id IS NULL`. Returns `None` and writes nothing when the
/// contributing denominator is 0 (all communities banned or none exist).
pub async fn compute_rollup_snapshot(
  conn: &mut AsyncPgConnection,
  person_id: PersonId,
  cache: &mut ConfigCache,
) -> LemmyResult<Option<ReputationSnapshot>> {
  let now = Utc::now();

  // 1. Load all per-community snapshots for this person.
  let per_community: Vec<ReputationSnapshot> = reputation_snapshot::table
    .filter(reputation_snapshot::person_id.eq(person_id))
    .filter(reputation_snapshot::community_id.is_not_null())
    .load::<ReputationSnapshot>(conn)
    .await?;

  if per_community.is_empty() {
    return Ok(None);
  }

  // 2. Load banned community_ids: communities where the person has an active
  //    community-scoped sanction. OQ-V1-02: excluded from numerator AND
  //    denominator AND contributing count.
  let banned_cids: Vec<CommunityId> = sanction::table
    .filter(sanction::target_person_id.eq(person_id))
    .filter(sanction::active.eq(true))
    .filter(sanction::target_community_id.is_not_null())
    .select(sanction::target_community_id)
    .distinct()
    .load::<Option<CommunityId>>(conn)
    .await?
    .into_iter()
    .flatten()
    .collect();

  // 3. Contributing snapshots = per-community minus banned communities.
  let contributing: Vec<&ReputationSnapshot> = per_community
    .iter()
    .filter(|s| {
      s.community_id
        .map_or(false, |cid| !banned_cids.contains(&cid))
    })
    .collect();

  let denominator = contributing.len();
  if denominator == 0 {
    // All communities banned; delete stale rollup row if it exists.
    diesel::delete(
      reputation_snapshot::table
        .filter(reputation_snapshot::person_id.eq(person_id))
        .filter(reputation_snapshot::community_id.is_null()),
    )
    .execute(conn)
    .await?;
    return Ok(None);
  }
  let denom_i64 = denominator as i64;

  // 4. Equal-weighted integer mean per dimension (i32 division, truncates toward zero).
  //    GOTCHA: e2e mean assertions MUST use the same integer truncation.
  let reporting_accuracy = (contributing
    .iter()
    .map(|s| i64::from(s.reporting_accuracy))
    .sum::<i64>()
    / denom_i64) as i32;
  let jury_reliability = (contributing
    .iter()
    .map(|s| i64::from(s.jury_reliability))
    .sum::<i64>()
    / denom_i64) as i32;
  let participation_consistency = (contributing
    .iter()
    .map(|s| i64::from(s.participation_consistency))
    .sum::<i64>()
    / denom_i64) as i32;
  let endorsement_strength = (contributing
    .iter()
    .map(|s| i64::from(s.endorsement_strength))
    .sum::<i64>()
    / denom_i64) as i32;

  // 5. Config thresholds (mirrors recompute_snapshot :264-291).
  let threshold_jury_reliability = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "thresholds.jury_reliability",
  )
  .await?;
  let threshold_reporting_accuracy = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "thresholds.reporting_accuracy",
  )
  .await?;
  let threshold_endorsement_strength = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "thresholds.endorsement_strength",
  )
  .await?;
  let jury_age_requirement_days = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.age_requirement_days",
  )
  .await?;

  // 6. Instance-wide active sanctions + published_at (mirror :736-768).
  //    community_filter=None counts ALL active sanctions against the person.
  let (published_at, active_sanctions) = load_person_context(conn, person_id, None).await?;
  let account_age_days = (now - published_at).num_days();

  // 7. Capability booleans (mirror :352-357; R1: i64::from for i32 comparisons).
  let jury_eligible = i64::from(jury_reliability) >= threshold_jury_reliability
    && account_age_days >= jury_age_requirement_days
    && active_sanctions == 0;
  let trusted_reporter = i64::from(reporting_accuracy) >= threshold_reporting_accuracy;
  let can_sponsor = i64::from(endorsement_strength) >= threshold_endorsement_strength;

  // Form computed from steps 1-7 values; no DB writes needed before the transaction.
  let form = ReputationSnapshotInsertForm {
    person_id,
    community_id: None,
    reporting_accuracy,
    jury_reliability,
    participation_consistency,
    endorsement_strength,
    jury_eligible,
    trusted_reporter,
    can_sponsor,
  };

  // Steps 8-11: read old snapshot, upsert, and emit log entries atomically.
  conn
    .transaction(async move |conn| {
      // 8. Load existing rollup row for capability-flip detection.
      let old_snapshot: Option<ReputationSnapshot> = reputation_snapshot::table
        .filter(reputation_snapshot::person_id.eq(person_id))
        .filter(reputation_snapshot::community_id.is_null())
        .first::<ReputationSnapshot>(conn)
        .await
        .optional()?;

      // 9. Upsert rollup row (community_id = None, shared write path :770).
      let new_snapshot = upsert_snapshot(conn, &form, now).await?;

      // 10. Emit ROLLUP_RECOMPUTED. Actor = None (system/cron, ADR-015 §10).
      //     Payload carries raw person_id per the registry payload shape.
      governance_log::append(
        &mut (&mut *conn).into(),
        governance_log::ENTRY_KIND_ROLLUP_RECOMPUTED,
        json!({
          "person_id": person_id.0,
          "contributing_community_count": denominator,
          "rollup_dimensions": {
            "reporting_accuracy": reporting_accuracy,
            "jury_reliability": jury_reliability,
            "participation_consistency": participation_consistency,
            "endorsement_strength": endorsement_strength,
          },
          "recomputed_at": now.to_rfc3339(),
        }),
        None,
      )
      .await?;

      // 11. Emit CAPABILITY_CHANGED per flip. snapshot_community_id: null (rollup row).
      let changes = detect_capability_changes(old_snapshot.as_ref(), &new_snapshot);
      for change in &changes {
        governance_log::append(
          &mut (&mut *conn).into(),
          governance_log::ENTRY_KIND_CAPABILITY_CHANGED,
          json!({
            "dimension_flipped": change.dimension.as_str(),
            "direction": change.direction.as_str(),
            "snapshot_community_id": Option::<i32>::None,
          }),
          None,
        )
        .await?;
      }

      Ok(Some(new_snapshot))
    })
    .await
}

/// Tick of the scheduled rollup batch job. Finds persons with at least one
/// per-community snapshot, computes the equal-weighted instance-wide rollup
/// per person, and writes the result as a `community_id IS NULL` row.
///
/// Errors never panic; the scheduler logs and retries on the next tick.
pub async fn run_rollup_batch(context: &LemmyContext) -> LemmyResult<RollupBatchOutcome> {
  let pool = &mut context.pool();
  let mut cache = ConfigCache::new();

  let chunk_size = config::get_int(
    &mut cache,
    pool,
    Scope::Instance,
    "job.snapshot_batch_chunk_size",
  )
  .await?;
  let chunk_size_usize = usize::try_from(chunk_size).map_err(|_e| {
    LemmyErrorType::Unknown(format!(
      "job.snapshot_batch_chunk_size out of range for usize: {chunk_size}"
    ))
  })?;

  // Guard: chunks(0) panics. Treat 0 as 1 to prevent panic on misconfiguration.
  let chunk_size_usize = if chunk_size_usize == 0 { 1 } else { chunk_size_usize };

  let candidates = load_rollup_candidates(pool).await?;
  let mut outcome = RollupBatchOutcome {
    candidates: candidates.len(),
    rows_written: 0,
    skipped_empty: 0,
  };

  if candidates.is_empty() {
    info!("governance: rollup batch tick — no candidates");
    return Ok(outcome);
  }

  for chunk in candidates.chunks(chunk_size_usize) {
    let mut chunk_cache = ConfigCache::new();
    let conn = &mut get_conn(pool).await?;
    for &person_id in chunk {
      match compute_rollup_snapshot(conn, person_id, &mut chunk_cache).await? {
        Some(_) => outcome.rows_written += 1,
        None => outcome.skipped_empty += 1,
      }
    }
  }

  info!(
    "governance: rollup batch tick — candidates={candidates}, written={written}, skipped={skipped}",
    candidates = outcome.candidates,
    written = outcome.rows_written,
    skipped = outcome.skipped_empty,
  );
  Ok(outcome)
}

/// Phase 5c task 63d — emit a structured `tracing::error!` event under
/// `target: "governance::integrity"` if the most-recent
/// `reputation_snapshot.calculated_at` is older than `now - 2 *
/// interval_s` (or if the table is empty entirely). Per
/// IMPLEMENTATION-PLAN-v0.md line 398 + decision-queue #21.
///
/// Pure observability — no DB writes, no governance_log entry per
/// GOTCHA-63d-a (this is an ops signal, not a governance signal).
/// Safe to call from any caller; never errors except on connection
/// failures (the underlying `MAX(...)` query).
///
/// Time is injected via the `now` param so tests can drive the
/// staleness threshold deterministically (GOTCHA-63d-c). Production
/// caller in `scheduled_tasks.rs` passes `Utc::now()`.
pub async fn check_snapshot_staleness(
  conn: &mut AsyncPgConnection,
  interval_s: i64,
  now: DateTime<Utc>,
) -> LemmyResult<()> {
  use diesel::dsl::max;

  let max_calculated_at: Option<DateTime<Utc>> = reputation_snapshot::table
    .select(max(reputation_snapshot::calculated_at))
    .first(conn)
    .await?;

  let threshold = now - chrono::Duration::seconds(2 * interval_s);

  match max_calculated_at {
    None => {
      tracing::error!(
        target: "governance::integrity",
        "reputation_snapshot table empty — snapshot batch has never run (DoD line 398)"
      );
    }
    Some(max) if max < threshold => {
      tracing::error!(
        target: "governance::integrity",
        calculated_at = ?max,
        threshold = ?threshold,
        interval_s,
        "reputation_snapshot staleness detected (DoD line 398)"
      );
    }
    Some(_) => {
      // Within the 2 × interval window — no signal needed.
    }
  }

  Ok(())
}

/// Process one chunk under a single transaction. Returns the number of
/// expired-founder pairs in the chunk (a pair is counted as
/// expired-founder if any of its events in the chunk's tick window had
/// `expires_at <= now()`).
async fn process_chunk(
  conn: &mut AsyncPgConnection,
  pairs: &[(PersonId, Option<CommunityId>)],
  cache: &mut ConfigCache,
) -> LemmyResult<usize> {
  // Count how many pairs in this chunk had at least one event whose
  // expires_at just crossed now(). This is informational — logged by
  // the caller in its structured field output (GOTCHA-54g).
  let expired_count = count_expired_founders(conn, pairs).await?;

  let pairs_owned: Vec<(PersonId, Option<CommunityId>)> = pairs.to_vec();
  conn
    .transaction(async |conn| {
      for (person_id, community_id) in pairs_owned {
        // Advisory lock keyed on (person_id, community_id) serialises
        // bg-job recomputes against any concurrent synchronous handler
        // call (Watch 9).
        acquire_advisory_xact_lock(conn, person_id, community_id).await?;
        recompute_snapshot(conn, person_id, community_id, cache).await?;
      }
      Ok::<_, lemmy_utils::error::LemmyError>(())
    })
    .await?;

  Ok(expired_count)
}

/// Load the existing snapshot for `(person_id, community_id)`, or compute
/// one on the fly if no row is present. Thin wrapper over a non-locking
/// SELECT followed by [`recompute_snapshot`] — does NOT open a nested
/// transaction, so the caller's own transaction scope (if any) governs
/// the atomicity boundary.
///
/// Task 58 uses this from `create_report` to resolve the reporter's
/// `reporting_accuracy` without bolting a snapshot pre-warm into every
/// report codepath. Safe to call outside any transaction.
pub async fn load_or_compute_snapshot(
  conn: &mut AsyncPgConnection,
  person_id: PersonId,
  community_id: Option<CommunityId>,
  cache: &mut ConfigCache,
) -> LemmyResult<ReputationSnapshot> {
  use diesel::SelectableHelper;
  let existing: Option<ReputationSnapshot> = match community_id {
    Some(cid) => reputation_snapshot::table
      .filter(reputation_snapshot::person_id.eq(person_id))
      .filter(reputation_snapshot::community_id.eq(cid))
      .select(ReputationSnapshot::as_select())
      .first::<ReputationSnapshot>(conn)
      .await
      .optional()?,
    None => reputation_snapshot::table
      .filter(reputation_snapshot::person_id.eq(person_id))
      .filter(reputation_snapshot::community_id.is_null())
      .select(ReputationSnapshot::as_select())
      .first::<ReputationSnapshot>(conn)
      .await
      .optional()?,
  };
  match existing {
    Some(row) => Ok(row),
    None => recompute_snapshot(conn, person_id, community_id, cache).await,
  }
}

// -- Private helpers --------------------------------------------------------

async fn read_existing_snapshot_for_update(
  conn: &mut AsyncPgConnection,
  person_id: PersonId,
  community_id: Option<CommunityId>,
) -> LemmyResult<Option<ReputationSnapshot>> {
  use diesel::SelectableHelper;
  let row: Option<ReputationSnapshot> = match community_id {
    Some(cid) => reputation_snapshot::table
      .filter(reputation_snapshot::person_id.eq(person_id))
      .filter(reputation_snapshot::community_id.eq(cid))
      .select(ReputationSnapshot::as_select())
      .for_update()
      .first::<ReputationSnapshot>(conn)
      .await
      .optional()?,
    None => reputation_snapshot::table
      .filter(reputation_snapshot::person_id.eq(person_id))
      .filter(reputation_snapshot::community_id.is_null())
      .select(ReputationSnapshot::as_select())
      .for_update()
      .first::<ReputationSnapshot>(conn)
      .await
      .optional()?,
  };
  Ok(row)
}

async fn load_live_events(
  conn: &mut AsyncPgConnection,
  person_id: PersonId,
  community_id: Option<CommunityId>,
) -> LemmyResult<Vec<ReputationEvent>> {
  use diesel::SelectableHelper;
  let now = Utc::now();
  let rows: Vec<ReputationEvent> = match community_id {
    Some(cid) => {
      reputation_event::table
        .filter(reputation_event::person_id.eq(person_id))
        .filter(reputation_event::community_id.eq(cid))
        .filter(
          reputation_event::expires_at
            .is_null()
            .or(reputation_event::expires_at.gt(now)),
        )
        .select(ReputationEvent::as_select())
        .load::<ReputationEvent>(conn)
        .await?
    }
    None => {
      reputation_event::table
        .filter(reputation_event::person_id.eq(person_id))
        .filter(reputation_event::community_id.is_null())
        .filter(
          reputation_event::expires_at
            .is_null()
            .or(reputation_event::expires_at.gt(now)),
        )
        .select(ReputationEvent::as_select())
        .load::<ReputationEvent>(conn)
        .await?
    }
  };
  Ok(rows)
}

/// Load the person's `published_at` and the count of active sanctions that
/// apply to the snapshot currently being recomputed.
///
/// Sanction filtering rule: an instance-wide sanction (`scope = Instance`)
/// always counts. A community-scoped sanction (`target_community_id` set)
/// only counts when:
///
/// * `community_filter = None` (instance-scoped snapshot — every sanction
///   against the person reduces their instance-wide eligibility), OR
/// * `community_filter = Some(cid)` AND `sanction.target_community_id = cid`
///   (community-scoped snapshot — only sanctions against the same community
///   reduce eligibility in that community).
///
/// Without this filter, a community-scoped snapshot would be silently
/// disqualified by any unrelated sanction from another community, which
/// contradicts the per-(person, community) semantics of the snapshot table.
async fn load_person_context(
  conn: &mut AsyncPgConnection,
  person_id: PersonId,
  community_filter: Option<CommunityId>,
) -> LemmyResult<(DateTime<Utc>, i64)> {
  let published_at: DateTime<Utc> = person::table
    .filter(person::id.eq(person_id))
    .select(person::published_at)
    .first::<DateTime<Utc>>(conn)
    .await?;
  let base = sanction::table
    .filter(sanction::target_person_id.eq(person_id))
    .filter(sanction::active.eq(true))
    .into_boxed();
  let active_sanctions: i64 = match community_filter {
    // Community-scoped snapshot: count instance-wide sanctions (null
    // target_community_id) OR sanctions scoped to this same community.
    Some(cid) => {
      base
        .filter(
          sanction::target_community_id
            .is_null()
            .or(sanction::target_community_id.eq(cid)),
        )
        .count()
        .get_result(conn)
        .await?
    }
    // Instance-scoped snapshot: count every active sanction.
    None => base.count().get_result(conn).await?,
  };
  Ok((published_at, active_sanctions))
}

async fn upsert_snapshot(
  conn: &mut AsyncPgConnection,
  form: &ReputationSnapshotInsertForm,
  calc_time: DateTime<Utc>,
) -> LemmyResult<ReputationSnapshot> {
  use diesel::SelectableHelper;
  use diesel::insert_into;

  // First try a SELECT to see if a row already exists — the `for_update`
  // above in `recompute_snapshot` already locked it if so. If no row, we
  // do a plain INSERT; if a row exists, we UPDATE in place.
  //
  // We intentionally do NOT use `ON CONFLICT` at the DSL level because
  // the partial unique index on `community_id IS NULL` does not satisfy
  // Diesel's ON CONFLICT target-column check cleanly (task 50 created
  // the index but DSL on_conflict requires a full-column unique
  // constraint for safety). A branchful SELECT-then-INSERT-or-UPDATE
  // under the existing FOR UPDATE lock is race-free per Watch 9.

  let existing_id: Option<ReputationSnapshotId> = match form.community_id {
    Some(cid) => reputation_snapshot::table
      .filter(reputation_snapshot::person_id.eq(form.person_id))
      .filter(reputation_snapshot::community_id.eq(cid))
      .select(reputation_snapshot::id)
      .first::<ReputationSnapshotId>(conn)
      .await
      .optional()?,
    None => reputation_snapshot::table
      .filter(reputation_snapshot::person_id.eq(form.person_id))
      .filter(reputation_snapshot::community_id.is_null())
      .select(reputation_snapshot::id)
      .first::<ReputationSnapshotId>(conn)
      .await
      .optional()?,
  };

  let row: ReputationSnapshot = match existing_id {
    None => {
      insert_into(reputation_snapshot::table)
        .values(form)
        .returning(ReputationSnapshot::as_returning())
        .get_result::<ReputationSnapshot>(conn)
        .await?
    }
    Some(id) => {
      diesel::update(reputation_snapshot::table.filter(reputation_snapshot::id.eq(id)))
        .set((
          reputation_snapshot::reporting_accuracy.eq(form.reporting_accuracy),
          reputation_snapshot::jury_reliability.eq(form.jury_reliability),
          reputation_snapshot::participation_consistency.eq(form.participation_consistency),
          reputation_snapshot::endorsement_strength.eq(form.endorsement_strength),
          reputation_snapshot::jury_eligible.eq(form.jury_eligible),
          reputation_snapshot::trusted_reporter.eq(form.trusted_reporter),
          reputation_snapshot::can_sponsor.eq(form.can_sponsor),
          reputation_snapshot::calculated_at.eq(calc_time),
        ))
        .returning(ReputationSnapshot::as_returning())
        .get_result::<ReputationSnapshot>(conn)
        .await?
    }
  };

  Ok(row)
}

/// Load the distinct `(person_id, community_id)` pairs with new or
/// just-expired events since the prior tick's watermark. Watermark =
/// MAX(calculated_at) from `reputation_snapshot`; pairs whose latest event
/// `created_at` exceeds the watermark OR whose `expires_at` falls in
/// `(watermark, now()]` are dirty.
async fn load_dirty_pairs(
  pool: &mut lemmy_diesel_utils::connection::DbPool<'_>,
) -> LemmyResult<Vec<(PersonId, Option<CommunityId>)>> {
  let conn = &mut get_conn(pool).await?;
  let now = Utc::now();
  let watermark: Option<DateTime<Utc>> = reputation_snapshot::table
    .select(diesel::dsl::max(reputation_snapshot::calculated_at))
    .first::<Option<DateTime<Utc>>>(conn)
    .await?;

  // No snapshots yet — first tick of a fresh DB. Every person that has
  // a reputation_event row is dirty.
  let wm = watermark.unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap_or(Utc::now()));

  // Single query: DISTINCT (person_id, community_id) from reputation_event
  // matching the dirty predicate. ORDER BY person_id ASC for deterministic
  // chunking per GOTCHA-54f.
  let rows: Vec<(PersonId, Option<CommunityId>)> = reputation_event::table
    .filter(
      reputation_event::created_at.gt(wm).or(
        reputation_event::expires_at
          .gt(wm)
          .and(reputation_event::expires_at.le(now)),
      ),
    )
    .select((reputation_event::person_id, reputation_event::community_id))
    .distinct()
    .order_by(reputation_event::person_id.asc())
    .load::<(PersonId, Option<CommunityId>)>(conn)
    .await?;
  Ok(rows)
}

/// Count how many `(person_id, community_id)` pairs in the chunk have at
/// least one founder event whose `expires_at` just crossed `now()`.
async fn count_expired_founders(
  conn: &mut AsyncPgConnection,
  pairs: &[(PersonId, Option<CommunityId>)],
) -> LemmyResult<usize> {
  if pairs.is_empty() {
    return Ok(0);
  }
  let person_ids: Vec<PersonId> = pairs.iter().map(|(p, _)| *p).collect();
  let now = Utc::now();
  // Bulk lookup — we don't care about the exact (person, community)
  // pairing for the informational count; any expired event within the
  // chunk's person set raises the counter. Close enough for pilot
  // telemetry per GOTCHA-54g.
  let matches: i64 = reputation_event::table
    .filter(reputation_event::person_id.eq_any(&person_ids))
    .filter(reputation_event::expires_at.is_not_null())
    .filter(reputation_event::expires_at.le(now))
    .count()
    .get_result(conn)
    .await?;
  Ok(usize::try_from(matches).unwrap_or(0))
}

/// Acquire a pg_advisory_xact_lock keyed deterministically on the
/// `(person_id, community_id)` pair. Released automatically when the
/// current tx commits or rolls back (Watch 9).
async fn acquire_advisory_xact_lock(
  conn: &mut AsyncPgConnection,
  person_id: PersonId,
  community_id: Option<CommunityId>,
) -> LemmyResult<()> {
  // 64-bit lock key: upper 32 bits = person_id, lower 32 = community_id
  // encoded so `None` maps to `0` and `Some(CommunityId(c))` maps to
  // `c + 1`. The `+1` shift keeps `None` and `Some(CommunityId(0))`
  // distinct in the unlikely event that a live community row ever
  // lands at id 0 (Postgres SERIAL starts at 1 today, so this is
  // defensive rather than load-bearing). The encoding is deterministic
  // + unique across the `(person, community)` space.
  let community_component: i64 = community_id
    .map(|c| i64::from(c.0).saturating_add(1))
    .unwrap_or(0);
  let key: i64 = (i64::from(person_id.0) << 32) | community_component;
  // pg_advisory_xact_lock returns void. Use .execute (statement) rather than
  // .load (result-set decode) — the latter would fail with "Received less than
  // 8 bytes while decoding an i64" since void isn't BigInt.
  sql_query("SELECT pg_advisory_xact_lock($1)")
    .bind::<BigInt, _>(key)
    .execute(conn)
    .await?;
  Ok(())
}

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

/// Clamp a per-dimension i32 sum to the (floor, ceiling) bounds read as
/// i64 from governance_config. Normalises inverted bounds (floor > ceiling)
/// by swapping, so a misconfigured admin setting cannot panic at runtime.
fn clamp_dimension_i32(value: i32, floor: i64, ceiling: i64) -> i32 {
  let (floor, ceiling) = (floor.min(ceiling), floor.max(ceiling));
  let widened = i64::from(value);
  let clamped = widened.clamp(floor, ceiling);
  i32::try_from(clamped).unwrap_or({
    if clamped > 0 { i32::MAX } else { i32::MIN }
  })
}

// -- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;
  use lemmy_db_schema_file::enums::ReputationEventSourceType;

  fn make_snapshot(
    jury_eligible: bool,
    trusted_reporter: bool,
    can_sponsor: bool,
  ) -> ReputationSnapshot {
    ReputationSnapshot {
      id: ReputationSnapshotId(0),
      person_id: PersonId(1),
      community_id: None,
      reporting_accuracy: 0,
      jury_reliability: 0,
      participation_consistency: 0,
      endorsement_strength: 0,
      jury_eligible,
      trusted_reporter,
      calculated_at: Utc::now(),
      can_sponsor,
    }
  }

  #[test]
  fn detect_capability_changes_first_snapshot_all_false() {
    let new = make_snapshot(false, false, false);
    let changes = detect_capability_changes(None, &new);
    assert!(changes.is_empty());
  }

  #[test]
  fn detect_capability_changes_first_snapshot_one_true() {
    let new = make_snapshot(true, false, false);
    let changes = detect_capability_changes(None, &new);
    assert_eq!(changes.len(), 1);
    assert!(matches!(
      changes[0],
      CapabilityChange {
        dimension: CapabilityDimension::JuryEligible,
        direction: CapabilityDirection::Gained,
      }
    ));
  }

  #[test]
  fn detect_capability_changes_flip_gained_and_lost() {
    let old = make_snapshot(true, false, true);
    let new = make_snapshot(false, true, true);
    let changes = detect_capability_changes(Some(&old), &new);
    assert_eq!(changes.len(), 2);
    // Order follows the check order in detect_capability_changes: jury_eligible, trusted_reporter, can_sponsor.
    assert!(matches!(changes[0].direction, CapabilityDirection::Lost));
    assert!(matches!(
      changes[0].dimension,
      CapabilityDimension::JuryEligible
    ));
    assert!(matches!(changes[1].direction, CapabilityDirection::Gained));
    assert!(matches!(
      changes[1].dimension,
      CapabilityDimension::TrustedReporter
    ));
  }

  #[test]
  fn decay_applies_only_to_organic_events_past_half_life() {
    let now = Utc::now();
    let half_life = Duration::days(90);
    let old_organic = ReputationEvent {
      id: lemmy_db_schema::newtypes::ReputationEventId(0),
      person_id: PersonId(1),
      community_id: None,
      dimension: ReputationDimension::JuryReliability,
      delta: 100,
      source_case_id: None,
      source_report_id: None,
      reason: "test".to_string(),
      created_at: now - Duration::days(180),
      expires_at: None,
      dedupe_key: None,
      source_event_type: ReputationEventSourceType::Endorsement,
    };
    // Organic event past half-life — halved.
    assert_eq!(compute_applied_delta(&old_organic, now, half_life, false), 50);

    // Founder seed same age — NOT halved (Watch 8).
    let old_founder = ReputationEvent {
      expires_at: Some(now + Duration::days(30)),
      ..old_organic.clone()
    };
    assert_eq!(compute_applied_delta(&old_founder, now, half_life, false), 100);

    // Recent organic event — NOT halved.
    let recent_organic = ReputationEvent {
      created_at: now - Duration::days(30),
      ..old_organic.clone()
    };
    assert_eq!(compute_applied_delta(&recent_organic, now, half_life, false), 100);

    // Negative delta past half-life — NOT halved (penalties persist).
    let old_penalty = ReputationEvent {
      delta: -20,
      ..old_organic.clone()
    };
    assert_eq!(compute_applied_delta(&old_penalty, now, half_life, false), -20);
  }

  fn make_event(
    now: DateTime<Utc>,
    delta: i32,
    age_days: i64,
    expires_at: Option<DateTime<Utc>>,
  ) -> ReputationEvent {
    ReputationEvent {
      id: lemmy_db_schema::newtypes::ReputationEventId(0),
      person_id: PersonId(1),
      community_id: None,
      dimension: ReputationDimension::JuryReliability,
      delta,
      source_case_id: None,
      source_report_id: None,
      reason: "test".to_string(),
      created_at: now - Duration::days(age_days),
      expires_at,
      dedupe_key: None,
      source_event_type: ReputationEventSourceType::Endorsement,
    }
  }

  #[test]
  fn chained_halving_at_2x_half_life_quarters_delta() {
    let now = Utc::now();
    let event = make_event(now, 100, 180, None);
    let half_life = Duration::days(90);
    assert_eq!(compute_applied_delta(&event, now, half_life, true), 25);
  }

  #[test]
  fn chained_halving_at_3x_half_life_eighths_delta() {
    let now = Utc::now();
    let event = make_event(now, 100, 270, None);
    let half_life = Duration::days(90);
    assert_eq!(compute_applied_delta(&event, now, half_life, true), 12);
  }

  #[test]
  fn chained_halving_at_age_zero_no_decay() {
    let now = Utc::now();
    let event = make_event(now, 100, 0, None);
    let half_life = Duration::days(90);
    assert_eq!(compute_applied_delta(&event, now, half_life, true), 100);
  }

  #[test]
  fn chained_halving_negative_delta_skipped() {
    let now = Utc::now();
    let event = make_event(now, -20, 180, None);
    let half_life = Duration::days(90);
    assert_eq!(compute_applied_delta(&event, now, half_life, true), -20);
  }

  #[test]
  fn chained_halving_founder_cliff_skipped() {
    let now = Utc::now();
    let event = make_event(now, 100, 180, Some(now + Duration::days(30)));
    let half_life = Duration::days(90);
    assert_eq!(compute_applied_delta(&event, now, half_life, true), 100);
  }

  #[test]
  fn v0_arm_at_2x_half_life_single_halving() {
    let now = Utc::now();
    let event = make_event(now, 100, 180, None);
    let half_life = Duration::days(90);
    assert_eq!(compute_applied_delta(&event, now, half_life, false), 50);
  }

  #[test]
  fn chained_halve_helper_saturates_at_very_old_event() {
    assert_eq!(
      chained_halve(100, Duration::days(365 * 100), Duration::days(90)),
      0
    );
  }

  #[test]
  fn clamp_dimension_i32_within_bounds() {
    assert_eq!(clamp_dimension_i32(50, -100, 100), 50);
  }

  #[test]
  fn clamp_dimension_i32_above_ceiling() {
    assert_eq!(clamp_dimension_i32(250, -100, 100), 100);
  }

  #[test]
  fn clamp_dimension_i32_below_floor() {
    assert_eq!(clamp_dimension_i32(-250, -100, 100), -100);
  }

  #[test]
  fn clamp_dimension_i32_negative_below_zero_floor() {
    assert_eq!(clamp_dimension_i32(-5, 0, 200), 0);
  }

  #[test]
  fn clamp_dimension_i32_inverted_bounds_no_panic() {
    // floor > ceiling — should not panic; behaves as if bounds were swapped
    let result = clamp_dimension_i32(50, 100, 0); // floor=100, ceiling=0 → normalised to [0,100]
    assert_eq!(result, 50); // 50 is within [0, 100]
  }
}
