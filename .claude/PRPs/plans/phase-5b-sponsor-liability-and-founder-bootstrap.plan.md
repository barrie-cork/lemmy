# Phase 5b — Sponsor-liability, jury gating, founder bootstrap

**Plan file.** `.claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md`
**Branch.** `phase-5b` (cut from `governance-v0` @ `5a4a0f0a5` at task 0).
**Scope.** Tasks 56–60 from IMPLEMENTATION-PLAN-v0.md §3 Phase 5b + task 0 (pre-phase audit) + task 61 (phase-close PR). Total **7 task slots, 5 substantive**.
**`/prp-ralph` iterations.** `--max-iterations 10` per advisor-context-phase-5.md rule 6.
**Prior-phase HEAD.** `governance-v0 @ 5a4a0f0a5` (post-Phase-5a merge, 2026-04-17). Pre-merge tip `9dbbbe14a`; PR #4 was fast-forwarded onto `governance-v0`.

---

## §0. Table of contents (line-numbered)

| §     | Section                                                                                     | Line |
|-------|---------------------------------------------------------------------------------------------|------|
| §1    | Summary                                                                                     | 37   |
| §2    | Sources / ADRs / OQs                                                                        | 62   |
| §3    | Problem statement                                                                           | 98   |
| §4    | Solution statement                                                                          | 121  |
| §5    | Metadata                                                                                    | 141  |
| §6    | Critical conventions (fork-local)                                                           | 162  |
| §7    | File tree (new + touched)                                                                   | 196  |
| §8    | Watchpoint coverage matrix                                                                  | 226  |
| §9    | Definition-of-done commands (per-task, dry-run-verified at plan-write time)                 | 250  |
| §10   | `SanctionAction::Restoration` fan-out audit                                                 | 277  |
| §11   | Step-by-step tasks                                                                          | 309  |
| §11.0 | Task 0 — pre-phase harness audit + branch cut + decision-queue intake                       | 311  |
| §11.1 | Task 56 — sponsor-liability helper + `submit_jury_vote` wire-in + `Restoration` variant     | 370  |
| §11.2 | Task 57 — reputation gating + concurrent-cap in `admin_assign_jury::select_eligible_jurors` | 532  |
| §11.3 | Task 58 — OQ-006 threshold formula in `create_report`                                       | 608  |
| §11.4 | Task 59 — founder seeding CLI                                                               | 684  |
| §11.5 | Task 60 — 5b e2e test (three branches)                                                      | 767  |
| §11.6 | Task 61 — phase-close validation + report + PR                                              | 872  |
| §12   | Validation commands (Level 0–5)                                                             | 920  |
| §13   | Testing strategy                                                                            | 974  |
| §14   | Risk register                                                                               | 993  |
| §15   | Acceptance criteria                                                                         | 1010 |
| §16   | Plan correction policy                                                                      | 1025 |
| §17   | Notes + decision-queue intake                                                               | 1040 |

---

## §1. Summary

Phase 5b ships the **sponsor-liability math** on top of the Phase 5a reputation infrastructure, with founder-multiplier semantics per OQ-022 (`now()` at case-close), honour-price floor clamp per OQ-024, and the `SanctionAction::Restoration { description: String }` variant per OQ-003 (amended). It mutates three Phase 4 handler files, adds one new module, one new migration (ALTER TYPE enum — `-- no-transaction`), one new CLI binary, and one new e2e test with three branches. The `report_to_modlog_golden_path` regression guard remains load-bearing: task 56 inserts the liability call between sanction-insert and case-flip, and the test's Phase 4 flow must still pass.

After 5b merges, every Phase 4 endpoint that writes governance state reads its thresholds/deltas/multipliers from `governance_config` — the "hardcoded constants" era that survived Phase 4 ends here. Phase 5c then ships the remaining 6 endpoints + capability-gating e2e test.

Deliverables:

1. **`apply_sponsor_liability`** helper in `crates/api/api/src/governance/sponsor_liability.rs` with exhaustive severity bucket match over all 8 `SanctionAction` variants (including new `Restoration`), integer floor-division with deterministic remainder-direction (Issue B), founder-multiplier check per-sponsor/per-event via `now()` at call time, and honour-price floor clamp with dual governance-log emission (`sponsor_liability_applied` + `sponsor_liability_clamped`).
2. **`SanctionAction::Restoration { description: String }`** variant added to `crates/db_schema_file/src/enums.rs` with matching Postgres `ALTER TYPE sanction_action ADD VALUE IF NOT EXISTS 'Restoration'` migration using the `-- no-transaction` Diesel annotation.
3. **`submit_jury_vote.rs`** Phase 4 mutation: replace hardcoded deltas (`JUROR_ALIGNED_DELTA` etc.) with config reads; insert `apply_sponsor_liability` call between step 8 (sanction insert) and step 9 (case flip).
4. **`admin_assign_jury::select_eligible_jurors`** Phase 4 mutation: reputation gating via `INNER JOIN reputation_snapshot ... WHERE jury_eligible = true`; concurrent-cap subquery; `exclude_person_ids` parameter added; `fallback_on_small_pool` config branch.
5. **`create_report.rs`** Phase 4 mutation: remove `V0_THRESHOLD` + `V0_REPORTER_WEIGHT`; compute OQ-006 formula from config with `f64::is_finite()` guard on the float product.
6. **Founder seeding CLI** — new binary at `crates/tools/seed_founders/` using `lemmy_api::governance::reputation_snapshot::recompute_snapshot` and `governance_log::append` directly (no HTTP).
7. **`sponsor_liability_with_founder_multiplier` e2e test** with three branches: `default_multiplier`, `founder_chain_survival`, `honour_price_floor_clamp`.

---

## §2. Sources / ADRs / OQs

**Primary (homeserver authoritative):**

- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\IMPLEMENTATION-PLAN-v0.md` §3 Phase 5b (tasks 56–60, lines 332–362) + "Phase 5 impact on shipped Phase 4 code" (lines 404–411) + "Phase 5 impact on shipped Phase 1 code" (lines 413–415) + §4 cross-cutting (hash chain, pseudonyms, redaction).
- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\99-decisions-and-open-questions.md` — **OQ-003 amended** (Restoration variant reservation), **OQ-004 resolved** (juror cap=3 instance-wide via config), **OQ-006 resolved** (threshold formula), **OQ-022 resolved** (founder-multiplier timestamp = `now()` at case-close), **OQ-024 resolved** (zero-floor clamp).
- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\01-vision-and-principles.md` §5.2 — sponsor-liability mechanics; honour-price floor rationale; sum-vs-severity note when founders present.
- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\02-domain-model.md` §4 — four reputation dimensions, event-sourced with decay.
- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\06-security-and-threat-model.md` §6.1 — redaction service is the single code path.
- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\04-data-model-and-api.md` §3 — Diesel model shapes for `ReputationEvent`, `Surety`, `Sanction`.

**Advisor context:** `C:\Users\barri\Developer\homeserver\.claude\advisor-context-phase-5.md` — **watchpoints 3, 4, 5, 10** are primary for 5b; §5 operational rule 6 (`--max-iterations 10`), rule 19 (PR workflow), rule 20 (grep-before-commit); §7 catch-fire procedures.

**Phase 5a outputs consumed (homeserver + fork on-disk after merge):**

- `.claude/PRPs/plans/phase-5a-config-and-reputation-infrastructure.plan.md` §17.2 — **verbatim** carry-forward (1) `-- no-transaction` for ALTER TYPE migration, carry-forward (3) `f64::is_finite()` guard on task 58 threshold formula. Both are copied into §11.1 and §11.3 task bodies below.
- `.claude/PRPs/reports/phase-5a-complete-report.md` §3 — 12 documented deviations. **Deviation 7** (branchful `SELECT FOR UPDATE` + INSERT-or-UPDATE upsert pattern) is the established fork pattern for snapshot writes. **Deviation 9** (clokwerk in `scheduled_tasks.rs`, NOT `tokio::spawn` from `governance.rs`) is architecturally locked in and unchanged in 5b.
- `crates/api/api/src/governance/config.rs` — the 34 seeded keys + typed accessors. 5b reads `deltas.sponsor_liability_{minor,moderate,severe}`, `liability.{founder_multiplier,regular_multiplier,sponsor_liability_floor}`, `report.{base_weight,clamp_min,clamp_max,recency_half_life_hours,case_threshold_micros}`, `jury.{max_concurrent_assignments,fallback_on_small_pool,age_requirement_days,panel_size}`, `thresholds.endorsement_strength`, `founder.{max_founders_active,max_expires_days,max_seed_delta}`, `deltas.{juror_aligned,juror_outlier,reporter_upheld,reporter_dismissed}`.
- `crates/api/api/src/governance/reputation_snapshot.rs` — 5b task 56's honour-price clamp reads current `endorsement_strength` from the most recent snapshot row via `read_existing_snapshot_for_update` (sibling of the function `recompute_snapshot` already exposes).
- `crates/api/api/src/governance/governance_log.rs` — `ENTRY_KIND_SPONSOR_LIABILITY_APPLIED`, `ENTRY_KIND_SPONSOR_LIABILITY_CLAMPED`, `ENTRY_KIND_FOUNDER_SEEDED` already exist as public constants (verified at plan-write time, lines 60–62 of governance_log.rs) — **no new ENTRY_KIND constants needed in 5b**.
- `crates/db_views/reputation/src/lib.rs` — `ReputationSummaryView` with `endorsement_strength: i32` + `jury_eligible: bool` + `active_sanctions: i64`. 5b task 60 may read this in test assertions.

**Fork-local rules (auto-loaded in `-p` mode):** `.claude/rules/phase-branch.md`, `.claude/rules/pre-phase-harness-audit.md`, `.claude/rules/cargo-output-capture.md`, `.claude/rules/no-cargo-output-paste.md`, `.claude/rules/decision-queue.md`, `.claude/rules/gh-pr-fork-target.md`, `.claude/rules/view-crate-selectable-template.md`.

---

## §3. Problem statement

After Phase 5a the reputation infrastructure works — `governance_config` is the single source of tuneable values, `reputation_snapshot` rows exist for every user with `can_sponsor` / `jury_eligible` / `trusted_reporter` computed fresh every 15 minutes, and `create_endorsement` dispatches on a config-driven gate strategy. **But the sanction-to-reputation pipeline is still Phase 4.** Four concrete gaps remain:

1. **`submit_jury_vote.rs:222–251`** emits a sanction row and writes `sanction_created` to the governance log, then flips the case to `Decided` without touching sponsor reputation. Per vision §5.2 and OQ-022, when a sanctioned user has active sponsors their reputation must absorb a share of the severity — weighted by `founder_multiplier` for sponsors whose `reputation_event.expires_at` is still in the future at close time. None of that exists today.

2. **`submit_jury_vote.rs:82–90`** hardcodes `JUROR_ALIGNED_DELTA: i32 = 10`, `JUROR_OUTLIER_DELTA: i32 = -5`, `REPORTER_ACCURATE_DELTA: i32 = 10`, `REPORTER_INACCURATE_DELTA: i32 = -5`. Phase 5a seeded the matching config keys (`deltas.juror_aligned` etc.) but did not remove the constants — that migration is task 56's scope.

3. **`admin_assign_jury.rs:162–188`** filters jurors only by "not target, not reporter, not deleted, accepted_application". No reputation gate, no concurrent-cap. Task 57 consumes the `reputation_snapshot.jury_eligible` column that Phase 5a started writing, plus the `config.jury.max_concurrent_assignments` key that 5a seeded. Phase 4b's decision-queue #10 (target-person inference for Post-target cases) remains out of scope — flagged as a carry-forward to 5c or v1, not opened here.

4. **`create_report.rs:65–72`** uses the `V0_THRESHOLD: i64 = 3` + `V0_REPORTER_WEIGHT: i64 = 1` stubs with an explicit `TODO(brehon-fork, phase-5)` marker pointing at OQ-006. The formula resolved at 5a plan-write time; 5b task 58 implements it: `weight_micros = base_weight × clamp(reporting_accuracy/100, clamp_min, clamp_max) × exp(-hours_old/recency_half_life_hours) × 1_000_000`, with an `is_finite()` guard against admin misconfiguration.

These four gaps and the **`SanctionAction::Restoration` enum amendment** (OQ-003) are the whole of Phase 5b. The founder-seeding CLI (task 59) is net-new but architecturally simple — it reuses the snapshot + governance_log helpers shipped by 5a. The e2e test (task 60) is the DoD gate.

---

## §4. Solution statement

Task 0 audits the wrapper scripts + DoD commands against the post-5a-merge baseline and cuts `phase-5b` from `governance-v0 @ 5a4a0f0a5`. Task 56 is the highest-reasoning task in all of Phase 5 — it lands the Restoration variant, writes the sponsor-liability helper, mutates `submit_jury_vote.rs`, and migrates the four delta constants to config reads, all in one commit. Task 57 extends `select_eligible_jurors` with the INNER JOIN on `reputation_snapshot` and the concurrent-cap subquery. Task 58 removes `V0_THRESHOLD`/`V0_REPORTER_WEIGHT` and implements the OQ-006 formula with `f64::is_finite()` guard. Task 59 ships the founder-seeding CLI as a new binary at `crates/tools/seed_founders/`. Task 60 is the e2e test with three branches. Task 61 is phase-close: Level 0–5 validation, completion report, PR open.

Tasks 56, 57, 58 each touch an existing Phase 4 file. The discipline from Phase 5a plan §16 corrections policy applies: grep every symbol before committing; dry-run every DoD command at plan-write time; do not invent migration file names (read the seeded ones from 5a at plan-write time).

**Plan-write-time dry-run results (see §9):** all five Level-1 commands (`cargo-check.bat --features full --workspace`, `cargo-clippy.bat --features full --workspace --no-deps -- -D warnings`, `cargo-test.bat -p lemmy_server --test e2e --no-run`, `cargo-test.bat -p lemmy_api --features full -- governance::config::parity`, `cargo-test.bat -p lemmy_server --test e2e -- report_to_modlog_golden_path`) pass against `governance-v0 @ 5a4a0f0a5`. No Level-0 carry-patch is required (unlike 5a, which needed the `pagination.rs #[expect]` fix).

