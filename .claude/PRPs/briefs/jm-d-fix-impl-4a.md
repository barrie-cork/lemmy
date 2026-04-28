---
role: impl-task
plan_task: 4
phase: v1-JM-d
created: 2026-04-28
status: ready
related_dq: 84
supersedes_premise: jm-d-impl-4.md (allowlist clippy fix-up only — feat is correct, one unused import)
---

# Brief — v1-JM-d Fix 4a — remove unused `Appeal` import (clippy)

## 1. Role + dispatch line

`[role:impl-task] v1-JM-d fix-4a — see .claude/PRPs/briefs/jm-d-fix-impl-4a.md`

## 2. Scope

**Context:** Task #45 (`0d2975388`, feat: admin_trigger_appeal_rejury + seat_appeal_panel) shipped the Task 4 work but Phase 1 workspace check (run `25073660917`, ci-watcher Task #46) failed clippy with one allowlist-class error:

```
error: unused import: `Appeal`
   --> crates/api/api/src/governance/admin_assign_jury.rs:62:5
    |
62  |     appeal::Appeal,
    |     ^^^^^^^^^^^^^^
    = note: `#[deny(unused_imports)]` on by default
```

The `appeal::Appeal` struct (line 62) was added to the import block but never referenced in the file. Verified: zero `Appeal::` (struct method) usages in `admin_assign_jury.rs`. The `schema::appeal` table import (line 80) is **separate** and IS used (lines 1225-1228 in `seat_appeal_panel`); leave it untouched.

**This is a §G4 allowlist auto-queue case** — single file, single line, clippy auto-fix class. No design decisions, no scope change. The feat itself is correct; only the orphan import needs removal.

**Branching strategy (read carefully):**

The Task 4 feat commit `0d2975388` and the daemon's failed auto-merge `d65b43c08` are NOT on `governance-v0` trunk. Trunk is at `765499f25`. To re-validate the feat + fix together, this fix-impl-task must branch off the feat commit, NOT off governance-v0.

```bash
# In the worktree the daemon set up for you (which is from governance-v0):
git fetch origin junior/role-impl-task-v1-jm-d-task-4-see-claude-prps-briefs-jm-d-impl-4-md-45
git checkout 0d2975388 -b junior/fix-task-4a-unused-import
# Now you are on top of the feat. Apply fix below.
```

**Produce** (one fix commit on top of `0d2975388`):

### Fix — `crates/api/api/src/governance/admin_assign_jury.rs`

Delete line 62 (`    appeal::Appeal,`) only. The surrounding `use lemmy_db_schema::{ ... source::governance::{ ... } }` block must remain — leave the other 3 entries (`jury_assignment::JuryAssignmentInsertForm`, `jury_constraint_violation_log::JuryConstraintViolationLogInsertForm`, `moderation_case::ModerationCase`) intact.

After the fix, the block should read:

```rust
use lemmy_db_schema::{
  newtypes::{AppealId, ModerationCaseId},
  source::governance::{
    jury_assignment::JuryAssignmentInsertForm,
    jury_constraint_violation_log::JuryConstraintViolationLogInsertForm,
    moderation_case::ModerationCase,
  },
};
```

That is the **only** edit. Do not touch any other line, file, or import. Do not remove the `schema::appeal` table import at line 80 — it is used.

**Do NOT:**

- Edit any other file.
- Run `cargo` locally (Shape G — workflow validates on GH Actions).
- Cherry-pick or rebase any other commits.
- Push to `phase-v1-JM-d` or `governance-v0` directly.
- Modify the feat commit `0d2975388` (no `--amend`, no rebase) — add a NEW fix commit on top.

## 3. Required reading

- `crates/api/api/src/governance/admin_assign_jury.rs:55-90` — current import block (post-feat)
- DQ entry #84 in `.claude/decision-queue.json` (governance-v0) — the failed validation context
- `feedback_clippy_unused_import_allowlist.md` — if it exists (consult-only)
- `.claude/agents/impl-task.md` — task-0 pre-flight + push discipline

## 4. Constraints

- **One commit.** Subject: `fix(v1-JM-d): remove unused appeal::Appeal import (fix-4a)`
- **Branch from `0d2975388`** (the Task #45 feat commit), NOT from governance-v0. Branch name suggestion: `junior/fix-task-4a-unused-import`. The daemon's default base branch is governance-v0; you must `git checkout 0d2975388 -b <branch>` BEFORE making the edit.
- **Push your branch to origin.**
- **Post-commit:** raise a NEW `kind: "validate-pending"` DQ entry on the `governance-v0` branch. Use `gh run list --repo barrie-cork/lemmy --branch <your-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId` to capture the workflow_run_id of the workspace-check that fires on your push. JSON skeleton:

```json
{
  "id": 85,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<ISO-8601 now>",
  "workflow_run_id": <id from gh run list>,
  "branch": "<your junior/* branch>",
  "phase_task": 4,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

  Append to `pending[]` in `.claude/decision-queue.json` on `governance-v0`. Commit subject: `chore(decision-queue): impl raised DQ #85 — validate-pending fix-4a workspace run <id>`. Push that DQ-update commit to `governance-v0`.

- DQ #84 (the failed entry) stays in `pending[]` for now. The advisor will close it as `superseded_by: 85` when fix-4a's run passes. Do NOT mutate or move DQ #84.
