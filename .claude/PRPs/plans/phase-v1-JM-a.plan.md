# Plan: Phase v1-JM-a — Jury-mechanics schema + enums + snapshot columns + backfill

## 1. Summary

v1-JM-a is the **foundation sub-phase** of the v1 jury-mechanics keystone. It ships the schema + config substrate for the rest of JM (b/c/d/e) and nothing else — no handler edits, no route wiring, no selection-algorithm work, no appeals logic. Concretely it lands:

1. Three new Postgres enums (`severity_tier`, `case_status_tier`, `jury_assignment_role`) in their own pre-ALTER migration so the downstream ALTERs can reference them
2. Six new columns on `moderation_case` (`severity_tier`, `status_tier`, `panel_size_snapshot`, `quorum_snapshot`, `threshold_count_snapshot`, `appeal_window_expires_at`) per PRD §8.1
3. Two new columns on `jury_assignment` (`selected_under_constraints` JSONB + `role jury_assignment_role`) per PRD §8.2
4. New table `jury_constraint_violation_log` per PRD §8.3 with `idx_jcvl_case_id`
5. v0→v1 backfill migration (PRD §8.4) — writes Minor/Regular/5/3/3 snapshots on every pre-v1 case plus `appeal_window_expires_at = COALESCE(closed_at, decided_at + 7d)`
6. `EXPECTED_SEED_COUNT_V1_JM: usize = 27` parametric const (following v1-AD-a §4.1 precedent) alongside the existing `EXPECTED_SEED_COUNT` (34) and `EXPECTED_SEED_COUNT_V1_AD` (27)
7. 27 net-new `jury.*` + `appeal.*` config keys seeded via a dedicated `seed_v1_jm_config_keys` migration, with matching `DEFAULT_*` consts, `const_default_*` match arms, `SEEDED_KEYS_WITH_CONSTS` entries, and `CONFIG_KEY_METADATA` entries
8. Six new `ENTRY_KIND_*` consts per PRD §8.5 (dual-file edit: DEFINE in `crates/db_schema/src/source/governance/governance_log.rs`, RE-EXPORT in `crates/api/api/src/governance/governance_log.rs`) — `jury_constraint_relaxed`, `appeal_panel_assembled`, `appeal_decided`, `appeal_rejected`, `appeal_window_expired`, `severity_tier_frozen`
9. `jury-mechanics-v1` section populated in `.claude/rules/governance-log-entry-kind-registry.md` (replacing the reserved stub) and new `RuleSetVersionId`-style newtype for `JuryConstraintViolationLogId`

No HTTP routes, no new handler files, no DTO changes, no `select_eligible_jurors` edits. Those ship in v1-JM-b/c/d/e.

## 2. Source

- [v1-jury-mechanics.prd.md §17 Implementation Phases](../prds/v1-jury-mechanics.prd.md) row 1 (v1-JM-a) + §8 Database & migration changes + §8.5 entry-kinds + §10 defaults matrix
- [IMPLEMENTATION-PLAN-v0.md §3 Phase-tree](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) — v1 sequencing (post-v0 governance mechanics block)
- [99-decisions-and-open-questions.md ADR-007, ADR-010, ADR-013, ADR-015](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) — jury-baseline / staged-release / emergency-remove / pseudonymity
- [01-vision-and-principles.md §5.6 + §5.7](../../../docs/brehon-law-inspired-network/01-vision-and-principles.md) — v1 voting thresholds + appeal panel sizing (canonical default source for §10 matrix)
- Precedent: [v1-admin-dashboard-a.plan.md](./completed/v1-admin-dashboard-a.plan.md) — `EXPECTED_SEED_COUNT_V1_*` pattern (§4.1), parity-test extension (§10.7), dual-file entry-kind edit (§10.8), migration reconciliation gate (§13 Task 7)

## 3. Problem statement

The v1 jury mechanics in PRD §§3–7 depend on a set of **snapshot columns** on `moderation_case` and `jury_assignment` that v0 does not carry. Every subsequent JM sub-phase (b/c/d/e) reads or writes these columns:

- v1-JM-b (`admin_assign_jury` cascade) writes `panel_size_snapshot`, `quorum_snapshot`, `threshold_count_snapshot`, `selected_under_constraints`
- v1-JM-c (`submit_jury_vote` 9-step handler) reads the quorum/threshold snapshots (ADR-010 no-retroactive-invalidation) and writes `appeal_window_expires_at`
- v1-JM-d (appeals) reads `appeal_window_expires_at` and writes `role = Appeal` jury_assignment rows
- v1-JM-e capstone asserts the `v0_case_completes_under_v0_rules_after_v1_config_flip` test (§11) which requires the backfill migration to have set Minor/Regular/5/3/3 on every pre-v1 case

Without JM-a's schema substrate + backfill, b/c/d/e have nothing to read or write against. JM-a also owns the config-key seeding + entry-kind-const land for the rest of JM, so b/c/d/e need only reference the consts without touching `config.rs` parity scaffolding or the dual-file log-kind registry.

Per PRD §17.2 (feature-flag posture), **JM-a cannot be dark-launched**: the backfill writes default snapshot values the moment it lands, and v1-JM-c's `submit_jury_vote` reads those snapshots as soon as it merges. There is no "dormant schema" state, so JM-a's PR review gate is stricter than v1-AD-a's — the `phase1_migrations_round_trip` test must extend LIFO round-trip coverage over JM-a's new migrations, AND the backfill must pass a smoke test asserting Minor/Regular/5/3/3 on seeded pre-v1 cases.

## 4. Solution statement

Follow the **v1-AD-a precedent** byte-for-byte where possible. The architecturally-load-bearing decisions locked in this sub-phase:

### 4.1 Lock-in decisions

