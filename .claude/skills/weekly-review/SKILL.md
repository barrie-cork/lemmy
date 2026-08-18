---
name: weekly-review
description: >
  Weekly health check — prune expired memories, aggregate metrics, clean branches, backup promotion sweep, role-signal drain + health report.
  DO use when: scheduled weekly run (Sunday 02:00 UTC), manual "review memories" request.
  Do NOT use for: end-of-task retro (use post-task-retro), daily cleanup, one-off memory writes.
---

# Weekly Review

Lightweight health check: prune, aggregate, clean, promote anything post-task-retro missed.

Primary pattern detection now happens in post-task-retro (auto-promotion on 3+ confirmations). This skill is a backup sweep and metrics aggregator.

> **Execution context (HTTP-topology split, 2026-05-31).** Most steps
> (prune, metrics, branch cleanup, lesson-frontmatter lint) can run as a
> Junior task on the daemon. **Two steps MUST run on the laptop:** Step 1b
> (PMD embedding backfill) and Step 2e (role-signal drain + health report)
> — both touch the laptop's canonical `.project-memory/memory.db` (what the
> HTTP server starts with, not network-accessible) and Step 2e also needs
> `ssh homeserver` to scp the queue files. Run both laptop-side
> (interactive or laptop cron); the rest may stay daemon-side. Details in
> each step's callout.

## Steps

### 1. Memory health + prune

1. Call `memory_review` — record counts, prunable entries, promotion candidates
2. Call `memory_prune` (NOT dry_run) — delete expired and superseded memories
3. Log what was pruned

### 1b. PMD embedding backfill (safety net for unembedded rows)

**Why:** the PMD MCP server has NO write-time embedding (verified by
source inspection — `dist/index.js` has zero embed/ollama refs).
Every `memory_write` / `memory_write_eval` (from `post-task-retro`,
the `lesson-pmd-sync.sh` hook, or any interactive write) inserts a
**text-only row**. `memory_search_hybrid` silently degrades to FTS5
for unembedded rows — the +62.7% semantic-recall advantage is absent
until the Ollama backfill embeds them. So weekly-review is the
embedding enforcement point. Per
`.claude/lessons/feedback_pmd_backfill_after_write.md`.

> **MUST run on the LAPTOP, not inside the daemon Junior task
> (corrected 2026-05-31).** Under the HTTP-daemon topology the live
> PMD store is the canonical laptop DB
> `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` —
> that is the exact `PROJECT_MEMORY_DB` the HTTP server starts with
> (see `MCPs/project-memory-mcp/scripts/start-pmd-http-server.ps1`
> line 33). `backfill.js` shares the server's `getDbPath()` resolution,
> so pointing it at that path embeds the *live* store. The OLD step ran
> backfill on the daemon against `/srv/brehon-fork/.project-memory/memory.db`
> — a **dead/stale store** the HTTP server never reads (Junior writes
> go over HTTP to the laptop, never to that file). The daemon run
> embedded the wrong DB and reported "MISSING after backfill: 0" =
> false confidence while the live store's rows stayed unembedded
> (the store-split: `feedback_pmd_retro_check_http_store_split.md`).
> SQLite is not network-accessible and the server exposes no backfill
> endpoint, so this step CANNOT be delegated to the daemon — it runs
> where the DB file and the Ollama-reachable network live: the laptop.

Run this **laptop-side** (interactive weekly-review, or a laptop cron),
NOT as part of the daemon Junior task. Non-fatal on failure (log +
continue; safety-net sweep, not a gate). Bash form:

```bash
DB="C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db"
BF="C:/Users/barri/Developer/MCPs/project-memory-mcp/dist/scripts/backfill.js"
OLLAMA="http://homeserver:11434"   # same endpoint the HTTP server uses (start script line 36)

MISS="$(python -c "import sqlite3;c=sqlite3.connect(r'$DB');print(c.execute('SELECT COUNT(*) FROM memories m WHERE NOT EXISTS (SELECT 1 FROM memory_vectors v WHERE v.memory_id=m.id)').fetchone()[0]);c.close()" 2>/dev/null || echo ERR)"
echo "PMD unembedded rows (live store): $MISS"
if [ -f "$BF" ] && [ "$MISS" != "0" ] && [ "$MISS" != "ERR" ]; then
  if curl -s -m5 "$OLLAMA/api/tags" >/dev/null 2>&1; then
    OLLAMA_URL="$OLLAMA" PROJECT_MEMORY_DB="$DB" PROJECT_ROOT=C:/Users/barri/Developer/brehon-fork \
      node "$BF" --verbose 2>&1 | tail -5
    python -c "import sqlite3;c=sqlite3.connect(r'$DB');print('MISSING after backfill:',c.execute('SELECT COUNT(*) FROM memories m WHERE NOT EXISTS (SELECT 1 FROM memory_vectors v WHERE v.memory_id=m.id)').fetchone()[0]);c.close()"
  else
    echo "WARN: Ollama unreachable at $OLLAMA — $MISS rows stay FTS5-only; log + continue (non-fatal). Tailscale down or homeserver Ollama stopped."
  fi
elif [ ! -f "$BF" ]; then
  echo "WARN: backfill.js not at $BF — cannot embed; surface in weekly-review report"
fi
```

