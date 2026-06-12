Query the PMD for role-signal utilisation rows and report per-role context health.

Argument: $ARGUMENTS (optional). Forms:
- empty → report all four roles
- `bm-task` / `planning` / `impl-task` / `ci-watcher` → single-role focus

This command is **read-only**. No files written, no tasks queued.

## Purpose

The role-customization initiative (`.claude/PRPs/handovers/role-customization-2026-05-24-session3.md`)
ships `.claude/roles/<role>/` manifests (rules.allowlist, mcp.json, system-prompt.md) as the
substrate for per-role context strips. This command consumes the `role-signal` rows written by
`role-signal-utilisation.sh` on task Stop to answer:

1. **Which rules did this role actually Read?** (strip candidates = rules in allowlist never seen)
2. **Which MCP tools did this role actually invoke?** (strip candidates = MCPs loaded but never called)
3. **How many DQ blockers did tasks of this role add?** (proxy for planning quality)
4. **How many tasks have reported so far?** (signal volume — below 5 = too early to strip)

## Procedure

### Step 1 — drain queue first

Before querying PMD, pull any new EliteDesk signals:

```bash
bash scripts/brehon/drain-role-signal-queue.sh
```

Skip if CWD is not the brehon-fork repo or if ssh homeserver is unreachable (offline).

### Step 2 — query PMD for role-signal rows

Use `memory_search_hybrid` to retrieve all role-signal utilisation rows, filtered by role if
`$ARGUMENTS` names one. Fetch up to 100 rows (signal corpus is small):

```
memory_search_hybrid(query: "role-signal utilisation", tags: "role-signal", limit: 100)
```

If `$ARGUMENTS` is a specific role (e.g. `bm-task`), also pass `tags: "role-signal,role:bm-task"`.

### Step 3 — aggregate per role

For each role with ≥1 row, compute:

| Metric | How |
|--------|-----|
| `n_tasks` | count of distinct `task_id` values |
| `rules_seen` | union of all `rules_read` arrays across rows |
| `mcps_seen` | union of all `mcp_tools_invoked` arrays across rows |
| `dq_blockers_total` | sum of `dq_blockers_added` across rows |
| `config_versions` | distinct `config_version` values (tracks manifest changes) |

Then load the role's allowlist to find strip candidates:

```bash
cat .claude/roles/<role>/rules.allowlist
cat .claude/roles/<role>/mcp.json
```

Strip candidates:
- **Rules**: entries in `rules.allowlist` NOT present in `rules_seen`
- **MCPs**: server names in `mcp.json` whose tools never appear in `mcps_seen`

### Step 4 — synthesize and report

Output a compact report per role. Example shape:

```
## bm-task (n=6 tasks, config=a6a21c07)

Rules loaded: 24 (from rules.allowlist)
Rules seen:   branch-manager.md, gh-pr-fork-target.md, decision-queue.md
Strip candidates (never read in 6 tasks):
  - cargo-output-capture.md
  - no-cargo-output-paste.md
  - pre-phase-harness-audit.md
  ... (+18 more)

MCPs loaded: junior-brehon, project-memory, ref-context
MCPs seen:   mcp__junior-brehon__, mcp__project-memory__
Strip candidates: ref-context (0 invocations)

DQ blockers added: 2 across 6 tasks
Signal volume: SUFFICIENT (≥5 tasks)
```

If `n_tasks < 5` for a role: print `Signal volume: LOW — strip candidates tentative`.

### Step 5 — drain status

Report the current `.claude/role-signal-drain/.drained-rows` line count and the
EliteDesk queue size (via `ssh homeserver "wc -l /srv/brehon-fork/.claude/role-signal-queue.jsonl 2>/dev/null || echo 0"`).

### Step 5b — dispatch-vs-signal-rate health check (MANDATORY surface)

Compute and surface the **ratio of Junior dispatches to signal rows
written in the last 24h**. This catches the "hook is silently exit-0
on the role gate" class of defect (4× recurrence on
`.claude/hooks/role-signal-utilisation.sh` across 24h+ windows — see
`.claude/lessons/feedback_role_detection_smoke_three_shapes.md`).

```bash
# Count Junior tasks in the last 24h (any status)
# via mcp__junior-brehon__list_tasks; filter created_at >= now-24h.
# Compute N_DISPATCHES = count of distinct task_ids.

# Count role-signal rows created in the last 24h:
# from Step 2's memory_search_hybrid results, filter created_at >= now-24h.
# Compute N_SIGNALS = count.

# Surface a line of the shape:
#   Dispatch-vs-signal (last 24h): <N_DISPATCHES> dispatches, <N_SIGNALS> signals.
# Plus a WARN line if N_DISPATCHES > 0 and N_SIGNALS == 0:
#   WARN: signal corpus has not grown in 24h despite N dispatches —
#         likely defect in .claude/hooks/role-signal-utilisation.sh
#         role gate. Run smoke harness:
#         bash .claude/hooks/test-role-signal-utilisation.sh
```

The WARN converts what was previously interpretation-dependent
("the corpus is small; maybe waiting for real work") into a
mechanical surface. The defect fixed in 10b29ec4d was 24h+ latent;
this surface would have caught it in 1 turn.

When the harness shows FAIL on any positive shape, that's the
diagnostic — go fix the hook and run the harness again until it
shows 4+ PASS on all real-shape cases.

## Interpretation guidance

- **High strip-candidate count** on a role = the manifest is loading rules that role never touches.
  Worth filing a DQ or proposing a `rules.allowlist` edit. Do NOT auto-strip without user gate.
- **All `rules_read: []`** = hook is firing but transcript parse found no `.claude/rules/*.md` Reads.
  This is expected for Haiku bm-task workers that skip explicit rule reads and rely on CLAUDE.md.
  Not a bug — just means the strip signal comes from the allowlist side, not observed reads.
  **BUT** — combine with Step 5b's dispatch-vs-signal rate. If `rules_read: []` AND
  `N_DISPATCHES > 0` AND `N_SIGNALS == 0` over 24h, the hook is NOT firing — it's silently
  exit-0'ing on the role gate. Distinguish the two cases:
  - "Empty arrays AND `config_version` is the live manifest SHA AND `N_SIGNALS > 0`" =
    real worker that just didn't `Read` anything (expected for Haiku).
  - "Empty arrays AND `N_SIGNALS == 0` over 24h despite dispatches" = instrumentation
    bug. Run smoke harness; check Step 5b's WARN.
- **Config version `pre-t2`** = signal from before the roles substrate was committed. Discount
  for strip analysis (manifest state unknown at task time).
- **`dq_blockers_total > 2n_tasks`** = role is generating many blockers relative to tasks dispatched.
  Surface to user as a planning-quality concern, not a context-strip concern.
