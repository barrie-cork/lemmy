# Weekly Review — 2026-W27 (daemon-side, job-776)

Run: 2026-07-05 02:31 UTC (Sunday), Junior task job-776, on homeserver/EliteDesk.
Skill: `.claude/skills/weekly-review/SKILL.md`. Branch:
`junior/review-read-and-follow-claude-skills-weekly-review-skill-md-776`.
Base repo HEAD: `1aba68822` — **frozen at 2026-06-28** (last W26/job-775 review merge;
no trunk commits merged in the intervening week).

## Headline finding (action required, STILL UNREAPED — now 3× flagged) — orphan PMD HTTP daemon

The systemd `project-memory-http.service` is **still an orphan holding deleted inodes.**
The service was restarted since W26 (MainPID is now **1532**, started 2026-07-05 00:04,
was 2928953 in W26) but is **re-orphaned** — its fds again point to unlinked inodes:

```
/proc/1532/fd/21 -> /srv/brehon-fork/.project-memory/memory.db      (deleted)
/proc/1532/fd/22 -> /srv/brehon-fork/.project-memory/memory.db-wal  (deleted)
/proc/1532/fd/23 -> /srv/brehon-fork/.project-memory/memory.db-shm  (deleted)
```

The on-disk `memory.db` (mtime 2026-07-03 21:52; `memory.db.good-20260629T113058Z.bak`
fossil alongside) was replaced under the running daemon by a restore/recovery event, so
the HTTP daemon serves the OLD (unlinked) inode. **Impact (unchanged from W26):**
HTTP/remote clients — i.e. LAPTOP advisor sessions, which author the bulk of retros —
read/write the frozen deleted-inode store; those writes are lost when the process dies.
Proof: this task's **stdio** MCP reads the LIVE on-disk store whose newest qa-result is
frozen at **2026-06-11** — a full ~3-week retro-write gap, because every laptop retro
since then routed to the orphan.

**A bare `systemctl restart` is NOT a durable fix** — W26 restarted it and it re-orphaned
within days because *something keeps replacing the on-disk `memory.db` out from under the
live daemon* (restore scripts / snapshot-restore). The durable fix must stop the
file-replacement-under-a-live-fd, then restart. Restart command (coordinate first — an
active laptop advisor session is an HTTP client):

```bash
sudo systemctl restart project-memory-http.service
systemctl show -p MainPID --value project-memory-http.service
ls -la /proc/$(systemctl show -p MainPID --value project-memory-http.service)/fd | grep memory.db  # must NOT say (deleted)
```

NOT executed by this Junior task: restarting a shared service an active laptop session
depends on is an outward-facing, coordinate-first action (no-destructive-defaults; W26
made the same call).

## Steps executed (daemon-runnable)

### 1. Memory health + prune ✓
Arithmetic integrity check confirms this task's **stdio** MCP targets the LIVE on-disk
store: pre-prune 1012 → pruned 446 → post-prune 566 (qa-result 550 → 104; exact).
- **Pruned: 446** expired entries (all expired qa-result task-retros, 30-day expiry). 0 superseded.
- Store: 1012 → **566 rows** (pattern 246, summary 193, qa-result 104, issue-note 12, decision 8, deploy-note 2, bug 1).
- Post-prune prunable: **0**. Live store clean.

