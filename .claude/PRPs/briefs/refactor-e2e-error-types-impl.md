---
phase: chore/refactor-e2e-error-types
role: impl-task
task: refactor
brief_n: 1
authored: 2026-05-14
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_findings: 3.E.1 (CRIT) + 3.E.2 (CRIT) + 3.E.3 (MAJ) + 3.E.4 (MAJ) — ranks 1, 2, 7, 8
parent_phase_tip: <set by bm-cut — branch tip is governance-v0 HEAD at bm-cut time>
---

# [role:impl-task] chore/refactor-e2e-error-types — unify error-type to LemmyResult + consolidate fixtures + split phase1_migrations_round_trip — see .claude/PRPs/briefs/refactor-e2e-error-types-impl.md

## §1 Role + dispatch

`[role:impl-task] chore/refactor-e2e-error-types — full e2e.rs Case-C cleanup + fixtures dedup + round-trip test split — bundled per audit §4 PR-1`

## §2 Scope

### §2.1 Driving audit findings (4 bundled per audit §4 PR-1)

**§3.E.1 [CRIT] — Case C error-type mixing across e2e.rs Brehon-authored test corpus.**
20+ test fns return `Result<(), Box<dyn Error>>`; `governance_fixtures::start_postgres` + helpers return `Box<dyn Error>`; BUT `admin_config_fixtures::bootstrap` (line 5681) AND `governance_fixtures::bootstrap` (line 767) return `LemmyResult<...>`. Mixed shapes across the same file. Adding a single Lemmy-native call to a Box-typed test triggers E0277 cascade per `feedback_lemmy_error_no_std_error.md` Case C hard refusal. Fix: pick Case A (uniform `LemmyResult<()>` outer + `LemmyResult<T>` helpers; mirror v1-SL-b canonical at ~line 11139).

**§3.E.2 [CRIT] — 4 phase-specific fixtures modules with ~70% duplication.**
`v1_jm_b_fixtures` (8029), `v1_jm_e_fixtures` (9948), `v1_sl_b_fixtures` (11139), `v1_sl_c_fixtures` (12086). Each defines variants of `seed_case`, `seed_jurors`, `seed_jury_eligible_snapshots`, `seed_community`, `seed_user`. Extract shared `governance_test_helpers` module; phase-specific fixtures wrap phase-specific assertions only.

**§3.E.3 [MAJ] — `phase1_migrations_round_trip` test fn is 447 lines covering 3 phases.**
Lines 1054-1500. Split into `test_phase1_migrations_forward`, `test_phase1_migrations_revert`, `test_phase1_migrations_reapply` with fresh-Postgres fixtures each.

**§3.E.4 [MAJ] — `PHASE_1_MIGRATION_COUNT` revert is "bookkeeping fiction".**
Inline TODO at ~line 1093 already flags switching to a named-migration list. Replace with explicit `MIGRATIONS_TO_REVERT: &[&str] = &[…]` using migration runner's named API.

### §2.2 Audit-confirmed file shape (verified 2026-05-14)

