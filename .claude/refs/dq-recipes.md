# DQ recipes (copy-pasteable)

Three concrete operations subagents perform on `.claude/decision-queue.json`. Each recipe is the canonical sequence — drift from it produces the recurring failure modes called out in the audit trail (attribution breaches, missed mid-task pushes, log-as-blocker conflations, id collisions).

Companion: `.claude/rules/decision-queue.md` (schema, attribution, hard refusals, kind routing). This file is the procedural copy-paste; the rules file is the contract.

## Recipe 1: Raise a blocker (the work is stuck)

Use when the question genuinely gates progress and you need an answer before continuing. If you can self-resolve with evidence, prefer Recipe 2 instead.

```bash
# Step A: generate a v3 composite id (no more max-scan)
NEXT_ID="$(bash scripts/brehon/dq-v3-new-entry.sh)"
echo "next_id: $NEXT_ID"
# Step B: edit decision-queue.json — append to "pending" array
# Use Edit tool with the JSON literal below. Substitute <NEXT_ID>, <SLUG>, etc.
```

```json
{
  "id": <NEXT_ID>,
  "from": "impl",
  "kind": "blocker",
  "timestamp": "<NOW_ISO>",
  "question": "<one-sentence question, under 50 words>",
  "options": ["<concrete option a>", "<concrete option b>"],
  "context": "<what you checked, 1-2 sentences>",
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

```bash
# Step C: commit + push so advisor can see (REQUIRED — see decision-queue.md "Mid-task visibility")
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): impl raised DQ #<NEXT_ID> — <SLUG>"
git push origin <current-branch>
```

If the question gates this task and no other task can proceed, stop the loop cleanly with `blocked-on-DQ-#<NEXT_ID>` in your completion summary. If you can continue with independent work, do so and come back.

## Recipe 2: Self-resolve with a `kind: log` entry (the work proceeds, but a finding is worth recording)

Use when you discovered something a future task on related code would have wanted to know — a subtle constraint, a plan inaccuracy, a footgun — and you have a defensible action you took or recommend. The advisor harvests these at retro time.

```bash
# Step A: generate a v3 composite id (no more max-scan)
NEXT_ID="$(bash scripts/brehon/dq-v3-new-entry.sh)"
echo "next_id: $NEXT_ID"
# Step B: edit decision-queue.json — append directly to "resolved" array (NOT pending)
```

```json
{
  "id": <NEXT_ID>,
  "from": "impl",
  "kind": "log",
  "timestamp": "<NOW_ISO>",
  "question": "<the finding framed as a question or a 'should we…' statement>",
  "options": ["<the action taken>", "<the alternative not taken>"],
  "context": "<what you observed, 1-2 sentences>",
  "answer": "<recommended action: file-as-lesson, amend-brief-template, watchpoint-for-next-phase, etc>",
  "answered_by": "impl-self-resolved",
  "resolved_at": "<NOW_ISO>"
}
```

```bash
# Step C: commit + push (the kind: log entry rides the same commit + push as the work that produced it)
git add .claude/decision-queue.json <other files from the work>
git commit -m "feat(<scope>): <work title> (task <N>) + log DQ #<NEXT_ID>"
git push origin <current-branch>
```

A `kind: log` entry MUST go directly into `resolved` with the writer in `answered_by`. Do NOT put it in `pending` — it doesn't gate anything, and putting it in `pending` triggers the advisor's polling loop to stop unnecessarily (the audit found ~25% of historical resolved entries were log-shape but mistakenly raised as blockers, polluting the loop).

If the finding doesn't fit the question/decision shape and is purely informational ("future me should know X"), prefer a `LESSON:` commit-trailer over a `kind: log` entry. The trailer keeps the queue tighter.

## Recipe 3: Self-resolve a blocker without an answer arriving

Use when you raised a `kind: blocker` (Recipe 1), then while waiting you found a defensible answer yourself with evidence — typically by reading more of the codebase or a spec you missed.

```bash
# Step A: read decision-queue.json, find the pending entry by id
# Step B: edit it — fill answer, answered_by, resolved_at; move from pending → resolved
```

Edit the entry in-place with these fields set (other fields untouched):

```json
{
  "answer": "<the answer you found, with evidence cited>",
  "answered_by": "impl-self-resolved",
  "resolved_at": "<NOW_ISO>"
}
```

Then move the entry from the `pending` array to the `resolved` array — same JSON shape, just relocated.

```bash
# Step C: commit + push
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): impl self-resolved DQ #<ID> — <SLUG>"
git push origin <current-branch>
```

Self-resolution is a judgment call. The bar: would the advisor have answered the same way given the same evidence? If yes, self-resolve. If no, leave it pending and respect the gate.
