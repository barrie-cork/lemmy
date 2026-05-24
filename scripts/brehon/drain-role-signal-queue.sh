#!/usr/bin/env bash
# drain-role-signal-queue.sh — pull EliteDesk's role-signal JSONL queue and
# ingest each row into the canonical laptop PMD.
#
# Why this script exists:
# The role-signal-utilisation.sh Stop hook on EliteDesk Junior workers cannot
# write directly to the canonical PMD (that DB lives on the laptop). The hook
# falls back to a JSONL queue at <worker-worktree>/.claude/role-signal-queue.jsonl.
# Worker worktrees get reaped after task completion, so the queue file would
# vanish. The daemon's main checkout DOES persist though, and Junior workers
# write their queue file into the worktree which lives under
# /srv/brehon-fork/.junior/worktrees/job-N/.claude/role-signal-queue.jsonl —
# the queue file CAN survive until reap. To capture signals lossless, we
# rsync any matching files from EliteDesk before the next worktree reap and
# ingest into the laptop PMD via the write-role-signal CLI.
#
# Usage:
#   bash scripts/brehon/drain-role-signal-queue.sh [--dry-run]
#
# Idempotent: lines already ingested are tracked via a sidecar
# .drained-rows file (one ingested ts:task_id pair per line) on the laptop.
# Re-running ingests only new lines.
#
# Exit codes:
#   0 — success (zero or more rows ingested)
#   1 — bad usage or missing dependency
#   2 — rsync or ingest error

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DRAIN_DIR="${REPO_ROOT}/.claude/role-signal-drain"
DRAINED_INDEX="${DRAIN_DIR}/.drained-rows"
CLI="node C:/Users/barri/Developer/MCPs/project-memory-mcp/dist/scripts/write-role-signal.js"

DRY_RUN=0
if [ "${1:-}" = "--dry-run" ]; then
  DRY_RUN=1
fi

mkdir -p "$DRAIN_DIR"
touch "$DRAINED_INDEX"

# 1. Pull queue files from EliteDesk worker worktrees + the daemon main
#    checkout (in case a hook ever ran with WORKER_DIR fallback to main).
#    Uses scp (Windows-compatible — rsync not available on Git-Bash) plus
#    a remote `find -print0 | xargs scp` for the worktree fan-out. Per
#    feedback_cross_platform_divergences.md (no rsync on Windows path).
echo "==> pull queue files from homeserver"

# Main checkout queue (single fixed path — most common case for bm-task)
scp -q homeserver:/srv/brehon-fork/.claude/role-signal-queue.jsonl \
  "${DRAIN_DIR}/main-checkout-queue.jsonl" 2>/dev/null \
  && echo "  pulled main-checkout-queue.jsonl" \
  || true

# Worktree queues (variable per-task paths)
mkdir -p "${DRAIN_DIR}/worktrees"
WORKTREE_QUEUES=$(ssh homeserver "find /srv/brehon-fork/.junior/worktrees -name 'role-signal-queue.jsonl' 2>/dev/null" 2>/dev/null || echo "")
if [ -n "$WORKTREE_QUEUES" ]; then
  while IFS= read -r remote_path; do
    [ -z "$remote_path" ] && continue
    # Flatten to one file per worktree: job-N-queue.jsonl
    job_name=$(basename "$(dirname "$(dirname "$remote_path")")")
    scp -q "homeserver:$remote_path" "${DRAIN_DIR}/worktrees/${job_name}-queue.jsonl" 2>/dev/null \
      && echo "  pulled ${job_name}-queue.jsonl" \
      || true
  done <<< "$WORKTREE_QUEUES"
fi

# 2. Walk every queue file, ingest lines not yet in DRAINED_INDEX.
INGESTED=0
SKIPPED=0
ERRORED=0

# Build a portable list of queue files
QUEUE_FILES=()
while IFS= read -r -d '' f; do
  QUEUE_FILES+=("$f")
done < <(find "$DRAIN_DIR" -name 'role-signal-queue.jsonl' -print0 2>/dev/null)
if [ -f "${DRAIN_DIR}/main-checkout-queue.jsonl" ]; then
  QUEUE_FILES+=("${DRAIN_DIR}/main-checkout-queue.jsonl")
fi

if [ ${#QUEUE_FILES[@]} -eq 0 ]; then
  echo "==> no queue files found on homeserver"
  exit 0
fi

echo "==> processing ${#QUEUE_FILES[@]} queue file(s)"

for qfile in "${QUEUE_FILES[@]}"; do
  echo "  -- $qfile"
  while IFS= read -r line; do
    [ -z "$line" ] && continue

    # Extract a stable dedup key: ts + task_id
    KEY=$(echo "$line" | python3 -c "
import json, sys
try:
    d = json.loads(sys.stdin.read())
    print(f\"{d.get('ts','')}:{d.get('task_id','')}\")
except Exception:
    pass
" 2>/dev/null || echo "")
    if [ -z "$KEY" ] || [ "$KEY" = ":" ]; then
      echo "    skip: unparseable line"
      ERRORED=$((ERRORED + 1))
      continue
    fi

    if grep -Fxq "$KEY" "$DRAINED_INDEX" 2>/dev/null; then
      SKIPPED=$((SKIPPED + 1))
      continue
    fi

    if [ "$DRY_RUN" = "1" ]; then
      echo "    DRY: would ingest $KEY"
      continue
    fi

    # Parse fields and call CLI
    ROLE=$(echo "$line" | python3 -c "import json,sys; print(json.loads(sys.stdin.read()).get('role',''))")
    KIND=$(echo "$line" | python3 -c "import json,sys; print(json.loads(sys.stdin.read()).get('kind',''))")
    TASK_ID=$(echo "$line" | python3 -c "import json,sys; print(json.loads(sys.stdin.read()).get('task_id',''))")
    BRANCH=$(echo "$line" | python3 -c "import json,sys; print(json.loads(sys.stdin.read()).get('branch',''))")
    CONFIG_VERSION=$(echo "$line" | python3 -c "import json,sys; print(json.loads(sys.stdin.read()).get('config_version',''))")
    CONTENT=$(echo "$line" | python3 -c "import json,sys; print(json.dumps(json.loads(sys.stdin.read()).get('content',{})))")

    if PROJECT_MEMORY_DB="${REPO_ROOT}/.project-memory/memory.db" PROJECT_ROOT="${REPO_ROOT}" \
       $CLI \
         --role "$ROLE" \
         --kind "$KIND" \
         --task-id "$TASK_ID" \
         --branch "$BRANCH" \
         --config-version "$CONFIG_VERSION" \
         --content "$CONTENT" 2>&1 >/dev/null; then
      echo "$KEY" >> "$DRAINED_INDEX"
      INGESTED=$((INGESTED + 1))
    else
      echo "    error: CLI failed on $KEY"
      ERRORED=$((ERRORED + 1))
    fi
  done < "$qfile"
done

echo "==> drain complete: ingested=$INGESTED, skipped=$SKIPPED, errored=$ERRORED"

if [ "$ERRORED" -gt 0 ]; then
  exit 2
fi
exit 0
