# Auto-phase — orchestration state machine for `/auto-phase`

This rule is referenced by `~/.claude/commands/auto-phase.md` (user-scope
skill). The skill compiles `.claude/rules/advisor-orchestrator.md`
§"Stage-shape orchestration" + §"Cohort dispatch sequence" + §"§G4
classifier" into a deterministic state machine that drives a sub-phase
end-to-end with calibrated `ScheduleWakeup` cadences.

The rule exists because `/auto-phase` is the first user-scope skill that
mutates state machine progress across multiple advisor session restarts
(the auto-state JSON survives restart). Its hard refusals + state-routing
rules live in-repo so they are loaded on-demand at the start of every
`/auto-phase` invocation (per the skill body's Phase 0 Step 0).

> **Loading note (2026-05-22):** this file was relocated from
> `.claude/rules/` to `.claude/refs/` to free Memory-files budget — it is
> NO LONGER auto-loaded at session start (unlike `branch-manager.md`,
> `advisor-orchestrator.md`, `decision-queue.md`). The skill body
> `~/.claude/commands/auto-phase.md` Phase 0 Step 0 reads this file
> before any routing decision; if invoking `/auto-phase` logic outside
> the skill (e.g. ad-hoc advisor-driven resume), Read this file FIRST.

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
11. **`--unattended` allowlist violation.** Under `--unattended`, the
    skill MUST refuse to auto-clear any of the four judgment gates
    (1 plan-approval, 2 ADR/scope-DQ, 3 CR-triage, 5 merge-confirm) and
    MUST refuse to write `decision: "user"` for a policy auto-clear.
    The allowlist is exhaustive (only gates 4 e2e→local + 6 retro
    sign-off auto-clear) and is NOT runtime-extensible — no flag widens
    it. The full hard contract (park-and-ping mechanics, the gate-6
    3-check sanity gate, audit-honesty rule) lives in
    `~/.claude/commands/auto-phase.md` "Phase 7 — `--unattended` gate
    allowlist". A merge fired while unattended, or a `"user"` decision
    label on an auto-clear, is a catch-fire in the same class as a
    forged `answered_by: "advisor"`. Per
    `feedback_auto_phase_unattended_gate_allowlist.md`.

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
- `.claude/auto-state/<phase>.digests.jsonl` — append-only overflow
  for the `stage_digests` ring once it exceeds the cap (schema-v2,
  gitignored sibling under `.claude/auto-state/`).
- `.claude/auto-state/<phase>.spill/<stage>-<tool>-<ts>.txt` —
  uniform tool-output spill guard target (schema-v3, gitignored
  sibling; head+tail+path kept in context, full output spilled here).
- `.claude/PRPs/handovers/<phase>-auto-<date>.md` — auto-emitted,
  refreshed on each stage transition from the latest digest +
  durable ledger fields; satisfies the `advisor-orchestrator.md` §1
  pre-compact handover discipline automatically. Tracked,
  advisor-owned; committed with a `chore(advisor):` subject per the
  attribution-integrity pattern.
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
- Under `.claude/auto-state/`, the skill writes only gitignored
  runtime state: the catch-fire `.md` dump and the
  `<phase>.digests.jsonl` digest overflow — neither is a tracked
  artifact. The catch-fire path remains the ONLY path that writes a
  `.md` file under `.claude/auto-state/`.

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

## Context-management invariants

These invariants protect the durable-context substrate (the ledger)
and the bounded-context discipline. Schema-v2 (see
`.claude/PRPs/templates/auto-phase-state.template.json`).

1. **Stage-digest ring is data-only.** On each stage transition the
   skill appends one digest to `stage_digests` AFTER updating `stage`,
   then trims to the last `DIGEST_RING_MAX` (12), spilling the oldest
   to `digest_overflow_path`. The ring MUST NOT gate, route, or change
   cadence — it is a resume/`/compact` read source, nothing more. A
   change that makes a routing decision depend on a digest is a
   regression.

2. **Durable context is wide; delegation packets are narrow.** The
   ledger (advisor-durable) carries phase/stage/cohort/gates/digests/
   verification/next-action across boundaries. Junior briefs stay
   narrow and brief-scoped (one task brief, scope, write boundaries,
   validation expectations, relevant error history) per
   `advisor-orchestrator.md` §2.2 — the digest ring MUST NOT leak into
   brief bodies. Symmetry is deliberate: wide where it survives
   restarts, narrow where it is dispatched.

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
   a resume report. Step E builds that COMPACT report primarily from
   `stage_digests[-3:]` plus `last_handover_path`, then marks the latest
   `next_action_hypothesis` as a hypothesis to re-verify against live
   TaskList/DQ/PR/branch state. **Wait for user 'continue' reply**
   before any state-changing action.

### Hard invariants

A. **The same `/auto-phase <phase>` invocation handles fresh start AND
   resume.** No flag is needed for the common case. Discoverability
   from a fresh session: the user types the same command they used at
   phase start; the skill detects the state file and routes to Phase
   0.5 automatically. This invariant is load-bearing — losing it
   would mean the user must remember a flag across days of elapsed
   session time, which defeats the skill's purpose.

B. **Phase 0.5 is read-only until user 'continue'.** The reconciliation
   loop calls `mcp__junior-brehon__list_tasks`, `git fetch`, reads
   `stage_digests[-3:]`, reads `last_handover_path`, and reads the
   canonical DQ resolver — but does NOT queue Junior tasks, write DQ
   entries, or fire `gh pr merge`. The digest's `next_action_hypothesis`
   is re-verify-only, never permission to act. The cost of one extra user
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
- `stage_digests` — bounded ring of per-transition digests (schema-v2);
  the self-contained "here's what was happening" narrative Phase 0.5
  Step E reads to build the COMPACT resume report without conversation
  context. Overflow beyond the cap lives in `digest_overflow_path`.
- `spill_dir` — gitignored directory for large tool-output spills
  (schema-v3); full output lives on disk while only head+tail+path stay
  in live context.
- `last_handover_path` + `last_handover_at` — pointer to the
  auto-emitted handover refreshed at the last transition; Phase 0.5
  Step E and `/compact` priority 1 cite it as the authoritative
  active-thread handover source.

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

## Stage-shape orchestration (canonical contract)

Externalised from `.claude/rules/advisor-orchestrator.md` §3.1 on
2026-05-22 (rule-trim pass). The rule file kept a 16-bullet summary +
forward-ref; the full state-transition contract — with Phase 1/Phase 2
multi-paragraph detail — lives here. Cite this file by anchor (e.g.
"auto-phase.md §impl-task complete (Shape G)") when a lesson, brief,
or handover needs to reference one transition specifically.

This block supersedes the prior "Stage-shape semantics (live in
`advisor-orchestrator.md`; the skill reads but does not replace)"
forward-ref — the canonical contract now lives in refs/, both
`advisor-orchestrator.md §3.1` and the auto-phase skill's state
machine compile against this section.

