# Brief: m3-core-recording Task 3 — bridge recording primitives + the one new S3 dep

## §1 Role + dispatch line

`[role:impl-task] m3-core-recording-task3-recording-rs-sha256-sink-rust-s3-dep — see .claude/PRPs/briefs/m3-core-recording-impl-3.md`

## §2 Scope

**Plan task: `.claude/PRPs/plans/m3-core-recording.plan.md` §13 Task 3** (read it — full ACTION/IMPLEMENT/GOTCHA there). Create the bridge recording primitives module + add the ONE new generic-S3 dependency.

**Produces (3 files, ONE commit):**

1. **`services/bridge/src/recording.rs` (NEW)** — per plan §10.3:
   - `pub fn compute_content_sha256(bytes: &[u8]) -> String` — pure, `sha2::Sha256` over the bytes, `hex::encode(h.finalize())`. **Verbatim mirror of §10.3 lines 242-246.**
   - `pub trait RecordingSink` — `trigger_egress(&mut self, room_id: &str) -> anyhow::Result<()>` + `upload(&mut self, key: &str, bytes: &[u8]) -> anyhow::Result<String>`. Mirror `stage.rs`'s `GrantSink` (§10.3 mirror ref `stage.rs:24-37`).
   - `#[allow(dead_code)] pub struct LiveSink<'a>` — the REAL production sink. `trigger_egress`: build the LiveKit Egress start request, mint a token via the existing `livekit_jwt`, POST via the existing `reqwest::Client` to `config.livekit_url`. `upload`: the new `rust-s3` client, endpoint/bucket/creds from `config` (`s3_endpoint`/`s3_bucket`/`s3_access_key`/`s3_secret_key`) — **R12: NO hardcoded `minio:9000`**. `#[allow(dead_code)]` because the live town-hall-start trigger lands Phase 6.
   - `pub struct Recorder` spy (or named per the `stage.rs` `Recorder` convention) — records `trigger_egress`/`upload` calls for deterministic tests. **Mirror `stage.rs`'s `Recorder` `GrantSink` spy (§10.3 mirror ref `stage.rs:349-720`).**
   - `#[cfg(test)] mod tests`: (a) `content_sha256_is_stable` — `compute_content_sha256(b"abc")` == the known SHA-256 hex of `abc` (`ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`); (b) the `Recorder` spy compiles + records calls.

2. **`services/bridge/Cargo.toml`** — per plan §10.3 / §19(2):
   - Add the generic-S3 dep: **`rust-s3`** (crate `rust-s3`, lib imported as `s3`). This is the ONE dep with new Linux Cargo.lock resolution risk.
   - Promote `sha2` from transitive→direct: add `sha2 = "0.10"` (it is currently in `Cargo.lock` but NOT `Cargo.toml`; zero new resolution risk since already locked).
   - **Regenerate `Cargo.lock` in the SAME commit** (R14). `hex` is already a direct dep — do not re-add.

3. **`services/bridge/src/main.rs`** — add `mod recording;` **alphabetically** (after `mod provision;` / before `mod relay;`, per plan §11 line 164 / `main.rs:12-26`).

**Do NOT (scope boundaries):**
- Do NOT add `maybe_record` (that is **Task 4**).
- Do NOT add `is_participant` / the fetch route (that is **Task 5**).
- Do NOT add the 5 `RoomEventPayload` recording fields (Task 1 did the binary side `753db0c8a`; the bridge DTO mirror is **Task 4** per §10.6).
- Do NOT add `Stage::record_uploaded` (that is **Task 4** §10.5).
- Do NOT write `governance_log` directly from `recording.rs` (R11 — `content_sha256` rides `append_room_event` via the EmitIntent in Task 4; this task only computes the hash, never writes the chain).
- Do NOT touch any `crates/**` file, any migration, any const, the registry, or any second dep.
- Do NOT add a LiveKit Rust SDK (Egress reuses `reqwest` + `livekit_jwt`).

**Branch:** forks from `phase-m3-core-recording` (current tip `dc22f2a03`).

## §3 Required reading

