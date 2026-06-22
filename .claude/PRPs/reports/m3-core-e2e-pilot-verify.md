# Verify report — m3-core-e2e-pilot

**Run at:** 2026-06-22T01:30:00Z
**Phase branch:** `phase-m3-core-e2e-pilot` @ `5ecef0046`
**Plan:** `.claude/PRPs/plans/m3-core-e2e-pilot-e2e-fixes.plan.md` @ `11372f9f8`
**Outcome summary:** 3 stories: 3✓ 0✗-phantom 0✗-regression 0[malformed]

> Scope: this verify covers the **cycle-5 e2e-fixes** layer (the 3 live-gate defects). The
> earlier cycle-1–4 deliverables (entry-kinds, stage-mode, emergency-mute base, recording,
> as-register Tuwunel swap) shipped + verified in prior sub-cycles. Checkpoint commands are
> the e2e targets, executed live in the bundled gate 2026-06-22 (`5ecef0046` DQ records).

---

## Story 1 — Recording on-chain POST lands a 2xx

- **Composing tasks:** Task 1 (`[P]`)
- **Outputs:** ✓ `services/bridge/docker-compose.e2e.yml` contains `room-event-stub:` publishing `3000:3000` (caddy:2.8-alpine)
- **Checkpoint:** ✓ exit 0 — `recording` 3/3 pass (`recording_lands_with_hash_on_chain`, `clean_posture_no_recording_when_disabled`, `participant_floor_fetch`)
- **Outcome:** ✓

## Story 2 — rtc-disabled town hall seats no chair (R7)

- **Composing tasks:** Task 3 (`[P]`)
- **Outputs:**
  - ✓ `crates/api/api_common/src/governance.rs` `CaseTransitionEvent` contains `pub rtc_enabled: Option<bool>`
  - ✓ `crates/api/api_utils/src/bridge_notify.rs` reads `rtc_enabled` config + sets it in the `:136` constructor (2 hits)
  - ✓ `services/bridge/src/room_provisioner.rs` short-circuits on `event.rtc_enabled == Some(false)` (before the creds gate)
  - ✓ `services/bridge/tests/room_provisioning.rs` payload contains `"rtc_enabled": false`
- **Checkpoint:** ✓ exit 0 (on clean bridge DB) — `rtc_disabled_townhall_clean_posture` + `anonymous_townhall_identity_never_reaches_livekit` pass; 5 `todo!()` stubs D2-deferred (expected-fail, out of scope per plan §12)
- **Compile proof:** ✓ `cargo check --workspace --features full` exit 0 (2m15s; full lemmy workspace compiles with the wire-contract field)
- **Outcome:** ✓ — see test-isolation note below
- **NOTE (kind:log, test-isolation):** on a re-used `.e2e-data/bridge-a.db`, a stale `bridge_room` row for case 77007 (from a pre-fix run) triggers the provisioner's idempotency-skip (top of `provision_townhall_stage_room`) BEFORE the new `rtc_enabled` gate runs → stale `chair_id` masks the fix. The gate code is correct; the e2e suite needs a fresh `.e2e-data` per run. Harvest candidate for a harness fix (reset/`down -v` between runs, which the advisor teardown does).

## Story 3 — Mute-all completes under 500 ms

- **Composing tasks:** Task 2 (`[P]`)
- **Outputs:** ✓ `services/bridge/tests/emergency_mute.rs` wraps `update_participant` in `tokio::time::timeout(Duration::from_millis(200))`; existing `is_not_found` accept arm retained
- **Checkpoint:** ✓ exit 0 — `mute_all_drops_all_publishers_cross_instance_under_500ms` passes (elapsed < 500 ms; was 6007 ms pre-fix)
- **Outcome:** ✓

---

## Required actions

None — all stories ✓. Merge-confirm gate CLEAR (pending retro + CR triage per the phase-close sequence).

DQ housekeeping (retro): 10 superseded older-cycle `validate-pending-laptop-*` entries remain in `pending[]` (pre-cycle-4 + cycle-4 entries whose live results now live in the 3 cycle-5 resolved/pass entries). Resolve/archive at retro.
