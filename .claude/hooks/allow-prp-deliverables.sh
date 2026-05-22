#!/usr/bin/env bash
# PreToolUse hook: auto-approve Junior worker writes to PRP DELIVERABLE
# artifact paths, bypassing the Claude Code v2.1.119 hardcoded
# ".claude/** sensitive file" gate that is NOT overridden by
# --dangerously-skip-permissions (bypassPermissions mode) nor by
# settings.json permissions.allow.
#
# Triggered by Junior #270 (2026-05-16): a [role:planning] worker
# authored a complete v1-ship-1-r1 plan but EVERY write to
# .claude/PRPs/plans/ (and even allow-listed .claude/runlog/ +
# .claude/PRPs/reviews/) was blocked with
#   "Claude requested permissions to edit ... which is a sensitive file"
# despite init.permissionMode == bypassPermissions. Root cause is a
# CC-CLI-level protection of the agent config tree, not a daemon defect.
# See .claude/PRPs/reports/v1-ship-1-r1-junior270-escalation.md and
# advisor task #10.
#
# Policy — when $CWD is inside /srv/<repo>/.junior/worktrees/<job>/:
#   Write/Edit/MultiEdit whose tool_input.file_path resolves INSIDE the
#   worktree AND lands under .claude/PRPs/{plans,briefs,reports}/  ->
#   permissionDecision "allow" (these are DELIVERABLE artifacts the
#   four-role model REQUIRES the Junior worker to author: plans,
#   briefs, retros/reports).
#
#   Everything else -> exit 0 with NO decision (pass-through). The CC
#   sensitive-file gate + worktree-guard.sh still apply normally to
#   behavior-config paths (.claude/rules/**, .claude/agents/**,
#   .claude/lessons/**, .claude/commands/**, .claude/skills/**),
#   .claude/decision-queue.json (already in permissions.allow), and
#   any out-of-worktree path-escape.
#
# Policy — when $CWD is NOT inside a Junior worktree (interactive
# advisor/admin session): exit 0 immediately. This hook only widens
# the autonomous worker's deliverable-write surface; it must not
# affect interactive sessions.
#
# This hook NEVER denies. It only ADDS an "allow" for the three
# deliverable globs inside the worktree. worktree-guard.sh (same
# matcher) still independently denies path-escape / sudo / symlink;
# an "allow" from this hook does NOT suppress another hook's "deny".
#
# Event: PreToolUse
# Matcher: Write|Edit|MultiEdit
# Timeout: 5000

set -o pipefail

INPUT=$(cat)

# Use the session cwd from the PreToolUse event (shells may reset $PWD
# when the hook subprocess starts). Mirror worktree-guard.sh exactly.
CWD=$(echo "$INPUT" | jq -r '.cwd // empty' 2>/dev/null)
[ -z "$CWD" ] && CWD="$PWD"

# Only act inside a Junior worktree. Interactive/admin sessions: noop.
case "$CWD" in
  /srv/*/.junior/worktrees/*) ;;
  *) exit 0 ;;
esac

TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null) || exit 0
[ -z "$TOOL_NAME" ] && exit 0

case "$TOOL_NAME" in
  Write|Edit|MultiEdit) ;;
  *) exit 0 ;;
esac

FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null) || exit 0
[ -z "$FILE_PATH" ] && exit 0

# Normalise to a worktree-relative path. Two cases:
#   1. absolute path: must be inside $CWD, then strip the $CWD prefix.
#   2. relative path: already relative to $CWD by definition.
REL=""
case "$FILE_PATH" in
  /*)
    case "$FILE_PATH" in
      "$CWD"/*) REL="${FILE_PATH#"$CWD"/}" ;;
      "$CWD")   REL="" ;;
      # Absolute path OUTSIDE the worktree: do NOT allow. Pass through
      # so worktree-guard.sh denies the escape.
      *) exit 0 ;;
    esac
    ;;
  *)
    REL="$FILE_PATH"
    ;;
esac

# Reject any path containing a ".." segment (defence-in-depth against
# .claude/PRPs/plans/../../rules/foo traversal). Pass through (no
# allow) so the normal gate applies.
case "/$REL/" in
  *"/../"*) exit 0 ;;
esac

# The ONLY three deliverable globs this hook approves. Behavior-config
# trees (rules/agents/lessons/commands/skills) and decision-queue.json
# are deliberately EXCLUDED — the sensitive-file gate stays in force
# there. decision-queue.json is separately covered by permissions.allow.
case "$REL" in
  .claude/PRPs/plans/*|.claude/PRPs/briefs/*|.claude/PRPs/reports/*)
    jq -n --arg p "$REL" '{
      hookSpecificOutput: {
        hookEventName: "PreToolUse",
        permissionDecision: "allow",
        permissionDecisionReason: ("Junior PRP deliverable write auto-approved by allow-prp-deliverables.sh (CC v2.1.119 .claude/** sensitive-gate workaround; advisor task #10). Path: " + $p + ". Scope: plans/briefs/reports only — behavior config (rules/agents/lessons/commands/skills) stays gated.")
      }
    }'
    exit 0
    ;;
  *)
    # Any other path (incl. .claude/rules/**, .claude/agents/**,
    # .claude/lessons/**, .claude/commands/**, .claude/skills/**,
    # .claude/decision-queue.json, non-.claude paths): pass through.
    exit 0
    ;;
esac
