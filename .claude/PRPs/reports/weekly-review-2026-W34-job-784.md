# Weekly Review — 2026-W34 (daemon-side, job-784)

Run: 2026-08-23 ~02:30 UTC, Junior task job-784, on homeserver/EliteDesk.
Skill: `.claude/skills/weekly-review/SKILL.md`. Branch:
`junior/review-read-and-follow-claude-skills-weekly-review-skill-md-784`.
Daemon trunk HEAD: `15909bfd5`. **Second run within ISO week 34** — job-783 ran
2026-08-18 (mid-week); this run landed on the actual Sunday 02:00 UTC cadence
slot (2026-08-23 is the last day of ISO week 34; week 35 starts Monday 2026-08-24).
Since job-783 (`31b30cd6a`), trunk advanced by exactly one commit: the merge of
job-783's own report (`15909bfd5`). No implementation/retro activity in the
window — 6th straight quiet cadence (W29–W32, W34×2).

## Headline — quiet week; the PMD prune-revert vector escalated (7th confirmation, now sub-week cadence)

1. **Store re-inflated to 1012 again — only 5 days after job-783 pruned it to
   458.** Pre-prune this run was **1012**, byte-identical (9592832) to job-783's
   pre-prune five days earlier, and the newest hourly snapshot (`brehon-fork-
   20260823T0200.db`, 32 min old at run time) matched exactly: **1012 rows, max
   id 1133**. Single store confirmed (no split) — this is the 7th confirmation
   of the revert vector (W27, W29/W30, W31, W32, W34-783, now W34-784), but the
   **cadence just got faster**: prior recurrences were fortnight-over-fortnight;
   this one reverted within under a week. **Strongest corroborating evidence
   yet:** this run's summary memory landed at **ID 1134 — the identical id**
   job-783, W31, and W32 all got for their summary writes, despite ~5 days of
   intervening role-signal/lesson-sync writes that should have pushed max id
   well past 1134. The store's max id keeps resetting to 1133 regardless of
   what was written in between. Still unidentified (needs `sudo`; blocked for
   Junior workers).
2. **RESOLVED-this-run — prune durable + checkpointed again.** Pruned 558
   expired (0 superseded) → **454**, WAL `TRUNCATE` checkpoint immediately
   after prune and again after the summary write (**455** final, WAL 0 bytes
   both times). Durable *as of now*, at the same (now demonstrably <7-day)
   revert risk as #1.
3. **STILL OPEN — same 3 merged `phase-m3-core-*` branches** (unchanged since
   W31/W32/W34-783; see findings).

## Steps executed (daemon-runnable)

### 0. Store-identity assertion (W30–W34 precedent) ✓
MCP `memory_review` = **1012**; on-disk `/srv/brehon-fork/.project-memory/memory.db`
= **1012**; newest snapshot `hourly/brehon-fork-20260823T0200.db` = **1012**,
max id **1133** — identical to job-783's pre-prune state. All agree → single
store, no split. `.mcp.json` confirms stdio transport,
`PROJECT_MEMORY_DB=/srv/brehon-fork/.project-memory/memory.db`; no HTTP daemon
unit exists on this host (`project-memory-http.service`: "could not be found").
Prune safe to trust this run.

### 1. Memory health + prune ✓ (durable, checkpointed)
- Pre-prune **1012**. Prunable: **558 expired, 0 superseded**.
- `memory_prune` (not dry-run) → **pruned 558** → **454**.
- `PRAGMA wal_checkpoint(TRUNCATE)` → WAL 0 bytes, durable count **454**.
  Post-summary write + 2nd checkpoint → **455**, WAL 0.

### 1b. PMD embedding backfill — LAPTOP-ONLY per skill; not run here (daemon
worker, no laptop DB access). No local diagnostic run this cycle (job-783
already confirmed 0-unembedded on-disk pre-summary; `memory-backfill.timer`
is the enforcement point and runs independent of this task).

