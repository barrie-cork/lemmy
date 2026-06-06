//! Brehon governance HTTP handlers and cross-cutting helpers.
//!
//! Organisation matches `docs/brehon-law-inspired-network/03-architecture.md §7`:
//! workflow handlers (`get_case`, `list_my_jury_queue`, `submit_jury_vote`,
//! `list_modlog`) live here in `lemmy_api`; CRUD-shaped handlers
//! (`create_report`) live in `lemmy_api_crud`. The core write-path helpers
//! (`governance_log`, `actor_pseudonym_helper`, `redaction`) are colocated
//! with the workflow handlers because every write path depends on them.
//! Verdict-enforcement helpers such as `sponsor_liability` live here with
//! the workflow that invokes them — see [IMPLEMENTATION-PLAN-v0.md §4.1, §4.2].

#![deny(clippy::disallowed_methods)]

pub mod accept_jury_assignment;
pub mod actor_pseudonym_helper;
pub mod admin_assign_jury;
pub mod admin_audit_stream;
pub mod admin_close_case;
pub mod admin_config;
pub mod admin_dashboard;
pub mod admin_dashboard_html;
pub mod admin_emergency_remove;
pub mod admin_reputation_rollup;
pub mod admin_reputation_stats;
pub mod admin_rule_sets;
pub mod admin_sponsor_allowlist;
pub mod admin_trigger_appeal_rejury;
pub mod appeal_window_expiry;
pub(crate) mod audit_projection;
pub mod bridge_auth;
pub mod case_open_snapshot;
pub mod config;
pub mod decline_jury_assignment;
pub mod federation_outbox;
pub mod get_case;
pub mod get_my_reputation;
pub mod governance_log;
pub mod jury_common;
pub mod state;
pub mod list_cases;
pub mod list_modlog;
pub mod list_my_jury_queue;
pub mod messaging_config;
pub mod participation_cron;
pub mod redaction;
pub mod reputation_snapshot;
pub mod room_event_handler;
pub mod sponsor_liability;
pub mod sponsor_liability_grace;
pub mod submit_jury_vote;
