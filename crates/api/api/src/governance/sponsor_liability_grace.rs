//! Sponsor-liability grace-window scheduler module.
//!
//! Per PRD §6 + §9.4. Runs every `job.grace_check_interval_minutes`
//! (default 5): finds `SponsorLiabilityPending` cases past their
//! `grace_expires_at` and transitions each to `SponsorLiabilityFired`
//! (apply_sponsor_liability fires) or `SponsorLiabilityEscaped`
//! (sponsor revoked since decided_at). Per-case `run_transaction`
//! isolation per PRD §6.2 + Watch 9 / `feedback_multi_write_handlers_need_transactions.md`.
//!
//! ## Watch 10 — PII discipline (ADR-015)
//!
//! Every governance-log payload field naming a person uses
//! `*_pseudonym` (string sourced via
//! `actor_pseudonym_helper::get_or_create`), NEVER raw `PersonId`.
//! `liability_escape_reason` JSONB written to `moderation_case`
//! follows the same rule.
//!
//! ## Compute/fire posture (advisor-locked, v1-SL-c masthead)
//!
//! This module calls **unsplit v0** `apply_sponsor_liability` from
//! the fire branch (per
//! `crates/api/api/src/governance/sponsor_liability.rs:142`). The
//! PRD §9.1 compute/fire split is SL-d's deliverable. SL-c's call
//! site is forward-compatible: when SL-d ships and the helper
//! either remains as a thin wrapper or is replaced by
//! `compute_sponsor_liability` + `fire_sponsor_liability`, the
//! call site updates as a no-op-behaviour change.
//!
//! ## Restoration-escape branch — STUB-ONLY in v1-SL-c (DQ #145)
//!
//! `EscapeStatus::Escape{reason: "restoration_completed", ...}` is
//! defined as a documented future-wire branch but
//! `evaluate_escape_conditions` NEVER constructs it. The
//! `restoration_completed` log entry has zero producers in v1-SL-c
//! (verified by registry rule). When restorative-mechanics-v1 PRD
//! ships the producer endpoint, that PRD's plan adds the read-side
//! query to `evaluate_escape_conditions` AND a corresponding e2e
//! test.

use chrono::{DateTime, Utc};
use diesel::{
  ExpressionMethods, NullableExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper,
  dsl::min,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::{CommunityId, ModerationCaseId},
  source::governance::moderation_case::ModerationCase,
};
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseStatus, SanctionAction, SanctionScope},
  schema::{endorsement, moderation_case, sanction, surety},
};
use lemmy_diesel_utils::connection::{DbConn, get_conn};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;
use tracing::{info, warn};

use crate::governance::{
  actor_pseudonym_helper,
  config::{self, ConfigCache, Scope},
  governance_log::{
    self, ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED, ENTRY_KIND_SPONSOR_LIABILITY_FIRED,
  },
  sponsor_liability,
  state::{GovernanceCase, SponsorLiabilityPending as SponsorLiabilityPendingState},
};

/// Outcome of a single `run_grace_check_batch` invocation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GraceCheckBatchOutcome {
  pub cases_processed: usize,
  pub fired: usize,
  pub escaped: usize,
  pub skipped: usize,
}

/// Outcome of one case's escape-condition evaluation.
///
/// `Fire` (default) means apply_sponsor_liability runs; `Escape`
/// means the case escapes liability (no sponsor reputation_event
/// rows; `liability_escape_reason` JSONB recorded). The
/// `reason` string discriminates branches: `"sponsor_revoked"`
/// (SL-c scheduler), or `"restoration_completed"` (future
/// restorative-mechanics-v1 PRD; stub-only in v1-SL-c).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscapeStatus {
  /// Escape branch fired. `reason` is one of the documented values.
  Escape {
    reason: String,
    actor_pseudonym: String,
    /// `endorsement.id.0` for `sponsor_revoked`;
    /// future `restoration.id.0` for `restoration_completed`.
    ref_id: i64,
  },
  /// Default — fire branch (apply_sponsor_liability).
  Fire,
}

