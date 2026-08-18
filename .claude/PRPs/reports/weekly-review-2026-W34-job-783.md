# Weekly Review — 2026-W34 (daemon-side, job-783)

Run: 2026-08-18 ~15:08 UTC, Junior task job-783, on homeserver/EliteDesk.
Skill: `.claude/skills/weekly-review/SKILL.md`. Branch:
`junior/review-read-and-follow-claude-skills-weekly-review-skill-md-783`.
Daemon trunk HEAD: `0af510669`. **Last weekly run was W32 (job-781); W33 was
not run.** Since W32 (`a490664cd`) trunk advanced by meta-work only — two skill
fixes: `0e026a0a3 fix(weekly-review): step 6 commits but never pushes` and
`0af510669 fix(retro): a zero-result 5a search is not evidence of a first
occurrence`. No implementation/retro activity, so the eval + harvest windows are
empty (5th straight quiet cadence: W29–W32, now W34).

## Headline — quiet week; the PMD prune-revert vector is confirmed a 6th time

1. **Store re-inflated to 1012 since W32.** Pre-prune this week was **1012** —
   identical byte size (9592832) *and* identical type breakdown to W32's pre-prune
   (qa-result 550, pattern 246, summary 193, issue-note 12, decision 8, deploy-note
   2, bug 1). W32 pruned to 460 and checkpointed; the store reverted to 1012 during
   the fortnight. **New corroborating evidence:** this week's summary memory got
   **ID 1134 — the same id W31 and W32 summaries got** — meaning the store keeps
   reverting to the same pre-W31 snapshot (max id 1133). This is a restore/replace
   job overwriting the pruned state, not lost writes. Still unidentified (needs
   `sudo`; blocked for Junior workers).
2. **RESOLVED-this-run — prune durable + fully embedded.** Pruned to 457 + WAL
   `TRUNCATE` checkpoint (durable now); embedding coverage 0 unembedded / 457
   vectors pre-summary. Durable *as of now*, at the same fortnight-over-fortnight
   revert risk as #1.
3. **STILL OPEN — 3 merged `phase-m3-core-*` branches** (unchanged from W31/W32;
   see findings). Duplicate-retro storm also unchanged.

## Steps executed (daemon-runnable)

### 0. Store-identity assertion (W30–W32 precedent) ✓
MCP `memory_review` = **1012**; on-disk `/srv/brehon-fork/.project-memory/memory.db`
= **1012**; newest snapshot `hourly/brehon-fork-20260818T1500.db` = **1012** with
identical type breakdown and identical byte size (9592832). All agree → single
store, no split. MCP is stdio/path-based (`.mcp.json` sets
`PROJECT_MEMORY_DB=/srv/brehon-fork/.project-memory/memory.db`); no HTTP daemon
serves the fork on the daemon (`project-memory-http.service` = `failed`/`not-found`,
a stale phantom unit — irrelevant to the stdio path). Prune safe to trust this run.

### 1. Memory health + prune ✓ (durable, checkpointed)
- Pre-prune **1012**. Prunable: **555 expired, 0 superseded** (550 qa-result + 2
  deploy-note + 2 issue-note + 1 summary). Confirmed by both MCP `memory_review`
  and a direct read-only SQL count.
- `memory_prune` dry-run reported "Would prune 555"; real `memory_prune`
  (not dry-run) → **pruned 555** → **457**.
- Prune landed in WAL; ran `PRAGMA wal_checkpoint(TRUNCATE)` → main-file mtime
  advanced to **15:06**, `memory.db-wal` truncated to **0 bytes**, durable count
  **457** (pattern 246, summary 192, issue-note 10, decision 8, bug 1; qa-result 0
  — all 550 expired). Post-summary write + 2nd checkpoint → **458**, WAL 0.

