---
phase: v1-federation-inbound-a
role: impl-task
kind: fix-impl
fix_impl_n: 5
authored: 2026-05-18
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
triggering_dq: 267
triggering_task: 9
classification: "NON-ALLOWLIST Phase-2 e2e regression, USER-AUTHORISED fix-forward (Option A via advisor AskUserQuestion catch-fire surface 2026-05-18). NOT a §G4 mechanical recipe — hand-authored. Phase-2 e2e for tip 8d78381d9 failed 1/91: v1_jm_a_backfill_populates_v0_snapshot @ e2e.rs:2021 — assertion `pg_type 'severity_tier' should be absent between revert and re-apply` (left:1 right:0). Root cause: this phase's federation migration (2026-05-17-000000-0000_add_federation_inbound_v1) has a down.sql that does NOT EXACTLY invert its up.sql; the residual dump-diff aborts the LIFO revert chain BEFORE it reaches the JM-a migration that drops severity_tier (feedback_phase1_migration_count_lifo class). The federation deliverable runtime is correct (both v1_federation_inbound_a_fixtures tests pass; 90/91 green) — the defect is purely migration-revert symmetry. The failing guard (PR #92 cr-5, e2e.rs:2021) is ACTIVE (NOT #[ignore]d) and doing its job. User chose Option A: correct the federation down.sql so the round-trip passes; keep the active guard active."
base: "phase-v1-federation-inbound-a @ 8d78381d9 (Cohort B 4/4 §5.2-VALIDATED-PASS: T6/T7/T8/T9 DQ #248/#249/#251/#252/#266; lane==origin==daemon-local synced; DQ #267 Phase-2 e2e validate-pending raised, result:null)"
cap: "<=3 file edits, confined to: (1) migrations/2026-05-17-000000-0000_add_federation_inbound_v1/down.sql (the primary fix), and AT MOST (2) migrations/2026-05-17-000000-0000_add_federation_inbound_v1/up.sql (ONLY if diff_check proves an up.sql object is unrevertable as written — e.g. a DEFAULT that must be added explicitly so down.sql can mirror it; do NOT change up.sql DDL semantics), and AT MOST (3) one e2e.rs revert-list helper IF AND ONLY IF it carries an explicit hardcoded table/type allowlist that must gain the new federation objects (grep first; if no such explicit list exists, do NOT touch e2e.rs). NEVER edit crates/** Rust, NEVER edit schema_setup/mod.rs, NEVER add #[ignore] to any test."
serial: "Cohort B strictly serial cap=1. This fix-impl repairs the federation migration revert symmetry so the active guard v1_jm_a_backfill_populates_v0_snapshot passes in a full Phase-2 e2e run. Phase-2 e2e (DQ #267) is re-run by the ADVISOR on the laptop AFTER this fix lands + finalize-merge; the phase advances to bm-pr only on DQ #267 result:pass."
---

# [role:impl-task] v1-federation-inbound-a fix-impl-5 — federation down.sql revert-symmetry (Option A) — see .claude/PRPs/briefs/federation-inbound-a-fix-impl-5.md

