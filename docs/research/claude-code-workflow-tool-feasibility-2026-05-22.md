# Claude Code Workflow Tool — Feasibility Assessment for Brehon `/auto-roadmap`

**Date:** 2026-05-22  
**Source repo:** https://github.com/ray-amjad/claude-code-workflow-creator  
**Context:** Assessed whether the Claude Code Workflow tool (unreleased, env-gated) could be used to convert `/auto-roadmap` and `/roadmap-next` to a deterministic multi-agent orchestration format.

---

## What the Workflow tool is

A JavaScript orchestrator built into the Claude Code binary, gated behind `CLAUDE_CODE_WORKFLOWS=1`. Key properties:

- **Deterministic JS control flow** — `pipeline()`, `parallel()`, `phase()` primitives; orchestrator layer spends zero model tokens
- **Fresh context per `agent()` call** — each leaf call gets an isolated context window; agents communicate only via prompt strings
- **Resume from run ID** — `resumeFromRunId` replays cached `agent()` results up to the edited point, within the same session
- **Hard sandbox** — no `Date.now()`, `Math.random()`, no fs/Node APIs in the orchestrator (those go inside `agent()` calls)
- **1000-agent lifetime cap** per workflow run
- **Files live at** `.claude/workflows/<name>.js` (project) or `~/.claude/workflows/<name>.js` (global)

### Core API

| Global | Purpose |
|---|---|
| `agent(prompt, opts?)` | Spawn one fresh-context subagent; returns string or validated object (if `schema` set) |
| `pipeline(items, ...stages)` | Stream items through stages, no barrier between stages — default for multi-stage work |
| `parallel(thunks)` | Run `() => Promise` thunks concurrently; a barrier — use only when a stage genuinely needs ALL prior results |
| `phase(title)` | Progress group in `/workflows` UI |
| `budget` | `{ total, spent(), remaining() }` — token-aware loop guard |
| `args` | Input passed through unchanged (`unknown` type — normalize before use) |
| `workflow(name, args?)` | Inline nested workflow (one level only) |

### Determinism rules (hard-enforced, throw if violated)
- `Date.now()`, `Math.random()`, argless `new Date()` — banned (break resume)
- No `require`, `fs`, `process` in the orchestrator
- `parallel()` takes thunks (`() => agent(...)`), NOT bare promises
- `meta` must be the first statement, a pure literal (no variables, spreads, function calls)

---

## Mapping to Brehon `/auto-roadmap` + `/auto-phase`

### What maps well

| Brehon pattern | Workflow equivalent | Fit |
|---|---|---|
| `[P]` cohort parallel dispatch | `parallel(cohortTasks.map(t => () => agent(...)))` | ✅ Clean |
| Sequential stage ordering (planning → bm-cut → impl → merge) | `pipeline()` or sequential `await` | ✅ Clean |
| Per-task fresh context (impl-task/ci-watcher isolation) | `agent()` fresh-window per call | ✅ Aligns by design |
| validate-pending + ci-watcher loop | `pipeline(implTasks, dispatchImpl, pollCiWatcher)` | ✅ Expressible |
| Phase progress visibility | `phase()` + `log()` + `/workflows` UI | ✅ Better than current polling |
| Plan-gap handler (PRD read + brief auto-author) | Bounded `parallel` fan-out | ✅ Clean sub-step fit |

### Hard blockers

| Requirement | Workflow tool | Status |
|---|---|---|
| **Cross-session resume** | `resumeFromRunId` works within same session only; session restart kills resume chain | ❌ Hard blocker |
| **Six mandatory user gates** (`AskUserQuestion`) | No pause-for-human primitive in JS orchestrator | ❌ Hard blocker |
| **`ScheduleWakeup` polling cadence** | No scheduling primitive (270s/1200s cache-TTL-aware cadence) | ❌ Missing |
| DQ writes + git commits between stages | Possible inside `agent()` calls but orchestrator can't sequence commit→user-gate→next-agent across hours | ⚠️ Structural mismatch |
| Junior daemon dispatch (EliteDesk MCP) | `mcp__junior-brehon__create_task` callable inside `agent()` in principle; but no cross-session polling | ⚠️ Architectural mismatch |

### Why `/auto-roadmap` is a worse fit than `/auto-phase`

`/auto-roadmap` wraps `/auto-phase` — its substance IS the `/auto-phase` invocation. Converting the wrapper without converting the inner skill is meaningless. Both hit the same three blockers.

---

## What IS worth converting (once tool ships)

### Option A — Cohort dispatch sub-workflow

Extract the `[P]`-cohort parallel fan-out into a standalone workflow:

```js
export const meta = {
  name: 'brehon-cohort-dispatch',
  description: 'Dispatch a parallel impl-task cohort to Junior daemon',
  phases: [{ title: 'Dispatch' }, { title: 'Poll' }],
}

const { tasks, phaseBranch } = typeof args === 'string' ? JSON.parse(args) : args

phase('Dispatch')
const results = await parallel(
  tasks.map(t => () => agent(
    `Dispatch Junior impl-task for ${t.slug}. Brief at ${t.briefPath}. Base branch: ${phaseBranch}.`,
    { label: `dispatch:${t.slug}`, schema: DISPATCH_RESULT_SCHEMA }
  ))
)

phase('Poll')
// each agent polls its own Junior task to terminal
const validated = await parallel(
  results.filter(Boolean).map(r => () => agent(
    `Poll Junior task ${r.taskId} to terminal. Mutate DQ entry ${r.dqId} on result.`,
    { label: `poll:${r.taskId}` }
  ))
)

return { dispatched: tasks.length, validated: validated.filter(Boolean).length }
```

This avoids all three blockers — cohort dispatch is bounded (~2–10 min per task), same-session, no user gates needed mid-fan-out.

### Option B — BM verb pipeline

`bm-poll-cr → bm-triage → fix-in-pr commits → bm-merge` is a bounded sequential pipeline (~10–30 min total) with no cross-session requirements. Clean `pipeline()` fit after gate 3 approval.

---

## Roadmap recommendation

Once `CLAUDE_CODE_WORKFLOWS=1` is officially released:

1. **Near-term:** `v1-workflow-cohort` sub-phase — convert cohort dispatch to a workflow. Advisor still owns user gates + DQ; workflow handles parallel fan-out + fresh-context isolation for impl-tasks.
2. **Medium-term:** `v1-workflow-bm-pipeline` — BM verb tail as a workflow (post gate-3).
3. **Long-term:** full `/auto-phase` conversion — only viable if Anthropic adds cross-session resume + a pause-for-human primitive to the Workflow tool.

---

## Source material

- Repo README: `ray-amjad/claude-code-workflow-creator`
- `SKILL.md` — authoring procedure, topology selection guide, gotchas
- `references/api-reference.md` — complete manual (all globals, caps, determinism sandbox)
- `references/patterns.md` — fan-out, pipeline, barrier, budget-loop, adversarial-verify patterns
- Binary inspection notes (from repo README): details verified against CC binary, not guessed
