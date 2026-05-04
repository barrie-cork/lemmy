---
role: impl-task
plan_task: 0
phase: v1-SL-b
created: 2026-05-04
related_dq: null
---

# Brief — v1-SL-b Task 0 — Pre-flight harness audit + branch verification + SL-a state confirmation

## 1. Role + dispatch line

`[role:impl-task] sl-b-impl-0 — see .claude/PRPs/briefs/sl-b-impl-0.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 0 from
`.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` §13 — verification probes
**only**. **No commits. No file modifications.** Plan §13 Task 0 is pure
environment audit; produces a probe-result summary in your task output.

## 2. Scope

**Produce:**

A written probe-result summary in your task output (Probes 0-12 from plan §13
Task 0). Each probe's exit code, output excerpt, and PASS/FAIL verdict.

**Do NOT** in this task:

- Touch ANY files (`crates/**`, `migrations/**`, `.claude/**`, `docs/**`,
  `scripts/**`, `Cargo.toml`, anything else).
- Make any git commits. Junior's daemon finalize-merge has nothing to merge —
  the worker branch should be at the same SHA as `phase-v1-SL-b` tip.
- Run cargo locally — Shape G plan; cargo runs on GH-hosted runners after
  push (and Task 0 doesn't push because it makes no commits).
- Proceed to Task 1 logic on any probe failure or critical anomaly.

## 3. Required reading

Read in this order before running any probe:

1. **Plan §13 Task 0** (`.claude/PRPs/plans/v1-sponsor-liability-b.plan.md`)
   — the full probe list (0-12). This brief mirrors it; the plan is canonical.
2. **`.claude/rules/pre-phase-harness-audit.md`** — the harness audit rule
   (R5: enumerate ALL probes); plan adapts the OS-aware wrapper guidance to
   Linux + the Junior daemon.
3. **`.claude/rules/pm-plugin-hooks-stable.md`** — load-bearing hook names
   Probe 10 checks.
4. **Lessons** (Glob `.claude/lessons/`, read any with keywords matching
   `harness`, `audit`, `probe`, `submodule`, `pipes`):
   - `feedback_pipes_mask_exit_codes.md` — never pipe output through grep/head;
     capture to file + check `$?` directly.
   - `feedback_worktree_submodules_not_auto_init.md` — Probe -1 catches this
     on Linux/Junior worktrees.
   - `feedback_pre_phase_dod_smoke_test.md` — DoD smoke informs probe shapes.

## 3a. Handover from prior cohort

(none — first task of phase-v1-SL-b. Branch tip per BM-cut Junior task #114
at SHA `ac8878d2d` `chore(bm): cut phase-v1-SL-b — runlog entry created`,
which sits 1 ahead of governance-v0 tip `6c217e0e0`.)

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branched off `phase-v1-SL-b` (tip
  `ac8878d2d` per `git log -1 --oneline phase-v1-SL-b`).
- `git branch --show-current` must return a `junior/role-impl-task-...`
  branch (NOT `phase-v1-SL-b` or `governance-v0` directly). Probe 1 in plan
  §13 expects `phase-v1-SL-b`; **adapt to your actual worktree branch name
  and confirm the worktree is downstream of `phase-v1-SL-b` HEAD via
  `git merge-base --is-ancestor phase-v1-SL-b HEAD || git log --oneline
  phase-v1-SL-b..HEAD`**.
- **No commits at task end.** Junior's daemon will see no diff and the
  finalize-merge will be a no-op — that's correct behaviour for Task 0.

### Probe discipline

- Run ALL probes in the plan §13 order (Probe 0, -1, 1, 2, 3, 4, 5, 6, 7,
  8, 9, 10, 11, 12).
- Capture probe output to `/tmp/sl-b-task0-probe-N.log` (writable temp on the
  EliteDesk Junior worktree).
- Check `$?` after each probe separately — do not pipe through `head`/`grep`
  in a way that masks the upstream exit code. Per
  `feedback_pipes_mask_exit_codes.md`.
- **On any Probe 0, 1, 2, 3, 4, 5, 6, 7, 10, or 11 FAIL:** STOP immediately.
  Write a DQ pending entry (`from: "impl"`, `kind: "blocker"`) with the probe
  output + exit code. Commit + push the DQ entry to your worktree branch
  immediately per `.claude/rules/decision-queue.md` "Mid-task visibility".
- **On Probe 8 (existing route line):** if output line number differs
  significantly from `~523`, that's INFORMATIONAL — Task 3 will anchor to the
  current line at task-time anyway. Just note it.
- **On Probe 9 (DQ pending list):** report findings; expect empty (DQ #143
  was resolved before this task). If non-empty, file a meta-DQ pending entry
  noting the leftover.
- **On Probe 11 (concurrent-PR check):** if output is non-empty, report to
  advisor via task output — do not STOP; advisor decides.
- **On Probe 12 (yamllint):** plan marks this informational; no STOP.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.
- Any pending DQ entry written from this task uses `from: "impl"`.

### Hard refusals

- Do NOT modify any source files. Task 0 is verification-only.
- Do NOT modify the plan, brief, or decision-queue.json (except to ADD a
  pending entry on probe failure; never modify resolved entries or the plan
  file).
- Do NOT push the worker branch unless writing a DQ blocker entry.
- Do NOT run cargo locally. Shape G — cargo runs off-box.
- Do NOT touch `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.

### Sentinel sequence (mid-task visibility)

Per `.claude/rules/decision-queue.md` "Mid-task visibility": if you write a
DQ blocker mid-task, push the worker branch IMMEDIATELY (not at task end) so
the advisor can see it on next polling tick. The push is the ONE non-finalize
push allowed from this task.

## 5. Acceptance

Task 0 passes if:

- All 14 probes (0, -1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12) run.
- Probes 0, 1, 2, 3, 4, 5, 6, 7, 10, 11 PASS (per plan §13 EXPECT clauses).
- Probes -1, 8, 9, 12 either PASS or report informational findings without
  STOP.
- Task output contains a one-line PASS/FAIL verdict per probe.
- No DQ pending entries from this task (or, if any, they are blocker-class
  with full evidence).
