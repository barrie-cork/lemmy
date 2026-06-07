#!/usr/bin/env bash
# comparator-code-gates.sh — deterministic (code-based) grading of a plan output.
#
# Per .claude/PRPs/specs/pi-model-comparator.spec.md §7: best-practice ranks
# code-based grading FIRST (fastest, most reliable, no LLM). This script grades
# the OBJECTIVE planning-rubric dimensions mechanically — pass/fail, no judgment.
# The nuanced dimensions (task-decomposition quality, T1-preemption reasoning)
# go to the third-model LLM judge separately.
#
# Run against BOTH arms' plan files (challenger output + Opus ground truth) so the
# divergence report compares like-for-like.
#
# Usage:
#   comparator-code-gates.sh <plan-file.md> [<repo-root-for-DoD-dry-run>]
#
# Emits a JSON object of gate results to stdout.

set -uo pipefail

PLAN="${1:-}"
REPO="${2:-$(pwd)}"

if [[ -z "${PLAN}" || ! -f "${PLAN}" ]]; then
  echo "ERROR: plan file not found: ${PLAN}" >&2
  exit 2
fi

# Gate 1: section completeness — count canonical '## ' headers (target: 20-section schema).
SECTIONS=$(grep -c '^## ' "${PLAN}" 2>/dev/null || echo 0)

# Gate 2: watchpoint specificity — §4 watchpoints must cite a file/table/line.
# Heuristic: lines in the watchpoints area referencing a path (crates/, .rs, a table name) or schema.rs:NN.
WATCHPOINT_CITES=$(grep -iE 'watch' "${PLAN}" | grep -cE 'crates/|\.rs|schema\.rs|migrations/|[a-z_]+\.[a-z_]+' 2>/dev/null || echo 0)

# Gate 3: ADR-015 preservation — the plan must name the pseudonym gate + a callsite, not just "respects ADR-015".
ADR015_NAMED=$(grep -cE 'actor_pseudonym|ADR-015|pseudonym' "${PLAN}" 2>/dev/null || echo 0)
ADR015_CALLSITE=$(grep -cE 'actor_pseudonym\.(pseudonym|get_or_create)|validate_identity_policy' "${PLAN}" 2>/dev/null || echo 0)

# Gate 4: §13 task decomposition present + MIRROR refs.
TASK_COUNT=$(grep -cE '^### Task [0-9]+|^### T[0-9]+|^#### Task' "${PLAN}" 2>/dev/null || echo 0)
MIRROR_REFS=$(grep -cE 'MIRROR|SOURCE:.*:[0-9]' "${PLAN}" 2>/dev/null || echo 0)

# Gate 5: §16a stories present (if the schema uses them) + checkpoint commands.
STORY_COUNT=$(grep -ciE 'story|§16a|checkpoint' "${PLAN}" 2>/dev/null || echo 0)

# Gate 6: §15 DoD command shape — every validation line should be a runnable cargo/diesel command.
# Count cargo/diesel commands; flag the -p+--features full footgun.
CARGO_CMDS=$(grep -cE 'cargo (check|clippy|test|build|run)|diesel migration' "${PLAN}" 2>/dev/null || echo 0)
P_FEATURES_FOOTGUN=$(grep -cE 'cargo (check|clippy|test).*-p .*--features full' "${PLAN}" 2>/dev/null || echo 0)

# Gate 7: T1-preemption SIGNAL (code-detectable portion) — does the plan mention
# validate-pending / lane mode / Mode A / Mode B / worktree at all? (The QUALITY
# of preemption is the LLM judge's job; this is just presence.)
T1_SIGNAL=$(grep -ciE 'validate-pending|lane mode|mode a|mode b|worktree|trunk.*phase sync' "${PLAN}" 2>/dev/null || echo 0)

{
  printf '{'
  printf '"plan":"%s",' "${PLAN}"
  printf '"sections":%s,' "${SECTIONS}"
  printf '"watchpoint_cites":%s,' "${WATCHPOINT_CITES}"
  printf '"adr015_named":%s,' "${ADR015_NAMED}"
  printf '"adr015_callsite":%s,' "${ADR015_CALLSITE}"
  printf '"task_count":%s,' "${TASK_COUNT}"
  printf '"mirror_refs":%s,' "${MIRROR_REFS}"
  printf '"story_signal":%s,' "${STORY_COUNT}"
  printf '"cargo_cmds":%s,' "${CARGO_CMDS}"
  printf '"p_features_footgun":%s,' "${P_FEATURES_FOOTGUN}"
  printf '"t1_preemption_signal":%s' "${T1_SIGNAL}"
  printf '}\n'
}
