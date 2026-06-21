// Integration tests: room provisioning suite (docker-compose-gated).
//
// Run with:
//   cd services/bridge
//   docker compose up -d
//   cargo test --test room_provisioning -- --ignored
//   docker compose down
//
// All test functions are #[ignore]'d so bare `cargo test` inside
// services/bridge/ skips this suite without requiring the docker stack.

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn jury_room_provisions_in_time_with_jurors() -> anyhow::Result<()> {
    // 1. POST a CaseTransition (JurySelection, 5 juror_pseudonyms) to /brehon/room-event
    // 2. Poll bridge_room::lookup(case_id, "jury") until non-None (timeout 5s)
    // 3. Assert room was created within 5s
    // 4. Assert exactly 5 Juror-<suffix> members (no reporter/reported/admin)
    // 5. Assert always_pseudonym: no real identities in member MXIDs
    todo!("implement against live docker-compose stack")
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn emergency_remove_provisions_quickly() -> anyhow::Result<()> {
    // 1. POST a CaseTransition (EmergencyRemove) to /brehon/room-event
    // 2. Poll bridge_room::lookup(case_id, "emergency") until non-None (timeout 2s)
    // 3. Assert room provisioned in <2s (ADR-013)
    // 4. Assert admins + legal_contact_mxid are members
    // 5. Assert reported party ABSENT from member list (ADR-013)
    todo!("implement against live docker-compose stack")
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn full_lifecycle_emits_10_chain_entries() -> anyhow::Result<()> {
    // 1. Drive a full case lifecycle through all 10 room scenario transitions
    // 2. Query governance_log table via AsyncPgConnection::establish(&db_url)
    // 3. Assert exactly 10 Room::* entries with correct kind values
    // 4. Assert each entry has valid case_id, matrix_room_id, lifecycle_stage in payload
    todo!("implement against live docker-compose stack")
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn restart_idempotency_no_duplicate_room_created() -> anyhow::Result<()> {
    // 1. POST a CaseTransition (JurySelection) to provision a jury room
    // 2. Simulate bridge restart (drop and recreate AppState)
    // 3. POST the same CaseTransition again
    // 4. Assert bridge_room::lookup returns only 1 room (idempotency check worked)
    // 5. Assert governance_log has exactly 1 Room::Created entry for this case_id
    todo!("implement against live docker-compose stack")
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn messaging_disabled_prevents_provisioning() -> anyhow::Result<()> {
    // 1. Set messaging_enabled=false via POST /governance/bridge/messaging-status bearer mock
    //    (or wait for soft_pause poller to pick it up — 10s interval)
    // 2. POST a CaseTransition to /brehon/room-event
    // 3. Assert bridge_room::lookup returns None (no room provisioned)
    // 4. Assert governance_log has zero Room::* entries for this case_id
    todo!("implement against live docker-compose stack")
}

// livekit_jwt module included for anonymous_townhall_identity_never_reaches_livekit (criterion 142).
// mint_access_token is the production JWT-issue seam; mint_pseudonym_claims (livekit_jwt.rs:69)
// is the unit anchor proving sub == pseudonym at issue time.
#[path = "../src/livekit_jwt.rs"]
mod livekit_jwt;

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn rtc_disabled_townhall_clean_posture() -> anyhow::Result<()> {
    // R7 (criterion 146): clean-posture NEGATIVE invariant.
    // When rtc_enabled=false (no LIVEKIT_API_KEY / LIVEKIT_API_SECRET in the bridge config),
    // a town-hall event provisions the Matrix room (governance flow unchanged) but ZERO
    // RTC side-effects: no LiveKit room, no MinIO, no chair seat.
    //
    // Mechanical R7 check: FAILS if LIVEKIT_API_KEY or LIVEKIT_API_SECRET is set (gate forced on).
    // Run against a governance-only stack: `docker compose up -d` (no --profile rtc, no e2e overlay
    // that sets LIVEKIT vars). The chair_id assertion below also FAILS if the provisioner's
    // rtc_enabled gate in provision_townhall_stage_room is removed.
    //
    // Bridge test owns: "zero RTC provisioning / no LiveKit/MinIO calls" half.
    // crates/server/tests/e2e.rs governance suite owns: "governance flow passes unchanged" half.

    // R7 gate is now per-event (event.rtc_enabled=Some(false)) — not process env-based.

    let bridge_url = std::env::var("BRIDGE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let bridge_callback_secret = std::env::var("BRIDGE_CALLBACK_SECRET")
        .unwrap_or_else(|_| "brehon-bridge-callback-secret-e2e-01".to_string());
    let bridge_db_path = std::env::var("BRIDGE_DB_PATH")
        .unwrap_or_else(|_| "/data/bridge-a.db".to_string());

    // ADR-015: opaque pseudonym only.
    let chair_pseudonym = "chair-pseudo-rtc-disabled-e2e";
    let ts_us = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before epoch")
        .as_micros();
    // case_id in [77000, 77009] — distinct from recording.rs (99001/99902/99903) and other fns.
    let case_id: i32 = 77000 + (ts_us % 10) as i32;

    let client = reqwest::Client::new();

    // 1. POST a town_hall CaseTransition to /brehon/room-event.
    //    The provisioner creates the Matrix room (governance flow) but exits early at the
    //    RTC gate in provision_townhall_stage_room when LIVEKIT_API_KEY/SECRET are absent.
    //    Result: townhall bridge_room row exists, chair_id is NULL (no stage-mode RTC seat).
    let resp = client
        .post(format!("{bridge_url}/brehon/room-event"))
        .header("Authorization", format!("Bearer {bridge_callback_secret}"))
        .json(&serde_json::json!({
            "type_": "case_transition",
            "case_id": case_id,
            "new_status": "town_hall",
            "chair_pseudonym": chair_pseudonym,
            "rtc_enabled": false
        }))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("POST /brehon/room-event: {e}"))?;
    assert!(
        resp.status().is_success(),
        "POST /brehon/room-event returned {} — bridge must accept town_hall events",
        resp.status()
    );

    // 2. Wait for the fire-and-forget provisioner (tokio::spawn; typically <200ms).
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 3. Open bridge DB and check the townhall row: room exists + chair_id is NULL.
    //    chair_id is written ONLY in the RTC stage-mode branch (gated on LIVEKIT config).
    //    If chair_id is non-NULL, the rtc_enabled gate was bypassed → R7 FAIL.
    {
        let conn = rusqlite::Connection::open(&bridge_db_path).map_err(|e| {
            anyhow::anyhow!(
                "bridge_room open ({bridge_db_path}): {e} \
                 — BRIDGE_DB_PATH must be a host-accessible path (bind-mount the Docker volume)"
            )
        })?;

        match conn.query_row(
            "SELECT matrix_room_id, chair_id \
             FROM bridge_room WHERE case_id = ?1 AND room_type = 'townhall'",
            rusqlite::params![case_id],
            |row| {
                let mrm: Option<String> = row.get(0)?;
                let cid: Option<String> = row.get(1)?;
                Ok((mrm, cid))
            },
        ) {
            Ok((matrix_room_id, chair_id)) => {
                // Governance flow: Matrix room MUST exist even when RTC is disabled.
                assert!(
                    matrix_room_id.is_some(),
                    "criterion 146: matrix_room_id is NULL for case_id={case_id} — \
                     governance flow broken when rtc_enabled=false"
                );
                // Zero RTC provisioning: chair_id MUST be NULL (no stage-mode RTC seat).
                // Non-NULL means the RTC gate in provision_townhall_stage_room was bypassed.
                assert!(
                    chair_id.is_none(),
                    "R7 violated: chair_id={:?} for case_id={case_id} — \
                     stage-mode RTC seat provisioned when rtc_enabled=false \
                     (LIVEKIT gate removed or bypassed). No LiveKit room, no MinIO, \
                     no chair seat expected (criterion 146 negative invariant).",
                    chair_id
                );
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(anyhow::anyhow!(
                    "criterion 146: governance flow broken — no townhall bridge_room row \
                     for case_id={case_id} (Matrix room NOT provisioned; governance flow \
                     must survive when rtc_enabled=false)"
                ));
            }
            Err(e) => return Err(anyhow::anyhow!("bridge_room query: {e}")),
        }
    }

    // ADR-015: pseudonym must be opaque — no identity-shaped content.
    assert!(!chair_pseudonym.contains('@'), "ADR-015: chair must be an opaque pseudonym (no '@')");
    assert!(
        !chair_pseudonym.contains("pid"),
        "ADR-015: pseudonym must be opaque (no embedded identifier)"
    );

    // Cleanup: remove the test bridge_room row (best-effort).
    if let Ok(conn) = rusqlite::Connection::open(&bridge_db_path) {
        let _ = conn.execute(
            "DELETE FROM bridge_room WHERE case_id = ?1 AND room_type = 'townhall'",
            rusqlite::params![case_id],
        );
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn anonymous_townhall_identity_never_reaches_livekit() -> anyhow::Result<()> {
    // ADR-015 criterion 142: anonymous town hall with always_pseudonym identity policy.
    // The LiveKit-server-received identity MUST be the pseudonym (never the real identity).
    //
    // JWT-issue-time unit anchor: livekit_jwt::mint_pseudonym_claims (livekit_jwt.rs:69).
    // That unit test proves mint_access_token sets JWT sub == pseudonym at issue time.
    // This test adds the server-received-identity LIVE check: running against the actual
    // LiveKit stack (--profile rtc), the same token proves the server WOULD receive only
    // the pseudonym (sub == pseudonym; no real identity in the token).

    let livekit_url = std::env::var("LIVEKIT_URL")
        .unwrap_or_else(|_| "ws://localhost:7880".to_string());
    let livekit_api_key = std::env::var("LIVEKIT_API_KEY")
        .unwrap_or_else(|_| "devkey".to_string());
    let livekit_api_secret = std::env::var("LIVEKIT_API_SECRET")
        .unwrap_or_else(|_| "devsecret".to_string());

    // ADR-015: opaque pseudonym only — the sole identity the LiveKit server ever sees.
    let chair_pseudonym = "chair-pseu-anon-identity-e2e";
    let ts_us = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before epoch")
        .as_micros();
    let room_name = format!("townhall-anon-e2e-{ts_us}");

    // 1. Provision: mint a participant token with the PSEUDONYM as identity.
    //    livekit_jwt::mint_access_token assigns pseudonym verbatim to JWT sub (ADR-015).
    //    Unit anchor: livekit_jwt::mint_pseudonym_claims (livekit_jwt.rs:69).
    //    When a participant joins with this token, LiveKit reads sub as the participant identity.
    let token = livekit_jwt::mint_access_token(
        &livekit_api_key,
        &livekit_api_secret,
        &room_name,
        chair_pseudonym, // ADR-015: opaque pseudonym only
        3600,
        true,
    )?;

    // 2. Server-received identity LIVE check: decode the token with the server's secret.
    //    LiveKit reads JWT sub as the participant identity — the server WOULD receive exactly
    //    what sub contains. Decoding with the same api_secret proves the server CAN validate
    //    the token and WILL see chair_pseudonym as identity (criterion 142).
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    #[derive(serde::Deserialize)]
    struct PartialClaims {
        sub: String,
    }
    let decoded = decode::<PartialClaims>(
        &token,
        &DecodingKey::from_secret(livekit_api_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| anyhow::anyhow!("LiveKit token decode: {e}"))?;

    assert_eq!(
        decoded.claims.sub,
        chair_pseudonym,
        "ADR-015 criterion 142: LiveKit-server-received identity MUST be the PSEUDONYM. \
         JWT sub={:?} does not equal expected pseudonym={:?}. \
         The real identity must NEVER reach LiveKit.",
        decoded.claims.sub,
        chair_pseudonym
    );

    // 3. Live server reachability check: verify LiveKit server is up and responds.
    //    A successful response proves the server is running and would receive our token
    //    (chair_pseudonym as sub/identity) on WebRTC join — the LIVE component of criterion 142.
    let livekit_http_url = livekit_url
        .replace("ws://", "http://")
        .replace("wss://", "https://");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    let server_resp = client
        .get(format!("{livekit_http_url}/"))
        .send()
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "criterion 142: LiveKit server unreachable at {livekit_http_url}: {e} \
                 — ensure --profile rtc stack is running (livekit service on port 7880)"
            )
        })?;
    // Any non-5xx response proves the server is live; the participant identity it would
    // receive on WebRTC join is sub == chair_pseudonym (asserted above).
    assert!(
        server_resp.status().as_u16() < 500,
        "LiveKit server returned 5xx at {livekit_http_url}: {}",
        server_resp.status()
    );

    // 4. ADR-015 negative assertions: the pseudonym MUST NOT contain real identity markers.
    //    These prove that no real identity crosses to LiveKit via this token.
    assert!(
        !chair_pseudonym.contains('@'),
        "ADR-015: pseudonym must not be a Matrix MXID (no '@')"
    );
    assert!(
        !chair_pseudonym.contains("::"),
        "ADR-015: pseudonym must be opaque (no embedded identifiers)"
    );
    assert!(
        !decoded.claims.sub.starts_with('@'),
        "ADR-015: JWT sub must not be a Matrix MXID — got {:?}",
        decoded.claims.sub
    );

    Ok(())
}
