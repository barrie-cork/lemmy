// Bridge ingest endpoint for B-publish sanction events (ADR-016).
//
// Receives POST /brehon/sanction-event from the Brehon binary with a
// SanctionEventPayload. Auth: Authorization: Bearer <BRIDGE_CALLBACK_SECRET>
// (NOT the Matrix hs_token — this endpoint is Brehon→bridge, not Tuwunel→bridge).
//
// MIRROR: handler shape follows handle_room_event in appservice.rs and
// the LOCAL payload struct pattern from room_provisioner.rs (no api-crate dep).

use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::appservice::AppState;
use crate::bridge_room;

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
        let leap = if y.is_multiple_of(400) { 1 } else if y.is_multiple_of(100) { 0 } else if y.is_multiple_of(4) { 1 } else { 0 };
        let dy = 365 + leap;
        if d < dy { break; }
        d -= dy;
        y += 1;
    }
    let leap = if y.is_multiple_of(400) { 1u64 } else if y.is_multiple_of(100) { 0 } else if y.is_multiple_of(4) { 1 } else { 0 };
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
    pub case_id: i64,
    /// actor_pseudonym.pseudonym — never person.name or local_user.email (ADR-015).
    pub subject_actor_pseudonym: String,
    pub effective_from: String,
    /// Part of the wire payload (Lemmy sends it); the handler applies a
    /// power-level change without an expiry timer yet, so it is not read here.
    #[allow(dead_code)]
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
/// On success: looks up the subject's jury rooms by case_id, computes
/// room-relative Matrix power-level overrides, applies them best-effort
/// across all rooms, and returns 200 with SanctionEventResponse.
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

    tracing::info!(
        subject = %payload.subject_actor_pseudonym,
        sanction_kind = %payload.sanction_kind,
        effective_from = %payload.effective_from,
        governance_log_entry_hash = %payload.governance_log_entry_hash,
        "sanction-event received"
    );

    // 2. Open bridge room DB — infra error → 500 (not 200 applied:false).
    let conn = match bridge_room::open(&state.bridge_db_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(err = %e, case_id = payload.case_id, "bridge_room::open failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "bridge_room open failed"})),
            )
                .into_response();
        }
    };

    // 3. Look up all rooms for this case.
    let rooms = match bridge_room::lookup_by_case(&conn, payload.case_id) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(err = %e, case_id = payload.case_id, "lookup_by_case failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "lookup_by_case failed"})),
            )
                .into_response();
        }
    };

    // 4. No rooms → 200 applied:false BEFORE ensure_puppet (load-bearing for Task 5 dep-free tests).
    if rooms.is_empty() {
        let now = now_rfc3339();
        return (
            StatusCode::OK,
            Json(SanctionEventResponse {
                applied: false,
                reason: format!("no rooms found for case_id={}", payload.case_id),
                applied_at: now,
            }),
        )
            .into_response();
    }

    // 5. Resolve puppet mxid for the subject.
    let mxid = match state.puppet_map.ensure_puppet(&payload.subject_actor_pseudonym).await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!(
                err = %e,
                subject = %payload.subject_actor_pseudonym,
                "ensure_puppet failed"
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "ensure_puppet failed"})),
            )
                .into_response();
        }
    };

    // 6. Per-room loop — best-effort: one room failure MUST NOT abort others.
    let rooms_found = rooms.len();
    let mut rooms_applied = 0usize;
    let mut rooms_failed = 0usize;
    let mut last_reason_code = "no_rooms_applied";

    for (_room_type, room_id) in &rooms {
        // GET current power levels.
        let mut content = match get_power_levels(&state, room_id).await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(err = %e, room_id = %room_id, "get_power_levels failed — skipping room");
                rooms_failed += 1;
                continue;
            }
        };

        // Compute room-relative override from fetched state.
        let (level, reason_code) = compute_power_override(&payload.sanction_kind, &content);
        last_reason_code = reason_code;

        // GET→merge→PUT full content (never fragment PUT — wipes events_default etc).
        if let Some(obj) = content.as_object_mut() {
            let users = obj.entry("users").or_insert_with(|| serde_json::json!({}));
            if let Some(users_map) = users.as_object_mut() {
                users_map.insert(mxid.to_string(), serde_json::json!(level));
            }
        }

        // PUT the merged full content back.
        match put_power_levels(&state, room_id, &content).await {
            Ok(()) => {
                tracing::info!(
                    room_id = %room_id,
                    mxid = %mxid,
                    level = level,
                    reason_code = reason_code,
                    "power level applied"
                );
                rooms_applied += 1;
            }
            Err(e) => {
                tracing::warn!(err = %e, room_id = %room_id, "put_power_levels failed — skipping room");
                rooms_failed += 1;
            }
        }
    }

    // 7. Build response.
    let now = now_rfc3339();
    let applied = rooms_applied > 0;
    let reason = format!(
        "rooms_found={rooms_found} rooms_applied={rooms_applied} rooms_failed={rooms_failed}; {last_reason_code}"
    );
    (
        StatusCode::OK,
        Json(SanctionEventResponse {
            applied,
            reason,
            applied_at: now,
        }),
    )
        .into_response()
}

