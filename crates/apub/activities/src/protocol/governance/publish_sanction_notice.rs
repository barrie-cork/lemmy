use activitypub_federation::{
  fetch::object_id::ObjectId,
  kinds::activity::CreateType,
  protocol::helpers::deserialize_one_or_many,
};
use lemmy_apub_objects::objects::person::ApubPerson;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

/// Object-discriminator for `PublishSanctionNotice`.
///
/// `SharedInboxActivities` is `#[serde(untagged)]` and the three governance
/// Create wrappers all carry `kind: CreateType` ("Create"), which alone is
/// insufficient to disambiguate them from each other or from a vanilla
/// Lemmy/Mastodon Create activity. To preserve federation correctness,
/// each wrapper requires its `object.type` field to match a fixed string.
/// `serde_json::Value` for the rest of the object preserves the
/// pass-through semantics until merge-1b replaces this with the typed
/// object struct from Agent B.
///
/// TODO(merge-1b): collapse this discriminator into
/// `lemmy_apub_objects::protocol::governance::SanctionNoticeProtocol`
/// once Agent B lands the typed struct (which carries the same
/// `type = "SanctionNotice"` discriminator natively via its own
/// kind!-style enum).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SanctionNoticeObjectStub {
  #[serde(rename = "type")]
  pub(crate) kind: SanctionNoticeKind,
  /// All other fields of the underlying `SanctionNotice` object are
  /// captured untyped. Agent E (plan task 75) parses these fields
  /// strictly inside `receive_remote_sanction_notice`.
  #[serde(flatten)]
  pub(crate) rest: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum SanctionNoticeKind {
  SanctionNotice,
}

/// AP `Create` wrapper carrying a `SanctionNotice` object across the wire.
///
/// Sanction notices are instance-wide broadcasts (not community-scoped
/// `Announce` envelopes), so this wrapper goes straight into
/// `SharedInboxActivities` rather than `AnnouncableActivities`.
///
/// Discrimination: the `object` field is a `SanctionNoticeObjectStub`
/// which requires `object.type == "SanctionNotice"`. Other Create
/// activities (vanilla Lemmy comment Create, sibling
/// `PublishTrustAttestation`, etc.) have a different `object.type` and
/// won't match this variant in the `SharedInboxActivities` untagged
/// dispatch.
///
/// TODO(merge-1b): replace `object: SanctionNoticeObjectStub` with
/// `object: lemmy_apub_objects::protocol::governance::SanctionNoticeProtocol`
/// once Agent B lands the typed object protocol struct.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishSanctionNoticeProtocol {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  #[serde(deserialize_with = "deserialize_one_or_many", default)]
  pub(crate) cc: Vec<Url>,
  pub(crate) object: SanctionNoticeObjectStub,
  #[serde(rename = "type")]
  pub(crate) kind: CreateType,
  pub(crate) id: Url,
}

/// Bare-name alias matching Lemmy convention (`Report`, `BlockUser`,
/// etc.). Used by `impl Activity for PublishSanctionNotice` and by
/// `SharedInboxActivities::PublishSanctionNotice(_)`.
pub type PublishSanctionNotice = PublishSanctionNoticeProtocol;
