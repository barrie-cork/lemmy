---
role: impl-task
plan_task: 4
phase: v1-SL-a
cohort: B
created: 2026-05-03
related_dq: []
preallocated_dq: [128]
---

# Brief — v1-SL-a Task 4 — UPDATE `crates/db_schema/src/source/governance/moderation_case.rs` — extend `ModerationCase` + InsertForm

## 1. Role + dispatch line

`[role:impl-task] v1-SL-a task 4 — see .claude/PRPs/briefs/sl-a-impl-4.md`

You are the **impl-task** subagent (Sonnet 4.6) executing plan Task 4
(cohort B sequential, NOT `[P]`) on `phase-v1-SL-a`. This task adds two
new fields (`grace_expires_at`, `liability_escape_reason`) to both the
`ModerationCase` struct AND the `ModerationCaseInsertForm` struct in
the SAME file. Cohort B = Task 4 followed by Task 5; both serial,
neither parallel.

## 2. Scope

**Produce:**

1. ONE source-code commit modifying
   `crates/db_schema/src/source/governance/moderation_case.rs` —
   inserting two new fields after `winning_decision` in BOTH structs
   (verbatim per plan §10.4).
2. ONE `kind: "validate-pending"` DQ entry referencing
   `cargo-validate-workspace.yml` (Rust-only edit; no `migrations/**`
   trigger). **Pre-allocated DQ id: #128.**
3. Push your worker branch. Junior daemon finalize-merges into
   `phase-v1-SL-a` (or advisor manually finalizes if daemon skips
   per the known finalize-merge bug — see `feedback_junior_daemon_finalize_skips_when_worker_pre_pushes`).

**Do NOT** in this task:

- Touch any other file (no `migrations/**`, no `schema.rs`, no other
  `crates/**` file, no `tests/**`, no docs). The single file
  `crates/db_schema/src/source/governance/moderation_case.rs` is the
  only editable file.
- Add `AsChangeset` derives or other macros — `AsChangeset` already
  exists on `ModerationCaseInsertForm` (line 87) and covers the new
  fields automatically.
- Touch `#[skip_serializing_none]` (line 13) — its existing presence
  on `ModerationCase` ensures both new `Option<_>` fields are omitted
  from JSON when None.
- Add new `use` statements — existing imports cover both new field
  types (`use chrono::{DateTime, Utc}` line 2; `use serde_json::Value`
  line 10).
- Run cargo locally. Shape G — workspace-check fires on push.

**Commit message** (verbatim per plan §13 line 1645): `feat(v1-SL-a): extend ModerationCase + InsertForm with grace_expires_at + liability_escape_reason fields (task 4)`

## 3. Required reading

Read in this order before writing any code:

1. **Plan Task 4 body:** `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md`
   lines 1590–1645 — task header, FILES YAML, IMPLEMENT directives,
   GOTCHAs (4 of them).
2. **Plan §10.4 (canonical Rust skeleton):** verbatim source for the
   two new fields + their `///` doc-comments. Citations in plan body.
3. **Read the file you'll edit FIRST:**
   `crates/db_schema/src/source/governance/moderation_case.rs` lines
   1–130. Confirm current state on phase tip:
   - Line 13 has `#[skip_serializing_none]` (preserve).
   - Line 83 has `pub winning_decision: Option<JuryDecision>,` in
     `pub struct ModerationCase` (verified by advisor 2026-05-03; insert AFTER this line).
   - Line 87 has `AsChangeset` derive on `ModerationCaseInsertForm` (preserve).
   - Line 122 has `pub winning_decision: Option<JuryDecision>,` in
     `pub struct ModerationCaseInsertForm` (verified by advisor 2026-05-03; insert AFTER this line).
   If actual line numbers diverge from the verified positions above,
   **adapt to the live file** and document in the commit body — the
   plan was written before Tasks 2+3 shipped, but Tasks 2+3 modified
   `enums.rs` + `schema.rs` (NOT this file), so divergence here is
   unlikely.
4. **`.claude/lessons/feedback_insertform_default_propagation.md`** —
   load-bearing for this task. Both new fields are `Option<_>` so v0
   callers continue to compile via `..Default::default()`. Pre-impl
   grep: `git grep -l 'ModerationCaseInsertForm {'` — verify every
   literal-construction site uses `..Default::default()` spread; if
   any site enumerates fields exhaustively, add the two new fields
   to that site too (R3 pattern).
5. **`.claude/lessons/feedback_clippy_test_style.md`** — workspace
   denies `unwrap`/`expect`/`#[allow]` escape-hatches.
