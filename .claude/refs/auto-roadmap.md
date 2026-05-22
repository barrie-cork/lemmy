# Auto-roadmap — orchestration rule for `/roadmap-next` + `/auto-roadmap`

This rule is referenced by `~/.claude/commands/roadmap-next.md` (skill 1,
canonical-checkout) and `~/.claude/commands/auto-roadmap.md` (skill 2,
lane worktree). The skills compose **on top of** `/auto-phase` to drive
multiple sub-phases of a v1 PRD lane to completion without re-deriving
"which sub-phase next?" at every boundary.

The rule exists because the skill pair mutates state across **two
worktrees** (canonical + lane-dedicated) and across **session
boundaries** (skill 1 runs in canonical CC, then the user manually opens
CC in the lane worktree and runs skill 2). Hard refusals + ownership
boundaries + handoff invariants live in-repo so they are loaded
on-demand at the start of every `/roadmap-next` and `/auto-roadmap`
invocation (per the skill bodies' Phase 0 Step 0).

> **Loading note (2026-05-22):** this file was relocated from
> `.claude/rules/` to `.claude/refs/` to free Memory-files budget — it is
> NO LONGER auto-loaded at session start (unlike `branch-manager.md`,
> `advisor-orchestrator.md`, `decision-queue.md`, `multi-lane-worktree.md`).
> The skill bodies `.claude/commands/roadmap-next.md` and
> `.claude/commands/auto-roadmap.md` Phase 0 Step 0 each Read this file
> before any routing decision; skill 2 additionally Reads
> `.claude/refs/auto-phase.md`. If invoking `/roadmap-next` or
> `/auto-roadmap` logic outside the skills (e.g. ad-hoc advisor-driven
> roadmap mutation), Read this file FIRST.

## Companion files

| File | Role |
|---|---|
| `~/.claude/commands/roadmap-next.md` | Skill 1 body — canonical-checkout, cuts lane worktree |
| `~/.claude/commands/auto-roadmap.md` | Skill 2 body — lane worktree, drives sub-phase via `/auto-phase` |
| `.claude/PRPs/v1-roadmap.json` | The roadmap skill 1 reads + both skills mutate |
| `.claude/PRPs/specs/auto-roadmap-skill-pair.md` | Feasibility spec (DRAFT 2026-05-22; ships alongside this rule) |
| `~/.claude/commands/auto-phase.md` | Single-sub-phase orchestrator skill 2 composes on top of |
| `.claude/refs/auto-phase.md` | Auto-phase state-machine rule (the source of truth skill 2 inherits) |
| `.claude/rules/multi-lane-worktree.md` | Worktree-per-lane discipline both skills honour |
| `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` | The 11-step bootstrap skill 1 walks |
| `.claude/commands/bm/bm-cut.md` | The BM verb skill 1 dispatches |

> **Mirror note:** like `auto-phase.md`, this rule may be mirrored at
> `homeserver/.claude/rules/auto-roadmap.md` if cross-machine resume
> ever becomes a use case. Today the skill pair runs only on the laptop;
> mirror is **not** in place. Surface a `diff` if it drifts.

## Hard refusals — skill 1 (`/roadmap-next`, canonical checkout)

The skill MUST refuse and STOP when any of these conditions hold. None
are auto-recoverable; user input is required.

1. **Wrong CWD.** Invocation from anything other than the canonical
   `C:/Users/barri/Developer/brehon-fork` checkout. Surface: "skill 1
   must run in canonical CWD; current CWD is X".
2. **Wrong branch.** Canonical CWD HEAD is anything other than
   `governance-v0`. Surface: "skill 1 requires governance-v0; current
   branch is X".
3. **Dirty working tree.** Per `multi-lane-worktree.md` Hard refusal #6
   atomic protocol — concurrent edits to `decision-queue.json` or
   `v1-roadmap.json` are the recurring race class. Surface
   `git status --short` and list files; do NOT auto-stash.
4. **Roadmap file missing or malformed.** Glob
   `.claude/PRPs/v1-roadmap.json`; refuse if not present or if
   `json.load` raises. Surface the file path + parse error verbatim.
5. **No eligible unstarted sub-phase.** Roadmap's
   `what_remains.high_priority_unstarted` is empty AND every lane is
   `done | skipped | in_flight`. Surface: "no eligible sub-phase to
   recommend; check roadmap or close in-flight lanes first".
6. **Recommended sub-phase already in flight.** A worktree at the
   target path already exists (`git worktree list` shows
   `../brehon-fork-<lane-suffix>`). Surface the existing worktree path;
   require user to either `git worktree remove` it or pick a different
   sub-phase.
7. **`bm-cut` Junior task fails.** Skill 1 dispatches `bm-cut` via
   the `branch-manager` subagent (haiku). On non-zero exit, do NOT
   improvise an alternate cut path. Surface the BM output verbatim.
8. **Bootstrap-checklist step verification fails.** The 11-step
   checklist at `feedback_phase_lane_worktree_bootstrap_checklist.md`
   includes programmatic verification at steps 8 + 10 + 11. On any
   FAIL, do NOT proceed with the roadmap-mutate-and-push step.
   Surface the failing verification + path of the file that needs
   re-application.
9. **`gh pr` or `git push` failure** when pushing
   `phase-<sub-phase>` to origin. Surface the gh/git output. The
   roadmap MUST NOT flip to `in_flight` if the push didn't land —
   future skill 2 invocations rely on the branch being visible on
   origin.
10. **Atomic-protocol race** during the roadmap update. Per
    `multi-lane-worktree.md` Hard refusal #6: between the `git fetch`
    and the `git push`, if another session commits to
    `governance-v0`, `git push` fails non-fast-forward. Re-fetch, re-
    read roadmap, re-mutate, re-commit, re-push (up to 3 attempts).
    On 3rd failure, surface to user — never `--force`.

## Hard refusals — skill 2 (`/auto-roadmap`, lane worktree)

11. **Wrong CWD.** Invocation from canonical `brehon-fork` or any
    non-lane-dedicated worktree. Surface: "skill 2 must run in a
    lane-dedicated worktree (`brehon-fork-<lane>`); current CWD is X.
    Use `/roadmap-next` from canonical to cut a lane first."
12. **Wrong branch.** Lane worktree HEAD is `governance-v0`, `main`,
    or any non-`phase-v*-*` branch. Surface: "skill 2 requires a
    `phase-v*-*` branch; current branch is X".
13. **Roadmap entry missing or wrong status.** Lane branch is
    `phase-v1-RT-r2` but roadmap's `lanes.RT.sub_phases.v1-RT-r2`
    either doesn't exist or has `status != "in_flight"`. Refuse —
    skill 1 was supposed to flip it.
14. **Sub-phase already merged** (status `done` in roadmap AND PR
    closed-merged). Refuse and point at `/brehon-phase-transition`
    or `/roadmap-next`.
15. **`/auto-phase` skill body missing.** Skill 2 invokes
    `/auto-phase` via Skill tool. If the user-scope file
    `~/.claude/commands/auto-phase.md` is absent, refuse — skill 2
    is dependent on `/auto-phase` per design.
16. **Plan-gap AND PRD-section-missing.** If
    `.claude/PRPs/plans/<sub-phase>.plan.md` is missing AND the PRD
    for the lane (e.g. `v1-reputation-tuning.prd.md`) has zero
    sections naming the sub-phase scope, refuse with: "plan missing
    and PRD has no scope section for this sub-phase. Author one
    manually and resume." Plan-gap auto-dispatch requires SOME
    scope source.
17. **Plan-gap auto-dispatch produces malformed brief.** Skill 2's
    Phase 0.5 auto-authors a planning brief. Before dispatching
    Junior, the skill MUST surface the brief to the user via
    AskUserQuestion (one extra gate per plan-gap case). The user
    confirms, edits inline (`Other` text), or aborts. Auto-dispatch
    without user confirmation is a hard refusal.
18. **`/auto-phase` returns with state `catch-fire`.** Skill 2
    propagates the catch-fire — it does NOT improvise recovery.
    User must intervene per the catch-fire's surfaced reason; on
    resume, the user re-runs `/auto-phase <sub-phase>` directly
    (not `/auto-roadmap`) to take advantage of `/auto-phase`'s
    `--start-from` flag and its existing catch-fire-recovery
    semantics.
19. **Pre-seeded auto-state JSON conflicts with existing file.** If
    `.claude/auto-state/<sub-phase>.json` already exists with a
    `stage` other than `init`, refuse — a prior `/auto-phase`
    invocation is in-progress on this sub-phase and the pre-seed
    would clobber its state. Surface the existing state file's
    `stage` + `last_action_at`.
20. **Roadmap update at sub-phase completion** — same atomic-protocol
    discipline as skill 1 hard refusal #10 (3-attempt retry on
    non-fast-forward, never `--force`). The lane worktree shares
    `.git/` with canonical, so the same race class applies.

