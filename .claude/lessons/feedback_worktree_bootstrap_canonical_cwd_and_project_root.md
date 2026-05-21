---
name: worktree-add pre-flight must check canonical CWD; .mcp.json bootstrap must atomically rewrite PROJECT_ROOT
description: Two friction points observed in v1-rls-r1 worktree bootstrap 2026-05-20. Canonical brehon-fork checkout drifted off governance-v0 onto a phase branch without notice; cp of .mcp.json into the new lane left PROJECT_ROOT pointing at the canonical path. Both caught only by mid-bootstrap verify steps the bootstrap procedure currently doesn't mandate.
type: feedback
status: WATCH (1 occurrence each — promote to .claude/rules/multi-lane-worktree.md §Lifecycle if either recurs)
---

# Two slips observed during the v1-rls-r1 worktree bootstrap (2026-05-20)

Both are quick to fix structurally; neither is yet a confirmed pattern. Filed
as forward-looking observations so the next bootstrap doesn't repeat them.

## Slip 1 — pre-worktree-add must check canonical CWD first

**What happened.** Advisor session was preparing to run `git worktree add
../brehon-fork-rls-r1 -b phase-v1-rls-r1 origin/governance-v0`. User asked
"is it safe to proceed?" The pre-flight I ran included target-dir-exists,
branch-name-collision, and `git status --short`. It did NOT include
`git branch --show-current` against the canonical checkout. Running it
revealed the canonical was on `phase-brehon-conformance-audit`, not
`governance-v0`. This violates `.claude/rules/multi-lane-worktree.md`
§"Layout" — canonical is reserved for `governance-v0` only.

**Cause.** The canonical brehon-fork worktree drifted onto a phase branch
in a prior session (likely a `git checkout phase-brehon-conformance-audit`
that should have happened in a lane-dedicated worktree). The drift was
silent because nothing in the worktree-add ritual checks it.

**Why it would have bitten.** If I'd proceeded without the verify,
`git worktree add ... origin/governance-v0` would have worked (the new
worktree forks from `origin/governance-v0`, not from canonical's local
HEAD) — BUT the canonical session would still be writing to phase
branches as if it were a lane-dedicated worktree, which is the exact
"shared `.claude/decision-queue.json` race" pattern multi-lane-worktree.md
exists to prevent.

**Structural fix candidate.** The bootstrap snippet in
`multi-lane-worktree.md` §"Lifecycle" Step 1 should prepend:

```bash
# Hard gate: canonical MUST be on governance-v0 before any worktree-add.
test "$(git -C C:/Users/barri/Developer/brehon-fork rev-parse --abbrev-ref HEAD)" = "governance-v0" \
  || { echo "ABORT: canonical is on $(git ...) — restore to governance-v0 first"; exit 1; }
```

Watch for a second occurrence before promoting this to a rule edit.

## Slip 2 — `.mcp.json` cp must atomically rewrite PROJECT_ROOT

**What happened.** After `cp .mcp.json ../brehon-fork-rls-r1/.mcp.json`,
the new lane's `.mcp.json` carried the canonical `PROJECT_ROOT` value
(`C:\Users\barri\Developer\brehon-fork`) instead of the lane-specific
path (`C:\Users\barri\Developer\brehon-fork-rls-r1`). Caught only by a
`grep -F "PROJECT_ROOT" ../brehon-fork-rls-r1/.mcp.json` verify and a
follow-up `Edit`.

**Why both fields exist + which is which.** Per
`.claude/lessons/feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`
+ `multi-lane-worktree.md` §"PMD is cross-lane shared":

- `PROJECT_MEMORY_DB` MUST stay canonical across lanes (the cross-lane
  invariant). A relative or per-lane value strands lessons + retros in a
  lane-local DB.
- `PROJECT_ROOT` MUST change per lane. A canonical `PROJECT_ROOT` in a
  lane-local `.mcp.json` causes the project-memory MCP to resolve file
  paths against the wrong worktree, which masks lane attribution in
  every `memory_write_eval` from that lane.

These two fields drift in opposite directions on cp — that is the
footgun. Plain `cp` is wrong for half of the env block.

**Structural fix candidate.** The bootstrap snippet should use `sed` or
`jq` to do the cp + rewrite as a single step:

```bash
LANE_DIR="C:/Users/barri/Developer/brehon-fork-${LANE_SLUG}"
LANE_DIR_W=$(echo "$LANE_DIR" | sed 's|/|\\\\|g')   # PowerShell-style backslashes
jq --arg root "$LANE_DIR_W" \
  '.mcpServers["project-memory"].env.PROJECT_ROOT = $root' \
  C:/Users/barri/Developer/brehon-fork/.mcp.json \
  > "$LANE_DIR/.mcp.json"
```

Or — if jq's not desired in the bootstrap path — at minimum a mandatory
`Edit`/`sed -i` step in the bootstrap checklist immediately after the cp,
not as a "verify and rewrite if needed" optional step.

Watch for a second occurrence before promoting this to a rule edit.

## Until both promote

If you cut a new lane worktree, the safe ritual is:

1. `git -C C:/Users/barri/Developer/brehon-fork rev-parse --abbrev-ref HEAD` — MUST be `governance-v0`.
2. `git worktree add ../brehon-fork-<lane> -b phase-v1-<lane> origin/governance-v0`.
3. `cp .mcp.json .env .claude/settings.local.json ../brehon-fork-<lane>/` (and the corresponding paths).
4. `git submodule update --init --recursive` inside the new lane.
5. **Verify** + **rewrite** `PROJECT_ROOT` in the lane's `.mcp.json`.
6. `grep PROJECT_MEMORY_DB ../brehon-fork-<lane>/.mcp.json` — confirm it's the canonical absolute path.
7. `git status --short` inside the new lane — confirm clean.

## See also

- `.claude/rules/multi-lane-worktree.md` §"Layout", §"Lifecycle",
  §"PMD is cross-lane shared".
- `.claude/lessons/feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`
  — the SessionStart-guard fix this lesson would benefit from.
- `.claude/lessons/feedback_settings_local_json_worktree_bootstrap.md`
  — adjacent bootstrap-fixed-files-list lesson.
- `.claude/lessons/feedback_pmd_cross_lane_canonical_db.md` — the
  `PROJECT_MEMORY_DB` cross-lane invariant.
- PMD eval id 441 (2026-05-20) — the retro that surfaced these.
