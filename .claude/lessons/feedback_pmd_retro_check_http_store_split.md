---
name: PMD retro-check hook reads local sqlite while MCP writes to HTTP server
description: retro-check.sh resolves DB via git-common-dir -> local sqlite; Junior MCP writes to HTTP server on laptop; two separate stores means hook enforcement is unreliable when they diverge.
type: feedback
---

# PMD retro-check hook reads local sqlite while MCP writes to HTTP server

## The gap

The PMD architecture evolved from a local sqlite store to an HTTP daemon after
pmd-invariants.md was written. retro-check.sh was not updated to match.

Old topology (what pmd-invariants.md described, pre-2026-05-30):
- Junior worker: memory_write_eval -> MCP (stdio) -> PROJECT_MEMORY_DB -> /srv/brehon-fork/.project-memory/memory.db
- retro-check.sh: git rev-parse --git-common-dir -> /srv/brehon-fork/.git -> /srv/brehon-fork/.project-memory/memory.db
- Both pointed at the same file. Pass.

Current topology (HTTP server, 2026-05-30+):
- Junior worker: memory_write_eval -> MCP (HTTP) -> http://100.104.171.26:11435/mcp -> laptop HTTP server -> laptop DB
- retro-check.sh: git rev-parse --git-common-dir -> /srv/brehon-fork/.git -> /srv/brehon-fork/.project-memory/memory.db (daemon-local)
- Two separate stores. The retro lands on the laptop; the hook reads the daemon. Fail.

## Evidence (2026-05-30 diagnostic probes)

Probe 6 (write-read round-trip) confirmed the split:
- memory_write_eval via HTTP MCP assigned ID 677 on the HTTP server.
- Daemon-local sqlite stayed at 577 rows -- ID 677 was absent immediately after write.
- Row count divergence: daemon-local 577 rows (latest: 2026-05-30 22:11:57);
  HTTP server had at least 677 rows (probe-6 created at 22:42:13).

## Why the daemon-local sqlite has rows at all

The daemon-local /srv/brehon-fork/.project-memory/memory.db (577 rows as of 2026-05-30)
is likely populated by backfill script runs against a downloaded snapshot, or historical
writes from before the HTTP topology migration. It is NOT a live sync of the HTTP server.
Writes from Junior workers (via HTTP MCP) do NOT appear in it.

## CORRECTION (2026-05-31 forensic re-investigation) — it IS a lagged replica

The "NOT a live sync" claim above was **incomplete**. Direct forensics (SSH probes,
`feedback_verify_automated_reviewer_claims_against_compiler.md` discipline) found the
daemon-local sqlite carries rows with the **same IDs as the laptop HTTP-server canonical
DB** (e.g. ID 683 "PMD validation probe" is a row written THIS session via the HTTP MCP;
18 `junior/*`-source retro rows present, IDs 680–689 matching the laptop sequence). It is
a **downstream replica with sync latency**, not an independent store.

The actual writers, confirmed:
- **`pmd-snapshot.timer`** (systemd, hourly) — `sqlite3 .backup` copies the canonical DB
  down to the daemon (+ `/srv/backups/pmd/brehon-fork/{hourly,daily,weekly}/`). This is
  what populates the daemon-local file. **READ-only w.r.t. the source.**
- **`memory-backfill.timer`** (systemd, every 5 min) — fills `memory_vectors` (embeddings)
  for the daemon-local file via Ollama. Writes vectors only, never `memories` rows.
  Currently a no-op (all rows vectorized).
- **No independent `memory_write` writer exists** — `ps`, systemd env, and the daemon
  `.mcp.json` (HTTP-only) all confirm nothing inserts `memories` rows locally.

**Why Probe 6 saw the row "absent immediately":** the hourly snapshot simply had not run
in the seconds between the HTTP write and the probe's local read. The absence was
**snapshot latency**, not permanent divergence. A re-check after the next snapshot would
have found the row. The Option-A HTTP-first fix is still correct (it eliminates the
latency window entirely), but the old sqlite-only hook was **lagged, not broken** — it
finds retros once the snapshot syncs them down, and it fails *open* during the lag window
(3× retry then allow), so no work is lost.

