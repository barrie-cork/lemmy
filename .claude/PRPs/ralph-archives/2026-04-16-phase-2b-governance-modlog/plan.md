# Plan: Phase 2b — `governance_modlog` crate + Phase 2 smoke tests

## Table of contents

| Section | Line |
|---|---|
| Summary | 47 |
| Source | 53 |
| Problem Statement | 66 |
| Solution Statement | 70 |
| Metadata | 76 |
| 1. Divergences from main plan (tasks 25–30) | 95 |
| 2. Divergences from [04 §4.4] — prose-vs-code drift to log | 109 |
| 2.1 Drift 1 — `decision: Option<JuryDecision>` | 115 |
| 2.2 Drift 2 — `sanction_action: Option<SanctionAction>` | 123 |
| 2.3 Drift 3 — `appealed: bool` | 134 |
| 2.4 Pseudonym watchpoint — canonical shape has NO person field | 144 |
| 3. Phase 2a follow-ups discovered | 156 |
| 4. Flow Design | 168 |
| 5. Mandatory Reading (implementation agent MUST read before task 25) | 226 |
| 6. Patterns to Mirror | 262 |
| 6.1 Cargo.toml skeleton | 266 |
| 6.2 Workspace registrations | 315 |
| 6.3 lib.rs — plain struct, impls module gated | 343 |
| 6.4 impls.rs — tuple row + build_view pattern | 404 |
| 6.5 EXISTS-subquery pattern (reference only) | 511 |
| 6.6 Integration-test fixture pattern | 531 |
| 7. Files to Change | 571 |
| 8. NOT Building (v0 scope limits) | 593 |
| 9. Step-by-Step Tasks (25–30) | 610 |
| 9.1 Task 25 — CREATE `crates/db_views/governance_modlog` skeleton + workspace registration | 614 |
| 9.2 Task 26 — CREATE `GovernanceModlogView` struct | 688 |
| 9.3 Task 27 — IMPLEMENT `list_public_case_log` | 725 |
| 9.4 Task 28 — IMPLEMENT `list_public_case_log_for_community` | 756 |
| 9.5 Task 29 — IMPLEMENT `read_public_case_log_entry` | 809 |
| 9.6 Task 30 — ADD Phase 2 smoke-test trio to `tests/e2e.rs` | 867 |
| 10. Testing Strategy | 969 |
| 11. Validation Commands | 999 |
| 12. Risks and Mitigations | 1074 |
| 13. Acceptance Criteria | 1090 |
| 14. Completion Checklist | 1108 |
| 15. Confidence rationale | 1125 |
| 16. Commit message convention | 1147 |

---

## Summary

Phase 2b ships the third and final Phase 2 view crate — `crates/db_views/governance_modlog` — with the `GovernanceModlogView` struct from [04 §4.4] and the three queries named in [IMPLEMENTATION-PLAN-v0.md §Phase 2] tasks 25–29 (`list_public_case_log`, `list_public_case_log_for_community`, `read_public_case_log_entry`). Task 30 adds the smoke-test trio that gates all three Phase 2 view crates (`governance_case` + `jury_queue` from 2a + `governance_modlog` from 2b) against a real Postgres container, matching the [§Phase 2 task 30] specification. No migrations, no schema changes, no API layer, no new Diesel source types — Phase 2b is pure read-model code on top of the Phase 1 + 2a foundation. Every file mirrors the Phase 2a `governance_case` + `jury_queue` conventions verified on-disk at `governance-v0` HEAD `67744b4b5` (ancestor of Phase 2a merge HEAD `f9fc612dd`). The phase lands on a feature branch `feature/phase-2b-governance-modlog` cut from `governance-v0`, with one commit per task (6 task commits: 25, 26, 27, 28, 29, 30) plus one optional pre-phase branch-prep commit if the audit surfaces wrapper drift.

---

## Source

- [IMPLEMENTATION-PLAN-v0.md §Phase 2, tasks 25–30](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) — authoritative task bodies
- [04 §4.4](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) — canonical `GovernanceModlogView` shape
- [04 §3 `PublicCaseLog`](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) — source-table Diesel model listing (ground-truth cross-check)
- [05 §3 + §4 Step 2](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) — v0 simplifications; modlog is the public-face transparency surface
- ADRs: **ADR-008** (append-only signed governance log — modlog is the separate **public** view, not the same as `governance_log`), **ADR-015** (GDPR pseudonyms — every string that lands in `public_case_log.summary` / `rationale_redacted` is already redacted at Phase 4 write-time), **ADR-010** (solo-dev stack, no external signer), **ADR-013** (`CaseStatus::EmergencyRemove` exhaustive matches)
- Phase 2a reference crates (existence proofs of the plain-struct + tuple-load + build pattern):
  - `crates/db_views/governance_case/` — end-to-end template with both summary and detail shapes
  - `crates/db_views/jury_queue/` — simpler single-struct template

---

## Problem Statement

Phase 4's `GET /api/v4/governance/modlog` handler (task 43 in the main plan) needs a read model that can paginate the public, redacted moderation log across all communities and per-community, plus fetch a single entry by id. Without this crate, the Phase 4 handler has nowhere to call into, and without the three queries, the Phase 4 transparency surface cannot be wired. Phase 2 as a whole cannot close until all three view crates are shipped and the smoke-test trio demonstrates each one returns real rows against a seeded database, gating the Phase 3 DTO work that depends on these shapes.

## Solution Statement

Create one workspace member under `crates/db_views/governance_modlog/` following the Phase 2a `governance_case` + `jury_queue` convention verbatim: `Cargo.toml` with workspace inheritance + `full`/`ts-rs` feature gates, `src/lib.rs` holding the plain `GovernanceModlogView` struct (NOT a Diesel `Queryable`/`Selectable` — drift stubs forbid the derive per rule `view-crate-selectable-template.md`), and `src/impls.rs` (gated `#[cfg(feature = "full")]`) with three query functions plus one `build_view` mapper. Queries return `LemmyResult<Vec<GovernanceModlogView>>` / `LemmyResult<Option<GovernanceModlogView>>` and take `pool: &mut DbPool<'_>`. No pagination wrapper — Phase 4 handlers will layer pagination on top, matching the `governance_case` precedent. Task 30 extends `crates/server/tests/e2e.rs` with three new `#[tokio::test]` functions that seed the Phase 1 tables via direct Diesel inserts, call the three Phase 2 view crate queries (one per crate), and assert shape against known-good row counts. `phase1_migrations_round_trip` remains untouched and its `.limit(6)` constant is not disturbed (watchpoint 2 from brief).

---

## Metadata

| Field | Value |
|---|---|
| Type | READ_MODEL + INTEGRATION_TESTS |
| Complexity | LOW-MEDIUM (new crate but single struct, three simple queries; test seeding is the only new mechanical work) |
| Crates Affected | `db_views/governance_modlog` (new), workspace `Cargo.toml`, `crates/server` (e2e tests only — no crate code) |
| v0 Step | Step 2 of [05 §4](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) |
| Dependencies | Phase 1 (source tables + migrations), Phase 2a (pattern reference only — no code dependency) |
| Tasks | 6 (task numbers 25–30 from the main plan) |
| Branch | `feature/phase-2b-governance-modlog` (cut from `governance-v0` at HEAD `67744b4b5`, ancestor of Phase 2a merge HEAD `f9fc612dd`) |
| Baseline form | ancestry-check only: `git merge-base --is-ancestor 94eba51a0 HEAD` (Phase 1 tip must be ancestor); do NOT pin a specific hash — per `feedback_plan_baseline_self_reference.md` |
| Iteration Budget | `--max-iterations 15` for `/prp-ralph` |
| Soft cap per task | 4 iterations — stop-and-diagnose if exceeded |
| Target context budget | <150k tokens for the whole phase (Phase 2a was 241k; we have new context-budget rules to tighten the back half) |
| Confidence | **8.5/10** — see §15 Confidence rationale |

---

## 1. Divergences from main plan ([IMPLEMENTATION-PLAN-v0.md §Phase 2, tasks 25–30])

Three structural corrections and one simplification, surfaced from reading the on-disk state of the `public_case_log` source table, the `joinable!` declarations, and the Phase 2a reference crates. None change scope.

1. **Plain struct, NOT `Queryable`/`Selectable`.** The main plan body at task 26 says "create the struct". Per mandatory rule `view-crate-selectable-template.md`, any view struct with kind-(c) fields (bare scalars with no source-column and no `select_expression`) **cannot derive `Selectable`** — the Diesel derive macro tries to resolve a schema table from the struct name, which fails with an `unresolved import` error. `GovernanceModlogView` has three such fields (`decision`, `sanction_action`, `appealed` — see §2 drift). The struct is plain, the impls module builds each view by mapping an explicit Diesel tuple select. This is the same shape used by all three Phase 2a Selectable-incompatible structs (`GovernanceCaseSummaryView`, `GovernanceCaseDetailRow`, `JuryQueueView`) — verified at `crates/db_views/governance_case/src/lib.rs:36-59` and `crates/db_views/jury_queue/src/lib.rs:20-43`.

2. **Query return shape — `Vec<View>` and `Option<View>`, not `PagedResponse`.** Mirroring Phase 2a: the three queries return `LemmyResult<Vec<GovernanceModlogView>>` for the list queries and `LemmyResult<Option<GovernanceModlogView>>` for the single-entry read (using `.first(...).await.optional()?`). Phase 4 handlers will wrap in pagination. The main plan's task 27 says "paginated, all communities" but pagination is a presentation concern belonging to the handler, not the read-model. This matches the Phase 2a decision in its §1 divergence 2.

3. **Task 30 scope: three test functions, NOT one.** The main plan body at task 30 says "Smoke tests for each view — `list_open_cases_returns_seeded_rows`, `jury_queue_view_returns_assignments`, `modlog_view_returns_published_entries`. Each test seeds the DB via direct Diesel inserts (not via API), then queries through the view, asserts shape." Phase 2b implements all three test functions. Two of them (`list_open_cases_returns_seeded_rows`, `jury_queue_view_returns_assignments`) exercise Phase 2a view crates that already exist on-disk — Phase 2b does NOT modify those crates, only consumes them from the test. If any Phase 2a view surfaces a real bug under smoke-test pressure, the fix lands as a separate commit in Phase 2b's commit chain but is NOT bundled with the test commit (watchpoint 4 from brief — no polish-under-cover-of-fix).

