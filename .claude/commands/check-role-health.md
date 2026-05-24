---
description: Generate a per-role health report joining .claude/roles/<role>/ manifest data ("what's loaded") with PMD role-signal rows ("what's used") and Task-retro rows ("outcome"). Names strip candidates for rules.allowlist + mcp.json when sample size is sufficient. Read-only, advisory only — never edits .claude/roles/<role>/.
argument-hint: <role: planning|impl-task|bm-task|ci-watcher> [--no-drain] [--strip-candidates-only]
---

<objective>
The `/check-role-health` command is the **consumer** of the role-signal substrate shipped in T1b + T2 + T3 (per [role-customization session 2-4 handovers](../PRPs/handovers/)). It reads accumulated `kind:utilisation` signal rows from canonical PMD and joins them against:

- **What's loaded** — `.claude/roles/<role>/rules.allowlist` (the list of `.claude/rules/*.md` files the role is dispatched with) + `mcp.json` (MCP servers).
- **What's used** — the `rules_read` array in each signal row's content JSON (which rule files the worker actually `Read` during the task) + `mcp_tools_invoked` (which `mcp__*` tools the worker actually invoked).
- **Outcome** — `Task retro:%` rows joined by `task_id` for success/partial/failure rate per `config_version`.
- **DQ-blocker rate** — sum of `dq_blockers_added` per signal row, grouped by `config_version`.

The report names specific files / MCP servers as **strip candidates** when the rule or MCP was loaded but never read across ≥18 dispatches for the role at a given `config_version`. **Advisory only — does NOT edit `.claude/roles/<role>/`**. The strip decision is the advisor's, informed by this report.

Per session-4 handover §4.1: T4a is the consumer the whole substrate exists for. Without this command, signal rows accumulate in PMD but no one reads them.
</objective>

<usage>
**`$ARGUMENTS`** = `<role>` (required, one of: `planning`, `impl-task`, `bm-task`, `ci-watcher`) followed by optional flags.

Examples:

