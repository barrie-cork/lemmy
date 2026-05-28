//! `EmergencyRemove` wiring per [99 ADR-013] and [06 §2.2.1].
//!
//! This helper is callable from a future emergency-remove HTTP route or
//! from a direct admin tool. v0 does NOT expose an HTTP surface for it —
//! the function exists so the cross-cutting `EmergencyRemove` requirement
//! ([IMPLEMENTATION-PLAN-v0.md §4.3]) is wired through the codebase and
//! so integration tests can exercise the code path.
//!
//! ## Per [99 ADR-013]: the jury CANNOT un-remove the content.
//!
//! The post-facto jury review produces accountability artefacts (vote
//! records, reputation deltas for the admin who pressed the button) but
//! the content stays down regardless of outcome. The helper's governance
//! log entry is tagged `entry_kind = "emergency_removed"` and is
//! extra-visible per [06 §2.2.1].

use crate::governance::{
  actor_pseudonym_helper, admin_assign_jury,
  config::{self, ConfigCache, Scope},
  governance_log::{self, ENTRY_KIND_EVIDENCE_QUALITY_RECORDED, ENTRY_KIND_SEVERITY_TIER_FROZEN},
};
use activitypub_federation::config::Data;
use actix_web::web::Json;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, insert_into, update};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_api_common::governance::{FlagBadFaithEmergencyReport, FlagBadFaithEmergencyReportResponse};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, ModerationCaseId, PostId},
  source::governance::{
    jury_assignment::JuryAssignmentInsertForm,
    moderation_case::{ModerationCase, ModerationCaseInsertForm},
    reputation_event::ReputationEventInsertForm,
  },
};
use lemmy_db_schema_file::{
  PersonId,
  enums::{
    CaseSeverity, CaseStatus, CaseStatusTier, CaseTargetType, JuryAssignmentStatus,
    ReputationDimension, ReputationEventSourceType, SeverityTier,
  },
  schema::{comment, community, jury_assignment, moderation_case, post, reputation_event},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;

/// Scoped target for an emergency removal. Encodes both the target type
/// and the target entity's newtype id so callers can't pass an
/// incoherent (Post + CommentId) combination.
#[derive(Debug, Clone, Copy)]
pub enum EmergencyRemoveTarget {
  Post(PostId),
  Comment(CommentId),
  Community(CommunityId),
}

impl EmergencyRemoveTarget {
  fn case_target_type(self) -> CaseTargetType {
    match self {
      Self::Post(_) => CaseTargetType::Post,
      Self::Comment(_) => CaseTargetType::Comment,
      Self::Community(_) => CaseTargetType::Community,
    }
  }
}

/// Admin-invoked emergency removal.
///
/// Steps, all inside a single `run_transaction` so a partial failure
/// never leaves the site in an illegal-content state:
///
/// 1. Flip the target's `removed` column to `true`. TODO(brehon-fork):
///    this is a stub per decision-queue #6; Phase 5 routes this call
///    through Lemmy's canonical remove pathway.
/// 2. Insert a `ModerationCase` with `status = EmergencyRemove`.
/// 3. Select 5 eligible jurors and seat them as `Accepted` — reuses
///    `admin_assign_jury::select_eligible_jurors`.
/// 4. Emit a governance log entry tagged `emergency_removed` with a
///    `visibility: "extra-visible"` marker per [06 §2.2.1].
pub async fn emergency_remove_open_case(
  pool: &mut DbPool<'_>,
  admin_id: PersonId,
  target: EmergencyRemoveTarget,
  community_id: Option<CommunityId>,
  reason: String,
) -> LemmyResult<ModerationCaseId> {
  let admin_pseudonym = actor_pseudonym_helper::get_or_create(pool, admin_id).await?;
  let conn = &mut get_conn(pool).await?;

  let admin_pseudonym_for_tx = admin_pseudonym.clone();
  let reason_for_tx = reason;

  conn
    .run_transaction(async |conn| {
      process_emergency_remove(
        conn,
        admin_id,
        admin_pseudonym_for_tx,
        target,
        community_id,
        reason_for_tx,
      )
      .await
    })
    .await
}

async fn process_emergency_remove(
  conn: &mut AsyncPgConnection,
  admin_id: PersonId,
  admin_pseudonym: String,
  target: EmergencyRemoveTarget,
  community_id: Option<CommunityId>,
  reason: String,
) -> LemmyResult<ModerationCaseId> {
  // 1. Force the target's removed flag to true. Stub per decision-queue
  //    #6; Phase 5 routes this through Lemmy's canonical remove pathway.
  // TODO(brehon-fork): wire to canonical Lemmy remove helper in Phase 5.
  match target {
    EmergencyRemoveTarget::Post(post_id) => {
      update(post::table.find(post_id))
        .set(post::removed.eq(true))
        .execute(conn)
        .await?;
    }
    EmergencyRemoveTarget::Comment(comment_id) => {
      update(comment::table.find(comment_id))
        .set(comment::removed.eq(true))
        .execute(conn)
        .await?;
    }
    EmergencyRemoveTarget::Community(community_id_arg) => {
      update(community::table.find(community_id_arg))
        .set(community::removed.eq(true))
        .execute(conn)
        .await?;
    }
  }

  // 2. Open a case with EmergencyRemove status.
  let (target_post_id, target_comment_id, target_community_id) = match target {
    EmergencyRemoveTarget::Post(id) => (Some(id), None, None),
    EmergencyRemoveTarget::Comment(id) => (None, Some(id), None),
    EmergencyRemoveTarget::Community(id) => (None, None, Some(id)),
  };
  let form = ModerationCaseInsertForm {
    community_id,
    creator_id: Some(admin_id),
    target_type: target.case_target_type(),
    target_post_id,
    target_comment_id,
    target_person_id: None,
    target_community_id,
    target_remote_url: None,
    reason_code: "emergency_remove".to_string(),
    severity: CaseSeverity::default(),
    severity_tier: Some(SeverityTier::Severe),
    status: CaseStatus::EmergencyRemove,
    threshold_score: 0,
    ..Default::default()
  };
  let case_row: ModerationCase = insert_into(moderation_case::table)
    .values(&form)
    .returning(ModerationCase::as_returning())
    .get_result(conn)
    .await?;
  let case_id = case_row.id;

  // 3. Post-facto jury — reuse the eligibility logic from admin_assign_jury.
  //    Per Phase 5b task 57 + v1-JM-b task 4, the filter is reputation-
  //    gated + concurrent-capped with diversity / cooldown constraints;
  //    the small-pool fallback keeps behaviour defined on bootstrapping
  //    instances. v1-JM-b task 6 flipped the case's `severity_tier` to
  //    `Severe`; PR #95 cr-3 routes the panel-seating step through the
  //    same cascade + snapshot + governance_log path that
  //    `admin_assign_jury::process_assignment` uses, so emergency-remove
  //    cases get the larger Severe-tier panel they deserve and their
  //    snapshot fields freeze at seat-time (consistent with ADR-010 —
  //    no retroactive invalidation by mid-flight config changes).
  let mut cache = ConfigCache::new();
  let severity = case_row.severity_tier;
  let status = if case_row.status_tier == CaseStatusTier::Regular {
    admin_assign_jury::compute_status_tier(conn, &case_row).await?
  } else {
    case_row.status_tier
  };
  let status_str = admin_assign_jury::status_tier_slug(status);
  let severity_str = admin_assign_jury::severity_tier_slug(severity);

  let panel_size = config::get_int_cascade(
    &mut cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.panel_size",
    &[status_str, severity_str],
  )
  .await?;
  let quorum_fraction = config::get_float_cascade(
    &mut cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.quorum_fraction",
    &[severity_str],
  )
  .await?;
  let threshold_fraction = config::get_float_cascade(
    &mut cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.threshold_fraction",
    &[severity_str],
  )
  .await?;

  let panel_size_i32: i32 = i32::try_from(panel_size).map_err(|_e| {
    lemmy_utils::error::LemmyErrorType::Unknown(format!("panel_size {panel_size} out of i32 range"))
  })?;
  let quorum =
    admin_assign_jury::ceil_count(f64::from(panel_size_i32) * quorum_fraction, "quorum")?;
  let threshold_count = admin_assign_jury::ceil_count(
    f64::from(panel_size_i32) * threshold_fraction,
    "threshold_count",
  )?;

  let (eligible, record) =
    admin_assign_jury::select_eligible_jurors(conn, &case_row, panel_size, None, &mut cache)
      .await?;

  // 3a. Snapshot the resolved tier + counts onto the case row. Mirrors
  //     `admin_assign_jury::process_assignment` step 6 (plan §10.5) but
  //     without the `status → JurySelection` flip — the case is already
  //     in `EmergencyRemove` status and stays there per ADR-013.
  update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
    .set((
      moderation_case::panel_size_snapshot.eq(Some(panel_size_i32)),
      moderation_case::quorum_snapshot.eq(Some(quorum)),
      moderation_case::threshold_count_snapshot.eq(Some(threshold_count)),
      moderation_case::status_tier.eq(status),
    ))
    .execute(conn)
    .await?;

  // 3b. severity_tier_frozen governance_log entry — admin is the actor.
  //     Plan §10.6 mirror.
  governance_log::append(
    &mut conn.into(),
    ENTRY_KIND_SEVERITY_TIER_FROZEN,
    json!({
      "case_id": case_id.0,
      "severity_tier": severity_str,
      "status_tier": status_str,
      "panel_size_snapshot": panel_size_i32,
      "quorum_snapshot": quorum,
      "threshold_count_snapshot": threshold_count,
    }),
    Some(admin_pseudonym.clone()),
  )
  .await?;

  if !eligible.is_empty() {
    // 3c. Persist the ConstraintRecord per-juror via JSONB. Plan §10.11
    //     mirror — same shape as process_assignment so a later
    //     decline/replacement reads the originally seated context.
    let constraints_applied_json = record.to_json();
    let forms: Vec<JuryAssignmentInsertForm> = eligible
      .iter()
      .map(|person_id| JuryAssignmentInsertForm {
        case_id,
        person_id: *person_id,
        status: JuryAssignmentStatus::Accepted,
        selected_under_constraints: Some(constraints_applied_json.clone()),
        ..Default::default()
      })
      .collect();
    insert_into(jury_assignment::table)
      .values(&forms)
      .execute(conn)
      .await?;
    for person_id in &eligible {
      let juror_pseudonym =
        actor_pseudonym_helper::get_or_create(&mut conn.into(), *person_id).await?;
      governance_log::append(
        &mut conn.into(),
        "jury_assigned",
        json!({
          "case_id": case_id.0,
          "juror_pseudonym": juror_pseudonym,
        }),
        Some(admin_pseudonym.clone()),
      )
      .await?;
    }

    // 3d. Extended panel_assembled payload — same shape as
    //     process_assignment §10.12 so the audit timeline of
    //     emergency-remove cases reads the same as normal cases.
    governance_log::append(
      &mut conn.into(),
      "panel_assembled",
      json!({
        "case_id": case_id.0,
        "juror_count": panel_size_i32,
        "severity_tier": severity_str,
        "status_tier": status_str,
        "constraints_applied": {
          "no_majority_from_same_sponsor_cluster": record.no_majority_from_same_sponsor_cluster,
          "geographic_diversity_preferred": record.geographic_diversity_preferred,
          "no_recent_juror_repeat": record.no_recent_juror_repeat,
          "no_same_endorsement_chain": record.no_same_endorsement_chain,
        },
        "relaxations": record.relaxations_fired,
      }),
      Some(admin_pseudonym.clone()),
    )
    .await?;
  }

  // 4. Extra-visible log entry per [06 §2.2.1]. The jury CANNOT
  //    un-remove the content per [99 ADR-013]; this entry is the
  //    public record of the admin-initiated removal.
  governance_log::append(
    &mut conn.into(),
    "emergency_removed",
    json!({
      "case_id": case_id.0,
      "target_type": target.case_target_type(),
      "reason": reason,
      "visibility": "extra-visible",
    }),
    Some(admin_pseudonym),
  )
  .await?;

  Ok(case_id)
}

/// Admin-flagged bad-faith report against an EmergencyRemove-status
/// case. Emits `-1 reporting_accuracy` for the case reporter via a
/// dedupe-keyed `reputation_event` row plus an
/// `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED` governance_log entry
/// attributed to the admin. PRD section 5.3 source 4b + ADR-013.
pub async fn flag_bad_faith_emergency_report(
  Json(data): Json<FlagBadFaithEmergencyReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<FlagBadFaithEmergencyReportResponse>> {
  is_admin(&local_user_view)?;

  let admin_id = local_user_view.person.id;
  let admin_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?;
  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let case_id = data.case_id;
  let admin_pseudonym_for_tx = admin_pseudonym;

  let flagged = conn
    .run_transaction(async |conn| {
      process_flag_bad_faith(conn, admin_pseudonym_for_tx, case_id).await
    })
    .await?;

  Ok(Json(FlagBadFaithEmergencyReportResponse {
    case_id,
    flagged,
  }))
}

async fn process_flag_bad_faith(
  conn: &mut AsyncPgConnection,
  admin_pseudonym: String,
  case_id: ModerationCaseId,
) -> LemmyResult<bool> {
  // 1. Load case + assert status.
  let case_row: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select(ModerationCase::as_select())
    .first(conn)
    .await
    .map_err(|_e| LemmyErrorType::NotFound)?;
  if case_row.status != CaseStatus::EmergencyRemove {
    return Err(
      LemmyErrorType::Unknown(format!(
        "case {} is in status {:?}; flag-bad-faith requires EmergencyRemove",
        case_id.0, case_row.status
      ))
      .into(),
    );
  }
  let reporter_id = case_row.creator_id.ok_or_else(|| {
    LemmyErrorType::Unknown(format!(
      "case {} has no reporter (creator_id); flag-bad-faith requires a reporter",
      case_id.0
    ))
  })?;

  // 2. Emit reputation_event + governance_log.
  let reporter_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), reporter_id).await?;
  let mut cache = ConfigCache::new();
  let evidence_delta_i64 = config::get_int(
    &mut cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "deltas.evidence_bad_faith",
  )
  .await?;
  let evidence_delta = i32::try_from(evidence_delta_i64).map_err(|_e| {
    LemmyErrorType::Unknown(format!(
      "deltas.evidence_bad_faith ({evidence_delta_i64}) overflows i32"
    ))
  })?;
  // file-private emit_reputation_event_local helper (mirrors
  // submit_jury_vote.rs:968 verbatim but lives in admin_emergency_remove.rs).
  emit_reputation_event_local(
    conn,
    reporter_id,
    case_row.community_id,
    ReputationDimension::ReportingAccuracy,
    evidence_delta,
    case_id,
    "evidence_bad_faith",
    ReputationEventSourceType::EvidenceQuality,
    Some(format!("evidence_bad_faith:{}", case_id.0)),
  )
  .await?;
  governance_log::append(
    &mut conn.into(),
    ENTRY_KIND_EVIDENCE_QUALITY_RECORDED,
    json!({
      "case_id": case_id.0,
      "reporter_pseudonym": reporter_pseudonym,
      "dimension": "reporting_accuracy",
      "delta": evidence_delta,
      "source_event_type": "evidence_quality",
      "trigger": "admin_flagged_bad_faith",
    }),
    Some(admin_pseudonym),
  )
  .await?;
  Ok(true)
}

#[expect(clippy::too_many_arguments)]
async fn emit_reputation_event_local(
  conn: &mut AsyncPgConnection,
  person_id: PersonId,
  community_id: Option<CommunityId>,
  dimension: ReputationDimension,
  delta: i32,
  source_case_id: ModerationCaseId,
  reason: &str,
  source_event_type: ReputationEventSourceType,
  dedupe_key: Option<String>,
) -> LemmyResult<()> {
  let form = ReputationEventInsertForm {
    person_id,
    community_id,
    dimension,
    delta,
    source_case_id: Some(source_case_id),
    source_report_id: None,
    reason: reason.to_string(),
    expires_at: None,
    dedupe_key,
    source_event_type: Some(source_event_type),
  };
  insert_into(reputation_event::table)
    .values(&form)
    .on_conflict_do_nothing()
    .execute(conn)
    .await?;
  Ok(())
}