### Brief authored, no planning task yet

Run `/brehon-clarify <brief-path>` → resolve every clarify-DQ entry →
queue planning. Skipping clarify on a planning brief is a process
breach.

### Planning complete

Run advisor-orchestrator.md §3.4 DoD smoke test → run §3.5 watchpoint
specificity → user gate 1 (plan approval) → on approval, queue
`bm-cut`.

### bm-cut complete

Queue impl per advisor-orchestrator.md §4 cohort dispatch: Task 1 (or
first non-pre-flight) `[P]` → compute cohort, queue all simultaneously;
otherwise queue alone.

### impl-task complete (pre-Shape-G, v1-JM-d and earlier)

If cohort has pending peers wait; else compute next cohort. All tasks
done → `chore(lint):` follow-up if needed → `bm-pr`.

### impl-task complete (Shape G, v1-JM-e onward)

Two-phase validation per option (b) 2026-04-28.

#### Phase 1 (workspace-check on `junior/*`)

impl-task already wrote `kind: "validate-pending"` post-push
(workflow_run_id + branch + phase_task; `result`/`log_slice`/
`failed_jobs` null). Queue `[role:ci-watcher]` Junior task with brief
from DQ entry fields (template
`.claude/PRPs/templates/ci-watcher-brief.template.md`). Originating
impl-task gated until ci-watcher resolves.

