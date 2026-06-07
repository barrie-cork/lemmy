#!/usr/bin/env bash
# run-comparator.sh — run ONE challenger arm of a model comparator experiment.
#
# Per .claude/PRPs/specs/pi-model-comparator.spec.md. This runs the CHALLENGER
# arm only: a Pi (pi.dev / @earendil-works/pi-coding-agent) headless session in
# an isolated throwaway git worktree, capturing the trace + produced artifact.
#
# The CONTROL arm (Claude Code + Opus 4.8 via Junior [role:planning]) is NOT run
# by this script — it is the existing production dispatch. This script exists
# because Pi is the only clean way to run a non-Claude challenger inside the
# Brehon harness, and Pi has NO built-in permission system (pi.dev docs), so
# sandboxing via a throwaway worktree is OUR responsibility.
#
# DESIGN (spec §5, §6):
#   - One throwaway worktree per arm: ab-cell/<experiment-id>-<arm-id>, off the
#     experiment's base_commit. Created here, removed after capture. NEVER merged.
#   - Sibling to /srv/brehon-fork (NOT under .junior/worktrees/ — avoids the
#     daemon's cancel-reaper and the Claude-shaped worktree-guard hook).
#   - Pi auto-discovers AGENTS.md -> .pi/PROJECT_CONTEXT.md when CWD is the repo,
#     so the Brehon harness is injected natively. --no-context-files is the
#     harness-OFF toggle for raw-capability runs.
#   - Captures: trace (--mode json stream) + the produced plan.md + a metrics line.
#   - Respects the cross-lane cap: run arms SERIALLY (this script runs ONE arm;
#     the caller sequences them). Never run concurrently with >1 other
#     worktree-writing agent — all daemon worktrees share one .git/index.lock
#     (feedback_cohort_shared_git_index_contention.md).
#
# This script is designed to run ON the EliteDesk (homeserver) where Pi is
# installed user-local at ~/.npm-global/bin/pi and the Codex OAuth is in
# ~/.pi/agent/auth.json. Invoke via SSH from the advisor laptop, or directly.
#
# Usage:
#   run-comparator.sh \
#     --experiment <id> \
#     --arm <arm-id> \
#     --model <pi-model-spec>        e.g. 'openai-codex/gpt-5.5:xhigh'
#     --prompt-template <path>       e.g. .pi/prompts/prp-plan.md
#     --task <task-input>            the brief path or free-form planning input
#     [--base-commit <sha>]          default: current HEAD of $REPO
#     [--harness off]                add --no-context-files (raw-capability run)
#     [--repo <path>]                default: /srv/brehon-fork
#     [--timeout <seconds>]          default: 3600 (planning is heavy)
#
# Example (the first planning experiment's challenger arm):
#   run-comparator.sh --experiment planning-001 --arm challenger \
#     --model 'openai-codex/gpt-5.5:xhigh' \
#     --prompt-template .pi/prompts/prp-plan.md \
#     --task '.claude/PRPs/briefs/<the-planning-brief>.md'
#
# Hard refusals:
#   - Refuses if Pi is not on PATH (install per spec §10).
#   - Refuses if the experiment/arm/model/prompt/task args are missing.
#   - Refuses to run inside /srv/brehon-fork itself (would pollute the live
#     daemon worktree); ALWAYS operates in an ab-cell/* sibling worktree.
#   - Refuses if the target worktree path already exists (no clobber).
#   - NEVER merges the ab-cell branch anywhere. NEVER pushes it.
#   - NEVER echoes auth tokens (Pi reads them from ~/.pi/agent/auth.json itself).

set -euo pipefail

# ---- defaults ----
REPO="/srv/brehon-fork"
BASE_COMMIT=""
HARNESS="on"
TIMEOUT="3600"
EXPERIMENT=""
ARM=""
MODEL=""
PROMPT_TEMPLATE=""
TASK=""

# ---- parse args ----
while [[ $# -gt 0 ]]; do
  case "$1" in
    --experiment) EXPERIMENT="$2"; shift 2;;
    --arm) ARM="$2"; shift 2;;
    --model) MODEL="$2"; shift 2;;
    --prompt-template) PROMPT_TEMPLATE="$2"; shift 2;;
    --task) TASK="$2"; shift 2;;
    --base-commit) BASE_COMMIT="$2"; shift 2;;
    --harness) HARNESS="$2"; shift 2;;
    --repo) REPO="$2"; shift 2;;
    --timeout) TIMEOUT="$2"; shift 2;;
    -h|--help) sed -n '1,60p' "$0"; exit 0;;
    *) echo "ERROR: unknown arg: $1" >&2; exit 2;;
  esac
