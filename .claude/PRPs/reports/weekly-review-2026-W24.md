# Weekly review — 2026-W24 (run 2026-06-14, daemon-side)

Run by a Junior task on `homeserver` per `.claude/skills/weekly-review/SKILL.md`. Daemon-side
execution: laptop-only steps (1b PMD backfill, 2d MEMORY.md ACTIVE-line drift) are deferred —
their `C:/Users/barri/...` paths do not exist on the daemon.

## 🔴 CRITICAL FINDING — live PMD store is corrupt (search subsystem DOWN)

The single most important finding this week. **The live project-memory store the HTTP daemon
serves is corrupt, and all semantic/keyword search is failing.** Do NOT attempt repair without
a deliberate plan — the recovery is delicate (details below). I did **not** attempt repair and
**stopped further PMD writes** after detecting it.

### Symptoms
- `memory_search` (FTS5) and `memory_search_hybrid` both fail: `Search failed: database disk
  image is malformed` (retried per circuit-breaker — persistent on both paths).
- `memory_review` and `memory_prune` **succeeded** (they query the intact `memories` main table;
  search hits the corrupt btree). So the corruption is localized, not total.

### Diagnostics (all read-only)
- `PRAGMA quick_check` on `/srv/brehon-fork/.project-memory/memory.db`: **`Tree 9 page NNNN:
  btreeInitPage() returns error code 11`** (SQLITE_CORRUPT) across a long run of pages —
  genuine on-disk btree corruption (an index / FTS shadow tree), not just logical FTS drift.
- Live on-disk file: 796 memories (8.4 MB, mtime 2026-06-14 01:52).

### The recovery is delicate — three compounding hazards
1. **Daemon serves a `(deleted)` inode.** `project-memory-http.service` (PID 2928953) has
   fd 20 → `/srv/brehon-fork/.project-memory/memory.db **(deleted)**`. The on-disk `memory.db`
   was swapped under the running daemon (consistent with yesterday's
   `recovery-20260613T222530Z/`, `memory.db.stale-20260613T222551Z`, and a 0-byte
   `memory.db (deleted)` dirent). The daemon is serving the OLD, now-unlinked inode.
2. **Uncommitted writes pinned in WAL.** fd 21 → `memory.db-wal` (1.2 MB, mtime 02:30) holds
   writes against that deleted inode. A naive daemon restart or `rm` of the sidecars **loses
   those writes**. The WAL must be checkpointed into a recovered copy, not discarded.
3. **The hourly backups are NOT backups of the live store.** Every snapshot in
   `/srv/backups/pmd/homeserver/hourly/` is clean (`quick_check=ok`) but holds only **256
   memories** — a *different/older store* than the live 796. Restoring from them would
   silently **lose ~540 memories**. The `pmd-snapshot.timer` is capturing the wrong DB
   (a store-split of the class warned about in `.claude/rules/pmd-invariants.md` §1).

### This is recurring
Forensic sprawl in `/srv/brehon-fork/.project-memory/` (46 MB): `memory-malformed-backup.db`,
`memory-malformed-2026-05-21.db`, `memory-repaired.db`, `memory-repaired2.db`,
`memory-recovered.db`, `memory_repaired.db`, plus `recovery-20260613T222530Z/`. The store was
recovered as recently as **2026-06-13 22:25** and is malformed again by 2026-06-14. Root-cause
(why the live store keeps corrupting, and why the snapshot timer points at a 256-row store)
needs deliberate attention, not another one-off repair.

### Recommended human/advisor actions (NOT auto-applied)
1. Recover from the live store's own data, not the 256-row hourly snapshots: copy the live
   `memory.db` + `memory.db-wal` aside, `sqlite3 ... '.recover'` into a fresh DB (or
   checkpoint WAL → `REINDEX` / rebuild FTS5), verify count ≈ 796 + `quick_check=ok`, then
   atomically swap and restart the daemon so it drops the `(deleted)` inode.
