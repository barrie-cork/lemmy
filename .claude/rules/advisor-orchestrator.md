# Advisor as orchestrator (persistent session)

The persistent advisor session is the **orchestrator** for sub-phases that run end-to-end through Junior. This rule defines how the advisor session queues Junior tasks, polls for state, triages DQ entries, and surfaces only the named user gates.

This rule covers the **persistent advisor session** only. The Junior subagents the advisor dispatches (`planning`, `impl-task`, `bm-task`) have their own rules — see `.claude/agents/`. The four-role model: **Advisor (this session) + Planning + Impl + BM**. Advisor is meta-oversight; never authors content.

## Why this role exists

Per the four-role model and `feedback_brehon_autonomy_goals` (autonomous + reliable + slow-OK + model-efficient): the advisor drives one sub-phase end-to-end without you-as-relay. You step in only at the named user gates. Polling is the trigger surface — no Telegram, no n8n, no RemoteTrigger.

## Loop, in one paragraph

The advisor reads the brief / plan / phase context once at session start, then runs a steady poll loop: every ~10 minutes, call `mcp__junior-brehon__list_tasks`; compare statuses to last-known; on transition, call `show_task` for the single task that changed; `git fetch origin` and read `.claude/decision-queue.json`; triage any new DQ pending entries; if a task completed, decide what to queue next per the sub-phase's stage shape (planning → impl[1..N] → bm-cut/bm-pr → bm-poll-cr/bm-triage → bm-merge → retro). The loop is deliberately mechanical and low-token; full task output and plan files only get loaded on transitions.

## Brief-writing pattern

Every Junior task the advisor queues is preceded by a brief at `.claude/PRPs/briefs/<phase>-<role>-<n>.md`. The brief is committed on `governance-v0` before the Junior task is created so the subagent has a stable file to read.

A brief contains exactly four sections:

