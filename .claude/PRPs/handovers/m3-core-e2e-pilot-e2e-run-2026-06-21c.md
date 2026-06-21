# Handover — m3-core-e2e-pilot e2e gate (2026-06-21c, cycle-4 validated → 3 new defects)

**Phase:** m3-core-e2e-pilot (M3-core Phase 6/6 — FINAL). Mode B (canonical `brehon-fork` on `governance-v0`; phase work via daemon).
**Stage:** cycle-4 fixes VALIDATED in-stack (Tuwunel swap + MinIO bucket work). The first-ever live `bridge→Tuwunel→createRoom→bridge_room` run exposed **3 distinct remaining defects** — now being PLANNED, not reactively fixed. Phase stays blocked at e2e.
**Author:** advisor (governance-v0).

> Supersedes `m3-core-e2e-pilot-e2e-run-2026-06-21b.md`. That handover's two open
> questions are partly resolved: (Q2 LiveKit `unavailable`) — the #764 value-fix
> landed but is INSUFFICIENT (see defect 3); (Q1 :3000) — still open (defect 1).
> Read THIS file first on resume.

## RESUME BLOCK (read first)

- **Phase branch tip (origin):** `d69d41c1a chore(decision-queue): record e2e gate #b9m — 4 entries fail (advisor-laptop)` on `origin/phase-m3-core-e2e-pilot`.
- **Last fix tip before gate:** `4df0ed991` (as-register T2 overlay merged). All cycle-4 fixes are on this tip.
- **Aux services: BOTH RESTORED.** e2e stack DOWN (0 containers), `docker-portainer-1` running, `web-archive-frontend` running on :8080. **No live cleanup obligations.**
- **Next concrete action:** a **planning task** for the 3 defects below is being queued (`m3-core-e2e-pilot-e2e-fixes` plan). Do NOT queue another reactive fix-impl. The RTC-gate defect is a wire-contract design decision (needs the planner).

## What this gate validated (cycle-4 fixes — CONFIRMED WORKING)

The bundled e2e gate ran on phase tip `4df0ed991` — the first live in-stack run of the Tuwunel-based harness. **The cycle-4 fixes work:**

