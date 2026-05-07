//! `POST /api/v4/governance/endorsement/revoke` — revoke a peer-to-peer
//! endorsement, optionally triggering sponsor-liability escape for matching
//! `SponsorLiabilityPending` cases in their grace window.
//!
//! ## Architectural decisions (locked in SL-b)
//!
//! Per §4.1 + §4.2 watchpoints in `v1-sponsor-liability-b.plan.md`:
//!
//! - Rate-limit count (PRD §12.1) runs PRE-TRANSACTION against
//!   `liability.revoke_rate_limit_per_day` (default 5; SL-a-seeded). Admins
//!   bypass the cap; the bypass is annotated in the log payload via
//!   `rate_limit_bypassed: true` per DQ #140. i64 comparison throughout —
//!   no `as` cast (Watch 14 / R1 / `feedback_clippy_test_style.md`).
//! - Reason is required; empty-after-trim rejected per PRD §12.2 + DQ #139.
//!   The raw reason string flows into `governance_log::append`'s `scrub_json`
//!   layer — the handler does NOT scrub it.
//! - Every write (endorsement UPDATE, surety UPDATE, up to N case UPDATEs,
//!   2 snapshot recomputes, 1 `endorsement_revoked` log entry + 0..N
//!   `sponsor_liability_escaped` entries) runs inside one `run_transaction`
//!   closure per `feedback_multi_write_handlers_need_transactions.md`.
//! - TOCTOU avoidance: `endorsement.revoked_at IS NULL` check is inside the
//!   tx with `FOR UPDATE` on the endorsement row (Watch 1 / §4.2 #1).
//! - Re-revocation is a no-op: returns existing `revoked_at` with empty
//!   `liability_chain_severed_for_cases` vec (PRD §5.4 idempotency).
//! - `actor_pseudonym` discipline (ADR-015): `caller_pseudonym` sourced via
//!   `actor_pseudonym_helper::get_or_create` BEFORE the tx; NEVER raw
//!   `caller_id` in any JSONB payload (Watch 4 / §4.2 #4).
//! - ConfigCache: per-case escape-rule read inside the grace-window loop
//!   deduplicates community reads automatically (DQ #142 — per-case, not
//!   pre-loop). Cases may belong to different communities; the same
//!   `ConfigCache` instance threaded through the closure handles dedup.
//! - Two `recompute_snapshot` calls inside the same tx: sponsor
//!   (`caller_id`) AND sponsee (`endorsement.to_person_id`) per §4.2 #6.
//! - `endorsement_revoked` log entry fires ALWAYS (even when no severance
//!   occurred) per §4.1 step 8 + §10.4.
//! - `match case.status` — zero new arms added; grace-window loop uses
//!   Diesel `.filter(status.eq(...))`, not a Rust `match`, per ADR-013 +
//!   §4.2 #4 (no new match sites in SL-b).
//! - `step_up_token` NOT enforced — reserved for v2 per PRD §12.3.

use actix_web::web::{Data, Json};
use chrono::{DateTime, Duration, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper, dsl::count_star, update};
use diesel_async::{AsyncPgConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api::governance::{
  actor_pseudonym_helper,
  config::{self, ConfigCache, Scope},
  governance_log,
  reputation_snapshot,
};
use lemmy_api_common::governance::{RevokeEndorsement, RevokeEndorsementResponse};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_local_user_valid, is_admin},
};
use lemmy_db_schema::{
  newtypes::ModerationCaseId,
  source::governance::{endorsement::Endorsement, moderation_case::ModerationCase},
};
use lemmy_db_schema_file::{
  PersonId,
  enums::CaseStatus,
  schema::{endorsement, moderation_case, surety},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;

pub async fn revoke_endorsement(
  Json(data): Json<RevokeEndorsement>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<RevokeEndorsementResponse>> {
  check_local_user_valid(&local_user_view)?;

  let caller_id = local_user_view.person.id;
  let caller_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), caller_id).await?;
  // is_admin returns Ok(()) for admins; .is_ok() converts to bool without
  // erroring on non-admin callers (self-revoke is valid for non-admins).
  let is_admin_caller = is_admin(&local_user_view).is_ok();

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  // PRE-TX: idempotency probe (cr-4 PR #119) — already-revoked retries
  // skip rate-limit accounting. Cheap single-row SELECT; the tx-level
  // FOR UPDATE re-check at Step 3 is the authoritative idempotency
  // gate. This probe avoids 429s on retries of a successful revoke.
  let already_revoked: Option<DateTime<Utc>> = endorsement::table
    .filter(endorsement::id.eq(data.endorsement_id))
    .select(endorsement::revoked_at)
    .first::<Option<DateTime<Utc>>>(conn)
    .await
    .optional()?
    .flatten();
  // If already revoked, fall through to the tx (it'll re-load FOR UPDATE
  // and idempotent-return); skip the rate-limit count + bypass logic.
  let (rate_limit_per_day, recent_count, bypass_recorded);
  if already_revoked.is_some() {
    rate_limit_per_day = i64::MAX;
    recent_count = 0i64;
    bypass_recorded = false;
  } else {
    // PRE-TX: rate-limit count (admin bypasses — Watch 14 / R1 i64 discipline).
    rate_limit_per_day = config::get_int(
      &mut ConfigCache::new(),
      &mut context.pool(),
      Scope::Instance,
      "liability.revoke_rate_limit_per_day",
    )
    .await?;
    let cutoff: DateTime<Utc> = Utc::now() - Duration::hours(24);
    recent_count = endorsement::table
      .filter(endorsement::from_person_id.eq(caller_id))
      .filter(endorsement::revoked_at.gt(cutoff))
      .select(count_star())
      .get_result(conn)
      .await?;
    // bypass_recorded: true only when admin AND would-have-been-rate-limited
    // (DQ #140). Standard admin-under-threshold = false; non-admin-under-threshold = false.
    bypass_recorded = is_admin_caller && recent_count >= rate_limit_per_day;
    if !is_admin_caller && recent_count >= rate_limit_per_day {
      return Err(LemmyErrorType::TooManyRequests.into());
    }
  }

  // PRE-TX: reason validation (DQ #139 — mirrors admin_close_case.rs:30-32).
  if data.reason.trim().is_empty() {
    return Err(
      LemmyErrorType::Unknown("revoke-endorsement reason required".to_string()).into(),
    );
  }

  let data_for_tx = data.clone();
  let pseudonym_for_tx = caller_pseudonym.clone();

  // is_admin_caller + bypass_recorded + caller_id are Copy; moved by value.
  // data_for_tx + pseudonym_for_tx are move'd per §4 GOTCHA closure scoping.
  let outcome = conn
    .run_transaction(|conn| {
      async move {
        process_revocation(
          conn,
          caller_id,
          pseudonym_for_tx,
          is_admin_caller,
          bypass_recorded,
          data_for_tx,
        )
        .await
      }
      .scope_boxed()
    })
    .await?;

  Ok(Json(outcome))
}

