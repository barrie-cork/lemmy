# Cohort DQ id collision — workers fork next_id from the same phase tip

When the advisor dispatches a parallel cohort under `[P]` markers, every worker forks from the same phase-branch tip simultaneously and each computes `next_id = max(all_ids) + 1` from its **isolated worktree view**. All workers see the same max id at fork time → all workers pick the same next_id → the cohort's DQ entries land with identical ids, and the advisor must renumber on consolidation.

**Why:** The cohort-dispatch protocol (`.claude/rules/advisor-orchestrator.md` §4.1) does pairwise FILES YAML disjointness check (so worker file-writes don't collide) but does NOT check the DQ id space. DQ ids are a **global cross-worker resource**, but the canonical next_id recipe (`.claude/refs/dq-recipes.md` §"Recipe 1") reads only the local worktree's view. Workers cannot see siblings' DQ writes (they haven't been pushed yet at the moment next_id is computed). The id-collision is determined the moment all workers fork.

**How to apply:**
- **Advisor pre-dispatch (preferred):** reserve `N` DQ ids per cohort BEFORE workers fork. Pre-write `N` empty stub entries to phase-branch's DQ pending[] (or a sidecar reserved-ids file) with `from: "advisor"`, `kind: "log"`, `answer: "reserved for cohort dispatch"`, mark as resolved. Each worker brief names its **assigned id explicitly** in §4 Constraints (e.g. "your DQ id MUST be 308"). Workers no longer compute next_id; they use the assigned id.
- **Recovery (post-collision):** advisor cherry-picks worker `feat()` commits onto phase branch, discards worker DQ-raise commits, authors one consolidated DQ commit with renumbered ids (e.g. phase_task order: 1→307, 2→308, 3→309, 4→310, 7→311). Mutate consolidated entries via single cargo-check (if FILES YAML disjoint + zero-Rust impact, one cargo invocation covers the cohort). Lesson body: per v1-rls-r1 Cohort A recovery at advisor commit `9dd80e025` (renumber) + `51fd76f00` (mutate).

**Incident:**
- v1-rls-r1 Cohort A (2026-05-21 07:18-08:08 UTC): 5 workers (#372 Task 1, #373 Task 2, #374 Task 3, #375 Task 4, #376 Task 7) all picked DQ id 307 (phase tip max was 306). Recovery cost: ~30 min advisor work (cherry-pick + consolidated DQ write + 3m 18s cargo-check). No data loss.
- Worker #372 (Task 1) raised the id-307 entry with `blocker`-shaped fields (`question`, `options`, `context`) because its pre-push cargo-check failed with exit 101 (lemmy_email build script "No such file or directory" — daemon worktree environment trap, not Task 1 scope). On canonical laptop the same cargo-check passed clean.

**See also:**
- `.claude/refs/dq-recipes.md` §"Recipe 1" — the canonical next_id read (worktree-local).
- `.claude/rules/advisor-orchestrator.md` §4.1 cohort dispatch — needs an "advisor pre-reserves N DQ ids" step.
- `.claude/rules/decision-queue.md` §"Mid-task visibility" + cross-archive next_id rule — neither covers cross-worktree-at-same-tip.
- `feedback_explicit_file_arrays_on_tasks.md` — the FILES YAML disjointness check that catches file-write collisions but not id-space collisions.
- `feedback_daemon_local_trunk_stale_multi_lane.md` — related: same-base-tip recurring family.
- `project_decision_queue_protocol.md` — DQ as async coordination channel.

**Fix status:** PENDING — see `.claude/PRPs/plans/<future-sub-phase>.plan.md` for the advisor-pre-reservation mechanism. Until then, advisors:
1. Either dispatch one cohort member at a time (collapse `[P]` to serial — kills the parallelism gain),
2. OR accept the recovery cost (~30 min) on `[P]` cohorts and budget for it,
3. OR (interim) advisor pre-authors `N` reserved DQ stubs in pending[] with explicit `answer: "reserved for v1-<phase> cohort <name>"` then names each id in each worker brief.

Option 3 is the cheapest tactical fix until a structural one ships.
