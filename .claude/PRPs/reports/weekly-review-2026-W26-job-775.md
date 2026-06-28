# Weekly Review — 2026-W26 (daemon-side, job-775)

Run: 2026-06-28 02:32 UTC (Sunday), Junior task job-775, on homeserver/EliteDesk.
Skill: `.claude/skills/weekly-review/SKILL.md`. Branch:
`junior/review-read-and-follow-claude-skills-weekly-review-skill-md-775`.

> **Duplicate-dispatch note:** this is the SECOND W26 weekly-review (job-773 ran
> W26 on 2026-06-26, report `weekly-review-2026-W26.md`). Duplicate weekly-review
> dispatch is now a 3× pattern (job-773 commit flagged a 773/774 dup). Surfaced
> for the human; this report is written under a `-job-775` suffix to avoid
> clobbering the job-773 W26 report.

## Headline finding (action required) — orphan PMD HTTP daemon, W26-flagged, STILL unreaped

The systemd `project-memory-http.service` (MainPID **2928953**) is an **orphan
holding deleted inodes**:

```
/proc/2928953/fd/20 -> /srv/brehon-fork/.project-memory/memory.db      (deleted)
/proc/2928953/fd/21 -> /srv/brehon-fork/.project-memory/memory.db-wal  (deleted)
/proc/2928953/fd/22 -> /srv/brehon-fork/.project-memory/memory.db-shm  (deleted)
```

The on-disk `memory.db` was replaced (recovery/restore — `memory.db.stale-20260613T222551Z`
+ `recovery-20260613T222530Z/` + a 0-byte `memory.db (deleted)` dirent are the
fossils) but the HTTP daemon was never restarted, so it keeps serving the OLD
(now-unlinked) inode. **Impact:** HTTP/remote clients — i.e. the LAPTOP advisor
sessions, which write the bulk of retros — read/write the frozen deleted-inode
store; those writes vanish when the process dies. This is exactly why the live
on-disk store's newest qa-result is frozen at **2026-06-11** while the live
store's newest entry of any type is 2026-06-26 (the few recent rows arrived via
local stdio/timer paths, not HTTP).

**This was surfaced in W26/job-773 (2026-06-26) "for reaping" and is still not
done.** Remediation (run on homeserver; restart is idempotent, but coordinate
— a live laptop advisor session is an HTTP client):

```bash
sudo systemctl restart project-memory-http.service
# verify it now serves the live inode (counts must match the on-disk read):
systemctl show project-memory-http.service -p MainPID
ls -la /proc/$(systemctl show -p MainPID --value project-memory-http.service)/fd | grep memory.db   # must NOT say (deleted)
```

NOT executed by this Junior task: restarting a shared service that an active
laptop session depends on is an outward-facing, coordinate-first action
(no-destructive-defaults; W26 made the same call).

## Steps executed (daemon-runnable)

### 1. Memory health + prune ✓
This task's **stdio** MCP correctly targets the LIVE on-disk store (proven by
arithmetic: pre-prune 1012 → pruned 308 → post-prune on-disk read 704; qa-result
550 → 242 — exact match). Prune was effective on the durable store.
- **Pruned: 308** expired entries (all expired qa-result task-retros, 30-day expiry). 0 superseded.
- Store: 1012 → **704 rows** (pattern 246, qa-result 242, summary 193, issue-note 12, decision 8, deploy-note 2, bug 1).
- Post-prune: **0 expired-but-present** remaining. Live store clean.

### 1b. PMD embedding backfill — LAPTOP-ONLY (skill); read-only diagnostic done here
Per skill, backfill MUST run laptop-side. Read-only diagnostic on the live store:
**0 unembedded rows** (NOT-EXISTS check). `memory_vectors`=508 (vec0 virtual-table
count quirk; the NOT-EXISTS check is authoritative). Live store embeddings are
current — no backfill needed for the on-disk store. (NB: the orphan store's
embedding state is irrelevant — it is doomed.)

