Roadmap-driven sub-phase cutter. Reads `.claude/PRPs/v1-roadmap.json`, recommends the next unstarted sub-phase, gets user confirmation, dispatches `bm-cut` to create the phase branch, opens a lane-dedicated git worktree, walks the bootstrap checklist, flips the roadmap entry from `unstarted` → `in_flight`, and prints a hand-off to the user for opening Claude Code in the new worktree and running `/auto-roadmap`.

Argument: none. The skill reads its target sub-phase from the roadmap; the user confirms via `AskUserQuestion`.

**This is skill 1 of a two-skill chain.** Skill 2 is `/auto-roadmap` (runs in the cut lane worktree). The hand-off between skills is **manual**: skill 1 ends by telling the user which directory to open Claude Code in; the user opens it and runs `/auto-roadmap`. Per `.claude/rules/auto-roadmap.md` state-routing invariant #6.

## Why this shape

`/auto-phase` is a single-sub-phase orchestrator. Driving multiple sub-phases of a v1 lane (e.g. RT-r2 → r3 → r4 → r5) today requires the advisor to re-derive "which sub-phase next?" at every boundary, re-author bm-cut briefs, re-walk the bootstrap checklist, and remember which sub-phases are in-flight in which lanes. The recurring footgun: the advisor manages this state in conversation context, which compacts away across session boundaries.

`/roadmap-next` makes the sub-phase choice **roadmap-driven** instead of conversation-driven. The roadmap file is the source of truth for what's done, what's in flight, and what's next. The skill reads it, presents the recommendation, and acts on user confirmation. The roadmap file survives session boundaries; the next session resumes by reading the same file.

The 70% reuse of `bm-cut` is deliberate — skill 1 borrows the BM pattern verbatim (per user steer 2026-05-22) rather than re-implementing branch-cut logic.

## What `/roadmap-next` automates

