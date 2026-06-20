# Brief: m3-core-recording Task 4 — flag-gated emission + clean-posture invariant + bridge DTO mirror

## §1 Role + dispatch line

`[role:impl-task] m3-core-recording-task4-maybe-record-record-uploaded-dto-mirror — see .claude/PRPs/briefs/m3-core-recording-impl-4.md`

## §2 Scope

**Plan task: `.claude/PRPs/plans/m3-core-recording.plan.md` §13 Task 4** (read it — full ACTION/IMPLEMENT/GOTCHA/VALIDATE there). Add the flag-gated recording controller + the emission + the 5-field bridge DTO mirror. Requires Tasks 1 (binary DTO) + 3 (recording.rs primitives) — both already on the phase branch.

**Produces (3 files modified, ONE commit):**

1. **`services/bridge/src/room_event_client.rs`** — per §10.6:
   - Add the 5 `#[serde(skip_serializing_if = "Option::is_none")]` recording fields (`media_url`, `content_sha256`, `duration_s`, `speakers`, `attendance_count`) to the bridge `RoomEventPayload` struct — mirror the `federated` field's exact serde shape (§10.6 mirror ref `room_event_client.rs:5-25`).
   - **Add the 5 fields as `None` (or `..Default::default()` if the struct derives Default) to EVERY existing test literal of `RoomEventPayload` in this file.** (R: struct-field-add propagation.)
   - Add a `room_recording_uploaded_request_json_shape` test (mirror the JSON-shape idiom at `:80-160`): build a `RoomEventRequest { entry_kind: "room_recording_uploaded", payload: {…5 recording fields Some(..)}, actor_pseudonym: Some("chair-pseudonym") }`, `to_value`, assert `entry_kind` + all 5 recording fields present/correct + `actor_pseudonym` a pseudonym + that `action`/`from_pseudonym`/`to_pseudonym`/`federated` are ABSENT (skip_serializing_if).

2. **`services/bridge/src/stage.rs`** — per §10.5:
   - **FIRST run `rg "RoomEventPayload\s*\{" services/bridge/src/stage.rs` to enumerate ALL existing `RoomEventPayload` literals (plan says 4: chair_override ×2, transfer, mute_all).** Add the 5 recording fields as `None` to EVERY one. (R: struct-field-add propagation — the build fails if ANY literal is missed.)
   - Add `pub fn record_uploaded(&mut self, media_url: String, content_sha256: String, duration_s: i64, speakers: Vec<String>, attendance_count: i32)` per §10.5 — push the `room_recording_uploaded` EmitIntent, `actor_pseudonym = self.chair.clone()`, all chair/federated fields `None`. Mirror `mute_all` push (§10.5 ref `stage.rs:316-348`).
   - Add `record_uploaded_emits_recording_intent` test: seat a chair, call `record_uploaded(..)`, assert exactly one EmitIntent with `entry_kind == "room_recording_uploaded"`, 5 recording fields populated, `actor_pseudonym == Some(chair)`.

3. **`services/bridge/src/recording.rs`** — per §10.4:
   - Add `pub fn maybe_record(enabled, sink, stage, room_id, mp4_bytes, duration_s, speakers, attendance_count) -> anyhow::Result<()>` — the `if !enabled { return Ok(()) }` flag-gate, then `sink.trigger_egress` → `compute_content_sha256` → `sink.upload` → `stage.record_uploaded(..)`. Verbatim shape from §10.4 (lines 277-295).
   - Add the **clean-posture negative-invariant** test `clean_posture_no_side_effects_when_disabled`: `Recorder` spy + seated `Stage`; call `maybe_record(false, …)`; assert spy recorded **zero** `trigger_egress`/`upload` calls AND `stage.pending_emits.is_empty()`. Add a comment documenting the **delete-the-gate check** (removing `if !enabled { return }` → false-case fires side-effects → test fails).
   - Add the **positive companion** `maybe_record(true, …)` test: records ≥1 sink call + exactly one EmitIntent (so the negative test asserts the GATE, not a trivial always-skip — R7).

