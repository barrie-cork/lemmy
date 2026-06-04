//! **Process-env safety constraint:** every test in this module mutates
//! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
//! Safety of those mutations is contingent on the Cargo runner flag
//! `--test-threads=1`. Running these tests with concurrent threads is
//! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
use crate::common::governance_fixtures;
use diesel::ExpressionMethods;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema::source::governance::federation_peer::{
  FederationPeerInsertForm, federation_inbox_check_peer_trust,
};
use lemmy_db_schema_file::InstanceId;
use lemmy_db_schema_file::enums::FederationPeerTrust;
use lemmy_db_schema_file::schema::{federation_peer, instance};
use lemmy_utils::error::LemmyResult;

async fn seed_federation_peer(
  conn: &mut AsyncPgConnection,
  domain: &str,
  trust: FederationPeerTrust,
) -> LemmyResult<InstanceId> {
  let instance_id: InstanceId = diesel::insert_into(instance::table)
    .values((
      instance::domain.eq(domain),
      instance::published_at.eq(diesel::dsl::now),
    ))
    .returning(instance::id)
    .get_result(conn)
    .await?;
  let form = FederationPeerInsertForm {
    instance_id,
    trust_level: Some(trust),
    added_by_actor: None,
    notes: None,
  };
  diesel::insert_into(federation_peer::table)
    .values(&form)
    .execute(conn)
    .await?;
  Ok(instance_id)
}

#[tokio::test(flavor = "multi_thread")]
async fn federation_peer_trust_lookup_returns_seeded_state() -> LemmyResult<()> {
  let (_container, _context, db_url) = governance_fixtures::bootstrap().await?;
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let _instance_id = seed_federation_peer(
    &mut conn,
    "allowlisted.test",
    FederationPeerTrust::Allowlisted,
  )
  .await?;
  let trust = federation_inbox_check_peer_trust("allowlisted.test", &mut conn).await?;
  assert_eq!(trust, FederationPeerTrust::Allowlisted);
  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn federation_peer_trust_lookup_returns_unknown_for_first_seen() -> LemmyResult<()> {
  let (_container, _context, db_url) = governance_fixtures::bootstrap().await?;
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let trust = federation_inbox_check_peer_trust("unknown-peer.test", &mut conn).await?;
  assert_eq!(trust, FederationPeerTrust::Unknown);
  Ok(())
}
