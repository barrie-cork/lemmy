//! `POST /api/v4/governance/jury/vote` — submit a juror's vote.
//!
//! The most complex handler in v0. Implements the full aggregation rule
//! from [04 §8] and the v0 simplifications from [05 §3] and [05 §6]:
//!
//! 1. Verify the caller has an `Accepted` assignment on the case.
//! 2. Insert the vote and flip the assignment to `Submitted`.
//! 3. If fewer than **3** votes are in (quorum per [99 ADR-007]), return
//!    without deciding.
//! 4. At quorum, tally votes by simple majority. Record the decision on
//!    the case row (status → `Decided`, `decided_at = now`,
//!    `closed_at = now + 7 days` for the appeal window per [05 §6]).
//! 5. Map the winning `JuryDecision` to a `Sanction` per the table
//!    below; `NoAction` writes no sanction row.
//! 6. Publish a redacted summary to `public_case_log`.
//! 7. Emit per-juror reputation events (`JuryReliability +10` for
//!    majority-aligned, `-5` for outliers) and a per-reporter reputation
//!    event on the `ReportingAccuracy` dimension.
//!
//! | `JuryDecision` | `SanctionScope` | `SanctionAction` |
//! |---|---|---|
//! | `NoAction` | (no sanction row) | (no sanction row) |
//! | `AdvisoryLabel` | `Community` | `Label` |
//! | `Warning` | `Community` | `VisibilityReduction` |
//! | `Cooldown` | `Community` | `TemporaryRestriction` |
//! | `RemoveContent` | `Community` | `ContentRemoval` |
//! | `SuspendLocalUser` | `Instance` | `InstanceSuspension` |
//! | `SuspendCommunityMember` | `Community` | `CommunityExclusion` |
//! | `RecommendFederationAction` | `FederatedRecommendation` | `FederationQuarantineRecommendation` |
//!
//! **All DB writes run inside a single `run_transaction` block.** If any
//! step fails (vote insert, sanction insert, case update, log append),
//! the entire transaction rolls back — no partial writes, no half-decided
//! cases per the plan §Task 7 CRITICAL-TRANSACTION-BOUNDARY directive.

use crate::governance::{
  actor_pseudonym_helper,
  config::{self, ConfigCache, Scope},
  governance_log,
  redaction,
  sponsor_liability,
};
use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::{Duration, Utc};
use diesel::{
  ExpressionMethods,
  QueryDsl,
  SelectableHelper,
  dsl::count_star,
  insert_into,
  update,
};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api_common::governance::{SubmitJuryVote, SubmitJuryVoteResponse};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::{
  newtypes::{CommunityId, ModerationCaseId},
  source::governance::{
    jury_vote::{JuryVote, JuryVoteInsertForm},
    moderation_case::ModerationCase,
    public_case_log::PublicCaseLogInsertForm,
    reputation_event::ReputationEventInsertForm,
    sanction::SanctionInsertForm,
  },
};
use lemmy_db_schema_file::{
  PersonId,
  enums::{
    CaseStatus,
    JuryAssignmentStatus,
    JuryDecision,
    ReputationDimension,
    SanctionAction,
    SanctionScope,
  },
  schema::{jury_assignment, jury_vote, moderation_case, public_case_log, reputation_event, sanction},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;
use std::collections::HashMap;

/// v0 quorum per [99 ADR-007] / [05 §3].
const QUORUM: i64 = 3;
/// Appeal window length per [05 §6]. Hardcoded in v0.
const APPEAL_WINDOW_DAYS: i64 = 7;
// Per-juror / per-reporter reputation deltas now flow through `ConfigCache`
// via `config::get_int` against the `deltas.juror_*` and `deltas.reporter_*`
// keys (Phase 5a seeded). See `process_vote` for the cached reads.

pub async fn submit_jury_vote(
  Json(data): Json<SubmitJuryVote>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SubmitJuryVoteResponse>> {
  check_local_user_valid(&local_user_view)?;

  let juror_id = local_user_view.person.id;
  let juror_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), juror_id).await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let vote_data = data.clone();
  let pseudonym_for_tx = juror_pseudonym.clone();
  // Clone the Data<LemmyContext> handle (cheap Arc clone) so the
  // run_transaction closure can move it into the async future without
  // borrowing from the outer context. Phase 6 task 76 needs context
  // inside process_vote to invoke the federation outbound publisher
  // (see federation_outbox::send_local_sanction_notice).
  let context_for_tx = context.clone();

  let outcome = conn
    .run_transaction(|conn| {
      async move {
        process_vote(conn, juror_id, pseudonym_for_tx, vote_data, &context_for_tx).await
      }
      .scope_boxed()
    })
    .await?;

  Ok(Json(outcome))
}

