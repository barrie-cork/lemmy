//! `POST /api/v4/governance/admin/assign-jury` — admin backstop to seat a
//! jury panel with reputation gating + concurrent-cap.
//!
//! Phase 5b task 57 replaces the Phase 4 "not target AND not reporter AND not
//! deleted AND accepted_application" eligibility filter with the reputation-
//! gated form: INNER JOIN `reputation_snapshot` on `jury_eligible = true`
//! and a concurrent-cap subquery excluding jurors already at the
//! `jury.max_concurrent_assignments` cap. All five caps come from
//! `governance_config` via `ConfigCache`; `PANEL_SIZE` is now
//! `jury.panel_size`.
//!
//! The case is flipped `Open | ThresholdMet | EmergencyRemove` →
//! `JurySelection`; the plan's shorthand `InPanel` maps to the enum's
//! `JurySelection` variant (decision-queue #7). Per [99 ADR-013] the
//! status match is exhaustive; no `_ =>` catchall.
//!
//! All DB writes run inside one `run_transaction` so the 5 juror inserts +
//! the case update + the 6 log entries land atomically. Partial execution
//! would leave the case in a half-assembled panel state.
//!
//! ## Small-pool fallback
//!
//! Instances bootstrapping a community may not yet have 5 jury-eligible
//! persons with snapshot rows. When the strict filter returns fewer than
//! `jury.panel_size` candidates, the handler consults
//! `jury.fallback_on_small_pool`:
//! - `true` (seeded default) — re-run the Phase 4 filter without the
//!   reputation gate; a `warn!` log entry fires so operators notice.
//! - `false` — return `NotFound` so the admin learns the pool is too
//!   small rather than silently seating an unqualified panel.

use crate::governance::{
  actor_pseudonym_helper,
  config::{self, ConfigCache, Scope},
  governance_log,
};
use actix_web::web::{Data, Json};
use diesel::{
  ExpressionMethods, QueryDsl, QueryableByName, SelectableHelper,
  sql_query,
  sql_types::{Array, BigInt, Integer, Nullable},
  insert_into,
  update,
};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api_common::governance::{AdminAssignJury, AdminAssignJuryResponse};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::governance::{
  jury_assignment::JuryAssignmentInsertForm,
  moderation_case::ModerationCase,
};
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseStatus, JuryAssignmentStatus},
  schema::{jury_assignment, local_user, moderation_case, person},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{connection::get_conn, utils::functions::random};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;
use tracing::warn;

