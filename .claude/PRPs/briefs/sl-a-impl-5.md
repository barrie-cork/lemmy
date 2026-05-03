---
role: impl-task
plan_task: 5
phase: v1-SL-a
cohort: B
created: 2026-05-03
related_dq: []
preallocated_dq: [129]
---

# Brief — v1-SL-a Task 5 — UPDATE 6 ADR-013 enum-exhaustiveness match sites — handle 3 new CaseStatus variants

## 1. Role + dispatch line

`[role:impl-task] v1-SL-a task 5 — see .claude/PRPs/briefs/sl-a-impl-5.md`

You are the **impl-task** subagent (Sonnet 4.6) executing plan Task 5
(cohort B sequential, NOT `[P]`) on `phase-v1-SL-a`. This task closes
the cohort A barrier (per plan §14 Story 1) by extending 6 `match`
sites (and 1 `matches!()` guard) so the workspace-check workflow
**finally passes** for the first time this sub-phase.

## 2. Scope

**Produce:**

1. ONE source-code commit modifying SIX files per plan §10.5
   per-site decision matrix (each match arm extended with explicit
   `CaseStatus::SponsorLiabilityPending | ...Fired | ...Escaped`
   patterns; NEVER `_ => ...`).
2. ONE `kind: "validate-pending"` DQ entry referencing
   `cargo-validate-workspace.yml` (Rust-only; no `migrations/**`
   trigger). **Pre-allocated DQ id: #129.**
3. Push your worker branch. Junior daemon finalize-merges (or advisor
   manually finalizes per the known finalize-merge bug).

**Do NOT** in this task:

- Touch any other file (no `migrations/**`, no `schema.rs`, no other
  `crates/**` file, no `tests/**`, no docs). The 6 files per plan
  §13 lines 1660–1666 are the only editable files.
- Use `_ => ...` arms — workspace clippy denies via
  `feedback_clippy_test_style.md`. Always enumerate explicit
  `| Pattern` arms.
- Modify `submit_jury_vote.rs:729-732` (the `process_appeal_vote`
  terminal-guard) — that stays UNCHANGED per plan §10.5 row 8
  rationale. ONLY edit `submit_jury_vote.rs:271-276` (the
  `process_vote` step-5 guard).
- Run cargo locally. Shape G — workspace-check fires on push.

**Commit message** (verbatim per plan §13 line 1729): `feat(v1-SL-a): ADR-013 enum-exhaustiveness sweep — 6 files handle 3 new CaseStatus variants (task 5)`

## 3. Required reading

Read in this order before writing any code:

1. **Plan Task 5 body:** `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md`
   lines 1647–1729 — task header, FILES YAML (6 files), 6 IMPLEMENT
   sub-edits (file 1 of 6 through file 6 of 6), GOTCHAs (3 of them).
2. **Plan §10.5 (per-site decision matrix):** the canonical source for
   "which arm pattern gets which new variant" per each of the 7 sites
   (6 files; submit_jury_vote.rs has 2 sites, only one edited). Read
   before writing — the matrix has nuance (some sites add to allowed-
   set, some to rejection-set, some to idempotency guard).
3. **Read each file you'll edit FIRST.** All line refs were verified
   by advisor on phase tip 2026-05-03:
   - `crates/api/api_crud/src/governance/request_appeal.rs:94-103`
     (line 94 = `match case.status {`; row 1 in §10.5)
   - `crates/api/api/src/governance/admin_close_case.rs:65-75`
     (line 65 = `match case.status {`; row 2)
   - `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs:72-81`
     (line 72 = `match case.status {`; row 3)
   - `crates/api/api/src/governance/accept_jury_assignment.rs:111-134`
     (line 111 = `JuryAssignmentRole::Original => match case.status {`;
     rows 4 + 5 — BOTH nested matches)
   - `crates/api/api/src/governance/admin_assign_jury.rs:140-147`
     (line 140 = `match case.status {`; row 6)
   - `crates/api/api/src/governance/submit_jury_vote.rs:271-276` ONLY
     (line 271 = `case_row.status,` inside `matches!()`; row 7)
   If actual line numbers diverge, **adapt to the live file** and
   document in commit body. Tasks 2, 3, 4 modified other files; line
   drift here is unlikely.
