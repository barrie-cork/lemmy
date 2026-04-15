use crate::newtypes::{CommunityId, ReputationSnapshotId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::PersonId;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::reputation_snapshot;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = reputation_snapshot))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A point-in-time aggregate of a person's reputation in a community context.
pub struct ReputationSnapshot {
  pub id: ReputationSnapshotId,
  pub person_id: PersonId,
  pub community_id: Option<CommunityId>,
  pub reporting_accuracy: i32,
  pub jury_reliability: i32,
  pub participation_consistency: i32,
  pub endorsement_strength: i32,
  pub jury_eligible: bool,
  pub trusted_reporter: bool,
  pub calculated_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = reputation_snapshot))]
pub struct ReputationSnapshotInsertForm {
  pub person_id: PersonId,
  pub community_id: Option<CommunityId>,
  pub reporting_accuracy: i32,
  pub jury_reliability: i32,
  pub participation_consistency: i32,
  pub endorsement_strength: i32,
  pub jury_eligible: bool,
  pub trusted_reporter: bool,
}
