use crate::GovernanceModlogView;
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::{CommunityId, ModerationCaseId, PublicCaseLogId};
use lemmy_db_schema_file::schema::{appeal, community, public_case_log};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::LemmyResult;
use std::collections::HashSet;

/// Main-query row shape for the modlog queries. Captures every scalar
/// column selected from the `public_case_log` + `community` left-join.
/// `appealed` is NOT in this tuple — it's resolved via a second
/// round-trip in [`appealed_case_ids`] and merged in [`build_view`].
type ModlogRow = (
  PublicCaseLogId,
  ModerationCaseId,
  Option<CommunityId>,
  Option<String>,
  String,
  DateTime<Utc>,
);

/// Aggregate helper: return the set of `moderation_case.id` values that
/// have at least one `appeal` row. Restricted to the given case-ID list.
/// Returns an empty set when `case_ids` is empty (short-circuits the
/// round-trip so a modlog page with zero rows doesn't spend a query).
///
/// Drift 3 from plan §2.3 — `public_case_log.appealed` has no source
/// column; this helper computes the bool via a bulk `IN`-list lookup on
/// `appeal`. The result set is deduped into a `HashSet` by the caller.
async fn appealed_case_ids(
  conn: &mut diesel_async::AsyncPgConnection,
  case_ids: &[ModerationCaseId],
) -> LemmyResult<HashSet<ModerationCaseId>> {
  if case_ids.is_empty() {
    return Ok(HashSet::new());
  }
  let rows: Vec<ModerationCaseId> = appeal::table
    .filter(appeal::case_id.eq_any(case_ids))
    .select(appeal::case_id)
    .load::<ModerationCaseId>(conn)
    .await?;
  Ok(rows.into_iter().collect())
}

/// Map a single main-query row + the appealed set into a
/// `GovernanceModlogView`, inlining the plan §2 drift stubs
/// (`decision = None`, `sanction_action = None`) at the same step.
fn build_view(
  row: ModlogRow,
  appealed: &HashSet<ModerationCaseId>,
) -> GovernanceModlogView {
  let (_pcl_id, case_id, community_id, community_name, summary, published_at) = row;
  GovernanceModlogView {
    case_id: case_id.0,
    community_id: community_id.map(|c| c.0),
    community_name,
    decision: None,
    sanction_action: None,
    summary,
    published_at,
    appealed: appealed.contains(&case_id),
  }
}

/// All public case log entries, newest-first by `published_at`. Powers
/// the cross-community `GET /api/v4/governance/modlog` endpoint (task 43
/// in Phase 4 — handler layer adds pagination).
///
/// Two round-trips: main join + appealed-case-id set.
pub async fn list_public_case_log(
  pool: &mut DbPool<'_>,
) -> LemmyResult<Vec<GovernanceModlogView>> {
  let conn = &mut get_conn(pool).await?;

  let rows: Vec<ModlogRow> = public_case_log::table
    .left_join(community::table)
    .order_by(public_case_log::published_at.desc())
    .select((
      public_case_log::id,
      public_case_log::case_id,
      public_case_log::community_id,
      community::name.nullable(),
      public_case_log::summary,
      public_case_log::published_at,
    ))
    .load::<ModlogRow>(conn)
    .await?;

  let case_ids: Vec<ModerationCaseId> = rows.iter().map(|r| r.1).collect();
  let appealed = appealed_case_ids(conn, &case_ids).await?;

  Ok(rows.into_iter().map(|r| build_view(r, &appealed)).collect())
}
