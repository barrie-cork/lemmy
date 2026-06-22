# role-customization-T4b retro

**Harness:** claude-code (advisor session, canonical brehon-fork / governance-v0)
**Session window:** 2026-06-22 (single session; T4b authored after the m3-core-e2e-pilot → role-customization-T4b transition)
**Branch at start:** `d2ad8d88a` (`governance-v0`)
**Branch at end:** `3915b0fe2` (`governance-v0`, +1 commit, pushed)
**Files touched:** 1 committed (`.claude/skills/weekly-review/SKILL.md`)
**Commits:** 1 (`3915b0fe2`; no auto-commits)

## TL;DR

Shipped T4b: a new **Step 2e in the weekly-review skill** that drains the
EliteDesk role-signal JSONL queue (`drain-role-signal-queue.sh`, scp-based)
then runs the `/check-role-health` report — wiring role-health into the weekly
cadence instead of ad-hoc. Pure `.claude/` skill meta-work, direct-on-governance-v0,
no PR/Junior/cargo. The session was mechanically simple (one skill edit composing
two already-shipped, verified pieces) and the heavy orchestration apparatus
(cohort dispatch, validate-pending, bm-* verbs) stayed dormant — correctly, per
the T4b bootstrap. The load-bearing finding was a **dogfood payoff**: dry-running
the drain script before wiring it surfaced **13 undrained role-signal rows from
the last ~24h**, which is direct evidence that the ad-hoc drain cadence was the
exact gap T4b closes — the feature justified itself at author time. Secondary
finding (post-ship, in the same conversation): a **"browser add-on" request was
mis-scoped onto the Brehon track** before being traced to a different repo
(agent-grey) entirely; the cost was bounded by stopping to search git history
before running any transition against the wrong repo.

This retro is the transition-gate artifact for closing role-customization-T4b.

---

## What surprised us

- **The dogfood dry-run surfaced a live, quantified gap.** Before wiring the
  drain into weekly-review, I ran `drain-role-signal-queue.sh --dry-run` to
  confirm it still worked (the lane record was 27 days old — the bootstrap flagged
  possible drift). It not only ran clean, it printed **13 undrained rows dated
  2026-06-21/22** sitting in `worktree-job-463-queue.jsonl`. That is the feature's
  own justification handed back as runtime evidence: signals *are* accumulating
  and *were not* being drained on any cadence. Surprise direction: positive — a
  verification step I ran for safety doubled as proof-of-need. **Generalises to:**
  dry-running the dependency of a "wire X into a cadence" task often reveals
  whether the cadence gap is real or hypothetical, for near-zero cost.

- **A non-Brehon request very nearly drove a Brehon transition.** "Next up
  browser add-on. Lets run transition session" read as a Brehon track. I started
  toward `/brehon-phase-transition` with a "browser-addon" next-id and scoped it
  as a Brehon governance browser extension. Two AskUserQuestion rounds + a git
  history sweep across all `C:/Users/barri/Developer` repos revealed the extension
  is a **WXT Chrome MV3 extension in `agent-grey`** (the dual-screening project),
  on an unmerged worktree branch `feat/extension-phase2-one-click-add` — a
  different repo with its own `.claude/` harness, nothing to do with Brehon.
  **Surprise:** two repos with similar-looking `.claude/` harnesses + a terse
  cross-context handoff is enough to point a transition at the wrong repo.
  Caught before any wrong-repo artifact was written.

- **The transition skill's retro gate fired correctly on a fresh-ship phase.**
  T4b shipped minutes before "run transition session" — so no retro existed yet,
  and the gate stopped the skill cold (`feedback_retro_not_report.md`). This is the
  gate working as designed (don't close a phase without the outgoing advisor's
  reflection), but it's worth noting the friction shape: "ship → immediately
  transition" always trips the gate because the retro is a separate authoring
  step. Not a defect; an inherent ordering.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When a user request names a "track" / "add-on" / "feature" with a terse "run transition", verify the **repo it belongs to** before invoking any repo-specific orchestration skill — `git history` / `find` sweep across `Developer/*` repos is the cheap check. Don't assume a request in a Brehon session is Brehon work. Candidate lesson: `feedback_verify_repo_before_repo_specific_skill.md` (or extend `feedback_advisor_instruction_mismatch_stop_and_ask.md` with a "which-repo" row). | Prevents pointing a repo-native skill (transition, bm-*, auto-phase) at the wrong repo; one mis-scoped transition avoided per occurrence | minor (1 lesson row) | 1× this session — **single occurrence, not promoted; recheck on 2nd cross-repo confusion** |
| 2 | The transition retro gate + "ship then immediately transition" ordering means a just-shipped phase always lacks a retro. Consider a one-line note in the transition skill's Step 0 STOP message pointing at `/session-retro` as the unblock path (it already says "outgoing advisor must write the retro first" — naming the command makes the next step mechanical). | Removes a "what do I run to satisfy the gate?" round-trip | trivial (1 line in skill STOP text) | 1× here; low-value — defer unless it recurs |

