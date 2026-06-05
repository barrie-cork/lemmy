# Findings: `e2e.rs` corruption from botched BUG-1 cherry-pick (`6f4947b48`)

**Author:** advisor session (m2-core-hook lane)
**Date:** 2026-06-05
**Status:** DIAGNOSED — fixes designed + compiler-verified, NOT yet applied (held for dev-team plan)
**Trigger:** m2-core-hook Task 8 e2e validation failed to compile; investigation traced root cause to a pre-existing trunk defect.

---

## TL;DR

A **botched cherry-pick** (`6f4947b48`, "fix(governance): post/comment authors are first-class defendants (BUG-1)", 2026-06-05 13:43) re-injected ~2,100 lines of already-extracted test content back into `crates/server/tests/e2e.rs`. This produced **three distinct breakages** that make the `e2e` test binary uncompilable. The defect is on **`governance-v0` trunk** (and inherited by `phase-m2-core-hook`) and has blocked **all** e2e compilation since 2026-06-05 13:43. A **separate, unrelated Task 8 code bug** (`CaseStatus::Active` — non-existent enum variant) was surfaced by the same failed compile.

**Net: 11 compile errors = 9 × E0428 (corruption) + 2 × E0599 (Task 8 bug).**

The original advisor fix (delete `e2e.rs` lines 110-893) addressed only breakage #1. Breakages #2 and #3 + the Task 8 bug remain.

---

## How it happened (git archaeology)

The `e2e.rs` file underwent a **decomposition refactor** in early June: a monolithic ~18,900-line file was split, moving fixtures + tests into `crates/server/tests/e2e/{common/mod.rs,governance.rs,admin_config.rs,jury_mechanics.rs,sponsor_liability.rs}` and reducing `e2e.rs` to a **156-line host** that pulls them back via `include!`.

Timeline:

| Commit | Date | `e2e.rs` state | Braces |
|---|---|---|---|
| `91abf7258` (original BUG-1) | 2026-06-01 20:34 | monolithic 18,911 lines | balanced |
| `5dfcb00b5` (extract common) | — | 17,208 lines | balanced |
| `f2a836ebb` (extract governance) | — | 12,044 lines | balanced |
| `611f0157c` (extract admin_config) | — | 8,188 lines | balanced |
| `0f3531c81` (extract jury+sponsor) | — | **156 lines** ← LAST GOOD | balanced |
| **`6f4947b48` (BUG-1 cherry-pick)** | **2026-06-05 13:43** | **2,256 lines** ← CORRUPT | **delta −1** |

The closeout e2e validation that "passed" (`3a5c9570d`, 2026-06-05 **00:10**) ran **before** the corrupting cherry-pick (13:43) — which is why no one caught it. **e2e has never compiled on trunk since 13:43.**

**Mechanism:** `6f4947b48` is a cherry-pick of the BUG-1 change. The original BUG-1 (`91abf7258`) edited the *monolithic* `e2e.rs`. By the time it was cherry-picked, `e2e.rs` had been decomposed to 156 lines. The 3-way merge hit conflicts and the resolution **pasted ~2,100 lines of pre-decomposition content back into `e2e.rs`** — content that already lives in the `include!`d children. Crucially, the re-injected fixtures block was pasted **without its `mod governance_fixtures {` opening line**, leaving a dangling `}`.

`git log 0f3531c81..HEAD -- crates/server/tests/e2e.rs` returns **only** `6f4947b48`. Every `+async fn` in that diff is a duplicate of something in `governance.rs`. **There are zero legitimate changes to `e2e.rs` since `0f3531c81`.**

---

## The three corruption breakages

### Breakage #1 — duplicate `governance_fixtures` module (delimiter error) — ALREADY FIXED
- `e2e.rs` lines **110-893** were a verbatim duplicate of `crates/server/tests/e2e/common/mod.rs` lines 52-834 (the `governance_fixtures` module: `schema_sentinel_satisfied`, `apply_all_schema`, `pg_template`, `start_postgres`, `seed_user`, `seed_community`, `seed_jurors`, etc. — identical 12-item inventory).
- Pasted **without** the `mod governance_fixtures {` opener → dangling `}` at line 893 → `error: unexpected closing delimiter`.
- The module already reaches `e2e.rs` legitimately via `mod common; use common::governance_fixtures;`.
- **Advisor already deleted lines 110-893** on `phase-m2-core-hook` (uncommitted working-tree change). This is **superseded by fix #2** (full restore is cleaner).

### Breakage #2 — duplicate test fns + helpers + const (E0428 ×9) — NOT FIXED
After #1, the file still contained re-injected **test functions** (not just fixtures) that duplicate `governance.rs`. Because `include!("e2e/governance.rs")` expands at crate scope, these are genuine duplicate definitions → `E0428: defined multiple times`:

