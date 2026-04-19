//! Shared verification helpers for governance federation activities.
//!
//! Thin wrappers over `activitypub_federation` today; v1 will extend with
//! governance-specific custom verification (e.g. jury-panel attestation
//! strength) per [04 §11]. Establishing the import path now so v1 callers
//! don't churn.
//!
//! HTTP signature verification of inbound governance activities is
//! already performed by `activitypub_federation::actix_web::inbox::receive_activity_with_hook`
//! at the HTTP boundary BEFORE `Activity::receive` runs — there is no
//! v0 work to do at this layer beyond the domain match.

use activitypub_federation::protocol::verification::verify_domains_match;
use lemmy_utils::error::LemmyResult;
use url::Url;

/// Verify that the activity id and the expected origin domain agree.
///
/// Wraps `verify_domains_match` so governance call sites import through a
/// single fork-local function name. v0 has no callers yet (the inbound
/// receiver delegates HTTP signature + domain checks to the
/// `activitypub_federation` middleware that runs before `Activity::receive`);
/// the helper exists so v1 receive paths that need an extra domain check
/// have an obvious home and a stable name.
pub fn verify_governance_activity_domain(
  activity_id: &Url,
  expected_domain: &Url,
) -> LemmyResult<()> {
  // `verify_domains_match` returns `Result<(), activitypub_federation::Error>`;
  // `?` converts via the existing `From<ActivityPubError> for LemmyError`
  // impl so the wrapper preserves the LemmyResult error envelope used
  // throughout the apub crates.
  verify_domains_match(activity_id, expected_domain)?;
  Ok(())
}
