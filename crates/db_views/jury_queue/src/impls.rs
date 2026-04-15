use crate::JuryQueueView;
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::{CommunityId, ModerationCaseId};
use lemmy_db_schema_file::{
  PersonId,
  enums::CaseSeverity,
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
    .left_join(
      community::table.on(community::id.nullable().eq(moderation_case::community_id)),
    )
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
