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
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;

use crate::config::BridgeConfig;

pub struct AppState {
    pub config: Arc<BridgeConfig>,
    pub puppet_map: Arc<crate::puppet::PuppetMap>,
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
/// Logs incoming events for now; relay to Brehon wired in Task 11.
async fn handle_transactions(
    Path(txn_id): Path<String>,
    Json(body): Json<PushEventsBody>,
) -> impl IntoResponse {
    tracing::info!(
        txn_id = %txn_id,
        event_count = body.events.len(),
        "received AS transaction"
    );
    // TODO(task 11): dispatch events through relay::handle_inbound
    (StatusCode::OK, Json(serde_json::json!({})))
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

/// Build the AS transaction router with hs_token auth middleware applied to
/// all three endpoints.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/_matrix/app/v1/transactions/:txn_id",
            put(handle_transactions),
        )
        .route("/_matrix/app/v1/users/:user_id", get(handle_query_user))
        .route(
            "/_matrix/app/v1/rooms/:room_alias",
            get(handle_query_room),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            hs_token_auth,
        ))
        .with_state(state)
}
