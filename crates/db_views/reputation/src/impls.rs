//! Tuple-load + `build_view` query helpers per the Phase 2b template.
//!
//! All three public queries take `&mut DbPool<'_>` and return the view
//! structs defined in `lib.rs`. No `Queryable`/`Selectable` derives — the
//! bare-scalar count fields on each view are derived via second round-trips
//! per `.claude/rules/view-crate-selectable-template.md`.

use crate::{EndorsementSummaryView, ReputationSummaryView};
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::CommunityId;
use lemmy_db_schema_file::PersonId;
use lemmy_db_schema_file::schema::{endorsement, reputation_snapshot, sanction, surety};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::LemmyResult;

/// Main-query tuple for `reputation_snapshot`. Selected from a single row
/// by primary key (or matching `(person_id, community_id)`). Does NOT
/// include `can_sponsor` — v0 does not expose that column through any view
/// per [99 OQ-014].
type SnapshotRow = (
  PersonId,
  Option<CommunityId>,
  i32,
  i32,
  i32,
  i32,
  bool,
  bool,
  DateTime<Utc>,
);

/// `COUNT(*)` of active sanctions targeting the given person. Active means
/// `sanction.active = true` — the `ends_at` column is independent (future
/// rows can be inactive; past rows can still be active until the modlog
/// worker resolves them).
async fn count_active_sanctions(
  conn: &mut diesel_async::AsyncPgConnection,
  person_id: PersonId,
) -> LemmyResult<i64> {
  let n: i64 = sanction::table
    .filter(sanction::target_person_id.eq(person_id))
    .filter(sanction::active.eq(true))
    .count()
    .get_result(conn)
    .await?;
  Ok(n)
}

/// Map a `SnapshotRow` + the round-trip sanction count into the view.
fn build_summary_view(row: SnapshotRow, active_sanctions: i64) -> ReputationSummaryView {
  let (
    person_id,
    community_id,
    reporting_accuracy,
    jury_reliability,
    participation_consistency,
    endorsement_strength,
    jury_eligible,
    trusted_reporter,
    calculated_at,
  ) = row;
  ReputationSummaryView {
    person_id: person_id.0,
    community_id: community_id.map(|c| c.0),
    reporting_accuracy,
    jury_reliability,
    participation_consistency,
    endorsement_strength,
    jury_eligible,
    trusted_reporter,
    calculated_at,
    active_sanctions,
  }
}

/// Read the snapshot for a `(person_id, community_id)` pair. Returns
/// `Ok(None)` when no snapshot row exists — the task 53 calculator writes
/// on demand, so first-time reads before any recompute legitimately return
/// `None`.
///
/// Community-scoped reads use `community_id = Some(id)`; instance-scoped
/// reads use `community_id = None`, which requires an `IS NULL` predicate
/// (Postgres does not treat `NULL = NULL` as true).
pub async fn read_reputation_summary(
  pool: &mut DbPool<'_>,
  person_id: PersonId,
  community_id: Option<CommunityId>,
) -> LemmyResult<Option<ReputationSummaryView>> {
  let conn = &mut get_conn(pool).await?;

  let row: Option<SnapshotRow> = match community_id {
    Some(cid) => reputation_snapshot::table
      .filter(reputation_snapshot::person_id.eq(person_id))
      .filter(reputation_snapshot::community_id.eq(cid))
      .select((
        reputation_snapshot::person_id,
        reputation_snapshot::community_id,
        reputation_snapshot::reporting_accuracy,
        reputation_snapshot::jury_reliability,
        reputation_snapshot::participation_consistency,
        reputation_snapshot::endorsement_strength,
        reputation_snapshot::jury_eligible,
        reputation_snapshot::trusted_reporter,
        reputation_snapshot::calculated_at,
      ))
      .first::<SnapshotRow>(conn)
      .await
      .optional()?,
    None => reputation_snapshot::table
      .filter(reputation_snapshot::person_id.eq(person_id))
      .filter(reputation_snapshot::community_id.is_null())
      .select((
        reputation_snapshot::person_id,
        reputation_snapshot::community_id,
        reputation_snapshot::reporting_accuracy,
        reputation_snapshot::jury_reliability,
        reputation_snapshot::participation_consistency,
        reputation_snapshot::endorsement_strength,
        reputation_snapshot::jury_eligible,
        reputation_snapshot::trusted_reporter,
        reputation_snapshot::calculated_at,
      ))
      .first::<SnapshotRow>(conn)
      .await
      .optional()?,
  };

  match row {
    None => Ok(None),
    Some(r) => {
      let active_sanctions = count_active_sanctions(conn, r.0).await?;
      Ok(Some(build_summary_view(r, active_sanctions)))
    }
  }
}

