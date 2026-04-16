# Decision queue

When you hit a decision you cannot make alone during a ralph loop, use
the decision queue at `.claude/decision-queue.json` instead of stopping
the loop entirely.

## When to use

Use the queue when:
- You need advisor input on a scope or design question
- The plan is ambiguous and two reasonable interpretations exist
- You discover something unexpected that changes the approach

Do NOT use the queue for:
- Routine checkpoint reports (those go in the completion report)
- Questions you can answer by reading the codebase or design docs
- Anything the plan or rules already cover

## How to write a question

Read `.claude/decision-queue.json`, add an entry to `pending`:

```json
{
  "id": <next integer>,
  "from": "impl",
  "timestamp": "<ISO 8601>",
  "question": "<clear, specific question — one sentence>",
  "options": ["option-a", "option-b"],
  "context": "<what you checked that led to this question — 1-2 sentences>",
  "answer": null,
  "answered_by": null
}
```

Keep `question` under 50 words. Put evidence in `context`, not in the
question. Always provide at least two concrete `options` — never ask
open-ended questions.

## After writing a question

1. **Check if you can continue with other tasks.** If the blocked task
   is independent of the remaining tasks, move to the next task and
   come back when the answer arrives. Note in your progress log:
   "Task N blocked on decision-queue #M, continuing with task N+1."

2. **If you're fully blocked** (the answer gates all remaining work),
   stop the loop cleanly with a message noting the decision-queue ID.

3. **Never guess.** If you're tempted to pick an option and continue
   without waiting, that's a signal the question wasn't worth queuing.
   Either answer it yourself with evidence or queue it properly and wait.

## How to read an answer

At the start of each ralph iteration, read `decision-queue.json`. If
any `pending` entry has `answer` filled in:

1. Read the answer
2. Move the entry from `pending` to `resolved`
3. Write the updated file back
4. Apply the decision and continue

## Who answers

- **Advisor session** (homeserver) — reads the queue, writes `answer`
  and `"answered_by": "advisor"`
- **User** — may answer directly with `"answered_by": "user"`
- **Never answer your own questions** in the queue. If you realise the
  answer while waiting, write it as a new resolved entry with
  `"from": "impl", "answered_by": "impl-self-resolved"` and explain why.

## Concurrency

Only one writer at a time. The impl agent writes during ralph
iterations. The advisor or user writes between iterations. Since the
impl agent pauses between iterations, there is no true concurrent
write risk — but always read before writing to avoid clobbering.
