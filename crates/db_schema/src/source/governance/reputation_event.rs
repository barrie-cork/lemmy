use crate::newtypes::{CommunityId, ModerationCaseId, ReputationEventId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::reputation_event;
use lemmy_db_schema_file::{
  PersonId,
  enums::{ReputationDimension, ReputationEventSourceType}, // v1-RT-r1: add ReputationEventSourceType
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = reputation_event))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A single reputation delta event targeting a person on a given dimension.
pub struct ReputationEvent {
  pub id: ReputationEventId,
  pub person_id: PersonId,
  pub community_id: Option<CommunityId>,
  pub dimension: ReputationDimension,
  pub delta: i32,
  pub source_case_id: Option<ModerationCaseId>,
  pub source_report_id: Option<i32>,
  pub reason: String,
  pub created_at: DateTime<Utc>,
  pub expires_at: Option<DateTime<Utc>>,
  /// v1-RT-r1 section 5.3 source 1: idempotency key for cron events.
  pub dedupe_key: Option<String>,
  /// v1-RT-r1 section 5.3 + 7: per-event source classification.
  pub source_event_type: ReputationEventSourceType,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = reputation_event))]
pub struct ReputationEventInsertForm {
  pub person_id: PersonId,
  pub community_id: Option<CommunityId>,
  pub dimension: ReputationDimension,
  pub delta: i32,
  pub source_case_id: Option<ModerationCaseId>,
  pub source_report_id: Option<i32>,
  pub reason: String,
  pub expires_at: Option<DateTime<Utc>>,
  /// v1-RT-r1: optional. Column DEFAULT covers callers that omit.
  pub dedupe_key: Option<String>,
  pub source_event_type: Option<ReputationEventSourceType>,
}
