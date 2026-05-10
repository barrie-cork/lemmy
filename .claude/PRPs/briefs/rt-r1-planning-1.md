# RT-r1 planning brief

**Written**: 2026-05-10 by advisor session (laptop CWD `C:\Users\barri\Developer\brehon-fork` on `governance-v0`) for execution by either Junior on EliteDesk OR a local `planning` subagent invocation depending on EliteDesk daemon state at dispatch time.

**Subagent target**: `planning` (Opus 4.7, color purple — see `.claude/agents/planning.md`).

**Worktree**: Junior cuts `junior/rt-r1-planning-1` from `governance-v0` per concurrency-1 default; OR if executed locally, the `planning` subagent operates in a sibling git worktree on `governance-v0`. The plan file commits and pushes to `governance-v0` at finalize.

**Authority anchor**: `v1-reputation-tuning.prd.md` §15 row 1 (`v1.r1 — schema & feature flag`) + §7 (Cross-Cutting Impact: schema additions + backfill) + §8 (Defaults Matrix — 28 new knobs + 1 feature flag) + §10 (feature flag semantics — flagged in §7 carry-forward as `feature.reputation_v1_decay_enabled`). PRD §15 row 1 dependency: **OQ-018 endpoint (admin config write)** — verified shipped in v1-AD-b PR #76 merge (commit `f03ed1cba`); dependency satisfied.

**Parallel-safety to SL-d (concurrent planning)**: SL-d touches `crates/api/api/src/governance/sponsor_liability.rs` + `crates/api/api/src/governance/submit_jury_vote.rs` + new tests in `crates/server/tests/e2e.rs` (anchor-Edits at file end). RT-r1 touches `crates/db_schema/migrations/**` (new migrations) + `crates/db_schema/src/source/reputation_event.rs` (column adds) + new `sponsor_allowlist.rs` source file + 28 new `DEFAULT_*` consts in `crates/api/api/src/governance/config.rs` + 7 new `ENTRY_KIND_*` consts in `crates/db_schema/src/source/governance/governance_log.rs` (re-exported in `crates/api/api/src/governance/governance_log.rs`). **Zero handler-file overlap with SL-d**; zero overlap with `submit_jury_vote.rs` (RT-r1 is schema + seed + feature-flag-only — no handler edits). Anchor-Edit collisions on `e2e.rs` are the only theoretical risk and are addressable by appending RT-r1's tests below SL-d's anchors per `feedback_junior_worker_e2e_edit_hang.md` cohort discipline.

**Scope (locked)**: RT-r1 is the schema-and-seed layer for the entire reputation-tuning v1 lane. Three deliverable halves (per PRD §15 row 1):

1. **Schema additions** — three additions per PRD §7 row "Schema changes":
   - new `dedupe_key TEXT` nullable column on `reputation_event` with **partial unique index** (`WHERE dedupe_key IS NOT NULL`)
   - new `source_event_type` enum column on `reputation_event` (PG enum, 9 variants — `Endorsement | JuryVote | SponsorLiability | FounderSeed | ParticipationCron | DormancyCron | VoteOutcome | EvidenceQuality | ManualSeed`); default `Endorsement` for backfill, populated by emitters going forward
   - new `sponsor_allowlist` table (`id`, `community_id` NULL, `person_id`, `added_by_admin_id`, `added_at`, `note`)

2. **Seed v1 config rows** — 28 new keys per PRD §8 Defaults Matrix + 1 feature flag (`feature.reputation_v1_decay_enabled`, default `false`). Seed via `EXPECTED_SEED_COUNT_V1_RT` parametric const (follows v1-AD-a + v1-JM-a precedent). Plus 7 new `ENTRY_KIND_*` consts (per PRD §7 hash-chain row): `ENTRY_KIND_PARTICIPATION_CRON_TICK`, `ENTRY_KIND_VOTE_OUTCOME_RECORDED`, `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED`, `ENTRY_KIND_ROLLUP_RECOMPUTED`, `ENTRY_KIND_DECAY_KNOB_CHANGED`, `ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED`, `ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED`. Dual-file edit per v1-AD-a §10.8 pattern: define in `db_schema`, re-export in `api` shim. Append reputation-tuning-v1 section to `.claude/rules/governance-log-entry-kind-registry.md`.