**Deployment status (2026-05-31):** the Option-A fix (`d5df9df4b`) is on `governance-v0`
but the daemon checkout sits on `phase-v1-quality-r3` (a lane branch cut before the fix),
so the daemon's *running* hook is still the sqlite3-only version. This is **normal
multi-lane lag, not a deploy failure** — the fix rides onto the daemon's branch at the
lane's next trunk-forward merge (bm-merge cadence). Force-deploying out-of-band is NOT
warranted for a self-healing, fail-open latency issue. Per the daemon-local-first
finalize discipline (`feedback_finalize_merge_where_to_look_first.md`).

## The fix (Option A -- HTTP MCP session protocol)

retro-check.sh now queries the HTTP server via the MCP session protocol when .mcp.json
configures project-memory as type=http:

  Step 1: POST /mcp (initialize) -> response header mcp-session-id
  Step 2: POST /mcp (tools/call -> memory_search) with Mcp-Session-Id header
  Step 3: Python3 parses SSE JSON response, filters by branch + time window

Token is extracted from .mcp.json headers.Authorization (Bearer stripped).
The local sqlite remains as a fallback for environments where the HTTP server is
absent (stdio MCP, offline dev, CI without Tailscale).

**Why:** - Option B (accept local sqlite as live sync) rejected because Probe 6 showed the row
never arrived (not stale, simply absent). A mtime check would give a false pass.
  - Option C (new env vars) not needed -- Option A reads from .mcp.json directly.

## The MCP HTTP/SSE protocol detail

The project-memory MCP server (v1.0.0, project-memory-mcp) uses HTTP/SSE transport.
Each POST is NOT stateless -- a session must be initialized first:
- POST /mcp without Accept: 400 "Not Acceptable"
- POST /mcp without session for tools/call: 400 "missing session id and not initialize"
- Correct flow: initialize (gets mcp-session-id header) then tools/call with that header.
  Both steps need: Accept: application/json, text/event-stream
- No REST API exists (/memories /search /health all return 404).

## Relationship to feedback_pmd_cross_lane_canonical_db.md

That lesson said "never edit the hook -- fix the MCP side." That guidance was correct
for the stdio-MCP topology where the hook's git-common-dir resolution was accurate.
With the HTTP topology, there is no single file to pin the MCP to -- the store is the
HTTP server's internal DB on the laptop. The hook MUST be updated (not the MCP config)
because the MCP is correctly configured. The prior "never edit the hook" guidance does
NOT apply to the HTTP topology.

## Lesson sync under HTTP topology (added 2026-05-31)

The same store-split breaks `sync-lessons-to-pmd.sh`: it writes directly to a
SQLite file via `PROJECT_MEMORY_DB`, which under HTTP is the daemon-local store
the HTTP server never reads. Lessons authored via Write/Edit therefore never
reached the HTTP PMD. Fix: `.claude/hooks/lesson-pmd-sync.sh` (PostToolUse,
Edit|Write matcher) auto-calls `memory_write` (or `memory_update` if a row for
that file_path already exists, via `memory_get_file_context`) against the HTTP
server whenever a `feedback_*.md` / `reference_*.md` lesson with valid
frontmatter is saved.

Two traps found building it (both verified by live test 2026-05-31):
1. **`${VAR@Q}` is NOT valid python.** Bash parameter-transform quoting emits
   `'it'\''s'` which breaks the python payload builder on any lesson containing
   an apostrophe. Pass values through the ENVIRONMENT (`os.environ`), never
   interpolate shell values into python source.
2. **`memory_search` (FTS5) misses filename-only queries** — querying the
   basename returned "No memories found", so the idempotency check failed and
   spawned duplicates. Use `memory_get_file_context` (looks up BY file_path)
   for the dedup key, pick the lowest id if duplicates exist so re-edits
   converge on one canonical row.

## See also

- feedback_pmd_cross_lane_canonical_db.md -- canonical-path invariant predecessor
- feedback_pmd_two_memory_systems_distinction.md -- System 1 vs System 2 (HTTP server IS System 2)
- .claude/hooks/retro-check.sh v4 -- the updated hook implementing Option A
- .claude/hooks/lesson-pmd-sync.sh -- the lesson auto-sync hook (this section)
- .claude/rules/pmd-invariants.md S1 -- updated with HTTP topology subsection
