// Room provisioner for governance-triggered case transitions.
//
// C2.1-C2.6 paths: when a CaseTransitionEvent arrives, route to the
// appropriate room provisioning path based on new_status and community_id.
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
    #[serde(default)]
    pub new_status: Option<String>,
    #[serde(default)]
    pub old_status: Option<String>,
    #[serde(default)]
    pub community_id: Option<i32>,
}

/// Bridge-local discriminated union for POST /brehon/room-event.
/// Mirrors the BridgeNotifyPayload enum in governance.rs (serde `type_` tag).
#[derive(Debug, Deserialize)]
#[serde(tag = "type_", rename_all = "snake_case")]
pub enum RoomEventPayload {
    PrivateMessage(serde_json::Value),
    CaseTransition(CaseTransitionEvent),
}

/// Dispatch room provisioning based on the case transition status.
///
/// Runs inside a tokio::spawn — never blocks the HTTP response (R3).
/// Emergency room is the latency-critical path (ADR-013 <2s) — checked first.
pub async fn handle_transition(state: Arc<AppState>, event: CaseTransitionEvent) {
    // (a) soft-pause gate
    if !state.relay_enabled.load(Ordering::Relaxed) {
        tracing::debug!(case_id = event.case_id, "relay paused — skipping room provisioning");
        return;
    }

    // Extract match conditions before moving event into sub-functions.
    let new_status = event.new_status.clone();
    let community_id = event.community_id;

    // Dispatch based on new_status; emergency checked first per ADR-013.
    match new_status.as_deref() {
        Some("emergency_remove") => provision_emergency_room(state, event).await,
        Some("jury_selection") => provision_jury_room(state, event).await,
        Some("appealed") => provision_appeal_room(state, event).await,
        Some("in_review") if community_id.is_some() => provision_spinout_room(state, event).await,
        Some("decided") | Some("closed") => provision_membership_mirror(state, event).await,
        _ if community_id.is_some() => provision_community_event_room(state, event).await,
        _ => {
            tracing::debug!(
                case_id = event.case_id,
                new_status = ?new_status,
                "no room scenario matched — skipping"
            );
        }
    }
}

/// C2.1 — Jury room provisioning.
///
/// Triggered on CaseStatus::JurySelection transitions.
/// always_pseudonym enforcement: skips if juror_pseudonyms is empty (ADR-012).
async fn provision_jury_room(state: Arc<AppState>, event: CaseTransitionEvent) {
    // always_pseudonym: jury rooms require juror_pseudonyms (ADR-015)
    if event.juror_pseudonyms.is_empty() {
        tracing::warn!(
            case_id = event.case_id,
            "juror_pseudonyms is empty on jury_selection — skipping provision (ADR-012)"
        );
        return;
    }

    // (b) idempotency check
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
    }

    // (c) provision the jury room
    let room_alias = format!("jury-case-{}", event.case_id);
    let room_id = match provision::create_community_room(&state.config, &room_alias).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(err = %e, case_id = event.case_id, "create_community_room failed for jury");
            return;
        }
    };

    // OQ-009: newly provisioned rooms have no messages — reveal state is false at creation.
    // The bridge re-checks on subsequent events via the soft-pause poller threshold.
    let room_has_messages = false;

    // (d) invite each juror puppet with OQ-009 display name
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
        let display_name = if room_has_messages {
            format!("Juror-{suffix}")
        } else {
            "Juror-pending".to_string()
        };
        tracing::info!(mxid = %mxid, display_name = %display_name, "inviting juror to jury room");
        if let Err(e) = invite_to_room(&*state, &room_id, &mxid).await {
            tracing::warn!(err = %e, mxid = %mxid, "invite failed — swallowing (ADR-012)");
        }
    }

    // (e) persist — reveal_state tracks OQ-009 status
    let reveal_state = if room_has_messages { "revealed" } else { "pending" };
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
            Some(reveal_state),
        ) {
            tracing::error!(err = %e, case_id = event.case_id, "bridge_room::upsert failed");
        }
    }
}

