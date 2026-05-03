---
role: impl-task
plan_task: 2
phase: v1-SL-a
cohort: A
created: 2026-05-03
related_dq: []
---

# Brief — v1-SL-a Task 2 [P] — UPDATE `crates/db_schema_file/src/enums.rs` — extend `CaseStatus` with three new variants

## 1. Role + dispatch line

`[role:impl-task] v1-SL-a task 2 — see .claude/PRPs/briefs/sl-a-impl-2.md`

You are the **impl-task** subagent (Sonnet 4.6) executing plan Task 2
of cohort A on `phase-v1-SL-a`. Cohort A = Tasks 1+2+3+6+7 (5-way
parallel per plan §13 line 1240); you own only
`crates/db_schema_file/src/enums.rs`.

## 2. Scope

**Produce:**

1. ONE commit modifying `crates/db_schema_file/src/enums.rs` —
   inserting three new `CaseStatus` variants (verbatim per plan
   §10.2) between `AdminReview,` and the closing `}`.
2. ONE `kind: "validate-pending"` DQ entry referencing
   `cargo-validate-workspace.yml` (this task is Rust-only; no
   migration trigger).
3. Push your worker branch. Junior daemon finalize-merges into
   `phase-v1-SL-a`.

**Do NOT** in this task:

- Touch any other file (no `migrations/**`, no `schema.rs`, no other
  `crates/**` file, no `tests/**`, no docs). `enums.rs` is the only
  editable file.
- Add `#[default]` on the new variants. The existing `Open` default
  at line 394 stays correct — new cases never start in a sponsor-
  liability state.
- Reorder existing variants. Insert AFTER `AdminReview,` and BEFORE
  the closing `}` only.
- Modify the existing `DbValueStyle = "verbatim"` directive.
- Run cargo locally. Shape G — workspace-check fires on push.

## 3. Required reading

Read in this order before writing any code:

1. **Plan Task 2 body:** `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md`
   lines 1505–1543 — task header, FILES YAML, IMPLEMENT directives,
   GOTCHAs (3 of them).
2. **Plan §10.2 (canonical Rust skeleton):**
   `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` lines 712–749.
   The verbatim source for the three new variants + their `///`
   doc-comments.
3. **Read the file you'll edit FIRST:**
   `crates/db_schema_file/src/enums.rs` lines 382–408 (the existing
   `CaseStatus` declaration). Confirm:
   - Line 389 declares `DbValueStyle = "verbatim"`.
   - Line 394 has `#[default] Open,`.
   - Line 407 (or thereabouts) has `AdminReview,`.
   - Line 408 (or thereabouts) is the closing `}`.
   If actual line numbers diverge from plan §10.2, **adapt to the
   live file** and document in the commit body. The plan was written
   against the brief-write-time HEAD; cohort B (tasks 4+) hasn't
   run yet, so divergence is unlikely.
4. **`.claude/lessons/feedback_advisor_cr_enum_drift.md`** — load-
   bearing for this task: "If the advisor or CR names a specific
   enum value string, the impl's default assumption should be: not
   in the PRD until proven otherwise." For SL-a the variants
   (`SponsorLiabilityPending`, `SponsorLiabilityFired`,
   `SponsorLiabilityEscaped`) ARE plan §10.2 verbatim AND PRD §3.1.
   Cross-check both before writing — if either disagrees with the
   other, file a DQ rather than guess.
5. **PRD §3.1 + OQ-025 origin** as cited in the doc-comments per
   plan §10.2. The doc-comments themselves are part of the
   verbatim copy.

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
      Probe 2 CaseStatus script count=14 (script precision issue: sed range
      includes CaseStatusTier); actual CaseStatus has 9 variants confirmed
      by file read. Probe 9 yamllint exit=127 (not installed); YAML
      validated via python yaml — all 3 files OK; flags --features full,
      --no-deps, -D warnings confirmed present.