| Step | Without skill (manual) | With skill |
|---|---|---|
| Identify next unstarted sub-phase | Advisor reads roadmap.json + reasons about ordering | Auto (skill reads `what_remains` + `implementation_steering`) |
| Surface recommendation + rationale | Advisor types it in conversation | AskUserQuestion with three options (confirm / pick different / defer) |
| Cut `phase-<sub-phase>` branch off `governance-v0` | Advisor types `git checkout -b ...` or invokes `/bm-cut` | `branch-manager` subagent dispatched verbatim |
| Open lane worktree at `../brehon-fork-<lane-suffix>` | Advisor types `git worktree add ...` | Auto |
| Walk 11-step bootstrap checklist | Advisor copies `.mcp.json`, edits `settings.local.json`, runs verification python | Auto (each step + each verification probe) |
| Flip roadmap `unstarted` → `in_flight` + add worktree path | Advisor edits JSON, commits, pushes | Auto (atomic protocol per multi-lane-worktree.md #6) |
| Print hand-off instructions to user | Advisor types it in conversation | Auto |

## What `/roadmap-next` does NOT automate

- **Open Claude Code in the new worktree.** That's the user's hand. Manual hand-off is the natural boundary where the user verifies the new worktree opened cleanly (SessionStart hooks fired without WARN) before any state-changing action begins.
- **Run `/auto-roadmap` automatically.** Same reason — manual hand-off.
- **Plan authoring for the cut sub-phase.** That's skill 2's plan-gap handler.
- **Author the bm-cut brief.** `bm-cut` is dispatched with `argument-hint: <branch-suffix>` per `.claude/commands/bm/bm-cut.md`; no brief file required.
- **Lane-worktree pruning at sub-phase end.** Per `multi-lane-worktree.md` §"Lifecycle" step 3, the user runs `git worktree remove` manually after merge. Skill 1 won't auto-prune.
- **Roadmap schema migration.** If the roadmap is at an unknown `$schema_version`, skill 1 refuses (hard refusal #N below — schema-version compatibility check). Schema bumps happen out-of-band.

## Procedure

### Phase 0 — Prerequisites

Verify before any state-changing call:

1. **CWD is canonical** `C:/Users/barri/Developer/brehon-fork`:
   ```bash
   pwd
   ```
   If not → refuse: "skill 1 must run in canonical CWD; current CWD is X" per `.claude/rules/auto-roadmap.md` hard refusal #1.

2. **Branch is `governance-v0`** with clean working tree:
   ```bash
   git -C C:/Users/barri/Developer/brehon-fork rev-parse --abbrev-ref HEAD
   git -C C:/Users/barri/Developer/brehon-fork status --short
   ```
   If not on `governance-v0` or working tree dirty → refuse per hard refusal #2 / #3.

3. **Trunk up to date with origin:**
   ```bash
   git -C C:/Users/barri/Developer/brehon-fork fetch origin
   git -C C:/Users/barri/Developer/brehon-fork log governance-v0..origin/governance-v0 --oneline | head -5
   ```
   If trunk behind → `git pull --ff-only origin governance-v0`. If trunk ahead of origin → STOP (unpushed commits indicate a different session is mid-work).

4. **Roadmap file exists + is valid JSON:**
   ```bash
   python -c "
   import io, json
   r = json.load(io.open('.claude/PRPs/v1-roadmap.json', encoding='utf-8'))
   assert 'lanes' in r, 'roadmap missing lanes key'
   assert r.get('\$schema_version') in {1, 2}, f'unknown schema_version: {r.get(\"\$schema_version\")}'
   print('OK: roadmap loaded; schema_version=', r['\$schema_version'])
   "
   ```
   If parse error or unknown schema → refuse per hard refusal #4. Per `feedback_python_utf8_encoding_windows.md` the `encoding='utf-8'` is mandatory on Windows.

5. **Multi-lane CWD check** per `.claude/rules/multi-lane-worktree.md` "Session-start ritual":
   ```bash
   git -C C:/Users/barri/Developer/brehon-fork worktree list
   ```
   Note other active worktrees. If any lane-dedicated worktree shows a recent commit (per `.claude/hooks/session-start-multi-lane-check.sh` WARN), surface to the user before proceeding — another session may be driving a sub-phase concurrently.

### Phase 1 — Read roadmap, compute recommendation

Read the roadmap, parse `what_remains.high_priority_unstarted` + `implementation_steering`:

```bash
python -c "
import io, json
r = json.load(io.open('.claude/PRPs/v1-roadmap.json', encoding='utf-8'))

# Build candidate list from high_priority_unstarted
candidates = []
for lane_summary in r.get('what_remains', {}).get('high_priority_unstarted', []):
    lane_id = lane_summary['lane'].split('(')[0].strip().split()[0]  # 'SR' from 'SR (Ship Readiness)'
    for item in lane_summary['items']:
        # item is like 'v1-ship-2' or 'v1-RT-r2'
        candidates.append({'sub_phase': item, 'lane': lane_id, 'rationale': lane_summary['rationale']})

# Annotate with status check (should all be unstarted — sanity check)
for c in candidates:
    lane_key = {'SR (Ship Readiness)': 'SR', 'RT (Reputation Tuning)': 'RT'}.get(c['lane'], c['lane'])
    lane = r['lanes'].get(lane_key, {})
    sub_phases = lane.get('sub_phases', {})
    entry = sub_phases.get(c['sub_phase'], {})
    c['status'] = entry.get('status', 'MISSING')
    c['has_plan'] = entry.get('plan') is not None

# implementation_steering.next_logical_sub_phase is the advisor's stated preference
steering = r.get('implementation_steering', {})
print('STEERING:', steering.get('next_logical_sub_phase', 'none'))
print()
for c in candidates:
    print(f\"  {c['sub_phase']} (lane={c['lane']}, status={c['status']}, has_plan={c['has_plan']})\")
    print(f\"    -> {c['rationale'][:120]}\")
"
```

The recommended next sub-phase is **the first candidate in `what_remains.high_priority_unstarted` whose status is `unstarted`**, modulo any override from `implementation_steering.next_logical_sub_phase` (which may name in-flight lanes that the user wanted to close first).

If `implementation_steering.next_logical_sub_phase` names an in-flight or skipped lane (e.g. "Resolve federation-inbound-d first"), surface that as the **primary recommendation** even though it's not in `high_priority_unstarted` — the steering field is the human-authored override.

### Phase 2 — User gate: confirm recommendation

Fire `AskUserQuestion` with three options:

```
question: "Roadmap recommends cutting <sub-phase> next (lane: <lane>, rationale: <one-line>). Confirm?"
header: "Next sub-phase"
options:
  1. "Cut phase-<sub-phase> (Recommended)" — proceed
  2. "Pick a different unstarted sub-phase" — surface dropdown with all candidates from Phase 1
  3. "Defer (exit without cutting)" — clean exit
```

If user picks option 2, fire a second AskUserQuestion with the full candidate list. If user types `Other` and names a sub-phase NOT in any lane's `sub_phases`, refuse with "sub-phase not in roadmap; add it to roadmap first".

Record the chosen sub-phase as `<target>` for the remaining phases. Compute `<lane-suffix>` from `<target>` per the deterministic rule:

| `<target>` | `<lane-suffix>` (lowercase, hyphenated) |
|---|---|
| `v1-RT-r2` | `rt-r2` |
| `v1-RT-r3` | `rt-r3` |
| `v1-ship-2` | `ship-2` |
| `v1-ship-3` | `ship-3` |
| `v1-AD-f` | `ad-f` (hypothetical future) |
| `v1-JM-f` | `jm-f` (hypothetical future) |

General rule: strip the `v1-` prefix, lowercase. The result is the suffix for `phase-v1-<TARGET>` → `brehon-fork-<lane-suffix>`. Codified here because the spec §8 open question 4 flagged ambiguity.

If user picks option 3 (defer), print "no changes made; re-run `/roadmap-next` later" and exit.

### Phase 3 — Dispatch `bm-cut` to cut the phase branch

Invoke the `branch-manager` subagent (haiku, low-effort) via `Agent` tool. The subagent reads `.claude/commands/bm/bm-cut.md` and follows the operational script. Arguments: `<target>` verbatim.

```
Use the `branch-manager` subagent to run `bm-cut`. Arguments: <target>.
Follow the phases in .claude/commands/bm/bm-cut.md — parse arg into
branch name (expect phase-<target>), verify trunk cleanliness, verify
the corresponding plan file exists on trunk (NOTE: for /roadmap-next
the plan may be MISSING — that's OK; skill 2 will handle plan-gap.
If bm-cut refuses because plan is missing, override per the DQ-option
"cut anyway (override)" — this is the documented exception path for
roadmap-driven cuts of plan-less sub-phases).
Cut the branch locally without pushing. Return a "Branch cut" summary.
```

**Plan-file caveat:** `bm-cut.md` Phase 2 STOPS if no plan file exists for a phase branch. Skill 1's expected case for RT-r2..r5 is plan-missing (those plans haven't been authored yet — that's skill 2's plan-gap handler's job). The dispatch passes an explicit override flag to the BM subagent ("plan-gap is expected; cut anyway"). The BM file DQ entry per its Phase-2 STOP gets self-resolved by the advisor in this skill's same commit window, citing this skill as the override authority.

If bm-cut returns success → proceed to push the new branch:
```bash
git -C C:/Users/barri/Developer/brehon-fork push -u origin phase-<target>
```

If bm-cut returns failure (working tree dirty post-bootstrap, name pattern mismatch, etc.) → refuse per hard refusal #7. Surface BM output verbatim.

### Phase 4 — Add lane worktree + walk bootstrap checklist

#### Step 4.1 — Worktree add

```bash
git -C C:/Users/barri/Developer/brehon-fork worktree add ../brehon-fork-<lane-suffix> phase-<target>
```

If the worktree path already exists → refuse per hard refusal #6.

#### Step 4.2 — Bootstrap walk

Per `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` steps 4-11. Each step is performed from the new lane CWD (`C:/Users/barri/Developer/brehon-fork-<lane-suffix>`):

| Step | Action | Verification |
|---|---|---|
| 4 | `cd C:/Users/barri/Developer/brehon-fork-<lane-suffix>` | `pwd` matches expected |
| 5 | `cp .mcp.json.example .mcp.json` | `grep PROJECT_MEMORY_DB .mcp.json` shows canonical absolute path `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` |
| 6 | Wire `pmd-canonical-guard.sh` SessionStart hook in `.claude/settings.local.json` (create or merge) | step 8 programmatic probe |
| 7 | (skipped — SessionStart banner check requires user opening CC) | — |
| 8 | Verify step 6 landed via python probe (per checklist line 41-53) | python prints `OK: pmd-canonical-guard.sh wired at SessionStart` |
| 9 | Wire `session-start-multi-lane-check.sh` SessionStart hook | step 10 programmatic probe |
| 10 | Verify step 9 landed via python probe (per checklist line 75-89) | python prints `OK: session-start-multi-lane-check.sh wired at SessionStart` |
| 11 | Verify tracked `refuse-ssh-reset-hard-shared-checkout.sh` PreToolUse hook is registered (per checklist line 93-107) | python prints `OK: refuse-ssh-reset-hard-shared-checkout.sh wired at PreToolUse` |

Each verification probe runs with `python -c "..."` per the checklist. If ANY probe fails → refuse per hard refusal #8. Do NOT proceed to Step 5 (roadmap flip).

The `.claude/settings.local.json` content is constructed by reading the example template + merging the two SessionStart hook entries per the snippets in the checklist lines 22-35 + 60-72. Use the `Write` tool (not raw `Edit` if file is fresh — `Write` is correct for new files; `Edit` is correct for existing files that need merge).

### Phase 5 — Flip roadmap entry + commit + push (atomic protocol)

Per `.claude/rules/multi-lane-worktree.md` Hard refusal #6 atomic protocol — read-mutate-commit-push in a single uninterrupted sequence. Race class: another session committing to `governance-v0` between fetch and push.

From canonical CWD (not the lane worktree — the roadmap commit lives on `governance-v0`):

```bash
cd C:/Users/barri/Developer/brehon-fork
git fetch origin governance-v0
```

Read roadmap fresh (a concurrent session may have rewritten it). Mutate:

```python
import io, json, datetime
r = json.load(io.open('.claude/PRPs/v1-roadmap.json', encoding='utf-8'))

# Locate the sub-phase entry in its lane
TARGET = '<target>'           # e.g. 'v1-RT-r2'
LANE = '<lane-id>'            # e.g. 'RT'
WORKTREE_PATH = 'C:/Users/barri/Developer/brehon-fork-<lane-suffix>'

entry = r['lanes'][LANE]['sub_phases'][TARGET]
assert entry['status'] == 'unstarted', f"expected unstarted; got {entry['status']}"

entry['status'] = 'in_flight'
entry['worktree'] = WORKTREE_PATH
entry['in_flight_since'] = datetime.datetime.utcnow().strftime('%Y-%m-%d')

# If lane status was 'partial' or 'done' (capstone re-open), don't touch lane-level status here
# Just per-sub-phase update.

# Bump roadmap-level metadata
r['last_updated_at'] = datetime.datetime.utcnow().isoformat() + 'Z'

with io.open('.claude/PRPs/v1-roadmap.json', 'w', encoding='utf-8') as f:
    json.dump(r, f, indent=2, ensure_ascii=False)
```

Stage + commit + push **as a single shell sequence** (minimise window between mutate and commit):

```bash
git add .claude/PRPs/v1-roadmap.json
git status .claude/PRPs/v1-roadmap.json   # verify ONLY this file is staged
git commit -m "chore(advisor): roadmap-next flip <target> in_flight + cut lane

Cut phase-<target> off governance-v0 + opened lane worktree at
<WORKTREE_PATH>. Bootstrap checklist (steps 4-11) verified.

Roadmap entry <target>: status unstarted -> in_flight
Worktree: <WORKTREE_PATH>"
git push origin governance-v0
```

After push, verify the commit survived (`git log -1`). If `git push` returned non-fast-forward → re-fetch, re-read, re-mutate, re-commit, re-push (up to 3 attempts). On 3rd failure → refuse per hard refusal #10; surface the race + exit.

If push reports "nothing added" — a concurrent commit clobbered the working-tree change between mutate and `git add`. Re-run from `git fetch`. (Same recipe per `.claude/rules/multi-lane-worktree.md` Hard refusal #6 step 5.)

### Phase 6 — Append runlog entry + print hand-off

Append one line to `.claude/runlog/bm-runlog.md` (canonical-side audit trail):

```bash
cat >> .claude/runlog/bm-runlog.md <<EOF

## advisor: roadmap-next cut phase-<target> — $(date -u +"%Y-%m-%dT%H:%M:%SZ")
- **branch:** phase-<target> (pushed to origin)
- **worktree:** <WORKTREE_PATH>
- **bootstrap:** ✓ .mcp.json   ✓ pmd-canonical-guard   ✓ multi-lane-check   ✓ ssh-reset-hard-refuse
- **roadmap:** <target> status unstarted -> in_flight
- **next:** user opens CC in worktree, runs /auto-roadmap
EOF

git add .claude/runlog/bm-runlog.md
git commit -m "chore(advisor): runlog — roadmap-next cut phase-<target>"
git push origin governance-v0
```

Then print the hand-off:

```
=== Lane cut: phase-<target> ===

Worktree:   C:/Users/barri/Developer/brehon-fork-<lane-suffix>
Branch:     phase-<target> (pushed to origin)
Bootstrap:  ✓ .mcp.json
            ✓ pmd-canonical-guard SessionStart wired
            ✓ multi-lane-check SessionStart wired
            ✓ refuse-ssh-reset-hard PreToolUse verified
Roadmap:    <target> status unstarted -> in_flight (commit <sha>)

Next steps (manual hand-off):
  1. Open Claude Code in:
     C:/Users/barri/Developer/brehon-fork-<lane-suffix>
  2. At session start, verify NO WARN from SessionStart hooks
     (pmd-canonical-guard.sh + session-start-multi-lane-check.sh)
  3. Run /auto-roadmap

/auto-roadmap will:
  - Identify <target> from the branch name
  - Check for an existing plan at .claude/PRPs/plans/<target>*.plan.md
  - If plan missing: auto-author planning brief from PRD + entry-kind
    registry, surface for your approval via AskUserQuestion, dispatch
    planning Junior, then six-mandatory-gate /auto-phase run
  - If plan exists: pre-seed auto-state JSON at impl-cohort-1 and
    invoke /auto-phase as a black box
```

Exit.

## Refusals

Per `.claude/rules/auto-roadmap.md` hard refusals #1-#10:

1. CWD not canonical → refuse.
2. Branch not `governance-v0` → refuse.
3. Working tree dirty → refuse; surface `git status --short`.
4. Roadmap missing or malformed → refuse; surface parse error.
5. No eligible unstarted sub-phase → refuse; surface roadmap summary.
6. Lane worktree already exists → refuse; surface existing path.
7. bm-cut fails → refuse; surface BM output.
8. Bootstrap-checklist verification fails → refuse; surface failing probe.
9. `git push` fails (non-network) → refuse; surface git output.
10. Atomic-protocol race on 3rd attempt → refuse; surface race + exit.

## What this skill is NOT

- **NOT** a multi-sub-phase loop — one invocation = one sub-phase cut.
- **NOT** an automatic launch of `/auto-roadmap` — manual hand-off is deliberate.
- **NOT** a `/auto-phase` replacement — `/auto-phase` still runs inside skill 2 unchanged.
- **NOT** a `bm-cut` replacement — `bm-cut` is dispatched verbatim.
- **NOT** authorized to mutate `crates/`, `migrations/`, `tests/`, or `docs/` — four-role model preserved.
- **NOT** authorized to mutate PRDs or plans — read-only on those.

## See also

- `.claude/rules/auto-roadmap.md` — the orchestration rule + hard refusals.
- `.claude/rules/multi-lane-worktree.md` — worktree discipline.
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` — the 11-step checklist Phase 4 walks.
- `.claude/commands/bm/bm-cut.md` — the BM verb Phase 3 dispatches.
- `~/.claude/commands/auto-roadmap.md` — skill 2, the hand-off target.
- `.claude/PRPs/v1-roadmap.json` — the roadmap.
- `.claude/PRPs/specs/auto-roadmap-skill-pair.md` — the authoring spec.

## Pre-commit dogfood

Mentally simulated against the current roadmap state (commit `98cccc734`, trunk @ `e9caf8933` at authoring time):

- T+0: user runs `/roadmap-next` from canonical `brehon-fork` on `governance-v0`. Working tree clean.
- T+5s: Phase 0 prereqs pass. CWD canonical, branch `governance-v0`, trunk up-to-date with origin, roadmap loads at `$schema_version: 1`, `git worktree list` shows `brehon-fork-fed-in-d` + `brehon-fork-quality-r1` + `brehon-fork-tooling` active (all `skipped` in roadmap).
- T+10s: Phase 1 — parse `what_remains.high_priority_unstarted` → primary candidates: `v1-ship-2`, `v1-ship-3` (SR); `v1-RT-r2`, `v1-RT-r3`, `v1-RT-r4`, `v1-RT-r5` (RT). `implementation_steering.next_logical_sub_phase` names "Resolve federation-inbound-d + quality-r1 first, then plan v1-ship-2 to continue closing the v0 ship-gate before tackling RT-r2..r5". Both in-flight lanes are `skipped` per user steer (not blocking new cuts of OTHER lanes), so the recommendation defers to the next item in `high_priority_unstarted`: **`v1-ship-2`** (SR lane primary).
- T+15s: Phase 2 — AskUserQuestion fires:
  > "Roadmap recommends cutting v1-ship-2 next (lane: SR, rationale: PRD §1 names 3 sequenced sub-phases; ship-1 + remediation cycles shipped, ship-2/3 unstarted). Confirm?"
  > Options: (1) Cut phase-v1-ship-2 (Recommended) / (2) Pick a different unstarted sub-phase / (3) Defer
- Suppose user picks option 2, then `v1-RT-r2` (mechanically derivable scope per registry — easier first dogfood than SR).
- T+~30s: `<target>` = `v1-RT-r2`, `<lane-suffix>` = `rt-r2`, `<lane-id>` = `RT`.
- T+~30s: Phase 3 — dispatch `branch-manager` subagent with `v1-RT-r2`. BM hits the plan-file STOP at its Phase 2 (no plan exists); skill 1 catches the BM DQ "cut anyway" override and resumes. BM cuts `phase-v1-RT-r2` locally. Skill 1 runs `git push -u origin phase-v1-RT-r2`. Branch on origin.
- T+~2min: Phase 4.1 — `git worktree add ../brehon-fork-rt-r2 phase-v1-RT-r2`. New directory at `C:/Users/barri/Developer/brehon-fork-rt-r2`.
- T+~2min30s: Phase 4.2 — bootstrap walk. `cp .mcp.json.example .mcp.json`. `grep PROJECT_MEMORY_DB .mcp.json` shows the canonical absolute path ✓. Edit `.claude/settings.local.json` to wire both SessionStart hooks. Python probes for steps 8, 10, 11 all return `OK`.
- T+~3min: Phase 5 — flip roadmap. `git fetch origin governance-v0` (no new commits). Read roadmap, set `lanes.RT.sub_phases.v1-RT-r2.status = 'in_flight'`, add `worktree` field. `git add` + verify staging shows ONLY `v1-roadmap.json` + commit + push. Push succeeds first try.
- T+~3min30s: Phase 6 — append runlog line, commit, push. Print hand-off.

What worked:
- BM's "plan missing" DQ is handled as an expected exception path, not a real STOP. The override is documented in this skill body; the BM brief honours it via the explicit override flag.
- Atomic protocol: single shell sequence for fetch→read→mutate→add→commit→push minimises the race window.
- The 11-step bootstrap checklist's programmatic verification at steps 8/10/11 catches the case where step 6 / 9 fail to land (file-clobber, JSON-merge bug).

What I worked through that this skill must NOT do:
- DO NOT auto-launch Claude Code in the new worktree. Manual hand-off is the natural session boundary where the user verifies hooks fired without WARN.
- DO NOT auto-run `/auto-roadmap`. Same reason.
- DO NOT silently override BM's plan-missing DQ. The override is documented in this skill body; BM still records the DQ as evidence trail.
- DO NOT push the new branch with empty content. The push happens after bm-cut completes, which establishes the local branch with a single empty commit (per bm-cut.md Phase 3) — that's fine for push.
- DO NOT bump roadmap `$schema_version` inside this skill. Schema bumps are separate `chore(advisor): roadmap schema bump` commits per `.claude/rules/auto-roadmap.md` state-routing invariant #8.
