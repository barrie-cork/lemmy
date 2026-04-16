//! Brehon governance CRUD-shaped handlers.
//!
//! Handlers that create new governance rows (reports, endorsements, etc.)
//! live here by convention, paralleling upstream Lemmy's split between
//! `lemmy_api` (workflow) and `lemmy_api_crud` (create/read/update/delete).
//! Phase 4a ships only `create_report`; further CRUD endpoints land in
//! Phase 5 per [IMPLEMENTATION-PLAN-v0.md §3].
