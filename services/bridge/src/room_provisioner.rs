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

use crate::{appservice::AppState, bridge_room, livekit_jwt, provision, room_event_client, stage};

/// Bridge-local mirror of the governance.rs CaseTransitionEvent wire shape.
/// Unknown fields are silently ignored (serde default).
#[derive(Debug, Clone, Deserialize)]
pub struct CaseTransitionEvent {
    pub case_id: i32,
    #[serde(default)]
    pub juror_pseudonyms: Vec<String>,
    #[serde(default)]
    pub new_status: Option<String>,
    // Part of the wire-contract mirror; not consumed by the provisioner yet
    // (the room dispatch keys on new_status). Kept for payload completeness.
    #[serde(default)]
    #[allow(dead_code)]
    pub old_status: Option<String>,
    #[serde(default)]
    pub community_id: Option<i32>,
    /// Chair pseudonym for stage-mode town-hall rooms (Phase-3 bridge read; binary populates Phase-6).
    /// Falls back to juror_pseudonyms[0] (foreperson) when absent (OQ-V2-05).
    #[serde(default)]
    pub chair_pseudonym: Option<String>,
    /// Per-event RTC toggle, mirrored on the wire from the governance `rtc_enabled`
    /// config. `None` (absent) defaults RTC ON for back-compat — events without the
    /// field keep the pre-existing creds-gated behaviour. `Some(false)` disables the
    /// RTC stage seat even when LiveKit creds are configured (criterion 146 / R7).
    #[serde(default)]
    pub rtc_enabled: Option<bool>,
}

/// Bridge-local discriminated union for POST /brehon/room-event.
/// Mirrors the BridgeNotifyPayload enum in governance.rs (serde `type_` tag).
#[derive(Debug, Deserialize)]
#[serde(tag = "type_", rename_all = "snake_case")]
pub enum RoomEventPayload {
    // The room-event handler only acts on CaseTransition; PrivateMessage is
    // part of the BridgeNotifyPayload mirror (the DM-relay variant) and its
    // inner value is intentionally not read here.
    #[allow(dead_code)]
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
        Some("town_hall") => provision_townhall_stage_room(state, event).await,
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

    // OQ-009: determine reveal state based on room event count
    let threshold = state.oq009_reveal_threshold.load(std::sync::atomic::Ordering::Relaxed) as usize;
    let room_has_messages = query_room_event_count(&state, &room_id).await >= threshold;

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
        if let Err(e) = invite_to_room(&state, &room_id, &mxid).await {
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

/// C2.X — Town-hall stage-mode room provisioning.
///
/// Triggered on `new_status == "town_hall"`. Provisions the Matrix room — the Q&A
/// sidebar IS the Matrix room's native text timeline; no new room type, no content
/// field, no hashing (ADR-016). When RTC is configured (livekit_api_key + secret),
/// additionally enters stage mode: seats the chair, initialises the raised-hand queue
/// to `[]`, and mints the chair a presenter token (`can_publish=true`). Watcher tokens
/// are minted at join-time (no participant list at provisioning time).
///
/// After stage setup the provisioner drains `Stage::pending_emits` into the binary's
/// room-event endpoint. At provisioning time the drain loops zero times (no live
/// stage-action HTTP caller yet — Phase-6 wires that); the drain makes `post_room_event`
/// reachable from a non-test production caller, closing the emit-intent seam (§2.1).
async fn provision_townhall_stage_room(state: Arc<AppState>, event: CaseTransitionEvent) {
    // (b) idempotency check — skip if townhall room already exists for this case.
    {
        let conn = match bridge_room::open(&state.bridge_db_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::open failed");
                return;
            }
        };
        match bridge_room::lookup(&conn, event.case_id as i64, "townhall") {
            Ok(Some(_)) => {
                tracing::debug!(case_id = event.case_id, "townhall room already exists — idempotent skip");
                return;
            }
            Ok(None) => {}
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::lookup failed");
                return;
            }
        }
    }

