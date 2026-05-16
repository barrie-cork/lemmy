---
name: Junior daemon finalize agent merges a bm-cut branch into trunk
description: The generic Junior finalize agent treats a bm-cut/branch-creation worktree as a feature branch and runs `git merge --no-ff phase-<X>` INTO daemon-local trunk. Distinct from the finalize-SKIP shape (worker pre-push → no merge). Symptom = spurious content-empty merge commit on daemon-local governance-v0. Recover via `git update-ref`, not `git reset --hard`.
type: feedback
---

When a `[role:bm-task]` Junior task runs `bm-cut` (create a phase branch off `governance-v0`), the daemon's post-job finalize agent — whose generic instruction is "commit any worktree changes, then merge the worktree's feature branch into the base branch" — **wrongly merges the freshly-created phase branch back into daemon-local `governance-v0`**. bm-cut's entire purpose is to *create divergence from trunk*; the phase branch must NEVER be merged back. The result is a spurious `--no-ff` merge commit on the daemon-local trunk (e.g. `5317fa1fd "Merge phase-v1-AD-e into governance-v0 (bm-cut task)"`).

This is a **distinct failure shape** from the one in `feedback_junior_finalize_skips_when_worker_pre_pushes.md`. That lesson covers the *inverse*: an impl-task worker pre-pushes via `git push origin HEAD`, the daemon's local-delta check sees no work, and finalize SKIPS the merge (work stranded on the worker branch). Here the opposite happens: the bm-cut worker did NOT successfully push (the CC v2.1.119 `.claude/**` gate / the bm-cut.md "do not push" default blocked it), so the finalize agent saw a local branch with "unmerged commits" relative to trunk and MERGED it. Same daemon-finalize-fragility family, opposite direction. Read both lessons together when triaging any `done` bm-task whose trunk looks wrong.

**Why:** the daemon finalize agent's prompt is feature-branch-shaped ("merge the task's branch into the base branch"). It has a `[role:bm-task]` skip in some code paths but it is not reliably applied to bm-cut, and the agent's branch-vs-trunk delta detection treats `phase-<X>` (which has the same tip as trunk at bm-cut time, plus is a *named divergence point*, not a merge candidate) as "work to merge back". The phase branch IS the deliverable; merging it into trunk defeats the entire point of cutting it.

