use crate::protocol::governance::publish_trust_attestation::{
  PublishTrustAttestation,
  TrustAttestationKind,
  TrustAttestationObjectStub,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::{activity::CreateType, public},
  traits::{Activity, Object},
};
use chrono::{DateTime, Utc};
use diesel_async::AsyncPgConnection;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::person::ApubPerson,
  protocol::governance::trust_attestation::TrustAttestationProtocol,
  utils::functions::{GetActorType, verify_is_public},
};
use lemmy_db_schema::source::activity::{
  ActivitySendTargets,
  SentActivity,
  SentActivityForm,
};
use lemmy_db_schema_file::{PersonId, enums::{ActorType, AttestationType}};
use lemmy_diesel_utils::{connection::DbPool, dburl::DbUrl};
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use serde_json::{Map, Value, json};
use tracing::info;
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

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    // Wired by Agent E (plan task 75) to
    // `crate::governance::inbox::receive_remote_trust_attestation`. See
    // `crate::governance::inbox` module doc for the dep-graph rationale
    // that puts the inbox here in `lemmy_apub_activities` rather than
    // `lemmy_apub` (DQ-6.6-inbound).
    crate::governance::inbox::receive_remote_trust_attestation(self, context).await
  }
}

// ===========================================================================
// Outbound publisher (Phase 6 task 74 — Agent D)
// ===========================================================================
//
// `send_local_trust_attestation` is plumbed but no v0 endpoint emits a
// trust attestation — see plan §3 ("v0 simplifications") and §641 (the
// task 74 brief explicitly says to ship the function shape so v1's
// endorsement-creation handler can wire it without an apub round trip).
// Same architectural shape as `send_local_sanction_notice`: this crate
// produces a [`TrustAttestationSendPlan`] and Agent F's wrapper (which
// won't actually exist in v0 — there's no caller) would orchestrate
// tx + log append.
//
// See `publish_sanction_notice.rs` head-of-module comment for DQ-6.6.

/// Pre-flight plan describing how to send a local trust attestation
/// across federation. Mirrors [`super::publish_sanction_notice::SanctionNoticeSendPlan`].
#[derive(Debug, Clone)]
pub struct TrustAttestationSendPlan {
  pub activity: PublishTrustAttestation,
  pub send_targets: ActivitySendTargets,
  pub log_payload: Value,
  pub subject_person_id: PersonId,
  pub subject_url: Url,
  pub attestation_type: AttestationType,
  pub valid_until: Option<DateTime<Utc>>,
  pub actor_apub_id: DbUrl,
  pub actor_type: ActorType,
}

/// Build the AP `Create`-wrapped `TrustAttestation` activity.
///
/// `subject` is the [`ApubPerson`] the attestation is about (e.g. a
/// TrustedReporter award given to person X). `actor` is the local admin
/// signing the attestation. v0 has no caller for this function — v1
/// wires it into endorsement creation per plan §3.
///
/// Like [`super::publish_sanction_notice::build_local_sanction_notice_plan`],
/// this performs DB **reads only**. The orchestrator opens its tx after
/// this returns and passes the plan to
/// [`enqueue_trust_attestation_activity`] + `governance_log::append`.
pub async fn build_local_trust_attestation_plan(
  actor: &ApubPerson,
  subject: &ApubPerson,
  attestation_type: AttestationType,
  valid_until: Option<DateTime<Utc>>,
  context: &Data<LemmyContext>,
) -> LemmyResult<TrustAttestationSendPlan> {
  let activity_id = generate_governance_activity_id(CreateType::Create, context)?;
  let inner_object_id = synthesise_object_id(&activity_id, attestation_type)?;
  let actor_object_id: ObjectId<ApubPerson> = actor.id().clone().into();
  let subject_url: Url = subject.id().clone();
  let now = Utc::now();

  let object_protocol = TrustAttestationProtocol::new(
    inner_object_id,
    actor_object_id.clone(),
    subject_url.clone(),
    attestation_type,
    valid_until,
    now,
  );

  let object_stub = stub_from_protocol(&object_protocol)?;
  let activity = PublishTrustAttestation {
    actor: actor_object_id.clone(),
    to: vec![public()],
    cc: vec![],
    object: object_stub,
    kind: CreateType::Create,
    id: activity_id.clone(),
  };

  let send_targets = ActivitySendTargets::to_all_instances();

  let log_payload = json!({
    "subject_url": subject_url.to_string(),
    "attestation_type": attestation_type,
    "valid_until": valid_until,
    "activity_id": activity_id.to_string(),
  });

  Ok(TrustAttestationSendPlan {
    activity,
    send_targets,
    log_payload,
    subject_person_id: subject.id,
    subject_url,
    attestation_type,
    valid_until,
    actor_apub_id: actor.ap_id.clone(),
    actor_type: actor.actor_type(),
  })
}

/// Top-level orchestrator-facing alias. See the equivalent in
/// `publish_sanction_notice.rs` for the layering rationale (DQ-6.6).
pub use build_local_trust_attestation_plan as send_local_trust_attestation_plan;

/// Insert the prepared activity into `sent_activity` on the caller's
/// transaction conn. Mirrors
/// [`super::publish_sanction_notice::enqueue_sanction_notice_activity`].
pub async fn enqueue_trust_attestation_activity(
  plan: &TrustAttestationSendPlan,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<()> {
  info!(
    "Saving outgoing governance activity {} (trust_attestation for subject {})",
    plan.activity.id(),
    plan.subject_person_id.0
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
// Internals (mirror `publish_sanction_notice.rs`)
// ---------------------------------------------------------------------------

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

fn synthesise_object_id(
  activity_id: &Url,
  attestation_type: AttestationType,
) -> Result<Url, url::ParseError> {
  let id = format!(
    "{}/object/{}",
    activity_id.as_str(),
    serde_json::to_value(attestation_type)
      .map(|v| v.as_str().unwrap_or("unknown").to_string())
      .unwrap_or_else(|_| "unknown".to_string()),
  );
  Url::parse(&id)
}

fn stub_from_protocol(
  protocol: &TrustAttestationProtocol,
) -> LemmyResult<TrustAttestationObjectStub> {
  let value = serde_json::to_value(protocol)?;
  let mut object_map: Map<String, Value> = match value {
    Value::Object(map) => map,
    _ => {
      return Err(
        LemmyErrorType::Unknown(
          "TrustAttestationProtocol did not serialise to an object".into(),
        )
        .into(),
      );
    }
  };
  object_map.remove("type");
  Ok(TrustAttestationObjectStub {
    kind: TrustAttestationKind::TrustAttestation,
    rest: object_map,
  })
}
