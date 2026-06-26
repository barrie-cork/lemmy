# Weekly review — 2026-W26 (run 2026-06-26, daemon-side)

Run by a Junior task (`job-773`) on `homeserver` per `.claude/skills/weekly-review/SKILL.md`.
Daemon-side execution: laptop-only steps (1b PMD embedding backfill, 2b lesson clustering, 2d
MEMORY.md ACTIVE-line drift) cannot run here — their `C:/Users/barri/...` paths and the
laptop-side checkout live on the laptop, not the daemon. Branch
`junior/review-...-773`, trunk HEAD `d2ad8d88a`.

## ✅ W25's store-split count-divergence is RESOLVED on the active path
Last week (W25) the live PMD store and the MCP view disagreed (on-disk `memory.db` = 632 rows
vs MCP = 831) — a file-swap-under-daemon split. **This week the active serving path is
consistent:** on-disk `/srv/brehon-fork/.project-memory/memory.db` = **831 rows** (pre-prune),
`PRAGMA quick_check` = `ok`, max_id 952 — and the MCP `memory_review` reported the same 831.
Topology confirms why: the live store (inode 7471942) is held **non-deleted** by two fresh MCP
server processes (PIDs `2926036` / `2926325`, started ~03:00 ago), which serve this session's
MCP connection. Mutations this run hit the healthy live store.

## ✅ Embedding fully caught up — semantic search at full strength
Post-prune the live store has **0 unembedded rows**. The `memory-backfill.timer`
(systemd, every 5 min) embeds new rows automatically on the daemon. The
`memory_search_hybrid` +62.7% semantic-recall advantage is fully available this week.
**Note:** the skill's Step 1b text still describes backfill as laptop-only (homeserver Ollama
over Tailscale). Under the current topology that work is automated daemon-side by the timer —
Step 1b is effectively a no-op safety net now, not a laptop chore. Skill text is stale here.

## 🟠 RESIDUAL — orphan PMD daemon still holds `(deleted)` inodes
The latent hazard W25 flagged is not fully cleared. The old systemd daemon
`project-memory-http.service` (MainPID **2928953**, up since 2026-06-13, ~12 days) still holds
deleted inodes: `fd 20 → memory.db (deleted)`, `fd 21 → ...-wal (deleted)`, `fd 22 →
...-shm (deleted)`. It serves nothing this session touched (the fresh PIDs above do the work),
so it is currently **harmless** — but it is dead weight holding an unlinked inode, and is the
seed of exactly the swap-under-daemon split W25 hit. Cruft in `.project-memory/` reinforces
this: a 0-byte file literally named `memory.db (deleted)` (Jun 13) and a
`memory.db.stale-20260613T222551Z` snapshot. **Action: reap the orphan daemon and clean the
cruft** (human/advisor — not auto-applied; restarting/killing a 12-day PMD process is not a
Junior code-task call).

## 🟠 Frontmatter sweep (Step 1c) — 2 lessons need a fix
`scripts/brehon/lesson-frontmatter-lint.sh` exited 2 (1 of 232 would be silently skipped):
- **BROKEN — `feedback_daemon_sync_before_dispatch.md`**: frontmatter present but `name:` field
  is empty/absent. **SILENTLY SKIPPED** by `sync-lessons-to-pmd.sh` and the `lesson-pmd-sync.sh`
  hook — invisible to `memory_search_hybrid` recall until fixed.
- **NESTED — `feedback_bm_pr_daemon_finalize_merges_phase_into_trunk.md`**: `type:` is nested
  under `metadata:` with no top-level `type:`. Imports, but diverges from the corpus convention;
  flatten to a top-level `type:`.

**Not auto-fixed** (per skill: frontmatter `name`/`description` is a human authoring decision).
Action: add the missing/flattened frontmatter to both, then re-run the sync.

