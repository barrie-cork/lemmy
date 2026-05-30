---
role: impl-task
phase: pmd-retro-hook-fix
n: 1
authored: 2026-05-30
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: governance-v0
---

# [role:impl-task] pmd-retro-hook-fix task 1 — validate PMD end-to-end + fix retro-check.sh HTTP-store split

## 1. Role + dispatch line

```
[role:impl-task] pmd-retro-hook-fix task-1 — PMD validation + retro-check HTTP fix — see .claude/PRPs/briefs/pmd-retro-hook-fix-impl-1.md
```

This is a **diagnostic + fix task**. It has two phases: (A) validate the full PMD write-read-search
pipeline end-to-end and report results, then (B) fix the `retro-check.sh` enforcement gap where
the hook reads a local sqlite file while Junior MCP writes to an HTTP server.

**Branch discipline:** this task runs on `governance-v0` directly (no phase branch) — it is
infrastructure meta-work with no `crates/` edits, consistent with the `phase-branch.md`
"Direct on governance-v0" policy for scripts/hooks/lessons.

## 2. Scope

### What to produce

1. **Phase A — diagnostic report** (emitted to task output, no commit required):
   A structured pass/fail table for each probe (§5). If any probe fails, raise a
   `kind: "blocker"` DQ entry for the advisor before starting Phase B.

2. **Phase B — fix `retro-check.sh`** (one commit):
   - Amend `retro-check.sh` to validate against the HTTP PMD server when the MCP endpoint
     is reachable, falling back to local sqlite only when the HTTP server is not reachable.
   - Write `feedback_pmd_retro_check_http_store_split.md` (new lesson).
   - Update `pmd-invariants.md` §1 to note the HTTP-server topology change and that the
     `PROJECT_MEMORY_DB` env-var guard is now superseded.

3. **One commit** (Phase B only — Phase A is report only):
   ```
   fix(hooks): retro-check.sh validates against HTTP PMD; update pmd-invariants §1 + lesson
   ```

### Explicit boundaries

- **No edits to `crates/`**, `migrations/`, or `tests/`.
- **No changes to the MCP server code** (`C:/Users/barri/Developer/MCPs/project-memory-mcp/`
  or `/srv/MCPs/` equivalents) — fix only the hook that reads PMD.
- **No architectural changes** to the PMD server itself. The fix is client-side (hook-side).
- **Do NOT remove the local sqlite fallback** — keep it as the fallback for environments
  where the HTTP server is absent (e.g. offline dev, CI).

## 3. Required reading

In this order:

1. **`.claude/rules/pmd-invariants.md`** — the five invariants; §1 (canonical path) is the
   target of the §2 Scope update. Read the full file before editing.

2. **`.claude/hooks/retro-check.sh`** — the current hook implementation. Read the full file.
   Key section: the `DB=""` resolution block (lines ~28–44) — this is what must change.

3. **`.claude/lessons/feedback_pmd_cross_lane_canonical_db.md`** — the v1-ship-1 stranding
   incident; documents the two-separate-stores failure mode the fix addresses.

4. **`.claude/lessons/feedback_pmd_two_memory_systems_distinction.md`** — System 1 (auto-load
   `.md` files) vs System 2 (queryable SQLite). The HTTP server IS System 2 — these must
   not be conflated in the updated invariant text.

5. **`.claude/lessons/feedback_pmd_backfill_after_write.md`** — the no-write-time-embedding
   invariant (PMD invariant §3). The fix must not break the backfill path.

6. **`.claude/rules/advisor-orchestrator.md` §5.5** — retro-bypass observability; the
   `emit_retro_bypass_log()` function in the hook must be preserved intact.

7. **`pmd-search-strategy.md`** — current PMD status section documents the HTTP topology
   change. Read §"brehon-fork PMD status" for current operational context.

### Context the advisor established (read before starting)

The PMD architecture changed since `pmd-invariants.md` was written:

**Old (what the rule describes):** MCP server reads `PROJECT_MEMORY_DB` env-var → local
sqlite file at `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`.

