# Weekly Review — 2026-W35 (daemon-side, job-785)

Run: 2026-08-30 ~02:35 UTC, Junior task job-785, on homeserver/EliteDesk.
Skill: `.claude/skills/weekly-review/SKILL.md`. Branch:
`junior/review-read-and-follow-claude-skills-weekly-review-skill-md-785`.
Daemon trunk HEAD at start: `d930594c0`. Since job-784 (2026-08-23), trunk
advanced by exactly one commit: the merge of job-784's own report
(`d930594c0`). No implementation/retro activity in the window — 7th
straight quiet cadence (W29–W32, W34×2, W35).

## Headline — quiet week; PMD prune-revert vector 8th confirmation, but restore-drill.sh FALSIFIED as the cause

1. **Store re-inflated to 1012 again — byte-identical to every prior
   recurrence.** Pre-prune this run: **1012 rows**, on-disk file
   **9,592,832 bytes** (byte-identical to W34-783 and W34-784), max id
   **1133**, newest row timestamp **2026-06-26 10:46:25** — same frozen
   snapshot content as every prior recurrence (W27, W29, W30, W31, W32,
   W34-783, W34-784; this is the **8th confirmation**). Confirmed via the
   0200 UTC hourly snapshot (`brehon-fork-20260830T0200.db`) matching the
   live on-disk `.project-memory/memory.db` exactly before this run's prune.
