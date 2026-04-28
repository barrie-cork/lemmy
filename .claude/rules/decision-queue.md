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

## Archive policy

The live `decision-queue.json` should hold only entries from **the
current sub-phase + the last-completed sub-phase**. Older entries
move to a sibling archive file. This keeps task-0 reads cheap (the
file is loaded by every Junior subagent at task start) and the
mental model clean ("these are the *current* decisions").

### When to archive

Archive at one of three triggers, whichever fires first:

1. The live file passes **100 entries** or **200 KB**, OR
2. A Junior subagent's task-0 read is observed dropping content due
   to context pressure, OR
3. **At sub-phase retro** — the natural close-out moment. Archive
   everything that doesn't reference the new sub-phase. This is the
   primary trigger; the others are safety nets.

As of 2026-04-27 (post-v1-JM-c ship): live file holds 51 entries,
~128 KB. Comfortable. First archive likely lands at v1-JM-d retro.

### How to archive

Use `homeserver/scripts/dq-archive.sh` (idempotent, supports
`--dry-run`). It moves resolved entries with `id <= --cutoff-id` from
the live file to a dated archive file at
`.claude/decision-queue-archive-<sub-phase-slug>.json` (e.g.
`decision-queue-archive-pre-v1-JM-d.json`). The script:

- Reads both `pending` and `resolved`; only resolved entries with
  `id <= cutoff` are eligible (pending entries are never archived).
- Preserves entry shape exactly — does NOT rewrite historical
  idiosyncrasies (per the v2 forward-only rule above).
- Sets `schema_version` on the archive file to **the value the
  archived entries used**, not the live file's version. If the
  archive contains v1 entries (no `kind`, drift in resolved-timestamp
  keys), the archive file is `schema_version: 1`. Mixed-vintage
  archives use `schema_version: 1` (the lower of any contained version)
  so consumers know to handle drift.
- Commits both files in one commit with subject
  `chore(decision-queue): archive entries up to #<cutoff-id> for <sub-phase> retro`.

### Reading across live + archive

When a subagent needs an entry by id and it's not in the live file,
check the archive(s):

```bash
ls .claude/decision-queue-archive-*.json
# Read each until the id is found
```

### Next-id calculation MUST span both

When computing the next id for a new entry (per Recipe 1 / Recipe 2),
include the archive(s) in the max:

```python
import json, glob
def next_id():
    all_ids = []
    for path in ['.claude/decision-queue.json'] + glob.glob('.claude/decision-queue-archive-*.json'):
        data = json.load(open(path))
        all_ids += [e['id'] for e in data.get('pending',[]) + data.get('resolved',[])]
    return max(all_ids, default=0) + 1
```

The DQ #50 collision incident (`e9fa1e01a`) was the lesson here. With
archives in play, ignoring the archive when computing next_id is the
new way to reproduce that bug. Don't.

### What does NOT get archived

- **Pending entries** — never. Pending entries are by definition
  blocking current work.
- **Entries cited by name** in any active rule, lesson, brief, or
  template. If `decision-queue.md` references DQ #37 by id (it does),
  that entry stays live as long as the citation does. Move the
  citation to the archive file path before archiving the entry.
- **Entries from the active sub-phase or its predecessor.** These are
  load-bearing for the current advisor session and any subagent that
  may need to revisit recent decisions.

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

## kind: "blocker" vs "log" vs "clarify" vs "validate-pending"

The `kind` field separates entries by what they gate and who writes
them. All go in `decision-queue.json` so they're committed/pushed
atomically with the work that produced them, but the advisor's
polling loop applies different routing per kind.

- **`kind: "blocker"`** — the writer (impl/bm) cannot or will not
  proceed without an answer. The advisor's polling loop must surface
  these. Goes to `pending` with `answered_by: null`. This is the
  original DQ purpose. Writers: impl, bm.
- **`kind: "log"`** — the writer found something durable a future task
  on related code would have wanted to know (subtle constraint, plan
  inaccuracy, footgun). Goes **directly to `resolved`** with the writer
  as `answered_by` (`impl-self-resolved`, `bm-self-resolved`, etc),
  `answer` filled with the recommended action ("file as a lesson",
  "amend the brief template", "watchpoint for next phase"). The
  advisor harvests these at retro time and promotes durable ones to
  `.claude/lessons/`. Writers: impl, bm, planner.
