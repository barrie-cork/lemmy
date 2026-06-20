// Integration tests: recording end-to-end suite (docker-compose-gated).
//
// Run with:
//   cd services/bridge
//   docker compose up -d
//   cargo test --test recording -- --ignored
//   docker compose down
//
// All test functions are #[ignore]'d so bare `cargo test` inside
// services/bridge/ skips this suite without requiring the docker stack.

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn recording_lands_with_hash_on_chain() -> anyhow::Result<()> {
    // 1. provision a town-hall room with `record_town_halls=true` in recording_config
    // 2. trigger Egress (LiveKit Egress start via the bridge's LiveSink using
    //    reqwest + livekit_jwt; the Egress captures the pseudonymous overlay at
    //    JWT-issue time — the `always_pseudonym` guarantee is inherited, not re-tested here)
    // 3. assert the MP4 object exists in MinIO (S3 object-exists check on the recording key;
    //    the generic-S3 endpoint comes from config, never a hardcoded minio:9000 — R12)
    // 4. assert a `governance_log` `room_recording_uploaded` row carries the schema
    //    `{media_url, content_sha256, duration_s, speakers, attendance_count}` where the
    //    real `content_sha256` matches `compute_content_sha256(mp4_bytes)` AND the chain
    //    row carries a valid signature + prev-hash link — the hash RODE `append_room_event`
    //    (R11), NOT a local variable or side-channel bypass digest (ADR-008/ADR-016)
    // 5. assert `speakers` + the uploader `actor_pseudonym` are PSEUDONYMS (ADR-015 —
    //    never `person_id`, username, or MXID; only pseudonyms reach the hash chain)
    todo!("implement against live docker-compose stack (Phase-6 pilot grade)")
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn clean_posture_no_recording_when_disabled() -> anyhow::Result<()> {
    // 1. provision a town-hall room with `record_town_halls=false` in recording_config
    // 2. run the town-hall session (no Egress trigger expected — the flag gate
    //    `if !enabled { return }` in `maybe_record` must suppress all recording side-effects;
    //    the deterministic unit equivalent is Task 4's
    //    `clean_posture_no_side_effects_when_disabled`)
    // 3. assert NO MinIO object exists for this room, NO `room_recording_uploaded` chain
    //    row in `governance_log`, NO Egress call was issued — the integration-level
    //    clean-posture invariant (zero recording side-effects when the flag is off)
    todo!("implement against live docker-compose stack (Phase-6 pilot grade)")
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn participant_floor_fetch() -> anyhow::Result<()> {
    // 1. a non-participant requester pseudonym fetches the recording via
    //    GET /brehon/recording/{id} → assert HTTP 403 (ADR-015 participant-floor;
    //    `is_participant` is called before serving — a recording must not leak to
    //    non-participants of the town-hall room)
    // 2. a participant requester pseudonym fetches the recording via the same route →
    //    assert HTTP 200 + the `media_url` in the response body
    todo!("implement against live docker-compose stack (Phase-6 pilot grade)")
}
