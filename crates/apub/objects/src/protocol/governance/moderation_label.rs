use crate::objects::person::ApubPerson;
use activitypub_federation::fetch::object_id::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

/// AP `type` discriminator for [`ModerationLabelProtocol`].
///
/// Single-variant enum (mirrors `NoteType` shape) — kept distinct so
/// untagged dispatch in `SharedInboxActivities` does not collide with
/// `SanctionNotice` or `TrustAttestation` variants.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, Default)]
pub enum ModerationLabelType {
  #[default]
  ModerationLabel,
}

/// AP wire-form of a moderation label (e.g. "context-warning", "disputed").
///
/// Stub in v0 — outbound-only and **no v0 emit path**. Phase 6 ships the
/// shape so the wire format is fixed before v1 wires the emitter from
/// admin label-application handlers. Inbound is intentionally not wired
/// (see [docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md §3]
/// — labels are advisory in v0 and not surfaced via federation receive).
///
/// `target` is a bare `Url` (mirrors `SanctionNoticeProtocol::target`).
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationLabelProtocol {
  #[serde(rename = "type")]
  pub(crate) kind: ModerationLabelType,
  pub id: Url,
  pub actor: ObjectId<ApubPerson>,
  pub target: Url,
  /// Free-form short label (e.g. `"context-warning"`); v1 may switch to a
  /// constrained enum once the label vocabulary is fixed.
  pub label: String,
  /// Optional expanded summary; subject to [§4.2 redaction] before send.
  pub summary: Option<String>,
  pub published: DateTime<Utc>,
}