/// C2.2 — Community-event room provisioning.
///
/// Triggered by community-scoped transitions (community_id.is_some()) that are
/// not jury, appeal, emergency, or spin-out.
async fn provision_community_event_room(state: Arc<AppState>, event: CaseTransitionEvent) {
    {
        let conn = match bridge_room::open(&state.bridge_db_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::open failed");
                return;
            }
        };
        match bridge_room::lookup(&conn, event.case_id as i64, "community") {
            Ok(Some(_)) => {
                tracing::debug!(case_id = event.case_id, "community room already exists — idempotent skip");
                return;
            }
            Ok(None) => {}
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::lookup failed");
                return;
            }
        }
    }

    let room_alias = format!("community-case-{}", event.case_id);
    let room_id = match provision::create_community_room(&state.config, &room_alias).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(
                err = %e,
                case_id = event.case_id,
                "create_community_room failed for community event"
            );
            return;
        }
    };

    // No pseudonym requirement for community-event rooms
    {
        let conn = match bridge_room::open(&state.bridge_db_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::open for upsert failed");
                return;
            }
        };
        if let Err(e) =
            bridge_room::upsert(&conn, event.case_id as i64, "community", &room_id, "active", None)
        {
            tracing::error!(err = %e, case_id = event.case_id, "bridge_room::upsert failed");
        }
    }
}

/// C2.3 — Spin-out room provisioning.
///
/// Triggered when a community case (community_id.is_some()) escalates to
/// instance-level review (new_status == "in_review").
async fn provision_spinout_room(state: Arc<AppState>, event: CaseTransitionEvent) {
    {
        let conn = match bridge_room::open(&state.bridge_db_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::open failed");
                return;
            }
        };
        match bridge_room::lookup(&conn, event.case_id as i64, "spinout") {
            Ok(Some(_)) => {
                tracing::debug!(case_id = event.case_id, "spinout room already exists — idempotent skip");
                return;
            }
            Ok(None) => {}
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::lookup failed");
                return;
            }
        }
    }

    let room_alias = format!("spinout-case-{}", event.case_id);
    let room_id = match provision::create_community_room(&state.config, &room_alias).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(err = %e, case_id = event.case_id, "create_community_room failed for spinout");
            return;
        }
    };

    {
        let conn = match bridge_room::open(&state.bridge_db_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::open for upsert failed");
                return;
            }
        };
        if let Err(e) =
            bridge_room::upsert(&conn, event.case_id as i64, "spinout", &room_id, "active", None)
        {
            tracing::error!(err = %e, case_id = event.case_id, "bridge_room::upsert failed");
        }
    }
}

/// C2.4 — Appeal room provisioning.
///
/// Triggered on CaseStatus::Appealed transitions.
/// always_pseudonym enforcement: skips if juror_pseudonyms is empty (ADR-012).
async fn provision_appeal_room(state: Arc<AppState>, event: CaseTransitionEvent) {
    // always_pseudonym: appeal rooms require juror_pseudonyms (ADR-015)
    if event.juror_pseudonyms.is_empty() {
        tracing::warn!(
            case_id = event.case_id,
            "juror_pseudonyms is empty on appealed transition — skipping appeal room provision (ADR-012)"
        );
        return;
    }

    {
        let conn = match bridge_room::open(&state.bridge_db_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::open failed");
                return;
            }
        };
        match bridge_room::lookup(&conn, event.case_id as i64, "appeal") {
            Ok(Some(_)) => {
                tracing::debug!(case_id = event.case_id, "appeal room already exists — idempotent skip");
                return;
            }
            Ok(None) => {}
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::lookup failed");
                return;
            }
        }
    }

    let room_alias = format!("appeal-case-{}", event.case_id);
    let room_id = match provision::create_community_room(&state.config, &room_alias).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(err = %e, case_id = event.case_id, "create_community_room failed for appeal");
            return;
        }
    };

    // OQ-009: newly provisioned rooms have no messages — reveal state is false at creation.
    let room_has_messages = false;

    // Invite appeals panel jurors from event.juror_pseudonyms
    for pseudonym in &event.juror_pseudonyms {
        let mxid = match state.puppet_map.ensure_puppet(pseudonym).await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(
                    err = %e,
                    pseudonym = %pseudonym,
                    "ensure_puppet failed — skipping appeal juror"
                );
                continue;
            }
        };
        let suffix = pseudonym.split('/').next_back().unwrap_or(pseudonym.as_str());
        let display_name = if room_has_messages {
            format!("Juror-{suffix}")
        } else {
            "Juror-pending".to_string()
        };
        tracing::info!(mxid = %mxid, display_name = %display_name, "inviting appeal juror");
        if let Err(e) = invite_to_room(&*state, &room_id, &mxid).await {
            tracing::warn!(
                err = %e,
                mxid = %mxid,
                "appeal room invite failed — swallowing (ADR-012)"
            );
        }
    }

    let reveal_state = if room_has_messages { "revealed" } else { "pending" };
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
            "appeal",
            &room_id,
            "active",
            Some(reveal_state),
        ) {
            tracing::error!(err = %e, case_id = event.case_id, "bridge_room::upsert failed");
        }
    }
}

