# Brief: v1-RT-r4 Task 3 — Strategy enum + dispatch

## 1. Role + dispatch

```
[role:impl-task] v1-RT-r4 task 3 strategy enum + dispatch in create_endorsement.rs — see .claude/PRPs/briefs/rt-r4-impl-3.md
```

## 2. Scope

**One file modified:** `crates/api/api_crud/src/governance/create_endorsement.rs`

**Deliver exactly:**

### 2.1 Enum — add 3 variants (`:84`)
In `enum SponsorGateStrategy`, add `AgeOrSurety`, `Reputation`, `Allowlist` ABOVE `Unknown(String)`:
```rust
enum SponsorGateStrategy {
  Age,
  AgeOrSurety,
  Reputation,
  Allowlist,
  Open,
  Closed,
  Unknown(String),
}
```

### 2.2 `parse()` — add 3 mappings (`:92`)
Inside `fn parse(s: &str)`, add before `other => Self::Unknown(...)`:
```rust
"age_or_surety" => Self::AgeOrSurety,
"reputation" => Self::Reputation,
"allowlist" => Self::Allowlist,
```

### 2.3 `label()` — add 3 arms (`:102`)
Inside `fn label(&self)`, add before `Self::Unknown(s) => s.as_str()`:
```rust
Self::AgeOrSurety => "age_or_surety",
Self::Reputation => "reputation",
Self::Allowlist => "allowlist",
```

### 2.4 `match &strategy` — add 3 arms (`:161`)
In `process_endorsement`, inside the match block, add ABOVE `SponsorGateStrategy::Unknown(s)`:
```rust
SponsorGateStrategy::AgeOrSurety => {
  let age_ok = enforce_age_gate(conn, &mut config, sponsor_id).await.is_ok();
  let surety_exists: bool = surety::table
    .filter(surety::sponsored_id.eq(sponsor_id))
    .filter(surety::revoked_at.is_null())
    .select(count_star())
    .get_result::<i64>(conn)
    .await? > 0;
  if !age_ok && !surety_exists {
    return Err(LemmyErrorType::NotFound.into());
  }
}
SponsorGateStrategy::Reputation => {
  let can_sponsor: bool = reputation_snapshot::table
    .filter(reputation_snapshot::person_id.eq(sponsor_id))
    .filter(reputation_snapshot::community_id.is_not_distinct_from(data.community_id))
    .select(reputation_snapshot::can_sponsor)
    .first::<bool>(conn)
    .await
    .unwrap_or(false);
  if !can_sponsor {
    return Err(LemmyErrorType::NotFound.into());
  }
}
SponsorGateStrategy::Allowlist => {
  let on_list = sponsor_allowlist_exists(sponsor_id, data.community_id, conn).await?;
  if !on_list {
    return Err(LemmyErrorType::NotFound.into());
  }
}
```

**MIRROR:** existing `Age` (`:168`) and `Unknown(s)` (`:170-172`) arms for the gate-call and error shape.

**GOTCHA-55a (HARD):** NO `_ =>` catchall. `Unknown(String)` MUST remain the final arm. Workspace clippy denies `_ =>` under `feedback_clippy_test_style`. The new arms are explicit and named; do NOT add a wildcard.

**DO NOT** author — do not create any new files, do not touch `mod.rs`, do not touch migrations, do not touch `e2e.rs`.

### 2.5 Imports
Add any missing `use` entries needed by the new arms. Expected additions at the top of the file (after existing uses):
- `use crate::governance::sponsor_allowlist::sponsor_allowlist_exists;` (the T1 helper)
- Diesel imports for `surety::table`, `reputation_snapshot::table`, `reputation_snapshot::can_sponsor` if not already present — check existing imports first, add only what is missing.

### 2.6 Validate (laptop — via DQ validate-pending-laptop)
Write a `kind: "validate-pending-laptop"` DQ entry (v3 id via `bash scripts/brehon/dq-v3-new-entry.sh`) with:
```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/validate-t3.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/validate-t3-clippy.log 2>&1"'
branch: <your worker branch>
phase_task: 3
```
Commit + push the DQ entry. Do NOT run cargo check yourself — the laptop advisor runs it.

**NOTE (pre-existing):** `-p lemmy_api_crud --features full` fails with a pre-existing dep-feature-propagation issue (`lemmy_db_views_reputation/full` not activated via that path). Use `--workspace --features full` per plan §15.1/§15.2.

## 3. Required reading

### 3.1 Plan
`.claude/PRPs/plans/v1-RT-r4.plan.md` — read Task 3 (§13, `:420-460`) for the full IMPLEMENT spec and GOTCHA-55a detail.

### 3.2 MIRROR ref
`crates/api/api_crud/src/governance/create_endorsement.rs` `:78-192` — the existing `SponsorGateStrategy` enum, `parse()`, `label()`, and the `match &strategy` block are the canonical shape. Read them verbatim before writing.

### 3.3 T1 helper
`crates/db_schema/src/source/governance/sponsor_allowlist.rs` — the `sponsor_allowlist_exists` fn signature (T1 commit). Import path: `crate::governance::sponsor_allowlist::sponsor_allowlist_exists` (check actual mod path in the file).

### 3.4 Mandatory lessons
- `feedback_clippy_test_style.md` — GOTCHA-55a: no `_ =>` catchall; workspace clippy denies it.
- `feedback_features_full_p_crate_incompatible.md` — use `--workspace`, NOT `-p lemmy_api_crud`.

## 4. Constraints

- **File-ownership:** modify ONLY `create_endorsement.rs`. No other files except the DQ entry commit.
- **GOTCHA-55a:** no `_ =>` arm. `Unknown(String)` is the exhaustive-final arm — never replace it with a wildcard.
- **Pre-push gate:** run `bash scripts/brehon/cargo-check.sh --workspace --features full` before committing. Non-zero → fix inline. Do NOT push a broken workspace.
- **DQ write:** use `bash scripts/brehon/dq-v3-new-entry.sh` for the id. Mid-task push the DQ commit immediately after writing it.
- **No impl code in DQ commit** — DQ commit is separate from the impl commit.
- **Commit subject:** `feat(rt-r4): add SponsorGateStrategy AgeOrSurety/Reputation/Allowlist arms (task 3)`

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1
    commit: a4c5c2dbd
    filesCreated: []
    filesModified:
      - crates/db_schema/src/source/governance/sponsor_allowlist.rs
    keyDecisions:
      - sponsor_allowlist_exists takes (PersonId, Option<CommunityId>, &mut AsyncPgConnection)
      - NullableExpressionMethods import was unnecessary — ExpressionMethods covers .is_null()
    notes: "3 inline helpers: sponsor_allowlist_insert, sponsor_allowlist_delete, sponsor_allowlist_exists. +59 lines. Clippy clean (one unused-import self-fix before commit)."
  - task: 2
    commit: 194f2442a
    filesCreated: []
    filesModified:
      - crates/api/api_common/src/governance.rs
    keyDecisions:
      - 4 DTOs appended as Group E; derive stack verbatim from AdminSetConfig:444
      - SponsorAllowlistId imported from lemmy_db_schema::newtypes
    notes: "AddSponsorAllowlist, AddSponsorAllowlistResponse, RemoveSponsorAllowlist, RemoveSponsorAllowlistResponse. +39 lines. Workspace check + clippy clean."
```