/// Compute the room-relative power override per sanction_kind.
/// `content` is the fetched m.room.power_levels object.
/// Returns (target_level, reason_code).
fn compute_power_override(sanction_kind: &str, content: &serde_json::Value) -> (i64, &'static str) {
    let events_default = content.get("events_default").and_then(|v| v.as_i64()).unwrap_or(0);
    let msg_threshold = content
        .get("events")
        .and_then(|e| e.get("m.room.message"))
        .and_then(|v| v.as_i64())
        .unwrap_or(events_default);
    let voice_threshold = content
        .get("events")
        .and_then(|e| {
            e.get("m.call.member")
                .or_else(|| e.get("org.matrix.msc3401.call.member"))
        })
        .and_then(|v| v.as_i64());
    match sanction_kind {
        // Silence on the message channel (one below the send threshold).
        "ban" | "mute" | "prevent_post" => (msg_threshold - 1, "power_level_reduced_below_post_threshold"),
        // Voice channel if a voice threshold exists, else conservative fallback.
        "mute_voice" => (
            voice_threshold.unwrap_or(events_default) - 1,
            "voice_power_reduced_fallback_post_threshold",
        ),
        // No native primitive: reduce posting, flag that redaction/reach is not enforced.
        "hide_content" => (msg_threshold - 1, "redaction_not_available_in_m2_late_2"),
        "restrict_reach" => (msg_threshold - 1, "restrict_reach_translated_to_power_level_reduction"),
        _ => (msg_threshold - 1, "unknown_sanction_kind_default_post_reduction"),
    }
}

/// GET /_matrix/client/v3/rooms/{room_id}/state/m.room.power_levels
/// Mirror: room_provisioner.rs:527-581 bearer_auth + URL-encoded room id + state.http_client.
pub(crate) async fn get_power_levels(state: &AppState, room_id: &str) -> Result<serde_json::Value> {
    let encoded_room_id = room_id.replace(':', "%3A");
    let url = format!(
        "{}/_matrix/client/v3/rooms/{}/state/m.room.power_levels",
        state.config.tuwunel_url, encoded_room_id
    );
    let json = state
        .http_client
        .get(&url)
        .bearer_auth(&state.config.as_token)
        .send()
        .await
        .context("GET /state/m.room.power_levels failed")?
        .error_for_status()
        .context("GET /state/m.room.power_levels returned non-2xx")?
        .json::<serde_json::Value>()
        .await
        .context("GET /state/m.room.power_levels body not valid JSON")?;
    Ok(json)
}

