# Brief: m3-core-recording impl-2 (Task 2)

## §1 Role + dispatch line

`[role:impl-task] m3-core-recording-task2-s3-config-record-flag-helpers — see .claude/PRPs/briefs/m3-core-recording-impl-2.md`

## §2 Scope

**Task 2 of plan `.claude/PRPs/plans/m3-core-recording.plan.md` (lines 504–535).** Add the generic-S3 operator settings to `BridgeConfig` and the `recording_config` read/write helpers + the `record_town_halls_enabled` pure flag-parse to `bridge_room.rs`. The `record_town_halls` flag lives in the EXISTING `recording_config` column (clarify DQ `a3d0e9941441-073` — m3-core-infra already shipped the column; **NO new column/migration**).

**Produces (exactly 2 file edits, ONE commit):**
1. `services/bridge/src/config.rs` — add `pub s3_endpoint: Option<String>`, `pub s3_bucket: Option<String>`, `pub s3_access_key: Option<String>`, `pub s3_secret_key: Option<String>` to `BridgeConfig` (mirror the existing `livekit_*` optionals); populate them in `from_env` from `S3_ENDPOINT`/`S3_BUCKET`/`S3_ACCESS_KEY`/`S3_SECRET_KEY` via `std::env::var(..).ok()`.
2. `services/bridge/src/bridge_room.rs` — add `read_recording_config`/`write_recording_config` (mirror `read_chair_id`/`write_chair_id`, INSERT…ON CONFLICT(case_id, room_type) DO UPDATE SET recording_config) + `record_town_halls_enabled(recording_config: &str) -> bool` (pure parse) + `#[cfg(test)]` test for the parse.

**The plan's Task 2 (lines 504–535) is the contract — follow its IMPLEMENT (file 1/2) / MIRROR / GOTCHA exactly.**

**`record_town_halls_enabled` semantics (plan §10.2):** parse the per-room `recording_config` JSON; return `true` ONLY for `{"record_town_halls": true}`; return **`false`** (default-deny) for `{"record_town_halls": false}`, `{}`, an empty string, or any unparseable garbage. Default-false is load-bearing — a missing/garbage knob must NOT enable recording (the clean-posture invariant depends on it).

**`read_recording_config`/`write_recording_config` semantics:** mirror `read_chair_id`/`write_chair_id` exactly — `write` is INSERT…ON CONFLICT(case_id, room_type) DO UPDATE SET recording_config = excluded.recording_config; `read` is SELECT recording_config WHERE case_id=?1 AND room_type=?2, returning `Option<String>` (None when no row or NULL).

**The `#[cfg(test)]` test `record_town_halls_flag_parse`:**
- `{"record_town_halls": true}` → `true`
- `{"record_town_halls": false}` → `false`
- `{}` → `false`
- `"garbage"` (unparseable) → `false`
- `""` (empty) → `false`

**R12 — generic S3, NO hardcoded endpoint:** the S3 endpoint comes from `config.s3_endpoint` (env), NEVER a hardcoded `minio:9000` in source. This task only adds the config fields; the actual S3 client wiring is Task 3. `grep -rn "minio\." services/bridge/src/` must return nothing after this task.

**Do NOT touch:** any `crates/**` file; any migration (the `recording_config` column ALREADY exists at `bridge_room.rs:15` — do NOT add a column or `ALTER TABLE`); `Cargo.toml` / `Cargo.lock` (NO new dependency in Task 2 — the `rust-s3` dep is Task 3); `recording.rs` / `stage.rs` / `room_event_client.rs` (later tasks); any file outside the 2 above.

**Branch:** forks from `phase-m3-core-recording` (current tip — the brief must be reachable there at task-spawn).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-recording.plan.md` — Task 2 (504–535), §10.2 (`record_town_halls_enabled` parse + `recording_config` helpers), §4(b) Config+flag-gate paragraph, §7 R8 (no new column/migration) + R12 (generic S3, no hardcoded endpoint).
- `services/bridge/src/bridge_room.rs:104-160` — `write_queue_state`/`read_queue_state`/`write_chair_id`/`read_chair_id` (MIRROR the INSERT…ON CONFLICT + SELECT pattern), `:15` (the `recording_config TEXT` column — ALREADY shipped), `:188,240` (column-existence test shape).
- `services/bridge/src/config.rs` — the `BridgeConfig` struct + `from_env` with the existing `livekit_*` optionals (MIRROR — add the 4 S3 optionals the same way).
- **Lessons (mandatory, §2.4 file-class injection — bridge code):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — all `services/bridge` cargo via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`); NEVER Windows-local (`ruma-common` E0119).
  - `feedback_linux_compile_proof_is_a_gate.md` — this task writes a `validate-pending-laptop-linux` DQ; `bm-pr` gates on `result:pass`.
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ and STOP; the laptop advisor runs all cargo/Docker.
  - `feedback_clippy_test_style.md` — the `#[cfg(test)]` test uses the bridge's `Result<()>` / `?` idiom (mirror the sibling `chair_id_roundtrip`/`queue_state_roundtrip` tests), no `unwrap`/`expect` under `-D warnings`.

## §4 Constraints

- **ONE commit, 2 files** — `feat(rtc): bridge S3 config + record_town_halls flag helpers (task 2)`.
- **The flag lives in the EXISTING `recording_config` column** (clarify DQ `a3d0e9941441-073`) — do NOT add a column, an `ALTER TABLE`, or a migration. If tempted, STOP and raise a `kind: "blocker"` DQ (R8 scope tripwire).
- **Default-false is load-bearing** — `record_town_halls_enabled` returns false for missing/garbage/false; only `{"record_town_halls": true}` returns true. The clean-posture invariant (Task 4) depends on this.
- **R12 — NO hardcoded S3 endpoint** — the endpoint reads from `config.s3_endpoint`/env. `grep -rn "minio\." services/bridge/src/` returns nothing after this task. A hardcoded `minio:9000` is a scope violation (catch-fire).
- **`--bins` bare fn name for bridge test filters (R13):** the validate DQ's test command uses the BARE fn name `record_town_halls_flag_parse`, NOT `bridge_room::tests::record_town_halls_flag_parse`.
- **NO new dependency** — `Cargo.toml`/`Cargo.lock` unchanged in Task 2 (the `rust-s3` dep is Task 3). If you need a new dep here, STOP and raise a `kind: "blocker"` DQ.
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml record_town_halls_flag_parse"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push on the worker branch, stop.
- **Handover:** deliver the HANDOVER block inline in your task output (do NOT write a `.claude/PRPs/handovers/` file — blocked by the sensitive-file guard in the Junior worktree context).
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior cohort

(none — first cohort)

## §5 HANDOVER (worker fills at task end — return inline in task output)

```yaml
HANDOVER:
  task: m3-core-recording-task2
  filesModified: [services/bridge/src/config.rs, services/bridge/src/bridge_room.rs]
  keyDecisions:
    - "4 S3 optionals (s3_endpoint/s3_bucket/s3_access_key/s3_secret_key) added to BridgeConfig + from_env, mirror livekit_*; endpoint from env, no hardcoded minio:9000"
    - "read_recording_config/write_recording_config mirror chair_id helpers (INSERT...ON CONFLICT); record_town_halls_enabled default-false"
    - "uses EXISTING recording_config column (no migration); registry/schema untouched"
  validate_dq: <composite-id of the validate-pending-laptop-linux DQ>
  notes: "<exact s3_* field names + record_town_halls_enabled signature Task 3 reuses for LiveSink config + Task 4 reuses for the maybe_record flag-gate>"
```