4. **`public_case_log` is already joinable to `community` and `moderation_case`.** The schema file at `crates/db_schema_file/src/schema.rs:1289-1290` declares `diesel::joinable!(public_case_log -> community (community_id));` and `diesel::joinable!(public_case_log -> moderation_case (case_id));`. No `.on(...)` pattern needed for the primary join (unlike Phase 2a where `moderation_case -> community` needed explicit `.on()`). This simplifies the query shape versus Phase 2a.

---

## 2. Divergences from [04 §4.4] — prose-vs-code drift to log

Three fields in [04 §4.4] reference data that **do not exist on `public_case_log`** — the table at `crates/db_schema_file/src/schema.rs:1034-1042` has only six columns: `id`, `case_id`, `community_id`, `summary`, `rationale_redacted`, `published_at`. The canonical view shape at [04 §4.4] lines 490-501 adds five more: `community_name`, `decision: Option<JuryDecision>`, `sanction_action: Option<SanctionAction>`, `appealed: bool`. `community_name` is a clean join on `community.name` (already declared joinable). The other three are the drift points.

**These are NOT Phase 1 or 2a bugs — they are prose-vs-code drift inside [04 §4.4].** Code wins. The correct move per ADR discipline is: compute each field at query time from Phase 1 state if feasible, return a drift stub if not, and log the drift for homeserver-side correction of [04 §4.4]. Phase 2a already established this pattern and shipped three similar drifts (`reporter_count`, `jury_needed`/`jury_submitted`, `deadline_at`).

### 2.1 Drift 1 — `GovernanceModlogView.decision: Option<JuryDecision>`

- **Spec location:** [04 §4.4](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) line 495
- **Problem:** `public_case_log` has no `decision` column. The winning decision lives on `jury_vote.decision` (multiple rows — one per juror) and must be tallied per [04 §8] simple-majority rules. Phase 2b would either need to (a) implement the tally in the view crate, or (b) let Phase 4 materialize a denormalised copy at `public_case_log` write time (which would require a schema migration — out of scope per brief).
- **Phase 2b resolution:** **Return `None`** as a stub in the `build_view` mapper. Document with `// [04 §4.4] drift — decision has no source in Phase 1 schema; Phase 4 will either materialise via a new column or compute via jury_vote tally.` Computing a simple-majority in-query via `GROUP BY decision ORDER BY COUNT(*) DESC LIMIT 1` would work but belongs in Phase 4 handler logic where the tally rules also drive sanction creation — putting it in the view duplicates the tally in two places.
- **Why stub and not in-query tally in Phase 2b:** Task 42 (Phase 4 `submit_jury_vote`) is the single source of truth for the tally per [04 §8]. Duplicating it in the view crate means any tuning (supermajority for critical sanctions in v1) needs two edits. The stub keeps the source of truth in one place.
- **Homeserver-side action (out of this repo):** [04 §4.4] should either (a) mark `decision` as "derived from jury_vote tally at handler layer" or (b) add `decision: Option<JuryDecision>` as a denormalised column on `public_case_log` in a new Phase 1 follow-up migration. Advisor to decide.

### 2.2 Drift 2 — `GovernanceModlogView.sanction_action: Option<SanctionAction>`

- **Spec location:** [04 §4.4](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) line 496
- **Problem:** `public_case_log` has no `sanction_action` column. The `sanction` table at `schema.rs:1120-1132` has the action column, but `sanction` has no `joinable!` declaration to `public_case_log` — they connect transitively through `moderation_case` (both have `case_id` → `moderation_case.id`, declared at `schema.rs:1290` and `schema.rs:1304`). Per [04 §8], v0 creates **one sanction row per decided case** (simple-majority rule; no supermajority). A `LEFT JOIN sanction ON sanction.case_id = public_case_log.case_id` is therefore 1:0-or-1 and safe.
- **Phase 2b resolution:** **Return `None`** as a stub in the `build_view` mapper, matching Drift 1. A `LEFT JOIN` would work technically, but:
  1. Phase 2b has no `sanction` rows yet — Phase 1 migrations create the table but nothing writes to it until Phase 4 task 42
  2. The test data seeded in task 30's `modlog_view_returns_published_entries` does NOT create a sanction row (only a `public_case_log` row) to keep the seed minimal, so a LEFT JOIN would return `None` for the test row anyway
  3. Adding the join in Phase 2b means this view crate ships two code paths (join-based and stub-based) for a single field, which is premature optimization
- **Phase 4 cleanup path:** Phase 4 task 42 can either (a) extend this query function with the LEFT JOIN when it writes the first real `sanction` rows, or (b) Phase 4 opens a new plan file for a schema migration that denormalises `sanction_action` onto `public_case_log`. Either path works; the Phase 2b stub is a clean zero-cost placeholder.
- **Homeserver-side action:** Same as Drift 1 — clarify in [04 §4.4] whether derived or denormalised.

### 2.3 Drift 3 — `GovernanceModlogView.appealed: bool`

- **Spec location:** [04 §4.4](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) line 500
- **Problem:** `public_case_log` has no `appealed` column. `appeal` table at `schema.rs:126-134` has `case_id` as a foreign key into `moderation_case`, and `appeal -> moderation_case` is declared joinable at `schema.rs:1212`. Checking "is this case appealed" is feasible via `EXISTS (SELECT 1 FROM appeal WHERE appeal.case_id = public_case_log.case_id)`. This is the ONE drift field where in-query computation is cheap and correct.
- **Phase 2b resolution:** **Compute via an `EXISTS` subquery** in the three list/read queries. Two options:
  - **Option A (chosen):** Post-query aggregation, same pattern as Phase 2a's `submitted_counts_by_case` helper. Load the tuple of scalar columns first, then a second round-trip that fetches `case_id`s with at least one appeal row, build a `HashSet<ModerationCaseId>`, set `appealed = set.contains(&row.case_id)` in `build_view`. Keeps each query mechanically simple and avoids Diesel `exists(...)` expression-type fragility in the select list.
  - **Option B (rejected):** Inline `diesel::dsl::exists(...)` in the select list via `sql::<Bool>("EXISTS ...")`. Rejected because Phase 2a's `jury_queue::list_available_jury_cases_for_person` demonstrates that inline `exists(...)` in a `.filter(...)` works, but it was never tested in a `.select(...)` list on this branch — the fragility risk is untested, and Phase 2a explicitly favoured the two-round-trip pattern for exactly this class of issue (Phase 2a plan §1 divergence 3).
- **Why Option A wins:** precedent + simplicity + avoids expression-type experiments under a time budget.
- **Homeserver-side action:** None needed — `appealed` is fully derivable from Phase 1 schema and the Phase 2b implementation matches [04 §4.4] at the value level.

### 2.4 Pseudonym watchpoint — canonical shape has NO person field

**Watchpoint from brief:** "`governance_modlog` must return pseudonyms, never raw `person_id`. If your plan has `actor_person_id: i32` in the view struct, that's wrong. Correct shape: `pseudonym_id: String` from a join against `actor_pseudonym`."

**On-disk verification:** `GovernanceModlogView` per [04 §4.4] lines 490-501 has **no person_id or pseudonym_id field at all**. The canonical shape is: `case_id, community_id, community_name, decision, sanction_action, summary, published_at, appealed` — purely case-scoped, community-scoped, and decision-scoped. There is no actor exposure in the view.

**Where ADR-015 applies:** The `summary: String` and `rationale_redacted: Option<String>` fields on `public_case_log` are the only strings that could carry leaked identifiers. Per [IMPLEMENTATION-PLAN-v0.md §4.2] and the redaction service contract in [04 §3 ActorPseudonym rules], these strings **must be scrubbed at write-time** (Phase 4 task 42 wraps the `public_case_log` insert in `redaction::scrub(...)`). Phase 2b just reads what's already in the column — no join against `actor_pseudonym`, no re-scrubbing at read time.

**Resolution and drift flag for homeserver:** The brief watchpoint appears to conflate the **governance log** (ADR-008 / task 5 — has `actor_pseudonym` column) with the **public case log** (task 1 migration — no actor column, only case/community/summary). These are two separate tables with different redaction strategies: governance log stores pseudonyms directly, public case log redacts identifiers out of the `summary` string before insert. Phase 2b implements the canonical [04 §4.4] shape — **no person/pseudonym field added** — and flags this to the advisor to either (a) confirm the canonical shape is correct, or (b) amend [04 §4.4] + the brief watchpoint in sync. Advisor to decide when this plan is reviewed. **If the advisor wants a pseudonym field, this plan must be rewritten** — it's not a silent fix during implementation.

---

## 3. Phase 2a follow-ups discovered

**Zero structural issues, one minor observation.**

- Phase 2a's three view structs (`GovernanceCaseSummaryView`, `GovernanceCaseDetailRow`, `JuryQueueView`) all follow the plain-struct + tuple-load + build pattern cleanly. The `build_summary` and `build_view` mappers at `crates/db_views/governance_case/src/impls.rs:191-214` and `crates/db_views/jury_queue/src/impls.rs:30-41` are the existence proofs Phase 2b mirrors directly.
- The Phase 2a `SummaryRow` tuple type is declared at **module scope** with a `type` alias (`crates/db_views/governance_case/src/impls.rs:26-35`). This is required because workspace clippy lints deny `items-after-statements` (declaring a type inside a function after other statements) and `type_complexity` (tuples with >3 components). **Phase 2b must do the same** — declare `ModlogRow` at module scope with a `type` alias, not inside the query functions. Rule `view-crate-selectable-template.md` codifies this.
- `lemmy_db_schema_file::schema::appeal` / `::community` / `::moderation_case` / `::public_case_log` are all available under `feature = "full"` and are the imports the impls module needs. Phase 2a imports them via `lemmy_db_schema_file::schema::{...}` (see `crates/db_views/governance_case/src/impls.rs:12-16`). Phase 2b imports `public_case_log, community, appeal` by the same path.

