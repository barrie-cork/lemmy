---
phase: v1-RT-r1
role: impl-task
task: 8-9-10-bundle
brief_n: 8-9-10
authored: 2026-05-12
parent_phase_tip: "<post-fix-impl-2 tip — confirm at task-creation>"
parallel_cohort: B-final (consolidates plan §13 Tasks 8+9+10)
companion_brief: rt-r1-fix-impl-2.md
---

# [role:impl-task] v1-RT-r1 Cohort B-final BUNDLED — config.rs + governance-log + e2e.rs — see .claude/PRPs/briefs/rt-r1-impl-8-9-10-bundle.md

## §1 Role + dispatch

`[role:impl-task] v1-RT-r1 cohort B-final bundled — Tasks 8 (config.rs 26 consts) + 9 (governance-log dual-file + registry) + 10 (e2e.rs phase1_migrations_round_trip extend) in one commit`

## §2 Scope — BUNDLED (Tasks 8 + 9 + 10 in one Junior task, one commit)

**Why bundled (not three parallel `[P]` tasks):** Per v1-RT-r1 halt retro `.claude/PRPs/reports/v1-RT-r1-halt-retro.md` L1 + user decision 2026-05-12. The plan §13 `[P]` markers for Tasks 8/9/10 are conceptually correct (FILES YAML arrays are disjoint — config.rs, governance_log.rs ×2 + registry.md, e2e.rs), but recurring cohort-isolation bugs in this phase (Tasks 1-4 Cohort A, Tasks 5/6/7 Cohort B-serial+bundled) have shifted strategy toward bundling tasks whose validation is correlated. Tasks 8/9/10 share a single workspace-check workflow (no migration touched), so bundling means one commit → one push → one workflow → one ci-watcher. Bookkeeping reduction outweighs lost parallelism on a 3-task cohort.

**Strategy ladder applied:** fix-impl-2 (mechanical 8-callsite pad) runs FIRST and ships separately. This bundle picks up the workspace-check-clean tip after fix-impl-2 ci-watcher passes, then bundles Tasks 8+9+10. Two Junior tasks total; two cohort cycles.

### §2.1 Sub-edit 1: `crates/api/api/src/governance/config.rs` (Task 8, plan §10.8)

**SIX sub-edits in one file.** All per plan §10.8 verbatim.

**(a) DEFAULT_* consts (26 new):** insert as a NEW `// -- v1-RT-r1 additions ...` block AFTER the existing v1-SL-a additions block. Exact text per plan §10.8 lines 838-887. All 26 consts named `DEFAULT_*` with explicit types (`i64` for 24 keys, `bool` for 2 keys).

**(b) Match arms across `const_default_int` (24 arms) + `const_default_bool` (2 arms):** add per plan §10.8 sub-edit 2. `const_default_float` + `const_default_text` get 0 new arms.

**(c) SEEDED_KEYS_WITH_CONSTS extension (26 tuples):** append as a NEW `// v1-RT-r1 additions ...` block AFTER the v1-SL-a block, alphabetised within the new block. Exact tuples per plan §10.8 lines 899-925.

**(d) EXPECTED_SEED_COUNT_V1_RT (4 consts):** insert AFTER `EXPECTED_SEED_COUNT_V1_SL`. Three new consts per plan §10.8 sub-edits 4 + 4b:
- `EXPECTED_SEED_COUNT_V1_RT_NETNEW: usize = 26;`
- `EXPECTED_SEED_COUNT_V1_RT_LOGICAL: usize = 29;`
- `EXPECTED_SEED_COUNT_V1_RT: usize = EXPECTED_SEED_COUNT_V1_RT_NETNEW;` (backwards-compat alias bound to NETNEW)

**(e) Parity test extension (lines 2692-2710):** extend the expected sum to `EXPECTED_SEED_COUNT + V1_AD + V1_JM + V1_SL + V1_RT_NETNEW = 34 + 27 + 27 + 13 + 26 = 127`. Update format-string to include `V1_RT_NETNEW`. Add compile-time `assert_eq!(EXPECTED_SEED_COUNT_V1_RT_LOGICAL, EXPECTED_SEED_COUNT_V1_RT_NETNEW + 3, "...")` inside the parity-test fn body.

