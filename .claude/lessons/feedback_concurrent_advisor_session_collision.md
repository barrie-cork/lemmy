---
name: Concurrent advisor sessions in shared checkout cause bundled-intent commits
description: Two advisor sessions writing the same brehon-fork checkout simultaneously will race. Triggers like "N lanes since X" are permission to migrate, not motivation. Structural migrations need a dedicated session + concrete blocker.
type: feedback
---

# Concurrent advisor sessions in a shared checkout cause bundled-intent commits

When two Claude Code advisor sessions operate on the same on-disk
checkout (`C:/Users/barri/Developer/brehon-fork`), even with
different mental scopes, their `git add` calls share staging area.
A `git commit` in one session sweeps up files staged by the other.
The resulting commit subject reflects only one session's intent
while the diff covers both — auditing later sees mismatched scope.

`.claude/rules/multi-lane-worktree.md` exists to prevent this via
per-lane worktrees. But the rule applies to **phase-branch work**
(separate worktrees per active `phase-v1-*`). For
**governance-v0 meta-work** (rules, lessons, templates, retro-watch-
items) both sessions land on the canonical `brehon-fork` checkout
by design — no per-lane worktree exists for "meta". That's the
gap. Two governance-v0 sessions racing is unprotected.

**Case study — 2026-05-13:**

Session A (RT-r1 retro watch-item promotion) was authoring lessons
+ template edits + bm-pr.md gate additions. Session B (collapse-
Junior migration) was authoring a delta plan + executing Phase B
(drop Junior artifacts).

Both sessions had `git add`'d files when Session B ran `git commit
-m "docs(meta): re-author collapse-Junior plan against current
HEAD"`. Session B's commit picked up Session A's three staged
files:

- `.claude/lessons/feedback_phase_2_e2e_gate_enforcement.md` (new — A)
- `.claude/commands/bm/bm-pr.md` (+48 lines, Phase 1c gate — A)
- `.claude/PRPs/reports/v1-RT-r1-retro.md` (1-line `[x]` mark — A)
- `.claude/PRPs/plans/collapse-junior-2026-05-13.delta.md` (new — B)

The resulting commit `b004856df` reports as a `docs(meta):` collapse
plan re-author, but ~half its diff is `feat(lessons,bm-pr)` watch-
item-3 closure. The commit subject does not reflect the diff.

Session B then proceeded with Phases C + D
(`bcaa782d9`, `b648c11c2`) — agent rewrites + citation trimming —
which deleted 13 rules including `multi-lane-worktree.md` itself.
Session A's next `git fetch` saw the deletion and ~132 citation
paths broken across `.claude/`. User intervened, Session B reverted
B + C + D (`5c0bac2d4`, `4aed5a51e`, `084300600`), kept `b004856df`
as historical record + retroactive audit-trail commit `95f341fbb`
clarified the bundled-attribution.

Total wallclock cost: ~3 hours of work bundled into wrong-attribution
commits, then reverted, plus 1-2 hours of session B audit + revert
+ Session A audit-trail cleanup.

**Two distinct lessons in one incident:**

### Lesson 1 — bundled-intent commits

A session's `git commit` operates on the staging area shared with
every other process touching the same `.git/index`. If another
session has staged work, your commit will include it. The commit
message will not reflect this.

**How to apply:**

- Before any state-changing `git commit` in a multi-session
  context, run `git status` AND verify the staged-file list matches
  your session's intent. If you see files you didn't `git add`,
  STOP. Either unstage them
  (`git restore --staged <path>`) or coordinate with the other
  session.
- After `git commit`, immediately read `git show HEAD --stat` and
  confirm the changed-file list matches your commit subject. Surface
  any mismatch to the user before pushing.
- If a bundled commit has already been pushed, do NOT amend or force-
  push to fix attribution. Land a `docs(attribution):` follow-up
  commit clarifying intent, file-by-file, citing the bundled commit
  SHA. Per `.claude/rules/decision-queue.md` Attribution-integrity
  §Detection: empty-content audit-trail commits are the canonical
  recovery.

### Lesson 2 — trigger staleness is not motivation

