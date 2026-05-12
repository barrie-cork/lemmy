---
phase: v1-RT-r1
role: impl-task
task: fix-3
brief_n: 3
authored: 2026-05-12
parent_dq: 207
---

# [role:impl-task] v1-RT-r1 fix-impl-3 — remove unused import ReputationEventSourceType from reputation_snapshot.rs — see .claude/PRPs/briefs/rt-r1-fix-impl-3.md

## §1 Role + dispatch

`[role:impl-task] v1-RT-r1 fix-impl-3 — remove unused import ReputationEventSourceType from reputation_snapshot.rs`

## §2 Scope

### §2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

> | Failure signature | Auto-fix | Source lesson |
> |---|---|---|
> | `warning: use of deprecated <api>` / unused import | replace with the suggested replacement | n/a (mechanical) |

**Exact error from DQ #207 log_slice:**

```
error: unused import: `ReputationEventSourceType`
  --> crates/api/api/src/governance/reputation_snapshot.rs:59:56
   |
59 | use lemmy_db_schema_file::enums::{ReputationDimension, ReputationEventSourceType};
   |                                                        ^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: `-D unused-imports` implied by `-D warnings`
```

**Root cause:** `ReputationEventSourceType` is used only at `reputation_snapshot.rs:875` inside `#[cfg(test)] mod tests { ... }`. In non-test builds (clippy `--workspace --features full` without `--tests`), the top-level import is unused.

**Fix — 1 line change in 1 file:**

At `crates/api/api/src/governance/reputation_snapshot.rs:59`, change:

```rust
use lemmy_db_schema_file::enums::{ReputationDimension, ReputationEventSourceType};
```

to:

```rust
use lemmy_db_schema_file::enums::ReputationDimension;
```

AND inside `mod tests` at `reputation_snapshot.rs:802`, add a local import:

```rust
use lemmy_db_schema_file::enums::ReputationEventSourceType;
```

**Only 2 line edits in 1 file.** Do not touch any other file.

### §2.2 Pre-flight verification

```bash
grep -n "ReputationEventSourceType" crates/api/api/src/governance/reputation_snapshot.rs
```

Expected: 2 hits — line ~59 (top-level use) and line ~875 (inside mod tests). If more hits exist, STOP and file a DQ blocker.

### §2.3 Validation

After editing, before commit:

```bash
grep -n "ReputationEventSourceType" crates/api/api/src/governance/reputation_snapshot.rs
# Expected: 2 hits — one inside mod tests use, one at the field assignment
```

Push the worker branch. The push triggers `cargo-validate-workspace.yml`.

**Raise a NEW `kind: "validate-pending"` DQ entry** with the fresh `workflow_run_id`.

## §3 Required reading

**Mandatory per file-class table (`.claude/rules/advisor-orchestrator.md` §2.4):**

- `.claude/lessons/feedback_clippy_test_style.md` — clippy `-D warnings` discipline; unused imports in non-test builds are errors.
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — verify no further clippy errors after this fix.

**Context:**
- `.claude/rules/decision-queue.md` — DQ schema, mutation pattern, attribution.
- `.claude/agents/impl-task.md` — subagent contract.

## §4 Constraints

- **Touch only:** `crates/api/api/src/governance/reputation_snapshot.rs` + `.claude/decision-queue.json`.
- **≤2 line edits. No other files.**
- **Commit message:** `fix(v1-RT-r1): move ReputationEventSourceType import into mod tests — unused in non-test build (fix-impl-3)`
- **Raise validate-pending DQ after push** with `workflow_run_id` from `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json databaseId`.
- **Attribution:** `from: "impl"`, never `"advisor"`.