pub async fn admin_assign_jury(
  Json(data): Json<AdminAssignJury>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminAssignJuryResponse>> {
  is_admin(&local_user_view)?;

  let admin_id = local_user_view.person.id;
  let admin_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let data_for_tx = data;
  let pseudonym_for_tx = admin_pseudonym.clone();

  let outcome = conn
    .run_transaction(|conn| {
      async move { process_assignment(conn, pseudonym_for_tx, data_for_tx).await }.scope_boxed()
    })
    .await?;

  Ok(Json(outcome))
}

/// Body of the `run_transaction` closure. Kept under the workspace
/// `large_futures` lint threshold by matching the submit_jury_vote split.
async fn process_assignment(
  conn: &mut diesel_async::AsyncPgConnection,
  admin_pseudonym: String,
  data: AdminAssignJury,
) -> LemmyResult<AdminAssignJuryResponse> {
  // ConfigCache lives for the whole assignment transaction. Same shape as
  // submit_jury_vote::process_vote.
  let mut cache = ConfigCache::new();

  // 1. Read the case for the status guard + exclusion IDs.
  let case: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(data.case_id))
    .select(ModerationCase::as_select())
    .first(conn)
    .await?;

  // 2. Exhaustive status match per [99 ADR-013].
  match case.status {
    CaseStatus::Open | CaseStatus::ThresholdMet | CaseStatus::EmergencyRemove => {}
    CaseStatus::JurySelection
    | CaseStatus::InReview
    | CaseStatus::Decided
    | CaseStatus::Appealed
    | CaseStatus::Closed
    | CaseStatus::AdminReview => return Err(LemmyErrorType::NotFound.into()),
  }

  // 3. Read panel_size from config for this assignment + downstream check.
  let panel_size =
    config::get_int(&mut cache, &mut (&mut *conn).into(), Scope::Instance, "jury.panel_size")
      .await?;

  // 4. Select eligible jurors with reputation gating + concurrent-cap.
  //    Falls back to the Phase 4 unfiltered shape if the strict filter
  //    under-fills and `jury.fallback_on_small_pool` is true.
  let eligible = select_eligible_jurors(conn, &case, None, &mut cache).await?;
  let eligible_count = i64::try_from(eligible.len())
    .map_err(|_e| LemmyErrorType::Unknown("eligible count overflow".to_string()))?;
  if eligible_count < panel_size {
    return Err(LemmyErrorType::NotFound.into());
  }

  // 5. Insert JuryAssignment rows with status=Accepted (v0 testability).
  let forms: Vec<JuryAssignmentInsertForm> = eligible
    .iter()
    .map(|person_id| JuryAssignmentInsertForm {
      case_id: data.case_id,
      person_id: *person_id,
      status: JuryAssignmentStatus::Accepted,
    })
    .collect();
  insert_into(jury_assignment::table)
    .values(&forms)
    .execute(conn)
    .await?;

  // 6. Flip case → JurySelection (plan shorthand "InPanel").
  update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
    .set(moderation_case::status.eq(CaseStatus::JurySelection))
    .execute(conn)
    .await?;

  // 7. One governance_log "jury_assigned" row per juror. Each payload
  //    carries the per-juror pseudonym so the modlog stays pseudonymous
  //    per [99 ADR-015].
  for person_id in &eligible {
    let juror_pseudonym =
      actor_pseudonym_helper::get_or_create(&mut conn.into(), *person_id).await?;
    governance_log::append(
      &mut conn.into(),
      "jury_assigned",
      json!({
        "case_id": data.case_id.0,
        "juror_pseudonym": juror_pseudonym,
      }),
      Some(admin_pseudonym.clone()),
    )
    .await?;
  }

  // 8. Single panel_assembled marker — canonical record of Open→panel.
  governance_log::append(
    &mut conn.into(),
    "panel_assembled",
    json!({
      "case_id": data.case_id.0,
      "juror_count": panel_size,
    }),
    Some(admin_pseudonym.clone()),
  )
  .await?;

  Ok(AdminAssignJuryResponse {
    case_id: data.case_id,
    assigned_person_ids: eligible,
  })
}

/// Select up to `jury.panel_size` eligible jurors for the case. Phase 5b
/// task 57 tightens eligibility with a reputation gate and a concurrent-cap:
///
/// - INNER JOIN `reputation_snapshot` on the per-case community (instance-
///   scoped snapshot if `community_id` is NULL on both sides) with
///   `jury_eligible = true`.
/// - NOT IN subquery over `jury_assignment`: any person already holding
///   `jury.max_concurrent_assignments` `Selected` or `Accepted` rows is
///   excluded.
/// - Exclusion IDs: `case.target_person_id`, `case.creator_id`, plus every
///   id in `exclude_person_ids` (Phase 5c's decline-replacement path
///   threads its already-picked ids through this parameter).
///
/// If fewer than `panel_size` candidates are returned and
/// `jury.fallback_on_small_pool` is true, the call recurses into
/// [`legacy_select_eligible_jurors`] — the Phase 4 shape. Operators see a
/// `warn!` line when the fallback fires.
///
/// `pub(crate)` so `admin_emergency_remove` can reuse it for the post-facto
/// jury review without duplicating the query.
pub(crate) async fn select_eligible_jurors(
  conn: &mut diesel_async::AsyncPgConnection,
  case: &ModerationCase,
  exclude_person_ids: Option<&[PersonId]>,
  cache: &mut ConfigCache,
) -> LemmyResult<Vec<PersonId>> {
  let panel_size =
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance, "jury.panel_size").await?;
  let max_concurrent = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.max_concurrent_assignments",
  )
  .await?;
  let fallback_allowed = config::get_bool(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.fallback_on_small_pool",
  )
  .await?;

  // Build excluded_ids set: case.target + case.reporter + caller-provided.
  let mut excluded: Vec<PersonId> = Vec::new();
  if let Some(t) = case.target_person_id {
    excluded.push(t);
  }
  if let Some(r) = case.creator_id {
    excluded.push(r);
  }
  if let Some(xs) = exclude_person_ids {
    excluded.extend_from_slice(xs);
  }

  let eligible = run_strict_eligibility_query(
    conn,
    case.community_id,
    &excluded,
    max_concurrent,
    panel_size,
  )
  .await?;

  let strict_count = i64::try_from(eligible.len())
    .map_err(|_e| LemmyErrorType::Unknown("strict eligible pool overflow".to_string()))?;
  if strict_count < panel_size {
    if !fallback_allowed {
      return Ok(eligible);
    }
    warn!(
      community_id = ?case.community_id.map(|c| c.0),
      strict_pool = strict_count,
      panel_size,
      "jury pool below panel_size — falling back to unfiltered Phase 4 pool per config.jury.fallback_on_small_pool=true",
    );
    return legacy_select_eligible_jurors(conn, case, panel_size).await;
  }
  Ok(eligible)
}

