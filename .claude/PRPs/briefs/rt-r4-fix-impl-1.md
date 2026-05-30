# Brief: v1-RT-r4 fix-impl-1 — CR PR #164 fix-in-PR (cr-1..cr-5, cr-7)

## 1. Role + dispatch

```
[role:impl-task] v1-RT-r4 fix-impl-1 CR #164 — sponsor-gate + allowlist handler fixes — see .claude/PRPs/briefs/rt-r4-fix-impl-1.md
```

## 2. Scope

Fix six CodeRabbit findings on PR #164, all verified against source by the advisor. This is a **deliberate user-approved fix-in-PR cycle**, not a mechanical auto-fix — apply the exact recipes below. **Four files**, each edit is small and localized.

> NOTE on file count: this brief touches 4 files (3 handler/helper + 1 e2e). That exceeds the ≤3-file mechanical-auto-fix cap, but this is a human-approved CR-fix cycle with verified per-finding recipes, not a §G4 allowlist auto-queue. The cap does not apply.

### Files
- `crates/api/api_crud/src/governance/create_endorsement.rs` — cr-1
- `crates/api/api/src/governance/admin_sponsor_allowlist.rs` — cr-2, cr-3, cr-4
- `crates/db_schema/src/source/governance/sponsor_allowlist.rs` — cr-7
- `crates/server/tests/e2e.rs` — cr-5 (3 one-line assertion edits — pre-locate anchors)

**DO NOT** touch: `Cargo.toml`, `Cargo.lock`, migrations, routes/lib.rs, mod.rs, the DTOs, or any file not listed. **DO NOT** edit `.claude/decision-queue.json` except the validate-pending-laptop entry per §2.7.

---

### 2.1 cr-1 (MAJOR) — AgeOrSurety must not swallow non-denial errors

**File:** `crates/api/api_crud/src/governance/create_endorsement.rs`, the `SponsorGateStrategy::AgeOrSurety` arm at **`:180-194`**.

**Problem (verified):** Line 183 `if enforce_age_gate(...).await.is_err()` treats EVERY error as "age failed" and falls through to the surety check. `enforce_age_gate` (`:388-410`) returns `Err(LemmyErrorType::NotFound)` ONLY for an actual age denial (`:407`), but propagates DB/config-load errors via `?` at `:399` and `:404` as OTHER error types. A transient DB error during the age gate currently falls through to surety and can admit a sponsor who should be rejected.

**Fix:** Only fall through to the surety check when the error is specifically the `NotFound` age-denial. Propagate all other errors. Replace the arm body:

```rust
    SponsorGateStrategy::AgeOrSurety => {
      // Pass if age gate clears OR the caller has at least one active surety
      // (someone has vouched for them: sponsored_id = caller, revoked_at IS NULL).
      // Only a NotFound age-denial triggers the surety fallback — propagate any
      // other error (DB/config/transient) instead of silently admitting via surety.
      match enforce_age_gate(conn, &mut config, sponsor_id).await {
        Ok(()) => {}
        Err(e) if matches!(e.error_type, LemmyErrorType::NotFound) => {
          let surety_count: i64 = surety::table
            .filter(surety::sponsored_id.eq(sponsor_id))
            .filter(surety::revoked_at.is_null())
            .select(count_star())
            .get_result(conn)
            .await?;
          if surety_count == 0 {
            return Err(LemmyErrorType::NotFound.into());
          }
        }
        Err(e) => return Err(e),
      }
    }
```

**Idiom precedent (read before writing):** `crates/api/api/src/governance/admin_rule_sets.rs:73` — `Err(e) if matches!(e.error_type, LemmyErrorType::NotAModerator) => false`. `LemmyError.error_type` is a public field (`crates/utils/src/error.rs:182`).

---

### 2.2 cr-2 (MAJOR) — add() must reject duplicate rows before insert

**File:** `crates/api/api/src/governance/admin_sponsor_allowlist.rs`, the `add` handler's transaction closure at **`:67-96`**.

