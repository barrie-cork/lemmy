//! Inbound governance activity receivers (Phase 6 task 75 — Agent E re-spawn).
//!
//! Crate placement per advisor decision DQ-6.6-inbound (resolved id 37 in
//! `.claude/decision-queue.json`): the receiver functions must live in
//! `lemmy_apub_activities` because `Activity::receive` is invoked by the
//! AP framework at this layer of the dep graph — there is no orchestrator
//! crate above us that could mediate the call (unlike outbound publishing,
//! where Agent F's `lemmy_api::governance::federation_outbox` wrapper plays
//! that role). The original task brief targeted `crates/apub/apub/src/governance/inbox.rs`,
//! but that location was unreachable for the same reason: a body in
//! `lemmy_apub` cannot be invoked from `lemmy_apub_activities`'s
//! `Activity::receive` impl.
//!
//! ## ADR-006 invariant — advisory-only
//!
//! Inbound sanction notices are **never** auto-applied. Per
//! [99 ADR-006](../../../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md),
//! a remote instance announcing a sanction does not automatically remove
//! content or ban actors on the receiving instance — those are local
//! decisions that must go through the local jury workflow. The receivers
//! below write **exactly two rows per inbound notice**:
//!
//! 1. One INSERT into `remote_sanction_notice` (or `federation_attestation`
//!    for trust attestations) with `local_case_id` left NULL — admins
//!    review the row and may open a corresponding local case in v1.
//! 2. One [`governance_log::append`] entry recording that the notice was
//!    received (entry kind `federation_sanction_received` or
//!    `federation_attestation_received`).
//!
//! No `Sanction::create`, no `emergency_remove_open_case`, no
//! `apply_remote_*` — those would violate ADR-006. Any future addition
//! that calls into local enforcement code from this module needs a new
//! ADR superseding ADR-006.
//!
//! ## Trust the framework
//!
//! HTTP signature verification is performed by
//! `activitypub_federation::actix_web::inbox::receive_activity_with_hook`
//! BEFORE `Activity::receive` is invoked. Schema validation is performed
//! by `serde::Deserialize` at the activity-parse step before `verify`
//! runs, and `Activity::verify` (in the trait impls) re-checks
//! `verify_is_public` for governance broadcasts. By the time these
//! receivers run, the input is already authenticated and structurally
//! valid — there is no signature inspection or header parsing here.
//!
//! See plan §task 75 for the full design and ADR-014 for the fork-only AP
//! type policy.

use crate::protocol::governance::{
  publish_sanction_notice::PublishSanctionNotice,
  publish_trust_attestation::PublishTrustAttestation,
};
use activitypub_federation::config::Data;
use diesel::insert_into;
use diesel_async::RunQueryDsl;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::protocol::governance::{
  sanction_notice::SanctionNoticeProtocol,
  trust_attestation::TrustAttestationProtocol,
};
use lemmy_db_schema::source::governance::{
  federation_attestation::FederationAttestationInsertForm,
  governance_log::{
    self,
    ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED,
    ENTRY_KIND_FEDERATION_SANCTION_RECEIVED,
  },
  remote_sanction_notice::RemoteSanctionNoticeInsertForm,
};
use lemmy_db_schema_file::schema::{federation_attestation, remote_sanction_notice};
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::{Value, json};
use tracing::info;

/// Persist an inbound `PublishSanctionNotice` as an advisory record and
/// log the receipt to the governance hash chain.
///
/// Wired from `PublishSanctionNotice::receive` (see
/// `crates/apub/activities/src/governance/publish_sanction_notice.rs`).
///
/// # Steps
///
/// 1. Decode the wrapper's untyped object stub into the typed
///    [`SanctionNoticeProtocol`]. The stub carries a `type` discriminator
///    that we strip when serialising back through `serde_json::to_value`,
///    so we re-attach it before deserialising into the typed protocol.
/// 2. INSERT one row into `remote_sanction_notice` with `local_case_id =
///    None` per ADR-006. The signature column carries the outer Create
///    activity id per DQ-6.2 (resolved id 32) — the actual HTTP signature
///    header value is consumed by `activitypub_federation` before
///    `Activity::receive` runs and is not trivially available here.
/// 3. APPEND one entry to `governance_log` with kind
///    `federation_sanction_received`. `actor_pseudonym = None` because
///    the actor is remote and has no local pseudonym (per ADR-015 the
///    pseudonym table only covers local Persons).
///
/// # Idempotency
///
/// Per DQ-6.3 (resolved id 33) deduplication is handled at the HTTP layer
/// by Lemmy's existing `ReceivedActivity::create` — we do not add a
/// second guard here. A double-receive in v0 would create two
/// advisory rows; admins handle them in the review queue. v1 may add a
/// `(source_instance, target_url, published_at)` partial unique index.
pub async fn receive_remote_sanction_notice(
  activity: PublishSanctionNotice,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  // Step 1 — decode the typed object from the discriminator-bearing stub.
  let object = decode_sanction_notice_object(&activity)?;

  // Trace before any DB work so a failed insert still leaves a breadcrumb.
  info!(
    "Receiving remote sanction notice {} (target={}, action={:?}, scope={:?})",
    activity.id, object.target, object.action, object.scope,
  );

  let source_instance = activity
    .actor
    .inner()
    .domain()
    .ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "remote sanction notice actor {} has no domain",
        activity.actor.inner(),
      ))
    })?
    .to_string();
  let target_url = object.target.to_string();
  let action = object.action;
  let scope = object.scope;

  // Step 2 — write the advisory row. local_case_id stays NULL per ADR-006.
  // signature carries the outer activity id per DQ-6.2 — the actual HTTP
  // signature value is consumed by activitypub_federation before
  // Activity::receive runs and is not exposed to handlers in v0.
  // TODO(v1): plumb actual HTTP signature through activitypub_federation hook.
  let form = RemoteSanctionNoticeInsertForm {
    source_instance: source_instance.clone(),
    target_url: target_url.clone(),
    action,
    scope,
    summary: object.summary.clone(),
    published_at: object.published,
    signature: activity.id.to_string(),
    local_case_id: None,
  };
  insert_remote_sanction_notice(&form, context).await?;

  // Step 3 — audit the receipt in the hash-chained governance log. No
  // local pseudonym for a remote actor; pass None per ADR-015.
  let payload = json!({
    "source_instance": source_instance,
    "target_url": target_url,
    "action": action,
    "scope": scope,
    "activity_id": activity.id.to_string(),
  });
  governance_log::append(
    &mut context.pool(),
    ENTRY_KIND_FEDERATION_SANCTION_RECEIVED,
    payload,
    None,
  )
  .await?;

  Ok(())
}

