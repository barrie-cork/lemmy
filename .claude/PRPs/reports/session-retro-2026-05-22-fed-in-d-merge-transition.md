# Session retro — 2026-05-22 — fed-in-d merge + transition

**Harness:** claude-code
**Session window:** 2026-05-22T18:00Z → 2026-05-22T20:30Z (~150 min, including context compaction boundary)
**Branch at start:** `2ad472025` (`brehon-fork-fed-in-d` / `phase-v1-federation-inbound-d`) — resumed post-compaction
**Branch at end:** `01ddb785a` (`governance-v0`) — transition commit pushed
**Files touched:** ~12 (verify report ×2, retro, DQ, runlog, bootstrap, MEMORY.md workflow-states ×2, bm-runlog, decision-queue)
**Commits:** 7 (governance-v0 chain: `ebfcf7aaf`, `ff65533ee`, `e2f44894b`[prior], `805370e73`[prior], `01ddb785a` + lane-side fixes)

## TL;DR

This session closed v1-federation-inbound-d: completed /brehon-verify Story 2 (e2e 103/0/5), executed the bm-merge inline when Junior #416 missed its brief (daemon stale-ref, 3rd+ recurrence), resolved a governance-v0 divergence conflict (v1-quality-r1 rustfmt), authored the four-role retro via background subagent, ran gate 6 (retro sign-off), and executed /brehon-phase-transition to bootstrap v1-federation-inbound-e. The headline finding is the daemon stale-ref pattern: it fired on BOTH the fix-in-pr and bm-merge workers in the same phase, confirming 3rd+ class recurrence with no structural fix yet shipped. The top change proposal is extending the PRE-PUSH MANDATE to BM-verb briefs explicitly, and adding a pre-dispatch daemon ref-currency check.

---

## What surprised us

- **Daemon stale-ref fired on BOTH BM-verb tasks in the same phase.** Junior #415 (fix-in-pr) and #416 (bm-merge) both declared success without having read their briefs, because the daemon-local ref lagged origin. Two different BM-verb types, same root cause, same session. Previously the pattern was documented for impl-task briefs; the BM-verb variant (brief on `governance-v0`, daemon-local `governance-v0` lags origin) was not explicitly documented as a distinct failure mode.

- **Pre-merge CONFLICTING from a concurrent reformatting PR.** The v1-quality-r1 rustfmt reformat (`2f13ffb80`) landed on `governance-v0` while fed-in-d was in flight. PR #146 was marked CONFLICTING. This class (concurrent reformatting CONFLICTING) was anticipated as a risk but had never actually fired before — it was the first real occurrence. The resolution (merge-forward on phase branch) worked cleanly but was not in any checklist.

- **Background subagent retro was high-quality on first pass.** The `general-purpose` subagent dispatched to author the four-role retro (`a247f47f5640e69c3`) produced a structurally sound retro with correct per-role signals, accurate complexity scores, and valid carry-forward bullets. Only one factual correction was needed (L14 runlog COMPLETE status — subagent said "outstanding" when it had already been committed). For a ~4k-token deliverable, the first-pass quality saved ~20 min of advisor authorship.

- **Retro subagent's L14 status error illustrates trust-but-verify discipline.** The subagent stated "L14 runlog COMPLETE entry outstanding — action for next session" when `ebfcf7aaf` had already committed it in this session. The error was caught only because the parent session checked `git log` before committing the retro. Without the verify step, a stale "action item" would have been committed to the historical record. This is exactly the trust-but-verify pattern from `feedback_bm_false_success_advisor_post_condition_catch.md` — it applies to subagent retro output as much as to BM Junior self-reports.

- **PR merge required advisor inline execution on the first try without re-queuing.** Because Junior #416 missed the brief (stale ref) AND the PR was CONFLICTING, the advisor had to: (1) merge governance-v0 into phase branch, (2) push resolution, (3) execute `gh pr merge` inline. Three sequential steps all done advisor-side in ~20 min. Compared to the previous fed-in-c cycle where bm-merge succeeded first-try (Junior #410), this was a significant regression — but the recovery path was clean and well-understood.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Pre-dispatch daemon ref-currency check:** before EVERY Junior task dispatch (impl AND BM-verb), run `ssh homeserver "cd /srv/brehon-fork && git fetch origin && git log --oneline origin/<branch> | head -3"` and verify the brief commit is visible | Catches daemon stale-ref BEFORE task creation rather than after Junior declares false success — eliminates the ~15-20 min of stale-task recovery per occurrence | minor (1 bash call per dispatch) | 3rd+ occurrence (class); 2× this phase (#415, #416) |
| 2 | **PRE-PUSH MANDATE extended to BM-verb briefs explicitly.** Add to bm-cut, bm-pr, bm-merge brief templates: "§4 Constraints: worker MUST verify it can read this brief's commit SHA from `origin/<branch>` before proceeding; if not visible, file `kind: blocker` DQ and stop." | Gives the worker a self-check gate rather than silently proceeding on stale state | minor (template text addition) | 2× this phase (BM-verb variant), recurs across phases as impl-task variant |
| 3 | **Governance-v0 divergence check added to advisor pre-gate-5 checklist.** Before surfacing gate 5 (merge confirm), run `git log --oneline origin/governance-v0 ^phase-v1-<phase>` and review for reformatting/structural commits. If non-empty: merge-forward before gate 5. Add to `advisor-orchestrator.md` §3.1 "bm-pr complete → gate 5" text. | Catches the CONFLICTING class before the user is asked to confirm a merge that will immediately fail | minor (one git command, one rule edit) | 1× this phase (first occurrence — watch for 2nd before promoting) |
| 4 | **Trust-but-verify discipline for subagent retro content, specifically: grep for all "action for next session" / "outstanding" / "TODO" phrases and cross-check against `git log`.** The background retro subagent produced one factual error of this class. | Prevents stale action items from being committed to historical retros | trivial (2-line grep after subagent returns) | 1× this session; pattern class recurs per `feedback_bm_false_success_advisor_post_condition_catch.md` |

