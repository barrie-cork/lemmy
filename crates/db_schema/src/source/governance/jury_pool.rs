use crate::newtypes::{CommunityId, JuryPoolId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::PersonId;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::jury_pool;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = jury_pool))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A person enrolled in a community's jury pool.
pub struct JuryPool {
  pub id: JuryPoolId,
  pub community_id: Option<CommunityId>,
  pub person_id: PersonId,
  pub eligible_from: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = jury_pool))]
pub struct JuryPoolInsertForm {
  pub community_id: Option<CommunityId>,
  pub person_id: PersonId,
}
