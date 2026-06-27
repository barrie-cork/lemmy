# Session retro — 2026-06-27 — comms-research messaging foundation

**Harness:** claude-code
**Session window:** ~2026-06-27 14:55 → 17:20 UTC (~145 min)
**Branch at start:** `49ca15eb8` (`governance-v0`)
**Branch at end:** `8c2519e27` (`governance-v0`)
**Files touched:** 22 (all under `docs/comms-research/`)
**Commits:** 7 (auto: 0, explicit: 7)

## TL;DR

Interactive docs-authoring session: built the Brehon Consensus communications
research foundation (15 theme files + source-notes + machine message bank +
the user-requested Perplexity deep-research prompt + a NEXT-SESSION handover),
for a grassroots-organiser-facing pamphlet/website/press. The session's defining
dynamic was a **stream of ~14 founder essence-clarifications arriving mid-work**;
the most load-bearing finding is the user's own process correction — *during
brainstorming, log notes rather than start writing* — which I half-adopted only
after the user said it explicitly. The top change: **detect brainstorm mode early
and default to capture-and-summarise, gating all file-writing on an explicit "now
write" signal.**

---

## What surprised us

- **The work was a moving target by design.** I scoped it up front via
  AskUserQuestion as "research + themes only", but the *content* kept evolving:
  the user fed ~14 distinct essence-clarifications across the session (anti-TINA →
  "lots of alternatives", mediation-vs-moderation-vs-governance, reputation-by-
  helping, engagement≠reputation, two-way standing, community-set stakes, the
  bot/provocateur defence, everybody-moderates, the rules-not-feelings correction,
  bottom-up rule-making). Each was a genuine sharpening, not noise — but I treated
  the first several as "write it into the files now" rather than "log and batch."
