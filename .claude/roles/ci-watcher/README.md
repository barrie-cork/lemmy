# ci-watcher role

Polls a single GitHub Actions workflow run via `gh run watch --exit-status`, then MUTATES the existing `kind: "validate-pending"` DQ entry by matching `workflow_run_id`. Pinned to Haiku 4.5 (low effort) — mechanical polling, no judgment, no fixes.

See `.claude/agents/ci-watcher.md` for the canonical subagent contract (allowed tools: Read, Edit, Write, Bash). Hard refusals: never invokes cargo, never edits `crates/`, never applies clippy auto-fixes, never writes a NEW DQ entry.

The option-2 single-entry mutation pattern (locked 2026-04-28) is documented in `.claude/rules/decision-queue.md` §"ci-watcher mutation pattern".

## Iteration 1 (baseline — substrate only, no functional change)

**The daemon does NOT yet honour this manifest.** Junior workers still dispatch with full auto-load of `CLAUDE.md`, `.claude/rules/*.md` (all 24), `.claude/skills/`, and the canonical `.mcp.json`. The files here are deliberate baselines:

- `manifest.yaml` — role metadata + signal-emission settings.
- `rules.allowlist` — all 24 rule filenames (no strips).
- `mcp.json` — verbatim copy of canonical `.mcp.json` with secrets replaced by `${ENV_VAR}` placeholders.
- `system-prompt.md` — empty (relies on `CLAUDE.md` + rules).

The point of iteration 1 is to create the **durable, git-tracked home** for future strips and additions — not to strip anything yet. Each future edit is a tracked commit whose subtree SHA (`git rev-parse HEAD:.claude/roles/ci-watcher`) becomes the `config_version` tag on RLS signals.

## Expected strip surface (hypothesis to be validated by RLS)

ci-watcher is the **narrowest role** of the four — mechanical workflow-run polling + a single JSON mutation. The vast majority of `.claude/rules/*.md` are likely irrelevant. Candidate strip targets (DO NOT strip yet — wait for evidence):

- All cargo/Rust/migration/test rules — ci-watcher doesn't write code.
- All planning-discipline rules (`pre-phase-harness-audit.md`, `view-crate-selectable-template.md`) — ci-watcher reads the brief, polls, mutates.
- `ref-context` MCP server — ci-watcher doesn't read library docs.
- `tavily` MCP server — ci-watcher doesn't web-search.
- Possibly `junior-brehon` MCP server (ci-watcher doesn't dispatch sub-tasks).

The first `/check-role-health ci-watcher` report should be the most aggressive. The role's expected post-strip rule slate may be as small as 3-5 rules (decision-queue.md, branch-manager.md, integrator.md, advisor-orchestrator.md §"ci-watcher mutation pattern" section, escalation.md).

## When to edit

Edit after `/check-role-health ci-watcher` (Task 4 — pending) reports stable strip candidates from accumulated `role-signal` rows in PMD. Do NOT strip blind — wait for evidence.

ci-watcher dispatches are typically short (~5 min for workspace check, up to 26 min for e2e). Signal accumulation is faster per-task than impl-task, so usable data may arrive within a single sub-phase.

The RLS loop is:

1. Hooks (Task 3 — pending) emit three `role-signal` rows per Junior dispatch (`kind: startup-cost / utilisation / outcome`).
2. `/check-role-health ci-watcher` aggregates last-30-days signals → names rule files / MCP servers never read or invoked.
3. User approves a strip → manual edit here → tracked commit.
4. Next dispatch picks up new `config_version`; signals attribute to the new SHA; before/after measurable.
5. Failure-rate regression after a strip → rollback signal in the next `/check-role-health` report.

## Wiring (deferred to Task 5)

When the daemon dispatcher eventually consumes this manifest, the invocation will be:

```bash
claude -p --bare \
  --settings .claude/roles/ci-watcher/settings.json \
  --mcp-config .claude/roles/ci-watcher/mcp.json \
  --append-system-prompt .claude/roles/ci-watcher/system-prompt.md \
  --allowedTools <derived from rules.allowlist>
```

Until then, the daemon dispatches plain `claude -p` and these files are signal targets only.

## See also

- `.claude/agents/ci-watcher.md` — subagent contract (canonical).
- `.claude/rules/decision-queue.md` §"ci-watcher mutation pattern" — option-2 design.
- `.claude/PRPs/handovers/role-customization-2026-05-24.md` — T2 substrate origin + RLS framing.
- `C:\Users\barri\.claude\plans\i-am-interesting-assessing-ethereal-shore.md` — 5-task plan (the contract for this effort).