**Dead-code `#[allow]` cleanup (DO THIS, then let clippy confirm):** Task 4 is the consumer that wires the forward-declared items from Tasks 2+3. After adding `maybe_record`:
- `compute_content_sha256` is now called by `maybe_record` → **remove** its `#[allow(dead_code)]` (recording.rs).
- `RecordingSink` trait methods `trigger_egress`/`upload` are now called by `maybe_record` → **remove** the `#[allow(dead_code)]` on the trait (recording.rs).
- `record_town_halls_enabled` (bridge_room.rs): only remove its `#[allow(dead_code)]` IF `maybe_record` or its caller actually reads it this task; if Task 4 does NOT call `record_town_halls_enabled` (the live flag-read may land in Task 6/Phase-6 wiring), **LEAVE its `#[allow]` in place**. Do NOT remove an `#[allow]` for a still-unused item.
- `LiveSink` struct (recording.rs:~26): its methods are the trait impl; `LiveSink` itself is still only constructed at Phase-6 live-wiring → **LEAVE its `#[allow(dead_code)]`**.
- **Rule:** remove an `#[allow(dead_code)]` ONLY for an item this task makes genuinely used in the NON-TEST build. When unsure, leave it — the validate-pending-laptop-linux clippy `-D warnings` gate will tell you (a now-USED item with a stale `#[allow]` is NOT an error; a still-UNUSED item without `#[allow]` IS). Err toward leaving `#[allow]`s; the gate catches over-removal.

**Do NOT (scope boundaries):**
- Do NOT add `is_participant` / the fetch route (that is **Task 5**).
- Do NOT write `governance_log` / call `append_room_event` directly from recording.rs or stage.rs (R11 — `content_sha256` rides the chain via the EmitIntent → drain_emits → post_room_event; the EmitIntent push is the ONLY emission mechanism).
- Do NOT touch any `crates/**` file (Task 1 did the binary DTO), any migration, any const, the registry, `Cargo.toml`/`Cargo.lock`.
- Do NOT add identity-shaped data — `speakers` + `actor_pseudonym` are PSEUDONYMS (ADR-015).

**Branch:** forks from `phase-m3-core-recording` (current tip `065117bd7`).

## §3 Required reading

- **`.claude/PRPs/plans/m3-core-recording.plan.md` §13 Task 4** (lines ~576-617) — authoritative ACTION/IMPLEMENT/GOTCHA/VALIDATE. Also §10.4 (maybe_record, lines 265-298), §10.5 (record_uploaded, 300-333), §10.6 (DTO mirror, 335-339).
- **`services/bridge/src/stage.rs:316-348`** — the `mute_all` EmitIntent push + `actor_pseudonym` shape to mirror for `record_uploaded`.
- **`services/bridge/src/stage.rs:349-720`** — the `Recorder` spy + negative-assertion test idioms.
- **`services/bridge/src/room_event_client.rs:5-25`** — the `RoomEventPayload` struct + the `federated` field serde shape to mirror for the 5 recording fields.
- **`services/bridge/src/room_event_client.rs:80-160`** — the JSON-shape test idiom.
- **`services/bridge/src/recording.rs`** (current, from Task 3) — `compute_content_sha256`, `RecordingSink` trait, `LiveSink`, `Recorder` spy — the items `maybe_record` wires together; the `#[allow(dead_code)]` attrs to selectively remove.
- **Lessons (mandatory, §2.4 file-class injection):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — all `services/bridge` cargo via `scripts/brehon/cargo-linux.sh`; NEVER Windows-local. YOU do not run cargo (see §4).
  - `feedback_linux_compile_proof_is_a_gate.md` — Task 4's validate-pending-laptop-linux DQ gates `bm-pr`.
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ and STOP.
  - `feedback_cheap_model_arm_drops_adr_constraints.md` — R7/R11/ADR-015 are load-bearing.

## §4 Constraints

