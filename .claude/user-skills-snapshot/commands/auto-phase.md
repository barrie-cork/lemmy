Single-trigger sub-phase orchestrator. Drives `bm-cut → planning → impl cohorts → ci-watcher cycles → bm-pr → bm-poll-cr → bm-triage → bm-merge → retro` end-to-end, stops only at the six mandatory user gates.

Argument: `$ARGUMENTS` (required). Forms:
- `v1-SL-c-2` → drive the named sub-phase. **Auto-resumes if `.claude/auto-state/<phase>.json` exists** — same invocation works for fresh start AND post-compaction / new-session pickup. No flags needed.
- `v1-SL-c-2 --dry-run` → print the dispatch plan + cadence schedule, queue nothing
- `v1-SL-c-2 --resume` → explicit resume (no-op if state file exists, since auto-resume is the default; fails fast with clear message if state file missing)
- `v1-SL-c-2 --start-from <stage>` → **explicit override**, ignores any current `stage` field in the state file and routes to the named stage. Use after `catch-fire` or when manually fast-forwarding past a stuck stage. Valid stages: `init`, `bm-cut-running`, `planning-running`, `impl-cohort-N`, `phase-2-e2e-N`, `bm-pr-pending`, `cr-wait`, `cr-triage-pending-user`, `verify-running`, `merge-pending-user`, `retro-author`.
- `v1-SL-c-2 --no-bm-cut` → skip `bm-cut` (phase branch already exists; rare — usually `--start-from planning-running` is more honest)
- `v1-SL-c-2 --reset` → DESTRUCTIVE: archive current state file to `.claude/auto-state/<phase>-archived-<ts>.json` and start fresh from `init`. Asks for explicit confirmation before archiving. Use only after user has manually reverted a phase branch.
- `v1-SL-c-2 --unattended` → **opt-in partial-autonomy mode.** Drives every mechanical transition AND auto-clears the two allowlisted gates (gate 4 e2e→always local; gate 6 retro sign-off, after a 3-check sanity gate). The four judgment gates (1 plan-approval, 2 ADR/scope-DQ, 3 CR-triage, 5 merge-confirm) **park-and-ping**: the phase advances every transition up to the gate, fires a Telegram gate-waiting notification, and waits. No judgment, code, ADR, billing, or merge decision is ever auto-cleared. Default (flag absent) stays fully-gated — all six gates interactive. See "Phase 7 — `--unattended` gate allowlist" below for the hard contract.
- `v1-SL-c-2 --unattended --e2e dispatch` → as `--unattended`, but overrides gate 4's `local` default to the **billed, public-log** CI dispatch path. Use only when you explicitly want the public green-check (pilot / external PR). Absent this flag, `--unattended` always picks `local`.

This command is **NOT a fully-autonomous mode**. By default it respects every one of the six mandatory user gates. What it automates is the *transitions between gates* — the per-stage briefs, Junior dispatches, ci-watcher cycles, cohort barriers, and §G4 fix-impl auto-queueing that today force the advisor to re-derive the orchestration logic at every transition.

**`--unattended` is an opt-in partial-autonomy layer on top of that** — it does NOT make the mode "fully autonomous." It auto-clears exactly **two** gates that carry no judgment, code, ADR, billing, or outward-facing risk (gate 4 e2e→local, gate 6 retro sign-off) and **parks-and-pings** the other four. The historically-rejected `--auto-all-gates` (per L15) — which would have let a model clear plan-approval, ADR-DQ, CR-triage, or merge — stays rejected. The autonomy in `--unattended` comes from a **fixed policy allowlist**, not from a model making decisions. No cheap-model agent is spun up; the existing Opus advisor stays the poller (already near-free while sleeping on `ScheduleWakeup`), and the existing daemon completion hook + one advisor-fired gate-waiting ping carry the notifications.

## Why this shape

c-1 retro evidence (4h 21min wall-clock, ~15 user touchpoints) showed the advisor's orchestration logic is correct, but the *trigger surface* is too granular. Each transition is a fresh "what do I queue next?" decision even though the answer is fully determined by the plan + the four-role rules. `/auto-phase` compiles `.claude/rules/advisor-orchestrator.md` §"Stage-shape orchestration" + §"Cohort dispatch sequence" + §"§G4 classifier" into a state machine with calibrated cadences. Target: ≤80% wall-clock vs c-1 baseline, 6–8 user touchpoints per phase, ≤60% context-token spend.

## What `/auto-phase` automates (15 transitions, 0 judgment calls)

| Transition | Without skill (manual) | With skill |
|---|---|---|
| Brief author → Junior dispatch | Advisor commits brief, queues Junior, sets ScheduleWakeup | Auto |
| Junior `done` → next-stage brief | Advisor reads task output, writes next brief | Auto |
| Cohort barrier (all `[P]` members complete) | Advisor checks each member, computes next cohort | Auto |
| ci-watcher `result: pass` → cohort advance | Advisor mutates DQ if needed, queues next | Auto |
| ci-watcher `result: fail` (allowlist) → fix-impl | Advisor authors §G4 fix-impl brief, queues | Auto |
| Phase 1 all-pass → Phase 2 e2e raise | Advisor raises DQ, dispatches ci-watcher | Auto if user pre-picked local-vs-dispatch |
| `bm-pr` complete → `bm-poll-cr` queue | Advisor waits for CR, queues poll | Auto with cadence |
| All fix-in-PR done → `/brehon-verify` | Advisor runs verify, surfaces story status | Auto |
| Merge confirmed → execute | Advisor runs `gh pr merge` directly (per L15) | Auto |
| Merge complete → retro author | Advisor authors retro per template | Auto |

## What `/auto-phase` does NOT automate (six mandatory user gates stay)

The user gates exist for visibility-to-others impact and judgment calls. They stay interactive:

1. **Plan approval** — DoD smoke + watchpoint specificity + AskUserQuestion.
2. **Judgment-heavy DQ entries** — ADR-affecting / scope-changing / visible-to-others.
3. **CR triage approval** — four-bucket counts surfaced to user.
4. **Phase 2 e2e — local vs dispatch** — surfaced once per phase; answer caches for that phase only.
5. **Merge confirm** — final yes/no before `gh pr merge`.
6. **Retro sign-off** — author retro, surface, wait.

Plus catch-fires (subagent hard-refusal, attribution breach, non-allowlist §G4 fail, ci-watcher exit-code surprise, EliteDesk daemon down).

## Procedure

### Phase 0 — Prerequisites + state init

**Step 0 (MANDATORY, before any routing decision):** Read
`.claude/refs/auto-phase.md` into context. That file holds the rule's
hard refusals + state-routing invariants + resume semantics — it does
NOT auto-load at session start (relocated 2026-05-22 from `.claude/rules/`
to `.claude/refs/` to free Memory-files budget).
Without this Read, Phase 0's branch/plan checks, Phase 0.5's resume gate, and
every stage transition below run on skill-body context alone, which is
insufficient for the hard refusals + L14/L15/L16 fixes. This Read is the
first action of every `/auto-phase` invocation, fresh or resume.

Verify before any state-changing call:

1. **Branch is `governance-v0`** with clean working tree:
   ```bash
   git -C C:/Users/barri/Developer/brehon-fork rev-parse --abbrev-ref HEAD
   git -C C:/Users/barri/Developer/brehon-fork status --short
   ```
   If not on `governance-v0` or working tree dirty → STOP and surface.

2. **Plan file exists** at `.claude/PRPs/plans/<phase>.plan.md` (or sibling-named — check `ls`) for the named sub-phase. If missing → STOP and surface ("planning brief must be authored + planning Junior must run before /auto-phase").

3. **No concurrent advisor session** writing this phase (per `feedback_parallel_advisor_session_phase_branch_coordination.md` if it exists; otherwise scan `.claude/agent-activity.json` for `mode: write`).

4. **Forbidden window check** (UTC) per `.claude/rules/advisor-orchestrator.md` §"Forbidden execution windows". If in-window → ScheduleWakeup at end of window with `prompt: <same /auto-phase invocation>` and exit current tick.

