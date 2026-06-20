# Brief: m3-core-e2e-pilot Task 2 — recording carry-forwards (cr-2 + cr-3) + live LiveSink (async Egress + S3 PUT)

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-task2-recording-live-sink-cr2-cr3 — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-2.md`

You are the **impl-task** subagent (Sonnet 4.6). This is the ONE substantive code change of the phase: make `RecordingSink` async, wire the live recording trigger (Egress POST + S3 PUT replacing the two `bail!` stubs), and land the cr-2/cr-3 recording-fetch carry-forwards. Bridge-src, **Linux-validated only**.

## §2 Scope

**Plan task: `.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` §13 Task 2** (read it — full ACTION/IMPLEMENT/GOTCHA/VALIDATE there). Also §10.3 (RecordingSink async + live LiveSink), §10.4 (cr-2/cr-3), §19 (2) (no async-trait rationale).

**Produces (≤3 files modified, ONE commit):**

1. **`services/bridge/src/recording.rs`** — per §10.3:
   - Convert `RecordingSink::{trigger_egress, upload}` to **`async fn`** (native async-fn-in-trait, Rust 1.95 — **NO `async-trait` dep**, §19 (2)).
   - Make `maybe_record` **generic over `S: RecordingSink`** (`maybe_record<S: RecordingSink>(…, sink: &mut S, …)`, async) — drop the `&mut dyn` so the async trait stays object-safety-free.
   - Replace the two `LiveSink` `bail!` stubs (`recording.rs:76` trigger_egress, `:104` upload) with the LIVE async calls: Egress POST via `reqwest` + the `livekit_jwt`-minted token to `{livekit_url}/twirp/livekit.proto.Egress/StartRoomCompositeEgress` (`.error_for_status()?`); S3 PUT via `rust-s3::Bucket::put_object` (endpoint/bucket/creds from `config` — R-S3ENDPOINT, NEVER a hardcoded literal).
   - Convert the `Recorder` spy methods + `clean_posture_no_side_effects_when_disabled` + `maybe_record_enabled_records_sink_and_emits_intent` to `#[tokio::test]` + `.await`.
   - **PRESERVE the clean-posture negative invariant + the delete-the-gate check (R7):** `maybe_record(false, …)` still produces ZERO `trigger_egress`/`upload`/EmitIntent; deleting the `if !enabled { return }` gate must still make `clean_posture_no_side_effects_when_disabled` FAIL.
   - **Dead-code `#[allow]`:** `LiveSink`'s methods now do real work — if `LiveSink` was `#[allow(dead_code)]` and is now constructed/used in the non-test build, remove the `#[allow]`; if still only Phase-6-live-wired (not yet constructed outside tests), LEAVE it. Err toward leaving — the clippy `-D warnings` gate catches a still-unused item missing `#[allow]`, but a now-used item with a stale `#[allow]` is harmless.

2. **`services/bridge/src/appservice.rs`** — per §10.4, in `handle_recording_fetch`:
   - **cr-2:** reject an EMPTY requester pseudonym with `403` AFTER the BRIDGE_CALLBACK_SECRET Bearer gate (`:243-257` is the trust boundary — the bridge trusts the forwarded pseudonym ONLY because the caller proved the callback secret). The header read is at `:259-265`.
   - **cr-3:** replace `let participants: Vec<String> = Vec::new();` (`:269`) with the REAL participant set for the recording's room (pseudonyms only). `is_participant` STAYS called BEFORE serving the 200 path.
   - Resolve the REAL `media_url` (from `bridge_room`/the recording record) instead of the `format!("{id}.mp4")` placeholder (`:281-282`).

3. **`services/bridge/src/bridge_room.rs`** (ONLY IF NEEDED) — add `participants_for_recording(conn, recording_id) -> Result<Vec<String>>` (PSEUDONYMS only) IF no existing membership query fits. **CHECK `lookup` (`:47`) / `lookup_by_case` (`:59`) FIRST** — reuse if one fits; only add a helper if neither does.

