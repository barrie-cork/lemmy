---
name: The .mcp.json canonical-PMD-path guard is documentation only — enforce it with a SessionStart check so a mis-pointed lane fails loud at session start, not silently mid-phase
description: .mcp.json.example carries a _comment_pmd_cross_lane guard + the canonical absolute PROJECT_MEMORY_DB, but a per-worktree .mcp.json (gitignored) can still be wrong with zero feedback until ~27+ false Stop-hook blocks accumulate across a phase. Promote the documentary guard to an enforced SessionStart hook that asserts the running lane's .mcp.json PROJECT_MEMORY_DB == the absolute canonical path and surfaces a loud warning at session start. Recommended design below; mechanism was clarify-deferred at the v1-ship-1-r2 retro.
type: feedback
---

# Enforce the canonical-PMD `.mcp.json` path at session start, don't just document it

The cross-lane PMD invariant (`feedback_pmd_cross_lane_canonical_db.md`)
is currently protected by **documentation only**:

- `.mcp.json.example` carries a `_comment_pmd_cross_lane` guard key and
  the correct absolute `PROJECT_MEMORY_DB`
  (`C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`).
- `.claude/rules/multi-lane-worktree.md` §"PMD is cross-lane shared"
  states the invariant.

But `.mcp.json` is **gitignored** (it holds API keys) and **per
worktree**. Nothing *checks* that a given lane's actual `.mcp.json`
matches the canonical path. A lane bootstrapped by hand (or by an
older template, or with a stale relative path) is wrong with **zero
feedback** until the symptom compounds: the v1-ship-1 incident
accumulated **~27+ false Stop-hook blocks** and stranded **21 retros**
across an entire phase before anyone noticed the MCP was writing
lane-local. The documentary guard did not prevent it because nobody
re-reads `.mcp.json.example` when opening a session — the human copies
`.mcp.json` once at lane-bootstrap and never looks again.

A guard that is never checked is `feedback_lesson_must_pair_with_
structural_fix_when_fixable.md` in a different costume: the *knowledge*
exists; the *enforcement* does not; the footgun re-fires silently.

## The recommended fix (mechanism was clarify-deferred — this is the spec)

The v1-ship-1-r2 retro flagged this "worth a clarify before the next
multi-lane phase" because the enforcement mechanism was undecided and
no SessionStart hook is wired in any lane worktree today (no
`.claude/settings.local.json` present; `worktree-guard.sh` is a
Junior-only `PreToolUse` hook that exits 0 for interactive sessions).
This lesson records the **concrete recommended design** so the next
multi-lane session implements from a spec rather than re-deriving it:

### Mechanism: a SessionStart hook script + per-worktree settings wiring