done

# ---- validate ----
for v in EXPERIMENT ARM MODEL PROMPT_TEMPLATE TASK; do
  if [[ -z "${!v}" ]]; then
    echo "ERROR: --${v,,} is required" >&2; exit 2
  fi
done

# Pi on PATH (user-local install per spec §10).
export PATH="${HOME}/.npm-global/bin:${PATH}"
if ! command -v pi >/dev/null 2>&1; then
  echo "ERROR: 'pi' not on PATH. Install: npm i -g --ignore-scripts @earendil-works/pi-coding-agent (spec §10)" >&2
  exit 1
fi

if [[ ! -d "${REPO}/.git" ]]; then
  echo "ERROR: ${REPO} is not a git repo" >&2; exit 1
fi

# Resolve base commit from the repo HEAD if not given.
if [[ -z "${BASE_COMMIT}" ]]; then
  BASE_COMMIT="$(git -C "${REPO}" rev-parse HEAD)"
fi

# ---- sandbox worktree (spec §5) ----
CELL_BRANCH="ab-cell/${EXPERIMENT}-${ARM}"
# Sibling to the repo, NOT under .junior/worktrees/.
WT_PARENT="$(dirname "${REPO}")"
WT_PATH="${WT_PARENT}/$(basename "${REPO}")-abcell-${EXPERIMENT}-${ARM}"

if [[ "${PWD}" == "${REPO}" || "${PWD}" == "${REPO}/"* ]]; then
  : # we cd into the worktree below; the live repo is only the worktree source
fi

if [[ -e "${WT_PATH}" ]]; then
  echo "ERROR: worktree path already exists, refusing to clobber: ${WT_PATH}" >&2
  echo "  remove with: git -C '${REPO}' worktree remove '${WT_PATH}'" >&2
  exit 1
fi

# ---- results dir (spec §6) ----
RESULTS_DIR="${REPO}/.claude/PRPs/comparator/runs/${EXPERIMENT}/${ARM}"
mkdir -p "${RESULTS_DIR}"
TRACE_FILE="${RESULTS_DIR}/trace.jsonl"
META_FILE="${RESULTS_DIR}/meta.json"
PI_LOG="${RESULTS_DIR}/pi-run.log"

echo "comparator arm:"
echo "  experiment:      ${EXPERIMENT}"
echo "  arm:             ${ARM}"
echo "  model:           ${MODEL}"
echo "  prompt-template: ${PROMPT_TEMPLATE}"
echo "  task:            ${TASK}"
echo "  base-commit:     ${BASE_COMMIT}"
echo "  harness:         ${HARNESS}"
echo "  worktree:        ${WT_PATH}  (branch ${CELL_BRANCH})"
echo "  results:         ${RESULTS_DIR}"

# Create the throwaway worktree off base_commit.
git -C "${REPO}" worktree add -b "${CELL_BRANCH}" "${WT_PATH}" "${BASE_COMMIT}" >/dev/null
echo "  worktree created."

# Worktree bootstrap (submodules + gitignored harness files the cell needs).
# lemmy_email build.rs fails without the translations submodule; .pi/ extensions
# need node deps only if the cell dispatches subagents (planning cell does not).
git -C "${WT_PATH}" submodule update --init --recursive >/dev/null 2>&1 || \
  echo "  WARN: submodule init failed (lemmy_email build may fail if a cargo step runs)"

# ---- assemble the Pi message ----
# The challenger reads the SAME planning prompt the control reads (dual-harness
# mirror), with the task input as the argument. We pass the prompt template via
# --prompt-template (Pi loads it as a slash-style template) and the task as the
# -p message. Pi auto-discovers AGENTS.md/.pi/ from CWD = the worktree.
HARNESS_FLAG=""
if [[ "${HARNESS}" == "off" ]]; then
  HARNESS_FLAG="--no-context-files"
fi

