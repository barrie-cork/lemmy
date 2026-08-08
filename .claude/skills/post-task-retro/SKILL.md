---
name: post-task-retro
description: >
  Run at the end of every Junior task. Writes structured eval memory, checks doc drift, auto-promotes confirmed patterns.
  DO use when: finishing any Junior task, task completed or failed.
  Do NOT use for: mid-task checkpoints, weekly reviews (use weekly-review), interactive sessions.
---

# Post-Task Retrospective

Run at the end of ANY Junior task. Writes an eval memory and auto-promotes confirmed patterns to CLAUDE.md.

## When to run

- At the end of every Junior task
- Skip ONLY if: task was cancelled externally, OR this is a weekly-review/scheduled audit task
- Do NOT skip for failures, trivial tasks, or "didn't change much"

## Enforcement contract (CRITICAL)

The `retro-check.sh` Stop hook enforces this skill. Its behaviour depends on the current git branch:

### Junior worktrees (`junior/*` branches) — branch-scoped, 30-minute window

The hook requires a `Task retro:%` row whose `source_ref` (OR `branch` field) equals the current branch AND was created in the last 30 minutes. Consequences:

1. **`source_ref` MUST be set to your exact branch name.** Use `$(git rev-parse --abbrev-ref HEAD)` to get it. A retro without this field — or with the wrong branch — will NOT satisfy the hook, even if it's fresh.
2. **You cannot coat-tail on another task's retro.** Each Junior task must write its own retro. This was a real bypass on my-food-system task #27 under hook v2 and is now closed.
3. **The `memory_write_eval` call MUST be the final action before you exit.** 30 minutes is forgiving, but not infinite — long auto-promotion or doc-drift work after the retro still eats into it.

### Other branches (main, master, interactive sessions) — time-window, 15-minute

Any `Task retro:%` row in the last 15 minutes satisfies the hook. Still write one, still write it last, but the branch requirement is relaxed so one-shot interactive work isn't blocked.

### Forgery and bypass detection

