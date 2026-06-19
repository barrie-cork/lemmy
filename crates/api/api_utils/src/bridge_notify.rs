use crate::context::LemmyContext;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_api_common::governance::{BridgeNotifyPayload, CaseTransitionEvent, PrivateMessagePayload};
use lemmy_db_schema::{
  newtypes::ModerationCaseId,
  source::governance::{
    governance_messaging_config::GovernanceMessagingConfig,
    moderation_case::ModerationCase,
  },
};
use lemmy_db_schema_file::enums::{CaseStatus, JuryAssignmentRole};
use lemmy_db_views_private_message::PrivateMessageView;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::LemmyResult;

/// Fallback HTTP endpoint for the Matrix bridge room-event path (plan §10.5).
/// Overridden by the `BRIDGE_ROOM_EVENT_URL` env var when set (the deployed
/// bridge listens on `/brehon/room-event`, not the historical `/brehon/notify`
/// placeholder). POST is fire-and-forget; transport errors are swallowed.
const BRIDGE_NOTIFY_URL_FALLBACK: &str = "http://localhost:9009/brehon/room-event";

/// Resolve the bridge room-event URL: `BRIDGE_ROOM_EVENT_URL` env, else the
/// fallback const. The bridge authenticates this path with
/// `BRIDGE_CALLBACK_SECRET` (Brehon→bridge), so callers attach that as a Bearer.
fn bridge_room_event_url() -> String {
  std::env::var("BRIDGE_ROOM_EVENT_URL")
    .ok()
    .filter(|s| !s.trim().is_empty())
    .unwrap_or_else(|| BRIDGE_NOTIFY_URL_FALLBACK.to_string())
}

/// The Brehon→bridge shared secret (matches the bridge's `BRIDGE_CALLBACK_SECRET`).
/// Empty when unset — the bridge will then 401 the POST (logged + swallowed).
fn bridge_callback_secret() -> String {
  std::env::var("BRIDGE_CALLBACK_SECRET").unwrap_or_default()
}

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
    .post(bridge_room_event_url())
    .header("Authorization", format!("Bearer {}", bridge_callback_secret()))
    .json(&payload)
    .send()
    .await
  {
    tracing::warn!("bridge notify failed (bridge may be down — non-fatal): {e}");
  }
  Ok(())
}

/// Fetch pseudonyms for jurors assigned to a case, via
/// `jury_assignment INNER JOIN actor_pseudonym ON person_id`. Returns only
/// `actor_pseudonym.pseudonym` values (ADR-015 — no real identities).
///
/// `role` scopes the fetch to one assignment round: `Original` for the
/// jury-selection room, `Appeal` for the appeal room. The original-jury and
/// appeal panels coexist as separate `jury_assignment` rows on the same case
/// (the appeal selector excludes the original jurors), so the bridge's
/// per-room provisioner must receive only the round it is about to provision —
/// otherwise an appeal room would be seeded with the original jurors' puppets.
async fn fetch_juror_pseudonyms(
  pool: &mut DbPool<'_>,
  case_id: ModerationCaseId,
  role: JuryAssignmentRole,
) -> LemmyResult<Vec<String>> {
  use lemmy_db_schema_file::schema::{actor_pseudonym, jury_assignment};
  let conn = &mut get_conn(pool).await?;
  let rows = jury_assignment::table
    .inner_join(actor_pseudonym::table.on(actor_pseudonym::person_id.eq(jury_assignment::person_id)))
    .filter(jury_assignment::case_id.eq(case_id))
    .filter(jury_assignment::role.eq(role))
    .select(actor_pseudonym::pseudonym)
    .load::<String>(conn)
    .await?;
  Ok(rows)
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
  // The bridge's room provisioner enforces ADR-015 always_pseudonym by skipping
  // any room whose `juror_pseudonyms` is empty. Each room-provisioning transition
  // must therefore carry the pseudonyms for its OWN assignment round: the
  // jury-selection room gets the `Original` panel; the appeal room gets the
  // `Appeal` panel (seated inside the request_appeal txn, committed before this
  // hook fires). Non-room transitions carry no jurors.
  let juror_pseudonyms = match new_status {
    CaseStatus::JurySelection => {
      fetch_juror_pseudonyms(pool, case.id, JuryAssignmentRole::Original).await?
    }
    CaseStatus::Appealed => {
      fetch_juror_pseudonyms(pool, case.id, JuryAssignmentRole::Appeal).await?
    }
    _ => Vec::new(),
  };
  let payload = BridgeNotifyPayload::CaseTransition(CaseTransitionEvent {
    case_id: case.id.0,
    old_status,
    new_status,
    community_id: case.community_id.map(|c| c.0),
    target_type: case.target_type,
    juror_pseudonyms,
    chair_pseudonym: None,
  });
  if let Err(e) = context
    .client()
    .post(bridge_room_event_url())
    .header("Authorization", format!("Bearer {}", bridge_callback_secret()))
    .json(&payload)
    .send()
    .await
  {
    tracing::warn!("bridge notify (case transition) failed (bridge may be down — non-fatal): {e}");
  }
  Ok(())
}