5. **Initialize OR resume auto-state** at `.claude/auto-state/<phase>.json` (gitignored — runtime only). Schema in `.claude/PRPs/templates/auto-phase-state.template.json`.
   - If file does NOT exist → fresh init: write template scaffold with `stage = "init"`, `started_at = <now>`, `session_id = <random-12char-hex>`, `last_session_ended_at = null`. Proceed to Phase 1.
   - If file EXISTS → **session resume path** (Phase 0.5 below). The same `/auto-phase <phase>` invocation handles both fresh start and resume — no flag needed for the common case.

6. **Mid-phase skill-edit deferred-effect awareness.** The skill body (`~/.claude/commands/auto-phase.md`) and the JSON template are loaded **once at session start**. Edits to those files made by a concurrent meta-editor session (or by the user mid-phase) take effect ONLY after the advisor session restarts — typically at the next `/auto-phase <phase>` resume.

   The rule file (`.claude/refs/auto-phase.md`) is loaded **per-invocation** via Step 0 above (it relocated from `.claude/rules/` to `.claude/refs/` 2026-05-22 and is no longer auto-loaded at session start). A mid-phase edit to the rule therefore takes effect on the **next `/auto-phase` tick**, not at session restart. This is faster than the pre-2026-05-22 behavior — but also means a rule edit in-flight may surface mid-phase.

   What this means in practice:
   - **Lessons + lazy-loaded references** (`.claude/lessons/feedback_*.md`, anything read on-demand at retro time or DQ-triage time) update freely mid-phase.
   - **Skill body + JSON template** are sticky for the current session. If the user expects a mid-phase edit to take effect immediately, the advisor must explicitly note: "current session loaded skill at trunk SHA `<X>`; your edit at `<Y>` will apply after restart". Surface in `auto-state.trunk_sha_at_init` (already in template).
   - **Rule file** (`.claude/refs/auto-phase.md`) is re-read each tick. Mid-phase edits apply on the next tick. Plan rule edits the same way as lesson edits.
   - **Hard refusals + cadence schedule + state-routing table** that LIVE in the rule file pick up edits per-tick. That LIVE in the skill body remain sticky until session restart.

   This is not a refusal — it's a documented invariant. Per session retro 2026-05-08: the first concurrent meta-editor + advisor pattern surfaced this; the auto-state JSON's `trunk_sha_at_init` field captures the loaded version for retro-time evaluation.

### Phase 0.5 — Session resume (post-compaction / new-session pickup)

This phase exists because Claude Code sessions end (compaction, restart, laptop reboot, deliberate `/clear`) but a sub-phase typically runs ~3-4 hours — longer than any single conversation lifetime. Auto-state JSON survives session boundaries; this phase reconciles the persisted state with current world state before resuming.

**Trigger:** `.claude/auto-state/<phase>.json` exists at invocation time.

**Token budget for resume.** A resumed advisor session needs the right context with minimum search burden. The total parent-context spend on Phase 0.5 should land in **~3 KB** (state file ≤2 KB + ≤1 KB synthesis from the reconciliation subagent). Anything above that is a regression — the advisor needs reasoning headroom for the rest of the phase, and probe noise on resume directly steals from that.

The phase is structured so that:
- Step A (state read + schema upgrade) runs inline in the parent — needed for routing decisions, only ~2 KB.
- Steps B-D (Junior reconciliation + DQ delta + phase tip drift) **delegate to a single `general-purpose` Explore subagent** that runs all probes in parallel and returns one ~30-line synthesis. The parent never sees the raw `mcp__junior-brehon__list_tasks` JSON, the full `decision-queue.json` body, or `git log` outputs — only the subagent's distilled findings.
- Step E (resume report + user 'continue' gate) renders inline.

**Why the subagent on resume specifically.** The general principle from `feedback_subagent_delegation_for_multi_probe_commands.md` and `feedback_read_only_commands_still_cost_context.md` applies most strongly here — at resume, the parent has zero cache warmth (post-compaction = cold context), every probe output displaces would-be-reasoning capacity. Delegation pays for itself even at the ~50-70k subagent token cost because the parent stays lean for the whole rest of the phase.

#### Step A — Read state file + reconcile clocks + upgrade schema if needed

```python
import json, datetime, secrets, subprocess
state = json.load(open('.claude/auto-state/<phase>.json'))
now = datetime.datetime.utcnow().isoformat() + 'Z'
elapsed_since_last_action = <now> - state['last_action_at']  # human-readable: "2h 17m ago"
```

**Schema upgrade-in-place (mandatory before any state-machine routing).** The
auto-state JSON schema may evolve between phases. A session that started on
the old schema may resume after a newer schema shipped. Phase 0.5 MUST
detect missing fields and add them with safe defaults BEFORE any
reconciliation step that depends on them. This prevents `KeyError` /
`AttributeError` on resume and preserves backward compatibility.

```python
# Add safe defaults for any field the current template requires but
# the state file lacks. Order matters: deeper fields last (so we don't
# overwrite a partial nested structure).

upgraded = False

if 'session_id' not in state:
    state['session_id'] = secrets.token_hex(6)  # 12-char hex
    upgraded = True

if 'last_session_ended_at' not in state:
    state['last_session_ended_at'] = None
    upgraded = True

if 'resume_count' not in state:
    state['resume_count'] = 0
    upgraded = True

if 'last_known_phase_tip' not in state:
    # Derive from current git state — this is a best-effort backfill;
    # if the phase branch doesn't exist yet (e.g. pre-bm-cut), set None.
    try:
        tip = subprocess.check_output(
            ['git', '-C', 'C:/Users/barri/Developer/brehon-fork',
             'rev-parse', f'phase-{state["phase"]}'],
            stderr=subprocess.DEVNULL
        ).decode().strip()
        state['last_known_phase_tip'] = tip
    except subprocess.CalledProcessError:
        state['last_known_phase_tip'] = None
    upgraded = True

if 'schema_version' not in state or state.get('schema_version', 0) < 1:
    state['schema_version'] = 1
    upgraded = True

# error_class_history backfill — per advisor-orchestrator.md §5.3 cycle-count meta-rule
for member in state.get('current_cohort', {}).get('members', []) or []:
    if 'error_class_history' not in member:
        member['error_class_history'] = []
        upgraded = True

# schema-v2 additive backfill — stage-digest ring (forward-only; does NOT
# reinterpret v1 fields). Per auto-phase-state.template.json _schema_v2_note.
if 'stage_digests' not in state:
    state['stage_digests'] = []
    upgraded = True
if 'digest_overflow_path' not in state or not state.get('digest_overflow_path'):
    state['digest_overflow_path'] = f".claude/auto-state/{state['phase']}.digests.jsonl"
    upgraded = True

# schema-v3 additive backfill — uniform tool-output spill guard + auto-handover
# pointers (forward-only; does NOT reinterpret v1/v2 fields). Per
# auto-phase-state.template.json _schema_v3_note.
if 'spill_dir' not in state or not state.get('spill_dir'):
    state['spill_dir'] = f".claude/auto-state/{state['phase']}.spill/"
    upgraded = True
if 'last_handover_path' not in state:
    state['last_handover_path'] = None
    upgraded = True
if 'last_handover_at' not in state:
    state['last_handover_at'] = None
    upgraded = True

# --unattended additive backfill (forward-only). A state file written before
# --unattended shipped lacks these fields. Default false/null = fully-gated,
# preserving prior behaviour exactly. CRITICAL: the flag is an INVOCATION-time
# choice, re-derived from the CURRENT $ARGUMENTS — NOT a sticky state-file
# property. A resume WITHOUT --unattended reverts to fully-gated (the safe
# direction); a resume WITH it re-enables. `arguments` below is the raw
# $ARGUMENTS string this /auto-phase invocation was called with.
arguments = "<raw $ARGUMENTS string for this invocation>"
prior_unattended = state.get('unattended')          # may be missing on old files
invocation_has_unattended = "--unattended" in arguments
invocation_e2e_dispatch = "--e2e dispatch" in arguments
state['unattended'] = invocation_has_unattended
state['unattended_e2e_override'] = (
    "dispatch" if (invocation_has_unattended and invocation_e2e_dispatch) else None
)
if prior_unattended != state['unattended']:
    upgraded = True

# Bump schema_version to 3 once the v2+v3 fields are present. Forward-only:
# never downgrade. Must run AFTER the field backfills above.
if state.get('schema_version', 0) < 3:
    state['schema_version'] = 3
    upgraded = True

if upgraded:
    # Persist the upgrade BEFORE any other state-changing action.
    # Surface the upgrade in the resume report so the user sees what changed.
    json.dump(state, open('.claude/auto-state/<phase>.json', 'w'),
              indent=2, ensure_ascii=False)
    upgrade_summary = "auto-state JSON schema upgraded in place: " + \
                      "added defaults for fields {<list>}"
    # Include in Step E resume report.
```

