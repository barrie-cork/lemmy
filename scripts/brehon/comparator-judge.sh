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
  # Fallback: check for any markdown plan in challenger dir root (recovered/interrupted runs).
  # Matches both *.plan.md and *-output-plan.md naming conventions.
  CHALLENGER_PLAN="$(find "${CHALLENGER_DIR}" -maxdepth 1 -name '*.plan.md' -o -maxdepth 1 -name '*-output-plan.md' 2>/dev/null | head -1)"
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

# Write plan content to temp files (avoid ARG_MAX on large plans passed to python/shell).
TMP_CONTROL_PLAN="$(mktemp)"
TMP_CHALLENGER_PLAN="$(mktemp)"

cat "${CONTROL_PLAN}" > "${TMP_CONTROL_PLAN}"

if [[ -z "${CHALLENGER_PLAN}" ]]; then
  echo "  WARN: no challenger plan file found; using partial-run sentinel for grading"
  printf '%s' "[NO PLAN PRODUCED — challenger run was interrupted before writing any plan output. Score dimension 1 (completeness) as 1 (absent). For all other dimensions, infer from the exploration trace summary if available, otherwise score 1.]" > "${TMP_CHALLENGER_PLAN}"
else
  cat "${CHALLENGER_PLAN}" > "${TMP_CHALLENGER_PLAN}"
fi

# Register cleanup.
trap 'rm -f "${TMP_CONTROL_PLAN}" "${TMP_CHALLENGER_PLAN}"' EXIT

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
  local plan_A_file="$2"   # path to file containing plan A text
  local plan_B_file="$3"   # path to file containing plan B text
  local output_json="${RESULTS_DIR}/${pass_name}.json"
  local output_reason="${RESULTS_DIR}/${pass_name}.reason"

  echo "  running judge pass: ${pass_name}..."

  # Build the full judge message using Python (all I/O via files, no ARG_MAX risk).
  # Reads: plan_A_file, plan_B_file, token_signals file, judge prompt template.
  # Writes: tmp_msg file containing the full judge prompt text.
  local tmp_msg tmp_signals
  tmp_msg="$(mktemp)"
  tmp_signals="${RESULTS_DIR}/token-signals.json"

  python3 - "${plan_A_file}" "${plan_B_file}" "${tmp_signals}" "${JUDGE_PROMPT_TEMPLATE}" "${tmp_msg}" <<'PYEOF'
import json, sys, re

plan_A_path, plan_B_path, signals_path, template_path, out_path = sys.argv[1:]

def digest_plan(text, max_chars=10000):
    """Extract the high-signal sections of a plan.
    Keeps: §1 goal/sub-phase, §4 watchpoints, §13 tasks (headings+DoD only),
    §15 DoD commands, §16a stories. Strips filler prose to fit in ~10k chars.
    This reduces the per-plan budget from ~50KB to ~10KB without losing scoreable signal.
    """
    # Split on ## headers
    parts = re.split(r'(?=^## )', text, flags=re.M)
    keep_patterns = [
        r'^## (1\.|Sub-phase|Goal|Overview)',
        r'^## 4\.',   # watchpoints
        r'^## 5\.',   # complexity
        r'^## (13\.|Tasks?)',
        r'^## (15\.|DoD|Definition)',
        r'^## (16a?\.|\[Story)',
    ]
    kept = []
    for part in parts:
        header = part.split('\n')[0]
        if any(re.search(p, header, re.I) for p in keep_patterns):
            kept.append(part)
        elif not kept:  # always keep the intro (before first ##)
            kept.append(part)
    digest = '\n'.join(kept)
    if len(digest) > max_chars:
        digest = digest[:max_chars] + '\n[... truncated for judge token budget ...]'
    return digest

with open(plan_A_path, encoding='utf-8', errors='replace') as f:
    plan_a_raw = f.read()
with open(plan_B_path, encoding='utf-8', errors='replace') as f:
    plan_b_raw = f.read()

plan_a = digest_plan(plan_a_raw)
plan_b = digest_plan(plan_b_raw)

# Emit digest sizes to stdout so --dry-run can report them before the API call.
sys.stderr.write(f"  digest: control {len(plan_a_raw)}->{len(plan_a)} chars, challenger {len(plan_b_raw)}->{len(plan_b)} chars\n")
sys.stderr.flush()

try:
    with open(signals_path) as f:
        token_signals = json.load(f)
except Exception:
    token_signals = {}
with open(template_path, encoding='utf-8', errors='replace') as f:
    template = f.read()

# Strip YAML frontmatter
if template.startswith('---'):
    end = template.find('\n---', 3)
    if end != -1:
        template = template[end+4:].lstrip()

# Build the arguments block (embedded in the template via $ARGUMENTS)
args_block = {
    "plan_A": plan_a,
    "plan_B": plan_b,
    "challenger_trace_signals": token_signals,
    "grading_note": (
        "Arm A and Arm B are anonymous — you do not know which AI system "
        "produced each plan. Score purely on plan text per the rubric. "
        "Plans are digested to key sections (goal, watchpoints, tasks, DoD, stories)."
    )
}
args_json = json.dumps(args_block)

# Replace $ARGUMENTS placeholder in the template
message = template.replace('$ARGUMENTS', args_json)

with open(out_path, 'w', encoding='utf-8') as f:
    f.write(message)
PYEOF

  local msg_size
  msg_size="$(wc -c < "${tmp_msg}")"
  echo "  message size: ${msg_size} bytes"

  if [[ "${DRY_RUN}" == true ]]; then
    echo "  [dry-run] would send judge message (${msg_size} bytes) to ${MODEL}"
    echo '{"dry_run": true}' > "${output_json}"
    echo "[dry-run]" > "${output_reason}"
    rm -f "${tmp_msg}"
    return 0
  fi

  local ENV_FLAG=""
  [[ -n "${ENV_FILE}" ]] && ENV_FLAG="--env-file ${ENV_FILE}"

  # minimax-api.sh --message-file reads the message from a file (no ARG_MAX)
  "${MINIMAX_API}" \
    --model "${MODEL}" \
    --max-tokens "${MAX_TOKENS}" \
    --temperature "0.0" \
    --message-file "${tmp_msg}" \
    --output "${output_json}" \
    --reason-file "${output_reason}" \
    ${ENV_FLAG}

  rm -f "${tmp_msg}"
  echo "  judge pass ${pass_name} complete: ${output_json}"
}

# Pass 1: control=A, challenger=B
run_judge_pass "pass1-A-control-B-challenger" "${TMP_CONTROL_PLAN}" "${TMP_CHALLENGER_PLAN}"

# Pass 2: challenger=A, control=B (position-swapped)
run_judge_pass "pass2-A-challenger-B-control" "${TMP_CHALLENGER_PLAN}" "${TMP_CONTROL_PLAN}"

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
    # Match OUTCOME: CONTROL_WINS or OUTCOME: **CONTROL_WINS** (markdown bold)
    m = re.search(r'OUTCOME:\s*\*{0,2}([A-Z_]+)\*{0,2}', text)
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