6. **`.claude/lessons/feedback_advisor_cr_enum_drift.md`** — same
   discipline for enum/struct field naming: cross-check plan §10.4
   verbatim against PRD §3.1 + OQ-025 references; if either disagrees
   with the other, file a DQ rather than guess.

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1-fix
    commit: a7f824047b21af41d349061d19c54dd785725b41
    filesCreated:
      - migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/up.sql
      - migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/down.sql
      - migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/up.sql
      - migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/down.sql
    filesDeleted:
      - migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql
      - migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/down.sql
    keyDecisions:
      - "split combined SL-a migration into enum-add (2026-05-03-000000-0000) + non-enum (2026-05-03-000100-0000) per Phase 5b Restoration precedent (Postgres rejects ALTER TYPE ADD VALUE referenced in same migration session)"
      - "migration count delta: SL-a contributes +2 migrations (was +1); Task 8 will reflect in PHASE_1_MIGRATION_COUNT"
    notes: |
      The PRD §8.5 + plan §10.1 both prescribed the combined shape; superseded by this split. Verified by DQ #126 cargo-validate-migration pass on workflow run 25278271449.
  - task: 2
    commit: bfc8eaa416498d6efd45e64d35e3b80a5466a39e
    filesModified:
      - crates/db_schema_file/src/enums.rs
    keyDecisions:
      - "added 3 CaseStatus variants (SponsorLiabilityPending/Fired/Escaped) per plan §10.2; verbatim doc-comments preserved"
      - "post-edit CaseStatus variant count = 12 (was 9); #[default] Open unchanged"
    notes: "variants inserted after AdminReview before closing }"
  - task: 3
    commit: c7a977078e1a2b84ec2a9c91ba4f86c8ac4a7b1c
    filesModified:
      - crates/db_schema_file/src/schema.rs
    keyDecisions:
      - "added 2 columns (grace_expires_at, liability_escape_reason) to moderation_case (id) per plan §10.3 verbatim"
      - "// v1-SL-a additions: marker preserved per @generated-style hand-edit convention"
    notes: "no new use imports needed"
