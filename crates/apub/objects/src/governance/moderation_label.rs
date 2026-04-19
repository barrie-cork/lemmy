use crate::protocol::governance::moderation_label::ModerationLabelProtocol;
use activitypub_federation::{
  config::Data,
  protocol::verification::verify_domains_match,
  traits::Object,
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use url::Url;

/// AP `Object` newtype for moderation labels.
///
/// **Stub — outbound-only in v0, with no v0 emit path.** Phase 6 ships
/// the Object trait impl + protocol struct (per
/// [docs/brehon-law-inspired-network/04-data-model-and-api.md §11]) so
/// the wire shape is fixed; v1 adds the admin label-application emitter.
///
/// There is no v0 DB table for moderation labels (the design doc
/// reserves `moderation_label` for v1+). This newtype therefore wraps
/// only the AP `id` URL — there is no row to deref to.
#[derive(Clone, Debug)]
pub struct ApubModerationLabel {
  pub ap_id: Url,
}

#[async_trait::async_trait]
impl Object for ApubModerationLabel {
  type DataType = LemmyContext;
  type Kind = ModerationLabelProtocol;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.ap_id
  }

  async fn read_from_id(
    _object_id: Url,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<Option<Self>> {
    // v0: ModerationLabel has no inbound receive wiring (see crate-level
    // doc in mod.rs). Apf's fetch path treats every id as "not local".
    Ok(None)
  }

  async fn delete(&self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    // v0: stub. v1 will add Undo for retracted labels.
    Err(LemmyErrorType::NotFound.into())
  }

  fn is_deleted(&self) -> bool {
    false
  }

  async fn into_json(
    self,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<ModerationLabelProtocol> {
    // v0: stub. No outbound emit path in v0.
    Err(LemmyErrorType::NotFound.into())
  }

  async fn verify(
    object: &ModerationLabelProtocol,
    expected_domain: &Url,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    verify_domains_match(&object.id, expected_domain)?;
    Ok(())
  }

  async fn from_json(
    _object: ModerationLabelProtocol,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<Self> {
    // v0: stub. No inbound receive path exists; PublishLabel::receive
    // (Phase 6 task 73 / Agent C) returns Ok(()) without a body.
    Err(LemmyErrorType::NotFound.into())
  }
}
