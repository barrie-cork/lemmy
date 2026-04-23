use crate::newtypes::{JuryConstraintViolationLogId, ModerationCaseId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::jury_constraint_violation_log;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = jury_constraint_violation_log))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// v1-JM-a audit row written alongside the `jury_constraint_relaxed`
/// governance_log entry every time `select_eligible_jurors` relaxes a
/// diversity / recency / cluster constraint per PRD §5.3 (v1-JM-b
/// owns the call-site). Append-only; no `AsChangeset` derive — same
/// rationale as `governance_log`.
pub struct JuryConstraintViolationLog {
  pub id: JuryConstraintViolationLogId,
  pub case_id: ModerationCaseId,
  pub constraint_name: String,
  pub relaxation_reason: String,
  pub pool_size_at_relax: i32,
  pub panel_size_target: i32,
  pub relaxed_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = jury_constraint_violation_log))]
pub struct JuryConstraintViolationLogInsertForm {
  pub case_id: ModerationCaseId,
  pub constraint_name: String,
  pub relaxation_reason: String,
  pub pool_size_at_relax: i32,
  pub panel_size_target: i32,
}
