// Bridge ingest endpoint for Brehon link-claim flow (ADR-015).
//
// Receives POST /brehon/link-claim from the Brehon binary with a
// LinkClaimPayload. Auth: Authorization: Bearer <BRIDGE_CALLBACK_SECRET>
// (NOT the Matrix hs_token — this is Brehon→bridge, not Tuwunel→bridge).
//
// MIRROR: handler shape follows handle_sanction_event in sanction_handler.rs
// (bearer check pattern, (StatusCode, Json).into_response(), AppState access).

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::app_actor_link;
use crate::appservice::AppState;

/// Incoming payload from Brehon's POST /link handler.
#[derive(Debug, Deserialize)]
pub struct LinkClaimPayload {
    /// Pseudonym UUID string (ADR-015) — NEVER a Lemmy person_id or username.
    pub brehon_actor_id: String,
    pub app_id: String,
    pub app_local_id: String,
    pub nonce: String,
    pub expires_at: String,
    /// ed25519 signature over claim_bytes (see reconstruct logic below).
    pub brehon_signature: Vec<u8>,
}

/// Body for the follow-up POST to brehon_link_confirm_url.
#[derive(Debug, Serialize)]
struct LinkConfirmRequest {
    brehon_actor_id: String,
    app_id: String,
    app_local_id: String,
    nonce: String,
    expires_at: String,
    brehon_signature: Vec<u8>,
    app_signature: Vec<u8>,
}

/// POST /brehon/link-claim
///
/// Auth: Authorization: Bearer <BRIDGE_CALLBACK_SECRET> (checked inline;
/// this route is added OUTSIDE the hs_token_auth middleware layer).
///
/// Steps:
///   1. Bearer auth
///   2. Verify Brehon's ed25519 sig on claim_bytes (verify_strict — no weak-sig)
///   3. Countersign with bridge's own ed25519 key
///   4. Persist app_actor_link mapping (ADR-015: brehon_actor_id is pseudonym UUID)
///   5. POST LinkConfirmRequest to brehon_link_confirm_url
///   6. Return 200 {"linked": true}
pub async fn handle_link_claim(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<LinkClaimPayload>,
) -> Response {
    // 1. Bearer auth — BRIDGE_CALLBACK_SECRET (not hs_token).
    let expected = &state.config.bridge_callback_secret;
    let provided = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(str::to_owned);

    if provided.as_deref() != Some(expected.as_str()) {
        tracing::warn!(
            brehon_actor_id = %payload.brehon_actor_id,
            "link-claim: unauthorized (bad or missing Bearer)"
        );
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "unauthorized" })),
        )
            .into_response();
    }

    // 2. Reconstruct claim_bytes — MUST match what Brehon signs in link_actor.
    let claim_bytes = format!(
        "{}\n{}\n{}\n{}\n{}",
        payload.brehon_actor_id,
        payload.app_id,
        payload.app_local_id,
        payload.nonce,
        payload.expires_at,
    )
    .into_bytes();

    // 3. Verify Brehon's ed25519 signature (verify_strict, not verify —
    //    re-point defence: reject sigs that require cofactor clearing).
    let verify_result: anyhow::Result<()> = (|| {
        let pubkey_bytes: [u8; 32] = hex::decode(&state.config.brehon_signing_pubkey)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("brehon_signing_pubkey wrong length"))?;
        let verifying_key = VerifyingKey::from_bytes(&pubkey_bytes)
            .map_err(|e| anyhow::anyhow!("brehon_signing_pubkey invalid: {e}"))?;
        let signature = Signature::try_from(payload.brehon_signature.as_slice())
            .map_err(|e| anyhow::anyhow!("brehon_signature invalid bytes: {e}"))?;
        verifying_key
            .verify_strict(&claim_bytes, &signature)
            .map_err(|e| anyhow::anyhow!("brehon sig verification failed: {e}"))?;
        Ok(())
    })();

    if let Err(e) = verify_result {
        tracing::warn!(err = %e, "link-claim: Brehon signature verification failed — upsert NOT called");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "brehon signature verification failed" })),
        )
            .into_response();
    }

    // 4. Countersign claim_bytes with the bridge's own ed25519 key.
    let bridge_signature: Vec<u8> = match (|| -> anyhow::Result<Vec<u8>> {
        let bridge_key_bytes: [u8; 32] = hex::decode(&state.config.bridge_signing_key)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("bridge_signing_key wrong length"))?;
        let bridge_signing_key = SigningKey::from_bytes(&bridge_key_bytes);
        Ok(bridge_signing_key.sign(&claim_bytes).to_bytes().to_vec())
    })() {
        Ok(sig) => sig,
        Err(e) => {
            tracing::error!(err = %e, "link-claim: countersign failed (infra error)");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "countersign failed" })),
            )
                .into_response();
        }
    };

    // 5. Persist the app_local_id ↔ brehon_actor_id mapping.
    //    ADR-015 (load-bearing): brehon_actor_id forwarded here is the pseudonym
    //    UUID string from the payload — NEVER a Lemmy person_id, username, or email.
    //    Brehon ensures this upstream; we assert the invariant in this comment.
    let upsert_result: anyhow::Result<()> = (|| {
        let conn = app_actor_link::open(&state.bridge_db_path)
            .map_err(|e| anyhow::anyhow!("app_actor_link::open failed: {e}"))?;
        app_actor_link::upsert(&conn, &payload.app_local_id, &payload.brehon_actor_id)
            .map_err(|e| anyhow::anyhow!("app_actor_link::upsert failed: {e}"))?;
        Ok(())
    })();

    if let Err(e) = upsert_result {
        tracing::error!(err = %e, "link-claim: DB upsert failed");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "db upsert failed" })),
        )
            .into_response();
    }

    // 6. POST LinkConfirmRequest to Brehon's confirm URL with bearer.
    let confirm_body = LinkConfirmRequest {
        brehon_actor_id: payload.brehon_actor_id.clone(),
        app_id: payload.app_id.clone(),
        app_local_id: payload.app_local_id.clone(),
        nonce: payload.nonce.clone(),
        expires_at: payload.expires_at.clone(),
        brehon_signature: payload.brehon_signature.clone(),
        app_signature: bridge_signature,
    };

    let post_result = state
        .http_client
        .post(&state.config.brehon_link_confirm_url)
        .bearer_auth(&state.config.bridge_callback_secret)
        .json(&confirm_body)
        .send()
        .await;

    if let Err(e) = post_result {
        tracing::error!(err = %e, "link-claim: POST to brehon_link_confirm_url failed");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "confirm POST to Brehon failed" })),
        )
            .into_response();
    }

    tracing::info!(
        brehon_actor_id = %payload.brehon_actor_id,
        app_id = %payload.app_id,
        app_local_id = %payload.app_local_id,
        "link-claim: linked successfully"
    );

    (StatusCode::OK, Json(serde_json::json!({ "linked": true }))).into_response()
}
