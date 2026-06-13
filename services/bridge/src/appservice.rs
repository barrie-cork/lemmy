// Application-service HTTP endpoint handlers.
//
// Implements the three Matrix AS API endpoints Tuwunel pushes events to:
//   PUT  /_matrix/app/v1/transactions/{txnId}
//   GET  /_matrix/app/v1/users/{userId}
//   GET  /_matrix/app/v1/rooms/{roomAlias}
//
// Auth: every request verified against BridgeConfig.hs_token via an axum
// middleware that checks Authorization: Bearer <hs_token>, with fallback to
// the ?access_token= query param (Matrix spec §2.1).
//
// MIRROR: ruma-appservice-api 0.16 endpoint types target the ruma HTTP
// client machinery (IncomingRequest trait), not axum extractors. Bare axum
// extractors (Path, Json<serde_json::Value>) are used per brief §4.1.
// ruma-appservice-api is retained as a dep for outbound AS calls in later
// tasks (puppet registration in Task 10, relay in Task 11).

use std::sync::Arc;

use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;

use crate::{config::BridgeConfig, link_handler, provision, relay, room_provisioner, sanction_handler};

pub struct AppState {
    pub config: Arc<BridgeConfig>,
    pub puppet_map: Arc<crate::puppet::PuppetMap>,
    pub http_client: reqwest::Client,
    pub relay_enabled: Arc<std::sync::atomic::AtomicBool>,
    pub oq009_reveal_threshold: Arc<std::sync::atomic::AtomicI64>,
    /// Path to the SQLite database file for bridge_room state.
    pub bridge_db_path: String,
}

/// Minimal representation of a Tuwunel transaction body.
/// `events` defaults to empty if absent (Matrix spec allows sparse bodies).
/// Unknown fields are silently ignored (serde default).
#[derive(Debug, Deserialize)]
struct PushEventsBody {
    #[serde(default)]
    events: Vec<Value>,
}

/// Middleware: verify Authorization: Bearer <hs_token> on every AS request.
/// Falls back to ?access_token= query param (Matrix spec §2.1).
/// Rejects with 401 + {"errcode":"M_FORBIDDEN","error":"Invalid token"}.
async fn hs_token_auth(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    // Authorization: Bearer <token>
    let bearer: Option<String> = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(str::to_owned);

    // Fallback: ?access_token=<token>
    let query_token: Option<String> = request.uri().query().and_then(|q| {
        q.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            (k == "access_token").then_some(v.to_owned())
        })
    });

    let valid = bearer
        .or(query_token)
        .map(|t| t == state.config.hs_token)
        .unwrap_or(false);

    if valid {
        next.run(request).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "errcode": "M_FORBIDDEN",
                "error": "Invalid token"
            })),
        )
            .into_response()
    }
}

/// PUT /_matrix/app/v1/transactions/{txnId}
/// Tuwunel pushes all events destined for this AS here.
/// Relays inbound Matrix DMs to Brehon via relay::handle_inbound (Task 11).
/// Returns 200 immediately with no relay when soft-pause is active.
async fn handle_transactions(
    State(state): State<Arc<AppState>>,
    Path(txn_id): Path<String>,
    Json(body): Json<PushEventsBody>,
) -> Response {
    if !state
        .relay_enabled
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        tracing::info!("relay paused — messaging_enabled=false");
        return (StatusCode::OK, Json(serde_json::json!({}))).into_response();
    }
    tracing::info!(
        txn_id = %txn_id,
        event_count = body.events.len(),
        "received AS transaction"
    );
    if let Err(e) = relay::handle_inbound(&body.events, &state.config, &state.http_client).await {
        tracing::warn!(err = %e, "relay::handle_inbound error");
    }
    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}

/// GET /_matrix/app/v1/users/{userId}
/// Called by Tuwunel to check whether this AS manages the given user.
/// Returns 200 {} (user is in AS namespace); puppet creation in Task 10.
async fn handle_query_user(Path(user_id): Path<String>) -> impl IntoResponse {
    tracing::info!(user_id = %user_id, "AS user query");
    // All users in the AS namespace from registration.yaml are handled here.
    // Puppet creation deferred to Task 10.
    (StatusCode::OK, Json(serde_json::json!({})))
}

