# Session retro — 2026-06-07 — promotion-leg-git-race

**Harness:** claude-code
**Session window:** ~11:05 → ~13:00 UTC (~115 min, post-compact continuation)
**Branch at start:** `36017f489` (`phase-m2-late-1` — canonical checkout wrongly parked here by a prior session)
**Branch at end:** `cac5d5b9a` (`governance-v0`)
**Files touched:** 6 (this session's scope; the `git diff --stat` range is polluted by a concurrent m2-late-1 session's interleaved commits — see TL;DR)
**Commits:** 3 (auto: 0, explicit: 3) — `a2d943127`, `2df7800ee`, `cac5d5b9a`

## TL;DR

A short, well-scoped meta-work session (ship 3 unchecked promotion candidates from the
pheromone retro) became a live demonstration of the exact hazard the multi-lane discipline
exists for. The canonical `brehon-fork` checkout was being driven by a SECOND active CC
session (m2-late-1 work) at the same time, sharing one `.git/`. Mid-`git commit`, the other
session committed twice and silently reset my staging area — my first commit landed as a
no-op. The most load-bearing finding wasn't in the planned deliverables; it emerged from the
execution: **the missing gate was a pre-meta-work `git status` foreign-WIP check**, now
shipped as `multi-lane-worktree.md` hard-refusal #7 + a companion lesson. The planned 3
deliverables (ssh-sql helper + 2 deploy-verify lessons) all shipped and verified; the
unplanned 4th (the foreign-WIP gate) is the highest-value output.

---

## What surprised us

- **A concurrent session was live in the canonical checkout the whole time — and I didn't
  register it until it ate my commit.** Between `git add` (4 files staged) and `git commit`,
  the m2-late-1 session committed twice + reset the index → my commit was a silent no-op
  ("no changes added to commit"). The signal had been visible the entire session (foreign
  `schema.rs` modification + `.bak` in `git status`), but I'd framed it as "is this WIP safe
  to leave?" not "does this tree have another owner right now?" — the wrong question.
- **The "stray empty `schema.rs`" was a DOCUMENTED failure signature, not random corruption.**
  A PMD search (prompted by the user: "check PMD, work was done fixing this") surfaced lesson
  id 881 — written by the other session *today* — stating `diesel print-schema > schema.rs`
  truncates the file to 0 bytes via the `>` redirect, then errors before rewriting. The empty
  file was the textbook artifact of an aborted regen. Without the PMD check I'd have guessed
  (correctly, but on luck); with it, "is this safe to discard?" became a confirmed fact.
