---
name: Daemon long-name junior/* refs not auto-created — fetch into explicit local ref for finalize-merge
description: The daemon's origin fetch does NOT create remote-tracking refs (refs/remotes/origin/junior/...) for very long junior/role-* branch names, so origin/<worker> resolves empty. Fetch into an explicit local ref before any finalize-merge or DQ-resolution push touching such a branch.
type: feedback
---

The daemon's `git fetch origin` does **not** auto-create remote-tracking refs (`refs/remotes/origin/junior/role-...`) for the very long `junior/role-<role>-<phase>-<slug>-<id>` branch names Junior generates. `git rev-parse origin/<worker>` resolves **empty** even though the branch exists on origin. Any finalize-merge or DQ-resolution that references `origin/<worker>` then fails silently (empty ref) or merges nothing.

**Why:** the branch names routinely exceed 120 chars (role + phase + brief-slug + task id). Some interaction between the daemon's fetch refspec config and the name length means the remote-tracking ref is skipped on the standard `git fetch origin <branch>` (which writes `FETCH_HEAD` but not `refs/remotes/origin/<branch>` for these). The branch tip is fetchable; the convenience ref just isn't there.

**How to apply:** for ANY operation that needs a `junior/role-*` worker tip — finalize-merge into a phase branch, a DQ-resolution merge, a diff-check — **fetch into an explicit local ref** rather than relying on `origin/<worker>`:

```bash
git fetch origin <full-worker-branch>:refs/heads/_fin<id>
git rev-parse _fin<id>          # now resolves
git merge --no-ff _fin<id> ...  # or diff-check, etc.
git branch -D _fin<id>          # clean up after
```

Make this the **default**, not a fallback discovered after `origin/<worker>` returns empty. It costs nothing when the remote-tracking ref happens to exist and always works when it doesn't.

**Recurrence:** 4× in m3-core-stage-mode alone (Task 5 finalize, Task 6 finalize, cr-4 code finalize, cr-4 DQ-resolution merge). Promoted from a per-phase candidate after the 4th. Pairs with `feedback_finalize_merge_where_to_look_first.md` (daemon-local-first push pattern) and `feedback_junior_292_stale_base_recover_recipe.md` (daemon bare-ref worktree-add quirk) — all three are daemon-side ref-resolution footguns the advisor hits at finalize time.

Clean up temp refs on both laptop and daemon at phase close (`git branch -D _tmp* _fin*` + `git worktree prune`).
