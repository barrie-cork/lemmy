use crate::protocol::governance::publish_sanction_notice::{
  PublishSanctionNotice,
  SanctionNoticeKind,
  SanctionNoticeObjectStub,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::{activity::CreateType, public},
  traits::{Activity, Object},
};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::person::ApubPerson,
  protocol::governance::sanction_notice::SanctionNoticeProtocol,
  utils::functions::{GetActorType, verify_is_public},
};
use lemmy_db_schema::{
  newtypes::{ModerationCaseId, SanctionId},
  source::{
    activity::{ActivitySendTargets, SentActivity, SentActivityForm},
    comment::Comment,
    community::Community,
    governance::{moderation_case::ModerationCase, sanction::Sanction},
    person::Person,
    post::Post,
  },
};
use lemmy_db_schema_file::{
  enums::{ActorType, CaseTargetType, SanctionScope},
  schema::{moderation_case, sanction},
};
use lemmy_diesel_utils::{
  connection::DbPool,
  dburl::DbUrl,
  traits::Crud,
};
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use serde_json::{Map, Value, json};
use tracing::info;
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

    // Actor binding: the activity signer (whose HTTP key was verified by
    // activitypub_federation::actix_web::inbox before we got here) MUST
    // match the actor named in the inner SanctionNotice object. Without
    // this check, a same-instance attacker can spoof object.actor while
    // signing the activity as a different principal, impersonating the
    // named actor as the sanction issuer. Sanction-notice inbox uses
    // activity.actor for source_instance derivation in v0, but the inner
    // object.actor is still stored downstream — and v1 enforcement paths
    // will read object.actor when applying remote sanctions, so the
    // binding must hold from v0 forward. CodeRabbit PR #46 finding #19.
    let object_actor = self
      .object
      .rest
      .get("actor")
      .and_then(serde_json::Value::as_str)
      .ok_or_else(|| {
        LemmyErrorType::Unknown("SanctionNotice object missing actor field".into())
      })?;
    let object_actor_url = Url::parse(object_actor).map_err(|e| {
      LemmyErrorType::Unknown(format!(
        "SanctionNotice object.actor not a valid URL: {e}"
      ))
    })?;
    if self.actor.inner() != &object_actor_url {
      return Err(
        LemmyErrorType::Unknown(format!(
          "SanctionNotice actor binding mismatch: activity.actor={} object.actor={}",
          self.actor.inner(),
          object_actor_url,
        ))
        .into(),
      );
    }
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    // Wired by Agent E (plan task 75) to
    // `crate::governance::inbox::receive_remote_sanction_notice`. Inbox
    // lives in `lemmy_apub_activities` (not `lemmy_apub`) per advisor
    // decision DQ-6.6-inbound — see `crate::governance::inbox` module
    // doc for the dep-graph rationale.
    crate::governance::inbox::receive_remote_sanction_notice(self, context).await
  }
}

// ===========================================================================
// Outbound publisher (Phase 6 task 74 — Agent D)
// ===========================================================================
//
// Architectural note (DQ-6.6, resolved as `impl-self-resolved` 2026-04-19):
// the brief asked for a single `send_local_sanction_notice` function in
// this crate that calls both `send_lemmy_activity` and
// `governance_log::append` inside a `conn.run_transaction`. Three blockers
// surfaced:
//
//   1. `lemmy_api::governance::governance_log::append` is in `lemmy_api`,
//      and `lemmy_apub_activities` is upstream of `lemmy_api` (no
//      `lemmy_api` dep here). Adding one would cycle with Agent F's
//      planned `lemmy_api → lemmy_apub_activities` dep for task 76.
//
//   2. `lemmy_api::governance::redaction::scrub` has the same cycle
//      problem. Resolved naturally by reading from `public_case_log.summary`
//      (already redacted at write time by `submit_jury_vote.rs:283`) so
//      this publisher never needs to call `scrub`.
//
//   3. The upstream `send_lemmy_activity` helper takes `&Data<LemmyContext>`
//      and resolves a fresh pool conn for `SentActivity::create`, so the
//      INSERT cannot share a transaction with a caller-owned conn unless
//      we either refactor that helper (viral upstream change) or inline
//      `SentActivity::create(&mut conn.into(), form)` ourselves.
//
// Resolution (option (b) in DQ-6.6): split the publisher into a pure
// builder that runs in this crate (no log, no tx — just AP-shape
// construction) and a tx-owning orchestrator that lives in `lemmy_api`
// (Agent F's `crate::governance::federation_outbox::send_local_sanction_notice`).
// The orchestrator opens the tx, calls `enqueue_sanction_notice_activity`
// from this crate inside the tx, then calls `governance_log::append`
// inside the same tx. Step 8 + step 9 share a transaction; the [06 §2.3]
// "before the user response returns" guarantee is preserved.
//
// Public surface to Agent F is unchanged: they still write
//
//     crate::governance::federation_outbox::send_local_sanction_notice(
//         case_id, &context).await?
//
// per their plan §76 brief; the wrapper inside `lemmy_api` consumes this
// crate's builder.

