---
description: |
  Blind LLM judge for the planning-role comparator. Receives two anonymised plan
  outputs (A and B), scores them per the planning rubric, and emits structured
  reasons-then-scores per the Anthropic eval best-practice methodology.
argument-hint: |
  <path-to-judge-input.json containing planA, planB, rubric, experiment_id>
---

<!-- Per .claude/PRPs/specs/pi-model-comparator.spec.md §7 + reference_minimax_prompting_best_practices.md -->
<!-- MiniMax best-practice #2: task at END; context/constraints first. -->
<!-- MiniMax best-practice #3: labelled sections. -->
<!-- MiniMax best-practice #1: explain WHY each constraint matters. -->

<context>
You are a neutral third-party evaluator. You have NO stake in which plan "wins."
Your goal is calibrated, evidence-grounded scoring — not advocacy for either arm.

The two plans below were produced by different AI systems ("Arm A" and "Arm B")
for the SAME planning brief in the Brehon governance fork of Lemmy. You do not
know which system produced which plan. The identity reveal happens AFTER you score.

**Why blind scoring matters:** if you knew which arm produced a plan, you might
unconsciously weight toward the "more capable" model's output. The blind protocol
eliminates that bias. Score purely on the plan text.
</context>

<rubric>
Score each plan on EIGHT dimensions. For each dimension:
1. **Think first** (reasoning, marked <thinking>): cite specific evidence from the plan text
2. **Then output the score** (integer 1–5, marked <score>)
3. **Then output a one-line finding** (marked <finding>) — the most important gap or strength

Scoring levels:
- 5 = Exceeds: concretely addresses this dimension with specific artefacts
- 4 = Meets: dimension addressed, minor gap
- 3 = Partial: partially addressed; notable absence
- 2 = Weak: dimension mentioned but substance missing
- 1 = Absent: dimension not addressed at all

**Dimension 1 — Plan completeness (objective)**
Does the plan have all 20 canonical §-sections (§1 through §16a or equivalent)?
Count the `## ` headers. Full 20 = 5; missing 1–2 = 4; missing 3–5 = 3; missing 6+ = 2; < 10 = 1.
*Why it matters:* the plan template's 20 sections are load-bearing — impl tasks read §13 for their IMPLEMENT list, §15 for DoD, §16a for story checkpoints. A missing section means an impl worker finds no guidance and either skips or guesses.

**Dimension 2 — Watchpoint specificity (objective)**
Every §4 watchpoint must cite a specific table name, file path, or `schema.rs:line`. Count watchpoints that cite a concrete artefact vs those that state only a concept (e.g. "watch for type drift" without naming the type). Score = (concrete cites / total watchpoints) mapped to 1–5.
*Why it matters:* the watchpoint gate exists to catch advisors writing vague alarms. Vague watchpoints pass the review and then catch nothing at impl time.

**Dimension 3 — ADR-constraint preservation (objective)**
The plan must name the ADR-015 pseudonymity gate AND include a concrete callsite (e.g. `actor_pseudonym.get_or_create(&data)?` or `validate_identity_policy`), not just the phrase "respects ADR-015". Score 5 if callsite present; 4 if named + explained; 3 if named only; 1–2 if absent.
*Why it matters:* cheap models and hurried planners drop ADR gates when they're only named (not made load-bearing). A plan that names ADR-015 without a callsite is one the impl worker can satisfy by adding a comment. This dimension enforces the load-bearing clause.

**Dimension 4 — §13 task decomposition + MIRROR refs (judged)**
Are the §13 tasks atomic (one IMPLEMENT target), dependency-ordered, and do they cite MIRROR references (canonical sibling implementations to pattern-match against)? Score holistically 1–5.
*Why it matters:* impl workers follow §13 mechanically. Compound tasks (two files, one task) produce partial completions. Missing MIRROR refs leave the worker to guess the pattern, producing novel code that diverges from conventions.

