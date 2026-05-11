---
name: cohort validation dependency check
description: `[P]` markers in plan §13 mark worktree-write disjointness, NOT validation-time independence. Use `requires:` in FILES YAML for cross-task data deps.
type: feedback
---

# Cohort validation dependency check

`[P]` markers in plan §13 declare that two tasks **write disjoint files**. They do NOT declare that each task's per-worker-branch validation can pass in isolation. When Task A's SQL/Rust/test code REFERENCES symbols, columns, or types created by Task B, Task A's workflow validation will fail on its worker branch (which contains only Task A's files) even though both tasks are correctly authored.

**Why:** Per `feedback_advisor_watchpoint_specificity.md` + retro evidence at `.claude/PRPs/reports/v1-RT-r1-halt-retro.md` (commit `ffa2876e3`). Cohort A (Tasks 1-4 migrations, all `[P]`) shipped clean SQL, but Task 3 (`backfill UPDATE reputation_event SET source_event_type = ...`) FAILED its per-worker `cargo-validate-migration.yml` (workflow `25636774556` → DQ #194) because Task 3's worker branch did NOT contain Task 1's `ALTER TABLE ... ADD COLUMN source_event_type` migration. The column reference was unresolvable on Task 3's isolated branch. Cohort B-serial Task 5 (Rust enum with `ExistingTypePath = "crate::schema::sql_types::ReputationEventSourceType"`) hit the same class: the schema.rs path that Task 6 creates didn't exist on Task 5's worker branch. ~4 hours advisor wallclock dominated by recovery vs ~30 min Junior work.

**How to apply:** When authoring plan §13 tasks, mark `[P]` for worktree-write disjointness AS USUAL, and ADDITIONALLY declare a `requires:` array in FILES YAML for each cross-task validation dependency:

```yaml
creates:
  - migrations/{ts}_backfill_reputation_event_source_type/up.sql
modifies: []
requires:
  - task: 1
    reason: Task 1's CREATE TYPE + ADD COLUMN source_event_type are referenced by this UPDATE
```

`requires:` and `[P]` are independent: a task may be `[P]` AND have `requires:` entries. Advisor cohort-dispatch (`.claude/rules/advisor-orchestrator.md` §4.1 step 4a) checks each `requires:` entry against `phase-<phase>` HEAD before queueing. If unsatisfied, the advisor either (a) defers the cohort until the required task lands, (b) refuses if the required task is also in this cohort (circular dependency → bundle them), or (c) bundles the requiring task with the required task into one Junior dispatch.

**Empty / missing `requires:` = back-compat:** pre-2026-05-11 plans assumed all `[P]` tasks were validation-independent. Old plans continue to work without re-authoring; new plans should populate `requires:` where applicable.

**Edge cases:**

- A task's `requires:` may include a task in the SAME cohort. This is a structural error — refuse the cohort, surface as `cohort refused: circular requires:`. Planner must re-order so the required task precedes (non-`[P]`), or bundle both into a single Junior task.
- A task's `requires:` may include a task from a prior cohort that hasn't yet been finalize-merged onto the phase branch (e.g. daemon push-skip). Defer the cohort with a polling re-check; do not block indefinitely.
- For SHIP-LEVEL validation (Phase 2 e2e on the phase branch tip, post-cohort), `requires:` is NOT needed — all prior tasks' commits are present by definition.

**Detection (retro):** an impl-task `validate-pending` DQ that returns `result: "fail"` whose log slice cites a symbol/column/type that another task in the same plan CREATES is evidence of an unsatisfied `requires:` that wasn't declared. Retro flags it. The planner's split-rejection (proceed-as-one) didn't catch the gap because per-task isolation is invisible during plan authorship.

**Companion lessons:**
- `feedback_explicit_file_arrays_on_tasks.md` — base FILES YAML schema.
- `feedback_parallel_cohort_dispatch.md` — `[P]` cohort dispatch mechanics.
- `feedback_ci_watcher_serial_per_task_pair.md` — sibling lesson on ci-watcher dispatch ordering.