**Problem (verified):** The table has `UNIQUE (community_id, person_id)` (migration `2026-04-22-000100-0000_add_sponsor_allowlist/up.sql:6`), but r1 dropped `community_id` to nullable (`2026-05-10-000100-0000_extend_sponsor_allowlist_for_r1/up.sql:11`) WITHOUT a partial unique index for the NULL case. Postgres treats NULLs as distinct in unique constraints → `add` can insert unlimited duplicate instance-wide rows (`community_id IS NULL`) for the same person. `remove` deletes one (`.first`), and the governance log double-emits `sponsor_allowlist_added`.

**Fix (in-handler, NO migration):** Before the insert, check existence and reject duplicates. Add `sponsor_allowlist_exists` to the existing import at `:28-30`, then guard inside the closure before `sponsor_allowlist_insert`:

Import change (`:28-30`) — add `sponsor_allowlist_exists` to the list:
```rust
use lemmy_db_schema::source::governance::sponsor_allowlist::{
  SponsorAllowlist, SponsorAllowlistInsertForm, sponsor_allowlist_delete, sponsor_allowlist_exists,
  sponsor_allowlist_insert,
};
```

Guard inside the `run_transaction` closure, immediately before `let form = ...` at `:69`:
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

**Convention precedent for the "already exists" error:** `admin_rule_sets` uses `LemmyErrorType::Unknown("rule_set_version already exists ...")` for a duplicate (see e2e.rs:7263). `Unknown` is the established convention for "duplicate" in this codebase — use it here. (This is NOT the same as cr-4, which is about a "not found" path that should be `NotFound`.)

---

### 2.3 cr-3 (MAJOR) — scrub the user-provided note before logging (ADR-015)

**File:** `crates/api/api/src/governance/admin_sponsor_allowlist.rs`, the `add` handler payload at **`:77-84`**.

**Problem (verified):** Line 82 puts the raw user-provided `note` into the governance-log payload. The `.coderabbit.yaml:108-110` path-instruction + ADR-015 require every user-visible string in a log payload to pass `scrub()`/redaction before leaving the handler. The `note` is admin-supplied free text and is NOT scrubbed.

**Fix:** Wrap the whole payload in `scrub_json(...)` before passing to `governance_log::append`. `scrub_json` recursively scrubs every string value (keys preserved). Add the import and apply it.

Add to the imports (top of file, after the existing `use crate::governance::{...}` block at `:16-19`) — import `scrub_json` from the redaction shim:
```rust
use crate::governance::redaction::scrub_json;
```

Apply at the append call (`:86-92`) — scrub the payload:
```rust
      governance_log::append(
        &mut conn.into(),
        ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED,
        scrub_json(&payload),
        Some(admin_pseudonym.clone()),
      )
      .await?;
```

**Idiom precedent (read before writing):** `crates/api/api/src/governance/admin_rule_sets.rs:52` imports `redaction::scrub`; `:320` applies `scrub(&rsv.rule_text)`. `scrub`/`scrub_json` are re-exported from `crates/api/api/src/governance/redaction.rs:22`. The `scrub_json` definition is at `crates/db_schema/src/source/governance/redaction.rs:88`. **`scrub_json` keeps object keys + non-string scalars intact** — so `allowlist_id` (int), `added_at` (timestamp), and the pseudonyms (already safe) are unaffected; only the free-text `note` value is scrubbed.

**Also apply the same fix to `remove`'s payload (`:159-165`)** for symmetry — wrap with `scrub_json(&payload)` at its `append` call (`:167-173`). The remove payload has no user free-text today, but scrubbing the payload uniformly matches the add path and is harmless (pseudonyms + ints pass through unchanged).

---

### 2.4 cr-4 (MEDIUM) — remove() returns NotFound for a missing row

**File:** `crates/api/api/src/governance/admin_sponsor_allowlist.rs`, the `remove` handler row-resolution at **`:155`**.

**Problem (verified):** Line 155 returns `LemmyErrorType::Unknown("sponsor_allowlist row not found")` when the row is absent. `Unknown` renders as an opaque 500; `NotFound` is the right 404-class error and is used everywhere else (incl. the `create_endorsement` strategy arms).

**Fix:** Replace the `.ok_or_else(...)` at `:155`:
```rust
      .ok_or(LemmyErrorType::NotFound)?;
```
(Drop the string message — `NotFound` is a unit variant; `.ok_or(LemmyErrorType::NotFound)?` works because `LemmyErrorType: Into<LemmyError>`.)

