#!/usr/bin/env bash
# comparator-eval-report.sh — assemble the 4 eval deliverables from one experiment.
#
# Per .claude/PRPs/specs/pi-model-comparator.spec.md §7 "Eval artifacts produced":
#   1. Per-dimension divergence report
#   2. Failure-mode → fix map
#   3. Token/cost breakdown
#   4. Replay-ready prompt+context bundle (already in challenger/replay-bundle/)
#
# Inputs:
#   - code-gate scores: comparator-code-gates.sh output for both arms
#   - judge scores: judge-summary.json from comparator-judge.sh
#   - token signals: token-signals.json from comparator-judge.sh
#
# Usage:
#   comparator-eval-report.sh \
#     --experiment <id>                        e.g. planning-001
#     --runs-dir <dir>                         default: .claude/PRPs/comparator/runs/
#     [--control-plan <path>]                  Opus plan (auto-detected if not given)
#     [--output <file>]                        default: <runs-dir>/<exp>/eval-report.md

set -euo pipefail

EXPERIMENT=""
RUNS_DIR=""
CONTROL_PLAN=""
OUTPUT_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --experiment)  EXPERIMENT="$2";  shift 2;;
    --runs-dir)    RUNS_DIR="$2";    shift 2;;
    --control-plan) CONTROL_PLAN="$2"; shift 2;;
    --output)      OUTPUT_FILE="$2"; shift 2;;
    -h|--help)     sed -n '1,20p' "$0"; exit 0;;
    *) echo "ERROR: unknown arg: $1" >&2; exit 2;;
  esac
done

if [[ -z "${EXPERIMENT}" ]]; then
  echo "ERROR: --experiment is required" >&2; exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
RUNS_DIR="${RUNS_DIR:-${REPO_ROOT}/.claude/PRPs/comparator/runs}"
EXP_DIR="${RUNS_DIR}/${EXPERIMENT}"
JUDGE_DIR="${EXP_DIR}/judge"
CHALLENGER_DIR="${EXP_DIR}/challenger"
OUTPUT_FILE="${OUTPUT_FILE:-${EXP_DIR}/eval-report.md}"

if [[ ! -d "${EXP_DIR}" ]]; then
  echo "ERROR: experiment dir not found: ${EXP_DIR}" >&2; exit 2
fi

# Auto-detect control plan if not given.
if [[ -z "${CONTROL_PLAN}" ]]; then
  # Standard location for m2-late planning experiment.
  CONTROL_PLAN="${REPO_ROOT}/.claude/PRPs/plans/m2-late.plan.md"
fi

if [[ ! -f "${CONTROL_PLAN}" ]]; then
  echo "WARN: control plan not found at ${CONTROL_PLAN}; code-gate scores will be partial" >&2
fi

CODE_GATES="${SCRIPT_DIR}/comparator-code-gates.sh"

# Run code gates on control plan.
CONTROL_GATES_FILE="${EXP_DIR}/control-code-gates.json"
CHALLENGER_GATES_FILE="${EXP_DIR}/challenger-code-gates.json"

echo "  running code gates on control plan..."
if [[ -f "${CONTROL_PLAN}" ]]; then
  "${CODE_GATES}" "${CONTROL_PLAN}" "${REPO_ROOT}" > "${CONTROL_GATES_FILE}" 2>/dev/null || \
    echo '{"error":"code gates failed on control"}' > "${CONTROL_GATES_FILE}"
else
  echo '{"error":"control plan not found"}' > "${CONTROL_GATES_FILE}"
fi

echo "  running code gates on challenger plan..."
# Look in output/ first (standard), then in challenger dir root (recovered runs).
# Use process substitution to avoid pipefail on missing directories.
CHALLENGER_PLAN=""
if [[ -d "${CHALLENGER_DIR}/output" ]]; then
  CHALLENGER_PLAN="$(find "${CHALLENGER_DIR}/output" -name '*.plan.md' 2>/dev/null | head -1)"
fi
if [[ -z "${CHALLENGER_PLAN}" ]]; then
  CHALLENGER_PLAN="$(find "${CHALLENGER_DIR}" -maxdepth 1 \( -name '*.plan.md' -o -name '*-output-plan.md' \) 2>/dev/null | head -1 || true)"
fi
if [[ -n "${CHALLENGER_PLAN}" ]]; then
  "${CODE_GATES}" "${CHALLENGER_PLAN}" "${REPO_ROOT}" > "${CHALLENGER_GATES_FILE}" 2>/dev/null || \
    echo '{"error":"code gates failed on challenger"}' > "${CHALLENGER_GATES_FILE}"
else
  echo '{"error":"no challenger plan produced"}' > "${CHALLENGER_GATES_FILE}"
fi

echo "  assembling eval report..."
python3 - "${EXP_DIR}" "${CONTROL_GATES_FILE}" "${CHALLENGER_GATES_FILE}" "${JUDGE_DIR}" "${OUTPUT_FILE}" "${CONTROL_PLAN}" <<'PYEOF'
import json, sys, os, datetime