2. Fix `pmd-snapshot.timer` so it snapshots the **live 796-row** store (the
   `PROJECT_MEMORY_DB=/srv/brehon-fork/.project-memory/memory.db` the daemon uses), not the
   256-row store — otherwise backups remain worthless.
3. Open a root-cause investigation into the recurring corruption + the file-swap-under-daemon
   pattern. Candidate lesson once understood.

## Steps that ran (daemon-executable)

| Step | Status | Result |
|---|---|---|
| 1. Memory review | ✅ | 883 memories (qa-result 592, pattern 235, …); 87 expired prunable, 0 superseded; no file/tag promotion candidates surfaced. |
| 1. Memory prune (not dry-run) | ✅ | **Pruned 87** expired memories → ~796 remain. (Ran before corruption was known; succeeded on the intact main table.) |
| 1c. Lesson frontmatter lint | ✅ | `scripts/brehon/lesson-frontmatter-lint.sh` RC=0 — all **223** lesson files have valid PMD-sync frontmatter. Clean. |
| 2. Promote candidates | ➖ | None surfaced by Step 1; nothing to promote. |
| 4. Git hygiene | ⚠️ partial | `git worktree prune` RC=0 (no stale worktrees; only canonical + this job-684). Branch deletion **deferred** — classifying terminal-status branches needs the Junior DB (junior MCP unavailable in this worker). 64 local branches: phase-* 37, ab-test 9, ab-cell 8, chore 2, junior 1. |
| 4b. Disk headroom | ⚠️ finding | `/` (only mount) at **89%** used, 25 GB free — ≥75% threshold tripped, just under the 90% surface-to-user line. Note: `.project-memory/` holds 46 MB of corruption-recovery `.db` sprawl (left in place — active-incident forensics, not for me to delete). |
| 6. Commit | ✅ | This report, `Review:`-prefixed (prevents cron cascade). |

## Steps DEGRADED by the PMD corruption (could not complete)

| Step | Why blocked |
|---|---|
| 2b. Lesson clustering | Requires `memory_search_hybrid` — search is down. (Also marked "TH only" in the skill.) |
| 3. Eval-metrics aggregation | Requires PMD search over `qa-result` — down. Partial signal from `memory_review`: qa-result=592 corpus; tags success=383, partial=104. Per-week success-rate/avg-score/top-root-cause aggregation deferred until search is restored. |
| 5. Summary memory_write | **Intentionally skipped** — writing into the corrupt FTS5 index risks worsening corruption. Summary lives in this committed report instead of a PMD `summary` memory. |
| 5. METRICS.md append | No top-level `METRICS.md` exists on the daemon side (lives laptop-side/TH). No row appended here. |

## Steps DEFERRED (laptop-only — wrong host)

| Step | Why |
|---|---|
| 1b. PMD embedding backfill | Must run laptop-side against the canonical DB + Ollama. **Moot until the live store is repaired** — backfilling a corrupt store is pointless/harmful. |
| 2d. MEMORY.md ACTIVE-line drift | Reads `C:/Users/barri/.claude/.../MEMORY.md` + `v1-roadmap.json` — laptop paths absent on daemon. |

## Step 2c — retro-harvest sweep (surfacing)

Wrote `.claude/harvest/2026-W24.md` (gitignored runtime journal). **37** retros committed in
the last 7 days (`git log`-derived — the mtime heuristic was unusable: a fresh worktree
checkout reset all 276 report mtimes to today, a false signal). **26** carry a proposal-bearing
section and form the manual-triage queue. No proposals auto-promoted; the
"already-promoted?" PMD cross-ref could not run (search down). Full STALE/LIVE/SUPERSEDED
triage deferred to the advisor with PMD search restored.

## Summary line
Pruned 87 | Promoted 0 | Lessons-frontmatter clean (223) | Branches cleaned 0 (deferred) |
Disk / 89% (finding) | **PMD live store CORRUPT — search DOWN, recurring, backups track wrong
store — ESCALATED, no repair attempted.**
