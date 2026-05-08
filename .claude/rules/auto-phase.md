# Auto-phase — orchestration state machine for `/auto-phase`

This rule is referenced by `~/.claude/commands/auto-phase.md` (user-scope
skill). The skill compiles `.claude/rules/advisor-orchestrator.md`
§"Stage-shape orchestration" + §"Cohort dispatch sequence" + §"§G4
classifier" into a deterministic state machine that drives a sub-phase
end-to-end with calibrated `ScheduleWakeup` cadences.

The rule exists because `/auto-phase` is the first user-scope skill that
mutates state machine progress across multiple advisor session restarts
(the auto-state JSON survives restart). Its hard refusals + state-routing
rules need to live in-repo so they are read at session start (alongside
`branch-manager.md`, `advisor-orchestrator.md`, `decision-queue.md`) by
any advisor session that resumes a `/auto-phase` invocation.

> **Mirror note:** like `advisor-orchestrator.md`, this rule may be
> mirrored at `homeserver/.claude/rules/auto-phase.md` if cross-machine
> resume becomes a use case. Today the skill runs only on the laptop
> advisor session; mirror is **not** in place. Surface a `diff` if it
> drifts.

## Companion files

| File | Role |
|---|---|
| `~/.claude/commands/auto-phase.md` | The skill body (user-scope, dispatched by `/auto-phase`) |
| `.claude/PRPs/templates/auto-phase-state.template.json` | Runtime state schema |
| `.claude/auto-state/<phase>.json` | Per-phase runtime state (gitignored — `.gitignore` line `.claude/auto-state/`) |
| `.claude/rules/advisor-orchestrator.md` | Canonical stage-shape source (read-only — `/auto-phase` does NOT supersede it) |
| `.claude/rules/branch-manager.md` | File-ownership boundaries the skill must respect |
| `.claude/commands/bm/bm-merge.md` | Split gate (advisor inline) from execute (Junior) per L15 |

## Hard refusals

The skill MUST refuse and STOP when any of these conditions hold. None
are auto-recoverable; user input is required.

1. **Wrong branch.** Invocation from anything other than `governance-v0`.
   Surface: "auto-phase requires governance-v0; current branch is X".
2. **Dirty working tree.** Surface the `git status --short` output and
   list files; suggest commit/stash.
3. **Plan file missing** for the named phase. Glob
   `.claude/PRPs/plans/<phase>*.plan.md`; refuse if zero or >1 match.
4. **Sub-phase already merged** (PR closed-merged). Refuse and point at
   `/brehon-phase-transition`.
5. **Concurrent advisor session** writing the same phase (per
   `.claude/agent-activity.json` `mode: write`). Refuse — user must
   close the other session.
6. **`--start-from <stage>`** with non-canonical stage name. Refuse and
   list valid stages from the auto-state schema enum.
7. **`--no-bm-cut`** without a matching `phase-<phase>` branch in the
   git worktree list. Refuse and drop the flag (or run normally).
8. **Forbidden window** at the precise moment of a state-changing
   action. NOT a refusal — defer per advisor-orchestrator.md and
   `ScheduleWakeup` to the end of the window with the `prompt`
   carrying the same `/auto-phase` invocation.
9. **`gh pr merge` exit non-zero** during `merge-executing` stage.
   Surface verbatim error; do NOT retry; do NOT improvise an alternate
   merge strategy. (Mirrors bm-merge.md hard refusal.)
10. **BM Junior breach** detected post-task (writes to `crates/`,
    `migrations/`, `tests/`, `docs/brehon-law-inspired-network/` per
    `.claude/rules/branch-manager.md` "File ownership boundaries"). The
    skill catch-fires and halts; user must surface the rule citation
    to the breached subagent's task output.

## Skill ownership boundaries

The skill **WRITES**:

- `.claude/auto-state/<phase>.json` — per-phase runtime state.
- `.claude/auto-state/<phase>-catchfire-<ts>.md` — catch-fire dumps.
- `.claude/PRPs/briefs/<phase>-<role>-<n>.md` — every Junior brief it
  authors (mechanical from templates; same as the manual-orchestration
  pattern).
- `.claude/decision-queue.json` — DQ writes for Phase 2 e2e dispatch
  (`from: "advisor"`, `kind: "validate-pending"`); DQ answers when the
  skill self-resolves a routine entry. Commit subjects MUST match
  `^(chore|docs)\((advisor|decision-queue)\)` per
  `.claude/rules/decision-queue.md` "Attribution integrity §Detection".
- `.claude/runlog/bm-runlog.md` — one-line `## advisor: auto-phase
  …` entries on stage transitions for the durable audit trail (also
  appends `docs(advisor)` blocks for the L14 belt-and-braces fallback
  if a BM Junior skips the runlog commit).
- The advisor's own commit subjects MUST follow attribution-integrity
  patterns (e.g. `chore(advisor): ` for state-machine transitions,
  `docs(advisor): ` for L14 retro re-applies).

The skill **NEVER WRITES**:

- `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**` (advisor never authors content
  per the four-role model).
- `.claude/PRPs/plans/*.plan.md` after the planning Junior committed
  it (plan revisions go through a planner re-run via DQ, never
  advisor-side edits).
- `.claude/PRPs/reviews/pr-*-findings.yaml` (BM-owned).
- The catch-fire path is the ONLY path that writes a `.md` file under
  `.claude/auto-state/` — and that file is gitignored runtime state,
  not a tracked artifact.

## State-routing invariants

The state machine routes per the table in `~/.claude/commands/auto-phase.md`
§"Phase 2 — Stage routing table". The rule below codifies what may not
change:

1. **Six user gates are non-skippable.** Any state listed as
   `*-pending-user` MUST resolve through `AskUserQuestion`, not through
   advisor self-decision, regardless of the skill's confidence in the
   answer. Pre-seeding gate answers (the rejected `--auto-all-gates`
   mode) is a hard refusal.

2. **Catch-fire is terminal.** Once `stage = catch-fire`, the only valid
   user transitions are: (a) explicit user resume after addressing the
   reason, advancing the auto-state JSON to a sane prior stage; or (b)
   abandon the phase. The skill MUST NOT auto-recover by retrying the
   failing transition.

3. **Cohort barriers are atomic.** The skill MUST NOT advance from
   `impl-cohort-N-validating` to `impl-cohort-N+1` (or `bm-pr-pending`)
   until ALL members of cohort N reach `result: pass` on their
   `validate-pending` DQ entries. Partial pass = wait or catch-fire.

4. **Phase 2 e2e gate fires once per phase.** The first
   `phase-2-e2e-N` stage MUST fire `AskUserQuestion`; subsequent
   `phase-2-e2e-M` (M>N) stages MUST honor the cached choice in
   `auto_state.phase_2_e2e_mode`. Re-asking is a UX regression
   (re-introduces the per-transition friction the skill exists to
   eliminate).

5. **DQ attribution stays advisor-side.** The skill is part of the
   advisor session, so its commits write `answered_by: "advisor"` or
   relay user replies as `answered_by: "user"`. It MUST NEVER write
   `answered_by` under any other label, and its commit subject MUST
   match `^(chore|docs)\((advisor|decision-queue)\)` per
   `.claude/rules/decision-queue.md` Attribution integrity §Detection.

6. **L15 fix is load-bearing.** The merge-gate's read-only checks run
   inline in the advisor session, NOT as a Junior task. The skill
   queues a Junior `[role:bm-task]` ONLY post-confirm for the actual
   `gh pr merge` execution AND the `chore(bm)` runlog commit. Any
   change that re-introduces a pre-confirm Junior dispatch is a
   process regression and must be surfaced.

