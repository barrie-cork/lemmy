Load current Brehon state from authoritative sources before driving the next advisor action.

Argument: $ARGUMENTS (optional). Forms:
- empty → general resume (full 9-probe spec below)
- `v1-JM-d` → focus on a named sub-phase (full spec, but heuristics + git probes prioritise that branch)
- `--fast <task-id>` → fast-resume mode for a single Junior task (4 probes, ~1 KB output, 10-line synthesis). Use when re-grounding on a single in-flight task — see "Fast-resume mode" section below.
- `--delegate` → run the full 9-probe spec inside a `general-purpose` subagent and return only the synthesis. Saves ~11 KB of probe output from the parent context. Use when polling repeatedly during a long advisor session — see "Delegated mode" section below.

This command is **read-only**. It runs no cargo, opens no PRs, sends no messages, edits no files. It synthesizes live state into a one-screen status report so the advisor session can decide the next action without injecting stored snapshots that could drift.

## Why this shape

Per the user constraint 2026-04-26: avoid pre-injecting stored Brehon context every session (bloat, drift). Instead, command-trigger live reads from the single sources of truth. Every probe below has one canonical home; this command only synthesizes, never copies.

## Context cost rough estimate

Per `feedback_read_only_commands_still_cost_context.md`: read-only ≠ free. Probe outputs land in the parent's working context window even though nothing is written to disk. Plan accordingly.

| Mode | Probes | Probe output | Synthesis | Use when |
|------|-------:|-------------:|----------:|----------|
| Full (default) | 9 | ~12 KB | ~25 lines | First call of a session, or unsure of overall phase state |
| `--delegate` | 9 (in subagent) | ~1 KB returned | ~25 lines | Re-running during a polling loop; protects parent context |
| `--fast <task-id>` | 5 | ~3 KB | ~12 lines | Resuming on one specific in-flight task, e.g. after a break |

## Procedure — run probes in parallel where possible, then synthesize

### Phase 1 — parallel probes

Issue these in a single message with multiple Bash tool calls:

#### A. Brehon-fork primary worktree state

```
git -C C:/Users/barri/Developer/brehon-fork rev-parse --abbrev-ref HEAD
git -C C:/Users/barri/Developer/brehon-fork rev-parse --short HEAD
git -C C:/Users/barri/Developer/brehon-fork log --oneline -10
git -C C:/Users/barri/Developer/brehon-fork status --short
```

#### B. All worktrees + their HEADs

```
git -C C:/Users/barri/Developer/brehon-fork worktree list
```

For each non-primary worktree (any path containing `phase-v` or `plan-`), capture the branch name. If $ARGUMENTS names a phase that matches a worktree, prioritize it for §C.

#### C. Phase-branch state (skip if no $ARGUMENTS and no worktrees ahead of trunk)

If $ARGUMENTS is set OR a worktree branch is ahead of `governance-v0`, run for the relevant branch `<B>`:

```
git -C C:/Users/barri/Developer/brehon-fork log governance-v0..<B> --oneline
git -C C:/Users/barri/Developer/brehon-fork rev-parse --short <B>
```

#### D. Open PRs on the fork

```
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,baseRefName,reviewDecision,mergeStateStatus
```

Filter out Dependabot PRs (those starting with `dependabot/` headRefName) for the synthesis — list them only as a count.

#### E. Decision queue state

```
cat C:/Users/barri/Developer/brehon-fork/.claude/decision-queue.json
```

Use `python3 -c` (not jq — `feedback_jq_not_on_windows_path`) to extract pending entries:
```
python3 -c "import json; d=json.load(open(r'C:/Users/barri/Developer/brehon-fork/.claude/decision-queue.json',encoding='utf-8')); print(f'pending={len(d[\"pending\"])} resolved={len(d[\"resolved\"])}'); [print(f'  #{e[\"id\"]} from={e[\"from\"]} q={e[\"question\"][:80]}') for e in d['pending']]"
```

#### F. BM runlog tail

```
tail -50 C:/Users/barri/Developer/brehon-fork/.claude/runlog/bm-runlog.md
```

Surface only the last 1-2 entries (look for `## ` headers) — they tell you the last BM/advisor action with timestamp.

#### G. Most recent handovers (advisor + impl)

```
ls -t C:/Users/barri/Developer/brehon-fork/.claude/PRPs/handovers/*.md 2>/dev/null | head -4
```

For each filename, parse the role prefix (`advisor-` / `impl-` / `pr*` / etc.) and date. Surface the newest advisor and newest impl by name + date — do NOT read the bodies (those are 200-line briefs; the existence + name is the signal).