> **Provenance:** Cohort B is 4/4 §5.2-VALIDATED-PASS (cargo check / clippy / test --no-run all green for Tasks 6-9; Task 9's fix-impl-4 correctly mirrored the canonical sibling). The **Phase-2 e2e** run for phase tip `8d78381d9` (full `cargo test --workspace --test e2e --features full`, 1988.55s) **failed 1/91**: `v1_jm_a_backfill_populates_v0_snapshot` panicked at `crates/server/tests/e2e.rs:2021` — `assertion 'left == right' failed: pg_type 'severity_tier' should be absent between revert and re-apply (left: 1, right: 0)`. This test is **ACTIVE** (only `#[tokio::test]` at e2e.rs:1910 — NO `#[ignore]`; the GH#43 `#[ignore]` markers at e2e.rs:1200/1507/1835 are on OTHER round-trip tests). The failing assertion is the **PR #92 cr-5 guard** (e2e.rs:2012-2024) that asserts the 4 JM-a PG enum types are absent between `schema_setup::revert()` and re-apply — written to catch *"a revert() that drops a table but leaves its backing enum dangling"*. `severity_tier` (a JM-a enum) surviving the revert means the federation migration's `down.sql` does NOT exactly invert its `up.sql`; the residual dump-diff aborts the LIFO revert chain before it can reach the JM-a migration that drops `severity_tier` (`feedback_phase1_migration_count_lifo` — migrations revert LIFO; a later migration whose down.sql is asymmetric blocks the chain reverting past it). **The federation deliverable runtime is correct** (both `v1_federation_inbound_a_fixtures` tests pass; 90/91 e2e green) — the defect is purely revert symmetry. **User chose Option A** (advisor AskUserQuestion catch-fire surface 2026-05-18): correct the federation `down.sql` so the round-trip passes; the active guard stays active (no `#[ignore]`, no known schema-revert hole). Classification: **non-allowlist, user-authorised fix-forward** — hand-authored recipe (NOT a §G4 mechanical paste).

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD branch is a Junior worktree branched off `phase-v1-federation-inbound-a` (base tip `8d78381d9`). `git merge-base --is-ancestor 8d78381d9 HEAD` MUST be true. If on `phase-v1-federation-inbound-a` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ **into `.claude/decision-queue.json`** (NOT a repo-root file — see §4 Constraint 3).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — if inside a forbidden window exit non-zero `FORBIDDEN_WINDOW: <window>`, UNLESS the dispatch description carries `(user-authorised forbidden-window override per DQ #<id>)`. Shape G SUSPENDED (cargo on laptop) — keep the check.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** run `git submodule update --init 2>&1` from the worktree root. `crates/email/translations` is NOT auto-initialized in a fresh worktree; without it cargo fails pre-existing (ENOENT on `translations/backend/`). Infra, NOT part of the fix.
- Confirm the federation migration dir is present + unmodified at base: `ls migrations/2026-05-17-000000-0000_add_federation_inbound_v1/` MUST list `up.sql` + `down.sql`. `git log -1 --format=%H -- migrations/2026-05-17-000000-0000_add_federation_inbound_v1/` should resolve within this phase's history. If the dir is absent or the migration name differs → STOP, file `kind: "blocker"` DQ into `.claude/decision-queue.json` (base mismatch).
- Confirm the guard is active (sanity): `grep -n "fn v1_jm_a_backfill_populates_v0_snapshot" crates/server/tests/e2e.rs` returns ~1910 AND the 3 lines above it are NOT `#[ignore`. If it shows `#[ignore]` → STOP, file `kind: "blocker"` (premise changed — the test was demoted by someone else; do not proceed with a fix predicated on it being active).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a fix-impl-5 — federation down.sql revert-symmetry (Option A)`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-a fix-impl-5 — see .claude/PRPs/briefs/federation-inbound-a-fix-impl-5.md
```

## §2 Scope

### 2.1 The failure being fixed (the contract)

Full Phase-2 e2e run for phase tip `8d78381d9`:

```
test result: FAILED. 90 passed; 1 failed; 5 ignored; 0 measured; 0 filtered out; finished in 1988.55s

thread 'v1_jm_a_backfill_populates_v0_snapshot' (9676) panicked at crates\server\tests\e2e.rs:2021:7:
assertion `left == right` failed: pg_type 'severity_tier' should be absent between revert and re-apply
  left: 1
 right: 0
```

The 5 "ignored" are the known GH#43 deferrals (`test_phase1_migrations_revert`/`_reapply`/round-trip-step-3). The 1 failure is the **active** PR#92-cr-5 guard `v1_jm_a_backfill_populates_v0_snapshot` (e2e.rs:1910, NO `#[ignore]`), asserting at e2e.rs:2012-2024 that `severity_tier`/`case_status_tier`/`jury_assignment_role`/`jury_constraint_relaxation_reason` are absent between `schema_setup::revert()` and re-apply.

### 2.2 Root cause (the diagnosis you must confirm, then fix)

`schema_setup` (crates/diesel_utils/src/schema_setup/mod.rs) reverts migrations **LIFO** (`revert_all_migrations`, mod.rs:302). The test forward-applies all migrations, then `revert()`s all, then re-applies, asserting the JM-a enums are absent in the reverted state. `severity_tier` count=1 (should be 0) in the reverted state means the LIFO chain **did not successfully revert past the federation migration** to the JM-a migration that `DROP TYPE severity_tier`. The federation migration `2026-05-17-000000-0000_add_federation_inbound_v1/down.sql` is **not an exact inverse** of its `up.sql` — a residual object/dump-diff makes the federation revert step incomplete, and the LIFO chain stops there (`feedback_phase1_migration_count_lifo`).

**The authoritative diagnosis tool is already in the codebase.** `schema_setup/mod.rs:84-100` runs, under `#[cfg(test)] enable_diff_check`, a per-migration `run → revert → diff_check::check_dump_diff` that prints exactly:

```
These changes need to be applied in migrations/2026-05-17-000000-0000_add_federation_inbound_v1/down.sql:
<the precise dump delta>
```

The fix worker MUST obtain that exact delta (do NOT guess from static up/down reading alone) and make `down.sql` mirror up.sql so the delta is empty.

### 2.3 The fix (hand-authored recipe — implement against the diff_check delta, not a static guess)

1. **Reproduce + capture the exact delta.** Run the migration round-trip diff_check locally in the worktree. The cheapest path is the dedicated round-trip test that exercises `enable_diff_check`:
   ```
   cmd //c "scripts\brehon\cargo-test.bat --workspace --features full --test e2e -- --ignored test_phase1_migrations > .claude/PRPs/debug/fi5-diffcheck.log 2>&1 && echo FI5_DIFFCHECK_EXIT_0 >> .claude/PRPs/debug/fi5-diffcheck.log || echo FI5_DIFFCHECK_EXIT_NONZERO >> .claude/PRPs/debug/fi5-diffcheck.log"
   ```
   (The `--ignored` round-trip tests run the per-migration diff_check and will print the `These changes need to be applied in migrations/2026-05-17-.../down.sql:` block for the federation migration if its down.sql is asymmetric. If that test path does not surface it, run the active guard directly: `... --test e2e v1_jm_a_backfill_populates_v0_snapshot` and read the panic + any preceding `check_dump_diff` stderr.) Verify the EXPLICIT `FI5_DIFFCHECK_EXIT_*` marker in the log file — do NOT trust the bg task-notification (it has lied 3× this phase; `feedback_background_task_notification_lies`).
2. **Read the precise delta** printed for `2026-05-17-000000-0000_add_federation_inbound_v1`. Candidate asymmetries to expect (the up.sql at lines 56-135 creates: 2 enum TYPEs, 4 TABLEs, ALTER ADD COLUMN on `remote_sanction_notice` (4 cols incl 2 enum-typed with DEFAULT) and `federation_attestation` (6 cols incl 2 enum-typed, one with DEFAULT), 8 INDEXes, 11 governance_config INSERT rows; the current down.sql at lines 1-38 reverses all of these). Likely real defects:
   - A **column DEFAULT** added in up.sql (e.g. `peer_trust_level_at_receipt federation_peer_trust_enum NOT NULL DEFAULT 'unknown'` on `remote_sanction_notice`) that leaves a dump residue the `DROP COLUMN` does not clear if drop order vs enum-type drop is wrong (drop columns that reference an enum BEFORE `DROP TYPE` — verify the current order does this).
   - An **enum type used by a pre-existing table** (`remote_sanction_notice`/`federation_attestation` are pre-existing Phase-6 tables; up.sql ALTERs them to add `federation_peer_trust_enum`/`federation_inbox_admin_action_enum` columns) — if down.sql `DROP TYPE federation_*_enum` runs while a column still references it (drop-order bug) the revert errors and the chain aborts.
   - The `INSERT ... ON CONFLICT (scope, key, valid_from) DO NOTHING` (up.sql:125-135, 11 rows) vs `DELETE ... WHERE scope='instance' AND key IN (...)` (down.sql:1-13, 11 keys) — verify the key list matches EXACTLY (same 11 keys, same spelling) so the DELETE removes precisely the inserted rows and no dump residue remains.
   - Any **INDEX** in up.sql not dropped in down.sql, or any partial-index `WHERE` predicate that the dump renders differently.
3. **Correct `down.sql`** so it is an exact inverse of `up.sql` (reverse dependency order: drop indexes → drop added columns → drop tables → drop types → delete seed rows, adjusting so no object is dropped while a dependent still references it). The diff_check delta is the contract — when it prints nothing for this migration, the fix is complete. Do NOT weaken up.sql DDL to make down.sql easier unless diff_check proves an up.sql object is fundamentally unrevertable as written (then make the minimal up.sql change that preserves DDL semantics, and note it in the commit body).
4. Do **NOT** touch `crates/server/tests/e2e.rs` UNLESS `grep -n "federation_peer\|federation_inbox\|severity_tier" crates/server/tests/e2e.rs | grep -iE "revert.list|allowlist|\[.*severity_tier.*\]"` reveals an explicit hardcoded revert-list array that must gain the new federation tables/types. The PR#92-cr-5 guard at e2e.rs:2012 only lists the **4 JM-a** enums (it does NOT need federation entries — it asserts JM-a state, and once the LIFO chain completes it will pass unchanged). If there is NO explicit federation revert-list array to extend, e2e.rs stays untouched.

### 2.4 Acceptance (the worker proves these before reporting done)

- The diff_check round-trip prints **no** `These changes need to be applied in migrations/2026-05-17-000000-0000_add_federation_inbound_v1/down.sql:` block.
- `down.sql` is an exact inverse of `up.sql` (every up.sql object has a matching reversal; correct dependency order).
- §4.2 pre-push cargo-check is green (the fix is SQL-only; cargo-check must still pass — no Rust touched).
- The worker does NOT run the full ~33-min e2e suite itself (advisor re-runs Phase-2 e2e on the laptop post-finalize-merge per the resume path). The worker's job is the down.sql correction + the diff_check-empty proof + pre-push cargo-check.

## §3 Required reading (read these IN ORDER before editing)

1. `.claude/lessons/feedback_phase1_migration_count_lifo.md` — **PRIMARY**. LIFO revert chain; a later migration's asymmetric down.sql blocks earlier reverts. This is the exact failure class.
2. `.claude/lessons/feedback_lemmy_migration_runner.md` — Lemmy migration mechanics (forbid_diesel_cli; how down.sql is invoked; `cargo run -p lemmy_diesel_utils` patterns).
3. `crates/diesel_utils/src/schema_setup/mod.rs` lines **80-135 + 274-310** — the `enable_diff_check` run→revert→check_dump_diff loop (mod.rs:84-100) and `revert_all_migrations` LIFO (mod.rs:296-310). This is the mechanism + the authoritative diagnosis tool.
4. `crates/server/tests/e2e.rs` lines **1900-2090** — the guard `v1_jm_a_backfill_populates_v0_snapshot` and the PR#92-cr-5 assertion block (2012-2024). Read to confirm it is active + understand exactly what it asserts; do NOT edit it (unless §2.3 step 4's grep finds an explicit federation revert-list array).
5. `migrations/2026-05-17-000000-0000_add_federation_inbound_v1/up.sql` + `down.sql` (full) — the artifacts you are correcting.
6. `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` §10 (federation schema) — confirm the intended schema so a down.sql correction does not contradict the plan.

## §4 Constraints (HARD — violation = STOP + kind:blocker DQ)

1. **One commit.** Subject: `fix(v1-federation-inbound-a): federation down.sql exact-inverse of up.sql — unblock LIFO revert chain (fix-impl 5)`. Body: list the precise down.sql corrections + the diff_check-empty proof + (if up.sql touched) why.
2. **Pre-push cargo-check (per `feedback_fix_impl_pre_push_cargo_check`):** before pushing the worker branch, run `cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fi5-precheck.log 2>&1 && echo FI5_PRECHECK_EXIT_0 >> .claude/PRPs/debug/fi5-precheck.log || echo FI5_PRECHECK_EXIT_NONZERO >> ..."`; verify the EXPLICIT marker (not bg notification). Non-zero → STOP, file `kind: "blocker"` DQ (SQL-only change must not break cargo; if it does, something is wrong). NEVER `#[allow]`-spam.
3. **DQ writes go into `.claude/decision-queue.json`** (the canonical lane file at the worktree's `.claude/` path) — NEVER a repo-root file. Compute `next_id = max(all ids across pending+resolved) + 1` from `.claude/decision-queue.json` + any `.claude/decision-queue-archive-*.json`; do NOT pre-pick an id without that scan (the v1-ship-1 collision precedent). Commit + push the DQ entry on the worker branch immediately (mid-task visibility).
4. **File-ownership:** edits ONLY to `migrations/2026-05-17-000000-0000_add_federation_inbound_v1/{down.sql, up.sql}` and AT MOST one e2e.rs revert-list array (per §2.3 step 4 grep gate). NEVER `crates/**` Rust, NEVER `schema_setup/mod.rs`, NEVER `Cargo.*`, NEVER add `#[ignore]` to any test, NEVER edit the plan/ADRs.
5. **MIRROR-ref discipline:** the diff_check delta is the authoritative spec — implement against what it prints, not against this brief's static guesses in §2.3 step 2 (those are candidates to check, not the contract). If the delta contradicts a §2.3 candidate, the delta wins.
6. **Attribution:** worker `from: "impl"`; never write `answered_by: "advisor"|"user"`; never `kind: "clarify"|"validate-result"|"validate-failed"`.
7. **Serial:** Cohort B strictly serial cap=1 — this is the only in-flight Junior for this lane.

## §5 What "done" looks like

One commit on a `junior/*` worktree branch off `8d78381d9` that makes `migrations/2026-05-17-000000-0000_add_federation_inbound_v1/down.sql` an exact inverse of `up.sql`, proven by the diff_check round-trip printing no delta block for this migration, with §4.2 pre-push cargo-check green. The advisor then finalize-merge-reconciles the worker branch lane-safe, re-runs Phase-2 e2e on the laptop, and on green mutates DQ #267 `result: "pass"` → phase advances to `bm-pr-pending`.
