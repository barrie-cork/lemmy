# /brehon-verify — m3-core-stage-mode

**Phase branch:** `phase-m3-core-stage-mode` @ `9152aa4d6`
**Verified:** 2026-06-19
**Outcome:** ✅ ALL STORIES PASS — no phantoms, no checkpoint failures

## §16a Story verification

| Story | Checkpoint | Brief-Scope output | Result |
|---|---|---|---|
| 1 — chair drives 4 mic-passes (marquee) | `test stage::fifo_mic_pass_in_sequence` | `stage.rs` `promote_next`/`raise_hand`/`on_activate` + FIFO `write_queue_state` | ✅ test passed (validate-719); symbols present |
| 2 — 30s no-activate auto-revoke + next-promote (boundary) | `test stage::grace_no_activate_auto_revokes_and_promotes_next` | `stage.rs` `on_grace_expired` + `GRACE_SECS=30` + `#[tokio::test(start_paused)]` | ✅ test passed (0.04s = virtual-time); symbols present |
| 3 — chair override emits `room_chair_override` (pseudonym target) | `test room_event_client::override_*` | `room_event_client.rs` `post_room_event`; `stage.rs::chair_override` pushes intent; `RoomEventPayload.action`/`target_pseudonym` | ✅ 2 room_event_client tests passed; payload fields present |
| 4 — chair transfer emits `room_chair_transferred` | `test room_event_client::transfer_*` | `stage.rs::transfer_chair` returns `(from,to)` + pushes intent; `RoomEventPayload.from/to/at` | ✅ passed; fields present |
| 5 — town-hall opens in stage mode, Q&A live, e2e emits chair entries | `test --test stage_mode --no-run` (compile-gate) | `room_provisioner.rs` stage-mode path seats `chair_id` (governance ?? foreperson) + Q&A=Matrix timeline; `tests/stage_mode.rs` asserts pseudonym chair rows | ✅ `stage_mode.rs` compiles (`--no-run`, `#[ignore]` not run); provisioning seats chair via `drain_emits` + `write_chair_id` |

## Validation summary (bridge Linux, advisor-laptop, DQ bc10e175f0da-001)

- `cargo-linux check --manifest-path services/bridge/Cargo.toml` → **rc=0**
- `cargo-linux clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings` → **rc=0** (3 dead_code allows removed; `drain_emits` is the production caller making the emitter live)
- `cargo-linux test --test stage_mode --no-run` → **rc=0** (docker-gated `#[ignore]` integration test compiles; live run is Phase-6 pilot)
- `cargo-linux test stage` → **10 passed, 0 failed**
- `cargo-linux test room_event_client` → **2 passed, 0 failed**

## Conformance-audit detection (§3.9.1)

NOT triggered — the conformance-audit fires on `crates/apub/activities/src/governance/**`, `crates/api/api/src/governance/**`, or `crates/db_schema/src/source/governance/**`. The m3-core-stage-mode diff is bridge-side (`services/bridge/**`) + the Task 1 `RoomEventPayload`/`CaseTransitionEvent` field adds (`crates/api/api_common/src/governance.rs`, `governance_log.rs`) — the latter are additive DTO fields, not governance-handler logic. No Tier-1 audit scope.

## ADR conformance (§15.5 cross-cutting)

- **ADR-015 (pseudonyms-only):** `chair_id` seated from `chair_pseudonym ?? juror_pseudonyms[0]` (both pseudonyms); `room_event_client` payload fields are `*_pseudonym`; new Task-6 lines grep-clean for `person_id`/username/MXID in the chain payload. ✅
- **ADR-016 (metadata-only):** Q&A sidebar = Matrix text timeline (no new room type, no content field, never hashed/POSTed). `post_room_event` carries chair-action metadata only. ✅
- **FIRST emitter:** `room_chair_transferred`/`room_chair_override` emit-intents pushed in `stage.rs`, drained to `post_room_event` via `drain_emits` from the provisioning path. ✅
- **No migration, no new dep.** ✅

## Verdict

✅ **Advance to bm-pr.** All 6 impl tasks shipped; all 5 stories verified; bridge Linux-compile gate at `result:pass`. No phantom completions, no regressions.
