---
name: Detect parallel-agent commit collisions via gh pr view
description: Before committing work on a shared phase branch, verify origin's headRefOid matches local HEAD — parallel-agent races only show up via explicit fetch
type: feedback
originSessionId: e837bbbe-283f-4941-9ab4-afb178937118
---
When two impl sessions work the same phase branch, local `git log` does not show what the other agent has pushed. `gh pr view <N> --json headRefOid` returns origin's true HEAD and is the cheapest collision-detection probe.

**Why:** Phase 6 PR #46 review cycle (2026-04-19) hit this twice. Impl2 was about to commit #19 actor-binding fix; `gh pr view 46` returned `d706109808`, local was `41f1d0379`. Fetch confirmed another session (also authored "Barrie") had landed equivalent work. Same pattern repeated for #22. Without the `gh pr view` probe, Impl2 would have built duplicate commits, pushed, and required a 2-way rebase.

**How to apply:**

1. Before staging any commit on a shared phase branch, run `gh pr view <PR> --repo <fork> --json headRefOid` and compare against `git rev-parse HEAD`.
2. If they differ, `git fetch origin` and re-check.
3. If origin has commits touching files in your WIP, `git show <origin-sha> --stat` + `git show <origin-sha>` body to see if the work overlaps.
4. If it does overlap, compare origin's diff stats (`+34 -0`, `+35 -1`, etc.) against your staged diff line counts. Matching stats + matching commit-message style + matching rationale = duplicate work.
5. Safe discard via `git stash` (NOT `git checkout -- .`) preserves audit trail in case the collision analysis was wrong.

**Signal that the "other agent" is you-not-you:** both commits authored as the same git user (e.g. `Barrie`) because solo-dev single-author repo. Cannot discriminate by author. Discriminate by commit-subject pattern (matches brief's assigned finding) + diff scope (matches the files you were editing).

**Cheap probe cadence:** `gh pr view` on session start + before every commit. Cost ~1 API call; catches the class of bug that costs minutes to unwind.
