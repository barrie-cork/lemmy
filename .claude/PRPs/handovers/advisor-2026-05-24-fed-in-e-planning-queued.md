# Advisor handover — 2026-05-24 — fed-in-e-planning-queued

**Written:** 2026-05-24T10:55:00Z
**Author:** advisor session (C:\Users\barri\Developer\brehon-fork)
**Branch:** governance-v0 @ 347033bf8
**Purpose:** Self-contained brief for next-session resume.

## TL;DR

- Shipped v1-ship-3 (PR #149 merged), fixed EliteDesk daemon auth, ran /brehon-clarify on
  the new v1-federation-inbound-e brief (SHA-256 + per-peer actor-map bound), corrected 3
  brief defects (callsite count, constant location, 4th file), added [CLOSED] headers to old
  fed-in-e briefs, added daemon ff step to bootstrap checklist.
- Planning Junior task #451 is **running** against the correct brief (SHA-256/per-peer-bound).
- No DQ pending; 1 open PR (dependabot #136, unrelated); 2 dirty files in canonical checkout
  (role-customization leftovers, not yet committed).
- Next session: wait for task #451 to complete, run DoD smoke test + plan approval gate,
  then queue bm-cut for the new phase branch.
- No deadline or CI gate pending; task #451 has no CI watcher (planning = no cargo).

## Cold-resume sequence

1. Read `CLAUDE.md` + `.claude/rules/*.md` (auto-loaded).
2. Read this file in full.
3. Read `.claude/decision-queue.json` — verify pending = 0.
4. Read `.claude/runlog/bm-runlog.md` tail (last 10 lines for state context).
5. Verify state:
   ```bash
   git fetch origin
   git rev-parse HEAD              # expect 347033bf851994a2e583c343c0f80b469db57e2d
   git status --short              # expect 2 modified files (see §State)
   gh pr list --repo barrie-cork/lemmy --state open --json number,title,mergeStateStatus
   ```
6. Poll task #451:
   ```
   mcp__junior-brehon__show_task id=451
   ```
   - If `done`: fetch + read the new plan at `.claude/PRPs/plans/v1-federation-inbound-e.plan.md`
     and run the DoD smoke test (§What's pending item 1).
   - If `running`: wait and re-poll in ~5 min.
   - If `failed`: read task logs, triage failure.
7. Append cold-resume event to runlog:
   ```
   <ISO-UTC> | advisor | meta | cold-resume | handover=advisor-2026-05-24-fed-in-e-planning-queued.md drift=<none|<detail>>
   ```

## State at handover

### Git
- Branch: `governance-v0`
- HEAD: `347033bf8` (`chore(lessons): closed-brief headers + daemon ff step in bootstrap checklist`)
- Working tree: 2 modified files (uncommitted, from role-customization session):
  - `M .claude/hooks/role-signal-utilisation.sh` (+49/-13 lines, role-signal hook fix)
  - `M .claude/lessons/feedback_handover_assumptions_need_empirical_verification.md` (+21 lines)
- Unpushed: clean (HEAD == origin/governance-v0)

### Worktrees
| Path | Branch | Status |
|---|---|---|
| `C:/Users/barri/Developer/brehon-fork` | `governance-v0` | canonical; active advisor session |
| `C:/Users/barri/Developer/brehon-fork-rt-r2` | `phase-v1-RT-r2` | STALE — PR #150 merged 2026-05-24; worktree can be removed |
| `C:/Users/barri/Developer/brehon-fork-rt-r3` | `phase-v1-RT-r3` | cut 2026-05-24; no impl tasks dispatched yet |
| `C:/Users/barri/Developer/brehon-fork-ship-3` | `phase-v1-ship-3` | STALE — PR #149 merged 2026-05-24; worktree can be removed |

### Open PRs
| PR# | Title | Branch | mergeState | reviewDecision |
|---|---|---|---|---|
| 136 | deps(cargo): bump cargo-all group (12 updates) | dependabot/cargo/… | UNKNOWN | (none) |

No advisor-gate PRs open. Dependabot #136 is unrelated to current work; leave until a
dedicated deps-bump session.

### Decision queue
- Pending requiring advisor: **0**
- Pending requiring impl: 0
- Total resolved: 173

### Relays
- None pending in `.claude/runlog/advisor-relays/` or `impl-relays/`.

### Plan / PRD in-flight
- `.claude/PRPs/plans/v1-federation-inbound-e.plan.md` — being authored by task #451 (running).
  Do NOT edit until task #451 completes. The file on disk is the OLD advisory-lock plan
  (from the closed phase); task #451 will overwrite it with the SHA-256/per-peer-bound plan.
- No other plan/PRD/ADR files modified.

### Dirty canonical checkout files
These two files were modified by the role-customization session (session 4, handover at
`.claude/PRPs/handovers/role-customization-2026-05-24-session3.md`) and not yet committed:

1. `.claude/hooks/role-signal-utilisation.sh` — role-signal hook fix (T4a prep)
2. `.claude/lessons/feedback_handover_assumptions_need_empirical_verification.md` — lesson
   expansion

**Do NOT commit these in the fed-in-e planning session.** They belong to the
role-customization lane. The role-customization advisor session should commit them when it
resumes (session 4 handover covers this).

## What's pending (ordered by priority)

1. **Wait for task #451 (planning) and run DoD smoke test**
   - Context: Task #451 is running the SHA-256/per-peer-bound plan. When `done`, fetch the
     plan and run every §15 command literally (per `feedback_pre_phase_dod_smoke_test.md`).
   - Expected §15 DoD: `cargo check --workspace --features full` +
     `cargo clippy --workspace --features full --no-deps -- -D warnings` +
     `cargo test -p lemmy_apub_activities --lib` (22 lib tests).
   - Run on laptop: `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full"`
   - Check watchpoints in plan §4 cite specific function names + line numbers
     (per `feedback_advisor_watchpoint_specificity.md`).
   - Expected outcome: all DoD commands exit 0; plan approval surfaced to user (gate 1).
   - Rollback if DoD fails: file DQ pending asking planner to revise §15 or fix watchpoints.

2. **Surface plan approval to user (gate 1)**
   - Context: After DoD smoke test passes, summarise plan §1 goal, §13 tasks, §15 DoD,
     §16a stories to user and wait for explicit approval.
   - File: `.claude/PRPs/plans/v1-federation-inbound-e.plan.md` (once task #451 writes it)
   - Expected outcome: user says "approved" or requests changes.
   - Rollback: queue a planner amendment task if changes needed.

3. **Queue bm-cut for phase-v1-federation-inbound-e (NEW phase)**
   - Context: After gate 1, author bm-cut brief and queue `[role:bm-task] bm-cut` Junior.
     The new phase branch name will be `phase-v1-federation-inbound-e` — but that name was
     used by the closed phase. Confirm with user whether to reuse or suffix (e.g.
     `phase-v1-federation-inbound-e-2`). Per `feedback_daemon_local_trunk_stale_multi_lane.md`,
     fast-forward daemon local `governance-v0` before dispatch (checklist step 12).
   - File: `.claude/PRPs/briefs/v1-federation-inbound-e-bm-cut-2.md` (new brief; the old
     `v1-federation-inbound-e-bm-cut-1.md` is now [CLOSED])
   - Expected outcome: phase branch cut; worktree added at `../brehon-fork-fed-in-e-2`.
   - Rollback: if branch name conflicts, rename per user decision.

4. **Clean up stale worktrees**
   - Context: `brehon-fork-rt-r2` (phase-v1-RT-r2, merged PR #150) and `brehon-fork-ship-3`
     (phase-v1-ship-3, merged PR #149) are both stale post-merge.
   - Commands (from canonical checkout):
     ```bash
     git worktree remove --force ../brehon-fork-rt-r2
     git branch -d phase-v1-RT-r2
     git worktree remove --force ../brehon-fork-ship-3
     git branch -d phase-v1-ship-3
     ```
   - Note: `--force` required because these worktrees have submodules
     (per `feedback_worktree_remove_force_for_submodules.md`).
   - Expected outcome: `git worktree list` shows only 3 entries (canonical + rt-r3 + new fed-in-e).
   - Rollback: if remove fails, check for lock files; do not `rm -rf`.

5. **v1-RT-r3 planning brief** (parallel track, lower urgency)
   - Context: Lane `brehon-fork-rt-r3` is cut but has no planning brief yet. Bootstrap at
     `.claude/PRPs/handovers/v1-RT-r3-bootstrap.md`. Author brief → `/brehon-clarify` → queue
     planning Junior. Can run concurrently with fed-in-e impl tasks once that phase is in flight.
   - File: `.claude/PRPs/briefs/v1-RT-r3-planning-1.md` (to be authored)

## What NOT to touch

Per `.claude/rules/branch-manager.md` and `.claude/rules/handover.md`:

- `crates/**`, `migrations/**`, `tests/**` — impl-owned.
- `.claude/PRPs/plans/v1-federation-inbound-e.plan.md` — being written by task #451;
  do not edit until task completes and plan approval gate passes.
- `.claude/hooks/role-signal-utilisation.sh` and
  `.claude/lessons/feedback_handover_assumptions_need_empirical_verification.md` —
  role-customization session owns these dirty files; commit only from the role-customization
  advisor context.
- `PR #136` (dependabot) — leave until a dedicated deps session.

## Bootstrap prompt (paste into next session)

```
I'm resuming advisor work on the Brehon governance fork. Previous session
wrote a handover at `.claude/PRPs/handovers/advisor-2026-05-24-fed-in-e-planning-queued.md`.

First steps:
1. Read CLAUDE.md + .claude/rules/*.md (auto-loaded in -p mode).
2. Read `.claude/PRPs/handovers/advisor-2026-05-24-fed-in-e-planning-queued.md` in full.
3. Execute its "Cold-resume sequence" in order.
4. Report the TL;DR summary plus any state drift you detected back to me.

Do not take any state-changing actions until I confirm.
```

## Closing state assertions (verify on resume)

- `git branch --show-current` → `governance-v0`
- `git rev-parse HEAD` → `347033bf851994a2e583c343c0f80b469db57e2d`
- `git rev-parse origin/governance-v0` → `347033bf851994a2e583c343c0f80b469db57e2d`
- `git status --short` → 2 modified files (role-signal-utilisation.sh + feedback_handover_assumptions…)
- `.claude/decision-queue.json` pending count → 0
- Junior task #451 status → `done` or `running`
- Active handover file exists: `.claude/PRPs/handovers/advisor-2026-05-24-fed-in-e-planning-queued.md`