/// Row shape for the raw-SQL strict eligibility query. Module-scope
/// because workspace lints deny `items-after-statements`.
#[derive(QueryableByName)]
struct StrictEligibilityRow {
  #[diesel(sql_type = Integer)]
  id: i32,
}

/// Raw-SQL variant of the strict eligibility query. Diesel's DSL cannot
/// express `IS NOT DISTINCT FROM` directly, and composing the
/// `NOT IN (... GROUP BY ... HAVING count(*) >= ?)` subquery in the typed
/// builder is finicky; a single parameterised `sql_query` is cleaner and
/// mirrors the Phase 5a task 53 pattern (see `reputation_snapshot` module).
///
/// The excluded ids ride as a single `int[]` array parameter (`$2`) so the
/// bind count is fixed and `sql_query`'s chained `.bind()` returns a
/// stable type (dynamic `for pid in …` rebinds don't compile — each
/// `.bind()` call produces a distinct `UncheckedBind<…>` type).
///
/// Parameter order:
/// - `$1` — community_id as nullable `Int4`
/// - `$2` — excluded person ids as `Int4[]`
/// - `$3` — `max_concurrent_assignments` (`BigInt`)
/// - `$4` — `panel_size` (`BigInt`)
async fn run_strict_eligibility_query(
  conn: &mut diesel_async::AsyncPgConnection,
  community_id: Option<lemmy_db_schema::newtypes::CommunityId>,
  excluded: &[PersonId],
  max_concurrent: i64,
  panel_size: i64,
) -> LemmyResult<Vec<PersonId>> {
  let community_bind: Option<i32> = community_id.map(|c| c.0);
  let excluded_bind: Vec<i32> = excluded.iter().map(|p| p.0).collect();

  let sql = "\
     SELECT p.id AS id \
     FROM person p \
     INNER JOIN local_user lu ON lu.person_id = p.id \
     INNER JOIN reputation_snapshot rs \
       ON rs.person_id = p.id \
       AND rs.community_id IS NOT DISTINCT FROM $1 \
     WHERE p.deleted = false \
       AND lu.accepted_application = true \
       AND rs.jury_eligible = true \
       AND p.id <> ALL($2) \
       AND p.id NOT IN ( \
         SELECT ja.person_id FROM jury_assignment ja \
         WHERE ja.status IN ('Selected', 'Accepted') \
         GROUP BY ja.person_id \
         HAVING count(*) >= $3 \
       ) \
     ORDER BY random() \
     LIMIT $4";

  let rows: Vec<StrictEligibilityRow> = sql_query(sql)
    .bind::<Nullable<Integer>, _>(community_bind)
    .bind::<Array<Integer>, _>(excluded_bind)
    .bind::<BigInt, _>(max_concurrent)
    .bind::<BigInt, _>(panel_size)
    .load(conn)
    .await?;
  Ok(rows.into_iter().map(|r| PersonId(r.id)).collect())
}

/// Phase 4 eligibility filter — "not target AND not reporter AND not
/// deleted AND accepted_application". Preserved verbatim so the small-pool
/// fallback path has a well-defined relaxed set. Does NOT consult
/// `reputation_snapshot`; does NOT apply the concurrent-cap.
async fn legacy_select_eligible_jurors(
  conn: &mut diesel_async::AsyncPgConnection,
  case: &ModerationCase,
  panel_size: i64,
) -> LemmyResult<Vec<PersonId>> {
  let target = case.target_person_id;
  let reporter = case.creator_id;

  let mut query = person::table
    .inner_join(local_user::table)
    .filter(person::deleted.eq(false))
    .filter(local_user::accepted_application.eq(true))
    .into_boxed();
  if let Some(target_id) = target {
    query = query.filter(person::id.ne(target_id));
  }
  if let Some(reporter_id) = reporter {
    query = query.filter(person::id.ne(reporter_id));
  }

  let eligible: Vec<PersonId> = query
    .order(random())
    .limit(panel_size)
    .select(person::id)
    .load::<PersonId>(conn)
    .await?;
  Ok(eligible)
}
