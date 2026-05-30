# Brief: v1-RT-r4 Task 4 — Admin handlers + mod decl

## 1. Role + dispatch

```
[role:impl-task] v1-RT-r4 task 4 admin_sponsor_allowlist handlers + mod decl — see .claude/PRPs/briefs/rt-r4-impl-4.md
```

## 2. Scope

**Creates:** `crates/api/api/src/governance/admin_sponsor_allowlist.rs`
**Modifies:** `crates/api/api/src/governance/mod.rs`

**Requires:** tasks 1 and 2 (db-helpers + DTOs) are already on the phase branch.

### 2.1 §G4 CANONICAL RECIPE (verbatim from plan §13 Task 4)

> **MIRROR:** `admin_config.rs:383-558` (the conformance-audit axis most likely to drift).
> Pseudonym at `:485`; `run_transaction` write branch `:497-558`.
> **GOTCHA:** `run_transaction` has NO SAVEPOINT (`admin_config.rs:10`);
> `governance_log::append` opens its OWN internal tx — do NOT wrap append in a second explicit tx.
> ADR-015: payload carries `person_pseudonym` + `added_by_admin_pseudonym`/`removed_by_admin_pseudonym`, NEVER raw `person_id`.

### 2.2 admin_sponsor_allowlist.rs — two handlers

Both follow the same step ordering as `admin_set_config` (`:383-558`):

**`pub async fn add`** signature:
```rust
pub async fn add(
  Json(data): Json<AddSponsorAllowlist>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AddSponsorAllowlistResponse>>
```

Steps:
1. `is_admin(&local_user_view)?` — mirror `admin_config.rs:62`/`:753`
2. Get `admin_id = local_user_view.person.id`
3. `let admin_pseudonym = actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?`
4. `let person_pseudonym = actor_pseudonym_helper::get_or_create(&mut context.pool(), data.person_id).await?`
5. Inside `conn.run_transaction(async |conn| { ... })`:
   - Build `SponsorAllowlistInsertForm { person_id: data.person_id, community_id: data.community_id, note: data.note.clone() }`
   - `let allowlist_id = sponsor_allowlist_insert(&form, conn).await?`
   - Build payload: `serde_json::json!({ "person_pseudonym": person_pseudonym, "added_by_admin_pseudonym": admin_pseudonym, "community_id": data.community_id, "note": data.note })`
   - `governance_log::append(&mut conn.into(), ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED, &payload, conn).await?`
   - Return `Ok(AddSponsorAllowlistResponse { allowlist_id })`

**`pub async fn remove`** signature:
```rust
pub async fn remove(
  Json(data): Json<RemoveSponsorAllowlist>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<RemoveSponsorAllowlistResponse>>
```

Steps:
1. `is_admin(&local_user_view)?`
2. Get `admin_id = local_user_view.person.id`
3. `let admin_pseudonym = actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?`
4. `let person_pseudonym = actor_pseudonym_helper::get_or_create(&mut context.pool(), data.person_id).await?`
5. Resolve `allowlist_id`: query `sponsor_allowlist` table for `(person_id, community_id)` — use a Diesel filter on `sponsor_allowlist::person_id.eq(data.person_id)` and `sponsor_allowlist::community_id.is_not_distinct_from(data.community_id)`, `.select(sponsor_allowlist::id).first(conn).await.optional()?` → `NotFound` if absent.
6. Inside `conn.run_transaction(async |conn| { ... })`:
   - `sponsor_allowlist_delete(allowlist_id, conn).await?`
   - Build payload: `serde_json::json!({ "person_pseudonym": person_pseudonym, "removed_by_admin_pseudonym": admin_pseudonym, "community_id": data.community_id })`
   - `governance_log::append(&mut conn.into(), ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED, &payload, conn).await?`
   - Return `Ok(RemoveSponsorAllowlistResponse { success: true })`

### 2.3 mod.rs — one line added

In `crates/api/api/src/governance/mod.rs`, add:
```rust
pub mod admin_sponsor_allowlist;
```
Alphabetically between adjacent existing `pub mod` lines (check current sibling list — `admin_sponsor_allowlist` sorts after `admin_set_config` and before `admin_…` names starting later in the alphabet; read the file first).

### 2.4 Validate (via DQ validate-pending-laptop)

Write `kind: "validate-pending-laptop"` DQ entry (id via `bash scripts/brehon/dq-v3-new-entry.sh`):
```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/validate-t4.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/validate-t4-clippy.log 2>&1"'
branch: <your worker branch>
phase_task: 4
```
Commit + push the DQ entry. Do NOT run cargo yourself.

