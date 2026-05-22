---
name: Session-start inherited stdout from a /clear-cleared turn anchors the new session into the inherited action
description: When a session opens with inherited stdout from a /clear-cleared prior turn that confidently recommends a next action, the inherited recommendation acts as a structural action anchor — the session-start CWD/lane ritual silently loses to the anchor unless the ritual is surfaced FIRST as a user-visible line BEFORE any tool call. Two independent sessions hit the same defect class within 24h.
type: feedback
---

# Session-start inherited context anchors the new session into the inherited action

## TL;DR

When a Claude Code session opens with inherited stdout from a prior turn that was `/clear`-cleared but still re-presented in the conversation, **the inherited recommendation acts as an action anchor**. The session-start CWD/lane ritual loses to the anchor unless the ritual is surfaced FIRST as a user-visible line before any tool call. This is **structural, not a discipline lapse** — the harness re-presents the inherited stdout to the model, the inherited recommendation is confident and specific, and the ritual's abstract instruction ("run these 3 git commands and verify the lane") competes against perceived momentum from the inherited line.

The fix is to **invert the order**: surface lane status to the user FIRST (as a one-line summary), THEN run any tool call, THEN respond to the inherited recommendation. The hook + ritual + this lesson together convert an implicit anchor into an explicit interrupt.

## Why this matters (recurrence threshold met 2026-05-21–22)

Two independent advisor sessions hit the same defect class within 24h. By the existing `feedback_principles_not_rules.md` recurrence discipline, that crosses the threshold from session-note to lesson.

### Occurrence 1 (2026-05-21, canonical `brehon-fork` session)

- Session opened with `/clear` followed by inherited stdout from a prior turn that confidently recommended *"plan RT-r2 now"* with a detailed 6-step path.
- `.claude/rules/advisor-orchestrator.md` §1 ("Session-start CWD check") instructs running `pwd && git branch --show-current && git worktree list` and verifying the lane.
- The advisor ran `git status / branch / worktree list` but parsed the output for *"is the tree clean"* rather than *"is another lane actively driven."*
- The advisor never read `.claude/agent-activity.json` — the file did not exist in this CWD (infrastructure absent, not consulted).
- Anchored by the inherited recommendation, the advisor pushed `cf93b7ba6` to `governance-v0` while another session was actively driving `phase-v1-federation-inbound-c` + the schema-v3 migration.
- Project memory: `project_concurrent_advisor_sessions_2026_05_21.md`.

### Occurrence 2 (independent, concurrent session, 2026-05-21–22)

- A separate advisor session in the `brehon-fork-fed-in-c` lane independently shipped `e6035f5e1 chore(advisor): v1-retro-followups-r1 plan + 4 dispatch briefs + session retro` whose body includes its own surface-first-ritual edit (different patch, same conclusion).
- That session reached the same diagnosis: **inherited context overrides session-start ritual; the structural fix is to surface lane status FIRST before any tool call.**

Two independent sessions diagnosing the same defect class in the same 24-hour window with the same proposed structural fix is the recurrence-threshold signal — neither session was a lapse; both were reading from the same broken default.

## The mechanism (why this is structural)

1. **The harness re-presents inherited stdout.** Even after `/clear`, the system reminder + conversation start may include the prior turn's stdout. The model sees it as part of "what the user just said," not as "stale context the user already discarded."
2. **The inherited recommendation is confident and specific.** A line like *"plan RT-r2 now: (1) check…, (2) cut bm-cut brief…, (3)…"* reads as a directive. The session-start ritual's instruction (*"run pwd && git branch && git worktree list and verify the lane"*) reads as a precondition check on an otherwise-already-decided action.
3. **The ritual is abstract; the recommendation is concrete.** Concrete beats abstract in token salience. Even an advisor reading the rule diligently treats the lane check as a low-stakes confirmation step, not as a gate that can flip the entire turn.
4. **Models don't reliably distinguish "this was a recommendation the user has cleared" from "this is what the user wants now."** Absent an explicit interrupt, perceived continuity from the inherited stdout wins.

