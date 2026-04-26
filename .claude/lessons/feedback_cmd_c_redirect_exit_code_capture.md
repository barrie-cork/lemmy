---
name: cmd //c redirect + exit code capture on Windows wrappers
description: When invoking Brehon `.bat` wrappers from bash via `cmd //c "...> log 2>&1"`, the exit code lands on the cmd.exe invocation — but a FOLLOWING `echo "$?"` runs in a different bash subshell (especially under `run_in_background: true`) and captures a different $?. Must capture in same bash process or read the background-task completion notification's exit code.
type: feedback
originSessionId: e33ef309-edff-48cb-816d-32cff3bd75bd
---
When running a Windows batch wrapper from Claude Code's Bash tool with output
redirection inside the `cmd //c` string, DO NOT check `$?` in a separate
subsequent line:

```bash
# ❌ WRONG — $? captures the wrong subshell
cmd //c "scripts\\brehon\\cargo-check.bat ... > /tmp/log 2>&1"
echo "exit: $?"      # runs in a new shell; always 0
```

The wrapper propagates `errorlevel` correctly (every `scripts/brehon/cargo-*.bat`
uses `setlocal enabledelayedexpansion` + `exit /b !errorlevel!` post-
`bb254e733` on 2026-04-18). But `run_in_background: true` starts a fresh
bash process each invocation, and the `echo "$?"` line runs AFTER the
`cmd //c` call has released its exit status.

**Why:** `How to apply:` two correct forms, both used regularly.

### ✅ Foreground — capture immediately, same bash process

```bash
cmd //c "scripts\\brehon\\cargo-check.bat ... > /tmp/log 2>&1"
STATUS=$?
echo "exit: $STATUS"
```

All three lines must be in the SAME `Bash` tool call (one `command` string).

### ✅ Background — trust the task-completion notification

With `run_in_background: true`, Claude Code automatically reports the
wrapper's exit code in the completion notification (e.g.
`"Background command ... completed (exit code 101)"`). Read that, do NOT
add a second bash command that echoes `$?`.

**When this matters:** the pre-phase harness audit (`.claude/rules/pre-phase-harness-audit.md`)
has Probe 4 explicitly to catch wrapper-side exit-code masking. If the
probe appears to false-fail (exit 0 on a known-bad invocation), suspect
the capture wiring BEFORE concluding the wrapper is broken. Re-run
foreground with a single `STATUS=$?` line to verify.

**Incident:** v1-AD-a iteration 1 on 2026-04-19 evening. Probes 4a and
4b both initially reported exit 0 under `run_in_background: true` — I
almost opened a DQ entry for "fix wrapper" before noticing the logs
showed cargo's error message correctly. Direct foreground re-invocation
confirmed exit 101. The wrapper was fine; the capture was wrong.

**Related memory:** `feedback_batch_goto_eof_clobbers_errorlevel.md` —
the wrapper-side bug class this was trying to catch. This memory is the
orthogonal capture-side bug class.
