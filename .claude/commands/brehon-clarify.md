---
description: Run a structured coverage-questions pass on a draft brief BEFORE the planning task is queued. Pre-fills DQ entries (advisor-mode) or surfaces them to user (user-relay mode). Spec-kit-pattern adoption — gates the planning stage in advisor-orchestrator.md.
argument-hint: <path/to/.claude/PRPs/briefs/<phase>-<role>-N.md> [--mode advisor|user-relay]
---

<objective>
The `/brehon-clarify` command runs structured coverage questions over a draft brief and produces DQ entries that gate the planning subagent. The intent is to **collapse round-trips** that today happen post-planning (plan revision after-the-fact when ambiguity in the brief surfaces during impl).

This is a spec-kit-derived pattern (see plan adoption rationale in `.claude/lessons/feedback_clarify_before_plan.md`). It runs **only in the advisor session** — never as a Junior task, never autonomously.

**Hard precondition (per advisor-orchestrator.md "Stage-shape orchestration"):** the planning task for `<phase>` is NOT queued until every clarify-DQ entry on the brief is resolved.
</objective>

<usage>
**`$ARGUMENTS`** = `<path/to/brief.md>` followed by optional `--mode advisor` (default) or `--mode user-relay`.

Examples:

- `/brehon-clarify .claude/PRPs/briefs/v1-JM-e-planning-1.md` — advisor-mode
- `/brehon-clarify .claude/PRPs/briefs/v1-JM-e-planning-1.md --mode user-relay` — escalate all questions to user via AskUserQuestion
- `/brehon-clarify .claude/PRPs/briefs/v1-JM-e-planning-1.md --mode advisor` — advisor self-answers from lessons + ADRs + prior plans

If `$ARGUMENTS` is empty, refuse: "specify a brief path".
</usage>

<workflow>

### Step 1: Read the brief

Read the entire file at the brief path. Confirm it conforms to `.claude/PRPs/templates/impl-task-brief.template.md` shape. If sections are missing (no §2 Scope, no §3 Required reading, no §4 Constraints), the brief is too underspecified for clarify — refuse and tell the advisor to extend the brief first.

### Step 2: Read referenced documents

For each path cited in §3 Required reading of the brief, do a `Read` (or, if oversized, a Grep for the cited symbol). Goal: load enough context to **detect ambiguity**, not to plan the work.

Also read:
- `.claude/decision-queue.json` — does any pending or recent resolved entry already cover this brief?
- `.claude/PRPs/prds/<phase-family>.prd.md` if cited
- The 2 most recent `.claude/PRPs/plans/<sibling-phase>.plan.md` files for reference patterns

### Step 3: Generate coverage questions

For each section in the brief, generate questions across these axes:

| Axis | Trigger | Question shape |
|---|---|---|
| **Scope ambiguity** | "do X" without saying which file/crate | "Should the change land in `crates/<a>` or `crates/<b>`? Cite §11 of plan if applicable." |
| **Undefined input** | symbol referenced but not defined | "What is the canonical shape of `<symbol>`? Cite the doc-comment or the schema.rs line." |
| **Undefined output** | "produce X" without naming the file | "Which exact file path is the deliverable? `<file-a>` or `<file-b>`?" |
| **Undeclared constraint** | a §10 mirror that disagrees with a §4 constraint | "When MIRROR §10.<X> conflicts with constraint §4.<Y>, which wins?" |
| **Missing required-reading** | brief mentions a pattern but doesn't cite the file | "What file:line implements the `<pattern>` cited in §2? Add to §3." |
| **Cross-phase invariant** | brief touches a file that a different sub-phase ships | "Does this brief overlap with `<sibling-phase>` Task <N>'s file ownership? File DQ if yes." |
| **DoD executability** | brief inherits §15 commands that won't run | "Will `cargo check --workspace --features full` work given this brief's scope, or does the planner need `--workspace` only? Per `feedback_features_full_p_crate_incompatible.md`." |
| **Watchpoint specificity** | brief or §4 cites a "watch X" without file:line | "Cite the specific table, file:line, or schema.rs line for the watchpoint. Per `feedback_advisor_watchpoint_specificity.md`." |

**Refuse to generate questions where:**

- The answer is in the brief already (re-read carefully).
- The answer is in `.claude/lessons/feedback_*.md` and the brief just hasn't cited the lesson — instead, **add a citation** to the brief's §3 Required reading.
- The answer is in a resolved DQ entry — instead, surface the DQ id in the brief's §3.

### Step 4: Apply mode

#### `--mode advisor` (default — cheap path)

For each question, attempt to answer from:
1. The brief itself (if you missed it on first read).
2. `.claude/lessons/feedback_*.md` corpus.
3. `.claude/decision-queue.json` resolved entries.
4. Prior `.claude/PRPs/plans/*.plan.md` files (the most recent sibling phase).
5. ADRs cited in the brief.

If you can answer with evidence: write the entry directly to `decision-queue.json` `resolved` array with `from: "advisor"`, `kind: "clarify"`, `answer: <the answer>`, `answered_by: "advisor"`, `resolved_at: <NOW_ISO>`. The `answer` text must cite the source file:line.