- **Three new Postgres enums in their own pre-ALTER migration.** `severity_tier`, `case_status_tier`, `jury_assignment_role` are declared via `CREATE TYPE ... AS ENUM` in migration `2026-04-23-000000-0000_add_jury_mechanics_enums/up.sql` (mirroring Phase 1's `2026-04-15-100000-0000_add_governance_enums` shape per Explore agent finding §2). The ALTERs that reference these enum types must land in a later timestamp so the enum type exists first. This is the same precedent pattern as the Phase 5a `add_person_membership_state` migration where `CREATE TYPE membership_state ...` preceded the `ALTER TABLE person ADD COLUMN membership_state`.
- **`DbValueStyle = "verbatim"` for all three new enums.** Explore agent §1 finding: `CaseStatus` / `JuryDecision` / `SanctionAction` all use `verbatim` (PascalCase variant names); `MembershipState` is the lone exception (snake_case for deferred-enforcement per that enum's doc comment). JM's three enums behave like `CaseStatus` — they're read/written by governance code, not by shell-script config writers — so they match the majority pattern.
- **Snapshot columns land NULL-nullable, backfill populates synchronously in the same migration set.** Per PRD §8.4 + §11, pre-v1 cases must see `panel_size_snapshot=5, quorum_snapshot=3, threshold_count_snapshot=3, severity_tier='Minor', status_tier='Regular', appeal_window_expires_at = COALESCE(closed_at, decided_at + 7d)`. The backfill is **part of the ALTER migration's up.sql** (single transaction) rather than a separate migration, so there is no intermediate state in which a row has the new columns but null snapshot values that v1-JM-c could misread. The enum columns themselves ship `NOT NULL DEFAULT 'Minor'/'Regular'` (fast metadata-only backfill per Postgres 11+, per Explore agent §5 — same technique as `ADD COLUMN membership_state NOT NULL DEFAULT 'member'`); the integer snapshot columns ship NULLABLE and are populated by an UPDATE inside the same up.sql.
- **`EXPECTED_SEED_COUNT_V1_JM: usize = 27`** — adds beside `EXPECTED_SEED_COUNT_V1_AD` without churning it. The three parity tests (`seeded_keys_count_matches_const_count`, `every_seeded_key_has_metadata`, `every_seeded_key_has_const_fallback`) extend to `EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD + EXPECTED_SEED_COUNT_V1_JM` per the advisor directive #4 precedent. The count **27 is a plan-time best estimate**; Task 8's pre-commit reconciliation gate (mirroring v1-AD-a Task 7 §advisor-edit-#1) is authoritative against the actual count of rows written to `SEEDED_KEYS_WITH_CONSTS` + `up.sql`.
- **Dual-file edit for the 6 new `ENTRY_KIND_*` consts** (per Explore agent §3/§4 finding + registry's explicit note at the top of `.claude/rules/governance-log-entry-kind-registry.md`): DEFINE in `crates/db_schema/src/source/governance/governance_log.rs` (append after `ENTRY_KIND_RULE_SET_VERSION_CREATED` which closed v1-AD-c), RE-EXPORT alphabetically in `crates/api/api/src/governance/governance_log.rs`'s `pub use` block. Single atomic commit across both files.
- **Registry section populated, not just reserved.** The `.claude/rules/governance-log-entry-kind-registry.md` already reserves a `jury-mechanics-v1` section. Task 9 replaces the reservation stub with the six populated rows (matching the v1-AD-c / v1-AD-a populated-section shape), bumping the acceptance-invariants count from 26 → 32.
- **No handler edits in this sub-phase.** `admin_assign_jury.rs`, `submit_jury_vote.rs`, `request_appeal.rs`, `accept_jury_assignment.rs`, `decline_jury_assignment.rs`, `admin_emergency_remove.rs` are untouched. A v1-JM-a session that finds itself editing any of these files has drifted scope and must STOP per decision-queue.md attribution discipline.

### 4.2 Rejected alternatives

- **Backfill in a separate migration after the ALTER.** Rejected: leaves a window (however brief) in which v1-JM-c's submit_jury_vote could read NULL snapshot values and crash. Single-transaction up.sql eliminates the window.
- **Ship all 27 keys as `scope = 'community'` by default.** Rejected: v1-AD-a seed migration set everything to `scope = 'instance'` (Explore agent §2 finding). Community overrides are per-community writes that happen at dashboard-edit time, not at seed time. Keep parity.
- **Merge `EXPECTED_SEED_COUNT_V1_JM` into `EXPECTED_SEED_COUNT_V1_AD`** to simplify the parity test. Rejected per advisor directive #4 (v1-AD-a §4.1): each sub-PRD's const stays pinned, only `SEEDED_KEYS_WITH_CONSTS.len()` aggregates. Keeps future sub-PRDs (SL, RT, FI) orthogonal.
- **Use `DbValueStyle = "snake_case"`** for the three new enums to match the seeded config text values. Rejected: MembershipState's snake_case is a **deferred-enforcement exception** (documented in its doc comment); governance enums that are read/written by handler code use `verbatim` per the majority pattern. Consistency matters more than one namespace's snake-case aesthetic.

## 5. Metadata

| Field | Value |
|---|---|
| Type | SCHEMA |
| Complexity | MEDIUM |
| Crates Affected | `lemmy_db_schema`, `lemmy_db_schema_file`, `lemmy_api` |
| v1 Step | v1-JM sub-phase A (per PRD §17 phase split row 1) |
| Dependencies | v1-AD-a merged (`EXPECTED_SEED_COUNT_V1_AD` + `CONFIG_KEY_METADATA` registry shipped at `602b45e56`) |
| Estimated Tasks | 11 (including Task 0 pre-flight and Task 10 retro) |
| PR count | 1 (`phase-v1-JM-a` → `governance-v0` via `gh pr create --repo barrie-cork/lemmy`) |
| CodeRabbit review | mandatory (non-draft PR per `.claude/rules/phase-branch.md`) |

## 6. Relationship to other v1 sub-phases

| Sub-phase | Depends on | Can parallelise with |
|---|---|---|
| **v1-JM-a** (this plan) | v1-AD-a merged | v1-rep-tuning-r1 (different tables, different migrations, zero file overlap) |
| v1-JM-b | v1-JM-a merged | — |
| v1-JM-c | v1-JM-b merged | — |
| v1-JM-d | v1-JM-c merged | v1-SL-d, v1-rep-tuning-r3/r4/r5 (per PRD §17.1) |
| v1-JM-e | v1-JM-d merged | — |

JM-a **must merge** before any of b/c/d/e start. The plan file and migration timestamps encode the assumption that JM-a is solo on `governance-v0` head at merge time; the v1-planning-queue and advisor confirm no concurrent PR is modifying `moderation_case` schema or `config.rs` seeding scaffolding.

## 7. Preflight guardrails inherited from v1-AD-d retro

Per PRD §17.4, four DQs (#42, #43, #44, #46) came out of the v1-AD-d retrospective. All four were resolved at the `/prp-core:prp-implement` command-template level and are already in place as of 2026-04-23. This plan **cites the DQ IDs for traceability only** — no inline compensating checks are needed because the tool-level fixes handle them automatically:

- **DQ #42 (task-resume safety)** — `/prp-core:prp-implement` §1.4 detects already-completed tasks by matching commit subjects against plan §13 COMMIT MESSAGE lines. Protects JM-a on resumed sessions.
- **DQ #43 (HTTP status code audit)** — not applicable to JM-a (no HTTP handlers in scope).
- **DQ #44 (Docker daemon preflight)** — `docker ps` probe in both `pre-phase-harness-audit.md` Probe 0 AND `/prp-core:prp-implement` §4.2.0 (belt-and-braces).
- **DQ #46 (v1/limitation GH issue capture)** — `/prp-core:prp-implement` Phase 5 REPORT template prompts for `v2-candidate` / `v1.5-candidate` GH issue sketches. JM-a has 3 identifiable candidates in PRD §2 OUT: cross-instance jury, composable constraints, `no_same_endorsement_chain`.

If any of these tool-level fixes is later reverted or superseded, JM-a's plan flips to "inline compensation required" for the affected DQ.

---

## 8. Flow design

### Before state (end of v1-AD-a merge; `governance-v0` HEAD `02189988d` post-meta-retro)

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                              BEFORE STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║   moderation_case (18 cols):                                                  ║
║     ... severity CaseSeverity (Low|Medium|High|Critical)                      ║
║     ... status CaseStatus                                                     ║
║     ... applied_config_snapshot JSONB    (v1-AD-a)                            ║
║     ... rule_set_version_id INT4 FK      (v1-AD-a)                            ║
║                                                                               ║
║   jury_assignment (7 cols):                                                   ║
║     id, case_id, person_id, status, selected_at, responded_at, submitted_at   ║
║                                                                               ║
║   submit_jury_vote.rs:                                                        ║
║     const QUORUM: i64 = 3;                                                    ║
║     const APPEAL_WINDOW_DAYS: i64 = 7;                                        ║
║     closed_at = decided_at + 7d (HARDCODED)                                   ║
║                                                                               ║
║   admin_assign_jury.rs:120                                                    ║
║     config::get_int(..., "jury.panel_size", 5).await?                         ║
║                                                                               ║
║   request_appeal.rs:112                                                       ║
║     case.closed_at > now()   (APPEAL WINDOW = closed_at sentinel)             ║
║                                                                               ║
║   config.rs:                                                                  ║
║     EXPECTED_SEED_COUNT = 34 (v0)                                             ║
║     EXPECTED_SEED_COUNT_V1_AD = 27 (AD-a)                                     ║
║     SEEDED_KEYS_WITH_CONSTS: 61 tuples                                        ║
║     CONFIG_KEY_METADATA: 61 entries                                           ║
║                                                                               ║
║   governance_log.rs (db_schema DEFINE + api SHIM RE-EXPORT):                  ║
║     26 ENTRY_KIND_* consts  (19 v0 + 4 Phase 6 + 2 AD-a + 1 AD-c)             ║
║                                                                               ║
║   .claude/rules/governance-log-entry-kind-registry.md:                        ║
║     jury-mechanics-v1 section RESERVED (no rows populated)                    ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### After state (end of v1-JM-a)

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                               AFTER STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║   moderation_case (24 cols — 18 + 6):                                         ║
║     ... severity CaseSeverity          (v0 — unchanged; NOT replaced)         ║
║     ... status CaseStatus              (v0 — unchanged)                       ║
║     ... severity_tier severity_tier    NOT NULL DEFAULT 'Minor'  (JM-a)       ║
║     ... status_tier case_status_tier   NOT NULL DEFAULT 'Regular' (JM-a)      ║
║     ... panel_size_snapshot INT4       NULL (populated by admin_assign_jury) ║
║     ... quorum_snapshot INT4           NULL                                   ║
║     ... threshold_count_snapshot INT4  NULL                                   ║
║     ... appeal_window_expires_at TIMESTAMPTZ NULL (set by submit_jury_vote)  ║
║   (backfill UPDATE: pre-v1 cases get Minor/Regular/5/3/3 + window from decided)║
║                                                                               ║
║   jury_assignment (9 cols — 7 + 2):                                           ║
║     ... selected_under_constraints JSONB NULL (written by admin_assign_jury) ║
║     ... role jury_assignment_role  NOT NULL DEFAULT 'Original' (JM-a)         ║
║                                                                               ║
║   jury_constraint_violation_log (new table, 6 cols + idx_jcvl_case_id):       ║
║     id, case_id, constraint_name, relaxation_reason, pool_size_at_relax,      ║
║     panel_size_target, relaxed_at                                             ║
║                                                                               ║
║   submit_jury_vote.rs / admin_assign_jury.rs / request_appeal.rs:             ║
║     UNCHANGED (handler edits land in JM-b/c/d)                                ║
║                                                                               ║
║   config.rs:                                                                  ║
║     EXPECTED_SEED_COUNT = 34 (v0 — unchanged)                                 ║
║     EXPECTED_SEED_COUNT_V1_AD = 27 (AD-a — unchanged)                         ║
║     EXPECTED_SEED_COUNT_V1_JM = 27 (this sub-phase — parametric)              ║
║     SEEDED_KEYS_WITH_CONSTS: 88 tuples (61 + 27)                              ║
║     CONFIG_KEY_METADATA: 88 entries (61 + 27)                                 ║
║     +27 DEFAULT_JURY_* / DEFAULT_APPEAL_* consts                              ║
║     +27 matching const_default_{int|float|bool|text} match arms               ║
║                                                                               ║
║   governance_log.rs (db_schema DEFINE + api SHIM RE-EXPORT):                  ║
║     32 ENTRY_KIND_* consts (26 + 6 new JM-a kinds, both files in sync)        ║
║                                                                               ║
║   .claude/rules/governance-log-entry-kind-registry.md:                        ║
║     jury-mechanics-v1 section POPULATED with 6 rows                           ║
║     Acceptance invariants count: 26 → 32                                      ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### Data flow — no HTTP surface changes in v1-JM-a

| Endpoint | Before | After |
|---|---|---|
| `POST /api/v4/governance/admin/assign-jury` | reads `jury.panel_size`, writes `jury_assignment` | **unchanged** (JM-b edits this in next sub-phase) |
| `POST /api/v4/governance/submit-jury-vote` | hardcoded QUORUM=3 + closed_at sentinel | **unchanged** (JM-c) |
| `POST /api/v4/governance/request-appeal` | `closed_at.is_some()` guard | **unchanged** (JM-d) |

---

## 9. Mandatory reading

The implementation agent MUST read these at first iteration before any file edit.

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `.claude/PRPs/prds/v1-jury-mechanics.prd.md` | §8.1, §8.2, §8.3, §8.4, §8.5, §10, §17 | Canonical schema, backfill, and config-key enumeration |
| P0 | `.claude/PRPs/plans/completed/v1-admin-dashboard-a.plan.md` | §4.1, §10.1–§10.8, Task 6, Task 7, Task 8 | Byte-accurate precedent for `EXPECTED_SEED_COUNT_V1_*`, parity test extension, migration reconciliation gate, dual-file entry-kind edit |
| P0 | `.claude/rules/governance-log-entry-kind-registry.md` | full | Registry invariants + dual-file rule + `jury-mechanics-v1` reservation section to populate |
| P0 | `crates/api/api/src/governance/config.rs` | 113–123 (struct), 820–924 (`SEEDED_KEYS_WITH_CONSTS`), 929–938 (`EXPECTED_SEED_COUNT*`), 1700–1798 (parity tests) | Extension site for all config scaffolding |
| P0 | `crates/db_schema_file/src/enums.rs` | 382–408 (`CaseStatus`), 488–509 (`JuryDecision`), 570–575 (`AppealStatus`), 624–648 (`MembershipState` — different style) | Enum declaration pattern. MIRROR CaseStatus/JuryDecision (verbatim), NOT MembershipState (snake_case is a deferred-enforcement exception) |
| P0 | `migrations/2026-04-15-100000-0000_add_governance_enums/up.sql` | 1–89 (full file) | CREATE TYPE ... AS ENUM shape |
| P0 | `migrations/2026-04-15-100000-0000_add_governance_enums/down.sql` | 1–11 | DROP TYPE LIFO order |
| P0 | `migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql` | 1–54 | v1-AD-a seed migration — byte-for-byte precedent for the INSERT shape |
| P0 | `migrations/2026-04-22-000300-0000_seed_v1_config_keys/down.sql` | 1–40 | DELETE WHERE-IN shape |
| P0 | `migrations/2026-04-22-000200-0000_add_case_applied_config_snapshot/up.sql` | 1–25 | Most recent `moderation_case` ALTER precedent |
| P0 | `migrations/2026-04-18-000100-0000_add_person_membership_state/up.sql` | 1–15 | Fast metadata-only `ADD COLUMN NOT NULL DEFAULT` precedent (Postgres 11+ attmissingval) |
| P0 | `crates/db_schema/src/source/governance/moderation_case.rs` | 1–68 | Current Queryable + InsertForm — extend shape |
| P0 | `crates/db_schema/src/source/governance/jury_assignment.rs` | 1–35 | Current Queryable + InsertForm — extend shape |
| P0 | `crates/db_schema/src/newtypes.rs` | 214–316 | `pub struct ModerationCaseId(pub i32);` precedent — mirror for `JuryConstraintViolationLogId` |
| P0 | `crates/server/tests/e2e.rs` | 306–454 (`phase1_migrations_round_trip`), 1361–1418 (`config_parity_round_trip`) | Both tests extend with new tables/enums/keys |
| P1 | `.claude/rules/phase-branch.md` | 5–15 | Task 0 branch-assertion gate |
| P1 | `.claude/rules/cargo-output-capture.md` + `no-cargo-output-paste.md` | full | Every cargo invocation redirects output to a file; `tail -20` only |
| P1 | `.claude/rules/pre-phase-harness-audit.md` | full (probes 0–4) | Pre-flight audit runs before Task 1 |
| P1 | `.claude/rules/decision-queue.md` | full | Any blocking decision goes here, NEVER self-resolved with `answered_by: "advisor"` |
| P1 | `.claude/rules/pm-plugin-hooks-stable.md` | full | Task 10 belt-and-braces check that none of the seven PM hook literals regressed |
| P1 | `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` | ADR-007 (jury baseline), ADR-010 (staged releases), ADR-013 (EmergencyRemove), ADR-015 (pseudonymity) | Contradiction-check surface for §12 acceptance |

### External documentation

Only the Explore agents confirmed existing workspace versions are needed. No new crates are introduced.

| Source | Version (from `Cargo.toml` L114–223) | Section | Why |
|---|---|---|---|
| [diesel](https://docs.rs/diesel/2.3.7) | 2.3.7 | `diesel::table!` macro, `ALTER TABLE ... ADD COLUMN` SQL in migrations | Already in workspace |
| [diesel-derive-enum](https://docs.rs/diesel-derive-enum/2.1.0) | 2.1.0 | `DbEnum` derive + `ExistingTypePath` + `DbValueStyle = "verbatim"` | Already in workspace |
| [chrono](https://docs.rs/chrono/0.4.44) | 0.4.44 | `DateTime<Utc>` for `appeal_window_expires_at` | Already in workspace |
| [serde_json](https://docs.rs/serde_json/1.0.149) | 1.0.149 | `Value` for `selected_under_constraints JSONB` | Already in workspace |

---

## 10. Patterns to mirror

### 10.1 New Postgres enum migration shape (pre-ALTER)

**SOURCE:** `migrations/2026-04-15-100000-0000_add_governance_enums/up.sql` (full file, 89 lines) + `down.sql` (LIFO drop order)

**COPY THIS PATTERN for Task 1** (file: `migrations/2026-04-23-000000-0000_add_jury_mechanics_enums/up.sql`):

```sql
CREATE TYPE severity_tier AS ENUM (
    'Minor',
    'Moderate',
    'Severe'
);

CREATE TYPE case_status_tier AS ENUM (
    'Founder',
    'Regular',
    'Probation'
);

CREATE TYPE jury_assignment_role AS ENUM (
    'Original',
    'Appeal'
);
```

```sql
-- down.sql — LIFO order (mirrors Phase 1 precedent)
DROP TYPE jury_assignment_role;
DROP TYPE case_status_tier;
DROP TYPE severity_tier;
```

**GOTCHA:** Variants use PascalCase. The matching Rust enum (Task 3) uses `DbValueStyle = "verbatim"` so the Rust-side names match the Postgres-side values 1:1. Do NOT lowercase either side.

### 10.2 Diesel enum Rust pattern

**SOURCE:** `crates/db_schema_file/src/enums.rs:382–408` (`CaseStatus`)

**COPY THIS PATTERN for Task 3:**

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::SeverityTier"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// v1 jury-mechanics severity tier. Maps `SanctionAction` to procedural
/// threshold per PRD §3.1. Frozen at admin_assign_jury time per ADR-010
/// (no retroactive invalidation of in-flight juries).
pub enum SeverityTier {
  #[default]
  Minor,
  Moderate,
  Severe,
}
```

Three parallel declarations: `SeverityTier`, `CaseStatusTier`, `JuryAssignmentRole`. All three use `DbValueStyle = "verbatim"` per §4.1 lock-in.

### 10.3 schema.rs sql_types module entry

**SOURCE:** `crates/db_schema_file/src/schema.rs` lines 3–119 (`sql_types` module). Each Postgres enum type declared in migrations gets a matching struct.

**COPY THIS PATTERN for Task 4:**

```rust
pub mod sql_types {
  // ... existing entries including CaseStatus, JuryDecision, SanctionAction, AppealStatus,
  //     MembershipState, etc. ...

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "severity_tier"))]
  pub struct SeverityTier;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "case_status_tier"))]
  pub struct CaseStatusTier;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "jury_assignment_role"))]
  pub struct JuryAssignmentRole;
}
```

### 10.4 ALTER TABLE ADD COLUMN + backfill (single transaction)

**SOURCE:** `migrations/2026-04-22-000200-0000_add_case_applied_config_snapshot/up.sql` (JSONB + FK shape) + `migrations/2026-04-18-000100-0000_add_person_membership_state/up.sql` (fast-metadata `ADD COLUMN NOT NULL DEFAULT`)

**COPY THIS COMPOSED PATTERN for Task 5:**

```sql
-- migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql

-- v1-JM-a: add 6 new columns to moderation_case + 2 to jury_assignment,
-- then backfill pre-v1 cases with v0-equivalent snapshots per PRD §8.4.
-- Single-transaction by design — no intermediate state where v1-JM-c's
-- submit_jury_vote could read NULL snapshot values on a pre-v1 case.
--
-- This migration is ADDITIVE (ADD COLUMN only, no DROP, no destructive
-- UPDATE-in-place on existing columns). The ".coderabbit.yaml" protected
-- governance-table rule targets destructive changes; additive columns +
-- a one-shot backfill of newly-nullable data are in scope.

-- Enum-typed columns use fast metadata-only attmissingval backfill (Postgres 11+)
ALTER TABLE moderation_case ADD COLUMN severity_tier severity_tier NOT NULL DEFAULT 'Minor';
ALTER TABLE moderation_case ADD COLUMN status_tier case_status_tier NOT NULL DEFAULT 'Regular';

-- Integer snapshot columns — NULLABLE because v1 cases populate them at
-- admin_assign_jury time; the backfill UPDATE (below) writes 5/3/3 to
-- every existing case (pre-v1 at migration time).
ALTER TABLE moderation_case ADD COLUMN panel_size_snapshot INTEGER;
ALTER TABLE moderation_case ADD COLUMN quorum_snapshot INTEGER;
ALTER TABLE moderation_case ADD COLUMN threshold_count_snapshot INTEGER;

-- Appeal-window column — NULLABLE; populated on case-decision by v1-JM-c.
ALTER TABLE moderation_case ADD COLUMN appeal_window_expires_at TIMESTAMPTZ;

-- jury_assignment additions
ALTER TABLE jury_assignment ADD COLUMN selected_under_constraints JSONB;
ALTER TABLE jury_assignment ADD COLUMN role jury_assignment_role NOT NULL DEFAULT 'Original';

