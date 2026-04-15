use crate::newtypes::{CommunityId, EndorsementId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::PersonId;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::endorsement;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = endorsement))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A peer-to-peer endorsement between two persons within a community context.
pub struct Endorsement {
  pub id: EndorsementId,
  pub from_person_id: PersonId,
  pub to_person_id: PersonId,
  pub community_id: Option<CommunityId>,
  pub created_at: DateTime<Utc>,
  pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = endorsement))]
pub struct EndorsementInsertForm {
  pub from_person_id: PersonId,
  pub to_person_id: PersonId,
  pub community_id: Option<CommunityId>,
}
