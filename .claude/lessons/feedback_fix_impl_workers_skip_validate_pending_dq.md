---
name: Fix-impl workers skip the validate-pending DQ-write step
description: Short mechanical fix-impl workers commit+push their fix but never raise the validate-pending-laptop[-e2e/-linux] DQ the brief asked for. The advisor must NOT wait for the DQ to surface — verify the fix landed via DoD grep on the phase tip and run the validation directly. Confirmed m3-core-infra cr-fix cohort (#703 bridge, #704 lemmy) 2026-06-18: neither fix commit touched decision-queue.json.
type: feedback
---

When a `fix-impl` brief instructs the worker to "write a `validate-pending-laptop[-e2e|-linux]`
DQ entry, commit + push, then stop," **short mechanical fix workers frequently skip the
DQ-write step entirely** — they make the code edit, commit, push, write their post-task retro,
and report done. The validation DQ the advisor's polling loop is waiting for never appears.

The symptom looks like "the finalize-merge dropped the DQ," but it is NOT. The worker never
wrote the DQ in the first place.

**Confirmed:** m3-core-infra CR-fix cohort, 2026-06-18.
- #703 (bridge cr-6/7/8) fix commit `2e4926322` touched only `services/bridge/src/{bridge_room,config,livekit_jwt}.rs` — no `decision-queue.json`.
- #704 (lemmy cr-4/5) fix commit `d421fb61c` touched only `governance.rs` + `down.sql` — no `decision-queue.json`.
- The most recent DQ commit on the phase tip predated BOTH fix commits. Neither worker raised
  its `validate-pending-laptop-linux` (#703) or `validate-pending-laptop-e2e` (#704) entry,
  despite both briefs explicitly requiring it in §4 with the exact JSON fragment.

**Why this happens (hypothesis):** the DQ-write step is "ceremony" appended after the real work.
A worker that treats `commit + push` as the task's completion — especially on a tiny, obviously-
correct edit — drops the trailing ceremony. The bigger the fix, the more likely the worker
follows the full brief; the smaller the fix, the more likely it shortcuts to done. (The original
impl-task workers on this same phase DID raise their validate DQs — it's the *fix*-impl variant,
with its smaller scope, that skips.)

## How to apply

- **Do NOT block on the validate-pending DQ surfacing for a fix-impl task.** After the worker
  reports `done` (and the daemon finalize-merges), the advisor's next action is NOT "wait for the
  ci-watcher / validate DQ." It is:
  1. **Verify the fix landed** — DoD grep on the phase tip (`git show <tip>:<file> | grep -c <anchor>`).
     This is the real gate, not the DQ.
  2. **Run the validation directly** — the advisor already knows exactly what each fix needs
     (the brief specified the cargo/e2e/Linux command + filter). Spin the throwaway validation
     worktree and run it. Do not dispatch a ci-watcher waiting for a DQ that won't come.
- **The brief instruction to "write the validate DQ" is still worth keeping** — it documents
  intent and the rare worker that follows it gives the advisor a free signal. But the advisor's
  control flow must treat the DQ as best-effort, not load-bearing, for fix-impl tasks.
- **Belt-and-braces brief wording** (use for fix-impl briefs whose validation the advisor will
  run anyway): "Write the validate DQ, **commit it + push it to the worker branch BEFORE you
  write your retro** (the daemon finalize-merge runs after you exit; an un-pushed DQ is invisible)."
  This nudges the worker to push the DQ before the boundary — but still verify-and-run-directly
  regardless.

## Distinction from the finalize-merge-gap family

This is a DIFFERENT failure from [[feedback_junior_finalize_skips_when_worker_pre_pushes]]
(daemon's local-delta check defeated by worker pre-push) and
[[feedback_finalize_merge_where_to_look_first]] (daemon-local-first push ordering). Those are
about *commits* not reaching the phase tip. This is about the *DQ entry* never being authored —
the code commits land fine via finalize-merge; only the validation-handoff DQ is missing.

Diagnose by checking whether the fix commit touched `decision-queue.json` at all
(`git show --stat <fix-sha> | grep decision-queue`). Empty → worker skipped the DQ-write (this
lesson). Present-on-worker-branch-but-not-phase-tip → a real finalize-merge drop (the other
lessons). Falsify before concluding which — the recovery differs.

## Recurrence / promotion

1st confirmed 2026-06-18 (m3-core-infra, 2 workers same cohort — so really 2 instances in one
event). If a 3rd fix-impl task in a later phase skips the validate DQ, promote to a
`pattern_*` (cross-cutting fix-impl-worker behaviour) and consider a structural nudge: the
impl-task subagent contract (`.claude/agents/impl-task.md`) could make the validate-DQ write a
hard finalize-gate the way the retro is. Until then: advisor verify-and-run-directly is the
durable mitigation.
