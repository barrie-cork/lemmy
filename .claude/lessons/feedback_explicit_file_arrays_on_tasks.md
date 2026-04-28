---
name: Per-task creates/modifies YAML in §13 is load-bearing for cohort dispatch + brehon-verify
description: Plan §13 tasks declare a machine-parseable FILES YAML block (`creates:` + `modifies:` arrays) above the IMPLEMENT lines. The advisor intersects these across [P]-cohort peers to refuse cohorts with file overlap; /brehon-verify uses `creates:` for the phantom-presence check. Forward-only schema additive — pre-V1 plans keep their prose-only IMPLEMENT lists with back-compat parsing.
type: feedback
---

In `.claude/PRPs/templates/plan.template.md` §13 (Step-by-step tasks), every task header now carries a **FILES** YAML block immediately above its IMPLEMENT prose:

```yaml
creates:
  - <new-file>
modifies:
  - <existing-file>   # what changes (one line)
```

The planner asserts `union(creates, modifies) == set(IMPLEMENT files)` at plan-write time. Drift is a planner-side miss the advisor surfaces via DQ pending entry before any impl runs.

**Why:** Brehon's `[P]` cohort-dispatch rule (per `feedback_parallel_cohort_dispatch.md`) lives or dies on file-set disjointness. Pre-V1 the planner judged disjointness implicitly from the IMPLEMENT lines and the advisor trusted the `[P]` marker. mfs's `prp-breakdown-tasks.md` decomposer (single-command analogue) showed a stronger pattern: each task carries machine-parseable `targetFiles`/`modifiesFiles` arrays so the orchestrator can compute cohort-membership by intersecting JSON arrays. Brehon's adaptation is YAML-in-prose (no sibling artifact, no schema duplication) — the planner emits one shallow YAML block per task that the advisor's cohort-dispatch step 4 intersects mechanically. The same arrays power `/brehon-verify`'s phantom-presence check (`for f in creates: assert exists(f) on phase branch + non-empty`), replacing the older descriptor-grammar parser for plans that have the YAML.

Without this, three failure modes recur:

- **Cohort-merge conflicts at finalize.** Two `[P]` peers each edit the same file (planner missed the overlap); both worktrees commit clean; finalize rebase flags conflict. Mechanical intersect catches this *before* dispatch.
- **Phantom completions.** A task lands its commit with one of three planned files missing; CR catches it eventually but the advisor's pre-merge verify pass doesn't catch it without explicit `creates:` to check against. With YAML, `/brehon-verify` Step 4a is two lines: assert exists, assert non-empty.
- **Cross-cohort file-ownership drift.** Non-`[P]` Task 7 modifies a file that Task 3 created; the chain isn't visible in prose IMPLEMENT lines. With YAML, the union across all §13 tasks is greppable — duplicate `creates:` entries surface immediately.

**How to apply (planner side — see `.claude/agents/planning.md`):**

- Author the FILES YAML block as the **first content** under each task header (above ACTION, IMPLEMENT, MIRROR, GOTCHA, VALIDATE).
- `creates:` lists every path that doesn't exist on the phase branch's base before this task runs (typically `phase-<phase>` cut from `governance-v0`); `modifies:` lists every existing path the task edits. A path is in exactly one of the two arrays — never both.
- One-line trailing comment per `modifies:` entry naming what changes (e.g. `# add pub mod gist_7`). Optional but recommended for cohort review readability.
- After writing the YAML, mentally enumerate the IMPLEMENT lines and assert `union(creates, modifies)` equals the set of files those IMPLEMENT lines name. If they disagree, fix the YAML or the IMPLEMENT lines so they match — drift is a planner-side miss.
- Apply this to `[P]` and non-`[P]` tasks uniformly. Non-`[P]` tasks need YAML too so cross-cohort overlap checks (a `[P]` task whose `creates:` overlaps a still-running non-`[P]` task's `modifies:`) work mechanically.

**How to apply (advisor side — see `.claude/rules/advisor-orchestrator.md` "Cohort dispatch sequence" step 4):**

- Parse the FILES YAML block from §13 for each task in a candidate cohort.
- Compute pairwise intersections of `union(creates, modifies)` across cohort members.
- If any intersection is non-empty, refuse the cohort and degrade to serial. Surface as `cohort overlap detected: tasks <A>+<B> share <path> — degrading to serial`.
- Do not file a DQ entry for the overlap — the planner's `[P]` marker was wrong, and the next retro should flag the planner miss; in-flight, serial dispatch is correct + safe.
- If the YAML block is missing on any cohort member (pre-V1 plan, or planner mistake), back-compat applies: skip the YAML check and trust the `[P]` marker (cohort can still merge-conflict at finalize, which is the failure mode this check exists to prevent forward-going).

**How to apply (verify side — see `.claude/commands/brehon-verify.md` Step 4a):**

- For each plan §16a story, parse its composing tasks' `creates:` arrays from §13.
- Phantom-presence check: `for f in creates: assert git show origin/phase-<phase>:<f> exits 0 + non-empty`. Two lines, mechanical, no descriptor-grammar.
- Brief-Scope outputs in §16a become a *secondary* layer (structural-pattern check inside the file content, parsed from "X contains Y" descriptors) — the YAML phantom-presence check is the primary signal for "did the task ship its files at all."
- Pre-V1 plans (no FILES YAML) keep the descriptor-grammar parser as the only signal. Back-compat unchanged.

**Refusals:**

- Never edit a §13 task body to "fix" YAML drift retroactively in a shipped plan — file a DQ pending entry asking the planner to revise instead. Plans are advisor-read-only post-approval.
- Never auto-degrade a cohort to serial silently — always surface the overlap in polling output.
- Never skip the YAML overlap check on cohort dispatch when the YAML is present — trust the YAML over the `[P]` marker when they disagree (the YAML is mechanical; the marker is planner judgment).

**Symptom to recognise in retrospect:** a phase whose retro flags "Task 4 [P] and Task 5 [P] both wrote to `crates/api/api/src/governance/log_entry/mod.rs` — finalize merge conflict" is the failure mode this rule prevents. If post-V1 plans hit this, the planner's YAML-vs-IMPLEMENT consistency check failed; if pre-V1 plans hit it, the back-compat path is operating as designed (no protection without the YAML).

**Generalises to:** any plan-driven workflow with multi-task cohorts. The mfs decomposer's per-task file arrays demonstrate the same primitive at a higher granularity (whole task-list JSON). The Brehon-specific simplification (YAML-in-prose, no sibling artifact) trades a small amount of parser complexity for zero new file-format surface — the plan stays the source of truth.

**Related:**
- `feedback_parallel_cohort_dispatch.md` — the `[P]` marker rule the YAML reinforces
- `feedback_parallel_agents_one_worktree_per_agent.md` — the per-worktree isolation invariant the cohort-overlap check protects
- `feedback_brehon_verify_pre_merge.md` — the verify-pass gate that consumes `creates:` for phantom-presence
- `feedback_schema_changing_spec_retrofit_question.md` — why this additive ships forward-only (pre-existing plans don't get YAML; advisor + verify back-compat to prose)
- `feedback_read_canonical_before_writing_spec.md` — the discipline that produced this lesson (read existing §13 examples + mfs decomposer before authoring the YAML schema)
