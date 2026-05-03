# Advisor handover — 2026-05-03 — sl-a-task-0-shipped-cohort-a-ready

**Written:** 2026-05-03T08:58Z
**Author:** advisor session (`C:\Users\barri\Developer\brehon-fork`)
**Branch:** `governance-v0` @ `93146ada2` (`chore(advisor): brief sl-a-impl-0 — Task 0 pre-flight + migrate-roundtrip.sh stub fix (DQ #114)`)
**Purpose:** Self-contained brief for next-session resume after SL-a Task 0 ships and Cohort A is ready to dispatch.

## TL;DR

- **This session shipped:** SL-a planning (Junior #81 done @ 08:20Z; plan `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` 2541 lines) → DQ #116 user-resolved (proceed-as-one) → bm-cut (Junior #82 done @ 08:34Z; `phase-v1-SL-a` cut at `ea322cd0a`) → Task 0 (Junior #83 done @ 08:56Z; migrate-roundtrip.sh stub replaced per DQ #114, finalize-merged into `phase-v1-SL-a` at `1b47a3ff4`).
- **Pending:** 0 DQ pending. 2 open PRs (`#108` clippy carry-forward, `#109` rust-lld follow-up) — both `mergeStateStatus: UNKNOWN`, advisor-gate unclear; not blockers for SL-a.
- **Blocked:** nothing.
- **Next session must:** queue Cohort A (Tasks 1+2+3 [P]) per plan §13 + cohort dispatch rules. Tasks 1+2+3 are file-disjoint (migration / `enums.rs` / `schema.rs` separately) and have FILES YAML blocks asserting disjointness. Authors 3 impl-task briefs, cherry-picks them onto `phase-v1-SL-a`, queues all 3 in parallel.
- **External gate:** Phase-2 e2e (workflow-dispatch-only) is Tasks-1-through-8-from-now; no immediate gate active.

## Cold-resume sequence

1. Read `CLAUDE.md` + `.claude/rules/*.md` (auto-loaded in `-p` mode; manually in interactive: `advisor-orchestrator.md` is the orchestrator law, `branch-manager.md`, `decision-queue.md`, `phase-branch.md`, `pre-phase-harness-audit.md`).
2. Read this file in full.
3. Read `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` §13 lines 1415-1605 (Tasks 1, 2, 3) + the FILES YAML block on each (mechanical YAML-overlap check confirms disjointness pre-cohort dispatch).
4. Read `.claude/decision-queue.json` — confirm pending = 0 (no new entries since 08:56Z).
5. Read `.claude/runlog/bm-runlog.md` tail — last 5 entries for state context.
6. Verify state:
   ```bash
   git fetch origin
   git rev-parse HEAD                                  # expect 93146ada2
   git status --short                                  # expect M .claude/hooks/retro-check.sh + 2 untracked (NOT mine — leave alone)
   git rev-parse origin/governance-v0                  # expect 93146ada2 (synced)
   git rev-parse origin/phase-v1-SL-a                  # expect 1b47a3ff4 (Task 0 finalize-merge)
   gh pr list --repo barrie-cork/lemmy --state open    # expect #108 + #109 (carry-forward, not SL-a)
   ssh homeserver 'cd /srv/brehon-fork && git branch --show-current'  # expect phase-v1-SL-a
   ssh homeserver 'sqlite3 /srv/brehon-fork/.junior/junior.db "SELECT id, status FROM jobs WHERE id IN (81,82,83);"'  # all 3 = done
   ```
7. Append cold-resume event to runlog:
   ```
   2026-05-0?T??:??Z | advisor | meta | cold-resume | handover=advisor-2026-05-03-sl-a-task-0-shipped-cohort-a-ready.md drift=<none|<detail>>
   ```

## State at handover

### Git

- **Laptop branch:** `governance-v0` @ `93146ada2` (synced with origin).
- **EliteDesk branch:** `phase-v1-SL-a` @ `1b47a3ff4` (Task 0 finalize-merge; pushed to origin).
- **Working tree (laptop):**
  - `M .claude/hooks/retro-check.sh` — **NOT mine.** User-authored edit (widens retro-check window from 15→60 min on non-junior branches). Leave it alone.
  - `?? .claude/PRPs/plans/tier-3-pg-dump-restore-template.plan.md` — untracked, not mine.
  - `?? .claude/PRPs/reviews/pr-107-redflag-ack.md` — untracked, not mine.
- **Unpushed:** none.

### Worktrees

```
C:/Users/barri/Developer/brehon-fork         93146ada2 [governance-v0]
C:/Users/barri/Developer/brehon-fork-tooling 3a233c077 [tooling-local-validation]
```

The `tooling-local-validation` worktree is for laptop tier-3 cargo+e2e validation; not relevant to SL-a impl flow.

### Open PRs

| # | Title | Branch | Base | mergeState | review |
|---|---|---|---|---|---|
| 109 | chore(local-tooling): rust-lld linker + profile.dev tweaks (PR #107 follow-up) | `chore/local-tooling-rust-lld` | `governance-v0` | UNKNOWN | (none) |
| 108 | fix: clippy::map_err_ignore on accept_jury_assignment (JM-e carry-forward) | `chore/clippy-map-err-ignore-jm-e` | `governance-v0` | UNKNOWN | (none) |

Neither is on the SL-a critical path. Both are user-authored carry-forwards from JM-e shipping. Don't touch.

### Decision queue

- **Pending:** 0
- **Recently resolved (this session):**
  - DQ #114 (user; clarify) — migrate-roundtrip.sh stub-fix scope → SL-a Task 0 (Option A)
  - DQ #115 (advisor; clarify) — `liability.grace_window_*_hours` keys are raw integer hours, NOT micros-scaled
  - DQ #116 (user; blocker) — proceed-as-one for v1-SL-a (complexity 13 > 8 split-or-proceed gate)
- **Resolved count:** 113 total

### Junior tasks (this session)

| # | Role | Status | Result | Output |
|---|---|---|---|---|
| 81 | planning | done @ 08:20Z | 62-min runtime; plan + DQ #116 raised | `v1-sponsor-liability-a.plan.md` 2541 lines on `governance-v0` |
| 82 | bm-task (bm-cut) | done @ 08:34Z | ~2 min Haiku | `phase-v1-SL-a` cut at `ea322cd0a` (worker over-pushed — see retro-harvest below) |
| 83 | impl-task (Task 0) | done @ 08:56Z | ~10 min Sonnet | migrate-roundtrip.sh stub replaced; commit `1f6131bf8`, finalize `1b47a3ff4` |

### Plan / PRD / ADR in-flight

- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — committed, plan-approval gate passed (DoD smoke + watchpoint specificity ✓).
- No PRD or ADR edits in flight.
- `.claude/PRPs/plans/tier-3-pg-dump-restore-template.plan.md` — untracked, not mine.

### Relays

- **Advisor → impl awaiting impl response:** none (no relay files written this session — DQ + briefs were the coordination surface).
- **Impl → advisor awaiting advisor answer:** none.

## What's pending (ordered by priority)

### 1. Author + queue Cohort A (Tasks 1+2+3 [P]) on `phase-v1-SL-a`

**Context:** Plan §13 marks Tasks 1, 2, 3 as `[P]`; Task 4 is the barrier (non-`[P]`). Per `.claude/rules/brehon-cohort-dispatch.md` (canonical at `advisor-orchestrator.md`), cohort dispatch sequence:

- (a) Read each task's FILES YAML block from §13. Compute pairwise intersections of `union(creates, modifies)`.
- (b) If intersections non-empty, **degrade to serial**.
- (c) Budget check is non-binding under Shape G (cargo runs off-box).
- (d) Forbidden-window check is non-binding under Shape G (off-box).
- (e) If all checks pass, queue all 3 tasks via parallel `mcp__junior-brehon__create_task` calls.

**Pre-checks (run before authoring briefs):**

```bash
# Read FILES YAML blocks
sed -n '/### Task 1 \[P\]:/,/### Task 2/p' C:/Users/barri/Developer/brehon-fork/.claude/PRPs/plans/v1-sponsor-liability-a.plan.md | grep -A 10 "^FILES:\|^creates:\|^modifies:"
sed -n '/### Task 2 \[P\]:/,/### Task 3/p' C:/Users/barri/Developer/brehon-fork/.claude/PRPs/plans/v1-sponsor-liability-a.plan.md | grep -A 10 "^FILES:\|^creates:\|^modifies:"
sed -n '/### Task 3 \[P\]:/,/### Task 4/p' C:/Users/barri/Developer/brehon-fork/.claude/PRPs/plans/v1-sponsor-liability-a.plan.md | grep -A 10 "^FILES:\|^creates:\|^modifies:"
```

Tasks at a glance (file-class):
- **Task 1:** CREATE `migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/{up,down}.sql`
- **Task 2:** UPDATE `crates/db_schema_file/src/enums.rs` (CaseStatus extension)
- **Task 3:** UPDATE `crates/db_schema_file/src/schema.rs` (moderation_case columns)

These three should be disjoint by definition; YAML overlap check is mechanical confirmation.

**Brief authoring path (3 briefs):**
- `.claude/PRPs/briefs/sl-a-impl-1.md` (Task 1)
- `.claude/PRPs/briefs/sl-a-impl-2.md` (Task 2)
- `.claude/PRPs/briefs/sl-a-impl-3.md` (Task 3)

Mirror `sl-a-impl-0.md` template structure (frontmatter + 7 sections). Plan §13 task body is canonical source; each brief MUST cite plan-line range for §3 Required reading. §3a "Handover from prior cohort" should reference Task 0's commit `1f6131bf8` and its `keyDecisions: ["replaced stub per DQ #114"]`.

**Dispatch:**

1. Commit all 3 briefs on `governance-v0` in one push.
2. Cherry-pick all 3 brief commits onto `phase-v1-SL-a` (per retro-harvest item — briefs must be reachable from worker base branch).
3. Push `phase-v1-SL-a` to origin.
4. Run `/precheck` to confirm EliteDesk readiness.
5. Queue all 3 Junior tasks in parallel (`mcp__junior-brehon__create_task` × 3 in one message).

**Expected outcome:** 3 worker branches push their respective `validate-pending` (workspace-check workflow) DQ entries; advisor dispatches 3 ci-watchers; cohort advancement waits for all 3 `result: "pass"`.

**Rollback if cohort overlap detected:** degrade to serial. Queue Task 1 alone, wait for finalize, then Task 2, then Task 3. Surface the overlap with: `cohort overlap detected: tasks <A>+<B> share <path> — degrading to serial`. **Do not file a DQ for this** — the planner's `[P]` marker was wrong, retro flags it.

### 2. Track 3 retro-harvest items for SL-a retro

These are advisor-side notes for the SL-a retrospective. Don't act on them mid-phase; harvest at retro time.

**(a) bm-cut Junior over-pushed phase-v1-SL-a.** Brief §5 + `bm-cut.md` Phase 3 say *"Pushed?: No (deferred to first commit + /bm-push)"*. Worker pushed `phase-v1-SL-a` to origin anyway. Harmless (empty branch on origin doesn't trigger CR), but a soft script-deviation. Investigate at retro: is `bm-cut.md` ambiguous in the Linux variant or did Haiku misread the script?

**(b) Settings.json model-field check is stale.** SL-a's bm-cut brief carried a JM-d carry-forward check for `.claude/settings.json` model field (`opus[1m]`). The brehon-fork's settings.json on this branch has no `model` field at all (only `effortLevel`, `skillListingBudgetFraction`, `hooks`). Either the field has been removed since JM-d or the JM-d setup was ad-hoc. Either way, future bm-cut briefs should not include the check unless there's a real field name to check.

**(c) Brief-on-phase-branch cherry-pick step is undocumented.** The advisor commits briefs on `governance-v0`, but Junior workers branch off `phase-v1-X` (the EliteDesk's currently-checked-out branch). Therefore briefs must be cherry-picked from `governance-v0` onto the active phase branch before queueing each impl/ci-watcher task. JM-e's history confirms the pattern. This step is **not** currently in `advisor-orchestrator.md` — should be documented in the "Stage shape" section.

### 3. Address 2 open carry-forward PRs (advisor-gate unclear)

PRs #108 + #109 both have `mergeStateStatus: UNKNOWN` (likely just stale GH state). Neither is on SL-a critical path. Defer until SL-a ships.

## What NOT to touch

Per `.claude/rules/branch-manager.md` and `.claude/rules/handover.md`:

- `crates/**`, `migrations/**`, `tests/**` — impl-owned. SL-a impl-tasks (Cohort A onward) are how these get touched.
- Active plan file `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — committed and approved; do not edit unless a planner DQ raises a defect.
- `.claude/hooks/retro-check.sh` — modified in working tree by user (not advisor). Leave as-is.
- Open PR findings YAML at `.claude/PRPs/reviews/pr-107-redflag-ack.md` — not advisor-owned; user's in-progress work.
- PRs #108 + #109 — carry-forward from JM-e, not SL-a.

## Notes on the four-role mechanical model (from this session's observations)

These refine the mental model in `advisor-orchestrator.md` "Stage-shape orchestration" — record for next session's awareness, not as authoritative rules.

- **Junior workers run in `/srv/brehon-fork` directly, NOT in isolated worktree directories.** The "worktree" is the `phase-*` or `junior/*` branch checkout in the shared repo (branch isolation, not directory isolation). `feedback_parallel_agents_one_worktree_per_agent.md` is misleading — it should read "one branch per worker."
- **Junior `base_branch` = the EliteDesk's currently-checked-out branch at queue time.** No per-task base specification is possible via MCP `create_task`. To dispatch off `phase-v1-SL-a`, EliteDesk must have it checked out before `create_task` fires.
- **Briefs must be cherry-picked onto the phase branch** before being readable to Junior workers (see retro-harvest 2(c) above).
- **bm-cut is itself a Junior task** that runs on `governance-v0` (cuts the phase branch off trunk); after bm-cut, advisor switches the EliteDesk to `phase-v1-X` for subsequent impl-tasks.

## Bootstrap prompt (paste into next session)

```
I'm resuming advisor work on the Brehon governance fork. Previous session
wrote a handover at `.claude/PRPs/handovers/advisor-2026-05-03-sl-a-task-0-shipped-cohort-a-ready.md`.

First steps:
1. Read CLAUDE.md + .claude/rules/*.md (auto-loaded in -p mode).
2. Read `.claude/PRPs/handovers/advisor-2026-05-03-sl-a-task-0-shipped-cohort-a-ready.md` in full.
3. Execute its "Cold-resume sequence" in order.
4. Report the TL;DR summary plus any state drift you detected back to me.

Do not take any state-changing actions until I confirm.
```

## Closing state assertions (verify on resume)

- `git branch --show-current` → `governance-v0`
- `git rev-parse HEAD` → `93146ada2de2ebbc47abd41b5024f88d7655906a`
- `git rev-parse origin/governance-v0` → `93146ada2de2ebbc47abd41b5024f88d7655906a` (synced)
- `git rev-parse origin/phase-v1-SL-a` → `1b47a3ff4...` (full SHA via `git rev-parse origin/phase-v1-SL-a` on resume)
- `git status --short` → `M .claude/hooks/retro-check.sh` + 2 untracked (`?? .claude/PRPs/plans/tier-3-pg-dump-restore-template.plan.md`, `?? .claude/PRPs/reviews/pr-107-redflag-ack.md`)
- `.claude/decision-queue.json` pending count → 0
- Junior #81/#82/#83 all `done`
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` exists on `governance-v0` (2541 lines)
- `phase-v1-SL-a` exists on origin with Task 0 finalize-merge as tip
- Active handover file exists: `test -f .claude/PRPs/handovers/advisor-2026-05-03-sl-a-task-0-shipped-cohort-a-ready.md`