- **`kind: "clarify"`** — the **advisor** raised a coverage question
  on a draft brief BEFORE the planning task is queued. Goes to `pending`
  with `from: "advisor"` and `answered_by: null` if mode=user-relay; or
  directly to `resolved` with `answered_by: "advisor"` if the advisor
  self-answered from the lessons corpus or prior plans. The advisor's
  stage-shape orchestrator gates the planning task on every clarify-DQ
  on this brief being resolved (per
  `.claude/rules/advisor-orchestrator.md` "Clarify gate"). Writer:
  **advisor only** — never impl, bm, or planner. Produced by
  `/brehon-clarify` (per `.claude/commands/brehon-clarify.md` and
  `feedback_clarify_before_plan.md`).
- **`kind: "validate-pending"`** — the impl-task subagent committed
  + pushed and is awaiting out-of-band cargo validation on GitHub
  Actions (Shape G, per `v1-validate-agent.plan.md` §4 + §10.7). Goes
  to `pending` with `from: "impl"` (or `from: "advisor"` for the
  Phase-2 e2e dispatch — see "Two-phase validation under Shape G"
  below), `answered_by: null`. Required fields at write time:
  `workflow_run_id` (integer, captured via `gh run list --branch
  <branch> --limit 1 --json databaseId`), `branch`, `phase_task` (the
  §13 task number). Required nullable fields at write time (populated
  by ci-watcher on mutation): `result` (null), `log_slice` (null),
  `failed_jobs` (null). The advisor's polling loop reads this entry
  and dispatches a `[role:ci-watcher]` Junior task to poll the
  workflow run. The originating impl-task is gated until ci-watcher
  mutates the entry. Writers: **impl-task** for Phase-1 workspace-
  check entries; **advisor** for Phase-2 e2e entries (the daemon
  finalize-merge triggers e2e on the phase branch tip, but only the
  advisor sees the new tip on its next poll, so the advisor raises
  the Phase-2 entry).

### ci-watcher mutation pattern (option 2, locked 2026-04-28)

ci-watcher does NOT write a separate result entry. It MUTATES the
existing `validate-pending` entry in place by matching
`workflow_run_id`, populating `result` + `log_slice` + `failed_jobs`
+ `answer` + `answered_by: "ci-watcher"` + `resolved_at`, and moving
the entry from `pending[]` to `resolved[]` only when
`result: "pass"`. Failures (fail / cancelled / timed_out / gh_unauth
/ run_not_found) stay in `pending[]` for advisor §G4 triage. The
entry's `kind` stays `"validate-pending"` regardless — kind records
what was raised, not the current state.

`result` enum values: `"pass" | "fail" | "cancelled" | "timed_out" |
"gh_unauth" | "run_not_found"`. `timed_out` uses the underscore
spelling to mirror GitHub's workflow `conclusion` API.

This option-2 pattern supersedes the historical two-entry design
(impl-task writes validate-pending; ci-watcher writes a sibling
validate-result or validate-failed). The two-entry pattern was
killed because nothing migrated the paired pending → resolved when
ci-watcher wrote a new entry — DQ #73 was orphaned for ~7 hours
before manual cleanup at `30597b436`. Single-entry mutation makes
the inconsistency impossible.

### Deprecated kinds (historical-only — do not use for new writes)

- **`kind: "validate-result"`** — DEPRECATED 2026-04-28 by option 2.
  ci-watcher MUST NOT write this kind. Reading-side: schema-v2
  consumers handle historical entries with this kind for back-compat.
  Two such entries exist on `governance-v0` as of 2026-04-28: DQ #74
  (the original two-entry validate-result) and the historical
  resolved version of DQ #73 at `30597b436` (manually migrated
  pending → resolved during the contamination cleanup; carries
  `kind: "validate-pending"` because it was migrated post-hoc, but
  paired with the deprecated DQ #74).
- **`kind: "validate-failed"`** — DEPRECATED 2026-04-28 by option 2.
  ci-watcher MUST NOT write this kind. Reading-side: schema-v2
  consumers handle historical entries with this kind. No such
  entries exist on `governance-v0` as of 2026-04-28; the deprecation
  is forward-looking only.

