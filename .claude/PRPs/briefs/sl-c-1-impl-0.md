---
role: impl-task
plan_task: 0
phase: v1-SL-c-1
created: 2026-05-07
related_dq: null
---

# Brief — v1-SL-c-1 Task 0 — Pre-flight harness audit + branch verification + SL-a/SL-b state confirmation

## 1. Role + dispatch line

`[role:impl-task] sl-c-1-impl-0 — see .claude/PRPs/briefs/sl-c-1-impl-0.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 0
from `.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md` §13 —
verification probes **only**. **No commits. No file modifications.**
Plan §13 Task 0 is pure environment audit; produces a probe-result
summary in your task output.

## 2. Scope

**Produce:**

A written probe-result summary in your task output. For each of
Probes 0, -1, 1-14 (the full set from plan §13 Task 0), capture exit
code, output excerpt, and PASS/FAIL verdict. Total: 16 probes.

**Do NOT** in this task:

- Touch ANY files (`crates/**`, `migrations/**`, `.claude/**`,
  `docs/**`, `scripts/**`, `Cargo.toml`, anything else).
- Make any git commits. Junior's daemon finalize-merge has nothing
  to merge — the worker branch should be at the same SHA as
  `phase-v1-SL-c-1` tip.
- Run cargo locally — Shape G plan; cargo runs on GH-hosted runners
  after push (and Task 0 doesn't push because it makes no commits).
- Proceed to Task 1 logic on any probe failure or critical anomaly.

## 3. Required reading

Read in this order before running any probe:

1. **Plan §13 Task 0**
   (`.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md`) — the
   full probe list (Probes 0, -1, 1-14). This brief mirrors it; the
   plan is canonical.
2. **`.claude/rules/pre-phase-harness-audit.md`** — the harness audit
   rule (R5: enumerate ALL probes); plan adapts the OS-aware wrapper
   guidance to Linux + the Junior daemon.
3. **`.claude/rules/pm-plugin-hooks-stable.md`** — the 6 load-bearing
   hook names Probe 11 checks (NOT 7 — the rule's "seven hooks"
   includes the notification hook which is in `notify.rs` and not
   probed by literal-string check).
4. **Lessons** (Glob `.claude/lessons/`, read any whose name matches
   `harness`, `audit`, `probe`, `submodule`, `pipes`):
   - `feedback_pipes_mask_exit_codes.md` — never pipe cargo / probe
     output through grep/head; capture to file + check `$?`
     separately.
   - `feedback_worktree_submodules_not_auto_init.md` — Probe -1
     catches this on Linux/Junior worktrees.
   - `feedback_pre_phase_dod_smoke_test.md` — informs probe shapes
     (Task 0 is the impl-side counterpart to the advisor-side DoD
     smoke).

## 3a. Handover from prior cohort

(none — first task of `phase-v1-SL-c-1`. Branch tip per BM-cut
Junior task #140 at SHA `477f0c55c` (`chore(bm): branch cut —
phase-v1-SL-c-1 off governance-v0`), which sits 1 commit ahead of
`governance-v0` tip `c93cf7e90` — that 1 commit is the runlog
append. Worker worktree is downstream of this tip.)

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branched off `phase-v1-SL-c-1`
  (tip `477f0c55c` per `git log -1 --oneline phase-v1-SL-c-1`).
- `git branch --show-current` must return a `junior/role-impl-task-...`
  branch (NOT `phase-v1-SL-c-1` or `governance-v0` directly). Plan
  §13 Probe 1 EXPECT line says `phase-v1-SL-c-1`; **adapt to your
  actual worktree branch name and confirm the worktree is downstream
  of `phase-v1-SL-c-1` HEAD via**
  `git merge-base --is-ancestor phase-v1-SL-c-1 HEAD || git log --oneline phase-v1-SL-c-1..HEAD`.
- **No commits at task end.** Junior's daemon will see no diff and
  the finalize-merge will be a no-op — that's correct behaviour for
  Task 0.

### Probe discipline

- Run ALL probes in plan §13 order (Probe 0, -1, 1, 2, 3, 4, 5, 6,
  7, 8, 9, 10, 11, 12, 13, 14).
- Capture probe output to `/tmp/sl-c-1-task0-probe-N.log` (writable
  temp on the EliteDesk Junior worktree).
- Check `$?` after each probe separately — do not pipe through
  `head`/`grep` in a way that masks the upstream exit code. Per
  `feedback_pipes_mask_exit_codes.md`.
- **On any Probe 1, 2, 3, 4, 5, 6, 8, 9, 10, or 11 FAIL:** STOP
  immediately. Write a DQ pending entry (`from: "impl"`,
  `kind: "blocker"`) with the probe output + exit code. Commit +
  push the DQ entry to your worktree branch immediately per
  `.claude/rules/decision-queue.md` "Mid-task visibility".
- **On Probe 0 (Docker):** plan marks this NON-BLOCKING for c-1
  (the workspace-check workflow doesn't use Docker; e2e is c-2's
  gate). Print `DOCKER OK` or `DOCKER NOT RUNNING (non-blocking
  for c-1; binds c-2)`. Do NOT STOP.
- **On Probe -1 (submodule init):** if `git submodule status`
  shows uninitialised entries (lines starting `-`), run
  `git submodule update --init --recursive`. Print exit code.
  Non-zero on the init step is a STOP.
- **On Probe 7 (SL-b shipped check):** plan marks this advisory
  with a fall-through path — `SL-b SHIPPED` is the expected branch
  (the brief verifies SL-b PR #119 merged on 2026-05-07 at
  governance-v0 `9ae4c332c`). If the file is missing, print
  `SL-b NOT YET MERGED — c-1 can still ship its module + scheduler
  wiring; c-2 anchor-Edit fallback applies` and continue. Do NOT
  STOP.
- **On Probe 12 (concurrent-PR check):** if output is non-empty,
  report to advisor via task output — do not STOP; advisor decides.
- **On Probe 13 (yamllint):** plan marks this informational; no
  STOP. Soft-fail print is acceptable.
- **On Probe 14 (DQ pending list):** plan marks this advisory.
  Report findings to task output. Expect at minimum DQ #156
  (validate-pending fail-record from SL-b e2e tip 620861f08, kept in
  pending[] per option-2 audit-trail rule). If additional pending
  entries appear, report them; do not STOP.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.
- Any pending DQ entry written from this task uses `from: "impl"`.

### Hard refusals

- Do NOT modify any source files. Task 0 is verification-only.
- Do NOT modify the plan, brief, or `.claude/decision-queue.json`
  (except to ADD a pending entry on a STOP-class probe failure;
  never modify resolved entries or the plan file).
- Do NOT push the worker branch unless writing a DQ blocker entry.
- Do NOT run cargo locally. Shape G — cargo runs off-box.
- Do NOT touch `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.

### Sentinel sequence (mid-task visibility)

Per `.claude/rules/decision-queue.md` "Mid-task visibility": if you
write a DQ blocker mid-task, push the worker branch IMMEDIATELY
(not at task end) so the advisor can see it on next polling tick.
The push is the ONE non-finalize push allowed from this task.

## 5. Acceptance

Task 0 passes if:

- All 16 probes (0, -1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
  14) run.
- Probes 1, 2, 3, 4, 5, 6, 8, 9, 10, 11 PASS (per plan §13 EXPECT
  clauses).
- Probes 0, -1, 7, 12, 13, 14 either PASS or report informational
  findings without STOP.
- Task output contains a one-line PASS/FAIL verdict per probe.
- No DQ pending entries written from this task (or, if any, they
  are blocker-class with full evidence).
