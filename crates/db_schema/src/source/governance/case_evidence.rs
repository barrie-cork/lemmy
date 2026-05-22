use crate::newtypes::{CaseEvidenceId, ModerationCaseId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::case_evidence;
use lemmy_db_schema_file::{PersonId, enums::EvidenceVisibility};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = case_evidence))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A piece of evidence attached to a moderation case.
pub struct CaseEvidence {
  pub id: CaseEvidenceId,
  pub case_id: ModerationCaseId,
  pub uploader_id: PersonId,
  pub storage_key: String,
  pub sha256: String,
  pub mime_type: String,
  pub visibility: EvidenceVisibility,
  pub created_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = case_evidence))]
pub struct CaseEvidenceInsertForm {
  pub case_id: ModerationCaseId,
  pub uploader_id: PersonId,
  pub storage_key: String,
  pub sha256: String,
  pub mime_type: String,
  pub visibility: EvidenceVisibility,
}
