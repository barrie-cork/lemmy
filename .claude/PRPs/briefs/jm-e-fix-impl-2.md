---
role: impl-task
plan_task: 1-fix2
phase: v1-JM-e
created: 2026-05-01
related_dq: 101
---

# Brief — v1-JM-e fix-impl-2 — Fix LocalSiteInsertForm::new call sites in e2e.rs

## 1. Role + dispatch line

`[role:impl-task] v1-JM-e fix-impl-2 — see .claude/PRPs/briefs/jm-e-fix-impl-2.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a narrow fix-impl-task: update two `LocalSiteInsertForm` construction sites in `crates/server/tests/e2e.rs` to match the updated `::new()` signature. ≤1 file edit, 2 call sites.

**Shape G (Layer G2 push-and-exit):** do NOT run cargo locally. After your commit, push to origin and write a `kind: "validate-pending"` DQ entry referencing the new `cargo-validate-workspace.yml` run id.

## 2. Scope

**Produce** (one commit):

- `crates/server/tests/e2e.rs` — fix two `LocalSiteInsertForm` construction sites (lines ~3820 and ~4087).

**Do NOT**:
- Touch any other file.
- Run any cargo command locally.
- Write a new DQ entry beyond the `validate-pending` entry Shape G requires.

**Commit message** (exactly): `fix(v1-JM-e): update LocalSiteInsertForm::new call sites — system_account now required (task 1 fix2)`

## 3. Background

`LocalSiteInsertForm` in `crates/db_schema/src/source/local_site.rs` was updated so `system_account: PersonId` is now a **required positional field** in `::new()` (not `#[new(default)]`). The constructor signature is now:

```rust
LocalSiteInsertForm::new(site_id: SiteId, system_account: PersonId)
```

Two existing e2e fixture blocks still use the old pattern:

```rust
// OLD (broken — does not compile):
let local_site_form_x = LocalSiteInsertForm {
  system_account: Some(sysacct.id),   // wrong: field is PersonId, not Option<PersonId>
  ..LocalSiteInsertForm::new(site_x.id)  // wrong: ::new() now requires 2 args
};
```

## 4. The fix (precise)

### Call site 1 — around line 3823 (instance A scaffold)

Find this block (exact text to match):
```rust
    let local_site_form_a = LocalSiteInsertForm {
      system_account: Some(sysacct.id),
      ..LocalSiteInsertForm::new(site_a.id)
    };
```

Replace with:
```rust
    let local_site_form_a = LocalSiteInsertForm::new(site_a.id, sysacct.id);
```

### Call site 2 — around line 4090 (instance B scaffold)

Find this block (exact text to match):
```rust
    let local_site_form_b = LocalSiteInsertForm {
      system_account: Some(sysacct.id),
      ..LocalSiteInsertForm::new(site_b.id)
    };
```

Replace with:
```rust
    let local_site_form_b = LocalSiteInsertForm::new(site_b.id, sysacct.id);
```

Both `sysacct` variables are already in scope at each call site — they are created immediately above each block via `Person::create(pool, &sysacct_form).await`.

## 4a. Constraints

### Branch + commit discipline
- You start on a Junior worktree branched off `phase-v1-JM-e` (tip: check `git log --oneline -3` — should include the fix-impl-1 commit).
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

Write a `kind: "validate-pending"` DQ entry in `.claude/decision-queue.json` (next_id = max of all ids + 1, scanning both pending and resolved):

```json
{
  "id": <next_id>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<UTC ISO 8601>",
  "question": "workspace-check for v1-JM-e task 1 fix2",
  "options": ["pass", "fail"],
  "context": "cargo-validate-workspace.yml triggered on push to <branch>",
  "workflow_run_id": <databaseId>,
  "branch": "<your-worktree-branch>",
  "phase_task": "1-fix2",
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

Commit the DQ entry: `chore(decision-queue): impl raised DQ #<id> — validate-pending task 1 fix2`
Push that commit too.

## 5. Validation gate (Shape G)

**No local cargo.** Validation is:
1. Push to `origin/<worktree-branch>`.
2. Confirm `cargo-validate-workspace.yml` triggered (via `gh run list`).
3. Write `kind: "validate-pending"` DQ entry with `workflow_run_id`.
4. Commit + push DQ entry.

## 6. Expected output (return to advisor)

```
## fix-impl-2 complete — LocalSiteInsertForm::new call sites updated

**Commit:** <sha> on <worktree-branch>
**File changed:**
  - crates/server/tests/e2e.rs (2 call sites: local_site_form_a + local_site_form_b)
**DQ raised:** #<id> kind: validate-pending, workflow_run_id: <id>, branch: <branch>
**Next:** advisor queues ci-watcher for DQ #<id>
```