Inherited stdout from a cleared turn **looks like context but acts like a directive.**

## When to apply

Apply at every session start where any of the following holds:

- The system reminder shows `Coordination state: DQ pending: N` (signal that another session may be active).
- `git worktree list` shows ≥2 active worktrees on `phase-v1-*` branches (multi-lane topology).
- Inherited stdout in the conversation frames a confident "next action" — especially one with numbered steps or a named target (a phase id, a task id, a branch name).
- The session opens with `/clear` followed by content that survives the clear (system reminders, slash-command renders, prior-turn stdout).

Apply even when none of those hold but the inherited stdout is *unusually confident* — the heuristic is "does the inherited line read like a directive to me right now?" If yes, treat it as one.

## How to apply

### The surface-first ritual (operational fix)

`.claude/rules/advisor-orchestrator.md` §1 (added 2026-05-22) is the operational fix. Before any tool call in response to the user's first turn:

1. **Surface lane status as a user-visible line.** One line, naming: current CWD, current branch, count of active worktrees, and whether `.claude/agent-activity.json` shows any other session in `mode: write` on the same repo (or its absence).
2. **State the inherited-context observation explicitly.** If the conversation includes inherited stdout from a `/clear`-cleared turn that recommends an action, surface it: *"Inherited stdout from a prior turn recommended X. The current lane status is Y. Confirm before proceeding?"*
3. **Wait for confirmation OR proceed with explicit acknowledgment** that the recommendation is congruent with current lane state. Never silently inherit the recommendation.

### The hook (mechanical signal)

`.claude/hooks/session-start-multi-lane-check.sh` (shipped) is the SessionStart hook that surfaces lane drift as a stderr WARN in the system reminder before the model's first response. When the hook WARNs, the warning is visible to the model in the system reminder — converting an implicit precondition into an explicit interrupt.

The hook + ritual + this lesson form a defense-in-depth chain:

- **Hook** — mechanical signal at the system-reminder level (always fires, machine-precise).
- **Ritual** — model-side discipline that surfaces the lane state to the user BEFORE the first tool call.
- **Lesson** — corpus-level entry that makes the structural nature of the anchor visible at retro time + at brief-author time.

### Hard refusals

1. **Never silently follow an inherited recommendation** without first surfacing the lane state to the user. The cost of one extra user round-trip is much smaller than the cost of pushing to the wrong branch on the wrong lane.
2. **Never treat `git worktree list` output as "clean ✓" without parsing for other-lane activity.** A clean working tree on the canonical checkout does NOT mean no other session is actively driving a phase branch. The check is *"is another lane actively driven,"* not *"is my tree clean."*
3. **Never skip reading `.claude/agent-activity.json`** at session start when the system reminder hints at multi-session topology. If the file is absent, surface its absence as a signal (not a green light).

## See also

- `.claude/rules/advisor-orchestrator.md` §1 — surface-first session-start ritual (the operational fix; landed 2026-05-22).
- `.claude/hooks/session-start-multi-lane-check.sh` — SessionStart hook emitting the WARN signal that makes the anchor visible to the model.
- `project_concurrent_advisor_sessions_2026_05_21.md` — Occurrence-1 project memory; the incident detail + reflog/timeline.
- `e6035f5e1` — Occurrence-2 commit on `brehon-fork-fed-in-c`; concurrent session's independent same-defect-class fix (second-occurrence evidence).
- `feedback_falsifiable_hypothesis_before_structural_fix.md` — adjacent "inherited hypothesis" lesson; same family of "inherited content overrides current-turn verification" defect class.
- `feedback_advisor_instruction_mismatch_stop_and_ask.md` — the broader "stop when the instruction and the on-disk state disagree" discipline; this lesson is the session-start specialisation.
- `feedback_principles_not_rules.md` — the recurrence-threshold discipline that promoted this from session-note to lesson (2 independent sessions, 24h window).
