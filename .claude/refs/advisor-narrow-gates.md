# Advisor narrow gates (low-frequency, externalized from advisor-orchestrator.md)

Two gates that fire at low frequency (~1× per quarter for the dogfood gate; even rarer for the retrofit gate). Externalized from `.claude/rules/advisor-orchestrator.md` §3.7 + §3.8 because always-loading them at session start was waste — they only matter at narrowly-scoped authoring moments. Read on demand when their trigger condition holds.

Companion: `.claude/rules/advisor-orchestrator.md` (the always-loaded stage shape + gates; this file is its low-frequency tail).

## Dogfood gate (new slash commands)

Per `feedback_dogfood_slash_command_specs.md`. Every new `.claude/commands/<verb>.md` must include a "Pre-commit dogfood" sub-section under `<rationale>` naming the real existing input the command was walked-through against, what worked, what didn't.

| Command class | Dogfood target |
|---|---|
| Planning-stage (e.g. `/brehon-clarify`) | Most-recent `.claude/PRPs/briefs/<phase>-planning-N.md` |
| Impl-stage | Most-recent `.claude/PRPs/briefs/<phase>-impl-N.md` |
| Verification (e.g. `/brehon-verify`) | Most-recent `.claude/PRPs/plans/<phase>.plan.md` |
| BM verb | Most-recent `.claude/runlog/<phase>.md` |

Cost of pre-commit dogfood ~5 min; cost of post-deploy fix ~10× that.

**Trigger condition:** the advisor is authoring a new `.claude/commands/<verb>.md` file. Read this section then; otherwise skip.

## Schema-changing-spec retrofit gate (plan-mode shape changes)

Per `feedback_schema_changing_spec_retrofit_question.md`. When plan-mode produces a plan that changes the shape of an artifact class (new section in a template, new marker in a section, new field in JSON/YAML schema, new required sub-section in a frontmatter), advisor calls `AskUserQuestion` **once before `ExitPlanMode`**:

- "The new pattern applies forward-only to artifacts authored after this lands. Should I also retrofit the existing artifact(s) [<list>] in a follow-up commit?"
- Options: "Retrofit all" / "Retrofit named subset" / "Forward-only (no retrofit)".

Answer goes into the plan's "Out of scope" or a new "Retrofit scope" section verbatim. "Forward-only" → plan ships with explicit "Pre-existing X are not affected; retrofit deferred indefinitely". "Retrofit" → Phase Z appended at end of implementation phases.

**Skip when:** purely additive functionality (new commands not changing existing shapes), bug fixes (retrofit IS the work), plans explicitly limited to one artifact.

**Trigger condition:** plan-mode produces a plan that changes the shape of an artifact class. Read this section then; otherwise skip.

## See also

- `.claude/rules/advisor-orchestrator.md` §3 "Stages and gates" — the always-loaded gate body. The one-line pointer at §3.7 in that file refers here.
- `.claude/lessons/feedback_dogfood_slash_command_specs.md` — source lesson for the dogfood gate.
- `.claude/lessons/feedback_schema_changing_spec_retrofit_question.md` — source lesson for the retrofit gate.
