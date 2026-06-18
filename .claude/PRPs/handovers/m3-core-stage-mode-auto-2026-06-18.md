---
phase: m3-core-stage-mode
auto_emitted: true
stage: impl-cohort-1-running
refreshed_at: 2026-06-18T22:38:00Z
---

# /auto-phase auto-handover — m3-core-stage-mode @ impl-cohort-1-running

**Sub-phase:** m3-core-stage-mode (M3 town halls Phase 3, bridge-side).
**Stage:** `impl-cohort-1-running` — Cohort A in flight (#708 Task 1 + #709 Task 2, file-disjoint `[P]`).
**Plan:** `.claude/PRPs/plans/m3-core-stage-mode.plan.md` @ `8278f465b` — gate-1 APPROVED. Complexity 3/10. 6 impl tasks + Task 0 + retro.
**Phase branch:** `phase-m3-core-stage-mode` @ `d722cbc1d` (after Mode-B trunk→phase brief sync). Workers #708/#709 fork from here.
**Last commit (governance-v0):** `3c92ff793` docs(advisor): impl briefs Cohort A.

## Cohort A (current)

- **#708 Task 1** — `crates/api` DTO: 5 optional `RoomEventPayload` chair fields + `CaseTransitionEvent.chair_pseudonym`. Windows validation → `validate-pending-laptop` DQ.
- **#709 Task 2** — `services/bridge/src/livekit_jwt.rs`: `can_publish` presenter/watcher grant. Linux validation → `validate-pending-laptop-linux` DQ (Docker).

## Next concrete action (re-verify on resume)

On BOTH #708 + #709 done: (1) read each worker branch for its validate-pending-laptop DQ. **Per `feedback_fix_impl_workers_skip_validate_pending_dq`: VERIFY the DQ was written; if missing, verify-via-DoD-grep on the phase tip + run validation directly anyway** (laptop is the runner — no ci-watcher for the `-laptop`/`-laptop-linux` kinds; advisor runs `cargo-check.bat`/`cargo-linux.sh` locally + mutates the DQ `answered_by: advisor-laptop`). (2) On both result:pass + daemon finalize-merge → advance. NOTE: stage-mode's marquee DoD is **deterministic unit tests**, NOT e2e — so phase-2-e2e may be a no-op; advance to **impl-cohort-2 = Task 3** (serial bridge: `stage.rs` FIFO + mic-pass state machine). Tasks 3→4→5→6 run serial (shared bridge crate + Cargo.lock + cold Linux builds).

## ⚠ Finalize-hazard recurrence-3 (RECOVERED this session)

bm-cut #707's daemon finalize spuriously merged `phase-m3-core-stage-mode` → daemon-local `governance-v0` (`b5ba35711`, would have deleted the auto-handover). RECOVERED: `git update-ref refs/heads/governance-v0 origin/governance-v0` reset daemon-local to clean origin `c7baa1dea`; origin gov-v0 was NEVER contaminated. **The MANDATORY post-bm-task daemon-trunk check caught it.** Apply the same check after bm-pr and bm-merge this phase.

## Cross-session deps

- DQ pending: 0 at dispatch (workers will raise validate-pending DQs mid-task).
- Concurrent activity: none (single canonical worktree).
- Junior: #706 (planning) done, #707 (bm-cut) done, #708+#709 (Cohort A) running.

## Watchlist (gate every impl brief + verify gate)

1. Chair dual-source `chair_id` ?? `juror_pseudonyms[0]` (Task 6). 2. FIFO persisted to `bridge_room.queue_state` (Task 3). 3. 30s grace = `tokio::time::advance` test (Story 2, Task 4) NOT happy-path. 4. FIRST emitter: `room_event_client.rs` POSTs `/api/v4/governance/room-event` (Task 5); `/api/v4` URL fix load-bearing. 5. Linux-compile gate every bridge task. 6. ADR-015 pseudonyms-only (§15.5 grep — use `git grep`, `rg` absent per DQ a3d0e9941441-068).

State: `.claude/auto-state/m3-core-stage-mode.json`. Resume via `/auto-phase m3-core-stage-mode`.