#### H. Most recent advisor brief

```
ls -t C:/Users/barri/Developer/brehon-fork/.claude/PRPs/briefs/*.md 2>/dev/null | head -3
```

For the newest, also run:
```
git -C C:/Users/barri/Developer/brehon-fork log -1 --pretty=format:'%h %cr %s' -- .claude/PRPs/briefs/<filename>
```

That tells you if the brief has been queued to Junior yet (recent commit + matching `chore(advisor)`) or is fresh-uncommitted.

#### I. Most recent plan + retro

```
ls -t C:/Users/barri/Developer/brehon-fork/.claude/PRPs/plans/*.plan.md 2>/dev/null | head -3
ls -t C:/Users/barri/Developer/brehon-fork/.claude/PRPs/retros/*.md 2>/dev/null | head -3
ls -t C:/Users/barri/Developer/brehon-fork/.claude/PRPs/reports/*-retro.md 2>/dev/null | head -3
```

The newest plan tells you the active sub-phase. The newest retro tells you the last completed sub-phase. (Some retros land under `reports/`, some under `retros/` — check both.)

#### J. Junior daemon health + recent tasks

```
ssh -o ConnectTimeout=5 homeserver 'systemctl is-active junior@brehon-fork && systemctl show -p ActiveEnterTimestamp,MainPID,MemoryCurrent junior@brehon-fork --no-pager'
```

```
ssh -o ConnectTimeout=5 homeserver 'sqlite3 /srv/brehon-fork/.junior/junior.db "SELECT id, status, datetime(updated_at, \"unixepoch\") as upd, substr(title,1,80) as title FROM jobs ORDER BY id DESC LIMIT 5;" 2>/dev/null'
```

If SSH times out or sqlite returns empty, surface that — daemon-down or empty-queue is itself a signal.

#### K. EliteDesk worktree git tip (probes if Junior pushed since last fetch)

```
ssh -o ConnectTimeout=5 homeserver 'cd /srv/brehon-fork && git log --oneline -3 governance-v0 && echo --- && git for-each-ref --format="%(refname:short) %(objectname:short)" refs/heads/junior/ 2>/dev/null | head -5'
```

If the EliteDesk's `governance-v0` is ahead of the laptop's, Junior has pushed; the laptop session needs `git fetch origin` before its next read.

#### L. MEMORY.md urgency marker cross-check

```
grep -n '🔴' C:/Users/barri/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/MEMORY.md
```

If any `🔴 NEXT SESSION` (or similar urgency marker) lines exist, cross-check each against live state before surfacing. For each hit: extract the named commit SHA, DQ id, or PR number, then verify it's still open/unresolved via `git log --oneline` or the DQ pending list (probe E). If the referenced work is already done (commit present on HEAD, DQ resolved, PR merged), the entry is stale — note it as "🔴 entry stale — already resolved" in the synthesis and suggest removal. This 30-second check converts a potential 5–10 min false-start investigation into an instant "already done" notice.

### Phase 2 — synthesize one-screen status

Format the synthesis in this exact shape (one screen, ~25 lines):

```
# Brehon status — <YYYY-MM-DD HH:MM UTC>

**Active sub-phase:** <inferred from latest plan + open PR + worktree state, or "between sub-phases" if last retro is newer than newest plan>
**Argument focus:** <$ARGUMENTS or "(general resume)">

## Git topology
- governance-v0 (primary): <short-sha> — last commit: <subject>
- Open phase branches (worktrees):
  - phase-v1-JM-X (<short-sha>, <N> commits ahead of trunk) → PR #<N> if open
  - ...
- Open PRs on fork: <count non-dependabot> + <count dependabot>
  - #<N> <title> [<reviewDecision> / <mergeStateStatus>]

## Decision queue
- Pending: <count>
  - #<id> from=<role> — <question first 80 chars>
- Resolved this week: <count>

## Most recent activity (newest first)
- <timestamp> — <role>: <action> (from bm-runlog tail OR from latest handover/brief/plan filename)

## MEMORY.md urgency markers
<if no 🔴 entries: "(none)">
<if stale 🔴 entries found: "⚠️ Stale: <entry summary> — already resolved (<evidence>). Remove from MEMORY.md.">
<if live 🔴 entries found: "🔴 <entry summary> — still open. Address first.">

## Junior daemon (EliteDesk)
- Status: <active/inactive> since <timestamp> · PID <N> · <MB> RAM
- Last 5 tasks: <id> <status> <date> <title-truncated>

## Suggested next action
<inference from above — see "Synthesis heuristics" below>

## Sources read
<list every probe that ran, so user can verify nothing was fabricated>
```

