# Plan: v1-RT-r1 — reputation-tuning schema + 26 net-new seed keys + sponsor_allowlist extension + 7 entry-kind consts + backfill

## 1. Summary

v1-RT-r1 is the **schema + seed foundation** of the v1 reputation-tuning lane (PRD `.claude/PRPs/prds/v1-reputation-tuning.prd.md` §15 row 1). It ships the substrate r2/r3/r4/r5/r6 will read and write against; it ships **no handler code**. Concretely it lands:

1. **One column-add migration** on `reputation_event` — new `dedupe_key TEXT` nullable (with partial unique index `WHERE dedupe_key IS NOT NULL`) + new `source_event_type` Postgres enum + matching `reputation_event.source_event_type` column NOT NULL DEFAULT `'Endorsement'`.
2. **One ALTER migration** on the existing `sponsor_allowlist` table (shipped pre-v1-AD-a per DQ #181 — RT-r1 EXTENDS, does NOT create): drop `community_id NOT NULL`, add `added_by_admin_id INTEGER NOT NULL REFERENCES person(id)`, add `note TEXT`. Existing `created_at` stays; existing `UNIQUE (community_id, person_id)` stays.
3. **One backfill migration** on existing `reputation_event` rows: populate `source_event_type` from a `reason` ILIKE precedence chain per DQ #184 (sponsor_liability% -> SponsorLiability; jury_reliability% -> JuryVote; founder_seed% -> FounderSeed; otherwise default `Endorsement` from the column DEFAULT).
4. **One seed migration** with **26 net-new** `governance_config` rows (per planner DQ #187 below: 29 PRD §8 rows minus 3 already-shipped under v1-AD-a — `deltas.participation_weekly_active`, `participation.dormancy_window_days`, `deltas.participation_dormant`).
5. **`EXPECTED_SEED_COUNT_V1_RT: usize = 26`** parametric const + parity-test extension at `crates/api/api/src/governance/config.rs:2692-2707`. Cumulative invariant: `34 + 27 + 27 + 13 + 26 = 127`.
6. **One Diesel-backed Rust enum** (`ReputationEventSourceType`, 9 variants per PRD §7) in `crates/db_schema_file/src/enums.rs` mirroring `ReputationDimension`'s `verbatim` `DbValueStyle`.
7. **`schema.rs` extensions**: new `sql_types::ReputationEventSourceType` struct; `reputation_event` table block extended with `dedupe_key + source_event_type`; `sponsor_allowlist` table block extended with `community_id` made nullable + `added_by_admin_id + note`.
8. **Diesel struct extensions**: `ReputationEvent` + `ReputationEventInsertForm` get the 2 new fields; `SponsorAllowlist` + `SponsorAllowlistInsertForm` get the 3 ALTER fields. Newtypes unchanged (per DQ #181).
9. **7 new `ENTRY_KIND_*` consts** dual-file edit (per `.claude/rules/governance-log-entry-kind-registry.md`): DEFINE in `crates/db_schema/src/source/governance/governance_log.rs`, RE-EXPORT in `crates/api/api/src/governance/governance_log.rs`. Names: `PARTICIPATION_CRON_TICK`, `VOTE_OUTCOME_RECORDED`, `EVIDENCE_QUALITY_RECORDED`, `ROLLUP_RECOMPUTED`, `DECAY_KNOB_CHANGED`, `SPONSOR_ALLOWLIST_ADDED`, `SPONSOR_ALLOWLIST_REMOVED`. All 7 are pre-landed-const exempt (call sites land in r2/r3/r4/r5).
10. **`reputation-tuning-v1` registry section populated** (replacing the current reservation stub at `.claude/rules/governance-log-entry-kind-registry.md` reputation-tuning-v1). Acceptance-invariants count bumped from **38 -> 45**.
11. **e2e migration round-trip extension**: `PHASE_1_MIGRATION_COUNT` bumped from 14 -> 18 (RT-r1 adds 4 migrations); post-condition probe lists in `phase1_migrations_round_trip` extended.

**No handler edits** — `crates/api/api/src/governance/<handler>.rs` files are untouched (per brief §2.2 + §4.1.b). The 7 entry-kind consts are declared but **not emitted** in r1 — emitting call sites land in r2/r3/r4/r5 per the registry rule's pre-landed-const exemption.

Headline acceptance: phase-branch tip passes `cargo-validate-workspace.yml` + `cargo-validate-migration.yml` on Junior worker pushes; the `every_seeded_key_has_metadata` + `seeded_keys_count_matches_const_count` parity tests stay green; the `phase1_migrations_round_trip` e2e test asserts the four new migrations apply + revert + re-apply cleanly; the registry-rule's `rg '^pub const ENTRY_KIND_'` invariant goes 38 -> 45.

## 2. Source

- `.claude/PRPs/briefs/rt-r1-planning-1.md` @ `0e9eb082a` — the advisor brief (post-`/brehon-clarify`; DQ #181-#185 resolved 2026-05-10 by advisor self-answer with citations).
- `.claude/PRPs/prds/v1-reputation-tuning.prd.md` @ `governance-v0` — parent PRD; §1 problem, §2 evidence, §5.1 dimension catalog, §5.2 decay knobs, §5.3 multi-source participation events (resolves OQ-019), §5.4 sponsor-gate strategies (RT-r1 ships `sponsor_allowlist` table only), §5.5 instance-wide rollup (r5), §7 cross-cutting impact (the 7 new entry-kinds + schema additions + backfill rows), §8 28-knob defaults matrix, §9 backwards-compat + §9.1 ADR-010 feature-flag posture, §10 security, §15 phase split row 1.
- `.claude/PRPs/prds/v1-admin-dashboard.prd.md` §10 + §15.5 — `seed_v1_config_keys` pattern + `EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD` parametric pattern that RT-r1 extends.
- `.claude/decision-queue.json` resolved entries #181, #182, #183, #184, #185 — pre-planning clarify gate (advisor self-answered 2026-05-10).
- `.claude/decision-queue.json` resolved entry #186 — planner-self-resolved `kind: log` filed by THIS plan author flagging the 3 v1-AD-a seed-key duplicates and the V1_RT count revision (29 -> 26).
- ADRs: ADR-005 (multi-dimensional reputation), ADR-008 (governance log append-only — entry_kind is TEXT), ADR-010 (staged release v0 -> v1 — RT-r1 weakens to feature-flag posture per PRD §9.1), ADR-013 (EmergencyRemove), ADR-015 (pseudonymisation).
- `.claude/rules/governance-log-entry-kind-registry.md` — dual-file rule + pre-landed-const exemption + count invariant (38 -> 45 post-RT-r1) + the `reputation-tuning-v1 (reserved)` section that Task 9 populates.
- `.claude/rules/decision-queue.md` — schema-v2 attribution + Recipe 2 (planner-self-resolved log shape).
- `.claude/rules/advisor-orchestrator.md` — Stage shape + cohort dispatch + §G4 classifier + Phase 2 e2e user gate.
- `.claude/rules/branch-manager.md` + `.claude/rules/phase-branch.md` — phase-branch discipline.
- `.claude/PRPs/templates/plan.template.md` — 20-section schema.

### Lessons that bind §13 decisions

- `feedback_lemmy_migration_runner.md` — use `cargo run -p lemmy_diesel_utils --features full -- migration run/revert`; never raw `diesel migration run`. Binds Tasks 1, 2, 3, 4.
- `feedback_postgres_jsonb_canonicalization.md` — RT-r1 ships zero JSONB columns; cite-only.
- `feedback_advisor_watchpoint_specificity.md` — every §4 watchpoint cites a concrete file/table/`schema.rs` line.
- `feedback_complexity_score_pre_split.md` — RT-r1 mirrors JM-a / SL-a schema-only-sub-phase precedent (proceed-as-one).
- `feedback_explicit_file_arrays_on_tasks.md` — every §13 task carries a FILES YAML block.
- `feedback_parallel_cohort_dispatch.md` — Cohort A is 4-way parallel; Cohort B is 3-way parallel.
- `feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md` — advisor runs every §15 DoD literally before plan approval.
- `feedback_features_full_p_crate_incompatible.md` — never `-p <crate>` + `--features full`. Workflow YAMLs use `--workspace --features full`.
- `feedback_features_full_workspace_only.md` — `--features full` activates `DbEnum` + `ts-rs` derives.
- `feedback_insertform_default_propagation.md` — `ReputationEventInsertForm.source_event_type: Option<>` (NOT-NULL DB DEFAULT covers callers); `SponsorAllowlistInsertForm.added_by_admin_id: PersonId` (required — RT-r4 endpoints set it; v1-AD-a callers do not exist).
- `feedback_clippy_test_style.md` — R1 every `i32 <-> i64` uses `i64::from(...)`. Bound in §4.
- `feedback_lemmy_error_no_std_error.md` — RT-r1 only EXTENDS `phase1_migrations_round_trip` (existing test fn returning `Result<(), Box<dyn Error>>`); helper closures use Case B annotated closure.
- `feedback_async_pool_test_pattern.md` — cite-only.
- `feedback_junior_worker_e2e_edit_hang.md` — only 1 e2e edit task (Task 10); single Edit-with-anchor.
- `feedback_newtype_locations_lemmy_db_schema_vs_file.md` — cite-only: zero new newtypes per DQ #181.
- `feedback_brehon_verify_pre_merge.md` — §16a Stories grain enables `/brehon-verify` phantom check.
- `feedback_principles_not_rules.md` — score >8 is signal; planner leans proceed-as-one.
- `feedback_pr_per_phase.md` — one PR per sub-phase; one commit per task.
- `feedback_read_canonical_before_writing_spec.md` — cite-only: RT-r1 does not add new commands/rules/lessons/templates.
- `feedback_schema_changing_spec_retrofit_question.md` — RT-r1 is purely additive; skip the retrofit gate.

### Related prior plans (canonical-shape mirror)

- `.claude/PRPs/plans/phase-v1-JM-a.plan.md` — closest-shape precedent. Schema-only first sub-phase of a lane (3 enums + 8 columns + 1 new table + 27 seed keys + 6 ENTRY_KIND consts + parametric `EXPECTED_SEED_COUNT_V1_JM`). RT-r1 mirrors §13 task ordering + §10 patterns + §17 completion checklist.
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — second canonical sibling. Schema-only first sub-phase with feature flag; same `EXPECTED_SEED_COUNT_V1_SL` parametric pattern. RT-r1 mirrors §15.6 Shape-G shape, §16a Stories grain, §13 FILES YAML blocks + `[P]` cohort markers, §5.1 complexity breakdown table.
- `.claude/PRPs/plans/v1-admin-dashboard-a.plan.md` (under `completed/`) — original `EXPECTED_SEED_COUNT_V1_AD = 27` precedent.

## 3. Problem statement

The v1 reputation calculator (r2 — chained-halving per-dimension decay), the multi-source participation event emitters (r3 — weekly-active + dormancy crons + vote-outcome + evidence-quality), the sponsor-gate strategy expansions (r4 — `age_or_surety` / `reputation` / `allowlist`), and the instance-wide rollup cron (r5) all depend on schema and config substrate that v0 + v1-AD-a + v1-JM-a do not provide:

- **r2 calculator** reads `feature.reputation_v1_decay_enabled` to gate v0-vs-v1 dispatch (PRD §5.2 + §9). The flag does not exist as a `governance_config` row today.
- **r3 idempotent activity-cron emitter** writes `reputation_event` rows with `dedupe_key = format!("participation_cron:{community_id}:{iso_week}")` (PRD §5.3 source 1). The `dedupe_key` column does not exist; without the partial unique index, concurrent cron ticks would double-emit.
- **r3 emitter typing** writes `source_event_type` per emit so r2 calculator can apply per-source decay parameters (and eventually snapshot the calculator version per PRD §9.1 v2-scope). The enum + column do not exist.
- **r4 allowlist strategy** reads `sponsor_allowlist` rows (PRD §5.4 third strategy). The table exists in stub form (v1-AD-a), but the ALTER additions r4 needs (`community_id` nullable for instance-wide allowlist, `added_by_admin_id` for audit, `note` for admin context) are missing. The newtype + source struct already exist (per DQ #181 — RT-r1 EXTENDS).
- **r5 rollup cron** + r3 cron emitters + r2 decay-knob admin writes need 26 new tunables in `governance_config` plus the `feature.reputation_v1_decay_enabled` flag plus 7 entry-kind consts in the registry.
- **v0 mid-flight `reputation_event` rows** at v1 deploy time get no source-event-type classification — without the backfill, r2 calculator (when behind the flag-flip) cannot apply per-source decay because every existing row would default to `Endorsement` even when `reason` indicates otherwise.

Without RT-r1, every downstream r2/r3/r4/r5 sub-phase has nothing to read or write against. RT-r1 closes the gap in a single schema-only sub-phase. Per PRD §15 row 1 dependency: **OQ-018 admin config write endpoint** — verified shipped in v1-AD-b PR #76 merge (`f03ed1cba`); RT-r1 seeded keys will be writable through that endpoint at runtime.

## 4. Solution statement

Four migrations + one enum extension + schema.rs edits + Diesel struct extensions + 26 DEFAULT consts + 7 ENTRY_KIND consts + e2e round-trip extension + retro. No handler code; no scheduler; no calculator; no route wiring; no DTO.

The migrations land in lexicographic timestamp order at deploy time:

1. `migrations/2026-05-10-000000-0000_add_reputation_event_v1_columns/{up,down}.sql` — `CREATE TYPE reputation_event_source_type AS ENUM (...)`; `ALTER TABLE reputation_event ADD COLUMN dedupe_key TEXT`; `CREATE UNIQUE INDEX reputation_event_dedupe_key_partial_idx ON reputation_event (dedupe_key) WHERE dedupe_key IS NOT NULL`; `ALTER TABLE reputation_event ADD COLUMN source_event_type reputation_event_source_type NOT NULL DEFAULT 'Endorsement'`. Single transaction (no `-- no-transaction` directive).

2. `migrations/2026-05-10-000100-0000_extend_sponsor_allowlist_for_r1/{up,down}.sql` — `ALTER TABLE sponsor_allowlist ALTER COLUMN community_id DROP NOT NULL`; `ALTER TABLE sponsor_allowlist ADD COLUMN added_by_admin_id INTEGER NOT NULL REFERENCES person(id)` (with sentinel default 1 then DROP DEFAULT); `ALTER TABLE sponsor_allowlist ADD COLUMN note TEXT`.

3. `migrations/2026-05-10-000200-0000_backfill_reputation_event_source_type/{up,down}.sql` — `UPDATE reputation_event SET source_event_type = ... WHERE ...` per the DQ #184 5-step precedence chain.

4. `migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/{up,down}.sql` — 26 `INSERT INTO governance_config ... VALUES (...) ON CONFLICT (scope, key, valid_from) DO NOTHING`; LIFO `DELETE` in down.

Rust side mirrors:

- New enum `ReputationEventSourceType` in `crates/db_schema_file/src/enums.rs` after line 615. 9 variants: `Endorsement` (default), `JuryVote`, `SponsorLiability`, `FounderSeed`, `ParticipationCron`, `DormancyCron`, `VoteOutcome`, `EvidenceQuality`, `ManualSeed`.
- New `sql_types::ReputationEventSourceType` struct in `schema.rs:118+`.
- `reputation_event` `table!` block extension at `schema.rs:1218-1229`.
- `sponsor_allowlist` `table!` block extension at `schema.rs:1338-1343`.
- `ReputationEvent` + `ReputationEventInsertForm` extension at `crates/db_schema/src/source/governance/reputation_event.rs:17-42`.
- `SponsorAllowlist` + `SponsorAllowlistInsertForm` extension at `crates/db_schema/src/source/governance/sponsor_allowlist.rs:21-36`.
- 26 new `DEFAULT_*` consts + 26 new match arms + 26 new `SEEDED_KEYS_WITH_CONSTS` tuples + 26 new `CONFIG_KEY_METADATA` entries + `EXPECTED_SEED_COUNT_V1_RT: usize = 26` + parity-test extension at `crates/api/api/src/governance/config.rs`.
- 7 new `ENTRY_KIND_*` consts in `crates/db_schema/src/source/governance/governance_log.rs` (DEFINE) + 7 new `pub use` re-exports in `crates/api/api/src/governance/governance_log.rs` + populated `reputation-tuning-v1` section in `.claude/rules/governance-log-entry-kind-registry.md`.
- e2e migration round-trip extension — `PHASE_1_MIGRATION_COUNT` 14 -> 18.

Per PRD §9 backwards-compat: v0 single-half-life decay path keeps reading `decay.positive_half_life_days`; RT-r1 seeds `feature.reputation_v1_decay_enabled = false` so v1 calculator (r2-pending) is dormant at deploy. Operators flip via OQ-018 admin endpoint after evaluating defaults.

### 4.1 Rejected alternatives

- **Seed all 29 PRD §8 rows including the 3 v1-AD-a duplicates.** Rejected per planner DQ #187: parity test would fail.
- **CREATE a fresh `sponsor_allowlist` table.** Rejected per DQ #181: r1 ALTERs.
- **Migration-SQL seed pattern at `crates/db_schema/migrations/`.** Rejected: actual migration directory is repo-root `migrations/`.
- **Pure-Rust seed-utils module at `crates/db_schema/src/utils/v1_rt_config_seed.rs`.** Rejected per DQ #183.
- **Backfill heuristic using `endorsement_id` / `jury_vote_id` columns.** Rejected per DQ #184: those columns do not exist.
- **Add per-event `applied_delta_snapshot` column.** Rejected per PRD §9.1: v2-scope.
- **Bundle r2/r3 work into r1.** Rejected per PRD §15 row 1 scope.

## 5. Metadata

- **Phase:** `v1-RT-r1`
- **Branch:** `phase-v1-RT-r1` (cut by BM-task before Task 1)
- **Target impl-task model:** `sonnet-4-6` (default)
- **Estimated tasks:** 12 (Task 0 pre-flight + 10 impl + Task 11 retro)
- **Estimated cargo budget:** N/A (Shape G — cargo runs off-box on GitHub-hosted runners)
- **Forbidden-window applicability:** standard; non-binding for impl-task throughput under Shape G
- **Complexity score:** **19/10** — see breakdown below; **proceed-as-one** with JM-a + SL-a precedent rationale

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Sonnet target -> split-DQ if score > 8.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 5 | Tasks 1-10 = 10 impl tasks; 10 - 5 = 5. |
| Migrations touched | +2 each | 8 | 4 new migrations. |
| Crates touched | +1 each | 3 | `lemmy_db_schema`, `lemmy_db_schema_file`, `lemmy_api`. |
| `crates/server/tests/e2e.rs` edits | +3 each | 3 | 1 task (Task 10). |
| New ADR-affecting decisions | +2 each | 0 | RT-r1 is PRD-aligned. |
| Cargo budget peak above 6 GB | +1 per GB | 0 | Shape G — cargo on GH-hosted runners. |
| **Total** | — | **19** | Threshold `>8` exceeded. |

### 5.2 Score-vs-threshold resolution (proceed-as-one)

Score 19 exceeds the Sonnet `>8` threshold. The planner's recommendation is **proceed-as-one** (per `feedback_principles_not_rules.md`):

- **Schema-only sub-phase precedent.** JM-a (mechanical score ~17) shipped 11 tasks without splitting. SL-a (score 13) shipped 9 tasks without splitting. RT-r1 sits between them in profile.
- **All work is mechanical** — pattern-matching from MIRROR refs in §10. No design decisions inside §13 task bodies.
- **No file overlap on the [P] cohorts** (verified by FILES YAML in §13). Cohort A (4 migrations) writes 8 disjoint files; Cohort B (config / governance_log / e2e) writes 3 disjoint files.
- **Splitting would force re-merging** — parity tests at `config.rs:2692-2707` only become green after BOTH the seed migration AND config.rs additions land. Splitting r1 into r1a/r1b would create an intermediate state where parity test fails.

If the advisor disagrees and forces "split," the natural cut is r1a (Tasks 0-4 + 5-7) and r1b (Tasks 8-10 + 11). **Default: proceed-as-one.**

### 5.3 Per-task complexity ceiling check

Sonnet target -> ceiling is <= 4 files per task / <= 2 distinct crates per task. Walking §13:

- T1, T2, T3, T4: each 2 files; 0 crates. PASS.
- T5: 1 file; 1 crate. PASS.
- T6: 1 file; 1 crate. PASS.
- T7: 2 files; 1 crate. PASS.
- T8: 1 file; 1 crate. PASS.
- T9: 3 files; 2 crates. PASS.
- T10: 1 file; 1 crate. PASS.

No task exceeds ceiling.

## 6. Relationship to other v1 sub-phases

| Sub-phase | Depends on | Can parallelise with |
|---|---|---|
| **v1-RT-r1** (this plan) | OQ-018 endpoint (v1-AD-b PR #76 `f03ed1cba`); v1-AD-a merged | v1-SL-d (zero handler-file overlap; one shared file is `e2e.rs` — RT-r1 only extends `phase1_migrations_round_trip` post-condition probes, no new test fns; SL-d extends with new tests under different `mod` names) |
| v1-RT-r2 | RT-r1 merged | — |
| v1-RT-r3 | RT-r2 merged | v1-SL-d, v1-JM-d |
| v1-RT-r4 | RT-r1 merged (allowlist table) | v1-RT-r3 |
| v1-RT-r5 | RT-r2 merged (decay live for rollup) | v1-RT-r3, v1-RT-r4 |
| v1-RT-r6 (carry-forward) | independent | r2/r3/r4/r5 |

RT-r1 must merge before any of r2/r3/r4/r5 start. Task 0 Probe 7 (concurrent-PR check) catches active drift.

## 7. Preflight guardrails inherited from prior phases

Per PRD §15 + JM-a / SL-a retros + advisor-orchestrator.md §3.4-§3.6 gates.

- **R1:** every `i32 <-> i64` comparison uses `i64::from(...)`, never `as` cast.
- **R2:** all match arms use explicit `| Pattern` enumeration — never `_ =>` on enums.
- **R5:** Task 0 enumerates ALL probes explicitly per `.claude/rules/pre-phase-harness-audit.md`.
- **R6:** all clippy invocations use `--no-deps` uniformly (encoded in `cargo-validate-workspace.yml`).
- **R7:** every task that modifies a struct or re-export runs `cargo test --no-run -p lemmy_server --test e2e` (encoded in workflow).
- **R8:** every cargo invocation uses `--workspace --features full` (NOT `-p <crate>` + `--features full`).
- **R9:** every per-task validation gate runs as a Shape-G workflow on the worker branch; impl-task captures the workflow_run_id post-push and writes a `kind: "validate-pending"` DQ entry.
- **R10 (Task 0 / Task 9):** governance-log entry-kind-registry count invariant — `rg -c '^pub const ENTRY_KIND_'` returns the same number on both files. Goes 38 -> 45 post-Task 9.
- **R11 (Task 8):** the parametric pattern is mandatory — RT-r1 adds `EXPECTED_SEED_COUNT_V1_RT = 26` beside locked counts.
- **R12 (Task 4):** all 26 seed `INSERT INTO governance_config` rows are byte-identical to the `SEEDED_KEYS_WITH_CONSTS` v1-RT-r1 additions block (Task 8). Pre-commit reconciliation gate (Task 4) catches drift.

## 8. Flow design

### 8.1 Before state (governance-v0 @ `72ad06e72` — RT-r1 plan-write time)

```
+------------------------------------------------------------------------------+
|                              BEFORE STATE                                    |
+------------------------------------------------------------------------------+
|   reputation_event (10 cols):                                                |
|     id, person_id, community_id, dimension, delta, source_case_id,           |
|     source_report_id, reason, created_at, expires_at                         |
|   (no dedupe_key column; no source_event_type column)                        |
|                                                                              |
|   sponsor_allowlist (4 cols, v1-AD-a-shipped stub):                          |
|     id, community_id NOT NULL, person_id, created_at                         |
|                                                                              |
|   ReputationEventSourceType enum: does not exist                             |
|                                                                              |
|   config.rs:                                                                 |
|     EXPECTED_SEED_COUNT = 34, V1_AD = 27, V1_JM = 27, V1_SL = 13             |
|     SEEDED_KEYS_WITH_CONSTS: 101 tuples                                      |
|     CONFIG_KEY_METADATA: 101 entries                                         |
|                                                                              |
|   governance_log.rs (db_schema DEFINE + api SHIM RE-EXPORT):                 |
|     38 ENTRY_KIND_* consts                                                   |
|                                                                              |
|   .claude/rules/governance-log-entry-kind-registry.md:                       |
|     reputation-tuning-v1 section RESERVED                                    |
|                                                                              |
|   crates/server/tests/e2e.rs:                                                |
|     PHASE_1_MIGRATION_COUNT = 14                                             |
+------------------------------------------------------------------------------+
```

### 8.2 After state (post-RT-r1 phase-branch tip)

```
+------------------------------------------------------------------------------+
|                               AFTER STATE                                    |
+------------------------------------------------------------------------------+
|   reputation_event (12 cols):                                                |
|     ... existing 10 cols unchanged ...                                       |
|     dedupe_key TEXT NULL                                                     |
|     source_event_type reputation_event_source_type NOT NULL                  |
|                                                  DEFAULT 'Endorsement'      |
|   (partial unique index reputation_event_dedupe_key_partial_idx)             |
|                                                                              |
|   sponsor_allowlist (6 cols):                                                |
|     id, community_id NULLABLE, person_id, created_at,                        |
|     added_by_admin_id NOT NULL FK person, note TEXT NULL                     |
|                                                                              |
|   ReputationEventSourceType enum: 9 variants                                 |
|                                                                              |
|   config.rs:                                                                 |
|     EXPECTED_SEED_COUNT_V1_RT = 26                                           |
|     SEEDED_KEYS_WITH_CONSTS: 127 tuples (101 + 26)                           |
|     CONFIG_KEY_METADATA: 127 entries                                         |
|                                                                              |
|   governance_log.rs:                                                         |
|     45 ENTRY_KIND_* consts (38 + 7 new RT-r1 kinds)                          |
|                                                                              |
|   .claude/rules/governance-log-entry-kind-registry.md:                       |
|     reputation-tuning-v1 section POPULATED with 7 rows                       |
|     Acceptance invariants count: 38 -> 45                                    |
|                                                                              |
|   crates/server/tests/e2e.rs:                                                |
|     PHASE_1_MIGRATION_COUNT = 18                                             |
+------------------------------------------------------------------------------+
```

### 8.3 Endpoint changes

None in RT-r1. The 26 seeded knobs become writable via OQ-018 admin endpoint shipped in v1-AD-b. The `feature.reputation_v1_decay_enabled` flag stays `false` at deploy.

The 7 new entry-kind consts are declared but **not emitted** in r1. Future emission map (advisory):

| Entry kind | Future emitter | Sub-phase |
|---|---|---|
| `ENTRY_KIND_PARTICIPATION_CRON_TICK` | `crates/routes/src/utils/scheduled_tasks.rs` weekly-active cron block | r3 |
| `ENTRY_KIND_VOTE_OUTCOME_RECORDED` | `crates/api/api/src/governance/submit_jury_vote.rs` post-decision align hook | r3 |
| `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED` | `submit_jury_vote.rs` rationale-cited heuristic + new `flag-bad-faith` admin endpoint | r3 |
| `ENTRY_KIND_ROLLUP_RECOMPUTED` | `scheduled_tasks.rs` `reputation_rollup_cron` block | r5 |
| `ENTRY_KIND_DECAY_KNOB_CHANGED` | `crates/api/api/src/governance/admin_config.rs` first decay-key write | r2 |
| `ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED` | new admin endpoint `POST /api/v4/governance/admin/sponsor-allowlist/add` | r4 |
| `ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED` | new admin endpoint `POST /api/v4/governance/admin/sponsor-allowlist/remove` | r4 |

## 9. Mandatory reading

The implementation agent MUST read these at first iteration before any file edit.

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `.claude/PRPs/prds/v1-reputation-tuning.prd.md` | §5.3, §5.4, §7, §8, §9.1, §15 row 1 | Canonical schema, defaults matrix, scope boundary |
| P0 | `.claude/decision-queue.json` resolved entries #181-#186 | full | Pre-planning clarify resolutions + planner DQ #187 |
| P0 | `.claude/PRPs/plans/phase-v1-JM-a.plan.md` | §13 task list, §15 DoD, §10.7-§10.10, Task 7-10 | Closest-shape precedent. Mirror byte-for-byte. |
| P0 | `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` | §13 Task 1, §10.6, §15.6 | Shape-G plan precedent + parametric pattern + per-task `[P]` cohort markers |
| P0 | `.claude/rules/governance-log-entry-kind-registry.md` | full | Registry invariants + dual-file rule + reservation section to populate |
| P0 | `crates/api/api/src/governance/config.rs` | 1391-1422 (consts), 2683-2710 (parity test) | Extension site for all config scaffolding |
| P0 | `crates/db_schema_file/src/enums.rs` | 598-615 (`ReputationDimension`) | Mirror for `ReputationEventSourceType` |
| P0 | `crates/db_schema_file/src/schema.rs` | 1-119 (sql_types), 1218-1229, 1338-1343, 1466-1482 | Insertion sites |
| P0 | `migrations/2026-04-15-100000-0000_add_governance_enums/up.sql` | full | `CREATE TYPE ... AS ENUM` shape |
| P0 | `migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql` | full | v1-AD-a precedent; documents 3 keys RT-r1 must NOT re-seed |
| P0 | `migrations/2026-04-22-000100-0000_add_sponsor_allowlist/up.sql` | full | Pre-existing v1-AD-a sponsor_allowlist CREATE — RT-r1 ALTERs |
| P0 | `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql` | full | JM-a ALTER + backfill UPDATE precedent |
| P0 | `crates/db_schema/src/source/governance/reputation_event.rs` | 1-42 | Current Queryable + InsertForm |
| P0 | `crates/db_schema/src/source/governance/sponsor_allowlist.rs` | 1-37 | Current Queryable + InsertForm |
| P0 | `crates/db_schema/src/newtypes.rs` | 269-275, 325-331 | Verify both newtypes already present per DQ #181 |
| P0 | `crates/db_schema/src/source/governance/governance_log.rs` | 1-200 | Insertion site for 7 new RT-r1 consts |
| P0 | `crates/api/api/src/governance/governance_log.rs` | full | Re-export shim — alphabetical `pub use` block |
| P0 | `crates/server/tests/e2e.rs` | 1054-1100 (`phase1_migrations_round_trip` + `PHASE_1_MIGRATION_COUNT`) | Test extension site |
| P1 | `.claude/rules/phase-branch.md` | full | Task 0 branch-assertion gate |
| P1 | `.claude/rules/cargo-output-capture.md` + `no-cargo-output-paste.md` | full | Cargo output discipline |
| P1 | `.claude/rules/pre-phase-harness-audit.md` | full | Pre-flight audit at Task 0 |
| P1 | `.claude/rules/decision-queue.md` | full | DQ writes; `kind: "validate-pending"` Shape G handling |
| P1 | `.claude/rules/advisor-orchestrator.md` | §3.1 + §4 + §5.2 + §5.3 | Stage-shape + cohort + classifier invariants |
| P1 | `.claude/rules/branch-manager.md` | full | File-ownership boundaries |
| P1 | `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` | ADR-005, ADR-008, ADR-010, ADR-013, ADR-015 + OQ-001/-019/-020/-021/-018 | Contradiction-check surface |

### 9.2 External documentation

Only existing workspace versions are needed.

| Source | Version | Section | Why |
|---|---|---|---|
| diesel | 2.3.7 | `diesel::table!`, `ALTER TABLE ADD COLUMN`, `ALTER COLUMN DROP NOT NULL` | Workspace |
| diesel-derive-enum | 2.1.0 | `DbEnum` derive + `DbValueStyle = "verbatim"` | Workspace |
| chrono | 0.4.44 | `DateTime<Utc>` (no new uses) | Workspace |
| serde_json | 1.0.149 | (no new uses; r1 ships zero JSONB) | Workspace |

## 10. Patterns to mirror

Per `feedback_advisor_watchpoint_specificity.md`: each entry cites a specific table, file, or `schema.rs` line.

### 10.1 Postgres enum migration shape (Task 1)

**Mirror primary:** `migrations/2026-04-15-100000-0000_add_governance_enums/up.sql` + `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql`.

**Task 1 up.sql skeleton:**

```sql
-- v1-RT-r1 task 1: add reputation_event v1 columns + new source-type enum.
-- ============================================================
-- ADR exception trail (protected governance tables — append-only)
-- ============================================================
-- ADDITIVE only: ADD COLUMN, CREATE INDEX, CREATE TYPE.
-- Authority trail:
--   - PRD: .claude/PRPs/prds/v1-reputation-tuning.prd.md sections 5.3 + 7
--   - Plan: .claude/PRPs/plans/v1-reputation-tuning-r1.plan.md section 10.1, Task 1
--   - DQ #182 (advisor 2026-05-10): governance/ subdir paths
-- ============================================================

CREATE TYPE reputation_event_source_type AS ENUM (
    'Endorsement',
    'JuryVote',
    'SponsorLiability',
    'FounderSeed',
    'ParticipationCron',
    'DormancyCron',
    'VoteOutcome',
    'EvidenceQuality',
    'ManualSeed'
);

ALTER TABLE reputation_event ADD COLUMN dedupe_key TEXT;
COMMENT ON COLUMN reputation_event.dedupe_key IS
    'Per PRD section 5.3 source 1+2: idempotency key for cron events.
     Format: source:community_id:iso_week or source:community_id:person_id:iso_week.
     NULL for non-cron events.';

CREATE UNIQUE INDEX reputation_event_dedupe_key_partial_idx
    ON reputation_event (dedupe_key)
    WHERE dedupe_key IS NOT NULL;
COMMENT ON INDEX reputation_event_dedupe_key_partial_idx IS
    'Per PRD section 5.3 source 1: idempotency for participation cron emits.
     Partial WHERE keeps the index small.';

ALTER TABLE reputation_event
    ADD COLUMN source_event_type reputation_event_source_type
    NOT NULL DEFAULT 'Endorsement';
COMMENT ON COLUMN reputation_event.source_event_type IS
    'Per PRD section 5.3 + 7: per-event source classification.
     v0 rows receive Endorsement via column DEFAULT;
     2026-05-10-000200-0000 backfill revises by reason ILIKE per DQ #184.';
```

**Task 1 down.sql skeleton:**

```sql
-- Reverse of v1-RT-r1 Task 1 up.sql.
ALTER TABLE reputation_event DROP COLUMN IF EXISTS source_event_type;
DROP INDEX IF EXISTS reputation_event_dedupe_key_partial_idx;
ALTER TABLE reputation_event DROP COLUMN IF EXISTS dedupe_key;
DROP TYPE IF EXISTS reputation_event_source_type;
```

**GOTCHA:** `CREATE TYPE` runs inside transactions — no `-- no-transaction` directive needed.

**GOTCHA:** PascalCase variants match `DbValueStyle = "verbatim"` on Rust side (Task 5).

**GOTCHA:** `NOT NULL DEFAULT 'Endorsement'` exploits Postgres 11+ `attmissingval` (O(1) backfill). Backfill (Task 3) is separate so per-row UPDATE never blocks the schema-add.

### 10.2 Sponsor_allowlist ALTER shape (Task 2)

**Mirror primary:** `migrations/2026-04-22-000200-0000_add_case_applied_config_snapshot/up.sql` + JM-a Task 2.

**Task 2 up.sql skeleton:**

```sql
-- v1-RT-r1 task 2: extend sponsor_allowlist (shipped pre-v1-AD-a per
-- DQ #181) for r4-allowlist-strategy admin readers.
-- ============================================================
-- Authority trail:
--   - PRD: section 5.4 third sponsor-gate strategy
--   - Plan: section 10.2, Task 2
--   - DQ #181 (advisor 2026-05-10): EXTEND, do NOT create
-- ============================================================

ALTER TABLE sponsor_allowlist
    ALTER COLUMN community_id DROP NOT NULL;
COMMENT ON COLUMN sponsor_allowlist.community_id IS
    'Per PRD section 5.4: NULL means instance-wide allowlist;
     NON-NULL scopes to one community.';

ALTER TABLE sponsor_allowlist
    ADD COLUMN added_by_admin_id INTEGER NOT NULL
        REFERENCES person(id) DEFAULT 1;
ALTER TABLE sponsor_allowlist
    ALTER COLUMN added_by_admin_id DROP DEFAULT;
COMMENT ON COLUMN sponsor_allowlist.added_by_admin_id IS
    'Per PRD section 5.4: admin who added the row (audit trail).
     Required non-null. r4 endpoints set from caller person_id.';

ALTER TABLE sponsor_allowlist ADD COLUMN note TEXT;
COMMENT ON COLUMN sponsor_allowlist.note IS
    'Per PRD section 5.4: admin-supplied free-text rationale. NULL allowed.';
```

**Task 2 down.sql skeleton:**

```sql
-- Reverse of v1-RT-r1 Task 2 up.sql.
ALTER TABLE sponsor_allowlist DROP COLUMN IF EXISTS note;
ALTER TABLE sponsor_allowlist DROP COLUMN IF EXISTS added_by_admin_id;
ALTER TABLE sponsor_allowlist ALTER COLUMN community_id SET NOT NULL;
-- NOTE: SET NOT NULL fails if any rows have community_id IS NULL.
-- r4-allowlist-strategy is the first writer of NULL rows; if r4 has
-- shipped + populated NULL rows before this down.sql is invoked, the
-- DBA must DELETE FROM sponsor_allowlist WHERE community_id IS NULL
-- first.
```

**GOTCHA:** `ALTER COLUMN community_id DROP NOT NULL` is fast metadata in Postgres 11+. `ADD COLUMN ... NOT NULL DEFAULT 1` then `DROP DEFAULT` exploits attmissingval — table empty at RT-r1 ship time.

**GOTCHA:** the `DROP DEFAULT` immediately after ADD COLUMN prevents accidental defaulting of future inserts. r4 endpoints MUST set `added_by_admin_id` explicitly.

**GOTCHA (down.sql):** SET NOT NULL fails if NULL community_id rows exist. Documented in down.sql comment.

### 10.3 Backfill UPDATE shape (Task 3)

**Mirror primary:** JM-a Task 2 backfill UPDATE block.

**Task 3 up.sql skeleton:**

```sql
-- v1-RT-r1 task 3: backfill source_event_type per DQ #184 5-step
-- precedence chain.
-- ============================================================
-- Authority trail:
--   - PRD: section 7 Backfill row + section 5.3 source enumeration
--   - Plan: section 10.3, Task 3
--   - DQ #184 (advisor 2026-05-10): use source_case_id + reason ILIKE
-- ============================================================

-- Precedence (apply in priority order — first match wins):
--   1. reason ILIKE 'sponsor_liability%' -> SponsorLiability
--   2. reason ILIKE 'jury_reliability%' OR similar -> JuryVote
--   3. reason ILIKE 'founder_seed%' -> FounderSeed
--   4. otherwise -> Endorsement (column DEFAULT covers; no-op)
--   5. fallback -> ManualSeed (no-op at backfill; future-stale rows)

UPDATE reputation_event
SET source_event_type = 'SponsorLiability'
WHERE source_event_type = 'Endorsement'
  AND reason ILIKE 'sponsor_liability%';

UPDATE reputation_event
SET source_event_type = 'JuryVote'
WHERE source_event_type = 'Endorsement'
  AND (reason ILIKE 'jury_reliability%'
       OR reason ILIKE 'jury_vote%'
       OR reason ILIKE 'jury_align%');

UPDATE reputation_event
SET source_event_type = 'FounderSeed'
WHERE source_event_type = 'Endorsement'
  AND reason ILIKE 'founder_seed%';
```

**Task 3 down.sql skeleton:**

```sql
-- Reverse: reset to column DEFAULT 'Endorsement'.
UPDATE reputation_event
SET source_event_type = 'Endorsement'
WHERE source_event_type IN ('SponsorLiability', 'JuryVote', 'FounderSeed');
```

**GOTCHA:** `WHERE source_event_type = 'Endorsement'` makes each UPDATE idempotent.

**GOTCHA:** smoke check at retro time (`SELECT source_event_type, COUNT(*) FROM reputation_event GROUP BY 1`) shows non-zero counts for at least Endorsement on a seeded DB. On fresh-deploy DB with no rows, smoke shows zero — correct.

**GOTCHA:** drift in `reason` strings (future v1+ rename) would mis-classify; acceptable best-effort per PRD §7.

### 10.4 Seed migration shape (Task 4)

**Mirror primary:** `migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql` (v1-AD-a) + JM-a `seed_v1_jm_config_keys`.

**Task 4 up.sql skeleton:**

```sql
-- v1-RT-r1 task 4: seed 26 reputation-tuning-owned governance_config rows.
-- ============================================================
-- Authoritative scope: PRD section 8 (29 rows) MINUS 3 v1-AD-a-shipped
-- per planner DQ #187:
--   - deltas.participation_weekly_active
--   - participation.dormancy_window_days
--   - deltas.participation_dormant
-- 26 rows below = 28 PRD knobs + 1 feature flag - 3 dupes.
--
-- Cumulative invariant: SEEDED_KEYS_WITH_CONSTS.len() must equal
-- 34 + 27 + 27 + 13 + 26 = 127. Parity test at config.rs:2692-2707
-- updated in Task 8.
--
-- Authority trail:
--   - PRD: section 8 Defaults Matrix
--   - Plan: section 10.4, Task 4
--   - DQ #185 (advisor): conceptual count is 29
--   - DQ #187 (planner): net-new is 26
-- ============================================================

INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES
    -- 8 decay.<dimension>.<direction>_half_life_days (int)
    ('instance', 'decay.reporting_accuracy.positive_half_life_days',          'int',  90,    NULL, NULL,  NULL),
    ('instance', 'decay.reporting_accuracy.negative_half_life_days',          'int',  180,   NULL, NULL,  NULL),
    ('instance', 'decay.jury_reliability.positive_half_life_days',            'int',  90,    NULL, NULL,  NULL),
    ('instance', 'decay.jury_reliability.negative_half_life_days',            'int',  180,   NULL, NULL,  NULL),
    ('instance', 'decay.participation_consistency.positive_half_life_days',   'int',  60,    NULL, NULL,  NULL),
    ('instance', 'decay.participation_consistency.negative_half_life_days',   'int',  60,    NULL, NULL,  NULL),
    ('instance', 'decay.endorsement_strength.positive_half_life_days',        'int',  90,    NULL, NULL,  NULL),
    ('instance', 'decay.endorsement_strength.negative_half_life_days',        'int',  180,   NULL, NULL,  NULL),
    -- 8 bounds.<dimension>.<floor|ceiling> (int)
    ('instance', 'bounds.reporting_accuracy.floor',                           'int',  -100,  NULL, NULL,  NULL),
    ('instance', 'bounds.reporting_accuracy.ceiling',                         'int',  100,   NULL, NULL,  NULL),
    ('instance', 'bounds.jury_reliability.floor',                             'int',  -100,  NULL, NULL,  NULL),
    ('instance', 'bounds.jury_reliability.ceiling',                           'int',  100,   NULL, NULL,  NULL),
    ('instance', 'bounds.participation_consistency.floor',                    'int',  -100,  NULL, NULL,  NULL),
    ('instance', 'bounds.participation_consistency.ceiling',                  'int',  100,   NULL, NULL,  NULL),
    ('instance', 'bounds.endorsement_strength.floor',                         'int',  0,     NULL, NULL,  NULL),
    ('instance', 'bounds.endorsement_strength.ceiling',                       'int',  200,   NULL, NULL,  NULL),
    -- 1 deltas.participation_juror_aligned (PRD section 5.3 source 3)
    ('instance', 'deltas.participation_juror_aligned',                        'int',  1,     NULL, NULL,  NULL),
    -- 2 participation.* context knobs (activity_threshold + lookback)
    ('instance', 'participation.activity_threshold_comments',                 'int',  1,     NULL, NULL,  NULL),
    ('instance', 'participation.lookback_days',                               'int',  7,     NULL, NULL,  NULL),
    -- 2 deltas.evidence_* (PRD section 5.3 source 4)
    ('instance', 'deltas.evidence_cited',                                     'int',  1,     NULL, NULL,  NULL),
    ('instance', 'deltas.evidence_bad_faith',                                 'int',  -1,    NULL, NULL,  NULL),
    -- 1 participation.evidence_cited_rationale_threshold_chars
    ('instance', 'participation.evidence_cited_rationale_threshold_chars',    'int',  256,   NULL, NULL,  NULL),
    -- 2 job.* cadence knobs (instance-only)
    ('instance', 'job.participation_interval_days',                           'int',  7,     NULL, NULL,  NULL),
    ('instance', 'job.rollup_interval_days',                                  'int',  7,     NULL, NULL,  NULL),
    -- 1 job.rollup_equal_weights (instance-only)
    ('instance', 'job.rollup_equal_weights',                                  'bool', NULL,  NULL, true,  NULL),
    -- 1 feature flag (instance-only)
    ('instance', 'feature.reputation_v1_decay_enabled',                       'bool', NULL,  NULL, false, NULL)
ON CONFLICT (scope, key, valid_from) DO NOTHING;

-- Total: 8 + 8 + 1 + 2 + 2 + 1 + 2 + 1 + 1 = 26 net-new rows.
```

**Task 4 down.sql skeleton:**

```sql
-- Reverse of v1-RT-r1 Task 4 up.sql.
DELETE FROM governance_config
WHERE scope = 'instance' AND key IN (
    'decay.reporting_accuracy.positive_half_life_days',
    'decay.reporting_accuracy.negative_half_life_days',
    'decay.jury_reliability.positive_half_life_days',
    'decay.jury_reliability.negative_half_life_days',
    'decay.participation_consistency.positive_half_life_days',
    'decay.participation_consistency.negative_half_life_days',
    'decay.endorsement_strength.positive_half_life_days',
    'decay.endorsement_strength.negative_half_life_days',
    'bounds.reporting_accuracy.floor',
    'bounds.reporting_accuracy.ceiling',
    'bounds.jury_reliability.floor',
    'bounds.jury_reliability.ceiling',
    'bounds.participation_consistency.floor',
    'bounds.participation_consistency.ceiling',
    'bounds.endorsement_strength.floor',
    'bounds.endorsement_strength.ceiling',
    'deltas.participation_juror_aligned',
    'participation.activity_threshold_comments',
    'participation.lookback_days',
    'deltas.evidence_cited',
    'deltas.evidence_bad_faith',
    'participation.evidence_cited_rationale_threshold_chars',
    'job.participation_interval_days',
    'job.rollup_interval_days',
    'job.rollup_equal_weights',
    'feature.reputation_v1_decay_enabled'
);
```

**GOTCHA:** count check — up.sql has 26 INSERTs; down.sql has 26 keys. Both match `EXPECTED_SEED_COUNT_V1_RT = 26`.

**GOTCHA:** `ON CONFLICT (scope, key, valid_from) DO NOTHING` is idempotent across reruns.

**GOTCHA:** the 3 v1-AD-a duplicates are NOT in this migration. Their entries stay in v1-AD-a's `SEEDED_KEYS_WITH_CONSTS` block, owned by V1_AD's count.

### 10.5 New Rust enum shape (Task 5)

**Mirror primary:** `crates/db_schema_file/src/enums.rs:598-615` (`ReputationDimension`).

**Task 5 IMPLEMENT (insert after line 615):**

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::ReputationEventSourceType"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Per-event source classification for `reputation_event` rows. Per
/// PRD section 5.3 + 7. v0 rows receive Endorsement via column DEFAULT;
/// 2026-05-10-000200-0000 backfill revises by reason ILIKE per DQ #184.
/// r3 emitters write the matching variant explicitly going forward.
pub enum ReputationEventSourceType {
  #[default]
  Endorsement,
  JuryVote,
  SponsorLiability,
  FounderSeed,
  ParticipationCron,
  DormancyCron,
  VoteOutcome,
  EvidenceQuality,
  ManualSeed,
}
```

**GOTCHA:** PascalCase variants. Postgres `CREATE TYPE` (Task 1) uses same. `DbValueStyle = "verbatim"` keeps Rust <-> Postgres names 1:1.

**GOTCHA:** `#[default]` MUST be `Endorsement` to match column DEFAULT.

**GOTCHA:** existing imports cover `Serialize, Deserialize, DbEnum, ts_rs` — no additions needed.

### 10.6 schema.rs sql_types + table! extensions (Task 6)

**Mirror primary:** `schema.rs:118` (sql_types module); `schema.rs:1218-1229`; `schema.rs:1338-1343`.

**Task 6 IMPLEMENT — three sub-edits:**

```rust
// Sub-edit 1 — sql_types module (line ~118 after ReputationDimension):

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "reputation_event_source_type"))]
  pub struct ReputationEventSourceType;

// Sub-edit 2 — reputation_event table block:

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ReputationDimension;
    use super::sql_types::ReputationEventSourceType;

    reputation_event (id) {
        id -> Int4,
        person_id -> Int4,
        community_id -> Nullable<Int4>,
        dimension -> ReputationDimension,
        delta -> Int4,
        source_case_id -> Nullable<Int4>,
        source_report_id -> Nullable<Int4>,
        reason -> Text,
        created_at -> Timestamptz,
        expires_at -> Nullable<Timestamptz>,
        // v1-RT-r1 additions:
        dedupe_key -> Nullable<Text>,
        source_event_type -> ReputationEventSourceType,
    }
}

// Sub-edit 3 — sponsor_allowlist table block:

diesel::table! {
    sponsor_allowlist (id) {
        id -> Int4,
        community_id -> Nullable<Int4>,            // v1-RT-r1: was Int4 (NOT NULL); now nullable
        person_id -> Int4,
        created_at -> Timestamptz,
        // v1-RT-r1 additions:
        added_by_admin_id -> Int4,
        note -> Nullable<Text>,
    }
}
```

**GOTCHA:** Hand-edit of `@generated` per governance convention. Document with `// v1-RT-r1 additions:` and `// v1-RT-r1: was Int4 (NOT NULL); now nullable`.

**GOTCHA:** `joinable!` block at `schema.rs:1466-1482` is NOT edited. Adding `joinable!(sponsor_allowlist -> person (added_by_admin_id))` would conflict with existing `sponsor_allowlist -> person (person_id)` joinable.

### 10.7 Diesel struct extension (Task 7)

**Mirror primary:** existing `reputation_event.rs:1-42` and `sponsor_allowlist.rs:1-37`; JM-a Task 5.

**Task 7 IMPLEMENT — two sub-edits in two files (one commit):**

```rust
// Sub-edit 1 — crates/db_schema/src/source/governance/reputation_event.rs

use crate::newtypes::{CommunityId, ModerationCaseId, ReputationEventId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::{
  PersonId,
  enums::{ReputationDimension, ReputationEventSourceType},   // v1-RT-r1: add ReputationEventSourceType
};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::reputation_event;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = reputation_event))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A single reputation delta event targeting a person on a given dimension.
pub struct ReputationEvent {
  pub id: ReputationEventId,
  pub person_id: PersonId,
  pub community_id: Option<CommunityId>,
  pub dimension: ReputationDimension,
  pub delta: i32,
  pub source_case_id: Option<ModerationCaseId>,
  pub source_report_id: Option<i32>,
  pub reason: String,
  pub created_at: DateTime<Utc>,
  pub expires_at: Option<DateTime<Utc>>,
  /// v1-RT-r1 section 5.3 source 1: idempotency key for cron events.
  pub dedupe_key: Option<String>,
  /// v1-RT-r1 section 5.3 + 7: per-event source classification.
  pub source_event_type: ReputationEventSourceType,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = reputation_event))]
pub struct ReputationEventInsertForm {
  pub person_id: PersonId,
  pub community_id: Option<CommunityId>,
  pub dimension: ReputationDimension,
  pub delta: i32,
  pub source_case_id: Option<ModerationCaseId>,
  pub source_report_id: Option<i32>,
  pub reason: String,
  pub expires_at: Option<DateTime<Utc>>,
  /// v1-RT-r1: optional. Column DEFAULT covers callers that omit.
  pub dedupe_key: Option<String>,
  pub source_event_type: Option<ReputationEventSourceType>,
}
```

```rust
// Sub-edit 2 — crates/db_schema/src/source/governance/sponsor_allowlist.rs

use crate::newtypes::{CommunityId, SponsorAllowlistId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::PersonId;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::sponsor_allowlist;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = sponsor_allowlist))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Per-community sponsor-eligibility allowlist row. NULL community_id
/// means instance-wide.
pub struct SponsorAllowlist {
  pub id: SponsorAllowlistId,
  /// v1-RT-r1: relaxed from NOT NULL to nullable.
  pub community_id: Option<CommunityId>,
  pub person_id: PersonId,
  pub created_at: DateTime<Utc>,
  /// v1-RT-r1: admin who added the row (audit trail).
  pub added_by_admin_id: PersonId,
  /// v1-RT-r1: admin-supplied free-text rationale.
  pub note: Option<String>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = sponsor_allowlist))]
pub struct SponsorAllowlistInsertForm {
  /// v1-RT-r1: relaxed from required to optional.
  pub community_id: Option<CommunityId>,
  pub person_id: PersonId,
  /// v1-RT-r1: required at insert time. r4 endpoints set explicitly.
  pub added_by_admin_id: PersonId,
  /// v1-RT-r1: optional admin note.
  pub note: Option<String>,
}
```

**GOTCHA:** `ReputationEventInsertForm.source_event_type` is `Option<>` per `feedback_insertform_default_propagation.md`. NOT-NULL column DEFAULT covers callers.

**GOTCHA:** `SponsorAllowlistInsertForm.added_by_admin_id` is `PersonId` (NOT `Option<>`) — required. Verified at task start by `rg "SponsorAllowlistInsertForm" crates/`; should return zero results outside the file.

**GOTCHA:** `AsChangeset` derive: KEEP on `ReputationEventInsertForm`; DO NOT add to `SponsorAllowlistInsertForm` (matches existing pattern).

### 10.8 EXPECTED_SEED_COUNT_V1_RT + parity test extension (Task 8)

**Mirror primary:** `config.rs:1391-1422` + `:2683-2710` + JM-a §10.9 + SL-a §10.6.

**Task 8 IMPLEMENT — six sub-edits in `config.rs`:**

```rust
// Sub-edit 1 — DEFAULT_* consts (insert in new // -- v1-RT-r1
// additions ... -- block AFTER the v1-SL-a block):

// -- v1-RT-r1 additions (reputation-tuning sub-phase r1) -------------
//
// 26 net-new keys per PRD section 8 minus 3 v1-AD-a-shipped duplicates
// per planner DQ #187. Their DEFAULT_* consts + SEEDED_KEYS_WITH_CONSTS
// + CONFIG_KEY_METADATA entries are owned by v1-AD-a and stay there.

// 8 decay.<dimension>.<direction>_half_life_days (i64):
pub const DEFAULT_DECAY_REPORTING_ACCURACY_POSITIVE_HALF_LIFE_DAYS: i64 = 90;
pub const DEFAULT_DECAY_REPORTING_ACCURACY_NEGATIVE_HALF_LIFE_DAYS: i64 = 180;
pub const DEFAULT_DECAY_JURY_RELIABILITY_POSITIVE_HALF_LIFE_DAYS: i64 = 90;
pub const DEFAULT_DECAY_JURY_RELIABILITY_NEGATIVE_HALF_LIFE_DAYS: i64 = 180;
pub const DEFAULT_DECAY_PARTICIPATION_CONSISTENCY_POSITIVE_HALF_LIFE_DAYS: i64 = 60;
pub const DEFAULT_DECAY_PARTICIPATION_CONSISTENCY_NEGATIVE_HALF_LIFE_DAYS: i64 = 60;
pub const DEFAULT_DECAY_ENDORSEMENT_STRENGTH_POSITIVE_HALF_LIFE_DAYS: i64 = 90;
pub const DEFAULT_DECAY_ENDORSEMENT_STRENGTH_NEGATIVE_HALF_LIFE_DAYS: i64 = 180;

// 8 bounds.<dimension>.<floor|ceiling> (i64):
pub const DEFAULT_BOUNDS_REPORTING_ACCURACY_FLOOR: i64 = -100;
pub const DEFAULT_BOUNDS_REPORTING_ACCURACY_CEILING: i64 = 100;
pub const DEFAULT_BOUNDS_JURY_RELIABILITY_FLOOR: i64 = -100;
pub const DEFAULT_BOUNDS_JURY_RELIABILITY_CEILING: i64 = 100;
pub const DEFAULT_BOUNDS_PARTICIPATION_CONSISTENCY_FLOOR: i64 = -100;
pub const DEFAULT_BOUNDS_PARTICIPATION_CONSISTENCY_CEILING: i64 = 100;
pub const DEFAULT_BOUNDS_ENDORSEMENT_STRENGTH_FLOOR: i64 = 0;
pub const DEFAULT_BOUNDS_ENDORSEMENT_STRENGTH_CEILING: i64 = 200;

// 1 deltas.participation_juror_aligned (i64):
pub const DEFAULT_DELTAS_PARTICIPATION_JUROR_ALIGNED: i64 = 1;

// 2 participation.* context knobs:
pub const DEFAULT_PARTICIPATION_ACTIVITY_THRESHOLD_COMMENTS: i64 = 1;
pub const DEFAULT_PARTICIPATION_LOOKBACK_DAYS: i64 = 7;

// 2 deltas.evidence_*:
pub const DEFAULT_DELTAS_EVIDENCE_CITED: i64 = 1;
pub const DEFAULT_DELTAS_EVIDENCE_BAD_FAITH: i64 = -1;

// 1 participation.evidence_cited_rationale_threshold_chars:
pub const DEFAULT_PARTICIPATION_EVIDENCE_CITED_RATIONALE_THRESHOLD_CHARS: i64 = 256;

// 2 job.* cadence knobs:
pub const DEFAULT_JOB_PARTICIPATION_INTERVAL_DAYS: i64 = 7;
pub const DEFAULT_JOB_ROLLUP_INTERVAL_DAYS: i64 = 7;

// 1 job.rollup_equal_weights (bool):
pub const DEFAULT_JOB_ROLLUP_EQUAL_WEIGHTS: bool = true;

// 1 feature flag (bool):
pub const DEFAULT_FEATURE_REPUTATION_V1_DECAY_ENABLED: bool = false;

// Total: 8 + 8 + 1 + 2 + 2 + 1 + 2 + 1 + 1 = 26 net-new consts.

// Sub-edit 2 — match arms across const_default_int (24 arms) +
// const_default_bool (2 arms). const_default_float and
// const_default_text get 0 new arms.

// Sub-edit 3 — SEEDED_KEYS_WITH_CONSTS extension. Append after the
// v1-SL-a block (alphabetised within the new block by key):

  // v1-RT-r1 additions (26 net-new keys per PRD section 8 minus 3
  // v1-AD-a-shipped duplicates per DQ #187).
  ("bounds.endorsement_strength.ceiling",                       "DEFAULT_BOUNDS_ENDORSEMENT_STRENGTH_CEILING",                       "int"),
  ("bounds.endorsement_strength.floor",                         "DEFAULT_BOUNDS_ENDORSEMENT_STRENGTH_FLOOR",                         "int"),
  ("bounds.jury_reliability.ceiling",                           "DEFAULT_BOUNDS_JURY_RELIABILITY_CEILING",                           "int"),
  ("bounds.jury_reliability.floor",                             "DEFAULT_BOUNDS_JURY_RELIABILITY_FLOOR",                             "int"),
  ("bounds.participation_consistency.ceiling",                  "DEFAULT_BOUNDS_PARTICIPATION_CONSISTENCY_CEILING",                  "int"),
  ("bounds.participation_consistency.floor",                    "DEFAULT_BOUNDS_PARTICIPATION_CONSISTENCY_FLOOR",                    "int"),
  ("bounds.reporting_accuracy.ceiling",                         "DEFAULT_BOUNDS_REPORTING_ACCURACY_CEILING",                         "int"),
  ("bounds.reporting_accuracy.floor",                           "DEFAULT_BOUNDS_REPORTING_ACCURACY_FLOOR",                           "int"),
  ("decay.endorsement_strength.negative_half_life_days",        "DEFAULT_DECAY_ENDORSEMENT_STRENGTH_NEGATIVE_HALF_LIFE_DAYS",        "int"),
  ("decay.endorsement_strength.positive_half_life_days",        "DEFAULT_DECAY_ENDORSEMENT_STRENGTH_POSITIVE_HALF_LIFE_DAYS",        "int"),
  ("decay.jury_reliability.negative_half_life_days",            "DEFAULT_DECAY_JURY_RELIABILITY_NEGATIVE_HALF_LIFE_DAYS",            "int"),
  ("decay.jury_reliability.positive_half_life_days",            "DEFAULT_DECAY_JURY_RELIABILITY_POSITIVE_HALF_LIFE_DAYS",            "int"),
  ("decay.participation_consistency.negative_half_life_days",   "DEFAULT_DECAY_PARTICIPATION_CONSISTENCY_NEGATIVE_HALF_LIFE_DAYS",   "int"),
  ("decay.participation_consistency.positive_half_life_days",   "DEFAULT_DECAY_PARTICIPATION_CONSISTENCY_POSITIVE_HALF_LIFE_DAYS",   "int"),
  ("decay.reporting_accuracy.negative_half_life_days",          "DEFAULT_DECAY_REPORTING_ACCURACY_NEGATIVE_HALF_LIFE_DAYS",          "int"),
  ("decay.reporting_accuracy.positive_half_life_days",          "DEFAULT_DECAY_REPORTING_ACCURACY_POSITIVE_HALF_LIFE_DAYS",          "int"),
  ("deltas.evidence_bad_faith",                                 "DEFAULT_DELTAS_EVIDENCE_BAD_FAITH",                                 "int"),
  ("deltas.evidence_cited",                                     "DEFAULT_DELTAS_EVIDENCE_CITED",                                     "int"),
  ("deltas.participation_juror_aligned",                        "DEFAULT_DELTAS_PARTICIPATION_JUROR_ALIGNED",                        "int"),
  ("feature.reputation_v1_decay_enabled",                       "DEFAULT_FEATURE_REPUTATION_V1_DECAY_ENABLED",                       "bool"),
  ("job.participation_interval_days",                           "DEFAULT_JOB_PARTICIPATION_INTERVAL_DAYS",                           "int"),
  ("job.rollup_equal_weights",                                  "DEFAULT_JOB_ROLLUP_EQUAL_WEIGHTS",                                  "bool"),
  ("job.rollup_interval_days",                                  "DEFAULT_JOB_ROLLUP_INTERVAL_DAYS",                                  "int"),
  ("participation.activity_threshold_comments",                 "DEFAULT_PARTICIPATION_ACTIVITY_THRESHOLD_COMMENTS",                 "int"),
  ("participation.evidence_cited_rationale_threshold_chars",    "DEFAULT_PARTICIPATION_EVIDENCE_CITED_RATIONALE_THRESHOLD_CHARS",    "int"),
  ("participation.lookback_days",                               "DEFAULT_PARTICIPATION_LOOKBACK_DAYS",                               "int"),

// Sub-edit 4 — EXPECTED_SEED_COUNT_V1_RT (insert after V1_SL):

/// v1-RT-r1 adds 26 reputation-tuning-owned keys to
/// SEEDED_KEYS_WITH_CONSTS. Parametric per advisor directive 2026-04-19 #4.
/// Net-new: PRD section 8 lists 29 conceptual rows; 3 already shipped
/// under v1-AD-a per planner DQ #187 — those count under V1_AD's 27.
/// Cumulative: 34 + 27 + 27 + 13 + 26 = 127.
pub const EXPECTED_SEED_COUNT_V1_RT: usize = 26;

// Sub-edit 5 — extend parity test at line 2692-2710. The expected
// sum becomes EXPECTED_SEED_COUNT + V1_AD + V1_JM + V1_SL + V1_RT
// = 34 + 27 + 27 + 13 + 26 = 127. Update format-string to include
// V1_RT alongside the existing 4 names.

// Sub-edit 6 — append 26 new CONFIG_KEY_METADATA entries before
// closing `];` of the array.
//
// Per-key apply_at:
//   - decay.* + bounds.* -> ApplyAt::Immediate
//   - deltas.* + participation.* -> ApplyAt::Immediate
//   - job.participation_interval_days + job.rollup_interval_days ->
//     ApplyAt::Immediate **with caveat**: clokwerk schedules pin at
//     scheduler setup(); flip takes effect at next server restart.
//   - job.rollup_equal_weights -> ApplyAt::Immediate
//   - feature.reputation_v1_decay_enabled -> ApplyAt::Immediate
//
// scope:
//   - decay.* + bounds.* + deltas.* + participation.* -> ConfigScope::Both
//   - job.* + feature.* -> ConfigScope::Instance
//
// requires_re_jury: false; requires_step_up: false; for all 26.
// valid_range: per PRD section 8 Range column.
// description: one-sentence operator-facing summary per key.
// doc_anchor: "v1-reputation-tuning.prd.md section 8" for all 26.
```

**GOTCHA (R11):** parametric pattern mandatory. Do NOT bump locked v0 / V1_AD / V1_JM / V1_SL counts.

**GOTCHA (R12):** pre-commit reconciliation gate (Task 4 + Task 8) catches drift across SEEDED_KEYS_WITH_CONSTS / DEFAULT_* / match arms / CONFIG_KEY_METADATA / migration up.sql / migration down.sql, all at exactly **26**.

**GOTCHA:** the 3 v1-AD-a duplicates NOT in v1-RT-r1 SEEDED_KEYS block. If impl-task accidentally adds them, `every_seeded_key_has_metadata` fails.

### 10.9 ENTRY_KIND const dual-file edit (Task 9)

**Mirror primary:** JM-a §10.10 + SL-a §10.7. Three-file atomic commit.

**Task 9 IMPLEMENT — three sub-edits:**

**File 1** — `crates/db_schema/src/source/governance/governance_log.rs`, append AFTER the v1-SL-a block (last existing const is `ENTRY_KIND_SPONSOR_LIABILITY_PENDING`):

```rust
// v1-RT-r1 additions (v1 reputation-tuning sub-phase r1). All seven
// emitting call sites land in r2/r3/r4/r5 per pre-landed-const exemption.
//   - PARTICIPATION_CRON_TICK -> r3 scheduled_tasks.rs (pending).
//   - VOTE_OUTCOME_RECORDED -> r3 submit_jury_vote.rs (pending).
//   - EVIDENCE_QUALITY_RECORDED -> r3 submit_jury_vote.rs + r3 admin
//     flag-bad-faith endpoint (pending).
//   - ROLLUP_RECOMPUTED -> r5 reputation_rollup_cron (pending).
//   - DECAY_KNOB_CHANGED -> r2 admin_config.rs (pending).
//   - SPONSOR_ALLOWLIST_ADDED -> r4 add endpoint (pending).
//   - SPONSOR_ALLOWLIST_REMOVED -> r4 remove endpoint (pending).
pub const ENTRY_KIND_PARTICIPATION_CRON_TICK: &str = "participation_cron_tick";
pub const ENTRY_KIND_VOTE_OUTCOME_RECORDED: &str = "vote_outcome_recorded";
pub const ENTRY_KIND_EVIDENCE_QUALITY_RECORDED: &str = "evidence_quality_recorded";
pub const ENTRY_KIND_ROLLUP_RECOMPUTED: &str = "rollup_recomputed";
pub const ENTRY_KIND_DECAY_KNOB_CHANGED: &str = "decay_knob_changed";
pub const ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED: &str = "sponsor_allowlist_added";
pub const ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED: &str = "sponsor_allowlist_removed";
```

**File 2** — `crates/api/api/src/governance/governance_log.rs`, insert ALPHABETICALLY into the `pub use` block. Specific positions:

- `ENTRY_KIND_DECAY_KNOB_CHANGED` between `ENTRY_KIND_CASE_DECIDED` and `ENTRY_KIND_EMERGENCY_REMOVED`.
- `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED` between `ENTRY_KIND_ENDORSEMENT_REVOKED` (v1-SL-a) and `ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED`.
- `ENTRY_KIND_PARTICIPATION_CRON_TICK` between `ENTRY_KIND_PANEL_ASSEMBLED` and `ENTRY_KIND_PUBLIC_LOG_PUBLISHED`.
- `ENTRY_KIND_ROLLUP_RECOMPUTED` between `ENTRY_KIND_RESTORATION_COMPLETED` (v1-SL-a) and `ENTRY_KIND_RULE_SET_VERSION_CREATED` (v1-AD-c).
- `ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED` between `ENTRY_KIND_SEVERITY_TIER_FROZEN` (v1-JM-a) and `ENTRY_KIND_SPONSOR_LIABILITY_APPLIED`.
- `ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED` immediately after `ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED`, before `ENTRY_KIND_SPONSOR_LIABILITY_APPLIED`.
- `ENTRY_KIND_VOTE_OUTCOME_RECORDED` between `ENTRY_KIND_THRESHOLD_MET` and the closing `};` of the `pub use` block.

**File 3** — `.claude/rules/governance-log-entry-kind-registry.md`. Replace the `### reputation-tuning-v1 (reserved — §7 of PRD enumerates 7 new kinds)` stub with a populated section. Update Acceptance invariants count from `38` -> `45`.

**Populated section text:**

```markdown
## v1-RT-r1 entry kinds (7, this sub-phase)

Landed alongside task 9 dual-file edit. v1-RT-r1 writes the const
declarations only; emitting call sites land in r2/r3/r4/r5 per the
registry rule pre-landed-const exemption.

| Rust const | `&str` value | Source | Emitting handler | Semantic |
|---|---|---|---|---|
| `ENTRY_KIND_PARTICIPATION_CRON_TICK` | `participation_cron_tick` | v1-RT-r1 const; v1-RT-r3 call site | v1-RT-r3 `crates/routes/src/utils/scheduled_tasks.rs` weekly-active cron block (pending) | One participation-cron tick fired for one community for one ISO week. Payload: `{community_id, iso_week, active_user_count, dedupe_key, weekly_active_delta}`. Per PRD §5.3 source 1. |
| `ENTRY_KIND_VOTE_OUTCOME_RECORDED` | `vote_outcome_recorded` | v1-RT-r1 const; v1-RT-r3 call site | v1-RT-r3 `crates/api/api/src/governance/submit_jury_vote.rs` post-decision align hook (pending) | Post-decision juror-aligned `+1 participation_consistency` event. Payload: `{case_id, juror_pseudonym, dimension, delta, source_event_type}`. Per PRD §5.3 source 3. |
| `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED` | `evidence_quality_recorded` | v1-RT-r1 const; v1-RT-r3 call sites | v1-RT-r3 `submit_jury_vote.rs` rationale-cited heuristic AND v1-RT-r3 `admin_emergency_remove.rs::flag_bad_faith` (both pending) | Reporter `+1 reporting_accuracy` (rationale-cited) OR `-1 reporting_accuracy` (admin-flagged bad-faith). Payload: `{case_id, reporter_pseudonym, dimension, delta, source_event_type, trigger}`. Per PRD §5.3 source 4. |
| `ENTRY_KIND_ROLLUP_RECOMPUTED` | `rollup_recomputed` | v1-RT-r1 const; v1-RT-r5 call site | v1-RT-r5 `scheduled_tasks.rs::reputation_rollup_cron` (pending) | Per-person instance-wide rollup snapshot recomputed. Payload: `{person_id, contributing_community_count, rollup_dimensions: {<dim>: <int>, ...}, recomputed_at}`. Per PRD §5.5. |
| `ENTRY_KIND_DECAY_KNOB_CHANGED` | `decay_knob_changed` | v1-RT-r1 const; v1-RT-r2 call site | v1-RT-r2 `crates/api/api/src/governance/admin_config.rs` first decay-key write when v1 calculator goes live (pending) | Admin write of one of the 8 `decay.*.*_half_life_days` knobs OR the `feature.reputation_v1_decay_enabled` flag. Payload: `{key, old_value, new_value, scope, community_id?, admin_pseudonym}`. Per PRD §10 + §6. |
| `ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED` | `sponsor_allowlist_added` | v1-RT-r1 const; v1-RT-r4 call site | v1-RT-r4 `admin_sponsor_allowlist.rs::add` (handler name TBD; pending) | New `sponsor_allowlist` row inserted by admin. Payload: `{allowlist_id, community_id?, person_pseudonym, added_by_admin_pseudonym, note?, added_at}`. Per PRD §5.4 third strategy. |
| `ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED` | `sponsor_allowlist_removed` | v1-RT-r1 const; v1-RT-r4 call site | v1-RT-r4 `admin_sponsor_allowlist.rs::remove` (pending) | `sponsor_allowlist` row deleted by admin. Payload: `{allowlist_id, community_id?, person_pseudonym, removed_by_admin_pseudonym, removed_at}`. Per PRD §5.4. |
```

Update the Acceptance invariants count line: `**45** at v1-RT-r1 end (19 v0 + 4 Phase 6 + 2 v1-AD-a + 1 v1-AD-c + 6 v1-JM-a + 1 v1-JM-c + 5 v1-SL-a + 7 v1-RT-r1)`.

Update "Confirmed exempt" enumeration to include the 7 RT-r1 consts and downstream sub-phase mappings.

**GOTCHA:** ALL THREE files land in the SAME commit.

**GOTCHA (registry pre-landed-const exemption):** each row MUST have `(pending)` marker + downstream-sub-phase citation.

**GOTCHA:** `pub use` ordering is STRICT alphabetical.

**GOTCHA (count invariant — verify post-commit):**
- `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` -> **45**.
- `rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l` -> **45**.
- `rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort | uniq -d` -> empty.

### 10.10 e2e migration round-trip extension (Task 10)

**Mirror primary:** JM-a Task 10 + SL-a §10.8.

**Task 10 IMPLEMENT — single Edit-with-anchor on `crates/server/tests/e2e.rs`:**

- **Anchor 1:** `PHASE_1_MIGRATION_COUNT` declaration at line 1094. Bump 14 -> 18. Update doc-comment.
- **Anchor 2:** `phase1_migrations_round_trip` test fn at line 1054+. Extend post-condition probe loops:
  - Column-name assertions on `reputation_event`: add `"dedupe_key"`, `"source_event_type"`.
  - Column-name assertions on `sponsor_allowlist`: add `"added_by_admin_id"`, `"note"`. Verify `community_id` nullable.
  - Index-name assertions on `pg_indexes`: add `"reputation_event_dedupe_key_partial_idx"`.
  - pg_type-name assertion: add `"reputation_event_source_type"`. Post-down.sql, the type is GONE (clean drop).
  - `governance_config` row count delta: assert +26 after up.sql, restored after down.sql.

**No new test fns** — extending existing test only. Per `feedback_junior_worker_e2e_edit_hang.md`: e2e.rs at >12,000 lines remains worker-hang risk surface.

**GOTCHA (R7):** `phase1_migrations_round_trip` is a test fn; workspace-check workflow's `cargo test --no-run -p lemmy_server --test e2e` step compiles it.

**GOTCHA (per `feedback_lemmy_error_no_std_error.md`):** existing test fn returns `Result<(), Box<dyn Error>>`. Probes stay within that error type. If a new probe needs Lemmy-native error wrap, use Case B annotated closure: `.map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("{e}").into() })?`.

**GOTCHA:** `CREATE TYPE ... DROP TYPE` cycle is clean — assert `pg_type` row absence post-down.sql.

**GOTCHA (Edit-with-anchor):** `Grep` for `PHASE_1_MIGRATION_COUNT` first to capture line numbers; targeted `Edit` calls. No wholesale `Read` of the 12,000-line file.

## 11. Files to change

### `lemmy_db_schema_file` crate

- `crates/db_schema_file/src/enums.rs` — `ReputationEventSourceType` enum (9 variants). **Task 5**.
- `crates/db_schema_file/src/schema.rs` — `sql_types::ReputationEventSourceType` + `reputation_event` extension + `sponsor_allowlist` extension. **Task 6**.

### `lemmy_db_schema` crate

- `crates/db_schema/src/source/governance/reputation_event.rs` — extend `ReputationEvent` + `ReputationEventInsertForm`. **Task 7**.
- `crates/db_schema/src/source/governance/sponsor_allowlist.rs` — extend `SponsorAllowlist` + `SponsorAllowlistInsertForm`. **Task 7**.
- `crates/db_schema/src/source/governance/governance_log.rs` — 7 new `ENTRY_KIND_*` const declarations. **Task 9**.

### `lemmy_api` crate

- `crates/api/api/src/governance/governance_log.rs` — 7 new alphabetical `pub use` re-exports. **Task 9**.
- `crates/api/api/src/governance/config.rs` — 26 new `DEFAULT_*` consts + 26 match arms + 26 SEEDED_KEYS tuples + `EXPECTED_SEED_COUNT_V1_RT` + parity test extension + 26 CONFIG_KEY_METADATA entries. **Task 8**.

### `lemmy_server` crate

- `crates/server/tests/e2e.rs` — `PHASE_1_MIGRATION_COUNT` 14 -> 18 + post-condition probe lists in `phase1_migrations_round_trip`. **Task 10**.

### Migration files

- `migrations/2026-05-10-000000-0000_add_reputation_event_v1_columns/{up,down}.sql`. **Task 1**.
- `migrations/2026-05-10-000100-0000_extend_sponsor_allowlist_for_r1/{up,down}.sql`. **Task 2**.
- `migrations/2026-05-10-000200-0000_backfill_reputation_event_source_type/{up,down}.sql`. **Task 3**.
- `migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/{up,down}.sql`. **Task 4**.

### Meta files (rules + reports)

- `.claude/rules/governance-log-entry-kind-registry.md` — populate `reputation-tuning-v1 (reserved)` section + Acceptance invariants count 38 -> 45. **Task 9**.
- `.claude/PRPs/reports/v1-RT-r1-retro.md` — CREATE retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`. **Task 11**.

### Files explicitly NOT touched

- Any `crates/api/api/src/governance/<handler>.rs` file (no handler edits; r2/r3/r4/r5 land them).
- `crates/api/api/src/governance/reputation_snapshot.rs` (v0 calculator stays untouched until r2).
- `crates/api/api/src/governance/{sponsor_liability,submit_jury_vote,create_endorsement,apply_sponsor_liability,request_appeal}.rs`.
- `crates/routes/src/utils/scheduled_tasks.rs` (no scheduler wiring).
- `crates/api/routes/**`, `crates/api/api_common/**`, `crates/db_views/**`.
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.coderabbit.yaml`.
- `.github/workflows/**` (Shape G workflows already cover RT-r1 classes).
- `crates/db_schema/src/newtypes.rs` (per DQ #181 — both newtypes already exist).
- `docs/brehon-law-inspired-network/**`.
- The 3 v1-AD-a-shipped duplicate keys' rows in v1-AD-a's SEEDED_KEYS_WITH_CONSTS / DEFAULT_* / CONFIG_KEY_METADATA.
- `migrations/2026-04-22-000300-0000_seed_v1_config_keys/{up,down}.sql`.

## 12. NOT building in v1-RT-r1

- **r2 chained-halving decay calculator** — replaces `compute_applied_delta` per PRD §5.2.
- **r3 multi-source emitters** — weekly activity cron, dormancy cron, vote-outcome hook, evidence-quality hook + `flag-bad-faith` admin endpoint.
- **r4 sponsor-gate strategy match arms** in `create_endorsement` + r4 allowlist admin endpoints.
- **r5 instance-wide rollup cron + admin endpoint**.
- **r6 CodeRabbit issue carry-forward** (#19, #20, #21, #22, #31).
- **Per-event `applied_delta_snapshot` column** — v2-scope per PRD §9.1.
- **Composition strategies** (`age_or_surety_or_allowlist` etc) — explicitly rejected per OQ-020.
- **Negative-band-with-reintegration** — out per OQ-021.
- **Cross-instance reputation portability** — federation-v2/v3.
- **Backfill smoke test as a NEW e2e test fn** — defers (Task 10 extends existing test only).
- **Seeding the 3 v1-AD-a duplicates a second time** — out per planner DQ #187.
- **Editing v1-AD-a's `SEEDED_KEYS_WITH_CONSTS` to migrate ownership** — would break locked V1_AD = 27.

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task** per `feedback_pr_per_phase.md`. Each task header carries a `[P]` marker iff its **FILES** YAML `union(creates, modifies)` shares no path with any other `[P]`-marked task in the same cohort.

> **Cohort dispatch (advisor-side):** Cohort A is Tasks 1+2+3+4 (4-way parallel — 4 disjoint migration directories); Cohort B is Tasks 8+9+10 (3-way parallel — `config.rs`, governance-log dual-file + registry, `e2e.rs` — disjoint files). Tasks 0, 5, 6, 7, 11 are barriers.

> **Shape G (Layer G2 push-and-exit):** §13 task bodies do NOT inline cargo invocations. Each task ends with a push to the worker branch; impl-task subagent writes a `kind: "validate-pending"` DQ entry. Migration-touching tasks (T1, T2, T3, T4) additionally trigger `cargo-validate-migration.yml`.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready; confirm branch is `phase-v1-RT-r1`; confirm prior phase deliverables intact.

**FILES:**

```yaml
creates: []
modifies: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5):**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — branch verification
git branch --show-current
# EXPECT: phase-v1-RT-r1

# Probe 2 — governance-v0 baseline counts
echo "ENTRY_KIND_ count (expect 38):"
grep -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
echo "EXPECTED_SEED_COUNT total (expect 101):"
grep -nE '^pub const EXPECTED_SEED_COUNT' crates/api/api/src/governance/config.rs
echo "ReputationEventSourceType absence:"
grep -E 'ReputationEventSourceType' crates/db_schema_file/src/enums.rs && {
  echo "ERROR: enum already in enums.rs — branch contamination"; exit 1
} || echo "enums.rs clean"

# Probe 3 — schema baseline: reputation_event has no RT-r1 columns
grep -E 'dedupe_key|source_event_type' crates/db_schema_file/src/schema.rs && {
  echo "ERROR: RT-r1 columns already in schema.rs — branch contamination"; exit 1
} || echo "schema.rs clean"

# Probe 4 — sponsor_allowlist baseline: community_id NOT NULL still
grep -A 5 'sponsor_allowlist (id)' crates/db_schema_file/src/schema.rs | head -10
# EXPECT: community_id -> Int4, (NOT Nullable<Int4>)

# Probe 5 — DQ #181-#186 resolved status
python3 -c "
import json
d = json.load(open('.claude/decision-queue.json'))
for eid in [181, 182, 183, 184, 185, 186]:
    found = any(e['id'] == eid for e in d.get('resolved', []))
    print(f'DQ #{eid}: resolved={found}')
"
# EXPECT: DQ #181-#186 all resolved=True

# Probe 6 — Pre-existing v1-AD-a 3 duplicate keys present
grep -E "deltas.participation_weekly_active|participation.dormancy_window_days|deltas.participation_dormant" \
  migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql | wc -l
# EXPECT: 3 (per planner DQ #187)

# Probe 7 — concurrent-PR check
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("reputation_event\\.rs|sponsor_allowlist\\.rs|enums\\.rs|governance_log\\.rs|config\\.rs|migrations/2026-05-10")) | {number, title, headRefName}'
# EXPECT: empty output

# Probe 8 — Shape G workflow YAMLs lint-clean
yamllint .github/workflows/cargo-validate-workspace.yml \
         .github/workflows/cargo-validate-migration.yml \
         .github/workflows/cargo-test-e2e.yml > /tmp/rt-r1-task0-yamllint.log 2>&1
echo "yamllint exit: $?"
grep -E '\-\-features full|\-\-no-deps|\-D warnings' .github/workflows/cargo-validate-workspace.yml

# Probe 9 — PM-plugin-hooks-stable
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  grep -rqE "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done
echo "all 6 PM hooks present"

# Probe 10 — Newtype presence (per DQ #181)
grep -nE 'pub struct (ReputationEventId|SponsorAllowlistId)' crates/db_schema/src/newtypes.rs
# EXPECT: 2 lines (both newtypes already exist)
```

**EXPECT block:** Probes 0, 1, 5, 6, 7, 8, 9, 10 exit 0; Probes 2, 3, 4 confirm baseline absence of RT-r1 effects.

**No commit at Task 0** — verification only.

**COMMIT MESSAGE:** N/A.

### Task 1 [P]: CREATE migration `2026-05-10-000000-0000_add_reputation_event_v1_columns/{up,down}.sql`

**ACTION:** new migration directory. CREATE TYPE + ALTER TABLE ADD COLUMN + CREATE UNIQUE INDEX in up.sql; LIFO DROP in down.sql.

**FILES:**

```yaml
creates:
  - migrations/2026-05-10-000000-0000_add_reputation_event_v1_columns/up.sql
  - migrations/2026-05-10-000000-0000_add_reputation_event_v1_columns/down.sql
modifies: []
```

**IMPLEMENT:** per §10.1 skeleton verbatim.

**MIRROR:** `migrations/2026-04-15-100000-0000_add_governance_enums/up.sql`; `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql`.

**GOTCHA:** the directory MUST sort lexicographically AFTER the most recent migration on `governance-v0` at impl time (currently `2026-05-03-000100-0000_add_sponsor_liability_grace_window`). RT-r1 picks `2026-05-10-000000-0000`. Impl-task: re-verify at task start.

**GOTCHA:** the migration runs INSIDE a transaction (no `-- no-transaction` directive needed).

**GOTCHA:** RT-r1 ships zero JSONB columns. `feedback_postgres_jsonb_canonicalization.md` cite-only.

**Push and exit (Shape G):** push to `junior/<task-slug>`. impl-task captures workflow_run_id for `cargo-validate-workspace.yml` AND `cargo-validate-migration.yml`. Write 2 `kind: "validate-pending"` DQ entries per Recipe 1.

**COMMIT MESSAGE:** `feat(v1-RT-r1): add reputation_event v1 columns + reputation_event_source_type enum + partial unique index (task 1)`

### Task 2 [P]: CREATE migration `2026-05-10-000100-0000_extend_sponsor_allowlist_for_r1/{up,down}.sql`

**ACTION:** new migration directory. ALTER TABLE sponsor_allowlist (drop NOT NULL on community_id; add added_by_admin_id with sentinel default 1 then DROP DEFAULT; add note) in up.sql.

**FILES:**

```yaml
creates:
  - migrations/2026-05-10-000100-0000_extend_sponsor_allowlist_for_r1/up.sql
  - migrations/2026-05-10-000100-0000_extend_sponsor_allowlist_for_r1/down.sql
modifies: []
```

**IMPLEMENT:** per §10.2 skeleton.

**MIRROR:** `migrations/2026-04-22-000200-0000_add_case_applied_config_snapshot/up.sql`; JM-a Task 2.

**GOTCHA (per DQ #181):** EXTEND the existing v1-AD-a-shipped table; do NOT create.

**GOTCHA:** sentinel value 1 is `person.id = 1`. Safe assumption since v1-AD-a table is empty.

**GOTCHA (down.sql):** SET NOT NULL fails if NULL community_id rows exist. Documented.

**Push and exit (Shape G):** two workflow_run_ids. Two `kind: "validate-pending"` DQ entries.

**COMMIT MESSAGE:** `feat(v1-RT-r1): extend sponsor_allowlist (DROP NOT NULL community_id, add added_by_admin_id + note) per DQ #181 (task 2)`

### Task 3 [P]: CREATE migration `2026-05-10-000200-0000_backfill_reputation_event_source_type/{up,down}.sql`

**ACTION:** new migration directory. UPDATE precedence chain per DQ #184.

**FILES:**

```yaml
creates:
  - migrations/2026-05-10-000200-0000_backfill_reputation_event_source_type/up.sql
  - migrations/2026-05-10-000200-0000_backfill_reputation_event_source_type/down.sql
modifies: []
```

**IMPLEMENT:** per §10.3 skeleton.

**MIRROR:** JM-a §10.4 backfill UPDATE block.

**GOTCHA (per DQ #184):** use `source_case_id` + `reason ILIKE` precedence; do NOT reference non-existent `endorsement_id` / `jury_vote_id` columns.

**GOTCHA:** each UPDATE's `WHERE source_event_type = 'Endorsement'` makes it idempotent.

**GOTCHA:** Task 3 timestamp MUST sort AFTER Task 1.

**Push and exit (Shape G).**

**COMMIT MESSAGE:** `feat(v1-RT-r1): backfill reputation_event.source_event_type via reason ILIKE precedence per DQ #184 (task 3)`

### Task 4 [P]: CREATE migration `2026-05-10-000300-0000_seed_v1_rt_config_keys/{up,down}.sql`

**ACTION:** new migration directory. 26 INSERT rows + ON CONFLICT DO NOTHING in up.sql; 26-row LIFO DELETE in down.sql.

**FILES:**

```yaml
creates:
  - migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/up.sql
  - migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/down.sql
modifies: []
```

**IMPLEMENT:** per §10.4 skeleton verbatim.

#### Task 4 pre-commit reconciliation gate (mandatory)

```bash
# Count INSERT rows in Task 4 up.sql
grep -cE "^\s*\('instance', '" \
  migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/up.sql
# Expected: 26

# Count DELETE keys in Task 4 down.sql
grep -cE "^\s*'" \
  migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/down.sql
# Expected: 26

# Cross-check: every key in up.sql appears in down.sql
diff \
  <(grep -oE "'(decay|bounds|deltas|participation|job|feature)\\.[a-z_.]+'" migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/up.sql | sort -u) \
  <(grep -oE "'(decay|bounds|deltas|participation|job|feature)\\.[a-z_.]+'" migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/down.sql | sort -u)
# Expected: empty

# Negative check: no key in up.sql is one of the 3 v1-AD-a duplicates
grep -E "deltas.participation_weekly_active|participation.dormancy_window_days|deltas.participation_dormant" \
  migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/up.sql
# Expected: empty (RT-r1 must NOT re-seed v1-AD-a-shipped duplicates per DQ #187)
```

If any check fails, DO NOT commit.

**MIRROR:** §10.4; `migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql`.

**GOTCHA (per DQ #187):** the 3 duplicates are NOT in this migration.

**GOTCHA:** value_type discriminator: 24 rows are 'int', 2 rows are 'bool', 0 rows are 'float', 0 rows are 'text'.

**Push and exit (Shape G):** triggers both workflow YAMLs.

**COMMIT MESSAGE:** `feat(v1-RT-r1): seed 26 reputation-tuning governance_config rows — idempotent + matches SEEDED_KEYS (task 4)`

### Task 5: UPDATE `crates/db_schema_file/src/enums.rs` — add `ReputationEventSourceType`

**ACTION:** append `ReputationEventSourceType` after `ReputationDimension` (line 615).

**Cohort B barrier.** Sequential ordering required.

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema_file/src/enums.rs
```

**IMPLEMENT:** per §10.5 skeleton.

**MIRROR:** `crates/db_schema_file/src/enums.rs:598-615` (`ReputationDimension`). DO NOT mirror `MembershipState`.

**IMPORTS:** existing imports cover `Serialize, Deserialize, DbEnum, ts_rs` — no additions.

**GOTCHA:** `#[default]` MUST be `Endorsement`.

**Push and exit (Shape G):** workspace-check workflow validates `cargo check --workspace --features full`.

**COMMIT MESSAGE:** `feat(v1-RT-r1): add ReputationEventSourceType Rust enum — 9 variants matching reputation_event_source_type pg_type (task 5)`

### Task 6: UPDATE `crates/db_schema_file/src/schema.rs` — sql_types + table! extensions

**ACTION:** Hand-edit `@generated` schema.rs. Three sub-edits per §10.6.

**Cohort B barrier.** Depends on Task 5.

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema_file/src/schema.rs
```

**IMPLEMENT:** per §10.6 skeleton (3 sub-edits).

**MIRROR:** `schema.rs:118` (sql_types); `schema.rs:1218-1229`; `schema.rs:1338-1343`.

**GOTCHA:** Hand-edit `@generated` per governance convention. Document with inline comments.

**GOTCHA:** `joinable!` block at `schema.rs:1466-1482` is NOT edited.

**GOTCHA (per DQ #182):** `governance/` subdirectory paths correct (Task 7).

**Push and exit (Shape G).**

**COMMIT MESSAGE:** `feat(v1-RT-r1): extend schema.rs — sql_types::ReputationEventSourceType + reputation_event/sponsor_allowlist column changes (task 6)`

### Task 7: UPDATE `crates/db_schema/src/source/governance/{reputation_event,sponsor_allowlist}.rs`

**ACTION:** Two sub-edits in two files (single commit) per §10.7.

**Cohort B barrier.** Depends on Task 6.

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema/src/source/governance/reputation_event.rs
  - crates/db_schema/src/source/governance/sponsor_allowlist.rs
```

**IMPLEMENT:** per §10.7 verbatim.

**MIRROR:** existing files at `:1-42` and `:1-37`; JM-a Task 5.

**IMPORTS:** in `reputation_event.rs`, extend `enums::{ReputationDimension}` to `enums::{ReputationDimension, ReputationEventSourceType}`. In `sponsor_allowlist.rs`, no new imports needed.

**GOTCHA:** `ReputationEventInsertForm.source_event_type` is `Option<>`; `SponsorAllowlistInsertForm.added_by_admin_id` is `PersonId` (NOT `Option<>`).

**GOTCHA:** verify at task start by `rg "SponsorAllowlistInsertForm" crates/`. Should return zero results outside the file. If a caller appears, file a DQ pending entry.

**GOTCHA:** `AsChangeset` derive: KEEP on `ReputationEventInsertForm`; DO NOT add to `SponsorAllowlistInsertForm`.

**Push and exit (Shape G).**

**COMMIT MESSAGE:** `feat(v1-RT-r1): extend Diesel models — ReputationEvent + SponsorAllowlist with new fields per DQ #181/#182 (task 7)`

### Task 8 [P]: UPDATE `crates/api/api/src/governance/config.rs`

**ACTION:** Six sub-edits per §10.8.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/config.rs
```

**IMPLEMENT:** per §10.8 verbatim.

#### Task 8 pre-commit reconciliation gate (mandatory)

```bash
# Count new SEEDED_KEYS_WITH_CONSTS v1-RT-r1 tuples
awk '/v1-RT-r1 additions/,/^];$/' crates/api/api/src/governance/config.rs | \
  grep -cE '^\s*\("'
# Expected: 26

# Count new DEFAULT_* declarations
awk '/-- v1-RT-r1 additions/,/^pub fn const_default_int/' crates/api/api/src/governance/config.rs | \
  grep -cE '^pub const DEFAULT_'
# Expected: 26

# Cross-check: SEEDED_KEYS keys match Task 4 up.sql keys exactly
diff \
  <(awk '/v1-RT-r1 additions/,/^];$/' crates/api/api/src/governance/config.rs | grep -oE '"[a-z_.]+"' | grep -E '^"(decay|bounds|deltas|participation|job|feature)\\.' | sort -u | tr -d '"') \
  <(grep -oE "'(decay|bounds|deltas|participation|job|feature)\\.[a-z_.]+'" migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/up.sql | sort -u | tr -d "'")
# Expected: empty
```

If any count disagrees, DO NOT commit.

**MIRROR:** §10.8; v1-SL-a §10.6 + v1-JM-a §10.9.

**IMPORTS:** existing imports cover all types — no additions.

**GOTCHA (R11):** parametric pattern mandatory. Do NOT bump locked counts.

**GOTCHA (per DQ #187):** the 3 v1-AD-a duplicates NOT in v1-RT-r1 SEEDED_KEYS block.

**GOTCHA (per `feedback_features_full_p_crate_incompatible.md`):** `--workspace --features full`, never `-p`.

**Push and exit (Shape G).**

**COMMIT MESSAGE:** `feat(v1-RT-r1): config.rs — 26 new reputation-tuning consts + metadata + SEEDED_KEYS + EXPECTED_SEED_COUNT_V1_RT parity (task 8)`

### Task 9 [P]: UPDATE governance_log dual-file + registry

**ACTION:** Three-file atomic commit per §10.9.

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema/src/source/governance/governance_log.rs
  - crates/api/api/src/governance/governance_log.rs
  - .claude/rules/governance-log-entry-kind-registry.md
```

**IMPLEMENT:** per §10.9 verbatim.

**MIRROR:** §10.9; v1-SL-a Task 7.

**GOTCHA:** ALL THREE files in the SAME commit.

**GOTCHA (registry pre-landed-const exemption):** each row MUST have `(pending)` marker + downstream-sub-phase citation.

**GOTCHA (count invariant — verify post-commit):** db_schema count = 45; api shim count = 45; uniqueness = empty.

**GOTCHA:** `pub use` ordering is STRICT alphabetical.

**Push and exit (Shape G).**

**COMMIT MESSAGE:** `feat(v1-RT-r1): add 7 ENTRY_KIND consts (db_schema define + api shim re-export) + registry v1-RT-r1 section (task 9)`

### Task 10 [P]: UPDATE `crates/server/tests/e2e.rs` — extend `phase1_migrations_round_trip`

**ACTION:** Single Edit-with-anchor per §10.10. Two sub-edits in one Edit call:
1. Bump `PHASE_1_MIGRATION_COUNT` from 14 to 18.
2. Extend post-condition probe lists per §10.10.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
```

**IMPLEMENT:** per §10.10. Use Edit-with-anchor — `Grep` for `PHASE_1_MIGRATION_COUNT` first.

**MIRROR:** §10.10; JM-a Task 10 + SL-a Task 8.

**GOTCHA (R7):** `phase1_migrations_round_trip` is a test fn; workspace-check workflow's `cargo test --no-run` step compiles it.

**GOTCHA (per `feedback_lemmy_error_no_std_error.md`):** existing test fn returns `Result<(), Box<dyn Error>>`. Probes stay within that error type.

**GOTCHA:** RT-r1's `CREATE TYPE ... DROP TYPE` cycle is clean — assert `pg_type` row absence post-down.sql.

**Push and exit (Shape G):** workspace-check runs `cargo test --no-run`. Phase 2 e2e runs after Junior daemon finalize-merges into `phase-v1-RT-r1`.

**COMMIT MESSAGE:** `test(v1-RT-r1): extend phase1_migrations_round_trip — bump count to 18 + new schema effects (task 10)`

### Task 11: WRITE `.claude/PRPs/reports/v1-RT-r1-retro.md` — retrospective before PR

**ACTION:** Author retrospective per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`.

**Cohort barrier.** Depends on Tasks 1-10.

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-RT-r1-retro.md
modifies: []
```

**IMPLEMENT:** mirror `.claude/PRPs/reports/v1-SL-c-2-retro.md`. Document specifically:

- The **planner DQ #187 discovery** — was the brief author's PRD-mechanical V1_RT count of 29 a planner-time miss or advisor-time miss? Recommend a `feedback_*` lesson if pattern reproduces.
- Whether all `[P]` cohorts parallelised cleanly on the EliteDesk daemon.
- Whether §13 task ordering minimised the number of barriers.
- Phase 2 e2e wall-clock under Shape G (local vs dispatch).
- Any drift from `feedback_explicit_file_arrays_on_tasks.md`.

**MIRROR:** `.claude/PRPs/reports/v1-SL-c-2-retro.md`.

**GOTCHA:** retro is written BEFORE `gh pr create`.

**GOTCHA:** §4 per-task complexity-score table is mandatory.

**Push and exit (Shape G — retro is meta-work).**

**COMMIT MESSAGE:** `docs(v1-RT-r1): phase retrospective before PR open (task 11)`

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cargo check --workspace --features full` via `cargo-validate-workspace.yml`.
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings` via the same workflow.
- **Test target compile (R7):** `cargo test --no-run -p lemmy_server --test e2e` via the same workflow.
- **Migration round-trip:** `bash scripts/brehon/migrate-roundtrip.sh` via `cargo-validate-migration.yml`. Triggers on Tasks 1, 2, 3, 4 pushes.
- **e2e round-trip (Phase 2):** `phase1_migrations_round_trip` test extended by Task 10.
- **Parity tests** (`#[cfg(test)] mod parity` in config.rs): runs as part of workspace-check chain. Catches Task 4 + Task 8 count drift.

Pre-merge advisor-side verification: `/brehon-verify` Brief-Scope outputs check.

---

## 15. Validation commands (DoD)

> **Shape G plan — DoD is per-workflow, not inline cargo.**

### 15.1 Per-task workspace check (Shape G)

For every §13 impl task that modifies `crates/**` (Tasks 5, 6, 7, 8, 9, 10) and every migration-touching task (Tasks 1, 2, 3, 4):

- **DoD entry:** `cargo-validate-workspace.yml` on `junior/<task-slug>` SHA `<sha>` -> `conclusion: "success"`
- **Validation command:** `gh run list --repo barrie-cork/lemmy --branch <branch> --workflow cargo-validate-workspace --limit 1 --json conclusion,databaseId --jq '.[0]'`
- **EXPECT:** `{"conclusion": "success", "databaseId": <id>}`

### 15.2 Migration round-trip (Shape G — Tasks 1, 2, 3, 4 only)

- **DoD entry:** `cargo-validate-migration.yml` on `junior/<task-slug>` SHA `<sha>` -> `conclusion: "success"`
- **Validation command:** `gh run list --repo barrie-cork/lemmy --branch <branch> --workflow cargo-validate-migration --limit 1 --json conclusion,databaseId --jq '.[0]'`
- **EXPECT:** `{"conclusion": "success", "databaseId": <id>}`

### 15.3 Phase 2 e2e (post-finalize-merge)

After all impl tasks finalize-merge into `phase-v1-RT-r1`, the advisor surfaces the **Phase 2 e2e local-vs-dispatch user gate**:

- **(a) local:** `cargo test -p lemmy_server --test e2e --features full -- --test-threads=1` on laptop in `run_in_background`; ~26 min wall-clock; zero billed.
- **(b) dispatch:** `gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-RT-r1`; ci-watcher polls; ~26 min billed.

Plan-side DoD: e2e exit code 0; failure path -> §G4 classifier on log slice.

### 15.4 Cross-cutting verification (Task 11 retro time)

- [ ] `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns **45** (38 + 7).
- [ ] `rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l` returns **45**.
- [ ] `rg -n '\(pending\)' .claude/rules/governance-log-entry-kind-registry.md` shows the 7 RT-r1 rows have `(pending)` markers with downstream-sub-phase citations.
- [ ] `grep -c "^\\s*\\('instance'," migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/up.sql` returns **26**.
- [ ] `grep -c '^pub const EXPECTED_SEED_COUNT_V1_RT' crates/api/api/src/governance/config.rs` returns **1**.
- [ ] `crates/db_schema_file/src/enums.rs ReputationEventSourceType` declaration has 9 variants.
- [ ] `SEEDED_KEYS_WITH_CONSTS.len() == 127` (= 34 + 27 + 27 + 13 + 26).
- [ ] R6: clippy invocations in `cargo-validate-workspace.yml` use `--no-deps -- -D warnings`.
- [ ] No edits to files outside §11 list.
- [ ] Every RT-r1 §16a story is `[done]`.
- [ ] No new newtype additions (per DQ #181).

### 15.5 ADR / OQ compliance verification

- [ ] ADR-005 honoured: per-dimension reputation tracking unchanged; new `source_event_type` is per-event meta.
- [ ] ADR-008 honoured: governance_log entry-kind is TEXT; 7 new consts added.
- [ ] ADR-010 honoured (with PRD §9.1 weakening): feature flag controls v0-vs-v1 calculator dispatch.
- [ ] ADR-013 honoured: `EVIDENCE_QUALITY_RECORDED` const declared.
- [ ] ADR-015 honoured: governance_log emissions (future) will use `actor_pseudonym`.
- [ ] OQ-001 progressed: r5 will populate `reputation_snapshot WHERE community_id IS NULL` rows.
- [ ] OQ-018 dependency satisfied: v1-AD-b PR #76 merge `f03ed1cba`.
- [ ] OQ-019 progressed: declares `PARTICIPATION_CRON_TICK` + `VOTE_OUTCOME_RECORDED` consts.
- [ ] OQ-020 progressed: `sponsor_allowlist` schema substrate r4 needs.

---

## 16. Acceptance criteria

- [ ] All 11 impl tasks completed in dependency order (Task 0 pre-flight, no commit; Tasks 1-11 each one commit).
- [ ] §15.1 `cargo-validate-workspace.yml` exit-success on every code-touching task push.
- [ ] §15.2 `cargo-validate-migration.yml` exit-success on every migration-touching task push (Tasks 1, 2, 3, 4).
- [ ] §15.3 Phase 2 e2e exit 0 on phase-branch tip.
- [ ] §15.4 cross-cutting verification — all 11 boxes ticked.
- [ ] §15.5 ADR / OQ compliance — all 9 boxes ticked.
- [ ] §16a stories — all stories `[done]`.
- [ ] No edits to files outside §11 list.
- [ ] Retro committed at `.claude/PRPs/reports/v1-RT-r1-retro.md` BEFORE `gh pr create`.
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`.
- [ ] CodeRabbit review complete with findings triaged.

---

## 16a. Stories (independently-testable behaviour units)

> **Why this section exists:** RT-r1 has 11 impl tasks; story-grain decomposition catches "task 8 broke task 4 invariant" earlier and enables `/brehon-verify` to surface phantom completions.

### Story 1: Migrations round-trip cleanly under Shape G

- **Composing tasks:** Task 1, Task 2, Task 3, Task 4 (Cohort A — 4-way `[P]`).
- **Checkpoint command:** `gh run list --repo barrie-cork/lemmy --branch phase-v1-RT-r1 --workflow cargo-validate-migration --limit 1 --json conclusion --jq '.[0]'`
- **Expected output:** `{"conclusion": "success"}`
- **Brief-Scope outputs to verify:**
  - `migrations/2026-05-10-000000-0000_add_reputation_event_v1_columns/up.sql` exists + non-empty + contains `CREATE TYPE reputation_event_source_type`
  - `migrations/2026-05-10-000100-0000_extend_sponsor_allowlist_for_r1/up.sql` exists + non-empty + contains `ALTER TABLE sponsor_allowlist ALTER COLUMN community_id DROP NOT NULL`
  - `migrations/2026-05-10-000200-0000_backfill_reputation_event_source_type/up.sql` exists + non-empty + contains `WHERE source_event_type = 'Endorsement'`
  - `migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/up.sql` exists + non-empty + has exactly 26 INSERT rows

### Story 2: Rust schema layer compiles with new column shapes

- **Composing tasks:** Task 5 (enum), Task 6 (schema.rs), Task 7 (Diesel structs).
- **Checkpoint command:** `gh run list --repo barrie-cork/lemmy --branch phase-v1-RT-r1 --workflow cargo-validate-workspace --limit 1 --json conclusion --jq '.[0]'`
- **Expected output:** `{"conclusion": "success"}`
- **Brief-Scope outputs to verify:**
  - `crates/db_schema_file/src/enums.rs` contains `pub enum ReputationEventSourceType` with 9 variants
  - `crates/db_schema_file/src/schema.rs` `reputation_event` block declares `dedupe_key -> Nullable<Text>` and `source_event_type -> ReputationEventSourceType`
  - `crates/db_schema_file/src/schema.rs` `sponsor_allowlist` block declares `community_id -> Nullable<Int4>` (NOT `Int4`) and `added_by_admin_id -> Int4` and `note -> Nullable<Text>`
  - `crates/db_schema/src/source/governance/reputation_event.rs` `ReputationEvent` struct has `dedupe_key: Option<String>` + `source_event_type: ReputationEventSourceType`
  - `crates/db_schema/src/source/governance/sponsor_allowlist.rs` `SponsorAllowlist` struct has `community_id: Option<CommunityId>` + `added_by_admin_id: PersonId` + `note: Option<String>`

### Story 3: Config + governance-log surface fully landed with parity invariants green

- **Composing tasks:** Task 8 (config.rs), Task 9 (governance-log dual-file + registry).
- **Checkpoint command:** `gh run list --repo barrie-cork/lemmy --branch phase-v1-RT-r1 --workflow cargo-validate-workspace --limit 1 --json conclusion --jq '.[0]'`
- **Expected output:** `{"conclusion": "success"}`
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/config.rs` declares `pub const EXPECTED_SEED_COUNT_V1_RT: usize = 26;`
  - `crates/api/api/src/governance/config.rs` parity test at line 2692-2710 includes `+ EXPECTED_SEED_COUNT_V1_RT`
  - `crates/api/api/src/governance/config.rs` v1-RT-r1 SEEDED_KEYS additions block has exactly 26 tuples
  - `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns 45
  - `rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l` returns 45
  - `.claude/rules/governance-log-entry-kind-registry.md` `## v1-RT-r1 entry kinds` section is populated with 7 rows + Acceptance invariants count is 45

### Story 4: e2e round-trip + post-condition probes pass

- **Composing tasks:** Task 10 (e2e.rs).
- **Checkpoint command:** `cargo test -p lemmy_server --test e2e phase1_migrations_round_trip --features full -- --test-threads=1` (local) OR Phase 2 e2e workflow (dispatch).
- **Expected output:** `1 passed; 0 failed`.
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` `PHASE_1_MIGRATION_COUNT` = 18
  - `phase1_migrations_round_trip` post-condition probe lists include `dedupe_key`, `source_event_type`, `reputation_event_dedupe_key_partial_idx`, `reputation_event_source_type` (pg_type), `added_by_admin_id`, `note`, governance_config row count delta +26

> **Verification mapping:** advisor `/brehon-verify` step iterates this section, runs each Story Checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0-10 all green)
- [ ] Tasks 1-11 each shipped one commit with `feat(v1-RT-r1): ... (task N)` / `test(v1-RT-r1): ... (task N)` / `docs(v1-RT-r1): ... (task N)` subject pattern
- [ ] Task 4 + Task 8 reconciliation gates passed (26 = 26 = 26)
- [ ] §15 validation green at every gate
- [ ] §16a stories all `[done]`
- [ ] Retro committed at `.claude/PRPs/reports/v1-RT-r1-retro.md`
- [ ] No file under `crates/api/api/src/governance/<handler>.rs` modified (except dual-file `governance_log.rs` re-export shim, which is non-handler)
- [ ] No file under `crates/api/routes/` modified
- [ ] No file under `crates/api/api_common/` modified
- [ ] No file under `crates/db_views/` modified
- [ ] No new newtype added under `crates/db_schema/src/newtypes.rs`
- [ ] No edit to `migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql` or `down.sql`
- [ ] PR opened by BM session against `governance-v0` with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review complete with findings triaged
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-RT-r1-verify.md` shows all stories check

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Plan-count drift: 26 estimated, actual count differs | LOW | LOW | Task 4 + Task 8 reconciliation gates MANDATORY pre-commit. |
| The 3 v1-AD-a duplicates re-enter v1-RT-r1 SEEDED_KEYS block | LOW | MED | Task 4 negative grep + Task 8 reconciliation both fail-loud. Plan §10.4/§10.6/§10.8 cite DQ #187 explicitly. |
| Backfill UPDATE misclassifies a row (reason doesn't match expected pattern) | LOW | LOW | Heuristic best-effort per PRD §7. r3 emitters write source_event_type explicitly going forward. |
| Dual-file ENTRY_KIND edit lands asymmetrically | LOW | MED | Task 9 single-commit-three-files; CI count check catches asymmetry. |
| `sponsor_allowlist` ALTER (community_id DROP NOT NULL) fails on pre-populated table | NONE | n/a | v1-AD-a doc-comment confirms table empty. Task 0 Probe 4 confirms. |
| `sponsor_allowlist` ALTER (down.sql SET NOT NULL) fails post-r4 | LOW | LOW | Documented in down.sql comment. |
| 7 ENTRY_KIND consts pre-landed without downstream call sites | EXPECTED | n/a | Pre-landed-const exemption per registry rule. |
| Cohort A 4-way [P] dispatch hits worktree-fan-out race | LOW | LOW | Per-task worktree per `feedback_parallel_agents_one_worktree_per_agent.md`. |
| Concurrent v1-SL-d planning task on `governance-v0` modifies a RT-r1 file | LOW | LOW | Brief §0 confirmed zero handler-file overlap. Task 0 Probe 7 catches active drift. |
| `ReputationEventInsertForm.source_event_type: Option<>` lets r3 emitters omit field | LOW | MED | InsertForm doc-comment cites PRD section r3 readers should consult. |
| Advisor pre-plan-approval DoD smoke test discovers a §15 command unexecutable | LOW | LOW | §15 entries are workflow-shape DoDs; advisor check is `gh run list ...`. |

---

## 19. Notes

- **Planner DQ #187 (filed in this plan-write commit).** During planning §13 Task 4 enumeration, the planner discovered that 3 of the 29 PRD §8 rows are already shipped under v1-AD-a's seed migration (`migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql:37-39`):
  - `deltas.participation_weekly_active` (default 1 — matches PRD §8 row 17)
  - `participation.dormancy_window_days` (default 30 — matches PRD §8 row 21)
  - `deltas.participation_dormant` (default -2 — matches PRD §8 row 20)

  Re-seeding via RT-r1 migration would attempt 3 redundant `INSERT INTO governance_config` calls (`ON CONFLICT DO NOTHING` swallows them), but adding their entries to `SEEDED_KEYS_WITH_CONSTS` v1-RT-r1 block AND to `CONFIG_KEY_METADATA` would break the parity test (`every_seeded_key_has_metadata` enforces 1-to-1 length match between the two arrays; duplicate keys are fatal).

  Resolution (planner-self-resolved per `decision-queue.md` Recipe 3): RT-r1 seed migration ships **26 net-new rows**. `EXPECTED_SEED_COUNT_V1_RT = 26`. Cumulative: 34 + 27 + 27 + 13 + 26 = 127. The 3 duplicates remain owned by V1_AD's count — they conceptually belong to RT-tuning, but count-bookkeeping reflects ship history. The DQ #185 advisor answer (V1_RT = 29) is treated as "logical count per PRD §8 verbatim"; the planner's count revision (V1_RT = 26) reflects ship-history accuracy. The advisor will see this DQ #187 entry on the next polling tick and may reject (forcing different resolution like ownership migration); default behaviour is acceptance.

- **Advisor directive #4 (parametric pattern).** Per `crates/api/api/src/governance/config.rs:1395-1401`: each v1 sub-PRD adds its own `EXPECTED_SEED_COUNT_V1_*` without churning others. RT-r1 honours.

- **Why r1 ships 7 entry-kind consts when only 1 (`DECAY_KNOB_CHANGED`) fires before r3.** Per registry pre-landed-const exemption (used by JM-a + SL-a precedent): pre-landing in r1 prevents future plans from shipping consts piecemeal across r2/r3/r4/r5.

- **Feature flag seeded `false` on purpose.** Per PRD §9: operators flip via dashboard after evaluating defaults. Mirrors v1-SL-a precedent.

- **Future GH issue candidates** (per `feedback_retro_not_report.md`):
  - `v1.5-candidate`: snapshot-column posture for `reputation_event` (currently v2 per PRD §9.1).
  - `v2-candidate`: cross-instance reputation portability.
  - `v1.5-candidate`: `sponsor_allowlist` UI surface in admin dashboard.

---

## 20. Confidence score

**8/10** for one-pass implementation success.

**Rationale:**

- **Plus**: Every task mirrors a specific JM-a / SL-a task byte-for-byte. Pattern is fresh and reproducible. Task 4 + Task 8 reconciliation gates catch the only realistic count-drift class. Migration set is mechanical; no business logic. The 7 ENTRY_KIND consts are declared-only. Diesel struct extensions follow `feedback_insertform_default_propagation.md` precisely.
- **Minus**: Planner DQ #187 (the 3-duplicate revision) is a planner-time discovery; if the advisor disagrees and forces V1_RT = 29 with ownership-migration, the plan needs §10.4 + §10.6 + §10.8 + §13 Task 4 + Task 8 revisions. Task 7's `SponsorAllowlistInsertForm.added_by_admin_id` change may break a hypothetical v1-AD-a-era caller (verified zero callers at plan-write). Cohort A 4-way [P] is one beyond the 3-way SL-a precedent.
- **Risk floor**: `cargo check --workspace --features full` + `cargo clippy --no-deps -- -D warnings` are early-fail signals.

**Next step:** advisor approves plan after §15 DoD smoke test + §3.5 watchpoint specificity gate. On approval, queue `bm-cut` for `phase-v1-RT-r1`, then Task 0 + Cohort A.
