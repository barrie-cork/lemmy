---
description: BM — cut a new phase or plan branch off governance-v0 trunk
argument-hint: <branch-suffix> (e.g. "v1-AD-e" → phase-v1-AD-e; "v1-AD-e --plan" → plan/v1-AD-e)
disable-model-invocation: true
---

# /bm-cut — cut phase or plan branch

**Input**: $ARGUMENTS

**Dispatcher → `branch-manager` subagent.** This command delegates
execution to the `branch-manager` subagent, which runs in its own
isolated context window. The subagent reads the operational script
below and follows it step by step.

Invoke:

> Use the `branch-manager` subagent to run `bm-cut`. Arguments:
> $ARGUMENTS. Follow the phases in `.claude/commands/bm/bm-cut.md` —
> parse the arg into a branch name, verify trunk cleanliness, verify
> the corresponding plan file exists on trunk (for phase branches),
> cut the branch locally without pushing. Return a "Branch cut" summary
> followed by a "Next suggested" line.

The parent (impl) session should call `Agent(subagent_type="branch-manager", model="haiku", prompt=<the above>)`. Mechanical local-only write — Haiku 4.5 is sufficient; regex branch-name check + 3 precondition gates.

---

## Operational script (for the subagent)

The BM agent cuts a new branch off `governance-v0` for a sub-phase
or plan. Verifies trunk is clean and up-to-date first, cuts the
branch locally (Phase 3), then pushes with `-u` to origin (Phase 4)
so the advisor's polling loop can see the phase branch and the
worker can fork from it. Auto, no prompt (local-only-write + one
remote push per `.claude/rules/branch-manager.md` autonomy table:
"Push a `phase-*` or `plan/*` branch to origin (`git push -u`) — Auto").
<!-- cr-1 (closes #88): wording aligned with the actual command flow —
     `git checkout -b` does not set upstream; `git push -u` in Phase 4 does. -->
<!-- 2026-05-25 friction-fix C: previous wording claimed branch was
     "local-only at this stage" with "Phase 5 below pushes". Phase 5
     was an output block, no push happened in-script — every bm-cut
     brief overrode and instructed push, so the script and practice
     diverged. Phase 4 now does the push; Phase 5 is runlog; Phase 6
     is output. -->



**Reads:** `.claude/rules/branch-manager.md`, `.claude/rules/phase-branch.md`.

---

## Phase 0 — Parse arguments

| Input | Branch name | Type |
|---|---|---|
| `v1-AD-e` | `phase-v1-AD-e` | phase |
| `v1-AD-e --plan` | `plan/v1-AD-e` | plan |
| `v1-AD-e --chore lint-cleanup` | `chore/lint-cleanup` | chore |

Valid name patterns (broadened 2026-05-20 to match production usage —
`phase-v1-federation-inbound-b`, `phase-v1-rep-tuning-r1`, and
`phase-brehon-conformance-audit` all break the historical narrow regex):

- **Phase branch:** `^phase-[a-z0-9][a-z0-9-]*$`
  - Examples: `phase-v1-SL-c-2`, `phase-v1-federation-inbound-b`,
    `phase-v1-rep-tuning-r1`, `phase-brehon-conformance-audit`,
    `phase-v1-AD-e`.
  - Convention: `phase-v<N>-<area>-<letter>` for V1 sub-phases;
    `phase-<slug>` for meta-tooling (audit / harness / cross-cutting).
- **Plan branch:** `^plan/[a-z0-9][a-z0-9-]*$` (mirrors phase shape).
- **Chore branch:** `^chore/[a-z0-9][a-z0-9-]*$` (one-shot meta-work
  that direct-commits to trunk per `phase-branch.md`).

If the branch name doesn't match one of the three patterns above,
**STOP** and ask the user to clarify the intended name.

---

## Phase 1 — Verify trunk state

```bash
git fetch origin
git status --short
git branch --show-current
git log governance-v0..origin/governance-v0 --oneline | head -5
git log origin/governance-v0..governance-v0 --oneline | head -5
```

**Decision tree:**

| State | Action |
|---|---|
| Working tree dirty | STOP: "Trunk is dirty — commit or stash first." |
| `governance-v0` behind `origin/governance-v0` | Run `git checkout governance-v0 && git pull --ff-only origin governance-v0`, then re-check. |
| `governance-v0` ahead of `origin/governance-v0` | STOP: "Trunk has unpushed commits — clarify before branching." |
| Currently on a `phase-*` or `plan/*` branch with commits ahead of trunk | WARN: "You're on {branch} with N unpushed commits — proceed cutting {new}? confirm" |

---

## Phase 2 — Verify the plan (for phase branches only)

A phase branch needs the corresponding plan file on trunk. Resolve
the plan file:

- `phase-v1-AD-e` → expect `.claude/PRPs/plans/v1-AD-e*.plan.md` on trunk

```bash
git checkout governance-v0
ls .claude/PRPs/plans/ | grep '<phase-suffix>' | head
```

If no plan file exists for a phase branch, **STOP** and write a
decision-queue entry:

```json
{
  "from": "bm",
  "question": "BM was asked to cut phase-{name} but no .claude/PRPs/plans/{suffix}*.plan.md exists on governance-v0. Plan PR not yet merged?",
  "options": ["wait for plan PR to merge", "cut anyway (override)"]
}
```

---

## Phase 3 — Cut the branch (local)

```bash
git checkout -b <branch-name> governance-v0
```

Phase 3 is local-only — the push happens in Phase 4. This split exists
so a refusal in Phase 3 (e.g. branch already exists locally) doesn't
leave a remote ref behind, and so the runlog can be written between
the cut and the push as a recovery point.

If `<branch-name>` already exists locally (e.g. a worktree was created
earlier by `/roadmap-next`), `git checkout -b` will fail. In that case
verify the existing branch matches the expected base (`git log
governance-v0..<branch-name> --oneline` shows only expected commits or
none), then `git checkout <branch-name>` (without `-b`) and proceed to
Phase 4. Do NOT delete and re-cut without confirming the existing
branch is unreferenced — it may have a worktree, a CI run history, or
a runlog citation.

---

## Phase 4 — Push to origin with upstream tracking

```bash
git push -u origin <branch-name>
```

After the push:
1. Verify the branch exists on origin:
   ```bash
   gh api repos/barrie-cork/lemmy/branches/<branch-name> \
     --jq '.name + " @ " + .commit.sha[0:9]'
   ```
2. If the push was a no-op because `<branch-name>` already existed on
   origin (e.g. pre-created by an earlier `/roadmap-next` run), confirm
   the remote tip matches the local tip — if it doesn't, STOP and file
   a `kind: "blocker"` DQ entry. Do NOT force-push.

Why push immediately (changed 2026-05-25): every recent bm-cut brief
overrode the previous "deferred push" instruction. The advisor's
polling loop reads `origin/<phase-branch>` to detect impl-task
worker pushes (per `advisor-orchestrator.md` §1). A worker can only
fork from a branch that exists on origin (per
`feedback_daemon_local_trunk_stale_multi_lane.md`). Pushing in
Phase 4 makes the branch reachable to both readers; the empty-ref
concern from the old "deferred" rationale is moot because phase
branches have a runlog commit (Phase 3 of bm-cut + Phase 4 of this
script + the runlog write below all happen on the new branch before
the push, so the branch is one-commit-ahead, not empty, at push time).

---

## Phase 5 — Append to runlog

The runlog write commit is the first commit on the new branch. Create
or append to `.claude/runlog/<phase>-runlog.md`:

```markdown
## bm: cut <branch-name> off governance-v0 @ {short-sha} — {ISO timestamp}
- **branch:** {branch-name}
- **off:** governance-v0 @ {short-sha}
- **plan:** .claude/PRPs/plans/{plan-file} (or "n/a — chore branch")
- **pushed:** yes — origin/{branch-name} (upstream tracking set via -u)
- **verified:** gh api branch endpoint confirmed
- **next:** advisor authors impl-task briefs; worker forks from {branch-name}
```

Commit + push the runlog entry:

```bash
git add .claude/runlog/<phase>-runlog.md
git commit -m "chore(bm-task): log branch cut <branch-name> post-planning approval"
git push origin <branch-name>
```

The advisor reads this entry on its next polling tick to confirm the
cut landed cleanly.

---

## Phase 6 — Output

```markdown
## /bm-cut complete

**Branch cut:** {branch-name}
**Off:** governance-v0 @ {short-sha}
**Plan file on trunk:** {path} (or "n/a")
**Pushed?:** Yes — origin/{branch-name} (upstream tracking via -u)
**Runlog:** .claude/runlog/<phase>-runlog.md updated + pushed
**Daemon worktree** (Junior runtime only): now on {branch-name} (was governance-v0). See §7.

### Hand-off to advisor

Advisor can now author impl-task briefs. Per
`advisor-orchestrator.md` §2.1 + `.claude/rules/multi-lane-worktree.md`
"Brief location and trunk→phase sync", impl-task briefs MUST be
visible on {branch-name} before the corresponding Junior task is
queued. See those docs for the sync procedure (advisor-side, not a
bm-task verb).
```

---

## Phase 7 — Daemon worktree state post-bm-cut (load-bearing side effect)

The bm-cut Junior task runs in the daemon's main worktree at
`/srv/<repo>` (its own worktree per `feedback_parallel_agents_one_worktree_per_agent`
is created by Junior for the task, but the `git checkout -b` in Phase 3
mutates the **daemon's** branch checkout because Junior dispatches the
bm-task into the main worktree's branch ref space, not into a separate
sub-worktree). After bm-cut completes, the daemon worktree is on the
new phase branch (`phase-<X>`) regardless of what branch it was on
before.

This is intentional and load-bearing for two downstream advisor
patterns:

1. **Advisor-side trunk→phase merge** (`multi-lane-worktree.md`
   §"Brief location and trunk→phase sync"). When the advisor authors
   an impl-task brief on trunk + needs to make it visible on the phase
   branch, the canonical path is to SSH into the daemon and run
   `git merge origin/governance-v0` from the daemon worktree —
   which works because the daemon is conveniently on `phase-<X>`
   after bm-cut. If the daemon worktree were on `governance-v0`,
   the advisor would need a temporary worktree or a checkout-switch
   to do the merge.

2. **Next-cohort impl-task fork base.** Junior creates per-task
   worktrees off the daemon-local branch ref (per
   `feedback_daemon_local_trunk_stale_multi_lane.md`). After bm-cut
   the daemon-local `phase-<X>` ref points at the runlog commit,
   matching origin — workers fork from the right base.

**Failure mode if daemon worktree is NOT on `phase-<X>` after bm-cut:**
a concurrent task switched it. Detection:
```bash
ssh homeserver 'cd /srv/<repo> && git symbolic-ref HEAD'
# EXPECT: refs/heads/phase-<X>
```
Recovery: from the daemon worktree, `git checkout phase-<X>` (safe;
the bm-task push already landed the branch on origin). DO NOT delete
or re-cut.

---

## Phase 8 — Finalize hazard (READ — daemon finalize agent is NOT bm-cut-aware)

bm-cut **creates divergence from trunk**. The phase branch is the
deliverable; it must **NEVER** be merged back into `governance-v0`.

The Junior daemon's generic post-job finalize agent is
feature-branch-shaped ("commit worktree changes, merge the task's
branch into the base branch"). On a bm-cut task it will **wrongly run
`git merge --no-ff phase-<X>` INTO daemon-local `governance-v0`**,
producing a spurious content-empty merge commit (e.g. `Merge
phase-v1-AD-e into governance-v0 (bm-cut task)`). Confirmed 2× —
v1-AD-e #282 + v1-ship-1, 2026-05-16. Full detail + recovery:
`.claude/lessons/feedback_junior_finalize_merges_bm_cut_branch.md`.

**Until the daemon finalize agent reliably skips bm-cut** (structural
fix = extend the `[role:bm-task]` finalize-skip, or a
`FINALIZE: do-not-merge` worktree sentinel — same class as the
planned `[role:impl-task]` skip in
`feedback_junior_finalize_skips_when_worker_pre_pushes.md`):

- **The bm-cut brief MUST carry a §8 "KNOWN harness limitation" block**
  stating bm-cut creates divergence (NOT a feature branch to merge
  back), the CC v2.1.119 runlog gate-block, and the advisor-relocate
  recovery (see `feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md`).
- **The advisor MUST verify daemon-local trunk post-bm-cut as a
  ROUTINE step** (not an exception path):
  `ssh homeserver 'cd /srv/<repo> && git log governance-v0 --oneline -1'`
  — if it shows a `Merge phase-<X> into governance-v0 (bm-cut task)`
  commit while `origin/governance-v0` is unchanged, recover with
  `git update-ref refs/heads/governance-v0 origin/governance-v0`
  (working-tree-safe — NOT `git reset --hard`, which switches the
  checkout and can race concurrent lane tasks), then
  `git push origin phase-<X>:phase-<X>`.

---

## Refusal cases

- Trunk dirty → STOP.
- `governance-v0` ahead of remote with unpushed commits → STOP, ask.
- No plan file on trunk for a phase branch → STOP, file DQ.
- Branch name pattern mismatch → STOP, ask.
- **Finalize agent merged the phase branch into trunk** (daemon-local
  `governance-v0` shows `Merge phase-<X> into governance-v0 (bm-cut
  task)`; origin unchanged) → NOT a bm-cut failure (branch was created
  correctly); recover per Phase 8 via `git update-ref` + push the
  phase branch; relocate the gate-blocked runlog advisor-side.

---

## See also

- `.claude/rules/branch-manager.md` — file ownership + autonomy bounds
- `.claude/rules/phase-branch.md` — phase-branch flow
- `.claude/commands/bm/bm-status.md` — see current state
- `.claude/commands/bm/bm-push.md` — push when impl has committed
- `.claude/lessons/feedback_junior_finalize_merges_bm_cut_branch.md` — Phase 8 hazard detail + recovery
- `.claude/lessons/feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` — runlog gate-block + advisor-relocate (§6)
