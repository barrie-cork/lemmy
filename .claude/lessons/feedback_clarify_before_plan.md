---
name: Clarify-before-plan (spec-kit pattern adoption)
description: Run a structured coverage-questions pass on every planning brief BEFORE the planning Junior task is queued. Pre-fills DQ entries that gate planning. Spec-kit calls this `/specify.clarify`; the Brehon adaptation is `/brehon-clarify`. Collapses the round-trip cost of plan-revision-after-the-fact when ambiguity in the brief surfaces during impl.
type: feedback
---

When the advisor authors a planning brief at `.claude/PRPs/briefs/<phase>-planning-N.md`, run `/brehon-clarify <brief-path>` **before** queueing the Junior planning task. The command runs structured coverage questions over the brief and produces DQ entries with `from: "advisor"` + `kind: "clarify"`. The planning task is gated on every clarify-DQ entry being resolved (advisor-mode self-answer with citation, or user-relay verbatim).

**Why:** Pre-planning ambiguity in briefs surfaces as plan revision rounds during impl. Recent observed loops (per `feedback_advisor_watchpoint_specificity.md`, `feedback_plan_dod_dry_run_at_write.md`) trace to brief sections that didn't say which file, didn't cite which `schema.rs` line, didn't acknowledge a sibling-phase file ownership conflict. The planner produces a plausible plan based on the brief; impl hits the ambiguity; round-trip ensues. Spec-kit's `/specify.clarify` runs the disambiguation pass at the cheapest point in the pipeline. The Brehon adaptation reuses the existing decision-queue v2 schema (`kind: "clarify"` is additive — pre-v2 default treats it as `"blocker"` for backward compat).

**How to apply:**

- **Before queueing any `[role:planning]` Junior task**, run `/brehon-clarify .claude/PRPs/briefs/<phase>-planning-N.md`.
- **Default to `--mode advisor`.** The lessons corpus + resolved DQ + prior plans answer most clarify questions trivially — the advisor session has all of them in working context.
- **Use `--mode user-relay`** only when a question is genuinely judgment-heavy (ADR-affecting, scope-changing, visible-to-others impact). The clarify command does the routing automatically when advisor-mode can't find evidence.
- **Coverage axes (from `.claude/commands/brehon-clarify.md`):**
  - Scope ambiguity (which file/crate?)
  - Undefined input (symbol referenced but not defined)
  - Undefined output (which exact path is the deliverable?)
  - Undeclared constraint (§10 mirror disagrees with §4 constraint?)
  - Missing required-reading (cite the file:line for the pattern)
  - Cross-phase invariant (overlap with sibling phase's file ownership?)
  - DoD executability (will the §15 commands work given this scope?)
  - Watchpoint specificity (cite the table/file/schema.rs line)
- **Refuse to generate a question** whose answer is in the brief, in `.claude/lessons/`, or in a resolved DQ — the corrective is to add the citation to the brief, not file a DQ.
- **Commit the brief edits + DQ entries together** with subject `chore(advisor): clarify <phase>-<role>-N — see DQ #<lo>-#<hi>`.

**When to skip:** impl-task briefs (derive from a clarified plan), bm-task briefs (mechanical), retro briefs (reflective). Always clarify planning briefs.

**Generalizes to:** any structured plan-driven workflow (PRP, ralph, Junior). The pattern is "ambiguity surfaces cheapest at brief-write time, not at plan-write time, and definitely not at impl time." Pre-planning clarification is the equivalent of running tests before running the build.

**Symptom to recognise in retrospect:** a plan revision commit (`docs(plan): <phase> revise §<N> per impl ambiguity`) landing mid-impl is the failure mode `/brehon-clarify` prevents. If you see one, audit the parent planning brief — clarify would have caught it.

**Process gate (load-bearing):** `.claude/rules/advisor-orchestrator.md` "Stage-shape orchestration" lists clarify as the first stage of every sub-phase. Skipping clarify on a planning brief is a process breach the advisor must justify in the planning task's commit body.