/// Persist an inbound `PublishTrustAttestation` and log the receipt.
///
/// Mirrors [`receive_remote_sanction_notice`] for trust attestations.
/// Inbound attestations are stored in the same `federation_attestation`
/// table as outbound (v0 design, [04 §3]); the `actor_url` column carries
/// the remote actor and admins can distinguish inbound from outbound by
/// matching `actor_url` against the local instance's domain.
///
/// v0 has no inbound enforcement — receiving a `TrustedReporter`
/// attestation from a remote instance does NOT grant the local user the
/// `TrustedReporter` capability. That decision flows from local
/// `endorsement` / capability code paths, not from federation. The row
/// exists so admins can audit cross-instance trust signals; v1 may add
/// an opt-in apply path.
pub async fn receive_remote_trust_attestation(
  activity: PublishTrustAttestation,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  // Step 1 — decode the typed object from the discriminator-bearing stub.
  let object = decode_trust_attestation_object(&activity)?;

  info!(
    "Receiving remote trust attestation {} (actor={}, subject={}, attestation_type={:?})",
    activity.id, object.actor.inner(), object.subject, object.attestation_type,
  );

  let actor_url = object.actor.inner().to_string();
  let subject_url = object.subject.to_string();

  // Step 2 — write the federation_attestation row. signature carries the
  // outer activity id per the same DQ-6.2 reasoning as sanction notices.
  // TODO(v1): plumb actual HTTP signature through activitypub_federation hook.
  let form = FederationAttestationInsertForm {
    actor_url: actor_url.clone(),
    subject_url: subject_url.clone(),
    attestation_type: object.attestation_type,
    valid_until: object.valid_until,
    signature: activity.id.to_string(),
  };
  insert_federation_attestation(&form, context).await?;

  // Step 3 — audit the receipt.
  let payload = json!({
    "actor_url": actor_url,
    "subject_url": subject_url,
    "attestation_type": object.attestation_type,
    "valid_until": object.valid_until,
    "activity_id": activity.id.to_string(),
  });
  governance_log::append(
    &mut context.pool(),
    ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED,
    payload,
    None,
  )
  .await?;

  Ok(())
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

/// Decode the wrapper's untyped `SanctionNoticeObjectStub` into the typed
/// [`SanctionNoticeProtocol`].
///
/// The stub captures `type` separately on `kind` and flattens the rest of
/// the fields into a `serde_json::Map`. The typed protocol expects
/// `"type": "SanctionNotice"` to be present, so we re-insert the
/// discriminator into the map before deserialising. Stub-side validation
/// already constrained `kind` to the single `SanctionNoticeKind::SanctionNotice`
/// variant, so this is just a shape adjustment, not a trust decision.
///
/// TODO(merge-1b): drop this helper when the wrapper carries the typed
/// protocol directly (matching outbound symmetry).
fn decode_sanction_notice_object(
  activity: &PublishSanctionNotice,
) -> LemmyResult<SanctionNoticeProtocol> {
  let mut map = activity.object.rest.clone();
  // Re-insert the type discriminator that the wrapper stripped on send.
  // SanctionNoticeProtocol's `kind` field is required and serde-renamed
  // to "type" in the wire format.
  map.insert("type".to_string(), Value::String("SanctionNotice".to_string()));
  let object = serde_json::from_value::<SanctionNoticeProtocol>(Value::Object(map))
    .map_err(|e| LemmyErrorType::Unknown(format!("decode SanctionNoticeProtocol: {e}")))?;
  Ok(object)
}

/// Decode the wrapper's untyped `TrustAttestationObjectStub` into the
/// typed [`TrustAttestationProtocol`]. Mirrors
/// [`decode_sanction_notice_object`].
fn decode_trust_attestation_object(
  activity: &PublishTrustAttestation,
) -> LemmyResult<TrustAttestationProtocol> {
  let mut map = activity.object.rest.clone();
  map.insert(
    "type".to_string(),
    Value::String("TrustAttestation".to_string()),
  );
  let object = serde_json::from_value::<TrustAttestationProtocol>(Value::Object(map))
    .map_err(|e| LemmyErrorType::Unknown(format!("decode TrustAttestationProtocol: {e}")))?;
  Ok(object)
}

/// INSERT one row into `remote_sanction_notice`. Direct diesel insert
/// (no `Crud` impl exists for this table — see Phase 6 task 71 model).
async fn insert_remote_sanction_notice(
  form: &RemoteSanctionNoticeInsertForm,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  insert_into(remote_sanction_notice::table)
    .values(form)
    .execute(conn)
    .await?;
  Ok(())
}

/// INSERT one row into `federation_attestation`. Direct diesel insert.
async fn insert_federation_attestation(
  form: &FederationAttestationInsertForm,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  insert_into(federation_attestation::table)
    .values(form)
    .execute(conn)
    .await?;
  Ok(())
}