-- Backfill — PRD §8.4 exact semantics. panel_size=5, quorum=3,
-- threshold_count=3 reproduces the v0 3-of-5 simple-majority rule.
-- appeal_window_expires_at: if case already closed, keep closed_at; else
-- decided_at + 7 days (matches v0 APPEAL_WINDOW_DAYS).
UPDATE moderation_case
SET panel_size_snapshot = 5,
    quorum_snapshot = 3,
    threshold_count_snapshot = 3,
    appeal_window_expires_at = COALESCE(
      closed_at,
      CASE
        WHEN decided_at IS NOT NULL THEN decided_at + INTERVAL '7 days'
        ELSE NULL
      END
    )
WHERE panel_size_snapshot IS NULL;
```

```sql
-- down.sql — reverse in LIFO order, keeping single-transaction shape.
ALTER TABLE jury_assignment DROP COLUMN IF EXISTS role;
ALTER TABLE jury_assignment DROP COLUMN IF EXISTS selected_under_constraints;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS appeal_window_expires_at;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS threshold_count_snapshot;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS quorum_snapshot;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS panel_size_snapshot;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS status_tier;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS severity_tier;
```

**GOTCHA:** The backfill `UPDATE ... WHERE panel_size_snapshot IS NULL` makes the migration **idempotent** — re-running it after down+up is a no-op. Without the `WHERE` clause, re-application would double-up on already-populated rows.

**GOTCHA:** `CASE WHEN decided_at IS NOT NULL` guards against pre-`Decided` cases (Open / ThresholdMet / JurySelection / InReview) — they have no decided_at and should keep `appeal_window_expires_at = NULL` until v1-JM-c writes one. `COALESCE(closed_at, CASE ...)` ensures a case that's already `Closed` uses its `closed_at` as the upper bound per PRD §11.

**GOTCHA:** `appeal_window_expires_at` is NULLABLE so mid-flight pre-`Decided` cases remain unaffected. The backfill populates it only for cases with a `decided_at` or a `closed_at`.

### 10.5 New table pattern

**SOURCE:** `migrations/2026-04-22-000000-0000_add_rule_set_versions/up.sql` (from v1-AD-a Task 1 per Explore agent §5 + in-repo verification)

**COPY THIS PATTERN for Task 5's `jury_constraint_violation_log` (migration `2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql` — part of the same migration as the column additions, keeping JM-a's schema changes atomic):**

```sql
-- Appended to the same up.sql after the backfill block:

CREATE TABLE jury_constraint_violation_log (
  id SERIAL PRIMARY KEY,
  case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
  constraint_name TEXT NOT NULL,
  relaxation_reason TEXT NOT NULL,
  pool_size_at_relax INTEGER NOT NULL,
  panel_size_target INTEGER NOT NULL,
  relaxed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_jcvl_case_id ON jury_constraint_violation_log (case_id);
```

```sql
-- down.sql — DROP in reverse:
DROP INDEX IF EXISTS idx_jcvl_case_id;
DROP TABLE IF EXISTS jury_constraint_violation_log;
-- ... then the column-drop block from §10.4 ...
```

**GOTCHA:** Table creation must precede column drops in down.sql so that FK from `case_id` to `moderation_case.id` is already gone before `moderation_case` regains its pre-v1 column shape. But since we're dropping the whole JCVL table before touching moderation_case, ordering works.

**GOTCHA:** `pool_size_at_relax` + `panel_size_target` are stored so the admin dashboard query can compare "how short was the pool" vs "how big should the panel have been" for a given case — no PII, only constraint metadata (Watch 10 discipline from v1-JM-b preview).

### 10.6 schema.rs table! block + column extensions

**SOURCE:** `crates/db_schema_file/src/schema.rs:736–762` (moderation_case current shape); `:512–525` (jury_assignment current shape)

**COPY THIS PATTERN for Task 4:**

```rust
diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CaseTargetType;
    use super::sql_types::CaseSeverity;
    use super::sql_types::CaseStatus;
    use super::sql_types::SeverityTier;
    use super::sql_types::CaseStatusTier;

    moderation_case (id) {
        id -> Int4,
        community_id -> Nullable<Int4>,
        creator_id -> Nullable<Int4>,
        target_type -> CaseTargetType,
        target_post_id -> Nullable<Int4>,
        target_comment_id -> Nullable<Int4>,
        target_person_id -> Nullable<Int4>,
        target_community_id -> Nullable<Int4>,
        target_remote_url -> Nullable<Text>,
        reason_code -> Text,
        severity -> CaseSeverity,
        status -> CaseStatus,
        threshold_score -> Int8,
        opened_at -> Timestamptz,
        decided_at -> Nullable<Timestamptz>,
        closed_at -> Nullable<Timestamptz>,
        applied_config_snapshot -> Nullable<Jsonb>,
        rule_set_version_id -> Nullable<Int4>,
        // v1-JM-a additions:
        severity_tier -> SeverityTier,
        status_tier -> CaseStatusTier,
        panel_size_snapshot -> Nullable<Int4>,
        quorum_snapshot -> Nullable<Int4>,
        threshold_count_snapshot -> Nullable<Int4>,
        appeal_window_expires_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::JuryAssignmentStatus;
    use super::sql_types::JuryAssignmentRole;

    jury_assignment (id) {
        id -> Int4,
        case_id -> Int4,
        person_id -> Int4,
        status -> JuryAssignmentStatus,
        selected_at -> Timestamptz,
        responded_at -> Nullable<Timestamptz>,
        submitted_at -> Nullable<Timestamptz>,
        // v1-JM-a additions:
        selected_under_constraints -> Nullable<Jsonb>,
        role -> JuryAssignmentRole,
    }
}

diesel::table! {
    jury_constraint_violation_log (id) {
        id -> Int4,
        case_id -> Int4,
        constraint_name -> Text,
        relaxation_reason -> Text,
        pool_size_at_relax -> Int4,
        panel_size_target -> Int4,
        relaxed_at -> Timestamptz,
    }
}
```

**GOTCHA:** Hand-edit of an `@generated` file (same convention documented at `schema.rs:1` header). Governance additions follow this pattern — Phase 1, 5a, 5b, 6, v1-AD-a all hand-edited schema.rs without regenerating from the DB.

### 10.7 Diesel Queryable + InsertForm pattern

**SOURCE:** `crates/db_schema/src/source/governance/moderation_case.rs:1–68` (post-v1-AD-a shape) and `jury_assignment.rs:1–35`

**COPY THIS PATTERN for Task 6:**

For `ModerationCase` struct extension (same file, same struct — add the 6 new fields after the v1-AD-a `rule_set_version_id`):

```rust
#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = moderation_case))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModerationCase {
  pub id: ModerationCaseId,
  // ... v0 + v1-AD-a fields (unchanged) ...
  pub applied_config_snapshot: Option<Value>,
  pub rule_set_version_id: Option<RuleSetVersionId>,
  /// v1-JM-a §8.1: severity tier frozen at admin_assign_jury time per
  /// ADR-010. NOT NULL; `'Minor'` default pre-populated by the backfill.
  pub severity_tier: SeverityTier,
  /// v1-JM-a §8.1: Founder/Regular/Probation tier determined from target's
  /// reputation_event / membership_state at case-open time.
  pub status_tier: CaseStatusTier,
  /// v1-JM-a §8.1: integer snapshot at jury-seating time. NULL for cases
  /// that haven't reached JurySelection yet.
  pub panel_size_snapshot: Option<i32>,
  pub quorum_snapshot: Option<i32>,
  pub threshold_count_snapshot: Option<i32>,
  /// v1-JM-a §8.1 + §6.5: set by v1-JM-c submit_jury_vote at case-decision
  /// time (`decided_at + appeal.window_days`). NULL until decided.
  pub appeal_window_expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = moderation_case))]
pub struct ModerationCaseInsertForm {
  // ... v0 + v1-AD-a fields unchanged ...
  pub applied_config_snapshot: Option<Value>,
  pub rule_set_version_id: Option<RuleSetVersionId>,
  /// v1-JM-a: severity_tier NOT NULL with DB DEFAULT 'Minor'.
  /// Optional on insert so v0/earlier-v1 callers that set only the
  /// severity CaseSeverity field continue to compile; DB backfill covers
  /// the rest. v1-JM-c / v1-JM-b writers will supply this explicitly.
  pub severity_tier: Option<SeverityTier>,
  pub status_tier: Option<CaseStatusTier>,
  pub panel_size_snapshot: Option<i32>,
  pub quorum_snapshot: Option<i32>,
  pub threshold_count_snapshot: Option<i32>,
  pub appeal_window_expires_at: Option<DateTime<Utc>>,
}
```

**IMPORTS** (extend the existing import block):

```rust
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseSeverity, CaseStatus, CaseTargetType, CaseStatusTier, SeverityTier},
};
```

For `JuryAssignment` extension in `crates/db_schema/src/source/governance/jury_assignment.rs`:

```rust
// Extend the existing struct:
pub struct JuryAssignment {
  // ... existing fields unchanged ...
  pub submitted_at: Option<DateTime<Utc>>,
  /// v1-JM-a §8.2: JSONB payload listing which constraints were applied
  /// at panel-pick time (per PRD Watch 10: constraint names only, no
  /// person_id). Written by v1-JM-b admin_assign_jury; NULL pre-v1.
  pub selected_under_constraints: Option<Value>,
  /// v1-JM-a §8.2: distinguishes original-jury rows from appeal-jury rows
  /// on the same case. Default 'Original'; appeal panels write 'Appeal'
  /// in v1-JM-d.
  pub role: JuryAssignmentRole,
}
```

New file `crates/db_schema/src/source/governance/jury_constraint_violation_log.rs`:

```rust
use crate::newtypes::{JuryConstraintViolationLogId, ModerationCaseId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::jury_constraint_violation_log;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = jury_constraint_violation_log))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// v1-JM-a audit row — written alongside the `jury_constraint_relaxed`
/// governance_log entry every time select_eligible_jurors relaxes a
/// constraint per PRD §5.3.
pub struct JuryConstraintViolationLog {
  pub id: JuryConstraintViolationLogId,
  pub case_id: ModerationCaseId,
  pub constraint_name: String,
  pub relaxation_reason: String,
  pub pool_size_at_relax: i32,
  pub panel_size_target: i32,
  pub relaxed_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = jury_constraint_violation_log))]