**(f) CONFIG_KEY_METADATA (26 new entries):** append BEFORE closing `];` of the array. Per-key apply_at / scope / valid_range / description / doc_anchor per plan §10.8 sub-edit 6 lines 967-987. All 26: `requires_re_jury: false`, `requires_step_up: false`, `doc_anchor: "v1-reputation-tuning.prd.md section 8"`.

### §2.2 Sub-edit 2: governance-log dual-file + registry (Task 9, plan §10.9)

**THREE files atomically (in this same bundle commit).** Per plan §10.9 verbatim.

**(a) `crates/db_schema/src/source/governance/governance_log.rs`:** append AFTER the v1-SL-a block (last existing const is `ENTRY_KIND_SPONSOR_LIABILITY_PENDING`). 7 new `ENTRY_KIND_*` consts per plan §10.9 lines 1014-1020. Each is `pub const ENTRY_KIND_<NAME>: &str = "<snake_case>"`. Block lead-in comment per plan §10.9 lines 1003-1013.

**(b) `crates/api/api/src/governance/governance_log.rs`:** insert 7 new `pub use` re-exports into the existing alphabetical block. EXACT positions per plan §10.9 lines 1024-1031:
- `DECAY_KNOB_CHANGED` between `CASE_DECIDED` and `EMERGENCY_REMOVED`
- `EVIDENCE_QUALITY_RECORDED` between `ENDORSEMENT_REVOKED` and `FEDERATION_ATTESTATION_RECEIVED`
- `PARTICIPATION_CRON_TICK` between `PANEL_ASSEMBLED` and `PUBLIC_LOG_PUBLISHED`
- `ROLLUP_RECOMPUTED` between `RESTORATION_COMPLETED` and `RULE_SET_VERSION_CREATED`
- `SPONSOR_ALLOWLIST_ADDED` between `SEVERITY_TIER_FROZEN` and `SPONSOR_LIABILITY_APPLIED`
- `SPONSOR_ALLOWLIST_REMOVED` immediately after `SPONSOR_ALLOWLIST_ADDED`, before `SPONSOR_LIABILITY_APPLIED`
- `VOTE_OUTCOME_RECORDED` between `THRESHOLD_MET` and the closing `};`

**(c) `.claude/rules/governance-log-entry-kind-registry.md`:** replace the `### reputation-tuning-v1 (reserved — §7 of PRD enumerates 7 new kinds)` stub with the populated section per plan §10.9 lines 1037-1053. Update Acceptance invariants count from `38` → `45`. Update "Confirmed exempt" enumeration to include the 7 RT-r1 consts and downstream sub-phase mappings.

### §2.3 Sub-edit 3: `crates/server/tests/e2e.rs` extend `phase1_migrations_round_trip` (Task 10, plan §10.10)

**Single Edit-with-anchor approach, two sub-anchors. NO new test fns.** Per plan §10.10 verbatim.

**Anchor 1 — `PHASE_1_MIGRATION_COUNT` declaration at ~line 1094:** bump `14` → `18`. Update the inline doc-comment to reflect v1-RT-r1's 4 new migrations.

**Anchor 2 — `phase1_migrations_round_trip` test fn body at ~line 1054+:** extend post-condition probe lists:
- Column-name assertions on `reputation_event`: add `"dedupe_key"`, `"source_event_type"`.
- Column-name assertions on `sponsor_allowlist`: add `"added_by_admin_id"`, `"note"`. Verify `community_id` nullable.
- Index-name assertions on `pg_indexes`: add `"reputation_event_dedupe_key_partial_idx"`.
- `pg_type` row absence post-down.sql: add `"reputation_event_source_type"` (CREATE TYPE / DROP TYPE cycle is clean).
- `governance_config` row count delta: assert +26 after up.sql, restored after down.sql.