```
e2e.rs total: 8945 lines (NB: audit had a stale 12000+ line claim; actual is 8945)

Brehon-authored top-level test fns returning Box<dyn Error>:
  20  postgres_container_boots
  55  template_dump_capture
  864 can_insert_moderation_case
  908 governance_log_hash_chain_holds
  1054 phase1_migrations_round_trip
  1518 v1_jm_a_backfill_populates_v0_snapshot
  1803 list_open_cases_returns_seeded_rows
  1866 jury_queue_view_returns_assignments
  1971 modlog_view_returns_published_entries
  2696 config_parity_round_trip
  2770 v1_jm_a_seed_migration_is_idempotent
  2864 sponsor_liability_with_founder_multiplier
  3702 all_mvp_endpoints_return_non_404
  3983 ineligible_user_cannot_be_picked_for_jury
  4318 governance_events_notify_fires
  4460 underscore_prefix_usernames_still_register
  4539 sanction_notice_round_trip
  5197 appeal_inside_window_succeeds_expired_rejects
  5390 declining_juror_not_picked_as_own_replacement

(plus test fns inside each *_fixtures mod, which need similar treatment)

Fixtures modules (4 sub-phase + 2 shared):
  88   mod governance_fixtures        (canonical shared)
  5658 mod admin_config_fixtures      (semi-shared, owns its own bootstrap)
  8029 mod v1_jm_b_fixtures           (phase-specific; ~70% duplicate)
  9948 mod v1_jm_e_fixtures           (phase-specific; ~70% duplicate)
  11139 mod v1_sl_b_fixtures          (phase-specific; ~70% duplicate; CANONICAL Case A shape)
  12086 mod v1_sl_c_fixtures          (phase-specific; ~70% duplicate)

Mixed-shape entry points (Case C source):
  767  governance_fixtures::bootstrap         returns LemmyResult<...>  CANONICAL
  5681 admin_config_fixtures::bootstrap       returns LemmyResult<...>  CANONICAL
  848  governance_fixtures::seed_jurors       (verify shape during read)
  4148 (sub-mod) seed_case                    (verify; likely Box<dyn Error>)
  8053 v1_jm_b_fixtures::seed_case            (per audit, mixed)
  8096 v1_jm_b_fixtures::seed_jury_eligible_snapshots
  ... etc
```

### §2.3 Canonical Case-A shape to mirror (v1-SL-b fixtures at line 11139+)

Read the v1-SL-b fixtures module body in the worker session. The audit identifies this as the canonical Case-A shape:

- Test fns: `async fn <test_name>() -> LemmyResult<()> { ... }`
- Helpers: `async fn <helper_name>(...) -> LemmyResult<T> { ... }`
- Outer signature uniform; no `Box<dyn Error>` anywhere in the module.
- Lemmy-native calls use plain `?`; no `.map_err` bridges needed.
- Test assertions use `assert!`, `assert_eq!`, no `.unwrap()` (per `feedback_clippy_test_style.md`).

### §2.4 The refactor — 4-pass approach

This refactor is L-effort and structural. Worker MUST proceed pass-by-pass, committing locally between passes (final commit is one squashed commit per §6, but local intermediate commits during work-in-progress are fine for git bisect during edit).

**Pass 1: Error-type unification (Case C → Case A, ALL Brehon-authored test fns)**

For EACH Brehon-authored async test fn returning `Result<(), Box<dyn Error>>`:

1. Change signature: `Result<(), Box<dyn Error>>` → `LemmyResult<()>`.
2. Drop the `use std::error::Error;` import if it's no longer referenced.
3. Convert any `.map_err(|e| -> Box<dyn ...> { ... })?` chains to plain `?` if the inner call already returns LemmyResult/LemmyError, OR to `LemmyErrorType::Unknown(format!("...: {e}")).into()` if it doesn't.
4. Run `cargo check --workspace --features full --tests` after each batch of ~5 test fns; do NOT proceed to the next batch if errors remain.

Helpers inside `*_fixtures` modules: same treatment. The 6 modules' bootstrap/seed_* helpers all become `LemmyResult<T>`-returning.

Per `feedback_lemmy_error_no_std_error.md` Case A — type-shape uniformity is mandatory across a single test module. No partial conversion. If you reach end-of-pass-1 with any `Box<dyn Error>` left in Brehon-authored code, the refactor is incomplete.

**Pass 2: Fixtures dedup (consolidate to `governance_test_helpers`)**

The 4 phase-specific modules (`v1_jm_b_fixtures`, `v1_jm_e_fixtures`, `v1_sl_b_fixtures`, `v1_sl_c_fixtures`) each define variants of `seed_case`, `seed_jurors`, `seed_jury_eligible_snapshots`, `seed_community`, `seed_user`. The duplication ~70% per audit; the differences are usually phase-specific assertions OR phase-specific seed-data tweaks.

