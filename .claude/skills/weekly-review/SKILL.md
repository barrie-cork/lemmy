---
name: weekly-review
description: >
  Weekly health check — prune expired memories, aggregate metrics, clean branches, backup promotion sweep.
  DO use when: scheduled weekly run (Sunday 02:00 UTC), manual "review memories" request.
  Do NOT use for: end-of-task retro (use post-task-retro), daily cleanup, one-off memory writes.
---

# Weekly Review

Lightweight health check: prune, aggregate, clean, promote anything post-task-retro missed. Runs as a Junior task Sunday 02:00 UTC.

Primary pattern detection now happens in post-task-retro (auto-promotion on 3+ confirmations). This skill is a backup sweep and metrics aggregator.

## Steps

### 1. Memory health + prune

1. Call `memory_review` — record counts, prunable entries, promotion candidates
2. Call `memory_prune` (NOT dry_run) — delete expired and superseded memories
3. Log what was pruned

### 2. Promote candidates (backup sweep)

For each promotion candidate from step 1 (tags/files in 3+ memories):

1. Check if already promoted to `docs/memory/PATTERNS.md`, `KNOWN_ISSUES.md`, or `DECISIONS.md`
2. If not yet promoted, call `memory_promote_to_file`
3. Also check if it should be in CLAUDE.md's `## Learned Patterns` section — if so, append it

This catches patterns that post-task-retro missed (e.g., cross-repo patterns visible only in aggregate).

### 2b. Lesson clustering (TH only)

Search for lesson memories written by `/reflect`:
```
memory_search_hybrid(query="recent lesson improvement", tags="lesson", limit=50)
```

Group results by the actionable advice they describe (semantic similarity, not exact title match). For each cluster:

- **3+ similar lessons** → confirmed pattern. Promote:
  1. Append to `docs/memory/PATTERNS.md`
  2. Write a `pattern` memory with `supersedes` pointing to the oldest lesson in the cluster
  3. Note in weekly summary: "Promoted lesson: <title>"
- **2 similar lessons** → emerging pattern. Flag in weekly summary: "Emerging: <title> (2 occurrences)"
- **1 occurrence** → no action, leave for future clustering

Skip this step on server repos (lessons are only written on TH via /reflect).

### 3. Aggregate eval metrics

Search for eval memories from the past 7 days:
```
memory_search(query="retro", memory_type="qa-result", limit=100)
```

Calculate:
- **Tasks evaluated** — count
- **Success rate** — count where outcome=success / total
- **Average score** — mean
- **Top root cause** — most common ROOT_CAUSE value
- **Retro coverage** — compare eval count to Junior DB task completions this week

If zero evals: note "No evals this week — check retro rule deployment."

### 4. Git hygiene

1. `git worktree prune`
2. Delete branches for completed/merged/failed Junior tasks
3. Clean orphaned `.junior/worktrees/job-*` directories
4. Flag review-status branches older than 7 days

### 5. Write summary + append metrics

**Summary memory:**
```
memory_write(
  title: "Weekly review <YYYY>-W<NN>",
  memory_type: "summary",
  tags: "tanglewood-hive",
  importance: 3,
  content: |
    Tasks: <count> | Success: <pct>% | Avg score: <num>
    Retro coverage: <M>/<N> (<pct>%)
    Top root cause: <category> (<count>)
    Pruned: <count> | Promoted: <count>
    Branches cleaned: <count> | Stale reviews: <count>
)
```

**Append to METRICS.md** (one row per repo):
```
| W<NN> | <repo> | <tasks> | <success%> | <avg> | <coverage> | <top_failure> | <worst_skill> | <root_cause> | <cal_gap> |
```

Use `—` for columns with no data.

### 6. Commit

- Commit message MUST start with `Review:` — prevents cron cascade
- Output a brief summary of what was done

## Notes

- Post-task-retro handles per-task pattern detection and auto-promotion. This skill is the weekly sweep.
- METRICS.md is append-only — don't edit previous rows
- Only delete branches for tasks with terminal status (completed/merged/failed)
