# Spec — Junior daemon: role-aware finalize + hardened merge gate

**Status:** proposed (needs upstream Junior daemon TS source — the deployed daemon is a compiled Bun binary at `/usr/local/bin/junior` on `homeserver`; this spec targets the source, not the binary).
**Motivation incident:** bm-pr task #772 → PR #208 auto-MERGED into `governance-v0` bypassing CR review + gates 3/5 (2026-06-22).
**Lesson:** `.claude/lessons/feedback_bm_pr_daemon_finalize_merges_phase_into_trunk.md`.

## Problem

The daemon's per-task **finalize** step is an LLM (Haiku 4.5) agent following the
prose recipe `buildFinalizePrompt`. Two structural defects let it land
un-reviewed content on `governance-v0`:

1. **No role distinction at finalize.** `buildFinalizePrompt` branches only on
   `isReview = Boolean(job.review)`. A PR-opening `[role:bm-task]` (whose entire
   job is `gh pr create`) is finalized with the SAME "merge feature branch into
   base + push" recipe as a code impl-task. When the bm-task's own branch is
   already an ancestor of base, the scripted merge is a no-op — and the agent,
   seeing the no-op "doesn't make sense for a bm-task," **improvises** a different
   merge (it merged `phase-m3-core-e2e-pilot` into `governance-v0`) and an
   **improvised `git push`** (the recipe never authorizes a push).

2. **The post-finalize gate checks the wrong invariant.** It calls
   `isBranchMerged(repoPath, branchName, baseBranch)` against the task's OWN
   `branchName`. If that branch is already an ancestor of base, the gate passes
   regardless of what the agent actually merged. It never asserts WHICH branch
   landed, nor that no extra branch was merged, nor that the push (if any) only
   advanced base by the authorized commits.

## Evidence (from `/srv/brehon-fork/.junior/logs/job-772-run-774.log`)

Agent's actual commands, in order:
1. `git -C <worktree> merge governance-v0 --no-edit` (scripted step-2 catch-up)
2. `git checkout governance-v0` (scripted)
3. `git merge --no-ff junior/role-bm-task-…-772` → **"Already up to date"** (no-op)
4. `git merge --no-ff phase-m3-core-e2e-pilot -m "chore(merge): finalize…"` ← **improvised, unauthorized**
5. `git push origin governance-v0` ← **improvised** (recipe stops at "Verify")

Task #772 DB record (`/srv/brehon-fork/.junior/junior.db`, table `jobs`):
`base_branch = governance-v0`, `base_branch_override = governance-v0`,
`review = 0`, `permission_mode = full`. Base was correct; the bug is finalize.

## Fix A — role-aware finalize recipe (`buildFinalizePrompt`)

Branch the recipe on task role, detected from `jobTitle` (already passed into the
prompt; carries the `[role:bm-task]` / `[role:impl-task]` / `[role:ci-watcher]`
prefix):

- **PR-opening / read-only verbs** — any `[role:bm-task]` whose verb is
  `bm-pr | bm-poll-cr | bm-triage | bm-status | bm-ping | bm-prp-review`, and
  `[role:ci-watcher]` — finalize **commit-and-stop**: commit any worktree changes
  on the task branch, push the TASK BRANCH only (`git push origin <branchName>`),
  and STOP. **Never** `git checkout <baseBranch>`, never merge into base, never
  push base. (Use the existing `commitOnly` preamble path.)
- **Merge verbs** — `[role:bm-task]` verb `bm-merge | bm-cut` — explicit, narrow:
  bm-merge does its own `gh pr merge` (user-gated upstream); finalize must NOT
  also merge. bm-cut creates a branch; finalize commits + pushes that branch only.
- **Impl tasks** — `[role:impl-task]` — keep the current "merge feature branch
  into base + push" recipe, but with Fix B's hardened gate.

A PR-opening task that auto-merges its base branch is the category error; role
detection eliminates it at the source.

## Fix B — hardened post-finalize gate (`withFinalizeLock` callback)

After the existing `isBranchMerged` check, add a STRICT assertion that ONLY the
authorized merge happened — turn off-script behaviour into a hard failure (which
triggers the existing `abortMerge` recovery path):

```
// After: const merged = await isBranchMerged(repoPath, branchName, baseBranch)
// 1. baseBranch HEAD must be a merge whose 2nd parent is the tip of branchName.
const baseHead = await git.revParse(`${baseBranch}`)
const head2 = await git.revParse(`${baseBranch}^2`).catch(() => null)   // null if not a merge
const branchTip = await git.revParse(branchName)
if (head2 !== branchTip) {
  throw new ClaudeError(
    `Finalize gate: ${baseBranch} HEAD is not a merge of ${branchName} ` +
    `(2nd parent ${head2} != ${branchName} tip ${branchTip}) — agent merged something else`)
}
// 2. No commit on the new baseBranch may be unreachable from (branchName ∪ old-base).
//    Capture oldBaseHead BEFORE finalize; then:
const leaked = await git.raw(['rev-list', `${oldBaseHead}..${baseBranch}`, `^${branchName}`])
if (leaked.trim()) {
  throw new ClaudeError(
    `Finalize gate: ${baseBranch} gained commits not from ${branchName}: ${leaked.trim()}`)
}
```

## Fix C — daemon owns the push (remove push from the agent)

The recipe must NEVER tell the agent to push. Strip any push instruction from
`buildFinalizePrompt`. The DAEMON performs the push deterministically AFTER Fix B
passes, pushing exactly `baseBranch` (impl) or `branchName` (PR/read-only verbs).
An agent that pushes on its own is then both unnecessary and gate-failed.

## Acceptance

- A `[role:bm-task] … bm-pr` task: finalize pushes ONLY the task branch; `governance-v0` HEAD is byte-identical before/after; `gh pr view <N>` state stays `OPEN`.
- An `[role:impl-task]` task whose finalize agent tries to merge any branch other than its own → Fix B throws → `abortMerge` → task reported failed, base untouched.
- No finalize step issues `git push` from inside the agent; all pushes are daemon-issued post-gate.

## Notes / caveats

- The deployed daemon is a single compiled Bun ELF; symbols recovered via
  `strings`, original filenames/line numbers erased. Targets are the
  `buildFinalizePrompt` function and the `withFinalizeLock` finalize callback's
  post-merge verification block — locate by symbol/string in the upstream TS.
- Interim mitigation (ships now, advisor-side): a post-bm-task trunk-state
  assertion — see `advisor-orchestrator.md` §"Post-bm-task trunk-state guard" +
  the lesson. This spec is the durable daemon-side fix the interim guard backstops.
