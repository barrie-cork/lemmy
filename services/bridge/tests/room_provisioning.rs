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
