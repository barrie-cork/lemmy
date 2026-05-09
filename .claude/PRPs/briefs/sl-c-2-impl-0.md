---
role: impl-task
plan_task: 0
phase: v1-SL-c-2
created: 2026-05-08
related_dq: null
---

# Brief — v1-SL-c-2 Task 0 — Pre-flight harness audit + branch verification + SL-a/SL-b/c-1 state confirmation

## 1. Role + dispatch line

`[role:impl-task] sl-c-2-impl-0 — see .claude/PRPs/briefs/sl-c-2-impl-0.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 0
from `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` §13 —
verification probes **only**. **No commits. No file modifications.**
Plan §13 Task 0 is pure environment audit; produces a probe-result
summary in your task output.

## 2. Scope

**Produce:**

A written probe-result summary in your task output. For each of
Probes 0, -1, 1-18 (the full set from plan §13 Task 0), capture exit
code, output excerpt, and PASS/FAIL verdict. Total: 20 probes.

**Do NOT** in this task:

- Touch ANY files (`crates/**`, `migrations/**`, `.claude/**`,
  `docs/**`, `scripts/**`, `Cargo.toml`, anything else).
- Make any git commits. Junior's daemon finalize-merge has nothing
  to merge — the worker branch should be at the same SHA as
  `phase-v1-SL-c-2` tip.
- Run cargo locally — Shape G plan; cargo runs on GH-hosted runners
  after push (and Task 0 doesn't push because it makes no commits).
- Proceed to Task 1 logic on any probe failure or critical anomaly.

## 3. Required reading

Read in this order before running any probe:

1. **Plan §13 Task 0**
   (`.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md`) — the
   full probe list (Probes 0, -1, 1-18). This brief mirrors it; the
   plan is canonical.
2. **`.claude/rules/pre-phase-harness-audit.md`** — the harness audit
   rule (R5: enumerate ALL probes); plan adapts the OS-aware wrapper
   guidance to Linux + the Junior daemon.
3. **`.claude/rules/pm-plugin-hooks-stable.md`** — the 6 load-bearing
   hook names Probe 14 checks (NOT 7 — the rule's "seven hooks"
   includes the notification hook which is in `notify.rs` and not
   probed by literal-string check).
4. **Lessons** (Glob `.claude/lessons/`, read any whose name matches
   `harness`, `audit`, `probe`, `submodule`, `pipes`, `e2e_edit`,
   `features_full`):
   - `feedback_pipes_mask_exit_codes.md` — never pipe cargo / probe
     output through grep/head; capture to file + check `$?`
     separately.
   - `feedback_worktree_submodules_not_auto_init.md` — Probe -1
     catches this on Linux/Junior worktrees.
   - `feedback_pre_phase_dod_smoke_test.md` — informs probe shapes
     (Task 0 is the impl-side counterpart to the advisor-side DoD
     smoke).
   - `feedback_junior_worker_e2e_edit_hang.md` — load-bearing for
     c-2 (5 anchor-Edit tasks against `crates/server/tests/e2e.rs`
     which is now 11,925 lines on `governance-v0` per advisor-side
     pre-flight). Watchpoint #12 in plan §4.2 cites this lesson.
     Task 0 does not Edit e2e.rs (no commits at all), but read so
     subsequent Tasks 1-5 inherit the discipline.
   - `feedback_features_full_p_crate_incompatible.md` — never use
     `-p <crate> --features full` (clippy/test invocations); use
     `--workspace --features full`. Bound on c-2 because §15.7
     manual snippets reference `cargo test -p lemmy_server --test
     e2e --features full` which is permitted ONLY because the
     `--test e2e` selector restricts to a single test target while
     keeping workspace feature resolution. Tasks 1-5 should never
     fall back to `-p <crate>` in workflow YAMLs.

## 3a. Handover from prior cohort

(none — first task of `phase-v1-SL-c-2`. Branch tip per BM-cut
Junior task #151 at SHA `47af884c8` (`chore(advisor): author bm-cut
brief for v1-SL-c-2 — see /auto-phase first dogfood`), which is the
trunk tip at the moment bm-cut ran. The `phase-v1-SL-c-2` branch on
origin points at this same SHA — Junior's bm-cut runlog append
landed via the daemon's finalize-merge, but the trunk tip was
unchanged because bm-cut commits to the phase branch only. Worker
worktree branches downstream of `phase-v1-SL-c-2` HEAD.)

**Cross-PR carry-forward (c-1 → c-2):** the c-1 PR (#121) merged at
`8bfc085dc` on 2026-05-08 18:42 UTC. c-1 shipped:
- `crates/api/api/src/governance/sponsor_liability_grace.rs` (4 pub
  async fns: `run_grace_check_batch`, `evaluate_escape_conditions`,
  `fire_or_escape_case`, `check_grace_staleness`)
- `pub mod sponsor_liability_grace;` declared in
  `crates/api/api/src/governance/mod.rs` (line 39)
- `SPONSOR_LIABILITY_GRACE_RUNNING: AtomicBool` + `Drop` impl +
  clokwerk tick block in `crates/routes/src/utils/scheduled_tasks.rs`
- `BREHON_DISABLE_GRACE_CHECK_JOB` env-var precedence (1 quoted
  literal at line 326 + 2 comment mentions)

c-2 imports the c-1 module directly from each test (Tasks 1-5 call
`lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch`).
c-2 ships ZERO new module / scheduler / migration code.

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branched off `phase-v1-SL-c-2`
  (tip `47af884c8` per `git log -1 --oneline phase-v1-SL-c-2`).
- `git branch --show-current` must return a `junior/role-impl-task-...`
  branch (NOT `phase-v1-SL-c-2` or `governance-v0` directly). Plan
  §13 Probe 1 EXPECT line says `phase-v1-SL-c-2`; **adapt to your
  actual worktree branch name and confirm the worktree is downstream
  of `phase-v1-SL-c-2` HEAD via**
  `git merge-base --is-ancestor phase-v1-SL-c-2 HEAD || git log --oneline phase-v1-SL-c-2..HEAD`.
- **No commits at task end.** Junior's daemon will see no diff and
  the finalize-merge will be a no-op — that's correct behaviour for
  Task 0.

### Probe discipline

- Run ALL probes in plan §13 order (Probes 0, -1, 1, 2, 3, 4, 5, 6,
  7, 7b, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18).
- Capture probe output to `/tmp/sl-c-2-task0-probe-N.log` (writable
  temp on the EliteDesk Junior worktree).
- Check `$?` after each probe separately — do not pipe through
  `head`/`grep` in a way that masks the upstream exit code. Per
  `feedback_pipes_mask_exit_codes.md`.
- **On any Probe 1, 2, 3, 4, 5, 6, 7, 7b, 8, 9, 12, 13, 14, or 16
  FAIL:** STOP immediately. Write a DQ pending entry
  (`from: "impl"`, `kind: "blocker"`) with the probe output + exit
  code. Commit + push the DQ entry to your worktree branch
  immediately per `.claude/rules/decision-queue.md` "Mid-task
  visibility".
- **Probe 7 is CRITICAL** for c-2 specifically — c-2 depends on c-1's
  module landing on `governance-v0`. The plan's Probe 7 STOP-clause
  is: `test -f crates/api/api/src/governance/sponsor_liability_grace.rs
  && echo "C1 MODULE FILE PRESENT" || { echo "C1 MODULE NOT FOUND
  — c-2 cannot proceed; check c-1 PR merge status"; exit 1; }`.
  At brief-write time c-1 PR #121 is merged at `8bfc085dc` (see §3a
  carry-forward); the file MUST be present.
- **On Probe 0 (Docker):** plan binds this to c-2 (e2e harness uses
  testcontainers-rs). Print `DOCKER OK` or `DOCKER NOT RUNNING`. If
  Docker is not running, this is **NON-BLOCKING for Task 0** (no
  e2e in Task 0) but advisory for Tasks 1-5 (those write tests but
  don't run them — Phase 2 e2e is laptop-side or workflow-dispatch).
  Print but do NOT STOP.
- **On Probe -1 (submodule init):** if `git submodule status`
  shows uninitialised entries (lines starting `-`), run
  `git submodule update --init --recursive`. Print exit code.
  Non-zero on the init step is a STOP.
- **On Probe 10 (SL-b merge confirmation):** plan marks this
  informational. SL-b shipped at `governance-v0 9ae4c332c` on
  2026-05-07 per `bm-runlog.md`. File MUST be present. If absent,
  STOP — c-2 anchor-Edit assumes `mod v1_sl_b_fixtures` exists.
- **On Probe 11 (last fixture mod):** report whatever the actual
  last `^mod v1_` is. Plan EXPECTs `mod v1_sl_b_fixtures` (post-
  SL-b-merge) — that's the c-2 Task 1 anchor.
- **On Probe 15 (concurrent-PR check):** if output is non-empty,
  report to advisor via task output — do not STOP; advisor decides.
- **On Probe 17 (yamllint):** plan marks this informational; no
  STOP. Soft-fail print is acceptable.
- **On Probe 18 (DQ pending list):** plan marks this advisory.
  Report findings to task output. At brief-write time DQ pending
  count is 0 (verified post-c-1 merge cleanup). If pending entries
  appear, report them; do not STOP.

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

- All 20 probes (0, -1, 1, 2, 3, 4, 5, 6, 7, 7b, 8, 9, 10, 11, 12,
  13, 14, 15, 16, 17, 18) run.
- Probes 1, 2, 3, 4, 5, 6, 7, 7b, 8, 9, 12, 13, 14, 16 PASS (per
  plan §13 EXPECT clauses; these are STOP-class).
- Probes 0, -1, 10, 11, 15, 17, 18 either PASS or report
  informational findings without STOP.
- Task output contains a one-line PASS/FAIL verdict per probe.
- No DQ pending entries written from this task (or, if any, they
  are blocker-class with full evidence).