pub struct JuryConstraintViolationLogInsertForm {
  pub case_id: ModerationCaseId,
  pub constraint_name: String,
  pub relaxation_reason: String,
  pub pool_size_at_relax: i32,
  pub panel_size_target: i32,
}
```

### 10.8 Newtype ID pattern

**SOURCE:** `crates/db_schema/src/newtypes.rs:214` (`ModerationCaseId`)

**COPY THIS PATTERN for Task 6's newtype additions:**

```rust
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct JuryConstraintViolationLogId(pub i32);
```

Insert alphabetically/logically near the governance newtype block (line ~215–293 range).

### 10.9 EXPECTED_SEED_COUNT_V1_JM const + parity test extension

**SOURCE:** `crates/api/api/src/governance/config.rs:929–938` (existing `EXPECTED_SEED_COUNT` + `EXPECTED_SEED_COUNT_V1_AD`) and `:1700–1798` (parity tests)

**COPY THIS PATTERN for Task 7:**

```rust
/// v1-JM-a adds 27 jury-mechanics-owned keys to `SEEDED_KEYS_WITH_CONSTS`.
/// Parametric per advisor directive 2026-04-19 #4 — each v1 sub-PRD adds
/// its own `EXPECTED_SEED_COUNT_V1_*` beside the v0 + v1-AD-a invariants
/// without churning them. Count is **authoritative against the on-disk
/// reality**: Task 8 reconciliation gate asserts
/// `SEEDED_KEYS_WITH_CONSTS` contains exactly this many new tuples and
/// the seed migration's `up.sql` has exactly this many INSERT rows.
pub const EXPECTED_SEED_COUNT_V1_JM: usize = 27;
```

Extend the parity test at `:1703–1717`:

```rust
#[test]
fn seeded_keys_count_matches_const_count() {
  let expected = EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD + EXPECTED_SEED_COUNT_V1_JM;
  assert_eq!(
    SEEDED_KEYS_WITH_CONSTS.len(),
    expected,
    "SEEDED_KEYS_WITH_CONSTS length ({}) must equal EXPECTED_SEED_COUNT ({}) + \
     EXPECTED_SEED_COUNT_V1_AD ({}) + EXPECTED_SEED_COUNT_V1_JM ({}) = {} — add/remove keys \
     in both places when changing the seed list",
    SEEDED_KEYS_WITH_CONSTS.len(),
    EXPECTED_SEED_COUNT,
    EXPECTED_SEED_COUNT_V1_AD,
    EXPECTED_SEED_COUNT_V1_JM,
    expected,
  );
}
```

The other two parity tests (`every_seeded_key_has_metadata`, `every_seeded_key_has_const_fallback`) extend automatically since they iterate over `SEEDED_KEYS_WITH_CONSTS` — no test body change needed.

### 10.10 ENTRY_KIND const dual-file edit

**SOURCE:** `.claude/PRPs/plans/completed/v1-admin-dashboard-a.plan.md:377–422` (§10.8) + current state of both files per Explore agent §4

**COPY THIS PATTERN for Task 9** (atomic single commit across both files):

FILE 1 (DEFINITION) — `crates/db_schema/src/source/governance/governance_log.rs`, append after `ENTRY_KIND_RULE_SET_VERSION_CREATED` (the v1-AD-c entry ending the current 26-const list):

```rust
// v1-JM-a additions (v1 jury-mechanics sub-phase A). All six emitting
// call sites land in v1-JM-b/c/d per PRD §9.2/§9.3/§9.4/§9.5. v1-JM-a
// ships the const declarations and the registry entry only.
pub const ENTRY_KIND_JURY_CONSTRAINT_RELAXED: &str = "jury_constraint_relaxed";
pub const ENTRY_KIND_APPEAL_PANEL_ASSEMBLED: &str = "appeal_panel_assembled";
pub const ENTRY_KIND_APPEAL_DECIDED: &str = "appeal_decided";
pub const ENTRY_KIND_APPEAL_REJECTED: &str = "appeal_rejected";
pub const ENTRY_KIND_APPEAL_WINDOW_EXPIRED: &str = "appeal_window_expired";
pub const ENTRY_KIND_SEVERITY_TIER_FROZEN: &str = "severity_tier_frozen";
```

FILE 2 (RE-EXPORT SHIM) — `crates/api/api/src/governance/governance_log.rs`, extend the `pub use` block alphabetically:

```rust
pub use lemmy_db_schema::source::governance::governance_log::{
  ENTRY_KIND_ADMIN_CONFIG_CHANGED,
  ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED,
  ENTRY_KIND_APPEAL_DECIDED,              // new — v1-JM-a
  ENTRY_KIND_APPEAL_PANEL_ASSEMBLED,      // new — v1-JM-a
  ENTRY_KIND_APPEAL_REJECTED,             // new — v1-JM-a
  ENTRY_KIND_APPEAL_REQUESTED,
  ENTRY_KIND_APPEAL_WINDOW_EXPIRED,       // new — v1-JM-a
  ENTRY_KIND_CAPABILITY_CHANGED,
  ENTRY_KIND_CASE_DECIDED,
  // ... existing alphabetical order preserved ...
  ENTRY_KIND_JURY_ASSIGNED,
  ENTRY_KIND_JURY_CONSTRAINT_RELAXED,     // new — v1-JM-a
  ENTRY_KIND_JURY_DECLINED,
  // ... existing entries ...
  ENTRY_KIND_SEVERITY_TIER_FROZEN,        // new — v1-JM-a
  ENTRY_KIND_SPONSOR_LIABILITY_APPLIED,
  // ... rest of existing entries ...
  GovernanceLog,
  GovernanceLogInsertForm,
  append,
};
```

**GOTCHA:** Alphabetical order enforced by eye; CodeRabbit flags drift. Insert `APPEAL_DECIDED` before `APPEAL_PANEL_ASSEMBLED` before `APPEAL_REJECTED` before the existing `APPEAL_REQUESTED` before the new `APPEAL_WINDOW_EXPIRED`.

**GOTCHA:** Both edits land in the **same commit**. A `db_schema` definition without the shim re-export breaks callers importing from the api path; a shim re-export without the `db_schema` definition fails to compile.

---

## 11. Files to change

| File | Action | Justification |
|---|---|---|
| `migrations/2026-04-23-000000-0000_add_jury_mechanics_enums/up.sql` | CREATE | 3 new Postgres enum types per PRD §8 |
| `migrations/2026-04-23-000000-0000_add_jury_mechanics_enums/down.sql` | CREATE | `DROP TYPE` in LIFO order |
| `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql` | CREATE | 6+2 ALTER TABLE ADD COLUMN + `jury_constraint_violation_log` CREATE TABLE + index + backfill UPDATE per PRD §8.1/§8.2/§8.3/§8.4 |
| `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/down.sql` | CREATE | Reverse-order DROP COLUMN + DROP TABLE |
| `migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql` | CREATE | 27 new `jury.*` + `appeal.*` config rows per PRD §10 matrix (idempotent via `ON CONFLICT`) |
| `migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/down.sql` | CREATE | `DELETE ... WHERE key IN (...)` scoped to the 27 keys |
| `crates/db_schema_file/src/enums.rs` | UPDATE | Add 3 new Rust enum declarations (`SeverityTier`, `CaseStatusTier`, `JuryAssignmentRole`) mirroring `CaseStatus` shape |
| `crates/db_schema_file/src/schema.rs` | UPDATE | Add 3 new `sql_types` structs + extend `moderation_case` table! with 6 new columns + extend `jury_assignment` with 2 new columns + new `jury_constraint_violation_log` table! block |
| `crates/db_schema/src/source/governance/moderation_case.rs` | UPDATE | Extend `ModerationCase` struct + `ModerationCaseInsertForm` with 6 new fields |
| `crates/db_schema/src/source/governance/jury_assignment.rs` | UPDATE | Extend `JuryAssignment` struct with 2 new fields |
| `crates/db_schema/src/source/governance/jury_constraint_violation_log.rs` | CREATE | New Queryable + InsertForm per §10.7 |
| `crates/db_schema/src/source/governance/mod.rs` | UPDATE | `pub mod jury_constraint_violation_log;` + re-export alphabetically |
| `crates/db_schema/src/newtypes.rs` | UPDATE | Add `JuryConstraintViolationLogId(pub i32)` |
| `crates/api/api/src/governance/config.rs` | UPDATE | +27 `DEFAULT_*` consts, +27 `const_default_*` match arms, +27 `SEEDED_KEYS_WITH_CONSTS` tuples, +27 `CONFIG_KEY_METADATA` entries, `EXPECTED_SEED_COUNT_V1_JM: usize = 27`, extend `seeded_keys_count_matches_const_count` to assert against `EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD + EXPECTED_SEED_COUNT_V1_JM`. Counts are parametric — Task 8 reconciliation gate authoritative. |
| `crates/db_schema/src/source/governance/governance_log.rs` | UPDATE | **Define** 6 new `ENTRY_KIND_*` consts (dual-file edit §10.10 File 1) |
| `crates/api/api/src/governance/governance_log.rs` | UPDATE | **Re-export** 6 new consts from the shim's `pub use` list alphabetically (dual-file edit §10.10 File 2) |
| `.claude/rules/governance-log-entry-kind-registry.md` | UPDATE | Replace the `jury-mechanics-v1` reservation stub with 6 populated rows; bump acceptance-invariants count 26 → 32 |
| `crates/server/tests/e2e.rs` | UPDATE | Extend `PHASE_1_MIGRATION_COUNT` from 9 to 12 (3 new JM-a migrations); extend `phase1_migrations_round_trip` tables + pg_type loops with the 3 new enum types and the new `jury_constraint_violation_log` table; add new e2e test `v1_jm_a_backfill_populates_v0_snapshot` asserting Minor/Regular/5/3/3 on a seeded pre-v1 case |

**Total: 18 files changed.**
- 6 NEW migration files (3 migration dirs × 2 files each)
- 1 NEW Rust source file (`jury_constraint_violation_log.rs`)
- 10 UPDATED Rust files (`enums.rs`, `schema.rs`, `moderation_case.rs`, `jury_assignment.rs`, `mod.rs`, `newtypes.rs`, `config.rs`, `db_schema/.../governance_log.rs`, `api/.../governance_log.rs`, `e2e.rs`)
- 1 UPDATED rules file (`governance-log-entry-kind-registry.md`)

---

## 12. NOT building in v1-JM-a

Explicitly out of scope. If one of these creeps in, STOP and file a decision-queue entry.

- **No `admin_assign_jury.rs` edits.** Panel-size cascade read, `select_eligible_jurors` constraint parameterisation, `severity_tier_frozen` emission — all v1-JM-b.
- **No `submit_jury_vote.rs` edits.** Quorum snapshot read, threshold check, deadlock-to-AdminReview, sponsor-liability branch, `appeal_window_expires_at` write — all v1-JM-c.
- **No `request_appeal.rs` edits.** Bounded-window check, reporter-rights, auto re-jury — all v1-JM-d.
- **No new handler file.** `admin_trigger_appeal_rejury.rs` is v1-JM-d.
- **No background job.** `appeal_window_expired` scheduler tick is v1-JM-d.
- **No cascade helper.** `config::get_int_cascade` / `get_float_cascade` land in v1-JM-b alongside their first consumer.
- **No route wiring.** `crates/api/routes/src/governance.rs` is untouched.
- **No DTO changes.** `crates/api/api_common/src/governance.rs` is untouched.
- **No `admin_emergency_remove.rs` edit.** Severe-by-default severity-tier assignment is a v1-JM-b / v1-JM-c concern (lives alongside the first writer that actually reads severity_tier).
- **No v0 const removal.** `QUORUM` and `APPEAL_WINDOW_DAYS` in `submit_jury_vote.rs` stay until v1-JM-c replaces their readers. Deleting them in JM-a would break v0 compilation.
- **No contradiction of v1-AD-a state.** Do not rename `EXPECTED_SEED_COUNT` or `EXPECTED_SEED_COUNT_V1_AD`. Do not alphabetise the v0 block of `SEEDED_KEYS_WITH_CONSTS`. Do not renumber existing `CONFIG_KEY_METADATA` entries.
- **No OQ resolution.** OQ-V1-JM-01..06 and OQ-026 are left open — their resolution belongs to JM-b/c/d impl runs.
- **No AGPL notice update.** Not a release artefact.
- **No federation / ActivityPub work.** ADR-014 content-level federation unaffected.
- **No PM-hook changes.** The seven PM plugin hooks (per `.claude/rules/pm-plugin-hooks-stable.md`) must remain present and unchanged; Task 10 includes a belt-and-braces grep check.

---

## 13. Step-by-step tasks

Execute in order. **One commit per task** on branch `phase-v1-JM-a`. Each task has a MIRROR reference, exact file paths, and a validation command. Task 0 is a pre-flight gate with no commits; Tasks 1–10 each produce one commit; Task 11 is the retro write-up (commit) before PR open.

### Task 0: PRE-FLIGHT — verify branch + wrapper sanity + v0 baseline

- **ACTION**: Confirm branch + run the pre-phase harness audit.
- **COMMIT MESSAGE**: No commit from Task 0.
- **GOTCHA**: If the current branch is `governance-v0`, STOP and write a `.claude/decision-queue.json` entry — advisor cuts the phase branch, not the impl agent (per `.claude/rules/phase-branch.md:7–15`).
- **GOTCHA**: All four pre-phase-harness probes (0/1/2/3) must pass AND probe 4 (negative exit-code propagation) must return non-zero exit codes. Failure = wrapper bug invalidating every downstream cargo signal; fix wrapper in a pre-Task 1 commit before proceeding.
- **VALIDATE**:

  ```bash
  git branch --show-current   # expect: phase-v1-JM-a
  git merge-base phase-v1-JM-a governance-v0   # record SHA
  git log -1 --format=%H governance-v0         # record SHA — should match merge-base
  docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER DOWN — start Docker Desktop"; exit 1; }
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema > .claude/PRPs/debug/v1-JM-a-audit-probe1.log 2>&1"
  echo "probe1 exit: $?"   # expect 0
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features nonexistent_xyz > .claude/PRPs/debug/v1-JM-a-audit-probe4.log 2>&1"
  echo "probe4 exit: $?"   # expect non-zero
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-JM-a-audit-probe3.log 2>&1"
  echo "probe3 exit: $?"   # expect 0 (baseline compile of e2e target)
  ```
- **EXPECT**: Current branch `phase-v1-JM-a`; `git merge-base` output equals current `governance-v0` HEAD; Docker up; probes 1+3 exit 0; probe 4 exits non-zero. No commits from this task.

### Task 1: CREATE `migrations/2026-04-23-000000-0000_add_jury_mechanics_enums/{up,down}.sql`

- **ACTION**: New migration directory. Three `CREATE TYPE ... AS ENUM` statements in up.sql; LIFO `DROP TYPE` in down.sql.
- **IMPLEMENT**: See §10.1 exact pattern. Enums are `severity_tier`, `case_status_tier`, `jury_assignment_role` with variants per PRD §3.1 / §4.1.
- **MIRROR**: `migrations/2026-04-15-100000-0000_add_governance_enums/{up,down}.sql` (Phase 1 precedent, full).
- **GOTCHA**: PascalCase variants (`'Minor'`, `'Founder'`, `'Original'`) — matches `DbValueStyle = "verbatim"` on the Rust side in Task 3.
- **GOTCHA**: `down.sql` drops in reverse insertion order. Phase 1 precedent uses DROP-alphabetical; we use DROP-reverse-create to be unambiguous about the dependency chain (none in v1-JM-a since the three enums are independent, but pattern-consistency matters for future rebases).
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema > .claude/PRPs/debug/v1-JM-a-task1-check.log 2>&1"
  echo "exit: $?"   # expect 0 (migrations aren't compiled; this just confirms the workspace still builds — the migration application runs in Task 10's e2e test)
  ```
- **COMMIT MESSAGE**: `feat(v1-JM-a): add 3 new Postgres enums — severity_tier, case_status_tier, jury_assignment_role (task 1)`

### Task 2: CREATE `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/{up,down}.sql`

