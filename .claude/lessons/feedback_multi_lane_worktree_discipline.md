---
name: multi-lane worktree discipline
description: Each concurrently-active Brehon phase needs its own git worktree on the human side. Shared checkout = .claude/decision-queue.json file conflicts + 3+ reconcile cycles per phase.
type: feedback
---

# Multi-lane worktree discipline

When two or more Brehon sub-phases (`phase-v1-*`) are concurrently active,
each phase MUST have its own git worktree at `C:/Users/barri/Developer/brehon-fork-<lane>`.
A shared `C:/Users/barri/Developer/brehon-fork` checkout for both advisor sessions
produces unavoidable file-level conflicts on `.claude/decision-queue.json` plus
daemon/origin divergence cycles that consume hours of advisor wallclock.

**Why:** Per `.claude/PRPs/reports/v1-RT-r1-halt-retro.md` (commit `ffa2876e3`) L4 +
user decision 2026-05-11 (option a — worktree-per-lane). Two advisor sessions ran
in parallel for SL-d (shutter session) + RT-r1 (this advisor) on the SAME laptop
checkout. Both wrote `.claude/decision-queue.json` on different phase branches.
Symptoms:

- 3 daemon/origin divergence cycles on `phase-v1-RT-r1` alone, each requiring
  manual `ssh homeserver "git pull --no-rebase" → Python merge-resolve → push`
  (~10 min each × 3 = ~30 min just on bookkeeping).
- 4-way DQ id collision when Cohort A members all computed `next_id = 189` off
  their worker branches simultaneously.
- HEAD-move surprises across sessions (reflog showed `checkout: moving from
  phase-v1-RT-r1 to origin/phase-v1-SL-d` mid-session — the OTHER session's
  checkout had moved my HEAD because of shared `.git/`).
- ~4 hours total advisor wallclock dominated by recovery work vs ~30 min
  Junior compute.

The lesson generalises: any active phase-v1-* branch with DQ writes from a
human-side advisor session needs an isolated working-tree file path for
`.claude/decision-queue.json`. Git worktrees give exactly that.

**How to apply:**

### After bm-cut

```bash
cd C:/Users/barri/Developer/brehon-fork
git fetch origin phase-v1-<lane>
git worktree add ../brehon-fork-<lane> phase-v1-<lane>
```

Open a new Claude Code session in `C:/Users/barri/Developer/brehon-fork-<lane>`. That session is the dedicated advisor for the lane.

### Throughout the phase

- Lane-dedicated session: writes DQ entries, dispatches Junior tasks, polls
  Junior status, mutates DQ on workflow results. Pushes to its phase branch.
- Canonical `brehon-fork` session: authors briefs (commits to governance-v0),
  edits rules/lessons/templates (commits to governance-v0). Does NOT mutate
  phase-branch DQ.

### At phase ship + merge

```bash
cd C:/Users/barri/Developer/brehon-fork
git worktree remove ../brehon-fork-<lane>
git branch -d phase-v1-<lane>
```

### Session-start ritual

```bash
pwd
git branch --show-current
git worktree list
```

If another worktree exists on a different `phase-v1-*` branch, verify CWD
matches the intended lane before any DQ write.

**Edge cases:**

- **Cross-lane meta-edits** (rule changes, lesson promotions, brief authoring,
  template revisions) belong on `governance-v0` and should be authored in the
  canonical `brehon-fork` worktree. The lane-dedicated worktrees inherit those
  via `git pull origin governance-v0` if needed (e.g. an SL-d session wants to
  cite a new rule mid-phase).
- **Cherry-picking from one lane to another** (rare: SL-d ships a lesson that
  RT-r1 wants to cite before SL-d's PR merges into governance-v0): do the
  cherry-pick on `governance-v0` in the canonical worktree, then `git pull`
  in the consuming worktree.
- **Daemon side at `/srv/brehon-fork`** uses `git worktree add` per Junior
  task already (per `feedback_parallel_agents_one_worktree_per_agent.md`).
  That mechanism is unchanged. Lane isolation applies to the human-side
  laptop checkout only.

**Anti-patterns to avoid:**

- ❌ `git checkout phase-v1-RT-r1` inside `brehon-fork` when an RT-r1 worktree
  exists at `brehon-fork-rt-r1`. Use the worktree.
- ❌ Sharing a Claude Code session across worktrees by `cd`-ing between them.
  One session, one CWD, one lane.
- ❌ Deleting the worktree directory with `rm -rf` instead of `git worktree
  remove`. The latter cleans up `.git/worktrees/<name>/` admin state.
- ❌ Writing `.claude/decision-queue.json` from `brehon-fork` for an entry
  that conceptually belongs on a phase branch.

**Detection (retro):** a phase-branch DQ that needed manual merge-resolution
against origin during the phase is evidence of the shared-checkout pattern
(or daemon push-skip pattern — see `feedback_junior_finalize_skips_when_worker_pre_pushes.md`).
The first occurrence is the human-side root cause; resolve via worktree.

**Companion lessons:**
- `feedback_parallel_agents_one_worktree_per_agent.md` — sibling principle on
  the Junior daemon side.
- `feedback_check_brehon_fork_user_memory.md` — broader on per-session memory
  isolation.
- `feedback_settings_local_json_worktree_bootstrap.md` — `.claude/settings.local.json`
  is per-worktree by default.

**Where codified:** `.claude/rules/multi-lane-worktree.md` (new rule, 2026-05-11).
