use crate::{GovernanceCaseDetailRow, GovernanceCaseDetailView, GovernanceCaseSummaryView};
use chrono::{DateTime, Utc};
use diesel::{
  ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl,
  SelectableHelper, dsl::count_star,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, ModerationCaseId},
  source::governance::{moderation_case::ModerationCase, sanction::Sanction},
};
use lemmy_db_schema_file::{
  PersonId,
  enums::{AppealStatus, CaseSeverity, CaseStatus, CaseTargetType, JuryAssignmentStatus},
  schema::{
    appeal, case_evidence, comment, community, jury_assignment, moderation_case, post, sanction,
  },
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
    .left_join(community::table.on(community::id.nullable().eq(moderation_case::community_id)))
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

/// All cases with `status = ThresholdMet` across the whole instance. Feeds
/// the Phase 4 background job that selects juries — no community scope on
/// purpose, and the `ThresholdMet` filter matches the task body "cases
/// whose reporter count has crossed the threshold and now need a jury."
pub async fn list_cases_needing_jury_selection(
  pool: &mut DbPool<'_>,
) -> LemmyResult<Vec<GovernanceCaseSummaryView>> {
  let conn = &mut get_conn(pool).await?;

  let rows: Vec<SummaryRow> = moderation_case::table
    .left_join(community::table.on(community::id.nullable().eq(moderation_case::community_id)))
    .filter(moderation_case::status.eq(CaseStatus::ThresholdMet))
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

/// All cases where `target_person_id` matches — a target sees every case
/// they're named in, active or closed. No community filter, no status
/// filter. Same two-round-trip pattern as `list_open_cases_for_community`.
pub async fn list_cases_for_person(
  pool: &mut DbPool<'_>,
  target_person_id: PersonId,
) -> LemmyResult<Vec<GovernanceCaseSummaryView>> {
  let conn = &mut get_conn(pool).await?;

  let rows: Vec<SummaryRow> = moderation_case::table
    .left_join(community::table.on(community::id.nullable().eq(moderation_case::community_id)))
    .filter(moderation_case::target_person_id.eq(target_person_id))
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

/// Filter bundle for the Phase 5c `list_cases_filtered` query. v0 surfaces
/// `community_id`, `status`, and pagination; `target_person_id` is wired
/// through but not yet reachable from the HTTP DTO — it's reserved for
/// the v1 assignee-filter per `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md`
/// Phase 5c task 67 GOTCHA.
#[derive(Debug, Clone, Copy, Default)]
pub struct CasesFilter {
  pub community_id: Option<CommunityId>,
  pub status: Option<CaseStatus>,
  pub target_person_id: Option<PersonId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

const DEFAULT_PAGE: i64 = 1;
const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 50;

/// Filtered case listing backing `GET /api/v4/governance/cases` (task 67).
///
/// Path-A per `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md`
/// Phase 5c task 67: sibling of `list_open_cases_for_community` rather
/// than a signature extension. Applies filters via a boxed query and
/// paginates at the SQL level (offset/limit) because the result set can be
/// large on a mature instance. Pagination bounds mirror `list_modlog`:
/// default page 1, default limit 20, max limit 50.
pub async fn list_cases_filtered(
  pool: &mut DbPool<'_>,
  filter: CasesFilter,
) -> LemmyResult<Vec<GovernanceCaseSummaryView>> {
  let conn = &mut get_conn(pool).await?;

  let page = filter.page.unwrap_or(DEFAULT_PAGE).max(1);
  let limit = filter.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
  let offset = page.saturating_sub(1).saturating_mul(limit);

  let mut query = moderation_case::table
    .left_join(community::table.on(community::id.nullable().eq(moderation_case::community_id)))
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
    .order_by((
      moderation_case::opened_at.desc(),
      moderation_case::id.desc(),
    ))
    .limit(limit)
    .offset(offset)
    .into_boxed();

  if let Some(cid) = filter.community_id {
    query = query.filter(moderation_case::community_id.eq(cid));
  }
  if let Some(st) = filter.status {
    query = query.filter(moderation_case::status.eq(st));
  }
  if let Some(pid) = filter.target_person_id {
    query = query.filter(moderation_case::target_person_id.eq(pid));
  }

  let rows: Vec<SummaryRow> = query.load::<SummaryRow>(conn).await?;

  let case_ids: Vec<ModerationCaseId> = rows.iter().map(|r| r.0).collect();
  let submitted_counts = submitted_counts_by_case(conn, &case_ids).await?;

  Ok(
    rows
      .into_iter()
      .map(|r| build_summary(r, &submitted_counts))
      .collect(),
  )
}

/// Fetch a `GovernanceCaseSummaryView` for a single case by id.
/// Returns `None` when no `moderation_case` row matches.
/// Mirrors the two-round-trip pattern of `list_open_cases_for_community`.
pub async fn read_summary_for_case(
  pool: &mut DbPool<'_>,
  case_id: ModerationCaseId,
) -> LemmyResult<Option<GovernanceCaseSummaryView>> {
  let conn = &mut get_conn(pool).await?;

  let row: Option<SummaryRow> = moderation_case::table
    .left_join(community::table.on(community::id.nullable().eq(moderation_case::community_id)))
    .filter(moderation_case::id.eq(case_id))
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
    .first::<SummaryRow>(conn)
    .await
    .optional()?;

  let Some(r) = row else {
    return Ok(None);
  };

  let submitted_counts = submitted_counts_by_case(conn, &[r.0]).await?;
  Ok(Some(build_summary(r, &submitted_counts)))
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
  let (
    case_id,
    status,
    severity,
    reason_code,
    opened_at,
    community_id,
    community_name,
    target_type,
  ) = row;
  let submitted = submitted_counts.get(&case_id).copied().unwrap_or(0);
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

/// Hydrated single-case detail view. Used by Phase 4 case-detail handler;
/// redaction and permission checks are the handler's job, not this
/// function's. Callers that match on `view.row.case_row.status` MUST
/// handle `CaseStatus::EmergencyRemove` and `CaseStatus::AdminReview`
/// exhaustively per ADR-013.
///
/// Four round-trips to keep each query mechanically simple:
/// 1. `moderation_case` row (Queryable on `ModerationCase`).
/// 2. `case_evidence` count grouped by nothing — a single scalar `i64`.
/// 3. Optional `appeal.status` for the most recent appeal on the case.
/// 4. `target_creator_id` resolved via COALESCE over post/comment/target_person_id.
/// 5. `sanctions` belonging to the case (fan-out, cannot share round-trip with 1-4).
///
/// The round-trip count isn't load-bearing — `read_case_detail` is called
/// per-request, not in bulk, so the latency cost is negligible compared to
/// the clarity gain from avoiding a giant hand-rolled tuple select.
pub async fn read_case_detail(
  pool: &mut DbPool<'_>,
  case_id: ModerationCaseId,
) -> LemmyResult<GovernanceCaseDetailView> {
  let conn = &mut get_conn(pool).await?;

  let case_row: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select(ModerationCase::as_select())
    .first(conn)
    .await?;

  let evidence_count: i64 = case_evidence::table
    .filter(case_evidence::case_id.eq(case_id))
    .count()
    .get_result(conn)
    .await?;

  // Most recent appeal for the case. `AppealStatus` is a `DbEnum`; the
  // column is `appeal::status` and we wrap it in `Option` since there may
  // be zero rows (the `.first().optional()` combinator handles the
  // no-row case without treating it as an error).
  let appeal_status: Option<AppealStatus> = appeal::table
    .filter(appeal::case_id.eq(case_id))
    .order_by(appeal::created_at.desc())
    .select(appeal::status)
    .first::<AppealStatus>(conn)
    .await
    .optional()?;

  let target_creator_id: Option<i32> = resolve_target_creator_id(conn, &case_row).await?;

  let sanctions: Vec<Sanction> = sanction::table
    .filter(sanction::case_id.eq(case_id))
    .select(Sanction::as_select())
    .load(conn)
    .await?;

  Ok(GovernanceCaseDetailView {
    row: GovernanceCaseDetailRow {
      case_row,
      evidence_count,
      appeal_status,
      target_creator_id,
    },
    sanctions,
  })
}

/// COALESCE target_post.creator_id, target_comment.creator_id,
/// moderation_case.target_person_id — expressed as three sequential
/// lookups in priority order, avoiding a raw SQL COALESCE that would
/// require an extra `sql::<...>` bridge. Returns the first non-None
/// value; `None` if the case has no post/comment/person target (e.g.
/// community or remote-instance target).
async fn resolve_target_creator_id(
  conn: &mut diesel_async::AsyncPgConnection,
  case_row: &ModerationCase,
) -> LemmyResult<Option<i32>> {
  if let Some(target_post_id) = case_row.target_post_id {
    let creator_id: Option<PersonId> = post::table
      .filter(post::id.eq(target_post_id))
      .select(post::creator_id)
      .first::<PersonId>(conn)
      .await
      .optional()?;
    if let Some(id) = creator_id {
      return Ok(Some(id.0));
    }
  }
  if let Some(target_comment_id) = case_row.target_comment_id {
    let creator_id: Option<PersonId> = comment::table
      .filter(comment::id.eq(target_comment_id))
      .select(comment::creator_id)
      .first::<PersonId>(conn)
      .await
      .optional()?;
    if let Some(id) = creator_id {
      return Ok(Some(id.0));
    }
  }
  Ok(case_row.target_person_id.map(|p| p.0))
}
