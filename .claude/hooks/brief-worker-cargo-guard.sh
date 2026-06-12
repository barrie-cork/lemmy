#!/usr/bin/env bash
# PreToolUse hook (matcher: Bash): WARN when an impl-task brief being committed
# contains worker-side cargo invocations.
#
# Motivation (recurring incident, 2× by 2026-06-12):
#   The advisor authors an `[role:impl-task]` brief whose probe/validation body
#   tells the EliteDesk worker to run cargo — `cmd //c "scripts\brehon\cargo-*.bat"`
#   or `cd services/bridge && cargo ...`. This violates the project HARD RULE
#   NO-CARGO-ON-ELITEDESK (project_laptop_canonical_cargo_runner.md): the daemon
#   is Linux, memory-constrained, has no cmd.exe and no cargo on PATH. A worker
#   that hits such a line either (a) catch-fires the whole /auto-phase on the
#   probe (`cmd: command not found`), or (b) — if a cargo IS present — OOMs the
#   daemon (2026-05-31 swap-exhaustion near-miss). cargo runs ONLY on the laptop
#   via the `validate-pending-laptop` DQ handler; the worker writes the DQ + stops.
#
#   This is an ENFORCEMENT gap, not a decision gap — the rule is already stated in
#   CLAUDE.md, .claude/agents/impl-task.md, and the forbidden-window check. The
#   accidental violation is what recurs. This hook makes the check mechanical at
#   brief-commit time, before the bad brief ever reaches the daemon.
#
# Event: PreToolUse (matcher Bash). Cheap by design: early-exits unless the
# proposed Bash command is a `git commit`/`git add` AND the staged set includes
# an impl-task brief. Otherwise exit 0 immediately.
#
# Exit 0 ALWAYS — WARN-not-FAIL, mirroring pre-commit-session-guard.sh +
# excluded-lockfile-guard.sh + pmd-canonical-guard.sh per
# .claude/rules/pmd-invariants.md §5 rationale. The WARN lands in the session;
# the human/agent decides whether to commit. A brief that LEGITIMATELY documents
# the laptop-side cargo command (clearly marked "laptop-advisor", "DO NOT run on
# worker", or inside a §4 advisor-side note) is the false-positive case the human
# acknowledges by committing anyway.

set -euo pipefail

HOOK_INPUT=$(cat 2>/dev/null || true)
[ -z "$HOOK_INPUT" ] && exit 0

# Extract the proposed Bash command from the hook stdin JSON.
COMMAND=$(printf '%s' "$HOOK_INPUT" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    print(data.get('tool_input', {}).get('command', ''))
except Exception:
    print('')
" 2>/dev/null || true)
[ -z "$COMMAND" ] && exit 0

# Relevance gate: only act on a commit/add (a brief is about to be committed).
case "$COMMAND" in
  *"git commit"*|*"git add"*) : ;;
  *) exit 0 ;;
esac

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || true)
[ -z "$REPO_ROOT" ] && exit 0

# Which impl-task briefs are staged? (also catch fix-impl briefs — same hazard)
STAGED=$(git -C "$REPO_ROOT" diff --cached --name-only 2>/dev/null || true)
[ -z "$STAGED" ] && exit 0

BRIEFS=$(printf '%s\n' "$STAGED" | grep -E '^\.claude/PRPs/briefs/.*(impl-task|impl-[0-9]|fix-impl).*\.md$' || true)
[ -z "$BRIEFS" ] && exit 0

# Worker-side cargo signatures. These are the forms a worker would EXECUTE.
# `cmd //c ...cargo-*.bat`  → Windows wrapper, unrunnable on the Linux daemon.
# `cd services/bridge && cargo` → bridge cargo run from the worker.
# A bare `cargo check/clippy/test/build` line (not inside a documented commands:
# array) → direct daemon cargo.
# We do NOT flag `cargo-linux.sh` or `validate-pending-laptop` mentions — those
# ARE the correct laptop-side forms. We do NOT flag lines that mention "laptop"
# or "DO NOT run" on the same line (explicit reference-only annotation).
# Collect matches into a temp file (printed with %s, never %b — brief content can
# contain backslashes like `scripts\brehon` that %b would mangle into control chars).
HITS_FILE=$(mktemp 2>/dev/null || echo "/tmp/brief-cargo-guard-$$")
trap 'rm -f "$HITS_FILE"' EXIT
found=0
while IFS= read -r brief; do
  [ -z "$brief" ] && continue
  bpath="$REPO_ROOT/$brief"
  # Read the STAGED version (what's about to be committed), not the worktree, so
  # a fixed-but-unstaged brief isn't flagged and a staged-bad one is.
  staged_content=$(git -C "$REPO_ROOT" show ":$brief" 2>/dev/null || cat "$bpath" 2>/dev/null || true)
  [ -z "$staged_content" ] && continue

  # grep for worker-cargo patterns, excluding clearly-annotated reference lines.
  matched=$(printf '%s\n' "$staged_content" \
    | grep -nE 'cmd //c.*cargo-[a-z]+\.(bat|sh)|cd services/bridge && cargo|^[[:space:]]*cargo (check|clippy|test|build)' \
    | grep -viE 'cargo-linux\.sh|validate-pending-laptop|laptop|DO NOT run|never .* on the worker|SKIP' \
    || true)
  if [ -n "$matched" ]; then
    found=1
    printf '  %s:\n' "$brief" >> "$HITS_FILE"
    printf '%s\n' "$matched" | sed 's/^/    /' >> "$HITS_FILE"
  fi
done <<< "$BRIEFS"

if [ "$found" -eq 1 ]; then
  echo "WARN [brief-worker-cargo-guard]: an impl-task brief being committed contains worker-side cargo." >&2
  cat "$HITS_FILE" >&2
  echo "  NO-CARGO-ON-ELITEDESK: the daemon is Linux (no cmd.exe, no cargo, memory-constrained)." >&2
  echo "  A worker hitting these lines catch-fires the probe ('cmd: not found') or OOMs the daemon." >&2
  echo "  Fix: the worker runs only Linux git/grep/gh probes; cargo sanity is a laptop-advisor" >&2
  echo "  pre-launch check or a validate-pending-laptop DQ. Mirror .claude/PRPs/briefs/m1-b-impl-0.md." >&2
  echo "  If this line is a clearly-annotated laptop/reference-only command, commit to acknowledge." >&2
  echo "  Per project_laptop_canonical_cargo_runner.md + .claude/agents/impl-task.md." >&2
fi

exit 0