**No Phase 2a bug fixes are bundled into Phase 2b.** If task 30's smoke tests surface a genuine bug in `governance_case` or `jury_queue`, it lands as a SEPARATE commit in the Phase 2b chain with a commit message like `fix(db_views): phase 2a regression — <summary>` and an explicit note in the commit body that it is a bug fix, not a task-30 test change. If no bug surfaces, no Phase 2a files are touched.

---

## 4. Flow Design

Phase 2b is backend-only. The "flow" is: one more read-model crate slotted into the Phase 2 dependency chain that Phase 4 handlers (future) will consume.

### Before state (at `governance-v0` HEAD `67744b4b5`, ancestor `f9fc612dd`)

```
╔════════════════════════════════════════════════════════════════════════════╗
║   PHASE 2B BASELINE (Phase 2a merged, PDF commit sits on top)              ║
╠════════════════════════════════════════════════════════════════════════════╣
║                                                                            ║
║   crates/db_views/governance_case     — shipped in Phase 2a                ║
║   crates/db_views/jury_queue          — shipped in Phase 2a                ║
║   crates/db_views/governance_modlog   — DOES NOT EXIST                     ║
║                                                                            ║
║   crates/server/tests/e2e.rs          — Phase 1 tests only (4 tests):      ║
║     postgres_container_boots                                               ║
║     can_insert_moderation_case                                             ║
║     governance_log_hash_chain_holds                                        ║
║     phase1_migrations_round_trip                                           ║
║                                                                            ║
║   CONSUMER: none yet — Phase 3 + 4 not started                             ║
║                                                                            ║
╚════════════════════════════════════════════════════════════════════════════╝
```

### After state (Phase 2b HEAD, 6 task commits ahead of `governance-v0`)

```
╔════════════════════════════════════════════════════════════════════════════╗
║                    PHASE 2B HEAD (6 commits ahead)                         ║
╠════════════════════════════════════════════════════════════════════════════╣
║                                                                            ║
║   crates/db_views/governance_modlog:                                       ║
║     pub struct GovernanceModlogView {...}  (plain struct, 8 fields)        ║
║     #[cfg(feature="full")] mod impls:                                      ║
║       fn list_public_case_log(pool)              -> Vec<ModlogView>        ║
║       fn list_public_case_log_for_community(                               ║
║         pool, CommunityId)                       -> Vec<ModlogView>        ║
║       fn read_public_case_log_entry(                                       ║
║         pool, PublicCaseLogId)                   -> Option<ModlogView>     ║
║                                                                            ║
║   crates/server/tests/e2e.rs — 3 new tests (7 total):                      ║
║     list_open_cases_returns_seeded_rows   (gates governance_case)          ║
║     jury_queue_view_returns_assignments   (gates jury_queue)               ║
║     modlog_view_returns_published_entries (gates governance_modlog)        ║
║                                                                            ║
║   CONSUMER: Phase 3 (api_common DTOs) and Phase 4 (GET /modlog handler)    ║
║                                                                            ║
╚════════════════════════════════════════════════════════════════════════════╝
```

### Endpoint changes

None. Phase 2b adds no HTTP routes. Phase 4 wires `GET /api/v4/governance/modlog` on top of these queries.

---