4. **`.claude/lessons/feedback_clippy_test_style.md`** — workspace
   clippy denies `_ =>` on enum matches where exhaustive-match is
   feasible. Always enumerate explicit `| Pattern` arms.
5. **`.claude/lessons/feedback_advisor_cr_enum_drift.md`** — same
   discipline: cross-check plan §10.5 row text against PRD §3.3
   references for each site; if either disagrees, file a DQ rather
   than guess.

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
    keyDecisions:
      - "split combined SL-a migration into enum-add + non-enum directories per Phase 5b Restoration precedent"
      - "migration count delta: SL-a contributes +2 migrations (was +1); Task 8 will reflect"
    notes: "PRD §8.5 + plan §10.1 superseded; verified pass via DQ #126"
  - task: 2
    commit: bfc8eaa416498d6efd45e64d35e3b80a5466a39e
    filesModified:
      - crates/db_schema_file/src/enums.rs
    keyDecisions:
      - "added 3 CaseStatus variants (SponsorLiabilityPending/Fired/Escaped) per plan §10.2 verbatim"
      - "post-edit count = 12 variants (was 9); #[default] Open unchanged"
    notes: "your match arms reference exactly these 3 variant names"
  - task: 3
    commit: c7a977078e1a2b84ec2a9c91ba4f86c8ac4a7b1c
    filesModified:
      - crates/db_schema_file/src/schema.rs
    keyDecisions:
      - "added 2 columns (grace_expires_at, liability_escape_reason) to moderation_case"
    notes: "schema-side mirror of Task 4 struct extension; not relevant to Task 5"
  - task: 4
    commit: <will-be-filled-by-advisor-on-task-4-completion>
    filesModified:
      - crates/db_schema/src/source/governance/moderation_case.rs
    keyDecisions:
      - "added 2 fields (grace_expires_at, liability_escape_reason) to BOTH ModerationCase + ModerationCaseInsertForm per plan §10.4"
    notes: "your edits do not touch this file; included for context"
