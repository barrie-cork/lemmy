#!/bin/bash

# PreToolUse hook — block cargo/wrapper output piped through tail/head/grep/etc.
# Enforces .claude/rules/cargo-output-capture.md deterministically.
#
# Triggered on Bash tool calls. Reads the proposed command from the hook's
# stdin JSON, scans for the forbidden cargo-pipe pattern, and exits 2 with
# a fix-it message that the agent reads as feedback.
#
# Allow-list:
#   - `set -o pipefail` immediately preceding the pipe (rule's documented
#     escape hatch).
#   - Commands that already redirect cargo to a file (`> file 2>&1`) before
#     piping the file contents — the file redirect captured the exit code.

set -euo pipefail

HOOK_INPUT=$(cat)

COMMAND=$(printf '%s' "$HOOK_INPUT" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    print(data.get('tool_input', {}).get('command', ''))
except Exception:
    pass
" 2>/dev/null || true)

if [[ -z "$COMMAND" ]]; then
  exit 0
fi

# Allow only when `set -o pipefail` appears BEFORE any pipeline operator
# in the command. CR #85: the prior unconstrained match let `set -o pipefail`
# slip through if it appeared *after* the cargo pipeline (where it would have
# no effect on that pipeline). Require the next non-whitespace token after
# `pipefail` to be `;` or `&&` (i.e. another statement boundary), so the
# pipefail directive precedes the cargo pipeline.
if printf '%s' "$COMMAND" | grep -Eq '(^|;|&&|\|\|)\s*set\s+-o\s+pipefail(\s*;|\s*&&)'; then
  exit 0
fi

# The cargo-pipe pattern: a cargo invocation (raw or via the wrapper batch
# files) feeding tail/head/grep/sed/awk/less/more/jq.
#
# `cargo` must appear as a *command word* (start of line, or after `;`, `&&`,
# `||`, `|`, `(`, or backtick) so we don't match filenames like
# `audit-cargo-*.log` or paths containing the substring "cargo".
#
# Matches:
#   cargo build | tail
#   cargo check --workspace 2>&1 | head -100
#   ./scripts/brehon/cargo-test.bat --test e2e | grep PASS
#   cmd //c "scripts\\brehon\\cargo-check.bat -p X" | tail -40
#
# Does not match:
#   cargo build > /tmp/log 2>&1 ; tail -40 /tmp/log
#   cat .claude/build.log | tail -20         (no cargo command on the left)
#   ls .claude/audit-cargo-*.log | head      (cargo is part of a filename)
PATTERN='(^|;|&&|\|\||\||\(|`)\s*(\.?/?[a-zA-Z_./\\"-]*(cargo|cargo-(check|test|clippy|build)\.bat))[^|]*\|\s*(tail|head|grep|sed|awk|less|more|jq)\b'

if printf '%s' "$COMMAND" | grep -Eq "$PATTERN"; then
  cat >&2 <<'EOF'
Blocked by check-cargo-pipe hook (.claude/rules/cargo-output-capture.md).

Piping cargo (or the cargo-*.bat wrappers) through tail/head/grep/sed/awk/jq
masks cargo's exit code: the pipe reports the *right-hand* command's status,
so a failed build silently looks green.

Fix: capture to a file, then read the tail separately. Pattern:

    cargo <args> > .claude/build-<task>.log 2>&1
    status=$?
    tail -40 .claude/build-<task>.log
    [ $status -eq 0 ] || exit $status

Or, if you genuinely want a single inline call, prefix `set -o pipefail`:

    set -o pipefail
    cargo <args> 2>&1 | tail -40

Re-issue the command using one of these patterns.
EOF
  exit 2
fi

exit 0
