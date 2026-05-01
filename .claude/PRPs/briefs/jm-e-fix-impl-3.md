---
role: impl-task
plan_task: 1-fix3
phase: v1-JM-e
created: 2026-05-01
related_dq: 103
---

# Brief — v1-JM-e fix-impl-3 — Fix config_get_int_cascade test assertion (most-specific const wins)

## 1. Role + dispatch line

`[role:impl-task] v1-JM-e fix-impl-3 — see .claude/PRPs/briefs/jm-e-fix-impl-3.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a narrow fix-impl-task: update one assertion + one assertion message + one docstring line in `crates/server/tests/e2e.rs` to match actual cascade semantics. ≤1 file edit, 3 line-region changes.

**Shape G (Layer G2 push-and-exit):** do NOT run cargo locally. After your commit, push to origin and write a `kind: "validate-pending"` DQ entry referencing the new `cargo-validate-workspace.yml` run id.

## 2. Scope

**Produce** (one commit):

- `crates/server/tests/e2e.rs` — fix `config_get_int_cascade_resolves_founder_severe_to_bare_then_const` test step 3 assertion (around lines 7677-7702).

**Do NOT**:
- Touch any other file (do NOT modify `crates/api/api/src/governance/config.rs` — the cascade implementation is correct).
- Run any cargo command locally.
- Write a new DQ entry beyond the `validate-pending` entry Shape G requires.
- Rename the test function.

**Commit message** (exactly): `test(v1-JM-e): fix cascade test step 3 — most-specific const wins (task 1 fix3)`

## 3. Background

The e2e test `config_get_int_cascade_resolves_founder_severe_to_bare_then_const` failed deterministically on the JM-e Phase 2 run with `left: 9, right: 5` at step 3.

**This is a test-authoring bug, not a code regression.** The cascade in `crates/api/api/src/governance/config.rs::get_int_cascade` (lines 389-443) walks DB candidates most-specific-first, then falls through to `const_default_int` over the SAME candidate list (lines 434-436):

```rust
candidates
    .iter()
    .find_map(|c| const_default_int(c))
    .ok_or_else(...)
```

For input `(namespace="jury.panel_size", segments=&["founder", "severe"])`, `candidates` is `["jury.panel_size.founder.severe", "jury.panel_size.severe", "jury.panel_size"]`.

When all DB rows are absent, `find_map` finds the first `Some` in the const table. Per `const_default_int` (line 958):
```rust
"jury.panel_size.founder.severe" => Some(DEFAULT_JURY_PANEL_SIZE_FOUNDER_SEVERE),  // = 9
```

So the cascade correctly returns `9` (the most-specific const), NOT `5` (the bare const). The v1-JM-a registry deliberately added per-tier consts so const fallback preserves cascade specificity.

The test's step 3 expected `5` per a pre-JM-a mental model. The fix updates the assertion + message + docstring to match real semantics.

## 4. The fix (precise)

### Edit 1 — docstring at line 7602-7606

Find this block (exact text to match):
```rust
/// v1-JM-b Task 7 test 6 — cascade walks
/// `jury.panel_size.founder.severe` → `jury.panel_size.severe` →
/// bare `jury.panel_size` → `DEFAULT_JURY_PANEL_SIZE` const. Uses raw SQL
/// to delete config rows between walks since `governance_config` is
/// append-only via `valid_from` but has no DELETE-forbid trigger.
```

Replace with:
```rust
/// v1-JM-b Task 7 test 6 — cascade walks
/// `jury.panel_size.founder.severe` → `jury.panel_size.severe` →
/// bare `jury.panel_size` (DB rows). When all DB rows are absent, the
/// const-table fallback walks the SAME candidate list (most-specific
/// first) and returns the per-tier const
/// `DEFAULT_JURY_PANEL_SIZE_FOUNDER_SEVERE = 9`. Uses raw SQL to delete
/// config rows between walks since `governance_config` is append-only
/// via `valid_from` but has no DELETE-forbid trigger.
```

### Edit 2 — assertion at lines 7697-7700

Find this block (exact text to match):
```rust
    assert_eq!(
      got, 5,
      "cascade falls to const DEFAULT_JURY_PANEL_SIZE = 5 when DB has no matching row"
    );
```

Replace with:
```rust
    assert_eq!(
      got, 9,
      "cascade falls to per-tier const DEFAULT_JURY_PANEL_SIZE_FOUNDER_SEVERE = 9 \
       when DB has no matching row (most-specific-first const cascade per v1-JM-a)"
    );
```

### Important notes

- Steps 1 and 2 of the test (lines ~7618-7675) are CORRECT and must NOT be changed — they exercise the DB-row cascade and pass.
- The test name itself (`..._resolves_founder_severe_to_bare_then_const`) is preserved — the docstring update communicates the corrected semantics.
- Do NOT change anything in `crates/api/api/src/governance/config.rs`. The cascade implementation is correct.

## 4a. Constraints

### Branch + commit discipline
- You start on a Junior worktree branched off `phase-v1-JM-e` (tip: `96b30eaff` — confirm via `git log --oneline -3`).
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

Write a `kind: "validate-pending"` DQ entry in `.claude/decision-queue.json` (next_id = max of all ids in pending + resolved + 1):

```json
{
  "id": <next_id>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<UTC ISO 8601>",
  "question": "workspace-check for v1-JM-e task 1 fix3",
  "options": ["pass", "fail"],
  "context": "cargo-validate-workspace.yml triggered on push to <branch>",
  "workflow_run_id": <databaseId>,
  "branch": "<your-worktree-branch>",
  "phase_task": "1-fix3",
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

Commit the DQ entry: `chore(decision-queue): impl raised DQ #<id> — validate-pending task 1 fix3`
Push that commit too.

## 5. Validation gate (Shape G)

**No local cargo.** Validation is:
1. Push to `origin/<worktree-branch>`.
2. Confirm `cargo-validate-workspace.yml` triggered (via `gh run list`).
3. Write `kind: "validate-pending"` DQ entry with `workflow_run_id`.
4. Commit + push DQ entry.

## 6. Expected output (return to advisor)

```
## fix-impl-3 complete — cascade test assertion fixed

**Commit:** <sha> on <worktree-branch>
**File changed:**
  - crates/server/tests/e2e.rs (1 docstring + 1 assertion + 1 assertion message at lines ~7602-7702)
**DQ raised:** #<id> kind: validate-pending, workflow_run_id: <id>, branch: <branch>
**Next:** advisor queues ci-watcher for DQ #<id>; on workspace-check pass, advisor re-dispatches Phase 2 e2e
```