Log the before/after missing-count in the weekly-review report. If
Ollama is down (Tailscale or homeserver Ollama stopped) or `backfill.js`
is missing, that is itself a finding to surface — semantic search is
degrading week-over-week until the next successful backfill.

### 1c. Lesson frontmatter sweep (silently-skipped-lesson backstop)

**Why:** PMD ingestion requires a leading `---\n…\n---\n` frontmatter
block with a non-empty `name:`. A lesson authored without it is SILENTLY
SKIPPED and is invisible to `memory_search_hybrid` recall — forever, with
no signal. On 2026-05-29, 8 lessons were found in this state, unindexed
for weeks. Under the HTTP topology (2026-05-31+), the live ingestion path
is the `lesson-pmd-sync.sh` PostToolUse hook (auto-`memory_write` to the
HTTP server) — it ALSO requires valid frontmatter, so a no-frontmatter
lesson is skipped by both the hook and the legacy `sync-lessons-to-pmd.sh`.
The author-time `lesson-frontmatter-reminder.sh` hook catches the common
case; this sweep is the backstop for files that slip in via direct git
operations (a `git mv`, a manual editor write, a merge) that fire no
PostToolUse hook. Per `feedback_lessons_need_frontmatter_for_pmd_sync.md`.

Runs on the daemon side. Non-fatal (log + continue; this is a
safety-net sweep, not a gate):

```bash
# Sweep every synced lesson file. Exit 2 if any would be skipped by the sync.
bash scripts/brehon/lesson-frontmatter-lint.sh
RC=$?
if [ "$RC" -eq 2 ]; then
  echo "WEEKLY-REVIEW FINDING: one or more lessons lack PMD-sync frontmatter (see BROKEN lines above)."
  echo "These are invisible to memory_search_hybrid until fixed. Surface in the weekly summary."
elif [ "$RC" -eq 0 ]; then
  echo "lesson frontmatter sweep: all clean"
fi
```

If the sweep reports BROKEN files, list them in the weekly-review
report under findings. **Do NOT auto-fix** — frontmatter content
(`name`/`description`) is a human authoring decision; surface the file
list so the maintainer adds the right metadata, then re-runs the sync.

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

### 2d. MEMORY.md ACTIVE-line drift check

**Why:** MEMORY.md "Active workflow state" lines stay after the phases they
describe have shipped, if the closing session didn't clean them. Stale ACTIVE
lines cause the next session to treat completed phases as in-flight, wasting
~45 min on no-op re-derivation. (2026-06-01 incident: 3 ACTIVE lines for
r3b/r3c/redaction-r1 still present after all three shipped; 2026-05-31:
role-customization workflow state drift.) Per
`.claude/lessons/feedback_bootstrap_handover_verified_at.md`.

Run laptop-side (requires `v1-roadmap.json` to be readable):

```bash
# Extract phase slugs from ACTIVE lines in MEMORY.md
MEMORY_INDEX="C:/Users/barri/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/MEMORY.md"
ROADMAP=".claude/PRPs/v1-roadmap.json"

# Pull every "ACTIVE:" line slug — heuristic: first word after "ACTIVE: [" up to "]"
python -c "
import re, json, sys
mem = open(r'$MEMORY_INDEX', encoding='utf-8').read()
roadmap = json.load(open(r'$ROADMAP', encoding='utf-8'))

# Flatten all sub_phases from all lanes
all_phases = {}
for lane in roadmap.get('lanes', {}).values():
    for slug, phase in lane.get('sub_phases', {}).items():
        all_phases[slug] = phase.get('status', 'unknown')

actives = re.findall(r'ACTIVE[^:]*:\s*\[([^\]]+)\]', mem)
stale = []
for a in actives:
    # Try to find matching slug by substring
    for slug, status in all_phases.items():
        if slug in a.lower().replace('-','_') or slug.replace('-','_') in a.lower().replace('-','_'):
            if status in ('done', 'cancelled', 'skipped'):
                stale.append((a[:60], slug, status))
if stale:
    print('STALE ACTIVE LINES (phase shipped but still listed as ACTIVE):')
    for line, slug, status in stale:
        print(f'  [{line}] → roadmap status={status}')
    sys.exit(2)
else:
    print('ACTIVE-line drift check: all clean')
"
```