/// Pre-flight plan describing how to send a finalised local sanction
/// notice across federation.
///
/// Built by [`build_local_sanction_notice_plan`]. Consumed by Agent F's
/// `crate::governance::federation_outbox` wrapper inside a transaction:
/// (a) [`enqueue_sanction_notice_activity`] inserts the `sent_activity`
/// row, (b) `lemmy_api::governance::governance_log::append` writes the
/// `federation_sanction_sent` log entry. Both writes share the same conn
/// and therefore the same tx scope.
///
/// `log_payload` is pre-built (and idempotent) so the orchestrator can
/// pass it straight to `governance_log::append`. `log_payload` does NOT
/// require [`lemmy_api::governance::redaction::scrub_json`] beyond what
/// the writer applies internally — the only string field is
/// `target_url`, which is an AP id (already public).
#[derive(Debug, Clone)]
pub struct SanctionNoticeSendPlan {
  /// The wrapper Create activity, ready to serialise into
  /// `SentActivityForm.data` via `serde_json::to_value`.
  pub activity: PublishSanctionNotice,
  /// Recipients per ADR-014: instance-wide outbound. v0 always uses
  /// `ActivitySendTargets::to_all_instances()`; v1 may add a peer
  /// allowlist.
  pub send_targets: ActivitySendTargets,
  /// Payload for the `federation_sanction_sent` governance log entry.
  /// Includes `case_id`, `sanction_id`, `target_url` per the brief.
  pub log_payload: Value,
  /// Echoed for Agent F's wrapper convenience (saves a re-load).
  pub case_id: ModerationCaseId,
  /// Echoed for Agent F's wrapper convenience.
  pub sanction_id: SanctionId,
  /// Echoed for Agent F's wrapper convenience.
  pub target_url: Url,
  /// AP id of the actor for `SentActivityForm.actor_apub_id`.
  pub actor_apub_id: DbUrl,
  /// Actor type for `SentActivityForm.actor_type`. Always
  /// [`ActorType::Person`] for a local admin in v0 (ADR-010 single-admin
  /// convention).
  pub actor_type: ActorType,
}

