use crate::newtypes::{AppealId, ModerationCaseId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::appeal;
use lemmy_db_schema_file::{
  PersonId,
  enums::{AppealRequesterRole, AppealStatus},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = appeal))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// An appeal of a decided moderation case.
pub struct Appeal {
  pub id: AppealId,
  pub case_id: ModerationCaseId,
  pub requester_id: PersonId,
  pub reason: String,
  pub status: AppealStatus,
  pub created_at: DateTime<Utc>,
  pub decided_at: Option<DateTime<Utc>>,
  /// v1-JM-d §6.4: who filed the appeal. Defendant always; original
  /// reporter only when winning_decision was NoAction / AdvisoryLabel.
  pub requester_role: AppealRequesterRole,
  /// v1-JM-d §6.1: snapshot of the appeal panel's size at seating
  /// time. NULL until select_appeal_panel runs (auto on accept, or
  /// admin-triggered).
  pub panel_size_snapshot: Option<i32>,
  /// v1-JM-d §6.3: threshold-count for the appeal panel (cascade
  /// resolution of jury.threshold_fraction at the bumped severity per
  /// appeal.threshold_tier_bump). NULL until select_appeal_panel runs.
  pub threshold_count_snapshot: Option<i32>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = appeal))]
pub struct AppealInsertForm {
  pub case_id: ModerationCaseId,
  pub requester_id: PersonId,
  pub reason: String,
  pub status: AppealStatus,
  /// v1-JM-d §6.4: the requester_role discriminator. `Option<_>`
  /// because the DB DEFAULT 'Defendant' covers v0 callers if any
  /// remain post-rewrite — but request_appeal Task 3 always sets
  /// `Some(...)` based on the eligibility branch taken.
  pub requester_role: Option<AppealRequesterRole>,
  pub panel_size_snapshot: Option<i32>,
  pub threshold_count_snapshot: Option<i32>,
}
