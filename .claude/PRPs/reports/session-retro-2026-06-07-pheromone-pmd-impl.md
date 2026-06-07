# Session retro — 2026-06-07 — pheromone-pmd-impl

**Harness:** claude-code
**Session window:** ~2026-06-07 09:40 → 11:15 UTC (~95 min active)
**Branch at start:** `governance-v0` (canonical checkout, later showed `phase-m2-late-1`)
**Branch at end:** `phase-m2-late-1` (CWD); commits landed on `governance-v0`
**Files touched:** ~10 (2 MCP src, 4 tooling scripts, 2 specs, 1 .gitignore, 1 EliteDesk settings.json) + 2 PMD DBs migrated
**Commits:** 6 (MCP repo: `bbeccfb`, `1d226e5`, `a59a2e0`; brehon-fork: `eb082abd3`, `2f54f81e9`, `6ee8288cb`) + 1 retro eval

## TL;DR

Implemented the read-pheromone stigmergic ranking spec (`mcp-pmd-read-pheromone.md`)
into `project-memory-mcp`, shipped it live to **both** PMDs (laptop + EliteDesk),
then on follow-up requests added the §7b FTS tie-breaker, a 24h health monitor,
per-event `read_events` telemetry for weight tuning, and a decide-later options doc
for true relevance-labeling. The most load-bearing finding: **specs encode stale
runtime facts** — the spec's "PID 5900, restart manually" was wrong on two counts
(it's an NSSM service needing elevation), and the EliteDesk MCP turned out to be a
much older lineage than the spec/laptop assumed, turning a "2-file copy" into a
full version uplift. The top carry-forward: **verify runtime/deploy facts against
the live system before acting on a spec's deploy section** — every deploy claim is
a hypothesis. Top change proposal: a reusable `ssh-sql` helper (base64-over-SSH)
to kill the recurring Windows-OpenSSH quote-stripping trap that bit the monitor.

---

## What surprised us

- **The spec's runtime facts were stale on multiple axes.** "PID 5900, restart
  `node dist/index.js` manually" → reality was an NSSM Windows service
  `pmd-http-mcp` needing *elevated* restart (which this non-interactive session
  can't do). Row counts (579) were stale (live 769). None broke the patch, but
  every deploy instruction was a hypothesis, not a fact.
- **The EliteDesk MCP was a far older lineage than anyone assumed.** It had NO
  hybrid/semantic search at all (158-line `search.ts` vs the laptop's 425) and
  lacked the `busy_timeout` fix. The user's "deploy there too" turned from a
  2-file copy into a full version uplift — a much bigger blast radius surfaced
  only because I diffed baselines before copying.
- **Lazy migration meant the live laptop columns appeared "missing" right after a
  successful restart.** The migration runs on first `getDb()`, and no MCP call had
  hit the server yet — so a raw `sqlite3` check showed no columns, looking like a
  failure. One real MCP search triggered the migration and resolved it. Correct by
  design, but momentarily alarming.
- **My first monitor draft cried wolf:** `fts_storm=5` looked like the exact bug
  the tripwire was built to catch. It was a false positive — rows *written* today
  (created_at==updated_at==today) legitimately matched a too-loose condition. The
  monitor I built to catch a bug nearly reported a phantom one.
- **The EliteDesk PMD was NOT at zero vectors** as an early subagent report
  claimed (it cited the spec's stale numbers). It had 225/256. A reminder that
  even a thorough subagent inherits stale inputs if it reads specs over live state.
- **Windows OpenSSH silently strips shell quotes** before the remote bash sees
  them — broke the monitor's EliteDesk SQL twice (parens → bash syntax error)
  until I switched to base64-encoding the SQL over the wire.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add a reusable `ssh-sql` / `ssh-remote-cmd` helper that base64-encodes the payload over the wire, to `.claude/tools/` (or a scripts lib) | Kills the Windows-OpenSSH quote-stripping class for any future remote-SQL/remote-script-with-parens call; the monitor already needs it | minor | 2× this session (monitor metrics + events query), ≥3× in prior sessions per `feedback_windows_bash_python_git_show_tmp_traps.md` cluster |
| 2 | New lesson `feedback_spec_deploy_facts_are_hypotheses.md`: a spec's deploy/runtime section (PIDs, restart commands, row counts, process manager) is a hypothesis — verify against the live system (`Get-Service`, `Get-NetTCPConnection -OwningProcess`, `git log` on the deploy target) BEFORE acting | Prevents acting on stale deploy instructions; would have pre-empted the NSSM/elevation surprise + the EliteDesk-lineage surprise | minor | 2× this session (NSSM restart + EliteDesk lineage), pairs with existing `feedback_runbook_audit_drift_post_event_check.md` |
| 3 | When deploying code to a second host that is NOT a git checkout, ALWAYS diff the target's baseline against the source's pre-change baseline before copying — fold into change #2's lesson or `feedback_sweep_all_repos.md` | Catches version-drift "uplift not copy" situations before a partial copy regresses the target | minor | 1× this session (EliteDesk), 0× prior |
| 4 | Monitor anomaly tripwires need a written false-positive test: the `fts_storm` condition must exclude `created_at >= today` (rows written today) — document the FP class in the monitor header (done) AND as a one-line check pattern | Prevents health monitors from reporting phantom bugs; the corrected query is the canonical form | minor | 1× this session, but high-cost-if-missed (a crying-wolf monitor gets ignored) |

## What to carry forward

- **Verify-before-trusting applied to live state, repeatedly and well.** Every
  claim got checked against reality: SQLite math built-ins probed at runtime (not
  assumed from version), `better-sqlite3` bundled version confirmed, NSSM config
  read directly, EliteDesk baseline diffed before copy. This discipline caught all
  five surprises above before they became damage. Keep it.
- **Temp-DB-copy validation for any DB-mutating change.** The §9 + §7b + read-events
  suites all ran against a `Copy-Item` of the live DB — zero live-data risk, full
  coverage. Used 3× cleanly this session. Make it the default for any PMD schema work.
- **AskUserQuestion at genuine forks, not for confirmation.** Used it at real
  decision points (deposit scope, deploy target, validation method, optimization
  depth, pickup timing) — each answer changed what I did next. Never used it to ask
  "is this okay." Clean signal-to-noise.
- **Best-effort telemetry wrapped in try/catch** so it can never break the hot path
  — the `logReadEvents` pattern (swallow errors, telemetry is not allowed to fail a
  search). The right default for any observability added to a critical path.
- **Honest scoping of what a deliverable can/can't do.** Twice I corrected an
  over-claim before it shipped: "fully tested" (it wasn't — re-ran against the live
  server) and "the monitor can optimize weights" (it can't — only health; built the
  per-event layer + speced the relevance-labeling gap rather than overselling).
- **Atomic stage+commit+verify burst to beat a concurrent-writer race on a shared
  `.git/`** *(added 2026-06-07, promotion-execution follow-on)*. While committing this
  retro's 3 promotion items, the canonical `brehon-fork` `.git/` was being driven by a
  second live session (m2-late-1): between my `git add` and `git commit`, the other
  session committed twice and silently reset the index, so my first commit landed as a
  no-op ("no changes added to commit"). The fix that worked: stage ONLY my own files +
  `git commit -F` + `git log -1`/`git show --stat` verify, chained with `&&` in ONE
  uninterrupted shell invocation — no separate add/commit round-trips for the race to
  slip between. The burst won where the split sequence lost. Use this when the multi-lane
  hard-refusal #6 protocol (wait-for-quiescence) isn't an option and you must land a
  small, file-scoped commit on a checkout another session is actively touching: the whole
  read-stage-commit-verify is a single `&&`-chained command, and the post-commit
  `git show --stat HEAD` confirms only your files landed (catches a foreign WIP being
  swept in). NOT a substitute for the wait-for-quiescence default — it's the
  "commit-now-anyway" fallback, and you still verify isolation after. Pairs with
  `feedback_cross_session_commit_attribution_collision.md`.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Explore subagents (×3, plan phase) | 25 | 0 | low | parallel ground-truth of db.ts/search.ts/build — fast, accurate, fed the plan |
| Plan-mode + ExitPlanMode | 15 | 0 | none | clean approved plan; the deviations (4-tool scope, §7b defer) were surfaced upfront |
| Temp-DB validation probes (×3) | 30 | 5 | medium | caught nothing broken but PROVED correctness; 5 min wasted on the fts_storm false-positive chase (which became a real fix) |
| `cross-machine sync` Explore subagent | 10 | 8 | medium | mapped topology well BUT cited stale spec numbers (579 rows, 0 EliteDesk vectors) — I had to re-verify live. Net positive but needed correction |
| AskUserQuestion (×5) | 8 | 0 | none | every fork changed the next action; no confirmation-noise |
| Monitor build + base64 ssh fix | — | 15 | high | Windows-OpenSSH quote-strip ate ~15 min across two attempts before base64 |
| read_events telemetry impl | — | 0 | low | counterfactual rank_without computed cleanly from the scores map; 12/12 probes passed first try |
| EliteDesk full uplift | — | 0 | high | baseline-diff caught the version-drift; backup-first kept it safe; ~0 wasted because I checked before copying |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. (Wall-clock approximate — this was an
interactive laptop session, not a Junior task with telemetry; max-log-silence N/A.)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Read-pheromone core + §7b impl | 2 | 2 | ~30 | n/a (interactive) |
| EliteDesk full version uplift | ~8 (whole src tree) | 0 (scp, not commit) | ~12 | n/a |
| read_events telemetry | 2 | 1 | ~25 | n/a |
| Monitor + analyze + tune scripts | 4 | 2 | ~30 | n/a |

No watchdog-envelope concern (interactive, not Junior). The EliteDesk uplift touched
the whole src tree (>8 files) but via scp of a verified-identical source, not authored
edits — low risk despite the file count.

## Decisions to revisit

- **The MCP source is local-only** (`feat/read-pheromone`, no PR) and the EliteDesk
  got code via scp (not a git checkout) — its tree is now ahead of any recorded
  history. If this MCP is ever to have one source of truth across machines, that's a
  future cleanup. Worth a deliberate decision, not drift.
- **Laptop NSSM restart for read_events** is still pending on the user (elevation-
  gated). The 24h handover flags it; if not done, laptop `events_total` stays 0.
- **Relevance-labeling** (`pmd-pheromone-relevance-labeling.md`) is speced decide-
  later; the §6 triggers (observed mis-ranking / gross mis-tuning signal) are the
  re-open condition. Don't build absent a trigger.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 (base64 `ssh-sql` helper): add to `.claude/tools/` + note in a lesson — kills the Windows-OpenSSH quote-strip class (2× this session, cluster in prior memory)
- [ ] Change #2 (spec deploy facts are hypotheses): promote to `.claude/lessons/feedback_spec_deploy_facts_are_hypotheses.md` (cross-harness lesson) — pairs with `feedback_runbook_audit_drift_post_event_check.md`
- [ ] Change #3 (diff baseline before cross-host copy): fold into change #2's lesson or `feedback_sweep_all_repos.md`
- [ ] Change #4 (monitor FP-test discipline): document the `created_at >= today` exclusion as the canonical fts_storm form (already in the monitor header comment)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
