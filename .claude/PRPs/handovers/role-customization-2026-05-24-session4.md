# Handover — Role-customized harness (hook-fix shipped + verified; T4a pending)

**Author:** advisor session, governance-v0 canonical checkout, 2026-05-24 (session 4 — continuation of [session3](role-customization-2026-05-24-session3.md))
**For:** next advisor session resuming role-customization after fresh-session restart
**Goal:** ship the per-role context-management substrate + RLS-driven evolution.

## 1. Resume in three reads

1. [Session 3 handover](role-customization-2026-05-24-session3.md) — what session 3 shipped (T1b NSSM service + HTTP MCP) + the original hook-fix diagnosis (which turned out to be **partly wrong** — see §2.2 below).
2. **This handover** — what session 4 shipped (hook fix end-to-end verified via smoke #449; drain script for cross-machine queue ingest) + the corrected root-cause analysis.
3. PMD row 535 (`role-signal: bm-task utilisation (task 8cc3270f...)`) — the first real Junior-emitted role-signal row in canonical PMD; proves the pipeline works.

If you read only one paragraph: **The role-signal Stop hook is now firing on Junior bm-task workers end-to-end. Smoke task #449 produced sentinel row 534 (deploy-note via HTTP MCP from worker) AND role-signal sibling row 535 (summary via the drain pipeline) in canonical laptop PMD. The hook fires on EliteDesk, parses `transcript_path` to detect the `[role:bm-task]` tag (the original handover's assumption that `CLAUDE_PROMPT` env exists was wrong — it doesn't in `claude -p` mode per official Claude Code hook docs), the CLI predictably fails (no local canonical PMD on EliteDesk), the hook queues to `/srv/brehon-fork/.claude/role-signal-queue.jsonl`, and `scripts/brehon/drain-role-signal-queue.sh` (scp-based, Windows-compatible) pulls + ingests into canonical PMD. T4a `/check-role-health <role>` is now genuinely unblocked — there IS data to consume.**

## 2. What session 4 shipped

### 2.1 Hook fix #1 — drop branch-gate, derive worker worktree (commit `feaa75db9`)

Original session-3 handover §3.1 diagnosis was correct: the `^junior/` branch gate on `git rev-parse --abbrev-ref HEAD` always saw the daemon's main checkout (`governance-v0`), never `junior/*`. Patch: drop branch-gate, derive `WORKER_DIR` from `transcript_path` (walk up to `.git`), read `branch` + `config_version` + `decision-queue.json` from `git -C $WORKER_DIR`. JSONL queue file relocated to `${WORKER_DIR}/.claude/` so it survives in the worker worktree per-task.

This patch was **necessary but not sufficient** — see §2.2.

### 2.2 Hook fix #2 — parse transcript for role tag (commit `729b13312`)

Session-3 handover §3.1 also said "gate on the `CLAUDE_PROMPT` env for `[role:` regex". **That env var does not exist.** Per Claude Code official hook docs (https://code.claude.com/docs/en/hooks.md), hook subprocesses get `CLAUDE_PROJECT_DIR` + `CLAUDE_PLUGIN_ROOT` + plugin paths but NOT prompt content. The `claude -p` invocation is by-design env-minimal.

Confirmed empirically:
- Junior daemon at `/usr/local/bin/junior` invokes: `["-p", prompt, "--output-format", "stream-json", "--verbose", "--dangerously-skip-permissions"]` — no env injection.
- `strings /usr/local/bin/junior | grep CLAUDE_PROMPT` → 0 hits.
- Manual test of fix #1 patch on EliteDesk against a real worker transcript: hook hit `[ -z "$ROLE" ]` early-exit → silent skip, no queue file, no signal.

Smoke task #448 (under fix #1 only) confirmed the gap: sentinel landed (row 533) but NO queue file on EliteDesk and NO role-signal sibling in PMD.

Real fix (#2): parse `transcript_path` JSONL file for the `[role:(planning|impl-task|bm-task|ci-watcher)]` tag. The dispatch prompt lives verbatim in the first `queue-operation:enqueue` line of the per-session transcript at `~/.claude/projects/<proj>/<session-id>.jsonl` — Claude Code's own transcript, NOT the Junior daemon log (the daemon log is a SUPERSET stream-json output; the per-session jsonl is what the Stop hook's `transcript_path` field points at).

Fast-path: keep the (always-empty in practice) `CLAUDE_PROMPT` env check first in case some future invocation does set it. Then transcript scan. Then early-exit if neither yields a role.

Verified on EliteDesk manually before commit: hook fires, detects `role:bm-task`, writes queue file. Then verified end-to-end via smoke task #449 (see §2.4).

### 2.3 Drain script for EliteDesk-to-laptop queue (commit `59d3b3414`)

`scripts/brehon/drain-role-signal-queue.sh` (added in `feaa75db9`, fixed in `59d3b3414`):

- scp pulls `/srv/brehon-fork/.claude/role-signal-queue.jsonl` (daemon main-checkout queue, where bm-task workers land per §2.2) and any `**/role-signal-queue.jsonl` under `/srv/brehon-fork/.junior/worktrees/` (worktree-resident queues for impl-task / planning workers that DO write into their worktree before reap).
- Ingests each line into canonical PMD via the laptop's `write-role-signal.js` CLI.
- Dedup index at `.claude/role-signal-drain/.drained-rows` (key: `ts:task_id`) so re-runs are idempotent.
- Fix in `59d3b3414`: original used rsync (handover §3.3 step 3 wording), but Git-Bash on Windows doesn't ship rsync. Replaced with `scp -q` for the fixed main-checkout path and `ssh + find + scp` fan-out for worktree queues.

Run manually with `bash scripts/brehon/drain-role-signal-queue.sh`. Eventually should be on a cron / weekly-review step. Stub for that: §4.3.

### 2.4 End-to-end verification — smoke task #449

Two rows in canonical PMD after task #449:
- **Row 534** — `deploy-note`, title `role-signal-hook-smoke-2026-05-24 sentinel`, tags `smoke,role-signal-fix,task-id-449`. This is the brief-specified sentinel written by the worker via `mcp__project-memory__memory_write` through HTTP MCP (T1b transport).
- **Row 535** — `summary`, title `role-signal: bm-task utilisation (task 8cc3270f-91e1-4e82-95ea-bd2c03ff838b)`, tags `role-signal,role:bm-task,kind:utilisation,task_id:8cc3270f-91e1-4e82-95ea-bd2c03ff838b,branch:governance-v0,config_version:a6a21c07070ed33c4cb9157e1754e8b32b38ae97`. This is the Stop-hook emission, ingested via the drain pipeline.

Smoke task #448 (under fix #1) produced only row 533 (sentinel-only — proved the gap fix #2 needed to close).

### 2.5 NSSM service `pmd-http-mcp` is rock-solid

No changes this session — `pmd-http-mcp` Windows service kept running across both smoke tasks. Both smoke tasks' sentinel writes (#448 row 533 and #449 row 534) prove HTTP MCP transport from EliteDesk → laptop canonical PMD continues to work as it did at end of session 3.

### 2.6 MCP repo state

Both unpushed commits from sessions 2+3 were pushed to `origin/main` early in session 4:
- `5cc7071 feat(role-signal): add write-role-signal CLI for hook-side PMD writes`
- `19a46c7 feat(nssm): add NSSM service installer for pmd-http-mcp`

EliteDesk's `/home/barrie/MCPs/project-memory-mcp/` is NOT a git checkout (per session-3 inspection — scp'd deployment from March). Session 4 scp'd `dist/scripts/write-role-signal.{js,d.ts}` directly to EliteDesk. The CLI is reachable there but predictably exits non-zero because `PROJECT_MEMORY_DB` resolves to a path that doesn't exist on EliteDesk (the canonical PMD is laptop-only). That's why the queue+drain pipeline is mandatory.

## 3. Architecture clarification (worth carrying forward)

EliteDesk Junior workers CANNOT directly write to canonical PMD. The laptop's PMD is local-SQLite-only; HTTP MCP is for **agent-tool calls** (the worker's `memory_write` MCP invocation works fine via T1b's pmd-http-mcp service on port 11435). The Stop hook is **shell-only** and uses `better-sqlite3` directly (not MCP), so it cannot reach a remote DB.

The two-path architecture:
- **Agent-side writes (Junior worker `memory_write`):** HTTP MCP → pmd-http-mcp service → canonical PMD. Synchronous.
- **Hook-side writes (Stop hook on Junior worker):** queue JSONL → drain script (scp + CLI on laptop) → canonical PMD. Async, requires drain.

Long-term option-c from session-3 handover (modify Junior daemon binary to invoke `claude -p` with prompt-aware env) would let the hook use the CLI directly. But it would require coordinating with Junior daemon owner; the queue+drain pipeline is good-enough for now.

## 4. What's still pending

### 4.1 T4a — `/check-role-health <role>` consumer (HIGH VALUE, NOW UNBLOCKED)

Unchanged scope from session-2 / session-3 handovers, but now there IS data: row 535 is the first real Junior-emitted signal. As bm-task / impl-task / planning workers run, more signals accumulate in PMD via the drain pipeline. Once ≥18 dispatches per role land, T4a's "rules loaded but never read" / "MCPs loaded but never invoked" analyses become meaningful.

Two implementation notes that surfaced this session:

1. **Drain must run before `/check-role-health`** — analyses are only as fresh as the last drain. If the consumer runs against canonical PMD without recent drain, recent Junior signals are invisible. Either (a) `/check-role-health` invokes drain as a pre-step, or (b) drain runs via weekly-review.

2. **`rules_read` and `mcp_tools_invoked` in `content` are always `[]` so far** — because the hook's transcript parsing for those fields (lines 109-130 of `role-signal-utilisation.sh`) only runs when the worker's transcript_path is a file AND `jq` is present. Both held during smoke #449, but the content still came out empty arrays. Likely because the worker's session was so short (28s, single tool call) that `jq -rcs` on the JSONL returned no Read/tool_use events matching `\\.claude/rules/.*\\.md$`. Will need real impl-task or planning workers (longer sessions, many Reads) before these populate.

### 4.2 T4b — weekly-review skill extension

Add a Step that runs `bash scripts/brehon/drain-role-signal-queue.sh` (drains accumulated queue) THEN summarises role-signal rows in canonical PMD by role. Lower priority than T4a but pairs naturally with the weekly cadence.

### 4.3 T5 / T8 — Daemon honours rules.allowlist via `--bare`

Unchanged. Plan once T4a shows stable strip candidates.

### 4.4 Documentation drift (carry-forward from session 3 §4.5)

- **PMD lesson 525** (NSSM 3 traps) needs mirroring to `.claude/lessons/feedback_nssm_powershell_install_traps.md` per `feedback_lesson_mirror_check.md`. Unchanged from session 3.
- **NEW lesson candidate from this session:** `feedback_claude_p_mode_env_minimal.md` — `claude -p` mode does NOT propagate prompt as `CLAUDE_PROMPT` env to hook subprocesses; hooks must read `transcript_path` (a per-session JSONL at `~/.claude/projects/<proj>/<session-id>.jsonl`) to recover prompt content. Failure mode: hooks that try to gate on prompt content via env-var see empty string → exit early → no side effects. Recurrence count: 1 (this session). Watch for second occurrence before promoting.
- **NEW lesson candidate:** `feedback_stop_hook_fires_silent_in_p_mode.md` — Stop hooks DO fire in `claude -p` mode, but hook events are suppressed from the stream-json log unless `--include-hook-events` flag is passed. Side effects (file writes) still happen — absence of `"hook_event":"Stop"` in worker logs is NOT evidence the hook didn't run; check side effects directly. Watch for second occurrence.
- **NEW lesson candidate:** `feedback_drain_script_no_rsync_on_windows.md` — Git-Bash on Windows lacks rsync; cross-machine sync scripts must use scp + ssh-find for fan-out. Folds into `pattern_cross_platform_divergences.md` as another row. Watch for second occurrence OR add the row directly to the pattern.
- **CLAUDE.md operational facts** — could gain a one-line `pmd-http-mcp` NSSM service entry (still unchanged from session 3).

### 4.5 Make drain script a real recurring task

Manual `bash scripts/brehon/drain-role-signal-queue.sh` only happens when the advisor remembers. Options for next session:
- **Option a:** invoke from weekly-review Step (above, T4b).
- **Option b:** invoke from `/check-role-health` as a pre-step (above, T4a note 1).
- **Option c:** standalone scheduled cron / Junior task on a fixed cadence (~15 min).

Recommend (a) + (b) — drains happen at the points data is consumed. (c) is overkill for current Junior dispatch volume.

## 5. Operational facts (additions to session 3 §5)

| Fact | Value |
|---|---|
| Role-signal CLI on EliteDesk | scp'd to `/home/barrie/MCPs/project-memory-mcp/dist/scripts/write-role-signal.{js,d.ts}` (was missing per session 3 diagnosis). Now reachable but expectedly fails — no local canonical PMD on EliteDesk. |
| Where the role tag actually lives | First `queue-operation:enqueue` JSONL line in the worker's Claude Code transcript at `~/.claude/projects/<proj>/<session-id>.jsonl` (the file `transcript_path` stdin field points to). NOT in `CLAUDE_PROMPT` env (doesn't exist). NOT in the Junior daemon log (a different file). |
| Hook side effects in `-p` mode | DO fire but events are suppressed in worker log unless `--include-hook-events` flag is set. Check file writes / DB inserts directly to verify firing. |
| Queue file path on EliteDesk | `/srv/brehon-fork/.claude/role-signal-queue.jsonl` (for bm-task workers, where WORKER_DIR resolution falls back to daemon main checkout). Worktree-resident path under `/srv/brehon-fork/.junior/worktrees/job-N/.claude/role-signal-queue.jsonl` for workers whose transcript_path resolves to a worktree (impl-task / planning, longer-lived). |
| Drain script | `scripts/brehon/drain-role-signal-queue.sh` — scp+ssh+CLI, idempotent via `.claude/role-signal-drain/.drained-rows` dedup index. Run manually for now. |
| First real Junior role-signal row | PMD row 535 (task #449, bm-task). Future T4a analyses key off `tags LIKE 'role-signal,%'`. |

## 6. Files modified this session

**Brehon repo (`barrie-cork/lemmy:governance-v0`, forward-merged to `phase-v1-RT-r3`):**
- `.claude/hooks/role-signal-utilisation.sh` — 2 commits (`feaa75db9` + `729b13312`), final state parses `transcript_path` for `[role:X]` and falls back to JSONL queue on CLI failure.
- `scripts/brehon/drain-role-signal-queue.sh` — new (`feaa75db9`), fixed (`59d3b3414`), scp-based cross-machine drain.
- `.claude/PRPs/briefs/role-signal-hook-smoke-2026-05-24.md` — smoke brief (`71cfc270a`).
- This handover file (next commit).

**MCP repo (`barrie-cork/project-memory-mcp:main`):**
- Commits `5cc7071` + `19a46c7` pushed to origin (no new code this session, just the missing push).

**EliteDesk:**
- `/home/barrie/MCPs/project-memory-mcp/dist/scripts/write-role-signal.{js,d.ts}` — scp'd from laptop early in session 4 (was missing per session-3 §3.2 diagnosis). Now reachable but predictably fails (no local PROJECT_MEMORY_DB).
- `/srv/brehon-fork` daemon main checkout — pulled `729b13312` mid-session, currently on `59d3b3414`.

**PMD rows written this session (canonical laptop DB):**
- 533 — Smoke #448 sentinel (deploy-note, task-id-448)
- 534 — Smoke #449 sentinel (deploy-note, task-id-449)
- 535 — Smoke #449 role-signal sibling (summary, role:bm-task) ← **first real Junior-emitted role-signal**

## 7. Carry-forward open questions

**From session 3 §7:** "Worker's `base_branch` parameter ignored for bm-tasks?" — not investigated this session. Still worth knowing the daemon's main-checkout branch is load-bearing for worker context.

**NEW from session 4:** Are there worker classes whose transcript_path resolves to a per-worktree path (impl-task / planning) and whose queue files therefore DON'T land at the fixed daemon-main-checkout path? The drain script's `ssh + find` fan-out covers this, but it hasn't been exercised yet — only bm-task has been tested. First impl-task or planning Junior task that runs under this commit will reveal whether the worker's transcript path stays under `~/.claude/projects/-srv-brehon-fork/` (one location) or shifts to `~/.claude/projects/-srv-brehon-fork--junior-worktrees-job-N/` (per-task location). The smoke task #449 transcript was under the latter, so for bm-task at least, the WORKER_DIR resolution worked correctly via `transcript_path` → walk-up `.git`.

## 8. Resume from here

```
TaskList                            # see what's pending
Read .claude/PRPs/handovers/role-customization-2026-05-24-session4.md (this file)
Read .claude/PRPs/handovers/role-customization-2026-05-24-session3.md (session 3 — diagnosis + NSSM)
# Recommended next: T4a /check-role-health <role> consumer (now has data to consume — row 535+).
# Author it to invoke `bash scripts/brehon/drain-role-signal-queue.sh` as a pre-step so
# analyses are always against the freshest signals.
# Alternative: mirror PMD lesson 525 (NSSM) to .claude/lessons/feedback_nssm_powershell_install_traps.md.
```
