# DQ cheatsheet

Async question queue at `decision-queue.json`. Full rules: `rules/decision-queue.md`. **Read those first if you're about to write `answered_by`** — attribution is load-bearing.

## On every iteration start

Read `decision-queue.json` → check `pending` → if your blocking entry has `answer` filled, move it to `resolved` and apply.

## When to write a pending entry

Write only when **all** hold:
- cannot answer from codebase / plan / rules
- plan is genuinely ambiguous OR something unexpected found
- have ≥2 concrete options

Do NOT queue: routine status, greppable info, plan-covered questions.

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

Continue with independent tasks. Log: "Task N blocked on DQ #M, continuing with N+1." Stop only if fully blocked. Never guess — self-resolve with evidence or wait.

## Advisor commit subjects (detection guard)

```yaml
dq_write_commit_subject_pattern: "^(chore|docs)\\((advisor|decision-queue)\\)"
violation_remedy: "docs(attribution): follow-up commit"
# Any commit introducing answered_by:"advisor" must match the pattern above
```
