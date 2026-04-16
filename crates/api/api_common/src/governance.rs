use lemmy_db_schema::newtypes::{CommunityId, EndorsementId, ModerationCaseId};
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseStatus, CaseTargetType, JuryDecision},
};
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
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from creating a governance report.
pub struct CreateGovernanceReportResponse {
  pub case_id: Option<ModerationCaseId>,
  pub threshold_met: bool,
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

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Decline a jury assignment. Triggers replacement juror selection.
pub struct DeclineJuryAssignment {
  pub case_id: ModerationCaseId,
  pub reason: Option<String>,
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

// ── Group C: Appeals ──────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Request an appeal on a decided case.
pub struct RequestAppeal {
  pub case_id: ModerationCaseId,
  pub reason: String,
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
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create an endorsement of another user.
pub struct CreateEndorsement {
  pub person_id: PersonId,
  pub community_id: Option<CommunityId>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Revoke an existing endorsement.
pub struct RevokeEndorsement {
  pub endorsement_id: EndorsementId,
}
