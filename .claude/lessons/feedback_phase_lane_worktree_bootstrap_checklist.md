---
name: Lane-bootstrap checklist — durable record of per-worktree setup steps
description: Each new lane-dedicated worktree under .claude/rules/multi-lane-worktree.md requires a small set of per-worktree setup steps (copy .mcp.json, copy / create .claude/settings.local.json, wire any SessionStart hooks the lane depends on). This checklist is the durable record so a new lane can be brought up without re-deriving each step from scattered lesson files. Per the v1-rls-r1 sub-phase + DQ #301.
type: feedback
---

# Lane-bootstrap checklist

## Why this lesson exists

Per `feedback_settings_local_json_worktree_bootstrap.md`, `.claude/settings.local.json` is per-worktree and gitignored; per `feedback_pmd_cross_lane_canonical_db.md` and `.claude/rules/multi-lane-worktree.md`, `.mcp.json` is also per-worktree and gitignored. Cumulative state means each new lane has 3-5 manual bootstrap steps that do not propagate via git. Without a checklist, each new-lane operator re-derives the steps from scattered lessons and rules, re-learning the same constraints (canonical-PMD path, settings allow-list, SessionStart hook wiring) that prior phases already paid to discover.

## Checklist

1. `cd C:/Users/barri/Developer/brehon-fork` (canonical checkout — ensures the worktree add command has the right git common dir).
2. `git fetch origin <phase-branch>`
3. `git worktree add ../brehon-fork-<lane> <phase-branch>`
4. `cd ../brehon-fork-<lane>`
5. **Write `.mcp.json` with real paths** — do NOT copy `.mcp.json.example`; the example carries `/path/to/` placeholder strings that produce `-32000` MCP connection failures at session start. Instead, write the file directly with the correct absolute paths (see template in `~/.claude/commands/roadmap-next.md` Phase 5). Required servers: `project-memory`, `junior-brehon`, `ref-context`, `tavily`. **`rust-analyzer-mcp` is NOT included** — the `rust-analyzer-mcp` wrapper binary is not installed; `rust-analyzer.exe` alone does not expose an MCP interface.

   **PMD verification depends on topology — read this before chasing a "missing path" finding (2026-06-04):**
   - **HTTP-daemon topology (2026-05-30+, current):** `project-memory` is an HTTP server — the `.mcp.json` block is `{"type": "http", "url": "http://localhost:11435/mcp"}` (Tailscale: `http://100.104.171.26:11435/mcp`). There is **NO `PROJECT_MEMORY_DB` env line and no `OLLAMA_URL`** — the server manages the DB + embeddings server-side. Verify with `grep '"url".*11435' .mcp.json` (must match). **The absence of a `PROJECT_MEMORY_DB` line is CORRECT, not a failure** — do not add one, and do not treat its absence as a bootstrap error. Per `pmd-invariants.md` #1 "Current topology". (A simple copy of the canonical `.mcp.json` already carries the right HTTP block — the fastest path is `cp C:/Users/barri/Developer/brehon-fork/.mcp.json .mcp.json`.)
   - **Legacy stdio topology (pre-2026-05-30, historical):** `project-memory` was a stdio server whose `env` set `PROJECT_MEMORY_DB` + `OLLAMA_URL`. There, `grep PROJECT_MEMORY_DB .mcp.json` had to show `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`. This check is obsolete under the HTTP daemon — kept here only so an old `.mcp.json` is recognisable.
6. **(NEW — v1-rls-r1)** Wire `pmd-canonical-guard.sh` as a `SessionStart` hook in `.claude/settings.local.json`. Create the file if absent; merge into it if it already exists — do not overwrite existing sections (`permissions`, `env`, etc). Paste the following snippet, sequenced BEFORE any existing `SessionStart` entry that calls `pre-phase-audit.sh` (the canonical guard must surface PMD-path drift before phase audit assumes the canonical PMD is reachable):

   ```json
   {
     "hooks": {
       "SessionStart": [
         {
           "matcher": ".*",
           "hooks": [
             {"type": "command", "command": "bash .claude/hooks/pmd-canonical-guard.sh"}
           ]
         }
       ]
     }
   }
   ```

7. Open Claude Code in the new worktree CWD. Verify the SessionStart banner shows no `pmd-canonical-guard.sh` WARN. A WARN means the `.mcp.json` `PROJECT_MEMORY_DB` still points at a wrong path — fix it and restart the MCP before writing any retros.