---

### 2.5 cr-7 (LOW) — sponsor_allowlist_exists should not materialize the row

**File:** `crates/db_schema/src/source/governance/sponsor_allowlist.rs`, the `sponsor_allowlist_exists` fn at **`:75-102`**.

**Problem (verified):** Lines 86/95 do `.first::<SponsorAllowlist>(conn).await.optional()?.is_some()` — selects every column of the row just to discard it, on the endorsement hot path.

**Fix:** Use a `SELECT EXISTS(...)` query that returns a bool without materializing the row. Replace the two match-arm bodies:
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
The existing `use ... QueryDsl, OptionalExtension, SelectableHelper` at `:4` may now leave `OptionalExtension`/`SelectableHelper` unused IN THIS FN — but they may still be used by `sponsor_allowlist_insert`/`_delete` above. **Run `cargo check` and only remove an import if the compiler flags it unused** (per the pre-push gate §2.6). Do NOT pre-emptively delete imports.

---

### 2.6 cr-5 (MEDIUM) — e2e deny-tests must assert the specific NotFound shape

**File:** `crates/server/tests/e2e.rs`. THREE one-line edits at the deny-test assertions. **This is an e2e edit — pre-locate every anchor verbatim before editing (file is ~18k lines; anchor drift hangs the editor).**

**MANDATORY pre-locate step** (per `feedback_fix_impl_pre_locate_e2e_anchors.md`):
```bash
grep -n 'assert!(result.is_err()' crates/server/tests/e2e.rs | awk -F: '$1 > 18095'
```
Confirm exactly these three lines (advisor located them at `8abec2d1f`; they may have shifted by the 2 advisor commits since — RE-LOCATE, do not trust these numbers blindly):
- `:18207` — `assert!(result.is_err(), "age_or_surety: fresh sponsor with no surety should be denied");`
- `:18273` — `assert!(result.is_err(), "reputation: sponsor with no snapshot should be denied");`
- `:18333` — `assert!(result.is_err(), "allowlist: sponsor not on allowlist should be denied");`

**Problem (verified):** All three gate-denial arms return `LemmyErrorType::NotFound`, but the tests `assert!(result.is_err())` which passes for ANY error (e.g. a DB setup failure) — a wrong-error regression would not be caught.

**Fix:** For each of the three, change the assertion to check the specific `NotFound` shape. Use each line's existing message verbatim. Pattern (precedent: `crates/api/api/src/federation/resolve_object.rs:185` `assert!(res.is_err_and(|e| e.error_type == LemmyErrorType::NotFound))`):

```rust
    assert!(
      result.is_err_and(|e| e.error_type == LemmyErrorType::NotFound),
      "age_or_surety: fresh sponsor with no surety should be denied with NotFound"
    );
```
…and analogously for the `reputation:` (`:18273`) and `allowlist:` (`:18333`) lines, each keeping its own subject in the message.

**Import check:** `LemmyErrorType` must be in scope in the `v1_rt_r4_fixtures` module. `grep -n "use lemmy_utils::error" crates/server/tests/e2e.rs` near the module header (~`:18097`); if `LemmyErrorType` is not imported in this module's `use` block, add it (other fixtures modules import it — mirror the sibling). Each edit is ONE assertion; do NOT touch the surrounding test bodies.

