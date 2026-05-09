---
role: impl-task
plan_task: 1-fix
phase: v1-SL-c-2
created: 2026-05-09
related_dq: 164
---

# Brief — v1-SL-c-2 fix-impl-1 — change test fn return type to `LemmyResult<()>` (compile fix for E0277)

## 1. Role + dispatch line

`[role:impl-task] sl-c-2-fix-impl-1 — see .claude/PRPs/briefs/sl-c-2-fix-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6). Apply a minimal,
mechanical compile fix. ONE Edit. ≤3 line change.

## 2. Scope

**The bug:** Workflow `25582548670` (cargo-validate-workspace on the worker
branch carrying sl-c-2 Task 1) failed at `cargo test compile (no-run)` with:

```
error[E0277]: `?` couldn't convert the error: `LemmyError: std::error::Error` is not satisfied
  --> crates/server/tests/e2e.rs:12049:79
   |
12049 |     let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
   |                                         --------------------------------------^
   |                                         |
   |                                         this has type `Result<_, LemmyError>`
   |
   = note: required for `Box<dyn std::error::Error>` to implement `std::convert::From<LemmyError>`
```

**Root cause:** The test fn `grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries`
declares return type `Result<(), Box<dyn Error>>`, but the body uses `?`
to propagate `LemmyError` from `governance_fixtures::bootstrap()` and other
calls (`Instance::read_or_create`, `seed_user`, `run_grace_check_batch`).
`LemmyError` does NOT implement `std::error::Error`, so `Box<dyn Error>: From<LemmyError>`
is missing and `?` can't convert.

**The fix:** Change the test fn return type from `Result<(), Box<dyn Error>>`
to `lemmy_utils::error::LemmyResult<()>`. This matches the canonical
sibling pattern (e.g. `crates/server/tests/e2e.rs:8009-8011`
`it_admin_assigns_jury_then_decides` test, line 8010 returns
`-> lemmy_utils::error::LemmyResult<()>`). Helpers `seed_pending_case`
and `seed_active_surety` already use `Result<_, Box<dyn Error>>` and they
compile fine (diesel errors implement `std::error::Error` natively); leave
helpers UNCHANGED.

**File to edit:** `crates/server/tests/e2e.rs`

**Specific lines to change** (post-impl-1 worker branch tip
`5baa9a54f`; verify with `grep -n "grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries" crates/server/tests/e2e.rs` at task start):

Line 12042-12044 currently reads:
```rust
  #[tokio::test]
  async fn grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries(
  ) -> Result<(), Box<dyn Error>> {
```

Change to:
```rust
  #[tokio::test]
  async fn grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries(
  ) -> lemmy_utils::error::LemmyResult<()> {
```

That's the ONLY change. Single Edit. No other files. No other lines. Do NOT
modify the helpers. Do NOT touch the use block (the `lemmy_utils` path
is fully-qualified inline, no new `use` import needed).

After the Edit:
1. Commit on worker branch with subject:
   `fix(v1-SL-c-2): test fn return type LemmyResult<()> for E0277 (fix-impl-1)`
2. Push the worker branch.
3. Write a NEW `kind: "validate-pending"` DQ entry per Shape G — capture the
   new workflow_run_id triggered by your push (`gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --workflow cargo-validate-workspace --json databaseId`).

DO NOT modify DQ #164 — that entry stays in `pending[]` as the failing
record. Your new entry is a fresh `kind: "validate-pending"` for the
post-fix workflow run.

## 3. Required reading

In order:

1. **Plan §13 Task 1** lines 1316-1508 — original test scope.
2. **The failing line at e2e.rs:12042-12049** (post-impl-1 worker branch
   `5baa9a54f`) — read 20 lines above + below.
3. **Sibling pattern at e2e.rs:8009-8011** — confirm the `LemmyResult<()>`
   signature is the working pattern.
4. **Lessons:**
   - `feedback_lemmy_error_no_std_error.md` — the underlying lesson that
     was missed in plan §13 Task 1 (planner-side miss; logged for retro).
   - `feedback_features_full_p_crate_incompatible.md` — bound on validate-pending DQ.
   - `feedback_pipes_mask_exit_codes.md` — bound on log capture.

## 4. Constraints

- ONE Edit on `crates/server/tests/e2e.rs`. Single 1-line replacement
  (the return-type clause on line 12044, with the surrounding 2-line
  context for unique anchor).
- No use-block changes (the type path is fully-qualified inline).
- No helper changes (helpers compile fine; don't touch them).
- No test-body changes.
- No other files.
- Commit on the SAME worker branch as impl-task #154
  (`junior/role-impl-task-sl-c-2-impl-1-see-claude-prps-briefs-sl-c-2-impl-1-md-154`),
  fast-forward append. Do NOT cut a new junior worker branch — this fix
  rides the existing worker branch so daemon finalize-merge will pick up
  both impl-1 and fix-impl-1 commits as one unit.

### Validate-pending DQ shape

```json
{
  "id": <max(all ids)+1>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<ISO 8601 UTC>",
  "question": "Does cargo-validate-workspace pass on sl-c-2 fix-impl-1 (test fn return type LemmyResult<()>)?",
  "options": [
    "(A) pass — advance to Task 2",
    "(B) fail — advisor §G4 triage"
  ],
  "context": "Pushed fix(v1-SL-c-2): test fn return type LemmyResult<()> for E0277 (fix-impl-1) to <branch>. Closes E0277 from DQ #164.",
  "branch": "<your worker branch — same as impl-1>",
  "phase_task": "sl-c-2-fix-impl-1",
  "workflow_run_id": <captured>,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null
}
```

Commit + push the DQ entry IMMEDIATELY after writing.

### Hard refusals

- Do NOT touch the helpers (`seed_pending_case`, `seed_active_surety`).
  They compile.
- Do NOT touch any other test fn or mod.
- Do NOT add `use lemmy_utils::error::LemmyResult` to the use block;
  use the fully-qualified path on the return type.
- Do NOT mutate DQ #164 (it stays as the failing record; the new entry
  describes the post-fix workflow).
- Do NOT run cargo locally.
- Do NOT modify `.claude/**`.

## 5. Acceptance

- Single commit on worker branch with subject
  `fix(v1-SL-c-2): test fn return type LemmyResult<()> for E0277 (fix-impl-1)`.
- Diff: ONLY the return type on the test fn — `Result<(), Box<dyn Error>>`
  → `lemmy_utils::error::LemmyResult<()>`. ~1 line changed.
- Worker branch pushed.
- One NEW `kind: "validate-pending"` DQ entry written + committed + pushed.
