# Advisor as orchestrator (persistent session)

> **Mirror note:** this file is mirrored at `homeserver/.claude/rules/advisor-orchestrator.md`. The brehon-fork copy is canonical for in-repo execution; the homeserver copy mirrors it. Keep them in sync — surface a `diff` if they drift.

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

Before writing a brief for a Junior task, the advisor must `memory_search_hybrid` the brehon-fork PMD (at `.project-memory/memory.db`) for lessons relevant to the task's scope. Query with 2-3 keywords drawn from the task slug or the system being modified (e.g. `query: "diesel migration"` for a JM-d-task touching `crates/db_schema/migrations/`, `query: "cargo features full"` for a workspace-wide build task).

This is **consult-only** — the advisor reads the hits, internalises them, and lets them shape the brief's Constraints section or Required reading paths. The brief does **not** need to cite the PMD search itself (no audit overhead), but if a hit is directly load-bearing (e.g. a known footgun the task will hit), surface it explicitly in §4 Constraints.

**Cost discipline (goal #4):** one `memory_search_hybrid` call per brief, `limit: 5`, total round-trip <2s. If the search returns nothing relevant, that's a one-line decision: nothing applies, move on. Do not chain multiple searches per brief.

This subsumes the "Memory injection happens at session start" line in the polling loop — session-start glob over `.claude/lessons/` is still required, but pre-queue search adds the fresh lookup right before the brief is written.

**Lesson corpus is indexed in PMD as of 2026-05-09** (commit landing this rule update). All 98 `.claude/lessons/feedback_*.md` files are imported as `memory_type: "pattern"` with `tags: "lesson,feedback"` via `scripts/sync-lessons-to-pmd.sh`. The hybrid search now returns lesson hits directly, not just adjacent eval/decision context.

**Re-sync after authoring a new lesson.** The import script is idempotent and additive — re-running it skips lessons whose title already exists. After committing a new lesson at `.claude/lessons/feedback_*.md`, run `bash scripts/sync-lessons-to-pmd.sh` to make it searchable. To force re-import after editing an existing lesson body: `sqlite3 .project-memory/memory.db "DELETE FROM memories WHERE title = '<frontmatter-name>';"` then re-run the script. (No automated hook is wired by design — the manual cadence matches lesson authorship cadence, ~1-2 per phase. A retro-watch signal is in place: if the next 2 phases show a "PMD missed lesson X because not yet synced" finding, build a `PostToolUse` hook on `.claude/lessons/feedback_*.md` Write/Edit.)

**Search mode (FTS5-only on the laptop until embeddings wired).** The brehon-fork PMD has the `memory_vectors` table provisioned but empty — the laptop's project-memory MCP server cannot reach Ollama at the EliteDesk. `memory_search_hybrid` auto-falls-back to FTS5 only. Practical implication: **prefer single distinctive keywords over multi-word natural-language queries**. `LemmyError`, `jsonb`, `e2e`, `migration` all return the right hits; `LemmyError doesn't implement std::error::Error in tests` returns nothing because FTS5 token-matches against common terms dilute the signal. When embeddings are wired (a future follow-up), multi-word queries will work via semantic recall — but the file-class injection table (sub-section below) is the deterministic backstop that doesn't depend on either FTS5 or embedding recall.

### Plan §5 complexity-score awareness (cargo-heavy threshold)

When the next pending §13 task is cargo-class (Brehon `[role:impl-task]` whose §15 DoD names `cargo check`, `cargo clippy --workspace`, or `cargo test --workspace`) AND the plan's §5.1 complexity score is `> 8`, the advisor consults the breakdown table before queueing. Per `feedback_complexity_score_pre_split.md`. Specifically:

1. The planner's split-or-proceed DQ should already be resolved (the planning gate refuses to ship a plan with score `> 8` and an unresolved planner-side DQ). Confirm by reading `.claude/decision-queue.json` resolved entries with `from: "planner"` referencing this plan.
2. If the dominant factor is `crates/lemmy_server/tests/e2e/*.rs` edits (≥2 e2e edits in this task or its cohort), append the `feedback_junior_worker_e2e_edit_hang.md` lesson to the brief's §3 Required reading. Per the "Pre-queue lesson check" cost discipline, this is a single citation, not a re-search.
3. If the dominant factor is migrations (≥2 migrations) AND the plan is pre-Shape-G, the validate-pending-laptop run should expect cargo+migration peak memory above ~6 GB on the laptop — note in the polling-loop output if the laptop is on battery + low. (The historical EliteDesk memory-headroom check is obsolete per the "Cargo never runs on the EliteDesk worker" sub-section below.)

This is mechanical: read score, read top factor, add citation. No additional DQ, no escalation. The complexity score is the planner's pre-impl signal; the advisor's job here is to make sure the lesson corpus consulted at brief-write time matches the score's top factor.

### Mandatory file-class lesson injection (mechanical)

The §5 complexity-score awareness check above triggers from the **plan's dominant factor** — a per-plan signal. That misses the case where a single task in the plan touches a file class that has a known footgun, even if that file class isn't the plan's overall dominant factor. (Empirical: the 2026-05-09 §G4 catch-fire on `error[E0277]: LemmyError` in a Task 1 e2e edit, where the plan's top factor flagged Tasks 1-5 e2e edits but the Junior worker's first edit hit the LemmyError `?` propagation pattern that has a documented mechanical fix in PMD; the brief omitted the lesson.)

To prevent recurrence: when authoring an `impl-task` brief, walk the §13 task's file list (the IMPLEMENT files plus any helper-creation files) against this table and inject every matching lesson into §3 Required reading. **No judgment call** — the table is canonical; if the file pattern matches, the lesson goes in.

| File pattern (in §13 IMPLEMENT or task creates new file matching) | Mandatory lesson(s) for §3 Required reading |
|---|---|
| `crates/server/tests/e2e.rs` (any edit, regardless of size) | `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md` |
| `crates/server/tests/e2e.rs` (≥2 edits in this task or its cohort) | + `feedback_junior_worker_e2e_edit_hang.md` |
| `crates/db_schema/migrations/**` (any new migration) | `feedback_lemmy_migration_runner.md`, `feedback_postgres_jsonb_canonicalization.md` (if migration touches JSONB) |
| Any new test file under `crates/*/tests/**` returning `Result<(), Box<dyn Error>>` | `feedback_lemmy_error_no_std_error.md` |
| Any handler under `crates/api/**/src/**` doing 2+ DB writes | `feedback_multi_write_handlers_need_transactions.md` |
| Any code adding `#[cfg(feature = "full")]` gates | `feedback_features_full_workspace_only.md`, `feedback_features_full_p_crate_incompatible.md` |
| Any code calling `pg_advisory_xact_lock` or void PG function | `feedback_pg_advisory_xact_lock_void_decode.md` |
| Any newtype addition under `crates/db_schema/src/newtypes/` | `feedback_newtype_locations_lemmy_db_schema_vs_file.md` |
| Any clippy fix that involves `-D warnings` + new code | `feedback_clippy_test_style.md`, `feedback_clippy_rerun_after_fix.md` |
| Any wrapper-script (`scripts/brehon/cargo-*.bat|sh`) edit | `feedback_wrapper_script_flag_silence.md`, `feedback_pq_sys_wrapper_env_propagation.md` |
| Any `.gitignore` / `.git/info/exclude` / hook addition | `feedback_settings_local_json_worktree_bootstrap.md` |

The `memory_search_hybrid` call from "Pre-queue lesson check" still runs (catches lessons not in the table). This file-class table is the **belt-and-braces backstop** for the well-known mechanical-fix patterns where missing the lesson costs an entire impl-task + ci-watcher cycle.

**How to apply when writing a brief:**

1. Read the §13 task's IMPLEMENT files list + any "creates" helper files.
2. For each file, walk the table top-to-bottom; inject every matching lesson path into §3 Required reading (deduplicated).
3. The brief commit body lists which mandatory lessons fired and why (one line each — keeps the audit trail).
4. The `memory_search_hybrid` call (per "Pre-queue lesson check") still runs after the table check; it catches non-mechanical / cross-cutting lessons.

**Maintenance:** when a new mechanical-fix pattern enters the §G4 allowlist (per "§G4 classifier" sub-section), check whether the failure correlates with a file class. If yes, add a row here so future briefs avoid the trip-up rather than recovering from it. The §G4 allowlist is reactive (auto-fix on failure); this table is preventative (don't fail in the first place). They share the same lesson corpus and grow together.

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
- **When the DQ to read lives on a non-trunk branch** (Junior worker branch, ci-watcher base branch), do NOT use `git show <branch-with-slashes>:<path>` — PowerShell underneath the Bash tool mangles the colon to a semicolon. Use `scripts/brehon/git-show-json.sh <ref> <path>` (resolves to a SHA first, captures to `$LOCALAPPDATA/Temp` on Windows / `/tmp` on Linux, and emits the path). Then read with `python -c "import io, json; d = json.load(io.open(r'<path>', encoding='utf-8')); ..."` — explicit `encoding='utf-8'` is mandatory because Python 3.14 default codec on Windows is cp1252. Per `feedback_windows_bash_python_git_show_tmp_traps.md`.
- **When reconciling DQ state for an in-flight sub-phase** (advisor session at session start, manual polling tick on a long-running phase, retro-time audit), prefer `scripts/brehon/resolve-dq-canonical.sh <phase>` over reading `.claude/decision-queue.json` from the laptop's local checkout. The canonical resolver unions phase-branch DQ with all open worker-branch DQs (per the auto-state's `current_cohort.members[].junior_id` + `fix_attempts`) and dedupes by entry id (worker-branch wins on collision = most recent state). Reading the laptop checkout alone misses entries on active worker branches that the EliteDesk daemon has pulled but not yet finalize-merged — the 2026-05-09 c-2 reconciliation incident: laptop saw pending=0 while live advisor session saw pending=2 because DQ #164+#165 lived only on worker-159's tip. The resolver also surfaces `_canonical_sources_consulted` in its output so retros can audit which refs contributed which entries. Used inside `/auto-phase` Phase 0.5 Step C; equally applicable when the manual polling loop reconciles state on resumed advisor sessions.
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

**Shape G note:** under Shape G (v1-validate-agent onward), cargo no longer runs locally for impl-task throughput — the workflow YAMLs at `.github/workflows/cargo-validate-*.yml` run cargo on GitHub-hosted runners (off-box, ephemeral). Forbidden windows are non-binding for Shape-G impl-task dispatch. They remain binding for: (a) ad-hoc local cargo validation by the advisor pre-plan-approval (DoD smoke test), (b) pre-Shape-G plan dispatches (v1-JM-d and earlier impl-task briefs that still run cargo locally), (c) any local diagnostic cargo run authorised by the user during a CR fix-in-PR cycle.

### Subagent enforcement (defence in depth)

The `impl-task` subagent's task-0 pre-flight check (per `.claude/agents/impl-task.md`) refuses to start work in a forbidden window and exits non-zero with `FORBIDDEN_WINDOW: <window>`. This catches the case where the advisor mistakenly queues during a forbidden window (e.g. cron table out of sync, daylight-saving edge case).

### When to override

Forbidden windows protect from contention, not from absolute prohibition. If the user explicitly authorises a forbidden-window run (e.g. one-off urgent fix during a crawl), the advisor:

1. Files a DQ entry citing the user's override.
2. Queues the task with a brief note: "user-authorised forbidden-window override per DQ #<id>".

Do not silently queue inside a forbidden window without a DQ trail.

### Cargo never runs on the EliteDesk worker (2026-04-28 incident)

**The EliteDesk worker (Junior daemon) does not run cargo for any plan, Shape-G or pre-Shape-G.** Per the 2026-04-28 task #47 incident: cargo check --workspace --features full ran on the EliteDesk worktree for >1 hour with sustained OOM-cascade risk (4.0 GB swap fully consumed, 2.4 GB available, contended with web-archive OpenSearch's 2.2 GB always-on JVM and impending NAS backups at 03:00 UTC).

Both validation modes route cargo OFF the worker:

- **Shape-G plans** (v1-validate-agent onward): cargo runs on GitHub-hosted runners. impl-task pushes branch, raises `kind: "validate-pending"` DQ entry; ci-watcher polls workflow, mutates entry. Already documented above under "Stage-shape orchestration → Each impl-task complete (under Shape G)".
- **Pre-Shape-G plans** (v1-JM-d and earlier): cargo runs on the **laptop** (the advisor session's CWD `C:\Users\barri\Developer\brehon-fork`). impl-task pushes branch, raises `kind: "validate-pending-laptop"` DQ entry naming the §15 DoD commands verbatim; the advisor (laptop) reads the entry on next polling tick, runs each command sequentially in the laptop's local checkout, and mutates the entry (see "validate-pending-laptop handler" below).

The historical "Memory headroom check" (free -h + web-archive-pause.sh) is now obsolete for cargo dispatch — kept only for informational reference if the user explicitly authorises a one-off local diagnostic cargo run on the EliteDesk during a CR fix-in-PR cycle, which should itself be rare. The default path for any cargo invocation is laptop-side.

### validate-pending-laptop handler

When a `kind: "validate-pending-laptop"` DQ entry appears in `pending[]` (raised by impl-task per `.claude/agents/impl-task.md` "Pre-Shape-G plans" sub-section):

0. **Pre-flight checks** (mandatory, before fetch):
   - **Clean working tree on laptop:** `git -C C:/Users/barri/Developer/brehon-fork status --short` must be empty. If dirty, the advisor cannot detached-HEAD checkout — surface to user: "laptop checkout is dirty (<files>); commit or stash before validate-pending-laptop runs". Do NOT auto-stash (user's in-progress work).
   - **Log directory exists:** `mkdir -p C:/Users/barri/.claude/logs/` (idempotent; first-run creates it).
   - **Concurrent-cargo serialization:** if another `validate-pending-laptop` DQ entry is currently being processed (advisor was already running cargo when this one arrived — most commonly because a `[P]` cohort dispatched and all members raised entries simultaneously), serialize: queue this entry behind the in-progress one. Two cargos against the same `target/` = lock contention + thrash. The rule of thumb: laptop processes validate-pending-laptop entries one at a time, in DQ entry-id order. (For Shape-G plans, ci-watcher already serializes per `[P]` cohort; this is the laptop equivalent.)
   - **Docker Desktop check (e2e only):** if the entry's `commands[]` includes any `cargo test ... --features full` or `cargo test ... e2e` (testcontainers-using e2e), check `docker ps` returns 0. If not running, surface to user: "Docker Desktop is not running on laptop; start it before continuing, or pick GH dispatch via the Phase 2 e2e user gate". cargo check / cargo clippy / `cargo test --no-run` (compile-only) do not need Docker — proceed without the check.

1. **Fetch the impl-task's worker branch** to the laptop:
   ```
   git -C C:/Users/barri/Developer/brehon-fork fetch origin <entry.branch>
   git -C C:/Users/barri/Developer/brehon-fork checkout origin/<entry.branch>
   ```
   The laptop checkout is detached-HEAD on the worker branch — no local edits, just for cargo to read the canonical source.

2. **Run each command in `entry.commands[]` sequentially.** Use `Bash` with `run_in_background: true` for cargo runs that take >5 min (cargo-check.sh ~8 min, e2e ~26 min). Capture stdout+stderr to per-command logs at `C:\Users\barri\.claude\logs\validate-laptop-<entry.id>-cmd-<n>.log`.

   - On each command, record exit code and capture log path.
   - If a command exits non-zero, **stop the chain** — do not run subsequent commands. Note which command failed.
   - If all commands exit zero, the entry passes.

3. **Mutate the DQ entry in place** (similar to ci-watcher's option-2 mutation):
   - On all-pass: set `result: "pass"`, `log_slice: null`, `failed_commands: null`, `answer: "All <N> validation commands passed locally on laptop."`, `answered_by: "advisor-laptop"`, `resolved_at: <now>`. Move entry from `pending[]` to `resolved[]`.
   - On any-fail: set `result: "fail"`, `log_slice: <last 100 lines of failing command's log>`, `failed_commands: [<command-string-that-failed>]`, `answer: <one-line summary>`, `answered_by: "advisor-laptop"`, `resolved_at: <now>`. Entry stays in `pending[]` for §G4 triage.

4. **Commit + push** the DQ mutation to `governance-v0`. Commit subject: `chore(decision-queue): advisor-laptop mutated DQ #<id> — <pass|fail> validate-pending-laptop`.

5. **Apply §G4 classifier on fail** same as Shape-G fail handling. Allowlist match → queue narrow fix-impl-task. Non-allowlist → catch-fire to user.

6. **Return laptop checkout to governance-v0** as the final step:
   ```
   git -C C:/Users/barri/Developer/brehon-fork checkout governance-v0
   git -C C:/Users/barri/Developer/brehon-fork pull --ff-only origin governance-v0
   ```
   The laptop must be on `governance-v0` (not detached on a junior/* branch) so that subsequent advisor commits (next brief, next DQ supersede annotation, next rule update) land on the trunk. Skip step 6 only if the user explicitly indicated they want the laptop kept on the worker branch (rare — typically for hand-debugging an impl-task's output).

**Wall-clock cost.** The laptop has more RAM than the EliteDesk's daemon cgroup and isn't contended with NAS/web-archive workloads. cargo check --workspace --features full on the laptop runs ~8-12 min cold, ~3-5 min warm. e2e runs ~26 min single-threaded. Use `run_in_background: true` and continue polling other tasks while cargo runs; do NOT block.

**E2E follows the same flow** — same DQ entry shape (or `kind: "validate-pending-laptop-e2e"` if e2e is the only validation), same mutation, same §G4 triage on fail.

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
- **Phase 2 e2e — local vs dispatch.** Per PR #105 (2026-04-28), `cargo-test-e2e.yml` is `workflow_dispatch`-only. Whenever Phase 2 e2e is needed (post-finalize-merge of an impl-task into `phase-v1-*`, or any phase-tip event that previously auto-fired e2e), surface a one-screen prompt to the user with two options: (a) **local** — `cargo test -p lemmy_server --test e2e --features full -- --test-threads=1` on the PC in `run_in_background`, ~26 min, zero billed; (b) **dispatch** — `gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-<phase>`, ~26 min billed, public log surface, ci-watcher polls. Wait for the user's choice before either path starts. Never auto-pick. Reason: user is monitoring the GH Actions minutes budget and wants explicit visibility on every e2e run.
- **Retro sign-off.** Author the retro per `feedback_retro_not_report`, `feedback_four_role_retro_signals`, and `feedback_retro_task_complexity_score` (per-task one-line `<files>/<commits>/<runtime-min>/<max-log-silence-min>` metric, aggregated in §5); surface to user. Wait for sign-off before phase transition.

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

## Schema-changing-spec retrofit gate (mandatory in plan-mode for shape changes)

Per `.claude/lessons/feedback_schema_changing_spec_retrofit_question.md`: when an advisor plan-mode session produces a plan that changes the shape of an existing artifact class (new section in a template, new marker in a section, new field in a schema, new required sub-section in a frontmatter), the advisor must call `AskUserQuestion` **once, before `ExitPlanMode`**, asking whether to retrofit existing artifacts.

Triggers:

- **New section in `*.template.md`** (e.g. §16a Stories block in plan.template.md).
- **New marker in an existing section** (e.g. `[P]` in §13 task headers).
- **New field in JSON/YAML schema** (e.g. `kind: "clarify"` in decision-queue.json).
- **New required sub-section in a frontmatter shape** (e.g. dogfood gate's `<rationale>` requirement on `.claude/commands/*.md`).

The question shape:

- "The new pattern applies forward-only to artifacts authored after this lands. Should I also retrofit the existing artifact(s) [<list>] in a follow-up commit?"
- Options: "Retrofit all" / "Retrofit named subset" / "Forward-only (no retrofit)"

The user's answer goes into the plan's "Out of scope" or a new "Retrofit scope" section verbatim. If user picks "Forward-only", the plan ships with an explicit "Pre-existing X are not affected; retrofit deferred indefinitely" line. If user picks retrofit, a Phase Z is added at the end of the implementation phases. Skipping the question is a process miss — the symptom shows up in the post-implementation retro as "should we retrofit X?" appearing as a deferred follow-up.

**When to skip:** plans that add purely additive functionality (new commands that don't change other commands' shape), bug fixes (the retrofit is the work itself), or plans explicitly limited to one artifact.

## Memory and lessons (one-system principle)

Advisor reads `.claude/lessons/` (same files Junior subagents read; cite by filename in briefs) and laptop PMD via memory MCP (search + `project_*` state). Retro-promoted lessons copy into `.claude/lessons/` in the retro commit.

## Stage-shape orchestration (the auto-decisions)

Per the c-inherited-dragon plan's "Stages of a sub-phase" map, the advisor knows what to queue next on each completion:

- **Brief authored, no planning task yet** → run `/brehon-clarify .claude/PRPs/briefs/<phase>-planning-N.md` → resolve every clarify-DQ entry (advisor-mode for evident, user-relay for judgment-heavy) → only then queue the planning task. Skipping `/brehon-clarify` on a planning brief is a process breach the advisor must justify in the planning task's commit body.
- **Planning complete** → run DoD smoke test → run watchpoint-specificity gate → surface to user → on user approval, queue `bm-cut` to make the phase branch
- **bm-cut complete** → queue impl per **Cohort dispatch** (next section): if plan §13 Task 1 (or first non-pre-flight task) carries `[P]`, compute the cohort and queue all members simultaneously; otherwise queue Task 1 alone.
- **Each impl-task complete (under pre-Shape-G plans, v1-JM-d and earlier)** → check plan task list; **if cohort still has pending peers, wait** for all-complete before computing next cohort; otherwise compute next cohort starting from the next pending task. If all tasks done, queue `bm-cut` follow-up (`chore(lint):` if needed) then `bm-pr`.
- **Each impl-task complete (under Shape G, v1-JM-e onward)** — two-phase validation per option (b) decision 2026-04-28:
  - **Phase 1 (workspace-check on `junior/*`):** impl-task already wrote a `kind: "validate-pending"` DQ entry post-push containing `workflow_run_id` (workspace-check run) + `branch` + `phase_task` + null `result`/`log_slice`/`failed_jobs`. Queue a `[role:ci-watcher]` Junior task with brief filled from the DQ entry's fields (template at `.claude/PRPs/templates/ci-watcher-brief.template.md`). The originating impl-task stays gated until ci-watcher resolves. ci-watcher runs ~10 sec model-time during the long-poll; the cargo work itself is GitHub-runner-side.
  - **Phase 2 (e2e, advisor-driven, off-Actions by default — 2026-04-28 minutes-budget audit):** `cargo-test-e2e.yml` no longer auto-triggers on `phase-v1-*` push. Instead, after Junior's daemon finalize-merges the impl-task worktree branch into the phase branch, the advisor's polling loop detects the new phase-branch tip on next `git fetch` and runs e2e **locally on the PC** in the brehon-fork worktree:
    1. `git -C C:/Users/barri/Developer/brehon-fork fetch origin --quiet && git -C ... checkout phase-v1-<phase> && git -C ... pull --ff-only`
    2. Open a `Bash` invocation in `run_in_background: true` mode using the wrapper: `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full -- --test-threads=1 > .claude/runlog/e2e-<phase>-<sha>.log 2>&1"` (~26 min wall-clock; cargo target/ is cached). **Never use `-p lemmy_server --features full`** — `lemmy_server` does not carry the `full` feature; use `--workspace` instead (`feedback_features_full_p_crate_incompatible.md`).
    3. Author the DQ entry up front: `kind: "validate-pending"`, `from: "advisor"`, `workflow_run_id: null` (no GH-Actions run id), `local_log_path: ".claude/runlog/e2e-<phase>-<sha>.log"`, `branch: "phase-v1-<phase>"`, `phase_task: <N>`, `result: null`. Commit subject: `chore(advisor): raise local e2e validate-pending for phase-v1-<phase> tip <sha>`.
    4. **Do NOT queue a ci-watcher Junior task** — there's no GH-Actions run to poll. Instead, on the next polling cycle the advisor checks the cargo background process status. When it exits, the advisor mutates the DQ entry directly: `result: "pass" | "fail"`, `log_slice` from the runlog tail (last failures block, ~150 lines), `answered_by: "advisor"`, `resolved_at`. Pass → entry → `resolved[]`; fail → §G4 classifier on the local log.
    5. **Audit-trail / PC-unavailable escape hatch:** if the user wants the GH-Actions log surface (e.g. for a contested run on `phase-v1-<phase>` tip `<sha>`), the advisor can dispatch the workflow manually: `gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-<phase>`. In that case the DQ entry shape reverts to the pre-2026-04-28 form (`workflow_run_id: <id>`, `local_log_path: null`) and a ci-watcher is queued exactly as for Phase 1. This path is reserved for explicit user request — default is local.

    Cohort advancement waits for BOTH workspace AND e2e validate-pending entries to be mutated to `result: "pass"` (workspace per Phase 1, e2e per Phase 2 — local OR dispatched).
- **ci-watcher complete** → read the mutated DQ entry. The entry's `kind` stays `"validate-pending"` regardless of result (kind records what was raised, not current state). If `result: "pass"` (entry now in `resolved[]`), advance per the cohort/task pipeline — for a workspace ci-watcher, await the Phase 2 e2e dispatch above; for an e2e ci-watcher, advance to the next §13 task. If `result: "fail" | "cancelled" | "timed_out"` (entry still in `pending[]`), run the §G4 classifier (see "§G4 classifier" sub-section below): allowlist match (≤3 file edits + clippy auto-fix or missing import or deprecated API) → queue narrow fix-impl-task brief; else → catch-fire to user with the `log_slice` + `failed_jobs`.
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
4. **YAML overlap check** (per `feedback_explicit_file_arrays_on_tasks.md`): for each cohort task, parse the **FILES** YAML block from §13 (`creates:` + `modifies:` arrays — the planner asserts `union(creates, modifies) == set(IMPLEMENT files)`). Compute pairwise intersections across cohort members. If any intersection is non-empty, **refuse the cohort and degrade to serial**: queue cohort members one at a time. Surface the overlap as: `cohort overlap detected: tasks <A>+<B> share <path> — degrading to serial`. Do not file a DQ for this — the planner's `[P]` marker was wrong, and the next retro should flag the planner miss; in-flight, serial dispatch is correct + safe. If the YAML block is missing on any cohort member (pre-V1 plans, or planner mistake), back-compat applies: skip the YAML check and trust the `[P]` marker (the cohort can still merge-conflict at finalize, which is the failure mode this check exists to prevent forward-going).
5. **Budget check**: estimate cumulative cargo memory for the cohort (each `cargo check --workspace --features full` ≈ 6 GB peak per the EliteDesk's deployed cap; serial budget is `MemoryMax=10G`). If `cohort_size × per_task_peak > 10 GB`, **degrade to serial** — queue the cohort tasks one at a time as if non-`[P]`. Per `feedback_resource_budget_pre_queue.md`. Note the degrade in the polling-loop output: `cohort degraded to serial: budget exceeded (<size> tasks × <peak> GB > 10 GB)`. Under Shape G the budget check is non-binding (cargo runs off-box).
6. **Forbidden-window check**: re-evaluate the forbidden-windows table for the cohort's expected start time. If any cohort task would start in a forbidden window, defer the entire cohort per the existing self-defer rule. Cohort dispatch and forbidden-window deferral compose naturally — the advisor defers the whole cohort, not individual tasks.
7. **Queue every cohort task simultaneously** via parallel `mcp__junior-brehon__create_task` calls (single message, multiple tool uses). Each task gets its own Junior worktree per `feedback_parallel_agents_one_worktree_per_agent.md`. Brief paths are unique per task (`.claude/PRPs/briefs/<phase>-impl-<N>.md`).
8. **Wait for all cohort members to reach complete or failed** before computing the next cohort. A failed task in the cohort blocks advancement — the advisor surfaces the failure (catch-fire if it's a hard-refusal violation) and does not queue beyond the failure boundary until resolved.
9. **On cohort completion (all members `complete` and validated):** run the cohort handover aggregation step (next sub-section) before computing the *next* cohort. The aggregated handover populates the next cohort's brief §3a "Handover from prior cohort" before any of those tasks gets queued.

### Cohort dispatch refusals

- **Never queue a cohort whose tasks have not all been clarified.** The clarify gate runs once per planning brief, but if a cohort's tasks reference §13 entries that surfaced new ambiguity post-clarify (e.g. a brief edit introduced overlap), file a DQ pending entry and re-run `/brehon-clarify` on the affected brief.
- **Never queue a cohort during a forbidden window**, even partially. Either the entire cohort defers or none does.
- **Never re-queue a cohort task that already shows running.** Junior's task IDs are unique per worktree; re-queueing creates a duplicate worktree and conflicting branch names.
- **Never queue a `[P]` task whose IMPLEMENT files overlap a non-`[P]` task that's still running**. The `[P]` marker is a planner-side promise of file disjointness within the cohort, not across cohort boundaries — if the prior cohort's barrier hasn't completed, wait.
- **Never queue a cohort with a non-empty YAML overlap intersection** without first degrading to serial. Per the YAML overlap check above (Cohort dispatch sequence step 4). Mechanical: `intersect(union(creates, modifies)_taskA, union(creates, modifies)_taskB) != ∅` → degrade. Trust the YAML over the `[P]` marker when they disagree.

### Cohort handover aggregation

Per `feedback_handover_trailer_cohort_propagation.md`. Once all cohort members reach `complete` (and under Shape G, all corresponding `validate-pending` DQ entries mutated to `result: "pass"` for both Phase 1 workspace and Phase 2 e2e per the option-b two-phase validation), the advisor populates the *next* cohort's brief §3a "Handover from prior cohort" before queueing any task in that next cohort.

Sequence:

1. For each cohort member's commit on `phase-<phase>`, parse the commit body for the `HANDOVER:` YAML trailer (per `.claude/agents/impl-task.md` "Per-task commit shape"). Use `git log -1 --format=%B <sha>` and a YAML parser. If a cohort member's commit has no trailer, that's a no-op for the trailer (single-task non-`[P]` exception, or the impl-task subagent skipped it — note in polling output but do not catch-fire; missing trailer is degraded handover, not failure).
2. Aggregate the parsed trailers into one block matching the §3a schema in `impl-task-brief.template.md`:

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

3. Locate the next cohort's brief paths (one per cohort task — `.claude/PRPs/briefs/<phase>-impl-<M>.md`). For each, Edit §3a in place, replacing `(none — first cohort)` or `(none — prior task non-[P])` with the aggregated block. The advisor commits the brief edits with subject `chore(advisor): inject prior-cohort handover for <next-cohort-tasks>` (matches `^(chore|docs)\((advisor|decision-queue)\)` per attribution-integrity).
4. Push the brief commits to `governance-v0` so Junior worktrees pick them up cleanly.
5. Proceed to next-cohort dispatch (Cohort dispatch sequence step 1, restarting with the next pending §13 task).

**Skip the aggregation step if** the next cohort is empty (i.e. the prior cohort was the last cohort before the retro task). The retro task reads §3a as `(none — last cohort)`.

**Single-task cohorts** (where one §13 task with `[P]` is followed by a non-`[P]` task and the cohort collapses to just that one `[P]` task) still aggregate handover — the next non-`[P]` task gains the prior `[P]` task's keyDecisions. The trailer is the unit of handover; cohort size doesn't change the rule.

### Plans without `[P]` markers (back-compat)

If a plan §13 has no `[P]` annotations (legacy plans pre-this-rule, or plans where the planner judged no parallelism was safe), every task is treated as non-`[P]` and dispatched serially. The cohort-dispatch logic does not broaden serial dispatch into accidental parallel — `[P]` must be explicit.

### Cohort dispatch under Shape G (parallel validate-pending)

Under Shape G (v1-JM-e onward), cohort members each enter
`kind: "validate-pending"` simultaneously after their respective
push — one Phase-1 workspace-check workflow run per cohort task,
fanned out on GitHub-hosted runners. The advisor dispatches one
`[role:ci-watcher]` Junior task per `validate-pending` entry. Each
ci-watcher mutates its paired entry on completion (per option 2;
the entry's `kind` stays `"validate-pending"`, but `result`/
`log_slice`/`failed_jobs`/`answer`/`answered_by`/`resolved_at` are
populated, and the entry moves `pending[]` → `resolved[]` only on
`result: "pass"`).

Cohort advancement waits for **all** Phase-1 cohort members to reach
`result: "pass"` (entries in `resolved[]`). A single member with
`result: "fail" | "cancelled" | "timed_out"` (entry remaining in
`pending[]`) blocks advancement and triggers the §G4 classifier
per the validate-stage Stage-shape rule. If multiple cohort members
fail simultaneously, classify each independently — auto-queue
allowlist matches as parallel fix-impl-tasks (each forming its own
[P]-marker degenerate cohort), surface non-allowlist failures to
user as a single catch-fire bundle.

After all Phase-1 cohort members pass and the daemon finalize-merges
each into the phase branch, the advisor raises a SINGLE Phase-2 e2e
`validate-pending` DQ entry for the post-finalize phase-branch tip
(one e2e run per cohort barrier, not per cohort member — option (b)
e2e fires once when the phase-branch tip moves). Cohort advancement
to the *next* cohort waits on this e2e ci-watcher resolving with
`result: "pass"` as well.

## §G4 classifier

Per `.claude/PRPs/plans/v1-validate-agent.plan.md` §4 watchpoint #7
+ §10.9. When a `validate-pending` DQ entry is mutated to
`result: "fail" | "cancelled" | "timed_out"` and remains in `pending[]`,
the advisor reads its `result`, `log_slice`, and `failed_jobs`, then
applies the classifier:

**Allowlist (auto-queue narrow fix-impl-task, ≤3 file edits):**

| Failure signature | Auto-fix | Source lesson |
|---|---|---|
| `clippy::doc_lazy_continuation` warning | reword + mid-paragraph "and" | `feedback_clippy_doc_lazy_continuation_in_doc_comments.md` |
| `error[E0432]: unresolved import` | add the missing `use` per the suggestion | n/a (mechanical) |
| `warning: use of deprecated <api>` | replace with the suggested replacement | n/a (mechanical) |
| `error[E0277]: ?` couldn't convert `LemmyError` (or `LemmyResult<T>`) to `Box<dyn Error>` at a `?` propagation site | wrap the call with `.map_err(\|e\| format!("{e}").into())` per the lesson; verify the test fn signature is `Result<(), Box<dyn Error>>` | `feedback_lemmy_error_no_std_error.md` |
| `error[E0277]: trait bound \`<T>: <Trait>\` not satisfied` where the lesson corpus has a citation | apply the recipe per the cited lesson | search `.claude/lessons/` for the failing trait + type before classifying |
| `clippy::map_err_ignore` (E0277-adjacent) | rename `\|_\|` → `\|_e\|` per `feedback_clippy_map_err_ignore_pattern_rename.md` (when authored) | mechanical |
| `error: cannot find macro \`<name>\` in this scope` | add the missing `use` from the macro's home crate | n/a (mechanical) |
| `error[E0599]: no method named \`<name>\`` (when method is on a re-exported trait) | add the missing `use` for the trait | n/a (mechanical, but verify the trait isn't intentionally hidden) |

For an allowlist match, the advisor authors a narrow fix-impl-task
brief at `.claude/PRPs/briefs/<phase>-fix-impl-<n>.md` containing:
the failed-job log slice (≤200 lines), the specific file:line cited
by the lint, the auto-fix recipe from the source lesson (or
mechanical replacement), and a hard cap "≤3 file edits". The brief
is dispatched as a normal `[role:impl-task]` Junior task; the
resulting commit lands on the phase branch and re-triggers the
workflow.

**Non-allowlist (catch-fire to user):**

- compile errors (any `error[E*]` other than `E0432`)
- test failures (panics, assertion fails, e2e flakes, testcontainers
  issues)
- timeout / OOM / runner death
- any failure whose log slice doesn't match a row in the allowlist

Surface as: "validate-failed on `<branch>` (workflow run `<id>`):
non-allowlist failure. Failed jobs: `<failed_jobs>`. Log slice
attached. Surfaced to user — no auto-fix attempted."

The allowlist is **conservative by design** (per
`feedback_principles_not_rules.md` — guidance over rigid rules).
Grow it only on retro evidence: if a CR-triage cycle classifies a
non-allowlist failure as "this could have been auto-fixed", record
it in the retro §5 watch-items and add to the allowlist on the next
sub-phase's plan if the pattern reproduces.

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
- A workflow run exceeds the 60-min ci-watcher cap → ci-watcher mutates the paired `validate-pending` entry to `result: "timed_out"` (entry stays in `pending[]`). Surface to user with the workflow run id and the elapsed wall-clock; do not auto-rerun.
- ci-watcher's `gh run watch <id> --exit-status` returns an exit code not enumerated in the empirical exit-code table at `.claude/agents/ci-watcher.md` "Empirical exit-code table" — surface as classifier-miss with the observed exit code, the workflow run id, and the run's `gh run view <id> --json status,conclusion` snapshot. The exit-code table is grow-on-evidence; record the new pair (exit_code → conclusion) and update the table at retro time.

For each, include the catch-fire reason and the rule it violated in the surfaced message.

## What this rule does NOT cover

Companion files: `.claude/agents/<name>.md` (subagent contracts), `.claude/rules/branch-manager.md` + `.claude/commands/bm/<verb>.md` (BM mechanics), `.claude/rules/decision-queue.md` (DQ attribution, auto-loaded), `homeserver/.claude/advisor-context-phase-<N>.md` (phase-specific texture, session-start read).