### Phase 3 — synthesis heuristics for "Suggested next action"

These are inferences, not commitments — surface them with "suggest" phrasing so the user can override.

| Observed state | Suggested next |
|---|---|
| Stale 🔴 MEMORY.md entry found (probe L cross-check shows work already done) | Remove stale entry from MEMORY.md before proceeding — prevents false-start overhead for this and future sessions |
| Pending DQ exists with `from: impl` or `from: bm` | Triage that DQ first (advisor-answer / catch-fire / user-relay per `.claude/rules/advisor-orchestrator.md` "DQ triage decision tree") |
| Newest brief is committed but no matching Junior task in `jobs` table | Queue the task: `mcp__junior-brehon__create_task(description: "<brief's dispatch line>")` |
| Junior task `status: running` exists | Poll loop — re-run `/start-brehon` in ~10 min, or just `mcp__junior-brehon__list_tasks` if MCP loaded |
| Junior task `status: complete` since last advisor commit | Read `mcp__junior-brehon__show_task <id>` + the artifact it produced (plan file, etc); run DoD smoke test if it's a plan |
| Open PR with CR findings + no recent BM-runlog activity | Run `/bm-poll-cr <PR#>` to ingest findings |
| Newest retro is older than newest plan AND PR for that plan is merged | Author retro per `feedback_retro_not_report` |
| EliteDesk's governance-v0 is ahead of laptop's | `git fetch origin` first, then re-run synthesis |
| EliteDesk's governance-v0 is BEHIND laptop's | Harmless — Junior fetches when next task created. Note in synthesis but don't push to user. |
| Stale leftover worktrees for already-merged phases (e.g. `phase-v1-JM-a` after JM-a merged) | Surface as housekeeping note, not as suggested-next. Removing is `git worktree remove <path>` — leave to user. |
| Newest "retro" file lives under `reports/*-retro.md` instead of `retros/` | Both dirs are valid (per-phase historical inconsistency); `/start-brehon` scans both. Not an issue. |
| All quiet (no pending DQ, no running Junior task, no open phase PR, newest retro is newer than newest plan) | Likely between sub-phases — check PRD §17 for next sub-phase, then plan the next planning brief |

### Phase 4 — closing line

End with: "Brehon state loaded. Ready to advise. Re-run `/start-brehon` after any state-changing action to re-sync."

Do NOT append the synthesis to any file. Do NOT update any memory. The whole point of this command is that it leaves no artifact — every invocation is a fresh, current read.

## Out-of-scope refusals

This command does NOT:
- Run cargo / write code / commit anything
- Open PRs / merge PRs / send Telegram pings
- Author plans / briefs / retros / handovers
- Edit `.claude/decision-queue.json` (that's the orchestrator's job per attribution rules)
- Read full bodies of plans, retros, briefs (filenames + dates only — body reads are on-demand based on the suggested next action)
- Cache or snapshot anything to disk

If $ARGUMENTS asks for any of the above ("`/start-brehon plan v1-JM-d`"), refuse and remind the user that `/start-brehon` is read-only — point them at the right command (`/prp-core:prp-plan`, `/bm-cut`, etc).

## Fast-resume mode (`--fast <task-id>`)

When `$ARGUMENTS` parses as `--fast <N>`, run the abbreviated procedure below. Skips the world-state probes; anchors on one Junior task. Use after a break, after a `/loop` poll wakeup, or before answering a single-task question.

### Probes (5, parallel)

