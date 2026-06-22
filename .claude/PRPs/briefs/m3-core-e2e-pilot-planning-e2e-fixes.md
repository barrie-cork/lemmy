# Brief — m3-core-e2e-pilot planning: 3 e2e defects exposed by the cycle-4 live gate

**Role + dispatch:** `[role:planning] e2e-fixes — see .claude/PRPs/briefs/m3-core-e2e-pilot-planning-e2e-fixes.md`

Base branch: `governance-v0` (planning briefs commit on trunk; plan finalize-merges to trunk).

## Scope

Author an implementation plan at
`.claude/PRPs/plans/m3-core-e2e-pilot-e2e-fixes.plan.md` that makes the 3 remaining
real e2e failures pass. These were exposed by the **first-ever live in-stack run** of
the Tuwunel-based harness (advisor e2e gate 2026-06-21, phase tip `4df0ed991`). The
cycle-4 fixes (Tuwunel swap, MinIO bucket, LiveKit `--keys`) are VALIDATED and working —
do NOT revisit them. The full gate results are in the handover
`.claude/PRPs/handovers/m3-core-e2e-pilot-e2e-run-2026-06-21c.md` — **read it first.**

Do NOT author implementation code. Produce only the plan file (template:
`.claude/commands/prp-plan.md` / `.claude/PRPs/templates/plan.template.md`).

## The 3 defects (GROUND TRUTH from the live gate — not hypothesis)

### Defect 1 — recording on-chain `:3000` connection refused (env/infra)
- `recording_lands_with_hash_on_chain` FAILS: `POST room-event to Brehon http://localhost:3000/api/v4/governance/room-event` — connection refused. Nothing on host `:3000`.
- `services/bridge/tests/recording.rs:39-40` reads `BREHON_ROOM_EVENT_URL` (default `:3000`); `:115` `?`-propagates the POST error → hard fail.
- The pilot Lemmy runs as `brehon-lemmy-1` on port **1236** (different stack). The bridge default is `:3000`.
- recording's OTHER 2 tests PASS (`clean_posture_no_recording_when_disabled`, `participant_floor_fetch`).

### Defect 2 — room_provisioning R7: chair seated when rtc_enabled=false (code/contract design)
- `rtc_disabled_townhall_clean_posture` (case 77007) now passes `matrix_room_id.is_some()` (Tuwunel swap worked — createRoom 2xx) but FAILS `chair_id.is_none()`: `R7 violated — chair_id=Some(...) when rtc_enabled=false`.
- ROOT CAUSE: `services/bridge/src/room_provisioner.rs:252` gates the chair-seat block on **`(livekit_api_key, livekit_api_secret)` both `Some`** — NOT on any per-event rtc flag. In the e2e stack the creds ARE configured (cycle-4 `--keys`), so the chair seats. `CaseTransitionEvent` (`room_provisioner.rs:20-37`) has **no `rtc_enabled` field**. The test (`room_provisioning.rs:120-138`) sends `{type_: case_transition, case_id, new_status: town_hall, chair_pseudonym}` — no way to express "rtc disabled" — and asserts no chair. The test's intent (rtc-disabled ⇒ no RTC stage seat) is not expressible in the current wire contract.
- This is a genuine **wire-contract design decision**, not a one-line fix.

### Defect 3 — emergency_mute 6007ms > 500ms perf gate (code/timing)
- `mute_all_drops_all_publishers_cross_instance_under_500ms` FAILS: `6007ms > 500ms` (criterion 141).
- The cycle-4 #764 fix correctly classifies psrpc `unavailable` as zero-holder-by-absence (a VALUE), but reaching that error costs a ~6s psrpc timeout per never-connected publisher, **inside the timed T0→T1 window** (`emergency_mute.rs:163` t0 → `:228` elapsed → `:232` assert).
- LiveKit v1.7 routes `UpdateParticipant` via psrpc to the node owning the participant's media session; a never-connected publisher (no Element Call client in the base stack) has no handler → timeout. The 6s is purely the absent-handler timeout, NOT real revocation latency.
- The PRD (DQ `3004b6625b83-001`) defines the <500ms as the in-instance AUTOMATED proof — keep it meaningful.

