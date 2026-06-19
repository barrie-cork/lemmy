// Integration tests: stage-mode town-hall suite (docker-compose-gated).
//
// Run with:
//   cd services/bridge
//   docker compose up -d
//   cargo test --test stage_mode -- --ignored
//   docker compose down
//
// All test functions are #[ignore]'d so bare `cargo test` inside
// services/bridge/ skips this suite without requiring the docker stack.

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn four_mic_pass_then_grace_boundary_emits_chair_entries() -> anyhow::Result<()> {
    // 1. provision a town-hall stage room (chair + 4 watchers)
    // 2. drive 4 raise-hand + promote/activate passes via the real LiveKit grants
    // 3. drive a 5th promote with no activation; assert auto-revoke + next-promote at 30s
    // 4. drive a chair transfer + a chair override
    // 5. query governance_log: assert room_chair_transferred {from_pseudonym, to_pseudonym, at}
    //    + room_chair_override {action, target_pseudonym} rows exist with PSEUDONYM payload
    //    fields (never a real identity or Matrix user ID — ADR-015)
    todo!("implement against live docker-compose stack (Phase-6 pilot grade)")
}
