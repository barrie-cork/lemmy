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
  actor_pseudonym_helper,
  admin_assign_jury,
  config::ConfigCache,
  governance_log,
};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, insert_into, update};
use diesel_async::{AsyncPgConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, ModerationCaseId, PostId},
  source::governance::{
    jury_assignment::JuryAssignmentInsertForm,
    moderation_case::{ModerationCase, ModerationCaseInsertForm},
  },
};
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseSeverity, CaseStatus, CaseTargetType, JuryAssignmentStatus},
  schema::{comment, community, jury_assignment, moderation_case, post},
};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::LemmyResult;
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
    .run_transaction(|conn| {
      async move {
        process_emergency_remove(
          conn,
          admin_id,
          admin_pseudonym_for_tx,
          target,
          community_id,
          reason_for_tx,
        )
        .await
      }
      .scope_boxed()
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
  //    Per Phase 5b task 57, the filter is reputation-gated + concurrent-
  //    capped; the small-pool fallback keeps behaviour defined on
  //    bootstrapping instances.
  let mut cache = ConfigCache::new();
  let eligible =
    admin_assign_jury::select_eligible_jurors(conn, &case_row, None, &mut cache).await?;
  if !eligible.is_empty() {
    let forms: Vec<JuryAssignmentInsertForm> = eligible
      .iter()
      .map(|person_id| JuryAssignmentInsertForm {
        case_id,
        person_id: *person_id,
        status: JuryAssignmentStatus::Accepted,
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
