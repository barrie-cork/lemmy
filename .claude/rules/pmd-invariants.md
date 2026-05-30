# PMD invariants

These five statements are non-negotiable system invariants, not heuristics. They protect the
PMD (Project Memory Database) substrate the entire Recursive Learning System depends on.
Scattering them as lesson files conflates invariants with recurrence-based heuristics; this
rule consolidates them. The lesson files stay as evidence trail. Per
`docs/research/brehon-rls-pmd-review.md` §4.8.

## 1. Canonical PMD path (absolute, cross-lane)

Every worktree's `.mcp.json` `PROJECT_MEMORY_DB` MUST be
`C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` — never relative, never
per-lane. Detection: `bash .claude/hooks/pmd-canonical-guard.sh` (v1-rls-r1 ships).

**Why this is non-negotiable:** The Stop hook `retro-check.sh` resolves the PMD via
`git rev-parse --git-common-dir`, which from any worktree always points at the canonical
`brehon-fork/.git` — so the hook reads `brehon-fork/.project-memory/memory.db` regardless
of where the MCP writes. During v1-ship-1 (2026-05-18), the `brehon-fork-ship-1` lane ran
its project-memory MCP with a relative `PROJECT_MEMORY_DB` resolved against the per-worktree
`PROJECT_ROOT`, so every `memory_write_eval` landed in a lane-local DB the hook could never
see. All 21 v1-ship-1 retros were stranded; ~27+ false Stop-hook blocks accumulated across
the entire phase before the path mismatch was identified. A one-character fix (`PROJECT_MEMORY_DB`
to the absolute canonical path) resolves a class of failure that compounds silently for weeks.

**How to apply:** `bash .claude/hooks/pmd-canonical-guard.sh` (shipped in v1-rls-r1 Task 3) runs
at every SessionStart and emits a loud WARN naming both paths if they diverge — lane drift surfaces
within seconds of session-start, not after a phase of stranding.

**See also:**

- `.claude/lessons/feedback_pmd_cross_lane_canonical_db.md` — canonical-path invariant source,
  v1-ship-1 incident detail, and the diagnosis recipe.
- `.claude/rules/multi-lane-worktree.md` §"PMD is cross-lane shared, NOT per-lane isolated"

**Current topology (HTTP server, 2026-05-30+):** The PMD MCP server is now an HTTP
daemon (`http://localhost:11435/mcp` on the laptop; Tailscale-reachable at
`http://100.104.171.26:11435/mcp` from the daemon). The `PROJECT_MEMORY_DB` /
`PROJECT_ROOT` env-var mechanism is superseded — the HTTP server manages the canonical
DB server-side. `pmd-canonical-guard.sh` is now a no-op for env-var path checking but
its SessionStart invocation is harmless. See `feedback_pmd_retro_check_http_store_split.md`
for the retro-check.sh enforcement gap this topology change created.

## 2. Two systems, one source of truth

System 1 (auto-loaded markdown under `~/.claude/projects/.../memory/`) and System 2
(queryable SQLite-vec DB at the canonical path) are NOT interchangeable; never write retro
content directly to System 1; never assume System 2 is loaded at SessionStart.

**Why this is non-negotiable:** The two systems share the name "project memory" and both
surface through the same MCP server, which makes them easy to conflate. During the
advisor-CWD migration (2026-04-30), a Step 4 "migrate 200 memory files" correctly moved the
System 1 `.md` files but included no step for the System 2 SQLite DB. `memory_search_hybrid`
returned zero results despite the `.md` files being present, because the DB was not seeded —
the systems look like one but are populated independently and have different failure modes.
Any runbook, audit, or session setup that treats them as one system misses half the required
work.

**How to apply:** `enforcement: PENDING — no automated check ships in v1-rls-r1`; apply the
two-system mental model manually: when touching "project memory", address System 1 (`ls
~/.claude/projects/.../memory/*.md | wc -l`) and System 2 (`memory_search_hybrid(query:
"test", limit: 1)` to confirm DB is seeded) explicitly.

**See also:**

- `.claude/lessons/feedback_pmd_two_memory_systems_distinction.md` — two-systems invariant
  source, migration incident detail, and per-system diagnostic checks.

## 3. No write-time embedding (yet — see item 4.2-spec)

Every `memory_write_eval` writes the FTS5 row but NOT the vector; `backfill.js` must run
between write and the next hybrid search OR the search degrades silently to FTS5-only.
Mitigations: session-retro inline backfill (laptop-only); weekly-review §1b safety-net
(Junior + safety net for missed). The `mcp-write-time-embedding.md` spec ships in v1-rls-r1;
the patch ships separately.

