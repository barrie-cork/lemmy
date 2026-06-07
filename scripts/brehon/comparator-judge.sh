#!/usr/bin/env bash
# comparator-judge.sh — run the MiniMax M3 blind LLM judge over two plan arms.
#
# Per .claude/PRPs/specs/pi-model-comparator.spec.md §7 + eval best-practices:
# - Judge is a THIRD model (neither arm) — MiniMax M3 here.
# - Blind: plans labelled A/B, identity NOT revealed until after scoring.
# - Position-swapped: run A-then-B AND B-then-A; average to kill ordering bias.
# - Reasons-then-scores: judge emits reasoning_content BEFORE the score.
# - Uses minimax-api.sh (raw curl, not Pi) for deterministic JSON output.
#
# Usage:
#   comparator-judge.sh \
#     --experiment <id>                  e.g. planning-001
#     --control-plan <path>              Opus ground-truth plan file
#     --challenger-dir <dir>             .claude/PRPs/comparator/runs/<exp>/challenger/
#     [--model <minimax-model-id>]       default: MiniMax-M3
#     [--max-tokens <n>]                 default: 16384
#     [--results-dir <dir>]              default: .claude/PRPs/comparator/runs/<exp>/judge/
#     [--env-file <path>]                default: .env in CWD
#     [--dry-run]                        print prompts but don't call the API
#
# Output (in results-dir/):
#   pass1-A-then-B.json      raw API response, control=A challenger=B
#   pass1-A-then-B.reason    reasoning_content from pass1
#   pass2-B-then-A.json      raw API response, control=B challenger=A
#   pass2-B-then-A.reason    reasoning_content from pass2
#   token-signals.json       comparator-token-extract output for the challenger
#   judge-summary.json       averaged scores + routing recommendation
#   judge-report.md          human-readable report (the per-dimension divergence artefact)

set -euo pipefail

EXPERIMENT=""
CONTROL_PLAN=""
CHALLENGER_DIR=""
MODEL="MiniMax-M3"
MAX_TOKENS=16384
RESULTS_DIR=""
ENV_FILE=""
DRY_RUN=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --experiment)     EXPERIMENT="$2";     shift 2;;
    --control-plan)   CONTROL_PLAN="$2";   shift 2;;
    --challenger-dir) CHALLENGER_DIR="$2"; shift 2;;
    --model)          MODEL="$2";          shift 2;;
    --max-tokens)     MAX_TOKENS="$2";     shift 2;;
    --results-dir)    RESULTS_DIR="$2";    shift 2;;
    --env-file)       ENV_FILE="$2";       shift 2;;
    --dry-run)        DRY_RUN=true;        shift;;
    -h|--help)        sed -n '1,25p' "$0"; exit 0;;
    *) echo "ERROR: unknown arg: $1" >&2; exit 2;;
  esac
done

for v in EXPERIMENT CONTROL_PLAN CHALLENGER_DIR; do
  if [[ -z "${!v}" ]]; then
    echo "ERROR: --${v,,} is required" >&2; exit 2
  fi
done

if [[ ! -f "${CONTROL_PLAN}" ]]; then
  echo "ERROR: control plan not found: ${CONTROL_PLAN}" >&2; exit 2
fi

TRACE="${CHALLENGER_DIR}/trace.jsonl"
TRACE_OPTIONAL=false
if [[ ! -f "${TRACE}" ]]; then
  # Allow judge to run without trace if token-signals.json already exists.
  if [[ -n "${RESULTS_DIR}" && -f "${RESULTS_DIR}/token-signals.json" ]]; then
    echo "  WARN: trace.jsonl not found locally; using pre-computed token-signals.json"
    TRACE_OPTIONAL=true
  else
    echo "ERROR: challenger trace not found: ${TRACE}" >&2
    echo "  hint: copy trace.jsonl from the runner host, or pre-generate token-signals.json" >&2
    exit 2
  fi
fi

# Check if challenger produced a plan file.
# Look first in output/ (standard), then directly in the challenger dir (recovered run).
CHALLENGER_PLAN=""
if [[ -d "${CHALLENGER_DIR}/output" ]]; then
  CHALLENGER_PLAN="$(find "${CHALLENGER_DIR}/output" -name '*.plan.md' | head -1)"