3. **v0→v1 backfill migration** — existing `reputation_event` rows get `dedupe_key = NULL` (only new cron events use it) + `source_event_type` default-mapped from existing `reason`/source columns via a one-off migration. Pattern follows v1-JM-a backfill (`feedback_lemmy_migration_runner.md` discipline; `forbid_diesel_cli` posture; `cargo run -p lemmy_diesel_utils --features full`).

---

## 1. Role + dispatch line

`[role:planning] v1-reputation-tuning-r1 plan — schema + 28 config keys + sponsor_allowlist + backfill`

The actual create-task description is a single line under 100 chars per `.claude/rules/advisor-orchestrator.md` Junior task description template:

```
[role:planning] v1-reputation-tuning-r1 plan — see .claude/PRPs/briefs/rt-r1-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` for sub-phase **v1-RT-r1**. The plan covers PRD §15 row 1 in full (schema additions + 28 seeded keys + sponsor_allowlist table + 7 entry-kind consts + backfill).

### 2.1 Five concrete deliverables (per PRD §15 row 1 + §7 + §8 + §10)

The plan's §13 task list MUST cover all five:

a. **Migration: `dedupe_key` + `source_event_type` on `reputation_event`.** New migration directory under `crates/db_schema/migrations/{ts}_add_reputation_event_v1_columns/`. `up.sql` adds:
   - `ALTER TABLE reputation_event ADD COLUMN dedupe_key TEXT`
   - `CREATE UNIQUE INDEX reputation_event_dedupe_key_partial_idx ON reputation_event (dedupe_key) WHERE dedupe_key IS NOT NULL`
   - PG enum type creation: `CREATE TYPE reputation_event_source_type AS ENUM ('Endorsement', 'JuryVote', 'SponsorLiability', 'FounderSeed', 'ParticipationCron', 'DormancyCron', 'VoteOutcome', 'EvidenceQuality', 'ManualSeed')`
   - `ALTER TABLE reputation_event ADD COLUMN source_event_type reputation_event_source_type NOT NULL DEFAULT 'Endorsement'`

   `down.sql` reverses in inverse order (drop column → drop index → drop column → drop type). Plan §13 task includes round-trip e2e gate (`phase1_migrations_round_trip` extended to cover the new migration; v1-JM-a precedent — `EXPECTED_SEED_COUNT_V1_RT` const + parametric matcher).

   Diesel `schema.rs` regen + `crates/db_schema/src/source/reputation_event.rs` struct update (new fields). Per `feedback_lemmy_migration_runner.md`: `cargo run -p lemmy_diesel_utils --features full -- migration run` for local apply; `forbid_diesel_cli`; `cargo run --bin lemmy_print_schema_with_pg_features --features full` for `schema.rs`. Per `feedback_postgres_jsonb_canonicalization.md` if any JSONB ends up here (none expected; flag for planner).

b. **Migration: `sponsor_allowlist` table.** Separate migration directory `crates/db_schema/migrations/{ts}_create_sponsor_allowlist/`. `up.sql`:
   ```sql
   CREATE TABLE sponsor_allowlist (
     id SERIAL PRIMARY KEY,
     community_id INTEGER REFERENCES community(id) ON DELETE CASCADE,  -- NULL for instance-wide
     person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
     added_by_admin_id INTEGER NOT NULL REFERENCES person(id),
     added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     note TEXT,
     UNIQUE (community_id, person_id)  -- one row per (community-or-instance, person)
   );
   ```
   `down.sql` `DROP TABLE sponsor_allowlist`. New diesel struct in `crates/db_schema/src/source/sponsor_allowlist.rs`. Newtype check: `SponsorAllowlistId` newtype lives in `crates/db_schema/src/newtypes.rs` per `feedback_newtype_locations_lemmy_db_schema_vs_file.md` — verify pattern; planner authors only the `_id` newtype, not the `Id` newtype (subtype confusion footgun).

