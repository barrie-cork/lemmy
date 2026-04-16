//! `POST /api/v4/governance/report` — open or append to a moderation case.
//!
//! v0 behaviour per [04 §6.1] and [05 §6]:
//! - No separate `report` table; reports collapse directly into
//!   `moderation_case`. Subsequent reports on the same (`target_type`,
//!   target id, community) increment the case's `threshold_score`.
//! - Reporter reputation weighting is stubbed at `1_i64` per report
//!   ([99 OQ-006] is deferred to v1).
//! - Threshold trigger: once `threshold_score > 3`, flip status from
//!   `Open` to `ThresholdMet` and emit a second `threshold_met` log
//!   entry on the same handler invocation.
//!
//! Both the case write and the log write are GDPR-compliant: the log
//! entry's `actor_pseudonym` is fetched via
//! `actor_pseudonym_helper::get_or_create`, never the raw PersonId.

use actix_web::web::{Data, Json};
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  OptionalExtension,
  QueryDsl,
  SelectableHelper,
  insert_into,
  update,
};
use diesel_async::RunQueryDsl;
use lemmy_api::governance::{actor_pseudonym_helper, governance_log};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, ModerationCaseId, PostId},
  source::{
    comment::Comment,
    community::Community,
    governance::moderation_case::{ModerationCase, ModerationCaseInsertForm},
    person::Person,
    post::Post,
  },
};
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseSeverity, CaseStatus, CaseTargetType},
  schema::moderation_case,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{connection::get_conn, traits::Crud};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;

use lemmy_api_common::governance::{CreateGovernanceReport, CreateGovernanceReportResponse};

/// Threshold at which an `Open` case flips to `ThresholdMet` per
/// [99 OQ-006] interim v0 constant. Phase 5 replaces this with a
/// reputation-weighted formula.
const V0_THRESHOLD: i64 = 3;

/// Reporter weight for the threshold score. Stubbed at `1_i64` per
/// reporter in v0 — [99 OQ-006] interim. TODO(brehon-fork): tune by
/// reporter reputation in Phase 5.
const V0_REPORTER_WEIGHT: i64 = 1;

