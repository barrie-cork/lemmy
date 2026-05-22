use crate::protocol::governance::publish_label::PublishLabel;
use activitypub_federation::{config::Data, traits::Activity};
use lemmy_api_utils::context::LemmyContext;
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
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

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    crate::governance::inbox::wrap_governance_inbound(self, context, |a, c| async move {
      crate::governance::inbox::receive_remote_moderation_label(a, c).await
    })
    .await
  }
}

#[async_trait::async_trait]
impl crate::governance::inbox::GovernanceInboundActivity for PublishLabel {
  fn activity_id(&self) -> &Url {
    &self.id
  }
  fn actor_domain(&self) -> LemmyResult<String> {
    self
      .actor
      .inner()
      .domain()
      .map(str::to_string)
      .ok_or_else(|| {
        LemmyErrorType::Unknown(format!(
          "PublishLabel actor {} has no domain",
          self.actor.inner()
        ))
        .into()
      })
  }
  fn payload_size_bytes(&self) -> LemmyResult<usize> {
    Ok(serde_json::to_vec(self)?.len())
  }
  fn payload_size_cap_key(&self) -> &'static str {
    "federation.inbound.max_payload_bytes_moderation_label"
  }
  // check_per_actor_rate_limit uses the trait default no-op.
}
