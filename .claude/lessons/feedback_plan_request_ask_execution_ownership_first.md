---
name: Ask execution-ownership + execution-tooling in the FIRST plan-request question batch
description: When a request is "create a plan / design X," the opening AskUserQuestion batch should ask WHO executes (you-drive vs I-drive vs four-role pipeline) and WITH WHAT (normal pipeline vs /workflows vs hybrid) alongside scope/isolation/deliverable. These two dimensions surface late as separate mid-work steering messages otherwise, costing extra clarify round-trips. 2× recurrence: 2026-06-04 v1-closeout (execution-ownership + /workflows both arrived mid-work) + 2026-05-23 ADR-016 scope-reframe (premise changed mid-session after the drift check).
type: feedback
---

## TL;DR

When the user asks you to **author a plan or design** for non-trivial work, the very first `AskUserQuestion` batch (the one you run before / at the start of plan mode) should cover **four** dimensions, not two:

1. **Scope** — what surfaces / how aggressive.
2. **Isolation / constraints** — branch, worktree, what-not-to-touch.
3. **Execution ownership** — *who drives the resulting plan?* (the user in a separate session / you the advisor now / the four-role Junior pipeline). ← easy to forget
4. **Execution tooling** — *with what machinery?* (the normal pipeline / a CC `/workflows` fan-out / a hybrid of both). ← easy to forget

Dimensions 3 and 4 change the *shape* of the plan (handoff doc vs inline execution; workflow-lane tags vs four-role gates), so asking them up front means the plan is authored correctly the first time. Skip them and they re-surface as separate mid-work steering messages, each forcing a clarify round-trip and a plan edit.

## Why this matters (2× recurrence)

**2026-06-04 — v1-closeout plan session.** The opening AskUserQuestion batch asked scope + isolation + deliverable (good), but NOT execution-ownership or tooling. Over the next ~80 minutes the user sent these as separate mid-work messages: *"I will continue the plan in a different worktree directory"* (→ ownership = user-drives) and *"optimise it for /workflows"* + *"perhaps hybrid"* (→ tooling = hybrid workflow/four-role). Each arrived after drafting had started, forcing a re-frame: the plan had to grow a "handoff model" section, an execution-lane column on every phase, and a separate bootstrap-handover deliverable. None of this was wasted — but two extra AskUserQuestion round-trips and several plan edits would have been one batch had dimensions 3+4 been in the opener. (Retro: `session-retro-2026-06-04-v1-closeout-plan-bootstrap.md`, PMD eval #777.)

**2026-05-23 — ADR-016 cross-app-backplane session.** Same class, different dimension: the session ran a PRD drift check; it returned "no drift," but the user then reframed the project premise mid-session ("Lemmy-fork with governance" → "governance backplane spanning Matrix/PeerTube/Lemmy-fork"). The "is X still accurate?" ritual jumped straight to fixes without first surfacing "is the *scope* still right?" The retro's carry-forward was: a drift/plan ritual should surface the meta-question (scope/ownership/tooling still right?) BEFORE diving into the work. (Retro: `session-retro-2026-05-23-adr-016-cross-app-backplane.md`, PMD eval #508.)

The common root: **the dimensions that decide a plan's *shape and destination* are the ones most easily left implicit**, because the literal request ("create a plan") foregrounds *content* (scope) and backgrounds *delivery* (who runs it, with what, in what session). Content questions get asked; delivery questions get discovered.

This is the same family as the **Phase-2 e2e "local vs dispatch" gate** in `advisor-orchestrator.md` §3.2 — an execution-ownership decision (advisor-laptop runs it vs dispatch to GH Actions) that the orchestrator learned to surface explicitly rather than auto-pick, after it kept surfacing late (post-PR #105).

## How to apply

When a user message matches "create/write/design a plan for X," "plan out X," or you're about to `EnterPlanMode` for a non-trivial deliverable, make the **opening** AskUserQuestion batch a 3–4 question set that always includes:

- **Execution ownership** — e.g. *"What deliverable do you want now — a plan only (you/I execute later), plan + I start the low-risk tier, or just an assessment?"* AND, when a handoff is plausible, *"Who drives execution — you in a separate session, or me now?"*
- **Execution tooling** — when the work has parallel/fan-out structure or the user has ever mentioned workflows, *"Normal four-role pipeline, a `/workflows` fan-out, or hybrid?"*

Then weave the answers into the plan's header (a one-line **"who executes / with what"** field) so the structure (handoff doc? lane tags? inline?) derives mechanically.

**Cheap test:** before writing the plan body, ask yourself — *"if I finish this plan, do I know who opens it next and what machinery runs it?"* If either answer is "I'd have to ask," ask now, in the current batch, not after the user tells you mid-draft.

**Don't over-rotate:** for a trivial plan (single-file change, obvious owner) skip the ceremony — this is for non-trivial, multi-phase, or handoff-likely work where the delivery shape is load-bearing. Per `feedback_principles_not_rules.md`, this is a principle (surface delivery dimensions early), not a mandatory 4-question template on every plan.

## Canonical example consulted

Format mirrors `feedback_falsifiable_hypothesis_before_structural_fix.md` (TL;DR → Why-with-incidents → How-to-apply). The "surface the meta-question before diving in" framing is the same shape that lesson applies to structural-fix DQs; this lesson applies it to plan-request question batches.

## See also

- `.claude/rules/advisor-orchestrator.md` §3.2 gate 4 (Phase-2 local-vs-dispatch — the canonical execution-ownership-surfaced-late precedent).
- `feedback_principles_not_rules.md` — why this is a principle, not a rigid template.
- `session-retro-2026-06-04-v1-closeout-plan-bootstrap.md` + `session-retro-2026-05-23-adr-016-cross-app-backplane.md` — the two recurrence incidents.