- **The user had to correct my interaction style mid-session** ("just record my
  points... then summarise them back. It's only then that we decide to do the
  writing, because going back and forth and writing is probably confusing for
  you"). This is the clearest signal of the session and I should have offered it
  *before* being told.
- **A 379KB Perplexity research brief appeared in `docs/comms-research/research/`
  during the session** (untracked at retro time) — the findings arrived faster
  than the "next session" framing assumed. The NEXT-SESSION.md handover I wrote
  assumes the research is delivered in a *future* session; it's already here.
- **The "bottom-up, not top-down" spine emerged late but unified almost
  everything** — it wasn't in the original framing; it surfaced from the user's
  Brehon-law-was-bottom-up point and retroactively organised 6+ themes.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Detect brainstorm/ideation mode and default to "capture + summarise back, defer writing."** When a session is clearly idea-generation (user feeding successive conceptual points, no concrete artifact requested yet), **log each point to a running notes file rather than begin writing tasks**, summarise back, and explicitly ask "keep going or shall I wire these in?" *before* editing files. **Use AskUserQuestion for clarity whenever a point is ambiguous** rather than guessing and writing. Offer this mode proactively at the first 2–3 clarifications, don't wait to be told. | Eliminates churn from re-editing files after every message; matches how the user wants to brainstorm; the user named this explicitly as confusing. | minor (behavioural) | 1× this session (user-stated, twice) → meets ≥2 |
| 2 | **Batch wiring into one consolidated pass at a natural checkpoint, not per-message.** Even outside explicit brainstorm mode, when ≥3 related conceptual edits are queued, hold them and apply once. I did this correctly for themes 11–15 at the end; do it from the start. | Fewer commits, less cross-file inconsistency mid-stream, lower context burn. | minor | 2× this session (early scattered edits vs. late batched pass) |
| 3 | **When founder framing leans on a factual claim, record it as a must-verify proof-point with a verification hook — never as settled fact.** I did this for the court-case claim (✓) and softened an over-confident README line, but only after it had already shipped confidently once. Apply at first mention. | Protects narrative-spine credibility; one debunked fact discredits the whole "alternatives exist" argument. | minor | 1× this session + matches existing `feedback_unattributed_security_artifact_reject` / verify-claims family in prior memory → ≥2 |
| 4 | **Add `docs/comms-research/research/` (large raw research dumps) to `.gitignore` OR commit-and-flag deliberately.** A 379KB brief is sitting untracked; decide its fate explicitly rather than leaving it in limbo. | Avoids accidental bloat commit or accidental loss. | trivial | 1× (note, not promote) |

## What to carry forward

- **AskUserQuestion up front to lock scope + audience + framing was high-value.**
  The 4-question opener ("who's the reader / first deliverable / Brehon-framing
  centrality / analogy use") set the whole session's direction cleanly and
  prevented building the wrong thing. Repeat at the start of any
  open-ended authoring task.
- **OKF-style human-readable `themes/` + machine-readable `machine/` layout**
  worked — the `messaging.yaml` + `glossary-plain.yaml` give downstream
  site/CMS reuse without re-deriving, and stayed in sync via the consolidated
  wiring pass.
- **Honesty discipline held throughout** — every unverified/aspirational claim
  tagged "designed to / verify before public", alpha status kept visible. This is
  exactly right for the savvy grassroots audience and should be the default for
  all public-facing copy work.
- **Writing a NEXT-SESSION.md handover before closing** means the research-
  processing work survives the session boundary with a clear section→file
  reconcile map. Good pattern for any multi-session deliverable.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| AskUserQuestion (scope opener) | 30 | 0 | low | 4-question fork locked audience/scope/framing cleanly; prevented wrong-deliverable build |
| Direct file authoring (15 themes + machine + prompt) | — | ~15 | medium | ~15 min churn re-editing files mid-stream before adopting notes-then-write mode |
| Per-message editing (early session) | 0 | ~15 | medium | the friction the user corrected — see What-to-change #1/#2 |
| Consolidated wiring pass (themes 11–15) | 20 | 0 | none | batching at end was clean and fast — the model for #2 |
| NEXT-SESSION.md handover | 15 | 0 | low | preserves cross-session continuity; one assumption already stale (research arrived same-session) |
| memory_write_eval (task retro) | — | 0 | none | clean |

## Complexity scores (heavy tasks only)

Not applicable in the Junior-impl sense (no Junior tasks, no cargo, no worker
log-silence). Adapted docs-authoring metric for the one heavy thread:

| Task | Files | Commits | Runtime (min) | Max gap (min) |
|---|---:|---:|---:|---:|
| comms-research foundation (whole session) | 22 | 7 | ~145 | n/a (interactive) |

No watchdog envelope applies (interactive session, no worker). The 22-files/7-
commits spread is healthy — small per-commit units, no single oversized edit.

## Decisions to revisit

- **NEXT-SESSION.md assumes research arrives next session; it's already here**
  (379KB brief untracked in `research/`). Next action should reconcile that brief
  against the themes *now-or-next*, per the handover's section→file map — the
  handover is still valid, just earlier than planned.
- **Is `docs/comms-research/` the right home, or should comms move out of the
  code repo** before go-public? It's docs meta-work on `governance-v0` now; if a
  separate marketing-site repo is created, this folder is the seed. Worth a
  clarify when the website work starts.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [x] What-to-change #1 (brainstorm mode = capture-and-summarise, defer writing):
      promoted to `.claude/lessons/feedback_brainstorm_log_notes_not_write.md`
      (cross-harness lesson) this session. **User explicitly requested this be
      captured** ("when brainstorming, log notes rather than begin writing tasks";
      "AskUserQuestion for clarity if needed").
- [x] What-to-change #2 (batch wiring into one consolidated pass): folded into the
      same lesson as #1 (same root: don't write per-message during ideation).
- [ ] What-to-change #3 (founder factual claims → must-verify proof-point at first
      mention): augment existing `feedback_verify_automated_reviewer_claims_against_compiler.md`
      family OR new `feedback_founder_claim_must_verify_at_first_mention.md`.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