- **R7 (clean-posture negative invariant — load-bearing):** the `clean_posture_no_side_effects_when_disabled` test MUST fail if the `if !enabled { return }` gate is deleted. **Why:** `record_town_halls=false` is the default clean posture — a recording that fires under the false flag is a silent data-capture regression. Pair it with the positive `enabled=true` test so the negative asserts the GATE, not a trivial always-skip. **DoD:** both tests present; the negative asserts zero `trigger_egress`/`upload` + `stage.pending_emits.is_empty()`.
- **R11 (`content_sha256` rides the chain — CATCH-FIRE on bypass):** `record_uploaded` pushes an EmitIntent carrying `content_sha256`; it does NOT write `governance_log` / call `append_room_event` directly. **Why:** the hash is the tamper-evidence link on the governance chain; bypassing `append_room_event` skips scrub_json + the ed25519 chain + signing (ADR-008/016 violation). **DoD:** `grep -nE "append_room_event|governance_log" services/bridge/src/recording.rs services/bridge/src/stage.rs` returns NOTHING (only the EmitIntent push; the binary drains it).
- **ADR-015 (pseudonymity):** `speakers` + `actor_pseudonym` are pseudonyms. **DoD:** `rg -i 'person_id|username|@.*:' services/bridge/src/recording.rs services/bridge/src/stage.rs` returns nothing identity-shaped.
- **Struct-field-add propagation (load-bearing — 3× fix-impl recurrence this phase):** adding the 5 fields to the bridge `RoomEventPayload` struct breaks EVERY existing literal that doesn't have them. **Enumerate FIRST:** `rg "RoomEventPayload\s*\{" services/bridge/src/` to list ALL literals (production + test) across stage.rs + room_event_client.rs; add the 5 fields (`None` or `..Default::default()`) to EACH in THIS commit. A missed literal = compile failure. **DoD:** the count of literals you edited == the `rg` count.
- **Dead-code `#[allow]` discipline:** remove an `#[allow(dead_code)]` ONLY for an item THIS task makes used in the NON-TEST build (`compute_content_sha256`, `RecordingSink` methods — both now called by `maybe_record`). LEAVE `#[allow]` on items still unused (`LiveSink`, `record_town_halls_enabled` if not read this task). When unsure, LEAVE it — the clippy `-D warnings` gate catches a still-unused item missing `#[allow]`, but a now-used item with a stale `#[allow]` is harmless (not an error). Err toward leaving.
- **NO daemon cargo.** Write a `validate-pending-laptop-linux` DQ with:
  ```
  commands: [
    "scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml",
    "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings",
    "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml record_uploaded_emits_recording_intent",
    "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml clean_posture_no_side_effects_when_disabled",
    "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml room_recording_uploaded_request_json_shape"
  ]
  ```
  Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself. Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **ONE commit, 3 files** — `feat(rtc): flag-gated recording emission + bridge DTO mirror (task 4)`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.
- **Handover:** deliver the HANDOVER block inline in your task output (do NOT write a `.claude/PRPs/handovers/` file — sensitive-file guard).
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior cohort / tasks

- Task 1 #736 `753db0c8a` — binary `RoomEventPayload` 5 recording fields `#[serde(default, skip_serializing_if="Option::is_none")]` (the deserialise target for the bridge POST).
- Task 2 #737 `936f41a51` (+ fix-impl-1 `551bea68a`, fix-impl-1b `d4bfd5968`) — `config.rs` S3 fields + `bridge_room.rs` `recording_config` helpers (incl. `record_town_halls_enabled`, `#[allow(dead_code)]`).
- Task 3 #739 `99622157e` (+ fix-impl-3 `8a5431b8a`) — `recording.rs`: `compute_content_sha256` + `RecordingSink` trait + `#[allow(dead_code)] LiveSink` + `Recorder` spy + 2 tests; `Cargo.toml` rust-s3 0.34 + sha2 0.10 + `[workspace]` table; `Cargo.lock` regen. The `compute_content_sha256` + `RecordingSink` `#[allow(dead_code)]` attrs are the ones Task 4 should now be able to remove (their consumer = your `maybe_record`).
- All validate DQ resolved/pass; phase tip `065117bd7`.

**Lesson carried in (3× fix-impl recurrence this phase — bridge_room.rs dead_code, sanction_handler E0063, recording.rs dead_code):** every fix-impl traced to a forward-declared-until-consumer OR struct-field-propagation gap. Task 4 is the highest-propagation task (5 fields × all literals across 2 files + consuming 2 forward-declared items). Enumerate the literals with `rg` BEFORE editing; note that `cargo test` passing does NOT prove clippy-clean (test code counts a forward-declared item as used; the non-test clippy bin-target build does not). The validate DQ runs clippy `-D warnings`, so the gate WILL catch any missed `#[allow]` / propagation.

## §5 HANDOVER (worker fills at task end — return inline in task output)

```yaml
HANDOVER:
  task: m3-core-recording-task4
  filesModified: [services/bridge/src/recording.rs, services/bridge/src/stage.rs, services/bridge/src/room_event_client.rs]
  keyDecisions:
    - "maybe_record flag-gate added; clean-posture negative + positive tests"
    - "Stage::record_uploaded EmitIntent push; record_uploaded_emits_recording_intent test"
    - "5 recording fields on bridge RoomEventPayload + propagated to N literals (rg count: <N>)"
    - "removed #[allow(dead_code)] from compute_content_sha256 + RecordingSink (now used by maybe_record); LEFT on LiveSink / record_town_halls_enabled (still unused)"
  validate_dq: <composite-id of the new validate-pending-laptop-linux DQ>
  notes: "<confirm rg literal count == edited count; confirm no append_room_event/governance_log write; confirm pseudonym-only>"
```