/// C2.5 — Emergency room provisioning (latency-critical, ADR-013 <2s target).
///
/// Provisions FIRST; chain-emission deferred to T5 async callback.
/// Members: legal_contact_mxid only. Reported party ABSENT (ADR-013).
async fn provision_emergency_room(state: Arc<AppState>, event: CaseTransitionEvent) {
    {
        let conn = match bridge_room::open(&state.bridge_db_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::open failed");
                return;
            }
        };
        match bridge_room::lookup(&conn, event.case_id as i64, "emergency") {
            Ok(Some(_)) => {
                tracing::debug!(case_id = event.case_id, "emergency room already exists — idempotent skip");
                return;
            }
            Ok(None) => {}
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::lookup failed");
                return;
            }
        }
    }

    // Provision FIRST for <2s ADR-013 target
    let room_alias = format!("emergency-case-{}", event.case_id);
    let room_id = match provision::create_community_room(&state.config, &room_alias).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(
                err = %e,
                case_id = event.case_id,
                "create_community_room failed for emergency"
            );
            return;
        }
    };

    // Invite legal_contact_mxid; reported party ABSENT per ADR-013
    let legal_mxid = state.config.legal_contact_mxid.clone();
    if let Err(e) = invite_to_room(&*state, &room_id, &legal_mxid).await {
        tracing::warn!(
            err = %e,
            mxid = %legal_mxid,
            "emergency room legal_contact invite failed — swallowing (ADR-012)"
        );
    }

    // Persist immediately; chain-emission deferred to T5
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
            "emergency",
            &room_id,
            "active",
            None,
        ) {
            tracing::error!(err = %e, case_id = event.case_id, "bridge_room::upsert failed");
        }
    }

    tracing::info!(
        case_id = event.case_id,
        room_id = %room_id,
        "emergency room provisioned (chain-emission deferred to T5)"
    );
}

/// C2.6 — Membership-mirror room provisioning.
///
/// Tracks community membership changes on case closure transitions (decided/closed).
async fn provision_membership_mirror(state: Arc<AppState>, event: CaseTransitionEvent) {
    {
        let conn = match bridge_room::open(&state.bridge_db_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::open failed");
                return;
            }
        };
        match bridge_room::lookup(&conn, event.case_id as i64, "membership") {
            Ok(Some(_)) => {
                tracing::debug!(
                    case_id = event.case_id,
                    "membership room already exists — idempotent skip"
                );
                return;
            }
            Ok(None) => {}
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::lookup failed");
                return;
            }
        }
    }

    let room_alias = format!("membership-case-{}", event.case_id);
    let room_id = match provision::create_community_room(&state.config, &room_alias).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(
                err = %e,
                case_id = event.case_id,
                "create_community_room failed for membership mirror"
            );
            return;
        }
    };

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
            "membership",
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
async fn invite_to_room(state: &AppState, room_id: &str, mxid: &str) -> anyhow::Result<()> {
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