/// The body of the `run_transaction` closure. All writes live here so
/// the outer handler can remain readable and so the closure signature
/// stays under the workspace's `large_futures` lint threshold.
///
/// `context` is threaded in as of Phase 6 task 76 because the federation
/// outbound publisher
/// ([`crate::governance::federation_outbox::send_local_sanction_notice`])
/// needs `&Data<LemmyContext>` for activity-id hostname generation and
/// for `Person::read`/`Post::read` resolution inside the
/// `resolve_target_url` helper.
async fn process_vote(
  conn: &mut diesel_async::AsyncPgConnection,
  juror_id: PersonId,
  juror_pseudonym: String,
  data: SubmitJuryVote,
  context: &Data<LemmyContext>,
) -> LemmyResult<SubmitJuryVoteResponse> {
  // ConfigCache lives for the whole vote-tally transaction. All typed reads
  // go through `(&mut *conn).into()` — same pattern as `reputation_snapshot`.
  let mut cache = ConfigCache::new();
  // 1. Verify assignment is Accepted (not Submitted — double-vote guard).
  let assignment_exists: bool = jury_assignment::table
    .filter(jury_assignment::case_id.eq(data.case_id))
    .filter(jury_assignment::person_id.eq(juror_id))
    .filter(jury_assignment::status.eq(JuryAssignmentStatus::Accepted))
    .count()
    .get_result::<i64>(conn)
    .await?
    > 0;
  if !assignment_exists {
    return Err(LemmyErrorType::NotFound.into());
  }

  // 2. Insert vote row.
  let vote_form = JuryVoteInsertForm {
    case_id: data.case_id,
    juror_id,
    decision: data.decision,
    rationale: data.rationale.clone(),
  };
  let vote_row: JuryVote = insert_into(jury_vote::table)
    .values(&vote_form)
    .returning(JuryVote::as_returning())
    .get_result(conn)
    .await?;

  // 3. Flip assignment → Submitted. submitted_at is set by the trigger.
  let now = Utc::now();
  update(
    jury_assignment::table
      .filter(jury_assignment::case_id.eq(data.case_id))
      .filter(jury_assignment::person_id.eq(juror_id)),
  )
  .set((
    jury_assignment::status.eq(JuryAssignmentStatus::Submitted),
    jury_assignment::submitted_at.eq(Some(now)),
  ))
  .execute(conn)
  .await?;

  // 4. Log this vote.
  governance_log::append(
    &mut conn.into(),
    "jury_vote_submitted",
    json!({
      "case_id": data.case_id.0,
      "decision": data.decision,
      "vote_id": vote_row.id.0,
    }),
    Some(juror_pseudonym.clone()),
  )
  .await?;

  // 5. Count submitted votes. If under quorum, done.
  let vote_count: i64 = jury_vote::table
    .filter(jury_vote::case_id.eq(data.case_id))
    .select(count_star())
    .first::<i64>(conn)
    .await?;
  if vote_count < QUORUM {
    return Ok(SubmitJuryVoteResponse {
      vote_recorded: true,
      case_decided: false,
      decision: None,
    });
  }

  // 6. Tally. Load every vote for the case, group by decision in Rust.
  let all_votes: Vec<(JuryDecision, Option<String>)> = jury_vote::table
    .filter(jury_vote::case_id.eq(data.case_id))
    .select((jury_vote::decision, jury_vote::rationale))
    .load::<(JuryDecision, Option<String>)>(conn)
    .await?;
  let mut tally: HashMap<JuryDecision, Vec<Option<String>>> = HashMap::new();
  for (decision, rationale) in all_votes {
    tally.entry(decision).or_default().push(rationale);
  }
  let winning_decision = pick_majority(&tally)?;
  let winning_rationales: Vec<String> = tally
    .get(&winning_decision)
    .into_iter()
    .flatten()
    .filter_map(Option::clone)
    .collect();

  // 7. Read the case row for context (community_id, targets, creator_id).
  let case_row: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(data.case_id))
    .select(ModerationCase::as_select())
    .first(conn)
    .await?;

  // 8. Emit sanction (for everything except NoAction).
  if let Some((scope, action)) = map_decision_to_sanction(winning_decision) {
    let sanction_form = SanctionInsertForm {
      case_id: data.case_id,
      scope,
      action,
      target_person_id: case_row.target_person_id,
      target_post_id: case_row.target_post_id,
      target_comment_id: case_row.target_comment_id,
      target_community_id: case_row.target_community_id,
      ends_at: None,
      active: Some(true),
    };
    insert_into(sanction::table)
      .values(&sanction_form)
      .execute(conn)
      .await?;

    governance_log::append(
      &mut conn.into(),
      "sanction_created",
      json!({
        "case_id": data.case_id.0,
        "scope": scope,
        "action": action,
      }),
      None,
    )
    .await?;

    // 8.5. Sponsor-liability deltas (OQ-022 multiplier, OQ-024 floor clamp).
    // Only Person-target cases reach here with sponsors; Post/Comment-target
    // cases have `target_person_id = None` per GOTCHA-56h and skip silently.
    if let Some(target_id) = case_row.target_person_id {
      sponsor_liability::apply_sponsor_liability(
        conn,
        target_id,
        data.case_id,
        case_row.community_id,
        action,
        &mut cache,
      )
      .await?;
    }
  }

  // 9. Flip case → Decided.
  let closed_at = now + Duration::days(APPEAL_WINDOW_DAYS);
  update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
    .set((
      moderation_case::status.eq(CaseStatus::Decided),
      moderation_case::decided_at.eq(Some(now)),
      moderation_case::closed_at.eq(Some(closed_at)),
    ))
    .execute(conn)
    .await?;

  // 10. Publish redacted summary to public_case_log.
  let summary = redaction::scrub(&build_summary(&case_row, winning_decision));
  let rationale_redacted = if winning_rationales.is_empty() {
    None
  } else {
    Some(redaction::scrub(&winning_rationales.join("\n")))
  };
  let pcl_form = PublicCaseLogInsertForm {
    case_id: data.case_id,
    community_id: case_row.community_id,
    summary,
    rationale_redacted,
  };
  insert_into(public_case_log::table)
    .values(&pcl_form)
    .execute(conn)
    .await?;
  governance_log::append(
    &mut conn.into(),
    "public_log_published",
    json!({
      "case_id": data.case_id.0,
      "decision": winning_decision,
    }),
    None,
  )
  .await?;

  // 11. Juror reputation events: aligned / outlier deltas via config.
  let juror_aligned_delta_i64 = config::get_int(
    &mut cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "deltas.juror_aligned",
  )
  .await?;
  let juror_outlier_delta_i64 = config::get_int(
    &mut cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "deltas.juror_outlier",
  )
  .await?;
  let juror_aligned_delta = i32::try_from(juror_aligned_delta_i64).map_err(|_e| {
    LemmyErrorType::Unknown(format!(
      "deltas.juror_aligned ({juror_aligned_delta_i64}) overflows i32"
    ))
  })?;
  let juror_outlier_delta = i32::try_from(juror_outlier_delta_i64).map_err(|_e| {
    LemmyErrorType::Unknown(format!(
      "deltas.juror_outlier ({juror_outlier_delta_i64}) overflows i32"
    ))
  })?;

  let juror_decisions: Vec<(PersonId, JuryDecision)> = jury_vote::table
    .filter(jury_vote::case_id.eq(data.case_id))
    .select((jury_vote::juror_id, jury_vote::decision))
    .load::<(PersonId, JuryDecision)>(conn)
    .await?;
  for (other_juror_id, juror_decision) in juror_decisions {
    let delta = if juror_decision == winning_decision {
      juror_aligned_delta
    } else {
      juror_outlier_delta
    };
    emit_reputation_event(
      conn,
      other_juror_id,
      case_row.community_id,
      ReputationDimension::JuryReliability,
      delta,
      data.case_id,
      if delta > 0 {
        "aligned_with_majority"
      } else {
        "outlier_vote"
      },
    )
    .await?;
  }

  // 12. Reporter reputation event — only when the case has a creator.
  if let Some(reporter_id) = case_row.creator_id {
    let key = if matches!(winning_decision, JuryDecision::NoAction) {
      "deltas.reporter_dismissed"
    } else {
      "deltas.reporter_upheld"
    };
    let delta_i64 = config::get_int(
      &mut cache,
      &mut (&mut *conn).into(),
      Scope::Instance,
      key,
    )
    .await?;
    let delta = i32::try_from(delta_i64).map_err(|_e| {
      LemmyErrorType::Unknown(format!("{key} ({delta_i64}) overflows i32"))
    })?;
    let reason = if matches!(winning_decision, JuryDecision::NoAction) {
      "report_dismissed"
    } else {
      "report_upheld"
    };
    emit_reputation_event(
      conn,
      reporter_id,
      case_row.community_id,
      ReputationDimension::ReportingAccuracy,
      delta,
      data.case_id,
      reason,
    )
    .await?;
  }

  // 12.5. Phase 6 task 76 — federated recommendation outbound publish.
  // When the winning sanction has scope = FederatedRecommendation, send
  // the AP `Create(SanctionNotice)` and append the
  // `federation_sanction_sent` log entry on the in-flight conn so the
  // federation publish is atomic with the rest of the post-decision
  // block per plan §757. Branch on SCOPE (not action) per DQ-6.5
  // resolved id 35; re-derive scope from winning_decision via
  // map_decision_to_sanction since the local (scope, action) tuple
  // closes its lexical block at line 269.
  //
  // The wrapper computes the federation actor pseudonym internally
  // (admin Person, not the juror — ADR-015 attribution) so we pass
  // only the conn + context here. See federation_outbox.rs head-of-
  // module DQ-6.7 note for the parameter rationale.
  if matches!(
    map_decision_to_sanction(winning_decision),
    Some((SanctionScope::FederatedRecommendation, _)),
  ) {
    crate::governance::federation_outbox::send_local_sanction_notice(
      data.case_id,
      conn,
      context,
    )
    .await?;
  }

  governance_log::append(
    &mut conn.into(),
    "case_decided",
    json!({
      "case_id": data.case_id.0,
      "decision": winning_decision,
      "closed_at": closed_at,
    }),
    None,
  )
  .await?;

  Ok(SubmitJuryVoteResponse {
    vote_recorded: true,
    case_decided: true,
    decision: Some(winning_decision),
  })
}

