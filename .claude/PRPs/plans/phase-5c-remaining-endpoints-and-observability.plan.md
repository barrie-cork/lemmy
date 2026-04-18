# Plan: Phase 5c — Remaining endpoints, observability, capability tests

**Plan file.** `.claude/PRPs/plans/phase-5c-remaining-endpoints-and-observability.plan.md`
**Branch.** `phase-5c` — cut from `governance-v0` **after** Phase 5b PR merges (phase-5b tip `1682a544f` at plan-write time — tasks 56–58 shipped; tasks 59 (founder CLI) + 60 (3-branch e2e) + 61 (PR) remain — see §2 blockers).
**Scope.** Tasks 61–69 from `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` §3 Phase 5c **plus** task 69a (V2 messaging hooks) folded in from `.claude/PRPs/plans/phase-5c-task-69a-v2-hooks.fragment.md`. Task 0 (pre-phase audit) + task 70 (phase-close PR) bracket the work. **Total 12 task slots, 10 substantive.**
**`/prp-ralph` iterations.** `--max-iterations 12` (1 audit + 9 substantive + 1 V2 hooks + 1 phase-close); advisor-context rule 6 cap of 10 relaxed by +2 because task 68 and task 69 are both ≥2-branch compound tasks.
**Prior-phase HEAD.** `governance-v0` @ Phase 5b merge commit (TBD — do not start Phase 5c until that tip exists).

---

## §0. Table of contents

| § | Section | Line |
|---|---|---|
| §1 | Summary | 23 |
| §2 | Sources / ADRs / OQs / blockers | 45 |
| §3 | Problem statement | 96 |
| §4 | Solution statement | 120 |
| §5 | Metadata | 145 |
| §6 | Critical conventions (fork-local) | 170 |
| §7 | File tree (new + touched) | 220 |
| §8 | DTO coverage matrix | 260 |
| §9 | Definition-of-done commands (per-task) | 295 |
| §10 | Cross-cutting invariants | 330 |
| §11 | Step-by-step tasks | 370 |
| §11.0 | Task 0 — pre-phase audit + branch cut + decision-queue intake | 372 |
| §11.1 | Task 61 — `get_my_reputation` handler | 410 |
| §11.2 | Task 62 — `admin_reputation_stats` observability | 450 |
| §11.3 | Task 63 — threshold-crossing log wire-up | 495 |
| §11.4 | Task 64 — `accept_jury_assignment` + `admin_assign_jury` flip | 540 |
| §11.5 | Task 65 — `decline_jury_assignment` + replacement pick | 595 |
| §11.6 | Task 66 — `request_appeal` (api_crud) | 645 |
| §11.7 | Task 67 — `list_cases` handler + view-crate filter extension | 695 |
| §11.8 | Task 68 — route registration + `all_mvp_endpoints_return_non_404` smoke | 745 |
| §11.9 | Task 69 — `ineligible_user_cannot_be_picked_for_jury` e2e (3 branches) | 795 |
| §11.10 | Task 69a — V2 messaging hooks (compound: NOTIFY + SUBSCRIPTIONS.md + 2 tests + plan-doc edit) | 855 |
| §11.11 | Task 70 — phase-close validation + report + PR | 920 |
| §12 | Validation commands (Level 0–5) | 955 |
| §13 | Testing strategy | 990 |
| §14 | Risk register | 1005 |
| §15 | Acceptance criteria | 1030 |
| §16 | Plan correction policy | 1050 |
| §17 | Notes + decision-queue intake | 1065 |

---

## §1. Summary

Phase 5c closes the v0 MVP surface: **all 11 endpoints live and routed, observability distribution endpoint added, jury accept/decline/appeal ship with a replacement-pick path, capability-gating regression holds, and V2 messaging hooks reserved (post-v0 door)**. Six handler files land (five in `crates/api/api/src/governance/`, one in `crates/api/api_crud/src/governance/`), one mutation to the Phase 4 `admin_assign_jury.rs:139` hardcode (Accepted → Selected), and one Phase 4 `report_to_modlog_golden_path` test edit (inserting five `accept_jury_assignment` calls between assign and vote). One new migration (Postgres `NOTIFY` trigger on `governance_log`), one new doc (`SUBSCRIPTIONS.md`), four new e2e tests.

All six new DTO request types are **already shipped** in `crates/api/api_common/src/governance.rs` from the Phase 3 bulk-add (see §8 matrix). Phase 5c adds response DTOs (`AcceptJuryAssignmentResponse`, `DeclineJuryAssignmentResponse`, `RequestAppealResponse`, `GetMyReputationResponse`, `AdminReputationStats`, `AdminReputationStatsResponse`).

After 5c merges, Phase 5 is complete. Phase 6 (federation outbound + advisory inbound) is the only remaining phase before v0 ship.

---

## §2. Sources / ADRs / OQs / blockers

**Primary (fork-local vendored):**

