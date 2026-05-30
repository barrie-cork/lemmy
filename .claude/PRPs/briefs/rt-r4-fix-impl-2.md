# Brief: v1-RT-r4 fix-impl-2 — CR #164 remaining majors (cr-2 dup-guard, cr-3 scrub, cr-7 exists)

## 1. Role + dispatch

```
[role:impl-task] v1-RT-r4 fix-impl-2 CR #164 — dup-guard + scrub note + exists-query — see .claude/PRPs/briefs/rt-r4-fix-impl-2.md
```

## 2. Scope

fix-impl-1 (commit `972e27ccf`, already on the phase branch) fixed cr-1/cr-4/cr-5 but **silently dropped cr-2 and cr-3 — the two highest-value majors.** This brief applies ONLY those two + the related cr-7 perf nit. **Two files**, both small localized edits.

### 2.0 MANDATORY pre-flight — confirm you forked from the RIGHT base

fix-impl-1 forked from a STALE base and missed these fixes. Before ANY edit, verify your worktree HEAD contains the fix-impl-1 commit:
```bash
git log --oneline -3
# You MUST see 972e27ccf "fix(rt-r4): CR #164 sponsor-gate + allowlist handler fixes (fix-impl-1)" in your history.
git merge-base --is-ancestor 972e27ccf HEAD && echo "BASE OK" || echo "STALE BASE — STOP, raise blocker DQ"
```
If you see `STALE BASE`, STOP and raise a `kind: "blocker"` DQ — do NOT proceed (you would re-drop cr-1/4/5 or conflict).

### Files
- `crates/api/api/src/governance/admin_sponsor_allowlist.rs` — cr-2 (dup-guard in `add`), cr-3 (scrub payload in `add` + `remove`)
- `crates/db_schema/src/source/governance/sponsor_allowlist.rs` — cr-7 (`select(exists)` not `.first`)

**DO NOT** touch: `create_endorsement.rs`, `e2e.rs` (fix-impl-1 already handled those — touching them risks reverting cr-1/cr-5), `Cargo.toml`, migrations, routes, mod.rs, the DTOs.

---

### 2.1 cr-2 (MAJOR) — add() must reject duplicate rows before insert

**File:** `crates/api/api/src/governance/admin_sponsor_allowlist.rs`, the `add` handler's `run_transaction` closure (currently `:65-96` — RE-LOCATE; line numbers shifted from fix-impl-1).

**Problem (verified by advisor):** `sponsor_allowlist` has `UNIQUE (community_id, person_id)` but r1 dropped `community_id` to nullable WITHOUT a partial unique index for the NULL case (migration `2026-05-10-...-extend_sponsor_allowlist_for_r1/up.sql:11`). Postgres treats NULLs as distinct → `add` can insert unlimited duplicate **instance-wide** rows (`community_id IS NULL`) for the same person; `remove` deletes one (`.first`); the governance log double-emits `sponsor_allowlist_added`.

**Fix (in-handler, NO migration):**

(a) Add `sponsor_allowlist_exists` to the existing `lemmy_db_schema::source::governance::sponsor_allowlist` import (currently:
```rust
use lemmy_db_schema::source::governance::sponsor_allowlist::{
  SponsorAllowlist, SponsorAllowlistInsertForm, sponsor_allowlist_delete, sponsor_allowlist_insert,
};
```
→ add `sponsor_allowlist_exists,` to the brace list, keeping alphabetical-ish order).

(b) Inside the `run_transaction` closure, IMMEDIATELY before `let form = SponsorAllowlistInsertForm {`, insert the guard:
```rust
      // Reject duplicates — UNIQUE(community_id, person_id) treats NULL community_id
      // as distinct, so the DB does not block instance-wide duplicates on its own.
      if sponsor_allowlist_exists(person_id, community_id, conn).await? {
        return Err(LemmyErrorType::Unknown(
          "sponsor_allowlist row already exists for this person/community".to_string(),
        )
        .into());
      }
```
`person_id` and `community_id` are already bound locals in the closure scope (pre-cloned before `run_transaction`). `LemmyErrorType` is already imported.

