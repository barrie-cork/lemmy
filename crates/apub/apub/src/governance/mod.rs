//! Public-API boundary for the governance federation pipeline (Phase 6).
//!
//! Per [docs/brehon-law-inspired-network/04-data-model-and-api.md §11], the
//! `lemmy_apub` crate hosts the *boundary* between consumers (handlers in
//! `lemmy_api`, scheduled jobs in `lemmy_server`) and the underlying
//! `lemmy_apub_objects` / `lemmy_apub_activities` machinery. For Phase 6
//! the boundary is three modules:
//!
//! - [`outbox`] — outbound publishers (Agent D, plan task 74).
//! - `inbox` — inbound receivers (Agent E, plan task 75; created in
//!   parallel by Agent E so this `mod.rs` does NOT declare it here.
//!   Agent E adds `pub mod inbox;` in their commit; the advisor merges
//!   the resulting alphabetised list).
//! - `verify` — shared verification helpers (Agent E, plan task 78).
//!
//! Agents D and E run in parallel and write to disjoint files within
//! this directory. To minimise merge-conflict surface, this `mod.rs` is
//! advised in the brief to declare ONLY the module the current agent
//! owns. Agent D therefore declares only `outbox`; `inbox` and `verify`
//! arrive in Agent E's commit. The advisor's `--no-ff` merge resolves
//! the (trivial) `pub mod` ordering conflict.

pub mod outbox;
