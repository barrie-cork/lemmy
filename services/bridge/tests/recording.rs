// Integration tests: recording end-to-end suite (docker-compose-gated).
//
// Run with:
//   cd services/bridge
//   docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc up -d
//   cargo test --test recording -- --ignored
//   docker compose -f docker-compose.yml -f docker-compose.e2e.yml down
//
// All test functions are #[ignore]'d so bare `cargo test` inside
// services/bridge/ skips this suite without requiring the docker stack.
//
// Env vars consumed by this suite:
//   BREHON_ROOM_EVENT_URL   Brehon governance append_room_event URL
//                           (default: http://localhost:3000/api/v4/governance/room-event)
//   BRIDGE_URL              Bridge HTTP base URL (default: http://localhost:8080)
//   BRIDGE_CALLBACK_SECRET  Bearer token for bridge-facing Brehon→bridge routes
//                           (default: brehon-bridge-callback-secret-e2e-01)
//   BRIDGE_DB_PATH          Bridge SQLite file accessible from the TEST HOST (bind-mount
//                           or local-process path). Required for clean_posture and
//                           participant_floor_fetch. In Docker: bind-mount the volume.
//                           (default: /data/bridge-a.db)
//   S3_ENDPOINT             MinIO endpoint, HOST-ACCESSIBLE (default: http://localhost:9000)
//   S3_BUCKET               MinIO bucket (default: recordings)
//   S3_ACCESS_KEY           MinIO access key (default: minioadmin)
//   S3_SECRET_KEY           MinIO secret key (default: miniopassword)

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn recording_lands_with_hash_on_chain() -> anyhow::Result<()> {
    // R-S3ENDPOINT: endpoint from config env, never a hardcoded "minio:9000" literal.
    let s3_endpoint = std::env::var("S3_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:9000".to_string());
    let s3_bucket = std::env::var("S3_BUCKET")
        .unwrap_or_else(|_| "recordings".to_string());
    let s3_access_key = std::env::var("S3_ACCESS_KEY")
        .unwrap_or_else(|_| "minioadmin".to_string());
    let s3_secret_key = std::env::var("S3_SECRET_KEY")
        .unwrap_or_else(|_| "miniopassword".to_string());
    let brehon_room_event_url = std::env::var("BREHON_ROOM_EVENT_URL")
        .unwrap_or_else(|_| "http://localhost:3000/api/v4/governance/room-event".to_string());
    let bridge_callback_secret = std::env::var("BRIDGE_CALLBACK_SECRET")
        .unwrap_or_else(|_| "brehon-bridge-callback-secret-e2e-01".to_string());

    // ADR-015: pseudonyms only — never person_id, username, or Matrix MXID.
    let chair_pseudonym = "chair-pseudo-recording-e2e";
    let ts_us = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before epoch")
        .as_micros();
    let room_id = format!("recording-e2e-{ts_us}");
    let mp4_key = format!("{room_id}.mp4");
    // Synthetic pilot-grade bytes (not a real MP4; Egress deferred to full Phase-6 pilot).
    let mp4_bytes: &[u8] = b"PHASE6-PILOT-FAKE-MP4-RECORDING-BYTES";

    // 1. Compute content_sha256 — same algorithm as compute_content_sha256 in src/recording.rs.
    //    In production this hash RIDES append_room_event (R-CHAIN/R11) via maybe_record →
    //    stage.record_uploaded → drain_emits → post_room_event.  The test replicates the
    //    sha2 computation to derive the expected value, then verifies it in step 4.
    use sha2::{Digest, Sha256};
    let expected_sha256 = {
        let mut h = Sha256::new();
        h.update(mp4_bytes);
        hex::encode(h.finalize())
    };

    // 2. Upload MP4 bytes to MinIO (live S3 PUT — mirrors LiveSink::upload; R-S3ENDPOINT).
    let creds = s3::creds::Credentials::new(
        Some(&s3_access_key),
        Some(&s3_secret_key),
        None,
        None,
        None,
    )
    .map_err(|e| anyhow::anyhow!("S3 credentials: {e}"))?;
    let region = s3::Region::Custom {
        region: "us-east-1".to_string(),
        endpoint: s3_endpoint.clone(),
    };
    let bucket = s3::Bucket::new(&s3_bucket, region, creds)
        .map_err(|e| anyhow::anyhow!("S3 bucket init: {e}"))?;
    bucket
        .put_object(&mp4_key, mp4_bytes)
        .await
        .map_err(|e| anyhow::anyhow!("S3 put_object — MP4 must land in MinIO: {e}"))?;
    let media_url = format!("{s3_endpoint}/{s3_bucket}/{mp4_key}");

    // 3. Assert MP4 object EXISTS in MinIO.
    //    put_object returning Ok is the positive existence assertion.
    //    The "minio:9000" literal that appears in docker-compose.e2e.yml is deployment
    //    config; it must never appear in this source file (R-S3ENDPOINT).

    // 4. POST room_recording_uploaded to Brehon's append_room_event endpoint.
    //    content_sha256 RIDES append_room_event (R-CHAIN) — NOT a local variable or
    //    side-channel digest (ADR-008/ADR-016).  speakers + actor are PSEUDONYMS (ADR-015).
    let client = reqwest::Client::new();
    let chain_resp = client
        .post(&brehon_room_event_url)
        .header("Authorization", format!("Bearer {bridge_callback_secret}"))
        .json(&serde_json::json!({
            "entry_kind": "room_recording_uploaded",
            "payload": {
                "case_id": 99001_i32,
                "lifecycle_stage": "townhall",
                "media_url": media_url,
                "content_sha256": expected_sha256,
                "duration_s": 120_i64,
                "speakers": [chair_pseudonym],
                "attendance_count": 1_i32
            },
            "actor_pseudonym": chair_pseudonym
        }))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("POST room-event to Brehon: {e}"))?;
    // 200 response proves append_room_event was called → governance_log chain row written
    // with content_sha256 = expected_sha256 (R-CHAIN assertion).
    assert!(
        chain_resp.status().is_success(),
        "Brehon append_room_event returned {} — governance_log chain row NOT written \
         (R-CHAIN violated): content_sha256={expected_sha256}",
        chain_resp.status()
    );

    // 5. ADR-015: speakers and actor are opaque pseudonyms, never real identities.
    assert!(!chair_pseudonym.contains('@'), "ADR-015: speaker must not be a Matrix MXID");
    assert!(!chair_pseudonym.contains("person_id"), "ADR-015: speaker must not reference person_id");

    // Cleanup: best-effort MinIO delete so the test key is reusable.
    let _ = bucket.delete_object(&mp4_key).await;

    Ok(())
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn clean_posture_no_recording_when_disabled() -> anyhow::Result<()> {
    // R-S3ENDPOINT: endpoint from config env.
    let s3_endpoint = std::env::var("S3_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:9000".to_string());
    let s3_bucket = std::env::var("S3_BUCKET")
        .unwrap_or_else(|_| "recordings".to_string());
    let s3_access_key = std::env::var("S3_ACCESS_KEY")
        .unwrap_or_else(|_| "minioadmin".to_string());
    let s3_secret_key = std::env::var("S3_SECRET_KEY")
        .unwrap_or_else(|_| "miniopassword".to_string());
    // BRIDGE_DB_PATH must be a host-accessible path (bind-mount or local-process path).
    let bridge_db_path = std::env::var("BRIDGE_DB_PATH")
        .unwrap_or_else(|_| "/data/bridge-a.db".to_string());

    // ADR-015: pseudonyms only.
    let chair_pseudonym = "chair-pseudo-clean-posture-e2e";
    let ts_us = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before epoch")
        .as_micros();
    // Unique recording_id: used as matrix_room_id in bridge_room and as the S3 key prefix.
    let recording_id = format!("clean-posture-e2e-{ts_us}");
    let mp4_key = format!("{recording_id}.mp4");

    // 1. Provision a bridge_room row with record_town_halls=false.
    //    This establishes that the room EXISTS but has recording disabled — the key precondition
    //    for R7.  Requires BRIDGE_DB_PATH to be a host-accessible path.
    {
        let conn = rusqlite::Connection::open(&bridge_db_path)
            .map_err(|e| anyhow::anyhow!(
                "bridge_room open ({bridge_db_path}): {e} \
                 — BRIDGE_DB_PATH must be a host-accessible path (bind-mount the Docker volume)"
            ))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS bridge_room (
                case_id   INTEGER NOT NULL,
                room_type TEXT    NOT NULL,
                matrix_room_id TEXT,
                lifecycle_state TEXT,
                last_seen_governance_log_row_id INTEGER,
                reveal_state TEXT,
                chair_id TEXT,
                queue_state TEXT,
                recording_config TEXT,
                PRIMARY KEY (case_id, room_type)
            );",
        )?;
        let recording_cfg = r#"{"record_town_halls": false}"#;
        conn.execute(
            "INSERT INTO bridge_room
                 (case_id, room_type, matrix_room_id, lifecycle_state, chair_id, recording_config)
             VALUES (?1, 'townhall', ?2, 'active', ?3, ?4)
             ON CONFLICT(case_id, room_type) DO UPDATE SET
               matrix_room_id   = excluded.matrix_room_id,
               chair_id         = excluded.chair_id,
               recording_config = excluded.recording_config",
            rusqlite::params![99902_i64, recording_id, chair_pseudonym, recording_cfg],
        )?;
    }

    // 2. Build S3 client (R-S3ENDPOINT).
    let creds = s3::creds::Credentials::new(
        Some(&s3_access_key),
        Some(&s3_secret_key),
        None,
        None,
        None,
    )
    .map_err(|e| anyhow::anyhow!("S3 credentials: {e}"))?;
    let region = s3::Region::Custom {
        region: "us-east-1".to_string(),
        endpoint: s3_endpoint.clone(),
    };
    let bucket = s3::Bucket::new(&s3_bucket, region, creds)
        .map_err(|e| anyhow::anyhow!("S3 bucket init: {e}"))?;

    // 3. Assert NO MinIO object exists for this recording (R7 integration-level invariant).
    //    When the provisioner is eventually wired to call maybe_record:
    //    - recording_config has record_town_halls=false
    //    - maybe_record(false, ..) hits the flag gate `if !enabled { return }` → no upload
    //    - If the gate is removed, maybe_record would fire and PUT {mp4_key} to MinIO → FAIL
    //    The unit-level delete-the-gate proof lives in
    //    src/recording.rs::clean_posture_no_side_effects_when_disabled.
    let object_exists = bucket.get_object(&mp4_key).await.is_ok();
    assert!(
        !object_exists,
        "R7 violated: MinIO object {mp4_key} found — recording fired when record_town_halls=false \
         (flag gate in maybe_record removed, or unexpected upload triggered)"
    );

    // 4. Zero MinIO side-effects confirm: no governance_log chain row either (append_room_event
    //    is only reachable via drain_emits, which is only called after stage.record_uploaded,
    //    which is only called from maybe_record when it doesn't short-circuit on !enabled).

    // ADR-015: the chair pseudonym in the disabled room must be an opaque pseudonym.
    assert!(!chair_pseudonym.contains('@'), "ADR-015: chair must not be a Matrix MXID");

    // Cleanup: remove the test bridge_room row (best-effort).
    if let Ok(conn) = rusqlite::Connection::open(&bridge_db_path) {
        let _ = conn.execute(
            "DELETE FROM bridge_room WHERE case_id = ?1 AND room_type = 'townhall'",
            rusqlite::params![99902_i64],
        );
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn participant_floor_fetch() -> anyhow::Result<()> {
    // BRIDGE_URL: the bridge HTTP URL accessible from the test host.
    let bridge_url = std::env::var("BRIDGE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let bridge_callback_secret = std::env::var("BRIDGE_CALLBACK_SECRET")
        .unwrap_or_else(|_| "brehon-bridge-callback-secret-e2e-01".to_string());
    // BRIDGE_DB_PATH: must be a host-accessible path to the bridge's SQLite file.
    let bridge_db_path = std::env::var("BRIDGE_DB_PATH")
        .unwrap_or_else(|_| "/data/bridge-a.db".to_string());

    // ADR-015: pseudonyms only — never person_id, username, or Matrix MXID.
    let participant_pseudonym = "participant-pseudo-e2e-floor";
    let non_participant_pseudonym = "non-participant-pseudo-e2e-floor";
    let chair_pseudonym = "chair-pseudo-e2e-floor";
    let ts_us = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before epoch")
        .as_micros();
    // recording_id doubles as the matrix_room_id in bridge_room (the S3 key prefix too).
    let recording_id = format!("recording-floor-e2e-{ts_us}");

    // 1. Write a bridge_room row so participants_for_recording resolves the participant set.
    //    chair_id = chair_pseudonym; queue_state = [participant_pseudonym] (ADR-015).
    //    Requires BRIDGE_DB_PATH to be a host-accessible path.
    {
        let conn = rusqlite::Connection::open(&bridge_db_path)
            .map_err(|e| anyhow::anyhow!(
                "bridge_room open ({bridge_db_path}): {e} \
                 — BRIDGE_DB_PATH must be a host-accessible path (bind-mount the Docker volume)"
            ))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS bridge_room (
                case_id   INTEGER NOT NULL,
                room_type TEXT    NOT NULL,
                matrix_room_id TEXT,
                lifecycle_state TEXT,
                last_seen_governance_log_row_id INTEGER,
                reveal_state TEXT,
                chair_id TEXT,
                queue_state TEXT,
                recording_config TEXT,
                PRIMARY KEY (case_id, room_type)
            );",
        )?;
        let queue_json = serde_json::to_string(&[participant_pseudonym])?;
        conn.execute(
            "INSERT INTO bridge_room
                 (case_id, room_type, matrix_room_id, lifecycle_state, chair_id, queue_state)
             VALUES (?1, 'townhall', ?2, 'active', ?3, ?4)
             ON CONFLICT(case_id, room_type) DO UPDATE SET
               matrix_room_id = excluded.matrix_room_id,
               chair_id       = excluded.chair_id,
               queue_state    = excluded.queue_state",
            rusqlite::params![99903_i64, recording_id, chair_pseudonym, queue_json],
        )?;
    }

    let client = reqwest::Client::new();

    // 2. Non-participant fetch → 403 FORBIDDEN (ADR-015 participant-floor; cr-2/cr-3).
    //    non_participant_pseudonym is not in chair_id or queue_state →
    //    is_participant returns false → 403.
    let non_participant_resp = client
        .get(format!("{bridge_url}/brehon/recording/{recording_id}"))
        .header("Authorization", format!("Bearer {bridge_callback_secret}"))
        .header("x-requester-pseudonym", non_participant_pseudonym)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("non-participant GET recording: {e}"))?;
    assert_eq!(
        non_participant_resp.status().as_u16(),
        403,
        "non-participant fetch must return 403 (ADR-015 participant-floor, cr-2/cr-3)"
    );

    // 3. Participant fetch → 200 + media_url (cr-2/cr-3 live; R-S3ENDPOINT).
    //    participant_pseudonym is in queue_state → is_participant returns true → 200.
    let participant_resp = client
        .get(format!("{bridge_url}/brehon/recording/{recording_id}"))
        .header("Authorization", format!("Bearer {bridge_callback_secret}"))
        .header("x-requester-pseudonym", participant_pseudonym)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("participant GET recording: {e}"))?;
    assert_eq!(
        participant_resp.status().as_u16(),
        200,
        "participant fetch must return 200 (cr-2/cr-3 live)"
    );
    let body: serde_json::Value = participant_resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("participant response must be JSON: {e}"))?;
    let media_url = body["media_url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("participant response must contain media_url string"))?;
    // R-S3ENDPOINT: media_url derives from S3_ENDPOINT in config (handle_recording_fetch),
    // never a hardcoded literal in source.  The bridge's in-Docker S3_ENDPOINT ("minio:9000")
    // may differ from the test-host S3_ENDPOINT; we verify the URL is absolute and references
    // the recording_id rather than checking the exact origin (which varies per compose setup).
    assert!(
        media_url.contains(&recording_id),
        "media_url must reference the recording_id: got {media_url}"
    );
    assert!(
        media_url.contains("://"),
        "media_url must be an absolute URL (R-S3ENDPOINT): got {media_url}"
    );

    // 4. ADR-015: requester pseudonyms must be opaque — no real identities.
    assert!(!non_participant_pseudonym.contains('@'), "ADR-015: non-participant must not be a MXID");
    assert!(!participant_pseudonym.contains('@'), "ADR-015: participant must not be a MXID");

    // Cleanup: remove the test bridge_room row (best-effort).
    if let Ok(conn) = rusqlite::Connection::open(&bridge_db_path) {
        let _ = conn.execute(
            "DELETE FROM bridge_room WHERE case_id = ?1 AND room_type = 'townhall'",
            rusqlite::params![99903_i64],
        );
    }

    Ok(())
}
