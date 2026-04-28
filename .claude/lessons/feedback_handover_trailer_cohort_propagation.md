---
name: Opt-in HANDOVER YAML trailer on [P]-cohort impl-task commits, advisor-aggregated into next-cohort §3a
description: Cohort N's `[P]` impl-task commits end with a HANDOVER YAML block (filesCreated, filesModified, keyDecisions, notes). The advisor parses these on cohort completion, aggregates them, and injects into cohort N+1's brief §3a "Handover from prior cohort" before queueing any task in N+1. Cross-cohort decision continuity without grep-discovery. Distinct from LESSON: trailer (cross-phase retroable) and from feedback_advisor_impl_communication_via_handover_files.md (cross-session narrative).
type: feedback
---

In `.claude/agents/impl-task.md` "Per-task commit shape", `[P]`-cohort members end the commit-message body with an opt-in `HANDOVER:` YAML trailer:

```
HANDOVER:
  filesCreated:
    - <path>
  filesModified:
    - <path>
  keyDecisions:
    - <one-line decision + brief reason citing plan §X.Y or MIRROR ref>
  notes: <free-text, ≤2 lines, gotchas the next cohort would benefit from>
```

The advisor reads these on cohort completion (per `.claude/rules/advisor-orchestrator.md` "Cohort handover aggregation"), aggregates across cohort peers into a single block matching the §3a schema in `impl-task-brief.template.md`, and injects into the **next** cohort's brief §3a "Handover from prior cohort" before queueing any task in N+1.

**Why:** Brehon already has the `LESSON:` trailer for cross-phase learning, but it's promotional — the advisor harvests at retro time and promotes durable lessons to `.claude/lessons/`. It does **not** propagate task-N's-output between cohort peers within the same sub-phase. Pre-V1, if Task 4 [P] decided "I'm using `parking_lot::RwLock` instead of `std::sync::RwLock`" and cohort N+1's Task 5 needed the same primitive, Task 5 would either grep-discover the choice (slow + fragile) or diverge silently (CR catches it eventually but the rebase cost is real). mfs's `prp-execute-with-agents.md` orchestrator showed a stronger pattern: per-task structured completion blocks the orchestrator aggregates and injects into successor task files. The Brehon adaptation is YAML-in-commit-body (no sibling artifact, no orchestrator script — the advisor does the parse + inject in its polling loop).

Three failure modes this prevents:

