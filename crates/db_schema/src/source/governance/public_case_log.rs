use crate::newtypes::{CommunityId, ModerationCaseId, PublicCaseLogId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::public_case_log;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = public_case_log))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A human-readable public summary of a moderation case outcome.
pub struct PublicCaseLog {
  pub id: PublicCaseLogId,
  pub case_id: ModerationCaseId,
  pub community_id: Option<CommunityId>,
  pub summary: String,
  pub rationale_redacted: Option<String>,
  pub published_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = public_case_log))]
pub struct PublicCaseLogInsertForm {
  pub case_id: ModerationCaseId,
  pub community_id: Option<CommunityId>,
  pub summary: String,
  pub rationale_redacted: Option<String>,
}