```

Task 0's note that **CaseStatus has 9 variants** is your
pre-write verification baseline: read `enums.rs` and confirm
9 existing variants before adding the 3 new ones (post-edit
count = 12).

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branch derived from `phase-v1-SL-a`
  (tip `1b47a3ff4`).
- `git branch --show-current` must return your `junior/<task-slug>`
  branch. Confirm `git merge-base --is-ancestor phase-v1-SL-a HEAD`
  exits 0 before any write.
- ONE source-code commit. Daemon finalize absorbs into `phase-v1-SL-a`.

### File discipline

- Files you may MODIFY: `crates/db_schema_file/src/enums.rs` ONLY.
- **Files you may NOT touch:** any `migrations/**`, `schema.rs`, any
  other `crates/**` file, `tests/**`, `Cargo.toml`, `Cargo.lock`,
  `rust-toolchain.toml`, docs, scripts. If you find yourself
  wanting to touch any of these, STOP and file a DQ.

### Edit discipline

- **Use Edit (not Write).** Surgical insertion of three variant lines
  + their doc-comments after `AdminReview,` and before the closing
  `}`. The rest of the file is untouched.
- **Verbatim from plan §10.2** — do not paraphrase doc-comments,
  re-order the three new variants, change PascalCase, or "improve"
  comment wording.
- **PascalCase variant names** (`SponsorLiabilityPending`,
  `SponsorLiabilityFired`, `SponsorLiabilityEscaped`) match the
  Postgres-side string values written by Task 1's migration
  (`'SponsorLiabilityPending'`, etc) — `DbValueStyle = "verbatim"`
  guarantees the mapping. Do NOT shorten or rename.
- **No `#[default]` on the new variants.** `#[default]` stays on
  `Open` only. Cases enter a sponsor-liability state mid-flight, not
  at creation.
- **Doc-comments** (the `///` lines) cite PRD §3.1 + OQ-025 per plan
  §10.2 — copy verbatim.

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
  `branch: "<your branch>"`, `phase_task: 2`, `result: null`,
  `log_slice: null`, `failed_jobs: null`, `answer: null`,
  `answered_by: null`, `resolved_at: null`. `context` says
  "workspace-check for v1-SL-a task 2".
- Commit + push the DQ entry to your worker branch immediately per
  "Mid-task visibility". Two commits acceptable: (1) `feat(v1-SL-a):
  add 3 CaseStatus variants ...`, (2) `chore(decision-queue): impl
  raised DQ #N — sl-a-task-2 validate-pending`.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.
- Compute next id by scanning **both** live `pending`/`resolved` AND
  any `decision-queue-archive-*.json`. **Cohort A coordination:**
  Tasks 1 + 2 + 3 are running in parallel; the live file may have
  been updated by Task 1's DQ writes between your `git pull` and
  your DQ write. Re-read the file immediately before computing
  next id (read-modify-write under cohort parallelism is the
  documented risk; per `.claude/rules/decision-queue.md`
  "Concurrency", read-before-write is the discipline).

### Commit shape

- ONE source-code commit:
  - Subject: `feat(v1-SL-a): add 3 CaseStatus variants — SponsorLiabilityPending/Fired/Escaped (task 2)` (verbatim per plan §13 line 1543).
  - Body: include the `HANDOVER:` YAML trailer:
    ```yaml
    HANDOVER:
    filesCreated: []
    filesModified:
      - crates/db_schema_file/src/enums.rs
    keyDecisions:
      - "added 3 CaseStatus variants (SponsorLiabilityPending/Fired/Escaped) per plan §10.2; verbatim doc-comments preserved"
      - <any OBSERVED line-number divergence from plan §10.2 with reason>
    notes: <e.g. "post-edit CaseStatus variant count = 12 (was 9); #[default] Open unchanged">
    ```
- THEN the DQ-entry commit (separate).
- Files in source-code commit: ONLY `crates/db_schema_file/src/enums.rs`.
  Files in DQ commit: ONLY `.claude/decision-queue.json`.

## 5. Validation gate

Plan §15 names these (run in GH Actions on push):

- `cargo check --workspace --features full` (workspace workflow).
- `cargo clippy --workspace --features full --no-deps -- -D warnings`
  (workspace workflow). The new variants must compile under deny-
  warnings without exhaustive-match warnings on the 6 `match` sites
  Task 5 will edit — **but Task 5 hasn't run yet, so a non-
  exhaustive-match clippy/compile failure here is EXPECTED and
  ci-watcher will surface it.** Advisor will not auto-queue a fix
  for non-exhaustive-match because Tasks 4–5 are scheduled to land
  before any cohort B validate-pass.

If the workspace-check fails on non-exhaustive-match in
`crates/api/api/src/governance/...` or `crates/api_crud/...`, that
is the EXPECTED failure mode and the cohort A barrier holds until
Task 5 lands the `match` extensions. Advisor §G4 classifier
recognises this. **Do NOT attempt local fix-up.**

If workspace-check fails on a different error (typo, missing
import, syntax), advisor will catch-fire — cohort B blocks.

## 6. Expected output (return to advisor)

```
## Task 2 complete — v1-SL-a CaseStatus extended

**Branch:** junior/role-impl-task-...
**Commits:**
  - <sha-a> feat(v1-SL-a): add 3 CaseStatus variants ... (task 2)
  - <sha-b> chore(decision-queue): impl raised DQ #<N> — sl-a-task-2 validate-pending
**File:** crates/db_schema_file/src/enums.rs
**Pre-edit CaseStatus variant count:** 9 (Open default, ... AdminReview)
**Post-edit CaseStatus variant count:** 12 (added: SponsorLiabilityPending, SponsorLiabilityFired, SponsorLiabilityEscaped)
**Doc-comments verbatim:** PRD §3.1 + OQ-025 cites preserved
**`#[default] Open` unchanged:** confirmed
**Workflow run captured:**
  - workspace-check workflow_run_id: <int> → DQ #<N>
**Expected workspace-check outcome:** non-exhaustive-match clippy failures on Task-5 sites (6 sites per plan §10.5) — this is the COHORT BARRIER, not a Task-2 defect
**Next:** advisor dispatches 1 ci-watcher; cohort A (Tasks 1+2+3+6+7) advances per plan §14 Story 1 (workspace-check success on Task 5's push, post cohort B Tasks 4+5)
```

If any pre-write check failed (file path missing, plan §10.2 line
range diverges past adaptation, MIRROR file unreadable), replace
with a DQ catch-fire entry committed + pushed to your worker branch
immediately.
