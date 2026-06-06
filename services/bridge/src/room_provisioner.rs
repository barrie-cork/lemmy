// Jury room provisioner for governance-triggered case transitions.
//
// C2.1 path: when a CaseTransitionEvent arrives with juror_pseudonyms,
// provision a Matrix jury room and invite each pseudonymous juror puppet.
// Called fire-and-forget via tokio::spawn from the /brehon/room-event handler.
// All Matrix transport errors are logged and swallowed (ADR-012).
// Juror identities are consumed from event.juror_pseudonyms only — never
// resolved from DB (ADR-015).

use std::sync::{Arc, atomic::Ordering};

use anyhow::Context;
use serde::Deserialize;

use crate::{appservice::AppState, bridge_room, provision};

/// Bridge-local mirror of the governance.rs CaseTransitionEvent wire shape.
/// Unknown fields are silently ignored (serde default).
#[derive(Debug, Clone, Deserialize)]
pub struct CaseTransitionEvent {
    pub case_id: i32,
    #[serde(default)]
    pub juror_pseudonyms: Vec<String>,
}

/// Bridge-local discriminated union for POST /brehon/room-event.
/// Mirrors the BridgeNotifyPayload enum in governance.rs (serde `type_` tag).
#[derive(Debug, Deserialize)]
#[serde(tag = "type_", rename_all = "snake_case")]
pub enum RoomEventPayload {
    PrivateMessage(serde_json::Value),
    CaseTransition(CaseTransitionEvent),
}

/// C2.1 jury room provisioning path.
///
/// Triggered when a CaseTransitionEvent with juror_pseudonyms arrives.
/// Runs inside a tokio::spawn — never blocks the HTTP response (R3).
pub async fn handle_transition(state: Arc<AppState>, event: CaseTransitionEvent) {
    // (a) soft-pause gate
    if !state.relay_enabled.load(Ordering::Relaxed) {
        tracing::debug!(case_id = event.case_id, "relay paused — skipping room provisioning");
        return;
    }

    // (b) idempotency check — skip if jury room already exists
    {
        let conn = match bridge_room::open(&state.bridge_db_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::open failed");
                return;
            }
        };
        match bridge_room::lookup(&conn, event.case_id as i64, "jury") {
            Ok(Some(existing_id)) => {
                tracing::debug!(
                    case_id = event.case_id,
                    matrix_room_id = %existing_id,
                    "jury room already exists — idempotent skip"
                );
                return;
            }
            Ok(None) => {}
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::lookup failed");
                return;
            }
        }
        // conn dropped here before await points
    }

    // (c) provision the jury room
    let room_alias = format!("jury-case-{}", event.case_id);
    let room_id = match provision::create_community_room(&state.config, &room_alias).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(err = %e, case_id = event.case_id, "create_community_room failed");
            return;
        }
    };

    // (d) invite each juror puppet
    if event.juror_pseudonyms.is_empty() {
        tracing::warn!(
            case_id = event.case_id,
            "juror_pseudonyms is empty — skipping invite loop (ADR-012)"
        );
    } else {
        for pseudonym in &event.juror_pseudonyms {
            let mxid = match state.puppet_map.ensure_puppet(pseudonym).await {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(
                        err = %e,
                        pseudonym = %pseudonym,
                        "ensure_puppet failed — skipping juror"
                    );
                    continue;
                }
            };
            let suffix = pseudonym.split('/').next_back().unwrap_or(pseudonym.as_str());
            tracing::info!(
                mxid = %mxid,
                juror_label = %format!("Juror-{suffix}"),
                "inviting juror to jury room"
            );
            if let Err(e) = invite_to_room(&*state, &room_id, &mxid).await {
                tracing::warn!(
                    err = %e,
                    mxid = %mxid,
                    room_id = %room_id,
                    "invite failed — swallowing (ADR-012)"
                );
            }
        }
    }

    // (e) persist the jury room record
    {
        let conn = match bridge_room::open(&state.bridge_db_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::open for upsert failed");
                return;
            }
        };
        if let Err(e) = bridge_room::upsert(
            &conn,
            event.case_id as i64,
            "jury",
            &room_id,
            "active",
            None,
        ) {
            tracing::error!(err = %e, case_id = event.case_id, "bridge_room::upsert failed");
        }
    }
}

/// POST /_matrix/client/v3/rooms/{roomId}/invite via the AS master token.
/// All errors are returned to the caller for log-and-swallow at the call site.
async fn invite_to_room(
    state: &AppState,
    room_id: &str,
    mxid: &str,
) -> anyhow::Result<()> {
    // Room IDs contain ':' which must be percent-encoded in path segments.
    let encoded_room_id = room_id.replace(':', "%3A");
    let url = format!(
        "{}/_matrix/client/v3/rooms/{}/invite",
        state.config.tuwunel_url, encoded_room_id
    );
    state
        .http_client
        .post(&url)
        .bearer_auth(&state.config.as_token)
        .json(&serde_json::json!({"user_id": mxid}))
        .send()
        .await
        .context("POST /_matrix/client/v3/rooms/.../invite failed")?
        .error_for_status()
        .context("invite returned non-2xx status")?;
    Ok(())
}
