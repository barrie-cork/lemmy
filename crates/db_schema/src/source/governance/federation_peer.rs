use chrono::{DateTime, Utc};
use lemmy_db_schema_file::{InstanceId, enums::FederationPeerTrust};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::{federation_peer, instance};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, SelectableHelper};
#[cfg(feature = "full")]
use diesel_async::{AsyncPgConnection, RunQueryDsl};
#[cfg(feature = "full")]
use lemmy_utils::error::LemmyResult;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = federation_peer))]
#[cfg_attr(feature = "full", diesel(primary_key(instance_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Per-peer trust-state overlay on Lemmy's `instance` table.
pub struct FederationPeer {
  pub instance_id: InstanceId,
  pub trust_level: FederationPeerTrust,
  pub added_at: DateTime<Utc>,
  pub added_by_actor: Option<String>,
  pub notes: Value,
  pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = federation_peer))]
pub struct FederationPeerInsertForm {
  pub instance_id: InstanceId,
  pub trust_level: Option<FederationPeerTrust>,
  pub added_by_actor: Option<String>,
  pub notes: Option<Value>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = federation_peer))]
pub struct FederationPeerUpdateForm {
  pub trust_level: Option<FederationPeerTrust>,
  pub added_by_actor: Option<Option<String>>,
  pub notes: Option<Value>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(feature = "full")]
pub async fn federation_inbox_check_peer_trust(
  peer_domain: &str,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<FederationPeerTrust> {
  let result: Option<FederationPeerTrust> = federation_peer::table
    .inner_join(instance::table.on(federation_peer::instance_id.eq(instance::id)))
    .filter(instance::domain.eq(peer_domain))
    .select(federation_peer::trust_level)
    .first(conn)
    .await
    .optional()?;
  Ok(result.unwrap_or(FederationPeerTrust::Unknown))
}

#[cfg(feature = "full")]
pub async fn federation_peer_upsert_trust(
  instance_id: InstanceId,
  trust_level: FederationPeerTrust,
  added_by_actor: Option<&str>,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<FederationPeer> {
  let form = FederationPeerInsertForm {
    instance_id,
    trust_level: Some(trust_level),
    added_by_actor: added_by_actor.map(String::from),
    notes: None,
  };
  let row = diesel::insert_into(federation_peer::table)
    .values(&form)
    .on_conflict(federation_peer::instance_id)
    .do_update()
    .set((
      federation_peer::trust_level.eq(trust_level),
      federation_peer::updated_at.eq(diesel::dsl::now),
    ))
    .returning(FederationPeer::as_returning())
    .get_result(conn)
    .await?;
  Ok(row)
}
