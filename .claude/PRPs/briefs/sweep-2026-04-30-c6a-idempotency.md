# [role:impl-task] sweep-2026-04-30 C6a — duplicate PublishSanctionNotice idempotency guard (issue #68)

## 1. Dispatch line

`[role:impl-task] sweep-c6a-idempotency — see .claude/PRPs/briefs/sweep-2026-04-30-c6a-idempotency.md`

## 2. Scope

Add an idempotency guard before the federation-send block in `crates/api/api/src/governance/submit_jury_vote.rs` (post-quorum block at approximately line 476) so that a single moderation case emits at most one `PublishSanctionNotice` activity, regardless of how many late-arriving votes (4th, 5th, etc.) re-enter the post-quorum block.

**Current state:**
- An existing guard at commit `41f1d0379` protects the **sanction INSERT** from duplication.
- That guard does NOT extend to the **federation-send** path. Late votes still re-enqueue `PublishSanctionNotice` activities.
- Per the issue, this means a single case can emit multiple federation notices, duplicating governance signals on receiving instances.

**Fix direction (per issue body):**

Add a guard before the federation-send block:

```rust
if Sanction::exists_for_case(conn, case_id).await? {
    return Ok(()); // quorum already processed — skip federation-send
}
```

Or alternatively, gate on `moderation_case.status` already being terminal (whichever guard is more idiomatic in this codebase — read the existing guard at `41f1d0379` to determine the pattern).

**Add a test** in the same crate's tests module that proves: votes 4 and 5 in a 3-of-5 quorum scenario do NOT re-enqueue `PublishSanctionNotice` activities. The test must assert exactly ONE `SentActivity` row of kind `PublishSanctionNotice` exists for the case after all 5 votes are submitted.

**Test placement:**
- If submit_jury_vote.rs has a `#[cfg(test)] mod tests` block, add the test there.
- If there's a sibling integration test under `crates/api/api/tests/governance/` or `crates/server/tests/governance/`, prefer that.
- **DO NOT add the test to `crates/server/tests/e2e.rs`** — that file is 8969 lines and Junior workers hang on full-file Edits per `feedback_junior_worker_e2e_edit_hang.md`.

**Out of scope:**
- Do NOT refactor `process_vote` more broadly. Surgical guard + test only.
- Do NOT touch `Sanction::exists_for_case` or any other helper outside this file (unless minor signature widening is required for the new check).
- Do NOT modify any other governance handler.

**Boundaries:**
- Edit `crates/api/api/src/governance/submit_jury_vote.rs` (the guard).
- Add or extend a test file outside `e2e.rs`.
- Single commit subject: `fix(federation): idempotency guard prevents duplicate PublishSanctionNotice on late votes (closes #68)`.

## 3. Required reading

- **`crates/api/api/src/governance/submit_jury_vote.rs`** — full file at HEAD. Pay attention to:
  - The `process_vote` function and where `vote_count >= QUORUM` is evaluated.
  - The existing guard from commit `41f1d0379` (`git show 41f1d0379 -- crates/api/api/src/governance/submit_jury_vote.rs`).
  - The post-quorum block around line 476.
- **`Sanction::exists_for_case`** — find via `grep -rn "exists_for_case" crates/db_schema/src/`. Confirm it returns a `LemmyResult<bool>` and is callable inside a `run_transaction` scope.
- **`.claude/lessons/feedback_junior_worker_e2e_edit_hang.md`** — critical lesson: do NOT touch e2e.rs.
- **`.claude/lessons/feedback_lemmy_error_no_std_error.md`** if you call any `LemmyError`-returning helper.
- **GitHub issue #68 body**: `gh issue view 68 --repo barrie-cork/lemmy --json body --jq .body`
- **PR #46 CR thread** (link in #68 body): the original Critical finding from CodeRabbit.

## 4. Constraints

**HARD FORBIDS:**
- `cargo *` on the worker. NO `cargo check`, NO `cargo test`, NO clippy. Validation runs OFF-box on GH Actions push trigger.
- Editing `crates/server/tests/e2e.rs`. Use the in-crate `mod tests` or `crates/api/api/tests/`.
- Refactoring beyond the surgical guard. Add 5-10 lines, not 50.
- Modifying `Sanction` struct or its DB schema.

**Required behaviour:**
- Single commit subject: `fix(federation): idempotency guard prevents duplicate PublishSanctionNotice on late votes (closes #68)`.
- Trailer: `Closes: barrie-cork/lemmy#68`.
- Push branch and exit.
- After push, raise a `kind: "validate-pending"` DQ entry naming the workspace-validate workflow run id from `gh run list --repo barrie-cork/lemmy --branch <your-worker-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId,headBranch`.
- DO NOT open a PR — Junior daemon's finalize-merge handles it after validate-pending resolves.

**File-locality:** `crates/api/api/src/governance/submit_jury_vote.rs` is THIS cluster's exclusive territory. Other clusters (C6b, C6c) touch different crates so no overlap. C6a is mutually-exclusive with any future fix to submit_jury_vote.rs in this batch.

**Mid-task DQ push:** if the existing `41f1d0379` guard already covers federation-send (in which case the issue is closed by inspection, not by edit), raise a `kind: "clarify"` DQ entry citing the line range and stop. Do not invent a new guard if one already exists.
