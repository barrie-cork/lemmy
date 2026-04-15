use crate::newtypes::{AppealId, ModerationCaseId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::{PersonId, enums::AppealStatus};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::appeal;
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
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = appeal))]
pub struct AppealInsertForm {
  pub case_id: ModerationCaseId,
  pub requester_id: PersonId,
  pub reason: String,
  pub status: AppealStatus,
}
