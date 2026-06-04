// v0 endpoint-count guideline (per ADR-010 + 05-mvp-and-delivery-plan §2):
// v0 is EXACTLY 11 user-facing endpoints. DTOs in this file that do not map
// to one of those 11 are explicitly carved out:
//
//   - `RevokeEndorsement` — endorsement-management UX flow, not a top-level
//     v0 endpoint.
//   - `Admin*` DTOs (`AdminAssignJury`, `AdminCloseCase`,
//     `AdminTriggerAppealRejury`, `AdminReputationStats`, `AdminSetConfig`,
//     `AdminGetConfig`, `AdminGetConfigAudit`, `AdminCreateRuleSet`,
//     `AdminListRuleSets`, `AdminDashboard*`, …) — admin backstops, not
//     user-facing. Approved as out-of-scope of the 11-endpoint count by
//     plan §11.2 GOTCHA + Phase 5b/5c decision notes.
//   - `AddSponsorAllowlist`, `AddSponsorAllowlistResponse`,
//     `RemoveSponsorAllowlist`, `RemoveSponsorAllowlistResponse` — v1-RT-r4
//     admin sponsor-allowlist endpoints; not a v0 user-facing endpoint.
//     Approved as out-of-scope per v1-RT-r4 plan §16 (admin backstop).
//
// New non-Admin DTOs added here that don't correspond to one of the 11
// require a new carve-out entry above. Closes #40.

