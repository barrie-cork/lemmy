---
name: Junior daemon finalize-merge gap (worker pre-push + advisor reset clobber)
description: Two failure shapes producing the same symptom (worker branch on origin but phase tip unchanged). (1) worker pre-pushes via `git push origin HEAD` — daemon's local-delta check sees no work; (2) advisor `git reset --hard origin/<branch>` between the daemon's local merge and its async push, clobbering the merge before it lands.
type: feedback
---

When a Junior `[role:impl-task]` worker runs `git push origin HEAD` to push its own worktree branch to origin (the standard Shape-G push-and-exit pattern), the daemon's post-job finalize-merge agent **does not see any work to merge** because by the time it runs, the local worktree's branch tip already matches origin. The daemon journal records `"No changes to merge, marking job as done"` and marks the job `done` without merging the worker's branch into the phase branch.

The worker's commits exist on origin under `junior/role-<slug>-<id>` but never land on `phase-v1-<phase>`, so Phase 2 (cargo-test-e2e on `phase-v1-*` push) cannot fire correctly until the advisor manually finalize-merges.

**Why:** the daemon's finalize agent detects work by checking for unmerged commits in the *local worktree directory*, not by checking for unmerged commits on origin's worker branch. When the worker pre-pushes via `git push origin HEAD`, the local check returns "no work" — even though origin has the work.

Confirmed: Brehon Task #35 (v1-JM-d Task 3, 2026-04-28). Worker did Parts A/B/C cleanly, ran `git push origin HEAD`, got `push exit: 0`. Daemon then logged `No changes to merge, marking job as done` and additionally `Failed to delete branch during cleanup`. The advisor had to manually `git merge origin/junior/role-impl-task-...-35 --no-ff` from the laptop to land Task 3 on phase-v1-JM-d.

A separate problem also surfaced: the daemon's finalize agent created a *local* merge with 30+ files of `cargo +nightly fmt` cleanup bundled in (commit `cc6dc4c` on EliteDesk — which never pushed because the push step is gated on the same defeated delta check). That fmt cleanup was discarded via `git reset --hard origin/phase-v1-JM-d`.

**How to apply:**

- **Shape G impl-tasks:** this gap is a known architectural limitation. The worker MUST push (for `workflow_run_id` capture), so the finalize gap is expected. The advisor detects and manually merges.
- **Pre-Shape-G impl-tasks:** do NOT push from the worker — let finalize handle it. If you push directly, the daemon will skip the merge.
- **Detection signal:** Junior task status `done`, worker branch on origin has commits, phase-branch tip on origin unchanged. Daemon journal shows `"No changes to merge, marking job as done"` + `"Failed to delete branch during cleanup"`.
- **Manual workaround:**
  ```bash
  # On laptop, from primary worktree of phase branch
  git fetch origin
  git merge origin/junior/<worker-branch> --no-ff
  git push origin <phase-branch>
  # On EliteDesk, discard daemon-side fmt noise
  ssh homeserver 'cd /srv/<repo> && git reset --hard origin/<phase-branch>'
  ```

**Planned fix (not yet shipped):** patch `executor.ts` to skip finalize for `[role:impl-task]` tasks (like the existing `[role:bm-task]` skip). Advisor-side merge via a `/brehon-finalize-merge` skill. See PMD #175 and `homeserver/.claude/plans/investigate-the-finalized-agent-eventual-cray.md` for the full investigation.

## Sibling shape: advisor `git reset --hard` clobbers a daemon merge that hasn't pushed yet