## Skill ownership boundaries

### Skill 1 (`/roadmap-next`) WRITES

- `.claude/PRPs/v1-roadmap.json` — flips one sub-phase entry from
  `status: "unstarted"` → `status: "in_flight"`; adds `worktree` field;
  bumps roadmap-level `last_updated_at`. **Forward-only**; never edits
  other sub-phases.
- `.claude/runlog/bm-runlog.md` — appends one line
  `## advisor: roadmap-next cut phase-<X>` (mechanical audit trail;
  mirrors `auto-phase.md` Phase 4 runlog discipline).
- (transitively via `bm-cut`) creates the `phase-<sub-phase>` branch
  locally + pushes to origin via the branch-manager subagent.
- (transitively) creates the lane worktree directory at
  `../brehon-fork-<lane-suffix>` and the per-worktree files the
  bootstrap checklist requires (`.mcp.json`, `.claude/settings.local.json`
  hook wiring).
- Commits to `governance-v0` with subject:
  `chore(advisor): roadmap-next flip <sub-phase> in_flight + cut lane`
  per `decision-queue.md` "Attribution integrity §Detection".

### Skill 1 NEVER WRITES

- `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**` — advisor never authors content
  per the four-role model. Skill 1 is meta-orchestration only.
- `.claude/PRPs/plans/*.plan.md` — plans are authored by the planning
  Junior subagent, not by the advisor.
