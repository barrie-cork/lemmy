use crate::newtypes::{ModerationCaseId, RemoteSanctionNoticeId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::{SanctionAction, SanctionScope};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::remote_sanction_notice;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = remote_sanction_notice))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// An advisory record of an inbound sanction notice received from a remote
/// instance. Per [99 ADR-006] these are NEVER auto-applied — `local_case_id`
/// stays NULL until an admin opens a corresponding local case. The
/// `received_at` column is a fork-local extension (DQ-6.1) used by
/// admin-review queries; the design doc lists `published_at` only.
pub struct RemoteSanctionNotice {
  pub id: RemoteSanctionNoticeId,
  pub source_instance: String,
  pub target_url: String,
  pub action: SanctionAction,
  pub scope: SanctionScope,
  pub summary: String,
  pub published_at: DateTime<Utc>,
  pub signature: String,
  pub local_case_id: Option<ModerationCaseId>,
  pub received_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = remote_sanction_notice))]
pub struct RemoteSanctionNoticeInsertForm {
  pub source_instance: String,
  pub target_url: String,
  pub action: SanctionAction,
  pub scope: SanctionScope,
  pub summary: String,
  pub published_at: DateTime<Utc>,
  pub signature: String,
  pub local_case_id: Option<ModerationCaseId>,
}