use chrono::{DateTime, Utc};
use lemmy_db_schema::newtypes::{AppealId, CommunityId, EndorsementId, ModerationCaseId, SponsorAllowlistId};
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseStatus, CaseTargetType, JuryDecision},
};
use lemmy_db_schema::source::governance::reputation_snapshot::ReputationSnapshot;
use lemmy_db_views_reputation::ReputationSummaryView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// ── Group A: Reports + Cases ──────────────────────────────────────────

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a governance report (may open or append to a case).
pub struct CreateGovernanceReport {
  pub community_id: Option<CommunityId>,
  pub target_type: CaseTargetType,
  /// The ID of the target entity. Interpretation depends on `target_type`:
  /// - Post → PostId
  /// - Comment → CommentId
  /// - Person → PersonId (as i32)
  /// - Community → CommunityId (as i32)
  ///
  /// Kept as raw i32 because a single field serves multiple newtype
  /// domains. The handler in Phase 4 will cast to the appropriate
  /// newtype based on `target_type`.
  pub target_id: i32,
  pub reason_code: String,
  pub description: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from creating a governance report.
pub struct CreateGovernanceReportResponse {
  pub case_id: Option<ModerationCaseId>,
  pub threshold_met: bool,
  pub case: lemmy_db_views_governance_case::GovernanceCaseSummaryView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Read a single governance case.
pub struct GetGovernanceCase {
  pub case_id: ModerationCaseId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// List governance cases, filtered by community and/or status.
pub struct ListGovernanceCases {
  pub community_id: Option<CommunityId>,
  pub status: Option<CaseStatus>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from `list_cases`. Returns a list of summary views filtered
/// by the request parameters.
pub struct ListGovernanceCasesResponse {
  pub cases: Vec<lemmy_db_views_governance_case::GovernanceCaseSummaryView>,
}

// ── Group B: Jury ─────────────────────────────────────────────────────

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Submit a jury vote on a case.
pub struct SubmitJuryVote {
  pub case_id: ModerationCaseId,
  pub decision: JuryDecision,
  pub rationale: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from submitting a jury vote. When `case_decided` is true,
/// `decision` holds the majority outcome; otherwise it is `None` and
/// the case remains open for further votes.
pub struct SubmitJuryVoteResponse {
  pub vote_recorded: bool,
  pub case_decided: bool,
  pub decision: Option<JuryDecision>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Accept a jury assignment.
pub struct AcceptJuryAssignment {
  pub case_id: ModerationCaseId,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from accepting a jury assignment.
pub struct AcceptJuryAssignmentResponse {
  pub case_id: ModerationCaseId,
  pub accepted: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Decline a jury assignment. Triggers replacement juror selection.
pub struct DeclineJuryAssignment {
  pub case_id: ModerationCaseId,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from declining a jury assignment. `replacement_person_id` is
/// `Some` if a replacement juror was selected; `None` if the eligible pool
/// was exhausted (v0 behaviour; admin can re-run assign-jury manually).
pub struct DeclineJuryAssignmentResponse {
  pub case_id: ModerationCaseId,
  pub declined: bool,
  pub replacement_person_id: Option<PersonId>,
}

// ── Group F: Admin Backstops ──────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Request payload for `POST /api/v4/governance/admin/assign-jury`.
/// Admin-only in v0 per [99 ADR-007]; flips a case from `Open`/`ThresholdMet`
/// to a five-juror panel with assignments auto-promoted to `Accepted` for
/// testability (Phase 5 introduces a proper Selected → Accepted flow).
pub struct AdminAssignJury {
  pub case_id: ModerationCaseId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from the admin-assign-jury backstop. `assigned_person_ids`
/// is the list of the five jurors selected.
pub struct AdminAssignJuryResponse {
  pub case_id: ModerationCaseId,
  pub assigned_person_ids: Vec<PersonId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Request payload for `POST /api/v4/governance/admin/close-case`.
/// Single-admin v0 simplification per [99 ADR-010] — quorum + delay
/// on admin close-case is a v2 item. `reason` is required and is
/// included in the governance log audit entry (scrubbed by the
/// redaction layer inside `governance_log::append`).
pub struct AdminCloseCase {
  pub case_id: ModerationCaseId,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from the admin-close-case backstop.
pub struct AdminCloseCaseResponse {
  pub case_id: ModerationCaseId,
  pub closed: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Request payload for `POST /api/v4/governance/admin/emergency-remove/flag-bad-faith`.
/// Admin-only per ADR-013 + PRD section 5.3 source 4b. Flags the
/// reporter of an `EmergencyRemove`-status case as bad-faith; emits
/// `-1 reporting_accuracy`.
pub struct FlagBadFaithEmergencyReport {
  pub case_id: ModerationCaseId,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from `flag_bad_faith_emergency_report`. `flagged: true`
/// confirms the reputation_event row was written (or de-duped to a
/// prior identical row via the dedupe_key partial unique index).
pub struct FlagBadFaithEmergencyReportResponse {
  pub case_id: ModerationCaseId,
  pub flagged: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Admin-triggered appeal-rejury request — used when
/// `appeal.auto_select_on_appeal_acceptance = false`.
pub struct AdminTriggerAppealRejury {
  pub case_id: ModerationCaseId,
  /// v1-JM-e + PRD §12.3: reserved slot for v2 step-up auth on admin-mediated mid-case
  /// actions. v1 ignore-behaviour — the field is read from the wire, never validated,
  /// never rejected.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub step_up_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from admin-trigger-appeal-rejury.
pub struct AdminTriggerAppealRejuryResponse {
  pub case_id: ModerationCaseId,
  pub appeal_id: AppealId,
  pub panel_person_ids: Vec<PersonId>,
}

// ── Group C: Appeals ──────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Request an appeal on a decided case.
pub struct RequestAppeal {
  pub case_id: ModerationCaseId,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from requesting an appeal. `appeal_id` is the newly-inserted
/// appeal row id. Case status flips from `Decided` → `Appealed`; no new
/// jury is assembled automatically in v0 (per ADR-010).
pub struct RequestAppealResponse {
  pub appeal_id: AppealId,
  pub case_id: ModerationCaseId,
}

// ── Group E: Public Log ───────────────────────────────────────────────

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// List the public governance modlog. Public endpoint, no auth required.
pub struct ListGovernanceModlog {
  pub community_id: Option<CommunityId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

// ── Group D: Reputation / Trust ───────────────────────────────────────

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Get the calling user's own reputation summary.
pub struct GetMyReputation {
  pub community_id: Option<CommunityId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from `get_my_reputation`. Wraps a `ReputationSummaryView` —
/// the four raw dimension scores (`reporting_accuracy`, `jury_reliability`,
/// `participation_consistency`, `endorsement_strength`) are `#[serde(skip)]`
/// per ADR-005, so the wire shape exposes capabilities (`jury_eligible`,
/// `trusted_reporter`) and counts (`active_sanctions`) only.
pub struct GetMyReputationResponse {
  pub view: ReputationSummaryView,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create an endorsement of another user.
pub struct CreateEndorsement {
  pub person_id: PersonId,
  pub community_id: Option<CommunityId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from creating an endorsement.
pub struct CreateEndorsementResponse {
  pub endorsement_id: EndorsementId,
  pub surety_created: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Revoke an existing endorsement. PRD §5.1 — required reason flows
/// through governance_log::append's scrub layer per ADR-015.
pub struct RevokeEndorsement {
  pub endorsement_id: EndorsementId,
  pub reason: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from revoking an endorsement. PRD §5.1.
/// `liability_chain_severed_for_cases` is non-empty when the
/// revocation severed one or more `SponsorLiabilityPending` cases'
/// grace windows (§5.3 step 4).
pub struct RevokeEndorsementResponse {
  pub endorsement_id: EndorsementId,
  pub revoked_at: DateTime<Utc>,
  pub liability_chain_severed_for_cases: Vec<ModerationCaseId>,
}

// ── Group G: Admin Observability ──────────────────────────────────────

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Admin-only request for cross-population reputation observability stats.
/// `community_id` filters the bucket queries to a specific community
/// (rows where `reputation_snapshot.community_id = community_id`); when
/// `None`, the queries run instance-wide on `community_id IS NULL` rows.
pub struct AdminReputationStats {
  pub community_id: Option<CommunityId>,
}

/// Per-dimension snapshot histograms. Five fixed buckets per dimension:
/// `[0, 1-30, 31-80, 81-200, 200+]` — matches probe-62 verified shape on
/// pg18 (see `scratch/phase-5c-probes/bucket_query.sql`).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ReputationBuckets {
  pub reporting_accuracy: [i64; 5],
  pub jury_reliability: [i64; 5],
  pub participation_consistency: [i64; 5],
  pub endorsement_strength: [i64; 5],
}

/// Current threshold values from the governance config (instance scope).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ThresholdsSnapshot {
  pub jury_reliability: i64,
  pub reporting_accuracy: i64,
  pub endorsement_strength: i64,
}

/// `COUNT(*)` aggregates over `reputation_snapshot` for the three v0
/// capability bits.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct CapabilityCounts {
  pub jury_eligible_count: i64,
  pub trusted_reporter_count: i64,
  pub can_sponsor_count: i64,
}

/// Founder-event tally per `expires_at` window (active = future-expiry,
/// expired = past-expiry).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct FounderEventStats {
  pub active_count: i64,
  pub expired_count: i64,
}

/// Response shape for `admin_reputation_stats` per IMPLEMENTATION-PLAN-v0.md
/// line 373. Single round-trip per dimension via `CASE WHEN` bucketing —
/// see plan §11.2 GOTCHA + probe 62-sql.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminReputationStatsResponse {
  pub buckets: ReputationBuckets,
  pub thresholds_current: ThresholdsSnapshot,
  pub capability_counts: CapabilityCounts,
  pub founder_event_stats: FounderEventStats,
  pub calculated_at: DateTime<Utc>,
}

// ── Group H: Admin Config (v1-AD-b) ───────────────────────────────────

/// Request payload for `POST /api/v4/governance/admin/config`.
///
/// `value_type` and `value` are typed-on-server: the handler rehydrates the
/// JSON value against the key's `CONFIG_KEY_METADATA.value_type` and rejects
/// type mismatches (plan §13 task 4 GOTCHA). `scope` is either the literal
/// string `"instance"` or `"community:<id>"` — the handler parses the
/// community-scoped form with a regex.
///
/// `apply_at` parses but v1-AD-b does NOT act on a future `valid_from` —
/// delayed activation lands in v1-AD-b.1 (plan §4.1). `dry_run = Some(true)`
/// returns the `ConfigChangePreview` without writing.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminSetConfig {
  pub key: String,
  pub value_type: String,
  pub value: serde_json::Value,
  pub scope: String,
  pub apply_at: Option<String>,
  pub dry_run: Option<bool>,
  pub reason: String,
}

/// Response from `POST /api/v4/governance/admin/config`.
///
/// When `dry_run = Some(true)` on the request, `applied = false`, both id
/// fields are `None`, `applied_at` is `None`, and only `preview` is
/// populated. When `dry_run` is false or absent, all four write-side fields
/// carry the new row's identifiers and the preview mirrors the pre-tx
/// snapshot.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminSetConfigResponse {
  pub applied: bool,
  pub config_id: Option<i64>,
  pub governance_log_id: Option<i64>,
  pub preview: ConfigChangePreview,
  pub applied_at: Option<DateTime<Utc>>,
}

/// Pre/post value + impact summary attached to every `AdminSetConfigResponse`
/// (including dry-runs).
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ConfigChangePreview {
  pub previous: ConfigValueWithProvenance,
  pub new: ConfigValueWithProvenance,
  pub downstream_impact: serde_json::Value,
}

/// A single `(value, provenance)` pair. `effective_from` is one of:
/// `"community:<id>"` — a community-scoped row overrides `instance`
/// `"instance"` — the instance-scoped row is in effect
/// `"default"` — no row exists; the Rust const default applies
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ConfigValueWithProvenance {
  pub value: serde_json::Value,
  pub effective_from: String,
}

/// Request payload for `POST /api/v4/governance/admin/messaging-config`.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminSetMessagingConfig {
  pub scope: String,
  pub key: String,
  pub value: serde_json::Value,
}

/// Response from `POST /api/v4/governance/admin/messaging-config`.
///
/// Returns the previous and new effective values after a single-write
/// update (no governance_log append, no dry-run — plan §10.3 + §12).
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminSetMessagingConfigResponse {
  pub previous: ConfigValueWithProvenance,
  pub new: ConfigValueWithProvenance,
}

/// Request payload for `GET /api/v4/governance/admin/config`.
///
/// With no params: returns every key in `CONFIG_KEY_METADATA`. With `key`:
/// returns a single `AdminConfigEntry` for that key. `community_id` narrows
/// the cascade to a specific community when the key's metadata scope allows.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminGetConfig {
  pub key: Option<String>,
  pub community_id: Option<CommunityId>,
}

/// Response from `GET /api/v4/governance/admin/config`. Single-element
/// `entries` when the request carried a `key`, otherwise one entry per
/// `CONFIG_KEY_METADATA` row.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminGetConfigResponse {
  pub entries: Vec<AdminConfigEntry>,
}

/// Per-key response row: the effective value + its provenance + the
/// metadata fields needed by a dashboard UI (scope, gating flags, valid
/// range/enum). Field names mirror `ConfigKeyMetadata` so the UI can bind
/// directly; `value` is always the typed JSON (int → number, bool → bool,
/// enum/text → string).
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminConfigEntry {
  pub key: String,
  pub value_type: String,
  pub value: serde_json::Value,
  pub effective_from: String,
  pub scope: String,
  pub requires_re_jury: bool,
  pub requires_step_up: bool,
  pub apply_at_default: String,
  pub description: String,
  pub doc_anchor: String,
  pub valid_range: Option<(f64, f64)>,
  pub valid_enum: Option<Vec<String>>,
}

/// Request payload for `GET /api/v4/governance/admin/config/audit`.
///
/// Pagination: `page` (1-based, default 1), `limit` (default 20, clamped to
/// [1, 100]). Filters: `key` / `scope` (payload-side post-filter in Rust —
/// JSONB filter pushdown deferred), `actor_pseudonym` (indexed column),
/// `since` / `until` (half-open range on `created_at`).
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminGetConfigAudit {
  pub key: Option<String>,
  pub scope: Option<String>,
  pub actor_pseudonym: Option<String>,
  pub since: Option<DateTime<Utc>>,
  pub until: Option<DateTime<Utc>>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

/// One row of the audit list. Fields are projected from the
/// `governance_log.payload` JSONB column into typed response fields so the
/// caller doesn't re-implement payload destructuring. `previous_value` and
/// `previous_from` are `None` on rows written before v1-AD-c's Issue #77
/// refactor (the shell wrapper and pre-v1-AD-c handler rows); populated
/// on every `admin_config_changed` row written by the post-v1-AD-c
/// handler. `denial_reason` is populated only on
/// `entry_kind = "admin_config_change_denied"` rows.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminConfigAuditEntry {
  pub id: i64,
  pub entry_kind: String,
  pub scope: String,
  pub key: String,
  pub value_type: String,
  pub previous_value: Option<serde_json::Value>,
  /// Provenance label for `previous_value` — one of `"default"` (const
  /// fallback, no DB row), `"instance"`, or `"community:<id>"`. `None` on
  /// pre-v1-AD-c rows (shell-written or pre-refactor handler-written) and
  /// on denials.
  pub previous_from: Option<String>,
  pub new_value: serde_json::Value,
  pub reason: String,
  pub actor_pseudonym: Option<String>,
  pub created_at: DateTime<Utc>,
  pub signature: Option<Vec<u8>>,
  pub denial_reason: Option<String>,
}

// ── Group C: Rule-set versioning (v1-AD-c) ────────────────────────────

/// Create an append-only rule-set version for a community. Flips
/// `rule_set.active_version_id` at `Scope::Community(community_id)` to
/// the newly-inserted version atomically with the INSERT.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminCreateRuleSet {
  pub community_id: CommunityId,
  pub rule_text: String,
  /// Optional previous `rule_set_version.id` this version chains from.
  /// Must belong to the same `community_id`. When `None`, the new version
  /// is the root of its chain.
  pub parent_id: Option<i32>,
  pub reason: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminCreateRuleSetResponse {
  pub rule_set_version_id: i32,
  pub version: i32,
  pub config_id: Option<i64>,
  pub governance_log_id: i64,
  pub created_at: DateTime<Utc>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminListRuleSetsRequest {
  pub community_id: CommunityId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminListRuleSetsResponse {
  pub versions: Vec<RuleSetVersionView>,
  pub active_version_id: Option<i32>,
}

/// A single rule-set version as projected for the list endpoint. The
/// `text_sha256` DB column is `BYTEA`; the wire representation is hex
/// (per Phase 6 federation convention). `created_by_pseudonym` is
/// resolved via a secondary lookup against `actor_pseudonym` in the
/// handler (`None` in v1-AD-c; full population lands with v1-AD-d).
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RuleSetVersionView {
  pub id: i32,
  pub community_id: CommunityId,
  pub version: i32,
  pub parent_id: Option<i32>,
  pub text_sha256_hex: String,
  pub rule_text: String,
  pub created_at: DateTime<Utc>,
  pub created_by_pseudonym: Option<String>,
}

// ── Group D: Admin dashboard aggregate (v1-AD-d) ──────────────────────

/// Single-fetch instance-wide dashboard aggregate. `Eq` is intentionally
/// omitted because `recent_config_changes` carries `serde_json::Value`
/// payload fields via `AdminConfigAuditEntry` (no `Eq` impl).
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminDashboardResponse {
  pub active_cases: ActiveCasesSummary,
  pub jury_queue: JuryQueueSummary,
  pub recent_config_changes: Vec<AdminConfigAuditEntry>,
  pub federation: FederationSummary,
  pub reputation: AdminReputationStatsResponse,
  pub rule_sets: RuleSetSummary,
  pub calculated_at: DateTime<Utc>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ActiveCasesSummary {
  /// Keys: PascalCase `CaseStatus` variants ("Open", "ThresholdMet", ...).
  /// `BTreeMap` (not `HashMap`) so serialisation order is stable for
  /// golden tests and matches the verbatim `DbValueStyle` casing of the
  /// Postgres `case_status` enum.
  pub by_status: std::collections::BTreeMap<String, i64>,
  /// Sum of counts for all variants EXCEPT `Decided`, `Closed`,
  /// `EmergencyRemove`. Clients that disagree about which variants are
  /// "active" (e.g. wanting to count `Appealed` separately) can re-derive
  /// from `by_status`.
  pub total_active: i64,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct JuryQueueSummary {
  /// `JuryAssignment.status = 'Selected'` — juror notified, not yet
  /// responded.
  pub pending_accept: i64,
  /// `JuryAssignment.status = 'Accepted'`.
  pub accepted: i64,
  /// `JuryAssignment.status = 'Submitted'` — vote cast.
  pub submitted: i64,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct FederationSummary {
  /// `valid_until IS NULL OR valid_until > now()`.
  pub active: i64,
  /// `valid_until IS NOT NULL AND valid_until <= now()`.
  pub expired: i64,
  pub total: i64,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RuleSetSummary {
  pub communities_with_rule_sets: i64,
  pub total_versions: i64,
  /// Bounded to 100 entries per the v1-AD-d plan §4.1 load-bearing
  /// decision; pilot instances have ≤5 communities with rule-sets in v1.
  pub per_community: Vec<PerCommunityActiveRuleSet>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct PerCommunityActiveRuleSet {
  pub community_id: CommunityId,
  /// `None` if the community has `rule_set_version` rows but no
  /// `rule_set.active_version_id` config row (allowed by design — v1-AD-c
  /// never seeds the key).
  pub active_version_id: Option<i32>,
}

// ── Group E: Admin sponsor-allowlist (v1-RT-r4) ───────────────────────

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AddSponsorAllowlist {
  pub person_id: PersonId,
  pub community_id: Option<CommunityId>,
  pub note: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AddSponsorAllowlistResponse {
  pub allowlist_id: SponsorAllowlistId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RemoveSponsorAllowlist {
  pub person_id: PersonId,
  pub community_id: Option<CommunityId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RemoveSponsorAllowlistResponse {
  pub success: bool,
}

// ── Group F: Admin reputation rollup (v1-RT-r5) ───────────────────────

/// Request payload for `GET /api/v4/governance/admin/reputation/rollup`.
/// `person_id` identifies the person whose instance-wide rollup row and
/// contributing per-community snapshots are returned.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminReputationRollup {
  pub person_id: PersonId,
}

/// Response from `GET /api/v4/governance/admin/reputation/rollup`.
/// `rollup` is `None` if no instance-wide snapshot exists yet for this
/// person (cron has not yet run, or all their communities are banned).
/// `contributing` is the ordered set of per-community snapshots (non-NULL
/// `community_id`) that fed the rollup computation; may be empty if the
/// person has no per-community rows.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminReputationRollupResponse {
  pub rollup: Option<ReputationSnapshot>,
  pub contributing: Vec<ReputationSnapshot>,
}
