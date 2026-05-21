# feedback: lane agent `git reset --hard` against shared daemon checkout orphans whatever HEAD is currently on

> **Note on title:** this lesson was originally titled *"daemon finalize step hard-resets governance-v0 to a non-trunk phase branch's tip"* — a hypothesis that was falsified on 2026-05-22 by sub-agent re-investigation (sub-agent ts 2026-05-22, transcript citation below). The daemon's finalize step is NOT the vector. The file slug is preserved for cross-reference stability; the title and body are corrected. See §"Falsified hypothesis" appendix.

## TL;DR

When a lane-dedicated advisor session ssh's into the shared Junior daemon checkout (`/srv/brehon-fork`) and runs `git reset --hard origin/<phase-branch>` to "fast-forward" the daemon's phase ref, the reset operates on **whatever branch HEAD is currently checked out**, not on the target branch named on the right-hand side. If the daemon's main checkout has been transiently switched to `governance-v0` by a Junior finalize-merge subagent (STEP 3 of `buildFinalizePrompt`: `git checkout governance-v0 → git merge --ff-only origin/governance-v0 → git merge --no-ff <worker>`), the lane agent's `reset --hard origin/<phase-branch>` will move `governance-v0` (not the phase branch) to the phase branch's tip — orphaning any plan-merge commit that was just produced.

**This is a coordination defect between two laptop sessions** (the canonical session running Junior orchestration + a lane-dedicated session FF-ing its phase ref on the daemon) that share write access to a single daemon-side `/srv/brehon-fork/.git/`. The Junior daemon is the **innocent bystander** whose finalize-merge subagent transiently switches HEAD as part of its prompt; the destructive write comes from the other laptop session.

## Why this mattered (incident 2026-05-21)

Daemon reflog sequence on `governance-v0`:

```
@{5}  2ad835aa4  merge junior/role-planning-...-399: Merge made by the 'ort' strategy   ← Junior finalize, CORRECT
@{4}  7bc103421  reset: moving to origin/phase-v1-federation-inbound-c                  ← DESTRUCTIVE; orphans @{5}
@{3}  c02dc8617  cherry-pick: docs(plan): v1-dq-schema-r1 plan written                  ← recovery
@{2}  c3d6bbd49  reset: moving to origin/governance-v0                                  ← re-align
@{1}  74dc2d5e7  pull --ff-only origin governance-v0                                    ← back to clean
```

Timestamp gap between @{5} (Junior finalize at `19:55:59Z`) and @{4} (destructive reset at `19:58:26Z`): **~2.5 minutes**. Long enough to rule out the Junior daemon itself (its finalize subagent had already exited successfully); short enough that a different actor — the fed-in-c lane laptop session — was issuing the reset as part of its own "FF daemon-local phase ref" routine.

Origin/governance-v0 was protected only because the reset was followed by recovery before any laptop push.

## Vector (confirmed 2026-05-22 by sub-agent forensics)

A lane-dedicated laptop Claude Code session (CWD: `brehon-fork-fed-in-c`) had been running a "lossless FF" pattern 8 times that day, each time invoking:

```bash
ssh homeserver "cd /srv/brehon-fork && git fetch origin <phase-branch> && git reset --hard origin/<phase-branch> && git rev-parse HEAD"
```

**Smoking-gun transcript:** `C:\Users\barri\.claude\projects\C--Users-barri-Developer-brehon-fork-fed-in-c\a6097edd-663b-4770-b73a-816a012b6d67.jsonl` event uuid `cf9c79f1-a709-41ea-a920-dc643a2dd0a5` ts `2026-05-21T19:58:17.028Z` (9 seconds before the daemon reflog records the destructive reset). Tool-call description: `"FF daemon-local phase ref to new merged tip 7bc103421 (lossless reset since lane is the sole writer)"`.

Of the 8 invocations:
- **7 were lossless** because daemon HEAD was already on the phase branch (or idle on it between finalizes). `reset --hard` on the phase branch with phase-branch target = no-op or true fast-forward.
- **1 (this incident)** landed during the 2.5-minute window after Junior finalize had switched daemon HEAD to `governance-v0` to merge the planning task into trunk. The reset moved `governance-v0` (current branch) to `origin/phase-v1-federation-inbound-c` (target ref). Plan commit orphaned.

The flaw is **not** the use of ssh, nor the use of `--hard`, nor the "lossless when lane is sole writer" reasoning *in the steady state*. The flaw is the **absence of a HEAD precondition check** before the reset. Daemon HEAD is shared state, mutated by every Junior finalize (transiently) and by every prior advisor SSH command (persistently). A correct routine must verify HEAD matches the named target — OR avoid touching HEAD-tracked branches at all.

## How to apply

### Correct routine: `update-ref`, not `reset --hard`

```bash
ssh homeserver "cd /srv/brehon-fork && git fetch origin <phase-branch> && git update-ref refs/heads/<phase-branch> origin/<phase-branch>"
```

