# Brehon hooks

This directory contains shell scripts wired to Claude Code lifecycle events
via `.claude/settings.json`. Each script receives the event payload as JSON
on stdin, and signals decisions through stdout/stderr + exit code per
[hooks docs](https://code.claude.com/docs/en/hooks).

## Inventory

| Script                      | Event              | Matcher           | Purpose                                                                                              |
| :-------------------------- | :----------------- | :---------------- | :--------------------------------------------------------------------------------------------------- |
| `prp-ralph-stop.sh`         | `Stop`             | (any)             | Keeps the PRP Ralph autonomous loop running between iterations until `<promise>COMPLETE</promise>`. |
| `check-cargo-pipe.sh`       | `PreToolUse`       | `Bash`            | Blocks `cargo … \| tail/head/grep/…` per `.claude/rules/cargo-output-capture.md`. Exit 2 with fix.  |
| `refuse-ssh-reset-hard-shared-checkout.sh` | `PreToolUse` | `Bash`        | Refuses `ssh ...homeserver "...git reset --hard origin/<phase-or-trunk>"` against `/srv/brehon-fork`. Per `.claude/lessons/feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md` (DQ #338). Exit 2 with `update-ref` recipe. Escape hatch: `BREHON_ALLOW_SSH_RESET_HARD=1`. |
| `inject-dq-state.sh`        | `UserPromptSubmit` | (any)             | Injects current decision-queue + task-hopper state when anything is pending. Silent in steady state. |
| `pre-phase-audit.sh`        | `SessionStart`     | `startup`/`resume`| On `phase-*` branches, reminds the agent to run the 4 wrapper probes from `pre-phase-harness-audit.md` if not yet completed. |
| `pmd-canonical-guard.sh`    | `SessionStart`     | `.*`              | Stderr WARN if this lane's `.mcp.json` `PROJECT_MEMORY_DB` differs from the canonical absolute path. WARN-not-FAIL. Per `.claude/rules/pmd-invariants.md` §5. |
| `session-start-multi-lane-check.sh` | `SessionStart` | `.*`         | Stderr WARN if another `phase-v1-*`/`phase-v2-*`/`phase-brehon-*` worktree's branch tip was advanced within 30 min. WARN-not-FAIL. Per `.claude/rules/advisor-orchestrator.md` §1. |

> **Inventory drift note (2026-05-22):** the table above is partial — `.claude/hooks/` contains ≥15 scripts as of this date but only ~7 are documented here. See `watch_hook_dir_audit_pending` PMD memory for the pending `harness-audit` skill invocation that should produce a complete inventory + redundancy + latency report.

## Setup

The hooks are wired in `.claude/settings.json` (committed), so they take
effect on next session start. No per-developer configuration required.

To temporarily disable all hooks for a session, set `"disableAllHooks": true`
in `.claude/settings.local.json`.

## Testing each hook manually

```bash
# check-cargo-pipe.sh — should block (exit 2) and print the fix message
echo '{"tool_name":"Bash","tool_input":{"command":"cargo build | tail -40"}}' \
  | bash .claude/hooks/check-cargo-pipe.sh
echo "exit: $?"

# check-cargo-pipe.sh — should allow (exit 0, no output)
echo '{"tool_name":"Bash","tool_input":{"command":"cargo build > /tmp/x.log 2>&1; tail -40 /tmp/x.log"}}' \
  | bash .claude/hooks/check-cargo-pipe.sh
echo "exit: $?"

# refuse-ssh-reset-hard-shared-checkout.sh — should block (exit 2) and print the fix message
echo '{"tool_name":"Bash","tool_input":{"command":"ssh homeserver \"cd /srv/brehon-fork && git reset --hard origin/phase-v1-federation-inbound-c\""}}' \
  | bash .claude/hooks/refuse-ssh-reset-hard-shared-checkout.sh
echo "exit: $?"

# refuse-ssh-reset-hard-shared-checkout.sh — should allow (exit 0, no output) for update-ref
echo '{"tool_name":"Bash","tool_input":{"command":"ssh homeserver \"cd /srv/brehon-fork && git update-ref refs/heads/governance-v0 origin/governance-v0\""}}' \
  | bash .claude/hooks/refuse-ssh-reset-hard-shared-checkout.sh
echo "exit: $?"

# inject-dq-state.sh — emits JSON only when DQ pending or hopper escalated
echo '{}' | bash .claude/hooks/inject-dq-state.sh
echo "exit: $?"

# pre-phase-audit.sh — emits reminder JSON only on phase-* branches without flag
echo '{"source":"startup"}' | bash .claude/hooks/pre-phase-audit.sh
echo "exit: $?"
```

After running all 4 wrapper probes from `pre-phase-harness-audit.md`, mark
the audit complete so the SessionStart reminder stops firing for this branch:

```bash
touch .claude/audit-$(git rev-parse --abbrev-ref HEAD | tr / -)-complete.flag
```

## Ralph loop specifics

The Stop hook (`prp-ralph-stop.sh`) keys off `.claude/prp-ralph.state.md`:

1. `/prp-ralph <plan>` creates the state file with iteration counter.
2. On every session-stop attempt, the hook checks for the state file.
3. If state exists and `<promise>COMPLETE</promise>` not in last assistant message:
   - Increments iteration counter
   - Feeds the plan execution prompt back to Claude
   - Loop continues
4. If completion promise detected OR max iterations reached:
   - State file is removed
   - Session exits normally

Manual cancellation: `/prp-ralph-cancel` or `rm .claude/prp-ralph.state.md`.

## Troubleshooting

### Hook not triggering

```bash
# Confirm wiring
jq '.hooks' .claude/settings.json

# Confirm executability
ls -la .claude/hooks/

# Tail the debug log (start session with: claude --debug-file /tmp/cc.log)
tail -f /tmp/cc.log
```

### Hook is too noisy

`inject-dq-state.sh` is designed to be silent when nothing is pending.
If it fires every prompt, check `.claude/decision-queue.json` for
unresolved entries you forgot to move from `pending` to `resolved`.

`pre-phase-audit.sh` is one-shot per branch — if it reminds you every
session, you haven't created the audit-complete flag for the branch yet.

### Hook output causes JSON parse errors

Hooks run in non-interactive shells. If your `~/.bashrc` echoes anything
unconditionally (e.g. `echo "Shell ready"`), wrap it in `[[ $- == *i* ]]`.
See [hooks reference — JSON validation](https://code.claude.com/docs/en/hooks-guide#json-validation-failed).