pub async fn create_report(
  Json(data): Json<CreateGovernanceReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CreateGovernanceReportResponse>> {
  check_local_user_valid(&local_user_view)?;

  let reason_code = data.reason_code.trim().to_string();
  if reason_code.is_empty() {
    return Err(LemmyErrorType::ReportReasonRequired.into());
  }
  if reason_code.chars().count() > 1000 {
    return Err(LemmyErrorType::ReportTooLong.into());
  }

  let reporter_id = local_user_view.person.id;
  let TargetRefs {
    target_post_id,
    target_comment_id,
    target_person_id,
    target_community_id,
    target_remote_url,
  } = resolve_target(&context, data.target_type, data.target_id).await?;

  let pool_ref = &mut context.pool();
  let conn = &mut get_conn(pool_ref).await?;

  let existing: Option<(ModerationCaseId, i64, CaseStatus)> = moderation_case::table
    .filter(moderation_case::target_type.eq(data.target_type))
    .filter(
      moderation_case::status
        .ne(CaseStatus::Decided)
        .and(moderation_case::status.ne(CaseStatus::Closed)),
    )
    .filter(match_target_filter(
      data.target_type,
      target_post_id,
      target_comment_id,
      target_person_id,
      target_community_id,
      target_remote_url.as_deref(),
    ))
    .select((
      moderation_case::id,
      moderation_case::threshold_score,
      moderation_case::status,
    ))
    .first::<(ModerationCaseId, i64, CaseStatus)>(conn)
    .await
    .optional()?;

  let (case_id, new_score, just_met_threshold) = match existing {
    Some((case_id, prior_score, prior_status)) => {
      let new_score = prior_score.saturating_add(V0_REPORTER_WEIGHT);
      let should_flip = matches!(prior_status, CaseStatus::Open) && new_score > V0_THRESHOLD;
      let target_status = if should_flip {
        CaseStatus::ThresholdMet
      } else {
        prior_status
      };
      update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
        .set((
          moderation_case::threshold_score.eq(new_score),
          moderation_case::status.eq(target_status),
        ))
        .execute(conn)
        .await?;
      (case_id, new_score, should_flip)
    }
    None => {
      let initial_score = V0_REPORTER_WEIGHT;
      let initial_status = if initial_score > V0_THRESHOLD {
        CaseStatus::ThresholdMet
      } else {
        CaseStatus::Open
      };
      let form = ModerationCaseInsertForm {
        community_id: data.community_id,
        creator_id: Some(reporter_id),
        target_type: data.target_type,
        target_post_id,
        target_comment_id,
        target_person_id,
        target_community_id,
        target_remote_url,
        reason_code: reason_code.clone(),
        severity: CaseSeverity::default(),
        status: initial_status,
        threshold_score: initial_score,
      };
      let row = insert_into(moderation_case::table)
        .values(&form)
        .returning(ModerationCase::as_returning())
        .get_result::<ModerationCase>(conn)
        .await?;
      (
        row.id,
        initial_score,
        matches!(initial_status, CaseStatus::ThresholdMet),
      )
    }
  };

  let pseudonym = actor_pseudonym_helper::get_or_create(pool_ref, reporter_id).await?;

  governance_log::append(
    pool_ref,
    "report_created",
    json!({
      "case_id": case_id.0,
      "target_type": data.target_type,
      "threshold_score": new_score,
      "threshold_met": just_met_threshold,
    }),
    Some(pseudonym.clone()),
  )
  .await?;

  if just_met_threshold {
    governance_log::append(
      pool_ref,
      "threshold_met",
      json!({
        "case_id": case_id.0,
        "threshold_score": new_score,
      }),
      Some(pseudonym),
    )
    .await?;
  }

  Ok(Json(CreateGovernanceReportResponse {
    case_id: Some(case_id),
    threshold_met: just_met_threshold,
  }))
}

struct TargetRefs {
  target_post_id: Option<PostId>,
  target_comment_id: Option<CommentId>,
  target_person_id: Option<PersonId>,
  target_community_id: Option<CommunityId>,
  target_remote_url: Option<String>,
}

async fn resolve_target(
  context: &Data<LemmyContext>,
  target_type: CaseTargetType,
  target_id: i32,
) -> LemmyResult<TargetRefs> {
  match target_type {
    CaseTargetType::Post => {
      let post_id = PostId(target_id);
      Post::read(&mut context.pool(), post_id).await?;
      Ok(TargetRefs {
        target_post_id: Some(post_id),
        target_comment_id: None,
        target_person_id: None,
        target_community_id: None,
        target_remote_url: None,
      })
    }
    CaseTargetType::Comment => {
      let comment_id = CommentId(target_id);
      Comment::read(&mut context.pool(), comment_id).await?;
      Ok(TargetRefs {
        target_post_id: None,
        target_comment_id: Some(comment_id),
        target_person_id: None,
        target_community_id: None,
        target_remote_url: None,
      })
    }
    CaseTargetType::Person => {
      let person_id = PersonId(target_id);
      Person::read(&mut context.pool(), person_id).await?;
      Ok(TargetRefs {
        target_post_id: None,
        target_comment_id: None,
        target_person_id: Some(person_id),
        target_community_id: None,
        target_remote_url: None,
      })
    }
    CaseTargetType::Community => {
      let community_id = CommunityId(target_id);
      Community::read(&mut context.pool(), community_id).await?;
      Ok(TargetRefs {
        target_post_id: None,
        target_comment_id: None,
        target_person_id: None,
        target_community_id: Some(community_id),
        target_remote_url: None,
      })
    }
    CaseTargetType::RemoteInstance => Err(LemmyErrorType::NotFound.into()),
  }
}

/// Type-checked filter for the "find existing open case on same target"
/// query. Emitted as a boxed expression so the caller can attach it to
/// the query builder without the compiler needing to reconcile the five
/// concrete `Eq` specialisations into a single `impl BoxableExpression`.
///
/// `RemoteInstance` is rejected upstream in [`resolve_target`] for v0,
/// so the match below never actually lands in the `RemoteInstance` arm
/// at runtime — but the arm is present for exhaustiveness.
fn match_target_filter(
  target_type: CaseTargetType,
  target_post_id: Option<PostId>,
  target_comment_id: Option<CommentId>,
  target_person_id: Option<PersonId>,
  target_community_id: Option<CommunityId>,
  target_remote_url: Option<&str>,
) -> Box<
  dyn diesel::BoxableExpression<
      moderation_case::table,
      diesel::pg::Pg,
      SqlType = diesel::sql_types::Nullable<diesel::sql_types::Bool>,
    >,
> {
  match target_type {
    CaseTargetType::Post => Box::new(moderation_case::target_post_id.eq(target_post_id)),
    CaseTargetType::Comment => Box::new(moderation_case::target_comment_id.eq(target_comment_id)),
    CaseTargetType::Person => Box::new(moderation_case::target_person_id.eq(target_person_id)),
    CaseTargetType::Community => {
      Box::new(moderation_case::target_community_id.eq(target_community_id))
    }
    CaseTargetType::RemoteInstance => Box::new(
      moderation_case::target_remote_url.eq(target_remote_url.map(str::to_string)),
    ),
  }
}