fi
if [[ -z "${CHALLENGER_PLAN}" ]]; then
  # Fallback: check for plan directly in challenger dir (recovered/interrupted runs)
  CHALLENGER_PLAN="$(find "${CHALLENGER_DIR}" -maxdepth 1 -name '*.plan.md' | head -1)"
fi

# Locate this script's directory to call siblings.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MINIMAX_API="${SCRIPT_DIR}/minimax-api.sh"
TOKEN_EXTRACT="${SCRIPT_DIR}/comparator-token-extract.sh"
JUDGE_PROMPT_TEMPLATE=""
# Find the judge prompt template from the repo .pi/prompts/.
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
JUDGE_PROMPT_TEMPLATE="${REPO_ROOT}/.pi/prompts/judge-planning.md"

if [[ ! -x "${MINIMAX_API}" ]]; then
  echo "ERROR: minimax-api.sh not executable: ${MINIMAX_API}" >&2; exit 1
fi

if [[ ! -f "${JUDGE_PROMPT_TEMPLATE}" ]]; then
  echo "ERROR: judge prompt template not found: ${JUDGE_PROMPT_TEMPLATE}" >&2; exit 1
fi

# Resolve results dir.
if [[ -z "${RESULTS_DIR}" ]]; then
  RESULTS_DIR="${REPO_ROOT}/.claude/PRPs/comparator/runs/${EXPERIMENT}/judge"
fi
mkdir -p "${RESULTS_DIR}"

echo "comparator-judge:"
echo "  experiment:    ${EXPERIMENT}"
echo "  control-plan:  ${CONTROL_PLAN}"
echo "  challenger:    ${CHALLENGER_DIR}"
echo "  plan-found:    ${CHALLENGER_PLAN:-NONE (run incomplete)}"
echo "  judge model:   ${MODEL}"
echo "  results:       ${RESULTS_DIR}"

# Extract token signals from the challenger trace (skip if pre-computed).
if [[ "${TRACE_OPTIONAL}" == false ]]; then
  echo "  extracting token signals from trace..."
  META_FILE="${CHALLENGER_DIR}/meta.json"
  META_FLAG=""
  [[ -f "${META_FILE}" ]] && META_FLAG="--meta ${META_FILE}"
  "${TOKEN_EXTRACT}" "${TRACE}" ${META_FLAG} > "${RESULTS_DIR}/token-signals.json"
else
  echo "  using pre-computed token-signals.json (no local trace)"
fi

# If no challenger plan was produced (run incomplete or failed), we grade the
# challenger as a partial run. The judge still scores what exists — if there's
# no plan file, we use a sentinel note so the judge can score it.
if [[ -z "${CHALLENGER_PLAN}" ]]; then
  echo "  WARN: no challenger plan file found; using partial-run sentinel for grading"
  CHALLENGER_PLAN_CONTENT="[NO PLAN PRODUCED — challenger run was interrupted before writing any plan output. Score dimension 1 (completeness) as 1 (absent). For all other dimensions, infer from the exploration trace summary if available, otherwise score 1.]"
else
  CHALLENGER_PLAN_CONTENT="$(cat "${CHALLENGER_PLAN}")"
fi

CONTROL_PLAN_CONTENT="$(cat "${CONTROL_PLAN}")"
TOKEN_SIGNALS="$(cat "${RESULTS_DIR}/token-signals.json")"

# Build the judge input (two passes for position-swapping).
build_judge_input() {
  local arm_A_label="$1"  # "control" or "challenger"
  local arm_A_content="$2"
  local arm_B_label="$3"
  local arm_B_content="$4"

  # The judge sees A/B labels, NOT the arm identities.
  python3 - <<PYEOF
import json, sys

a_label = '${arm_A_label}'
b_label = '${arm_B_label}'
a_content = ${1}  # positional substitution won't work; done below
PYEOF
  # Use a temp file to avoid quoting hell with large plan content.
  local tmp
  tmp="$(mktemp)"
  python3 - "${arm_A_content}" "${arm_B_content}" "${TOKEN_SIGNALS}" > "${tmp}" <<'PYEOF'
import json, sys

a_content = open(sys.argv[1]).read() if sys.argv[1] != 'SENTINEL' else sys.argv[1]
# argv[1] is the content string itself for inline sentinels
PYEOF
  # Simpler approach: write files and compose.
  local tmp_a tmp_b
  tmp_a="$(mktemp)"
  tmp_b="$(mktemp)"
  printf '%s' "${arm_A_content}" > "${tmp_a}"
  printf '%s' "${arm_B_content}" > "${tmp_b}"
  python3 - "${tmp_a}" "${tmp_b}" "${TOKEN_SIGNALS}" <<'PYEOF'
import json, sys

with open(sys.argv[1]) as f: plan_a = f.read()
with open(sys.argv[2]) as f: plan_b = f.read()
token_signals = json.loads(sys.argv[3])

result = {
    "plan_A": plan_a,
    "plan_B": plan_b,
    "metadata": {
        "experiment": "planning-001",
        "challenger_trace_signals": token_signals,
        "grading_note": (
            "Arm A and Arm B are anonymous — you do not know which AI system "
            "produced each plan. Score purely on plan text per the rubric."
        )
    }
}
print(json.dumps(result, indent=2))
PYEOF
  rm -f "${tmp_a}" "${tmp_b}"
}