2. **New this run — root-cause candidate investigated and FALSIFIED.**
   Found `restore-drill.timer`/`restore-drill.service` — "Weekly automated
   restore drill (rotates pg/pmd/volume/sysconfig)", scheduled **Sunday
   04:00 UTC** (i.e. ~1.5h *after* weekly-review's ~02:30 UTC slot) — a
   highly plausible suspect given the weekly cadence and the literal string
   "pmd" in its rotation. Read `/usr/local/bin/restore-drill.sh` directly
   (world-readable, no sudo needed): the `pmd` class (1) picks a **random
   repo from `REPOS=(agent-grey dog-shelter food-producer my-food-system
   midleton-market security web-archive)` — brehon-fork is NOT in this
   list**, (2) copies that repo's latest hourly snapshot into a throwaway
   `/srv/restore-test/` scratch dir, (3) runs `PRAGMA integrity_check` +
   row count as a smoke test, (4) `rm -rf`s the scratch dir. It never reads
   or writes brehon-fork's live path, and it's read-only into scratch even
   for the repo it does pick. **This mechanism is exculpated** — per the
   falsifiable-hypothesis discipline (`feedback_falsifiable_hypothesis_
   before_structural_fix.md`), do not re-raise it as a suspect without new
   evidence. Root cause is still unidentified; still needs `sudo`-level
   investigation (blocked for Junior workers — confirmed again this run,
   `sudo -n` refused with "Junior workers may not invoke sudo").
   Secondary check: `memory-backfill.timer`/`.service` (runs every 5 min,
   `backfill-all.sh` → `backfill.js`) only adds embeddings — additive,
   not a plausible revert vector; not investigated further.
   Host rebooted twice since job-784 (2026-08-25 23:03, 2026-08-28 00:35 —
   plain ext4-on-LVM root, no snapshot/overlay fs per `/etc/fstab`), so a
   filesystem-snapshot-on-boot theory is also weak — worth noting for
   whoever picks this up next, but not chased further this run (time-boxed).
3. **RESOLVED-this-run — prune durable + checkpointed again.** Pruned 561
   expired (0 superseded) → **451**. WAL `TRUNCATE` checkpoint immediately
   after prune (0 bytes) and again after the summary write (**452** final,
   WAL 0 bytes).
4. **Summary memory landed at ID 1134 again — 5th consecutive run** (W31,
   W32, W34-783, W34-784, now W35) to get this exact id despite each
   intervening run's own writes. Strongest evidence to date that whatever
   reverts the store resets it to the identical max-id-1133 snapshot every
   time, not merely "loses recent writes" independently per cycle.
5. **STILL OPEN — same 3 merged `phase-m3-core-*` branches**, unchanged
   since W31 (5 consecutive reviews now).

## Steps executed (daemon-runnable)

### 0. Store-identity assertion (W30–W35 precedent) ✓
MCP `memory_review` pre-prune = **1012**; on-disk `/srv/brehon-fork/
.project-memory/memory.db` = **1012**/9,592,832 bytes/max-id **1133**;
newest hourly snapshot `hourly/brehon-fork-20260830T0200.db` = same
(copied to `/tmp/snap0200.db` for read access — live snapshot files are
owned root read-only). All agree → single store, no split; prune safe to
trust. `.mcp.json` confirms **stdio transport** direct to
`/srv/brehon-fork/.project-memory/memory.db` (not the documented HTTP
topology per `pmd-invariants.md` — noted but not actioned, matches prior
runs' observation that this worktree's config predates/bypasses the HTTP
daemon cutover). A `project-memory-http@brehon-fork.service` unit IS
running on this host (contra job-784's claim of no HTTP unit — it exists
now, possibly added at the 2026-08-28 reboot) but this Junior worktree's
`.mcp.json` doesn't use it.

### 1. Memory health + prune ✓ (durable, checkpointed)
- Pre-prune **1012**. Prunable: **561 expired, 0 superseded**.
- `memory_prune` (not dry-run) → **pruned 561** → **451**.
- `PRAGMA wal_checkpoint(TRUNCATE)` → WAL 0 bytes, durable count **451**.
  Post-summary write + 2nd checkpoint → **452**, WAL 0.

### 1b. PMD embedding backfill — LAPTOP-ONLY per skill; not run here.

### 1c. Lesson frontmatter sweep ✓ — ALL CLEAN
`scripts/brehon/lesson-frontmatter-lint.sh` over **234** lesson files →
RC=0, "sync would import cleanly." Unchanged since W31.

### 2. Promote candidates — none actionable
`memory_review` surfaces only generic infra tags (brehon-fork 523,
success 384, lesson 311, advisor 248, feedback 238, role-signal 194,
kind:utilisation 190, impl-task 167, role:impl-task 137,
config_version:… 137). No file/pattern cluster warrants `docs/memory/`
promotion (directory does not exist in this repo). Same as W26–W34.

### 2b. Lesson clustering — SKIPPED (skill: TH-only; skip on server repos).

### 2c. Retro-harvest sweep ✓ — 0 new proposals
`git log --since='7 days ago' -- .claude/PRPs/reports/
.claude/decision-queue.json` → **0 commits in window** (job-784's own
report commit from 2026-08-23 falls just outside the 7-day window at
run time). Zero new `session-retro-*.md`/`*-retro.md` files. Newest
retro on disk (by git authorship date, not mtime — confirmed the
mtime pitfall again: `find -mtime -7` returned ~230 files, a clone-
artifact false positive) is still **2026-06-27**, unchanged since
W32/W34. Written to gitignored `.claude/harvest/2026-W35.md`.

### 2d. MEMORY.md ACTIVE-line drift — LAPTOP-ONLY (path absent on daemon).

### 2e. Role-signal drain + role-health — LAPTOP-ONLY.

### 3. Eval-metrics aggregation ✓ (computed from frozen pre-prune snapshot)
Computed against `/tmp/snap0200.db` (copy of the 0200 UTC hourly
snapshot; qa-result rows were already pruned from the live store by the
time this ran):
- **0 evals in the trailing 7d.** Newest `qa-result` is **2026-06-11**
  (unchanged since W31/W32/W34) — eval-producing activity gap now
  **~11.4 weeks**. Activity gap, not retro-bypass.
- 550 qa-result total; **527 scored, avg 0.706, median 0.700** — flat vs
  W34, squarely in the calibrated 0.60–0.75 band.
- Calibration: **23/527 (4.4%) scored >0.85** — flat vs W30–W34.
- Top non-artifact ROOT_CAUSE categories: `environment-issue` (25),
  `process` (21), `missing-verification` (7) — same leaders as prior
  weeks (regex parse is noisy; a `'n'` bucket of 41 is parsing artifact,
  not a real category).

### 4. Git hygiene ✓ — clean; 3 branches surfaced (not force-deleted)
- `git worktree prune`: nothing to prune. Only **2** worktrees on disk
  (canonical `governance-v0` + this job-785).
- **33** local branches (unchanged count). `phase-m3-core-{e2e-pilot,
  emergency-mute,entry-kinds}` re-verified as **ancestors of
  `origin/governance-v0`** (content merged, safe) but still `-d`-refused
  (diverged upstream tracking). **NOT force-deleted** per
  `no-destructive-defaults.md`. Surfaced for human, 5th consecutive week.

### 4b. Disk headroom ✓ — 77%, stable
`/` at **77%** (169G/232G, 52G free) — +1% vs W34's 76%, essentially
flat. Ran `df` locally (this task runs *on* homeserver). Brehon's
footprint stable at **19G** (`target/` 8.1G). Only 1 worktree dir on
disk → no worktree buildup. systemd journal 3.6G (vacuumable to 500M if
pressure grows). No auto-remediation: below the 90% surface-to-user line.

### 5. Summary ✓
Summary memory written to the durable on-disk store — **ID 1134**, the
**same id** W31, W32, W34-783, and W34-784 all got (5th consecutive
occurrence). Checkpointed (WAL 0 bytes). `METRICS.md` does not exist in
this repo; report-only precedent from W26 onward followed, not created
unilaterally.

### 6. Commit + push ✓
This report, `Review:`-prefixed (prevents weekly-review cron cascade).
Pushed to origin per the W32-era Step-6 fix.

## Findings for the human (priority order)

1. **PMD prune-revert vector — 8th confirmation, restore-drill.sh
   FALSIFIED as the cause this run.** The obvious suspect
   (`restore-drill.timer`, Sunday 04:00 UTC, "rotates pg/pmd/volume/
   sysconfig") was investigated and ruled out by reading
   `/usr/local/bin/restore-drill.sh` directly: its `pmd` class never
   includes brehon-fork in its repo rotation and only ever copies a
   snapshot into a throwaway scratch dir it deletes — it cannot be
   writing to `/srv/brehon-fork/.project-memory/memory.db`. This
   narrows the search space but the actual mechanism remains
   unidentified. Suggested next steps for whoever has `sudo`: (a) `grep
   -r "brehon-fork" /etc/cron.d/ /etc/systemd/system/*.service
   /usr/local/bin/*.sh` for any OTHER script that touches this specific
   path; (b) check for a stale bind-mount or overlay specifically on
   `/srv/brehon-fork/.project-memory/` (the rest of `/` is plain ext4,
   but a sub-path could differ); (c) check `auditd`/`inotifywait` logs
   if available for writes to `memory.db` between weekly runs; (d)
   check whether `project-memory-http@brehon-fork.service` (newly
   observed running this week, absent in job-784's check) has its own
   restore-on-start behavior reading a bundled/committed seed DB.
   *(W27, W29, W30, W31, W32, W34-783, W34-784 — open, now with one
   candidate eliminated.)*
2. **Fold the proven daemon-side fixes into `SKILL.md`.** Still absent
   after 7+ runs: (a) Step 0 store-identity assertion (compare MCP
   count vs newest snapshot, including max id); (b) Step 2c should
   specify `git log --since` not `-mtime -7`; (c) Step 4b should branch
   on `hostname` (self-SSH `df` is a no-op on the daemon). *(W30 #2/#4,
   W31 #2, W32 #2, W34-783 #2, W34-784 #2 — unactioned across 5+ runs
   now.)*
3. **3 merged `phase-m3-core-*` branches need human `-D`.** All
   ancestors of `origin/governance-v0` (content safe) but `-d`-refused
   (diverged upstream tracking). Either `git branch -D
   phase-m3-core-{e2e-pilot,emergency-mute,entry-kinds}` after
   confirming, or `git branch --unset-upstream` each then `-d`. Left in
   place per no-destructive-defaults. *(W31 #3, W32 #3, W34-783 #3,
   W34-784 #3 — unactioned.)*
4. **Disk `/` at 77%, stable.** Non-Brehon growth (`/srv/media`), not
   worktree buildup. 52G free / below 90% — monitor, no action needed.
   *(W34-784 #4 — unchanged.)*
5. **Weekly-review cadence: no fresh eval/retro material for 7+
   consecutive runs.** Newest qa-result **2026-06-11**, newest retro
   **2026-06-27** — both unchanged. Worth revisiting whether weekly
   dispatch should skip/shrink when nothing has changed since the last
   report, vs the double-run seen at W34 (job-783 mid-week, job-784 on
   the Sunday slot) — no repeat of that double-run this week (single
   dispatch, job-785, landed on the Sunday slot). *(W30 #7, W31 #6, W32
   #6, W34-783 #7, W34-784 #5 — unactioned.)*
6. **Laptop-only steps 1b-backfill/2d/2e + TH-only 2b not run**
   (daemon context) — run laptop-side to complete the week. *(W31 #5,
   W32 #5, W34-783 #6, W34-784 #6.)*
