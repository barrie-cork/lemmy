# Advisor validation, classification, recovery

Externalised from `.claude/rules/advisor-orchestrator.md` §5.1 + §5.2
on 2026-05-22 (rule-trim pass). The rule file kept §5.3 (the G4
classifier table — load-bearing for fix-impl brief verbatim
blockquotes), §5.4 (DQ triage decision tree), §5.5 (retro-bypass
observability), §5.6 (catch-fire procedures). This file holds the
forbidden-execution-window table and the validate-pending-laptop
handler procedure — both fire infrequently (forbidden windows are
non-binding under Shape G, laptop handler runs only on pre-Shape-G
or Phase-2 e2e local).

Cite this file as `advisor-validation.md §"Forbidden execution windows"`
/ `advisor-validation.md §"validate-pending-laptop handler"`.

> **Loading note:** this file is NOT auto-loaded at session start.
> Read on-demand when (a) a `validate-pending-laptop` DQ entry
> appears, (b) a forbidden-window deferral is needed (rare; Shape G
> non-binding for most cases), or (c) authoring a brief whose §15
> commands run on the laptop.

## Forbidden execution windows

EliteDesk shares cron-driven workloads (NAS backups, web-archive
crawls, weekly review) with Brehon Junior tasks. Repeated OOM
cascades (2026-04-27) confirm temporal isolation > spatial isolation.
Source-of-truth: `homeserver/docs/troubleshooting-laptop-elitedesk.md`
"Temporal isolation" section.

| Window (UTC) | Why |
|---|---|
| Daily 02:55–04:15 | NAS backup chain (03:00, 03:15) + web-archive `govie-search` (03:00, 03:30) |
| Sunday 01:55–02:35 | HSE crawl (02:00) + `junior-weekly-review.sh` (02:30) |
| Sunday 03:55–04:30 | `restore-drill.timer` (04:00) |
| Wednesday 03:55–04:15 | `web-archive govie-cdx` (04:00) — subset of daily |

**Recommended Brehon execution windows:** Primary 16:00–02:30 UTC
(10.5 h, evening/overnight). Secondary 04:30–14:59 UTC (10.5 h,
post-crawl, pre-evening).

**Advisor enforcement** — before queueing any new `impl-task`:

1. Compute next "safe" minute (end of current forbidden window).
2. Note deferral in polling output: `deferring <task-slug> until <HH:MM UTC>`.
3. Re-check on next poll. Queue once window closes. **No DQ for routine deferrals.**

Mechanical, not heuristic — read table + `date -u`.

**Subagent enforcement (defence in depth):** the `impl-task` subagent's
task-0 pre-flight check refuses to start in a forbidden window, exits
non-zero with `FORBIDDEN_WINDOW: <window>` (per
`.claude/agents/impl-task.md`). Catches advisor-mistaken queues (e.g.
cron table out of sync, DST edge case).

**Shape G note:** under Shape G (v1-validate-agent onward), cargo runs
on GitHub-hosted runners — forbidden windows non-binding for Shape-G
impl-task dispatch. Still binding for: (a) ad-hoc local cargo by
advisor pre-plan-approval (advisor-orchestrator.md §3.4 DoD smoke
test), (b) pre-Shape-G plan dispatches (v1-JM-d and earlier), (c) any
local diagnostic cargo authorised by user during a CR fix-in-PR
cycle.

### When to override

Forbidden windows protect from contention, not absolute prohibition.
If user authorises a forbidden-window run:

1. File a DQ entry citing the user's override.
2. Queue with brief note: "user-authorised forbidden-window override
   per DQ #<id>".

Do not silently queue inside a forbidden window without a DQ trail.

### Cargo never runs on the EliteDesk worker

