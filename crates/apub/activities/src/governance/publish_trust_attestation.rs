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
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::person::ApubPerson,
  protocol::governance::trust_attestation::TrustAttestationProtocol,
  utils::functions::{GetActorType, verify_is_public},
};
use lemmy_db_schema::source::{
  activity::{ActivitySendTargets, SentActivity, SentActivityForm},
  governance::governance_log::ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_ACTOR,
};
use lemmy_db_schema_file::{PersonId, enums::{ActorType, AttestationType}, schema::governance_config};
use lemmy_diesel_utils::{connection::{DbPool, get_conn}, dburl::DbUrl};
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

    // Actor binding — see PublishSanctionNotice::verify for the threat
    // model. Trust attestations are MORE exposed than sanction notices in
    // v0 because the inbox at
    // crates/apub/activities/src/governance/inbox.rs:195 stores
    // object.actor.inner() into federation_attestation.actor_url, which
    // means a forged object.actor would land in the DB attributing the
    // attestation to a victim's admin. Without this binding check, an
    // attacker can sign a Create(TrustAttestation) with their own key
    // while naming a different actor in object.actor, and the receiving
    // instance would store the spoofed attribution. CodeRabbit PR #46
    // finding #19.
    let object_actor = self
      .object
      .rest
      .get("actor")
      .and_then(serde_json::Value::as_str)
      .ok_or_else(|| {
        LemmyErrorType::Unknown("TrustAttestation object missing actor field".into())
      })?;
    let object_actor_url = Url::parse(object_actor).map_err(|e| {
      LemmyErrorType::Unknown(format!(
        "TrustAttestation object.actor not a valid URL: {e}"
      ))
    })?;
    if self.actor.inner() != &object_actor_url {
      return Err(
        LemmyErrorType::Unknown(format!(
          "TrustAttestation actor binding mismatch: activity.actor={} object.actor={}",
          self.actor.inner(),
          object_actor_url,
        ))
        .into(),
      );
    }
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    crate::governance::inbox::wrap_governance_inbound(self, context, |a, c| async move {
      crate::governance::inbox::receive_remote_trust_attestation(a, c).await
    })
    .await
  }
}

// v1-federation-inbound-b Task 6 — GovernanceInboundActivity impl
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl crate::governance::inbox::GovernanceInboundActivity for PublishTrustAttestation {
  fn activity_id(&self) -> &Url {
    &self.id
  }
  fn actor_domain(&self) -> LemmyResult<String> {
    self.actor.inner().domain()
      .map(str::to_string)
      .ok_or_else(|| {
        LemmyErrorType::Unknown(
          format!("PublishTrustAttestation actor {} has no domain", self.actor.inner())
        )
        .into()
      })
  }
  fn payload_size_bytes(&self) -> LemmyResult<usize> {
    Ok(serde_json::to_vec(self)?.len())
  }
  fn payload_size_cap_key(&self) -> &'static str {
    "federation.inbound.max_payload_bytes_trust_attestation"
  }
  async fn check_per_actor_rate_limit(
    &self,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    // Extract the attested subject URL from the untyped object stub. The
    // `subject` field is carried in `rest` because TrustAttestationObjectStub
    // uses a catch-all map for non-first-class fields.
    let subject_url = self
      .object
      .rest
      .get("subject")
      .and_then(serde_json::Value::as_str)
      .ok_or_else(|| {
        LemmyErrorType::Unknown("TrustAttestation object missing subject field".into())
      })?;

    // Read the per-actor rate cap. Mirrors the `get_inbound_config_int`
    // helper in inbox.rs (private there; duplicated here to avoid requiring a
    // pub(crate) expansion of inbox.rs internals — the same circular-dep
    // constraint that caused inbox.rs to define the helper locally).
    const CONFIG_KEY: &str = "federation.inbound.per_actor_attestation_rate_per_hour";
    let actor_cap: i64 = {
      let pool = &mut context.pool();
      let conn = &mut get_conn(pool).await?;
      let val: Option<i64> = governance_config::table
        .filter(governance_config::scope.eq("instance"))
        .filter(governance_config::key.eq(CONFIG_KEY))
        .select(governance_config::value_int)
        .first::<Option<i64>>(conn)
        .await
        .map_err(|_e| {
          LemmyErrorType::Unknown(format!("governance_config.{CONFIG_KEY} not seeded"))
        })?;
      val.ok_or_else(|| {
        LemmyErrorType::Unknown(
          format!("governance_config.{CONFIG_KEY} has null value_int"),
        )
        .into()
      })?
    };

    // Increment the per-actor counter. Key structure mirrors the per-peer
    // counter in §10.4 wrapper step 4: (subject_url_string, hour_bucket).
    let bucket = crate::governance::inbox::current_hour_bucket();
    let exceeded_actor = {
      let mut counts = crate::governance::inbox::rate_per_actor_counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
      // Opportunistic prune: drop buckets older than the previous hour.
      counts.retain(|(_, b), _| *b >= bucket - 1);
      let entry = counts.entry((subject_url.to_string(), bucket)).or_insert(0);
      *entry = entry.saturating_add(1);
      i64::from(*entry) > actor_cap
    };

    if exceeded_actor {
      let peer_domain = self.actor_domain()?;
      let activity_id_str = self.id.to_string();
      let pool = &mut context.pool();
      let conn = &mut get_conn(pool).await?;
      crate::governance::inbox::log_inbox_drop(
        &peer_domain,
        Some(activity_id_str.as_str()),
        "rate_limit_actor",
        None,
        ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_ACTOR,
        conn,
      )
      .await?;
      return Err(LemmyErrorType::FederationActorRateLimitExceeded.into());
    }

    Ok(())
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
