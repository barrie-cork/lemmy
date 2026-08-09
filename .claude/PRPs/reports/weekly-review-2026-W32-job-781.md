# Weekly Review — 2026-W32 (daemon-side, job-781)

Run: 2026-08-09 ~02:31 UTC (Sunday), Junior task job-781, on homeserver/EliteDesk.
Skill: `.claude/skills/weekly-review/SKILL.md`. Branch:
`junior/review-read-and-follow-claude-skills-weekly-review-skill-md-781`.
Daemon trunk HEAD: `a490664cd` (2026-08-08) — advanced since W31 (`dac3292d5`) with
meta-work only: the W31 review self-commit + merge, two Dependabot bumps
(cargo-all ×33, npm-all ×9), and `1b5f768f0 fix(retro): step 0 no longer sweeps a
concurrent session's work`. No implementation/retro activity, so the eval + harvest
windows are empty (same as W29–W31).

## Headline — another quiet week; the PMD prune-revert vector is confirmed a 5th time

1. **Store re-inflated 460 → 1012 since W31.** Pre-prune this week was **1012**
   (identical type breakdown to W31's pre-prune). W31 pruned to 460 and
   checkpointed; the store reverted to 1012 during the week. **New corroborating
   evidence:** this week's summary memory got **ID 1134** — the *same id W31's
   summary got* — meaning W31's write (ID 1134) was lost when the store reverted to
   a pre-W31 snapshot. The revert vector is a restore/replace job, not lost writes.
   Still unidentified (needs `sudo`; blocked for Junior workers).
2. **RESOLVED-this-run — prune durable + fully embedded.** Pruned to 459 + WAL
   `TRUNCATE` checkpoint (durable now); embedding coverage 0 unembedded / 459
   vectors. Durable *as of now*, at the same week-over-week revert risk as #1.
3. **STILL OPEN — 3 merged `phase-m3-core-*` branches + duplicate-retro storm**
   (unchanged from W31; see findings).

## Steps executed (daemon-runnable)

### 0. Store-identity assertion (W30/W31 precedent) ✓
MCP `memory_review` = **1012**; on-disk `/srv/brehon-fork/.project-memory/memory.db`
= **1012**; newest snapshot `hourly/brehon-fork-20260809T0200.db` = **1012** with
identical type breakdown (qa-result 550, pattern 246, summary 193, issue-note 12,
decision 8, deploy-note 2, bug 1). All agree → single inode, no store-split. MCP is
stdio/path-based (`.mcp.json` sets `PROJECT_MEMORY_DB=/srv/brehon-fork/.project-memory/memory.db`);
no HTTP daemon serves the fork (`project-memory-http.service` = `not-found`, a stale
phantom `failed` unit — irrelevant to the stdio path). Prune safe to trust this run.

### 1. Memory health + prune ✓ (durable, checkpointed)
- Pre-prune **1012**. Prunable: **553 expired, 0 superseded** (550 qa-result + 3
  expired summary/issue-note).
- `memory_prune` (not dry-run) → **pruned 553** → **459**.
- Prune landed in WAL; ran `PRAGMA wal_checkpoint(TRUNCATE)` → main-file mtime
  advanced to **02:34**, `memory.db-wal` truncated to **0 bytes**, durable count
  **459** (pattern 246, summary 193, issue-note 11, decision 8, bug 1; qa-result 0
  — all 550 expired). Post-summary write + 2nd checkpoint → **460**, WAL 0.

### 1b. PMD embedding backfill — LAPTOP-ONLY per skill; read-only diagnostic here ✓
On-disk store: **0 unembedded rows** / **459 memory_vectors** (complete).
`memory-backfill.timer` **active (waiting)**, next trigger 02:36 (5-min cadence).
Embeddings current for the durable store. The skill's Windows `backfill.js` path is
laptop-side and not run here; under the homeserver stdio topology the timer is the
enforcement point, so no gap.

### 1c. Lesson frontmatter sweep ✓ — ALL CLEAN
`scripts/brehon/lesson-frontmatter-lint.sh` over **234** lesson files → RC=0,
"sync would import cleanly." No BROKEN/NESTED files (finding stayed resolved since W31).

### 2. Promote candidates — none actionable
Post-prune `memory_review` surfaces only generic infra tags (brehon-fork, success,
lesson, advisor, feedback, role-signal, kind:utilisation, impl-task…). No file/pattern
cluster warrants `docs/memory/` promotion. Same as W26–W31. (`docs/memory/` does not
exist in this repo.)

### 2b. Lesson clustering — SKIPPED (skill: TH-only; skip on server repos).

### 2c. Retro-harvest sweep ✓ — 0 new proposals
Used `git log --since='7 days ago' -- .claude/PRPs/reports/` (the `-mtime -7` filter
is unusable in a fresh worktree — all files carry the checkout mtime, W30 fix). Only
report-touching commit in the window: the W31 review self-commit `1945d8b29`. No
sub-phase (`*-retro.md`) or session (`session-retro-*.md`) retros written; newest
dated retro on disk is **2026-06-27**. Repo activity was meta-work / dependency
maintenance only. Written to gitignored `.claude/harvest/2026-W32.md`. Surfacing only.

### 2d. MEMORY.md ACTIVE-line drift — LAPTOP-ONLY (skill path `C:/Users/barri/…` absent on daemon).

### 2e. Role-signal drain + role-health — LAPTOP-ONLY (needs laptop DB + `ssh homeserver` scp *from* the laptop; this task runs *on* homeserver).

### 3. Eval-metrics aggregation ✓ (pre-prune on-disk — frozen window)
Computed before the prune while qa-result rows were present:
- **0 evals in the trailing 7d, 0 in the trailing 30d.** Newest `qa-result` is
  **2026-06-11** (unchanged since W31) — the eval-producing activity gap is now
  ~8.5 weeks. Activity gap, not retro-bypass.
- 550 qa-result total; **526 scored, avg 0.706, median 0.700** — squarely in the
  calibrated 0.60–0.75 band.
- Calibration: **23/526 (4.4%) scored >0.85**, 2 at 1.00. Above the rubric's "rare"
  bar but mild and flat vs W30/W31 (both 4.4%).
- Outcomes: **406 success / 99 partial / 11 failure / 34 untagged** → 74% of all,
  79% of tagged.
- ROOT_CAUSE present 198/550 (36%). Top meaningful: `environment-issue` (25),
  `process` (21) — the raw top token `n/a` ×45 is a parse artifact.
- **Duplicate-retro storm persists:** same clusters as W30/W31 — `v1-SL-e idle
  polling` ×6, `idle continues` ×5, `m2-late-1-task-5 spawn-site` ×5. Root is the
  retro-check same-session-visibility bug.

### 4. Git hygiene ✓ — clean; 3 branches surfaced (not force-deleted)
- `git worktree prune`: nothing to prune. Only **2** worktrees on disk (canonical
  `governance-v0` + this job-781).
- **33** local branches. `phase-m3-core-{e2e-pilot,emergency-mute,entry-kinds}` are
  all verified **ancestors of `governance-v0`** (content safe) but `git branch -d`
  refuses them (diverged `origin/<branch>` upstream). **NOT force-deleted** —
  `no-destructive-defaults.md` forbids `-D`/`--force` without explicit user request.
  Surfaced for human (unchanged from W31).

### 4b. Disk headroom ✓ — healthy (rose 5%)
`/` at **67%** (147G/232G, 75G free) — up from **62%** at W31, still well below the
75% threshold. No separate `/var/log` or `/tmp` mounts (all on `/`). Ran `df` locally
(this task runs *on* homeserver; the skill's `ssh homeserver "df"` self-SSH is a no-op).

### 5. Summary ✓
Summary memory written to the durable on-disk store (**ID 1134**), checkpointed.
Global `METRICS.md` does not exist in this repo; W26–W31 set the report-only
precedent — followed, not created unilaterally.

### 6. Commit ✓
This report, `Review:`-prefixed (prevents the weekly-review cron cascade).

## Findings for the human (priority order)

1. **PMD prune still not durable week-over-week — find the revert vector.** Confirmed
   a 5th time (W27, W29/W30, W31, now W32). This run pruned to 459 + checkpointed +
   the summary write brings it to 460; expect it to revert to 1012 unless the vector
   is closed. New evidence: the reused summary ID 1134 proves a snapshot restore/replace
   overwrites the pruned state (not lost writes). Needs `sudo` to inspect a restore
   job / `(deleted)`-inode marker, then reconcile. *(W29 #1/#2, W30 #1, W31 #1 — open.)*
2. **Fold the proven fixes into `SKILL.md`.** Three fixes are now proven across
   multiple runs but still absent from the skill: (a) **Step 0 store-identity
   assertion** (compare MCP count vs newest `/srv/backups/pmd/<repo>/hourly/*.db`
   before pruning); (b) **Step 2c** should specify `git log --since` not `-mtime -7`
   (mtime is meaningless in a fresh daemon worktree); (c) **Step 4b** should branch
   on `hostname` (the daemon *is* homeserver; self-SSH `df` is a no-op). *(W30 #2/#4,
   W31 #2 — unactioned.)*
3. **3 merged `phase-m3-core-*` branches need human `-D`.** All ancestors of
   `governance-v0` (content safe) but `-d`-refused (diverged upstream tracking).
   Either `git branch -D phase-m3-core-{e2e-pilot,emergency-mute,entry-kinds}` after
   confirming, or `git branch --unset-upstream` each then `-d`. Left in place per
   no-destructive-defaults. *(W31 #3 — unactioned.)*
4. **Retro calibration + duplicate-retro rate.** 4.4% >0.85 and recurring ≥3× title
   clusters inflate task-count denominators. Root is the retro-check same-session-
   visibility bug. *(W30 #5, W31 #4 — unactioned.)*
5. **Laptop-only steps 1b-backfill/2d/2e + TH-only 2b not run** (daemon context) —
   run laptop-side to complete the week, especially 2e's role-health report (its
   role-signal queue on homeserver only accumulates until worker worktrees reap).
   *(W31 #5.)*
6. **Weekly-review keeps dispatching against a daemon worktree with no fresh
   eval/retro material** (newest qa-result 2026-06-11; this week's trunk activity is
   meta-work + Dependabot). Metrics/harvest windows stay empty for the 4th straight
   week. Consider running weekly-review laptop-side, or gating it on impl activity in
   the window. *(W30 #7, W31 #6 — unactioned.)*