**Current (what actually runs):**
- Laptop: node PMD HTTP server listening on `localhost:11435` (PID 5900 on laptop).
- `.mcp.json` on both laptop and daemon point to HTTP endpoints:
  - Laptop: `http://localhost:11435/mcp`
  - Daemon: `http://100.104.171.26:11435/mcp` (Tailscale IP of laptop)
- The HTTP server owns the canonical DB server-side on the laptop.
- The daemon has a **separate local** `/srv/brehon-fork/.project-memory/memory.db` (577 rows)
  — this is NOT the same file the HTTP server reads. It is either a stale fork or populated
  by a different path.

**The gap:** `retro-check.sh` on the daemon resolves DB via:
```bash
GIT_COMMON=$(git rev-parse --git-common-dir)
MAIN_REPO=$(dirname "$GIT_COMMON")
DB="${MAIN_REPO}/.project-memory/memory.db"   # → /srv/brehon-fork/.project-memory/memory.db
sqlite3 "$DB" "SELECT COUNT(*) FROM memories WHERE ..."
```
But Junior workers write retros via `memory_write_eval` → HTTP MCP → laptop HTTP server.
The hook reads the daemon-local sqlite; the retro is written to the laptop HTTP server.
**These are two different stores.** The hook sees the daemon-local rows, not the rows the
worker just wrote. Whether the hook currently passes or fails depends on whether the
daemon-local file happens to have a recent-enough row from a previous run — it is unreliable.

**Additional confirmed facts:**
- `curl -m 3 http://100.104.171.26:11435/mcp` from the daemon returns "unauthorized" (bearer
  token required). The daemon `.mcp.json` supplies `Authorization: Bearer <token>` in its
  MCP client headers, so the MCP client can write; but `curl` without the header cannot.
- The daemon-local sqlite has 577 rows with rows as recent as 2026-05-30 — so the file IS
  being written to by *something*, but not by the Junior workers' `memory_write_eval` calls.
  It may be a legacy path or populated by the backfill script run against a downloaded copy.

## 4. Constraints

### Phase A — probes are diagnostic only

Run every probe in §5, capture output, emit a structured report. If a probe fails, raise a
`kind: "blocker"` DQ entry (id from `bash scripts/brehon/dq-v3-new-entry.sh`) and stop —
do NOT proceed to Phase B until advisor resolves the blocker.

### Phase B — fix shape

The fix must handle three cases in `retro-check.sh`:

**Case 1 (preferred — HTTP endpoint reachable):**
The daemon `.mcp.json` has an HTTP PMD endpoint. Validate the retro was written by querying
the HTTP server's REST API (if it exposes one) or by a `memory_search` MCP call that checks
for the branch-scoped retro row. If the MCP tool is available to the hook, prefer it.

**Case 2 (fallback — HTTP not reachable, local sqlite exists):**
Current behaviour — `sqlite3` query against local file. Keep as-is.

**Case 3 (neither reachable):**
Fail open (exit 0, emit retro-bypass log). Unchanged from current.

**Implementation note:** the hook runs as a bare bash script with no access to MCP tools
directly. The cleanest approach that avoids adding a curl dependency with auth-header
complexity is:

Option A — query the HTTP server's `/memories` REST endpoint with the bearer token extracted
from `.mcp.json` (one `jq` call + one `curl`).

Option B — accept that on the daemon, the local sqlite is a sync of the HTTP server's DB
(verify this in Phase A Probe 4), and keep the sqlite path but verify it is current (not
stale by more than 24h via `stat` mtime check).

Option C — add a new env var `PMD_HTTP_URL` + `PMD_HTTP_TOKEN` that the daemon's
`settings.json` pre-populates (via a SessionStart hook or daemon env injection), and have
`retro-check.sh` prefer curl to that URL when set.

**Advisor directive: pick the option Phase A Probe 4 supports.** If Probe 4 shows the
daemon-local sqlite is a live sync of the HTTP server (rows match within seconds), Option B
is sufficient and lowest-risk. If they diverge, implement Option A or C. Document the choice
in the commit body and the new lesson file.

### Lesson file shape

`feedback_pmd_retro_check_http_store_split.md` must have valid PMD sync frontmatter:

```yaml
---
name: PMD retro-check hook reads local sqlite while MCP writes to HTTP server
description: retro-check.sh resolves DB via git-common-dir → local sqlite; Junior MCP writes to HTTP server on laptop; two separate stores means hook enforcement is unreliable when they diverge.
type: feedback
---
```

Body: incident description, the two-store topology diagram (who writes where), the three
option fix shapes, which option was chosen and why, and a `See also` pointing to
`feedback_pmd_cross_lane_canonical_db.md` (the canonical-path invariant predecessor).

### pmd-invariants.md §1 update

Add a **"Current topology (HTTP server, post-v1-rls-r1)"** sub-section AFTER the existing
`PROJECT_MEMORY_DB` env-var text. Keep the existing text (it describes the old topology for
historical context). The new subsection:

```markdown
**Current topology (HTTP server, 2026-05-30+):** The PMD MCP server is now an HTTP
daemon (`http://localhost:11435/mcp` on the laptop; Tailscale-reachable at
`http://100.104.171.26:11435/mcp` from the daemon). The `PROJECT_MEMORY_DB` /
`PROJECT_ROOT` env-var mechanism is superseded — the HTTP server manages the canonical
DB server-side. `pmd-canonical-guard.sh` is now a no-op for env-var path checking but
its SessionStart invocation is harmless. See `feedback_pmd_retro_check_http_store_split.md`
for the retro-check.sh enforcement gap this topology change created.
```

Do NOT remove the `PROJECT_MEMORY_DB` text — it remains accurate for environments
running the old stdio MCP server.

## 5. Validation probes (Phase A — run all before Phase B)

Run each probe, capture stdout + exit code. Emit "PROBE N: PASS" or "PROBE N: FAIL — <reason>".

**Probe 1 — HTTP server reachability from daemon (Tailscale):**
```bash
# From the daemon (SSH):
ssh homeserver "curl -s -o /dev/null -w '%{http_code}' \
  -H 'Authorization: Bearer 4964f33f2de863cce274a6da1b69e1d1022e9d2b8ac7cf8e' \
  -m 5 http://100.104.171.26:11435/health 2>/dev/null || echo 'CURL_FAIL'"
```
EXPECT: HTTP 200 (or similar success code). FAIL if curl errors or returns 4xx/5xx.
Note: if the server has no `/health` endpoint, try `/mcp` — a 405 or 200 is still
"reachable"; a connection refused is FAIL.

**Probe 2 — Daemon-local sqlite is queryable:**
```bash
ssh homeserver "sqlite3 /srv/brehon-fork/.project-memory/memory.db \
  'SELECT COUNT(*) FROM memories;' 2>/dev/null || echo 'SQLITE_FAIL'"
```
EXPECT: a non-zero integer (we know it has 577 rows). FAIL if sqlite3 errors.

**Probe 3 — Most recent daemon-local row vs most recent HTTP-server row:**
```bash
# Daemon-local:
ssh homeserver "sqlite3 /srv/brehon-fork/.project-memory/memory.db \
  'SELECT title, created_at FROM memories ORDER BY created_at DESC LIMIT 1;'"
# HTTP server (via advisor MCP — call memory_search_hybrid with limit:1 and no filter):
# Use: memory_search_hybrid(query: "Task retro", limit: 1)
# Compare the two timestamps.
```
EXPECT: timestamps within 24h of each other → Option B is viable.
FAIL (diverge >24h): Option A or C required.

**Probe 4 — Junior worker retro survives hook check on daemon:**
Reproduce the failure: from the daemon, simulate what the hook does:
```bash
ssh homeserver "cd /srv/brehon-fork && BRANCH='junior/test-pmd-probe-branch' \
  sqlite3 .project-memory/memory.db \
  \"SELECT COUNT(*) FROM memories \
    WHERE memory_type='qa-result' \
    AND title LIKE 'Task retro:%' \
    AND (source_ref='\$BRANCH' OR branch='\$BRANCH') \
    AND created_at >= datetime('now', '-30 minutes');\""