- `.claude/PRPs/prds/*.prd.md` — PRDs are user-authored or pre-existing.
- Any file under any other lane's worktree (`brehon-fork-<other>`) —
  cross-lane writes from canonical are a hard refusal.
- The `/auto-phase` skill body or rule (preserved verbatim per spec).
- `.claude/decision-queue.json` — skill 1 does NOT raise DQ entries.
  Its only failure mode is "refuse and surface to user". DQ-raising is
  reserved for skill 2's plan-gap clarify cycle (which dispatches
  `/brehon-clarify`).

### Skill 2 (`/auto-roadmap`) WRITES

- `.claude/PRPs/v1-roadmap.json` — at sub-phase completion, flips the
  entry from `status: "in_flight"` → `status: "done"`; populates `pr`,
  `merge_commit`, `retro`, `verify` from the auto-state JSON. Same
  atomic-protocol discipline as skill 1.
- `.claude/PRPs/briefs/<sub-phase>-planning-1.md` — when plan-gap
  fires, auto-authors the planning brief from PRD + entry-kind registry.
  Brief commits to the lane branch (per
  `advisor-orchestrator.md` §2.1 — impl/planning briefs land on the
  phase branch the worker forks from).
- `.claude/auto-state/<sub-phase>.json` — pre-seeds the auto-state JSON
  with `stage: "impl-cohort-1"` (or `stage: "planning-running"` if
  plan-gap dispatch is in flight, which transitions to
  `planning-approved-pending-user` via `/auto-phase`'s normal path).
- `.claude/runlog/bm-runlog.md` — appends one line per stage transition
  (delegated to `/auto-phase`'s own runlog discipline once skill 2
  hands off).
- Commits to `governance-v0` (the roadmap update at sub-phase end)
  via the lane worktree's shared `.git/`. Subject pattern:
  `chore(advisor): auto-roadmap flip <sub-phase> done — PR #<N>`.
- Commits to the lane branch (the planning brief when plan-gap fires).
  Subject pattern: `chore(advisor): auto-roadmap auto-author planning
  brief for <sub-phase>` per `advisor-orchestrator.md` §3.3 clarify
  gate (the brief commits before `/brehon-clarify` runs).

### Skill 2 NEVER WRITES

- `crates/**`, `migrations/**`, `tests/**`, `docs/`. Same four-role
  posture as skill 1.
- `.claude/PRPs/plans/*.plan.md` — produced by the planning Junior
  subagent that skill 2 may dispatch, not by skill 2 itself.
- `.claude/PRPs/prds/*.prd.md` — read-only.
- The `/auto-phase` skill body, rule, or template — preserved verbatim.
- `.claude/decision-queue.json` directly — except via `/brehon-clarify`
  which writes `kind: "clarify"` entries on skill 2's behalf during the
  plan-gap handler.
- `.claude/auto-state/<other-phase>.json` — skill 2 only mutates its
  own sub-phase's state file.

## State-routing invariants

The skill pair's state machine is described in the skill bodies. The
rule below codifies what may not change:

1. **Six mandatory `/auto-phase` user gates remain inviolate.** Skill 2
   inherits all six gates from `/auto-phase`. Skill 2 adds **two extra
   gates** of its own (per the spec §3.2 risk table):
   - **Gate 0:** skill 1's "confirm roadmap recommendation"
     AskUserQuestion before bm-cut dispatch.
   - **Gate X (conditional):** skill 2's plan-gap "approve auto-authored
     planning brief" AskUserQuestion before planning-Junior dispatch.
     Skipped if plan exists.
   Pre-seeding any gate answer (the `--auto-all-gates` mode rejected
   for `/auto-phase`) is a hard refusal for the skill pair too.

2. **Catch-fire propagates from `/auto-phase`.** Skill 2 does not
   wrap, swallow, or re-classify `/auto-phase` catch-fires. The
   catch-fire surfaces to the user with `/auto-phase`'s original
   reason + recovery instructions. Skill 2 itself goes to a terminal
   `catch-fire-propagated` stage and exits.

3. **Lane-dedicated worktree is the unit of skill 2 lifecycle.** Skill
   2 runs in `brehon-fork-<lane>`, drives one sub-phase, and exits.
   It does NOT loop into the next sub-phase. Looping would require
   keeping the worktree open across sub-phases, conflicting with
   `multi-lane-worktree.md` §"Lifecycle" step 3 (worktree pruned
   after merge).

4. **DQ attribution stays advisor-side.** Both skills run in the
   persistent advisor session (skill 1 in canonical CC; skill 2 in
   lane-worktree CC — both are advisor sessions, just different
   CWDs). They write `answered_by: "advisor"` or relay user replies
   as `answered_by: "user"`. They MUST NEVER write `answered_by`
   under any other label. Commit subjects MUST match
   `^(chore|docs)\((advisor|decision-queue)\)` per
   `decision-queue.md` Attribution integrity §Detection.

5. **L15 / L14 / L16 fixes inherit through `/auto-phase`.** Skill 2
   does not duplicate the bm-merge gate-then-execute split, the
   runlog re-apply fallback, or the branch-deletion verification —
   those live inside `/auto-phase` and execute when skill 2 hands
   off. Skill 2 only orchestrates the wrapper around `/auto-phase`.

6. **Hand-off between skills is via user-typed `/auto-roadmap`.** No
   automatic continuation. Skill 1 prints "open CC in the new
   worktree and run `/auto-roadmap`"; the user does so manually. The
   manual hand-off step is the natural boundary where the user can
   verify the lane worktree opened cleanly (SessionStart hooks fired
   without WARN, no stranded state from a prior session) before any
   state-changing action begins.

7. **PMD canonical-path discipline is enforced at bootstrap step 5
   + verified at SessionStart.** Skill 1's bootstrap walk copies
   `.mcp.json.example` → `.mcp.json` verbatim (the example carries
   the canonical absolute `PROJECT_MEMORY_DB`). The `pmd-canonical-
   guard.sh` SessionStart hook wired at step 6 then catches drift
   on every future session in the lane. Skill 1's verification at
   step 8 + 10 + 11 fails the bootstrap before the roadmap flips
   to `in_flight`.

8. **Roadmap schema versions monotonically advance.** When the
   roadmap schema changes (e.g. adding the `worktree` field — `v1
   → v2`), the bump is performed by a separate `chore(advisor):
   roadmap schema bump v1 → vN` commit, NOT silently inside a
   `/roadmap-next` flip. Skill 1 refuses if the roadmap schema
   version is unknown to the skill (current skill expects
   `$schema_version` in `{1, 2}`).

## Cadence invariants

The skill pair is **interactive at the gate boundaries** and inherits
`/auto-phase`'s cadence schedule for the in-skill-2 work. Two specific
notes:

1. **Skill 1 is synchronous from user-typing to bm-cut-completion**
   (~2 min wall-clock end-to-end for the cut + bootstrap walk + push).
   No `ScheduleWakeup` cadences fire inside skill 1.

2. **Skill 2's pre-`/auto-phase` work is also synchronous** (plan-gap
   brief auto-authoring + clarify-gate + planning Junior dispatch
   takes ~30-90 min wall-clock; user gates fire as AskUserQuestion).
   Once `/auto-phase` is invoked, its own cadence schedule takes over
   (per `auto-phase.md` Phase 3). Skill 2 wakes up when `/auto-phase`
   returns (terminal stage or catch-fire).

