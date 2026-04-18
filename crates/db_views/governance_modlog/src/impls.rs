use crate::{CapabilityChangeLogEntry, GovernanceModlogView};
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, NullableExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::{CommunityId, ModerationCaseId, PublicCaseLogId};
use lemmy_db_schema_file::schema::{appeal, community, governance_log, public_case_log};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::LemmyResult;
use serde_json::Value;
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

/// Public case log entries filtered by community, newest-first. Powers
/// the community-scoped `GET /api/v4/governance/modlog?community_id=X`
/// variant (task 43 in Phase 4 — handler layer adds pagination).
///
/// Two round-trips: main join + appealed-case-id set.
pub async fn list_public_case_log_for_community(
  pool: &mut DbPool<'_>,
  community_id: CommunityId,
) -> LemmyResult<Vec<GovernanceModlogView>> {
  let conn = &mut get_conn(pool).await?;

  let rows: Vec<ModlogRow> = public_case_log::table
    .left_join(community::table)
    .filter(public_case_log::community_id.eq(community_id))
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

/// Read a single `public_case_log` entry by its primary key. Returns
/// `None` if no row with that id exists (via `.first(...).await.optional()?`
/// — the missing-row case is NOT an error). Powers Phase 4 handlers that
/// need to deep-link a single modlog entry.
///
/// Two round-trips when the row exists: main join + appealed-case-id
/// lookup. Short-circuits to one round-trip when the row is missing.
pub async fn read_public_case_log_entry(
  pool: &mut DbPool<'_>,
  entry_id: PublicCaseLogId,
) -> LemmyResult<Option<GovernanceModlogView>> {
  let conn = &mut get_conn(pool).await?;

  let row: Option<ModlogRow> = public_case_log::table
    .left_join(community::table)
    .filter(public_case_log::id.eq(entry_id))
    .select((
      public_case_log::id,
      public_case_log::case_id,
      public_case_log::community_id,
      community::name.nullable(),
      public_case_log::summary,
      public_case_log::published_at,
    ))
    .first::<ModlogRow>(conn)
    .await
    .optional()?;

  match row {
    None => Ok(None),
    Some(r) => {
      let appealed = appealed_case_ids(conn, &[r.1]).await?;
      Ok(Some(build_view(r, &appealed)))
    }
  }
}

/// Tuple-row shape for the capability-change query.
type CapabilityChangeRow = (
  i64,
  String,
  Value,
  Option<String>,
  DateTime<Utc>,
  Option<Vec<u8>>,
);

/// List `governance_log` entries with `entry_kind = 'capability_changed'`
/// strictly newer than `since_id`, ordered by id ascending, capped at
/// `limit` rows.
///
/// Reads directly from `governance_log` (NOT from `public_case_log`):
/// `capability_changed` entries are governance-log signals — they never
/// land on the public modlog and have no corresponding `public_case_log`
/// row. Plan §11.3 GOTCHA + Explore agent #3 §7.
///
/// **In-flight jury assignments are NOT recalled by threshold edits**
/// per IMPLEMENTATION-PLAN-v0.md line 375 — a `capability_changed` entry
/// where the user lost `jury_eligible` does NOT imply that user's
/// existing `Selected`/`Accepted` `jury_assignment` rows were revoked.
/// Snapshot recompute affects future selections only. v1 may add a
/// downstream consumer that revokes on flip; v0 does not.
///
/// **No new HTTP endpoint** in v0 — task 62
/// (`admin_reputation_stats`) MAY call this if useful; otherwise the
/// surface is reserved for v1's admin dashboard. Plan §11.3.
pub async fn list_capability_changed_entries_since(
  pool: &mut DbPool<'_>,
  since_id: i64,
  limit: i64,
) -> LemmyResult<Vec<CapabilityChangeLogEntry>> {
  let conn = &mut get_conn(pool).await?;

  let rows: Vec<CapabilityChangeRow> = governance_log::table
    .filter(governance_log::entry_kind.eq("capability_changed"))
    .filter(governance_log::id.gt(since_id))
    .order_by(governance_log::id.asc())
    .limit(limit)
    .select((
      governance_log::id,
      governance_log::entry_kind,
      governance_log::payload,
      governance_log::actor_pseudonym,
      governance_log::created_at,
      governance_log::signature,
    ))
    .load::<CapabilityChangeRow>(conn)
    .await?;

  Ok(
    rows
      .into_iter()
      .map(|(id, entry_kind, payload, actor_pseudonym, created_at, signature)| {
        CapabilityChangeLogEntry {
          id,
          entry_kind,
          payload,
          actor_pseudonym,
          created_at,
          signature,
        }
      })
      .collect(),
  )
}
