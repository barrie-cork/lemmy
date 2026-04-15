use crate::newtypes::{JuryAssignmentId, ModerationCaseId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::{PersonId, enums::JuryAssignmentStatus};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::jury_assignment;
use serde::{Deserialize, Serialize};
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
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = jury_assignment))]
pub struct JuryAssignmentInsertForm {
  pub case_id: ModerationCaseId,
  pub person_id: PersonId,
  pub status: JuryAssignmentStatus,
}
