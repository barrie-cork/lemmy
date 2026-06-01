# Session retro — 2026-06-01 — git-tree-cleanup

**Harness:** claude-code
**Session window:** ~2026-06-01T14:00Z → ~2026-06-01T15:15Z (~75 min)
**Branch at start:** `f7c75127a` (`governance-v0`)
**Branch at end:** `6241ad832` (`governance-v0`)
**Files touched:** 4 (conformance audit report, roadmap.json, 2× handovers)
**Commits:** 3 explicit (conformance audit, roadmap state-reconcile, session retro from prior session)

## TL;DR

A housekeeping session: stash inspection and cleanup, git tree pruning (3 local branches,
11 remote phase branches, 203 junior worker branches, 4 chore branches), and one untracked
conformance audit report committed. Key surprise: one stash (`stash@{0}`) held a stale Phase
2 e2e DQ entry for a phase shipped months ago — recoverable only because we inspected before
dropping. The main carry-forward: a stash inspection ritual before dropping should be standard
(especially stash@{0} which is the most recently created and thus the most likely to be live).
The worktree removal for `brehon-fork-redaction-r1` soft-failed (locked by another process)
and required manual diagnosis — the `feedback_worktree_remove_force_for_submodules.md` lesson
didn't apply here; the actual failure was a process-lock on an empty directory.

---

## What surprised us

- **Stash `{0}` held a live-looking DQ entry for `v1-SL-d` Phase 2 e2e** — a phase shipped
  ~3 weeks ago, but the DQ entry (`id: 190`, `result: null`) was never resolved and never
  committed. It had been stashed during a pre-phase-checkout and forgotten. We verified against
  HEAD before dropping, which is correct, but there was no existing ritual enforcing this check.
  One misread could have prompted reopening a resolved phase.

- **`git worktree remove --force` returned `Permission denied` on an empty directory.** The
  directory had zero files inside (recursive count = 0) but a process held a handle on the
  directory itself (separate from file handles). `[System.IO.Directory]::Delete` confirmed the
  same. The git admin metadata (`.git/worktrees/<name>/`) had already been pruned, so `git worktree list`
  reported a clean state — the only issue was the orphaned filesystem directory. The lesson file
  `feedback_worktree_remove_force_for_submodules.md` was not the right reference; the actual
  failure mode is a process lock, not a submodule issue.

- **203 junior branches on origin** — larger than expected. Prior sessions had not been doing
  remote branch cleanup post-merge. The `git push origin --delete $(cat /tmp/junior-branches.txt | ...)`
  pattern worked cleanly in one shot.

- **`origin/phase-v1-quality-r3c` was already gone remotely** before we tried to delete it —
  GitHub auto-deletes merged PR branches when the repo setting is enabled. We hit `remote ref does
  not exist` on the first batch push. Should filter with `git ls-remote` before bulk-delete to
  avoid the spurious error.