c. **Backfill migration for existing `reputation_event` rows.** Separate migration `{ts}_backfill_reputation_event_source_type/`. `up.sql` populates `source_event_type` from existing column heuristics:
   - Rows where `endorsement_id IS NOT NULL` → `'Endorsement'`
   - Rows where `jury_vote_id IS NOT NULL` → `'JuryVote'`
   - Rows where `case_id IS NOT NULL AND reason ILIKE '%sponsor_liability%'` → `'SponsorLiability'`
   - Rows where `reason ILIKE '%founder_seed%'` → `'FounderSeed'`
   - Default fallback for unmatched → `'ManualSeed'`

   Heuristic mapping is best-effort. Planner DQ if PRD §7 row "Backfill" doesn't enumerate the heuristic patterns precisely (it doesn't — the brief's heuristics above are the planner's starting point, NOT a contract). Plan §13 task includes a smoke check: post-migration, `SELECT source_event_type, COUNT(*) FROM reputation_event GROUP BY 1` shows non-zero counts for at least `Endorsement` (the dominant v0 source).

d. **Seed 28 new config keys + 1 feature flag in `seed_v1_rt_config_keys`.** Pattern mirrors v1-AD-a `seed_v1_config_keys` and v1-JM-a `seed_v1_jm_config_keys`. Located in `crates/db_schema/src/utils/v1_rt_config_seed.rs` (or planner-chosen sibling location). Each key seeded with the default value from PRD §8 Defaults Matrix (full table copied verbatim into plan §13 — 28 rows + 1 feature flag row). Per-key writer uses the existing `governance_config::INSERT_INTO` helper. Parametric const `EXPECTED_SEED_COUNT_V1_RT = 29` (28 knobs + 1 feature flag). Parity test extended: `SEEDED_KEYS_WITH_CONSTS.len() == EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD + EXPECTED_SEED_COUNT_V1_JM + EXPECTED_SEED_COUNT_V1_RT` (mirrors v1-AD-a §19 cumulative invariant; planner confirms current cumulative count from v1-JM-a + v1-SL-a additions before authoring).

   28 `DEFAULT_*` consts added to `crates/api/api/src/governance/config.rs` (one per knob; type per PRD §8 column 3). The `feature.reputation_v1_decay_enabled` flag follows the existing `feature.*` namespace pattern (per `crates/api/api/src/governance/feature_flags.rs` if it exists — planner check).

e. **7 new `ENTRY_KIND_*` consts (zero migration; `governance_log.entry_kind` is TEXT).** Dual-file edit per v1-AD-a §10.8 pattern:
   - Define in `crates/db_schema/src/source/governance/governance_log.rs`: 7 `pub const ENTRY_KIND_*: &str = "...";` lines with snake_case string values matching the const name.
   - Re-export in `crates/api/api/src/governance/governance_log.rs` (the api shim) via `pub use crate::source::governance::governance_log::{ENTRY_KIND_PARTICIPATION_CRON_TICK, ...};`.
   - Append a new section to `.claude/rules/governance-log-entry-kind-registry.md` titled `## reputation-tuning-v1 (v1.r1 schema layer)` enumerating the 7 kinds + which RT sub-phase each one fires from (r1 declares; r3/r4/r5 fire). Pattern mirrors the jury-mechanics-v1 section appended in v1-JM-a.

### 2.2 Inside-handler step ordering

**RT-r1 has no handler edits.** It is schema + seed + entry-kind consts + backfill ONLY. PRD §15 row 1 dependency table column "Touches handlers": none. Plan §13 must NOT include any task that edits `crates/api/api/src/governance/<handler>.rs` files (other than the dual-file `governance_log` re-export edit and `config.rs` const additions, both of which are non-handler module edits).

The 7 entry-kind consts are **declared but NOT emitted** in RT-r1. Emitters land in RT-r3 (`PARTICIPATION_CRON_TICK`, `VOTE_OUTCOME_RECORDED`, `EVIDENCE_QUALITY_RECORDED`), RT-r4 (`SPONSOR_ALLOWLIST_ADDED`, `SPONSOR_ALLOWLIST_REMOVED`), RT-r5 (`ROLLUP_RECOMPUTED`), RT-r2 (`DECAY_KNOB_CHANGED` — at first decay-config write under v1 calc). Plan §13 documents this mapping in a "Future emission map" subsection (advisory, not a task).

### 2.3 Scope boundary — what is NOT in RT-r1

Per PRD §15 phase table:

