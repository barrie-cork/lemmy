//! `POST /api/v4/governance/admin/trigger-appeal-rejury` — admin-triggered
//! appeal panel seating when `appeal.auto_select_on_appeal_acceptance = false`.
//!
//! The normal path in `request_appeal.rs` seats the panel automatically when
//! the config flag is true. When false, the appeal row is inserted with NULL
//! snapshot fields and the case sits in `Appealed` until an admin calls this
//! endpoint to complete the seating.
//!
//! Status match is exhaustive per [99 ADR-013]; no `_ =>` catchall.

use crate::governance::{
  actor_pseudonym_helper,
  admin_assign_jury::{seat_appeal_panel, select_appeal_panel},
  config::ConfigCache,
};
use actix_web::web::{Data, Json};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_api_common::governance::{AdminTriggerAppealRejury, AdminTriggerAppealRejuryResponse};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::governance::{appeal::Appeal, moderation_case::ModerationCase};
use lemmy_db_schema_file::{
  enums::{CaseStatus, JuryAssignmentRole},
  schema::{appeal, jury_assignment, moderation_case},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn admin_trigger_appeal_rejury(
  Json(data): Json<AdminTriggerAppealRejury>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminTriggerAppealRejuryResponse>> {
  is_admin(&local_user_view)?;

  let admin_id = local_user_view.person.id;
  let admin_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let data_for_tx = data;
  let pseudonym_for_tx = admin_pseudonym.clone();

  let outcome = conn
    .run_transaction(async |conn| {
      process_trigger_rejury(conn, pseudonym_for_tx, data_for_tx).await
    })
    .await?;

  Ok(Json(outcome))
}

async fn process_trigger_rejury(
  conn: &mut diesel_async::AsyncPgConnection,
  admin_pseudonym: String,
  data: AdminTriggerAppealRejury,
) -> LemmyResult<AdminTriggerAppealRejuryResponse> {
  // Load the case for the status guard. Exhaustive match per [99 ADR-013].
  let case: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(data.case_id))
    .select(ModerationCase::as_select())
    .first(conn)
    .await?;

  // TODO(type-state): single-variant guard — cleanest type-state candidate; replace with
  // GovernanceCase<Appealed>::try_from(case) — see
  // .claude/lessons/feedback_governance_type_state_handlers.md
  match case.status {
    CaseStatus::Appealed => {} // proceed
    CaseStatus::Open
    | CaseStatus::ThresholdMet
    | CaseStatus::JurySelection
    | CaseStatus::InReview
    | CaseStatus::Decided
    | CaseStatus::EmergencyRemove
    | CaseStatus::AdminReview
    | CaseStatus::Closed
    // PRD §3.3 + ADR-013: appeal rejury only valid on Appealed cases;
    // sponsor-liability lifecycle is orthogonal to the appeal lifecycle.
    | CaseStatus::SponsorLiabilityPending
    | CaseStatus::SponsorLiabilityFired
    | CaseStatus::SponsorLiabilityEscaped => return Err(LemmyErrorType::NotFound.into()),
  }

  // Idempotency: reject if appeal panel already seated for this case.
  let existing_appeal_assignments: i64 = jury_assignment::table
    .filter(jury_assignment::case_id.eq(data.case_id))
    .filter(jury_assignment::role.eq(JuryAssignmentRole::Appeal))
    .count()
    .get_result(conn)
    .await?;
  if existing_appeal_assignments > 0 {
    return Err(LemmyErrorType::AlreadyExists.into());
  }

  // Load the existing Appeal row filed by request_appeal when auto_select = false.
  let existing_appeal: Appeal = appeal::table
    .filter(appeal::case_id.eq(data.case_id))
    .select(Appeal::as_select())
    .first(conn)
    .await?;

  // Select and seat the appeal panel.
  let mut cache = ConfigCache::new();
  let selection = select_appeal_panel(conn, &case, &mut cache).await?;
  seat_appeal_panel(
    conn,
    case.id,
    existing_appeal.id,
    &selection,
    admin_pseudonym,
  )
  .await?;

  Ok(AdminTriggerAppealRejuryResponse {
    case_id: case.id,
    appeal_id: existing_appeal.id,
    panel_person_ids: selection.person_ids,
  })
}