```
EXPECT: 0 (no matching row for a fake branch) — this confirms the hook would correctly
BLOCK a task that hasn't written a retro. If it returns 0, probe PASS (hook logic correct).
If it returns >0 unexpectedly, investigate.

**Probe 5 — retro-check.sh DB resolution path on daemon:**
```bash
ssh homeserver "cd /srv/brehon-fork && \
  GIT_COMMON=\$(git rev-parse --git-common-dir 2>/dev/null) && \
  echo \"git-common-dir: \$GIT_COMMON\" && \
  MAIN_REPO=\$(dirname \"\$GIT_COMMON\") && \
  DB=\"\${MAIN_REPO}/.project-memory/memory.db\" && \
  echo \"Resolved DB: \$DB\" && \
  ls -la \"\$DB\" 2>/dev/null || echo 'DB file absent'"
```
EXPECT: resolves to `/srv/brehon-fork/.project-memory/memory.db` and file exists.
This confirms the current hook reads the daemon-local file (not the HTTP server).

**Probe 6 — Memory write-read round-trip via HTTP MCP:**
Use the MCP tool (available in this task via the daemon's `.mcp.json` HTTP client):
```
memory_write(
  memory_type: "qa-result",
  title: "Task retro: pmd-retro-hook-fix probe-6 round-trip test",
  score: 0.70,
  tags: "probe,pmd-retro-hook-fix",
  source_ref: "governance-v0",
  content: "SCORE: 0.70\nCONFIDENCE: 0.9\nGoal achieved: yes\nTests: none\nClean execution: yes\nSummary: probe-6 round-trip write"
)
```
Then immediately query the daemon-local sqlite to check if the row appeared:
```bash
ssh homeserver "sqlite3 /srv/brehon-fork/.project-memory/memory.db \
  \"SELECT title, created_at FROM memories \
    WHERE title LIKE '%probe-6%' ORDER BY created_at DESC LIMIT 1;\""
```
EXPECT: if the row appears in the local sqlite within a few seconds → local sqlite IS
a live sync of the HTTP server (Option B fix is viable, no code change needed beyond
hook logic validation).
FAIL (row absent after 10 seconds): local sqlite is NOT in sync → Option A or C required.

**Probe 7 — backfill status (embedding coverage):**
```bash
ssh homeserver "sqlite3 /srv/brehon-fork/.project-memory/memory.db \
  'SELECT COUNT(*) FROM memory_vectors;' 2>/dev/null || echo 'NO_VECTORS_TABLE'"
# Also run from advisor MCP:
# memory_search_hybrid(query: "Task retro pmd probe", limit: 3) — check if Probe 6 row is findable
```
EXPECT: `memory_vectors` table exists with non-zero rows. If absent or zero, embeddings
are not populated and `memory_search_hybrid` is degrading to FTS5-only.

## 6. Expected output (return to advisor)

```
## pmd-retro-hook-fix task-1 complete

### Phase A — Probe results
| Probe | Result | Notes |
|-------|--------|-------|
| 1 — HTTP reachability | PASS/FAIL | <status code or error> |
| 2 — Daemon sqlite queryable | PASS/FAIL | <row count> |
| 3 — Timestamp delta (local vs HTTP) | PASS/FAIL | <delta in hours> |
| 4 — Hook logic correct (fake branch = 0 rows) | PASS | (expect always PASS) |
| 5 — DB resolution path | PASS | <resolved path> |
| 6 — Write-read round-trip | PASS/FAIL | <appeared in local sqlite: yes/no, latency> |
| 7 — backfill / vectors | PASS/FAIL | <vector count> |

### Phase B — Fix applied (if no Phase A blockers)
**Commit:** <sha>
**Option chosen:** <A / B / C — reason>
**Files changed:**
  - `.claude/hooks/retro-check.sh` — <description of change>
  - `.claude/lessons/feedback_pmd_retro_check_http_store_split.md` — new lesson
  - `.claude/rules/pmd-invariants.md` — §1 HTTP topology subsection added
**Validation:** hook smoke-tested with a probe-6-style write + hook invocation
**Remaining concern (if any):** <any gap the fix does not fully close>
```

Raise a `kind: "blocker"` DQ entry if Phase A shows blockers that change the fix shape
before Phase B begins.