- **The dedicated worktree already existed.** When asked to "set up `brehon-fork-m2late`," I
  checked first (per the user's instruction) and found it already present AND clean — the real
  m2-late-1 work was properly isolated there all along. The foreign WIP in canonical was *stray
  debris* from an earlier branch-confusion, not active work. The fix was cleanup, not setup.
- **The atomic-burst commit actually beat the race on the second try.** Re-staging + `commit -F`
  + verify chained in ONE `&&`-invocation gave the concurrent writer no gap to slip between
  add and commit. The split sequence lost; the burst won — a real, reproducible technique.
- **The PMD auto-sync hook worked end-to-end with zero manual step.** Both new lessons appeared
  as FTS-searchable PMD rows (ids 879, 880) at file-write time via the `lesson-pmd-sync` hook,
  independent of git — which is *why* deferring the contended commit would have cost nothing
  (the lessons were already recallable before being committed).

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **SHIPPED this session:** `multi-lane-worktree.md` hard-refusal #7 — `git status` before ANY canonical meta-work; foreign WIP = another session live → STOP, route to (a) dedicated worktree / (b) defer / (c) atomic-burst fallback. Companion lesson `feedback_canonical_checkout_foreign_wip_means_stop.md`. | Pre-empts the shared-`.git/` race at the *decision* point, not the *recovery* point (#6 was recovery-only). Would have routed this session to defer (zero cost) instead of racing. | minor (done) | 1× this session + 1× prior (`feedback_cross_session_commit_attribution_collision`) = threshold met |
| 2 | **Default to a dedicated/throwaway worktree for ad-hoc + meta work** when any phase session might be active — `git worktree add ../brehon-fork-adhoc governance-v0`, work there, remove when done. Encode as the recommended default in the lesson (done) AND surface in `advisor-orchestrator.md` §1 session-start as a one-liner. | Removes the precondition for the race entirely (one session, one tree). The user's own instinct ("just do a bm-cut when I need meta work") is the right operating model. | minor | 1× this session; promote the §1 one-liner if it recurs |
| 3 | **PMD-check-before-discard for any "stray/corrupt file" cleanup judgment.** Before `git checkout --`/`rm` on a file that looks like failure debris, `memory_search_hybrid` the filename/subsystem — a recent session may have documented its provenance. | Converts a cleanup *guess* into a confirmed fact; caught the documented-truncation provenance this session. Cheap (one search). | minor | 1× this session; note, promote if 2nd |
| 4 | **The `>`-redirect-truncates-then-fails footgun** (`diesel print-schema > schema.rs` empties the file on any regen error) deserves a wrapper that writes to a temp file + atomic-renames on success only. The other session captured the *diagnosis* (lesson 881) but not the *structural fix*. | Stops the empty-`schema.rs` debris class at the source — no aborted regen ever truncates the committed file. | medium | cross-session (their lesson 881 + this session's debris) — flag to m2-late retro |

## What to carry forward

- **Check-first on "set up X" requests.** The user said "set up the worktree, but check it's not
  already there." It was. Verifying topology before creating saved a duplicate-worktree error and
  reframed the whole task (cleanup, not setup). Make "does this already exist?" the reflex before
  any create/setup action on shared infra.
- **PMD-as-provenance-oracle.** A targeted `memory_search_hybrid` on a confusing artifact's
  subsystem turns "I think this is safe" into "lesson 881 says this is the documented signature."
  Used once decisively this session; the right move whenever cleanup touches another session's debris.
- **Atomic stage+commit+verify burst** (already added to the pheromone retro) — the
  accept-residual-race fallback for landing a small file-scoped commit on a contended checkout:
  stage ONLY your files → `commit -F` → `git show --stat HEAD` isolation-verify, one `&&`-chain.
  NOT the default (defer or dedicated-worktree is), but a proven recovery.
- **Verify isolation after every contended commit.** `git show --stat HEAD | grep -E "<foreign>"`
  after each commit confirmed only my files landed — caught nothing bad this session precisely
  because I checked every time. Keep the post-commit isolation grep as routine on shared `.git/`.
- **AskUserQuestion at genuine cleanup forks, not for confirmation.** Used it to choose lesson
  shape (new file vs extend) and cleanup disposition (what to delete vs preserve) — each answer
  changed what I did. Never used it to ask "is this okay." The one rejected tool-use (user
  redirected to "check PMD first") was the right interrupt and improved the outcome.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Plan-from-`/compact` (the 3-item promotion plan) | 10 | 0 | none | clear inherited task; the plan file + memory handover let me resume cleanly post-compact |
| `ssh-sql.ps1` author + **live test** | 15 | 8 | medium | live test (per pattern_test_against_reality) caught a `-Db`/`-Debug` alias collision + an MSYS path-translation gotcha that prose review would have missed; the 8 "wasted" min were the debug loop, but they *prevented* shipping a broken helper |
| 2 lesson authors (spec-deploy, cross-host) | 12 | 0 | low | canonical format from sibling read; auto-synced to PMD (879/880) at write time |
| `memory_search_hybrid` (PMD provenance check) | 20 | 0 | high | surfaced lesson 881 → confirmed the empty-`schema.rs` provenance; turned a cleanup guess into a fact; user-prompted, high-value |
| AskUserQuestion (×5: lesson shape, foreign-WIP routing ×2, cleanup disposition ×2) | 8 | 0 | none | every answer changed the next action; clean signal-to-noise |
| Atomic-burst commit (×3 across the session) | 12 | 10 | high | the 10 wasted min = diagnosing the first no-op commit; the technique then won 3× and became a carry-forward |
| Canonical cleanup (schema.rs restore + junk delete) | 5 | 0 | low | zero-loss restore (committed everywhere); PMD-confirmed safe before acting |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. Format: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| 3-item promotion (ssh-sql + 2 lessons) incl. live test + race recovery | 4 | 1 | 50 | 3 |
| Foreign-WIP gate (hard-refusal #7 + lesson) incl. race diagnosis | 2 | 1 | 35 | 2 |
| Canonical cleanup (investigate + PMD-check + restore + delete) | 1 | 0 | 20 | 2 |

No task breached the >55min / >40min-silence / >8-files flags. The race added ~15 min of
unplanned recovery distributed across the first two tasks.

## Decisions to revisit

- **m2-late should have had its own dedicated worktree from bm-cut** (like m2-rooms-a does). The
  whole race traces to canonical being used for phase work. Worth a one-line check at every bm-cut:
  "is this phase getting a dedicated lane worktree, or being driven Mode B from canonical?" —
  surface the mode explicitly so canonical never silently doubles as a phase tree.
- **The 3 debug files left in canonical** (`m2-late-1-t1-*`, `m2-late-split-dq-resolution.json`)
  are the other session's notes — disposition deferred to that session (commit / fold into m2-late
  retro / mirror the DQ fragment to the phase branch). The split-DQ-resolution JSON in particular
  may need mirroring to the `phase-m2-late-1` DQ for audit durability.
- **Change #4 (the `>`-truncate-then-fail wrapper)** is a real structural fix the other session's
  lesson 881 stopped short of — flag into the m2-late retro so it's owned, not orphaned.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [x] Change #1 (foreign-WIP gate): **already shipped** as `multi-lane-worktree.md` #7 + `feedback_canonical_checkout_foreign_wip_means_stop.md` (committed `cac5d5b9a`). Threshold met (this + `feedback_cross_session_commit_attribution_collision`).
- [ ] Change #2 (dedicated-worktree-for-ad-hoc default): add a one-liner to `advisor-orchestrator.md` §1 session-start ritual pointing at hard-refusal #7 + the lesson — promote if it recurs.
- [ ] Change #3 (PMD-check-before-discard): promote to a lesson if a 2nd instance lands; currently a single high-value instance noted here.
- [ ] Change #4 (`diesel print-schema > schema.rs` atomic-rename wrapper): flag into the m2-late retro; structural fix owned by the m2-late lane, not this meta session.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
