---
name: Verify branch diff-vs-trunk before concluding code is missing
description: When a PR's net diff looks like it's missing expected code (a "phantom PR" — title/branch promises code the diff doesn't show), the branch's git log is NOT the diff-vs-trunk. Run `git merge-base --is-ancestor` + `git diff trunk...branch -- <file>` BEFORE concluding the code never landed. The code may already be on trunk (arrived via a merge-forward), making the branch's net contribution legitimately small. 2026-06-01 PR #172 incident: nearly re-dispatched an impl task to "re-land" e2e tests that were byte-identical on trunk already.
type: feedback
---

## TL;DR

A PR diff that appears to be "missing" expected code is a hypothesis, not a
fact. Before concluding the code never landed (and especially before
re-dispatching work to "re-write" it), spend ≤2 minutes verifying:

```bash
git fetch origin <branch> <trunk> --quiet
# Is the suspect commit's content already on trunk?
git merge-base --is-ancestor <suspect-sha> origin/<trunk> && echo "on trunk" || echo "not an ancestor"
# Does trunk's copy of the file already contain the symbol?
git grep -l "<expected_fn_or_symbol>" origin/<trunk> -- <path>
# Is the file actually identical between trunk and branch?
git diff origin/<trunk> origin/<branch> -- <path> --quiet && echo "IDENTICAL" || git diff --stat origin/<trunk> origin/<branch> -- <path>
```

If the file is **identical** between trunk and branch, the code is present —
it reached trunk first (typically via a merge-forward into the branch), so
the branch's *net* diff over trunk is legitimately small. There is no
phantom and nothing to re-dispatch. Re-dispatching would DUPLICATE code
already on trunk.

## Why this matters (PR #172 incident 2026-06-01)

PR #172 (`phase-v1-quality-r3c` → `governance-v0`) had a title promising
"sponsor-allowlist e2e sweep + BREHON_DISABLE_* guards," but
`gh pr view 172 --json files` showed a net diff of only 3 meta files
(`.coderabbit.yaml`, `decision-queue.json`, a brief) — **zero `crates/` or
`tests/` changes.** Copilot independently flagged the same thing ("title
claims code; diff is meta-only"). Combined with a recovery note saying
worker #558 was cancelled mid-compile, the obvious conclusion was: *the e2e
code never landed; re-dispatch impl-2 to write it.* A Task was created to do
exactly that.

That conclusion was **wrong**, and the user caught it. The branch's
`git log` showed the real test commits (`5d6e858a3 test(e2e): sponsor-allowlist
sweep`, `f7a47d6f5 test(e2e): BREHON_DISABLE_* guards`, plus 3 fix-impl
assertion fixes). The 30-second check inverted the RCA:

- `git merge-base --is-ancestor 5d6e858a3 origin/governance-v0` → **not an ancestor** (different SHA), which *looked* like confirmation the code wasn't on trunk…
- …but `git grep -l "test_brehon_disable_fed_replay_cleanup_job" origin/governance-v0 -- crates/server/tests/e2e.rs` → **returned the file** (symbol present on trunk).
- And `git diff origin/governance-v0 origin/phase-v1-quality-r3c -- crates/server/tests/e2e.rs --quiet` → **IDENTICAL**.

The resolution: the e2e code reached `governance-v0` first (content traceable
to `41c6e51ee` on trunk), then was merged *back into* the r3c branch via the
`0216babc9` pre-bm-pr merge-forward. So relative to trunk the test commits are
a no-op, and #172's net diff is genuinely just the 3 meta files. **Both
observations were true at once** — the branch did the work (its `git log`),
AND the diff-vs-trunk is meta-only (because the work already landed). Neither
contradicts the other.

**Cost avoided:** re-dispatching impl-2 would have re-added test functions
already byte-identical on trunk → merge conflict / duplicate-symbol churn,
plus a wasted Junior cycle (and the prior #558 attempt OOM'd the daemon).

## The trap

The error is conflating two different git questions:

| Question | Answered by | NOT answered by |
|---|---|---|
| "What did this branch *do*?" | `git log trunk..branch` | the diff |
| "What does this branch *add over trunk*?" | `git diff trunk...branch` | the log |

`merge-base --is-ancestor <sha> trunk` alone is also a trap: a commit can be
"not an ancestor" of trunk (its SHA isn't on trunk) while its *content* is on
trunk (carried in by a different commit, e.g. a squash, a cherry-pick, or a
merge-forward that re-applied the same hunks). **Content identity
(`git diff --quiet`), not SHA ancestry, is the authoritative test for "is this
code present on trunk."**

## When to apply

Any time you're about to conclude "the expected code is missing from this PR"
— especially before re-dispatching, re-writing, or filing a phantom-PR
finding as actionable. Triggering signatures:

- A PR/Copilot "title claims code the diff doesn't contain" finding.
- A net diff that's surprisingly small vs the branch's commit count
  (24 commits, 3 files changed).
- A recovery/handover note saying a worker was cancelled mid-task — strong
  prior for "code missing," which is exactly when to verify rather than trust
  the prior.
- A merge-forward commit (`chore(sync): merge <trunk> into <branch>`) anywhere
  in the branch log — that's the mechanism by which trunk's content becomes a
  no-op in the branch's diff.

## Relationship to the merge-forward flow

This is a *normal* consequence of the advisor-orchestrator merge-forward
discipline (`.claude/rules/advisor-orchestrator.md` §3.1: "before gate 5 →
merge-forward check → non-empty → checkout + merge + push"). After a
merge-forward, any code that was already on trunk shows up in the branch's log
(via the merge commit's ancestry) but contributes nothing to `diff
trunk...branch`. A near-complete sub-phase PR whose net diff is "just the
remaining meta files" is the *expected* shape at bm-pr time, not an anomaly.

## See also

- `feedback_falsifiable_hypothesis_before_structural_fix.md` — the parent
  pattern: a confidently-written RCA (DQ context, a phantom-PR finding) is a
  hypothesis; spend ≤30 min falsifying it before acting. This lesson is that
  pattern applied to branch state instead of code paths.
- `pattern_verify_before_trusting_shell_output.md` — `git log` ≠ `git diff`;
  both can mislead if you ask the wrong one.
- `.claude/rules/advisor-orchestrator.md` §3.1 — the merge-forward step that
  produces this diff shape legitimately.