- `/check-role-health bm-task` — full report for bm-task role (drains queue first, then reports).
- `/check-role-health impl-task --no-drain` — skip the drain pre-step (use when freshness doesn't matter or to avoid scp).
- `/check-role-health planning --strip-candidates-only` — terse output, only the strip-candidate section.

If `$ARGUMENTS` is empty or the role isn't one of the four canonical roles, refuse: "specify a role: planning, impl-task, bm-task, or ci-watcher".
</usage>

<precondition>
This command runs from brehon-fork (canonical checkout `brehon-fork/` OR any lane worktree `brehon-fork-*/`). All four dependencies it needs are tracked in git and propagate via `git worktree add`:

- `.claude/roles/<role>/` — role substrate (tracked).
- `.claude/commands/check-role-health.md` — this file (tracked).
- `scripts/brehon/drain-role-signal-queue.sh` — drain script (tracked).

The fourth dependency, **canonical PMD**, is NOT tracked and is NOT per-worktree. Per `.claude/rules/pmd-invariants.md` invariant #1, the canonical PMD is **always** at the absolute path:

```
C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db
```

This path is hardcoded in this command's SQL queries — never relative. From any lane worktree (e.g. `brehon-fork-rt-r3/`), the query reads the canonical DB on the canonical checkout. From the canonical checkout itself, the path resolves to the same file. Lane drift impossible.

`pwd` must resolve under `C:/Users/barri/Developer/brehon-fork` (canonical or `-<lane>` sibling). If not, refuse with "run from brehon-fork CWD".

`sqlite3` must be on PATH. If absent: refuse with "install sqlite3 CLI first".

If the canonical PMD file is missing: refuse with "canonical PMD not found at C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db — has the system been initialized?".
</precondition>

<workflow>

### Step 1: Validate inputs

- Parse `$ARGUMENTS` for role + optional flags.
- Confirm `role` ∈ `{planning, impl-task, bm-task, ci-watcher}`.
- Confirm CWD is brehon-fork (per <precondition>).
- Confirm `.claude/roles/<role>/manifest.yaml` exists; else refuse with "no role substrate for <role> — has T2 shipped?".

### Step 2: Drain pre-step (unless `--no-drain`)

Run `bash scripts/brehon/drain-role-signal-queue.sh` to pull EliteDesk's accumulated role-signal JSONL queues into canonical PMD. Per session-4 handover §4.1 note 1: analyses are only as fresh as the last drain.

If the script exits non-zero, surface the error and continue (don't block the report — stale data is better than no report). Note in the report's metadata section that drain failed.

If `--no-drain` is set, skip this step and note "drained: skipped" in the report metadata.

### Step 3: Read what's loaded

For the role:

```
ROLE=<role>
ALLOWLIST=$(cat .claude/roles/$ROLE/rules.allowlist)        # one rule filename per line
MANIFEST=$(cat .claude/roles/$ROLE/manifest.yaml)            # YAML manifest
MCP_JSON=$(cat .claude/roles/$ROLE/mcp.json)                 # MCP servers JSON
SYSTEM_PROMPT_BYTES=$(wc -c < .claude/roles/$ROLE/system-prompt.md | tr -d ' ')
```

Compute:
- `RULES_LOADED` = lines in `rules.allowlist` (count of `.md` files).
- `RULES_BYTES_TOTAL` = sum of `wc -c .claude/rules/<file>.md` for each file in the allowlist (warn on missing files — these are stale allowlist entries).
- `MCP_LOADED` = JSON keys under `.mcpServers` from `mcp.json`.
- `CONFIG_VERSION_CURRENT` = `git rev-parse "HEAD:.claude/roles/$ROLE"` (subtree SHA of the role dir at current HEAD). This is the `config_version` future signals will carry.

### Step 4: Query PMD for utilisation signals (last 30d)

**Use the absolute canonical PMD path** per pmd-invariants invariant #1:

```bash
PMD="C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db"
sqlite3 "$PMD" "<query>"
```

```sql
-- Exclude sentinel test rows: they were authored manually by hook smoke tests
-- (T3a/T3b/T3c/T1b/T2 verification phases) with `tags LIKE '%,sentinel'` and
-- carry empty rules_read/mcp_tools_invoked arrays. Including them pollutes
-- utilisation frequencies + falsely flags every allowlisted rule as "never
-- read". Filter by tag suffix `,sentinel` (per the convention established by
-- write-role-signal CLI's --sentinel mode used during T1-T3 verification).
SELECT id, source_ref, branch, content, tags, created_at
FROM memories
WHERE repo_name = 'brehon-fork'
  AND memory_type = 'summary'
  AND tags LIKE '%role-signal%'
  AND tags LIKE '%role:' || ? || '%'
  AND tags LIKE '%kind:utilisation%'
  AND tags NOT LIKE '%,sentinel'
  AND tags NOT LIKE '%,sentinel,%'
  AND created_at >= datetime('now', '-30 days')
ORDER BY created_at DESC;
```

Bind `?` to the role. For each row:

- Parse `content` as JSON → `{rules_read: [...], mcp_tools_invoked: [...], dq_blockers_added: <int>}`.
- Extract `task_id` from tags (`task_id:<value>`).
- Extract `config_version` from tags (`config_version:<value>`).
- Group rows by `config_version`.

**Filter out historical config_versions that cannot recur:**

- `config_version: pre-t2` — pre-T2 baseline (T2 substrate didn't exist yet); will never recur once any T2-or-later signal exists.
- `config_version` matching the literal string `HEAD:.claude/roles/<role>` — hook-bug artefact from when `git rev-parse HEAD:<ref>` stdout leaked through on missing-ref before commit `f6088a83d`. Fixed; will not recur.
- Any `config_version` matching neither a 40-char hex SHA nor the literal `pre-t2` — log as `unknown`, skip aggregation, surface in Notes.

Show filtered-out groups in the report's Notes section with count + reason so the operator can see they were skipped, not lost.

`DISPATCHES_TOTAL` = total row count of **non-filtered** rows.
`DISPATCHES_PER_VERSION` = count per `config_version` (non-filtered only).

### Step 5: Compute utilisation frequencies

For each rule file in `ALLOWLIST` (across all `config_version` groups):

- `READ_COUNT[file]` = number of dispatches whose `rules_read` array contained `file`.
- `READ_FREQ[file]` = `READ_COUNT[file] / DISPATCHES_TOTAL` (rounded to 2 decimal places).

For each MCP server in `MCP_LOADED`:

- `INVOKED_COUNT[server]` = number of dispatches whose `mcp_tools_invoked` contained at least one `mcp__<server>__*` tool (substring match on the server name).
- `INVOKED_FREQ[server]` = `INVOKED_COUNT[server] / DISPATCHES_TOTAL`.

Also compute:
- `DQ_BLOCKER_TOTAL` = `sum(dq_blockers_added)` across all rows.
- `DQ_BLOCKER_AVG` = `DQ_BLOCKER_TOTAL / DISPATCHES_TOTAL` (rounded to 2 decimal).

### Step 6: Join Task-retro rows for outcome (DEFERRED — schema gap)

**Status: DEFERRED.** The retro-join is structurally impossible against the current PMD schema and the post-task-retro skill convention.

**Schema gap:** signal rows store `source_ref = <task_id>` (a Junior task UUID, e.g. `8cc3270f-91e1-4e82-95ea-bd2c03ff838b`). Task-retro rows store `source_ref = <branch>` (e.g. `governance-v0`) or `<commits>` (comma-separated short SHAs). A direct `source_ref` join cannot resolve task identity. Retros also carry no `task_id:<value>` tag — only branch + role tags.

**Two paths to fix (both out of scope for T4a v1):**

1. **Update post-task-retro skill** (cleanest) — make Junior worker retros emit an extra tag `task_id:<id>` when running under Junior (detectable via env or branch shape `junior/*`). One-line change in `.claude/rules/post-task-retro.md` + `.claude/skills/post-task-retro/SKILL.md`. Once shipped, future retros become joinable by task_id; this command's Step 6 enables.
2. **Best-effort branch-join** (lossy) — match retro `source_ref` ↔ signal `branch` tag. Works for single-task branches (e.g. `junior/<slug>-<id>` worker branches) but collapses when a branch hosts multiple tasks (always true for `governance-v0` and any long-lived phase branch). Reports "retro count exceeds dispatch count" when N retros land on one branch.

**For this command v1: skip the Outcome section.** The report prints a single line under `## Outcome`:

> **DEFERRED — retro-to-signal join requires post-task-retro skill change** (add `task_id:<id>` tag). See [session-4 handover §4.1](../PRPs/handovers/role-customization-2026-05-24-session4.md) note 1 for the architectural decision.

Future revision (v2 of this command) re-enables Step 6 once the retro skill ships the tag. Do not silently best-effort-join in v1 — it produces wrong-but-plausible numbers that look authoritative.

### Step 7: Identify strip candidates

Per session-2 §3.2 + user direction (≥18 threshold):

For each `config_version` group with `DISPATCHES_TOTAL[version] >= 18`:

- **Rule strip candidate**: any file in `ALLOWLIST` with `READ_COUNT[file] == 0` across that config_version's dispatches.
- **MCP strip candidate**: any server in `MCP_LOADED` with `INVOKED_COUNT[server] == 0` across that config_version's dispatches.

For groups with `DISPATCHES_TOTAL[version] < 18`: report "insufficient signals (n=<count>/18)" in place of strip suggestions.

If `--strip-candidates-only` flag is set: skip Step 5+6 aggregations from the output and report only this section.

### Step 8: Format report

Print one-screen markdown to stdout (advisor reads it inline). Shape:

```markdown
# /check-role-health <role> — report

**Generated:** <ISO 8601 UTC>
**Drained:** <yes | skipped | failed-but-continued>
**Sample window:** last 30 days
**Current config_version:** <subtree SHA>

## Loaded substrate

- **Rules:** <RULES_LOADED> files, <RULES_BYTES_TOTAL> bytes total
- **MCP servers:** <count> (<comma-separated server names>)
- **System-prompt:** <SYSTEM_PROMPT_BYTES> bytes
- **Stale allowlist entries:** <list of files in allowlist but absent on disk, or "none">

## Utilisation (last 30d)

- **Dispatches:** <DISPATCHES_TOTAL> (<DISPATCHES_PER_VERSION per config_version: "<sha-12>: <count>" per line>)
- **DQ-blocker rate:** <DQ_BLOCKER_AVG>/dispatch (<DQ_BLOCKER_TOTAL> total)

### Rules read (top 5 most-read + bottom 5 least-read)

| Rule | Read count | Frequency |
|------|-----------:|----------:|
| <file> | <count> | <freq> |
...

### MCP tools invoked (top 5)

| Server | Invocations | Frequency |
|--------|-----------:|----------:|
| <server> | <count> | <freq> |
...

## Outcome

DEFERRED — retro-to-signal join requires post-task-retro skill change (add `task_id:<id>` tag).
See [session-4 handover §4.1](../PRPs/handovers/role-customization-2026-05-24-session4.md) note 1.

## Strip candidates

For each **current** config_version with ≥18 dispatches:

### config_version <sha-12> (n=<DISPATCHES_TOTAL[version]>)

**Rules never read:**
- <file> — loaded but 0 reads across <n> dispatches
- ...

**MCP servers never invoked:**
- <server> — loaded but 0 invocations across <n> dispatches
- ...

For each current config_version with <18 dispatches:

### config_version <sha-12> (n=<count>/18)

Insufficient signals — accumulate <18-count> more dispatches at this config_version before reporting strip candidates.

## Notes

- **Sentinel rows excluded** from utilisation aggregation: rows with `tags LIKE '%sentinel%'` were authored manually during T1-T3 hook verification and carry empty rules_read/mcp_tools_invoked arrays. Including them would falsely flag every allowlisted rule as "never read".
- **Historical config_versions skipped:** <list rows like "`pre-t2` (n=N, T2 baseline; will not recur)" + "`HEAD:.claude/roles/<role>` (n=M, hook-bug artefact pre-`f6088a83d`; will not recur)" + "`unknown` (n=K, surface to user — possible hook breakage)">.
- The hook captures `rules_read` only when the worker `Read` the file via Claude Code's `Read` tool (path matches `\.claude/rules/.*\.md$`). Reads via `Bash cat`, `Grep`, etc. are NOT counted — by design (a Read is a deliberate context-load signal).
- `mcp_tools_invoked` covers every `mcp__*` tool call; matched to MCP server names by `mcp__<server>__<tool>` prefix.
- This report is advisory only. To strip a rule from `<role>` allowlist, edit `.claude/roles/<role>/rules.allowlist` + commit; the next dispatch will record signals at a new `config_version`.

```

### Step 9: Refuse to edit role substrate

This command NEVER writes to `.claude/roles/<role>/`. If the user asks the command to "apply" strip candidates, refuse with: "strip decisions are advisor judgment, not automated. Edit `.claude/roles/<role>/rules.allowlist` directly + commit + dispatch a new task; the next signal row will carry the new config_version."

</workflow>

<file_ownership>

### This command READS

- `.claude/roles/<role>/manifest.yaml`
- `.claude/roles/<role>/rules.allowlist`
- `.claude/roles/<role>/mcp.json`
- `.claude/roles/<role>/system-prompt.md`
- `.claude/rules/*.md` (for byte counts; not modified)
- `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` (read-only sqlite queries — absolute canonical path; works from any lane worktree)
- `scripts/brehon/drain-role-signal-queue.sh` (invoked as subprocess)

### This command NEVER WRITES

- `.claude/roles/**` (strip decisions are advisor judgment)
- `.project-memory/memory.db` (consumer only; drain-script writes via CLI)
- `.claude/decision-queue.json` (no DQ entries — this is a read-only consumer)
- `crates/**`, `migrations/**`, `tests/**` (always out-of-scope)

### Side effects (via drain pre-step)

- `bash scripts/brehon/drain-role-signal-queue.sh` may write new rows to canonical PMD via `write-role-signal` CLI. These are signal rows already authored by EliteDesk Junior workers — not new content.
- May write to `.claude/role-signal-drain/.drained-rows` (dedup index).

</file_ownership>

<see_also>

- `.claude/rules/pmd-invariants.md` invariant #1 — canonical PMD absolute path discipline (this command's hardcoded path enforces it)
- `.claude/rules/multi-lane-worktree.md` §"PMD is cross-lane shared, NOT per-lane isolated" — why this command works identically from canonical + lane worktrees
- `.claude/PRPs/handovers/role-customization-2026-05-24-session2.md` §3.2 — original T4a spec (lean-shape pivot)
- `.claude/PRPs/handovers/role-customization-2026-05-24-session4.md` §4.1 — current state (data unblocked, first real row #535)
- `.claude/hooks/role-signal-utilisation.sh` — signal authoring (content JSON schema lives here)
- `scripts/brehon/drain-role-signal-queue.sh` — pre-step drain pipeline
- `.claude/roles/<role>/` — substrate this command reads
- `.claude/rules/post-task-retro.md` — retro authoring (joined for outcome rate)

</see_also>
