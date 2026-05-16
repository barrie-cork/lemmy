---
phase: v1-ship-1
role: impl-task
task: 1
brief_n: 1
plan: .claude/PRPs/plans/v1-ship-1-r1.plan.md
created: 2026-05-16
related_dq: null
---

# [role:impl-task] v1-ship-1 task 1 — DTOs — see .claude/PRPs/briefs/v1-ship-1-impl-1.md

## §1 Role + dispatch

`[role:impl-task] v1-ship-1 task 1 — see .claude/PRPs/briefs/v1-ship-1-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 1
from `.claude/PRPs/plans/v1-ship-1-r1.plan.md` §13 (Task 1, lines
~717-786). This is the **solo DTO barrier** — every downstream task
(`requires:` Task 1) depends on the three types you add here.

## §2 Scope

**Produce** (exactly one commit):

- `crates/db_views/site/src/api.rs` — append THREE new public structs
  (`SourceDisclosure`, `GetSource`, `GetSourceResponse`) immediately
  after the closing `}` of `GetSiteResponse` (plan cites current line
  355 — **verify by `grep -n "^pub struct GetSiteResponse {" ` first;
  the line may have drifted, the symbol is the contract**). Use the
  **verbatim** struct bodies + derives + doc-comments from plan §10.1
  / §13 Task 1 IMPLEMENT block (lines ~732-768). Copy them
  character-for-character.

**Do NOT** in this task:

- Add the `source_disclosure` field to `GetSiteResponse` — that is
  **Task 2** (keeps the field-add atomic with handler population;
  adding it here breaks the workspace compile because no constructor
  populates it).
- Touch `read.rs`, `build.rs`, `mod.rs`, `lib.rs`, `source.rs`,
  `e2e.rs` — those are Tasks 2/3/4.

**Commit message** (exactly):
`feat(db_views_site): add SourceDisclosure, GetSource, GetSourceResponse DTOs (task 1)`

## §3 Required reading

In this order:

1. **`.claude/PRPs/plans/v1-ship-1-r1.plan.md` §13 Task 1** (lines
   ~717-786) — the canonical step list + verbatim struct bodies.
2. **Plan §10.1** — the DTO derive shape + verbatim doc-comment text
   (the §13 Task 1 IMPLEMENT block already inlines it; §10.1 is the
   cross-reference).
3. **MIRROR ref:** `crates/db_views/site/src/api.rs:337-355` — the
   already-present `GetSiteResponse` struct, for derive-shape
   alignment. NOTE: `GetSiteResponse` uses `#[skip_serializing_none]`
   because it has `Option` fields; `SourceDisclosure` and
   `GetSourceResponse` have **no `Option` fields** — OMIT
   `#[skip_serializing_none]` on the new structs (per plan §10.1
   MIRROR note).
4. **`.claude/lessons/feedback_clippy_doc_lazy_continuation_in_doc_comments.md`**
   — `clippy::doc_lazy_continuation` fires on `/// line1\n///   line2`
   patterns. The plan §10.1 verbatim doc-comments already comply
   (single-line `///` or blank `///` between paragraphs). Do NOT
   reformat them; copy verbatim.
5. **`.claude/lessons/feedback_clippy_test_style.md`** — the Lemmy
   workspace denies `unwrap`/`expect`/`#[allow]` escape-hatches under
   `-D warnings`. (Not directly exercised by a pure struct-add, but the
   clippy validation gate enforces it workspace-wide.)
6. **`.claude/lessons/feedback_pipes_mask_exit_codes.md`** — capture
   cargo output to a file, THEN check exit code, THEN tail separately.
   Never pipe cargo through tail/grep when you need the exit status.

> **PMD note:** the project-memory MCP is not wired in this lane
> worktree (`.mcp.json` + `.project-memory/` are gitignored, not
> propagated to `git worktree add` checkouts). The §2.3 hybrid PMD
> search was substituted by a direct `.claude/lessons/` grep at brief
> authorship; the relevant lessons are listed above. No mandatory
> file-class row fires for a pure `crates/db_views/site/src/api.rs`
> struct-add (per `.claude/rules/advisor-orchestrator.md` §2.4).

## §3a Handover from prior cohort

(none — Task 1 follows Task 0 which is a non-`[P]` verification-only
pre-flight; no `HANDOVER:` trailer to propagate. Task 0 confirmed the
`GetSiteResponse` anchor exists via Probe 6.)