1. Read each of the 4 modules' `seed_*` helpers in full.
2. Identify the SHARED parameters and SHARED behavior. Extract a canonical helper into `governance_fixtures` (the top-level shared module at line 88) OR a new `governance_test_helpers` module sibling.
3. Phase-specific overrides become thin wrappers OR parameter-driven shape inside the canonical.
4. Each phase-specific `*_fixtures` module retains ONLY:
   - Phase-specific assertions
   - Test fns themselves (or stay in top-level scope, see Pass 3)
   - Re-exports from the canonical helpers as needed

5. If two phase helpers genuinely diverge (not just cosmetic), keep both with distinct names (`seed_case_with_appeal_panel`, `seed_case_simple`). Document the divergence inline.

**Pass 3: Split `phase1_migrations_round_trip` into 3 test fns**

Lines 1054-1500 (~447 lines, single test fn). Per audit §3.E.3, split into:

- `test_phase1_migrations_forward` — fresh Postgres → apply all 18 phase-1 migrations → assert post-forward schema invariants
- `test_phase1_migrations_revert` — fresh Postgres → apply all → revert all → assert clean revert
- `test_phase1_migrations_reapply` — fresh Postgres → apply → revert → re-apply → assert reapply idempotent

Each gets its own `start_postgres` fixture; each is independent. Extract a shared `bootstrap_phase1_apply` helper if the apply-step is identical across all 3.

Naming convention per `feedback_clippy_test_style.md` + Lemmy convention: `test_<area>_<scenario>` snake_case; test fn doc comment explains scenario.

**Pass 4: Replace `PHASE_1_MIGRATION_COUNT` with named-migration list (audit §3.E.4)**

Current at line 1097: `const PHASE_1_MIGRATION_COUNT: u64 = 18;` driving `Options::default().revert().limit(PHASE_1_MIGRATION_COUNT)` at line 1287.

Replace with explicit list of migration directory basenames that should be reverted (in reverse order):

```rust
const MIGRATIONS_TO_REVERT_PHASE_1: &[&str] = &[
  // List the 18 migration directory names, NEWEST FIRST.
  // Source-of-truth: `ls migrations/2026-*/` ordered by directory name desc, top 18.
  "2026-05-10-000300-0000_seed_v1_rt_config_keys",
  // ... 17 more entries ...
];
```

Use the migration runner's named-revert API (check `diesel_migrations` or the workspace's `lemmy_diesel_utils` for the canonical "revert specific migrations by name" call shape). If the runner only supports "revert N" and not "revert by name", document this limitation in an inline comment and KEEP the count-based revert for now BUT comment it explicitly as "bookkeeping fiction until migration runner supports named revert".

Per `feedback_phase1_migration_count_lifo.md` (memory captures this lesson).

### §2.5 Pre-flight worker checklist

Before starting any edits:

