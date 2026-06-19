//! B-publish sanction propagation publisher (ADR-016).
//!
//! Additive in m2-late-1: compiles but has no caller yet.
//! T5 (submit_jury_vote.rs) wires the spawn site in m2-late-2.
#[cfg(feature = "full")]
use {
  activitypub_federation::config::Data,
  chrono::{DateTime, Utc},
  diesel::{ExpressionMethods, OptionalExtension, QueryDsl, insert_into},
  diesel_async::RunQueryDsl,
  lemmy_api_utils::context::LemmyContext,
  lemmy_db_schema::source::governance::{
    sanction::Sanction,
    sanction_event::SanctionEventInsertForm,
    sanction_subscriber::SanctionSubscriber,
  },
  lemmy_db_schema_file::{
    enums::SanctionKind,
    schema::{sanction_event as sanction_event_dsl, sanction_subscriber as sanction_subscriber_dsl},
  },
  lemmy_diesel_utils::connection::{DbPool, get_conn},
  lemmy_utils::error::{LemmyError, LemmyResult},
  serde::{Deserialize, Serialize},
};

#[cfg(feature = "full")]
use crate::governance::{
  actor_pseudonym_helper,
  governance_log::{self, ENTRY_KIND_SANCTION_CREATED},
  sanction_kind_map::map_sanction_action,
};

/// Webhook payload POSTed to each registered subscriber (B-publish, ADR-016).
/// Subject identifier is `actor_pseudonym.pseudonym` ONLY (ADR-015).
#[cfg(feature = "full")]
#[derive(Debug, Serialize, Deserialize)]
pub struct SanctionEventPayload {
  pub sanction_kind: SanctionKind,
  pub case_id: i64,
  /// actor_pseudonym.pseudonym — never person.name or local_user.email (ADR-015).
  pub subject_actor_pseudonym: String,
  pub effective_from: DateTime<Utc>,
  pub effective_until: Option<DateTime<Utc>>,
  /// Hex-encoded governance_log.entry_hash for the sanction_created entry.
  pub governance_log_entry_hash: String,
}

/// Context threaded into the spawned task — a cloned `Data<LemmyContext>` handle.
#[cfg(feature = "full")]
pub type SanctionContext = Data<LemmyContext>;

/// Minimal row returned by the governance_log entry-hash lookup.
#[cfg(feature = "full")]
#[derive(diesel::QueryableByName)]
struct EntryHashRow {
  #[diesel(sql_type = diesel::sql_types::Binary)]
  entry_hash: Vec<u8>,
}

