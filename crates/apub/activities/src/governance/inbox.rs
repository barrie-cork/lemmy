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
use diesel_async::{AsyncPgConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::protocol::governance::{
  moderation_label::ModerationLabelProtocol,
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
// v1-federation-inbound-b Task 4 additions
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use diesel::{ExpressionMethods, QueryDsl};
use url::Url;
use crate::protocol::governance::publish_label::PublishLabel;
use lemmy_db_schema::source::governance::{
  federation_inbox_dropped_log::FederationInboxDroppedLogInsertForm,
  federation_inbox_nonce::FederationInboxNonceInsertForm,
  federation_peer::federation_inbox_check_peer_trust,
  remote_moderation_label::RemoteModerationLabelInsertForm,
  governance_log::{
    ENTRY_KIND_FEDERATION_INBOUND_BLOCKED,
    ENTRY_KIND_FEDERATION_INBOUND_DROPPED_OVERSIZE,
    ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_PEER,
    ENTRY_KIND_FEDERATION_INBOUND_DROPPED_REPLAY,
    ENTRY_KIND_FEDERATION_INBOUND_DROPPED_STORAGE_CAP_EVICTED,
    ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED,
    ENTRY_KIND_FEDERATION_LABEL_RECEIVED,
  },
};
use lemmy_db_schema_file::enums::FederationPeerTrust;
use lemmy_db_schema_file::schema::{
  federation_inbox_dropped_log,
  federation_inbox_nonce,
  governance_config,
  remote_moderation_label,
};
use lemmy_diesel_utils::connection::DbConn;
use lemmy_diesel_utils::connection::DbPool;

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
  let activity_id_str = activity.id.to_string();

  // PRD §7.3 storage-cap eviction config read before the main pool borrow.
  let evict_cap =
    get_inbound_config_int(&mut context.pool(), "federation.inbound.per_peer_storage_cap").await?;

  // Steps 2+3 — write the advisory row and append the hash-chain entry
  // in one transaction so ADR-006's "exactly two rows per inbound notice"
  // invariant holds under failure. If the log append errors after the
  // insert, the whole transaction rolls back and we drop both writes
  // (caller retries via AP redelivery).
  let form = RemoteSanctionNoticeInsertForm {
    source_instance: source_instance.clone(),
    target_url: target_url.clone(),
    action,
    scope,
    summary: object.summary.clone(),
    published_at: object.published,
    signature: activity_id_str.clone(),
    local_case_id: None,
    ..Default::default()
  };
  let payload = json!({
    "source_instance": source_instance,
    "target_url": target_url,
    "action": action,
    "scope": scope,
    "activity_id": activity_id_str,
  });

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  // PRD §7.3 — evict oldest unreviewed row for this peer if cap reached.
  evict_oldest_unreviewed_if_needed(&source_instance, "remote_sanction_notice", evict_cap, conn)
    .await?;
  let outcome = conn
    .run_transaction(|conn| {
      async move {
        insert_remote_sanction_notice(&form, conn).await?;
        // Audit the receipt in the hash-chained governance log. No local
        // pseudonym for a remote actor; pass None per ADR-015.
        governance_log::append(
          &mut (&mut *conn).into(),
          ENTRY_KIND_FEDERATION_SANCTION_RECEIVED,
          payload,
          None,
        )
        .await?;
        Ok(())
      }
      .scope_boxed()
    })
    .await;
  // Best-effort persist_failed emit outside the rollback — swallow any
  // secondary failure so the original error is what surfaces to the caller.
  if let Err(e) = &outcome {
    let _ = governance_log::append(
      &mut context.pool(),
      ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED,
      json!({
        "peer_domain": source_instance,
        "activity_id": activity_id_str,
        "table": "remote_sanction_notice",
        "error": format!("{e}"),
      }),
      None,
    )
    .await;
  }
  outcome
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

  let peer_domain = activity
    .actor
    .inner()
    .domain()
    .ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "remote trust attestation actor {} has no domain",
        activity.actor.inner(),
      ))
    })?
    .to_string();
  let actor_url = object.actor.inner().to_string();
  let subject_url = object.subject.to_string();
  let activity_id_str = activity.id.to_string();

  // PRD §7.3 storage-cap eviction config read before the main pool borrow.
  let evict_cap =
    get_inbound_config_int(&mut context.pool(), "federation.inbound.per_peer_storage_cap").await?;

  // Steps 2+3 — write the attestation row and append the hash-chain entry
  // in one transaction so the ADR-006 "exactly two rows" invariant holds
  // under failure (see `receive_remote_sanction_notice` for the same
  // rationale).
  let form = FederationAttestationInsertForm {
    actor_url: actor_url.clone(),
    subject_url: subject_url.clone(),
    attestation_type: object.attestation_type,
    valid_until: object.valid_until,
    signature: activity_id_str.clone(),
    source_instance: Some(peer_domain.clone()),
    ..Default::default()
  };
  let payload = json!({
    "actor_url": actor_url,
    "subject_url": subject_url,
    "attestation_type": object.attestation_type,
    "valid_until": object.valid_until,
    "activity_id": activity_id_str,
  });

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  // PRD §7.3 — evict oldest unreviewed row for this peer if cap reached.
  evict_oldest_unreviewed_if_needed(&peer_domain, "federation_attestation", evict_cap, conn)
    .await?;
  let outcome = conn
    .run_transaction(|conn| {
      async move {
        insert_federation_attestation(&form, conn).await?;
        governance_log::append(
          &mut (&mut *conn).into(),
          ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED,
          payload,
          None,
        )
        .await?;
        Ok(())
      }
      .scope_boxed()
    })
    .await;
  // Best-effort persist_failed emit outside the rollback.
  if let Err(e) = &outcome {
    let _ = governance_log::append(
      &mut context.pool(),
      ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED,
      json!({
        "peer_domain": peer_domain,
        "activity_id": activity_id_str,
        "table": "federation_attestation",
        "error": format!("{e}"),
      }),
      None,
    )
    .await;
  }
  outcome
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
/// Takes a caller-provided conn so it can run inside the same transaction
/// as the subsequent `governance_log::append` call (ADR-006 atomicity).
async fn insert_remote_sanction_notice(
  form: &RemoteSanctionNoticeInsertForm,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<()> {
  insert_into(remote_sanction_notice::table)
    .values(form)
    .execute(conn)
    .await?;
  Ok(())
}

/// INSERT one row into `federation_attestation`. Direct diesel insert.
/// Takes a caller-provided conn for ADR-006 atomicity (see
/// [`insert_remote_sanction_notice`]).
async fn insert_federation_attestation(
  form: &FederationAttestationInsertForm,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<()> {
  insert_into(federation_attestation::table)
    .values(form)
    .execute(conn)
    .await?;
  Ok(())
}

// ---------------------------------------------------------------------------
// v1-federation-inbound-b Task 4 — wrapper, trait, rate-limit, label handler
// ---------------------------------------------------------------------------

/// Local adapter for `governance_config` integer reads. Avoids the circular
/// dependency `lemmy_api` → `lemmy_apub` → `lemmy_apub_activities` that would
/// arise from calling `lemmy_api::governance::config::get_int` here.
async fn get_inbound_config_int(pool: &mut DbPool<'_>, config_key: &str) -> LemmyResult<i64> {
  let conn = &mut get_conn(pool).await?;
  let val: Option<i64> = governance_config::table
    .filter(governance_config::scope.eq("instance"))
    .filter(governance_config::key.eq(config_key))
    .select(governance_config::value_int)
    .first::<Option<i64>>(conn)
    .await
    .map_err(|_e| {
      LemmyErrorType::Unknown(format!("governance_config.{config_key} not seeded"))
    })?;
  val.ok_or_else(|| {
    LemmyErrorType::Unknown(format!(
      "governance_config.{config_key} has null value_int",
    ))
    .into()
  })
}

/// Per-handler discriminators the inbox wrapper needs. Impls land in the
/// `publish_{sanction_notice,trust_attestation,label}.rs` files (Tasks 5-7).
#[async_trait::async_trait]
pub(crate) trait GovernanceInboundActivity: Sized {
  /// AP activity id used for the replay nonce + drop-log.
  fn activity_id(&self) -> &Url;
  /// Domain of the activity actor (for peer-trust + per-peer rate).
  fn actor_domain(&self) -> LemmyResult<String>;
  /// Serialised payload size in bytes (compared against the per-type cap).
  fn payload_size_bytes(&self) -> LemmyResult<usize>;
  /// `governance_config` key for this activity's size cap.
  fn payload_size_cap_key(&self) -> &'static str;
  /// Per-actor rate-limit check. Default is a no-op; only trust attestations
  /// override this (Task 6).
  async fn check_per_actor_rate_limit(
    &self,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let _ = context;
    Ok(())
  }
}

/// In-memory per-peer hourly rate-limit counters. Key: (peer_domain, hour_bucket).
pub(crate) fn rate_per_peer_counts() -> &'static Mutex<HashMap<(String, i64), u32>> {
  static CELL: OnceLock<Mutex<HashMap<(String, i64), u32>>> = OnceLock::new();
  CELL.get_or_init(|| Mutex::new(HashMap::new()))
}

/// In-memory per-actor hourly rate-limit counters. Key: (subject_url, hour_bucket).
#[expect(dead_code, reason = "pre-landed federation-inbound enforcement infra; callers wired by Cohort B Tasks 5-7 (impl GovernanceInboundActivity + wrap_governance_inbound call sites), which declare requires: task 4 per plan §13")]
pub(crate) fn rate_per_actor_counts() -> &'static Mutex<HashMap<(String, i64), u32>> {
  static CELL: OnceLock<Mutex<HashMap<(String, i64), u32>>> = OnceLock::new();
  CELL.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Current UTC hour as an `i64` bucket identifier for rate-limit maps.
pub(crate) fn current_hour_bucket() -> i64 {
  chrono::Utc::now().timestamp() / 3600
}

/// Apply the five-gate inbound enforcement policy then delegate to `inner`.
///
/// Called from each `Activity::receive` impl (Tasks 5-7) so the gate
/// sequence is identical across all governance activity types.
#[expect(dead_code, reason = "pre-landed federation-inbound enforcement infra; callers wired by Cohort B Tasks 5-7 (impl GovernanceInboundActivity + wrap_governance_inbound call sites), which declare requires: task 4 per plan §13")]
pub(crate) async fn wrap_governance_inbound<F, Fut, A>(
  activity: A,
  context: &Data<LemmyContext>,
  inner: F,
) -> LemmyResult<()>
where
  F: FnOnce(A, &Data<LemmyContext>) -> Fut,
  Fut: std::future::Future<Output = LemmyResult<()>>,
  A: GovernanceInboundActivity + std::marker::Sync,
{
  let peer_domain = activity.actor_domain()?;
  let activity_id = activity.activity_id().to_string();

  // Read all governance_config values BEFORE the main conn acquisition so we
  // never re-borrow the pool while conn is live (DbPool<'_> lifetime conflict).
  let size_cap =
    get_inbound_config_int(&mut context.pool(), activity.payload_size_cap_key()).await?;
  let peer_cap =
    get_inbound_config_int(&mut context.pool(), "federation.inbound.per_peer_rate_per_hour")
      .await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  // Gate 1 — peer trust: Blocklisted → 403.
  let trust = federation_inbox_check_peer_trust(&peer_domain, conn).await?;
  if trust == FederationPeerTrust::Blocklisted {
    log_inbox_drop(
      &peer_domain,
      Some(&activity_id),
      "blocklisted",
      None,
      ENTRY_KIND_FEDERATION_INBOUND_BLOCKED,
      conn,
    )
    .await?;
    return Err(LemmyErrorType::FederationPeerBlocklisted.into());
  }

  // Gate 2 — per-type payload size cap → 413.
  let size = i64::try_from(activity.payload_size_bytes()?).unwrap_or(i64::MAX);
  if size > size_cap {
    log_inbox_drop(
      &peer_domain,
      Some(&activity_id),
      "oversize",
      None,
      ENTRY_KIND_FEDERATION_INBOUND_DROPPED_OVERSIZE,
      conn,
    )
    .await?;
    return Err(LemmyErrorType::FederationPayloadTooLarge.into());
  }

  // Gate 3 — schema strictness enforced at the serde deserialisation layer
  // (deny_unknown_fields on protocol structs). No wrapper helper needed.

  // Gate 4 — per-peer hourly rate limit → 429.
  let bucket = current_hour_bucket();
  let exceeded_peer = {
    let mut counts = rate_per_peer_counts()
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner);
    counts.retain(|(_, b), _| *b >= bucket - 1);
    let entry = counts.entry((peer_domain.clone(), bucket)).or_insert(0);
    *entry = entry.saturating_add(1);
    i64::from(*entry) > peer_cap
  };
  if exceeded_peer {
    log_inbox_drop(
      &peer_domain,
      Some(&activity_id),
      "rate_limit_peer",
      None,
      ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_PEER,
      conn,
    )
    .await?;
    return Err(LemmyErrorType::FederationPeerRateLimitExceeded.into());
  }

  // Gate 5 — per-actor rate limit; attestations override, other types no-op.
  activity.check_per_actor_rate_limit(context).await?;

  // Gate 6 — replay nonce: unique-violation → 409.
  let nonce_form = FederationInboxNonceInsertForm {
    peer_instance: peer_domain.clone(),
    activity_id: activity_id.clone(),
  };
  let nonce_result = diesel::insert_into(federation_inbox_nonce::table)
    .values(&nonce_form)
    .execute(conn)
    .await;
  if let Err(diesel::result::Error::DatabaseError(
    diesel::result::DatabaseErrorKind::UniqueViolation,
    _,
  )) = nonce_result
  {
    log_inbox_drop(
      &peer_domain,
      Some(&activity_id),
      "replay",
      None,
      ENTRY_KIND_FEDERATION_INBOUND_DROPPED_REPLAY,
      conn,
    )
    .await?;
    return Err(LemmyErrorType::FederationActivityReplayed.into());
  }
  nonce_result?;

  // All gates passed — delegate to the Phase-6 (or new label) handler.
  inner(activity, context).await
}

