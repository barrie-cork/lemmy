# Session retro — 2026-05-16 — Shape G suspension + GH Actions audit

**Harness:** claude-code  
**Session window:** 2026-05-15 ~evening → 2026-05-16 ~morning (est. ~90 min interactive)  
**Branch at start:** `116e87db3` (`governance-v0`)  
**Branch at end:** `821cbe718` (`governance-v0`)  
**Files touched:** 10  
**Commits:** 32 this window (most authored earlier in the same calendar day; session-specific: `086cfa5d4`, `821cbe718`)

## TL;DR

User received a GitHub notification: 2800/3000 Actions minutes consumed. Session investigated root causes (private-repo 2× multiplier, Shape G firing on every `junior/*` push, refactor-tier 6-PR blitz triggering adr-compliance 12× in one day), wrote a structured audit report, and executed a suspension plan: three workflows disabled via API, `impl-task.md` and `impl-task-brief.template.md` updated to default to `validate-pending-laptop`, DQ #228 (log) + #229 (June 1 re-enable blocker) committed. **Key finding for future phases: the 3000 min/month free tier is structurally undersized for active Shape G development pace — plan upgrade or per-workflow optimisation before the next billing cycle.**

---

## What surprised us

- **Copilot code review cannot be disabled via API.** `gh api --method PUT .../workflows/265500620/disable` returns HTTP 422. It is platform-managed. The only control is preventing PR open/sync events. This was unknown — we assumed all workflows were API-disableable since adr-compliance confirmed it first.
- **The 6-PR refactor blitz was the proximate cause, not the background rate.** A normal sub-phase with 10 impl tasks consumes ~400 billable min from Shape G alone. The blitz added ~24 extra runs (12× adr-compliance + 5× Copilot + 5× cargo-validate-workspace) in a single day, crossing the threshold that would have been reached ~5 days later anyway. The structural problem is the tier size, not the blitz.
- **`validate-pending-laptop` machinery was already fully documented and operational** — `advisor-orchestrator.md §5.2` covers every edge case (Docker check for e2e, concurrent-cargo serialisation, Windows `.bat` invocation). No new rules were needed; just a template update and workflow disables. Suspension was faster to execute than expected (~15 min from plan approval to pushed commit).
- **DQ #229 was accidentally self-resolved in the same session** (commit `821cbe718` before the retro). The advisor noticed and committed `chore(decision-queue): advisor self-resolved DQ #229 — Shape G re-enable reminder 2026-06-01` but this is a schema breach — a blocker DQ placed as a future reminder should NOT be self-resolved the same session it was raised. A June 1 session will not see it in `pending[]`. See "What to change" #1.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Restore DQ #229 to `pending[]`** — it was prematurely self-resolved in `821cbe718`. Either revert the resolution or add a new DQ #230 blocker for June 1 re-enable. | June 1 session sees the re-enable reminder in `pending[]` as designed. | minor (one DQ edit + commit) | 1× this session |
| 2 | **Add a "Minutes remaining" advisory note to the advisor session-start ritual** — before any Junior dispatch, check `gh api repos/barrie-cork/lemmy/actions/workflows` and surface if any workflow has fired >50 times in the current billing period. A simple threshold warning prevents future exhaustion surprises. | Early warning before hitting the limit; less reactive. | medium (read `advisor-orchestrator.md` §1 Polling loop; add a new optional CWD check step) | 1× (first time this happened) |
| 3 | **Add a June 1 reversal brief template comment to the plan file** — the plan at `C:/Users/barri/.claude/plans/create-a-plan-to-zippy-lamport.md` has a "June 1 reversal checklist" section; promote that checklist into a DQ `pending` entry (not self-resolved) so it surfaces automatically. | Next session that opens DQ sees the reversal without reading the plan file. | minor | same root cause as #1 |
| 4 | **Structural: upgrade to GitHub Pro ($4/mo) before resuming next sub-phase** — at active impl pace (10+ tasks/phase, Shape G re-enabled), the free 3000 min/month exhausts within the first sub-phase. Pro adds 2000 min/month → 5000 total. At ~400 min/sub-phase from cargo-validate-workspace, that's ~12 sub-phases/month headroom. | Eliminates the budget constraint permanently for the current development cadence. | $4/month | structural (will recur every billing cycle at current pace) |