1. Read `crates/server/tests/e2e.rs` lines 11139-12086 (v1_sl_b_fixtures — canonical Case A) IN FULL.
2. Read `crates/server/tests/e2e.rs` lines 88-862 (governance_fixtures — the canonical shared module).
3. Read `crates/server/tests/e2e.rs` lines 5658-5760 (admin_config_fixtures::bootstrap shape).
4. Read `crates/server/tests/e2e.rs` lines 1054-1500 (phase1_migrations_round_trip, the test being split).
5. Read `crates/server/tests/e2e.rs` lines 8029-12500 (the 4 phase-specific fixtures modules) — full read needed because dedup analysis requires comparing them.
6. Read `.claude/lessons/feedback_lemmy_error_no_std_error.md` Case A enumeration.
7. Read `.claude/lessons/feedback_clippy_test_style.md` for test-style invariants.
8. Read `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — ANCHOR-BASED EDITS ONLY on this 8945-line file.

If the file has drifted since 2026-05-14 (e.g. line numbers have moved by >50 from the audit's reading), STOP and file a DQ blocker. The refactor depends on stable file positions.

### §2.6 Edit discipline (mandatory; e2e.rs is the worker-hang risk surface)

Per `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md`: the worker has hung on e2e.rs Edit operations in the past. MANDATORY discipline:

- **NEVER `Read` the full e2e.rs file** — always use `offset` + `limit`.
- **`Grep` for the exact anchor string BEFORE every `Edit`** to confirm the `old_string` is unique.
- **Each `Edit` call uses surgical `old_string`** (3-5 lines of context, not whole functions).
- **NO `replace_all: true` on this file** — too risky.
- **Commit after every ~10 edits** locally (you can squash later) so progress isn't lost on a crash.

### §2.7 Validation gate (Shape G)

After all 4 passes complete, push the worker branch. Triggers `cargo-validate-workspace.yml`.

Workspace check runs `cargo test --no-run` which compiles tests (does NOT run them). For full validation, the user gate 4 (Phase 2 e2e — local vs dispatch) per `feedback_phase_2_e2e_gate_enforcement.md` runs the actual e2e suite. Given this is a test-refactor, the Phase 2 e2e is the LOAD-BEARING signal — workspace check alone is insufficient.

**Worker MUST run the full e2e suite locally before push**:

```bash
bash scripts/brehon/cargo-test.sh --workspace --features full --test e2e > .claude/PRPs/debug/refactor-e2e-runtests.log 2>&1
status=$?
tail -40 .claude/PRPs/debug/refactor-e2e-runtests.log
[ $status -eq 0 ] || exit $status
```

This is ~26 min on a typical machine. Run locally; do NOT push until all tests pass. Per `.claude/rules/cargo-output-capture.md` + `no-cargo-output-paste.md` (capture-then-tail; never paste full log into commit/DQ).

After local pass + push, raise `kind: "validate-pending"` DQ entry per Recipe 1.

## §3 Required reading

- `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` §§3.E.1, 3.E.2, 3.E.3, 3.E.4 — all 4 driving findings; §5.6 cross-cutting test-corpus observation
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` — PR-1 of 6 context; this is the largest PR in the tier
- `crates/server/tests/e2e.rs` lines 11139-12086 (v1_sl_b_fixtures — canonical Case A shape)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A/B/C enumeration; Case A is the target
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — anchor-Edit discipline
- `.claude/lessons/feedback_clippy_test_style.md` — test-style invariants
- `.claude/lessons/feedback_phase1_migration_count_lifo.md` — Pass 4 motivation
- `.claude/agents/impl-task.md` — subagent contract
- `.claude/rules/decision-queue.md` Recipe 1
- `.claude/rules/cargo-output-capture.md` + `.claude/rules/no-cargo-output-paste.md` — capture-then-tail; never paste cargo output
- Upstream Lemmy `crates/api/api/tests/` or equivalent (via `git show upstream/main:<path>`) — uniform LemmyResult test style for reference

## §4 Constraints

- **Files:** ONLY `crates/server/tests/e2e.rs`. NO other files. The refactor is self-contained.
- **No behavior change:** every existing test must still pass after refactor with the same assertions. Refactor is mechanical (type signatures, dedup) + structural (split phase1_migrations test). No new test cases. No removed assertions. **If a test breaks because of the refactor and you can't fix it cleanly, STOP and file a DQ blocker** — don't `#[ignore]` it, don't delete the assertion, don't paper over.
- **Final test count must equal pre-refactor count + 2** (the round-trip split adds 2 — original 1 fn becomes 3 fns; net +2). Verify with `grep -c "^async fn " crates/server/tests/e2e.rs`.
- **No `#[ignore]` annotations** added.
- **No `#[allow(...)]` escape hatches** added.
- **Branch:** `chore/refactor-e2e-error-types`.
- **Anchor-Edit discipline (mandatory per `feedback_junior_worker_e2e_edit_hang.md`):** every Edit on this 8945-line file uses surgical `old_string` (3-5 line context). NO `replace_all: true`. NO full-file Read. Per-pass batching: ~10 edits at a time, then local commit + cargo check.
- **Pre-push gates (mandatory per `feedback_fix_impl_pre_push_cargo_check.md` + this brief):**

  ```bash
  # Gate 1: cargo check
  bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/refactor-e2e-precheck.log 2>&1
  status=$?
  tail -20 .claude/PRPs/debug/refactor-e2e-precheck.log
  [ $status -eq 0 ] || exit $status

  # Gate 2: clippy --tests
  bash scripts/brehon/cargo-clippy.sh --workspace --features full --tests --no-deps -- -D warnings > .claude/PRPs/debug/refactor-e2e-clippy.log 2>&1
  status=$?
  tail -20 .claude/PRPs/debug/refactor-e2e-clippy.log
  [ $status -eq 0 ] || exit $status

  # Gate 3: e2e tests (FULL run, ~26 min) — load-bearing signal
  bash scripts/brehon/cargo-test.sh --workspace --features full --test e2e > .claude/PRPs/debug/refactor-e2e-tests.log 2>&1
  status=$?
  tail -40 .claude/PRPs/debug/refactor-e2e-tests.log
  [ $status -eq 0 ] || exit $status
  ```

  All 3 gates must exit 0. Per `.claude/rules/cargo-output-capture.md` — capture-then-tail.