## Resume semantics

### Skill 1 resume

Skill 1 has no resume path. It's a one-shot: read roadmap → recommend
→ confirm → cut → bootstrap → flip → exit. If a session is interrupted
mid-skill-1:

- **Before bm-cut dispatch:** no state mutated; re-run `/roadmap-next`
  from scratch.
- **After bm-cut succeeds but before bootstrap completes:** the lane
  branch exists on origin but the worktree directory may be partial.
  Skill 1's hard refusal #6 (worktree already exists) will fire on
  re-invocation — user must manually `git worktree remove` (if
  partial) or `git worktree add` (if missing) and re-run the
  bootstrap-checklist walk manually, then re-run `/roadmap-next` to
  flip the roadmap.
- **After bootstrap completes but before roadmap flip:** the worktree
  is fully set up but the roadmap still says `unstarted`. Skill 1's
  hard refusal #6 fires on re-invocation. User edits the roadmap
  manually (`status: "in_flight"`, add `worktree` field), commits,
  pushes, then runs `/auto-roadmap` in the new worktree directly.

This is acceptable because skill 1's mid-action interrupt window is
~2 minutes total — a user is unlikely to compact mid-skill-1.

### Skill 2 resume

Skill 2 itself has minimal resume — most of its work is delegated to
`/auto-phase`, whose Phase 0.5 resume reconciliation handles the
mid-sub-phase resume case. Skill 2's own resume path is:

