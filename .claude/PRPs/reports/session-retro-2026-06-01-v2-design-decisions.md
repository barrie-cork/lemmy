# Session retro — 2026-06-01 — v2-design-decisions

**Harness:** claude-code
**Session window:** ~2026-06-01T14:00 → ~2026-06-01T17:30 IST (~210 min)
**Branch at start:** `3add63c07` (`governance-v0`)
**Branch at end:** `governance-v0` (no new commits this session — doc edits only)
**Files touched:** 3 (99-decisions-and-open-questions.md, v2-messaging-rtc.prd.md, docs/research/matrix-homeserver-selection-2026.md)
**Commits:** 0 (session was design/research — no commits authored)

## TL;DR

A design-clarification session that closed three open questions on the V2 messaging track: OQ-V2-10 (Matrix homeserver → Tuwunel), OQ-009 (juror anonymity → graduated mutual visibility), and OQ-V2-09 lean (disable-after-enable → soft pause). The most load-bearing finding is that **both Tuwunel AS API blockers were already fixed** (issue #219 late 2025, issue #465 v1.7.1 May 2026) — the homeserver decision that was deferred "until V2a schedule time" could be made today with full confidence. The top change proposal: close open V2 OQs earlier rather than parking them; the research overhead per OQ was ~15 min and the decisions were clean once the data was in front of us.

---

## What surprised us

- **Tuwunel bugs already fixed.** Both critical AS API issues the research report flagged as "verify before committing" were already closed with fixes shipped — #219 in late 2025 (6 months ago), #465 in v1.7.1 (10 days ago). The OQ had been parked with "re-take at V2a schedule time" but there was no reason to wait; the data was available now. Lesson: "park until schedule time" can mean unnecessarily deferring decisions that are already resolvable.

- **Dendrite and original Conduit both eliminated.** When OQ-V2-10 was opened in April 2026 the candidate list was Synapse / Conduit / Dendrite. By June 2026 the landscape had simplified to Tuwunel or Synapse — two of the three original candidates either archived (Dendrite, Nov 2024) or abandoned (Conduit). The "re-take at V2a schedule time" instruction in the OQ was correct; the surprise is how much the ecosystem shifted in just 6 weeks.

- **OQ-009 resolution was richer than the original lean.** The original OQ-009 lean was "revealed to each other after accepting assignment" — a binary toggle. The session produced a more nuanced graduated-visibility model (handles visible only when entering a room where discussion is already in progress, threshold admin-configurable). This is better than the original lean and required only ~15 min of design reasoning. The lean should have been revisited earlier; it had been sitting unresolved since April 2026.

- **Jury is fully functional without V2 messaging.** The audit confirmed zero hard gaps — the only UX gap is OQ-005 (juror notification on assignment). This is a useful anchor: V2 messaging is a deliberation-UX enhancement, not a prerequisite for producing valid verdicts. Worth surfacing explicitly in V2a planning so scope creep doesn't treat messaging as a prerequisite.

- **Perplexity research quality on Matrix ecosystem was high.** 42 sources, correct version numbers, active GitHub issue links checked in real time. The research prompt yielded actionable output first pass with no need for a follow-up search. The investment (crafting the prompt, running it, copying the result) was ~20 min total for a decision that would otherwise require multi-hour independent research.

---

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Add a "resolvability check" step to the OQ parking discipline.** When an OQ is parked with "re-take at X time", the advisor should run a 5-min feasibility check at each relevant session: "is the blocking condition now met?" OQ-V2-10 could have been resolved in any session after Nov 2025 (when #219 was fixed) but sat parked until now. Add a note to `.claude/rules/advisor-orchestrator.md` §3.3 or the V2 section: before any V2a planning session, scan V2-track OQs for "parked" status and attempt resolution first. | Closes decisions earlier, reduces V2a planning-session ambiguity | minor — one-line addition to existing rule | 1× this session; OQ-V2-10 pattern |
| 2 | **Promote the "Perplexity deep research" pattern as a named tool for ecosystem-survey decisions.** The Matrix homeserver selection was a well-scoped ecosystem question (multiple options, evolving landscape, clear evaluation criteria). The prompt-crafting took ~10 min, the research output was actionable first pass. This pattern — write a structured Perplexity prompt, copy result to `docs/research/`, synthesise into OQ resolution — is reusable for any future ecosystem decision (e.g. OQ-027 Autonomi research, OQ-ADR016-01 B-fetch adapter SPI). Add a pointer in `.claude/brehon-reference.md` under "Research tools" or as a note in the PRD template. | Reduces future ecosystem-decision friction; makes the research-to-OQ-resolution pipeline explicit | minor — reference note only | 1× this session; applicable to OQ-027, ADR016 OQs |
| 3 | **Commit doc-only sessions.** This session made substantive edits to two tracked files (99-decisions-and-open-questions.md, v2-messaging-rtc.prd.md) and copied a new research file, but produced zero commits. The edits are uncommitted working-tree changes. Per `feedback_commit_aggressively_in_shared_repos.md`, doc edits on `governance-v0` should be committed immediately — especially OQ resolutions, which are append-only audit artefacts. At session close, always commit open doc edits before writing the retro. | Prevents uncommitted OQ resolutions from being lost or clobbered by a concurrent session | minor — discipline reminder | 1× this session; 2+ in prior memory (known pattern) |

---

## What to carry forward

- **OQ-first agenda for V2a planning.** Before the first V2a planning Junior task is queued, resolve the remaining two V2a-blocking OQs: OQ-V2-08 (vanilla-Lemmy interop — needs a user decision) and OQ-V2-09 full spec (soft-pause mechanics — V2a sub-PRD owns it). Neither requires ecosystem research; both need ~15 min of design reasoning.

- **Tuwunel deployment topology note is load-bearing.** The containerised-non-host-network caveat (bridge traffic is non-loopback, #465 fix doesn't help) must land in the V2a sub-PRD ops section explicitly. It's easy to forget between now and V2a schedule time. The 99-decisions OQ-V2-10 entry has it; make sure the V2a brief §4 Constraints cites it.

- **OQ-009 graduated-visibility model is V2b input, not V2a.** V2a (bridge + 1:1 DM + admin config) doesn't provision jury rooms — that's V2b. The OQ-009 resolution is ready when V2b planning starts; don't let it re-open during V2a scope discussions.

- **Jury notification (OQ-005) is the real V1 UX gap.** The jury completeness audit confirmed this: zero hard functional gaps without messaging, but jurors have no in-platform notification of assignment. This should be on the v1-quality or v1.5 agenda regardless of V2 schedule.

- **Soft-pause as the OQ-V2-09 lean is now on record.** V2a sub-PRD author needs to read this lean and spec: bridge process shutdown signal, in-flight room handling, active-jury-room-on-disable policy, media/GDPR artefact cleanup runbook. The structural decision (soft pause, not hard decommission) is made; the four concrete questions are open.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Explore subagent (V2 scope survey) | 30 | 0 | low | Clean first-pass synthesis; Perplexity research context already in the file |
| Explore subagent (jury completeness audit) | 25 | 0 | low | Confirmed zero hard gaps cleanly; e2e line references were precise |
| Explore subagent (juror anonymity model) | 20 | 0 | none | Retrieved exact OQ text + V2b PRD group-property argument; synthesis was accurate |
| Explore subagent (open tech stack survey) | 20 | 0 | low | Comprehensive; ADR-016 OQs correctly scoped to M1/M2 |
| Perplexity deep research (Matrix homeserver) | 60 | 0 | high | 42-source report, active issue links, correct version numbers — resolved a parked OQ immediately |
| `gh issue view` (Tuwunel #465, #219) | 5 | 0 | medium | Both already closed/fixed — wasn't expected; saved the "wait for V2a" deferral |
| OQ-009 design reasoning (session conversation) | 15 | 0 | low | Graduated-visibility model was richer than original lean; clean one-pass design |
| Doc edits (99-decisions.md, v2 PRD) | 10 | 0 | none | Mechanical; OQ-V2-10, OQ-009, OQ-V2-09 all updated correctly |

## Complexity scores (heavy tasks only)

No impl-tasks ran this session. Pure research + design + doc-edit session. N/A.

---

## Decisions to revisit

- **OQ-V2-08 (vanilla-Lemmy interop)** — the one remaining hard gate before V2a integration testing. Needs a user decision: Brehon↔Brehon only (option a) or degraded-mode interop for vanilla peers (option b). ~10 min to resolve; no research needed.
- **OQ-005 (juror notification UX)** — not a V2 gate, but the real UX gap in v1 today. Worth scheduling for v1-quality or v1.5.
- **Commit open doc edits** — 99-decisions-and-open-questions.md and v2-messaging-rtc.prd.md have uncommitted changes from this session. Commit before next session.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Commit doc-only sessions before retro**: promote to a note in `feedback_commit_aggressively_in_shared_repos.md` — add explicit mention that OQ resolutions and design-doc edits on governance-v0 must be committed at session close, not left as working-tree changes. (Recurrence: 2+ in prior memory per MEMORY.md entry.)

- [ ] **Perplexity ecosystem-research pattern**: add a pointer in `.claude/brehon-reference.md` under a "Research tools" section — "for ecosystem-survey OQs, craft a structured Perplexity prompt → copy to docs/research/ → synthesise into OQ resolution; ~15-20 min per OQ, first-pass actionable." (1× this session; applicable to OQ-027 and ADR016 OQs — pre-recurrence promotion candidate.)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
