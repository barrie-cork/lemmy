---
role: impl-task
phase: m2-rooms-a
task_number: "6"
base_branch: phase-m2-rooms-a
requires: ["3", "5"]
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md   # write DQ + STOP; no cargo on daemon
  # Bridge-only task — DoD is `cd services/bridge && cargo test --no-run`
  # NOT --workspace --features full (R8 toolchain boundary)
  # GOTCHA: bridge tests use anyhow, NOT LemmyError
  # Do NOT apply feedback_lemmy_error_no_std_error.md here
---

# [role:impl-task] m2-rooms-a task-6 — room_provisioning.rs e2e tests + soft_pause_enable_disable_cycle

## §1 Role + dispatch

`[role:impl-task] m2-rooms-a task-6 room provisioning tests — see .claude/PRPs/briefs/m2-rooms-a-impl-6.md`

Author the bridge integration test suite: 5 `#[ignore]` docker-compose tests in
`services/bridge/tests/room_provisioning.rs` (new file) + un-`todo!()` the
`soft_pause_enable_disable_cycle` test in `dm_round_trip.rs`.
**Bridge toolchain. DoD: `cd services/bridge && cargo test --no-run`** (compile only).

## §2 Scope

**Produce:**
1. `services/bridge/tests/room_provisioning.rs` — NEW file with 5 `#[ignore]` tests
2. `services/bridge/tests/dm_round_trip.rs` — replace `todo!()` in `soft_pause_enable_disable_cycle`

**Do NOT:**
- Touch any `crates/**` workspace files
- Use `LemmyError` / `LemmyResult` — bridge tests use `anyhow::Result`
- Run the tests (they require a live docker-compose stack; `#[ignore]` keeps them out of CI)
- Add new cargo dependencies (all required deps already in `services/bridge/Cargo.toml`)

## §3 Required reading

1. `feedback_validate_pending_laptop_write_then_stop.md` — write DQ + STOP; no cargo on daemon
2. **R8 + GOTCHA:** bridge tests use `anyhow`, NOT `LemmyError`. Do NOT apply `feedback_lemmy_error_no_std_error.md`.

**MIRROR refs — read before writing:**
- `services/bridge/tests/dm_round_trip.rs` (full file — the `#[ignore = "requires docker-compose stack"]` pattern; `#[tokio::test]` signature; how existing `todo!()` bodies are structured; the module-level doc-comment + docker-compose run instructions)
- `services/bridge/src/room_provisioner.rs` lines 60–170 (C2.1 jury path: idempotency check via `bridge_room::lookup`, `always_pseudonym` enforcement, `Juror-<suffix>` display name construction — tests assert these invariants)
- `services/bridge/src/soft_pause.rs` (full file — bearer-auth poll; `run_poller` new signature with `Arc<AtomicI64>`; `poll_once` now returns `(bool, i64)` — `soft_pause_enable_disable_cycle` test asserts no 401)
- `services/bridge/src/bridge_room.rs` (full file — `lookup` + `upsert` signatures; how idempotency watermark works — used in restart_idempotency test)

## §4 IMPLEMENT

### File 1 (new): `services/bridge/tests/room_provisioning.rs`

Module-level doc comment (mirror `dm_round_trip.rs` style):
```
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
```

Five test functions, all `#[tokio::test]` + `#[ignore = "requires docker-compose stack"]`.
Return type: `anyhow::Result<()>` (NOT `LemmyResult`). Bodies are real skeletons with
comments describing the assertion steps and a final `todo!("implement against live stack")`.

**Test 1 — `jury_room_provisions_in_time_with_jurors`:**
```
// 1. POST a CaseTransition (JurySelection, 5 juror_pseudonyms) to /brehon/room-event
// 2. Poll bridge_room::lookup(case_id, "jury") until non-None (timeout 5s)
// 3. Assert room was created within 5s
// 4. Assert exactly 5 Juror-<suffix> members (no reporter/reported/admin)
// 5. Assert always_pseudonym: no real identities in member MXIDs
```

**Test 2 — `emergency_remove_provisions_quickly`:**
```
// 1. POST a CaseTransition (EmergencyRemove) to /brehon/room-event
// 2. Poll bridge_room::lookup(case_id, "emergency") until non-None (timeout 2s)
// 3. Assert room provisioned in <2s (ADR-013)
// 4. Assert admins + legal_contact_mxid are members
// 5. Assert reported party ABSENT from member list (ADR-013)
```

**Test 3 — `full_lifecycle_emits_10_chain_entries`:**
```
// 1. Drive a full case lifecycle through all 10 room scenario transitions
// 2. Query governance_log table via AsyncPgConnection::establish(&db_url)
// 3. Assert exactly 10 Room::* entries with correct kind values
// 4. Assert each entry has valid case_id, matrix_room_id, lifecycle_stage in payload
```

**Test 4 — `restart_idempotency_no_duplicate_room_created`:**
```
// 1. POST a CaseTransition (JurySelection) to provision a jury room
// 2. Simulate bridge restart (drop and recreate AppState)
// 3. POST the same CaseTransition again
// 4. Assert bridge_room::lookup returns only 1 room (idempotency check worked)
// 5. Assert governance_log has exactly 1 Room::Created entry for this case_id
```

**Test 5 — `messaging_disabled_prevents_provisioning`:**
```
// 1. Set messaging_enabled=false via POST /governance/bridge/messaging-status bearer mock
//    (or wait for soft_pause poller to pick it up — 10s interval)
// 2. POST a CaseTransition to /brehon/room-event
// 3. Assert bridge_room::lookup returns None (no room provisioned)
// 4. Assert governance_log has zero Room::* entries for this case_id
```

All 5 bodies end with `todo!("implement against live docker-compose stack")`.

### File 2 (modify): `services/bridge/tests/dm_round_trip.rs`

Replace the `todo!()` body of `soft_pause_enable_disable_cycle` with a real skeleton:

```rust
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
```

### Validate-pending-laptop DQ entry (write + STOP)

After committing both files, append via
`bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 now>",
  "question": "Does `cd services/bridge && cargo test --no-run` pass after adding room_provisioning.rs tests?",
  "options": ["pass", "fail"],
  "context": "T6 complete: room_provisioning.rs (5 #[ignore] tests) + dm_round_trip.rs soft_pause_enable_disable_cycle un-stubbed. Bridge test compile.",
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "commands": ["cd services/bridge && cargo test --no-run"],
  "branch": "phase-m2-rooms-a",
  "phase_task": "6",
  "workflow_run_id": null,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "failed_commands": null
}
```

Commit: `chore(decision-queue): impl raised validate-pending-laptop for m2-rooms-a task-6`
Push to `origin/phase-m2-rooms-a`. Then **STOP**.

## §5 Constraints

- Commit subject: `test(bridge): add room_provisioning suite + un-stub soft_pause_enable_disable_cycle (task 6)`
- Bridge toolchain: `cd services/bridge && cargo test --no-run` (laptop only — compile check, NOT run)
- Return type: `anyhow::Result<()>` — NOT `LemmyResult<()>` (R8 + GOTCHA)
- All tests: `#[tokio::test]` + `#[ignore = "requires docker-compose stack"]`
- No new Cargo.toml dependencies
- `todo!("...")` bodies are correct and intentional — these are manual docker-compose tests

## §6 HANDOVER

Write `.claude/PRPs/handovers/m2-rooms-a-t6-done.md` with:
- last commit SHA on phase-m2-rooms-a
- DQ entry id for the new validate-pending-laptop
- confirmation both files committed (room_provisioning.rs created, dm_round_trip.rs modified)
- any import issues (`anyhow`, `tokio`)
