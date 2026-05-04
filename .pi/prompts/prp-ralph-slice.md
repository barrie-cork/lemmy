---
description: Start a bounded autonomous Ralph loop scoped to a named slice of a Phase 5b plan (fork-local)
argument-hint: <plan.md> <slice:A|B|C> [--max-iterations N]
---

# PRP Ralph Slice

**Input**: $ARGUMENTS

---

## Purpose

Standard `/prp-ralph` executes an entire plan until `<promise>COMPLETE</promise>`. For Phase 5b (and similar large phases), the plan spans 7 tasks (~900 LOC + enum migration + new CLI + 3-branch e2e) — too large for one autonomous session without token-budget exhaustion.

This command runs a **bounded slice** of the plan — a fixed subset of tasks — inside a ralph loop. When the slice's tasks are all committed and the slice's validation commands pass, the loop exits with `<promise>COMPLETE</promise>` **even though the broader plan still has remaining slices**. The next session picks up from there via a fresh `/prp-ralph-slice` invocation for the next letter.

The stop-hook mechanism (`.claude/hooks/prp-ralph-stop.sh`) is reused verbatim — this command writes a state file the existing hook understands, with a **slice scope** section the iterating agent reads to know which tasks are in-scope.

---

## When to use

- The plan is ≥6 tasks or ≥700 LOC or crosses a migration boundary
- A prior session handed off partial progress (state file exists with "slice" frontmatter)
- The user wants to checkpoint-and-rest between slices (the explicit use case that motivated this command)

**Do not use** for single-task plans or <300-LOC plans — plain `/prp-ralph` is cheaper.

---

## Phase 5b slice map (hard-coded)

When the plan basename matches `phase-5b-sponsor-liability-and-founder-bootstrap.plan.md`, the slices are:

| Slice | Tasks           | Rationale                                                                                                                    | Max iter default |
|-------|-----------------|------------------------------------------------------------------------------------------------------------------------------|------------------|
| A     | 0, 56           | Foundation: pre-phase audit, branch cut, decision-queue close; task 56 is the highest-reasoning task (enum+migration+helper) | 6                |
| B     | 57, 58          | Phase 4 mutations: admin_assign_jury reputation gating + create_report OQ-006 formula                                        | 4                |
| C     | 59, 60, 61      | New surfaces + phase close: founder CLI, 3-branch e2e, PR + report                                                           | 6                |

Slice A **must run first** (migration + helper are load-bearing). Slice B depends on the config-reading pattern established by A. Slice C depends on B's create_report formula for e2e test assertions.

For plans other than Phase 5b, the slice map must be authored in the plan file itself (look for a `## §Slices` section). **If the plan has no slice map and is not Phase 5b, STOP** — ask the user to amend the plan first.

---

## Phase 1: PARSE — validate input

### 1.1 Extract arguments

From `$ARGUMENTS`:
- **Plan path**: must end in `.plan.md`
- **Slice letter**: must be one of `A`, `B`, `C` (Phase 5b) or a letter defined in the plan's `## §Slices` section
- **Max iterations**: `--max-iterations N` (default from slice map above; never exceeds 10 per advisor-context-phase-5.md rule 6)

### 1.2 Validate

```bash
test -f "{plan_path}" && echo "EXISTS" || echo "NOT_FOUND"
```

If NOT_FOUND → stop with error:

```text
Plan not found at {plan_path}.
Create one with /prp-plan, or pass the correct path.
```

If slice letter not valid for plan → stop with error.

### 1.3 Identify slice scope

