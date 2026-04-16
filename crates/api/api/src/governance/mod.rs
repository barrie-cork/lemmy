//! Brehon governance HTTP handlers and cross-cutting helpers.
//!
//! Organisation matches `docs/brehon-law-inspired-network/03-architecture.md §7`:
//! workflow handlers (`get_case`, `list_my_jury_queue`, `submit_jury_vote`,
//! `list_modlog`) live here in `lemmy_api`; CRUD-shaped handlers
//! (`create_report`) live in `lemmy_api_crud`. The three cross-cutting
//! helpers (`governance_log`, `actor_pseudonym_helper`, `redaction`) are
//! colocated with the workflow handlers because every write path depends
//! on them — see [IMPLEMENTATION-PLAN-v0.md §4.1, §4.2].

pub mod actor_pseudonym_helper;
pub mod get_case;
pub mod governance_log;
pub mod list_modlog;
pub mod list_my_jury_queue;
pub mod redaction;
pub mod submit_jury_vote;
