# .claude/rules/archived/

Rules parked here are **not auto-loaded** but preserved for reference. A rule lands here when its triggering context is dormant (completed phase, retired workflow, superseded by a newer rule) but the content may still be useful if similar work returns.

## How Claude Code treats this directory

Claude Code auto-loads `.claude/rules/*.md` at session start. **Subdirectories are not recursively auto-loaded** — files under `.claude/rules/archived/` load only when explicitly Read. This gives us a zero-cost way to retire a rule without deleting it.

## Current archive

| File | Retired when | Re-surface condition |
|---|---|---|
| `task-hopper.md` | 2026-04-23, after Phase 6 federation closed and v1 sub-phases reverted to single-ralph execution | Re-introduce parallel-agent layered execution (2+ concurrent agents per task) |
