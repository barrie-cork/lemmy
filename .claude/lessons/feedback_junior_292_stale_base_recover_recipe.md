---
name: Junior #292 stale-base-self-merge + worker-exited-without-push — detect & recover recipe
description: Junior workers on a multi-lane phase fork from a STALE daemon-local phase ref, self-merge the prior abandoned worker, and would mass-delete every advisor file landed after the stale base. base_branch= on create_task does NOT prevent it. Plus a distinct variant — worker did+committed all work but exited without git push (distracted by the Stop-hook retro). Reliable recover recipe inside.
type: feedback
---

When multiple Brehon sub-phases are concurrently active (multi-lane), a Junior worker dispatched on `phase-v1-<lane>` frequently forks NOT from `origin/phase-v1-<lane>` but from the **daemon's stale local `phase-v1-<lane>` ref**, which lags origin. The worker then self-merges the prior (already-recovered, abandoned) worker's branch and produces a tree whose diff vs the live phase tip is a **mass deletion of every advisor file landed after the stale merge-base** (briefs, lessons, runlog, decision-queue.json, rules — 766 to 2819 deletions observed).

**`base_branch=phase-v1-<lane>` on `mcp__junior-brehon__create_task` does NOT fix this** — the daemon forks the worktree from its *local* ref, which the arg does not refresh. (Companion: `feedback_daemon_local_trunk_stale_multi_lane`, `feedback_junior_finalize_skips_when_worker_pre_pushes` — neither covers the stale-base mass-delete or the recover recipe; this lesson does.)

**Confirmed: 5× across v1-AD-e alone** (Junior #292 bm-pr -1910, #295 bm-poll-cr -766, #296 bm-triage -985, #297 fix-impl-1 -1496, #299 fix-impl-2 -2596, plus #301 bm-merge stale worktree). Effectively *every* Junior worker on the phase. It is the NORM on a multi-lane phase, not an exception.

## Detection (the unambiguous tells)

```bash
# 1. Is the live phase tip an ancestor of the worker branch? NO = stale base.
git merge-base --is-ancestor origin/phase-v1-<lane> origin/junior/<worker-branch> \
  && echo "clean base" || echo "STALE BASE (#292)"

# 2. Mass advisor-file deletions in the worker→phase-tip diff = the reversion signature.
git diff --stat origin/phase-v1-<lane> origin/junior/<worker-branch> | tail -5
#   → hundreds of deletions across .claude/PRPs/briefs, .claude/lessons,
#     .claude/runlog, .claude/decision-queue.json, .claude/rules = #292.
```

A `done`/`succeeded` Junior task whose worker branch fails test (1) and shows test (2) is the #292 pattern. **NEVER finalize-merge that worker branch** — it would revert advisor work.

## Recover recipe (reliable, exercised 5×)

1. Identify the **CLEAN** commit on the worker branch (the actual `fix(...)`/`chore(bm)` work — NOT the stale-base merge commit). `git log origin/phase-v1-<lane>..origin/junior/<worker> --oneline`; the clean commit's `git show --stat` must be exactly the expected file set (e.g. 1 file for a scoped fix; YAML+runlog for a bm verb).
2. Cherry-pick **ONLY** that commit onto the live phase tip:
   `git checkout phase-v1-<lane> && git cherry-pick <clean-sha>`.
3. Resolve any runlog add/add conflict as a **UNION** — keep the phase-branch superset, append the worker's new entry. Never `--theirs` the whole file (loses the advisor superset); never `--ours` it blindly (loses the worker's entry).
4. **Verify before push:** `git diff --stat <prev-phase-tip>..HEAD` must be ONLY the expected files, **ZERO advisor-file deletions**. This is the gate — if any `.claude/` deletion appears, the cherry-pick grabbed the wrong commit.
5. Push; verify origin survival (`git ls-remote` / re-read the file from `origin/phase-v1-<lane>`).
6. If the worker raised a `validate-pending-laptop` (or any) DQ, it references its **stale-base SHA**. Write a FRESH DQ pointing at the NEW on-branch cherry-picked SHA (compute next-id across live + archives + worker branches per `decision-queue.md`). Do NOT reuse the worker's DQ blob.

## Variant: worker-exited-without-push (distracted by the Stop-hook retro)

**Distinct from the deliberate Shape-G pre-push pattern.** Here the worker did all the work correctly AND committed it, but its final reasoning ("I should `git push origin HEAD`... let me push, then write the retro") got **distracted by the mandatory Stop-hook retro requirement and it exited WITHOUT running `git push`**, wrongly assuming Junior finalize would push (finalize skips on no-prepush). The daemon reports `succeeded`.

- **Tell:** worker branch ABSENT from origin + phase tip unchanged + no fix commit on origin, despite a `succeeded` task.
- **The work is recoverable** from the daemon worktree object store + reflog:
  ```bash
  ssh homeserver 'cd /srv/brehon-fork && git for-each-ref refs/heads/junior/ | grep <id>; \
    git reflog --all | grep -iE "<id>|<fix-sha>"; git cat-file -t <sha>'
  # Have the daemon push the containing SHA to an origin recovery ref
  # (exposes the objects, moves no daemon ref):
  ssh homeserver 'cd /srv/brehon-fork && git push origin <sha>:refs/heads/recovery/<task>-<id>'
  ```
  Then fetch `recovery/<task>-<id>` and run the recover recipe above (it will itself be a stale-base tree — cherry-pick ONLY the clean commit).
- Confirmed: Junior #297 (v1-AD-e fix-impl-1, 2026-05-17). Commit `835681ff5` intact in the daemon object store; recovered via a `recovery/` ref + clean cherry-pick.

## How to apply

- On a multi-lane phase, **budget the recover-cherry-pick as the EXPECTED path for every Junior worker.** Pre-stage this recipe into the stall-safety wakeup prompt so the next segment doesn't re-derive it.
- **NEVER trust the daemon's "done/succeeded"** — always run detection tests (1)+(2) before advancing the pipeline. A naive "task done → finalize-merge / advance" silently reverts advisor work or merges a phase tip missing the fix.
- `gh pr merge` is **server-side** — it can succeed even when the bm-merge worker's worktree is #292-stale (the merge operates on GitHub's PR state, not the local tree). Verify the merge via `gh pr view <N> --json state,mergeCommit`, not the Junior task status (which may report `failed` at daemon finalize while the merge actually landed — the #301 false-failure).

## Upstream fix (file a daemon-side ticket — the real cure)

Before each `git worktree add` for a task, the daemon MUST `git fetch origin <phase>:<phase>` (fast-forward the daemon-local phase ref to origin), OR fork the worktree directly from `origin/<phase>` instead of the local ref. Likely root cause of the daemon-local/origin divergence: the `RECOVERY-PASTE-elitedesk.md` `reset --hard` TOCTOU corrupting daemon-local refs (compounding with `feedback_daemon_local_trunk_stale_multi_lane`). Until the daemon fix lands, the recover recipe above is mandatory on every multi-lane phase.
