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
or plan. Verifies trunk is clean and up-to-date first. The branch is
local-only at this stage (Phase 5 below pushes with `-u` to set
upstream tracking). Auto, no prompt (local-only action per
`.claude/rules/branch-manager.md`).
<!-- cr-1 (closes #88): wording aligned with the actual command flow —
     `git checkout -b` does not set upstream; `git push -u` in Phase 5 does. -->


**Reads:** `.claude/rules/branch-manager.md`, `.claude/rules/phase-branch.md`.

---

## Phase 0 — Parse arguments

| Input | Branch name | Type |
|---|---|---|
| `v1-AD-e` | `phase-v1-AD-e` | phase |
| `v1-AD-e --plan` | `plan/v1-AD-e` | plan |
| `v1-AD-e --chore lint-cleanup` | `chore/lint-cleanup` | chore |

If the branch name doesn't match `^phase-v\d+-[A-Z]+-[a-z]$` (phase),
`^plan/v\d+-[A-Z]+-[a-z]$` (plan), or `^chore/[a-z0-9-]+$` (chore),
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

## Phase 3 — Cut the branch

```bash
git checkout -b <branch-name> governance-v0
```

Do NOT push yet. The branch sits local until impl makes its first
commit. (Pushing an empty branch creates a useless remote ref with no
PR target.)

---

## Phase 4 — Append to runlog

Create or append to `.claude/runlog/<phase>-runlog.md`:

```markdown
## bm: branch cut — {ISO timestamp}
- **branch:** {branch-name}
- **off:** governance-v0 @ {short-sha}
- **plan:** .claude/PRPs/plans/{plan-file} (or "n/a — chore branch")
- **next:** impl session takes over for task 1
```

---

## Phase 5 — Output

```markdown
## /bm-cut complete

**Branch cut:** {branch-name}
**Off:** governance-v0 @ {short-sha}
**Plan file on trunk:** {path} (or "n/a")
**Pushed?:** No (deferred to first commit + /bm-push)
**Runlog:** .claude/runlog/<phase>-runlog.md updated

### Hand-off to impl session

Impl can now run `/prp-implement` against the plan above. BM will pick
up the branch on `/bm-status` once commits land.
```

---

## Phase 6 — Finalize hazard (READ — daemon finalize agent is NOT bm-cut-aware)

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

- **The bm-cut brief MUST carry a §6 "KNOWN harness limitation" block**
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
  correctly); recover per Phase 6 via `git update-ref` + push the
  phase branch; relocate the gate-blocked runlog advisor-side.

---

## See also

- `.claude/rules/branch-manager.md` — file ownership + autonomy bounds
- `.claude/rules/phase-branch.md` — phase-branch flow
- `.claude/commands/bm/bm-status.md` — see current state
- `.claude/commands/bm/bm-push.md` — push when impl has committed
- `.claude/lessons/feedback_junior_finalize_merges_bm_cut_branch.md` — Phase 6 hazard detail + recovery
- `.claude/lessons/feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` — runlog gate-block + advisor-relocate (§6)
