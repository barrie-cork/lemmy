use activitypub_federation::{
  fetch::object_id::ObjectId,
  kinds::activity::CreateType,
  protocol::helpers::deserialize_one_or_many,
};
use lemmy_apub_objects::objects::person::ApubPerson;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

/// Object-discriminator for `PublishTrustAttestation`. See
/// `publish_sanction_notice.rs` for the rationale.
///
/// TODO(merge-1b): collapse into
/// `lemmy_apub_objects::protocol::governance::TrustAttestationProtocol`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustAttestationObjectStub {
  #[serde(rename = "type")]
  pub(crate) kind: TrustAttestationKind,
  #[serde(flatten)]
  pub(crate) rest: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum TrustAttestationKind {
  TrustAttestation,
}

/// AP `Create` wrapper carrying a `TrustAttestation` object across the wire.
///
/// Trust attestations are instance-wide broadcasts (not community-scoped
/// `Announce` envelopes). Plumbed for v0 but no v0 endpoint emits one;
/// v1 wires this from endorsement creation per plan task 74.
///
/// TODO(merge-1b): replace `object: TrustAttestationObjectStub` with
/// `object: lemmy_apub_objects::protocol::governance::TrustAttestationProtocol`
/// once Agent B lands the typed object protocol struct.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishTrustAttestationProtocol {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  #[serde(deserialize_with = "deserialize_one_or_many", default)]
  pub(crate) cc: Vec<Url>,
  pub(crate) object: TrustAttestationObjectStub,
  #[serde(rename = "type")]
  pub(crate) kind: CreateType,
  pub(crate) id: Url,
}

/// Bare-name alias matching Lemmy convention.
pub type PublishTrustAttestation = PublishTrustAttestationProtocol;
