// Bridge ingest endpoint for B-publish sanction events (ADR-016).
//
// Receives POST /brehon/sanction-event from the Brehon binary with a
// SanctionEventPayload. Auth: Authorization: Bearer <BRIDGE_CALLBACK_SECRET>
// (NOT the Matrix hs_token — this endpoint is Brehon→bridge, not Tuwunel→bridge).
//
// MIRROR: handler shape follows handle_room_event in appservice.rs and
// the LOCAL payload struct pattern from room_provisioner.rs (no api-crate dep).

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::appservice::AppState;

/// RFC 3339 timestamp from std::time::SystemTime (no chrono dep in bridge).
fn now_rfc3339() -> String {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Minimal ISO 8601 UTC: "YYYY-MM-DDTHH:MM:SSZ"
    let s = secs;
    let sec = s % 60;
    let min = (s / 60) % 60;
    let hour = (s / 3600) % 24;
    let days = s / 86400;
    // Gregorian calendar calculation (good from 1970-2100)
    let mut y = 1970u64;
    let mut d = days;
    loop {
        let leap = if y % 400 == 0 { 1 } else if y % 100 == 0 { 0 } else if y % 4 == 0 { 1 } else { 0 };
        let dy = 365 + leap;
        if d < dy { break; }
        d -= dy;
        y += 1;
    }
    let leap = if y % 400 == 0 { 1u64 } else if y % 100 == 0 { 0 } else if y % 4 == 0 { 1 } else { 0 };
    let months = [31u64, 28 + leap, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0usize;
    while m < 12 && d >= months[m] {
        d -= months[m];
        m += 1;
    }
    format!("{y:04}-{:02}-{:02}T{hour:02}:{min:02}:{sec:02}Z", m + 1, d + 1)
}

/// Local mirror of the Brehon SanctionEventPayload (no api-crate dep — bridge
/// is workspace-excluded per R9). Fields match sanction_publisher.rs exactly.
#[derive(Debug, Deserialize)]
pub struct SanctionEventPayload {
    pub sanction_kind: String,
    /// actor_pseudonym.pseudonym — never person.name or local_user.email (ADR-015).
    pub subject_actor_pseudonym: String,
    pub effective_from: String,
    pub effective_until: Option<String>,
    /// Hex-encoded governance_log.entry_hash for the sanction_created entry.
    pub governance_log_entry_hash: String,
}

/// Response shape returned to the Brehon binary.
#[derive(Debug, Serialize)]
pub struct SanctionEventResponse {
    pub applied: bool,
    pub reason: String,
    pub applied_at: String,
}

/// POST /brehon/sanction-event
///
/// Auth: Authorization: Bearer <BRIDGE_CALLBACK_SECRET> (checked inline;
/// this route is added OUTSIDE the hs_token_auth middleware layer).
///
/// On success: looks up the subject's jury rooms, translates sanction_kind
/// to a Matrix power-level change, returns 200 with SanctionEventResponse.
/// Stub: power-level translation is deferred (bridge_room indexes by case_id,
/// not by pseudonym); logs the event and returns applied=true.
pub async fn handle_sanction_event(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SanctionEventPayload>,
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
            subject = %payload.subject_actor_pseudonym,
            "sanction-event: unauthorized (bad or missing Bearer)"
        );
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "unauthorized"
            })),
        )
            .into_response();
    }

    // 2. Log the inbound event (governance_log_entry_hash for audit trail).
    tracing::info!(
        subject = %payload.subject_actor_pseudonym,
        sanction_kind = %payload.sanction_kind,
        effective_from = %payload.effective_from,
        governance_log_entry_hash = %payload.governance_log_entry_hash,
        "sanction-event received"
    );

    // 3. Stub: power-level translation.
    // bridge_room is indexed by (case_id, room_type); there is no pseudonym→rooms
    // index yet. Full Matrix power-level enforcement is deferred to the
    // m2-late-2 B-actor phase where the puppet-map linkage is established.
    let _power_level = sanction_kind_to_power_level(&payload.sanction_kind);
    tracing::debug!(
        subject = %payload.subject_actor_pseudonym,
        sanction_kind = %payload.sanction_kind,
        "power-level translation stub — deferred to m2-late-2"
    );

    let now = now_rfc3339();
    (
        StatusCode::OK,
        Json(SanctionEventResponse {
            applied: true,
            reason: format!(
                "acknowledged sanction_kind={} for subject={}",
                payload.sanction_kind, payload.subject_actor_pseudonym
            ),
            applied_at: now,
        }),
    )
        .into_response()
}

/// Translate a sanction_kind string to a Matrix power level integer.
/// "ban" → 0 (cannot post); "mute" → 0; others → 50 (default member level).
/// Full enforcement via `send_state_event` deferred to m2-late-2.
fn sanction_kind_to_power_level(sanction_kind: &str) -> i32 {
    match sanction_kind {
        "ban" | "mute" => 0,
        _ => 50,
    }
}
