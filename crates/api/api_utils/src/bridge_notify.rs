use crate::context::LemmyContext;
use lemmy_db_schema::source::governance::governance_messaging_config::GovernanceMessagingConfig;
use lemmy_db_views_private_message::PrivateMessageView;
use lemmy_utils::error::LemmyResult;

/// Default HTTP endpoint for the Matrix bridge notify path (plan §10.5).
/// No bridge runs in M1 — POST is fire-and-forget and transport errors are swallowed.
const BRIDGE_NOTIFY_URL: &str = "http://localhost:9009/brehon/notify";

/// Fire-and-forget notify to the Matrix bridge. Reads `messaging_enabled`;
/// false → no-op (clean v0 governance-only posture). True → POST to the bridge,
/// log+swallow any transport error. NEVER fails the PM path. ADR-012 (no new hook).
pub async fn notify_if_enabled(
  context: &LemmyContext,
  view: &PrivateMessageView,
) -> LemmyResult<()> {
  let pool = &mut context.pool();
  let enabled = match GovernanceMessagingConfig::read_current(pool, "instance", "messaging_enabled")
    .await?
  {
    Some(row) => row.value_bool.unwrap_or(false),
    None => false,
  };
  if !enabled {
    return Ok(());
  }
  // fire-and-forget POST; bridge being down must NOT break PM delivery (plan §10.5)
  #[derive(serde::Serialize)]
  struct Payload {
    private_message_id: i32,
    creator_id: i32,
    recipient_id: i32,
  }
  let payload = Payload {
    private_message_id: view.private_message.id.0,
    creator_id: view.creator.id.0,
    recipient_id: view.recipient.id.0,
  };
  if let Err(e) = context
    .client()
    .post(BRIDGE_NOTIFY_URL)
    .json(&payload)
    .send()
    .await
  {
    tracing::warn!("bridge notify failed (bridge may be down — non-fatal): {e}");
  }
  Ok(())
}