- **pr-172-findings.yaml dirty state** — the file held fully-triaged triage decisions (all
  findings bucketed `wont-fix`/`done`) that had never been committed. This was viable state
  from a prior session's BM triage work that ended without a commit. Discarding was correct
  (PR #172 merged), but a committed record of final triage decisions is better practice.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add stash inspection step to session-start ritual in `advisor-orchestrator.md`: `git stash list && git stash show stash@{0} --stat` before any checkout or phase-transition | Surfaces forgotten stash entries before they're silently dropped; takes <5s | minor | 1× this session; 1× prior (stash@{1} was also a prior-session remnant) |
| 2 | Add `git ls-remote origin <branch>` pre-flight to any bulk remote-delete batch, or use `git push origin --delete` with `2>&1 \| grep -v "remote ref does not exist"` to suppress expected-not-found noise | Eliminates confusing error exit code on GitHub auto-deleted branches | minor | 1× this session |
| 3 | Add a note to `bm-merge.md` (or `phase-branch.md`) that the BM verb should commit the findings YAML at final triage (when all buckets are terminal) before session end, even if no follow-up is needed | Prevents loss of final-triage record on closed PRs | minor | 1× this session |
| 4 | Process-lock worktree cleanup: document in `feedback_worktree_remove_force_for_submodules.md` (or a new lesson) that an empty worktree directory that resists `Remove-Item` indicates a process handle — not a submodule issue. Diagnosis: `[System.IO.Directory]::Delete(path, true)` surfaces the process-locked error. Remediation: close other CC sessions, then retry. | Prevents mis-applying the submodule deinit recipe for an unrelated failure mode | minor | 1× this session |

## What to carry forward

- **Inspect-before-drop stash discipline:** always `git stash show stash@{N} -p` before
  `git stash drop`, even for stashes labelled "pre-recovery" or "unrelated dirty files" —
  the label can be wrong. Five seconds of inspection vs hours of recovery.

- **Remote branch list has upstream noise:** `git branch -r` on a fork includes all `upstream/*`
  and `origin/*` branches from the parent repo (LemmyNet/lemmy). Don't interpret the full list
  as "ours to manage" — filter by `origin/phase-v1-`, `origin/junior/`, `origin/chore/` prefixes
  only. The upstream branches are managed by LemmyNet, not by us.

- **The `git worktree prune` + `git worktree list` combination reliably detects orphaned admin
  state.** In this session, the worktree directory was empty but the admin entry was already
  pruned — `git worktree list` showed a clean single-worktree state before we tried the FS
  removal. This is the correct sequencing for diagnosis.

- **Bulk junior-branch deletion via tmpfile + xargs works cleanly at scale:** 203 branches
  deleted in a single push invocation with no failures. The pattern
  `git branch -r | grep "origin/junior/" | sed 's|origin/||' > /tmp/branches.txt && cat /tmp/branches.txt | tr '\n' ' ' | xargs git push origin --delete`
  is reusable.

- **Untracked conformance audit reports belong in git.** The `conformance-audit-*.md` files
  under `.claude/PRPs/reports/` are all tracked (not gitignored), and the one from this session's
  pre-brief prevention checkpoint was the only untracked file with lasting value. Correct action:
  always commit them before session end.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Stash inspection (manual) | 10 | 2 | medium | Identified v1-SL-d stale DQ entry; correctly dropped after verification. Surprise: stash was there at all |
| Remote branch bulk delete (junior/*) | 25 | 2 | low | 203 branches in one push. 2 min wasted on first pass hitting `remote ref does not exist` error |
| Worktree removal | 0 | 8 | high | `git worktree remove --force` failed on an empty directory due to process lock, not submodule state. Had to diagnose with `[System.IO.Directory]::Delete` |
| `git branch -r` remote audit | 5 | 3 | low | Upstream `origin/*` noise from LemmyNet fork required manual filtering |
| `gh pr list` to verify merged status | 3 | 0 | none | Clean signal for which branches were safe to delete |
| Conformance audit commit | 5 | 0 | low | Found untracked report that belonged in git; committed cleanly |

## Complexity scores (heavy tasks only)

No impl-tasks ran this session (git housekeeping only). N/A.

## Decisions to revisit

- **`brehon-fork-redaction-r1` directory** remains on disk, locked by another process. Will
  clean itself up when the other CC session (if any) closes. No action needed; worth a
  `Test-Path` check at next session start.
- **GitHub auto-delete on PR merge** — the repo appears to have this enabled (quality-r3c was
  already deleted when we tried). Good behaviour; we should rely on it and pre-filter bulk
  deletes accordingly.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Stash inspection before drop** (Change #1, recurrence 2×): promote note to
  `advisor-orchestrator.md` §1 "Polling loop" session-start ritual — add `git stash list`
  as a mandatory step alongside `git worktree list`.
- [ ] **Worktree process-lock vs submodule confusion** (Change #4): write
  `feedback_worktree_process_lock_vs_submodule.md` lesson distinguishing the two failure
  modes; link from `feedback_worktree_remove_force_for_submodules.md`.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