/// Build the AP `Create`-wrapped `SanctionNotice` activity from a
/// finalised local case + winning sanction, plus the metadata Agent F's
/// orchestrator needs to log the send inside its transaction.
///
/// This function performs DB **reads only** — no INSERTs, no UPDATEs.
/// The caller supplies a `&mut AsyncPgConnection`; if the caller is in
/// an outer transaction (e.g. Agent F's `submit_jury_vote` orchestrator
/// per plan §757), the build sees the in-flight tx's writes (including
/// the just-inserted `sanction` row at `submit_jury_vote.rs:226-242`).
/// If a future caller is not in a transaction (e.g. a scheduled
/// republish job), they can adapt by checking out a fresh conn from
/// the pool with `lemmy_diesel_utils::connection::get_conn` and
/// passing the deref'd `&mut AsyncPgConnection` here.
///
/// `actor` MUST be the local admin Person (per ADR-010 single-admin
/// convention) wrapped as [`ApubPerson`]. The orchestrator looks it up
/// via `PersonView::list_admins` (filter `local_user::admin = true`)
/// and `.into()`s the `Person` into `ApubPerson`.
///
/// `context` is required for activity-id hostname lookup and for
/// `read_from_id` resolution inside `resolve_target_url`'s `Person::read`
/// /`Post::read`/etc. — those calls take `&mut DbPool<'_>`, which we
/// derive from the conn via `(&mut *conn).into()`.
///
/// # Errors
///
/// - `NotFound` if the case has no row, no winning sanction, or no
///   resolvable target.
/// - DB errors from the conn.
pub async fn build_local_sanction_notice_plan(
  case_id: ModerationCaseId,
  actor: &ApubPerson,
  conn: &mut AsyncPgConnection,
  context: &Data<LemmyContext>,
) -> LemmyResult<SanctionNoticeSendPlan> {
  // Defensive guard 1 — actor must be local. ADR-014 federation is
  // outbound-only and ADR-010 makes the local admin the sole signer of
  // governance activities. No current caller violates (orchestrator
  // looks up via PersonView::list_admins which is local-by-construction)
  // but the guard hardens against future call sites that hand-pick an
  // actor from a federated lookup. CodeRabbit PR #46 #2p-2.
  assert_actor_is_local(actor)?;

  // Step 1 — load ModerationCase (target_type + per-target id columns).
  let case: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select(ModerationCase::as_select())
    .first(conn)
    .await?;

  // Step 1 (cont) — load the winning Sanction. Each finalised case has
  // at most one active sanction (submit_jury_vote.rs:226-242 inserts
  // exactly once when the decision is non-NoAction). If the winning
  // decision was NoAction, this query returns no rows and the function
  // returns NotFound — the orchestrator should not have called us.
  let winning_sanction: Sanction = sanction::table
    .filter(sanction::case_id.eq(case_id))
    .filter(sanction::active.eq(true))
    .select(Sanction::as_select())
    .first(conn)
    .await?;

  // Defensive guard 2 — scope must be FederatedRecommendation. Only that
  // scope produces a federation-visible signal per ADR-014 / [05 §3];
  // Community/Instance scopes stay local and must not generate an
  // outbound activity. submit_jury_vote currently only calls the send
  // path when scope == FederatedRecommendation (see plan §757), but
  // hardening the builder prevents a future caller from publishing a
  // Community/Instance sanction by accident. CodeRabbit PR #46 #2p-3.
  assert_scope_is_federated(winning_sanction.scope)?;

  // Step 3 — resolve target AP id by target type. Each variant uses the
  // matching id column on the case row; remote targets carry the URL
  // verbatim in `target_remote_url`.
  let target_url = resolve_target_url(&case, conn).await?;

  // Step 4 — read the redacted summary from public_case_log. The summary
  // was scrubbed at write time by submit_jury_vote.rs:283 via
  // redaction::scrub(); we never re-scrub here. See DQ-6.6 §2 for why
  // this is safe (and avoids the lemmy_api dep cycle).
  let summary_redacted = read_redacted_summary(case_id, conn).await?;

  // Step 5 — construct the typed AP object protocol struct.
  let activity_id = generate_governance_activity_id(CreateType::Create, context)?;
  let inner_object_id = synthesise_object_id(&activity_id, &winning_sanction.id)?;
  let actor_object_id: ObjectId<ApubPerson> = actor.id().clone().into();
  let object_protocol = SanctionNoticeProtocol::new(
    inner_object_id,
    actor_object_id.clone(),
    target_url.clone(),
    winning_sanction.action,
    winning_sanction.scope,
    summary_redacted,
    Utc::now(),
  );

  // Step 6 — wrap in the Create activity. The wrapper carries
  // `SanctionNoticeObjectStub` rather than the typed protocol per Agent
  // C's task 73 — see TODO(merge-1b) in
  // `protocol/governance/publish_sanction_notice.rs`. Serialise the
  // typed protocol once and copy its fields into the stub's flattened
  // map; preserve the `type` discriminator on `stub.kind`.
  let object_stub = stub_from_protocol(&object_protocol)?;
  let activity = PublishSanctionNoticeFromBuilder {
    actor: actor_object_id.clone(),
    to: vec![public()],
    cc: vec![],
    object: object_stub,
    kind: CreateType::Create,
    id: activity_id.clone(),
  }
  .into_protocol();

  // Step 7 — broadcast to all instances. v0 has no peer allowlist
  // (federation_peer table is v1 work per plan §455).
  let send_targets = ActivitySendTargets::to_all_instances();

  // Step 9 (deferred to orchestrator) — pre-build the log payload.
  let log_payload = json!({
    "case_id": case_id.0,
    "sanction_id": winning_sanction.id.0,
    "target_url": target_url.to_string(),
    "scope": winning_sanction.scope,
    "action": winning_sanction.action,
    "activity_id": activity_id.to_string(),
  });

  Ok(SanctionNoticeSendPlan {
    activity,
    send_targets,
    log_payload,
    case_id,
    sanction_id: winning_sanction.id,
    target_url,
    actor_apub_id: actor.ap_id.clone(),
    actor_type: actor.actor_type(),
  })
}