- **ACTION**: Atomic migration — 6+2 ALTER COLUMN + new `jury_constraint_violation_log` table + backfill UPDATE, all in one up.sql. down.sql reverses.
- **IMPLEMENT**: See §10.4 + §10.5 exact patterns. Backfill UPDATE uses `COALESCE(closed_at, CASE WHEN decided_at ... END)` per §10.4 GOTCHA.
- **MIRROR**: `migrations/2026-04-22-000200-0000_add_case_applied_config_snapshot/up.sql` (v1-AD-a JSONB+FK shape); `migrations/2026-04-18-000100-0000_add_person_membership_state/up.sql` (fast-metadata `ADD COLUMN NOT NULL DEFAULT 'member'` for the enum-typed columns).
- **GOTCHA**: Timestamp `000100-0000` **MUST** be after `000000-0000` so enums exist before the ALTER references them. Diesel-style lexicographic ordering.
- **GOTCHA**: Enum-typed columns use `NOT NULL DEFAULT 'Minor'` / `'Regular'` / `'Original'` — fast metadata-only backfill per Postgres 11+ `attmissingval`, avoiding a full table rewrite. Integer snapshot columns are NULLABLE; their values are populated by the explicit `UPDATE` at the bottom of up.sql per PRD §8.4.
- **GOTCHA**: `UPDATE ... WHERE panel_size_snapshot IS NULL` makes the backfill idempotent across down/up cycles.
- **GOTCHA**: `appeal_window_expires_at` is NULLABLE and backfilled only for cases with `decided_at IS NOT NULL` OR `closed_at IS NOT NULL` — pre-`Decided` cases remain NULL until v1-JM-c writes one.
- **GOTCHA**: `jury_constraint_violation_log` creation **and** its `idx_jcvl_case_id` CREATE INDEX belong in this same migration file (single-commit schema change) — NOT split into a third migration.
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema > .claude/PRPs/debug/v1-JM-a-task2-check.log 2>&1"
  echo "exit: $?"   # expect 0 — again, migrations are runtime SQL; Task 10 runs them end-to-end
  ```
- **COMMIT MESSAGE**: `feat(v1-JM-a): add moderation_case + jury_assignment snapshot columns + jury_constraint_violation_log + v0→v1 backfill (task 2)`

### Task 3: UPDATE `crates/db_schema_file/src/enums.rs` — add 3 new Rust enums

- **ACTION**: Append three Diesel-backed enum declarations after the existing ones. All three use `DbValueStyle = "verbatim"` and full derive set per §10.2.
- **IMPLEMENT**: See §10.2 pattern × 3. Variants:
  - `SeverityTier { Minor (default), Moderate, Severe }`
  - `CaseStatusTier { Founder, Regular (default), Probation }`
  - `JuryAssignmentRole { Original (default), Appeal }`
- **MIRROR**: `crates/db_schema_file/src/enums.rs:382–408` (`CaseStatus`). **DO NOT** mirror `MembershipState:624–648` — that uses `snake_case` which is a deferred-enforcement exception documented in its own comment.
- **IMPORTS**: existing imports at top of file already cover `Serialize, Deserialize, DbEnum, ts_rs` — no additions needed.
- **GOTCHA**: Each enum's doc comment must cite the PRD section that defines its semantics (`/// v1 jury-mechanics severity tier, PRD §3.1 ...`) so future readers can trace back. Mirror the style of `MembershipState`'s deferred-enforcement comment.
- **GOTCHA**: `#[default]` must match the Postgres-side `DEFAULT` value set in Task 2's migration (`Minor` / `Regular` / `Original`). Mismatches lead to hard-to-diagnose insert errors where the Rust struct provides a different default than the column expects.
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-a-task3-check.log 2>&1"
  echo "exit: $?"   # expect 0
  ```
  (`--features full` required per `.claude/memory/feedback_features_full_workspace_only.md` to activate the `DbEnum` + `ts-rs` derives.)
- **COMMIT MESSAGE**: `feat(v1-JM-a): add SeverityTier + CaseStatusTier + JuryAssignmentRole Rust enums (task 3)`

### Task 4: UPDATE `crates/db_schema_file/src/schema.rs` — sql_types + table! extensions

- **ACTION**: Hand-edit `@generated` schema.rs per governance convention. Three sub-edits:
  1. Add three structs to the `sql_types` module per §10.3 pattern.
  2. Extend `moderation_case` table! block with 6 new columns per §10.6.
  3. Extend `jury_assignment` table! block with 2 new columns.
  4. Add new `jury_constraint_violation_log` table! block.
- **IMPLEMENT**: See §10.3 + §10.6.
- **MIRROR**: `schema.rs:3–119` (`sql_types` module); `schema.rs:736–762` (extending `moderation_case`); `schema.rs:512–525` (extending `jury_assignment`); `schema.rs:412–455` (precedent for hand-adding a fresh table! block — `governance_config`).
- **GOTCHA**: Hand-editing `@generated`. Document all edits inline (`// v1-JM-a additions:` comments).
- **GOTCHA**: `use super::sql_types::SeverityTier;` + `CaseStatusTier` + `JuryAssignmentRole` must be added to the respective `table!` `use` blocks. Without them, diesel compilation fails with "undefined type" errors.
- **GOTCHA**: No `joinable!()` macros for the new tables — `jury_constraint_violation_log.case_id → moderation_case.id` does not need one (grep shows no existing joinable! for governance tables; follow the precedent).
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema_file > .claude/PRPs/debug/v1-JM-a-task4-check-file.log 2>&1"
  echo "exit: $?"   # expect 0
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-a-task4-check-workspace.log 2>&1"
  echo "exit: $?"   # expect 0
  ```
- **COMMIT MESSAGE**: `feat(v1-JM-a): extend schema.rs — new sql_types + moderation_case/jury_assignment/jury_constraint_violation_log table! (task 4)`

### Task 5: CREATE `crates/db_schema/src/source/governance/jury_constraint_violation_log.rs` + UPDATE `moderation_case.rs`, `jury_assignment.rs`, `mod.rs`, `newtypes.rs`

- **ACTION**: Four sub-edits, one commit:
  1. CREATE `jury_constraint_violation_log.rs` with `JuryConstraintViolationLog` Queryable + `JuryConstraintViolationLogInsertForm` Insertable per §10.7.
  2. UPDATE `moderation_case.rs` — extend `ModerationCase` + `ModerationCaseInsertForm` with 6 new fields per §10.7.
  3. UPDATE `jury_assignment.rs` — extend `JuryAssignment` struct with 2 new fields per §10.7.
  4. UPDATE `mod.rs` — `pub mod jury_constraint_violation_log;` alphabetically; re-export if convention demands.
  5. UPDATE `newtypes.rs` — add `JuryConstraintViolationLogId(pub i32)` per §10.8.
- **IMPLEMENT**: See §10.7 + §10.8 exact patterns.
- **MIRROR**: `crates/db_schema/src/source/governance/moderation_case.rs:1–68` (current shape post-v1-AD-a) and `jury_assignment.rs:1–35`. `crates/db_schema/src/newtypes.rs:214` for the newtype.
- **GOTCHA**: `ModerationCaseInsertForm`'s 6 new fields are `Option<_>` so v0/earlier-v1 callers continue to compile without setting them explicitly. The DB DEFAULTs cover the not-null enum columns; the nullable integer columns default to NULL.
- **GOTCHA**: `JuryAssignment` struct does NOT get an `InsertForm` extension for `role` because the DEFAULT `'Original'` covers every v0 writer path. v1-JM-d (`select_appeal_panel`) will write `role = Appeal` via a new call site, not by extending the InsertForm.
- **GOTCHA**: `JuryConstraintViolationLogInsertForm` deliberately does not include `relaxed_at` (DB DEFAULT `now()`) or `id` (SERIAL).
- **GOTCHA**: Do NOT derive `AsChangeset` on the new `JuryConstraintViolationLog*` structs — the table is append-only per the governance-table append-only invariant (same reasoning as `governance_log`).
- **IMPORTS** in `moderation_case.rs`:

  ```rust
  use lemmy_db_schema_file::{
    PersonId,
    enums::{CaseSeverity, CaseStatus, CaseTargetType, CaseStatusTier, SeverityTier},
  };
  ```
  In `jury_assignment.rs`, add `JuryAssignmentRole` to the enum imports alongside `JuryAssignmentStatus`. In `jury_constraint_violation_log.rs`, import `ModerationCaseId` from `crate::newtypes`.
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace > .claude/PRPs/debug/v1-JM-a-task5-check.log 2>&1"
  echo "exit: $?"   # expect 0
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-a-task5-check-full.log 2>&1"
  echo "exit: $?"   # expect 0
  ```
- **COMMIT MESSAGE**: `feat(v1-JM-a): extend Diesel models — ModerationCase + JuryAssignment fields + new JuryConstraintViolationLog + newtype (task 5)`

### Task 6: UPDATE `crates/api/api/src/governance/config.rs` — 27 new consts + metadata entries + match arms + parity test extension

- **ACTION**: Five sub-edits, one commit:
  1. Add 27 new `pub const DEFAULT_JURY_*` / `DEFAULT_APPEAL_*` declarations alongside the existing ones (after `DEFAULT_GOVERNANCE_DASHBOARD_STEP_UP_ENFORCED` which closes the v1-AD-a block).
  2. Add 27 new match arms across the four `const_default_{int,float,bool,text}` functions.
  3. Extend `SEEDED_KEYS_WITH_CONSTS` with a commented v1-JM-a block (27 new tuples after the v1-AD-a block at `:924`).
  4. Add `pub const EXPECTED_SEED_COUNT_V1_JM: usize = 27;` immediately after `EXPECTED_SEED_COUNT_V1_AD` at `:938` per §10.9.
  5. Extend `seeded_keys_count_matches_const_count` test at `:1703–1717` to include `EXPECTED_SEED_COUNT_V1_JM` per §10.9.
  6. Add 27 new `CONFIG_KEY_METADATA` entries matching the 27 seeded keys (one entry per key name, order irrelevant per the existing registry comment).
- **IMPLEMENT**: 27 keys per the PRD §10 Defaults Matrix. Breakdown by namespace:

  **`jury.panel_size.<status>.<severity>` — 9 keys (int):**
  | Key | Default | Namespace rationale |
  |---|---|---|
  | `jury.panel_size.regular.minor` | 5 | PRD §3.3 v0 baseline preserved |
  | `jury.panel_size.regular.moderate` | 5 | PRD §3.3 v0 baseline |
  | `jury.panel_size.regular.severe` | 7 | PRD §3.3 [01 §5.6] v1 target |
  | `jury.panel_size.founder.minor` | 5 | PRD §3.3 OQ-026 |
  | `jury.panel_size.founder.moderate` | 7 | PRD §3.3 OQ-026 |
  | `jury.panel_size.founder.severe` | 9 | PRD §3.3 OQ-026 max-care |
  | `jury.panel_size.probation.minor` | 3 | PRD §3.3 probation fast turnaround |
  | `jury.panel_size.probation.moderate` | 5 | PRD §3.3 OQ-026 |
  | `jury.panel_size.probation.severe` | 5 | PRD §3.3 OQ-026 |

  **`jury.quorum_fraction.<severity>` — 3 keys (float):**
  | Key | Default | Rationale |
  |---|---|---|
  | `jury.quorum_fraction.minor` | 0.6 | 3-of-5 v0 baseline |
  | `jury.quorum_fraction.moderate` | 0.6 | Mirror Minor v1 |
  | `jury.quorum_fraction.severe` | 0.71 | 5-of-7 = 71% |

  **`jury.threshold_fraction.<severity>` — 3 keys (float):**
  | Key | Default | Rationale |
  |---|---|---|
  | `jury.threshold_fraction.minor` | 0.5001 | [01 §5.6] simple majority strict >50% |
  | `jury.threshold_fraction.moderate` | 0.6 | [01 §5.6] 60% |
  | `jury.threshold_fraction.severe` | 0.75 | [01 §5.6] 75% supermajority |

  **`jury.constraints.*` — 5 keys (4 bool + 1 int):**
  | Key | Type | Default |
  |---|---|---|
  | `jury.constraints.no_majority_from_same_sponsor_cluster` | bool | true |
  | `jury.constraints.geographic_diversity_preferred` | bool | true |
  | `jury.constraints.no_recent_juror_repeat` | bool | true |
  | `jury.constraints.juror_cooldown_days` | int | 7 |
  | `jury.constraints.no_same_endorsement_chain` | bool | false |

  **`jury.constraints.max_retries_before_relax` + `jury.max_concurrent_assignments_per_juror_total` — 2 keys (int):**
  | Key | Default |
  |---|---|
  | `jury.constraints.max_retries_before_relax` | 5 |
  | `jury.max_concurrent_assignments_per_juror_total` | 2 |

  **`appeal.*` — 5 keys (2 float + 2 int + 1 bool):**
  | Key | Type | Default |
  |---|---|---|
  | `appeal.panel_size_multiplier` | float | 1.5 |
  | `appeal.panel_size_floor_increment` | int | 2 |
  | `appeal.threshold_tier_bump` | int | 1 |
  | `appeal.window_days` | int | 7 |
  | `appeal.auto_select_on_appeal_acceptance` | bool | true |

  **Count check: 9 + 3 + 3 + 5 + 2 + 5 = 27 ✓**

  **By value_type:**
  - int: 9 panel_size + 1 cooldown + 1 retries + 1 per_juror_total + 2 appeal (floor_increment, tier_bump, window_days) = **13** wait recount: 9 + 1 + 1 + 1 + 3 = 15. Let me recount: panel_size (9 int) + quorum_fraction (3 float) + threshold_fraction (3 float) + constraints bools (4) + cooldown (1 int) + max_retries (1 int) + per_juror_total (1 int) + appeal panel_size_multiplier (1 float) + appeal floor_increment (1 int) + appeal tier_bump (1 int) + appeal window_days (1 int) + appeal auto_select (1 bool) = **27** ✓.
  - **int:** 9 (panel) + 1 (cooldown) + 1 (retries) + 1 (per_juror_total) + 3 (floor_increment, tier_bump, window_days) = 15
  - **float:** 3 (quorum_fraction) + 3 (threshold_fraction) + 1 (panel_size_multiplier) = 7
  - **bool:** 4 (constraints) + 1 (auto_select_on_appeal_acceptance) = 5
  - **text:** 0
  - **Total: 15 + 7 + 5 = 27** ✓

  **SEEDED_KEYS_WITH_CONSTS new tuples** (appended after the v1-AD-a block at `:924`, with section comment):

  ```rust
  // v1-JM-a additions (v1 jury-mechanics sub-phase A — 27 new keys per
  // PRD §10 defaults matrix + §3.5 cascade)
  ("jury.panel_size.regular.minor", "DEFAULT_JURY_PANEL_SIZE_REGULAR_MINOR", "int"),
  ("jury.panel_size.regular.moderate", "DEFAULT_JURY_PANEL_SIZE_REGULAR_MODERATE", "int"),
  // ... etc — 27 tuples, alphabetized within the block by key name ...
  ```

  **CONFIG_KEY_METADATA entries** (insert 27 new struct literals into the registry, each with `key`, `value_type`, `valid_range` (for numeric), `scope: ConfigScope::Both`, `requires_re_jury: false` (snapshot mechanics make this false per PRD §10 matrix column "Requires re-jury?"), `requires_step_up: false`, `apply_at_default: ApplyAt::NextJuryCycle` for panel/quorum/threshold/constraints; `ApplyAt::Immediate` for appeal.window_days and appeal.auto_select_on_appeal_acceptance per PRD §9.1 "LIVE config read, NOT snapshotted"; `description`, `doc_anchor: "v1-jury-mechanics.prd.md§10"`).

