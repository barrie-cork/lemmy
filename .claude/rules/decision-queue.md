# Decision queue

When you hit a decision you cannot make alone during a ralph loop, use
the decision queue at `.claude/decision-queue.json` instead of stopping
the loop entirely.

## Schema

Top-level keys: `pending` (array), `resolved` (array), `schema_version`
(integer; currently `3`). v3 fields per entry: `id` (composite — see
§"Schema (v3)" below), `from`, `kind`, `timestamp`, `question`,
`options`, `context`, `answer`, `answered_by`, `resolved_at`,
`approved_by`, `approved_at`.

**Forward-only consistency.** Never rewrite pre-v3 entries. Treat
`schema_version` missing or `1` as v1 (no `kind`; varied resolved-
timestamp keys: `answered_at` / `ts_resolved` / `resolved_timestamp` /
missing); do not backfill `kind` or `resolved_at` on read. The legacy
`phase` top-level field was removed in v2 (drifted stale; sub-phase
derivable from `timestamp` + git history) — do not re-add.

## Schema (v3)

v3 is an additive, forward-only extension of schema-v2. All v2 fields and semantics are preserved. Do not rewrite pre-v3 entries.

Top-level keys are unchanged from v2: `pending` (array), `resolved` (array), `schema_version` (now `3`). The `dq-schema-v3-migrate.sh` migration adds two new fields to every existing entry and bumps `schema_version` to `3`.

**Composite id (post-v1-dq-schema-r1, 2026-05-21).** Every new DQ entry written after v3 ships uses `id: "<session_id>-<sequence>"` where `session_id` is a per-CC-session 12-hex UUID (cached in gitignored `.claude/.dq-session-id`) and `sequence` is a per-session monotonic 3-digit counter (`001`, `002`, ...). Example: `a1b2c3d4e5f6-001`. The global `next_id = max(all_ids) + 1` recipe is abolished for new v3 entries — id namespaces are now per-session, making collisions arithmetically impossible.

Pre-v3 entries keep their original integer `id` unchanged. Migration adds `id_v1: <int>` as an alias for back-compat readers. Citations like "DQ #50" continue to resolve via `id == 50` on legacy entries; new entries are cited as `DQ <prefix>-<seq>` (e.g. `DQ a1b2c3d4e5f6-001`).

Generate v3 ids via `bash scripts/brehon/dq-v3-new-entry.sh`. The script reads or creates `.claude/.dq-session-id`, scans the live DQ + archives for the highest sequence in this session, and prints the next composite id.

**Approval fields (post-v1-dq-schema-r1, 2026-05-21).** Every entry gains `approved_by: <user-id> | null` and `approved_at: <ISO8601> | null`. Migration adds both as `null` to all pre-v3 entries. Going forward, the advisor writes `approved_by` only after an `AskUserQuestion` user-gate relay on a judgment-heavy entry. These fields are **advisor-exclusive** — Junior subagents must never write a non-null `approved_by`.

**New hard refusals (v3, all sessions except the persistent advisor session):**

8. **NEVER write `approved_by` from a non-advisor session** — that field is advisor-exclusive, populated only after AskUserQuestion user-gate relay.
9. **NEVER use the abolished `next_id = max(all_ids)+1` recipe on v3 writes** — use `bash scripts/brehon/dq-v3-new-entry.sh`.

Archive files (`decision-queue-archive-*.json`) are NOT migrated to v3 — they retain schema_version 1 or 2 per PRECON-3 (forward-only; archive migration is permanently deferred).

## Archive policy

Live `decision-queue.json` holds entries from the current sub-phase + the last-completed sub-phase. Older entries move to dated archive files via `homeserver/scripts/dq-archive.sh` (idempotent, `--dry-run` supported). Triggers: live file >100 entries or >200 KB, Junior task-0 read drops content under context pressure, or **at sub-phase retro** (primary trigger).

Full procedure (script flags, archive `schema_version` rules, commit-subject pattern, "what does NOT get archived"): `.claude/refs/dq-mechanics.md` §"Archive policy".

### Next-id calculation (v3 — canonical)