**Convention precedent:** `admin_rule_sets` uses `LemmyErrorType::Unknown("rule_set_version already exists ...")` for a duplicate (see `crates/server/tests/e2e.rs:7263`). `Unknown` is the established convention for "duplicate" in this codebase.

---

### 2.2 cr-3 (MAJOR) — scrub the user-provided note before logging (ADR-015)

**File:** same file. The `add` handler payload (`json!{}` currently `:77-84`) AND the `remove` handler payload (currently `:159-165`).

**Problem (verified by advisor):** The raw user-provided `note` flows into the governance-log payload un-scrubbed (`add` payload, `"note": note`). `.coderabbit.yaml:108-110` + ADR-015 require every user-visible string in a log payload to pass `scrub()`/redaction before leaving the handler.

**Fix:** Wrap each payload in `scrub_json(...)` at its `governance_log::append` call. `scrub_json` recursively scrubs every string value; keys + non-string scalars (ints, timestamps) pass through unchanged.

(a) Add the import. The existing top-of-file `use crate::governance::{...}` block is:
```rust
use crate::governance::{
  actor_pseudonym_helper,
  governance_log::{self, ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED, ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED},
};
```
→ add `redaction::scrub_json,` as a member (alphabetical: after `governance_log::{...}`):
```rust
use crate::governance::{
  actor_pseudonym_helper,
  governance_log::{self, ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED, ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED},
  redaction::scrub_json,
};
```

(b) In the `add` handler `append` call, change `payload,` → `scrub_json(&payload),`:
```rust
      governance_log::append(
        &mut conn.into(),
        ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED,
        scrub_json(&payload),
        Some(admin_pseudonym.clone()),
      )
      .await?;
```

(c) Same in the `remove` handler `append` call: `payload,` → `scrub_json(&payload),` (with `ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED`).

**Idiom precedent (read before writing):** `crates/api/api/src/governance/admin_rule_sets.rs:52` imports `redaction::scrub`; `:320` applies it. `scrub`/`scrub_json` are re-exported from `crates/api/api/src/governance/redaction.rs:22` (`pub use lemmy_db_schema::source::governance::redaction::{scrub, scrub_json};`). The `scrub_json` def is at `crates/db_schema/src/source/governance/redaction.rs:88` — it keeps object keys + non-string scalars intact, so `allowlist_id`/`added_at`/pseudonyms are unaffected; only the free-text `note` value is scrubbed.

---

### 2.3 cr-7 (LOW) — sponsor_allowlist_exists should not materialize the row

**File:** `crates/db_schema/src/source/governance/sponsor_allowlist.rs`, the `sponsor_allowlist_exists` fn (currently `:75-102`).

