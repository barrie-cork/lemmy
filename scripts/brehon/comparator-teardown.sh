#!/usr/bin/env bash
# comparator-teardown.sh — clean up stale ab-cell worktrees and branch refs.
#
# The run-comparator.sh script creates throwaway worktrees under
# ~/comparator-worktrees/ and keeps the ab-cell/<exp>-<arm> branch ref for
# forensic git-diff. This script removes stale ones.
#
# Usage:
#   comparator-teardown.sh [options]
#     --experiment <id>   remove worktrees/branches for this experiment only
#     --all               remove ALL ab-cell/ worktrees and branches
#     --list              list existing ab-cell worktrees (dry-run)
#     --repo <path>       default: /srv/brehon-fork (or CWD if .git exists)
#
# Hard refusals:
#   - Never removes the live daemon worktrees in /srv/brehon-fork/.junior/worktrees/
#   - Never removes branches that are NOT prefixed with ab-cell/
#   - Never force-removes a worktree with uncommitted tracked changes (warns instead)

set -euo pipefail

EXPERIMENT=""
ALL=false
LIST_ONLY=false
REPO=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --experiment) EXPERIMENT="$2"; shift 2;;
    --all)        ALL=true; shift;;
    --list)       LIST_ONLY=true; shift;;
    --repo)       REPO="$2"; shift 2;;
    -h|--help)    sed -n '1,20p' "$0"; exit 0;;
    *) echo "ERROR: unknown arg: $1" >&2; exit 2;;
  esac
done

# Resolve repo.
if [[ -z "${REPO}" ]]; then
  if [[ -f "/srv/brehon-fork/.git/config" ]]; then
    REPO="/srv/brehon-fork"
  elif [[ -f ".git/config" ]]; then
    REPO="$(pwd)"
  else
    echo "ERROR: could not find a git repo. Use --repo <path>." >&2; exit 2
  fi
fi

WT_PARENT="${COMPARATOR_WT_DIR:-${HOME}/comparator-worktrees}"

echo "comparator-teardown: repo=${REPO}"
echo "  worktree parent: ${WT_PARENT}"

# List all ab-cell worktrees from git.
WORKTREES="$(git -C "${REPO}" worktree list --porcelain 2>/dev/null | grep -E '^worktree|^branch' || true)"

# Find ab-cell/ entries.
AB_CELL_WTS=()
while IFS= read -r wt_path; do
  # Filter to WT_PARENT path + only ab-cell dirs.
  if [[ "${wt_path}" == "${WT_PARENT}/abcell-"* ]]; then
    AB_CELL_WTS+=("${wt_path}")
  fi
done < <(git -C "${REPO}" worktree list --porcelain 2>/dev/null | grep '^worktree ' | awk '{print $2}')

AB_CELL_BRANCHES=()
while IFS= read -r branch; do
  if [[ "${branch}" == ab-cell/* ]]; then
    AB_CELL_BRANCHES+=("${branch}")
  fi
done < <(git -C "${REPO}" branch --list 'ab-cell/*' --format='%(refname:short)' 2>/dev/null || true)

echo "  ab-cell worktrees found: ${#AB_CELL_WTS[@]}"
for wt in "${AB_CELL_WTS[@]}"; do echo "    ${wt}"; done
echo "  ab-cell branches found: ${#AB_CELL_BRANCHES[@]}"
for br in "${AB_CELL_BRANCHES[@]}"; do echo "    ${br}"; done

if [[ "${LIST_ONLY}" == true ]]; then
  echo "  [list-only mode — no changes made]"
  exit 0
fi

if [[ "${ALL}" == false && -z "${EXPERIMENT}" ]]; then
  echo "ERROR: specify --experiment <id> or --all" >&2; exit 2
fi

REMOVED_WTS=0
REMOVED_BRANCHES=0

for wt in "${AB_CELL_WTS[@]}"; do
  # Filter by experiment if requested.
  if [[ "${ALL}" == false ]]; then
    if [[ "${wt}" != *"abcell-${EXPERIMENT}-"* ]]; then
      continue
    fi
  fi

  # Check for uncommitted changes (safety — refuse to drop dirty worktree).
  if git -C "${wt}" status --short 2>/dev/null | grep -q '^[AM]'; then
    echo "  WARN: ${wt} has uncommitted tracked changes — skipping (remove manually after inspection)"
    continue
  fi

  echo "  removing worktree: ${wt}"
  git -C "${REPO}" worktree remove --force "${wt}" 2>/dev/null || \
    echo "  WARN: worktree remove failed; remove manually: git -C '${REPO}' worktree remove --force '${wt}'"
  REMOVED_WTS=$((REMOVED_WTS + 1))
done

for br in "${AB_CELL_BRANCHES[@]}"; do
  if [[ "${ALL}" == false ]]; then
    if [[ "${br}" != "ab-cell/${EXPERIMENT}-"* ]]; then
      continue
    fi
  fi
  echo "  deleting branch: ${br}"
  git -C "${REPO}" branch -D "${br}" 2>/dev/null || \
    echo "  WARN: branch delete failed: ${br}"
  REMOVED_BRANCHES=$((REMOVED_BRANCHES + 1))
done

echo "  done: removed ${REMOVED_WTS} worktrees, ${REMOVED_BRANCHES} branches"