The hook rejects:
- `created_at` in the future (the `2099-12-31` trick — see PMD #927)
- `created_at` older than the window (stale retros from previous sessions)
- Direct hook modification (the sync script re-deploys on every run)
- Raw SQL INSERTs that skip `source_ref` (won't match branch-scoped check)

Do not attempt any of these — they are tracked and will show up in the weekly review as `session-bug` memories.

## Ordering rationale

The workflow below is ordered so all the *preparation* (commit, stale checks, scoring, root cause, lesson extraction, pattern promotion, doc drift, blast radius, memory routing) happens FIRST, and the `memory_write_eval` call is the very last step. Follow the order. If you must do extra work after step 7, write the retro a second time at the true end.

## Steps

### 0. Commit and push

**Run `git status --porcelain` first and read it before staging anything.** Any entry this task did not create — especially a non-blank *first* column (staged) — means another session is working in this tree. `git add -A` or a bare `git commit` would sweep their in-flight work into your commit.

**Solo (nothing in the tree but your work):**

1. Stage and commit all changes
2. Push: `git push origin HEAD`
3. Verify: `git status` shows a clean tree

**Concurrent (someone else's work is in the tree) — commit your paths only:**

1. `git commit -m "<msg>" -- <your paths>` — a **pathspec commit**. It records the working-tree content of exactly those paths, ignores whatever is staged in the index, and updates the index for your paths only. Their staged renames and edits survive untouched, and your paths come out clean. For a path that is still untracked, `git add <that path>` first (it stages only that path) — a pathspec commit cannot name a file git does not know.
2. Push, then re-run `git status --porcelain` and confirm **their** entries are exactly as you found them. The check here is *unchanged for them*, not "clean tree" — the tree is supposed to still be dirty.

**If a file you must commit is also being edited by the other session, stop and coordinate.** Every mechanism records the working-tree blob, so their half-finished edit rides along in your commit no matter which you pick. `CLAUDE.md` is the usual flashpoint — both sessions promote patterns into it (step 5).

> **Do not use `GIT_INDEX_FILE` for this.** It commits exactly what a pathspec commit would, and it leaves a trap: the main index still holds the **pre-commit** blobs for your paths, so `git status` shows them `MM` — a staged revert of your own work — and the other session's next plain `git commit` silently undoes it. If you have already committed that way, repair with `git reset HEAD -- <your paths>` before continuing.

If nothing to commit (failure with no code changes), skip to step 1.

### 0a. Verify no stale references (if task removed code)

If this task deleted or renamed a class, function, or file:

1. Identify the removed symbol names
2. Run `grep -r "<removed_name>" --include="*.py" .` (or `*.ts`, `*.dart` etc. for the project language)
3. Check specifically: `__init__.py` re-exports, test imports, URL config, serializer references
4. If any stale references remain → fix them before proceeding
5. Score impact: if stale references were found and fixed here, note it in clean execution (partial deduction)

Skip if: task only added new code, or modified existing code without removing any symbols.

### 1. Assess the outcome

| Outcome | Criteria |
|---------|----------|
| **success** | Task goal fully met, code committed |
| **partial** | Committed but with known gaps or skipped edge cases |
| **failure** | Task goal not met, errors unresolved |

**Confidence (0.0-1.0):** Record your gut-level belief the task succeeded BEFORE checking signals. Used for calibration.

### 2. Score the run (3 signals)

| Signal | Score | Notes |
|--------|-------|-------|
| **Goal achieved?** | yes=0.40, partial=0.20, no=0.00 | Did the task accomplish what was asked? |
| **Tests pass?** | yes=0.30, no tests=0.15, fail=0.00 | If the task IS "run tests", this signal = 0.00 |
| **Clean execution?** | yes=0.30, no=0.00 | No stuck loops (3+ retries), no scope creep, no blind chaining |

**Score = sum of signal values. Range: 0.00-1.00.**

Typical scores: 0.45-0.70 (partial + no tests + clean). A score above 0.85 should be rare.

### 3. Root cause (on partial/failure, or any score below 0.75)

Required when outcome is partial or failure, **or** when the score is below 0.75 (regardless of outcome classification). Tasks that succeed at a low ceiling score 0.55-0.70 and carry just as much diagnostic value as failures — without this requirement, failure-mode aggregation in weekly-review cannot work (it had ROOT_CAUSE on only ~10% of retros, so the "Top Failure" signal was empty).

Pick ONE — the deepest cause:

| Root cause | When |
|------------|------|
| `requirements-misread` | Task goal misunderstood |
| `wrong-files` | Edited wrong files or missed right ones |
| `bad-approach` | Strategy was flawed |
| `code-error` | Logic bug, syntax error, test failure |
| `missing-verification` | Skipped testing or validation |
| `environment-issue` | Infra, tooling, or external service problem |
| `no-op` | Task was trivially small or no-op; low score ceiling, not a defect |

Write a 1-2 sentence explanation of what specifically went wrong (or, for `no-op`, why the score ceiling was low).

### 4. Extract lesson (partial/failure only)

If outcome was partial or failure, write ONE lesson memory to feed the weekly-review clustering pipeline:

1. Search first: `memory_search_hybrid(query="<root cause keywords> lesson", tags="lesson,<repo-name>", limit=5)`
2. If no near-identical lesson exists, write:

```
memory_write(
  memory_type: "issue-note",
  title: "lesson: <short actionable description>",
  tags: "lesson,<repo-name>,<root-cause-category>",
  importance: 2,
  content: "TASK: <branch name>\nLESSON: <what to do differently next time>\nEVIDENCE: <what went wrong and why>"
)
```

**Skip if:**
- Outcome was success (no lesson to extract)
- Root cause is `environment-issue` (not actionable by the agent)
- A near-identical lesson already exists in PMD

These lessons are the input for weekly-review's clustering step — at 3+ similar lessons, a pattern gets auto-promoted to PATTERNS.md.

### 5. Auto-promote confirmed patterns

**Automation switch:** if CLAUDE.md's `## Automation Switches` marks **Lessons-file PR automation `OFF`**, still run the search and still write the `pattern` memory, but SKIP the CLAUDE.md `## Learned Patterns` append and the `learn:` commit — the pattern is preserved in PMD, no CLAUDE.md commit or PR is created. Steps 0 (task-code commit) and 7 (eval memory) are unaffected by the switch.

**5a. Repeat ROOT_CAUSE hard trigger — mandatory whenever Step 3 recorded a ROOT_CAUSE.**

```
memory_search_hybrid(query="<root cause category> <1-2 topical keywords>", tags="<repo>", memory_type="qa-result", limit=10)
```

If ANY prior retro (not this one) carries the same `ROOT_CAUSE:` category **and** is topically the same underlying issue (two unrelated retros sharing a category, e.g. `code-error`, don't count) — this is a repeat. Promote NOW; do not defer to a 3rd occurrence. This is a hard trigger, not a judgment call: skipping it because the task is already long is the exact failure mode PMD #389 documented — a lesson recurred across 3 retros and 2 real CI hits before anyone promoted it.

If no prior retro matches, this is the 1st occurrence — continue to 5b.

**5b. Generic bug-memory clustering.**

```
memory_search_hybrid(query="<root cause keywords>", tags="<repo>", memory_type="bug", limit=5)
```

If 3+ similar memories exist, this is a confirmed pattern.

**If 5a or 5b triggers, auto-promote:**

1. Read the current `CLAUDE.md`
2. Find or create a `## Learned Patterns` section at the bottom
3. Append the pattern as a bullet: `- **<title>** — <1-line actionable description>`
4. Commit: `learn: <pattern title> (auto-promoted from repeat ROOT_CAUSE | 3+ confirmations)`
5. Write a `pattern` memory: `memory_write(title="Pattern: <title>", memory_type="pattern", ...)`

**Guardrails:**
- Only append to `## Learned Patterns`, never modify other CLAUDE.md content
- Max 1 auto-promotion per task
- Check the existing Learned Patterns section — don't duplicate

### 6. Doc drift check

If the task changed anything user-facing **or infrastructure** (services, ports, dependencies, search backends, env vars):

1. Check these `CLAUDE.md` sections explicitly: **Key Commands**, **Docker Services**, **Directory Layout**, **Scripts & API**
2. Check `README.md` or other referenced docs
3. Fix in the same commit — keep edits concise

### 6a. Cross-repo infra drift

If your task changed Docker, ports, env vars, cron, MCP, or firewall config:

Write an `issue-note` memory:
```
memory_write(
  title: "Infra drift: <what changed>",
  memory_type: "issue-note",
  tags: "homeserver,infra-drift,infrastructure",   # infra-drift is REQUIRED — scripts/memory-audit.sh greps for it
  importance: 4,
  content: "Repo: <this repo>\nChange: <what>\nAffected docs: <paths>\nSuggested update: <brief>"
)
```

Skip if task made no infrastructure-affecting changes.

### 6b. Blast radius check (30 seconds)

If the task changed shared interfaces (API contracts, DB schema, env vars, Docker ports):
- List downstream consumers that may need updating
- Remember to add `DOWNSTREAM: <list>` when you write the eval memory in step 7.

Skip for internal-only changes.

### 6c. Memory routing check (30 seconds — laptop interactive sessions)

Cross-cutting infra knowledge written ONLY to a laptop's auto-memory
(`~/.claude/projects/.../memory/` — `MEMORY.md` + topic files) is invisible to
Junior workers (they run on the EliteDesk, a different machine, non-git, not
synced) AND to the other laptop. The shared, Junior-reachable store is the
centralized homeserver PMD on the EliteDesk (`/srv/project-memory/homeserver.db`).

Ask: did this session produce a cross-cutting infra fact (Junior patterns, MCP
config, n8n, networking, CI/workflow config, skills architecture, deploy topology)
that landed only in laptop auto-memory or only in CLAUDE.md prose?

- **Yes** → also `memory_write` it to the homeserver PMD as a `decision` or
  `pattern` (tags first = repo, then `infrastructure`/`configuration`/etc).
  Per CLAUDE.md: "Only write directly to the homeserver PMD for cross-cutting
  infra." Per-repo insight → queue a Junior task on that repo's daemon instead.
- **No**, or the insight is genuinely session-local (this-conversation context) → skip.

Skip entirely on `junior/*` branches — Junior workers have no laptop auto-memory,
so there is nothing to route. This check exists for the interactive laptop seat.

### 7. Write the eval memory (LAST — do this immediately before exit)

This is the **final action** of the skill and must be the final action of the task. See the Enforcement contract at the top of this file — on Junior worktrees the Stop hook matches the retro's `source_ref` against the current branch, so getting that field right is mandatory, not optional.

**Step 7a — confirm the branch name.** Run `git rev-parse --abbrev-ref HEAD` to get the exact branch. It should look like `junior/refactor-...-26`. Copy this verbatim into the `source_ref` field below.

**Step 7a (role-signal repos only) — also capture the task id.** If this repo
tracks role-signal rows (the Brehon four-role model — i.e. `/check-role-health`
exists in this repo's `.claude/skills/`), extract the Junior task id from the
trailing `-<id>` of the branch (e.g. `junior/refactor-...-26` → task id `26`).
For non-Junior branches (advisor sessions, hand-cut work), task id is `n/a`. Add
it to `tags` as `task_id:<id>` so `/check-role-health`'s outcome-section
retro-join can pair retros to role-signal rows by task id (both row classes
store `source_ref=<branch>` for the hook, but role-signal rows cross-key on
`task_id` — the tag is the schema bridge). **Skip this entirely if the repo has
no `/check-role-health`** — the tag has no consumer there and adds nothing.

**Step 7b — call `memory_write_eval`:**

```
title: "Task retro: <short task description>"
skill_or_tool: "<primary skill used, or 'general'>"
score: <0.0-1.0>
tags: "<outcome>,<repo-name>"          # role-signal repos: append ,task_id:<id>
source_ref: "<exact output of git rev-parse --abbrev-ref HEAD>"
content: |
  SCORE: <score>
  CONFIDENCE: <confidence>
  Goal achieved: <yes/partial/no>
  Tests: <pass/fail/none>
  Clean execution: <yes/no>
  Summary: <1-line what was done>
  ROOT_CAUSE: <category> — <explanation>  (partial/failure, or score below 0.75)
  DOWNSTREAM: <list>  (only if step 6b identified consumers)
```

**Why `source_ref` is mandatory:** the Stop hook on a `junior/*` branch runs
```sql
... WHERE (source_ref = '<current_branch>' OR branch = '<current_branch>')
    AND created_at >= datetime('now', '-30 minutes')
```
If `source_ref` is empty or wrong, this returns 0 rows and the hook blocks even though you wrote a retro. This was the root cause of the task #27 coat-tail bypass on my-food-system (PMD #927).

After this call returns, exit. Do NOT queue more work, run more checks, or re-open files.

## Important

- Be honest in scoring — inflated scores defeat the purpose
- The eval memory expires in 30 days (qa-result type) — intentional
- This adds ~30 seconds to task completion — worth it
