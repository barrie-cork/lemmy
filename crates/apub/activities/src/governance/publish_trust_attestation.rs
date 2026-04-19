use crate::protocol::governance::publish_trust_attestation::PublishTrustAttestation;
use activitypub_federation::{config::Data, traits::Activity};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::utils::functions::verify_is_public;
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

#[async_trait::async_trait]
impl Activity for PublishTrustAttestation {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    // Trust attestations are instance-wide broadcasts (ADR-014).
    verify_is_public(&self.to, &self.cc)?;
    Ok(())
  }

  async fn receive(self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    // STUB ONLY — Agent E (plan task 75) wires this to
    // `crates/apub/apub/src/governance/inbox.rs::receive_remote_trust_attestation`.
    //
    // TODO(task75): wire receive_remote_trust_attestation
    Ok(())
  }
}