```

**Load-bearing context for Task 4:**

- The Rust enum variants you'll reference in field doc-comments
  (`CaseStatus::SponsorLiabilityPending`, etc) ALREADY EXIST as of
  Task 2 commit. You don't need to add them.
- The Postgres columns this struct mirrors (`grace_expires_at TIMESTAMPTZ`,
  `liability_escape_reason JSONB`) ALREADY EXIST in the migration
  (Task 1-fix split half 2) AND in `schema.rs` (Task 3). Your Rust
  struct field types must match: `Option<DateTime<Utc>>` for the
  TIMESTAMPTZ column (per plan §10.3 + §10.4) and `Option<Value>`
  for the JSONB column. The `diesel(check_for_backend(diesel::pg::Pg))`
  attribute on `ModerationCase` cross-checks this at compile time.

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branch derived from `phase-v1-SL-a`
  (current tip `b8fb40ad9` after manual ci-watcher 96+97 finalize-merge).
- `git branch --show-current` must return your `junior/<task-slug>`
  branch. Confirm `git merge-base --is-ancestor phase-v1-SL-a HEAD`
  exits 0 before any write.
- ONE source-code commit. Daemon finalize-merges into `phase-v1-SL-a`
  (or advisor manually finalizes if daemon skips).

### File discipline

- Files you may MODIFY: `crates/db_schema/src/source/governance/moderation_case.rs` ONLY.
- **Files you may NOT touch:** any `migrations/**`, `schema.rs`,
  any other `crates/**` file, `tests/**`, `Cargo.toml`, `Cargo.lock`,
  `rust-toolchain.toml`, docs, scripts. If you find yourself wanting
  to touch any of these, STOP and file a DQ.

### Edit discipline

- **Use Edit (not Write).** Surgical insertion of two fields + their
  doc-comments after `winning_decision` in BOTH structs. The rest of
  the file is untouched. ~6-10 lines added per struct (2 fields × 2
  structs, with `///` doc-comment lines).
- **Verbatim from plan §10.4** — do not paraphrase doc-comments,
  re-order the two new fields, change snake_case names, or "improve"
  comment wording.
- **Field names + types must match phase-tip schema:**
  - `pub grace_expires_at: Option<chrono::DateTime<chrono::Utc>>,`
    (or `Option<DateTime<Utc>>` if existing imports use the short
    form — check the file's existing pattern at line 83's
    `winning_decision` neighbours and match).
  - `pub liability_escape_reason: Option<serde_json::Value>,`
    (or `Option<Value>` per file pattern).
- Insert in the **same order** in both structs (e.g. `grace_expires_at`
  first, then `liability_escape_reason`) for symmetry.
- **Both fields are `Option<_>`** — no exceptions. v0 callers must
  continue to compile via `..Default::default()` spread (per
  `feedback_insertform_default_propagation.md`).

### R3 pattern (struct extension grep sweep)

After your Edit but BEFORE commit, run:

```bash
git grep -l 'ModerationCaseInsertForm {'
```

For each result, open the file and verify the construction site uses
`..Default::default()` spread. If any site enumerates fields
exhaustively (rare, but exists in tests), add the two new fields to
that site in the SAME commit. The plan's GOTCHA references this.
Note any sites you fixed in the commit body.

### Shape G discipline (no local cargo)

- Push your worker branch. The push to `crates/**` triggers
  `cargo-validate-workspace.yml`. **Migration workflow does NOT
  trigger** (no `migrations/**` change in this task).
- Capture the workflow run id:
  ```bash
  gh run list --repo barrie-cork/lemmy --branch <branch> --workflow cargo-validate-workspace --limit 1 --json databaseId
  ```
- Write ONE `kind: "validate-pending"` DQ entry per
  `.claude/rules/decision-queue.md` Recipe 1: **id = 128**
  (pre-allocated by advisor; do NOT compute, do NOT use any other
  id), `from: "impl"`, `kind: "validate-pending"`,
  `workflow_run_id: <captured id>`, `branch: "<your branch>"`,
  `phase_task: 4`, `result: null`, `log_slice: null`,
  `failed_jobs: null`, `answer: null`, `answered_by: null`,
  `resolved_at: null`. `context` says "workspace-check for v1-SL-a
  task 4 (ModerationCase + InsertForm field extension)".
- Commit + push the DQ entry to your worker branch immediately per
  "Mid-task visibility". Two commits acceptable: (1) `feat(v1-SL-a):
  extend ModerationCase ... (task 4)`, (2) `chore(decision-queue):
  impl raised DQ #128 — sl-a-task-4 validate-pending`.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.
- Pre-allocated id #128 means **no read-modify-write race** with
  Task 5 (which will use #129). Do not scan for next-available — use
  #128 directly.

### Commit shape

- ONE source-code commit:
  - Subject: `feat(v1-SL-a): extend ModerationCase + InsertForm with grace_expires_at + liability_escape_reason fields (task 4)` (verbatim per plan §13 line 1645).
  - Body: include the `HANDOVER:` YAML trailer:
    ```yaml
    HANDOVER:
    filesCreated: []
    filesModified:
      - crates/db_schema/src/source/governance/moderation_case.rs
    keyDecisions:
      - "added 2 fields (grace_expires_at, liability_escape_reason) to BOTH ModerationCase + ModerationCaseInsertForm per plan §10.4; verbatim doc-comments preserved"
      - "field types: Option<DateTime<Utc>> + Option<serde_json::Value>; AsChangeset on InsertForm covers new columns automatically"
      - "<any OBSERVED line-number divergence from plan §10.4 with reason>"
      - "<R3 sweep result: e.g. 'no exhaustive-construction sites found' OR 'fixed N sites under tests/'>"
    notes: <e.g. "post-edit ModerationCase field count = N+2; v0 ..Default::default() callers compile per feedback_insertform_default_propagation">
    ```
- THEN the DQ-entry commit (separate).
- Files in source-code commit: ONLY `crates/db_schema/src/source/governance/moderation_case.rs` (plus any test-site fixes from R3 sweep, if needed).
  Files in DQ commit: ONLY `.claude/decision-queue.json`.

## 5. Validation gate

Plan §15 + Task 4 §15 names these (run in GH Actions on push):

- `cargo check --workspace --features full` (workspace workflow).
- `cargo clippy --workspace --features full --no-deps -- -D warnings`
  (workspace workflow).

**Expected outcome:** workspace-check **STILL FAILS** post-Task-4
because Task 5 hasn't extended the 6 `match` sites yet. The expected
failures are E0004 non-exhaustive patterns on `CaseStatus::SponsorLiability*`
at the same 6 sites that DQ #125 surfaced (planner-intentional cohort
A barrier per DQ #117 Option B). Task 5 lands those `match` extensions.

If workspace-check fails on a different error (typo in field name,
wrong field type, missing import, broken `diesel(check_for_backend)`
type-check), advisor will catch-fire — Task 5 blocks.

If workspace-check unexpectedly passes (Task 5 sites are already
covered for some reason), surface as a DQ pending entry — that's
unexpected and worth verifying before assuming success.

## 6. Expected output (return to advisor)

```
## Task 4 complete — v1-SL-a ModerationCase + InsertForm extended

**Branch:** junior/role-impl-task-...
**Commits:**
  - <sha-a> feat(v1-SL-a): extend ModerationCase + InsertForm with grace_expires_at + liability_escape_reason fields (task 4)
  - <sha-b> chore(decision-queue): impl raised DQ #128 — sl-a-task-4 validate-pending
**File:** crates/db_schema/src/source/governance/moderation_case.rs
**Pre-edit field count:** ModerationCase = N, ModerationCaseInsertForm = M
**Post-edit field count:** ModerationCase = N+2, ModerationCaseInsertForm = M+2
**R3 grep sweep result:** <X exhaustive-construction sites found, all use ..Default::default()> OR <fixed N sites at <file>:<line>>
**Workflow run captured:**
  - workspace-check workflow_run_id: <int> → DQ #128
**Expected workspace-check outcome:** non-exhaustive-match clippy failures still present on Task-5 sites (cohort B barrier; Task 5 closes them)
**Next:** advisor dispatches 1 ci-watcher; cohort B advances to Task 5 sequentially
```

If any pre-write check failed (file path missing, plan §10.4 line
range diverges past adaptation, MIRROR file unreadable), replace
with a DQ catch-fire entry committed + pushed to your worker branch
immediately.