**Do NOT:**
- Add `async-trait` (use native async-fn-in-trait + generic `S`). If native surfaces a `Send`/object-safety issue at validate, **surface a `kind: "blocker"` DQ first** — do NOT silently add the dep.
- Write `governance_log` / call `append_room_event` directly from `recording.rs` (R-CHAIN — `content_sha256` rides the chain via `record_uploaded` → EmitIntent → `drain_emits` → `post_room_event` → `append_room_event`).
- Add identity-shaped data — `speakers`/`actor`/participants/requester are PSEUDONYMS (ADR-015).
- Hardcode the S3 endpoint in source (R-S3ENDPOINT — `rg -n 'minio\.' services/bridge/src/` returns nothing).
- Touch any `crates/**`, migration, const, registry, the harness composes, or the smoke script (Task 1 owns those — it's GREEN).
- Add a new entry-kind or bump the registry (frozen at 72).

**Branch:** forks from `phase-m3-core-e2e-pilot` (current tip `cfc7e19e3`).

## §3 Required reading

- **`.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` §13 Task 2** (~lines 403-439) — authoritative ACTION/IMPLEMENT/GOTCHA/VALIDATE. Also §10.3 (~218-255), §10.4 (~257-275), §19 (2) (~747, no-async-trait rationale).
- **`services/bridge/src/recording.rs:17-22`** (current sync `RecordingSink` trait → async), `:31-49` (`maybe_record` → generic+async), `:54-106` (`LiveSink` + the two `bail!` at `:76`/`:104` + the Phase-6 POST/PUT comments at `:71-75`/`:102`), `:115-263` (the `Recorder` spy + the 2 deterministic tests → `#[tokio::test]`).
- **`services/bridge/src/appservice.rs:238-284`** (`handle_recording_fetch` — cr-2 at `:259-265`, cr-3 at `:267-269`, media_url at `:281-282`), `:243-257` (the BRIDGE_CALLBACK_SECRET Bearer gate = the cr-2 trust boundary).
- **`services/bridge/src/bridge_room.rs:47`** (`lookup`), `:59` (`lookup_by_case`) — check these BEFORE adding `participants_for_recording`.
- **`services/bridge/src/livekit_jwt.rs:31-46`** (`mint_access_token` — the Egress token mint).
- **Lessons (mandatory, §2.4 file-class injection):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — all `services/bridge` cargo via `scripts/brehon/cargo-linux.sh`; NEVER Windows-local (ruma-common E0119). YOU run NO cargo (see §4).
  - `feedback_linux_compile_proof_is_a_gate.md` — Task 2's validate-pending-laptop-linux DQ gates `bm-pr`.
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ and STOP.
  - `feedback_cheap_model_arm_drops_adr_constraints.md` — R7 (clean-posture) + R-CHAIN + ADR-015 (cr-2 pseudonym) are LOAD-BEARING, not merely named.

## §4 Constraints

- **R-CHAIN (`content_sha256` rides the chain — CATCH-FIRE on bypass):** `maybe_record` → `stage.record_uploaded(...)` pushes the EmitIntent; it does NOT write `governance_log` / call `append_room_event` directly. **Why:** the hash is the tamper-evidence link on the governance chain; bypassing `append_room_event` skips scrub_json + the ed25519 chain + signing (ADR-008/016 violation). **DoD:** `grep -nE "append_room_event|governance_log|INSERT" services/bridge/src/recording.rs` returns NOTHING (only the EmitIntent push; the binary drains it).
- **R9 / cr-2 (ADR-015 — LOAD-BEARING):** the requester pseudonym is accepted ONLY from the BRIDGE_CALLBACK_SECRET-Bearer-authed forward (the Bearer gate IS the trust boundary); reject an empty pseudonym with 403. **Why it can't be deferred:** a pseudonymous town hall's recording served to a non-participant (or an unauthenticated requester) leaks the event — the D5 Option-C access bar. **DoD:** `grep is_participant services/bridge/src/appservice.rs` returns the callsite AND it runs BEFORE the 200 path; the empty-pseudonym 403 is present.
- **ADR-015 (pseudonymity):** participants/requester/speakers/actor are PSEUDONYMS. **DoD:** `rg -i 'person_id|username|@.*:' services/bridge/src/recording.rs services/bridge/src/appservice.rs` returns nothing identity-shaped.
- **R7 (clean-posture negative invariant):** `clean_posture_no_side_effects_when_disabled` MUST fail if the `if !enabled { return }` gate is deleted; the positive `maybe_record_enabled_…` companion proves it asserts the GATE, not a trivial always-skip.
- **R-S3ENDPOINT:** S3 endpoint is config-sourced; no `minio.`-shaped literal in source.
- **No async-trait** (native async-fn-in-trait + generic `S`); DQ-surface before adding the dep if native fails.
- **NO daemon cargo.** Write a `validate-pending-laptop-linux` DQ (`bash scripts/brehon/dq-v3-new-entry.sh` for id, `dq-v3-append-fragment.sh <frag>.json --pending` to append):
  ```
  kind: "validate-pending-laptop-linux"
  branch: "phase-m3-core-e2e-pilot"
  phase_task: 2
  commands: [
    "scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml",
    "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings",
    "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml clean_posture_no_side_effects_when_disabled",
    "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml maybe_record_enabled_records_sink_and_emits_intent"
  ]
  result: null
  log_slice: null
  failed_commands: null
  ```
  Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo yourself. Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **ONE commit** — `feat(rtc): live LiveSink (async Egress + S3 PUT) + cr-2/cr-3 recording-fetch (task 2)`. End the body with a `LESSON:` trailer if anything durable (e.g. an async-fn-in-trait gotcha).
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.
- **Handover:** inline in task output (no handover file — sensitive-file guard).

## §3a Handover from prior tasks

- Task 1 (#748 + fix #749 rocksdb + fix #750 CONDUIT_PORT) — e2e harness GREEN: `docker-compose.e2e.yml` (MinIO + tuwunel-b distinct-domain + bridge-b), `e2e-harness-smoke.sh` (reach-the-containers, MinIO readiness poll), `registration-b.yaml`. Reach-smoke = E2E_HARNESS_REACH_OK (DQ `2c511098c5cc-001` PASS). Conduit on 8448 (rocksdb backend, CONDUIT_PORT=8448). Your live LiveSink S3 PUT + participant-fetch will be exercised against this harness at the Tasks 3-6 `-e2e` gate; YOUR DoD is the COMPILE gate (independent of the harness).
- Phase tip `cfc7e19e3`.
- Prior M3 phases shipped the seams you wire: `stage.rs::record_uploaded` (the EmitIntent push), `recording.rs::{compute_content_sha256, is_participant, RecordingSink, LiveSink, Recorder}`, `livekit_jwt::mint_access_token`, `rust-s3` dep (Task 3 of m3-core-recording).

## §5 HANDOVER (worker fills at task end — return inline)

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-task2
  filesModified: [services/bridge/src/recording.rs, services/bridge/src/appservice.rs]  # +bridge_room.rs IF a helper was needed
  keyDecisions:
    - "RecordingSink -> async (native async-fn-in-trait, no async-trait); maybe_record<S> generic+async"
    - "LiveSink trigger_egress = live reqwest Egress POST; upload = live rust-s3 put_object (endpoint from config)"
    - "cr-2: empty requester pseudonym -> 403 after Bearer gate; cr-3: real participant set (<reused lookup | added participants_for_recording>); real media_url"
    - "Recorder spy + 2 deterministic tests -> #[tokio::test]; clean-posture negative invariant + delete-the-gate check preserved"
    - "#[allow(dead_code)] on LiveSink: <removed (now used) | left (still Phase-6-live-only)>"
  validate_dq: <composite-id of the new validate-pending-laptop-linux DQ>
  notes: "<confirm no append_room_event/governance_log direct write; confirm no minio. literal in src; confirm pseudonym-only; confirm no async-trait added>"
```