/// Private outcome enum used by `fire_or_escape_case_inner` to
/// communicate per-case result back to `run_grace_check_batch`'s
/// counter increments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PerCaseOutcome {
  Fired,
  Escaped,
  Skipped,
}

/// Scheduler entry point. Iterates `SponsorLiabilityPending` cases past
/// their `grace_expires_at` and transitions each to Fired or Escaped.
///
/// Does NOT open a transaction — each case opens its own per-case tx
/// (per PRD §6.2 + watchpoint #8). Per-case errors are caught and
/// logged; the outer function returns `Ok(...)` regardless.
pub async fn run_grace_check_batch(context: &LemmyContext) -> LemmyResult<GraceCheckBatchOutcome> {
  let pool = &mut context.pool();
  let mut batch_cache = ConfigCache::new();

  let batch_size = config::get_int(
    &mut batch_cache,
    pool,
    Scope::Instance,
    "job.grace_check_batch_size",
  )
  .await?;
  let batch_size_i64: i64 = batch_size;

  let conn = &mut get_conn(pool).await?;
  let now: DateTime<Utc> = Utc::now();

  // Outer batch query — NO transaction here. Snapshot of cases past their
  // grace_expires_at; per-case tx revalidates with FOR UPDATE.
  // TODO(type-state): currently uses filter-query pattern (not exhaustive match); harden
  // to exhaustive match first, then wrap per-case as GovernanceCase<SponsorLiabilityPending>
  // inside the for-loop — see .claude/lessons/feedback_governance_type_state_handlers.md
  let candidates: Vec<ModerationCase> = moderation_case::table
    .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityPending))
    .filter(moderation_case::grace_expires_at.le(Some(now)))
    .order_by(moderation_case::grace_expires_at.asc())
    .limit(batch_size_i64)
    .select(ModerationCase::as_select())
    .load(conn)
    .await?;

  let mut outcome = GraceCheckBatchOutcome {
    cases_processed: 0,
    fired: 0,
    escaped: 0,
    skipped: 0,
  };

  if candidates.is_empty() {
    info!("governance: grace-check tick — no pending cases past grace_expires_at");
    return Ok(outcome);
  }

  for case in candidates {
    let case_id = case.id;
    // Per-case run_transaction — outer batch never returns Err on per-case
    // failures (per PRD §6.3 + watchpoint #8).
    let case_outcome = conn
      .run_transaction(async |conn| {
        fire_or_escape_case_inner(conn, &case, now).await
      })
      .await;
    match case_outcome {
      Ok(PerCaseOutcome::Fired) => outcome.fired += 1,
      Ok(PerCaseOutcome::Escaped) => outcome.escaped += 1,
      Ok(PerCaseOutcome::Skipped) => outcome.skipped += 1,
      Err(e) => {
        warn!(
          "grace-check: case_id={case_id_int} per-case tx failed: {e}",
          case_id_int = case_id.0
        );
        outcome.skipped += 1;
      }
    }
    outcome.cases_processed += 1;
  }

  info!(
    "governance: grace-check tick — cases_processed={}, fired={}, escaped={}, skipped={}",
    outcome.cases_processed, outcome.fired, outcome.escaped, outcome.skipped
  );
  Ok(outcome)
}

