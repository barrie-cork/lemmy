//! `POST /api/v4/governance/admin/close-case` — admin force-close backstop.
//!
//! Single-admin v0 simplification per [99 ADR-010]; quorum + delay on
//! admin close-case is a v2 item. `reason` is required and flows through
//! `governance_log::append`, which scrubs identifiers in-place via the
//! redaction layer — the handler does NOT scrub `reason` directly.
//!
//! Status match is exhaustive per [99 ADR-013]; no `_ =>` catchall.

use crate::governance::{actor_pseudonym_helper, governance_log};
use actix_web::web::{Data, Json};
use diesel::{ExpressionMethods, NullableExpressionMethods, QueryDsl, SelectableHelper, update};
use diesel_async::RunQueryDsl;
use lemmy_api_common::governance::{AdminCloseCase, AdminCloseCaseResponse};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::governance::moderation_case::ModerationCase;
use lemmy_db_schema_file::{enums::CaseStatus, schema::moderation_case};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{connection::get_conn, utils::now};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;

pub async fn admin_close_case(
  Json(data): Json<AdminCloseCase>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminCloseCaseResponse>> {
  is_admin(&local_user_view)?;

  if data.reason.trim().is_empty() {
    return Err(LemmyErrorType::Unknown("close-case reason required".to_string()).into());
  }

  let admin_id = local_user_view.person.id;
  let admin_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let data_for_tx = data.clone();
  let pseudonym_for_tx = admin_pseudonym.clone();

  let outcome = conn
    .run_transaction(async |conn| {
      process_close(conn, pseudonym_for_tx, data_for_tx).await
    })
    .await?;

  Ok(Json(outcome))
}

async fn process_close(
  conn: &mut diesel_async::AsyncPgConnection,
  admin_pseudonym: String,
  data: AdminCloseCase,
) -> LemmyResult<AdminCloseCaseResponse> {
  // Read the case for the status guard. Exhaustive match per [99 ADR-013].
  let case: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(data.case_id))
    .select(ModerationCase::as_select())
    .first(conn)
    .await?;

  // TODO(type-state): inverted guard (reject Closed only, accept all others) — model as
  // GovernanceCase<NotYetClosed> via exhaustive allow-list TryFrom covering 12 variants —
  // see .claude/lessons/feedback_governance_type_state_handlers.md
  match case.status {
    CaseStatus::Open
    | CaseStatus::ThresholdMet
    | CaseStatus::JurySelection
    | CaseStatus::InReview
    | CaseStatus::Decided
    | CaseStatus::Appealed
    | CaseStatus::EmergencyRemove
    | CaseStatus::AdminReview
    // PRD §3.3 + ADR-013: admins may force-close terminal liability states for
    // ops purposes (e.g. scheduler stuck in SponsorLiabilityPending).
    | CaseStatus::SponsorLiabilityPending
    | CaseStatus::SponsorLiabilityFired
    | CaseStatus::SponsorLiabilityEscaped => {}
    CaseStatus::Closed => return Err(LemmyErrorType::NotFound.into()),
  }

  update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
    .set((
      moderation_case::status.eq(CaseStatus::Closed),
      moderation_case::closed_at.eq(now().nullable()),
    ))
    .execute(conn)
    .await?;

  governance_log::append(
    &mut conn.into(),
    "admin_case_closed",
    json!({
      "case_id": data.case_id.0,
      "reason": data.reason,
    }),
    Some(admin_pseudonym),
  )
  .await?;

  Ok(AdminCloseCaseResponse {
    case_id: data.case_id,
    closed: true,
  })
}
