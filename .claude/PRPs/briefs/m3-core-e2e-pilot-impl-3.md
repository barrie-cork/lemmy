# Brief: m3-core-e2e-pilot Task 3 [P] — stage_mode.rs LIVE (4-user mic-pass + 30s grace + chair override + transfer)

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-task3-stage-mode-live-e2e — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-3.md`

You are the **impl-task** subagent (Sonnet 4.6). Turn ONE bridge integration test (`stage_mode.rs`) from `#[ignore] todo!()` into a LIVE body against the e2e harness. Bridge-src tests, **Linux-validated**. Part of Cohort A (with Task 5 `recording.rs` — DISJOINT file).

## §2 Scope

**Plan task: `.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` §13 Task 3** (read it — full ACTION/IMPLEMENT/MIRROR/GOTCHA/VALIDATE) + §16a Story 1 (criteria 136-140).

**Produces (1 file modified, ONE commit):**

- **`services/bridge/tests/stage_mode.rs`** — turn `four_mic_pass_then_grace_boundary_emits_chair_entries` from `todo!()` into a LIVE integration test (keep `#[tokio::test] #[ignore = "requires docker-compose stack"]` — it needs the live stack). Per the scaffold step comments (`stage_mode.rs:14-22`):
  - Provision a town-hall stage room (chair + 4 watchers).
  - Drive 4 raise-hand + promote/activate passes via the real LiveKit grants.
  - Drive a 5th promote with no activation; assert auto-revoke + next-promote at the 30s grace boundary.
  - Drive a chair transfer + a chair override.
  - Query `governance_log` and assert `room_chair_transferred {from_pseudonym, to_pseudonym, at}` + `room_chair_override {action, target_pseudonym}` rows exist with PSEUDONYM payload fields (ADR-015).
  - Assert on OBSERVABLE state (governance_log rows + LiveKit grant transitions), NOT config/struct shape.

**Do NOT:**
- Touch any file other than `services/bridge/tests/stage_mode.rs` (Task 5 owns `recording.rs` — Cohort A is file-disjoint; DO NOT edit recording.rs/appservice.rs/stage.rs/etc).
- Touch the harness composes/smoke (Task 1 — GREEN), any `crates/**`, migration, const, registry, or plan.
- Add identity-shaped data — pseudonyms only (ADR-015).
- Add a dependency.

**Branch:** forks from `phase-m3-core-e2e-pilot` (current tip `5abf83328`).

## §3 Required reading

- **`.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` §13 Task 3** (~lines 441-471) + §16a Story 1 (~656-662).
- **`services/bridge/tests/stage_mode.rs`** (the scaffold — the `todo!("implement against live docker-compose stack (Phase-6 pilot grade)")` body is the unique Edit anchor; step comments at `:14-22`).
- **`services/bridge/src/stage.rs:105-338`** — the seams the test EXERCISES (do NOT rebuild): `promote_next` (mic-pass + 30s grace), `chair_override`, `transfer_chair`.
- **`services/bridge/tests/room_provisioning.rs`** — the `AsyncPgConnection::establish` + `governance_log` query idiom to MIRROR.
- **Lessons (mandatory):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo via cargo-linux.sh; you run NO cargo.
  - `feedback_build_what_tests_exercise.md` + `pattern_test_against_reality_not_syntax.md` — assert OBSERVABLE state.
  - `feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate the unique `todo!()` anchor.
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ and STOP.

## §4 Constraints

- **R-ANCHOR (pre-locate the Edit anchor):** run `grep -c 'todo!("implement against live docker-compose stack (Phase-6 pilot grade)")' services/bridge/tests/stage_mode.rs` — it MUST be `1` before you edit. The `todo!()` body is the unique anchor (one fn in this file).
- **ADR-015 (pseudonymity):** the asserted `governance_log` payload fields (`from_pseudonym`/`to_pseudonym`/`target_pseudonym`) are PSEUDONYMS. **DoD:** `rg -i 'person_id|username|@.*:' services/bridge/tests/stage_mode.rs` returns nothing identity-shaped.
- **Assert observable state (R: build-what-tests-exercise):** the test asserts the actual `governance_log` rows + LiveKit grant transitions, not config or struct shape.
- **Keep `#[ignore]`:** the test stays `#[tokio::test] #[ignore = "requires docker-compose stack"]` (needs the live stack — runs at the laptop -e2e gate, not in the default suite).
- **NO daemon cargo.** Write TWO DQs (use `bash scripts/brehon/dq-v3-new-entry.sh` for ids, `dq-v3-append-fragment.sh <frag>.json --pending` to append):
  1. `validate-pending-laptop-linux` (the worker-writable COMPILE gate):
     ```
     kind: "validate-pending-laptop-linux"
     branch: "phase-m3-core-e2e-pilot"
     phase_task: 3
     commands: [
       "scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml",
       "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings",
       "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test stage_mode --no-run"
     ]
     result: null
     log_slice: null
     failed_commands: null
     ```
  2. `validate-pending-laptop-e2e` (the LIVE run — gate-4=LOCAL, advisor runs after the stack is up):
     ```
     kind: "validate-pending-laptop-e2e"
     branch: "phase-m3-core-e2e-pilot"
     phase_task: 3
     commands: ["scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test stage_mode -- --ignored"]
     e2e_filter: null
     result: null
     log_slice: null
     failed_commands: null
     ```
  Commit + push BOTH DQs on the worker branch, then **STOP** — do NOT run cargo/Docker yourself.
- **ONE commit** — `feat(rtc): stage_mode.rs live e2e — 4-user mic-pass + grace + chair override/transfer (task 3)`. End the body with a `LESSON:` trailer if anything durable.
- **No stray temp artifacts:** if you create a DQ fragment file under `.claude/PRPs/debug/`, DELETE it after appending (do NOT commit it — Task 2 left a stray `dq-frag-task2.json`; don't repeat).
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.
- **Handover:** inline in task output (no handover file — sensitive-file guard).

## §3a Handover from prior tasks

- Task 1 (#748+#749+#750) — e2e harness GREEN (reach-smoke E2E_HARNESS_REACH_OK; Conduit rocksdb+port 8448; MinIO+2nd-instance distinct-domain). Your `-e2e` live run executes against this.
- Task 2 (#751, `5abf83328`) — live LiveSink async + cr-2/cr-3. Phase tip `5abf83328`.
- Cohort A: you (Task 3 stage_mode.rs) run alongside Task 5 (recording.rs) — DISJOINT files, no coordination needed.

## §5 HANDOVER (worker fills at task end — return inline)

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-task3
  filesModified: [services/bridge/tests/stage_mode.rs]
  keyDecisions:
    - "four_mic_pass_then_grace_boundary_emits_chair_entries live body (4 mic-passes + 30s grace + chair override + transfer)"
    - "asserts governance_log room_chair_transferred + room_chair_override rows with pseudonym fields (ADR-015); LiveKit grant transitions"
    - "kept #[ignore] (needs live stack)"
  validate_dq_linux: <id of the validate-pending-laptop-linux DQ>
  validate_dq_e2e: <id of the validate-pending-laptop-e2e DQ>
  notes: "<confirm todo!() anchor count was 1; confirm pseudonym-only; confirm no recording.rs/other-file edit; confirm no stray dq-frag committed>"
```
