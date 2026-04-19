//! Shared helpers for jury-flow handlers (accept, decline, replacement-pick).
//!
//! Extracted at task 64 to preempt duplicated code between
//! `accept_jury_assignment` and `decline_jury_assignment` (task 65) per the
//! Phase 5c risk-reduction strategy Move 5. Both handlers need the same
//! sponsor-cluster conflict check against the case target.

use diesel::{QueryableByName, sql_query, sql_types::{Bool, Integer}};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::PersonId;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[derive(QueryableByName)]
struct BoolRow {
  #[diesel(sql_type = Bool)]
  present: bool,
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
