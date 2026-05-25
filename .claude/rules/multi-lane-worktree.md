# Multi-lane worktree discipline

When multiple Brehon sub-phases (`phase-v1-*`) are concurrently active, each phase MUST have its own git worktree on the human side. This rule loads at session start.

## Why this rule exists

Per `.claude/PRPs/reports/v1-RT-r1-halt-retro.md` (commit `ffa2876e3`) L4 + user
decision 2026-05-11 (option a — worktree-per-lane). When two advisor sessions
operate on the same on-disk checkout (e.g. `C:/Users/barri/Developer/brehon-fork`)
and both write `.claude/decision-queue.json` on different phase branches, the
shared file path produces:

- Working-tree races (checkout of phase-A modifies the file; checkout of
  phase-B sees stale state).
- Merge conflicts on every phase-branch reconcile cycle (3 cycles in
  v1-RT-r1 alone, ~4 hours wallclock overhead).
- Cross-lane DQ id collisions (each session computes `next_id` against its
  own working-tree view).
- Reflog HEAD-move surprises across sessions sharing the same `.git/`.

Per-worktree isolation removes the root cause: `.claude/decision-queue.json`
becomes a per-worktree file path, and each phase branch has exactly one
human-side writer.

## Layout

```
C:/Users/barri/Developer/
├── brehon-fork                 ← canonical checkout; tracks governance-v0; meta-edits only (rules, lessons, templates, briefs landed on trunk)
├── brehon-fork-tooling         ← existing worktree (tooling-local-validation branch)
├── brehon-fork-<lane>          ← per-lane worktree, one per active phase-v1-<lane>
└── …
```

The canonical `brehon-fork` checkout is reserved for `governance-v0` work:
authoring briefs (committed to trunk), plan files (after planner Junior
finalize-merges), rule edits, lesson edits, template edits, retro authorship.
**The canonical checkout MUST NOT be used to mutate phase-branch DQ entries**
(applies to both Mode A and Mode B below). The carve-out for plan-time DQ
writes on `governance-v0` (clarify entries, planning-time blockers) is
documented in Hard refusal #6 below.

Each active sub-phase in Mode A (`phase-v1-SL-d`, `phase-v1-RT-r1`,
`phase-v1-RT-r2`, `phase-v1-JM-f`, …) gets its own worktree. In Mode B,
no laptop-side phase worktree exists — see §"Lane modes" below.

## Lane modes (added 2026-05-25)

A lane operates in one of two modes. Both modes are valid; pick at
bm-cut time and document in the lane's bootstrap handover.

### Mode A — Dedicated lane worktree (default)

Laptop side has a phase worktree at
`C:/Users/barri/Developer/brehon-fork-<lane>`, running its own Claude
Code session.

- Lane session CWD = `brehon-fork-<lane>`, branch = `phase-v1-<lane>`.
- Lane session writes phase-branch DQ entries, runs validate-pending-laptop
  cargo locally, authors impl-task briefs directly on the phase branch
  (no trunk→phase sync needed).
- Canonical `brehon-fork` session does meta-edits on `governance-v0`
  (rules, lessons, templates, retros). Does NOT mutate phase-branch DQ.
- Lifecycle per §"Lifecycle" below (worktree add → drive → worktree remove).
- Use when: you have laptop disk + RAM headroom for a second worktree
  (~3–5 GB after submodules + .git) AND you'll be at the laptop the
  whole phase.

### Mode B — Mobile remote-control (added 2026-05-25)

Laptop has ONLY the canonical `brehon-fork` checkout. All phase-branch
work happens via Junior tasks on the daemon (EliteDesk). The advisor
session runs in canonical `brehon-fork` and dispatches everything via
`mcp__junior-brehon__create_task` with `base_branch=phase-v1-<lane>`.