## What to carry forward

- **Advisor inline execution as the reliable fallback when Junior BM-verb misses.** Both fix-in-pr (cr-1 mutex guard) and bm-merge (#416) were executed advisor-inline in this phase. The recovery path is now practiced and fast (~10-20 min each). The pattern: detect miss (no file change / PR still CONFLICTING / wrong self-report) → apply fix directly from lane worktree → push → verify → continue. This is not a failure of the four-role model; it's the designed fallback.

- **`feedback_bm_false_success_advisor_post_condition_catch.md` is load-bearing.** 6× confirmed across the project. Apply to ALL Junior task self-reports: check the actual artifact (file, PR state, branch tip, runlog append), not the self-report. Applied this session to both BM Junior tasks AND the background retro subagent.

- **Background subagent for retro authorship works.** The `general-purpose` subagent with a detailed prompt produced a high-quality four-role retro on first pass. Worth repeating for future phase retros that follow the same four-role shape.

- **Conflict resolution pattern for governance-v0 divergence (merge-forward).** When a PR is CONFLICTING due to a concurrent governance-v0 commit: checkout phase branch → `git merge origin/governance-v0` → resolve (`.claude/` files: `--ours`; Rust files: verify content then accept auto-resolution) → push → then execute `gh pr merge`. All three conflict files resolved cleanly in this session.

- **E2e baseline 103/0/5 held through fed-in-d.** The 5 ignored are pre-existing v0-polish TODOs. Any regression shows as a change to this number. Carry this baseline explicitly into fed-in-e.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Background retro subagent (`general-purpose`, `a247f47f5640e69c3`) | 20 | 3 | medium | High-quality first pass; one factual error (L14 status) caught by parent verify step |
| `/brehon-verify` Story 2 (e2e, resumed post-compaction) | 5 | 0 | none | DQ entry, verify report update, gate 5 surface — clean |
| Advisor inline bm-merge (PR #146) | 0 | 20 | high | Should have been a Junior BM-verb task; ended up advisor-side due to stale-ref + CONFLICTING. 20 min of recovery vs ~2 min for a functioning Junior task |
| Governance-v0 conflict resolution (merge-forward) | 0 | 20 | medium | First occurrence of rustfmt-reformat CONFLICTING class; resolution was methodical but unplanned |
| `/brehon-phase-transition v1-federation-inbound-d` | 25 | 0 | low | Smooth; all four steps executed cleanly; bootstrap file written in one pass |
| `/session-retro` (this invocation) | — | — | — | in progress |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) | Notes |
|---|---:|---:|---:|---:|---|
| Advisor inline fix-in-pr cr-1 (mutex guard) | 1 | 1 | ~10 | n/a | Clean single-file edit; lib test rerun |
| Governance-v0 conflict resolution | 3 | 1 | ~20 | n/a | Three conflict files; careful manual review |
| Advisor inline bm-merge (PR #146) | 1 | 1 | ~20 | n/a | gh pr merge + L14 runlog + L16 verify |
| Phase-2 e2e (background, bat wrapper) | 0 | 0 | ~39 | ~39 | 103/0/5. Background process; polling via ScheduleWakeup |

## Decisions to revisit

- **Daemon structural fix (git fetch before worktree add) is still unshipped.** DQ #338 covers the daemon wrong-ref class; the BM-verb variant is now documented in the fed-in-d retro but no structural fix has landed. This is the highest-ROI outstanding process fix. Consider prioritising a Junior task to ship the daemon-side fetch before fed-in-e dispatches impl workers.

- **Governance-v0 divergence check is a manual habit, not a rule.** The check (`git log --oneline origin/governance-v0 ^phase-v1-<phase>` before gate 5) should be added to `advisor-orchestrator.md` §3.1 explicitly so it's not forgotten when the advisor session is resumed from compaction context.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Pre-dispatch daemon ref-currency check (#1 above):** promote to `.claude/lessons/feedback_daemon_stale_bm_verb_brief_miss.md` (BM-verb variant of the daemon stale-ref pattern; complements `feedback_daemon_local_trunk_stale_multi_lane.md`). 3rd+ class recurrence across phases.
- [ ] **Governance-v0 divergence check (#3 above):** add to `advisor-orchestrator.md` §3.1 as a mandatory step at "bm-pr complete → before gate 5". 1× this session; watch for 2nd before full promotion.
- [ ] **Trust-but-verify for subagent retro output (#4 above):** extend `feedback_bm_false_success_advisor_post_condition_catch.md` to include subagent retro authorship as an explicit case. Pattern is the same class (self-report vs reality); extending the existing lesson is cheaper than a new file.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
