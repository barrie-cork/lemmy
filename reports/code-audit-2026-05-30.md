# Code Audit Report — brehon-fork (governance crates)

**Date:** 2026-05-30
**Scope:** `crates/api/api/src/governance/`, `crates/api/api_crud/src/governance/`, `crates/apub/activities/src/governance/`, `crates/db_schema/src/source/governance/`
**Files scanned:** 84 governance `.rs` files (~20,927 lines governance-custom code; ~81,564 total across `crates/`)
**Serena:** not configured
**Overall health:** **Fair** — strong safety discipline (zero `.unwrap()` in most files, exhaustive matches, explicit transaction boundaries, good module comments). Primary issues are *size-driven complexity* in a handful of critical handlers, not structural rot.

---

## Cheap Metrics Summary

| # | File | Lines | Fns | Longest fn | Max nesting | Unwraps | Composite |
|---|------|-------|-----|------------|-------------|---------|-----------|
| 1 | `config.rs` | 4,216 | 21 | ~3,336 | 4 | 0 | **9.8** |
| 2 | `submit_jury_vote.rs` | 1,140 | 6 | ~684 | 6 | 0 | **7.4** |
| 3 | `admin_assign_jury.rs` | 1,216 | 12 | ~376 | 4 | 0 | **6.6** |
| 4 | `reputation_snapshot.rs` | 1,187 | 37 | ~192 | 5 | 0 | **5.8** |
| 5 | `admin_config.rs` | 1,372 | 38 | ~141 | 4 | 3 | **5.7** |
| 6 | `sponsor_liability.rs` | 579 | 7 | ~363 | 5 | 0 | **5.3** |
| 7 | `create_endorsement.rs` | 415 | 5 | ~242 | 6 | 0 | **5.2** |
| 8 | `admin_audit_stream.rs` | 254 | 3 | ~16 | 7 | 0 | **5.1** |
| 9 | `admin_emergency_remove.rs` | 477 | 6 | ~234 | 4 | 0 | **4.9** |
| 10 | `participation_cron.rs` | 383 | 2 | ~159 | 5 | 0 | **4.7** |

*Composite = 0.20×(lines/500) + 0.25×(longest_fn/100) + 0.20×(nesting/6) + 0.25×(violations/20) + 0.10×(fn_count/30), capped per factor, ×10*

---

## Deep Analysis — Top Files

### 1. `config.rs` (composite: 9.8) — `crates/api/api/src/governance/config.rs`

**Nature:** Config reader + const registry. The file is in two conceptually separate parts: (A) the runtime reader (lines 1–807, ~800 lines) which is well-structured, and (B) the const-default section (lines 808–4,216, ~3,400 lines) which is a flat list of `pub const DEFAULT_*` values followed by four match-dispatch functions (`const_default_int`, `const_default_float`, `const_default_bool`, `const_default_text`).

**Findings:**

| # | Technique | Location | Description | Score | Risk |
|---|-----------|----------|-------------|-------|------|
| 1.1 | Extract Method | `const_default_int` L1050–1200 | 150-arm match dispatch for int defaults. Each new sub-phase adds 5–27 arms. Arms will hit ~300 by v1 end. Consider splitting into per-sub-phase modules with a chained fallback: `phase_v0_defaults(key).or_else(|| phase_v1_ad_defaults(key))` etc. No behaviour change; same const values. | 7.2 | Low |
| 1.2 | Extract Method | Const block L808–1049 | `pub const DEFAULT_*` block for 50+ constants occupies 240 lines and grows with every sub-phase. These could live in sub-phase-scoped submodules (`mod v0_defaults`, `mod v1_ad_defaults`) with a `pub use` re-export so call sites don't change. **Not urgent** — the current layout is explicit and grep-friendly; only worth splitting once the file crosses ~5k lines. | 4.5 | None |
| 1.3 | Simplify Conditional | `get_int_cascade` / `get_float_cascade` L403–533 | The cascade logic (`try scope.key.s0.s1`, then `.s0`, then the base key) is repeated verbatim for int and float. A generic `get_cascade<T, F>` helper with a typed-accessor closure would halve the duplication. Rust's `async fn` in trait limitations make this mildly awkward, but `Box<dyn Future>` or an enum-dispatch approach is feasible. | 6.0 | Medium |

