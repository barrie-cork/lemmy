# impl-task role

Executes one implementation task from a Brehon sub-phase plan. Pinned to Sonnet 4.6 (medium effort) — pattern-following from MIRROR refs, not heavy reasoning. Falls back to a clean DQ pending entry instead of guessing.

See `.claude/agents/impl-task.md` for the canonical subagent contract (allowed tools, file-ownership boundaries, refusal patterns).

## Iteration 1 (baseline — substrate only, no functional change)

**The daemon does NOT yet honour this manifest.** Junior workers still dispatch with full auto-load of `CLAUDE.md`, `.claude/rules/*.md` (all 24), `.claude/skills/`, and the canonical `.mcp.json`. The files here are deliberate baselines:

- `manifest.yaml` — role metadata + signal-emission settings.
- `rules.allowlist` — all 24 rule filenames (no strips).
- `mcp.json` — verbatim copy of canonical `.mcp.json` with secrets replaced by `${ENV_VAR}` placeholders.
- `system-prompt.md` — empty (relies on `CLAUDE.md` + rules).

The point of iteration 1 is to create the **durable, git-tracked home** for future strips and additions — not to strip anything yet. Each future edit is a tracked commit whose subtree SHA (`git rev-parse HEAD:.claude/roles/impl-task`) becomes the `config_version` tag on RLS signals.

## Why impl-task gets special attention

impl-task is the **highest-volume role** in the four-role model — most Junior dispatches per sub-phase are impl-tasks. Its signal corpus will be the richest input to `/check-role-health`. Expect the most strip candidates to surface here first.

Two corollaries:

- Read-utilisation hooks fire most often for this role. Hook B (utilisation) should produce stable median-Read-count statistics after ~10 impl-task dispatches.
- The cohort-dispatch pattern (per `.claude/rules/advisor-orchestrator.md` §4) means multiple impl-task workers may dispatch in parallel from the same advisor invocation. Each worker emits its own signal triad — `config_version` will be identical across parallel members of the same cohort, which is correct and load-bearing for cohort-level analysis later.

## When to edit

Edit after `/check-role-health impl-task` (Task 4 — pending) reports stable strip candidates from accumulated `role-signal` rows in PMD. Do NOT strip blind — wait for evidence.

The RLS loop is:

1. Hooks (Task 3 — pending) emit three `role-signal` rows per Junior dispatch (`kind: startup-cost / utilisation / outcome`).
2. `/check-role-health impl-task` aggregates last-30-days signals → names rule files / MCP servers never read or invoked.
3. User approves a strip → manual edit here → tracked commit.
4. Next dispatch picks up new `config_version`; signals attribute to the new SHA; before/after measurable.
5. Failure-rate regression after a strip → rollback signal in the next `/check-role-health` report.

## Wiring (deferred to Task 5)

When the daemon dispatcher eventually consumes this manifest, the invocation will be:

```bash
claude -p --bare \
  --settings .claude/roles/impl-task/settings.json \
  --mcp-config .claude/roles/impl-task/mcp.json \
  --append-system-prompt .claude/roles/impl-task/system-prompt.md \
  --allowedTools <derived from rules.allowlist>
```

Until then, the daemon dispatches plain `claude -p` and these files are signal targets only.

## See also

- `.claude/agents/impl-task.md` — subagent contract (canonical).
- `.claude/PRPs/handovers/role-customization-2026-05-24.md` — T2 substrate origin + RLS framing.
- `C:\Users\barri\.claude\plans\i-am-interesting-assessing-ethereal-shore.md` — 5-task plan (the contract for this effort).
- `.claude/rules/pmd-invariants.md` — PMD invariants that the signal hooks must preserve.
