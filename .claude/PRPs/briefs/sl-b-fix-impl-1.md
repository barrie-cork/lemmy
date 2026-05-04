---
role: impl-task
plan_task: 2-fix
phase: v1-SL-b
created: 2026-05-04
related_dq: 145
---

# Brief — v1-SL-b fix-impl-1 — clippy::indexing-slicing fix on revoke_endorsement.rs:302

## 1. Role + dispatch line

`[role:impl-task] sl-b-fix-impl-1 — see .claude/PRPs/briefs/sl-b-fix-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a **§G4-allowlist
narrow fix-in-pr** task per advisor-orchestrator.md "§G4 classifier" — clippy
auto-fix on a single line. Scope is **strictly** the one clippy violation
identified in DQ #145; no other changes permitted.

## 2. Scope

**Produce:**

1. **One impl commit** with a single-line edit on
   `crates/api/api_crud/src/governance/revoke_endorsement.rs` line 302.
2. **Push the worker branch** to `origin/junior/<task-slug>`.
3. **Write a `kind: "validate-pending"` DQ entry** capturing the workflow
   run id of `cargo-validate-workspace.yml` per Shape G Layer G2. This is
   a SECOND commit on the same worker branch.

**Do NOT** in this task:

- Touch any file other than `revoke_endorsement.rs` and
  `.claude/decision-queue.json`.
- Refactor any other code paths or "improve" surrounding logic.
- Touch the docstring, the use block, or any other line.
- Add new tests.
- Run cargo locally — Shape G; cargo runs on GH-hosted runners.
- Modify `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.
- Resolve DQ #145 — its mutation already populated `result: fail`; this
  fix-impl raises a NEW `validate-pending` DQ entry referencing a NEW
  workflow run on the new worker branch.

## 3. Required reading

Read in this order before writing the Edit:

1. **DQ #145** in `.claude/decision-queue.json` `pending[]` — read the
   `log_slice` field for the verbatim clippy error (line + context +
   help text).
2. **`crates/api/api_crud/src/governance/revoke_endorsement.rs`** lines
   285-310 — get the surrounding context. The target site is line 302
   inside the post-tx payload assembly + `endorsement_revoked` log
   entry block.
3. **`.claude/lessons/feedback_clippy_rerun_after_fix.md`** — re-run
   clippy locally is impractical under Shape G; the workspace-check
   workflow is the only gate. Be careful that your edit doesn't unmask
   another lint.
4. **`.claude/lessons/feedback_clippy_test_style.md`** — clippy lint
   discipline; this fix uses the standard `as_object_mut().insert(...)`
   pattern for `serde_json::Value` mutation.
5. **`.claude/rules/decision-queue.md`** — schema-v2 for the new
   `validate-pending` entry.

## 3a. Handover from prior cohort

**From Task 2 (sl-b-impl-2, task #118):** SHIPPED but failed
workspace-check.
- `crates/api/api_crud/src/governance/revoke_endorsement.rs` exists at
  317 lines, mostly correct.
- mod.rs wire shipped.
- Failure: clippy::indexing-slicing at line 302 —
  `payload["rate_limit_bypassed"] = json!(true);`. `serde_json::Value`
  string-indexing trips clippy because it panics on missing keys / non-
  object payloads.
- DQ #145 mutated by advisor with full log slice. Stays in `pending[]`.

**Phase tip:** `d96ded588 chore(advisor): mutate DQ #145 — ci-watcher fail
recovery (sl-b-impl-2 clippy)`.

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branched off `phase-v1-SL-b` (tip
  `d96ded588`).
- `git branch --show-current` should return a `junior/role-impl-task-...`
  branch (adapt to your actual worktree branch name).
- Two commits at task end:
  1. Impl: `fix(v1-SL-b): clippy::indexing-slicing on revoke_endorsement.rs:302`
  2. DQ: `chore(decision-queue): impl raised DQ #<next-id> — sl-b-fix-impl-1 validate-pending`

### Implementation discipline — the EXACT edit

**Anchor on lines 301-303** in the worktree's
`crates/api/api_crud/src/governance/revoke_endorsement.rs`. Current text
(verify by reading the file):

```rust
  if bypass_recorded {
    payload["rate_limit_bypassed"] = json!(true);
  }
```

**Replace with:**

```rust
  if bypass_recorded {
    if let Some(obj) = payload.as_object_mut() {
      obj.insert("rate_limit_bypassed".to_string(), json!(true));
    }
  }
```

**Why this fix:**
- `serde_json::Value::operator[](&str)` on a non-Object Value panics →
  triggers `clippy::indexing-slicing`.
- `as_object_mut()` returns `Option<&mut Map<String, Value>>`. The
  `if let Some(obj)` branch is a no-op when `payload` isn't an Object,
  but the prior `payload = json!({...})` at line 293 always constructs
  an Object — so the pattern always matches in practice. Safe.
- Standard Rust idiom. No `unwrap()`, no panic risk. Clippy-clean.

**Alternative rejected:** `payload.as_object_mut().unwrap().insert(...)`
— uses `unwrap()` which is itself a clippy::unwrap_used candidate
(though not currently lint-fired, the workspace-check rules may evolve).
The `if let Some(...)` form is strictly safer.

**Do NOT change anything else.** Not the doc-comments, not the use
block, not the surrounding `payload = json!({...})` literal, not the
subsequent `governance_log::append(...)` call.

### Validate-pending DQ entry shape (Shape G Layer G2)

```json
{
  "id": <next-int>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<NOW_ISO>",
  "subject": "sl-b-fix-impl-1 workspace-check validate-pending",
  "branch": "junior/<task-slug>",
  "phase_task": "sl-b-fix-impl-1",
  "workflow_run_id": <int from gh run list>,
  "commands": ["cargo check --workspace --features full", "cargo clippy --workspace --features full --no-deps -- -D warnings", "cargo test --no-run -p lemmy_server --test e2e"],
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

After pushing the impl commit, capture the `workflow_run_id`:

```bash
gh run list --repo barrie-cork/lemmy --branch <your-worker-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId --jq '.[0].databaseId'
```

If `gh run list` returns empty, wait up to 60 seconds and retry.

### Hard refusals

- Do NOT modify any file other than the two named (handler + DQ).
- Do NOT touch anything other than line 302 (and its 2 surrounding
  brace-only lines that `if let Some(...)` introduces).
- Do NOT add `unwrap()` or `expect()`.
- Do NOT comment-out the lint via `#[allow(clippy::indexing_slicing)]`
  — fix the root cause.
- Do NOT mutate or resolve DQ #145. Raise a new entry.
- Do NOT push the worker branch BEFORE both commits land.

## 5. Acceptance

Fix-impl-1 passes if:

- `revoke_endorsement.rs:302` no longer contains
  `payload["rate_limit_bypassed"]`. Instead contains
  `payload.as_object_mut()` + `obj.insert("rate_limit_bypassed"...)`.
- `git diff HEAD~1 -- crates/api/api_crud/src/governance/revoke_endorsement.rs`
  shows ONLY 3-line replacement around line 302 (5 lines if counting
  the conditional brace pair). No other diff lines in `.rs` files.
- One impl commit + one DQ-entry commit on the worker branch.
- DQ entry written with `kind: "validate-pending"`, `from: "impl"`,
  populated `workflow_run_id`.
- ci-watcher's later poll returns `conclusion: "success"`.
