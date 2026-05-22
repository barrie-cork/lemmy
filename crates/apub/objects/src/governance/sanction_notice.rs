use crate::protocol::governance::sanction_notice::SanctionNoticeProtocol;
use activitypub_federation::{
  config::Data, protocol::verification::verify_domains_match, traits::Object,
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::governance::remote_sanction_notice::RemoteSanctionNotice;
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use std::ops::Deref;
use url::Url;

/// AP `Object` newtype for inbound sanction notices.
///
/// Carries the advisory `RemoteSanctionNotice` row alongside the AP `id`
/// URL of the originating activity object. The URL is required by
/// `Object::id` (apf's `http_response` default impl reads it) but the
/// stored row's schema does not include `ap_id` (advisory rows are not
/// URL-addressable in v0; see ADR-006), so it must be carried out-of-band.
///
/// v0 only uses this on the inbound receive path; outbound construction
/// is done directly from the local `Sanction` row by Phase 6 task 74
/// (Agent D), not through `Object::into_json` on this type.
///
/// Per ADR-006 the row's `local_case_id` is always NULL on insert — admin
/// review opens any local case manually. See
/// [docs/brehon-law-inspired-network/04-data-model-and-api.md §11].
#[derive(Clone, Debug)]
pub struct ApubSanctionNotice {
  pub row: RemoteSanctionNotice,
  pub ap_id: Url,
}

impl Deref for ApubSanctionNotice {
  type Target = RemoteSanctionNotice;
  fn deref(&self) -> &Self::Target {
    &self.row
  }
}

#[async_trait::async_trait]
impl Object for ApubSanctionNotice {
  type DataType = LemmyContext;
  type Kind = SanctionNoticeProtocol;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.ap_id
  }

  async fn read_from_id(
    _object_id: Url,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<Option<Self>> {
    // v0: SanctionNotice is outbound-only as a federated payload; the
    // inbound advisory row is not URL-addressable. Returning None lets
    // apf's fetch path treat the object as "not present locally" without
    // any DB hit.
    Ok(None)
  }

  async fn delete(&self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    // v0: advisory rows are immutable audit records. v1 admin-review
    // tooling may add a redact path; v0 has none.
    Err(LemmyErrorType::NotFound.into())
  }

  fn is_deleted(&self) -> bool {
    false
  }

  async fn into_json(self, _context: &Data<Self::DataType>) -> LemmyResult<SanctionNoticeProtocol> {
    // v0: governance objects are never re-served by the local instance.
    // Outbound construction lives in Phase 6 task 74 (Agent D), which
    // builds SanctionNoticeProtocol directly from a local Sanction row.
    // This NotFound stub is reached only if a remote instance HTTP-GETs
    // the URL of a stored advisory notice — not a v0 path.
    Err(LemmyErrorType::NotFound.into())
  }

  async fn verify(
    object: &SanctionNoticeProtocol,
    expected_domain: &Url,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    // Verify the activity originates from the domain that owns the id.
    // Actor authority to sanction is NOT checked in v0 (v2 concern per
    // [docs/brehon-law-inspired-network/06-security-and-threat-model.md §7]).
    verify_domains_match(&object.id, expected_domain)?;
    Ok(())
  }

  async fn from_json(
    _object: SanctionNoticeProtocol,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<Self> {
    // The v0 inbound receive path goes through
    // crates/apub/apub/src/governance/inbox.rs::receive_remote_sanction_notice
    // (Phase 6 task 75 / Agent E), not through Object::from_json. This stub
    // is unreachable from any v0 wiring because PublishSanctionNotice::receive
    // (Phase 6 task 73 / Agent C) calls the free function directly.
    Err(LemmyErrorType::NotFound.into())
  }
}