- **Plan-gap brief authored, planning Junior not yet dispatched:** the
  brief commit on the lane branch is the durable signal. Re-running
  `/auto-roadmap` detects the brief exists, skips the auto-authoring
  step, proceeds to clarify gate.
- **Planning Junior dispatched, awaiting completion:** the lane's
  `.claude/decision-queue.json` may have clarify entries; the auto-state
  JSON is not yet written (skill 2 writes it only after planning
  approval). Skill 2's resume reads pending clarify entries; if any
  are unresolved, surfaces via AskUserQuestion or self-answers per
  the clarify gate.
- **Planning approved, auto-state JSON pre-seeded:** `/auto-phase` is
  in flight. Skill 2 is effectively idle until `/auto-phase` returns.
  Re-running `/auto-roadmap` detects the auto-state JSON exists at
  `stage: "impl-cohort-1"` (or later), refuses with hard refusal #19,
  and points the user at running `/auto-phase <sub-phase>` directly.
- **`/auto-phase` returned `done`:** the auto-state JSON shows
  `stage: "done"`. Skill 2 reads the JSON, mutates the roadmap, commits,
  pushes, prints the next-sub-phase hint, exits.

### What does NOT survive across session boundaries

- **Conversation context** — both skills MUST print self-contained
  output that can be acted on by a future session with no memory of
  the prior session's reasoning. Skill 1's "next steps" output is
  the canonical handoff for skill 2; skill 2's "next sub-phase" output
  is the canonical handoff back to skill 1.

- **Background processes** — neither skill spawns long-running
  background tasks. The planning Junior is a Junior task (lives on
  the EliteDesk daemon, survives session boundaries naturally).
  `/auto-phase`'s own background-cargo invocations are owned by
  `/auto-phase`.

- **AskUserQuestion answers** — if a session ends mid-gate, the next
  session re-fires the gate from durable state (roadmap entry status,
  auto-state JSON stage, planning brief existence). The user does
  not lose their place in the chain.

## What this rule does NOT cover

- The skills' tick procedures (live in `~/.claude/commands/roadmap-
  next.md` and `~/.claude/commands/auto-roadmap.md`).
- `/auto-phase` state-machine semantics (live in
  `.claude/refs/auto-phase.md` and `~/.claude/commands/auto-phase.md`).
- BM verb scripts (`/bm-cut` reused unchanged from
  `.claude/commands/bm/bm-cut.md`).
- DQ schema (lives in `.claude/rules/decision-queue.md`).
- Multi-lane worktree mechanics (lives in
  `.claude/rules/multi-lane-worktree.md`).
- Bootstrap checklist (lives in
  `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md`).
- Forbidden execution windows (canonical list in
  `advisor-orchestrator.md`; both skills inherit via `/auto-phase`).
- Brehon four-role model (canonical in CLAUDE.md; both skills
  respect but do not codify).

## See also

- `~/.claude/commands/roadmap-next.md` — skill 1 body
- `~/.claude/commands/auto-roadmap.md` — skill 2 body
- `~/.claude/commands/auto-phase.md` — the skill skill 2 composes on
- `.claude/refs/auto-phase.md` — state-machine source skill 2 inherits
- `.claude/rules/advisor-orchestrator.md` — clarify gate + DoD smoke
  + watchpoint gate + cohort dispatch the pair honours
- `.claude/rules/branch-manager.md` — BM file-ownership skill 1 honours
- `.claude/rules/decision-queue.md` — attribution + atomic protocol
- `.claude/rules/multi-lane-worktree.md` — worktree-per-lane discipline
- `.claude/PRPs/v1-roadmap.json` — the file both skills mutate
- `.claude/PRPs/specs/auto-roadmap-skill-pair.md` — the feasibility
  spec authoring these skills + the rule
