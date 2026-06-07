---
name: git status before meta-work on canonical — foreign WIP means another session is live; default to a dedicated worktree for ad-hoc/meta work
description: Before any meta-edit on the canonical brehon-fork checkout (lesson/rule/template/brief/retro/governance-v0 DQ), run `git status --short`. Any tracked modification or untracked file you did NOT create this session means a concurrent CC session is actively driving the SAME shared `.git/` — STOP and route to a dedicated worktree for the other work, or defer. The 2026-06-07 retro-promotion leg ran meta-work on canonical while a second session ran a live diesel regen in the same tree; the second session's commits silently reset the index mid-`git commit`, landing a no-op. The durable fix is upstream: for ad-hoc / meta work, spin a throwaway worktree (you already have a clean bm-cut + `git worktree add` flow) instead of borrowing the canonical checkout another session may be using.
type: feedback
---

## TL;DR

The canonical `brehon-fork` checkout shares one `.git/` (one index, one working
tree) with every CC session that uses it. When two sessions touch it at once, the
second's `git add`/`git commit`/`git reset`/`git checkout` silently mutates the
first's staging area — staged files get unstaged, commits land as no-ops, HEAD moves
under you.

**Two-part discipline:**

1. **Gate (recovery):** before the FIRST meta-edit on canonical, run
   `git status --short`. Any tracked modification or untracked file you did NOT
   create this session = a concurrent session is live in this tree. STOP. Route to a
   dedicated worktree for the other work, or defer.

2. **Default (prevention):** for ad-hoc or meta work, don't borrow the canonical
   checkout at all if a phase session might be using it — **spin a dedicated
   throwaway worktree** (`git worktree add ../brehon-fork-<slug> <branch>`). The
   bm-cut + worktree flow is already clean and fast; use it. Canonical stays reserved
   for `governance-v0` meta-edits done when the tree is yours and clean.

## Why this matters (2026-06-07 retro-promotion leg)

Three intents were funneled through one working tree:

- canonical `brehon-fork` was parked on `phase-m2-late-1` (left for "the other
  session"),
- a second CC session was *actively driving* m2-late-1 work in that same checkout — a
  live diesel `print_schema` regeneration (emptied `schema.rs`, `diesel.toml.bak`,
  `patch_file` line removed),
- this session switched canonical to `governance-v0` to commit 3 meta-files.

While committing, between `git add` (4 files staged) and `git commit -F`, the other
session committed **twice** and reset the index. The commit ran against an emptied
staging area: `no changes added to commit` — a silent no-op. HEAD had also moved two
commits forward under the session's feet.

No data was lost (the foreign WIP was scoped out, the meta-files were untracked-safe),
but ~10 min went to diagnosing "why did my commit vanish," plus a re-commit. The
race was won the second time by an atomic burst (below) — but the race should never
have been entered.

**The real mistake was upstream, not in the git mechanics:** the m2-late-1 work
belonged in a dedicated lane worktree (like `brehon-fork-m2rooms-a` already is), so
canonical would have stayed clean and contention-free. And the signal was there —
`git status` showed `crates/db_schema_file/src/schema.rs` modified + a `.bak` I never
created. That foreign WIP, by itself, was sufficient evidence to STOP and not write to
this tree.

## When to apply

The gate fires **before the first meta-edit** on the canonical `brehon-fork` checkout:
a lesson, a rule, a template, a brief on `governance-v0`, a retro, a plan-time DQ
write — anything that will `git add`/`git commit` into the canonical `.git/`.

Foreign-WIP signatures (in `git status --short`, files you did NOT create this
session):

- `crates/**`, `migrations/**`, `Cargo.*` modifications — you're doing meta-work; these
  are someone's impl/regen WIP.
- A `*.bak`, `*.orig`, a half-regenerated generated file (e.g. an emptied
  `schema.rs`).
- Untracked debug artifacts under `.claude/PRPs/debug/` you didn't author this session.
- Any staged change in `git diff --cached` you didn't stage.

Does NOT fire when:

- The only changes are ones YOU made this session (your own in-progress edits).
- You're in a **dedicated lane worktree** (`brehon-fork-<lane>`) — that's yours; the
  whole point of the worktree is single-session ownership.
- The tree is clean (`git status --short` empty) — proceed normally.

## How to apply

1. **`git status --short`** before the first meta-edit. Empty or yours-only → proceed.

2. **Foreign WIP present → STOP and route:**
   - **(a) Dedicated worktree for the OTHER work (the right fix when it's a live
     phase):** `git worktree add ../brehon-fork-<lane> <their-branch>`, tell the other
     session to move there. Canonical is now yours + clean.
   - **(b) Defer your meta-work** until the tree is quiescent and back on
     `governance-v0`. Usually cheap: a new lesson's PMD row is written at file-save
     time by the `lesson-pmd-sync` hook, independent of git — it's searchable even
     before it's committed, so there's rarely urgency to race the commit.
   - **(c) Atomic burst (accept-residual-race fallback, surface to user first):** if
     you must land a small, file-scoped commit on a checkout another session is
     touching, do the whole thing in ONE uninterrupted `&&`-chained shell invocation —
     stage ONLY your files → `git commit -F <msgfile>` → `git log -1` +
     `git show --stat HEAD` to verify only your files landed (catches a foreign WIP
     swept in). The single invocation gives the concurrent writer no gap to slip
     between your add and commit. This is the fallback, never the default.

3. **Prefer the worktree default for ad-hoc/meta sessions going forward.** If you know
   you'll be doing meta or ad-hoc work and a phase session is (or might be) running,
   `git worktree add ../brehon-fork-adhoc governance-v0` and work there — one session,
   one CWD, one tree (hard refusal #5). Remove it when done
   (`git worktree remove ../brehon-fork-adhoc`).

## Hard refusals

- **NEVER write a meta-edit into the canonical checkout without a `git status` check
  first when any other CC session could be active.** Foreign WIP is a stop signal, not
  a "is this safe to leave?" curiosity — it means the tree has another owner right now.

- **NEVER treat the #6 atomic-burst as the default for contended commits.** It accepts
  residual race risk and must be paired with a post-commit isolation verify + a
  user surface. The default is (a) dedicated worktree or (b) defer.

- **NEVER funnel a second intent through a checkout that already has one.** One
  session, one CWD, one lane (hard refusal #5). Two intents in one tree is the
  precondition for every shared-`.git/` race.

## Cross-references

- `.claude/rules/multi-lane-worktree.md` Hard refusal #7 — the terse rule form of this
  gate (and #6, the atomic read-mutate-commit protocol it falls back to; #5, one
  session one lane).
- `feedback_cross_session_commit_attribution_collision.md` — the sibling: sessions
  sharing `.git/` race staged files; mitigation `git status` between add and commit.
  This lesson is the *pre-flight* gate (check before you start); that one is the
  *mid-flight* mitigation (check between add and commit).
- `.claude/rules/multi-lane-worktree.md` §"Lifecycle" + `feedback_phase_lane_worktree_bootstrap_checklist.md`
  — the dedicated-worktree setup this lesson recommends as the default for ad-hoc/meta
  work.
- `.claude/PRPs/reports/session-retro-2026-06-07-pheromone-pmd-impl.md` "What to carry
  forward" — the atomic-burst technique + the incident that surfaced this gate.
