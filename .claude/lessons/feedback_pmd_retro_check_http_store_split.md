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

## See also

- feedback_pmd_cross_lane_canonical_db.md -- canonical-path invariant predecessor
- feedback_pmd_two_memory_systems_distinction.md -- System 1 vs System 2 (HTTP server IS System 2)
- .claude/hooks/retro-check.sh v4 -- the updated hook implementing Option A
- .claude/rules/pmd-invariants.md S1 -- updated with HTTP topology subsection