/// GET /_matrix/app/v1/rooms/{roomAlias}
/// Called by Tuwunel to check whether this AS manages the given room alias.
/// Returns 200 {}; room provisioning wired in Task 12.
async fn handle_query_room(Path(room_alias): Path<String>) -> impl IntoResponse {
    tracing::info!(room_alias = %room_alias, "AS room alias query");
    // All room aliases in the AS namespace are handled here.
    // Provisioning deferred to Task 12.
    (StatusCode::OK, Json(serde_json::json!({})))
}

/// POST /admin/provision-room
/// Creates a Matrix community room via `provision::create_community_room`.
/// Body: `{"room_alias": "<alias>"}`. Returns `{"room_id": "<room_id>"}` on success.
/// cr-009: returns 403 when bridge is in soft-pause (relay_enabled=false).
/// cr-010: returns 400 when room_alias is absent or empty.
async fn handle_provision_room(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    // cr-009: capability check — refuse provisioning when bridge is in soft-pause
    if !state
        .relay_enabled
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "bridge is in soft-pause; provisioning disabled"})),
        )
            .into_response();
    }

    // cr-010: room_alias is required and must be non-empty
    let room_alias = match body
        .get("room_alias")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        Some(a) => a.to_owned(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "room_alias is required"})),
            )
                .into_response();
        }
    };

    match provision::create_community_room(&state.config, &room_alias).await {
        Ok(room_id) => (
            StatusCode::OK,
            Json(serde_json::json!({"room_id": room_id})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(err = %e, "provision failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

/// POST /brehon/room-event
/// Accepts a BridgeNotifyPayload envelope; dispatches jury room provisioning
/// fire-and-forget for CaseTransition variants. Returns 200 immediately (R3).
async fn handle_room_event(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<room_provisioner::RoomEventPayload>,
) -> Response {
    // Bearer auth — BRIDGE_CALLBACK_SECRET (Brehon→bridge), NOT hs_token.
    // Mirrors sanction_handler::handle_sanction_event. This route is added
    // OUTSIDE the hs_token_auth layer in router().
    let expected = &state.config.bridge_callback_secret;
    let provided = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(str::to_owned);
    if provided.as_deref() != Some(expected.as_str()) {
        tracing::warn!("room-event: unauthorized (bad or missing Bearer)");
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "unauthorized" })),
        )
            .into_response();
    }

    if let room_provisioner::RoomEventPayload::CaseTransition(event) = payload {
        tracing::info!(case_id = event.case_id, "room-event received");
        tokio::spawn(room_provisioner::handle_transition(state, event));
    }
    Json(serde_json::json!({})).into_response()
}

/// Build the AS transaction router with hs_token auth middleware applied to
/// all five endpoints.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        // axum 0.8 capture-group syntax is `{param}` (the 0.7 `:param` form
        // panics at router-build time: "Path segments must not start with `:`").
        .route(
            "/_matrix/app/v1/transactions/{txn_id}",
            put(handle_transactions),
        )
        .route("/_matrix/app/v1/users/{user_id}", get(handle_query_user))
        .route(
            "/_matrix/app/v1/rooms/{room_alias}",
            get(handle_query_room),
        )
        .route("/admin/provision-room", post(handle_provision_room))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            hs_token_auth,
        ))
        // /brehon/room-event + /brehon/sanction-event are Brehon→bridge calls
        // (NOT Tuwunel→bridge), so they authenticate with BRIDGE_CALLBACK_SECRET
        // inline, NOT the Matrix hs_token. Added AFTER route_layer so they do
        // NOT inherit hs_token_auth.
        .route("/brehon/room-event", post(handle_room_event))
        .route(
            "/brehon/sanction-event",
            post(sanction_handler::handle_sanction_event),
        )
        .route("/brehon/link-claim", post(link_handler::handle_link_claim))
        .with_state(state)
}
