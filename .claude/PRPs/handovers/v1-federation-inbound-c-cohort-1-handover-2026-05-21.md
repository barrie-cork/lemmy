---
author: advisor (canonical brehon-fork checkout session, post-bm-cut-#393 + lane bootstrap)
next_role: advisor (lane-dedicated session on brehon-fork-fed-in-c)
next_session: lane worktree at C:/Users/barri/Developer/brehon-fork-fed-in-c, branch phase-v1-federation-inbound-c
authored: 2026-05-21
purpose: Hand off cohort 1 dispatch (DQ id pre-reservation + impl briefs 1+2 + Junior cohort queue) from the canonical brehon-fork session to a fresh lane-dedicated session on the brehon-fork-fed-in-c worktree. Read this brief first; then run the session-start ritual; then proceed to cohort 1.
---

# Advisor handover — v1-federation-inbound-c, cohort 1 dispatch

## What's on the phase branch (current state)

`phase-v1-federation-inbound-c` on origin @ `cb37dc584`:

- `22f15bd9a feat(fed-in-c): author v1-federation-inbound-c plan (scope (a) only)` — the approved plan, cherry-picked from planning Junior #391 worker commit `3b187627e`.
- `6dc489c9e chore(advisor): author v1-federation-inbound-c bm-cut brief (plan-approved user gate 1)` — base of phase branch (cut from this trunk SHA).
- `21583e798 chore(advisor): re-apply v1-federation-inbound-c runlog — Junior #393 finalize deleted index-only file` — advisor belt-and-braces; see DQ #326 for RCA.
- `cb37dc584 chore(advisor): log DQ #326 — bm-cut #393 finalize deleted index-only runlog (kind:log)` — head.

DQ on phase branch: 122 resolved + 1 new resolved (#326 kind:log) = 123 resolved, 0 pending. Both clarify-DQ #324 + #325 (e2e INSERT-vs-UPDATE + workaround-comment plan) are already resolved on this branch (carried over from governance-v0).

Plan @ `.claude/PRPs/plans/v1-federation-inbound-c.plan.md` (also accessible via `git show 22f15bd9a:.claude/PRPs/plans/v1-federation-inbound-c.plan.md`). 870 lines; 5 tasks; complexity 2/10.

Runlog @ `.claude/runlog/v1-federation-inbound-c-runlog.md` with `## bm:` cut entry + `## advisor:` re-apply entry.

## Closing state assertions (canonical session)

- Canonical `brehon-fork` CWD: `governance-v0` @ `6dc489c9e` (clean; `git status --short` only shows pre-existing untracked files unrelated to this session).
- Lane worktree exists at `C:/Users/barri/Developer/brehon-fork-fed-in-c` on `phase-v1-federation-inbound-c` @ `cb37dc584` (clean).
- Lane `.mcp.json` carries canonical `PROJECT_MEMORY_DB: "C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db"` (verified at copy).
- Lane `.claude/settings.local.json` copied from canonical (inherits permissions + allow rules).
- Daemon-local `/srv/brehon-fork governance-v0` @ `6dc489c9e` (0/0 ahead/behind origin after this session's hard-reset recovery).
- Junior #391 (planning) and #393 (bm-cut) both `done`; no active jobs.

## Open watch-items (NOT blockers, but heads-up for retro harvest)

1. **`pmd-canonical-guard.sh` NOT wired** in either canonical or lane `settings.local.json` — the file is present at `.claude/hooks/pmd-canonical-guard.sh` but never plugged into a `SessionStart` hook. Per the DQ #301 dual-wire requirement in `feedback_phase_lane_worktree_bootstrap_checklist.md` Step 6, both files should carry the wiring. Risk in lane: SessionStart won't WARN on PMD-path drift. Mitigation in lane: `.mcp.json` already verified canonical at copy-time. Defer the wiring decision to retro (or fix opportunistically if the lane session has spare cycles).

2. **bm-cut #393 finalize-agent runlog deletion** — DQ #326 captures the full RCA. Phase branch on origin is now self-consistent (runlog re-applied at `21583e798`), but the structural fix is deferred: either (a) edit `.claude/hooks/allow-prp-deliverables.sh` to allow `.claude/runlog/**` writes, or (b) harden Junior finalize-agent prompt to cross-check assistant deliverables before committing deletions. Both are retro-decided. **Watch for recurrence in the next bm-task (`bm-pr` or `bm-merge`).**

## Next actions (in dependency order, by the lane session)

### 1. Session-start ritual (mandatory)

Per `advisor-orchestrator.md` §1 "Polling loop" + `multi-lane-worktree.md` §"Session-start ritual":

```bash
pwd                                              # expect C:/Users/barri/Developer/brehon-fork-fed-in-c
git branch --show-current                        # expect phase-v1-federation-inbound-c
git worktree list                                # see all active worktrees
git fetch origin phase-v1-federation-inbound-c   # ensure tip is current
git log -3 --oneline                             # confirm cb37dc584 visible
```

Confirm: lane CWD, lane branch, no concurrent advisor session writing on the same `phase-v1-federation-inbound-c` (the canonical session is now closed for fed-in-c work; only the conformance-audit lane is active on a different branch).

### 2. Pre-reserve cohort 1 DQ ids (PRECON-7, Option 3)

Per `feedback_cohort_dq_id_collision.md`: BEFORE dispatching the [P] cohort (Tasks 1+2), reserve two `kind: "validate-pending-laptop"` stubs in `pending[]` on the phase branch DQ. Each stub's `commands` array is null at reserve time (impl-task fills it post-push); each stub names its phase_task explicitly.

next_id at handover author time: walk live + archives + cross-lane DQs → expect **327** (Task 1 reserve) and **328** (Task 2 reserve). Re-run the cross-archive next_id walk at reserve time:

```python
import json, glob, io
all_ids = []
paths = [
  'C:/Users/barri/Developer/brehon-fork-fed-in-c/.claude/decision-queue.json',
] + glob.glob('C:/Users/barri/Developer/brehon-fork-fed-in-c/.claude/decision-queue-archive-*.json') + glob.glob('C:/Users/barri/Developer/brehon-fork/.claude/decision-queue.json') + glob.glob('C:/Users/barri/Developer/brehon-fork/.claude/decision-queue-archive-*.json')
for p in paths:
    with io.open(p, encoding='utf-8') as f:
        d = json.load(f)
    all_ids += [e['id'] for e in d.get('pending',[]) + d.get('resolved',[])]
print('next_id:', max(all_ids)+1)
```

Also walk active worker-branch refs per `multi-lane-worktree.md` §"Worktree-aware DQ id discipline" — the brehon-conformance-audit lane may have advanced its DQ. Use `scripts/brehon/git-show-json.sh origin/phase-brehon-conformance-audit .claude/decision-queue.json` and include in the max.

Stub shape:

```json
{
  "id": <reserved>,
  "from": "advisor",
  "kind": "validate-pending-laptop",
  "timestamp": "<NOW_ISO>",
  "question": "Reserved for v1-federation-inbound-c cohort 1 task <N>",
  "options": [],
  "context": "Pre-reserved per PRECON-7 Option 3 (feedback_cohort_dq_id_collision.md). impl-task #<N> fills commands[] + branch + workflow_run_id-equivalent post-push.",
  "answer": "reserved for v1-federation-inbound-c cohort 1 task <N>",
  "answered_by": null,
  "resolved_at": null,
  "commands": null,
  "branch": null,
  "phase_task": <N>,
  "result": null,
  "log_slice": null,
  "failed_commands": null
}
```

Commit subject (atomic): `chore(advisor): pre-reserve cohort 1 DQ ids #<id-A> + #<id-B> for v1-federation-inbound-c Tasks 1+2`.

Push to phase branch.

### 3. Author impl briefs 1+2

Per `advisor-orchestrator.md` §2.1: impl-task briefs are committed on the phase branch (NOT on governance-v0). Paths:

- `.claude/PRPs/briefs/v1-federation-inbound-c-impl-1.md`
- `.claude/PRPs/briefs/v1-federation-inbound-c-impl-2.md`

Canonical-schema-first reference: read the most recent shipped impl-task brief (likely under `.claude/PRPs/briefs/v1-federation-inbound-b-impl-*.md` on governance-v0). Each brief must:

- Carry the YAML FILES block matching plan §13 task (`creates`, `modifies`, `requires`).
- Quote plan §10.1 verbatim as MIRROR ref.
- Cite the assigned DQ stub id in §4 Constraints.
- Apply §2.4 mandatory file-class lesson injection — `inbox.rs`/`publish_trust_attestation.rs` edits → `feedback_lemmy_error_no_std_error.md` (Case A) NOT triggered here (not e2e), but `feedback_clippy_test_style.md` for the validate-step.
- Specify the §15.1 + §15.2 commands verbatim for the impl-task's validate-pending-laptop DQ stub.

Cohort dispatch is parallel (Tasks 1+2 share zero IMPLEMENT files per plan §13 YAML overlap). Brief paths are unique; each gets its own worktree on the daemon side.

### 4. Queue Junior cohort (parallel)

```python
mcp__junior-brehon__create_task(
  description="[role:impl-task] v1-fed-in-c task 1 — see .claude/PRPs/briefs/v1-federation-inbound-c-impl-1.md",
  base_branch="phase-v1-federation-inbound-c"
)
mcp__junior-brehon__create_task(
  description="[role:impl-task] v1-fed-in-c task 2 — see .claude/PRPs/briefs/v1-federation-inbound-c-impl-2.md",
  base_branch="phase-v1-federation-inbound-c"
)
```

Send both in the same message (parallel tool uses) per `advisor-orchestrator.md` §4.1 step 7.

### 5. After cohort completes: dispatch Task 3 (e2e regression test)

Task 3 has `requires: [1, 2]` (per plan §13). Wait for both Tasks 1+2 finalize-merged + validate-pending-laptop resolved-pass before queuing Task 3.

## DQ id pre-reservation atomicity (important)

Per `multi-lane-worktree.md` §"Hard refusals" #6 (atomic read-mutate-commit): the cohort 1 DQ id reservation MUST be a single uninterrupted shell sequence (read → mutate → verify → add → commit → push) per the canonical-checkout-write protocol. The lane worktree is less race-prone than canonical (only one session writes phase-branch DQ), but the discipline is the same: `git fetch` → re-read → re-compute next_id → mutate → verify → commit → push, all in one go.

## Companion files (read at session start)

- `.claude/rules/advisor-orchestrator.md` — §1, §2.1, §2.4, §3.1, §4.1, §5.2 (validate-pending-laptop handler).
- `.claude/rules/multi-lane-worktree.md` — Hard refusals #1-6.
- `.claude/rules/decision-queue.md` — v2 schema + attribution rules + Hard refusals.
- `.claude/PRPs/plans/v1-federation-inbound-c.plan.md` — the full plan.
- `.claude/PRPs/briefs/v1-federation-inbound-c-bm-cut-1.md` — for canonical-shape reference of the bm-task brief.
- `.claude/lessons/feedback_cohort_dq_id_collision.md` — PRECON-7 Option 3.
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` — bootstrap reference.
- `.claude/lessons/feedback_advisor_authoring_under_daemon_stress.md` — pace-yourself reminder.

## Out-of-scope (do NOT do in cohort 1 dispatch)

- Wiring `pmd-canonical-guard.sh` in `settings.local.json` (deferred to retro decision).
- Editing `.claude/hooks/allow-prp-deliverables.sh` (deferred to retro decision).
- Any commit touching `crates/**` or `migrations/**` (impl-task subagent's lane, not advisor's).
- Any push to `governance-v0` (lane session is phase-branch only).

---

_Brief author: advisor session (canonical brehon-fork checkout, `governance-v0` @ `6dc489c9e`, 2026-05-21). Closing this session; user opens a fresh Claude Code session in `C:/Users/barri/Developer/brehon-fork-fed-in-c` per `multi-lane-worktree.md` §"Hard refusals" #5._
