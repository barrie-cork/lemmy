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

set -euo pipefail

PLAN="${1:-}"
REPO="${2:-$(pwd)}"

if [[ -z "${PLAN}" || ! -f "${PLAN}" ]]; then
  echo "ERROR: plan file not found: ${PLAN}" >&2
  exit 2
fi

# All counting done in Python to avoid bash grep-pipe exit-code traps
# (grep -c returns exit 1 on 0 matches, which || echo 0 doubles the output
# in a pipeline — use re.findall instead, always exits 0).
python3 - "${PLAN}" <<'PYEOF'
import re, sys, json

plan_path = sys.argv[1]
with open(plan_path, encoding='utf-8', errors='replace') as f:
    text = f.read()
lines = text.splitlines()

# Gate 1: section completeness — count '## ' headers (target: 20-section schema).
sections = sum(1 for l in lines if re.match(r'^## ', l))

# Gate 2: watchpoint specificity — lines mentioning "watch" that also cite a
# concrete artefact (file path, .rs, table name, schema.rs:NN).
watch_lines = [l for l in lines if re.search(r'watch', l, re.I)]
watchpoint_cites = sum(
    1 for l in watch_lines
    if re.search(r'crates/|\.rs|schema\.rs|migrations/|[a-z_]+\.[a-z_]+', l)
)

# Gate 3: ADR-015 — must name the pseudonym gate + include a concrete callsite.
adr015_named = len(re.findall(r'actor_pseudonym|ADR-015|pseudonym', text, re.I))
adr015_callsite = len(re.findall(
    r'actor_pseudonym\.(pseudonym|get_or_create)|validate_identity_policy', text
))

# Gate 4: §13 task decomposition + MIRROR refs.
task_count = len(re.findall(r'^### Task \d+|^### T\d+|^#### Task', text, re.M))
mirror_refs = len(re.findall(r'MIRROR|SOURCE:.*:\d', text))

# Gate 5: §16a stories + checkpoint commands.
story_signal = len(re.findall(r'story|§16a|checkpoint', text, re.I))

# Gate 6: §15 DoD — cargo/diesel commands; flag -p+--features full footgun.
cargo_cmds = len(re.findall(
    r'cargo (?:check|clippy|test|build|run)|diesel migration', text
))
p_features_footgun = len(re.findall(
    r'cargo (?:check|clippy|test).*-p .*--features full', text
))

# Gate 7: T1-preemption signal — presence of validate-pending / lane-mode terms.
t1_signal = len(re.findall(
    r'validate-pending|lane mode|mode a|mode b|worktree|trunk.*phase sync', text, re.I
))

result = {
    "plan": plan_path,
    "sections": sections,
    "watchpoint_cites": watchpoint_cites,
    "adr015_named": adr015_named,
    "adr015_callsite": adr015_callsite,
    "task_count": task_count,
    "mirror_refs": mirror_refs,
    "story_signal": story_signal,
    "cargo_cmds": cargo_cmds,
    "p_features_footgun": p_features_footgun,
    "t1_preemption_signal": t1_signal,
}
print(json.dumps(result))
PYEOF
