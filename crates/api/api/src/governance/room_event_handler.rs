#[cfg(feature = "full")]
use {
  actix_web::{
    web::{Data, Json},
    HttpRequest,
  },
  lemmy_api_utils::context::LemmyContext,
  lemmy_utils::error::LemmyResult,
  serde::Deserialize,
};
#[cfg(feature = "full")]
use super::{
  bridge_auth,
  governance_log::{append_room_event, RoomEventPayload},
};

#[cfg(feature = "full")]
#[derive(Debug, Deserialize)]
pub struct RoomEventRequest {
  pub entry_kind: String,
  pub payload: RoomEventPayload,
  pub actor_pseudonym: Option<String>,
}

#[cfg(feature = "full")]
pub async fn handle_room_event(
  req: HttpRequest,
  body: Json<RoomEventRequest>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<serde_json::Value>> {
  bridge_auth::verify_bridge_secret(&req)?;
  let pool = &mut context.pool();
  let req_body = body.into_inner();
  append_room_event(pool, &req_body.entry_kind, req_body.payload, req_body.actor_pseudonym).await?;
  Ok(Json(serde_json::json!({})))
}
