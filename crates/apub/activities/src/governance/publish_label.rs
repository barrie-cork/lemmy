use crate::protocol::governance::publish_label::PublishLabel;
use activitypub_federation::{config::Data, traits::Activity};
use lemmy_api_utils::context::LemmyContext;
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

#[async_trait::async_trait]
impl Activity for PublishLabel {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    // Stub — labels are not wired from any v0 endpoint. Reserving the
    // variant in `SharedInboxActivities` lets v1 add publishing without
    // churning federation handshakes.
    Ok(())
  }

  async fn receive(self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    // Stub — see `verify` above. Acceptable per plan §task 73 for v0.
    Ok(())
  }
}
