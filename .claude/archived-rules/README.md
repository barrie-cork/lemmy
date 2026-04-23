# .claude/archived-rules/

Rules parked here are **not auto-loaded** but preserved for reference. A rule lands here when its triggering context is dormant (completed phase, retired workflow, superseded by a newer rule) but the content may still be useful if similar work returns.

## How Claude Code treats this directory

Claude Code auto-loads every `.md` file under `.claude/rules/` at session start, **including recursive subdirectories**. To retire a rule without deleting it, move it OUT of `.claude/rules/` entirely — this directory (`.claude/archived-rules/`) is the convention. Files here load only when explicitly Read.

Earlier Phase A (commit `eb8ab5fc1`, 2026-04-23) parked retired rules at `.claude/rules/archived/` assuming subdirectories were not recursed; Phase B verification found they were. The directory was renamed in the Phase B cleanup.

## Current archive

| File | Retired when | Re-surface condition |
|---|---|---|
| `task-hopper.md` | 2026-04-23, after Phase 6 federation closed and v1 sub-phases reverted to single-ralph execution | Re-introduce parallel-agent layered execution (2+ concurrent agents per task) |
