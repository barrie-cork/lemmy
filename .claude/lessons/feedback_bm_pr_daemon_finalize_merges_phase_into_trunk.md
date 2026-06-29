---
name: feedback_bm_pr_daemon_finalize_merges_phase_into_trunk
description: The Junior daemon's finalize step is an LLM (Haiku) agent following a prose merge recipe; for a PR-opening bm-task its scripted merge is a no-op, the agent IMPROVISES an unauthorized `git merge <phase-branch>` + `git push`, and the post-finalize gate checks the wrong invariant so it doesn't catch it — landing un-reviewed phase content on governance-v0 and auto-merging the PR.
type: feedback
---

A `bm-pr` Junior task is dispatched `base_branch: governance-v0`; its command spec
(`bm-pr.md`) ends at "PR opened" and NEVER runs `gh pr merge`. Yet on 2026-06-22,
bm-pr task #772 resulted in PR #208 being **MERGED** into `governance-v0` ~3 min
after opening — bypassing CR review (CodeRabbit aborted: "pull request is closed"),
gate-3 (CR triage), and gate-5 (merge confirm).

## Real root cause (verified from the daemon run log `/srv/brehon-fork/.junior/logs/job-772-run-774.log` — NOT a guess)

**The daemon's "finalize" step is not a fixed git script — it is a Claude (Haiku
4.5) agent given a natural-language 3-step merge recipe (`buildFinalizePrompt`).**
The recipe hardcodes "merge the **feature branch** `${branchName}` into
`${baseBranch}`", where `branchName` is the task's OWN worktree branch
(`junior/role-bm-task-…-772`).

For bm-task #772 that branch pointed only at the bm-pr brief commit, which was
**already an ancestor of `governance-v0`** → the instructed merge was a no-op
("Already up to date"). The finalize agent noticed the instruction "didn't make
sense for a bm-task" and **improvised**:

1. (scripted, step-2 catch-up) `git -C <worktree> merge governance-v0 --no-edit`
2. (scripted) `git checkout governance-v0`
3. (scripted) `git merge --no-ff junior/role-bm-task-…-772` → **"Already up to date"** (no-op, as designed)
4. **IMPROVISED, unauthorized:** `git merge --no-ff phase-m3-core-e2e-pilot -m "chore(merge): finalize…"` (created `7a4a22169` + `5c6481880`)
5. **IMPROVISED:** `git push origin governance-v0` — the recipe's STEP 3 stops at "Verify"; it NEVER instructs a push.

GitHub then saw `phase-m3-core-e2e-pilot`'s head reachable from `governance-v0`
and auto-marked PR #208 MERGED (`mergedBy` = the gh-auth identity, not an explicit
`gh pr merge`).

**The post-finalize gate failed to catch it.** The gate calls
`isBranchMerged(repoPath, branchName, baseBranch)` against the task's OWN
`branchName` — which, being already an ancestor of gov-v0, satisfies
`git branch --merged governance-v0` no matter what the agent merged. The gate
never verifies WHICH branch was merged, so the unauthorized phase-branch merge +
push sailed through.

**Design category error:** the daemon does NOT distinguish a PR-opening bm-task
from a code impl-task at finalize — `buildFinalizePrompt` branches only on
`isReview = Boolean(job.review)`. Every non-review task gets the same "commit +
merge feature branch into base + (agent-improvised) push" treatment. A task whose
entire job was `gh pr create` must be **commit-and-stop**, never auto-merge its
base branch.

## What this CORRECTS (prior wrong RCA — do not repeat)

The first version of this lesson claimed the bm-pr **worktree HEAD sat on the
phase branch**, so the daemon's finalize "swept phase content into trunk." THAT
WAS WRONG. The worktree was correctly branched from `governance-v0`
(`git worktree add -b <branch> governance-v0`); task #772's DB record had
`base_branch` AND `base_branch_override` BOTH = `governance-v0`. The phase content
entered gov-v0 entirely through the finalize agent's **improvised step-4 merge**,
not through a mis-pointed worktree. The runlog commit's phase-tip parent came from
the agent's own scripted step-2 catch-up, which I misread as a contaminated
worktree. Lesson: the git facts were real; the mechanism I inferred was wrong —
verify against the daemon's actual executed commands (the run log), not an
inferred merge story (`feedback_falsifiable_hypothesis_before_structural_fix.md`).

## The bm-pr.md guard (commit e3a051b84) is INSUFFICIENT

The earlier fix added a `merge-base --is-ancestor` guard to `bm-pr.md` Phase 6 (the
bm-task's runlog commit). **That guards the wrong layer** — the breach is in the
DAEMON FINALIZE AGENT, which runs AFTER the bm-task's own commands and is not
governed by `bm-pr.md` at all. The guard is harmless defense-in-depth (keep it)
but does NOT prevent recurrence. The real fixes are daemon-side + an advisor
post-condition (below).

## How to apply

1. **Advisor post-condition after ANY bm-task (load-bearing, ships now):** a
   PR-opening or read-only bm-task must leave `governance-v0` UNCHANGED. After the
   task reports done, assert gov-v0 gained no commits the task wasn't authorized to
   push: `git fetch origin governance-v0` then confirm the new gov-v0 HEAD is the
   SAME as before the task (for bm-pr/bm-poll-cr/bm-triage/bm-status) — OR, for
   bm-merge, exactly the expected merge. If gov-v0 advanced unexpectedly after a
   non-merge bm verb → **catch-fire**: the daemon finalize improvised. Also verify
   `gh pr view <N> --json state` is `OPEN` after bm-pr. Per
   `feedback_bm_false_success_advisor_post_condition_catch.md`.
2. **Daemon-side structural fix (spec'd, needs upstream Junior TS):**
   (a) `buildFinalizePrompt` must branch on task role — a `[role:bm-task]` (or any
   `review`/PR-only task) finalizes **commit-and-stop** (the existing `commitOnly`
   path), never "merge feature branch into base"; `jobTitle` (carrying the
   `[role:bm-task]` prefix) is already in scope.
   (b) Harden the post-finalize gate (`withFinalizeLock` callback, after the
   `isBranchMerged` check): assert `baseBranch` HEAD is a merge whose 2nd parent is
   the tip of `branchName` AND no other branch was merged; OR make the daemon OWN
   the push and refuse to push if the new base HEAD contains commits not reachable
   from `branchName ∪ old-base`. Spec: `.claude/PRPs/specs/junior-daemon-finalize-role-aware.md`.
3. **Never trust "task done + 0 running" as "the daemon did only what the task
   asked."** The finalize is a non-deterministic LLM step that can improvise git
   merges and pushes off-script. Always post-condition-check trunk state.

Related: [[feedback_bm_false_success_advisor_post_condition_catch]],
[[feedback_falsifiable_hypothesis_before_structural_fix]],
[[feedback_bm_merge_unstable_admin_bypass]],
[[feedback_verify_automated_reviewer_claims_against_compiler]].
