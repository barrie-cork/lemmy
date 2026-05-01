---
paths:
  - ".claude/decision-queue.json"
  - ".claude/PRPs/briefs/**ci-watcher**"
  - ".claude/PRPs/briefs/**fix-impl**"
  - ".claude/rules/**"
  - ".claude/commands/**"
  - ".claude/lessons/**"
  - ".claude/PRPs/templates/**"
---

# Brehon validate-stage + spec-authorship gates

> Path-scoped sub-rule of `advisor-orchestrator.md`. Loads when the advisor opens decision-queue or ci-watcher/fix-impl briefs (validate-stage triage) OR when authoring new rules/commands/lessons/templates (spec-authorship gates). Brehon-fork canonical mirror at `brehon-fork/.claude/rules/advisor-orchestrator.md` (one file, all sections inline).

## §G4 classifier

Per `.claude/PRPs/plans/v1-validate-agent.plan.md` §4 watchpoint #7
+ §10.9. When a `validate-pending` DQ entry is mutated to
`result: "fail" | "cancelled" | "timed_out"` and remains in `pending[]`,
the advisor reads its `result`, `log_slice`, and `failed_jobs`, then
applies the classifier:

**Allowlist (auto-queue narrow fix-impl-task, ≤3 file edits):**

| Failure signature | Auto-fix | Source lesson |
|---|---|---|
| `clippy::doc_lazy_continuation` warning | reword + mid-paragraph "and" | `feedback_clippy_doc_lazy_continuation_in_doc_comments.md` |
| `error[E0432]: unresolved import` | add the missing `use` per the suggestion | n/a (mechanical) |
| `warning: use of deprecated <api>` | replace with the suggested replacement | n/a (mechanical) |

For an allowlist match, the advisor authors a narrow fix-impl-task
brief at `.claude/PRPs/briefs/<phase>-fix-impl-<n>.md` containing:
the failed-job log slice (≤200 lines), the specific file:line cited
by the lint, the auto-fix recipe from the source lesson (or
mechanical replacement), and a hard cap "≤3 file edits". The brief
is dispatched as a normal `[role:impl-task]` Junior task; the
resulting commit lands on the phase branch and re-triggers the
workflow.

**Non-allowlist (catch-fire to user):**

- compile errors (any `error[E*]` other than `E0432`)
- test failures (panics, assertion fails, e2e flakes, testcontainers
  issues)
- timeout / OOM / runner death
- any failure whose log slice doesn't match a row in the allowlist

Surface as: "validate-failed on `<branch>` (workflow run `<id>`):
non-allowlist failure. Failed jobs: `<failed_jobs>`. Log slice
attached. Surfaced to user — no auto-fix attempted."

The allowlist is **conservative by design** (per
`feedback_principles_not_rules.md` — guidance over rigid rules).
Grow it only on retro evidence: if a CR-triage cycle classifies a
non-allowlist failure as "this could have been auto-fixed", record
it in the retro §5 watch-items and add to the allowlist on the next
sub-phase's plan if the pattern reproduces.

## Canonical-schema-first gate (mandatory before authoring any spec)

Per `.claude/lessons/feedback_read_canonical_before_writing_spec.md`: before the advisor (or any subagent the advisor dispatches) authors a new spec, template, or rule that prescribes the shape of an artifact, `Glob` + `Read` 1-2 existing canonical instances of that artifact class first.

This applies to:

- **New rules** under `.claude/rules/` — read 1-2 sibling rules to match the section-header style and the "auto-loaded" + "cite by filename" conventions.
- **New commands** under `.claude/commands/` — read 1-2 sibling commands (`bm/<verb>.md` or `prp-core/<verb>.md`) to match frontmatter shape (`description:`, `argument-hint:`) and the `<objective>` / `<workflow>` / `<hard-refusals>` block conventions.
- **New lessons** under `.claude/lessons/` — read 1-2 sibling lessons to match the `name: / description: / type: feedback` frontmatter and the "Why / How to apply / Generalises to / Symptom to recognise" body shape.
- **New templates** under `.claude/PRPs/templates/` — read the canonical instances of the artifact the template prescribes (e.g. for `plan.template.md`, read `phase-v1-JM-a.plan.md` + `v1-jury-mechanics-c.plan.md` first; the section schema is §1..§20 with specific titles).
- **Schema additions to existing rules** — read the existing enumeration before adding a value; cite the new value's writers + readers in the same edit.

The gate is mechanical: an advisor (or planner) commit that adds a `*.md` under `.claude/{rules,commands,lessons,PRPs/templates}` without citing a canonical example in the file body or commit body is a process miss. The retro should flag it. Generalises to any spec/template/rule authorship — `grep '^##'` against an existing instance is always worth the 2-second read.

## Dogfood gate (mandatory for new slash commands)

Per `.claude/lessons/feedback_dogfood_slash_command_specs.md`: every new slash command authored under `.claude/commands/` must include a "Pre-commit dogfood" sub-section under its `<rationale>` block. The sub-section names a real existing input the command was mentally walked-through against (a brief, plan, log, or runlog), what worked, and what didn't.

Specific dogfood targets:

- **Planning-stage command** (e.g. `/brehon-clarify`) → most-recent planning brief at `.claude/PRPs/briefs/<phase>-planning-N.md`.
- **Impl-stage command** → most-recent impl brief at `.claude/PRPs/briefs/<phase>-impl-N.md`.
- **Verification command** (e.g. `/brehon-verify`) → most-recent shipped plan at `.claude/PRPs/plans/<phase>.plan.md`.
- **BM verb** → most-recent runlog entry at `.claude/runlog/<phase>.md`.

The gate is mechanical: a commit that adds `.claude/commands/<verb>.md` without a "Pre-commit dogfood" note in the body is a process miss. Prose lints catch typos; dogfood catches semantics. Cost of pre-commit dogfood ≈ 5 minutes; cost of post-deploy fix ≈ 10× that.

## Schema-changing-spec retrofit gate (mandatory in plan-mode for shape changes)

Per `.claude/lessons/feedback_schema_changing_spec_retrofit_question.md`: when an advisor plan-mode session produces a plan that changes the shape of an existing artifact class (new section in a template, new marker in a section, new field in a schema, new required sub-section in a frontmatter), the advisor must call `AskUserQuestion` **once, before `ExitPlanMode`**, asking whether to retrofit existing artifacts.

Triggers:

- **New section in `*.template.md`** (e.g. §16a Stories block in plan.template.md).
- **New marker in an existing section** (e.g. `[P]` in §13 task headers).
- **New field in JSON/YAML schema** (e.g. `kind: "clarify"` in decision-queue.json).
- **New required sub-section in a frontmatter shape** (e.g. dogfood gate's `<rationale>` requirement on `.claude/commands/*.md`).

The question shape:

- "The new pattern applies forward-only to artifacts authored after this lands. Should I also retrofit the existing artifact(s) [<list>] in a follow-up commit?"
- Options: "Retrofit all" / "Retrofit named subset" / "Forward-only (no retrofit)"

The user's answer goes into the plan's "Out of scope" or a new "Retrofit scope" section verbatim. If user picks "Forward-only", the plan ships with an explicit "Pre-existing X are not affected; retrofit deferred indefinitely" line. If user picks retrofit, a Phase Z is added at the end of the implementation phases. Skipping the question is a process miss — the symptom shows up in the post-implementation retro as "should we retrofit X?" appearing as a deferred follow-up.

**When to skip:** plans that add purely additive functionality (new commands that don't change other commands' shape), bug fixes (the retrofit is the work itself), or plans explicitly limited to one artifact.