/// Top-level orchestrator-facing alias.
///
/// The brief's signature for Agent F's `crate::governance::federation_outbox`
/// wrapper is `send_local_sanction_notice(case_id, conn, context, actor_pseudonym)
/// -> LemmyResult<()>` (the in-flight-conn signature mandated by plan §757
/// — see DQ-6.7 resolved id 38). That wrapper lives in `lemmy_api` and
/// calls [`build_local_sanction_notice_plan`] +
/// [`enqueue_sanction_notice_activity`] + `governance_log::append` on the
/// caller's in-flight conn so the federation publish is atomic with
/// `submit_jury_vote`'s outer transaction.
///
/// We expose this *name* as a re-export point in
/// `crates/apub/apub/src/governance/outbox.rs`, but the actual *function*
/// with that signature lives in `lemmy_api` because only that crate can
/// own the call site (and reach `governance_log::append` directly even
/// after Agent E2's relocation).
pub use build_local_sanction_notice_plan as send_local_sanction_notice_plan;

/// Insert the prepared activity into `sent_activity` on the caller's
/// transaction conn.
///
/// This is the only DB-write step on the activities side. It mirrors the
/// upstream `send_lemmy_activity` helper at `lib.rs:114-144` in shape;
/// the difference is that it takes a pre-checked-out `&mut DbConn`
/// (coerced to `&mut DbPool::Conn`) rather than `&Data<LemmyContext>`,
/// so the INSERT participates in the caller's transaction.
///
/// The caller MUST call `lemmy_api::governance::governance_log::append`
/// on the same `conn` immediately afterwards, with `entry_kind =
/// ENTRY_KIND_FEDERATION_SANCTION_SENT` and the plan's `log_payload`,
/// to satisfy the brief's "step 9 must run iff step 8 commits"
/// guarantee.
pub async fn enqueue_sanction_notice_activity(
  plan: &SanctionNoticeSendPlan,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<()> {
  info!(
    "Saving outgoing governance activity {} (sanction_notice for case {})",
    plan.activity.id(),
    plan.case_id.0
  );

  let send_targets = &plan.send_targets;
  let form = SentActivityForm {
    ap_id: plan.activity.id().clone().into(),
    data: serde_json::to_value(&plan.activity)?,
    sensitive: false,
    send_inboxes: send_targets
      .inboxes
      .iter()
      .map(|u| Some(u.clone().into()))
      .collect(),
    send_all_instances: send_targets.all_instances,
    send_community_followers_of: send_targets.community_followers_of.map(|c| c.0),
    actor_type: plan.actor_type,
    actor_apub_id: plan.actor_apub_id.clone(),
  };

  let mut pool: DbPool<'_> = conn.into();
  SentActivity::create(&mut pool, form).await?;
  Ok(())
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

/// Reject a non-local actor at the builder boundary. ADR-010 single-admin
/// + ADR-014 outbound-only federation make a remote actor publishing a
/// sanction-notice from this instance an invariant violation, not a
/// recoverable error. Returns `LemmyErrorType::Unknown` (no dedicated
/// variant exists; precedent in this file at lines 83/86/92 for verify()
/// guards uses the same shape).
fn assert_actor_is_local(actor: &ApubPerson) -> LemmyResult<()> {
  if !actor.local {
    return Err(
      LemmyErrorType::Unknown(format!(
        "publish_sanction_notice builder refused remote actor: ap_id={}",
        actor.ap_id,
      ))
      .into(),
    );
  }
  Ok(())
}

/// Reject a non-federated sanction at the builder boundary. Only
/// `SanctionScope::FederatedRecommendation` produces a federation-visible
/// signal per ADR-014; Community/Instance scopes stay local. Returns
/// `LemmyErrorType::Unknown` matching the actor-guard precedent above.
fn assert_scope_is_federated(scope: SanctionScope) -> LemmyResult<()> {
  if scope != SanctionScope::FederatedRecommendation {
    return Err(
      LemmyErrorType::Unknown(format!(
        "publish_sanction_notice builder refused non-federated scope: {scope:?}",
      ))
      .into(),
    );
  }
  Ok(())
}

/// Resolve the AP id of the case's target. Works the same way for
/// finalised local sanctions whose targets live on the home instance
/// (Person/Post/Comment/Community) and for cases reported against a
/// remote URL stored in `target_remote_url`.
async fn resolve_target_url(
  case: &ModerationCase,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<Url> {
  let mut pool: DbPool<'_> = conn.into();
  match case.target_type {
    CaseTargetType::Person => {
      let id = case
        .target_person_id
        .ok_or(LemmyErrorType::NotFound)?;
      let person = Person::read(&mut pool, id).await?;
      Ok(person.ap_id.into())
    }
    CaseTargetType::Post => {
      let id = case.target_post_id.ok_or(LemmyErrorType::NotFound)?;
      let post = Post::read(&mut pool, id).await?;
      Ok(post.ap_id.into())
    }
    CaseTargetType::Comment => {
      let id = case
        .target_comment_id
        .ok_or(LemmyErrorType::NotFound)?;
      let comment = Comment::read(&mut pool, id).await?;
      Ok(comment.ap_id.into())
    }
    CaseTargetType::Community => {
      let id = case
        .target_community_id
        .ok_or(LemmyErrorType::NotFound)?;
      let community = Community::read(&mut pool, id).await?;
      Ok(community.ap_id.into())
    }
    CaseTargetType::RemoteInstance => {
      let url_str = case
        .target_remote_url
        .as_ref()
        .ok_or(LemmyErrorType::NotFound)?;
      Url::parse(url_str).map_err(|_e| LemmyErrorType::NotFound.into())
    }
  }
}

/// Read the already-redacted summary from `public_case_log`. The row is
/// inserted by `submit_jury_vote.rs:283-298` exactly once when the case
/// flips to Decided, with `redaction::scrub` already applied, so the
/// federation publisher never re-scrubs here. See DQ-6.6 §2.
async fn read_redacted_summary(
  case_id: ModerationCaseId,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<String> {
  use lemmy_db_schema_file::schema::public_case_log;
  let summary: String = public_case_log::table
    .filter(public_case_log::case_id.eq(case_id))
    .order(public_case_log::published_at.desc())
    .select(public_case_log::summary)
    .first::<String>(conn)
    .await?;
  Ok(summary)
}

/// Generate a governance activity id of the form
/// `https://<host>/activities/<kind>/<uuid>`. Mirrors the crate-private
/// `generate_activity_id` at `lib.rs:86-97`; reimplemented here because
/// that helper is not pub.
fn generate_governance_activity_id<T>(
  kind: T,
  context: &LemmyContext,
) -> Result<Url, url::ParseError>
where
  T: ToString,
{
  let id = format!(
    "{}/activities/{}/{}",
    context.settings().get_protocol_and_hostname(),
    kind.to_string().to_lowercase(),
    uuid::Uuid::new_v4(),
  );
  Url::parse(&id)
}

/// Synthesise an inner-object id for the `SanctionNotice` AP object,
/// derived from the wrapper activity id + the local sanction id. The
/// inner id is required by the AP `Object` schema even though v0
/// receivers do not dereference it (see
/// `objects/src/governance/sanction_notice.rs::read_from_id` returning
/// `Ok(None)`).
fn synthesise_object_id(
  activity_id: &Url,
  sanction_id: &SanctionId,
) -> Result<Url, url::ParseError> {
  let id = format!("{}/object/{}", activity_id.as_str(), sanction_id.0);
  Url::parse(&id)
}

/// Convert a typed `SanctionNoticeProtocol` into the discriminator-bearing
/// `SanctionNoticeObjectStub` that Agent C's wrapper carries inside
/// `PublishSanctionNotice.object`. Per the brief: "Construct the stub by
/// serialising your `SanctionNoticeProtocol` with `serde_json::to_value`
/// and copying into the `Map`."
///
/// TODO(merge-1b): remove this shim when the wrapper switches to
/// carrying the typed protocol directly.
fn stub_from_protocol(
  protocol: &SanctionNoticeProtocol,
) -> LemmyResult<SanctionNoticeObjectStub> {
  let value = serde_json::to_value(protocol)?;
  let mut object_map: Map<String, Value> = match value {
    Value::Object(map) => map,
    _ => {
      return Err(
        LemmyErrorType::Unknown(
          "SanctionNoticeProtocol did not serialise to an object".into(),
        )
        .into(),
      );
    }
  };
  // The stub captures `type` separately on `kind`; remove it from `rest`
  // so serde does not double-emit it on the wire.
  object_map.remove("type");
  Ok(SanctionNoticeObjectStub {
    kind: SanctionNoticeKind::SanctionNotice,
    rest: object_map,
  })
}

/// Construct a `PublishSanctionNotice` wrapper. Lives in this file because
/// the wrapper's fields are `pub(crate)` to the activities crate (Agent
/// C's task 73 design). Same-crate use lets us struct-literal it; a
/// public constructor on the wrapper itself is deliberately not added
/// because external constructors would invite consumers to bypass the
/// builder above.
struct PublishSanctionNoticeFromBuilder {
  actor: ObjectId<ApubPerson>,
  to: Vec<Url>,
  cc: Vec<Url>,
  object: SanctionNoticeObjectStub,
  kind: CreateType,
  id: Url,
}

impl PublishSanctionNoticeFromBuilder {
  fn into_protocol(self) -> PublishSanctionNotice {
    PublishSanctionNotice {
      actor: self.actor,
      to: self.to,
      cc: self.cc,
      object: self.object,
      kind: self.kind,
      id: self.id,
    }
  }
}

#[cfg(test)]
mod tests {
  //! Pure-function tests for the two builder-time invariant guards added
  //! per CodeRabbit PR #46 #2p-2 (reject remote actor) and #2p-3 (enforce
  //! federated scope). The full `build_local_sanction_notice_plan` needs
  //! a live AsyncPgConnection and real moderation_case / sanction /
  //! public_case_log rows, so end-to-end coverage of the builder lives
  //! in `crates/server/tests/e2e.rs`. These tests cover the guards in
  //! isolation so the invariant holds even if the surrounding builder
  //! shape changes.
  //!
  //! No unwrap/expect per workspace lints — tests return LemmyResult.
  use super::{ApubPerson, SanctionScope, assert_actor_is_local, assert_scope_is_federated};
  use chrono::Utc;
  use lemmy_db_schema::source::person::Person;
  use lemmy_db_schema_file::{
    PersonId, InstanceId,
    enums::MembershipState,
  };
  use lemmy_utils::error::LemmyResult;
  use url::Url;

  /// Construct a stub Person with the given `local` flag. All other fields
  /// take placeholder values; the guard does not read them.
  fn fixture_person(local: bool) -> LemmyResult<ApubPerson> {
    let ap_id_url = Url::parse("https://example.test/u/picard")?;
    let inbox_url = Url::parse("https://example.test/u/picard/inbox")?;
    Ok(ApubPerson(Person {
      id: PersonId(1),
      name: "picard".into(),
      display_name: None,
      avatar: None,
      banner: None,
      published_at: Utc::now(),
      updated_at: None,
      ap_id: ap_id_url.into(),
      bio: None,
      local,
      private_key: None,
      public_key: "pk".into(),
      last_refreshed_at: Utc::now(),
      inbox_url: inbox_url.into(),
      matrix_user_id: None,
      bot_account: false,
      instance_id: InstanceId(1),
      deleted: false,
      post_count: 0,
      post_score: 0,
      comment_count: 0,
      comment_score: 0,
      // See person.rs:471 — Brehon Phase 5a task 51 carry-patch field;
      // placeholder identical to upstream test fixture pattern.
      membership_state: MembershipState::Member,
    }))
  }

  #[test]
  fn assert_actor_is_local_accepts_local() -> LemmyResult<()> {
    let actor = fixture_person(true)?;
    assert_actor_is_local(&actor)?;
    Ok(())
  }

  #[test]
  fn assert_actor_is_local_rejects_remote() -> LemmyResult<()> {
    let actor = fixture_person(false)?;
    let err = assert_actor_is_local(&actor).err();
    assert!(
      err.is_some(),
      "remote actor must be rejected by the builder guard",
    );
    let msg = format!("{:?}", err);
    assert!(
      msg.contains("remote actor"),
      "error message should mention remote actor, got: {msg}",
    );
    Ok(())
  }

  #[test]
  fn assert_scope_is_federated_accepts_federated() -> LemmyResult<()> {
    assert_scope_is_federated(SanctionScope::FederatedRecommendation)?;
    Ok(())
  }

  #[test]
  fn assert_scope_is_federated_rejects_community() -> LemmyResult<()> {
    let err = assert_scope_is_federated(SanctionScope::Community).err();
    assert!(
      err.is_some(),
      "Community-scope sanction must not produce a federation activity",
    );
    let msg = format!("{:?}", err);
    assert!(
      msg.contains("non-federated scope"),
      "error message should mention non-federated scope, got: {msg}",
    );
    Ok(())
  }

  #[test]
  fn assert_scope_is_federated_rejects_instance() -> LemmyResult<()> {
    let err = assert_scope_is_federated(SanctionScope::Instance).err();
    assert!(
      err.is_some(),
      "Instance-scope sanction must not produce a federation activity",
    );
    Ok(())
  }
}
