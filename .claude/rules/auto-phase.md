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
| `scripts/brehon/resolve-dq-canonical.sh` | Canonical DQ resolver (phase-branch ⋃ worker-branches, dedup by id, worker wins on collision) — used by Phase 0.5 Step C |

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
5. **Concurrent advisor session** writing the same phase. Refuse —
   user must close the other session.

   Detection: `.claude/agent-activity.json` shows another active
   session with `mode: write` AND `phase: <same-phase>` AND
   `role: advisor`. Both `phase` and `role` must match before
   refusing — a `governance-v0` meta-editor session (writing skill
   bodies, rules, lesson files, templates) is NOT a concurrent
   advisor under this rule. Meta-editor commit subjects match
   `^(feat|chore|docs)\((advisor|rules|lessons|templates)\)` AND
   the session writes only files outside `phase-<phase>` worktree
   ownership; advisor commits write briefs + auto-state JSON +
   dispatch Junior tasks scoped to the phase.

   In short: refuse only when two sessions are racing on the same
   `phase-<phase>` topology. A meta-editor improving the skill
   while a separate advisor session runs `/auto-phase` is the
   normal cross-session-improvement pattern (per session retro
   2026-05-08 — first observed during c-2 first-run).
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

7. **L14 fix is belt-and-braces (REVISED 2026-05-18: runlog
   POST-merge).** The bm-merge brief explicitly orders the runlog
   COMPLETE write to happen **AFTER** `gh pr merge --delete-branch`:
   `gh pr merge → (merge succeeds) → git checkout governance-v0 +
   pull → Edit runlog COMPLETE entry with real merge sha → git add
   → git commit -m chore(bm): merge PR #<N> complete → git push
   origin governance-v0`. The prior pre-merge ordering (commit
   runlog to governance-v0 BEFORE `gh pr merge`) was **removed**:
   it self-conflicted with the bm-pr step's phase-branch runlog
   entry on the append-only `.claude/runlog/bm-runlog.md`, forcing
   the PR to DIRTY/CONFLICTING and blocking the merge (v1-ship-1-r2
   Junior #322; see `feedback_l14_runlog_on_trunk_self_conflicts_
   with_bm_pr.md` + DQ #265). The skill's post-merge tick scans
   `git log -3 governance-v0` for the `chore(bm): merge PR #<N>
   complete` matching the merge timeframe; if missing, advisor
   authors a `docs(advisor): L14 belt-and-braces — runlog COMPLETE
   re-apply` block with the verified real merge sha. This fallback
   is NOT auto-skipped; if the BM brief's git sequence is silent OR
   the BM Junior skipped the POST-merge runlog commit (observed —
   Junior #323), the post-merge re-apply MUST run. Audit-trail
   durability is preserved by the bm-pr "PR opened" entry (already
   on record before any merge attempt) plus this fallback.

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

Auto-state JSON is per-phase and survives session boundaries (Claude
Code compaction, `/clear`, deliberate session restart, laptop reboot).
A sub-phase typically runs ~3-4 hours — longer than any single
conversation lifetime — so resume MUST work as a first-class path,
not as an exceptional case.

### Invocation paths

On `/auto-phase v1-SL-c-2` invocation:

1. If `.claude/auto-state/v1-SL-c-2.json` does NOT exist → fresh init,
   stage = `init`, run Phase 0 prerequisites.
2. If file exists with `stage = done` → refuse: "phase already complete;
   delete state file or run new phase".
3. If file exists with `stage = catch-fire` → refuse auto-resume;
   surface the catch-fire dump path; require explicit
   `--start-from <stage>` after the user has addressed the underlying
   cause.
4. Otherwise → run **Phase 0.5 — Session resume** (skill body). Phase
   0.5 reconciles the persisted state with current world state (Junior
   task statuses, DQ delta, phase tip drift, daemon health) and prints
   a resume report. **Wait for user 'continue' reply** before any
   state-changing action.

### Hard invariants

A. **The same `/auto-phase <phase>` invocation handles fresh start AND
   resume.** No flag is needed for the common case. Discoverability
   from a fresh session: the user types the same command they used at
   phase start; the skill detects the state file and routes to Phase
   0.5 automatically. This invariant is load-bearing — losing it
   would mean the user must remember a flag across days of elapsed
   session time, which defeats the skill's purpose.

B. **Phase 0.5 is read-only until user 'continue'.** The reconciliation
   loop calls `mcp__junior-brehon__list_tasks`, `git fetch`, and reads
   the canonical DQ resolver — but does NOT queue Junior tasks,
   write DQ entries, or fire `gh pr merge`. The cost of one extra user
   touch on resume is much smaller than the cost of an unwanted resume
   action (e.g. re-queueing a Junior task that's still alive on the
   daemon, double-running cargo bg processes, racing against a peer
   advisor session).

   **DQ scan MUST use the canonical resolver** at
   `scripts/brehon/resolve-dq-canonical.sh <phase>`. The resolver
   unions phase-branch DQ with all open worker-branch DQs (per
   `auto-state.current_cohort.members[].junior_id` + `fix_attempts`)
   and dedupes by entry id (worker-branch wins on collision = most
   recent state). Reading only `.claude/decision-queue.json` from
   the laptop checkout misses entries on active worker branches that
   the EliteDesk daemon has pulled but not yet finalize-merged — the
   2026-05-09 c-2 resume incident: laptop saw pending=0 while live
   advisor saw pending=2 (DQ #164+#165 on worker-159). Same bug
   pattern would re-fire on every cohort with an in-flight ci-watcher.
   The resolver makes the canonical view explicit and reproducible,
   and the synthesis surfaces the contributing source labels
   (`phase-branch`, `worker-N`) so retro can audit which refs were
   in play.

   **Resume report is COMPACT (≤15 lines, ≤500 tokens).** Step E's
   surface-to-user output packs the world reconciliation onto one
   line per category; verbose probe output goes to a gitignored
   debug file the user requests with 'debug' (path:
   `.claude/auto-state/<phase>-resume-debug-<UTC-iso>.md`). Per
   2026-05-09 retro: the prior ~80-line verbose report cost ~3-5k
   parent tokens on every resume; with `resume_count: 2` already on
   c-2, that's ~10k tokens displaced from reasoning headroom for
   no signal-vs-noise gain.

   **Lazy-load discipline.** Phase 0.5 MUST NOT preload rule tables,
   lesson corpora, or §G4 classifiers at resume time. They get read
   just-in-time when a routing decision actually needs them
   (file-class table → at impl-cohort-N action; §G4 allowlist → at
   validate-pending fail handling; retro template → at retro-author
   entry). Per 2026-05-09 c-2 resume anti-pattern: live advisor
   preloaded the file-class table for "Tasks 2-5 dispatch later" —
   those tasks were ≥30 min away; ~5-8k tokens burned on never-used
   data.

   **Reconciliation Steps B-D delegate to a `general-purpose` subagent
   by default**, returning a single ~1 KB synthesis instead of ~12 KB
   of raw probe outputs to the parent. Justified by token efficiency
   on resume (parent context post-compaction is cold; reasoning
   headroom is the scarcest resource). The subagent's prompt is
   self-contained and explicitly forbids: Junior task dispatch, DQ
   writes, gh pr merge, auto-state JSON mutation. Step A (state read +
   schema upgrade) and Step E (resume report + user 'continue' gate)
   stay inline in the parent — they're load-bearing for routing
   decisions and surface-to-user respectively.

C. **`session_id` rotates on every resume.** The skill writes a new
   random 12-char hex on every Phase 0.5 entry. The previous
   `session_id` is preserved in `last_session_ended_at`'s nearby
   metadata so retros can identify session-boundary points. High
   `resume_count` (>3) on a single phase is a retro signal worth
   investigating.

D. **Phase tip drift is a normal advance signal, not a catch-fire.**
   If `last_known_phase_tip` lags `phase-<phase>` HEAD on resume, the
   Junior daemon finalize-merged a worker branch while the session
   was down. Phase 0.5 Step D detects this and advances state-machine
   forward; the skill does not catch-fire on phase-tip-ahead-of-state.

E. **No automatic Junior task re-queue on resume.** If a Junior task
   that was `running` at session-end is now `failed` or `cancelled`
   or missing from the daemon DB, surface the outcome to user; never
   auto-retry. The user decides whether the prior failure is real
   (root cause to fix) or transient (re-queue acceptable). This
   matches the standard `failed`/`cancelled` handling in Phase 5
   "Failure modes" of the skill body.

F. **AskUserQuestion gates persist across session boundaries.** If a
   session ended with stage = `*-pending-user`, the next session's
   Phase 0.5 re-fires the gate from the persisted question state.
   The user does not lose their place; they answer the same question
   they were asked before the session ended.

### What survives across session boundaries

The state file's authoritative fields (each must be enough to resume
without conversation context):

- `phase` — sub-phase name (e.g. `v1-SL-c-2`).
- `stage` — current state-machine state.
- `current_cohort` — for `impl-cohort-N-running` and friends, the
  cohort members' Junior task ids + their `validate-pending` DQ ids.
- `phase_2_e2e_mode` — cached choice from gate 4 ("local" / "dispatch")
  so resume doesn't re-ask.
- `junior_tasks` — map of stage to most-recent Junior task id, so
  `list_tasks` reconciliation can compare recorded vs current.
- `last_known_phase_tip` — phase branch SHA at last tick, for drift
  detection.
- `last_dq_pending_count` + `last_dq_pending_ids` — for delta scan in
  Step C.
- `user_gate_history` — what the user has decided so far this phase
  (used by retro author).

### What does NOT survive (deliberately)

- Conversation context — the new session has no memory of the prior
  session's reasoning. Phase 0.5's resume report must be self-
  contained ("here's what was happening; here's what you decided;
  here's the next action").
- Background processes — if a session ended with `cargo test --features
  full ... > .claude/runlog/e2e-...log 2>&1` running in the parent's
  bash session, that process may have died with the session OR may be
  orphaned. Phase 0.5 Step B for `phase-2-e2e-N-running` checks the
  log file's mtime + the OS process table to disambiguate.
- Subagent context — any Explore / general-purpose agents the prior
  session spawned are gone. Re-spawn if needed.

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
