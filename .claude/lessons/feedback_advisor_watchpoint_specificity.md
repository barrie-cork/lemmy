---
name: Plan watchpoints must cite a specific table, file, or schema line — not a concept
description: Every watchpoint in a plan's §4 (Solution / risks-to-watch) must name a concrete artifact — a table, a file path, a schema.rs line, an enum, a function — that the impl-task can grep for. Concept-only watchpoints ("watch for trait drift", "be careful with the lock") are unactionable at impl time; the advisor's pre-plan-approval gate (advisor-orchestrator.md §3.5) files a DQ requesting revision before the plan is approved.
type: feedback
---

A **watchpoint** in a Brehon plan (§4 of the plan template — the "things that could go wrong, watch these" list the planner hands to the impl-task) is only useful if the impl-task can act on it. A watchpoint that names a **concrete artifact** — a table, a file path, a `schema.rs` line range, an enum variant, a function signature — is checkable: the impl-task greps for it, reads it, confirms the risk is or isn't present. A watchpoint phrased as a **concept** — "watch for trait drift", "be careful with the advisory lock", "mind the federation edge cases" — is not checkable: the impl-task has nowhere to look and no pass/fail test, so the watchpoint is decorative.

This is an **advisor-side pre-plan-approval gate** (`.claude/rules/advisor-orchestrator.md` §3.5). When the planning Junior task ships a plan, the advisor reads every §4 watchpoint before surfacing the plan for user approval (gate 1). Any watchpoint that does not cite a specific table / file / `schema.rs` line / enum / function → the advisor files a `kind: "blocker"` DQ (`from: "advisor"`) requesting the planner revise that watchpoint to name the concrete artifact, and the plan does not advance to approval until it's fixed.

## Why this matters

The watchpoint is the planner's mechanism for transferring *risk knowledge* the planner has (from reading design docs, prior retros, the canonical schema) to the impl-task, which is a Sonnet worker that reads only what the plan + MIRROR refs name. The impl-task cannot independently rediscover "the `submit_jury_vote` handler has a terminal-state idempotency trap" — that knowledge has to arrive *as a named site*. A concept-only watchpoint loses the transfer: the planner *knew* something specific, compressed it to a vague phrase, and the specificity — the actual table/line/function — is exactly what got dropped.

The failure mode is silent. A concept-only watchpoint *reads* fine in the plan ("§4: watch for type-state guard gaps") and *feels* like coverage. It only fails later, at impl time, when the worker has no concrete site to check and either (a) skips the risk entirely, or (b) guesses at where it applies and gets it wrong. By then the cheap fix — the planner naming `submit_jury_vote.rs:271` in the watchpoint — is gone, and the cost is a wrong impl + a CR finding + a fix cycle.

Concretely, the planner-side discipline that produces good watchpoints is already visible in strong planning retros: the v1-RT-r4 planning retro notes "watchpoints cite specific tables/files (GOTCHA-55a, admin_config tx ordering, ADR-015 pseudonym, community-scope NULL-vs-NULL)" — each watchpoint names an artifact the impl-task can grep. That is the bar. A watchpoint like "be careful with the transaction ordering" without naming `admin_config.rs` and the specific keys is below it.

## How to apply (advisor, at the pre-plan-approval gate)

When the planning task ships and you (advisor) review the plan before surfacing it for user approval (gate 1, after the §3.4 DoD smoke test):

1. Read every watchpoint in plan §4.
2. For each, ask: **does it name a concrete artifact an impl-task could grep for?** — a table name, a file path, a `schema.rs` line or line-range, an enum, a function, a const, an ADR section with a specific rule. If yes → pass. If it's a concept with no named site → fail.
3. For each failed watchpoint, file a `kind: "blocker"` DQ from `advisor` (`chore(decision-queue): advisor — watchpoint <N> lacks a specific cite`) asking the planner to revise it to name the table/file/line. The plan does not advance to approval until every watchpoint passes.
4. Acceptable cites (examples): "`crates/api/api/src/governance/submit_jury_vote.rs:271` terminal-state idempotency guard"; "`moderation_case.status` enum — the `Closed`/`EmergencyRemoved` variants"; "`schema.rs:1204` the `reputation_snapshot` JSONB column"; "ADR-015 §pseudonym — every governance-log write must use `actor_pseudonym`". Unacceptable: "watch the case status handling", "mind the JSONB", "respect GDPR".

The check is cheap (read §4, ~2 minutes for a typical 4–6-watchpoint plan) and happens once per plan. It is the same family as the §3.4 DoD smoke test — both are advisor pre-approval gates that catch a plan defect *before* it costs an impl cycle, by testing the plan's claims against a concrete bar rather than accepting them as written.

## Generalises to

Any orchestration model where a planning role hands risk-knowledge to a separate execution role that reads only what it's told. The transfer only works if the risk is named as a *checkable site*, not a concept — because the execution role cannot rediscover the planner's context. The advisor (who sees both the plan and the codebase) is the only actor positioned to enforce the bar, which is why it's an advisor pre-approval gate and not a planner self-check. Same family as `feedback_plan_drift_metadata_cross_check.md` (grep the named metadata key before trusting the plan row) and `feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md` (verify a transcribed rule's preconditions against current state) — in all three, the cheap concrete check at authoring/approval time replaces an expensive failure at execution time.

## See also

- `.claude/rules/advisor-orchestrator.md` §3.5 — the watchpoint-specificity gate this lesson is the rationale for (the rule that cites this file).
- `.claude/PRPs/templates/plan.template.md` §4 — where watchpoints live in the plan.
- `feedback_plan_drift_metadata_cross_check.md` — the sibling grep-before-trust discipline for plan-named metadata keys.
- `feedback_pre_phase_dod_smoke_test.md` — the other advisor pre-plan-approval gate (run every §15 DoD command literally).
- `feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md` — the brief-author-time precondition check (same cheap-now-vs-costly-later family).
