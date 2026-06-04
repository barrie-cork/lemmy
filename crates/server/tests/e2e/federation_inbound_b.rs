//! **Process-env safety constraint:** every test in this module mutates
//! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
//! Safety of those mutations is contingent on the Cargo runner flag
//! `--test-threads=1`. Running these tests with concurrent threads is
//! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
use crate::common::governance_fixtures;
use activitypub_federation::config::FederationConfig;
use activitypub_federation::traits::Activity as ActivityTrait;
use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_activities::protocol::governance::publish_label::PublishLabel;
use lemmy_apub_activities::protocol::governance::publish_sanction_notice::PublishSanctionNotice;
use lemmy_db_schema::source::governance::{
  federation_inbox_nonce::FederationInboxNonceInsertForm,
  federation_peer::FederationPeerInsertForm, remote_moderation_label::RemoteModerationLabel,
  remote_sanction_notice::RemoteSanctionNotice,
};
use lemmy_db_schema_file::InstanceId;
use lemmy_db_schema_file::enums::FederationPeerTrust;
use lemmy_db_schema_file::schema::{
  federation_inbox_dropped_log, federation_inbox_nonce, federation_peer, governance_config,
  governance_log, instance, remote_moderation_label, remote_sanction_notice,
};
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use testcontainers::{ContainerAsync, GenericImage};