The upgrade is **idempotent** — re-running it on an already-current state
file is a no-op. The upgrade is **forward-only** — never downgrades
(removing fields a newer schema added). If the schema lacks a field that
state machine logic depends on (e.g. `current_cohort` before
`impl-cohort-N` exists), the absence is normal — backfill with `None`,
not with a fabricated structure.

Check the `stage` field. If `stage == "done"` → refuse: "phase already complete; delete state file or run new phase". If `stage == "catch-fire"` → refuse auto-resume: surface the catch-fire dump path; require user to explicitly invoke `--start-from <stage>` after addressing the underlying cause.

Otherwise, route by stage class (read-only world reconciliation, no state-changing calls yet):

| Stage class | Reconciliation actions |
|---|---|
| `*-running` (Junior in flight) | `mcp__junior-brehon__list_tasks` filtered to the phase; check if `state.junior_tasks.<stage>` id is still `running` / has transitioned / no longer exists. Possible outcomes below. |
| `*-pending-user` (waiting on AskUserQuestion) | The previous session ended without the user answering. Re-fire AskUserQuestion now. State file is authoritative for the question content. |
| `*-validating` (ci-watcher cycle) | Read `.claude/decision-queue.json` for the `validate-pending` entries listed in `state.current_cohort.members[].validate_pending_dq_id`. If now mutated to `result: pass` → advance state. If still pending → check if a paired ci-watcher Junior task is still running; if not, dispatch a fresh ci-watcher (the prior task may have died with the session that queued it). |
| `merge-executing` | Did the merge complete? `gh pr view <PR#> --json state` — if `MERGED`, advance to `retro-author`; if `OPEN`, the merge must have failed mid-execution — surface to user, do NOT auto-retry. |
| `retro-author` | Check if `.claude/PRPs/retros/<phase>-retro.md` (or `reports/`) exists and is non-empty. If yes → advance to `retro-pending-user`. If no → re-author. |

#### Step B/C/D — Single delegated reconciliation pass (token-efficient)

Replace the prior three-step inline reconciliation with **one `general-purpose` subagent dispatch** that runs all probes in parallel and returns a single synthesis. The advisor session reads the synthesis, not the raw probe outputs.

**Subagent prompt (must be self-contained — subagent has zero parent context):**

```
Brehon /auto-phase resume reconciliation. Self-contained probes + synthesis.

Inputs (all reads, no mutations):
  state_file = "C:/Users/barri/Developer/brehon-fork/.claude/auto-state/<phase>.json"
  brehon_repo = "C:/Users/barri/Developer/brehon-fork"
  phase_branch = "phase-<phase>"   # e.g. "phase-v1-SL-c-2"

Run these in parallel and produce the synthesis below:

1. Read state_file. Extract: stage, current_cohort.members[].junior_id,
   junior_tasks (full map), last_dq_pending_count, last_dq_pending_ids,
   last_known_phase_tip, user_gate_history (last 3 entries' notes only).

2. Junior task reconciliation (primary: sqlite3-over-SSH; fallback: MCP shim):
   - **Primary path** — direct daemon DB read, ~1KB output:
     `ssh -o ConnectTimeout=5 homeserver "sqlite3 /srv/brehon-fork/.junior/junior.db \"SELECT id, status, branch, session_id, datetime(updated_at, 'unixepoch') AS upd FROM jobs WHERE id IN (<comma-separated junior_id list>);\""`
     Build the id list from `state.junior_tasks` (values) + `state.current_cohort.members[].junior_id` + `current_cohort.members[].fix_attempts[].junior_id`.
   - **Fallback** — if SSH times out OR sqlite returns empty OR Tailscale unreachable: call `mcp__junior-brehon__list_tasks` (status only, no show_task) and note `daemon-DB read fell back to MCP shim` in the synthesis. Per session-retro-2026-05-09 proposal #4: shim returned 97k chars on a single list_tasks call, exceeding direct ingestion limits.
   - For every junior_id, produce one line: `task #<N>: recorded=<old> current=<new>`.
   - For tasks where recorded != current, classify the transition per
     this table:
       running→complete   = task finished while session was down
       running→failed     = surface to user, do NOT auto-retry
       running→cancelled  = surface, require --start-from
       running→(missing)  = treat as failed (DB race / daemon clean)
       complete→complete  = next-stage transition may have been missed

