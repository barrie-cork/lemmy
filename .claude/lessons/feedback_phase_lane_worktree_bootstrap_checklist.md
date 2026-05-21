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
5. `cp .mcp.json.example .mcp.json` — the example carries the canonical absolute `PROJECT_MEMORY_DB` (`C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`) per `feedback_pmd_cross_lane_canonical_db.md`. Verify: `grep PROJECT_MEMORY_DB .mcp.json` must show that canonical absolute path, not a relative one.
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
