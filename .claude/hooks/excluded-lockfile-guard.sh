#!/usr/bin/env bash
# PreToolUse hook (matcher: Bash): WARN if a workspace-excluded binary crate
# lacks a committed Cargo.lock when a commit is imminent.
#
# Motivation (incident 2026-06-12, services/bridge): a workspace-excluded crate
# with no committed Cargo.lock lets a cold CI/Docker build re-resolve deps to
# newest-in-range and break silently (services/bridge got `time 0.3.48` →
# `ruma-common 0.19.0` E0119, 25 errors). The prior mitigation was advisory and
# got ignored. This makes the check mechanical at commit time.
#
# Event: PreToolUse (matcher Bash). Cheap by design: early-exits unless there is
# staged content AND the staged set touches the root Cargo.toml `exclude` array
# or adds a new excluded-crate Cargo.toml — the only commits that can introduce
# the bug. Otherwise exit 0 immediately.
#
# Exit 0 ALWAYS — WARN-not-FAIL, mirroring pre-commit-session-guard.sh +
# pmd-canonical-guard.sh + session-start-multi-lane-check.sh per
# .claude/rules/pmd-invariants.md §5 rationale (no false-positive paralysis).
# The WARN lands in the session; the human/agent decides whether to commit.
#
# The actual check logic lives in scripts/brehon/check-excluded-lockfiles.sh
# (reusable, testable standalone, exit 1 on missing lock). This hook is the thin
# commit-time trigger + warn surface.

set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || true)
[ -z "$REPO_ROOT" ] && exit 0

# Only relevant when something is staged (a commit may be imminent).
STAGED=$(git -C "$REPO_ROOT" diff --cached --name-only 2>/dev/null || true)
[ -z "$STAGED" ] && exit 0

# Cheap relevance gate: only run the full check if the staged set touches the
# root Cargo.toml (where `exclude` lives) OR stages a Cargo.toml under any path
# (a candidate new excluded crate). Anything else cannot introduce the bug.
if ! grep -qE '(^Cargo\.toml$|/Cargo\.toml$)' <<< "$STAGED"; then
  exit 0
fi

GUARD="$REPO_ROOT/scripts/brehon/check-excluded-lockfiles.sh"
[ -f "$GUARD" ] || exit 0

# Run the check. It exits 1 if an excluded binary crate lacks a committed lock.
if ! out=$(bash "$GUARD" 2>&1); then
  echo "WARN [excluded-lockfile-guard]: a workspace-excluded binary crate is missing a committed Cargo.lock." >&2
  echo "$out" | sed 's/^/  /' >&2
  echo "  This commit may introduce silent dep-drift breakage on a cold CI build." >&2
  echo "  Generate + commit the lock (cargo-linux.sh --manifest-path <crate>/Cargo.toml), or proceed knowingly." >&2
  echo "  Per .claude/lessons/feedback_workspace_excluded_crate_must_have_lockfile.md" >&2
fi

exit 0
