# AGENTS.md — Pi entry point for Lemmy/Brehon

This is a **dual-harness repo**:

- **Pi sessions** (you, if you're reading this) load this file plus `.pi/PROJECT_CONTEXT.md` (auto-injected by `.pi/extensions/lemmy-hooks.ts`).
- **Claude Code sessions** load `CLAUDE.md` at this same root. Both harnesses share the four-role model; pi is the **advisor** role in that model.

## Rules for pi sessions

1. **The four-role advisor/planning/impl/bm model is the default working pattern.** Pi sessions on this repo are the **advisor** role. The advisor's job is meta-oversight: author briefs at `.claude/PRPs/briefs/<phase>-<role>-<n>.md`, queue `[role:planning]` / `[role:impl-task]` / `[role:bm-task]` tasks on the EliteDesk Junior daemon, poll, run user gates. **The advisor does NOT author plan files, Rust code, migrations, or PRs directly.** The full model is in `CLAUDE.md` + `.claude/rules/advisor-orchestrator.md`; the pi-side mirror is in `.pi/PROJECT_CONTEXT.md` §"Four-role advisor pattern". The rule + the failure case that motivated it: `.claude/lessons/feedback_pi_advisor_role_dispatches_to_junior.md`.
2. **`.claude/PRPs/` has split ownership.**
   - Four-role-owned (advisor touches only via the dispatch surface, never direct edits): `.claude/PRPs/plans/`, `.claude/decision-queue.json`, `.claude/runlog/`.
   - Advisor-authored (allowed in harness-maintenance mode): `.claude/PRPs/briefs/`, `.claude/PRPs/reports/`, `.claude/lessons/`.
   - The advisor reads from all of these when the task touches them; writes only to the advisor-authored subset.
3. **Do read** the canonical project docs when relevant:
   - `.pi/PROJECT_CONTEXT.md` — pi-specific rules, cargo wrappers, four-role mirror
   - `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md`
   - `docs/brehon-law-inspired-network/04-data-model-and-api.md`
   - `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` (ADRs — append-only)
   - `CLAUDE.md` + `.claude/rules/advisor-orchestrator.md` when the four-role model needs detail beyond what `.pi/PROJECT_CONTEXT.md` carries
4. **Never write Rust without a plan file** in `.claude/PRPs/plans/` — this is a Brehon hard constraint that applies to BOTH harnesses. Plan files are produced by the `[role:planning]` worker, not the advisor.
5. **Do not modify `.claude/PRPs/plans/`, `.claude/decision-queue.json`, or `.claude/runlog/` directly** — those are the four-role dispatch surface. The advisor's only writes to `.claude/` are the advisor-authored subset (rule 2) plus the harness-config exceptions (this file, `.pi/PROJECT_CONTEXT.md`, `.pi/extensions/*.ts`).

The substantive pi context (Brehon constraints, Rust commands, error conventions, four-role mirror) lives in `.pi/PROJECT_CONTEXT.md` and is injected automatically.

## Subagents (delegate, don't load)

For GitHub Actions / CI work and Brehon Branch Manager verbs, prefer
delegating to a project-scope subagent rather than loading the full
detail into your own context. `.pi/agents/ci-debug.md` and
`.pi/agents/bm-pi.md` are the two harnesses; they auto-load focused
context (`.claude/lessons/feedback_gha_pi_loop_postmortem.md` for
ci-debug; `.pi/skills/bm-task/SKILL.md` + `.claude/rules/branch-manager.md`
for bm-pi). Invoke with `agentScope: "both"`. Full setup + rationale
in `.pi/PROJECT_CONTEXT.md` §"Project subagents — delegate, don't load".