/// Per-person inbound / outbound / surety endorsement aggregates. Returns
/// a `Vec` with at most one element — empty when the person has no
/// endorsements, exactly one element otherwise. The `Vec` signature matches
/// [04 §4.3]'s `list_endorsements_for_person` per GOTCHA-52b.
///
/// `active_only = true` filters `revoked_at IS NULL` on both the endorsement
/// and surety counts; `active_only = false` counts all-time.
pub async fn list_endorsements_for_person(
  pool: &mut DbPool<'_>,
  person_id: PersonId,
  active_only: bool,
) -> LemmyResult<Vec<EndorsementSummaryView>> {
  let conn = &mut get_conn(pool).await?;

  // Three independent `SELECT COUNT(*)` queries — no single join shape
  // aggregates all three counts cleanly. Surety / endorsement tables use
  // DIFFERENT column names per table (surety: sponsor_id/sponsored_id;
  // endorsement: from_person_id/to_person_id) — do NOT confuse them
  // (GOTCHA-52c).

  let inbound: i64 = {
    let base = endorsement::table
      .filter(endorsement::to_person_id.eq(person_id))
      .into_boxed();
    let filtered = if active_only {
      base.filter(endorsement::revoked_at.is_null())
    } else {
      base
    };
    filtered.count().get_result(conn).await?
  };

  let outbound: i64 = {
    let base = endorsement::table
      .filter(endorsement::from_person_id.eq(person_id))
      .into_boxed();
    let filtered = if active_only {
      base.filter(endorsement::revoked_at.is_null())
    } else {
      base
    };
    filtered.count().get_result(conn).await?
  };

  let active_sureties_inbound: i64 = {
    let base = surety::table
      .filter(surety::sponsored_id.eq(person_id))
      .into_boxed();
    let filtered = if active_only {
      base.filter(surety::revoked_at.is_null())
    } else {
      base
    };
    filtered.count().get_result(conn).await?
  };

  // Zero-row short-circuit: if all three counts are zero and active_only,
  // return empty vec per GOTCHA-52b doc-comment contract. For
  // `active_only = false` the caller may still want the zero-row
  // acknowledgement, so only collapse when strictly active_only.
  if active_only && inbound == 0 && outbound == 0 && active_sureties_inbound == 0 {
    return Ok(Vec::new());
  }

  Ok(vec![EndorsementSummaryView {
    person_id: person_id.0,
    inbound_endorsements: inbound,
    outbound_endorsements: outbound,
    active_sureties_inbound,
    active_sureties_outbound: 0,
  }])
}

/// Per-person outbound-sureties aggregate. Returns a `Vec` matching the
/// [04 §4.3] signature; semantically reports the caller's sponsoring-side
/// role (endorsements they wrote + sureties they vouched for).
///
/// Empty vec when the caller has no rows and `active_only = true`.
pub async fn list_sureties_for_person(
  pool: &mut DbPool<'_>,
  person_id: PersonId,
  active_only: bool,
) -> LemmyResult<Vec<EndorsementSummaryView>> {
  let conn = &mut get_conn(pool).await?;

  let inbound: i64 = {
    let base = endorsement::table
      .filter(endorsement::to_person_id.eq(person_id))
      .into_boxed();
    let filtered = if active_only {
      base.filter(endorsement::revoked_at.is_null())
    } else {
      base
    };
    filtered.count().get_result(conn).await?
  };

  let outbound: i64 = {
    let base = endorsement::table
      .filter(endorsement::from_person_id.eq(person_id))
      .into_boxed();
    let filtered = if active_only {
      base.filter(endorsement::revoked_at.is_null())
    } else {
      base
    };
    filtered.count().get_result(conn).await?
  };

  // Sureties WHERE this person is the SPONSOR (outbound-vouch direction),
  // not the sponsored. GOTCHA-52c — confusingly, `list_sureties_for_person`
  // in the [04 §4.3] signature is the OUTBOUND-vouch query, mirror of the
  // caller's own sponsoring role.
  let active_sureties_outbound: i64 = {
    let base = surety::table
      .filter(surety::sponsor_id.eq(person_id))
      .into_boxed();
    let filtered = if active_only {
      base.filter(surety::revoked_at.is_null())
    } else {
      base
    };
    filtered.count().get_result(conn).await?
  };

  if active_only && inbound == 0 && outbound == 0 && active_sureties_outbound == 0 {
    return Ok(Vec::new());
  }

  Ok(vec![EndorsementSummaryView {
    person_id: person_id.0,
    inbound_endorsements: inbound,
    outbound_endorsements: outbound,
    active_sureties_inbound: 0,
    active_sureties_outbound,
  }])
}
