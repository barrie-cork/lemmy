# bm-task role

Executes one Brehon Branch Manager verb (bm-cut, bm-push, bm-pr, bm-status, bm-poll-cr, bm-prp-review, bm-triage, bm-merge, bm-ping). Pinned to Haiku 4.5 (low effort) — git/yq/gh ops, no heavy reasoning.

Distinct from the `branch-manager` foreground subagent. That one is for impl-session use; bm-task is for Junior-dispatched advisor orchestration.

See `.claude/agents/bm-task.md` for the canonical subagent contract (allowed tools: Read, Edit, Bash, Glob, Grep) and the file-ownership boundaries from `.claude/rules/branch-manager.md`.

## Iteration 1 (baseline — substrate only, no functional change)

**The daemon does NOT yet honour this manifest.** Junior workers still dispatch with full auto-load of `CLAUDE.md`, `.claude/rules/*.md` (all 24), `.claude/skills/`, and the canonical `.mcp.json`. The files here are deliberate baselines:

- `manifest.yaml` — role metadata + signal-emission settings.
- `rules.allowlist` — all 24 rule filenames (no strips).
- `mcp.json` — verbatim copy of canonical `.mcp.json` with secrets replaced by `${ENV_VAR}` placeholders.
- `system-prompt.md` — empty (relies on `CLAUDE.md` + rules).

The point of iteration 1 is to create the **durable, git-tracked home** for future strips and additions — not to strip anything yet. Each future edit is a tracked commit whose subtree SHA (`git rev-parse HEAD:.claude/roles/bm-task`) becomes the `config_version` tag on RLS signals.

## Expected strip surface (hypothesis to be validated by RLS)

bm-task is the **lowest-reasoning role** — mechanical git/yq/gh ops. Most of the cargo/Rust/migration/test-focused rules in `.claude/rules/` are likely never read by a bm-task worker. Candidate strip targets (DO NOT strip yet — wait for evidence):

- `cargo-output-capture.md`, `no-cargo-output-paste.md` — bm-task doesn't invoke cargo.
- `view-crate-selectable-template.md` — DB schema concern, not BM territory.
- `pmd-search-strategy.md` — bm-task rarely searches PMD (it writes finding YAMLs, not retros).

The first `/check-role-health bm-task` report should confirm or refute these. If `kind: utilisation` rows show zero Reads of these files across 10+ bm-task dispatches, the strip is evidence-backed.

## When to edit

Edit after `/check-role-health bm-task` (Task 4 — pending) reports stable strip candidates from accumulated `role-signal` rows in PMD. Do NOT strip blind — wait for evidence.

The RLS loop is:

1. Hooks (Task 3 — pending) emit three `role-signal` rows per Junior dispatch (`kind: startup-cost / utilisation / outcome`).
2. `/check-role-health bm-task` aggregates last-30-days signals → names rule files / MCP servers never read or invoked.
3. User approves a strip → manual edit here → tracked commit.
4. Next dispatch picks up new `config_version`; signals attribute to the new SHA; before/after measurable.
5. Failure-rate regression after a strip → rollback signal in the next `/check-role-health` report.

## Wiring (deferred to Task 5)

When the daemon dispatcher eventually consumes this manifest, the invocation will be:

```bash
claude -p --bare \
  --settings .claude/roles/bm-task/settings.json \
  --mcp-config .claude/roles/bm-task/mcp.json \
  --append-system-prompt .claude/roles/bm-task/system-prompt.md \
  --allowedTools <derived from rules.allowlist>
```

Until then, the daemon dispatches plain `claude -p` and these files are signal targets only.

## See also

- `.claude/agents/bm-task.md` — subagent contract (canonical).
- `.claude/rules/branch-manager.md` — BM file-ownership boundaries (HARD).
- `.claude/commands/bm/<verb>.md` — verb scripts the bm-task worker reads.
- `.claude/PRPs/handovers/role-customization-2026-05-24.md` — T2 substrate origin + RLS framing.
- `C:\Users\barri\.claude\plans\i-am-interesting-assessing-ethereal-shore.md` — 5-task plan (the contract for this effort).
