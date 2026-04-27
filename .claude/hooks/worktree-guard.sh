#!/usr/bin/env bash
# PreToolUse hook: block Junior worker claude from escaping its worktree.
#
# Triggered by PMD #998 (2026-04-11): a Junior worker for agent-grey
# ran `sudo chown -R /srv/agent-grey/.project-memory` + a bad `ln -sf`
# from PWD=/srv/agent-grey (outside its own worktree), destroying the
# live PMD via a self-referencing symlink. Worker claude inherits
# passwordless sudo as user `barrie` and runs with
# `--dangerously-skip-permissions`, giving it unconfined host access.
#
# Policy — when $PWD is inside /srv/<repo>/.junior/worktrees/<job>/:
#   1. Bash: deny any command containing `sudo` (word-boundary) or
#      `ln -s` / `ln -sf` (symlink creation).
#   2. Edit/Write/MultiEdit: deny if tool_input.file_path is not
#      under $PWD (absolute path escape OR resolves outside worktree).
#   3. All other tools: pass through.
#
# Policy — when $PWD is NOT inside a Junior worktree (interactive Mac
# session, server admin shell, etc.): exit 0 immediately. This hook
# only constrains unattended workers.
#
# Event: PreToolUse
# Matcher: Bash|Edit|Write|MultiEdit
# Timeout: 5000

set -o pipefail

INPUT=$(cat)

# Claude Code passes the session cwd as a top-level field in PreToolUse
# events. Use that rather than $PWD — shells may reset PWD when the hook
# subprocess starts. Fall back to $PWD if the field is missing.
CWD=$(echo "$INPUT" | jq -r '.cwd // empty' 2>/dev/null)
[ -z "$CWD" ] && CWD="$PWD"

# Detect worktree context. Junior creates worktrees under
# /srv/<repo>/.junior/worktrees/<job-dir>. If we're not in one, this
# is an interactive or admin session — don't touch it.
case "$CWD" in
  /srv/*/.junior/worktrees/*) ;;
  *) exit 0 ;;
esac

TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null) || exit 0
[ -z "$TOOL_NAME" ] && exit 0

deny() {
  local reason="$1"
  jq -n --arg reason "$reason" '{
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: "deny",
      permissionDecisionReason: $reason
    }
  }'
  exit 0
}

case "$TOOL_NAME" in
  Bash)
    CMD=$(echo "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null) || exit 0
    [ -z "$CMD" ] && exit 0

    # Block sudo as a standalone word. Covers `sudo chown`, `| sudo sh`,
    # `&& sudo ...`, etc. Does NOT match `pseudo` or `sudoers` reads.
    if printf '%s' "$CMD" | grep -qE '(^|[[:space:];&|])sudo([[:space:]]|$)'; then
      deny "Junior workers may not invoke sudo. This call would run outside the worktree sandbox. If the task legitimately needs elevated access, redesign it as a Mac-side admin action or a dedicated systemd unit. PMD #998."
    fi

    # Block symlink creation. `ln -s`, `ln -sf`, `ln -sfn`, etc. The
    # agent-grey incident used `ln -sf` to create a self-loop over a
    # gitignored shared directory. Hard block — workers should never
    # need to create host symlinks.
    if printf '%s' "$CMD" | grep -qE '(^|[[:space:];&|])ln[[:space:]]+-[a-zA-Z]*s'; then
      deny "Junior workers may not create symlinks (ln -s...). This was the vector for PMD #998 (self-loop symlink destroyed agent-grey PMD). If you need to share state with the main repo, let the human do it."
    fi
    ;;

  Edit|Write|MultiEdit)
    FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null) || exit 0
    [ -z "$FILE_PATH" ] && exit 0

    # Only absolute paths can escape the worktree in practice. Relative
    # paths resolve against cwd by definition.
    case "$FILE_PATH" in
      /*)
        case "$FILE_PATH" in
          "$CWD"|"$CWD"/*) ;;
          *)
            deny "Junior workers may not write outside their worktree. Attempted path: $FILE_PATH (worktree: $CWD). PMD #998."
            ;;
        esac
        ;;
    esac
    ;;
esac

exit 0
