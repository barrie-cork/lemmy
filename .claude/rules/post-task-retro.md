# Mandatory Post-Task Retrospective

Before exiting ANY task, you MUST complete a post-task retrospective. This is not optional.

## How to run

Read and follow the instructions in `.claude/skills/post-task-retro/SKILL.md`. Do NOT use a slash command — read the file directly and execute the steps.

## Minimum required action

At minimum, you must call `memory_write_eval` with:
- A title starting with "Task retro: "
- A score (0.0-1.0) using the 3-signal rubric in the skill file
- A confidence estimate (0.0-1.0) — your gut-level belief the task succeeded, recorded BEFORE scoring
- Tags including the outcome (success/partial/failure) and repo name

**Required output lines** (in the eval memory content):
```
SCORE: <composite>
CONFIDENCE: <pre-scoring estimate>
Goal achieved: <yes/partial/no>
Tests: <pass/fail/none>
Clean execution: <yes/no>
```

On partial/failure, also include:
```
ROOT_CAUSE: <category> — <explanation>
```

On partial/failure, also write a lesson memory (see Step 4 in skill file).

## Enforcement (retro-check.sh Stop hook)

The `memory_write_eval` call MUST be the FINAL action before you exit. The Stop hook's behaviour depends on the current branch:

- **On a `junior/*` branch** (Junior worktrees): the hook requires a `Task retro:%` row whose `source_ref = $(git rev-parse --abbrev-ref HEAD)` created in the last **30 minutes**. You MUST set `source_ref` to your exact branch name — an empty or wrong `source_ref` will fail the check even with a fresh retro. You cannot coat-tail on another task's retro.
- **On any other branch**: any `Task retro:%` row in the last 15 minutes satisfies the hook.

The skill's step order is: commit → scoring → root cause → lesson → auto-promote → doc drift → blast radius → **memory_write_eval LAST**. Follow that order. Do not reorder it.

Bypass attempts (future `created_at`, direct SQL, hook modification) are tracked and surface in weekly-review as `session-bug` memories.

## Score calibration

3 signals: goal achieved (0.40), tests pass (0.30), clean execution (0.30). Typical scores: 0.45-0.70. Scores above 0.85 should be rare.

## Auto-promotion

After writing the eval, search PMD for similar issues. If 3+ exist, auto-promote the pattern to CLAUDE.md's `## Learned Patterns` section. See skill file for details.

## Skip ONLY if

- The task was cancelled externally (not by you)
- This is a weekly-review task (those have their own reporting)
- This is a scheduled audit task (security daily/weekly scans)

Do NOT skip for failures, trivial tasks, or "didn't change much".