3. DQ delta scan (CANONICAL — spans phase + worker branches):
   - `git -C ${brehon_repo} fetch origin --quiet`
   - **Run the canonical resolver:**
     `${brehon_repo}/scripts/brehon/resolve-dq-canonical.sh ${phase_argument}`
     The resolver reads auto-state to find active worker branches
     (current_cohort.members[].junior_id + fix_attempts), unions
     phase-branch DQ with each open worker-branch DQ, dedupes by
     id (worker-branch wins on collision = most recent state).
     Output: absolute path to a merged JSON file.
   - Parse the merged JSON. Count pending.
   - Compare to state.last_dq_pending_count.
   - For NEW pending entries (id not in state.last_dq_pending_ids),
     output one line per: `DQ #<id> source=<label> from=<role> kind=<k> q=<first 80 chars>`.
     The `source=` label (e.g. `phase-branch`, `worker-159`) is from
     the resolver's `_canonical_source` field — surfaces which ref
     contributed the entry, distinguishing daemon-finalize-merged
     entries from worker-branch-only ones.
   - For RESOLVED entries (id was pending, no longer in pending), output
     one line per: `DQ #<id> resolved (was pending)`.
   - **Why canonical:** reading only `${brehon_repo}/.claude/decision-queue.json`
     (= phase-branch tip on the laptop checkout) misses entries on
     active worker branches that the EliteDesk daemon has pulled but
     not yet finalize-merged. The 2026-05-09 c-2 resume incident:
     laptop saw pending=0 while live advisor session saw pending=2
     (DQ #164+#165 on worker-159). Same bug pattern would re-fire on
     every cohort with an in-flight ci-watcher. The resolver makes
     the canonical view explicit and reproducible.

4. Phase tip drift:
   - `git -C ${brehon_repo} rev-parse ${phase_branch}` (catch error if
     branch missing — pre-bm-cut state).
   - Compare to state.last_known_phase_tip.
   - If advanced: `git -C ${brehon_repo} log --oneline
     ${state.last_known_phase_tip}..${phase_branch} | head -10`
   - Output: `phase tip: <recorded sha> → <current sha> (<N> commits
     ahead)` plus the oneline log if N>0.

5. Daemon health (single SSH probe):
   - `ssh -o ConnectTimeout=5 homeserver
     'systemctl is-active junior@brehon-fork'`
   - One line: `daemon: <active|inactive>`.

6. Forbidden window:
   - `date -u +"%a %H:%M UTC"` and check against the table in
     advisor-orchestrator.md "Forbidden execution windows".
   - One line: `forbidden window: <OK|IN WINDOW until HH:MM UTC>`.

Synthesis shape (return EXACTLY this format, ≤30 lines, ≤1 KB):

```
=== Phase 0.5 reconciliation — <phase> ===

State summary:
  Stage at last-session end: <S>
  Last action: <ts>, <human-elapsed> ago
  Recent gate notes (last 3): <bullet per non-empty notes field, 1 line each>

Junior tasks (recorded → current):
  task #<N> (<role>): <recorded> → <current>  <classification if changed>
  ...

DQ:
  pending: <recorded N> → <current M>  (delta: +<x> -<y>)
  new: <list of new entry one-liners> OR "none"
  resolved: <list of resolved id one-liners> OR "none"

Phase tip:
  <recorded sha> → <current sha>  (<N> commits ahead)
  [if N>0] commit oneline log

Daemon: <active|inactive>
Forbidden window: <OK|IN WINDOW until HH:MM UTC>

Computed next action: <one sentence — see Phase 0.5 routing in
.claude/refs/auto-phase.md "Resume semantics" + the stage-class
table; pick the action that matches state.stage class>
```

Refusals:
- If state_file is unreadable or malformed JSON → return synthesis
  with first line `RECONCILIATION FAILED: <reason>` and stop.
- Do NOT run `mcp__junior-brehon__create_task` (subagents cannot
  dispatch Junior tasks per the auto-phase.md rule's hard refusal).
- Do NOT mutate the state file (parent owns it).
- Do NOT run cargo, edit any file, or call gh outside the read-only
  probes listed.
```

**Cost.** One `Agent({ subagent_type: "general-purpose", ... })` call. Subagent burns ~50-70k tokens; parent receives ~30 lines (~1 KB). Inline mode would land ~12 KB of probe output in the parent for the equivalent information. The trade is net-positive at resume because parent context post-compaction is cold and any displaced reasoning capacity is harder to recover than the subagent's tokens.

**The reconciliation pass only READS** — it does not modify Junior state, mutate the auto-state JSON, or queue tasks. State-machine advances happen via the normal Phase 1 tick after Phase 0.5 completes.

#### Step E — Print COMPACT resume report + ask user to confirm before any state-changing action

Build the COMPACT resume report (<=15 lines, <=500 tokens) primarily from the ledger, not from
conversation: read `stage_digests[-3:]` for the recent narrative and `last_handover_path` for the
self-contained handover. Surface the latest `next_action_hypothesis` as a HYPOTHESIS to re-verify
against live TaskList/DQ/PR state before any action (per `feedback_thin_wakeup_prompts_verify_live_state.md`).
Keep lazy-load + Steps-B-D subagent-delegation discipline unchanged; the digests are additional
durable signal, not a new preload.

After Steps A-D complete, print a **≤15-line compact** report. Verbose detail goes to a gitignored debug file the user can drill into on demand.

**Compact format (target: ≤15 lines, ≤500 tokens):**

```
=== /auto-phase <phase> — RESUMING (resume #<N>) ===
Stage: <S>  ·  Last action: <elapsed> ago  ·  Daemon: <active|inactive>  ·  Window: <OK|wait HH:MM>
Schema: <up-to-date | UPGRADED [+<field-list>]>
World: Junior tasks <transitions-summary>  ·  DQ pending <recorded>→<current> (sources: <phase-branch[,worker-N...]>)  ·  Phase tip <unchanged|+<N> commits>
[if any anomaly] ⚠ <one line per — e.g. "ci-watcher #159 running 24m past 12m typical">
Next action: <one sentence>

Reply 'continue' to resume.  Optional: 'status' for full state dump · 'debug' to dump verbose probe output to .claude/auto-state/<phase>-resume-debug-<ts>.md · 'reset' to discard state.
```

**Rules:**
- Pack the world-reconciliation onto ONE line. The user only needs to know "is anything weird"; if they want detail, 'status' or 'debug' provides it.
- Anomaly lines (⚠) are reserved for things requiring user awareness even if they don't block resume — long-running ci-watcher, unexpected phase-tip drift, daemon flapping. Suppress when nothing's anomalous.
- The DQ source labels (`phase-branch`, `worker-N`) come from the canonical resolver's `_canonical_sources_consulted` — surfaces which refs contributed without verbose listing.
- Schema status only shows `UPGRADED [+<field-list>]` when fields were backfilled this resume; otherwise just `up-to-date`.

**Verbose detail on demand.** If user replies 'debug', write the full probe outputs (subagent synthesis verbatim + per-task `mcp__junior-brehon__show_task` for transitioned tasks + canonical DQ JSON tail) to `.claude/auto-state/<phase>-resume-debug-<UTC-iso>.md` (gitignored under `.claude/auto-state/`) and reply with the path. Cost: subagent already produced the synthesis, so the cost is one additional file write — negligible. The full subagent synthesis is NOT loaded into the parent context unless the user explicitly asks.

**Token-efficiency target:** ≤500 tokens for the compact report. Compared to the prior ~80-line verbose version (~3-5 KB), this saves ~3-5k parent tokens per resume — applied across `resume_count: 2` already on c-2, that's ~10k tokens saved per resume cycle.

**Wait for user reply** before any state-changing action on resume. This is a deliberate friction step — the cost of an unwanted resume action (e.g. re-queueing a Junior task that's actually still alive on the daemon) outweighs the cost of one extra user touch on resume. After the first user 'continue', subsequent ticks proceed autonomously per Phase 1.

**Concurrent-advisor-session probe (fallback when `agent-activity.json` absent).** Before printing the compact report, run:

```bash
git -C C:/Users/barri/Developer/brehon-fork log --since='10 minutes ago' \
  --grep='^chore(advisor)\|^docs(advisor)\|^chore(decision-queue)' \
  --author='solo-dev' --oneline
```

(Use `'10 minutes ago'` not `'10 min ago'` — the abbreviated form fails silently on git-for-Windows ≤2.45.)

If the probe returns ≥1 commits AND the most recent commit's timestamp is newer than `state.last_action_at`, prepend an extra `AskUserQuestion` BEFORE the 'continue' wait, with options: "Continue (treat their work as authoritative; advance auto-state to align)" / "Stop (let other session finish)" / "Reset state file (`--reset` semantics)". Surfacing layer only — the strict refusal path stays `auto-phase.md` Hard refusal #5 (via `agent-activity.json`). Per session-retro-2026-05-09 proposal #3 + `feedback_disclose_shared_state_edits.md`: a different advisor session authored bit-identical artifacts in parallel and won the commit race. The probe makes the parallelism visible.

Update the state file:
- `session_id` → new random hex
- `last_session_ended_at` (was: prior session's end) → keep as historical record
- `resume_count` → increment

Then proceed to Phase 1.

### Phase 0.6 — Subagent offload (resume-only by default; broader use deferred to retro signal)

**Active offload (built-in):** Phase 0.5 Steps B-D delegate reconciliation to a `general-purpose` subagent on every resume. Justified by user 2026-05-08: *"it should be highly efficient use of tokens on resuming, so the advisory is given the right context without too much searching"*. Resume is the cold-context moment where parent reasoning headroom is most precious; subagent delegation pays for itself there.

**Deferred offload (not built):** broader offload during normal-tick polling, plan re-read mid-phase, retro draft authoring, §G4 classifier, etc — these MAY benefit from delegation but the existing four-role model + Junior subagents already handle the heavy lifting cleanly. **Building offload machinery for these pre-c-2 is speculative scope.** The retro discipline is the right place to surface them: if c-2's `/context` measurement shows the advisor parent burning >200k tokens by mid-phase outside resume, the c-2 retro should propose specific offload candidates with evidence; promote to skill body in c-3+.

**Hard rule for any subagent dispatch under `/auto-phase`:** subagents MUST NOT dispatch Junior tasks, write `answered_by: "advisor"` DQ entries, fire `gh pr merge`, or mutate `.claude/auto-state/<phase>.json`. Those stay in the parent advisor session per attribution + L15 invariants. The Phase 0.5 reconciliation subagent honors this — its prompt explicitly forbids the dispatch + mutate paths.

### Phase 0.7 — Lazy-load discipline on resume (token-frugal)

The advisor's parent context after a fresh-session resume is cold (post-compaction or new conversation). Every file the parent reads at resume-time displaces reasoning capacity for the rest of the phase. The skill body MUST defer reads that are not required for the immediate next-action decision.

**Read on resume (mandatory — required to route the next tick):**

- `.claude/auto-state/<phase>.json` (state file — ~2 KB)
- `.claude/decision-queue.json` from the canonical resolver (~5-30 KB, but offloaded to subagent and synthesised to ~1 KB)
- The skill body itself (~12 KB — Claude Code auto-loads it on `/auto-phase` invocation)
- `CLAUDE.md` + auto-loaded rules (~30 KB — Claude Code auto-loads on session start; skill cannot avoid this)

**DEFER (do NOT read on resume — read just-in-time when actually needed):**

- The "Mandatory file-class lesson injection" table in `.claude/rules/advisor-orchestrator.md` — only needed when authoring the next impl-task brief (which is itself preceded by ≥1 cohort barrier or fresh stage transition). Read it inside the `impl-cohort-N` action handler, not at resume.
- The §G4 classifier allowlist — only needed when a `validate-pending` entry mutates to `result: fail`. Read it inside the failure-handling branch.
- The retro template — only needed when entering `retro-author`. Read it then.
- The `/brehon-verify` story-checking logic — only needed at `verify-running`. Read then.

**Anti-pattern (observed 2026-05-09 on c-2 resume):** the live advisor session preloaded the file-class table into parent context "in case Tasks 2-5 dispatch later" — those tasks were ≥30 minutes away (gated on ci-watcher pass + phase tip movement + cohort 2 closing). Net cost: ~5-8k tokens of parent context burned on never-used data. Resume tokens climbed to 15% (151k) — outsized for a cheap state-machine tick.

**Anchor:** when a stage handler in Phase 2's routing table calls for a specific rule lookup (e.g. "walk the §13 task's IMPLEMENT files against the file-class table"), the handler does the read AT THAT POINT, not earlier. Treat resume as a state-machine routing decision, not as preloading-the-tools work.

### Phase 1 — Per-tick state machine

Each tick reads (parallel where possible):

- `mcp__junior-brehon__list_tasks` — Junior states (status only; no `show_task` unless transition).
- `git -C C:/Users/barri/Developer/brehon-fork fetch origin` — quiet, lightweight; skip if last fetch <60s ago.
- `.claude/decision-queue.json` — pending count + new entries (always read; cheap).
- `.claude/auto-state/<phase>.json` — current stage + last action.

State machine routes per the table below. If no transition fires (no Junior status change + no new DQ + no new commits remote-side), no-op tick: log "no change" and ScheduleWakeup at the cadence for current stage.

#### Phase 1 standing rules (apply on every tick, before routing)

These three rules implement the schema-v2/v3 context-management contract
(`.claude/refs/auto-phase.md` "Durable context" invariants + the state
template's `_stage_digests_note` / `_spill_dir_note` / `_last_handover_note`).
They are **data-only**: none of them gates, routes, or changes cadence.

**1. Stage-digest ring append (on every stage transition).** When a routing
row below fires and updates the `stage` field, AFTER persisting the new
`stage`, append one digest object to `stage_digests`:

```python
DIGEST_RING_MAX = 12
digest = {
    "stage": state["stage"],                  # the stage just entered
    "entered_at": now,                         # ISO8601 UTC
    "outcome": "<one line — what the prior stage produced>",
    "key_ids": {"cohort": ..., "junior": ..., "dq": ..., "pr": ...},  # ids touched
    "next_action_hypothesis": "<one sentence — re-verify on resume>",
}
state["stage_digests"].append(digest)
# Trim to the ring cap; spill the oldest beyond the cap to the JSONL overflow.
while len(state["stage_digests"]) > DIGEST_RING_MAX:
    oldest = state["stage_digests"].pop(0)
    with open(state["digest_overflow_path"], "a", encoding="utf-8") as f:
        f.write(json.dumps(oldest, ensure_ascii=False) + "\n")
# Persist the ledger (the digest is durable advisor context, NOT a brief input).
```

The ring MUST NOT leak into Junior brief bodies (advisor context is wide;
delegation packets stay narrow — `advisor-orchestrator.md` §2.2). Read by
Phase 0.5 Step E to rebuild the COMPACT resume report.

**2. Tool-output spill guard (applies to all tool results during the tick).**
When any tool result exceeds 16000 chars, write the full result to
`auto_state.spill_dir/<stage>-<tool>-<UTC-iso>.txt` (default
`.claude/auto-state/<phase>.spill/`), and retain only the first ~40 and last
~40 lines plus the spill path in context. For broad PMD semantic queries,
prefer running inside a subagent (per `pmd-search-strategy.md`) so the dump
never enters main context; spill is the fallback when a large result was not
anticipated.

**3. Auto-handover refresh (immediately after the digest append in rule 1).**
After the digest append, write/overwrite
`.claude/PRPs/handovers/<phase>-auto-<UTC-date>.md` with: current sub-phase,
stage, last commit on the relevant branch (SHA + subject), next concrete
action (the digest's `next_action_hypothesis`, marked "re-verify on resume"),
DQ pending ids, and a one-line concurrent-activity note. Set
`auto_state.last_handover_path` + `last_handover_at`. Commit on
`governance-v0` with subject
`chore(advisor): auto-phase handover refresh <phase> <stage>`. This is the
floor that satisfies the `advisor-orchestrator.md` §1 pre-compact handover
discipline automatically.

### Phase 2 — Stage routing table

The state machine is a literal compilation of `.claude/rules/advisor-orchestrator.md` §"Stage-shape orchestration". Each row: state → trigger → action → next state.

| Current state | Trigger | Action | Next state |
|---|---|---|---|
| `init` | Phase 0 passed | Author bm-cut brief, push, queue Junior `[role:bm-task]` | `bm-cut-running` |
| `bm-cut-running` | Junior done | Verify phase branch exists | `bm-cut-done` |
| `bm-cut-done` | (immediate) | Consult PMD + author planning brief, push, queue Junior `[role:planning]` | `planning-running` |
| `planning-running` | Junior done | Read plan output, run DoD smoke + watchpoint gate | `planning-approved-pending-user` (PASS) / `planning-revise` (FAIL) |
| `planning-approved-pending-user` | (immediate) | **Gate 1 (always human, even under `--unattended`).** If `--unattended`: park — fire the gate-waiting Telegram ping (per Phase 7), record `parked-pending-user` in `user_gate_history`, ScheduleWakeup at idle cadence, and re-fire **AskUserQuestion** on the tick the user returns. Else: **AskUserQuestion** now. | `impl-cohort-1` (approve) / `init` (reject + revise) |
| `impl-cohort-N` | (immediate) | Compute cohort from §13 `[P]` markers + YAML overlap check + budget check; **for each task in cohort, walk the §13 task's IMPLEMENT files against the "Mandatory file-class lesson injection" table in `.claude/rules/advisor-orchestrator.md` and inject every matching lesson into the brief's §3 Required reading**; run the standard `memory_search_hybrid` PMD lookup; author N briefs; queue N Junior tasks in parallel. | `impl-cohort-N-running` |
| `impl-cohort-N-running` | Each member done | Read DQ for `validate-pending` entries; queue ci-watcher per entry | `impl-cohort-N-validating` |
| `impl-cohort-N-validating` | All ci-watchers `result: pass` | Daemon finalize-merges; advisor detects new phase tip | `phase-2-e2e-N` |
| `phase-2-e2e-N` | (immediate, first run) | **Gate 4 (allowlisted).** If `--unattended`: auto-set `phase_2_e2e_mode = "local"` (or `"dispatch"` iff `--e2e dispatch` was passed), record `decision: "auto-approve-unattended"` + `notes: "e2e default local"` in `user_gate_history`, NO surface. Else: **AskUserQuestion** — local vs dispatch. | `phase-2-e2e-N-running` (cache choice) |
| `phase-2-e2e-N` | (immediate, subsequent) | Use cached choice; raise Phase 2 DQ; dispatch ci-watcher OR spawn cargo `run_in_background` | `phase-2-e2e-N-running` |
| `phase-2-e2e-N-running` | ci-watcher / bg cargo done with `pass` | Advance to next cohort or `bm-pr-pending` if last cohort | `impl-cohort-N+1` / `bm-pr-pending` |
| `impl-cohort-N-validating` | Any ci-watcher `result: fail` | Parse log slice for `error[<class>]` + first-cited file basename; append `{workflow_run_id, error_class, file, ts}` to `current_cohort.members[<task>].error_class_history`. Then **cycle-count meta-rule** per `advisor-orchestrator.md` §5.3 — if count of same `(error_class, file)` in history ≥3 → catch-fire (no fix-impl). Else §G4 classifier on log slice. | `catch-fire` (cycle-count ≥3) / `impl-fix-N-running` (allowlist) / `catch-fire` (non-allowlist) |
| `impl-fix-N-running` | Junior done | Re-trigger validation (Phase 1) | `impl-cohort-N-validating` |
| `bm-pr-pending` | (immediate) | Author bm-pr brief, push, queue Junior `[role:bm-task]` | `bm-pr-running` |
| `bm-pr-running` | Junior done | Verify PR exists | `cr-wait` |
| `cr-wait` | Poll cadence: CR comment posted | Verify findings present | `bm-poll-cr-running` |
| `bm-poll-cr-running` | Junior done | Verify findings YAML written | `bm-triage-running` |
| `bm-triage-running` | Junior done | Read four-bucket counts | `cr-triage-pending-user` |
| `cr-triage-pending-user` | (immediate) | **Gate 3 (always human, even under `--unattended`).** If `--unattended`: park + gate-waiting ping (per Phase 7), re-fire on user return. Else now. Then **AskUserQuestion** — surface four-bucket triage. | `fix-in-pr-cohort-1` (approve fix work) / `verify-running` (no findings) |
| `fix-in-pr-cohort-N` | Same as impl-cohort-N (mirrored) | (mirrored) | (mirrored) → `verify-running` when done |
| `verify-running` | (immediate) | Run `/brehon-verify <phase>` inline | `merge-pending-user` (all stories ✓) / `catch-fire` (phantom) |
| `merge-pending-user` | (immediate) | **Gate 5 (always human, even under `--unattended`).** `gh pr merge` NEVER fires unattended — irreversible + outward-facing. If `--unattended`: park + gate-waiting ping (per Phase 7), re-fire on user return. Else now. Then **AskUserQuestion** — merge confirm. | `merge-executing` (confirm) / `cr-triage-pending-user` (reject + more fixes) |
| `merge-executing` | (immediate) | **Advisor runs `gh pr merge` inline (L15)**; verify branch deletion (L16); author runlog `docs(advisor)` if BM brief skipped (L14) | `retro-author` |
| `retro-author` | (immediate) | Advisor authors retro per template | `retro-pending-user` |
| `retro-pending-user` | (immediate) | **Gate 6 (allowlisted).** If `--unattended`: run the 3-check sanity gate (retro file exists; non-empty; required §-sections present per Phase 7). All three pass → auto-clear, record `decision: "auto-approve-unattended"` + `notes: "retro sanity 3/3"` in `user_gate_history`, advance. Any check fails → **catch-fire** (do NOT auto-clear a malformed retro; surface which check failed). Else: **AskUserQuestion** — retro sign-off. | `phase-transition` (auto-clear or approve) / `catch-fire` (sanity fail) |
| `phase-transition` | (immediate) | Run `/brehon-phase-transition` skill | `done` |
| `catch-fire` | — | Dump current state + last 3 polls to `.claude/auto-state/<phase>-catchfire-<ts>.md`; surface to user | (manual resume) |

### Phase 3 — Cadence schedule (cache-aware ScheduleWakeup)

Sleep windows respect the prompt-cache 5-min TTL. Anything 270–300s stays cache-warm; anything ≥1200s pays cache-miss once and amortises across a long wait. **No 300s sleeps** (worst-of-both per ScheduleWakeup tool guidance).

| Stage | Expected duration | Initial poll | Subsequent |
|---|---|---|---|
| `bm-cut-running` | ~2 min | 120s | 60s |
| `planning-running` | 30–90 min | 1200s | 600s |
| `impl-task` running (single) | 8–25 min | 600s | 300s → use 270s instead |
| `impl-task` cohort (≥2 [P]) | 8–25 min parallel | 600s | 270s until barrier |
| `ci-watcher` Phase 1 workspace | 8–12 min | 270s | 270s |
| `ci-watcher` Phase 2 e2e (dispatched) | ~26 min | 1500s one-shot | 270s if not done |
| Phase 2 e2e (local cargo bg) | ~26 min | 1500s one-shot | 270s if process still running |
| `bm-pr-running` | ~3 min | 120s | 60s |
| `cr-wait` | 5–15 min | 600s | 300s → use 270s instead |
| `bm-poll-cr-running` | ~2 min | 120s | 60s |
| `bm-merge-executing` | ~1 min | 60s | (one-shot) |
| `retro-author` (advisor model) | ~25 min | (no Junior poll) | — |
| Idle / no transition | — | 1200s default | — |

Always pass `prompt: <verbatim /auto-phase invocation>` to `ScheduleWakeup` so the next firing re-enters the skill at the same phase argument.

### Phase 4 — Critical fixes from c-1 lessons (encoded inline)

The skill's bm-merge handling encodes c-1 session lessons:

- **L15 (gate-then-execute split duplicates context):** the merge-gate's read-only checks (`gh pr view --json mergeStateStatus,mergeable,statusCheckRollup`, findings YAML scan, DQ scan, CR re-poll-since) run **inline in the advisor session**, NOT as a Junior task. Junior is queued only post-confirm for the actual `gh pr merge` execution AND the `chore(bm)` runlog commit. One brief, one Junior task, one push grant for the brief.
- **L14 (BM Junior improvises git workflow when briefs are silent):** the bm-merge brief explicitly orders the BM Junior's git sequence: `Edit runlog → git add → git commit -m chore(bm):... → git push origin governance-v0 → THEN gh pr merge`. The runlog commit lands BEFORE the merge. If the BM somehow still skips it (recurrence detection: post-merge tick scans `git log -3 governance-v0` for `chore(bm)` matching the merge timeframe), the advisor's post-merge tick authors a `docs(advisor): re-apply` block — same precedent as DQ #148.
- **L16 (`gh pr merge --delete-branch` silently fails under head-branch protection):** after `gh pr merge` returns, the advisor verifies branch deletion via `git ls-remote origin refs/heads/<phase-branch>`. If still present (silent-skip case), advisor runs `gh api -X DELETE -H "Accept: application/vnd.github+json" /repos/barrie-cork/lemmy/git/refs/heads/<phase-branch>` advisor-side. No retry, no surface — 1-line fix per known failure mode.

### Phase 5 — Failure modes

| Failure | Handling |
|---|---|
| Junior task `failed` mid-cohort | Stop cohort advance, surface task output, await direction (do NOT auto-retry — could be a real bug) |
| ci-watcher mutates DQ to `result: timed_out` | Surface to user (workflow exceeded 60-min cap; not auto-retryable) |
| `--unattended`: gate 6 retro sanity check fails (missing / empty / wrong-shape retro) | **Catch-fire** — do NOT auto-clear; dump which of the 3 checks failed; require manual retro fix + `--start-from retro-author`. A malformed retro is a phantom-completion, not an auto-approve. |
| `--unattended`: Telegram MCP disconnected at a parked gate | Silent skip the ping + note in the auto-handover; the gate still parks and waits (pings are notifications, not gating signals per `feedback_telegram_scope_notification_only.md`). The user finds the parked gate on their next session regardless. |
| `--unattended`: catch-fire fires (any cause) | Park + ping exactly like a gate, then wait for manual resume. `--unattended` NEVER suppresses or auto-recovers a catch-fire. |
| `--unattended`: a `user_gate_history` entry is found with `decision: "user"` whose surrounding evidence shows it was a policy auto-clear | Attribution breach (same class as a forged `answered_by: "advisor"`). Catch-fire; surface the entry; require `docs(advisor): correct gate attribution` follow-up. |
| Cycle-count meta-rule fires (≥3 same `(error_class, file)` in cohort member history) | Catch-fire per `advisor-orchestrator.md` §5.3; surface history verbatim; do NOT auto-retry; require revert+re-plan or `--start-from <prior-stage>` after addressing recipe-family root cause |
| EliteDesk SSH timeout during poll | Single retry with 30s backoff; on second timeout, surface and ScheduleWakeup at 1200s |
| Forbidden window encountered before queue | Defer per advisor-orchestrator.md §"Forbidden execution windows"; ScheduleWakeup at end of window |
| Hook policy blocks `governance-v0` push | Surface typed-grant request to user |
| `gh pr merge` exits non-zero | Surface error verbatim; do NOT retry; do NOT improvise alternative merge strategy (per bm-merge.md hard refusal) |
| BM Junior breach (writes to `crates/`, etc.) | Catch-fire to user with rule citation; halt skill |
| Plan §13 has no clear task numbering | Surface as plan-shape error; require planner re-issue before bm-cut |
| Phase tip diverges from EliteDesk-side worktree | Surface; offer hard-reset option (typed grant required) |
| `--start-from <stage>` invalid | Refuse with valid stage list |
| Plan file missing for named phase | Surface ("plan must be authored before /auto-phase") |
| Multiple plans match phase glob | Refuse with file list; require user disambiguation |

### Phase 6 — `--dry-run` output

When `--dry-run` flag set, run all read-only Phase 0 prerequisites + read the plan + compute the dispatch plan, then print:

```
=== /auto-phase v1-SL-c-2 — DRY RUN ===

Plan: .claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md
§13 tasks: <count>
Cohorts: <list of cohort groupings — Task 1 alone, Task 2-3 [P], Task 4 alone, ...>

Forbidden-window check: <UTC time> — <OK / IN WINDOW until HH:MM>
Plan DoD smoke (would run): <list of §15 commands>

Dispatch sequence:
  1. bm-cut (Junior, ~2 min)
  2. planning (Junior, ~30-90 min) → user gate: plan approval
  3. impl-cohort-1: Task 1 (Junior, ~10 min)
     - Phase 1 ci-watcher (Junior, ~10 min)
     - Phase 2 e2e (first run) → user gate: local vs dispatch
  4. impl-cohort-2: Task 2-3 [P] (2 parallel Juniors)
  ...
  N. bm-pr (Junior) → cr-wait (~10 min) → bm-poll-cr → bm-triage → user gate: CR triage
  N+1. (if findings) fix-in-pr cohorts
  N+2. /brehon-verify (advisor inline)
  N+3. user gate: merge confirm → advisor gh pr merge
  N+4. retro-author (advisor) → user gate: retro sign-off
  N+5. /brehon-phase-transition

Estimated wall-clock: ~3-4 hours (vs c-1 4h 21min baseline)
Estimated user touchpoints: 6 (plan, e2e first, CR triage, merge, retro, brehon-phase-transition)
Estimated push grants: 3-4 (brief commits)

Re-run without --dry-run to execute.
```

**When `--unattended` is ALSO set**, append a gate-handling block to the dry-run output:

```
=== --unattended gate handling ===
  Gate 1 plan-approval   → PARK + ping (human)
  Gate 2 ADR/scope DQ    → PARK + ping (human)
  Gate 3 CR triage       → PARK + ping (human)
  Gate 4 e2e mode        → AUTO: local   [or "dispatch" if --e2e dispatch]
  Gate 5 merge confirm   → PARK + ping (human)
  Gate 6 retro sign-off  → AUTO-CLEAR (after 3-check sanity gate)

Estimated unattended user touchpoints: 4 (plan, CR triage, merge — plus 1 if a
  judgment-heavy DQ surfaces mid-phase) + phase-transition. Gates 4 & 6 cleared
  by policy. You'll receive a Telegram ping at each parked gate.
```

No state-changing calls in dry-run mode.

### Phase 7 — `--unattended` gate allowlist (HARD CONTRACT)

`--unattended` is an opt-in partial-autonomy layer. It changes **only** how the
six mandatory user gates are handled; every other transition is identical to the
default mode. There is **no cheap-model agent** — the autonomy is a fixed policy
table, executed by the same Opus advisor session, with the existing daemon
completion hook + one advisor-fired gate-waiting ping for notifications.

#### The allowlist (exhaustive — do NOT extend without a new user-confirmed contract)

| Gate | Stage | `--unattended` behaviour | Rationale |
|---|---|---|---|
| **1 Plan approval** | `planning-approved-pending-user` | **PARK + ping.** Never auto-cleared. | Judgment-heavy; a bad plan poisons the whole phase. |
| **2 Judgment-heavy DQ** | (mid-stage, any ADR-affecting / scope-changing / visible-to-others DQ surfaced via the §5.4 triage tree) | **PARK + ping.** Never auto-answered. | This is the cheap-model-drops-ADR failure class (`feedback_cheap_model_arm_drops_adr_constraints.md`, m1-b incident). |
| **3 CR triage** | `cr-triage-pending-user` | **PARK + ping.** Never auto-cleared. | Each finding is a compile-checkable judgment call; mis-bucketing a `critical` ships a bug. |
| **4 e2e local-vs-dispatch** | `phase-2-e2e-N` (first run) | **AUTO: `local`** (or `dispatch` iff `--e2e dispatch`). Set `phase_2_e2e_mode`, log `auto-approve-unattended`, no surface. | `local` is free + zero-billed + no public log. The only thing auto-`local` forecloses is the *billed, public* dispatch path — which a human should opt INTO, never out of. |
| **5 Merge confirm** | `merge-pending-user` | **PARK + ping.** `gh pr merge` NEVER fires unattended. | Irreversible, outward-facing, highest-stakes action in the phase. |
| **6 Retro sign-off** | `retro-pending-user` | **AUTO-CLEAR after 3-check sanity gate** (below). Any check fails → catch-fire. | Retro is internal, append-only, reversible, no code impact. The advisor authored it; sign-off is acknowledgement. |

**Plus all catch-fires stay catch-fires** — `--unattended` never suppresses a
catch-fire (subagent hard-refusal, attribution breach, non-allowlist §G4 fail,
ci-watcher exit-code surprise, daemon down, cycle-count ≥3). A catch-fire under
`--unattended` parks + pings exactly like a gate, then waits for manual resume.

#### Gate 6 — the 3-check retro sanity gate

Before auto-clearing gate 6, ALL THREE must pass. These are mechanical existence
checks, NOT a quality judgment (the advisor model authored the retro one stage
earlier; this gate guards against a *truncated / empty / wrong-shape* write, not
against a *bad* retro):

1. **Exists** — the retro file resolves (`.claude/PRPs/reports/<phase>-retro.md`
   or the `retros/` sibling, per the retro-author stage's write path) and is a
   regular file.
2. **Non-empty** — byte length ≥ a floor (≥ 800 bytes; a real retro is multi-KB).
3. **Required §-sections present** — `grep` confirms the retro contains the
   four-role retro signal headings the template mandates (per
   `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`):
   the per-role signal sections AND a "What to change" / §3-actions section AND a
   per-task complexity line. Absence of ANY → fail.

On any fail: **catch-fire**, dump which check failed, do NOT auto-clear. A
malformed retro is exactly the phantom-completion class `/brehon-verify` exists
to catch — auto-signing it off would launder a broken artifact past the audit.

#### Park-and-ping mechanics (gates 1, 2, 3, 5)

When a parked gate is reached under `--unattended`:

1. Record the gate as parked in `user_gate_history`:
   `{gate, decided_at: null, decision: "parked-pending-user", notes: "<one-line — what's waiting>"}`.
   `decided_at: null` marks it as not-yet-decided — the eventual user answer
   overwrites this entry with the real decision + timestamp.
2. **Fire ONE Telegram gate-waiting ping** (advisor-side, within Telegram scope —
   it is a `dq-blocking` / `merge-ready`-class notification per
   `feedback_telegram_scope_notification_only.md`): name the phase, the gate, and
   a one-line "what's waiting." No diff content, no secrets, no file dumps. MCP
   disconnected → silent skip + note in the auto-handover (pings are
   notifications, not gating signals — a missed ping never advances or blocks the
   gate). Per `branch-manager.md` "Telegram scope": this ping ALWAYS-confirm rule
   is satisfied by the user's `--unattended` opt-in itself — invoking `--unattended`
   IS the standing authorisation for the gate-waiting ping shape (and ONLY that
   shape). Do NOT batch; one ping per gate-reached event.
3. **Keep driving every mechanical transition that does NOT depend on this gate's
   answer.** A parked gate blocks only its own downstream branch. If the plan has
   independent work the gate doesn't gate (rare for gates 1/5 which are
   serializing; possible for a gate-2 DQ that's scoped to one cohort member while
   others proceed), advance it. When the gate is the sole serializing point
   (gate 1 plan-approval, gate 5 merge), there is nothing downstream to drive —
   park, ping, and `ScheduleWakeup` at idle cadence (1200s).
4. **On the tick the user returns** (a user message arrives, or the next poll
   after they've answered the relayed AskUserQuestion out-of-band), re-fire the
   gate's `AskUserQuestion` so the decision is made live, then overwrite the
   `parked-pending-user` history entry with the real `decision` + `decided_at`.

#### Audit honesty (load-bearing — per the "BM false-success" promoted pattern)

Auto-cleared gates write `decision: "auto-approve-unattended"` in
`user_gate_history`, **NEVER** `"user"`. This is non-negotiable: the audit trail
must always distinguish a gate a human touched from a gate the policy cleared. A
retro reading `user_gate_history` can then report exactly which gates ran
unattended. Writing `"user"` for an auto-clear is an attribution breach in the
same class as a non-advisor session writing `answered_by: "advisor"`
(`decision-queue.md` "Attribution integrity") — catch-fire if ever detected on
read.

#### Hard refusals specific to `--unattended`

- **NEVER auto-clear gates 1, 2, 3, or 5** under any flag combination. There is no
  override flag that extends the allowlist — extending it requires a new
  user-confirmed contract edited into THIS table, not a runtime flag.
- **NEVER fire `gh pr merge` while a session is unattended.** Gate 5 parks; the
  merge waits for a live human confirm.
- **NEVER write `decision: "user"` for an auto-cleared gate.** Use
  `"auto-approve-unattended"`.
- **NEVER suppress a catch-fire** because the session is unattended. Catch-fires
  park + ping, never silently continue.
- **NEVER auto-clear gate 6 with a failing sanity check.** A malformed retro is a
  catch-fire, not an auto-approve.

## Synthesis at session start (first invocation only)

On first invocation (no `<phase>-auto-state.json` exists yet), print a one-screen summary:

```
=== /auto-phase v1-SL-c-2 — initialized ===

Plan: <path> (<N> tasks, <complexity score>)
Trunk: governance-v0 @ <sha>
EliteDesk daemon: <active/inactive>
Forbidden window: <OK / wait until HH:MM>
Auto-state: .claude/auto-state/v1-SL-c-2.json (initialized, stage=init)

Starting bm-cut. Next user gate: plan approval (~30-90 min).
```

On subsequent invocations (state file exists), print:

```
=== /auto-phase v1-SL-c-2 — resuming at stage=<S> ===

Last action: <action> at <ts> (<elapsed> ago)
Pending Junior tasks: <list>
DQ pending: <count>
Next expected transition: <description>

Continuing.
```

## Refusals

- `--start-from` with non-canonical stage name → refuse with valid list.
- Plan missing → refuse, point at `/prp-core:prp-plan` or planning Junior dispatch.
- Branch not `governance-v0` → refuse, point at `git checkout governance-v0`.
- Working tree dirty → refuse, list files, suggest commit/stash.
- Concurrent advisor session writing same phase → refuse, point at `agent-activity.json`.
- Sub-phase named is already merged (PR closed-merged) → refuse, point at `/brehon-phase-transition`.
- `--no-bm-cut` flag without an existing phase branch matching `phase-<phase>` → refuse, drop the flag.

## What this skill is NOT

- **NOT** a fully-autonomous mode (`--auto-all-gates` was rejected per L15). **`--unattended` is NOT that** — it auto-clears only the two allowlisted no-judgment gates (4 e2e-local, 6 retro sign-off) and parks-and-pings the four judgment gates (1/2/3/5). Plan-approval, ADR/scope DQ, CR-triage, and merge ALWAYS need a live human. The rejected `--auto-all-gates` would have cleared those four; `--unattended` never does.
- **NOT** a cheap-model monitor agent. `--unattended` spins up no Haiku/cheap agent — the autonomy is a fixed policy table run by the same Opus advisor, with the existing daemon completion hook + one advisor-fired gate-waiting ping for notifications. (The original "spin up a cheap model to monitor" framing was dissolved at design time — the daemon hook + the advisor's own `ScheduleWakeup` loop already fill the monitor role; a cheap agent would add a hop + a mis-classification surface, not remove one.)
- **NOT** a replacement for `/start-brehon` (read-only state probe stays separate).
- **NOT** a model-tier change (Junior subagents stay at their tiered models per the four-role tiering patch).
- **NOT** a parallel-session coordinator (single advisor session at a time).
- **NOT** auto-applied — user must explicitly invoke `/auto-phase`. Default polling loop stays manual-trigger.

## See also

- `.claude/refs/auto-phase.md` — the orchestration state-machine rule (in-repo, mirror-aware).
- `.claude/rules/advisor-orchestrator.md` — canonical stage-shape source.
- `.claude/PRPs/templates/auto-phase-state.template.json` — runtime state schema.
- `.claude/commands/bm/bm-merge.md` — split gate (advisor inline) from execute (Junior) per L15.
- `feedback_subagent_delegation_for_multi_probe_commands.md` — the skill optionally delegates the polling tick itself to a `general-purpose` Explore subagent when parent context is hot (>500 KB).
- `feedback_read_only_commands_still_cost_context.md` — even read-only probes cost context tokens; cadence respects this.

## Pre-commit dogfood

Mentally simulated against `v1-SL-c-2` plan (per `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md`). Trace:

- T+0–60s: prerequisites + state init pass; auto-state JSON initialized.
- T+60s: bm-cut brief authored + pushed; Junior queued; ScheduleWakeup 120s.
- T+3m: bm-cut done → planning brief authored (PMD-consulted for "e2e testcontainers" lessons) + pushed; planning Junior queued; ScheduleWakeup 1200s.
- T+25m–60m: planning Junior done → DoD smoke + watchpoint gate pass → AskUserQuestion (gate 1).
- T+~70m: user approves → impl-cohort-1 (Task 0 pre-flight) queued → ScheduleWakeup 600s.
- T+~80m: Task 0 done → impl-cohort-2 (Task 1 — `grace_check_fired` e2e) queued → impl-task pushes + raises `validate-pending` DQ → advisor dispatches Phase 1 ci-watcher → ScheduleWakeup 270s.
- T+~90m: ci-watcher mutates DQ `result: pass` → daemon finalize-merges → advisor detects phase tip change → first Phase 2 e2e → AskUserQuestion (gate 4: local vs dispatch) → user picks local → cargo `run_in_background` started → ScheduleWakeup 1500s.
- T+~115m: cargo bg done with exit 0 → mutate Phase 2 DQ `result: pass` → impl-cohort-3 (Task 2). Subsequent Phase 2 runs auto-route local (cached).
- T+~3h: Tasks 3, 4, 5 cycle. bm-pr-pending → bm-pr Junior → cr-wait → ScheduleWakeup 600s.
- T+~3h15m: CR posts → bm-poll-cr → bm-triage → AskUserQuestion (gate 3) → user picks fix-in-pr scope.
- T+~4h: verify-running clean → AskUserQuestion (gate 5: merge confirm) → user confirms → **advisor runs `gh pr merge` inline** → verify branch deletion (L16) → author runlog `docs(advisor)` (L14 belt-and-braces) → push.
- T+~4h5m: retro-author → AskUserQuestion (gate 6) → user signs off → /brehon-phase-transition → DONE.

User touchpoints across the whole phase: 5 mandatory gates + 1 phase-transition skill ≈ 6 interruptions (+ ~3 push grants for brief commits) ≈ ~8 total interruptions across ~4 hours. Compared to c-1's ~15 interruptions in same wall-clock, ~50% reduction.

Worked through what the skill must NOT do mid-walk: re-litigate DQ #151 (planner-overridden complexity-17 proceed); re-author Task 6 retro (plan §13 owns retro authoring under c-2); skip Phase 2 first-run gate (must AskUserQuestion even if user previously picked local in c-1).