| Definition | In `e2e.rs` (post-#1) | Also in `governance.rs` |
|---|---|---|
| `MIGRATIONS_TO_REVERT_PHASE_1` (const) | ~line 200 | line 202 |
| `can_insert_moderation_case` | 112 | 2 |
| `governance_log_hash_chain_holds` | 156 | 46 |
| `assert_revert_list_matches_disk` (helper) | 364 | 251 |
| `phase1_revert_list_matches_disk` | 413 | 300 |
| `test_phase1_migrations_forward` | 423 | 310 |
| `test_phase1_migrations_revert` | 739 | 626 |
| `test_phase1_migrations_reapply` | 1075 | 962 |
| `v1_jm_a_backfill_populates_v0_snapshot` | 1148 | 1035 |

Only `postgres_container_boots` (45) and `template_dump_capture` (80) in `e2e.rs` are **legitimate** (they exist *only* in `e2e.rs`, by design).

### Breakage #3 — BUG-1's migration missing from the canonical revert list — NOT FIXED
BUG-1's revert-list update landed in the **dead duplicate** `e2e.rs` block, never reaching the canonical `governance.rs`. As a result:
- `governance.rs` `MIGRATIONS_TO_REVERT_PHASE_1` has **20 entries**, newest = `2026-06-03_add_governance_messaging_config` (M1-b).
- It is **missing** `2026-06-01-000000-0000_backfill_author_defendant` (BUG-1's own migration, which IS on disk).
- The disk newest-21 window is contiguous and includes BOTH. The current list skips index-1.
- `phase1_revert_list_matches_disk` is a plain `#[test]` that runs by default and asserts `MIGRATIONS_TO_REVERT_PHASE_1 == dirs[:len]` (newest-first). **It will fail** even after #2 is fixed.

---

## The separate Task 8 code bug (NOT corruption)

### E0599 ×2 — `CaseStatus::Active` does not exist
- Task 8's worker wrote `m2_hook_suppressed_when_messaging_disabled` (in `crates/server/tests/e2e/governance.rs`, ~lines 5364 and 5377) using `CaseStatus::Active`.
- The real `CaseStatus` enum (`crates/db_schema_file/src/enums.rs:393`) has variants: `Open` (default), `ThresholdMet`, `JurySelection`, `InReview`, `Decided`, `Appealed`, `Closed`, `EmergencyRemove`, `AdminReview`, `SponsorLiabilityPending`, `SponsorLiabilityFired`, `SponsorLiabilityEscaped`. **No `Active`.**
- Usage context: the test inserts a `ModerationCaseInsertForm { status: CaseStatus::Active, .. }` then calls `governance_case_after_transition(&ctx, &case, Some(CaseStatus::Active), CaseStatus::ThresholdMet)`. It just needs a valid pre-transition status; `Open` is the natural case-opening state.
- This is a Task 8 defect (worker invented a variant); it is **independent** of the cherry-pick corruption. It would have failed Task 8's e2e regardless.

---

## Proposed fixes (compiler-verified, all on `phase-m2-core-hook`)

| # | Breakage | Fix | File | Verification |
|---|---|---|---|---|
| 1 | dup fixtures module | superseded by #2 | `e2e.rs` | — |
| 2 | dup test fns/helpers/const (E0428 ×9) | `git checkout 0f3531c81 -- crates/server/tests/e2e.rs` (clean 156-line restore) | `e2e.rs` | `0f3531c81` confirmed ancestor of HEAD; its `e2e.rs` has 0 of the duplicated defs; only the 2 legit smoke tests |
| 3 | BUG-1 migration missing from revert list | insert `"2026-06-01-000000-0000_backfill_author_defendant",` at index 1 of `MIGRATIONS_TO_REVERT_PHASE_1` (list 20→21); update the M1-b count comment | `crates/server/tests/e2e/governance.rs` (~line 204) | disk newest-21 computed = list-with-insert; contiguous, no gaps |
| 4 | Task 8 `CaseStatus::Active` (E0599 ×2) | `CaseStatus::Active` → `CaseStatus::Open` at 2 sites | `crates/server/tests/e2e/governance.rs` (~5364, 5377) | real enum read from `enums.rs:393`; `Open` is `#[default]` open state |

**Fix #2 caveat:** restore ONLY `e2e.rs` to `0f3531c81`. Do **NOT** restore `governance.rs` or any other file to that SHA — `governance.rs` has legitimate later changes (`7011014c3` BUG-1 emergency-remove test, `66b80d1ad`/Task 6 hook wiring, `667e0c54d` Task 8 tests, M1-b list update). Fixes #3 and #4 are surgical edits to the *current* `governance.rs`.

**Re fix #4 + four-role model:** the advisor must not author `crates/**`. The clean-architecture option is to re-dispatch the `CaseStatus::Active`→`Open` fix as a Junior `fix-impl-task` rather than fix it inline. (User to decide.)

---

## What is NOT damaged (investigation vectors cleared)

- **Vector 1 — other files `6f4947b48` touched:** `admin_emergency_remove.rs`, `submit_jury_vote.rs`, `create_report.rs` have **no duplicate fns, balanced delimiters**. Their BUG-1 source diffs (24/8/12 lines) applied **cleanly** — diff vs original BUG-1 `91abf7258` is 0 lines for 2 of them (the 3rd's 74-line delta is legit Task-6/Phase-7 evolution). **BUG-1 production logic is present and correct** (`target_person_id` from `creator_id`: 13 refs in create_report, 2 in admin_emergency_remove).
- **Vector 1 — migration files:** `backfill_author_defendant/{up,down}.sql` balanced (16/16, 9/9 parens), present on disk.
- **Vector 2 — other botched merges:** recent trunk merge lineage (last 40) shows only standard PR/dependabot/feature merges. No other large-additive-diff-to-restructured-file signature.
- **Vector 3 — other `include!` scopes:** `include!` is used **only** in `e2e.rs`. (Other `*_include!` hits are the unrelated `assert_json_include!` macro.) The include!-scope duplication hazard exists nowhere else.
- **Full `crates/` delimiter scan** (braces+parens+brackets, strings/comments stripped): **zero imbalances** repo-wide (after the #1 partial fix).
- **Within-file repeated fn names** in `governance.rs` (`seed_person` ×6) and `sponsor_liability.rs` (`count_log_entries`/`read_log_payload`/`seed_pending_case` ×3): these are **legitimate** per-test nested fixture modules (indented, module-scoped) — the compiler did NOT flag them (only the 9 cross-scope ones). Not damage.

---

## Blast radius / process notes for the dev team

1. **Trunk is e2e-uncompilable.** `governance-v0` HEAD carries breakages #1-3. Any lane attempting e2e since 2026-06-05 13:43 hits this. The m2 fix (on `phase-m2-core-hook`) will propagate to trunk only at m2 PR merge — consider whether a **direct trunk hotfix** is warranted given other lanes may need e2e sooner.
2. **The "passing" closeout e2e was pre-corruption** — do not treat the closeout retro's e2e-green as evidence the current trunk compiles.
3. **Cherry-pick-onto-restructured-file is the root hazard.** When a fix authored against a monolithic file is cherry-picked after that file is decomposed, the 3-way merge can silently re-inject pre-decomposition content. Mitigation candidates: (a) re-author the fix against the new structure instead of cherry-picking; (b) post-cherry-pick duplicate-definition scan; (c) a CI `cargo test --no-run --workspace --test e2e` gate on trunk (the corruption would have been caught immediately).
4. **False-green wrapper exit code:** the background `cargo-test.bat && echo EXIT_0 || echo EXIT_NONZERO` pattern reported task "exit code 0" (the `echo` succeeded) while cargo actually exited 101. The in-log marker (`E2E_T8_V2_EXIT_NONZERO`) is the truth, per `.claude/rules/cargo-output-capture.md`. Trust the marker, not the task notification.
5. **Detection tooling:** a duplicate-fn scan across the `include!` scope (`e2e.rs` + the 4 `include!`d files) is the fastest signal for this corruption class. Script logic: regex `^\s*(?:pub...)?(?:async )?fn (\w+)` across the file set, flag names appearing in 2+ files.

---

## Reproduction / verification commands

```bash
# Confirm the duplicate-definition corruption (run from a phase-m2-core-hook worktree):
grep -nE "^async fn " crates/server/tests/e2e.rs                      # 8 fns; only first 2 are legit
grep -nE "^#\[tokio::test\]" crates/server/tests/e2e/governance.rs    # the canonical home

# Confirm the last-good restore target:
git log --oneline 0f3531c81..HEAD -- crates/server/tests/e2e.rs       # only 6f4947b48
git show 0f3531c81:crates/server/tests/e2e.rs | grep -cE "can_insert_moderation_case|MIGRATIONS_TO_REVERT"  # 0

# Confirm the revert-list drift (#3):
#   list newest-2 = [messaging_config 06-03, federation_inbound 05-17]
#   disk newest-2 = [messaging_config 06-03, backfill_author_defendant 06-01]   <- list missing 06-01
ls -d migrations/2026-0[56]* | sort | tail -3

# Confirm the Task 8 bug (#4):
grep -n "CaseStatus::Active" crates/server/tests/e2e/governance.rs    # 2 hits (~5364, 5377); Active is not a variant
sed -n '393,420p' crates/db_schema_file/src/enums.rs                  # the real variants

# After fixes, re-validate (from phase-m2-core-hook worktree, Docker up):
cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e m2_ > %LOCALAPPDATA%\Temp\m2-e2e.log 2>&1 && echo EXIT_0 >> %LOCALAPPDATA%\Temp\m2-e2e.log || echo EXIT_NONZERO >> %LOCALAPPDATA%\Temp\m2-e2e.log"
# trust the in-log marker, not the task-notification exit code
```

---

## Current working-tree state (m2-core-hook lane, worktree `brehon-fork-m2`)

- `crates/server/tests/e2e.rs` — **pristine at HEAD (corrupt)**. The advisor's partial fix #1 (delete lines 110-893) was applied then **reverted** so the dev team starts from the real corrupted state. Apply fix #2 cleanly: `git checkout 0f3531c81 -- crates/server/tests/e2e.rs`.
- DQ `1dcd6a201003-002` (`validate-pending-laptop-e2e`, pending) — Task 8's e2e gate; stays pending until the e2e suite compiles + passes.
- `governance-v0` local tip `40e64a459` (Task-8 DQ merge) — **not yet pushed** to origin.
- No other uncommitted changes.
