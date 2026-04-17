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

/// Register governance-relevant background jobs.
///
/// In Phase 4b these are all no-ops that log a message at startup so we
/// can confirm the composition root is reached. Phase 5 replaces them
/// with real schedulers. The actual route registration lives in
/// `lemmy_api_routes::config` where the `/governance` scope is already
/// wired (Phase 4a + task 5 of this phase).
pub fn schedule_governance_jobs(_context: &LemmyContext) {
  info!(
    "governance: snapshot recalculation job registered via \
     lemmy_api::governance::reputation_snapshot::run_snapshot_batch \
     (15-minute tick in scheduled_tasks::setup; BREHON_DISABLE_BACKGROUND_JOBS=1 disables for e2e)",
  );
}