- **MIRROR**: `config.rs:820–924` (SEEDED_KEYS_WITH_CONSTS entries extension); `config.rs:697–810` (const_default_* match arm style); `config.rs:629–810` (DEFAULT_* const style); `config.rs:961–1595` (CONFIG_KEY_METADATA entry style — particularly the v1-AD-a entries at `:1371–1595`); `config.rs:1700–1798` (parity tests — specifically lines 1703–1717 for the first parity test's body).
- **GOTCHA**: **`jury.threshold_fraction.*` is distinct from the v1-AD-a `jury.severity_thresholds.*` text keys** (the latter were human-display strings `'majority'`/`'60%'`/`'75%'`; the former are the actual float fractions the cascade will use). Do NOT rename or repurpose the v1-AD-a keys.
- **GOTCHA**: **`jury.constraints.no_majority_from_same_sponsor_cluster` is distinct from the v1-AD-a `jury.diversity_constraints_enabled` coarse toggle** (per PRD §5.1 catalog). Both coexist — the coarse toggle acts as a global kill-switch; the granular keys let communities tune individual constraints.
- **GOTCHA**: **`appeal.panel_size_multiplier` + `appeal.panel_size_floor_increment` are distinct from the v1-AD-a `jury.appeal_panel_size_increase`** (the latter is a v0-era simple int increment; the former is the v1 multiplier+floor formula). The AD-a key remains seeded but is not read by any v1-JM-c/d code.
- **GOTCHA**: `ApplyAt::NextJuryCycle` for panel/quorum/threshold keys because changing them mid-flight does NOT affect in-flight juries (ADR-010). `ApplyAt::Immediate` for `appeal.window_days` because v1-JM-c reads it live at decision time per PRD §9.1 step 9.
- **GOTCHA**: The `NumericRange` for `panel_size` keys is `{ min: 3.0, max: 11.0 }` per PRD §3.4 bounds. For fraction keys: `{ min: 0.5, max: 1.0 }` (quorum) / `{ min: 0.5001, max: 1.0 }` (threshold). For cooldown / retries / window_days: use sensible operational bounds per PRD §10 range column.
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-a-task6-check.log 2>&1"
  echo "exit: $?"   # expect 0
  cmd //c "scripts\\brehon\\cargo-test.bat --workspace --lib parity > .claude/PRPs/debug/v1-JM-a-task6-parity.log 2>&1"
  echo "exit: $?"   # expect 0 (in-crate parity tests pass)
  ```
- **COMMIT MESSAGE**: `feat(v1-JM-a): config.rs — 27 new jury/appeal consts + metadata + SEEDED_KEYS extension + EXPECTED_SEED_COUNT_V1_JM parity (task 6)`

### Task 7: CREATE `migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/{up,down}.sql` + pre-commit reconciliation gate

- **ACTION**: Write the seed migration with exactly 27 INSERT rows matching Task 6's `SEEDED_KEYS_WITH_CONSTS` extension byte-for-byte. Run the reconciliation gate **before** committing.
- **IMPLEMENT**: Full up.sql shape (mirroring v1-AD-a precedent at `migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql`):

  ```sql
  -- v1-JM-a task 7: seed 27 jury-mechanics-owned governance_config rows.
  --
  -- Authoritative scope (per PRD §10 defaults matrix): 9 panel_size cells
  -- + 3 quorum_fraction + 3 threshold_fraction + 5 jury.constraints.*
  -- + 2 jury.constraints.max_retries + jury.max_concurrent_per_juror
  -- + 5 appeal.* = 27 keys total. Count reconciles to
  -- EXPECTED_SEED_COUNT_V1_JM = 27 in
  -- crates/api/api/src/governance/config.rs.
  --
  -- Byte-for-byte the same key/value/type tuple set as the v1-JM-a
  -- additions block in SEEDED_KEYS_WITH_CONSTS (config.rs lines 925+).
  -- The parity test `every_seeded_key_has_const_fallback` +
  -- `every_seeded_key_has_metadata` + the e2e `config_parity_round_trip`
  -- walk this list and fail closed on drift.
  --
  -- ON CONFLICT (scope, key, valid_from) DO NOTHING keeps this migration
  -- idempotent across reruns; the per-statement now() resolves once so
  -- the 27 rows share a valid_from within a single run.
  INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES
      ('instance', 'jury.panel_size.regular.minor',                          'int',   5,     NULL, NULL, NULL),
      ('instance', 'jury.panel_size.regular.moderate',                       'int',   5,     NULL, NULL, NULL),
      ('instance', 'jury.panel_size.regular.severe',                         'int',   7,     NULL, NULL, NULL),
      ('instance', 'jury.panel_size.founder.minor',                          'int',   5,     NULL, NULL, NULL),
      ('instance', 'jury.panel_size.founder.moderate',                       'int',   7,     NULL, NULL, NULL),
      ('instance', 'jury.panel_size.founder.severe',                         'int',   9,     NULL, NULL, NULL),
      ('instance', 'jury.panel_size.probation.minor',                        'int',   3,     NULL, NULL, NULL),
      ('instance', 'jury.panel_size.probation.moderate',                     'int',   5,     NULL, NULL, NULL),
      ('instance', 'jury.panel_size.probation.severe',                       'int',   5,     NULL, NULL, NULL),
      ('instance', 'jury.quorum_fraction.minor',                             'float', NULL,  0.6,  NULL, NULL),
      ('instance', 'jury.quorum_fraction.moderate',                          'float', NULL,  0.6,  NULL, NULL),
      ('instance', 'jury.quorum_fraction.severe',                            'float', NULL,  0.71, NULL, NULL),
      ('instance', 'jury.threshold_fraction.minor',                          'float', NULL,  0.5001, NULL, NULL),
      ('instance', 'jury.threshold_fraction.moderate',                       'float', NULL,  0.6,  NULL, NULL),
      ('instance', 'jury.threshold_fraction.severe',                         'float', NULL,  0.75, NULL, NULL),
      ('instance', 'jury.constraints.no_majority_from_same_sponsor_cluster', 'bool',  NULL,  NULL, true, NULL),
      ('instance', 'jury.constraints.geographic_diversity_preferred',        'bool',  NULL,  NULL, true, NULL),
      ('instance', 'jury.constraints.no_recent_juror_repeat',                'bool',  NULL,  NULL, true, NULL),
      ('instance', 'jury.constraints.juror_cooldown_days',                   'int',   7,     NULL, NULL, NULL),
      ('instance', 'jury.constraints.no_same_endorsement_chain',             'bool',  NULL,  NULL, false, NULL),
      ('instance', 'jury.constraints.max_retries_before_relax',              'int',   5,     NULL, NULL, NULL),
      ('instance', 'jury.max_concurrent_assignments_per_juror_total',        'int',   2,     NULL, NULL, NULL),
      ('instance', 'appeal.panel_size_multiplier',                           'float', NULL,  1.5,  NULL, NULL),
      ('instance', 'appeal.panel_size_floor_increment',                      'int',   2,     NULL, NULL, NULL),
      ('instance', 'appeal.threshold_tier_bump',                             'int',   1,     NULL, NULL, NULL),
      ('instance', 'appeal.window_days',                                     'int',   7,     NULL, NULL, NULL),
      ('instance', 'appeal.auto_select_on_appeal_acceptance',                'bool',  NULL,  NULL, true, NULL)
  ON CONFLICT (scope, key, valid_from) DO NOTHING;
  ```

  down.sql:

  ```sql
  -- Reverse of 2026-04-23-000200-0000_seed_v1_jm_config_keys up.sql.
  DELETE FROM governance_config
  WHERE scope = 'instance' AND key IN (
      'jury.panel_size.regular.minor',
      'jury.panel_size.regular.moderate',
      'jury.panel_size.regular.severe',
      'jury.panel_size.founder.minor',
      'jury.panel_size.founder.moderate',
      'jury.panel_size.founder.severe',
      'jury.panel_size.probation.minor',
      'jury.panel_size.probation.moderate',
      'jury.panel_size.probation.severe',
      'jury.quorum_fraction.minor',
      'jury.quorum_fraction.moderate',
      'jury.quorum_fraction.severe',
      'jury.threshold_fraction.minor',
      'jury.threshold_fraction.moderate',
      'jury.threshold_fraction.severe',
      'jury.constraints.no_majority_from_same_sponsor_cluster',
      'jury.constraints.geographic_diversity_preferred',
      'jury.constraints.no_recent_juror_repeat',
      'jury.constraints.juror_cooldown_days',
      'jury.constraints.no_same_endorsement_chain',
      'jury.constraints.max_retries_before_relax',
      'jury.max_concurrent_assignments_per_juror_total',
      'appeal.panel_size_multiplier',
      'appeal.panel_size_floor_increment',
      'appeal.threshold_tier_bump',
      'appeal.window_days',
      'appeal.auto_select_on_appeal_acceptance'
  );
  ```
- **MIRROR**: `migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql` + `/down.sql` (v1-AD-a full files).

#### Task 8: MANDATORY pre-commit reconciliation gate

Before `git commit` of Task 7's migration, run the reconciliation gate (mirroring v1-AD-a Task 7 advisor-edit-#1):

```bash
# Count SEEDED_KEYS_WITH_CONSTS v1-JM-a extension rows (lines after the
# v1-AD-a block comment, before the closing `];`)
awk '/v1-JM-a additions/,/^];$/' crates/api/api/src/governance/config.rs | \
  grep -cE '^\s*\("' 
# Expected: 27

# Count INSERT rows in Task 7's up.sql
grep -cE "^\s*\('instance'," migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql
# Expected: 27

# Count DELETE keys in Task 7's down.sql
grep -cE "^\s*'" migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/down.sql
# Expected: 27

# Cross-check: every key in up.sql INSERT appears in down.sql DELETE
comm -23 \
  <(grep -oE "'(jury|appeal)\\.[a-z_.]+'" migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql | sort -u) \
  <(grep -oE "'(jury|appeal)\\.[a-z_.]+'" migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/down.sql | sort -u)
# Expected: empty (no keys in up.sql missing from down.sql)

# Dry-run the parametric parity test
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server config_parity_round_trip > .claude/PRPs/debug/v1-JM-a-task7-precommit-parity.log 2>&1"
echo "precommit parity exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-a-task7-precommit-parity.log
```

If any of the four counts disagrees or the parity test fails, DO NOT commit. Three outcomes:
- **(a) SEEDED_KEYS_WITH_CONSTS count > up.sql count**: add missing INSERT rows OR remove superfluous SEEDED_KEYS entries until they match.
- **(b) SEEDED_KEYS_WITH_CONSTS count < up.sql count**: add missing `SEEDED_KEYS_WITH_CONSTS` entries + matching `const_default_*` match arms + `CONFIG_KEY_METADATA` entries.
- **(c) up.sql ↔ down.sql mismatch**: fix down.sql to mirror up.sql's key list exactly.
- **(d) neither side matches PRD §10**: open a DQ entry (attribution `impl-self-resolved` is NOT allowed here — PRD §10 is the source of truth; label `answered_by: null`, wait for advisor).

Rationale: converts count drift from "surprise at test time" to "explicit reconciliation at migration-write time" per v1-AD-a advisor edit #1.

- **GOTCHA**: `ON CONFLICT (scope, key, valid_from) DO NOTHING` — v1-AD-a GOTCHA reminder: `valid_from` defaults to `now()`, so two schema_setup runs at different times do NOT duplicate since the conflict key requires exact `valid_from` match. Inside a single transaction, all 27 rows share one `now()` and conflict correctly.
- **GOTCHA**: `appeal.window_days = 7` matches the existing v0 `APPEAL_WINDOW_DAYS = 7` in `submit_jury_vote.rs:88` — v1-JM-c will read this key live at decision time to replace the const.
- **GOTCHA**: Timestamp `000200-0000` is after the column migration `000100-0000` because the seed rows need `governance_config` schema which is already v0 (from `2026-04-18-000000-0000_add_governance_config`). Ordering is not semantically required for dependencies in this case but kept for temporal consistency within the v1-JM-a set.
- **VALIDATE**: already in the reconciliation gate above.
- **COMMIT MESSAGE**: `feat(v1-JM-a): seed 27 jury/appeal governance_config rows — idempotent + matches SEEDED_KEYS_WITH_CONSTS (task 7)`

### Task 9: UPDATE `crates/db_schema/src/source/governance/governance_log.rs` + `crates/api/api/src/governance/governance_log.rs` + `.claude/rules/governance-log-entry-kind-registry.md` — 6 new ENTRY_KIND consts + registry section

- **ACTION**: Three-file atomic commit:
  1. DEFINE 6 new `ENTRY_KIND_*` consts in `crates/db_schema/src/source/governance/governance_log.rs` (after `ENTRY_KIND_RULE_SET_VERSION_CREATED`).
  2. RE-EXPORT 6 new consts alphabetically in `crates/api/api/src/governance/governance_log.rs`.
  3. POPULATE the `jury-mechanics-v1` section in `.claude/rules/governance-log-entry-kind-registry.md` (replacing the reservation stub with 6 populated rows matching the v1-AD-c populated-section shape).
- **IMPLEMENT**: See §10.10 exact patterns for files 1 and 2. For file 3, write:

  ```markdown
  ## v1-JM-a entry kinds (6, this sub-phase)

  Landed alongside task 9's dual-file edit. v1-JM-a writes the const
  declarations only; emitting call sites land in v1-JM-b (constraint
  relaxation), v1-JM-b (severity_tier_frozen on admin_assign_jury),
  v1-JM-d (appeal_* kinds), and v1-JM-d background job
  (appeal_window_expired).

  | Rust const | `&str` value | Source | Emitting handler | Semantic |
  |---|---|---|---|---|
  | `ENTRY_KIND_JURY_CONSTRAINT_RELAXED` | `jury_constraint_relaxed` | v1-JM-a const; v1-JM-b call site | v1-JM-b `crates/api/api/src/governance/admin_assign_jury.rs::select_eligible_jurors` (pending) | R1/R2/R3 relaxation cascade fired; payload carries `{case_id, constraint_dropped, reason, phase}` |
  | `ENTRY_KIND_APPEAL_PANEL_ASSEMBLED` | `appeal_panel_assembled` | v1-JM-a const; v1-JM-d call site | v1-JM-d `crates/api/api_crud/src/governance/request_appeal.rs::select_appeal_panel` + `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs` (both pending) | Appeal jury seated (original jurors excluded, higher threshold tier); payload carries `{case_id, new_panel_pseudonyms, excluded_juror_count, appeal_threshold_count}` |
  | `ENTRY_KIND_APPEAL_DECIDED` | `appeal_decided` | v1-JM-a const; v1-JM-d call site | v1-JM-d `submit_jury_vote.rs` appeal-panel vote-tally path (pending) | Appeal panel returned a verdict; payload mirrors the original case_decided shape plus `{original_winning_decision, appeal_winning_decision}` |
  | `ENTRY_KIND_APPEAL_REJECTED` | `appeal_rejected` | v1-JM-a const; v1-JM-d call site | v1-JM-d `admin_reject_appeal.rs` (handler name TBD; pending) | Admin denied the appeal request before the appeal panel was seated; payload carries `{case_id, reason, reviewer_pseudonym}` |
  | `ENTRY_KIND_APPEAL_WINDOW_EXPIRED` | `appeal_window_expired` | v1-JM-a const; v1-JM-d background job | v1-JM-d `crates/server/src/governance.rs` appeal-window-expiry scheduled task (pending) | Cron tick found a `Decided` case with `appeal_window_expires_at < now()`; case flipped to `Closed`; payload carries `{case_id, decided_at, window_expired_at}` |
  | `ENTRY_KIND_SEVERITY_TIER_FROZEN` | `severity_tier_frozen` | v1-JM-a const; v1-JM-b call site | v1-JM-b `admin_assign_jury.rs` (pending) | `moderation_case.severity_tier` snapshotted at jury-assemble time per PRD §9.2; payload carries `{case_id, severity_tier, status_tier, panel_size_snapshot, quorum_snapshot, threshold_count_snapshot, cascade_resolved_path}` |
  ```

  And bump the Acceptance invariants count at the bottom of the registry from `26` → `32` (19 v0 + 4 Phase 6 + 2 v1-AD-a + 1 v1-AD-c + 6 v1-JM-a).
- **MIRROR**: `.claude/rules/governance-log-entry-kind-registry.md` current state (v1-AD-a section + v1-AD-c section + the jury-mechanics-v1 reservation stub) — replace the stub with a populated section matching the shape of the v1-AD-a + v1-AD-c populated sections.
- **GOTCHA**: `pub use` ordering is alphabetical — **strictly** alphabetical, not "alphabetical within the new additions". Insert the six new names into the existing alphabetical sort:
  - `ENTRY_KIND_APPEAL_DECIDED` between `ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED` and `ENTRY_KIND_APPEAL_PANEL_ASSEMBLED` (new).
  - `ENTRY_KIND_APPEAL_PANEL_ASSEMBLED` after the DECIDED new entry, before `ENTRY_KIND_APPEAL_REJECTED` (new).
  - `ENTRY_KIND_APPEAL_REJECTED` after PANEL, before the existing `ENTRY_KIND_APPEAL_REQUESTED`.
  - `ENTRY_KIND_APPEAL_WINDOW_EXPIRED` after `ENTRY_KIND_APPEAL_REQUESTED`, before `ENTRY_KIND_CAPABILITY_CHANGED`.
  - `ENTRY_KIND_JURY_CONSTRAINT_RELAXED` between `ENTRY_KIND_JURY_ASSIGNED` and `ENTRY_KIND_JURY_DECLINED`.
  - `ENTRY_KIND_SEVERITY_TIER_FROZEN` between `ENTRY_KIND_SANCTION_CREATED` and `ENTRY_KIND_SPONSOR_LIABILITY_APPLIED`.
- **GOTCHA**: All three files land in the **same commit** — a schema definition without the shim re-export breaks callers importing from the api path; a shim re-export without the schema definition fails to compile; a registry update without either is informational drift.
- **GOTCHA**: Registry invariant count check (per `.claude/rules/governance-log-entry-kind-registry.md` top):

  ```bash
  rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
  # Expected after this task: 32 (was 26)

  rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs \
    | awk -F: '/ENTRY_KIND_/ {print}' \
    | grep -oE '"[a-z_]+"' | sort | uniq -d
  # Expected: empty (no duplicate literals)
  ```
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-a-task9-check.log 2>&1"
  echo "exit: $?"   # expect 0

  # Const-count invariant
  rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
  # expect 32

  # Shim re-export count invariant (api shim)
  rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l
  # expect 32 (matches the db_schema define count)
  ```
- **COMMIT MESSAGE**: `feat(v1-JM-a): add 6 ENTRY_KIND consts (db_schema define + api shim re-export) + registry v1-JM-a section (task 9)`

### Task 10: UPDATE `crates/server/tests/e2e.rs` — extend migration round-trip + add backfill smoke test

- **ACTION**: Three sub-edits in `crates/server/tests/e2e.rs`:
  1. Extend `PHASE_1_MIGRATION_COUNT` from 9 to 12 (3 new JM-a migrations).
  2. Extend `phase1_migrations_round_trip` post-condition probes with the new `jury_constraint_violation_log` table + the 3 new enum pg_type entries (`severity_tier`, `case_status_tier`, `jury_assignment_role`).
  3. Add new e2e test `v1_jm_a_backfill_populates_v0_snapshot` — seeds a pre-v1 case shape directly via SQL (matching pre-migration state), applies JM-a migrations, asserts backfill values are Minor/Regular/5/3/3 per PRD §8.4.
- **IMPLEMENT**: 

  **Sub-edit 1** — `crates/server/tests/e2e.rs:321`:

  ```rust
  /// Count of branch-added migrations that must revert cleanly for the
  /// round-trip assertions below to hold. Updated to 12 in v1-JM-a (adds
  /// 3: add_jury_mechanics_enums, add_jury_mechanics_columns,
  /// seed_v1_jm_config_keys). Breakdown:
  ///   - 6 Phase 1 migrations
  ///   - 2 Phase 5a migrations
  ///   - 1 Phase 5b migration (restoration)
  ///   - 4 v1-AD-a migrations (rule_set_versions, sponsor_allowlist,
  ///     case_applied_config_snapshot, seed_v1_config_keys)
  ///   - 3 v1-JM-a migrations
  ///  = 16 total? No — v1-AD-a is NOT grandfathered into this test's
  ///    reverts because AD-a's migrations are additive-only (no
  ///    ROUND_TRIP_TABLE_LIST entries at AD-a time). The count here is
  ///    "contiguous-governance-bootstrap migrations that the round-trip
  ///    test reverts LIFO". JM-a extends this count because JM-a's
  ///    migrations add new tables + enums that must revert cleanly.
  const PHASE_1_MIGRATION_COUNT: u64 = 12;
  ```

  **Wait — reconciliation needed.** v1-AD-a shipped 4 migrations. Did the PHASE_1_MIGRATION_COUNT move to 13 or stay at 9? The Explore agent §3 finding says it's currently 9. This means v1-AD-a did NOT extend the round-trip revert count — either because AD-a's tables are tested differently, or because the count only tracks the contiguous Phase-1-style bootstrap block. **Impl agent: verify the current count at task start and adjust both the constant and the comment accordingly.** If AD-a's 4 migrations should have been in the count, the drift is a pre-existing v1-AD-a defect, not a JM-a concern — file a DQ entry (blocking, answered_by: null) citing this §10 note and the discrepancy; do NOT self-resolve by silently extending.

  **Sub-edit 2** — extend the table lists in both post-condition loops (around lines 344–358 and 381–395) to add `"jury_constraint_violation_log"`. Extend the pg_type enum-drop assertion loop (lines 406–433) with `"severity_tier"`, `"case_status_tier"`, `"jury_assignment_role"`.

  **Sub-edit 3** — new test in e2e.rs:

  ```rust
  /// v1-JM-a backfill smoke test (PRD §8.4 + §11). Seeds a pre-v1-style
  /// moderation_case row BEFORE applying JM-a's migrations (i.e. simulates
  /// a v0 case in flight at v1-JM-a ship date), then applies migrations,
  /// then asserts the backfill UPDATE populated the snapshot columns to
  /// Minor/Regular/5/3/3 per PRD §8.4. This is the v1-JM-a-specific
  /// regression that v1-JM-e's `v0_case_completes_under_v0_rules_after_v1_config_flip`
  /// capstone builds on.
  #[tokio::test]
  async fn v1_jm_a_backfill_populates_v0_snapshot() -> LemmyResult<()> {
    // Strategy: apply all migrations UP TO (but not including) v1-JM-a's
    // add_jury_mechanics_columns (timestamp 2026-04-23-000100), insert a
    // moderation_case row with v0 shape (no snapshot columns), then apply
    // the remaining migrations (including the backfill UPDATE in JM-a's
    // up.sql), then assert the row has Minor/Regular/5/3/3 per §8.4.
    //
    // Concretely: the `schema_setup::run` runner applies ALL pending
    // migrations. To test mid-state, we:
    //   1. Start fresh Postgres container.
    //   2. Apply all migrations (up to HEAD including JM-a).
    //   3. Revert JM-a's three migrations LIFO (count=3).
    //   4. Seed a pre-v1 case row with default severity+status only
    //      (no JM-a columns yet).
    //   5. Re-apply JM-a's three migrations in order.
    //   6. Query the row, assert snapshot columns are backfilled.
    //
    // This exercises the exact up/down/up cycle that production will see
    // if an admin deploys JM-a, rolls it back (down.sql), and re-deploys.
    //
    // [full test body per Lemmy e2e harness conventions — 50-80 lines]
  }
  ```
- **MIRROR**: `crates/server/tests/e2e.rs:306–454` (full `phase1_migrations_round_trip` as the structural template); the backfill test follows the async + `LemmyResult<()>` pattern from `feedback_clippy_test_style.md` (no `unwrap`/`expect`, use `?` operator).
- **GOTCHA**: The backfill smoke test assertion compares **exact integer values** (5, 3, 3) and **exact enum string values** (`'Minor'`, `'Regular'`). Off-by-one on severity tier name or quorum=3 vs quorum_fraction=0.6 rounding would be caught here and not by any parity test.
- **GOTCHA**: Docker daemon must be running — per DQ #44, the `docker ps` probe in `/prp-core:prp-implement §4.2.0` runs before every `cargo test --test e2e`. The test itself uses testcontainers-rs.
- **VALIDATE**:

  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server phase1_migrations_round_trip > .claude/PRPs/debug/v1-JM-a-task10-phase1.log 2>&1"
  echo "phase1 exit: $?"   # expect 0 — confirms JM-a migrations revert/re-apply cleanly
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server v1_jm_a_backfill_populates_v0_snapshot > .claude/PRPs/debug/v1-JM-a-task10-backfill.log 2>&1"
  echo "backfill exit: $?"   # expect 0 — confirms backfill values match PRD §8.4
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server config_parity_round_trip > .claude/PRPs/debug/v1-JM-a-task10-parity.log 2>&1"
  echo "parity exit: $?"   # expect 0 — confirms 88 SEEDED_KEYS rows all typed-read successfully
  ```
- **COMMIT MESSAGE**: `test(v1-JM-a): extend phase1_migrations_round_trip (enums + jcvl) + new backfill smoke test (task 10)`

### Task 11: WRITE `.claude/PRPs/reports/phase-v1-JM-a-retro.md` — retrospective before PR

- **ACTION**: Write the retrospective per `feedback_retro_not_report.md` — honest assessment of what happened, lessons, DQs to file. File lives under `.claude/PRPs/reports/` not `.claude/PRPs/plans/`.
- **IMPLEMENT**: Template sections (mirror `v1-admin-dashboard-d-retro.md` structure): Executive summary, Timeline / what shipped, §1 what went well, §2 what surprised us, §3 what to carry forward (GH issues for v1/v1.5/v2 candidates per DQ #46), §4 handoff notes for v1-JM-b.
- **MIRROR**: `.claude/PRPs/reports/v1-admin-dashboard-d-retro.md` as the most recent retro precedent.
- **GOTCHA**: Retro is written BEFORE `gh pr create` so CodeRabbit review can pull context from the retro (per feedback memory). PR body references the retro at `.claude/PRPs/reports/phase-v1-JM-a-retro.md`.
- **VALIDATE**: Manual read-through; no automated test for retro shape.
- **COMMIT MESSAGE**: `docs(v1-JM-a): phase retrospective before PR open (task 11)`

### Task 12 (optional, BM session): OPEN PR `phase-v1-JM-a` → `governance-v0`

This task is NOT executed by the impl session. After Task 11 commits, control hands off to the branch-manager (BM) session per `.claude/rules/branch-manager.md`. BM runs:
- `git push -u origin phase-v1-JM-a`
- `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-JM-a --title "v1-JM-a: jury-mechanics schema + enums + backfill" --body-file .claude/PRPs/plans/phase-v1-JM-a.plan.md` (abbreviated; actual body composed from retro + commit log)
- Waits for CodeRabbit review.

Impl session responsibility ends at Task 11 commit.

---

## 14. Testing strategy

Per [IMPLEMENTATION-PLAN-v0.md §5](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): **integration-only** for Brehon (no unit tests by default). Parity tests + config assertions are an exception because they protect compile-time invariants that e2e would catch too late. All integration tests live in `crates/server/tests/e2e.rs`.

### Tests added in this sub-phase

| Test Name | Where | What It Validates |
|---|---|---|
| `parity::seeded_keys_count_matches_const_count` (EXTENDED) | `crates/api/api/src/governance/config.rs` | Now asserts `SEEDED_KEYS_WITH_CONSTS.len() == EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD + EXPECTED_SEED_COUNT_V1_JM` (= 88) |
| `parity::every_seeded_key_has_metadata` (UNCHANGED BODY, BROADER COVERAGE) | same file | Iterates 88 entries vs 61 pre-JM-a |
| `parity::every_seeded_key_has_const_fallback` (UNCHANGED BODY, BROADER COVERAGE) | same file | Iterates 88 entries vs 61 pre-JM-a; ensures every key has a matching Rust const |
| `parity::rule_set_active_version_not_in_seeded_keys` (UNCHANGED) | same file | v1-AD-a invariant still holds |
| `phase1_migrations_round_trip` (EXTENDED) | `crates/server/tests/e2e.rs` | Adds `jury_constraint_violation_log` + 3 new enum types to the revert-and-reapply round-trip |
| `config_parity_round_trip` (UNCHANGED BODY, BROADER COVERAGE) | `crates/server/tests/e2e.rs:1361–1418` | Runs one typed read per `SEEDED_KEYS_WITH_CONSTS` entry (88 post-JM-a) against real Postgres with seeded rows |
| `v1_jm_a_backfill_populates_v0_snapshot` (NEW) | `crates/server/tests/e2e.rs` | Seeds a pre-v1 case, applies JM-a migrations, asserts Minor/Regular/5/3/3 backfill per PRD §8.4 |

### Edge cases covered by existing tests

- Forward-revert-forward migration round-trip (catches broken down.sql).
- All 3 new enum types are dropped by down.sql (pg_type assertion loop).
- All 27 new config keys round-trip through the typed accessor + `ConfigCache`.
- Every new key has a matching `DEFAULT_*` const (no DB-only key can land).

### Edge cases NOT covered by v1-JM-a (deferred to later sub-phases)

- Panel-size cascade read (`get_int_cascade("jury.panel_size", status, severity)`) — v1-JM-b tests this.
- Backfill idempotency on re-apply with existing v1 rows (up-down-seed-new-case-up sequence) — v1-JM-b's first admin_assign_jury write will test this implicitly.
- `selected_under_constraints` JSONB write + query — v1-JM-b.
- `appeal_window_expires_at` write + query — v1-JM-c.
- `jury_constraint_violation_log` row write — v1-JM-b.
- Appeal `role = 'Appeal'` row write + original-jurors-excluded query — v1-JM-d.

---

## 15. Validation commands (DoD)

Use these exact commands — do NOT substitute `cargo` direct, must go through the `scripts/brehon/` wrappers per Windows libpq + vcvars requirements. All outputs redirect to file + read via `tail -20` per `.claude/rules/cargo-output-capture.md` + `no-cargo-output-paste.md`.

### Level 1: per-task `cargo check` (run after every task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace > .claude/PRPs/debug/v1-JM-a-lvl1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-a-lvl1-check.log
```
**EXPECT:** exit 0, zero errors, zero warnings.

### Level 2: `--features full` (tasks 3, 4, 5, 6, 9 specifically)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-a-lvl2-check-full.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-a-lvl2-check-full.log
```
**EXPECT:** exit 0. `--features full` activates the `DbEnum` + `ts-rs` derives on the new enums and the `Queryable/Selectable/Insertable` derives on the new Diesel structs.

### Level 3: parity unit tests (Task 6+)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --lib parity > .claude/PRPs/debug/v1-JM-a-lvl3-parity.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-a-lvl3-parity.log
```
**EXPECT:** exit 0. All four parity tests in `config.rs::parity` module pass: `seeded_keys_count_matches_const_count` (88 rows), `every_seeded_key_has_metadata`, `every_seeded_key_has_const_fallback`, `rule_set_active_version_not_in_seeded_keys`.

### Level 4: e2e migration round-trip + backfill + config parity (Task 10)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server phase1_migrations_round_trip v1_jm_a_backfill_populates_v0_snapshot config_parity_round_trip > .claude/PRPs/debug/v1-JM-a-lvl4-e2e.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-JM-a-lvl4-e2e.log
```
**EXPECT:** exit 0. The three tests pass under a freshly-started Postgres container:
1. `phase1_migrations_round_trip` applies + reverts + re-applies JM-a's 3 new migrations cleanly; asserts enum types dropped on revert; asserts `jury_constraint_violation_log` table dropped on revert.
2. `v1_jm_a_backfill_populates_v0_snapshot` seeds pre-v1 case, applies backfill, confirms `severity_tier='Minor'`, `status_tier='Regular'`, `panel_size_snapshot=5`, `quorum_snapshot=3`, `threshold_count_snapshot=3`, `appeal_window_expires_at = decided_at + 7d` (or closed_at).
3. `config_parity_round_trip` walks all 88 `SEEDED_KEYS_WITH_CONSTS` entries, confirms each returns a typed value from the actual Postgres instance.

### Level 5: clippy (per plan-drift CI)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-a-lvl5-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-a-lvl5-clippy.log
```
**EXPECT:** exit 0, zero warnings treated as errors. `--no-deps` avoids upstream lint debt per `feedback_clippy_vs_check_wrapper.md`. `--features full` activates governance code paths so the governance enums + structs get linted.

### Level 6: PM-hook integrity (belt-and-braces per `.claude/rules/pm-plugin-hooks-stable.md`)

```bash
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  rg -q "\"$h\"" crates/ || { echo "MISSING HOOK LITERAL: $h"; exit 1; }
done
echo "all 6 PM hooks present"
```
**EXPECT:** `all 6 PM hooks present` and exit 0. v1-JM-a touches no PM code, so the invariant should hold trivially — this check confirms no accidental regression via schema or enum renames.

### Level 7: ENTRY_KIND registry invariants (Task 9 specifically)

```bash
rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
# expect 32 (was 26 pre-JM-a; +6)

rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l
# expect 32 (shim re-export parity)

rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs \
  | awk -F: '/ENTRY_KIND_/ {print}' \
  | grep -oE '"[a-z_]+"' | sort | uniq -d
# expect empty (no duplicate literals)
```

### Level 8: MANUAL validation

After Level 4 passes end-to-end, spot-check:
```bash
# Quick sanity: confirm the migration files exist at the right paths
ls migrations/2026-04-23-000000-0000_add_jury_mechanics_enums/
ls migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/
ls migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/
# Each should show exactly `up.sql` and `down.sql`

# Confirm the registry entry is populated (not reserved)
grep -A 10 "^## v1-JM-a entry kinds" .claude/rules/governance-log-entry-kind-registry.md | head -20
# Expected: shows a row table with ENTRY_KIND_JURY_CONSTRAINT_RELAXED, ENTRY_KIND_APPEAL_PANEL_ASSEMBLED, ...
```

---

## 16. Acceptance criteria

- [ ] All 11 impl tasks completed in dependency order (Task 0 is pre-flight, no commit; Tasks 1–11 each one commit)
- [ ] Level 1: `cargo check --workspace` exits 0 after every task
- [ ] Level 2: `cargo check --workspace --features full` exits 0 after tasks 3, 4, 5, 6, 9
- [ ] Level 3: all 4 parity tests pass
- [ ] Level 4: all 3 e2e tests pass (`phase1_migrations_round_trip`, `v1_jm_a_backfill_populates_v0_snapshot`, `config_parity_round_trip`)
- [ ] Level 5: `cargo clippy --workspace --features full --no-deps -- -D warnings` exits 0
- [ ] Level 6: all 6 PM hook string literals still present under `crates/`
- [ ] Level 7: ENTRY_KIND invariants hold (32 defines, 32 re-exports, no duplicate literals)
- [ ] `SEEDED_KEYS_WITH_CONSTS.len() == EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD + EXPECTED_SEED_COUNT_V1_JM` (parametric; Task 8 reconciliation gate authoritative)
- [ ] `EXPECTED_SEED_COUNT == 34` (unchanged; v0 invariant preserved)
- [ ] `EXPECTED_SEED_COUNT_V1_AD == 27` (unchanged; v1-AD-a invariant preserved)
- [ ] `EXPECTED_SEED_COUNT_V1_JM` matches the actual number of v1-JM-a-added tuples in `SEEDED_KEYS_WITH_CONSTS` AND the actual number of `ON CONFLICT DO NOTHING` INSERT rows in Task 7's `up.sql` AND the actual number of DELETE keys in Task 7's `down.sql`
- [ ] No contradictions with ADRs 007/010/013/015 (schema shape + backfill defaults + EmergencyRemove handling + pseudonymity all preserved)
- [ ] 6 new `ENTRY_KIND_*` consts present in both the `db_schema` definition file AND the api shim's `pub use` block (alphabetical order preserved)
- [ ] `.claude/rules/governance-log-entry-kind-registry.md` v1-JM-a section populated with 6 rows matching the shape of v1-AD-c's populated section; acceptance-invariants count bumped to 32
- [ ] New migrations (3) apply cleanly, revert cleanly, re-apply cleanly
- [ ] Backfill UPDATE sets Minor/Regular/5/3/3 on every pre-v1 case (verified by `v1_jm_a_backfill_populates_v0_snapshot`)
- [ ] No handler file edited (per §12 OUT list): `admin_assign_jury.rs`, `submit_jury_vote.rs`, `request_appeal.rs`, `accept_jury_assignment.rs`, `decline_jury_assignment.rs`, `admin_emergency_remove.rs` have no changes in this PR
- [ ] No route file edited (per §12 OUT list): `crates/api/routes/src/governance.rs` has no changes
- [ ] No DTO file edited (per §12 OUT list): `crates/api/api_common/src/governance.rs` has no changes
- [ ] Retro written at `.claude/PRPs/reports/phase-v1-JM-a-retro.md` BEFORE PR opens

---

## 17. Completion checklist

- [ ] Task 0 pre-flight ran clean (branch assertion + probes 0/1/3/4 all green)
- [ ] Task 1–10 each shipped one commit with the `feat(v1-JM-a): ... (task N)` or `test(v1-JM-a): ... (task N)` subject pattern
- [ ] Task 11 retro committed with `docs(v1-JM-a): phase retrospective before PR open (task 11)`
- [ ] Level 1–7 validation passed at final commit
- [ ] `git log governance-v0..HEAD --oneline` shows 11 v1-JM-a commits (tasks 1–11)
- [ ] No file under `crates/api/api/src/governance/{submit_jury_vote,admin_assign_jury,request_appeal,accept_jury_assignment,decline_jury_assignment,admin_emergency_remove}.rs` modified
- [ ] No file under `crates/api/routes/` modified
- [ ] No file under `crates/api/api_common/` modified
- [ ] Plan marked complete in `.claude/PRPs/plans/` (the plan file itself stays; the `completed/` subdir move happens at post-merge retrospective time per `feedback_pr_per_phase.md`)
- [ ] Handoff complete; BM session takes over for push + PR create

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Plan-count drift: 27 estimated, actual count differs after reconciliation | MED | LOW | Task 8 reconciliation gate is MANDATORY before Task 7 commit. Drift triggers either adjust-the-count or file-a-DQ paths — never self-resolve. |
| Backfill UPDATE misses edge case (case with decided_at NULL but closed_at not-null, or vice versa) | LOW | HIGH | `COALESCE(closed_at, CASE WHEN decided_at ...)` covers all four combinations; `v1_jm_a_backfill_populates_v0_snapshot` asserts the result. |
| Dual-file ENTRY_KIND edit lands asymmetrically (schema const without shim re-export OR vice versa) | LOW | MED | Task 9 is single-commit-three-files; CI check `rg -c pub const ...` vs `rg ENTRY_KIND_ ... shim | wc -l` catches asymmetry. |
| CodeRabbit flags the v1-AD-a `jury.severity_thresholds.*` (AD-a-owned text display strings) vs `jury.threshold_fraction.*` (JM-a-owned float cascade) as "duplicate keys" | MED | LOW | Preemptive comment in `SEEDED_KEYS_WITH_CONSTS` block explaining the distinct roles. Expected CR response: rebuttal (per `feedback_coderabbit_triage_four_buckets_confirmed.md`). |
| `DbValueStyle = "verbatim"` PascalCase variants break on a case-sensitive diesel query that expects snake_case | LOW | HIGH | Mirrors every other governance enum (CaseStatus, JuryDecision, SanctionAction). Breakage here would break those too; the e2e test's `"SELECT count(*) FROM pg_type WHERE typname = 'severity_tier'"` query confirms Postgres-side typname is lowercase while Rust-side variant stays PascalCase, per DbEnum convention. |
| v1-AD-a migration count `PHASE_1_MIGRATION_COUNT = 9` (not 13) indicates AD-a didn't extend the round-trip test — JM-a must decide whether to extend by 3 (→12) or by 7 (→16 catching AD-a drift) | LOW | MED | Impl agent files a DQ at Task 10 start if they find AD-a is missing from the count. Do NOT silently extend by 7. If the AD-a drift is a real defect, advisor decides the correct fix (retrofit AD-a in a separate chore commit, or accept the scope limitation). |
| Testcontainers-rs fails to pull postgres:16 image during a `cargo test --test e2e` run due to Docker daemon or network state | LOW | LOW | DQ #44 probe 0 (docker ps) in both pre-phase-harness-audit AND `/prp-core:prp-implement §4.2.0` catches the Docker-down case; network pulls are retried by testcontainers-rs. |
| Upstream Lemmy 1.0-beta rebase changes `moderation_case` schema mid-phase | LOW | MED | `CLAUDE.md` fork pin; `phase-branch.md` isolates JM-a on its own branch; rebase decisions are deferred to post-merge per `feedback_lemmy_rebase_cadence.md`. |
| CodeRabbit flags the 27-row seed migration as "large batch" because each INSERT row is a new governance_config row, invoking ADR-008 append-only concerns | LOW | LOW | `ON CONFLICT DO NOTHING` is idempotent; `governance_config` table's hash-chain trigger on `governance_log` (Phase 4b) is not invoked by `governance_config` writes (they don't land in `governance_log`). Expected CR response: clean pass. |