- **Shape G:** push triggers workspace check. Raise 1 validate-pending DQ entry.
- **DQ atomic raise + ensure_ascii=False + next_id-spans-archives.**
- **Phase 2 e2e gate:** the user MUST exercise user gate 4 at bm-pr time per `feedback_phase_2_e2e_gate_enforcement.md`. This refactor touches `e2e.rs`; the bm-pr.md Phase 1c plan-aware gate WILL fire. The DQ for this gate is satisfied by the local e2e run from Gate 3 above (raise `kind: "validate-pending-laptop-e2e"` per `advisor-orchestrator.md` §5.2 validate-pending-laptop handler).
- **COMMIT MESSAGE:** `chore(test): unify e2e error-type to LemmyResult + consolidate fixtures + split phase1_migrations_round_trip (audit 3.E.1+2+3+4)`

## §5 Out of scope

- Adding NEW test fns (audit didn't request any beyond the 2-from-1 split in Pass 3).
- Removing existing assertions.
- Refactoring `governance_fixtures` or `admin_config_fixtures` beyond what Pass 2 dedup requires.
- Updating CI / GH Actions workflows.
- Changing the migration round-trip's assertion content (only the structure splits).
- Touching `apub/activities/src/governance/` files (those are audit §3.E.12-18 — separate findings, separate refactors).
- Renaming any of the 6 module names (`governance_fixtures`, `admin_config_fixtures`, etc).
- Migrating the file out of `crates/server/tests/e2e.rs` into smaller files (deferred to v0-test-coverage future work).

## §6 HANDOVER trailer

CRITICAL-tier finding bundle (4 findings, ranks 1+2+7+8). Trailer mandatory:

```yaml
HANDOVER:
  filesCreated: []
  filesModified:
    - crates/server/tests/e2e.rs
  keyDecisions:
    - All Brehon-authored test fns + helpers in e2e.rs now uniform LemmyResult<()> outer + LemmyResult<T> helpers (Case A per feedback_lemmy_error_no_std_error.md). Box<dyn Error> eliminated from Brehon-authored scope.
    - 4 phase-specific fixtures modules (v1_jm_b, v1_jm_e, v1_sl_b, v1_sl_c) consolidated shared seeders into governance_fixtures (or new governance_test_helpers); phase-specific modules retain only phase-specific assertions.
    - phase1_migrations_round_trip split into 3 fns (forward / revert / reapply); each independent with own fresh-Postgres fixture.
    - PHASE_1_MIGRATION_COUNT replaced with MIGRATIONS_TO_REVERT_PHASE_1 named-migration list, per feedback_phase1_migration_count_lifo.md (or documented as bookkeeping-fiction-pending-runner-API if migration runner does not support named revert).
  notes: Largest refactor in fix-before-next-phase tier (L effort). Test count post-refactor = pre-refactor + 2 (round-trip split adds 2). Local e2e suite pass (~26 min) is the load-bearing validation; workspace-check workflow alone is insufficient for a test-refactor. Phase 2 e2e gate satisfied via local `validate-pending-laptop-e2e` DQ raise at push time.
```
