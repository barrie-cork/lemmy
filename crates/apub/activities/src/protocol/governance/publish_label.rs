use activitypub_federation::{
  fetch::object_id::ObjectId, kinds::activity::CreateType,
  protocol::helpers::deserialize_one_or_many,
};
use lemmy_apub_objects::objects::person::ApubPerson;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

/// Object-discriminator for `PublishLabel`. See
/// `publish_sanction_notice.rs` for the rationale.
///
/// v2-cleanup: typed struct `ModerationLabelProtocol` already exists in
/// `lemmy_apub_objects::protocol::governance` — collapse this stub then.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationLabelObjectStub {
  #[serde(rename = "type")]
  pub(crate) kind: ModerationLabelKind,
  #[serde(flatten)]
  pub(crate) rest: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ModerationLabelKind {
  ModerationLabel,
}

/// AP `Create` wrapper carrying a `ModerationLabel` object across the wire.
///
/// Stub for v0 — no handler emits or receives labels; the type and
/// trait impl exist so downstream phases can wire it without churning
/// the `SharedInboxActivities` registration.
///
/// v2-cleanup: replace `object: ModerationLabelObjectStub` with
/// `lemmy_apub_objects::protocol::governance::ModerationLabelProtocol`.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishLabelProtocol {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  #[serde(deserialize_with = "deserialize_one_or_many", default)]
  pub(crate) cc: Vec<Url>,
  pub(crate) object: ModerationLabelObjectStub,
  #[serde(rename = "type")]
  pub(crate) kind: CreateType,
  pub(crate) id: Url,
}

/// Bare-name alias matching Lemmy convention.
pub type PublishLabel = PublishLabelProtocol;
