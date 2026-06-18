---
phase: m3-core-stage-mode
auto_emitted: true
stage: bm-cut-running
refreshed_at: 2026-06-18T21:28:00Z
---

# /auto-phase auto-handover — m3-core-stage-mode @ bm-cut-running

**Sub-phase:** m3-core-stage-mode (M3 town halls Phase 3, bridge-side).
**Stage:** `bm-cut-running` (bm-cut Junior #707 in flight).
**Plan:** `.claude/PRPs/plans/m3-core-stage-mode.plan.md` @ `8278f465b` — gate-1 APPROVED 21:10 UTC. Complexity 3/10, proceed-as-one. 6 impl tasks + Task 0 + retro.
**Last commit (governance-v0):** `c588c9882` docs(advisor): m3-core-stage-mode bm-cut brief.
**Phase branch:** `phase-m3-core-stage-mode` — being cut by #707 (does not exist yet at refresh time).

## Next concrete action (re-verify on resume)

On #707 done: (1) `git ls-remote origin phase-m3-core-stage-mode` — confirm branch exists. (2) MANDATORY daemon-trunk check (finalize-hazard recurrence-3): `ssh homeserver "cd /srv/brehon-fork && git log governance-v0 --oneline -1"` — must NOT contain a spurious `merge ... phase-m3-core-stage-mode`; recover via `git update-ref` + push if it did. (3) Route to `impl-cohort-1` = Cohort A (Task 1 `[P]` crates-Windows DTO + Task 2 `[P]` bridge-Linux livekit_jwt — file-disjoint across crates + validation runners). Planning re-run is SKIPPED (plan exists + approved).

## Cross-session deps

- DQ pending: 0. Resolved planner DQs ratified at gate-1: `da838b8fc109-001` (build bridge→binary callback client), `da838b8fc109-002` (extend RoomEventPayload 5 optional chair fields — the one in-scope crates/** touch). Advisor DoD note `a3d0e9941441-068` (§15.5 uses `rg`, absent → swap `git grep` at verify gate).
- Concurrent activity: none (single canonical worktree, 0 other running Junior tasks).
- Junior tasks: #706 (planning) done; #707 (bm-cut) running.

## Watchlist (gate every impl brief + the verify gate)

1. Chair dual-source: `chair_id` from `event.chair_pseudonym` ?? `juror_pseudonyms[0]` (Task 6).
2. FIFO persisted to `bridge_room.queue_state` (Task 3).
3. 30s grace = deterministic `tokio::time::advance` test (§16a Story 2, Task 4) — NOT happy-path.
4. FIRST emitter: `room_event_client.rs` POSTs to `/api/v4/governance/room-event` (Task 5); the `/api/v4` URL fix is load-bearing.
5. Linux-compile gate on EVERY bridge task (2–6): `validate-pending-laptop-linux` at result:pass before bm-pr.
6. ADR-015: chair payloads carry pseudonyms only (§15.5 grep, run with `git grep` not `rg`).

State: `.claude/auto-state/m3-core-stage-mode.json`. Resume via `/auto-phase m3-core-stage-mode`.