**Confirmed (recurrence 2×):**
- **v1-AD-e bm-cut, Junior #282, 2026-05-16.** Worker created `phase-v1-AD-e` at `09e0572cc` correctly. Finalize agent then ran `git merge --no-ff phase-v1-AD-e` into daemon-local `governance-v0` → `5317fa1fd` ("14 files, 1988 insertions" — but the diff vs `origin/governance-v0` was **content-empty**: every commit was already on both branches because the phase branch was just-cut from trunk). The merge never pushed (CC v2.1.119 gate / no-push-step blocked it); `origin/governance-v0` + the laptop checkout stayed pristine at `09e0572cc`.
- **v1-ship-1 bm-cut, 2026-05-16** (per that brief's §6 note + session-retro-2026-05-16-v1-ship-1-replan-bmcut-handoff.md): same CC v2.1.119 gate-block on the runlog; the finalize-merge shape recurred in the same window.

**Blast radius is usually contained — verify before panicking.** The spurious merge typically lives ONLY on the daemon-local `/srv/brehon-fork` `governance-v0`. It does not push (the same gate that blocks the runlog write, or the absence of an explicit push step in finalize, prevents propagation). Always check origin first: `git ls-remote origin refs/heads/governance-v0` (or compare laptop `origin/governance-v0` tip). If origin is unchanged and the merge diff vs origin is empty, the only damage is a daemon-local-only content-empty merge commit — small, fully recoverable.

**How to apply:**

- **Detection signal:** a `[role:bm-task]` bm-cut task shows `done`; daemon-local `governance-v0` has a `Merge phase-<X> into governance-v0 (bm-cut task)` commit at HEAD; `origin/governance-v0` is UNCHANGED at the pre-bm-cut tip; the merge's diff vs `origin/governance-v0` is empty (content-empty merge — all commits already on both sides).
- **Recovery — use `git update-ref`, NOT `git reset --hard`:** the daemon main checkout is typically *on* `governance-v0` at the moment you recover. `git update-ref refs/heads/governance-v0 origin/governance-v0` moves the branch pointer **without touching the working tree** — it does not switch the checkout, does not disturb untracked files (e.g. the in-flight `.claude/hooks/allow-prp-deliverables.sh`), and crucially does NOT race any concurrent Junior tasks running on *other* phase branches in *other* worktrees. `git reset --hard` would switch/clean the working tree and is the heavier, riskier instrument here.
  ```bash
  # Pre-snapshot for safety (the spurious merge SHA is recoverable from reflog anyway,
  # but an explicit backup makes "is this reversible?" trivially answerable):
  ssh homeserver 'cd /srv/brehon-fork && git rev-parse governance-v0 > /tmp/pre-reset-gov-v0-<jobid>.txt'
  # Confirm content-empty (diff vs origin must be empty):
  ssh homeserver 'cd /srv/brehon-fork && git diff origin/governance-v0..governance-v0 --stat'
  # Drop the spurious merge (working-tree-safe):
  ssh homeserver 'cd /srv/brehon-fork && git update-ref refs/heads/governance-v0 origin/governance-v0'
  # The phase branch IS the real deliverable — push it so the lane worktree can fork it:
  ssh homeserver 'cd /srv/brehon-fork && git push origin phase-<X>:phase-<X>'
  ```
- **Then recover the gate-blocked runlog advisor-side** (per the bm-cut brief §6 + `feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md`): write `.claude/runlog/<phase>-runlog.md` on `governance-v0` from the laptop, `docs(advisor):` subject (advisor relocating a gate-blocked BM deliverable — attribution-honest).

**Preventative (until the daemon finalize agent is bm-cut-aware):** every bm-cut brief MUST carry a §6 note instructing the worker that bm-cut creates divergence (NOT a feature branch to merge back), AND the advisor MUST verify daemon-local trunk post-bm-cut as a routine step (not an exception path). The bm-cut brief template's §6 / `.claude/commands/bm/bm-cut.md` "Refusal cases" should state this explicitly. The structural fix is the same as the planned `executor.ts` finalize-skip for `[role:impl-task]` (see `feedback_junior_finalize_skips_when_worker_pre_pushes.md` "Planned fix") — extend the `[role:bm-task]` finalize skip to reliably cover bm-cut, OR add a `FINALIZE: do-not-merge` worktree sentinel the finalize agent honours.

**Generalises to:** any daemon-driven worker pipeline where the generic post-work finalize step is feature-branch-shaped ("merge the task branch into base") but some task classes (branch-creation, tag-creation, divergence-point tasks) must NOT be merged back. The finalize agent needs per-task-class awareness; absent that, the orchestrator must verify the base branch post-task for those classes.

**Symptom to recognise:** `git log governance-v0 --oneline -1` on the daemon shows `Merge phase-<X> into governance-v0 (bm-cut task)` while `origin/governance-v0` shows the pre-bm-cut commit. That divergence — local trunk ahead of origin by exactly one content-empty merge after a bm-cut — is this bug, not legitimate work.

**Companion lessons:**
- `feedback_junior_finalize_skips_when_worker_pre_pushes.md` — the sibling (opposite) finalize-fragility shape; read both together.
- `feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` — why the runlog write is blocked on the same bm-cut task (the gate also incidentally contains this merge's blast radius by blocking its push).
- `feedback_multi_lane_worktree_discipline.md` — why `update-ref` (not a checkout-switching reset) matters when concurrent lane tasks share the daemon `.git/`.

**Where this bit:** `.claude/PRPs/reports/session-retro-2026-05-16-v1-ad-e-gate-bmcut-finalize-recovery.md` (the recovery walkthrough + the `update-ref`-over-`reset` rationale).