If you cannot answer with evidence (the question is genuinely judgment-heavy), fall through to user-relay for that question only.

#### `--mode user-relay`

For each question, surface to user via AskUserQuestion. Use the brief's role for the `header` field (e.g. "Brief scope", "DoD executability"). Pre-fill `options` with the concrete alternatives from Step 3.

For each user answer: write to `decision-queue.json` `resolved` with `from: "advisor"`, `kind: "clarify"`, `answer: <user's answer verbatim>`, `answered_by: "user"`, `resolved_at: <NOW_ISO>`. The `answer` text must include "(per user 2026-MM-DD)".

### Step 5: Update the brief if needed

If a clarify-DQ resolution **changes** the brief — adds a required reading, narrows a scope, names a file — Edit the brief in-place and reference the DQ id. Commit the brief update + DQ entries together with subject:

```
chore(advisor): clarify <phase>-<role>-N — see DQ #<lo>-#<hi>
```

If clarifications produced no brief changes (all answers were "already evident, citation added"), commit only the DQ entries:

```
chore(decision-queue): advisor clarify pass on <phase>-<role>-N
```

### Step 6: Final report (return to user)

```
## Clarify pass complete — <phase>-<role>-N

**Brief:** `.claude/PRPs/briefs/<phase>-<role>-N.md`
**Mode:** advisor | user-relay
**Questions generated:** <total>
**Resolved by advisor:** <count> (citations: DQ #<lo>-#<hi>)
**Resolved by user:** <count> (citations: DQ #<lo>-#<hi>)
**Brief edits:** <count> (new §3 citations + scope narrowings)
**Commit:** <sha>

**Planning gate:** CLEAR — all clarify-DQ entries resolved. Advisor may queue planning task per advisor-orchestrator.md "Stage-shape orchestration".
```

</workflow>

<hard-refusals>

1. **Never mark planning task as queueable** unless every clarify-DQ entry for this brief is resolved. The advisor-orchestrator rule (`.claude/rules/advisor-orchestrator.md` "Stage-shape orchestration") enforces this — do not bypass.

2. **Never write `from: "planner"` or `from: "impl"`** for clarify entries. Clarify is an advisor-side action — `from: "advisor"` always.

3. **Never write `kind: "log"`** for clarify entries. Clarify questions either gate planning (`kind: "clarify"` ≈ blocker but pre-planning) or don't exist. A clarify entry that doesn't gate is a question you should have answered yourself.

4. **Never run during a Junior task on the same brief.** If a Junior task is currently `running` against `<phase>` (per `mcp__junior-brehon__list_tasks`), the brief is committed and live — clarify pass would race. Refuse and tell the advisor to wait or cancel.

5. **Never commit to `governance-v0` if the brief is on a phase branch.** Match the brief's branch — clarify-DQ entries push wherever the brief lives.

6. **Never auto-answer a question whose answer would change ADR semantics.** ADRs are append-only (per `prp-plan.md`). If a clarify question would re-litigate an ADR, surface to user even in advisor-mode.

</hard-refusals>

<rationale>

### Why this command exists

Pre-planning ambiguity in briefs surfaces as plan revision rounds during impl. Recent observed loops (per `feedback_advisor_watchpoint_specificity.md`, `feedback_plan_dod_dry_run_at_write.md`) trace to brief sections that didn't say which file, didn't cite which schema.rs line, didn't acknowledge a sibling-phase file ownership conflict. The planner produces a plausible plan based on the brief; impl hits the ambiguity; round-trip ensues.

`/brehon-clarify` runs the disambiguation pass **before** the plan is written — at the cheapest point in the pipeline. Spec-kit calls this `/specify.clarify`. The Brehon adaptation:

- Reuses the existing decision-queue v2 schema (just adds `kind: "clarify"`).
- Reuses the existing attribution-integrity rules (advisor-only labels).
- Plugs into the existing advisor-orchestrator stage-shape (between brief-author and planning-task-queue).
- Stays aware of mid-task DQ visibility (mode=advisor commits + pushes from the laptop directly; no Junior worktree involved).

### Why not generate questions exhaustively

Coverage-question generation is bounded by the axes table in Step 3. Generating questions for every conceivable ambiguity would flood the queue and dilute attention to the load-bearing ones. The axes table is the gate.

### Why advisor-mode default

The lessons corpus + resolved DQ + prior plans answer most clarify questions trivially — the advisor session has all of them in working context. User-relay is for the residual judgment-heavy questions (ADR-affecting, scope-changing, visible-to-others impact).

### When to skip /brehon-clarify entirely

- Re-running impl-task on a brief whose plan already shipped (no plan = no ambiguity to disambiguate).
- BM-task briefs (BM verbs are mechanical; clarify isn't useful — see `.claude/commands/bm/<verb>.md`).
- Retro briefs (retros are reflective, not forward-looking).

For planning briefs (the load-bearing case): always run `/brehon-clarify` first. Skipping is a process breach the advisor must justify in the planning task's commit body.

</rationale>
