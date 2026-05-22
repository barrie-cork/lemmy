use crate::newtypes::{CommentId, CommunityId, ModerationCaseId, PostId, RuleSetVersionId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::moderation_case;
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseSeverity, CaseStatus, CaseStatusTier, CaseTargetType, JuryDecision, SeverityTier},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = moderation_case))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A governance moderation case — the central artefact of the jury workflow.
pub struct ModerationCase {
  pub id: ModerationCaseId,
  pub community_id: Option<CommunityId>,
  pub creator_id: Option<PersonId>,
  pub target_type: CaseTargetType,
  pub target_post_id: Option<PostId>,
  pub target_comment_id: Option<CommentId>,
  pub target_person_id: Option<PersonId>,
  pub target_community_id: Option<CommunityId>,
  pub target_remote_url: Option<String>,
  pub reason_code: String,
  pub severity: CaseSeverity,
  pub status: CaseStatus,
  pub threshold_score: i64,
  pub opened_at: DateTime<Utc>,
  pub decided_at: Option<DateTime<Utc>>,
  pub closed_at: Option<DateTime<Utc>>,
  /// v1-AD-a task 3/4: JSONB snapshot of `governance_config` values in effect
  /// when this case opened. Populated by v1-AD-b's case-open handler; NULL
  /// for pre-v1 cases and for v1 cases that pre-date the snapshot feature.
  pub applied_config_snapshot: Option<Value>,
  /// v1-AD-a task 3/4: FK to `rule_set_version.id` capturing which version
  /// of the community rules was in force at case-open time. Populated by
  /// v1-AD-c's rule-set wiring; NULL until then.
  pub rule_set_version_id: Option<RuleSetVersionId>,
  /// v1-JM-a §8.1: severity tier frozen at admin_assign_jury time per
  /// ADR-010. NOT NULL; `'Minor'` default pre-populated by the backfill
  /// in `2026-04-23-000100_add_jury_mechanics_columns/up.sql`. Distinct
  /// from `severity` (CaseSeverity Low/Medium/High/Critical) — this is
  /// the procedural-tier classifier that drives the
  /// `jury.panel_size.<status>.<severity>` cascade.
  pub severity_tier: SeverityTier,
  /// v1-JM-a §8.1: target-status tier typed as `CaseStatusTier`
  /// (Founder / Regular / Probation — see `crates/db_schema_file/src/enums.rs`),
  /// determined from the target's `reputation_event` / `membership_state` at
  /// case-open time. Distinct from `severity_tier` (`SeverityTier`
  /// Minor/Moderate/Severe — procedural tier). Both snapshot at jury-seating
  /// and together key the `jury.panel_size.<status>.<severity>` cascade per
  /// PRD §3.3.
  pub status_tier: CaseStatusTier,
  /// v1-JM-a §8.1: integer snapshot at jury-seating time. NULL for
  /// cases that haven't reached `JurySelection` yet; populated by
  /// v1-JM-b's admin_assign_jury cascade. Pre-v1 cases get 5 from the
  /// JM-a backfill.
  pub panel_size_snapshot: Option<i32>,
  /// v1-JM-a §8.1: minimum vote count for the panel to render a
  /// decision. NULL pre-seating; populated by v1-JM-b. Pre-v1 backfill: 3.
  pub quorum_snapshot: Option<i32>,
  /// v1-JM-a §8.1: number of panel votes required for a sanction to
  /// pass (per `jury.threshold_fraction.<severity>`). Pre-v1 backfill: 3.
  pub threshold_count_snapshot: Option<i32>,
  /// v1-JM-a §8.1 + §6.5: set by v1-JM-c submit_jury_vote at
  /// case-decision time (`decided_at + appeal.window_days`). NULL until
  /// the case is decided; pre-v1 backfill: COALESCE(closed_at,
  /// decided_at + 7d) — see `2026-04-23-000100_*/up.sql`.
  pub appeal_window_expires_at: Option<DateTime<Utc>>,
  /// v1-JM-d §9.3: snapshot of the winning JuryDecision recorded by
  /// submit_jury_vote at decision time. Used by request_appeal's
  /// reporter-rights check (PRD §6.4) to determine whether the
  /// original reporter has a legitimate grievance (only on NoAction or
  /// AdvisoryLabel outcomes). NULL until JM-d is in place; pre-v1
  /// Decided cases stay NULL and reporter-rights is naturally false.
  pub winning_decision: Option<JuryDecision>,
  /// v1-SL-a §8.1: grace-window deadline. NULL for cases not in
  /// `SponsorLiabilityPending` state. Set by SL-d's `submit_jury_vote`
  /// rewrite at `Decided -> SponsorLiabilityPending` transition; pre-v1
  /// backfill sets it for v0 mid-flight cases. Locked thereafter.
  pub grace_expires_at: Option<DateTime<Utc>>,
  /// v1-SL-a §8.1 + OQ-V1-SL-05: structured escape-reason payload
  /// (version: 1 from day one). NULL for non-escaped cases. Set by
  /// SL-b (revoke_endorsement escape branch) or SL-c (scheduler escape
  /// branch).
  pub liability_escape_reason: Option<Value>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = moderation_case))]
pub struct ModerationCaseInsertForm {
  pub community_id: Option<CommunityId>,
  pub creator_id: Option<PersonId>,
  pub target_type: CaseTargetType,
  pub target_post_id: Option<PostId>,
  pub target_comment_id: Option<CommentId>,
  pub target_person_id: Option<PersonId>,
  pub target_community_id: Option<CommunityId>,
  pub target_remote_url: Option<String>,
  pub reason_code: String,
  pub severity: CaseSeverity,
  pub status: CaseStatus,
  pub threshold_score: i64,
  /// v1-AD-a additions. Optional on insert — v1-AD-b handler populates
  /// them; v0 + earlier-v1 callers leave both as `None`.
  pub applied_config_snapshot: Option<Value>,
  pub rule_set_version_id: Option<RuleSetVersionId>,
  /// v1-JM-a additions. All `Option<_>` so v0/earlier-v1 callers that
  /// set only `severity` (CaseSeverity) continue to compile; the
  /// Postgres-side DEFAULT 'Minor' / 'Regular' covers the not-null
  /// enum columns when omitted, and the integer snapshots stay NULL
  /// until v1-JM-b/c writers populate them at jury-seating time.
  pub severity_tier: Option<SeverityTier>,
  pub status_tier: Option<CaseStatusTier>,
  pub panel_size_snapshot: Option<i32>,
  pub quorum_snapshot: Option<i32>,
  pub threshold_count_snapshot: Option<i32>,
  pub appeal_window_expires_at: Option<DateTime<Utc>>,
  /// v1-JM-d §9.3 InsertForm extension. Default `None`; submit_jury_vote
  /// writes via update().set(...) rather than re-inserting, so the
  /// InsertForm path is only exercised by case-open writers
  /// (create_report, admin_emergency_remove) which don't yet know the
  /// decision.
  pub winning_decision: Option<JuryDecision>,
  /// v1-SL-a additions. Both `Option<_>` so v0/earlier-v1 callers
  /// continue to compile via `..Default::default()`.
  pub grace_expires_at: Option<DateTime<Utc>>,
  pub liability_escape_reason: Option<Value>,
}
