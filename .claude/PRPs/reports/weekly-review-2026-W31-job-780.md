# Weekly Review — 2026-W31 (daemon-side, job-780)

Run: 2026-08-02 ~02:33 UTC (Sunday), Junior task job-780, on homeserver/EliteDesk.
Skill: `.claude/skills/weekly-review/SKILL.md`. Branch:
`junior/review-read-and-follow-claude-skills-weekly-review-skill-md-780`.
Daemon trunk HEAD: `dac3292d5` (2026-07-31) — **no longer frozen** (W30 was stuck
at 2026-07-19); this week trunk advanced with meta-work commits (skill-sync
manifest, PMD-write retag homeserver←tanglewood-hive, `fix(rules)` infra-drift).
No implementation/retro activity, so eval + harvest windows remain empty.

## Headline — a quiet week: two long-standing findings RESOLVED, one still open

1. **RESOLVED — lesson frontmatter lint is clean.** All **234** lesson files
   now pass `lesson-frontmatter-lint.sh` (RC=0). The 2 files W30 flagged as
   BROKEN/NESTED and *unchanged for 4 consecutive weeks*
   (`feedback_daemon_sync_before_dispatch.md`,
   `feedback_bm_pr_daemon_finalize_merges_phase_into_trunk.md`) are fixed.
