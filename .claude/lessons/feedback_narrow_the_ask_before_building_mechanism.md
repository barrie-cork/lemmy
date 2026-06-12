---
name: Narrow the ask before building a requested mechanism
description: When a user requests a specific MECHANISM (add X agent / X hook / X script / X service), the requested mechanism is a hypothesis about the fix, not the fix itself. Spend 1-4 AskUserQuestion gates surfacing the friction UNDERNEATH it, and read the actual architecture, BEFORE building. The 2026-06-12 "Haiku monitor agent" request dissolved into a ~67-line policy edit once the real need (unattended progression) surfaced and the architecture (monitor role already filled twice) was read. Same family as feedback_falsifiable_hypothesis_before_structural_fix.md.
type: feedback
---

## TL;DR

When a user asks for a specific **mechanism** — "add a Haiku agent that monitors X", "add a hook that fires on Y", "write a script that does Z", "spin up a service for W" — treat the named mechanism as the user's **hypothesis about the fix**, not the fix itself. Before building it: (1) run 1-4 `AskUserQuestion` gates to surface the *friction underneath* the requested mechanism, and (2) read the actual architecture to check whether the mechanism's job is already done by something that exists. The right deliverable is often far smaller than, or structurally different from, the thing requested.

## Why this matters (2026-06-12 "Haiku monitor agent" incident)

The user asked: *"add Haiku model agents to /auto-phase so that when junior tasks are scheduled, a cheap model is spun up that just monitors with sensible check-ins until the task has completed, at which stage it lets advisory know so it can progress the phase."*

Taken literally, that's a "build a new persistent cheap-model agent" project. Two moves inverted it:

1. **Read the architecture first.** `~/.claude/commands/auto-phase.md` + the daemon-completion-hook lesson + the auto-state template showed the monitor role was **already filled twice**: passively by the daemon's task-completion Telegram hook (out-of-band ✅/❌), and actively by the Opus advisor's own `ScheduleWakeup` loop (near-free while sleeping; one cheap status-only `list_tasks` on wake). A Haiku agent in the middle would **add a hop** (the advisor must still wake to author the next brief / dispatch / run the gate — work Haiku is not authorised to do), **cost continuous tokens**, and **add a mis-classification surface** the advisor would have to re-verify anyway (the `bm false-success` pattern, 5× confirmed).

2. **Narrow the ask with AskUserQuestion.** Four gates, each load-bearing:
   - *What's the real friction?* → not "monitor", but **"true unattended progression"**.
   - *What should happen at a gate when away?* → auto-clear the safe ones, park the rest.
   - *Which gates are safe?* → only 2 of 6 (e2e→local, retro sign-off); 4 stay human.
   - *How to implement the notifier half?* → **no agent** — reuse the daemon hook + one advisor-fired ping.

**Result:** a "spin up a cheap-model agent" project collapsed into a `--unattended` policy allowlist — ~67 in-repo lines + a skill-body edit, no new agent, strictly safer (no cheap model anywhere near an ADR, merge, or billing decision). The mechanism the user named was never built; the problem they had was solved better without it.

## The pattern

This is the **same family** as `feedback_falsifiable_hypothesis_before_structural_fix.md`: there, a DQ's named-code-path RCA is a hypothesis, not a contract — falsify it before fixing. Here, a user's named mechanism is a hypothesis about the fix — narrow it before building. Both resist the pull to execute a confidently-stated premise verbatim.

| | Falsifiable-hypothesis (sibling) | Narrow-the-ask (this) |
|---|---|---|
| The premise | A DQ/handover names a defect *site* | A user names a fix *mechanism* |
| The risk | Build a fix for an innocent code path | Build infra the user didn't actually need |
| The cheap inversion | ≤30 min grep the named path | 1-4 AskUserQuestion + read the architecture |
| The tell | "fix the daemon's reset step" | "add a cheap agent / hook / script that…" |

## When to apply

Fires when a user request **names a mechanism** rather than **states an outcome**:

- "Add a [agent / subagent / monitor / watcher / daemon] that…" → does an existing component already do this job?
- "Add a [hook / handler / trigger] for…" → is the event already handled, or is the real need a policy not a hook?
- "Write a [script / tool] to…" → is there a one-liner or an existing script, or is the friction upstream of where the script would run?
- "Spin up a [service / process]…" → does the cost/failure-surface of a new long-lived process pay for itself vs a config/policy change?

Does NOT fire when the user states an **outcome** and leaves the mechanism open ("I want the phase to progress while I'm away" — that's already the friction, narrow is done) or for trivial explicit edits ("rename this function", "bump this dep").

## How to apply

1. **Before writing any code**, read the architecture the mechanism would plug into. Ask: *is this job already done by something that exists?* If yes, the mechanism is redundant — surface that.
2. **Open with AskUserQuestion** (1-4 gates) to separate the *requested mechanism* from the *underlying friction*. The first question is usually "what's the actual problem you want this to fix?" with 3-4 concrete friction hypotheses as options.
3. **Recommend the minimal deliverable** that solves the surfaced friction — often a policy/config/lesson edit, not the requested agent/hook/script. State plainly when you're declining to build the literal request and why.
4. **If the mechanism IS the right fix** after narrowing, build it — narrowing confirms as often as it inverts. The discipline is the *check*, not a bias against building.

## Cost / benefit

- **Cost:** 1-4 AskUserQuestion round-trips + ~4 architecture file reads (~10-15 min).
- **Benefit (2026-06-12):** turned a multi-file new-agent build (continuous token cost + a new failure surface + a cheap model near governance decisions) into a ~67-line policy edit. The AskUserQuestion ×4 saved 90+ minutes of building-the-wrong-thing and prevented a structurally unsafe design from shipping.

## See also

- `feedback_falsifiable_hypothesis_before_structural_fix.md` — sibling: a named defect *site* is a hypothesis, not a contract. This lesson is the user-request-side analogue (a named *mechanism* is a hypothesis).
- `feedback_speculative_scope_check_before_drafting.md` — adjacent: AskUserQuestion-before-drafting on mid-flight "also X" scope additions. That lesson is about *scope width* (is X in scope?); this one is about *mechanism choice* (is the named mechanism the right fix at all?). Both use the AskUserQuestion-before-building discipline; apply together when a request is both broad AND names a mechanism.
- `feedback_auto_phase_unattended_gate_allowlist.md` — the feature this pattern produced; documents the rejected monitor-agent framing in its "Why this shape" section.
- `pattern_bm_false_success_advisor_post_condition_catch` (PMD) — why a cheap-model monitor's summary would have to be re-verified anyway, one of the reasons the monitor agent was the wrong fix.
