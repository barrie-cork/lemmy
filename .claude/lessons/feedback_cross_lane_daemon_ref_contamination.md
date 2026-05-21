---
name: Junior daemon's single-checkout model contaminates phase branch refs across lanes
description: The Junior daemon at /srv/brehon-fork uses one shared checkout. When concurrent Junior workers from different lanes (phase-v1-X and phase-v1-Y) dispatch in close succession, the daemon's local phase-v1-X branch ref can be reset to phase-v1-Y commits between dispatches. Workers forking from the contaminated daemon-local ref pick up the wrong base.
type: feedback
---

# Cross-lane daemon ref contamination

## The mechanism

The Junior daemon at `homeserver:/srv/brehon-fork` is a single-checkout repository (no `git worktree` per dispatch on the daemon side itself; only the impl/planning/etc worker tasks create per-job worktrees under `/srv/brehon-fork/.junior/worktrees/job-N/`). The shared checkout's `HEAD` plus the local `refs/heads/phase-v1-*` refs are mutated by `git fetch + checkout` cycles when dispatching workers across multiple lanes.

The contamination instance observed in v1-rls-r1 (3× during one ~9-hour wall-clock window):
1. v1-rls-r1 lane dispatches Junior #380, daemon does `git fetch + git checkout phase-v1-rls-r1`. OK.
2. Concurrent brehon-conformance-audit lane dispatches Junior #382, daemon does `git fetch + git checkout phase-brehon-conformance-audit`. The conformance-audit worker finalize-merges into `phase-brehon-conformance-audit`.
3. v1-rls-r1 lane dispatches Junior #383. Daemon's `git checkout phase-v1-rls-r1` finds the local ref now pointing at a conformance-audit commit (`f9eb94969`) — the daemon's last `git fetch` brought down conformance-audit work, the local ref reset got applied to the wrong branch.
4. Junior #383's worktree forks from `f9eb94969` — base is wrong; the worker would commit on top of conformance-audit code, then daemon finalize-merge would push the resulting tree to `origin/phase-v1-rls-r1`, blowing away the real v1-rls-r1 chain.

## Detection

**Pre-dispatch (mandatory under concurrent lanes):**
```bash
ssh homeserver "cd /srv/brehon-fork && git log phase-v1-<phase> --oneline -3"
```
- Compare top commit subjects against `origin/phase-v1-<phase>`. If subjects mention a different lane (e.g. `feat(brehon-conformance-audit):`), the daemon ref is contaminated.
- If subjects match the expected lane: OK to dispatch.

**Post-dispatch (additional check):**
```bash
ssh homeserver "cd /srv/brehon-fork/.junior/worktrees/job-<id> && git log --oneline -3"
```
- The worker's worktree was forked from the daemon-local ref. If contaminated, the worker's base is wrong; cancel immediately before commit.

## Recovery (lane-safe FF)

The naive `git branch -f phase-v1-<phase> origin/phase-v1-<phase>` fails when the branch is checked out (`fatal: cannot force update the branch ... used by worktree`). Sequence:

```bash
ssh homeserver "cd /srv/brehon-fork && \
  git checkout --detach HEAD && \
  git fetch origin phase-v1-<phase> && \
  git branch -f phase-v1-<phase> origin/phase-v1-<phase> && \
  git checkout phase-v1-<phase>"
```

The detach lets `git branch -f` proceed; the checkout restores the daemon to the correct branch with origin-aligned content. Per `feedback_junior_292_stale_base_recover_recipe.md` (related family).

## How to apply

- **Pre-dispatch in concurrent-lane mode:** run the detection probe on every Junior dispatch when `git worktree list` on the daemon shows another active lane's worker.
- **Tactical:** mid-phase, advisor accepts the latency cost of detach-and-FF before each dispatch.
- **Strategic (v1-rls-r2+ candidate):** investigate per-lane ref namespacing on the daemon (e.g. `refs/heads/lanes/<lane>/phase-v1-...`) so the daemon's `checkout` doesn't traverse the same ref across lanes.

## See also

- `feedback_junior_292_stale_base_recover_recipe.md` — adjacent stale-base recipe (workers fork from daemon-LOCAL stale ref)
- `feedback_daemon_local_trunk_stale_multi_lane.md` — the original observation
- `.claude/rules/multi-lane-worktree.md` — the human-side lane-isolation discipline (works); the daemon-side equivalent is what's missing
- `feedback_advisor_authoring_under_daemon_stress.md` — the fallback when this fires repeatedly
- `.claude/PRPs/reports/v1-rls-r1-retro.md` — first formal observation
