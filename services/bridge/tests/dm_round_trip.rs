// Integration test: docker-compose-gated DM round-trip (Task 13).
//
// Run with:
//   cd services/bridge
//   docker compose up -d
//   cargo test --test dm_round_trip -- --ignored
//   docker compose down
//
// All test functions are #[ignore]'d so bare `cargo test` inside
// services/bridge/ skips this suite without requiring the docker stack.
// Full implementation is M2 scope (brief §2 M1 scope note).

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn dm_text_round_trip() {
    // 1. Wait for Tuwunel to be ready (HTTP poll /_matrix/client/v3/versions)
    // 2. Send a text DM via the bridge relay endpoint
    // 3. Assert response arrives at matrix-sdk client within 3s (criterion #1)
    todo!("implement when docker-compose stack is wired")
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn dm_image_round_trip() {
    // 1. Upload an image to the bridge
    // 2. Assert m.image event arrives at recipient puppet within 3s
    todo!("implement when docker-compose stack is wired")
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn dm_voice_round_trip() {
    // 1. Upload a voice note (m.audio with org.matrix.msc3245.voice)
    // 2. Assert event arrives at recipient puppet within 3s
    todo!("implement when docker-compose stack is wired")
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn soft_pause_enable_disable_cycle() {
    // T4b/T5: bearer-authed poll now uses /governance/bridge/messaging-status.
    // This test asserts the 401 bug is fixed (T4b) and the cycle works end-to-end.
    //
    // 1. Verify bridge is started with BRIDGE_CALLBACK_SECRET set
    // 2. Assert initial relay_enabled=true (messaging_enabled=true in Brehon)
    // 3. POST to /governance/admin/messaging-config to set messaging_enabled=false
    // 4. Wait ≤20s for soft_pause poller to detect change (10s interval + margin)
    // 5. Assert relay_enabled=false (POST /brehon/room-event returns 403)
    // 6. POST to restore messaging_enabled=true
    // 7. Wait ≤20s for poller to re-enable
    // 8. Assert relay_enabled=true (POST /brehon/room-event returns 200)
    todo!("implement soft-pause bearer-auth cycle test — no 401 expected")
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn restart_persistence() {
    // Criterion #2: admin-panel restart-persistence fixture.
    // 1. Set messaging config via admin API (identity_policy, hard_delete_after_days)
    // 2. Restart the bridge process
    // 3. Read config back and assert persisted values match (reads from Brehon DB,
    //    not bridge-local state, so restart is transparent)
    todo!("implement restart fixture")
}
