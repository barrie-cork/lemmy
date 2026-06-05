//! Appeal-window expiry background job per PRD §9.5.
//!
//! Runs hourly: finds Decided cases whose appeal_window_expires_at is
//! in the past and flips them to Closed, emitting `appeal_window_expired`
//! per row. Uses FOR UPDATE SKIP LOCKED to coexist with concurrent
//! request_appeal handlers without deadlocks (per JM-c retro §3.2
//! amendment 1 / DQ #50 lock-acquisition order).

use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, dsl::update};
use diesel_async::RunQueryDsl;
use lemmy_api_utils::{bridge_notify::governance_case_after_transition, context::LemmyContext};
use lemmy_db_schema::source::governance::moderation_case::ModerationCase;
use lemmy_db_schema_file::{enums::CaseStatus, schema::moderation_case};
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::LemmyResult;
use serde_json::json;
use tracing::info;

use crate::governance::{governance_log, governance_log::ENTRY_KIND_APPEAL_WINDOW_EXPIRED};

#[derive(Debug, Default)]
pub struct AppealWindowExpiryOutcome {
  pub cases_processed: usize,
}

/// Hourly tick — sweeps Decided cases past their
/// appeal_window_expires_at, flips each to Closed, emits
/// appeal_window_expired. SKIP LOCKED keeps concurrent request_appeal
/// handlers safe (request_appeal acquires its row lock implicitly via
/// the case load + update sequence; SKIP LOCKED here means a row
/// currently being mutated by request_appeal is left for the next
/// tick — preferable to a deadlock).
pub async fn run_appeal_window_expiry_batch(
  context: &LemmyContext,
) -> LemmyResult<AppealWindowExpiryOutcome> {
  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let mut outcome = AppealWindowExpiryOutcome::default();
  let now = Utc::now();

  // Single-statement select with FOR UPDATE SKIP LOCKED so concurrent
  // mutations on the same row defer to the next tick. Per the JM-c
  // retro §3.2 amendment 1: lock-acquisition order matters, and a
  // batch UPDATE is the only writer here, so taking the lock as the
  // first action on each row is safe.
  let candidates: Vec<ModerationCase> = moderation_case::table
    .filter(moderation_case::status.eq(CaseStatus::Decided))
    .filter(moderation_case::appeal_window_expires_at.lt(now))
    .for_update()
    .skip_locked()
    .load(conn)
    .await?;

  let mut pending_hooks: Vec<(ModerationCase, CaseStatus)> = Vec::new();
  for case in &candidates {
    update(moderation_case::table.filter(moderation_case::id.eq(case.id)))
      .set((
        moderation_case::status.eq(CaseStatus::Closed),
        moderation_case::closed_at.eq(Some(now)),
      ))
      .execute(conn)
      .await?;

    governance_log::append(
      &mut (&mut *conn).into(),
      ENTRY_KIND_APPEAL_WINDOW_EXPIRED,
      json!({
        "case_id": case.id.0,
        "decided_at": case.decided_at,
        "window_expired_at": case.appeal_window_expires_at,
      }),
      // No actor pseudonym — system-issued by the scheduler.
      None,
    )
    .await?;

    pending_hooks.push((case.clone(), CaseStatus::Closed));
    outcome.cases_processed += 1;
  }

  // Fire hooks after loop — DB work complete, not inside a transaction.
  for (case, new_status) in pending_hooks {
    governance_case_after_transition(context, &case, Some(CaseStatus::Decided), new_status)
      .await
      .ok();
  }

  if !candidates.is_empty() {
    info!(
      "governance: appeal-window expiry tick — cases_processed={}",
      outcome.cases_processed
    );
  }
  Ok(outcome)
}