8. **(NEW — v1-federation-inbound-c session 2026-05-21)** Programmatic verification that step 6 actually landed. Step 7 catches `.mcp.json` mispoints (the guard fires and surfaces a WARN); it does NOT catch the case where the wiring itself is missing (no WARN appears because the guard never ran). The two failure modes are distinct: mispointed PMD = guard ran + surfaced; missing wiring = guard never ran + silence. Run from the new lane CWD:

   ```bash
   python -c "
   import io, json
   s = json.load(io.open('.claude/settings.local.json', encoding='utf-8'))
   ss = s.get('hooks', {}).get('SessionStart', [])
   wired = any(
       'pmd-canonical-guard.sh' in h.get('command', '')
       for entry in ss for h in entry.get('hooks', [])
   )
   assert wired, 'FAIL: pmd-canonical-guard.sh SessionStart wiring missing — DQ #301 dual-wire incomplete; re-apply step 6'
   print('OK: pmd-canonical-guard.sh wired at SessionStart')
   "
   ```

   Expected output: `OK: pmd-canonical-guard.sh wired at SessionStart`. Any other output (FAIL assertion, JSON parse error, file-not-found) means step 6 was skipped or the file was clobbered — re-apply step 6 and re-run this check before proceeding. Per `feedback_python_utf8_encoding_windows.md`, the `io.open(..., encoding='utf-8')` is mandatory on Windows — bare `json.load(open(...))` will hit the cp1252 codec on non-ASCII content (recurred during v1-federation-inbound-c session 2026-05-21 when a DQ snapshot read crashed on a non-ASCII char at byte 41041).

9. **(NEW — 2026-05-22, post-RT-r2 boundary incident)** Wire `session-start-multi-lane-check.sh` as a `SessionStart` hook in the same `.claude/settings.local.json`. Add it as a second `hooks` entry inside the existing `SessionStart` array (do NOT replace the `pmd-canonical-guard.sh` entry — both run on every session start). Order: pmd-canonical-guard first (PMD-path drift is more critical), multi-lane-check second. Final `SessionStart` array shape:

   ```json
   {
     "hooks": {
       "SessionStart": [
         {
           "matcher": ".*",
           "hooks": [
             {"type": "command", "command": "bash .claude/hooks/pmd-canonical-guard.sh"},
             {"type": "command", "command": "bash .claude/hooks/session-start-multi-lane-check.sh"}
           ]
         }
       ]
     }
   }
   ```