run_judge_pass() {
  local pass_name="$1"
  local plan_A_content="$2"
  local plan_B_content="$3"
  local output_json="${RESULTS_DIR}/${pass_name}.json"
  local output_reason="${RESULTS_DIR}/${pass_name}.reason"

  echo "  running judge pass: ${pass_name}..."

  # Write plan files to temp for python composition
  local tmp_a tmp_b
  tmp_a="$(mktemp)"
  tmp_b="$(mktemp)"
  printf '%s' "${plan_A_content}" > "${tmp_a}"
  printf '%s' "${plan_B_content}" > "${tmp_b}"

  # Read the judge prompt template (up to the <plans> tag, then inject, then task).
  # The template uses $ARGUMENTS. We replace it with the judge input JSON path.
  local judge_input
  judge_input="$(python3 - "${tmp_a}" "${tmp_b}" "${TOKEN_SIGNALS}" <<'PYEOF'
import json, sys
with open(sys.argv[1]) as f: plan_a = f.read()
with open(sys.argv[2]) as f: plan_b = f.read()
token_signals_raw = sys.argv[3]
try:
    token_signals = json.loads(token_signals_raw)
except:
    token_signals = {}
block = {
    "plan_A": plan_a,
    "plan_B": plan_b,
    "challenger_trace_signals": token_signals,
    "grading_note": (
        "Arm A and Arm B are anonymous — you do not know which AI system "
        "produced each plan. Score purely on plan text per the rubric."
    )
}
# Compact JSON (large plans) — the judge sees the full text
print(json.dumps(block))
PYEOF
)"

  rm -f "${tmp_a}" "${tmp_b}"

  # Build the full judge message: template body with $ARGUMENTS replaced.
  local template_body
  template_body="$(cat "${JUDGE_PROMPT_TEMPLATE}")"
  # Strip YAML frontmatter (lines between first and second ---)
  local message
  message="$(printf '%s' "${template_body}" | python3 -c "
import sys
content = sys.stdin.read()
# Strip YAML frontmatter
if content.startswith('---'):
    end = content.find('\n---', 3)
    if end != -1:
        content = content[end+4:].lstrip()
# Replace \$ARGUMENTS with the judge input
import json
args_json = json.loads($(printf '%s' "${judge_input}" | python3 -c "import sys,json; print(repr(sys.stdin.read()))"))
content = content.replace('\$ARGUMENTS', json.dumps(args_json))
print(content)
")"

  if [[ "${DRY_RUN}" == true ]]; then
    echo "  [dry-run] would send judge message (${#message} chars) to ${MODEL}"
    echo '{"dry_run": true}' > "${output_json}"
    echo "[dry-run]" > "${output_reason}"
    return 0
  fi

  # Write message to temp file.
  local tmp_msg
  tmp_msg="$(mktemp)"
  printf '%s' "${message}" > "${tmp_msg}"

  local ENV_FLAG=""
  [[ -n "${ENV_FILE}" ]] && ENV_FLAG="--env-file ${ENV_FILE}"

  "${MINIMAX_API}" \
    --model "${MODEL}" \
    --max-tokens "${MAX_TOKENS}" \
    --temperature "0.0" \
    --message "$(cat "${tmp_msg}")" \
    --output "${output_json}" \
    --reason-file "${output_reason}" \
    ${ENV_FLAG}

  rm -f "${tmp_msg}"
  echo "  judge pass ${pass_name} complete: ${output_json}"
}

# Pass 1: control=A, challenger=B
run_judge_pass "pass1-A-control-B-challenger" "${CONTROL_PLAN_CONTENT}" "${CHALLENGER_PLAN_CONTENT}"