## 5. Mandatory Reading (implementation agent MUST read before task 25)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `docs/brehon-law-inspired-network/04-data-model-and-api.md` | 488-506 | Canonical `GovernanceModlogView` shape + query list |
| P0 | `docs/brehon-law-inspired-network/04-data-model-and-api.md` | 350-358 | `PublicCaseLog` source struct — exact fields that back the view |
| P0 | `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` | 204-216 | Task 25–30 bodies |
| P0 | `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` | 403-411 | Cross-cutting §4.2 — redaction service contract (explains why view crates don't re-scrub) |
| P0 | `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` | 256-270 | ADR-015 (GDPR pseudonyms — governance_log vs public_case_log distinction) |
| P0 | `crates/db_views/governance_case/src/lib.rs` | 1-94 | Phase 2a existence proof — lib.rs shape, plain struct, gated impls mod |
| P0 | `crates/db_views/governance_case/src/impls.rs` | 1-214 | Phase 2a existence proof — tuple row + HashMap aggregation + `build_summary` mapper (task 30 test seeding references this file heavily) |
| P0 | `crates/db_views/jury_queue/src/lib.rs` | 1-43 | Phase 2a single-struct existence proof |
| P0 | `crates/db_views/jury_queue/src/impls.rs` | 1-149 | Phase 2a simpler query pattern + `build_view` mapper |
| P0 | `crates/db_views/governance_case/Cargo.toml` | 1-42 | Exact Cargo.toml template for a Phase 2 view crate |
| P0 | `crates/db_views/jury_queue/Cargo.toml` | 1-42 | Second template (identical shape — confirms the pattern) |
| P0 | `crates/db_schema/src/source/governance/public_case_log.rs` | 1-34 | `PublicCaseLog` source struct + `PublicCaseLogInsertForm` — used by task 30 seeding |
| P0 | `crates/db_schema/src/source/governance/appeal.rs` | 1-35 | `Appeal` source struct — drift 3 EXISTS subquery target |
| P0 | `crates/db_schema_file/src/schema.rs` | 1034-1042 | `public_case_log` table binding |
| P0 | `crates/db_schema_file/src/schema.rs` | 126-134 | `appeal` table binding |
| P0 | `crates/db_schema_file/src/schema.rs` | 1289-1290 | `public_case_log -> community` and `public_case_log -> moderation_case` joinable declarations |
| P0 | `crates/db_schema_file/src/schema.rs` | 1212-1213 | `appeal -> moderation_case` joinable declaration |
| P0 | `crates/db_schema/src/newtypes.rs` | grep `PublicCaseLogId` + `CommunityId` + `ModerationCaseId` | Typed IDs for query signatures |
| P0 | `crates/server/tests/e2e.rs` | 1-444 | Phase 1 test harness — `governance_fixtures::apply_all_schema`, `start_postgres`, `db_url`. Task 30 tests MUST reuse these helpers |
| P0 | `Cargo.toml` (workspace root) | 27-68, 130-145 | `[workspace] members` list (alphabetical insertion point) + `[workspace.dependencies]` path-dep format |
| P0 | `.claude/rules/cargo-output-capture.md` | all 69 | Capture every cargo invocation to a log file; pipes mask exit codes |
| P0 | `.claude/rules/no-cargo-output-paste.md` | all 70 | Never dump full cargo logs into conversation; `tail -20` from the log file |
| P0 | `.claude/rules/view-crate-selectable-template.md` | all 128 | Mandatory rule — plain struct + tuple load for Selectable-incompatible shapes |
| P0 | `.claude/rules/pre-phase-harness-audit.md` | all 145 | Mandatory pre-phase audit — runs at task 0 before task 25 |
| P1 | `crates/db_views/modlog/src/lib.rs` | 1-49 | Upstream Lemmy modlog shape (DIFFERENT crate — the upstream Lemmy modlog, NOT governance_modlog). Reference only for the upstream feature-gate template; do NOT copy field shapes |
| P1 | `crates/db_views/modlog/src/impls.rs` | 1-177 | Upstream pattern reference — use Phase 2a governance_case + jury_queue over this if there's any conflict |
| P1 | `.claude/PRPs/plans/phase-2a-governance-case-jury-queue.plan.md` | all | Phase 2a plan — exact template this plan follows |

**External documentation:** none new. Phase 2b introduces no new crate dependencies beyond what Phase 2a's `governance_case` and `jury_queue` already pull in transitively.

---

## 6. Patterns to Mirror

Every snippet below is copied verbatim from the workspace at HEAD `67744b4b5`. **Do not invent alternatives.** The reference file + line is in a header comment so the implementation agent can verify each snippet is current.

### 6.1 Cargo.toml skeleton

```toml
# SOURCE: crates/db_views/governance_case/Cargo.toml:1-42
# NEW FILE: crates/db_views/governance_modlog/Cargo.toml

[package]
name = "lemmy_db_views_governance_modlog"
version.workspace = true
edition.workspace = true
description.workspace = true
license.workspace = true
homepage.workspace = true
documentation.workspace = true
repository.workspace = true
rust-version.workspace = true

[lib]
doctest = false

[lints]
workspace = true

[features]
full = [
  "lemmy_utils",
  "diesel",
  "diesel-async",
  "i-love-jesus",
  "lemmy_db_schema/full",
  "lemmy_db_schema_file/full",
  "lemmy_diesel_utils/full",
]
ts-rs = ["dep:ts-rs", "lemmy_db_schema/ts-rs", "lemmy_db_schema_file/ts-rs"]

[dependencies]
lemmy_db_schema = { workspace = true }
lemmy_utils = { workspace = true, optional = true }
lemmy_db_schema_file = { workspace = true }
lemmy_diesel_utils = { workspace = true }
diesel = { workspace = true, optional = true }
diesel-async = { workspace = true, optional = true }
serde = { workspace = true }
serde_with = { workspace = true }
chrono = { workspace = true }
ts-rs = { workspace = true, optional = true }
i-love-jesus = { workspace = true, optional = true }
```

### 6.2 Workspace registrations

```toml
# EDIT: Cargo.toml (workspace root)
# SOURCE OF TRUTH: Cargo.toml:27-68, 130-145 at HEAD 67744b4b5

# --- members list (alphabetical within db_views/) ---
# Insert "crates/db_views/governance_modlog" between "governance_case" and "jury_queue"
# at Cargo.toml:48-49 (current state shown below):
members = [
  # ... other crates ...
  "crates/db_views/modlog",
  "crates/db_views/governance_case",
  "crates/db_views/governance_modlog",   # ← NEW, insert here
  "crates/db_views/jury_queue",
  # ... other crates ...
]

# --- workspace dependencies path list (alphabetical) ---
# Insert between "lemmy_db_views_governance_case" (line 138) and
# "lemmy_db_views_jury_queue" (line 139):
lemmy_db_views_governance_case = { version = "=1.0.0-test-arm-qemu.0", path = "./crates/db_views/governance_case" }
lemmy_db_views_governance_modlog = { version = "=1.0.0-test-arm-qemu.0", path = "./crates/db_views/governance_modlog" }  # ← NEW
lemmy_db_views_jury_queue = { version = "=1.0.0-test-arm-qemu.0", path = "./crates/db_views/jury_queue" }
```

**Gotcha:** Both insertions must happen in the SAME commit as the crate's `Cargo.toml` creation (task 25). Per `feedback_commit_hygiene_lockfiles_and_task_labels.md`, the workspace `Cargo.lock` must be staged alongside any `Cargo.toml` edit. Run `cargo check -p lemmy_db_views_governance_modlog` once (or `cargo metadata`) to force lockfile update, then `git add Cargo.toml Cargo.lock crates/db_views/governance_modlog/Cargo.toml crates/db_views/governance_modlog/src/lib.rs`.

### 6.3 lib.rs — plain struct, impls module gated

```rust
// SOURCE: crates/db_views/jury_queue/src/lib.rs:1-43
// NEW FILE: crates/db_views/governance_modlog/src/lib.rs

//! Read model for the public redacted moderation log.
//!
//! Phase 2b ships one plain-struct view per [04 §4.4]:
//!
//! * [`GovernanceModlogView`] — one row per published `public_case_log`
//!   entry, enriched with community name and (in Phase 2b) drift stubs
//!   for `decision`, `sanction_action`, and `appealed`. The struct is a
//!   plain Rust type (no `Queryable`/`Selectable` derives); impls build
//!   it by mapping an explicit Diesel tuple select plus a second
//!   round-trip that resolves `appealed` via `EXISTS` on `appeal` —
//!   same pattern as `lemmy_db_views_governance_case`'s
//!   `submitted_counts_by_case` helper.
//!
//! **ADR-015 redaction lives elsewhere.** Every string that lands in
//! `public_case_log.summary` / `public_case_log.rationale_redacted` is
//! scrubbed at Phase 4 write-time by the `redaction::scrub` wrapper. This
//! view crate reads the columns as-is — no re-scrubbing at read time, no
//! `actor_pseudonym` join in the canonical shape per [04 §4.4].

use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::{JuryDecision, SanctionAction};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
/// One row of the public redacted moderation log per [04 §4.4].
///
/// Plain Rust struct, NOT a Diesel Queryable. Built by mapping an explicit
/// tuple select + `appealed` aggregation in the impls module — see
/// `impls::list_public_case_log` for the canonical shape.
pub struct GovernanceModlogView {
  pub case_id: i32,
  pub community_id: Option<i32>,
  pub community_name: Option<String>,
  /// [04 §4.4] drift — no source in Phase 1 schema (tally lives in Phase 4
  /// jury_vote aggregation). Always `None` in Phase 2b; see plan §2.1.
  pub decision: Option<JuryDecision>,
  /// [04 §4.4] drift — no source in Phase 1 schema (sanction rows written
  /// by Phase 4 task 42). Always `None` in Phase 2b; see plan §2.2.
  pub sanction_action: Option<SanctionAction>,
  pub summary: String,
  pub published_at: DateTime<Utc>,
  /// Derived via a separate `EXISTS` round-trip in impls — true if any
  /// `appeal` row exists with `appeal.case_id = public_case_log.case_id`.
  /// See plan §2.3.
  pub appealed: bool,
}
```

### 6.4 impls.rs — tuple row + build_view pattern

```rust
// SOURCE: crates/db_views/jury_queue/src/impls.rs:1-93
//         crates/db_views/governance_case/src/impls.rs:26-214 (HashMap pattern)
// NEW FILE: crates/db_views/governance_modlog/src/impls.rs

use crate::GovernanceModlogView;
use chrono::{DateTime, Utc};
use diesel::{
  ExpressionMethods, NullableExpressionMethods, OptionalExtension, QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::{CommunityId, ModerationCaseId, PublicCaseLogId};
use lemmy_db_schema_file::schema::{appeal, community, public_case_log};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::LemmyResult;
use std::collections::HashSet;

/// Main-query row shape for the three modlog queries. Captures every
/// scalar column selected from the `public_case_log` + `community` join.
/// `appealed` is NOT in this tuple — it's resolved via a second round-trip
/// in [`appealed_case_ids`] and merged in [`build_view`].
type ModlogRow = (
  PublicCaseLogId,
  ModerationCaseId,
  Option<CommunityId>,
  Option<String>,
  String,
  DateTime<Utc>,
);

/// Aggregate helper: return the set of `moderation_case.id` values that
/// have at least one `appeal` row. Restricted to the given case-ID list.
/// Returns an empty set when `case_ids` is empty (short-circuits the
/// round-trip so a modlog page with zero rows doesn't spend a query).
///
/// Drift 3 from plan §2.3 — `public_case_log.appealed` has no source
/// column; this helper computes the bool via `EXISTS` on `appeal`.
async fn appealed_case_ids(
  conn: &mut diesel_async::AsyncPgConnection,
  case_ids: &[ModerationCaseId],
) -> LemmyResult<HashSet<ModerationCaseId>> {
  if case_ids.is_empty() {
    return Ok(HashSet::new());
  }
  let rows: Vec<ModerationCaseId> = appeal::table
    .filter(appeal::case_id.eq_any(case_ids))
    .select(appeal::case_id)
    .load::<ModerationCaseId>(conn)
    .await?;
  Ok(rows.into_iter().collect())
}

/// Map a single main-query row + the appealed set into a
/// `GovernanceModlogView`, inlining the plan §2 drift stubs at the same
/// step (`decision = None`, `sanction_action = None`).
fn build_view(
  row: ModlogRow,
  appealed: &HashSet<ModerationCaseId>,
) -> GovernanceModlogView {
  let (_pcl_id, case_id, community_id, community_name, summary, published_at) = row;
  GovernanceModlogView {
    case_id: case_id.0,
    community_id: community_id.map(|c| c.0),
    community_name,
    decision: None,
    sanction_action: None,
    summary,
    published_at,
    appealed: appealed.contains(&case_id),
  }
}

/// All public case log entries, newest-first by `published_at`. Powers
/// the cross-community `GET /api/v4/governance/modlog` endpoint (task 43
/// in Phase 4 — handler layer adds pagination).
///
/// Two round-trips: main join + appealed-case-id set.
pub async fn list_public_case_log(
  pool: &mut DbPool<'_>,
) -> LemmyResult<Vec<GovernanceModlogView>> {
  let conn = &mut get_conn(pool).await?;

  let rows: Vec<ModlogRow> = public_case_log::table
    .left_join(community::table)
    .order_by(public_case_log::published_at.desc())
    .select((
      public_case_log::id,
      public_case_log::case_id,
      public_case_log::community_id,
      community::name.nullable(),
      public_case_log::summary,
      public_case_log::published_at,
    ))
    .load::<ModlogRow>(conn)
    .await?;

  let case_ids: Vec<ModerationCaseId> = rows.iter().map(|r| r.1).collect();
  let appealed = appealed_case_ids(conn, &case_ids).await?;

  Ok(rows.into_iter().map(|r| build_view(r, &appealed)).collect())
}
```

**Note on the `left_join(community::table)`**: because `public_case_log -> community` is declared joinable at `schema.rs:1289`, Diesel can infer the `ON` clause and we don't need the explicit `.on(community::id.nullable().eq(...))` that Phase 2a used for its `moderation_case -> community` join (Phase 2a needed it because that pair lacks a joinable! declaration).

### 6.5 EXISTS-subquery pattern (reference only — Phase 2b uses the two-round-trip variant)

```rust
// SOURCE: crates/db_views/jury_queue/src/impls.rs:122-135
// REFERENCE ONLY — Phase 2b chooses the two-round-trip HashMap/HashSet
// pattern over this inline `exists` approach because the two-round-trip
// shape is proven working in three Phase 2a queries and the inline
// `exists` in a `.select(...)` list (as opposed to a `.filter(...)`) is
// untested on this branch.

use diesel::dsl::{exists, not};

// Example from list_available_jury_cases_for_person:
.filter(not(exists(
  jury_assignment::table
    .filter(jury_assignment::case_id.eq(moderation_case::id))
    .filter(jury_assignment::person_id.eq(person_id)),
)))
```

### 6.6 Integration-test fixture pattern

```rust
// SOURCE: crates/server/tests/e2e.rs:115-156 (can_insert_moderation_case)
//         crates/server/tests/e2e.rs:42-113 (governance_fixtures mod)
// REUSE the governance_fixtures helpers — do NOT rebuild the container
// harness. Each test starts its own container (`start_postgres`),
// applies all migrations (`apply_all_schema`), and tears down via
// container drop. Known-working as of the Phase 1 tests.

#[tokio::test]
async fn list_open_cases_returns_seeded_rows() -> Result<(), Box<dyn Error>> {
  use diesel::{Connection as _, PgConnection, RunQueryDsl};
  use lemmy_db_schema::source::governance::moderation_case::ModerationCaseInsertForm;
  use lemmy_db_schema_file::enums::{CaseSeverity, CaseStatus, CaseTargetType};
  use lemmy_db_schema_file::schema::moderation_case;

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let mut conn = PgConnection::establish(&db_url)?;
  governance_fixtures::apply_all_schema(&mut conn)?;

  // Seed: insert one community (need to be careful here — community
  // creation in Lemmy has FK requirements into person/site/instance that
  // may not be trivially satisfiable. Test may need to seed with
  // community_id = None — see task 30 for the exact seeding contract.)

  // ... then insert a ModerationCase with an open status, and call
  // governance_case::impls::list_open_cases_for_community via a
  // diesel-async DbPool (NOT a sync PgConnection — the governance_case
  // impls take `&mut DbPool<'_>`, not `&mut PgConnection`).

  Ok(())
}
```

**Critical gotcha for task 30**: Phase 2a view crate impls take `&mut DbPool<'_>` (a diesel-async pool), but the existing Phase 1 test at `crates/server/tests/e2e.rs:115-156` uses a synchronous `PgConnection::establish(&db_url)` for the insert step and a sync `diesel::insert_into(...).execute(&mut conn)` for the seed data. Task 30 tests must **build a separate diesel-async DbPool** to drive the view crate calls. See task 30 detailed steps in §9.6 for the exact pattern.

---

## 7. Files to Change