Under schema-v3, **always** generate new entry ids via
`bash scripts/brehon/dq-v3-new-entry.sh`. The script returns a composite
`<session_id>-<seq>` (e.g. `a1b2c3d4e5f6-001`) drawn from a per-session
12-hex UUID and a monotonic per-session counter, so id namespaces never
intersect across worktrees or sessions. Cross-lane id deduplication is
no longer needed for new entries.

To append the new entry to the DQ, **always** use
`bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> [--pending]`.
The helper generates the id, injects it into the fragment, appends to
`resolved[]` (or `pending[]` with `--pending`), and writes the file —
zero inline Python. Authoring fragments via the Write tool also avoids
the Windows backslash-path mangling class (see
`feedback_windows_backslash_path_dq_via_write_fragment.md`).

### Next-id calculation (pre-v3 — historical only)

Pre-v3 used `max(all_ids, default=0) + 1` spanning live + archives. **Abolished** for new v3 writes (Hard refusal #9) — raises `TypeError` on mixed int/string ids. Do **not** copy this recipe; the v3 composite-id mechanism structurally eliminates the cross-lane race (`feedback_cohort_dq_id_collision.md`) + the DQ #50 collision incident (`e9fa1e01a`). Full pre-v3 historical context: `.claude/refs/dq-mechanics.md` §"Pre-v3 next-id calculation".

### What does NOT get archived

Pending entries (never), entries cited by name in any active rule/lesson/brief/template (move the citation first), entries from the active sub-phase or its predecessor. Detail: `.claude/refs/dq-mechanics.md` §"What does NOT get archived".

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
  Phase-2 e2e dispatch — see `.claude/refs/dq-mechanics.md`
  §"Two-phase validation under Shape G"), `answered_by: null`. Required fields at write time:
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
- **`kind: "validate-pending-laptop"`** — the impl-task subagent
  committed + pushed under a **pre-Shape-G plan** and the §15 cargo
  commands are delegated to the laptop advisor session (per the
  2026-04-28 task #47 decision; see `advisor-orchestrator.md`
  "Cargo never runs on the EliteDesk worker" + "validate-pending-
  laptop handler" sub-sections). Goes to `pending` with `from:
  "impl"`, `answered_by: null`. Required fields at write time:
  `commands` (string array, the §15 DoD lines verbatim including
  scope flags + `--features full`), `branch`, `phase_task`.
  Required nullable fields populated by advisor-laptop on mutation:
  `result` (null), `log_slice` (null), `failed_commands` (null).
  Mutation: advisor-laptop sets `result: "pass" | "fail"`, populates
  `log_slice` (last 100 lines of failing command on fail) +
  `failed_commands` (subset that exited non-zero), `answered_by:
  "advisor-laptop"`, `resolved_at`. On `pass` entry moves to
  `resolved[]`; on fail stays in `pending[]` for §G4 triage.
  Variant: `kind: "validate-pending-laptop-e2e"` for entries whose
  sole content is e2e (testcontainers + Docker required); same shape
  + same mutation, separate kind only for clarity in DQ scans.
  Writer: **impl-task** (Pre-Shape-G plans only). The advisor
  laptop session reads this entry and runs the commands locally; no
  Junior subagent is dispatched (the laptop IS the runner).

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

This option-2 single-entry mutation supersedes the historical two-entry
design (the abandoned design + the DQ #73 orphan incident that motivated
the switch, plus the deprecated `validate-result`/`validate-failed` kinds
and the two-phase Shape-G flow — **Shape G SUSPENDED until 2026-06-01**):
`.claude/refs/dq-mechanics.md` §"Deprecated kinds" + §"Two-phase validation under Shape G".

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

The advisor's polling loop reads `decision-queue.json` and routes by `(kind, status)` pair. **The full per-kind routing matrix is in `.claude/refs/dq-mechanics.md` §"Polling-loop routing per kind (canonical detail)"** — it's advisor-loop mechanism the orchestration skills (`/auto-phase`, `/check-dq`, `/start-brehon`) carry in their bodies. Quick shape: `(blocker, pending)` → surface via the DQ triage tree; `(log, pending)` → schema breach, catch-fire; `(clarify, pending)` → gates planning; `(validate-pending, pending)` pre-mutation → dispatch ci-watcher, post-mutation-failure → §G4 classifier; `resolved` variants → advance pipeline / historical. The advisor-side commit-subject pattern + surfacing decision tree live in `advisor-orchestrator.md` §"DQ triage decision tree".

## Recipes (copy-pasteable)

Three operations subagents perform on the queue. Procedural blocks (Python+bash+JSON literals) live in `.claude/refs/dq-recipes.md` — read just-in-time when writing or self-resolving an entry.

- **Recipe 1: raise blocker** — work is stuck; cannot self-resolve. Recipe in `.claude/refs/dq-recipes.md` §"Recipe 1".
- **Recipe 2: self-resolve with `kind: log`** — work proceeds; finding is worth recording for retro harvest. Goes directly to `resolved[]` with `answered_by: "<role>-self-resolved"`. Recipe in `.claude/refs/dq-recipes.md` §"Recipe 2". Never put a `log` in `pending[]` (~25% of historical resolved entries were misclassified there; polluted the polling loop).
- **Recipe 3: self-resolve a blocker** — you raised a blocker, then found a defensible answer with evidence. Recipe in `.claude/refs/dq-recipes.md` §"Recipe 3". Bar for self-resolve: would the advisor have answered the same given the same evidence?

Drift from the canonical recipe sequence produces the recurring failure modes the audit trail flagged (attribution breaches, missed mid-task pushes, log-as-blocker conflations, id collisions). The schema, attribution rules, hard refusals, and per-kind routing in this file are the contract; the refs file is the procedure.

## Hard refusals (write-side)

These are the recurring failure modes the audit and Phase-6 #37 incident produced. Every Junior subagent must refuse:

1. **NEVER write `"answered_by": "advisor"` from a non-advisor session.** This is the Phase-6 #37 process breach. The advisor label is reserved for commits authored by the persistent advisor session in its own writes — detected by commit-subject pattern (see "Attribution integrity" below). If you self-attribute under "advisor", that's a breach even if no one notices.
2. **NEVER reuse an existing id.** Under schema-v3, always generate ids via `bash scripts/brehon/dq-v3-new-entry.sh` — see "Next-id calculation (v3 — canonical)" above. The pre-v3 `max(all_ids, default=0) + 1` recipe is abolished (Hard refusal #9) and produces `TypeError` on the mixed int/string id corpus that exists post-migration. The DQ #50 incident (collision with #49, had to be relocated via `e9fa1e01a`) was the lesson that motivated v3.
3. **NEVER raise a `kind: blocker` for a question you can answer by reading the codebase, the plan, or `.claude/lessons/`.** The advisor-loop stop is expensive. If the answer is in a file you haven't read yet, read it first.
4. **NEVER skip the mid-task commit + push** for a Junior worktree write. Without the push, the entry is trapped on the worktree until Junior's finalize step. The advisor cannot see it. (See "Mid-task visibility" below for the full mechanism.)
5. **NEVER ask open-ended questions.** Always provide at least two concrete `options`. "What should I do?" is not a question; "should I take option-A (use feature X) or option-B (use feature Y) given <evidence>" is.
6. **NEVER write `kind: "clarify"` from a non-advisor session.** That kind is advisor-only — produced by `/brehon-clarify` at the pre-planning gate. A Junior subagent that writes `kind: "clarify"` has misunderstood the workflow (impl-task ambiguity → `kind: "blocker"` from `from: "impl"`; brief ambiguity at planning time is the advisor's responsibility, not the planner's).
7. **NEVER write `kind: "validate-result" | "validate-failed"` from any session.** Those kinds are DEPRECATED (option 2 supersedes; locked 2026-04-28). Applies to all sessions, ci-watcher included: ci-watcher MUTATES the existing `validate-pending` entry in place (never writes a new entry) per the §"ci-watcher mutation pattern" contract above. impl-task writes new `validate-pending` entries (Phase 1); advisor writes them for the Phase-2 e2e dispatch.

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
on author as a discriminator. (Origin incident: Phase 6 DQ #37 self-attributed
under the advisor label — corrected via `docs(phase-6): correct DQ #37 attribution`.)

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
