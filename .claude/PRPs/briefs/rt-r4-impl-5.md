# Brief: v1-RT-r4 Task 5 — Route registration

## 1. Role + dispatch

```
[role:impl-task] v1-RT-r4 task 5 route registration for admin_sponsor_allowlist in routes/lib.rs — see .claude/PRPs/briefs/rt-r4-impl-5.md
```

## 2. Scope

**One file modified:** `crates/api/routes/src/lib.rs`

**Deliver exactly:**

### 2.1 Use import

Add `admin_sponsor_allowlist` to the existing governance `use` block near the top of the file. Find where the other governance admin handlers are imported (e.g. `admin_config::{admin_get_config, ...}`, `admin_assign_jury`, etc.) and add:
```rust
admin_sponsor_allowlist::{add as admin_allowlist_add, remove as admin_allowlist_remove},
```
or follow the existing import style exactly (check whether sibling handlers use aliased or direct names).

### 2.2 Route registration

Inside the `scope("/admin")` block (`:493`), add a new `.service(...)` entry for sponsor-allowlist, mirroring the `/config`, `/rule-sets`, and `/emergency-remove` sibling scopes (`:504-523`):

```rust
.service(
  scope("/sponsor-allowlist")
    .route("/add", post().to(admin_allowlist_add))
    .route("/remove", post().to(admin_allowlist_remove)),
)
```

Place it alphabetically among the sibling `.service(scope(...))` calls within the admin scope — after `/rule-sets` and before any later-alphabetical entries. Read the file to determine the exact insertion point.

**GOTCHA (plan §13 Task 5):** Path prefix is relative to the existing admin scope. The full resolved path will be `/api/v4/governance/admin/sponsor-allowlist/{add,remove}`. Do NOT re-prefix the full path — register only the relative `/sponsor-allowlist` scope.

### 2.3 Validate (via DQ validate-pending-laptop)

Write `kind: "validate-pending-laptop"` DQ entry (id via `bash scripts/brehon/dq-v3-new-entry.sh`):
```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/validate-t5.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/validate-t5-clippy.log 2>&1"'
branch: <your worker branch>
phase_task: 5
```
Commit + push the DQ entry separately from the impl commit.

## 3. Required reading

### 3.1 Plan
`.claude/PRPs/plans/v1-RT-r4.plan.md` §13 Task 5 (`:510-530`) — full IMPLEMENT spec.

### 3.2 MIRROR ref (read before writing)
`crates/api/routes/src/lib.rs` `:490-525` — the existing admin scope `.service(scope("/config") ...)`, `.service(scope("/rule-sets") ...)`, `.service(scope("/emergency-remove") ...)` registrations. Mirror their exact shape.

### 3.3 T4 handler names
`crates/api/api/src/governance/admin_sponsor_allowlist.rs` — read the actual `pub async fn` names (`add`, `remove`) to confirm correct import and `to()` usage.

## 4. Constraints

- **File-ownership:** modify ONLY `crates/api/routes/src/lib.rs` (plus DQ entry commit). No other files.
- **Relative path only:** register `/sponsor-allowlist` relative to admin scope. Never the full path.
- **Read before writing:** check the file's existing import block and admin scope service list before adding anything — do not duplicate.
- **Pre-push gate:** `bash scripts/brehon/cargo-check.sh --workspace --features full` before committing. Non-zero → fix inline.
- **DQ write:** separate commit from impl; push mid-task.
- **Commit subject:** `feat(rt-r4): register admin_sponsor_allowlist routes add+remove (task 5)`

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 4
    commit: f06c38a02
    filesCreated:
      - crates/api/api/src/governance/admin_sponsor_allowlist.rs
    filesModified:
      - crates/api/api/src/governance/mod.rs
    keyDecisions:
      - Handler fn names are `add` and `remove` (pub async fn)
      - mod.rs: pub mod admin_sponsor_allowlist; inserted alphabetically between admin_rule_sets and admin_trigger_appeal_rejury
      - Both handlers return LemmyResult<Json<...Response>>
    notes: "Handlers follow admin_set_config step ordering. Workspace check + clippy clean."
```
