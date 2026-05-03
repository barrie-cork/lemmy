---
role: impl-task
plan_task: 3
phase: v1-SL-a
cohort: A
created: 2026-05-03
related_dq: []
---

# Brief — v1-SL-a Task 3 [P] — UPDATE `crates/db_schema_file/src/schema.rs` — add 2 columns to `moderation_case` table block

## 1. Role + dispatch line

`[role:impl-task] v1-SL-a task 3 — see .claude/PRPs/briefs/sl-a-impl-3.md`

You are the **impl-task** subagent (Sonnet 4.6) executing plan Task 3
of cohort A on `phase-v1-SL-a`. Cohort A = Tasks 1+2+3+6+7 (5-way
parallel per plan §13 line 1240); you own only
`crates/db_schema_file/src/schema.rs`.

## 2. Scope

**Produce:**

1. ONE commit modifying `crates/db_schema_file/src/schema.rs` —
   appending two new column declarations (`grace_expires_at` +
   `liability_escape_reason`) to the `moderation_case (id)` table
   block (per plan §10.3 verbatim).
2. ONE `kind: "validate-pending"` DQ entry referencing
   `cargo-validate-workspace.yml` (this task is Rust-only; no
   migration trigger).
3. Push your worker branch. Junior daemon finalize-merges into
   `phase-v1-SL-a`.

**Do NOT** in this task:

- Touch any other file (no `migrations/**`, no `enums.rs`, no other
  `crates/**` file, no `tests/**`, no docs). `schema.rs` is the
  only editable file.
