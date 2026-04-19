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
//! - [`inbox`] — re-export of the inbound receivers that live in
//!   `lemmy_apub_activities::governance::inbox` (Agent E, plan task 75).
//!   Per advisor decision DQ-6.6-inbound (resolved id 37 in
//!   `.claude/decision-queue.json`) the inbox bodies live in the
//!   activities crate so `Activity::receive` can call them; this
//!   `pub use` provides the consistent
//!   `lemmy_apub::governance::inbox::receive_remote_*` import path that
//!   external callers (admin tooling, future v1 apply paths) can use
//!   without taking a direct dep on `lemmy_apub_activities`.

pub mod outbox;
pub mod verify;

pub use lemmy_apub_activities::governance::inbox;
