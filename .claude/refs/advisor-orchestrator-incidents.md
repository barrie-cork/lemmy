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

**Condition (d) grep procedure:** run `grep -h "SUSPENDED\|RE-ARMED" ~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/workflow_state_*.md 2>/dev/null | head -3` and cross-check against the MEMORY.md index line for the same phase. Mismatch → surface WARN + raise AskUserQuestion for the authoritative state before proceeding.

**2× recurrence after initial incident:**
- 2026-06-07: `.mcp.json` concurrent rewrite between two sessions on the same checkout.
- 2026-06-12: AB trial concurrent suspension (workflow_state_*.md flag contradicted MEMORY.md index).

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

## Session-start stash check

**Incident (2026-06-01):** a session found `stash@{0}` holding an unresolved Phase 2 e2e DQ entry for `v1-SL-d` (shipped weeks earlier). The stash looked like noise (old phase, nothing happening) but contained a live DQ entry. Dropping without inspecting would have permanently lost the entry. The session recovered it and re-applied the DQ mutation only because the inspect-before-drop discipline was followed. `git stash drop` is irreversible — once dropped, the stash state is gone from the reflog after the next GC.

## Finalize-merge look-order

Checking origin first shows a stale pre-merge tip and triggers a multi-probe hunt: the daemon's finalize-merge (`mcp__junior-brehon__finalize_task`) lands on daemon-local `governance-v0` before the daemon pushes to origin. `git ls-remote origin governance-v0` therefore returns the pre-merge tip, and any check that reads origin first will see the wrong SHA — a polling loop then burns 2–3 extra probes looking for a commit that only the daemon has locally. Look daemon-local first (`ssh homeserver "cd /srv/brehon-fork && git log governance-v0 -1 --oneline"`), then verify via origin if the daemon tip looks unexpectedly old.

## OQ resolvability check

**Why the check saves time:** a 15-min resolution of a parked OQ prevents the same ambiguity from being re-surfaced at every stage of the planning + impl cycle: the planner adds a note, the impl-task adds another note, the CR reviewer asks a question. Resolving it once costs ~15 min; not resolving it costs ~1 hour of diffuse friction.

**OQ-V2-10 incident (2026-06-01 retro finding):** OQ-V2-10 was tagged `Status: parked` pending an ecosystem-level fix. That fix shipped in a library update ~2 months before the 2026-06-01 planning session. No session had run the resolvability check in the interim, so the OQ sat parked and resolvable for 6+ months — discovered only when the V2a planning session finally hit the blocking condition and checked.

## Shape-G-disabled fast-path

**HTTP-422 background:** `cargo-validate-workspace.yml` can be `disabled_manually` at the GitHub API level (via `gh workflow disable`). When disabled, `gh workflow run cargo-validate-workspace.yml ...` returns HTTP 422 "Workflow is disabled" — the workflow file exists in `.github/workflows/` but the API refuses to queue a run. Workers that succeed in committing + pushing will correctly detect this via `gh workflow run`'s exit code and write `workflow_run_id: 0` as the sentinel.

**Test-dogfood 2026-06-12:** During the Shape-G residual-only test, `gh workflow run` returned 422 on `cargo-validate-workspace.yml`. The advisor detected the sentinel (`workflow_run_id: 0`), skipped `gh workflow run`, created a throwaway worktree on the impl branch, ran `commands[]` locally, and mutated the DQ entry with `answered_by: "advisor-laptop"` + `result: "pass"`. Total elapsed: ~8 min vs the ~12-min GH Actions baseline.