Read the slice map above (or the plan's `## §Slices` section for non-Phase-5b plans). Extract:
- `{tasks_in_slice}` — ordered list of task numbers
- `{max_iterations}` — clamped to ≤10

---

## Phase 2: GUARD — refuse if a loop is already running

```bash
test -f .claude/prp-ralph.state.md && echo "LOOP_ACTIVE" || echo "NO_LOOP"
```

If `LOOP_ACTIVE`:

```text
An active ralph loop is already running.
Cancel it first: /prp-ralph-cancel
Or let it complete.
```

STOP. Do not overwrite the state file.

---

## Phase 3: SETUP — write the slice-scoped state file

Create `.claude/prp-ralph.state.md` with this exact structure:

```markdown
---
iteration: 1
max_iterations: {max_iterations}
plan_path: "{plan_path}"
input_type: "plan"
slice: "{slice_letter}"
slice_tasks: "{tasks_csv}"
started_at: "{ISO timestamp}"
---

# PRP Ralph Slice — {plan_basename} — Slice {slice_letter}

## Slice scope (hard gate for completion)

This loop only implements these tasks:

{bulleted list of tasks with short descriptions from the plan}

**Out of scope for this slice** (other slices will handle these, do NOT work on them here):

{bulleted list of the other slices' tasks}

## Completion criteria (all must be true before `<promise>COMPLETE</promise>`)

1. Every task in the slice scope above has a commit on the current feature branch with message matching `feat\|docs\|test\|chore\|fix(.*): task N`.
2. `git diff --stat HEAD~{N} HEAD` shows the expected files changed for the slice (cross-reference the plan's §7 File tree).
3. Every DoD command listed in the plan's §9 for the in-scope tasks exits 0.
4. The Level 1 workspace validation (`cargo-check.bat --features full --workspace` and `cargo-clippy.bat --features full --workspace --no-deps -- -D warnings`) exits 0 on the slice's final commit.
5. If the slice mutates a Phase 4 file (check: A touches submit_jury_vote; B touches admin_assign_jury and create_report), the `report_to_modlog_golden_path` e2e test passes.
6. Fork-local lint guards still pass: `bash scripts/brehon/lint-no-membership-read.sh` and `bash scripts/brehon/lint-no-can-sponsor-read.sh`.
7. This state file's progress log has an entry for each iteration with captured exit codes (not task-notification summaries — per no-cargo-output-paste.md, read log tails only).

## Codebase Patterns

(Consolidate reusable patterns here across iterations. Read this section first each iteration.)

## Current Task

Execute tasks {tasks_csv} from `{plan_path}`. Do not start or implement any task outside this slice.

## Instructions (per iteration)

1. Read this state file (top to bottom).
2. Read the plan file's §11 for the tasks listed in the slice scope — re-read if iterating.
3. Check `git log --oneline` for which slice tasks are already committed.
4. For the first incomplete task in the slice:
   - Implement per plan §11 exactly
   - Capture every cargo invocation to `.claude/build-task{N}-<level>.log` per cargo-output-capture.md
   - Run the task's DoD commands; tail ≤20 lines per log per no-cargo-output-paste.md
   - Commit with the required message form
5. After each commit, re-run Level 1 + regression guards (`report_to_modlog_golden_path` if Phase 4 files touched, `governance::config::parity` always).
6. When every task in the slice scope is committed AND every completion-criterion above is true:
   - Append a progress-log entry summarising the slice
   - Output `<promise>COMPLETE</promise>`
7. Otherwise:
   - Append a progress-log entry describing what was done and what remains
   - End response normally; the stop hook feeds the prompt back for the next iteration

## What NOT to do (slice discipline)

- Do NOT touch tasks outside the slice scope. If a later-slice task looks trivial, leave it — cross-slice coupling is how token budgets die.
- Do NOT run the full plan's §9 DoD sweep. Only run the DoD commands for in-scope tasks + the Level 1 + regression guards listed in Completion Criterion 4.
- Do NOT write a `docs(report)` commit — that's slice C (task 61). Slices A and B end cleanly with their task commits on the feature branch; no phase-close report.
- Do NOT open a PR — that's slice C (task 61).
- Do NOT merge branches.

## Progress Log

(Append one entry per iteration. Format per §3.8 of /prp-ralph.)

---
```

---

## Phase 4: DISPLAY — startup message

```markdown
## PRP Ralph Slice Activated

**Plan**: {plan_path}
**Slice**: {slice_letter} ({tasks_csv})
**Iteration**: 1
**Max iterations**: {max_iterations}

The stop hook is now active. When you try to exit:
- If slice completion criteria not all true → same prompt fed back
- If all criteria true AND you output `<promise>COMPLETE</promise>` → loop exits cleanly

To monitor: `cat .claude/prp-ralph.state.md`
To cancel: `/prp-ralph-cancel`

---

CRITICAL — slice discipline:
- Work ONLY on tasks {tasks_csv}. Do not touch other slices' tasks even if they look easy.
- Validate with the plan's §9 DoD commands for in-scope tasks + Level 1 + regression guards.
- Only output `<promise>COMPLETE</promise>` when every completion criterion in the state file is true.
- Do NOT write the phase-close report or open the PR unless this slice is C.

---

Starting iteration 1...
```

---

## Phase 5: BEGIN EXECUTION — hand off to the ralph iteration

From here the stop hook + `prp-ralph-loop` skill take over. The hook reads `iteration`, `max_iterations`, and feeds the prompt back until either:

- The agent outputs `<promise>COMPLETE</promise>` (success), OR
- `iteration >= max_iterations` (budget exhausted — hook auto-removes state file, surfaces message)

The iterating agent reads the state file's "Slice scope" + "Completion criteria" sections every iteration to stay on-boundary.

---

## Success criteria

- **STATE_FILE_WRITTEN**: `.claude/prp-ralph.state.md` exists with slice frontmatter
- **SLICE_SCOPE_ENFORCED**: State file names the in-scope tasks and the out-of-scope tasks explicitly
- **MAX_ITERATIONS_CAPPED**: Value ≤10 per advisor-context-phase-5.md rule 6
- **NO_CROSS_SLICE_WORK**: Iterations do not implement tasks outside the slice
- **CLEAN_EXIT**: `<promise>COMPLETE</promise>` only when every completion criterion is true

---

## Handoff between slices

When slice A completes, the user runs:

```text
/prp-ralph-slice .claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md B
```

to continue with slice B in a fresh session. The slice command does NOT auto-chain — each slice is a deliberate human checkpoint.

**Slice C's final action IS the phase close** (task 61 writes the completion report and opens the PR). Only slice C produces a `docs(report)` commit and a `gh pr create` call.

---

## Notes

- This command intentionally reuses `.claude/hooks/prp-ralph-stop.sh` unchanged. The stop hook does not know about slices — it only knows about `iteration`, `max_iterations`, and the `<promise>COMPLETE</promise>` sentinel. Slice enforcement lives entirely in the state file's body and the iterating agent's discipline.
- For non-Phase-5b plans: authors must add a `## §Slices` section to their plan listing slice letters + tasks. Until such a plan exists, this command is Phase 5b-only.
- The advisor-context-phase-5.md rule 6 caps ralph budgets at `--max-iterations 10`. This command respects that ceiling; default per-slice budgets are lower (4–6) because slices are deliberately small.