/// Write one `federation_inbox_dropped_log` row and one `governance_log` entry
/// atomically. Callers return a distinct `LemmyError` after this returns `Ok`.
pub(crate) async fn log_inbox_drop(
  peer_domain: &str,
  activity_id: Option<&str>,
  reason: &str,
  excerpt: Option<&str>,
  entry_kind: &'static str,
  conn: &mut DbConn<'_>,
) -> LemmyResult<()> {
  let form = FederationInboxDroppedLogInsertForm {
    source_instance: peer_domain.to_string(),
    activity_id: activity_id.map(str::to_string),
    drop_reason: reason.to_string(),
    payload_excerpt: excerpt.map(str::to_string),
  };
  let payload = json!({
    "peer_domain": peer_domain,
    "activity_id": activity_id,
    "reason": reason,
  });
  conn
    .run_transaction(|conn| {
      async move {
        diesel::insert_into(federation_inbox_dropped_log::table)
          .values(&form)
          .execute(conn)
          .await?;
        governance_log::append(
          &mut (&mut *conn).into(),
          entry_kind,
          payload,
          None,
        )
        .await?;
        Ok(())
      }
      .scope_boxed()
    })
    .await
}

/// COUNT(*) result row type used by `evict_oldest_unreviewed_if_needed`.
/// Module-level to satisfy `clippy::items_after_statements`.
#[derive(diesel::QueryableByName)]
struct CountRow {
  #[diesel(sql_type = diesel::sql_types::BigInt)]
  count: i64,
}

