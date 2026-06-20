---
role: bm-task
verb: bm-pr
phase: m3-core-recording
base_branch: governance-v0
created: 2026-06-20
---

# bm-task brief — m3-core-recording bm-pr

**Role:** `[role:bm-task]`
**Verb:** `bm-pr`
**Phase:** `m3-core-recording`
**Base branch:** `governance-v0`

## Dispatch line

```
[role:bm-task] m3-core-recording bm-pr — see .claude/PRPs/briefs/m3-core-recording-bm-pr-1.md
```

## Scope

Open a PR from `phase-m3-core-recording` into `governance-v0` on `barrie-cork/lemmy`.

- Title: `feat(rtc,bridge): M3 town-hall optional recording — Egress→S3→content_sha256 chain emission + participant-floor fetch, FIRST room_recording_uploaded entry`
- NOT a draft (CodeRabbit must review)
- `--repo barrie-cork/lemmy` mandatory

## PR body

```
## Summary

Adds **optional evidentiary-store recording** to M3 town-hall rooms (Phase 5, bridge-side): when `record_town_halls = true` for a room, the bridge triggers **LiveKit Egress** → an MP4 lands in a **generic-S3 object store** (MinIO reference backend) → the bridge computes the MP4's **`content_sha256`** → and pushes the **FIRST** `room_recording_uploaded` governance-log chain entry (the const was registered in Phase 2; this is emit-only — entry-kind registry stays 72). The hash reaches the chain ONLY via the existing `EmitIntent → drain_emits → post_room_event → append_room_event` seam (R11 — never a side-channel digest). A `record_town_halls = false` room has a defined **zero-side-effect clean posture** (the negative invariant). A **participant-floor fetch** endpoint serves a recording only to a participant of the room (ADR-015 — the floor cannot be zero under `always_pseudonym`, else a pseudonymous town hall's audio/video leaks). The `record_town_halls` flag is a knob in the EXISTING `bridge_room.recording_config TEXT` column (no new migration). The ONE new dep is a generic-S3 client (`rust-s3`); the Egress trigger reuses the existing `reqwest` + `livekit_jwt`. All chain payloads are pseudonyms-only (ADR-015); `room_recording_uploaded` carries metadata only — `{media_url, content_sha256, duration_s, speakers, attendance_count}` + top-level `actor_pseudonym` — the MP4 bytes go to S3, never the chain (ADR-016). Deterministic DoD is unit tests; the docker-gated end-to-end is an `#[ignore]`'d compile-gated stub (Phase-6 pilot grade).

