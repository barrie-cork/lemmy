use crate::protocol::governance::trust_attestation::TrustAttestationProtocol;
use activitypub_federation::{
  config::Data, protocol::verification::verify_domains_match, traits::Object,
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::governance::federation_attestation::FederationAttestation;
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use std::ops::Deref;
use url::Url;

/// AP `Object` newtype for trust attestations.
///
/// Carries the local `FederationAttestation` row alongside its AP `id`
/// URL. The schema column `federation_attestation` does not include
/// `ap_id` (the row stores `actor_url` + `subject_url` directly), so the
/// activity-id must be carried out-of-band.
///
/// v0 outbound only: no v0 endpoint emits a trust attestation; the type
/// exists to lock the wire shape per
/// [docs/brehon-law-inspired-network/04-data-model-and-api.md §3].
/// Phase 6 task 74 ships `send_local_trust_attestation` plumbed but
/// uncalled; v1 wires it into endorsement-creation.
#[derive(Clone, Debug)]
pub struct ApubTrustAttestation {
  pub row: FederationAttestation,
  pub ap_id: Url,
}

impl Deref for ApubTrustAttestation {
  type Target = FederationAttestation;
  fn deref(&self) -> &Self::Target {
    &self.row
  }
}

#[async_trait::async_trait]
impl Object for ApubTrustAttestation {
  type DataType = LemmyContext;
  type Kind = TrustAttestationProtocol;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.ap_id
  }

  async fn read_from_id(
    _object_id: Url,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<Option<Self>> {
    // v0: TrustAttestation is outbound-only; not URL-addressable on receive.
    Ok(None)
  }

  async fn delete(&self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    // v0: federation_attestation rows are immutable. v1 will add Undo
    // activities for revoked attestations (see [§4.4 OQ-014]).
    Err(LemmyErrorType::NotFound.into())
  }

  fn is_deleted(&self) -> bool {
    false
  }

  async fn into_json(
    self,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<TrustAttestationProtocol> {
    // v0: not re-served. Outbound construction lives in Phase 6 task 74.
    Err(LemmyErrorType::NotFound.into())
  }

  async fn verify(
    object: &TrustAttestationProtocol,
    expected_domain: &Url,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    verify_domains_match(&object.id, expected_domain)?;
    Ok(())
  }

  async fn from_json(
    _object: TrustAttestationProtocol,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<Self> {
    // v0 inbound receive path is the free function in
    // crates/apub/apub/src/governance/inbox.rs (Phase 6 task 75).
    Err(LemmyErrorType::NotFound.into())
  }
}
