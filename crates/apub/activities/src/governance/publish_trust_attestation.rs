use crate::protocol::governance::publish_trust_attestation::PublishTrustAttestation;
use activitypub_federation::{config::Data, traits::Activity};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::utils::functions::verify_is_public;
use lemmy_db_schema::source::governance::governance_log::ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_ACTOR;
use lemmy_db_schema_file::schema::governance_config;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use url::Url;

/// Maximum number of distinct (subject_url, hour_bucket) keys held in the
/// per-actor rate-limit map at any moment. A single allowlisted peer can
/// craft arbitrarily many subject URLs within one hour bucket; the
/// insertion-order eviction at `check_per_actor_rate_limit` keeps the map
/// bounded regardless of attacker key cardinality. See v1-federation-inbound-d
/// plan §3.
const MAX_PER_ACTOR_RATE_ENTRIES: usize = 10_000;

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
    self
      .actor
      .inner()
      .domain()
      .map(str::to_string)
      .ok_or_else(|| {
        LemmyErrorType::Unknown(format!(
          "PublishTrustAttestation actor {} has no domain",
          self.actor.inner()
        ))
        .into()
      })
  }
  fn payload_size_bytes(&self) -> LemmyResult<usize> {
    Ok(serde_json::to_vec(self)?.len())
  }
  fn payload_size_cap_key(&self) -> &'static str {
    "federation.inbound.max_payload_bytes_trust_attestation"
  }
  async fn check_per_actor_rate_limit(&self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    const CONFIG_KEY: &str = "federation.inbound.per_actor_attestation_rate_per_hour";
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
    let actor_cap: i64 = {
      let pool = &mut context.pool();
      let conn = &mut get_conn(pool).await?;
      let val: Option<i64> = governance_config::table
        .filter(governance_config::scope.eq("instance"))
        .filter(governance_config::key.eq(CONFIG_KEY))
        .select(governance_config::value_int)
        .order_by(governance_config::valid_from.desc())
        .first::<Option<i64>>(conn)
        .await
        .map_err(|_e| {
          LemmyErrorType::Unknown(format!("governance_config.{CONFIG_KEY} not seeded"))
        })?;
      val.ok_or_else(|| {
        LemmyErrorType::Unknown(format!("governance_config.{CONFIG_KEY} has null value_int"))
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

      // Insertion-order bound: cap distinct (subject_url, bucket) keys at
      // MAX_PER_ACTOR_RATE_ENTRIES. A single allowlisted peer can craft
      // arbitrarily many subject_url values within one hour bucket; retain()
      // above only drops prior-hour entries. Without this bound the map grows
      // O(attacker key cardinality). See v1-federation-inbound-d plan §3.
      let key = (subject_url.to_string(), bucket);
      if counts.len() >= MAX_PER_ACTOR_RATE_ENTRIES
        && !counts.contains_key(&key)
        && let Some(oldest_key) = counts
          .iter()
          .min_by_key(|((_, b), _)| *b)
          .map(|(k, _)| k.clone())
      {
        counts.remove(&oldest_key);
      }

      let entry = counts.entry(key).or_insert(0);
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

// The Phase-6 task-74 outbound builder path (`TrustAttestationSendPlan`,
// `build_local_trust_attestation_plan`, `enqueue_trust_attestation_activity`)
// was plumbed but never called — no v0/v1 endpoint emits a trust attestation.
// Cut 2026-07-02 (chore/cut-dead-code-safe-set); mirror
// `publish_sanction_notice.rs` when the v1 endorsement-creation handler
// actually wires outbound attestations.

#[cfg(test)]
mod tests_per_actor_bound {
  //! Pure-function tests for the per-actor rate-map insertion-order bound
  //! added per v1-federation-inbound-d plan §3. Exercises
  //! `MAX_PER_ACTOR_RATE_ENTRIES` cap behaviour against the live
  //! `rate_per_actor_counts()` `OnceLock` — clears the global at test start
  //! AND end so test order is not load-bearing across the apub-activities
  //! lib-test binary.
  //!
  //! No unwrap/expect per workspace lints — uses `PoisonError::into_inner`
  //! for Mutex-poison recovery (canonical pattern; see
  //! `publish_trust_attestation.rs:161` + `inbox.rs:547`).
  use super::MAX_PER_ACTOR_RATE_ENTRIES;
  use crate::governance::inbox::{current_hour_bucket, rate_per_actor_counts};
  use std::sync::PoisonError;

  #[test]
  fn per_actor_map_evicts_oldest_when_cap_reached() {
    // Acquire a single guard and hold it through the entire test body to prevent
    // interleaving from concurrent tests in the same binary.
    let mut counts = rate_per_actor_counts()
      .lock()
      .unwrap_or_else(PoisonError::into_inner);

    // Clear stale state from prior tests (the OnceLock is process-global).
    counts.clear();

    let bucket = current_hour_bucket();
    let cap = MAX_PER_ACTOR_RATE_ENTRIES;

    // Insert `cap + 1` distinct keys, applying the same bound logic that
    // ships in `check_per_actor_rate_limit` (Task 1).
    for i in 0..=cap {
      let key = (format!("https://test/{i}"), bucket);
      counts.retain(|(_, b), _| *b >= bucket - 1);
      if counts.len() >= cap
        && !counts.contains_key(&key)
        && let Some(oldest_key) = counts
          .iter()
          .min_by_key(|((_, b), _)| *b)
          .map(|(k, _)| k.clone())
      {
        counts.remove(&oldest_key);
      }
      let entry = counts.entry(key).or_insert(0);
      *entry = entry.saturating_add(1);
    }

    // Assert: map size capped at MAX_PER_ACTOR_RATE_ENTRIES AND the trigger key
    // (i=cap, the key that forced the eviction) is present.
    // Note: which specific prior key gets evicted is not guaranteed — the eviction
    // uses min_by_key on the bucket value, and when all keys share the same bucket
    // (as in this test), HashMap iteration order is unspecified.
    assert_eq!(
      counts.len(),
      cap,
      "per-actor map must be bounded at MAX_PER_ACTOR_RATE_ENTRIES after cap + 1 inserts",
    );
    let trigger_key = (format!("https://test/{cap}"), bucket);
    assert!(
      counts.contains_key(&trigger_key),
      "trigger key (i=cap) must be present after insertion-order-bound eviction",
    );

    // Cleanup: clear the map so other tests in this binary start fresh.
    counts.clear();
  }
}
