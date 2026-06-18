# Session retro — 2026-06-18 — m3-entry-kinds-plan

**Harness:** claude-code
**Session window:** 2026-06-17 ~22:39 → ~23:38 BST (~60 min witnessed)
**Branch at start:** `300161009` (`governance-v0`)
**Branch at end:** `538952385` (`governance-v0`, after rebase+push)
**Files touched:** 3 (1 plan CREATE, 1 doc-drift UPDATE, +5 prior-unpushed commits rebased)
**Commits:** 2 explicit this session (`b5df28dcf` drift, `324d72d2f`→`538952385` plan), 0 auto

## TL;DR

`/prp-plan` on the M3 town-halls PRD. The PRD has 6 pending phases; one decision (which phase
to plan) was genuinely the user's, so I asked it up front via `AskUserQuestion` rather than
defaulting — user picked Phase 2 (entry kinds), the cleanest in-workspace Rust unit. Two
targeted `Explore` agents on the exact files the PRD named returned everything needed in one
pass, including a finding the PRD missed: a `ROOM_KINDS` runtime-validation array that hardcodes
"10 allowed" and must be bumped to 13 (the one non-mechanical edge). Plan written at 9/10
confidence. The agents also surfaced pre-existing `04-data-model-and-api.md` doc-drift (entry-kind
count stale at 55, should be 69) — fixed in a separate `docs(drift):` commit once the user
confirmed. Main carry-forward: **targeted Explore agents keyed to PRD-named file:line beat broad
exploration for pattern-mirror plans** — narrow scope, one-pass, caught the gate the PRD author
didn't.

---

## What surprised us

- **The PRD's own "crate(s) affected" list under-counted the edit.** The PRD named the `db_schema`
  const block and the `api` shim re-export but omitted the `ROOM_KINDS` validation array in the
  same shim file — an `#[cfg(feature = "full")]` runtime gate that hardcodes "the 10 allowed kinds"
  and rejects anything else (`append_room_event`'s ADR-008 integrity check). M3's 3 new kinds would
  compile clean but be runtime-rejected by the phase-3/4 bridge emitters if that array isn't bumped
  10→13. The Explore agent caught it because the brief asked specifically "is there any exhaustiveness
  mechanism over these consts." A broad "find relevant code" prompt likely wouldn't have.
- **The `04` doc-drift was two layers deep.** Not only was the count stale (55 vs 69), the era-breakdown
  string itself disagreed with the canonical registry (`7 v1-JM` lumped vs registry's `6 v1-JM-a + 1 v1-JM-c`).
  Fixing the headline number without realigning the breakdown would have left a subtler drift.
- **The push wasn't a fast-forward.** Origin had advanced by one unrelated docker-pin commit
  (`a13204da5`) while my local branch carried 6 unpushed `docs/`/`.claude/` commits. `--ff-only` aborted;
  a clean conflict-free rebase resolved it (zero file overlap, verified before rebasing).

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When `/prp-plan` targets an entry-kind-registration phase, the Explore brief MUST ask "is there a runtime allowlist/validation array (e.g. `ROOM_KINDS`) gating these consts, and does it hardcode a count?" — add as a standing question in the prp-plan entry-kind pattern | Catches the `ROOM_KINDS`-bump edge automatically; the next entry-kind sub-phase (M3 has more coming) won't rediscover it | minor | 1× this session, but structural — recurs every chair/mute/room-kind sub-phase |
| 2 | `04-data-model-and-api.md` §11 entry-kind count is a recurring drift surface (was stale at 55 through all of M2). Add a one-line acceptance-invariant pointer in `04` §11 that says "count tracked authoritatively in the registry; cross-check at every entry-kind sub-phase ship" — OR have `/brehon-phase-transition` grep `04` for the count when an entry-kind phase closes | Stops `04`'s count drifting a 4th time | minor | drifted across m2-core-hook + m2-late-1 + m2-late-b-actor (3 sub-phases) without a 04 update |

## What to carry forward

- **Targeted Explore agents keyed to PRD-named file:line beat broad exploration for pattern-mirror
  plans.** Two agents, each scoped to the exact files the PRD §Technical-Approach named, returned
  copy-paste-ready snippets + the missed gate in one pass. No re-search needed to write the plan.
  This is the right shape whenever the plan is "apply an existing pattern an Nth time."
- **Ask the genuinely-user decision up front, default everything else.** The PRD had 6 pending phases;
  "which to plan" is a real fork (dependency order vs. cleanest-first vs. all-at-once) the user owns.
  One `AskUserQuestion` with a recommendation + previews resolved it cleanly; everything downstream
  (skip Phase 3 research, skip e2e test section, scope out the `04` drift from Junior's edit) was
  defaulted with a one-line rationale.
- **Verify file-overlap before rebasing a diverged push.** `git diff --name-only origin...HEAD` vs
  `git show <their-commit> --name-only` confirmed zero overlap → rebase was safe and conflict-free.
  Cheap check, avoids a blind rebase.
- **Honest session-boundary discipline at retro time.** The git log showed an `/auto-phase` leg
  (bm-cut, impl brief, cohort dispatch) starting 67s after my witnessed session ended — a continuation
  I didn't see. Scoped the retro to the witnessed leg only; did NOT assess the auto-phase execution
  (Step 0.5 trigger correctly does not fire — I neither invoked nor mutated auto-state).

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `AskUserQuestion` (phase scope) | 5 | 0 | none | Clean fork on which of 6 phases to plan; previews made the trade-off legible |
| 2× `Explore` agents (parallel) | 25 | 0 | medium | Returned exact snippets + caught the `ROOM_KINDS` gate the PRD missed; one-pass, no re-search |
| `/prp-plan` skill body | 15 | 0 | low | Phase 3 (external research) correctly skipped — pure in-workspace mirror; confidence 9/10 |
| `04` drift fix + commit | 3 | 0 | low | Two-layer drift (count + breakdown); realigned to registry-canonical |
| rebase + push | 2 | 3 | low | `--ff-only` aborted on divergence; overlap-check + rebase recovered; ~3 min on the abort/diagnose |

## Complexity scores (heavy tasks only)

None qualify — no task exceeded the >55min / >40min-silence / >8-files thresholds. The plan-authoring
task was `1/1/~20/0` (1 file, 1 commit, ~20 min including 2 parallel Explore agents, no log silence).
This is a comfortable-zone interactive session, not a Junior-dispatched one — complexity-score
discipline noted as N/A.

## Decisions to revisit

- M3 has 4 more bridge-side phases (1, 3, 4, 5, 6) still to plan, all higher complexity (Linux-only
  cargo, Docker sidecars, distributed-systems mute<500ms). The clean Phase-2-first ordering means the
  next `/prp-plan` run is Phase 1 (infra) — a different beast (mostly out-of-workspace). Worth a fresh
  scope decision when it comes.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [x] Change #1 (Explore brief must ask about runtime allowlist arrays for entry-kind phases): SHIPPED in
      `4b6f96f0a` → `.claude/lessons/feedback_entry_kind_runtime_allowlist_check.md` (92 lines; generalises to
      config-seed/AP-registry shapes). Landed by the post-plan `/auto-phase` continuation, not this session.
- [x] Change #2 (`04` §11 count cross-check at entry-kind phase ship): SHIPPED in `4b6f96f0a` →
      `~/.claude/skills/brehon-phase-transition/SKILL.md:421` — two-layer cross-check (headline + era-breakdown),
      citing the lesson + this retro. Landed by the same continuation.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