- **RT-r2 (next):** Per-dimension chained-halving decay calculator. RT-r1 does NOT modify `compute_applied_delta` or `recompute_snapshot`. The feature flag `feature.reputation_v1_decay_enabled` is seeded as `false` so the v0 calculator continues running until RT-r2 lands the v1 calculator.
- **RT-r3:** Multi-source participation events (weekly cron + dormancy cron + vote-outcome + evidence-quality emitters). RT-r1 declares the entry-kind consts but does NOT add cron registrations or emitter call sites.
- **RT-r4:** Sponsor-gate strategies (`age_or_surety`, `reputation`, `allowlist`) + `sponsor_allowlist` admin endpoints. RT-r1 ships the `sponsor_allowlist` TABLE; RT-r4 ships the admin endpoints (`POST /api/v4/governance/admin/sponsor-allowlist/add` + `/remove`) and the strategy match arms in `create_endorsement`.
- **RT-r5:** Instance-wide rollup cron (`reputation_rollup_cron`) + `GET /api/v4/governance/admin/reputation/rollup` endpoint. RT-r1 reuses `reputation_snapshot` with `community_id IS NULL` (no migration); RT-r5 adds the cron + endpoint.
- **RT-r6:** Carry-forward CodeRabbit fixes (#19, #20, #21, #22, #31). RT-r1 does NOT touch these issues; RT-r6 bundles them at the end of the lane.

**Hard out-of-scope for RT-r1** (per PRD §2 OUT + §15):

- Admin override of reputation events (v2 — ADR-010 step-up auth).
- Per-event-source snapshotting (`applied_delta_snapshot` column) — v2 per PRD §9.1 ADR-010 compliance posture note.
- Composition strategies (`'age_or_surety_or_allowlist'`) — explicitly rejected per OQ-020.

### 2.4 Plan file deliverable shape

Plan file at `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` follows `.claude/PRPs/templates/plan.template.md` verbatim per `feedback_read_canonical_before_writing_spec.md`. Required sections (template-mandated):

- §1 Goal + non-goals
- §2 Stories (§16a — write checkpoint commands so `/brehon-verify` can iterate)
- §3 Risks + mitigations
- §4 Watchpoints — must cite specific tables + files per `feedback_advisor_watchpoint_specificity.md`. Concept-only watchpoints fail the gate.
- §5 Complexity score — per `feedback_complexity_score_pre_split.md`. RT-r1 is schema-heavy (3 migrations) but each migration is mechanical. Estimate: 6–7 (below the 8 split threshold). Planner confirms.
- §13 Task list — DOD-keyed, each task names IMPLEMENT files + per-task validation gate (`cargo check --workspace --features full`, migration round-trip, `cargo test --workspace --test e2e --features full --test parity_seed_count` for the seed parity test).
  - Mark `[P]` per `feedback_parallel_cohort_dispatch.md` only where YAML file arrays prove non-overlap (the three migrations + sponsor_allowlist source file are independent; the `config.rs` const additions + `governance_log.rs` const additions touch shared files and are NOT `[P]`-able).
  - Each task includes a `FILES:` YAML block with `creates:` + `modifies:` arrays per `feedback_explicit_file_arrays_on_tasks.md`.
- §15 DoD validation commands — per `feedback_pre_phase_dod_smoke_test.md`, advisor will run every command literally before plan approval. Each command must be executable on a clean checkout of `phase-v1-RT-r1`. Use `--workspace` (NOT `-p lemmy_server --features full` per `feedback_features_full_p_crate_incompatible.md`).
- §16a Stories with checkpoints — per `feedback_brehon_verify_pre_merge.md`, each story has an `expects:` + `checkpoint:` block so `/brehon-verify` can iterate.

---

## 3. Required reading (in order, before drafting any plan section)

### 3.1 PRD anchors (load first)

1. `.claude/PRPs/prds/v1-reputation-tuning.prd.md` — full PRD. Sections most load-bearing for r1: §5.3 (multi-source participation event schema implications — `dedupe_key`), §5.4 (`sponsor_allowlist` table schema — read line 181 verbatim), §7 (Cross-Cutting Impact: schema changes + backfill rows), §8 (Defaults Matrix — 28 rows + 1 feature flag — copy verbatim into plan §13 task d), §15 row 1 (phase definition + dependency = OQ-018 endpoint).

2. `.claude/PRPs/prds/v1-admin-dashboard.prd.md` §10 (`seed_v1_config_keys` pattern) + §15 §15.5 (`SEEDED_KEYS_WITH_CONSTS.len() == EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD` parity test). RT-r1 extends the cumulative invariant.

3. `.claude/PRPs/plans/phase-v1-JM-a.plan.md` — canonical sibling for "schema-only sub-phase with backfill + entry-kind const additions + parametric `EXPECTED_SEED_COUNT_V1_*`". Read §13 task list + §15 DoD verbatim. RT-r1 mirrors structurally.

4. `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` §3 (phase-by-phase) — RT lane fits at the lane boundary; verify alignment.

### 3.2 ADR + open-question anchors

5. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` ADR-010 (won't-disadvantage rule — RT-r1's deliberate weakening via feature flag instead of snapshot columns; PRD §9.1 cites this).

6. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` ADR-013 (`CaseStatus::EmergencyRemove` posture — RT-r1 does NOT touch CaseStatus, but PRD §7 cites EmergencyRemove for the bad-faith flag endpoint, which is RT-r3 scope, not r1).

7. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` ADR-008 (governance_log append-only — RT-r1 declares 7 new entry-kinds; verify zero `governance_log::entry_kind` schema migration; the column is TEXT per PRD §7).

8. OQ-018 (admin config write endpoint) — verify shipped via v1-AD-b PR #76 merge `f03ed1cba`. RT-r1's seeded keys will be writable through this endpoint at runtime; r1 itself does not call the endpoint.

### 3.3 Mandatory file-class lessons (per advisor-orchestrator.md §2.4 file-class table)

RT-r1 file list (IMPLEMENT files per §2.1 deliverables):

- `crates/db_schema/migrations/{ts}_add_reputation_event_v1_columns/up.sql` + `down.sql`
- `crates/db_schema/migrations/{ts}_create_sponsor_allowlist/up.sql` + `down.sql`
- `crates/db_schema/migrations/{ts}_backfill_reputation_event_source_type/up.sql` + `down.sql`
- `crates/db_schema/src/source/reputation_event.rs` (modify)
- `crates/db_schema/src/source/sponsor_allowlist.rs` (create)
- `crates/db_schema/src/source/governance/governance_log.rs` (modify — 7 new const lines)
- `crates/api/api/src/governance/governance_log.rs` (modify — re-export shim)
- `crates/api/api/src/governance/config.rs` (modify — 28 new `DEFAULT_*` consts)
- `crates/db_schema/src/utils/v1_rt_config_seed.rs` (create)
- `crates/db_schema/src/newtypes.rs` (modify — `SponsorAllowlistId` newtype)
- `crates/db_schema/src/schema.rs` (regenerated)
- `.claude/rules/governance-log-entry-kind-registry.md` (modify — append RT section)
- `crates/server/tests/e2e.rs` (modify — extend `phase1_migrations_round_trip` parametric matcher; per-test anchor-Edit at file end if a separate parity test is added; e2e.rs is **12,819 lines** post-SL-c-2 — anchor-Edit discipline mandatory per `feedback_junior_worker_e2e_edit_hang.md`).

Per the §2.4 file-class table:

| Pattern matched | Mandatory lesson(s) |
|---|---|
| `crates/db_schema/migrations/**` (3 new migrations) | `feedback_lemmy_migration_runner.md`, `feedback_postgres_jsonb_canonicalization.md` (no JSONB expected; planner verifies) |
| `crates/server/tests/e2e.rs` (any edit) | `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md`. **When a v1-SL-* or v1-JM-* fixtures sibling module already exists, mirror its error-shape case verbatim** (Case A or B per lesson). Pick by reading sibling at the cited line range BEFORE authoring. |
| `crates/server/tests/e2e.rs` (≥2 edits in this task or cohort) | + `feedback_junior_worker_e2e_edit_hang.md` |
| Any newtype under `crates/db_schema/src/newtypes/` | `feedback_newtype_locations_lemmy_db_schema_vs_file.md` |

Plan body must cite these lessons in §3 Required reading and reflect them in §4 Watchpoints. The §2.3 hybrid PMD search runs after this table check (per `advisor-orchestrator.md` §2.3) — catches non-mechanical / cross-cutting lessons. Planner runs `memory_search_hybrid(query: "reputation_event migration backfill", tags: "lesson")` once; cites hits.

### 3.4 Plan-template + canonical-sibling reads

9. `.claude/PRPs/templates/plan.template.md` — section structure verbatim.

10. `.claude/PRPs/plans/phase-v1-JM-a.plan.md` — canonical sibling. Mirror task-list shape, §15 DoD shape, §16a Stories shape.

11. `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — second canonical sibling for "first sub-phase of a v1 lane authoring schema-only deliverables behind a feature flag". Mirror the feature-flag seeding posture (RT-r1 seeds `feature.reputation_v1_decay_enabled = false` at the same migration; SL-a seeded `feature.sponsor_liability_grace_window_enabled = false` at its first migration — pattern is identical).

### 3.5 PMD-promoted patterns (per `pattern_*.md` corpus — load on demand)

- `pattern_cargo_feature_flag_propagation.md` — `--features full` discipline; `--workspace` over `-p`; lockfile.
- `pattern_spec_schema_co_commit.md` — schema and parity-test edits land in the same commit/task.
- `pattern_junior_pre_queue_discipline.md` — 6-gate: git/lessons/mirror/memory/cron/PMD before queueing the impl tasks (downstream of this brief).
- `pattern_context_is_finite.md` — RT-r1 plan likely 700–900 lines (similar to phase-v1-JM-a.plan.md's 1558 lines); plan task wins memory budget at impl time.

---

## 4. Constraints (hard rules — violating any of these is a process breach)

### 4.1 Plan-content discipline

a. **PRD-aligned narrow scope only.** RT-r1 = schema + seed + backfill. Any temptation to bundle r2's calculator or r4's admin endpoints "while we're in the file" is OUT — split rejection per `feedback_complexity_score_pre_split.md` and PRD §15 dependency chain.

b. **No handler edits.** RT-r1's only file edits in `crates/api/api/src/` are: (1) `config.rs` const additions, (2) `governance_log.rs` re-export shim. Both are non-handler module edits. Plan §13 must NOT name `submit_jury_vote.rs`, `create_endorsement.rs`, `apply_sponsor_liability.rs`, or any `crates/api/api/src/governance/<handler>.rs` file in any task's IMPLEMENT files.

c. **§8 Defaults Matrix is the contract.** Plan §13 task d (seed v1 config keys) copies the 28-row table verbatim from PRD §8. Drift is a process miss; if the planner finds a typo or missing range, file a planner DQ asking for PRD revision rather than silently fix.

d. **Schema-first discipline.** Migrations in plan §13 must precede the diesel struct edits in `reputation_event.rs` (the struct must reflect what the migration ships). `schema.rs` regen runs after migrations apply. Per `feedback_lemmy_migration_runner.md`.

### 4.2 Decision-queue discipline (per `.claude/rules/decision-queue.md`)

a. **Planner writes `kind: "blocker"` from `from: "planner"`** with `answered_by: null` for genuine blockers.

b. **Planner may pre-seed `answered_by: "planner"` with citation** for non-blocking design choices the PRD already addresses (e.g. backfill heuristic precedence — cite PRD §7 Backfill row).

c. **Planner does NOT write `answered_by: "advisor"` or `answered_by: "user"`.** Attribution-integrity rule (per decision-queue.md §Attribution integrity).

d. **Mid-task push discipline** — every DQ write commits + pushes immediately on the worker branch (`junior/rt-r1-planning-1`) per decision-queue.md "Mid-task visibility" section. Without the push, the advisor cannot see the entry on its next poll.

e. **Schema-changing-spec retrofit gate (per advisor-orchestrator.md §3.8)** — RT-r1 is purely additive (new migrations, new keys, new entry-kinds; no shape changes to existing artifacts). Skip the retrofit AskUserQuestion gate per §3.8 "Skip when: purely additive functionality".

### 4.3 File ownership

Planner writes ONLY:
- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` (the single deliverable)
- `.claude/decision-queue.json` (DQ entries — pending or planner-self-resolved)

Planner does NOT write:
- Any file under `crates/**` (impl-task work — planner authors plan, impl-task authors code)
- Any file under `migrations/**` (impl-task work)
- Any other plan file or brief (out of scope; one brief = one plan)
- `docs/brehon-law-inspired-network/**` (design-doc territory)
- `.claude/lessons/**` (lessons authored by user / advisor / retro)

### 4.4 Schema-first discipline

Plan §13 task ordering must respect schema-first dependency chain:

1. Migration up.sql + down.sql authored
2. Diesel `schema.rs` regenerated
3. `crates/db_schema/src/source/reputation_event.rs` struct field additions
4. `crates/db_schema/src/source/sponsor_allowlist.rs` new file
5. Seed function (`seed_v1_rt_config_keys`) authored — depends on `DEFAULT_*` consts
6. `DEFAULT_*` consts in `config.rs`
7. Entry-kind consts in `governance_log.rs` (db_schema source) + re-export in api shim
8. Round-trip migration test extension in `e2e.rs`

(Strict ordering not required across all tasks — `[P]` parallelism marks where YAML file arrays prove non-overlap. Migrations 1, 2, 3 are independent files — `[P]`-able. Steps 5–8 share `config.rs` / `governance_log.rs` / `e2e.rs` and are NOT `[P]`.)

### 4.5 Cross-cutting from PMD-promoted patterns

a. `pattern_cargo_feature_flag_propagation.md` — every cargo command in §15 DoD uses `--workspace` + `--features full`. Plan §15 DoD must NOT use `-p lemmy_server --features full`.

b. `pattern_spec_schema_co_commit.md` — migration + diesel struct + parity test land in the same commit (one task per migration, with all three file types in the task's `creates:` + `modifies:` array).

c. `pattern_context_is_finite.md` — plan body should fit ~800 lines max. Avoid copying entire PRD sections verbatim; cite by anchor (`per PRD §8 row 5`) and copy only contracts (the 28-row Defaults Matrix is a contract; copy verbatim).

### 4.6 Attribution integrity reminder

Every commit subject on the planning worker branch matches the planner-attribution pattern: `chore(plan): rt-r1 — <slug>` or `feat(plan): rt-r1 — <slug>`. Subjects matching `^(chore|docs)\((advisor|decision-queue)\)` are advisor-only — planner must not author them. Per decision-queue.md §Detection.

### 4.7 Forbidden execution windows

Planner is dispatched as a Junior task; impl-task pre-flight refuses to start in a forbidden window per advisor-orchestrator.md §5.1. The advisor's pre-queue check applies at dispatch time.

---

## 5. Pre-commit dogfood (per advisor-orchestrator.md §3.7)

This brief was walked through against:
- `.claude/PRPs/prds/v1-reputation-tuning.prd.md` §15 row 1 (parsed), §7 (parsed schema additions row), §8 (parsed all 28 knob rows), §10 (feature flag flagged).
- `.claude/PRPs/briefs/sl-d-planning-1.md` (canonical sibling — same brief shape, mirrored §1–§5 structure).
- `.claude/PRPs/plans/phase-v1-JM-a.plan.md` (canonical sibling for schema-only sub-phase plan — informs plan §13 expected shape downstream).

What worked:
- PRD §15 row 1 is unambiguous on scope (`Add dedupe_key, source_event_type columns; add sponsor_allowlist table; seed 28 new config rows + feature flag; backfill existing reputation_event rows`). No Hypothesis A/B scope question; no clarify gate ambiguity expected at PRD level.
- §8 Defaults Matrix is contract-shaped (per-knob namespace + type + default + range + per-community? + citation columns). Copy-paste verbatim into plan §13 task d.
- Concurrency analysis vs SL-d (parallel-running planning task): zero file overlap on handler files; only theoretical contention is `e2e.rs` anchor-Edit collision, addressable per cohort discipline.

What didn't (and is captured as expected planner DQ entries):
- PRD §7 Backfill row says `source_event_type default-mapped from existing reason/source columns` but doesn't enumerate the heuristic patterns. Brief §2.1c lists planner's starting heuristics; planner files DQ if any of the four heuristics ship pseudocode that PRD §7 doesn't authorise.
- PRD doesn't specify `EXPECTED_SEED_COUNT_V1_RT` exact value; brief asserts 29 (28 knobs + 1 feature flag). Planner confirms by counting §8 table rows.

---

## 6. Acceptance for this brief

Brief is queueable when:
- DQ pending count = 0 OR all pending entries are non-blocking for RT-r1 planning
- SL-d planning task status is **observable** (still running, complete, or failed) — RT-r1 planning may run concurrently; advisor handles the cohort mechanics at dispatch time
- Forbidden-window check at dispatch time per advisor-orchestrator.md §5.1
- Clarify gate completed per advisor-orchestrator.md §3.3 (this brief is the PRE-clarify input; `/brehon-clarify .claude/PRPs/briefs/rt-r1-planning-1.md` runs next)

Brief is committed to `governance-v0` with subject:
```
chore(advisor): rt-r1-planning brief — schema + seed + backfill (parallel-safe to SL-d)
```
