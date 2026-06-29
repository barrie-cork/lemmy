---
phase: ops-maintenance (not a sub-phase)
authored: 2026-06-29
authored_by: advisor (homeserver CWD session — PMD repair executed there, follow-ups land here)
trunk_branch: governance-v0
purpose: >
  Hand the three brehon-fork-seat follow-ups from the 2026-06-29 PMD-corruption
  remediation to the next advisor session opened in this CWD. The DB is already
  repaired (done from the homeserver seat); what remains needs THIS repo's
  advisor context + MCP wiring. Read the RESUME block first.
---

# ⏩ RESUME 2026-06-29 — read this first

The brehon-fork PMD (`/srv/brehon-fork/.project-memory/memory.db`) was **corrupt
and is now repaired** (done 2026-06-29 from the homeserver CWD, not here). All
1012 memories / 810 vectors / 86 read_events salvaged, `integrity_check=ok`,
daemon restarted and verified healthy on the rebuilt DB. Full root cause +
repair recipe is **PMD #378** (homeserver/centralized DB) — search
`memory_search_hybrid query:"brehon-fork PMD corruption WAL"` if you want it.

**You do NOT need to repair anything.** Three follow-ups remain, all of which
need the brehon-fork seat (this CWD) and could not be done from the homeserver
session:

## 1. Branch consolidation `governance-v0 → main` (user request, sequence-sensitive)

Deferred here deliberately — it's an advisor action that wants this repo's
context, and it's sequence-sensitive. Order **matters**:

1. Confirm no Junior task is mid-flight on `governance-v0`
   (`mcp__junior-brehon__list_tasks` — all terminal).
2. **Stop** `junior@brehon-fork` (`ssh homeserver 'sudo systemctl stop junior@brehon-fork.service'`)
   — a branch move under a live worker orphans its worktree.
3. Do the move (fast-forward `main` to `governance-v0`, push, update origin HEAD).
4. **Restart** the daemon (`sudo systemctl start junior@brehon-fork.service`)
   and confirm `integrity_check=ok` + `is-active`.

The DB repair is already done, so unlike the original handover you only stop the
daemon for the branch move itself, not for a recovery.

## 2. `/weekly-review` from this checkout

The W26 daemon-side review (Junior job-775) **ran and committed to git but could
not write the `Weekly review %` PMD row** because the DB was corrupt at the time
— that's why the `review-due` dashboard shows brehon-fork ~29d stale while git
shows recent W26 health-check merges. Now that the DB is healthy, run
`/weekly-review` from here to write the missing heartbeat row and reconcile the
dashboard. (Open question the original handover flagged: decide whether the
daemon review path should write that row itself — until it does, the daemon
review and the skill review disagree.)

## 3. MCP durability fix — THE recurrence will continue without this

The repair is a **band-aid**. Root cause (PMD #378): `project-memory-mcp` opens
the WAL with better-sqlite3 default `synchronous=NORMAL` and sets **no
`busy_timeout`**. The MCP has a correct SIGTERM checkpoint handler, but the MCP
runs as a child of the Junior worker (`claude -p`); when Junior's inactivity
watchdog **SIGKILLs** a worker mid-PMD-write (~6 min no-stdout, exit 143), the
MCP child dies with **no SIGTERM**, abandoning an uncheckpointed WAL →
torn b-tree page (Tree 9) on the next open. Matches the monthly cadence (May 12,
May 21, Jun 13, Jun 29).

**Fix** (in `project-memory-mcp/db.js` `openDb`, alongside the WAL pragma):
```
PRAGMA synchronous = FULL;     -- load-bearing: abandoned WAL stays recoverable, not torn
PRAGMA busy_timeout = 5000;    -- concurrent-open safety
```
The MCP source lives at `/home/barrie/MCPs/project-memory-mcp/` on the EliteDesk
and is **NOT checked out on homeserver** → obey the hotfix rule: fix in a local
MCP checkout → commit → push → pull on the box → `npm run build` → restart the
daemons. This is the one item that stops the corruption recurring.

---

## Repair facts worth keeping (so the next repair is faster)

- **`.recover` is DEAD on the EliteDesk** — its `sqlite3` CLI (3.45.1) is compiled
  **without `sqlite_dbpage`**, so `.recover` bails after ~7 lines
  (`no such table: sqlite_dbpage`). Use **`.dump | sqlite3 fresh.db`** instead.
- **Checkpoint-then-recover fails on a malformed DB** (`database disk image is
  malformed`) — you can't fold a WAL into a corrupt main file. A main-only
  `.dump` was complete here (newest row Jun 26; the 4.1 MB Jun 28 WAL held
  nothing newer that survived).
- **Backups preserved** at `/srv/brehon-fork/.project-memory/archive/` — all real
  date-stamped backups + the 2026-06-29 corrupt/pre-recover snapshots. The live
  dir now holds only `memory.db` + one `.good-*.bak` + `archive/`. **Never delete
  the real backups** (repo rule).

## What was NOT changed

No branch was moved. `governance-v0` is untouched and in sync with origin. The
daemon is **running** (restarted 2026-06-29 11:35Z). Nothing here is mid-flight.
