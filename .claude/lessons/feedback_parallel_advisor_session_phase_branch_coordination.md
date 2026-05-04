---
name: Parallel advisor sessions on the same phase branch need per-action coordination
description: When two advisor sessions run on the same phase branch (e.g. one finishing impl/retro, one running PR/CR triage), per-action coordination via `.claude/agent-activity.json` is mandatory. Without it, retro pushes get rejected on phase-tip drift and recovery requires `git pull --rebase` plus re-pushes.
type: feedback
---
**Rule:** before opening a PR / dispatching a fix-impl / pushing any commit on an active phase branch, check `.claude/agent-activity.json` (per `session-awareness.md`) for any other session's `mode: write` claim. Surface the conflict before action; don't silently push and hope the merge-by-rebase wins.

**Why this matters:**
- v1-SL-a §3.6: two advisor sessions ran on `phase-v1-SL-a` simultaneously — session A finishing the Task 9 retro author commit (`ca2f84034`), session B opening PR #111 + running CR triage + dispatching fix-impl-4 + raising DQ #137 + merging (`17a63356c`, `5af6a55f9`, `96cc2ade2`). Session A's retro push was rejected (phase-tip drift); recovery via `git pull --rebase` + re-push. New phase tip after rebase: `c7908632d`.
- Cost: ~5 min recovery on session A side + the lost-mental-model cost of "what changed?" Both sessions had `mode: write` on the same branch. Neither checked agent-activity.json before action.
- This is **distinct from** impl-impl race (which is solved by Junior's per-task worktree isolation per `feedback_parallel_agents_one_worktree_per_agent.md`). Advisor sessions share the laptop's working checkout — there's no per-session worktree. Coordination is the only mechanism.
- The 4-role model (advisor + planning + impl + BM) didn't enumerate parallel-advisor as a concurrency mode. SL-a confirmed it's a real category.

**How to apply:**
- **Advisor session-start ritual extension:** `cat .claude/agent-activity.json` after the standard start-banner reads. If another session has `mode: write` on a Brehon phase branch, surface the claim immediately to the user before any state-changing action.
- **Per-action gate:** before any commit + push to a phase branch, repeat the check. Session-start is necessary but not sufficient — the other session may claim mid-flight.
- **Conflict resolution:** when a conflict surfaces, default is to YIELD to the active session and re-grain to a non-conflicting task. The yielding session can poll `agent-activity.json` periodically and resume when the active session's claim drops (typically when `bm-merge` completes or the session ends).
- **Push-rejected recovery (when the gate is missed):** `git pull --rebase` is correct first response; if commits are independent (e.g. retro vs CR triage on different paths), rebase succeeds cleanly. If commits touch the same file, manual conflict resolution; surface to user. Do NOT `git push --force` to reclaim the branch.
- **Forward gate (proposed v1-SL-a §7 follow-up #8):** automated `.claude/agent-activity.json` check + claim-write integrated into bm-cut / bm-pr / impl-task dispatch. Currently advisory; needs hook discipline.

**Generalises to:** any concurrent agent work on the same shared branch. Not just brehon-fork — any repo where session-awareness is advisory rather than enforced. The Tanglewood-Hive pattern applies: claim before write, release on completion or session end.

**Symptom to recognise:** a `git push origin <branch>` returns "Updates were rejected because the remote contains work that you do not have locally" AND you didn't expect new commits on the branch. If you didn't author them and they're not from a Junior worker, suspect a parallel advisor session.

**Retire when:** the brehon-fork tooling auto-claims phase-branch ownership on advisor session start + auto-releases on session end. Until then, manual per-action gate is the rule. Source: v1-SL-a retro §3.6 + §7 follow-up #8.
