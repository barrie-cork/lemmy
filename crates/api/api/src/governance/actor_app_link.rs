//! B-actor portable-ID link handlers (ADR-016).
//!
//! Three endpoints:
//!  - `GET  /link`         — `link_actor`   — JWT-authed; mints a dual-signed claim
//!  - `POST /link/confirm` — `link_confirm` — bearer-authed (bridge→Brehon); creates
//!                           the `actor_app_link` row after verifying dual signatures
//!  - `POST /link/revoke`  — `revoke_link`  — JWT-authed; prospectively unlinks
//!
//! ADR-015: `person.id` is used ONLY as input to `actor_pseudonym_helper::get_or_create`.
//! It MUST NOT appear in any claim byte buffer, governance log payload, or wire response.
//! ADR-008: `link_confirm` and `revoke_link` MUST call `governance_log::append` before
//! returning success. Both writes are wrapped in `run_transaction`.

use crate::governance::{
  actor_pseudonym_helper,
  bridge_auth,
  governance_log,
};
use actix_web::{
  web::{Data, Json, Query},
  HttpRequest,
  HttpResponse,
};
use chrono::{Duration, Utc};
use diesel::{ExpressionMethods, QueryDsl, insert_into, update};
use diesel_async::RunQueryDsl;
use ed25519_dalek::{Signature, VerifyingKey};
use lemmy_api_common::governance::{LinkActorRequest, LinkClaimPayload, LinkConfirmRequest};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::newtypes::ActorPseudonymId;
use lemmy_db_schema::source::governance::actor_app_link::ActorAppLinkInsertForm;
use lemmy_db_schema::source::governance::governance_log::{
  sign_link_claim,
  ENTRY_KIND_ACTOR_APP_LINK_CREATED,
  ENTRY_KIND_ACTOR_APP_LINK_REVOKED,
};
use lemmy_db_schema_file::schema::{actor_app_link, actor_pseudonym};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use serde_json::json;
use uuid::Uuid;