The 34-seed parity test from Phase 5a guards against 5b adding a const fallback without seeding a row (Watch 1). Since 5b does NOT add new config keys — it only reads keys seeded in 5a — the parity test is a passive regression check, not a 5b DoD gate. Every 5b config read cites the exact key name from `crates/api/api/src/governance/config.rs`'s `SEEDED_KEYS` list at plan-write time.

---

## §5. Metadata

| Field                                              | Value                                                                                                                                                             |
|----------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Type                                               | HANDLER + SCHEMA + CLI + TEST + CROSS_CUTTING                                                                                                                     |
| Complexity                                         | HIGH — sponsor-liability integer math + founder-multiplier ordering + Phase 4 file mutations + first-migration-since-5a (ALTER TYPE with `-- no-transaction`)    |
| Crates affected (code)                             | `crates/db_schema_file` (enums), `crates/api/api` (governance/sponsor_liability.rs + 3 Phase 4 mutations), `crates/api/api_crud` (governance/create_report.rs mutation), `crates/tools/seed_founders` (new binary), `crates/server/tests/e2e.rs` (3 new branches) |
| Crates affected (schema)                           | `migrations/{timestamp}_add_restoration_sanction_variant/` — new migration with `-- no-transaction`                                                              |
| New public surface                                 | `lemmy_api::governance::sponsor_liability::apply_sponsor_liability(...)` (crate-visible pub; called only from `submit_jury_vote`); `SanctionAction::Restoration { description: String }` variant |
| Mutated public surface                             | `admin_assign_jury::select_eligible_jurors(conn, case, exclude_person_ids: Option<&[PersonId]>)` — added parameter; existing call site in `admin_emergency_remove` passes `None` |
| DoD-command count                                  | 13 (Level 0 grep parity × 1 + Level 1 × 2 + Level 2 × 3 + Level 3 × 1 + Level 4 × 2 + Level 5 × 4)                                                                |
| Expected LOC                                       | ~900 net-add (sponsor_liability.rs ~250; founder CLI ~200; e2e test ~350; migration ~10; Phase 4 mutations ~90)                                                 |
| Expected commits                                   | 7 (task 0 audit-only = 0 commits; tasks 56–60 = 5 task commits; task 61 = 1 docs(report) commit; plus optional 1 `docs(plan):` correction if grep turns up drift) |
| Ralph-iteration budget                             | `--max-iterations 10`                                                                                                                                            |
| Per-task soft iteration cap                        | 4. Tasks 56 (integer math) and 60 (multi-branch e2e) may need more — monitor but do not pre-emptively raise                                                      |
| Decision-queue entries pre-seeded                  | #11 (juror cap = 3; resolved by OQ-004 — close at task 0), #12 (sponsor-liability units; resolved by plan body — close at task 0)                                 |

---

## §6. Critical conventions (fork-local)

These conventions are load-bearing; every task step below assumes them. Listed once here rather than restated per task.

1. **Windows wrapper scripts only.** Every cargo invocation uses `cmd //c "scripts\\brehon\\cargo-check.bat ..."` / `cargo-clippy.bat` / `cargo-test.bat`. Direct `cargo` calls fail (vcvars + libpq). Task 0 probes these wrappers.

2. **Output capture then tail, never pipe.** Per `.claude/rules/cargo-output-capture.md`: `cmd //c "... > .claude/build-taskN.log 2>&1"; status=$?; tail -20 .claude/build-taskN.log; echo "exit: $status"`. Piping to `tail`/`grep` masks exit codes.

3. **No cargo output paste.** Per `.claude/rules/no-cargo-output-paste.md`: tail ≤20 lines per log read. Full logs stay on disk for diagnosis.

4. **Phase branch only.** Per `.claude/rules/phase-branch.md`: commits land on `phase-5b` from Phase 5 onward; no direct `governance-v0` commits; PR against `governance-v0` at phase close via `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-5b`.

5. **`--repo barrie-cork/lemmy`.** Per `.claude/rules/gh-pr-fork-target.md`: every `gh pr create` / `gh pr view` carries the flag. `gh` defaults to upstream `LemmyNet/lemmy` on forks.

6. **Commit-message convention.** `feat(scope): task N — <one-line summary>` for task commits; `docs(plan): <summary>` for plan corrections; `docs(report): <summary>` for completion report. Body ≤60 words except `docs(plan):` which may run to 100.

7. **One commit per task.** Tasks 56–60 each land as exactly one commit except where Phase 4 mutations inherently require an atomic multi-file change (task 56 is the largest — Restoration variant + migration + sponsor_liability.rs + submit_jury_vote.rs + delta-const-removal all in one commit to preserve the "compile green after each task" invariant).

8. **Exhaustive matches on governance enums.** Per `feedback_clippy_test_style.md` + workspace clippy denies `_ =>` wildcards. Every `match decision: JuryDecision` / `match action: SanctionAction` names every variant. Task 56 is the primary site.

9. **`actor_pseudonym_helper::get_or_create` is the single path.** Per [06 §6.1] + Watch 10: every `person_id` that reaches a governance_log payload goes through the helper. Raw `person_id.0` in a payload fails the Watch 10 grep in task 60.

10. **`run_transaction` for multi-write handlers.** Per `feedback_multi_write_handlers_need_transactions.md`: the existing `submit_jury_vote` closure (lines 109–116) wraps every write. Task 56's `apply_sponsor_liability` call is INSIDE the same closure. Tasks 57 and 58 do not add new write sites; they modify read-side filters.

11. **View-crate `Selectable` template.** Per `.claude/rules/view-crate-selectable-template.md`: 5b does not create new view crates. Pre-existing rule applies only if task 60 adds a helper struct (it does not; assertions read `ReputationSummaryView` and raw `reputation_event` rows).

12. **Cargo-output capture at each validation step.** Every DoD command captured to `.claude/build-taskN-<level>.log`; status checked; tail read. Commits gate on exit 0 for all levels, not just `cargo check`.

---

## §7. File tree (new + touched)

**New files:**

```
migrations/
  2026-04-19-000000-0000_add_restoration_sanction_variant/
    up.sql                # -- no-transaction; ALTER TYPE sanction_action ADD VALUE IF NOT EXISTS 'Restoration'
    down.sql              # -- no-op per OQ-003 + Postgres enum-drop limitation; doc-comment only
crates/api/api/src/governance/
  sponsor_liability.rs    # apply_sponsor_liability + severity bucket + founder-multiplier + floor clamp
crates/tools/
  Cargo.toml              # workspace crate declaration (if not already present)
  seed_founders/
    Cargo.toml            # binary crate; deps on lemmy_api, lemmy_db_schema, lemmy_api_utils
    src/main.rs           # CLI entry point; reads DATABASE_URL env; parses --admin-user + --founder <spec>
```

**Touched files:**

```
crates/db_schema_file/src/enums.rs                         # add Restoration { description: String } variant to SanctionAction
crates/api/api/src/governance/submit_jury_vote.rs          # remove 4 delta consts (lines 82-90); replace with ConfigCache reads; insert apply_sponsor_liability call between step 8 and step 9; add Restoration arm to any future exhaustive match (none exists today; this is a no-op for v0)
crates/api/api/src/governance/admin_assign_jury.rs         # extend select_eligible_jurors with reputation gating + concurrent-cap + exclude_person_ids parameter + fallback_on_small_pool branch
crates/api/api/src/governance/admin_emergency_remove.rs    # pass exclude_person_ids=None to select_eligible_jurors (parameter addition ripple)
crates/api/api/src/governance/mod.rs                       # pub mod sponsor_liability
crates/api/api_crud/src/governance/create_report.rs        # remove V0_THRESHOLD + V0_REPORTER_WEIGHT; compute OQ-006 formula; f64::is_finite() guard
crates/server/tests/e2e.rs                                 # sponsor_liability_with_founder_multiplier test with 3 branches
Cargo.toml (workspace)                                     # add `crates/tools/seed_founders` to [workspace.members] (ONLY if tools/ not already in members); verify at task 59 plan-write time below
```

---

## §8. Watchpoint coverage matrix