### 1c. Lesson frontmatter sweep ✓ — ALL CLEAN
`scripts/brehon/lesson-frontmatter-lint.sh` over **234** lesson files → RC=0,
"sync would import cleanly." No BROKEN/NESTED files (unchanged since W31).

### 2. Promote candidates — none actionable
`memory_review` surfaces only generic infra tags (brehon-fork 523, success 384,
lesson 311, advisor 248, feedback 238, role-signal 194, kind:utilisation 190,
impl-task 167, role:impl-task 137, config_version:… 137). No file/pattern
cluster warrants `docs/memory/` promotion. Same as W26–W34-783.
(`docs/memory/` does not exist in this repo.)

### 2b. Lesson clustering — SKIPPED (skill: TH-only; skip on server repos).

### 2c. Retro-harvest sweep ✓ — 0 new proposals
`git log --since='7 days ago' -- .claude/PRPs/reports/ .claude/decision-queue.json`
→ 1 commit in window, and it is job-783's own weekly-review report — not a
session/phase retro. Zero new `session-retro-*.md` or `*-retro.md` files
authored in the window. Newest retro on disk (by git authorship date, not
mtime — phase-retro files share a clone-artifact mtime) is still
**2026-06-27**, unchanged since W32/W34-783. Written to gitignored
`.claude/harvest/2026-W34.md`. Surfacing only.

### 2d. MEMORY.md ACTIVE-line drift — LAPTOP-ONLY (skill path `C:/Users/barri/…`
absent on daemon).

### 2e. Role-signal drain + role-health — LAPTOP-ONLY (needs laptop DB +
`ssh homeserver` scp *from* the laptop; this task runs *on* homeserver).

### 3. Eval-metrics aggregation ✓ (computed from frozen pre-prune snapshot)
Computed against a copy of `hourly/brehon-fork-20260823T0200.db` (qa-result
rows were already pruned from the live store by the time this ran):
- **0 evals in the trailing 7d.** Newest `qa-result` is **2026-06-11**
  (unchanged since W31/W32/W34-783) — the eval-producing activity gap is now
  **~10.7 weeks**. Activity gap, not retro-bypass.
- 550 qa-result total; **526 scored, avg 0.706, median 0.700** — same as
  W34-783, squarely in the calibrated 0.60–0.75 band.
- Calibration: **23/526 (4.4%) scored >0.85** — flat vs W30–W34-783.
- ROOT_CAUSE present but parse-noisy (regex differs slightly run to run);
  top non-artifact categories `environment-issue` (25), `process` (21),
  `missing-verification` (7) — consistent with prior weeks' `process`/
  `environment` leaders.