/// GET /link — JWT-authed entry point.
///
/// Mints a Brehon-signed `LinkClaimPayload` and POSTs it to the app bridge
/// callback (fire-and-forget per ADR-012). Returns an ack immediately.
#[tracing::instrument(skip(context, local_user_view))]
pub async fn link_actor(
  data: Query<LinkActorRequest>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<HttpResponse> {
  check_local_user_valid(&local_user_view)?;

  // ADR-015: person.id used ONLY here, never in any payload or claim bytes
  let person_id = local_user_view.person.id;
  let brehon_actor_id = actor_pseudonym_helper::get_or_create(&mut context.pool(), person_id).await?;

  // Build deterministic claim bytes: pseudonym + app coords + nonce + expiry
  let nonce = Uuid::new_v4().to_string();
  let expires_at = Utc::now() + Duration::minutes(5);
  let claim_bytes_str = format!(
    "{}\n{}\n{}\n{}\n{}",
    brehon_actor_id,
    data.app_id,
    data.app_local_id,
    nonce,
    expires_at.to_rfc3339()
  );

  let brehon_signature = sign_link_claim(claim_bytes_str.as_bytes())?;

  let payload = LinkClaimPayload {
    brehon_actor_id,
    app_id: data.app_id.clone(),
    app_local_id: data.app_local_id.clone(),
    nonce,
    expires_at,
    brehon_signature,
  };

  // POST claim to bridge callback — fire-and-forget, log-and-swallow on error (ADR-012)
  let bridge_url = std::env::var("BRIDGE_LINK_CLAIM_URL").unwrap_or_default();
  if !bridge_url.is_empty() {
    let bridge_secret = std::env::var("BRIDGE_CALLBACK_SECRET").unwrap_or_default();
    let _ = context
      .client()
      .post(&bridge_url)
      .header("Authorization", format!("Bearer {bridge_secret}"))
      .json(&payload)
      .send()
      .await
      .map_err(|e| tracing::warn!("link_actor bridge POST failed: {e}"));
  }

  Ok(HttpResponse::Ok().json(json!({ "ok": true })))
}

/// POST /link/confirm — bearer-authed (bridge-to-Brehon callback).
///
/// Dual-signature gate (OQ-ADR016-03c): rejects if EITHER the Brehon ed25519
/// signature (already in the `LinkClaimPayload`) OR the app countersignature
/// (`app_signature` over `nonce\napp_local_id`) fails `verify_strict`.
///
/// INSERT + governance_log::append are wrapped in a single `run_transaction`
/// per `feedback_multi_write_handlers_need_transactions`.
#[tracing::instrument(skip(context, req))]
pub async fn link_confirm(
  Json(data): Json<LinkConfirmRequest>,
  context: Data<LemmyContext>,
  req: HttpRequest,
) -> LemmyResult<HttpResponse> {
  // Bearer auth FIRST — before any DB or crypto work
  bridge_auth::verify_bridge_secret(&req)?;

  // Verify app countersignature over (nonce\napp_local_id)
  let app_pubkey_hex = std::env::var("BRIDGE_LINK_APP_PUBKEY")
    .map_err(|_| LemmyError::from(LemmyErrorType::InvalidUrl))?;
  let app_pubkey_bytes = hex::decode(app_pubkey_hex.trim())
    .map_err(|_| LemmyError::from(LemmyErrorType::InvalidUrl))?;
  let app_pubkey_arr: [u8; 32] = app_pubkey_bytes
    .try_into()
    .map_err(|_| LemmyError::from(LemmyErrorType::InvalidUrl))?;
  let verifying_key =
    VerifyingKey::from_bytes(&app_pubkey_arr).map_err(|_| LemmyError::from(LemmyErrorType::InvalidUrl))?;

  let signed_bytes = format!("{}\n{}", data.nonce, data.app_local_id);
  let sig_arr: [u8; 64] = data
    .app_signature
    .as_slice()
    .try_into()
    .map_err(|_| LemmyError::from(LemmyErrorType::InvalidUrl))?;
  let app_sig = Signature::from_bytes(&sig_arr);

  // Dual-signature: hard reject if app countersig fails
  verifying_key
    .verify_strict(signed_bytes.as_bytes(), &app_sig)
    .map_err(|_| LemmyError::from(LemmyErrorType::NotLoggedIn))?;

  // Look up actor_pseudonym integer id from the UUID pseudonym string
  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let actor_pseudonym_id: ActorPseudonymId = actor_pseudonym::table
    .filter(actor_pseudonym::pseudonym.eq(&data.brehon_actor_id))
    .select(actor_pseudonym::id)
    .first::<ActorPseudonymId>(conn)
    .await
    .map_err(|_| LemmyError::from(LemmyErrorType::InvalidUrl))?;

  let app_id = data.app_id.clone();
  let app_local_id = data.app_local_id.clone();
  let brehon_actor_id = data.brehon_actor_id.clone();

  // INSERT + governance_log::append in a single transaction (ADR-008 + feedback_multi_write)
  conn
    .run_transaction(async |conn| {
      let form = ActorAppLinkInsertForm {
        brehon_actor_id: actor_pseudonym_id,
        app_id: app_id.clone(),
        app_local_id: app_local_id.clone(),
      };

      let rows_inserted = insert_into(actor_app_link::table)
        .values(&form)
        .on_conflict_do_nothing()
        .execute(conn)
        .await?;

      // ADR-008: only log on first creation — duplicate confirms are idempotent no-ops
      if rows_inserted > 0 {
        governance_log::append(
          &mut (&mut *conn).into(),
          ENTRY_KIND_ACTOR_APP_LINK_CREATED,
          json!({
            "brehon_actor_pseudonym": &brehon_actor_id,
            "app_id": &app_id,
            "app_local_id": &app_local_id,
          }),
          Some(brehon_actor_id.clone()),
        )
        .await?;
      }

      Ok(())
    })
    .await?;

  Ok(HttpResponse::Ok().json(json!({ "ok": true })))
}

/// POST /link/revoke — JWT-authed; prospectively unlinks a Brehon actor from an app.
///
/// Sets `revoked_at = now()` on the matching active link row. Historical
/// `actor_app_link_created` log entries are NOT rewritten (ADR-008 append-only).
///
/// UPDATE + governance_log::append are wrapped in a single `run_transaction`
/// per `feedback_multi_write_handlers_need_transactions`.
#[tracing::instrument(skip(context, local_user_view))]
pub async fn revoke_link(
  Json(data): Json<LinkActorRequest>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<HttpResponse> {
  check_local_user_valid(&local_user_view)?;

  // ADR-015: person.id used ONLY here, never in any payload
  let person_id = local_user_view.person.id;
  let brehon_actor_id = actor_pseudonym_helper::get_or_create(&mut context.pool(), person_id).await?;

  // Look up actor_pseudonym integer id
  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let actor_pseudonym_id: ActorPseudonymId = actor_pseudonym::table
    .filter(actor_pseudonym::pseudonym.eq(&brehon_actor_id))
    .select(actor_pseudonym::id)
    .first::<ActorPseudonymId>(conn)
    .await?;

  let app_id = data.app_id.clone();
  let app_local_id = data.app_local_id.clone();
  let pseudonym = brehon_actor_id.clone();

  // UPDATE + governance_log::append in a single transaction (ADR-008 + feedback_multi_write)
  conn
    .run_transaction(async |conn| {
      // Prospective revocation: set revoked_at on the active link (revoked_at IS NULL)
      update(
        actor_app_link::table
          .filter(actor_app_link::brehon_actor_id.eq(actor_pseudonym_id))
          .filter(actor_app_link::app_id.eq(&app_id))
          .filter(actor_app_link::app_local_id.eq(&app_local_id))
          .filter(actor_app_link::revoked_at.is_null()),
      )
      .set(actor_app_link::revoked_at.eq(Utc::now()))
      .execute(conn)
      .await?;

      // ADR-008: log revocation before return (even if 0 rows updated — idempotent log)
      governance_log::append(
        &mut (&mut *conn).into(),
        ENTRY_KIND_ACTOR_APP_LINK_REVOKED,
        json!({
          "brehon_actor_pseudonym": &pseudonym,
          "app_id": &app_id,
          "app_local_id": &app_local_id,
        }),
        Some(pseudonym.clone()),
      )
      .await?;

      Ok(())
    })
    .await?;

  Ok(HttpResponse::Ok().json(json!({ "ok": true })))
}
