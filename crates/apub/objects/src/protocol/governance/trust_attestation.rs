use crate::objects::person::ApubPerson;
use activitypub_federation::fetch::object_id::ObjectId;
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::AttestationType;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

/// AP `type` discriminator for [`TrustAttestationProtocol`].
///
/// Single-variant enum (mirrors `NoteType` shape) so untagged enum dispatch
/// in `SharedInboxActivities` can distinguish trust attestations from
/// other Create wrappers.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, Default)]
pub enum TrustAttestationType {
  #[default]
  TrustAttestation,
}

/// AP wire-form of a trust attestation (e.g. TrustedReporter, JuryEligible).
///
/// Outbound-only in v0 — no v0 endpoint emits a trust attestation; the
/// shape exists to prove the protocol roundtrips. v1 wires this into the
/// endorsement-creation handler. See [docs/brehon-law-inspired-network/04-data-model-and-api.md §3]
/// for the inverse `FederationAttestation` row written on receive.
///
/// `subject` is a bare `Url` (matches `SanctionNoticeProtocol::target`)
/// for the same reason: v0 does not dereference the subject on receive.
///
/// `valid_until` mirrors the optional column on `federation_attestation`.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustAttestationProtocol {
  #[serde(rename = "type")]
  pub(crate) kind: TrustAttestationType,
  pub id: Url,
  pub actor: ObjectId<ApubPerson>,
  pub subject: Url,
  pub attestation_type: AttestationType,
  pub valid_until: Option<DateTime<Utc>>,
  pub published: DateTime<Utc>,
}

impl TrustAttestationProtocol {
  /// Cross-crate constructor for Phase 6 task 74's outbound publisher.
  ///
  /// Mirrors [`super::sanction_notice::SanctionNoticeProtocol::new`]:
  /// `kind` is `pub(crate)` so the activities crate cannot struct-literal
  /// it. Plumbed for v0 outbound but no v0 endpoint emits a trust
  /// attestation; v1 wires this from endorsement creation per plan
  /// task 74's `send_local_trust_attestation` shape.
  pub fn new(
    id: Url,
    actor: ObjectId<ApubPerson>,
    subject: Url,
    attestation_type: AttestationType,
    valid_until: Option<DateTime<Utc>>,
    published: DateTime<Utc>,
  ) -> Self {
    Self {
      kind: TrustAttestationType::default(),
      id,
      actor,
      subject,
      attestation_type,
      valid_until,
      published,
    }
  }
}