> **Note on enum values:** GitHub's workflow `conclusion` API returns
> `timed_out` (with underscore) for timeout state — the `result` enum
> mirrors GitHub's exact spelling. `run_not_found` covers the case
> where `gh run view <id>` fails with run-not-found (e.g. wrong
> branch, run garbage-collected, GitHub-side eviction); ci-watcher's
> pre-flight run-existence check mutates the paired entry with this
> result and exits 0.

### Two-phase validation under Shape G (option (b), locked 2026-04-28)

Per `cargo-test-e2e.yml` triggering on push to `phase-v1-*` only
(not `junior/*` worktree branches — option (b) defers e2e to the
phase-branch tip; saves ~80% of e2e runs across a sub-phase):

- **Phase 1 (workspace check on `junior/*`):** impl-task writes a
  `validate-pending` entry referencing the
  `cargo-validate-workspace.yml` run id. Goes to `pending`,
  `from: "impl"`. Advisor queues a ci-watcher to mutate it.
- **Phase 2 (e2e on `phase-v1-*`):** after Junior's daemon finalize-
  merges the impl-task worktree branch into the phase branch, the
  advisor's polling loop detects the new phase-branch tip on next
  `git fetch`. The push to `phase-v1-*` triggers
  `cargo-test-e2e.yml`. The advisor captures the e2e workflow_run_id
  via `gh run list --repo barrie-cork/lemmy --branch
  phase-v1-<phase> --workflow cargo-test-e2e --limit 1 --json
  databaseId`, and writes a NEW `validate-pending` entry,
  `from: "advisor"` (the advisor commit subject is `chore(advisor):
  raise e2e validate-pending for phase-v1-<phase> tip <sha>` per
  `^(chore|docs)\((advisor|decision-queue)\)`). Advisor queues a
  second ci-watcher to mutate it.

Both phases use the same single-entry mutation pattern. Cohort
advancement waits on both phases per the cohort dispatch rule in
`.claude/rules/advisor-orchestrator.md`.

Use `kind: "log"` instead of writing a `LESSON:` commit-trailer when
the finding is gated to a specific question/decision pattern. Use a
`LESSON:` trailer when the finding is purely informational and would
clutter the queue. When in doubt, prefer `LESSON:` trailer — DQ should
stay tight.

Use `kind: "clarify"` only at the pre-planning gate; never to
re-litigate a brief mid-planning or to clarify an impl-task brief.
Impl-task ambiguity uses `kind: "blocker"` from `from: "impl"`.

Default `kind` (if missing on a pre-v2 entry) is treated as
`"blocker"` — preserves prior intent for the 51 entries already in
the file.

### Polling-loop routing per kind

The advisor's polling loop reads `decision-queue.json` and routes by
`(kind, status)` pair:

- `(blocker, pending)` — surface to user via the DQ triage decision
  tree (advisor-answer / catch-fire / user-relay). The Junior task
  that raised it is gated on a resolution.
- `(blocker, resolved)` — historical record only. No action.
- `(log, resolved)` — harvest at retro time. No mid-loop action.
- `(log, pending)` — **schema breach** (per Hard refusals: log entries
  always go to resolved). Surface as catch-fire.
- `(clarify, pending)` — advisor's own backlog from `/brehon-clarify`.
  If mode=user-relay, surface to user via AskUserQuestion. The
  planning task is gated until every clarify-pending on the brief is
  resolved.
- `(clarify, resolved)` — historical record. No action.
- `(validate-pending, pending)` — two cases:
  - **Pre-mutation** (`result == null`): advisor dispatches a
    `[role:ci-watcher]` Junior task with brief filled from the
    entry's `workflow_run_id` + `branch` + `phase_task`. The
    originating impl-task (Phase 1) or the cohort barrier (Phase 2)
    stays gated until ci-watcher mutates the entry. Per
    `advisor-orchestrator.md` Stage-shape "Each impl-task complete
    (under Shape G)".
  - **Post-mutation, failure** (`result ∈ {"fail", "cancelled",
    "timed_out", "gh_unauth", "run_not_found"}`): advisor runs the
    §G4 classifier (per `advisor-orchestrator.md` "§G4 classifier"
    sub-section). Allowlist match → auto-queue narrow
    fix-impl-task. Non-allowlist → catch-fire to user with
    `log_slice` + `failed_jobs`.
- `(validate-pending, resolved)` — post-mutation success (`result ==
  "pass"`). Advisor advances the §13-task pipeline (cohort check
  / Phase-2 e2e dispatch). No mid-loop user surfacing.
