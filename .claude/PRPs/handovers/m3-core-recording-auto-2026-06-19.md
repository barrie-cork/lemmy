---
phase: m3-core-recording
kind: auto-phase-handover-refresh
stage: impl-task-3-running
refreshed_at: 2026-06-20T01:05:00Z
---

# m3-core-recording — auto-phase handover (auto-refreshed)

**Stage:** `impl-task-3-running` · **Phase branch tip:** `ae3afb19c` · **Trunk:** governance-v0 @ `383536e70` · **Lane mode:** B (mobile remote-control; no laptop phase worktree)

## Next concrete action (re-verify on resume)

Poll **Junior #739** (Task 3 impl, base phase `ae3afb19c`). On `done`:
1. `git fetch origin phase-m3-core-recording`; find the new tip; read the worker's new `validate-pending-laptop-linux` DQ (the worker writes it + STOPs; it does NOT run cargo).
2. **Check for a worker blocker DQ first:** if the worker raised a `kind: "blocker"` DQ naming `rust-s3` Cargo.lock resolution failure → **advisor ratifies the `aws-sdk-s3` (endpoint_url) fallback** per plan §19(2), re-brief + re-dispatch. Do NOT silently swap; this is the headline risk.
3. **Run the cargo LOCALLY** in a throwaway worktree on the new phase tip (`git worktree add ../brehon-fork-validate-739 origin/phase-m3-core-recording`; bootstrap submodules + `.env`):
   - `scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml`
   - `scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings`
   - `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml`
   - This is the HEADLINE gate — the rust-s3 cold Cargo.lock re-resolve (~10-20 min cold). Trust the CHECK/CLIPPY/TEST_EXIT echoes in the log body, NOT the bg-task notification.
4. On GREEN: mutate the validate DQ → `resolved/pass` (`answered_by: "advisor-laptop"`) via the **daemon** (Mode B; canonical checkout must NOT mutate phase DQ). Clean up the throwaway worktree.
5. **Advance to Task 4** (serial, requires Tasks 1+3): `maybe_record` flag-gate (§10.4) + `Stage::record_uploaded` emit (§10.5) + bridge `RoomEventPayload` 5-field DTO mirror (§10.6) + clean-posture negative invariant test. CONSUMES `record_town_halls_enabled` → naturally clears the Task-2 dead-code `#[allow]`.

## State

- **Plan:** `.claude/PRPs/plans/m3-core-recording.plan.md` @ `99ab500ef` (ancestor of gov-v0 + phase branch; 7 tasks, complexity 3/10)
- **Auto-state:** `.claude/auto-state/m3-core-recording.json` (stage impl-task-3-running, resume_count 6)
- **Cohort A:** COMPLETE — Task 1 #736 (crates GREEN) + Task 2 #737 (bridge), 2 fix-impls (551bea68a dead-code, d4bfd5968 E0063 fixture). All 3 validate DQ resolved/pass (dc22f2a03). pending 0.
- **In-flight Junior:** #739 (Task 3 impl, base phase ae3afb19c)
- **Daemon main checkout:** ON `phase-m3-core-recording` (clean); direct commits there are lane-safe while 0 workers run. After #739 starts a worker, daemon has 1 worker worktree — do NOT direct-commit until it finishes.

## Cohort/task map (plan §13)

- Cohort A: Task 1 + Task 2 — DONE.
- **Task 3 (#739, IN FLIGHT):** recording.rs (compute_content_sha256 + RecordingSink trait + #[allow(dead_code)] LiveSink + Recorder spy + 2 tests) + Cargo.toml (rust-s3 dep + sha2 promote + Cargo.lock same commit) + main.rs (mod recording;). HEADLINE -linux gate.
- Task 4: serial, requires 1+3 (maybe_record + Stage::record_uploaded + bridge DTO mirror + clean-posture invariant; CONSUMES record_town_halls_enabled)
- Task 5: serial, requires 3 (participant-floor fetch; is_participant; CONSUMES read_recording_config)
- Task 6: serial-docker, requires 3,4,5 (#[ignore] integration tests/recording.rs, compile-only)
- Task 7: retro

## Cross-session deps / tripwires

- **R11 (CATCH-FIRE):** content_sha256 MUST ride append_room_event; recording.rs (Task 3) computes the hash but does NOT write the chain (Task 4 wires the emit). DoD: grep recording.rs for append_room_event/governance_log returns nothing in Task 3.
- **R12:** no hardcoded S3 endpoint; LiveSink reads config s3_* fields.
- **R14:** Cargo.lock rides the dep-add commit.
- **Headline risk:** rust-s3 Linux Cargo.lock resolution. Worker raises a blocker DQ on failure; advisor ratifies aws-sdk-s3 fallback (plan §19(2)). Do NOT silently swap.
- **Masked-exit trap:** bg cargo notifications report process exit, not cargo result — always read the CHECK/CLIPPY/TEST_EXIT echoes in the log body (cargo-output-capture rule). Bit us on the T2 clippy + the fix-impl-1 test gate.
- **Retro carry-forward (consolidated):** Task 2 brief missed TWO struct-field propagation obligations — (1) #[allow(dead_code)] on forward-declared helpers, (2) update ALL in-tree literal constructors incl. test fixtures. Both cost a fix-impl cycle (dead_code + E0063). Brief template should enumerate literal constructors + the dead-code-until-consumer rule when a task adds struct fields. Add to phase retro §What-to-change.
- **Remaining gates:** e2e local-vs-dispatch / CR-triage / merge-confirm / retro-sign-off (gate-1 plan-approval DONE).
