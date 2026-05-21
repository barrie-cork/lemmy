# Session retro — v1-rls-r1 ship close-out 2026-05-21

**Phase shipped:** v1-rls-r1 (Recursive Learning System hardening Wave 1)
**PR:** #140 → merged at `16ede83b6` on `governance-v0`
**Tip:** `073a34040` (`chore(bm): merge PR #140 complete`)
**Session duration:** plan author 2026-05-20 → PR merge 2026-05-21 (~16 hours wall)

## What surprised us

- **BM Junior #385 ran clean (1m 50s, zero contamination, zero hang).** First Junior dispatch this entire phase that didn't require advisor recovery. The contrast vs Tasks 6 + 8 worker hangs + Task 9 worker fork-from-contaminated-base suggests the hang pattern correlates with `impl-task` long-running work (file authoring + PMD-write attempts + pre-push cargo). The `bm-task` shape (git ops + gh CLI + bm-runlog edit) is short, has no PMD write, and doesn't invoke cargo at all — much smaller hang surface. Possible structural insight for v1-rls-r2 retro: hang risk scales with worker context lifetime + MCP write attempts.

- **CodeRabbit's re-review on fix-impl-1 raised exactly one new finding (cr-9 anchored-relative-path), which CR's analysis-chain script verbatim showed it had reasoned about both my new code and the abstract failure case.** This is the kind of CR run where the value-per-token is high: real correctness bug in the configuration space we hadn't tested, with the proposed fix already minimal. The advisor's pattern of treating cr-9 as a Major-with-clean-fix rather than a "well-actually" rebut was the right call — total cost ~5 min advisor edit + ~5 min re-run dogfood probes + ~1 min commit/push.

- **The advisor-authoring fallback (Tasks 9-13 + fix-impl-1 + fix-impl-2) shipped 7 commits in ~80 min of advisor wall-clock.** Per the per-task complexity table in `.claude/PRPs/reports/v1-rls-r1-retro.md`, advisor-authored tasks averaged ~2-4 min per task — faster than the Junior round-trips even on clean runs (Junior best-case is ~3-5 min wall + finalize-merge latency). For zero-Rust meta-work, advisor-authoring is the cheaper path; the Junior round-trip's value is bm-task attribution + per-task isolation, not throughput.

## What to change

- **Add a "post-PR-open, advisor-authors-fix-impl when N findings ≤ M" criterion** to `feedback_advisor_authoring_under_daemon_stress.md`. v1-rls-r1's 11 fix-in-pr findings were authored advisor-side in one consolidated commit (cc1590904) + one follow-up commit (be93c4460). When the PR-review-cycle is short (1-2 fix-impl rounds expected, ≤15 findings total) AND meta-work-only, advisor-authoring the fix-impl chain beats dispatching N fix-impl Juniors. Add to the lesson's §"How to apply" bullet list.

- **Pre-cohort DQ id reservation should become a §4.1 prerequisite for `[P]` cohorts.** `feedback_cohort_dq_id_collision.md` mitigation 3 (advisor pre-authors N reserved DQ stubs in pending[] before dispatch) is mechanical and predictable. A v1-rls-r2 plan revision should make this the default for any `[P]`-marked cohort with N≥2 members. The 30-min Cohort A recovery + 5-way renumber commit was avoidable.

- **Dogfood checklist should run on the lane worktree BEFORE bm-pr.** Task 10's dogfood ran post-Task-9 advisor-authored work; it caught Task 7's function-after-exit bug pre-PR. But the cr-1 + cr-5 + cr-6 PR comments + the cr-9 cross-checkout fix would have been caught by a "run dogfood in the canonical brehon-fork checkout" probe — exactly what cr-9 was warning about. Future v1-rls-* dogfood reports should include sub-runs on BOTH the lane worktree AND the canonical checkout where applicable.

## What to carry forward

- **L14 REVISED post-merge runlog ordering works end-to-end.** PR #140's merge sequence: `gh pr merge --delete-branch` → `git pull --ff-only origin governance-v0` → `bm-runlog.md` append + commit + push. Zero conflicts. BM Junior #385 executed verbatim per the brief. v1-ship-1-r2's L14 self-conflict cost ~hours; the REVISED ordering saves it on every subsequent phase.

- **Lane-dedicated worktree pattern** (`brehon-fork-rls-r1`) kept human-side state clean throughout. All contamination was daemon-side. The lane teardown (`git worktree remove`) is the natural close-out per `multi-lane-worktree.md` lifecycle step 3.

- **Advisor-side gate-only inline checks (L15)** — running merge-gate read-only checks in this advisor session rather than dispatching a separate gate Junior eliminated ~one full Junior dispatch round-trip per gate. The pattern generalises beyond `/auto-phase`: any read-only pre-condition check should be inline.

## Per-PR-comment / fix-impl complexity (post-merge addendum)

- **fix-impl-1** (commit `cc1590904`): 11 fix-in-pr findings consolidated into one commit. 9 files changed, +62/-29. ~25 min advisor wall (read+plan+edit+probe+commit+push). 11 findings / 25 min = ~2.3 min per finding.
- **fix-impl-2** (commit `be93c4460`): 1 fix-in-pr finding (cr-9 anchored-relative-path). 2 files changed, +39/-2. ~10 min advisor wall (read CR analysis chain + extract helper + re-run 4 probes + commit + push).
- **3 PR comments** (cr-1 rebut + cr-5 + cr-6 wont-fix): ~5 min total. Posted via `gh pr comment --body-file <tempfile>`. Single user-gate-3 approval covered all three.

## Decisions to revisit

- **CR's cr-1 rebut on DQ #297 (prompt_hash schema as advisor-decided):** the rebut cited "ratified at plan-approval gate" — accurate for v1-rls-r1 because the plan was user-approved. For phases where the plan is more open-ended OR the DQ raises a schema NOT in the plan body, CR's heuristic ("schema-shape DQ → require user approval at resolve time") IS the right pattern. Worth a formal addition to `.claude/rules/decision-queue.md` "Attribution integrity" §: "Schema-shape decisions: advisor self-resolve OK if and only if the schema is named verbatim in the user-approved plan body; otherwise surface to user via AskUserQuestion before resolving."

- **PR `--repo barrie-cork/lemmy` flag discipline** worked perfectly this phase (BM Junior #384 + #385 both used the flag; no upstream-targeting mistakes). Worth keeping as a hard-refusal rule even as the team grows.

## Lessons promoted in this retro commit

No new lessons promoted in this session retro — the 3 lessons promoted in Task 13 retro (`feedback_advisor_authoring_under_daemon_stress.md`, `feedback_worker_hang_post_dq_raise.md`, `feedback_cross_lane_daemon_ref_contamination.md`) covered the in-impl observations. The post-merge observations above (BM Junior smooth run, dogfood-on-canonical sub-run, CR cr-1 schema-shape heuristic) are minor refinements better captured in the **existing** lessons' next-revision cycle.

## Next-up

- **Lane teardown (manual, user runs from canonical session):**
  ```
  cd C:/Users/barri/Developer/brehon-fork
  git worktree remove ../brehon-fork-rls-r1
  git branch -d phase-v1-rls-r1   # local cleanup; remote already deleted
  ```

- **Next-ready phase per MEMORY.md update:** `v1-federation-inbound-c` was already noted as next-ready before v1-rls-r1 started. With v1-rls-r1 shipped, fed-in-c is the resumption point.

- **Brehon-conformance-audit lane (`brehon-fork-conformance-audit`):** still active under a separate advisor session. No coordination action needed from this lane.
