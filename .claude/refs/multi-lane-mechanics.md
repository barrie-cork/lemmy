# Multi-lane worktree mechanics

Externalised from `.claude/rules/multi-lane-worktree.md` on 2026-05-22
(rule-trim pass). The rule file kept the layout summary, session-start
ritual, hard refusals, PMD-cross-lane invariant, and the daemon-side
one-paragraph note. This file holds the lifecycle procedures
(setup/teardown) that fire once per lane and the post-v3 historical
context for worktree-aware DQ id discipline.

Cite this file as `multi-lane-mechanics.md §"Lifecycle"` /
`multi-lane-mechanics.md §"Worktree-aware DQ id discipline"`.

> **Loading note:** this file is NOT auto-loaded at session start.
> Read on-demand when (a) bootstrapping a new phase lane, (b) shipping
> a phase and tearing down its lane, or (c) reviewing legacy DQ id
> discipline for a pre-v3 entry.

## Lifecycle

### 1. After bm-cut creates a new phase branch

When the BM Junior task creates `phase-v1-<lane>`, the advisor (or
user) runs ONCE on the laptop:

```bash
cd C:/Users/barri/Developer/brehon-fork
git fetch origin phase-v1-<lane>
git worktree add ../brehon-fork-<lane> phase-v1-<lane>
```

A new Claude Code session opens with CWD = `C:/Users/barri/Developer/brehon-fork-<lane>`.
That session is the **dedicated advisor for the lane** until phase ship.

See also `feedback_phase_lane_worktree_bootstrap_checklist.md` —
submodule init + `.mcp.json` + `.env` + `settings.local.json` copies
are NOT auto-handled by `git worktree add`; the bootstrap checklist is
mandatory.

### 2. During the phase

The lane-dedicated advisor session:

- Writes DQ entries (validate-pending raises, advisor mutations,
  clarify entries) to the worktree's `.claude/decision-queue.json` —
  which is a **separate working-tree file from `brehon-fork`'s copy**,
  but lives at the same logical path within `phase-v1-<lane>`.
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

The lane-dedicated Claude Code session ends (or transitions to the
next phase by opening a fresh worktree).

## Worktree-aware DQ id discipline (post-v3 — historical context only)

Schema-v3 (post-v1-dq-schema-r1) makes cross-lane next_id coordination
obsolete. Under schema-v3, each CC session generates its own 12-hex
UUID prefix (cached in `.claude/.dq-session-id`) and a per-session
monotonic 3-digit sequence counter. Because no two sessions share a
UUID prefix, their id namespaces never intersect — the global-monotonic-
integer race that required the cross-lane coordination described in the
pre-v3 version of this section is structurally eliminated.

To generate a new DQ entry id, run
`bash scripts/brehon/dq-v3-new-entry.sh`. The script reads or creates
`.claude/.dq-session-id`, scans the live DQ + archives for the highest
sequence in this session, and prints the next composite id (e.g.
`a1b2c3d4e5f6-001`). Cross-lane id deduplication is no longer needed
for new entries — run the script in any worktree without coordination.

Pre-v3 entries retain their original integer `id` and gain
`id_v1: <int>` as a back-compat alias added by the
`dq-schema-v3-migrate.sh` migration. Citations like "DQ #50" continue
to resolve via `id == 50` on legacy entries. The
`resolve-dq-canonical.sh` resolver handles mixed int/string id sorts
via `str(e['id'])` coercion (Task 3 of v1-dq-schema-r1).

The pre-v3 workaround documented in
`feedback_cohort_dq_id_collision.md` (advisor pre-reserving N DQ ids
before cohort dispatch) is now superseded by the v3 composite-id
mechanism. That lesson's `## Status` section marks it accordingly.
Pre-reservation stubs from pre-v3 cohorts remain as historical
entries; no cleanup required.

## Daemon side (EliteDesk)

The daemon at `/srv/brehon-fork` uses `git worktree add` per Junior
task (`feedback_parallel_agents_one_worktree_per_agent.md`). That
mechanism is unchanged. The lane-isolation rule applies to the
HUMAN-SIDE laptop checkout only.

## Migration plan (existing topology → multi-lane)

Multi-lane adoption is complete as of v1-RT-r1 (2026-05-11). New lanes
are bootstrapped at bm-cut time per §"Lifecycle" above.

## See also

- `.claude/rules/multi-lane-worktree.md` — layout, session-start
  ritual, hard refusals, PMD cross-lane invariant.
- `.claude/rules/decision-queue.md` §"Schema (v3)" — composite-id
  contract.
- `feedback_phase_lane_worktree_bootstrap_checklist.md` — submodules
  + `.mcp.json` + `.env` + `settings.local.json` bootstrap.
- `feedback_cohort_dq_id_collision.md` — pre-v3 race the v3
  composite-id mechanism eliminates.
