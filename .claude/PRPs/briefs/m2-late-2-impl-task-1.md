---
phase: m2-late-2
role: impl-task
n: 1
authored: 2026-06-12
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-late-2
task_number: 1
parallel_group: cohort-1
---

# [role:impl-task] m2-late-2 Task 1 — case_id payload field + e2e assertion

## 1. Role + dispatch line

```
[role:impl-task] m2-late-2 task-1 case_id payload — see .claude/PRPs/briefs/m2-late-2-impl-task-1.md
```

## 2. Scope

Add `case_id: i64` to `SanctionEventPayload` in `sanction_publisher.rs`, populate it from the
sanction row, then assert the field is present and correct in the e2e test.

### IMPLEMENT

| File | Change |
|---|---|
| `crates/api/api/src/governance/sanction_publisher.rs` | (A) Add `pub case_id: i64` field to `SanctionEventPayload` struct. (B) Populate via `case_id: i64::from(sanction.case_id.0)` at the `SanctionEventPayload { .. }` build site. |
| `crates/server/tests/e2e/m2_late.rs` | (C) After the `governance_log_entry_hash must be non-empty` assertion, add a `case_id` assertion block (see §3 Pattern and §4 Anchor). |

### Explicit out-of-scope

- Do NOT touch `services/bridge/` — bridge mirror is Task 3.
- Do NOT wrap `enqueue_sanction_event` in a transaction — that is Task 2.
- Do NOT add any new structs, traits, or crates.
- No commit before all three sub-changes compile and the e2e test passes.

## 3. Implementation pattern (verbatim from plan §10)

### Pattern (A+B) — struct field + build site

In `sanction_publisher.rs`, locate `SanctionEventPayload` struct definition (currently has
`sanction_kind: String` field; add immediately after it):

```rust
pub case_id: i64,
```

At the struct build site (the `SanctionEventPayload { .. }` expression, currently around
lines 130–136 where `sanction_kind: format!("{:?}", sanction.sanction_kind)` is set), add:

```rust
case_id: i64::from(sanction.case_id.0),
```

**GOTCHA R1:** use `i64::from(sanction.case_id.0)`, NOT `sanction.case_id.0 as i64`.
`ModerationCaseId` wraps `i32`; `from` is lossless and expresses intent. The `.0` field
accessor gives the inner `i32`; `i64::from` widens safely.

### Pattern (C) — e2e assertion

After the `governance_log_entry_hash must be non-empty` assertion block, insert:

```rust
    let payload_case_id = payload.get("case_id").and_then(|v| v.as_i64())
      .expect("payload missing case_id");
    assert_eq!(payload_case_id, i64::from(case_id.0), "payload case_id matches the case");
```

`case_id` is the `ModerationCaseId` captured at the `let case_id = { ... }` block (~line 262
in the test). The `.0` accessor gives the inner value; `i64::from` widens it to match the
payload's JSON number.

### Anchor uniqueness (mandatory pre-Edit check)

The test file is large. Before running any Edit, confirm your `old_string` anchor is unique:

```bash
grep -c 'governance_log_entry_hash must be non-empty' crates/server/tests/e2e/m2_late.rs
```

Expected: `1`. If > 1, revise the anchor to include the surrounding two lines. DO NOT
dispatch an Edit with a non-unique anchor (causes hang/wrong-site edit). Verified at brief
authorship: count=1 for `"governance_log_entry_hash must be non-empty"`.

## 4. Validation gate (validate-pending-laptop DQ — write then STOP)

After committing the change, write a `kind: "validate-pending-laptop"` DQ entry with:

```json
{
  "commands": [
    "cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --test e2e -p lemmy_server --features full sanction_event_delivered_to_subscriber\""
  ],
  "branch": "<current-worktree-branch>",
  "phase_task": 1,
  "e2e_filter": null
}
```

Commit the DQ entry + push the branch, then **STOP**. Do NOT run `cargo-check.bat`,
`cargo-clippy.bat`, or `cargo-test.bat` yourself — those run on the laptop advisor session
only (NO-CARGO-ON-ELITEDESK).

Generate the DQ id via `bash scripts/brehon/dq-v3-new-entry.sh`.
Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`.

## 5. Required reading

- `.claude/PRPs/plans/m2-late-2.plan.md` — §13 Task 1 spec (field name, build site, Pattern
  10.1 context), §10 Patterns catalogue, GOTCHA R1, §15 DoD commands
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — LemmyResult return pattern; any
  new `?` in the modified path uses Case A (`.map_err(LemmyError::from)`)
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish`
  pattern used in the e2e test; do not alter the existing pool/conn setup
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate verbatim
  `old_string` anchors before any Edit to `m2_late.rs`; anchor uniqueness gate is mandatory

## 6. Constraints

- **NO cargo on EliteDesk (NO-CARGO-ON-ELITEDESK)** — write the validate-pending-laptop DQ
  and STOP. Do NOT invoke any cargo/bat wrapper.
- **No extra files** — only touch the two files in §2 IMPLEMENT.
- **No transaction wrapping in this task** — T2 owns the `run_transaction` refactor.
- **DQ v3 id** — generate via `bash scripts/brehon/dq-v3-new-entry.sh`.
- **Mid-task push discipline** — commit DQ entry + push immediately after writing it, before
  stopping, so the laptop advisor can see the entry on its next poll.
- **Commit prefix** — `feat(governance): ` subject, with `LESSON:` trailer if any non-obvious
  finding arises.

### Mandatory lesson fires (file-class table check)

| File class | Match? | Lesson injected |
|---|---|---|
| `crates/server/tests/e2e.rs` (any edit) | ✅ `m2_late.rs` | `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md` |
| `crates/server/tests/e2e.rs` (≥2 edits in this task) | ✅ 1 edit here | `feedback_fix_impl_pre_locate_e2e_anchors.md` (per ≥2-edits-in-cohort rule; T1+T3 share the cohort but different files — single file edit is still subject to anchor pre-locate) |
| Any handler doing 2+ DB writes | ❌ T1 adds no writes | — |
