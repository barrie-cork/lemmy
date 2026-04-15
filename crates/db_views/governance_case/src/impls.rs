use crate::GovernanceCaseSummaryView;
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, dsl::count_star};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::{CommunityId, ModerationCaseId};
use lemmy_db_schema_file::{
  enums::{CaseSeverity, CaseStatus, CaseTargetType, JuryAssignmentStatus},
  schema::{community, jury_assignment, moderation_case},
};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::LemmyResult;
use std::collections::HashMap;

/// Main-query row shape for the summary-view queries (tasks 16, 18, 19).
/// Captures every column selected from the `moderation_case` + `community`
/// join; the aggregate `jury_submitted` count comes from a second query
/// via [`submitted_counts_by_case`] and the drift-stub constants from
/// plan §2 are inlined in [`build_summary`].
type SummaryRow = (
  ModerationCaseId,
  CaseStatus,
  CaseSeverity,
  String,
  DateTime<Utc>,
  Option<CommunityId>,
  Option<String>,
  CaseTargetType,
);

/// Open cases in a community — the home-screen list for community mods.
///
/// Filter: `community_id` matches AND `status IN (Open, ThresholdMet,
/// JurySelection, InReview)`. `EmergencyRemove` and `AdminReview` are
/// intentionally excluded — they belong to Phase 4 admin views.
///
/// Two round-trips: the main query joins `moderation_case` to `community`
/// via an explicit `.on(...)` (no `joinable!` declaration exists for the
/// pair), and a second aggregation counts submitted jury assignments per
/// case. Phase 2a's view shape requires a manual map step regardless
/// because `reporter_count` and `jury_needed` are stubs/constants that
/// cannot be expressed as Diesel column selections (plan §2 drift).
pub async fn list_open_cases_for_community(
  pool: &mut DbPool<'_>,
  community_id: CommunityId,
) -> LemmyResult<Vec<GovernanceCaseSummaryView>> {
  let conn = &mut get_conn(pool).await?;

  let open_statuses = [
    CaseStatus::Open,
    CaseStatus::ThresholdMet,
    CaseStatus::JurySelection,
    CaseStatus::InReview,
  ];

  let rows: Vec<SummaryRow> = moderation_case::table
    .left_join(
      community::table.on(community::id.nullable().eq(moderation_case::community_id)),
    )
    .filter(moderation_case::community_id.eq(community_id))
    .filter(moderation_case::status.eq_any(open_statuses))
    .select((
      moderation_case::id,
      moderation_case::status,
      moderation_case::severity,
      moderation_case::reason_code,
      moderation_case::opened_at,
      moderation_case::community_id,
      community::name.nullable(),
      moderation_case::target_type,
    ))
    .load::<SummaryRow>(conn)
    .await?;

  let case_ids: Vec<ModerationCaseId> = rows.iter().map(|r| r.0).collect();
  let submitted_counts = submitted_counts_by_case(conn, &case_ids).await?;

  Ok(
    rows
      .into_iter()
      .map(|r| build_summary(r, &submitted_counts))
      .collect(),
  )
}

/// Aggregate helper: count `jury_assignment` rows with
/// `status = 'submitted'` grouped by `case_id`, restricted to the given
/// list of cases. Returns an empty map when `case_ids` is empty.
async fn submitted_counts_by_case(
  conn: &mut diesel_async::AsyncPgConnection,
  case_ids: &[ModerationCaseId],
) -> LemmyResult<HashMap<ModerationCaseId, i64>> {
  if case_ids.is_empty() {
    return Ok(HashMap::new());
  }
  let pairs: Vec<(ModerationCaseId, i64)> = jury_assignment::table
    .filter(jury_assignment::case_id.eq_any(case_ids))
    .filter(jury_assignment::status.eq(JuryAssignmentStatus::Submitted))
    .group_by(jury_assignment::case_id)
    .select((jury_assignment::case_id, count_star()))
    .load::<(ModerationCaseId, i64)>(conn)
    .await?;
  Ok(pairs.into_iter().collect())
}

/// Map a single main-query row + the submitted-counts lookup into a
/// `GovernanceCaseSummaryView`, inlining the drift stubs and [05 §3]
/// constants at the same step.
fn build_summary(
  row: SummaryRow,
  submitted_counts: &HashMap<ModerationCaseId, i64>,
) -> GovernanceCaseSummaryView {
  let (case_id, status, severity, reason_code, opened_at, community_id, community_name, target_type) =
    row;
  let submitted = submitted_counts
    .get(&case_id)
    .copied()
    .unwrap_or(0);
  GovernanceCaseSummaryView {
    case_id: case_id.0,
    status,
    severity,
    reason_code,
    opened_at,
    community_id: community_id.map(|c| c.0),
    community_name,
    target_type,
    reporter_count: 0,
    jury_needed: 5,
    jury_submitted: i32::try_from(submitted).unwrap_or(i32::MAX),
  }
}
