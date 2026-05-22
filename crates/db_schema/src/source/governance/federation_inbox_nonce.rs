#[cfg(feature = "full")]
use chrono::Duration;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::federation_inbox_nonce;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use diesel::{ExpressionMethods, QueryDsl};
#[cfg(feature = "full")]
use diesel_async::{AsyncPgConnection, RunQueryDsl};
#[cfg(feature = "full")]
use lemmy_utils::error::LemmyResult;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = federation_inbox_nonce))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct FederationInboxNonce {
  pub peer_instance: String,
  pub activity_id: String,
  pub seen_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = federation_inbox_nonce))]
pub struct FederationInboxNonceInsertForm {
  pub peer_instance: String,
  pub activity_id: String,
}

#[cfg(feature = "full")]
// TODO(v1-federation-inbound-b): wired by replay-cleanup cron
pub async fn delete_older_than(
  window_days: i64,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<usize> {
  if window_days <= 0 {
    return Ok(0);
  }
  let cutoff = Utc::now() - Duration::days(window_days);
  let count = diesel::delete(
    federation_inbox_nonce::table.filter(federation_inbox_nonce::seen_at.lt(cutoff)),
  )
  .execute(conn)
  .await?;
  Ok(count)
}
