# Multi-lane worktree discipline

When multiple Brehon sub-phases (`phase-v1-*`) are concurrently active, each phase MUST have its own git worktree on the human side. This rule loads at session start. Full procedures + incident narratives: `.claude/refs/multi-lane-mechanics.md`.

## Why this rule exists

Two advisor sessions on the same on-disk checkout, both writing `.claude/decision-queue.json` on different phase branches, race the shared file path (working-tree races, reconcile conflicts, cross-lane DQ id collisions, reflog surprises). Per-worktree isolation makes the DQ a per-worktree file with one human-side writer per phase branch. Failure-mode narrative + v1-RT-r1 retro evidence: `.claude/refs/multi-lane-mechanics.md` §"Why this rule exists".

## Layout

```
C:/Users/barri/Developer/
├── brehon-fork                 ← canonical checkout; tracks governance-v0; meta-edits only
├── brehon-fork-tooling         ← existing worktree (tooling-local-validation branch)
├── brehon-fork-<lane>          ← per-lane worktree, one per active phase-v1-<lane>
└── …
```

Canonical `brehon-fork` is reserved for `governance-v0` work: briefs (committed to trunk), plan files (after planner finalize-merges), rule/lesson/template edits, retro authorship. **The canonical checkout MUST NOT mutate phase-branch DQ entries** (both modes). Carve-out for plan-time DQ writes on `governance-v0` (clarify entries, planning-time blockers): Hard refusal #6.

## Lane modes

A lane operates in one of two modes. Both valid; pick at bm-cut time, document in the lane's bootstrap handover.

| | Mode A — Dedicated lane worktree (default) | Mode B — Mobile remote-control |
|---|---|---|
| Laptop has | phase worktree at `brehon-fork-<lane>` + own CC session | ONLY canonical `brehon-fork` |
| Lane session CWD / branch | `brehon-fork-<lane>` / `phase-v1-<lane>` | `brehon-fork` / `governance-v0` |
| Phase-branch DQ writes | lane session writes them directly | happen on daemon (worker branch → daemon merge → advisor sees on fetch); canonical does NOT write phase DQ |
| impl-task briefs | author on phase branch directly | author on trunk + trunk→phase sync (below) |
| validate-pending cargo | lane session runs locally | dispatched as Junior tasks |
| Use when | laptop has disk+RAM headroom AND you're at the laptop the whole phase | laptop constrained/mobile, OR lane runs while away, OR minimizing per-lane bootstrap |

**How to tell which mode:** `git -C C:/Users/barri/Developer/brehon-fork worktree list` — Mode A shows `brehon-fork-<lane>`; Mode B shows only canonical. The lane bootstrap handover SHOULD state the mode in its RESUME block.

## Brief location and trunk→phase sync