2. **RESOLVED (this week) — no store-split.** The Step-0 store-identity
   assertion (W30 finding #2, adopted here) **passed**: MCP path-store,
   live on-disk `memory.db`, and the entire snapshot lineage all read
   **1012 rows** pre-prune — a single inode, no `(deleted)`-inode HTTP orphan
   this week. (Wiring is stdio path-based to
   `/srv/brehon-fork/.project-memory/memory.db`; no HTTP daemon serving a fork.)
3. **STILL OPEN — the prune revert vector.** 1012 pre-prune means W30's
   checkpointed prune-to-460 **reverted to 1012** during the week. The store
   re-inflated by exactly the 552 expired rows W30 removed. The
   replacement/revert vector remains unidentified (needs `sudo` — blocked for
   Junior workers, PMD #998). This run's prune (below) is again durable *as of
   now* but at the same week-over-week revert risk.

## Steps executed (daemon-runnable)

### 0. Store-identity assertion (NEW — W30 finding #2 adopted) ✓
On-disk `memory.db` = 1012; newest snapshot `brehon-fork-20260802T0200.db` = 1012;
hourly/daily lineage 2026-07-31→08-02 all = 1012; `memory_review` (MCP) = 1012.
All agree → MCP is talking to the snapshotted inode. Prune is safe to trust
*this run* (durability across the week is the separate open finding #3).

### 1. Memory health + prune ✓ (durable, checkpointed)
- Pre-prune 1012 (qa-result 550, pattern 246, summary 193, issue-note 12,
  decision 8, deploy-note 2, bug 1). Prunable: **552 expired, 0 superseded.**
- `memory_prune` (not dry-run) → **pruned 552** → 460.
- Prune landed in WAL (main-file mtime still 01:54). Ran
  `PRAGMA wal_checkpoint(TRUNCATE)` from a 2nd connection → main-file mtime
  advanced to **02:33**, `memory.db-wal` truncated to **0 bytes**, durable count
  **460** (pattern 246, summary 193, issue-note 12, decision 8, bug 1;
  qa-result 0 — all 550 were expired).

### 1b. PMD embedding backfill — LAPTOP-ONLY per skill; read-only diagnostic here ✓
On-disk store: **0 unembedded rows** (`memory_vectors` complete).
`memory-backfill.timer` **active** (5-min cadence; last fired 02:29, next 02:34).
Embeddings current for the durable store. (Skill's Windows `backfill.js` path is
laptop-side; not run here. Under current homeserver topology the timer is the
enforcement point, so no gap.)

### 1c. Lesson frontmatter sweep ✓ — ALL CLEAN (finding resolved)
`scripts/brehon/lesson-frontmatter-lint.sh` over **234** lesson files → RC=0,
"sync would import cleanly." The W30 backlog (2 files, 4 weeks) is cleared.

### 2. Promote candidates — none actionable
`memory_review` surfaced only generic infra tags (brehon-fork 523, success 384,
lesson 311, advisor 248, feedback 238, role-signal 194, kind:utilisation 190,
impl-task 167…). No file/pattern cluster warrants `docs/memory/` promotion.
Same as W26/W27/W29/W30. (`docs/memory/` does not exist in this repo.)

### 2b. Lesson clustering — SKIPPED (skill: TH-only; skip on server repos).

### 2c. Retro-harvest sweep ✓ — 0 new proposals
Used `git log --since='7 days ago'` (the `-mtime -7` filter is unusable in a
fresh worktree — all files carry the checkout mtime; W30 fix). Only report-
touching commits in the window: the W30 review self-commit `b82f12909` + its
merge `ea40b1836`. No sub-phase (`*-retro.md`) or session (`session-retro-*.md`)
retros written. Repo activity was meta-work only. Written to gitignored
`.claude/harvest/2026-W31.md`. Surfacing only.

### 2d. MEMORY.md ACTIVE-line drift — LAPTOP-ONLY (skill path `C:/Users/barri/…` absent on daemon).

### 2e. Role-signal drain + role-health — LAPTOP-ONLY (needs laptop DB + `ssh homeserver` scp *from* the laptop; this task runs *on* homeserver).

### 3. Eval-metrics aggregation ✓ (from pre-prune snapshot — frozen window)
qa-result rows were expired-pruned in Step 1, so metrics computed over the
pre-prune `hourly/20260802T0200.db` snapshot (durable record):
- **0 evals in the trailing 7d, 0 in the trailing 30d.** Newest `qa-result` is
  **2026-06-11** — an unchanged activity gap (repo idle for eval-producing work
  ~7.5 weeks), not a retro-bypass.
- 550 qa-result total; **526 scored, avg 0.706, median 0.700** — squarely in the
  calibrated 0.60–0.75 band.
- Calibration: **23/526 (4.4%) scored >0.85**, 2 at 1.00. Above the rubric's
  "rare" bar but mild (identical to W30's 4.4%).
- Outcomes: 387 success / 92 partial / 6 failure / 65 untagged → **70% of all,
  80% of tagged**.
- ROOT_CAUSE present in 198/550 (36%). Top meaningful: `process`, `environment`
  (the raw top token `n`×46 is an `n/a` parse artifact).
- **Duplicate-retro storm persists:** 5 title clusters ≥3 — worst
  `v1-SL-e idle polling` ×6, `idle continues` ×5, `m2-late-1-task-5 spawn-site`
  ×5. Same clusters as W30; the retro-check same-session-visibility bug is the
  root.

### 4. Git hygiene ✓ — clean; 3 branches surfaced (not force-deleted)
- `git worktree prune`: nothing to prune. Only 2 worktrees on disk (canonical
  `governance-v0` + this job-780) — W30's stale worktrees are reaped.
- 33 local branches. Merged-into-`governance-v0` candidates: **3** —
  `phase-m3-core-{e2e-pilot,emergency-mute,entry-kinds}`. All 3 are verified
  **ancestors of `governance-v0`** (work is safe in trunk), BUT `git branch -d`
  **refuses** them: each has a configured upstream `origin/<branch>` that has
  diverged, so git's safety check sees them as "not fully merged to upstream."
  **NOT force-deleted** — `git branch -D` is a force flag, and
  `no-destructive-defaults.md` forbids `--force`/`-D` without explicit user
  request. **Surfaced for human** (see findings).
- Unmerged branches (RT-r2/r5, SL-c-1/c-2/d, deps-r1/r2, federation-inbound-c/e,
  quality-r2, redaction-r1, ship-2/3, m2-rooms-a, ab-cell/ab-test cells,
  `jr-*-local`, `temp-bm-push`, `tmp-bm-merge`) correctly left alone.

### 4b. Disk headroom ✓ — healthy
`/` at **62%** (136G/232G, 85G free) — steady vs W30 (61%), well below the 75%
threshold. No separate `/var/log` or `/tmp` mounts. (Ran `df` locally — this task
runs *on* homeserver; skill's `ssh homeserver "df"` self-SSH is a no-op here, per
W30.) `.project-memory/` holds a single `memory.db.good-20260629T113058Z.bak`
(8.7 MB) — benign.

### 5. Summary ✓
Summary memory written to the durable on-disk store (ID 1134), checkpointed.
Global `METRICS.md` does not exist (only the conformance-audit's own); W26–W30
set the report-only precedent — followed, not created unilaterally.

### 6. Commit ✓
This report, `Review:`-prefixed (prevents the weekly-review cron cascade).

## Findings for the human (priority order)

1. **PMD prune still not durable week-over-week — find the revert vector.**
   Confirmed a 3rd time (W27, W29/W30, now W31): the store re-inflates to 1012
   within the week. This run pruned to 460 + checkpointed, but expect it to
   revert unless the vector is closed. Needs sudo:
   `sudo cat /proc/$(systemctl show -p MainPID --value project-memory-http.service)/fd/*`
   for `(deleted)` markers (if the HTTP daemon is even running — this week the
   MCP was pure stdio/path, so the vector may be a restore/replace job, not a
   live HTTP orphan), then restart/reconcile. *(W29 #1/#2, W30 #1 — still open.)*
2. **Fold the adopted fixes into `SKILL.md`.** Three fixes are now proven across
   multiple runs but not yet in the skill: (a) **Step 0 store-identity assertion**
   (this run adopted W30 finding #2 — compare MCP count vs newest
   `/srv/backups/pmd/<repo>/hourly/*.db` before pruning); (b) **Step 2c** should
   specify `git log --since` not `-mtime -7`; (c) **Step 4b** should branch on
   `hostname` (the daemon *is* homeserver; self-SSH `df` fails). *(W30 #2/#4,
   unactioned.)*
3. **3 merged `phase-m3-core-*` branches need human `-D`.** All 3 are ancestors
   of `governance-v0` (content safe) but `-d`-refused due to diverged upstream
   tracking refs. Either `git branch -D phase-m3-core-{e2e-pilot,emergency-mute,entry-kinds}`
   after confirming, or `git branch --unset-upstream` each then `-d`. Left in
   place this run per no-destructive-defaults.
4. **Retro calibration + duplicate-retro rate.** 4.4% >0.85 and recurring ≥3×
   title clusters inflate task-count denominators. Root cause is the retro-check
   same-session-visibility bug. *(W30 #5, unactioned.)*
5. **Laptop-only steps 1b/2d/2e + TH-only 2b not run** (daemon context) — run
   laptop-side to complete the week, especially 2e's role-health report (its
   role-signal queue on homeserver only accumulates until worker worktrees reap).
6. **Weekly-review dispatches against a daemon worktree with no fresh eval/retro
   material** (newest qa-result 2026-06-11; this week's trunk activity is
   meta-work). Metrics/harvest windows stay empty. Consider running weekly-review
   laptop-side, or only when there has been impl activity in the window.
   *(W30 #7, unactioned.)*
