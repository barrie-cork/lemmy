#!/usr/bin/env bash
# comparator-token-extract.sh — extract token/cost/compaction signals from a
# Pi --mode json trace file (trace.jsonl).
#
# Per .claude/PRPs/specs/pi-model-comparator.spec.md §6 + §7: the full trace
# is the authoritative forensic capture. This script extracts the structured
# signals the eval report and judge step need.
#
# Usage:
#   comparator-token-extract.sh <trace.jsonl> [--meta <meta.json>]
#
# Emits a JSON object to stdout:
#   {
#     "trace": <path>,
#     "total_lines": <n>,
#     "event_counts": { "turn_start": n, "tool_execution_start": n, ... },
#     "tool_calls": { "read": n, "write": n, ... },
#     "turns": <n>,
#     "compaction_count": <n>,
#     "input_tokens": <n or null>,
#     "output_tokens": <n or null>,
#     "wall_seconds": <n or null>,   // from meta.json if available
#     "plans_produced": <n or null>, // from meta.json if available
#     "pi_exit": <n or null>,        // from meta.json if available
#     "run_complete": <bool>,        // true if agent_end seen
#     "last_event_type": <str>
#   }

set -euo pipefail

TRACE="${1:-}"
META_FILE=""

if [[ -z "${TRACE}" ]]; then
  echo "ERROR: usage: comparator-token-extract.sh <trace.jsonl> [--meta <meta.json>]" >&2
  exit 2
fi

shift
while [[ $# -gt 0 ]]; do
  case "$1" in
    --meta) META_FILE="$2"; shift 2;;
    *) echo "ERROR: unknown arg: $1" >&2; exit 2;;
  esac
done

if [[ ! -f "${TRACE}" ]]; then
  echo "ERROR: trace file not found: ${TRACE}" >&2
  exit 2
fi

python3 - "${TRACE}" "${META_FILE}" <<'PYEOF'
import json, sys, os

trace_path = sys.argv[1]
meta_path = sys.argv[2] if len(sys.argv) > 2 else ""

# Parse trace events
event_counts = {}
tool_calls = {}
turns = 0
compactions = 0
input_tokens = None
output_tokens = None
last_event_type = None
run_complete = False
total_lines = 0

with open(trace_path, encoding='utf-8', errors='replace') as f:
    for line in f:
        line = line.strip()
        if not line:
            continue
        total_lines += 1
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        t = obj.get('type', '?')
        event_counts[t] = event_counts.get(t, 0) + 1
        last_event_type = t

        if t == 'turn_start':
            turns += 1
        elif t == 'compaction_start':
            compactions += 1
        elif t == 'agent_end':
            run_complete = True
        elif t == 'tool_execution_start':
            tool_name = obj.get('toolName', 'unknown')
            tool_calls[tool_name] = tool_calls.get(tool_name, 0) + 1
        elif t == 'message_end':
            msg = obj.get('message', {})
            usage = msg.get('usage', {})
            if usage:
                inp = usage.get('inputTokens') or usage.get('input_tokens')
                out = usage.get('outputTokens') or usage.get('output_tokens')
                if inp is not None:
                    input_tokens = (input_tokens or 0) + inp
                if out is not None:
                    output_tokens = (output_tokens or 0) + out

# Load meta.json if available
meta = {}
if meta_path and os.path.isfile(meta_path):
    try:
        with open(meta_path) as f:
            meta = json.load(f)
    except Exception:
        pass

# Derive plans_produced from git commits on the ab-cell branch — the meta.json
# counter never increments (run-comparator.sh counts git diff --cached at teardown
# time, which races with Pi's own commit; the worktree is gone by then). Authoritative
# check: commits touching .claude/PRPs/plans/ on ab-cell/<exp>-<arm>.
# Requires the trace path to be inside RESULTS_DIR (…/runs/<exp>/<arm>/trace.jsonl).
import re as _re, subprocess as _sub
plans_produced_git = None
try:
    # Infer experiment+arm from trace path: …/runs/<exp>/<arm>/trace.jsonl
    parts = trace_path.replace('\\', '/').split('/')
    # Find 'runs' segment; exp is next, arm after that
    if 'runs' in parts:
        runs_idx = len(parts) - 1 - parts[::-1].index('runs')
        if runs_idx + 2 < len(parts):
            exp = parts[runs_idx + 1]
            arm = parts[runs_idx + 2].rstrip('/')
            cell_branch = f'ab-cell/{exp}-{arm}'
            # Try to resolve the repo root (two levels up from scripts/brehon or via git)
            import os as _os
            repo_guess = _os.path.abspath(_os.path.join(_os.path.dirname(trace_path), '..', '..', '..', '..'))
            for candidate in [repo_guess]:
                try:
                    out = _sub.check_output(
                        ['git', '-C', candidate, 'log', cell_branch,
                         '--oneline', '--', '.claude/PRPs/plans/'],
                        stderr=_sub.DEVNULL, text=True
                    ).strip()
                    plans_produced_git = len([l for l in out.splitlines() if l])
                    break
                except Exception:
                    pass
except Exception:
    pass

# Use git-derived count when available; fall back to meta.json (may be 0 due to the
# timing race — treat as unreliable if git count disagrees).
plans_produced_meta = meta.get('plans_produced')
plans_produced = plans_produced_git if plans_produced_git is not None else plans_produced_meta

result = {
    "trace": trace_path,
    "total_lines": total_lines,
    "event_counts": dict(sorted(event_counts.items(), key=lambda x: -x[1])),
    "tool_calls": dict(sorted(tool_calls.items(), key=lambda x: -x[1])),
    "turns": turns,
    "compaction_count": compactions,
    "input_tokens": input_tokens,
    "output_tokens": output_tokens,
    "wall_seconds": meta.get('wall_seconds'),
    "plans_produced": plans_produced,
    "plans_produced_meta": plans_produced_meta,  # raw meta.json value for comparison
    "plans_produced_git": plans_produced_git,     # git-derived count (authoritative)
    "pi_exit": meta.get('pi_exit'),
    "run_complete": run_complete,
    "last_event_type": last_event_type,
}
print(json.dumps(result, indent=2))
PYEOF
