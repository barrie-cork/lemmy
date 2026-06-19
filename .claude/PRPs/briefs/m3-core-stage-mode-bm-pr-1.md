---
role: bm-task
verb: bm-pr
phase: m3-core-stage-mode
base_branch: governance-v0
created: 2026-06-19
---

# bm-task brief — m3-core-stage-mode bm-pr

**Role:** `[role:bm-task]`
**Verb:** `bm-pr`
**Phase:** `m3-core-stage-mode`
**Base branch:** `governance-v0`

## Dispatch line

```
[role:bm-task] m3-core-stage-mode bm-pr — see .claude/PRPs/briefs/m3-core-stage-mode-bm-pr-1.md
```

## Scope

Open a PR from `phase-m3-core-stage-mode` into `governance-v0` on `barrie-cork/lemmy`.

- Title: `feat(rtc,bridge): M3 town-hall stage mode — chair-controlled mic-passing, FIFO raised-hand queue, 30s grace auto-revoke, chair transfer/override, FIRST room_chair_* chain emission`
- NOT a draft (CodeRabbit must review)
- `--repo barrie-cork/lemmy` mandatory

## PR body

```
## Summary

Adds **chair-controlled stage mode** to M3 town-hall rooms (Phase 3, bridge-side): a single presenter slot the chair drives, a FIFO raised-hand queue, mic-passing via LiveKit publish-grants (not Matrix power-levels), a 30s grace auto-revoke + next-promote boundary, chair transfer (delegate) + chair override (force promote/demote), and the **FIRST emission** of `room_chair_transferred` / `room_chair_override` governance-log chain entries from the bridge. All chain payloads are pseudonyms-only (ADR-015); the Q&A sidebar is the Matrix text timeline, never hashed (ADR-016). The deterministic DoD is unit tests (no e2e); the docker-gated end-to-end is an #[ignore]'d compile-gated stub (Phase-6 pilot grade).

- **`RoomEventPayload` chair-action fields** (binary): 5 optional `#[serde(default)]` fields (`action`, `target_pseudonym`, `from_pseudonym`, `to_pseudonym`, `at`) + `CaseTransitionEvent.chair_pseudonym`; non-breaking (zero existing constructors); `bridge_notify.rs` sets `chair_pseudonym: None` (Phase-3 default) (Task 1).
- **LiveKit publish-grant mint** (`livekit_jwt.rs::mint_access_token(can_publish)`): presenter token (`can_publish=true`) vs watcher token (`false`) — mic = publish-grant re-mint, NO Matrix power-level call (Task 2).
- **Stage state machine** (`services/bridge/src/stage.rs` NEW): chair seat + persisted FIFO raised-hand queue (`bridge_room.queue_state`) + mic-pass state machine (`raise_hand`/`promote_next`/`on_activate`) + `GrantSink` adapter + type-state `Result<()>` guards; FIFO survives bridge restart via `Stage::load` (Task 3).
- **30s grace auto-revoke** (`run_grace`/`on_grace_expired`/`GRACE_SECS=30`): a promoted speaker who doesn't activate within 30s is auto-revoked and the next is promoted — deterministic `#[tokio::test(start_paused)]` virtual-time boundary test (the load-bearing boundary) (Task 4).
- **Bridge→binary room-event client** (`room_event_client.rs` NEW): `post_room_event` POSTs chair-action METADATA to `/api/v4/governance/room-event` with Bearer auth; the emit-intent seam (`Stage::pending_emits`) keeps `transfer_chair`/`chair_override`/`promote_next` synchronous; `drain_emits` is the async controller drain (fire-and-forget warn-on-error) wired from the provisioning path (Tasks 5+6).
- **Stage-mode provisioning + Q&A sidebar** (`room_provisioner.rs`): town-hall path opens in stage mode (chair presenter, watchers muted), seats `chair_id` from `chair_pseudonym ?? juror_pseudonyms.first()` (foreperson fallback, OQ-V2-05), initialises the FIFO to `[]`, drains room-event intents; the Q&A sidebar IS the Matrix text timeline (no new room type, no content hashing) (Task 6).
- **Docker-gated e2e stub** (`services/bridge/tests/stage_mode.rs` NEW): `#[ignore]` compile-gated end-to-end asserting `room_chair_transferred {from,to,at}` + `room_chair_override {action,target_pseudonym}` rows land with pseudonym payloads on a live stack (Phase-6 pilot grade) (Task 6).

## Validation

- **Bridge Linux-compile gate** (`cargo-linux.sh check + clippy --no-deps -- -D warnings`, Docker rust:1.95): EXIT 0 @ `9152aa4d6` (advisor-laptop, DQ bc10e175f0da-001). clippy 0 errors — the Task-5 scaffold dead_code allows were removed when Task 6's `drain_emits` made `post_room_event`/`RoomEventRequest`/`brehon_room_event_url` live.
- **Bridge unit tests** (`cargo-linux.sh test`): `stage` 10 passed, 0 failed (incl `fifo_mic_pass_in_sequence` marquee, `grace_no_activate_auto_revokes_and_promotes_next` 30s virtual-time boundary at 0.04s, 2 emit-intent tests); `room_event_client` 2 passed, 0 failed (override/transfer JSON shape).
- **Integration-test compile** (`cargo-linux.sh test --test stage_mode --no-run`): EXIT 0 — the docker-gated `#[ignore]` `stage_mode.rs` test compiles (1 filtered/not-run); live run is Phase-6 pilot.
- **crates static** (Task 1, Windows `--workspace --features full`): check + clippy EXIT 0 (governance-log chair-payload serde roundtrip).
- **No migration** (no new `bridge_room` column; `chair_id`/`queue_state` from Task 3 m3-core-stage-mode). **No new dep.**

## Plan reference

`.claude/PRPs/plans/m3-core-stage-mode.plan.md` — Tasks 1–6 + §16a Stories 1–5.

## Commit log (phase diff)

See `git log governance-v0..phase-m3-core-stage-mode --oneline`
```

## Required reading

- `.claude/rules/branch-manager.md`
- `.claude/rules/gh-pr-fork-target.md`
- `.claude/commands/bm/bm-pr.md`

## Constraints

- Base MUST be `governance-v0` (never `main`)
- `--repo barrie-cork/lemmy` on all `gh pr` commands
- NOT a draft (CodeRabbit skips drafts)
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`, `services/bridge/**` — PR-open only, no code edits
- Body assembles from this brief + `git log governance-v0..phase-m3-core-stage-mode --oneline`