## §4 Constraints

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-ship-1`. Finalize
  merges your worktree branch back; do NOT push to `phase-v1-ship-1`
  directly.
- **One commit.** If clippy/check fails on the first attempt, amend or
  fixup — do NOT split into multiple commits.
- Mid-task DQ visibility: if you raise a `pending` entry, `git add
  .claude/decision-queue.json && git commit && git push origin
  <worktree-branch>` immediately. Compute `next_id` across
  `.claude/decision-queue.json` + `.claude/decision-queue-archive-*.json`
  (the DQ #50 collision lesson — live max id is currently 235; archives
  may hold higher).
- Attribution: `from: "impl"`, `answered_by: null`. NEVER
  `from: "advisor"` / `answered_by: "advisor"` / `answered_by: "user"`
  / `kind: "clarify"`.

### Plan-cited line numbers may have drifted

The plan cites `GetSiteResponse` closing `}` at line 355. **Verify
with `grep -n "^pub struct GetSiteResponse" crates/db_views/site/src/api.rs`
and read the struct to find its real closing brace before appending.**
If the line drifted, follow the grep, not the plan number (per
`feedback_plan_baseline_self_reference.md`). The symbol presence is
the contract; the exact line is advisory.

### Lesson trailer (encouraged)

If you discover a non-obvious constraint a future impl-task on this
area would want, end the commit body with a `LESSON:` line (one
discrete lesson, cite file:line) per
`feedback_junior_pmd_write_convention.md`.

### Shape-G suspended until 2026-06-01 — validation handoff

**Shape G is suspended repo-wide** (DQ #228/#229). After committing
+ pushing your worktree branch, write a
`kind: "validate-pending-laptop"` DQ entry (NOT `kind:
"validate-pending"`; do NOT capture a `workflow_run_id` — the GH
Actions workflow `cargo-validate-workspace.yml` is DISABLED and will
not fire). The advisor laptop session runs the cargo commands and
mutates the entry. Entry shape per `.claude/rules/decision-queue.md`
§"kind: validate-pending-laptop":

```
{
  "id": <next_id across live + archives>,
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO 8601 UTC>",
  "question": "v1-ship-1 task 1 DTOs — laptop cargo validation pending",
  "branch": "<your worktree branch>",
  "phase_task": 1,
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full",
    "bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings",
    "bash scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e"
  ],
  "context": "Task 1 added 3 DTO structs to crates/db_views/site/src/api.rs; awaiting advisor-laptop §5.2 validate-pending-laptop run.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

Commit + push that DQ entry on your worktree branch (subject:
`chore(decision-queue): impl raised DQ #<id> — v1-ship-1 task1 validate-pending-laptop`).

## §4.1 CANONICAL CASE OVERRIDE

Not applicable — Task 1 authors no e2e test. No `LemmyResult` /
`Box<dyn Error>` Case A/B/C decision (that is Task 4, pre-resolved as
Case A in plan §10.6 GOTCHA 1).

## §5 Validation gates (per plan §13 Task 1 VALIDATE block)

The VALIDATE block in plan §13 Task 1 is written in Shape-G form
(`git push origin junior/v1-ship-1-task-1` + capture
`workflow_run_id`). **Shape G is suspended** — instead:

1. Commit per §2 commit message.
2. Push your worktree branch (the daemon-cut `junior/...` branch).
3. Write the `kind: "validate-pending-laptop"` DQ entry per §4 above
   (commands: cargo check / clippy / test --no-run, all `--workspace
   --features full` except test which is `-p lemmy_server --test
   e2e`). The advisor-laptop runs them; you do NOT run cargo yourself
   (the daemon has `MemoryMax=10G` and cargo on the worker is
   forbidden per `.claude/rules/advisor-orchestrator.md`).
4. Return the §6 expected output.

Do NOT `#[allow]`-spam to make clippy pass — the structs are verbatim
from §10.1 and already clippy-clean; if clippy fails on the
advisor-laptop run, that's a real signal the advisor triages, not
something you patch around.

## §6 Expected output (return to advisor)

```
## Task 1 complete — v1-ship-1 DTOs

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/db_views/site/src/api.rs (+3 structs: SourceDisclosure, GetSource, GetSourceResponse appended after GetSiteResponse at line <actual>)
**Validation:** kind:"validate-pending-laptop" DQ #<id> raised on <worktree-branch> (Shape G suspended — advisor-laptop runs cargo)
**Next:** advisor runs §5.2 validate-pending-laptop; on pass, dispatches Cohort A (Tasks 2+3)
```

Plus any DQ #N references if you raised a blocker mid-task.

## §7 Why this brief differs from the plan

One deviation, session-state-driven (not a plan defect):

1. **VALIDATE block: Shape-G → Shape-G-suspended-laptop.** Plan §13
   Task 1's VALIDATE block captures a `workflow_run_id` from
   `cargo-validate-workspace.yml`. That workflow is DISABLED repo-wide
   until 2026-06-01 (DQ #228/#229; bootstrap §5; commit `086cfa5d4`).
   Per `.claude/rules/decision-queue.md` §"kind:
   validate-pending-laptop" + `.claude/rules/advisor-orchestrator.md`
   §5.2, impl-task writes `kind: "validate-pending-laptop"` with the
   §15 DoD commands verbatim instead; the advisor laptop session runs
   them. This is the standing session directive, not a plan
   correction — the plan's design + IMPLEMENT blocks are unchanged.
