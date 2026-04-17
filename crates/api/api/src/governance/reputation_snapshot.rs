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

use crate::governance::{actor_pseudonym_helper, config::{self, ConfigCache, Scope}, governance_log};
use chrono::{DateTime, Duration, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, sql_query, sql_types::BigInt};
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
  let check = |old_val: bool, new_val: bool, dim: CapabilityDimension, changes: &mut Vec<CapabilityChange>| {
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
  //    active-sanction count for the eligibility guard.
  let (published_at, active_sanctions) = load_person_context(conn, person_id).await?;

  // 4. Read the config thresholds and decay half-life via the cache.
  //    NOTE: all reads flow through a single ConfigCache so repeated
  //    lookups in one recompute don't hit the DB multiple times.
  let decay_half_life_days =
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance, "decay.positive_half_life_days").await?;
  let threshold_jury_reliability =
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance, "thresholds.jury_reliability").await?;
  let threshold_reporting_accuracy =
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance, "thresholds.reporting_accuracy").await?;
  let threshold_endorsement_strength =
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance, "thresholds.endorsement_strength").await?;
  let jury_age_requirement_days =
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance, "jury.age_requirement_days").await?;

  // 5. Sum deltas by dimension, applying decay only when expires_at is
  //    None (Watch 8 — double-decay guard). Founders keep their full delta
  //    until their cliff fires; organic events get halved past half-life.
  let now = Utc::now();
  let half_life = Duration::days(decay_half_life_days);
  let mut reporting_accuracy = 0_i32;
  let mut jury_reliability = 0_i32;
  let mut participation_consistency = 0_i32;
  let mut endorsement_strength = 0_i32;

  for event in &events {
    let applied_delta = compute_applied_delta(event, now, half_life);
    match event.dimension {
      ReputationDimension::ReportingAccuracy => reporting_accuracy += applied_delta,
      ReputationDimension::JuryReliability => jury_reliability += applied_delta,
      ReputationDimension::ParticipationConsistency => participation_consistency += applied_delta,
      ReputationDimension::EndorsementStrength => endorsement_strength += applied_delta,
    }
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
/// event (Watch 8). Organic events (expires_at IS NONE) get halved past
/// the half-life; founders keep their full delta until their cliff
/// fires (which the query-level filter has already enforced).
///
/// Only POSITIVE deltas decay — negative deltas (penalties) persist at
/// full value per plan task 53 step 2.c.
fn compute_applied_delta(event: &ReputationEvent, now: DateTime<Utc>, half_life: Duration) -> i32 {
  let original = event.delta;
  // Watch 8 — decay half-life must NOT apply to events with expires_at set.
  // The branch gates on `if event.expires_at.is_none()` (a distinct predicate
  // from the query-level filter in `load_live_events`). Founder seeds keep
  // their full delta until their cliff fires.
  if event.expires_at.is_none() {
    // Penalty path — negative deltas never decay (penalties persist).
    if original <= 0 {
      return original;
    }
    // Organic positive event: halve once if older than the half-life.
    // v0 uses a single half-life window; more aggressive schedules
    // (chained halving per half-life elapsed) are v1.
    let age = now - event.created_at;
    if age > half_life { original / 2 } else { original }
  } else {
    // expires_at is Some — founder seed. Skip decay entirely.
    original
  }
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

  let chunk_size =
    config::get_int(&mut cache, pool, Scope::Instance, "job.snapshot_batch_chunk_size").await?;
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
    info!(
      "governance: snapshot batch tick — no dirty pairs (chunk_size={chunk_size_usize})"
    );
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

/// Process one chunk under a single transaction. Returns the number of
/// expired-founder pairs in the chunk (a pair is counted as
/// expired-founder if any of its events in the chunk's tick window had
/// `expires_at <= now()`).
async fn process_chunk(
  conn: &mut AsyncPgConnection,
  pairs: &[(PersonId, Option<CommunityId>)],
  cache: &mut ConfigCache,
) -> LemmyResult<usize> {
  use diesel_async::scoped_futures::ScopedFutureExt;

  // Count how many pairs in this chunk had at least one event whose
  // expires_at just crossed now(). This is informational — logged by
  // the caller in its structured field output (GOTCHA-54g).
  let expired_count = count_expired_founders(conn, pairs).await?;

  let pairs_owned: Vec<(PersonId, Option<CommunityId>)> = pairs.to_vec();
  conn
    .transaction(|conn| {
      async move {
        for (person_id, community_id) in pairs_owned {
          // Advisory lock keyed on (person_id, community_id) serialises
          // bg-job recomputes against any concurrent synchronous handler
          // call (Watch 9).
          acquire_advisory_xact_lock(conn, person_id, community_id).await?;
          recompute_snapshot(conn, person_id, community_id, cache).await?;
        }
        Ok::<_, lemmy_utils::error::LemmyError>(())
      }
      .scope_boxed()
    })
    .await?;

  Ok(expired_count)
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
    Some(cid) => reputation_event::table
      .filter(reputation_event::person_id.eq(person_id))
      .filter(reputation_event::community_id.eq(cid))
      .filter(
        reputation_event::expires_at
          .is_null()
          .or(reputation_event::expires_at.gt(now)),
      )
      .select(ReputationEvent::as_select())
      .load::<ReputationEvent>(conn)
      .await?,
    None => reputation_event::table
      .filter(reputation_event::person_id.eq(person_id))
      .filter(reputation_event::community_id.is_null())
      .filter(
        reputation_event::expires_at
          .is_null()
          .or(reputation_event::expires_at.gt(now)),
      )
      .select(ReputationEvent::as_select())
      .load::<ReputationEvent>(conn)
      .await?,
  };
  Ok(rows)
}

async fn load_person_context(
  conn: &mut AsyncPgConnection,
  person_id: PersonId,
) -> LemmyResult<(DateTime<Utc>, i64)> {
  let published_at: DateTime<Utc> = person::table
    .filter(person::id.eq(person_id))
    .select(person::published_at)
    .first::<DateTime<Utc>>(conn)
    .await?;
  let active_sanctions: i64 = sanction::table
    .filter(sanction::target_person_id.eq(person_id))
    .filter(sanction::active.eq(true))
    .count()
    .get_result(conn)
    .await?;
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
    None => insert_into(reputation_snapshot::table)
      .values(form)
      .returning(ReputationSnapshot::as_returning())
      .get_result::<ReputationSnapshot>(conn)
      .await?,
    Some(id) => diesel::update(reputation_snapshot::table.filter(reputation_snapshot::id.eq(id)))
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
      .await?,
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
  #[derive(diesel::QueryableByName)]
  struct IgnoredRow {
    #[diesel(sql_type = BigInt)]
    _lock_key: i64,
  }

  // 64-bit lock key: upper 32 bits = person_id, lower 32 = community_id
  // (or 0 for None). Deterministic + unique across the (person, community)
  // space.
  let key: i64 = (i64::from(person_id.0) << 32) | i64::from(community_id.map(|c| c.0).unwrap_or(0));
  let _ignored: Vec<IgnoredRow> = sql_query("SELECT pg_advisory_xact_lock($1) AS _lock_key")
    .bind::<BigInt, _>(key)
    .load::<IgnoredRow>(conn)
    .await?;
  Ok(())
}

// -- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;

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
    assert!(matches!(changes[0].dimension, CapabilityDimension::JuryEligible));
    assert!(matches!(changes[1].direction, CapabilityDirection::Gained));
    assert!(matches!(changes[1].dimension, CapabilityDimension::TrustedReporter));
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
    };
    // Organic event past half-life — halved.
    assert_eq!(compute_applied_delta(&old_organic, now, half_life), 50);

    // Founder seed same age — NOT halved (Watch 8).
    let old_founder = ReputationEvent {
      expires_at: Some(now + Duration::days(30)),
      ..old_organic.clone()
    };
    assert_eq!(compute_applied_delta(&old_founder, now, half_life), 100);

    // Recent organic event — NOT halved.
    let recent_organic = ReputationEvent {
      created_at: now - Duration::days(30),
      ..old_organic.clone()
    };
    assert_eq!(compute_applied_delta(&recent_organic, now, half_life), 100);

    // Negative delta past half-life — NOT halved (penalties persist).
    let old_penalty = ReputationEvent {
      delta: -20,
      ..old_organic.clone()
    };
    assert_eq!(compute_applied_delta(&old_penalty, now, half_life), -20);
  }
}