/// Evict the oldest admin-unreviewed row for `peer_domain` in `table_name`
/// if the per-peer count reaches `cap`. Writes a drop-log row and a
/// `governance_log` entry atomically. Uses raw SQL because Diesel's typed
/// DSL cannot accept a runtime-determined table name.
async fn evict_oldest_unreviewed_if_needed(
  peer_domain: &str,
  table_name: &str,
  cap: i64,
  conn: &mut DbConn<'_>,
) -> LemmyResult<()> {
  let count_sql = format!(
    "SELECT COUNT(*)::bigint AS count FROM {table_name} \
     WHERE source_instance = $1 AND admin_reviewed_at IS NULL"
  );
  let count_row = diesel::sql_query(count_sql)
    .bind::<diesel::sql_types::Text, _>(peer_domain)
    .get_result::<CountRow>(conn)
    .await?;
  if count_row.count < cap {
    return Ok(());
  }
  let delete_sql = format!(
    "DELETE FROM {table_name} WHERE id = \
     (SELECT id FROM {table_name} WHERE source_instance = $1 \
      AND admin_reviewed_at IS NULL ORDER BY received_at ASC LIMIT 1)"
  );
  let form = FederationInboxDroppedLogInsertForm {
    source_instance: peer_domain.to_string(),
    activity_id: None,
    drop_reason: "storage_cap_evicted".to_string(),
    payload_excerpt: None,
  };
  let payload = json!({
    "peer_domain": peer_domain,
    "table": table_name,
    "reason": "storage_cap_evicted",
  });
  let peer_str = peer_domain.to_string();
  conn
    .run_transaction(|conn| {
      async move {
        diesel::sql_query(delete_sql)
          .bind::<diesel::sql_types::Text, _>(peer_str.as_str())
          .execute(conn)
          .await?;
        diesel::insert_into(federation_inbox_dropped_log::table)
          .values(&form)
          .execute(conn)
          .await?;
        governance_log::append(
          &mut (&mut *conn).into(),
          ENTRY_KIND_FEDERATION_INBOUND_DROPPED_STORAGE_CAP_EVICTED,
          payload,
          None,
        )
        .await?;
        Ok(())
      }
      .scope_boxed()
    })
    .await
}

