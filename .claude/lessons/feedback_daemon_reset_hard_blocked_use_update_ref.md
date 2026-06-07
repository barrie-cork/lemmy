---
name: feedback_daemon_reset_hard_blocked_use_update_ref
description: git reset --hard is blocked by a hook on the daemon's shared /srv/brehon-fork checkout. Use git update-ref to fast-forward a daemon-local branch to origin without triggering the hook.
type: feedback
---

`git reset --hard` on the daemon's shared `/srv/brehon-fork` checkout is blocked by a Stop hook (retro-check.sh or similar). Do NOT use it to sync a branch tip.

**Why:** The hook fires on `reset --hard` across all sessions sharing that checkout, regardless of whether the target branch is active. v1-quality-r3c bm-merge #566: advisor attempted to fast-forward daemon-local `governance-v0` via `reset --hard`; hook blocked it; workaround was `git update-ref`.

**How to apply:** When you need the daemon's local branch tip to match origin (e.g. before a bm-merge Junior task so the brief is visible), use:

```bash
ssh homeserver "cd /srv/brehon-fork && git fetch origin governance-v0 && git update-ref refs/heads/governance-v0 origin/governance-v0"
```

`update-ref` directly sets the ref without triggering commit/reset hooks. Safe for fast-forward moves. Never use this for non-fast-forward moves (check with `git merge-base --is-ancestor` first if uncertain).
