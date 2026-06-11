---
phase: test
role: planning
brief_n: 1
authored: 2026-06-11
base_branch: governance-v0
---

# [role:planning] test dogfood plan — see .claude/PRPs/briefs/test-planning-1.md

## §1 Role + dispatch

`[role:planning] test dogfood sandbox plan — schema-v3 /auto-phase context-management wiring`

Actual `mcp__junior-brehon__create_task` description (single line, <100 chars):

```
[role:planning] test dogfood plan — see .claude/PRPs/briefs/test-planning-1.md
```

## §2 Scope

Produce **one plan file** at `.claude/PRPs/plans/test.plan.md` for the `test` dogfood sub-phase.

**Purpose:** This is a sandbox sub-phase to exercise the `/auto-phase` schema-v3 context-management wiring (stage_digests ring, spill guard, auto-handover refresh, digest-first resume) WITHOUT committing real Brehon governance scope. The plan should define 2-3 small, low-risk tasks that:

1. Touch only non-cargo, non-migration files (to avoid triggering Shape G ci-watcher cycles or e2e runs in a sandbox).
2. Have clear, verifiable DoD lines that the advisor can smoke-test.
3. Demonstrate the plan template structure the auto-phase state machine expects.

**Suggested task scope (advisory — planner may refine):**

- **Task 0 (pre-flight):** Harness audit — verify `.claude/hooks/` registration, `list_hooks` on daemon, confirm Telegram completion hook absent (known non-gating). Write a short `test-harness-check.md` report to `.claude/PRPs/reports/`.
- **Task 1:** Add a `test` entry to `.claude/runlog/test-runlog.md` documenting the auto-phase dogfood observation checklist (first impl task — exercises the impl-task brief authoring, PMD write, LESSON: trailer discipline). Files: `.claude/runlog/test-runlog.md`. No Rust, no migrations.
- **Task 2 [P] (optional):** If planner judges a second parallel task safe, author a second task that writes a short `auto-phase-test-observations.md` to `.claude/PRPs/reports/`. The `[P]` marker exercises the cohort-dispatch wiring.

**Hard scope constraints:**
- No `crates/**`, `migrations/**`, `tests/**` changes.
- No `docs/brehon-law-inspired-network/**` changes.
- No new cargo features, no new dependencies.
- DoD lines must be verifiable by file existence + content check (no `cargo check` needed).
- Plan MUST follow the canonical 20-section template at `.claude/PRPs/templates/plan.template.md`.

**Plan file deliverable:**
- Path: `.claude/PRPs/plans/test.plan.md`
- Must include §5 complexity score, §13 task list with FILES YAML (`creates:` / `modifies:`), §15 DoD commands, §16a Stories block.
- Commit the plan file on `governance-v0` and push.

## §3 Required reading

- `.claude/PRPs/templates/plan.template.md` — the canonical 20-section plan template (follow exactly).
- `.claude/PRPs/plans/v1-validate-agent.plan.md` — most recently authored plan; mirror §1..§14 + §16..§20 shape.
- `.claude/rules/advisor-orchestrator.md` §"Mandatory file-class lesson injection" — the planner must include §16a Stories block.
- `.claude/rules/decision-queue.md` — DQ schema (planner may pre-seed `kind: "log"` entries for notable decisions).
- `.claude/agents/planning.md` — planning subagent contract + commit discipline.

## §4 Constraints

- Plan file MUST be at `.claude/PRPs/plans/test.plan.md` — no other path.
- Commit the plan on `governance-v0` (not `phase-test` — planning briefs and plans go on trunk per the brief-location rules).
- `answered_by` on any DQ entry: `"planner"` (never `"advisor"` or `"user"`).
- `kind: "blocker"` only if genuinely blocked; prefer `kind: "log"` for observations.
- Do NOT author impl code. Do NOT edit `crates/**` or `migrations/**`.
- The plan's §15 DoD MUST be executable by `cat <file>` or `ls <path>` — no cargo commands (this is a sandbox/meta phase; no Rust compilation).
- LESSON: trailer in the commit body per PMD invariant #4 (`feedback_junior_pmd_write_convention.md`).
