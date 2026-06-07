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

1. **Resolve a validation worktree on `entry.branch` — NEVER a bare
   checkout in the canonical tree** (the 2026-06-07 m2-late-1 T1
   incident; see §"Why a worktree, not a checkout" below). `git fetch
   origin <entry.branch>` first, then:
   - **Mode A (a lane worktree exists at `brehon-fork-<lane>` on
     `entry.branch`):** `cd` there. The lane worktree IS the validation
     surface. Confirm its HEAD: `git -C <lane-path> rev-parse HEAD` ==
     `git rev-parse origin/<entry.branch>`; if behind,
     `git -C <lane-path> merge --ff-only origin/<entry.branch>`.
   - **Mode B (no lane worktree):** create a throwaway validation
     worktree off the up-to-date remote ref:
     `git worktree add ../brehon-fork-validate-<entry.id>
     origin/<entry.branch>`. **Then run the lane bootstrap** (else
     cargo fails on `lemmy_email` with `Os code 3 NotFound` — per
     CLAUDE.md "Lane worktree bootstrap"): `git -C
     <validate-path> submodule update --init --recursive` and copy
     `.mcp.json` / `.env` / `.claude/settings.local.json` from
     canonical. Remove the throwaway in step 6.
   The bare `git checkout origin/<branch>` (detached-HEAD in the
   canonical tree) is **abolished** — it violates
   `multi-lane-worktree.md` hard refusal #1 and silently reverts to
   `governance-v0` across a `/compact` boundary, which is exactly how
   the T1 run produced 8 failed steps against the wrong tree.
2. **Per-command branch assertion (mandatory, before EACH command).**
   Immediately before running each command in `entry.commands[]`,
   assert the validation worktree is still on the right tip — a
   `/compact` mid-chain can revert working-tree state without touching
   conversation state:
   ```bash
   ACTUAL=$(git -C <wt-path> rev-parse HEAD)
   EXPECTED=$(git rev-parse origin/<entry.branch>)
   [ "$ACTUAL" = "$EXPECTED" ] || { echo "BRANCH MISMATCH: $ACTUAL != $EXPECTED — STOP"; exit 1; }
   ```
   On mismatch, do NOT run the command; surface to user. This is the
   belt-and-suspenders check that would have caught the T1 failure at
   step 4 instead of step 11.
3. **Windows wrapper substitution (mandatory on Windows).** The DQ
   `commands[]` are authored Linux-shaped (bare `cargo …`) by the
   Linux-daemon impl-task worker. Before running on Windows, substitute
   the wrapper — bash `$PATH` does NOT reach the Windows PE DLL loader,
   so bare cargo dies `STATUS_DLL_NOT_FOUND` (0xc0000135). Mapping:
   | DQ command shape | Run instead (Windows) |
   |---|---|
   | `cargo run -p lemmy_diesel_utils … migration run` (or any diesel migration runner) | `bash scripts/brehon/migrate-roundtrip.sh` — it self-provisions a throwaway Postgres, OS-switches to `migrate-roundtrip-cargo.bat`, and does forward+idempotency. Run from the **phase-branch worktree** (it diffs vs `origin/governance-v0` to find the new migration — from the wrong branch it correctly reports "no new migrations; exit 0", which is a **wrong-branch signal, not a no-op to route around**). |
   | `./scripts/brehon/cargo-check.sh --workspace --features full` | `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > <log> 2>&1"` |
   | `cargo test … --test e2e …` | `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1"` (see §"Windows invocation") |
4. Run each command (post-substitution) sequentially. `Bash`
   `run_in_background: true` for runs >5 min (`cargo-check.sh` ~8 min
   cold / 3-5 min warm; e2e ~26 min). Capture to
   `C:\Users\barri\.claude\logs\validate-laptop-<entry.id>-cmd-<n>.log`.
   Non-zero exit → stop chain. **Migration tasks: snapshot
   `SELECT MAX(version) FROM __diesel_schema_migrations` before+after
   and confirm it advanced** — a silent `exit 0` on "nothing to apply"
   is ambiguous (it can mean wrong branch OR already-applied); the
   before/after delta disambiguates.
5. Mutate the DQ entry in place per the canonical mutation shape
   (see decision-queue.md ref above). Failure stays in `pending[]`
   for §G4 triage; pass moves to `resolved[]`.
6. Commit + push to `governance-v0`. Subject: `chore(decision-queue):
   advisor-laptop mutated DQ #<id> — <pass|fail>
   validate-pending-laptop`. **Mode B teardown:** `git worktree remove
   ../brehon-fork-validate-<entry.id>` (use `--force` only if the
   submodule worktree blocks plain remove — per
   `feedback_worktree_remove_force_for_submodules.md`). Mode A: leave
   the lane worktree in place.
7. On fail, run §G4 classifier (advisor-orchestrator.md §5.3):
   allowlist → narrow fix-impl-task; non-allowlist → catch-fire.

### Why a worktree, not a checkout (2026-06-07 m2-late-1 T1 RCA)

The T1 `validate-pending-laptop` run failed 8 consecutive steps
because the advisor ran `git checkout phase-m2-late-1` in the
**canonical** `brehon-fork` tree (Mode B), a `/compact` boundary
silently reverted the working tree to `governance-v0`, and every
subsequent cargo/migration command then ran against the wrong tree.
The "no new migrations vs governance-v0; exit 0", the missing
`sanction_*` tables in the regenerated schema, the DLL-not-found — all
downstream of the one wrong-branch error. A **worktree HEAD is
independent of the canonical tree's HEAD**, so it is immune to
compact-boundary reversion; combined with the per-command assertion
(step 2) this makes the failure class structurally impossible rather
than merely detectable. Full trace:
`.claude/PRPs/debug/m2-late-1-t1-validate-pending-advisor-session-trace.md`.

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