---

## 19. Notes

- **v1-AD-a precedent is the contract.** Every decision in this plan that says "follows v1-AD-a" maps to a specific section of `.claude/PRPs/plans/completed/v1-admin-dashboard-a.plan.md`. The impl agent should treat divergences from v1-AD-a as suspicious — if a JM-a step is doing something different from AD-a's equivalent step, there's a reason (usually enum-vs-table difference), but confirm the reason explicitly.
- **The `EXPECTED_SEED_COUNT_V1_JM` pattern is parametric per advisor directive #4.** v1-SL-a, v1-rep-tuning-r1, v1-FI-a will each add their own `EXPECTED_SEED_COUNT_V1_*` constant. Each parametric count stays pinned; only the aggregate `SEEDED_KEYS_WITH_CONSTS.len()` drifts.
- **JM-a deliberately does NOT touch any handler.** A JM-a session that finds itself in `submit_jury_vote.rs` has drifted scope. The purpose of the JM split (per PRD §16 Decisions Log) is to isolate schema from the densest-reasoning code (constraint relaxation in JM-b, 9-step handler in JM-c, appeals in JM-d). Preserve this isolation.
- **Task 10's backfill smoke test is the load-bearing regression** for the rest of JM. If it passes, JM-b/c/d/e can safely assume pre-v1 cases have sane snapshot values. If it fails or is skipped, every downstream sub-phase has to rewrite its compatibility tests from scratch.
- **Follow-up GH issues (v1/v2 candidates per DQ #46)** — JM-a has three identifiable candidates to file after merge:
  - `v1.5-candidate`: composable diversity constraints (per PRD §2 OUT line 128)
  - `v1.5-candidate`: `no_same_endorsement_chain` constraint activation (per PRD §2 OUT line 130 + Table §5.1)
  - `v2-candidate`: cross-instance jury eligibility (per PRD §2 OUT line 126 + OQ-V1-JM-04 parked)

  Impl session writes them into the retro (Task 11) with draft issue text; user / advisor decides which to actually file post-merge.

---

## 20. Confidence score

**8.5/10** for one-pass implementation success.

**Rationale:**
- **Plus**: Every task mirrors a specific v1-AD-a task byte-for-byte. The pattern is fresh (v1-AD-a merged 2026-04-23), the impl agent will have high-quality mechanical work rather than design work. Reconciliation gate at Task 8 catches the only realistic drift class (27 keys count). The backfill smoke test is narrowly scoped (6 assertions) and reproducible.
- **Minus**: The 27-key count is a plan-time estimate. If the PRD §10 matrix enumeration has drifted from the `SEEDED_KEYS_WITH_CONSTS` contribution sub-table (admin-dashboard vs jury-mechanics vs sponsor-liability ownership), Task 8 catches it but may require advisor input to resolve. Task 10's `PHASE_1_MIGRATION_COUNT` reconciliation (9 → 12 vs 13 → 16) is an unknown that needs first-iteration verification.
- **Risk floor**: `cargo check --features full` is the early-fail signal. If any of tasks 3–6 breaks it, the remainder of the plan cannot execute cleanly — but that failure is a compile-time signal, not a semantic drift.

**Next step:** `/prp-core:prp-implement .claude/PRPs/plans/phase-v1-JM-a.plan.md` (advisor provisions `phase-v1-JM-a` worktree per PRD §17.3 before invocation).