### 1b. PMD embedding backfill — LAPTOP-ONLY (skill); read-only diagnostic done here ✓
Per skill, `backfill.js` MUST run laptop-side. Read-only diagnostic on the live on-disk
store: **0 unembedded rows** (NOT-EXISTS check), `memory_vectors` = 566 = total 566.
Live-store embeddings are current — no backfill needed for the durable store. (The orphan
HTTP store's embedding state is irrelevant — it is doomed on next process death.)

### 1c. Lesson frontmatter sweep ✓ — 2 files flagged (surfaced, NOT auto-fixed)
`scripts/brehon/lesson-frontmatter-lint.sh` over 232 lesson files (exit 2):
- **BROKEN:** `feedback_daemon_sync_before_dispatch.md` — frontmatter present but `name:`
  empty/absent → **SILENTLY SKIPPED by PMD sync, invisible to `memory_search_hybrid`.**
  Fix: add a non-empty top-level `name:`, re-run `scripts/sync-lessons-to-pmd.sh`.
- **NESTED:** `feedback_bm_pr_daemon_finalize_merges_phase_into_trunk.md` — `type:` nested
  under `metadata:`, absent at top level → imports but diverges from corpus convention; flatten.
- **Unchanged from W26** — both files still unfixed. Per skill, frontmatter content is a
  human authoring decision — surfaced, not auto-fixed.

### 2. Promote candidates — no actionable promotions
`memory_review` promotion candidates were generic infra tags only (lesson, feedback,
role-signal, kind:utilisation, brehon-fork, success, impl-task…), no file/pattern
clusters warranting a `docs/memory/` promotion. None promoted (same as W26).

### 2b. Lesson clustering — SKIPPED (skill: "TH only / skip on server repos").

### 2c. Retro-harvest sweep ✓
Recency via `git log --since` (mtime unreliable in fresh worktree — job-773 W26 lesson).
**Repo HEAD frozen at 2026-06-28 → NO new retro reports merged since W26.** The
carry-over candidates (docker-age-teardown guard, task-status triage agent, go-public
README/CONTRIBUTING/docs-reports-move) were re-verified as **still un-promoted** and
written to gitignored `.claude/harvest/2026-W27.md`. Go-public items are System-1
tracked (`project_go_public_plan.md`).

### 2d. MEMORY.md ACTIVE-line drift — LAPTOP-ONLY (skill uses `C:/Users/barri/…` path, absent on daemon). Not runnable here.

### 2e. Role-signal drain + role-health — LAPTOP-ONLY (skill: needs laptop DB + `ssh homeserver` scp FROM homeserver). Not runnable from the daemon. Surfaced. NB: `role-signal` (192) + `kind:utilisation` (190) rows are present in the live store, so the drain HAS been reaching the canonical DB historically; the per-role strip-candidate report still requires the laptop `/check-role-health` pass.

### 3. Eval metrics aggregation ✓ (with frozen-window caveat)
Window: newest 7 days available in the live store = **2026-06-04 → 2026-06-11** (all 104
remaining qa-results predate 2026-06-11 — see headline: writes after that routed to the orphan).
Sample n=104 qa-results:
- **Outcome:** 85 success / 13 partial / 1 failure / 5 unscored.
- **Avg score:** 0.707 (n=100 scored) — within the calibrated 0.60–0.75 band (upper edge).
- **Top root causes:** environment-issue (10), type_alias_caller_mismatch (6), missing-verification (5), infra (3).
- **Caveat:** the ~3-week retro-coverage gap (newest qa 2026-06-11) is an ARTIFACT of
  the orphan daemon diverting recent laptop retro writes, NOT a true retro-bypass.
  Resolves once the daemon file-replacement issue is fixed + service restarted.

### 4. Git hygiene ✓ — clean
`git worktree prune`: nothing to prune. 2 live worktrees (canonical governance-v0 + this
job-776); 1 job dir on disk (this one). 72 local branches incl. ~40 merged `phase-*` —
NOT deleted (daemon-side report-only precedent; branch deletion on the shared `.git/` is
a coordinate-first action while other jobs may reference them). Flagged for laptop-side
cleanup. Only `junior/*` branch present is this task's own.

### 4b. Disk headroom ⚠ — at threshold (75%)
`/` at **75%** (165G/232G, 57G free) — up from 73% at W26, now AT the skill's 75%
investigate threshold (below the 90% surface-to-user level). Largest consumers:
`/srv/brehon-fork/target` (**8.1G** cargo build cache) and systemd journal (**3.3G**).
No separate `/var/log` or `/tmp` mounts (all `/`). NOT auto-remediated (Junior worker;
below 90%). Recommended laptop/human actions if headroom is wanted: `cargo clean` in an
idle window (reclaims ~8G) and/or `journalctl --vacuum-size=500M` (reclaims ~2.8G).

### 5. Summary
Summary memory written to PMD (live on-disk store, durable via stdio MCP). Global
`METRICS.md` does NOT exist (only `.claude/skills/brehon-conformance-audit/METRICS.md`,
a different artifact); W26 set the report-only precedent — followed here. Step-5
METRICS.md append skipped (no such file; not created unilaterally).

## Findings for the human (priority order)
1. **Fix the PMD HTTP daemon file-replacement-under-live-fd, then restart** (PID 1532;
   deleted inodes; 3× flagged W26×2 + W27; laptop retro writes lost since ~2026-06-11).
   A bare restart is not durable — the on-disk `memory.db` keeps getting replaced under
   the running daemon. Command above.
2. **Fix `feedback_daemon_sync_before_dispatch.md` frontmatter** (empty `name:` →
   invisible to semantic recall) + flatten `metadata.type` in
   `feedback_bm_pr_daemon_finalize_merges_phase_into_trunk.md`. Both unchanged since W26.
3. **Disk `/` at 75%** — `target/` 8.1G + journal 3.3G. `cargo clean` / journal vacuum
   in an idle window if headroom wanted.
4. **Laptop-only steps 1b/2d/2e + TH-only 2b NOT run** (daemon context) — run laptop-side
   to complete the week (esp. 2e role-health strip-candidate report).
5. **Weekly-review is dispatching against a frozen daemon worktree** (HEAD 2026-06-28) —
   metrics + harvest have no fresh material to work with. Consider running weekly-review
   laptop-side (interactive), or syncing the daemon worktree to current trunk before dispatch.
6. Harvest carry-over (still open, `.claude/harvest/2026-W27.md`): docker-age-teardown
   guard, task-status triage agent, go-public README/CONTRIBUTING/docs-reports-move.