exp_dir = sys.argv[1]
control_gates_file = sys.argv[2]
challenger_gates_file = sys.argv[3]
judge_dir = sys.argv[4]
output_file = sys.argv[5]
control_plan = sys.argv[6]

def load_json(path):
    if os.path.isfile(path):
        with open(path) as f:
            return json.load(f)
    return {}

control_gates = load_json(control_gates_file)
challenger_gates = load_json(challenger_gates_file)
judge_summary = load_json(os.path.join(judge_dir, "judge-summary.json"))
token_signals = load_json(os.path.join(judge_dir, "token-signals.json"))

experiment = judge_summary.get("experiment", os.path.basename(exp_dir))
now = datetime.datetime.utcnow().strftime("%Y-%m-%d %H:%M UTC")

# --- Helper: gate comparison row ---
def gate_row(name, control_val, challenger_val, note=""):
    delta = ""
    try:
        c = int(control_val)
        ch = int(challenger_val)
        if ch > c: delta = f" ▲+{ch-c}"
        elif ch < c: delta = f" ▼-{c-ch}"
        else: delta = " ="
    except Exception:
        pass
    return f"| {name} | {control_val} | {challenger_val}{delta} | {note} |"

# --- Assemble ---
lines = []
lines.append(f"# Eval report — {experiment}")
lines.append(f"")
lines.append(f"**Generated:** {now}")
lines.append(f"**Judge:** {judge_summary.get('judge_model', 'MiniMax-M3')} (blind, position-swapped)")
lines.append(f"**n=1 disclaimer:** this is ONE planning task. Per spec §8, n≥5 required before a binary routing decision.")
lines.append(f"")

# --- 1. Per-dimension divergence ---
lines.append(f"## 1. Per-dimension divergence")
lines.append(f"")
lines.append(f"### 1a. Objective code gates")
lines.append(f"")
lines.append(f"| Gate | Control (Opus) | Challenger (GPT-5.5) | Delta |")
lines.append(f"|---|---|---|---|")
gate_fields = [
    ("sections", "§-sections present (target 20)"),
    ("watchpoint_cites", "watchpoint artefact cites"),
    ("adr015_named", "ADR-015 named"),
    ("adr015_callsite", "ADR-015 callsite"),
    ("task_count", "§13 task count"),
    ("mirror_refs", "MIRROR refs"),
    ("story_signal", "§16a story signals"),
    ("cargo_cmds", "§15 cargo commands"),
    ("t1_preemption_signal", "T1-preemption signals"),
]
for field, label in gate_fields:
    c_val = control_gates.get(field, "n/a")
    ch_val = challenger_gates.get(field, "n/a")
    footgun = ""
    if field == "p_features_footgun" and (c_val or ch_val):
        footgun = "⚠️ -p+--features full footgun"
    lines.append(gate_row(label, c_val, ch_val, footgun))

lines.append(f"")
lines.append(f"### 1b. LLM judge scores (averaged over 2 position-swapped passes)")
lines.append(f"")
c_scores = judge_summary.get("control_scores", [])
ch_scores = judge_summary.get("challenger_scores", [])
dim_names = [
    "1. Completeness",
    "2. Watchpoint specificity",
    "3. ADR preservation",
    "4. Task decomposition",
    "5. Story coverage",
    "6. DoD smoke-test",
    "7. T1-preemption",
]
lines.append(f"| Dimension | Control (Opus) | Challenger (GPT-5.5) | Delta |")
lines.append(f"|---|---|---|---|")
for i, name in enumerate(dim_names):
    cv = c_scores[i] if i < len(c_scores) else "?"
    chv = ch_scores[i] if i < len(ch_scores) else "?"
    try:
        d = float(chv) - float(cv)
        d_str = f"{d:+.1f}" if d != 0 else "="
    except Exception:
        d_str = ""
    lines.append(f"| {name} | {cv} | {chv} | {d_str} |")

c_total = judge_summary.get("control_total")
ch_total = judge_summary.get("challenger_total")
lines.append(f"| **Total (of 35)** | **{c_total}** | **{ch_total}** | |")

# --- 2. Failure-mode → fix map ---
lines.append(f"")
lines.append(f"## 2. Failure-mode → fix map")
lines.append(f"")
lines.append(f"Based on code-gate divergence and judge findings.")
lines.append(f"")

failure_modes = []
if challenger_gates.get("sections", 0) < control_gates.get("sections", 20):
    failure_modes.append({
        "gap": "Missing §-sections",
        "cause": "harness-mismatch — Pi .pi/ prp-plan.md may not enforce the 20-section template as strictly as .claude/commands/prp-core/prp-plan.md",
        "fix": "Add explicit 'Required 20 sections: §1–§16a' enforcement to .pi/prompts/prp-plan.md with a checklist gate before writing the plan"
    })