/// Fire-and-forget B-publish: map the sanction to a `SanctionKind`, resolve the
/// subject pseudonym, POST to every active subscriber, then append a governance-log
/// entry recording publish success or delivery failure.
///
/// Called from a `tokio::spawn` outside the vote transaction (T5, m2-late-2).
/// Uses a fresh pool connection for every DB operation — never touches the
/// transaction's `conn` (R8).
#[cfg(feature = "full")]
pub async fn enqueue_sanction_event(sanction: Sanction, ctx: SanctionContext) -> LemmyResult<()> {
  // 1. Map SanctionAction → Option<SanctionKind>. None = no Matrix primitive.
  let Some(sanction_kind) = map_sanction_action(sanction.action) else {
    tracing::debug!(
      action = ?sanction.action,
      "sanction action has no Matrix primitive; skipping B-publish"
    );
    return Ok(());
  };

  // 2. Resolve subject pseudonym (ADR-015). Non-person targets should have been
  //    filtered by the T5 spawn guard, but guard defensively.
  let Some(target_person_id) = sanction.target_person_id else {
    tracing::debug!("sanction has no target_person_id; skipping B-publish");
    return Ok(());
  };
  let subject =
    actor_pseudonym_helper::get_or_create(&mut ctx.pool(), target_person_id).await?;

  // 3. Read active subscribers.
  let subscribers: Vec<SanctionSubscriber> = {
    let mut pool = ctx.pool();
    let conn = &mut get_conn(&mut pool).await?;
    sanction_subscriber_dsl::table
      .filter(sanction_subscriber_dsl::active.eq(true))
      .load::<SanctionSubscriber>(conn)
      .await?
  };

  if subscribers.is_empty() {
    tracing::debug!("no active sanction subscribers; skipping B-publish");
    return Ok(());
  }

  // 4. Resolve governance_log entry hash for the sanction_created entry.
  //    Use sql_query to filter on the JSONB payload's case_id field.
  let governance_log_entry_hash: String = {
    let mut pool = ctx.pool();
    let conn = &mut get_conn(&mut pool).await?;
    let row: Option<EntryHashRow> = diesel::sql_query(
      "SELECT entry_hash \
       FROM governance_log \
       WHERE entry_kind = $1 AND (payload->>'case_id')::INT = $2 \
       ORDER BY id DESC LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(ENTRY_KIND_SANCTION_CREATED)
    .bind::<diesel::sql_types::Integer, _>(sanction.case_id.0)
    .get_result::<EntryHashRow>(conn)
    .await
    .optional()?;
    match row {
      Some(r) => hex::encode(&r.entry_hash),
      None => {
        return Err(LemmyError::from(anyhow::anyhow!(
          "sanction-created governance_log entry not found for case_id {}",
          sanction.case_id.0
        )))
      }
    }
  };

  // 5. Build payload and POST to each subscriber.
  let payload = SanctionEventPayload {
    sanction_kind,
    case_id: i64::from(sanction.case_id.0),
    subject_actor_pseudonym: subject.clone(),
    effective_from: sanction.starts_at,
    effective_until: sanction.ends_at,
    governance_log_entry_hash,
  };
  // Best-effort outbound: unset secret → empty bearer → subscribers reject (non-gating,
  // mirrors the publisher's log-and-continue delivery contract). Not a hard fault here.
  #[expect(clippy::disallowed_methods)]
  let secret = std::env::var("BRIDGE_CALLBACK_SECRET").unwrap_or_default();
  let client = reqwest::Client::builder()
    .connect_timeout(std::time::Duration::from_secs(10))
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .map_err(|e| LemmyError::from(anyhow::anyhow!("failed to build HTTP client: {e}")))?;
  let mut any_ok = false;

  for sub in &subscribers {
    match client
      .post(&sub.callback_url)
      .header("Authorization", format!("Bearer {secret}"))
      .json(&payload)
      .send()
      .await
    {
      Ok(resp) if resp.status().is_success() => {
        any_ok = true;
        tracing::debug!(url = %sub.callback_url, "sanction event delivered");
      }
      Ok(resp) => {
        tracing::warn!(
          url = %sub.callback_url,
          status = %resp.status(),
          "sanction event delivery failed"
        );
      }
      Err(e) => {
        tracing::warn!(url = %sub.callback_url, error = %e, "sanction event delivery error");
      }
    }
  }

  // 6. Record the sanction_event row and governance log atomically (ADR-008).
  let kind = if any_ok {
    governance_log::ENTRY_KIND_SANCTION_PUBLISHED
  } else {
    governance_log::ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED
  };

  let mut pool = ctx.pool();
  let conn = &mut get_conn(&mut pool).await?; // FRESH conn — NOT the vote tx (R8)
  conn
    .run_transaction(async |conn| {
      let event_form = SanctionEventInsertForm {
        sanction_id: sanction.id,
        sanction_kind: payload.sanction_kind,
        subject_actor_pseudonym: subject.clone(),
        effective_from: sanction.starts_at,
        effective_until: sanction.ends_at,
        governance_log_entry_hash: payload.governance_log_entry_hash.clone(),
      };
      insert_into(sanction_event_dsl::table)
        .values(&event_form)
        .execute(conn)
        .await?;

      // Reborrow: `append`'s own run_transaction becomes a SAVEPOINT of THIS tx.
      governance_log::append(
        &mut (&mut *conn).into(),
        kind,
        serde_json::json!({
          "sanction_id": sanction.id.0,
          "sanction_kind": &payload.sanction_kind,
          "subscriber_count": subscribers.len(),
          "subject_actor_pseudonym": &subject,
        }),
        Some(subject.clone()),
      )
      .await?;

      Ok(())
    })
    .await?;

  Ok(())
}

/// Idempotent startup seed: insert `callback_url` as the sole active subscriber
/// if not already present. Called from `lemmy_server` startup (T6, m2-late-2)
/// when `BRIDGE_SANCTION_CALLBACK_URL` is set.
#[cfg(feature = "full")]
pub async fn seed_sanction_subscriber(url: &str, pool: &mut DbPool<'_>) -> LemmyResult<()> {
  use lemmy_db_schema::source::governance::sanction_subscriber::SanctionSubscriberInsertForm;

  let conn = &mut get_conn(pool).await?;
  let form = SanctionSubscriberInsertForm {
    callback_url: url.to_string(),
    active: Some(true),
  };
  insert_into(sanction_subscriber_dsl::table)
    .values(&form)
    .on_conflict(sanction_subscriber_dsl::callback_url)
    .do_update()
    .set(sanction_subscriber_dsl::active.eq(true))
    .execute(conn)
    .await?;
  Ok(())
}
