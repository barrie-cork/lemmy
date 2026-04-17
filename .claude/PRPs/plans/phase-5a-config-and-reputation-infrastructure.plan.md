# Plan: Phase 5a — `governance_config`, `membership_state`, Reputation Infrastructure, `create_endorsement`

## Table of Contents (line numbers approximate — use `grep -n '^## §'` or `^## §N` for exact coordinates after edits)

- §1  Summary                                       — line ~16
- §2  Source / ADRs / OQs                           — line ~32
- §3  Problem Statement                             — line ~56
- §4  Solution Statement                            — line ~82
- §5  Metadata                                      — line ~118
- §6  Flow Design                                   — line ~138
- §7  Mandatory Reading                             — line ~188
- §8  Patterns to Mirror                            — line ~236
- §9  Files to Change (by task)                     — line ~340
- §10 NOT Building (v0 scope limits)                — line ~386
- §11 Watchpoint coverage matrix                    — line ~406
- §12 Step-by-Step Tasks                            — line ~440
  - §12.0 Task 0 — Harness audit + branch + pagination.rs carry-patch commit
  - §12.1 Task 50 — `governance_config` mig + read
  - §12.2 Task 51 — `membership_state` col + enum
  - §12.3 Task 52 — `crates/db_views/reputation`
  - §12.4 Task 53 — Reputation snapshot calculator
  - §12.5 Task 54 — Snapshot background job
  - §12.6 Task 55 — `create_endorsement` handler
  - §12.7 Task 56 — Phase-close validation + PR
- §13 Testing Strategy
- §14 Validation Commands (Level 0: carry-patch precondition; Levels 1–6)
- §15 Acceptance Criteria + Completion Checklist
- §16 Risks and Mitigations
- §17 Notes + decision-queue pre-seed + carry-patch inventory recommendation

---

## §1. Summary

Phase 5a ships the reputation infrastructure the rest of Phase 5 depends on. The deliverables:

1. A new **`governance_config`** table plus a Rust reader that every handler uses instead of hardcoded constants — with a `valid_from` audit trail, a `CHECK` discriminator on the typed value columns, a `governance_config_current` SQL view, and a compile-time seed-vs-const parity test (per design-soundness review Q2).
2. A deferred-enforcement **`person.membership_state`** column + `MembershipState` enum, grep-guarded by a CI lint script so no v0 handler reads it (per OQ-016 and Watch 7).
3. A new **`crates/db_views/reputation`** view crate (per [04 §4.3]) mirroring the Phase 2 tuple-load + `build_view` pattern.
4. A **reputation snapshot calculator** (`recompute_snapshot`) with the `expires_at`-aware founder cliff, organic-event decay half-life, three capability booleans (`jury_eligible`, `trusted_reporter`, `can_sponsor`), concurrent-recompute guards, and a `capability_changed` governance-log emit on every boolean flip (per Watches 2, 8, 9, 10).
5. A **snapshot recalculation background job** registered in Lemmy's existing `scheduled_tasks` scheduler (per [03 §11] composition-root-stays-declarative + Lemmy's clokwerk convention), with a `BREHON_DISABLE_BACKGROUND_JOBS=1` test override (per S4).
6. A **`create_endorsement` handler** (`POST /api/v4/governance/endorsement`) that dispatches on a config-driven sponsor-gate strategy (`'age' | 'open' | 'closed'`), emits the OQ-013 endorsement deltas, inserts a `surety` row when the sponsee has fewer than two sureties, and records the active strategy in the governance log.

After Phase 5a merges, the reputation mechanic is provably tuneable at runtime, the endorsement lifecycle exists end-to-end, and Phase 5b can wire sponsor-liability without any further infrastructure work. The Phase 4 `report_to_modlog_golden_path` e2e test must still pass — Phase 5a touches migrations the test's `embed_migrations!` picks up automatically, plus a `threshold_score` micros rescale that the golden-path test's admin-forced `ThresholdMet` pattern is already robust to.

---

## §2. Source / ADRs / OQs

