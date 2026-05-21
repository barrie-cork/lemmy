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

### 1b. PMD embedding backfill (safety net for unembedded rows)

**Why:** the PMD MCP server has NO write-time embedding (verified by
source inspection — `dist/index.js` has zero embed/ollama refs).
Every `memory_write` / `memory_write_eval` from `post-task-retro`
(Junior-side) and every `sync-lessons-to-pmd.sh` run inserts
**text-only rows**. `memory_search_hybrid` silently degrades to FTS5
for unembedded rows — the +62.7% semantic-recall advantage is absent
until `backfill.js` embeds them. `post-task-retro` CANNOT run the
backfill inline (its `memory_write_eval` must be the absolute final
action before exit per its Stop-hook contract; a post-eval backfill
would violate that + risk the Junior watchdog). So weekly-review is
the enforcement point for the Junior-side DB. Per
`.claude/lessons/feedback_pmd_backfill_after_write.md`.

This runs as a Junior task on the EliteDesk daemon — use daemon-side
paths (the MCP install location differs from the laptop's; locate
`backfill.js` dynamically) and localhost Ollama. Non-fatal on
failure (log + continue; this is a safety-net sweep, not a gate):

```bash
DB="/srv/brehon-fork/.project-memory/memory.db"
# Locate backfill.js on the daemon (install path is not fixed):
BF="$(find /srv /home /usr/local/lib -maxdepth 6 -name backfill.js -path '*project-memory*' 2>/dev/null | head -1)"
MISS="$(python3 -c "import sqlite3;c=sqlite3.connect('$DB');print(c.execute('SELECT COUNT(*) FROM memories m WHERE NOT EXISTS (SELECT 1 FROM memory_vectors v WHERE v.memory_id=m.id)').fetchone()[0]);c.close()" 2>/dev/null || echo ERR)"
echo "PMD unembedded rows: $MISS"
if [ -n "$BF" ] && [ "$MISS" != "0" ] && [ "$MISS" != "ERR" ]; then
  if curl -s -m5 http://localhost:11434/api/tags >/dev/null 2>&1; then
    OLLAMA_URL=http://localhost:11434 PROJECT_MEMORY_DB="$DB" PROJECT_ROOT=/srv/brehon-fork \
      node "$BF" --verbose 2>&1 | tail -5
    # Verify:
    python3 -c "import sqlite3;c=sqlite3.connect('$DB');print('MISSING after backfill:',c.execute('SELECT COUNT(*) FROM memories m WHERE NOT EXISTS (SELECT 1 FROM memory_vectors v WHERE v.memory_id=m.id)').fetchone()[0]);c.close()"
  else
    echo "WARN: daemon Ollama unreachable — $MISS rows stay FTS5-only; log + continue (non-fatal)"
  fi
elif [ -z "$BF" ]; then
  echo "WARN: backfill.js not found on daemon — cannot embed; surface in weekly-review report"
fi
```

Log the before/after missing-count in the weekly-review report. If
`backfill.js` can't be located or Ollama is down, that is itself a
finding to surface (semantic search is degrading week-over-week).

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

### 2c. Retro-harvest sweep

**Why:** The harvest tier (`.claude/skills/retro-harvest/SKILL.md`) surfaces unchecked proposals from session retros and sub-phase retros. Before this sub-phase, the skill ran ad-hoc — leaving ~48 unchecked proposals accumulated across 52 retro files (per RLS-PMD review §4.6 evidence) and 7-day-before-loss anti-patterns. Folding it into weekly cadence guarantees the harvest tier runs at the same rhythm as backfill and sync.

1. Glob `.claude/PRPs/reports/*.md` filtered to mtime within last 7 days.
2. For each retro: Read the §"What to change" + §"Decisions to revisit" sections.
3. Extract proposals NOT yet promoted to `.claude/lessons/` OR `CLAUDE.md` (check by grepping lesson filenames + canonical pattern text against the proposal verbatim quote).
4. Write a single weekly artifact at `.claude/harvest/<iso-week>.md` with proposals enumerated, each as a `(retro-source: <path>, proposal-text: <verbatim quote>, ground-truth-evidence: <if any>)` triple.
5. **SURFACING, not auto-promoting** — manual review thereafter per the RLS-PMD review §4.6 contract. Promotion to `.claude/lessons/` or `CLAUDE.md` is a human decision, not an automated step.

**Output path:** `.claude/harvest/<iso-week>.md` is gitignored (per `.gitignore`; see Task 5). Runtime-journal semantics — summary lands in the weekly-review report; harvest files prune after N weeks (planner-time choice; see §19.1 pre-seed #2).

**Invoke-by-reference:** `.claude/skills/retro-harvest/SKILL.md` contains the full sweep and currency-triage logic (STALE/LIVE/SUPERSEDED). This Step 2c is the cadence call point — invoke the retro-harvest skill here for the complete sweep procedure.

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