If the script exits 2, list the stale entries in the weekly report and
manually remove them from MEMORY.md. **Do NOT auto-remove** — check that
there is no concurrent lane work for that entry first. This step is
advisory; do not fail the weekly-review run on exit 2.

### 2e. Role-signal drain + role-health report

**Why:** role-signal utilisation rows (written by the
`role-signal-utilisation.sh` Stop hook on EliteDesk Junior workers) land
in a JSONL queue that survives only until the worker worktree is reaped.
`/check-role-health` drains + reports, but it ran ad-hoc — so strip-candidate
signals only accumulated when someone remembered to run it. Folding the drain
+ health report into the weekly cadence guarantees the corpus grows on a
rhythm. Per role-customization-T4b (`workflow_state_role_customization.md`).

**MUST run laptop-side** — the drain (`drain-role-signal-queue.sh`) scp's queue
files from `homeserver` and ingests via the `write-role-signal.js` CLI into the
canonical laptop PMD `.project-memory/memory.db` (the store the HTTP daemon
serves). It needs `ssh homeserver` reachability + the laptop DB file; it cannot
run inside the daemon Junior task. Non-fatal (log + continue; safety-net sweep):

```bash
# Drain EliteDesk's role-signal queue into the canonical PMD (scp-based; idempotent).
bash scripts/brehon/drain-role-signal-queue.sh 2>&1 | tail -3 || \
  echo "WARN: role-signal drain failed (ssh homeserver unreachable?) — surface in weekly report; non-fatal"
```

Then run the role-health report by invoking `/check-role-health` (no arg = all
four roles) per `~/.claude/commands/check-role-health.md` — it re-drains
(idempotent), queries the PMD for `role-signal` rows, and emits the per-role
strip-candidate report + the Step-5b dispatch-vs-signal-rate WARN. Surface in
the weekly summary:

- Per-role `n_tasks` + signal volume (SUFFICIENT ≥5 / LOW <5).
- Any strip candidates (rules in allowlist never Read; MCPs loaded never invoked).
- The dispatch-vs-signal WARN if N_DISPATCHES > 0 and N_SIGNALS == 0 over 24h
  (instrumentation-defect signal — run the smoke harness if it fires).

**Do NOT auto-strip or auto-edit any `rules.allowlist` / `mcp.json`** — strip
proposals are a user-gated decision, not a weekly-review action. This step only
drains + reports.

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

### 4b. Homeserver disk headroom check

Run: `ssh homeserver "df -h /srv /var/log /tmp | tail -n +2"`

If any mount is ≥75% used:
- `/srv`: Junior worktree buildup → `ssh homeserver "ls -lt /srv/brehon-fork/.junior/worktrees/ | head -20"` to identify stale worktrees; reap via `git worktree remove --force` for done tasks.
- `/var/log`: systemd journal → `journalctl --disk-usage`; trim with `journalctl --vacuum-size=500M` if >500 MB.
- `/tmp`: cargo tmp artefacts → `du -sh /tmp/cargo-*` or similar; safe to remove if no cargo is running.

Surface to user if any mount ≥90% — do not auto-remediate at that level.

### 5. Write summary + append metrics

**Summary memory:**
```
memory_write(
  title: "Weekly review <YYYY>-W<NN>",
  memory_type: "summary",
  tags: "<repo-name>",          # the repo this review ran in, not the control plane
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

### 6. Commit and push

- Commit message MUST start with `Review:` — prevents cron cascade
- **Push it: `git push origin HEAD`.** A review that only commits is invisible: on a daemon-run repo the METRICS.md rows pile up on the server, never reach GitHub, and the next run inherits a diverged checkout. Step 6 said "Commit" alone across the whole fleet, which is how five checkouts drifted up to 5 commits ahead of origin.
- If the push is rejected because the branch moved, `git pull --rebase` then push. Never force.
- **If you find the checkout already ahead of origin, push it — that is the fix, not an observation.** A prior run recorded "origin/main behind local main again — W31 merge unpushed" in its own commit message and pushed nothing; the note cost more than the push would have.
- Output a brief summary of what was done

## Notes

- Post-task-retro handles per-task pattern detection and auto-promotion. This skill is the weekly sweep.
- METRICS.md is append-only — don't edit previous rows
- Only delete branches for tasks with terminal status (completed/merged/failed)