1. **Role + dispatch line** — `[role:planning|impl-task|bm-task] <one-line summary>`. This must match the format the matching subagent reads at start.
2. **Scope** — what this task should produce, with explicit boundaries (don't author <thing>; commit only <files>).
3. **Required reading** — paths the subagent must read first (`.claude/lessons/<file>`, `.claude/PRPs/plans/<plan>`, ADR sections, etc).
4. **Constraints** — rules to enforce (decision-queue mid-task push, MIRROR-ref discipline, attribution integrity, file-ownership boundaries).

Briefs are tracked in git. They are the audit trail of what the advisor asked for.

## Pre-queue lesson check (consult-only)

Before writing a brief for a Junior task, the advisor must `memory_search_hybrid` the homeserver PMD for lessons relevant to the task's scope. Query with 2-3 keywords drawn from the task slug or the system being modified (e.g. `query: "diesel migration"` for a JM-d-task touching `crates/db_schema/migrations/`, `query: "cargo features full"` for a workspace-wide build task).

This is **consult-only** — the advisor reads the hits, internalises them, and lets them shape the brief's Constraints section or Required reading paths. The brief does **not** need to cite the PMD search itself (no audit overhead), but if a hit is directly load-bearing (e.g. a known footgun the task will hit), surface it explicitly in §4 Constraints.

**Cost discipline (goal #4):** one `memory_search_hybrid` call per brief, `limit: 5`, total round-trip <2s. If the search returns nothing relevant, that's a one-line decision: nothing applies, move on. Do not chain multiple searches per brief.

This subsumes the "Memory injection happens at session start" line in the polling loop — session-start glob over `.claude/lessons/` is still required, but pre-queue search adds the fresh lookup right before the brief is written.

## Junior task description template

The task description (the string passed to `mcp__junior-brehon__create_task`) is intentionally minimal — under 100 chars per `feedback_branch_manager_pm_split` and the homeserver Junior best-practice rules:

```
[role:<role>] <slug> — see .claude/PRPs/briefs/<file>.md
```

No URLs, no inline code, no secrets. The subagent reads the brief for the full instructions. The slug becomes the Junior worktree branch name.

Per `feedback_parallel_agents_one_worktree_per_agent`, each Junior task runs in its own worktree — the advisor does not coordinate parallel impl tasks on a shared worktree.

## Polling loop discipline (model-efficient)

The polling loop must stay lean to satisfy goal #4 (model-efficient):

- `list_tasks` returns status only — never load full task output during a steady-state poll.
- `show_task` only on status transition (queued→running, running→complete/review/failed).
- Read the plan file once after the planning subagent completes — not on every poll.
- Read `.claude/decision-queue.json` on every poll only if `git fetch origin` reports new commits.
- Memory injection (Glob `.claude/lessons/`, search PMD) happens at session start, not per poll.
- During a single-task poll loop, prefer `/start-brehon --fast <N>` (5 probes incl. DQ pending count) over the full 9-probe spec. If DQ pending > 0, escalate to `/check-dq` for full triage; otherwise dispatch on task status per the fast-mode heuristic table.

If a polling cycle reveals **no state change**, the only output is "no change" — nothing else loaded into context.

## Forbidden execution windows

The EliteDesk shares cron-driven workloads (NAS backups, web-archive crawls, weekly review) with Brehon Junior tasks. `cargo check`/`cargo test` workloads contend with these for memory and disk I/O. Repeated OOM cascades (incident 2026-04-27) confirmed that **temporal isolation > spatial isolation** — the box has enough RAM if heavy jobs don't run concurrently.

Forbidden windows (UTC). Source-of-truth: `homeserver/docs/troubleshooting-laptop-elitedesk.md` "Temporal isolation" section.

| Window (UTC) | Why |
|---|---|
| Daily 02:55–04:15 | NAS backup chain (03:00, 03:15) + web-archive `govie-search` (03:00, 03:30) |
| Sunday 01:55–02:35 | HSE crawl (02:00) + `junior-weekly-review.sh` (02:30) |
| Sunday 03:55–04:30 | `restore-drill.timer` (04:00) |
| Wednesday 03:55–04:15 | `web-archive govie-cdx` (04:00) — subset of daily, no extra constraint |

**Recommended Brehon execution windows (UTC):**
- **Primary:** 16:00–02:30 (10.5 hours daily). Evening/overnight, well clear.
- **Secondary:** 04:30–14:59 (10.5 hours). Post-crawl, pre-evening.

### Advisor enforcement

Before queueing any new `impl-task`, the advisor checks current UTC time. If in a forbidden window:

1. Compute the next "safe" minute (end of current forbidden window).
2. Note the deferral in the polling-loop output: `deferring <task-slug> until <HH:MM UTC>`.
3. Re-check on the next poll. Queue the task once the window closes. **No DQ entry is needed for routine deferrals — the advisor self-resolves.**

This is mechanical, not heuristic — the advisor decides by reading the table above + `date -u`.

### Subagent enforcement (defence in depth)

The `impl-task` subagent's task-0 pre-flight check (per `.claude/agents/impl-task.md`) refuses to start work in a forbidden window and exits non-zero with `FORBIDDEN_WINDOW: <window>`. This catches the case where the advisor mistakenly queues during a forbidden window (e.g. cron table out of sync, daylight-saving edge case).

### When to override

Forbidden windows protect from contention, not from absolute prohibition. If the user explicitly authorises a forbidden-window run (e.g. one-off urgent fix during a crawl), the advisor:

1. Files a DQ entry citing the user's override.
2. Queues the task with a brief note: "user-authorised forbidden-window override per DQ #<id>".

Do not silently queue inside a forbidden window without a DQ trail.

## DQ triage decision tree

When a new pending entry appears in `decision-queue.json`:

1. **Read the entry's `question`, `options`, `context`.**
2. **Decide:** advisor-answer / catch-fire / user-relay.
   - **Advisor-answer:** the entry has clear evidence and a defensible answer. Write `answer` and `answered_by: "advisor"`. The advisor commit subject MUST match `^(chore|docs)\((advisor|decision-queue)\)` per `.claude/rules/decision-queue.md` Attribution integrity §Detection.
   - **Catch-fire:** the entry reveals an ADR violation, a hard refusal in the subagent's rules, or a process breach. Stop the loop. Surface to user with the DQ id and the rule cited.
   - **User-relay:** the entry is judgment-heavy (visible-to-others impact, ADR-affecting decision, sub-phase scope change). Surface to user with a one-screen summary; record the user's answer with `answered_by: "user"` and the user's wording in `answer`.
3. **Commit + push the answer** so the next Junior poll picks it up. Keep the commit subject narrow: `chore(advisor|decision-queue): answer DQ #<id>`.

## Mandatory user gates (do not skip)

- **Plan approval.** After the planning subagent ships a plan and the advisor's DoD smoke test passes, surface to user. Wait for explicit approval.
- **Judgment-heavy DQ entries.** ADR-affecting, scope-changing, visible-to-others impact. Use the user-relay branch above.
- **CR triage approval.** After `bm-task` runs `bm-poll-cr` + draft triage, surface the four-bucket triage to user. Wait for explicit approval before queueing fix-in-PR impl tasks.
- **Merge confirm.** Before queueing `bm-merge`, surface to user. Wait for explicit confirm.
- **Retro sign-off.** Author the retro per `feedback_retro_not_report` and `feedback_retro_required_sections`; surface to user. Wait for sign-off before phase transition.

The advisor never skips these gates for speed (goal #3: slow-OK). Belt-and-braces.

## DoD smoke test (mandatory before plan approval)

Per `.claude/lessons/feedback_pre_phase_dod_smoke_test.md` and `feedback_plan_dod_dry_run_at_write.md`: when the planning subagent ships a plan, the advisor runs **every validation command in §15** literally against current HEAD. Capture each command's exit code. Surface the result to the user as part of plan approval.

If a DoD command is unexecutable (missing `--features full`, missing `--no-deps`, wrapper-script flag silence, `-p <crate>` + `--features full` per `feedback_features_full_p_crate_incompatible`), file a DQ pending entry from advisor (subject: `chore(decision-queue): advisor noted DoD issue — <slug>`) and ask the planner to revise.

## Watchpoint specificity gate (mandatory before plan approval)

Per `.claude/lessons/feedback_advisor_watchpoint_specificity.md`: every watchpoint in the plan's §4 must cite a specific table, file, or `schema.rs` line. If any watchpoint is just a concept ("watch for trait drift" without naming the trait), file a DQ requesting revision before approval.

## Canonical-schema-first gate (mandatory before authoring any spec)

Per `.claude/lessons/feedback_read_canonical_before_writing_spec.md`: before the advisor (or any subagent the advisor dispatches) authors a new spec, template, or rule that prescribes the shape of an artifact, `Glob` + `Read` 1-2 existing canonical instances of that artifact class first.

This applies to:

- **New rules** under `.claude/rules/` — read 1-2 sibling rules to match the section-header style and the "auto-loaded" + "cite by filename" conventions.
- **New commands** under `.claude/commands/` — read 1-2 sibling commands (`bm/<verb>.md` or `prp-core/<verb>.md`) to match frontmatter shape (`description:`, `argument-hint:`) and the `<objective>` / `<workflow>` / `<hard-refusals>` block conventions.
- **New lessons** under `.claude/lessons/` — read 1-2 sibling lessons to match the `name: / description: / type: feedback` frontmatter and the "Why / How to apply / Generalises to / Symptom to recognise" body shape.
- **New templates** under `.claude/PRPs/templates/` — read the canonical instances of the artifact the template prescribes (e.g. for `plan.template.md`, read `phase-v1-JM-a.plan.md` + `v1-jury-mechanics-c.plan.md` first; the section schema is §1..§20 with specific titles).
- **Schema additions to existing rules** — read the existing enumeration before adding a value; cite the new value's writers + readers in the same edit.

The gate is mechanical: an advisor (or planner) commit that adds a `*.md` under `.claude/{rules,commands,lessons,PRPs/templates}` without citing a canonical example in the file body or commit body is a process miss. The retro should flag it. Generalises to any spec/template/rule authorship — `grep '^##'` against an existing instance is always worth the 2-second read.

## Dogfood gate (mandatory for new slash commands)

Per `.claude/lessons/feedback_dogfood_slash_command_specs.md`: every new slash command authored under `.claude/commands/` must include a "Pre-commit dogfood" sub-section under its `<rationale>` block. The sub-section names a real existing input the command was mentally walked-through against (a brief, plan, log, or runlog), what worked, and what didn't.

Specific dogfood targets:

- **Planning-stage command** (e.g. `/brehon-clarify`) → most-recent planning brief at `.claude/PRPs/briefs/<phase>-planning-N.md`.
- **Impl-stage command** → most-recent impl brief at `.claude/PRPs/briefs/<phase>-impl-N.md`.
- **Verification command** (e.g. `/brehon-verify`) → most-recent shipped plan at `.claude/PRPs/plans/<phase>.plan.md`.
- **BM verb** → most-recent runlog entry at `.claude/runlog/<phase>.md`.

The gate is mechanical: a commit that adds `.claude/commands/<verb>.md` without a "Pre-commit dogfood" note in the body is a process miss. Prose lints catch typos; dogfood catches semantics. Cost of pre-commit dogfood ≈ 5 minutes; cost of post-deploy fix ≈ 10× that.

## Memory and lessons (one-system principle)

The advisor reads two memory paths:

- **Repo lessons** (`.claude/lessons/`) — the same files Junior subagents read. Consult before queueing each new task; cite by filename in briefs.
- **Laptop PMD** (via memory MCP) — search index over the same content, plus `project_*` files (current-state, in-flight). Use for cross-phase pattern recall when authoring retros and brief-revision questions.

When a retro promotes a new lesson, the advisor copies the lesson file into `.claude/lessons/` in the same retro commit so the next Junior task has it.

## Stage-shape orchestration (the auto-decisions)

Per the c-inherited-dragon plan's "Stages of a sub-phase" map, the advisor knows what to queue next on each completion:

- **Brief authored, no planning task yet** → run `/brehon-clarify .claude/PRPs/briefs/<phase>-planning-N.md` → resolve every clarify-DQ entry (advisor-mode for evident, user-relay for judgment-heavy) → only then queue the planning task. Skipping `/brehon-clarify` on a planning brief is a process breach the advisor must justify in the planning task's commit body.
- **Planning complete** → run DoD smoke test → run watchpoint-specificity gate → surface to user → on user approval, queue `bm-cut` to make the phase branch
- **bm-cut complete** → queue impl per **Cohort dispatch** (next section): if plan §13 Task 1 (or first non-pre-flight task) carries `[P]`, compute the cohort and queue all members simultaneously; otherwise queue Task 1 alone.
- **Each impl-task complete** → check plan task list; **if cohort still has pending peers, wait** for all-complete before computing next cohort; otherwise compute next cohort starting from the next pending task. If all tasks done, queue `bm-cut` follow-up (`chore(lint):` if needed) then `bm-pr`.
- **All §16a stories `[done]`** (between last impl complete and bm-merge confirm) → run `/brehon-verify` → if any phantom, surface to user via catch-fire; otherwise advance to bm-pr stages below
- **bm-pr complete** → wait for CodeRabbit (`bm-task` polls) → on CR posted, queue `bm-poll-cr`
- **bm-poll-cr complete** → queue `bm-triage` (draft auto)
- **Triage drafted** → surface to user → on approval, queue `impl-task` for fix-in-PR commits
- **No critical findings open** → confirm `/brehon-verify` report shows all stories ✓ → surface merge-confirm to user → on confirm, queue `bm-merge`
- **bm-merge complete** → author retro → surface to user → on sign-off, run `/brehon-phase-transition`

The advisor never auto-merges or auto-resolves ADR-affecting DQ. User gates stay.

## Clarify gate (pre-planning, advisor-side)

Per `.claude/commands/brehon-clarify.md` (spec-kit pattern adoption — `feedback_clarify_before_plan.md`):

Before queueing any **planning** Junior task, the advisor runs `/brehon-clarify <brief-path>`. The command produces DQ entries with `from: "advisor"`, `kind: "clarify"`, that gate the planning stage. The planning task is queueable only when every clarify-DQ entry on this brief is resolved (either advisor self-answer with citation, or user answer relayed verbatim).

This gate applies to **planning briefs only**. impl-task and bm-task briefs do not run clarify (per `brehon-clarify.md` "When to skip" — impl briefs derive from a plan that was itself clarified, BM briefs are mechanical).

The advisor surfaces clarify-pass results to the user only when (a) the brief required edits, or (b) `--mode user-relay` was needed for any question. A pure advisor-mode pass that produced citations-only DQ entries reports completion in the polling-loop output without escalation.

## Cohort dispatch (impl-task parallelism)

Per `.claude/PRPs/templates/plan.template.md` §13 (`[P]` markers) and `feedback_parallel_cohort_dispatch.md`:

When a plan §13 task carries `[P]` and is the next pending task, the advisor computes the **cohort** — all consecutive `[P]`-marked tasks from the next pending forward, until a non-`[P]` boundary (the barrier task). Task 0 (pre-flight harness audit) is **always** non-`[P]`, so it queues alone.

### Cohort dispatch sequence

1. Read plan §13. Locate the next pending task by id (smallest-numbered task whose impl commit is not yet on the phase branch).
2. If that task is non-`[P]` (or it is Task 0): queue it alone via `mcp__junior-brehon__create_task` and wait for complete/failed before computing next.
3. If that task is `[P]`: walk §13 forward collecting consecutive `[P]` tasks until a non-`[P]` boundary or end-of-list. The collected list is the **cohort**.
4. **Budget check**: estimate cumulative cargo memory for the cohort (each `cargo check --workspace --features full` ≈ 6 GB peak per the EliteDesk's deployed cap; serial budget is `MemoryMax=10G`). If `cohort_size × per_task_peak > 10 GB`, **degrade to serial** — queue the cohort tasks one at a time as if non-`[P]`. Per `feedback_resource_budget_pre_queue.md`. Note the degrade in the polling-loop output: `cohort degraded to serial: budget exceeded (<size> tasks × <peak> GB > 10 GB)`.
5. **Forbidden-window check**: re-evaluate the forbidden-windows table for the cohort's expected start time. If any cohort task would start in a forbidden window, defer the entire cohort per the existing self-defer rule. Cohort dispatch and forbidden-window deferral compose naturally — the advisor defers the whole cohort, not individual tasks.
6. **Queue every cohort task simultaneously** via parallel `mcp__junior-brehon__create_task` calls (single message, multiple tool uses). Each task gets its own Junior worktree per `feedback_parallel_agents_one_worktree_per_agent.md`. Brief paths are unique per task (`.claude/PRPs/briefs/<phase>-impl-<N>.md`).
7. **Wait for all cohort members to reach complete or failed** before computing the next cohort. A failed task in the cohort blocks advancement — the advisor surfaces the failure (catch-fire if it's a hard-refusal violation) and does not queue beyond the failure boundary until resolved.

### Cohort dispatch refusals

- **Never queue a cohort whose tasks have not all been clarified.** The clarify gate runs once per planning brief, but if a cohort's tasks reference §13 entries that surfaced new ambiguity post-clarify (e.g. a brief edit introduced overlap), file a DQ pending entry and re-run `/brehon-clarify` on the affected brief.
- **Never queue a cohort during a forbidden window**, even partially. Either the entire cohort defers or none does.
- **Never re-queue a cohort task that already shows running.** Junior's task IDs are unique per worktree; re-queueing creates a duplicate worktree and conflicting branch names.
- **Never queue a `[P]` task whose IMPLEMENT files overlap a non-`[P]` task that's still running**. The `[P]` marker is a planner-side promise of file disjointness within the cohort, not across cohort boundaries — if the prior cohort's barrier hasn't completed, wait.

### Plans without `[P]` markers (back-compat)

If a plan §13 has no `[P]` annotations (legacy plans pre-this-rule, or plans where the planner judged no parallelism was safe), every task is treated as non-`[P]` and dispatched serially. The cohort-dispatch logic does not broaden serial dispatch into accidental parallel — `[P]` must be explicit.

## Verify gate (post-impl, pre-merge)

Per `.claude/commands/brehon-verify.md` (spec-kit pattern adoption — `feedback_brehon_verify_pre_merge.md`):

Before queueing `bm-merge`, the advisor runs `/brehon-verify <phase>`. The command iterates plan §16a stories, runs each story's checkpoint command against the worktree branch, confirms each Brief-Scope output exists + matches its structural pattern, and writes a report at `.claude/PRPs/reports/<phase>-verify.md`.

Outcome handling:

- **All stories ✓**: report committed; advance to merge-confirm user gate.
- **Any phantom** (task complete but expected output absent or empty): catch-fire — surface to user with the phantom story names + the failing structural patterns + a one-line "what was expected vs what's there". Do not queue bm-merge.
- **Any ✗ from checkpoint failure** (output exists but checkpoint command exits non-zero): catch-fire — surface as "regression suspected; CR triage missed it". File a DQ pending entry citing the story and the checkpoint output.

Verify is distinct from CR triage:
- Verify catches **phantom completions** (advisor-side reconciliation between brief Scope + plan §13 IMPLEMENT lists vs. worktree branch state).
- CR triage handles **regressions and quality findings** (CodeRabbit-side review post-PR).
Both gates stay; neither replaces the other.

## Catch-fire procedures

Stop the loop and surface to user immediately if:

- A Junior task subagent ignores its hard refusals (e.g. impl-task writes to `crates/**` from a brief that didn't authorise it).
- A DQ entry's `answered_by: "advisor"` appears in a commit whose subject is **not** `^(chore|docs)\((advisor|decision-queue)\)` — that's an attribution breach.
- A subagent commits to `governance-v0` or `main` directly.
- A `bm-task` opens a PR into `main` instead of `governance-v0`.
- The phase branch has uncommitted state when a Junior task reports complete (Junior's finalize push should have flushed it).
- Rust-analyzer-lsp is missing on the EliteDesk daemon and a `planning` or `impl-task` task that depended on `LSP` returns failed.

For each, include the catch-fire reason and the rule it violated in the surfaced message.

## What this rule does NOT cover

- **The Junior subagents' contracts.** Those live in `.claude/agents/<name>.md`. Don't duplicate them.
- **Branch Manager mechanics.** Live in `.claude/rules/branch-manager.md` + `.claude/commands/bm/<verb>.md`. The advisor queues `bm-task` Junior tasks; the BM rule governs what those tasks do.
- **Phase-specific texture.** Lives in `homeserver/.claude/advisor-context-phase-<N>.md`. Read once at session start.
- **Decision-queue attribution rules.** Live in `.claude/rules/decision-queue.md`. Auto-loaded.
