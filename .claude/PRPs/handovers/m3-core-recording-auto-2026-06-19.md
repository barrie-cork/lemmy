---
phase: m3-core-recording
kind: auto-phase-handover-refresh
stage: bm-cut-running
refreshed_at: 2026-06-19T23:34:00Z
---

# m3-core-recording — auto-phase handover (auto-refreshed)

**Stage:** `bm-cut-running` · **Last commit on governance-v0:** `2804e5a51` (chore(advisor): author m3-core-recording bm-cut brief)

## Next concrete action (re-verify on resume)

Poll bm-cut Junior **#735**. On `done`: verify `phase-m3-core-recording` exists on origin (`git ls-remote origin refs/heads/phase-m3-core-recording`), then route `bm-cut-done` **directly to impl-cohort-1** — planning is PRE-COMPLETE (Junior #734, plan `99ab500ef`, gate-1 approved 2026-06-19), so do NOT re-author a planning brief or re-run gate-1. Cohort A = Task 1 [P] (crates Windows, 5-field recording DTO) + Task 2 [P] (bridge Linux, S3 config + record_town_halls flag helpers) — file-disjoint across different runners.

## State

- **Plan:** `.claude/PRPs/plans/m3-core-recording.plan.md` @ `99ab500ef` (7 tasks, complexity 3/10, confidence 8/10)
- **Auto-state:** `.claude/auto-state/m3-core-recording.json`
- **DQ pending:** 0
- **In-flight Junior:** #735 (bm-cut, base governance-v0)
- **Lane mode:** B (no laptop worktree; drive via Junior dispatch)
- **Mode:** fully-gated (no `--unattended`)

## Cohort map (from plan §13)

- Cohort A: Task 1 [P] (crates/Windows) + Task 2 [P] (bridge/Linux) — genuine parallel, different runners
- Tasks 3–6: serial (shared bridge crate + cold Linux builds; Task 3 adds rust-s3 dep = cold Cargo.lock re-resolve)
- Task 7: retro

## Cross-session deps / tripwires

- Task 3 = headline `validate-pending-laptop-linux` gate (rust-s3 Linux Cargo.lock resolution; fallback aws-sdk-s3, surface DQ don't silently swap)
- `content_sha256` MUST ride `append_room_event` (R11 catch-fire on bypass)
- `record_town_halls=false` clean-posture zero-side-effect invariant (delete-the-gate mechanical check)
- bm-cut long-name-refspec finalize hazard: advisor verifies daemon-local trunk post-task
- 6 mandatory gates: gate-1 plan-approval DONE; remaining = ADR-DQ / CR-triage / e2e local-vs-dispatch / merge-confirm / retro-sign-off
