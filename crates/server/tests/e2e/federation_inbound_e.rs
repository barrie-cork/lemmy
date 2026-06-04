//! **Process-env safety constraint:** every test in this module mutates
//! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
//! Safety of those mutations is contingent on the Cargo runner flag
//! `--test-threads=1`. Running these tests with concurrent threads is
//! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
use crate::common::governance_fixtures;
use activitypub_federation::config::FederationConfig;
use activitypub_federation::traits::Activity as ActivityTrait;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_activities::protocol::governance::publish_sanction_notice::PublishSanctionNotice;
use lemmy_db_schema::source::governance::federation_peer::FederationPeerInsertForm;
use lemmy_db_schema_file::InstanceId;
use lemmy_db_schema_file::enums::FederationPeerTrust;
use lemmy_db_schema_file::schema::{
  federation_inbox_dropped_log, federation_peer, governance_config, instance,
  remote_sanction_notice,
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use testcontainers::{ContainerAsync, GenericImage};

async fn bootstrap_with_peer(
  domain: &str,
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
  let form = FederationPeerInsertForm {
    instance_id: InstanceId(peer_instance_id),
    trust_level: Some(FederationPeerTrust::Allowlisted),
    added_by_actor: None,
    notes: None,
  };
  diesel::insert_into(federation_peer::table)
    .values(&form)
    .execute(&mut conn)
    .await?;
  Ok((container, federation_config, db_url, InstanceId(peer_instance_id)))
}

async fn seed_storage_cap(conn: &mut AsyncPgConnection, cap: i64) -> LemmyResult<()> {
  diesel::insert_into(governance_config::table)
    .values((
      governance_config::scope.eq("instance"),
      governance_config::key.eq("federation.inbound.per_peer_storage_cap"),
      governance_config::value_type.eq("int"),
      governance_config::value_int.eq(Some(cap)),
      governance_config::valid_from.eq(diesel::dsl::now),
    ))
    .execute(conn)
    .await?;
  Ok(())
}

async fn preseed_sanction_notices(
  conn: &mut AsyncPgConnection,
  source_instance: &str,
  n: usize,
) -> LemmyResult<()> {
  for i in 0..n {
    diesel::sql_query(
      "INSERT INTO remote_sanction_notice \
       (source_instance, target_url, action, scope, summary, signature, published_at, received_at) \
       VALUES ($1, $2, 'FederationQuarantineRecommendation'::sanction_action, \
       'Instance'::sanction_scope, $3, $4, NOW(), NOW() - ($5 || ' seconds')::interval)",
    )
    .bind::<diesel::sql_types::Text, _>(source_instance)
    .bind::<diesel::sql_types::Text, _>(format!("https://{source_instance}/target/{i}"))
    .bind::<diesel::sql_types::Text, _>(format!("preseed sanction {i}"))
    .bind::<diesel::sql_types::Text, _>(format!(
      "https://{source_instance}/activity/preseed/{i}"
    ))
    .bind::<diesel::sql_types::Text, _>(format!("{}", 100 - i))
    .execute(conn)
    .await?;
  }
  Ok(())
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

#[tokio::test(flavor = "multi_thread")]
async fn storage_cap_holds_under_concurrent_receivers() -> LemmyResult<()> {
  let (_container, fed_cfg, db_url, _peer_id) =
    bootstrap_with_peer("concurrent.test").await?;

  let mut setup_conn = AsyncPgConnection::establish(&db_url).await?;
  seed_storage_cap(&mut setup_conn, 5).await?;
  preseed_sanction_notices(&mut setup_conn, "concurrent.test", 5).await?;

  let context = fed_cfg.to_request_data();

  let mut handles = Vec::with_capacity(8);
  for i in 0..8u32 {
    let context_i = context.reset_request_count();
    let handle = tokio::spawn(async move {
      let activity = build_sanction_notice_with_id(
        "concurrent.test",
        &format!("https://concurrent.test/activity/concurrent/{i}"),
      )?;
      ActivityTrait::receive(activity, &context_i).await
    });
    handles.push(handle);
  }
  for handle in handles {
    handle
      .await
      .map_err(|e| LemmyErrorType::Unknown(format!("join: {e}")))?? ;
  }

  let mut verify_conn = AsyncPgConnection::establish(&db_url).await?;
  let final_count: i64 = remote_sanction_notice::table
    .filter(remote_sanction_notice::source_instance.eq("concurrent.test"))
    .filter(remote_sanction_notice::admin_reviewed_at.is_null())
    .count()
    .get_result(&mut verify_conn)
    .await?;
  assert_eq!(
    final_count,
    5,
    "per-peer storage cap must hold under concurrent receivers (Race B regression)"
  );

  let drop_log_count: i64 = federation_inbox_dropped_log::table
    .filter(federation_inbox_dropped_log::source_instance.eq("concurrent.test"))
    .filter(federation_inbox_dropped_log::drop_reason.eq("storage_cap_evicted"))
    .count()
    .get_result(&mut verify_conn)
    .await?;
  assert_eq!(
    drop_log_count,
    8,
    "each concurrent receiver must fire exactly one eviction event (Race A regression)"
  );

  Ok(())
}