## Required recon — RUN ON THE DAEMON (`ssh homeserver`) before writing the plan

Establish feasibility per defect; do NOT pick blindly. The plan's §4 watchpoints + §5 complexity must cite the probe evidence.

### Defect 1 — pick from 3, probe first
- **Option A — point `BREHON_ROOM_EVENT_URL` at the running pilot.** PROBE: does `brehon-lemmy-1` (port 1236) expose `POST /api/v4/governance/room-event` accepting the bridge's callback secret? `curl` it with a sample room-event payload from the daemon; record the status. If 2xx, the fix is an env override in the test/compose (cheapest). Risk: the pilot is a live stack — e2e writes would pollute pilot governance state. Weigh isolation.
- **Option B — stand up a dedicated `lemmy_server` with governance routes on :3000 for e2e.** Largest. Recon the cost: does an existing compose (`docker-compose.pilot.yml`?) bring up a governance-capable Lemmy quickly, or is this a from-scratch service? Only choose if A and C are inadequate.
- **Option C — harness stub: a tiny container on :3000 that 200s + records the append.** The recording test's real assertion is "the bridge POSTs a well-formed room-event with the recording hash", not "Lemmy persisted it" — confirm by reading `recording.rs:90-130` (what does it assert AFTER the POST?). If the test only needs a 2xx + maybe echoes the body, a 10-line mock (e.g. a `mockserver`/`caddy respond`/tiny python sidecar) is the lowest-risk e2e-honest fix. **Likely the right answer** — but verify against what the test actually checks.

### Defect 2 — pick from 3, this is the design call
- **Option A (recommended pending probe) — add `rtc_enabled: Option<bool>` to the wire contract.** Add the field to `CaseTransitionEvent` AND its mirror `BridgeNotifyPayload` in the governance crate (find it: `grep -rn BridgeNotifyPayload crates/`). Gate the chair block (`room_provisioner.rs:252`) on `event.rtc_enabled != Some(false) && creds_present` (default-on preserves existing behavior for events without the field; explicit `false` disables). Test sends `rtc_enabled: false`. **ADR check:** this touches the bridge↔governance wire contract — verify against ADR-016 (cross-app backplane) + apply `feedback_entry_kind_runtime_allowlist_check.md` discipline (a new field that compiles but is ignored at the other end is a silent contract break — confirm BOTH ends consume it). This is a governance-logic change → see the Constraints note: it MAY need to cross the bridge/governance boundary, which is in scope here ONLY for the contract field, not for reworking provisioning logic.
- **Option B — re-scope the test to disable RTC the way the code already gates** (run case 77007 against a config with LiveKit creds absent). Harder in a shared stack; the creds are stack-wide. Weigh whether a per-test config override is feasible.
- **Option C — move the negative invariant to a unit test** (`#[test]` in `room_provisioner.rs` with creds-absent config), and `#[ignore]` the e2e assertion. Lowest effort; cost = the negative invariant is no longer an e2e acceptance gate.
- The plan must state which option and WHY, and whether it crosses into `crates/` (the BridgeNotifyPayload mirror). If A, enumerate BOTH edit sites.

### Defect 3 — pick from 3
- **Option A (recommended) — wrap each `update_participant` in a short `tokio::time::timeout`** (e.g. 200ms) and treat the timeout as zero-holder-by-absence (same as the existing `unavailable` arm). Fail fast → stay under 500ms → the automated perf gate stays meaningful (a fast-failing call still proves the revocation PATH is <500ms; the 6s was pure absent-handler timeout). This is a TEST-file change (`emergency_mute.rs`), not bridge src.
- **Option B — pre-check publisher connectivity** and skip the live call for absent publishers. More code; same outcome.
- **Option C — re-scope criterion 141's <500ms to the D2 pilot only** (real clients), assert only correctness (zero-holder) in the base stack. Cost = the automated perf proof moves to manual D2. Weigh against the PRD's intent that <500ms is the automated proof.
- PROBE: confirm `livekit-api`'s `update_participant` is `tokio`-awaitable so `tokio::time::timeout` wraps it cleanly (it is — it's `.await`ed at `:171`). Confirm 200ms is comfortably above a real revocation round-trip but below 500ms.