if challenger_gates.get("adr015_callsite", 0) < 1:
    failure_modes.append({
        "gap": "ADR-015 callsite absent",
        "cause": "brief-ambiguity — ADR constraint named but not made load-bearing (ref: reference_minimax_prompting_best_practices.md #1)",
        "fix": "Add §2.4a ADR-constraint load-bearing clause to the Pi planner prompt: name the specific callsite, state why it can't be deferred, make it a DoD line"
    })
if challenger_gates.get("mirror_refs", 0) < 3:
    failure_modes.append({
        "gap": "Low MIRROR-ref count",
        "cause": "reasoning-gap — Pi harness didn't load the MIRROR-ref discipline from .claude/rules/advisor-orchestrator.md",
        "fix": "Add MIRROR-ref requirement to .pi/prompts/prp-plan.md §13 instructions with examples"
    })
if token_signals.get("run_complete", True) is False:
    failure_modes.append({
        "gap": "Challenger run incomplete",
        "cause": "context-pressure or external kill — run was interrupted mid-generation (turns: {turns}, compactions: {comps})".format(
            turns=token_signals.get("turns", "?"),
            comps=token_signals.get("compaction_count", "?")
        ),
        "fix": "Increase --timeout in run-comparator.sh; investigate why Pi was killed (OOM, daemon cron window, session timeout)"
    })
if token_signals.get("compaction_count", 0) > 0:
    failure_modes.append({
        "gap": "Context compaction triggered",
        "cause": "context-pressure — challenger hit 272K threshold during exploration",
        "fix": "Tune context injection: reduce PROJECT_CONTEXT.md verbosity for planning cells; use --no-context-files for a raw-capability pass to isolate harness vs model quality"
    })

if not failure_modes:
    failure_modes.append({
        "gap": "None identified from code gates",
        "cause": "n/a",
        "fix": "Review judge pass1/pass2 reasoning files for qualitative gaps"
    })

lines.append(f"| Gap | Cause category | Next-run fix |")
lines.append(f"|---|---|---|")
for fm in failure_modes:
    lines.append(f"| {fm['gap']} | `{fm['cause']}` | {fm['fix']} |")

# --- 3. Token/cost breakdown ---
lines.append(f"")
lines.append(f"## 3. Token/cost breakdown")
lines.append(f"")
lines.append(f"| Metric | Control (Opus) | Challenger (GPT-5.5) |")
lines.append(f"|---|---|---|")
lines.append(f"| Run complete | ✅ (ground truth) | {'✅' if token_signals.get('run_complete') else '❌ INTERRUPTED'} |")
lines.append(f"| Wall seconds | n/a (used existing plan) | {token_signals.get('wall_seconds', 'n/a')} |")
lines.append(f"| Turns | n/a | {token_signals.get('turns', 'n/a')} |")
tc = token_signals.get('tool_calls', 'n/a')
tc_str = str(sum(tc.values()) if isinstance(tc, dict) else tc)
lines.append(f"| Tool calls | n/a | {tc_str} |")
lines.append(f"| Input tokens | n/a | {token_signals.get('input_tokens', 'n/a')} |")
lines.append(f"| Output tokens | n/a | {token_signals.get('output_tokens', 'n/a')} |")
lines.append(f"| Compaction events | 0 | {token_signals.get('compaction_count', 0)} |")
lines.append(f"")
lines.append(f"> **Cost note:** control arm runs on Claude Code + Opus subscription (not billed per-token here). Challenger runs via Codex OAuth. True cost comparison requires a common denominator — see spec §4 cost note.")

# --- 4. Replay bundle reference ---
lines.append(f"")
lines.append(f"## 4. Replay-ready bundle")
lines.append(f"")
lines.append(f"Challenger replay bundle at: `{os.path.relpath(exp_dir + '/challenger/replay-bundle', start=exp_dir + '/../..')}`")
lines.append(f"")
lines.append(f"Contents:")
replay_dir = os.path.join(exp_dir, "challenger", "replay-bundle")
if os.path.isdir(replay_dir):
    for fname in sorted(os.listdir(replay_dir)):
        lines.append(f"- `{fname}`")
else:
    lines.append(f"- *(replay-bundle dir not found)*")

# --- Routing recommendation ---
lines.append(f"")
lines.append(f"## Routing recommendation")
lines.append(f"")
routing = judge_summary.get("routing_consensus", "EXTEND_5")
c_total_str = str(c_total) if c_total is not None else "?"
ch_total_str = str(ch_total) if ch_total is not None else "?"
lines.append(f"**Outcome:** `{routing}`")
lines.append(f"")
lines.append(f"Control total: {c_total_str}/35 | Challenger total: {ch_total_str}/35")
lines.append(f"")
lines.append(f"*n=1 — this is a single data point. Extend to n≥5 paired tasks before a production routing flip (spec §8).*")

with open(output_file, 'w', encoding='utf-8') as f:
    f.write('\n'.join(lines) + '\n')
print(f"eval-report.md written: {output_file}")
PYEOF

echo "eval-report complete: ${OUTPUT_FILE}"