| File | Action | Justification |
|---|---|---|
| `Cargo.toml` (workspace root) | UPDATE | Add `crates/db_views/governance_modlog` to `members` list AND add `lemmy_db_views_governance_modlog` to `[workspace.dependencies]` |
| `Cargo.lock` | UPDATE | Auto-regenerated by cargo during first `cargo check`; must be staged alongside `Cargo.toml` per `feedback_commit_hygiene_lockfiles_and_task_labels.md` |
| `crates/db_views/governance_modlog/Cargo.toml` | CREATE | New crate manifest — copy verbatim from `governance_case/Cargo.toml` with `name` swapped |
| `crates/db_views/governance_modlog/src/lib.rs` | CREATE | Plain `GovernanceModlogView` struct + module-level docs + gated `pub mod impls;` |
| `crates/db_views/governance_modlog/src/impls.rs` | CREATE | `ModlogRow` tuple type, `appealed_case_ids` helper, `build_view` mapper, three query functions |
| `crates/server/tests/e2e.rs` | UPDATE | Add three `#[tokio::test]` functions at the end of the file: `list_open_cases_returns_seeded_rows`, `jury_queue_view_returns_assignments`, `modlog_view_returns_published_entries` |
| `crates/server/Cargo.toml` | UPDATE | Add `lemmy_db_views_governance_modlog` as a `dev-dependencies` entry (alongside the Phase 2a crates if they're already listed there for tests; if not, add all three) |

**Files NOT to change** (watchpoints from brief):
- `migrations/**` — zero migrations in Phase 2b
- `crates/db_schema/**` — zero schema changes
- `crates/db_schema_file/**` — zero enum or table binding changes
- `crates/db_views/governance_case/**` — Phase 2a, do not touch unless a real bug surfaces under smoke-test pressure (in which case: separate commit, see §1 divergence 3)
- `crates/db_views/jury_queue/**` — same as governance_case
- `phase1_migrations_round_trip` (in `e2e.rs`) — do NOT modify, do NOT touch its `PHASE_1_MIGRATION_COUNT: u64 = 6` constant (watchpoint 2)

---

## 8. NOT Building (v0 scope limits)

Explicit non-goals for Phase 2b, in priority order:

- **No pagination** — `LemmyResult<Vec<GovernanceModlogView>>` is the shape; Phase 4 handlers will layer pagination. Deferred per Phase 2a §1 divergence 2 precedent.
- **No real `decision` / `sanction_action` computation** — both return `None` as drift stubs (§2.1, §2.2). Phase 4 decides the materialisation path.
- **No `actor_pseudonym` join** — canonical [04 §4.4] shape has no person field (§2.4). Per ADR-015, redaction of `summary` happens at Phase 4 write-time, not at Phase 2b read-time.
- **No API DTOs** — deferred to Phase 3 (`crates/api/api_common/src/governance.rs`).
- **No HTTP routes** — deferred to Phase 4 task 43.
- **No `impls` unit tests inside the crate** — Phase 2 tests live in `crates/server/tests/e2e.rs` per [IMPLEMENTATION-PLAN-v0.md §5.2]. The rule is "integration-only until something breaks twice."
- **No updates to `phase1_migrations_round_trip`** — Phase 2b adds zero migrations, so the `.limit(6)` is correct unchanged. Watchpoint 2.
- **No Phase 2a touch-ups** — if a Phase 2a view shows a bug under smoke-test pressure, fix in a separate commit with explicit `fix(db_views):` prefix; no polish-under-cover-of-fix (watchpoint 4).
- **No Extism plugin hooks** — governance hooks for `before_modlog_read` / `after_modlog_read` are a v1+ concern; Phase 2b is direct Rust per [99 ADR-012] "use Extism where it simplifies governance hooks" and Phase 2 has no hook surface.
- **No v1/v2/v3 scope** — `deploy/` directory, Keycloak, OPA/OpenFGA, external signer, blockchain anchoring all deferred per [99 ADR-010].

---

## 9. Step-by-Step Tasks (25–30)

Execute in order. One commit per task. Each task has a MIRROR reference, exact file paths, a validation command captured to a log file, and an expected exit behavior. Task 25's first step is the pre-phase audit — if any audit probe fails, stop and escalate per rule `pre-phase-harness-audit.md`.

### 9.1 Task 25 — CREATE `crates/db_views/governance_modlog` skeleton + workspace registration

**Step 0 (branch + audit, MUST run before any file edit):**

```bash
# Baseline ancestry — do NOT pin a specific hash per feedback_plan_baseline_self_reference.md
# Assumes impl session is already on feature/phase-2b-governance-modlog (plan commit sits on
# the branch tip). governance-v0 has advanced to e3f335a78 (CI commit) and the feature branch
# has been rebased onto that tip — see "load-bearing prerequisite" advisor note.
git branch --show-current  # Must print feature/phase-2b-governance-modlog
git merge-base --is-ancestor 94eba51a0 HEAD || { echo "Phase 1 tip 94eba51a0 is not an ancestor — STOP"; exit 1; }
git status --short  # Must show a clean tree (no untracked, no modified). The earlier .github/*
                    # untracked state was committed upstream as e3f335a78 before Phase 2b started.
```

**Step 1 (pre-phase audit per `pre-phase-harness-audit.md`):** Run three wrapper probes before any file edit. Skipping is not permitted.

```bash
# Probe 1 — per-crate check honors -p
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_views_governance_case > .claude/audit-cargo-check-p.log 2>&1"
tail -20 .claude/audit-cargo-check-p.log
# Expect: only lemmy_db_views_governance_case compiles, exit 0

# Probe 2 — feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_views_governance_case --features full > .claude/audit-cargo-check-features.log 2>&1"
tail -20 .claude/audit-cargo-check-features.log
# Expect: compiles with --features full, exit 0

# Probe 3 — cargo-test wrapper honors target selection
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/audit-cargo-test.log 2>&1"
tail -20 .claude/audit-cargo-test.log
# Expect: e2e test binary builds, exit 0
```

If any probe fails, **stop the phase** and escalate to advisor with the log file paths — do not proceed to step 2.

**Step 2 (DoD smoke test per `pre-phase-harness-audit.md`):** Dry-run every validation command that appears in tasks 25–30 DoDs against the current branch tip. Unchanged-crate commands must currently pass; new-crate commands must currently fail with a "crate not found" / "unresolved module" style error (expected-red set, turns green in the task that creates the file).

**Step 3 (CREATE files in this exact order):**

- **Step 3a (directory):** `mkdir -p crates/db_views/governance_modlog/src`
- **Step 3b:** CREATE `crates/db_views/governance_modlog/Cargo.toml` — copy verbatim from §6.1 above (or from `crates/db_views/governance_case/Cargo.toml:1-42` with `name` swapped to `lemmy_db_views_governance_modlog`).
- **Step 3c:** CREATE `crates/db_views/governance_modlog/src/lib.rs` — minimal placeholder content so the crate compiles. Just the module doc comment and an empty body, or the struct-only skeleton without `pub mod impls;` — the full struct + impls land in tasks 26 and 27. Example minimal:
  ```rust
  //! Read model for the public redacted moderation log.
  //! Phase 2b task 25 — crate skeleton only.
  //! Task 26 adds the `GovernanceModlogView` struct; task 27 adds `impls`.
  ```
- **Step 3d:** EDIT `Cargo.toml` (workspace root) — insert the `crates/db_views/governance_modlog` line into `members` (alphabetical, between `governance_case` and `jury_queue`) AND insert `lemmy_db_views_governance_modlog = { ... path = "./crates/db_views/governance_modlog" }` into `[workspace.dependencies]` (same alphabetical position).
- **Step 3e:** EDIT `crates/server/Cargo.toml` — add `lemmy_db_views_governance_modlog` to `dev-dependencies` (alongside `lemmy_db_views_governance_case` and `lemmy_db_views_jury_queue` if they're already there; if not, task 25 adds all three so task 30's tests can call into them). Use `cargo add` or a manual edit — check first by reading the file.

**Step 4 (validation DoD):**

```bash
# Must succeed — the new crate compiles empty
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_views_governance_modlog > .claude/build-task25.log 2>&1"
tail -20 .claude/build-task25.log
# Expect: exit 0. If exit ≠ 0, diagnose from the full log; do NOT paste it into conversation.

# Must succeed — the server test harness still compiles (dev-deps wiring is sound)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features full > .claude/build-task25-server.log 2>&1"
tail -20 .claude/build-task25-server.log
# Expect: exit 0
```

**Step 5 (commit):**

```bash
git add Cargo.toml Cargo.lock crates/db_views/governance_modlog/Cargo.toml \
        crates/db_views/governance_modlog/src/lib.rs crates/server/Cargo.toml
git status --short  # VERIFY: only the above Phase 2b files staged; .github/* still untracked
git commit -m "feat(db_views): task 25 — governance_modlog crate skeleton"
```

### 9.2 Task 26 — CREATE `GovernanceModlogView` struct

**Step 1:** REPLACE the placeholder `crates/db_views/governance_modlog/src/lib.rs` content with the full version from §6.3 above. The full content includes:
- Module-level `//!` doc comment (including the ADR-015 pseudonym explanation)
- Imports: `chrono::{DateTime, Utc}`, `lemmy_db_schema_file::enums::{JuryDecision, SanctionAction}`, `serde::{Deserialize, Serialize}`, `serde_with::skip_serializing_none`
- `#[cfg(feature = "full")] pub mod impls;` — important even though impls is empty (task 27 fills it)
- The `GovernanceModlogView` struct with all 8 fields + drift-stub doc comments
- NO `#[cfg_attr(feature = "full", derive(Queryable, Selectable))]` — plain struct per rule `view-crate-selectable-template.md`

**Step 2 (CREATE empty impls stub so the `pub mod impls;` reference resolves):**

```rust
// crates/db_views/governance_modlog/src/impls.rs
// Task 26 — empty stub; task 27 fills in the query functions.
```

**Step 3 (validation DoD):**

```bash
# Struct compiles without features (no impls module visible)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_views_governance_modlog > .claude/build-task26.log 2>&1"
tail -20 .claude/build-task26.log
# Expect: exit 0

# Struct + empty impls module compiles WITH features
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_views_governance_modlog --features full > .claude/build-task26-full.log 2>&1"
tail -20 .claude/build-task26-full.log
# Expect: exit 0

# Clippy is clean — mandatory --features full and --no-deps per Phase 2a lesson
cmd //c "scripts\\brehon\\cargo-check.bat clippy -p lemmy_db_views_governance_modlog --features full --no-deps -- -D warnings > .claude/build-task26-clippy.log 2>&1"
tail -20 .claude/build-task26-clippy.log
# Expect: exit 0
```

**Step 4 (commit):** `git add crates/db_views/governance_modlog/src/lib.rs crates/db_views/governance_modlog/src/impls.rs && git commit -m "feat(db_views): task 26 — GovernanceModlogView struct"`

### 9.3 Task 27 — IMPLEMENT `list_public_case_log`

**Step 1:** REPLACE `crates/db_views/governance_modlog/src/impls.rs` with the content from §6.4 above. This includes:
- All use statements (`diesel`, `diesel_async`, `lemmy_db_schema::newtypes`, `lemmy_db_schema_file::schema`, `lemmy_diesel_utils::connection`, `lemmy_utils::error`, `std::collections::HashSet`)
- The `ModlogRow` tuple type (module scope — NOT inside a function, per rule `view-crate-selectable-template.md`)
- `appealed_case_ids` helper function
- `build_view` mapper function
- `list_public_case_log` public async function

**Step 2 — gotchas and decisions to watch for:**

- **`left_join(community::table)` without `.on(...)`.** `public_case_log -> community` is declared joinable at `schema.rs:1289`, so Diesel infers the ON clause. This is cleaner than Phase 2a's governance_case explicit `.on()` — verify by comparing against Phase 2a's `jury_queue::impls::list_jury_assignments_for_person:54-55` which uses `.inner_join(moderation_case::table)` without `.on()` (both joinable pairs).
- **`.order_by(public_case_log::published_at.desc())`** — the modlog is newest-first per Lemmy modlog convention.
- **Don't use `Selectable::as_select()`** — the view struct has no `Queryable` derive; select explicit columns into the `ModlogRow` tuple.
- **`public_case_log::community_id` is `Nullable<Int4>`** — the select expression yields `Option<CommunityId>` (via newtype wrapping in Diesel). The `community::name` column is `Text`, not nullable, but because of the LEFT JOIN the select must use `.nullable()` to yield `Option<String>`.
- **`PublicCaseLogId` newtype** — verify at `crates/db_schema/src/newtypes.rs`. If it doesn't exist, use `i32` directly for the id field in `ModlogRow`. Check before writing imports.

**Step 3 (validation DoD):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_views_governance_modlog --features full > .claude/build-task27.log 2>&1"
tail -20 .claude/build-task27.log
# Expect: exit 0

cmd //c "scripts\\brehon\\cargo-check.bat clippy -p lemmy_db_views_governance_modlog --features full --no-deps -- -D warnings > .claude/build-task27-clippy.log 2>&1"
tail -20 .claude/build-task27-clippy.log
# Expect: exit 0
```

**Step 4 (commit):** `git add crates/db_views/governance_modlog/src/impls.rs && git commit -m "feat(db_views): task 27 — list_public_case_log query"`

### 9.4 Task 28 — IMPLEMENT `list_public_case_log_for_community`

**Step 1:** Append a second query function to `crates/db_views/governance_modlog/src/impls.rs`. It's almost identical to `list_public_case_log` but adds a `.filter(public_case_log::community_id.eq(community_id))` between the join and the `.order_by`.

```rust
/// Public case log entries filtered by community, newest-first. Powers
/// the community-scoped `GET /api/v4/governance/modlog?community_id=X`
/// variant (task 43 in Phase 4 — handler layer adds pagination).
///
/// Two round-trips: main join + appealed-case-id set.
pub async fn list_public_case_log_for_community(
  pool: &mut DbPool<'_>,
  community_id: CommunityId,
) -> LemmyResult<Vec<GovernanceModlogView>> {
  let conn = &mut get_conn(pool).await?;

  let rows: Vec<ModlogRow> = public_case_log::table
    .left_join(community::table)
    .filter(public_case_log::community_id.eq(community_id))
    .order_by(public_case_log::published_at.desc())
    .select((
      public_case_log::id,
      public_case_log::case_id,
      public_case_log::community_id,
      community::name.nullable(),
      public_case_log::summary,
      public_case_log::published_at,
    ))
    .load::<ModlogRow>(conn)
    .await?;

  let case_ids: Vec<ModerationCaseId> = rows.iter().map(|r| r.1).collect();
  let appealed = appealed_case_ids(conn, &case_ids).await?;

  Ok(rows.into_iter().map(|r| build_view(r, &appealed)).collect())
}
```

**Step 2 — gotcha:** the `.filter(public_case_log::community_id.eq(community_id))` uses the `CommunityId` newtype. Diesel's `ExpressionMethods::eq` on a `Nullable<Int4>` column accepts `Option<CommunityId>` — if a raw `CommunityId` fails to type-check, wrap it as `Some(community_id)` or use `.eq(community_id.0)` to drop to the primitive type. Verify by reading how Phase 2a's `governance_case::impls::list_open_cases_for_community:64` calls `.filter(moderation_case::community_id.eq(community_id))` — that same pattern is the proof.

**Step 3 (validation DoD):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_views_governance_modlog --features full > .claude/build-task28.log 2>&1"
tail -20 .claude/build-task28.log

cmd //c "scripts\\brehon\\cargo-check.bat clippy -p lemmy_db_views_governance_modlog --features full --no-deps -- -D warnings > .claude/build-task28-clippy.log 2>&1"
tail -20 .claude/build-task28-clippy.log
# Expect: exit 0 on both
```

**Step 4 (commit):** `git add crates/db_views/governance_modlog/src/impls.rs && git commit -m "feat(db_views): task 28 — list_public_case_log_for_community query"`

### 9.5 Task 29 — IMPLEMENT `read_public_case_log_entry`

**Step 1:** Append a third query function. Single-entry read, returns `LemmyResult<Option<GovernanceModlogView>>`.

```rust
/// Read a single `public_case_log` entry by its primary key. Returns
/// `None` if no row with that id exists (via `.first(...).await.optional()?`
/// — the missing-row case is NOT an error). Powers Phase 4 handlers that
/// need to deep-link a single modlog entry.
///
/// Two round-trips when the row exists: main join + appealed case-id
/// lookup. Short-circuits to one round-trip when the row is missing.
pub async fn read_public_case_log_entry(
  pool: &mut DbPool<'_>,
  entry_id: PublicCaseLogId,
) -> LemmyResult<Option<GovernanceModlogView>> {
  let conn = &mut get_conn(pool).await?;

  let row: Option<ModlogRow> = public_case_log::table
    .left_join(community::table)
    .filter(public_case_log::id.eq(entry_id))
    .select((
      public_case_log::id,
      public_case_log::case_id,
      public_case_log::community_id,
      community::name.nullable(),
      public_case_log::summary,
      public_case_log::published_at,
    ))
    .first::<ModlogRow>(conn)
    .await
    .optional()?;

  match row {
    None => Ok(None),
    Some(r) => {
      let appealed = appealed_case_ids(conn, &[r.1]).await?;
      Ok(Some(build_view(r, &appealed)))
    }
  }
}
```

**Step 2 — gotcha:** `PublicCaseLogId` newtype must be imported. If the newtype doesn't exist (check `crates/db_schema/src/newtypes.rs`), use `i32` directly — but the parameter type in the function signature must match whatever other Phase 2a queries use for ID params (verify via `governance_case::impls::read_case_detail:233` which takes `case_id: ModerationCaseId`).

**Step 3 (validation DoD):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_views_governance_modlog --features full > .claude/build-task29.log 2>&1"
tail -20 .claude/build-task29.log

cmd //c "scripts\\brehon\\cargo-check.bat clippy -p lemmy_db_views_governance_modlog --features full --no-deps -- -D warnings > .claude/build-task29-clippy.log 2>&1"
tail -20 .claude/build-task29-clippy.log
# Expect: exit 0 on both
```

**Step 4 (commit):** `git add crates/db_views/governance_modlog/src/impls.rs && git commit -m "feat(db_views): task 29 — read_public_case_log_entry query"`

### 9.6 Task 30 — ADD Phase 2 smoke-test trio to `tests/e2e.rs`

**Scope:** Three new `#[tokio::test]` functions at the end of `crates/server/tests/e2e.rs`, each gating one Phase 2 view crate:

1. `list_open_cases_returns_seeded_rows` — seeds `moderation_case` with at least one row (community_id = None to avoid community FK setup), calls `lemmy_db_views_governance_case::impls::list_open_cases_for_community` OR (alternative that avoids community setup) `list_cases_for_person` or `list_cases_needing_jury_selection`, asserts the returned Vec has the expected length + shape.
2. `jury_queue_view_returns_assignments` — seeds `moderation_case` + `jury_assignment` rows for a fake `PersonId`, calls `lemmy_db_views_jury_queue::impls::list_jury_assignments_for_person`, asserts length + shape.
3. `modlog_view_returns_published_entries` — seeds `moderation_case` + `public_case_log` rows, calls `lemmy_db_views_governance_modlog::impls::list_public_case_log`, asserts length + shape + drift-stub values (`decision = None`, `sanction_action = None`, `appealed = false`).

**The diesel-async / sync pool mismatch — key gotcha.**

The Phase 1 test `can_insert_moderation_case` at `crates/server/tests/e2e.rs:115-156` uses a **synchronous** `PgConnection::establish(&db_url)?` and inserts via sync `diesel::insert_into(...).execute(&mut conn)?`. The Phase 2a view crate impls take `pool: &mut DbPool<'_>` which is a **diesel-async** pool. Task 30 tests must therefore:

- **Option A (chosen):** Seed the data via a sync `PgConnection` (matching the Phase 1 pattern) AND build a separate async `DbPool` for the view-crate calls, pointing at the same Postgres container. Both connections work against the same database, just via different drivers.
- **Option B (rejected):** Convert the Phase 1 seed pattern to async. Rejected because it would touch Phase 1 code (out of scope per watchpoint) and require rewriting the migration harness.

**How to build a diesel-async DbPool for tests:** check how the upstream Lemmy tests build pools. Search `crates/db_views/modlog/src/impls.rs:204-350` for the `build_db_pool_for_tests` helper or its equivalent in `lemmy_diesel_utils::connection`. If no helper exists, the test needs to build a `bb8::Pool` from `diesel_async::pooled_connection::AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(&db_url)`.

**If building an async pool turns out to be fragile:** fall back to bypassing the view crate and asserting via raw SQL against the sync connection (test still demonstrates the seeding round-trips correctly, but doesn't gate the view crate). Flag as a partial task 30 and escalate to advisor. This is a known tripwire from Phase 2a — the implementer hit it and deferred async pool construction to Phase 2b precisely because tests were out of scope then.

**Test shape template (for all three):**

```rust
#[tokio::test]
async fn list_open_cases_returns_seeded_rows() -> Result<(), Box<dyn Error>> {
  use diesel::{Connection as _, PgConnection, RunQueryDsl};
  use lemmy_db_schema::source::governance::moderation_case::ModerationCaseInsertForm;
  use lemmy_db_schema_file::enums::{CaseSeverity, CaseStatus, CaseTargetType};
  use lemmy_db_schema_file::schema::moderation_case;

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);

  // Step 1: apply schema via sync connection (reuses Phase 1 harness)
  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;

    // Step 2: seed — insert one ModerationCase with Open status
    let form = ModerationCaseInsertForm {
      community_id: None,
      creator_id: None,
      target_type: CaseTargetType::RemoteInstance,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: None,
      target_community_id: None,
      target_remote_url: Some("https://example.invalid/post/1".to_string()),
      reason_code: "spam".to_string(),
      severity: CaseSeverity::Low,
      status: CaseStatus::Open,
      threshold_score: 1,
    };
    diesel::insert_into(moderation_case::table)
      .values(&form)
      .execute(&mut sync_conn)?;
  }

  // Step 3: build async pool + call view crate
  // ... (pool construction — investigate lemmy_diesel_utils helpers first)
  // let mut pool = build_test_pool(&db_url).await?;
  // let rows = lemmy_db_views_governance_case::impls::list_cases_needing_jury_selection(&mut pool).await?;
  // Note: community_id = None precludes list_open_cases_for_community (which filters by community);
  // use list_cases_needing_jury_selection after updating the seeded row to ThresholdMet,
  // OR use list_cases_for_person after seeding a target_person_id.

  // Step 4: assert shape (non-empty, at least the drift-stub constants)
  // assert_eq!(rows.len(), 1);
  // assert_eq!(rows[0].jury_needed, 5); // [05 §3] constant
  // assert_eq!(rows[0].reporter_count, 0); // Phase 2a drift 1 stub

  Ok(())
}
```

**Seed-data decisions per test:**

- **`list_open_cases_returns_seeded_rows`**: seed one `moderation_case` with `community_id = None`, status = `ThresholdMet`, call `list_cases_needing_jury_selection(&mut pool)` (not `list_open_cases_for_community`, which needs a community to filter on), assert length = 1 and `jury_needed == 5` and `reporter_count == 0`.
- **`jury_queue_view_returns_assignments`**: seed one `moderation_case` as above, one `jury_assignment` with a fake `person_id` (just a positive i32 — Phase 1 schema may or may not enforce the FK to `person`; if it does, seeding a minimal person row is necessary — check `crates/db_schema_file/src/schema.rs` for `jury_assignment.person_id -> person` constraints). Call `list_jury_assignments_for_person(&mut pool, person_id)`, assert length = 1 and `deadline_at == None` (Phase 2a drift 3 stub).
- **`modlog_view_returns_published_entries`**: seed one `moderation_case`, one `public_case_log` with a `summary: String` (already-redacted value like `"Case summary — no identifiers"`), call `list_public_case_log(&mut pool)`, assert length = 1 and `appealed == false` and `decision == None` and `sanction_action == None`.

**FK gotcha — `jury_assignment.person_id`:** per `schema.rs:460-468`, `jury_assignment.person_id -> Int4` likely has a FK to `person.id`. Seeding `jury_assignment` without a real `person` row will fail with a constraint violation. Two options: (a) seed a minimal `person` row first (requires `instance_id` and `public_key` — non-trivial), or (b) relax the test to skip `jury_queue_view_returns_assignments` and flag as a blocker with advisor. **Recommended path:** try (a) first; if it exceeds ~2 iterations, fall back to (b) and escalate.

**Step N (validation DoD):**

```bash
# Build the test binary first (no run) to catch compile issues
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/build-task30-compile.log 2>&1"
tail -20 .claude/build-task30-compile.log
# Expect: exit 0

# Run the full test binary — expects 7 tests total (4 Phase 1 + 3 new Phase 2b)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/build-task30-run.log 2>&1"
tail -40 .claude/build-task30-run.log
# Expect: exit 0, "7 passed, 0 failed" or "test result: ok. 7 passed"
# CRITICAL: phase1_migrations_round_trip MUST still be in the passing set —
# if its .limit(6) broke, task 30 should FAIL and the test change must be reverted.
```

**Step N+1 (commit):** `git add crates/server/tests/e2e.rs && git commit -m "test(e2e): task 30 — Phase 2 view-crate smoke tests"`

---

## 10. Testing Strategy

Per [IMPLEMENTATION-PLAN-v0.md §5]: **integration-only** for v0, no unit tests until something breaks twice. All tests live in `crates/server/tests/e2e.rs`.

### Tests added by Phase 2b (task 30)

| Test Name | What It Validates | Gates |
|---|---|---|
| `list_open_cases_returns_seeded_rows` | Seeded `moderation_case` in `ThresholdMet` is returned by `list_cases_needing_jury_selection`; `jury_needed == 5`, `reporter_count == 0` (Phase 2a drift stubs) | `governance_case` view crate |
| `jury_queue_view_returns_assignments` | Seeded `jury_assignment` for a fake person is returned by `list_jury_assignments_for_person`; `deadline_at == None` (Phase 2a drift 3 stub) | `jury_queue` view crate |
| `modlog_view_returns_published_entries` | Seeded `public_case_log` is returned by `list_public_case_log`; `decision == None`, `sanction_action == None` (Phase 2b drifts 1+2), `appealed == false` (no appeal row seeded) | `governance_modlog` view crate |

### Tests preserved (Phase 1, must not regress)

| Test Name | Contract |
|---|---|
| `postgres_container_boots` | Container harness still boots |
| `can_insert_moderation_case` | Phase 1 seed path still works; this is the template Phase 2b tests mirror |
| `governance_log_hash_chain_holds` | Hash chain integrity (ADR-008) still holds |
| `phase1_migrations_round_trip` | `PHASE_1_MIGRATION_COUNT: u64 = 6` MUST NOT CHANGE — watchpoint 2 |

### Edge cases documented (not tested in Phase 2b)

- Empty modlog — `list_public_case_log(&mut pool)` returns `Vec::new()` when no rows exist. Covered implicitly by `appealed_case_ids` short-circuiting on empty case-id list.
- Multiple appeals for one case — `appealed_case_ids` collects into a `HashSet<ModerationCaseId>`, which deduplicates automatically. Not tested in Phase 2b because Phase 4 hasn't written `appeal` rows yet.
- Community deleted after the public_case_log row was written — `LEFT JOIN community` yields `community_name: None` but the row still appears. Not tested in Phase 2b.
- ADR-013 `CaseStatus::EmergencyRemove` entries in the modlog — public_case_log doesn't track status, so emergency-remove entries look identical to normal entries. The `summary` text is expected to carry the "emergency removal" indication per [06 §2.2.1]. Phase 4 is responsible for this; Phase 2b reads whatever was written.

---

## 11. Validation Commands

Use these exact commands — do NOT substitute npm/pnpm/anything else. This is a Rust project on Windows with vcpkg libpq, and every long-running command must be captured to a log file per `cargo-output-capture.md`.

### Level 1: PER-CRATE STATIC ANALYSIS

```bash
# Crate compile — every task
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_views_governance_modlog --features full > .claude/build-levelN.log 2>&1"
tail -20 .claude/build-levelN.log
# Expect: exit 0

# Clippy — every task; MUST include --features full and --no-deps per
# Phase 2a lesson. Bare `cargo clippy -p <crate> -- -D warnings` is
# unexecutable on governance crates.
cmd //c "scripts\\brehon\\cargo-check.bat clippy -p lemmy_db_views_governance_modlog --features full --no-deps -- -D warnings > .claude/build-levelN-clippy.log 2>&1"
tail -20 .claude/build-levelN-clippy.log
# Expect: exit 0
```

### Level 2: WORKSPACE CHECK (end of phase)

```bash
# Smoke test — default features. Feature-gated correctness is Level 1's job
# (per-crate clippy with --features full catches the governance-specific bugs).
# Level 2 is a "does the workspace still compile at all" gate.
cmd //c "scripts\\brehon\\cargo-check.bat --workspace > .claude/build-ws.log 2>&1"
tail -20 .claude/build-ws.log
# Expect: exit 0
```

### Level 3: INTEGRATION TESTS (task 30 + end of phase)

```bash
# Build only (faster feedback loop when debugging compile errors)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/build-e2e-compile.log 2>&1"
tail -20 .claude/build-e2e-compile.log
# Expect: exit 0

# Full run — includes Phase 1 tests (must still pass) and Phase 2b tests
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/build-e2e-run.log 2>&1"
tail -40 .claude/build-e2e-run.log
# Expect: exit 0, 7 tests passed (4 Phase 1 + 3 Phase 2b), 0 failed
```

### Level 4: BRANCH-READY CHECK (just before merge)

```bash
# 1. Clean workspace build
cmd //c "scripts\\brehon\\cargo-check.bat --workspace > .claude/build-final-ws.log 2>&1"
tail -20 .claude/build-final-ws.log

# 2. Per-crate clippy on the new crate
cmd //c "scripts\\brehon\\cargo-check.bat clippy -p lemmy_db_views_governance_modlog --features full --no-deps -- -D warnings > .claude/build-final-clippy.log 2>&1"
tail -20 .claude/build-final-clippy.log

# 3. Full test suite
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/build-final-e2e.log 2>&1"
tail -40 .claude/build-final-e2e.log

# 4. Git hygiene — verify only Phase 2b files changed
git diff --stat governance-v0..HEAD
# Expect: only the files listed in §7; no .github/* changes, no migrations/**, no crates/db_schema/**
```

### Level 5: CROSS-CUTTING VERIFICATION

- [ ] `GovernanceModlogView` struct has no `person_id` / `pseudonym_id` / `actor_id` field (canonical [04 §4.4] shape, §2.4 watchpoint resolution)
- [ ] Every match on `CaseStatus` in Phase 2b code — **zero** (Phase 2b's queries don't filter on or pattern-match `CaseStatus`; if a future edit adds one, it MUST handle `EmergencyRemove` and `AdminReview` exhaustively per ADR-013)
- [ ] `impls.rs` never calls `redaction::scrub` — reading from `public_case_log.summary` is read-only; scrubbing happens at Phase 4 write-time
- [ ] `phase1_migrations_round_trip` constant `PHASE_1_MIGRATION_COUNT: u64 = 6` unchanged — `git diff governance-v0..HEAD -- crates/server/tests/e2e.rs | grep PHASE_1_MIGRATION_COUNT` returns nothing
- [ ] `crates/db_schema/**` and `crates/db_schema_file/**` and `migrations/**` unchanged — `git diff --stat governance-v0..HEAD -- crates/db_schema crates/db_schema_file migrations` returns nothing

---

## 12. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `diesel-async` pool construction in tests is fragile (Phase 2a's implementer deferred this to Phase 2b) | MED | MED | Investigate `lemmy_diesel_utils` test helpers first (Phase 2a's impls.rs test patterns show bb8 pool construction in upstream Lemmy). If nothing reusable exists, build a minimal `bb8::Pool` inline in the test. Fall back to bypassing the view crate with raw SQL and flag task 30 as partial |
| `jury_assignment.person_id` FK blocks seeding a test `jury_assignment` without a real `person` row | MED | MED | Seed a minimal `person` with just the required columns; if it takes >2 iterations, drop `jury_queue_view_returns_assignments` and flag advisor — tasks 25–29 still land and the other two smoke tests gate 2/3 crates |
| `community_id = None` path in `list_open_cases_for_community` is a dead query — test must use `list_cases_needing_jury_selection` or `list_cases_for_person` instead | LOW | LOW | Documented in §9.6 seed-data decisions. The brief's task 30 naming (`list_open_cases_returns_seeded_rows`) is the test name, not a requirement to call a specific Phase 2a function |
| `PublicCaseLogId` newtype may not exist in `crates/db_schema/src/newtypes.rs` | LOW | LOW | Check first; fall back to `i32` if absent and update the function signature. Verify by grepping `newtypes.rs` for `PublicCaseLogId` in task 25 audit step |
| `lemmy_db_views_governance_modlog` name collides with upstream Lemmy's `lemmy_db_views_modlog` in developer mental model | LOW | LOW | Doc comment in lib.rs explicitly disambiguates: "governance modlog (public redacted case log), NOT Lemmy's upstream modlog view" |
| Drift stubs for `decision`/`sanction_action` silently become wrong when Phase 4 materialises them | MED | MED | Drift fields have TODO-style doc comments citing plan §2.1 and §2.2 with the Phase 4 cleanup path. Phase 4 plan review will catch if the handler layer expects non-None values and Phase 4 implementer will either update the view or convert the drift to a real join |
| Workspace lockfile churn — `Cargo.lock` re-shuffles transitive versions | LOW | LOW | Stage `Cargo.lock` with `Cargo.toml` in task 25 per `feedback_commit_hygiene_lockfiles_and_task_labels.md`; don't let it bleed across commits |
| Context budget blowout — Phase 2a was 241k tokens, target is <150k | MED | LOW | Task-split into 6 commits (enforces checkpoint boundaries); log-capture rule prevents cargo output bloat; `no-cargo-output-paste.md` prevents re-reading full logs |
| Advisor rewrites task 0 to ancestry form before ralph starts | LOW | LOW | Plan §Metadata baseline uses ancestry form literal — advisor won't need to rewrite. Confirmed alignment with brief question 1 answer |

---

## 13. Acceptance Criteria

- [ ] All six tasks (25–30) committed to `feature/phase-2b-governance-modlog`
- [ ] `cargo check -p lemmy_db_views_governance_modlog --features full` exits 0
- [ ] `cargo clippy -p lemmy_db_views_governance_modlog --features full --no-deps -- -D warnings` exits 0
- [ ] `cargo check --workspace` exits 0
- [ ] `cargo test --test e2e -p lemmy_server` exits 0 with 7 tests passed (4 Phase 1 + 3 Phase 2b)
- [ ] `phase1_migrations_round_trip` still passes unchanged
- [ ] `GovernanceModlogView` struct matches canonical [04 §4.4] shape (8 fields including the three drift stubs)
- [ ] Three drift points explicitly documented with `// [04 §4.4] drift — ...` comments at the field sites
- [ ] Zero changes to `migrations/**`, `crates/db_schema/**`, `crates/db_schema_file/**`
- [ ] Zero changes to Phase 2a crates (`crates/db_views/governance_case/**`, `crates/db_views/jury_queue/**`) — unless a separate `fix(db_views):` commit is justified and approved
- [ ] Zero changes to `.github/**` (left alone per brief question 2 answer)
- [ ] No contradictions with the 15 ADRs in [99](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)
- [ ] Commit message convention `feat(db_views): task N — <summary>` from task 25 onward (test commit uses `test(e2e):` prefix)

---

## 14. Completion Checklist

- [ ] Task 0 (branch + audit) complete
- [ ] Task 25 (crate skeleton + workspace registration) complete, Level 1 passes
- [ ] Task 26 (view struct) complete, Level 1 passes
- [ ] Task 27 (`list_public_case_log`) complete, Level 1 passes
- [ ] Task 28 (`list_public_case_log_for_community`) complete, Level 1 passes
- [ ] Task 29 (`read_public_case_log_entry`) complete, Level 1 passes
- [ ] Task 30 (smoke test trio) complete, Level 3 passes with 7/7 tests green
- [ ] Level 4 branch-ready check passes
- [ ] Level 5 cross-cutting verification passes
- [ ] All Acceptance Criteria ticked
- [ ] Final `git diff --stat governance-v0..HEAD` reviewed against §7 expected file list
- [ ] Merge-back strategy: fast-forward only into `governance-v0` (per brief)

---

## 15. Confidence rationale

**Confidence: 8.5/10** for one-pass implementation success.

**What increases confidence (from 7 baseline to 8.5):**

- **+1.0 Phase 2a existence proofs.** Every pattern in this plan has a literal on-disk reference at `crates/db_views/governance_case/` and `crates/db_views/jury_queue/`. No invention. No "here's how I think Diesel works" — it's "here's how Diesel works in this workspace at this HEAD."
- **+0.5 Auto-loaded rules.** `pre-phase-harness-audit.md`, `view-crate-selectable-template.md`, `cargo-output-capture.md`, `no-cargo-output-paste.md` are all auto-loaded in `-p` mode. The implementer will see them on iteration 1. The Phase 2a cascade (checkpoint-2 wrapper bug) is prevented structurally.
- **+0.5 Smaller scope.** Phase 2a was 11 tasks (14–24), Phase 2b is 6 tasks (25–30). Less surface, fewer places to fail.
- **+0.25 `public_case_log` is pre-joinable to `community` and `moderation_case`.** No explicit `.on()` dance — Phase 2a's `governance_case` had to add explicit joins because `moderation_case -> community` lacked a joinable! declaration. Phase 2b has cleaner join shapes out of the box.
- **+0.25 Drift story is already precedent-set.** Phase 2a shipped three drift stubs (`reporter_count`, `jury_needed`/`jury_submitted`, `deadline_at`) and the pattern is proven. Phase 2b adds two more drift stubs (`decision`, `sanction_action`) and one derived value (`appealed`) — this is well-trodden ground.

**What remains risky (keeps it below 9):**

- **−0.25 Task 30 async pool construction is unproven on this branch.** Phase 2a did not ship integration tests, so nobody has built a diesel-async `DbPool` from a test binary yet. The upstream Lemmy `crates/db_views/modlog` tests have a pattern, but whether it's cleanly reusable from `crates/server/tests/` is unverified.
- **−0.25 `jury_assignment.person_id` FK may block the middle test.** If seeding a minimal `person` row is non-trivial (needs instance_id, public_key), task 30 may partially land — 2/3 smoke tests instead of 3/3. This is flagged as a risk with a known fallback (drop the test, flag to advisor).
- **−0.0 Advisor watchpoint ambiguity on pseudonyms.** The brief watchpoint contradicts the canonical [04 §4.4] shape. The plan surfaces this in §2.4 for advisor decision. If the advisor confirms "use canonical shape," the plan runs as-is; if they say "add pseudonym field," the plan is rewritten (which is WHY it's flagged now, not deferred to review).

**Net:** plan is strong enough to hand to `/prp-ralph` once the advisor confirms the pseudonym question and approves the pre-phase audit. The main failure mode is Task 30's async pool construction, which is isolated to one task and has a documented fallback.

---

## 16. Commit message convention

Per `feedback_commit_hygiene_lockfiles_and_task_labels.md`, all task commits use the prospective format `feat(scope): task N — <summary>`.

| Task | Commit |
|---|---|
| 25 | `feat(db_views): task 25 — governance_modlog crate skeleton` |
| 26 | `feat(db_views): task 26 — GovernanceModlogView struct` |
| 27 | `feat(db_views): task 27 — list_public_case_log query` |
| 28 | `feat(db_views): task 28 — list_public_case_log_for_community query` |
| 29 | `feat(db_views): task 29 — read_public_case_log_entry query` |
| 30 | `test(e2e): task 30 — Phase 2 view-crate smoke tests` |

Optional pre-phase commit (only if the audit surfaces wrapper drift):
- `chore(scripts): fix cargo-check wrapper -p flag discard` (or similar, depending on the bug)

If a Phase 2a bug fix is needed to get task 30's tests green, it lands as its own commit between task 29 and task 30:
- `fix(db_views): phase 2a regression — <summary>` (watchpoint 4: no polish under cover of fix)

---

**End of plan.**
