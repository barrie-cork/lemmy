//! `POST /api/v4/governance/report` — open or append to a moderation case.
//!
//! Phase 5b task 58 replaces the Phase 4 `V0_THRESHOLD` + `V0_REPORTER_WEIGHT`
//! stubs with the OQ-006 config-driven formula. The reporter's
//! `reporting_accuracy` is loaded from `reputation_snapshot` (or computed
//! on the fly via `load_or_compute_snapshot` when no snapshot row exists
//! yet) and fed into:
//!
//! ```text
//! weight_micros = base_weight
//!               * clamp(reporting_accuracy / 100, clamp_min, clamp_max)
//!               * exp(-hours_old / recency_half_life_hours)
//!               * 1_000_000
//! ```
//!
//! Case-flip threshold is `report.case_threshold_micros` (micros-scaled by
//! the Phase 5a task 50 migration). A hostile or nonsensical admin config
//! (`recency_half_life_hours = 0`, negative bases, etc.) can produce
//! `Inf`/`NaN`; the `is_finite()` guard logs a structured `error!` and
//! falls back to `base_weight × 1_000_000` so a request never fails on
//! config-induced non-finite math. [99 §17.2 carry-forward (3)]
//!
//! GDPR invariant: the `governance_log` entry's `actor_pseudonym` is
//! fetched via `actor_pseudonym_helper::get_or_create`, never the raw
//! PersonId.

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
use lemmy_api::governance::{
  actor_pseudonym_helper,
  case_open_snapshot,
  config::{self, ConfigCache, Scope},
  governance_log,
  reputation_snapshot,
};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, ModerationCaseId, PostId, RuleSetVersionId},
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

  // Phase 5b task 58: compute the OQ-006 weight from config + reporter
  // snapshot. ConfigCache lives for the whole handler invocation.
  let mut cache = ConfigCache::new();

  let reporter_snapshot =
    reputation_snapshot::load_or_compute_snapshot(conn, reporter_id, data.community_id, &mut cache)
      .await?;
  let base_weight =
    config::get_float(&mut cache, &mut conn.into(), Scope::Instance, "report.base_weight").await?;
  let clamp_min =
    config::get_float(&mut cache, &mut conn.into(), Scope::Instance, "report.clamp_min").await?;
  let clamp_max =
    config::get_float(&mut cache, &mut conn.into(), Scope::Instance, "report.clamp_max").await?;
  let half_life_hours = config::get_float(
    &mut cache,
    &mut conn.into(),
    Scope::Instance,
    "report.recency_half_life_hours",
  )
  .await?;
  let threshold_micros = config::get_int(
    &mut cache,
    &mut conn.into(),
    Scope::Instance,
    "report.case_threshold_micros",
  )
  .await?;

  let reporter_reputation =
    (f64::from(reporter_snapshot.reporting_accuracy) / 100.0).clamp(clamp_min, clamp_max);
  // Fresh report at write time; stale-report recomputation is v1.
  let hours_old = 0.0_f64;
  let recency_factor = (-hours_old / half_life_hours).exp();
  let weight_micros = compute_weight_micros(base_weight, reporter_reputation, recency_factor);

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
      let new_score = prior_score.saturating_add(weight_micros);
      let should_flip = matches!(prior_status, CaseStatus::Open) && new_score > threshold_micros;
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
      let initial_score = weight_micros;
      let initial_status = if initial_score > threshold_micros {
        CaseStatus::ThresholdMet
      } else {
        CaseStatus::Open
      };

      // Pin the case-open snapshot per v1-AD-c §4.1. The scope mirrors
      // the case's community: community-scoped reports pin against
      // Scope::Community(cid); instance-scoped reports (no
      // community_id) pin against Scope::Instance. In-flight juries
      // read the pinned snapshot so later admin-config edits cannot
      // retroactively change panel size, quorum, or thresholds
      // (ADR-010 append-only invariant).
      let case_scope = match data.community_id {
        Some(cid) => Scope::Community(cid),
        None => Scope::Instance,
      };
      let applied_config_snapshot =
        case_open_snapshot::build_applied_config_snapshot(&mut conn.into(), case_scope).await?;
      let active_version_i64 = config::get_int_opt(
        &mut cache,
        &mut conn.into(),
        case_scope,
        "rule_set.active_version_id",
      )
      .await?;
      let rule_set_version_id = active_version_i64
        .and_then(|i| i32::try_from(i).ok())
        .map(RuleSetVersionId);

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
        applied_config_snapshot: Some(applied_config_snapshot),
        rule_set_version_id,
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
      "reporter_reputation_multiplier": reporter_reputation,
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

