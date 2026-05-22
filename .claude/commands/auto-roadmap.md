Lane-worktree sub-phase driver. Reads the sub-phase from the current `phase-v*-*` branch name, checks for an existing plan, dispatches a planning Junior if the plan is missing (with user gate to approve the auto-authored brief), pre-seeds the auto-state JSON, invokes `/auto-phase` as a black box, and on `/auto-phase` returning `done` flips the roadmap entry from `in_flight` → `done` and prints the next-sub-phase hint.

Argument: none. The skill derives its target sub-phase from `git branch --show-current`.

**This is skill 2 of a two-skill chain.** Skill 1 is `/roadmap-next` (runs in canonical `brehon-fork`). Skill 2 expects to run in a lane-dedicated worktree (`brehon-fork-<lane-suffix>`) on a `phase-v1-<sub-phase>` branch — typically the worktree just cut by skill 1. Per `.claude/rules/auto-roadmap.md` state-routing invariant #6, the hand-off between skills is manual: the user opens Claude Code in the lane worktree and runs `/auto-roadmap` after `/roadmap-next` exits.

## Why this shape

Skill 2's job is to make `/auto-phase` roadmap-aware without modifying `/auto-phase` itself. `/auto-phase` is single-sub-phase orchestration; skill 2 wraps it with:

1. **Plan-gap handler.** If `.claude/PRPs/plans/<sub-phase>.plan.md` doesn't exist (the expected case for unstarted RT-r2..r5), skill 2 auto-authors a planning brief from the PRD + entry-kind registry, gates user approval on the brief, dispatches the planning Junior, runs the clarify gate, and waits for the planning + DoD smoke gate to pass before pre-seeding `/auto-phase`. This is the chunk of work skill 2 adds.

2. **Pre-seed `/auto-phase`'s auto-state JSON.** Once planning is approved, skill 2 writes `.claude/auto-state/<sub-phase>.json` directly with `stage: "impl-cohort-1"` (or `stage: "init"` if `bm-cut` is also needed — but in the skill-1/skill-2 chain, bm-cut already ran in skill 1, so `impl-cohort-1` is the canonical pre-seed). `/auto-phase` then resumes into that state and proceeds normally.

3. **Roadmap update at sub-phase end.** When `/auto-phase` returns `done`, skill 2 flips the roadmap entry `in_flight` → `done`, populates PR + merge SHA + retro + verify paths from the auto-state JSON, commits, pushes. The roadmap update is the formal "lane progresses" signal that future `/roadmap-next` invocations read.

The plan-gap auto-dispatch is the new piece of advisor logic. Everything else is composition.

## What `/auto-roadmap` automates

