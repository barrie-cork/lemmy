use crate::context::LemmyContext;
use lemmy_api_common::governance::{BridgeNotifyPayload, CaseTransitionEvent, PrivateMessagePayload};
use lemmy_db_schema::source::governance::{
  governance_messaging_config::GovernanceMessagingConfig,
  moderation_case::ModerationCase,
};
use lemmy_db_schema_file::enums::CaseStatus;
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
  let payload = BridgeNotifyPayload::PrivateMessage(PrivateMessagePayload {
    private_message_id: view.private_message.id.0,
    creator_id: view.creator.id.0,
    recipient_id: view.recipient.id.0,
  });
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

/// Fire-and-forget notify to the Matrix bridge when a moderation case transitions status.
/// Reads `messaging_enabled`; false → no-op. True → POST CaseTransition event to bridge.
/// NEVER fails the governance path — transport errors are swallowed (plan §10.5, ADR-012).
pub async fn governance_case_after_transition(
  context: &LemmyContext,
  case: &ModerationCase,
  old_status: Option<CaseStatus>,
  new_status: CaseStatus,
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
  let payload = BridgeNotifyPayload::CaseTransition(CaseTransitionEvent {
    case_id: case.id.0,
    old_status,
    new_status,
    community_id: case.community_id.map(|c| c.0),
    target_type: format!("{:?}", case.target_type),
  });
  if let Err(e) = context
    .client()
    .post(BRIDGE_NOTIFY_URL)
    .json(&payload)
    .send()
    .await
  {
    tracing::warn!("bridge notify (case transition) failed (bridge may be down — non-fatal): {e}");
  }
  Ok(())
}