## 3. Required reading

### 3.1 Plan
`.claude/PRPs/plans/v1-RT-r4.plan.md` §13 Task 4 (`:460-510`) — full IMPLEMENT spec.

### 3.2 MIRROR ref (read before writing)
`crates/api/api/src/governance/admin_config.rs` `:383-558` — step ordering, pseudonym pattern, run_transaction shape, governance_log::append call. Read these lines verbatim before writing any handler code.

### 3.3 T1 helpers
`crates/db_schema/src/source/governance/sponsor_allowlist.rs` — read the signatures of `sponsor_allowlist_insert`, `sponsor_allowlist_delete`, `sponsor_allowlist_exists` and the `SponsorAllowlistInsertForm` struct (fields, types).

### 3.4 T2 DTOs
`crates/api/api_common/src/governance.rs` — read `AddSponsorAllowlist`, `AddSponsorAllowlistResponse`, `RemoveSponsorAllowlist`, `RemoveSponsorAllowlistResponse` field names and types.

### 3.5 ENTRY_KIND constants
`crates/api/api/src/governance/admin_config.rs` — find `ENTRY_KIND_ADMIN_CONFIG_CHANGED` to understand the import path, then check `.claude/rules/governance-log-entry-kind-registry.md` for `ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED` and `ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED` (rows `:190-191`).

### 3.6 Mandatory lessons
- `feedback_multi_write_handlers_need_transactions.md` — both add and remove do 2+ DB writes; both MUST use `run_transaction`.
- `feedback_clippy_test_style.md` — no `unwrap`, no `expect`, LemmyResult + `?`.

## 4. Constraints

- **File-ownership:** create ONLY `admin_sponsor_allowlist.rs`; modify ONLY `mod.rs`; plus the DQ entry. No other files.
- **ADR-015 (HARD):** payload MUST use `person_pseudonym` and `admin_pseudonym` — NEVER raw `person_id` or `admin_id` in any log payload.
- **ADR-008 (HARD):** governance_log written via `governance_log::append` only, never direct INSERT.
- **No SAVEPOINT:** do NOT wrap `governance_log::append` in a second explicit tx. It opens its own.
- **Alphabetical mod.rs:** verify position by reading the file first.
- **Pre-push gate:** `bash scripts/brehon/cargo-check.sh --workspace --features full` before committing. Non-zero → fix inline.
- **DQ write:** separate commit from impl; push mid-task.
- **Commit subject:** `feat(rt-r4): add admin_sponsor_allowlist handlers add+remove (task 4)`

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1
    commit: a4c5c2dbd
    filesCreated: []
    filesModified:
      - crates/db_schema/src/source/governance/sponsor_allowlist.rs
    keyDecisions:
      - sponsor_allowlist_insert(form, conn) -> LemmyResult<SponsorAllowlistId>
      - sponsor_allowlist_delete(id, conn) -> LemmyResult<()>
      - sponsor_allowlist_exists(PersonId, Option<CommunityId>, conn) -> LemmyResult<bool>
      - SponsorAllowlistInsertForm { person_id, community_id: Option<CommunityId>, note: Option<String> }
    notes: "3 inline db-helpers, +59 lines. Clippy clean."
  - task: 2
    commit: 194f2442a
    filesCreated: []
    filesModified:
      - crates/api/api_common/src/governance.rs
    keyDecisions:
      - AddSponsorAllowlist { person_id: PersonId, community_id: Option<CommunityId>, note: Option<String> }
      - AddSponsorAllowlistResponse { allowlist_id: SponsorAllowlistId }
      - RemoveSponsorAllowlist { person_id: PersonId, community_id: Option<CommunityId> }
      - RemoveSponsorAllowlistResponse { success: bool }
    notes: "4 DTOs with AdminSetConfig:444 derive stack. Workspace check + clippy clean."
  - task: 3
    commit: 88f4a0d41
    filesCreated: []
    filesModified:
      - crates/api/api_crud/src/governance/create_endorsement.rs
    keyDecisions:
      - SponsorGateStrategy now has 7 arms — Age/AgeOrSurety/Reputation/Allowlist/Open/Closed/Unknown(String)
      - reputation_snapshot schema import aliased as rs_snapshot to avoid collision with module name
      - GOTCHA-55a preserved — no _ => catchall
    notes: "enum+parse+label+3 dispatch arms. Workspace check + clippy clean."
```
