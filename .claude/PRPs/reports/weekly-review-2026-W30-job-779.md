# Weekly Review — 2026-W30 (daemon-side, job-779)

Run: 2026-07-26 ~02:35 UTC (Sunday), Junior task job-779, on homeserver/EliteDesk.
Skill: `.claude/skills/weekly-review/SKILL.md`. Branch:
`junior/review-read-and-follow-claude-skills-weekly-review-skill-md-779`.
Daemon trunk HEAD: `365594650` — **still frozen at 2026-07-19** (the W29/job-778
merge; no trunk commits merged in the intervening week). Repo idle.

## Headline — PMD prune is NOT durable week-over-week; W29's own prune never persisted

The store-split flagged W26→W29 is **confirmed recurring and now materially
data-affecting**. Two out-of-band facts establish it this week:

1. **The path store reverted to its full 1012-row pre-prune size.** W29
   (job-778) reported pruning the on-disk store `1012 → 462`. This week's
   `memory_review` (same stdio/path MCP wiring, `PROJECT_MEMORY_DB=/srv/brehon-fork/.project-memory/memory.db`)
   found **1012 rows again** (qa-result 550, expired 552).

2. **Every hourly/daily/weekly snapshot Jul 19→26 is 1012 rows.** Copied and
   counted `weekly/20260719` (1012), `daily/20260725` (1012), `daily/20260726`
   (1012), `hourly/20260726T0100` (1012), `hourly/20260726T0200` (1012). The
   store `pmd-snapshot.service` backs up **never dropped below 1012 all week** —
   so W29's prune-to-462 **never durably reached the snapshotted store**.

W29 corrected W27 for the identical "store-identity blindness," then fell into
it itself: it trusted the MCP's internal before/after arithmetic (`1012 → 462`)
as proof of durability. The out-of-band snapshot lineage falsifies that.

### What this run did to make its own prune durable
- Confirmed MCP wiring is **stdio, path-based** to `/srv/brehon-fork/.project-memory/memory.db`
  (both the worktree `.mcp.json` and repo-root `.mcp.json`). So `memory_prune`
  targeted the path store, not an HTTP orphan.
- DB is **WAL mode**. The prune landed in `memory.db-wal` (main-file mtime was
  still 01:52, pre-prune). Ran `PRAGMA wal_checkpoint(TRUNCATE)` from a second
  connection → main-file mtime advanced to 02:34, count durably **460**.
- The next `pmd-snapshot` (03:00) should therefore capture 460 — **if** no
  concurrent writer (the suspected HTTP-daemon orphan, or a restore vector)
  overwrites `memory.db`+`-wal` first. That is the open risk and cannot be
  closed from a non-sudo daemon task.

### Still unidentified (needs the human, needs sudo)
- **The revert/replacement vector.** Could not read `/proc/<MainPID>/fd` for
  `(deleted)` markers — `sudo` is blocked for Junior workers (PMD #998). Could
  not restart `project-memory-http.service`. W29's finding #2 (vector
  unidentified) and finding #1 (restart the HTTP service) **remain open** and
  are now the load-bearing fix: until the vector is found, every weekly prune is
  cosmetic — the store re-inflates to 1012 within the week.

## Steps executed (daemon-runnable)

### 1. Memory health + prune ✓ (durability-caveated above)
- Pre-prune 1012 (qa 550, expired 552, superseded 0) → **pruned 552** → 460.
- WAL-checkpointed to persist (see headline). Post: pattern 246, summary 193,
  issue-note 12, decision 8, deploy-note 2, bug 1, qa-result 0.

### 1b. PMD embedding backfill — LAPTOP-ONLY per skill; read-only diagnostic here ✓
On-disk store: **0 unembedded rows**. `memory-backfill.timer` live (5-min
cadence, last fired 02:31). Embeddings current for the durable store.

### 1c. Lesson frontmatter sweep ✓ — 2 files flagged, NOT auto-fixed
`scripts/brehon/lesson-frontmatter-lint.sh` over **232** lesson files (exit 2):
- **BROKEN:** `feedback_daemon_sync_before_dispatch.md` — `name:` empty/absent →
  silently skipped by PMD sync, invisible to `memory_search_hybrid`.
- **NESTED:** `feedback_bm_pr_daemon_finalize_merges_phase_into_trunk.md` —
  `type:` nested under `metadata:`; imports but diverges from corpus convention.
- **Unchanged for 4 consecutive weeks** (W26 → W27 → W29 → W30). Human authoring
  decision — surfaced, not auto-fixed, per skill.

### 2. Promote candidates — none actionable
`memory_review` returned only generic infra tags (brehon-fork 523, success 384,
lesson 311, advisor 248, feedback 238, role-signal 194…). No file/pattern
clusters warranting `docs/memory/` promotion. Same as W26/W27/W29.

### 2b. Lesson clustering — SKIPPED (skill: TH-only; skip on server repos).

