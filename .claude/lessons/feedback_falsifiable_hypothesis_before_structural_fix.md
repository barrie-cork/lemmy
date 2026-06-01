---
name: Verify the hypothesis behind a structural-fix DQ before queueing the fix
description: When a decision-queue entry proposes a structural fix (TypeScript patch, harness change, daemon code-path edit) and names a specific code path as the defect site, the DQ's own RCA is a hypothesis — not a contract. Spend ≤30 min falsifying it BEFORE picking an option-a/b/c routing path. The 2026-05-21 DQ #338 incident burned ~3 hours on a wrong-premise option-a investigation that ≤30 min of grep would have inverted.
type: feedback
---

## TL;DR

When a DQ proposes a structural fix (TypeScript patch, harness change, daemon-side defect) and names a specific code path (file, function, prompt) as the defect site, spend ≤30 min verifying the hypothesis behind the failure attribution **before** committing to the fix path. Treat the DQ's RCA as the leading hypothesis, not a closed case. If the named code path does NOT contain the suspect operation, surface the falsification to the user before any structural work begins.

## Why this matters (DQ #338 incident 2026-05-21)

DQ #338 attributed a destructive `git reset --hard origin/phase-v1-federation-inbound-c` (on daemon-local `governance-v0`) to **"daemon's finalize-merge step issued an unintended reset."** User selected option-a (structural investigation of `/opt/junior-src/src/daemon/executor.ts`). After ~30 min of forensics:

- `buildFinalizePrompt` body contains zero `reset --hard` instructions.
- `executor.ts` + `finalize-lock.ts` + `git.ts` have zero `git reset` calls (grep clean across `/opt/junior-src/src/` and `/opt/junior-src/dist/`).
- Task #399 finalize bash log shows clean STEP 3 (`git checkout governance-v0 → git merge --ff-only origin/governance-v0 → git merge --no-ff <worker>`).
- All recent Junior task logs (jobs 395-402) contain zero `reset --hard` calls.
- `who` empty on the daemon; `last -n 20 -F` showed last interactive SSH login as April 10. No human at a shell on the daemon at the incident timestamp.

**The destructive reset came from outside the Junior daemon entirely.** Sub-agent re-investigation 2026-05-22 confirmed the real vector via smoking-gun transcript citation: a lane-dedicated laptop Claude Code session (CWD: `brehon-fork-fed-in-c`) ran `ssh homeserver "...git reset --hard origin/<phase>"` as a "lossless FF" routine against the shared daemon checkout. Daemon HEAD had been transiently switched to `governance-v0` by Junior #399's finalize STEP 3; the lane agent's reset hit `governance-v0` instead of the named phase branch.

**Cost of the wrong-premise investigation: ~3 hours of session displacement** before the falsification gate would have inverted the recommendation. Without the gate, every future structural-fix DQ inherits the same risk — the DQ's `context` field is confidently written, the routing tree picks an option based on the named defect site, and the structural fix is scoped to a code surface that turns out to be innocent.

## When to apply

Every time a `(blocker, pending)` DQ proposes a structural fix AND names a specific code path as the defect site. The gate fires AFTER reading the DQ's `question` / `options` / `context` and BEFORE picking a routing path (advisor-answer / catch-fire / user-relay per `.claude/rules/advisor-orchestrator.md` §5.4).

Triggering signatures in the DQ entry:

- `options[]` includes a structural fix (TypeScript patch, harness change, daemon code-path edit, prompt rewrite, hook change).
- `context` field names a specific code path (file, function, prompt, script) as the defect site.
- The fix would touch code outside the current sub-phase scope (harness, daemon, infrastructure).

Does NOT fire when:

- The DQ is purely judgment (ADR-affecting decision; scope-change vote; deferment).
- The DQ is a `validate-pending` / `validate-pending-laptop` mutation request — those are mechanical per §G4 classifier, not structural-fix decisions.
- The structural fix has already been validated by a sibling DQ or prior retro (no new hypothesis introduced).