**Linter highlights:** Zero `.unwrap()` in the runtime reader. The three unwraps in `admin_config.rs` not here.

---

### 2. `submit_jury_vote.rs` (composite: 7.4) — `crates/api/api/src/governance/submit_jury_vote.rs`

**Nature:** The most complex handler in the codebase. `process_vote` is ~684 lines implementing a 9-step transactional protocol with multiple sub-branches. File also contains `process_appeal_vote` (appeal panel path).

**Findings:**

| # | Technique | Location | Description | Score | Risk |
|---|-----------|----------|-------------|-------|------|
| 2.1 | Extract Method | `process_vote` L162–700 | The SL-d grace-window branch (steps 8.5–8.9, ~70 lines) handles sponsor-liability path-kind computation, pseudonym assembly, and case-status dispatch. This block is a single logical operation that can be extracted as `determine_sld_path(conn, case_row, action, &mut cache)` returning a `SLDPath` enum. Reduces `process_vote` by ~70 lines with zero behaviour change. | 7.5 | Low |
| 2.2 | Extract Method | `process_vote` step 7 + 7.5 (L317–415) | The threshold tally loop + deadlock branch is ~100 lines. Extract as `tally_votes(conn, case_id, threshold_count) -> LemmyResult<TallyResult>` where `TallyResult` is an enum `{Winner(JuryDecision, Vec<String>), Deadlock, Partial}`. The deadlock governance_log write stays in `process_vote` (it needs `juror_pseudonym`), but the pure tally math separates cleanly. | 7.0 | Low |
| 2.3 | Extract Method | `process_vote` step 9 + appeal path duplication | Reputation-event writes for jurors (aligned/outlier logic) follow the same pattern in both `process_vote` and `process_appeal_vote`. Extract as `emit_juror_reputation_events(conn, case_id, tally, winning_decision, &mut cache)`. Currently repeated with minor variation between the two functions. | 6.5 | Low |
| 2.4 | Parameter Object | `process_vote` signature | `process_vote(conn, juror_id, juror_pseudonym, data, context)` — the `juror_id` + `juror_pseudonym` pair always travel together. A `JurorContext { id: PersonId, pseudonym: String }` newtype would make it harder to accidentally swap or omit one. Pattern already established at `SponsorDelta` in `sponsor_liability.rs`. | 5.0 | None |

---

### 3. `admin_assign_jury.rs` (composite: 6.6) — `crates/api/api/src/governance/admin_assign_jury.rs`

**Nature:** Admin handler for seating a jury panel. `process_assignment` is ~376 lines; `select_eligible_jurors` is a separate 200-line function with a 3-phase diversity-aware algorithm including SQL queries and loop-based relaxation.

**Findings:**

| # | Technique | Location | Description | Score | Risk |
|---|-----------|----------|-------------|-------|------|
| 3.1 | Extract Method | `select_eligible_jurors` constraint-relaxation loop | The R1/R2/R3 relaxation cascade (checking `cluster_pressure`, `cooldown`, `small_pool`) spans ~100 lines inside `select_eligible_jurors`. Extract as `apply_constraint_relaxation(conn, case, pool_size, max_retries, &mut cache) -> LemmyResult<RelaxationResult>`. The outer function becomes a read-SQL + relaxation-dispatch. | 6.5 | Low |
| 3.2 | Guard Clause | `process_assignment` step 2 status guard | The exhaustive status match at L122–135 is correct per ADR-013 and should NOT be simplified to a guard clause — the compile-time exhaustiveness is a design constraint. **No change recommended here.** | N/A | N/A |

---

### 4. `reputation_snapshot.rs` (composite: 5.8) — `crates/api/api/src/governance/reputation_snapshot.rs`

**Nature:** Recomputes reputation snapshots per-person per-community. 37 functions (many small). Max function length ~192 lines.

**Findings:**

| # | Technique | Location | Description | Score | Risk |
|---|-----------|----------|-------------|-------|------|
| 4.1 | Remove Duplicates | Config reads for decay dimensions | The pattern `config::get_float(&mut cache, pool, Scope::Instance, "decay.<dim>.*_half_life_days")` appears for 4+ dimensions (jury_reliability, reporting_accuracy, endorsement_strength, participation_consistency). A helper `fn decay_config(cache, pool, dim) -> LemmyResult<DecayParams>` that reads all half-life keys for one dimension in one call would reduce the volume without changing behaviour. | 6.0 | Low |
| 4.2 | Rename | `raw_*` local variable names | Several functions use `raw_val` or `raw_result` where `unclamped_val` / `pre_clamp_val` would express the invariant (this value is about to be clamped). Minor but aids debugging when reading governance-log deltas. | 3.5 | None |