`git update-ref` operates on the **named** ref, not on HEAD. It is lane-safe and worktree-safe regardless of which branch daemon's main checkout is currently on, regardless of which Junior worker is mid-finalize. This is the recipe `feedback_planner_dq_id_via_origin_not_daemon_local.md` already recommends for the analogous "ff daemon-local trunk" case; the same recipe applies for ff-ing any daemon-local phase branch.

### If you must use `git reset --hard` (don't, but if you must)

Guard with an explicit HEAD-precondition check:

```bash
ssh homeserver "cd /srv/brehon-fork && \
  CURRENT=\$(git branch --show-current) && \
  if [ \"\$CURRENT\" != \"<phase-branch>\" ]; then \
    echo \"REFUSE: HEAD on \$CURRENT, expected <phase-branch>\" >&2; exit 1; \
  fi && \
  git fetch origin <phase-branch> && \
  git reset --hard origin/<phase-branch>"
```

The guard converts the silent destructive-reset bug into a loud refusal. But `update-ref` is structurally better and should be the default.

### Hard refusals (any lane laptop session)

1. **Never `ssh homeserver "...git reset --hard..."` against `/srv/brehon-fork` without a HEAD precondition check.** The daemon's main checkout HEAD is shared mutable state across all laptop sessions + the Junior finalize subagents; you cannot assume it's where you left it 30 seconds ago.
2. **Never use `git reset --hard <ref>` when you mean "advance refs/heads/<branch> to <ref>".** `reset` moves HEAD's branch; `update-ref` moves the named branch. Two different operations.
3. **Never "fast-forward" a daemon-local ref while a Junior task with that base_branch (or governance-v0) is mid-finalize.** The Junior daemon publishes task status via `mcp__junior-brehon__list_tasks`; check that no task with `status: running` and a matching base_branch is in flight.

## Recovery recipe (still valid — verified 2026-05-21)

If the destructive reset has already happened and a commit is orphaned on the daemon's reflog:

1. **SSH to daemon, cherry-pick from reflog:**
   ```bash
   ssh homeserver "cd /srv/brehon-fork && \
     git reflog governance-v0 | head -5 && \
     git cherry-pick <orphaned-merge-sha>~1..<orphaned-merge-sha>"
   ```
2. **Push to a recovery branch (NOT trunk):**
   ```bash
   ssh homeserver "cd /srv/brehon-fork && \
     git push origin HEAD:refs/heads/recovery/<phase>-plan"
   ```
3. **Laptop side pull-cherry-pick:**
   ```bash
   git fetch origin recovery/<phase>-plan
   git cherry-pick origin/recovery/<phase>-plan
   git push origin governance-v0
   git push origin :recovery/<phase>-plan
   ```
4. **Daemon-side trunk re-alignment via `update-ref` (NOT `reset --hard`, per this lesson):**
   ```bash
   ssh homeserver "cd /srv/brehon-fork && \
     git fetch origin governance-v0 && \
     git update-ref refs/heads/governance-v0 origin/governance-v0"
   ```

## Falsified hypothesis (preserved for cross-reference; see also `feedback_falsifiable_hypothesis_before_structural_fix.md`)

The original DQ #338 (2026-05-21) attributed the destructive reset to "the daemon's finalize step issued an unintended `git reset --hard origin/phase-v1-federation-inbound-c` or equivalent on governance-v0." Sub-agent re-investigation 2026-05-22 confirmed:

- `buildFinalizePrompt` (`/opt/junior-src/src/core/claude.ts:239`) has NO `reset --hard` instruction.
- `/opt/junior-src/src/daemon/executor.ts` + `git.ts` have ZERO `git reset` calls (grep `/opt/junior-src/src/ /opt/junior-src/dist/` clean).
- Task #399's bash log shows clean STEP 3 finalize: `checkout → merge --ff-only → merge --no-ff`. No reset.
- All recent Junior task logs (jobs 395-402) contain zero `reset --hard` calls.

The daemon-finalize hypothesis is therefore false. The investigation pointer in the prior version of this lesson (`/opt/junior-src/src/daemon/executor.ts finalize-merge code path`) is invalid as an option-a target. The structural-fix recommendation in DQ #338 v1 was scoped to the wrong code surface.

## Companion DQ + lessons

- **DQ #338** — original recommendation falsified; mutate the `answer` field to record correct RCA + the `update-ref`-not-`reset --hard` structural fix.
- `feedback_falsifiable_hypothesis_before_structural_fix.md` — the meta-lesson from this investigation: verify the hypothesis behind a structural-fix DQ in ≤30 min before committing to the fix path.
- `feedback_planner_dq_id_via_origin_not_daemon_local.md` — already recommends `git update-ref refs/heads/governance-v0 origin/governance-v0` for the trunk case; this lesson generalizes the same recipe to phase branches.
- `feedback_daemon_local_trunk_stale_multi_lane.md` — broader pattern of daemon-local refs drifting from origin under multi-lane operation.
- `feedback_junior_finalize_merge_race_lossless_reconcile.md` — BOTH-RAN reconcile pattern; complementary defense for the finalize-race case.
- `project_concurrent_advisor_sessions_2026_05_21.md` — the incident-window project memory.
