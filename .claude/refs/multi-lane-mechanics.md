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

## Why this rule exists

> Relocated from `.claude/rules/multi-lane-worktree.md` on 2026-05-29
> (context-budget redesign B3). The rule file keeps only a one-line
> rationale + pointer here; the full failure-mode narrative lives below.
> 0 Pi/Claude-side heading citations — pure rationale, safe to externalise.

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

## Brief location and trunk→phase sync procedures

Per `advisor-orchestrator.md` §2.1: impl-task briefs MUST be visible on the phase branch the worker forks from. Procedure differs by mode.

### Mode A — author directly on phase branch

Lane worktree session is already on `phase-v1-<lane>`. Author brief there, `git commit`, `git push origin phase-v1-<lane>`. Worker sees it immediately. (Mode A skip: the whole trunk→phase sync below doesn't apply.)

### Mode B — author on trunk, sync to phase via daemon SSH

The canonical session cannot `git checkout phase-v1-<lane>` (Hard refusal #1) and the lane worktree doesn't exist on the laptop. Use the daemon's main worktree, which is on `phase-v1-<lane>` after bm-cut (per bm-cut.md §7):

```bash
# Step 1 (laptop, canonical): author brief on governance-v0, commit, push.
git add .claude/PRPs/briefs/<phase>-impl-<n>.md
git commit -m "chore(advisor): brief <phase> impl-task <n> — <slug>"
git push origin governance-v0

# Step 2 (daemon, via SSH): merge trunk into the phase branch from the
# daemon's main worktree (on phase-v1-<lane> post-bm-cut).
ssh homeserver "cd /srv/<repo> \
  && git fetch origin governance-v0 \
  && git merge origin/governance-v0 --no-edit -m 'Merge governance-v0 into <phase> — pull impl-<n> brief for Task <n> dispatch' \
  && git push origin phase-v1-<lane>"

# Step 3 (laptop, canonical): queue the Junior task.
# mcp__junior-brehon__create_task with base_branch=phase-v1-<lane>.
```

**Precondition for step 2:** daemon worktree must be on the phase branch. Verify with `ssh homeserver 'cd /srv/<repo> && git symbolic-ref HEAD'`. If it returns a different branch, a concurrent task switched it — recover by `git checkout phase-v1-<lane>` on the daemon before merging. Do NOT delete and re-cut; the branch already has upstream tracking from bm-cut Phase 4.

**Alternative for step 2 (if daemon worktree unavailable):** queue a tiny one-line bm-task `[role:bm-task] trunk-sync <phase> — merge governance-v0 into phase-v1-<lane> for brief visibility`. Heavier than the SSH merge but doesn't depend on daemon-worktree state. Add a `bm-merge-forward` verb if this recurs.

### Mode A — trunk-authorship from a phase lane (bm-merge brief + lesson edits)

When a lane session is locked to `phase-v1-<lane>` (Hard refusal #1) but must commit to `governance-v0`, use the daemon-temp-worktree SSH path:

```bash
ssh homeserver "cd /srv/<repo> \
  && git worktree add /tmp/brehon-gov-tmp governance-v0 \
  && cd /tmp/brehon-gov-tmp \
  && <author brief / edit file> \
  && git add <file> \
  && git commit -m 'chore(advisor): <subject>' \
  && git push origin governance-v0 \
  && git worktree remove /tmp/brehon-gov-tmp"
```

Canonical example: v1-RT-r3 bm-merge brief commit `621bc7115`.

### Single-file pull from governance-v0 into a phase lane (Mode A, laptop)

When a phase lane needs one specific governance-v0 artifact without inheriting all of governance-v0's diverged content (which conflicts on `v1-roadmap.json`), use surgical checkout — NOT a full merge:

```bash
# Step 1: ensure the file is on origin (commit + push on governance-v0 first).
# Step 2: in the phase lane worktree, fetch and checkout just the file.
git fetch origin governance-v0
git checkout FETCH_HEAD -- .claude/<target-file>
git commit -m "chore(decision-queue): pull <slug> from governance-v0"
```

**Why not full merge:** a full merge inherits every governance-v0 divergence including `v1-roadmap.json`, which always conflicts with an active phase branch. Surgical checkout is lossless and conflict-free. **Ordering:** `git push origin governance-v0` MUST precede the fetch — a local-only commit is invisible to another worktree's `git fetch`. Canonical example: 2026-05-31 DQ archive pull into `phase-v1-RT-r5` (`066791558` + `283f56ad5`).

## Hard refusal #6 — atomic read-mutate-commit protocol (full detail)

Hard refusal #2 forbids *phase-branch* DQ writes from the canonical checkout, but legitimate `governance-v0` plan-time DQ writes (advisor planning DQs, clarify entries, gate-1 pre-seeds) STILL race concurrent CC sessions that share the canonical `.git/` and may commit `decision-queue.json` between a session's file-mutate and its commit.

**Incident (2026-05-16, v1-AD-e gate-1):** a concurrent session's `b114937b8` (DQ #229 move) landed between the first `#237/#238` append and its commit, silently discarding the uncommitted append — `git add` reported "nothing added" and the work was lost until re-applied.

**Required protocol for ANY canonical-checkout DQ write:**

1. `git fetch origin governance-v0` immediately before the write.
2. Read `decision-queue.json` fresh (do NOT rely on an earlier read).
3. Re-compute `next_id` across all lanes (a concurrent session may have consumed ids).
4. Mutate → verify JSON (`python -c "json.load(...)"` + assert new ids present) → `git add` → `git commit` → `git push` as a single uninterrupted shell sequence, NOT across multiple tool calls.
5. After push, verify the entry survived (`git log -1 --stat` + re-read). If the commit reported "nothing added" or the entry is absent, a concurrent commit clobbered it — re-run from step 1.

Structural fix (future scope): gate-1 pre-seed DQ writes should happen on a lane-dedicated worktree even before bm-cut, OR a PreToolUse guard should refuse canonical-checkout DQ writes when `.claude/agent-activity.json` shows another write-mode session.

## See also

- `.claude/rules/multi-lane-worktree.md` — layout, session-start
  ritual, hard refusals, PMD cross-lane invariant.
- `.claude/rules/decision-queue.md` §"Schema (v3)" — composite-id
  contract.
- `feedback_phase_lane_worktree_bootstrap_checklist.md` — submodules
  + `.mcp.json` + `.env` + `settings.local.json` bootstrap.
- `feedback_cohort_dq_id_collision.md` — pre-v3 race the v3
  composite-id mechanism eliminates.
- `feedback_mode_b_trunk_phase_sync.md` — Mode B trunk→phase sync lesson.
- `feedback_single_file_pull_from_trunk.md` — surgical-checkout lesson.

## Hard refusal #7 — canonical-checkout foreign-WIP incident

**Incident (2026-06-07 retro-promotion leg):** the canonical `brehon-fork` checkout was sitting on `phase-m2-late-1` (a prior session had checked out the phase branch) while a second session ran a live Diesel regeneration in the SAME tree. The retro-promotion session staged only its files and ran `git commit`, but the phase-branch context meant `git add` reported "nothing added" (the staged files were already tracked in the phase-branch tree). The atomic burst (`&&`-chain) recovered it, but the race cost ~20 min. Pre-check (`git status --short` before any meta-edit) would have routed to option (a) — spin a dedicated lane worktree — or option (b) — defer — and avoided the race entirely.

**The lesson:** the `git status --short` pre-check is load-bearing, not advisory. Foreign uncommitted WIP in the canonical tree means another session is actively driving it — even if that session appears idle. The status check takes 2 seconds; losing the race costs 20–60 min.

## Hard refusal #1 — incidents

**2026-06-07 m2-late-1 T1:** a validation session needed to inspect the `phase-m2-late-1` tree in Mode B (no dedicated lane worktree). The session ran `git checkout phase-m2-late-1` inside the canonical `brehon-fork` checkout instead of creating a throwaway worktree. When `/compact` ran mid-session, the working-tree checkout was reset to `governance-v0` but conversation state retained the phase-branch context — all subsequent tool calls (8 steps) ran against the wrong tree. Recovery required re-reading the correct branch tip from origin and re-running all 8 steps.