**Why this is non-negotiable:** Source inspection of `dist/index.js` (2026-05-16) confirmed
that `memory_write`, `memory_write_eval`, `memory_promote_to_file`, and `memory_update`
contain zero `embed` / `ollama` / `memory_vectors` / `generateEmbedding` references — all
embedding happens exclusively in the separate `dist/scripts/backfill.js`. Every PMD write is
therefore immediately FTS5-searchable but NOT semantically searchable until backfill runs.
`memory_search_hybrid` silently degrades to FTS5-only for unembedded rows with only a single
stderr line as signal. The +62.7% Recall@10 hybrid advantage is absent during the window
between write and backfill, which for Junior daemon-side writes can be up to ~6.5 days.
Eight PMD memories were found unembedded at one audit point — five of them from sessions
that simply did not know the backfill step existed, because the gap is silent by design.

**How to apply:** `enforcement: PENDING — see .claude/PRPs/specs/mcp-write-time-embedding.md +
future MCPs/project-memory-mcp PR`; until the write-time embedding patch ships, run
`backfill.js` after every interactive laptop `memory_write_eval`; the weekly-review Step 1b
safety-net covers the Junior daemon-side gap.

**See also:**

- `.claude/lessons/feedback_pmd_backfill_after_write.md` — no-write-time-embedding invariant
  source, the `dist/index.js` inspection, and the two enforcement points (session-retro Step
  5.5 + weekly-review Step 1b).
- `.claude/PRPs/specs/mcp-write-time-embedding.md` — the patch contract (v1-rls-r1 ships
  the spec; the actual `dist/index.js` patch is a separate `MCPs/project-memory-mcp` PR).

## 4. LESSON-trailer discipline

Every learning observation either fires a `LESSON:` trailer (in commit body) OR a `kind:
"log"` DQ entry (mid-task push, resolved-immediately). Never both, never neither for a
durable observation.

**Why this is non-negotiable:** Junior subagents run on the EliteDesk in worktrees and
cannot write directly to laptop-side PMD — different machine, no SSH-back path. Without a
discipline, lessons learned during a Junior task have nowhere to land: commit messages capture
the *what* but not retrospective framing; DQ resolved entries capture *answers* but not
generalisable signals; worktrees get reaped after task completion. A phase with 10–30
promotable lessons that were never signalled produces a PMD that looks current but silently
misses the most recent sub-phase's hard-won knowledge. The LESSON-trailer convention creates
a machine-readable signal the advisor harvests at retro time; the `kind: "log"` DQ path
provides mid-task durability for time-sensitive observations.

**How to apply:** LESSON-trailer convention per `feedback_junior_pmd_write_convention.md` is
the shipped enforcement — end commit-message bodies with `LESSON: <one-line observation>` when
a Junior subagent discovers a durable pattern; advisor harvests at retro time and promotes to
PMD + `.claude/lessons/`.

**See also:**

- `.claude/lessons/feedback_junior_pmd_write_convention.md` — LESSON-trailer invariant source,
  the two-part convention (Junior `LESSON:` trailer + advisor harvest), and the retro file
  "Lessons promoted this phase" section.

## 5. SessionStart canonical-PMD guard (post-v1-rls-r1)

The tracked `pmd-canonical-guard.sh` hook runs at every session start; lane drift surfaces
as a loud WARN within seconds of session-start, not after a phase of stranding. Per
`feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`.

**Why this is non-negotiable:** The `.mcp.json.example` template has carried a
`_comment_pmd_cross_lane` guard key and the correct absolute `PROJECT_MEMORY_DB` since
v1-ship-1 but nothing *checks* that a given lane's actual `.mcp.json` matches the canonical
path. A lane bootstrapped from an older template, or with a stale relative path, fails with
zero feedback until the symptom compounds — v1-ship-1 accumulated ~27+ false Stop-hook blocks
before the mismatch was found. A documentary guard that is never checked is equivalent to no
guard: the knowledge exists; the enforcement does not; the footgun re-fires silently. The
SessionStart hook converts a paper invariant into a loud signal at the cheapest possible
moment — before the first retro write of the session, not after an entire phase of stranding.

**How to apply:** `bash .claude/hooks/pmd-canonical-guard.sh` is shipped in v1-rls-r1 Task 3
and wired via a `SessionStart` entry in the per-worktree `settings.local.json` (the wiring
must be part of the lane-bootstrap checklist per
`feedback_settings_local_json_worktree_bootstrap.md` — it does not propagate automatically).

**See also:**

- `.claude/lessons/feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` —
  SessionStart-guard invariant source, the mechanism spec, and the WARN-vs-FAIL rationale.
- `.claude/lessons/feedback_pmd_cross_lane_canonical_db.md` — the canonical-path invariant
  this guard enforces + the diagnosis recipe it automates.

## See also

- `.claude/rules/multi-lane-worktree.md` §"PMD is cross-lane shared, NOT per-lane isolated"
- `.claude/rules/pmd-search-strategy.md` (hybrid-search status and FTS5 fallback)
- `.claude/hooks/pmd-canonical-guard.sh` (v1-rls-r1 ships per Task 3)
- `docs/research/brehon-rls-pmd-review.md` §3 + §4 + §5.3