7. **L14 fix is belt-and-braces.** The bm-merge brief explicitly
   orders `Edit runlog → git add → git commit -m chore(bm):...
   → git push origin governance-v0 → THEN gh pr merge`. The skill's
   post-merge tick scans `git log -3 governance-v0` for the
   `chore(bm)` matching the merge timeframe; if missing, advisor
   authors a `docs(advisor):` re-apply block. This fallback is NOT
   auto-skipped; if the BM brief's git sequence is silent, the post-
   merge re-apply MUST run.

8. **L16 fix is post-condition.** After `gh pr merge` returns, the
   skill verifies branch deletion via `git ls-remote origin
   refs/heads/<phase-branch>`. If still present (silent-skip case),
   advisor runs `gh api -X DELETE -H "Accept:
   application/vnd.github+json"
   /repos/barrie-cork/lemmy/git/refs/heads/<phase-branch>`. Single
   attempt; on second failure, surface to user.

## Cadence invariants

The skill's `ScheduleWakeup` cadences are derived from c-1 retro
wall-clock evidence (per
`.claude/PRPs/reports/v1-SL-c-1-retro.md`). Two invariants:

1. **No 300s sleeps.** 300s is the worst-of-both: pays the cache miss
   without amortising. The skill chooses 270s (cache-warm) or ≥1200s
   (one cache miss across long wait). This rule does NOT change with
   plan complexity — it's a property of the prompt-cache TTL.

2. **Cadence respects stage.** Long-running stages (planning ≥30 min,
   e2e ~26 min) start with a one-shot ≥1200s sleep, then drop to 270s
   after the expected mid-point. Short-running stages
   (`bm-cut-running` ~2 min, `bm-merge-executing` ~1 min) use 60-120s.
   The skill MUST NOT override this without recording wall-clock
   evidence in a future retro that justifies the change.

## Resume semantics

Auto-state JSON is per-phase. On `/auto-phase v1-SL-c-2` invocation:

1. If `.claude/auto-state/v1-SL-c-2.json` does NOT exist → fresh init,
   stage = `init`, run Phase 0 prerequisites.
2. If file exists with `stage = done` → refuse: "phase already complete;
   re-running would re-trigger /brehon-phase-transition". Suggest the
   user delete the state file or run a new phase.
3. If file exists with `stage = catch-fire` → surface the catch-fire
   dump path; refuse to auto-resume; require explicit user instruction
   ("/auto-phase v1-SL-c-2 --start-from <stage>" after fixing the
   underlying issue).
4. Otherwise → resume from current stage. Print resume summary; route
   to current stage's tick handler.

## What this rule does NOT cover

- The skill's tick procedure body (lives in
  `~/.claude/commands/auto-phase.md` Phase 1).
- Stage-shape semantics (live in `advisor-orchestrator.md`; the skill
  reads but does not replace).
- BM verb scripts (the skill dispatches them unchanged from
  `.claude/commands/bm/*.md`).
- DQ schema (lives in `.claude/rules/decision-queue.md`).
- Forbidden execution windows (canonical list in
  `advisor-orchestrator.md`; skill respects but does not duplicate).
- Brehon four-role model (canonical in CLAUDE.md; skill respects but
  does not codify).

## See also

- `~/.claude/commands/auto-phase.md` — skill body, dispatched by
  `/auto-phase` slash command.
- `.claude/rules/advisor-orchestrator.md` — the orchestration source
  the skill compiles into its state machine.
- `.claude/rules/branch-manager.md` — file ownership + autonomy bounds
  the skill must respect.
- `.claude/rules/decision-queue.md` — DQ schema + attribution rules.
- `.claude/PRPs/templates/auto-phase-state.template.json` — auto-state
  JSON schema.
- `.claude/PRPs/reports/v1-SL-c-1-retro.md` — wall-clock evidence + c-1
  L14/L15/L16 lessons encoded in the skill design.
