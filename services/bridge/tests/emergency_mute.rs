// Integration tests: emergency-mute cross-instance suite (docker-compose-gated).
//
// Run with:
//   cd services/bridge
//   docker compose up -d   # requires two federated bridge instances
//   cargo test --test emergency_mute -- --ignored
//   docker compose down
//
// All test functions are #[ignore]'d so bare `cargo test` inside
// services/bridge/ skips this suite without requiring the docker stack.

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn mute_all_drops_all_publishers_cross_instance_under_500ms() -> anyhow::Result<()> {
    // 1. provision a federated town-hall stage room across two bridge instances
    //    (chair + N publishers), with participants whose home instance differs from
    //    the chair's home instance so the cross-instance path is exercised.

    // 2. record a per-publisher publish-active baseline measured at the publisher
    //    client (not the server) — confirm each non-chair publisher can publish
    //    before mute-all fires (the pre-condition for the <500ms assertion).

    // 3. fire mute-all: invoke mute_all_power_levels (the cross-instance Matrix
    //    m.room.power_levels PUT — raises the publish/voice threshold above
    //    users_default so every non-elevated participant loses publish in one PUT,
    //    which Matrix federation propagates to the remote instance; OQ-V2-06) AND
    //    the local Stage::mute_all sweep (RevokePublish for every locally-known
    //    publisher — the in-instance belt-and-suspenders; instant local effect).

    // 4. assert EVERY non-chair publisher's publish right is revoked measured at
    //    the publisher client within 500ms of the mute-all call.
    //    Zero-holder negative invariant, cross-instance: no listed publisher retains
    //    a publish grant after mute-all, regardless of which bridge instance they
    //    are homed on.
    //    PRD fallback (document, not drop): if cross-instance <500ms proves
    //    infeasible at the Phase-6 pilot due to Matrix federation propagation
    //    latency, the fallback is a best-effort cross-instance SLA combined with
    //    the in-instance <500ms guarantee provided by Stage::mute_all's LiveKit
    //    RevokePublish sweep; a DQ entry surfaces before declaring this criterion
    //    failed — it is never silently dropped.

    // 5. query governance_log: assert a room_mute_all row exists with
    //    actor_pseudonym equal to the chair PSEUDONYM (an opaque pseudonym string —
    //    ADR-015: only pseudonyms reach the hash chain, never a real user identity
    //    or a Matrix user address) and payload.federated == true (ADR-016: metadata
    //    only, no speech/video bytes).

    todo!("implement against live docker-compose stack (Phase-6 pilot grade)")
}