Confirmed 2026-04-28 during fix-3c Phase 2 e2e cleanup. **Different trigger, same outcome:** the daemon DOES finalize-merge into local `governance-v0` (e.g. ci-watcher Task #44 → merge commit `0dd9c65` parent `7f07253` + worker `1be9888`), but the daemon's push step is asynchronous and slow. If the advisor runs `ssh homeserver 'cd /srv/brehon-fork && git reset --hard origin/<branch>'` between the daemon's local merge and the daemon's push, the merge is wiped — `origin/<branch>` reflects the *pre-merge* tip, so the reset clobbers the in-progress finalize.

The lost commits are recoverable from `git reflog` (`HEAD@{N}: merge ...`); cherry-pick the worker-branch tip's mutation commit onto current `origin/<branch>` and push.

**How to apply:**

- **Before any `git reset --hard origin/<branch>` on EliteDesk:** check `git log HEAD --not origin/<branch> -5` first. If local has unpushed commits authored within the last ~2 minutes, the daemon may be mid-finalize. Either wait 60s and re-fetch, or recover via reflog after the reset.
- **`ci-watcher` Junior tasks are especially vulnerable** — they're fast (~2 min model-time) and the daemon finalize-merge usually lands seconds before the advisor's next sync attempt. The advisor's instinct to `git reset --hard` after merging an unrelated PR (e.g. PR #105 / #106 housekeeping) is the trigger.
- **Recovery recipe:**
  ```bash
  ssh homeserver 'cd /srv/brehon-fork && git reflog --since="10 minutes ago"'
  # Find the merge: HEAD@{N}: merge junior/role-...
  # Note its second parent (the worker-branch tip)
  ssh homeserver 'cd /srv/brehon-fork && git format-patch -1 <worker-tip> --stdout' > /tmp/recover.patch
  # On the orchestrator host (PC), on the trunk branch:
  git am /tmp/recover.patch
  git push origin <branch>
  ```

**Generalises to:** any daemon-driven worker pipeline where the worker has shell access and the post-work finalize step uses a "local delta" detection. Pre-pushing from inside the worker defeats the detection. **AND**: any orchestrator workflow where the advisor `git reset --hard`s a host whose state is being asynchronously updated by another agent. The reset is destructive against in-flight work; only run it when the host is known-quiescent.

## Sibling shape: multi-lane redundant-finalize divergence (daemon-local phase ref non-fast-forward)

Confirmed **twice in a single sub-phase** (v1-ship-1-r2, 2026-05-18 — commits `1c75769af` then `1e933207d`). When a worker pre-pushes its conflict-resolution merge to the worker branch AND the daemon's finalize agent ALSO runs (creating a *redundant* `fix(merge): ...` commit on its local phase ref *on top of* the worker's already-pushed merge), the daemon-local `phase-v1-<lane>` ref **diverges** from `origin/phase-v1-<lane>` — it is no longer a fast-forward. The next `/precheck` Check 3b refspec-fetch (`git fetch origin <branch>:<branch>`) is **REJECTED (non-fast-forward)**, blocking the next Junior dispatch on that lane.

This is the multi-lane recurrence of the pre-push gap above, with a different fix path because the daemon-local ref has *extra* commits (not just missing ones):

**Detection:** `/precheck` Check 3b reports `STALE`; the refspec-fetch fails `! [rejected] ... (non-fast-forward)`. `git merge-base --is-ancestor <daemon-local> origin/<branch>` returns NON-zero (daemon-local has commits not on origin).

**Recovery (MANDATORY zero-code-loss verification first, then user-gated):**
1. Identify the divergent commit: `ssh homeserver 'cd /srv/<repo> && git log --oneline -3 <branch>'` — it is typically a redundant daemon `fix(merge):` finalize commit on top of the worker's real merge.
2. **Prove zero code-loss:** `git diff --stat <daemon-local-sha> origin/<branch> -- crates/ migrations/ Cargo.lock Cargo.toml`. If this is **EMPTY**, the redundant finalize commit's code payload is byte-identical to origin (origin = worker's merge + advisor commits; daemon-local = same code + a redundant merge-topology commit). Only `.claude/` meta files differ (advisor commits made after the worker pushed). The reset is safe.
3. **Surface to user** with the tree-diff evidence (this is judgment-heavy — never silently force a cross-lane ref). Options: lane-safe `update-ref` / investigate / merge-from-laptop.
4. On user approval, **lane-safe primitive only:**
   - If the daemon checkout is on **another lane** (`git branch --show-current` ≠ `<branch>`): `ssh homeserver 'cd /srv/<repo> && git update-ref refs/heads/<branch> origin/<branch>'` — updates the ref WITHOUT switching the checkout. Then verify `git branch --show-current` is **unchanged** and the other lane's ref is untouched.
   - If the daemon checkout IS on `<branch>` AND it is a clean fast-forward (ancestor) AND no Junior is running: a plain `git pull --ff-only origin <branch>` on the daemon is safe and lane-appropriate (the daemon is already on this lane).
5. **HARD REFUSAL:** NEVER `git checkout <branch>` on the daemon to fix staleness when the checkout is on another lane — that is a `.claude/rules/multi-lane-worktree.md` hard-refusal-class destructive cross-lane op. The `update-ref` (divergent) or in-place `ff-pull` (clean-ff, daemon already on lane) are the ONLY correct primitives.

**Generalises further to:** any multi-lane daemon where finalize-skip (pre-push) and finalize-run (redundant merge) can BOTH happen on the same task, leaving the daemon-local ref ahead-by-redundant-topology rather than behind. The fix is never "force the ref on faith" — it is "prove the code payload is identical via tree-diff, surface the evidence, then lane-safe ref-update under user gate." Recurred 2× same sub-phase → if it recurs a 3rd time, escalate to a structural daemon-finalize fix (skip finalize for conflict-resolution impl-tasks the way `[role:bm-task]` is skipped), not just the recovery recipe.