/// Evaluates escape conditions for a pending case.
///
/// Returns `EscapeStatus::Escape{reason: "sponsor_revoked", ...}` if any
/// active surety for the target was revoked on or after `decided_at`.
/// Returns `EscapeStatus::Fire` otherwise.
///
/// Restoration-escape branch is STUB-ONLY in v1-SL-c (DQ #145).
pub async fn evaluate_escape_conditions(
  conn: &mut AsyncPgConnection,
  case_id: ModerationCaseId,
  target_person_id: PersonId,
  _community_id: Option<CommunityId>,
  decided_at: DateTime<Utc>,
  _cache: &mut ConfigCache,
) -> LemmyResult<EscapeStatus> {
  // Branch 1: any active surety for target_person_id with
  // revoked_at >= decided_at? → Escape{reason: "sponsor_revoked"}.
  //
  // We join surety to endorsement to derive the actor pseudonym
  // (the revoking sponsor's PersonId, then via actor_pseudonym_helper).
  // surety.revoked_at doesn't carry the actor; use surety.sponsor_id
  // as the actor source.
  let revoked_surety: Option<(PersonId, DateTime<Utc>)> = surety::table
    .filter(surety::sponsored_id.eq(target_person_id))
    .filter(surety::revoked_at.is_not_null())
    .filter(surety::revoked_at.ge(Some(decided_at)))
    .order_by(surety::revoked_at.asc())
    .limit(1)
    .select((surety::sponsor_id, surety::revoked_at.assume_not_null()))
    .first::<(PersonId, DateTime<Utc>)>(conn)
    .await
    .optional()?;

  if let Some((revoker_id, _revoked_at)) = revoked_surety {
    let actor_pseudonym =
      actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), revoker_id).await?;
    // ref_id: best-effort endorsement.id lookup. If multiple
    // endorsements exist (sponsor → sponsee at multiple
    // community scopes), pick the first by id ascending. If none
    // (defensive — a surety implies an endorsement existed),
    // ref_id stays 0; downstream consumers handle gracefully.
    let endorsement_id: Option<i32> = endorsement::table
      .filter(endorsement::from_person_id.eq(revoker_id))
      .filter(endorsement::to_person_id.eq(target_person_id))
      .order_by(endorsement::id.asc())
      .limit(1)
      .select(endorsement::id)
      .first::<i32>(conn)
      .await
      .optional()?;
    let ref_id = endorsement_id.map(i64::from).unwrap_or(0);
    return Ok(EscapeStatus::Escape {
      reason: "sponsor_revoked".to_string(),
      actor_pseudonym,
      ref_id,
    });
  }

  // Branch 2 (STUB-ONLY in v1-SL-c per DQ #145):
  // Restoration completed during the grace window?
  // The producer endpoint `restoration/complete` does not exist
  // in v1; ENTRY_KIND_RESTORATION_COMPLETED has zero emit-sites.
  // When restorative-mechanics-v1 ships the producer, this branch
  // queries governance_log for a `restoration_completed` entry
  // for target_person_id between decided_at and now, returning
  // EscapeStatus::Escape { reason: "restoration_completed", ... }.
  //
  // Until then, fall through to Fire.
  let _ = case_id; // suppress unused-var on stub branch

  Ok(EscapeStatus::Fire)
}

/// Public entry for caller-driven per-case fire/escape. The batch loop
/// uses `fire_or_escape_case_inner` directly inside its per-case
/// `run_transaction` closure. This public entry is available for
/// future callers (admin-driven manual runs, etc.).
///
/// For the Fire branch, this entry returns an error — callers must
/// use `run_grace_check_batch` which has access to the sanction-action
/// context needed by the Fire branch.
pub async fn fire_or_escape_case(
  conn: &mut DbConn<'_>,
  case: ModerationCase,
  status: EscapeStatus,
  _cache: &mut ConfigCache,
) -> LemmyResult<()> {
  let now: DateTime<Utc> = Utc::now();
  conn
    .run_transaction(async |conn| {
      match status {
        EscapeStatus::Escape {
          reason,
          actor_pseudonym,
          ref_id,
        } => {
          let escape_reason_json = json!({
            "version": 1,
            "reason": reason,
            "actor_pseudonym": actor_pseudonym,
            "endorsement_id": ref_id,
          });
          diesel::update(
            moderation_case::table.filter(moderation_case::id.eq(case.id)),
          )
          .set((
            moderation_case::status.eq(CaseStatus::SponsorLiabilityEscaped),
            moderation_case::liability_escape_reason.eq(Some(escape_reason_json)),
          ))
          .execute(conn)
          .await?;
          governance_log::append(
            &mut (&mut *conn).into(),
            ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED,
            json!({
              "case_id": case.id.0,
              "escaped_at": now,
              "reason": reason,
              "actor_pseudonym": actor_pseudonym,
              "endorsement_id": ref_id,
            }),
            Some(actor_pseudonym),
          )
          .await?;
          Ok(())
        }
        EscapeStatus::Fire => {
          // Fire branch (caller-driven) — requires sanction-action context.
          // Callers without that context must use run_grace_check_batch instead.
          Err(
            LemmyErrorType::Unknown(
              "fire_or_escape_case public entry: Fire branch requires sanction-action context; call run_grace_check_batch instead".to_string(),
            )
            .into(),
          )
        }
      }
    })
    .await
}

