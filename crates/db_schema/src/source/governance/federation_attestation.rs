use crate::newtypes::FederationAttestationId;
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::{
  AttestationType, FederationInboxAdminAction, FederationPeerTrust,
};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::federation_attestation;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = federation_attestation))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// An attestation made by an actor about a subject, signed and federated
/// outbound. Used for trust attestations (TrustedReporter, JuryEligible) and
/// outbound sanction-notice receipts. v0 only writes — inbound attestations
/// are stored via the AP receive path in Phase 6.
pub struct FederationAttestation {
  pub id: FederationAttestationId,
  pub actor_url: String,
  pub subject_url: String,
  pub attestation_type: AttestationType,
  pub valid_until: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
  pub signature: String,
  // v1-federation-inbound-a additions:
  pub source_instance: Option<String>,
  pub received_at: Option<DateTime<Utc>>,
  pub peer_trust_level_at_receipt: Option<FederationPeerTrust>,
  pub admin_reviewed_at: Option<DateTime<Utc>>,
  pub admin_action: FederationInboxAdminAction,
  pub dismissal_rationale: Option<String>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = federation_attestation))]
pub struct FederationAttestationInsertForm {
  pub actor_url: String,
  pub subject_url: String,
  pub attestation_type: AttestationType,
  pub valid_until: Option<DateTime<Utc>>,
  pub signature: String,
  // v1-federation-inbound-a additions (all Option<_> to preserve ..Default::default() caller compat):
  pub source_instance: Option<String>,
  pub received_at: Option<DateTime<Utc>>,
  pub peer_trust_level_at_receipt: Option<FederationPeerTrust>,
  pub admin_reviewed_at: Option<DateTime<Utc>>,
  pub admin_action: Option<FederationInboxAdminAction>,
  pub dismissal_rationale: Option<String>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = federation_attestation))]
pub struct FederationAttestationUpdateForm {
  pub source_instance: Option<String>,
  pub received_at: Option<DateTime<Utc>>,
  pub peer_trust_level_at_receipt: Option<FederationPeerTrust>,
  pub admin_reviewed_at: Option<DateTime<Utc>>,
  pub admin_action: Option<FederationInboxAdminAction>,
  pub dismissal_rationale: Option<String>,
}
