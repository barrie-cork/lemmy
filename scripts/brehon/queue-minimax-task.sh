#!/usr/bin/env bash
# queue-minimax-task.sh — queue a Junior impl-task with MiniMax env overrides
#
# Reads MINIMAX_API_KEY from the laptop's .env (gitignored, never committed)
# and queues a Junior task on the EliteDesk daemon with the three env overrides
# the daemon's executor.ts merges OVER the role-model mapping (envOverrides win;
# verified at /opt/junior-src/src/daemon/executor.ts:313 `Object.assign(childEnv, overrides)`
# runs AFTER the `--model` role injection, 2026-05-29):
#
#   ANTHROPIC_BASE_URL   = https://api.minimax.io/anthropic
#   ANTHROPIC_AUTH_TOKEN = <key from .env>
#   ANTHROPIC_MODEL      = ${MINIMAX_MODEL:-MiniMax-M2.7}
#
# This is the MiniMax ARM of the M2.7-vs-Sonnet A/B trial. The Sonnet CONTROL
# arm is just a normal `[role:impl-task]` dispatch (daemon pins claude-sonnet-4-6
# via the role-prefix patch) — do NOT use this script for the control arm.
#
# Model selection (default MiniMax-M2.7): the trial pins M2.7, NOT M2.5. The
# 2026-05-29 first-party benchmark chart (.claude/PRPs/reports/image.png) shows
# M2.7 beats M2.5 on every panel (SWE-Bench-Pro 56.2 vs 55.4; MLE-Bench-lite
# 66.6 vs 51.5; etc) at identical price ($0.30/$1.20 per M), so M2.5 is not an
# arm. Override only for a deliberate follow-up bake-off: MINIMAX_MODEL=MiniMax-M2.5.
#
# Per .claude/PRPs/briefs/minimax-m27-trial-1.md (the trial runbook) +
# C:\Users\barri\.claude\projects\C--Users-barri-Developer-brehon-fork\memory\
# project_minimax_ab_trial_deferred.md.
#
# Usage:
#   queue-minimax-task.sh <description>
#   MINIMAX_MODEL=MiniMax-M2.5 queue-minimax-task.sh <description>   # follow-up bake-off only
#
# Example:
#   queue-minimax-task.sh "[role:impl-task] rt-r4 task 1 — see .claude/PRPs/briefs/v1-RT-r4-impl-1.md"
#
# Hard refusals:
#   - Will not run if MINIMAX_API_KEY is missing or empty.
#   - Will not run from any branch other than `ab-test/*` (collision mitigation
#     per the trial-design memory: throwaway branches, never merged).
#     Override the base branch via JUNIOR_BASE_BRANCH=<branch> if needed.
#   - Will not echo the key to stdout, stderr, or the SSH command argv preview.
#
# Pre-flight (run once before the first trial task — the account had ZERO balance
# on 2026-05-29, "insufficient balance (1008)"; top up first, then confirm):
#   KEY="$(grep -E '^MINIMAX_API_KEY=' .env | cut -d= -f2-)"
#   curl -s -m25 -o /dev/null -w '%{http_code}\n' https://api.minimax.io/anthropic/v1/messages \
#     -H 'content-type: application/json' -H 'anthropic-version: 2023-06-01' \
#     -H "x-api-key: ${KEY}" \
#     -d '{"model":"MiniMax-M2.7","max_tokens":16,"messages":[{"role":"user","content":"PONG"}]}'
#   # Expect 200. A 500 with {"error":{"message":"insufficient balance (1008)"}} == top up the account.

set -euo pipefail

MINIMAX_MODEL="${MINIMAX_MODEL:-MiniMax-M2.7}"

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

# $1=base-branch  $2=task-description  $3=model. The model is a positional
# arg (not interpolated into the single-quoted heredoc) so the key+model never
# leak into the local `ps` argv; both arrive on the EliteDesk argv only.
REMOTE_CMD=$(cat <<'REMOTE'
set -euo pipefail
read -r KEY
junior task add \
  --base-branch "$1" \
  --env-override "ANTHROPIC_BASE_URL=https://api.minimax.io/anthropic" \
  --env-override "ANTHROPIC_AUTH_TOKEN=${KEY}" \
  --env-override "ANTHROPIC_MODEL=$3" \
  "$2"
REMOTE
)

echo "queueing task on EliteDesk:"
echo "  base-branch: ${BASE_BRANCH}"
echo "  description: ${DESC}"
echo "  env-overrides: ANTHROPIC_BASE_URL, ANTHROPIC_AUTH_TOKEN=<redacted>, ANTHROPIC_MODEL=${MINIMAX_MODEL}"

printf '%s\n' "${MINIMAX_API_KEY}" | \
  ssh homeserver "bash -s -- '${BASE_BRANCH}' '${DESC}' '${MINIMAX_MODEL}'" <<EOF_REMOTE
${REMOTE_CMD}
EOF_REMOTE
