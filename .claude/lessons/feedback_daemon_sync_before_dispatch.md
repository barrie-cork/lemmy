---
name: feedback_daemon_sync_before_dispatch
type: feedback
title: Sync daemon-local base_branch with origin before every Junior dispatch
description: Laptop pushes to origin (briefs, DQ answers, triage) without syncing daemon → worker forks stale → finalize-merge diverges → cascading DQ merge conflicts
tags: junior, daemon, pre-dispatch, git, dq-conflict
---

Before `mcp__junior-brehon__create_task`, always run:

```bash
ssh homeserver "cd /srv/brehon-fork && git fetch origin <base_branch> && git checkout <base_branch> && git merge --ff-only FETCH_HEAD"
```

**Why:** Workers fork from the daemon's LOCAL ref, not origin. If the laptop pushed commits to origin (briefs, DQ answers, findings YAML triage) without syncing the daemon, the worker starts from a stale point. When the worker's finalize-merge lands, it diverges from origin — every subsequent sync (daemon→origin push, governance-v0→phase merge-forward) hits a DQ merge conflict because both sides wrote to `decision-queue.json` from different base commits.

**How to apply:** Mechanically, before every `create_task` call. Cost is ~2s (SSH + ff-only merge). The cross-lane total cap check (`list_tasks(status="running")`) runs first; this runs second.

**Incident:** m2-late-b-actor bm-poll-cr Task #685 (2026-06-14). Laptop pushed bm-poll-cr brief to `origin/governance-v0` but did not sync daemon. Task #685 finalized 3 commits on the stale daemon-local governance-v0. Result: 3 cascading DQ merge conflicts across daemon→origin push, governance-v0→phase merge-forward, and origin/phase pull — ~10 min to resolve manually via Python union scripts.