**DO NOT** add the cr-6/F6 row-deletion assertion to the remove round-trip test in this brief — that was bundled conceptually with cr-5 but is a separate, larger test edit; leave the round-trip test as-is (it's carry-forward).

---

### 2.7 Validate (validate-pending-laptop-e2e)

Run BEFORE committing, to confirm test-compile + workspace check:
```bash
cargo check --workspace --features full
cargo test --workspace --test e2e --no-run --features full
```
Both must exit 0. Push only on exit 0.

Then write a `kind: "validate-pending-laptop-e2e"` DQ entry (id via `bash scripts/brehon/dq-v3-new-entry.sh`; append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`):
```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/validate-fix1-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/validate-fix1-e2e.log || echo E2E_EXIT_NONZERO >> .claude/validate-fix1-e2e.log"'
branch: <your worker branch>
phase_task: fix-impl-1
```
Commit + push the DQ entry mid-task (separate commit from the impl). Do NOT run the full e2e yourself — the laptop runs it.

## 3. Required reading

### 3.1 The findings + verdicts
`.claude/PRPs/reviews/pr-164-findings.yaml` — the advisor's per-finding verification notes (cr-1..cr-12). Read the `notes:`/`rationale:` for cr-1, cr-2, cr-3, cr-4, cr-5, cr-7 — they cite the exact source lines.

### 3.2 MIRROR refs (read verbatim before authoring)
- cr-1 error-matching: `crates/api/api/src/governance/admin_rule_sets.rs:73` (`Err(e) if matches!(e.error_type, ...)`)
- cr-3 scrub: `crates/api/api/src/governance/admin_rule_sets.rs:52,320` + `crates/api/api/src/governance/redaction.rs:22` (the shim re-export) + `crates/db_schema/src/source/governance/redaction.rs:88` (`scrub_json` def)
- cr-5 e2e assert: `crates/api/api/src/federation/resolve_object.rs:185` + `crates/server/tests/e2e.rs:7380-7385` (match on `error_type`)
- cr-7 exists query: standard Diesel `select(exists(...))` — no Lemmy-specific gotcha.

### 3.3 Mandatory lessons (e2e.rs edit + LemmyError)
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate verbatim anchors BEFORE editing e2e.rs (file is ~18k lines). **HARD.**
- `feedback_lemmy_error_no_std_error.md` — Case A shape (the `v1_rt_r4_fixtures` module uses `LemmyResult<()>` outer + `?`; the cr-5 edits stay inside that — do NOT introduce `Box<dyn Error>`).
- `feedback_async_pool_test_pattern.md` — pool/conn pattern (you are NOT changing setup, only assertions — but read so you don't disturb it).
- `feedback_clippy_rerun_after_fix.md` + `feedback_clippy_test_style.md` — after the cr-7 import change, re-run clippy; the workspace is `-D warnings`.

## 4. Constraints

- **File-ownership:** modify ONLY the 4 files in §2 + the DQ validate-pending entry. No DTO, route, mod.rs, migration, or Cargo edits.
- **Verified recipes are the contract.** Apply the §2.1-2.6 recipes as written. If a recipe does not compile as given, STOP and raise a `kind: "blocker"` DQ citing the exact compile error — do NOT improvise an alternative (per circuit-breaker.md: 2 retries then escalate).
- **cr-3 scrub:** `scrub_json` MUST be the redaction shim re-export (`crates/api/api/src/governance/redaction.rs`), NOT a hand-rolled scrubber. The helper exists and is reachable — confirmed by advisor.
- **Pre-locate e2e anchors:** re-run the grep in §2.6; use verbatim `old_string` from the ACTUAL current lines (they may have shifted ±2 from the 2 advisor commits since `8abec2d1f`). Never guess line numbers.
- **Pre-push gate:** `cargo check --workspace --features full` AND `cargo test --workspace --test e2e --no-run --features full` both exit 0 before committing (per `feedback_fix_impl_pre_push_cargo_check.md`). Non-zero → fix inline if in-scope, else blocker DQ.
- **DQ mid-task push:** commit + push the validate-pending-laptop-e2e entry on your worker branch so the advisor sees it.
- **Commit subject:** `fix(rt-r4): CR #164 fix-in-PR — sponsor-gate error handling, dup-guard, scrub, NotFound, e2e asserts (cr-1..cr-5,cr-7)`
- **Commit body:** list each cr-N fixed + the one-line recipe applied, and note the HANDOVER trailer is N/A (single fix commit, no downstream cohort).

## 3a. Handover from prior tasks

```yaml
prior_cohort_tasks: []   # fix-impl on an already-shipped phase; the 7 impl tasks + verify are on the phase branch. No cohort handover.
context:
  phase_branch_tip: 15e1e371b   # includes the 2 advisor-mechanical fixes (cr-10 DQ, cr-11 runlog) already landed
  pr: 164
  all_recipes_verified_against_source_by_advisor: true
```