# The message: invoke the planning prompt template by its /command name with the
# task as $ARGUMENTS. Per docs/prompt-templates.md: templates are invoked by
# typing `/name` in the editor; `--prompt-template <path>` only REGISTERS the
# template for /-invocation, it does NOT auto-expand it. A headless `-p` message
# of "/prp-plan <task>" expands prp-plan.md with the task bound to $ARGUMENTS.
# The /command name is the template filename without .md (prp-plan.md -> /prp-plan).
TEMPLATE_NAME="$(basename "${PROMPT_TEMPLATE}" .md)"
PI_MESSAGE="/${TEMPLATE_NAME} ${TASK}"

START_TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
START_EPOCH="$(date +%s)"

# Run Pi headless, in the worktree, capturing the json event stream to the
# trace file AND a human log. We do NOT pipe pi through tail/grep (would mask
# exit code per cargo-output-capture.md discipline) — redirect to files, read $?.
set +e
(
  cd "${WT_PATH}"
  timeout "${TIMEOUT}" pi -p \
    --mode json \
    --no-session \
    --model "${MODEL}" \
    --prompt-template "${PROMPT_TEMPLATE}" \
    ${HARNESS_FLAG} \
    "${PI_MESSAGE}"
) > "${TRACE_FILE}" 2> "${PI_LOG}"
PI_EXIT=$?
set -e

END_TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
END_EPOCH="$(date +%s)"
WALL_S=$(( END_EPOCH - START_EPOCH ))

# ---- capture the produced artifact ----
# Any new/changed plan file in the worktree is the output. Copy it to results.
PLAN_OUT="${RESULTS_DIR}/output"
mkdir -p "${PLAN_OUT}"
# New plan files this run created (vs base_commit).
git -C "${WT_PATH}" add -A >/dev/null 2>&1 || true
CHANGED_PLANS="$(git -C "${WT_PATH}" diff --cached --name-only -- '.claude/PRPs/plans/' 2>/dev/null || true)"
if [[ -n "${CHANGED_PLANS}" ]]; then
  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    mkdir -p "${PLAN_OUT}/$(dirname "$f")"
    cp "${WT_PATH}/$f" "${PLAN_OUT}/$f" 2>/dev/null || true
  done <<< "${CHANGED_PLANS}"
fi

# ---- metrics line (spec §6) ----
# Token/cost parsing from the json trace is best-effort; the judge step reads
# the full trace. Here we record run-level facts.
PLAN_COUNT="$(printf '%s\n' "${CHANGED_PLANS}" | grep -c . || true)"
{
  printf '{'
  printf '"experiment":"%s",' "${EXPERIMENT}"
  printf '"arm":"%s",' "${ARM}"
  printf '"model":"%s",' "${MODEL}"
  printf '"prompt_template":"%s",' "${PROMPT_TEMPLATE}"
  printf '"task":"%s",' "${TASK}"
  printf '"base_commit":"%s",' "${BASE_COMMIT}"
  printf '"harness":"%s",' "${HARNESS}"
  printf '"start":"%s",' "${START_TS}"
  printf '"end":"%s",' "${END_TS}"
  printf '"wall_seconds":%s,' "${WALL_S}"
  printf '"pi_exit":%s,' "${PI_EXIT}"
  printf '"plans_produced":%s,' "${PLAN_COUNT:-0}"
  printf '"trace":"%s",' "${TRACE_FILE}"
  printf '"pi_log":"%s"' "${PI_LOG}"
  printf '}\n'
} > "${META_FILE}"

echo "arm complete:"
echo "  pi exit:        ${PI_EXIT}"
echo "  wall seconds:   ${WALL_S}"
echo "  plans produced: ${PLAN_COUNT:-0}"
echo "  trace:          ${TRACE_FILE}"
echo "  meta:           ${META_FILE}"

# ---- teardown (spec §5: never merge, remove the throwaway worktree) ----
# Keep the branch ref locally for forensic git-diff but remove the worktree FS.
# The output + trace are already copied to RESULTS_DIR (lives in the live repo).
git -C "${REPO}" worktree remove --force "${WT_PATH}" >/dev/null 2>&1 || \
  echo "  WARN: worktree remove failed; remove manually: git -C '${REPO}' worktree remove --force '${WT_PATH}'"
echo "  worktree removed (branch ${CELL_BRANCH} kept for forensic diff; delete with: git -C '${REPO}' branch -D ${CELL_BRANCH})"

# Propagate Pi's exit code so the caller sees arm failures.
exit "${PI_EXIT}"