## What to carry forward

- **Dry-run the dependency before wiring it into a cadence.** The drain dry-run
  cost ~3s and returned both "the script still works after 27 days" AND "here are
  13 rows proving the gap is real." For any "add X to the weekly/scheduled
  cadence" task, dry-run X first — it validates the dependency and quantifies the
  need in one step. (Same family as the T4a "dogfood the spec at write-time"
  carry-forward.)

- **Stop and search before running a repo-specific skill on an ambiguous track.**
  The browser-add-on detour resolved cleanly because I searched git history across
  repos *before* running the transition, not after. Carry forward: terse
  cross-context handoffs ("next up X, run the transition") are exactly the case to
  spend 30s confirming *which repo* X lives in. Per
  `feedback_falsifiable_hypothesis_before_structural_fix.md` (the premise of the
  request is a hypothesis until checked).

- **Honour the "small task → dormant apparatus" framing.** T4b's bootstrap said
  the heavy orchestration (cohort/validate-pending/bm-*) is dormant for this task,
  and it was. No Junior dispatch, no cargo, no PR — a single direct-commit skill
  edit. Resisting the pull to over-ceremony a 45-insertion skill edit kept it to
  one clean commit. Carry forward: match ceremony to task size (`feedback_principles_not_rules.md`).

---

## Per-role signals (four-role model)

T4b was an **advisor-side-only** task: a direct skill edit authored inline in the
advisor session, no Junior dispatch. The other three roles were dormant by design.

| Role | Activity this phase | Signal |
|---|---|---|
| **advisor** | Authored Step 2e, dogfooded the drain, committed direct-on-gov-v0, synced state | Clean. One detour (browser-add-on repo mis-scope) caught before cost. |
| **planning** | none (no plan needed — bootstrap §T4b was the contract) | dormant (correct) |
| **impl-task** | none (advisor authored the skill edit inline; not Junior work) | dormant (correct) |
| **bm-task** | none (no PR flow — `.claude/` meta-work direct-on-trunk) | dormant (correct) |
| **ci-watcher** | none (no cargo/CI) | dormant (correct) |

No per-role friction to harvest beyond the advisor row. This is the expected
shape for a `.claude/`-only single-edit phase.

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Read bootstrap + lane record + plan §T4b | 4 | 0 | low | Current; gave exact next-action |
| Drain script dry-run (dogfood) | 6 | 0 | medium | Confirmed live + surfaced 13 undrained rows (proof-of-need) |
| Skill edit (Step 2e + callout + frontmatter) | 8 | 0 | none | 3 edits, clean slot between 2d and 3 |
| Step-ordering / artifact-exists verification | 2 | 0 | none | grep heading order + test -f cited artifacts; no dup |
| AskUserQuestion (browser-addon scoping ×2) | 0 | 5 | high | Mis-scoped onto Brehon track before git-history sweep redirected to agent-grey |
| git history sweep (locate the extension) | 4 | 0 | medium | Found it in agent-grey, not brehon-fork; prevented wrong-repo transition |
| `/brehon-phase-transition` (retro-gate STOP) | 2 | 0 | low | Gate fired correctly on fresh-ship phase |

**Net:** the T4b work itself was ~20 min productive, near-zero waste. The
browser-add-on detour cost ~5 min of mis-scoped scoping, recovered by a git-history
sweep before any wrong artifact landed.

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. No task crossed the
heavy threshold (>55min runtime / >40min log silence / >8 files).

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Author + dogfood + commit weekly-review Step 2e | 1 | 1 | ~20 | <1 (interactive) |

No outliers; no carry-forward signals from complexity.

## Decisions to revisit

- **T5 hold condition.** T5 (end-to-end verify) stays HELD until ≥5 non-sentinel
  dispatches per role exist. The 13 rows surfaced this session are encouraging
  signal-volume growth — but they're unfiltered (sentinel vs real unknown until
  drained + per-role-counted). Worth a `/check-role-health` full run (not just the
  drain) at the next weekly-review to re-measure per-role non-sentinel counts
  against the ≥5 gate. role-customization parks at T4b until that gate is met.

- **The browser extension (agent-grey) is separate, real, unmerged work.**
  `feat/extension-phase2-one-click-add` @ `16fd1541`, 8 commits ahead of main,
  clean tree, Phase 2 built ("needs finalising + testing" per user). Not a Brehon
  artifact — recorded here only so the cross-context thread isn't lost. When picked
  up, it's a fresh agent-grey session with agent-grey's own harness.

---

## Promotion candidates (recurrence ≥ 2, or ≥ 1 here + ≥ 1 prior)

- [ ] Change #1 (verify-repo-before-repo-specific-skill): single occurrence;
      not promoted. Recheck on 2nd cross-repo confusion.
- [ ] Change #2 (name `/session-retro` in transition STOP text): low-value;
      defer unless recurs.

---

_Lessons consulted: `feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`, `feedback_dogfood_slash_command_specs.md`,
`feedback_falsifiable_hypothesis_before_structural_fix.md`._
