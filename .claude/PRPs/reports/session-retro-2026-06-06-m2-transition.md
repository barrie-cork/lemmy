# Session Retro — 2026-06-06 m2-transition

**Session:** m2-core-hook close + m2-rooms-a bootstrap
**Date:** 2026-06-06
**Branch/CWD:** `brehon-fork-m2` (lane worktree) / `phase-m2-core-hook`
**Commits this session:** `f7a9c9c12` (`chore(brehon): close m2-core-hook, bootstrap m2-rooms-a`)

## What surprised us

**Advisor:**

- **The "🔴 NEXT SESSION" MEMORY.md entry was completely stale.** All four
  e2e.rs corruption fixes had been applied (`719c800b1`), the DQ entry
  `1dcd6a201003-002` was already at `result:pass`, and PR #184 was fully merged
  well before this session started. The handover doc
  `m2-core-hook-apply-e2e-corruption-fixes-2026-06-05.md` was written
  mid-diagnosis (before fixes were applied) and the MEMORY.md 🔴 entry was
  never cleared after the session completed the work. A fresh session
  reading MEMORY.md would have spent 5+ minutes "applying fixes" before
  discovering nothing to do. This is at least the second time a stale 🔴
  entry has caused false-start overhead.

- **`git worktree remove ../brehon-fork-m2 --force` failed with `fatal:
  '../brehon-fork-m2' is not a working tree`** when run from the canonical
  `brehon-fork` checkout. The relative-path resolution `../brehon-fork-m2`
  from `C:/Users/barri/Developer/brehon-fork` should expand correctly, but
  git did not recognise the worktree as registered. Cause was not fully
  investigated before session end — session was interrupted after the failure.

**Planning / Impl / BM:**

- Not active this session (transition-only session; no Junior tasks dispatched).

## What to change

**Advisor (recurrence basis — ≥2 occurrences):**

1. **MEMORY.md "🔴 NEXT SESSION" entries need a completion ritual.** When a
   session completes work that was flagged 🔴, the MEMORY.md entry must be
   updated or removed as part of that session's close — not just the DQ entry.
   The current post-task-retro close checklist has no step for this.
   **Proposed addition to `post-task-retro` SKILL.md (or to advisor-orchestrator
   §"Pre-compact handover discipline"):** "If this session resolves a 🔴 NEXT
   SESSION item in MEMORY.md, clear or replace it before the handover commit.
   The handover doc is the living record; MEMORY.md's 🔴 bucket is a dead-drop
   that expires on completion." The lesson already named is
   `feedback_stale_metric_lesson_guard.md` — the discipline should extend to 🔴
   entries (which are not metrics, but share the same staleness failure mode).

2. **Worktree removal needs a `git worktree list` pre-check.** Before `git
   worktree remove <path>`, run `git worktree list` to read the exact registered
   absolute path, then use that path — not a relative path derived from the shell
   CWD. The `../brehon-fork-m2` form failed; `git worktree list` would have shown
   the canonical absolute path. **Proposed addition to `brehon-phase-transition`
   skill closing surface:** after writing the bootstrap file and committing, emit:
   "Closing worktree cleanup — run `git -C brehon-fork worktree list` first, then
   use the listed path in `git worktree remove <ABSOLUTE_PATH>; git branch -d
   phase-<id>`." The m2-rooms-a bootstrap file already notes this in its "Next
   concrete action" block — the skill should emit it as a closing surface item
   without requiring the user to read the bootstrap.

**Advisor (single-occurrence, record only):**