**Serial ci-watcher rule** (per `feedback_ci_watcher_serial_per_task_pair.md`
2026-05-11): when a cohort has N members each with a validate-pending
DQ, dispatch **ONE ci-watcher per logical task** (long-polling 1-2
workflow runs in sequence inside that ci-watcher), NOT N parallel
ci-watchers. Parallel ci-watchers all fork off the same phase-branch
tip; sequential finalize-merges then auto-resolve
`.claude/decision-queue.json` conflicts by reverting earlier
ci-watchers' mutations back to `pending` state ("resurrection bug").
Serial dispatch keeps each ci-watcher's worker branch in causal-order
with the previous mutation.

**Atomic raise-before-dispatch rule** (per
`feedback_dq_raise_before_ci_watcher_queue.md` 2026-05-11): the
`validate-pending` DQ entry MUST be committed and pushed BEFORE the
`[role:ci-watcher]` Junior task is created. Junior worker branches
fork from the current `phase-<phase>` tip at task-creation time; if
the raise hasn't pushed yet, the ci-watcher's worker branch will not
see the entry and will file a `kind: "blocker"` contract-violation.
The atomic ordering is: (a) `git add .claude/decision-queue.json &&
git commit && git push origin <phase-branch>`, THEN (b)
`mcp__junior-brehon__create_task`. NEVER reverse this order.

#### Phase 2 (e2e, advisor-driven, off-Actions by default — 2026-04-28 minutes-budget audit)

`cargo-test-e2e.yml` no longer auto-fires on `phase-v1-*` push. After
daemon finalize-merges, advisor sees new tip on next `git fetch` and
runs e2e locally. See advisor-orchestrator.md §5.2 validate-pending-laptop
handler for the full flow (raise `kind: "validate-pending"` with
`local_log_path` + `from: "advisor"`, no ci-watcher dispatch, advisor
mutates the entry directly when bg cargo exits). User-gate 4 (Phase 2
e2e — local vs dispatch) selects local vs `gh workflow run
cargo-test-e2e.yml`. Cohort advancement waits on BOTH workspace AND
e2e mutated to `result: "pass"`.

### ci-watcher complete

Read mutated entry. `kind` stays `"validate-pending"` regardless of
result. `result: "pass"` (in `resolved[]`) → advance pipeline.
`result: "fail" | "cancelled" | "timed_out"` (still in `pending[]`)
→ run advisor-orchestrator.md §5.3 §G4 classifier.

### All §16a stories `[done]`

Between last impl complete and bm-merge confirm: run `/brehon-verify`
→ phantom → catch-fire; else advance to bm-pr.

### bm-pr complete

**Fetch before write (mandatory):** immediately after `bm-pr` task transitions to `done`, run `git fetch origin governance-v0` before authoring the next brief, DQ entry, or auto-state update. The bm-pr Junior's finalize-merge lands on `governance-v0` between the task's `status: complete` and the advisor's next write — a stale local ref causes a non-fast-forward rejection on the subsequent `git push`. Per test-dogfood 2026-06-12 Race-A incident: bm-pr #656 finalize-merged concurrently with advisor DQ write → non-fast-forward; resolved via daemon-side `git merge origin/governance-v0` + push + laptop `git pull`. The fetch step is ≤1s and eliminates the race class.

Wait for CodeRabbit (`bm-task` polls) → on CR posted, queue
`bm-poll-cr`.

### bm-poll-cr complete

Queue `bm-triage` (draft auto).

### Triage drafted

User gate 3 (CR triage) → on approval, queue `impl-task` for
fix-in-PR commits.

### bm-pr complete → before gate 5: merge-forward check

Run `git log --oneline origin/governance-v0 ^phase-v1-<phase>` and
review for reformatting/structural commits landed on governance-v0
while the phase was in flight. Non-empty output = merge-forward
required: checkout phase branch → `git merge origin/governance-v0` →
resolve conflicts (`.claude/` files: `--ours`; Rust/migration files:
verify content, accept auto-resolution) → push. Then proceed to
`/brehon-verify` + gate 5. First occurrence:
v1-federation-inbound-d PR #146 blocked CONFLICTING by v1-quality-r1
rustfmt commit `2f13ffb80`.

### No critical findings open

