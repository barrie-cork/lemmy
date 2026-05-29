# Advisor-orchestrator §1 polling-loop — incident narratives + hook taxonomies

Read-on-demand companion to `.claude/rules/advisor-orchestrator.md` §1 "Polling loop".
The rule file carries the terse operational statements; this file carries the **why** —
the incident post-mortems and the hook false-positive / false-negative taxonomies that
justify each ritual. Extracted 2026-05-29 (harness-audit P1) to keep the always-load
rule body lean while preserving the full reasoning for sessions that need it.

None of the §1 statements are referenced by `.pi/` by section anchor, so this extraction
is Pi-safe — the rule file's bullets stay; only the explanatory prose moved here.

## Surface-first ritual — why it competes against inherited-context momentum

The surface-first ritual (introduced 2026-05-22 after the post-RT-r2 session boundary
incident) requires that when ANY of these hold at session start — (a) the
UserPromptSubmit DQ-pending hook reports pending > 0, (b) `git worktree list` shows ≥2
worktrees, (c) the `session-start-multi-lane-check.sh` hook emitted a WARN — the FIRST
user-visible response in the session MUST be a one-line lane status:
`lanes: <CWD>:<branch> active; other-active: <list-of-other-active-lanes-or-"none">`.

The line goes BEFORE any task-execution response, BEFORE any tool call, even when an
inherited turn's stdout (e.g. a post-`/clear` recommendation) frames the next action.

**Why the ritual exists:** it competes against inherited-context momentum. On 2026-05-21
the canonical session pushed `cf93b7ba6` to `governance-v0` while another session was
driving `phase-v1-federation-inbound-c` + the schema-v3 migration — the worktree list
was visible but never surfaced; the inherited "plan RT-r2 now" anchored the session into
action before the lane check ran. Per `project_concurrent_advisor_sessions_2026_05_21.md`
+ `feedback_falsifiable_hypothesis_before_structural_fix.md`.

## SessionStart multi-lane check — false-positive / false-negative taxonomy

The tracked `.claude/hooks/session-start-multi-lane-check.sh` (added post-RT-r2 boundary
incident 2026-05-22) runs at every session start in lanes wired per the bootstrap
checklist. It reads `git worktree list`; for each non-CWD worktree on a `phase-v1-*` /
`phase-v2-*` / `phase-brehon-*` branch, it checks whether the branch tip was advanced
within the last 30 minutes (heuristic for "another session is actively driving"). On
detection: stderr WARN with this lane + the other active lane(s) + their last-commit age.
WARN-not-FAIL (exit 0 always); the WARN lands in SessionStart system-reminders so the
advisor sees it BEFORE the first tool call. Pairs with the surface-first ritual — the
hook is the mechanical signal, the ritual is the prose response.

- **False-positive class:** another lane's worktree exists but the session is idle. The
  threshold is generous to bias toward over-warning.
- **False-negative class:** a session active for >30 min without a commit.

Both are accepted because WARN-not-FAIL is cheap and the alternative (a sub-30-min-window
cliff) misses the real defect class — sessions that DID commit recently, like the
2026-05-21 fed-in-c session. Threshold tunable via `SESSION_START_MULTI_LANE_THRESHOLD`
env var.

## Pre-compact handover discipline — the validation example

Any advisor session likely to span `/compact`, session-end, or context truncation MUST
author a self-contained handover file BEFORE the boundary, at
`.claude/PRPs/handovers/<phase>-<scope>-<date>.md`. The file's body MUST be readable by a
future session with zero conversation context — it states the current sub-phase, the
stage in the state machine, the last commit on the relevant branch, the next concrete
action, and any cross-session dependencies (DQ pending entries by id, concurrent session
activity).

**Validation:** the v1-dq-schema-r1 Cohort 2 handover
(`.claude/PRPs/handovers/v1-dq-schema-r1-cohort-2-handover-2026-05-22.md`) enabled a
first-pass resume in 3 tool calls. Commit + push the handover file BEFORE invoking
`/compact`; the handover lives on `governance-v0` even if the work is on a phase branch,
so the canonical checkout always sees the latest. Introduced post-v1-retro-followups-r1,
2026-05-22.