---

### 5. `admin_config.rs` (composite: 5.7) — `crates/api/api/src/governance/admin_config.rs`

**Nature:** Admin CRUD for `governance_config` rows. 38 functions.

**Findings:**

| # | Technique | Location | Description | Score | Risk |
|---|-----------|----------|-------------|-------|------|
| 5.1 | Guard Clause | Validation functions in `admin_config.rs` | Three `.unwrap()` calls present (the only three across all governance files). All are on `Regex::new(pattern)` with compile-time string literals — these cannot fail at runtime and are a documented Rust idiom. **Not a risk**, but could use `expect("static regex")` for clearer panic messages. | 3.0 | None |
| 5.2 | Simplify Conditional | Validation dispatch by `value_type` | The validation dispatch for `Int`/`Float`/`Bool`/`Text`/`Enum` is repeated in both the write handler and in the metadata lookup. A `validate_value(meta: &ConfigKeyMetadata, raw: &str) -> LemmyResult<()>` helper would centralise type-checked validation. Currently ~60 lines duplicated across 2 functions. | 5.5 | Low |

---

### 6. `create_endorsement.rs` (composite: 5.2) — `crates/api/api_crud/src/governance/create_endorsement.rs`

**Nature:** Endorsement handler with 6-arm `SponsorGateStrategy` dispatch. Max nesting depth 6.

**Findings:**

| # | Technique | Location | Description | Score | Risk |
|---|-----------|----------|-------------|-------|------|
| 6.1 | Guard Clause | Gate-strategy dispatch in `process_endorsement` | The `SponsorGateStrategy::Closed` arm returns early; the remaining arms (`Age`, `AgeOrSurety`, `Reputation`, `Allowlist`, `Open`) each run the same endorsement-write block after different pre-checks. Converting `Closed` to a guard at function top frees one level of nesting for the remaining arms. | 5.5 | Low |
| 6.2 | Extract Method | Age + surety pre-check (~60 lines) | The `AgeOrSurety` strategy checks both the age gate and the surety cap. This block is the longest single arm and could be extracted as `check_age_or_surety_gate(conn, sponsor_id, community_id, &mut cache)`. | 5.0 | Low |

---

### 7. `admin_audit_stream.rs` (composite: 5.1) — `crates/api/api/src/governance/admin_audit_stream.rs`

**Nature:** SSE handler for real-time governance event streaming via Postgres LISTEN.

**Findings:**

| # | Technique | Location | Description | Score | Risk |
|---|-----------|----------|-------------|-------|------|
| 7.1 | Extract Method | `poll_fn` driver closure L143–166 | The notification-routing closure inside `tokio::spawn` is 24 lines with a `loop { match ... }` and 4 branches. Extract as `fn make_notification_driver(pg_conn: C, tx: Sender) -> impl Future<Output=()>`. Makes the `admin_audit_stream` function body shorter and the driver logic independently testable. | 5.0 | Low |
| 7.2 | Extract Method | SSE stream body L182–245 | The `stream!` macro body is ~63 lines handling heartbeat + channel receive + filter + hydration + projection + yield. The hydration step (DB fetch by `entry_id`) could be extracted as `async fn hydrate_log_entry(pool, entry_id) -> Option<AuditEntry>`. The stream body becomes a straightforward select! loop over two futures. | 6.0 | Low |
| 7.3 | Note: High nesting is inherent | L144–165 (`poll_fn` + nested `match` arms) | The nesting depth 7 is driven by the `poll_fn` async poll protocol — `Poll::Ready(Some(Ok(AsyncMessage::Notification(n))))` is 4 levels of nested enums, unavoidable with the tokio-postgres 0.7 API. Not an actionable simplification. | N/A | N/A |

---

### 8. `participation_cron.rs` (composite: 4.7) — `crates/api/api/src/governance/participation_cron.rs`

**Findings:**