3. **`/brehon-phase-transition` invoked with only the completing phase ID**
   (missing next-id). The skill correctly asked for it; user confirmed `m2-rooms-a`
   naturally. No change needed to the skill (the ask-before-proceeding is the
   right behaviour), but worth noting: the usage in MEMORY.md's "Active workflow
   state" bullet should be kept to the full form (`/brehon-phase-transition
   <completing-id> <next-id>`) as a reminder. One occurrence, not a pattern.

4. **`/auto-phase M2` hard-refused** because the session CWD was `brehon-fork-m2`
   on `phase-m2-core-hook`, not canonical `brehon-fork` on `governance-v0`. The
   hard refusal was correct. The MEMORY.md M2 PRD entry should note that
   `/auto-phase` requires governance-v0 CWD as a prerequisite — users invoking
   it from a lane worktree will get the hard refusal. One occurrence; adding it
   to the bootstrap's operational rules would be sufficient.

## What to carry forward

**Advisor:**

- **Stale MEMORY.md verification at `/start-brehon` time is worth adding.** The
  session-start skill already synthesises live state from git/gh/DQ. Adding a
  30-second cross-check — "if MEMORY.md has a 🔴 NEXT SESSION item, compare it
  against `git log --oneline` to confirm the named commit or DQ entry is still
  open before surfacing" — converts a potential 5-min false start into an instant
  "already done" notice. Low implementation cost.

- **`git worktree list` before `git worktree remove` is muscle memory now.**
  The brehon-phase-transition skill should emit the command as a closing surface
  item (not buried in the bootstrap). Applied to m2-rooms-a bootstrap already
  (§"Next concrete action" block cites `git worktree list` first).

- **The brehon-phase-transition skill itself worked correctly.** All four steps
  executed cleanly: CLOSED record created for m2-core-hook (no prior file existed),
  m1-b two-ago record deleted correctly, `workflow_state_m2_rooms_a.md` skeleton
  created, `m2-rooms-a-bootstrap.md` committed at `f7a9c9c12` on `governance-v0`.
  The skill is stable for M-track transitions (not just v1-track).

- **m2-rooms-a scope is well-defined.** The M2 PRD cleanly separates m2-core-hook
  (binary hook + consts, shipped) from m2-rooms-a (bridge provisioning + hash-chain
  + integration tests). The bootstrap has 5 concrete stop-and-ask tripwires and a
  clear watchlist. Carry forward unchanged.

---

## §3 — Automation opportunities

| # | Friction | Recurrence | Proposal |
|---|---|---|---|
| A | Stale 🔴 MEMORY.md entries not cleared at session close | 2× | Add cleanup step to post-task-retro SKILL.md or pre-compact handover discipline |
| B | `git worktree remove` relative-path failure | 1× | `brehon-phase-transition` closing surface emits `git worktree list` pre-check command |
| C | `/start-brehon` loads stale 🔴 items without verifying them | 1× | Add 30-second live-state cross-check for 🔴 entries at session-start |

Only A meets the recurrence threshold (≥2) for lesson promotion. B and C are
recorded; if B recurs in m2-rooms-a transition, promote the `git worktree list
first` discipline to a lesson.

---

## §5 — Quantified outcomes

**Tasks dispatched to Junior:** 0 (transition-only session)
**Commits landed:** 1 (`f7a9c9c12` on `governance-v0`)
**User gates cleared:** 1 (phase-transition confirmation = informal gate 6 retro sign-off)
**Catch-fires:** 0
**Auto-phase trigger:** refused at Phase 0 prerequisite check (correct)

**Per-task complexity (this session):**
- Phase-transition work: 0 files of Rust changed, 4 PMD files written/deleted,
  1 bootstrap file created, 1 commit. Wall-clock ~45 min (including stale-state
  investigation). No watchdog risk.

**Stale-state investigation overhead:** ~10 min (confirmed e2e.rs already clean,
DQ already resolved, PR already merged). This is the wasted-signal from the stale
🔴 entry.

**Three-signal scores:**

| Invocation | Saved (min) | Wasted (min) | Surprise |
|---|---|---|---|
| `/auto-phase M2` | 0 (refused correctly) | 2 | Low — hard refusal is the correct gate |
| `/brehon-phase-transition m2-core-hook m2-rooms-a` | ~40 (4-step transition automated) | 5 (worktree remove failed) | Low |
| Stale MEMORY.md investigation | 0 | 10 | Medium — thought there was work to do; there wasn't |
| `/session-retro` | ~15 (template + discipline) | 0 | None |

---

## Lessons promoted this session

| Lesson | Status | Note |
|---|---|---|
| Stale 🔴 MEMORY.md completion ritual | **record only** (1× → pending 2nd) | Not yet promoted; flagged in §3-A |
| `git worktree list` before `git worktree remove` | **record only** (1×) | Not yet promoted; flagged in §3-B |

No new `.claude/lessons/feedback_*.md` files authored this session (no new patterns
met the ≥2 threshold for promotion).