### 2c. Retro-harvest sweep ✓ — 0 new proposals
`-mtime -7` unusable in a fresh worktree; used `git log --since='7 days ago'`
(W29's fix). Only 1 report-touching commit in 7d = the W29 review self-commit
(not a retro). No sub-phase/session retros written in the window. Written to
gitignored `.claude/harvest/2026-W30.md`. Surfacing only.

### 2d. MEMORY.md ACTIVE-line drift — LAPTOP-ONLY (skill path `C:/Users/barri/…` absent on daemon).

### 2e. Role-signal drain + role-health — LAPTOP-ONLY (needs laptop DB + `ssh homeserver` scp *from* homeserver).

### 3. Eval metrics aggregation ✓ (from pre-prune snapshot; frozen-window)
qa-result rows were pruned in Step 1, so metrics were computed over the
pre-prune `hourly/20260726T0200.db` snapshot (durable record):
- **0 evals in the trailing 7d, 0 in the trailing 30d.** Newest `qa-result`
  is **2026-06-11** — a genuine activity gap (repo idle ~6 weeks), not a
  retro-bypass.
- 550 qa-result total; **526 scored, avg 0.706, median 0.700** — squarely in the
  calibrated 0.60–0.75 band.
- Calibration: **23/526 (4.4%) scored >0.85**, 2 at 1.00. Above the rubric's
  "rare" bar but far milder than W29's 10% (W29 sampled 100; this is the full
  corpus).
- Outcomes: 371 success / 106 partial / 8 failure / 65 untagged → success
  67% of all, 76% of tagged.
- ROOT_CAUSE present in only 198/550 (36%). Top: `process` (29),
  `environment` (26), `missing` (7), `partial` (7).
- **Duplicate-retro storm persists:** 5 title clusters ≥3 rows — worst
  `v1-SL-e idle polling` ×6, `idle continues` ×5, `m2-late-1-task-5 spawn-site`
  ×5. Consistent with the known retro-check same-session-visibility bug.

### 4. Git hygiene ✓ — 39 merged branches deleted
`git worktree prune`: nothing. `git branch -d` (merged-only, safe) over 42
candidates merged into `governance-v0`:
- **Deleted 39** — all `ab-cell/*`, `ab-test/*`, `phase-v1-*`, `phase-m1-*`,
  `phase-m2-late-*`, three `phase-m3-core-*` (infra/recording/stage-mode),
  `phase-test*`, `chore/rls-infra-setup`, `phase-brehon-conformance-audit`,
  `smoke/*`, `t607-tip`, `tmp-governance`.
- **Skipped 3** — `phase-m3-core-{e2e-pilot,emergency-mute,entry-kinds}`:
  reported merged but `-d` refused (checked out in stale `.junior/worktrees`).
  Left in place; will clear once those worktrees are reaped.
- Unmerged branches (RT-r2/r5, SL-c/d, deps, redaction, ship-2/3, `*-local`,
  `temp-bm-push`, `tmp-bm-merge`) correctly left alone (no `-D`, per
  no-destructive-defaults). Daemon is a separate clone from the laptop's lane
  worktrees, so these deletions are purely daemon-local.

### 4b. Disk headroom ✓ — healthy
`/` at **61%** (134G/232G, 88G free) — steady vs W29, well below thresholds.
No separate `/var/log` or `/tmp` mounts. (Skill's `ssh homeserver "df"` fails
here — this task runs *on* homeserver; ran `df` locally, per W29.)

### 5. Summary ✓
Summary memory written to the durable on-disk store. Global `METRICS.md` does
not exist (only the conformance-audit's own METRICS.md); W26/W27/W29 set the
report-only precedent — followed, not created unilaterally.

## Findings for the human (priority order)

1. **PMD prune is not durable — W29's prune reverted; find the replacement/revert
   vector.** This is now confirmed twice (W27, W29 both left cosmetic prunes).
   The path store re-inflates to 1012 within the week. Needs sudo: read
   `/proc/$(systemctl show -p MainPID --value project-memory-http.service)/fd`
   for `(deleted)` markers, then `sudo systemctl restart project-memory-http.service`.
   Until then every weekly prune (including this one, checkpointed to 460) is at
   risk of reverting. *(W29 findings #1+#2, still open, now with stronger evidence.)*
2. **Adopt a Step-0 store-identity assertion in the skill.** Before pruning,
   compare the MCP row count against the newest `/srv/backups/pmd/<repo>/hourly/*.db`
   snapshot; if they diverge, the MCP is not talking to the snapshotted inode —
   halt the prune and surface. Would have caught W27, W29, and this week upfront.
   *(W29 finding #3, unactioned.)*
3. **Fix the 2 lesson frontmatter files** — unchanged 4 weeks (see 1c).
4. **Fold W29's fixes into `SKILL.md`:** step 2c should specify
   `git log --since` (not `-mtime -7`); step 4b should branch on `hostname`
   (the daemon *is* homeserver, self-SSH fails). *(W29 findings #4+#5, unactioned.)*
5. **Retro calibration + duplicate-retro rate** — 4.4% >0.85 and a 5.8% duplicate
   rate inflating task-count denominators. The retro-check same-session-visibility
   bug is the root of the duplicates.
6. **Laptop-only steps 1b/2d/2e + TH-only 2b not run** (daemon context) — run
   laptop-side to complete the week, especially 2e's role-health report.
7. **Weekly-review keeps dispatching against a frozen daemon worktree**
   (HEAD 2026-07-19; newest PMD activity 2026-06-11) — metrics/harvest have no
   fresh material. Consider running weekly-review laptop-side, or syncing the
   daemon worktree to trunk before dispatch. *(Repeated W27 #5 / W29 #9, unactioned.)*
