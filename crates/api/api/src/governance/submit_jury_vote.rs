//! `POST /api/v4/governance/jury/vote` — submit a juror's vote.
//!
//! The most complex handler in v0. Implements the full aggregation rule
//! from [04 §8] and the v0 simplifications from [05 §3] and [05 §6]:
//!
//! 1. Verify the caller has an `Accepted` assignment on the case.
//! 2. SELECT FOR UPDATE on the moderation_case row (lock-ordering: must precede the jury_vote
//!    INSERT so concurrent voters serialise here rather than deadlocking on FK SHARE → EXCLUSIVE
//!    upgrade — PR #98 cr-2 / DQ #50).
//! 3. Insert the vote and flip the assignment to `Submitted`.
//! 4. If the case is already in a terminal state (Decided / Closed / Appealed / EmergencyRemove /
//!    AdminReview), return early — the vote was recorded for audit, but no post-decision side
//!    effects re-fire (idempotency guard for late-arriving votes 4-5 of 5).
//! 5. If fewer than `case.quorum_snapshot` votes are in (per [99 ADR-007]), return without
//!    deciding.
//! 6. At quorum, tally votes by simple majority. Record the decision on the case row (status →
//!    `Decided`, `decided_at = now`, `appeal_window_expires_at = now + appeal.window_days` per PRD
//!    §9.1 step 9 — LIVE config read, the single deliberate exception to the snapshot-everything
//!    rule).
//! 7. Map the winning `JuryDecision` to a `Sanction` per the table below; `NoAction` writes no
//!    sanction row.
//! 8. Publish a redacted summary to `public_case_log`.
//! 9. Emit per-juror reputation events (`JuryReliability +10` for majority-aligned, `-5` for
//!    outliers) and a per-reporter reputation event on the `ReportingAccuracy` dimension.
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
  governance_log::{self, ENTRY_KIND_APPEAL_DECIDED, ENTRY_KIND_JURY_DEADLOCK},
  redaction,
  sponsor_liability,
};
use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::{DateTime, Duration, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper, dsl::count_star, insert_into, update};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api_common::governance::{SubmitJuryVote, SubmitJuryVoteResponse};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::{
  newtypes::{CommunityId, ModerationCaseId},
  source::governance::{
    appeal::Appeal,
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
    AppealStatus,
    CaseStatus,
    JuryAssignmentRole,
    JuryAssignmentStatus,
    JuryDecision,
    ReputationDimension,
    SanctionAction,
    SanctionScope,
  },
  schema::{
    appeal,
    jury_assignment,
    jury_vote,
    moderation_case,
    public_case_log,
    reputation_event,
    sanction,
  },
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::{Value, json};
use std::collections::HashMap;

// Per-juror / per-reporter reputation deltas now flow through `ConfigCache`
// via `config::get_int` against the `deltas.juror_*` and `deltas.reporter_*`
// keys (Phase 5a seeded). See `process_vote` for the cached reads.

// Stable enum-order iteration list for tally loops in `process_vote` (original
// jury) + `process_appeal_vote` (appeal jury). Single source of truth — adding
// a new `JuryDecision` variant requires extending this list. Compile-time
// exhaustiveness is enforced by `map_decision_to_sanction` below; this const
// is the runtime tally order. Mirrors stable enum-declaration order in
// `crates/db_schema_file/src/enums.rs` and the [04 §8] aggregation rule.
const ALL_JURY_DECISIONS: [JuryDecision; 8] = [
  JuryDecision::NoAction,
  JuryDecision::AdvisoryLabel,
  JuryDecision::Warning,
  JuryDecision::Cooldown,
  JuryDecision::RemoveContent,
  JuryDecision::SuspendLocalUser,
  JuryDecision::SuspendCommunityMember,
  JuryDecision::RecommendFederationAction,
];

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
  // 1. Verify assignment is Accepted (not Submitted — double-vote guard); capture role for
  //    step-2.5 branch dispatch (v1-JM-e: Appeal-role jurors go to process_appeal_vote).
  let role: JuryAssignmentRole = jury_assignment::table
    .filter(jury_assignment::case_id.eq(data.case_id))
    .filter(jury_assignment::person_id.eq(juror_id))
    .filter(jury_assignment::status.eq(JuryAssignmentStatus::Accepted))
    .select(jury_assignment::role)
    .first::<JuryAssignmentRole>(conn)
    .await
    .map_err(|_e| LemmyErrorType::NotFound)?;

  // 2. SELECT FOR UPDATE on moderation_case BEFORE the jury_vote INSERT.
  // Lock-ordering: the jury_vote INSERT at step 3 below takes an FK SHARE
  // lock on this same moderation_case row (Postgres acquires it implicitly
  // for FK validation). Acquiring FOR UPDATE first means concurrent voters
  // serialise on the row-exclusive lock here, then run the INSERT under an
  // already-held EXCLUSIVE; the FK SHARE within the same transaction is
  // compatible with our own EXCLUSIVE so no upgrade ever fires. The
  // previous order (INSERT first → FOR UPDATE second) caused a
  // deterministic deadlock when two concurrent transactions both held
  // FK SHARE and both then waited to upgrade to EXCLUSIVE. Regression
  // test: `submit_jury_vote_concurrent_votes_decide_exactly_once` in
  // `crates/server/tests/e2e.rs`. PR #98 cr-2.
  //
  // Loading the full row here also gives every later step direct access to
  // `case_row.quorum_snapshot`, `panel_size_snapshot`, `target_*`,
  // `community_id`, `creator_id` — eliminating a second locking read.
  let case_row: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(data.case_id))
    .select(ModerationCase::as_select())
    .for_update()
    .first(conn)
    .await?;

  let now = Utc::now();

  // Step 2.5 (v1-JM-e): branch on jury_assignment.role BEFORE the idempotency guard.
  // Appeal-role jurors follow the appeal-panel tally path; Original-role jurors continue below.
  // GOTCHA: the idempotency guard below lists `Appealed` as terminal, but appeal-panel votes
  // MUST fire on Appealed cases — so the dispatch precedes the guard.
  if role == JuryAssignmentRole::Appeal {
    return process_appeal_vote(conn, juror_id, juror_pseudonym, data, &case_row, now).await;
  }

  // 3. Insert vote row.
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

  // 4. Flip assignment → Submitted. submitted_at is set by the trigger.
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

  // 5. Log this vote.
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

  // 6. Idempotency guard. Votes 4 and 5 in a 5-juror panel arrive after
  // vote 3 has flipped the case to Decided. We still INSERT their vote
  // rows above (audit integrity — every vote is recorded), but the
  // post-decision block (sanction insert, sponsor liability,
  // public_case_log, reputation events, federation publish,
  // governance_log case_decided / sanction_created / public_log_published)
  // must not re-fire. Regression tests: report_to_modlog_golden_path
  // votes all 5 jurors and asserts exactly-once on every post-decision
  // write; sanction_notice_round_trip does the same for federation
  // publish; submit_jury_vote_concurrent_votes_decide_exactly_once
  // covers the truly-concurrent variant. CodeRabbit PR #46 finding #15.
  //
  // Tracks terminal states explicitly (rather than `!= Open`) so live-flow
  // states like ThresholdMet / JurySelection / InReview don't trip the
  // guard, and so v1 additions land in the correct default-behaviour
  // category unless explicitly added to the terminal list.
  if matches!(
    case_row.status,
    CaseStatus::Decided
      | CaseStatus::Closed
      | CaseStatus::Appealed
      | CaseStatus::EmergencyRemove
      | CaseStatus::AdminReview
  ) {
    return Ok(SubmitJuryVoteResponse {
      vote_recorded: true,
      case_decided: true,
      decision: None,
    });
  }

  // 7. Count submitted votes. If under quorum, done.
  //
  // Quorum is read from `case_row.quorum_snapshot` (v1-JM-b writes this at
  // admin_assign_jury time per PRD §9.1 step 4 / ADR-010). A NULL snapshot
  // can only arise if the case bypassed admin_assign_jury, which is
  // impossible for any case in JurySelection or later post-JM-a-backfill
  // — `Unknown` here surfaces a process breach loudly.
  let quorum_snapshot: i32 = case_row.quorum_snapshot.ok_or_else(|| {
    LemmyErrorType::Unknown(format!(
      "case {} has NULL quorum_snapshot; admin_assign_jury did not run",
      data.case_id.0
    ))
  })?;
  let vote_count: i64 = jury_vote::table
    .filter(jury_vote::case_id.eq(data.case_id))
    .select(count_star())
    .first::<i64>(conn)
    .await?;
  if vote_count < i64::from(quorum_snapshot) {
    return Ok(SubmitJuryVoteResponse {
      vote_recorded: true,
      case_decided: false,
      decision: None,
    });
  }

  // 7. Per-decision threshold tally. Read the JM-b-written snapshot fields
  // (panel_size_snapshot, threshold_count_snapshot) — both populated
  // alongside `quorum_snapshot` at admin_assign_jury time, so a NULL here
  // is the same kind of process breach as the NULL quorum_snapshot path
  // above. Iterate JuryDecision variants in stable enum-order; the first
  // decision meeting `threshold_count_snapshot` wins. PRD §9.1 step 5.
  let panel_size_snapshot: i32 = case_row.panel_size_snapshot.ok_or_else(|| {
    LemmyErrorType::Unknown(format!(
      "case {} has NULL panel_size_snapshot; admin_assign_jury did not run",
      data.case_id.0
    ))
  })?;
  let threshold_count_snapshot: i32 = case_row.threshold_count_snapshot.ok_or_else(|| {
    LemmyErrorType::Unknown(format!(
      "case {} has NULL threshold_count_snapshot; admin_assign_jury did not run",
      data.case_id.0
    ))
  })?;
  let threshold_count_i64 = i64::from(threshold_count_snapshot);

  // Single query: load every vote for the case (decision + rationale).
  // Rationales feed the public_case_log redaction below; the per-decision
  // count drives the threshold pick. Loading both in one query keeps the
  // round-trip count identical to v0.
  let all_votes: Vec<(JuryDecision, Option<String>)> = jury_vote::table
    .filter(jury_vote::case_id.eq(data.case_id))
    .select((jury_vote::decision, jury_vote::rationale))
    .load::<(JuryDecision, Option<String>)>(conn)
    .await?;
  let mut tally: HashMap<JuryDecision, Vec<Option<String>>> = HashMap::new();
  for (decision, rationale) in all_votes {
    tally.entry(decision).or_default().push(rationale);
  }

  // Stable enum-order tally per [04 §8] — see ALL_JURY_DECISIONS const at
  // module top for the single source of truth. `map_decision_to_sanction`
  // below is the compile-time exhaustiveness check.
  let mut winning_decision: Option<JuryDecision> = None;
  for candidate in ALL_JURY_DECISIONS {
    let count = i64::try_from(tally.get(&candidate).map_or(0, Vec::len)).map_err(|_e| {
      LemmyErrorType::Unknown(format!(
        "vote count for {candidate:?} on case {} overflows i64",
        data.case_id.0
      ))
    })?;
    if count >= threshold_count_i64 {
      winning_decision = Some(candidate);
      break;
    }
  }

  // 7.5. Deadlock branch + partial-tally early return.
  // SOURCE MIRROR: admin_assign_jury.rs:211-224 (governance_log::append +
  // json! payload + actor_pseudonym shape). The casting juror's pseudonym
  // is the actor: they triggered the deadlock detection by being the
  // panel_size-th voter without a winner emerging. ADR-015 attribution.
  let winning_decision = match winning_decision {
    Some(decision) => decision,
    None => {
      if vote_count == i64::from(panel_size_snapshot) {
        // DEADLOCK: all jurors voted, no decision met threshold.
        // Status flips to AdminReview ONLY — no decided_at, no closed_at,
        // no appeal_window_expires_at. A deadlocked case is not "decided";
        // it's "stuck pending admin." Lifecycle terminates here. Per plan
        // §10.4 GOTCHA.
        update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
          .set(moderation_case::status.eq(CaseStatus::AdminReview))
          .execute(conn)
          .await?;

        let tally_payload: serde_json::Map<String, Value> = tally
          .iter()
          .map(|(decision, votes)| (format!("{decision:?}"), Value::from(votes.len())))
          .collect();

        governance_log::append(
          &mut conn.into(),
          ENTRY_KIND_JURY_DEADLOCK,
          json!({
            "case_id": data.case_id.0,
            "panel_size_snapshot": panel_size_snapshot,
            "threshold_count_snapshot": threshold_count_snapshot,
            "tally": tally_payload,
          }),
          Some(juror_pseudonym.clone()),
        )
        .await?;
      }
      // Both deadlock and partial-tally paths return the same response
      // shape (vote_recorded: true, case_decided: false). Deadlock differs
      // by the side effects (status UPDATE + governance_log entry) above.
      return Ok(SubmitJuryVoteResponse {
        vote_recorded: true,
        case_decided: false,
        decision: None,
      });
    }
  };
  let winning_rationales: Vec<String> = tally
    .get(&winning_decision)
    .into_iter()
    .flatten()
    .filter_map(Option::clone)
    .collect();

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
    //
    // SOURCE: NEW in JM-c — load-bearing TODO at the SL-d graft point
    //
    // TODO(v1-sponsor-liability-d): replace this v0 apply_sponsor_liability call with the
    // compute/fire split per .claude/PRPs/prds/v1-sponsor-liability.prd.md §9.1 + §9.3:
    //   - compute_sponsor_liability(...) returns deltas (no event rows yet)
    //   - flip case.status = CaseStatus::SponsorLiabilityPending
    //   - set case.grace_expires_at = now + grace_window_for_severity(severity)
    //   - emit governance_log entry sponsor_liability_pending
    //   - notify_sponsor_of_pending_liability(...) for each delta
    //   - DEFER public_case_log + juror reputation_events to scheduler fire/escape time
    //
    // The current v0 apply_sponsor_liability stays in place for JM-c — SL-d is the rewrite.
    // JM-c's appeal_window_expires_at write at step 9 fires on BOTH this v0 path AND the
    // (future) sponsor-liability path; SL-d must preserve that semantic.
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

  // 8.9. Flip case → Decided. Step 9 (appeal_window_expires_at write) lands
  // AFTER the step-10/11/12 writes below — keeping it as a separate UPDATE
  // makes SL-d's graft cleaner per PRD §9.1 cross-references (SL-d will set
  // `case.status = SponsorLiabilityPending` here instead of Decided, and the
  // appeal_window write must fire on BOTH paths). `closed_at` is no longer
  // written by submit_jury_vote — it becomes a JM-d concern (the
  // appeal-window-expiry background job will set `closed_at = now()` when
  // transitioning Decided → Closed).
  update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
    .set((
      moderation_case::status.eq(CaseStatus::Decided),
      moderation_case::decided_at.eq(Some(now)),
      moderation_case::winning_decision.eq(Some(winning_decision)),
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
    let delta_i64 =
      config::get_int(&mut cache, &mut (&mut *conn).into(), Scope::Instance, key).await?;
    let delta = i32::try_from(delta_i64)
      .map_err(|_e| LemmyErrorType::Unknown(format!("{key} ({delta_i64}) overflows i32")))?;
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
  // 9. Appeal-window expiry write — LIVE config read per PRD §9.1 step 9
  // (the deliberate snapshot-rule exception: changing `appeal.window_days`
  // mid-flight affects future decisions, not in-flight cases — but the
  // *appeal window itself* is a procedural input read at the *decision
  // moment*, not the *jury-seating moment*). JM-c is the first writer of
  // this column on post-JM-a cases. JM-d will be the first reader via the
  // bounded-window appeal check.
  let window_days_i64 = config::get_int(
    &mut cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "appeal.window_days",
  )
  .await?;
  let appeal_window_expires_at = now + Duration::days(window_days_i64);
  update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
    .set(moderation_case::appeal_window_expires_at.eq(Some(appeal_window_expires_at)))
    .execute(conn)
    .await?;

  // Log the case decision FIRST so the hash chain records the local
  // determination before any federation broadcast that results from it
  // (ADR-008 causality). Previously `federation_sanction_sent` appeared
  // ahead of its triggering `case_decided` entry because the send block
  // ran before this append. Both writes share the outer `run_transaction`,
  // so moving one above the other doesn't change atomicity.
  governance_log::append(
    &mut conn.into(),
    "case_decided",
    json!({
      "case_id": data.case_id.0,
      "decision": winning_decision,
      "appeal_window_expires_at": appeal_window_expires_at,
    }),
    None,
  )
  .await?;

  if matches!(
    map_decision_to_sanction(winning_decision),
    Some((SanctionScope::FederatedRecommendation, _)),
  ) {
    crate::governance::federation_outbox::send_local_sanction_notice(data.case_id, conn, context)
      .await?;
  }

  Ok(SubmitJuryVoteResponse {
    vote_recorded: true,
    case_decided: true,
    decision: Some(winning_decision),
  })
}

/// Appeal-panel vote tally (v1-JM-e). Dispatched from `process_vote` when
/// `jury_assignment.role == Appeal`. Runs inside the same `run_transaction`
/// block as the outer handler — do NOT open a new transaction here.
///
/// Steps 3-5 mirror `process_vote` verbatim. Step 6 uses a narrower
/// idempotency guard (`Appealed` is the EXPECTED case status here, not a
/// terminal). Steps 7-8 load the `Appeal` row snapshots and run the appeal-
/// panel threshold tally.
async fn process_appeal_vote(
  conn: &mut diesel_async::AsyncPgConnection,
  juror_id: PersonId,
  juror_pseudonym: String,
  data: SubmitJuryVote,
  case_row: &ModerationCase,
  now: DateTime<Utc>,
) -> LemmyResult<SubmitJuryVoteResponse> {
  // Step 3: Insert vote row.
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

  // Step 4: Flip assignment → Submitted.
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

  // Step 5: Log this vote. The v0 literal is preserved verbatim per plan §12 out-of-scope
  // (JM-c v0-literal cleanup is coordinated for a later sub-phase).
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

  // Step 6: Narrower idempotency guard — `Appealed` is the EXPECTED status during appeal-panel
  // voting. Only truly terminal states short-circuit here.
  if matches!(
    case_row.status,
    CaseStatus::Closed | CaseStatus::EmergencyRemove | CaseStatus::AdminReview
  ) {
    return Ok(SubmitJuryVoteResponse {
      vote_recorded: true,
      case_decided: true,
      decision: None,
    });
  }

  // Step 7-appeal: Load Appeal row; read panel snapshots. These were written by
  // `seat_appeal_panel` (JM-d). NULL on either snapshot is a process breach.
  let appeal_row: Appeal = appeal::table
    .filter(appeal::case_id.eq(data.case_id))
    .select(Appeal::as_select())
    .first(conn)
    .await
    .map_err(|_e| LemmyErrorType::NotFound)?;

  let appeal_panel_size: i32 = appeal_row.panel_size_snapshot.ok_or_else(|| {
    LemmyErrorType::Unknown(format!(
      "appeal {} has NULL panel_size_snapshot; seat_appeal_panel did not run",
      appeal_row.id.0
    ))
  })?;
  let appeal_threshold_count: i32 = appeal_row.threshold_count_snapshot.ok_or_else(|| {
    LemmyErrorType::Unknown(format!(
      "appeal {} has NULL threshold_count_snapshot; seat_appeal_panel did not run",
      appeal_row.id.0
    ))
  })?;
  let appeal_threshold_count_i64 = i64::from(appeal_threshold_count); // R1: i64::from, never `as`

  // Step 7.5-appeal: Per-decision tally on Appeal-role jurors only (JOIN filters out
  // original-jury votes which would otherwise pollute the appeal tally).
  let all_appeal_votes: Vec<(JuryDecision, Option<String>)> = jury_vote::table
    .inner_join(
      jury_assignment::table.on(
        jury_assignment::case_id
          .eq(jury_vote::case_id)
          .and(jury_assignment::person_id.eq(jury_vote::juror_id)),
      ),
    )
    .filter(jury_vote::case_id.eq(data.case_id))
    .filter(jury_assignment::role.eq(JuryAssignmentRole::Appeal))
    .select((jury_vote::decision, jury_vote::rationale))
    .load::<(JuryDecision, Option<String>)>(conn)
    .await?;
  let mut tally: HashMap<JuryDecision, Vec<Option<String>>> = HashMap::new();
  for (decision, rationale) in all_appeal_votes {
    tally.entry(decision).or_default().push(rationale);
  }
  let appeal_vote_count: i64 =
    i64::try_from(tally.values().map(Vec::len).sum::<usize>())
      .map_err(|_e| LemmyErrorType::Unknown("appeal vote count overflows i64".to_string()))?;

  // Stable enum-order tally — same source-of-truth as process_vote.
  let mut appeal_winning_decision: Option<JuryDecision> = None;
  for candidate in ALL_JURY_DECISIONS {
    let count = i64::try_from(tally.get(&candidate).map_or(0, Vec::len)).map_err(|_e| {
      LemmyErrorType::Unknown(format!(
        "appeal vote count for {candidate:?} on case {} overflows i64",
        data.case_id.0
      ))
    })?;
    if count >= appeal_threshold_count_i64 {
      appeal_winning_decision = Some(candidate);
      break;
    }
  }

  // Step 8-appeal: Deadlock-or-decide.
  let appeal_winning_decision = match appeal_winning_decision {
    Some(decision) => decision,
    None => {
      if appeal_vote_count == i64::from(appeal_panel_size) {
        // APPEAL DEADLOCK: all appeal jurors voted, no decision met threshold.
        // Flip to AdminReview — do NOT set appeal.decided_at. Mirrors process_vote deadlock.
        update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
          .set(moderation_case::status.eq(CaseStatus::AdminReview))
          .execute(conn)
          .await?;

        let tally_payload: serde_json::Map<String, Value> = tally
          .iter()
          .map(|(decision, votes)| (format!("{decision:?}"), Value::from(votes.len())))
          .collect();

        governance_log::append(
          &mut conn.into(),
          ENTRY_KIND_JURY_DEADLOCK,
          json!({
            "case_id": data.case_id.0,
            "appeal_id": appeal_row.id.0,
            "panel_kind": "appeal",
            "panel_size_snapshot": appeal_panel_size,
            "threshold_count_snapshot": appeal_threshold_count,
            "tally": tally_payload,
          }),
          Some(juror_pseudonym.clone()),
        )
        .await?;
      }
      return Ok(SubmitJuryVoteResponse {
        vote_recorded: true,
        case_decided: false,
        decision: None,
      });
    }
  };

  // Appeal verdict reached threshold.
  // Set appeal.decided_at + appeal.status = Decided.
  update(appeal::table.filter(appeal::id.eq(appeal_row.id)))
    .set((
      appeal::decided_at.eq(Some(now)),
      appeal::status.eq(AppealStatus::Decided),
    ))
    .execute(conn)
    .await?;

  // Flip case to Closed (appeal verdict is terminal — Decided→Closed in one step per PRD §6.7).
  // Write closed_at = now. The appeal_window_expiry job's filter (status=Decided) will NOT
  // match after this UPDATE, so the two writers never collide.
  update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
    .set((
      moderation_case::status.eq(CaseStatus::Closed),
      moderation_case::closed_at.eq(Some(now)),
    ))
    .execute(conn)
    .await?;

  let original_winning_decision: Option<JuryDecision> = case_row.winning_decision;

  // ENTRY_KIND_APPEAL_DECIDED fires here; ENTRY_KIND_CASE_DECIDED does NOT re-fire
  // (it already fired at original-jury decision time). No sanction row, no federation
  // outbound. Per plan §4.1 v1 simplifications + ADR-014.
  governance_log::append(
    &mut conn.into(),
    ENTRY_KIND_APPEAL_DECIDED,
    json!({
      "case_id": data.case_id.0,
      "appeal_id": appeal_row.id.0,
      "original_winning_decision": original_winning_decision,
      "appeal_winning_decision": appeal_winning_decision,
      "appeal_panel_size": appeal_panel_size,
      "appeal_threshold_count": appeal_threshold_count,
    }),
    Some(juror_pseudonym.clone()),
  )
  .await?;

  Ok(SubmitJuryVoteResponse {
    vote_recorded: true,
    case_decided: true,
    decision: Some(appeal_winning_decision),
  })
}

/// Map a winning `JuryDecision` onto a `(SanctionScope, SanctionAction)`
/// pair. Returns `None` for `NoAction` (no sanction row is created).
/// Exhaustive per [ADR-013].
fn map_decision_to_sanction(decision: JuryDecision) -> Option<(SanctionScope, SanctionAction)> {
  match decision {
    JuryDecision::NoAction => None,
    JuryDecision::AdvisoryLabel => Some((SanctionScope::Community, SanctionAction::Label)),
    JuryDecision::Warning => Some((
      SanctionScope::Community,
      SanctionAction::VisibilityReduction,
    )),
    JuryDecision::Cooldown => Some((
      SanctionScope::Community,
      SanctionAction::TemporaryRestriction,
    )),
    JuryDecision::RemoveContent => Some((SanctionScope::Community, SanctionAction::ContentRemoval)),
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