impl-task briefs MUST be visible on the phase branch the worker forks from (worker's worktree starts at `phase-v1-<lane>` HEAD). Procedure differs by mode:

- **Mode A:** author on phase branch, commit, `git push origin phase-v1-<lane>`. Worker sees it immediately.
- **Mode B:** author on `governance-v0`, then SSH-merge trunk into the phase branch from the daemon's main worktree. Full bash + precondition check + bm-task alternative: `.claude/refs/multi-lane-mechanics.md` §"Brief location and trunk→phase sync procedures".
- **Mode A trunk-authorship** (lane locked to phase branch but must commit to `governance-v0` — bm-merge brief, lesson edit): daemon-temp-worktree SSH path. Procedure in refs §same.
- **Single-file pull from governance-v0 into a phase lane:** surgical `git checkout FETCH_HEAD -- <file>`, NOT a full merge (full merge conflicts on `v1-roadmap.json`). Push trunk first. Procedure in refs §same.

## Lifecycle

Three steps, once per lane:

1. **After bm-cut creates `phase-v1-<lane>`:** `git worktree add ../brehon-fork-<lane> phase-v1-<lane>` from canonical. Open a new CC session with CWD = `brehon-fork-<lane>`.
2. **During the phase:** lane session writes DQ to its own worktree, pushes to `origin/phase-v1-<lane>`, dispatches Junior with `base_branch=phase-v1-<lane>`. Canonical does meta-edits on `governance-v0`, MUST NOT mutate phase-branch DQ.
3. **At phase ship:** after `bm-merge`, `git worktree remove ../brehon-fork-<lane>` + `git branch -d phase-v1-<lane>` from canonical.

Full bash + bootstrap-checklist forward-ref (submodules, `.mcp.json`, `.env`, `settings.local.json`): `.claude/refs/multi-lane-mechanics.md` §"Lifecycle".

## Session-start ritual

```bash
pwd                         # confirm CWD
git branch --show-current   # confirm branch
git worktree list           # see ALL active worktrees
```

- Another worktree on a `phase-v1-*` branch (Mode A) → verify this session's CWD matches its intended lane (or governance-v0 for canonical) and it will NOT write DQ outside that lane.
- ONLY canonical `brehon-fork` shown but a lane should be active (per workflow_state/roadmap) → the lane is Mode B; canonical drives it via Junior dispatch with `base_branch=phase-v1-<lane>`.
- Wrong CWD (opened in `brehon-fork` intending Mode A lane work) → surface to user, ask: (a) switch to Mode A (close + reopen in `brehon-fork-<lane>`; `git worktree add` creates it), (b) switch to Mode B (drive from canonical via Junior; document the flip), or (c) proceed in `brehon-fork` for meta-edits only.

## Hard refusals

1. **Never `git checkout phase-v1-* / phase-m2-* / phase-*` inside `brehon-fork`** — destructive cross-lane operation, AND not durable across a `/compact` boundary (the working-tree checkout reverts to `governance-v0`; conversation state does not record it). Use the dedicated lane worktree. When validation needs a phase-branch tree in Mode B, the `validate-pending-laptop` handler creates a **throwaway worktree** (`git worktree add ../brehon-fork-validate-<id> origin/<branch>`), never a bare checkout — see `advisor-validation.md §"validate-pending-laptop handler"` Sequence step 1 + §"Why a worktree, not a checkout". 2026-06-07 m2-late-1 T1: bare checkout + compact = 8 failed steps against the wrong tree.
2. **Never write `.claude/decision-queue.json` from `brehon-fork`** for an entry belonging on a phase branch. The DQ on `governance-v0` holds only plan/brief-time entries (advisor planning DQs, clarify entries), not validate-pending or ci-watcher mutations.
3. **Never `git push --force` against another lane's branch** from any worktree. Per `no-destructive-defaults.md`.
4. **Never delete a worktree directory with `rm -rf`** — use `git worktree remove <path>` so `.git/worktrees/<name>/` admin state gets cleaned.
5. **Never share CC sessions across worktrees** — one session, one CWD, one lane. To switch: close session, open a new one in the target worktree.
6. **Atomic read-mutate-commit for any DQ write on the canonical checkout.** Legitimate `governance-v0` plan-time DQ writes STILL race concurrent CC sessions sharing the canonical `.git/`. Protocol: fetch → read fresh → recompute next_id → mutate+verify+add+commit+push as ONE uninterrupted shell sequence → verify entry survived post-push. Incident (v1-AD-e gate-1, `b114937b8` clobbered an uncommitted append) + full 5-step protocol + structural-fix scope: `.claude/refs/multi-lane-mechanics.md` §"Hard refusal #6 — atomic read-mutate-commit protocol".

## Worktree-aware DQ id discipline

Schema-v3 makes cross-lane next_id coordination obsolete. Each CC session generates its own 12-hex UUID prefix (`.claude/.dq-session-id`) + monotonic 3-digit sequence; namespaces never intersect. Generate via `bash scripts/brehon/dq-v3-new-entry.sh`. Pre-v3 entries retain integer `id` + gain `id_v1: <int>` alias. The `feedback_cohort_dq_id_collision.md` pre-reservation workaround is superseded. Post-v3 historical context: `.claude/refs/multi-lane-mechanics.md` §"Worktree-aware DQ id discipline".

## PMD is cross-lane shared, NOT per-lane isolated

The decision-queue is per-lane isolated (each worktree owns its own); the **PMD is the opposite** — lessons/retros/patterns are global, one canonical store `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` (absolute, never per-lane). Full invariant: `.claude/rules/pmd-invariants.md` §1. (Pointer only — avoids duplicating pmd-invariants.md #1.)

## Daemon side (EliteDesk)

The daemon at `/srv/brehon-fork` uses `git worktree add` per Junior task. Lane-isolation rule applies to the HUMAN-SIDE laptop checkout only. Detail: `.claude/refs/multi-lane-mechanics.md` §"Daemon side".

## See also

- `.claude/rules/branch-manager.md` "Session-start ritual"
- `.claude/rules/advisor-orchestrator.md` §1 "Polling loop"
- `.claude/rules/decision-queue.md` "Mid-task visibility"
- `feedback_multi_lane_worktree_discipline.md` — companion lesson with retro evidence + session-flow examples.