Per advisor-context-phase-5.md §4. Every Watch mitigation is phrased as a grep-verifiable invariant (per feedback_watch_mitigation_style.md's spirit and the task-53 Phase 5a pattern).

| Watch | Concern                                                                                   | Task(s)      | Mitigating artefact (grep-verifiable post-impl)                                                                                                                                                                       |
|-------|-------------------------------------------------------------------------------------------|--------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 3     | Sponsor-liability integer math + founder multiplier + exhaustive `SanctionAction` match   | 56           | `grep -nE 'as f64 \* (founder\|regular)_multiplier' crates/api/api/src/governance/sponsor_liability.rs` returns zero hits (integer-only after ×multiplier stage); `grep -nE 'SanctionAction::(Label\|VisibilityReduction\|TemporaryRestriction\|ContentRemoval\|CommunityExclusion\|InstanceSuspension\|FederationQuarantineRecommendation\|Restoration)' crates/api/api/src/governance/sponsor_liability.rs` returns ≥8 hits (one per variant arm, no wildcard) |
| 4     | Phase 4 golden-path regression after submit_jury_vote + create_report mutations           | 56, 58, 61   | `cargo-test.bat -p lemmy_server --test e2e -- report_to_modlog_golden_path` exits 0 at task-56 HEAD AND at task-58 HEAD AND at task-61 HEAD (three Level-2 runs)                                                       |
| 5     | Branch and PR workflow (`phase-5b` from post-5a-merge `governance-v0`)                    | 0, 61        | `git branch --show-current` returns `phase-5b` after task 0; `gh pr view --repo barrie-cork/lemmy` shows `base:governance-v0 head:phase-5b` after task 61                                                              |
| 10    | Modlog PII leakage in new payloads (`sponsor_liability_applied/clamped`, `founder_seeded`) | 56, 59, 60   | `grep -nE '"person_id"\|"target_person_id"\|"sponsored_id"\|"sponsor_id"' crates/api/api/src/governance/sponsor_liability.rs crates/tools/seed_founders/src/main.rs` returns zero hits; task 60 runs an assertion loop over every governance_log payload from the test run and asserts the same grep pattern yields zero matches (payloads must only contain pseudonyms) |

Watch 1 (seed/const parity) is 5a's concern; 5b reads keys but adds none. Watch 2, 6–9 are 5a's concerns; 5b does not modify the snapshot calculator or its filters. Watch 11 (admin attribution) is 5c's concern (admin-config-write.sh wrapper per decision-queue #13).

---

## §9. Definition-of-done commands (per-task, dry-run-verified at plan-write time)

All commands use the wrapper scripts per §6 rule 1 and output capture per rule 2. DoD is "exit 0 on all listed commands, on the task-N HEAD, with tail inspected and no `error:`/`warning:`/`failed` markers."

**Dry-run results against `governance-v0 @ 5a4a0f0a5` at plan-write time (2026-04-17):**

| Command                                                                                                                                                          | Intent                                        | Expected on task-N HEAD   |
|------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------|---------------------------|
| `cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/l1-check.log 2>&1"`                                                             | Workspace-wide `cargo check` with `full` feature | Exit 0, zero warnings     |
| `cmd //c "scripts\\brehon\\cargo-clippy.bat --features full --workspace --no-deps -- -D warnings > .claude/l1-clippy.log 2>&1"`                                  | Workspace clippy, deny warnings                | Exit 0                    |
| `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/l3-compile.log 2>&1"`                                                    | e2e test target compiles                       | Exit 0                    |
| `cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- config_parity_round_trip > .claude/l2-parity.log 2>&1"`                                  | Phase 5a parity test still passes (DB round-trip; narrowed per decision-queue #15) | Exit 0                    |
| `cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- report_to_modlog_golden_path > .claude/l2-golden.log 2>&1"`                              | Phase 4 golden-path regression guard (Watch 4) | Exit 0                    |
| `cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- sponsor_liability_with_founder_multiplier > .claude/l2-5b-e2e.log 2>&1"`                 | Task 60 new test (all 3 branches)              | Exit 0 after task 60      |
| `bash scripts/brehon/lint-no-membership-read.sh`                                                                                                                 | Phase 5a Watch 7 lint guard (regression)       | Exit 0                    |
| `bash scripts/brehon/lint-no-can-sponsor-read.sh`                                                                                                                | Phase 5a Watch 7 sibling guard (regression)    | Exit 0                    |
| `head -1 migrations/$(ls migrations/ \| grep restoration_sanction_variant)/up.sql \| grep -Fx '-- no-transaction'`                                                | §17.2 carry-forward (1) assertion              | Exit 0 after task 56      |

**Wrapper-script audit (§8 rule 1 alignment).** At plan-write time I grep-verified that `-p lemmy_server`, `-p lemmy_api`, and `-p lemmy_server --test e2e` invocations **do not** pass `--features full` on `lemmy_server` (5a deviation #3 — `lemmy_server/Cargo.toml` has no `full` feature). All Level 1/2/3 commands above honour that constraint. The wrapper's `%*` forwarding preserves these flags unchanged; no wrapper fix needed.

**DoD-narrowing note (decision-queue #15, 2026-04-17).** Plan-write-time dry run incorrectly claimed `cargo-test.bat -p lemmy_api --features full -- governance::config::parity` exited 0 at `5a4a0f0a5`; pre-phase-harness-audit (`.claude/rules/pre-phase-harness-audit.md`) caught the drift. Root cause: `lemmy_api_crud` is a dev-dep of `lemmy_api` (`crates/api/api/Cargo.toml:88`), so `cargo test -p lemmy_api` compiles it. Upstream OAuth code at `crates/api/api_crud/src/user/create.rs:638` calls `.form()` on `reqwest_middleware::client::RequestBuilder` — method only available under full workspace feature unification, not `-p lemmy_api --features full` partial unification. Narrowed the DoD to use `-p lemmy_server --test e2e -- config_parity_round_trip` (verified exit 0 at task-0 audit); the in-crate `parity::seeded_keys_count_matches_const_count` structural test is still covered by Level 1 `cargo-check.bat --features full --workspace` (workspace feature unification compiles it). Coverage preserved; narrowing is proportionate.

---

## §10. `SanctionAction::Restoration` fan-out audit

Per the task-write-time grep against `C:\Users\barri\Developer\brehon-fork\crates\**\*.rs` (excluding `.claude/worktrees/`):

**Exhaustive-match sites on `SanctionAction`:** **zero** today. The only current exhaustive match involving `SanctionAction` is in `submit_jury_vote.rs::map_decision_to_sanction` (lines 378–404), but that match is over `JuryDecision` (8 variants) with `SanctionAction` appearing only on the RHS of arms — it does NOT `match` on `SanctionAction` itself. Adding `Restoration` to `SanctionAction` does not force a new arm there.

**Passive usage sites (imports + struct fields, no match):**

- `crates/db_schema/src/source/governance/sanction.rs:3,24,40` — `Sanction.action: SanctionAction` (struct field; auto-accepts new variant) and `SanctionInsertForm.action: SanctionAction` (same).
- `crates/db_views/governance_modlog/src/lib.rs:27,55` — `sanction_action: Option<SanctionAction>` (view drift stub; auto-accepts).
- `crates/db_schema_file/src/schema.rs` — Diesel sql_types binding; regenerated by the migration, no Rust fan-out.
- `crates/db_schema_file/src/enums.rs:540–549` — the enum declaration itself; task 56 is the editing site.

**Match sites created BY task 56 (not pre-existing):**

- `crates/api/api/src/governance/sponsor_liability.rs::severity_for_action(action: SanctionAction) -> LiabilitySeverity` — the bucket mapper. Exhaustive over all 8 variants **including `Restoration`** (bucket: minor, matching `Label`'s severity per §11.1 GOTCHA-56a). This match is BORN exhaustive — there is no "before" state with 7 arms and an "after" state with 8; task 56 writes 8 in one commit.

**Plan-lesson implication (per brief).** Fan-out for `Restoration` is **1 new exhaustive-match site created by task 56 itself**. Unlike the Phase 5a Person-literal fan-out (3 carry-patches needed in upstream test fixtures because Lemmy's existing `Person` struct gained a required field), adding a new enum variant does NOT force carry-patches — the existing passive field sites auto-accept. **No `chore(carry-patches):` commit is anticipated in 5b.** Task 56 consolidates the variant + migration + helper into a single atomic commit that compiles green on first application.

**Migration file naming (per §17.2 carry-forward 1).** The `ALTER TYPE` lives in its own migration directory: `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/`. Timestamp chosen one day after Phase 5a's migrations (`2026-04-18-*`) so the embed-migrations lexicographic order matches the Phase-5a-then-5b intent. `up.sql` first line is `-- no-transaction`; `down.sql` is a doc-comment only (Postgres enum-value drop unsupported without full type rebuild).

---

## §11. Step-by-step tasks

### §11.0 Task 0 — Pre-phase harness audit + branch cut + decision-queue intake

**Goal.** Catch wrapper-script drift and pre-existing DoD-command breakage before any 5b code lands. Cut the `phase-5b` feature branch from post-5a-merge `governance-v0`. Close out decision-queue entries #11 and #12.

**Steps** (all in `cmd //c` with output capture):

1. **Branch verification and cut.**
   ```bash
   git fetch origin
   git checkout governance-v0
   git pull --ff-only
   git log -1 --format=%H origin/governance-v0     # expect: 5a4a0f0a5... (or successor if CodeRabbit follow-ups merged)
   git status --short                               # expect: clean tree
   git checkout -b phase-5b
   git branch --show-current                        # expect: phase-5b
   ```

2. **Wrapper flag audit (Probes 1–3 per `.claude/rules/pre-phase-harness-audit.md`).**
   ```bash
   cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/audit-cargo-check-p.log 2>&1"
   status=$?; tail -15 .claude/audit-cargo-check-p.log; echo "exit: $status"
   # EXPECT: only lemmy_api compiles. If you see "Checking lemmy_db_schema" — STOP, wrapper discards -p.

   cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/audit-cargo-check-features.log 2>&1"
   status=$?; tail -15 .claude/audit-cargo-check-features.log; echo "exit: $status"
   # EXPECT: --features full visible in cargo invocation; lemmy_api compiles.

   cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/audit-cargo-test.log 2>&1"
   status=$?; tail -15 .claude/audit-cargo-test.log; echo "exit: $status"
   # EXPECT: only e2e test target compiles. No spurious additional test binaries.
   ```

3. **DoD smoke test — every §9 command against phase-5b HEAD (pre-task-56).**
   ```bash
   # Level 1
   cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/audit-l1-check.log 2>&1"
   status=$?; tail -15 .claude/audit-l1-check.log; echo "check exit: $status"; [ $status -eq 0 ] || exit 1

   cmd //c "scripts\\brehon\\cargo-clippy.bat --features full --workspace --no-deps -- -D warnings > .claude/audit-l1-clippy.log 2>&1"
   status=$?; tail -30 .claude/audit-l1-clippy.log; echo "clippy exit: $status"; [ $status -eq 0 ] || exit 1

   # Level 2 — Phase 5a parity (DB round-trip, narrowed per decision-queue #15) + Phase 4 golden-path regression guards
   cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- config_parity_round_trip > .claude/audit-l2-parity.log 2>&1"
   status=$?; tail -15 .claude/audit-l2-parity.log; echo "parity exit: $status"; [ $status -eq 0 ] || exit 1

   cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- report_to_modlog_golden_path > .claude/audit-l2-golden.log 2>&1"
   status=$?; tail -15 .claude/audit-l2-golden.log; echo "golden exit: $status"; [ $status -eq 0 ] || exit 1

   # Level 5 — lint guards
   bash scripts/brehon/lint-no-membership-read.sh; echo "membership guard exit: $?"
   bash scripts/brehon/lint-no-can-sponsor-read.sh; echo "can_sponsor guard exit: $?"
   ```

4. **Decision-queue close-out.** Read `.claude/decision-queue.json`. Questions #11 and #12 are pre-seeded for 5b and resolved by OQ-004 and the plan body. Move them to `resolved` with `"answered_by": "planner"`:

   - **#11** (juror cap=3): `answer`: "Hardcode semantically via `config.jury.max_concurrent_assignments` — task 57's concurrent-cap subquery reads this key. Seeded value at 5a is 3; config-change without code change (OQ-004 resolution). Task 57 does NOT `const` the cap — it reads from ConfigCache so the cap is tuneable per §6 rule 10."
   - **#12** (sponsor-liability severity units minor=-10/moderate=-50/severe=-200): `answer`: "Confirmed per IMPLEMENTATION-PLAN-v0.md §3 Phase 5b task 55 and §17.2 carry-forwards. Mapping per §11.1 GOTCHA-56a: `Label` + `VisibilityReduction` + `Restoration` → minor; `TemporaryRestriction` + `ContentRemoval` → moderate; `CommunityExclusion` + `InstanceSuspension` + `FederationQuarantineRecommendation` → severe. All three units are `deltas.sponsor_liability_{minor,moderate,severe}` config keys from 5a's seed list."

   Write the updated JSON back. Do NOT open new decision-queue questions at task 0 — task 0 is pre-flight only; questions emerge mid-task.

5. **CodeRabbit re-review carry-over resolution.** Per advisor-context-phase-5.md paragraph at §1 ("plus one new 5b carry-over from CodeRabbit re-review: `config.rs:63-70` `Scope::as_str` allocation refactor"): bundle this into task 56's scope AS a pre-commit refactor step. Specifically:
   - Task 56 step 0 (before the main task body) changes `fn as_str(self) -> String` (line 64 of `config.rs`) to `fn as_str(self) -> std::borrow::Cow<'static, str>` or similar allocation-free form. Advisor-specified at §17 below. See §11.1 task 56 step 0 for the exact shape.

**DoD.**
- [ ] Current branch is `phase-5b`.
- [ ] All three wrapper probes pass (exit 0, no flag discard).
- [ ] All five DoD dry-runs exit 0.
- [ ] Both lint guards exit 0.
- [ ] Decision-queue entries #11 and #12 moved from `pending` to `resolved`.
- [ ] **If any audit step fails, STOP.** Do not start task 56 on a broken baseline. Surface to advisor via a fresh decision-queue entry.

**Commit.** Zero commits from task 0 itself (unlike 5a's task 0 which landed the pagination carry-patch — 5b baseline needs no carry-patch per the plan-write-time dry run). Task 0 produces `.claude/audit-*.log` files and the decision-queue mutation; the branch exists but has no commits yet.

---

### §11.1 Task 56 — Sponsor-liability helper + `submit_jury_vote` wire-in + `SanctionAction::Restoration` variant

**Goal.** Ship the sponsor-liability helper, the `Restoration` enum variant + its Postgres migration, the `submit_jury_vote.rs` mutation (delta consts → config reads + insertion of the `apply_sponsor_liability` call between step 8 and step 9), and the config.rs `Scope::as_str` allocation refactor. **This is the highest-reasoning task in all of Phase 5**; integer math + founder-multiplier ordering + first-migration-since-5a + Phase 4 file mutation all concentrated here.

**Steps.**

0. **Pre-task refactor — `Scope::as_str` allocation elimination (CodeRabbit carry-over).**
   Change `crates/api/api/src/governance/config.rs:63-70`:
   ```rust
   // BEFORE (current, 2 String allocations per Scope::Instance read):
   impl Scope {
     fn as_str(self) -> String {
       match self {
         Scope::Instance => "instance".to_string(),
         Scope::Community(CommunityId(id)) => format!("community:{id}"),
       }
     }
   }

   // AFTER (Cow borrows static str for Instance, allocates only for Community):
   impl Scope {
     fn as_str(self) -> std::borrow::Cow<'static, str> {
       match self {
         Scope::Instance => std::borrow::Cow::Borrowed("instance"),
         Scope::Community(CommunityId(id)) => std::borrow::Cow::Owned(format!("community:{id}")),
       }
     }
   }
   ```
   Update all six call sites inside `config.rs` (`get_int`/`get_float`/`get_bool`/`get_text`/`fetch_value` — four accessors each currently call `scope.as_str()` to build the cache key and the DB filter). Use `.as_ref()` or `.into_owned()` where a `&str` / `String` is needed. **This is strictly a refactor with no behaviour change**; the Phase 5a `config_parity_round_trip` test and the `parity::seeded_keys_count_matches_const_count` compile-time test both pass unchanged after this edit.

1. **Migration — `ALTER TYPE sanction_action ADD VALUE 'Restoration'`** (§17.2 carry-forward 1, verbatim).

   Create `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/up.sql`:
   ```sql
   -- no-transaction
   ALTER TYPE sanction_action ADD VALUE IF NOT EXISTS 'Restoration';
   ```
   The `-- no-transaction` directive is Diesel's opt-out: in Postgres < 12 `ALTER TYPE ... ADD VALUE` is forbidden inside a transaction block; in Postgres 12+ the catalog update must commit before the new value can be used elsewhere. `IF NOT EXISTS` makes the migration idempotent across `diesel migration redo`.

   Create `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/down.sql`:
   ```sql
   -- no-transaction
   -- Postgres does not support dropping an enum value without rebuilding the entire type.
   -- Down-path is intentionally a no-op; rolling back past this migration requires a full
   -- type rebuild (DROP TYPE + CREATE TYPE + update every column that uses it).
   -- See [99 OQ-003] for the reasoning behind making the variant reservation irreversible
   -- at the v0 migration level.
   SELECT 1;
   ```
   Per the carry-forward: "**5b DoD assertion**: `head -1 crates/db_schema/migrations/{timestamp}_add_restoration_sanction_variant/up.sql` returns `-- no-transaction` exactly." Adapted for the fork's actual migration path: `migrations/` (not `crates/db_schema/migrations/`). Exact assertion at §9 DoD row: `head -1 migrations/$(ls migrations/ | grep restoration_sanction_variant)/up.sql | grep -Fx '-- no-transaction'` exits 0.

2. **Rust enum variant — `SanctionAction::Restoration { description: String }`.**

   Edit `crates/db_schema_file/src/enums.rs:540-549`:
   ```rust
   // BEFORE (7 unit variants):
   pub enum SanctionAction {
     #[default]
     Label,
     VisibilityReduction,
     TemporaryRestriction,
     ContentRemoval,
     CommunityExclusion,
     InstanceSuspension,
     FederationQuarantineRecommendation,
   }

   // AFTER (7 unit variants + 1 struct variant):
   pub enum SanctionAction {
     #[default]
     Label,
     VisibilityReduction,
     TemporaryRestriction,
     ContentRemoval,
     CommunityExclusion,
     InstanceSuspension,
     FederationQuarantineRecommendation,
     /// v0 reserved slot for `folog n-othrusa`-style restorative sanctions per
     /// [99 OQ-003 (amended 2026-04-17)](../../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md).
     /// Not selected by any v0 handler — reserved for future code paths (v1
     /// `admin_restorative_action` endpoint) and enum-exhaustiveness in downstream
     /// matches so v1's refinement into specific variants (`Apology | ContentCorrection
     /// | CommunityService`) is variant-refinement, not pattern-replacement.
     /// Severity bucket: minor (same as `Label`) per §11.1 GOTCHA-56a.
     Restoration { description: String },
   }
   ```

   **GOTCHA-56b — `DbEnum` struct-variant compatibility.** `diesel-derive-enum`'s `DbEnum` derive supports struct variants ONLY via `DbValueStyle = "verbatim"` + an accompanying `ToSql`/`FromSql` implementation, or via a supporting `pg_type` representation. At plan-write time I did not verify the fork's `diesel-derive-enum` version handles struct variants natively — the existing `SanctionAction` derive at line 532–537 uses `ExistingTypePath = "crate::schema::sql_types::SanctionAction"` + `DbValueStyle = "verbatim"`. **If the task 56 ralph iteration hits a macro-expansion error on the `Restoration { description: String }` derive**, the fallback plan is: keep the enum variant as `Restoration { description: String }` at the Rust layer but serialise it to Postgres as the unit token `'Restoration'` (discarding the description at the DB boundary), storing the description in a separate column. At plan-write time the cleanest path is likely: define `Restoration` WITHOUT a payload at the enum (matching the 7 existing unit variants), and add a separate `sanction.restoration_description: Option<String>` column via a sibling migration. **If the ralph iteration confirms struct-variant derives work**, keep the struct variant as specified. Document the chosen path in the task 56 commit body. This is the single most likely plan-iteration point in 5b; budget for it.

3. **Regenerate schema file for the enum addition.** After the migration runs (in the test harness via `schema_setup::run`), verify `crates/db_schema_file/src/schema.rs` generated for the `sanction_action` enum includes `Restoration`. If the fork generates schema.rs manually, edit the `sql_types::SanctionAction` module. At plan-write time I grep-verified the existing entries:
   ```
   grep -n 'SanctionAction' crates/db_schema_file/src/schema.rs
   ```
   returns 2 hits (sql_types module + table column binding). No Rust code changes expected here; the migration handles the type update at the DB level and Diesel's derive-enum consumes the Rust variant list.

4. **Sponsor-liability helper — `crates/api/api/src/governance/sponsor_liability.rs`.**

   Full signature and function body sketch:
   ```rust
   use crate::governance::{
     actor_pseudonym_helper,
     config::{self, ConfigCache, Scope},
     governance_log,
     reputation_snapshot,
   };
   use diesel::{ExpressionMethods, QueryDsl, dsl::{exists, select}};
   use diesel_async::{AsyncPgConnection, RunQueryDsl};
   use lemmy_api_utils::context::LemmyContext;
   use lemmy_db_schema::newtypes::{CommunityId, ModerationCaseId};
   use lemmy_db_schema::source::governance::reputation_event::ReputationEventInsertForm;
   use lemmy_db_schema_file::{PersonId, enums::{ReputationDimension, SanctionAction}};
   use lemmy_db_schema_file::schema::{reputation_event, reputation_snapshot, surety};
   use lemmy_utils::error::LemmyResult;
   use serde_json::json;

   /// Severity bucket for sponsor-liability delta lookup.
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   enum LiabilitySeverity { Minor, Moderate, Severe }

   impl LiabilitySeverity {
     fn config_key(self) -> &'static str {
       match self {
         Self::Minor    => "deltas.sponsor_liability_minor",
         Self::Moderate => "deltas.sponsor_liability_moderate",
         Self::Severe   => "deltas.sponsor_liability_severe",
       }
     }
   }

   /// Map `SanctionAction` to severity bucket. **Exhaustive, no `_ =>` wildcard.**
   /// GOTCHA-56a: `Restoration` maps to Minor — same bucket as `Label` per vision §4 principle 5
   /// (restorative actions are the softest intervention). Upgrade to a dedicated `restoration` bucket
   /// is a v1 refinement; see [99 OQ-003] for the variant-refinement roadmap.
   fn severity_for_action(action: &SanctionAction) -> LiabilitySeverity {
     match action {
       SanctionAction::Label
       | SanctionAction::VisibilityReduction
       | SanctionAction::Restoration { .. }                                  => LiabilitySeverity::Minor,
       SanctionAction::TemporaryRestriction
       | SanctionAction::ContentRemoval                                      => LiabilitySeverity::Moderate,
       SanctionAction::CommunityExclusion
       | SanctionAction::InstanceSuspension
       | SanctionAction::FederationQuarantineRecommendation                  => LiabilitySeverity::Severe,
     }
   }

   /// Apply sponsor-liability deltas to all active sponsors of `target_person_id`.
   ///
   /// # Ordering (Watch 3)
   ///
   /// For each sponsor:
   ///   raw_delta (from config.deltas.sponsor_liability_<bucket>, negative)
   ///   → /sponsor_count (integer floor-division; remainder to first |r| sponsors by sponsor_id ASC)
   ///   → ×multiplier (f64; `founder_multiplier` if sponsor has unexpired founder event at now(), else `regular_multiplier`)
   ///   → round half-to-even back to i64
   ///   → clamp_to_floor (read current `endorsement_strength` from most recent snapshot; if `current + final_delta < floor`, clamp so post-event == floor)
   ///
   /// # Returns
   ///
   /// Count of active sponsors processed (same as the number of
   /// `reputation_event` rows and `sponsor_liability_applied` log entries
   /// this call writes).
   pub(crate) async fn apply_sponsor_liability(
     conn: &mut AsyncPgConnection,
     context: &LemmyContext,
     target_person_id: PersonId,
     case_id: ModerationCaseId,
     community_id: Option<CommunityId>,
     action: &SanctionAction,
     cache: &mut ConfigCache,
   ) -> LemmyResult<usize> { /* ... */ }
   ```

   **Body (full pseudocode; impl agent materialises).**

   - Map action to severity; read `raw_delta: i64 = config::get_int(cache, pool, Scope::Instance, severity.config_key()).await?`.
   - Query active sureties: `surety WHERE sponsored_id = target_person_id AND revoked_at IS NULL ORDER BY sponsor_id ASC`. Load `Vec<PersonId>` of sponsor ids.
   - If zero sponsors, early-return `Ok(0)`.
   - Compute `per_sponsor_base: i64 = raw_delta / sponsor_count` and `remainder: i64 = raw_delta % sponsor_count` (Rust's `/` and `%` on signed integers are trunc-toward-zero; for `raw_delta = -50` and `sponsor_count = 3` this yields `per_sponsor_base = -16, remainder = -2`). For **Issue B remainder direction (§17.2 carry-forward 2 from plan DoD line 357): for negative deltas, the first `|remainder|` sponsors by `sponsor_id ASC` each receive one extra unit of negative (i.e. MORE negative), so total sums to severity.** The first 2 of the 3 sponsors get `-17` (base `-16` + one extra `-1`); the third gets `-16`. Sum: `-17 + -17 + -16 = -50`. ✓
   - For each sponsor in ASC order, index `i` from 0:
     - `remainder_bump: i64 = if (i as i64) < remainder.unsigned_abs() as i64 { if raw_delta < 0 { -1 } else { 1 } } else { 0 }`.
     - `pre_multiplier_delta: i64 = per_sponsor_base + remainder_bump`.
     - **Founder check (OQ-022 — §17.2 carry-forward on founder-multiplier timestamp: `now()` at case-close, NOT case-open):** `is_founder: bool` from `diesel::select(exists(reputation_event::table.filter(reputation_event::person_id.eq(sponsor_id)).filter(reputation_event::dimension.eq(ReputationDimension::EndorsementStrength)).filter(reputation_event::expires_at.is_not_null()).filter(reputation_event::expires_at.gt(diesel::dsl::now))))`. The `now()` reference is the **current SQL time at the call** — which, because this function is called from `submit_jury_vote::process_vote` at step 8.5 (between sanction-insert and case-flip), semantically equals `now() at case-close`. Document this in a GOTCHA on the function body.
     - `multiplier_key = if is_founder { "liability.founder_multiplier" } else { "liability.regular_multiplier" }`; `multiplier: f64 = config::get_float(cache, ..., multiplier_key).await?`.
     - **Integer-safe multiply:** `let multiplied_f64 = (pre_multiplier_delta as f64) * multiplier; let post_multiplier_delta: i64 = multiplied_f64.round_ties_even() as i64;` (Rust 1.77+; if unavailable, fallback `(multiplied_f64 + 0.5_f64.copysign(multiplied_f64)) as i64`). Round half-to-even (banker's rounding) to avoid cumulative drift across sponsors per GOTCHA-56c.
     - **Honour-price floor clamp (OQ-024):**
       - `current_endorsement_strength: i32 = reputation_snapshot::read_endorsement_strength(conn, sponsor_id, community_id).await.unwrap_or(0);` (helper to be added in the same task to `reputation_snapshot.rs` — or inlined if cleaner; read the most recent snapshot row's `endorsement_strength`, default 0 if none exists).
       - `floor: i64 = config::get_int(cache, ..., "liability.sponsor_liability_floor").await?;` (default seeded 0).
       - `final_delta: i64 = post_multiplier_delta; let mut clamped_from: Option<i64> = None; if (current_endorsement_strength as i64) + final_delta < floor { clamped_from = Some(final_delta); final_delta = floor - (current_endorsement_strength as i64); }`.
       - `final_delta_i32 = i32::try_from(final_delta).map_err(|_| LemmyErrorType::Unknown("sponsor-liability delta overflow".into()))?;`
     - **Write `reputation_event`** with `person_id = sponsor_id`, `community_id` (case-level), `dimension = EndorsementStrength`, `delta = final_delta_i32`, `source_case_id = Some(case_id)`, `reason = "sponsor_liability_applied"`, `expires_at = None`.
     - **Emit `governance_log` entry `sponsor_liability_applied`** with pseudonymised sponsor via `actor_pseudonym_helper::get_or_create`. Payload:
       ```rust
       json!({
         "case_id": case_id.0,
         "sponsor_pseudonym": sponsor_pseudonym,  // NOT sponsor_id
         "severity": severity_str,                // "minor" | "moderate" | "severe"
         "pre_multiplier_delta": pre_multiplier_delta,
         "multiplier": multiplier,
         "post_multiplier_delta": post_multiplier_delta,
         "final_delta": final_delta,
         "is_founder": is_founder,
       })
       ```
       Per Watch 10: no `sponsor_id`, no `target_person_id`, no raw numeric person ids anywhere.
     - **If `clamped_from.is_some()`, emit second log entry `sponsor_liability_clamped`** with pseudonym + uncapped delta + clamped delta + floor value. Payload:
       ```rust
       json!({
         "case_id": case_id.0,
         "sponsor_pseudonym": sponsor_pseudonym,
         "uncapped_delta": clamped_from.unwrap(),
         "clamped_delta": final_delta,
         "floor": floor,
         "current_endorsement_strength": current_endorsement_strength,
       })
       ```
   - Return `Ok(sponsor_count)`.

   **GOTCHA-56a (severity bucket for Restoration).** Restorative sanctions ARE still sanctions (they imply the target violated norms); the sponsors absorb the softest bucket. Per vision §4 principle 5, restoration keeps the person *inside* the system — but the sponsor chain still bears some accountability for the decision to sponsor someone whose behaviour required intervention. Mapping to Minor is the minimum-damage choice for v0.

   **GOTCHA-56c (integer math drift).** `round_ties_even` on stable Rust 1.77+ is correct. If the toolchain is pinned below 1.77 (check `rust-toolchain.toml` — CLAUDE.md says `1.94`; fine), use it. If unavailable, do NOT use `as i64` directly on the multiplied float (truncate-toward-zero introduces bias over many sponsors); use the copysign trick above.

   **GOTCHA-56d (zero-sponsor early return).** When `target_person_id` has no active sureties, `apply_sponsor_liability` returns `Ok(0)` without writing any `reputation_event` or `governance_log` entry. Phase 4's golden-path test creates the target via `LocalUserBuilder` with no sponsor chain, so the 0-sponsor path is the default and the regression guard relies on it.

   **GOTCHA-56e (transaction scope).** `apply_sponsor_liability` runs inside `submit_jury_vote::process_vote`'s `run_transaction` closure (step 8.5; between the existing step 8 at line 222 and step 9 at line 253). The `conn: &mut AsyncPgConnection` parameter is the transaction connection; all writes are part of the outer transaction. **Do NOT open a nested `run_transaction`.** Reads (config cache fills, founder check, current endorsement_strength) use the same connection and see the transaction's pre-write state of `reputation_snapshot` + `reputation_event`. This matters because `submit_jury_vote` does NOT call `recompute_snapshot` itself — the snapshot refresh happens asynchronously via the 15-min clokwerk tick. The floor clamp therefore reads a potentially stale snapshot, which is the semantics per OQ-024 (clamp based on the snapshot in effect at case-close; snapshot refresh is Phase 5a's concern).

   **GOTCHA-56f (founder check correctness).** Per OQ-022 the timestamp is `now()` at case-close. Because the function runs inside the vote-tally transaction and Postgres `now()` in a transaction returns the transaction start time (`transaction_timestamp()`), this is **exactly case-close ± a few milliseconds of query time**. Federation reproducibility (Phase 6 concern) follows — every federating node that replays the case-close evaluates `now()` at their own transaction start, which may produce a different answer for a sponsor whose founder event expired in the intervening seconds. Documented as an OQ-022 consequence accepted in v0.

   **GOTCHA-56g (no SELECT-then-INSERT race).** The sponsor list read + founder-event existence check + reputation_snapshot read + reputation_event insert all happen in the same transaction. Concurrent `create_endorsement` / `revoke_endorsement` on the same sponsee cannot race because `submit_jury_vote` is single-threaded per case (the vote tally flips the case to `Decided` and further votes error with `NotFound` at line 140). The worst case is a concurrent endorsement on a DIFFERENT case's target that creates a new sponsor row not visible here — correct, because that new sponsor didn't vouch for THIS case's target at case-close time.

5. **`submit_jury_vote.rs` mutation — config reads + `apply_sponsor_liability` insertion.**

   - **Remove** lines 82–90 (the four `const` blocks: `JUROR_ALIGNED_DELTA`, `JUROR_OUTLIER_DELTA`, `REPORTER_ACCURATE_DELTA`, `REPORTER_INACCURATE_DELTA`).
   - **Replace** the direct const references at lines 300–302 + 322–325 with `ConfigCache` reads. Inside `process_vote`, instantiate `let mut cache = ConfigCache::new();` at function entry and thread it through. For each config key read, call `config::get_int(&mut cache, &mut pool_ref, Scope::Instance, "deltas.juror_aligned").await?` etc. The `pool_ref` here is a `DbPool` built from `context` — check if `process_vote` has access to `context`; it currently does not (takes only `conn` + `juror_id` + `juror_pseudonym` + `data`). Either:
     - **Path A:** add `context: &LemmyContext` to `process_vote` and pass it from `submit_jury_vote` — threads through, cleanest. Config reads fetch their own pool from context.
     - **Path B:** add `config_cache: &mut ConfigCache` to `process_vote` and initialise it in the outer `submit_jury_vote` before the `run_transaction`; have it pre-populate at transaction start by reading every config key the handler needs BEFORE entering the transaction (since `get_int` currently takes `&mut DbPool<'_>` and would need another connection inside a transaction). **At plan-write time, Path B is cleaner** — avoid nested connection acquisition inside a transaction. Pre-read all five keys (`deltas.juror_aligned`, `deltas.juror_outlier`, `deltas.reporter_upheld`, `deltas.reporter_dismissed`, plus whatever sponsor_liability keys task 56's helper needs) into the cache at handler entry, then read from the cache inside the transaction.

     **At implementation time, check the actual `config::get_int` signature and confirm whether `run_transaction`'s `conn: &mut AsyncPgConnection` can satisfy it, or whether the pool-outside pattern is required.** Path B with pre-warm is the conservative choice; Path A with context-threading is cleaner if the signatures allow. This is a plan-iteration flex point.

   - **Insert** between existing lines 251 (end of step 8: `sanction_created` governance_log emission) and 253 (start of step 9: case flip to `Decided`):
     ```rust
     // 8.5. Apply sponsor-liability to active sponsors of target (if any).
     // Per OQ-022, founder-multiplier evaluated at now() — i.e. now (case-close).
     // Per OQ-024, honour-price floor clamp prevents permanent-outcast state.
     if let Some(target_id) = case_row.target_person_id {
       sponsor_liability::apply_sponsor_liability(
         conn,
         /* context: &LemmyContext -- via Path A/B decision */,
         target_id,
         data.case_id,
         case_row.community_id,
         &action,  // from the `if let Some((scope, action))` at line 223
         &mut cache,
       ).await?;
     }
     ```
     The `action` variable is in scope inside the `if let Some((scope, action))` block (lines 223–251). Move the `apply_sponsor_liability` call into that block (so it runs only when a sanction was created — NoAction skips both).

   **GOTCHA-56h.** `case_row.target_person_id` is `None` for Post/Comment-target cases. Phase 4b's decision-queue #10 documented that bug; the golden-path test uses Person-target so the `if let Some(target_id)` path is exercised. For Post-target cases, sponsor-liability is silently skipped — a real hole per OQ-024, tracked in `.claude/decision-queue.json` as a 5c carry-forward (to be opened by task 61 if the hole affects task 60's test branches; it does not — all three branches use Person-target).

6. **`mod.rs` wiring.** Add `pub mod sponsor_liability;` to `crates/api/api/src/governance/mod.rs`. Keep pub-crate via `pub(crate)` on the function; no external consumers in 5b (task 59's CLI uses the helper NOT — the CLI only seeds events, doesn't apply liability).

7. **Validation — after the single task-56 commit.**

   Run the full §9 DoD sweep. Expected green: Level 1 check + clippy; Level 2 parity (Phase 5a regression) + golden-path (Phase 4 regression, **critical** — task 56 mutates `submit_jury_vote`); Level 3 e2e compile; Level 5 lint guards. Level 2 5b e2e NOT yet green (test not written).

   Specifically run:
   ```bash
   cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- report_to_modlog_golden_path > .claude/build-task56-golden.log 2>&1"
   status=$?; tail -30 .claude/build-task56-golden.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
   ```
   If this fails after task 56, the most likely cause is the Path A/B threading for config reads — the golden-path test doesn't seed the config keys task 56 now reads, OR the pool-threading inside `run_transaction` is broken. Both are fixable without a plan change.

**DoD.**
- [ ] `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/up.sql` exists; first line is exactly `-- no-transaction`; second line is `ALTER TYPE sanction_action ADD VALUE IF NOT EXISTS 'Restoration';`.
- [ ] `SanctionAction::Restoration` variant exists in `enums.rs` (payload form per GOTCHA-56b decision — struct or unit).
- [ ] `crates/api/api/src/governance/sponsor_liability.rs` compiles; `severity_for_action` covers all 8 variants exhaustively (no `_ =>`).
- [ ] `config.rs:63-70` `Scope::as_str` returns `Cow<'static, str>` (or equivalent allocation-free form); all six call sites updated.
- [ ] Four Phase 4 delta consts removed from `submit_jury_vote.rs:82-90`; ConfigCache reads in their place.
- [ ] `apply_sponsor_liability` call inserted between step 8 and step 9 in `process_vote`, inside the `if let Some((scope, action))` block.
- [ ] Level 1 check + clippy exit 0.
- [ ] `report_to_modlog_golden_path` exits 0 (Watch 4).
- [ ] `config_parity_round_trip` exits 0 (Watch 1 regression; narrowed per decision-queue #15).
- [ ] Watch 10 grep on `sponsor_liability.rs` for `\b(person_id|target_person_id|sponsored_id|sponsor_id)\b` returns zero hits.

**Commit.** `feat(governance): task 56 — sponsor_liability helper + Restoration variant + submit_jury_vote config reads + Scope::as_str refactor` (body ≤60 words, cite OQ-003/022/024, carry-forward §17.2 resolution).

---

### §11.2 Task 57 — Reputation gating + concurrent-cap in `admin_assign_jury::select_eligible_jurors`

**Goal.** Extend the Phase 4 eligibility filter with (a) INNER JOIN on `reputation_snapshot` with `jury_eligible = true`, (b) concurrent-cap subquery via `NOT IN (SELECT person_id FROM jury_assignment WHERE status IN ('Selected', 'Accepted') GROUP BY person_id HAVING count(*) >= max_concurrent_assignments)`, (c) new `exclude_person_ids: Option<&[PersonId]>` parameter for task 57's future decline-replacement pick in 5c, (d) `fallback_on_small_pool` config branch.

**Steps.**

1. **Signature mutation.**

   ```rust
   // BEFORE (Phase 4):
   pub(crate) async fn select_eligible_jurors(
     conn: &mut diesel_async::AsyncPgConnection,
     case: &ModerationCase,
   ) -> LemmyResult<Vec<PersonId>> { ... }

   // AFTER (task 57):
   pub(crate) async fn select_eligible_jurors(
     conn: &mut diesel_async::AsyncPgConnection,
     context: &LemmyContext,  // for ConfigCache reads via pool; optional if Path B pre-warm is adopted
     case: &ModerationCase,
     exclude_person_ids: Option<&[PersonId]>,
     cache: &mut ConfigCache,
   ) -> LemmyResult<Vec<PersonId>> { ... }
   ```

2. **Query body.**

   Current body (lines 162–188 of `admin_assign_jury.rs`) builds a boxed query on `person INNER JOIN local_user` with `deleted = false` + `accepted_application = true` + `ne(target)` + `ne(reporter)`. Task 57 adds:

   - `INNER JOIN reputation_snapshot ON reputation_snapshot.person_id = person.id AND reputation_snapshot.community_id IS NOT DISTINCT FROM <case.community_id>`. The `IS NOT DISTINCT FROM` handles NULL-matches-NULL correctly — the partial unique index from Phase 5a task 50 (`reputation_snapshot_person_null_community`) ensures instance-scoped snapshots exist and dedupe. **Note:** Diesel's DSL may not expose `IS NOT DISTINCT FROM` directly; the fallback is `sql::<Bool>("reputation_snapshot.community_id IS NOT DISTINCT FROM ?").bind::<Nullable<Integer>, _>(case.community_id)` or equivalent `sql_query` raw SQL. At plan-write time, grep for existing uses of `IS NOT DISTINCT FROM` in the fork:
     ```
     grep -rn 'IS NOT DISTINCT FROM' crates/
     ```
     returns 1 hit at `crates/api/api/src/governance/reputation_snapshot.rs` (Phase 5a task 53 used it via `sql_query` raw SQL). Same pattern applies here.
   - `.filter(reputation_snapshot::jury_eligible.eq(true))`.
   - `.filter(person::id.ne_all(excluded_ids))` where `excluded_ids` is the constructed list of: `case.target_person_id` (if Some), `case.creator_id` (if Some), plus each id from `exclude_person_ids` parameter. (Prefer `ne_all` or a single `.filter(person::id.ne(...))` per id — Diesel doesn't always compile an arbitrary `NOT IN` cleanly with `into_boxed`.)
   - Concurrent-cap — the crux. The subquery:
     ```sql
     person.id NOT IN (
       SELECT person_id FROM jury_assignment
       WHERE status IN ('Selected', 'Accepted')
       GROUP BY person_id
       HAVING count(*) >= <max_concurrent_assignments>
     )
     ```
     Diesel DSL path: `use diesel::dsl::{exists, not}; .filter(not(exists(jury_assignment::table.filter(jury_assignment::person_id.eq(person::id)).filter(jury_assignment::status.eq_any([JuryAssignmentStatus::Selected, JuryAssignmentStatus::Accepted])).having(diesel::dsl::count(jury_assignment::person_id).ge(max_cap)).group_by(jury_assignment::person_id))))`. If the DSL doesn't compile cleanly (Diesel subquery with `HAVING` is finicky), fall back to `sql_query` raw SQL with a `.load::<PersonId>(conn)` driven by a templated SQL string. **At plan-write time, assume DSL works; budget 1 ralph iteration for fallback to raw SQL if not.**

3. **Config reads.**

   ```rust
   let panel_size: i64 = config::get_int(cache, ..., Scope::Instance, "jury.panel_size").await?;
   let max_concurrent: i64 = config::get_int(cache, ..., Scope::Instance, "jury.max_concurrent_assignments").await?;
   let fallback_allowed: bool = config::get_bool(cache, ..., Scope::Instance, "jury.fallback_on_small_pool").await?;
   ```
   `jury.panel_size` replaces the `PANEL_SIZE: i64 = 5` const at `admin_assign_jury.rs:40` — remove the const.

4. **Fallback branch.**

   ```rust
   let eligible = <new filtered query>.order(random()).limit(panel_size).select(person::id).load::<PersonId>(conn).await?;
   if (eligible.len() as i64) < panel_size {
     if !fallback_allowed {
       return Err(LemmyErrorType::NotFound.into());
     }
     warn!("jury pool below panel_size={panel_size} — falling back to unfiltered pool per config.jury.fallback_on_small_pool=true");
     // Fallback: Phase 4 body, i.e. the filter WITHOUT the reputation_snapshot INNER JOIN / concurrent-cap / exclude_person_ids restrictions.
     return legacy_select_eligible_jurors(conn, case).await;
   }
   Ok(eligible)
   ```
   Preserve the Phase 4 logic as a private `async fn legacy_select_eligible_jurors` that the fallback branch calls. Do NOT fall through to the old filter — document that tier 1 filters are stricter.

5. **Call-site updates.**

   - `admin_assign_jury.rs:94` (the main handler call) — pass `context`, `case`, `exclude_person_ids=None`, `cache`.
   - `admin_emergency_remove.rs` — grep for `select_eligible_jurors` call:
     ```
     grep -n 'select_eligible_jurors' crates/api/api/src/governance/admin_emergency_remove.rs
     ```
     At plan-write time returns 1 hit. Pass `exclude_person_ids=None` + threaded cache.

**GOTCHA-57a.** The Phase 5a `reputation_snapshot` table may not have rows for every person at the time `admin_assign_jury` fires — new users registered between the last 15-min tick and the current moment have no snapshot. Since task 53 wrote zero-valued rows for zero-events users at snapshot time, the edge case is "user registered since the last tick." The `INNER JOIN` will exclude them. This is correct per Phase 5a's design — a user without a snapshot is not yet eligible for jury duty. The `fallback_on_small_pool` branch catches the pathological case where the entire server has fewer than 5 snapshot rows.

**GOTCHA-57b (Phase 4b decision-queue #10 remains).** The target-person inference for Post/Comment-target cases is NOT fixed in task 57. The filter still excludes only `case.target_person_id` (None for Post-target) and `case.creator_id`. Budget this as a 5c carry-forward or open a decision-queue question at task 61 if the golden-path test regresses (it won't — the test uses Person-target per 4b deviation #4).

**Validation — after the task-57 commit.**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/build-task57-check.log 2>&1"
status=$?; tail -15 .claude/build-task57-check.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- report_to_modlog_golden_path > .claude/build-task57-golden.log 2>&1"
status=$?; tail -30 .claude/build-task57-golden.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
```

The golden-path test must still pass (Watch 4). The test's target (a Person) has no endorsements — task 56's `apply_sponsor_liability` early-returns `Ok(0)`. For the jury panel, the test uses `admin_assign_jury` — task 57's filter must either succeed (if the test seeds enough eligible snapshot rows) or fall back cleanly. **Inspect the golden-path test's user seeding at plan-write time** via grep on `crates/server/tests/e2e.rs` for the test's setup block (around lines 742+), and confirm whether `fallback_on_small_pool=true` (5a-seeded default) is what lets the test's limited-user-count pool satisfy the assignment. If it doesn't, either the test needs to seed more users OR `fallback_on_small_pool` default is the right compromise.

**Commit.** `feat(governance): task 57 — reputation gating + concurrent cap in admin_assign_jury (config-driven panel size, fallback, exclude_person_ids)` (body ≤60 words).

---

### §11.3 Task 58 — OQ-006 threshold formula in `create_report`

**Goal.** Replace `V0_THRESHOLD` + `V0_REPORTER_WEIGHT` at `create_report.rs:65-72` with the OQ-006 config-driven formula + `f64::is_finite()` guard per §17.2 carry-forward (3).

**Steps.**

1. **Remove constants.** Delete lines 53–71 of `create_report.rs` (comment block + `V0_THRESHOLD` + `V0_REPORTER_WEIGHT`). Keep the imports.

2. **Add snapshot read helper.**

   Per plan text: "At handler entry, call `load_or_compute_snapshot(conn, reporter_id, community_id, &mut config)` (helper that checks for existing row, computes if missing)."

   If no such helper exists in 5a, add a thin wrapper in `reputation_snapshot.rs`:
   ```rust
   pub async fn load_or_compute_snapshot(
     conn: &mut AsyncPgConnection,
     context: &LemmyContext,
     person_id: PersonId,
     community_id: Option<CommunityId>,
     cache: &mut ConfigCache,
   ) -> LemmyResult<ReputationSnapshot> {
     // SELECT the existing row; if None, call recompute_snapshot + return its result.
   }
   ```
   **At plan-write time, grep** `crates/api/api/src/governance/reputation_snapshot.rs` for `load_or_compute_snapshot`: 0 hits — helper does not yet exist. Budget task 58 to add it (~30 lines).

3. **Compute formula.**

   ```rust
   let reporter_snapshot = reputation_snapshot::load_or_compute_snapshot(
     conn, &context, reporter_id, /* community_id */, &mut cache
   ).await?;
   let reporting_accuracy: f64 = reporter_snapshot.reporting_accuracy as f64;

   let base_weight: f64 = config::get_float(&mut cache, ..., Scope::Instance, "report.base_weight").await?;
   let clamp_min: f64 = config::get_float(&mut cache, ..., "report.clamp_min").await?;
   let clamp_max: f64 = config::get_float(&mut cache, ..., "report.clamp_max").await?;
   let half_life_hours: f64 = config::get_float(&mut cache, ..., "report.recency_half_life_hours").await?;
   let threshold_micros: i64 = config::get_int(&mut cache, ..., "report.case_threshold_micros").await?;

   let reporter_reputation = (reporting_accuracy / 100.0).clamp(clamp_min, clamp_max);
   let hours_old: f64 = 0.0;  // fresh report at write time; stale-report recomputation is v1
   let recency_factor: f64 = (-hours_old / half_life_hours).exp();

   let weight_f64 = base_weight * reporter_reputation * recency_factor * 1_000_000.0;

   // §17.2 carry-forward (3) — is_finite() guard. VERBATIM from plan.
   let weight_micros: i64 = if weight_f64.is_finite() {
     weight_f64 as i64
   } else {
     tracing::error!(
       base_weight = base_weight,
       reporter_reputation = reporter_reputation,
       recency_factor = recency_factor,
       "report-weight calculation produced non-finite value; falling back to base_weight × 1_000_000. \
        Likely cause: an admin-edited config key producing Inf/NaN (e.g. recency_half_life_hours = 0, \
        or a negative base_weight combined with an odd-exponent pow).",
     );
     (base_weight * 1_000_000.0) as i64
   };
   ```
   Store `weight_micros` on the moderation_case row (replaces the current `V0_REPORTER_WEIGHT` integer addition at line 126).

4. **Update case-flip threshold check.**

   ```rust
   let new_score = prior_score.saturating_add(weight_micros);
   let should_flip = matches!(prior_status, CaseStatus::Open) && new_score > threshold_micros;
   ```

5. **Observability — log `reporter_reputation` in the `report_created` payload.**

   Find the `governance_log::append("report_created", ...)` call in `create_report.rs` and extend the payload to include `reporter_reputation_multiplier: reporter_reputation` (observability item #2 from plan line 342).

**GOTCHA-58a (stale threshold_score migration).** Phase 5a task 50's `up.sql` multiplied existing `moderation_case.threshold_score` rows by 1_000_000 (per 5a plan GOTCHA-50c). The Phase 4 golden-path test uses `ThresholdMet` forcing — an admin directly sets the status, bypassing the formula. The test's `threshold_score: 1` literal gets rescaled to `1_000_000` by the 5a migration; the test assertion (if any) on `threshold_score` was updated in 5a to accept the new unit. Task 58 does NOT break this: the formula writes micros, the threshold read is micros, the test path is unchanged.

**GOTCHA-58b (unit test for is_finite() fallback).** Per §17.2 carry-forward (3): a unit test that sets `config.report.recency_half_life_hours = 0.0`, calls the computation, and asserts:
1. Fallback value `(base_weight * 1_000_000.0) as i64` is the result (not 0, not `i64::MAX`).
2. `error!` log is emitted (capture via `tracing_subscriber::fmt::test`, or Lemmy's existing test helper — grep for `tracing_subscriber` in `crates/server/tests/e2e.rs` at task 58 impl time to find the pattern).
3. The handler returns `Ok(...)` — non-finite weight does NOT error the request.

This test can live either in `create_report.rs` as a `#[cfg(test)] mod tests` block (preferred; no DB dependency for the math portion) OR as a small e2e test branch in task 60. **Plan-write-time choice: inline unit test in `create_report.rs`** — faster, no container needed for the pure-math part; the `error!` capture via `tracing-subscriber` test layer works in-process. If the `load_or_compute_snapshot` call is unavoidable, defer to task 60. Document the choice in the task 58 commit body.

**Commit.** `feat(governance): task 58 — OQ-006 threshold formula in create_report with is_finite() guard` (body ≤60 words; cite OQ-006 + §17.2 carry-forward (3)).

---

### §11.4 Task 59 — Founder seeding CLI

**Goal.** Standalone binary at `crates/tools/seed_founders/` that inserts `reputation_event` rows with `expires_at` + `reason = "founder_seed"`, calls `recompute_snapshot`, emits `governance_log` entry `founder_seeded`, validated against config-seeded caps.

**Steps.**

1. **Crate scaffolding.**

   Check if `crates/tools/` is already a workspace member:
   ```bash
   grep -n 'crates/tools' Cargo.toml
   ```
   If not, add `"crates/tools/seed_founders"` to `[workspace.members]` in the root `Cargo.toml`. If `crates/tools/` does not exist, create the directory plus `crates/tools/seed_founders/Cargo.toml`:
   ```toml
   [package]
   name = "brehon_seed_founders"
   version = "0.1.0"
   edition.workspace = true
   publish = false

   [[bin]]
   name = "seed_founders"
   path = "src/main.rs"

   [dependencies]
   lemmy_api = { workspace = true, features = ["full"] }
   lemmy_api_utils = { workspace = true }
   lemmy_db_schema = { workspace = true, features = ["full"] }
   lemmy_db_schema_file = { workspace = true, features = ["full"] }
   lemmy_diesel_utils = { workspace = true, features = ["full"] }
   lemmy_utils = { workspace = true }
   chrono = { workspace = true }
   clap = { version = "4", features = ["derive"] }
   diesel = { workspace = true, features = ["postgres", "chrono"] }
   diesel-async = { workspace = true, features = ["postgres"] }
   serde_json = { workspace = true }
   tokio = { workspace = true, features = ["full"] }
   tracing = { workspace = true }
   tracing-subscriber = { workspace = true }
   ```
   (Check actual workspace-dep entries at plan-write time — `clap` may already be in workspace deps; if so, use `clap = { workspace = true, features = ["derive"] }`.)

2. **`main.rs`.**

   CLI skeleton using `clap`:
   ```rust
   #[derive(clap::Parser)]
   struct Args {
     #[clap(long)]
     admin_user: String,  // person.name; the CLI resolves to PersonId at run time
     /// Repeatable: --founder <person_id>:<jury_reliability>:<reporting_accuracy>:<endorsement_strength>
     #[clap(long)]
     founder: Vec<String>,
     /// ISO-8601 date for expiry (required — no default)
     #[clap(long)]
     expires_at: chrono::DateTime<chrono::Utc>,
   }

   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
     tracing_subscriber::fmt::init();
     let args = <Args as clap::Parser>::parse();
     let database_url = std::env::var("DATABASE_URL")?;
     let pool = /* build a DbPool from DATABASE_URL using lemmy_diesel_utils::connection::build_pool or similar */;
     let mut cache = ConfigCache::new();

     // Resolve admin — SELECT person WHERE name = args.admin_user.
     let admin_id: PersonId = /* query */;
     let admin_pseudonym = actor_pseudonym_helper::get_or_create(&mut pool.into(), admin_id).await?;

     // Read config caps.
     let max_active: i64 = config::get_int(&mut cache, ..., Scope::Instance, "founder.max_founders_active").await?;
     let max_expires_days: i64 = config::get_int(&mut cache, ..., "founder.max_expires_days").await?;
     let max_seed_delta: i64 = config::get_int(&mut cache, ..., "founder.max_seed_delta").await?;

     // Validate expires_at ≤ now() + max_expires_days.
     let horizon = chrono::Utc::now() + chrono::Duration::days(max_expires_days);
     if args.expires_at > horizon { bail!("expires_at exceeds founder.max_expires_days horizon"); }

     // Count existing unexpired founder events.
     let current_founder_count: i64 = reputation_event::table
       .filter(reputation_event::reason.eq("founder_seed"))
       .filter(reputation_event::expires_at.is_not_null())
       .filter(reputation_event::expires_at.gt(diesel::dsl::now))
       .select(count_distinct(reputation_event::person_id))
       .first(&mut conn).await?;

     // Parse founder specs + validate.
     let specs = parse_founder_specs(&args.founder, max_seed_delta)?;
     let new_founder_count = specs.len() as i64;
     if current_founder_count + new_founder_count > max_active {
       bail!("would exceed founder.max_founders_active cap");
     }

     // For each founder spec: insert 3 reputation_event rows + emit founder_seeded log + recompute_snapshot.
     for spec in specs {
       let founder_pseudonym = actor_pseudonym_helper::get_or_create(...).await?;
       for (dimension, delta) in &[
         (ReputationDimension::JuryReliability,    spec.jury_reliability),
         (ReputationDimension::ReportingAccuracy,  spec.reporting_accuracy),
         (ReputationDimension::EndorsementStrength, spec.endorsement_strength),
       ] {
         insert_into(reputation_event::table).values(&ReputationEventInsertForm {
           person_id: spec.person_id,
           community_id: None,
           dimension: *dimension,
           delta: *delta as i32,
           source_case_id: None,
           source_report_id: None,
           reason: "founder_seed".to_string(),
           expires_at: Some(args.expires_at),
         }).execute(&mut conn).await?;
       }
       governance_log::append(
         &mut pool.into(),
         "founder_seeded",
         json!({
           "admin_pseudonym": admin_pseudonym,
           "founder_pseudonym": founder_pseudonym,
           "jury_reliability": spec.jury_reliability,
           "reporting_accuracy": spec.reporting_accuracy,
           "endorsement_strength": spec.endorsement_strength,
           "expires_at": args.expires_at,
         }),
         Some(admin_pseudonym.clone()),
       ).await?;
       reputation_snapshot::recompute_snapshot(&mut conn, spec.person_id, None, &mut cache).await?;
     }

     println!("Seeded {} founders with expiry {}", new_founder_count, args.expires_at);
     Ok(())
   }
   ```

3. **participation_consistency is deliberately NOT seeded** per plan line 344 Q3. Do not add a fourth loop iteration.

4. **Idempotency check — per plan line 344 GOTCHA: "Running again adds new events, doesn't disturb existing ones; supports re-seeding after expiry or adding later cohorts of founders."** The `insert_into(reputation_event::table)` ALWAYS creates new rows; no ON CONFLICT handling needed. Verify the existing unique constraints on `reputation_event` permit multiple rows per (person_id, dimension) with different `created_at`; at plan-write time the table allows it (no UNIQUE on those columns; `reputation_event.id` is the PK).

5. **No HTTP surface.** The binary has no route registration, no `actix-web` imports, no JWT middleware. Auth is DB-credential-level via `DATABASE_URL` env var.

**GOTCHA-59a (actor_pseudonym via pool).** `actor_pseudonym_helper::get_or_create` takes `&mut DbPool` (not a connection). The CLI needs a pool — see the Lemmy test harness for the `build_pool` pattern or construct manually via `bb8::Pool::builder().build(...)`. Check `crates/server/src/lib.rs` for the production pattern.

**GOTCHA-59b (Cargo.toml workspace discipline).** Per `feedback_commit_hygiene_lockfiles_and_task_labels.md`: stage `Cargo.lock` with the `Cargo.toml` edit in the same commit. Adding the new crate to `[workspace.members]` plus its `seed_founders/Cargo.toml` both land in the task 59 commit.

**GOTCHA-59c (carry-patch TODO format).** If building the CLI requires touching any upstream Lemmy code (unlikely — it only reads/writes through the governance interfaces), mark the touch with `TODO(brehon-fork): upstream this to LemmyNet/lemmy — PR #___` per `feedback_carry_patch_todos.md`.

**Validation — after the task-59 commit.**

```bash
# Check the binary compiles.
cmd //c "scripts\\brehon\\cargo-check.bat -p brehon_seed_founders > .claude/build-task59-check.log 2>&1"
status=$?; tail -15 .claude/build-task59-check.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

# Workspace clippy still green.
cmd //c "scripts\\brehon\\cargo-clippy.bat --features full --workspace --no-deps -- -D warnings > .claude/build-task59-clippy.log 2>&1"
status=$?; tail -20 .claude/build-task59-clippy.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
```

**Commit.** `feat(tools): task 59 — founder seeding CLI (crates/tools/seed_founders)` (body ≤60 words; cite ADR-015 pseudonym discipline).

---

### §11.5 Task 60 — 5b e2e test: `sponsor_liability_with_founder_multiplier` (three branches)

**Goal.** One new test in `crates/server/tests/e2e.rs` with three explicit branches (default_multiplier, founder_chain_survival, honour_price_floor_clamp) + a PII-grep assertion loop (Watch 10).

**Steps.**

1. **Test boilerplate.**

   ```rust
   #[actix_rt::test]
   async fn sponsor_liability_with_founder_multiplier() -> Result<(), Box<dyn Error>> {
     // Per governance_fixtures pattern — spin up testcontainer with
     // `--user $(id -u):$(id -g)` avoidance (Phase 4 Docker root-owned files gotcha).
     // Apply all migrations via schema_setup::run.

     std::env::set_var("BREHON_DISABLE_SNAPSHOT_JOB", "1");  // disable clokwerk; call recompute_snapshot manually

     let (context, _guard) = governance_fixtures::test_context().await?;
     let pool = context.pool();
     let mut conn = get_conn(pool).await?;

     branch_default_multiplier(&context, &mut conn).await?;
     branch_founder_chain_survival(&context, &mut conn).await?;
     branch_honour_price_floor_clamp(&context, &mut conn).await?;

     // Watch 10 PII grep assertion (see step 5).
     assert_no_raw_identifiers_in_governance_log(&mut conn).await?;

     Ok(())
   }
   ```

2. **Branch 1 — `default_multiplier`.**

   - Create users A (target), B (regular sponsor, 40d old), C (founder sponsor, 40d old). Phase 4's `governance_fixtures` has a person-builder; reuse it. Set `person.published` to `now - 40d` on B and C so `account_age_days ≥ 30` (sponsor age gate).
   - Invoke the founder CLI for C. At plan-write time, the CLI is a binary — the test can either (a) shell out with `std::process::Command::new("cargo").args(&["run", "-p", "brehon_seed_founders", "--", ...])` OR (b) call the CLI's `main`-equivalent library function directly. **Plan-write-time choice: (b)** — extract the CLI body into a `pub fn seed_founders(args: SeedArgs) -> Result<...>` inside `brehon_seed_founders` and have the test dep on the crate as a lib target in addition to the bin. Cheaper, faster, no subprocess.
   - Have B and C `create_endorsement` for A (B endorses first, C second, both within the same test run).
   - Force a case — direct DB insert into `moderation_case` with `status: CaseStatus::ThresholdMet`, `target_person_id: Some(a.id)`, etc. (Phase 4's test uses the same pattern at `e2e.rs:800ish`; reuse.)
   - Call `admin_assign_jury` with a jury pool seeded beforehand — 5 eligible users with `jury_eligible = true` in their snapshot, none of whom are A/B/C. Verify `fallback_on_small_pool` is not triggered by logging.
   - Submit 3 votes (jurors 1/2/3) for `ContentRemoval` (moderate). Jurors 4/5 may submit `NoAction` or not at all — quorum is 3, majority wins.
   - After the 3rd vote, the case flips `Decided`.
   - **Assertions:**
     - B's newest `reputation_event` (dimension=EndorsementStrength, source_case_id=case.id) has `delta = -25` (`-50 / 2 sponsors × 1.0 regular_multiplier = -25`).
     - C's newest `reputation_event` has `delta = -50` (`-50 / 2 sponsors × 2.0 founder_multiplier = -50`).
     - `governance_log` contains 2 `sponsor_liability_applied` entries, one with `multiplier=1.0` and one with `multiplier=2.0`.
     - After calling `recompute_snapshot(conn, c.id, community, &mut cache).await?`, C's snapshot has `endorsement_strength = 50` (100 seed + -50 liability).
     - After calling `recompute_snapshot(conn, b.id, community, &mut cache).await?`, B's snapshot has `endorsement_strength = -25` (0 baseline + -25 liability). Floor is 0 — B would be clamped if its baseline were 5 (see branch 3).
   - Then **flip `config.liability.founder_multiplier` from 2.0 to 3.0** via a direct `INSERT INTO governance_config (scope, key, value_type, value_float, valid_from, ...)` with `valid_from = now()` so the `governance_config_current` view returns the new value. Run a second sanction round (new case against a new user D, same sponsor structure). Verify the new `reputation_event` from C has `delta = -75` (`-50 / 2 × 3.0 = -75`).

3. **Branch 2 — `founder_chain_survival` (per design-review Issue F).**

   - Create 2 founders C1 + C2 via CLI, each seeded `endorsement_strength = 100, expires_at = now+90d`.
   - Both sponsor user D.
   - Sanction D with `CommunityExclusion` (severe, -200 raw delta).
   - Per the math: `raw_delta = -200`, 2 sponsors → `per_sponsor_base = -100` each. Both are founders → `×2.0 = -200` each.
   - **With `sponsor_liability_floor = 0`:** both founders have current_endorsement_strength = 100; 100 + -200 = -100 < 0; clamp to `final_delta = 0 - 100 = -100`. Both founders' post-event `endorsement_strength = 0`.
   - **Assertion:** "both founders should retain `can_sponsor=true` under default config" — which requires `endorsement_strength ≥ config.thresholds.endorsement_strength = 25`. BUT with the floor clamp at 0 applied, both founders end at `endorsement_strength = 0`, which is BELOW 25. **This means `can_sponsor` goes FALSE for both founders.**
   - **Per plan line 346**: "both founders should retain `can_sponsor=true` under default config (i.e. their post-sanction endorsement_strength stays above the 25 threshold). If they don't, the default `sponsor_liability_severe = -200` and `founder_multiplier = 2.0` defaults need flagging in the 5b retro for v1 tuning."
   - **Implementation choice:** assert what the math actually produces (`can_sponsor = false` for both founders after the floor clamp — because default severity + default multiplier + default floor cannot survive; the defaults are not "survival" tuned). **Flag this in the 5b retro** per plan line 346. Do NOT "fix" the defaults in the plan — OQ-024 resolution accepted the floor; OQ-003 accepts the Restoration slot; the surviving-founder-chain story requires tuning, which is a v1 config-write concern.
   - **Revised assertion for branch 2:** verify the math produces the expected CLAMPED outcome, record the failure-of-aspiration in the test body comments, and add a `println!("FOUNDER_CHAIN_SURVIVAL: post-sanction endorsement_strength for C1={}, C2={}; default config does NOT preserve can_sponsor=true; retro follow-up for v1 tuning", ...)`. The test PASSES (assertions match the math) but surfaces the design question.

4. **Branch 3 — `honour_price_floor_clamp` (per OQ-024).**

   - Create regular sponsor E (no founder seed, minimal accrual). Seed `endorsement_strength = 5` via a `reputation_event` row with `dimension = EndorsementStrength, delta = 5, expires_at = None`. Call `recompute_snapshot` to materialise the snapshot row.
   - Create user F.
   - E endorses F (sole sponsor).
   - Sanction F with `ContentRemoval` (moderate, -50 raw delta).
   - Per the math: `raw_delta = -50`, 1 sponsor → `per_sponsor_base = -50` (no remainder). Regular multiplier = 1.0 → `post_multiplier = -50`. Current `endorsement_strength = 5`. `5 + -50 = -45 < 0` (floor). Clamp: `final_delta = 0 - 5 = -5`.
   - **Assertions:**
     - E's newest `reputation_event` has `delta = -5` (not -50, not -45).
     - `governance_log` contains 1 `sponsor_liability_applied` entry with `final_delta = -5`.
     - `governance_log` contains 1 `sponsor_liability_clamped` entry with `uncapped_delta = -50`, `clamped_delta = -5`, `floor = 0`, `current_endorsement_strength = 5`.
     - After `recompute_snapshot`, E's `endorsement_strength = 0` (not -45).
     - E's `can_sponsor = false` (0 < threshold 25) — the disablement mechanic fires, the permanent-outcast state does not (OQ-024 confirmed).

5. **Watch 10 PII grep assertion.**

   ```rust
   async fn assert_no_raw_identifiers_in_governance_log(conn: &mut AsyncPgConnection) -> Result<(), Box<dyn Error>> {
     use regex::Regex;
     let banned = Regex::new(r#""(person_id|target_person_id|sponsored_id|sponsor_id)"\s*:\s*\d+"#)?;
     let rows: Vec<serde_json::Value> = governance_log::table
       .select(governance_log::payload)
       .load::<serde_json::Value>(conn).await?;
     for payload in rows {
       let serialized = serde_json::to_string(&payload)?;
       if banned.is_match(&serialized) {
         return Err(format!("governance_log payload contains raw identifier: {serialized}").into());
       }
     }
     Ok(())
   }
   ```
   Matches `"person_id": 42` but NOT `"admin_pseudonym": "abc-uuid"` (the latter is the UUID, not an integer). Per plan: "Pseudonyms look like UUIDs; raw IDs are short integers — easy to differentiate."

6. **Return type.** Per feedback `feedback_clippy_test_style.md`: `-> Result<(), Box<dyn std::error::Error>>`. Use `.map_err(|e| format!("{e}").into())` to bridge `LemmyError` into `Box<dyn std::error::Error>` per feedback memory.

7. **Testcontainer user flag.** Per Phase 4 pattern — use `governance_fixtures::test_context` which already handles the Docker `--user` flag (5a deviation: existing fixture handles it).

**GOTCHA-60a (multiplier type).** `liability.founder_multiplier` is `float` in config (verified at plan-write time, config.rs:428). `delta * multiplier` is f64-multiply → round-half-to-even → i64. If the test branch flips the multiplier to 3.0 and gets -75, the math matches task 56's pseudocode. If the test expects -50 but gets -51 (rounding), that's a bug in task 56's rounding direction.

**GOTCHA-60b (snapshot freshness timing).** The test calls `recompute_snapshot` manually after each sanction round because `BREHON_DISABLE_SNAPSHOT_JOB=1` disables the clokwerk tick. `apply_sponsor_liability` does NOT call `recompute_snapshot` itself — per GOTCHA-56e, the liability function only writes the `reputation_event` row. The test's assertion on the snapshot's `endorsement_strength` requires an explicit recompute between the sanction and the assertion.

**GOTCHA-60c (branch 1 config edit midway).** Direct `INSERT INTO governance_config ... valid_from = now()` with a SECOND row for `liability.founder_multiplier` (the first is from the seed) — the `governance_config_current` view returns the row with the latest `valid_from`. Verify the view definition in `migrations/2026-04-18-000000-0000_add_governance_config/up.sql` matches this behaviour. Per 5a task 50 the view is defined as such.

**GOTCHA-60d (test isolation between branches).** The three branches share one testcontainer + DB. Between branches, the test should NOT truncate tables — it's fine to accumulate state because each branch uses fresh person IDs and asserts on its own specific reputation_event rows via `source_case_id = case.id`. The PII grep at the end surveys ALL governance_log rows from ALL three branches — this is correct.

**Commit.** `test(governance): task 60 — sponsor_liability_with_founder_multiplier e2e (default + founder_chain_survival + honour_price_floor_clamp)` (body ≤60 words).

---

### §11.6 Task 61 — Phase-close validation + completion report + PR

**Goal.** Run every §12 validation command, write the completion report, open the PR `phase-5b → governance-v0`.

**Steps.**

1. **Run every §12 validation command.** Capture to files; tail only. Confirm all Level 0–5 green.

2. **Run regression guards explicitly** (three-layer defence):
   ```bash
   cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- report_to_modlog_golden_path > .claude/build-task61-golden.log 2>&1"
   status=$?; tail -30 .claude/build-task61-golden.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

   cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- config_parity_round_trip > .claude/build-task61-parity.log 2>&1"
   status=$?; tail -15 .claude/build-task61-parity.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

   cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- sponsor_liability_with_founder_multiplier > .claude/build-task61-5b-e2e.log 2>&1"
   status=$?; tail -30 .claude/build-task61-5b-e2e.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

   bash scripts/brehon/lint-no-membership-read.sh; echo "membership guard exit: $?"
   bash scripts/brehon/lint-no-can-sponsor-read.sh; echo "can_sponsor guard exit: $?"
   ```

3. **Completion report** at `.claude/PRPs/reports/phase-5b-complete-report.md`. Structure: (a) delivered vs plan; (b) commit list + SHAs; (c) deviations from plan (especially: GOTCHA-56b enum struct-variant path chosen; Path A/B for config threading; founder chain survival math outcome); (d) decision-queue entries closed (#11, #12) and any new ones opened; (e) carry-forward into 5c (note branch 2 founder-chain-survival non-survival flag for v1 config tuning retro); (f) retro nomination (short form per advisor rule 12, full if any checkpoint fires — sponsor-liability math is the advisor's flagged retro site per rule 12).

4. **Open PR.**
   ```bash
   git push -u origin phase-5b
   gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-5b \
     --title "Phase 5b — Sponsor-liability, jury gating, founder bootstrap" \
     --body "$(cat <<'EOF'
   ## Summary
   - Sponsor-liability helper with founder-multiplier + honour-price floor clamp (OQ-022, OQ-024)
   - SanctionAction::Restoration variant (OQ-003 amended) + ALTER TYPE migration
   - Reputation gating + concurrent-cap in admin_assign_jury (OQ-004)
   - OQ-006 threshold formula in create_report with is_finite() guard
   - Founder seeding CLI (crates/tools/seed_founders)
   - sponsor_liability_with_founder_multiplier e2e test (3 branches)

   ## Completion report
   `.claude/PRPs/reports/phase-5b-complete-report.md`

   ## Plan reference
   `.claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md`
   Design docs (homeserver):
   `docs/research/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` §3 Phase 5b

   ## Regression guard
   - Phase 4 `report_to_modlog_golden_path` passes (Watch 4)
   - Phase 5a `config_parity_round_trip` passes (Watch 1 regression; narrowed per decision-queue #15)
   - Zero clippy warnings under `--features full --workspace --no-deps -- -D warnings`
   - Watch 10 PII grep over all governance_log payloads yields zero matches
   EOF
   )"
   ```

**Commit.** `docs(report): Phase 5b complete — sponsor-liability + founder bootstrap + OQ-006 threshold formula` (body cites DoD completion and retro nomination).

---

## §12. Validation commands (Level 0–5)

Use the wrappers exclusively. Every invocation → file → `$?` → tail per `.claude/rules/cargo-output-capture.md` + `.claude/rules/no-cargo-output-paste.md`.

### Level 0 — PRE-TASK_BASELINE (task 0 DoD)

- `git branch --show-current` returns `phase-5b`.
- All three wrapper probes exit 0.
- Decision-queue #11 and #12 moved from `pending` to `resolved`.
- No carry-patches required (unlike 5a's pagination.rs fix — dry-run at plan-write time confirmed no Level-0 failure).

### Level 1 — STATIC_ANALYSIS

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/l1-check.log 2>&1"
status=$?; tail -20 .claude/l1-check.log; echo "check exit: $status"; [ $status -eq 0 ] || exit 1

cmd //c "scripts\\brehon\\cargo-clippy.bat --features full --workspace --no-deps -- -D warnings > .claude/l1-clippy.log 2>&1"
status=$?; tail -40 .claude/l1-clippy.log; echo "clippy exit: $status"; [ $status -eq 0 ] || exit 1
```

**EXPECT.** Exit 0, zero errors, zero warnings. Wrapper exit-code propagation verified in 5a (plan §17.3 notes); no change in 5b.

### Level 2 — UNIT/INTEGRATION TESTS

```bash
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- config_parity_round_trip > .claude/l2-parity.log 2>&1"
status=$?; tail -15 .claude/l2-parity.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- report_to_modlog_golden_path > .claude/l2-golden.log 2>&1"
status=$?; tail -30 .claude/l2-golden.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- sponsor_liability_with_founder_multiplier > .claude/l2-5b-e2e.log 2>&1"
status=$?; tail -40 .claude/l2-5b-e2e.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

# Full e2e suite — catches drift from unrelated tests.
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e > .claude/l2-full-e2e.log 2>&1"
status=$?; tail -60 .claude/l2-full-e2e.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
```

**EXPECT.** All existing Phase 4+5a tests pass; new `sponsor_liability_with_founder_multiplier` passes (all 3 branches); total e2e test count increases by exactly 1.

### Level 3 — FULL_BUILD

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/l3-compile.log 2>&1"
status=$?; tail -15 .claude/l3-compile.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

cmd //c "scripts\\brehon\\cargo-check.bat -p brehon_seed_founders > .claude/l3-cli.log 2>&1"
status=$?; tail -15 .claude/l3-cli.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
```

### Level 4 — MIGRATION_VALIDATION

```bash
# Restoration migration first-line assertion (§17.2 carry-forward 1 DoD).
head -1 migrations/$(ls migrations/ | grep restoration_sanction_variant)/up.sql | grep -Fx '-- no-transaction'
echo "no-transaction header exit: $?"

# phase1_migrations_round_trip — task 50 in 5a bumped PHASE_1_MIGRATION_COUNT to 8 per deviation #6; 5b adds a 9th. Bump the constant.
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- phase1_migrations_round_trip > .claude/l4-round-trip.log 2>&1"
status=$?; tail -20 .claude/l4-round-trip.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
```

**GOTCHA-L4a.** Per 5a deviation #6: `PHASE_1_MIGRATION_COUNT` was bumped 6→8. Task 56's migration is the 9th (or Nth+1). Update the constant in `e2e.rs` alongside the migration commit.

### Level 5 — CROSS_CUTTING_VERIFICATION

```bash
# 5a Watch 7 / sibling guards — regression check.
bash scripts/brehon/lint-no-membership-read.sh; echo "membership guard exit: $?"
bash scripts/brehon/lint-no-can-sponsor-read.sh; echo "can_sponsor guard exit: $?"

# Watch 10 — PII grep over 5b new files.
grep -nE '"(person_id|target_person_id|sponsored_id|sponsor_id)"' \
  crates/api/api/src/governance/sponsor_liability.rs \
  crates/tools/seed_founders/src/main.rs
echo "PII grep exit: $? (expect 1 — no matches)"

# Watch 3 — exhaustive match on SanctionAction.
grep -nE 'SanctionAction::(Label|VisibilityReduction|TemporaryRestriction|ContentRemoval|CommunityExclusion|InstanceSuspension|FederationQuarantineRecommendation|Restoration)' crates/api/api/src/governance/sponsor_liability.rs | wc -l
echo "variant arm count (expect ≥8 — all 8 variants named)"
```

---

## §13. Testing strategy

### 13.1 New tests in 5b

- **`sponsor_liability_with_founder_multiplier`** (task 60) — one e2e test function, three internal branches. This is THE 5b DoD test per IMPLEMENTATION-PLAN-v0.md line 346.
- **`create_report::tests::is_finite_fallback`** (task 58 GOTCHA-58b) — inline unit test; asserts fallback value, `error!` log capture, handler-returns-Ok semantics per §17.2 carry-forward (3).

### 13.2 Compile-time tests

- The 5a `parity::seeded_keys_count_matches_const_count` passes unchanged (5b does not add seed keys).

### 13.3 Regression guards

- `postgres_container_boots` (Phase 0)
- `can_insert_moderation_case` (Phase 1)
- `governance_log_hash_chain_holds` (Phase 1)
- `phase1_migrations_round_trip` (Phase 1) — `PHASE_1_MIGRATION_COUNT` bumps +1 per 5b migration.
- `list_open_cases_returns_seeded_rows` (Phase 2a)
- `jury_queue_view_returns_assignments` (Phase 2a)
- `modlog_view_returns_published_entries` (Phase 2b)
- `redaction_strips_identifiers` (Phase 4)
- `report_to_modlog_golden_path` (Phase 4b) — **critical**; Watch 4.
- `config_parity_round_trip` (Phase 5a)

All 10 must stay green.

### 13.4 What's NOT tested in 5b

- **Capability-gating e2e** (`ineligible_user_cannot_be_picked_for_jury`) — Phase 5c task 69.
- **All MVP endpoints 200 on happy-path smoke test** — Phase 5c task 68.
- **Snapshot-staleness alert** — Phase 5c per design-review B1.
- **Post-target sponsor-liability path** — carry-forward from Phase 4b decision-queue #10. Surfaces in 5c (task 57 target-inference fix) or v1.
- **Concurrent endorsement + snapshot recompute race** — Phase 5a GOTCHA acknowledged; task 60's honour_price_floor_clamp branch exercises the clamp path transitively but does not drive concurrent writes.

---

## §14. Risk register

| ID  | Risk                                                                                       | Likelihood | Impact | Mitigation                                                                                                                                                    |
|-----|--------------------------------------------------------------------------------------------|------------|--------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| R1  | `SanctionAction::Restoration { description: String }` breaks `DbEnum` derive (GOTCHA-56b)  | Medium     | Medium | Fallback to unit `Restoration` + separate `sanction.restoration_description` column migration. Document in task 56 commit body.                               |
| R2  | Config threading into `run_transaction` closure requires pool-outside pattern              | Medium     | Low    | Path B (pre-warm cache before transaction). Decision at impl time; both paths sketched in §11.1 step 5.                                                       |
| R3  | Founder-chain-survival math (branch 2) produces clamped-to-zero, can_sponsor=false         | High       | Low    | Branch 2 assertion adjusted to match math; non-survival flagged for v1 retro per plan line 346. Test passes; design question surfaced.                        |
| R4  | Diesel DSL subquery with `HAVING` for concurrent-cap (task 57) fails to compile            | Medium     | Low    | Fallback to `sql_query` raw SQL; 1 ralph iteration budget. Grep for similar pattern in 5a task 53 confirms `sql_query` is an acceptable fork-local pattern.  |
| R5  | `load_or_compute_snapshot` helper (task 58) cascades into snapshot writes during `create_report` transaction | Low        | Medium | Helper reads existing row first; only calls `recompute_snapshot` if missing. 5a's snapshot writes are transaction-safe per deviation #7 branchful-SELECT pattern. |
| R6  | Founder CLI dep on `lemmy_api` full feature causes compile-time churn in workspace         | Low        | Low    | CLI reuses existing exported helpers only; no new pub surfaces in `lemmy_api`.                                                                                |
| R7  | Watch 10 PII grep (task 60) matches pseudonym uuid containing digits as a raw id           | Low        | Medium | Regex anchors on `"<name>"\s*:\s*\d+` — integer-only; UUIDs contain hyphens and are quoted as strings (`"abc-uuid"` → `\s*:\s*"`). False-positive risk low.  |

---

## §15. Acceptance criteria

- [ ] `phase-5b` branch cut from `governance-v0 @ 5a4a0f0a5` before the first commit.
- [ ] Task 0 audit completed; decision-queue #11 + #12 resolved.
- [ ] Tasks 56–60 each land as one commit (task 56 may be the largest; follow one-commit-per-task rule).
- [ ] Task 61 lands `docs(report)` commit.
- [ ] Every Level 0–5 validation command exits 0 on task-61 HEAD.
- [ ] `report_to_modlog_golden_path` passes at task 56 HEAD AND task 57 HEAD AND task 58 HEAD AND task 61 HEAD (four measurement points; Watch 4).
- [ ] `sponsor_liability_with_founder_multiplier` passes all three branches.
- [ ] `config_parity_round_trip` passes (Watch 1 regression; narrowed per decision-queue #15).
- [ ] Watch 10 PII grep on new files yields zero matches.
- [ ] `SanctionAction::Restoration` exists in the enum; Postgres migration applied with `-- no-transaction`; `grep -Fx '-- no-transaction' migrations/.../up.sql` exits 0 on first line.
- [ ] The 15 ADRs in [99] remain uncontradicted; no ADR amendments in 5b.
- [ ] PR open via `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-5b`; CodeRabbit auto-review fires.

---

## §16. Plan correction policy

Per advisor rule 20 (grep-before-commit) and feedback `feedback_plan_dod_dry_run_at_write.md`:

1. **Plan errata discovered mid-loop land as `docs(plan): ...` commits** on `phase-5b`, NOT as patch-in-situ. Example: if task 56 GOTCHA-56b's struct-variant path proves impossible and the fallback is chosen, amend this plan file with a `docs(plan): pivot task 56 to unit-variant Restoration + sibling column migration` commit before the task 56 functional commit. Keeps the plan and the code truthful together.

2. **Decision-queue entries opened mid-loop on advisor-scale questions.** The CodeRabbit carry-over (`Scope::as_str` allocation) is pre-resolved here; new mid-loop questions at that scale go to the queue.

3. **Never downgrade watchpoint mitigations to avoid surfacing a bug.** If a task discovers a Watch violation inherited from Phase 4 (e.g. decision-queue #10 Post-target inference), document as carry-forward; do not silently weaken the assertion.

4. **Level-0 gate predicate style (per 5a retro).** Prefer negative-space assertions ("stale `#[expect]` absent") over positive-space assertions ("`#[allow]` present") when the specific fix form is permitted to deviate. Applies to any grep gate added in 5b.

---

## §17. Notes + decision-queue intake

### 17.1 Decision-queue intake (pre-seeded for 5b plan)

Already resolved in this plan's body; close at task 0 step 4:

- **#11** — juror cap = 3. **Resolution:** seeded via `config.jury.max_concurrent_assignments = 3` (5a task 50); task 57 reads the key via `ConfigCache`. `answered_by: "planner"`.
- **#12** — sponsor-liability severity units (minor=-10, moderate=-50, severe=-200). **Resolution:** confirmed per IMPLEMENTATION-PLAN-v0.md §3 Phase 5b task 55 + §17.2 carry-forwards. Mapping per §11.1 GOTCHA-56a: `Label` + `VisibilityReduction` + `Restoration` → minor; `TemporaryRestriction` + `ContentRemoval` → moderate; `CommunityExclusion` + `InstanceSuspension` + `FederationQuarantineRecommendation` → severe. Config keys `deltas.sponsor_liability_{minor,moderate,severe}` (seeded in 5a). `answered_by: "planner"`.

### 17.2 CodeRabbit re-review carry-over resolution (from Phase 5a advisor §1)

- **`Scope::as_str` allocation refactor.** Bundled into task 56 step 0 (pre-task refactor; see §11.1). Resolved at plan-write time. Not a separate decision-queue entry.

### 17.3 Open questions that emerge at plan-write time (non-blocking)

- **Branch 2 (founder_chain_survival) default-config non-survival.** Per §11.5 branch 2: default `sponsor_liability_severe = -200` + `founder_multiplier = 2.0` + `sponsor_liability_floor = 0` → clamped-to-zero → `can_sponsor = false` for the founder. This contradicts plan line 346's aspirational "both founders should retain `can_sponsor=true` under default config". The math does NOT produce survival with default config. **Choice:** test branch asserts the mathematical outcome and surfaces the gap via `println!` + retro nomination for v1 tuning. Not a plan-blocking issue; test passes.

- **Phase 4b decision-queue #10 (Post-target inference).** Not fixed in 5b. Task 57's eligibility filter still depends on `case.target_person_id` which is None for Post/Comment cases. Carry-forward to 5c task 69 or v1 admin_assign_jury patch.

- **Task 58 unit test location (inline vs e2e).** `create_report::tests::is_finite_fallback` lands inline in `create_report.rs`. Integration-sized e2e coverage rides on task 60's other branches (the is_finite fallback is unit-testable without a container).

### 17.4 What the task 61 completion report must include

- Exit codes for all §12 levels, captured from log tails (not task-notification summaries per 5a §17.3 notification-bug warning).
- Commit SHAs for tasks 56–60 + task 61.
- Deviations (expected: GOTCHA-56b enum-variant shape; Path A/B config-threading decision; founder chain survival retro flag).
- Decision-queue entries closed (#11, #12) and any new ones opened.
- Carry-forward into 5c:
  - Post-target sponsor-liability (decision-queue #10 from 4b).
  - Founder-chain-survival config tuning (§17.3 above).
  - Any task-57 fallback-to-raw-SQL for Diesel DSL subquery if R4 fires.
- Retro nomination (short form default; full form if any checkpoint fires — sponsor-liability math is explicitly flagged by advisor rule 12 as a retro-worthy site per advisor-context-phase-5.md §5 rule 12).