/// PUT /_matrix/client/v3/rooms/{room_id}/state/m.room.power_levels
/// Replaces the entire content — always PUT the full merged object (Pattern 10.3 GOTCHA).
pub(crate) async fn put_power_levels(
    state: &AppState,
    room_id: &str,
    content: &serde_json::Value,
) -> Result<()> {
    let encoded_room_id = room_id.replace(':', "%3A");
    let url = format!(
        "{}/_matrix/client/v3/rooms/{}/state/m.room.power_levels",
        state.config.tuwunel_url, encoded_room_id
    );
    state
        .http_client
        .put(&url)
        .bearer_auth(&state.config.as_token)
        .json(content)
        .send()
        .await
        .context("PUT /state/m.room.power_levels failed")?
        .error_for_status()
        .context("PUT /state/m.room.power_levels returned non-2xx")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicI64},
    };
    use axum::{
        extract::State,
        http::{header, HeaderMap, StatusCode},
        Json,
    };

    use crate::{appservice::AppState, config::BridgeConfig, puppet::PuppetMap};
    use super::{SanctionEventPayload, handle_sanction_event};

    fn make_config(secret: &str) -> Arc<BridgeConfig> {
        Arc::new(BridgeConfig {
            tuwunel_url: "http://localhost:8448".to_string(),
            as_token: "test-as-token".to_string(),
            hs_token: "test-hs-token".to_string(),
            bridge_port: 9999,
            brehon_read_url: "http://localhost:9000/read".to_string(),
            brehon_notify_url: "http://localhost:9000/notify".to_string(),
            brehon_room_event_url: "http://localhost:9000/room-event".to_string(),
            bridge_callback_secret: secret.to_string(),
            legal_contact_mxid: "@legal:localhost".to_string(),
            matrix_server_name: "localhost".to_string(),
            brehon_signing_pubkey: "test-pubkey".to_string(),
            bridge_signing_key: "test-signing-key".to_string(),
            brehon_link_confirm_url: "http://localhost:9000/link".to_string(),
            livekit_url: None,
            livekit_api_key: None,
            livekit_api_secret: None,
        })
    }

    fn make_state(secret: &str, db_path: &str) -> Arc<AppState> {
        let config = make_config(secret);
        Arc::new(AppState {
            puppet_map: PuppetMap::new(config.clone()),
            config,
            http_client: reqwest::Client::new(),
            relay_enabled: Arc::new(AtomicBool::new(false)),
            oq009_reveal_threshold: Arc::new(AtomicI64::new(0)),
            bridge_db_path: db_path.to_string(),
        })
    }

    fn test_payload(case_id: i64) -> SanctionEventPayload {
        SanctionEventPayload {
            sanction_kind: "ban".to_string(),
            case_id,
            subject_actor_pseudonym: "test-actor".to_string(),
            effective_from: "2026-01-01T00:00:00Z".to_string(),
            effective_until: None,
            governance_log_entry_hash: "deadbeef".to_string(),
        }
    }

    /// Test 1: bad bearer → 401, no Matrix call (auth check returns before any DB/network op).
    #[tokio::test]
    async fn test_bad_bearer_returns_401() {
        let state = make_state("correct-secret", "unused-auth-test.db");
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            "Bearer wrong-secret".parse().unwrap(),
        );
        let resp = handle_sanction_event(State(state), headers, Json(test_payload(1))).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test 2: no rooms for case → 200 applied:false, no Matrix call.
    /// Per Pattern 10.4 ordering, ensure_puppet is never reached in this path.
    #[tokio::test]
    async fn test_no_rooms_returns_200_applied_false() {
        let db_path = std::env::temp_dir()
            .join(format!("bridge-test-norooms-{}.db", std::process::id()))
            .to_string_lossy()
            .to_string();
        // Open (creates schema) then drop — no rows inserted; schema persists on disk.
        let _ = crate::bridge_room::open(&db_path).expect("create temp db");

        let state = make_state("test-secret", &db_path);
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            "Bearer test-secret".parse().unwrap(),
        );
        let resp =
            handle_sanction_event(State(state), headers, Json(test_payload(999))).await;

        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        let _ = std::fs::remove_file(&db_path);

        assert_eq!(status, StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            !body["applied"].as_bool().unwrap_or(true),
            "applied must be false when no rooms found"
        );
        assert!(
            body["reason"].as_str().unwrap_or("").contains("no rooms"),
            "reason must mention 'no rooms', got: {}",
            body["reason"]
        );
    }

    /// Test 3: compute_power_override pure-function cases — no I/O.
    #[test]
    fn test_compute_power_override_cases() {
        let content = serde_json::json!({
            "events_default": 0,
            "events": { "m.room.message": 0 }
        });

        let (level, reason) = super::compute_power_override("ban", &content);
        assert_eq!(level, -1);
        assert_eq!(reason, "power_level_reduced_below_post_threshold");

        let (level, reason) = super::compute_power_override("mute", &content);
        assert_eq!(level, -1);
        assert_eq!(reason, "power_level_reduced_below_post_threshold");

        let (level, reason) = super::compute_power_override("prevent_post", &content);
        assert_eq!(level, -1);
        assert_eq!(reason, "power_level_reduced_below_post_threshold");

        // mute_voice with no voice threshold → events_default - 1 = -1
        let (level, reason) = super::compute_power_override("mute_voice", &content);
        assert_eq!(level, -1);
        assert_eq!(reason, "voice_power_reduced_fallback_post_threshold");

        let (level, reason) = super::compute_power_override("hide_content", &content);
        assert_eq!(level, -1);
        assert_eq!(reason, "redaction_not_available_in_m2_late_2");

        let (level, reason) = super::compute_power_override("restrict_reach", &content);
        assert_eq!(level, -1);
        assert_eq!(reason, "restrict_reach_translated_to_power_level_reduction");

        let (level, reason) = super::compute_power_override("unknown_kind", &content);
        assert_eq!(level, -1);
        assert_eq!(reason, "unknown_sanction_kind_default_post_reduction");
    }

    /// Test 4: live integration test (skipped in CI — requires docker-compose stack).
    #[tokio::test]
    #[ignore = "requires docker-compose stack"]
    async fn test_power_levels_applied_live() {
        // One room provisioned for a case → POST sanction event →
        // assert GET+PUT fired and applied:true.
        // Mirror: services/bridge/tests/room_provisioning.rs convention.
        todo!("implement against live docker-compose stack")
    }
}