The collapse-Junior plan listed a trigger ("SL-b shipped + 4 lanes
since"). The trigger was satisfied on 2026-05-05 and grew progressively
more stale over 5 subsequent lanes (SL-c-1, SL-c-2, SL-d, SL-e, RT-r1).
On 2026-05-13 a session interpreted "trigger satisfied + 5 lanes stale"
as motivation to execute the migration.

It was not. The trigger was **permission to migrate when motivated**,
not a requirement. The actual motivation needed was either:

(a) A concrete next-phase blocker that the migration resolves
    (e.g. "Junior daemon's EliteDesk RAM ceiling blocks RT-r2's
    new e2e suite"), OR
(b) A user-stated change in priorities (e.g. "I want to ship Junior
    decommission this week").

Neither held on 2026-05-13. The Junior model had just carried 5
lanes through to retro with no concrete cost. Migrating in that
state put the codebase in a 132-citation-broken-link window
solely to satisfy a staleness count.

**How to apply:**

- Migration triggers in plan documents are **permission gates**,
  not deadlines. Treat them like feature flags: trigger satisfied
  unlocks the migration; concrete motivation gates the execution.
- Before executing a planned migration, write a 2-sentence
  "concrete motivation" note. If the note reduces to "the trigger
  is satisfied", STOP and ask the user whether the migration is
  still wanted. Triggers age out of relevance; motivations don't.
- When a planning artifact carries staleness counts (e.g. "deferred
  from SL-c-2; surface when SL-c ships"), include an explicit
  invalidation condition: "Re-evaluate at trigger time; deferred
  indefinitely if no concrete blocker exists by Lane+3."

### Lesson 3 — structural migrations need a dedicated session

The collapse-Junior plan had 9 phases (A through I). Executing
piecemeal across a concurrent-session window guaranteed broken
intermediate states. The right shape for a structural migration is:

- **One session, exclusively** for the duration of the migration.
- **No parallel writers** to the same checkout — neither human-side
  edits, nor Junior tasks dispatched against `governance-v0`, nor
  another advisor session doing meta-work.
- **Citation fixes in the same commit as the deletion** that breaks
  them — never as a follow-up phase. The 132-broken-link window is
  a structural hazard, not a paperwork item.
- **Validation gate at each phase boundary** — after each
  destructive step, `rg` for stale citations + verify no broken
  links + confirm the workspace still passes `cargo check`.

**How to apply:**

- Surface to user before starting any migration with phase count > 2:
  "this is a dedicated-session migration; do you want to pause
  parallel session work for ~N hours?"
- If user agrees, write `.claude/agent-activity.json` with
  `mode: "exclusive-migration"` + `phase: "collapse-junior"` so any
  other session opened during the window sees the lock and surfaces
  a warning at session-start.
- Treat the lock as a refusable invariant: another session that
  ignores it and writes anyway is a process breach (retro flags it).

**Companion lessons:**

- `feedback_multi_lane_worktree_discipline.md` — the existing rule
  that protects phase-branch work; this lesson extends to
  governance-v0 meta-work.
- `feedback_parallel_agents_one_worktree_per_agent.md` — Junior-side
  isolation pattern; same principle, different surface.
- `feedback_principles_not_rules.md` — triggers as permission gates,
  not deadlines.
- `feedback_advisor_instruction_mismatch_stop_and_ask.md` — when
  signals conflict, stop and ask rather than improvise.

**Where codified:**

- `.claude/PRPs/reports/v1-RT-r1-retro.md` §5 watch-item 7 (resolution
  text references this lesson).
- `.claude/PRPs/plans/collapse-junior-2026-05-13.delta.md` — historical
  record of the failed migration; future plans should cite this case
  study before executing similar.
- This lesson file.

**Detection (retro):** any `docs(meta):` or `chore(meta):` commit
whose `git show --stat` includes files outside `.claude/PRPs/plans/`,
`.claude/PRPs/handovers/`, or `.claude/runlog/` is candidate for
bundled-attribution review. Retro flags it; `docs(attribution):`
follow-up is the canonical fix.