- Lane session CWD = `brehon-fork` (canonical), branch = `governance-v0`.
- Advisor authors impl-task briefs in canonical, commits to
  `governance-v0`, then triggers a trunk→phase sync (per §"Brief
  location and trunk→phase sync" below) so the brief is visible to
  workers forking from `phase-v1-<lane>`.
- Junior workers run in per-task worktrees on the daemon. They write
  worker-branch DQ entries; daemon-side finalize-merges them into
  `phase-v1-<lane>` and pushes.
- The lane's `.claude/decision-queue.json` mutations happen on the
  worker branch (Junior's mid-task push) → daemon merges into
  `phase-v1-<lane>` → advisor sees them on `origin/phase-v1-<lane>`
  fetch. **The advisor's canonical checkout does NOT write DQ entries
  for the phase branch in Mode B** — phase DQ writes happen on the
  daemon, not the laptop.
- The canonical-checkout "MUST NOT mutate phase-branch DQ entries"
  rule (§Layout) STILL applies — Mode B doesn't relax it. Plan-time DQ
  writes on `governance-v0` (clarify entries, planning-time blockers)
  remain allowed per the §Layout carve-out + Hard refusal #6's
  "legitimate `governance-v0` plan-time DQ writes" clause.
- Use when: laptop is constrained (mobile / low disk), OR the lane
  will run mostly while you're away from the laptop, OR you want to
  minimize per-lane worktree bootstrap overhead.

### How to tell which mode you're in

```bash
git -C C:/Users/barri/Developer/brehon-fork worktree list
# Mode A: shows `brehon-fork-<lane>` for the active phase
# Mode B: shows only `brehon-fork` (canonical)
```

The lane's bootstrap handover (`.claude/PRPs/handovers/<phase>-bootstrap.md`)
SHOULD state the mode in its RESUME block. Example phrasing:
> "Lane mode for this <phase> session: Mode B (mobile remote-control).
> User driving from canonical brehon-fork; all impl/bm phase-branch
> work dispatched as Junior tasks targeting `phase-v1-<lane>`."

## Brief location and trunk→phase sync (added 2026-05-25)

Per `advisor-orchestrator.md` §2.1: **impl-task briefs MUST be visible
on the phase branch the worker forks from** (because the worker's
worktree starts at `phase-v1-<lane>` HEAD; if the brief isn't at that
ref, the worker can't read it). This applies regardless of mode.

The procedure differs by mode:

### Mode A — author directly on phase branch

The lane worktree session is already on `phase-v1-<lane>`. Author the
brief there, `git commit`, `git push origin phase-v1-<lane>`. The
worker forking from the phase branch sees the brief immediately.

### Mode B — author on trunk, sync to phase via daemon SSH

The canonical session cannot `git checkout phase-v1-<lane>` (Hard
refusal #1) and the lane worktree doesn't exist on the laptop. The
working primitive uses the daemon's main worktree, which is on
`phase-v1-<lane>` after bm-cut completes (per bm-cut.md §7 "Daemon
worktree state post-bm-cut"):

```bash
# Step 1 (laptop, canonical): author brief on governance-v0, commit, push.
git add .claude/PRPs/briefs/<phase>-impl-<n>.md
git commit -m "chore(advisor): brief <phase> impl-task <n> — <slug>"
git push origin governance-v0

# Step 2 (daemon, via SSH): merge trunk into the phase branch from the
# daemon's main worktree (it's on phase-v1-<lane> post-bm-cut). The
# merge bridges the new brief commit into the phase branch.
ssh homeserver "cd /srv/<repo> \
  && git fetch origin governance-v0 \
  && git merge origin/governance-v0 --no-edit -m 'Merge governance-v0 into <phase> — pull impl-<n> brief for Task <n> dispatch' \
  && git push origin phase-v1-<lane>"

# Step 3 (laptop, canonical): queue the Junior task.
# mcp__junior-brehon__create_task with base_branch=phase-v1-<lane>.
```

**Precondition for step 2:** daemon worktree must be on the phase
branch. Verify with `ssh homeserver 'cd /srv/<repo> && git symbolic-ref HEAD'`.
If it returns `refs/heads/phase-v1-<lane>`, proceed. If it returns a
different branch (governance-v0, another phase), a concurrent task
switched it — recover by `git checkout phase-v1-<lane>` on the daemon
before merging. Do NOT delete and re-cut; the branch already has
upstream tracking from bm-cut Phase 4.

**Alternative for step 2 (Mode B, if daemon worktree is unavailable):**
queue a tiny one-line bm-task with description `[role:bm-task]
trunk-sync <phase> — merge governance-v0 into phase-v1-<lane> for
brief visibility — see .claude/PRPs/briefs/<phase>-trunk-sync-<n>.md`.
This costs one Junior task per brief and is heavier than the SSH
merge but doesn't depend on daemon-worktree state. Add a
`bm-merge-forward` verb in the future if this alternative recurs
across phases.

**Mode A skip:** if you're in Mode A, this whole section doesn't
apply — author the brief on the phase branch directly.

## Lifecycle

Three steps, fire once per lane:

1. **After bm-cut creates `phase-v1-<lane>`:** `git worktree add ../brehon-fork-<lane> phase-v1-<lane>` from canonical checkout. Open a new Claude Code session with CWD = `brehon-fork-<lane>` (= dedicated lane advisor).
2. **During the phase:** lane session writes DQ to its own worktree's `.claude/decision-queue.json`, commits + pushes to `origin/phase-v1-<lane>`, dispatches Junior tasks with `base_branch=phase-v1-<lane>`. Canonical `brehon-fork` session continues meta-edits on `governance-v0` (briefs, rules, lessons) but MUST NOT mutate phase-branch DQ entries.
3. **At phase ship:** after `bm-merge`, run `git worktree remove ../brehon-fork-<lane>` + `git branch -d phase-v1-<lane>` from canonical checkout.

Full bash commands + bootstrap-checklist forward-ref (submodules, `.mcp.json`, `.env`, `settings.local.json`): `.claude/refs/multi-lane-mechanics.md` §"Lifecycle".

## Session-start ritual

At Claude Code session start (governance-v0 lane OR phase-v1-<lane>), run:

```bash
pwd                                              # confirm CWD
git branch --show-current                        # confirm branch
git worktree list                                # see ALL active worktrees
```

If `git worktree list` shows another active worktree on a `phase-v1-*`
branch (Mode A for that lane), the current session MUST verify:

- Its CWD matches the intended lane (or governance-v0 for the canonical
  meta-edit lane).
- It will NOT write `.claude/decision-queue.json` outside that lane.

If `git worktree list` shows ONLY canonical `brehon-fork` and the
intended lane is supposed to be active (per workflow_state /
roadmap.json), the lane is in Mode B (mobile remote-control) — the
canonical session drives the lane via Junior dispatch with
`base_branch=phase-v1-<lane>`. See §"Lane modes" + §"Brief location
and trunk→phase sync".

If the session was opened in the wrong CWD (e.g. user opened Claude Code
in `brehon-fork` intending to drive a lane in Mode A), surface to user
and ask whether to:
- (a) switch to Mode A by closing + reopening Claude Code in
  `brehon-fork-<lane>` (requires the worktree to exist; `git worktree
  add ../brehon-fork-<lane> phase-v1-<lane>` from canonical creates it).
- (b) switch to Mode B and drive the lane from canonical via Junior
  dispatch. Document the mode flip in the lane bootstrap handover.
- (c) proceed in `brehon-fork` for meta-edits only (no lane work this
  session).

## Hard refusals

1. **Never run `git checkout phase-v1-*` inside `brehon-fork`** — that's a
   destructive cross-lane operation. Use the dedicated worktree.

2. **Never write `.claude/decision-queue.json` from `brehon-fork`** for an
   entry that belongs on a phase branch. The DQ on `governance-v0` only
   holds entries authored at plan/brief time (advisor planning DQs,
   clarify entries), not validate-pending or ci-watcher mutations.

3. **Never run `git push --force` against another lane's branch** from
   any worktree. Per `.claude/rules/no-destructive-defaults.md`.

4. **Never delete a worktree directory directly with `rm -rf`** — use
   `git worktree remove <path>` so `.git/worktrees/<name>/` admin state
   gets cleaned.

5. **Never share Claude Code sessions across worktrees** — one session,
   one CWD, one lane. To switch lanes: close session, open a new one in
   the target worktree.

6. **Atomic read-mutate-commit for any DQ write on the canonical
   `brehon-fork` checkout.** Hard refusal #2 forbids *phase-branch* DQ
   writes from the canonical checkout, but **legitimate `governance-v0`
   plan-time DQ writes** (advisor planning DQs, clarify entries, gate-1
   pre-seeds — explicitly allowed by #2's carve-out) STILL race
   concurrent CC sessions that share the canonical `.git/` and may
   commit `decision-queue.json` between a session's file-mutate and its
   commit. Confirmed 2026-05-16 (v1-AD-e gate-1): a concurrent session's
   `b114937b8` (DQ #229 move) landed between the first `#237/#238`
   append and its commit, **silently discarding the uncommitted
   append** — `git add` reported "nothing added" and the work was lost
   until re-applied. The required protocol for ANY canonical-checkout DQ
   write:

   1. `git fetch origin governance-v0` immediately before the write.
   2. Read `decision-queue.json` fresh (do NOT rely on a read from
      earlier in the session — a concurrent session may have rewritten
      it).
   3. Re-compute `next_id` across all lanes per "Worktree-aware DQ id
      discipline" below (a concurrent session may have consumed ids).
   4. Mutate → verify the JSON (`python -c "json.load(...)"` +
      assert the new ids present) → `git add` → `git commit` →
      `git push` **as a single uninterrupted shell sequence**, NOT
      across multiple tool calls. Minimise the window between
      file-mutate and commit.
   5. After push, verify the entry survived (`git log -1 --stat` +
      re-read). If the commit reported "nothing added" or the entry is
      absent post-push, a concurrent commit clobbered the working-tree
      change between mutate and `git add` — re-run from step 1.

   The structural fix (still future scope) is that gate-1 pre-seed DQ
   writes should happen on a lane-dedicated worktree even *before*
   bm-cut, OR a PreToolUse guard should refuse canonical-checkout DQ
   writes when `.claude/agent-activity.json` shows another write-mode
   session. Until then, the atomic protocol above is mandatory.

## Worktree-aware DQ id discipline

Schema-v3 makes cross-lane next_id coordination obsolete. Each CC session generates its own 12-hex UUID prefix (`.claude/.dq-session-id`) + monotonic 3-digit sequence; namespaces never intersect, structurally eliminating the global-monotonic-integer race.

Generate via `bash scripts/brehon/dq-v3-new-entry.sh`. Pre-v3 entries retain integer `id` + gain `id_v1: <int>` alias via migration. The `feedback_cohort_dq_id_collision.md` pre-reservation workaround is superseded.

Full post-v3 historical context: `.claude/refs/multi-lane-mechanics.md` §"Worktree-aware DQ id discipline".

## PMD is cross-lane shared, NOT per-lane isolated

The decision-queue is deliberately **per-lane isolated** (each worktree
owns its own `.claude/decision-queue.json` — see §"Layout" + §"Hard
refusals" #2). The **project-memory DB (PMD) is the exact opposite**:
lessons, retros, and patterns are **global knowledge** that every lane
must read and write to a **single canonical store**.

The canonical PMD is **`C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`**
(the canonical checkout's `.project-memory/`, never a per-worktree copy).

### Hard invariant

Every worktree's `.mcp.json` (gitignored — holds API keys) MUST set the
`project-memory` server's `PROJECT_MEMORY_DB` to the **absolute canonical
path above** — NEVER a relative `.project-memory/memory.db` (that
resolves against the per-worktree `PROJECT_ROOT` and strands writes in a
lane-local DB) and NEVER a `brehon-fork-<lane>/.project-memory/...` path.

The tracked `.mcp.json.example` template encodes this with a
`_comment_pmd_cross_lane` guard key. When bootstrapping a new lane
worktree's `.mcp.json` from the template, the absolute canonical
`PROJECT_MEMORY_DB` carries over verbatim — only `PROJECT_ROOT` changes
per worktree.

### Why this invariant is load-bearing

The Stop hook `.claude/hooks/retro-check.sh` resolves the PMD via
`git rev-parse --git-common-dir` → which from **any** worktree points at
the **canonical** `brehon-fork/.git`, so the hook always reads
`brehon-fork/.project-memory/memory.db`. If a lane's MCP writes retros
to its own lane-local DB instead, the hook can never see them: the agent
writes genuine retros and the hook false-blocks indefinitely (observed
on v1-ship-1: ~27+ false Stop-hook blocks across the phase; all 21
v1-ship-1 retros stranded in `brehon-fork-ship-1/.project-memory/memory.db`,
invisible to the canonical-DB-reading hook). Pinning every lane's MCP to
the canonical absolute path makes MCP-writes and hook-reads converge.

The hook file is **NOT** the thing to fix here — its git-common-dir
resolution is correct (it intentionally lands on the canonical shared
DB). The defect class is always MCP-side: a relative or per-lane
`PROJECT_MEMORY_DB`. Never edit the hook to "fix" a stranded-retro
symptom; fix the offending lane's `.mcp.json`.

See `.claude/lessons/feedback_pmd_cross_lane_canonical_db.md` for the
full incident + the diagnosis recipe.

## Daemon side (EliteDesk)

The daemon at `/srv/brehon-fork` uses `git worktree add` per Junior task. Lane-isolation rule applies to the HUMAN-SIDE laptop checkout only. Detail: `.claude/refs/multi-lane-mechanics.md` §"Daemon side".

## Migration plan (existing topology → multi-lane)

Multi-lane adoption complete as of v1-RT-r1 (2026-05-11). New lanes bootstrapped at bm-cut time per §"Lifecycle".

## See also

- `.claude/rules/branch-manager.md` "Session-start ritual" — BM-side ritual.
- `.claude/rules/advisor-orchestrator.md` §1 "Polling loop" — adds CWD check
  to the polling tick.
- `.claude/rules/decision-queue.md` "Mid-task visibility" — push discipline
  unchanged across worktrees.
- `.claude/rules/no-destructive-defaults.md` — never `rm -rf` a worktree.
- `feedback_multi_lane_worktree_discipline.md` — companion lesson with retro
  evidence + practical session-flow examples.