## 🟠 Duplicate dispatch — jobs 773 and 774 both ran this weekly-review
`git worktree list` shows two concurrent worktrees on the same task slug
(`...weekly-review-skill-md-773` and `...-774`), both at `d2ad8d88a`. Both will write a W26
report + a summary memory and finalize-merge into `governance-v0` — expect a merge conflict on
this report file and a duplicate summary memory. Action: dedup the dispatch upstream.

## Steps that ran (daemon-executable)
- **1. Memory health + prune** — `memory_review`: 831 total (550 qa-result, 246 pattern, others).
  `memory_prune` (real): **292 expired qa-results deleted, 0 superseded → 539 rows**. Verified
  on-disk post-prune.
- **1c. Lesson frontmatter sweep** — 2 findings (above).
- **3. Eval metrics aggregation** — 40-eval sample: **success 80%** (32 success / 4 partial /
  0 failure), **avg score 0.657** (min 0.45, max 0.88 — within the 0.60–0.75 calibration band,
  no inflation). Top root cause `type_alias_caller_mismatch` (6), then a retro-hook/infra class
  (~7), then stale-worktree partials (4). Caveat: `memory_get_recent`'s "recent" slice skewed to
  2026-06-07..06-11 (known PMD title-vs-created time-skew), so a strict last-7-day count is
  unreliable; numbers are a recent-sample aggregate.
- **4. Git hygiene** — `git worktree prune`: clean (only job-773/774 present, both live).
- **4b. Disk headroom** — root fs **73% used** (61 G free); `/var/log` and `/tmp` are not
  separate mounts. Under the 75% threshold — no action.

## Steps DEFERRED / not runnable here
- **1b. PMD embedding backfill** — superseded by the daemon `memory-backfill.timer`; 0
  unembedded rows confirm it is current. Laptop run unnecessary this week.
- **2 / 2b. Promotion + lesson clustering** — `memory_review` surfaced only generic
  high-frequency tags (brehon-fork, success, lesson…), no specific file/pattern promotion
  candidate. Lesson clustering is TH/laptop-only per the skill.
- **2c. Retro-harvest sweep** — the `mtime -7` filter is meaningless in a fresh `git worktree`
  (all 301 retro files carry the checkout mtime, not authorship). Defer to the dedicated
  `retro-harvest` skill / a laptop run.
- **2d. MEMORY.md ACTIVE-line drift** — needs the laptop `C:/Users/barri/...` MEMORY.md path;
  laptop-only.
- **5 METRICS.md append** — no general weekly METRICS.md in this repo (only the unrelated
  `brehon-conformance-audit/METRICS.md`); the hive-wide file is laptop-side. Summary captured in
  the summary memory instead, matching W25.

## Action items for advisor / human (none auto-applied)
1. Reap orphan PMD daemon PID 2928953 (12-day `(deleted)`-inode hold) + clean `.project-memory/`
   cruft (`memory.db (deleted)` 0-byte, `.stale-20260613T222551Z`).
2. Fix frontmatter on `feedback_daemon_sync_before_dispatch.md` (empty `name:`) and
   `feedback_bm_pr_daemon_finalize_merges_phase_into_trunk.md` (nested `metadata.type`), then
   re-run `scripts/sync-lessons-to-pmd.sh`.
3. Dedup the duplicate weekly-review dispatch (jobs 773 + 774).
4. Verify + delete merged `m3-core-stage-mode` `junior/*` branches (needs Junior task-status
   query — not done here: MCP unavailable in this worktree + shared repo + concurrent sibling →
   no destructive branch deletion).
5. Consider refreshing the skill's Step 1b prose to reflect the daemon `memory-backfill.timer`
   topology (currently describes a laptop-only flow that the timer has superseded).

## Summary line
W26: pruned 292 (831→539), 0 unembedded (search healthy), store-split count-divergence resolved
on active path, orphan deleted-inode daemon + 2 frontmatter lessons + duplicate 773/774 dispatch
surfaced. Eval sample: 80% success, avg 0.657. Disk 73%. No auto-remediation beyond the prune.
