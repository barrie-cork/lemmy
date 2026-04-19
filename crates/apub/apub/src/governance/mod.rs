//! Public-API boundary for the governance federation pipeline (Phase 6).
//!
//! Per [docs/brehon-law-inspired-network/04-data-model-and-api.md §11], the
//! `lemmy_apub` crate hosts the *boundary* between consumers (handlers in
//! `lemmy_api`, scheduled jobs in `lemmy_server`) and the underlying
//! `lemmy_apub_objects` / `lemmy_apub_activities` machinery.
//!
//! - [`outbox`] — outbound publishers (Agent D, plan task 74). Builder
//!   that returns a `SanctionNoticeSendPlan`; the orchestrator (Agent F's
//!   `lemmy_api::governance::federation_outbox`) opens the tx and calls
//!   both the activity enqueue + governance_log append. Resolves DQ-6.6.
//! - [`verify`] — shared verification helpers (Agent E, plan task 78).
//!   Thin wrapper over `activitypub_federation::protocol::verification`.
//!
//! `inbox` (plan task 75) is pending DQ-6.6 inbound resolution — see the
//! plan and `.claude/decision-queue.json` id 36 for the architectural
//! constraint that pushes the inbound receive body down to
//! `lemmy_apub_activities` or `lemmy_db_schema`.

pub mod outbox;
pub mod verify;