1. **Task DB row** — `ssh homeserver 'sqlite3 /srv/brehon-fork/.junior/junior.db "SELECT id, status, datetime(updated_at, \"unixepoch\"), title FROM jobs WHERE id=<N>;"'`
2. **Log mtime + raw tail** — `ssh homeserver 'LOG=$(ls -t /srv/brehon-fork/.junior/logs/job-<N>-run-*.log | head -1); stat -c "size=%s mtime=%y" "$LOG"; wc -l "$LOG"; tail -3 "$LOG" | cut -c1-500'` — primary signal is `stat` mtime (cheap liveness proxy) + raw tail of the last 3 events. Use the raw tail to read the latest tool_use / text / tool_result. Do NOT lead with a python heredoc filter — last 10 lines often contain only `system.task_updated` events with no assistant/user content for the filter to surface (returned empty in 3 of 4 polls during session 2026-04-27, see `feedback_stream_json_digest_filter_fragile.md`). The digest filter can run as a best-effort augmentation over `tail -40` if the raw tail is uninformative.
3. **Worktree git tip + working-tree status** — `ssh homeserver 'WT=/srv/brehon-fork/.junior/worktrees/job-<N>; [ -d "$WT" ] && git -C "$WT" log --oneline -3 governance-v0..HEAD; git -C "$WT" rev-parse --abbrev-ref HEAD; git -C "$WT" status --short | head -10'` — if the worktree is gone (cleaned up after cancel/complete), report that and skip §3 in synthesis. Status delta vs prior poll tells you whether the task is editing files or just thinking.
4. **Worker liveness + artifacts** — `ssh homeserver 'WT=/srv/brehon-fork/.junior/worktrees/job-<N>; ls -la "$WT/.claude/PRPs/debug/" 2>/dev/null | tail -10; ls -la "$WT/.claude/PRPs/plans/" 2>/dev/null | tail -5; ls -la "$WT/.claude/PRPs/reports/" 2>/dev/null | tail -5; echo ---worker---; pgrep -P $(systemctl show -p MainPID --value junior@brehon-fork) -f "claude -p" | xargs -r ps -o pid,etime,pcpu,rss,cmd 2>/dev/null | head -2'` — combines artifact listing with a worker-process check. The `claude -p` child of the daemon is the actual model run. **Critical when log mtime is stale on a `running` task**: if `%CPU > 5%`, model is thinking (wait); if `%CPU == 0%` and process exists, genuinely hung; if process is gone, daemon already cleaned up. See `feedback_junior_task_liveness_check.md` — log mtime alone is misleading because Opus reasoning between tool calls can run multi-minute silent.
5. **DQ pending check** — read the laptop-side `decision-queue.json` for any pending entries (running impl-tasks push blockers here mid-flight; full-mode probe E checks this, but fast-mode skipped it pre-2026-04-27). Surface only counts + IDs from the current task's role:

   ```
   python3 -c "import json; d=json.load(open(r'C:/Users/barri/Developer/brehon-fork/.claude/decision-queue.json',encoding='utf-8')); p=d['pending']; print(f'pending={len(p)}'); [print(f'  #{e[\"id\"]} from={e[\"from\"]} kind={e.get(\"kind\",\"?\")} q={e[\"question\"][:80]}') for e in p]"
   ```

   Note: this reads laptop-side state, not the EliteDesk's. If the running task pushed a DQ to the worktree but hasn't merged to `governance-v0` yet, this probe won't see it. For the strictest read, prefix with `git -C C:/Users/barri/Developer/brehon-fork fetch origin governance-v0` — but that adds a network round-trip; only do it on the first poll of a session, not on every fast-resume.

### Synthesis shape (12 lines)

```
# Fast resume — task #<N> — <YYYY-MM-DD HH:MM UTC>

Task: <title>
Status: <s> for <Nm> (since <updated_at>)  ·  Run: #<r>  ·  Log: <L> lines · last write <mtime> (<Nm> ago)
Worker: PID <pid>, <etime> elapsed, <cpu>% CPU, <rss> RSS  <or "process gone">
Branch: <branch> @ <sha> (<X> commits ahead of governance-v0)
Latest activity: <one-line digest of newest tool_use or text from raw tail>
Artifacts: <plan file path | "none yet" | "complete: <list>">
DQ: pending=<count>  <if >0: list IDs + from-roles, one per line>
Roles engaged so far: Advisor → <Planning|Impl|BM> <progress-bar>
Next user gate: <plan approval | DQ triage | CR triage | merge confirm | retro sign-off | "none">

Suggest: <single concrete action — see fast-mode heuristic table>

Sources: 5 probes (--fast mode)
```

### Fast-mode heuristic table

DQ check first: if `pending > 0` in probe 5, the suggestion is **"run `/check-dq` for full DQ analysis before acting on the task status"** — DQ blockers from a running impl-task pre-empt all task-status heuristics below. Fast-mode's probe 5 only surfaces the count + IDs; the full triage (advisor-answer / catch-fire / user-relay) lives in `/check-dq` and `.claude/rules/advisor-orchestrator.md`.

If `pending = 0`, dispatch on task status — but **check worker CPU first** when log mtime is stale on a `running` task (per `feedback_junior_task_liveness_check.md`):

