/// Bridge-side Serialize mirror of the binary's room-event payload.
/// Serializes to the exact JSON shape /api/v4/governance/room-event expects.
/// ADR-015: all *_pseudonym fields are opaque pseudonym strings, never real identities.
/// ADR-016: payload carries METADATA only — never Q&A text, speech, or video content.
#[derive(serde::Serialize)]
pub struct RoomEventPayload {
    pub case_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matrix_room_id: Option<String>,
    pub lifecycle_stage: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_pseudonym: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_pseudonym: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_pseudonym: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub federated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speakers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attendance_count: Option<i32>,
}

/// Wire shape sent to the binary's room-event handler. Mirrors room_event_handler.rs:19-23.
#[derive(serde::Serialize)]
struct RoomEventRequest<'a> {
    entry_kind: &'a str,
    payload: RoomEventPayload,
    actor_pseudonym: Option<String>,
}

/// POST to the binary's /api/v4/governance/room-event with Bearer auth.
/// Propagates transport/status errors via `?`; the CALLER swallows them
/// (fire-and-forget per bridge_notify.rs:62-72). ADR-016: payload is metadata only.
pub async fn post_room_event(
    client: &reqwest::Client,
    url: &str,
    secret: &str,
    entry_kind: &str,
    payload: RoomEventPayload,
    actor_pseudonym: Option<String>,
) -> anyhow::Result<()> {
    let req = RoomEventRequest { entry_kind, payload, actor_pseudonym };
    client
        .post(url)
        .header("Authorization", format!("Bearer {secret}"))
        .json(&req)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

/// Drain all pending room-event POST intents from `stage.pending_emits`.
///
/// Called by the async provisioning controller after any stage-mutating operation.
/// Fire-and-forget: transport errors are warned and swallowed per bridge_notify.rs:62-72.
/// At provisioning time `pending_emits` is empty (no live stage-action HTTP endpoint yet —
/// Phase-6 wires that); the drain loops zero times but makes `post_room_event` reachable
/// from a non-test production caller, removing the dead_code scaffolding (§2.1).
pub async fn drain_emits(
    stage: &mut crate::stage::Stage,
    client: &reqwest::Client,
    url: &str,
    secret: &str,
) {
    for intent in stage.pending_emits.drain(..) {
        if let Err(e) =
            post_room_event(client, url, secret, intent.entry_kind, intent.payload, intent.actor_pseudonym)
                .await
        {
            tracing::warn!("room-event POST failed (non-fatal — best-effort chain entry): {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    /// room_chair_override JSON shape: entry_kind, action, target_pseudonym present;
    /// transfer-only fields (from_pseudonym, to_pseudonym) ABSENT via skip_serializing_if.
    #[test]
    fn override_request_json_shape() -> Result<()> {
        let req = RoomEventRequest {
            entry_kind: "room_chair_override",
            payload: RoomEventPayload {
                case_id: 1,
                matrix_room_id: None,
                lifecycle_stage: "governance".to_string(),
                member_count: None,
                action: Some("force_demote".to_string()),
                target_pseudonym: Some("pseudonym-abc".to_string()),
                from_pseudonym: None,
                to_pseudonym: None,
                at: None,
                federated: None,
                media_url: None,
                content_sha256: None,
                duration_s: None,
                speakers: None,
                attendance_count: None,
            },
            actor_pseudonym: Some("chair-pseudonym".to_string()),
        };
        let v = serde_json::to_value(&req)?;
        assert_eq!(v["entry_kind"], "room_chair_override", "entry_kind must be room_chair_override");
        assert_eq!(v["payload"]["action"], "force_demote", "action must be present");
        assert_eq!(v["payload"]["target_pseudonym"], "pseudonym-abc", "target_pseudonym must be present");
        // skip_serializing_if = "Option::is_none" must omit absent-None fields entirely.
        assert!(
            v["payload"].get("from_pseudonym").is_none(),
            "from_pseudonym must be absent (skip_serializing_if) in override request"
        );
        assert!(
            v["payload"].get("to_pseudonym").is_none(),
            "to_pseudonym must be absent (skip_serializing_if) in override request"
        );
        Ok(())
    }

    /// room_chair_transferred JSON shape: from_pseudonym, to_pseudonym, at all present.
    #[test]
    fn transferred_request_json_shape() -> Result<()> {
        let req = RoomEventRequest {
            entry_kind: "room_chair_transferred",
            payload: RoomEventPayload {
                case_id: 2,
                matrix_room_id: None,
                lifecycle_stage: "governance".to_string(),
                member_count: None,
                action: None,
                target_pseudonym: None,
                from_pseudonym: Some("pseudonym-from".to_string()),
                to_pseudonym: Some("pseudonym-to".to_string()),
                at: Some("2026-01-01T00:00:00Z".to_string()),
                federated: None,
                media_url: None,
                content_sha256: None,
                duration_s: None,
                speakers: None,
                attendance_count: None,
            },
            actor_pseudonym: Some("pseudonym-from".to_string()),
        };
        let v = serde_json::to_value(&req)?;
        assert_eq!(v["entry_kind"], "room_chair_transferred", "entry_kind must be room_chair_transferred");
        assert_eq!(v["payload"]["from_pseudonym"], "pseudonym-from", "from_pseudonym must be present");
        assert_eq!(v["payload"]["to_pseudonym"], "pseudonym-to", "to_pseudonym must be present");
        assert_eq!(v["payload"]["at"], "2026-01-01T00:00:00Z", "at must be present");
        Ok(())
    }

    /// room_mute_all JSON shape: entry_kind and payload.federated present;
    /// chair-action fields (action, from_pseudonym, to_pseudonym) ABSENT via skip_serializing_if.
    #[test]
    fn mute_all_request_json_shape() -> Result<()> {
        let req = RoomEventRequest {
            entry_kind: "room_mute_all",
            payload: RoomEventPayload {
                case_id: 3,
                matrix_room_id: None,
                lifecycle_stage: "governance".to_string(),
                member_count: None,
                action: None,
                target_pseudonym: None,
                from_pseudonym: None,
                to_pseudonym: None,
                at: None,
                federated: Some(true),
                media_url: None,
                content_sha256: None,
                duration_s: None,
                speakers: None,
                attendance_count: None,
            },
            actor_pseudonym: Some("chair-pseudonym".to_string()),
        };
        let v = serde_json::to_value(&req)?;
        assert_eq!(v["entry_kind"], "room_mute_all", "entry_kind must be room_mute_all");
        assert_eq!(v["payload"]["federated"], true, "payload.federated must be true");
        assert_eq!(v["actor_pseudonym"], "chair-pseudonym", "actor_pseudonym must be the chair pseudonym");
        // skip_serializing_if = "Option::is_none" must omit absent-None fields entirely.
        assert!(
            v["payload"].get("action").is_none(),
            "action must be absent (skip_serializing_if) in mute_all request"
        );
        assert!(
            v["payload"].get("from_pseudonym").is_none(),
            "from_pseudonym must be absent (skip_serializing_if) in mute_all request"
        );
        assert!(
            v["payload"].get("to_pseudonym").is_none(),
            "to_pseudonym must be absent (skip_serializing_if) in mute_all request"
        );
        Ok(())
    }

    /// room_recording_uploaded JSON shape: entry_kind, all 5 recording fields, actor_pseudonym present;
    /// chair-action fields (action, from_pseudonym, to_pseudonym, federated) ABSENT via skip_serializing_if.
    /// ADR-015: actor_pseudonym and speakers are pseudonyms (opaque strings, not real identities).
    #[test]
    fn room_recording_uploaded_request_json_shape() -> Result<()> {
        let req = RoomEventRequest {
            entry_kind: "room_recording_uploaded",
            payload: RoomEventPayload {
                case_id: 4,
                matrix_room_id: None,
                lifecycle_stage: "governance".to_string(),
                member_count: None,
                action: None,
                target_pseudonym: None,
                from_pseudonym: None,
                to_pseudonym: None,
                at: None,
                federated: None,
                media_url: Some("https://s3.example.com/room-1.mp4".to_string()),
                content_sha256: Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_string()),
                duration_s: Some(600),
                speakers: Some(vec!["speaker-pseudonym-a".to_string(), "speaker-pseudonym-b".to_string()]),
                attendance_count: Some(42),
            },
            actor_pseudonym: Some("chair-pseudonym".to_string()),
        };
        let v = serde_json::to_value(&req)?;
        assert_eq!(v["entry_kind"], "room_recording_uploaded", "entry_kind must be room_recording_uploaded");
        assert_eq!(v["payload"]["media_url"], "https://s3.example.com/room-1.mp4", "media_url must be present");
        assert_eq!(
            v["payload"]["content_sha256"],
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            "content_sha256 must be present"
        );
        assert_eq!(v["payload"]["duration_s"], 600, "duration_s must be present");
        assert_eq!(
            v["payload"]["speakers"],
            serde_json::json!(["speaker-pseudonym-a", "speaker-pseudonym-b"]),
            "speakers must be present and contain pseudonyms (ADR-015)"
        );
        assert_eq!(v["payload"]["attendance_count"], 42, "attendance_count must be present");
        assert_eq!(v["actor_pseudonym"], "chair-pseudonym", "actor_pseudonym must be the chair pseudonym (ADR-015)");
        // skip_serializing_if = "Option::is_none" must omit chair-action and federated fields.
        assert!(
            v["payload"].get("action").is_none(),
            "action must be absent (skip_serializing_if) in recording_uploaded request"
        );
        assert!(
            v["payload"].get("from_pseudonym").is_none(),
            "from_pseudonym must be absent (skip_serializing_if) in recording_uploaded request"
        );
        assert!(
            v["payload"].get("to_pseudonym").is_none(),
            "to_pseudonym must be absent (skip_serializing_if) in recording_uploaded request"
        );
        assert!(
            v["payload"].get("federated").is_none(),
            "federated must be absent (skip_serializing_if) in recording_uploaded request"
        );
        Ok(())
    }
}
