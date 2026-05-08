---
name: Scope-check via AskUserQuestion before drafting on "also" / "optimise" mid-flight requests
description: When the user adds "also optimise X" / "also handle Y" mid-flight on a skill-extension or design session, run AskUserQuestion FIRST with 2-4 scope sketches before drafting. Drafting first, then trimming after a course-correction, wastes effort and introduces speculative scope that bleeds into the artifact even when trimmed.
type: feedback
---

When the user issues a mid-flight scope addition during a skill-extension, plan-authoring, or design session — phrasings like *"also optimise X"*, *"also handle Y"*, *"also we should..."*, *"while you're at it, ..."* — pause and run `AskUserQuestion` BEFORE drafting. Sketch 2-4 concrete scope options with file-list previews (per the standard preview-using-AskUserQuestion pattern) so the user can steer the scope before sunk-draft cost accumulates.

**Why:** the friction this avoids is a recurring "draft-then-trim" anti-pattern. Two empirical instances on 2026-05-08:

1. *Session-A (auto-phase skill-ship), late commit*: user said "we should also handle subagent offload"; ~8 min were spent drafting a full Phase 0.6 with CAN/MUST-NOT tables, an Agent invocation example, and cost-discipline guidance, before the user steered: *"Might not be a need. perhaps a sensible aproahc would eb to let retros pick up signals"*. The trim was clean (~30 sec to delete the over-built section), but the 8-min draft was sunk cost AND the session retro flagged it as a finding.

2. *Session-B (resume + retro extensions, hour later)*: same user-axis ("also optimise to use subagents to off load unnecessary context") — and I drafted a full subagent-offload table again before the user re-issued the same defer-to-retro-signal steer. Two recurrences within an hour from the same user-axis indicates the friction is structural, not session-specific.

The pattern: when a user says "also X" mid-flight, they often haven't yet decided whether X is in-scope, deferred, or rejected. They're flagging a concern, not committing to a build. Drafting full machinery treats the concern as a commitment — burns time AND nudges the artifact toward the speculative direction, which makes the eventual trim partial (some scope leaks even when the explicit section is deleted).

**How to apply:** when you receive a mid-flight "also X" request, BEFORE drafting:

1. Identify 2-4 distinct scope levels for X. Cover at minimum:
   - **Full now** — build the machinery as if X is committed
   - **Partial now** — build a hook-point or stub; defer the body
   - **Deferred to next signal** — note X as a watchpoint, don't build
   - **No-op / out-of-scope** — explicitly reject X with rationale

2. Run `AskUserQuestion` with each option carrying a `preview` showing the concrete files / sections that would be added at that scope level. The previews make the trade-off tangible — "full now" preview shows 100+ line addition; "deferred" preview shows a 4-line note; the cost difference is visible before the user decides.

3. Wait for the answer. If the user picks "full now", THEN draft. If "partial" or "deferred", apply the smaller-scope option directly. If "no-op", note the discussion in the retro under "decisions to revisit" and move on.

The 5 minutes for AskUserQuestion + previews is much less than the 8+ min of sunk-draft + trim observed in both empirical cases. Even if the user picks "full now", the previews ensured the draft scope matches their actual ask (no second steer, no second trim).

**When to skip:** when the user explicitly committed to scope upfront ("build the full subagent offload table now"), running AskUserQuestion is pure friction. The skip condition is: the user named the scope level in their own request. If they only flagged the concern ("also X"), apply the rule.

Also skip when X is genuinely small (≤5 line edit, single-file change). The AskUserQuestion overhead exceeds the drafting cost; just apply.

**Generalises to:** any session where the user's ask is open-ended and the assistant's natural inclination is to "do thoroughly". Plan-mode plans that drift into adjacent scope, code-review responses that propose refactors beyond the diff, retro-extension tables that grow categories beyond the actual signal. The discipline is the same: when the request is broad, narrow via choice before committing the artifact.

**Symptom to recognise:** in retro three-signal scoring, a session has Wasted >5 min on a single skill / agent invocation AND the wasted minutes correspond to a draft-then-trim pattern (file shows insertions later trimmed in the same session, OR the assistant's mid-session text shows "I drafted X but the user steered me to defer"). When you see that pattern in your own session retros twice in two consecutive sessions, this lesson is the rule to apply on the third.

**Related rule.** This lesson reinforces (and is reinforced by) `feedback_principles_not_rules.md` (judgment over rigid rules) — AskUserQuestion is the principle-aligned response when the principle "user's framing > assistant's inferred scope" is in play. It also pairs with `feedback_dogfood_slash_command_specs.md` — if the scope-question's preview can use a real file or section as the dogfood target, it's even more concrete.
