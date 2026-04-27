---
name: Ask one retrofit question before plan-approve when a spec changes the schema
description: When a plan in plan-mode introduces a new shape for an existing artifact class (templates, briefs, rules, schemas), the advisor must ask one explicit question via AskUserQuestion before ExitPlanMode about whether to retrofit existing artifacts. Forward-only-by-default; explicit retrofit on user opt-in. Closes the loop on "the new pattern is shipped but no existing artifact uses it."
type: feedback
---

When a plan-mode session produces a plan that **changes the shape of an existing artifact class** — adds a new section to a plan template, adds a frontmatter field, adds a new `kind:` value to a schema, introduces new task-header markers — the advisor must ask one explicit retrofit question via `AskUserQuestion` before `ExitPlanMode`.

The question shape:

- "The new pattern applies forward-only to artifacts authored after this lands. Should I also retrofit the existing artifact(s) [<list>] in a follow-up commit?"
- Options: "Retrofit all" / "Retrofit named subset" / "Forward-only (no retrofit)"

**Why:** 2026-04-27 spec-kit pattern adoption session. Shipped plan template changes (`[P]` markers in §13, §16a stories block) + schema-additive `kind: "clarify"` to decision-queue.json. The active `v1-jury-mechanics-d.plan.md` doesn't have either — it predates the upgrade. The reflection retrospectively identified this as a missed follow-up: "should I retrofit JM-d, or leave it under the old schema?" That question should have been asked **once, in plan mode, before approval**, not surfaced afterwards.

The retrofit question is parallel to the "what was rejected and why" pattern (which the spec-kit plan did include): both are checkpoints that prevent the "pattern shipped but never applied" failure mode. Schema changes that ship without retrofit guidance leave a graveyard of pre-upgrade artifacts that never benefit from the upgrade.

**How to apply:**

- During plan mode, identify whether the plan changes shape of any existing artifact class. Triggers:
  - New section in a `*.template.md` (e.g. §16a Stories block).
  - New marker in an existing section (e.g. `[P]` in §13 task headers).
  - New field in JSON/YAML schema (e.g. `kind: "clarify"` in decision-queue).
  - New required content in a frontmatter shape (e.g. dogfood gate adding a `<rationale>` sub-section requirement to all `.claude/commands/*.md`).
- If yes, before ExitPlanMode call `AskUserQuestion` with the retrofit question. Phrase it concretely — name the existing artifacts that would qualify for retrofit (e.g. "JM-d plan, JM-c plan, all 4 prior plans").
- The user's answer goes into the plan's "Out of scope" or "Follow-up" section verbatim. The plan is then approved with retrofit scope decided up-front.
- If user picks "Forward-only", the plan ships with an explicit "Pre-existing X are not affected; retrofit deferred indefinitely" line — closing the loop.
- If user picks "Retrofit all" or "Retrofit named subset", the plan adds a Phase Z (retrofit) at the end of the implementation phases.

**When to skip:**

- Plan that adds purely additive functionality (a new slash command that doesn't change other commands' shape).
- Plan that fixes a bug in an existing artifact (the retrofit is the work itself).
- Plan whose scope is explicitly limited to one artifact (no other instance to retrofit).

**Generalises to:** any plan that introduces a new shape, marker, field, or required sub-section that pre-existing artifacts would benefit from. The retrofit question is cheap (one AskUserQuestion call); the cost of skipping it shows up as "pattern adopted but no artifact uses it" in the retro.

**Symptom to recognise in retrospect:** in the post-implementation retro, a question of the form "should we retrofit X?" appears as a deferred follow-up — that question was catchable in plan mode. The retro is the wrong place for it (too late; the plan has shipped).

**Brehon-specific application:**

- Per `.claude/rules/advisor-orchestrator.md` "Schema-changing-spec retrofit gate", every advisor plan-mode session that introduces a new artifact-shape must call `AskUserQuestion` once before `ExitPlanMode` with the retrofit question. The plan file names the chosen retrofit scope (all / subset / forward-only) explicitly. Skipping is a process miss.
