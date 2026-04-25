use crate::newtypes::{JuryAssignmentId, ModerationCaseId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::{
  PersonId,
  enums::{JuryAssignmentRole, JuryAssignmentStatus},
};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::jury_assignment;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = jury_assignment))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// An assignment of a juror to a specific case.
pub struct JuryAssignment {
  pub id: JuryAssignmentId,
  pub case_id: ModerationCaseId,
  pub person_id: PersonId,
  pub status: JuryAssignmentStatus,
  pub selected_at: DateTime<Utc>,
  pub responded_at: Option<DateTime<Utc>>,
  pub submitted_at: Option<DateTime<Utc>>,
  /// v1-JM-a §8.2: JSONB payload listing which constraints were applied
  /// at panel-pick time (per PRD Watch 10: constraint names only, no
  /// person_id). Written by v1-JM-b admin_assign_jury; NULL pre-v1.
  pub selected_under_constraints: Option<Value>,
  /// v1-JM-a §8.2: distinguishes original-jury rows from appeal-jury
  /// rows on the same case. DB DEFAULT 'Original' covers every v0
  /// writer; v1-JM-d's `select_appeal_panel` writes 'Appeal' via a new
  /// call site. The InsertForm intentionally does NOT add a `role`
  /// field — DEFAULT covers the v0/v1-JM-b writer paths.
  pub role: JuryAssignmentRole,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = jury_assignment))]
pub struct JuryAssignmentInsertForm {
  pub case_id: ModerationCaseId,
  pub person_id: PersonId,
  pub status: JuryAssignmentStatus,
  /// v1-JM-a drift-fix: JSONB carrying the applied-constraints map written
  /// by v1-JM-b `admin_assign_jury::select_eligible_jurors`. The read-side
  /// `JuryAssignment` field already shipped in JM-a; the InsertForm was
  /// missed in JM-a Task 5 (commit `fdbe7f25c`). Default `None` so v0
  /// writers (Phase 4/5 paths, `decline_jury_assignment` replacement) that
  /// construct the form via `..Default::default()` remain source-compatible.
  /// Added as a preliminary fix in v1-JM-b per decision-queue #48.
  pub selected_under_constraints: Option<Value>,
}
