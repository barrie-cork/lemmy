---
name: Handover commands
description: Two user-invocable /handover-* commands for advisor + impl session park/resume briefs; live under .claude/commands/handover/
type: reference
originSessionId: 10156cd8-266b-4685-89b7-1f5b4ea2d04b
---
Two slash commands for writing a resume brief when a Brehon session closes (PC restart, DQ-batch closure, phase transition, task-N→N+1 boundary, blocked-on-advisor):

- `/handover-advisor` — primary worktree on `governance-v0`; writes `advisor-<ISO-date>-<slug>.md`
- `/handover-impl` — phase worktree on `phase-*`; writes `impl-<ISO-date>-<slug>.md`

Both take optional arg 1 (slug) and `--no-write` (dry-run). Output lands in `.claude/PRPs/handovers/` (tracked in git). Advisor appends to `.claude/runlog/bm-runlog.md`; impl appends to `.claude/runlog/<phase>-runlog.md` if present else skips (never cross-branch writes).

**Shape:** mirrors `/bm-cut` 5-phase structure — parse args, attribution guard, sample state, detect pending, write brief + emit bootstrap prompt.

**Shared invariants:** `.claude/rules/handover.md` (attribution, file ownership, what-never-to-include, when-to-write, 150–300 line target).

**Files:**
- `.claude/commands/handover/handover-advisor.md`
- `.claude/commands/handover/handover-impl.md`
- `.claude/rules/handover.md`

**Created:** 2026-04-24 ahead of planned v1-JM-a retro (pattern stable after 6 hand-written exemplars under `.claude/PRPs/handovers/` + `.claude/PRPs/reports/*-handover-*.md`).

**Not yet built:** `/handover-bm`. BM state already line-by-line reconstructable from `bm-runlog.md`; revisit after v1-JM-b if BM sessions accumulate state the runlog doesn't capture.

**Canonical exemplars the commands mirror:**
- Advisor: `.claude/PRPs/reports/v1-AD-c-advisor-resume-state.md` (204 lines)
- Impl: `.claude/PRPs/reports/phase-5a-handover-task-54-onward.md` (200+ lines)
- Bootstrap prompt: `.claude/PRPs/handovers/v1-prep-bootstrap.md` (29 lines)