## How to apply

Three mechanical steps, total ≤30 min:

1. **Grep the named code path for the suspect operation.** Example: DQ names `/opt/junior-src/src/daemon/executor.ts finalize-merge code path` as the defect site for a destructive `reset --hard`; run `ssh homeserver "grep -rn 'reset --hard\|reset' /opt/junior-src/src/ /opt/junior-src/dist/"`. Zero matches → falsification candidate. Non-zero matches → hypothesis still alive.

2. **Read the relevant subagent's task log for the actual tool calls issued.** Filter to the relevant tool class — for git ops, parse Bash tool calls; for filesystem mutations, parse Edit/Write calls. Pattern: `python3 -c "import json; [json.loads(l) for l in open('<log>')]"` with filter on `tool_use` events. If the named code path's execution path never issued the suspect operation, falsification candidate.

3. **Check `who` / `last` / `git reflog` on the named host or worktree.** Rules out interactive-session contamination. Empty `who` + months-old `last` rules out a typed shell command. `git reflog HEAD` + `git reflog <branch>` shows the timestamp window — if the suspect operation lands during a window with no Junior process active and no interactive session, the source is elsewhere (most often a laptop-side Claude Code Bash tool call against the host via SSH).

If steps 1+2+3 falsify the hypothesis: surface to the user via `AskUserQuestion` with the candidate vectors (laptop-side ssh-reset is the highest-probability bucket for destructive-reset class) and a revised options list. Do NOT advance to the original option-a structural fix.

If steps 1+2+3 confirm the hypothesis: proceed with the originally-picked routing path.

## Hard refusals

- **NEVER queue a structural-fix Junior task on a DQ whose hypothesis you have not falsification-tested.** The fix is scoped to the named code surface; if that surface is innocent, the Junior task ships a code change that doesn't address the actual defect AND displaces the real vector's investigation window.

- **NEVER skip step 1 just because the DQ context is detailed.** Detail in the `context` field is correlated with the DQ author's confidence, NOT with the hypothesis's truth. DQ #338 had ~600 chars of detailed context naming specific reflog SHAs + finalize sequence + recurrence claims; every detail was true; the attribution (daemon's finalize) was still wrong.

- **NEVER expand the falsification window past 30 min without surfacing to the user.** The gate exists to bound cost; if 30 min of grep + log inspection produces ambiguous results, that's itself a signal — surface "investigation inconclusive; here's what I found" rather than continuing to dig.

## Cross-references

- `.claude/rules/advisor-orchestrator.md` §5.4 "DQ triage decision tree" — the gate is integrated as a sub-bullet of step 1 (read entry → falsify if structural-fix-class → decide routing).
- `.claude/lessons/feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md` — the lesson originally written under the falsified hypothesis; rewritten 2026-05-22 with the correct vector after sub-agent forensics.
- `.claude/lessons/feedback_verify_automated_reviewer_claims_against_compiler.md` — sibling pattern: CR / Copilot trait/type claims are hypotheses; compile-check the proposed change BEFORE triaging.
- `.claude/lessons/feedback_verify_branch_diff_vs_trunk_before_concluding_code_missing.md` — sibling pattern applied to branch state: a "phantom PR / code is missing" conclusion is a hypothesis; verify `git diff trunk...branch` (content identity, not SHA ancestry) BEFORE re-dispatching. PR #172 2026-06-01.
- DQ #338 (2026-05-21; resolved option-b 2026-05-22) — incident origin; full RCA + structural-fix recipe in the entry's `answer` field.
- Sub-agent forensic discipline (`Agent` tool with `general-purpose` subagent_type) — the 5-min investigation that returned the smoking-gun transcript citation; preserved ~60 min of laptop-side Windows-path / jsonl-parsing burden that the parent session would have been bad at.
- `.claude/rules/multi-lane-worktree.md` — the broader pattern (shared `.git/` across worktrees + concurrent sessions) within which the destructive-reset incident occurred.