| # | Technique | Location | Description | Score | Risk |
|---|-----------|----------|-------------|-------|------|
| 8.1 | Remove Duplicates | Config-read + clamp + warn pattern | The pattern `let raw = config::get_int(...).await.unwrap_or(DEFAULT_*); let clamped = raw.max(1); if raw < 1 { warn!(...) }` appears 3 times at the top of `run_activity_batch`. Extract as `fn clamped_config_int(cache, pool, key, default, min, name) -> LemmyResult<i64>` that reads, clamps, and warns. | 5.5 | Low |

---

## Refactoring Task Queue

| Priority | Task description | Target | Technique | Risk |
|----------|-----------------|--------|-----------|------|
| P1 | Refactor config.rs: split const_default_int into per-phase helpers. Follow .claude/skills/code-refactor/SKILL.md | `config.rs` | Simplify Conditional | Low |
| P1 | Refactor submit_jury_vote.rs: extract SLD grace-window branch. Follow .claude/skills/code-refactor/SKILL.md | `submit_jury_vote.rs` | Extract Method | Low |
| P2 | Refactor submit_jury_vote.rs: extract tally_votes. Follow .claude/skills/code-refactor/SKILL.md | `submit_jury_vote.rs` | Extract Method | Low |
| P2 | Refactor admin_assign_jury.rs: extract constraint-relaxation loop. Follow .claude/skills/code-refactor/SKILL.md | `admin_assign_jury.rs` | Extract Method | Low |
| P2 | Refactor config.rs: extract generic get_cascade helper. Follow .claude/skills/code-refactor/SKILL.md | `config.rs` | Simplify Conditional | Medium |
| P3 | Refactor admin_config.rs: centralise validate_value dispatch. Follow .claude/skills/code-refactor/SKILL.md | `admin_config.rs` | Simplify Conditional | Low |
| P3 | Refactor participation_cron.rs: extract clamped_config_int helper. Follow .claude/skills/code-refactor/SKILL.md | `participation_cron.rs` | Remove Duplicates | Low |
| P3 | Refactor admin_audit_stream.rs: extract SSE hydration step. Follow .claude/skills/code-refactor/SKILL.md | `admin_audit_stream.rs` | Extract Method | Low |
| P3 | Refactor create_endorsement.rs: add Closed guard clause. Follow .claude/skills/code-refactor/SKILL.md | `create_endorsement.rs` | Guard Clause | Low |
| P3 | Refactor submit_jury_vote.rs: extract juror reputation event writes. Follow .claude/skills/code-refactor/SKILL.md | `submit_jury_vote.rs` | Remove Duplicates | Low |

---

## Summary

- **Total findings:** 18 (across 8 files)
- **By risk:** None: 3, Low: 13, Medium: 2
- **By technique:** Extract Method: 9, Simplify Conditional: 4, Remove Duplicates: 3, Guard Clause: 2, Rename: 1, Parameter Object: 1
- **Dead code flagged:** 0 (Serena not available; no manual dead-code sweep performed)
- **Systemic patterns:**

  1. **Large transaction closures** — `process_vote`, `process_assignment`, `process_endorsement` all use the `run_transaction(async |conn| { ... })` pattern with 300–680 line bodies. The pattern is intentional (correct transaction boundaries) but all three would benefit from sub-function extraction within the transaction closure. The constraint is that Rust's borrow checker requires the sub-functions to accept `&mut AsyncPgConnection` explicitly — this is fine but slightly verbose.

  2. **Config const sprawl** — `config.rs` grows by 5–27 arms per sub-phase. This is by design (every seeded key has a matching const; the parity test enforces it) but the file has already passed the comfortable-single-file threshold at 4,216 lines. A module split is the right response, not a change to the parity mechanism.

  3. **Config read + clamp + warn triplication** — Appears in `participation_cron.rs` and likely in `admin_assign_jury.rs` + `reputation_snapshot.rs`. A utility helper would reduce noise in these cron-batch handlers.

- **What NOT to touch:** The exhaustive `match` on `CaseStatus` in every handler (ADR-013 compile-time enforcement), the `ALL_JURY_DECISIONS` const ordering (single source of truth), the `governance_log::append` call sites (registry discipline), and the `run_transaction` boundaries (CRITICAL-TRANSACTION-BOUNDARY plan directives). These are intentional structural choices, not refactor candidates.