/// Staleness observability: emits `tracing::error!` if any
/// `SponsorLiabilityPending` case has `decided_at` older than
/// `max_grace_hours × multiplier` (per PRD §6.3 + DQ #146).
///
/// Pure observability — no DB writes, no governance_log entries.
#[expect(
  clippy::as_conversions,
  clippy::cast_precision_loss,
  clippy::cast_possible_truncation,
  reason = "max_grace_hours is a config (hours), bounded to ≤8760 in practice (1 year); multiplier is a small float (2.0 default per PRD §6.3); product fits i64 with no precision loss for the realistic range. Truncation of round() bounded by the same."
)]
pub async fn check_grace_staleness(
  conn: &mut AsyncPgConnection,
  max_grace_hours: i64,
  multiplier: f64,
  now: DateTime<Utc>,
) -> LemmyResult<()> {
  // threshold_hours = max_grace_hours * multiplier (DQ #146).
  // Defaults: 720 * 2.0 = 1440h ≈ 60d (matches PRD §6.3 "(>60 days)").
  let threshold_hours_f = (max_grace_hours as f64) * multiplier;
  let threshold_hours_i = threshold_hours_f.round() as i64;
  let threshold = now - chrono::Duration::hours(threshold_hours_i);

  // Per-case test: now - decided_at > threshold_hours
  // (i.e. decided_at < threshold). Measure from decided_at, NOT
  // grace_expires_at — the semantic is "pending so long it's escaped
  // notice"; decided_at is the canonical case-age anchor.
  let stuck_min_decided: Option<DateTime<Utc>> = moderation_case::table
    .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityPending))
    .filter(moderation_case::decided_at.lt(Some(threshold)))
    .select(min(moderation_case::decided_at))
    .first(conn)
    .await?;

  if let Some(min_decided) = stuck_min_decided {
    tracing::error!(
      target: "governance::integrity",
      decided_at = ?min_decided,
      threshold = ?threshold,
      max_grace_hours,
      multiplier,
      "sponsor_liability_grace staleness detected: at least one case has decided_at older than threshold (PRD §6.3)"
    );
  }

  Ok(())
}