## What to carry forward

- **`validate-pending-laptop` as the suspension fallback is clean and complete.** The §5.2 handler covers all edge cases; impl-task.md's Pre-Shape-G section was already fully authored. Suspension required only: 3 API calls + 2 template edits + 2 DQ entries. If Shape G needs to be suspended again, the pattern is known.
- **Explore-agent parallelism for the investigation phase was effective.** Two Explore agents ran simultaneously (one for workflow mechanics, one for current phase state), returning compact summaries (~400 words each) without loading full rule files into the parent context. The investigation phase cost ~5 min wall-clock.
- **Audit report at `.claude/PRPs/reports/gh-actions-minutes-audit-2026-05-15.md` is a reusable reference.** Next time Actions minutes spike, the "Options to investigate" section is already framed. Load it instead of re-deriving.
- **AskUserQuestion before plan approval correctly surfaced the template-vs-manual override choice.** User picked "automatic" (update template); the question was genuinely load-bearing. This is the right gate for implementation choices that affect the Junior dispatch contract.
- **Copilot code review's 422 was surfaced immediately** (not after the commit was made) because the verify step ran before celebrating success. Worth repeating: always verify disabled state via `--jq '{name,state}'` before the commit.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Explore agents ×2 (parallel investigation) | 20 | 0 | low | Clean synthesis; adr-compliance workflow trigger + current DQ state in parallel. Effective. |
| AskUserQuestion (template vs manual override) | 5 | 0 | none | Right gate; single question; user answered cleanly. |
| Plan mode (ExitPlanMode) | 10 | 0 | none | Plan was tight (~5 decision points pre-clarified); approval was immediate. |
| `gh api --method PUT .../disable` (3 calls) | 15 | 2 | medium | adr-compliance confirmed API-disableable from earlier test. Copilot returned 422 — unexpected. 2 min to identify + document. |
| DQ #229 premature self-resolution | 0 | 5 | high | Advisor committed `self-resolved` on a future-reminder blocker in the same session that raised it. Process miss. See "What to change" #1. |
| GH Actions run-list analysis (Python one-liners) | 10 | 3 | low | `billableMs` field not in API — 3 min to discover and switch to counting runs as proxy. |

## Complexity scores (heavy tasks only)

This session had no Junior impl-task dispatches. The only "heavy task" was the plan-execute cycle for the Shape G suspension, which was advisor-side only.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Shape G suspension plan + execute | 4 | 2 | ~15 | ~3 |

Well within the safe envelope. No watchdog risk.

## Decisions to revisit

- **DQ #229 restore-to-pending:** needs a follow-up commit this or next session before June 1. If left as self-resolved, the June 1 reminder is lost. (See "What to change" #1.)
- **GitHub Pro upgrade timing:** ideally before resuming the next sub-phase (v1-ship-1-r1), which will immediately start generating `cargo-validate-workspace` runs again under Shape G. Upgrade is a ~2-min GitHub Settings action; should happen before any new Junior dispatch.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **DQ blocker raised and self-resolved same session:** promote to `feedback_dq_future_reminder_not_self_resolved.md` lesson — a blocker placed as a future reminder must survive past the originating session in `pending[]`. Do not self-resolve same session. (1× here; pattern risk for future reminder-style DQs.)
- [ ] **Actions minutes exhaustion audit report pattern:** promote checklist to `.claude/lessons/feedback_gh_actions_minutes_audit.md` — the audit report structure (billing multiplier, run-by-workflow counts, day-by-day table, options matrix) is reusable. (1× here; structural; high value if billing recurs.)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
