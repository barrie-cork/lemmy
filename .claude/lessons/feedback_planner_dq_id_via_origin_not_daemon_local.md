# feedback: planner DQ id collisions via daemon-local-trunk staleness

## TL;DR

When a planning Junior task computes `next_id` for a DQ entry, it walks **daemon-local** `.claude/decision-queue.json` (whatever the daemon-local trunk last fetched). If advisor commits have advanced origin between the daemon's last fetch and the task spawn, ids will collide with the advisor's recent writes.

The planner's discipline must extend the `max(all_ids) + 1` computation across **origin** refs too, not just the worktree-local view. Equivalently: the advisor MUST `git fetch origin` + ff daemon-local trunk before each Junior task dispatch when trunk has been advanced in this session.

## Concrete failure (brehon-conformance-audit, 2026-05-20)

- 2026-05-20 ~13:30 UTC: advisor pushed `bbe4def06` (clarify-pass DQ #291-#294) to `origin/governance-v0`.
- 2026-05-20 ~13:45 UTC: advisor queued planning Junior #352 with `base_branch=governance-v0`.
- Daemon-local trunk was BEHIND `bbe4def06` — last sync had been at `a16a4e2e1` or earlier.
- Planner read its worktree's `.claude/decision-queue.json`, computed `max(all_ids) = 290`, picked `next_id = 291`.
- Planner committed DQ #291 (`from: "planner", kind: "blocker"`) as `5d1d38bbf` on its worker branch.
- Daemon finalize-merge brought that into daemon-local `governance-v0` as merge commit `69f7e210f` — but never pushed.
- Advisor caught the collision because origin's #291 was the advisor's clarify entry; advisor's `git fetch origin` revealed the divergence.

Cost: ~45 min wallclock recovery (SSH reset + cherry-pick plan-only commit; fresh advisor commit with the planner's split-or-proceed question re-numbered as DQ #295 with `answered_by: "user"`).

## Root cause

Two compounding factors:

1. **Daemon's local `governance-v0` was stale** (not yet fetched the advisor's clarify-pass commit). The lesson `feedback_daemon_local_trunk_stale_multi_lane.md` documents the broader pattern.
2. **Planner discipline does not include `git fetch + git log origin/<base>..HEAD` before computing next_id.** Planner reads only its worktree view.

## When to apply

ALWAYS, when dispatching a planning task whose base branch has been advanced by the advisor in this session — even by a single commit.

## How to apply

**Advisor-side (immediate fix, no shim patch needed):**

Before EVERY `mcp__junior-brehon__create_task` call for ANY role that may write DQ:

```bash
# 1. Ensure local advisor session pushed everything
git push origin governance-v0  # if there are any unpushed advisor commits

# 2. Force daemon-local refs to match origin
ssh homeserver "cd /srv/brehon-fork && git fetch origin governance-v0 && git update-ref refs/heads/governance-v0 origin/governance-v0"

# 3. NOW queue the Junior task
```

The `git update-ref` is lane-safe (no checkout switch, no working-tree disturbance). Skipping this step risks the stale-base defect class.

**Planner-side (longer-term fix, requires planning.md agent definition update):**

Extend the planner's task-0 pre-flight to include:

```bash
# Compute next_id across origin too, not just worktree-local
python3 -c "
import json, subprocess
local = json.load(open('.claude/decision-queue.json'))
origin = subprocess.check_output(['git', 'show', 'origin/governance-v0:.claude/decision-queue.json'], text=True)
origin_d = json.loads(origin)
all_ids = set([e['id'] for e in local.get('pending',[]) + local.get('resolved',[])])
all_ids |= set([e['id'] for e in origin_d.get('pending',[]) + origin_d.get('resolved',[])])
print('next_id:', max(all_ids) + 1)
"
```

Useful pattern but not load-bearing if the advisor-side ff is done routinely.

## Cross-references

- `.claude/rules/decision-queue.md` Hard refusal #2 ("NEVER reuse an existing id").
- `.claude/rules/multi-lane-worktree.md` §"Worktree-aware DQ id discipline".
- `feedback_check_git_before_junior_queue.md` — pre-dispatch git state check.
- `feedback_daemon_local_trunk_stale_multi_lane.md` — daemon-local refspec drift.
- Session retro: `.claude/PRPs/reports/session-retro-2026-05-20-brehon-conformance-audit-bootstrap.md` §2.2.
- The recovery: DQ #295 commit `c9bd60598`; plan-only cherry-pick `a16a4e2e1`.