/// Inner per-case transaction body. Called by `run_grace_check_batch`
/// inside each `run_transaction` closure. Steps per PRD §6.2:
/// 1. Re-load case with FOR UPDATE.
/// 2. Re-check status (defence against scheduler-vs-handler race).
/// 3. Lookup sanction action.
/// 4. Evaluate escape conditions.
/// 5. Branch: Escape → UPDATE + log; Fire → apply_sponsor_liability + UPDATE + log.
async fn fire_or_escape_case_inner(
  conn: &mut AsyncPgConnection,
  case: &ModerationCase,
  now: DateTime<Utc>,
) -> LemmyResult<PerCaseOutcome> {
  let mut per_case_cache = ConfigCache::new();
  let case_id = case.id;

  // Step 1: re-load with FOR UPDATE (watchpoint #1 + #2).
  let re_loaded: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .for_update()
    .select(ModerationCase::as_select())
    .first(conn)
    .await?;

  // Step 2: re-check status (defence against scheduler-vs-handler race).
  // Type-state guard: SponsorLiabilityPending only; all other variants → Skipped.
  let re_loaded = match GovernanceCase::<SponsorLiabilityPendingState>::try_from(re_loaded) {
    Ok(c) => c.inner,
    Err(_) => return Ok(PerCaseOutcome::Skipped),
  };

  let target_person_id = match re_loaded.target_person_id {
    Some(pid) => pid,
    None => {
      tracing::error!(
        target: "governance::integrity",
        case_id = re_loaded.id.0,
        "SponsorLiabilityPending case has NULL target_person_id"
      );
      return Ok(PerCaseOutcome::Skipped);
    }
  };
  let decided_at = match re_loaded.decided_at {
    Some(d) => d,
    None => {
      tracing::error!(
        target: "governance::integrity",
        case_id = re_loaded.id.0,
        "SponsorLiabilityPending case has NULL decided_at"
      );
      return Ok(PerCaseOutcome::Skipped);
    }
  };

  // Step 3: lookup sanction action (watchpoint #6).
  let sanction_row: Option<(SanctionAction, SanctionScope)> = sanction::table
    .filter(sanction::case_id.eq(case_id))
    .order_by(sanction::id.asc())
    .limit(1)
    .select((sanction::action, sanction::scope))
    .first::<(SanctionAction, SanctionScope)>(conn)
    .await
    .optional()?;
  let (action, _scope) = match sanction_row {
    Some(s) => s,
    None => {
      tracing::error!(
        target: "governance::integrity",
        case_id = re_loaded.id.0,
        "SponsorLiabilityPending case has zero sanction rows — v0 invariant violation; skipping"
      );
      return Ok(PerCaseOutcome::Skipped);
    }
  };

  // Step 4: evaluate escape conditions.
  let escape = evaluate_escape_conditions(
    conn,
    case_id,
    target_person_id,
    re_loaded.community_id,
    decided_at,
    &mut per_case_cache,
  )
  .await?;

  // Step 5: branch on EscapeStatus (ADR-013: exhaustive match, no `_ =>`).
  match escape {
    EscapeStatus::Escape {
      reason,
      actor_pseudonym,
      ref_id,
    } => {
      let escape_reason_json = json!({
        "version": 1,
        "reason": reason,
        "actor_pseudonym": actor_pseudonym,
        "endorsement_id": ref_id,
      });
      diesel::update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
        .set((
          moderation_case::status.eq(CaseStatus::SponsorLiabilityEscaped),
          moderation_case::liability_escape_reason.eq(Some(escape_reason_json)),
        ))
        .execute(conn)
        .await?;
      governance_log::append(
        &mut (&mut *conn).into(),
        ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED,
        json!({
          "case_id": case_id.0,
          "escaped_at": now,
          "reason": reason,
          "actor_pseudonym": actor_pseudonym,
          "endorsement_id": ref_id,
        }),
        Some(actor_pseudonym),
      )
      .await?;
      Ok(PerCaseOutcome::Escaped)
    }
    EscapeStatus::Fire => {
      let sponsor_count = sponsor_liability::apply_sponsor_liability(
        conn,
        target_person_id,
        case_id,
        re_loaded.community_id,
        action,
        &mut per_case_cache,
      )
      .await?;
      diesel::update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
        .set(moderation_case::status.eq(CaseStatus::SponsorLiabilityFired))
        .execute(conn)
        .await?;
      let target_pseudonym =
        actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), target_person_id).await?;
      governance_log::append(
        &mut (&mut *conn).into(),
        ENTRY_KIND_SPONSOR_LIABILITY_FIRED,
        json!({
          "case_id": case_id.0,
          "fired_at": now,
          "target_pseudonym": target_pseudonym,
          "sponsor_count": sponsor_count,
        }),
        Some(target_pseudonym),
      )
      .await?;
      Ok(PerCaseOutcome::Fired)
    }
  }
}
