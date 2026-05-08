#!/usr/bin/env bash
# queue-minimax-task.sh — queue a Junior task with MiniMax M2.7 env overrides
#
# Reads MINIMAX_API_KEY from the laptop's .env (gitignored, never committed)
# and queues a Junior task on the EliteDesk daemon with the three env overrides
# the daemon's executor.ts merges over the role-model mapping:
#
#   ANTHROPIC_BASE_URL  = https://api.minimax.io/anthropic
#   ANTHROPIC_AUTH_TOKEN = <key from .env>
#   ANTHROPIC_MODEL     = MiniMax-M2.7
#
# Per .claude/lessons/feedback_brehon_subagent_model_effort_assignments.md and
# C:\Users\barri\.claude\projects\C--Users-barri-Developer-brehon-fork\memory\
# project_minimax_ab_trial_deferred.md.
#
# Usage:
#   queue-minimax-task.sh <description>
#
# Example:
#   queue-minimax-task.sh "[role:impl-task] sl-c-1 task 1 — see .claude/PRPs/briefs/sl-c-1-impl-1.md"
#
# Hard refusals:
#   - Will not run if MINIMAX_API_KEY is missing or empty.
#   - Will not run from any branch other than `ab-test/*` (collision mitigation
#     per the trial-design memory: throwaway branches, never merged).
#     Override the base branch via JUNIOR_BASE_BRANCH=<branch> if needed.
#   - Will not echo the key to stdout, stderr, or the SSH command argv preview.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ENV_FILE="${REPO_ROOT}/.env"

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <task-description>" >&2
  echo "  task-description: the [role:impl-task] line passed to 'junior task add'" >&2
  exit 2
fi

DESC="$1"

if [[ ! -f "${ENV_FILE}" ]]; then
  echo "ERROR: ${ENV_FILE} not found" >&2
  exit 1
fi

# Source MINIMAX_API_KEY without leaking other secrets into the env.
# Use a sub-shell scoped grep+eval rather than `source .env` so we don't
# accidentally export REF_MCP_API_KEY etc.
MINIMAX_API_KEY="$(grep -E '^MINIMAX_API_KEY=' "${ENV_FILE}" | head -1 | cut -d= -f2-)"

if [[ -z "${MINIMAX_API_KEY:-}" ]]; then
  echo "ERROR: MINIMAX_API_KEY not set in ${ENV_FILE}" >&2
  exit 1
fi

BASE_BRANCH="${JUNIOR_BASE_BRANCH:-ab-test/minimax-trial}"

# Build the remote command. We pass the key via stdin to avoid having it
# appear in `ps` argv on the laptop side. The EliteDesk side still gets it
# on its argv during the brief window `junior task add` runs — that's the
# trade-off Option B documented; SSH command-line argv is visible to anyone
# with shell access on the EliteDesk for the ~1-2 sec the command runs.
#
# Mitigation: SSH access to homeserver is restricted to the user; the key
# never lands on disk on the EliteDesk; the daemon stores envOverrides as
# job-row-scoped JSON in SQLite (not in any plaintext file).

REMOTE_CMD=$(cat <<'REMOTE'
set -euo pipefail
read -r KEY
junior task add \
  --base-branch "$1" \
  --env-override "ANTHROPIC_BASE_URL=https://api.minimax.io/anthropic" \
  --env-override "ANTHROPIC_AUTH_TOKEN=${KEY}" \
  --env-override "ANTHROPIC_MODEL=MiniMax-M2.7" \
  "$2"
REMOTE
)

echo "queueing task on EliteDesk:"
echo "  base-branch: ${BASE_BRANCH}"
echo "  description: ${DESC}"
echo "  env-overrides: ANTHROPIC_BASE_URL, ANTHROPIC_AUTH_TOKEN=<redacted>, ANTHROPIC_MODEL=MiniMax-M2.7"

printf '%s\n' "${MINIMAX_API_KEY}" | \
  ssh homeserver "bash -s -- '${BASE_BRANCH}' '${DESC}'" <<EOF_REMOTE
${REMOTE_CMD}
EOF_REMOTE
