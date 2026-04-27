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

## Memory and lessons (one-system principle)

The advisor reads two memory paths:

- **Repo lessons** (`.claude/lessons/`) — the same files Junior subagents read. Consult before queueing each new task; cite by filename in briefs.
- **Laptop PMD** (via memory MCP) — search index over the same content, plus `project_*` files (current-state, in-flight). Use for cross-phase pattern recall when authoring retros and brief-revision questions.

When a retro promotes a new lesson, the advisor copies the lesson file into `.claude/lessons/` in the same retro commit so the next Junior task has it.

## Stage-shape orchestration (the auto-decisions)

Per the c-inherited-dragon plan's "Stages of a sub-phase" map, the advisor knows what to queue next on each completion:

- **Planning complete** → run DoD smoke test → surface to user → on user approval, queue `bm-cut` to make the phase branch
- **bm-cut complete** → queue `impl-task` for plan task 1
- **Each impl-task complete** → check plan task list; queue `impl-task` for next task; or if all tasks done, queue `bm-cut` follow-up (`chore(lint):` if needed) then `bm-pr`
- **bm-pr complete** → wait for CodeRabbit (`bm-task` polls) → on CR posted, queue `bm-poll-cr`
- **bm-poll-cr complete** → queue `bm-triage` (draft auto)
- **Triage drafted** → surface to user → on approval, queue `impl-task` for fix-in-PR commits
- **No critical findings open** → surface merge-confirm to user → on confirm, queue `bm-merge`
- **bm-merge complete** → author retro → surface to user → on sign-off, run `/brehon-phase-transition`

The advisor never auto-merges or auto-resolves ADR-affecting DQ. User gates stay.

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