### 1c. Lesson frontmatter sweep ✓ — 2 files flagged (surfaced, NOT auto-fixed)
`scripts/brehon/lesson-frontmatter-lint.sh` over 232 lesson files:
- **BROKEN:** `feedback_daemon_sync_before_dispatch.md` — frontmatter present but `name:` empty/absent → **SILENTLY SKIPPED by PMD sync, invisible to `memory_search_hybrid`.** Fix: add a non-empty top-level `name:`, re-run `scripts/sync-lessons-to-pmd.sh`.
- **NESTED:** `feedback_bm_pr_daemon_finalize_merges_phase_into_trunk.md` — `type:` nested under `metadata:`, absent at top level → imports but diverges from corpus convention; flatten to a top-level `type:`.
- Per skill: frontmatter content is a human authoring decision — surfaced, not auto-fixed.

### 2. Promote candidates — no actionable promotions
`memory_review` promotion candidates were generic infra tags only (brehon-fork,
success, lesson, advisor, feedback, role-signal…), no file/pattern clusters
warranting a `docs/memory/` promotion. None promoted.

### 2b. Lesson clustering — SKIPPED (skill: "TH only / skip on server repos")

### 2c. Retro-harvest sweep ✓
Recency via `git log --since` (NOT mtime — fresh-worktree mtime is unreliable,
per the job-773 W26 lesson). 5 retro/verify reports changed in last 7 days swept;
un-promoted proposals written to gitignored `.claude/harvest/2026-W26.md`.
Strongest harness-promotion candidates (lesson filename named + threshold met):
- `feedback_docker_end_state_check_age_before_teardown.md` (container-age-before-teardown guard).
- L3 MEMORY.md index drift: `feedback_build_what_tests_exercise.md` named but absent.
Go-public repo-content items (README/CONTRIBUTING/docs-move) are System-1 tracked
in `project_go_public_plan.md`, routed there, not to the lesson corpus.

### 2d. MEMORY.md ACTIVE-line drift — LAPTOP-ONLY (skill uses `C:/Users/barri/...` path, absent on daemon). Not runnable here.

### 2e. Role-signal drain + role-health — LAPTOP-ONLY (skill: needs laptop DB + `ssh homeserver` scp FROM homeserver). Not runnable from the daemon. Surfaced.

### 3. Eval metrics aggregation ✓ (with caveat)
Window: newest 7 days available in the live store = **2026-06-06 → 2026-06-11**
(qa-result writes after 2026-06-11 are absent from the live store — see headline:
they routed to the orphan). Sample n=60 qa-results (ids 860–941):
- **Outcome:** 46 success / 7 partial / 7 unscored.
- **Avg score:** 0.673 (n=59 scored) — within the calibrated 0.60–0.75 band.
- **Top root causes:** type_alias_caller_mismatch (6), environment-issue (5), infra (3).
- **Caveat:** the 17-day retro-coverage gap (newest qa 2026-06-11) is an
  ARTIFACT of the orphan daemon diverting recent retro writes, not a true
  retro-bypass. Resolves once the daemon is restarted.

### 4. Git hygiene ✓ — clean
`git worktree prune`: nothing to prune. 2 live worktrees (governance-v0 +
this job-775). Only `junior/*` branch is this task's own (not deleted). Nothing
to clean.

### 4b. Disk headroom ✓ — healthy
`/` at **73%** (160G/232G, 62G free) — below the 75% threshold. Same as W26
baseline. No action.

### 5. Summary
Summary memory written to PMD (live store, durable via stdio MCP). Global
`METRICS.md` does NOT exist (only `.claude/skills/brehon-conformance-audit/METRICS.md`);
job-773 W26 set the precedent of report-only — followed here. METRICS.md
Step-5 append skipped (no such file; not created unilaterally).

## Findings for the human (priority order)
1. **Reap/restart the orphan PMD HTTP daemon PID 2928953** (deleted inodes; W26-flagged, still live; laptop writes being lost). Command above.
2. **Fix `feedback_daemon_sync_before_dispatch.md` frontmatter** (empty `name:` → invisible to semantic recall). + flatten `metadata.type` in `feedback_bm_pr_daemon_finalize_merges_phase_into_trunk.md`.
3. **Duplicate W26 weekly-review dispatch** (job-773 + job-775) — 3× pattern; investigate the scheduler.
4. Laptop-only steps 1b/2d/2e + TH-only 2b NOT run (daemon context) — run laptop-side to complete the week.
5. Harvest candidates for review: docker-age-teardown guard + L3 index drift (`.claude/harvest/2026-W26.md`).