Confirm `/brehon-verify` ✓ → user gate 5 (merge confirm) → queue
`bm-merge`.

### bm-merge complete

Author retro → user gate 6 (retro sign-off) → run
`/brehon-phase-transition`.

### Refusal

Advisor never auto-merges or auto-resolves ADR-affecting DQ.

## Cohort dispatch mechanism (canonical detail)

> Relocated from `advisor-orchestrator.md` §4 on 2026-05-29 (context-budget
> redesign B1, per `harness-redesign-session-profiles-2026-05-29.md`). The rule
> file keeps §4 / §4.1 as frozen heading stubs + a pointer here; the mechanism
> fires only inside `/auto-phase` `impl-cohort-N`, which reads this section JIT.
> Read this before acting on any cohort dispatch decision outside `/auto-phase`.

Per `.claude/PRPs/templates/plan.template.md` §13 (`[P]` markers) +
`feedback_parallel_cohort_dispatch.md`. When a §13 task carries `[P]` and is the
next pending, advisor computes the **cohort** — consecutive `[P]`-marked tasks
until a non-`[P]` boundary. Task 0 (pre-flight harness audit) is always non-`[P]`.

### Cohort dispatch sequence (steps 1–9)

1. Read plan §13. Locate next pending task by id (smallest task whose impl commit is not on the phase branch).
2. Non-`[P]` (or Task 0) → queue alone via `mcp__junior-brehon__create_task`; wait for complete/failed.
3. `[P]` → walk §13 forward collecting consecutive `[P]` until non-`[P]` boundary or end-of-list. Collected list = cohort.
4. **YAML overlap check** (per `feedback_explicit_file_arrays_on_tasks.md`): parse FILES YAML (`creates:` + `modifies:` arrays). Pairwise intersect across cohort members. Non-empty intersection → degrade to serial. Surface: `cohort overlap detected: tasks <A>+<B> share <path> — degrading to serial`. No DQ filed; planner's `[P]` was wrong; retro flags it. Missing YAML on any member → back-compat: skip check, trust `[P]`. Trust YAML over `[P]` when they disagree.
4a. **`requires:` dependency check** (per `feedback_cohort_validation_dependency_check.md` 2026-05-11). For each cohort member, parse FILES YAML `requires:` array. For each `requires: - task: <N>` entry: verify task `<N>`'s impl commit is already on `phase-<phase>` (`git log phase-<phase> --grep "(task <N>)" --oneline | head -1` returns non-empty). If task `<N>` is NOT yet merged: **(a)** if `<N>` is also in this cohort, refuse the cohort — these tasks need bundling, not parallelism. Surface: `cohort refused: tasks <A>+<B> have circular requires: — planner must bundle or re-order`. **(b)** if `<N>` is in a prior cohort not yet merged, defer the cohort until `<N>` lands. Surface: `cohort deferred: task <M> requires task <N> not yet on phase branch`. Missing `requires:` field on a member = no cross-task dependency claimed; trust the planner's `[P]` marker alone (back-compat: pre-2026-05-11 plans). This step prevents the Cohort A / Cohort B-serial isolation-validation bug class (per v1-RT-r1 halt retro `ffa2876e3`).
5. **Budget check** (pre-Shape-G only; Shape G non-binding, cargo runs off-box): each `cargo check --workspace --features full` ~6 GB peak (EliteDesk cap `MemoryMax=10G`). `cohort_size × per_task_peak > 10 GB` → degrade to serial. Per `feedback_resource_budget_pre_queue.md`. Surface: `cohort degraded to serial: budget exceeded (<size> tasks × <peak> GB > 10 GB)`.
5a. **Shared-`.git/index.lock` hazard check** (per `feedback_cohort_shared_git_index_contention.md`, 2026-05-25): on the EliteDesk daemon, every `git add` / `git commit` in any parallel worker acquires the SAME `.git/index.lock` — this is NOT in any task's FILES YAML and the YAML overlap check cannot detect it. Hard rule: **if the daemon topology is a single shared `.git/` (i.e. per-task worktrees share one `.git/worktrees/<name>/` admin tree under a single `.git/`) AND the cohort size is ≥3**, auto-degrade to serial regardless of file-disjointness. Surface: `cohort degraded to serial: shared .git/index.lock hazard (daemon single-.git/, cohort size <N> ≥ 3)`. Cohort size ≤2 is allowed (two workers contending on a lock is low-probability; three or more produce D-state cascades empirically — 2026-05-25 RT-r3 cohort-2 with #467/#468/#469). Under Mode A (lane worktrees on the laptop): this check is not applicable — each lane worktree has its own `.git/` isolated from other lanes. The rule applies only to daemon-dispatched `[P]` cohorts of size ≥3.
6. **Forbidden-window check** (advisor-orchestrator §5.1): if any cohort task starts in a forbidden window, defer the entire cohort.
7. **Queue every cohort task simultaneously** via parallel `create_task` calls (single message, multiple tool uses). Each task gets its own worktree. Brief paths unique per task.
8. **Wait for all members to reach complete or failed** before next cohort. A failed member blocks advancement.
9. **On cohort completion** (all members complete + validated) → run the handover aggregation (below) before computing next cohort.

