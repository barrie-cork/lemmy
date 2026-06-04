# Plan: v1-sponsor-liability-a — schema + enum + migration foundation

## 1. Summary

v1-SL-a ships the schema-only foundation of the sponsor-liability athgabál
grace-window lane: three new `CaseStatus` enum variants
(`SponsorLiabilityPending` / `SponsorLiabilityFired` /
`SponsorLiabilityEscaped`), two new `moderation_case` columns
(`grace_expires_at TIMESTAMPTZ`, `liability_escape_reason JSONB`), one
scheduler-access partial index (`moderation_case_grace_expires_idx`), one
Issue #24 partial index on `surety` (`surety_sponsored_id_active`),
13 new `governance_config` seed rows (10 `liability.*` + 3
`job.grace_check_*`), 5 new `ENTRY_KIND_*` const declarations
(declared-only — emitting handlers land in SL-b/SL-c/SL-d /
restorative-mechanics-v1 per the registry-rule pre-landed-const
exemption), an ADR-013 enum-exhaustiveness sweep across 6 existing match
sites, and a backfill `UPDATE` for v0 mid-flight cases per ADR-010
won't-disadvantage. No handler ships in SL-a — the lane's actual writers
(revoke_endorsement, scheduler tick, submit_jury_vote split) ship in
SL-b/c/d. Headline acceptance: phase-branch tip passes
`cargo-validate-workspace.yml` + `cargo-validate-migration.yml` + the
e2e `phase1_migrations_round_trip` check on Junior worker pushes; the
`every_seeded_key_has_metadata` + `seeded_keys_count_matches_const_count`
parity tests stay green; the registry-rule's
`rg '^pub const ENTRY_KIND_'` invariant goes 33 -> 38.

## 2. Source

- `.claude/PRPs/briefs/sl-a-planning-1.md` @ `d1c25efb9` — the advisor
  brief (post-`/brehon-clarify`; DQ #114 + #115 resolved 2026-05-03).
- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` @ `governance-v0` —
  parent PRD; §1, §2, §3.1–§3.4, §4, §6, §7, §8.1–§8.5, §10, §11, §15,
  §17, §18 (B4 key-rename table).
- `.claude/PRPs/v1-planning-queue.json` — B4 (key-rename ->
  flat `liability.*`), NOT2 (registry-rule lives at
  `.claude/rules/governance-log-entry-kind-registry.md`), NOT4
  (`ConfigKeyMetadata` as `&'static [ConfigKeyMetadata]`).
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  — ADR-010 (no retroactive invalidation; informs §8.4 backfill 24h
  minor-default), ADR-013 (CaseStatus enum-exhaustiveness invariant —
  load-bearing for §13 Task 5), ADR-015 (pseudonymisation;
  `liability_escape_reason` JSONB carries `actor_pseudonym`, not raw
  `person_id`); OQ-025 (sponsor-liability v1 — resolved into PRD),
  OQ-V1-SL-05 (`liability_escape_reason` schema versioning ->
  `version: 1` from day one).
- `.claude/rules/governance-log-entry-kind-registry.md` — pre-landed-
  const exemption + count-invariant (33 -> 38 post-SL-a) + the
  `sponsor-liability-v1 (reserved)` section that SL-a populates.
- `.claude/rules/decision-queue.md` — schema-v2 attribution + Recipe 2
  (planner-resolved DQ shape) + `kind: "validate-pending"` Shape G
  routing.
- `.claude/rules/advisor-orchestrator.md` — Stage shape "Each impl-task
  complete (under Shape G)" + cohort dispatch (FILES YAML overlap rule
  + budget check non-binding under Shape G) + §G4 classifier.
- `.claude/PRPs/templates/plan.template.md` — 20-section schema (this
  plan literally follows it).

### Lessons that bind §13 decisions

- `feedback_lemmy_migration_runner.md` — use `cargo run -p
  lemmy_diesel_utils --features full -- migration run` / `migration
  revert` (forbid_diesel_cli trigger landed upstream); never raw
  `diesel migration run`. Binds Task 0 (migrate-roundtrip.sh stub
  fix) + Task 8.
- `feedback_advisor_watchpoint_specificity.md` — every §4 watchpoint
  cites a concrete file/table/`schema.rs` line. Binds §4 entries.
- `feedback_complexity_score_pre_split.md` — score 13 -> planner files
  split-or-proceed DQ #116 before commit. Binds §5.2.
- `feedback_explicit_file_arrays_on_tasks.md` — every §13 task carries
  a FILES YAML block; cohort dispatch reads `union(creates, modifies)`.
- `feedback_parallel_cohort_dispatch.md` — Cohort A and Cohort B
  dispatched per `[P]` markers; YAML overlap rule mechanically refuses
  unsafe cohorts.
- `feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md`
  — advisor-side DoD smoke-test runs every §15 command literally before
  plan approval. Under Shape G, the §15 entries name workflow YAML
  paths + `gh run list` queries — both verifiable mid-plan-approval.
- `feedback_features_full_p_crate_incompatible.md` — never
  `-p <crate>` + `--features full`. Workflow YAMLs already comply
  (`cargo check --workspace --features full` per
  `cargo-validate-workspace.yml:88-89`); §15.7 manual snippets respect
  the rule.
- `feedback_features_full_workspace_only.md` — `--features full`
  required to activate `DbEnum` + `ts-rs` derives. Encoded in
  workflow YAML.
- `feedback_insertform_default_propagation.md` — Task 4 (Diesel
  `ModerationCaseInsertForm`) extends with `Option<_>` fields so v0
  callers compile via `..Default::default()`.