**Pre-commit reconciliation gate (mandatory, Task 8 portion):**

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
  <(awk '/v1-RT-r1 additions/,/^];$/' crates/api/api/src/governance/config.rs | grep -oE '"[a-z_.]+"' | grep -E '^"(decay|bounds|deltas|participation|job|feature)\.' | sort -u | tr -d '"') \
  <(grep -oE "'(decay|bounds|deltas|participation|job|feature)\.[a-z_.]+'" migrations/2026-05-10-000300-0000_seed_v1_rt_config_keys/up.sql | sort -u | tr -d "'")
# Expected: empty
```

**Pre-commit count invariants (Task 9 portion):**

```bash
rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs   # Expected: 45
rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l               # Expected: 45
rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort | uniq -d
# Expected: empty (no duplicates)
```

**If any count disagrees, DO NOT commit. File a DQ blocker.**

### §2.4 Validation

After committing all three sub-edits as a single bundle commit:

1. Push the worker branch.
2. The push triggers `cargo-validate-workspace.yml` (path filter `crates/**` matches; `.claude/rules/**` does NOT trigger any workflow, so the registry edit is meta-only).
3. **Raise a NEW `kind: "validate-pending"` DQ entry** with the fresh `workflow_run_id` from:

```bash
gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json databaseId
```

## §3 Required reading

**Mandatory per file-class table (`.claude/rules/advisor-orchestrator.md` §2.4):**

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — e2e.rs edit; existing `phase1_migrations_round_trip` returns `Result<(), Box<dyn Error>>`. Probes stay within that error type. If a new probe needs Lemmy-native error wrap, use Case B annotated closure: `.map_err(\|e\| -> Box<dyn std::error::Error + Send + Sync> { format!("{e}").into() })?`.
- `.claude/lessons/feedback_async_pool_test_pattern.md` — e2e fixtures pattern context. Edits must NOT touch surrounding `let pool = ...` / `let mut conn = pool.get_conn().await?` plumbing.
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **e2e.rs Edit discipline. MANDATORY anchor-based Edit, never full-file Read.** Use `Grep` for `PHASE_1_MIGRATION_COUNT` and `phase1_migrations_round_trip` first to capture line numbers; targeted `Edit` calls with surgical `old_string` context.
- `.claude/lessons/feedback_postgres_jsonb_canonicalization.md` — N/A (no JSONB edits in this bundle), but RE-READ anyway since e2e.rs touches probe assertions.
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — N/A (Task 8/9/10 add zero handler writes).
- `.claude/lessons/feedback_clippy_test_style.md` — Task 8's parity test extension. LemmyResult / `?` style mandatory; no `unwrap()`, no `allow_attributes`.

**Plan and recipe reading:**

- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §10.8 (Task 8 verbatim — 6 sub-edits in config.rs)
- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §10.9 (Task 9 verbatim — 3 sub-edits across 3 files)
- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §10.10 (Task 10 verbatim — single Edit-with-anchor on e2e.rs)
- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §13 Tasks 8, 9, 10 (FILES YAML, GOTCHAs, MIRROR primaries, pre-commit gates)
- `.claude/rules/governance-log-entry-kind-registry.md` (registry shape — read current state to confirm the v1-RT-r1 stub line and Acceptance invariants count `38`)

**Mirror files (for shape verification):**

- `crates/api/api/src/governance/config.rs:1391-1422` (existing EXPECTED_SEED_COUNT block — mirror for sub-edit (d))
- `crates/api/api/src/governance/config.rs:2683-2710` (existing parity test — mirror for sub-edit (e))
- `crates/db_schema/src/source/governance/governance_log.rs` (existing v1-SL-a additions block — mirror for sub-edit 2(a))
- `crates/api/api/src/governance/governance_log.rs` (existing `pub use` alphabetical block — mirror for sub-edit 2(b))
- `crates/server/tests/e2e.rs:1094` (PHASE_1_MIGRATION_COUNT current value `14` — mirror for Anchor 1)
- `crates/server/tests/e2e.rs:1054+` (phase1_migrations_round_trip fn body — mirror for Anchor 2)

**Rules:**

- `.claude/rules/decision-queue.md` Recipe 1 — DQ entry shape for `kind: "validate-pending"`.
- `.claude/rules/advisor-orchestrator.md` §5.3 §G4 classifier — Shape-G two-phase validation flow.
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — `--workspace --features full`, never `-p <crate>` (per plan §10.8 GOTCHA).
- `.claude/lessons/feedback_dq_raise_before_ci_watcher_queue.md` — atomic raise discipline.

## §3a Handover from prior cohort

**fix-impl-2 (Junior #TBD) committed `<sha-TBD>` ALREADY on phase-v1-RT-r1.** That commit padded 8 `ReputationEventInsertForm` literals in `create_endorsement.rs`, `seed_founders/main.rs`, `e2e.rs`. After fix-impl-2's `cargo-validate-workspace.yml` workflow returned `success` and ci-watcher mutated the DQ to `result: "pass"`, the daemon finalize-merged into `phase-v1-RT-r1`. Phase tip moved `37a62f9b4 → <post-fix-impl-2 tip>`.

**Phase state at this bundle dispatch:**
- Cohort A (Tasks 1-4): SHIPPED. Migration files on phase.
- Cohort B-serial (Task 5): SHIPPED. `ReputationEventSourceType` enum at commit `5142dba54`.
- Cohort B-bundled (Tasks 6+7): SHIPPED. schema.rs + Diesel structs at commits `03f6c670e` + `05cf5ae1d`.
- fix-impl-1 (2 callsites in `crates/api/api/src/governance/`): SHIPPED at commit `d52f62124`.
- fix-impl-2 (8 callsites in api_crud + seed_founders + e2e): SHIPPED at commit `<sha-TBD>`.
- Cohort B-final (Tasks 8+9+10): THIS bundle.
- Task 11 (canonical retro): pending after this bundle ships.

**Worker branch forks from `<post-fix-impl-2 tip>`** — confirm at task-0 via `git rev-parse phase-v1-RT-r1`.

## §4 Constraints

- **Bundled atomicity:** all 5 files modified in ONE commit. Junior MUST NOT split into multiple commits — the workspace-check validates the bundle as a unit.
  - Files: `crates/api/api/src/governance/config.rs`, `crates/db_schema/src/source/governance/governance_log.rs`, `crates/api/api/src/governance/governance_log.rs`, `.claude/rules/governance-log-entry-kind-registry.md`, `crates/server/tests/e2e.rs`.
- **Pre-commit reconciliation gates:** run BOTH the Task 8 reconciliation gate (`§2.3` shell block 1) AND the Task 9 count invariants (`§2.3` shell block 2) BEFORE `git commit`. If any disagree, file a DQ blocker and STOP.
- **`pub use` ordering is STRICT alphabetical** — `pub use` block in `crates/api/api/src/governance/governance_log.rs` must remain alphabetised. Use `Read` + careful insertion at the exact positions listed in §2.2(b).
- **e2e.rs edit discipline:** MANDATORY anchor-based Edit. `Grep` for `PHASE_1_MIGRATION_COUNT` and `phase1_migrations_round_trip` first; do NOT `Read` the full 12,000-line e2e.rs. Per `feedback_junior_worker_e2e_edit_hang.md`.
- **Parametric pattern (R11):** Task 8 must use parametric counts; DO NOT bump locked v0 / V1_AD / V1_JM / V1_SL counts. The parity test sum is `34 + 27 + 27 + 13 + 26 = 127`.
- **Registry pre-landed-const exemption:** each row in §2.2(c) MUST have `(pending)` marker + downstream-sub-phase citation per plan §10.9 lines 1046-1052.
- **No handler edits:** Tasks 8/9/10 add zero call-site emitters. v1-RT-r1 is substrate-only; r2/r3/r4/r5 land the emitters. If any handler under `crates/api/api/src/governance/<handler>.rs` would change, file a DQ blocker.
- **No `-p <crate>` validation:** workspace check is `cargo check --workspace --features full` per `feedback_features_full_p_crate_incompatible.md`. Never `-p lemmy_api --features full`.
- **Branch:** standard Junior daemon flow off `phase-v1-RT-r1` post-fix-impl-2 tip. Confirm at task-0 the merge-base IS the post-fix-impl-2 tip.
- **Shape G:** after committing, push the worker branch; capture `cargo-validate-workspace` workflow_run_id; write `kind: "validate-pending"` DQ entry per Recipe 1.
- **DQ atomic raise:** sequence is **(1) write DQ entry in worker branch checkout, (2) git add .claude/decision-queue.json && git commit && git push origin <worker-branch>, (3) end task** — advisor dispatches the ci-watcher AFTER seeing the DQ on origin, per L3.
- **Encoding:** Python `json.dump(..., ensure_ascii=False, indent=2)` for the DQ entry.
- **next_id calculation:** span both `.claude/decision-queue.json` AND `.claude/decision-queue-archive-*.json`. Per decision-queue.md Hard refusal #2.
- **COMMIT MESSAGE:** `feat(v1-RT-r1): cohort B-final bundle — config.rs 26 consts + governance-log 7 ENTRY_KIND + registry + e2e PHASE_1_MIGRATION_COUNT (tasks 8+9+10)`
- **HANDOVER YAML trailer:** add `filesCreated: []`, `filesModified: [<5-file list>]`, `keyDecisions: [bundled per retro L1 decision; one workspace-check workflow; resolves Cohort B-final cohort barrier]`, `notes: <verbatim handoff for Task 11 retro>`.

## §4.1 CANONICAL CASE OVERRIDE

Task 10's e2e.rs edit may surface a Case-A/B/C decision per `feedback_lemmy_error_no_std_error.md` IF a new probe needs Lemmy-native error wrap. The existing test fn returns `Result<(), Box<dyn Error>>` (Case B outer). New probes that stay within `Box<dyn Error>` via existing Diesel `?` propagation need NO change. New probes that call Lemmy-native fns returning `LemmyResult<T>` need the **annotated closure** per Case B:

```rust
.map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("{e}").into() })?
```

Plan §10.10 indicates new probes are pure Diesel SQL queries (`pg_indexes` row counts, `pg_type` row absence, `governance_config` row count delta) — all within Diesel's error type, no Lemmy bridge needed. **Verify** by reading the existing probe pattern at e2e.rs:1054+ before writing any new probe. If you find a need for a Lemmy bridge, use Case B annotated closure — do NOT flip the test fn outer signature.

## §5 Out of scope

- Task 11 (canonical phase retro) — separate barrier task AFTER this bundle's ci-watcher passes. Author at `.claude/PRPs/reports/v1-RT-r1-retro.md`.
- Any handler-side emitter wiring for the 7 new ENTRY_KIND consts — pre-landed-const exemption; r2/r3/r4/r5 land emitters.
- v1-AD-a's 3 duplicate config keys — out per planner DQ #187 (`deltas.participation_weekly_active`, `participation.dormancy_window_days`, `deltas.participation_dormant`). Their DEFAULT_* + SEEDED_KEYS + CONFIG_KEY_METADATA stay in v1-AD-a's blocks.
- Migrations — no `migrations/**` edits in this bundle (the 4 RT-r1 migrations shipped in Cohort A).
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.coderabbit.yaml` — out per plan §11 "Files explicitly NOT touched".
- Adding builder helpers for `ReputationEventInsertForm` — out per plan §11.
- Editing `reputation_snapshot.rs` (v0 calculator) — out; r2 replaces it.
- New e2e test fns — out per plan §10.10 "No new test fns".
- Splitting Task 9's three files across separate commits — bundled atomicity is mandatory.