- `(validate-result | validate-failed, *)` — DEPRECATED kinds.
  Schema-v2 readers must handle historical entries with these kinds
  for back-compat (DQ #74 + the resolved version of DQ #73 on
  `governance-v0`). Routing for historical entries: treat
  `(validate-result, resolved)` as the success-pass equivalent of
  `(validate-pending, resolved)` — the §13-task pipeline already
  advanced when this entry was historically written. No mid-loop
  action. Treat any other historical (kind, status) combinations
  here as bookkeeping; the advisor never receives new entries with
  these kinds because ci-watcher is hard-refused from writing them
  (Hard refusal #7 below).

## Recipes (copy-pasteable)

Three concrete operations subagents perform on the queue. Each recipe is the canonical sequence — drift from it produces the recurring failure modes called out in the audit trail (attribution breaches, missed mid-task pushes, log-as-blocker conflations, id collisions).

### Recipe 1: Raise a blocker (the work is stuck)

Use when the question genuinely gates progress and you need an answer before continuing. If you can self-resolve with evidence, prefer Recipe 2 instead.

```bash
# Step A: compute next safe id (no collisions)
python3 - <<'PY'
import json
data = json.load(open('.claude/decision-queue.json'))
all_ids = [e['id'] for e in data.get('pending',[]) + data.get('resolved',[])]
print(f'next_id: {max(all_ids, default=0) + 1}')
PY

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
# Step C: commit + push so advisor can see (REQUIRED — see Mid-task visibility)
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): impl raised DQ #<NEXT_ID> — <SLUG>"
git push origin <current-branch>
```

If the question gates this task and no other task can proceed, stop the loop cleanly with `blocked-on-DQ-#<NEXT_ID>` in your completion summary. If you can continue with independent work, do so and come back.

### Recipe 2: Self-resolve with a `kind: log` entry (the work proceeds, but a finding is worth recording)

Use when you discovered something a future task on related code would have wanted to know — a subtle constraint, a plan inaccuracy, a footgun — and you have a defensible action you took or recommend. The advisor harvests these at retro time.

```bash
# Step A: compute next safe id (same as Recipe 1)
python3 - <<'PY'
import json
data = json.load(open('.claude/decision-queue.json'))
all_ids = [e['id'] for e in data.get('pending',[]) + data.get('resolved',[])]
print(f'next_id: {max(all_ids, default=0) + 1}')
PY

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

### Recipe 3: Self-resolve a blocker without an answer arriving

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

## Hard refusals (write-side)

These are the recurring failure modes the audit and Phase-6 #37 incident produced. Every Junior subagent must refuse:

1. **NEVER write `"answered_by": "advisor"` from a non-advisor session.** This is the Phase-6 #37 process breach. The advisor label is reserved for commits authored by the persistent advisor session in its own writes — detected by commit-subject pattern (see "Attribution integrity" below). If you self-attribute under "advisor", that's a breach even if no one notices.
2. **NEVER reuse an existing id.** Always compute `max(all_ids, default=0) + 1` from both `pending` and `resolved`. The DQ #50 incident (collision with #49, had to be relocated via `e9fa1e01a`) was the lesson.
3. **NEVER raise a `kind: blocker` for a question you can answer by reading the codebase, the plan, or `.claude/lessons/`.** The advisor-loop stop is expensive. If the answer is in a file you haven't read yet, read it first.
4. **NEVER skip the mid-task commit + push** for a Junior worktree write. Without the push, the entry is trapped on the worktree until Junior's finalize step. The advisor cannot see it. (See "Mid-task visibility" below for the full mechanism.)
5. **NEVER ask open-ended questions.** Always provide at least two concrete `options`. "What should I do?" is not a question; "should I take option-A (use feature X) or option-B (use feature Y) given <evidence>" is.
6. **NEVER write `kind: "clarify"` from a non-advisor session.** That kind is advisor-only — produced by `/brehon-clarify` at the pre-planning gate. A Junior subagent that writes `kind: "clarify"` has misunderstood the workflow (impl-task ambiguity → `kind: "blocker"` from `from: "impl"`; brief ambiguity at planning time is the advisor's responsibility, not the planner's).
7. **NEVER write `kind: "validate-result" | "validate-failed"` from any session.** Those kinds are DEPRECATED (option 2 supersedes; PMD #156, locked 2026-04-28). ci-watcher mutates the existing `validate-pending` entry in place by matching `workflow_run_id` — populates `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by: "ci-watcher"` + `resolved_at`; moves `pending[]` → `resolved[]` only on `result: "pass"`. The entry's `kind` stays `"validate-pending"` (kind records what was raised, not current state). impl-task writes new `validate-pending` entries (Phase 1, `from: "impl"`); advisor writes new `validate-pending` entries for the Phase-2 e2e dispatch (`from: "advisor"`); ci-watcher MUTATES — never writes a new entry. Hard refusal applies to all sessions, including ci-watcher itself.

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
(`[role:planning|impl-task|bm-task|ci-watcher]`). The subagent's
identity determines which `from` value is valid:

- `planning` subagent → `from: "planner"`. May pre-seed `pending` or
  `resolved` entries with `answered_by: "planner"` (forward-looking
  OQs the planner has a recommendation on). Writes `kind: "blocker"`
  or `kind: "log"`.
- `impl-task` subagent → `from: "impl"`. Pending entries only —
  `answered_by: null`. Self-resolve only with
  `answered_by: "impl-self-resolved"`. Writes `kind: "blocker"`,
  `kind: "log"`, or `kind: "validate-pending"` (the last one
  introduced by Shape G in v1-validate-agent — the post-push
  validation handoff). impl-task continues writing as `from: "impl"`
  even when the entry carries `kind: "validate-pending"` (per DQ
  #63); the new `from: "ci-watcher"` is ci-watcher's only.
- `bm-task` subagent → `from: "bm"`. Pending entries with
  `answered_by: null`, or self-resolve as `"bm-self-resolved"`. Writes
  `kind: "blocker"` or `kind: "log"`.
- `ci-watcher` subagent → mutates an existing `kind: "validate-pending"`
  entry (option 2, locked 2026-04-28). Finds the entry by matching
  `workflow_run_id`; populates `result` (`"pass" | "fail" | "cancelled"
  | "timed_out" | "gh_unauth" | "run_not_found"`) + `log_slice`
  (last ~200 lines on fail, null otherwise) + `failed_jobs` (array
  on fail, null otherwise) + `answer` + `answered_by: "ci-watcher"` +
  `resolved_at`. Moves the entry from `pending[]` to `resolved[]` only
  on `result: "pass"`; failures stay in `pending[]` for advisor §G4
  triage. The entry's `kind` stays `"validate-pending"`; the entry's
  `from` stays as written (`"impl"` for Phase 1, `"advisor"` for
  Phase 2). Pinned to Haiku 4.5, narrow-tools (Read, Edit, Write, Bash).
  Never invokes cargo, never edits code in `crates/` / `migrations/`
  / `tests/` / `docs/` / `.github/workflows/`, never applies clippy
  auto-fixes (those are advisor §G4 classifier work), never writes a
  new DQ entry. Per `.claude/agents/ci-watcher.md`.
- **Advisor session** (homeserver, persistent, never on Junior worktree)
  → `from: "advisor"`. Writes the full triage spectrum: answers to
  pending blockers (`answered_by: "advisor"` on resolved entries from
  the DQ triage decision tree); user-relayed answers
  (`answered_by: "user"`); and pre-planning clarify entries via
  `/brehon-clarify` (`kind: "clarify"`, see "kind: clarify" above).
  **The advisor is the only writer of `kind: "clarify"`.**

None of the Junior subagents may write `answered_by: "advisor"` or
`answered_by: "user"`. Those labels are reserved for commits authored
by the persistent advisor session (label: `advisor`) or for entries
where the advisor relayed a user reply in-channel (label: `user`).
None of the Junior subagents may write `kind: "clarify"` — that kind
is advisor-only by design (clarify gates planning before any Junior
task runs). impl-task writes `kind: "validate-pending"` for Phase 1
(workspace check on `junior/*`); the advisor writes `kind:
"validate-pending"` for Phase 2 (e2e on `phase-v1-*` post-finalize-
merge); ci-watcher MUTATES existing `validate-pending` entries (does
not write new ones). The deprecated `kind: "validate-result" |
"validate-failed"` MUST NOT be written by any session. This rule
applies to both foreground and Junior-dispatched invocations.
