---
name: feedback_bm_merge_daemon_local_ref_staleness
description: Before dispatching bm-merge, verify the daemon's local governance-v0 tip matches origin. A stale daemon-local ref causes "fatal: path does not exist in governance-v0" when BM tries to read the brief.
metadata:
  type: feedback
---

bm-merge dispatches a Junior bm-task worker on the daemon. The worker reads the brief from `governance-v0` as seen by the daemon-local `.git`. If the advisor committed and pushed the brief to `origin/governance-v0` but the daemon's local `governance-v0` ref is still behind, the worker gets a "fatal: path does not exist in 'governance-v0'" error and either silently fails or raises a DQ.

**Why:** v1-quality-r3c bm-merge #566: advisor's brief `84e297b46` was on `origin/governance-v0` but daemon-local was at `fa091be00`. The bm-task worker spawned off the stale daemon ref, couldn't find the brief, and the task showed `done` with no merge performed. ~20 min lost diagnosing.

**How to apply:** Before every bm-merge brief dispatch, run:

```bash
ssh homeserver "cd /srv/brehon-fork && git log governance-v0 -1 --oneline"
```

Compare to `git log origin/governance-v0 -1 --oneline` locally. If they differ, fast-forward the daemon:

```bash
ssh homeserver "cd /srv/brehon-fork && git fetch origin governance-v0 && git update-ref refs/heads/governance-v0 origin/governance-v0"
```

See [[feedback_daemon_reset_hard_blocked_use_update_ref]] — do NOT use `git reset --hard` on the daemon shared checkout; hooks block it.

This check should be added as a mandatory step in the bm-merge brief template's §4 (Pre-merge state).
