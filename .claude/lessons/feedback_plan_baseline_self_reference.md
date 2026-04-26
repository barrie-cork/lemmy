---
name: Plan baseline references are self-referential
description: Plan files that name a specific current-HEAD hash as the "expected baseline" self-invalidate the moment the plan file itself is edited on the base branch. Use ancestry checks, not exact-hash checks, in plan pre-flights.
type: feedback
originSessionId: 50cf5c6f-6b81-45b0-b3b4-45ac46b352d8
---
If a plan file names a specific git HEAD hash as the "expected baseline" for a pre-flight check (e.g. `must equal c51147754`), **that check self-invalidates the moment the plan file itself is edited on the base branch**. Any edit is a new commit, and the new commit advances HEAD past the named hash. The check now fails, and fixing it by naming the new hash just creates another new commit that fails again.

**Why:** Hit during Brehon Phase 2a setup (2026-04-15). The Phase 2a plan file had seven references to `c51147754` in its task-0 verification section. The plan commit itself (`6b7095daf`) advanced `governance-v0` past that hash without changing source. Updating the references to `6b7095daf` created `e89118201`, which made the newly-updated references stale again. A third edit to break the loop landed at `a77554f5c`. Three commits, all docs-only, all self-invalidating, until the check was reframed to not name any hash at all.

**How to apply:**

- **Use ancestry-based checks, not exact-hash checks.** `git merge-base --is-ancestor <stable-anchor> <moving-head>` where the stable anchor is a commit that will never change (e.g. a prior phase's merge HEAD), and the moving head is `governance-v0` or whatever the current base is. Ancestry checks are stable under any number of future docs-only commits.
- **The stable anchor should be a phase-boundary commit**, not "the current baseline." Phase boundary commits are defined once at phase close and never advance thereafter — they're the natural Archimedean point for pre-flight checks. Phase 1's merge HEAD `94eba51a0` is the anchor for all Phase 2+ pre-flights in the Brehon fork.
- **Also use sync checks, not hash checks:** `git rev-parse governance-v0 == git rev-parse origin/governance-v0`. This proves local and remote agree on the baseline without naming a specific value.
- **And relative checks for feature branches:** `git rev-parse feature/X == git rev-parse governance-v0` immediately after the branch cut. This proves the cut happened at the expected point without caring which point that is.
- **Three checks together** — ancestry + sync + relative — give a complete pre-flight verification that is stable under any future docs-only advancement of the base branch. This is the shape to use in every plan's task 0.

**Generalizes to:** any place where a plan, script, or memory file names a "current HEAD" that will be edited by the same mechanism that advances HEAD. Examples: plan files committed to the branch they'll be executed on, memory files that cite current-state hashes which a later session will update, scripts that hardcode expected commit SHAs. The rule is **don't hardcode a hash that the thing containing the hash will itself invalidate**.

**Symptom to recognise:** after updating a plan/memory/script to fix a stale hash reference, the very commit doing the fix makes the reference stale again. If you find yourself writing "bump hash from X to Y" commit messages, stop and reframe to ancestry or relative checks.
