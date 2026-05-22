use crate::JuryQueueView;
use chrono::{DateTime, Utc};
use diesel::{
  BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
  dsl::{exists, not},
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::{CommunityId, ModerationCaseId};
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseSeverity, CaseStatus, JuryAssignmentStatus},
  schema::{community, jury_assignment, moderation_case},
};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::LemmyResult;

/// Main-query row shape for the jury-queue view queries. All scalar
/// columns selected from the `jury_assignment` or `moderation_case`
/// tables; `deadline_at` is always built as `None` in the map step per
/// plan §2 Drift 3.
type JuryRow = (
  ModerationCaseId,
  CaseSeverity,
  String,
  DateTime<Utc>,
  Option<CommunityId>,
  Option<String>,
);

fn build_view(row: JuryRow) -> JuryQueueView {
  let (case_id, severity, reason_code, opened_at, community_id, community_name) = row;
  JuryQueueView {
    case_id: case_id.0,
    severity,
    reason_code,
    opened_at,
    deadline_at: None,
    community_id: community_id.map(|c| c.0),
    community_name,
  }
}

/// Cases the given person has already been assigned to as a juror, sorted
/// newest-first by `jury_assignment.selected_at` (plan §1 divergence 1 —
/// `selected_at`, not `accepted_at`, because the source struct has no
/// `accepted_at` column).
pub async fn list_jury_assignments_for_person(
  pool: &mut DbPool<'_>,
  person_id: PersonId,
) -> LemmyResult<Vec<JuryQueueView>> {
  let conn = &mut get_conn(pool).await?;

  let rows: Vec<JuryRow> = jury_assignment::table
    .inner_join(moderation_case::table)
    .left_join(community::table.on(community::id.nullable().eq(moderation_case::community_id)))
    .filter(jury_assignment::person_id.eq(person_id))
    .order_by(jury_assignment::selected_at.desc())
    .select((
      moderation_case::id,
      moderation_case::severity,
      moderation_case::reason_code,
      moderation_case::opened_at,
      moderation_case::community_id,
      community::name.nullable(),
    ))
    .load::<JuryRow>(conn)
    .await?;

  Ok(rows.into_iter().map(build_view).collect())
}

/// Count jury assignments whose status is `Selected` or `Accepted` — the
/// "unsubmitted" set for the Phase 4 background timeout job.
///
/// Phase 2a ships the unfiltered count. Per plan §2 Drift 3 the deadline
/// source is undecided, so this function returns the total count of
/// not-yet-submitted assignments across the whole instance; Phase 4 will
/// layer the `selected_at + <config_offset>` time filter on top when the
/// homeserver-side decision lands.
pub async fn count_unsubmitted_jury_assignments(pool: &mut DbPool<'_>) -> LemmyResult<i64> {
  let conn = &mut get_conn(pool).await?;
  let count: i64 = jury_assignment::table
    .filter(jury_assignment::status.eq_any([
      JuryAssignmentStatus::Selected,
      JuryAssignmentStatus::Accepted,
    ]))
    .count()
    .get_result(conn)
    .await?;
  Ok(count)
}

/// Cases the given person is eligible to opt into as a juror but is not
/// yet assigned to. v0 rule per [IMPLEMENTATION-PLAN-v0.md §Phase 2 task
/// 23]: simple filter, no reputation gating (Phase 5 adds that). The
/// filter is: case is in `ThresholdMet`, the person is not the target
/// and not the creator, and no `jury_assignment` row already links this
/// person to the case. Sort `opened_at.asc()` so the oldest-waiting
/// cases surface first.
///
/// `target_person_id` and `creator_id` are both `Nullable<Int4>` on
/// `moderation_case`, so the "not the target" and "not the creator"
/// filters are expressed as `IS NULL OR != person_id` to include cases
/// with no target person or no creator (e.g. federated reports) in the
/// result set. A bare `.ne(person_id)` would silently drop those rows
/// because `NULL != X` evaluates to UNKNOWN in SQL.
pub async fn list_available_jury_cases_for_person(
  pool: &mut DbPool<'_>,
  person_id: PersonId,
) -> LemmyResult<Vec<JuryQueueView>> {
  let conn = &mut get_conn(pool).await?;

  let rows: Vec<JuryRow> = moderation_case::table
    .left_join(community::table.on(community::id.nullable().eq(moderation_case::community_id)))
    .filter(moderation_case::status.eq(CaseStatus::ThresholdMet))
    .filter(
      moderation_case::target_person_id
        .is_null()
        .or(moderation_case::target_person_id.ne(person_id)),
    )
    .filter(
      moderation_case::creator_id
        .is_null()
        .or(moderation_case::creator_id.ne(person_id)),
    )
    .filter(not(exists(
      jury_assignment::table
        .filter(jury_assignment::case_id.eq(moderation_case::id))
        .filter(jury_assignment::person_id.eq(person_id)),
    )))
    .order_by(moderation_case::opened_at.asc())
    .select((
      moderation_case::id,
      moderation_case::severity,
      moderation_case::reason_code,
      moderation_case::opened_at,
      moderation_case::community_id,
      community::name.nullable(),
    ))
    .load::<JuryRow>(conn)
    .await?;

  Ok(rows.into_iter().map(build_view).collect())
}