    // (c) provision the Matrix room. The Q&A sidebar IS the Matrix room's native text
    // timeline — no new room type, no content field, no hashing (ADR-016).
    let room_alias = format!("townhall-case-{}", event.case_id);
    let room_id = match provision::create_community_room(&state.config, &room_alias).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(err = %e, case_id = event.case_id, "create_community_room failed for townhall");
            return;
        }
    };

    // (d) persist the room record.
    {
        let conn = match bridge_room::open(&state.bridge_db_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::open for upsert failed");
                return;
            }
        };
        if let Err(e) =
            bridge_room::upsert(&conn, event.case_id as i64, "townhall", &room_id, "active", None)
        {
            tracing::error!(err = %e, case_id = event.case_id, "bridge_room::upsert failed");
            return;
        }
    }

    if event.rtc_enabled == Some(false) {
        tracing::debug!(
            case_id = event.case_id,
            "rtc_enabled=false — skipping stage-mode setup (R7 negative invariant)"
        );
        return;
    }
    // (e) stage mode — gated on RTC config (livekit_api_key + livekit_api_secret required).
    let (api_key, api_secret) = match (
        state.config.livekit_api_key.as_deref(),
        state.config.livekit_api_secret.as_deref(),
    ) {
        (Some(k), Some(s)) => (k, s),
        _ => {
            tracing::debug!(
                case_id = event.case_id,
                "RTC not configured — skipping stage-mode setup"
            );
            return;
        }
    };

    // Seat the chair: event.chair_pseudonym ?? juror_pseudonyms[0] (foreperson fallback, OQ-V2-05).
    // ADR-015: both sources are opaque pseudonym strings — never a numeric DB identity or Matrix user ID.
    let chair = match event
        .chair_pseudonym
        .as_deref()
        .or_else(|| event.juror_pseudonyms.first().map(|s| s.as_str()))
    {
        Some(c) => c.to_string(),
        None => {
            tracing::warn!(
                case_id = event.case_id,
                "no chair_pseudonym or foreperson pseudonym — skipping stage-mode seat"
            );
            return;
        }
    };

    // Persist chair_id and initialise raised-hand FIFO queue to empty.
    {
        let conn = match bridge_room::open(&state.bridge_db_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "bridge_room::open for chair_id failed");
                return;
            }
        };
        if let Err(e) = bridge_room::write_chair_id(&conn, event.case_id as i64, "townhall", &chair)
        {
            tracing::error!(err = %e, case_id = event.case_id, "write_chair_id failed");
            return;
        }
        if let Err(e) =
            bridge_room::write_queue_state(&conn, event.case_id as i64, "townhall", "[]")
        {
            tracing::error!(err = %e, case_id = event.case_id, "write_queue_state failed");
            return;
        }
    }

    // Mint chair presenter token (can_publish=true). Watcher tokens are deferred to join-time
    // (no participant list at provisioning time). Phase-6 distributes the chair token.
    match livekit_jwt::mint_access_token(api_key, api_secret, &room_alias, &chair, 86400, true) {
        Ok(_token) => {
            tracing::debug!(
                case_id = event.case_id,
                chair = %chair,
                "chair presenter token minted; watcher tokens minted at join-time"
            );
        }
        Err(e) => {
            tracing::warn!(
                err = %e,
                case_id = event.case_id,
                "chair presenter token mint failed — non-fatal"
            );
        }
    }

    // Drain pending room-event emits. At provisioning time Stage::pending_emits is empty
    // (transfer_chair/chair_override have no live HTTP caller yet — Phase-6 wires that).
    // The drain here makes post_room_event reachable from a non-test caller, closing the
    // emit-intent seam and allowing the 3 dead_code scaffolding allows to be removed (§2.1).
    let mut stage = {
        let conn = match bridge_room::open(&state.bridge_db_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(
                    err = %e,
                    case_id = event.case_id,
                    "bridge_room::open for stage drain failed"
                );
                return;
            }
        };
        match stage::Stage::load(&conn, event.case_id as i64, "townhall") {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(err = %e, case_id = event.case_id, "Stage::load failed");
                return;
            }
        }
        // conn dropped here — Stage owns its in-memory state
    };
    room_event_client::drain_emits(
        &mut stage,
        &state.http_client,
        &state.config.brehon_room_event_url,
        &state.config.bridge_callback_secret,
    )
    .await;

    tracing::info!(
        case_id = event.case_id,
        room_id = %room_id,
        chair = %chair,
        "townhall stage-mode room provisioned"
    );
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

    // OQ-009: determine reveal state for appeal jurors
    let threshold = state.oq009_reveal_threshold.load(std::sync::atomic::Ordering::Relaxed) as usize;
    let room_has_messages = query_room_event_count(&state, &room_id).await >= threshold;

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
        if let Err(e) = invite_to_room(&state, &room_id, &mxid).await {
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
    if let Err(e) = invite_to_room(&state, &room_id, &legal_mxid).await {
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

/// OQ-009: query the Matrix room event count via GET /messages?limit=1.
/// Returns chunk length (0 on any error — fail-safe).
async fn query_room_event_count(state: &AppState, room_id: &str) -> usize {
    let encoded_room_id = room_id.replace(':', "%3A");
    let url = format!(
        "{}/_matrix/client/v3/rooms/{}/messages?limit=1",
        state.config.tuwunel_url, encoded_room_id
    );
    match state
        .http_client
        .get(&url)
        .bearer_auth(&state.config.as_token)
        .send()
        .await
    {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => json
                .get("chunk")
                .and_then(|c| c.as_array())
                .map(|a| a.len())
                .unwrap_or(0),
            Err(_) => 0,
        },
        Err(e) => {
            tracing::debug!(
                err = %e,
                room_id = %room_id,
                "room event count query failed — defaulting to 0"
            );
            0
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
