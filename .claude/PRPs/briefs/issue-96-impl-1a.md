# Brief: issue-96-impl-1a — Rust correctness cluster (cr-5, cr-8)

## 1. Role + dispatch

[role:impl-task] issue-96-cluster1 — fix GitHub issue #96 cr-5 (config const fallback) + cr-8 (ConstraintRecord on decline replacement)

## 2. Scope

Fix two carry-forward Major findings from PR #95. Both are independent code paths.

**cr-8** — `crates/api/api/src/governance/decline_jury_assignment.rs:164`
The replacement juror insert hardcodes `selected_under_constraints: None`. The `_record` binding on line 164 discards the `ConstraintRecord` returned by `select_eligible_jurors`. Fix: capture the record and persist it.

Change:
```rust
let (replacements, _record) = select_eligible_jurors(...)
```
To:
```rust
let (replacements, record) = select_eligible_jurors(...)
```
And on the `JuryAssignmentInsertForm` (around line 177–186), change:
```rust
selected_under_constraints: None,
```
To:
```rust
selected_under_constraints: Some(record.to_json()),
```

MIRROR ref: `crates/api/api/src/governance/admin_emergency_remove.rs:224-269` — the exact same pattern (capture `record`, call `record.to_json()`, set `selected_under_constraints: Some(...)`).

**cr-5** — `crates/api/api/src/governance/config.rs:434`
`get_int_cascade` falls back to `const_default_int(namespace)` after exhausting DB rows. It should walk the full `candidates` list first, trying `const_default_int(candidate)` for each, returning the first `Some`. The `namespace` bare-string fallback is the last resort only. Apply the identical fix to `get_float_cascade` (line 486).

Change the final fallback block in `get_int_cascade` (line 434–439) from:
```rust
const_default_int(namespace).ok_or_else(|| { ... })
```
To:
```rust
candidates
  .iter()
  .find_map(|c| const_default_int(c))
  .ok_or_else(|| { ... })
```
Apply the same change to `get_float_cascade` (line 486–491), using `const_default_float`.

**Files authorised to edit:**
- `crates/api/api/src/governance/decline_jury_assignment.rs`
- `crates/api/api/src/governance/config.rs`

Do NOT edit any other file. Do NOT open PRs. Do NOT push to governance-v0. Push your worktree branch only.

**Branch:** `fix/issue-96-cluster1` (Junior creates this automatically in your worktree)

## 3. Required reading

- `.claude/lessons/feedback_check_git_before_junior_queue.md`
- `.claude/lessons/feedback_junior_finalize_skips_when_worker_pre_pushes.md`
- GitHub issue body: `gh issue view 96 --repo barrie-cork/lemmy` (read for full finding context)
- MIRROR ref: read `crates/api/api/src/governance/admin_emergency_remove.rs` lines 224–269 before editing `decline_jury_assignment.rs`

## 4. Constraints

- **Commit shape:** two commits, one per finding:
  1. `fix(jury): persist ConstraintRecord on decline_jury_assignment replacement (cr-8 of #96)`
  2. `fix(config): walk candidate-level const fallbacks before namespace-only (cr-5 of #96)`
- **Shape G validation:** after committing both fixes and pushing your worktree branch to origin, write a `kind: "validate-pending"` entry to `.claude/decision-queue.json` containing the `workflow_run_id` of the triggered `cargo-validate-workspace.yml` run (check via `gh run list --repo barrie-cork/lemmy --branch <your-worktree-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId`). Commit and push the DQ entry immediately.
- Do not author plan files, rule files, ADR changes, or lesson files.
- Do not amend commits after pushing.
- Do not run `cargo test --workspace` — workspace check only (`cargo check --workspace --features full` if running locally, but Shape G handles this on GitHub runners).