1. **New script** `.claude/hooks/pmd-canonical-guard.sh` (tracked):

   - Reads the running worktree's `.mcp.json` (it is at the CWD root;
     gitignored but present at runtime). Resolve
     `mcpServers."project-memory".env.PROJECT_MEMORY_DB` with the
     UTF-8-safe Python one-liner already used in
     `feedback_pmd_cross_lane_canonical_db.md`'s diagnosis recipe:
     `python -c "import json,io;print(json.load(io.open('.mcp.json',encoding='utf-8'))['mcpServers']['project-memory']['env']['PROJECT_MEMORY_DB'])"`
   - Compute the canonical target from `git rev-parse --git-common-dir`
     → `$(dirname "$GC")/.project-memory/memory.db` (this is exactly
     what `retro-check.sh` does, so the guard and the hook agree by
     construction).
   - If the `.mcp.json` value (normalised — resolve to an absolute
     path, case-fold the drive letter on Windows) **!=** the canonical
     target → emit a **loud, non-blocking** SessionStart warning naming
     both paths, the incident, and the one-line fix ("edit `.mcp.json`
     `PROJECT_MEMORY_DB` to `<canonical>` and restart the MCP — the
     running MCP cached its handle at startup, see the sequencing
     constraint in `feedback_pmd_cross_lane_canonical_db.md`").
   - If `.mcp.json` is absent (some lanes have none and resolve to
     canonical by default — the federation-inbound-a case) → exit 0
     silently; absence is *safe* here (MCP default resolves canonical),
     unlike a *wrong explicit* value.
   - Exit 0 always (SessionStart should not hard-block the session —
     the cost of a blocked session > the cost of a loud warning the
     human acts on; this mirrors the WARN-vs-FAIL reasoning in the
     `sync-lessons-to-pmd.sh --strict` design, but at session start the
     correct default is WARN because there is no automated caller to
     swallow stderr — the human sees the SessionStart banner).

2. **Wiring** — a `SessionStart` hook entry in
   `.claude/settings.local.json`. **Caveat (load-bearing):**
   `.claude/settings.local.json` is itself **per-worktree and
   gitignored** (`feedback_settings_local_json_worktree_bootstrap.md`),
   so adding the hook in one lane does NOT propagate to others. The
   wiring must therefore be part of the **lane-bootstrap checklist**
   (`feedback_phase_lane_worktree_bootstrap_checklist.md`) — the same
   place the `.mcp.json` copy already happens. Bootstrapping a lane =
   copy `.mcp.json` from template (canonical path carries over) AND add
   the `pmd-canonical-guard.sh` SessionStart entry to that lane's
   `settings.local.json`. The guard then *verifies on every session
   start* what the bootstrap *set once*, catching drift (stale copy,
   hand-edit, template regression).

   The script being **tracked** while the wiring is **per-worktree** is
   the correct split: the *check logic* is shared knowledge (lives in
   git, improves once for all lanes); the *activation* is per-lane
   config (like `.mcp.json` itself).

### Why a SessionStart WARN, not a PreToolUse block or a hard fail

- The defect is a *configuration* error fixed by a one-line `.mcp.json`
  edit + MCP restart — there is nothing the session can do mid-flight
  (the MCP cached its handle at startup), so blocking tool calls would
  only frustrate, not fix.
- SessionStart is the exact moment the human can act before any retro
  is written into the wrong DB — surfacing it there prevents the
  *entire phase's* worth of stranding, which is the actual cost.
- Non-blocking because a false positive (e.g. an unusual but valid
  canonical path on a different machine) must not wedge the session;
  the human reads the banner and judges. Same principle as
  `feedback_principles_not_rules.md`.

## How to apply

- **Until the hook ships:** at the start of any multi-lane session,
  manually run the `feedback_pmd_cross_lane_canonical_db.md` diagnosis
  recipe steps 1–2 (compare `.mcp.json` `PROJECT_MEMORY_DB` vs
  `git rev-parse --git-common-dir` → canonical). 10 seconds; prevents a
  phase of stranding. This is the interim mitigation, and its
  *existence as a manual step* is the signal the check should be
  automated (this lesson).
- **When implementing the hook** (a focused session, before the next
  multi-lane phase per the retro's clarify deferral): follow the spec
  above; add the `settings.local.json` SessionStart wiring to the
  lane-bootstrap checklist, not as a one-off in the current lane.
- **Triage signal:** if you ever see `retro-check.sh` false-block
  despite a genuine retro, FIRST run the diagnosis recipe — do not
  edit the hook (hard refusal in `feedback_pmd_cross_lane_canonical_db.md`).
  The guard, once shipped, makes this triage unnecessary because the
  SessionStart banner already told the human at session start.

## Generalises to

Any invariant that is "documented in a tracked example/rule" but
"enforced only by a gitignored per-worktree config nobody re-reads".
The fix pattern is always: a *tracked* check script (shared logic) +
*per-worktree* activation folded into the bootstrap checklist (so the
activation travels with every new lane and the check catches drift on
every session). Documentation states the invariant; the SessionStart
check makes a violation *loud at the cheapest possible moment* instead
of silent until it compounds.

## See also

- `feedback_pmd_cross_lane_canonical_db.md` — the invariant + the
  diagnosis recipe this guard automates; the hard refusal against
  editing `retro-check.sh`.
- `feedback_lesson_must_pair_with_structural_fix_when_fixable.md` — a
  documentary guard that is never checked is the same anti-pattern as
  a footgun-lesson without a shipped fix; this lesson IS the
  "fix status: PENDING — SessionStart hook, spec'd here, deferred to a
  focused pre-multi-lane-phase session" record for that gap.
- `feedback_settings_local_json_worktree_bootstrap.md` — why the
  SessionStart wiring is per-worktree and must be in the bootstrap
  checklist, not committed.
- `feedback_phase_lane_worktree_bootstrap_checklist.md` — the
  lane-bootstrap checklist the wiring step attaches to.
- `.claude/PRPs/reports/session-retro-2026-05-18-pmd-stranding-remediation.md`
  — "What to change" #3 / "Decisions to revisit" (the clarify
  deferral this lesson resolves into a concrete spec).
