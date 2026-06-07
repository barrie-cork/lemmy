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
#   ANTHROPIC_API_KEY    = <key from .env>
#   ANTHROPIC_MODEL      = ${MINIMAX_MODEL:-MiniMax-M3}
#
# This is the MiniMax ARM of the M3-vs-Sonnet A/B trial. The Sonnet CONTROL
# arm is just a normal `[role:impl-task]` dispatch (daemon pins claude-sonnet-4-6
# via the role-prefix patch) — do NOT use this script for the control arm.
#
# Model selection (default MiniMax-M3): the trial pins M3 (upgraded from M2.7
# 2026-06-07 per user instruction). Override only for a deliberate bake-off
# against an older variant: MINIMAX_MODEL=MiniMax-M2.7.
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
#     -d '{"model":"MiniMax-M3","max_tokens":16,"messages":[{"role":"user","content":"PONG"}]}'
#   # Expect 200. A 500 with {"error":{"message":"insufficient balance (1008)"}} == top up the account.

set -euo pipefail

MINIMAX_MODEL="${MINIMAX_MODEL:-MiniMax-M3}"

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

# Single SSH command, ONE stdin source (the key pipe). The non-secret values
# (base-branch, description, model) are interpolated LOCALLY into the remote
# command string; the daemon reads ONLY the key from stdin via `read -r KEY`.
# The key never appears on argv (local `ps` or daemon `ps`) — it lives only in
# the piped stdin and the env-override the daemon stores in its SQLite job row.
#
# BUGFIX 2026-06-04 (m1-b T3 dispatch): the prior form had TWO stdin sources —
#   printf '%s\n' "$KEY" | ssh ... "bash -s -- ..." <<EOF_REMOTE \n $REMOTE_CMD \n EOF_REMOTE
# The `<<EOF_REMOTE` heredoc redirect WON over the `printf |` pipe, so ssh's
# stdin became the REMOTE_CMD body (not the key); `read -r KEY` read the first
# script line ("set -euo pipefail") as the key, and the `junior task add \`
# continuation lines parsed as standalone commands → the observed
# `--base-branch: command not found`. Fix below: a single-line remote command
# (no heredoc), key piped as the sole stdin. This form was verified working
# on the m1-b T3 MiniMax dispatch (#577).
#
# Quote-safety: DESC/BASE_BRANCH must not contain a single-quote (the dispatch
# lines never do). Guard explicitly rather than risk a broken remote command.
case "${DESC}${BASE_BRANCH}" in
  *\'*) echo "ERROR: description/base-branch must not contain a single-quote (')." >&2; exit 1;;
esac

echo "queueing task on EliteDesk:"
echo "  base-branch: ${BASE_BRANCH}"
echo "  description: ${DESC}"
echo "  env-overrides: ANTHROPIC_BASE_URL, ANTHROPIC_API_KEY=<redacted>, ANTHROPIC_MODEL=${MINIMAX_MODEL}"

# The remote command string: base/desc/model are wrapped in single-quotes by
# this local heredoc-free assembly; `read -r KEY` pulls the key from stdin.
# `printf | ssh '<cmd>'` — the single-quoted ssh arg is the literal remote
# command; the daemon's shell sees it after local ${...} expansion.
#
# BUGFIX 2026-06-04 (m1-b T4 dispatch): the prior form ran `junior task add`
# WITHOUT first `cd ${REMOTE_REPO}`. `ssh homeserver` lands in the login
# default cwd (/home/barrie), where the `junior` CLI resolves to a DIFFERENT
# daemon instance's DB (/home/barrie/.junior/junior.db — different schema)
# instead of the brehon daemon at /srv/brehon-fork/.junior/junior.db. The
# dispatch reported "Task #N queued" from the wrong DB and NO job appeared in
# the brehon DB (no worktree created → silent no-op for brehon). Fix: `cd`
# into the brehon repo so the CLI targets the brehon daemon. ALWAYS verify
# the new job lands in /srv/brehon-fork/.junior/junior.db after dispatch
# (the CLI's echoed id can be from any junior instance — see
# issue_note_minimax_dispatch_wrong_daemon_db.md findings A+B).
REMOTE_REPO="${JUNIOR_REMOTE_REPO:-/srv/brehon-fork}"
REMOTE_CMD="cd '${REMOTE_REPO}' && read -r KEY && junior task add --base-branch '${BASE_BRANCH}' --env-override 'ANTHROPIC_BASE_URL=https://api.minimax.io/anthropic' --env-override \"ANTHROPIC_API_KEY=\${KEY}\" --env-override 'ANTHROPIC_MODEL=${MINIMAX_MODEL}' '${DESC}'"

printf '%s\n' "${MINIMAX_API_KEY}" | ssh homeserver "${REMOTE_CMD}"