### 1b. PMD embedding backfill — LAPTOP-ONLY per skill; read-only diagnostic here ✓
On-disk store: **0 unembedded rows** / **457 memory_vectors** (complete before the
summary write). `memory-backfill.timer` **active** (5-min cadence). The 1 unembedded
row after the summary write is that new summary (no write-time embedding per PMD
invariant #3); the timer is the enforcement point on the daemon and will embed it.
The skill's Windows `backfill.js` path is laptop-side and not run here — no gap.

### 1c. Lesson frontmatter sweep ✓ — ALL CLEAN
`scripts/brehon/lesson-frontmatter-lint.sh` over **234** lesson files → RC=0,
"sync would import cleanly." No BROKEN/NESTED files (finding stayed resolved since W31).

### 2. Promote candidates — none actionable
Post-`memory_review` surfaces only generic infra tags (brehon-fork 523, success 384,
lesson 311, advisor 248, feedback 238, role-signal 194, kind:utilisation 190,
impl-task 167…). No file/pattern cluster warrants `docs/memory/` promotion. Same as
W26–W32. (`docs/memory/` does not exist in this repo.)

### 2b. Lesson clustering — SKIPPED (skill: TH-only; skip on server repos).

### 2c. Retro-harvest sweep ✓ — 0 new proposals
Used `git log --since='7 days ago' -- .claude/PRPs/reports/` (the `-mtime -7` filter
is unusable in a fresh worktree — all files carry the checkout mtime, W30 fix). Zero
report/retro-touching commits in the window; the only two commits are the skill
fixes noted above. No sub-phase (`*-retro.md`) or session (`session-retro-*.md`)
retros written; newest dated retro on disk is **2026-06-27** (unchanged since W32).
Written to gitignored `.claude/harvest/2026-W34.md`. Surfacing only.

### 2d. MEMORY.md ACTIVE-line drift — LAPTOP-ONLY (skill path `C:/Users/barri/…` absent on daemon).

### 2e. Role-signal drain + role-health — LAPTOP-ONLY (needs laptop DB + `ssh homeserver` scp *from* the laptop; this task runs *on* homeserver).

### 3. Eval-metrics aggregation ✓ (computed from frozen pre-prune snapshot)
Computed against the read-only `hourly/brehon-fork-20260818T1500.db` snapshot (qa-result
rows already pruned from the live store):
- **0 evals in the trailing 7d, 0 in the trailing 30d.** Newest `qa-result` is
  **2026-06-11** (unchanged since W31/W32) — the eval-producing activity gap is now
  ~9.7 weeks. Activity gap, not retro-bypass.
- 550 qa-result total; **526 scored, avg 0.706, median 0.700** — squarely in the
  calibrated 0.60–0.75 band.
- Calibration: **23/526 (4.4%) scored >0.85**, 2 at 1.00. Flat vs W30/W31/W32 (all 4.4%).
- Outcomes: **371 success / 106 partial / 8 failure / 65 untagged** → 67% of all,
  76% of tagged.
- ROOT_CAUSE present 164/550 (30%). Top meaningful: `process` (29), `environment`
  (26) — the raw `n/a` ×17 is a parse artifact.
- **Duplicate-retro storm persists:** same clusters as W30/W31/W32 — `v1-SL-e idle
  polling` ×6, `idle continues` ×5, `m2-late-1-task-5 spawn-site` ×5. Root is the
  retro-check same-session-visibility bug.

### 4. Git hygiene ✓ — clean; 3 branches surfaced (not force-deleted)
- `git worktree prune`: nothing to prune. Only **2** worktrees on disk (canonical
  `governance-v0` + this job-783).
- **33** local branches. `phase-m3-core-{e2e-pilot,emergency-mute,entry-kinds}` are
  all verified **ancestors of `origin/governance-v0`** (content merged, safe) but
  `git branch -d` refuses them (diverged `origin/<branch>` upstream). **NOT
  force-deleted** — `no-destructive-defaults.md` forbids `-D`/`--force` without
  explicit user request. Surfaced for human (unchanged from W31/W32).

### 4b. Disk headroom ✓ — crossed 75% (non-Brehon growth)
`/` at **76%** (166G/232G, 55G free) — up from **67%** at W32, now over the 75%
threshold but well below the 90% surface-to-user line. Ran `df` locally (this task
runs *on* homeserver; the skill's `ssh homeserver "df"` self-SSH is a no-op). The
~21GB week-over-week growth is **non-Brehon**: `/srv/media-local` (162G) dominates
total usage; `/srv/web-archive-data` (14G) also grows. Brehon's footprint is stable
at **19G** (of which `target/` build dir 8.1G). Only **1** worktree dir → no
worktree buildup. systemd journal is 3.5G (vacuumable to 500M if pressure grows). No
auto-remediation: below 90%, and the growth is non-Brehon media data outside this
task's remit.

### 5. Summary ✓
Summary memory written to the durable on-disk store (**ID 1134** — same id as W31/W32,
the revert-vector fingerprint), checkpointed. Global `METRICS.md` does not exist in
this repo; W26–W32 set the report-only precedent — followed, not created unilaterally.

### 6. Commit + push ✓
This report + the gitignored-excluded harvest note, `Review:`-prefixed (prevents the
weekly-review cron cascade). Pushed to origin per the W32-era Step-6 fix
(`0e026a0a3`) — a review that only commits is invisible on a daemon-run repo.

## Findings for the human (priority order)

1. **PMD prune still not durable across the cadence — find the revert vector.**
   Confirmed a 6th time (W27, W29/W30, W31, W32, now W34). This run pruned to 457 +
   checkpointed + the summary write brings it to 458; expect it to revert to 1012
   unless the vector is closed. Reused summary **ID 1134** (3rd consecutive run)
   proves a snapshot restore/replace overwrites the pruned state to the same pre-W31
   state — not lost writes. Needs `sudo` to inspect a restore job / `(deleted)`-inode
   marker (a cron restore, a service that re-seeds from a snapshot), then reconcile.
   *(W29 #1/#2, W30 #1, W31 #1, W32 #1 — open.)*
2. **Fold the proven daemon-side fixes into `SKILL.md`.** Still absent from the skill
   after multiple runs: (a) **Step 0 store-identity assertion** (compare MCP count vs
   newest `/srv/backups/pmd/<repo>/hourly/*.db` before pruning); (b) **Step 2c**
   should specify `git log --since` not `-mtime -7` (mtime is meaningless in a fresh
   daemon worktree); (c) **Step 4b** should branch on `hostname` (the daemon *is*
   homeserver; self-SSH `df` is a no-op). *(W30 #2/#4, W31 #2, W32 #2 — unactioned.)*
3. **3 merged `phase-m3-core-*` branches need human `-D`.** All ancestors of
   `origin/governance-v0` (content safe) but `-d`-refused (diverged upstream
   tracking). Either `git branch -D phase-m3-core-{e2e-pilot,emergency-mute,entry-kinds}`
   after confirming, or `git branch --unset-upstream` each then `-d`. Left in place
   per no-destructive-defaults. *(W31 #3, W32 #3 — unactioned.)*
4. **Disk `/` crossed 75% (76%).** Non-Brehon (`/srv/media-local` 162G, web-archive
   growth), not worktree buildup. Still 55G free / below 90% — monitor. If pressure
   grows: `journalctl --vacuum-size=500M` reclaims ~3G. New this run. *(NEW.)*
5. **Retro calibration + duplicate-retro rate.** 4.4% >0.85 and recurring ≥3× title
   clusters inflate task-count denominators. Root is the retro-check same-session-
   visibility bug. *(W30 #5, W31 #4, W32 #4 — unactioned.)*
6. **Laptop-only steps 1b-backfill/2d/2e + TH-only 2b not run** (daemon context) —
   run laptop-side to complete the week, especially 2e's role-health report (its
   role-signal queue on homeserver only accumulates until worker worktrees reap).
   *(W31 #5, W32 #5.)*
7. **Weekly-review keeps dispatching against a daemon worktree with no fresh
   eval/retro material** (newest qa-result 2026-06-11; this fortnight's trunk activity
   is skill meta-fixes only). Metrics/harvest windows empty for the 5th run. Also
   note **W33 was skipped entirely** (last run W32/job-781). Consider running
   weekly-review laptop-side, or gating it on impl activity in the window.
   *(W30 #7, W31 #6, W32 #6 — unactioned.)*
