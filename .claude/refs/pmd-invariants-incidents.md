# PMD invariants — incident narratives and rationale

Extracted from `.claude/rules/pmd-invariants.md` per `feedback_rule_narrative_to_refs_at_author_time.md`. Rule statements remain in the rule file; narrative and FP/FN taxonomy lives here.

## Invariant 1: Canonical PMD path

**Why non-negotiable:** The Stop hook `retro-check.sh` resolves the PMD via `git rev-parse --git-common-dir`, which from any worktree always points at the canonical `brehon-fork/.git` — so the hook reads `brehon-fork/.project-memory/memory.db` regardless of where the MCP writes. During v1-ship-1 (2026-05-18), the `brehon-fork-ship-1` lane ran its project-memory MCP with a relative `PROJECT_MEMORY_DB` resolved against the per-worktree `PROJECT_ROOT`, so every `memory_write_eval` landed in a lane-local DB the hook could never see. All 21 v1-ship-1 retros were stranded; ~27+ false Stop-hook blocks accumulated across the entire phase before the path mismatch was identified. A one-character fix resolves a class of failure that compounds silently for weeks.

**How to apply:** `bash .claude/hooks/pmd-canonical-guard.sh` (shipped in v1-rls-r1 Task 3) runs at every SessionStart and emits a loud WARN naming both paths if they diverge — lane drift surfaces within seconds, not after a phase of stranding.

## Invariant 2: Two systems, one source of truth

**Why non-negotiable:** The two systems share the name "project memory" and both surface through the same MCP server, which makes them easy to conflate. During the advisor-CWD migration (2026-04-30), a Step 4 "migrate 200 memory files" correctly moved the System 1 `.md` files but included no step for the System 2 SQLite DB. `memory_search_hybrid` returned zero results despite the `.md` files being present, because the DB was not seeded — the systems look like one but are populated independently and have different failure modes.

**How to apply:** `enforcement: PENDING — no automated check ships in v1-rls-r1`; apply the two-system mental model manually: when touching "project memory", address System 1 (`ls ~/.claude/projects/.../memory/*.md | wc -l`) and System 2 (`memory_search_hybrid(query: "test", limit: 1)` to confirm DB is seeded) explicitly.

## Invariant 3: No write-time embedding

**Why non-negotiable:** Source inspection of `dist/index.js` (2026-05-16) confirmed that `memory_write`, `memory_write_eval`, `memory_promote_to_file`, and `memory_update` contain zero `embed` / `ollama` / `memory_vectors` / `generateEmbedding` references — all embedding happens exclusively in the separate `dist/scripts/backfill.js`. Every PMD write is therefore immediately FTS5-searchable but NOT semantically searchable until backfill runs. `memory_search_hybrid` silently degrades to FTS5-only for unembedded rows with only a single stderr line as signal. The +62.7% Recall@10 hybrid advantage is absent during the window between write and backfill (up to ~6.5 days for Junior daemon-side writes). Eight PMD memories were found unembedded at one audit point — five from sessions that simply did not know the backfill step existed.

**HTTP topology note (2026-05-30+):** Under the HTTP-daemon topology the interactive enforcement point is gone: lessons auto-sync to the HTTP server via the `lesson-pmd-sync.sh` PostToolUse hook (FTS5-immediate), and the weekly-review Step 1b sweep is the sole embedding backfill. The `mcp-write-time-embedding.md` spec ships in v1-rls-r1; the actual `dist/index.js` patch is a separate `MCPs/project-memory-mcp` PR.

## Invariant 4: LESSON-trailer discipline

**Why non-negotiable:** Junior subagents run on the EliteDesk in worktrees and cannot write directly to laptop-side PMD — different machine, no SSH-back path. Without a discipline, lessons learned during a Junior task have nowhere to land: commit messages capture the *what* but not retrospective framing; DQ resolved entries capture *answers* but not generalisable signals; worktrees get reaped after task completion. A phase with 10–30 promotable lessons that were never signalled produces a PMD that looks current but silently misses the most recent sub-phase's hard-won knowledge. The LESSON-trailer convention creates a machine-readable signal the advisor harvests at retro time; the `kind: "log"` DQ path provides mid-task durability for time-sensitive observations.

## Invariant 5: SessionStart canonical-PMD guard

**Why non-negotiable:** The `.mcp.json.example` template has carried the correct absolute `PROJECT_MEMORY_DB` since v1-ship-1 but nothing *checks* that a given lane's actual `.mcp.json` matches the canonical path. A lane bootstrapped from an older template, or with a stale relative path, fails with zero feedback until the symptom compounds — v1-ship-1 accumulated ~27+ false Stop-hook blocks before the mismatch was found. A documentary guard that is never checked is equivalent to no guard: the knowledge exists; the enforcement does not; the footgun re-fires silently. The SessionStart hook converts a paper invariant into a loud signal at the cheapest possible moment.

## 2026-06-11 homeserver cutover — IP correction + store split

**IP CORRECTION (2026-06-11):** `100.104.171.26` is **the LAPTOP** (`desktop-jtgr71s`), NOT homeserver — earlier topology notes that cited it as "the Tailscale endpoint" reflected the OLD laptop-hosted daemon. homeserver = `100.81.145.58`. A laptop-local PMD daemon may still be listening on `:11435`; do NOT point `.mcp.json` at the laptop IP — that store split from homeserver on 2026-06-11 (4 rows, reconciled). Verify the node before trusting any `100.*` PMD IP: `tailscale status | grep <ip>`.

**Pre-cutover note (for archive context):** until 2026-06-11 the daemon ran on the laptop and the sqlite3-CLI file sync (`scripts/sync-lessons-to-pmd.sh`) DID reach the live store; that is no longer true after the homeserver HTTP-daemon cutover. The laptop-local `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` is now a STALE copy; writes to it do NOT propagate to the live daemon.
