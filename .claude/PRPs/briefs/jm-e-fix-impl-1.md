---
role: impl-task
plan_task: 1-fix
phase: v1-JM-e
created: 2026-05-01
related_dq: 100
---

# Brief — v1-JM-e fix-impl-1 — Add missing Diesel trait imports in submit_jury_vote.rs

## 1. Role + dispatch line

`[role:impl-task] v1-JM-e fix-impl-1 — see .claude/PRPs/briefs/jm-e-fix-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a narrow §G4-allowlist fix-impl-task: add two missing `use` imports to `crates/api/api/src/governance/submit_jury_vote.rs`. ≤3 file edits total.

**Shape G (Layer G2 push-and-exit):** do NOT run cargo locally. After your commit, push to origin and write a `kind: "validate-pending"` DQ entry referencing the new `cargo-validate-workspace.yml` run id.

## 2. Scope

**Produce** (one commit):

- `crates/api/api/src/governance/submit_jury_vote.rs` — add `JoinOnDsl` and `BoolExpressionMethods` to the existing `use diesel::{...}` import line.

**Do NOT**:
- Touch any other file.
- Run any cargo command locally.
- Touch `crates/api/api_common/src/governance.rs` (already correct from Task 1).
- Write a new DQ entry beyond the `validate-pending` entry Shape G requires.

**Commit message** (exactly): `fix(v1-JM-e): add missing JoinOnDsl + BoolExpressionMethods imports (task 1 fix)`

## 3. Required reading

Read in this order before editing:

1. **`.claude/decision-queue.json`** — locate DQ #100 (`kind: "validate-pending"`, `result: "fail"`, `phase_task: 1`). The `log_slice` there contains the two compiler errors you are fixing.
2. **`crates/api/api/src/governance/submit_jury_vote.rs` lines 41-55** — the existing `use diesel::{...}` block. The fix goes here.

## 3a. Handover from prior cohort

Prior task (Task 1 impl, commit `e9dbc044a`) added `process_appeal_vote` fn to `submit_jury_vote.rs`. The `process_appeal_vote` fn at line ~760 calls `jury_assignment::table.on(...)` and uses `.and(...)` on a Diesel expression. Both require traits not yet in scope:
- `diesel::JoinOnDsl` — required for `.on(...)` on `jury_assignment::table`
- `diesel::BoolExpressionMethods` — required for `.and(...)` on a `Grouped<Eq<...>>`

## 4. The fix (precise)

The current `use diesel::{...}` line in the file (line ~52) reads:

```rust
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, dsl::count_star, insert_into, update};
```

Change it to:

```rust
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper, dsl::count_star, insert_into, update};
```

That is the **only** edit needed. Add `BoolExpressionMethods, ` before `ExpressionMethods` and `JoinOnDsl, ` after `ExpressionMethods` (alphabetical order per codebase convention).

## 4a. Constraints

### Branch + commit discipline
- You start on a Junior worktree branched off `phase-v1-JM-e` (tip `ebb34bb41`).
- One commit only.
- No `answered_by: "advisor"` or `"user"` from this subagent.

### Shape G push-and-exit
After committing, push to `origin/<your-worktree-branch>` and capture the `cargo-validate-workspace.yml` workflow run id:

```bash
git push origin HEAD
# Wait ~10 seconds for GH Actions to trigger, then:
gh run list --repo barrie-cork/lemmy --branch <your-branch> \
  --workflow cargo-validate-workspace.yml --limit 1 \
  --json databaseId,status --jq '.[0]'
```

Write a `kind: "validate-pending"` DQ entry in `.claude/decision-queue.json` (next_id = max of all ids + 1):

```json
{
  "id": <next_id>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<UTC ISO 8601>",
  "question": "workspace-check for v1-JM-e task 1 fix",
  "options": ["pass", "fail"],
  "context": "cargo-validate-workspace.yml triggered on push to <branch>",
  "workflow_run_id": <databaseId>,
  "branch": "<your-worktree-branch>",
  "phase_task": "1-fix",
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

Commit the DQ entry: `chore(decision-queue): impl raised DQ #<id> — validate-pending task 1 fix`
Push that commit too.

## 5. Validation gate (Shape G)

**No local cargo.** Validation is:
1. Push to `origin/<worktree-branch>`.
2. Confirm `cargo-validate-workspace.yml` triggered (via `gh run list`).
3. Write `kind: "validate-pending"` DQ entry with `workflow_run_id`.
4. Commit + push DQ entry.

## 6. Expected output (return to advisor)

```
## fix-impl-1 complete — missing Diesel trait imports

**Commit:** <sha> on <worktree-branch>
**File changed:**
  - crates/api/api/src/governance/submit_jury_vote.rs (added BoolExpressionMethods, JoinOnDsl to use diesel::{...})
**DQ raised:** #<id> kind: validate-pending, workflow_run_id: <id>, branch: <branch>
**Next:** advisor queues ci-watcher for DQ #<id>
```
