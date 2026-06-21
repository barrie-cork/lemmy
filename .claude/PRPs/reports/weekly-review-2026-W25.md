# Weekly review — 2026-W25 (run 2026-06-21, daemon-side)

Run by a Junior task on `homeserver` per `.claude/skills/weekly-review/SKILL.md`. Daemon-side
execution: laptop-only steps (1b PMD embedding backfill, 2b lesson clustering, 2d MEMORY.md
ACTIVE-line drift) are deferred — their `C:/Users/barri/...` paths and Ollama-reachable network
live on the laptop, not the daemon. Branch `junior/review-...-758`, HEAD `08696191d`.

## ✅ W24's CRITICAL finding is RESOLVED — PMD search is back up
Last week (W24) the live PMD store was corrupt and **all search was DOWN**
(`database disk image is malformed` on both `memory_search` and `memory_search_hybrid`). **This
week search works.** `memory_search` / `memory_get_recent` return clean results, and
`PRAGMA quick_check` on the on-disk `memory.db` returns `ok` (632 rows, 8.8 MB). The btree
corruption was repaired at some point since 2026-06-14.

## 🟠 NEW finding — the store-split / file-swap-under-daemon condition has RECURRED
The corruption is fixed but the **underlying split that W24 flagged for root-cause
investigation is still live**, and it has bitten again:

- **The HTTP daemon serves a `(deleted)` inode.** `project-memory-http.service` (MainPID
  2928953) holds `fd 20 → /srv/brehon-fork/.project-memory/memory.db **(deleted)**`,
  `fd 21 → memory.db-wal (deleted)`, `fd 22 → memory.db-shm (deleted)`. The on-disk `memory.db`
  was swapped under the running daemon; the daemon is serving the OLD, now-unlinked inode — the
  exact hazard #1 from W24.
- **Two stores diverge.** The MCP (daemon, deleted-inode store) reported 831 memories pre-prune.
  The *current on-disk* `memory.db` (mtime 2026-06-21 01:57) has **632** rows, max
  `created_at = 2026-06-18`, with 11 rows since 2026-06-14 — **all `pattern` type (lesson syncs),
  0 qa-result**. A daemon restart would load this on-disk file and silently swap the served
  store.
- **Consequence:** `memory_write` / `memory_write_eval` via the MCP land in the deleted-inode
  store and are lost on the next daemon restart or snapshot; the on-disk store and the served
  store are not the same data. Per `.claude/rules/pmd-invariants.md` §1 store-split class.

### Recommended human/advisor actions (NOT auto-applied; no sudo in this worker)
1. Checkpoint the daemon's WAL into a recovered copy, then atomically restart
   `project-memory-http.service` so it drops the `(deleted)` inode and reopens a single
   canonical on-disk file. Verify `fd 20` points at a non-`(deleted)` path afterward.
2. Reconcile the two stores BEFORE restarting — a naive restart adopts the 632-row on-disk file
   and abandons whatever only exists in the deleted inode (the daemon store had 831 pre-prune).
3. Finally open the recurring-store-split root cause W24 already requested — this is now the
   **2nd consecutive week** the file-swap-under-daemon pattern has appeared.

## 🟠 NEW finding — no qa-result evals written in the last 7 days
`memory_get_recent(qa-result, 100)` shows the **most recent eval is 2026-06-11** (10 days ago);
**zero qa-result evals created in the 2026-06-14→21 window** — in either store. Yet git shows
**12 retro reports committed** in that window and active m3-core development. Either
per-task `memory_write_eval` writes are not reaching the canonical store (consistent with the
store-split above), or recent Junior tasks skipped the post-task-retro eval write. Surface to
advisor: the retro-coverage signal (autonomy criterion 5.2) cannot be computed this week and the
gap itself is the finding.

