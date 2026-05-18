# Multi-lane worktree discipline

When multiple Brehon sub-phases (`phase-v1-*`) are concurrently active, each
phase MUST have its own git worktree on the human side. This rule loads at
session start.

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
**The canonical checkout MUST NOT be used to mutate phase-branch DQ entries.**

Each active sub-phase (`phase-v1-SL-d`, `phase-v1-RT-r1`, `phase-v1-RT-r2`,
`phase-v1-JM-f`, …) gets its own worktree.

## Lifecycle

### 1. After bm-cut creates a new phase branch

When the BM Junior task creates `phase-v1-<lane>`, the advisor (or user)
runs ONCE on the laptop:

```bash
cd C:/Users/barri/Developer/brehon-fork
git fetch origin phase-v1-<lane>
git worktree add ../brehon-fork-<lane> phase-v1-<lane>
```

A new Claude Code session opens with CWD = `C:/Users/barri/Developer/brehon-fork-<lane>`.
That session is the **dedicated advisor for the lane** until phase ship.

### 2. During the phase

The lane-dedicated advisor session:

- Writes DQ entries (validate-pending raises, advisor mutations, clarify
  entries) to the worktree's `.claude/decision-queue.json` — which is a
  **separate working-tree file from `brehon-fork`'s copy**, but lives at the
  same logical path within `phase-v1-<lane>`.
- Commits + pushes to `origin/phase-v1-<lane>`.
- Dispatches Junior tasks with `base_branch=phase-v1-<lane>`.
- Polls + reconciles its own lane only.

The canonical `brehon-fork` session continues to:

- Author briefs (committed to `governance-v0`).
- Edit rules/lessons/templates (committed to `governance-v0`).
- Pull recent governance-v0 commits to stay current.
- Do NOT mutate phase-branch DQ entries.

### 3. At phase ship + merge

When `bm-merge` completes (PR merged into governance-v0, phase branch
deleted from origin), the user removes the worktree:

```bash
cd C:/Users/barri/Developer/brehon-fork
git worktree remove ../brehon-fork-<lane>
git branch -d phase-v1-<lane>   # delete local tracking
```

The lane-dedicated Claude Code session ends (or transitions to the next
phase by opening a fresh worktree).

## Session-start ritual

At Claude Code session start (governance-v0 lane OR phase-v1-<lane>), run:

```bash
pwd                                              # confirm CWD
git branch --show-current                        # confirm branch
git worktree list                                # see ALL active worktrees
```

If `git worktree list` shows another active worktree on a `phase-v1-*`
branch, the current session MUST verify:

- Its CWD matches the intended lane (or governance-v0 for the canonical
  meta-edit lane).
- It will NOT write `.claude/decision-queue.json` outside that lane.

If the session was opened in the wrong CWD (e.g. user opened Claude Code
in `brehon-fork` intending to drive RT-r1), surface to user and ask whether
to (a) switch CWD by closing + reopening Claude Code in
`brehon-fork-rt-r1`, or (b) proceed in `brehon-fork` for meta-edits only.

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

Per `.claude/rules/decision-queue.md` "Archive policy" + "Mid-task visibility"
+ next-id cross-archive rule: when computing `next_id` for a new DQ entry,
walk:

```
- .claude/decision-queue.json (current worktree's view)
- .claude/decision-queue-archive-*.json (current worktree's view)
- bash scripts/brehon/git-show-json.sh origin/<other-active-lane> .claude/decision-queue.json (per other active worktree)
```

This widens the cross-archive rule to include cross-worktree refs.
Implementation: `scripts/brehon/resolve-dq-canonical.sh` already supports
spanning phase-branch + active worker branches; extend it to also walk
`git worktree list` output and compute `next_id` across all visible refs.
**Future scope** — for now, advisor sessions manually check the largest id
across `origin/phase-v1-*` refs before picking next_id.

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

The daemon at `/srv/brehon-fork` uses `git worktree add` per Junior task
(`feedback_parallel_agents_one_worktree_per_agent.md`). That mechanism is
unchanged. The lane-isolation rule applies to the HUMAN-SIDE laptop checkout
only.

## Migration plan (existing topology → multi-lane)

For the current state where `brehon-fork` is the shared checkout:

1. **Phase v1-RT-r1 (paused at `37a62f9b4`):** after L4 ship, before resuming,
   cut `brehon-fork-rt-r1` worktree off `phase-v1-RT-r1`:
   ```bash
   git worktree add ../brehon-fork-rt-r1 phase-v1-RT-r1
   ```
2. **Phase v1-SL-d (shipped 2026-05-11):** no new worktree needed; merged.
3. **Future phases:** worktree at bm-cut time per §"Lifecycle" above.

## See also

- `.claude/rules/branch-manager.md` "Session-start ritual" — BM-side ritual.
- `.claude/rules/advisor-orchestrator.md` §1 "Polling loop" — adds CWD check
  to the polling tick.
- `.claude/rules/decision-queue.md` "Mid-task visibility" — push discipline
  unchanged across worktrees.
- `.claude/rules/no-destructive-defaults.md` — never `rm -rf` a worktree.
- `feedback_multi_lane_worktree_discipline.md` — companion lesson with retro
  evidence + practical session-flow examples.
