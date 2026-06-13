use crate::newtypes::ActorAppLinkId;
use crate::newtypes::ActorPseudonymId;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::actor_app_link;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = actor_app_link))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A portable Brehon actor ↔ external-app-user mapping (B-actor, ADR-016).
/// `brehon_actor_id` references actor_pseudonym(id); never stores raw person_id (ADR-015).
pub struct ActorAppLink {
  pub id: ActorAppLinkId,
  pub brehon_actor_id: ActorPseudonymId,
  pub app_id: String,
  pub app_local_id: String,
  pub created_at: DateTime<Utc>,
  pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = actor_app_link))]
pub struct ActorAppLinkInsertForm {
  pub brehon_actor_id: ActorPseudonymId,
  pub app_id: String,
  pub app_local_id: String,
}
