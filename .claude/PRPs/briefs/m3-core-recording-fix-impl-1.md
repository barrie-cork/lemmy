# Brief: m3-core-recording fix-impl-1 (Task 2 clippy dead-code)

## §1 Role + dispatch line

`[role:impl-task] m3-core-recording-fix-impl-1-bridge-room-allow-dead-code — see .claude/PRPs/briefs/m3-core-recording-fix-impl-1.md`

## §2 Scope

**Mechanical fix for the Task 2 `validate-pending-laptop-linux` clippy failure (DQ `4ee45cbb6d67-001`).** The bridge clippy gate (`cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings`) FAILS with 3 dead-code errors:

```
error: function `write_recording_config` is never used   --> src/bridge_room.rs:172
error: function `read_recording_config` is never used    --> src/bridge_room.rs:188
error: function `record_town_halls_enabled` is never used --> src/bridge_room.rs:208
error: could not compile `brehon-bridge` due to 3 previous errors
```

These 3 helpers are **forward-declared by-design** — their consumers land in Task 4 (`maybe_record` reads the flag via `record_town_halls_enabled`) and Task 4/5 (`read_recording_config`/`write_recording_config`). Under `-D warnings` the not-yet-wired functions are dead-code errors. The fix is the EXACT pattern emergency-mute used for its forward-declared `mute_all_power_levels` (`#[allow(dead_code)]` until the consumer lands), and `bridge_room.rs` already uses this attribute elsewhere (line 93).

**Produces (exactly 1 file edit, ONE commit):**
1. `services/bridge/src/bridge_room.rs` — add `#[allow(dead_code)]` on its OWN line directly ABOVE each of the 3 `pub fn` signatures:
   - above `pub fn write_recording_config(` (currently line 172)
   - above `pub fn read_recording_config(` (currently line 188)
   - above `pub fn record_town_halls_enabled(recording_config: &str) -> bool {` (currently line 208)

Mirror the existing `#[allow(dead_code)]` at `bridge_room.rs:93` verbatim (same attribute, same indentation = column 0, no `(reason = ...)`). Three single-line insertions. Do NOT change any fn body, signature, doc-comment, or the existing `#[allow]` at :93.

**Do NOT touch:** any `crates/**` file; any migration; `Cargo.toml`/`Cargo.lock`; `config.rs`; `recording.rs`/`stage.rs`/`room_event_client.rs`; any file outside `bridge_room.rs`; any fn body. This is a 3-line attribute-only fix.

**Branch:** forks from `phase-m3-core-recording` (current tip — the brief + fix land on the phase branch).

## §3 Required reading

- `services/bridge/src/bridge_room.rs:93` — the EXISTING `#[allow(dead_code)]` to mirror (verbatim attribute shape).
- `services/bridge/src/bridge_room.rs:172-210` — the 3 target fns (read to confirm exact current line positions before editing; line numbers may drift ±1 — anchor on the `pub fn <name>(` signature, NOT the absolute line).
- **Lessons (mandatory, §2.4 file-class injection — bridge code):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — all `services/bridge` cargo via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`; NEVER Windows-local (`ruma-common` E0119).
  - `feedback_linux_compile_proof_is_a_gate.md` — this fix re-validates the Task 2 clippy gate; the new `validate-pending-laptop-linux` DQ gates `bm-pr`.
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ and STOP; the laptop advisor runs all cargo/Docker.
  - `feedback_clippy_rerun_after_fix.md` — re-run clippy after the fix to confirm the 3 errors clear (the advisor does this, not you — you write the DQ).

## §4 Constraints

- **ONE commit, 1 file** — `fix(rtc): #[allow(dead_code)] on forward-declared recording_config helpers (fix-impl 1)`.
- **Attribute-only** — add 3 `#[allow(dead_code)]` lines; change NOTHING else. If you find yourself editing a fn body or signature, STOP — that's out of scope.
- **Anchor on the fn signature, not the line number** — line numbers in the error output are from a prior snapshot and may have drifted; locate each `pub fn <name>(` and insert the attribute directly above it.
- **Mirror `:93` verbatim** — `#[allow(dead_code)]` at column 0, no reason string (match the existing in-file style exactly).
- **NO daemon cargo.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml record_town_halls_flag_parse"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push on the worker branch, stop.
- **Handover:** deliver the HANDOVER block inline in your task output (do NOT write a `.claude/PRPs/handovers/` file — blocked by the sensitive-file guard in the Junior worktree context).
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior cohort

(Cohort A: Task 1 #736 `753db0c8a` RoomEventPayload 5 recording fields — crates GREEN; Task 2 #737 `936f41a51` S3 config + record_town_halls helpers — bridge check GREEN, clippy FAIL on the 3 dead-code helpers this fix-impl resolves.)

## §5 HANDOVER (worker fills at task end — return inline in task output)

```yaml
HANDOVER:
  task: m3-core-recording-fix-impl-1
  filesModified: [services/bridge/src/bridge_room.rs]
  keyDecisions:
    - "#[allow(dead_code)] added above write_recording_config / read_recording_config / record_town_halls_enabled (forward-declared; consumers land Task 4/5)"
    - "mirrors existing bridge_room.rs:93 #[allow(dead_code)] + emergency-mute mute_all_power_levels precedent"
    - "attribute-only; no fn body/signature change"
  validate_dq: <composite-id of the new validate-pending-laptop-linux DQ>
  notes: "<confirm the 3 fn names + that no other file changed>"
```
