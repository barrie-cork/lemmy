#!/bin/bash

# PreToolUse hook — REFUSE `ssh ...homeserver "...git reset --hard origin/<phase-or-trunk>"`
# against the shared Junior daemon checkout (`/srv/brehon-fork`).
#
# Why: see `.claude/lessons/feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md`
# (post-2026-05-22 rewrite). `git reset --hard <target>` operates on the
# CURRENT branch checked out at `/srv/brehon-fork`, not the named target ref.
# Daemon HEAD is shared mutable state: every Junior finalize-merge subagent
# transiently switches it (STEP 3 of buildFinalizePrompt does
# `git checkout governance-v0`); every prior laptop session may have left it
# on any branch. A lane session attempting to "fast-forward" a phase ref via
# `reset --hard origin/<phase>` will silently clobber whatever branch HEAD is
# currently on if it happens to differ from the intended target.
#
# 2026-05-21 incident: this exact pattern, run 8x as a "lossless FF" routine,
# orphaned plan-merge 2ad835aa4 on the 1 invocation that hit during a
# Junior-finalize HEAD-switch window. Transcript:
# brehon-fork-fed-in-c\a6097edd-663b-4770-b73a-816a012b6d67.jsonl uuid
# cf9c79f1-a709-41ea-a920-dc643a2dd0a5 ts 2026-05-21T19:58:17.028Z.
#
# This hook BLOCKS (exit 2) rather than WARNs because the safer alternative
# (`git update-ref refs/heads/<branch> origin/<branch>`) is mechanical to
# apply and the cost of a wrong reset is high (orphaned commit + ~25min
# recovery). Unlike `pmd-canonical-guard.sh` which is WARN-not-FAIL because
# config-fix is the user's job, this defect is fully mitigable by changing
# the command.
#
# Escape hatch: env var BREHON_ALLOW_SSH_RESET_HARD=1 lets the call through
# for legitimate edge cases (you actually do want to clobber the daemon's
# local ref on purpose). Loud refusal forces conscious opt-in.
#
# Pattern scope (refuses):
#   ssh homeserver "cd /srv/brehon-fork && ... git reset --hard origin/phase-v1-*"
#   ssh homeserver "cd /srv/brehon-fork && ... git reset --hard origin/governance-v0"
#   ssh homeserver "cd /srv/brehon-fork && ... git reset --hard origin/phase-brehon-*"
#   any equivalent (`bash`, `ssh -t`, escaped vs unescaped quotes, multi-line)
#
# Pattern scope (allows):
#   ssh homeserver "...git update-ref ..."           (the recommended pattern)
#   ssh homeserver "...git reset --hard HEAD~N"      (local-history rewrite, no remote-tracking)
#   ssh homeserver "...git reset --hard <sha>"       (explicit SHA target — not origin/<branch>)
#   ssh homeserver "...git reset --soft origin/..."  (soft reset; doesn't destroy worktree)
#   git reset --hard origin/governance-v0            (LOCAL laptop reset; not the bug class)

set -euo pipefail

# --- Escape hatch (conscious opt-in) ---
if [ "${BREHON_ALLOW_SSH_RESET_HARD:-}" = "1" ]; then
  exit 0
fi

# --- Read tool input from hook stdin JSON ---
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

# --- Detection pattern ---
#
# Require ALL of:
#   1. `ssh` invocation (any flags, any user@host syntax)
#   2. Reference to the shared daemon checkout (`/srv/brehon-fork` — covers
#      bare path, `cd /srv/brehon-fork`, `git -C /srv/brehon-fork`, etc.)
#   3. `git reset --hard origin/(phase-|governance-v0|phase-brehon-)<rest>`
#
# All three must appear in the same command string. Match is case-sensitive
# (git refs are case-sensitive); whitespace-flexible.
#
# Single-grep with -P (PCRE) handles the lookahead chain cleanly.

if printf '%s' "$COMMAND" | grep -Pq '(?=.*\bssh\b)(?=.*/srv/brehon-fork)(?=.*\bgit\s+reset\s+--hard\s+origin/(phase-|governance-v0))'; then
  cat >&2 <<'EOF'
Blocked by refuse-ssh-reset-hard-shared-checkout hook.

You're about to run `git reset --hard origin/<phase-or-trunk>` against the
shared Junior daemon checkout at /srv/brehon-fork over ssh. This is the bug
class that orphaned plan-merge 2ad835aa4 on 2026-05-21 — see
.claude/lessons/feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md
(post-2026-05-22 rewrite).

WHY THIS IS UNSAFE:
- `git reset --hard <target>` operates on the CURRENT branch checked out at
  /srv/brehon-fork, NOT the named target ref.
- Daemon HEAD is shared mutable state. Every Junior finalize-merge subagent
  transiently switches it (STEP 3 of buildFinalizePrompt:
  `git checkout governance-v0`); every prior laptop session may have left
  it on a different branch than you expect.
- If HEAD is not on the branch named on the right-hand side at the moment
  your reset runs, you will silently clobber whatever branch HEAD IS on,
  pointing it at the target ref's tip. Any unpushed commits on that branch
  are orphaned.
- 2026-05-21 incident: 7 of 8 invocations of this exact pattern were
  lossless because HEAD was on the phase branch by accident; the 1 that
  caught a Junior-finalize HEAD-switch window destroyed a plan-merge.

SAFER ALTERNATIVE (use this instead):

    ssh homeserver "cd /srv/brehon-fork && \
      git fetch origin <branch> && \
      git update-ref refs/heads/<branch> origin/<branch>"

`git update-ref` operates on the NAMED ref. It is lane-safe and
worktree-safe regardless of which branch daemon's main checkout is
currently on, regardless of which Junior worker is mid-finalize. This is
the recipe already documented in
`.claude/lessons/feedback_planner_dq_id_via_origin_not_daemon_local.md`
for the trunk case — apply the same recipe for phase branches.

ESCAPE HATCH (if you really mean to use reset --hard):

    BREHON_ALLOW_SSH_RESET_HARD=1 <your command>

This forces conscious opt-in; the hook will not block. Use only when you
genuinely understand HEAD will be on the named target branch.
EOF
  exit 2
fi

exit 0