/// Body of the `run_transaction` closure. Named helper so the outer future
/// stays under the workspace `large_futures` lint threshold (mirrors
/// `create_endorsement::process_endorsement`).
async fn process_revocation(
  conn: &mut AsyncPgConnection,
  caller_id: PersonId,
  caller_pseudonym: String,
  is_admin_caller: bool,
  bypass_recorded: bool,
  data: RevokeEndorsement,
) -> LemmyResult<RevokeEndorsementResponse> {
  let mut config = ConfigCache::new();

  // Step 1: load endorsement FOR UPDATE.
  // (TOCTOU avoidance — Watch 1 / §4.2 #1).
  let row: Endorsement = endorsement::table
    .filter(endorsement::id.eq(data.endorsement_id))
    .for_update()
    .select(Endorsement::as_select())
    .first(conn)
    .await?;

  // Step 2: capability check — self-revoke or admin (cr-5 PR #119:
  // before the idempotent early-return; otherwise unauthorised callers
  // distinguish revoked-vs-not-found via revoked_at leak).
  if !is_admin_caller && row.from_person_id != caller_id {
    return Err(LemmyErrorType::NotFound.into());
  }

  // Step 3: idempotency (PRD §5.4) — re-revocation is a no-op for
  // authorised callers (was Step 1.5 pre-cr-5).
  if let Some(existing_revoked_at) = row.revoked_at {
    return Ok(RevokeEndorsementResponse {
      endorsement_id: row.id,
      revoked_at: existing_revoked_at,
      liability_chain_severed_for_cases: vec![],
    });
  }

  let now: DateTime<Utc> = Utc::now();

  // Step 4: UPDATE endorsement.revoked_at.
  update(endorsement::table.filter(endorsement::id.eq(row.id)))
    .set(endorsement::revoked_at.eq(Some(now)))
    .execute(conn)
    .await?;

  // Step 5: UPDATE matching surety row by triple (sponsor_id, sponsored_id,
  // community_id). UPDATE no-op if not found — community-scoped vs. unscoped
  // endorsement, or cap-exceeded surety (no row inserted at create time when
  // MAX_ACTIVE_SURETIES_PER_SPONSEE was already reached).
  //
  // SQL-NULL semantics: `column = NULL` never matches in Postgres. For
  // instance-scope endorsements (community_id IS NULL), use `is_null()`
  // instead of `eq(None)`. Two branches because `update()` does not take a
  // `.into_boxed()` filter.
  let revoking_sponsor_id = row.from_person_id;
  match row.community_id {
    Some(c) => {
      update(
        surety::table
          .filter(surety::sponsor_id.eq(revoking_sponsor_id))
          .filter(surety::sponsored_id.eq(row.to_person_id))
          .filter(surety::community_id.eq(c))
          .filter(surety::revoked_at.is_null()),
      )
      .set(surety::revoked_at.eq(Some(now)))
      .execute(conn)
      .await?;
    }
    None => {
      update(
        surety::table
          .filter(surety::sponsor_id.eq(revoking_sponsor_id))
          .filter(surety::sponsored_id.eq(row.to_person_id))
          .filter(surety::community_id.is_null())
          .filter(surety::revoked_at.is_null()),
      )
      .set(surety::revoked_at.eq(Some(now)))
      .execute(conn)
      .await?;
    }
  }

  // Steps 6 + 7: grace-window evaluation loop.
  // Query SponsorLiabilityPending cases for the sponsee still within their
  // grace window. The Diesel filter on `status.eq(SponsorLiabilityPending)`
  // ensures SL-b never mutates Decided or Closed cases (ADR-013 / Watch 7;
  // no new match arms introduced).
  let pending_cases: Vec<ModerationCase> = moderation_case::table
    .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityPending))
    .filter(moderation_case::target_person_id.eq(Some(row.to_person_id)))
    .filter(moderation_case::grace_expires_at.gt(Some(now)))
    .select(ModerationCase::as_select())
    .load(conn)
    .await?;

  let mut severed: Vec<ModerationCaseId> = vec![];
  for case in &pending_cases {
    // Per DQ #142: per-case read inside the loop. ConfigCache deduplicates
    // duplicate-community reads automatically (cases may belong to different
    // communities when the backfill is instance-wide — PRD §11.2).
    let scope = match case.community_id {
      Some(cid) => Scope::Community(cid),
      None => Scope::Instance,
    };
    let escape_rule = config::get_text(
      &mut config,
      &mut (&mut *conn).into(),
      scope,
      "liability.multi_sponsor_escape_rule",
    )
    .await?;

    // Step 6: re-query active sponsors for `to_person_id` (post-step-5 state).
    let active_sponsor_count: i64 = surety::table
      .filter(surety::sponsored_id.eq(row.to_person_id))
      .filter(surety::revoked_at.is_null())
      .select(count_star())
      .get_result(conn)
      .await?;

    // Step 7: apply escape rule.
    let escapes = match escape_rule.as_str() {
      "all_revocation" => active_sponsor_count == 0,
      // cr-6 PR #119: the synchronous handler cannot correctly evaluate
      // majority_revocation without a persisted baseline_sponsor_count
      // captured at the Decided -> SponsorLiabilityPending transition.
      // The authoritative evaluator lives in v1-SL-c's grace-check
      // (sponsor_liability_grace.rs::evaluate_escape_conditions). Until
      // that lands, fall through to any_revocation behaviour for cases
      // where the operator selected "majority_revocation". See
      // .claude/runlog/advisor-relays/adhoc-sl-c-baseline-sponsor-count.md.
      // TODO(v1-SL-c): restore majority threshold once
      // moderation_case.baseline_sponsor_count column lands.
      "majority_revocation" => true,
      // "any_revocation" (default) + unknown/NULL fallback (defensive — no error)
      _ => true,
    };

    if escapes {
      let escape_reason = json!({
        "version": 1,
        "reason": "sponsor_revoked",
        "actor_pseudonym": caller_pseudonym,
        "endorsement_id": row.id.0,
      });
      update(moderation_case::table.filter(moderation_case::id.eq(case.id)))
        .set((
          moderation_case::status.eq(CaseStatus::SponsorLiabilityEscaped),
          moderation_case::liability_escape_reason.eq(Some(escape_reason)),
        ))
        .execute(conn)
        .await?;
      governance_log::append(
        &mut (&mut *conn).into(),
        governance_log::ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED,
        json!({
          "case_id": case.id.0,
          "escaped_at": now,
          "reason": "sponsor_revoked",
          "actor_pseudonym": caller_pseudonym,
          "endorsement_id": row.id.0,
        }),
        Some(caller_pseudonym.clone()),
      )
      .await?;
      severed.push(case.id);
    }
  }

  // Step 8: recompute snapshots — sponsor AND sponsee inside same tx
  // (Watch 6 / §4.2 #6 — mirrors create_endorsement.rs:307-309).
  reputation_snapshot::recompute_snapshot(conn, revoking_sponsor_id, row.community_id, &mut config)
    .await?;
  reputation_snapshot::recompute_snapshot(conn, row.to_person_id, row.community_id, &mut config)
    .await?;

  // Step 9: emit endorsement_revoked log entry ALWAYS (even when no severance).
  // Per DQ #140: include rate_limit_bypassed ONLY when (admin AND
  // would-have-been-rate-limited); omit for standard revocations.
  let target_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), row.to_person_id).await?;
  let mut payload = json!({
    "version": 1,
    "endorsement_id": row.id.0,
    "sponsor_pseudonym": caller_pseudonym,
    "target_pseudonym": target_pseudonym,
    "community_id": row.community_id.map(|c| c.0),
    "reason": data.reason,
    "liability_chain_severed_for_cases": severed.iter().map(|c| c.0).collect::<Vec<_>>(),
  });
  if bypass_recorded
    && let Some(obj) = payload.as_object_mut()
  {
    obj.insert("rate_limit_bypassed".to_string(), json!(true));
  }
  governance_log::append(
    &mut (&mut *conn).into(),
    governance_log::ENTRY_KIND_ENDORSEMENT_REVOKED,
    payload,
    Some(caller_pseudonym),
  )
  .await?;

  Ok(RevokeEndorsementResponse {
    endorsement_id: row.id,
    revoked_at: now,
    liability_chain_severed_for_cases: severed,
  })
}
