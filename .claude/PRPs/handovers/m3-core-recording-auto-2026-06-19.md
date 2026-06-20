---
phase: m3-core-recording
kind: auto-phase-handover-refresh
stage: impl-cohort-1-running
refreshed_at: 2026-06-19T23:42:00Z
---

# m3-core-recording — auto-phase handover (auto-refreshed)

**Stage:** `impl-cohort-1-running` · **Phase branch tip:** `cdd9d0869` (phase-m3-core-recording) · **Trunk:** governance-v0 @ `7215eb9e1`

## Next concrete action (re-verify on resume)

Poll Cohort A: Task 1 **#736** (crates/Windows, RoomEventPayload 5 recording fields) + Task 2 **#737** (bridge/Linux, S3 config + record_town_halls flag helpers). On BOTH `done`:
1. Read each worker's `validate-pending-laptop` (T1) / `validate-pending-laptop-linux` (T2) DQ entry.
2. **Run the cargo LOCALLY** (Shape-G RESIDUAL-ONLY — no ci-watcher, laptop IS the runner): T1 via `cmd //c "scripts\\brehon\\cargo-check.bat ..."` + clippy (Windows); T2 via `scripts/brehon/cargo-linux.sh check/clippy/test --manifest-path services/bridge/Cargo.toml` (Docker Linux).
3. Mutate each DQ: `answered_by: "advisor-laptop"`, `result: "pass"|"fail"`, `resolved_at`. Pass → resolved[]; fail → stays pending[] for §G4.
4. Cohort barrier: BOTH pass → advance. Next is **Task 3** (serial, `requires: task 2`) — the `rust-s3` dep-add, the HEADLINE `validate-pending-laptop-linux` gate (Linux Cargo.lock resolution risk; fallback `aws-sdk-s3`, surface DQ don't silently swap).

## State

- **Plan:** `.claude/PRPs/plans/m3-core-recording.plan.md` @ origin gov-v0 (7 tasks, complexity 3/10)
- **Auto-state:** `.claude/auto-state/m3-core-recording.json` (stage impl-cohort-1-running, resume_count 1)
- **DQ pending:** 0 (workers will raise validate-pending entries on push)
- **In-flight Junior:** #736 (T1 crates), #737 (T2 bridge) — Cohort A, base phase-m3-core-recording @ cdd9d0869
- **Lane mode:** B · **Mode:** fully-gated (no --unattended)

## Cohort map (plan §13)

- Cohort A (RUNNING): Task 1 [P] #736 (crates/Win) + Task 2 [P] #737 (bridge/Linux) — file-disjoint, different runners
- Task 3: serial, requires task 2 (rust-s3 dep + recording.rs primitives) — HEADLINE -linux gate
- Task 4: serial, requires tasks 1+3 (maybe_record flag-gate + Stage::record_uploaded emit + bridge DTO mirror + clean-posture invariant)
- Task 5: serial, requires task 3 (participant-floor fetch endpoint)
- Task 6: serial-docker, requires tasks 3,4,5 (#[ignore] integration test, compile-only)
- Task 7: retro

## Cross-session deps / tripwires

- **bm-cut finalize hazard RECOVERED this session** — daemon spuriously merged bm-cut worker into daemon-local gov-v0; origin gov-v0 stayed clean @ (now) 7215eb9e1; daemon recovered via update-ref + git restore (ssh-reset-hard hook correctly blocked the --hard attempt). Watch for the same on every bm-task finalize this phase.
- Task 3 = rust-s3 Linux Cargo.lock resolution (headline risk)
- content_sha256 MUST ride append_room_event (R11 catch-fire on bypass)
- record_town_halls=false clean-posture zero-side-effect (delete-the-gate check)
- Remaining gates: e2e local-vs-dispatch / CR-triage / merge-confirm / retro-sign-off (gate-1 plan-approval DONE)
