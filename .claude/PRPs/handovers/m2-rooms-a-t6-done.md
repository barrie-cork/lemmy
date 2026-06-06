# Handover: m2-rooms-a Task 6 — Room Provisioning Tests

## Status: COMPLETE — awaiting laptop validate-pending-laptop

## Last commit on phase-m2-rooms-a

`c4806ae99` — `chore(decision-queue): impl raised validate-pending-laptop for m2-rooms-a task-6`

(Preceded by `e0a331e4b` — the actual test commit.)

## DQ entry id

`d07588989c3c-001` — `kind: validate-pending-laptop`, in `pending[]`

Command: `cd services/bridge && cargo test --no-run`

## Files committed

- **Created:** `services/bridge/tests/room_provisioning.rs` — 5 `#[ignore = "requires docker-compose stack"]` tests:
  - `jury_room_provisions_in_time_with_jurors`
  - `emergency_remove_provisions_quickly`
  - `full_lifecycle_emits_10_chain_entries`
  - `restart_idempotency_no_duplicate_room_created`
  - `messaging_disabled_prevents_provisioning`
  All return `anyhow::Result<()>` with `todo!("implement against live docker-compose stack")` bodies.

- **Modified:** `services/bridge/tests/dm_round_trip.rs` — replaced `todo!("implement soft-pause cycle test")` body in `soft_pause_enable_disable_cycle` with the full T4b/T5 skeleton (8 steps, bearer-auth context, `todo!("implement soft-pause bearer-auth cycle test — no 401 expected")`).

## Import notes

- No new dependencies added — `anyhow` and `tokio` already in `services/bridge/Cargo.toml`
- Bridge toolchain boundary respected — no `--workspace --features full` (R8 constraint)
- `LemmyError` NOT used — bridge tests use `anyhow::Result<()>` throughout
