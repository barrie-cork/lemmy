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
}

/// Wire shape sent to the binary's room-event handler. Mirrors room_event_handler.rs:19-23.
#[derive(serde::Serialize)]
// ponytail: scaffold-ahead-of-caller — Task 6 wires the async controller drain that constructs this.
#[allow(dead_code)]
struct RoomEventRequest<'a> {
    entry_kind: &'a str,
    payload: RoomEventPayload,
    actor_pseudonym: Option<String>,
}

/// POST to the binary's /api/v4/governance/room-event with Bearer auth.
/// Propagates transport/status errors via `?`; the CALLER swallows them
/// (fire-and-forget per bridge_notify.rs:62-72). ADR-016: payload is metadata only.
#[allow(dead_code)] // Task 6: the async bridge controller drains Stage::pending_emits and calls this.
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
}
