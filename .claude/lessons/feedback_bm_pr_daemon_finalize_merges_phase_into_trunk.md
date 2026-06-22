---
name: feedback_bm_pr_daemon_finalize_merges_phase_into_trunk
description: A bm-pr Junior task whose worktree HEAD sits on the phase branch (not its gov-v0 base) gets its phase content finalized onto governance-v0 by the daemon, auto-marking the PR MERGED and skipping CR review + the merge gate.
metadata:
  type: feedback
---

A `bm-pr` Junior task is dispatched with `base_branch: governance-v0` and its
command spec (`bm-pr.md`) ends at "PR opened" — it NEVER runs `gh pr merge`
(merge is the separate `/bm-merge` verb, user-gated). Yet on 2026-06-22, bm-pr
task #772 resulted in PR #208 being **MERGED** into `governance-v0` ~3 minutes
after opening — bypassing gate-3 (CR triage) and gate-5 (merge confirm).
CodeRabbit's review aborted ("Review failed: pull request is closed") and the
governance-AI-review returned nothing — **every external reviewer was
short-circuited by the premature merge.**

**Root cause (not a rogue `gh pr merge`):** the daemon runs a finalize-merge
for EVERY Junior task — it merges the task's worktree branch back into the
task's `base_branch`. The bm-pr task wrote its `chore(runlog): … bm-pr #208
opened` commit onto a worktree HEAD that was sitting on the **phase branch tip**
(`c3707ebe9`), not on `governance-v0`. (The worktree contained phase content
because the phase branch had just been merge-forwarded with gov-v0, and the
bm-task's git ops landed on the phase HEAD.) When the daemon finalized job-772,
it merged that worktree — carrying the ENTIRE phase delivery — into
`governance-v0` (`5c6481880 chore(merge): finalize … bm-pr (job-772)`). GitHub
then saw `phase-m3-core-e2e-pilot`'s head reachable from `governance-v0` and
**auto-marked PR #208 MERGED** (`mergedBy` = the gh-auth identity, NOT an
explicit merge command).

**Why this is a trap, not a one-off:** any PR-producing Junior verb (`bm-pr`)
based on `governance-v0` is dangerous if its worktree HEAD ends up carrying
phase-branch content. The daemon's finalize-merge is unconditional; it does not
know that "this content should go through a PR + gates, not a direct
finalize-merge into trunk." The benign-looking runlog commit is the carrier.

**How to apply:**
1. **bm-pr must NOT leave phase content on its worktree HEAD.** The runlog
   commit (`bm-pr.md` Phase 6) must land on `governance-v0` only — never on a
   HEAD that has the phase branch as an ancestor. Either: (a) the bm-pr task
   writes the runlog to gov-v0 in a worktree guaranteed to be rooted at gov-v0
   (check `git merge-base --is-ancestor <phase-tip> HEAD` returns false before
   the finalize), or (b) bm-pr does NOT commit a runlog at all (move the runlog
   write to the advisor, post-PR-open).
2. **Advisor post-condition check after bm-pr (mandatory):** after a bm-pr task
   reports done, verify the PR is `OPEN` (not `MERGED`) before proceeding —
   `gh pr view <N> --json state`. A `MERGED` state after bm-pr is a gate-bypass
   catch-fire (per `feedback_bm_false_success_advisor_post_condition_catch`).
3. **Do NOT merge-forward the phase branch right before bm-pr in a way that the
   bm-task worktree inherits.** If a merge-forward is needed, do it, push, and
   ensure the bm-task forks a CLEAN gov-v0 worktree (daemon trunk sync of
   gov-v0, not phase).
4. The benign case (this incident): the merged content was validated-green
   (e2e gate + `/brehon-verify` ✓), so the OUTCOME was acceptable — but the
   PROCESS lost all external review. Treat as a breach regardless of outcome;
   the review value is the point of the PR flow (`phase-branch.md`).

Related: [[feedback_bm_false_success_advisor_post_condition_catch]],
[[feedback_bm_merge_unstable_admin_bypass]],
[[feedback_junior_finalize_merge_race_lossless_reconcile]],
[[feedback_daemon_local_trunk_stale_multi_lane]].