## Steps that ran (daemon-executable)
| Step | Status | Result |
|---|---|---|
| 1. Memory review | ✅ | 831 memories (qa-result 550, pattern 246, issue-note 12, summary 12, decision 8, deploy-note 2, bug 1); **199 expired** prunable, 0 superseded; no file/tag promotion candidates surfaced. |
| 1. Memory prune (not dry-run) | ✅ | **Pruned 199** (expired + superseded) against the live daemon store. |
| 1c. Lesson frontmatter lint | ⚠️ finding | `scripts/brehon/lesson-frontmatter-lint.sh` **RC=2** — 1 of 230 lesson files would be SILENTLY SKIPPED by the PMD sync: **`feedback_daemon_sync_before_dispatch.md`** (frontmatter present but no `name:` field — it uses `title:` instead). Invisible to `memory_search_hybrid` until fixed. **Not auto-fixed** (name/description is a human authoring decision per Step 1c); see action below. |
| 2. Promote candidates | ➖ | None surfaced by Step 1; nothing to promote. |
| 2c. Retro-harvest sweep | ✅ | Wrote `.claude/harvest/2026-W25.md` (gitignored). 12 retros committed last 7 days; **7** proposal-bearing → manual-triage queue. Full STALE/LIVE/SUPERSEDED triage deferred to advisor (PMD cross-ref untrustworthy until store-split reconciled). |
| 3. Eval-metrics aggregation | ⚠️ no data | **0 evals in the 7-day window** (most recent 2026-06-11). Success-rate / avg-score / top-root-cause / retro-coverage all uncomputable this week — see "no qa-result evals" finding above. |
| 4. Git hygiene | ⚠️ partial | `git worktree prune --dry-run` RC=0 — nothing stale (only canonical `phase-m3-core-e2e-pilot` + this job-758 worktree; 1 job worktree on disk). Branch deletion **deferred** — classifying terminal-status branches needs the Junior DB (junior MCP unavailable in this worker). 81 local branches: junior 10, ab-test 9, ab-cell 8, chore 2, plus phase-*/tmp-*/smoke. |
| 4b. Disk headroom | ✅ clean | `/` (only mount) at **68%** used, 73 GB free — under the 75% threshold (W24 was 89%; improved). Note: `.project-memory/` still holds **46 MB** of corruption-recovery `.db` sprawl (`memory-malformed-2026-05-21.db`, `memory-repaired*.db`, two 0-byte `memory*recovered/repaired.db`) — left in place (forensics, not this worker's call). |
| 5. Summary memory_write | ✅ (caveated) | Written to the live searchable store, but durability is suspect — it lands in the deleted-inode daemon store and may not survive a restart. This committed report is the durable record. |
| 6. Commit | ✅ | This report, `Review:`-prefixed (prevents weekly-review cron cascade). |

## Steps DEFERRED (laptop-only — wrong host)
| Step | Why |
|---|---|
| 1b. PMD embedding backfill | Must run laptop-side against the canonical DB + Ollama (`homeserver:11434`). Reads/writes the laptop `C:/Users/barri/...memory.db`; SQLite is not network-accessible. **Recommend running it laptop-side this week — but only after the store-split above is reconciled** (backfilling a fork of the served store is wasted work). |
| 2b. Lesson clustering | Marked "TH only" in the skill; requires semantic clustering judgment over `/reflect` lessons. Defer to laptop. |
| 2d. MEMORY.md ACTIVE-line drift | Reads `C:/Users/barri/.claude/.../MEMORY.md` + `v1-roadmap.json` — laptop paths absent on daemon. |

## Action items for advisor / human (none auto-applied)
1. **Reconcile + restart the PMD HTTP daemon** to clear the `(deleted)` inode and converge the
   two stores (see store-split finding) — then open the recurring root-cause investigation.
2. **Fix `feedback_daemon_sync_before_dispatch.md` frontmatter** — add a `name:` field (the file
   has `title:`/`description:`/`tags:` but the sync requires `name:`), then re-run
   `scripts/sync-lessons-to-pmd.sh`. The obvious slug is `daemon-sync-before-dispatch`.
3. **Run Step 1b backfill laptop-side** once the store is reconciled.
4. **Investigate the eval-write gap** — confirm whether recent m3-core Junior tasks wrote
   qa-result evals at all, or whether the writes were eaten by the store-split.

## Summary line
Pruned 199 | Promoted 0 | Lessons-frontmatter **1 BROKEN** (`feedback_daemon_sync_before_dispatch`)
| Branches cleaned 0 (deferred) | Disk / 68% (clean) | Evals last-7d **0** (coverage uncomputable)
| **W24 PMD corruption RESOLVED — search back up; BUT store-split/deleted-inode RECURRED (2nd
week), reconcile + restart needed before backfill.**