# Pass 2: challenger=A, control=B (position-swapped)
run_judge_pass "pass2-A-challenger-B-control" "${CHALLENGER_PLAN_CONTENT}" "${CONTROL_PLAN_CONTENT}"

# Synthesise averaged scores + routing recommendation.
echo "  synthesising judge summary..."
python3 - "${RESULTS_DIR}" "${EXPERIMENT}" <<'PYEOF'
import json, sys, os, re

results_dir = sys.argv[1]
experiment = sys.argv[2]

pass1_file = os.path.join(results_dir, "pass1-A-control-B-challenger.json")
pass2_file = os.path.join(results_dir, "pass2-A-challenger-B-control.json")

def extract_answer(json_file):
    if not os.path.isfile(json_file):
        return None
    with open(json_file) as f:
        d = json.load(f)
    if d.get('dry_run'):
        return "[dry-run]"
    choices = d.get('choices', [])
    if not choices:
        return None
    return choices[0].get('message', {}).get('content', '')

pass1_answer = extract_answer(pass1_file) or ""
pass2_answer = extract_answer(pass2_file) or ""

# Extract dimension scores from each pass using regex.
# Pattern: <score>N</score> (1-5)
def extract_scores(text):
    # Find all <score>N</score> patterns in order.
    return [int(m) for m in re.findall(r'<score>(\d)</score>', text)]

def extract_routing(text):
    m = re.search(r'OUTCOME:\s*([A-Z_]+)', text)
    return m.group(1) if m else "UNKNOWN"

# Pass 1: A=control B=challenger
# Scores come in order: dim1_A, dim1_B, dim2_A, dim2_B, ...
# Per template structure: Arm A scores + Arm B scores alternating per dimension.
pass1_scores = extract_scores(pass1_answer)
pass2_scores = extract_scores(pass2_answer)

# In pass1: A=control, B=challenger
# In pass2: A=challenger, B=control
# Each pass produces 2 scores per dimension (14 scores for 7 judged dimensions).
# Dimension 8 (cost) is not scored 1-5.
# Extract control/challenger scores from each pass:
def split_ab(scores):
    # Alternating A, B per dimension
    a_scores = scores[0::2]
    b_scores = scores[1::2]
    return a_scores, b_scores

p1_A, p1_B = split_ab(pass1_scores)   # p1_A=control, p1_B=challenger
p2_A, p2_B = split_ab(pass2_scores)   # p2_A=challenger, p2_B=control

# Average control scores: p1_A + p2_B; challenger: p1_B + p2_A
def avg(a, b):
    pairs = zip(a, b)
    return [round((x + y) / 2, 1) for x, y in pairs]

control_scores = avg(p1_A, p2_B) if p1_A and p2_B else p1_A or p2_B or []
challenger_scores = avg(p1_B, p2_A) if p1_B and p2_A else p1_B or p2_A or []

routing1 = extract_routing(pass1_answer)
routing2 = extract_routing(pass2_answer)

token_signals = {}
ts_file = os.path.join(results_dir, "token-signals.json")
if os.path.isfile(ts_file):
    with open(ts_file) as f:
        token_signals = json.load(f)

summary = {
    "experiment": experiment,
    "judge_model": "MiniMax-M3",
    "passes": 2,
    "control_scores": control_scores,
    "challenger_scores": challenger_scores,
    "control_total": sum(control_scores) if control_scores else None,
    "challenger_total": sum(challenger_scores) if challenger_scores else None,
    "routing_pass1": routing1,
    "routing_pass2": routing2,
    "routing_consensus": routing1 if routing1 == routing2 else "MIXED_ROUTING",
    "challenger_run_complete": token_signals.get("run_complete", False),
    "challenger_compactions": token_signals.get("compaction_count", 0),
    "challenger_turns": token_signals.get("turns", 0),
    "challenger_tool_calls": token_signals.get("tool_calls", {}),
    "note": (
        "n=1 — one planning task. Per spec §8, n≥5 required for a binary "
        "routing decision. This is a data point, not a verdict."
    )
}
out_file = os.path.join(results_dir, "judge-summary.json")
with open(out_file, 'w') as f:
    json.dump(summary, f, indent=2)
print(f"  judge-summary.json written: {out_file}")
print(json.dumps(summary, indent=2))
PYEOF

echo "judge complete: ${RESULTS_DIR}"