### Cohort dispatch refusals

- Never queue a cohort whose tasks have not all been clarified. Re-run `/brehon-clarify` if post-clarify edits introduced overlap.
- Never queue a cohort during a forbidden window, even partially.
- Never re-queue a cohort task that already shows running (Junior task IDs are unique per worktree; duplicate worktree + branch name).
- Never queue a `[P]` task whose IMPLEMENT files overlap a non-`[P]` task still running. `[P]` is a within-cohort disjointness promise, not across-boundary.
- Never queue a cohort with non-empty YAML intersection without first degrading to serial. Mechanical: `intersect(union(creates, modifies)_taskA, union(creates, modifies)_taskB) != ∅` → degrade.

### Cohort handover aggregation

Per `feedback_handover_trailer_cohort_propagation.md`. Once all cohort members reach `complete` (and under Shape G, all paired `validate-pending` entries mutated to `result: "pass"` for both Phase 1 and Phase 2), populate the *next* cohort's brief §3a "Handover from prior cohort" before queueing.

1. For each cohort member's commit on `phase-<phase>`, parse `HANDOVER:` YAML trailer (`git log -1 --format=%B <sha>`). Missing trailer = degraded handover (note in polling output; not a catch-fire).
2. Aggregate into one block matching §3a schema in `impl-task-brief.template.md`:

```yaml
prior_cohort_tasks:
  - task: <N>
    commit: <sha>
    filesCreated: [...]
    filesModified: [...]
    keyDecisions: [...]
    notes: <verbatim from trailer>
  - task: <N+1>
    ...
```

3. For each next-cohort brief at `.claude/PRPs/briefs/<phase>-impl-<M>.md`, Edit §3a in place — replace `(none — first cohort)` or `(none — prior task non-[P])` with the aggregated block. Commit subject: `chore(advisor): inject prior-cohort handover for <next-cohort-tasks>`.
4. Push to `governance-v0`.
5. Proceed to next-cohort dispatch (step 1 above).

**Skip if** the next cohort is empty (prior cohort was last before retro) — retro task reads §3a as `(none — last cohort)`. **Single-task cohorts** (one `[P]` followed by non-`[P]`) still aggregate — trailer is the unit of handover; cohort size doesn't change the rule.

### Cohort dispatch notes

**Plans without `[P]` markers** (legacy or planner judged no parallelism safe) → every task non-`[P]`, dispatched serially. Cohort logic does NOT broaden serial into accidental parallel — `[P]` must be explicit.

**Cohort dispatch under Shape G:** members enter `kind: "validate-pending"` simultaneously after their respective push (one workspace-check workflow run per cohort task on GitHub-hosted runners). Advisor dispatches one ci-watcher per pending entry. Cohort advancement waits for **all** Phase-1 members to reach `result: "pass"`. Multiple simultaneous failures → classify each independently per advisor-orchestrator §5.3 (allowlist match → parallel fix-impl-tasks; non-allowlist → single catch-fire bundle). After all Phase-1 pass and daemon finalize-merges each into the phase branch, advisor raises ONE Phase-2 e2e `validate-pending` for the post-finalize phase-branch tip. Next-cohort advancement waits on Phase 2 e2e `result: "pass"` as well.