**Problem (verified):** Lines 86/95 do `.first::<SponsorAllowlist>(conn).await.optional()?.is_some()` — selects every column of the row to discard it, on the endorsement hot path (now also called by cr-2's guard).

**Fix:** Use `SELECT EXISTS(...)` returning a bool. Replace the two match-arm bodies:
```rust
#[cfg(feature = "full")]
pub async fn sponsor_allowlist_exists(
  person_id: PersonId,
  community_id: Option<CommunityId>,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<bool> {
  use diesel::dsl::{exists, select};
  let found = match community_id {
    Some(cid) => {
      select(exists(
        sponsor_allowlist::table
          .filter(sponsor_allowlist::person_id.eq(person_id))
          .filter(sponsor_allowlist::community_id.eq(cid)),
      ))
      .get_result::<bool>(conn)
      .await?
    }
    None => {
      select(exists(
        sponsor_allowlist::table
          .filter(sponsor_allowlist::person_id.eq(person_id))
          .filter(sponsor_allowlist::community_id.is_null()),
      ))
      .get_result::<bool>(conn)
      .await?
    }
  };
  Ok(found)
}
```
After this change, `OptionalExtension` / `SelectableHelper` may become unused IN THIS FN but may still be used by `sponsor_allowlist_insert`/`_delete`. **Run `cargo check` and only remove an import if the compiler flags it unused.** Do NOT pre-emptively delete imports.

---

### 2.4 Validate (validate-pending-laptop)

Run BEFORE committing:
```bash
cargo check --workspace --features full
cargo clippy --workspace --features full --no-deps -- -D warnings
cargo test --workspace --test e2e --no-run --features full
```
All three must exit 0. Push only on exit 0. (No new e2e tests in this brief — fix-impl-1 already added the deny-assertion edits; the existing `admin_allowlist_add_remove_round_trip` test still covers add/remove.)

Then write a `kind: "validate-pending-laptop-e2e"` DQ entry (id via `bash scripts/brehon/dq-v3-new-entry.sh`; append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`):
```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/validate-fix2-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/validate-fix2-e2e.log || echo E2E_EXIT_NONZERO >> .claude/validate-fix2-e2e.log"'
branch: <your worker branch>
phase_task: fix-impl-2
```
Commit + push the DQ entry mid-task (separate commit from the impl). Do NOT run full e2e yourself.

## 3. Required reading

### 3.1 Findings + the gap
`.claude/PRPs/reviews/pr-164-findings.yaml` — read cr-2, cr-3, cr-7 `notes:`. fix-impl-1 applied cr-1/4/5 only; this brief closes cr-2/cr-3/cr-7.

### 3.2 MIRROR refs (read verbatim)
- cr-3 scrub: `crates/api/api/src/governance/admin_rule_sets.rs:52,320` + `crates/api/api/src/governance/redaction.rs:22` (the shim) + `crates/db_schema/src/source/governance/redaction.rs:88` (`scrub_json` def).
- cr-2 "already exists": `crates/server/tests/e2e.rs:7263` (Unknown-for-duplicate convention).
- cr-7 exists: standard Diesel `select(exists(...))`.

### 3.3 Mandatory lessons
- `feedback_clippy_rerun_after_fix.md` + `feedback_clippy_test_style.md` — after cr-7's import change, re-run clippy (`-D warnings`).
- `feedback_fix_impl_pre_push_cargo_check.md` — pre-push cargo check is mandatory (§2.4).

## 4. Constraints

- **File-ownership:** modify ONLY the 2 files in §2 + the DQ validate-pending entry. Do NOT touch `create_endorsement.rs` or `e2e.rs` (fix-impl-1 owns those — touching them risks reverting cr-1/cr-5).
- **Pre-flight base check (§2.0) is HARD** — verify `972e27ccf` is in your history before editing. STALE base → blocker DQ, do not proceed.
- **Verified recipes are the contract.** Apply §2.1-2.3 as written. If a recipe does not compile as given, STOP and raise a `kind: "blocker"` DQ with the exact compile error — do NOT improvise (circuit-breaker: 2 retries then escalate).
- **cr-3 scrub:** `scrub_json` MUST be the redaction shim re-export — confirmed reachable by advisor. Do NOT hand-roll a scrubber.
- **Pre-push gate:** all three §2.4 cargo commands exit 0 before committing.
- **DQ mid-task push:** commit + push the validate-pending entry on your worker branch.
- **Commit subject:** `fix(rt-r4): CR #164 remaining majors — dup-guard + scrub note + exists-query (fix-impl-2, cr-2/cr-3/cr-7)`
- **Commit body:** list each cr-N + the recipe applied; note fix-impl-1 (`972e27ccf`) is the base and was NOT modified.

## 3a. Handover from prior tasks

```yaml
prior_tasks:
  - task: fix-impl-1
    commit: 972e27ccf
    appliedFixes: [cr-1 (AgeOrSurety error-match), cr-4 (NotFound + delete-count race guard), cr-5 (e2e NotFound asserts)]
    droppedFixes: [cr-2 (dup-guard), cr-3 (scrub note), cr-7 (exists-query)]   # THIS brief closes these
    notes: "fix-impl-1 forked stale + silently dropped cr-2/cr-3/cr-7. Daemon local phase-v1-RT-r4 FF'd to 972e27ccf before this dispatch. All recipes verified vs source by advisor."
context:
  phase_branch_tip: 972e27ccf
  pr: 164
```