- **Duplicate-retro storm** not re-checked this run (0 new evals means no new
  duplicate clusters could form); prior weeks' clusters (`v1-SL-e idle
  polling` etc.) are frozen in the same pruned/expired corpus.

### 4. Git hygiene ✓ — clean; 3 branches surfaced (not force-deleted)
- `git worktree prune`: nothing to prune. Only **2** worktrees on disk
  (canonical `governance-v0` + this job-784).
- **33** local branches. `phase-m3-core-{e2e-pilot,emergency-mute,entry-kinds}`
  are all verified **ancestors of `origin/governance-v0`** (content merged,
  safe) but `git branch -d` still refuses them (diverged `origin/<branch>`
  upstream tracking, several `[ahead N]`/`[gone]`). **NOT force-deleted** —
  `no-destructive-defaults.md` forbids `-D`/`--force` without explicit user
  request. Surfaced for human (unchanged from W31/W32/W34-783).

### 4b. Disk headroom ✓ — 76%, stable
`/` at **76%** (167G/232G, 54G free) — essentially unchanged from W34-783's
76% five days ago (+1G). Ran `df` locally (this task runs *on* homeserver;
the skill's `ssh homeserver "df"` self-SSH is a no-op). Non-Brehon growth
dominates: `/srv/media` 158G, `/srv/web-archive-data` 14G. Brehon's footprint
is stable at **19G** (of which `target/` 8.1G). Only **1** worktree dir → no
worktree buildup. systemd journal 3.5G (vacuumable to 500M if pressure
grows). No auto-remediation: below the 90% surface-to-user line, and the
growth is non-Brehon media data outside this task's remit.

### 5. Summary ✓
Summary memory written to the durable on-disk store — **ID 1134**, the
**same id** W31, W32, and W34-783 all got, despite ~5 days of intervening
writes between this run and job-783's. This is the strongest evidence yet
that the store's max-id resets to 1133 on every revert, not just that row
*counts* match. Checkpointed. Global `METRICS.md` does not exist in this
repo; W26–W34 set the report-only precedent — followed, not created
unilaterally.

### 6. Commit + push ✓
This report + the gitignored-excluded harvest note, `Review:`-prefixed
(prevents the weekly-review cron cascade). Pushed to origin per the
W32-era Step-6 fix (`0e026a0a3`).

## Findings for the human (priority order)

1. **PMD prune-revert vector escalated to sub-week cadence — find the revert
   source.** Confirmed a 7th time, and for the first time the revert happened
   in **under 5 days** (prior recurrences were fortnight-scale: W27, W29/W30,
   W31, W32, then W34-783→W34-784 is the first back-to-back-week repeat with a
   measured interval). The repeated **ID 1134** across four separate runs
   (W31, W32, W34-783, W34-784) proves this is a snapshot restore/reseed job
   overwriting the live store back to the exact pre-W31 state (max id 1133),
   not merely lost writes accumulating independently each time. Given the
   faster cadence, this now needs a `sudo`-level investigation (cron table,
   systemd timers, a `pmd-snapshot`/restore service) sooner rather than later
   — the pruned state is durable for less than a week at current rate.
   *(W29 #1/#2, W30 #1, W31 #1, W32 #1, W34-783 #1 — open, escalating.)*
2. **Fold the proven daemon-side fixes into `SKILL.md`.** Still absent after
   6+ runs: (a) **Step 0 store-identity assertion** (compare MCP count vs
   newest `/srv/backups/pmd/<repo>/hourly/*.db` before pruning — now also
   worth comparing max `id`, not just row count, per finding #1); (b)
   **Step 2c** should specify `git log --since` not `-mtime -7`; (c)
   **Step 4b** should branch on `hostname` (self-SSH `df` is a no-op on the
   daemon). *(W30 #2/#4, W31 #2, W32 #2, W34-783 #2 — unactioned across 4+
   runs now.)*
3. **3 merged `phase-m3-core-*` branches need human `-D`.** All ancestors of
   `origin/governance-v0` (content safe) but `-d`-refused (diverged upstream
   tracking). Either `git branch -D phase-m3-core-{e2e-pilot,emergency-mute,
   entry-kinds}` after confirming, or `git branch --unset-upstream` each then
   `-d`. Left in place per no-destructive-defaults.
   *(W31 #3, W32 #3, W34-783 #3 — unactioned.)*
4. **Disk `/` at 76%, stable.** Non-Brehon (`/srv/media` growth), not
   worktree buildup. 54G free / below 90% — monitor, no action needed this
   run (essentially flat vs W34-783). *(W34-783 #4 — unchanged, downgraded
   from "new" since it's now stable.)*
5. **Weekly-review keeps dispatching against a daemon worktree with no fresh
   eval/retro material.** Newest qa-result **2026-06-11**, newest retro
   **2026-06-27** — both unchanged for 6+ consecutive runs now. This run also
   landed as an unplanned **second run within the same ISO week** (job-783
   mid-week, job-784 on the actual Sunday slot) — worth checking whether the
   cron/dispatch cadence is double-firing, or whether mid-week catch-up runs
   should skip re-running the same ISO week's report. *(W30 #7, W31 #6, W32
   #6, W34-783 #7 — unactioned, now compounded by the same-week double-run.)*
6. **Laptop-only steps 1b-backfill/2d/2e + TH-only 2b not run** (daemon
   context) — run laptop-side to complete the week, especially 2e's
   role-health report (its role-signal queue on homeserver only accumulates
   until worker worktrees reap). *(W31 #5, W32 #5, W34-783 #6.)*
