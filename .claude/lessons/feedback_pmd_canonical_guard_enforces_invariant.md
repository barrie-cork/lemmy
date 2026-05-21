---
name: SessionStart canonical-PMD guard is now SHIPPED (closes PENDING status)
description: Pre-v1-rls-r1 the canonical-PMD cross-lane invariant was protected by documentation only. v1-rls-r1 shipped .claude/hooks/pmd-canonical-guard.sh + the lane-bootstrap-checklist update; the SessionStart wiring is per-lane in .claude/settings.local.json per the updated checklist. The prior lesson's fix status PENDING is now closed.
type: feedback
---

# pmd-canonical-guard.sh — canonical-PMD invariant enforced at SessionStart

## Pre-v1-rls-r1 state

Lesson `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` carried `fix status: PENDING` — the canonical-PMD cross-lane invariant existed only as documentary text. The tracked `.mcp.json.example` template encoded the rule and a `_comment_pmd_cross_lane` guard key recorded the intent, but no runtime check surfaced drift. The v1-ship-1 stranding incident (21 retros written into a lane-local DB invisible to the canonical-DB-reading Stop hook) was the cost: the rule was correct, the documentation existed, the lessons named the failure mode, and the bug still recurred. Documentary protection alone is insufficient when the failure mode is silent.

## Post-v1-rls-r1 state

Track A Task 3 ships `.claude/hooks/pmd-canonical-guard.sh` — a tracked SessionStart hook that resolves the canonical PMD path via `git rev-parse --git-common-dir`, reads the current lane's `.mcp.json`, compares `PROJECT_MEMORY_DB` against canonical, and emits a multi-line stderr WARN on mismatch naming BOTH paths plus the 1-line fix. Exit 0 on every code path — WARN-not-FAIL per autonomy-readiness. Track B Task 6 ships the lane-bootstrap-checklist (`feedback_phase_lane_worktree_bootstrap_checklist.md`) documenting the per-worktree wiring of this hook into `.claude/settings.local.json`. Per DQ #301 the wiring is applied in both canonical `C:/Users/barri/Developer/brehon-fork/.claude/settings.local.json` AND lane-dedicated `C:/Users/barri/Developer/brehon-fork-rls-r1/.claude/settings.local.json` (gitignored; manual hand-off per §11 of the plan).

## How to apply

- **New lanes:** apply step 6 of the bootstrap checklist before opening Claude Code in the new worktree CWD. Verify the SessionStart banner shows no `pmd-canonical-guard.sh` WARN.
- **Existing lanes:** wire on the next bootstrap cycle. No retro-fit is strictly required since the structural fix's gate is set the moment a session opens in a wired lane.
- **Drift detection:** if a SessionStart shows the WARN banner, edit `.mcp.json` per the banner's 1-line fix and restart MCP. The running MCP cached its handle at startup; the script CANNOT fix the running session — only the WARN tells the human to restart.

## See also

- `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` — the prior PENDING this closes
- `feedback_pmd_cross_lane_canonical_db.md` — invariant context
- `feedback_phase_lane_worktree_bootstrap_checklist.md` (Task 6) — the wiring checklist
- `.claude/rules/pmd-invariants.md` invariant #1 + #5 (Task 2) — authoritative reference
- `.claude/hooks/pmd-canonical-guard.sh` (Task 3) — the script itself
- `.claude/PRPs/plans/v1-rls-r1.plan.md` — this plan
