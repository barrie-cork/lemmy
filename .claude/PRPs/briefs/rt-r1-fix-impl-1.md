---
phase: v1-RT-r1
role: impl-task
task: fix-1
brief_n: 1
authored: 2026-05-11
parent_worker_branch: junior/role-impl-task-v1-rt-r1-tasks-6-7-bundled-schema-rs-diesel-struct-extensions-see-claude-prps-briefs-rt-r1-impl-225
parent_dq: 204
---

# [role:impl-task] v1-RT-r1 fix-impl-1 — add `dedupe_key: None, source_event_type: None` to two `ReputationEventInsertForm` literals — see .claude/PRPs/briefs/rt-r1-fix-impl-1.md

## §1 Role + dispatch

`[role:impl-task] v1-RT-r1 fix-impl-1 — pad two ReputationEventInsertForm literals with the v1 Option fields`

## §2 Scope

### §2.1 §G4 CANONICAL RECIPE (mechanical, two-call-site struct-literal padding)

This sits adjacent to the §G4 classifier allowlist (compiler-pointed mechanical edit, ≤2 file edits, ≤4 line additions). Failure signature on parent DQ #204 (workflow run `25669550114` at worker tip `832e373d2`):

```
error[E0063]: missing fields `dedupe_key` and `source_event_type`
   in initializer of `ReputationEventInsertForm`
  --> crates/api/api/src/governance/sponsor_liability.rs:296:16
  --> crates/api/api/src/governance/submit_jury_vote.rs:935:14
```

Two existing callers build `ReputationEventInsertForm { ... }` as a struct literal; Task 7 added two new fields (`dedupe_key: Option<String>`, `source_event_type: Option<ReputationEventSourceType>`) and they no longer compile.

### §2.2 Specific fix

Both fields are `Option<>` per `feedback_insertform_default_propagation.md`. Callers pass `None`; the NOT-NULL DB DEFAULT (`Endorsement`) covers `source_event_type` at the column level. `dedupe_key` is nullable in the schema. Substrate-only sub-phase — r2/r3 will start populating these.

**Edit 1 — `crates/api/api/src/governance/sponsor_liability.rs:296`:**

Inside the `let form = ReputationEventInsertForm { ... }` block whose last field is currently `expires_at: None,`, append two lines before the closing `};`:

```rust
      dedupe_key: None,
      source_event_type: None,
```

**Edit 2 — `crates/api/api/src/governance/submit_jury_vote.rs:935`:**

Inside the `emit_reputation_event` helper's `let form = ReputationEventInsertForm { ... }` block (last field also `expires_at: None,`), append the same two lines.

**Only four added lines total across two files.** No reordering of existing fields. No comment changes. No other edits.

### §2.3 Validation

After editing:

```bash
rg -nC1 "dedupe_key: None" crates/api/api/src/governance/
# Expected: exactly 2 hits, one in each file
rg -nC1 "source_event_type: None" crates/api/api/src/governance/
# Expected: exactly 2 hits, one in each file
```

Then push the worker branch. The push triggers `cargo-validate-workspace.yml` (path filter `crates/**` matches). Raise a NEW `kind: "validate-pending"` DQ entry with the fresh `workflow_run_id` from:

```bash
gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json databaseId
```

## §3 Required reading

- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §10.7 — `ReputationEventInsertForm` field shape: `dedupe_key: Option<String>`, `source_event_type: Option<ReputationEventSourceType>`
- `.claude/lessons/feedback_insertform_default_propagation.md` — why callers may pass `None` and rely on the NOT-NULL DB DEFAULT
- `.claude/rules/decision-queue.md` Recipe 1 — DQ entry shape for `kind: "validate-pending"`
- `.claude/rules/advisor-orchestrator.md` §G4 classifier — Shape-G two-phase validation flow
- `crates/db_schema/src/source/governance/reputation_event.rs` on this worker branch tip (after pulling) — confirms `ReputationEventInsertForm` struct lines 40+50+51 carry the two new Option fields

## §3a Handover from prior task

- **Parent task (Junior #225):** `[role:impl-task] v1-RT-r1 tasks 6+7 BUNDLED`. Brief: `.claude/PRPs/briefs/rt-r1-impl-6-7-bundle.md`. Status: Junior **succeeded**; commits `03f6c670e` (schema.rs) + `05cf5ae1d` (Diesel structs) landed on worker branch `junior/role-impl-task-v1-rt-r1-tasks-6-7-bundled-...-225`.
- **Workflow result:** `cargo-validate-workspace` run `25669550114` **failed** (workspace compile error E0063 — two call-site literals missing the new fields). Workflow run logs surface the two file:line pointers verbatim.
- **DQ #204:** raised by impl post-push at commit `832e373d2`; advisor mutated to `result: fail` with the workspace log slice at commit `cd7f8dd52` (worker branch tip). Stays in `pending[]` per option-2 failure semantics.
- **Parent worker branch is NOT finalize-merged to `phase-v1-RT-r1` yet** — the fix-impl commit lands on the SAME worker branch (`junior/role-impl-task-v1-rt-r1-tasks-6-7-bundled-...-225`) so the daemon's finalize-merge carries both the bundled tasks 6+7 commits AND this fix into the phase branch atomically. Do NOT cut a new junior branch off the worker branch; do NOT cut off `phase-v1-RT-r1` (would orphan the bundled commits).

## §4 Constraints

- **Files:** only `crates/api/api/src/governance/sponsor_liability.rs` and `crates/api/api/src/governance/submit_jury_vote.rs`. No other files.
- **Edits:** exactly 4 lines added (2 per file). No deletes, no reorders, no comment changes.
- **Branch:** check out the parent worker branch FIRST. Do not branch off:

  ```
  git fetch origin
  git checkout junior/role-impl-task-v1-rt-r1-tasks-6-7-bundled-schema-rs-diesel-struct-extensions-see-claude-prps-briefs-rt-r1-impl-225
  ```

  Commit directly on top of `cd7f8dd52`.

- **Shape G:** after committing, push the worker branch; capture the resulting `cargo-validate-workspace` workflow_run_id; write a `kind: "validate-pending"` DQ entry per `decision-queue.md` Recipe 1. Compute `next_id` including `decision-queue-archive-*.json` files (per c-2 retro watch-item #4).
- **DQ mid-task push:** if you hit a DQ blocker (e.g. the struct literal you reach does not match this brief's description — different fields, different name, different file:line), commit + push immediately per `decision-queue.md` "Mid-task visibility".
- **COMMIT MESSAGE:** `fix(v1-RT-r1): pad ReputationEventInsertForm literals with dedupe_key + source_event_type (fix-impl-1)`

## §5 Out of scope

- Any cargo-validate-migration consideration — this fix changes no migration files.
- Adding non-`None` *values* for `dedupe_key` or `source_event_type` per call site — both are still `None` (substrate-only sub-phase; r2/r3 will start populating them per PRD §5.3).
- Editing `ReputationEventInsertForm` itself — Task 7's struct definition is correct as shipped on the worker branch.
- Adding builder helpers to elide the `None`s — plan §10.7 did not introduce a builder; the struct-literal pattern stays as-is.
- Any work outside `crates/api/api/src/governance/` — if rg shows other call sites of `ReputationEventInsertForm { ... }` (e.g. tests), file a DQ pending entry. Plan §13 Task 7 (already shipped on this branch) was supposed to update all callers; this fix is bounded to the two compile-error sites.