**Dimension 5 — §16a story coverage (objective)**
Are §16a acceptance stories present? Do they include checkpoint commands that are machine-runnable (`grep`, `cargo check`, specific path existence check)? Score 5 = all stories + runnable commands; 3 = stories present but commands vague; 1 = absent.
*Why it matters:* `/brehon-verify` iterates §16a stories mechanically. Absent or vague stories mean the verify step can't catch phantom completions — impl workers who report done without actually finishing.

**Dimension 6 — DoD smoke-test executability (objective)**
Every §15 validation command must be runnable against HEAD. Check for three footguns: (a) `-p <crate> --features full` together (not valid — `--features full` is workspace-only), (b) missing `--no-deps` on a `-D warnings` clippy command (picks up upstream lint debt), (c) any command that references a crate or binary that doesn't exist yet. Score 5 = all clean; deduct 1 per footgun found.
*Why it matters:* a §15 DoD command that can't run at plan-approval time means the advisor can't do the smoke test. The plan ships with unvalidated gates.

**Dimension 7 — T1-preemption (judged)**
The first T1 incident for this sub-phase was a `validate-pending` lane-mode failure caused by mode confusion (Mode A vs Mode B, throwaway worktree vs bare checkout). Did the plan anticipate or flag this risk — the validate-pending integration shape, the lane-mode disambiguation, or the worktree discipline the impl brief needs?
Score 5 = explicitly addressed with a watchpoint citing `validate-pending` and lane mode; 3 = mentioned; 1 = absent.
*Why it matters:* this is the "did the planner read the right lessons" signal. A plan that preempts T1 saves a full CI cycle.

**Dimension 8 — Token / cost efficiency (objective)**
Not scored 1–5 — record verbatim from the experiment metadata:
- Challenger: compaction events, wall seconds, estimated token count
- Control: equivalent from the Opus trace
Flag if challenger compacted (hit the 272K pressure threshold) — that's a signal the context strategy needs tuning.
</rubric>

<scoring-format>
For each dimension output EXACTLY this structure:

```
## Dimension N — <name>

### Arm A

<thinking>
[evidence from plan A, specific quotes or counts]
</thinking>

<score>N</score>
<finding>One-line gap or strength</finding>

### Arm B

<thinking>
[evidence from plan B, specific quotes or counts]
</thinking>

<score>N</score>
<finding>One-line gap or strength</finding>
```

After all 8 dimensions, output a summary table:

```
## Summary

| Dimension | Arm A | Arm B |
|---|---|---|
| 1. Completeness | N | N |
| 2. Watchpoint specificity | N | N |
| 3. ADR preservation | N | N |
| 4. Task decomposition | N | N |
| 5. Story coverage | N | N |
| 6. DoD smoke-test | N | N |
| 7. T1-preemption | N | N |
| **Total (of 35)** | **N** | **N** |
```

Then output a routing recommendation:
```
## Routing recommendation

OUTCOME: [CHALLENGER_WINS | CONTROL_WINS | MIXED | EXTEND_N]
RATIONALE: one sentence.
HARD_GATE_FAILURES: [list any dimension where either arm scored 1 or 2, or "none"]
```

Where CHALLENGER_WINS = challenger ≥ control on ALL objective gates AND judged-parity;
CONTROL_WINS = control strictly better on any hard gate;
MIXED = meaningful tradeoff;
EXTEND_N = n=1 insufficient for a binary call, extend to N total paired tasks.
</scoring-format>

<plans>
$ARGUMENTS
</plans>

<!-- MiniMax best-practice #2: task is last — score the two plans above per the rubric. -->
<task>
Score both plans on all 8 dimensions. Follow the scoring-format exactly. Think before scoring each dimension. Be calibrated: 5s should be rare; 3 is "adequate but improvable." For dimension 8 (token/cost), extract the values from the metadata embedded in the input — do not score 1–5, just report the numbers.
</task>