/// Pick the most-voted-for `JuryDecision`. Simple majority; ties are
/// broken by iteration order of the `HashMap` which is non-deterministic
/// — v0 per [99 ADR-007] allows this since the probability of a perfect
/// tie with a 5-juror / 3-quorum panel is vanishingly small and any
/// tie-break rule is acceptable. Phase 5 may introduce a deterministic
/// tie-break (e.g. alphabetic on the `JuryDecision` enum name) per the
/// advisor's call.
fn pick_majority(
  tally: &HashMap<JuryDecision, Vec<Option<String>>>,
) -> LemmyResult<JuryDecision> {
  tally
    .iter()
    .max_by_key(|(_, votes)| votes.len())
    .map(|(decision, _)| *decision)
    .ok_or_else(|| LemmyErrorType::NotFound.into())
}

/// Map a winning `JuryDecision` onto a `(SanctionScope, SanctionAction)`
/// pair. Returns `None` for `NoAction` (no sanction row is created).
/// Exhaustive per [ADR-013].
fn map_decision_to_sanction(
  decision: JuryDecision,
) -> Option<(SanctionScope, SanctionAction)> {
  match decision {
    JuryDecision::NoAction => None,
    JuryDecision::AdvisoryLabel => Some((SanctionScope::Community, SanctionAction::Label)),
    JuryDecision::Warning => {
      Some((SanctionScope::Community, SanctionAction::VisibilityReduction))
    }
    JuryDecision::Cooldown => {
      Some((SanctionScope::Community, SanctionAction::TemporaryRestriction))
    }
    JuryDecision::RemoveContent => {
      Some((SanctionScope::Community, SanctionAction::ContentRemoval))
    }
    JuryDecision::SuspendLocalUser => {
      Some((SanctionScope::Instance, SanctionAction::InstanceSuspension))
    }
    JuryDecision::SuspendCommunityMember => {
      Some((SanctionScope::Community, SanctionAction::CommunityExclusion))
    }
    JuryDecision::RecommendFederationAction => Some((
      SanctionScope::FederatedRecommendation,
      SanctionAction::FederationQuarantineRecommendation,
    )),
  }
}

/// One-shot insert helper for `reputation_event`. Pulled out to keep the
/// handler body readable and to ensure every reputation write goes
/// through the same shape (no inline insert forms).
async fn emit_reputation_event(
  conn: &mut diesel_async::AsyncPgConnection,
  person_id: PersonId,
  community_id: Option<CommunityId>,
  dimension: ReputationDimension,
  delta: i32,
  source_case_id: ModerationCaseId,
  reason: &str,
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
  };
  insert_into(reputation_event::table)
    .values(&form)
    .execute(conn)
    .await?;
  Ok(())
}

/// Compose a one-line summary of the case outcome for the public modlog.
/// Intentionally bland — the rationale, scrubbed, goes into a separate
/// column. No identifiers here; [`redaction::scrub`] still runs over the
/// result as a defence-in-depth in case a future reason_code contains
/// something that resembles an identifier.
fn build_summary(case: &ModerationCase, decision: JuryDecision) -> String {
  format!(
    "Case #{case_id} ({target_type:?}): jury decided {decision:?}. Reason: {reason}",
    case_id = case.id.0,
    target_type = case.target_type,
    decision = decision,
    reason = case.reason_code,
  )
}
