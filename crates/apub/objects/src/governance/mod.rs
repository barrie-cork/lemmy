//! AP `Object` newtypes for governance object types (Phase 6).
//!
//! Three newtypes — `ApubSanctionNotice`, `ApubTrustAttestation`,
//! `ApubModerationLabel` — wrapping the local DB rows that store inbound
//! advisory federation traffic (`remote_sanction_notice`,
//! `federation_attestation`). Outbound flow does **not** route through
//! `Object::into_json` on these newtypes — Phase 6 task 74 (Agent D)
//! constructs `SanctionNoticeProtocol` etc. directly from the local
//! `Sanction`/`Person`/etc rows and wraps it in the Create Activity.
//!
//! Inbound flow:
//! 1. `Activity::receive` is called by `activitypub_federation`'s
//!    actix-web inbox after HTTP signature verification.
//! 2. Phase 6 task 75 (Agent E) fills the `from_json` bodies here to
//!    insert advisory rows into `remote_sanction_notice` /
//!    `federation_attestation` (with `local_case_id` left NULL per
//!    ADR-006 — never auto-applied).
//!
//! v0 scope per [docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md §3]:
//! `read_from_id` returns `Ok(None)` (governance objects aren't
//! URL-addressable); `into_json` returns `NotFound` (not re-served);
//! `delete` returns `NotFound` (advisory rows are not deletable in v0).

pub mod moderation_label;
pub mod sanction_notice;
pub mod trust_attestation;
