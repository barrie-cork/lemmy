---
name: Verify squash-merge content at byte level, not by line count
description: When a PR is squash-merged, don't assume a commit's content is in the squash just because it's in the PR's commit list — verify by diffing actual file content
type: feedback
originSessionId: fb551263-c278-4af9-8a8b-af5e79166d8a
---
Rule: when a PR is squash-merged, do not claim a commit's content is in the squash based on (a) being listed in the PR's commit log, (b) line-count similarity, or (c) "this was before the merge therefore it must be in". Verify with `git show <squash-sha>:<file>` + `git show <commit-sha>:<file>` + diff.

**Why:** During Phase 5c setup 2026-04-18 I told the advisor that `d8c544ebe` and `6e74e5eaf` (Bucket 1 + flaky-timing) "ARE in the squash" at `6566dce43` because I saw `sponsor_liability.rs 340 lines` in the squash. Turned out the squash included an *earlier* version of the file (pre-bucket), not the bucket-fixed version. The squash rolled up commits only up to `b517fc577` (phase-5b-complete-report), and the 4 post-merge bucket commits (`d8c544ebe`, `6e74e5eaf`, `cbab85bb8`, `f5c771638`) were **genuinely missing** from `governance-v0`. This created a false-green branch-cut plan (decision-queue #17 Option A would have lost live bug fixes).

Caught by the advisor asking me to verify, and by `git status` on `governance-v0` showing 6 modified files from a partial restore that the advisor session had done — the modifications were *exactly* the bucket content missing from the squash.

**How to apply:** Any time a Brehon PR is merged (especially squash), before relying on "the fix is now on governance-v0", run at least one byte-level check:

```bash
git show origin/governance-v0:<file> | grep -n '<keyword from the fix>'
```

For the founder_seed filter: `grep 'founder_seed'`. If the keyword is missing, the fix is NOT on governance-v0, regardless of what the squash commit message says.

A safer pattern for any post-squash branch-cut decision is:

```bash
git diff origin/governance-v0 origin/phase-<N> --stat
```

If the stat shows non-trivial file changes, investigate each one before branching. Don't assume doc/rule-only diffs — I made that assumption in decision-queue #17 and the advisor caught it.

**Carry-forward for Phase 5c task 70:** when opening the 5c PR, use `gh pr merge --merge` (explicit `--merge`, not UI default) so the task-per-commit history survives to governance-v0. Squash-merge is the class of failure described here.