Per 2026-04-28 task #47 incident: `cargo check --workspace --features
full` ran for >1 h with sustained OOM-cascade risk. Both validation
modes route cargo OFF the worker:

- **Shape-G plans:** GitHub-hosted runners. impl-task pushes branch,
  raises `kind: "validate-pending"`; ci-watcher polls workflow,
  mutates entry.
- **Pre-Shape-G plans:** the **laptop** (advisor's CWD
  `C:\Users\barri\Developer\brehon-fork`). impl-task pushes branch,
  raises `kind: "validate-pending-laptop"` naming §15 DoD commands
  verbatim; advisor reads on next poll, runs each command
  sequentially, mutates the entry. See §"validate-pending-laptop
  handler" below.

## validate-pending-laptop handler

When a `kind: "validate-pending-laptop"` (or `*-laptop-e2e`) entry
appears in `pending[]`, the advisor (laptop session) runs the §15
commands locally. Mutation shape, log-slice rules, kind enum, and
§G4 fail handling: see `.claude/rules/decision-queue.md` §"ci-watcher
mutation pattern" + §"Two-phase validation under Shape G" (mutation
is identical; only the runner identity differs —
`answered_by: "advisor-laptop"` instead of `"ci-watcher"`).

### Pre-flight (mandatory, before fetch)

- Clean working tree (`git -C C:/Users/barri/Developer/brehon-fork
  status --short` empty); dirty → surface to user, do NOT auto-stash.
- `mkdir -p C:/Users/barri/.claude/logs/` (idempotent).
- Concurrent-cargo serialization: if another `validate-pending-laptop`
  is in-flight (`[P]` cohort fan-out), process this one behind it in
  DQ id order — two cargos on the same `target/` = lock + thrash.
- Docker Desktop check (e2e only): if `commands[]` includes
  `--features full` testcontainers paths, `docker ps` must return 0;
  not running → surface "start Docker Desktop or pick GH dispatch via
  Phase 2 e2e user gate". `cargo check`/`clippy`/`test --no-run` skip
  the check.

### Sequence

1. `git fetch origin <entry.branch>` then `git checkout
   origin/<entry.branch>` (detached-HEAD; no edits, just cargo source).
2. Run each command in `entry.commands[]` sequentially. `Bash`
   `run_in_background: true` for runs >5 min (`cargo-check.sh` ~8 min
   cold / 3-5 min warm; e2e ~26 min). Capture to
   `C:\Users\barri\.claude\logs\validate-laptop-<entry.id>-cmd-<n>.log`.
   Non-zero exit → stop chain.
3. Mutate the DQ entry in place per the canonical mutation shape
   (see decision-queue.md ref above). Failure stays in `pending[]`
   for §G4 triage; pass moves to `resolved[]`.
4. Commit + push to `governance-v0`. Subject: `chore(decision-queue):
   advisor-laptop mutated DQ #<id> — <pass|fail>
   validate-pending-laptop`.
5. On fail, run §G4 classifier (advisor-orchestrator.md §5.3):
   allowlist → narrow fix-impl-task; non-allowlist → catch-fire.
6. `git checkout governance-v0 && git pull --ff-only origin
   governance-v0` (skip only if user wants laptop kept on worker
   branch for hand-debug).

### Phase 2 e2e (advisor-driven, off-Actions default — 2026-04-28 minutes-budget audit)

Advisor raises the entry up front (`from: "advisor"`,
`workflow_run_id: null`, `local_log_path:
".claude/runlog/e2e-<phase>-<sha>.log"`, `branch: "phase-v1-<phase>"`,
`phase_task: <N>`, `result: null`). Subject: `chore(advisor): raise
local e2e validate-pending for phase-v1-<phase> tip <sha>`. No
ci-watcher dispatch (nothing on GH to poll). On bg cargo exit,
advisor mutates directly: `answered_by: "advisor"`, `resolved_at`,
`log_slice` from runlog tail (~150 lines failures block).

**Escape hatch (explicit user request only):** `gh workflow run
cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-<phase>`
— entry reverts to pre-2026-04-28 shape (`workflow_run_id: <id>`,
`local_log_path: null`); ci-watcher queued as for Phase 1.

### Falsification check (pre-fix, post-fail)

Before raising a fix-impl-task after a `validate-pending-laptop` fail:

1. Re-read the log slice. Identify the **proximate error** (the line `cargo` or the test printed) vs the **assumed root cause** (what you think caused it).
2. If the assumed cause names a specific file, function, or env var: `grep` for it in the actual log; confirm it appears in the failure path, not just in a different context.
3. Check `entry.commands[]` — confirm the command that failed is the one you think failed (multi-command chains can mask which command triggered the non-zero exit).
4. If the proximate error does NOT match the assumed root cause, surface the discrepancy to the user before queuing a fix. Per `feedback_falsifiable_hypothesis_before_structural_fix.md` + §5.4 DQ falsifiable-hypothesis gate.

### Windows invocation (mandatory — 2026-05-09 RCA)

Use `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test
e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> ||
echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true`.
Never bare `cargo test` on Windows — libpq.dll requires the bat
wrapper's vcpkg PATH setup; bash PATH export does not propagate to
the Windows PE DLL loader. Never `-p lemmy_server --features full`
— `lemmy_server` has no `full` feature; use `--workspace`. See
`feedback_windows_e2e_requires_bat_wrapper.md` and RCA at
`.claude/PRPs/reports/rca-phase2-e2e-invocation-failure-2026-05-09.md`.

## See also

- `.claude/rules/advisor-orchestrator.md` §5.3 — G4 classifier
  (allowlist table + non-allowlist + verbatim-blockquote gate;
  stays in-rule because briefs verbatim-quote it).
- `.claude/rules/advisor-orchestrator.md` §5.4 — DQ triage decision
  tree.
- `.claude/rules/advisor-orchestrator.md` §5.5 — retro-bypass
  observability.
- `.claude/rules/advisor-orchestrator.md` §5.6 — catch-fire
  procedures.
- `.claude/rules/decision-queue.md` §"ci-watcher mutation pattern" +
  §"Two-phase validation under Shape G" — canonical mutation shape.
- `feedback_windows_e2e_requires_bat_wrapper.md` — Windows
  invocation source.