## Brief location per role + lane mode (canonical detail)

> Relocated from `advisor-orchestrator.md` §2.1 on 2026-05-29 (context-budget
> redesign B1). The rule keeps §2.1 as a heading + the dispatch-string rule; the
> per-role brief-location table + the Mode-A/Mode-B impl-task procedures live here.
> Read when authoring a brief (always inside a stage transition).

Every Junior task is preceded by a brief at `.claude/PRPs/briefs/<phase>-<role>-<n>.md`, committed on the branch the Junior worker will fork from **BEFORE** the task is created. Workers fork from `base_branch`; the brief must be visible in that branch's tree at task-spawn time. Per role:

- **Planning briefs** → committed on `governance-v0` (no phase branch yet).
- **bm-cut briefs** → committed on `governance-v0` (no phase branch yet). Author from `.claude/PRPs/templates/bm-task-brief.template.md` (promoted 2026-05-29) + 1-2 sibling `*-bm-cut-*.md` briefs as the canonical-schema-first reference.
- **bm-pr / bm-poll-cr / bm-triage / bm-merge / bm-push / bm-ping briefs** → committed on `governance-v0` (the bm-task worker reads from trunk). Author from `.claude/PRPs/templates/bm-task-brief.template.md` + 1-2 sibling briefs of the SAME verb. The template's per-verb cheat sheets (§2.1, §3, §4) encode the recurring shape.
- **Impl-task briefs** → MUST be visible on `phase-<X>` (the phase branch the impl worker forks from) before `create_task` is called. **The how depends on the lane mode** (per `.claude/rules/multi-lane-worktree.md` §"Lane modes"):
  - **Mode A (dedicated lane worktree):** author directly on the phase branch in the lane worktree session. `git commit` + `git push origin phase-<X>`. The fed-in-b pattern (commit `0ea7ab4f7`) is the canonical example.
  - **Mode B (mobile remote-control):** author on `governance-v0` in canonical, `git commit` + `git push origin governance-v0`, then trigger a trunk→phase sync per `multi-lane-worktree.md` §"Brief location and trunk→phase sync" (SSH-merge from the daemon's main worktree, which is on the phase branch post-bm-cut). Verify with `git -C <canonical> fetch origin phase-<X> && git log governance-v0..origin/phase-<X> --oneline` — the brief commit must appear via the merge commit.
  - In BOTH modes the worker forks from `phase-v1-<lane>`; the brief must be reachable at that ref at task-spawn time. The "how" differs; the "what" doesn't. Per session retro 2026-05-20 §2.8 + 2026-05-25 (Mode B procedure discovered empirically; documented post-session).
- **ci-watcher briefs** → committed on `governance-v0` (mutation lives on whatever ref the workflow_run_id's branch was; the brief just names IDs).

## §G4 classifier — allowlist + recipe tables (canonical detail)

> Relocated from `advisor-orchestrator.md` §5.3 on 2026-05-29 (context-budget
> redesign B1). The rule file keeps §5.3 as a resident heading with the
> cycle-count meta-rule + the allowlist-match-vs-catch-fire SAFETY statement; the
> allowlist/non-allowlist TABLES + callsite/pre-push discipline + anti-paraphrase
> gate live here. They fire only when a `validate-pending` entry mutates to a
> failure (mid-orchestration); the skill's Phase-2 routing reads them JIT. **The
> anti-paraphrase gate's verbatim-source pointer now resolves here:** fix-impl
> briefs copy the matched row from THIS table.

For an allowlist match: author a narrow fix-impl-task brief at `.claude/PRPs/briefs/<phase>-fix-impl-<n>.md` containing failed-job log slice (≤200 lines), specific file:line cited by the lint, auto-fix recipe from source lesson (or mechanical replacement), hard cap "≤3 file edits". Dispatched as normal `[role:impl-task]` Junior task; resulting commit lands on phase branch and re-triggers the workflow.

### Allowlist (auto-queue narrow fix-impl-task, ≤3 file edits)

| Failure signature | Auto-fix | Source lesson |
|---|---|---|
| `clippy::doc_lazy_continuation` warning | reword + mid-paragraph "and" | `feedback_clippy_doc_lazy_continuation_in_doc_comments.md` |
| `clippy::doc_overindented_list_items` warning | re-indent the doc-list continuation to clippy's suggested column (4 spaces under `///`) per the `help: try using` hint | n/a (mechanical; added 2026-06-19 m3-core-stage-mode Task 4 — sibling of `doc_lazy_continuation`) |
| `error[E0432]: unresolved import` | add the missing `use` per the suggestion | n/a (mechanical) |
| `warning: use of deprecated <api>` | replace with the suggested replacement | n/a (mechanical) |
| **4a** `error[E0277]: ?` couldn't convert `LemmyError`/`LemmyResult<T>` to `Box<dyn Error>` AND test fn returns `Result<(), Box<dyn Error>>` AND helpers all return `Result<T, Box<dyn Error>>` (Case B per lesson) | wrap each Lemmy-native call with **annotated** closure: `.map_err(\|e\| -> Box<dyn std::error::Error + Send + Sync> { format!("{e}").into() })?`. Bare `.map_err(\|e\| format!("{e}").into())?` will fail E0283 because `_` in `Into<_>` cannot resolve through abstract trait objects. | `feedback_lemmy_error_no_std_error.md` Case B |
| **4b** `error[E0277]: ?` couldn't convert `LemmyError`/`LemmyResult<T>` to `Box<dyn Error>` AND a v1-SL-* / v1-JM-* sibling fixtures module exists in the same file using `LemmyResult<()>` outer | flip the test fn signature to `LemmyResult<()>` AND flip ALL helper signatures in this module's fixtures mod to `LemmyResult<T>`. No `.map_err` bridges. Mirror the canonical sibling shape verbatim (Case A per lesson). | `feedback_lemmy_error_no_std_error.md` Case A |
| **4c** `error[E0277]: ?` propagation Send/Sync/Sized cascade (≥3 sites at once) — Case C symptom: test fn outer differs in Result type from helper outer | **HARD REFUSAL** — do NOT auto-queue a fix-impl task. Surface to user as a re-plan signal: type-shape uniformity is mandatory across a single test module; no mechanical bridge resolves Case C. The recipe family is wrong-shaped. | `feedback_lemmy_error_no_std_error.md` Case C |
| `error[E0277]: trait bound \`<T>: <Trait>\` not satisfied` where the lesson corpus has a citation | apply the recipe per the cited lesson | search `.claude/lessons/` for the failing trait + type before classifying |
| `clippy::map_err_ignore` (E0277-adjacent) | rename `\|_\|` → `\|_e\|` per `feedback_clippy_map_err_ignore_pattern_rename.md` (when authored) | mechanical |
| `error: cannot find macro \`<name>\` in this scope` | add the missing `use` from the macro's home crate | n/a (mechanical) |
| `error[E0599]: no method named \`<name>\`` (when method is on a re-exported trait) | add the missing `use` for the trait | n/a (mechanical, but verify the trait isn't intentionally hidden) |

**Callsite-enumeration discipline (per `feedback_fix_impl_enumerate_all_callsites.md` 2026-05-11):** when the failure signature is a struct-shape change (E0063 missing-field on `<Type>` initializer; renamed/added field; trait-impl signature change) and the compile error points at K specific call sites, the advisor MUST `rg "<Type>" crates/ tests/` to enumerate the FULL set of N callsites BEFORE authoring the fix-impl brief. The brief lists all N callsites; the file-edit cap is the count of distinct files containing those callsites (NOT the ≤3 default — that cap was designed for clippy/unused-import patterns, not struct-shape changes). If N > 10 callsites or > 5 files, the change is no longer "narrow mechanical" — catch-fire to user with the enumeration list so user can decide whether to extend the brief or split the fix across multiple commits. Without enumeration, fix-impl-1 patches only the compile-error-cited K sites and the workflow fails AGAIN on the next N-K callsites (v1-RT-r1 fix-impl-1 DQ #205 incident: brief covered 2 sites; workspace check FAILED with same E0063 on 8 more callsites, forcing fix-impl-2).

**Pre-push cargo-check discipline (per `feedback_fix_impl_pre_push_cargo_check.md` 2026-05-13):** mechanical fix-impl briefs MUST include a §4 Constraint requiring `bash scripts/brehon/cargo-check.sh --workspace --features full` (or `.bat` on Windows worker) BEFORE the worker pushes the worker branch. Non-zero exit → patch in same commit (if in-scope) OR file `kind: "blocker"` DQ (if out-of-scope). NEVER `#[allow]`-spam to bypass. Without this gate, an adjacent regression class (typically unused-import surfacing after a struct-field pad — e.g. v1-RT-r1 fix-impl-3 incident: `seed_founders/main.rs` import unused in non-test build after fix-impl-2 padded the consumers) costs a full ci-watcher cycle + fix-impl-(N+1) recovery. Local `cargo check` is ~30s warm; one extra ci-watcher cycle is ~5 min. Net positive on every cycle.

### Non-allowlist (catch-fire to user)

| Trigger | Action |
|---|---|
| Compile errors (any `error[E*]` other than `E0432`) | Catch-fire |
| Test failures (panics, assertion fails, e2e flakes, testcontainers issues) | Catch-fire |
| Timeout / OOM / runner death | Catch-fire |
| Any failure whose log slice doesn't match an allowlist row | Catch-fire |
| Conformance-audit Tier-1 finding on `crates/apub/activities/src/governance/**.rs` OR `crates/api/api/src/governance/**.rs` OR `crates/db_schema/src/source/governance/**.rs` | **HARD REFUSAL — catch-fire to user** with audit report + suggested per-axis fix. NOT auto-fix; human-in-the-loop decides. (Per `feedback_mirror_phase6_convention_in_same_file.md`.) |

Surface as: "validate-failed on `<branch>` (workflow run `<id>`): non-allowlist failure. Failed jobs: `<failed_jobs>`. Log slice attached. Surfaced to user — no auto-fix attempted."

The allowlist is **conservative by design** (per `feedback_principles_not_rules.md`). Grow only on retro evidence — if a CR-triage cycle classifies a non-allowlist as "this could have been auto-fixed", record in retro §5 watch-items and add to next sub-phase's plan if pattern reproduces.

### Mandatory verbatim §G4 row in fix-impl briefs (anti-paraphrase gate)

When a fix-impl-task brief's triggering DQ matches an allowlist row above, the brief's §2 Scope MUST contain a **verbatim block-quote of the matched row text — both columns (Failure signature + Auto-fix recipe + Source lesson) — copy-pasted as a markdown blockquote (`> ...`) BEFORE any file:line context.** Canonical recipe text is the contract; paraphrase is a process miss even if meaning preserved.

Blockquote shape (matches the table row literally):

```markdown
## 2. Scope

### 2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/refs/auto-phase.md` §"Allowlist (auto-queue narrow fix-impl-task, ≤3 file edits)")

> | Failure signature | Auto-fix | Source lesson |
> | <row text 1> | <row text 2> | <row text 3> |
```

After the blockquote, brief MAY add file:line context, line-by-line diff targets, acceptance criteria — but the recipe text is the contract Junior implements against. If the rest of the brief contradicts the blockquote, the blockquote wins (Junior hard refusal: stop, raise `kind: "blocker"` DQ citing this gate).

**Detection:** an advisor commit adding `.claude/PRPs/briefs/*-fix-impl-*.md` matching an allowlist row but lacking the verbatim §2 blockquote is a process miss — retro flags it. Future PostToolUse hook on brief Write/Edit can scan §2 for the literal blockquote (not yet implemented).

**Does NOT apply** to non-allowlist fix-impl briefs (catch-fire with hand-authored recipe). Gate prevents paraphrase drift on mechanical recipes, not user-driven fixes.

// 2026-05-09 c-2 cycle-2 catch-fire: fix-impl-1 brief cited canonical lesson in §3 but paraphrased in §2 (signature flip + forbid `.map_err`, instead of canonical "signature stays Box<dyn Error> + add `.map_err`"). Junior #158 followed brief literally. Cost: ~17 min. Verbatim copy-paste makes "I read the row but prescribed something different" structurally impossible.

## What this rule does NOT cover

- The skill's tick procedure body (lives in
  `~/.claude/commands/auto-phase.md` Phase 1).
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