- **`.claude/PRPs/plans/m3-core-recording.plan.md` §13 Task 3** (lines ~537-575) — the authoritative ACTION/IMPLEMENT/GOTCHA. Also §10.3 (lines 231-263, the recording.rs MIRROR), §19(2) (the rust-s3 dep decision + the aws-sdk-s3 fallback rule).
- **`services/bridge/src/stage.rs:24-37`** — `GrantCmd`/`GrantSink` trait shape to mirror for `RecordingSink`.
- **`services/bridge/src/stage.rs:349-720`** — the `Recorder` spy convention to mirror for the test spy.
- **`services/bridge/src/config.rs:61-72`** — the `s3_endpoint`/`s3_bucket`/`s3_access_key`/`s3_secret_key` `Option<String>` config fields (added Task 2) that `LiveSink` reads.
- **`services/bridge/src/bridge_room.rs:172-210`** — the `recording_config` helpers (added Task 2; `record_town_halls_enabled` etc) — context, NOT to edit.
- **Lessons (mandatory, §2.4 file-class injection):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — all `services/bridge` cargo via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`; NEVER Windows-local (`ruma-common` E0119). YOU do not run cargo (see §4).
  - `feedback_linux_compile_proof_is_a_gate.md` — Task 3 is the HEADLINE `validate-pending-laptop-linux` gate; `bm-pr` gates on `result:pass`. The dep add forces a cold Cargo.lock re-resolve where a licence/version conflict surfaces.
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ and STOP; the laptop advisor runs all cargo/Docker.
  - `feedback_cheap_model_arm_drops_adr_constraints.md` — the R11/R12/ADR-015 constraints below are load-bearing, not decorative.

## §4 Constraints

- **R11 (`content_sha256` rides the chain — load-bearing, CATCH-FIRE on bypass):** `compute_content_sha256` computes the hash with `sha2`; that is correct. The hash MUST flow `recording.rs → (Task 4) EmitIntent → drain_emits → post_room_event → append_room_event`. `recording.rs` in THIS task does **not** write `governance_log` and does **not** call `append_room_event` directly (Task 4 wires the emit). **Why it can't be deferred:** `content_sha256` is the tamper-evidence hash on the governance chain (OQ-V2-04); a raw `sha2` write that bypasses `append_room_event` skips `scrub_json` + the ed25519 hash-chain + signing — an ADR-008/ADR-016 violation. **DoD:** `grep -n "append_room_event\|governance_log" services/bridge/src/recording.rs` returns NOTHING (this task only computes; it never writes the chain).
- **R12 (generic S3, no hardcoded endpoint — load-bearing):** `LiveSink::upload` reads endpoint/bucket/creds from `config` (the `s3_*` fields). **Why:** OQ-V2-04 D5 mandates a generic S3 API so an operator can swap real S3/R2 — a hardcoded `minio:9000` breaks the swap story. **DoD:** `grep -rn "minio\.\|minio:9000\|hardcod" services/bridge/src/recording.rs` returns nothing.
- **R14 (Cargo.lock sync on dep add):** the new `rust-s3` dep's regenerated `Cargo.lock` MUST land in THIS commit. This is the ONLY task that touches deps.
- **HEADLINE GATE — rust-s3 fallback rule (do NOT silently swap):** `rust-s3` is recommended for the lighter Linux dep tree. **IF you cannot resolve `rust-s3` (Cargo.lock resolution fails — version/licence/transitive conflict), do NOT silently substitute `aws-sdk-s3`.** Instead: write a `kind: "blocker"` DQ (`from: "impl"`) naming the exact resolution error, commit + push, and STOP. The advisor ratifies the fallback to `aws-sdk-s3` (endpoint_url) per plan §19(2). Per `feedback_falsifiable_hypothesis_before_structural_fix.md` discipline — surface the real error, don't assume.
- **ADR-015 (pseudonymity):** `trigger_egress` operates on the already-pseudonymised LiveKit stream (the overlay is at JWT-issue time, m3-core-infra — §10.8). `recording.rs` adds NO identity surface. `rg -i 'person_id|username|@.*:' services/bridge/src/recording.rs` returns nothing identity-shaped.
- **NO daemon cargo.** Write a `validate-pending-laptop-linux` DQ with:
  ```
  commands: [
    "scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml",
    "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings",
    "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml"
  ]
  ```
  Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself. Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **ONE commit, 3 files** — `feat(rtc): bridge recording primitives + rust-s3 dep (task 3)`. (`Cargo.lock` is auto-regenerated, not hand-edited — it rides the commit but does not count as a hand-edit.)
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push on the worker branch, stop.
- **Handover:** deliver the HANDOVER block inline in your task output (do NOT write a `.claude/PRPs/handovers/` file — blocked by the sensitive-file guard).
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior cohort (Cohort A)

- Task 1 #736 `753db0c8a` — binary `RoomEventPayload` 5 recording fields (crates GREEN).
- Task 2 #737 `936f41a51` — `config.rs` S3 fields (`s3_endpoint`/`s3_bucket`/`s3_access_key`/`s3_secret_key`) + `bridge_room.rs` `recording_config` helpers (`write_recording_config`/`read_recording_config`/`record_town_halls_enabled`, all `#[allow(dead_code)]` until consumers land).
- fix-impl-1 `551bea68a` — `#[allow(dead_code)]` on the 3 bridge_room.rs forward-declared helpers (cleared clippy `-D warnings`).
- fix-impl-1b `d4bfd5968` — 4 `s3_* None` lines in `sanction_handler.rs` `make_config` test helper (cleared E0063).
- DQ resolution `dc22f2a03` — Cohort A validate-pending all `result:pass`.

**Lesson carried in (Task 2 brief gap):** when adding struct fields, ALL in-tree literal constructors (incl. test fixtures) need the new fields, AND forward-declared helpers need `#[allow(dead_code)]` until consumers land. Your `LiveSink` is already correctly `#[allow(dead_code)]` per §10.3.

## §5 HANDOVER (worker fills at task end — return inline in task output)

```yaml
HANDOVER:
  task: m3-core-recording-task3
  filesModified: [services/bridge/src/recording.rs, services/bridge/Cargo.toml, services/bridge/Cargo.lock, services/bridge/src/main.rs]
  keyDecisions:
    - "rust-s3 dep added (or: rust-s3 resolution FAILED → blocker DQ raised, fallback to advisor)"
    - "sha2 promoted transitive->direct (0.10)"
    - "recording.rs: compute_content_sha256 + RecordingSink trait + #[allow(dead_code)] LiveSink + Recorder spy + 2 tests"
    - "mod recording; added alphabetically to main.rs"
  validate_dq: <composite-id of the new validate-pending-laptop-linux DQ>
  notes: "<confirm content_sha256_is_stable test uses the real abc SHA-256; confirm no append_room_event/governance_log write in recording.rs; confirm no hardcoded S3 endpoint>"
```