```

**Load-bearing context for Task 5:**

- The Rust enum variants you'll add to match arms
  (`CaseStatus::SponsorLiabilityPending`, `...Fired`, `...Escaped`)
  ALREADY EXIST as of Task 2 commit `bfc8eaa41`. Compile-time
  exhaustiveness check is what's currently failing on phase tip;
  your edits close it.
- Phase tip workspace-check has been failing since cohort A push
  (DQ #119, #120, #121, #122 all reported E0004 non-exhaustive
  patterns at exactly the 6 sites you'll edit). DQ #125 (after the
  Task-1 fix) **STILL** reports the same E0004 errors — confirming
  Task 5 is the unblocker.
- After Task 5 lands, the `cargo-validate-workspace` workflow on
  Task 5's push **MUST pass green** for the first time this sub-
  phase. This is plan §14 Story 1's checkpoint. If it doesn't pass,
  the advisor catch-fires and cohort B blocks.

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branch derived from `phase-v1-SL-a`
  (current tip post-Task-4 finalize, which the advisor will
  populate). Use `git fetch origin && git log phase-v1-SL-a` to
  confirm.
- `git branch --show-current` must return your `junior/<task-slug>`
  branch. Confirm `git merge-base --is-ancestor phase-v1-SL-a HEAD`
  exits 0 before any write.
- ONE source-code commit (across all 6 files; that's the plan's
  "one commit per task" rule). Daemon finalize-merges into
  `phase-v1-SL-a`.

### File discipline

- Files you may MODIFY: the 6 files in plan §13 lines 1660–1666
  (verbatim list above in §3 Required reading point 3).
- **Files you may NOT touch:** any `migrations/**`, `schema.rs`,
  any `crates/**` file outside the 6, `tests/**`, `Cargo.toml`,
  `Cargo.lock`, `rust-toolchain.toml`, docs, scripts. If you find
  yourself wanting to touch any of these (especially
  `submit_jury_vote.rs` lines 729-732 — the appeal terminal-guard),
  STOP and re-read plan §10.5 row 8 rationale. That guard stays
  unchanged on purpose.

### Edit discipline (per-site decision matrix — plan §10.5)

For EACH of the 6+1 sites, follow the matrix verbatim:

| Site | File | Pattern style |
|---|---|---|
| Row 1 | request_appeal.rs:94-103 | Add `SponsorLiabilityPending` to allowed-set co-located with `Decided => {}`; add `Fired \| Escaped` to rejection arm |
| Row 2 | admin_close_case.rs:65-75 | Add ALL 3 to allowed-set (admins force-close terminal liability states for ops) |
| Row 3 | admin_trigger_appeal_rejury.rs:72-81 | Add ALL 3 to rejection-set |
| Rows 4+5 | accept_jury_assignment.rs:111-134 | BOTH nested matches (Original + Appeal branches) reject ALL 3 |
| Row 6 | admin_assign_jury.rs:140-147 | Add ALL 3 to rejection-set |
| Row 7 | submit_jury_vote.rs:271-276 | Add `\| SponsorLiabilityPending \| ...Fired \| ...Escaped` to existing `matches!()` early-return list |
| Row 8 | submit_jury_vote.rs:729-732 | **UNCHANGED** — narrow appeal terminal-guard per §10.5 row 8 rationale |

- **Use Edit (not Write).** Surgical extension of each match arm.
- **Verbatim doc-comments per §10.5** (each row prescribes a
  short comment citing PRD §3.3 + ADR-013).
- **Explicit `| Pattern` enumeration** — never `_ => ...`. Workspace
  clippy denies via `feedback_clippy_test_style.md`.

### R3 pattern (post-edit grep sweep)

After your Edit but BEFORE commit, run:

```bash
git grep -nE 'if matches!\([^,]+\.status, CaseStatus::' crates/
git grep -nE 'match \w+\.status' crates/
```

Compare to the 7 sites you edited (or knowingly skipped per row 8).
If any NEW site has appeared on `governance-v0` since brief-write
time (e.g. an upstream rebase or a parallel sub-phase added a new
governance handler), fix it in the SAME commit. Note any new sites
in the commit body. The plan's GOTCHA references this.

### Shape G discipline (no local cargo)

- Push your worker branch. The push triggers
  `cargo-validate-workspace.yml`. **Migration workflow does NOT
  trigger** (no `migrations/**` change in this task).
- Capture the workflow run id:
  ```bash
  gh run list --repo barrie-cork/lemmy --branch <branch> --workflow cargo-validate-workspace --limit 1 --json databaseId
  ```
- Write ONE `kind: "validate-pending"` DQ entry: **id = 129**
  (pre-allocated by advisor; do NOT compute, do NOT use any other
  id), `from: "impl"`, `kind: "validate-pending"`,
  `workflow_run_id: <captured id>`, `branch: "<your branch>"`,
  `phase_task: 5`, `result: null`, `log_slice: null`,
  `failed_jobs: null`, `answer: null`, `answered_by: null`,
  `resolved_at: null`. `context` says "workspace-check for v1-SL-a
  task 5 (ADR-013 enum-exhaustiveness sweep — UNIFIED GREEN-GATE per
  plan §14 Story 1)".
- Commit + push the DQ entry to your worker branch immediately per
  "Mid-task visibility". Two commits acceptable.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.
- Pre-allocated id #129 (Task 4 owns #128). No read-modify-write
  race.

### Commit shape

- ONE source-code commit:
  - Subject: `feat(v1-SL-a): ADR-013 enum-exhaustiveness sweep — 6 files handle 3 new CaseStatus variants (task 5)` (verbatim per plan §13 line 1729).
  - Body: include the `HANDOVER:` YAML trailer:
    ```yaml
    HANDOVER:
    filesCreated: []
    filesModified:
      - crates/api/api_crud/src/governance/request_appeal.rs
      - crates/api/api/src/governance/admin_close_case.rs
      - crates/api/api/src/governance/admin_trigger_appeal_rejury.rs
      - crates/api/api/src/governance/accept_jury_assignment.rs
      - crates/api/api/src/governance/admin_assign_jury.rs
      - crates/api/api/src/governance/submit_jury_vote.rs
    keyDecisions:
      - "extended 6 match sites + 1 matches!() guard per plan §10.5 per-site decision matrix; explicit | Pattern enumeration (no _ => arms); appeal terminal-guard at submit_jury_vote.rs:729 stays unchanged per row 8 rationale"
      - "<R3 grep sweep result: e.g. 'no new match sites since brief-write' OR 'extended N additional sites at <file>:<line> with reason'>"
      - "<any OBSERVED line-number divergence from plan §10.5 with reason>"
    notes: <e.g. "Task 5 closes the cohort A barrier — workspace-check expected GREEN on this push for the first time this sub-phase">
    ```
- THEN the DQ-entry commit (separate).
- Files in source-code commit: ONLY the 6 files listed above (plus any
  R3-sweep additions if found). Files in DQ commit: ONLY
  `.claude/decision-queue.json`.

## 5. Validation gate

Plan §15 + Task 5 §15 names these (run in GH Actions on push):

- `cargo check --workspace --features full` (workspace workflow).
- `cargo clippy --workspace --features full --no-deps -- -D warnings`
  (workspace workflow).

**Expected outcome (UNIFIED GREEN-GATE per plan §14 Story 1):**
workspace-check **MUST pass green**. This is the first time this
sub-phase. The 6 sites you edited close the E0004 non-exhaustive-
match errors that have been blocking workspace-check since cohort A
push.

If workspace-check fails on:
- E0004 non-exhaustive at a SEVENTH site you didn't edit → file a DQ
  pending entry (the planner missed a site; advisor will queue a
  narrow fix-impl-task).
- A different error class (typo, syntax, wrong variant name) →
  advisor catch-fires.
- Clippy warning on `_ => ...` you accidentally introduced → re-edit
  to enumerate explicit `| Pattern` and amend.

## 6. Expected output (return to advisor)

```
## Task 5 complete — v1-SL-a ADR-013 enum-exhaustiveness sweep

**Branch:** junior/role-impl-task-...
**Commits:**
  - <sha-a> feat(v1-SL-a): ADR-013 enum-exhaustiveness sweep — 6 files handle 3 new CaseStatus variants (task 5)
  - <sha-b> chore(decision-queue): impl raised DQ #129 — sl-a-task-5 validate-pending
**Files modified:** 6 files per plan §13 lines 1660–1666
**Sites extended:** 7 (rows 1-7 per plan §10.5); row 8 deliberately skipped per §10.5 rationale
**R3 grep sweep result:** <X new sites found, all extended> OR <no new sites since brief-write>
**Workflow run captured:**
  - workspace-check workflow_run_id: <int> → DQ #129
**Expected workspace-check outcome:** GREEN — first pass this sub-phase per plan §14 Story 1 (cohort A barrier closed)
**Next:** advisor dispatches 1 ci-watcher; on pass, cohort B complete and Tasks 6+7 cohort already shipped via cohort A — only Tasks 8 (e2e PHASE_1_MIGRATION_COUNT bump) + 9 (retro) remain
```

If any pre-write check failed (file path missing, plan §10.5 row text
diverges past adaptation, MIRROR file unreadable), replace with a DQ
catch-fire entry committed + pushed to your worker branch
immediately.
