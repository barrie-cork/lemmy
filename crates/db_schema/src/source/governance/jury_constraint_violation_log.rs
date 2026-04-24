use crate::newtypes::{JuryConstraintViolationLogId, ModerationCaseId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::JuryConstraintRelaxationReason;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::jury_constraint_violation_log;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
///
/// PR #92 cr-9: `reason_code` is a bounded-vocabulary enum (4 values
/// per PRD §5.3 R1/R2/R3 cascade + §8.3 AdminOverride) replacing the
/// originally-proposed free-text `relaxation_reason` column to close
/// the ADR-015 pseudonymisation gap. `relaxation_metadata` is
/// optional JSONB for bounded structured ancillary payload (e.g.
/// `{"dropped_constraint_name": "no_recent_juror_repeat",
///   "phase": "pool_build"}`); call sites MUST NEVER write free-text
/// user-supplied strings into this column.
pub struct JuryConstraintViolationLog {
  pub id: JuryConstraintViolationLogId,
  pub case_id: ModerationCaseId,
  pub constraint_name: String,
  pub reason_code: JuryConstraintRelaxationReason,
  pub relaxation_metadata: Option<Value>,
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
  pub reason_code: JuryConstraintRelaxationReason,
  pub relaxation_metadata: Option<Value>,
  pub pool_size_at_relax: i32,
  pub panel_size_target: i32,
}
