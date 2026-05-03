# SL-a planning brief

**Written**: 2026-05-03 by advisor session (laptop, brehon-fork CWD) for Junior dispatch on EliteDesk.
**Subagent target**: `planning` (Opus 4.7, color purple — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/sl-a-planning-1` from `governance-v0` per concurrency-1 default; the planning subagent commits the plan file and pushes back to `governance-v0` at finalize.
**Authority anchor**: `v1-sponsor-liability.prd.md` §15 row 1 (`v1-SL-a — Schema + enum + migration`). This is the FIRST sub-phase of the SL lane to plan from this CWD; no SL-* sub-phase has shipped to `governance-v0` yet (verified at brief-write time, governance-v0 HEAD `48c4f379d`).

---

## 1. Role + dispatch line

`[role:planning] v1-sponsor-liability-a plan — schema + enum + migration + ENTRY_KIND consts + seeds + Issue #24 index`

The actual `mcp__junior-brehon__create_task` description is a single line under 100 chars per `.claude/rules/advisor-orchestrator.md` Junior task description template:

```
[role:planning] v1-sponsor-liability-a plan — see .claude/PRPs/briefs/sl-a-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` for sub-phase **v1-SL-a**, the schema-only foundation for the SL lane. The plan covers PRD §15 row 1 in full: enum extensions, two new `moderation_case` columns, two indexes (one for the scheduler partial-index, one for Issue #24), 13 new `governance_config` seed rows, 5 new `ENTRY_KIND_*` consts (declared but call-sites-pending — per registry-rule pre-landed-const exemption), and a backfill query for v0 mid-flight cases.

### 2.1 Six concrete deliverables (per PRD §3, §8, §10, §17 + registry rule)

The plan's §13 task list MUST cover all six:

a. **Three new `CaseStatus` enum variants** — `SponsorLiabilityPending`, `SponsorLiabilityFired`, `SponsorLiabilityEscaped`. Per PRD §3.1 + §3.4. The enum is at `crates/db_schema_file/src/enums.rs:393-408` (verified at brief-write time — currently 9 variants). Three additions => 12 variants post-SL-a. Migration uses `ALTER TYPE case_status ADD VALUE IF NOT EXISTS ...` with the `-- no-transaction` Diesel directive (per Phase 5b task 56's `Restoration` variant precedent).

b. **Two new `moderation_case` columns + one partial index** — `grace_expires_at TIMESTAMPTZ` (nullable; set at Decided→Pending transition, locked thereafter) and `liability_escape_reason JSONB` (nullable; structured per PRD §8.1 schema with `version: 1` per OQ-V1-SL-05 forward-compat resolution). Partial index `moderation_case_grace_expires_idx ON moderation_case (grace_expires_at) WHERE status = 'SponsorLiabilityPending'` is the scheduler's primary access path per PRD §6.

c. **Issue #24 partial index on `surety`** — `CREATE INDEX surety_sponsored_id_active ON surety (sponsored_id, sponsor_id) WHERE revoked_at IS NULL`. Per PRD §8.2; folded into this same migration file to avoid a second migration churn (per the PR #10 CR finding GH #24 itself raised). Speeds up `jury_common::select_eligible_jurors` EXISTS subquery + `apply_sponsor_liability` sponsor enumeration.

d. **5 new `ENTRY_KIND_*` consts** — `ENTRY_KIND_SPONSOR_LIABILITY_PENDING`, `_FIRED`, `_ESCAPED`, `ENTRY_KIND_ENDORSEMENT_REVOKED`, `ENTRY_KIND_RESTORATION_COMPLETED`. Per PRD §17 cross-cutting impact + `.claude/rules/governance-log-entry-kind-registry.md` reserved section "sponsor-liability-v1". Dual-file edit (per v1-AD-a §10.8 pattern + v1-JM-a §15 pattern):
   - DEFINE in `crates/db_schema/src/source/governance/governance_log.rs` (alphabetical-within-block per registry rule).
   - RE-EXPORT via `pub use` in `crates/api/api/src/governance/governance_log.rs` (alphabetical within the `pub use` block).
   Registry file `.claude/rules/governance-log-entry-kind-registry.md` MUST be updated: replace the reserved-section placeholder with a populated table of the 5 consts + their downstream emitting handlers (per the registry rule's pre-landed-const exemption — emitting handlers are SL-b/SL-c/SL-d, named with `(pending)` markers and linked to specific downstream plans). Registry count goes 33 → 38; the registry-rule's acceptance-invariant `rg '^pub const ENTRY_KIND_'` count must update accordingly.

e. **13 new `governance_config` seed rows** — 10 `liability.*` keys (grace-window minor/moderate/severe/minimum/maximum/alert-threshold + restoration-escape-bool + restoration-severity-reduction + multi-sponsor-escape-rule + revoke-rate-limit-per-day) + 3 `job.grace_check_*` keys (interval-minutes + batch-size + staleness-alert-multiplier). Per PRD §10 Defaults Matrix. **PRD §10's footer says 13 keys, not 12 — verified at brief-write time; the homeserver advisor-context-v1-SL-d.md cited "12" which is drift.** All seeded via `INSERT INTO governance_config ... ON CONFLICT DO NOTHING` (Phase 5a idempotent-seed pattern). Each key gets a `SEEDED_KEYS_WITH_CONSTS` tuple entry at `crates/api/api/src/governance/config.rs:1072` AND a `ConfigKeyMetadata` entry (per v1-AD-a §3.2 compile-time pattern, NOT4 resolution). Parametric seed count: SL-a adds `pub const EXPECTED_SEED_COUNT_V1_SL: usize = 13;` beside the existing `EXPECTED_SEED_COUNT_V1_AD = 27` and `EXPECTED_SEED_COUNT_V1_JM = 27` at `config.rs:1304-1322`; the parity test at `config.rs:2423` adds `+ EXPECTED_SEED_COUNT_V1_SL` and updates the error message at line 2428.

   **Per DQ #115 resolution (advisor 2026-05-03 via /brehon-clarify):** the 6 `liability.grace_window_*_hours` keys (24/72/168/1/720/24) are stored as **raw integer hours**, NOT micros-scaled. Per `feedback_brehon_config_micros_scaled.md` the lesson's micros pattern scopes to reputation/score formulas (per-call weight × `reputation_snapshot` row × `× 1,000,000`, compared with strict `>`); grace-window hours are wall-clock units consumed by Postgres `INTERVAL '<N> hours'` in PRD §8.4's backfill SQL and the scheduler's `now() >= grace_expires_at` comparison. Same applies to `liability.revoke_rate_limit_per_day` (count, not score) and `job.grace_check_*` (minutes / count / float multiplier). Bool keys (`liability.restoration_escapes_liability`) and enum keys (`liability.multi_sponsor_escape_rule`) are likewise non-micros. **No `liability.*` or `job.*` key SL-a seeds is micros-scaled.**

f. **Migration backfill** — UPDATE for v0 mid-flight cases per PRD §8.4 + ADR-010 won't-disadvantage rule. Three nested `EXISTS` guards (surety presence, sanction presence, no-prior-fire). 24-hour minor-default grace (most lenient). The plan §15 must run the migration round-trip script (`scripts/brehon/migrate-roundtrip.sh`).

   **Per DQ #114 resolution (user 2026-05-03 via /brehon-clarify):** the `migrate-roundtrip.sh` stub fix is folded into SL-a's plan §13 Task 0. The same impl-task that ships the SL-a schema migration also replaces the stub body with real round-trip logic (testcontainers Postgres + `cargo run -p lemmy_diesel_utils --features full ... migration run` / `migration revert` / re-run idempotency). This closes DQ #68 atomically with SL-a's ship. Validation chain works end-to-end on the very first push from the SL-a phase branch.

### 2.2 Scope boundary — what comes AFTER SL-a (NOT this sub-phase)

Per PRD §15 phase table:

- **SL-b (next):** `revoke_endorsement` handler + DTO + route + integration tests. The handler reads the columns + enum variants SL-a creates. SL-a does NOT author the handler.
- **SL-c (after SL-b):** Scheduler module `sponsor_liability_grace.rs` + clokwerk wiring. Reads `moderation_case.grace_expires_at` + `status = 'SponsorLiabilityPending'` rows. SL-a does NOT author the scheduler.
- **SL-d (after SL-c):** `submit_jury_vote` mutation + `apply_sponsor_liability` compute/fire split + `Decided → SponsorLiabilityPending` transition. SL-a does NOT author the split (the v0 `apply_sponsor_liability` stays intact through SL-a; only the new state space is created).
- **SL-e (after SL-d):** e2e suite for the lane (revocation-during-window-escapes; restoration-during-window-escapes; window-expiry-fires; backfill-of-mid-flight). SL-a's own validation is migration-round-trip + workspace-check + clippy + ADR-013 grep-sweep, not e2e.

**Hard out-of-scope for SL-a** under either interpretation (per PRD §2 OUT + §15):

- `revoke_endorsement` handler (SL-b's). DTO struct may land via PRD §5.1 if a downstream sub-phase needs the type-shape early — but the handler itself is SL-b's.
- Cross-instance sponsor-liability federation (deferred to v2 per ADR-014).
- Step-up auth enforcement (PRD §12.3 — v1 reserves slot only).
- Restoration completion endpoint (PRD §7.4 — owned by restorative-mechanics-v1 PRD).
- Notification UX (PRD §13 OQ-V1-SL-03).
- Dashboard write surface for grace-window keys (PRD §14 — owned by admin-dashboard-v1 PRD).
- The dynamic compute/fire split (SL-d's). v0's `apply_sponsor_liability` keeps its v0 shape through SL-a.

### 2.3 Plan file deliverable shape

- **One file:** `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md`.
- **Follow `.claude/PRPs/templates/plan.template.md` 20-section schema literally.** §16a Stories is mandatory (SL-a is post-spec-kit-adoption; not optional).
- **Shape G applies — SL-a is the second non-JM plan under Shape G** (after JM-e + ahead of any future SL-b/c/d). §15 DoD MUST use the per-workflow shape (workflow path + phase-branch SHA + `conclusion: "success"`). Inline cargo invocations are forbidden in §15.
- **§5 complexity score with breakdown table** per `feedback_complexity_score_pre_split.md`. Pre-estimate: **5-7** (5-7 §13 tasks × 0 above-5 = 0 / 1 migration × 2 = 2 / 4 crates touched (`db_schema_file`, `db_schema`, `api`, `lemmy_diesel_utils`) × 1 = 4 / 0 e2e edits / 0 ADR-affecting / Shape G cargo budget = 0 → ~6). NOT split-DQ-tripping; planner unlikely to file split-or-proceed.
- **§16a Stories** — every story names (a) composing §13 tasks, (b) a checkpoint command in the Shape-G workflow shape, (c) Brief-Scope outputs to verify (file:line + symbol) per `feedback_advisor_watchpoint_specificity.md`. Concept-only watchpoints rejected.
- **§4 watchpoints** — every entry cites a specific file, table, or `schema.rs` line. No abstract concepts.
- **Explicit scope-boundary section in §6** ("Relationship to other v1-SL sub-phases") listing what SL-b/c/d/e will ship and confirming SL-a does not duplicate their work.

**Commit only the plan file.** Do not author Rust code, do not open PRs, do not touch any file under `crates/`, `migrations/`, or `tests/`. Junior finalize pushes the plan-file commit to `junior/sl-a-planning-1`; advisor merges it into `governance-v0` after DoD smoke + watchpoint-specificity gates pass and the user approves.

---

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract; defines model, tools, "Before you start (always)" sequence, plan-content discipline cross-references.
2. `.claude/PRPs/templates/plan.template.md` — canonical Brehon plan template (20 sections incl. §15.6 Shape-G DoD and §16a Stories). Structure is load-bearing.
3. `.claude/PRPs/prds/v1-sponsor-liability.prd.md` — the parent PRD. **Read in full the first time.** Specifically:
   - §1 (vision/goals — Brehon `athgabál` framing; gives motivation but not implementation)
   - §2 (scope IN/OUT — confirm SL-a is schema/enum/seeds/index/backfill only)
   - §3 (CaseStatus extensions — three new variants; **§3.3 ADR-013 compliance is the load-bearing match-site grep enumeration; SL-a's plan MUST grep against current `governance-v0` and produce the actual list of sites**)
   - §4 (grace-window durations — informs the seed-row defaults)
   - §6 (grace-window scheduler — SL-c's, but the partial-index in SL-a is its primary access path)
   - §7 (restoration interaction — informs `liability_escape_reason` JSONB schema)
   - §8 (DB & migration changes — **SL-a's primary deliverable section**; read §8.1, §8.2, §8.3, §8.4 backfill, §8.5 migration ordering)
   - §10 (defaults matrix — **13 keys not 12**; verify at planning time, the count is the only authoritative source)
   - §11 (backwards compat incl. documented behavioural changes — informs §16a Story 1 acceptance criteria)
   - §15 (implementation phases — confirms SL-a is row 1; SL-d depends on Phases 1 + 3)
   - §17 (cross-cutting impact — names the 5 ENTRY_KIND consts; confirms ADR-013 enum-exhaustiveness invariant; ADR-015 pseudonymisation)
   - §18 (resolutions applied 2026-04-19 — B4 key-rename table is authoritative for the 10 `liability.*` keys)
4. `.claude/rules/governance-log-entry-kind-registry.md` — read the entire file, especially:
   - The "sponsor-liability-v1 (reserved)" section that SL-a fills in with the 5 consts.
   - The "Acceptance invariants" section (the count + collision invariants that SL-a must update).
   - The "pre-landed-const exemption" rule (SL-a may pre-land consts whose call sites land in SL-b/SL-c/SL-d, provided each pending row names its downstream plan).
5. `crates/api/api/src/governance/config.rs` — the `SEEDED_KEYS_WITH_CONSTS` array at line 1072, the `EXPECTED_SEED_COUNT_V1_*` constants at lines 1304-1322 (parametric extension precedent), the `ConfigKeyMetadata` array (per v1-AD-a §3.2 + NOT4 resolution), and the parity test at line 2423-2433 (SL-a amends to add `EXPECTED_SEED_COUNT_V1_SL`).
6. `crates/db_schema_file/src/enums.rs:393-408` — the `CaseStatus` enum SL-a extends. Read the surrounding enum-block discipline (DbEnum derive, ExistingTypePath path, DbValueStyle).
7. `crates/db_schema/src/source/governance/governance_log.rs` — read the entire ENTRY_KIND const block (line 112+) to understand the alphabetical-within-section convention. Locate where SL-a's 5 new consts insert (after the existing v1-JM-a/JM-c block, before any future v1 block).
8. `crates/api/api/src/governance/governance_log.rs` — the api shim. Verify the alphabetical `pub use` block where SL-a's 5 re-exports land.
9. `migrations/` — list the most recent migration directories to determine SL-a's timestamp prefix (must sort after JM-d's appeal-window-expires-at migration if present, and after any AD-c migration if present). Use `ls -la migrations/ | tail -5` to get the actual prefix range. Per `feedback_pin_strategy_check_derives.md` and the Phase 1 retro guidance, prefer day-level granularity to avoid same-day cosmetic collision.
10. `crates/lemmy_diesel_utils/` — read the migration-runner pattern (per `feedback_lemmy_migration_runner.md`). SL-a uses `cargo run -p lemmy_diesel_utils --features full` for round-trip validation, NOT raw `diesel migration run`.
11. `.claude/PRPs/v1-planning-queue.json` — the cross-PRD coherence audit log. Specifically:
    - **B4 resolution** (key-rename table; flat `liability.*`; lists 10 SL-owned keys SL-a seeds).
    - **NOT2 resolution** (governance-log entry-kind registry → tracked via GH #41, lives at `.claude/rules/governance-log-entry-kind-registry.md`).
    - **NOT4 resolution** (ConfigKeyMetadata as `&'static [ConfigKeyMetadata]` compile-time array; matches v1-AD-a's pattern; SL-a follows).
12. `crates/db_schema/src/source/moderation_case.rs` (or wherever `moderation_case` Diesel struct lives — confirm path at planning time) — verify the `ModerationCase` struct + `Insertable`/`Queryable` derives. SL-a's `grace_expires_at` and `liability_escape_reason` columns must propagate through the struct + the `ModerationCaseInsertForm` so InsertForm drift (per `feedback_insertform_default_propagation.md`) doesn't strand the columns at write time.
13. `crates/db_schema_file/src/schema.rs` — confirm the `moderation_case` table block (line range varies; planner reads at planning time). After SL-a runs `cargo run -p lemmy_diesel_utils ... migration run`, the schema regeneration adds the two new columns to this file. The plan §13 must include a regenerate-schema task (per Phase 1 task 10 precedent).
14. **Glob `.claude/lessons/feedback_*.md` and Read every file whose name keyword matches:** `migration`, `migration_runner`, `no-transaction`, `enum`, `cargo`, `features-full`, `features_full_p_crate`, `wrapper-script`, `clippy`, `test-style`, `complexity_score`, `pre_queue`, `watchpoint`, `dod`, `validate`, `shape_g`, `validate_pending`, `governance_log`, `entry_kind`, `seed`, `seeded_keys`, `parametric`, `dual_file`, `re_export`, `partial_index`, `surety`, `sponsor`, `liability`, `pre_phase_dod`, `dry_run`, `wrapper_silence`, `principles_not_rules`, `read_canonical`, `parallel_cohort`, `cohort_yaml`, `schema_changing_spec_retrofit`, `four_role`, `retro_not_report`, `insertform`, `propagation`, `micros`, `micros_scaled`. That is the lessons-corpus discipline per `planning.md` step 3.

    **Per DQ #115 (clarify):** `feedback_brehon_config_micros_scaled.md` is included in the keyword sweep above. Its scope is reputation/score-formula math, NOT wall-clock units. SL-a's seed values use raw integer hours/minutes/days/counts; this is correct per PRD §8.4 + §6.1 + §10. The lesson is cited so the planner knows to AVOID applying micros-scaling to the SL-a keys, not to apply it.
15. **Glob `.claude/lessons/reference_*.md` and Read every file whose name keyword matches:** `nutomic`, `governance_log_entry_kind_registry`, `phase_branch`, `branch_manager`, `worktree`.
16. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — read **ADR-010** (no retroactive invalidation; informs the §8.4 backfill 24-hour minor-default), **ADR-013** (CaseStatus enum-exhaustiveness — every match site updates explicitly; **ADR-013 is the load-bearing invariant for SL-a**), **ADR-015** (pseudonymisation; informs `liability_escape_reason` JSONB schema includes `actor_pseudonym` not raw `person_id`). Also OQ-025 (sponsor-liability v1 — resolution into this PRD), OQ-003 (Restoration variant interaction with grace window), OQ-V1-SL-05 (`liability_escape_reason` schema versioning — version: 1 from day one).
17. `.claude/PRPs/plans/phase-v1-JM-a.plan.md` — the closest-shape precedent. v1-JM-a was the first JM lane sub-phase (analogous to SL-a's role in the SL lane): schema + enums + snapshot columns + 6 ENTRY_KIND consts + parametric `EXPECTED_SEED_COUNT_V1_JM`. Read §13 task ordering, §10 patterns, §15 DoD shape (pre-Shape-G — for contrast against SL-a's Shape-G shape), §17 completion checklist.
18. `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — the most recent post-spec-kit-adoption + Shape-G plan. Read §15 per-workflow DoD shape, §16a Stories shape, §5 complexity score breakdown table, the `[P]` cohort markers + FILES YAML blocks in §13. SL-a's plan adopts these exact shapes.
19. `.claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md` — the v0 sponsor-liability foundation. Provides the existing `apply_sponsor_liability` shape (line refs in `crates/api/api/src/governance/sponsor_liability.rs`) that SL-d will eventually split. SL-a does NOT touch this file; the read is for context (so the SL-a plan understands how `_APPLIED` and `_CLAMPED` consts already exist in `governance_log.rs` at lines 122-123 and the 5 new consts insert *after* them alphabetically).
20. `.claude/PRPs/briefs/jm-e-planning-1.md` and `.claude/PRPs/briefs/jm-d-planning-1.md` — exemplar planning briefs for shape and constraint language.
21. `.github/workflows/cargo-validate-workspace.yml` + `.github/workflows/cargo-validate-features-full.yml` + `.github/workflows/cargo-test-e2e.yml` — the existing Shape-G workflows the plan §15 references. Verify they carry `--no-deps -- -D warnings` (R6 inheritance) and `--features full` where applicable. SL-a does NOT add new workflows or edit existing ones; if any flag is missing the planner files a DQ.
22. **DQ #67 resolved (per user 2026-04-27)** — workflow YAML DoD dry-run discipline. Honoured by SL-a §15: do NOT prescribe `act` invocations; do NOT require pre-merge full dry-run.

---

## 4. Constraints (hard rules — violating any of these is a process breach)

### 4.1 Plan-content discipline

- **Shape G DoD shape (forward-only).** §15 MUST use the per-workflow shape per `.claude/PRPs/templates/plan.template.md` §15.6. Inline cargo invocations are forbidden in §15. Per `feedback_schema_changing_spec_retrofit_question.md`. The migration round-trip validation is the one exception — `bash scripts/brehon/migrate-roundtrip.sh <id>__<name>` may live in §14 Testing strategy as a pre-merge wrapper, but it is NOT an inline-cargo §15 entry.
- **§16a Stories mandatory** (NOT optional). Every story names composing §13 tasks, a Shape-G checkpoint command, Brief-Scope outputs to verify (file:line + symbol), and lists the §13 IMPLEMENT entries it covers. Per `.claude/PRPs/templates/plan.template.md` §16a + `.claude/commands/brehon-verify.md`. Likely 2 stories: (1) "schema migration round-trips cleanly + new columns + new enum variants exist + ADR-013 match sites compile" (composing tasks 1-3); (2) "13 new config keys are seeded and parity test passes + 5 new ENTRY_KIND consts exist with downstream pending markers + registry rule updates" (composing tasks 4-5).
- **§4 watchpoints cite specific files / tables / `schema.rs` lines**, never abstract concepts. Per `feedback_advisor_watchpoint_specificity.md`. Concrete watchpoints SL-a's plan §4 MUST include:
   1. **ADR-013 enum-exhaustiveness sweep.** Every existing `match case.status { ... }` site in `crates/api/`, `crates/api_crud/`, `crates/db_views/`, `crates/lemmy_server/tests/`, etc. must be updated to handle the three new variants. Plan §4 watchpoint MUST grep at planning time and list the exact file:line:variant_arm sites. PRD §3.3 is the seed list (`request_appeal.rs:90-102`, `admin_close_case.rs:65-75`, etc.) but the planner re-greps because the JM lane added more match sites since 2026-04-19.
   2. **Parity-test invariant.** `config.rs:2423` (the `assert_eq!(SEEDED_KEYS_WITH_CONSTS.len(), expected)` check) — adding `EXPECTED_SEED_COUNT_V1_SL` and updating both the calculation AND the error message at line 2428. Drift between the constant and the actual seeded-count is the test that fails-loud.
   3. **Registry rule count invariant.** `governance-log-entry-kind-registry.md` Acceptance Invariants section: after SL-a, `rg '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs | wc -l` returns 38 (33 + 5). The shim re-export count at `crates/api/api/src/governance/governance_log.rs` MUST equal 38.
   4. **Pre-landed const exemption.** Each of the 5 new consts has its row in the registry-rule table populated with a `(pending)` marker on the "Emitting handler" column AND a citation of which downstream plan ships the call site (SL-b for `endorsement_revoked` per PRD §5.3; SL-c/SL-d for the three `sponsor_liability_*`; restoration-mechanics-v1 PRD or SL-c for `restoration_completed`). A pre-landed const without a downstream-plan link triggers the registry-rule "registry-pollution bug" assertion.
   5. **InsertForm drift on `moderation_case`.** Per `feedback_insertform_default_propagation.md`. The two new columns (`grace_expires_at`, `liability_escape_reason`) must propagate to `ModerationCaseInsertForm` with `Default::default()` for nullable columns. v0 callers (e.g. `create_report.rs`) construct the InsertForm without these fields; the `Default` derive picks `None`. Plan §4 watchpoint cites the InsertForm location and asserts the planner verifies at planning time.
   6. **Migration timestamp ordering.** The migration directory prefix must sort AFTER any prior v1-JM-* migrations to avoid the cosmetic collision Phase 1 retro flagged. Per `project_brehon_phase_1_complete.md` and v1-AD-a retro carry-forward. Plan §13 Task 0 (audit) lists the `ls migrations/ | tail -5` output as evidence.
- **§5 complexity score breakdown table mandatory** per `feedback_complexity_score_pre_split.md`. Pre-estimate 5-7. NOT split-tripping.
- **e2e edit discipline** per `feedback_junior_worker_e2e_edit_hang.md`. SL-a does NOT author e2e tests (they belong to SL-e and individual handler sub-phases). If the planner is tempted to add an e2e test "to validate the schema works", refuse — this is a schema-only sub-phase, validation is migration round-trip + workspace-check + the parity tests.
- **Cargo feature/flag propagation** per `pattern_cargo_feature_flag_propagation.md`. Never combine `-p <crate>` with `--features full` in any DoD command (per `feedback_features_full_p_crate_incompatible.md`); use `--workspace --features full`. Even though §15 is Shape-G workflow-shape, the planner-side DoD smoke test (advisor side, pre-merge) and any §15.7 manual validation snippets respect this.
- **Build only what tests exercise** per PMD #14 / `feedback_build_what_tests_exercise.md`. SL-a is intentionally the smallest footprint of the lane — drift-stub everything beyond schema/enums/seeds/consts. The 5 new ENTRY_KIND consts are pre-landed without call sites (per registry rule exemption); this IS the build-only-what-tests-exercise principle in action.
- **R-rule inheritance from JM-a/b/c/d/e and AD-a retros** — every R1-R7 from prior retros applies to SL-a. R6 in particular (clippy `--no-deps -- -D warnings`) — under Shape G this lives in the workflow YAML. R5 (Task 0 enumerates ALL probes explicitly) is load-bearing for SL-a's pre-flight harness audit.
- **Wrapper-script flag silence** per `feedback_wrapper_script_flag_silence.md`. Under Shape G non-binding for §15 DoD; if §15.7 manual-validation snippets are included, they MUST cite Linux `.sh` wrappers and verify wrapper `$@` passthrough.
- **Multi-write handlers transactionality** per `feedback_multi_write_handlers_need_transactions.md` — non-applicable for SL-a (no handlers; only schema). Cited for completeness; SL-b/c/d will need it.
- **Migration round-trip discipline** per `feedback_lemmy_migration_runner.md` (forbid_diesel_cli trigger landed upstream). Use `cargo run -p lemmy_diesel_utils --features full ... migration run` / `migration revert`. Plan §13 must include a Phase-1-task-13-style round-trip test (or extend `phase1_migrations_round_trip` to cover SL-a's migration).

### 4.2 Decision-queue discipline (per `.claude/rules/decision-queue.md`)

- **Attribution integrity.** Any DQ entry seeded by the planning subagent uses `answered_by: "planner"` (forward-looking pre-resolved entries) or `answered_by: null` (genuinely needs advisor input). NEVER `"advisor"`, `"user"`, `"impl-self-resolved"`. Subjects on planner DQ commits MUST start with `chore(decision-queue): planner raised DQ #<id> — <slug>`.
- **Mid-task DQ commits push immediately**, not at finalize, per `decision-queue.md` §"Mid-task visibility (Junior worktrees)". Push to `junior/sl-a-planning-1` (this worktree's branch); advisor's polling loop fetches all branches.
- **Boundary-of-judgment — when to STOP and queue rather than guess:**
  - If the migration timestamp prefix the planner picks would cosmetically collide with an existing migration directory (e.g. another v1 sub-phase plan in flight) → planner DQ asking which prefix to use.
  - If the `liability_escape_reason` JSONB schema needs additional fields beyond what PRD §8.1 specifies (e.g. a `version: 1` plus extra metadata for SL-d's restoration-completion case) → planner DQ proposing the extended schema.
  - If the parametric `EXPECTED_SEED_COUNT_V1_SL` value differs from PRD §10's "13 keys" footer (e.g. the planner counts only 12 in §10 but PRD §4.2 + §6.4 + §13 sum to 13) → planner DQ surfacing the discrepancy with the actual count from the PRD body, NOT silently picking one. (Advisor pre-checked at brief-write time: PRD §10's enumerated table + footer say 13; the homeserver advisor-context cited 12, that's drift.)
  - **DQ #114 resolution (user 2026-05-03):** `migrate-roundtrip.sh` stub fix is part of SL-a's Task 0. The planner does NOT file a fresh DQ on this; the scope is settled. If the planner discovers the stub has already been fixed in a commit between brief-write and planning time, fold the existence-check into Task 0's pre-flight harness audit instead of authoring a new fix-task.
  - If at planning time the registry rule's `sponsor-liability-v1 (reserved)` block has been populated by some unexpected commit (concurrent SL planning?) → planner DQ asking the advisor to reconcile before SL-a edits.
  - **DO NOT file a scope-hypothesis DQ.** SL-a's scope is unambiguous from PRD §15 row 1; this brief is explicit. If the planner finds itself wanting to bundle SL-b or SL-c into SL-a, refuse the temptation and file a DQ asking the advisor — but the default is "stay narrow per PRD".

### 4.3 File ownership

- **Touch only:**
  - `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` (CREATE).
  - `.claude/decision-queue.json` (APPEND new pending or planner-resolved entries).
- **Do NOT edit anything under** `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, `.github/workflows/`, or any other plan file in `.claude/PRPs/plans/`. Even though SL-a's plan §13 will direct the impl-task subagent to edit `.claude/rules/governance-log-entry-kind-registry.md` (populating the reserved section), the planner does NOT make that edit — the impl-task does, after the plan is approved.
- **One commit at finalize:** `feat(plan): v1-SL-a sub-phase plan` (matching JM-a/JM-c/JM-d/JM-e + AD-a planner-task commit subjects).

### 4.4 Schema-first discipline

- **Read `crates/db_schema_file/src/schema.rs` BEFORE designing §13.** Confirm at planning time that:
  - `moderation_case` table block exists and the SL-a-target columns (`grace_expires_at`, `liability_escape_reason`) are absent (else SL-a's migration would no-op or conflict).
  - `surety` table block exists (otherwise Issue #24 partial index has no target).
  - `case_status` enum type is in the sql-types section and currently lists 9 variants (the SL-a migration adds 3).
- **Read `crates/db_schema_file/src/enums.rs:393-408`** to confirm the Rust enum has 9 variants currently. If any of these baseline assumptions fails, file a `kind: "blocker"` DQ before writing §13.

### 4.5 Cross-cutting from PMD-promoted patterns

- **PMD #126** (`automation candidate: gov-v0 → phase-branch merge-up with DQ renumbering`) — if SL-a's phase branch diverges from `governance-v0` post-cut (e.g. parallel admin-dashboard or rep-tuning work lands), DQ ID collisions are likely. Plan §18 Risks should include a row for this (Likelihood LOW, Impact LOW, Mitigation: gov-v0 IDs canonical, phase-side renumbered).
- **PMD #14** (`build only what tests exercise`) — drift-stub anything beyond schema/enums/seeds/consts. The 5 pre-landed ENTRY_KIND consts exemplify this.
- **Pattern: dual-file ENTRY_KIND edit** (per v1-AD-a §10.8 + v1-JM-a §15) — declare in `db_schema`, re-export in `api` shim, populate the registry-rule's reserved table block. SL-a follows the established pattern.

### 4.6 Attribution integrity reminder

The only valid `answered_by` labels for a Junior planning subagent are `"planner"` or `null`. Never `"advisor"`, `"user"`, `"impl-self-resolved"`. Any commit with `answered_by: "advisor"` from this subagent triggers the catch-fire procedure in `advisor-orchestrator.md`.

---

**Lean / advisor-side tip (not a constraint):** SL-a is the cleanest possible sub-phase to plan and ship — it's mechanical work (schema migration + enum extension + 13 seed rows + 5 pre-landed consts), the precedent from v1-AD-a and v1-JM-a is rehearsed, and the parametric `EXPECTED_SEED_COUNT_V1_*` invariant means the parity test catches any drift loudly. Complexity 5-7 means no split-DQ, no cohort dispatch (sequential tasks fit fine), no e2e contention. The single non-obvious risk is the ADR-013 enum-exhaustiveness sweep — three new `CaseStatus` variants force every existing `match case.status` site in the codebase to update explicitly, and the JM lane has added match sites since the PRD's §3.3 grep was authored 2026-04-19. The planner MUST grep at planning time and list every site as a §4 watchpoint. Missing a site = compile error at the boundary; finding all sites pre-impl is what makes SL-a's complexity stay low.

A second observation: the registry rule's pre-landed-const exemption (each pending const linked to a specific downstream plan) means SL-a's plan §13 must include the registry-rule edit AND the dual-file consts AND the re-export, but does NOT need to wire any emitting call site. The five pending downstream-plan links: SL-b ships `endorsement_revoked` (per PRD §5.3 step 5); SL-c ships `sponsor_liability_fired` and `sponsor_liability_escaped` (per PRD §6.2 fire/escape branches); SL-d ships `sponsor_liability_pending` (per PRD §9.3 step 1 transition); restoration-mechanics-v1 PRD ships `restoration_completed` (per PRD §7.4 cross-PRD coordination, with this PRD owning only the escape semantics). All five rows in the registry-rule table get the `(pending)` marker; this is by design, not a registry-pollution bug.

A third observation: the `liability_escape_reason` JSONB schema with `version: 1` (per OQ-V1-SL-05 forward-compat resolution) is set by SL-a but never written-to in SL-a — the writers are SL-b (revocation escape) + SL-c (scheduler-driven fire/escape) + SL-d (transition). SL-a documents the schema in the column COMMENT (per PRD §8.1) and that's it. Any test that tries to read the JSONB at SL-a time will return NULL (no rows have it set yet); that's correct semantics for schema-only.

A fourth observation: SL-a unblocks SL-b and SL-c in parallel — they touch different files (`api_crud/src/governance/revoke_endorsement.rs` vs `crates/routes/src/utils/scheduled_tasks.rs` + `crates/api/api/src/governance/sponsor_liability_grace.rs`) and could in principle cohort-dispatch if their plans land within a polling-cycle of each other. SL-a is the gate for both. Planning SL-a → ship SL-a → cohort-dispatch SL-b + SL-c → SL-d → SL-e is the sequence I'd recommend the user when they get to that decision.

---

_Brief author: advisor session (laptop CWD `C:\Users\barri\Developer\brehon-fork` on `governance-v0` @ `48c4f379d`, 2026-05-03). Brief committed on `governance-v0` before Junior planning task is queued. Per the advisor-orchestrator clarify gate, advisor runs `/brehon-clarify .claude/PRPs/briefs/sl-a-planning-1.md` after this brief commits and pushes; planning task only queues after every clarify-DQ entry is resolved. The earlier-authored `sl-d-planning-1.md` is parked (not deleted) as a draft to revisit at SL-d planning time after SL-a/b/c ship; its §3 Required reading + §4 Constraints will mostly carry forward, only §2 scope changes when the upstream phases have shipped._
