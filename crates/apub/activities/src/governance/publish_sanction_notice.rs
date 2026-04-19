use crate::protocol::governance::publish_sanction_notice::PublishSanctionNotice;
use activitypub_federation::{config::Data, traits::Activity};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::utils::functions::verify_is_public;
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

#[async_trait::async_trait]
impl Activity for PublishSanctionNotice {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    // Sanction notices are instance-wide broadcasts (ADR-014). They must
    // be addressed to the AP `Public` collection — either in `to` or `cc`.
    // Per-object schema validation (action/scope/target shape) happens at
    // the `serde::Deserialize` step before `verify` runs, so unrecognised
    // enum variants are rejected before we get here.
    verify_is_public(&self.to, &self.cc)?;
    Ok(())
  }

  async fn receive(self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    // STUB ONLY — Agent E (plan task 75) wires this to
    // `crates/apub/apub/src/governance/inbox.rs::receive_remote_sanction_notice`.
    // The split is intentional: it lets Layer 3 Agents D and E proceed
    // in parallel without a file conflict on this method body.
    //
    // TODO(task75): wire receive_remote_sanction_notice
    Ok(())
  }
}
