# Decision queue

When you hit a decision you cannot make alone during a ralph loop, use
the decision queue at `.claude/decision-queue.json` instead of stopping
the loop entirely.

## Schema (v2)

Top-level keys: `pending` (array), `resolved` (array), `schema_version`
(integer; currently `2`). The legacy `phase` field was removed in v2 —
it tracked the Brehon sub-phase but was never read by any agent or
rule and drifted four sub-phases stale (last bumped at "6", current is
v1-JM-d). Sub-phase membership is derivable from the entry's
`timestamp` + git history; no top-level field needed.

v2 adds the `kind` field on entries (`"blocker"` | `"log"` — see
"kind: blocker vs log" below) and standardises a `resolved_at` ISO 8601
timestamp on resolved entries (replacing the historical drift between
`answered_at` / `ts_resolved` / `resolved_timestamp` / missing).
**Do not rewrite historical entries** — pre-v2 idiosyncrasies stay as
the audit trail. Forward-only consistency.

If `schema_version` is missing or `1`, treat entries as v1 (no `kind`,
varied resolved-timestamp keys). When you read a v1 entry into a v2
write context, do not backfill `kind` or `resolved_at` — leave the
historical record untouched.

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
  "kind": "blocker",
  "timestamp": "<ISO 8601>",
  "question": "<clear, specific question — one sentence>",
  "options": ["option-a", "option-b"],
  "context": "<what you checked that led to this question — 1-2 sentences>",
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

Keep `question` under 50 words. Put evidence in `context`, not in the
question. Always provide at least two concrete `options` — never ask
open-ended questions.

## kind: "blocker" vs "log"

The `kind` field separates entries that gate a task from entries that
record a finding for later harvest. Both go in `decision-queue.json` so
they're committed/pushed atomically with the work that produced them,
but the advisor's polling loop only stops for `kind: "blocker"` pending
entries.

- **`kind: "blocker"`** — the writer cannot or will not proceed without
  an answer. The advisor's polling loop must surface these. Goes to
  `pending` with `answered_by: null`. This is the original DQ purpose.
- **`kind: "log"`** — the writer found something durable a future task
  on related code would have wanted to know (subtle constraint, plan
  inaccuracy, footgun). Goes **directly to `resolved`** with the writer
  as `answered_by` (`impl-self-resolved`, `bm-self-resolved`, etc),
  `answer` filled with the recommended action ("file as a lesson",
  "amend the brief template", "watchpoint for next phase"). The
  advisor harvests these at retro time and promotes durable ones to
  `.claude/lessons/`.

Use `kind: "log"` instead of writing a `LESSON:` commit-trailer when
the finding is gated to a specific question/decision pattern. Use a
`LESSON:` trailer when the finding is purely informational and would
clutter the queue. When in doubt, prefer `LESSON:` trailer — DQ should
stay tight.

Default `kind` (if missing on a pre-v2 entry) is treated as
`"blocker"` — preserves prior intent for the 51 entries already in
the file.

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
2. Set `resolved_at` to the current ISO 8601 timestamp (UTC)
3. Move the entry from `pending` to `resolved`
4. Write the updated file back
5. Apply the decision and continue

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

## Mid-task visibility (Junior worktrees)

When the writer is a Junior subagent (`impl-task`, `bm-task`, or any
non-interactive session running on a worktree on the EliteDesk), DQ writes
must commit and push immediately. Without this, the entry is trapped
in the worktree until Junior's finalize step runs at task end —
sometimes minutes, sometimes hours. The advisor's polling loop reads
`decision-queue.json` on `governance-v0` (or the phase branch) and
will not see entries that haven't been pushed.

**Required steps after writing a `pending` entry from a Junior
subagent:**

```
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): <role> raised DQ #<id> — <slug>"
git push origin <current-branch>
```

Use `<role>` as `impl`, `bm`, or `planner` per the entry's `from`
field. The slug is a 3-5 word handle from the question. Push to the
**current branch** (the worktree's phase branch or junior-task
branch), not to `governance-v0`. The advisor's `git fetch origin` on
its next poll picks up the branch's new HEAD and sees the entry.

**Foreground sessions (laptop, interactive):** the existing pattern
applies — write the entry, commit at the next natural break (per-task
commit, end-of-iteration commit, etc). The push happens when the user
or `bm-task` pushes the branch. No special mid-iteration push is
required because the advisor session is on the same machine and reads
the file directly.

**Why this asymmetry exists:** Junior's per-task isolation is
deliberate (per `feedback_parallel_agents_one_worktree_per_agent`).
The advisor cannot read worktree-local state without `git fetch`.
The mid-task push is the bridge — it preserves isolation while
restoring visibility.

## Subagents and attribution

Junior dispatches tasks to subagents named in the dispatch line
(`[role:planning|impl-task|bm-task]`). The subagent's identity
determines which `from` value is valid:

- `planning` subagent → `from: "planner"`. May pre-seed `pending` or
  `resolved` entries with `answered_by: "planner"` (forward-looking
  OQs the planner has a recommendation on).
- `impl-task` subagent → `from: "impl"`. Pending entries only —
  `answered_by: null`. Self-resolve only with
  `answered_by: "impl-self-resolved"`.
- `bm-task` subagent → `from: "bm"`. Pending entries with
  `answered_by: null`, or self-resolve as `"bm-self-resolved"`.

None of the subagents may write `answered_by: "advisor"` or
`answered_by: "user"`. Those labels are reserved for commits authored
by the persistent advisor session (label: `advisor`) or for entries
where the advisor relayed a user reply in-channel (label: `user`).
This rule applies to both foreground and Junior-dispatched
invocations.
