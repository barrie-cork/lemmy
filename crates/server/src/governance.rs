//! Governance composition root.
//!
//! Ties governance routes into the server binary and schedules the
//! governance-relevant background jobs. Stays strictly declarative per
//! [03 §11]. No business logic lives here; every ounce of decision-making
//! is in `crates/api/api/src/governance/*` or
//! `crates/api/api_crud/src/governance/*`.
//!
//! Phase 4b ships route wiring + **stub** background jobs. The real
//! schedulers (snapshot refresh, sanction cleanup, jury timeout reaping)
//! land in Phase 5/6 per [IMPLEMENTATION-PLAN-v0.md §3 Phase 5/6].

use lemmy_api_utils::context::LemmyContext;
use tracing::info;

/// Declarative stub — actual registration happens in
/// `lemmy_api_routes::utils::scheduled_tasks::setup`. This function only
/// logs at startup so we can confirm the composition root is reached and
/// keep a single place to add future governance-job logging without
/// touching `scheduled_tasks::setup`. Phase 6's real cleanup jobs (sanction
/// expiry, jury timeout reaping) will register additional closures there.
pub fn schedule_governance_jobs(_context: &LemmyContext) {
  info!(
    "governance: snapshot recalculation job registered via \
     lemmy_api::governance::reputation_snapshot::run_snapshot_batch \
     (15-minute tick in scheduled_tasks::setup; BREHON_DISABLE_SNAPSHOT_JOB=1 disables for e2e)",
  );
}