**Primary plan references (homeserver authoritative paths — the fork's vendored copies are stale for Phase 5):**

- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\IMPLEMENTATION-PLAN-v0.md` §3 Phase 5a (tasks 50–55) and §4 cross-cutting requirements (hash chain, pseudonyms, redaction, `EmergencyRemove`).
- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\04-data-model-and-api.md` §3 (Diesel models — `ReputationEvent`, `ReputationSnapshot`), §4.3 (reputation view crate shape), §5 (DTOs — `CreateEndorsement` already exists in api_common), §6.1 (`create_endorsement` contract), §7 (routes — `/endorsement`), §13 (drift-stub guidance).
- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\99-decisions-and-open-questions.md` — **ADR-005** (reputation event-sourced + `expires_at`), **ADR-007** (5/3/simple-majority), **ADR-013** (exhaustive match on `CaseStatus`), **ADR-015** (GDPR pseudonyms). **Resolved OQs that govern 5a**: **OQ-004** (juror cap=3), **OQ-013** (`+5`/`+5` endorsement deltas), **OQ-014** (age-only sponsor gate; `can_sponsor` computed-but-unread), **OQ-016** (`membership_state` ships deferred-enforcement), **OQ-022** (founder-multiplier `now()` at case-close — locked in for 5b reads, 5a must not bake a different semantic), **OQ-024** (zero-floor clamp on sponsor-liability — 5a seeds the config key that 5b consumes). **Lean-OPEN**: **OQ-006** (threshold formula — 5a seeds the five config keys; 5b task 58 activates the formula).
- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\01-vision-and-principles.md` §5.2 (honour-price floor rationale — informs the seeded default `sponsor_liability_floor=0`).
- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\02-domain-model.md` §4 (four reputation dimensions, event-sourced with decay).
- `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\06-security-and-threat-model.md` §6.1 (redaction service is the single code path for every string into the log).

**Advisor context:** `C:\Users\barri\Developer\homeserver\.claude\advisor-context-phase-5.md` — **watchpoints 1, 2, 6, 7, 8, 9, 11** fall in 5a scope; **watchpoint 10** (payload PII leakage) fires in 5b but the pattern (`actor_pseudonym_helper::get_or_create` → payload) is established in 5a by the `capability_changed` emitter at task 53.

**Fork-local context:** `CLAUDE.md`, `.claude/rules/*.md` (phase-branch.md, pre-phase-harness-audit.md, cargo-output-capture.md, no-cargo-output-paste.md, decision-queue.md, gh-pr-fork-target.md, view-crate-selectable-template.md).

---

## §3. Problem Statement

After Phase 4b the governance mechanic is runnable end-to-end (`report_to_modlog_golden_path` at `crates/server/tests/e2e.rs:734` passes), but every tuneable value is a Rust `const` recompile-to-change:

- `admin_assign_jury.rs:40` — `PANEL_SIZE: i64 = 5` (plan says config).
- `submit_jury_vote.rs:79–90` — `QUORUM`, `APPEAL_WINDOW_DAYS`, `JUROR_ALIGNED_DELTA=10`, `JUROR_OUTLIER_DELTA=-5`, `REPORTER_ACCURATE_DELTA=10`, `REPORTER_INACCURATE_DELTA=-5` (plan says config; 5b task 56 migrates the four delta consts).
- `create_report.rs:67,71` — `V0_THRESHOLD: i64 = 3`, `V0_REPORTER_WEIGHT: i64 = 1` with `TODO(brehon-fork, phase-5)` markers (5b task 58 replaces these with the OQ-006 formula).
- No reputation snapshot exists — the `reputation_snapshot` table schema is present but no row has ever been written. `admin_assign_jury::select_eligible_jurors` at `admin_assign_jury.rs:162` thus filters only by "not target, not reporter, not deleted, accepted_application" — no reputation gate, no concurrent-cap. 5b task 57 needs a fresh `reputation_snapshot` row per person to `INNER JOIN` against.
- No `/endorsement` route exists. The `CreateEndorsement` DTO at `crates/api/api_common/src/governance.rs:193` is defined but orphaned. The OQ-014 cold-start question resolved to "age-only gate"; Phase 5a is the gate's implementation site.
- The `person.membership_state` column does not exist. OQ-016 demands the column ship in v0 so v1 can flip config without a schema migration on `person` (expensive at scale).
- The snapshot recalculation background job is a stub — `crates/server/src/governance.rs:23` logs "not yet scheduled". Phase 5b needs it live so `jury_eligible` is fresh at jury-assignment time.
- The hash-chain log has nowhere to record capability changes. Without a `capability_changed` entry at snapshot time, admins who raise `thresholds.jury_reliability` will cascade user capability losses into the modlog with no attributable root cause (Watch 11).

**These seven gaps collectively block Phase 5b and 5c.** Phase 5a closes them by shipping the config table + reader, the snapshot calculator + job, the deferred `membership_state` column, the reputation view crate, and the endorsement handler. Nothing else.

---

## §4. Solution Statement

Phase 5a is six substantive tasks + a pre-phase harness audit + a phase-close validation step:

- **Task 50** creates migration `add_governance_config` with a typed-value row shape (`value_int | value_float | value_text | value_bool`), a `CHECK` discriminator, a `valid_from` audit timestamp, an `ALTER TABLE reputation_snapshot ADD COLUMN can_sponsor BOOLEAN NOT NULL DEFAULT false` (so task 53 has a real column to write), a `threshold_score` micros rescale (`UPDATE moderation_case SET threshold_score = threshold_score * 1_000_000`), and 32 seed rows via `INSERT ... ON CONFLICT (scope, key, valid_from) DO NOTHING`. The Rust reader at `crates/api/api/src/governance/config.rs` exposes `get_int`/`get_float`/`get_bool`/`get_text` with `community:<id> → instance → Rust const default` fallback, plus a per-request `ConfigCache` that dedupes repeat reads in one handler. A compile-time test (`seed_and_const_parity`) statically asserts every seeded key has a Rust `const` fallback and every `const` has a seeded row (Watch 1).

- **Task 51** creates migration `add_person_membership_state` — adds `MembershipState` enum (`Member | Provisional | Suspended`) to `crates/db_schema_file/src/enums.rs` and `ALTER TABLE person ADD COLUMN membership_state MembershipState NOT NULL DEFAULT 'member'` using Postgres 11+ fast-path (nullable add + `UPDATE` default + `SET NOT NULL` in separate statements) so the column add does not rewrite the table. Patches `crates/api/api_crud/src/user/create.rs::register` at line 82 to write `config.onboarding.default_membership_state` via the `PersonInsertForm` struct-update at line 476. Ships `scripts/brehon/lint-no-membership-read.sh` (and sibling `lint-no-can-sponsor-read.sh`) that grep-fails the phase close if any handler reads the column outside the register patch + schema files (Watch 7).

- **Task 52** creates the `crates/db_views/reputation` crate following the Phase 2 pattern (`lib.rs` + `impls.rs`, `full` feature, `Cargo.toml` copied from `governance_modlog`). Ships `ReputationSummaryView` and `EndorsementSummaryView` structs (plain Rust, no `Selectable` — the `active_sanctions: i64` field is a kind (c) bare-scalar per `.claude/rules/view-crate-selectable-template.md`, so tuple-load + `build_view` is the only shape that compiles). Three queries: `read_reputation_summary`, `list_endorsements_for_person`, `list_sureties_for_person`.

- **Task 53** creates `crates/api/api/src/governance/reputation_snapshot.rs` with `recompute_snapshot(conn, person_id, community_id, config: &mut ConfigCache) -> LemmyResult<ReputationSnapshot>`. Selects the old snapshot row with `FOR UPDATE` (Watch 9 serialisation), loads `reputation_event WHERE expires_at IS NULL OR expires_at > now()` (Watch 2 founder cliff), applies organic half-life **only when `expires_at.is_none()`** (Watch 8 double-decay guard), computes the three capability booleans from config thresholds, upserts via `ON CONFLICT (person_id, community_id) DO UPDATE`, and emits one `capability_changed` governance-log entry per boolean transition with the pseudonym via `actor_pseudonym_helper::get_or_create` (Watch 10 PII prevention, Watch 11 attribution). Also ships `detect_capability_changes(old, new) -> Vec<CapabilityChange>`, a partial unique index `reputation_snapshot_person_null_community ON reputation_snapshot (person_id) WHERE community_id IS NULL` (dedupe instance-scoped snapshots against Postgres `NULL ≠ NULL` on unique constraints), and the canonical `entry_kind` `pub const` block at the top of `governance_log.rs` enumerating every v0 string so typos fail at compile time.

- **Task 54** replaces the Phase 4b stub in `crates/server/src/governance.rs` with a one-line call-through to `run_snapshot_batch`, and registers the 15-minute interval tick inside Lemmy's existing `crates/routes/src/utils/scheduled_tasks.rs::setup` using `scheduler.every(CTimeUnits::minutes(15)).run(...)` — **not** a raw `tokio::spawn` (advisor drift vs. Lemmy convention resolved in favour of Lemmy; 03 §11 keeps server.rs declarative, and clokwerk is the single scheduler on-disk). The registration closure short-circuits if `std::env::var("BREHON_DISABLE_BACKGROUND_JOBS").as_deref() == Ok("1")` so e2e tests can call `recompute_snapshot` directly without racing (S4). The closure wraps the call in `loop {}`-less `inspect_err` + `.ok()` following the Lemmy-native pattern at `scheduled_tasks.rs:74–80` (Watch 6 — errors logged, scheduler never exits, next tick always retries).

- **Task 55** creates `crates/api/api_crud/src/governance/create_endorsement.rs` → `POST /api/v4/governance/endorsement`. Inside one `run_transaction`: reads `config.onboarding.sponsor_gate_strategy`; dispatches via exhaustive match on a private `SponsorGateStrategy` enum (with a doc-commented `Unknown(String)` fallback arm that logs `warn!` and behaves as `'age'` — no `_ =>` wildcard, per `feedback_clippy_test_style`); enforces max-5-active-endorsements-from-caller and 48h-cooldown-from-caller's-last-endorsement; inserts `endorsement` row; conditionally inserts a `surety` row when `SELECT COUNT(*) FROM surety WHERE sponsored_id = target AND revoked_at IS NULL < 2`; emits two `reputation_event` rows (`+config.deltas.endorsement_created_sponsor` on `EndorsementStrength` for caller, `+config.deltas.endorsement_created_sponsee` on `ParticipationConsistency` for target); calls `recompute_snapshot` for both parties (immediate read-your-writes consistency); emits governance-log `endorsement_created` with both pseudonyms and the active gate-strategy in payload. **`can_sponsor` is not consulted** per OQ-014.

- **Task 56** is the phase-close: run every §14 validation command against the phase HEAD, confirm `report_to_modlog_golden_path` still passes, assemble the completion report, open the PR `phase-5a → governance-v0` via `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-5a` per `.claude/rules/gh-pr-fork-target.md` + `phase-branch.md`.

---

## §5. Metadata

| Field | Value |
|---|---|
| Type | SCHEMA + READ_MODEL + HANDLER + CROSS_CUTTING |
| Complexity | HIGH — first migrations since Phase 1, first background job, first compile-time parity test, first config-driven tunability pattern |
| Crates Affected (code) | `crates/db_schema_file` (enums), `crates/db_schema` (source/governance models), `crates/db_views/reputation` (new), `crates/api/api` (config + reputation_snapshot), `crates/api/api_crud` (create_endorsement + register patch), `crates/api/routes` (lib.rs route wiring + scheduled_tasks.rs), `crates/server` (governance.rs one-line edit) |
| Crates Affected (Cargo.toml) | new: `crates/db_views/reputation/Cargo.toml`; edited: workspace `Cargo.toml`, `crates/api/api/Cargo.toml` (new dep on reputation view crate), `crates/api/api_crud/Cargo.toml` (same) |
| Migrations | `migrations/2026-04-18-000000-0000_add_governance_config`, `migrations/2026-04-18-000100-0000_add_person_membership_state` (both up + down) |
| Lint scripts | `scripts/brehon/lint-no-membership-read.sh`, `scripts/brehon/lint-no-can-sponsor-read.sh` |
| v0 Step | Step 5 from [05 §4] — sub-phase 5a of 5a/5b/5c split |
| Dependencies | Phases 1–4 complete. HEAD pre-5a = `26274db05` |
| Branch | `phase-5a` cut from `governance-v0` — **mandatory before task 50 first commit**, per `.claude/rules/phase-branch.md` |
| PR target | `phase-5a → governance-v0` via `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-5a` |
| Estimated Tasks | 6 substantive (50–55) + task 0 audit (which includes 1 pre-5a carry-patch commit on pagination.rs) + task 56 phase-close = **9 commits total** (1 carry-patch + 6 task commits + 1 completion-report commit + 1 PR-open no-commit). PR task 56 has no code commit; its artefact is the report file + the `gh pr create` call. |

---

## §6. Flow Design

### Before State (at HEAD `26274db05`)

```
╔══════════════════════════════════════════════════════════════════════╗
║  Phase 4 complete — governance mechanic runnable, but every value   ║
║  is a compiled constant:                                             ║
║                                                                      ║
║   create_report.rs      V0_THRESHOLD=3, V0_REPORTER_WEIGHT=1          ║
║   submit_jury_vote.rs   QUORUM=3, APPEAL_WINDOW_DAYS=7,              ║
║                         JUROR_ALIGNED_DELTA=10, OUTLIER=-5,          ║
║                         REPORTER_ACCURATE=10, INACCURATE=-5          ║
║   admin_assign_jury.rs  PANEL_SIZE=5; no reputation gate,            ║
║                         no concurrent-cap                            ║
║                                                                      ║
║  reputation_snapshot    table exists; zero rows                      ║
║  reputation_event       table exists; zero rows                      ║
║  endorsement            table exists; zero rows; DTO orphaned        ║
║  surety                 table exists; zero rows                      ║
║  person.membership_state  column does NOT exist                      ║
║  governance_config      table does NOT exist                         ║
║                                                                      ║
║  crates/server/src/governance.rs  27-line stub logging               ║
║                                    "not yet scheduled"               ║
║  crates/routes/src/utils/scheduled_tasks.rs  3 existing clokwerk     ║
║                                    schedulers; no governance job     ║
║                                                                      ║
║  crates/db_views/governance_case, jury_queue, governance_modlog      ║
║    exist (Phase 2). No reputation view crate.                        ║
╚══════════════════════════════════════════════════════════════════════╝
```

### After State (5a close)

```
╔══════════════════════════════════════════════════════════════════════╗
║  Phase 5a complete — config-driven reputation infra live:           ║
║                                                                      ║
║   governance_config     32 seed rows; typed CHECK; valid_from;       ║
║                         _current view; per-request ConfigCache       ║
║   reputation_snapshot   + can_sponsor col; partial-unique-index       ║
║                         for community_id IS NULL; one row per         ║
║                         person-community after each tick             ║
║   person.membership_state  exists, DEFAULT 'member'; register         ║
║                         patch writes config-driven default; two      ║
║                         grep-guard scripts enforce no v0 reads       ║
║                                                                      ║
║   New handler:                                                       ║
║     POST /api/v4/governance/endorsement  → create_endorsement         ║
║       strategy 'age'   (default) — account-age ≥ 30d gate            ║
║       strategy 'open'  — bypass (recruitment-drive)                  ║
║       strategy 'closed' — reject (lockdown)                          ║
║       unknown-text    — warn + behave as 'age'                       ║
║                                                                      ║
║   New library functions:                                             ║
║     reputation_snapshot::recompute_snapshot(conn, pid, cid, cache)    ║
║     reputation_snapshot::run_snapshot_batch(context) — the tick      ║
║     config::get_{int,float,bool,text}(...)  w/ fallback cascade       ║
║                                                                      ║
║   New view crate:                                                    ║
║     crates/db_views/reputation (ReputationSummaryView,                ║
║                                 EndorsementSummaryView)              ║
║                                                                      ║
║   Background job:                                                    ║
║     scheduler.every(CTimeUnits::minutes(15)).run(snapshot_batch)     ║
║       short-circuits if BREHON_DISABLE_BACKGROUND_JOBS=1             ║
║       error-logs on failure; next tick retries                       ║
║                                                                      ║
║   New governance-log entry kinds emitted by 5a code:                 ║
║     capability_changed (task 53)                                     ║
║     endorsement_created (task 55)                                    ║
║     — both pseudonymised via actor_pseudonym_helper                  ║
║                                                                      ║
║  Phase 4 golden-path test still passes; 8 existing e2e tests green.  ║
╚══════════════════════════════════════════════════════════════════════╝
```

### Endpoint Changes

| Endpoint | Before 5a | After 5a |
|---|---|---|
| `POST /api/v4/governance/endorsement` | not registered | registered — config-driven gate, emits deltas + log |
| (everything else under `/governance/*`) | unchanged | unchanged (5c adds the remaining 6) |

### Cross-cutting hash-chain additions

| `entry_kind` | Emitter | Pseudonym source | Payload fields |
|---|---|---|---|
| `capability_changed` | `reputation_snapshot::recompute_snapshot` | `get_or_create(person_id)` | `dimension_flipped`, `direction: gained|lost`, `snapshot_community_id` |
| `endorsement_created` | `create_endorsement` | `get_or_create(sponsor_id)` and `get_or_create(target_id)` | `sponsor_pseudonym`, `target_pseudonym`, `community_id`, `gate_strategy` |

Both payloads pass through `scrub_json` automatically via `governance_log::append` (unchanged from Phase 4b — verified at `governance_log.rs:72`).

---

## §7. Mandatory Reading (implementation agent must read before starting)

Use `Read(offset, limit)` with the §7 coordinates — do not read whole files.

### P0 — design docs (homeserver paths; fork copies are stale)

| File | Section | Why |
|---|---|---|
| `homeserver/docs/research/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` | §3 Phase 5a (tasks 50–55), §4 cross-cutting | Authoritative task list + hash-chain + redaction + pseudonym contracts |
| `homeserver/docs/research/brehon-law-inspired-network/04-data-model-and-api.md` | §3 (ReputationEvent, ReputationSnapshot), §4.3 (reputation views), §5 (DTOs), §6.1 (create_endorsement contract), §7 (routes) | Struct + query + route signatures |
| `homeserver/docs/research/brehon-law-inspired-network/99-decisions-and-open-questions.md` | **ADR-005, ADR-013, ADR-015**; resolved **OQ-004, OQ-013, OQ-014, OQ-016, OQ-022, OQ-024**; open **OQ-006** | Hard constraints; every 5a decision ratified by these |
| `homeserver/docs/research/brehon-law-inspired-network/01-vision-and-principles.md` | §5.2 (honour-price floor, sum-vs-severity note) | Justifies `sponsor_liability_floor=0` default |
| `homeserver/docs/research/brehon-law-inspired-network/06-security-and-threat-model.md` | §6.1 (redaction single code path) | Every string into the log passes through `scrub`/`scrub_json` |
| `homeserver/.claude/advisor-context-phase-5.md` | §4 Watches 1, 2, 6, 7, 8, 9, 11; §5 rules 11, 13, 19, 20; §7 catch-fire | Plan-to-impl texture for this sub-phase |

### P1 — fork-local infrastructure rules

| File | Why |
|---|---|
| `CLAUDE.md` (fork root) | Pinned upstream SHA, branch model, command map |
| `.claude/rules/phase-branch.md` | `phase-5a` branching + PR flow (mandatory from Phase 5 on) |
| `.claude/rules/pre-phase-harness-audit.md` | Task 0 checklist — 10–15 min pre-flight |
| `.claude/rules/cargo-output-capture.md` | Every cargo invocation → file → `$?` → tail |
| `.claude/rules/no-cargo-output-paste.md` | Tail 20 lines max into conversation |
| `.claude/rules/decision-queue.md` | How to queue an advisor question without stalling |
| `.claude/rules/gh-pr-fork-target.md` | `--repo barrie-cork/lemmy` on every `gh pr` |
| `.claude/rules/view-crate-selectable-template.md` | Task 52 uses tuple-load + `build_view` (NOT `Selectable`) because `active_sanctions: i64` is kind (c) bare-scalar |

### P0 — fork source files to mirror

| File:lines | Pattern to mirror |
|---|---|
| `crates/db_schema_file/src/schema.rs:1086-1097` | `reputation_event` table — fields to sum in snapshot calc |
| `crates/db_schema_file/src/schema.rs:1100-1112` | `reputation_snapshot` table — target of upsert |
| `crates/db_schema_file/src/schema.rs:370-377` | `endorsement` table — `from_person_id`, `to_person_id`, `community_id`, `revoked_at` |
| `crates/db_schema_file/src/schema.rs:1192-1199` | `surety` table — `sponsor_id`, `sponsored_id`, `revoked_at` |
| `crates/db_schema_file/src/schema.rs:833-860` | `person` table — line to insert `membership_state` column |
| `migrations/2026-04-15-100000-0000_add_governance_enums/up.sql` | Enum-migration pattern (reference for `MembershipState`) |
| `migrations/2026-04-15-100300-0000_add_reputation_and_surety/up.sql` | Reputation + endorsement + surety table creation (reference; do not re-create) |
| `crates/api/api/src/governance/governance_log.rs:60-99` | `append(pool, entry_kind, payload, actor_pseudonym)` — task 53 + 55 call sites |
| `crates/api/api/src/governance/actor_pseudonym_helper.rs:21-60` | `get_or_create(pool, person_id)` — pseudonym generator |
| `crates/api/api/src/governance/submit_jury_vote.rs:240-290` | `governance_log::append(&mut conn.into(), ...)` pattern from within a `run_transaction` (task 53, 55 mirror this) |
| `crates/api/api/src/governance/admin_assign_jury.rs:50-66` | `conn.run_transaction(|conn| async move { ... }.scope_boxed())` — task 55 mirrors this |
| `crates/api/api_crud/src/governance/create_report.rs:50-100` | api_crud handler + imports — task 55 mirrors this file shape directly |
| `crates/db_views/governance_modlog/src/lib.rs` (whole, ≤62 lines) | Phase 2 view-crate `lib.rs` shape — plain struct, `#[cfg(feature = "full")] pub mod impls;` |
| `crates/db_views/governance_modlog/src/impls.rs:1-95` | Tuple-load + `build_view` + two-round-trip pattern — task 52 mirrors this directly |
| `crates/db_views/governance_modlog/Cargo.toml` | Copy verbatim for reputation crate (adjust name) |
| `crates/routes/src/utils/scheduled_tasks.rs:63-152` | `clokwerk::AsyncScheduler` setup + interval registration — task 54 adds one block |
| `crates/api/routes/src/lib.rs:498-513` | `/governance` scope — task 55 adds `.route("/endorsement", post().to(create_endorsement))` |
| `crates/api/api_crud/src/user/create.rs:82,476-485` | `register()` + `PersonInsertForm` struct-update site — task 51 patch site |
| `crates/server/tests/e2e.rs:40-113` | `governance_fixtures::apply_all_schema` — migrations are auto-embedded by `embed_migrations!("../../migrations")`, so new migrations are picked up by every test run without harness changes |

### P1 — completed plan files for shape reference

- `.claude/PRPs/plans/completed/phase-4a-routes-and-first-five-handlers.plan.md` (TOC + tasks + DoD shape)
- `.claude/PRPs/plans/completed/phase-4b-admin-backstops-and-golden-path.plan.md` (TOC + §Flow Design ASCII blocks + GOTCHAs + completion-report cross-link)
- `.claude/PRPs/plans/completed/phase-2b-governance-modlog.plan.md` (view crate shape precedent — reputation crate mirrors)

### External documentation (only where research beyond fork is required)

| Source | Section | Why |
|---|---|---|
| `docs.rs/clokwerk/latest/clokwerk/` | `AsyncScheduler`, `TimeUnits::minutes` | Already used at `scheduled_tasks.rs:4` — no new dep. Verify signature matches task 54's call |
| `docs.rs/diesel-derive-enum/latest` | `DbEnum` macro | Task 51's `MembershipState` enum requires this derive (the existing `CaseStatus` enum at `crates/db_schema_file/src/enums.rs` uses the same pattern — read it before writing the new one) |
| PostgreSQL 15+ `CHECK` constraint + `ADD COLUMN NOT NULL DEFAULT` fast-path semantics | `postgresql.org/docs/15/ddl-alter.html` | Task 50's `CHECK` discriminator + task 51's fast-path column add |

---

## §8. Patterns to Mirror

Every snippet below is a **real** extract from the on-disk workspace at HEAD `26274db05`. Grep-verified per advisor rule 20.

### 8.1 Migration up.sql shape (mirror)

```sql
-- SOURCE: migrations/2026-04-15-100300-0000_add_reputation_and_surety/up.sql
-- COPY THIS PATTERN for migration structure, enum-FK ordering, and indexes.
-- Task 50's up.sql adds a new TABLE with typed-value CHECK, a VIEW, and
-- 32 seeded rows via INSERT ... ON CONFLICT.
CREATE TABLE <name> (
  id SERIAL PRIMARY KEY,
  ...
);
CREATE INDEX <name>_<col>_idx ON <name> (<col>);
```

### 8.2 Diesel enum addition (mirror for task 51's `MembershipState`)

```rust
// SOURCE: crates/db_schema_file/src/enums.rs (read the existing CaseStatus
// block at the top of the file; copy the derive stack verbatim, change
// the name + variants).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(diesel_derive_enum::DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::sql_types::<EnumName>"
)]
#[serde(rename_all = "snake_case")]
pub enum <EnumName> { <Variant>, ... }
```

### 8.3 Plain-struct view with tuple-load + build_view (mirror for task 52)

```rust
// SOURCE: crates/db_views/governance_modlog/src/lib.rs:44-62
// Task 52's ReputationSummaryView mirrors this. Note: NO Selectable/Queryable
// derives — the view contains bare-scalar fields (`active_sanctions: i64`
// derived via second round-trip) per .claude/rules/view-crate-selectable-template.md.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
pub struct <Name>View {
  pub <field>: <type>,
  ...
}
```

```rust
// SOURCE: crates/db_views/governance_modlog/src/impls.rs:1-95
// Task 52's three queries mirror this directly.
async fn <aggregate_helper>(
  conn: &mut diesel_async::AsyncPgConnection,
  ids: &[<Id>],
) -> LemmyResult<HashSet<<Id>>> {
  if ids.is_empty() { return Ok(HashSet::new()); }
  // ...
}

fn build_view(row: <Tuple>, aux: &<Aux>) -> <View> { ... }

pub async fn list_<entity>(pool: &mut DbPool<'_>) -> LemmyResult<Vec<<View>>> {
  let conn = &mut get_conn(pool).await?;
  let rows: Vec<<Tuple>> = <table>::table
    .left_join(<other>::table)
    .order_by(<col>.desc())
    .select((<explicit columns>))
    .load::<<Tuple>>(conn)
    .await?;
  let ids: Vec<<Id>> = rows.iter().map(|r| r.<n>).collect();
  let aux = <aggregate_helper>(conn, &ids).await?;
  Ok(rows.into_iter().map(|r| build_view(r, &aux)).collect())
}
```

### 8.4 Handler inside `run_transaction` (mirror for task 55)

```rust
// SOURCE: crates/api/api/src/governance/admin_assign_jury.rs:42-66
pub async fn <handler>(
  Json(data): Json<<Req>>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<<Res>>> {
  check_local_user_valid(&local_user_view)?;

  let caller_id = local_user_view.person.id;
  let caller_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), caller_id).await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let data_for_tx = data;
  let pseudonym_for_tx = caller_pseudonym.clone();

  let outcome = conn
    .run_transaction(|conn| {
      async move { process_<action>(conn, pseudonym_for_tx, data_for_tx).await }.scope_boxed()
    })
    .await?;

  Ok(Json(outcome))
}
```

### 8.5 Governance-log append from inside a tx (mirror for tasks 53, 55)

```rust
// SOURCE: crates/api/api/src/governance/submit_jury_vote.rs:240-250
// The `&mut conn.into()` adapts an in-tx AsyncPgConnection to the DbPool
// signature append() expects, so the log write rides the caller's tx.
governance_log::append(
  &mut conn.into(),
  "endorsement_created", // use the const from governance_log::ENTRY_KIND_* (task 53)
  json!({
    "sponsor_pseudonym": sponsor_pseudonym,
    "target_pseudonym": target_pseudonym,
    "community_id": data.community_id.map(|c| c.0),
    "gate_strategy": gate_strategy_label,
  }),
  Some(sponsor_pseudonym.clone()),
).await?;
```

### 8.6 Route registration (mirror for task 55's one-line add)

```rust
// SOURCE: crates/api/routes/src/lib.rs:498-513
// Task 55 adds exactly one `.route("/endorsement", ...)` inside the
// existing `/governance` scope. Do NOT reorder the block.
.service(
  scope("/governance")
    .route("/report", post().to(create_report))
    .route("/endorsement", post().to(create_endorsement)) // <<< ADDED in task 55
    .route("/case", get().to(get_case))
    .route("/modlog", get().to(list_modlog))
    .service(scope("/jury").route("/me", get().to(list_my_jury_queue)).route("/vote", post().to(submit_jury_vote)))
    .service(scope("/admin").route("/assign-jury", post().to(admin_assign_jury)).route("/close-case", post().to(admin_close_case))),
),
```

### 8.7 Clokwerk registration (mirror for task 54)

```rust
// SOURCE: crates/routes/src/utils/scheduled_tasks.rs:67-82 (10-minute tick)
// Task 54 adds a sibling block for 15 minutes, short-circuits if the env
// var is set, and calls run_snapshot_batch from reputation_snapshot.rs.
let context_gov = context.clone();
scheduler.every(CTimeUnits::minutes(15)).run(move || {
  let context = context_gov.clone();
  async move {
    if std::env::var("BREHON_DISABLE_BACKGROUND_JOBS").as_deref() == Ok("1") {
      return;
    }
    lemmy_api::governance::reputation_snapshot::run_snapshot_batch(&context)
      .await
      .inspect_err(|e| warn!("Failed to run snapshot batch: {e}"))
      .ok();
  }
});
```

### 8.8 Register-handler patch site (mirror for task 51)

```rust
// SOURCE: crates/api/api_crud/src/user/create.rs:476-485
// Task 51 adds `membership_state: Some(config_default),` via struct-update,
// reading from `config::get_text(&mut conn.into(), Scope::Instance,
// "onboarding.default_membership_state")` upstream of this site.
let person_form = PersonInsertForm {
  ap_id: Some(ap_id.clone()),
  inbox_url: Some(generate_inbox_url()?),
  private_key: Some(actor_keypair.private_key),
  membership_state: Some(parse_membership_state(&config_default)), // <<< ADDED
  ..PersonInsertForm::new(username.clone(), actor_keypair.public_key, site_view.site.instance_id)
};
```

---

## §9. Files to Change (by task)

| Task | File | Action | Why |
|---|---|---|---|
| 50 | `migrations/2026-04-18-000000-0000_add_governance_config/up.sql` | CREATE | Table + CHECK + view + 32 seed rows + `can_sponsor` column + threshold_score micros rescale |
| 50 | `migrations/2026-04-18-000000-0000_add_governance_config/down.sql` | CREATE | Reverse: drop view, drop CHECK, drop column, drop table; restore threshold_score integer units |
| 50 | `crates/db_schema_file/src/schema.rs` | UPDATE | Regenerated by `diesel print-schema` after migration runs — add `governance_config`, modify `reputation_snapshot` (+1 col) |
| 50 | `crates/db_schema/src/source/governance/governance_config.rs` | CREATE | `GovernanceConfig`, `GovernanceConfigInsertForm`; `Queryable` + `Insertable` derives (no Selectable — single-table) |
| 50 | `crates/db_schema/src/source/governance/mod.rs` | UPDATE | `pub mod governance_config; pub use governance_config::*;` |
| 50 | `crates/api/api/src/governance/config.rs` | CREATE | `ConfigCache`, `Scope::{Instance, Community(i32)}`, `get_int/get_float/get_bool/get_text`, `pub const DEFAULT_*` block (32 const defaults), `#[cfg(test)] mod parity` (compile-time seed-vs-const test) |
| 50 | `crates/api/api/src/governance/mod.rs` | UPDATE | `pub mod config;` |
| 50 | `crates/api/api/Cargo.toml` | UPDATE | add `lemmy_db_views_reputation` dep (skipped in task 50 if ordering puts task 52 first — see §12.3) |
| 51 | `migrations/2026-04-18-000100-0000_add_person_membership_state/up.sql` | CREATE | Fast-path column add |
| 51 | `migrations/2026-04-18-000100-0000_add_person_membership_state/down.sql` | CREATE | Drop column, drop enum type |
| 51 | `crates/db_schema_file/src/enums.rs` | UPDATE | Add `MembershipState` enum + `DbEnum` derive block |
| 51 | `crates/db_schema_file/src/schema.rs` | UPDATE | `person` table: add `membership_state -> MembershipState,` — regenerated by diesel |
| 51 | `crates/db_schema/src/source/person.rs` | UPDATE | Add `membership_state: MembershipState` to `Person` + `PersonInsertForm` (upstream — carry-patch marker per `feedback_carry_patch_todos`) |
| 51 | `crates/api/api_crud/src/user/create.rs` | UPDATE | Patch `register()` at line 82 to read config default + pass via `PersonInsertForm` struct-update at line 476 |
| 51 | `scripts/brehon/lint-no-membership-read.sh` | CREATE | Grep-guard — fails if `membership_state` appears outside `crates/db_schema*`, `crates/api/api_crud/src/user/create.rs`, migrations, or enums |
| 51 | `scripts/brehon/lint-no-can-sponsor-read.sh` | CREATE | Sibling guard for `reputation_snapshot.can_sponsor` |
| 52 | `crates/db_views/reputation/Cargo.toml` | CREATE | Copy from `crates/db_views/governance_modlog/Cargo.toml`, rename |
| 52 | `crates/db_views/reputation/src/lib.rs` | CREATE | `ReputationSummaryView`, `EndorsementSummaryView` structs; plain Rust, no Selectable |
| 52 | `crates/db_views/reputation/src/impls.rs` | CREATE | `read_reputation_summary`, `list_endorsements_for_person`, `list_sureties_for_person` |
| 52 | `Cargo.toml` (workspace) | UPDATE | Add member `crates/db_views/reputation` + workspace dep entry `lemmy_db_views_reputation` |
| 53 | `crates/api/api/src/governance/reputation_snapshot.rs` | CREATE | `recompute_snapshot`, `detect_capability_changes`, `run_snapshot_batch`, `CapabilityChange` |
| 53 | `crates/api/api/src/governance/governance_log.rs` | UPDATE | Prepend `pub const ENTRY_KIND_*` block enumerating v0 entry kinds (compile-time typo guard) |
| 53 | `crates/api/api/src/governance/mod.rs` | UPDATE | `pub mod reputation_snapshot;` |
| 53 | `migrations/2026-04-18-000000-0000_add_governance_config/up.sql` | UPDATE (same migration as task 50 — add before committing task 50) | Add partial unique index `reputation_snapshot_person_null_community` and `can_sponsor` column. Avoiding a third migration keeps the ordering simple |
| 54 | `crates/routes/src/utils/scheduled_tasks.rs` | UPDATE | New `scheduler.every(CTimeUnits::minutes(15))` block + `std::env::var("BREHON_DISABLE_BACKGROUND_JOBS")` short-circuit |
| 54 | `crates/routes/Cargo.toml` | UPDATE | Add `lemmy_api` workspace dep (for the `governance::reputation_snapshot::run_snapshot_batch` call) — may already exist; verify |
| 54 | `crates/server/src/governance.rs` | UPDATE | Replace the stub `info!` line with a `tracing::info!` that confirms the scheduled_tasks registration is active (≤5 lines changed; total file stays ≤35 lines per 03 §11) |
| 55 | `crates/api/api_crud/src/governance/create_endorsement.rs` | CREATE | Full handler: gate-strategy dispatch, max-5 + 48h, insert endorsement + conditional surety, two rep events, two recompute_snapshots, log entry |
| 55 | `crates/api/api_crud/src/governance/mod.rs` | UPDATE | `pub mod create_endorsement;` + `pub use create_endorsement::create_endorsement;` |
| 55 | `crates/api/api_common/src/governance.rs` | UPDATE | Add `CreateEndorsementResponse { endorsement_id: EndorsementId, surety_created: bool }` (DTO `CreateEndorsement` at line 193 already exists) |
| 55 | `crates/api/routes/src/lib.rs` | UPDATE | Add `.route("/endorsement", post().to(create_endorsement))` inside the `/governance` scope at line 500 area |
| 55 | `crates/api/routes/Cargo.toml` | Verify only | `lemmy_api_crud` dep already present (Phase 4a) — no edit needed |
| 56 | `.claude/PRPs/reports/phase-5a-complete-report.md` | CREATE | Completion report for the phase-branch PR body |

---

## §10. NOT Building (v0 scope limits)

Deferred explicitly — do NOT drift into these:

- **Sponsor-liability math + `submit_jury_vote` mutation** — Phase 5b task 56.
- **Reputation-gated jury selection + concurrent cap in `admin_assign_jury`** — Phase 5b task 57.
- **OQ-006 threshold formula activation in `create_report`** — Phase 5b task 58. 5a seeds the five config keys (`base_weight`, `clamp_min`, `clamp_max`, `recency_half_life_hours`, `case_threshold_micros`) and rescales existing `threshold_score` rows to micros; 5b removes the constants and wires the formula.
- **Founder-seeding CLI** — Phase 5b task 59. Task 50 seeds the three founder-cap config keys (`max_founders_active`, `max_expires_days`, `max_seed_delta`); the CLI that consumes them is Phase 5b.
- **`get_my_reputation` endpoint and admin backstops like `reputation-stats`** — Phase 5c tasks 61, 62.
- **`jury/accept`, `jury/decline`, `appeal`, `list_cases` handlers + Selected→Accepted flip** — Phase 5c tasks 64, 65, 66, 67 (+ mutation of `admin_assign_jury.rs:107` from `Accepted` to `Selected`, which also requires the golden-path test to be updated).
- **Admin HTTP endpoint for config writes (OQ-018)** — v1. 5a's config is edit-via-psql for admin tuning.
- **`participation_consistency` non-endorsement event sources (OQ-019)** — v1. 5a emits `participation_consistency` only via the `+5` sponsee delta in task 55.
- **v1 sponsor gate strategies `'age_or_surety' | 'reputation' | 'allowlist'` (OQ-020)** — v1.
- **Endorsement revoke endpoint (`POST /endorsement/revoke`)** — deferred per [05 §2].
- **v1 status-conditional config extension pattern (OQ-026)** — v1.
- **Reading `person.membership_state` or `reputation_snapshot.can_sponsor`** from any v0 handler — guarded by the two grep-lint scripts; adding a read requires a new ADR.

---

## §11. Watchpoint coverage matrix

Per advisor context §4. Every watchpoint that lands in 5a scope is addressed in an explicit task step with a GOTCHA; the impl agent must produce a grep/test that demonstrates the mitigation.

| Watch | Concern | Mitigating task | Mitigation artefact |
|---|---|---|---|
| 1 | Seed/const parity for `governance_config` | 50 | `#[cfg(test)] mod parity` in `config.rs` — compile-time assertion (a `const` list and a seeded-key list matched one-for-one) |
| 2 | Snapshot `expires_at` filter (cliff) | 53 | `recompute_snapshot` WHERE `expires_at IS NULL OR expires_at > now()`; unit assertion in doc comment; Task 56 phase-close run of `report_to_modlog_golden_path` confirms no regression |
| 6 | Background job failure mode | 54 | `.inspect_err(|e| warn!(...)).ok()` pattern inside closure; clokwerk scheduler never exits regardless of job outcome |
| 7 | `membership_state` silent read | 51 | `scripts/brehon/lint-no-membership-read.sh` fails CI/phase-close if unauthorised read exists |
| 8 | Decay + `expires_at` double-discount | 53 | `if event.expires_at.is_none() { apply_decay }` branch — a distinct predicate from the query-level filter |
| 9 | Concurrent recompute race | 53 | `SELECT ... FOR UPDATE` on old_snapshot inside the tx; background job acquires `pg_advisory_xact_lock(hash(person_id, community_id))` before calling recompute — serialisation only needed between concurrent writers |
| 11 | Admin governance-weight action modlog at write time | 53 | `capability_changed` log entry emitted on every boolean flip so downstream cascades are attributable; the admin who changes the config root cause (via psql or future OQ-018 endpoint) is covered by the `scripts/brehon/admin-config-write.sh` wrapper documented here (v0 is psql-only; wrapper lands if an admin runs the command before v1 endpoint ships — advisor-side discretion per rule 11) |

Watch 10 (payload PII) is established in 5a by the two new `entry_kind` emitters using `actor_pseudonym_helper::get_or_create` exclusively — no raw `person_id` in any payload. The 5b/5c e2e test grep assertion from Watch 10 is on Phase 5b's task 60 scope, not 5a.

Watch 3, 4, 5 are 5b/5c concerns (Watch 3 sponsor-liability math — 5b; Watch 4 golden-path regression — addressed in 5a task 56 phase-close as a regression gate; Watch 5 branch-and-PR workflow — addressed in task 0 and task 56).

---

## §12. Step-by-Step Tasks

One commit per task. Validate after each. Commit messages follow Phase 4 convention: `feat(scope): task N — <one-line summary>` from task 50 onwards.

### §12.0 Task 0 — Pre-phase harness audit + branch creation

**Goal.** Catch wrapper-script drift and pre-existing DoD-command breakage BEFORE any code lands. Cut the `phase-5a` feature branch. This is the 10–15-minute cost that prevents Phase 2a-style checkpoint-cascades.

**Steps (all in `cmd //c` with output capture per `.claude/rules/cargo-output-capture.md`; tail ≤20 lines per `.claude/rules/no-cargo-output-paste.md`):**

1. **Branch verification and creation.**
   ```bash
   git branch --show-current   # expect: governance-v0
   git status --short          # expect: clean (the advisor has already committed .claude/decision-queue.json)
   git log -1 --format=%H      # expect: 26274db05...
   git checkout -b phase-5a    # MUST run before any commit in this phase per .claude/rules/phase-branch.md
   ```

2. **Wrapper flag audit (Probes 1–3 per `.claude/rules/pre-phase-harness-audit.md`).**
   ```bash
   cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/audit-cargo-check-p.log 2>&1"
   status=$?; tail -15 .claude/audit-cargo-check-p.log; echo "exit: $status"
   # Expect: ONLY lemmy_api compiles. If you see Checking lemmy_db_schema or other crates,
   # the wrapper is discarding -p. STOP and fix the wrapper before task 50.

   cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/audit-cargo-check-features.log 2>&1"
   status=$?; tail -15 .claude/audit-cargo-check-features.log; echo "exit: $status"
   # Expect: compiles with --features full in the cargo invocation line.

   cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/audit-cargo-test.log 2>&1"
   status=$?; tail -15 .claude/audit-cargo-test.log; echo "exit: $status"
   # Expect: only e2e test target compiles.
   ```

3. **DoD smoke test — run every §14 command against pre-5a HEAD, capture result.**
   ```bash
   cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/audit-5a-dod-check.log 2>&1"
   status=$?; tail -15 .claude/audit-5a-dod-check.log; echo "exit: $status"
   # Expected: exit 0 (baseline green). PLAN-DRIFT NOTE: IMPLEMENTATION-PLAN-v0.md §3 Phase 5a DoD says
   # `--no-deps`; that flag is NOT valid on `cargo check` in this cargo version (verified at plan-write
   # time — log shows "error: unexpected argument '--no-deps'"). The plan's DoD therefore uses
   # `cargo check --features full --workspace` without --no-deps. Clippy keeps --no-deps (see below).

   cmd //c "scripts\\brehon\\cargo-clippy.bat --features full --workspace --no-deps -- -D warnings > .claude/audit-5a-dod-clippy.log 2>&1"
   status=$?; tail -30 .claude/audit-5a-dod-clippy.log; echo "exit: $status"
   # Expected at plan-write time: exit 2 — "this lint expectation is unfulfilled" at
   # crates/diesel_utils/src/pagination.rs:220:10 on `#[expect(clippy::multiple_bound_locations)]`
   # under `-D unfulfilled-lint-expectations` (implied by `-D warnings`). CONFIRMED in plan's §16
   # Risks table row 1. Before task 50: land a carry-patch commit that replaces the attribute at
   # pagination.rs:220 with `#[allow(clippy::multiple_bound_locations)]` (or deletes it if the
   # underlying lint no longer fires), with a `TODO(brehon-fork): upstream this to
   # LemmyNet/lemmy — PR #___` marker per `feedback_carry_patch_todos.md`. Commit message:
   # `chore(lint): clear unfulfilled lint expectation in pagination.rs pre-5a (carry-patch)`.
   # Re-run the clippy dry-run afterwards; must now exit 0. If it still fails on a different
   # lint, surface via decision-queue to the advisor.

   cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --features full --no-run -p lemmy_server > .claude/audit-5a-dod-e2e-compile.log 2>&1"
   status=$?; tail -15 .claude/audit-5a-dod-e2e-compile.log; echo "exit: $status"
   # Expected: exit 0 — e2e target compiles on pre-5a HEAD.
   ```

4. **Land the pre-5a carry-patch commit — MANDATORY before task 50.** Confirmed by user direction 2026-04-17 (Option B). Edit `crates/diesel_utils/src/pagination.rs:220`:

   ```rust
   // BEFORE (fails clippy under -D warnings):
   #[expect(clippy::multiple_bound_locations)]

   // AFTER:
   // TODO(brehon-fork): upstream this to LemmyNet/lemmy — PR #___
   // The `#[expect(...)]` form requires the lint to actually fire on the
   // annotated item; on the cargo/clippy version pinned by rust-toolchain.toml
   // it no longer fires, which `-D unfulfilled-lint-expectations` (implied
   // by `-D warnings`) promotes to an error. `#[allow(...)]` preserves the
   // original intent (the bound pattern may resurface on a future clippy
   // upgrade) without the expectation-unfulfilled failure.
   #[allow(clippy::multiple_bound_locations)]
   ```

   **Carry-patch inventory.** No `scripts/brehon/CARRY-PATCHES.md` inventory file exists in the fork at plan-write time (verified: the only `*patch*` file is `crates/db_schema_file/diesel_ltree.patch`, a codegen patch unrelated to carry-forward). Per `feedback_carry_patch_todos.md` convention: the commit message contains the full context; the TODO comment above the attribute holds the in-code marker; and the completion report §17.3 names this as a carry-patch that needs an inventory entry on the homeserver side (advisor task to update `homeserver/.claude/memory/feedback_carry_patch_todos.md` — not fork-side work).

   Commit:
   ```bash
   git add crates/diesel_utils/src/pagination.rs
   git commit -m "$(cat <<'EOF'
   chore(lint): clear unfulfilled lint expectation in pagination.rs pre-5a (carry-patch)

   The pinned cargo/clippy version no longer triggers
   `clippy::multiple_bound_locations` on the annotated item in
   `crates/diesel_utils/src/pagination.rs:220`. Under `-D warnings` this
   promotes `unfulfilled-lint-expectations` to an error, blocking the
   Phase 5a §14 Level 1 clippy DoD.

   Replace `#[expect(clippy::multiple_bound_locations)]` with
   `#[allow(clippy::multiple_bound_locations)]` to preserve the original
   intent (the bound pattern may return on a future clippy upgrade)
   without the expectation-unfulfilled failure.

   Marked with `TODO(brehon-fork): upstream this to LemmyNet/lemmy — PR #___`
   per `feedback_carry_patch_todos.md`. Inventory file
   `scripts/brehon/CARRY-PATCHES.md` does not exist in this fork at
   plan-write time; surface to advisor for homeserver-side
   `feedback_carry_patch_todos.md` update (Phase 5a completion report §17.3).
   EOF
   )"
   ```

   Re-run the clippy dry-run after the commit; must now exit 0.

5. **Decision-queue review.** Read `.claude/decision-queue.json`. Questions #11 (juror cap=3) and #12 (sponsor-liability units) are pre-seeded for Phase 5b; both are resolved in OQ-004 and the plan text for 5b task 56. They do **not** block 5a. If either is still `"answer": null` at 5a close, **do not** resolve them in 5a — leave for 5b.

**DoD.**
- [ ] Current branch is `phase-5a`.
- [ ] All three wrapper probes pass; wrapper does not silently drop flags.
- [ ] All three DoD dry-runs exit 0 — including clippy, which requires step 4's carry-patch to have landed first.
- [ ] Carry-patch commit `chore(lint): clear unfulfilled lint expectation in pagination.rs pre-5a (carry-patch)` visible in `git log --oneline phase-5a`.
- [ ] `.claude/decision-queue.json` reviewed; any pending entries flagged in the commit message.
- [ ] **If any audit step fails, STOP.** Do not start task 50 on a broken baseline. Surface to advisor via a fresh decision-queue entry.

**Commit.** Exactly **one** commit from this task: the pagination.rs carry-patch at step 4. The audit logs and branch are produced but not committed (logs live in `.claude/` which is typically `.gitignore`d; the branch itself is the artefact).

---

### §12.1 Task 50 — `governance_config` migration + reader (with valid_from history, CHECK discriminator, `can_sponsor` + partial-unique-index on `reputation_snapshot`, threshold_score micros rescale, compile-time seed-vs-const parity)

**Goal.** Ship the single source of tuneable values for the whole governance platform, plus the snapshot-table prerequisites task 53 needs. Every hardcoded constant from Phase 4 that migrates to config in 5b must have a seed row in this task.

**Steps.**

1. **Create migration directory** `migrations/2026-04-18-000000-0000_add_governance_config/` with `up.sql` and `down.sql`.

2. **`up.sql` — table + CHECK + view + seed + snapshot schema changes + micros rescale.**

   Sketch (agent produces executable SQL):

   ```sql
   -- 1. governance_config table with typed-value discriminator.
   CREATE TABLE governance_config (
     id              SERIAL PRIMARY KEY,
     scope           TEXT NOT NULL,            -- 'instance' or 'community:<id>'
     key             TEXT NOT NULL,            -- dotted namespace, e.g. 'onboarding.sponsor_min_account_age_days'
     value_type      TEXT NOT NULL,            -- 'int' | 'float' | 'bool' | 'text'
     value_int       BIGINT,
     value_float     DOUBLE PRECISION,
     value_bool      BOOLEAN,
     value_text      TEXT,
     valid_from      TIMESTAMPTZ NOT NULL DEFAULT now(),
     updated_by      INTEGER REFERENCES person(id) ON DELETE SET NULL,
     CONSTRAINT governance_config_typed CHECK (
       (value_type = 'int'   AND value_int   IS NOT NULL AND value_float IS NULL AND value_bool IS NULL AND value_text IS NULL) OR
       (value_type = 'float' AND value_float IS NOT NULL AND value_int   IS NULL AND value_bool IS NULL AND value_text IS NULL) OR
       (value_type = 'bool'  AND value_bool  IS NOT NULL AND value_int   IS NULL AND value_float IS NULL AND value_text IS NULL) OR
       (value_type = 'text'  AND value_text  IS NOT NULL AND value_int   IS NULL AND value_float IS NULL AND value_bool IS NULL)
     )
   );

   -- Unique on (scope, key, valid_from) lets admin edits insert a new row
   -- and keep history; the `_current` view reads the latest per (scope, key).
   CREATE UNIQUE INDEX governance_config_scope_key_valid_from_idx
     ON governance_config (scope, key, valid_from);
   CREATE INDEX governance_config_scope_key_idx
     ON governance_config (scope, key);

   -- 2. governance_config_current view — most-recent row per (scope, key).
   CREATE VIEW governance_config_current AS
   SELECT DISTINCT ON (scope, key)
     id, scope, key, value_type, value_int, value_float, value_bool, value_text, valid_from, updated_by
   FROM governance_config
   ORDER BY scope, key, valid_from DESC;

   -- 3. reputation_snapshot.can_sponsor column (task 53 prerequisite).
   ALTER TABLE reputation_snapshot ADD COLUMN can_sponsor BOOLEAN NOT NULL DEFAULT false;
   COMMENT ON COLUMN reputation_snapshot.can_sponsor IS
     'Computed by Phase 5a task 53 but NOT read by any v0 handler. See [99 OQ-014]; v1 flips config.sponsorship.require_reputation_gate = true to activate enforcement.';

   -- 4. reputation_snapshot partial unique index — Postgres treats NULL as
   --    distinct in unique constraints, so instance-scoped snapshots
   --    (community_id IS NULL) would not dedupe without this index.
   CREATE UNIQUE INDEX reputation_snapshot_person_null_community
     ON reputation_snapshot (person_id) WHERE community_id IS NULL;

   -- 5. threshold_score micros rescale — Phase 4 stored integer units; task 58
   --    expects micros (×1_000_000). Multiply existing rows so the golden-path
   --    test's admin-forced ThresholdMet pattern remains consistent.
   UPDATE moderation_case SET threshold_score = threshold_score * 1000000;

   -- 6. Seed 32 instance-scoped config rows via ON CONFLICT DO NOTHING.
   --    All 32 keys match the pub const DEFAULT_* block in config.rs exactly —
   --    the seed-and-const parity test in task 50's Rust code enforces this.
   INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES
     ('instance', 'thresholds.jury_reliability',       'int',   50,   NULL, NULL, NULL),
     ('instance', 'thresholds.reporting_accuracy',     'int',   50,   NULL, NULL, NULL),
     ('instance', 'thresholds.endorsement_strength',   'int',   25,   NULL, NULL, NULL),
     ('instance', 'jury.panel_size',                   'int',   5,    NULL, NULL, NULL),
     ('instance', 'jury.quorum',                       'int',   3,    NULL, NULL, NULL),
     ('instance', 'jury.age_requirement_days',         'int',   60,   NULL, NULL, NULL),
     ('instance', 'jury.max_concurrent_assignments',   'int',   3,    NULL, NULL, NULL),
     ('instance', 'jury.fallback_on_small_pool',       'bool',  NULL, NULL, true, NULL),
     ('instance', 'deltas.juror_aligned',              'int',   10,   NULL, NULL, NULL),
     ('instance', 'deltas.juror_outlier',              'int',   -5,   NULL, NULL, NULL),
     ('instance', 'deltas.reporter_upheld',            'int',   10,   NULL, NULL, NULL),
     ('instance', 'deltas.reporter_dismissed',         'int',   -5,   NULL, NULL, NULL),
     ('instance', 'deltas.endorsement_created_sponsor','int',   5,    NULL, NULL, NULL),
     ('instance', 'deltas.endorsement_created_sponsee','int',   5,    NULL, NULL, NULL),
     ('instance', 'deltas.sponsor_liability_minor',    'int',   -10,  NULL, NULL, NULL),
     ('instance', 'deltas.sponsor_liability_moderate', 'int',   -50,  NULL, NULL, NULL),
     ('instance', 'deltas.sponsor_liability_severe',   'int',   -200, NULL, NULL, NULL),
     ('instance', 'liability.founder_multiplier',      'float', NULL, 2.0,  NULL, NULL),
     ('instance', 'liability.regular_multiplier',      'float', NULL, 1.0,  NULL, NULL),
     ('instance', 'liability.sponsor_liability_floor', 'int',   0,    NULL, NULL, NULL),
     ('instance', 'report.base_weight',                'float', NULL, 1.0,  NULL, NULL),
     ('instance', 'report.clamp_min',                  'float', NULL, 0.1,  NULL, NULL),
     ('instance', 'report.clamp_max',                  'float', NULL, 2.0,  NULL, NULL),
     ('instance', 'report.recency_half_life_hours',    'float', NULL, 168.0,NULL, NULL),
     ('instance', 'report.case_threshold_micros',      'int',   3000000, NULL, NULL, NULL),
     ('instance', 'decay.positive_half_life_days',     'int',   90,   NULL, NULL, NULL),
     ('instance', 'onboarding.default_membership_state','text', NULL, NULL, NULL, 'member'),
     ('instance', 'onboarding.sponsor_gate_strategy',  'text',  NULL, NULL, NULL, 'age'),
     ('instance', 'onboarding.sponsor_min_account_age_days','int', 30, NULL, NULL, NULL),
     ('instance', 'founder.max_founders_active',       'int',   20,   NULL, NULL, NULL),
     ('instance', 'founder.max_expires_days',          'int',   365,  NULL, NULL, NULL),
     ('instance', 'founder.max_seed_delta',            'int',   200,  NULL, NULL, NULL),
     ('instance', 'job.snapshot_interval_seconds',     'int',   900,  NULL, NULL, NULL),
     ('instance', 'job.snapshot_batch_chunk_size',     'int',   500,  NULL, NULL, NULL)
   ON CONFLICT (scope, key, valid_from) DO NOTHING;
   -- 34 rows (counted; grew from 33 to 34 after Perplexity-review 2026-04-17
   -- added `job.snapshot_batch_chunk_size` so the snapshot batch chunking
   -- knob in task 54 is tuneable at pilot time without a code change). Adjust
   -- `const EXPECTED_SEED_COUNT` in config.rs to match.
   ```

3. **`down.sql` — reverse.** Drop view, drop partial unique index, drop `can_sponsor` column, drop CHECK + indexes, drop table. **Rescale `threshold_score` rows back**: `UPDATE moderation_case SET threshold_score = threshold_score / 1000000`. Order: reverse of up.sql strictly — DROP VIEW first, then ALTER TABLE DROP COLUMN, then DROP TABLE.

4. **`crates/db_schema/src/source/governance/governance_config.rs` — Diesel source struct.**
   Mirror `crates/db_schema/src/source/governance/moderation_case.rs` shape. Derives: `Queryable`, `Selectable`, `Identifiable`, `Insertable` (on `GovernanceConfigInsertForm`). No Phase 2 complications — every field is a straight column. `Associations` to `person` via `updated_by` is optional in 5a; skip if Diesel complains on the nullable FK.

5. **`crates/db_schema/src/source/governance/mod.rs`** — add `pub mod governance_config; pub use governance_config::*;`.

6. **`crates/api/api/src/governance/config.rs` — the reader.**

   Shape:

   ```rust
   //! Config reader for governance_config. Fallback cascade:
   //! 1. community:<id> row (most specific)
   //! 2. instance row
   //! 3. Rust const default (DEFAULT_* below — Watch 1 parity contract)

   pub enum Scope { Instance, Community(CommunityId) }

   pub struct ConfigCache { /* HashMap<(ScopeRepr, String), Value>, per request */ }
   impl ConfigCache { pub fn new() -> Self; }

   pub async fn get_int(cache: &mut ConfigCache, pool: &mut DbPool<'_>, scope: Scope, key: &str) -> LemmyResult<i64>;
   pub async fn get_float(...) -> LemmyResult<f64>;
   pub async fn get_bool(...) -> LemmyResult<bool>;
   pub async fn get_text(...) -> LemmyResult<String>;

   // Const defaults — one per seeded row. Name scheme: scope removed, key snake-cased,
   // dotted → _ (e.g. `thresholds.jury_reliability` → DEFAULT_THRESHOLDS_JURY_RELIABILITY).
   pub const DEFAULT_THRESHOLDS_JURY_RELIABILITY: i64 = 50;
   pub const DEFAULT_THRESHOLDS_REPORTING_ACCURACY: i64 = 50;
   // ... 31 more, including:
   pub const DEFAULT_JOB_SNAPSHOT_INTERVAL_SECONDS: i64 = 900;
   pub const DEFAULT_JOB_SNAPSHOT_BATCH_CHUNK_SIZE: i64 = 500;

   // Compile-time + runtime parity between seed list and const list (Watch 1
   // + Perplexity-review 2026-04-17 strengthening — item (5)).
   //
   // Two tests together form the parity contract:
   //
   //   1. `seeded_keys_count_matches_const_count` — structural assertion that
   //      no new seed row lands without also declaring a Rust const. Runs
   //      without a DB.
   //
   //   2. `seeded_keys_round_trip_via_config_cache` — runtime assertion that
   //      every seeded key can be READ with the typed accessor matching its
   //      declared `value_type`. Catches the class of bug where the SQL
   //      stores a `value_text` row but the Rust const declares `f64`, or
   //      where the impl agent mis-types a const. The DB-level CHECK
   //      discriminator prevents inconsistent storage; this test ensures the
   //      Rust-side declaration matches the storage shape.
   #[cfg(test)]
   mod parity {
     use super::*;
     /// Every seeded key in up.sql must appear here, mapped to the matching
     /// const name AND the value_type used in the seed. Task 0 of 5b will
     /// re-validate after the 5b upsum of sponsor-liability keys (none added
     /// to the seed in 5b; 5b reads what 5a seeded — but 5b adds Rust consts
     /// in the same `config.rs` module, so this list stays authoritative).
     const SEEDED_KEYS_WITH_CONSTS: &[(&str, &str, &str)] = &[
       ("thresholds.jury_reliability", "DEFAULT_THRESHOLDS_JURY_RELIABILITY", "int"),
       // ... 33 more rows, one per seed. Each tuple element carries:
       //   .0 = key name (matches up.sql exactly)
       //   .1 = Rust const name (matches `pub const DEFAULT_*` declaration)
       //   .2 = value_type ("int" | "float" | "bool" | "text")
     ];

     #[test]
     fn seeded_keys_count_matches_const_count() {
       const EXPECTED: usize = 34; // bumped from 33 after Perplexity-review 2026-04-17 added job.snapshot_batch_chunk_size
       assert_eq!(SEEDED_KEYS_WITH_CONSTS.len(), EXPECTED);
     }

     /// Round-trip every seeded key through the typed accessor that matches
     /// its declared `value_type`. Asserts the accessor succeeds and returns
     /// a value of the expected Rust type. Runs against a test DB with the
     /// governance_config migrations applied (testcontainers, same shape as
     /// other e2e tests — see `crates/server/tests/e2e.rs::governance_fixtures`).
     #[tokio::test]
     async fn seeded_keys_round_trip_via_config_cache() -> Result<(), Box<dyn std::error::Error>> {
       // Harness — see governance_fixtures for the shared helper. If this
       // test moves up into crates/server/tests/e2e.rs as an integration
       // test, the fixture reuses directly; if it stays inside lemmy_api
       // crate's `#[cfg(test)]` block, the agent needs a miniature fixture
       // (start pgautoupgrade:18-alpine + embed_migrations + establish conn).
       let (pool, _container) = governance_fixtures::start_and_migrate().await?;
       let mut cache = ConfigCache::new();

       for (key, _const_name, vtype) in SEEDED_KEYS_WITH_CONSTS {
         match *vtype {
           "int"   => { let _: i64    = get_int(&mut cache, &mut pool.clone(), Scope::Instance, key).await?; }
           "float" => { let _: f64    = get_float(&mut cache, &mut pool.clone(), Scope::Instance, key).await?; }
           "bool"  => { let _: bool   = get_bool(&mut cache, &mut pool.clone(), Scope::Instance, key).await?; }
           "text"  => { let _: String = get_text(&mut cache, &mut pool.clone(), Scope::Instance, key).await?; }
           other   => panic!("unknown value_type {other} for key {key}"),
         }
       }
       Ok(())
     }
   }
   ```

   **Parity-test rationale (Perplexity-review 2026-04-17 item 5).** The
   original structural-only parity check passes if a `value_int` row is
   paired with a Rust const declared as `f64` — both exist, count matches,
   test is green, but `get_float(..., key)` fails at handler runtime because
   it looks for a `value_float` row and gets `None`, falling back to the
   const (correct value but wrong code path) or to an error (wrong const
   type). The round-trip test catches this class: each seed → each typed
   accessor → each must succeed. The DB-level CHECK constraint on
   `value_type` already blocks storing inconsistent rows; this test closes
   the Rust-side declaration-vs-storage gap.

**GOTCHAs (task 50).**

- **GOTCHA-50a.** The seed `INSERT ... ON CONFLICT (scope, key, valid_from) DO NOTHING` idempotency depends on the unique index on `(scope, key, valid_from)`. Admin edits that `INSERT` a new row with `valid_from = now()` create history rows without violating the constraint. Do NOT use `(scope, key)` as the conflict target — that would block admin edits after first seed.
- **GOTCHA-50b.** The `threshold_score` micros rescale runs **unconditionally** in up.sql. If a developer runs `diesel migration redo`, the down + up sequence multiplies back: down divides, up multiplies — idempotent. Test this round-trip in §14 Level 4.
- **GOTCHA-50c.** The `can_sponsor` column addition is an intentional override of [04 §13.2]'s "two booleans" shortcut. This is logged as a design decision in the completion report. The `lint-no-can-sponsor-read.sh` guard (task 51) enforces that no v0 handler reads the column.
- **GOTCHA-50d.** The Postgres `CHECK` discriminator is non-trivial to evolve in v1 (e.g. adding `'json'` value_type). Document the evolution path in the migration's `up.sql` header comment: add a new value_type + a new column + a new CHECK in a fresh migration, not by modifying this one.
- **GOTCHA-50e.** The compile-time parity test is a `#[cfg(test)]` module — it runs under `cargo test` but not `cargo check`. The 5a DoD therefore includes `cargo test --test e2e --features full` (which exercises the module transitively if it reaches config reader in any path) AND a dedicated `cargo test -p lemmy_api governance::config::parity --features full`. Add the latter to §14 Level 2.
- **GOTCHA-50f.** The seeded row count is **34**, not 32 as the plan narrative said — recount: 3 threshold + 5 jury + 9 deltas + 3 liability + 5 report + 1 decay + 3 onboarding + 3 founder + 2 job = 34. The advisor-context-phase-5.md §1 "32" was off by one (at +33 after the pre-Perplexity walkthrough); Perplexity review 2026-04-17 added a 34th key `job.snapshot_batch_chunk_size = 500` so task 54's chunked batch size is tuneable at pilot time without a code change. `IMPLEMENTATION-PLAN-v0.md §Phase 5a task 50` pre-dates the chunk-size addition — advisor will update that doc alongside any future narrative edits. Document the 34-vs-32 delta in the completion report.
- **GOTCHA-50g.** Every `governance_config` key name in `config.rs` MUST match the seed SQL exactly character-for-character; typos would produce silent fallback to the const (and no parity-test failure because the structural parity test is structure-only). Run a sanity grep: `grep -oE "'(thresholds|jury|deltas|liability|report|decay|onboarding|founder|job)\.[a-z_]+'" migrations/2026-04-18-000000-0000_add_governance_config/up.sql | sort -u` and compare against the `SEEDED_KEYS_WITH_CONSTS` list — line counts must match. The round-trip parity test (GOTCHA-50h) catches this class too via the `get_<type>()` call failing, but the sanity grep is faster feedback at implementation time.
- **GOTCHA-50h. Round-trip parity test requires a DB (Perplexity-review 2026-04-17 item 5).** The new `seeded_keys_round_trip_via_config_cache` test is `#[tokio::test]`, not `#[test]`, and starts a pgautoupgrade:18-alpine container per run. It cannot live in a pure `#[cfg(test)] mod parity` inside `lemmy_api` without reusable fixtures — the existing `lemmy_api` crate has no testcontainers dependency. Two acceptable landing spots:
  - **Landing spot A (preferred)**: the round-trip test lives in `crates/server/tests/e2e.rs` alongside the other e2e tests, calling into `lemmy_api::governance::config::{get_int, get_float, get_bool, get_text, ConfigCache, Scope, SEEDED_KEYS_WITH_CONSTS}`. The `SEEDED_KEYS_WITH_CONSTS` list must be `pub` (not `pub(super)`) so the e2e crate can read it. This is the cleaner split; the `lemmy_api` crate stays without a testcontainers dependency.
  - **Landing spot B (fallback)**: the round-trip test stays in `lemmy_api::governance::config::parity` but gated behind a new `[dev-dependencies]` entry in `crates/api/api/Cargo.toml` for `testcontainers` + embed_migrations helper. Heavier; adds a test-only dep to `lemmy_api` that no other test there uses.
  - **Decision**: prefer A. The agent should move the round-trip test body into a new top-level test function in `crates/server/tests/e2e.rs::config_parity_round_trip` and leave the structural `seeded_keys_count_matches_const_count` test inside `lemmy_api::governance::config::parity` (no DB needed, runs in-crate).
- **GOTCHA-50i.** The `SEEDED_KEYS_WITH_CONSTS` list is now **3-tuple** `(key, const_name, value_type)`, not 2-tuple. The `const_name` field is not consumed at runtime (the typed accessor takes the key string, not the const); it's there for the structural assertion that a matching Rust const exists on a future convert-to-reflection refactor. Keep the field — removing it would regress review-time readability.

**Validation (task 50).**

```bash
# Build the schema + source crates.
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/build-task50-check-schema.log 2>&1"
status=$?; tail -15 .claude/build-task50-check-schema.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

# Build the reader.
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task50-check-api.log 2>&1"
status=$?; tail -15 .claude/build-task50-check-api.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

# Run the structural parity test (no DB; in-crate unit test).
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_api --features full -- governance::config::parity > .claude/build-task50-parity.log 2>&1"
status=$?; tail -15 .claude/build-task50-parity.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

# Run the round-trip parity test (DB-backed; lives in crates/server/tests/e2e.rs
# per GOTCHA-50h landing spot A).
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e --features full -- config_parity_round_trip > .claude/build-task50-roundtrip.log 2>&1"
status=$?; tail -15 .claude/build-task50-roundtrip.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

# Migration round-trip (DB layer only — tests/e2e.rs doesn't need to run yet).
# Runs via lemmy_diesel_utils::schema_setup, not raw diesel CLI (forbid_diesel_cli trigger).
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e --features full -- can_insert_moderation_case > .claude/build-task50-migration-smoke.log 2>&1"
status=$?; tail -15 .claude/build-task50-migration-smoke.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
# The existing can_insert_moderation_case test runs every migration in migrations/ via embed_migrations!.
# It's the canonical migration round-trip test for this project.
```

**Commit.** `feat(governance): task 50 — add governance_config table + reader with valid_from history + CHECK discriminator + seed-vs-const parity`

---

### §12.2 Task 51 — `membership_state` column + enum (deferred enforcement, grep-guard CI)

**Goal.** Ship the column so v1 can flip config to enforce without a schema migration (OQ-016). Lock down that no v0 handler can silently read it (Watch 7).

**Steps.**

1. **`crates/db_schema_file/src/enums.rs` — add `MembershipState` enum.** Read the existing `CaseStatus` definition at the top of the file; copy the derive stack; the variants are `Member | Provisional | Suspended`.

2. **Migration `migrations/2026-04-18-000100-0000_add_person_membership_state/up.sql` — Postgres 11+ fast-path column add.**

   ```sql
   CREATE TYPE membership_state AS ENUM ('member', 'provisional', 'suspended');
   ALTER TABLE person ADD COLUMN membership_state membership_state NOT NULL DEFAULT 'member';
   -- Postgres 11+ treats this as metadata-only (no table rewrite) because
   -- NOT NULL with a non-volatile DEFAULT is handled via the pg_attribute
   -- attmissingval mechanism. Verify with EXPLAIN (BUFFERS, ANALYZE) on a
   -- large person table before running in production.
   ```

   `down.sql`: `ALTER TABLE person DROP COLUMN membership_state; DROP TYPE membership_state;` — reverse order.

3. **`crates/db_schema_file/src/schema.rs` — regenerated.** After the migration applies, `diesel print-schema` emits the new column; commit the regenerated file.

4. **`crates/db_schema/src/source/person.rs` — add field to Person struct and to `PersonInsertForm`.** Note the `TODO(brehon-fork): upstream this to LemmyNet/lemmy — PR #___` marker above the field (per `feedback_carry_patch_todos`). `PersonInsertForm::new(...)` can keep its existing signature — `membership_state` is `Option<MembershipState>` with `None` → SQL default `'member'`.

5. **`crates/api/api_crud/src/user/create.rs` — patch `register()`.**

   Read `config::get_text(&mut pool, Scope::Instance, "onboarding.default_membership_state")` near the top of the function (after `check_local_user_valid` at line 78 region). Parse to the enum via a dedicated `fn parse_membership_state(s: &str) -> MembershipState` helper (exhaustive match on `"member" | "provisional" | "suspended"`, with unknown → `Member` + `warn!` log). Add the field to the `PersonInsertForm` struct-update at line 476 as shown in §8.8. **Do not** read the column downstream anywhere in this file — the check+write pattern is one-shot at registration.

6. **Grep guards — `scripts/brehon/lint-no-membership-read.sh` + `lint-no-can-sponsor-read.sh`.**

   ```bash
   #!/usr/bin/env bash
   # Deny reads of person.membership_state outside the authorised sites.
   set -euo pipefail
   FORBIDDEN=$(grep -rn --include='*.rs' 'membership_state' crates/ \
     | grep -v 'crates/db_schema_file/' \
     | grep -v 'crates/db_schema/src/source/person.rs' \
     | grep -v 'crates/api/api_crud/src/user/create.rs' \
     || true)
   if [ -n "$FORBIDDEN" ]; then
     echo "ERROR: unauthorised membership_state read(s) detected:"
     echo "$FORBIDDEN"
     exit 1
   fi
   ```

   Sibling `lint-no-can-sponsor-read.sh` allows `reputation_snapshot.rs` (task 53 writes the column) but no other crate.

   Both scripts run as part of the 5a phase-close §14 Level 5.

**GOTCHAs (task 51).**

- **GOTCHA-51a.** The fast-path column add relies on a *non-volatile* DEFAULT. `'member'::membership_state` is a literal — non-volatile. If task 51 ever needs a `now()` default (it does not, but future readers might consider), the column add switches to a full rewrite at write-time; avoid by adding the column nullable, updating, then `SET NOT NULL`.
- **GOTCHA-51b.** The register patch must read config **before** opening the `run_transaction` (if any — `register()` currently does not use one). Doing the config read inside a tx blocks the tx for the read's duration; reading upstream keeps the tx tight.
- **GOTCHA-51c.** The `parse_membership_state` helper must be in a shared utility module (ideally `crates/api/api/src/governance/config.rs` next to the text reader) so tasks 52/53/55 can use it if they ever need to — but 5a downstream code does NOT read the column, so the helper stays private to the register path until v1.
- **GOTCHA-51d.** The grep-guard scripts run in a **bash subshell under Windows** via `cmd //c "bash scripts/brehon/lint-no-membership-read.sh"` or simply `bash scripts/brehon/lint-no-membership-read.sh` from Git Bash. Verify the `#!/usr/bin/env bash` shebang works in the Windows environment; if not, convert to `scripts/brehon/lint-no-membership-read.bat` with `findstr`.
- **GOTCHA-51e.** The guards must **exclude this plan file itself** (and the phase-close report) from the grep — otherwise the plan's mention of `membership_state` fails the guard. Add `| grep -v '.claude/PRPs/plans/' | grep -v '.claude/PRPs/reports/'` to the exclusion chain.

**Validation (task 51).**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/build-task51-check.log 2>&1"
status=$?; tail -15 .claude/build-task51-check.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

# Run the two guard scripts.
bash scripts/brehon/lint-no-membership-read.sh; echo "membership guard exit: $?"
bash scripts/brehon/lint-no-can-sponsor-read.sh; echo "can_sponsor guard exit: $?"

# Existing e2e tests still compile + pass (embedded migrations include the new one).
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e --features full > .claude/build-task51-e2e.log 2>&1"
status=$?; tail -15 .claude/build-task51-e2e.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
```

**Commit.** `feat(governance): task 51 — add person.membership_state column + enum + grep-guards (deferred enforcement per OQ-016)`

---

### §12.3 Task 52 — `crates/db_views/reputation` crate (plain-struct views + tuple-load)

**Goal.** Ship the Phase 2 view crate that task 53 and Phase 5c's `get_my_reputation` handler depend on.

**Steps.**

1. **Create the crate directory** `crates/db_views/reputation/` with:
   - `Cargo.toml` — copy verbatim from `crates/db_views/governance_modlog/Cargo.toml`, rename `name` to `lemmy_db_views_reputation`.
   - `src/lib.rs` — two structs + `pub mod impls;` behind `#[cfg(feature = "full")]`.
   - `src/impls.rs` — three queries.

2. **`lib.rs` shape.**

   ```rust
   use chrono::{DateTime, Utc};
   use serde::{Deserialize, Serialize};
   use serde_with::skip_serializing_none;

   #[cfg(feature = "full")]
   pub mod impls;

   #[skip_serializing_none]
   #[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
   #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
   #[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
   pub struct ReputationSummaryView {
     pub person_id: i32,
     pub community_id: Option<i32>,
     pub reporting_accuracy: i32,
     pub jury_reliability: i32,
     pub participation_consistency: i32,
     pub endorsement_strength: i32,
     pub jury_eligible: bool,
     pub trusted_reporter: bool,
     /// Derived via a separate round-trip on `sanction` — see impls module.
     pub active_sanctions: i64,
   }

   #[skip_serializing_none]
   #[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
   #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
   #[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
   pub struct EndorsementSummaryView {
     pub person_id: i32,
     pub inbound_endorsements: i64,
     pub outbound_endorsements: i64,
     pub active_sureties: i64,
   }
   ```

   **Note.** `can_sponsor` is intentionally **not** in `ReputationSummaryView` because v0 does not read it. v1 adds a fourth boolean field and a new drift-stub block in this file.

3. **`impls.rs` — three queries.** Mirror the Phase 2b two-round-trip pattern (§8.3). All three queries use `tuple select → load<Tuple> → build_view()`; no `Selectable` derive because `active_sanctions: i64` is a bare-scalar kind (c) field per `.claude/rules/view-crate-selectable-template.md`.

   Query shapes:

   ```rust
   pub async fn read_reputation_summary(
     pool: &mut DbPool<'_>,
     person_id: PersonId,
     community_id: Option<CommunityId>,
   ) -> LemmyResult<Option<ReputationSummaryView>>;

   pub async fn list_endorsements_for_person(
     pool: &mut DbPool<'_>,
     person_id: PersonId,
     active_only: bool,
   ) -> LemmyResult<Vec<EndorsementSummaryView>>;
   // Signature note: the plan text says Vec, but semantically this returns one
   // row per person queried — since the caller passes a single person_id, the
   // Option<EndorsementSummaryView> signature is cleaner. Agent discretion:
   // match [04 §4.3] which says "list_endorsements_for_person" returns a list
   // of per-community endorsement counts. Keep Vec; justify in doc comment.

   pub async fn list_sureties_for_person(
     pool: &mut DbPool<'_>,
     person_id: PersonId,
     active_only: bool,
   ) -> LemmyResult<Vec<crate::EndorsementSummaryView>>;
   ```

4. **Wire into workspace `Cargo.toml`.** Add the new crate to `[workspace]` `members` and to the `[workspace.dependencies]` as `lemmy_db_views_reputation = { path = "crates/db_views/reputation", version = "..." }`.

5. **Add dep to downstream consumers.** `crates/api/api/Cargo.toml` gains `lemmy_db_views_reputation = { workspace = true }` for task 53's `recompute_snapshot` to reuse the view structs (optional — may prefer to access `ReputationSnapshot` directly via `lemmy_db_schema::source::governance::reputation_snapshot` and only depend on the view crate when Phase 5c adds `get_my_reputation`). Decide at implementation time; err on the side of adding it now.

**GOTCHAs (task 52).**

- **GOTCHA-52a.** Per `view-crate-selectable-template.md`, do NOT derive `Selectable` on `ReputationSummaryView`. The `active_sanctions: i64` field has no source column — it comes from a second-round-trip `SELECT COUNT(*) FROM sanction WHERE target_person_id = ? AND active = true`. If the impl agent tries to add `Selectable` "just this once", the compile error is the `governance_case_detail_rows` schema module lookup error cited in that rules file.
- **GOTCHA-52b.** `list_endorsements_for_person` returns a `Vec<EndorsementSummaryView>` per design-doc signature, but semantically the view carries per-person aggregates, not per-endorsement rows. Document in the doc comment that the `Vec` is empty when `person_id` has no endorsements (not `None`) and contains exactly 1 item otherwise.
- **GOTCHA-52c.** `endorsement.from_person_id` is the sponsor side; `endorsement.to_person_id` is the sponsee side. `surety.sponsor_id` is the sponsor; `surety.sponsored_id` is the sponsee. Column names **differ between the two tables** — do not confuse them. Verify with the §8 schema patterns (lines 370 and 1192 of `crates/db_schema_file/src/schema.rs`).

**Validation (task 52).**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_views_reputation --features full > .claude/build-task52-check.log 2>&1"
status=$?; tail -15 .claude/build-task52-check.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/build-task52-ws.log 2>&1"
status=$?; tail -15 .claude/build-task52-ws.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
```

**Commit.** `feat(views): task 52 — add crates/db_views/reputation view crate with summary + endorsement views`

---

### §12.4 Task 53 — Reputation snapshot calculator (expires_at filter, decay half-life guard, three capability booleans, FOR UPDATE concurrency guard, capability_changed log emit)

**Goal.** The heart of Phase 5a. Computes `ReputationSnapshot` rows from `ReputationEvent` history. Correct by-default against the five named watchpoints (2, 6, 8, 9, 11).

**Steps.**

1. **`crates/api/api/src/governance/reputation_snapshot.rs` — new file.**

   Function signatures:

   ```rust
   pub async fn recompute_snapshot(
     conn: &mut diesel_async::AsyncPgConnection,
     person_id: PersonId,
     community_id: Option<CommunityId>,
     config: &mut crate::governance::config::ConfigCache,
   ) -> LemmyResult<ReputationSnapshot>;

   pub async fn run_snapshot_batch(context: &LemmyContext) -> LemmyResult<SnapshotBatchOutcome>;

   pub struct SnapshotBatchOutcome { pub pairs_processed: usize, pub expired_founders: usize }

   #[derive(Debug, Clone)]
   pub struct CapabilityChange {
     pub dimension: CapabilityDimension, // jury_eligible | trusted_reporter | can_sponsor
     pub direction: CapabilityDirection, // Gained | Lost
   }

   pub fn detect_capability_changes(
     old: Option<&ReputationSnapshot>, // None = first snapshot ever written
     new: &ReputationSnapshot,
   ) -> Vec<CapabilityChange>;
   ```

2. **`recompute_snapshot` body (outline).**

   1. `SELECT * FROM reputation_snapshot WHERE person_id = ? AND community_id IS NOT DISTINCT FROM ? FOR UPDATE` — may return zero rows (first snapshot).
   2. Read reputation events: `reputation_event WHERE person_id = ? AND community_id IS NOT DISTINCT FROM ? AND (expires_at IS NULL OR expires_at > now())`. Load all events, group by dimension.
   3. For each dimension: sum the deltas, **applying decay only when `event.expires_at.is_none()`** (Watch 8) and the event's age exceeds `config.decay.positive_half_life_days` AND the delta is positive. Decay is in-memory (ADR-005 forbids writing back). Document: the event is halved once per half-life window it exceeds — for v0, halve once at >half_life_days; more aggressive schedules are v1.
   4. Read `person.published_at` for age (compute `account_age_days` in Rust). Read `sanction` for `active_sanctions` count.
   5. Compute three capability booleans:
      - `jury_eligible = jury_reliability >= config.thresholds.jury_reliability AND account_age_days >= config.jury.age_requirement_days AND active_sanctions == 0`
      - `trusted_reporter = reporting_accuracy >= config.thresholds.reporting_accuracy`
      - `can_sponsor = endorsement_strength >= config.thresholds.endorsement_strength`
   6. Build the new `ReputationSnapshot` struct.
   7. Upsert: `INSERT INTO reputation_snapshot (...) VALUES (...) ON CONFLICT (person_id, community_id) DO UPDATE SET ... WHERE reputation_snapshot.calculated_at < excluded.calculated_at` (last-writer-wins via calculated_at). The partial unique index from task 50 takes care of the `community_id IS NULL` case.
   8. `detect_capability_changes(old, new)` → `Vec<CapabilityChange>`.
   9. For each change, emit `governance_log::append(&mut conn.into(), "capability_changed", json!({"dimension_flipped": ..., "direction": ..., "snapshot_community_id": community_id}), Some(actor_pseudonym_helper::get_or_create(&mut conn.into(), person_id).await?))`. No raw person_id in payload (Watch 10).
   10. Return the new snapshot.

3. **`run_snapshot_batch` body.** Called by the clokwerk scheduler (task 54). **Chunked per GOTCHA-54f.**

   1. Read watermark: `SELECT MAX(calculated_at) FROM reputation_snapshot`.
   2. Read chunk size: `let chunk_size = config::get_int(..., Scope::Instance, "job.snapshot_batch_chunk_size").await?;` (default 500).
   3. Find distinct `(person_id, community_id)` pairs where:
      - `reputation_event.created_at > watermark` (new events since last tick), OR
      - `reputation_event.expires_at > watermark AND reputation_event.expires_at <= now()` (founder seeds that expired in this tick — forces recompute on the decay cliff).
      Order results by `person_id ASC` (single-column; no secondary ordering per GOTCHA-54f).
   4. **Chunk the pair list** into slices of `chunk_size`. For each chunk, open a fresh transaction; for each pair within the chunk, acquire `pg_advisory_xact_lock(hash(person_id, community_id))` per Watch 9, then call `recompute_snapshot`; commit the chunk transaction before starting the next chunk. A crash mid-chunk rolls back only the current chunk; earlier chunks stay committed and their `calculated_at` watermarks advance.
   5. Count + log tick-start / tick-end / pairs_total / chunks / chunk_size / expired_founders via `tracing::info!` with structured fields (S2 from design review + GOTCHA-54g).

4. **Prepend `ENTRY_KIND_*` const block in `crates/api/api/src/governance/governance_log.rs`.**

   ```rust
   /// Canonical v0 entry_kind strings for governance_log. Using a const at
   /// every call site turns typos into compile-time errors.
   pub const ENTRY_KIND_REPORT_CREATED: &str = "report_created";
   pub const ENTRY_KIND_THRESHOLD_MET: &str = "threshold_met";
   pub const ENTRY_KIND_JURY_ASSIGNED: &str = "jury_assigned";
   pub const ENTRY_KIND_PANEL_ASSEMBLED: &str = "panel_assembled";
   pub const ENTRY_KIND_JURY_VOTED: &str = "jury_voted";
   pub const ENTRY_KIND_SANCTION_CREATED: &str = "sanction_created";
   pub const ENTRY_KIND_PUBLIC_LOG_PUBLISHED: &str = "public_log_published";
   pub const ENTRY_KIND_REPUTATION_DELTA: &str = "reputation_delta";
   pub const ENTRY_KIND_CASE_DECIDED: &str = "case_decided";
   pub const ENTRY_KIND_CAPABILITY_CHANGED: &str = "capability_changed";
   pub const ENTRY_KIND_SPONSOR_LIABILITY_APPLIED: &str = "sponsor_liability_applied";
   pub const ENTRY_KIND_SPONSOR_LIABILITY_CLAMPED: &str = "sponsor_liability_clamped";
   pub const ENTRY_KIND_FOUNDER_SEEDED: &str = "founder_seeded";
   pub const ENTRY_KIND_ENDORSEMENT_CREATED: &str = "endorsement_created";
   pub const ENTRY_KIND_EMERGENCY_REMOVED: &str = "emergency_removed";
   ```

   Existing call sites (submit_jury_vote, admin_assign_jury, admin_emergency_remove, create_report) continue to work with string literals — conversion is out of 5a scope. New call sites (tasks 53, 55) use the constants.

**GOTCHAs (task 53).**

- **GOTCHA-53a.** **Watch 2.** The WHERE clause must be `expires_at IS NULL OR expires_at > now()`, not `expires_at IS NOT NULL AND expires_at > now()` (which would drop permanent organic events). If the agent writes the wrong predicate, the 5b sponsor-liability test (`honour_price_floor_clamp`) will fail because regular organic events disappear from the sum.
- **GOTCHA-53b.** **Watch 8.** The decay branch **must** be `if event.expires_at.is_none()` — a predicate distinct from the query-level filter. Without this, a founder seed with `expires_at = now() + 90d` and `delta = 100` is halved to 50 after 90 days AND cliff-filtered. Triple-discounted reputation is a silent bug because the snapshot row still writes a plausible number.
- **GOTCHA-53c.** **Watch 9.** The `FOR UPDATE` on the old_snapshot SELECT serialises concurrent recomputes inside a tx. The background job runs without a caller tx, so it acquires `pg_advisory_xact_lock(hash(person_id::int8 * 1000000 + COALESCE(community_id, 0)))` (or an equivalent deterministic hash) inside its own mini-tx. Two concurrent endorsement handlers targeting the same sponsee run under `run_transaction`; their `FOR UPDATE` blocks serialise, so both see consistent old_snapshot rows.
- **GOTCHA-53d.** **Watch 10.** `capability_changed` payload **must not** include `person_id` directly. Use `actor_pseudonym_helper::get_or_create` and put the pseudonym in the `actor_pseudonym` column (the fourth arg of `governance_log::append`), not in the payload JSON. Payload fields for `capability_changed`: `dimension_flipped` (string), `direction` (string "gained"|"lost"), `snapshot_community_id` (nullable int). That's it. No `person_id`, no `target_person_id`.
- **GOTCHA-53e.** **Watch 11.** When an admin raises `config.thresholds.jury_reliability` from 50 to 80 via psql, the NEXT tick of `run_snapshot_batch` cascades this into many `capability_changed` entries. The v0 posture per `advisor-context-phase-5.md §4 Watch 11` requires that the admin action itself leave an `admin_config_changed` governance_log entry, emitted via the wrapper script `scripts/brehon/admin-config-write.sh` (ship in task 51? — **no**, defer to optional task under 5c when `OQ-018` is closer; 5a documents the requirement in the migration's up.sql header comment and in `.claude/PRPs/reports/phase-5a-complete-report.md`, but does not ship the wrapper). The `capability_changed` entries this task emits are still attributable to the admin *through* the admin_config_changed entry whenever the admin uses the wrapper. Document in the phase-close report.
- **GOTCHA-53f.** The `ReputationSnapshotInsertForm` (Phase 1) already exists but does not include `can_sponsor`. Task 50's migration adds the column; the Diesel model struct + InsertForm must be updated in this task (or task 50 — whichever; the agent should prefer task 50 to keep migration + model in one commit). If left to task 53, the commit message must note the Phase-1-model amendment.
- **GOTCHA-53g.** The `run_snapshot_batch` function returns `LemmyResult<SnapshotBatchOutcome>`. Task 54's scheduler closure calls `.inspect_err().ok()`, consuming the Err branch and letting the tick loop continue. **Do not panic** on any error path inside `run_snapshot_batch` — all failures must be `Err(LemmyErrorType::...)`.
- **GOTCHA-53h.** The `SELECT ... FOR UPDATE` on the old_snapshot requires that the row exist. The first recompute of a person in a community (no prior row) cannot FOR UPDATE something that doesn't exist. The correct pattern is: `SELECT ... FROM reputation_snapshot WHERE person_id = ? AND community_id IS NOT DISTINCT FROM ?` — if zero rows, no FOR UPDATE needed (nothing to lock), AND the subsequent `INSERT ... ON CONFLICT DO UPDATE` uses the constraint itself for serialisation. Document in the doc comment; test the path where two concurrent recomputes of the same *new* pair race (each computes independently; one succeeds, the other hits ON CONFLICT and overwrites — the `WHERE calculated_at < excluded.calculated_at` guard prevents lost updates).
- **GOTCHA-53i. Naive `FOR UPDATE`, NOT `SKIP LOCKED` or `NOWAIT` (Perplexity-review 2026-04-17).** Watch 9's mitigation uses plain `SELECT ... FOR UPDATE` on the old_snapshot. Perplexity surfaced concern about potential deadlocks. **Rationale for keeping naive FOR UPDATE in v0**:
  - v0 has **exactly two recompute callers**: the background job (task 54's `run_snapshot_batch`) and the synchronous `create_endorsement` handler (task 55). Both go through the same `recompute_snapshot` function with the same lock-acquisition order. Asymmetric ordering — the textbook deadlock precondition — does not exist.
  - **`SKIP LOCKED` rejected** for v0: it would let the synchronous handler silently skip the recompute when the background job currently holds the lock. The user just endorsed someone; their capability indicator stays stale until the next background tick. That's worse UX than a ~50ms wait for the lock.
  - **`NOWAIT` rejected** for v0: it converts contention into a user-facing transient error (`LemmyErrorType::...`). Same UX hit as SKIP LOCKED plus explicit noise.
  - Workspace-default `statement_timeout` (check `postgresql.conf`) caps any pathological wait at tens of seconds — a loud failure mode beats a silent-skip one for a 5a that's shipping fresh code.
  - **v1 revisits** if pilot telemetry shows actual lock contention (Grafana dashboard of `pg_stat_activity` `wait_event` columns is the data signal). Likely v1 move: widen to a per-community advisory lock rather than per-(person, community) to collapse contention during admin config cascades, OR introduce SKIP LOCKED behind a feature flag once the stale-indicator UX harm is quantified.
  - **Impl-agent guidance**: do NOT defensively add `SKIP LOCKED` or `NOWAIT` mid-loop "to be safe". Naive FOR UPDATE is the advisor-approved v0 choice. If the impl agent feels the need to add either, queue a decision-queue question instead of committing.

**Validation (task 53).**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task53-check.log 2>&1"
status=$?; tail -15 .claude/build-task53-check.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/build-task53-ws.log 2>&1"
status=$?; tail -15 .claude/build-task53-ws.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
```

**Commit.** `feat(governance): task 53 — reputation snapshot calculator with expires_at filter, decay half-life guard, capability_changed log emit, FOR UPDATE concurrency guard`

---

### §12.5 Task 54 — Snapshot recalculation background job (clokwerk scheduler, BREHON_DISABLE_BACKGROUND_JOBS=1 test override, never-exit loop via inspect_err)

**Goal.** Wire `run_snapshot_batch` into Lemmy's existing clokwerk scheduler so snapshots are fresh at jury-assignment time for 5b. Keep `crates/server/src/governance.rs` declarative.

**Steps.**

1. **`crates/routes/src/utils/scheduled_tasks.rs` — add a new scheduler block.**

   Where: just after the daily scheduler (line 145 region, before the `loop { ... }` at line 148).

   ```rust
   // Brehon governance: reputation snapshot recalculation. Every 15
   // minutes, find users with new events since the last tick (or with
   // founder seeds that just expired) and recompute their snapshot row.
   // Controlled by `job.snapshot_interval_seconds` config at v1; the
   // clokwerk schedule-at-registration time means a config change needs
   // a server restart in v0 (acceptable limitation).
   let context_gov_snapshot = context.reset_request_count();
   scheduler.every(CTimeUnits::minutes(15)).run(move || {
     let context = context_gov_snapshot.reset_request_count();
     async move {
       // Test override: e2e tests set this env var to call run_snapshot_batch
       // directly without the scheduler racing them. S4 from design review.
       if std::env::var("BREHON_DISABLE_BACKGROUND_JOBS").as_deref() == Ok("1") {
         return;
       }
       lemmy_api::governance::reputation_snapshot::run_snapshot_batch(&context)
         .await
         .inspect_err(|e| warn!("Failed to run snapshot batch: {e}"))
         .ok();
     }
   });
   ```

2. **`crates/routes/Cargo.toml` — verify `lemmy_api` is a workspace dep.** It should already be (routes imports from it for route registration). If not, add.

3. **`crates/server/src/governance.rs` — update the stub message.** The file stays ≤30 lines. Change the `info!` message from "not yet scheduled" to "registered via scheduled_tasks (15-minute tick)" so the startup log confirms the composition root knows the job is live.

   ```rust
   pub fn schedule_governance_jobs(_context: &LemmyContext) {
     info!(
       "governance: snapshot recalculation job registered via \
        lemmy_api::governance::reputation_snapshot::run_snapshot_batch \
        (15-minute tick in scheduled_tasks::setup; BREHON_DISABLE_BACKGROUND_JOBS=1 disables for e2e)",
     );
   }
   ```

**GOTCHAs (task 54).**

- **GOTCHA-54a.** **Advisor narrative drift.** IMPLEMENTATION-PLAN-v0.md §3 Phase 5a task 54 says `tokio::spawn` directly from `crates/server/src/governance.rs`. That conflicts with Lemmy's single-scheduler convention in `crates/routes/src/utils/scheduled_tasks.rs` AND with 03 §11's "server stays declarative" rule. **Resolution**: implement via the clokwerk pattern. Document the deviation in the task 54 commit message and in `.claude/PRPs/reports/phase-5a-complete-report.md` so the advisor can raise an ADR fixup if needed. The behaviour (15-minute tick, error-logged, never-exit) is identical either way.
- **GOTCHA-54b.** **Watch 6.** The `.inspect_err().ok()` pattern is the Lemmy-native way to log-and-continue inside a clokwerk closure (see `scheduled_tasks.rs:74`). The scheduler's outer `loop { scheduler.run_pending().await; tokio::time::sleep(...).await; }` at line 148 ensures ticks continue regardless of closure outcome. Never introduce an inner `loop {}` inside the closure — clokwerk's `run()` wants a single-shot async block.
- **GOTCHA-54c.** **BREHON_DISABLE_BACKGROUND_JOBS=1** is an env-var check inside the closure, not at registration time. The e2e test must set it BEFORE spawning the server's scheduler (i.e. before `setup()` is called). In practice, `tests/e2e.rs` spawns its own `scheduled_tasks::setup` task OR bypasses it entirely (the existing e2e tests do not spin up the HTTP server; they drive the DB directly). Verify at task 54 implementation time whether any existing test setup calls `setup()` — if not, the env-var is defensive for future e2e tests that do.
- **GOTCHA-54d.** The 15-minute interval is hardcoded in the scheduler registration, but the config key `job.snapshot_interval_seconds = 900` (seeded in task 50) exists for v1. Document in the registration comment that flipping the config to a new value requires a server restart; live-re-schedule is v1.
- **GOTCHA-54e.** `context.reset_request_count()` is the Lemmy-native pattern for passing a `Data<LemmyContext>` into a cron closure without carrying over the request-count state — see `scheduled_tasks.rs:108` (daily block). Do the same here.
- **GOTCHA-54f. Chunked batch semantics (Perplexity-review 2026-04-17).** `run_snapshot_batch` MUST process dirty `(person_id, community_id)` pairs in chunks of `config.job.snapshot_batch_chunk_size` (default 500, seeded by task 50), ordered by `person_id ASC`, committing after each chunk. Without chunking, an admin config edit that cascades into mass capability recomputes (e.g. raising `thresholds.jury_reliability` from 50 to 80) builds up one gigantic transaction — bloats `pg_wal`, holds locks for the whole run, and interacts badly with the `FOR UPDATE` per-pair acquired inside each recompute. Rules for the impl:
  - **Ordering**: `ORDER BY person_id ASC` only. Single-column ordering keeps chunking deterministic and resumable between ticks. **Do NOT** add secondary `ORDER BY` (created_at / priority / community_id) — no multi-column ordering in v0. Ranking is a v1 concern if pilot data shows freshness-starvation on high-activity users.
  - **Commit discipline**: one transaction per chunk. Chunk N completes + commits; chunk N+1 starts a fresh transaction. A scheduler tick processing 3,200 dirty pairs therefore runs 7 transactions (6 full + 1 partial), each under its own `FOR UPDATE` footprint.
  - **Chunk size read-at-tick**: `config::get_int(..., "job.snapshot_batch_chunk_size")` is read once at the start of each `run_snapshot_batch` invocation; editing the config between ticks takes effect on the next tick (no server restart for chunk size — unlike the interval, which is clokwerk-registered once at setup-time per GOTCHA-54d).
  - **`capability_changed` log emission stays one-per-user-per-tick** — no per-tick burst-collapse, no batching by community, no dedupe across chunks. For v0 pilot scale (single-digit thousands of users) this is correct; v1 revisits if community size grows past 10k. Advisor-approved 2026-04-17.
  - **No retry-resume** across tick boundaries. If the job crashes mid-chunk, next tick restarts from the watermark. The partial chunk's commits stay (previous chunks already committed); the uncommitted chunk rolls back and is re-picked-up by the new-events query on the next tick.
- **GOTCHA-54g. Observability.** Log the chunk count + dirty-pair total + expired-founder count at each `run_snapshot_batch` invocation using structured `tracing::info!` fields (S2 from design review). Example field shape: `pairs_total = 3200, chunks = 7, chunk_size = 500, expired_founders = 3`. This is cheap and makes pilot-time tuning of chunk size trivial.

**Validation (task 54).**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_routes --features full > .claude/build-task54-routes.log 2>&1"
status=$?; tail -15 .claude/build-task54-routes.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features full > .claude/build-task54-server.log 2>&1"
status=$?; tail -15 .claude/build-task54-server.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/build-task54-ws.log 2>&1"
status=$?; tail -15 .claude/build-task54-ws.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
```

**Commit.** `feat(governance): task 54 — register snapshot recalc job via scheduled_tasks (15-minute clokwerk tick, BREHON_DISABLE_BACKGROUND_JOBS=1 override)`

---

### §12.6 Task 55 — `create_endorsement` handler (config-driven gate-strategy dispatch)

**Goal.** The only new HTTP endpoint in 5a. Config-driven `'age' | 'open' | 'closed'` dispatch; deltas + conditional surety insert; recompute both parties; governance log.

**Steps.**

1. **`crates/api/api_common/src/governance.rs` — add response DTO.** Insert after `CreateEndorsement` at line 193:

   ```rust
   #[skip_serializing_none]
   #[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
   #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
   #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
   /// Response from creating an endorsement.
   pub struct CreateEndorsementResponse {
     pub endorsement_id: EndorsementId,
     pub surety_created: bool,
   }
   ```

2. **`crates/api/api_crud/src/governance/create_endorsement.rs` — new file.**

   Shape — see §8.4 for the `run_transaction` outer wrapper, then the inner `process_endorsement` function:

   ```rust
   async fn process_endorsement(
     conn: &mut AsyncPgConnection,
     sponsor_id: PersonId,
     sponsor_pseudonym: String,
     data: CreateEndorsement,
     mut config: ConfigCache,
   ) -> LemmyResult<CreateEndorsementResponse> {
     // Step 1 — read the config gate strategy.
     let strategy_str = config::get_text(&mut config, &mut conn.into(), Scope::Instance,
                                          "onboarding.sponsor_gate_strategy").await?;
     let strategy = SponsorGateStrategy::parse(&strategy_str);

     // Step 2 — dispatch on strategy.
     match strategy {
       SponsorGateStrategy::Closed => {
         warn!("endorsement attempt under 'closed' gate from {sponsor_pseudonym}");
         return Err(LemmyErrorType::NotFound.into());
       }
       SponsorGateStrategy::Open => { /* bypass */ }
       SponsorGateStrategy::Age => {
         let min_age = config::get_int(&mut config, &mut conn.into(), Scope::Instance,
                                        "onboarding.sponsor_min_account_age_days").await?;
         let sponsor_published: DateTime<Utc> = person::table
           .filter(person::id.eq(sponsor_id))
           .select(person::published_at)
           .first(conn).await?;
         let age_days = (Utc::now() - sponsor_published).num_days();
         if age_days < min_age {
           return Err(LemmyErrorType::NotFound.into());
         }
       }
       SponsorGateStrategy::Unknown(s) => {
         warn!("unknown sponsor_gate_strategy '{s}' — falling back to 'age'");
         // Fall through to the 'age' branch above by re-dispatch — or duplicate the
         // age check inline here. Inline keeps one arm per strategy; prefer that.
         let min_age = config::get_int(...).await?;
         // ... same as Age
       }
     }

     // Step 3 — target must exist and not be self.
     if data.person_id == sponsor_id {
       return Err(LemmyErrorType::CantBlockYourself.into());
       // (or a governance-specific error; 5a may add LemmyErrorType::CantEndorseYourself
       // via the lemmy_utils error enum — decide at implementation time; reuse existing
       // error type preferred to keep scope small)
     }
     Person::read(&mut conn.into(), data.person_id).await?; // errors if missing

     // Step 4 — max-5 active endorsements from caller.
     let active_count: i64 = endorsement::table
       .filter(endorsement::from_person_id.eq(sponsor_id))
       .filter(endorsement::revoked_at.is_null())
       .count().get_result(conn).await?;
     if active_count >= 5 {
       return Err(LemmyErrorType::NotFound.into()); // or a dedicated error
     }

     // Step 5 — 48h cooldown from caller's last endorsement.
     let cutoff = Utc::now() - Duration::hours(48);
     let recent_count: i64 = endorsement::table
       .filter(endorsement::from_person_id.eq(sponsor_id))
       .filter(endorsement::created_at.gt(cutoff))
       .count().get_result(conn).await?;
     if recent_count > 0 {
       return Err(LemmyErrorType::NotFound.into()); // cooldown
     }

     // Step 6 — insert endorsement row.
     let form = EndorsementInsertForm {
       from_person_id: sponsor_id,
       to_person_id: data.person_id,
       community_id: data.community_id,
     };
     let e: Endorsement = insert_into(endorsement::table)
       .values(&form).get_result(conn).await?;

     // Step 7 — conditional surety row (sponsee has <2 active sureties).
     let active_sureties: i64 = surety::table
       .filter(surety::sponsored_id.eq(data.person_id))
       .filter(surety::revoked_at.is_null())
       .count().get_result(conn).await?;
     let surety_created = if active_sureties < 2 {
       let sf = SuretyInsertForm {
         sponsor_id,
         sponsored_id: data.person_id,
         community_id: data.community_id,
       };
       insert_into(surety::table).values(&sf).execute(conn).await?;
       true
     } else { false };

     // Step 8 — emit two reputation_event rows.
     let sponsor_delta = config::get_int(&mut config, ..., "deltas.endorsement_created_sponsor").await?;
     let sponsee_delta = config::get_int(&mut config, ..., "deltas.endorsement_created_sponsee").await?;
     insert_into(reputation_event::table).values(ReputationEventInsertForm {
       person_id: sponsor_id,
       community_id: data.community_id,
       dimension: ReputationDimension::EndorsementStrength,
       delta: sponsor_delta as i32,
       source_case_id: None,
       source_report_id: None,
       reason: "endorsement_created_sponsor".to_string(),
       expires_at: None,
     }).execute(conn).await?;
     insert_into(reputation_event::table).values(ReputationEventInsertForm {
       person_id: data.person_id,
       community_id: data.community_id,
       dimension: ReputationDimension::ParticipationConsistency,
       delta: sponsee_delta as i32,
       ..Default::default()
     }).execute(conn).await?;

     // Step 9 — recompute both snapshots (immediate read-your-writes).
     reputation_snapshot::recompute_snapshot(conn, sponsor_id, data.community_id, &mut config).await?;
     reputation_snapshot::recompute_snapshot(conn, data.person_id, data.community_id, &mut config).await?;

     // Step 10 — governance log entry.
     let target_pseudonym =
       actor_pseudonym_helper::get_or_create(&mut conn.into(), data.person_id).await?;
     governance_log::append(
       &mut conn.into(),
       governance_log::ENTRY_KIND_ENDORSEMENT_CREATED,
       json!({
         "sponsor_pseudonym": sponsor_pseudonym,
         "target_pseudonym":  target_pseudonym,
         "community_id":      data.community_id.map(|c| c.0),
         "gate_strategy":     strategy.label(),
         "surety_created":    surety_created,
       }),
       Some(sponsor_pseudonym),
     ).await?;

     Ok(CreateEndorsementResponse { endorsement_id: e.id, surety_created })
   }
   ```

3. **`crates/api/api_crud/src/governance/mod.rs` — export.** Add `pub mod create_endorsement; pub use create_endorsement::create_endorsement;`.

4. **`crates/api/routes/src/lib.rs` line 500 area** — insert `.route("/endorsement", post().to(create_endorsement))` inside the `scope("/governance")` block, after `.route("/report", ...)` per §8.6.

**GOTCHAs (task 55).**

- **GOTCHA-55a.** The `SponsorGateStrategy` enum in this file uses `Unknown(String)` as an exhaustive-but-final arm rather than `_ =>` — the latter is forbidden by `feedback_clippy_test_style`. `parse()` returns `Unknown(s)` for anything not `"age" | "open" | "closed"`; the match arm logs and falls through to `'age'` behaviour.
- **GOTCHA-55b.** `can_sponsor` is NOT checked by any gate strategy in v0 per OQ-014. The `reputation_snapshot.can_sponsor` column added in task 50 is populated by task 53 but not read here. The `lint-no-can-sponsor-read.sh` script (task 51) verifies this at phase-close.
- **GOTCHA-55c.** `Endorsement` table columns are `from_person_id` / `to_person_id`; `surety` table columns are `sponsor_id` / `sponsored_id`. Do NOT confuse them. The diff — endorsement is the semantic (sender→receiver); surety is the legal chain (sponsor guarantees sponsored). Verify schema at `crates/db_schema_file/src/schema.rs:370` and `:1192`.
- **GOTCHA-55d.** The max-5 check counts `revoked_at IS NULL`; the 48h-cooldown check counts rows with `created_at > now() - 48h` regardless of `revoked_at` (a revoked endorsement still counts against the cooldown because the burst of intent happened). Document in the doc comment.
- **GOTCHA-55e.** `recompute_snapshot` inside the tx is safe per Watch 9 (`FOR UPDATE` + `ON CONFLICT`). Concurrent endorsements targeting the same sponsee by different sponsors will both call `recompute_snapshot(sponsee_id)` — the FOR UPDATE serialises them, each sees the other's delta before computing. The `governance_log` may show interleaved `capability_changed` entries; acceptable per plan task 55 narrative.
- **GOTCHA-55f.** The error type returned by the guards (strategy=closed, age < min, max-5, cooldown, self-endorse) is currently `LemmyErrorType::NotFound` — this is a conscious v0 choice for the `'closed'` case per the plan text. For the others (age gate fail, max-5, cooldown, self-endorse), consider introducing a dedicated error like `LemmyErrorType::EndorsementRejected` in `crates/utils/src/error.rs`. If that's upstream-held, fall back to `NotFound` to avoid an upstream patch in 5a; note as a carry-patch TODO.
- **GOTCHA-55g.** The `run_transaction` wrapper (mirror from admin_assign_jury at §8.4) constructs `context.pool()` outside the tx and passes `conn: &mut AsyncPgConnection` inside. The ConfigCache is a per-request cache constructed at handler entry (`ConfigCache::new()`) and threaded into the tx closure via `mut`. All calls to `config::get_*` flow through this cache — the first read per key hits the DB, subsequent reads in the same handler use the cached value.
- **GOTCHA-55h.** `reputation_event.expires_at` on the two new events must be `None` (permanent organic events — not founder seeds). Default is `Some` per the `ReputationEventInsertForm` shape, so be explicit: `expires_at: None`.

**Validation (task 55).**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_crud --features full > .claude/build-task55-crud.log 2>&1"
status=$?; tail -15 .claude/build-task55-crud.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
# Note per `feedback_api_crud_oauth_feature_quirk.md`: -p lemmy_api_crud may
# false-red on user/create.rs OAuth reqwest code. If it does, fall back to
# --workspace below; the workspace check catches the same errors.

cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/build-task55-ws.log 2>&1"
status=$?; tail -15 .claude/build-task55-ws.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_routes --features full > .claude/build-task55-routes.log 2>&1"
status=$?; tail -15 .claude/build-task55-routes.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
```

**Commit.** `feat(governance): task 55 — create_endorsement handler with config-driven gate-strategy dispatch (age|open|closed) per OQ-014`

---

### §12.7 Task 56 — Phase-close validation + PR

**Goal.** Prove the regression invariants and open the PR. Every command here runs unconditionally against the task-55 HEAD.

**Steps.**

1. **Run every §14 validation command.** Capture to files; tail only.

2. **Run `report_to_modlog_golden_path`.** The single most load-bearing regression guard.
   ```bash
   cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e --features full -- report_to_modlog_golden_path > .claude/build-task56-golden.log 2>&1"
   status=$?; tail -30 .claude/build-task56-golden.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
   ```

3. **Run both lint guards.**
   ```bash
   bash scripts/brehon/lint-no-membership-read.sh; echo "membership exit: $?"
   bash scripts/brehon/lint-no-can-sponsor-read.sh; echo "can_sponsor exit: $?"
   ```

4. **Write the completion report** at `.claude/PRPs/reports/phase-5a-complete-report.md`. Structure: (a) delivered vs plan; (b) commit list + SHAs; (c) deviations from plan (especially task 54's scheduler choice); (d) decision-queue entries opened/closed; (e) carry-forward into 5b/5c; (f) retro nomination (short form per advisor rule 12, full if checkpoint fires).

5. **Open PR.**
   ```bash
   git push -u origin phase-5a
   gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-5a \
     --title "Phase 5a — governance_config, membership_state, reputation infra, create_endorsement" \
     --body "$(cat <<'EOF'
   ## Summary
   - governance_config table + reader (Rust const parity)
   - person.membership_state deferred-enforcement column + grep-guard CI
   - crates/db_views/reputation view crate
   - reputation_snapshot calculator + 15-min background job
   - create_endorsement handler (config-driven age|open|closed gate)

   ## Completion report
   `.claude/PRPs/reports/phase-5a-complete-report.md`

   ## Plan reference
   `.claude/PRPs/plans/phase-5a-config-and-reputation-infrastructure.plan.md`
   Design docs (homeserver):
   `docs/research/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` §3 Phase 5a

   ## Regression guard
   - Phase 4 `report_to_modlog_golden_path` passes
   - 8 existing e2e tests pass
   - Zero clippy warnings under `--features full --workspace --no-deps -- -D warnings`
   EOF
   )"
   ```

**Validation (task 56).** Every level in §14 passes.

**Commit.** `docs(report): Phase 5a complete — governance_config + reputation infra + create_endorsement`

---

## §13. Testing Strategy

### 13.1 New tests in 5a

**None of task 48's nine-test list is added in 5a.** The 5a DoD is regression-only on existing e2e tests.

### 13.2 Compile-time tests

- `lemmy_api::governance::config::parity::seeded_keys_count_matches_const_count` — Watch 1 mitigation. Fails the task 50 commit if the list desyncs.

### 13.3 Regression guards

- `postgres_container_boots` (Phase 0)
- `can_insert_moderation_case` (Phase 1)
- `governance_log_hash_chain_holds` (Phase 1)
- `list_open_cases_returns_seeded_rows` (Phase 2a)
- `jury_queue_view_returns_assignments` (Phase 2a)
- `modlog_view_returns_published_entries` (Phase 2b)
- `redaction_strips_identifiers` (Phase 4)
- `report_to_modlog_golden_path` (Phase 4b)

All 8 must stay green. The migrations added by task 50 and 51 run via `embed_migrations!("../../migrations")` at `crates/server/tests/e2e.rs:51`; no harness change is needed for the new migrations to be picked up.

### 13.4 What's NOT tested in 5a

- **Sponsor-liability with founder multiplier** — Phase 5b task 60.
- **Founder-chain survival / honour-price floor clamp** — Phase 5b task 60.
- **Ineligible user cannot be picked for jury** — Phase 5c task 69.
- **All MVP endpoints 200 on happy path** — Phase 5c task 68 (six endpoints still unshipped after 5a).
- **Snapshot recomputation race with concurrent endorsement** — Phase 5b task 60's `honour_price_floor_clamp` exercises the path transitively; the explicit concurrency test is deferred to 5b/5c.

---

## §14. Validation Commands

Use the wrapper scripts exclusively. Every invocation → file → `$?` → tail per `.claude/rules/cargo-output-capture.md` and `.claude/rules/no-cargo-output-paste.md`.

### Level 0 — CARRY_PATCH_PRECONDITIONS (must pass before Level 1 is meaningful)

Pre-5a clippy dry-run against HEAD `26274db05` confirmed a pre-existing upstream lint-expectation failure at `crates/diesel_utils/src/pagination.rs:220` — `#[expect(clippy::multiple_bound_locations)]` under `-D warnings → -D unfulfilled-lint-expectations`. Task 0 lands a `chore(lint)` carry-patch commit swapping `#[expect]` → `#[allow]` (Option B per user direction 2026-04-17). Level 0 asserts the carry-patch is in place before Level 1 is trusted:

```bash
# Grep-assert the carry-patch is landed. The line should now read `#[allow(...)`;
# the old `#[expect(...)]` form must be gone.
grep -n 'multiple_bound_locations' crates/diesel_utils/src/pagination.rs
# EXPECT exactly one line like:
#   220:#[allow(clippy::multiple_bound_locations)]  // TODO(brehon-fork): upstream to LemmyNet/lemmy — PR #___
# FAIL if `#[expect(` appears — the carry-patch has not landed.
grep -q '^#\[allow(clippy::multiple_bound_locations)\]' crates/diesel_utils/src/pagination.rs || {
  echo 'ERROR: pre-5a carry-patch on pagination.rs:220 not landed. Run task 0 step 4 first.'; exit 1;
}
grep -q '#\[expect(clippy::multiple_bound_locations)\]' crates/diesel_utils/src/pagination.rs && {
  echo 'ERROR: stale #[expect(...)] attribute still present on pagination.rs:220. Replace with #[allow(...)].'; exit 1;
} || true
```

**EXPECT.** Attribute has been replaced; carry-patch commit appears in `git log --oneline phase-5a` between the branch-cut and the task-50 commit.

### Level 1 — STATIC_ANALYSIS

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/l1-check.log 2>&1"
status=$?; tail -20 .claude/l1-check.log; echo "check exit: $status"; [ $status -eq 0 ] || exit 1

cmd //c "scripts\\brehon\\cargo-clippy.bat --features full --workspace --no-deps -- -D warnings > .claude/l1-clippy.log 2>&1"
status=$?; tail -40 .claude/l1-clippy.log; echo "clippy exit: $status"; [ $status -eq 0 ] || exit 1
```

**Plan-drift note.** `cargo check --no-deps` is not a valid flag on this cargo version (verified at plan-write time — see task 0 audit). The narrative in IMPLEMENTATION-PLAN-v0.md §3 Phase 5a DoD reading `cargo check --features full --workspace --no-deps` is unexecutable. This plan uses `cargo check --features full --workspace` (no `--no-deps`). Clippy supports `--no-deps` and keeps it per design intent (avoid lint debt from upstream deps).

**Wrapper exit-code propagation — VERIFIED at plan-write time.** Both `scripts/brehon/cargo-check.bat` (line 20) and `scripts/brehon/cargo-clippy.bat` (line 20) have `cargo.exe <subcommand> %*` as the final statement; Windows batch files inherit the final command's exit code, so a non-zero cargo exit propagates through `cmd /c` to `$?` in bash. Verified in the foreground: `cmd //c scripts\brehon\cargo-clippy.bat ... -- -D warnings` exited `101` (cargo's lint-violation code) when `pagination.rs:220` was red, and `0` when the carry-patch landed.

**Background-task exit-code WARNING.** The Bash tool's `run_in_background` mode reports `exit code 0` in `<task-notification>` summaries even when the underlying `cmd` returns non-zero — observed at plan-write time for the identical clippy invocation. **Always run DoD validation commands in the foreground**, or always cross-check the log tail for `error:` / `warning: build failed` markers before trusting the `exit code` line in a notification. `.claude/rules/cargo-output-capture.md` covers the pipe-masking-exit variant; this is the sibling warning specific to the background-task summary layer.

**EXPECT.** Exit 0, zero errors, zero warnings.

### Level 2 — UNIT/INTEGRATION TESTS

```bash
# Seed-const structural parity (in-crate, no DB).
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_api --features full -- governance::config::parity > .claude/l2-parity.log 2>&1"
status=$?; tail -15 .claude/l2-parity.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1

# Full e2e regression suite — includes `config_parity_round_trip` per
# GOTCHA-50h (round-trip via ConfigCache::get_<type>(), Perplexity-review
# 2026-04-17 item 5).
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e --features full > .claude/l2-e2e.log 2>&1"
status=$?; tail -40 .claude/l2-e2e.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
```

**EXPECT.** All 8 existing e2e tests pass; the new `config_parity_round_trip` test passes; structural parity test passes.

### Level 3 — FULL_BUILD

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features full > .claude/l3-e2e-compile.log 2>&1"
status=$?; tail -15 .claude/l3-e2e-compile.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1
```

**EXPECT.** Exit 0.

### Level 4 — MIGRATION_VALIDATION

The existing e2e harness (`governance_fixtures::apply_all_schema`) runs every migration in `migrations/` at the start of any e2e test. Level 2's `l2-e2e.log` therefore exercises the migrations. Additionally:

```bash
# Round-trip: run forwards, back, forwards, verify no drift.
# The harness does not directly expose migration revert, so the round-trip
# validation is: a fresh container + apply_all_schema works (already in
# l2-e2e). A secondary smoke — the `can_insert_moderation_case` test is
# the canonical migration round-trip — ensures the `threshold_score: 1`
# insert still works after the micros rescale (the test inserts 1 directly,
# which the rescale touches; verify the test assertion either ignores
# threshold_score or the rescale leaves seeded 1 alone because the UPDATE
# only runs on existing rows at migration time, not in the test insert).
```

**EXPECT.** `can_insert_moderation_case` still passes without modification — its `threshold_score: 1` literal is written post-rescale, so it stays `1` (the rescale `UPDATE moderation_case SET threshold_score = threshold_score * 1000000` runs once, before the test's insert, on zero rows).

### Level 5 — CROSS_CUTTING_VERIFICATION

```bash
# Guards from task 51.
bash scripts/brehon/lint-no-membership-read.sh; echo "membership guard exit: $?"
bash scripts/brehon/lint-no-can-sponsor-read.sh; echo "can_sponsor guard exit: $?"

# Grep every new governance-log payload from 5a for PII leakage (Watch 10).
grep -rn --include='*.rs' 'capability_changed\|endorsement_created' crates/api/api/src/governance/reputation_snapshot.rs crates/api/api_crud/src/governance/create_endorsement.rs \
  | grep -E 'person_id|target_person_id|sponsored_id|sponsor_id' \
  | grep -v 'actor_pseudonym_helper::get_or_create' \
  | grep -v 'match\|fn ' \
  || echo "PII grep: no direct identifier usage in new payloads"

# Check that governance_log::append is the only writer to governance_log in new 5a code.
grep -rn --include='*.rs' 'insert_into(governance_log::table)' crates/ \
  | grep -v 'crates/api/api/src/governance/governance_log.rs' \
  || echo "single writer guard: pass"
```

**EXPECT.** Both guards exit 0. PII grep returns no findings with raw identifiers in payloads. Single writer guard confirms only `governance_log.rs:76` inserts into `governance_log`.

### Level 6 — MANUAL_VALIDATION

Optional smoke:

```bash
# Start a throwaway Postgres, run migrations, read back.
docker run --rm -d --user 1000:1000 --name brehon-smoke -e POSTGRES_USER=lemmy -e POSTGRES_PASSWORD=password -e POSTGRES_DB=lemmy -p 5555:5432 pgautoupgrade/pgautoupgrade:18-alpine
# Wait for the container to be ready (check `docker logs brehon-smoke` for "ready to accept connections").
psql -h localhost -p 5555 -U lemmy -d lemmy -c "SELECT scope, key, value_type, COALESCE(value_int::text, value_float::text, value_bool::text, value_text) FROM governance_config_current ORDER BY scope, key;"
docker stop brehon-smoke
```

**EXPECT.** 33 rows returned, all `scope = 'instance'`, all with populated typed columns matching the `CHECK` constraint.

---

## §15. Acceptance Criteria + Completion Checklist

### Acceptance criteria

- [ ] Pre-5a carry-patch `chore(lint): clear unfulfilled lint expectation in pagination.rs pre-5a (carry-patch)` committed on `phase-5a` before task 50.
- [ ] All six substantive tasks (50–55) ship with matching commit messages.
- [ ] Level 0 (carry-patch precondition grep) green.
- [ ] Level 1 (check + clippy) green.
- [ ] Level 2 (parity test + e2e regression) green.
- [ ] Level 3 (e2e compile) green.
- [ ] Level 5 (guards + PII grep) green.
- [ ] `report_to_modlog_golden_path` passes on the phase-5a HEAD.
- [ ] The 15 ADRs in [99] remain uncontradicted. Any near-miss is logged in the completion report.
- [ ] Cross-cutting §4 of IMPLEMENTATION-PLAN-v0.md is respected by both new handlers (task 53 + task 55): every governance write passes through `governance_log::append` + `actor_pseudonym_helper::get_or_create` + `redaction::scrub_json`.
- [ ] The three watchpoint-driven code sites are verifiable by grep:
  - Watch 2: `expires_at IS NULL OR expires_at > now()` in `reputation_snapshot.rs`
  - Watch 8: `if event.expires_at.is_none()` in the decay branch in `reputation_snapshot.rs`
  - Watch 9: `FOR UPDATE` on the old_snapshot SELECT in `reputation_snapshot.rs`
- [ ] **Perplexity-review 2026-04-17 acceptance additions**:
  - Task 50 seed list contains `job.snapshot_batch_chunk_size = 500` (grep: `grep -c "job.snapshot_batch_chunk_size" migrations/2026-04-18-000000-0000_add_governance_config/up.sql` returns `1`).
  - Task 50 Rust reader exposes `DEFAULT_JOB_SNAPSHOT_BATCH_CHUNK_SIZE` const AND the parity test includes it in `SEEDED_KEYS_WITH_CONSTS` (total count = 34).
  - Task 53/54 `run_snapshot_batch` reads `config.job.snapshot_batch_chunk_size` at tick-start and chunks with `ORDER BY person_id ASC` — grep confirms `snapshot_batch_chunk_size` in `reputation_snapshot.rs`.
  - Task 50 parity test round-trips through `ConfigCache::get_<type>()` for every seeded key (not just existence) — grep: the parity test body calls `config.get_int`/`get_float`/`get_bool`/`get_text` inside the loop.
  - Task 53 GOTCHA-53i (naive FOR UPDATE rationale) is present and cites the "one bg job + one sync caller" v0 structure.
- [ ] PR open `phase-5a → governance-v0`; CodeRabbit auto-review begins.

### Completion checklist

- [ ] Task 0 audit completed and logs captured.
- [ ] Tasks 50–55 committed one commit each.
- [ ] Task 56 phase-close report written.
- [ ] Branch pushed to `origin/phase-5a`.
- [ ] PR opened with `--repo barrie-cork/lemmy`.
- [ ] Decision-queue entries #11 and #12 left untouched (they belong to 5b).
- [ ] No commits directly on `governance-v0` — all work on `phase-5a`.

---

## §16. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Pre-phase clippy baseline is red (upstream lint debt) — **CONFIRMED at plan-write time** | ACTIVE | MED | Pre-5a clippy dry-run at `.claude/audit-5a-clippy.log` fails with: `error: this lint expectation is unfulfilled --> crates/diesel_utils/src/pagination.rs:220:10 #[expect(clippy::multiple_bound_locations)]` under `-D unfulfilled-lint-expectations` (implied by `-D warnings`). **Task 0 must land a `chore(lint): clear unfulfilled expectation in pagination.rs (carry-patch)` commit** as a pre-task-50 fixup on the `phase-5a` branch — change the attribute at `crates/diesel_utils/src/pagination.rs:220` from `#[expect(clippy::multiple_bound_locations)]` to `#[allow(clippy::multiple_bound_locations)]` (or delete if the underlying lint no longer fires), with a `TODO(brehon-fork): upstream this to LemmyNet/lemmy — PR #___` marker per `feedback_carry_patch_todos.md`. The fallback is to narrow the §14 Level 1 clippy DoD to the 5a-touched crates only (`-p lemmy_api -p lemmy_api_crud -p lemmy_routes -p lemmy_db_views_reputation -p lemmy_db_schema -p lemmy_server`) — documented in completion report either way. Cargo check baseline is confirmed green; only clippy is affected. |
| `cargo check --features full --workspace --no-deps` DoD is unexecutable (verified at plan-write time) | HIGH | MED | Plan's Level 1 command drops `--no-deps` from check; keeps it on clippy. Completion report notes the upstream IMPLEMENTATION-PLAN-v0.md text needs a doc fix (advisor discretion). |
| `ALTER TABLE reputation_snapshot ADD COLUMN can_sponsor` blocks on a large table | LOW | LOW-MED | v0 dev + pilot DBs have <1M rows; Postgres 11+ fast-path is metadata-only for non-volatile defaults. Production migration timing is a v1 ops concern — flagged in completion report. |
| `threshold_score * 1000000` overflows i64 for existing rows | VERY LOW | HIGH | Phase 4 `threshold_score` column is `i64` (`BIGINT` in schema); max seeded value is `V0_THRESHOLD = 3`; `3 * 1e6 = 3e6` is nowhere near i64 max. Sanity assertion: `SELECT MAX(threshold_score) FROM moderation_case` pre-rescale; refuse if > `9e12`. |
| Config reader fallback cascade leaks between tests (persistent cache) | LOW | MED | `ConfigCache` is per-request, not static. A test harness that constructs its own cache won't see other tests' state. |
| Advisor drift: task 54's "tokio::spawn from server.rs" vs Lemmy's clokwerk convention | MED | LOW | Plan resolves to clokwerk; commit message flags the deviation; advisor-side ADR fixup or updated narrative is a downstream concern. Behaviourally identical. |
| Decision-queue #11/#12 get "answered" during 5a out of helpfulness | LOW | MED | Task 56 explicitly leaves them untouched; they're Phase 5b decisions. Hold-hand instruction in the plan's §12.7. |
| Upstream Lemmy 1.0-beta rebase lands with `person` schema change, breaking task 51 | LOW | HIGH | Task 0 runs `cargo check --workspace` to catch any baseline break; if a rebase lands mid-5a, `/prp-debug` is the escape hatch per IMPLEMENTATION-PLAN-v0.md §7.1. |
| `PersonInsertForm` struct-update in task 51 collides with a pending upstream field add | LOW | MED | `TODO(brehon-fork): upstream this to LemmyNet/lemmy — PR #___` marker makes the diff obvious on rebase. |

---

## §17. Notes + decision-queue pre-seed

### 17.1 Open questions logged (non-blocking)

- **OQ-006 threshold formula activation** — 5a seeds the 5 config keys in task 50; 5b task 58 removes `V0_THRESHOLD` / `V0_REPORTER_WEIGHT` in `create_report.rs` and wires the formula. 5a preserves the Phase 4 e2e test by not touching `create_report.rs` behaviour — only the threshold rescale (units change, not semantics).

- **Task 54 scheduler location drift** — advisor narrative says `tokio::spawn` from `server.rs`; this plan says clokwerk in `scheduled_tasks.rs`. Resolved in favour of Lemmy convention; documented in the task 54 commit and completion report. If the advisor prefers the original, a 5b or 5c correction commit can move it — behaviourally identical.

- **Seeded row count = 34, not 32** as the advisor narrative says. Grew from 33 to 34 after Perplexity-review 2026-04-17 added `job.snapshot_batch_chunk_size = 500` for task 54's chunked batch. Documented as GOTCHA-50f; the parity test enforces the real count.

- **`CreateEndorsementResponse` DTO field shape** — 5a adds `endorsement_id: EndorsementId, surety_created: bool`. `EndorsementId` is already exported from `crates/db_schema/src/newtypes.rs:269` (verified at plan-write time). No carry-patch needed. `ReputationEventInsertForm`, `SuretyInsertForm`, `EndorsementInsertForm` all exist under `crates/db_schema/src/source/governance/`.

### 17.2 Decision-queue pre-seeds (for 5b to resolve)

Entries #11 and #12 are already in `.claude/decision-queue.json` from an earlier advisor pre-seed; both are resolved in the design-doc ADRs and do not need 5a action. Do not touch.

Add a new entry #13 at task 56 commit time (after the full phase lands) flagging the `admin-config-write.sh` wrapper script: **#13: Should the admin-config-write.sh wrapper ship in 5c or defer to v1 along with OQ-018's HTTP endpoint?** — options: (a) ship wrapper in 5c as an interim mitigation for Watch 11; (b) defer to v1 when the HTTP endpoint arrives. Context: 5a's `capability_changed` emitter attributes cascades *transitively* via a prior `admin_config_changed` entry if the wrapper is used — without it, admins editing config via raw psql leave no action-time attribution. Left to advisor.

#### Phase 5b carry-forward requirements from Perplexity-review 2026-04-17

These two items target **Phase 5b tasks (56 and 58), not Phase 5a**. Recording here so the 5b plan author incorporates them at 5b plan-write time — they are NOT applied to this plan's body because doing so would violate the "do NOT generate 5b or 5c plans" constraint. Copy-paste verbatim into the corresponding Phase 5b plan tasks when written.

**Carry-forward (1) — Phase 5b Task 56 `ALTER TYPE sanction_action ADD VALUE 'Restoration'` migration must be `no-transaction` (MANDATORY).**

Phase 5b task 56 adds a new variant to the Postgres `sanction_action` enum via `ALTER TYPE sanction_action ADD VALUE 'Restoration'`. Postgres rules:
- In Postgres < 12, `ALTER TYPE ... ADD VALUE` is forbidden inside a transaction block.
- In Postgres 12+, it is allowed, but the newly-added variant **cannot be used in the same transaction that adds it** (the catalog update needs to commit before other transactions can see the value).

Diesel's migration runner wraps migrations in a transaction by default. The migration must opt out via a `-- no-transaction` directive at the top of `up.sql` (and symmetrically in `down.sql` if the reverse path drops the value — which Postgres does not support without rebuilding the enum, so the down is likely a no-op or a full rebuild).

**Requirements for Phase 5b Task 56**:
- Migration file naming: the `ALTER TYPE` migration lives in its own migration directory, separate from any migration that *uses* the new variant. Even if both ship in 5b, they are sequential migration files, not co-located in one file. Suggested naming: `migrations/{ts}_add_restoration_sanction_variant/up.sql` (ALTER TYPE only, `-- no-transaction`), followed by any subsequent migration that inserts / queries rows with `SanctionAction::Restoration`.
- First line of that `up.sql`:
  ```sql
  -- no-transaction
  ALTER TYPE sanction_action ADD VALUE IF NOT EXISTS 'Restoration';
  ```
- `ALTER TYPE ... ADD VALUE IF NOT EXISTS` is the idempotent form — safe across `diesel migration redo`.
- **GOTCHA for the 5b plan**: name the `-- no-transaction` requirement explicitly; cite the Postgres catalog-commit constraint; and include the reverse-path caveat (Postgres cannot drop an enum value; down.sql is either a no-op or a full enum rebuild, with advisor guidance required).
- **5b DoD assertion**: `head -1 crates/db_schema/migrations/{timestamp}_add_restoration_sanction_variant/up.sql` returns `-- no-transaction` exactly; and the migration applies cleanly via `lemmy_diesel_utils::schema_setup::run` in a fresh test container.

**Carry-forward (3) — Phase 5b Task 58 threshold-formula float-to-integer safety (MANDATORY).**

Phase 5b task 58 computes `weight_micros: i64 = (base_weight * reporter_reputation * recency_factor * 1_000_000.0) as i64`. On stable Rust, `f64 as i64` is a saturating cast: `NaN` maps to `0`, `+inf` to `i64::MAX`, `-inf` to `i64::MIN`. Saturation is defined behaviour but the result is meaningless to downstream logic — e.g. if an admin sets `report.recency_half_life_hours = 0.0`, `exp(-hours/half_life) = exp(-inf) = 0.0` (OK) but `exp(-hours/0.0)` is `inf / 0.0` which is `NaN`, and the cast silently produces `0` → the threshold is effectively always met / never met depending on direction.

**Requirements for Phase 5b Task 58**:
- Wrap the float computation with an `is_finite()` guard:
  ```rust
  let weight_f64 = base_weight * reporter_reputation * recency_factor * 1_000_000.0;
  let weight_micros: i64 = if weight_f64.is_finite() {
    weight_f64 as i64
  } else {
    tracing::error!(
      base_weight, reporter_reputation, recency_factor,
      "report-weight calculation produced non-finite value; falling back to base_weight × 1_000_000. \
       Likely cause: an admin-edited config key producing Inf/NaN (e.g. recency_half_life_hours = 0, \
       or a negative base_weight combined with an odd-exponent pow).",
    );
    (base_weight * 1_000_000.0) as i64
  };
  ```
- **GOTCHA for the 5b plan**: name `is_finite()` explicitly; name the fallback semantics (`base_weight × 1_000_000`); require the `error!` log with structured fields so an operator reading the log knows which config key is suspect.
- **5b test requirement (add to task 58 validation or 5b task 60 e2e)**: a unit test that sets `config.report.recency_half_life_hours = 0.0` (or uses a mocked `ConfigCache` returning 0.0), calls the threshold computation, and asserts:
  1. The fallback value (`base_weight × 1_000_000`) is the result, not `0` or `i64::MAX`.
  2. An `error!` log is emitted (capture via `tracing-subscriber::fmt::test` or `tracing::subscriber::with_default` + a custom layer — Lemmy's test helpers may already have a pattern; search at 5b plan-write time).
  3. The handler still returns `Ok(...)` — a non-finite weight does NOT error the request; it logs and proceeds with the fallback. This matters because an admin misconfiguring `recency_half_life_hours` should not take down `POST /report`.

**Rationale for recording here, not inlining**: Items (1) and (3) are Phase 5b task-body changes. This plan's constraint is "do NOT generate Phase 5b or 5c plans — those come after 5a merges." The author of the 5b plan — advisor or a future `/prp-plan` invocation — is the correct site for the GOTCHAs and DoD rows. Having the full verbatim text here means the 5b author doesn't have to re-derive; they copy-paste.

### 17.3 What the completion report must include

- Exit codes for all §14 levels — **use the log tail, not the task-notification summary**. Plan-write-time observation resolved as follows:
  - **Wrapper exit-code propagation**: verified correct. Foreground `cmd //c scripts\brehon\cargo-clippy.bat ... -- -D warnings` exits `101` when cargo exits `101`, and `0` when cargo exits `0`. Both `cargo-check.bat` and `cargo-clippy.bat` end with `cargo.exe <subcommand> %*` as the final statement (lines 20 of each file); Windows batch inherits the final command's exit code. **No wrapper fix needed.**
  - **Background-task notification summary bug**: the Bash tool's `run_in_background` mode reported `exit code 0` in the `<task-notification>` summary when the underlying cmd actually returned `101` — observed for the identical clippy invocation pre-carry-patch. This is a notification-layer bug, not a wrapper bug. The lesson for the impl agent: **never trust a `<task-notification>` `exit code` summary for DoD validation**. Either run DoD commands in the foreground, or read the log tail for `error:` / `warning: build failed` / `Finished` markers and ignore the notification summary. `.claude/rules/cargo-output-capture.md` covers the pipe-masking-exit variant; consider a sibling rule or an amendment to cover the background-notification variant (advisor discretion — could be a 5c or 5b followup).

- **Carry-patch inventory entry** — the pagination.rs fix above is the first carry-patch of Phase 5a. No `scripts/brehon/CARRY-PATCHES.md` exists in the fork. Recommendation (completion report discussion):
  - **Short-term (5a or 5b)**: create `scripts/brehon/CARRY-PATCHES.md` listing every TODO(brehon-fork) marker with file:line + description + upstream-PR-target state. Task 0-level effort; could land as a sibling of the carry-patch commit in any phase. The file already implicit-exists via grep: `grep -rn 'TODO(brehon-fork)' crates/ scripts/` returns the pre-5a set the fork carries. Enumerate.
  - **Homeserver-side**: advisor should update `homeserver/.claude/memory/feedback_carry_patch_todos.md` to add the pagination.rs entry and (if adopting the short-term rec) the path to `scripts/brehon/CARRY-PATCHES.md` as the authoritative inventory. Not fork-side work.

- Commit SHAs for the pre-5a carry-patch + tasks 50–55 + task 56 report.
- Deviations from IMPLEMENTATION-PLAN-v0.md §3 Phase 5a (at least: task 54 scheduler location choice, seed count 34 vs 32 after Perplexity-review chunk-size addition, pre-5a clippy carry-patch required, cargo check `--no-deps` drop, task 54 chunked-batch semantics).
- Any new decision-queue entries opened.
- Carry-forward risks into 5b (prominent — 5b reads 5a's config and snapshot).
- One-line retro nomination per advisor rule 12 (full retro if a checkpoint fires).
