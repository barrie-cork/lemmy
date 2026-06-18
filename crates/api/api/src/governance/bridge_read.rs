#[cfg(feature = "full")]
use {
  actix_web::{
    web::{Data, Json, Query},
    HttpRequest,
  },
  lemmy_api_utils::context::LemmyContext,
  lemmy_db_schema::source::governance::governance_messaging_config::GovernanceMessagingConfig,
  lemmy_db_schema_file::PersonId,
  lemmy_utils::error::LemmyResult,
  serde::{Deserialize, Serialize},
};
#[cfg(feature = "full")]
use super::{actor_pseudonym_helper, bridge_auth};

#[cfg(feature = "full")]
#[derive(Debug, Serialize)]
pub struct BridgeStatus {
  pub messaging_enabled: bool,
  pub oq009_reveal_threshold: i64,
  pub rtc_enabled: bool,
}

#[cfg(feature = "full")]
pub async fn get_bridge_messaging_status(
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<BridgeStatus>> {
  bridge_auth::verify_bridge_secret(&req)?;
  let pool = &mut context.pool();

  let messaging_enabled =
    GovernanceMessagingConfig::read_current(pool, "instance", "messaging_enabled")
      .await?
      .and_then(|r| r.value_bool)
      .unwrap_or(false);

  let oq009_reveal_threshold =
    GovernanceMessagingConfig::read_current(pool, "instance", "oq009_reveal_threshold")
      .await?
      .and_then(|r| r.value_int)
      .unwrap_or(1);

  let rtc_enabled =
    GovernanceMessagingConfig::read_current(pool, "instance", "rtc_enabled")
      .await?
      .and_then(|r| r.value_bool)
      .unwrap_or(false); // absent → false (clean-posture default)

  Ok(Json(BridgeStatus {
    messaging_enabled,
    oq009_reveal_threshold,
    rtc_enabled,
  }))
}

#[cfg(feature = "full")]
#[derive(Debug, Deserialize)]
pub struct BridgeActorPseudonymQuery {
  pub person_id: i32,
}

#[cfg(feature = "full")]
#[derive(Debug, Serialize)]
pub struct BridgeActorPseudonym {
  pub pseudonym: String,
}

#[cfg(feature = "full")]
pub async fn get_bridge_actor_pseudonym(
  req: HttpRequest,
  Query(params): Query<BridgeActorPseudonymQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<BridgeActorPseudonym>> {
  bridge_auth::verify_bridge_secret(&req)?;
  let pool = &mut context.pool();
  let pseudonym =
    actor_pseudonym_helper::get_or_create(pool, PersonId(params.person_id)).await?;
  Ok(Json(BridgeActorPseudonym { pseudonym }))
}