- Touch the `surety` table block (lines 1343–1352). SL-a only adds
  an INDEX on `surety` (handled in Task 1's migration), no column
  edit.
- Touch any other table block in `schema.rs`. Cohort B tasks have
  no schema.rs ownership conflict, but neither do you have license
  to "tidy".
- Add `use super::sql_types::...` imports — `use diesel::sql_types::*`
  at line 764 already covers `Timestamptz` and `Jsonb`.
- Run cargo locally. Shape G — workspace-check fires on push.

## 3. Required reading

Read in this order before writing any code:

1. **Plan Task 3 body:** `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md`
   lines 1545–1588 — task header, FILES YAML, IMPLEMENT directives,
   GOTCHAs (4 of them).
2. **Plan §10.3 (canonical schema.rs skeleton):**
   `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` lines 750–771.
   The verbatim source for the two new column lines + the inline
   marker comment.
3. **Read the file you'll edit FIRST:**
   `crates/db_schema_file/src/schema.rs` lines 763–799 (the existing
   `moderation_case (id)` block). Confirm:
   - Line 763 (or near) opens the block.
   - Line 764 has `use diesel::sql_types::*`.
   - Line 797 (or near) has `winning_decision -> Nullable<JuryDecision>,`.
   - Line 798 (or near) is the closing `}`.
   If actual line numbers diverge, adapt and document in commit body.
4. **PRD §3.1 / §8.4** — column semantics
   (`grace_expires_at` = grace window deadline timestamp;
   `liability_escape_reason` = JSONB `{"version": 1, ...}` shape
   filled by SL-b/c/d, not SL-a).

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 0
    commit: 1f6131bf823f006bd2ace06b2931e2c1b56cd8ad
    filesCreated: []
    filesModified:
      - scripts/brehon/migrate-roundtrip.sh
    keyDecisions:
      - replaced stub per DQ #114 (Option A: Task 0 of SL-a)
      - "lemmy_diesel_utils binary surface — no CLI sub-commands; uses
        LEMMY_DATABASE_URL env var only (binary at
        crates/diesel_utils/src/main.rs reads from env, takes no positional
        args). Adapted plan §13 GOTCHA cargo invocations accordingly."
    notes: |
      Probe 3 (clean of SL-a columns) PASSED on phase-v1-SL-a tip — neither
      grace_expires_at nor liability_escape_reason currently present in
      schema.rs. Confirms Task 3 starting state.
```

Task 0's Probe 3 result confirms the file is clean of your target
columns at task start.

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branch derived from `phase-v1-SL-a`
  (tip `1b47a3ff4`).
- `git branch --show-current` must return your `junior/<task-slug>`
  branch. Confirm `git merge-base --is-ancestor phase-v1-SL-a HEAD`
  exits 0 before any write.
- ONE source-code commit. Daemon finalize absorbs into `phase-v1-SL-a`.

### File discipline

- Files you may MODIFY: `crates/db_schema_file/src/schema.rs` ONLY.
- **Files you may NOT touch:** any `migrations/**`, `enums.rs`, any
  other `crates/**` file, `tests/**`, `Cargo.toml`, `Cargo.lock`,
  `rust-toolchain.toml`, docs, scripts. If you find yourself
  wanting to touch any of these, STOP and file a DQ.

### Edit discipline

- **Use Edit (not Write).** schema.rs is large (multi-thousand lines).
  A surgical Edit between `winning_decision` and the closing `}` of
  the `moderation_case (id)` block is the only mutation.
- **Verbatim from plan §10.3** including:
  - The marker comment `// v1-SL-a additions:` immediately before
    the new lines (per Task 3 GOTCHA "schema.rs is `@generated`-style
    hand-edited per project convention").
  - `grace_expires_at -> Nullable<Timestamptz>,` (capitalisation,
    `Nullable<>`, comma exact).
  - `liability_escape_reason -> Nullable<Jsonb>,` (capitalisation,
    `Nullable<>`, comma exact).
- **No new `use` imports.** The block-level
  `use diesel::sql_types::*;` at line 764 already covers
  `Timestamptz` and `Jsonb`. Adding `use super::sql_types::Timestamptz`
  or similar is a defect.
- **Do NOT touch the `surety` table block** at lines 1343–1352 (per
  Task 3 GOTCHA "surety table block needs NO edit"). The Task 1
  migration adds a partial index on `surety`, but `table!` macro
  only tracks columns + foreign keys, not indexes.
- **No partial-index declarations.** Postgres-side indexes from
  Task 1 are tracked in `pg_indexes`, not `schema.rs` (per Task 3
  GOTCHA "two new partial indexes are NOT declared here").

### Shape G discipline (no local cargo)

- Push your worker branch to `junior/<task-slug>`. The push to
  `crates/**` triggers `cargo-validate-workspace.yml`. **Migration
  workflow does NOT trigger** (no `migrations/**` change in this
  task).
- Capture the workflow run id:
  ```bash
  gh run list --repo barrie-cork/lemmy --branch <branch> --workflow cargo-validate-workspace --limit 1 --json databaseId
  ```
- Write ONE `kind: "validate-pending"` DQ entry per
  `.claude/rules/decision-queue.md` Recipe 1: distinct `id`,
  `from: "impl"`, `kind: "validate-pending"`, `workflow_run_id: <id>`,
  `branch: "<your branch>"`, `phase_task: 3`, `result: null`,
  `log_slice: null`, `failed_jobs: null`, `answer: null`,
  `answered_by: null`, `resolved_at: null`. `context` says
  "workspace-check for v1-SL-a task 3".
- Commit + push the DQ entry to your worker branch immediately per
  "Mid-task visibility". Two commits acceptable: (1) `feat(v1-SL-a):
  extend moderation_case ...`, (2) `chore(decision-queue): impl
  raised DQ #N — sl-a-task-3 validate-pending`.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.
- Compute next id by scanning **both** live `pending`/`resolved` AND
  any `decision-queue-archive-*.json`. **Cohort A coordination:**
  Tasks 1 + 2 + 3 are running in parallel; the live file may have
  been updated by Task 1 / Task 2's DQ writes between your `git
  pull` and your DQ write. Re-read the file immediately before
  computing next id.

### Commit shape

- ONE source-code commit:
  - Subject: `feat(v1-SL-a): extend moderation_case table block with grace_expires_at + liability_escape_reason columns (task 3)` (verbatim per plan §13 line 1588).
  - Body: include the `HANDOVER:` YAML trailer:
    ```yaml
    HANDOVER:
    filesCreated: []
    filesModified:
      - crates/db_schema_file/src/schema.rs
    keyDecisions:
      - "added 2 columns (grace_expires_at, liability_escape_reason) to moderation_case (id) per plan §10.3 verbatim"
      - "// v1-SL-a additions: marker preserved per @generated-style hand-edit convention"
      - <any OBSERVED line-number divergence from plan §10.3 with reason>
    notes: <e.g. "no new `use` imports; existing diesel::sql_types::* covers Timestamptz/Jsonb">
    ```
- THEN the DQ-entry commit (separate).
- Files in source-code commit: ONLY `crates/db_schema_file/src/schema.rs`.
  Files in DQ commit: ONLY `.claude/decision-queue.json`.

## 5. Validation gate

Plan §15 names these (run in GH Actions on push):

- `cargo check --workspace --features full` (workspace workflow).
- `cargo clippy --workspace --features full --no-deps -- -D warnings`
  (workspace workflow). The two new columns introduce no clippy
  surface area on their own. **However**, Task 4 (cohort B) is the
  task that extends `ModerationCase` struct + `InsertForm` to mirror
  these columns. Until Task 4 lands, downstream `crates/db_schema/
  src/source/governance/moderation_case.rs` uses of the
  `moderation_case::table` type may produce missing-field errors.
  Advisor §G4 classifier recognises this as an EXPECTED cohort-A
  barrier, not a Task-3 defect.

If workspace-check fails on a Task-3-specific error (typo in column
name, wrong type, missing comma), advisor catch-fires.

## 6. Expected output (return to advisor)

```
## Task 3 complete — v1-SL-a moderation_case schema extended

**Branch:** junior/role-impl-task-...
**Commits:**
  - <sha-a> feat(v1-SL-a): extend moderation_case ... (task 3)
  - <sha-b> chore(decision-queue): impl raised DQ #<N> — sl-a-task-3 validate-pending
**File:** crates/db_schema_file/src/schema.rs
**Lines edited:** moderation_case (id) block — inserted 3 lines (marker comment + 2 columns) between `winning_decision` and closing `}`
**No new `use` imports:** confirmed (block-level `use diesel::sql_types::*` covers Timestamptz/Jsonb)
**`surety` table block at lines 1343–1352:** unchanged (confirmed)
**Workflow run captured:**
  - workspace-check workflow_run_id: <int> → DQ #<N>
**Expected workspace-check outcome:** missing-field errors in `crates/db_schema/src/source/governance/moderation_case.rs` until Task 4 lands — this is the COHORT BARRIER, not a Task-3 defect
**Next:** advisor dispatches 1 ci-watcher; cohort A (Tasks 1+2+3+6+7) advances per plan §14 Story 1 (workspace-check success on Task 5's push, post cohort B)
```

If any pre-write check failed (file path missing, plan §10.3
line range diverges past adaptation), replace with a DQ catch-fire
entry committed + pushed to your worker branch immediately.
