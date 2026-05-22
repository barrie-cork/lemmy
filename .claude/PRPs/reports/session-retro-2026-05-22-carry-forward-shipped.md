# Session retro — 2026-05-22 — carry-forward shipped

**Harness:** claude-code
**Session window:** 2026-05-22T09:05Z → 2026-05-22T09:20Z (~15 min — short follow-up segment, post-prior-retro pickup)
**Branch at start:** `a1b62280f` (`governance-v0`)
**Branch at end:** `180a0f9aa` (`governance-v0`)
**Files touched:** 2 (own commit) — `.claude/rules/advisor-orchestrator.md` + new `.claude/lessons/feedback_cross_session_commit_attribution_collision.md`
**Commits:** 1 (`180a0f9aa` — rebased on top of fed-in-c's `45ef46a7f`)
**Prior retro:** `session-retro-2026-05-22-parallel-subagent-dispatch.md`. This is a **third short segment** of the same wall-clock day, post-compact pickup.

## TL;DR

Acted on the three carry-forwards from the prior retro: extended `feedback_cross_session_commit_attribution_collision.md` to cover both Race A (index-staging) and Race B (working-tree-mutation), authored an in-repo mirror (the user-memory copy was the only existing one), and codified parallel sub-agent dispatch + verify-after-subagent in `advisor-orchestrator.md` §6. The push hit a non-FF rejection from fed-in-c's concurrent `45ef46a7f` merge — **the exact Race-B-adjacent surface the lesson describes**; clean rebase + post-rebase verify-by-grep dogfooded the §6.2 discipline live. No new surprises this segment; one new dogfood instance.

## What surprised us

### Advisor

- **The push-then-rebase live-dogfooded §6.2 in real-time.** I shipped a rule edit codifying "verify distinctive strings after any ref alignment" — then ten seconds later, hit a non-FF push, rebased, and ran exactly that verify-by-grep step on the post-rebase tree before re-pushing. Both files still carried their distinctive strings (`"Subagent delegation"` + `"Race B — working-tree-mutation revert"`) post-rebase because the concurrent commit (`45ef46a7f` = bm-runlog edit on a different file) didn't conflict. **Net:** an unintended dogfood pass on the just-shipped rule, ~30s of overhead, zero rework. The rule survives first-contact.

### Planning / Impl / BM

(N/A — no planning, impl, or BM verbs.)

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Consider when "verify-by-grep after rebase" should join §6.2 as a stated apply-case.** §6.2 currently triggers on "sub-agent edit + ≥30s gap." Today the same mechanism caught a clean-rebase scenario (no sub-agent involved). The pattern is: any tree-state alignment (FF, rebase, merge) on a shared-`.git/` topology can change unstaged work. Single occurrence today; defer formal extension until a 2nd applicable case. | Catches working-tree drift from FF/rebase at session-level, not just sub-agent dispatch. | minor (one sentence in §6.2) | 1× this segment (post-rebase verify); defer formal promotion. |

## What to carry forward

- **Stage immediately + verify-by-grep is dogfood-stable.** Used twice in this segment (after the initial Edit, and after the post-rebase ref alignment); both passes returned non-zero matches on first try. Mechanism is callable inline without ceremony.
- **The non-FF push is the lightest possible Race-B-class detector.** It costs ~10s of overhead, gives a clean rebase if no conflict, and forces the verify-by-grep step. Don't suppress non-FF rejections by reflexive `git pull` — let them surface; the rejection IS the signal that the world advanced.
- **Pre-existing frontmatter parse warnings in 5 lesson files** (`feedback_briefs_land_on_branch_worker_forks_from.md`, `feedback_cohort_dq_id_collision.md`, `feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md`, `feedback_daemon_refspec_excludes_meta_phase_branches.md`, `feedback_planner_dq_id_via_origin_not_daemon_local.md`) — surfaced by `sync-lessons-to-pmd.sh`. Not introduced by this session; technical debt for a separate sweep.

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Edit on `feedback_cross_session_commit_attribution_collision.md` (user-memory + in-repo mirror) | ~5 | 0 | none | Race-B sub-section landed first-try; cross-refs intact. |
| Edit on `advisor-orchestrator.md` §6 | ~7 | 1 | low | Initial Edit failed because file wasn't in cache; one re-Read (20 lines) recovered. |
| Stage-immediately + `git status` verify | ~3 | 0 | none | Pattern reflex. |
| Verify-by-grep (3 distinctive strings) | ~5 | 0 | low | All 3 strings present first try. |
| `git push` → non-FF rejection → `git pull --rebase` → re-verify-by-grep → re-push | ~2 | 1 | medium | Clean rebase; no conflict. The unintended live-dogfood (good surprise). |
| `sync-lessons-to-pmd.sh --db <canonical> --strict` + `backfill.js` against canonical DB | ~5 | 0 | none | 6 lessons synced, all embedded; 0 missing vectors. |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Carry-forward bundle commit | 2 | 1 | ~15 | n/a (interactive) |

Below all watchdog thresholds by design.

## Promotion candidates (recurrence ≥ 2)

- [ ] Change #1: extend §6.2 verify-by-grep to cover post-rebase / post-FF tree alignments. Recurrence 1; defer.

(Nothing else to promote this segment — the three carry-forwards from the prior retro are now shipped, which IS the promotion event for that retro's items.)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