- `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` §3 Phase 5c (tasks 61–69, lines 363–401), §4 cross-cutting (hash chain, pseudonyms, redaction, `EmergencyRemove`), and the **"Phase 5 impact on shipped Phase 4 code"** block (lines 404–411).
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` §4.3 (`ReputationSummaryView`), §5 (DTOs — **already shipped**), §6.1 `request_appeal`, §6.2 `list_cases`, `list_modlog`, `accept_jury_assignment`, `decline_jury_assignment`, `get_my_reputation`, §7 route table.
- `docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md` §2 (exactly 11 endpoints — final state), §3 (v0 simplifications — 5-juror panel, quorum 3, simple majority, no endorsement-revoke in MVP), §9 Done-definition rows 1–3 + row 7.
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`:
  - **ADR-010** — staged releases; v1 features explicitly out of 5c (no larger appeal jury, no retroactive jury-revocation on threshold edit).
  - **ADR-013** — `CaseStatus::EmergencyRemove` must be handled exhaustively in every match; no `_ =>`.
  - **ADR-015** — every governance_log write uses `actor_pseudonym`, every user-text string passes through `redaction::scrub_json` via `governance_log::append`.
  - **OQ-004 resolved** — jury concurrent cap = 3 instance-wide (task 69 flips to 5 on its third branch to assert config-driven behaviour).
  - **OQ-008** — direct moderator actions on an in-review case; not opened in 5c (AdminReview variant exists, Phase 5c handlers match it exhaustively but don't create it).
  - **OQ-009** — juror anonymity in decision phase; v1 concern; Phase 5c logs pseudonyms not names (already the existing pattern).
- `docs/brehon-law-inspired-network/06-security-and-threat-model.md` §2.2.1 (EmergencyRemove public-log redaction contract) and §6 (redaction is a single code path).
- `docs/brehon-law-inspired-network/V2/messaging.md` §8 — four V2 hooks task 69a reserves: §8.1 (PM hooks stable — covered by `.claude/rules/pm-plugin-hooks-stable.md`, no 5c code), §8.2 (`@_`-prefix usernames must remain registerable), §8.3 (SSE-over-WS preference — plan-doc annotation), §8.4 (subscribable event stream — `NOTIFY` trigger).

**Phase 5b outputs consumed (must exist before 5c starts):**

- All Phase 5b tasks 56–61 **shipped** as of 2026-04-18 via squash-merge 6566dce43 on governance-v0 (PR #7), plus 4 cherry-picked post-review fixes (a6e6265f9, 0e86dcb4e, 9102abba2, 72fd4be1f) + retro at eaa413cd8. Current governance-v0 tip is f20728323 or later. Decision-queue #17 (branch base) **resolved** — phase-5c cuts from current governance-v0 tip.
- **§2.1 Resolved (was Blocker).** Phase 5b task 59 `seed_founders` CLI shipped at `crates/tools/seed_founders/`. Phase 5c task 69's third branch MAY reuse the CLI helper OR inline a direct `reputation_event` INSERT (both work; inline is cleaner for self-contained test). Verify at task-0 time.
- **§2.2 Resolved (was Blocker).** Phase 5b task 60 `sponsor_liability_with_founder_multiplier` shipped in `crates/server/tests/e2e.rs`. DoD line 396 "still passes (regression)" is evaluable. Task 0 audit re-runs this test as part of the Phase-5b-baseline smoke to confirm current green before 5c mutations.
- **§2.3 Drift.** Phase 5b task 56 landed `SanctionAction::Restoration` as a **unit variant** (per GOTCHA-56b, `enums.rs:553-557`), not `Restoration { description: String }` as IMPLEMENTATION-PLAN-v0.md line 360 originally specified. This is documented in the `enums.rs` comment itself — Phase 5c respects the shipped form. No plan-level action required; mentioned here so 5c handlers matching `SanctionAction` use the `Restoration` unit arm.
- **§2.4 Usable now.** `load_or_compute_snapshot(conn, person_id, community_id, cache)` at `reputation_snapshot.rs:454-481` (Phase 5b task 58) is exactly the "row-existence first, only recompute if missing" helper task 61 needs. No handler signature change required — task 61 is a thin wrapper.
- **§2.5 Risk-reduction strategy applied (2026-04-18).** Decision-queue entries #19 (actix `config` fn name → verified `pub fn config(cfg: &mut ServiceConfig, rate_limit: &RateLimit)` at `crates/api/routes/src/lib.rs:201`), #20 (tokio-postgres 0.7.16 notifications — poll-based not stream-based; task 69a.3 snippet needs rewrite to use `Connection::poll_message` + channel bridge), #21 (DoD line 398 staleness alert landed as task 63d) all answered **before task 0**. Plan sections §11.3, §11.10, §11.0 updated accordingly. See `C:\Users\barri\.claude\plans\what-would-be-a-proud-balloon.md` for the full strategy.

**Advisor context / rules (auto-loaded in `-p` mode):**

- `.claude/rules/phase-branch.md`, `.claude/rules/pre-phase-harness-audit.md`, `.claude/rules/cargo-output-capture.md`, `.claude/rules/no-cargo-output-paste.md`, `.claude/rules/decision-queue.md`, `.claude/rules/gh-pr-fork-target.md`, `.claude/rules/view-crate-selectable-template.md`, `.claude/rules/pm-plugin-hooks-stable.md`.

---

## §3. Problem statement

After Phase 5b, six of the eleven MVP endpoints are live: `POST /report`, `GET /case`, `GET /modlog`, `GET /jury/me`, `POST /jury/vote`, `POST /endorsement`. Five remain unrouted: `GET /cases`, `GET /reputation/me`, `POST /jury/accept`, `POST /jury/decline`, `POST /appeal`. Three additional gaps:

1. **No observability.** An instance admin cannot see the reputation distribution, the threshold crossings that have happened, or capability-count snapshots. The `capability_changed` log entries already emit (Phase 5a task 53, `reputation_snapshot.rs:286-308`) but aren't surfaced — the `GovernanceModlogView` derives from `public_case_log`, not `governance_log`, so entry-kind filtering is structurally absent.

2. **`admin_assign_jury.rs:139` hardcodes `JuryAssignmentStatus::Accepted`** for v0 testability. The accept/decline handshake flow needs this flipped to `Selected` — which in turn requires editing the Phase 4 `report_to_modlog_golden_path` test to call `accept_jury_assignment` between assign and vote. This is the "Phase 5 impact on shipped Phase 4 code" entry in IMPLEMENTATION-PLAN-v0.md lines 406–407.

3. **V2 messaging hooks not reserved.** `V2/messaging.md` §8 lists four hooks that post-v0 depends on: PM plugin stability (already covered by rule, no code), MXID-looking usernames (needs e2e regression), SSE-over-WS preference (needs plan-doc annotation), subscribable governance event stream (needs Postgres `NOTIFY` trigger). If 5c closes without these, V2 either patches core (violates ADR-012 rationale) or polls `governance_log` (adds latency).

Closing these gaps ends Phase 5. Phase 6 (federation) is the only remaining work before v0 ship.

---

## §4. Solution statement

**Architecture fit.** Phase 5c is **pure additive** to the existing crate layout — no new crates, no ADR amendments. Six new handler files in `crates/api/api/src/governance/` (one in `crates/api/api_crud/src/governance/`), one touched file (`admin_assign_jury.rs:139`), one touched test (`e2e.rs::report_to_modlog_golden_path`), one new migration (`add_governance_log_notify`), one new doc, six new DTO response types in `api_common/src/governance.rs`, new route entries in `crates/api/routes/src/lib.rs:505-522` scope block (note: there is **no** separate `crates/api/routes/src/governance.rs` — all routes are inline in `lib.rs`, confirmed by Explore agent #1).

**Dependency order.** Schema → (already done, no migrations beyond the NOTIFY trigger) → DTOs (just add 6 response types to the existing `governance.rs`) → handlers (6 new files + 1 Phase 4 edit) → routes (1 block edit) → e2e tests (4 new tests + 1 Phase 4 test edit). Entry-kind constants for new log kinds land alongside the first handler that uses them.

**What changes, what does not.**

- **Changes:** `admin_assign_jury.rs:139` Accepted → Selected; `e2e.rs::report_to_modlog_golden_path` inserts 5 `accept` calls; route scope block extends by 6 entries; `governance_log.rs` adds 4 new entry-kind constants; `db_views/governance_case/src/impls.rs` extends `list_open_cases_for_community` signature OR adds a new `list_cases_filtered` (task 67 §11.7 decision); `db_views/governance_modlog/src/impls.rs` adds a `list_capability_changed_entries_since` helper reading from `governance_log` directly (task 63 §11.3 decision).
- **Does not change:** no ADR amendments; no schema migrations beyond `add_governance_log_notify` (NOTIFY trigger); no Phase 1 enum variants added; no `config.rs` constants changed (task 69 uses runtime config INSERT, not const edit); no federation code (Phase 6 scope).

**Why this shape over alternatives.**

- **Endorsement-revoke endpoint not shipped** per [05 §2] "no endorsement revoke in MVP". DTO exists (`RevokeEndorsement`), handler deferred to v1. Phase 5c smoke test (task 68) does NOT include `/endorsement/revoke` in its route iteration.
- **Appeal larger-jury flow not shipped** per ADR-010 / [05 §3]. Task 66 inserts an `Appeal` row and flips case `Decided → Appealed`, then the case sits waiting for `admin_close_case` — no automatic re-jury.
- **Retroactive jury-revocation on threshold edit not shipped** per IMPLEMENTATION-PLAN-v0.md line 375 explicit v1 reservation. Task 63's GOTCHA documents this.
- **Observability via aggregate SQL, not Rust-side loop** per IMPLEMENTATION-PLAN-v0.md line 373 GOTCHA. Task 62 uses `CASE WHEN ... END` bucketing in `sql_query`, mirroring the pattern already in `admin_assign_jury.rs:304-322`.

---

## §5. Metadata

| Field | Value |
|---|---|
| Type | HANDLER (5 new) + HANDLER (api_crud, 1 new) + SCHEMA (1 trigger migration) + ROUTE (1 block edit) + TEST (5 new/edited) + CROSS_CUTTING (entry-kind consts) |
| Complexity | MEDIUM-HIGH (task 68 route regression + task 69 three-branch test + task 67 view-crate refactor) |
| Crates Affected | `lemmy_api` (5 new handlers + 1 edit + 4 new constants), `lemmy_api_crud` (1 new handler), `lemmy_api_common` (6 new DTOs), `lemmy_api_routes` (1 scope block edit), `lemmy_db_views_governance_case` (1 fn signature extension or new fn), `lemmy_db_views_governance_modlog` (1 new fn), `lemmy_server` (5 new e2e tests + 1 edited) |
| v0 Step | Step 5 — Reputation & sponsorship (closes Step 5) |
| Dependencies | Phase 5b merged into `governance-v0` (tasks 56–60 all shipped) |
| Estimated Tasks | 12 slots (1 audit + 9 substantive + 1 V2 hooks compound + 1 phase-close) |
| `/prp-ralph` iterations | 12 |
| Expected LOC (per task caps below) | ~1400 total, distributed per §11 task caps |

**Per-task LOC caps** (for ralph-loop prompt budget; compound tasks count all branches):

| Task | Cap | Reason |
|---|---|---|
| 0 | 0 | Audit only — no code |
| 61 | 80 | Single handler, thin wrapper around `load_or_compute_snapshot` |
| 62 | 180 | 1 SQL query, 1 Rust aggregator, 1 handler, 1 response DTO |
| 63 | 140 | New helper in governance_modlog crate + existing site wiring + 63d staleness alert (~20 lines, Move 2) |
| 64 | 170 | Handler + `admin_assign_jury.rs:139` flip (Move 3 flip-verification sequence) + test edit + new `jury_common.rs` helper file (Move 5) |
| 65 | 180 | Handler + replacement-pick logic + test (imports shares_active_sponsor from jury_common per Move 5) |
| 66 | 120 | Handler in api_crud + case status flip |
| 67 | 200 | View-crate filter extension + handler |
| 68 | 320 | 6 route adds + `all_mvp_endpoints_return_non_404` smoke test (Phase A non-404 sweep + Phase B per-handler happy-path assertions, Move 4) |
| 69 | 220 | 3-branch compound test |
| 69a | 200 | 1 migration + 1 doc + 2 tests + 1 plan-doc line |
| 70 | 0 | Phase-close PR only |

---

## §6. Critical conventions (fork-local)

These conventions are in-tree and auto-enforced. Phase 5c handlers MUST respect them.

**C1. Exhaustive match on `CaseStatus`.** ADR-013. All 9 variants (`Open, ThresholdMet, JurySelection, InReview, Decided, Appealed, Closed, EmergencyRemove, AdminReview`) enumerated explicitly — no `_ =>` fall-through. Existence proofs: `admin_close_case.rs:65-75`, `admin_assign_jury.rs:108-116`, `get_case.rs:47-55`. Workspace clippy denies `unreachable`, `unwrap_used`, `expect_used`, `as_conversions`, `allow_attributes`, `items_after_statements`.

**C2. Actor pseudonym + governance log pairing.** Every write path touching moderation state calls `actor_pseudonym_helper::get_or_create(&mut pool, person_id)` **before** calling `governance_log::append(...)`. Existence proof: `submit_jury_vote.rs:100`, `admin_assign_jury.rs:71-72, 157-158`. Inside a tx closure the idiom is `(&mut *conn).into()` instead of `&mut context.pool()`.

**C3. Redaction is transparent via `governance_log::append`.** The `append` function internally calls `scrub_json(&payload)` before insert (`governance_log.rs:96` — verified by Explore agent #1). Handlers do NOT call `scrub` directly on payload fields. The only exception is if a handler writes user text directly into `public_case_log.rationale_redacted` or similar user-visible columns, in which case explicit `redaction::scrub(text)` is required. Phase 5c task 66 (`request_appeal`) writes `reason` into `appeal.reason` — that column is not user-visible (admin-only eventually), so `scrub` is not called there; the governance_log entry for `appeal_requested` carries the reason through the auto-scrub path.

**C4. `LemmyResult<()>` + `?` in tests.** Workspace denies `.unwrap()`, `.expect()`. All e2e tests return `Result<(), Box<dyn std::error::Error>>` or `LemmyResult<()>` and use `?`. Mirror: `e2e.rs:741` `report_to_modlog_golden_path`.

**C5. Route registration lives inline.** There is **no** `crates/api/routes/src/governance.rs`. Task 68 extends the existing scope block at `crates/api/routes/src/lib.rs:505-522`. Imports extend the existing `use lemmy_api::governance::{...}` at `lib.rs:35-42` and `use lemmy_api_crud::governance::{...}` at `lib.rs:138`.

**C6. Entry-kind constants.** New log kinds MUST be declared as `pub const ENTRY_KIND_*` in `crates/api/api/src/governance/governance_log.rs` (lines 50-64 currently). Phase 4 call sites still use string literals — migrating them is the optional cosmetic task noted in the file comment at line 46-48; Phase 5c new code uses the constants. If task 64/65 adds `"jury_accepted"`, `"jury_declined"`, `"jury_replacement_selected"`, and task 66 adds `"appeal_requested"`, each **must** be declared as a const and used from the handler.

**C7. Admin guard is a sync helper, not middleware.** `is_admin(&local_user_view)?` inside the handler body as the first call (after `check_local_user_valid`). Mirror: `admin_assign_jury.rs:68`, `admin_close_case.rs:23-28`. Task 62 (`admin_reputation_stats`) uses this.

**C8. Status-code mapping.** `LemmyErrorType::NotFound → 404`, `IncorrectLogin → 401`, everything else (including `NotAnAdmin`) → 400 (per `crates/utils/src/error.rs:224-231`). Task 68's smoke test must assert `[200, 400, 401]` — **not** `[200, 401, 403]` — because `NotAnAdmin` maps to 400 in this fork, not 403.

**C9. Testcontainers for DB.** e2e tests boot Postgres via `testcontainers::GenericImage::new("pgautoupgrade/pgautoupgrade", "18-alpine")` — not `docker run --user`. The `--user $(id -u):$(id -g)` advice in the command template is obsolete here; testcontainers handles UID mapping via socket. See `e2e.rs:83-112` `governance_fixtures`.

**C10. ConfigCache per-request.** Handlers build `let mut cache = ConfigCache::default();` (or `::new()`) at entry; pass by `&mut` through all config reads inside the request. Cross-request cache does not exist — task 69's config-flip third branch works because the next handler call builds a fresh cache (no invalidation needed).

---

## §7. File tree (new + touched)

**New files (10):**

```
crates/api/api/src/governance/get_my_reputation.rs                  (task 61, ~80 lines)
crates/api/api/src/governance/admin_reputation_stats.rs             (task 62, ~180 lines)
crates/api/api/src/governance/accept_jury_assignment.rs             (task 64, ~120 lines)
crates/api/api/src/governance/decline_jury_assignment.rs            (task 65, ~150 lines)
crates/api/api/src/governance/list_cases.rs                         (task 67, ~120 lines)
crates/api/api_crud/src/governance/request_appeal.rs                (task 66, ~120 lines)
migrations/2026-04-20-000000-0000_add_governance_log_notify/up.sql  (task 69a)
migrations/2026-04-20-000000-0000_add_governance_log_notify/down.sql (task 69a)
docs/brehon-law-inspired-network/SUBSCRIPTIONS.md                   (task 69a)
.claude/PRPs/reports/phase-5c-complete-report.md                    (task 70)
```

**Touched files (10):**

```
crates/api/api/src/governance/mod.rs                    (+5 mod decls for new handlers)
crates/api/api_crud/src/governance/mod.rs               (+1 mod decl: request_appeal)
crates/api/api/src/governance/governance_log.rs         (+4 ENTRY_KIND consts)
crates/api/api/src/governance/admin_assign_jury.rs      (line 139 Accepted→Selected; 1-line change)
crates/api/api_common/src/governance.rs                 (+6 response DTOs)
crates/api/routes/src/lib.rs                            (+6 route entries in governance scope; +7 imports)
crates/db_views/governance_case/src/impls.rs            (task 67 extends or adds filtered list fn)
crates/db_views/governance_modlog/src/impls.rs          (task 63 adds list_capability_changed_entries fn)
crates/server/tests/e2e.rs                              (task 64 test edit; tasks 68/69/69a add 4 new tests)
docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md (task 69a.5 SSE-over-WS paragraph — ONE paragraph)
```

**Untouched (deliberately):**

- No `crates/db_schema/src/source/governance/*.rs` changes — all source structs already exist with correct shapes.
- No `Cargo.toml` changes. `tokio-postgres` is already a workspace dep (`Cargo.toml:223`) — task 69a.3 uses it as a dev-dep addition to `crates/server/Cargo.toml` if not already present there; Explore agent #2 confirms the workspace dep exists.
- No `crates/tools/` directory creation. Task 69's founder-seeding uses direct `reputation_event` INSERT inside the test (not CLI).

---

## §8. DTO coverage matrix

**Already shipped (verified by Explore agent #1 — `crates/api/api_common/src/governance.rs` at plan-write time):**

| DTO | Line | Used by |
|---|---|---|
| `AcceptJuryAssignment` | 89-95 | Task 64 |
| `DeclineJuryAssignment` | 97-105 | Task 65 |
| `RequestAppeal` | 156-163 | Task 66 |
| `GetMyReputation` | 180-187 | Task 61 |
| `ListGovernanceCases` | 51-61 | Task 67 |

**Phase 5c adds (6 response types):**

| DTO | Shape | Used by |
|---|---|---|
| `AcceptJuryAssignmentResponse` | `{ case_id: ModerationCaseId, accepted: bool }` | Task 64 |
| `DeclineJuryAssignmentResponse` | `{ case_id: ModerationCaseId, replacement_person_id: Option<PersonId> }` | Task 65 |
| `RequestAppealResponse` | `{ appeal_id: AppealId, case_id: ModerationCaseId }` | Task 66 |
| `GetMyReputationResponse` | Wraps `ReputationSummaryView` (ADR-005 visibility rules already applied via `#[serde(skip)]`) | Task 61 |
| `AdminReputationStats` | `{ community_id: Option<CommunityId> }` — request | Task 62 |
| `AdminReputationStatsResponse` | See §11.2 for full shape (buckets + counts + founder stats) | Task 62 |

Each new DTO follows the existing convention: `#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]` + `#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]` + `#[skip_serializing_none]`. `RevokeEndorsement` already exists (`api_common/src/governance.rs:213`) but **no handler is shipped in 5c** — it is deferred to v1 per [05 §2].

---

## §9. Definition-of-done commands (per-task)

All commands use the Windows wrapper scripts per `.claude/rules/cargo-output-capture.md`. Exit code propagated via `> file.log 2>&1` + explicit `exit $status`.

| Task | DoD command(s) | Expected |
|---|---|---|
| 0 | `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_common --features full > .claude/audit-check.log 2>&1"` | exit 0 (baseline) |
| 61 | `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task61.log 2>&1"` | exit 0 |
| 62 | `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task62.log 2>&1"` | exit 0 |
| 63 | `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_views_governance_modlog --features full > .claude/build-task63-view.log 2>&1"` then `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task63-api.log 2>&1"` | both exit 0 |
| 64 | `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task64.log 2>&1"` then `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server report_to_modlog_golden_path --features full > .claude/test-task64.log 2>&1"` | both exit 0 — regression guard |
| 65 | `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task65.log 2>&1"` | exit 0 |
| 66 | `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_crud --features full > .claude/build-task66.log 2>&1"` OR `scripts\\brehon\\cargo-check.bat --workspace --features full` per `feedback_api_crud_oauth_feature_quirk.md` if api_crud check false-reds | exit 0 |
| 67 | `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_views_governance_case --features full > .claude/build-task67-view.log 2>&1"` then `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task67-api.log 2>&1"` | both exit 0 |
| 68 | `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_routes --features full > .claude/build-task68-routes.log 2>&1"` then `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server all_mvp_endpoints_return_non_404 --features full > .claude/test-task68.log 2>&1"` | both exit 0 |
| 69 | `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server ineligible_user_cannot_be_picked_for_jury --features full > .claude/test-task69.log 2>&1"` | exit 0 (all 3 branches) |
| 69a | `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server governance_events_notify_fires underscore_prefix_usernames_still_register --features full > .claude/test-task69a.log 2>&1"` | exit 0 |
| 70 | full workspace check + clippy + all e2e tests (see §12 Level 3–5) | all exit 0 |

**Clippy baseline capture** (required by `.claude/rules/pre-phase-harness-audit.md` §3, task 0): `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --no-deps --features full -- -D warnings > .claude/audit-clippy-baseline.log 2>&1"` — expected exit 0 at Phase 5b tip; any non-zero is pre-existing debt to surface via decision queue before task 1.

---

## §10. Cross-cutting invariants

Per IMPLEMENTATION-PLAN-v0.md §4. Phase 5c respects all four; spot-check:

**§4.1 Hash-chained governance log.** Every new handler (tasks 61–67) that mutates state calls `governance_log::append(...)` before returning. Specific entries: task 61 NO log (read-only); task 62 one `admin_reputation_stats_queried` entry (observability audit trail — optional, pending decision-queue, see §17 Q4); task 63 already emits `capability_changed` via Phase 5a code; task 64 emits `jury_accepted`; task 65 emits `jury_declined` + `jury_replacement_selected`; task 66 emits `appeal_requested`; task 67 NO log (read-only).

**§4.2 `actor_pseudonym` + redaction.** Every handler that writes state calls `actor_pseudonym_helper::get_or_create` before the first `governance_log::append`. Redaction is auto (see §6 C3). Task 66's `reason` field rides the auto-scrub path. Task 69a.3 NOTIFY trigger payload carries only `entry_id`, `kind`, `published_at` — no raw user text — so no additional scrubbing concern.

**§4.3 `EmergencyRemove` exhaustive match.** Task 64 (`accept_jury_assignment`) and task 65 (`decline_jury_assignment`) gate on `jury_assignment.status == Selected`; they don't branch on `CaseStatus`. Task 66 (`request_appeal`) gates on `case.status == Decided` explicitly; it must match all 9 variants without `_ =>`. Task 67 (`list_cases`) filters on `status: Option<CaseStatus>` — the Rust-side match on the Option wraps an enum filter, exhaustive at the Diesel DSL level (no Rust match needed). Task 62 (`admin_reputation_stats`) does not touch `CaseStatus`.

**§4.4 AGPLv3 / source disclosure.** Not triggered — no release artefacts in 5c.

---

## §11. Step-by-step tasks

### §11.0 Task 0 — pre-phase audit + branch cut + decision-queue intake

**ACTION.**
1. Verify Phase 5b fully merged into `governance-v0`. Expected `git log governance-v0 --oneline | head -15` shows the squash-merge 6566dce43 plus 4 cherry-picked review fixes (a6e6265f9, 0e86dcb4e, 9102abba2, 72fd4be1f), retro eaa413cd8, and the risk-reduction commits from Moves 1/7 (decision-queue entries 19-21 + cargo-test.bat guard). Tasks 56–61 are rolled into the squash commit — byte-level-verified at eaa413cd8. If any of the above commits are missing, surface via decision queue; do NOT start Phase 5c.
2. `git checkout governance-v0 && git pull origin governance-v0 && git checkout -b phase-5c`. Per `.claude/rules/phase-branch.md` — **advisor creates the branch at transition handoff**, impl agent only verifies `git branch --show-current == phase-5c`. If the current branch is `governance-v0`, surface in decision queue.
3. Run the 3 wrapper probes per `.claude/rules/pre-phase-harness-audit.md` §1:
   - Probe 1: `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/audit-cargo-check-p.log 2>&1"` — assert only `lemmy_utils` compiled.
   - Probe 2: `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/audit-cargo-check-features.log 2>&1"` — assert features activated.
   - Probe 3: `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/audit-cargo-test.log 2>&1"` — assert only e2e built. The cargo-test.bat wrapper now auto-appends `-- --test-threads=1` for e2e runs without `--no-run` (per Move 7 of the risk-reduction strategy); this probe uses `--no-run` so the guard does NOT fire. Other task-level e2e invocations WILL see the guard.
4. Clippy baseline per §9: expected clean at 5b tip.
5. DoD smoke per `pre-phase-harness-audit.md` §2: run each DoD command in §9 once against current HEAD. Expected-red: every new-file DoD (tasks 61-69, 69a because handler/test doesn't exist yet). Expected-green: tasks 0, 70 (framework-level checks). Additional: re-run Phase 5b regression `sponsor_liability_with_founder_multiplier` as a green check that the cherry-picked fixes are in place.
6. Decision-queue check: `.claude/decision-queue.json` entries #17, #19, #20, #21 (from Moves 1-2 of the risk-reduction strategy) must be resolved with answers. If any are still `answer: null`, stop and surface. Impl may self-resolve Q1-Q4 + Q7 (the original plan §17 intake) at this point using the recommendations in §17; Q8 is now #21 and already resolved.
7. **Step 7 deleted.** (Plan file was landed on governance-v0 @ 6c68f0d78 via decision-queue #18 on 2026-04-18. No self-commit needed. Leave this list item number for commit-history continuity; just document the delta.)
8. **External-API probes** (Move 6 of the risk-reduction strategy). Run all three in parallel from `scratch/phase-5c-probes/` — each ≤ 30 lines of scratch Rust + SQL — and land as a single `chore(scratch): phase-5c external-API probes` commit on phase-5c before task 1:
   - **Probe 62-sql** — `scratch/phase-5c-probes/bucket_query.sql`: one `CASE WHEN` bucketing query against `reputation_snapshot.jury_reliability`. Run with `psql -f bucket_query.sql` against the local e2e testcontainer (or manually via `docker exec`); assert the result shape is 5 rows with `count` column. If CASE-WHEN syntax varies on postgres 18 (the testcontainer image), surface as a decision-queue blocker before task 62.
   - **Probe 68-actix** — `scratch/phase-5c-probes/actix_smoke.rs`: a 15-line test that constructs `App::new().configure(|cfg| lemmy_api_routes::config(cfg, &RateLimit::default()))`, sends a `TestRequest::post().uri("/api/v4/governance/report")` with empty body, asserts 400 (not 404). Verifies Q19's answer. If 404, the route wiring is broken OR the import path is wrong; surface.
   - **Probe 69a-notify** — `scratch/phase-5c-probes/notify_smoke.rs`: a 25-line scratch that connects via tokio-postgres 0.7.16 (per Q20), spawns a connection-pump task that polls `poll_message` and forwards `AsyncMessage::Notification` onto an `mpsc::UnboundedSender`, does `LISTEN hello`, issues `NOTIFY hello, 'world'` from a second connection, awaits `{ "channel": "hello", "payload": "world" }` from the receiver with a 1-second timeout. Verifies Q20's answer.
   - If any probe fails, the plan's assumption is broken — **stop task 0 and surface**; do not burn task 1+ iteration on a broken assumption.

**DoD.**
- On `phase-5c` branch.
- `.claude/audit-*.log` files all exit-0-verified.
- Decision queue has entries #17, #19, #20, #21 **resolved** (from risk-reduction strategy) + any original §17 intake questions (Q1-Q4, Q7) that impl chose to pre-seed vs self-resolve.
- `scratch/phase-5c-probes/` contains the three probes listed in step 8, each exit-0, committed as `chore(scratch): phase-5c external-API probes`.
- Phase 5b regression `sponsor_liability_with_founder_multiplier` re-confirmed green.
- No commits of new handler code yet (probes are scratch only, not handler code).

---

### §11.1 Task 61 — `get_my_reputation` handler

**ACTION.** Create `crates/api/api/src/governance/get_my_reputation.rs`. Add mod decl to `crates/api/api/src/governance/mod.rs`. Add `GetMyReputationResponse` DTO to `crates/api/api_common/src/governance.rs`.

**HANDLER SHAPE** (target: ~80 lines including imports + doc comment):

```rust
pub async fn get_my_reputation(
  Query(data): Query<GetMyReputation>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetMyReputationResponse>> {
  check_local_user_valid(&local_user_view)?;
  let mut cache = ConfigCache::default();
  let mut conn_owned = get_conn(&mut context.pool()).await?;
  let snapshot = load_or_compute_snapshot(
    &mut conn_owned,
    local_user_view.person.id,
    data.community_id,
    &mut cache,
  ).await?;

  // Row-existence check avoids the read-path recompute when a snapshot
  // already exists (IMPLEMENTATION-PLAN-v0.md line 371 GOTCHA).
  // load_or_compute_snapshot already does SELECT-first so the gotcha
  // is satisfied — this comment is for future maintainers, not an
  // additional guard.

  // Wrap the snapshot in the view shape the public API returns.
  // ReputationSummaryView already applies ADR-005 visibility rules
  // via #[serde(skip)] on the four raw dimension scores.
  let view = ReputationSummaryView::from(&snapshot);
  let active = count_active_sanctions(&mut context.pool(), local_user_view.person.id).await?;
  Ok(Json(GetMyReputationResponse {
    view: ReputationSummaryView { active_sanctions: active, ..view },
  }))
}
```

**MIRROR.** Mix of `list_my_jury_queue.rs:14-22` (auth + simple Json response) and `create_report.rs:99` (`load_or_compute_snapshot` call pattern).

**GOTCHA.** `ReputationSummaryView` does NOT currently have a `From<&ReputationSnapshot>` impl (checked Explore agent #3 output §5). Task 61 adds it in `crates/db_views/reputation/src/lib.rs`. Active-sanction count is a second round-trip via an existing `count_active_sanctions` helper in `db_views/reputation/src/impls.rs:38-49`.

**GOTCHA.** `load_or_compute_snapshot` takes `conn: &mut AsyncPgConnection` not `&mut DbPool` — must call `get_conn` first. Mirror `reputation_snapshot.rs:454-481` callers.

**GOTCHA.** No governance_log write (read-only handler).

**VALIDATE.** §9 task 61 DoD.

---

### §11.2 Task 62 — `admin_reputation_stats` observability

**ACTION.** Create `crates/api/api/src/governance/admin_reputation_stats.rs`. Add `AdminReputationStats` + `AdminReputationStatsResponse` DTOs to `api_common/src/governance.rs`. Add mod decl.

**RESPONSE DTO SHAPE** (per IMPLEMENTATION-PLAN-v0.md line 373):

```rust
pub struct AdminReputationStatsResponse {
  pub buckets: ReputationBuckets,          // per-dimension histograms
  pub thresholds_current: ThresholdsSnapshot,
  pub capability_counts: CapabilityCounts,
  pub founder_event_stats: FounderEventStats,
  pub calculated_at: DateTime<Utc>,
}

pub struct ReputationBuckets {
  pub reporting_accuracy: [i64; 5],  // [0, 1-30, 31-80, 81-200, 200+]
  pub jury_reliability: [i64; 5],
  pub participation_consistency: [i64; 5],
  pub endorsement_strength: [i64; 5],
}

pub struct CapabilityCounts {
  pub jury_eligible_count: i64,
  pub trusted_reporter_count: i64,
  pub can_sponsor_count: i64,
}

pub struct FounderEventStats {
  pub active_count: i64,    // expires_at > now()
  pub expired_count: i64,   // expires_at <= now()
}

pub struct ThresholdsSnapshot {
  pub jury_reliability: i64,
  pub reporting_accuracy: i64,
  pub endorsement_strength: i64,
}
```

**HANDLER SHAPE.**
1. `is_admin(&local_user_view)?;`
2. Build `ConfigCache`, read three thresholds.
3. Run one `sql_query(...)` per dimension for buckets. Mirror `admin_assign_jury.rs:304-322` shape + Explore agent #3 §15 suggested CASE-WHEN SQL.
4. Run one query for capability counts (3 `count(*) FILTER (WHERE ...)` aggregates on `reputation_snapshot`).
5. Run one query for founder stats (`count(*) FILTER (WHERE expires_at > now())` on `reputation_event`).
6. Optional: one `governance_log::append(ENTRY_KIND_ADMIN_REPUTATION_STATS_QUERIED, ...)` — see §17 Q4 decision queue.

**GOTCHA.** Per IMPLEMENTATION-PLAN-v0.md line 373: "bucketing via SQL `CASE WHEN` per dimension is cheaper than loading all snapshots into Rust. One query per dimension." Four dimension queries + one capability query + one founder query = 6 round-trips. Acceptable for an admin-only endpoint.

**GOTCHA.** Admin backstops (`admin_reputation_stats`, `admin_assign_jury`, `admin_close_case`) are NOT counted in the 11 MVP endpoints per IMPLEMENTATION-PLAN-v0.md line 385. They are registered but separate.

**GOTCHA.** Scope is `Scope::Instance` for thresholds (per-community thresholds are post-MVP). The `community_id` filter on the input is optional and applies to the bucket WHERE clause (filter `reputation_snapshot.community_id = ?` or `IS NULL` for instance-wide).

**MIRROR.** `admin_close_case.rs` for guard pattern + optional log entry; `admin_assign_jury.rs:304-322` for `sql_query` binding.

**VALIDATE.** §9 task 62 DoD.

---

### §11.3 Task 63 — threshold-crossing log wire-up

**ACTION.** Three sub-steps; single commit.

**63a.** Verify `detect_capability_changes` already emits `capability_changed` entries per Explore agent #1 §7 (confirmed: `reputation_snapshot.rs:286-308` emits one entry per flip, per dimension, with `{dimension_flipped, direction, snapshot_community_id}` payload). **No code change here** — task 63 is mostly wire-up + test.

**63b.** Add `list_capability_changed_entries_since(pool, since_id, limit)` helper to `crates/db_views/governance_modlog/src/impls.rs`. This reads **from `governance_log` directly** (NOT from `public_case_log`) filtering on `entry_kind = 'capability_changed'` with `id > since_id ORDER BY id ASC LIMIT ?`. Returns a new `CapabilityChangeLogEntry` view struct (id, entry_kind, payload, actor_pseudonym, created_at, signature).

**63c.** Add e2e test `capability_change_entries_reachable_via_modlog_crate` to `e2e.rs`: seed 3 users with reputation events that cross the `jury_reliability` threshold of 50, call `run_snapshot_batch`, assert `list_capability_changed_entries_since(pool, 0, 10)` returns ≥3 entries with `direction: "gained"`.

**63d — Snapshot-staleness alert (new per decision-queue #21, Move 2 of risk-reduction strategy).** In the existing snapshot-recompute scheduled task (Phase 5a task 54 wired the clokwerk registration in `crates/server/src/scheduled_tasks.rs`), after `run_snapshot_batch` completes, check snapshot freshness:

```rust
// Pseudo-sketch; actual integration point depends on scheduled_tasks.rs
// layout from Phase 5a task 54.
let max_calculated_at: Option<DateTime<Utc>> = reputation_snapshot::table
    .select(diesel::dsl::max(reputation_snapshot::calculated_at))
    .first(conn).await?;
let interval_s = config.scheduler.snapshot_interval_seconds;
let threshold = Utc::now() - Duration::seconds(2 * interval_s);
if let Some(max) = max_calculated_at {
    if max < threshold {
        tracing::error!(
            target: "governance::integrity",
            calculated_at = ?max,
            threshold = ?threshold,
            interval_s,
            "reputation_snapshot staleness detected (DoD line 398)"
        );
    }
} else {
    tracing::error!(
        target: "governance::integrity",
        "reputation_snapshot table empty — snapshot batch has never run"
    );
}
```

**GOTCHA-63d-a.** No governance_log entry (this is an ops signal, not a governance signal). Emits to `tracing` only — subscribers (e.g. `tracing_subscriber::fmt` in production, `tracing-test` in tests) route it appropriately.

**GOTCHA-63d-b.** The config key `scheduler.snapshot_interval_seconds` must already exist from Phase 5a task 54 seed. Verify with `ripgrep "snapshot_interval_seconds" crates/` before writing the read; if missing, fall back to a local `const SNAPSHOT_INTERVAL_S: i64 = 60` and flag as a decision-queue entry (shouldn't happen but defensive).

**GOTCHA-63d-c.** Test coverage: extend `63c` to add a second assertion — seed reputation_snapshot with an artificially stale `calculated_at` (now() - 10 minutes), call the integrity-check path (expose the logic as an extracted `check_snapshot_staleness` fn that takes `(conn, interval_s, now)` so the test can inject a frozen time), assert `tracing-test` captured an `ERROR`-level event at `target: "governance::integrity"`.

**GOTCHA.** IMPLEMENTATION-PLAN-v0.md line 375 calls out two v1 reservations that Phase 5c must NOT implement:
- Config threshold edits cascading into mass capability losses are **one log entry per affected user per tick** (no dedupe, no burst-collapse). v0 behaviour; do not optimise.
- In-flight `jury_assignment` rows (`Selected` or `Accepted`) are NOT revoked when a threshold edit drops a juror below `jury_eligible`. Snapshot recompute affects **future** selections only. Add a doc comment to the 63b helper referencing this.

**GOTCHA.** The `GovernanceModlogView` derives from `public_case_log` not `governance_log` per Explore agent #3 §7 — `capability_changed` entries cannot be surfaced by extending `list_public_case_log`. Task 63b's new function is a sibling, not a filter parameter on the existing function.

**GOTCHA.** Task 63 does NOT add a new HTTP endpoint for capability-change entries. The surfacing is via the crate function only — task 62 (`admin_reputation_stats`) can call it if useful; otherwise it's reserved for v1's admin dashboard. Document this in the task commit message.

**MIRROR.** `list_public_case_log` at `db_views/governance_modlog/src/impls.rs:72-95` for the function shape.

**VALIDATE.** §9 task 63 DoD (two crate checks).

---

### §11.4 Task 64 — `accept_jury_assignment` + `admin_assign_jury.rs:139` flip

**ACTION.** Six sub-steps, final single commit. The ordering matters per Move 3 of the risk-reduction strategy: the test edit is written FIRST on a local throwaway commit to prove it has semantic teeth to catch a broken flip; then the flip + handler land; then the final squash produces one task 64 commit. This prevents a broken test edit from masking a broken flip.

**64e-pre (NEW — Move 3 flip-verification).** Land the test edit FIRST on a throwaway LOCAL commit (not pushed, will be squashed):
1. Edit `crates/server/tests/e2e.rs::report_to_modlog_golden_path` per the 64e skeleton below (5 accept calls + governance_log assertion extension).
2. Run: `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server report_to_modlog_golden_path --features full > .claude/task64-preflip-test.log 2>&1"`
3. **Assert the test fails** — either build error (handler doesn't exist) or runtime failure (assignments still have status=Accepted so the accept calls are no-ops OR filter mismatches). If the test PASSES here, the test edit does not have the semantic teeth to catch a broken flip — STOP and re-check the assertion delta.
4. Keep the local commit; proceed to 64a.

**64a.** Edit `crates/api/api/src/governance/admin_assign_jury.rs:139` from `status: JuryAssignmentStatus::Accepted,` to `status: JuryAssignmentStatus::Selected,`. Update the comment on line 133 (currently: "Insert JuryAssignment rows with status=Accepted (v0 testability)") to reference task 64's accept-flow activation.

**After 64a.** Re-run the test from 64e-pre step 2. Expected: still fails (flip works — `Accepted → Selected` — but handler still missing, so the 5 accept calls don't find a matching assignment). This confirms the flip had observable effect.

**64b.** Create `crates/api/api/src/governance/accept_jury_assignment.rs`. Handler body:

```rust
pub async fn accept_jury_assignment(
  Json(data): Json<AcceptJuryAssignment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AcceptJuryAssignmentResponse>> {
  check_local_user_valid(&local_user_view)?;
  let caller_id = local_user_view.person.id;
  let caller_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), caller_id).await?;

  let mut conn = get_conn(&mut context.pool()).await?;
  conn.run_transaction(|conn| async move {
    // 1. Load assignment; verify status = Selected.
    let assignment: JuryAssignment = jury_assignment::table
      .filter(jury_assignment::case_id.eq(data.case_id))
      .filter(jury_assignment::person_id.eq(caller_id))
      .filter(jury_assignment::status.eq(JuryAssignmentStatus::Selected))
      .first(conn).await
      .map_err(|_| LemmyErrorType::NotFound)?;

    // 2. Load case for conflict checks.
    let case: ModerationCase = moderation_case::table
      .filter(moderation_case::id.eq(data.case_id))
      .first(conn).await?;

    // 3. Conflict check 1: caller is not case reporter.
    //    Phase 5c v0 decision: "reporter" is an abstract concept — the
    //    `moderation_case.creator_id` is the first reporter. Check against that.
    if case.creator_id == Some(caller_id) {
      return Err(LemmyErrorType::NotFound.into());  // mask as 404 per pseudo-403 convention
    }

    // 4. Conflict check 2: caller NOT in target's sponsor cluster.
    //    v0 definition: "shares ≥1 active sponsor with target".
    if let Some(target_id) = case.target_person_id {
      let same_cluster =
        shares_active_sponsor(conn, caller_id, target_id).await?;
      if same_cluster {
        return Err(LemmyErrorType::NotFound.into());
      }
    }

    // 5. Flip status → Accepted; stamp responded_at.
    diesel::update(jury_assignment::table.filter(jury_assignment::id.eq(assignment.id)))
      .set((
        jury_assignment::status.eq(JuryAssignmentStatus::Accepted),
        jury_assignment::responded_at.eq(Some(chrono::Utc::now())),
      ))
      .execute(conn).await?;

    // 6. Log.
    governance_log::append(
      &mut conn.into(),
      ENTRY_KIND_JURY_ACCEPTED,
      json!({
        "case_id": data.case_id.0,
        "juror_pseudonym": caller_pseudonym,
      }),
      Some(caller_pseudonym.clone()),
    ).await?;
    Ok::<_, LemmyError>(())
  }.scope_boxed()).await?;

  Ok(Json(AcceptJuryAssignmentResponse { case_id: data.case_id, accepted: true }))
}
```

**64c (REVISED per Move 5 of risk-reduction strategy).** Helper `shares_active_sponsor(conn, a, b) -> LemmyResult<bool>` returns true if any active `surety` row pair shares the same sponsor for both `a` and `b`. Implementation: one `sql_query` with `EXISTS (SELECT 1 FROM surety s1 JOIN surety s2 ON s1.sponsor_id = s2.sponsor_id WHERE s1.sponsored_id = $1 AND s2.sponsored_id = $2 AND s1.revoked_at IS NULL AND s2.revoked_at IS NULL LIMIT 1)`.

**Location — place in NEW file `crates/api/api/src/governance/jury_common.rs`** (not inline in `accept_jury_assignment.rs`). Declaration: `pub(crate) async fn shares_active_sponsor(...)`. Rationale: task 65's decline handler reuses the same predicate for its own sponsor-cluster guard (see §11.5); extracting now prevents duplicated code between the two handlers and a CodeRabbit "duplicate logic" finding. Add `pub mod jury_common;` to `crates/api/api/src/governance/mod.rs`. File layout:

```rust
// crates/api/api/src/governance/jury_common.rs
// Shared helpers for jury-flow handlers (accept, decline, replacement-pick).
// Extracted at task 64 to preempt duplicated code between accept and decline
// per the risk-reduction strategy Move 5.

use diesel::sql_types::{BigInt, Bool};
use diesel::{QueryableByName, sql_query};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema::newtypes::PersonId;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[derive(QueryableByName)]
struct BoolRow { #[diesel(sql_type = Bool)] present: bool }

pub(crate) async fn shares_active_sponsor(
    conn: &mut AsyncPgConnection,
    a: PersonId,
    b: PersonId,
) -> LemmyResult<bool> {
    let row: BoolRow = sql_query(
        "SELECT EXISTS (
             SELECT 1 FROM surety s1
             JOIN surety s2 ON s1.sponsor_id = s2.sponsor_id
             WHERE s1.sponsored_id = $1
               AND s2.sponsored_id = $2
               AND s1.revoked_at IS NULL
               AND s2.revoked_at IS NULL
             LIMIT 1
         ) AS present",
    )
    .bind::<BigInt, _>(a.0)
    .bind::<BigInt, _>(b.0)
    .get_result(conn)
    .await
    .map_err(|_| LemmyErrorType::Unknown)?;
    Ok(row.present)
}
```

**Import in `accept_jury_assignment.rs` and (later) `decline_jury_assignment.rs`:** `use super::jury_common::shares_active_sponsor;`.

**GOTCHA-64c-a.** The `surety` table name + `revoked_at` column — confirm these match the actual schema before writing the SQL. Phase 5a shipped the sponsorship model; grep `crates/db_schema/src/schema.rs` for `surety` and `revoked_at` to confirm. If columns differ (e.g. `revoked_ts` vs `revoked_at`), adjust the SQL at implementation time and update this GOTCHA with the actual name.

**GOTCHA-64c-b.** Workspace clippy denies `.unwrap()` / `.expect()` / `as_conversions`. The `map_err` above propagates correctly; no `.unwrap()` anywhere. `a.0` and `b.0` are `i32` → `BigInt` bind works directly without cast.

**64d.** Add `pub const ENTRY_KIND_JURY_ACCEPTED: &str = "jury_accepted";` to `governance_log.rs` lines 50-64 block.

**64e.** Edit `crates/server/tests/e2e.rs::report_to_modlog_golden_path`. Insert a new block after line 961 (after `assigned_person_ids.len() == 5` assertion) and before line 1001 (juror vote loop):

```rust
// -- 10a. Every assigned juror calls accept_jury_assignment (task 64) --
for juror_id in &assign_resp.assigned_person_ids {
  let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
  let _resp = accept_jury_assignment(
    Json(AcceptJuryAssignment { case_id }),
    context.clone(),
    juror_view,
  ).await?.into_inner();
}
```

Plus extend the governance_log count assertion at e2e.rs:997 to include `jury_accepted == Some(&5)` and keep `jury_assigned == Some(&5)`, `panel_assembled == Some(&1)`.

**64f (NEW — Move 3 squash-and-verify).** After 64a–64e all land locally:
1. Re-run the task 64 DoD per §9: `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server report_to_modlog_golden_path --features full > .claude/task64-postflip-test.log 2>&1"`. Assert exit 0. This is the triangulation point: the test now PASSES **because** the flip works AND the handler exists AND the test edit has the right assertions (the three were independently verified at 64e-pre → 64a → 64b–64d).
2. Squash the local commits (64e-pre + 64a + 64b + 64c + 64d + 64e-post) into ONE commit: `feat(governance): task 64 — accept_jury_assignment + admin_assign_jury flip (Selected for accept handshake)`. Commit message body documents the 3-step flip-verification sequence (64e-pre fail, post-64a fail, post-64b–e green).
3. Single commit pushed. PR contains the final state only; local throwaway history provides the verification audit trail in the commit message.

**GOTCHA.** `submit_jury_vote.rs:137` filters on `status.eq(Accepted)`. Task 64 preserves this. Without task 64b's new accept calls, `submit_jury_vote` would find no Accepted assignment (because 64a made it Selected) and fail with NotFound — this is the regression test guard.

**GOTCHA.** "Caller is not the case's reporter" — IMPLEMENTATION-PLAN-v0.md line 377. In v0 there is no separate `report` table (ADR-013 collapsed reports into `moderation_case`). The "first reporter" is `moderation_case.creator_id`. Task 64b uses that as the reporter proxy. If v1 adds a `case_report` table, task 64's conflict check gets revisited.

**GOTCHA.** Sponsor-cluster definition is one-hop (shares a common sponsor). v1 may extend to two-hop or endorsement-cluster. Document this in handler doc comment pointing at OQ-008.

**GOTCHA.** The case row read (step 2 of 64b) is not strictly necessary for the status flip but IS necessary for the conflict check. Keep it.

**GOTCHA.** `case.creator_id` is `Option<PersonId>` — for system-generated cases (e.g. from federation or admin emergency_remove) it can be None. Current Phase 4 code always has Some creator; task 64b's guard check handles both.

**MIRROR.** `submit_jury_vote.rs:116-170` transaction pattern + DSL update; `admin_assign_jury.rs:157-169` for `governance_log::append` inside a tx.

**VALIDATE.** §9 task 64 DoD (regression-guarding the Phase 4 test is the key assertion).

---

### §11.5 Task 65 — `decline_jury_assignment` + replacement pick

**ACTION.** Create `crates/api/api/src/governance/decline_jury_assignment.rs`. Add `ENTRY_KIND_JURY_DECLINED` + `ENTRY_KIND_JURY_REPLACEMENT_SELECTED` consts. **Import `shares_active_sponsor` from `super::jury_common`** (already created in task 64 per Move 5 of risk-reduction strategy; do NOT replicate the helper).

**HANDLER SHAPE.**

```rust
pub async fn decline_jury_assignment(
  Json(data): Json<DeclineJuryAssignment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<DeclineJuryAssignmentResponse>> {
  check_local_user_valid(&local_user_view)?;
  let caller_id = local_user_view.person.id;
  let caller_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), caller_id).await?;
  let mut cache = ConfigCache::default();
  let mut conn = get_conn(&mut context.pool()).await?;

  let replacement = conn.run_transaction(|conn| async move {
    // 1. Load assignment; status must be Selected or Accepted.
    let assignment: JuryAssignment = jury_assignment::table
      .filter(jury_assignment::case_id.eq(data.case_id))
      .filter(jury_assignment::person_id.eq(caller_id))
      .filter(jury_assignment::status.eq_any([JuryAssignmentStatus::Selected, JuryAssignmentStatus::Accepted]))
      .first(conn).await
      .map_err(|_| LemmyErrorType::NotFound)?;

    // 2. Load case.
    let case: ModerationCase = moderation_case::table
      .filter(moderation_case::id.eq(data.case_id))
      .first(conn).await?;

    // 3. Flip status → Declined; stamp responded_at.
    diesel::update(jury_assignment::table.filter(jury_assignment::id.eq(assignment.id)))
      .set((
        jury_assignment::status.eq(JuryAssignmentStatus::Declined),
        jury_assignment::responded_at.eq(Some(chrono::Utc::now())),
      ))
      .execute(conn).await?;

    // 4. Log decline.
    governance_log::append(
      &mut conn.into(),
      ENTRY_KIND_JURY_DECLINED,
      json!({
        "case_id": data.case_id.0,
        "juror_pseudonym": caller_pseudonym,
        "reason_redacted_present": data.reason.is_some(),
        "reason": data.reason.clone(),  // auto-scrubbed by append
      }),
      Some(caller_pseudonym.clone()),
    ).await?;

    // 5. Build exclude list: all current (non-Declined, non-Expired)
    //    assignees on this case.
    let current_assignees: Vec<PersonId> = jury_assignment::table
      .filter(jury_assignment::case_id.eq(data.case_id))
      .filter(jury_assignment::status.ne(JuryAssignmentStatus::Declined))
      .filter(jury_assignment::status.ne(JuryAssignmentStatus::Expired))
      .select(jury_assignment::person_id)
      .load(conn).await?;

    // 6. Call admin_assign_jury::select_eligible_jurors with exclude list.
    //    Panel size = 1 (just one replacement).
    let replacements = crate::governance::admin_assign_jury::select_eligible_jurors(
      conn, &case, Some(&current_assignees), &mut cache,
    ).await?;
    let replacement_id = replacements.into_iter().next();

    // 7. If we got a replacement, insert a new jury_assignment row (status=Selected).
    if let Some(new_id) = replacement_id {
      insert_into(jury_assignment::table)
        .values(JuryAssignmentInsertForm {
          case_id: data.case_id,
          person_id: new_id,
          status: JuryAssignmentStatus::Selected,
        })
        .execute(conn).await?;

      let new_pseudonym =
        actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), new_id).await?;
      governance_log::append(
        &mut conn.into(),
        ENTRY_KIND_JURY_REPLACEMENT_SELECTED,
        json!({
          "case_id": data.case_id.0,
          "new_juror_pseudonym": new_pseudonym,
        }),
        Some(caller_pseudonym.clone()),
      ).await?;
    }

    Ok::<_, LemmyError>(replacement_id)
  }.scope_boxed()).await?;

  Ok(Json(DeclineJuryAssignmentResponse {
    case_id: data.case_id,
    replacement_person_id: replacement,
  }))
}
```

**GOTCHA.** `select_eligible_jurors` already accepts `exclude_person_ids: Option<&[PersonId]>` (Phase 5b task 57 pre-wired this — `admin_assign_jury.rs:212`). No signature change.

**GOTCHA.** Concurrent-cap check applies to the replacement pick too — handled automatically because `select_eligible_jurors` reads `max_concurrent_assignments` internally (task 57).

**GOTCHA.** `select_eligible_jurors` returns up to `panel_size` (default 5). We only want ONE replacement. Use `.into_iter().next()`.

**GOTCHA.** If no replacement is eligible (pool exhausted), `replacement_person_id = None`. The case sits with one fewer juror. v0 accepts this degraded state; quorum may still be reachable (quorum=3 of 5, so up to 2 declines without replacement is tolerable). v1 adds a background job to retry.

**GOTCHA.** `JuryAssignmentStatus::Declined` and `Expired` are excluded from the "current assignees" list because those rows represent past jurors no longer on the panel — re-picking them for a different slot is fine (they haven't rejoined the active panel).

**GOTCHA.** The original declining juror IS in the exclude list (their status is now Declined, but §11.5 step 5 uses `ne(Declined)` — step 5 runs **after** step 3 updated this juror to Declined, so the juror is correctly excluded from being re-picked). Verify the ordering carefully.

**GOTCHA.** `data.reason` is carried in the governance_log payload, automatically scrubbed by `append`'s internal `scrub_json`. No direct `redaction::scrub` call needed.

**MIRROR.** `submit_jury_vote.rs` transaction pattern; `admin_assign_jury::select_eligible_jurors` existing export.

**VALIDATE.** §9 task 65 DoD.

---

### §11.6 Task 66 — `request_appeal` (api_crud)

**ACTION.** Create `crates/api/api_crud/src/governance/request_appeal.rs`. Add mod decl to `crates/api/api_crud/src/governance/mod.rs`. Add `ENTRY_KIND_APPEAL_REQUESTED` const. Add `RequestAppealResponse` DTO.

**HANDLER SHAPE.**

```rust
pub async fn request_appeal(
  Json(data): Json<RequestAppeal>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<RequestAppealResponse>> {
  check_local_user_valid(&local_user_view)?;
  let caller_id = local_user_view.person.id;
  let caller_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), caller_id).await?;
  let mut conn = get_conn(&mut context.pool()).await?;

  let appeal_id = conn.run_transaction(|conn| async move {
    // 1. Load case; must be in Decided state.
    let case: ModerationCase = moderation_case::table
      .filter(moderation_case::id.eq(data.case_id))
      .first(conn).await
      .map_err(|_| LemmyErrorType::NotFound)?;

    // 2. Exhaustive match on case.status per ADR-013.
    match case.status {
      CaseStatus::Decided => {}  // happy path
      CaseStatus::Open
      | CaseStatus::ThresholdMet
      | CaseStatus::JurySelection
      | CaseStatus::InReview
      | CaseStatus::Appealed
      | CaseStatus::Closed
      | CaseStatus::EmergencyRemove
      | CaseStatus::AdminReview => {
        return Err(LemmyErrorType::NotFound.into());
      }
    }

    // 3. Appeal window guard per [04 §6.1]: "Verify appeal window is still open".
    //    v0 definition: case.closed_at is None (case not yet closed by admin).
    //    The closed_at field is None until admin_close_case runs (Phase 4 task 46).
    if case.closed_at.is_some() {
      return Err(LemmyErrorType::NotFound.into());
    }

    // 4. Caller must be sanction target (v0: original-reporter appeals deferred to v1
    //    per IMPLEMENTATION-PLAN-v0.md line 381).
    if case.target_person_id != Some(caller_id) {
      return Err(LemmyErrorType::NotFound.into());
    }

    // 5. Insert Appeal row.
    let form = AppealInsertForm {
      case_id: data.case_id,
      requester_id: caller_id,
      reason: data.reason.clone(),
      status: AppealStatus::Requested,
    };
    let new_appeal: Appeal = insert_into(appeal::table)
      .values(&form)
      .get_result(conn).await?;

    // 6. Flip case Decided → Appealed.
    diesel::update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
      .set(moderation_case::status.eq(CaseStatus::Appealed))
      .execute(conn).await?;

    // 7. Log.
    governance_log::append(
      &mut conn.into(),
      ENTRY_KIND_APPEAL_REQUESTED,
      json!({
        "case_id": data.case_id.0,
        "appeal_id": new_appeal.id.0,
        "reason": data.reason,  // auto-scrubbed
      }),
      Some(caller_pseudonym.clone()),
    ).await?;

    Ok::<_, LemmyError>(new_appeal.id)
  }.scope_boxed()).await?;

  Ok(Json(RequestAppealResponse { appeal_id, case_id: data.case_id }))
}
```

**GOTCHA.** No re-jury in v0 per IMPLEMENTATION-PLAN-v0.md line 381. The case sits in `Appealed` until admin calls `admin_close_case`. Phase 4's `admin_close_case` already handles `Appealed` in its exhaustive match per Explore agent #3 §14 (`admin_close_case.rs:65-75`) — no change needed there.

**GOTCHA.** Original-reporter appeals deferred to v1. Current v0 only allows target to appeal. Doc comment must point at IMPLEMENTATION-PLAN-v0.md line 381 + ADR-010 v1 roadmap.

**GOTCHA.** `appeal.reason` column is `TEXT NOT NULL` (migration `add_governance_core/up.sql:47-55`, Appeal struct `crates/db_schema/src/source/governance/appeal.rs:17-25`). Empty reason is technically allowed by schema but the DTO type `RequestAppeal { reason: String }` requires a String (not `Option<String>`). Callers providing empty string store empty string — no additional validation.

**GOTCHA.** Appeal window closure: when does appeal become unrequestable? v0 answer is "when `case.closed_at IS NOT NULL`". Since admin_close_case sets `closed_at` and flips case to `Closed`, the sequence is: Decided → (target appeals) → Appealed → (admin reviews) → admin_close_case sets Closed + closed_at. If the target waits until after admin_close_case, `closed_at IS NOT NULL` blocks the appeal. **Decision-queue entry §17 Q3** if that's not the right policy.

**GOTCHA.** `AppealInsertForm` has four fields (`case_id, requester_id, reason, status`) — no `created_at`, no `decided_at`. DB defaults handle the former; the latter is NULL at insert.

**MIRROR.** `create_report.rs` for overall shape; `create_endorsement.rs:118-334` for the `run_transaction` wrapper pattern.

**VALIDATE.** §9 task 66 DoD.

---

### §11.7 Task 67 — `list_cases` handler + view-crate filter extension

**ACTION.** Two-part task with one architectural decision.

**67a — Decision.** `list_open_cases_for_community` (`db_views/governance_case/src/impls.rs:49-90`) currently takes `community_id: CommunityId` (NOT Option) and has a hardwired status filter (`Open | ThresholdMet | JurySelection | InReview`). `list_cases_needing_jury_selection` and `list_cases_for_person` each have one fixed filter. NONE of these accepts `{community_id: Option, status: Option, assignee: Option, page, limit}`. Two paths:

- **Path A** — add a new function `list_cases_filtered(pool, filter: CasesFilter) -> LemmyResult<Vec<GovernanceCaseSummaryView>>` that boxes the query and applies optional filters. Keeps existing three functions intact (callers unaffected).
- **Path B** — extend `list_open_cases_for_community` signature to accept `Option<CommunityId>` + `Option<CaseStatus>` + `Option<PersonId>`. Breaks existing callers (there is only one: the Phase 2a e2e test at `e2e.rs:471`; that's easy).

**Path A chosen.** Lower blast radius. Existing tests keep passing. New `list_cases_filtered` takes a single `CasesFilter` struct to avoid argument-count explosion.

**67b — Filter struct + function:**

```rust
// In db_views/governance_case/src/impls.rs
pub struct CasesFilter {
  pub community_id: Option<CommunityId>,
  pub status: Option<CaseStatus>,
  pub target_person_id: Option<PersonId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

pub async fn list_cases_filtered(
  pool: &mut DbPool<'_>,
  filter: CasesFilter,
) -> LemmyResult<Vec<GovernanceCaseSummaryView>> {
  // Build a boxed query, apply filters, run two-round-trip pattern
  // (main SummaryRow load + submitted_counts_by_case hydration).
  // Defaults: page=1, limit=20, max_limit=50 (mirror list_modlog.rs:26-28).
  // ...
}
```

**67c — Handler:**

```rust
// crates/api/api/src/governance/list_cases.rs
pub async fn list_cases(
  Query(data): Query<ListGovernanceCases>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,   // NOT Option — requires auth (per [04 §6.2])
) -> LemmyResult<Json<Vec<GovernanceCaseSummaryView>>> {
  check_local_user_valid(&local_user_view)?;
  let filter = CasesFilter {
    community_id: data.community_id,
    status: data.status,
    target_person_id: None,  // reserved — v1 adds assignee filter
    page: data.page,
    limit: data.limit,
  };
  let rows = list_cases_filtered(&mut context.pool(), filter).await?;
  Ok(Json(rows))
}
```

**GOTCHA.** `ListGovernanceCases` DTO has no `assignee` field (verified in §8 — `api_common/src/governance.rs:51-61` has `community_id, status, page, limit`). IMPLEMENTATION-PLAN-v0.md line 383 mentions `assignee` as a filter, but the DTO doesn't carry it. Treat assignee as v1 scope; document in handler doc comment.

**GOTCHA.** The `GovernanceCaseSummaryView` has drift stubs: `reporter_count: 0_i64`, `jury_needed: 5` hardcoded (per Explore agent #3 §6 + `04-data-model-and-api.md:441-444`). These stay as stubs; task 67 does NOT unstub them (v1 scope).

**GOTCHA.** `list_modlog.rs:30-47` pattern uses Rust-side `.skip()/.take()` over the full Vec. For `list_cases_filtered`, apply pagination at the SQL level (`.offset(offset).limit(limit)`) because the result set can be large (many thousands of cases in a mature instance). Mirror Lemmy's existing post/comment-list pagination.

**GOTCHA.** Pagination bounds: `DEFAULT_PAGE = 1`, `DEFAULT_LIMIT = 20`, `MAX_LIMIT = 50` — same as `list_modlog` for consistency. Clamp input before use.

**GOTCHA.** Auth requirement: [04 §6.2] implies auth-required for `list_cases` (contrast with `list_modlog` which is public). Phase 5c task 67 requires `LocalUserView` extractor (required, not Option). This means unauthenticated callers get actix's extractor-level 401 (`IncorrectLogin` via Lemmy's error bridge, mapping to 401 per §6 C8). Task 68's smoke test asserts this.

**GOTCHA.** Per-community permission filter may be required in v1 (don't expose case list from communities user doesn't belong to). v0 exposes all cases to any authenticated user — document as a v1 scope reservation in handler doc comment.

**MIRROR.** `list_modlog.rs:30-47` for handler shape; `list_open_cases_for_community` at `impls.rs:49-90` for the view-crate query pattern + `submitted_counts_by_case` hydration.

**VALIDATE.** §9 task 67 DoD (two crate checks — view crate + api crate).

---

### §11.8 Task 68 — route registration + `all_mvp_endpoints_return_non_404` smoke

**ACTION.** Three parts.

**68a — Route scope extension.** Edit `crates/api/routes/src/lib.rs`:

Add imports at lines 35-42 (existing `use lemmy_api::governance::{...}` block):
```rust
governance::{
  accept_jury_assignment::accept_jury_assignment,       // NEW (task 64)
  admin_assign_jury::admin_assign_jury,
  admin_close_case::admin_close_case,
  admin_reputation_stats::admin_reputation_stats,       // NEW (task 62)
  decline_jury_assignment::decline_jury_assignment,     // NEW (task 65)
  get_case::get_case,
  get_my_reputation::get_my_reputation,                 // NEW (task 61)
  list_cases::list_cases,                               // NEW (task 67)
  list_modlog::list_modlog,
  list_my_jury_queue::list_my_jury_queue,
  submit_jury_vote::submit_jury_vote,
},
```

Add import at line 138 (existing `use lemmy_api_crud::governance::{...}` block):
```rust
governance::{
  create_endorsement::create_endorsement,
  create_report::create_report,
  request_appeal::request_appeal,                       // NEW (task 66)
},
```

Extend the scope block at lines 505-522 to:
```rust
.service(
  scope("/governance")
    .wrap(rate_limit.post())
    .route("/report", post().to(create_report))
    .route("/endorsement", post().to(create_endorsement))
    .route("/appeal", post().to(request_appeal))                       // NEW (task 66)
    .route("/case", get().to(get_case))
    .route("/cases", get().to(list_cases))                             // NEW (task 67)
    .route("/modlog", get().to(list_modlog))
    .route("/reputation/me", get().to(get_my_reputation))              // NEW (task 61)
    .service(
      scope("/jury")
        .route("/me", get().to(list_my_jury_queue))
        .route("/accept", post().to(accept_jury_assignment))           // NEW (task 64)
        .route("/decline", post().to(decline_jury_assignment))         // NEW (task 65)
        .route("/vote", post().to(submit_jury_vote)),
    )
    .service(
      scope("/admin")
        .route("/assign-jury", post().to(admin_assign_jury))
        .route("/close-case", post().to(admin_close_case))
        .route("/reputation-stats", post().to(admin_reputation_stats)), // NEW (task 62)
    ),
),
```

**68b — Final endpoint count verification.**
- **11 MVP endpoints:** `POST /report`, `GET /case`, `GET /cases`, `GET /modlog`, `GET /reputation/me`, `POST /endorsement`, `GET /jury/me`, `POST /jury/accept`, `POST /jury/decline`, `POST /jury/vote`, `POST /appeal`. ✓
- **Admin backstops (not counted):** `POST /admin/assign-jury`, `POST /admin/close-case`, `POST /admin/reputation-stats`. ✓
- **`endorsement/revoke` is NOT routed** per [05 §2]. If task 68 accidentally adds it, remove.

**68c — e2e test `all_mvp_endpoints_return_non_404` + per-handler happy-path assertions (REVISED per Move 4 of risk-reduction strategy).** Per the original plan, the smoke test asserts every route returns `200 | 400 | 401` (not 404). Move 4 extends this by seeding minimal fixtures + authed requests for the 4 handlers whose only coverage would otherwise be route-level (61 `get_my_reputation`, 62 `admin_reputation_stats`, 66 `request_appeal`, 67 `list_cases`) — catching DTO shape bugs and SQL syntax errors that the non-404 smoke cannot. LOC cap bumped 250 → 320 for this task.

The smoke test body now has two phases: (A) the raw-route non-404 sweep (original 14-endpoint loop), then (B) 4 authed happy-path assertions with response-body decoding. Pattern:

```rust
// -- Phase B (NEW — Move 4): per-handler happy-path assertions --
// Seed fixtures ONCE, reuse across the 4 handlers.
let admin_view = seed_admin_local_user_view(&context).await?;
let admin_jwt = mint_test_jwt(&context, &admin_view).await?;
let user_view = seed_plain_local_user_view(&context, "probe_user").await?;
let user_jwt = mint_test_jwt(&context, &user_view).await?;
let target_view = seed_plain_local_user_view(&context, "probe_target").await?;
let case_decided = seed_case_at_status(&context, target_view.person.id, CaseStatus::Decided).await?;
let case_open = seed_case_at_status(&context, target_view.person.id, CaseStatus::Open).await?;

// B.1 — GET /reputation/me (task 61)
let resp = TestRequest::get().uri("/api/v4/governance/reputation/me")
    .insert_header(("authorization", format!("Bearer {user_jwt}")))
    .send_request(&app).await;
assert_eq!(resp.status(), 200, "reputation/me expected 200");
let body: GetMyReputationResponse = test::read_body_json(resp).await;
assert_eq!(body.view.active_sanctions, 0, "fresh user should have zero active sanctions");

// B.2 — POST /admin/reputation-stats (task 62)
let resp = TestRequest::post().uri("/api/v4/governance/admin/reputation-stats")
    .insert_header(("authorization", format!("Bearer {admin_jwt}")))
    .insert_header(("content-type", "application/json"))
    .set_payload("{}")
    .send_request(&app).await;
assert_eq!(resp.status(), 200, "admin/reputation-stats expected 200 for admin caller");
let body: AdminReputationStatsResponse = test::read_body_json(resp).await;
assert_eq!(body.buckets.jury_reliability.len(), 5, "jury_reliability bucket shape [i64; 5]");

// B.3 — POST /appeal (task 66) — target appeals a Decided case
let target_jwt = mint_test_jwt(&context, &target_view).await?;
let resp = TestRequest::post().uri("/api/v4/governance/appeal")
    .insert_header(("authorization", format!("Bearer {target_jwt}")))
    .insert_header(("content-type", "application/json"))
    .set_payload(format!(r#"{{"case_id":{},"reason":"probe"}}"#, case_decided.0))
    .send_request(&app).await;
assert_eq!(resp.status(), 200, "appeal expected 200 when target appeals a Decided case");
let body: RequestAppealResponse = test::read_body_json(resp).await;
assert!(body.appeal_id.0 > 0, "appeal_id must be a positive integer");

// B.4 — GET /cases (task 67) — any authed caller sees the one open case
let resp = TestRequest::get().uri("/api/v4/governance/cases")
    .insert_header(("authorization", format!("Bearer {user_jwt}")))
    .send_request(&app).await;
assert_eq!(resp.status(), 200, "cases expected 200 for any authed caller");
let body: Vec<GovernanceCaseSummaryView> = test::read_body_json(resp).await;
assert!(body.iter().any(|c| c.case.id == case_open), "seeded open case must appear in list");
```

**GOTCHA-68c-Move4-a.** `mint_test_jwt` helper: check whether a shipped helper already exists (grep `mint_jwt|test_jwt|issue_jwt` in `crates/api/api_utils/src/claims.rs` + `tests/e2e.rs`). If not, write a 10-line helper in the test module that calls `Claims::jwt(...)` with the test secret. Do NOT ship a public JWT helper — test-local only per §11.8 original "keep local" GOTCHA.

**GOTCHA-68c-Move4-b.** `seed_case_at_status(pool, target_person_id, CaseStatus::Decided)` — if the existing Phase 2a helpers don't cover this, add a 20-line helper in the test module that inserts `ModerationCase` + runs the status through to `Decided` via direct UPDATE (skip the vote loop for test speed).

**GOTCHA-68c-Move4-c.** Phase B runs AFTER Phase A's 404-sweep. Phase A uses unauthenticated requests and empty bodies (just checking routes exist). Phase B's assertions are additive, not replacement. If Phase A's sweep fails (any endpoint 404s), fail fast before Phase B runs.

**68c — original e2e test body.** The Phase A non-404 sweep:

```rust
#[tokio::test(flavor = "multi_thread")]
async fn all_mvp_endpoints_return_non_404() -> Result<(), Box<dyn std::error::Error>> {
  // Boot testcontainer Postgres + build actix App.
  // This test differs from Phase 4/5a/5b tests (which invoke handlers directly)
  // because it needs the actix routing layer to distinguish 404 from 401/400.
  //
  // Pattern: use `actix_web::test::{init_service, TestRequest}` with the same
  // lib.rs::config(cfg, &rate_limit) wiring the production server uses.

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  // ... DB setup (apply_all_schema, etc.)
  let pool = build_db_pool_for_tests_from_url(&db_url(host_port)).await?;
  let context = build_test_context(pool, ...).await?;
  let rate_limit = RateLimit::default();

  let app = actix_web::test::init_service(
    actix_web::App::new()
      .app_data(Data::new(context.clone()))
      .configure(|cfg| lemmy_api_routes::lib::config(cfg, &rate_limit))
  ).await;

  // 11 MVP endpoints + 3 admin backstops = 14 total.
  let cases = [
    ("POST", "/api/v4/governance/report",             "{}"),
    ("POST", "/api/v4/governance/endorsement",        "{}"),
    ("POST", "/api/v4/governance/appeal",             "{}"),
    ("GET",  "/api/v4/governance/case?case_id=1",     ""),
    ("GET",  "/api/v4/governance/cases",              ""),
    ("GET",  "/api/v4/governance/modlog",             ""),
    ("GET",  "/api/v4/governance/reputation/me",      ""),
    ("GET",  "/api/v4/governance/jury/me",            ""),
    ("POST", "/api/v4/governance/jury/accept",        "{}"),
    ("POST", "/api/v4/governance/jury/decline",       "{}"),
    ("POST", "/api/v4/governance/jury/vote",          "{}"),
    ("POST", "/api/v4/governance/admin/assign-jury",  "{}"),
    ("POST", "/api/v4/governance/admin/close-case",   "{}"),
    ("POST", "/api/v4/governance/admin/reputation-stats", "{}"),
  ];

  for (method, path, body) in cases {
    let req = match method {
      "GET"  => TestRequest::get().uri(path).to_request(),
      "POST" => TestRequest::post().uri(path)
                  .insert_header(("content-type", "application/json"))
                  .set_payload(body.to_string())
                  .to_request(),
      _ => unreachable!(),
    };
    let resp = actix_web::test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    assert!(
      matches!(status, 200 | 400 | 401),
      "{method} {path} returned {status} (expected 200/400/401, NOT 404)"
    );
  }
  Ok(())
}
```

**GOTCHA.** Per §6 C8 — `NotAnAdmin → 400`, not 403. So the admin endpoints without credentials return 400 (or 401 if unauthenticated extractor fires first). Assert `[200, 400, 401]`.

**GOTCHA.** Unauthenticated requests to handlers requiring `LocalUserView` (non-Option extractor) fail at the extractor layer with actix's 401. Endpoints with `Option<LocalUserView>` (e.g. `list_modlog`, `get_case`) always reach the handler — for `list_modlog` the handler returns 200 with an empty result; for `get_case` with no matching case, 404 (this is the only "real" 404 case).

**GOTCHA.** `get_case` returning 404 for a non-existent case is correct behaviour — but task 68's smoke uses `case_id=1` which doesn't exist in the test's empty DB. The assertion would fail. Options: (a) seed a case before the test, (b) exclude `get_case` from the iteration and assert it separately, (c) accept 404 for `get_case` specifically. **Decision: accept 404 for `get_case`** with a per-endpoint expected-status override:

```rust
let cases: &[(&str, &str, &str, &[u16])] = &[
  ("GET",  "/api/v4/governance/case?case_id=1", "", &[200, 400, 401, 404]),  // 404 allowed for empty DB
  ...
  // All others: no 404 allowed.
];
```

**GOTCHA.** Setting up actix `App` + `RateLimit` + `LemmyContext` for the test requires re-using the e2e harness helper pattern. Mirror the context-build block at `e2e.rs:838-845`. There is no `build_test_context` helper shipped; task 68 may need to author one if the inline pattern is too verbose. If so, keep it a local helper inside the test module — do NOT ship a public test harness (v1 scope).

**GOTCHA.** `lemmy_api_routes::lib::config(cfg, &rate_limit)` — the actual entry point is the `config` function exported from `crates/api/routes/src/lib.rs`. Check the fn name at plan-execute time (may be `api_config` or similar in Lemmy 1.0-beta). Explore agent #2 confirmed actix-web `scope(...)` + `.route(...)` is the pattern but didn't dump the exported config fn name. **Decision-queue entry §17 Q5** if the exported fn is named differently.

**MIRROR.** `lib.rs:505-522` for the scope block; actix-web upstream docs for `init_service` / `TestRequest` pattern.

**VALIDATE.** §9 task 68 DoD.

---

### §11.9 Task 69 — `ineligible_user_cannot_be_picked_for_jury` e2e (3 branches)

**ACTION.** Add one compound test to `e2e.rs`. Three branches in one `#[tokio::test]`.

**TEST SKELETON.**

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ineligible_user_cannot_be_picked_for_jury() -> Result<(), Box<dyn std::error::Error>> {
  // ... boot pg, apply schema, build context ...

  // -- Seed 7 users --
  // 5 "eligible": 200+ jury_reliability via founder-seed-style reputation_event rows
  //               (expires_at = now() + 30d; delta large enough to cross threshold 50).
  // 2 "ineligible": no reputation events; stay at 0.
  let eligibles: Vec<PersonId> = seed_eligible_users(&context, 5, 30).await?;
  let ineligibles: Vec<PersonId> = seed_ineligible_users(&context, 2).await?;

  // -- Run snapshot batch so jury_eligible flags are up-to-date --
  run_snapshot_batch(&context).await?;

  // -- Seed a case targeting someone outside both groups --
  let target_person = seed_person_plain(&context, "target").await?;
  let reporter = seed_person_plain(&context, "reporter").await?;
  let case_id = seed_case(&context, target_person, reporter).await?;

  // ============ BRANCH 1: basic capability gate ============
  let admin_view = seed_admin(&context).await?;
  let resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view.clone(),
  ).await?.into_inner();
  assert_eq!(resp.assigned_person_ids.len(), 5);
  for pid in &resp.assigned_person_ids {
    assert!(eligibles.contains(pid), "picked person {:?} is not in eligible set", pid);
    assert!(!ineligibles.contains(pid), "picked ineligible person {:?}", pid);
  }

  // ============ BRANCH 2: concurrent-cap ============
  // Pre-seed 3 active (Selected|Accepted) jury_assignment rows for eligibles[0].
  for other_case in seed_3_dummy_cases(&context, 3).await? {
    insert_assignment(&context, other_case, eligibles[0], JuryAssignmentStatus::Accepted).await?;
  }
  // New case, new assign-jury call.
  let case_id_2 = seed_case(&context, target_person, reporter).await?;
  let resp_2 = admin_assign_jury(
    Json(AdminAssignJury { case_id: case_id_2 }),
    context.clone(),
    admin_view.clone(),
  ).await?.into_inner();
  assert!(
    !resp_2.assigned_person_ids.contains(&eligibles[0]),
    "eligibles[0] at concurrent-cap of 3 was still picked"
  );

  // ============ BRANCH 3: config flip 3 → 5 ============
  // INSERT new governance_config row with value_int=5 and valid_from=now().
  flip_config(&context, "jury.max_concurrent_assignments", 5).await?;

  let case_id_3 = seed_case(&context, target_person, reporter).await?;
  let resp_3 = admin_assign_jury(
    Json(AdminAssignJury { case_id: case_id_3 }),
    context.clone(),
    admin_view.clone(),
  ).await?.into_inner();
  assert!(
    resp_3.assigned_person_ids.contains(&eligibles[0]),
    "after config flip to 5, eligibles[0] should be pickable (has 3 active)"
  );

  Ok(())
}
```

**GOTCHA.** "Seed 5 eligible / 2 ineligible (not the reverse) to avoid triggering the task 57 fallback to unfiltered pool." (IMPLEMENTATION-PLAN-v0.md line 387). The strict eligibility query returns at most `panel_size` (5); if it returns <5 the fallback engages. With 5 eligible and 2 ineligible, the strict query returns exactly 5, the fallback does not engage, and the 2 ineligibles stay out.

**GOTCHA.** `seed_eligible_users` inserts `reputation_event` rows with:
```rust
ReputationEventInsertForm {
  person_id,
  community_id: None,
  dimension: ReputationDimension::JuryReliability,
  delta: 60,  // > threshold 50
  expires_at: Some(Utc::now() + Duration::days(30)),
  reason: "founder_seed".to_string(),
  source_case_id: None,
  source_report_id: None,
}
```
Matches the pattern Phase 5b task 59's CLI was supposed to emit (see §2 blocker §2.1). If task 59 shipped, reuse its helper. If not, inline in test.

**GOTCHA.** `run_snapshot_batch(&context)` — exported from `reputation_snapshot.rs:361`. Call this between seeding events and running assign-jury, otherwise the snapshot row's `jury_eligible` flag is stale.

**GOTCHA.** `flip_config` helper — direct SQL INSERT:
```rust
async fn flip_config(context: &LemmyContext, key: &str, new_value: i64) -> LemmyResult<()> {
  let conn = &mut get_conn(&mut context.pool()).await?;
  diesel::sql_query(
    "INSERT INTO governance_config (scope, key, value_type, value_int, valid_from) \
     VALUES ('instance', $1, 'int', $2, now())"
  )
    .bind::<Text, _>(key)
    .bind::<BigInt, _>(new_value)
    .execute(conn).await?;
  Ok(())
}
```

**GOTCHA.** Each `admin_assign_jury` call creates a fresh `ConfigCache` internally (per-request, per §6 C10). After `flip_config`, the next call reads the new value.

**GOTCHA.** Branch 2 seeds 3 concurrent assignments for `eligibles[0]` at status=Accepted. The strict query's NOT IN subquery (`admin_assign_jury.rs:304-322`) filters persons with `count(*) >= $3` where `$3 = max_concurrent`. Default 3 → `eligibles[0]` excluded. After flip to 5, cap is 5, `count(*) = 3 < 5` → `eligibles[0]` pickable again.

**GOTCHA.** The 2 ineligibles never have reputation events, so their snapshot rows have `jury_eligible = false` after `run_snapshot_batch`. The strict query's `INNER JOIN reputation_snapshot ON jury_eligible = true` excludes them.

**GOTCHA.** If task 59 (founder CLI) shipped with Phase 5b, the seed_founders binary COULD be called from the test. Cleaner to inline in test to keep the test self-contained.

**MIRROR.** `report_to_modlog_golden_path` for overall structure + fixture pattern; `admin_assign_jury` for the handler call site.

**VALIDATE.** §9 task 69 DoD.

---

### §11.10 Task 69a — V2 messaging hooks (compound)

**ACTION.** This is the existing fragment at `.claude/PRPs/plans/phase-5c-task-69a-v2-hooks.fragment.md` folded in. Five sub-steps:

**69a.1 — Migration `add_governance_log_notify`.** Create `migrations/2026-04-20-000000-0000_add_governance_log_notify/up.sql` and `down.sql` per the fragment's body (§15 of the fragment). Trigger NAME: `governance_log_notify_trigger`. Function NAME: `governance_log_notify()`. Channel NAME: `governance_events`. Payload: 3-field JSON (entry_id, kind, published_at).

**GOTCHA.** Payload <8KB cap → deliberately no `payload` field in the NOTIFY. Subscribers fetch full rows separately.

**GOTCHA.** Channel name lowercase, no quoting. Stability contract — breaking change = downstream breakage.

**GOTCHA.** NOTIFY ephemeral → catch-up via `SELECT ... WHERE id > $last_seen_id`. Documented in SUBSCRIPTIONS.md.

**69a.2 — Create `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md`.** Copy the skeleton from the fragment §55-134. Include: channel name, payload shape, catch-up pattern, full list of v0 entry kinds (16 total now — the 14 existing + 4 new from tasks 64/65/66 + `emergency_removed`).

**69a.3 — e2e test `governance_events_notify_fires`.** Uses `tokio_postgres::Client` directly (workspace dep at `Cargo.toml:223`). Pattern:
```rust
let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await?;
tokio::spawn(connection);
client.batch_execute("LISTEN governance_events").await?;

// Via handler, insert a governance_log row by calling create_report (minimal fixture).
call_create_report_minimal(&context, reporter, target).await?;

// Collect notification with 1-second timeout.
let notif = tokio::time::timeout(
  std::time::Duration::from_secs(1),
  async { loop { if let Some(n) = client.transaction().await?.notifications().next().await { return Ok::<_, tokio_postgres::Error>(n); } } }
).await??;

let payload: serde_json::Value = serde_json::from_str(notif.payload())?;
assert_eq!(payload["kind"], "report_created");
```

**GOTCHA.** `tokio-postgres` notification API: use `client.notifications()` or async loop pattern. The exact shape depends on version — confirm at implementation time via `docs.rs/tokio-postgres/0.7.16`. Explore agent #2 confirmed `tokio-postgres = "0.7.16"` in workspace deps but the notification-stream API has changed across versions.

**GOTCHA.** LISTEN must be established **before** the INSERT happens or the test races. Sequence: LISTEN → insert (handler call) → await notification with timeout.

**GOTCHA.** Add `tokio-postgres` to `crates/server/Cargo.toml` `[dev-dependencies]` if not already there (Explore agent #2 noted it's currently only a runtime dep of `lemmy_diesel_utils`).

**69a.4 — e2e test `underscore_prefix_usernames_still_register`.** Per fragment:
```rust
let username = "_lemmy_test_user";
let person = seed_person_plain(&context, username).await?;
assert_eq!(person.name, username);
```

**GOTCHA.** `is_valid_actor_name` at `crates/utils/src/utils/validation.rs:39-55` (verified by Explore agent #2 §13): regex `^(?:[a-zA-Z0-9_]+|[0-9_\p{Arabic}]+|[0-9_\p{Cyrillic}]+)$`, min length 3, max 20. `_lemmy_test_user` is 16 chars, all in `[a-zA-Z0-9_]` — passes. Test is pure regression coverage.

**69a.5 — Plan-doc annotation.** Edit `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md`. Add a single paragraph to an appropriate section (the fragment suggests "What NOT to build in v0" or post-v0 transport discussion). Location: after §6 Monday-morning checklist OR immediately after the Phase 6 definition. Paragraph body from fragment §202-210:

> **Real-time transport (if proposed post-v0):** if a future phase proposes adding a real-time push channel for governance notifications (jury invitations, case status changes, etc.), default to Server-Sent Events (SSE) over WebSocket. SSE composes with HTTP caching, has simpler backpressure semantics, and works through the same middleware stack as the existing API. The V2 messaging bridge does not require Brehon's own RT transport — it polls `governance_log` or subscribes via Postgres NOTIFY per `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md`. See V2/messaging.md §8.3 for the full reasoning.

**GOTCHA.** The IMPLEMENTATION-PLAN-v0.md path is vendored into this fork per CLAUDE.md. Edit the fork-local copy at `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md`. The prp-plan workflow guidance says "don't edit homeserver's copy" — but since this file is vendored in-tree, the fork-local edit is authoritative.

**Commit message for 69a** (per fragment §242-256):
```
feat(governance): task 69a — Postgres NOTIFY on governance_log + V2 hooks

Reserves V2 messaging (post-v0) hooks per docs/brehon-law-inspired-network/V2/messaging.md §8:
- governance_log NOTIFY trigger (§8.4)
- SUBSCRIPTIONS.md channel contract
- e2e: governance_events_notify_fires
- e2e: underscore_prefix_usernames_still_register (§8.2)
- IMPLEMENTATION-PLAN-v0.md SSE-over-WS preference (§8.3)

No Rust handler changes. No new endpoint. Reversible migration.
```

**VALIDATE.** §9 task 69a DoD.

---

### §11.11 Task 70 — phase-close validation + report + PR

**ACTION.**
1. Run full §12 validation (Level 0–5).
2. Generate `.claude/PRPs/reports/phase-5c-complete-report.md` summarising: tasks shipped, DoD status, carry-forwards, deviations from plan.
3. Open PR: `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-5c --title "Phase 5c — Remaining endpoints, observability, capability tests" --body "$(cat ... EOF)"` per `.claude/rules/phase-branch.md` + `.claude/rules/gh-pr-fork-target.md`.
4. Do NOT merge — CodeRabbit + human review gate.

**DoD.**
- All 12 tasks committed on phase-5c.
- `cargo check --workspace --features full` exit 0.
- `cargo clippy --workspace --no-deps --features full -- -D warnings` exit 0.
- `cargo test --test e2e -p lemmy_server --features full` exit 0 (all 9-11 tests depending on task 63c + task 68c + task 69 + task 69a.3 + task 69a.4).
- `.claude/PRPs/reports/phase-5c-complete-report.md` written.
- PR opened against `governance-v0`.

---

## §12. Validation commands (Level 0–5)

**Level 0 — Plan audit.** Task 0 already runs; confirms wrapper probes pass + clippy baseline clean. Not re-run at phase-close.

**Level 1 — Static analysis.**
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/level1-check.log 2>&1"
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --no-deps --features full -- -D warnings > .claude/level1-clippy.log 2>&1"
```
Exit 0 both.

**Level 2 — Integration tests.**
```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server --features full > .claude/level2-e2e.log 2>&1"
```
Exit 0; expect 13-14 tests passing (previous 9 + 4-5 new).

**Level 3 — Full build.**
```bash
cmd //c "scripts\\brehon\\cargo-build.bat --workspace --features full > .claude/level3-build.log 2>&1"
```
Exit 0.

**Level 4 — Migration round-trip.**
```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server phase1_migrations_round_trip --features full > .claude/level4-migrations.log 2>&1"
```
Exit 0. Note: `phase1_migrations_round_trip` iterates through all migrations including the new `add_governance_log_notify` (task 69a.1). If the migration's `down.sql` is malformed, this test catches it.

**Level 5 — Cross-cutting verification.**
- Every new `governance_log::append` call uses an `ENTRY_KIND_*` constant (grep check).
- Every new handler that mutates state calls `actor_pseudonym_helper::get_or_create` before `append`.
- Every new `CaseStatus` match is exhaustive (review by hand).
- `grep -c 'local_private_message_before_create\|local_private_message_after_create\|local_private_message_before_update\|local_private_message_after_update\|federated_private_message_before_receive\|federated_private_message_after_receive' crates/` returns ≥6 (per `.claude/rules/pm-plugin-hooks-stable.md`).

---

## §13. Testing strategy

Per IMPLEMENTATION-PLAN-v0.md §5: **integration-only for v0, no unit tests until something breaks twice**. All tests in `crates/server/tests/e2e.rs`.

**Tests to add (5 new + 1 edited):**

| Test Name | Branch Count | Task |
|---|---|---|
| `ineligible_user_cannot_be_picked_for_jury` | 3 | 69 |
| `all_mvp_endpoints_return_non_404` | 14 endpoints | 68 |
| `capability_change_entries_reachable_via_modlog_crate` | 1 | 63 |
| `governance_events_notify_fires` | 1 | 69a.3 |
| `underscore_prefix_usernames_still_register` | 1 | 69a.4 |
| `report_to_modlog_golden_path` (edit) | existing + 5 accept calls | 64 |

**Edge cases:**
- Task 64: missing `Selected` assignment → 404. Caller is reporter → 404. Caller in sponsor cluster → 404.
- Task 65: no eligible replacement → `replacement_person_id: None`. Cap=0 users available → still decline the original, just no replacement.
- Task 66: case in non-`Decided` state → 404 (pseudo-403). `closed_at IS NOT NULL` → 404. Caller not target → 404.
- Task 67: pagination beyond result set → empty Vec. Filters that exclude everything → empty Vec.
- Task 69a.1: `governance_log_hash_chain_holds` (Phase 1 test) still passes — the NOTIFY trigger fires after the hash-chain trigger and does not interfere.

---

## §14. Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Phase 5b Slice C (tasks 59–60) not shipped before 5c starts | MED | MED | §2.1/§2.2 blocker checks in task 0; decision-queue intake A1/A2; 5c task 69 inlines founder seeding if CLI absent |
| `admin_assign_jury.rs:139` Accepted→Selected flip breaks Phase 4 golden-path test | LOW | HIGH | Task 64e regression test edit runs FIRST in 64's commit; §9 task 64 DoD re-runs `report_to_modlog_golden_path`; if test fails, commit does not land |
| `list_cases_filtered` Path A refactor introduces an n+1 query | MED | MED | Mirror `list_open_cases_for_community`'s two-round-trip pattern (main SummaryRow + `submitted_counts_by_case` hydration); keep SQL flat |
| `tokio-postgres` notification API shape mismatch vs 0.7.16 | MED | LOW | Task 69a.3 has an explicit "confirm API at implementation time" GOTCHA; fallback pattern in `.claude/decision-queue.json` §17 Q6 |
| `actix-web::test::init_service` config function name mismatch | LOW | MED | §17 Q5 decision-queue intake; alternative is to build `App` without `config()` entry point and inline the scope |
| `GovernanceModlogView` drift stubs (`decision`, `sanction_action` both `None`) surprise task 63's smoke test | LOW | LOW | Task 63b reads from `governance_log` directly (not `public_case_log`); the drift stubs don't apply |
| Config threshold changes cascade slow under load in large instances | LOW | MED | v1 concern (burst-collapse batching); v0 accepts "one log entry per affected user per tick" per IMPLEMENTATION-PLAN-v0.md line 375 |
| `create_report` auto-transitions case past `Decided` before task 66's window check runs | LOW | MED | Task 66 checks `case.status == Decided` explicitly; earlier-state cases → 404; concurrent transition is not a v0 concern |
| CodeRabbit flags the three-branch task 69 as too-complex | MED | LOW | One test with three asserts is ~80 lines; manageable. If flagged, split into three `#[tokio::test]` functions post-review |
| Task 69a migration timestamp collision with Phase 6 federation migration | LOW | LOW | Use timestamp `2026-04-20-000000-0000` (well before planned Phase 6); if Phase 6 needs earlier slot, rename 69a migration during rebase |
| **e2e suite races under parallelism** (Phase 5b carry-forward #3 — `SETTINGS` singleton caches first test's `LEMMY_DATABASE_URL`) | **MED** | **MED** | **Move 7 of risk-reduction strategy landed the `--test-threads=1` guard into `scripts/brehon/cargo-test.bat` BEFORE phase-5c cuts. Wrapper auto-appends flag when `--test e2e` is present without `--no-run` and without caller-override. No in-test change needed.** |
| **4 new handlers (61, 62, 66, 67) have no direct happy-path e2e** — only route-level non-404 smoke | **MED** | **MED** | **Move 4 of risk-reduction strategy extended task 68c with Phase B per-handler happy-path assertions: GET /reputation/me (user context), POST /admin/reputation-stats (admin context), POST /appeal (target appeals Decided case), GET /cases (authed caller sees seeded case). Catches DTO shape + SQL syntax bugs that a non-404 smoke cannot.** |
| **Task 64's line-139 flip + test edit in same commit could mask a broken test** | **LOW** | **HIGH** | **Move 3 of risk-reduction strategy reordered task 64: write test edit on throwaway LOCAL commit FIRST (64e-pre), verify it fails before the flip lands; apply flip; verify still fails; add handler; verify now passes; squash into single task-64 commit. Three independent verifications prove test has semantic teeth AND flip works AND handler is correct.** |
| **Task 64c `shares_active_sponsor` helper could be duplicated in task 65 if left inline** | **LOW** | **LOW** | **Move 5 of risk-reduction strategy extracts to new `crates/api/api/src/governance/jury_common.rs` at task-64 time. Task 65 imports from `jury_common`. Zero duplication; preempts CodeRabbit "duplicate logic" finding.** |
| **Task 62 CASE-WHEN bucketing, task 68c actix `init_service`, task 69a.3 tokio-postgres notifications — all rely on APIs with no existing fork mirror** | **MED** | **MED** | **Move 6 of risk-reduction strategy adds step 8 to task 0: three scratch probes in `scratch/phase-5c-probes/`, run in parallel, each ≤ 30 lines. Verify CASE-WHEN returns 5-row shape on postgres 18; verify actix `config` wiring returns 400 for empty POST; verify `Connection::poll_message` + channel bridge receives `NOTIFY hello 'world'` within 1-second timeout. Fails fast before task 1 iteration burns context on a broken assumption.** |
| **Authoritative DoD line 398 (snapshot staleness alert) has no task slot** | **LOW** | **MED** (would fail phase-close DoD evaluation) | **Move 2 of risk-reduction strategy allocated to task 63d (~20 lines in `scheduled_tasks.rs` per decision-queue #21). Emits `tracing::error!` when `MAX(reputation_snapshot.calculated_at) < now() - 2×snapshot_interval_seconds`. Test-coverage via extracted `check_snapshot_staleness` fn + `tracing-test` assertion.** |

---

## §15. Acceptance criteria

- [ ] All 11 MVP endpoints registered and callable (task 68 smoke test green).
- [ ] All 3 admin backstops registered: `assign-jury`, `close-case`, `reputation-stats`.
- [ ] `endorsement/revoke` NOT registered (v1 scope per [05 §2]).
- [ ] `report_to_modlog_golden_path` passes with 5 accept calls inserted (task 64 edit).
- [ ] `ineligible_user_cannot_be_picked_for_jury` passes all 3 branches (task 69).
- [ ] `all_mvp_endpoints_return_non_404` passes for all 14 routes (task 68).
- [ ] `capability_change_entries_reachable_via_modlog_crate` passes (task 63).
- [ ] `governance_events_notify_fires` passes within 1s timeout (task 69a.3).
- [ ] `underscore_prefix_usernames_still_register` passes (task 69a.4).
- [ ] `phase1_migrations_round_trip` still passes after adding the NOTIFY migration.
- [ ] `governance_log_hash_chain_holds` still passes (NOTIFY trigger doesn't interfere).
- [ ] `sponsor_liability_with_founder_multiplier` still passes if 5b task 60 shipped (else decision-queue carry-forward).
- [ ] `cargo check --workspace --features full` exit 0.
- [ ] `cargo clippy --workspace --no-deps --features full -- -D warnings` exit 0.
- [ ] `SUBSCRIPTIONS.md` committed with full entry-kind list.
- [ ] IMPLEMENTATION-PLAN-v0.md has SSE-over-WebSocket paragraph (task 69a.5).
- [ ] `.claude/PRPs/reports/phase-5c-complete-report.md` written.
- [ ] PR opened against `governance-v0` via `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-5c ...`.
- [ ] No ADR contradictions.
- [ ] No new `TODO(brehon-fork)` markers without referenced upstream issue number.
- [ ] PM plugin-hook count ≥6 (grep-based invariant per `.claude/rules/pm-plugin-hooks-stable.md`).

---

## §16. Plan correction policy

If during execution the plan diverges from reality (e.g. an Explore-agent finding proves incomplete, or a Phase 5b carry-forward unlocks):

1. **Surface via decision-queue** with a clear option set. Do NOT silently deviate.
2. If the deviation is minor (e.g. "the `list_cases_filtered` function name is `list_governance_cases_filtered`"), record it in the phase-close report §3 deviations block.
3. If the deviation is ADR-affecting, STOP the loop — require human sign-off before continuing.
4. If a task blocks on a Phase 5b gap (tasks 58-60 not merged), continue with the other tasks; mark the blocked task in decision-queue and return to it once unblocked. Do not conflate 5b and 5c merge histories.

---

## §17. Notes + decision-queue intake

**Risk-reduction strategy pre-resolves (2026-04-18).** Three of the seven original plan intake questions have been lifted to standalone `.claude/decision-queue.json` entries and answered BEFORE task 0 per Moves 1-2 of the risk-reduction strategy:

- **Q5 → decision-queue #19** — actix `config` fn name. **Answered:** `pub fn config(cfg: &mut ServiceConfig, rate_limit: &RateLimit)` at `crates/api/routes/src/lib.rs:201`. Plan §11.8 68c wiring is correct.
- **Q6 → decision-queue #20** — tokio-postgres 0.7.16 notifications API. **Answered:** poll-based via `Connection::poll_message()`. Plan §11.10 69a.3 snippet (using `client.notifications().next().await`) is **wrong for this version** and needs rewrite to use `poll_message` + mpsc channel bridge. Implementer adjusts at task 69a time using the pattern in #20's answer body.
- **Q8 (new) → decision-queue #21** — DoD line 398 snapshot-staleness alert placement. **Answered:** fold into task 63 as step 63d (~20 lines). Plan §11.3 updated.

**Remaining intake (keep as task-0 decision-queue pre-seed, or impl-self-resolve using recommendations):**

**Q1 (A1 per §2.1) — Phase 5b task 59 CLI status.** Has `crates/tools/seed_founders/` shipped? If no, task 69's third branch uses inline `reputation_event` INSERT instead of calling the CLI. Options: (a) inline seeding in test, (b) wait for task 59. Recommendation: (a) to avoid stalling 5c on 5b tail. (Note: §2.1 now says Phase 5b all shipped; Q1 likely resolves to "yes CLI exists, but inline is still cleaner for self-contained test".)

**Q2 (A2 per §2.2) — Phase 5b task 60 regression guard.** Does `sponsor_liability_with_founder_multiplier` exist in `e2e.rs`? If no, 5c's DoD line 396 cannot be evaluated. Options: (a) drop that DoD row for 5c, (b) wait for task 60, (c) write a minimal placeholder test in 5c task 0. Recommendation: (a) was original; now (b) moot because §2.2 confirms task 60 shipped. Re-run at task 0 baseline smoke to confirm still green.

**Q3 — Appeal window policy.** §11.6 task 66 guards on `case.closed_at.is_none()`. Is that the correct appeal-window definition? IMPLEMENTATION-PLAN-v0.md line 381 says "Verify `now() < case.closed_at`" which is the OPPOSITE — requires closed_at to be future-dated. But case closure in v0 is admin-driven (no automatic close timer), so `closed_at` is either None (case still open-decided) or set-to-now() after admin close. The plain reading of "now() < closed_at" doesn't fit v0 semantics. Options: (a) `closed_at IS NULL` as the window (current §11.6 plan), (b) add a `decided_at + 7 days` computed window, (c) defer to admin-flip-based window. Recommendation: (a), document as a v0 simplification.

**Q4 — Task 62 observability-query log entry.** Should `admin_reputation_stats` emit a log entry when called? Pro: audit trail of admin queries. Con: none of the other admin backstops log queries (only writes). Recommendation: no log, add `TODO(brehon-fork): audit-log admin queries in v1` comment.

**Q7 — Cosmetic entry-kind literal migration.** The `governance_log.rs` comment (lines 46-48) flags that Phase 4 string-literal emit sites (`"report_created"` etc.) should migrate to the consts in a Phase 5c cosmetic task. Should task 70 (phase-close) include this cleanup? Options: (a) yes, bundle with phase-close, (b) separate cosmetic commit in 5c, (c) defer to v1. Recommendation: (c) — scope creep; the literals are functionally identical to the consts. Document as carry-forward.

**Intake commits:** Task 0 verifies decision-queue entries #19, #20, #21 are already resolved (from Moves 1-2). Q1-Q4 + Q7 may be impl-self-resolved at task 0 time using the recommendations above, OR pre-seeded for advisor pickup. No blocker if impl chooses self-resolve for Q1-Q4 + Q7 (they are informational / v0-simplification choices, not compile-time gates).

---

## End of plan

**Prior HEAD at plan-write:** `phase-5b @ 1682a544f` (2026-04-17). **Expected 5c branch HEAD:** `governance-v0 @ f20728323` (2026-04-18) or later — after the risk-reduction strategy lands `chore(scripts): enforce --test-threads=1 for e2e` + the plan-file Move 2-6 edits on governance-v0.

**Confidence score (updated 2026-04-18 after risk-reduction strategy):** 9/10 for one-pass implementation success.

**Confidence rationale:**
- +2 — Every target file exists with verified line numbers from 3 Explore agents.
- +2 — All 5 input DTOs are already shipped (no api_common churn risk).
- +2 — `select_eligible_jurors` already has the `exclude_person_ids` parameter pre-wired by Phase 5b task 57.
- +2 — `governance_log::append` auto-scrubs via `scrub_json`, so task 66's user-text `reason` is safe by default.
- +1 — Phase 5b fully merged + cherry-picked + retro'd at eaa413cd8; §2.1/§2.2 blockers resolved.
- +1 — Decision-queue #19 (actix config fn), #20 (tokio-postgres poll-based), #21 (staleness alert) all resolved BEFORE task 0 per risk-reduction Moves 1-2.
- +1 — Move 3 flip-verification sequence eliminates the "test edit masks broken flip" risk on task 64.
- +1 — Move 4 per-handler happy-path assertions close the DTO shape + SQL syntax coverage gap for tasks 61, 62, 66, 67.
- +1 — Move 5 extracts `shares_active_sponsor` to `jury_common.rs` at task-64 time, preempting CodeRabbit duplicate-logic finding.
- +1 — Move 6 external-API probes catch broken assumptions at task-0 boundary, not mid-task.
- +1 — Move 7 `--test-threads=1` wrapper guard eliminates the Phase 5b e2e race class before 5c starts.
- -1 — Two compound tasks still (68 LOC cap 320, 69 3-branch test); if Slice A ralph iteration count exceeds 8, apply Move 8 and split Slice B off naturally at task 68 boundary.
- -1 — tokio-postgres `Connection::poll_message` + channel bridge pattern is new to the fork; scratch probe at task 0 step 8 de-risks but does not eliminate unknowns.

**Next step.** Advisor confirms governance-v0 tip includes the Move 7 wrapper edit + Move 2-6 plan edits, cuts phase-5c from the new tip, and kicks off `/prp-ralph-slice "phase-5c-remaining-endpoints-and-observability"` — OR per-task `/prp-implement` runs. Risk-reduction strategy reference: `C:\Users\barri\.claude\plans\what-would-be-a-proud-balloon.md`.