| Task status + signals | Suggested next |
|---|---|
| `running` + log mtime fresh (< 5min ago) | Healthy progress — read raw tail to see latest tool_use, re-poll in 10-15 min. |
| `running` + log mtime stale (> 5min) + worker CPU > 5% | **Model is thinking, not stalled.** Opus reasoning over large worktree context can run 5-20+ min silent between tool calls. Wait. Watchdog is `ACTIVITY_TIMEOUT_MS = 60min` (per `project_junior_watchdog_60min`) — slack remains. |
| `running` + log mtime stale + worker CPU == 0% + process exists | Genuinely hung. `mcp__junior-brehon__task_logs <N>` for full tail; consider cancel + requeue if no recovery in next poll. |
| `running` + log mtime stale + worker process gone | Daemon already cleaned up; DB will catch up. Wait one poll cycle then re-check status (likely transitions to `failed`/`cancelled`/`complete`). |
| `running` + no plan/PR file (general) | Wait — task is still working. Re-run `/start-brehon --fast <N>` later. |
| `running` + plan file present | Plan is being finalised (DoD checks); wait for status transition. |
| `done`/`complete` + plan file | Read the plan, run DoD smoke test, then surface to user for plan approval. |
| `done`/`complete` + no plan file | Inspect the worktree — task may have produced a different artifact. Run full `/start-brehon` for orchestration context. |
| `failed` | `mcp__junior-brehon__task_logs <N>` for error tail; check auth, settings drift, MCP shim quoting bug per `feedback_windows_mcp`. |
| `cancelled` | Confirm cancel was intentional. If retry needed, fix root cause first then `junior task add` (not retry — fresh worktree). |
| `review` | Switch to full `/start-brehon` — review-mode tasks need orchestration context, not single-task focus. |

### Refusals in --fast mode

If `--fast` is given without a task-id, or with a non-numeric task-id, refuse and surface `/start-brehon --fast <N>` with examples of recent task IDs (cheap one-line probe: `ssh homeserver 'sqlite3 ... ORDER BY id DESC LIMIT 3'`). Do not fall back to full mode silently.

## Delegated mode (`--delegate`)

When `$ARGUMENTS` is `--delegate` (or starts with `--delegate ` for a focused sub-phase), run the **full 9-probe spec inside a `general-purpose` Agent subagent** instead of the parent context. The subagent issues all probes, performs the synthesis, and returns only the one-screen report (~1 KB) instead of the raw probe outputs (~12 KB).

### Why

Per `feedback_subagent_delegation_for_multi_probe_commands.md` and `feedback_read_only_commands_still_cost_context.md`: read-only ≠ free. The parent advisor's context window is the scarcest resource. During polling loops or extended advisor sessions, repeated full `/start-brehon` calls invalidate cache and accumulate probe noise. Delegating preserves parent context for actual reasoning + orchestration.

### Procedure

Issue exactly one tool call:

```
Agent({
  subagent_type: "general-purpose",
  description: "Brehon live-state probe + synthesis",
  prompt: "<full spec from §Phase 1 + §Phase 2 of /start-brehon, with $ARGUMENTS focus passed through>"
})
```

The subagent's prompt must be **self-contained** — it doesn't have parent conversation context. Include the absolute paths (`C:/Users/barri/Developer/brehon-fork`, `/srv/brehon-fork/.junior/junior.db`, etc), the SSH alias (`homeserver`), and the synthesis shape verbatim.

### When to use

- Inside a `/loop` polling cadence — every poll otherwise re-injects the probe noise.
- After a state-changing action where you want fresh state but don't want to displace the cache from your main reasoning.
- When the synthesis itself is what you need; not when you're about to debug something the synthesis collapsed (in that case, run inline).

### Trade-offs

- **Cost:** subagent token use is non-zero (~70k tokens for a typical run, see prior /reflect). Inline mode costs the parent ~12 KB of probe output. Pick based on which budget is scarcer right now.
- **Latency:** delegated mode adds ~30-60s vs inline parallel probes (~5s). Don't use during time-pressed actions.
- **Errors:** if a probe fails inside the subagent, the surface error is "Agent returned no synthesis" — re-run inline (no `--delegate`) to see the raw failure.

## See also

- `.claude/hooks/session-banner.sh` — passive layer; shows brehon-fork Junior tasks at session start (no cost)
- `homeserver` PMD entry `project_brehon_advisor_takes_over_plan.md` — the durable bookmark for "what is the current Brehon initiative" (auto-loads at session start via MEMORY.md index)
- `brehon-fork/.claude/rules/advisor-orchestrator.md` — the orchestrator-rule the advisor reads at session start (currently mirrored at `homeserver/.claude/rules/advisor-orchestrator.md` — keep in sync; surface diff if they drift)
- `brehon-fork/.claude/commands/repo-context.md` (if it ever exists) — `/repo-context $ARGUMENTS` is the generalized precedent for this pattern