/// Pure OQ-006 weight computation in micros.
///
/// Returns `(base_weight * reporter_reputation * recency_factor * 1_000_000)`
/// truncated to `i64`. When any input (or the product) is non-finite —
/// `NaN` or `±Inf` — logs a structured `error!` and falls back to
/// `(base_weight * 1_000_000)` also truncated to `i64`. A hostile or
/// typo'd admin config (`recency_half_life_hours = 0`, negative bases)
/// cannot propagate as a 500 to the caller. [99 §17.2 carry-forward (3)]
#[expect(
  clippy::as_conversions,
  reason = "f64 → i64 after explicit is_finite() guard; saturation semantics match the OQ-006 \
            spec (micros overflowing i64::MAX clamp at i64::MAX, mirroring Rust's `as` behaviour \
            for finite out-of-range floats)."
)]
fn compute_weight_micros(base_weight: f64, reporter_reputation: f64, recency_factor: f64) -> i64 {
  let weight_f64 = base_weight * reporter_reputation * recency_factor * 1_000_000.0;
  if weight_f64.is_finite() {
    weight_f64 as i64
  } else {
    tracing::error!(
      base_weight,
      reporter_reputation,
      recency_factor,
      "report-weight calculation produced non-finite value; falling back to base_weight × 1_000_000. \
       Likely cause: an admin-edited config key producing Inf/NaN (e.g. recency_half_life_hours = 0, \
       or a negative base_weight combined with an odd-exponent pow)."
    );
    let fallback = base_weight * 1_000_000.0;
    if fallback.is_finite() {
      fallback as i64
    } else {
      0
    }
  }
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

#[cfg(test)]
mod tests {
  use super::*;
  use tracing_test::traced_test;

  #[test]
  #[traced_test]
  fn is_finite_fallback_nan() {
    // recency_half_life_hours = 0 ⇒ (-0/0).exp() = NaN, which propagates
    // into the weight multiplication. The guard should log and fall back
    // to base_weight × 1_000_000.
    let base = 2.0_f64;
    let reputation = 1.0_f64;
    let recency = f64::NAN;
    let weight = compute_weight_micros(base, reputation, recency);
    assert_eq!(weight, 2_000_000, "NaN input must fall back to base × 1_000_000");
    assert!(logs_contain("non-finite"), "error! log must fire on non-finite input");
  }

  #[test]
  #[traced_test]
  fn is_finite_fallback_infinity() {
    let weight = compute_weight_micros(1.0, 1.0, f64::INFINITY);
    assert_eq!(weight, 1_000_000, "+Inf must fall back to base × 1_000_000");
    assert!(logs_contain("non-finite"));
  }

  #[test]
  fn finite_path_returns_product() {
    // 1.5 × 1.2 × 1.0 × 1_000_000 = 1_800_000 in real arithmetic; the IEEE-754
    // product is 1_799_999.999… so the `as i64` truncates to 1_799_999.
    // The test tolerates one-ULP truncation rather than asserting the
    // mathematical answer — documenting that the formula rounds toward zero.
    let weight = compute_weight_micros(1.5, 1.2, 1.0);
    assert!(
      (1_799_999..=1_800_000).contains(&weight),
      "weight {weight} not within ±1 of expected 1_800_000"
    );
  }
}