- ✅ **Tuwunel swap (as-register T1+T2):** both `tuwunel` (:8448) and `tuwunel-b` (:8449) boot on RocksDB, auto-load `registration.yaml`, and **createRoom no longer 401s** — the AS bearer token is accepted. The `rtc_disabled_townhall_clean_posture` test now passes `matrix_room_id.is_some()` (it didn't before). The as-register diagnosis (Conduit v0.6.0 never registers the AS) was correct.
- ✅ **MinIO bucket (cycle-4 fix-impl #3):** `minio-init` (mc) sidecar creates `local/recordings` on startup — verified in logs. The recording test's `put_object` no longer hits `NoSuchBucket`.
- ✅ **LiveKit `--keys` (cycle-4 fix-impl #3):** no more 401 auth failures; the bridge's devsecret-signed JWTs verify.

## The 3 remaining defects (the cycle-5 layer — DO NOT auto-fix; PLANNING)

| # | Target | Failure | Root cause | Class |
|---|---|---|---|---|
| 1 | recording | `recording_lands_with_hash_on_chain` FAIL: `POST room-event :3000 connection refused` | **Brehon governance :3000 not running in e2e.** The deferred open question, NOT a regression. recording test default `BREHON_ROOM_EVENT_URL=http://localhost:3000/api/v4/governance/room-event`; nothing on host :3000. | env/infra |
| 2 | room_provisioning | `rtc_disabled_townhall_clean_posture` FAIL: `R7 violated — chair_id=Some(...) for case 77007 when rtc_enabled=false` | **Test/contract mismatch.** `CaseTransitionEvent` has NO `rtc_enabled` field. The provisioner gates stage-mode (chair seat) on **LiveKit creds being configured** (`room_provisioner.rs:252`), not on any per-event rtc flag. In the e2e stack creds ARE present (cycle-4 `--keys`), so the chair seats. The test sends a town_hall event with no way to express "rtc disabled" and expects `chair_id IS NULL`. | code/contract design |
| 3 | emergency_mute | `mute_all_...under_500ms` FAIL: `6007ms > 500ms` | **#764 fix insufficient.** It accepts psrpc `unavailable` as a zero-holder-by-absence VALUE (correct), but reaching that error costs a ~6s psrpc timeout per never-connected publisher, INSIDE the timed T0→T1 window (`emergency_mute.rs:163-228`). Accepting the error ≠ failing fast → blows criterion 141. | code/timing |

Passing now: `recording` 2/3 (`clean_posture_no_recording_when_disabled`, `participant_floor_fetch`); `room_provisioning` 1/8 (`anonymous_townhall_identity_never_reaches_livekit`); the 5 `todo!()` room_provisioning stubs are D2-deferred (expected-fail).

## Defect detail (for the planner)

### Defect 1 — recording on-chain :3000
- `services/bridge/tests/recording.rs:39-40` reads `BREHON_ROOM_EVENT_URL` (default `:3000`); `:115` `?`-propagates the POST error → hard fail.
- Host has nothing on :3000. The pilot Lemmy runs as `brehon-lemmy-1` (port 1236, different stack).
- **Options:** (a) point `BREHON_ROOM_EVENT_URL` at the running pilot governance API (env override — cheapest, but pilot may not expose append_room_event at the matching path); (b) stand up a dedicated `lemmy_server` with governance routes on :3000 for e2e (largest); (c) re-scope: stub/mock the room-event endpoint in the harness (a tiny container that 200s + records the append), since the test's real assertion is "bridge POSTs a well-formed room-event", not "Lemmy persisted it".

### Defect 2 — RTC-gate / rtc_enabled contract
- `room_provisioner.rs:201` `provision_townhall_stage_room`: provisions Matrix room unconditionally (correct — governance flow is RTC-independent), then at `:252` gates the chair-seat block on `(livekit_api_key, livekit_api_secret)` both `Some`. No `event.rtc_enabled` check exists; the field doesn't exist on `CaseTransitionEvent` (`:20-37`).
- Test `room_provisioning.rs:120-175` sends `{type_: case_transition, case_id, new_status: town_hall, chair_pseudonym}` — no rtc flag — and asserts `chair_id.is_none()`. Test comment claims gating is "on LIVEKIT config" but expects no-chair when creds ARE configured. Contradiction.
- **Options:** (a) add `rtc_enabled: Option<bool>` to `CaseTransitionEvent` (wire contract — mirror in governance.rs `BridgeNotifyPayload`), gate the chair block on `event.rtc_enabled == Some(true) && creds_present`, test sends `rtc_enabled: false`; (b) re-scope the test to disable RTC the way the code gates — run this one case against a stack/config with LiveKit creds absent (harder in a shared stack); (c) decide the negative invariant belongs in a unit test, not e2e. **(a) is the honest fix** — `rtc_enabled` is a real governance concept (criterion 146 references it) and the wire contract should carry it. ADR-adjacent: touches the BridgeNotifyPayload mirror → check ADR-016 + `feedback_entry_kind_runtime_allowlist_check.md` style contract-mirror discipline.

### Defect 3 — emergency_mute 6s timeout
- `emergency_mute.rs:163` `t0 = Instant::now()`; loop calls `lk_client.update_participant(...)` per publisher; each never-connected publisher → ~6s psrpc timeout (LiveKit routes UpdateParticipant via psrpc to the node owning the media session; no client → no handler → timeout). Then `:228` `elapsed = t0.elapsed()`; `:232` asserts `< 500ms`.
- The #764 fix made the *error classification* correct but did nothing about the *6s to get there*.
- **Options:** (a) wrap each `update_participant` in a short `tokio::time::timeout` (e.g. 200ms) and treat timeout-elapsed as zero-holder-by-absence — fail fast, stay under budget; (b) for absent publishers (the base-stack reality), skip the live call entirely — the zero-holder invariant is satisfied by absence, so the test could pre-check connectivity OR the perf budget should exclude absent-publisher timeouts; (c) re-scope criterion 141's <500ms to the D2 pilot (real clients) only, and in the base stack assert only correctness (zero-holder) not timing. **The PRD (DQ 3004b6625b83-001) says the <500ms is the in-instance automated proof** — so (a) is the fix that keeps the automated perf gate meaningful: a fast-failing call still proves the revocation path is <500ms; the 6s is purely the absent-handler timeout, not real revocation latency.

## 4 `-e2e` DQ entries (all `result: fail`, stay pending on phase branch for triage)

`13951b0fc56a-001` emergency_mute · `c2e678abdbb2-002` recording · `be3e6842603f-001` as-reg T1 createRoom (HTTP-200 MET, downstream R7 fail) · `f433ffd8a4a2-001` as-reg T2 createRoom :8448+:8449 (HTTP-200 MET, downstream R7 fail). Mutated `answered_by: advisor-laptop` at `d69d41c1a`. The two createRoom DQs are marked fail because the test they gate fails on defect 2, but their narrow HTTP-200 question is satisfied (noted in each `log_slice`).

Also still pending (pre-cycle-4, superseded): `44243654b24d-002`, `1c37b605fd1a-002`, `1c37b605fd1a-001`, `6d1cb2511b90-001`, `ec2aa316b69b-001`, `53441afbc986-001`. Archive/resolve these at retro — they're earlier-cycle entries whose newer siblings carry the live results.

## Known stack quirk (not a defect)

`bridge-b` cannot bind host :8081 (caddy holds `127.0.0.1:8081`). bridge-b is the D2-deferred cross-instance container — does NOT block in-instance tests. Durable fix (deferred): remap bridge-b host port off 8081, per `issue_note_e2e_host_port_conflicts_daemon.md`.

## Re-run procedure (after the 3 fixes land)

On daemon (`ssh homeserver`), native cargo (PATH + chown durable):
1. Free :8080: `docker stop web-archive-frontend`. Up stack: `cd /srv/brehon-fork/services/bridge && docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc up -d`. (minio-init creates bucket; bridge-b :8081 bind fails — expected.)
2. Stand up / wire :3000 governance per defect 1's chosen option.
3. `export PATH=$HOME/.cargo/bin:$PATH && export BRIDGE_DB_PATH=/srv/brehon-fork/services/bridge/.e2e-data/bridge-a/bridge-a.db`
4. Per target: `cargo test --manifest-path services/bridge/Cargo.toml --test <T> -- --ignored > /tmp/m3-e2e-<n>-<T>.log 2>&1; echo EXIT=$?` (NO pipe; read the LOG, not the exit code — cargo returns 101 for lock-fail too). Targets: `recording`, `room_provisioning`, `emergency_mute`.
5. Teardown stack + `docker start docker-portainer-1 web-archive-frontend`. End-state: stack DOWN, portainer + web-archive up.

## After all real tests pass

Task 7 (D2 pilot runbook, NON-impl) → retro → `/brehon-verify m3-core-e2e-pilot` → bm-pr → gate-3 CR triage → gate-5 merge-confirm → bm-merge → gate-6 retro sign-off → `/brehon-phase-transition`.

## Resume command

`Resume m3-core-e2e-pilot. cycle-4 fixes VALIDATED in-stack (Tuwunel swap + MinIO + LiveKit --keys all work). e2e gate exposed 3 NEW defects — being PLANNED, not reactively fixed. Read .claude/PRPs/handovers/m3-core-e2e-pilot-e2e-run-2026-06-21c.md FIRST. NO live cleanup. Next = planning task m3-core-e2e-pilot-e2e-fixes (1: :3000 for recording on-chain; 2: rtc_enabled wire-contract gate for room_provisioning R7; 3: fast-fail update_participant for emergency_mute 500ms). Phase tip d69d41c1a.`