| Step | Without skill (manual) | With skill |
|---|---|---|
| Identify sub-phase from current branch | Advisor reads `git branch --show-current` and pattern-matches | Auto |
| Plan-gap check | Advisor globs `.claude/PRPs/plans/<sub-phase>*.plan.md` | Auto |
| Auto-author planning brief from PRD + entry-kind registry | Advisor reads PRD sections + registry rows, hand-authors brief | Auto (then surfaces brief for user approval via AskUserQuestion) |
| Clarify gate per advisor-orchestrator.md §3.3 | Advisor invokes `/brehon-clarify` manually | Auto (invokes `/brehon-clarify` per the gate's existing semantics) |
| Dispatch planning Junior | Advisor types `mcp__junior-brehon__create_task` | Auto |
| Wait for planning Junior completion | Advisor polls every ~10min | Auto (ScheduleWakeup 1200s → 600s cadence) |
| DoD smoke + watchpoint specificity gates per §3.4 + §3.5 | Advisor runs §15 commands, scans plan §4 | Auto |
| Pre-seed auto-state JSON for `/auto-phase` | Advisor writes the JSON manually | Auto (per `.claude/PRPs/templates/auto-phase-state.template.json` schema) |
| Invoke `/auto-phase <sub-phase>` | Advisor types `/auto-phase v1-RT-r2` | Auto (Skill tool invocation) |
| Mark sub-phase `done` in roadmap at end | Advisor edits roadmap, commits, pushes | Auto (atomic protocol) |

## What `/auto-roadmap` does NOT automate

- **The six mandatory `/auto-phase` user gates.** All six fire inside `/auto-phase` as normal. Skill 2 adds at most two extra gates (the "approve next sub-phase" gate is owned by skill 1, NOT skill 2; the plan-gap "approve auto-authored brief" gate is owned by skill 2 ONLY when plan is missing).
- **Looping into the next sub-phase.** Skill 2 is strict single-sub-phase. It exits cleanly when `/auto-phase` returns `done`. The user closes Claude Code in this worktree, prunes the worktree manually (`git worktree remove`), opens canonical CC, and runs `/roadmap-next` for the next sub-phase.
- **Catch-fire recovery.** If `/auto-phase` returns `catch-fire`, skill 2 propagates the catch-fire and exits. The user takes over per `/auto-phase`'s catch-fire surface, possibly via `/auto-phase --start-from <stage>` directly.
- **Editing the PRD or roadmap structure.** Skill 2 reads PRDs; it doesn't write them. Skill 2's roadmap mutations are field-level (status, pr, merge_commit, retro, verify) only — never structural.
- **Removing the lane worktree.** That's manual per `.claude/rules/multi-lane-worktree.md` §"Lifecycle" step 3.

## Procedure

### Phase 0 — Prerequisites + sub-phase identification

Verify before any state-changing call:

1. **CWD is a lane-dedicated worktree** matching `brehon-fork-<lane-suffix>`:
   ```bash
   pwd
   ```
   If CWD is canonical `brehon-fork` or any non-lane path → refuse per `.claude/rules/auto-roadmap.md` hard refusal #11.

2. **Branch is `phase-v*-*`** (not `governance-v0`, `main`, or any other shape):
   ```bash
   BRANCH=$(git branch --show-current)
   echo "$BRANCH"
   ```
   Pattern: `^phase-v[0-9]+-[a-z][a-z0-9-]*$`. If pattern mismatch → refuse per hard refusal #12.

3. **Extract sub-phase identifier** from branch name:
   - `phase-v1-RT-r2` → sub-phase = `v1-RT-r2`, lane = `RT`
   - `phase-v1-ship-2` → sub-phase = `v1-ship-2`, lane = `SR` (per roadmap mapping)
   - `phase-v1-AD-f` → sub-phase = `v1-AD-f`, lane = `AD`

   The branch name strips `phase-` prefix to get sub-phase. The lane key is derived by reading the roadmap and matching the sub-phase against each lane's `sub_phases` map. Compute the lane key in python:

   ```python
   import io, json
   r = json.load(io.open('.claude/PRPs/v1-roadmap.json', encoding='utf-8'))
   SUB_PHASE = '<extracted from branch>'
   LANE = next((k for k, v in r['lanes'].items() if SUB_PHASE in v.get('sub_phases', {})), None)
   assert LANE, f'sub-phase {SUB_PHASE} not in any lane'
   print(f'sub_phase={SUB_PHASE}  lane={LANE}')
   ```

4. **Roadmap entry status == `in_flight`** (skill 1 should have flipped it):
   ```python
   entry = r['lanes'][LANE]['sub_phases'][SUB_PHASE]
   assert entry['status'] == 'in_flight', f'expected in_flight; got {entry["status"]}'
   ```
   If status is `unstarted` → refuse with "skill 1 should have flipped this; re-run `/roadmap-next` from canonical" per hard refusal #13.
   If status is `done` → refuse per hard refusal #14.

5. **`/auto-phase` skill body exists** at `~/.claude/commands/auto-phase.md`:
   ```bash
   test -f ~/.claude/commands/auto-phase.md
   ```
   If absent → refuse per hard refusal #15.

6. **SessionStart hooks fired without WARN** — `.claude/hooks/pmd-canonical-guard.sh` and `.claude/hooks/session-start-multi-lane-check.sh`. The user should have verified this at session-open (per skill 1's hand-off output). Skill 2 cannot directly re-run these (they're SessionStart-time only) but it can sanity-check the wiring:

   ```python
   import io, json
   s = json.load(io.open('.claude/settings.local.json', encoding='utf-8'))
   ss = s.get('hooks', {}).get('SessionStart', [])
   pmd_wired = any('pmd-canonical-guard.sh' in h.get('command', '') for entry in ss for h in entry.get('hooks', []))
   multi_wired = any('session-start-multi-lane-check.sh' in h.get('command', '') for entry in ss for h in entry.get('hooks', []))
   assert pmd_wired and multi_wired, f'SessionStart wiring incomplete: pmd={pmd_wired} multi={multi_wired}'
   print('OK: SessionStart hooks wired')
   ```

   If wiring missing → surface to user; ask whether to (a) re-apply per `feedback_phase_lane_worktree_bootstrap_checklist.md` steps 6+9, or (b) proceed anyway (user accepts the risk).

7. **Working tree clean** in lane worktree:
   ```bash
   git status --short
   ```
   If dirty → surface; ask user how to proceed.

### Phase 0.5 — Plan-gap handler

Glob for an existing plan:

```bash
ls .claude/PRPs/plans/<sub-phase>*.plan.md 2>/dev/null
```

**Plan exists** → record path; skip to Phase 1.

**Plan missing** → enter the plan-gap handler (the largest piece of new logic in skill 2):

#### Step 0.5.1 — Read PRD + entry-kind registry for sub-phase scope

The roadmap entry's `lane.prd` field names the PRD path (e.g. `.claude/PRPs/prds/v1-reputation-tuning.prd.md`). The entry's own `scope` field has the high-level scope (e.g. for v1-RT-r2: "v1 decay calculator goes live; admin_config emits ENTRY_KIND_DECAY_KNOB_CHANGED on first decay-key write; flips feature.reputation_v1_decay_enabled").

For RT lane (the expected first dogfood target), the entry-kind registry at `.claude/rules/governance-log-entry-kind-registry.md` "v1-RT-r1 entry kinds" section names the handler file for each emitting const. Cross-reference:

```python
import re
registry = open('.claude/rules/governance-log-entry-kind-registry.md').read()
# Find rows matching this sub-phase
sub_phase_rows = re.findall(
    r'\| `(ENTRY_KIND_\w+)` \|.*?\| v1-RT-r1 const; ' + re.escape(SUB_PHASE) + r' call site \|.*?\| ' + re.escape(SUB_PHASE) + r' `([^`]+)` ([^|]+)\|',
    registry,
)
for const, file_path, qualifier in sub_phase_rows:
    print(f'  {const} fires at {file_path} ({qualifier.strip()})')
```

The registry rows give the handler file paths the sub-phase must touch. The PRD §5/§7/§8 sections give the data-model and feature-flag context. Together they form the scope.

Read PRD sections referenced by the entry's `scope` field (e.g. for RT-r2 read PRD §10 "Security" + §6 around feature flag semantics). Also read the canonical sibling brief (the most recent same-lane brief) to mirror its shape — for RT-r2, that's `.claude/PRPs/briefs/rt-r1-planning-1.md` (since r1 is the most recent shipped lane brief).

#### Step 0.5.2 — Auto-author planning brief

Write the brief at `.claude/PRPs/briefs/<sub-phase-lowercase>-planning-1.md` (e.g. `.claude/PRPs/briefs/rt-r2-planning-1.md`). The brief follows the canonical-sibling shape (§1 Role + dispatch line, §2 Scope, §3 Required reading, §4 Constraints, §5 Pre-commit dogfood, §6 Acceptance).

**Key scope content** must include:
- The PRD section anchor (e.g. "PRD §15 row 2 + §10")
- The entry-kind registry rows naming the handler files this sub-phase touches
- The canonical sibling brief reference + the canonical sibling plan reference
- A "Scope boundary" sub-section listing what's NOT in this sub-phase (per the canonical-sibling brief's §2.3 pattern)
- Per advisor-orchestrator.md §2.4: walk the file list against the file-class lesson table and inject every matching lesson into §3 Required reading

The brief is committed to the **lane branch** (`phase-<sub-phase>`), not `governance-v0`, per `advisor-orchestrator.md` §2.1 — impl/planning briefs land on the phase branch the worker forks from.

#### Step 0.5.3 — User gate: approve auto-authored brief (extra gate, beyond /auto-phase's six)

Per `.claude/rules/auto-roadmap.md` hard refusal #17 — auto-dispatch without user confirmation is a hard refusal. Fire AskUserQuestion:

```
question: "Auto-authored planning brief for <sub-phase> at <brief-path>. Confirm dispatch to planning Junior?"
header: "Brief approval"
options:
  1. "Confirm and dispatch (Recommended)" — proceed to clarify gate
  2. "Surface brief for review (paste content into chat)" — print brief body inline
  3. "Edit brief manually first" — exit; user edits brief, re-runs /auto-roadmap
  4. "Abort" — clean exit
```

If user picks option 2, print the brief body inline + re-fire the question. If user picks option 3 or 4, exit cleanly. If user picks option 1, commit the brief to the lane branch:

```bash
git add .claude/PRPs/briefs/<sub-phase-lowercase>-planning-1.md
git commit -m "chore(advisor): auto-roadmap auto-author planning brief for <sub-phase>"
git push origin phase-<sub-phase>
```

#### Step 0.5.4 — Clarify gate (mandatory per advisor-orchestrator.md §3.3)

Invoke `/brehon-clarify <brief-path>` via the Skill tool. This is the standard pre-planning clarify gate. It writes `kind: "clarify"` DQ entries (advisor-mode self-answer or user-relay). Wait for every clarify-DQ to be resolved.

#### Step 0.5.5 — Dispatch planning Junior

Use `mcp__junior-brehon__create_task` with `base_branch=phase-<sub-phase>`. Task description (under 100 chars):

```
[role:planning] <sub-phase> plan — see .claude/PRPs/briefs/<sub-phase-lowercase>-planning-1.md
```

Record the Junior task id in the auto-state JSON pre-seed (Phase 0.7 below).

#### Step 0.5.6 — Wait for planning Junior + run DoD smoke + watchpoint specificity gates

Per `advisor-orchestrator.md` §3.4 + §3.5. ScheduleWakeup cadence: 1200s initial, 600s subsequent (planning Juniors typically run 30-90 min).

When planning Junior completes:
- Run the §15 DoD smoke test against current HEAD per `feedback_pre_phase_dod_smoke_test.md` — every command literally.
- Verify §4 watchpoints cite specific tables/files per `feedback_advisor_watchpoint_specificity.md`.
- Surface plan + DoD result via the standard plan-approval AskUserQuestion gate (this is `/auto-phase`'s gate 1, but skill 2 fires it here BEFORE handing off to `/auto-phase`).

If user approves → proceed to Phase 0.7 (pre-seed). If user rejects → exit; the plan needs revision and the next session re-runs `/auto-roadmap` after the plan is fixed.

### Phase 0.7 — Pre-seed auto-state JSON

This is the option-1 invocation mechanism per spec §3.4 (recommended). Write `.claude/auto-state/<sub-phase>.json` directly with `stage: "impl-cohort-1"` so `/auto-phase` resumes into the impl phase, skipping bm-cut + planning (both already done).

Per `.claude/PRPs/templates/auto-phase-state.template.json` schema. The pre-seed shape:

```python
import io, json, datetime, secrets, subprocess
import os

SUB_PHASE = '<extracted>'
PHASE_TIP = subprocess.check_output(['git', 'rev-parse', 'phase-' + SUB_PHASE]).decode().strip()
NOW = datetime.datetime.utcnow().isoformat() + 'Z'
TRUNK_SHA = subprocess.check_output(['git', 'rev-parse', 'governance-v0']).decode().strip()

# Read auto-state template to anchor schema shape
template = json.load(io.open('.claude/PRPs/templates/auto-phase-state.template.json', encoding='utf-8'))

pre_seed = {
    'schema_version': template.get('schema_version', 1),
    'phase': SUB_PHASE,
    'started_at': NOW,
    'session_id': secrets.token_hex(6),
    'last_session_ended_at': None,
    'resume_count': 0,
    'last_known_phase_tip': PHASE_TIP,
    'stage': 'impl-cohort-1',
    'current_cohort': None,                 # /auto-phase computes from §13 [P] markers at impl-cohort-1 entry
    'phase_2_e2e_mode': None,
    'phase_2_e2e_mode_chosen_at': None,
    'junior_tasks': {
        '_note': 'pre-seeded by /auto-roadmap; bm-cut + planning already done',
        'bm-cut': None,                     # null — happened in skill 1 (no recorded id here)
        'planning': '<id-of-planning-junior>',  # if plan-gap fired
        'bm-pr': None,
        'bm-poll-cr': None,
        'bm-triage': None,
    },
    'last_action': 'auto-roadmap pre-seed at impl-cohort-1',
    'last_action_at': NOW,
    'last_poll_at': None,
    'last_fetch_at': None,
    'last_dq_pending_count': 0,             # /auto-phase Phase 0.5 reconciliation will refresh
    'last_dq_pending_ids': [],
    'user_gate_history': [
        {
            'gate': 'roadmap-recommendation',
            'decided_at': NOW,
            'decision': 'confirm',
            'notes': f'cut by /roadmap-next; ratified by /auto-roadmap pre-seed',
        }
    ],
    'push_grants_used': 0,
    'catch_fire': None,
    'trunk_sha_at_init': TRUNK_SHA,
    '_pre_seeded_by': '/auto-roadmap',
    '_pre_seed_notes': (
        'Pre-seeded at stage=impl-cohort-1 because skill 1 (/roadmap-next) '
        'already ran bm-cut and the auto-roadmap plan-gap handler already '
        'ran planning (if plan was missing). /auto-phase resumes into '
        'impl-cohort-1 via its Phase 0.5 reconciliation path.'
    ),
}

# If pre-existing auto-state JSON exists with stage != init, refuse per hard refusal #19
import os
state_path = f'.claude/auto-state/{SUB_PHASE}.json'
if os.path.exists(state_path):
    existing = json.load(io.open(state_path, encoding='utf-8'))
    assert existing.get('stage') == 'init', \
        f'auto-state JSON exists at stage={existing.get("stage")}; refusing pre-seed clobber'

os.makedirs('.claude/auto-state', exist_ok=True)
with io.open(state_path, 'w', encoding='utf-8') as f:
    json.dump(pre_seed, f, indent=2, ensure_ascii=False)
```

Auto-state JSON is gitignored (per `.gitignore` line `.claude/auto-state/`); no commit needed. The file is per-worktree runtime state.

### Phase 1 — Invoke `/auto-phase` as a black box

```
Skill tool: skill=auto-phase, args=<sub-phase>
```

`/auto-phase` detects the pre-seeded state file (per its Phase 0.5 — "if file EXISTS → session resume path"). Routes through Phase 0.5 reconciliation. Sees `stage: "impl-cohort-1"`. Resumes into the impl phase. The six mandatory `/auto-phase` user gates fire as normal:

- Plan approval gate: ALREADY FIRED in Phase 0.5.6 if plan-gap ran (or implied-approved if plan pre-existed; `/auto-phase` records the gate as already-passed). The skill prints "plan approval pre-confirmed by /auto-roadmap (gate fired in plan-gap handler at <ts>)" if applicable.
- Judgment-heavy DQ gate: fires per `/auto-phase` normal path.
- CR triage gate: fires per `/auto-phase` normal path.
- Phase 2 e2e local-vs-dispatch gate: fires per `/auto-phase` normal path.
- Merge confirm gate: fires per `/auto-phase` normal path.
- Retro sign-off gate: fires per `/auto-phase` normal path.

Skill 2 is effectively idle while `/auto-phase` runs. When `/auto-phase` returns:

| `/auto-phase` exit state | Skill 2 action |
|---|---|
| `done` | Proceed to Phase 2 (roadmap flip) |
| `catch-fire` | Propagate per hard refusal #18: surface the catch-fire reason verbatim + `/auto-phase`'s recovery instructions; exit |
| Anything else | Treat as in-flight; cannot happen post-return; if it does, refuse and surface |

### Phase 2 — Roadmap update (atomic protocol)

`/auto-phase` returned `done`. Read the auto-state JSON for the recorded PR + merge SHA + retro path + verify path:

```python
import io, json
state = json.load(io.open(f'.claude/auto-state/{SUB_PHASE}.json', encoding='utf-8'))
# Recorded by /auto-phase at its done-stage transition
PR_NUMBER = state.get('pr_number')           # e.g. 142
MERGE_COMMIT = state.get('merge_commit_sha') # e.g. 'a27812631'
RETRO_PATH = state.get('retro_path')         # e.g. '.claude/PRPs/reports/v1-RT-r2-retro.md'
VERIFY_PATH = state.get('verify_path')       # e.g. '.claude/PRPs/reports/v1-RT-r2-verify.md'
```

Then mutate roadmap with the atomic protocol per `.claude/rules/multi-lane-worktree.md` Hard refusal #6:

```bash
# Single uninterrupted shell sequence
cd C:/Users/barri/Developer/brehon-fork-<lane-suffix>   # currently in this worktree
git fetch origin governance-v0
```

```python
import io, json, datetime
r = json.load(io.open('.claude/PRPs/v1-roadmap.json', encoding='utf-8'))
entry = r['lanes'][LANE]['sub_phases'][SUB_PHASE]

assert entry['status'] == 'in_flight', f'expected in_flight; got {entry["status"]}'

entry['status'] = 'done'
entry['pr'] = PR_NUMBER
entry['merge_commit'] = MERGE_COMMIT
entry['retro'] = RETRO_PATH
entry['verify'] = VERIFY_PATH
entry['done_at'] = datetime.datetime.utcnow().strftime('%Y-%m-%d')
# entry['worktree'] field remains as durable record of where the work happened
# (user prunes the worktree manually; the roadmap entry is read-only by then)

r['last_updated_at'] = datetime.datetime.utcnow().isoformat() + 'Z'

with io.open('.claude/PRPs/v1-roadmap.json', 'w', encoding='utf-8') as f:
    json.dump(r, f, indent=2, ensure_ascii=False)
```

```bash
git add .claude/PRPs/v1-roadmap.json
git status .claude/PRPs/v1-roadmap.json
git commit -m "chore(advisor): auto-roadmap flip <sub-phase> done — PR #${PR_NUMBER}

Sub-phase <sub-phase> merged via PR #${PR_NUMBER} (commit ${MERGE_COMMIT}).
Retro: ${RETRO_PATH}
Verify: ${VERIFY_PATH}

Roadmap entry <sub-phase>: status in_flight -> done"
git push origin governance-v0
```

The lane worktree shares `.git/` with canonical, so the commit lands on `governance-v0` directly. Per hard refusal #20: 3-attempt retry on non-fast-forward race.

### Phase 3 — Surface next sub-phase, exit

Read the roadmap again. Compute the next eligible unstarted sub-phase (same logic as `/roadmap-next` Phase 1).

Print:

```
=== /auto-roadmap <sub-phase> — done ===

PR:    #<PR_NUMBER>
Merge: <MERGE_COMMIT>
Retro: <RETRO_PATH>
Verify: <VERIFY_PATH>

Roadmap updated: <sub-phase> status in_flight -> done (commit <sha>)

Next unstarted sub-phase in roadmap: <next-sub-phase>
  (lane: <next-lane>, rationale: <one-line>)

To continue:
  1. Close Claude Code in this worktree
  2. Run `git worktree remove C:/Users/barri/Developer/brehon-fork-<lane-suffix>`
     from canonical brehon-fork
  3. Open Claude Code in C:/Users/barri/Developer/brehon-fork (canonical)
  4. Run /roadmap-next
```

Exit cleanly. Skill 2 does NOT auto-prune the worktree (write-while-in-use risk per spec §3.2 risk table); the user runs `git worktree remove` manually after closing the lane's CC session.

## Refusals

Per `.claude/rules/auto-roadmap.md` hard refusals #11-#20:

11. Wrong CWD (not lane-dedicated) → refuse.
12. Branch not `phase-v*-*` → refuse.
13. Roadmap entry missing or wrong status → refuse.
14. Sub-phase already merged → refuse.
15. `/auto-phase` skill body missing → refuse.
16. Plan-gap AND PRD-section-missing → refuse.
17. Plan-gap auto-dispatch without user confirmation → hard refusal — always fire the brief approval gate.
18. `/auto-phase` returns catch-fire → propagate, exit.
19. Pre-seeded auto-state JSON conflicts with existing → refuse.
20. Atomic-protocol race on 3rd attempt → refuse.

## What this skill is NOT

- **NOT** a multi-sub-phase loop. Single sub-phase per invocation.
- **NOT** a `/auto-phase` replacement. It composes on top.
- **NOT** authorized to mutate `crates/`, `migrations/`, `tests/`, `docs/`. Four-role model preserved.
- **NOT** authorized to edit PRDs.
- **NOT** authorized to edit the `/auto-phase` skill body, rule, or template.
- **NOT** a catch-fire recovery handler.

## See also

- `.claude/rules/auto-roadmap.md` — the orchestration rule + hard refusals.
- `~/.claude/commands/auto-phase.md` — the skill skill 2 composes on.
- `.claude/rules/auto-phase.md` — `/auto-phase` state-machine source.
- `.claude/rules/advisor-orchestrator.md` — clarify gate + DoD smoke + watchpoint gates skill 2 honours.
- `.claude/rules/multi-lane-worktree.md` — worktree-per-lane discipline.
- `.claude/rules/decision-queue.md` — attribution + atomic protocol.
- `.claude/PRPs/templates/auto-phase-state.template.json` — auto-state JSON schema.
- `.claude/PRPs/v1-roadmap.json` — the roadmap.
- `.claude/PRPs/specs/auto-roadmap-skill-pair.md` — the authoring spec.

## Pre-commit dogfood

Mentally simulated against a hypothetical `v1-RT-r2` cut worktree (post-/roadmap-next), trunk @ `e9caf8933` at authoring time:

- T+0: user has `/roadmap-next`'d at canonical to cut `phase-v1-RT-r2`. Opens Claude Code in `C:/Users/barri/Developer/brehon-fork-rt-r2`. SessionStart banner shows no WARN (PMD canonical path correct, no other active lane within 30min window besides the steered-out ones).
- T+0: user types `/auto-roadmap`.
- T+5s: Phase 0 — CWD = `brehon-fork-rt-r2` ✓, branch = `phase-v1-RT-r2` ✓, sub-phase = `v1-RT-r2`, lane = `RT`. Roadmap entry status = `in_flight` ✓. `/auto-phase` exists ✓. SessionStart hook wiring probes pass ✓. Working tree clean ✓.
- T+10s: Phase 0.5 — glob `.claude/PRPs/plans/v1-RT-r2*.plan.md` → no match. Plan-gap fired.
- T+20s: Step 0.5.1 — read PRD `v1-reputation-tuning.prd.md` §15 row 2 (RT-r2 = decay calculator go-live + feature flag flip + first ENTRY_KIND_DECAY_KNOB_CHANGED fire). Read entry-kind registry "v1-RT-r1 entry kinds" — `ENTRY_KIND_DECAY_KNOB_CHANGED` row names `crates/api/api/src/governance/admin_config.rs` as the v1-RT-r2 fire site. Read canonical sibling brief `.claude/PRPs/briefs/rt-r1-planning-1.md` for shape.
- T+~1min: Step 0.5.2 — author brief at `.claude/PRPs/briefs/rt-r2-planning-1.md`. §1 dispatch line, §2 Scope = "decay calculator + feature flag + first fire site", §3 Required reading (PRD §15 row 2, §10, §6; registry row for `_DECAY_KNOB_CHANGED`; canonical sibling brief; mandatory file-class lessons for the handler file `admin_config.rs` — none mandatory since the file isn't in the file-class table for handlers, just the multi-write transaction rule if 2+ writes), §4 Constraints (no handler edits outside `admin_config.rs`, feature-flag toggle is the contract), §5 Pre-commit dogfood (against this PRD + registry), §6 Acceptance (DQ pending = 0 + clarify gate resolved).
- T+~1min30s: Step 0.5.3 — AskUserQuestion fires:
  > "Auto-authored planning brief for v1-RT-r2 at .claude/PRPs/briefs/rt-r2-planning-1.md. Confirm dispatch to planning Junior?"
  > Options: (1) Confirm and dispatch / (2) Surface brief for review / (3) Edit manually first / (4) Abort
- Suppose user picks option 2 (review). Brief printed inline. User picks option 1.
- T+~3min: commit brief to phase-v1-RT-r2, push.
- T+~3min30s: Step 0.5.4 — invoke `/brehon-clarify .claude/PRPs/briefs/rt-r2-planning-1.md`. Clarify produces 2 DQ entries (admin_config.rs already has 8 decay knobs from RT-r1; what's the trigger heuristic for "first decay-key write under v1 calculator"? Self-resolvable from PRD §6/§10). Both DQ resolved advisor-mode.
- T+~5min: Step 0.5.5 — dispatch `[role:planning] v1-RT-r2 plan — see .claude/PRPs/briefs/rt-r2-planning-1.md` Junior. Record id `#400` (hypothetical).
- T+~5min: ScheduleWakeup 1200s.
- T+~25min: planning Junior done. Read plan output. Run §15 DoD smoke (e.g. `cargo check --workspace --features full`, migration round-trip, e2e `parity_seed_count` — all PASS). Watchpoint gate: §4 watchpoints cite specific tables/files ✓.
- T+~30min: plan approval AskUserQuestion fires. User approves.
- T+~30min: Phase 0.7 — pre-seed `.claude/auto-state/v1-RT-r2.json` at `stage: "impl-cohort-1"`. Record planning Junior id #400 in `junior_tasks.planning`.
- T+~30min: Phase 1 — invoke `/auto-phase v1-RT-r2`. Black box from skill 2's perspective; runs ~3-4 hours.
- T+~4h: `/auto-phase` returns `done`. Reads auto-state JSON for PR + merge + retro paths. PR #142, merge commit (hypothetical), retro at `.claude/PRPs/reports/v1-RT-r2-retro.md`, verify at `.claude/PRPs/reports/v1-RT-r2-verify.md`.
- T+~4h: Phase 2 — roadmap update. `git fetch origin governance-v0`. Read roadmap. Set `lanes.RT.sub_phases.v1-RT-r2.status = 'done'`, populate fields. Commit + push. First-try push succeeds.
- T+~4h5m: Phase 3 — read roadmap again. Compute next: `v1-RT-r3`. Print hand-off.

What worked:
- Plan-gap auto-dispatch composes cleanly. The user is shown the auto-authored brief before any Junior is dispatched — they can edit/abort.
- Pre-seeded auto-state JSON makes `/auto-phase` resume into impl-cohort-1 cleanly. No `--start-from` flag needed; `/auto-phase`'s existing Phase 0.5 resume path does the right thing.
- The roadmap update at sub-phase end is atomic. The lane worktree shares `.git/` with canonical so the commit lands on `governance-v0` correctly.

What I worked through that this skill must NOT do mid-walk:
- DO NOT auto-prune the worktree. User does that manually after CC closes (write-while-in-use risk).
- DO NOT loop into v1-RT-r3 automatically. Manual hand-off back to canonical + `/roadmap-next` is the right boundary.
- DO NOT swallow `/auto-phase` catch-fires. Propagate per hard refusal #18.
- DO NOT skip the user gate on the auto-authored brief. Auto-dispatch is a hard refusal (#17).
- DO NOT mutate the `/auto-phase` skill body to teach it about pre-seeded state files. `/auto-phase`'s existing Phase 0.5 resume path handles it.
- DO NOT modify `.claude/auto-state/<other-phase>.json`. One sub-phase per invocation.