/// Decode the wrapper's untyped `ModerationLabelObjectStub` into the typed
/// [`ModerationLabelProtocol`]. Mirrors [`decode_sanction_notice_object`].
fn decode_moderation_label_object(
  activity: &PublishLabel,
) -> LemmyResult<ModerationLabelProtocol> {
  let mut map = activity.object.rest.clone();
  map.insert(
    "type".to_string(),
    Value::String("ModerationLabel".to_string()),
  );
  serde_json::from_value::<ModerationLabelProtocol>(Value::Object(map))
    .map_err(|e| LemmyErrorType::Unknown(format!("decode ModerationLabelProtocol: {e}")).into())
}

/// Persist an inbound `PublishLabel` as an advisory record and log the receipt.
/// Fills the Phase-6 no-op stub.
///
/// Per DQ #273 option-a, `peer_trust_level_at_receipt` is re-queried here
/// rather than threaded down from the wrapper.
pub async fn receive_remote_moderation_label(
  activity: PublishLabel,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let object = decode_moderation_label_object(&activity)?;
  let peer_domain = activity
    .actor
    .inner()
    .domain()
    .map(str::to_string)
    .unwrap_or_default();
  let actor_url = activity.actor.inner().to_string();
  let target_url = object.target.to_string();
  let label = object.label;
  let summary = object.summary;
  let published_at = object.published;
  let activity_id_str = activity.id.to_string();

  info!(
    "Receiving remote moderation label {} (target={}, label={})",
    activity_id_str, target_url, label,
  );

  // Read config before acquiring the main conn.
  let evict_cap =
    get_inbound_config_int(&mut context.pool(), "federation.inbound.per_peer_storage_cap").await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  // DQ #275 option-a — eviction at insert time.
  evict_oldest_unreviewed_if_needed(&peer_domain, "remote_moderation_label", evict_cap, conn)
    .await?;

  // DQ #273 option-a — re-query peer trust inside the handler body.
  let trust = federation_inbox_check_peer_trust(&peer_domain, conn).await?;

  let form = RemoteModerationLabelInsertForm {
    source_instance: peer_domain.clone(),
    actor_url: actor_url.clone(),
    target_url: target_url.clone(),
    label: label.clone(),
    summary: summary.clone(),
    published_at,
    signature: activity_id_str.clone(),
    local_case_id: None,
    peer_trust_level_at_receipt: Some(trust),
  };
  let payload = json!({
    "source_instance": peer_domain,
    "actor_url": actor_url,
    "target_url": target_url,
    "label": label,
    "activity_id": activity_id_str,
  });

  let outcome = conn
    .run_transaction(|conn| {
      async move {
        diesel::insert_into(remote_moderation_label::table)
          .values(&form)
          .execute(conn)
          .await?;
        governance_log::append(
          &mut (&mut *conn).into(),
          ENTRY_KIND_FEDERATION_LABEL_RECEIVED,
          payload,
          None,
        )
        .await?;
        Ok(())
      }
      .scope_boxed()
    })
    .await;
  // Best-effort persist_failed emit outside the rollback.
  if let Err(e) = &outcome {
    let _ = governance_log::append(
      &mut context.pool(),
      ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED,
      json!({
        "peer_domain": peer_domain,
        "activity_id": activity_id_str,
        "table": "remote_moderation_label",
        "error": format!("{e}"),
      }),
      None,
    )
    .await;
  }
  outcome
}