- `feedback_brehon_config_micros_scaled.md` — citation-only (per DQ
  #115): the lesson scopes to reputation/score-formula math; SL-a's
  `liability.grace_window_*_hours` keys are wall-clock units (raw
  integer hours), NOT micros-scaled. Binds §13 Task 1 + Task 6 const
  values.
- `feedback_clippy_test_style.md` — R1 every `i32 <-> i64` uses
  `i64::from(...)`. Bound in §4.
- `feedback_schema_changing_spec_retrofit_question.md` — SL-a does
  NOT change any spec/template shape; this gate does not trigger.
- `feedback_read_canonical_before_writing_spec.md` — SL-a does not
  add new commands/rules/lessons/templates; this gate does not
  trigger. Plan §10 still cites JM-a + JM-e + AD-a as canonical
  precedents per the spirit (mirror their §13 task ordering + §15
  shape).
- `feedback_principles_not_rules.md` — score >8 is a signal; planner
  observation in §5.2 leans proceed-as-one with rationale.
- `feedback_junior_worker_e2e_edit_hang.md` — only 1 e2e edit task
  (Task 8) modifies `crates/server/tests/e2e.rs`; single Edit-with-
  anchor (PHASE_1_MIGRATION_COUNT bump + post-condition probes
  extension). No new test functions; backfill semantics deferred to
  SL-c/SL-e e2e tests.
- `feedback_brehon_verify_pre_merge.md` — §16a Stories grain enables
  `/brehon-verify` phantom check before `bm-merge`.

### Related prior plans (canonical-shape mirror)

- `.claude/PRPs/plans/phase-v1-JM-a.plan.md` — closest-shape
  precedent (schema-only first sub-phase of a lane: enum +
  columns + indexes + 27 seed keys + 6 ENTRY_KIND consts +
  parametric `EXPECTED_SEED_COUNT_V1_JM`). SL-a mirrors §13 task
  ordering + §10 patterns + §17 completion checklist.
- `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — most recent
  Shape-G plan + post-spec-kit-adoption. SL-a adopts §15.6 Shape-G
  shape, §16a Stories grain, §13 FILES YAML blocks + `[P]` cohort
  markers, §5.1 complexity breakdown table.
- `.claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md`
  — context-only (the v0 sponsor-liability foundation that SL-d will
  eventually split). SL-a does NOT touch this code.

## 3. Problem statement

The v0 sponsor-liability path (Phase 5b task 56's
`apply_sponsor_liability` at `crates/api/api/src/governance/sponsor_liability.rs:142`)
fires reputation events on sponsors immediately at case-decision time
in the same `run_transaction` as the sanction insert (called from
`submit_jury_vote.rs:467`). This **diverges from Brehon athgabál**
(Higgins 2010 p.7): the surety received no formal notice, has no
opportunity to revoke or restore within a grace window, and learns of
the reputation hit only after the modlog publishes.

PRD §1.1 + §1.2 specify five v1 corrections — formal notice, severity-
proportional grace window, sponsor-revocation escape, defendant-
restoration escape, audit-log of every state change. Implementing all
five simultaneously is too large for one sub-phase: v1-SL was scoped
into 5 sub-phases (PRD §15) with v1-SL-a as the **schema/enum/seed
foundation** the rest depend on.

Without SL-a:

- The three new `CaseStatus` variants don't exist as Rust enum values
  OR Postgres enum values, so SL-d's `submit_jury_vote` rewrite cannot
  transition the case into the grace lifecycle.
- The `moderation_case.grace_expires_at` column doesn't exist, so
  SL-c's scheduler has no field to filter on.
- The `moderation_case.liability_escape_reason` JSONB column doesn't
  exist, so SL-b's revoke_endorsement handler has no field to record
  the escape mechanism.
- The 13 `governance_config` keys aren't seeded, so neither SL-c nor
  SL-d can read tunable defaults.
- The 5 `ENTRY_KIND_*` consts don't exist as canonical Rust strings,
  so every governance-log emission in SL-b/c/d would either use a
  string literal (typo-prone) or fail to compile against the registry-
  rule invariant.
- Issue #24's `surety_sponsored_id_active` partial index doesn't
  exist; `jury_common::select_eligible_jurors` EXISTS subquery and
  `apply_sponsor_liability` sponsor enumeration stay slow.
- v0 mid-flight cases at v1 deploy time get no retroactive grace —
  ADR-010 won't-disadvantage rule is violated.

SL-a closes all of these gaps in a single schema-only sub-phase. The
deliverables are mechanical (no business logic; everything is
declarations + seed rows + a backfill UPDATE) and the JM-a precedent
proves the pattern works.

## 4. Solution statement

One Postgres migration combines six SQL effects in a single
`-- no-transaction` file (PRD §8.5 ordering): three `ALTER TYPE
case_status ADD VALUE IF NOT EXISTS` for the new variants; two `ALTER
TABLE moderation_case ADD COLUMN` for `grace_expires_at` (NULLABLE
TIMESTAMPTZ) + `liability_escape_reason` (NULLABLE JSONB) each with a
`COMMENT ON COLUMN` explaining the v1 semantics; partial index
`moderation_case_grace_expires_idx ON moderation_case
(grace_expires_at) WHERE status = 'SponsorLiabilityPending'` (the
scheduler's primary access path); partial index
`surety_sponsored_id_active ON surety (sponsored_id, sponsor_id) WHERE
revoked_at IS NULL` (folds Issue #24); idempotent `INSERT INTO
governance_config ... ON CONFLICT DO NOTHING` for 13 keys (10
`liability.*` + 3 `job.grace_check_*`); backfill `UPDATE
moderation_case SET status = 'SponsorLiabilityPending', grace_expires_at
= decided_at + INTERVAL '24 hours' WHERE ...` with three nested
`EXISTS` guards (surety presence, sanction presence, no-prior
reputation_event) bounded by `decided_at > now() - INTERVAL '24
hours'`.

Rust side mirrors: enum extension in
`crates/db_schema_file/src/enums.rs:393-408` (3 new variants); column
declarations in `crates/db_schema_file/src/schema.rs:772-798`
`moderation_case` table block (2 new lines); `ModerationCase` +
`ModerationCaseInsertForm` extension in
`crates/db_schema/src/source/governance/moderation_case.rs:21-122`
(2 new fields, both `Option<_>` so v0 callers compile via
`..Default::default()`).

ADR-013 enum-exhaustiveness sweep updates 6 existing match sites
(grep-enumerated at planning time — see §4 watchpoint 1) to handle
the three new variants explicitly. No `_ =>` arms.

config.rs additions: 13 new `pub const DEFAULT_LIABILITY_*` /
`DEFAULT_JOB_GRACE_*` declarations alongside existing ones; 13 new
match arms across `const_default_int` / `_float` / `_bool` / `_text`;
13 new `SEEDED_KEYS_WITH_CONSTS` tuples; new
`pub const EXPECTED_SEED_COUNT_V1_SL: usize = 13;` beside the
existing parametric constants; parity-test extension at line
2422-2436 to add the new term to the sum and error-message; 13 new
`CONFIG_KEY_METADATA` struct literals (per NOT4 resolution +
v1-AD-a precedent).

5 new `ENTRY_KIND_*` consts: dual-file edit (declare in
`crates/db_schema/src/source/governance/governance_log.rs` after the
v1-JM-c block; re-export alphabetically in
`crates/api/api/src/governance/governance_log.rs`); registry rule's
`sponsor-liability-v1 (reserved)` section populated with 5 rows
including `(pending)` markers + downstream-plan citations per the
pre-landed-const exemption (SL-b for endorsement_revoked + escape;
SL-c for sponsor_liability_fired + escape; SL-d for
sponsor_liability_pending; restorative-mechanics-v1 PRD for
restoration_completed). Registry count goes 33 -> 38.

E2e migration round-trip extension: `crates/server/tests/e2e.rs`
extends `PHASE_1_MIGRATION_COUNT` by +1 and extends the post-condition
probe lists in `phase1_migrations_round_trip` to assert the new
column / index / enum-value names exist post-up.sql and (where
Postgres permits) disappear post-down.sql.

DoD per Shape G: every impl-task pushes its worker branch and writes
`kind: "validate-pending"` for `cargo-validate-workspace.yml`;
migration-touching tasks additionally trigger
`cargo-validate-migration.yml`. Post-finalize-merge into the phase
branch, the advisor surfaces the Phase 2 e2e local-vs-dispatch user
gate per PR #105 (2026-04-28).

## 5. Metadata

- **Phase:** `v1-SL-a`
- **Branch:** `phase-v1-SL-a` (cut by BM-task before Task 1)
- **Estimated tasks:** 10 (Task 0 pre-flight + Tasks 1-8 impl + Task 9
  retro)
- **Estimated cargo budget:** 0 GB peak local (Shape G — cargo runs on
  GH-hosted runners)
- **Forbidden-window applicability:** non-binding for impl-task
  throughput under Shape G; standard for any local diagnostic cargo
  run (rare under Shape G)
- **Complexity score:** **13/10** — see breakdown below. Threshold-
  tripping; planner DQ #116 filed.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Computed mechanically
from §13 task list:

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **3** | 8 impl tasks (Tasks 1-8; Task 0 + retro excluded). `max(0, 8-5) = 3` |
| Migrations touched | +2 each | **2** | One logical migration: `migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/{up,down}.sql` (per `feedback_parallel_cohort_dispatch.md` "the up.sql + down.sql pair is one logical unit") |
| Crates touched | +1 each | **5** | `lemmy_db_schema_file` (enums + schema), `lemmy_db_schema` (moderation_case + governance_log source), `lemmy_api` (config.rs + governance_log shim + 5 ADR-013 sites + submit_jury_vote.rs ADR-013 fix), `lemmy_api_crud` (request_appeal.rs ADR-013), `lemmy_server` (tests/e2e.rs round-trip) |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | **3** | Task 8 modifies `crates/server/tests/e2e.rs` — file-class match. 1 task × +3 = 3 |
| New ADR-affecting decisions | +2 each | **0** | PRD already settled all decisions; SL-a is implementation only. The ADR-013 enum-exhaustiveness sweep is enforcement of an existing ADR, not a new decision |
| Cargo budget peak above 6 GB | +1 per GB | **0** | Shape G — cargo runs off-box |
| **Total** | — | **13** | Threshold for split-DQ: `>8`. **Tripped.** |

### 5.2 Split-or-proceed DQ

Per `feedback_complexity_score_pre_split.md` + `planning.md` §5
"Decision-queue — pre-seed forward-looking OQs": planner files
**DQ #116** (`from: "planner"`, `kind: "blocker"`, `answered_by:
null`) BEFORE committing the plan, asking "Complexity score 13
exceeds 8 — split `v1-sponsor-liability-a` into `v1-SL-a-1` (Tasks
1-5) + `v1-SL-a-2` (Tasks 6-8) + retro, or proceed as one plan?"

**Planner observation (non-binding lean):** the dominant factors are
the 5-crate footprint (+5) and the e2e round-trip extension (+3).
Splitting would yield scores of `0+2+4+0 = 6` (SL-a-1: 5 impl tasks,
1 migration, 4 crates, 0 e2e) and `0+0+3+3 = 6` (SL-a-2: 3 impl
tasks, 0 migrations, 3 crates, 1 e2e). Both pieces would be < 8 — but
the split fragments a logically atomic schema-foundation sub-phase
(PRD §15 row 1 is named "Schema + enum + migration", a single
deliverable). JM-a shipped as one at a similar shape (11 tasks
including 27 seed keys + 6 ENTRY_KIND consts), and JM-e shipped at
score 15 under proceed-as-one with no operational regret per the JM-e
retro.

The brief's pre-estimate of 5-7 was mathematically optimistic
(undercounted crates by missing `api_crud` + the e2e round-trip
extension; mis-attributed `lemmy_diesel_utils` as a touched crate
when only the shell wrapper changes). This plan ships under the
**proceed-as-one** assumption pending DQ #116 resolution.

Per `feedback_principles_not_rules.md`: the score is a signal, not a
hard rule. Mechanical schema work with a strong precedent does not
benefit from artificial fragmentation.

---

## 6. Relationship to other v1-SL sub-phases

Per PRD §15:

| Sub-phase | Status | What it ships | SL-a relationship |
|---|---|---|---|
| **v1-SL-a (THIS PLAN)** | NOT YET CUT | Schema + enum + migration + 13 seeds + 5 ENTRY_KIND consts + Issue #24 index + ADR-013 sweep + backfill | — |
| v1-SL-b | pending (after SL-a) | `revoke_endorsement` handler + DTO + route + integration tests | SL-b reads `liability.revoke_rate_limit_per_day` SL-a seeds; emits `endorsement_revoked` const SL-a declares; reads `moderation_case.status = SponsorLiabilityPending` SL-a creates; writes `liability_escape_reason` JSONB column SL-a creates; calls `governance_log::append(ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED, ...)` (escape branch) |
| v1-SL-c | pending (after SL-a; parallel-safe with SL-b) | Scheduler module `sponsor_liability_grace.rs` + clokwerk wiring at 5-min tick + per-case `FOR UPDATE` + staleness check | SL-c reads `moderation_case.grace_expires_at` + `status = SponsorLiabilityPending` rows via `moderation_case_grace_expires_idx` SL-a creates; reads `job.grace_check_*` keys SL-a seeds; emits `sponsor_liability_fired` + `sponsor_liability_escaped` consts SL-a declares |
| v1-SL-d | pending (after SL-c) | `submit_jury_vote` mutation + `apply_sponsor_liability` compute/fire split + `Decided -> SponsorLiabilityPending` transition | SL-d transitions cases into the new variant SL-a creates; reads `liability.grace_window_*_hours` SL-a seeds; writes `moderation_case.grace_expires_at` SL-a creates; emits `sponsor_liability_pending` const SL-a declares; the v0 `apply_sponsor_liability` at `sponsor_liability.rs:142` stays intact through SL-a (SL-d is the rewrite) |
| v1-SL-e | pending (after SL-d) | e2e suite for the lane (revocation-during-window-escapes; restoration-during-window-escapes; window-expiry-fires; backfill-of-mid-flight) | SL-e validates SL-a's backfill UPDATE behaviour end-to-end; SL-a's own validation is migration-round-trip + workspace-check + parity tests + the ADR-013 grep-sweep, not a behavioural e2e |

**Cross-PRD sequencing (per PRD §17.1 + §14):**

- **admin-dashboard-v1 PRD** provides the config-write surface for
  §4.2 community-scoped grace-window keys (not blocking for SL-a:
  admins can edit via direct `psql` until dashboard ships).
- **jury-mechanics-v1 PRD** provides the v1 severity tiers
  (Low/Medium/High/Critical -> Minor/Moderate/Severe sponsor-liability
  buckets) that feed §4.1 grace-window duration mapping. SL-a does NOT
  read these — `severity_for_action` already exists at
  `crates/api/api/src/governance/sponsor_liability.rs:112`.
- **restorative-mechanics-v1 PRD** owns the
  `POST /api/v4/governance/restoration/complete` endpoint. SL-a
  declares the `ENTRY_KIND_RESTORATION_COMPLETED` const that the
  restorative endpoint will emit; the restorative endpoint itself
  ships in that PRD's plan.

## 7. Preflight guardrails inherited from prior phases

Per `.claude/rules/pre-phase-harness-audit.md` + JM-a/c/d/e + AD-a/c
retros + applicable lessons:

- **R1** — every `i32 <-> i64` comparison uses `i64::from(...)`
  (per `feedback_clippy_test_style.md`). Bound in §13 Tasks 4 + 8.
- **R5** — Task 0 enumerates ALL probes explicitly (per JM-b retro
  Event 4 + pre-phase-harness-audit.md). Probes 0..10 listed in §13
  Task 0.
- **R6** — uniform `--no-deps -- -D warnings` clippy. Encoded in
  `cargo-validate-workspace.yml:92`.
- **R7** — `cargo test --no-run -p lemmy_server --test e2e` on tasks
  touching a struct or re-export. Encoded in
  `cargo-validate-workspace.yml:95`.
- **JM-a Task 7 reconciliation gate** — pre-commit count-match
  between `SEEDED_KEYS_WITH_CONSTS` block + seed migration's INSERT
  count + `EXPECTED_SEED_COUNT_V1_SL`. Bound in §13 Task 6 inline
  reconciliation block.
- **JM-d retro §5 — `feedback_laptop_default_for_validate_pending.md`** —
  Phase 2 e2e local-default. Honoured at advisor-side surfacing per
  `advisor-orchestrator.md` mandatory user gate "Phase 2 e2e — local
  vs dispatch".
- **DQ #67 resolution (user 2026-04-27)** — workflow YAML DoD dry-run
  discipline: §15 does NOT prescribe `act` invocations.
- **DQ #114 (user 2026-05-03)** — `migrate-roundtrip.sh` stub fix is
  folded into Task 0 of THIS plan (closes DQ #68 atomically with
  SL-a's first phase-branch push).
- **DQ #115 (advisor 2026-05-03)** — `liability.grace_window_*_hours`
  keys are stored as raw integer hours, NOT micros-scaled. Per
  `feedback_brehon_config_micros_scaled.md` lesson scope is
  reputation/score-formula math; wall-clock units are out of scope.
  Bound in §13 Task 1 (seed values) + Task 6 (Rust const types `i64`).
- **PMD #126** — gov-v0 -> phase-branch merge-up with DQ renumbering.
  Bound in §18 Risks.

## 8. Flow design

### 8.1 Before state (governance-v0 @ `48c4f379d`)

`CaseStatus` enum (Rust + Postgres) has 9 variants. `moderation_case`
table has no `grace_expires_at` / `liability_escape_reason` columns.
`surety` table has no `surety_sponsored_id_active` partial index.
`SEEDED_KEYS_WITH_CONSTS` has 88 entries summing to
`EXPECTED_SEED_COUNT (34) + EXPECTED_SEED_COUNT_V1_AD (27) +
EXPECTED_SEED_COUNT_V1_JM (27) = 88`. `governance_log.rs` has 33
`ENTRY_KIND_*` consts. Per `sponsor_liability.rs:142`, v0
`apply_sponsor_liability` writes `reputation_event` rows + governance
log entries immediately at `submit_jury_vote.rs:467` decision time.

ADR-013 match sites (grep at brief-write time, verified by Task 0
Probe 4 at impl time):

- `crates/api/api_crud/src/governance/request_appeal.rs:94-103` —
  `case.status` exhaustive match (8 explicit arms; no `_`)
- `crates/api/api/src/governance/admin_close_case.rs:65-75` —
  `case.status` exhaustive match (8 explicit arms; no `_`)
- `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs:72-81`
  — `case.status` exhaustive match (8 explicit arms; no `_`)
- `crates/api/api/src/governance/accept_jury_assignment.rs:111-132` —
  TWO nested matches keyed on `JuryAssignmentRole` × `case.status`
  (each branch lists 8 arms exhaustively; no `_`)
- `crates/api/api/src/governance/admin_assign_jury.rs:140-147` —
  `case.status` exhaustive match (8 explicit arms; no `_`)
- `crates/api/api/src/governance/submit_jury_vote.rs:271-276`
  (`process_vote` step-5 idempotency guard) — `matches!()` pattern;
  new variants don't force compile errors but require deliberate
  inclusion-or-exclusion decisions per ADR-013 design intent.
  `submit_jury_vote.rs:729-732` (`process_appeal_vote` step-6 narrower
  terminal guard) stays unchanged per §10.5 row 8 rationale.

### 8.2 After state (post-SL-a phase-branch tip)

`CaseStatus` enum has 12 variants. `moderation_case` table has two
new NULLABLE columns + one partial index. `surety` table has the
Issue #24 partial index. `SEEDED_KEYS_WITH_CONSTS` has 101 entries
summing to `88 + EXPECTED_SEED_COUNT_V1_SL (13) = 101`.
`governance_log.rs` has 38 `ENTRY_KIND_*` consts.

The 8 ADR-013 match sites (one site per file × 6 files; with
`accept_jury_assignment` having two matches and `submit_jury_vote`
having two `matches!()` guards) all explicitly enumerate the three
new variants per the per-file decisions documented in §10.5.

v0 mid-flight cases (those with `decided_at` within the last 24h of
v1-SL-a deploy time AND active sureties AND a sanction row AND no
prior `reputation_event` with `reason = 'sponsor_liability_applied'`)
have their `status` flipped to `SponsorLiabilityPending` with
`grace_expires_at = decided_at + INTERVAL '24 hours'`. SL-c's
scheduler picks them up.

### 8.3 Endpoint changes

None in SL-a. The grace-window enters use only when SL-d's
`submit_jury_vote` rewrite ships. Wire-format: v0 clients reading
`GET /api/v4/governance/case` observe the three new `CaseStatus`
variants only on cases v1 deploy backfilled OR future cases after
SL-d ships. Per PRD §11.3 deprecation pattern: clients matching
exhaustively on `CaseStatus` will fail-loud; clients switching on a
few variants with default-ignore are unaffected.

---

## 9. Mandatory reading

### 9.1 Brehon design docs (P0)

- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` — full read first
  iteration. Specifically §1, §2, §3.1–§3.4, §4, §6, §7, §8.1–§8.5,
  §10, §11, §17, §18 (B4 key-rename table).
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  — ADR-010, ADR-013, ADR-015 + OQ-025 + OQ-V1-SL-05.
- `.claude/rules/governance-log-entry-kind-registry.md` — pre-landed-
  const exemption + count invariant (33 -> 38) + the
  `sponsor-liability-v1 (reserved)` section that Task 7 populates.
- `.claude/PRPs/v1-planning-queue.json` — B4 (key-rename), NOT2
  (registry rule), NOT4 (ConfigKeyMetadata shape).

### 9.2 Codebase reads (P0 — mirror these patterns)

- `crates/db_schema_file/src/enums.rs:380-408` — `CaseStatus` enum
  declaration. Task 2 mirrors.
- `crates/db_schema_file/src/schema.rs:763-799` — `moderation_case`
  table block. Task 3 inserts two new lines after `winning_decision`.
- `crates/db_schema_file/src/schema.rs:1343-1352` — `surety` table
  block (no edit needed; only adding an index).
- `crates/db_schema/src/source/governance/moderation_case.rs:1-123` —
  `ModerationCase` struct + `ModerationCaseInsertForm`. Task 4
  extends both.
- `crates/db_schema/src/source/governance/governance_log.rs:105-185` —
  `ENTRY_KIND_*` const block. Task 7 inserts five new consts after
  the existing v1-JM-a/JM-c block.
- `crates/api/api/src/governance/governance_log.rs:39-76` — api shim
  re-export block. Task 7 adds five new alphabetically-ordered
  `pub use` lines.
- `crates/api/api/src/governance/config.rs:770-902` — DEFAULT_*
  declarations (the v1-JM-a additions block at lines 850-901 is the
  closest precedent). Task 6 appends a v1-SL-a additions block.
- `crates/api/api/src/governance/config.rs:903-1063` —
  `const_default_int` / `_float` / `_bool` / `_text` match-arm
  functions. Task 6 adds 13 new match arms across the four functions.
- `crates/api/api/src/governance/config.rs:1067-1299` —
  `SEEDED_KEYS_WITH_CONSTS` array. Task 6 appends 13 new tuples.
- `crates/api/api/src/governance/config.rs:1304-1322` —
  parametric `EXPECTED_SEED_COUNT_*` constants. Task 6 adds
  `EXPECTED_SEED_COUNT_V1_SL = 13`.
- `crates/api/api/src/governance/config.rs:1345-2414` —
  `CONFIG_KEY_METADATA` array. Task 6 appends 13 new entries.
- `crates/api/api/src/governance/config.rs:2420-2465` — parity
  module. Task 6 amends `seeded_keys_count_matches_const_count`.
- `crates/api/api/src/governance/sponsor_liability.rs:1-358` — context
  only. Read `severity_for_action` at lines 112-124.
- `crates/api/api_crud/src/governance/request_appeal.rs:90-110`,
  `crates/api/api/src/governance/admin_close_case.rs:65-78`,
  `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs:72-83`,
  `crates/api/api/src/governance/accept_jury_assignment.rs:111-135`,
  `crates/api/api/src/governance/admin_assign_jury.rs:140-150`,
  `crates/api/api/src/governance/submit_jury_vote.rs:265-285` +
  `:725-740` — the 6 ADR-013 match sites Task 5 updates.
- `crates/server/tests/e2e.rs:60-100` (migration-runner fixture)
  + `:300-450` (`phase1_migrations_round_trip` block) +
  `:1260-1280` (`PHASE_1_MIGRATION_COUNT` constant). Task 8 extends
  both the migration-count constant and the post-condition probe
  lists.
- `migrations/2026-04-22-000300-0000_seed_v1_config_keys/{up,down}.sql`
  — v1-AD-a precedent for an idempotent seed migration with 27 INSERT
  rows + LIFO down DELETE.
- `migrations/2026-04-23-000000-0000_add_jury_mechanics_enums/up.sql`
  + `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql`
  — JM-a precedents for `CREATE TYPE ... AS ENUM` + `ALTER TABLE ...
  ADD COLUMN` + backfill UPDATE patterns.
- `crates/diesel_utils/src/schema_setup/mod.rs:185` (the `pub fn run`
  entrypoint) + `:214` (the `pg_advisory_lock(0)` + forbid_diesel_cli
  bypass). Confirms the migration runner's CLI surface for Task 0
  migrate-roundtrip.sh fix.

### 9.3 Rules (P0 — auto-loaded)

- `.claude/rules/governance-log-entry-kind-registry.md` — registry
  invariants.
- `.claude/rules/decision-queue.md` — schema-v2 attribution + Recipe
  2 + `kind: "validate-pending"` Shape G routing.
- `.claude/rules/advisor-orchestrator.md` — Stage shape "Each impl-
  task complete (under Shape G)" + cohort dispatch + §G4 classifier +
  Phase 2 e2e user gate.
- `.claude/rules/branch-manager.md` + `.claude/rules/phase-branch.md`
  — phase-branch discipline.
- `.claude/rules/cargo-output-capture.md` +
  `.claude/rules/no-cargo-output-paste.md`.
- `.claude/rules/pre-phase-harness-audit.md` — Task 0 probe rubric.
- `.claude/rules/pm-plugin-hooks-stable.md` — Task 0 Probe 7.

### 9.4 External documentation

- chrono: `Duration::hours(i64)` and `DateTime<Utc>` — already used
  across the governance crates.
- diesel + diesel-async: `RunQueryDsl::execute`, `.set((...))` tuple
  form. Already used in `submit_jury_vote.rs` and elsewhere.
- serde_json: `json!(...)` shape for `liability_escape_reason`
  payload schema documentation in column COMMENT.

---

## 10. Patterns to mirror

Per `feedback_advisor_watchpoint_specificity.md`: each entry cites a
specific table, file, or `schema.rs` line.

### 10.1 Combined SQL migration (enum + columns + indexes + seeds + backfill)

**Mirror primary:** `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql`
(lines 1-50 — header doc-comment) +
`migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql` (lines
1-30 — idempotent ON CONFLICT seed pattern).

**Mirror enum-additions:** Phase 5b task 56's `Restoration` variant
addition migration used `-- no-transaction` per the
`ALTER TYPE case_status ADD VALUE IF NOT EXISTS ...` Diesel directive
requirement.

**up.sql skeleton (Task 1 IMPLEMENT):**

```sql
-- v1-SL-a task 1: combined schema + seed + backfill migration.
-- ============================================================
-- ADR exception trail (protected governance tables)
-- ============================================================
-- ADDITIVE only: ADD COLUMN, CREATE INDEX, ALTER TYPE ... ADD VALUE
-- IF NOT EXISTS, INSERT ... ON CONFLICT DO NOTHING, UPDATE ...
-- (backfill only, bounded by `decided_at > now() - INTERVAL '24
-- hours'` per ADR-010 won't-disadvantage). No DROP, no ALTER on
-- existing columns, no UPDATE on already-fired cases.
--
-- Controlling ADR: ADR-010 (staged releases).
-- Authority trail:
--   - PRD: .claude/PRPs/prds/v1-sponsor-liability.prd.md §8.1-§8.5
--   - Plan: .claude/PRPs/plans/v1-sponsor-liability-a.plan.md §10.1, Task 1
--   - DQ #115 (advisor 2026-05-03): grace_window_*_hours stored as
--     raw integer hours, NOT micros-scaled.
-- ============================================================

-- no-transaction
-- (Required for ALTER TYPE; per Phase 5b Restoration variant precedent.)

ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityPending';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityFired';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityEscaped';

ALTER TABLE moderation_case ADD COLUMN grace_expires_at TIMESTAMPTZ;
COMMENT ON COLUMN moderation_case.grace_expires_at IS
    'Per OQ-025 v1 sponsor-liability grace window. Set when transitioning
     Decided -> SponsorLiabilityPending; locked thereafter. NULL for cases
     not in the grace lifecycle (NoAction outcomes, no-sponsor target,
     v0 backfill exclusions).';

