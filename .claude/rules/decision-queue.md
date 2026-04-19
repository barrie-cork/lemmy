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

## Attribution integrity (load-bearing)

The `answered_by` field is the audit trail for who made which decision.
Self-attribution under a label you don't own is a process breach.

**Hard rules for any non-advisor session** (impl, planner, orchestrator,
ralph loops, task-hopper sweeps, Agent *):

1. **NEVER write `"answered_by": "advisor"`.** This label is reserved
   for commits authored by the advisor session in its own writes to
   `decision-queue.json`. The only valid labels a non-advisor session
   may write are `"impl-self-resolved"`, `"user"` (when the user stated
   the answer in-channel), or `"planner"` (for plan-author pre-seeds).
2. **A pre-seeded "advisor" answer is not the same as an advisor answer.**
   If a plan document includes a recommended answer from the advisor,
   the planner writes `"answered_by": "planner"` and names the advisor
   in the `answer` text as the source ("per advisor review 2026-04-17").
3. **If impl needs advisor input and advisor hasn't answered:** queue as
   `pending` with `answered_by: null` and wait, or self-resolve with
   evidence labelled `"impl-self-resolved"`. Do not backfill the advisor
   label under any justification.
4. **Bulk pending-sweeps must preserve original `answered_by`.** A task-0
   decision-queue sweep that moves pending → resolved must not rewrite
   the `answered_by` field. If the original was `null`, the sweep sets
   it to `"impl-self-resolved"` with the iteration commit's SHA cited
   in the `answer` text.

**Detection:** an advisor-session commit that introduces an
`"answered_by": "advisor"` entry will always appear in git log with a
commit subject matching `^(chore|docs)\((advisor|decision-queue)\)` —
i.e. an explicit advisor/DQ-scoped commit, authored in a human-run
advisor session. If `"answered_by": "advisor"` appears in a commit
whose subject is `feat(...)`, a ralph iteration commit, or any other
non-`chore(advisor|decision-queue):` / non-`docs(advisor|decision-queue):`
subject, that is a process breach and must be corrected via a
`docs(attribution):` follow-up commit. The match is on commit-subject
pattern, not author identity — solo-dev single-author repos cannot rely
on author as a discriminator.

**Why this rule exists:** Phase 6 DQ #37 (governance_log relocation) was
self-attributed by an impl-side session under the advisor label and
committed without advisor review. The refactor stands — invariants hold
— but was logged as advisor-approved when it was not. This rule is the
process fix. See `docs(phase-6): correct DQ #37 attribution`.

## Concurrency

Only one writer at a time. The impl agent writes during ralph
iterations. The advisor or user writes between iterations. Since the
impl agent pauses between iterations, there is no true concurrent
write risk — but always read before writing to avoid clobbering.