async fn bootstrap_with_peer(
  domain: &str,
  trust: Option<FederationPeerTrust>,
) -> LemmyResult<(
  ContainerAsync<GenericImage>,
  FederationConfig<LemmyContext>,
  String,
  InstanceId,
)> {
  let (container, actix_context, db_url) = governance_fixtures::bootstrap().await?;
  let federation_config = FederationConfig::builder()
    .domain((**actix_context).settings().hostname.clone())
    .app_data((**actix_context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let peer_instance_id: i32 = diesel::insert_into(instance::table)
    .values((
      instance::domain.eq(domain),
      instance::published_at.eq(diesel::dsl::now),
    ))
    .returning(instance::id)
    .get_result(&mut conn)
    .await?;
  if let Some(t) = trust {
    let form = FederationPeerInsertForm {
      instance_id: InstanceId(peer_instance_id),
      trust_level: Some(t),
      added_by_actor: None,
      notes: None,
    };
    diesel::insert_into(federation_peer::table)
      .values(&form)
      .execute(&mut conn)
      .await?;
  }
  Ok((
    container,
    federation_config,
    db_url,
    InstanceId(peer_instance_id),
  ))
}

#[tokio::test(flavor = "multi_thread")]
async fn blocklisted_peer_returns_403() -> LemmyResult<()> {
  let (_container, fed_cfg, db_url, _peer_id) =
    bootstrap_with_peer("blocked.test", Some(FederationPeerTrust::Blocklisted)).await?;
  let context = fed_cfg.to_request_data();
  let activity = build_minimal_sanction_notice_activity("blocked.test")?;
  let result = ActivityTrait::receive(activity, &context).await;
  assert!(result.is_err(), "wrapper must reject Blocklisted peer");
  let err: LemmyError = result.err().unwrap();
  assert!(matches!(
    err.error_type,
    LemmyErrorType::FederationPeerBlocklisted
  ));
  assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let drop_rows: i64 = federation_inbox_dropped_log::table
    .filter(federation_inbox_dropped_log::source_instance.eq("blocked.test"))
    .filter(federation_inbox_dropped_log::drop_reason.eq("blocklisted"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(drop_rows, 1);
  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn per_peer_rate_limit_returns_429() -> LemmyResult<()> {
  let (_container, fed_cfg, db_url, _peer_id) =
    bootstrap_with_peer("rate-test.test", Some(FederationPeerTrust::Allowlisted)).await?;
  let context = fed_cfg.to_request_data();
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  // governance_config is append-history: UPDATE mutates the existing seed row;
  // the post-v1-federation-inbound-c reader (.order_by(valid_from.desc())) makes
  // INSERT-with-newer-valid_from also safe. This test uses UPDATE for historical
  // continuity; new override tests use INSERT (see appended_config_override_takes_effect_returns_429).
  diesel::sql_query(
    "UPDATE governance_config SET value_int = 2 \
     WHERE scope = 'instance' AND key = 'federation.inbound.per_peer_rate_per_hour'",
  )
  .execute(&mut conn)
  .await?;
  for i in 0..2 {
    let activity = build_unique_sanction_notice_activity("rate-test.test", i)?;
    ActivityTrait::receive(activity, &context).await?;
  }
  let activity3 = build_unique_sanction_notice_activity("rate-test.test", 2)?;
  let result = ActivityTrait::receive(activity3, &context).await;
  assert!(result.is_err());
  let err = result.err().unwrap();
  assert!(matches!(
    err.error_type,
    LemmyErrorType::FederationPeerRateLimitExceeded
  ));
  assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn appended_config_override_takes_effect_returns_429() -> LemmyResult<()> {
  let (_container, fed_cfg, db_url, _peer_id) =
    bootstrap_with_peer("override-test.test", Some(FederationPeerTrust::Allowlisted)).await?;
  let context = fed_cfg.to_request_data();
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  // governance_config is append-only — INSERT a newer-valid_from row to
  // override the seeded federation.inbound.per_peer_rate_per_hour (seed
  // value_int = 100 per migration 2026-04-18-000000-0000_add_governance_config).
  // Post-v1-federation-inbound-c, get_inbound_config_int reads the latest row;
  // the override cap=2 means the 3rd activity should 429.
  diesel::insert_into(governance_config::table)
    .values((
      governance_config::scope.eq("instance"),
      governance_config::key.eq("federation.inbound.per_peer_rate_per_hour"),
      governance_config::value_type.eq("int"),
      governance_config::value_int.eq(Some(2_i64)),
      governance_config::valid_from.eq(diesel::dsl::now),
    ))
    .execute(&mut conn)
    .await?;
  for i in 0..2 {
    let activity = build_unique_sanction_notice_activity("override-test.test", i)?;
    ActivityTrait::receive(activity, &context).await?;
  }
  let activity3 = build_unique_sanction_notice_activity("override-test.test", 2)?;
  let result = ActivityTrait::receive(activity3, &context).await;
  assert!(
    result.is_err(),
    "3rd activity must 429 against override cap=2"
  );
  let err = result.err().unwrap();
  assert!(matches!(
    err.error_type,
    LemmyErrorType::FederationPeerRateLimitExceeded
  ));
  assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
  // Optional but recommended: assert the drop log row landed.
  let drop_rows: i64 = federation_inbox_dropped_log::table
    .filter(federation_inbox_dropped_log::source_instance.eq("override-test.test"))
    .filter(federation_inbox_dropped_log::drop_reason.eq("rate_limit_peer"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(drop_rows, 1, "exactly one rate_limit_peer drop expected");
  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn replayed_activity_returns_409() -> LemmyResult<()> {
  let (_container, fed_cfg, db_url, _peer_id) =
    bootstrap_with_peer("replay-test.test", Some(FederationPeerTrust::Allowlisted)).await?;
  let context = fed_cfg.to_request_data();
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let nonce_form = FederationInboxNonceInsertForm {
    peer_instance: "replay-test.test".to_string(),
    activity_id: "https://replay-test.test/activities/create/1".to_string(),
  };
  diesel::insert_into(federation_inbox_nonce::table)
    .values(&nonce_form)
    .execute(&mut conn)
    .await?;
  let activity = build_sanction_notice_with_id(
    "replay-test.test",
    "https://replay-test.test/activities/create/1",
  )?;
  let result = ActivityTrait::receive(activity, &context).await;
  assert!(result.is_err());
  let err = result.err().unwrap();
  assert!(matches!(
    err.error_type,
    LemmyErrorType::FederationActivityReplayed
  ));
  assert_eq!(err.status_code(), StatusCode::CONFLICT);
  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn allowlisted_happy_path_persists_advisory_row() -> LemmyResult<()> {
  let (_container, fed_cfg, db_url, _peer_id) =
    bootstrap_with_peer("happy.test", Some(FederationPeerTrust::Allowlisted)).await?;
  let context = fed_cfg.to_request_data();
  let activity = build_minimal_sanction_notice_activity("happy.test")?;
  ActivityTrait::receive(activity, &context).await?;
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let rows: i64 = remote_sanction_notice::table
    .filter(remote_sanction_notice::source_instance.eq("happy.test"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(rows, 1);
  let advisory: RemoteSanctionNotice = remote_sanction_notice::table
    .filter(remote_sanction_notice::source_instance.eq("happy.test"))
    .select(RemoteSanctionNotice::as_select())
    .first(&mut conn)
    .await?;
  assert!(
    advisory.local_case_id.is_none(),
    "ADR-006: local_case_id MUST be NULL"
  );
  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn moderation_label_handler_persists_and_logs() -> LemmyResult<()> {
  let (_container, fed_cfg, db_url, _peer_id) =
    bootstrap_with_peer("label.test", Some(FederationPeerTrust::Allowlisted)).await?;
  let context = fed_cfg.to_request_data();
  let activity = build_minimal_publish_label_activity("label.test")?;
  ActivityTrait::receive(activity, &context).await?;
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let rows: i64 = remote_moderation_label::table
    .filter(remote_moderation_label::source_instance.eq("label.test"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(rows, 1);
  let label_row: RemoteModerationLabel = remote_moderation_label::table
    .filter(remote_moderation_label::source_instance.eq("label.test"))
    .select(RemoteModerationLabel::as_select())
    .first(&mut conn)
    .await?;
  assert!(
    label_row.local_case_id.is_none(),
    "ADR-006: local_case_id MUST be NULL"
  );
  let log_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("federation_label_received"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(log_count, 1);
  Ok(())
}

fn build_minimal_sanction_notice_activity(
  peer_domain: &str,
) -> LemmyResult<PublishSanctionNotice> {
  build_unique_sanction_notice_activity(peer_domain, 0)
}

fn build_unique_sanction_notice_activity(
  peer_domain: &str,
  seq: u32,
) -> LemmyResult<PublishSanctionNotice> {
  build_sanction_notice_with_id(
    peer_domain,
    &format!("https://{peer_domain}/activities/create/{seq}"),
  )
}

fn build_sanction_notice_with_id(
  peer_domain: &str,
  activity_id: &str,
) -> LemmyResult<PublishSanctionNotice> {
  let actor_url = format!("https://{peer_domain}/u/admin");
  let object_id = format!("https://{peer_domain}/objects/sanction/1");
  let target_url = format!("https://{peer_domain}/u/target");
  let val = serde_json::json!({
    "type": "Create",
    "actor": actor_url,
    "to": ["https://www.w3.org/ns/activitystreams#Public"],
    "cc": [],
    "id": activity_id,
    "object": {
      "type": "SanctionNotice",
      "id": object_id,
      "actor": actor_url,
      "target": target_url,
      "action": "federation_quarantine_recommendation",
      "scope": "federated_recommendation",
      "summary": "test sanction notice",
      "published": "2024-01-01T00:00:00Z"
    }
  });
  let activity: PublishSanctionNotice = serde_json::from_value(val)?;
  Ok(activity)
}

fn build_minimal_publish_label_activity(peer_domain: &str) -> LemmyResult<PublishLabel> {
  let actor_url = format!("https://{peer_domain}/u/admin");
  let object_id = format!("https://{peer_domain}/objects/label/1");
  let target_url = format!("https://{peer_domain}/u/target");
  let activity_url = format!("https://{peer_domain}/activities/create/1");
  let val = serde_json::json!({
    "type": "Create",
    "actor": actor_url,
    "to": ["https://www.w3.org/ns/activitystreams#Public"],
    "cc": [],
    "id": activity_url,
    "object": {
      "type": "ModerationLabel",
      "id": object_id,
      "actor": actor_url,
      "target": target_url,
      "label": "context-warning",
      "published": "2024-01-01T00:00:00Z"
    }
  });
  let activity: PublishLabel = serde_json::from_value(val)?;
  Ok(activity)
}