## Required reading

- `.claude/PRPs/handovers/m3-core-e2e-pilot-e2e-run-2026-06-21c.md` — the full gate results + per-defect detail + options analysis (THIS brief's source of truth).
- `services/bridge/tests/recording.rs` (lines 30-130 — the on-chain POST + what it asserts after).
- `services/bridge/src/room_provisioner.rs:20-37` (`CaseTransitionEvent`) + `:201-310` (`provision_townhall_stage_room`, the `:252` creds gate, the `:284-295` chair write).
- `services/bridge/tests/room_provisioning.rs:74` + `:120-180` (the rtc_disabled test setup + R7 assertion).
- `services/bridge/tests/emergency_mute.rs:12-20` (criterion 141 doc) + `:152-235` (the timed revoke loop + the existing `unavailable` accept arm + the <500ms assert).
- For defect 2 option A: `grep -rn 'BridgeNotifyPayload' crates/` — find the governance-side enum the `CaseTransitionEvent` mirrors; `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` ADR-016.
- `.claude/lessons/feedback_entry_kind_runtime_allowlist_check.md` (wire-contract: a field that compiles but isn't consumed at the far end is a silent break).
- `.claude/lessons/feedback_build_what_tests_exercise.md` (validate by observable behavior — defect 1 option C rests on what the test actually asserts).
- `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` (any `services/bridge` cargo runs on Linux via `scripts/brehon/cargo-linux.sh`; gate-4 e2e is LOCAL on the daemon).

## Constraints

- **Gate-4 = LOCAL** (decided 2026-06-20): e2e validation runs on the laptop/daemon stack, not GH Actions. Each impl task's DoD is a `validate-pending-laptop-e2e` (or `-linux` for the compile proof) DQ that the advisor runs. Write-then-STOP — workers do NOT run cargo on the daemon for the main crates (NO-CARGO-ON-ELITEDESK); `services/bridge` cargo is the carve-out (separate Linux workspace) but follow `feedback_validate_pending_laptop_write_then_stop.md` for the e2e RUN itself (advisor is the runner).
- **Scope boundary:** defects 1 and 3 are harness/test-file only (no governance-logic Rust). Defect 2 option A DOES cross into `crates/` (the `BridgeNotifyPayload` mirror) + `services/bridge/src/` (the gate + struct field) — this is IN SCOPE for this plan because it's a real contract bug the e2e caught, but the plan must (a) confirm via ADR-016 it's not re-litigating a hard constraint, (b) enumerate every edit site for the wire-contract field, (c) keep the change minimal (one field + one gate condition + test setup), NOT a provisioning-logic rework.
- **Cycle-3 catch-fire is in effect for the OLD reactive auto-fix loop** — this plan is the deliberate replanned path the user authorized. NO reactive compose/code guesses; pick each option on probe evidence.
- **Three defects = up to 3 tasks** (one per defect), possibly fewer if a defect is a single re-scope. Mark `[P]` where tasks are file-disjoint (defect 1 = recording.rs/harness; defect 3 = emergency_mute.rs; defect 2 = room_provisioner.rs + crates/ + room_provisioning.rs — these 3 sets are disjoint, so all 3 are cohort-parallel candidates). The plan's §13 must mark cohort eligibility.
- If the brief is ambiguous, raise a `kind: "log"` note; do NOT block. Clarify-DQ is the advisor's `/brehon-clarify` gate, run before this planning task is queued.
- §16a stories (one per defect, with checkpoint commands) are MANDATORY — `/brehon-verify` runs them post-impl.