ALTER TABLE moderation_case ADD COLUMN liability_escape_reason JSONB;
COMMENT ON COLUMN moderation_case.liability_escape_reason IS
    'Per OQ-025 + OQ-V1-SL-05: structured escape-reason payload.
     Schema (version: 1):
       {"version": 1,
        "reason": "sponsor_revoked"|"restoration_completed"|"admin_override",
        "actor_pseudonym": "<scrubbed via ADR-015>",
        "endorsement_id": <i32>|null,
        "restoration_id": <i32>|null}.
     NULL for non-escaped cases.';

CREATE INDEX moderation_case_grace_expires_idx
    ON moderation_case (grace_expires_at)
    WHERE status = 'SponsorLiabilityPending';
COMMENT ON INDEX moderation_case_grace_expires_idx IS
    'Per PRD §8.1: scheduler primary access path. Partial index keeps
     the index small (~handful of pending cases at any time) and bounds
     the SL-c grace-check batch query in O(rows-pending).';

CREATE INDEX surety_sponsored_id_active
    ON surety (sponsored_id, sponsor_id)
    WHERE revoked_at IS NULL;
COMMENT ON INDEX surety_sponsored_id_active IS
    'Per Issue #24 (CodeRabbit, PR #10). Speeds up
     jury_common::select_eligible_jurors EXISTS subquery and
     apply_sponsor_liability sponsor enumeration. Partial WHERE matches
     the existing "active sureties" filter pattern in both call sites.';

INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES
    -- 6 grace-window hour keys (raw integer hours per DQ #115)
    ('instance', 'liability.grace_window_minor_hours',                   'int',   24,    NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_moderate_hours',                'int',   72,    NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_severe_hours',                  'int',   168,   NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_minimum_hours',                 'int',   1,     NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_maximum_hours',                 'int',   720,   NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_alert_threshold_hours',         'int',   24,    NULL, NULL,  NULL),
    -- 2 restoration-escape keys (per PRD §7.3)
    ('instance', 'liability.restoration_escapes_liability',              'bool',  NULL,  NULL, true,  NULL),
    ('instance', 'liability.restoration_severity_reduction_steps',       'int',   0,     NULL, NULL,  NULL),
    -- 1 multi-sponsor escape rule (per PRD §13.1 OQ-V1-SL-01)
    ('instance', 'liability.multi_sponsor_escape_rule',                  'text',  NULL,  NULL, NULL,  'any_revocation'),
    -- 1 revocation rate-limit (per PRD §12.1)
    ('instance', 'liability.revoke_rate_limit_per_day',                  'int',   5,     NULL, NULL,  NULL),
    -- 3 grace-check scheduler keys (per PRD §6.4)
    ('instance', 'job.grace_check_interval_minutes',                     'int',   5,     NULL, NULL,  NULL),
    ('instance', 'job.grace_check_batch_size',                           'int',   100,   NULL, NULL,  NULL),
    ('instance', 'job.grace_check_staleness_alert_multiplier',           'float', NULL,  2.0,  NULL,  NULL)
ON CONFLICT (scope, key, valid_from) DO NOTHING;

UPDATE moderation_case
SET status = 'SponsorLiabilityPending',
    grace_expires_at = decided_at + INTERVAL '24 hours'
WHERE status = 'Decided'
  AND decided_at IS NOT NULL
  AND decided_at > now() - INTERVAL '24 hours'
  AND target_person_id IS NOT NULL
  AND id IN (
    SELECT mc.id
    FROM moderation_case mc
    WHERE EXISTS (
      SELECT 1 FROM surety s
      WHERE s.sponsored_id = mc.target_person_id
        AND s.revoked_at IS NULL
    )
    AND EXISTS (
      SELECT 1 FROM sanction sa
      WHERE sa.case_id = mc.id
    )
    AND NOT EXISTS (
      SELECT 1 FROM reputation_event re
      WHERE re.source_case_id = mc.id
        AND re.reason = 'sponsor_liability_applied'
    )
  );
```

**down.sql skeleton:**

```sql
-- Reverse of v1-SL-a Task 1 up.sql.
--
-- Postgres enum-value drop is unsupported without full type rebuild —
-- the three new variants stay as orphan values in case_status (per
-- Phase 5b Restoration precedent + PRD §3.4 doc-comment). down.sql
-- documents this and recovers everything else.

-- Reverse the backfill: any case still in SponsorLiabilityPending
-- that was set by Task 1's UPDATE goes back to Decided. The
-- `grace_expires_at IS NOT NULL` guard distinguishes backfilled rows
-- from forward-progress writes (none yet at SL-a time, but
-- defence-in-depth).
UPDATE moderation_case
SET status = 'Decided',
    grace_expires_at = NULL
WHERE status = 'SponsorLiabilityPending'
  AND grace_expires_at IS NOT NULL;

DELETE FROM governance_config
WHERE scope = 'instance' AND key IN (
    'liability.grace_window_minor_hours',
    'liability.grace_window_moderate_hours',
    'liability.grace_window_severe_hours',
    'liability.grace_window_minimum_hours',
    'liability.grace_window_maximum_hours',
    'liability.grace_window_alert_threshold_hours',
    'liability.restoration_escapes_liability',
    'liability.restoration_severity_reduction_steps',
    'liability.multi_sponsor_escape_rule',
    'liability.revoke_rate_limit_per_day',
    'job.grace_check_interval_minutes',
    'job.grace_check_batch_size',
    'job.grace_check_staleness_alert_multiplier'
);

DROP INDEX IF EXISTS surety_sponsored_id_active;
DROP INDEX IF EXISTS moderation_case_grace_expires_idx;

ALTER TABLE moderation_case DROP COLUMN IF EXISTS liability_escape_reason;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS grace_expires_at;

-- Postgres enum values (SponsorLiabilityPending/Fired/Escaped) remain
-- as orphan variants in the case_status type — Postgres does not
-- support DROP VALUE without a full type rebuild. This matches the
-- Phase 5b Restoration variant down.sql doc-comment precedent.
```

### 10.2 Rust enum extension (Task 2)

**Mirror:** `crates/db_schema_file/src/enums.rs:382-408` `CaseStatus`
declaration. Three new variants insert AFTER `AdminReview` (line 407)
and BEFORE the closing `}` (line 408). Each new variant gets a `///`
doc-comment citing PRD §3.1 + the OQ-025 origin.

```rust
pub enum CaseStatus {
  // ... existing 9 variants unchanged ...
  AdminReview,
  /// v1-SL-a §8.1 + PRD §3.1 (OQ-025 athgabál grace window). Case has
  /// been Decided AND a sanction with sponsor-liability implications
  /// was created; the case is in its grace window.
  /// `moderation_case.grace_expires_at` holds the computed deadline.
  /// Sponsor revocation OR defendant restoration during this window
  /// transitions to `SponsorLiabilityEscaped`. Window expiry triggers
  /// `SponsorLiabilityFired`. Set by SL-d (`submit_jury_vote` rewrite);
  /// pre-v1 backfill in v1-SL-a sets it for v0 mid-flight cases.
  SponsorLiabilityPending,
  /// v1-SL-a §8.1 + PRD §3.1. Terminal — the grace window expired
  /// without escape. The `apply_sponsor_liability` helper (v0 Phase 5b
  /// code, gated post-SL-c) ran and the `reputation_event` rows for
  /// sponsors were written. Set by SL-c scheduler.
  SponsorLiabilityFired,
  /// v1-SL-a §8.1 + PRD §3.1. Terminal — sponsor revoked OR defendant
  /// restored within the grace window. `moderation_case.liability_escape_reason`
  /// records the escape mechanism. No `reputation_event` rows for
  /// sponsors were written. Set by SL-b (revoke_endorsement) or SL-c
  /// (scheduler escape branch).
  SponsorLiabilityEscaped,
}
```

The existing derive set is unchanged — DbEnum picks up new variants
automatically; verbatim style maps PascalCase Rust -> PascalCase
Postgres.

### 10.3 schema.rs column additions (Task 3)

**Mirror:** the existing `winning_decision` line at
`crates/db_schema_file/src/schema.rs:797`. Insert two new column
declarations between `winning_decision` (line 797) and the closing
`}` of the `moderation_case (id)` table block (line 798).

```rust
    moderation_case (id) {
        // ... existing columns 1-20 unchanged ...
        winning_decision -> Nullable<JuryDecision>,
        grace_expires_at -> Nullable<Timestamptz>,
        liability_escape_reason -> Nullable<Jsonb>,
    }
```

The `use diesel::sql_types::*` block at line 764 already covers
`Timestamptz` and `Jsonb`. No `joinable!` macro changes; no
`sql_types` module changes. The two new partial indexes are NOT
declared in schema.rs — Diesel's `table!` macro only tracks columns +
foreign keys.

### 10.4 ModerationCase + InsertForm extension (Task 4)

**Mirror:** the existing v1-AD-a + v1-JM-a additions at
`crates/db_schema/src/source/governance/moderation_case.rs:38-83`
(struct extensions) + `:102-122` (InsertForm extensions; all
`Option<_>`).

```rust
// In `pub struct ModerationCase`, after winning_decision (line 83):
pub winning_decision: Option<JuryDecision>,
/// v1-SL-a §8.1: grace-window deadline. NULL for cases not in
/// `SponsorLiabilityPending` state. Set by SL-d's `submit_jury_vote`
/// rewrite at `Decided -> SponsorLiabilityPending` transition; pre-v1
/// backfill sets it for v0 mid-flight cases. Locked thereafter.
pub grace_expires_at: Option<DateTime<Utc>>,
/// v1-SL-a §8.1 + OQ-V1-SL-05: structured escape-reason payload
/// (version: 1 from day one). NULL for non-escaped cases. Set by
/// SL-b (revoke_endorsement escape branch) or SL-c (scheduler escape
/// branch).
pub liability_escape_reason: Option<Value>,
```

```rust
// In `pub struct ModerationCaseInsertForm`, after winning_decision (line 122):
pub winning_decision: Option<JuryDecision>,
/// v1-SL-a additions. Both `Option<_>` so v0/earlier-v1 callers
/// continue to compile via `..Default::default()`.
pub grace_expires_at: Option<DateTime<Utc>>,
pub liability_escape_reason: Option<Value>,
```

Imports: existing `use serde_json::Value` covers
`liability_escape_reason`; existing `use chrono::{DateTime, Utc}`
covers `grace_expires_at`. No new imports.

### 10.5 ADR-013 exhaustive-match extension shape (Task 5)

**Mirror primary:** the v1-JM-a precedent — when `JuryAssignmentRole`
gained `Appeal`, every existing `match jury_assignment.role` site
got the new arm. Same discipline for `CaseStatus` + 3 new variants.

**Per-site decision matrix (per PRD §3.3 + §11.3):**

| Site | New variant arm decision (with rationale) |
|---|---|
| `request_appeal.rs:94-103` | `SponsorLiabilityPending => {}` (allow appeal during grace; per PRD §3.3 row 1: appeals during grace window are valid; appeals reset case to `Appealed` and cancel the grace timer). `SponsorLiabilityFired \| SponsorLiabilityEscaped => return Err(LemmyErrorType::NotFound)` (case is terminal — appeal window already passed) |
| `admin_close_case.rs:65-75` | All three new variants added to the allowed-set (admins may force-close terminal liability states for ops purposes — e.g. scheduler is broken and case is stuck in `SponsorLiabilityPending` past the staleness threshold) |
| `admin_trigger_appeal_rejury.rs:72-81` | All three new variants added to the rejection set (appeal rejury only valid on `Appealed` cases; sponsor-liability lifecycle is orthogonal to appeal lifecycle) |
| `accept_jury_assignment.rs:111-122` (Original branch) | All three new variants added to the rejection set (original-jury cases are pre-Decided; sponsor-liability lifecycle is post-Decided) |
| `accept_jury_assignment.rs:123-134` (Appeal branch) | All three new variants added to the rejection set (appeal-jury seated only on Appealed cases) |
| `admin_assign_jury.rs:140-147` | All three new variants added to the rejection set (jury assignment only valid on Open / ThresholdMet / EmergencyRemove; sponsor-liability is post-decision) |
| `submit_jury_vote.rs:271-276` (`process_vote` step-5 idempotency guard) | All three new variants added to the early-return list — same semantics as `Decided`/`Closed`/`Appealed` (case has progressed past vote-tally; new vote should short-circuit with `case_decided: true`) |
| `submit_jury_vote.rs:729-732` (`process_appeal_vote` step-6 narrower terminal guard) | New variants NOT added — appeal-vote cases reach this code only when `case.status == Appealed` (per `process_appeal_vote` dispatch precondition); a sponsor-liability case never transitions to Appealed once in the grace lifecycle (SL-d's transition is `Decided -> SponsorLiabilityPending` direct, not via Appealed). The terminal-guard list at line 729 (`Closed | EmergencyRemove | AdminReview`) stays intact |

### 10.6 config.rs 13-key extension shape (Task 6)

**Mirror:** v1-JM-a precedent at
`crates/api/api/src/governance/config.rs:850-902` (DEFAULT_*
declarations) + `:1175-1298` (SEEDED_KEYS_WITH_CONSTS v1-JM-a block)
+ `:2422-2436` (parity-test extension).

**13 new DEFAULT_* declarations** (insert in a new `// -- v1-SL-a
additions ... --` block AFTER the v1-JM-a block at line 902):

```rust
// -- v1-SL-a additions (sponsor-liability sub-phase A) ---------------
//
// 13 new keys seeded by migration `2026-05-03-000000-0000_add_sponsor_liability_grace_window`.
// Authoritative list = PRD §10 defaults matrix. Breakdown:
// 6 grace_window_*_hours (int) + 2 restoration escape (1 bool + 1 int)
// + 1 multi_sponsor_escape_rule (text) + 1 revoke_rate_limit_per_day (int)
// + 3 job.grace_check_* (2 int + 1 float) = 13 keys total.
//
// Per DQ #115 (advisor 2026-05-03): all hour keys are raw integer hours
// (NOT micros-scaled); used as Postgres `INTERVAL '<N> hours'` operands
// in PRD §6.2 scheduler + PRD §8.4 backfill SQL. The
// `feedback_brehon_config_micros_scaled.md` lesson scopes to
// reputation/score-formula math; wall-clock units are out of scope.
//
// Distinct from existing v0 `liability.*` keys (founder_multiplier,
// regular_multiplier, sponsor_liability_floor — lines 795-797). Both
// coexist; SL-a's keys all carry `liability.grace_window_*` /
// `liability.restoration_*` / `liability.multi_sponsor_*` /
// `liability.revoke_*` namespace prefixes per PRD §18 B4 key-rename
// table.

// liability.grace_window_*_hours — 6 keys (int; raw hours per DQ #115)
pub const DEFAULT_LIABILITY_GRACE_WINDOW_MINOR_HOURS: i64 = 24;
pub const DEFAULT_LIABILITY_GRACE_WINDOW_MODERATE_HOURS: i64 = 72;
pub const DEFAULT_LIABILITY_GRACE_WINDOW_SEVERE_HOURS: i64 = 168;
pub const DEFAULT_LIABILITY_GRACE_WINDOW_MINIMUM_HOURS: i64 = 1;
pub const DEFAULT_LIABILITY_GRACE_WINDOW_MAXIMUM_HOURS: i64 = 720;
pub const DEFAULT_LIABILITY_GRACE_WINDOW_ALERT_THRESHOLD_HOURS: i64 = 24;

// liability.restoration_* — 2 keys (1 bool + 1 int)
pub const DEFAULT_LIABILITY_RESTORATION_ESCAPES_LIABILITY: bool = true;
pub const DEFAULT_LIABILITY_RESTORATION_SEVERITY_REDUCTION_STEPS: i64 = 0;

// liability.multi_sponsor_escape_rule — 1 key (text; enum: any_revocation | all_revocation | majority_revocation)
pub const DEFAULT_LIABILITY_MULTI_SPONSOR_ESCAPE_RULE: &str = "any_revocation";

// liability.revoke_rate_limit_per_day — 1 key (int; per-user per-rolling-24h cap)
pub const DEFAULT_LIABILITY_REVOKE_RATE_LIMIT_PER_DAY: i64 = 5;

// job.grace_check_* — 3 keys (2 int + 1 float)
pub const DEFAULT_JOB_GRACE_CHECK_INTERVAL_MINUTES: i64 = 5;
pub const DEFAULT_JOB_GRACE_CHECK_BATCH_SIZE: i64 = 100;
pub const DEFAULT_JOB_GRACE_CHECK_STALENESS_ALERT_MULTIPLIER: f64 = 2.0;
```

**13 new match arms** across `const_default_int` (10 arms),
`const_default_float` (1 arm), `const_default_bool` (1 arm),
`const_default_text` (1 arm). Walk: 6 grace_window int +
restoration_severity_reduction + revoke_rate_limit + 2 job
(interval_minutes + batch_size) = 10 int arms;
restoration_escapes_liability (bool); multi_sponsor_escape_rule
(text); job.grace_check_staleness_alert_multiplier (float). Total
10 + 1 + 1 + 1 = 13.

**13 new SEEDED_KEYS_WITH_CONSTS tuples** (append after the v1-JM-a
block at line 1298, before the closing `];` at line 1299;
alphabetised within the new block by key):

```rust
  // v1-SL-a additions (sponsor-liability sub-phase A — 13 new keys per
  // PRD §10 defaults matrix; flat liability.* + job.grace_check_*
  // namespaces per PRD §18 B4 key-rename table).
  ("job.grace_check_batch_size", "DEFAULT_JOB_GRACE_CHECK_BATCH_SIZE", "int"),
  ("job.grace_check_interval_minutes", "DEFAULT_JOB_GRACE_CHECK_INTERVAL_MINUTES", "int"),
  ("job.grace_check_staleness_alert_multiplier", "DEFAULT_JOB_GRACE_CHECK_STALENESS_ALERT_MULTIPLIER", "float"),
  ("liability.grace_window_alert_threshold_hours", "DEFAULT_LIABILITY_GRACE_WINDOW_ALERT_THRESHOLD_HOURS", "int"),
  ("liability.grace_window_maximum_hours", "DEFAULT_LIABILITY_GRACE_WINDOW_MAXIMUM_HOURS", "int"),
  ("liability.grace_window_minimum_hours", "DEFAULT_LIABILITY_GRACE_WINDOW_MINIMUM_HOURS", "int"),
  ("liability.grace_window_minor_hours", "DEFAULT_LIABILITY_GRACE_WINDOW_MINOR_HOURS", "int"),
  ("liability.grace_window_moderate_hours", "DEFAULT_LIABILITY_GRACE_WINDOW_MODERATE_HOURS", "int"),
  ("liability.grace_window_severe_hours", "DEFAULT_LIABILITY_GRACE_WINDOW_SEVERE_HOURS", "int"),
  ("liability.multi_sponsor_escape_rule", "DEFAULT_LIABILITY_MULTI_SPONSOR_ESCAPE_RULE", "text"),
  ("liability.restoration_escapes_liability", "DEFAULT_LIABILITY_RESTORATION_ESCAPES_LIABILITY", "bool"),
  ("liability.restoration_severity_reduction_steps", "DEFAULT_LIABILITY_RESTORATION_SEVERITY_REDUCTION_STEPS", "int"),
  ("liability.revoke_rate_limit_per_day", "DEFAULT_LIABILITY_REVOKE_RATE_LIMIT_PER_DAY", "int"),
```

**EXPECTED_SEED_COUNT_V1_SL** (insert after line 1322):

```rust
/// v1-SL-a adds 13 sponsor-liability-owned keys to `SEEDED_KEYS_WITH_CONSTS`.
/// Parametric per advisor directive 2026-04-19 #4 — each v1 sub-PRD adds
/// its own `EXPECTED_SEED_COUNT_V1_*` beside the v0 + v1-AD-a + v1-JM-a
/// invariants without churning them. Count is authoritative against
/// on-disk reality: plan §13 Task 6 reconciliation gate asserts
/// `SEEDED_KEYS_WITH_CONSTS` contains exactly this many v1-SL-a-block
/// tuples AND the seed migration has exactly this many INSERT rows.
/// Per DQ #115: all `liability.grace_window_*_hours` keys are raw
/// integer hours (NOT micros-scaled); the lesson
/// `feedback_brehon_config_micros_scaled.md` scopes to reputation/score-
/// formula math, not wall-clock INTERVAL operands.
pub const EXPECTED_SEED_COUNT_V1_SL: usize = 13;
```

**Parity-test extension** at line 2422:

```rust
fn seeded_keys_count_matches_const_count() {
  let expected = EXPECTED_SEED_COUNT
    + EXPECTED_SEED_COUNT_V1_AD
    + EXPECTED_SEED_COUNT_V1_JM
    + EXPECTED_SEED_COUNT_V1_SL;
  assert_eq!(
    SEEDED_KEYS_WITH_CONSTS.len(),
    expected,
    "SEEDED_KEYS_WITH_CONSTS length ({}) must equal EXPECTED_SEED_COUNT ({}) + \
     EXPECTED_SEED_COUNT_V1_AD ({}) + EXPECTED_SEED_COUNT_V1_JM ({}) + \
     EXPECTED_SEED_COUNT_V1_SL ({}) = {} — add/remove keys in both places \
     when changing the seed list",
    SEEDED_KEYS_WITH_CONSTS.len(),
    EXPECTED_SEED_COUNT,
    EXPECTED_SEED_COUNT_V1_AD,
    EXPECTED_SEED_COUNT_V1_JM,
    EXPECTED_SEED_COUNT_V1_SL,
    expected,
  );
}
```

**13 new CONFIG_KEY_METADATA entries** (insert before the closing
`];` of the array at line 2414; mirror the v1-JM-a entries at lines
2200-2412 for shape; per NOT4 resolution + v1-AD-a §3.2 precedent).
Each entry's `apply_at_default`:

- `liability.grace_window_*_hours` -> `ApplyAt::Immediate`
  (config-read live at SL-d's `Decided -> SponsorLiabilityPending`
  transition per PRD §4.3; locked at transition time per row).
- `liability.restoration_*` -> `ApplyAt::Immediate` (community policy
  read live by SL-c's escape-evaluator).
- `liability.multi_sponsor_escape_rule` -> `ApplyAt::Immediate`.
- `liability.revoke_rate_limit_per_day` -> `ApplyAt::Immediate`.
- `job.grace_check_interval_minutes` -> `ApplyAt::Immediate` BUT with
  doc-comment caveat per PRD §6.4: clokwerk schedules pin at
  scheduler `setup()`, so config flip takes effect at next server
  restart.
- `job.grace_check_batch_size` -> `ApplyAt::Immediate`.
- `job.grace_check_staleness_alert_multiplier` -> `ApplyAt::Immediate`.

`scope`: `ConfigScope::Both` for all 10 `liability.*` keys EXCEPT
`liability.grace_window_minimum_hours`, `_maximum_hours`,
`_alert_threshold_hours`, `_revoke_rate_limit_per_day` which are
`ConfigScope::Instance` (instance-only per PRD §4.2 + §12.1).
`job.*` keys: `ConfigScope::Instance` (per PRD §6.4).

`requires_re_jury: false`, `requires_step_up: false` for all 13.

`valid_range`: per PRD §10 defaults-matrix `Range` column —
hour-keys typically `1.0..=720.0`, `0.0..=3.0` for restoration steps,
`1.0..=100.0` for revoke rate-limit, `1.0..=60.0` for grace_check
interval, `1.0..=10000.0` for batch_size, `1.0..=10.0` for staleness
multiplier.

`valid_enum`: only `liability.multi_sponsor_escape_rule` carries one
(`&["any_revocation", "all_revocation", "majority_revocation"]`).

`description`: one-sentence operator-facing summary per key.

`doc_anchor`: `"v1-sponsor-liability.prd.md§10"` for all 13 entries.

### 10.7 ENTRY_KIND const dual-file edit (Task 7)

**Mirror:** v1-JM-a precedent at
`.claude/PRPs/plans/phase-v1-JM-a.plan.md` Task 9 (six new ENTRY_KIND
consts dual-file edit + registry section population).

**Five new consts in `crates/db_schema/src/source/governance/governance_log.rs`**
(insert AFTER the v1-JM-a block at line 185):

```rust
// v1-SL-a additions (v1 sponsor-liability sub-phase A). All five
// emitting call sites land in v1-SL-b/c/d + restorative-mechanics-v1
// per the registry rule's pre-landed-const exemption. v1-SL-a writes
// the const declarations only; actual governance_log::append calls
// land with the handler edits in later sub-phases (b: endorsement_revoked
// + sponsor_liability_escaped (revocation branch); c: sponsor_liability_fired
// + sponsor_liability_escaped (scheduler escape branch); d:
// sponsor_liability_pending; restorative-mechanics-v1: restoration_completed).
pub const ENTRY_KIND_ENDORSEMENT_REVOKED: &str = "endorsement_revoked";
pub const ENTRY_KIND_RESTORATION_COMPLETED: &str = "restoration_completed";
pub const ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED: &str = "sponsor_liability_escaped";
pub const ENTRY_KIND_SPONSOR_LIABILITY_FIRED: &str = "sponsor_liability_fired";
pub const ENTRY_KIND_SPONSOR_LIABILITY_PENDING: &str = "sponsor_liability_pending";
```

**Five new re-exports in `crates/api/api/src/governance/governance_log.rs`**
(insert ALPHABETICALLY into the existing `pub use` block at lines
39-76):

- `ENTRY_KIND_ENDORSEMENT_REVOKED` between `ENTRY_KIND_ENDORSEMENT_CREATED`
  (line 50) and `ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED` (line 51).
- `ENTRY_KIND_RESTORATION_COMPLETED` between
  `ENTRY_KIND_REPUTATION_DELTA` (line 66) and
  `ENTRY_KIND_RULE_SET_VERSION_CREATED` (line 67).
- `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` between
  `ENTRY_KIND_SPONSOR_LIABILITY_CLAMPED` (line 71) and
  `ENTRY_KIND_SPONSOR_LIABILITY_FIRED` (new).
- `ENTRY_KIND_SPONSOR_LIABILITY_FIRED` after `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED`
  (new), before `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` (new).
- `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` after `ENTRY_KIND_SPONSOR_LIABILITY_FIRED`
  (new), before `ENTRY_KIND_THRESHOLD_MET` (line 72).

**Registry section population** (replace the
`### sponsor-liability-v1 (reserved — §17 of PRD enumerates 5 new kinds)`
stub at lines 160-165 of `.claude/rules/governance-log-entry-kind-registry.md`
with a populated section matching the v1-JM-a section shape at lines
128-143):

```markdown
## v1-SL-a entry kinds (5, this sub-phase)

Landed alongside task 7's dual-file edit. v1-SL-a writes the const
declarations only; emitting call sites land in v1-SL-b/c/d +
restorative-mechanics-v1 per the registry rule's pre-landed-const
exemption (each pending row names a specific downstream plan).

| Rust const | `&str` value | Source | Emitting handler | Semantic |
|---|---|---|---|---|
| `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` | `sponsor_liability_pending` | v1-SL-a const; v1-SL-d call site | v1-SL-d `crates/api/api/src/governance/submit_jury_vote.rs::process_vote` Decided->SponsorLiabilityPending transition (pending) | Case transitioned to grace-window state at jury-decision time per PRD §9.3 step 1. Payload: `{case_id, target_person_id, severity, grace_expires_at, sponsors_pseudonyms}`. Replaces v0 immediate-fire path on cases with active sureties. |
| `ENTRY_KIND_SPONSOR_LIABILITY_FIRED` | `sponsor_liability_fired` | v1-SL-a const; v1-SL-c call site | v1-SL-c `crates/api/api/src/governance/sponsor_liability_grace.rs::run_grace_check_batch` fire branch (pending) | Grace window expired without escape; the v0 `apply_sponsor_liability` ran and `reputation_event` rows for sponsors were written. Payload: `{case_id, fired_at, sponsor_count, deltas: [{sponsor_pseudonym, delta}, ...]}`. Per PRD §6.2 step 5. |
| `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` | `sponsor_liability_escaped` | v1-SL-a const; v1-SL-b + v1-SL-c call sites | v1-SL-b `crates/api/api/src/governance/revoke_endorsement.rs` escape branch (pending) AND v1-SL-c `sponsor_liability_grace.rs::evaluate_escape_conditions` (pending) | Sponsor revocation OR defendant restoration severed the liability chain during grace window; case transitioned to terminal `SponsorLiabilityEscaped`; no `reputation_event` rows for sponsors. Payload mirrors `liability_escape_reason` JSONB column: `{case_id, escaped_at, reason, actor_pseudonym, endorsement_id\|restoration_id}`. Per PRD §5.3 step 4 + §6.2 step 4. |
| `ENTRY_KIND_ENDORSEMENT_REVOKED` | `endorsement_revoked` | v1-SL-a const; v1-SL-b call site | v1-SL-b `crates/api/api_crud/src/governance/revoke_endorsement.rs` (pending) | Endorsement revocation succeeded (always emitted, even when no grace-window severance occurred). Payload: `{endorsement_id, revoker_pseudonym, revoked_at, sponsored_id, reason, liability_chain_severed_for_cases: [<case_ids>]}`. Per PRD §5.3 step 5. |
| `ENTRY_KIND_RESTORATION_COMPLETED` | `restoration_completed` | v1-SL-a const; restorative-mechanics-v1 call site | restorative-mechanics-v1 PRD `crates/api/api_crud/src/governance/restoration_complete.rs` (pending — owned by separate PRD) | Defendant marked restoration complete + admin attested. Payload: `{restoration_id, defendant_pseudonym, attestor_pseudonym, completed_at, sanction_id}`. SL-a declares the const here for `governance_log.rs` const-discipline (per PRD §17 cross-cutting impact); the actual emitter ships in restorative-mechanics-v1. Cross-PRD coordination: sponsor-liability owns ESCAPE semantics (§7.3); restorative-mechanics-v1 owns the COMPLETION mechanism. |
```

**Registry "Acceptance invariants" section update**: the count claim
at the bottom of the registry must update from `33` to `38`. Per the
registry rule's "Acceptance invariants" section: the `rg -c
'^pub const ENTRY_KIND_'` invariant returns 38; shim re-export count
returns 38; uniqueness check returns empty.

### 10.8 e2e migration round-trip extension (Task 8)

**Mirror:** the v1-JM-a Task 10 precedent
(`.claude/PRPs/plans/phase-v1-JM-a.plan.md` lines 1240-1322).

**Sub-edits:**

1. Find `PHASE_1_MIGRATION_COUNT` declaration. Impl-task verifies
   current value at task start. Bump by +1.
2. Extend the post-condition probe lists in `phase1_migrations_round_trip`
   to assert SL-a's effects:
   - Two new column names appear in `moderation_case` after up.sql:
     `grace_expires_at`, `liability_escape_reason`.
   - Two new index names exist in `pg_indexes`:
     `moderation_case_grace_expires_idx`, `surety_sponsored_id_active`.
   - Three new enum values exist in `pg_enum` for `case_status`:
     `SponsorLiabilityPending`, `SponsorLiabilityFired`,
     `SponsorLiabilityEscaped`.
   - `governance_config` row count increases by 13 after up.sql vs
     pre-up.sql (per JM-a precedent on AD-a's 27-row seed extension).
3. After down.sql:
   - Two new column names absent from `moderation_case`.
   - Two new index names absent from `pg_indexes`.
   - Three Postgres enum values REMAIN (Postgres limitation per PRD
     §3.4 down.sql doc-comment) — this is asserted with a doc-comment
     in the test explaining the expected residual.
   - `governance_config` row count restored to pre-up.sql baseline.

**No new test fns** — extending the existing
`phase1_migrations_round_trip` test only. Per
`feedback_junior_worker_e2e_edit_hang.md`: single Edit-with-anchor
restricted to one block.

---

## 11. Files to change

### `lemmy_db_schema_file` crate

- `crates/db_schema_file/src/enums.rs` — extend `CaseStatus` enum with
  three new variants. **Task 2**.
- `crates/db_schema_file/src/schema.rs` — extend `moderation_case`
  table block with two new columns. **Task 3**.

### `lemmy_db_schema` crate

- `crates/db_schema/src/source/governance/moderation_case.rs` — extend
  `ModerationCase` + `ModerationCaseInsertForm` with two new fields
  (both `Option<_>`). **Task 4**.
- `crates/db_schema/src/source/governance/governance_log.rs` — five
  new `ENTRY_KIND_*` const declarations (DEFINE side). **Task 7**.

### `lemmy_api` crate

- `crates/api/api/src/governance/governance_log.rs` — five new
  alphabetical `pub use` re-export lines. **Task 7**.
- `crates/api/api/src/governance/config.rs` — 13 new DEFAULT_*
  declarations + 13 new match arms across 4 const_default_*
  functions + 13 new SEEDED_KEYS_WITH_CONSTS tuples + new
  EXPECTED_SEED_COUNT_V1_SL constant + parity-test extension + 13
  new CONFIG_KEY_METADATA entries. **Task 6**.
- `crates/api/api/src/governance/admin_close_case.rs` — extend
  `case.status` match with three new variants in the allowed-set.
  **Task 5**.
- `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs` —
  extend `case.status` match with three new variants in the rejection
  set. **Task 5**.
- `crates/api/api/src/governance/accept_jury_assignment.rs` — extend
  both nested `case.status` matches (Original branch + Appeal branch)
  with three new variants. **Task 5**.
- `crates/api/api/src/governance/admin_assign_jury.rs` — extend
  `case.status` match with three new variants. **Task 5**.
- `crates/api/api/src/governance/submit_jury_vote.rs` — extend the
  `process_vote` step-5 idempotency `matches!()` guard at lines
  271-276 with three new variants. **Task 5**.

### `lemmy_api_crud` crate

- `crates/api/api_crud/src/governance/request_appeal.rs` — extend
  `case.status` match with `SponsorLiabilityPending` in allowed-set
  + `SponsorLiabilityFired \| SponsorLiabilityEscaped` in rejection
  set. **Task 5**.

### `lemmy_server` crate

- `crates/server/tests/e2e.rs` — extend `PHASE_1_MIGRATION_COUNT` +
  post-condition probe lists in `phase1_migrations_round_trip`.
  **Task 8**.

### Migration files

- `migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql`
  + `down.sql` — combined enum-additions + column-additions + indexes
  + 13 seed rows + backfill UPDATE. **Task 1**.

### Shell scripts

- `scripts/brehon/migrate-roundtrip.sh` — replace the stub body with
  real round-trip logic per DQ #114. **Task 0**.

### Meta files (rules + reports)

- `.claude/rules/governance-log-entry-kind-registry.md` — populate
  the `sponsor-liability-v1 (reserved)` section with 5 rows + update
  the Acceptance invariants count from 33 to 38. **Task 7**.
- `.claude/PRPs/reports/v1-SL-a-retro.md` — CREATE retro per
  `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`
  + `feedback_retro_task_complexity_score.md`. **Task 9**.

### Files explicitly NOT touched

- `crates/api/api/src/governance/sponsor_liability.rs` — v0 helper
  stays intact through SL-a; SL-d is the rewrite.
- `crates/api/api/src/governance/submit_jury_vote.rs` past line 276 —
  the SL-d-graft TODO at line 455-465 stays as-is.
- `crates/api/api/src/governance/sponsor_liability_grace.rs` — does
  not exist yet. SL-c creates.
- `crates/api/api_crud/src/governance/revoke_endorsement.rs` — does
  not exist yet. SL-b creates.
- `crates/api/routes/src/lib.rs` — no new routes.
- `crates/routes/src/utils/scheduled_tasks.rs` — no new scheduler
  wiring.
- `crates/db_views/governance_case/src/impls.rs` — uses
  `let open_statuses = [Open, ThresholdMet, JurySelection, InReview]`
  (an array, not an exhaustive match); new SponsorLiability* variants
  are correctly excluded by design (post-decision states). No edit.
- `crates/db_views/jury_queue/src/impls.rs` — filters
  `status.eq(CaseStatus::ThresholdMet)`; not exhaustive; no edit.
- `migrations/**` other than the new SL-a directory — no edits.
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.coderabbit.yaml`
  — no dep / build-config / review-config changes.
- `.github/workflows/**` — no workflow YAML changes.

---

## 12. NOT building in v1-SL-a

- **`revoke_endorsement` handler + DTO + route** — SL-b's.
- **Scheduler module `sponsor_liability_grace.rs` + clokwerk wiring**
  — SL-c's.
- **`submit_jury_vote` mutation: compute/fire split** — SL-d's. The v0
  `apply_sponsor_liability` at `sponsor_liability.rs:142` keeps its v0
  shape through SL-a.
- **`restoration/complete` endpoint** — restorative-mechanics-v1 PRD's.
  SL-a declares the `RESTORATION_COMPLETED` const for `governance_log.rs`
  const-discipline; the endpoint ships separately.
- **e2e behavioural tests** (revocation-during-window-escapes, etc) —
  SL-e's. SL-a's only e2e edit is extending `phase1_migrations_round_trip`.
- **Sponsor notification UX** — out per PRD §13 OQ-V1-SL-03.
- **Step-up auth enforcement** — out per PRD §12.3 (v2 reservation).
- **Dashboard write surface for grace-window keys** — admin-dashboard-v1
  PRD's owns. v0 admins edit via direct `psql` until that ships.
- **Cross-instance sponsor-liability federation** — out per PRD §2 OUT
  + ADR-014.
- **Sponsor-of-sponsor liability chain depth** — kept 1-deep per PRD
  §13 OQ-V1-SL-02.
- **Backfill SMOKE TEST in e2e.rs** — JM-a included one
  (`v1_jm_a_backfill_populates_v0_snapshot`); SL-a defers to SL-e
  where actual scheduler logic exists.
- **`down.sql` Postgres enum-value drop** — Postgres limitation; the
  three new variants stay as orphans. Documented in down.sql.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task** (per
`feedback_pr_per_phase.md`'s code-only-via-PR rule). Each task header
carries a `[P]` marker iff its **FILES** YAML
`union(creates, modifies)` shares no path with any other `[P]`-marked
task in the same cohort. Task 0 (pre-flight harness audit) is
**always** non-`[P]`.

> **Cohort dispatch (advisor-side):** Cohort A is Tasks 1+2+3+6+7
> (5-way parallel); Cohort B is Tasks 4+5 (2-way parallel); Tasks 0,
> 8, 9 are barriers.

> **Shape G (Layer G2 push-and-exit):** §13 task bodies do NOT inline
> cargo invocations. Each task ends with a push to the worker branch;
> the impl-task subagent writes a `kind: "validate-pending"` DQ entry
> referencing `cargo-validate-workspace.yml`. Migration-touching tasks
> additionally trigger `cargo-validate-migration.yml`.

### Task 0: Pre-flight harness audit + branch verification + migrate-roundtrip.sh stub fix

**Goal:** verify environment + branch (`phase-v1-SL-a`) +
governance-v0 baseline state intact + replace migrate-roundtrip.sh
stub body with real round-trip logic per DQ #114.

**FILES:**

```yaml
creates: []
modifies:
  - scripts/brehon/migrate-roundtrip.sh   # replace stub body per DQ #114
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes):**

```bash
# Probe 0 — Docker daemon (e2e harness uses testcontainers-rs)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe -1 — submodule init (Linux/Junior worktree quirk)
git submodule status > /tmp/sl-a-task0-submodule.log 2>&1
if grep -q '^-' /tmp/sl-a-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/sl-a-task0-submodule-init.log 2>&1
  echo "submodule init exit: $?"
fi

# Probe 1 — branch verification
git branch --show-current
# EXPECT: phase-v1-SL-a (BM-task cuts before Task 1)

# Probe 2 — governance-v0 baseline counts
echo "ENTRY_KIND_ count (expect 33):"
grep -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs

echo "EXPECTED_SEED_COUNT total (expect 88 = 34 v0 + 27 AD + 27 JM):"
grep -nE '^pub const EXPECTED_SEED_COUNT' crates/api/api/src/governance/config.rs

echo "CaseStatus variant count (expect 9):"
sed -n '/^pub enum CaseStatus/,/^}/p' crates/db_schema_file/src/enums.rs | grep -cE '^\s*[A-Z][a-zA-Z]*,?$|#\[default\]'

# Probe 3 — schema baseline: moderation_case has no SL-a columns
grep -E 'grace_expires_at|liability_escape_reason' crates/db_schema_file/src/schema.rs && {
  echo "ERROR: SL-a columns already in schema.rs — branch contamination"
  exit 1
} || echo "schema.rs clean of SL-a columns"

# Probe 4 — ADR-013 match-site enumeration (live grep for §10.5 site list)
grep -rnE 'match\s+\w+\.status\s*\{' crates/api crates/api_crud --include='*.rs' > /tmp/sl-a-task0-cs-sites.log 2>&1
echo "match sites found:"
cat /tmp/sl-a-task0-cs-sites.log
# EXPECT: 6 sites matching §10.5. If new sites have appeared since
#         brief-write time, file a DQ pending entry naming the new
#         site + recommend including in Task 5.

# Probe 5 — Postgres-side: case_status enum has 9 values
echo "case_status enum values (expect 9 currently):"
grep -B1 -A20 "CREATE TYPE case_status" migrations/2026-04-15-100000-0000_add_governance_enums/up.sql 2>/dev/null | head -15

# Probe 6 — DQ pending status (advisory)
python3 -c "import json; d=json.load(open('.claude/decision-queue.json')); print('pending:', [(e['id'], e.get('kind')) for e in d.get('pending',[])])"

# Probe 7 — PM-plugin-hooks-stable check
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  grep -rqE "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done

# Probe 8 — concurrent-PR check
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("moderation_case\\.rs|enums\\.rs|governance_log\\.rs|config\\.rs|migrations/2026-05-03")) | {number, title, headRefName}'
# EXPECT: empty output

# Probe 9 — Shape G workflow YAMLs lint-clean + carry the right flags
yamllint .github/workflows/cargo-validate-workspace.yml \
         .github/workflows/cargo-validate-migration.yml \
         .github/workflows/cargo-test-e2e.yml > /tmp/sl-a-task0-yamllint.log 2>&1
echo "yamllint exit: $?"
grep -E '\-\-features full|\-\-no-deps|\-D warnings' .github/workflows/cargo-validate-workspace.yml
# EXPECT: --workspace --features full + --no-deps -- -D warnings present

# Probe 10 — migrate-roundtrip.sh stub status (per DQ #114)
grep -q "migrate-roundtrip.sh: no migrations/ change detected vs governance-v0; stub exiting 0" \
  scripts/brehon/migrate-roundtrip.sh && echo "STUB STILL PRESENT — Task 0 must replace" || echo "STUB ALREADY FIXED — verify via dry-run before continuing"
```

**migrate-roundtrip.sh real implementation (replaces lines 32-58 of
the existing stub; preserves header doc-comment + the
`origin/governance-v0` fetch-verify guard at lines 33-44):**

```bash
# After the existing fetch-verify guard:

# Identify any new migration directories vs governance-v0.
NEW_MIGRATIONS=$(git diff --name-only --diff-filter=A origin/governance-v0...HEAD -- 'migrations/*/up.sql' | xargs -I{} dirname {} 2>/dev/null | sort -u)
if [ -z "$NEW_MIGRATIONS" ]; then
    echo "migrate-roundtrip.sh: no new migrations vs governance-v0; exit 0."
    exit 0
fi

echo "migrate-roundtrip.sh: detected new migrations:"
echo "$NEW_MIGRATIONS"

# Spin up an ephemeral Postgres container.
PG_PORT=$(shuf -i 30000-39999 -n 1)
PG_CONTAINER=$(docker run -d --rm \
    -e POSTGRES_PASSWORD=ci-roundtrip-throwaway \
    -e POSTGRES_DB=lemmy_roundtrip \
    -p ${PG_PORT}:5432 \
    pgautoupgrade/pgautoupgrade:18-alpine)
trap 'docker stop $PG_CONTAINER >/dev/null 2>&1 || true' EXIT

# Wait for Postgres ready
for i in $(seq 1 30); do
    if docker exec "$PG_CONTAINER" pg_isready -U postgres >/dev/null 2>&1; then break; fi
    sleep 1
done

export DATABASE_URL="postgres://postgres:ci-roundtrip-throwaway@localhost:${PG_PORT}/lemmy_roundtrip"

# 1. Apply ALL migrations (forward, including the new ones)
cargo run -p lemmy_diesel_utils --features full --bin lemmy_diesel_utils -- run "$DATABASE_URL"
echo "forward exit: $?"

# 2. Revert the new migrations LIFO
for migration_dir in $(echo "$NEW_MIGRATIONS" | tac); do
    migration_name=$(basename "$migration_dir")
    cargo run -p lemmy_diesel_utils --features full --bin lemmy_diesel_utils -- revert "$DATABASE_URL"
    echo "revert $migration_name exit: $?"
done

# 3. Re-apply forward (idempotency check)
cargo run -p lemmy_diesel_utils --features full --bin lemmy_diesel_utils -- run "$DATABASE_URL"
echo "re-forward exit: $?"

echo "migrate-roundtrip.sh: round-trip complete for $(echo "$NEW_MIGRATIONS" | wc -l) new migration(s)."
exit 0
```

**GOTCHA:** if `lemmy_diesel_utils` does NOT have a `revert` binary
sub-command (impl-task verifies at task start by reading
`crates/diesel_utils/src/main.rs` or equivalent), the script falls
back to `cargo run -p lemmy_diesel_utils --features full -- redo`
or replays from a clean container per the current diesel_utils CLI
surface. **Impl-task: confirm the actual binary sub-command shape
before committing**; if unclear, file a DQ pending entry rather than
guess.

**GOTCHA (per `feedback_lemmy_migration_runner.md`):** the
`forbid_diesel_cli` trigger landed upstream — raw
`diesel migration run` will fail because diesel_cli doesn't take the
`pg_advisory_lock(0)`. Always use `cargo run -p lemmy_diesel_utils ...`
which DOES take the lock per
`crates/diesel_utils/src/schema_setup/mod.rs:214`.

**EXPECT:** Probes 0, 1, 2, 3, 4, 5, 7, 8, 9 exit 0; Probe 6 informational;
Probe 10 confirms the stub status (FIXED after Task 0 replaces).

**COMMIT MESSAGE:** `chore(scripts): replace migrate-roundtrip.sh stub with real round-trip logic (DQ #114; v1-SL-a task 0)`

### Task 1 [P]: CREATE migration directory `2026-05-03-000000-0000_add_sponsor_liability_grace_window/{up,down}.sql`

**ACTION:** create the combined migration (per PRD §8.5 ordering):
enum additions + column additions + indexes + 13 seed rows + backfill
UPDATE in up.sql; LIFO down.sql.

**FILES:**

```yaml
creates:
  - migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql
  - migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/down.sql
modifies: []
```

**IMPLEMENT (file 1 of 2):** in
`migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql`,
write the full SQL per §10.1 up.sql skeleton (verbatim, including the
ADR-exception-trail header comment, all 13 seed rows alphabetised
within the INSERT, and the bounded backfill UPDATE).

**IMPLEMENT (file 2 of 2):** in
`migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/down.sql`,
write the LIFO reverse per §10.1 down.sql skeleton (UPDATE reverse +
DELETE seeds + DROP indexes + ALTER TABLE DROP COLUMN; orphan-enum-
value doc-comment).

**MIRROR:** §10.1;
`migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql`
(idempotent ON CONFLICT seed pattern);
`migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql`
(ADR exception trail header + ADD COLUMN + backfill UPDATE pattern).

**GOTCHA:** the migration directory MUST sort lexicographically AFTER
`2026-04-27-000100-0000_add_appeals_v1_columns` (most recent on
governance-v0 at brief-write time). `2026-05-03-000000-0000` satisfies
this. Impl-task: re-verify at task start by `ls migrations/ | tail -3`;
if a new migration has landed between brief-write and impl, pick
`2026-05-03-000100-0000` instead and document.

**GOTCHA (per `feedback_lemmy_migration_runner.md`):** the
`-- no-transaction` directive on line 1 (after the header comment) is
REQUIRED because Postgres `ALTER TYPE ... ADD VALUE` cannot run inside
a transaction. Diesel's runner detects the directive and runs the
migration outside its default BEGIN/COMMIT.

**GOTCHA (per DQ #115 — micros vs raw hours):** all 6
`liability.grace_window_*_hours` keys carry raw integer hours
(24/72/168/1/720/24); these are consumed as `INTERVAL '<N> hours'` in
the backfill UPDATE + the future SL-c scheduler. Bool keys
(`liability.restoration_escapes_liability`) and enum keys
(`liability.multi_sponsor_escape_rule`) are likewise non-micros.

**GOTCHA (PRD §8.4 backfill performance per PRD §8.4 performance
note):** the three nested EXISTS guards are load-bearing (surety
presence, sanction presence, no-prior-fire) — do not collapse into a
single JOIN at SL-a time; the load is bounded by the
`decided_at > now() - INTERVAL '24 hours'` driving filter. SL-c can
optimise the shape later if production EXPLAIN ANALYZE reveals seq-
scan or >1s runtime; SL-a ships the conservative, correct shape.

**GOTCHA (per OQ-V1-SL-05 — JSONB versioning):** the
`liability_escape_reason` column COMMENT documents
`{"version": 1, ...}` from day one for forward-compat. SL-a does NOT
write to the column (no rows have it set yet at SL-a ship); SL-b/c/d
write and the column comment is the authoritative schema reference.

**GOTCHA (Issue #24 fold-in):** the
`surety_sponsored_id_active` index is folded into THIS migration to
avoid a second migration churn — per PR #10 CodeRabbit finding GH
#24 itself raised. The index targets
`(sponsored_id, sponsor_id) WHERE revoked_at IS NULL`, which matches
both `jury_common::select_eligible_jurors` (EXISTS subquery filter)
and `apply_sponsor_liability` (sponsor enumeration filter) in the
existing v0 code paths.

**Push and exit (Shape G):** push to `junior/<task-slug>`. impl-task
subagent captures BOTH workflow_run_ids:
- `cargo-validate-workspace.yml` (triggers on `crates/**` /
  `migrations/**` push to `junior/*`)
- `cargo-validate-migration.yml` (triggers on `migrations/**` push to
  `junior/*`)

Write TWO `kind: "validate-pending"` DQ entries (one per workflow
run id) per `.claude/rules/decision-queue.md` Recipe 1; commit + push
the DQ entries to `junior/<task-slug>` per the mid-task visibility
rule.

**COMMIT MESSAGE:** `feat(v1-SL-a): combined migration — case_status enum + grace columns + indexes + 13 config seeds + v0 mid-flight backfill (task 1)`

### Task 2 [P]: UPDATE `crates/db_schema_file/src/enums.rs` — extend `CaseStatus` with three new variants

**ACTION:** append three new variants to the `CaseStatus` enum
declaration after `AdminReview`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema_file/src/enums.rs
```

**IMPLEMENT (file 1 of 1):** in
`crates/db_schema_file/src/enums.rs:393-408`, after line 407
(`AdminReview,`) and before line 408 (`}`), insert the three new
variants per §10.2 (verbatim doc-comments included).

**MIRROR:** §10.2; existing `CaseStatus` declaration at lines
382-408 (DbEnum derive, ExistingTypePath, DbValueStyle = "verbatim").

**IMPORTS:** existing imports unchanged.

**GOTCHA:** each new variant gets its own `///` doc-comment citing
PRD §3.1 + the OQ-025 origin.

**GOTCHA:** PascalCase variant names match the Postgres-side string
values (`'SponsorLiabilityPending'`, etc) per
`DbValueStyle = "verbatim"` at line 389.

**GOTCHA:** no `#[default]` attribute on the new variants (the
existing `Open` default at line 394 stays correct — new cases never
start in a sponsor-liability state).

**Push and exit (Shape G):** push to `junior/<task-slug>`; one
`kind: "validate-pending"` DQ entry referencing
`cargo-validate-workspace.yml`. No migration trigger (Rust-only).

**COMMIT MESSAGE:** `feat(v1-SL-a): add 3 CaseStatus variants — SponsorLiabilityPending/Fired/Escaped (task 2)`

### Task 3 [P]: UPDATE `crates/db_schema_file/src/schema.rs` — add 2 columns to `moderation_case` table block

**ACTION:** append two new column declarations to the `moderation_case
(id)` table block after `winning_decision`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema_file/src/schema.rs
```

**IMPLEMENT (file 1 of 1):** in
`crates/db_schema_file/src/schema.rs:763-799`, after line 797
(`winning_decision -> Nullable<JuryDecision>,`) and before line 798
(`}`), insert two new column declarations per §10.3:

```
        grace_expires_at -> Nullable<Timestamptz>,
        liability_escape_reason -> Nullable<Jsonb>,
```

**MIRROR:** §10.3; existing `moderation_case (id)` block at lines
763-799 (use of `Nullable<Timestamptz>` for `decided_at`, etc).

**GOTCHA:** the `use diesel::sql_types::*` at line 764 already covers
`Timestamptz` and `Jsonb`. No additional `use super::sql_types::...`
needed.

**GOTCHA:** schema.rs is `@generated`-style hand-edited per project
convention — add a `// v1-SL-a additions:` inline comment marker
before the new lines.

**GOTCHA:** the two new partial indexes are NOT declared here —
Diesel's `table!` macro only tracks columns + foreign keys.
Postgres-side indexes exist in `pg_indexes` after the migration applies.

**GOTCHA:** `surety` table block at lines 1343-1352 needs NO edit —
SL-a only adds an index, not a column.

**Push and exit (Shape G).**

**COMMIT MESSAGE:** `feat(v1-SL-a): extend moderation_case table block with grace_expires_at + liability_escape_reason columns (task 3)`

### Task 4: UPDATE `crates/db_schema/src/source/governance/moderation_case.rs` — extend `ModerationCase` + InsertForm

**ACTION:** add two new fields to both the `ModerationCase` struct
and the `ModerationCaseInsertForm` struct.

**Cohort B member.** Depends on Task 3 (schema.rs columns must exist
for `diesel(check_for_backend(diesel::pg::Pg))` to type-check).

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema/src/source/governance/moderation_case.rs
```

**IMPLEMENT (file 1 of 1):** in
`crates/db_schema/src/source/governance/moderation_case.rs`:

1. After `winning_decision` field (line 83) in `pub struct ModerationCase`,
   insert the two new fields per §10.4 (verbatim doc-comments).
2. After `winning_decision` field (line 122) in
   `pub struct ModerationCaseInsertForm`, insert the two new fields
   per §10.4 (verbatim doc-comments). Both `Option<_>` so v0 callers
   continue to compile via `..Default::default()`.

**MIRROR:** §10.4; existing v1-AD-a + v1-JM-a additions at lines
38-83 (struct field doc-comment style + `Option<_>` propagation).

**IMPORTS:** existing imports cover both new field types
(`use chrono::{DateTime, Utc}` at line 2; `use serde_json::Value` at
line 10). No additions.

**GOTCHA (per `feedback_insertform_default_propagation.md`):** both
new fields are `Option<_>` so v0 callers continue to compile via
`..Default::default()`. Pre-impl grep:
`git grep -l 'ModerationCaseInsertForm {'` — verify every literal-
construction site uses `..Default::default()` spread; if any site
enumerates fields exhaustively, add the two new fields to that site
too (R3 pattern).

**GOTCHA:** the `AsChangeset` derive on `ModerationCaseInsertForm`
(line 87) means SL-d's future `update().set(...)` calls will be able
to write the new columns without a separate `AsChangeset` struct.

**GOTCHA:** the `#[skip_serializing_none]` attribute on
`ModerationCase` (line 13) ensures both new `Option<_>` fields are
omitted from JSON serialisation when None.

**GOTCHA (R7):** struct-extension affects compile across re-exports;
the workspace-check workflow's `cargo test --no-run -p lemmy_server
--test e2e` step (`cargo-validate-workspace.yml:95`) picks up failures.

**Push and exit (Shape G).**

**COMMIT MESSAGE:** `feat(v1-SL-a): extend ModerationCase + InsertForm with grace_expires_at + liability_escape_reason fields (task 4)`

### Task 5: UPDATE 6 ADR-013 enum-exhaustiveness match sites — handle 3 new CaseStatus variants

**ACTION:** Per §10.5 per-site decision matrix, update each of 6
files to handle `SponsorLiabilityPending`, `SponsorLiabilityFired`,
`SponsorLiabilityEscaped` explicitly (no `_ =>` arms).

**Cohort B member.** Depends on Task 2 (Rust enum variants must exist
for the new arm names to compile).

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api_crud/src/governance/request_appeal.rs
  - crates/api/api/src/governance/admin_close_case.rs
  - crates/api/api/src/governance/admin_trigger_appeal_rejury.rs
  - crates/api/api/src/governance/accept_jury_assignment.rs
  - crates/api/api/src/governance/admin_assign_jury.rs
  - crates/api/api/src/governance/submit_jury_vote.rs
```

**IMPLEMENT (file 1 of 6):** in
`crates/api/api_crud/src/governance/request_appeal.rs:94-103`, extend
the `match case.status { ... }` block per §10.5 row 1. Add
`CaseStatus::SponsorLiabilityPending => {}` to the allowed-set
(co-located with `Decided => {}`) AND
`CaseStatus::SponsorLiabilityFired | CaseStatus::SponsorLiabilityEscaped`
to the rejection arm pattern. Doc-comment cites PRD §3.3 row 1 +
ADR-013.

**IMPLEMENT (file 2 of 6):** in
`crates/api/api/src/governance/admin_close_case.rs:65-75`, extend the
match per §10.5 row 2. Add all three new variants to the allowed-set
pattern (admins may force-close terminal liability states for ops
purposes).

**IMPLEMENT (file 3 of 6):** in
`crates/api/api/src/governance/admin_trigger_appeal_rejury.rs:72-81`,
extend per §10.5 row 3. Add all three new variants to the rejection
set pattern.

**IMPLEMENT (file 4 of 6):** in
`crates/api/api/src/governance/accept_jury_assignment.rs:111-134`,
extend BOTH nested matches per §10.5 rows 4 + 5. Both branches
(Original + Appeal) reject all three new variants.

**IMPLEMENT (file 5 of 6):** in
`crates/api/api/src/governance/admin_assign_jury.rs:140-147`, extend
per §10.5 row 6. Add all three new variants to the rejection set
pattern.

**IMPLEMENT (file 6 of 6):** in
`crates/api/api/src/governance/submit_jury_vote.rs:271-276`, extend
the `process_vote` step-5 idempotency `matches!()` guard per §10.5
row 7. Add `| CaseStatus::SponsorLiabilityPending |
CaseStatus::SponsorLiabilityFired | CaseStatus::SponsorLiabilityEscaped`
to the existing list. The `process_appeal_vote` terminal-guard at
line 729-732 stays UNCHANGED per §10.5 row 8 rationale.

**MIRROR:** §10.5; the v1-JM-a precedent for `JuryAssignmentRole::Appeal`
extension at `accept_jury_assignment.rs:111-134`.

**GOTCHA (R3 — struct extension grep sweep):** after Task 5 commits,
grep for any pattern like `if matches!(.*\.status, CaseStatus::` or
`match \w+\.status` not yet captured by §10.5 — if a new site has
appeared on governance-v0 since brief-write time, fix it in the SAME
commit.

**GOTCHA (per `feedback_clippy_test_style.md`):** all match arms use
explicit `| Pattern` enumeration — never `_ => ...`. Workspace clippy
denies `_ =>` on `match` over enum types where exhaustive-match is
feasible.

**GOTCHA (per ADR-013 design intent):** the `submit_jury_vote.rs`
`matches!()` guards are NOT exhaustive matches — they're early-return
patterns. Adding the three new variants to the step-5 guard at line
271 is a deliberate inclusion choice per §10.5 row 7. The step-6
guard at line 729 stays narrow per §10.5 row 8 rationale.

**Push and exit (Shape G).** Workspace check covers all 6 files.

**COMMIT MESSAGE:** `feat(v1-SL-a): ADR-013 enum-exhaustiveness sweep — 6 files handle 3 new CaseStatus variants (task 5)`

### Task 6 [P]: UPDATE `crates/api/api/src/governance/config.rs` — 13 new consts + match arms + SEEDED_KEYS + EXPECTED_SEED_COUNT_V1_SL + ConfigKeyMetadata + parity test

**ACTION:** Six sub-edits in `config.rs`:

1. 13 new `pub const DEFAULT_*` declarations after the v1-JM-a block.
2. 13 new match arms across `const_default_int` (10), `const_default_float`
   (1), `const_default_bool` (1), `const_default_text` (1).
3. 13 new tuples appended to `SEEDED_KEYS_WITH_CONSTS` after the
   v1-JM-a block (alphabetised within the new block by key).
4. New `pub const EXPECTED_SEED_COUNT_V1_SL: usize = 13;` after
   `EXPECTED_SEED_COUNT_V1_JM` at line 1322.
5. Extend `seeded_keys_count_matches_const_count` parity test at
   line 2422-2436 to include `EXPECTED_SEED_COUNT_V1_SL` in both the
   sum AND the error-message format string.
6. 13 new `CONFIG_KEY_METADATA` struct literals appended before the
   closing `];` of the array at line 2414.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/config.rs
```

**IMPLEMENT (file 1 of 1):** in
`crates/api/api/src/governance/config.rs`, perform all 6 sub-edits per
§10.6.

#### Pre-commit reconciliation gate (mandatory)

Before `git commit` of Task 6, run the reconciliation gate (mirroring
v1-JM-a Task 8 advisor-edit-#1):

```bash
# Count new SEEDED_KEYS_WITH_CONSTS v1-SL-a tuples
awk '/v1-SL-a additions/,/^];$/' crates/api/api/src/governance/config.rs | \
  grep -cE '^\s*\("'
# Expected: 13

# Count new DEFAULT_* declarations in the v1-SL-a additions block
awk '/-- v1-SL-a additions/,/^pub\(crate\) fn const_default_int/' crates/api/api/src/governance/config.rs | \
  grep -cE '^pub const DEFAULT_'
# Expected: 13

# Count INSERT rows in Task 1's up.sql (the seed migration)
grep -cE "^\s*\('instance', 'liability\.|^\s*\('instance', 'job\.grace_check_" \
  migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql
# Expected: 13

# Cross-check: every seeded key appears in down.sql DELETE
diff <(grep -oE "'(liability|job)\\.[a-z_.]+'" migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql | sort -u) \
     <(grep -oE "'(liability|job)\\.[a-z_.]+'" migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/down.sql | sort -u)
# Expected: empty (no key in up.sql missing from down.sql)
```

If any count disagrees, DO NOT commit. Three outcomes:

- **(a) SEEDED_KEYS count > up.sql count**: add missing INSERT rows
  to up.sql OR remove superfluous SEEDED_KEYS entries until they match.
- **(b) SEEDED_KEYS count < up.sql count**: add missing
  `SEEDED_KEYS_WITH_CONSTS` entries + matching `const_default_*`
  match arms + `CONFIG_KEY_METADATA` entries.
- **(c) up.sql <-> down.sql mismatch**: fix down.sql to mirror up.sql's
  key list exactly.
- **(d) neither side matches PRD §10**: file a DQ pending entry
  (`answered_by: null`) — PRD §10 is the source of truth; do not
  silently pick one count.

**MIRROR:** §10.6; v1-JM-a precedent at lines 850-902 + 1175-1298
+ 1315-1322 + 2200-2412 + 2422-2436.

**IMPORTS:** existing imports cover all new types
(`ConfigKeyMetadata`, `ConfigScope`, `ApplyAt`, `NumericRange`,
`ValueType` already in scope at top of file). No additions.

**GOTCHA:** the parametric pattern is mandatory per advisor directive
2026-04-19 #4 — each v1 sub-PRD adds its own
`EXPECTED_SEED_COUNT_V1_*` beside the v0 + AD + JM invariants
without churning them. Do NOT bump `EXPECTED_SEED_COUNT` (the v0
invariant) or `EXPECTED_SEED_COUNT_V1_AD` / `EXPECTED_SEED_COUNT_V1_JM`.

**GOTCHA (per DQ #115):** all 6 `liability.grace_window_*_hours`
DEFAULT_* consts are typed `i64` (raw integer hours), NOT `f64`
micros.

**GOTCHA:** SL-a's `liability.*` keys are DISTINCT from the existing
v0 `liability.*` keys at lines 1090-1092. Both coexist; SL-a's keys
all carry `liability.grace_window_*` / `liability.restoration_*` /
`liability.multi_sponsor_*` / `liability.revoke_*` namespace prefixes
per PRD §18 B4 key-rename table. No rename of existing v0 keys.

**GOTCHA:** SL-a's `job.*` keys at line 105-106 (existing
`job.snapshot_interval_seconds`, `job.snapshot_batch_chunk_size`)
remain unchanged; SL-a's three `job.grace_check_*` keys are additions
to the same flat namespace.

**GOTCHA (per `feedback_features_full_p_crate_incompatible.md`):** any
local diagnostic cargo invocation under §15.7 must use
`--workspace --features full`, never `-p lemmy_api --features full`.

**Push and exit (Shape G).**

**COMMIT MESSAGE:** `feat(v1-SL-a): config.rs — 13 new liability/job consts + metadata + SEEDED_KEYS + EXPECTED_SEED_COUNT_V1_SL parity (task 6)`

### Task 7 [P]: UPDATE `crates/db_schema/src/source/governance/governance_log.rs` + `crates/api/api/src/governance/governance_log.rs` + `.claude/rules/governance-log-entry-kind-registry.md` — 5 new ENTRY_KIND consts + registry section

**ACTION:** Three-file atomic commit:

1. DEFINE 5 new `ENTRY_KIND_*` consts in
   `crates/db_schema/src/source/governance/governance_log.rs` (insert
   AFTER the v1-JM-a block at line 185).
2. RE-EXPORT 5 new consts alphabetically in
   `crates/api/api/src/governance/governance_log.rs:39-76`.
3. POPULATE the `sponsor-liability-v1` reservation section in
   `.claude/rules/governance-log-entry-kind-registry.md` (replace the
   stub at lines 160-165 with 5 populated rows). Update the Acceptance
   invariants count from 33 to 38.

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema/src/source/governance/governance_log.rs
  - crates/api/api/src/governance/governance_log.rs
  - .claude/rules/governance-log-entry-kind-registry.md
```

**IMPLEMENT (file 1 of 3):** in
`crates/db_schema/src/source/governance/governance_log.rs`, after
line 185 (`pub const ENTRY_KIND_SEVERITY_TIER_FROZEN: &str = "severity_tier_frozen";`),
insert the 5 new consts per §10.7.

**IMPLEMENT (file 2 of 3):** in
`crates/api/api/src/governance/governance_log.rs:39-76`, insert 5
new alphabetical `pub use` lines per §10.7 specific positions.

**IMPLEMENT (file 3 of 3):** in
`.claude/rules/governance-log-entry-kind-registry.md`, replace lines
160-165 (the `### sponsor-liability-v1 (reserved ...)` stub) with the
populated section per §10.7. Also update the Acceptance invariants
section at lines 187-193 — change the count claim from `33` to `38`.

**MIRROR:** §10.7; v1-JM-a precedent at JM-a Task 9 (dual-file edit
+ registry section population).

**GOTCHA:** ALL THREE files land in the SAME commit — a schema
definition without the shim re-export breaks callers importing from
the api path; a shim re-export without the schema definition fails
to compile; a registry update without either is informational drift.

**GOTCHA (registry pre-landed-const exemption):** each of the 5 new
rows in the populated section MUST have a `(pending)` marker in the
"Emitting handler" column AND a citation of the specific downstream
plan that ships the call site. Per §10.7:
- `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` -> SL-d submit_jury_vote.rs
- `ENTRY_KIND_SPONSOR_LIABILITY_FIRED` -> SL-c sponsor_liability_grace.rs
- `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` -> SL-b revoke_endorsement.rs
  AND SL-c sponsor_liability_grace.rs
- `ENTRY_KIND_ENDORSEMENT_REVOKED` -> SL-b revoke_endorsement.rs
- `ENTRY_KIND_RESTORATION_COMPLETED` -> restorative-mechanics-v1 PRD
  restoration_complete.rs

A pre-landed const without a downstream-plan link is a registry-
pollution bug per the registry rule's Acceptance invariants line 192.

**GOTCHA:** `pub use` ordering is STRICT alphabetical, not "alphabetical
within the new additions". Insert the five new names into the existing
alphabetical sort at the specific positions per §10.7.

**GOTCHA (count invariant — verify post-commit):**
- `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs`
  -> expect **38** (was 33; +5).
- `rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l`
  -> expect **38**.
- `rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort | uniq -d`
  -> expect empty (no duplicate string literals).

**Push and exit (Shape G).**

**COMMIT MESSAGE:** `feat(v1-SL-a): add 5 ENTRY_KIND consts (db_schema define + api shim re-export) + registry v1-SL-a section (task 7)`

### Task 8: UPDATE `crates/server/tests/e2e.rs` — extend phase1_migrations_round_trip

**ACTION:** Two sub-edits in `crates/server/tests/e2e.rs`:

1. Bump `PHASE_1_MIGRATION_COUNT` constant by +1.
2. Extend the post-condition probe lists in
   `phase1_migrations_round_trip` to assert SL-a's effects per §10.8.

**Cohort B barrier.** Depends on Tasks 1, 2, 3, 4, 5, 6, 7.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
```

**IMPLEMENT (file 1 of 1):** in `crates/server/tests/e2e.rs`,
Edit-with-anchor (per `feedback_junior_worker_e2e_edit_hang.md`):

- **Anchor 1:** the `PHASE_1_MIGRATION_COUNT` declaration. Impl-task
  greps `'PHASE_1_MIGRATION_COUNT' crates/server/tests/e2e.rs` for
  the exact line and surrounding doc-comment. Bumps the integer by
  +1; updates the doc-comment to add the SL-a entry to the
  per-sub-phase breakdown.
- **Anchor 2:** the `phase1_migrations_round_trip` test block.
  Locates the post-condition probe loops. Extends:
  - Column-name assertion list: add `"grace_expires_at"`,
    `"liability_escape_reason"`.
  - Index-name assertion list: add `"moderation_case_grace_expires_idx"`,
    `"surety_sponsored_id_active"`.
  - pg_enum-value assertion list (case_status enum): add
    `"SponsorLiabilityPending"`, `"SponsorLiabilityFired"`,
    `"SponsorLiabilityEscaped"`. The post-down.sql assertion EXPECTS
    these three remain (Postgres limitation).
  - governance_config row count delta: assert +13 after up.sql,
    restored after down.sql.

**MIRROR:** §10.8; v1-JM-a Task 10.

**GOTCHA (R7):** `phase1_migrations_round_trip` is a test fn; the
workspace-check workflow's `cargo test --no-run -p lemmy_server
--test e2e` step compiles it.

**GOTCHA:** NO new test fns added — extending the existing
`phase1_migrations_round_trip` only. Single Edit-with-anchor block.
Per `feedback_junior_worker_e2e_edit_hang.md`: e2e.rs at >9000 lines
remains the worker-hang risk surface; one anchor per task is the
mitigation.

**GOTCHA:** the Postgres pg_enum residual assertion (three new values
remain after down.sql) is intentional per PRD §3.4 down.sql doc-
comment + Phase 5b Restoration variant precedent. The test asserts
the residual EXISTS to make the Postgres limitation explicit.

**Push and exit (Shape G):** push to `junior/<task-slug>`. The
workspace-check workflow runs `cargo test --no-run -p lemmy_server
--test e2e` (compile only). Phase 2 e2e (full execution) runs after
Junior's daemon finalize-merges into `phase-v1-SL-a` and the advisor
surfaces the local-vs-dispatch user gate.

**COMMIT MESSAGE:** `test(v1-SL-a): extend phase1_migrations_round_trip — bump count + new schema effects (task 8)`

### Task 9: WRITE `.claude/PRPs/reports/v1-SL-a-retro.md` — retrospective before PR

**ACTION:** Author the retrospective per
`feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`
+ `feedback_retro_task_complexity_score.md`. Sections: TL;DR; §1 What
worked; §2 Per-role signals (Advisor / Planning / Impl / BM); §3 What
didn't work; §4 Per-task complexity score table; §5 Lessons
promoted; §6 Confidence score; §7 Follow-up GH issue candidates.

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-SL-a-retro.md
modifies: []
```

**IMPLEMENT (file 1 of 1):** in
`.claude/PRPs/reports/v1-SL-a-retro.md`, mirror the most recent
shipped retro at `.claude/PRPs/reports/v1-JM-e-retro.md` for section
shape (or JM-d-retro fallback).

**Cross-cutting verification (Task 9 retro time — invariants the
retro asserts hold):**

- `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs`
  returns **38** (33 + 5).
- `rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l`
  returns **38**.
- `rg -n '\(pending\)' .claude/rules/governance-log-entry-kind-registry.md | rg -E 'SPONSOR_LIABILITY_(PENDING|FIRED|ESCAPED)|ENDORSEMENT_REVOKED|RESTORATION_COMPLETED'`
  returns 5 lines (all 5 SL-a consts marked `(pending)` with
  downstream-plan citations).
- `grep -c 'EXPECTED_SEED_COUNT_V1_SL' crates/api/api/src/governance/config.rs`
  returns >= 3 (declaration + parity test reference + error-message
  format string reference).
- The parity test
  `seeded_keys_count_matches_const_count` asserts
  `SEEDED_KEYS_WITH_CONSTS.len() == 88 + 13 == 101`.
- `crates/db_schema_file/src/enums.rs CaseStatus` declaration has 12
  variants (9 v0 + 3 SL-a).
- All 6 ADR-013 match sites enumerated in §10.5 enumerate the 3 new
  variants explicitly.
- `/brehon-verify` reports both §16a stories `[done]`.

**MIRROR:** `.claude/PRPs/reports/v1-JM-e-retro.md` (or JM-d-retro
fallback) for section structure.

**GOTCHA:** retro is written BEFORE `gh pr create` so CodeRabbit
review can pull context from the retro (per
`feedback_retro_not_report.md`).

**GOTCHA:** §4 per-task complexity-score table is mandatory per
`feedback_retro_task_complexity_score.md`. Each row carries the
post-hoc actual `<files-changed>/<commits>/<runtime-min>/<max-log-silence-min>`
metric.

**Push and exit (Shape G — retro is meta-work; `cargo-validate-workspace`
won't trigger because no `crates/**` change).**

**COMMIT MESSAGE:** `docs(v1-SL-a): phase retrospective before PR open (task 9)`

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cargo check --workspace --features full`
  via `cargo-validate-workspace.yml:89`.
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D
  warnings` via `cargo-validate-workspace.yml:92` (R6).
- **Test target compile (R7):** `cargo test --no-run -p lemmy_server
  --test e2e` via `cargo-validate-workspace.yml:95`.
- **Migration round-trip:** `bash scripts/brehon/migrate-roundtrip.sh`
  via `cargo-validate-migration.yml`. Triggers on Task 1's push (paths
  filter `migrations/**`). Validates the new migration applies +
  reverts + re-applies cleanly via `lemmy_diesel_utils` per the
  script's real implementation Task 0 ships.
- **e2e round-trip (Phase 2):** the existing
  `phase1_migrations_round_trip` test in `crates/server/tests/e2e.rs`,
  extended by Task 8 with SL-a-specific post-condition assertions.
- **Parity tests** (`#[cfg(test)] mod parity` in config.rs): runs as
  part of `cargo test --workspace --features full --test e2e`
  compile + Phase 2 e2e execute. Catches Task 6 count drift.

Pre-merge advisor-side verification (per §16a Stories): Story 1
checkpoint is workspace-check workflow `conclusion: "success"` on
Task 5's push (last code-touching task in Cohort B + barriers); Story
2 checkpoint is e2e `phase1_migrations_round_trip` exit 0 on the
post-Task 8 phase-branch tip; `/brehon-verify` Brief-Scope outputs
check.

---

## 15. Validation commands (DoD)

> **Shape G plan — DoD is per-workflow, not inline cargo.** Per
> `.claude/PRPs/templates/plan.template.md` §15.6 + the SL-a brief
> §2.3 mandate ("inline cargo invocations are forbidden in §15"). All
> validation runs on GH-hosted runners; the migration round-trip uses
> the real `migrate-roundtrip.sh` body Task 0 ships.

### 15.1 Per-task workspace check (Shape G)

For every §13 impl task that modifies `crates/**` (Tasks 1, 2, 3, 4,
5, 6, 7, 8):

- **DoD entry:** `cargo-validate-workspace.yml` on
  `junior/<task-slug>` SHA `<sha>` -> `conclusion: "success"`
- **Validation command:** `gh run list --repo barrie-cork/lemmy
  --branch <branch> --workflow cargo-validate-workspace --limit 1
  --json conclusion,databaseId --jq '.[0]'`
- **EXPECT:** `{"conclusion": "success", "databaseId": <id>}`

The workflow runs `cargo check --workspace --features full`,
`cargo clippy --workspace --features full --no-deps -- -D warnings`,
and `cargo test --no-run -p lemmy_server --test e2e` per
`.github/workflows/cargo-validate-workspace.yml:88-95`. R6 + R7 are
encoded.

### 15.2 Migration round-trip (Shape G — Task 1 only)

For Task 1 (the migration-touching task):

- **DoD entry:** `cargo-validate-migration.yml` on
  `junior/<task1-slug>` SHA `<sha>` -> `conclusion: "success"`
- **Validation command:** `gh run list --repo barrie-cork/lemmy
  --branch <branch> --workflow cargo-validate-migration --limit 1
  --json conclusion,databaseId --jq '.[0]'`
- **EXPECT:** `{"conclusion": "success", "databaseId": <id>}`

The workflow runs `bash scripts/brehon/migrate-roundtrip.sh` per
`.github/workflows/cargo-validate-migration.yml`. Task 0's stub-fix is
load-bearing — without it, the workflow exits 1 on Task 1's push.

### 15.3 Phase 2 e2e (post-finalize-merge)

After all impl tasks finalize-merge into `phase-v1-SL-a`, the advisor
surfaces the **Phase 2 e2e local-vs-dispatch user gate** per
`advisor-orchestrator.md`:

- **(a) local:** `cargo test -p lemmy_server --test e2e --features full -- --test-threads=1`
  on laptop in `run_in_background`; ~26 min wall-clock; zero billed.
- **(b) dispatch:** `gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-SL-a`;
  ci-watcher polls; ~26 min billed.

Plan-side DoD: e2e exit code 0; failure path -> §G4 classifier on log
slice.

### 15.4 Cross-cutting verification (Task 9 retro time)

- [ ] `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs`
  returns **38** (33 + 5).
- [ ] `rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l`
  returns **38**.
- [ ] `rg -n '\(pending\)' .claude/rules/governance-log-entry-kind-registry.md`
  shows the 5 SL-a rows have `(pending)` markers with downstream-plan
  citations.
- [ ] `grep -c 'liability\.\|job\.grace_check_' migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql`
  returns 13.
- [ ] `crates/db_schema_file/src/enums.rs CaseStatus` declaration has
  12 variants (9 v0 + 3 SL-a).
- [ ] All 6 ADR-013 match sites enumerated in §10.5 explicitly handle
  the 3 new variants (no `_ =>` arms).
- [ ] R1: every `i32 <-> i64` comparison in Task 4 + 8 uses
  `i64::from(...)`.
- [ ] R6: clippy invocations in `cargo-validate-workspace.yml` use
  `--no-deps -- -D warnings`.
- [ ] No edits to files outside §11 list.
- [ ] Every SL-a §16a story is `[done]`.

### 15.5 ADR / OQ compliance verification

- [ ] **ADR-010** honoured — backfill UPDATE bounded by
  `decided_at > now() - INTERVAL '24 hours'` (most-lenient default
  per won't-disadvantage rule); pre-existing `Decided` cases with
  `reputation_event` rows excluded by the third EXISTS guard.
- [ ] **ADR-013** honoured — exhaustive `CaseStatus` match preserved
  across all 6 sites; no `_ =>` arms introduced.
- [ ] **ADR-014** honoured — no federation outbound activity types
  for sponsor-liability events.
- [ ] **ADR-015** honoured — `liability_escape_reason` JSONB column
  COMMENT documents `actor_pseudonym` (scrubbed via
  `governance_log::append`'s `scrub_json` per ADR-015), NOT raw
  `person_id`.
- [ ] **OQ-V1-SL-05** honoured — `liability_escape_reason` schema
  documents `version: 1` from day one in the column COMMENT.
- [ ] **PRD §2 OUT** honoured — no `revoke_endorsement` handler / no
  scheduler module / no `submit_jury_vote` mutation / no
  restoration_complete endpoint.

### 15.6 DoD per workflow (canonical Shape G shape)

**Phase 1 (workspace check):**

- Workflow: `.github/workflows/cargo-validate-workspace.yml`
- Branch (per task): `junior/<task-slug>`
- Expected `conclusion`: `"success"`

**Phase 1b (migration round-trip — Task 1 only):**

- Workflow: `.github/workflows/cargo-validate-migration.yml`
- Branch: `junior/<task1-slug>`
- Expected `conclusion`: `"success"`
- Pre-condition: `scripts/brehon/migrate-roundtrip.sh` body fixed by
  Task 0.

**Phase 2 (e2e):**

- Workflow: `.github/workflows/cargo-test-e2e.yml` (only on user
  dispatch per PR #105 / 2026-04-28)
- OR local: `cargo test -p lemmy_server --test e2e --features full -- --test-threads=1`
  on laptop
- Branch: `phase-v1-SL-a`
- Expected: all tests pass (specifically including the extended
  `phase1_migrations_round_trip`)

### 15.7 Manual validation snippets (advisor-side smoke only — non-binding)

> Per `feedback_features_full_p_crate_incompatible.md`: never
> `-p <crate>` + `--features full`; use `--workspace --features full`.

```bash
yamllint .github/workflows/cargo-validate-workspace.yml \
         .github/workflows/cargo-validate-migration.yml \
         .github/workflows/cargo-test-e2e.yml
echo "yamllint exit: $?"

# Local dry-run of the seed migration matches SEEDED_KEYS count
diff <(grep -oE "'(liability|job)\\.[a-z_.]+'" migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql | sort -u) \
     <(grep -oE 'liability\.|job\.grace_check_' crates/api/api/src/governance/config.rs | sort -u | head -13)
```

These are advisor-side only; do not count as plan §16 acceptance
criteria. Per DQ #67 resolution.

---

## 16. Acceptance criteria

- [ ] All 10 tasks (Task 0..8 + Task 9 retro) committed.
- [ ] §15.1 (`cargo-validate-workspace.yml`) `conclusion: "success"`
  after every impl task push.
- [ ] §15.2 (`cargo-validate-migration.yml`) `conclusion: "success"`
  after Task 1's push.
- [ ] §15.3 (Phase 2 e2e — local or dispatch) all tests pass; the
  extended `phase1_migrations_round_trip` validates SL-a's effects.
- [ ] §15.4 (cross-cutting verification — 10 boxes) all ticked.
- [ ] §15.5 (ADR / OQ compliance — 6 boxes) all ticked.
- [ ] §16a stories — both `[done]`.
- [ ] No edits to files outside §11 list.
- [ ] Retro committed per Task 9.
- [ ] PR opens against `governance-v0` (NOT `main`) with
  `--repo barrie-cork/lemmy`.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-SL-a-verify.md`
  shows all stories ✓.

---

## 16a. Stories (independently-testable behaviour units)

Per `.claude/PRPs/templates/plan.template.md` §16a +
`feedback_story_grain_checkpoint.md` +
`feedback_brehon_verify_pre_merge.md`. Two stories — one per
behaviourally distinct unit.

### Story 1: Schema migration round-trips cleanly + new columns + new enum variants exist + ADR-013 match sites compile

- **Composing tasks:** Tasks 1, 2, 3, 4, 5
- **Checkpoint workflow (Phase 1):** `cargo-validate-workspace.yml` on
  Task 5's worker branch SHA -> `conclusion: "success"`.
- **Checkpoint workflow (Phase 1b):** `cargo-validate-migration.yml` on
  Task 1's worker branch SHA -> `conclusion: "success"`.
- **Checkpoint workflow (Phase 2):** Phase 2 e2e for the extended
  `phase1_migrations_round_trip` -> exit 0.
- **Expected output (local Phase 2):** `1 passed; 0 failed` for
  `phase1_migrations_round_trip`.
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql`
    exists and contains `ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityPending'`
    AND `ALTER TABLE moderation_case ADD COLUMN grace_expires_at TIMESTAMPTZ`
    AND `CREATE INDEX moderation_case_grace_expires_idx`
    AND `CREATE INDEX surety_sponsored_id_active`
    AND `INSERT INTO governance_config` (× 13 rows)
    AND `UPDATE moderation_case SET status = 'SponsorLiabilityPending'`.
  - `migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/down.sql`
    exists and contains `DELETE FROM governance_config` AND
    `DROP INDEX IF EXISTS surety_sponsored_id_active` AND
    `DROP INDEX IF EXISTS moderation_case_grace_expires_idx` AND
    `ALTER TABLE moderation_case DROP COLUMN IF EXISTS liability_escape_reason`
    AND `ALTER TABLE moderation_case DROP COLUMN IF EXISTS grace_expires_at`.
  - `crates/db_schema_file/src/enums.rs` `CaseStatus` declaration
    contains `SponsorLiabilityPending`, `SponsorLiabilityFired`,
    `SponsorLiabilityEscaped` variant names.
  - `crates/db_schema_file/src/schema.rs` `moderation_case (id) { ... }`
    block contains `grace_expires_at -> Nullable<Timestamptz>` AND
    `liability_escape_reason -> Nullable<Jsonb>` lines.
  - `crates/db_schema/src/source/governance/moderation_case.rs`
    `pub struct ModerationCase` contains `grace_expires_at:
    Option<DateTime<Utc>>` AND `liability_escape_reason: Option<Value>`
    fields. `pub struct ModerationCaseInsertForm` contains the same
    two `Option<_>` fields.
  - All 6 ADR-013 sweep files (per §11) explicitly enumerate
    `SponsorLiabilityPending`, `SponsorLiabilityFired`,
    `SponsorLiabilityEscaped` in their respective `match case.status`
    or `matches!()` patterns. No `_ =>` arms introduced.
  - `scripts/brehon/migrate-roundtrip.sh` body has been replaced
    (no longer contains the `migrate-roundtrip.sh: no migrations/
    change detected vs governance-v0; stub exiting 0` literal at
    the end of the file); contains `cargo run -p lemmy_diesel_utils`
    invocation literal.

### Story 2: 13 config keys + 5 ENTRY_KIND consts land with parity + registry invariants

- **Composing tasks:** Tasks 6, 7, 8 (Task 8 covers the parity-test
  invocation in `phase1_migrations_round_trip`'s extended assertions).
- **Checkpoint workflow (Phase 1):** `cargo-validate-workspace.yml` on
  Task 8's worker branch SHA -> `conclusion: "success"`. The
  workspace-check's `cargo test --no-run` step catches parity-test
  compile errors; the parity test itself runs in Phase 2.
- **Checkpoint workflow (Phase 2):** Phase 2 e2e parity test +
  extended `phase1_migrations_round_trip` (governance_config row count
  delta of +13) -> exit 0.
- **Expected output (local Phase 2):** `seeded_keys_count_matches_const_count`
  passes (101 == 101); `every_seeded_key_has_metadata` passes (101 ==
  101); `phase1_migrations_round_trip` passes.
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `crates/api/api/src/governance/config.rs` contains
    `pub const EXPECTED_SEED_COUNT_V1_SL: usize = 13;` declaration.
  - `crates/api/api/src/governance/config.rs` parity test at line ~2422
    contains the literal `+ EXPECTED_SEED_COUNT_V1_SL` term in its
    sum computation AND `EXPECTED_SEED_COUNT_V1_SL` in its error-
    message format string.
  - `crates/api/api/src/governance/config.rs` `SEEDED_KEYS_WITH_CONSTS`
    extension block contains the 13 expected `("liability....", "DEFAULT_LIABILITY_...", "...")`
    or `("job.grace_check_...", "DEFAULT_JOB_GRACE_...", "...")`
    tuples (verifiable via `rg` count: 13 such tuples in a contiguous
    `// v1-SL-a additions` block).
  - `crates/api/api/src/governance/config.rs` contains 13 new
    `pub const DEFAULT_LIABILITY_...` / `DEFAULT_JOB_GRACE_...`
    declarations in a `// -- v1-SL-a additions` block.
  - `crates/db_schema/src/source/governance/governance_log.rs`
    contains 5 new `pub const ENTRY_KIND_...` declarations:
    `ENTRY_KIND_SPONSOR_LIABILITY_PENDING`,
    `ENTRY_KIND_SPONSOR_LIABILITY_FIRED`,
    `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED`,
    `ENTRY_KIND_ENDORSEMENT_REVOKED`,
    `ENTRY_KIND_RESTORATION_COMPLETED`.
  - `crates/api/api/src/governance/governance_log.rs` `pub use` block
    re-exports the 5 new const names.
  - `.claude/rules/governance-log-entry-kind-registry.md` contains a
    populated `## v1-SL-a entry kinds (5, this sub-phase)` section with
    5 markdown table rows AND 5 `(pending)` markers in the "Emitting
    handler" column.
  - `.claude/rules/governance-log-entry-kind-registry.md` Acceptance
    invariants section count is updated to **38** (`rg -c '^pub const
    ENTRY_KIND_' ...` invariant assertion).
  - `crates/server/tests/e2e.rs` `PHASE_1_MIGRATION_COUNT` constant is
    +1 vs pre-SL-a baseline; `phase1_migrations_round_trip` block
    includes assertion strings for `grace_expires_at`,
    `liability_escape_reason`, `moderation_case_grace_expires_idx`,
    `surety_sponsored_id_active`, `SponsorLiabilityPending`,
    `SponsorLiabilityFired`, `SponsorLiabilityEscaped`.

> **Verification mapping:** `/brehon-verify` iterates this section,
> runs each Story's checkpoint workflow on the worktree branch, and
> confirms each Brief-Scope output exists + matches its structural
> pattern. Phantoms (task complete but output absent or empty) trigger
> the catch-fire procedure in `advisor-orchestrator.md`.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0..10 confirmed; migrate-roundtrip.sh
  fixed).
- [ ] Tasks 1..8 committed.
- [ ] Task 9 retro committed.
- [ ] §15 validation green at every gate (Phase 1 workspace check ×8,
  Phase 1b migration round-trip ×1, Phase 2 e2e ×1).
- [ ] §16a stories all `[done]`.
- [ ] PR opened by BM session against `governance-v0`.
- [ ] CodeRabbit review complete with findings triaged per
  `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-SL-a-verify.md`
  shows all stories ✓.
- [ ] Post-merge phase branch retained for retro reads.
- [ ] DQ #114 (migrate-roundtrip.sh stub fix) closed atomically with
  Task 0.
- [ ] DQ #115 (micros-vs-raw-hours clarification) cited in §13 Task 1
  + Task 6 doc-comments.
- [ ] DQ #116 (split-or-proceed) resolved by advisor with
  proceed-as-one rationale (per the JM-e precedent at score 15).

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `migrate-roundtrip.sh` real implementation fails on first run because `lemmy_diesel_utils` CLI surface differs from assumed `run`/`revert` sub-commands | MED | HIGH | Task 0 GOTCHA — impl-task verifies binary sub-command shape from `crates/diesel_utils/src/main.rs` (or equivalent) before committing; if unclear, files DQ pending |
| `cargo-validate-migration.yml` exits non-zero on Task 1's push (stub still present from a stale checkout) | LOW | HIGH | Task 0 STAGES the stub fix BEFORE Task 1 ships the migration; the workflow's path filter on `migrations/**` ensures Task 0's commit alone (modifying only the script, not migrations) does not trigger the workflow — Task 1's combined push triggers both workflows |
| Cohort A (5-way parallel) suffers GitHub Actions rate-limiting / concurrent-runner contention | LOW | LOW | GH Free quota allows up to 20 concurrent runners on linux; SL-a's 5 parallel workflows fit; cohort budget check non-binding under Shape G per `advisor-orchestrator.md` |
| Cohort B (Tasks 4 + 5) cohort-overlap because Task 5 modifies `submit_jury_vote.rs` and a future drift in §10.5 introduces additional sites that overlap with Task 4 | LOW | MED | Per cohort-dispatch YAML overlap rule, advisor mechanically refuses overlap; degraded to serial. Pre-impl Task 0 Probe 4 grep enumerates the live ADR-013 site list and surfaces drift |
| Postgres-side `ALTER TYPE ... ADD VALUE` runs outside transaction (per `-- no-transaction` directive) and leaves partial state on mid-migration crash | LOW | MED | Per Phase 5b Restoration variant precedent: the directive is required by Postgres; partial-state recovery is documented; re-running the migration is idempotent via `IF NOT EXISTS` |
| Backfill UPDATE locks `moderation_case` rows during deploy, blocking concurrent reads | LOW | MED | Driving filter `decided_at > now() - INTERVAL '24 hours'` bounds the row-set tightly (typically <100 rows); Postgres row-level locking does not block reads |
| Backfill UPDATE incorrectly flips a v0 case that already fired sponsor-liability into `SponsorLiabilityPending`, double-charging sponsors when SL-c scheduler runs | LOW | HIGH | The third `NOT EXISTS reputation_event WHERE reason = 'sponsor_liability_applied'` guard is load-bearing per PRD §8.4 + §11.1; Task 1 GOTCHA reaffirms; e2e `phase1_migrations_round_trip` extension validates the row-count delta |
| Issue #24 partial index `surety_sponsored_id_active` collides with a future SL-b/SL-c index | LOW | LOW | Index name is canonical per Issue #24; SL-b/SL-c use distinct names |
| Complexity score 13 rejected by advisor (split-mandated) | HIGH | LOW | §5.2 — DQ #116 filed; if split, this plan becomes `v1-SL-a-1.plan.md` and `v1-SL-a-2.plan.md` is sibling. JM-e precedent at score 15 shipped proceed-as-one with no operational regret |
| `EXPECTED_SEED_COUNT_V1_SL` value (13) drifts from PRD §10's enumerated count (also 13) | LOW | MED | Task 6 pre-commit reconciliation gate runs grep counts that must agree (SEEDED_KEYS, DEFAULT_*, up.sql INSERT, down.sql DELETE) |
| Registry rule `(pending)` row for `restoration_completed` becomes stale if restorative-mechanics-v1 PRD ships under a different file name | LOW | LOW | Registry §10.7 cites `restorative-mechanics-v1 PRD restoration_complete.rs`; the citation is descriptive — drift is fixable post-hoc when restorative-mechanics-v1 ships |
| GH-Actions minutes budget exceeded by 5 Phase-1 + 1 Phase-1b + 1 Phase-2 e2e | LOW | LOW | Phase 2 e2e local-default per JM-d retro §5; Phase 1 workspace-check ~3 min × 8 tasks = 24 min; Phase 1b migration round-trip ~5 min × 1 = 5 min; total ~29 min monthly burn well within Free 3000 min |
| PMD #126 — DQ ID collision from concurrent SL-b / rep-tuning-r3 work | LOW | LOW | next-id calc spans archives + live; advisor monitors at finalize-merge |
| Junior worker pre-pushes break finalize-merge | LOW | LOW | Junior daemon's finalize step is post-Shape-G correct (per JM-d retro §1.1 + `feedback_junior_finalize_skips_when_worker_pre_pushes.md`) |

---

## 19. Notes

### 19.1 Planner DQs filed

- **DQ #116** (`from: "planner"`, `kind: "blocker"`, `answered_by:
  null`) — complexity-score-split decision per §5.2. Question:
  "Complexity score 13 exceeds 8 — split `v1-sponsor-liability-a`
  into `v1-SL-a-1` (Tasks 1-5) + `v1-SL-a-2` (Tasks 6-8) + retro,
  or proceed as one plan?". Options: split / proceed. Planner
  observation favours proceed (mechanical schema work; JM-e
  precedent at score 15 shipped proceed-as-one with no operational
  regret; PRD §15 names the deliverable as one atomic unit
  "Schema + enum + migration").

### 19.2 Self-resolved planner findings (LESSON candidates)

- **The brief's pre-estimate of 5-7 was mathematically optimistic.**
  The brief undercounted crates touched (didn't include `api_crud`
  for request_appeal.rs ADR-013 sweep), mis-attributed
  `lemmy_diesel_utils` as a touched crate (only the shell wrapper
  changes; the Rust binary is unchanged), and assumed 0 e2e edits
  when the brief's own §3 Required reading item 13 mandates the
  `phase1_migrations_round_trip` extension. Recommendation for retro:
  brief-template adjustment — pre-estimate rubric should walk the
  FILES YAML block per task before declaring a count, not infer from
  the headline.
- **Postgres pg_enum value drop is not supported** — SL-a's down.sql
  documents this; the three new values stay as orphans after revert.
  Phase 5b's Restoration variant down.sql had the same precedent.
  Retro candidate: promote `feedback_postgres_pg_enum_no_drop_variant.md`
  if not already present.
- **The cohort A 5-way parallel dispatch** (Tasks 1, 2, 3, 6, 7) is
  the largest cohort the SL lane will see. Tasks 4 + 5 (Cohort B) are
  2-way. Subsequent SL-b/c/d/e plans will likely cohort smaller (the
  handlers are code-heavy with cross-file coupling).

### 19.3 Pre-existing pending DQ entries

- **None at planning time.** `decision-queue.json` shows 0 pending
  entries on governance-v0 @ `48c4f379d`. Recently resolved: DQ #114
  (migrate-roundtrip.sh fold-in to Task 0), DQ #115 (micros-vs-raw-
  hours), DQ #109..#113 (JM-e e2e regression validate-pending-laptop-
  e2e entries — all resolved).

### 19.4 Out-of-scope follow-ups (Task 9 retro candidates)

- **DQ #68** (`migrate-roundtrip.sh` stub fix) — closed atomically by
  Task 0 of this plan per DQ #114 resolution.
- **`apply_sponsor_liability` compute/fire split** — SL-d candidate.
- **`revoke_endorsement` handler** — SL-b candidate.
- **`sponsor_liability_grace.rs` scheduler module** — SL-c candidate.
- **e2e behavioural tests** (revocation-during-window-escapes,
  restoration-during-window-escapes, window-expiry-fires,
  backfill-of-mid-flight) — SL-e candidate.
- **`restoration_complete` endpoint** — restorative-mechanics-v1 PRD
  candidate.
- **Sponsor notification UX** (Lemmy notification + email opt-in) —
  PRD §13 OQ-V1-SL-03 candidate; gated on notification surface
  generalised by jury-mechanics-v1 OQ-005.
- **Step-up auth enforcement on `RevokeEndorsement.step_up_token`** —
  v2 candidate (Keycloak + WebAuthn substrate).

### 19.5 Confidence bands

- **High (9/10):** schema migration shape — exact mirror of JM-a Task
  1 + Task 2 with PRD §8.5 explicit ordering.
- **High (9/10):** ADR-013 enum-exhaustiveness sweep — 6 sites grep-
  enumerated at planning time + per-site decision matrix in §10.5.
- **Moderate (7/10):** migrate-roundtrip.sh real implementation —
  diesel_utils CLI surface is verified at impl-task time; if binary
  sub-command shape differs from assumption, Task 0 GOTCHA falls back
  to DQ.
- **High (8/10):** parametric `EXPECTED_SEED_COUNT_V1_SL` invariant —
  parity test catches drift loudly; pre-commit reconciliation gate at
  Task 6 catches earlier.
- **High (9/10):** ENTRY_KIND dual-file edit + registry section
  population — JM-a precedent is direct.
- **Moderate (7/10):** §5 complexity score 13 will trip split-DQ; plan
  ships under proceed-as-one assumption pending DQ #116 resolution.

### 19.6 Why no clarify DQ at impl time

Brief §4.2 enumerates the boundary-of-judgment cases that should
trigger an impl-side DQ pending entry. None apply to this plan as
written:

- Migration timestamp `2026-05-03-000000-0000` does not collide with
  any existing migration directory at brief-write time (verified at
  Task 0 Probe 5 + via `ls migrations/ | tail -3`).
- `liability_escape_reason` JSONB schema is set per OQ-V1-SL-05 with
  `version: 1` from day one (PRD §8.1) — no schema-extension question.
- `EXPECTED_SEED_COUNT_V1_SL = 13` matches PRD §10's footer count + the
  enumerated table entries (13 = 6 grace_window + 2 restoration + 1
  multi-sponsor + 1 revoke + 3 grace_check). No discrepancy.
- DQ #114 already settled the `migrate-roundtrip.sh` stub-fix scope
  (Task 0 fold-in).
- Registry rule's `sponsor-liability-v1 (reserved)` block is empty at
  brief-write time (verified per `.claude/rules/governance-log-entry-kind-registry.md`
  lines 160-165 stub status).

If any of these baseline assumptions changes between plan-write and
impl-time, the impl-task subagent files a DQ pending entry rather than
proceeding silently.

---

## 20. Confidence score

- **Plan correctness:** 8/10 — patterns mirror JM-a directly + §10
  §13 §16a all match the JM-e Shape-G shape; the
  migrate-roundtrip.sh real implementation is the highest-uncertainty
  edit (Moderate confidence per §19.5).
- **Cargo budget:** 10/10 — Shape G (cargo runs off-box; budget
  non-binding; forbidden-window non-binding).
- **Test coverage:** 7/10 — `phase1_migrations_round_trip` extension +
  parity test cover schema + parity invariants; behavioural backfill
  semantics intentionally deferred to SL-c/SL-e (per the brief §2.2
  scope-boundary).
- **Story-grain decomposition:** 9/10 — every task maps to exactly one
  story; both stories have concrete checkpoint workflows + grep-
  verifiable Brief-Scope outputs.
- **Plan-shape conformance:** 9/10 — 20-section schema followed
  literally; §15.6 Shape G shape adopted; §16a mandatory present;
  §13 per-task FILES YAML block on every task; §5.1 complexity
  breakdown table mechanical.

---

_Plan author: planning subagent (Junior `sl-a-planning-1`,
2026-05-03). Plan committed on
`junior/role-planning-v1-sponsor-liability-a-plan-see-claude-prps-briefs-sl-a-planning-1-md-81`;
finalize step pushes to that branch; advisor session merges via the
standard sub-phase flow into `governance-v0` where SL-a's BM-task
then cuts `phase-v1-SL-a`. One planner DQ raised at commit time:
DQ #116 (split-or-proceed; advisor blocking). Confidence 8/10. Plan
ships under proceed-as-one assumption pending DQ #116 resolution._

LESSON: schema-foundation sub-phases (the "first sub-phase of a v1
lane") consistently land 6-8 impl tasks once the ADR-013-style
exhaustive-match sweep and the parametric `EXPECTED_SEED_COUNT_V1_*`
parity discipline are honoured. The brief-template's pre-estimate
heuristic for complexity score should walk the FILES YAML per task
before declaring a count — inferring from the headline ("schema +
enum + seeds + consts") consistently undercounts. Future planning
briefs for SL-b/c/d should drop the pre-estimate altogether and let
the planner compute it (the score gates split-DQ at >8; advisor
review either way).