- **Cohort-N+1 decision drift.** Two cohort peers in N+1 each pick a different lock primitive because cohort N's choice wasn't visible. Both ship clean commits. CR triage flags inconsistency post-merge. With HANDOVER, cohort N's `keyDecisions` ride into N+1's brief §3a; the impl-task subagent reads §3a in step 6 of "Before you start" and either matches the decision or diverges with a stated reason (which goes into N+1's own HANDOVER trailer).
- **Re-discovery cost.** Cohort N+1 task starts; needs to know "did cohort N create the helper module yet?" — without HANDOVER, a grep + Read; with HANDOVER, the `filesCreated` array names it.
- **Silent overlap with cohort-N modifications.** Cohort N's `filesModified` should match the FILES YAML `modifies:` block (per `feedback_explicit_file_arrays_on_tasks.md`). Drift between the two is a discrepancy the advisor surfaces — file a DQ pending entry naming the drift before pushing.

**How to apply (impl-task subagent — see `.claude/agents/impl-task.md` "HANDOVER trailer"):**

- Write the trailer **only if** the task is `[P]`-marked in plan §13 and the cohort has at least one peer (cohort size ≥ 2). Single-task cohorts and non-`[P]` tasks skip the trailer.
- Place the trailer at the **end of the commit body**, before any `LESSON:` lines. Both can co-exist; they're independent (HANDOVER is opt-in cohort-internal-share; LESSON is opt-in cross-phase retroable).
- `filesCreated` + `filesModified` MUST match the plan §13 FILES YAML block for this task (`creates:` + `modifies:` per `feedback_explicit_file_arrays_on_tasks.md`). Drift between trailer + plan is a discrepancy — file a DQ pending entry naming the drift before pushing.
- `keyDecisions`: one-line each, with brief reason citing plan §X.Y or MIRROR ref. The bar is "future cohort peer would otherwise have to grep for this." Routine implementation choices fully described in plan §13 task body don't need a trailer entry. Same bar as the optional `LESSON:` trailer.
- `notes`: free-text, ≤2 lines, gotchas the next cohort would benefit from. Optional. Skip if the keyDecisions cover it.

**How to apply (advisor — see `.claude/rules/advisor-orchestrator.md` "Cohort handover aggregation"):**

- Trigger: all cohort N members reach `complete` (and under Shape G, all corresponding `validate-result` DQ entries with `result: "pass"`).
- For each cohort member's commit on `phase-<phase>`, parse the commit body for the `HANDOVER:` YAML trailer using `git log -1 --format=%B <sha>` and a YAML parser. Skip cohort members with no trailer (single-task non-`[P]` exception, or impl-task subagent skipped — note in polling output, do not catch-fire).
- Aggregate parsed trailers into one block matching the §3a schema:

  ```yaml
  prior_cohort_tasks:
    - task: <N>
      commit: <sha>
      filesCreated: [...]
      filesModified: [...]
      keyDecisions: [...]
      notes: <verbatim from trailer>
    - task: <N+1>
      ...
  ```

- Locate the next cohort's brief paths (one per cohort task — `.claude/PRPs/briefs/<phase>-impl-<M>.md`). For each, Edit §3a in place, replacing `(none — first cohort)` or `(none — prior task non-[P])` with the aggregated block. Commit subject: `chore(advisor): inject prior-cohort handover for <next-cohort-tasks>` (matches `^(chore|docs)\((advisor|decision-queue)\)` per attribution-integrity).
- Push the brief commits to `governance-v0` so Junior worktrees pick them up cleanly when the next-cohort tasks queue.
- Skip the aggregation step if the next cohort is empty (prior cohort was the last cohort before the retro task — retro task reads §3a as `(none — last cohort)`).

**How to apply (downstream impl-task — Step 6 of "Before you start"):**

- Read brief §3a "Handover from prior cohort" if the section is non-empty.
- `keyDecisions` from prior cohort tasks are **load-bearing context**. Diverging without a stated reason is a planner gap — file a DQ pending entry. Diverging with a stated reason ("Cohort N chose A; this task chose B because plan §10.5 mirror demands B") is fine and goes into this task's own HANDOVER trailer.
- `(none — first cohort)` or `(none — prior task non-[P])` is a no-op; proceed without §3a context.

**Refusals:**

- Never write a HANDOVER trailer on Task 0 (pre-flight harness — no impl content) or the retro task (no successor).
- Never include cargo output, diff blocks, or full file contents in `notes` — keep ≤2 lines per `.claude/rules/cargo-output-capture.md` discipline.
- Never silently override a prior-cohort `keyDecision` — divergence with reason is fine; divergence without a stated reason needs a DQ pending entry.
- Never aggregate a HANDOVER trailer that mentions a file outside the plan §13 FILES YAML for that task — the trailer is bounded by the plan-side declaration. Drift = DQ.

**When to skip:**

- **Single-task cohorts where the next task is non-`[P]`.** Strictly speaking the next task is the only successor and could read the trailer; the rule still aggregates because the bar is "the unit of handover is the trailer, not the cohort size" — a `[P]` task with one peer ahead of a non-`[P]` barrier still propagates its keyDecisions. The trailer is opt-in by `[P]`-membership, not by cohort size.
- **Task 0 (pre-flight harness audit).** Always non-`[P]`, no impl content, nothing to hand over.
- **Retro task.** No successor cohort.
- **Pre-V1 plans with no §3a in their impl-task briefs.** Back-compat: trailer optional, no aggregation. The advisor's polling loop handles the missing-§3a case as `(none — pre-V1 brief)`.

**Distinguishing this trailer from related conventions:**

- **`LESSON:` trailer** (per `feedback_junior_pmd_write_convention.md`) — opt-in cross-phase retroable. Advisor harvests at retro and promotes durable ones to `.claude/lessons/` + PMD. Different reader (retro author, weeks later) and different shape (one discrete lesson per line, no YAML).
- **`feedback_advisor_impl_communication_via_handover_files.md`** — cross-session narrative briefs at `.claude/PRPs/handovers/<role>-<date>-<topic>.md`. Different scope (cross-session, advisor↔impl, narrative format) and different durability (file on disk, 150-300 lines, survives session-close). HANDOVER trailer is intra-session, intra-sub-phase, structured-data-only.
- **DQ entries** (per `.claude/rules/decision-queue.md`) — blocking questions with structured options. HANDOVER is non-blocking, non-question; it's structured *output* propagation. They're complementary.

**Symptom to recognise in retrospect:** a sub-phase whose retro flags "Cohort 2's tasks made inconsistent choices Cohort 1 had already decided" is the failure mode this rule prevents. If post-V1 plans hit this with HANDOVER trailers present, either the trailer's `keyDecisions` were under-specified (the impl-task subagent's bar was too high) or the next cohort's impl-task didn't read §3a (missed step 6 of "Before you start"). Either way, the retro should flag which side missed and the next sub-phase iterates the contract.

**Generalises to:** any cohort-based execution model where cohort N+1 needs ambient awareness of cohort N's decisions. mfs's per-task JSON completion block + orchestrator-injection is the same primitive at a different granularity (whole task-list JSON vs per-commit YAML trailer). The Brehon-specific simplification (commit-body YAML, advisor as the orchestrator) trades automation for low surface area — no new file format, no orchestrator script.

**Related:**
- `feedback_explicit_file_arrays_on_tasks.md` — `filesCreated`/`filesModified` in HANDOVER must match plan §13 FILES YAML; drift = DQ
- `feedback_parallel_cohort_dispatch.md` — the cohort dispatch rule HANDOVER aggregation hooks into
- `feedback_advisor_impl_communication_via_handover_files.md` — the cross-session narrative convention this trailer is *not*
- `feedback_junior_pmd_write_convention.md` — the LESSON: trailer this co-exists with (independent, both opt-in)
- `feedback_schema_changing_spec_retrofit_question.md` — why this additive ships forward-only (pre-V1 briefs lack §3a; advisor back-compat handles missing-section as `(none — pre-V1 brief)`)
