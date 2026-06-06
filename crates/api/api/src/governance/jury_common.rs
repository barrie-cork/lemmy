//! Shared helpers for jury-flow handlers (accept, decline, replacement-pick).
//!
//! Extracted at task 64 to preempt duplicated code between
//! `accept_jury_assignment` and `decline_jury_assignment` (task 65) per the
//! Phase 5c risk-reduction strategy Move 5. Both handlers need the same
//! sponsor-cluster conflict check against the case target.

use diesel::{
  QueryableByName, sql_query,
  sql_types::{Array, BigInt, Bool, Integer},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::PersonId;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[derive(QueryableByName)]
struct BoolRow {
  #[diesel(sql_type = Bool)]
  present: bool,
}

#[derive(QueryableByName)]
struct ClusterCountRow {
  #[diesel(sql_type = BigInt)]
  max_shared: i64,
}

/// Returns `true` when `a` and `b` share at least one active sponsor
/// (`surety.revoked_at IS NULL` on both ends). v0 definition is one-hop
/// per `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` §11.4 GOTCHA
/// — two-hop and endorsement-cluster are v1 items.
pub(crate) async fn shares_active_sponsor(
  conn: &mut AsyncPgConnection,
  a: PersonId,
  b: PersonId,
) -> LemmyResult<bool> {
  let row: BoolRow = sql_query(
    "SELECT EXISTS (
       SELECT 1 FROM surety s1
       JOIN surety s2 ON s1.sponsor_id = s2.sponsor_id
       WHERE s1.sponsored_id = $1
         AND s2.sponsored_id = $2
         AND s1.revoked_at IS NULL
         AND s2.revoked_at IS NULL
       LIMIT 1
     ) AS present",
  )
  .bind::<Integer, _>(a.0)
  .bind::<Integer, _>(b.0)
  .get_result(conn)
  .await
  .map_err(|_e| LemmyErrorType::Unknown("shares_active_sponsor query failed".to_string()))?;
  Ok(row.present)
}

/// Returns `true` when `>50%` of the supplied panel shares a common upstream
/// sponsor. PRD §5.1 hard constraint — used by v1-JM-b's 3-phase
/// select_eligible_jurors as the Phase-2 re-roll trigger and by later
/// sub-phases as the cross-juror cluster invariant.
///
/// "Shares a common upstream sponsor" is the same pairwise relation as
/// [`shares_active_sponsor`]: two members A and B are in the same cluster
/// when any `surety` row exists for both with the same `sponsor_id`. This
/// function counts, for each sponsor, how many panel members share that
/// sponsor — matching the pairwise semantics via a self-join on `surety`
/// — and checks whether the largest such cluster reaches a majority.
///
/// Majority threshold is `(len/2)+1` for the common odd panel sizes (5/7/9).
/// `len < 2` returns `Ok(false)` because a one-person panel cannot have a
/// majority cluster by definition (and the `(1/2)+1 = 1` threshold would
/// always be satisfied by one sponsor covering that lone member).
///
/// Note: PostgreSQL allows `$1` to appear multiple times in a prepared
/// statement; both positions in the WHERE clause bind to the same array.
pub(crate) async fn panel_has_sponsor_majority_cluster(
  conn: &mut AsyncPgConnection,
  person_ids: &[PersonId],
) -> LemmyResult<bool> {
  if person_ids.len() < 2 {
    return Ok(false);
  }
  let ids_bind: Vec<i32> = person_ids.iter().map(|p| p.0).collect();

  let row: ClusterCountRow = sql_query(
    "SELECT COALESCE(MAX(c), 0) AS max_shared FROM ( \
       SELECT s1.sponsor_id, COUNT(DISTINCT s1.sponsored_id) AS c \
       FROM surety s1 \
       JOIN surety s2 ON s1.sponsor_id = s2.sponsor_id \
       WHERE s1.sponsored_id = ANY($1) \
         AND s2.sponsored_id = ANY($1) \
         AND s1.revoked_at IS NULL \
         AND s2.revoked_at IS NULL \
       GROUP BY s1.sponsor_id \
     ) t",
  )
  .bind::<Array<Integer>, _>(ids_bind)
  .get_result(conn)
  .await
  .map_err(|_e| {
    LemmyErrorType::Unknown("panel_has_sponsor_majority_cluster query failed".to_string())
  })?;

  let majority = (person_ids.len() / 2) + 1;
  let max_shared: usize = row.max_shared.try_into().unwrap_or(usize::MAX); // BigInt can't exceed panel_size in practice; saturate instead of error.
  Ok(max_shared >= majority)
}