10. **(NEW — 2026-05-22)** Programmatic verification that step 9 actually landed (mirrors step 8's belt-and-braces for `pmd-canonical-guard.sh`). Run from the new lane CWD:

    ```bash
    python -c "
    import io, json
    s = json.load(io.open('.claude/settings.local.json', encoding='utf-8'))
    ss = s.get('hooks', {}).get('SessionStart', [])
    wired = any(
        'session-start-multi-lane-check.sh' in h.get('command', '')
        for entry in ss for h in entry.get('hooks', [])
    )
    assert wired, 'FAIL: session-start-multi-lane-check.sh SessionStart wiring missing — re-apply step 9'
    print('OK: session-start-multi-lane-check.sh wired at SessionStart')
    "
    ```

    Expected output: `OK: session-start-multi-lane-check.sh wired at SessionStart`. Any other output means step 9 was skipped or the file was clobbered — re-apply step 9 and re-run.

11. **(NEW — 2026-05-22, DQ #338 option-b)** Verify the tracked `PreToolUse` hook `refuse-ssh-reset-hard-shared-checkout.sh` is registered in this lane's `.claude/settings.json`. This hook is wired in the TRACKED `settings.json` (not `settings.local.json`), so it activates automatically for every lane checking out the tracked file — but a lane that ever ran `disableAllHooks: true` in its `settings.local.json` (or pre-dates the 2026-05-22 commit `8a392f217`+) would not have the protection. Programmatic check, from the new lane CWD:

    ```bash
    python -c "
    import io, json
    s = json.load(io.open('.claude/settings.json', encoding='utf-8'))
    pre = s.get('hooks', {}).get('PreToolUse', [])
    wired = any(
        'refuse-ssh-reset-hard-shared-checkout.sh' in h.get('command', '')
        for entry in pre for h in entry.get('hooks', [])
    )
    assert wired, 'FAIL: refuse-ssh-reset-hard-shared-checkout.sh PreToolUse wiring missing — re-pull governance-v0 or re-apply commit 8a392f217+'
    print('OK: refuse-ssh-reset-hard-shared-checkout.sh wired at PreToolUse')
    "
    ```

    Expected output: `OK: refuse-ssh-reset-hard-shared-checkout.sh wired at PreToolUse`. Hook refuses `ssh ...homeserver "...git reset --hard origin/<phase-or-trunk>"` against `/srv/brehon-fork`; safer alternative is `git update-ref refs/heads/<branch> origin/<branch>`. Escape hatch: `BREHON_ALLOW_SSH_RESET_HARD=1`. See `.claude/lessons/feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md` for the incident + RCA.

12. **(NEW — 2026-05-24, task #450 stale-base incident)** Before queuing any **planning
    Junior task**, fast-forward the daemon's local `governance-v0` ref to match origin.
    Without this, the planning worker branches from a stale daemon-local ref and may read
    an old brief with the same filename, producing a plan for a closed phase.

    ```bash
    ssh homeserver "cd /srv/brehon-fork && git fetch origin && \
      git update-ref refs/heads/governance-v0 origin/governance-v0 && \
      git log --oneline governance-v0 | head -3"
    ```

    Verify the top commit matches the latest `governance-v0` tip seen on the laptop
    (`git log --oneline origin/governance-v0 | head -1`). If they differ, the fetch
    failed — check SSH + GitHub connectivity before re-queuing.

    **Root cause:** the daemon's finalize-merge step fast-forwards its local
    `governance-v0` only when it merges a completed Junior worktree branch. If no Junior
    task ran recently (session boundary, phase close, long idle), the daemon's local ref
    drifts behind origin. The advisor's `git push origin governance-v0` writes to origin
    but does NOT update the daemon's local ref — those are two separate git objects.

    This step is **planning-only** (impl-task briefs are committed to the phase branch
    which Junior checks out by name from origin; planning briefs land on `governance-v0`
    which Junior reads from the daemon's LOCAL ref).

## DQ #301 dual-wire (v1-rls-r1 ships)

Step 6's wiring MUST be applied in BOTH locations:

- The **canonical** `C:/Users/barri/Developer/brehon-fork/.claude/settings.local.json` — durable across phase-v1-rls-r1 worktree removal; ensures the canonical checkout session also benefits from the guard on every future session start.
- The **lane-dedicated** `C:/Users/barri/Developer/brehon-fork-rls-r1/.claude/settings.local.json` — immediate dogfood for the r1 session itself.

Cost ≈ 30 seconds extra; belt-and-braces. Future lanes apply the wiring at step 6 per this checklist; the canonical wiring is the durable record that outlives any individual lane worktree.

Note: both `settings.local.json` files are gitignored — this is a manual hand-off the advisor / user applies post-merge. The JSON snippet above is the authoritative form to paste.

## See also

- `feedback_settings_local_json_worktree_bootstrap.md` — per-worktree gitignored discipline + why the allow-list must be copied at bootstrap time.
- `feedback_pmd_cross_lane_canonical_db.md` — canonical-PMD path invariant; hard refusal against editing `retro-check.sh`; diagnosis recipe.
- `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` — the spec for `pmd-canonical-guard.sh`; why a SessionStart WARN rather than a block.
- `.claude/rules/multi-lane-worktree.md` — lane lifecycle + hard refusals around cross-lane DQ writes and shared-checkout phase-branch ops.
- `.claude/rules/pmd-invariants.md` (Task 2) — the five consolidated PMD meta-invariants; invariant #1 is the canonical-path rule this checklist step 5+6 enforces.
- `.claude/hooks/pmd-canonical-guard.sh` (Task 3) — the script step 6 wires; tracks into git so the check logic is shared knowledge while the activation remains per-worktree config.
- `.claude/hooks/session-start-multi-lane-check.sh` (2026-05-22) — the script step 9 wires; detects concurrent advisor session activity via worktree-tip-age heuristic. Companion lessons: `feedback_falsifiable_hypothesis_before_structural_fix.md` (the false-RCA pattern this hook was authored to prevent) + `project_concurrent_advisor_sessions_2026_05_21.md` (the incident).
- `.claude/hooks/refuse-ssh-reset-hard-shared-checkout.sh` (2026-05-22, DQ #338 option-b) — the script step 11 verifies; refuses the bug-class invocation that orphaned plan-merge 2ad835aa4. Wired in TRACKED `settings.json` PreToolUse Bash matcher; verification step exists only to catch lanes that disabled all hooks. Companion lesson: `feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md` (post-2026-05-22 rewrite).
