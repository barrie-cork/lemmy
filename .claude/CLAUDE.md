# DQ cheatsheet

Async question queue at `decision-queue.json`. Full rules: `rules/decision-queue.md`. **Read those first if you're about to write `answered_by`** — attribution is load-bearing.

## On every iteration start

Read `decision-queue.json` → check `pending` → if your blocking entry has `answer` filled, move it to `resolved` and apply.

## When to write a pending entry

Only when **all** are true: you cannot answer it from the codebase / plan / rules; the plan is genuinely ambiguous OR you found something unexpected; you have ≥2 concrete options. Routine status, info you can grep, plan-covered questions → not the queue.

## Pending entry shape

```json
{
  "id": <max(all ids) + 1>,
  "from": "<planner|impl|bm|advisor>",
  "timestamp": "<UTC ISO 8601>",
  "question": "<one sentence, <50 words, evidence stays out>",
  "options": ["(A) concrete option", "(B) concrete option"],
  "context": "<1-2 sentences: what you checked>",
  "answer": null,
  "answered_by": null
}
```

## Attribution — the only labels you may write

| Your role | Valid `from` | Valid `answered_by` (self-resolve only) |
|-----------|--------------|-----------------------------------------|
| planning  | `planner`    | `planner` (pre-seed with named source) |
| impl-task | `impl`       | `impl-self-resolved` |
| bm-task   | `bm`         | `bm-self-resolved` |
| advisor   | `advisor`    | `advisor` or `user` (when relaying) |

**Never write `answered_by: "advisor"` or `"user"` from a subagent**, ever. Phase-6 DQ #37 was the lesson; rule is now load-bearing. Pre-seeded advisor recommendations from the planner → `answered_by: "planner"`, name the advisor in `answer` text.

## Mid-task visibility (Junior subagents only)

After writing a pending entry, **commit + push immediately** to your current branch — not at finalize, or the advisor's polling loop won't see it:

```
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): <role> raised DQ #<id> — <3-5 word slug>"
git push origin HEAD
```

Foreground sessions: commit at the next natural break, no special push.

## After writing — keep moving

If task N+1 is independent of the answer, do it. Note in progress log: "Task N blocked on DQ #M, continuing with N+1." Only stop the loop if fully blocked.

**Never guess the answer** while waiting. If you're tempted, the question wasn't worth queuing — answer with evidence and self-resolve, or wait properly.

## Advisor commit subjects (detection guard)

Advisor-session DQ writes MUST match `^(chore|docs)\((advisor|decision-queue)\)`. Any other subject introducing `answered_by: "advisor"` is a process breach, fixed via a `docs(attribution):` follow-up.
