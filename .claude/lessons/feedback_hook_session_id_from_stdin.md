---
name: Claude Code hooks get session_id + transcript_path via stdin JSON, never an env var — CLAUDE_SESSION_ID does not exist
description: Stop / PostToolUse / PreToolUse hooks receive {session_id, transcript_path, cwd, permission_mode, ...} as JSON on STDIN, not as environment variables. There is NO CLAUDE_SESSION_ID env var. Code that reads ${CLAUDE_SESSION_ID:-$PPID} silently falls through to the unstable PPID. For a stable per-session key, parse stdin.
type: feedback
---

# Claude Code hooks read session identity from stdin JSON, not env vars

## TL;DR

Claude Code pipes a **hook-input JSON object** to the hook command's **stdin**.
For a `Stop` hook it contains at least:

```json
{
  "session_id": "abc123",
  "transcript_path": "/.../transcript.jsonl",
  "cwd": "/path/to/project",
  "permission_mode": "default",
  "effort": { "level": "medium" },
  "hook_event_name": "Stop"
}
```

**There is no `CLAUDE_SESSION_ID` environment variable.** (Verified against
`code.claude.com/docs/en/hooks`, 2026-06-11.) The documented hook env vars are
`CLAUDE_PROJECT_DIR`, `CLAUDE_PLUGIN_ROOT`, `CLAUDE_PLUGIN_DATA`, `CLAUDE_EFFORT`,
`CLAUDE_ENV_FILE` (SessionStart/Setup/CwdChanged/FileChanged only), and
`CLAUDE_CODE_REMOTE`. Session identity is **not** among them.

Any hook code reading `${CLAUDE_SESSION_ID:-${PPID:-unknown}}` for a *stable*
per-session key is silently broken: the env var never resolves, so it always
falls through to `PPID` — which is **unstable across hook invocations** (each
Stop hook is a fresh bash with a potentially different parent), the exact failure
this fallback was meant to avoid. The bug is invisible because the fallback
"works" (produces *a* value); it's just the wrong, unstable one.

## Why it matters

A hook that needs to track per-session state across multiple firings — a
session-age marker, a per-session retry counter, a once-per-session gate — needs
a **stable** key. PPID is not stable. The real `session_id` from stdin is the
only stable per-session identifier available to a hook.

Incident (2026-06-11, `retro-check.sh`): the session-age gate keyed its
first-seen-epoch marker on the session id. Had it used the phantom
`CLAUDE_SESSION_ID` (→ PPID fallback), the marker filename would have changed
between Stop invocations, so age would reset to zero every turn-end and the gate
would never engage. Reading the real stdin `session_id` is what makes the age
accumulate monotonically.

## How to apply

Read stdin **once** at the top of the hook (Stop hooks don't otherwise consume
fd 0), parse with `jq`, sanitize for filesystem use:

```bash
HOOK_STDIN=""
if [ ! -t 0 ]; then
  HOOK_STDIN="$(cat || true)"
fi
SESSION_ID=""
if [ -n "$HOOK_STDIN" ] && command -v jq >/dev/null 2>&1; then
  SESSION_ID="$(printf '%s' "$HOOK_STDIN" | jq -r '.session_id // ""' 2>/dev/null || true)"
fi
# Fallback only if parse failed — keeps the hook functional but coarser.
[ -z "$SESSION_ID" ] && SESSION_ID="ppid-${PPID:-unknown}"
SESSION_ID="$(printf '%s' "$SESSION_ID" | tr -c 'a-zA-Z0-9._-' '_')"
```

Notes:
- Read stdin **before** any other consumer. If a later step needs the JSON
  (e.g. `transcript_path`), parse it from the captured `$HOOK_STDIN`, not a
  second `cat` (stdin is already drained).
- `transcript_path` from the same JSON is the path to the conversation `.jsonl`
  — usable for inspecting the turn programmatically, or (its oldest mtime) as a
  *rough* session-start proxy, though a per-session marker keyed on `session_id`
  is more reliable than mtime on Windows/Git-Bash.
- Keep `${PPID}` only as a last-resort fallback for logging-only fields where
  instability is harmless (e.g. a bypass-log `session_id` column). Never for a
  field that gates behaviour.

## Generalises to

All Claude Code hook events that receive the common input fields
(`session_id`, `transcript_path`, `cwd`, `permission_mode`, `hook_event_name`):
`PreToolUse`, `PostToolUse`, `Stop`, `SubagentStop`, `UserPromptSubmit`,
`Notification`, etc. When a hook needs identity or context beyond the documented
env vars, the answer is almost always "parse stdin," not "find the env var."

## Symptom to recognise

A hook that's supposed to do something "once per session" or "after the session
reaches age N" but fires every turn / resets every turn. Grep the hook for
`CLAUDE_SESSION_ID` — if present, it's the phantom-env-var trap; the code is
running on the PPID fallback. Replace with the stdin-parse snippet above.