- **Binary `RoomEventPayload` recording fields** (`governance_log.rs`): five optional `#[serde(default, skip_serializing_if = "Option::is_none")]` fields — `media_url`, `content_sha256`, `duration_s`, `speakers: Option<Vec<String>>` (pseudonyms), `attendance_count` — so the `room_recording_uploaded` chain entry carries the Success-Criteria schema; non-breaking (skip-if-none keeps existing room emissions byte-identical); `#[cfg(test)]` case asserts present-when-Some + omitted-when-None (Task 1, Windows `--workspace --features full` validated).
- **Bridge S3 config + `record_town_halls` flag-gate** (`config.rs` + `bridge_room.rs`): 4 operator-level generic-S3 settings (`s3_endpoint`/`s3_bucket`/`s3_access_key`/`s3_secret_key`, all `Option<String>` — endpoint from config/env, NEVER a hardcoded `minio:9000`); `read_recording_config`/`write_recording_config` mirror the `chair_id` helpers (INSERT…ON CONFLICT); pure `record_town_halls_enabled(&str) -> bool` parses the per-room knob from the existing column (no new column/migration) (Task 2).
- **Recording primitives + the one new dep** (`recording.rs` NEW): `compute_content_sha256(&[u8]) -> String` (pure `sha2::Sha256` hex); a `RecordingSink` trait (mirror `stage.rs`'s `GrantSink`) with `trigger_egress` + `upload`, a real impl (Egress via `reqwest` + `livekit_jwt`; S3 PUT via `rust-s3`) and a spy for tests. The ONE new dep is `rust-s3` 0.34; `Cargo.lock` regenerated in the SAME commit (Task 3).
- **Flag-gated emission + clean-posture invariant** (`recording.rs::maybe_record` + `stage.rs::record_uploaded`): `maybe_record` returns immediately with ZERO side-effects when `!enabled` (no Egress, no S3 PUT, no EmitIntent); when enabled it drives the sink + pushes the `room_recording_uploaded` EmitIntent (drained by the existing seam). The bridge `room_event_client::RoomEventPayload` mirror gains the 5 fields. The clean-posture negative-invariant test asserts zero side-effects under `enabled=false` AND FAILS if the `if !enabled { return }` gate is deleted (delete-the-gate mechanical check), with a positive `enabled=true` companion (Task 4).
- **Participant-floor fetch** (`recording.rs::is_participant` + `appservice.rs`): a pure `is_participant(requester_pseudonym, participants: &[String]) -> bool` gate called BEFORE serving in `handle_recording_fetch` (route `/brehon/recording/{id}`, mirror `handle_room_event`'s Bearer-`BRIDGE_CALLBACK_SECRET` auth) — non-participant → 403, participant → 200; empty participant set → false (the floor cannot be zero, ADR-015). The participant set + requester are pseudonyms (Task 5).
- **Docker-gated end-to-end stub** (`services/bridge/tests/recording.rs` NEW): 3 `#[ignore]` compile-gated tests — (1) `recording_lands_with_hash_on_chain` (MP4 in MinIO + a `room_recording_uploaded` chain row carrying the real `content_sha256` with a valid signature/prev-hash link + pseudonym speakers/actor); (2) `clean_posture_no_recording_when_disabled` (zero side-effects); (3) `participant_floor_fetch` (403/200). `todo!()` bodies (Phase-6 pilot grade) (Task 6).

## Validation

- **Bridge Linux-compile gate** (`cargo-linux.sh check + clippy --no-deps -- -D warnings` + unit `test`, Docker rust:1.95): EXIT 0 @ phase tip (advisor-laptop). All 5 bridge `validate-pending-laptop-linux` DQ at `result: pass` (Cohort A `4ee45cbb6d67-001`, Task 3 `031b1fbf8eb1-001`, Task 4 `96dd2bccbd79-001`, Task 5 `cdbea3487eca-001`, Task 6 `0aa481cce3a6-001`). The `rust-s3` 0.34 dep resolved clean on Linux first-try (no Cargo.lock licence/version conflict).
- **Bridge unit tests** (`cargo-linux.sh test`): `record_town_halls_flag_parse`, `record_uploaded_emits_recording_intent`, `clean_posture_no_side_effects_when_disabled` (delete-the-gate negative invariant), `is_participant_floor` — all pass; the `#[ignore]` integration tests skipped.
- **Integration-test compile** (`cargo-linux.sh test --test recording --no-run`): EXIT 0 — `recording.rs` `#[ignore]` stub compiles (Executable `recording-2cb64ee9` built; live run is Phase-6 pilot).
- **crates static** (Task 1, Windows `--workspace --features full`): check + clippy EXIT 0 (5-field serde roundtrip; non-breaking optional fields).
- **No migration** (no new DB column; `record_town_halls` is a knob in the existing `recording_config` column; `room_recording_uploaded` const shipped Phase 2). **Exactly ONE new dep** (`rust-s3`). **Registry count UNCHANGED at 72** (emit-only).

## Plan reference

`.claude/PRPs/plans/m3-core-recording.plan.md` — Tasks 1–6 + §16a Stories 1–5.

## Commit log (phase diff)

See `git log governance-v0..phase-m3-core-recording --oneline`
```

## Required reading

- `.claude/rules/branch-manager.md`
- `.claude/rules/gh-pr-fork-target.md`
- `.claude/commands/bm/bm-pr.md`

## Constraints

- Base MUST be `governance-v0` (never `main`)
- `--repo barrie-cork/lemmy` on all `gh pr` commands
- NOT a draft (CodeRabbit skips drafts)
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`, `services/bridge/**`, `docs/**`, plan/PRD files — PR-open only, no code edits
- Body assembles from this brief + `git log governance-v0..phase-m3-core-recording --oneline`
- If a PR already exists on this branch (`gh pr view`), fall through to `gh pr edit --body-file` instead of `gh pr create`
