# IMPLEMENTATION-PLAN-v0.md — Brehon-Law-Inspired Network MVP

**Status:** Phase 0 + 1 complete, Phase 2a ready to start (2026-04-15). Phase 5 reframe applied 2026-04-17 — split into 5a/5b, config-table posture, founder seeding, OQs 004/006/013/014 resolved.
**Audience:** Solo developer (you)
**Scope:** v0 only. v1/v2/v3 work referenced from [99 ADR-010](99-decisions-and-open-questions.md) and [05 §7](05-mvp-and-delivery-plan.md) is **out of scope**.

## Phase status

| Phase | Name | Status | Notes |
|---|---|---|---|
| 0 | Test harness (preflight) | ✅ done 2026-04-14 | `cargo test --test e2e` passes in ~41s. testcontainers + pgautoupgrade:18-alpine. libpq via vcpkg on Windows. |
| 1 | Schema + Diesel foundation | ✅ done 2026-04-15 | 14 tasks + Level 3 round-trip test + lockfile chore + plan reframe = 18 commits. Merged to `origin/governance-v0` HEAD `94eba51a0`. `phase1_migrations_round_trip` now a permanent test fixture. |
| 2a | Read models — `governance_case` + `jury_queue` | ⏭ ready | 11 tasks (14–24). Fresh ralph session. Plan generated externally. |
| 2b | Read models — `governance_modlog` + smoke tests | 🔒 blocked on P2a | 6 tasks (25–30). Smoke tests in task 30 gate the whole Phase 2. |
| 3 | API common DTOs | 🔒 blocked on P2b | 7 tasks (31–37). |
| 4 | First 5 endpoints + golden path | 🔒 blocked on P3 | 12 tasks. Human-in-the-loop per Phase 0 assessment. |
| 5a | Config + reputation infra + founder bootstrap | 🔒 blocked on P4 | 10 tasks (50–60). Introduces `governance_config` table; everything tuneable lives there. Founder seeding CLI + sponsor-liability e2e test anchor the phase. |
| 5b | Remaining endpoints + observability + capability tests | 🔒 blocked on P5a | 8 tasks (61–69). Ships all 11 MVP endpoints, `admin/reputation-stats` observability, capability-gating e2e. |
| 6 | Federation (outbound-only) | 🔒 blocked on P5b | 9 tasks (70–78). |

This plan is the executable blueprint for v0. It is keyed to the 11-endpoint MVP scope in [05 §2](05-mvp-and-delivery-plan.md), the 6-step implementation order in [05 §4](05-mvp-and-delivery-plan.md), and the canonical agent prompt at [AGENT-PROMPT-mvp-implementation-plan.md](AGENT-PROMPT-mvp-implementation-plan.md).

If anything below contradicts an ADR in [99](99-decisions-and-open-questions.md), the ADR wins and the contradiction belongs in §8 of this plan, not in the body.

---

## 1. Executive summary

v0 ships the **vertical slice** of the Brehon governance mechanic on top of a fork of [Lemmy 1.0-beta](https://github.com/LemmyNet/lemmy) ([99 ADR-012](99-decisions-and-open-questions.md)). One member can report another's content; weighted reports open a moderation case; an admin backstop assigns a 5-juror panel; jurors vote; a simple-majority decision creates one sanction row; a public, redacted entry lands in the modlog; reputation events ripple to jurors, reporters, and (once Step 5 lands) sponsors. All 11 endpoints in [05 §2](05-mvp-and-delivery-plan.md) return 200 on the happy path; an integration test exercises the full report → decision → log → modlog flow against a real Postgres in Docker; the local hash chain over `public_case_log` and the governance log table is verifiable end-to-end. **Done is when [05 §9](05-mvp-and-delivery-plan.md) "Done-definition for v0 ship" is checked off** — feature-complete but not security-hardened (security hardening is v2 per [99 ADR-010](99-decisions-and-open-questions.md)).

Effort order-of-magnitude for a solo developer: **8–12 weeks** end-to-end, see §9.

---

## 2. Pre-flight

These are one-time decisions and setup tasks that must happen before Step 1 of [05 §4](05-mvp-and-delivery-plan.md). Several need a human call.

### 2.1 Repo decision — ✅ RESOLVED (2026-04-14): Option A

**Decided:** Option A — separate repo. The fork lives at [`barrie-cork/lemmy`](https://github.com/barrie-cork/lemmy) on branch `governance-v0`. Local clone at `C:\Users\barri\Developer\brehon-fork\`. The `homeserver` repo keeps the design docs (this directory) as the canonical source; the fork vendors a copy under `docs/brehon-law-inspired-network/` which may drift — treat homeserver as authoritative.

Historical options below for context:

| Option | Pros | Cons |
|---|---|---|
| **A. Separate repo** (chosen) — fork Lemmy 1.0-beta on GitHub, clone locally, develop independently | Clean separation from `homeserver` infra repo. Easy AGPLv3 source-disclosure compliance ([99 ADR-011](99-decisions-and-open-questions.md)) — the fork is its own thing. Upstream rebases are isolated. CI lives in the fork. | Two repos to context-switch between. Cross-repo refs in docs. |
| **B. Subtree under `homeserver`** (rejected) | Single working directory. No context switching. | Mixes AGPLv3 fork with non-AGPLv3 infra-as-code repo — license boundary becomes confusing. Upstream rebases are awkward inside a subtree. Loses GitHub fork lineage. |

### 2.2 Project name — DEFER

[99 OQ-012](99-decisions-and-open-questions.md): final name not chosen. Working title "brehon-fork" is fine for the first weeks. Rename before first public push (which is a v0-ship event, not a Step-1 event).

### 2.3 Dev environment — checklist

Before writing code, make sure these are installed and working on the dev machine:

- [ ] Rust stable (whatever Lemmy 1.0-beta's `rust-toolchain.toml` pins — accept it, don't fight it)
- [ ] `cargo` + `rustup`, `rustfmt`, `clippy` components
- [ ] Postgres 16 client tools (`psql`, `pg_dump`)
- [ ] Docker + Docker Compose (Lemmy's existing dev compose file works)
- [ ] `diesel_cli` with the postgres feature: `cargo install diesel_cli --no-default-features --features postgres`
- [ ] Lemmy 1.0-beta's existing Cargo workspace builds clean (`cargo check --workspace`) before any governance code is added — establishes a known-good baseline
- [ ] Postgres test database container starts and `diesel migration run` against the upstream schema works

### 2.4 Initial repo setup tasks (one-shot, before Step 1 starts)

1. Fork Lemmy 1.0-beta on GitHub (Option A above)
2. Clone, create branch `governance-v0`
3. Verify upstream `cargo check --workspace` passes
4. Add `LICENSE` if not already present (AGPLv3 — should be inherited from upstream; just verify and don't overwrite)
5. Add an `AGPL-NOTICE.md` at repo root explaining the fork relationship and source-disclosure obligation per [99 ADR-011](99-decisions-and-open-questions.md)
6. Create the empty governance directory skeleton (no `.rs` files yet, just `mod.rs` placeholders) so the structure is in version control before any code lands:
   - `crates/db_schema/src/source/governance/`
   - `crates/db_views/governance_case/`
   - `crates/db_views/jury_queue/`
   - `crates/db_views/reputation/`
   - `crates/db_views/governance_modlog/`
   - `crates/api/api_common/src/governance.rs` (empty `pub mod governance;` in `lib.rs`)
   - `crates/api/api/src/governance/`
   - `crates/api/api_crud/src/governance/`
   - `crates/apub/objects/src/governance/`
   - `crates/apub/activities/src/governance/`
   - `crates/apub/apub/src/governance/`
   - `crates/server/src/governance.rs`

   Crate paths come from [03 §7](03-architecture.md) and [04 §3, §4, §6, §9–§12](04-data-model-and-api.md).
7. Create `tests/e2e.rs` placeholder file (just the test harness boilerplate; tests come in Step 4)
8. Create `migrations/` directory under `crates/db_schema/` if Lemmy doesn't already use it (it does — verify the convention before assuming)
9. Wire CI early: GitHub Actions workflow that runs `cargo check --workspace` + `cargo clippy --workspace -- -D warnings` + `cargo test -p server --test e2e` on every push. **Don't defer CI — solo devs without CI ship rot.**

### 2.5 Extism plugin system — confirm it exists

[99 ADR-012](99-decisions-and-open-questions.md) commits us to using Lemmy 1.0-beta's Extism-based plugin system "where it simplifies governance hooks." For v0, we will **not** ship governance hooks as Extism plugins — they are direct Rust code in the new crates. But before Step 1 starts, **read the Lemmy 1.0-beta Extism plugin docs and source** to confirm the `before_*` / `after_*` hook surface ([99 ADR-012](99-decisions-and-open-questions.md)) actually exists and is documented. If it does not, file an open question and decide in v1 whether to wire governance through plugins or stay with direct Rust.

This is a Step-0 verification, not a Step-1 task. Time-box to half a day.

---

## 3. Phase-by-phase task breakdown

Each phase corresponds 1:1 to a step in [05 §4](05-mvp-and-delivery-plan.md). Tasks are numbered, sized to one commit, and reference [04](04-data-model-and-api.md) for struct/enum definitions. Definitions of done come from [05 §4](05-mvp-and-delivery-plan.md).

> **Cross-cutting note:** every phase emits hash-chain log entries and uses `actor_pseudonym` where the governance log is touched. The mechanics live in §4 of this plan so they don't bloat each phase. If you find yourself implementing them per phase, stop and pull them up into the cross-cutting layer.

### Phase 1 — Schema + Diesel foundation (Step 1 of [05 §4](05-mvp-and-delivery-plan.md))

**Goal:** `moderation_case`, `jury_assignment`, `jury_vote`, `sanction`, `public_case_log` exist as tables, have Diesel models, and can be inserted/queried by an integration test.

**Tasks:**

1. **Migration `add_governance_core`** — write `up.sql` and `down.sql` for [04 §1 Migration 1](04-data-model-and-api.md): creates `moderation_case`, `case_evidence`, `sanction`, `appeal`, `public_case_log`. Files: `crates/db_schema/migrations/{timestamp}_add_governance_core/{up,down}.sql`. Indexes from [04 §1.5](04-data-model-and-api.md): `moderation_case(status, created_at)`, `public_case_log(community_id, published_at)`.

2. **Migration `add_jury_system`** — [04 §1 Migration 2](04-data-model-and-api.md): `jury_pool`, `jury_assignment`, `jury_vote`. Index: `jury_assignment(person_id, status)`. Files: `crates/db_schema/migrations/{timestamp}_add_jury_system/{up,down}.sql`.

3. **Migration `add_reputation_and_surety`** — [04 §1 Migration 3](04-data-model-and-api.md): `surety`, `endorsement`, `reputation_event`, `reputation_snapshot`. Index: `reputation_event(person_id, community_id, created_at)`. Files: `crates/db_schema/migrations/{timestamp}_add_reputation_and_surety/{up,down}.sql`. **Created in Phase 1 even though Phase 5 wires the logic** — having all four migrations land at the same time avoids schema drift mid-build.

4. **Migration `add_actor_pseudonym`** — per [99 ADR-015](99-decisions-and-open-questions.md) and [04 §3 ActorPseudonym](04-data-model-and-api.md). Table: `actor_pseudonym(id, person_id FK, pseudonym TEXT UNIQUE NOT NULL, created_at)`. Cryptographically-random pseudonym generation enforced at the Rust layer, not the DB. Files: `crates/db_schema/migrations/{timestamp}_add_actor_pseudonym/{up,down}.sql`. **GDPR-mandatory; do not skip.**

5. **Migration `add_governance_log`** — the append-only signed log table (no detail in [04](04-data-model-and-api.md) §1 because [03 §6](03-architecture.md) only describes the interface). Suggested shape: `governance_log(id BIGSERIAL PRIMARY KEY, prev_hash BYTEA NOT NULL, entry_hash BYTEA NOT NULL, entry_kind TEXT NOT NULL, payload JSONB NOT NULL, actor_pseudonym TEXT, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), signature BYTEA)`. **Append-only** is enforced at two layers: (a) a Postgres trigger that raises on `UPDATE` or `DELETE`, (b) a DB grant that denies `UPDATE`/`DELETE` to the app role. Files: `crates/db_schema/migrations/{timestamp}_add_governance_log/{up,down}.sql`.

6. **Hash-chain trigger** — Postgres trigger function `governance_log_hash_chain()` that computes `entry_hash = sha256(prev_hash || entry_kind || payload || created_at)` on `BEFORE INSERT`. The trigger reads the latest row's `entry_hash` to use as `prev_hash`. Per [03 §6](03-architecture.md): tamper-evident chain. Signing is **deferred to a Rust post-insert step** in Phase 4 so the trigger stays pure SQL. File: in Migration 5's `up.sql`.

7. **Migration `add_governance_enums`** — creates the Postgres enum types matching [04 §2](04-data-model-and-api.md): `case_status`, `case_target_type`, `case_severity`, `evidence_visibility`, `jury_assignment_status`, `jury_decision`, `sanction_scope`, `sanction_action`, `appeal_status`, `reputation_dimension`, `attestation_type`. **`case_status` MUST include `EmergencyRemove`** per [99 ADR-013](99-decisions-and-open-questions.md). Files: `crates/db_schema/migrations/{timestamp}_add_governance_enums/{up,down}.sql`. Order this **before** Migration 1 in the timestamp sequence — the tables in 1 reference these enums.

8. **Diesel enum types** — under `crates/db_schema/src/source/governance/enums.rs`. Use `diesel-derive-enum` (already in the workspace per [04 §2](04-data-model-and-api.md)). Re-export from `crates/db_schema/src/lib.rs`. Each enum from [04 §2](04-data-model-and-api.md) gets a `#[derive(DbEnum)]` definition. **`CaseStatus::EmergencyRemove` is non-negotiable** ([99 ADR-013](99-decisions-and-open-questions.md), [02 §3.1](02-domain-model.md)).

9. **Diesel models for the five core tables** — per [04 §3](04-data-model-and-api.md): `ModerationCase` + `ModerationCaseInsertForm`, `CaseEvidence`, `Sanction`, `Appeal`, `PublicCaseLog`. Files: `crates/db_schema/src/source/governance/{moderation_case,case_evidence,sanction,appeal,public_case_log}.rs`. Plus `schema.rs` regen and `mod.rs` exports.

10. **Diesel models for the jury tables** — per [04 §3](04-data-model-and-api.md): `JuryAssignment`, `JuryVote`. Plus a placeholder `JuryPool` if it gets used by the snapshot. Files: `crates/db_schema/src/source/governance/{jury_pool,jury_assignment,jury_vote}.rs`.

11. **Diesel models for reputation, surety, pseudonym** — per [04 §3](04-data-model-and-api.md): `Surety`, `Endorsement`, `ReputationEvent`, `ReputationSnapshot`, `ActorPseudonym`. **Created in Phase 1 even though Phase 5 uses them.** Files: `crates/db_schema/src/source/governance/{surety,endorsement,reputation_event,reputation_snapshot,actor_pseudonym}.rs`.

12. **First integration smoke test** — `tests/e2e.rs::can_insert_moderation_case`. Spins up Postgres in Docker (or uses the dev container), runs all migrations, inserts a `ModerationCase` via `ModerationCaseInsertForm`, queries it back, asserts the row matches. **One commit per test.**

13. **Hash-chain unit-of-test** — `tests/e2e.rs::governance_log_hash_chain_holds`. Inserts three log rows with mock payloads, reads them back, recomputes the chain in Rust, asserts the stored `entry_hash` values match. Verifies the Postgres trigger from task 6 actually works.

**Dependencies:** none — this is the first phase.

**Definition of done** (from [05 §4 Step 1](05-mvp-and-delivery-plan.md)): migrations run forward and backward without errors (`diesel migration run` / `diesel migration redo`), all models compile (`cargo check -p db_schema`), the smoke test in task 12 passes, the hash-chain test in task 13 passes.

---

### Phase 2 — Read models (Step 2 of [05 §4](05-mvp-and-delivery-plan.md))

**Goal:** `db_views/governance_case`, `db_views/jury_queue`, `db_views/governance_modlog` crates exist with the view structs from [04 §4](04-data-model-and-api.md) and at least one query each that returns data against a seeded test DB.

**Split into two sub-phases** (decided 2026-04-15 after Phase 1 retro — Phase 1 burned ~360k tokens in messages across 14 tasks, which is past the ~200k effective-reasoning threshold; splitting 2 into 2a + 2b keeps each ralph loop in a fresh context):

- **Phase 2a — `governance_case` + `jury_queue` crates** (tasks 14–24, 11 tasks). These are the two read-model crates Phase 4's endpoints depend on most heavily. Ships to `governance-v0` independently.
- **Phase 2b — `governance_modlog` crate + smoke tests** (tasks 25–30, 6 tasks). Includes the three smoke tests in task 30 that validate all three crates together — so 2b is the gate that proves 2a's work as well as its own.

Each sub-phase runs its own `/prp-ralph` invocation on a fresh session, merges to `governance-v0` via fast-forward, and closes cleanly before the next starts. Commit-message convention from Phase 2a task 1 onward is `feat(scope): task N — <summary>` per the Phase 1 retro.

**Tasks:**

14. **Create `crates/db_views/governance_case`** — new crate. `Cargo.toml`, `src/lib.rs`. Add to workspace `Cargo.toml` members. Per [04 §4.1](04-data-model-and-api.md) and [03 §7.1](03-architecture.md).

15. **`GovernanceCaseSummaryView` + `GovernanceCaseDetailView` structs** — per [04 §4.1](04-data-model-and-api.md). File: `crates/db_views/governance_case/src/lib.rs`.

16. **Query `list_open_cases_for_community`** — per [04 §4.1](04-data-model-and-api.md) "Queries". Returns `Vec<GovernanceCaseSummaryView>` filtered by `community_id` and `status IN (Open, ThresholdMet, JurySelection, InReview)`. File: `crates/db_views/governance_case/src/lib.rs` (or `queries.rs` if it grows).

17. **Query `read_case_detail`** — returns `GovernanceCaseDetailView` for one `case_id`. **Permission-aware: this view is consumed by `get_case` in Phase 4, which decides whether to return public/juror/admin shape.** The view function itself just hydrates everything; redaction is the handler's job.

18. **Query `list_cases_for_person`** — for "what cases am I a target of" UI. Filters by `target_person_id`.

19. **Query `list_cases_needing_jury_selection`** — for the jury-selection background job in Phase 4. `WHERE status = 'ThresholdMet'`.

20. **Create `crates/db_views/jury_queue`** — new crate. `Cargo.toml`, `src/lib.rs`. Per [04 §4.2](04-data-model-and-api.md).

21. **`JuryQueueView` struct** — per [04 §4.2](04-data-model-and-api.md). File: `crates/db_views/jury_queue/src/lib.rs`.

22. **Query `list_jury_assignments_for_person`** — joins `jury_assignment` to `moderation_case` and the community name. Powers `GET /api/v4/governance/jury/me`.

23. **Query `list_available_jury_cases_for_person`** — for v0 stays simple; will grow in Phase 5 with reputation gating.

24. **Query `count_unsubmitted_jury_assignments`** — used by the assignment-timeout background job.

25. **Create `crates/db_views/governance_modlog`** — new crate. Per [04 §4.4](04-data-model-and-api.md).

26. **`GovernanceModlogView` struct** — per [04 §4.4](04-data-model-and-api.md). File: `crates/db_views/governance_modlog/src/lib.rs`.

27. **Query `list_public_case_log`** — paginated, all communities. Powers `GET /api/v4/governance/modlog`.

28. **Query `list_public_case_log_for_community`** — filtered by `community_id`.

29. **Query `read_public_case_log_entry`** — single-entry read.

30. **Smoke tests for each view** — `tests/e2e.rs::list_open_cases_returns_seeded_rows`, `tests/e2e.rs::jury_queue_view_returns_assignments`, `tests/e2e.rs::modlog_view_returns_published_entries`. Each test seeds the DB via direct Diesel inserts (not via API), then queries through the view, asserts shape.

**Dependencies:** Phase 1 (migrations + models). The `db_views` crates depend on `db_schema`.

**Definition of done** (from [05 §4 Step 2](05-mvp-and-delivery-plan.md)): each view has at least one query that returns data against a seeded test DB; tests in task 30 pass.

> **Defer to Phase 5:** `crates/db_views/reputation`. The reputation views aren't needed for the Step-4 vertical slice ([05 §4 Step 4](05-mvp-and-delivery-plan.md)). They land in Phase 5 with the rest of the reputation work to keep Phase 2 narrow.

---

### Phase 3 — API common DTOs (Step 3 of [05 §4](05-mvp-and-delivery-plan.md))

**Goal:** every request/response type from [04 §5](04-data-model-and-api.md) compiles in `crates/api/api_common/src/governance.rs` and is exported.

**Tasks:**

31. **Create `crates/api/api_common/src/governance.rs`** — new module file. Wire it via `pub mod governance;` in `crates/api/api_common/src/lib.rs`.

32. **Reports + cases DTOs** — per [04 §5](04-data-model-and-api.md): `CreateGovernanceReport`, `CreateGovernanceReportResponse`, `GetGovernanceCase`, `ListGovernanceCases`. Derive `Serialize`, `Deserialize`. If Lemmy 1.0-beta uses `ts-rs` for client-type generation, derive `TS` too.

33. **Jury DTOs** — `SubmitJuryVote`, `AcceptJuryAssignment`, `DeclineJuryAssignment`. Per [04 §5](04-data-model-and-api.md).

34. **Appeals DTOs** — `RequestAppeal`. Per [04 §5](04-data-model-and-api.md).

35. **Reputation/trust DTOs** — `GetMyReputation`, `CreateEndorsement`, `RevokeEndorsement`. Per [04 §5](04-data-model-and-api.md). **`RevokeEndorsement` is in v0 even though the endpoint is deferred** ([05 §2](05-mvp-and-delivery-plan.md) — `endorsement/revoke` is sprint-2). It costs nothing to define the DTO now.

36. **Public log DTO** — `ListGovernanceModlog`. Per [04 §5](04-data-model-and-api.md).

37. **Compile + ts-rs export check** — `cargo check -p api_common` passes; if `ts-rs` is in use, `cargo test -p api_common --features ts-rs-export` regenerates the TS types and the diff is reviewed.

**Dependencies:** Phase 1 (some DTOs reference enums from `db_schema` — `CaseStatus`, `CaseTargetType`, `JuryDecision`, etc.).

**Definition of done** (from [05 §4 Step 3](05-mvp-and-delivery-plan.md)): DTOs compile, are exported, frontend type generation works if applicable.

---

### Phase 4 — First five endpoints (end-to-end slice) (Step 4 of [05 §4](05-mvp-and-delivery-plan.md))

**Goal:** the **true MVP slice** is wired end-to-end. Two test users → user A reports user B's post → admin backstop assigns 5 jurors → jurors vote → decision recorded → public log entry exists → modlog endpoint returns it. This is the phase that proves the governance mechanic works.

**Tasks:**

38. **Routes module** — `crates/api/routes/src/governance.rs`. Registers the route tree under `/api/v4/governance/` per [04 §7](04-data-model-and-api.md). For Phase 4, only the first five endpoints + the two admin backstops are wired; the rest get added in Phase 5.

39. **`POST /api/v4/governance/report` → `create_report`** — handler in `crates/api/api_crud/src/governance/create_report.rs` per [04 §6.1](04-data-model-and-api.md). Responsibilities: validate target exists; compute initial threshold contribution using the formula from [99 OQ-006](99-decisions-and-open-questions.md) **interim default** (`base_weight = 1`, `reporter_reputation = clamp(reporting_accuracy / 100, 0.1, 2.0)` — but in Phase 4 reputation is a stub, so `reporter_reputation = 1`; `recency_factor = exp(-hours_old / 168)`, but a fresh report has `recency_factor = 1`); upsert `moderation_case`; if accumulated weight > 3.0, mark `status = ThresholdMet`; **emit a governance log entry via the cross-cutting writer** (§4.1); return `CreateGovernanceReportResponse`. **No `report` table** — collapse into `moderation_case` per [04 §13 shortcut](04-data-model-and-api.md). **GDPR**: log entry uses `actor_pseudonym(reporter_person_id)`, never the username.

40. **`GET /api/v4/governance/case` → `get_case`** — handler in `crates/api/api/src/governance/get_case.rs` per [04 §6.2](04-data-model-and-api.md). Permission-aware: caller is target / reporter / juror / admin / public, returns the appropriate slice. For v0 the permission tree is hardcoded in the handler reading `reputation_snapshot` flags ([99 ADR-007 alternative paragraph](99-decisions-and-open-questions.md), confirmed by the canonical prompt's "Authz: hardcoded capability checks in Rust"). Wraps `read_case_detail` from Phase 2.

41. **`GET /api/v4/governance/jury/me` → `list_my_jury_queue`** — handler in `crates/api/api/src/governance/list_my_jury_queue.rs` per [04 §6.2](04-data-model-and-api.md). Wraps `list_jury_assignments_for_person` from Phase 2. Sorted by `selected_at`.

42. **`POST /api/v4/governance/jury/vote` → `submit_jury_vote`** — handler in `crates/api/api/src/governance/submit_jury_vote.rs` per [04 §6.2](04-data-model-and-api.md) and the aggregation logic in [04 §8](04-data-model-and-api.md). The pseudocode from [05 §6](05-mvp-and-delivery-plan.md) is the spec. Steps:
    - Verify the calling user has an `Accepted` `jury_assignment` for `case_id`
    - Insert `jury_vote` row
    - Count submitted votes for this case
    - If `< 3`: return 200, no further action
    - If `>= 3`:
      - Tally decisions; pick simple majority
      - Insert one `sanction` row (scope/action derived from the winning `JuryDecision`)
      - Update `moderation_case.status = Decided`, set `decided_at`
      - Insert a `public_case_log` row via the **redaction service** (§4.2) — the rationale is scrubbed before insert
      - Insert `reputation_event` rows for each juror (aligned-with-decision → +rep on `JuryReliability`; outlier → −rep). For each reporter on this case (`+rep` on `ReportingAccuracy` if sanctioned; `-rep` if dismissed)
      - Compute `closed_at` deadline (appeal window, e.g. `now() + 7 days`) and store on the case
      - **Emit governance log entries via the cross-cutting writer** (§4.1) for the vote, the decision, the sanction creation, the public log publication, and each reputation event
    - Hard-coded MVP parameters from [04 §8](04-data-model-and-api.md): jury size 5, quorum 3, simple majority. **No supermajority logic** — that's v1.

43. **`GET /api/v4/governance/modlog` → `list_modlog`** — handler in `crates/api/api/src/governance/list_modlog.rs` per [04 §6.2](04-data-model-and-api.md). Wraps `list_public_case_log` from Phase 2. Pagination via `page` + `limit`; per-community filter via `community_id`. **Public endpoint, no auth required.** This is the transparency surface — make sure unauthenticated callers get a clean, redacted response.

44. **`POST /api/v4/governance/admin/assign-jury` → `admin_assign_jury`** — handler in `crates/api/api/src/governance/admin_assign_jury.rs` per [04 §6.2](04-data-model-and-api.md). **Temporary admin-only backstop for v0** — picks 5 random eligible users (in Phase 4 "eligible" just means "not the target, not the reporter"; reputation gating arrives in Phase 5b task 57), inserts 5 `jury_assignment` rows with `status = Selected`, then immediately auto-promotes them to `Accepted` for testability. **Phase 5c task 64 reverts the auto-promotion** so the accept/decline flow is exercisable; Phase 4's golden-path test gains a round of `POST /jury/accept` calls accordingly. **Admin auth check: hardcoded `is_instance_admin` flag, no MFA in v0** ([05 §2](05-mvp-and-delivery-plan.md) admin backstops + [99 ADR-007](99-decisions-and-open-questions.md)).

45. **`POST /api/v4/governance/admin/close-case` → `admin_close_case`** — handler in `crates/api/api/src/governance/admin_close_case.rs` per [04 §6.2](04-data-model-and-api.md). Force-closes a case to `Closed` regardless of jury state. Required for emergency unblocking and for the `EmergencyRemove` post-facto path. Logs an audit entry. **Quorum + delay is a v2 item** ([99 ADR-010](99-decisions-and-open-questions.md)) — for v0 this is single-admin. Document in code that this is a known v0 simplification.

46. **`EmergencyRemove` wiring** — `admin_assign_jury` and `admin_close_case` must understand `CaseStatus::EmergencyRemove`. Create a thin helper `emergency_remove_open_case(target, admin_id, reason)` in `crates/api/api/src/governance/admin_emergency_remove.rs` that:
    - Removes the content (calls into Lemmy's existing remove pathway)
    - Inserts a `moderation_case` with `status = EmergencyRemove` and the removal as the initial fact
    - Triggers `admin_assign_jury` for post-facto review
    - Emits an extra-visible governance log entry per [06 §2.2.1](06-security-and-threat-model.md)
    No HTTP route is required for v0 — the function is callable from a future emergency-remove route or from a direct admin tool. It exists so the cross-cutting `EmergencyRemove` requirement (§4.3) is wired through. Per [99 ADR-013](99-decisions-and-open-questions.md): the jury **cannot un-remove** the content; document in the helper's doc comment.

47. **Server wiring** — `crates/server/src/governance.rs` per [04 §12](04-data-model-and-api.md) and [03 §7.3](03-architecture.md). **Composition root only** — registers the routes from task 38, schedules the (initially empty) background jobs (snapshot, sanction cleanup, jury timeout — these are stubs in Phase 4 and become real in Phase 5a task 54 and Phase 6), and that's it. **No business logic here** ([03 §11](03-architecture.md), [04 §13.2](04-data-model-and-api.md)).

48. **The end-to-end golden-path test** — `tests/e2e.rs::report_to_modlog_golden_path`. The single most important test in v0. Steps:
    - Spin up Postgres in Docker (or use the existing dev container)
    - Run all migrations
    - Seed two test users (A and B), one community, one post by B in the community
    - Acquire JWT for A (Lemmy's existing auth path)
    - `POST /api/v4/governance/report` as A targeting B's post; assert response includes `case_id` and `threshold_met = true` (since one report at weight 1.0 in v0 is below the 3.0 threshold — **adjust the test**: file 4 reports from 4 different users to clear the threshold, OR use the admin backstop to force `status = ThresholdMet` without the threshold logic running. **Pick option B for v0** — it makes the test deterministic and the threshold formula stays an OQ-006 placeholder)
    - Use admin user to call `POST /api/v4/governance/admin/assign-jury` for the case → assigns 5 jurors
    - For each of 3 jurors, `POST /api/v4/governance/jury/vote` with `JuryDecision::AdvisoryLabel`
    - On the 3rd vote, the case becomes `Decided`; assert one `sanction` row exists with `action = Label`, one `public_case_log` row exists, and 3 `reputation_event` rows exist for the jurors
    - As an unauthenticated caller, `GET /api/v4/governance/modlog?community_id={id}` and assert the case appears
    - Verify the `governance_log` table has entries for: report, threshold-met transition, jury assignment x5, jury vote x3, decision, sanction creation, public log publication, reputation events x3
    - Verify the hash chain over `governance_log` is consistent
- **Use `--user $(id -u):$(id -g)` on the Docker container** (PMD pattern: Docker test commands create root-owned files that block worktree cleanup)

49. **Threshold formula placeholder** — until [99 OQ-006](99-decisions-and-open-questions.md) is resolved, use the interim default from the OQ "Current lean": `base_weight = 1`, `reporter_reputation = clamp(reporting_accuracy / 100, 0.1, 2.0)` (in Phase 4 stubbed to `1.0`), `recency_factor = exp(-hours_old / 168)` (in Phase 4 stubbed to `1.0`). Threshold: weighted sum > 3.0. Hardcoded as Rust constants in `create_report.rs` with a doc-comment pointing at [99 OQ-006](99-decisions-and-open-questions.md). **Open ticket in repo**: "Resolve OQ-006 threshold formula before v1."

**Dependencies:** Phases 1–3.

**Definition of done** (from [05 §4 Step 4](05-mvp-and-delivery-plan.md)): the integration test in task 48 passes. **This is the true v0 vertical slice. If this test passes, the governance mechanic works.**

---

### Phase 5 — Reputation, sponsorship & adaptive governance (Step 5 of [05 §4](05-mvp-and-delivery-plan.md))

**Posture:** Phase 5 introduces `governance_config` as the single source of tuneable values — every threshold, delta, multiplier, formula parameter, decay rule, and onboarding setting is a config row with a Rust-seeded default. This exists because the platform is an experiment in adaptive moderation: the "right" numbers will come from pilot-community feedback, not up-front design. Admins edit config rows directly in v0 (DB access); the write endpoint is v1. Founder seeding is a first-class CLI tool using `reputation_event.expires_at` for decayable bootstrap trust.

**Split across Phase 5a + 5b + 5c** (per 2026-04-17 design-soundness review, Option C) to keep each ralph session under the 10–12 task ceiling and isolate sponsor-liability integer math into a small focused PR for review-quality gain. 5a ships the reputation infrastructure (6 tasks); 5b ships the cross-cutting math + Phase 4 mutations + founder bootstrap (5 tasks); 5c ships the remaining endpoints + observability + capability tests (9 tasks).

---

#### Phase 5a — Config, reputation infrastructure, endorsement handler (6 tasks)

**Goal:** `governance_config` table + reader shipped, with `valid_from` history + const-parity test (per design-soundness review §3). `membership_state` column ships as deferred-enforcement, with a CI grep-guard that no v0 handler reads it. Reputation view crate + snapshot calculator + 15-min background job all in place; background job writes a structured-fields trace on every tick (S2 from design review). `create_endorsement` handler works under config-driven gate strategy. After 5a, the reputation infrastructure and endorsement lifecycle are provably tuneable at runtime. Sponsor-liability + founder bootstrap land in 5b; remaining endpoints land in 5c.

**Tasks:**

50. **`governance_config` migration + reader module** — new migration `add_governance_config` at `crates/db_schema/migrations/{timestamp}_add_governance_config/`. Table: `(id SERIAL PK, scope TEXT, key TEXT, value_type TEXT, value_int BIGINT, value_float DOUBLE PRECISION, value_text TEXT, value_bool BOOLEAN, updated_at TIMESTAMPTZ, updated_by INTEGER → person(id), UNIQUE(scope, key))`. Scope is `'instance'` or `'community:<id>'`. Reader module at `crates/api/api/src/governance/config.rs` with `get_int/get_float/get_bool(conn, scope, key)` methods that fall back `community:<id> → instance → Rust const default`, and a per-request `ConfigCache` so repeated reads in one handler hit the DB once. `up.sql` seeds all instance-scoped rows via `INSERT ... ON CONFLICT (scope, key) DO NOTHING` so re-runs don't wipe admin edits. **Seeded keys (32 rows):** thresholds (`jury_reliability=50`, `reporting_accuracy=50`, `endorsement_strength=25`), jury params (`panel_size=5`, `quorum=3`, `age_requirement_days=60`, `max_concurrent_assignments=3`, `fallback_on_small_pool=true`), deltas (`juror_aligned=10`, `juror_outlier=-5`, `reporter_upheld=10`, `reporter_dismissed=-5`, `endorsement_created_sponsor=5`, `endorsement_created_sponsee=5`, `sponsor_liability_minor=-10`, `sponsor_liability_moderate=-50`, `sponsor_liability_severe=-200`), liability multipliers (`founder_multiplier=2.0`, `regular_multiplier=1.0`, `sponsor_liability_floor=0` per [99 OQ-024](99-decisions-and-open-questions.md)), report formula (`base_weight=1.0`, `clamp_min=0.1`, `clamp_max=2.0`, `recency_half_life_hours=168.0`, `case_threshold_micros=3000000`), decay (`positive_half_life_days=90`), onboarding (`default_membership_state='member'`, `sponsor_gate_strategy='age'`, `sponsor_min_account_age_days=30`), founder caps (`max_founders_active=20`, `max_expires_days=365`, `max_seed_delta=200`), and job cadence (`snapshot_interval_seconds=900`). `sponsor_gate_strategy` is type `text` with legal values `'age' | 'open' | 'closed'` — task 55 dispatches on it; v1 adds `'age_or_surety' | 'reputation' | 'allowlist'` per OQ-020. **Snapshot schema subtask:** the same migration adds `ALTER TABLE reputation_snapshot ADD COLUMN can_sponsor BOOLEAN NOT NULL DEFAULT false` so task 53 has a real column to write to; doc-comment the new column with a pointer to [99 OQ-014](99-decisions-and-open-questions.md). **GOTCHA:** Rust `const` fallbacks mirror every seeded value so a missing row is self-healing rather than an error. **GOTCHA:** Phase 4's existing `threshold_score` column is currently populated in integer units; `up.sql` must multiply existing rows by 1_000_000 so task 58's micros representation is consistent. **GOTCHA:** the `can_sponsor` column addition is an intentional override of the "two booleans" shortcut in [04 §13.2] — v0 now ships three snapshot booleans (`jury_eligible`, `trusted_reporter`, `can_sponsor`), with `can_sponsor` computed-but-unread until v1 flips enforcement.

51. **`membership_state` column + enum** — new migration `add_person_membership_state`. Adds `MembershipState` enum (`Member | Provisional | Suspended`) to `crates/db_schema_file/src/enums.rs`. `ALTER TABLE person ADD COLUMN membership_state MembershipState NOT NULL DEFAULT 'member'` — existing users grandfathered. Patch Lemmy's register handler (`crates/api/api_crud/src/site/register.rs` or fork equivalent) to read `governance.onboarding.default_membership_state` from config at signup time and write the value. **No v0 handler reads the column**; it ships so v1 can flip config and start enforcing without a schema migration (OQ-016). **GOTCHA:** document in the register patch's doc comment that this is a deferred-enforcement column; future guards belong in handlers, not here.

52. **`crates/db_views/reputation` crate** — per [04 §4.3](04-data-model-and-api.md). View structs: `ReputationSummaryView` (including `active_sanctions` count), `EndorsementSummaryView`. Queries: `read_reputation_summary(conn, person_id, community_id)`, `list_endorsements_for_person(conn, person_id, active_only: bool)`, `list_sureties_for_person(conn, person_id, active_only: bool)`. Files under `crates/db_views/reputation/`. Add workspace `Cargo.toml` entry. **GOTCHA:** `active_sanctions` count uses the two-round-trip pattern from Phase 2b's `sanction_action_by_case` rather than an inline SELECT join.

53. **Reputation snapshot calculator** — `crates/api/api/src/governance/reputation_snapshot.rs`. Pure async function `recompute_snapshot(conn, person_id, community_id, config: &mut ConfigCache) -> LemmyResult<ReputationSnapshot>`. Loads `reputation_event` rows where `expires_at IS NULL OR expires_at > now()` — **the `expires_at` filter is what implements founder decay-cliff**. Sums deltas per dimension; for positive events older than `config.decay.positive_half_life_days`, halves the delta in-memory (events remain immutable per ADR-005). Joins `person.published` for account age and `sanction` for active-sanctions count. Computes capability booleans from config thresholds: `jury_eligible = jury_reliability ≥ threshold AND account_age ≥ age_requirement AND active_sanctions = 0`; `trusted_reporter = reporting_accuracy ≥ threshold`; `can_sponsor = endorsement_strength ≥ threshold` (computed and written to the `can_sponsor` column added by task 50; not read by v0 gates per [99 OQ-014](99-decisions-and-open-questions.md)). v0 therefore ships three snapshot booleans, deliberately overriding the two-boolean shortcut in [04 §13.2]. Upserts via `INSERT ... ON CONFLICT (person_id, community_id) DO UPDATE`. **Always writes a zero-valued row for zero-events users** so downstream queries can use INNER JOIN. **Emits `governance_log` entry `capability_changed` when any of the three booleans flips in either direction** — this is observability item #3 from the "less arbitrary" additions. **GOTCHA:** the Phase 1 migration already creates `UNIQUE (person_id, community_id)` on `reputation_snapshot`, but Postgres treats `NULL` as distinct in unique constraints — bundle a subtask here to add a partial unique index `CREATE UNIQUE INDEX reputation_snapshot_person_null_community ON reputation_snapshot (person_id) WHERE community_id IS NULL` so instance-scoped snapshots also dedupe. **GOTCHA:** compute the `old_snapshot` via an initial SELECT before the upsert so `detect_capability_changes(old, new)` can fire the log diff. **GOTCHA:** `recompute_snapshot` is safe both inside `run_transaction` (task 55's endorsement flow reads-its-own-writes for immediate consistency) and outside (task 54's background job runs with a pooled connection). Decay is computation-only per ADR-005 — `reputation_event` rows remain immutable; the halved value is derived each tick from `created_at` vs `positive_half_life_days` and never written back. **GOTCHA:** add a `pub const` block at the top of `crates/api/api/src/governance/governance_log.rs` enumerating every v0 `entry_kind` string (`report_created`, `threshold_met`, `jury_assigned`, `jury_voted`, `sanction_created`, `public_log_published`, `reputation_delta`, `case_decided`, `capability_changed`, `sponsor_liability_applied`, `founder_seeded`, `endorsement_created`) so call-site typos fail at compile time. The column itself stays free-form TEXT.

54. **Snapshot recalculation background job** — replace the stub `schedule_governance_jobs` in `crates/server/src/governance.rs` with a `tokio::spawn` that loops on `tokio::time::interval(Duration::from_secs(config.governance.job.snapshot_interval_seconds))`. Function body `run_snapshot_batch(context)` lives in `reputation_snapshot.rs`: reads `MAX(calculated_at)` as watermark, finds distinct `(person_id, community_id)` pairs with new events OR with founder events that expired since the last tick (forces recompute on the decay cliff), calls `recompute_snapshot` per pair with a shared `ConfigCache`. Logs tick-start/end, pair count, expired-founder count. **GOTCHA:** composition root stays declarative per [03 §11] — the tokio block is ≤10 lines of glue; all logic is in `reputation_snapshot.rs`. **GOTCHA:** the interval value is read once at task spawn; changing `snapshot_interval_seconds` requires server restart. Acceptable v0 limitation. **GOTCHA:** do not queue this job as a Junior task — Junior's inactivity watchdog kills workers after ~6 min of no stdout; this is an in-process scheduler by design.

55. **`create_endorsement` handler** — new file `crates/api/api_crud/src/governance/create_endorsement.rs`, route `POST /api/v4/governance/endorsement`. Inside one `run_transaction`: **dispatch on `config.onboarding.sponsor_gate_strategy`** — `'closed'` → reject with `LemmyErrorType::NotFound`; `'open'` → skip age check entirely (recruitment-drive mode); `'age'` (default) and any unknown value → check caller's account age against `config.onboarding.sponsor_min_account_age_days`, reject if under. The strategy enum is decoded via an exhaustive `match` on a private `SponsorGateStrategy` enum (no `_ =>` wildcard per [clippy test style feedback]) with a fallback arm that treats unknown text as `'age'` and logs a `warn!` naming the offending string. After the gate passes: validate target exists and is not self; enforce max-5-active endorsements from caller; enforce 48h cooldown from caller's last endorsement creation; insert `endorsement` row; if target has <2 active sureties, also insert a `surety` row; emit two `reputation_event` rows (`+config.deltas.endorsement_created_sponsor` on EndorsementStrength for caller, `+config.deltas.endorsement_created_sponsee` on ParticipationConsistency for target); call `recompute_snapshot` for both parties (same transaction, immediate consistency); emit `governance_log` entry `endorsement_created` with both IDs pseudonymised via `actor_pseudonym_helper::get_or_create` and the active gate-strategy recorded in the payload so the audit trail shows which gate passed. **`can_sponsor` is NOT checked by the gate in v0 per [99 OQ-014](99-decisions-and-open-questions.md)** — the capability column from task 50 is computed and stored but not consulted here; v1 adds a `'reputation'` strategy that reads it. Document this in the handler's doc comment with pointers to OQ-014, OQ-020, and the v1 migration path. **GOTCHA:** concurrent endorsements targeting the same sponsee may produce overlapping `recompute_snapshot` calls in separate transactions; last-writer-wins on the snapshot row is acceptable for v0, but `governance_log` may show non-monotonic per-user totals during the interleave — acknowledged, not fixed in v0. **GOTCHA:** flipping `sponsor_gate_strategy` from `'age'` to `'open'` (or back) takes effect on the **next request** — no cache invalidation needed because `ConfigCache` is per-request. An in-flight endorsement that started under `'age'` completes under `'age'`.

**Phase 5a dependencies:** Phases 1–4 complete.

**Phase 5a definition of done:**
- `cargo check --features full --workspace --no-deps` — zero warnings
- `cargo clippy --features full --workspace --no-deps -- -D warnings` passes
- `cargo test --test e2e --features full` — Phase 4's `report_to_modlog_golden_path` **still passes** (regression guard) plus all existing 8 e2e tests
- Snapshot background job logs a tick message within the configured interval on test-harness startup (per S4 of design review, the tokio::spawn checks `BREHON_DISABLE_BACKGROUND_JOBS=1` so e2e tests can call `recompute_snapshot` directly without racing)
- Migration on `person.membership_state` uses Postgres 11+ fast-path (NULLABLE + DEFAULT then SET NOT NULL via separate statement, or verified-fast NOT NULL DEFAULT) so column-add does not rewrite the table (per B2 of design review)
- `reputation_snapshot.can_sponsor` column exists and is written for every recomputed row; no v0 handler reads it (verify via `scripts/brehon/lint-no-membership-read.sh` and a sibling `lint-no-can-sponsor-read.sh`)
- `governance_config` `CHECK` constraint enforces `value_type` discriminator; `valid_from` column populates with `now()` on every insert; `governance_config_current` view returns the most recent row per (scope, key)
- Seed-vs-Rust-const parity test in `crates/api/api/src/governance/config.rs` passes — every seeded key has a Rust const fallback and every const has a seeded row
- 5a phase-close PR opened against `governance-v0` via `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-5a`

---

#### Phase 5b — Sponsor-liability, jury gating, founder bootstrap (5 tasks)

**Goal:** sponsor-liability helper wired into `submit_jury_vote` (with founder multiplier per OQ-022 `now()`-at-case-close semantics); reputation-gated jury selection with concurrent-cap; OQ-006 threshold formula live in `create_report`; founder-seeding CLI; 5b e2e test verifies the full sanction-to-sponsor-liability flow with founder multiplier and includes a "founder chain survival" scenario per design-review Issue F.

**Tasks:**

56. **Sponsor-liability helper + wire into `submit_jury_vote`** — new file `crates/api/api/src/governance/sponsor_liability.rs`. Signature `apply_sponsor_liability(conn, target_person_id, case_id, community_id, action: SanctionAction, config: &mut ConfigCache) -> LemmyResult<usize>` (returns active-sponsor count). Maps `SanctionAction` via exhaustive match (no `_ =>` wildcard — per the `feedback_clippy_test_style` convention forbidding wildcard matches on governance enums) to severity bucket: Label / VisibilityReduction → minor; TemporaryRestriction / ContentRemoval → moderate; CommunityExclusion / InstanceSuspension / FederationQuarantineRecommendation → severe. **A new `Restoration { description: String }` variant is added to `SanctionAction` in this task as the v0 reserved slot for `folog n-othrusa`-style restorative sanctions per [99 OQ-003 (amended 2026-04-17)](99-decisions-and-open-questions.md); severity bucket: minor (same as `Label`).** The match is now exhaustive over **8** `SanctionAction` variants, not `JuryDecision`'s 8 — `map_decision_to_sanction` already filters `JuryDecision::NoAction` to `None` upstream, and `apply_sponsor_liability` is not called on the `None` path (see skip rule below). Reads the bucket's delta from config. Queries `surety WHERE sponsored_id = target AND revoked_at IS NULL ORDER BY sponsor_id ASC` — active sureties only. Early-returns `Ok(0)` if zero active sponsors. Floor-divides delta by sponsor count; distributes remainder deterministically to first N sponsors by `sponsor_id ASC` (so a -50 across 3 sponsors sums to exactly -50). **For each sponsor, checks `EXISTS (SELECT 1 FROM reputation_event WHERE person_id = ? AND dimension = 'endorsement_strength' AND expires_at IS NOT NULL AND expires_at > now())` — if true the sponsor is a founder and `config.liability.founder_multiplier` is applied; otherwise `regular_multiplier`.** **Honour-price floor clamp ([99 OQ-024](99-decisions-and-open-questions.md)):** before writing the `reputation_event`, compute `current_endorsement_strength` for the sponsor from their most recent `reputation_snapshot` row; if `current + final_delta < config.liability.sponsor_liability_floor` (default 0), clamp `final_delta = config.liability.sponsor_liability_floor - current` so the post-event value equals the floor exactly (never below). If the clamp fires, emit an additional `governance_log` entry `sponsor_liability_clamped` recording pseudonym, uncapped delta, clamped delta, floor value — auditors see both the ordered amount and the amount actually applied. Writes one `reputation_event` on EndorsementStrength per sponsor with the final (multiplied, floor-clamped) delta. Emits one `governance_log` entry `sponsor_liability_applied` per sponsor recording pseudonym, original delta, multiplier used, final delta. Extend `submit_jury_vote.rs`: between the sanction insert (step 8) and the case flip to Decided (step 9), add `if let Some(target_id) = case_row.target_person_id { sponsor_liability::apply_sponsor_liability(...).await? }`. Skip entirely when `map_decision_to_sanction` returns `None` (NoAction). **GOTCHA:** founder-status check is per-sponsor, per-event — a founder whose seed expired naturally falls back to regular_multiplier without any re-seeding ceremony. **GOTCHA:** after integer-multiply, round half-to-even to avoid cumulative drift. **GOTCHA:** Phase 4 hardcoded constants `JUROR_ALIGNED_DELTA = 10` etc. in `submit_jury_vote.rs` (lines 82-90) also move to config reads in this task to keep the tuning story consistent. **GOTCHA:** the floor clamp happens AFTER the founder-multiplier is applied — the ordering is `raw_delta → /sponsor_count → ×multiplier → clamp_to_floor`. This ordering means a founder's effective loss is capped at their pre-sanction `endorsement_strength`, not at the multiplied amount, which is the honour-price-faithful interpretation per [99 OQ-024](99-decisions-and-open-questions.md). **GOTCHA:** the new `Restoration` variant exists in `SanctionAction` after this task but is not selected by any v0 handler — juror UI still offers only the existing 7 variants for vote-tallying. `Restoration` is reserved for (a) future code paths (e.g. a v1 `admin_restorative_action` endpoint) and (b) enum exhaustiveness in downstream matches so v1's refinement into specific variants (`Apology | ContentCorrection | CommunityService` per [99 OQ-003](99-decisions-and-open-questions.md)) is a variant-refinement, not a pattern-replacement. Add a doc-comment on the variant referencing OQ-003.

57. **Reputation gating + concurrent-cap in `admin_assign_jury`** — extend `select_eligible_jurors` in `crates/api/api/src/governance/admin_assign_jury.rs`. Read `config.jury.panel_size`, `age_requirement_days`, `max_concurrent_assignments`. Add `INNER JOIN reputation_snapshot ON person_id = person.id AND community_id IS NOT DISTINCT FROM case.community_id` with filter `reputation_snapshot.jury_eligible = true` (INNER JOIN is safe because task 53 guarantees every person has at least a zero-valued row). Add NOT IN subquery excluding persons with `max_concurrent_assignments` or more active/selected assignments. Change signature to accept optional `exclude_person_ids: Option<&[PersonId]>` parameter (task 65 decline-handler uses it for replacement picks; `admin_emergency_remove` passes `None`). If `eligible.len() < panel_size`, log `warn!` and fall back to Phase 4's "not-target, not-reporter" unfiltered pool — guarded by `config.jury.fallback_on_small_pool` (bool, default `true`; admin can flip to `false` for stricter production behaviour). **GOTCHA:** per feedback memory, if the join churn causes build issues on Windows, `cargo clean -p pq-sys` unsticks it.

58. **OQ-006 threshold formula implementation in `create_report`** — remove `V0_THRESHOLD` and `V0_REPORTER_WEIGHT` constants + TODO markers from `crates/api/api_crud/src/governance/create_report.rs`. At handler entry, call `load_or_compute_snapshot(conn, reporter_id, community_id, &mut config)` (helper that checks for existing row, computes if missing). Read formula params from config: `base_weight`, `clamp_min`, `clamp_max`, `recency_half_life_hours`, `case_threshold_micros`. Compute `reporter_reputation = clamp(accuracy as f64 / 100.0, clamp_min, clamp_max)`; `recency_factor = (-hours_old as f64 / half_life_hours).exp()` (fresh reports → 1.0; stale-report recomputation is v1); `weight_micros = (base_weight * reporter_reputation * recency_factor * 1_000_000.0) as i64`. Store `weight_micros` on `moderation_case.threshold_score`. Case flips `Open → ThresholdMet` when accumulated score > `case_threshold_micros`. **Log the computed `reporter_reputation` multiplier in the `governance_log` payload for `report_created` entries** — observability item #2. **GOTCHA:** `clamp(0.0, 0.1, 2.0) = 0.1` is the floor — a zero-reputation reporter still contributes; the OQ-006 mechanic prevents silent-zero gaming. **GOTCHA:** existing Phase 4 `threshold_score` rows are migrated to micros by task 50's `up.sql`; the Phase 4 golden-path test's threshold-forcing pattern (admin backstop forces `ThresholdMet` directly) remains unchanged.

59. **Founder seeding CLI** — new binary `crates/tools/seed_founders` (or `scripts/seed_founders.sh` calling a shared Rust entrypoint). Invocation takes `--admin-user`, one or more `--founder <person_id>:<jury_reliability>:<reporting_accuracy>:<endorsement_strength>`, and a required `--expires-at <date>` (no default — admin must specify). Requires admin-level DB credentials (env var, not JWT — this is out-of-band). Reads caps from config (`max_founders_active`, `max_expires_days`, `max_seed_delta`). Validates each spec: person exists, all three deltas ≤ `max_seed_delta`, `expires_at ≤ now + max_expires_days`. Counts existing unexpired founder events; refuses if `current + new > max_founders_active`. For each founder: inserts 3 `reputation_event` rows (one per named dimension) with `expires_at = parsed`, `reason = "founder_seed"`, both `source_*_id` NULL; emits prominent `governance_log` entry `founder_seeded` with pseudonymised admin + pseudonymised founder + deltas + expiry (**modlog-surfaced** — community sees "X was seeded as founder, expiring Y"); calls `recompute_snapshot`. Prints summary. **participation_consistency is deliberately NOT seeded** per Q3 — founders earn it through real activity. **GOTCHA:** CLI reuses `actor_pseudonym_helper::get_or_create` for the admin so the log stays GDPR-compliant (ADR-015). **GOTCHA:** idempotent — running again adds new events, doesn't disturb existing ones; supports re-seeding after expiry or adding later cohorts of founders. **GOTCHA:** the CLI reuses the event-insert helpers from `reputation_snapshot.rs` and `governance_log.rs` but does **not** invoke any HTTP-layer auth middleware — admin authority is DB-credential-level only (env var holding the Postgres URL, never a JWT). The binary is out-of-band by design; it never exposes an HTTP surface.

60. **Phase 5b e2e test — `sponsor_liability_with_founder_multiplier` + founder-chain-survival** — `tests/e2e.rs::sponsor_liability_with_founder_multiplier`. Spin up Postgres, run migrations (includes config seed). Create users A (target), B (regular sponsor, 40d old), C (founder sponsor, 40d old). Invoke founder CLI for C (`jury_reliability=100, reporting_accuracy=100, endorsement_strength=100, expires_at=now+90d`). B endorses A; C endorses A — both age checks pass, both sureties inserted. Create a case against A via direct DB insert at `ThresholdMet`. Admin assigns 5 jurors (seed the pool with enough eligible users to skip fallback). 3 jurors vote `ContentRemoval` (moderate). After decision assert: B's new `reputation_event` has `delta = -25` (−50 moderate / 2 sponsors × 1.0); C's has `delta = -50` (−50 / 2 sponsors × 2.0 founder_multiplier); `governance_log` contains two `sponsor_liability_applied` entries recording `multiplier=1.0` and `multiplier=2.0` respectively. After `recompute_snapshot`, verify C's `endorsement_strength = 50` (100 seed + −50 liability), B's `= -25`. Then flip `config.liability.founder_multiplier` to `3.0`, run a second sanction round, verify subsequent liability events use the new multiplier. **Add second test branch — `founder_chain_survival`** (per design-review Issue F): two founders C1+C2 each sponsor user D (founder seeds 100 endorsement_strength each); D is sanctioned `CommunityExclusion` (severe, −200); after `apply_sponsor_liability`, both founders should retain `can_sponsor=true` under default config (i.e. their post-sanction `endorsement_strength` stays above the `25` threshold). If they don't, the default `sponsor_liability_severe = -200` and `founder_multiplier = 2.0` defaults need flagging in the 5b retro for v1 tuning. **Add third test branch — `honour_price_floor_clamp`** (per [99 OQ-024](99-decisions-and-open-questions.md)): regular sponsor E with `endorsement_strength = 5` (no founder seed; minimal organic accrual) is the sole active sponsor of user F; F is sanctioned `ContentRemoval` (moderate, −50); the sole-sponsor floor-divide gives E the full −50; the honour-price floor clamp reduces E's `final_delta` to exactly `−5` so E's post-event `endorsement_strength = 0` (not −45). Assert: E's `reputation_event` row has `delta = -5`; `governance_log` contains both a `sponsor_liability_applied` (delta −5) and a `sponsor_liability_clamped` (uncapped −50, clamped −5, floor 0) entry; E's `can_sponsor` is now `false` (below the 25 threshold) — the disablement mechanic still fires, the permanent-outcast state does not. **GOTCHA:** testcontainer invocation uses `--user $(id -u):$(id -g)` per Phase 4 pattern (Docker tests create root-owned files otherwise). **GOTCHA:** return `Result<(), Box<dyn std::error::Error>>` and coerce LemmyError via `.map_err(|e| format!("{e}").into())` — clippy test-style rules forbid `.unwrap()`.

**Phase 5b dependencies:** Phase 5a merged into `governance-v0`.

**Phase 5b definition of done:**
- `cargo check --features full --workspace --no-deps` — zero warnings
- `cargo clippy --features full --workspace --no-deps -- -D warnings` passes
- `cargo test --test e2e --features full sponsor_liability_with_founder_multiplier` passes (both `default_multiplier` and `founder_chain_survival` branches)
- Phase 4's `report_to_modlog_golden_path` **still passes** (regression guard — task 56 inserts `apply_sponsor_liability` between sanction-insert and case-flip)
- Existing Phase 4 `moderation_case.threshold_score` rows multiplied by 1_000_000 by task 50's `up.sql` (already true after 5a merge); golden-path test's admin-forced `ThresholdMet` pattern remains unaffected
- Founder-seeding CLI usable against the test DB; caps enforced; `governance_log` entries appear
- Sponsor-liability remainder direction documented in task 56 pseudocode (per design-review Issue B): for negative deltas, the first |remainder| sponsors by `sponsor_id ASC` each receive one extra unit of negative (i.e. MORE negative), so total sums to severity (modulo founder-multiplier amplification per OQ-022 / vision §5.2 note)
- Founder-multiplier timestamp documented in task 56 GOTCHA per OQ-022: `now()` at case-close, NOT case-open
- Honour-price floor clamp ([99 OQ-024](99-decisions-and-open-questions.md)) exercised by an explicit e2e sub-test: a regular sponsor with `endorsement_strength = 5` absorbing a `-50` moderate liability writes exactly `delta = -5` (clamped to floor 0) and emits both `sponsor_liability_applied` and `sponsor_liability_clamped` governance_log entries
- `Restoration { description: String }` variant exists in `SanctionAction`; compile-time exhaustive matches in `apply_sponsor_liability`, `map_decision_to_sanction`, and any other `SanctionAction` match site include it (even if as a minor-severity passthrough); doc comment on the variant references [99 OQ-003](99-decisions-and-open-questions.md)
- 5b phase-close PR opened against `governance-v0`; CodeRabbit findings on sponsor-liability math addressed

---

#### Phase 5c — Remaining endpoints, observability, capability tests (9 tasks)

**Goal:** all 11 MVP endpoints wired. Reputation observability endpoint + threshold-crossing log emission live. Jury accept/decline/appeal handlers shipped. All 11 endpoints pass happy-path. Capability-gating regression test passes with config-flip assertion.

**Tasks:**

61. **`GET /api/v4/governance/reputation/me` → `get_my_reputation`** — handler in `crates/api/api/src/governance/get_my_reputation.rs` per [04 §6.2](04-data-model-and-api.md). Returns `ReputationSummaryView` for the caller, scoped by optional `community_id` query param. Calls `recompute_snapshot` on entry only if the snapshot row is missing (row-existence check first to avoid read-path overhead). **GOTCHA:** API response includes raw counters so v1's admin dashboard can render distributions; user-facing UI labels should render capability booleans and badges only (not numbers), per vision doc §5.3.

62. **`POST /api/v4/governance/admin/reputation-stats` → `admin_reputation_stats`** — observability item #1. New admin-only endpoint (`is_admin` guard) in `crates/api/api/src/governance/admin_reputation_stats.rs`. Returns distribution histograms per dimension (buckets: 0, 1–30, 31–80, 81–200, 200+), current threshold value, count of users above threshold, capability counts (`jury_eligible_count`, `trusted_reporter_count`, `can_sponsor_count`), and founder-event stats (active vs expired). Registered as an **additional admin backstop** alongside `assign-jury` and `close-case` — not counted in the 11 MVP endpoints. **GOTCHA:** bucketing via SQL `CASE WHEN` per dimension is cheaper than loading all snapshots into Rust. One query per dimension.

63. **Threshold-crossing log wire-up** — observability item #3. The snapshot calculator (task 53) already emits `capability_changed` log entries when a boolean flips. Task 63 wires the remainder: (a) ensure `capability_changed` entries are surfaced in `GovernanceModlogView` (extend the Phase 2b view's entry-kind filter if it has one); (b) add helper `detect_capability_changes(old_snapshot, new_snapshot) -> Vec<CapabilityChange>` in `reputation_snapshot.rs` used by task 53; (c) verify test coverage — running `recompute_snapshot` on a user crossing the `jury_eligible` threshold produces a visible log entry with `direction: 'gained' | 'lost'`. **GOTCHA:** config threshold edits (e.g. admin raises `thresholds.jury_reliability` from 50 to 80) can cascade into mass capability losses on the next snapshot batch; v0 emits one `capability_changed` log entry per affected user per tick (no dedupe, no burst-collapse — batching by community is v1). **GOTCHA:** in-flight `jury_assignment` rows (status `Selected` or `Accepted`) are NOT revoked when a threshold edit drops a juror below `jury_eligible`. Snapshot recompute affects **future** selections only; the currently-seated juror completes the vote. Retroactive revocation is explicitly v1 scope — mention in the task's doc comment so the future ralph session doesn't helpfully add it.

64. **`POST /api/v4/governance/jury/accept` → `accept_jury_assignment`** — handler in `crates/api/api/src/governance/accept_jury_assignment.rs` per [04 §6.2](04-data-model-and-api.md). Guards: assignment is in `Selected` state; caller is not in the same `surety` cluster as the target (v0 definition: "shares ≥1 active sponsor with target"); caller is not the case's reporter. Updates `jury_assignment.status = Accepted`. Emits governance log entry. **GOTCHA:** Phase 4's `admin_assign_jury` currently inserts with `status = Accepted` for test convenience — this task changes that to `Selected` to exercise the accept/decline flow. Phase 4's golden-path test must be updated to call `accept` between assignment and vote (see "Impact on other phases" below).

65. **`POST /api/v4/governance/jury/decline` → `decline_jury_assignment`** — handler in `crates/api/api/src/governance/decline_jury_assignment.rs` per [04 §6.2](04-data-model-and-api.md). Updates caller's assignment to `Declined`, logs optional reason. **Triggers replacement juror selection** by calling `select_eligible_jurors` (from `admin_assign_jury.rs`) with `exclude_person_ids = [all current assignees on this case]`, then inserts one new `jury_assignment` row with `status = Selected`. Emits log entries. **GOTCHA:** concurrent-cap check applies to the replacement pick too (already handled by task 57's extended eligibility query).

66. **`POST /api/v4/governance/appeal` → `request_appeal`** — handler in `crates/api/api_crud/src/governance/request_appeal.rs` per [04 §6.1](04-data-model-and-api.md). Verifies `now() < case.closed_at`; verifies caller is the sanction target (original-reporter appeals deferred to v1); inserts `appeal` row with `status = Requested`; flips case `Decided → Appealed`. **No re-jury logic in v0** — the case sits in `Appealed` until an admin calls `admin_close_case`. The full "larger jury panel" flow is v1 per [05 §3] / ADR-010. Emits log. **GOTCHA:** handler doc comment points at ADR-010 + v1 roadmap.

67. **`GET /api/v4/governance/cases` → `list_cases`** — handler in `crates/api/api/src/governance/list_cases.rs` per [04 §6.2](04-data-model-and-api.md). Filters by optional `community_id`, `status`, `assignee`. Wraps Phase 2a's `list_open_cases_for_community`, extending the query if filters are richer than what Phase 2a ships. **GOTCHA:** match Phase 4's `list_modlog` pagination pattern (`page` + `limit`) so frontend clients can reuse paging code.

68. **Route registration + final MVP-endpoint smoke test** — extends `crates/api/routes/src/governance.rs`: the 5 Phase 4 routes (`POST /report`, `GET /case`, `GET /modlog`, `GET /jury/me`, `POST /jury/vote`) and the 2 Phase 4 admin backstops (`POST /admin/assign-jury`, `POST /admin/close-case`) stay registered; this task adds the 6 new 5a/5b handler routes (`POST /endorsement`, `GET /reputation/me`, `POST /jury/accept`, `POST /jury/decline`, `POST /appeal`, `GET /cases`) plus `POST /admin/reputation-stats` from task 62. **11 MVP endpoints (final state):** `POST /report`, `GET /case`, `GET /cases`, `GET /modlog`, `GET /reputation/me`, `POST /endorsement`, `GET /jury/me`, `POST /jury/accept`, `POST /jury/decline`, `POST /jury/vote`, `POST /appeal`. **Admin backstops (not counted in 11):** `POST /admin/assign-jury`, `POST /admin/close-case`, `POST /admin/reputation-stats` (task 62). Add `tests/e2e.rs::all_mvp_endpoints_return_non_404` — each route responds with 200 or appropriate 401/403, not 404. **GOTCHA:** `endorsement/revoke` is explicitly deferred to v1 per [05 §2]; do not miscount by including it.

69. **Capability-gating e2e test — `ineligible_user_cannot_be_picked_for_jury`** — `tests/e2e.rs::ineligible_user_cannot_be_picked_for_jury`. Seed 7 users: 5 with founder-CLI seed events (short 30d expiry) pushing `jury_reliability` above threshold; 2 left at zero. Run `admin_assign_jury` against a seeded case. Assert: the 2 zero-reputation users are NOT in the assignment list; the 5 eligible users ARE (or a subset of 5). Second branch: pre-seed 3 active `jury_assignment` rows for one of the 5 eligible users, run assignment on a different case, assert that user is NOT picked (concurrent-cap). Third branch: flip `config.governance.jury.max_concurrent_assignments` from 3 to 5, re-run same scenario, assert that user IS picked this time (config change takes effect without code change). **GOTCHA:** seed 5 eligible / 2 ineligible (not the reverse) to avoid triggering the task 57 fallback to unfiltered pool.

**Phase 5c dependencies:** Phase 5b merged into `governance-v0`.

**Phase 5c definition of done:**
- All Phase 5a + 5b checks still pass (regression)
- `cargo test --test e2e --features full` full suite green
- All 11 MVP endpoints return 200 on happy-path (Task 68 smoke test)
- `ineligible_user_cannot_be_picked_for_jury` passes including the config-flip branch
- `sponsor_liability_with_founder_multiplier` still passes (regression — both branches)
- Phase 4's `report_to_modlog_golden_path` **still passes** after task 64's Selected→Accepted handshake insertion
- Snapshot-staleness alert fires (per design-review B1): governance_log integrity-check job emits an `error!` log if `MAX(reputation_snapshot.calculated_at) < now() - 2 * snapshot_interval_seconds`
- [05 §9] Done-definition checklist rows 1–3 and row 7 are green
- 5c phase-close PR opened against `governance-v0`; merging 5c closes Phase 5

---

**Phase 5 impact on shipped Phase 4 code (mandatory changes during 5a/5b/5c):**

- `crates/api/api/src/governance/admin_assign_jury.rs` line 105 — change `status: JuryAssignmentStatus::Accepted` to `Selected` for task 64's accept-flow to be exercisable.
- `tests/e2e.rs::report_to_modlog_golden_path` — insert 5 `POST /api/v4/governance/jury/accept` calls between `admin/assign-jury` and the first `jury/vote` (~10 lines).
- `admin_assign_jury.rs::select_eligible_jurors` — signature gains optional `exclude_person_ids: Option<&[PersonId]>`; existing callers (`admin_emergency_remove`) pass `None`.
- `create_report.rs` constants block (lines 53–72) — task 58 removes `V0_THRESHOLD`, `V0_REPORTER_WEIGHT`, and the TODO markers; replaced with config-driven formula.
- `submit_jury_vote.rs` — task 56 inserts `apply_sponsor_liability` call immediately after line 251 (end of step 8: the `insert_into(sanction::table)` block and its `governance_log::append`), before line 253 (start of step 9: the `CaseStatus::Decided` update).
- `submit_jury_vote.rs` lines 82–90 — Phase 4 hardcoded reputation deltas (`JUROR_ALIGNED_DELTA = 10` etc.) become config reads in task 56 so the tuning story is consistent.

**Phase 5 impact on shipped Phase 1 code (added 2026-04-17):**

- `crates/db_schema_file/src/enums.rs::SanctionAction` (Phase 1 task 9) — Phase 5b task 56 adds a new `Restoration { description: String }` variant per [99 OQ-003 (amended)](99-decisions-and-open-questions.md). This is the first deliberate amendment to a Phase 1 enum from a later phase. Justification: avoids the v1 data-migration tax across every consumer (handlers, view crates, AP serialisers in Phase 6); reserves the slot for `folog n-othrusa`-style restorative sanctions per [vision §4 principle 5](01-vision-and-principles.md). Every existing exhaustive `match` on `SanctionAction` (in `apply_sponsor_liability`, `map_decision_to_sanction`, any view-crate severity mapper) gains a `Restoration { .. } => /* minor severity */` arm in this task. Schema migration: if `SanctionAction` is stored as a Postgres enum, a Phase 5b sub-migration adds the value via `ALTER TYPE sanction_action ADD VALUE 'Restoration'`. If stored as JSON, no migration is needed — Diesel-side enum updates suffice.

**Phase 5 impact on planned Phase 6 tasks:** none. Task 70's `submit_jury_vote` ordering becomes: sanction insert → sponsor-liability (5b) → case flip → public log → juror/reporter reputation → **federation publish (Phase 6)** → `case_decided` log entry. The federation publish remains gated on `scope = FederatedRecommendation` and is independent of sponsor-liability.

---

### Phase 6 — Federation objects & activities, outbound-only (Step 6 of [05 §4](05-mvp-and-delivery-plan.md))

**Goal:** when instance A decides a case with `FederationQuarantineRecommendation` scope, instance B receives a `SanctionNotice` ActivityPub activity, verifies it, stores it in `remote_sanction_notice`, surfaces it to admin review, and **does not auto-apply** it ([99 ADR-006](99-decisions-and-open-questions.md), [05 §3](05-mvp-and-delivery-plan.md), [03 §3.3](03-architecture.md)).

**Tasks:**

70. **Migration `add_federation_attestations`** — per [04 §1 Migration 4](04-data-model-and-api.md). Tables: `federation_attestation`, `remote_sanction_notice`. Files: `crates/db_schema/migrations/{timestamp}_add_federation_attestations/{up,down}.sql`. **Was technically part of Phase 1 by ordering**, but the columns and logic land here so the migration is moved to this phase.

71. **Diesel models for federation tables** — `FederationAttestation`, `RemoteSanctionNotice`. Per [04 §3](04-data-model-and-api.md). Files: `crates/db_schema/src/source/governance/{federation_attestation,remote_sanction_notice}.rs`.

72. **AP object types** — `crates/apub/objects/src/governance/{moderation_label.rs,trust_attestation.rs,sanction_notice.rs}` per [04 §9](04-data-model-and-api.md). `ModerationLabelObject`, `TrustAttestationObject`, `SanctionNoticeObject`. Use Lemmy's existing AP serialisation conventions (`activitypub_federation` crate or whatever 1.0-beta uses). **`ap_id` not `actor_id`** per [99 ADR-012](99-decisions-and-open-questions.md).

73. **AP activity types** — `crates/apub/activities/src/governance/{publish_label.rs,publish_attestation.rs,publish_sanction_notice.rs}` per [04 §10](04-data-model-and-api.md). Wraps the objects from task 72 in `Create` activities. `Undo` variants exist as stubs but aren't wired in v0 unless trivially cheap.

74. **Outbound publisher** — `crates/apub/apub/src/governance/outbox.rs` per [04 §11](04-data-model-and-api.md). Function `send_local_sanction_notice(case_id)`: builds a `SanctionNoticeObject` from the case + sanction rows, signs the activity (Lemmy's existing key-signing path), enqueues it on the federated outbox for delivery to all peers in `Allow` or `Limit` state. Function `send_local_trust_attestation(person_id, attestation_type)` similar but for attestations. **Scope restricted to peers in the federation_attestation `Allow` / `Limit` set; no peer table for v0, so all federated instances are treated as `Allow`** — document this as a v0 simplification.

75. **Inbound receiver** — `crates/apub/apub/src/governance/inbox.rs` per [04 §11](04-data-model-and-api.md). Function `receive_remote_sanction_notice(activity)`: signature verify (reject on failure, log it); schema validate; insert `remote_sanction_notice` row with `local_case_id = NULL`; **no auto-apply** ([99 ADR-006](99-decisions-and-open-questions.md), [06 §5](06-security-and-threat-model.md)). Function `receive_remote_trust_attestation` similar.

76. **Wire `submit_jury_vote` to publish on Critical sanctions** — extend the handler from task 42 (and from task 56 in Phase 5b) so that if the winning decision creates a sanction with `scope = FederatedRecommendation`, `send_local_sanction_notice(case_id)` is called as part of the post-decision flow. Ordering inside `submit_jury_vote`: sanction insert → sponsor-liability (5b) → case flip → public log → juror/reporter reputation → **federation publish (this task)** → `case_decided` log entry. Single-admin v0 (no quorum + delay — that's v2 per [99 ADR-010](99-decisions-and-open-questions.md)). Emits a governance log entry for the federation send.

77. **Federation integration test** — `tests/e2e.rs::sanction_notice_round_trip`. Spins up two API server processes against two Postgres databases (different ports). Decides a case on instance A with `scope = FederatedRecommendation`. Asserts instance A's outbox queue has a `SanctionNoticeObject`. Manually delivers the activity to instance B's `receive_remote_sanction_notice`. Asserts instance B has a `remote_sanction_notice` row with the correct fields and `local_case_id IS NULL`. **Do not require a real HTTP federation transport** for this test — call the inbox function directly with a serialised object. The HTTP transport is Lemmy's existing code; we trust it.

78. **Verify `verify.rs`** — `crates/apub/apub/src/governance/verify.rs` per [04 §11](04-data-model-and-api.md). Signature verification helper used by `inbox.rs`. Likely just delegates to Lemmy's existing AP signature verifier; the file exists so that future custom verification logic has a home.

**Dependencies:** Phases 1–5 (5a + 5b). The federation tables depend on the migrations from task 70.

**Definition of done** (from [05 §4 Step 6](05-mvp-and-delivery-plan.md)): the round-trip test in task 77 passes — sanction on instance A produces a stored advisory record on instance B, never auto-applied.

---

## 4. Cross-cutting requirements (apply to every phase)

These are not phase tasks. They are invariants that every phase must respect. They get wired up early (see "where introduced" below) and used by all subsequent phases.

### 4.1 Append-only, hash-chained governance log

**Where introduced:** Phase 1 (table + Postgres trigger, tasks 5–6). **Where used:** every phase from Phase 4 onward emits log entries.

- **Table:** `governance_log` per task 5.
- **Hash chain:** Postgres trigger from task 6 computes `entry_hash = sha256(prev_hash || entry_kind || payload || created_at)` on INSERT. **Append-only enforced two ways**: trigger raises on UPDATE/DELETE; DB grant denies UPDATE/DELETE to the app role.
- **Signing:** the trigger does not sign — it only hashes. A Rust helper in `crates/api/api/src/governance/governance_log.rs::append(entry_kind, payload, actor_pseudonym)` performs the insert, then signs the resulting `entry_hash` with `ed25519-dalek` and stores the signature in the `signature` column via a follow-up UPDATE. **Wait** — the signature is on the hash, which is computed inside the trigger. Two options:
  - **Option A (chosen for v0):** the helper does the insert (trigger fires, hash computed), reads back the row's `entry_hash`, signs it, and stores the signature in a **second `UPDATE` statement that is exempt from the no-update trigger via a row-level check on `signature IS NULL`**. The trigger allows exactly one update: the `signature IS NULL → signature = X` transition, after which the row is locked.
  - **Option B (deferred to v2):** signature is computed by an external signer process per [99 ADR-010 v2](99-decisions-and-open-questions.md). For v0 the signing key lives in `.env` as `GOVERNANCE_LOG_SIGNING_KEY` per the canonical prompt's hard constraints.
- **Verification:** a daily background job (registered in `crates/server/src/governance.rs`, task 47 + extension in Phase 5a task 54) walks the log, recomputes the chain, and verifies signatures. Divergence triggers an error log. **Alerting hookup is a v2 item** per [07 §7.1](07-operations-and-federation.md).
- **Crates needed:** `sha2`, `ed25519-dalek`, `rs_merkle` (the last is for the v3 Merkle-root anchoring path; included now per [99 ADR-010](99-decisions-and-open-questions.md) hard-constraint list, even if unused, so the dep tree is stable).
- **What gets logged:** every write to `moderation_case` state, `jury_assignment`, `jury_vote`, `sanction`, `appeal`, `federation_attestation`, `reputation_event`, `public_case_log`, and any `actor_pseudonym` deletion (per [04 §3 ActorPseudonym rules](04-data-model-and-api.md)). Per [06 §2.3](06-security-and-threat-model.md): "before the user response returns."

### 4.2 `actor_pseudonym` table and redaction service

**Where introduced:** Phase 1 (table, task 4) + Phase 4 (redaction service first use, task 42). **Where used:** every governance log write and every public log write.

- **Pseudonym generation:** Rust helper `actor_pseudonym::get_or_create(person_id) -> String`. Reads existing row; if absent, generates a new UUIDv4 (per [04 §3 ActorPseudonym rules](04-data-model-and-api.md): cryptographically random, never derived from `person_id`), inserts, returns. **Single source of truth — never inline `Uuid::new_v4()` in handlers.**
- **Right-to-delete:** admin tool (CLI or admin endpoint, **not in the 11**) deletes the `actor_pseudonym` row for a given `person_id` and writes an entry to a separate `gdpr_audit` table (NOT the governance log — the point is anonymisation in the governance log). After deletion, subsequent activity by the same user generates a **new** pseudonym (per [04 §3 ActorPseudonym rules](04-data-model-and-api.md)).
- **Redaction service:** `crates/api/api/src/governance/redaction.rs::scrub(text: &str) -> String`. Strips usernames (regex `@[a-zA-Z0-9_-]+`), email-shaped tokens, URLs that contain a username path segment, and a configurable blocklist of display names (loaded once at startup from the `local_user` table — yes, this is expensive; for v0 it's fine, for v1 we cache). **Single code path** per [06 §6](06-security-and-threat-model.md): "the redaction service is a single code path — not distributed across handlers."
- **Hard prerequisite:** every string written to `public_case_log.summary`, `public_case_log.rationale_redacted`, `governance_log.payload` (the JSONB) must pass through `scrub()`. Enforced by **wrapping the helpers** from §4.1 (`governance_log::append`) and from `submit_jury_vote` (the public-log-publish step in task 42) so that callers cannot accidentally bypass.
- **Test:** `tests/e2e.rs::redaction_strips_identifiers` — feeds a rationale containing a username, an email, and a URL; asserts they are absent from the resulting `public_case_log` row.

### 4.3 `EmergencyRemove` case status wired through

**Where introduced:** Phase 1 (enum variant, task 8). **Where used:** Phase 4 (helper function, task 46) and any future code path that branches on `CaseStatus`.

- **Enum variant:** `CaseStatus::EmergencyRemove` exists in [04 §2](04-data-model-and-api.md) and in the Rust enum from task 8. **Non-negotiable per [99 ADR-013](99-decisions-and-open-questions.md).**
- **Helper function:** `emergency_remove_open_case(target, admin_id, reason)` in `crates/api/api/src/governance/admin_emergency_remove.rs` — created in task 46. Removes the content via Lemmy's existing remove pathway; opens a case in `EmergencyRemove`; assigns a post-facto jury via `admin_assign_jury`; emits an extra-visible governance log entry per [06 §2.2.1](06-security-and-threat-model.md).
- **Match-arm sweep:** every `match case.status { ... }` in the codebase **must** handle `EmergencyRemove`. Use Rust's exhaustive-match — if you find yourself adding `_ =>`, stop and handle it explicitly. The variant exists from Phase 1 specifically so the compiler enforces this.
- **No HTTP route in v0** — the helper is callable from a CLI tool or future emergency-remove route. v1 adds the route once UX is clearer.
- **Public log behaviour:** when an `EmergencyRemove` case is published to `public_case_log`, the entry shows the **fact** of removal but **not** the content ([06 §2.2.1](06-security-and-threat-model.md)). The redaction service handles this — pass an explicit empty `content_excerpt` field.

### 4.4 AGPLv3 licensing and source disclosure

**Where introduced:** Pre-flight (§2.4 task 4 + 5). **Where used:** every public release.

- **`LICENSE` file:** AGPLv3 text, inherited from upstream Lemmy. Verify it's there post-fork; do not overwrite.
- **`AGPL-NOTICE.md` at repo root:** human-readable notice explaining (a) this is a fork of Lemmy, (b) AGPLv3 applies, (c) instance operators must make modified source available to users on request, (d) link to the source repo. Per [99 ADR-011](99-decisions-and-open-questions.md): "instance operators running this fork are on notice."
- **In-app source-disclosure link:** add a link to the source repo in the API server's HTTP `200 OK /` response (or wherever Lemmy already exposes a "powered by" link). v0 minimum: a string in `crates/server/src/lib.rs` that points at the repo URL once it's chosen post-Pre-flight.
- **CI artifact license:** ensure GitHub Actions does not strip or relicence anything during the build.
- **No relicencing** ([99 ADR-011](99-decisions-and-open-questions.md)): never attempt. Lemmy's copyright is held by many contributors.

---

## 5. Test strategy

Solo-dev appropriate. **Integration-only** until something breaks twice. Per [05 §9](05-mvp-and-delivery-plan.md) and the canonical prompt.

### 5.1 What exists

- **`tests/e2e.rs`** — lives at the workspace root or `crates/server/tests/e2e.rs` (Lemmy's convention determines which). All e2e tests in one file for v0, split when it crosses ~1500 lines.
- **Postgres in Docker** — the e2e harness starts a fresh Postgres 16 container per test run (`docker run --rm -d --user $(id -u):$(id -g) postgres:16` — note the `--user` flag per the PMD pattern about root-owned files blocking worktree cleanup), runs migrations, runs the test, tears down. Use `testcontainers-rs` if it's already in the Lemmy workspace; otherwise a small bash wrapper is fine.
- **No mocks, no test doubles for the DB.** Per the user's feedback memory: integration tests must hit a real database. (Memory: "feedback_integration_tests_real_db" — the user got burned by mocked DB tests masking a broken migration.)

### 5.2 The required tests

Nine tests cover the v0 scope:

1. `can_insert_moderation_case` (Phase 1, task 12)
2. `governance_log_hash_chain_holds` (Phase 1, task 13)
3. `list_open_cases_returns_seeded_rows` + `jury_queue_view_returns_assignments` + `modlog_view_returns_published_entries` (Phase 2, task 30)
4. **`report_to_modlog_golden_path`** (Phase 4, task 48) — **the v0 acceptance test**
5. `redaction_strips_identifiers` (cross-cutting §4.2)
6. `sponsor_liability_with_founder_multiplier` (Phase 5b, task 60)
7. `ineligible_user_cannot_be_picked_for_jury` (Phase 5c, task 69)
8. `all_mvp_endpoints_return_non_404` (Phase 5c, task 68)
9. `sanction_notice_round_trip` (Phase 6, task 77)

That's it for v0. Unit tests get added when something **breaks twice** and a unit test would have caught it.

### 5.3 CI

GitHub Actions workflow (one job, ~5 minutes target wall-clock):

```yaml
on: [push, pull_request]
jobs:
  ci:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env: { POSTGRES_PASSWORD: dev }
        ports: ["5432:5432"]
    steps:
      - checkout
      - rust toolchain (pinned)
      - cache cargo registry + target/
      - cargo check --workspace
      - cargo clippy --workspace -- -D warnings
      - cargo test --test e2e
```

**Don't add lints, formatters, or coverage tools beyond clippy in v0.** They get added when a regression slips through that they would have caught.

### 5.4 What's not tested

- **No frontend testing** — frontend isn't in scope for v0 ([05 §10](05-mvp-and-delivery-plan.md), the canonical prompt's "no UX or frontend work").
- **No load testing** — v2 work per [06 §2.6](06-security-and-threat-model.md).
- **No abuse-case red-team pass** — v2 work per [06 §2.10](06-security-and-threat-model.md).
- **No cross-instance federation HTTP transport test** — task 77 calls the inbox function directly. Real HTTP federation testing is v1 staging work per [07 §1.3](07-operations-and-federation.md).

---

## 6. Monday-morning checklist

The first 5 concrete tasks. Distilled from [05 §5](05-mvp-and-delivery-plan.md). **Hand this to yourself on day one.**

1. **Fork Lemmy 1.0-beta** and clone (depends on §2.1 decision: separate repo Option A). Verify `cargo check --workspace` passes against the unmodified upstream. **File path created:** the new repo's working directory.

2. **Write Migration 0 (`add_governance_enums`)** — `crates/db_schema/migrations/{ts}_add_governance_enums/{up,down}.sql`. Creates the Postgres enum types matching [04 §2](04-data-model-and-api.md). **`case_status` MUST include `EmergencyRemove`** ([99 ADR-013](99-decisions-and-open-questions.md)). Run `diesel migration run` and verify the enums exist via `psql`.

3. **Write Migration 1 (`add_governance_core`)** — `crates/db_schema/migrations/{ts}_add_governance_core/{up,down}.sql`. Creates `moderation_case`, `case_evidence`, `sanction`, `appeal`, `public_case_log` per [04 §1 Migration 1](04-data-model-and-api.md). Indexes from [04 §1.5](04-data-model-and-api.md). Run `diesel migration run` and verify with `\d moderation_case` in `psql`.

4. **Write Diesel models for the five core tables** — files under `crates/db_schema/src/source/governance/{moderation_case,case_evidence,sanction,appeal,public_case_log}.rs`. Per [04 §3](04-data-model-and-api.md) — start with `ModerationCase` and `ModerationCaseInsertForm` exactly as written. Update `crates/db_schema/src/lib.rs` to export the new module. Run `cargo check -p db_schema`.

5. **Write the first integration test** — `tests/e2e.rs::can_insert_moderation_case` (Phase 1, task 12). This is the first commit that proves the schema, the model, and the test harness are all working together. Run `cargo test --test e2e can_insert_moderation_case` and watch it pass.

If those five tasks land in week 1, the schema foundation is solid and Phase 1 is on track.

---

## 6.1. Real-time transport (if proposed post-v0)

If a future phase proposes adding a real-time push channel for governance notifications (jury invitations, case status changes, etc.), default to **Server-Sent Events (SSE) over WebSocket**. SSE composes with HTTP caching, has simpler backpressure semantics, and works through the same middleware stack as the existing API. The V2 messaging bridge does not require Brehon's own RT transport — it polls `governance_log` or subscribes via Postgres NOTIFY per [`SUBSCRIPTIONS.md`](SUBSCRIPTIONS.md). See [`V2/messaging.md §8.3`](V2/messaging.md) for the full reasoning.

---

## 7. Risks and unknowns

What could go wrong, what needs verification before v1.

### 7.1 Top risks

- **Lemmy 1.0-beta API stability** — by definition a beta. Schema changes, field renames, internal API churn during the v0 build window. **Mitigation:** pin to a specific 1.0-beta commit hash in `Cargo.toml`; rebase from upstream weekly during v0; budget 1 day per upstream rebase.
- **Extism plugin hook surface may not match the design assumption** — [99 ADR-012](99-decisions-and-open-questions.md) says we'll use it "where it simplifies governance hooks." If the `before_*` / `after_*` hooks for governance events don't actually exist in 1.0-beta or are flagged unstable, we fall back to direct Rust code (which is already the v0 plan — no impact). **Verification:** §2.5 Pre-flight task.
- **`actor_pseudonym` + redaction service correctness is high-stakes** — [99 ADR-015](99-decisions-and-open-questions.md) is GDPR-compliance-critical. A leak of a username into the governance log defeats the right-to-delete strategy permanently. **Mitigation:** the cross-cutting wrappers (§4.1, §4.2) are the only way to write to the log; the test in §4.2 enforces it. Also: review the redaction regex with a focused test pass after Phase 4 lands and before Phase 5 starts.
- **Hash-chain trigger + signature update interaction** is novel — the §4.1 Option A scheme (insert fires trigger, then update to add signature, allowed only on `signature IS NULL`) is fiddly. **Mitigation:** the `governance_log_hash_chain_holds` test (task 13) and an additional test that asserts `UPDATE governance_log SET signature = X WHERE signature IS NOT NULL` raises will catch regressions. If the scheme proves brittle, fall back to "store the signature in a side table keyed by `entry_hash`" — same security property, less trigger weirdness.
- **Threshold formula is a placeholder** ([99 OQ-006](99-decisions-and-open-questions.md)) — the v0 default of "weighted sum > 3.0" with a `reputation = 1.0` stub means the threshold is effectively "3 reports from logged-in users." That's fine for the test in task 48 but is **not production-ready**. Resolving OQ-006 is a v1 deliverable, not v0. Document the placeholder in code.

### 7.2 Other risks

- **Solo dev burnout** — 8–12 weeks of focused work on a single project is a lot. Take weeks off without guilt; this is not a sprint that has to end on a specific date.
- **Frontend work creep** — there's no frontend in v0, but the temptation is real. Resist. Use `curl` or a REST client for manual testing. The canonical prompt is explicit: "Do NOT produce UX or frontend work."
- **Lemmy's existing test infrastructure may not match this plan's `tests/e2e.rs` shape** — verify in Pre-flight whether Lemmy 1.0-beta has its own e2e harness convention. If so, conform to it rather than fighting it. Update §5.1 accordingly.
- **Diesel migration ordering is fragile** — Migrations 0 (enums) and 1 (tables that reference enums) must have correct timestamp ordering. Use `diesel migration generate` to get the timestamp right; do not manually craft directory names.
- **The `governance_log` table will grow unbounded** — that's by design, but archival/snapshotting is not in v0. By v1 the log may have millions of rows. v1 should add a `governance_log_archive` table and a periodic move job. Note for v1 backlog.

---

## 8. Open questions to escalate

Only OQs that **genuinely block Phase 1 (Step 1)**. Other OQs from [99](99-decisions-and-open-questions.md) are tracked there and will be addressed in their own time.

### Blocking Phase 1

- **None.** No OQ in [99](99-decisions-and-open-questions.md) blocks the schema migrations in Phase 1.

### Resolved with placeholders, revisit before v1

- **[99 OQ-006](99-decisions-and-open-questions.md) — Case threshold formula.** ✅ **Decided 2026-04-14:** accept the OQ "Current lean" as a hardcoded placeholder for v0; defer tuning to v1. Phase 4 task 39 uses `base_weight = 1`, `reporter_reputation = 1.0` (stub), `recency_factor = 1.0` (stub), threshold `weighted sum > 3.0`. The Phase 4 e2e test in task 48 **bypasses** the threshold by using `admin_assign_jury` to force the case into the jury phase (task 255's Option B), leaving the threshold formula untested in v0. This is acceptable because the threshold is not on the v0 acceptance-test path — jury assignment is the path that matters. Code must include a `TODO(brehon-v1): revisit per OQ-006` comment at the hardcoded constants in `create_report.rs`. Open a GitHub issue tagged `v1` titled "Resolve OQ-006 threshold formula" when Phase 4 lands.

### Blocking Phase 4 specifically

- **[99 OQ-007](99-decisions-and-open-questions.md) — Community creation in a fork that disables moderators.** ✅ **Resolved 2026-04-16:** Instance admins create communities in v0. Lemmy's existing `create_community` API is unchanged in the fork. The Phase 4 e2e test (task 48) seeds communities via admin user. Community governance (founder facilitators, initial rule sets, instance-wide jury pool fallback) deferred to v1 per ADR-010. No code change needed in v0.
- **[99 OQ-008](99-decisions-and-open-questions.md) — Direct moderator action on a case under jury review.** Per the OQ, target is "during Step 1 of the delivery plan." **The compatibility layer ([99 ADR-009](99-decisions-and-open-questions.md)) means this can happen.** For v0 the resolution can be: "if a moderator acts directly, the case auto-transitions to a new `admin-review` state, jury voting is paused, and an admin must explicitly resume." **But the new `admin-review` state isn't in the [04 §2 CaseStatus enum](04-data-model-and-api.md).** Two options: (a) add a new enum variant to `CaseStatus` (`AdminReview`) in Phase 1; or (b) reuse `EmergencyRemove` semantically (bad — they're different concerns). **Recommend (a)** — add `CaseStatus::AdminReview` in task 8 alongside `EmergencyRemove`. Confirm with user.

### Blocking Phase 5

- **[99 OQ-004](99-decisions-and-open-questions.md) — Maximum concurrent jury assignments per user.** ✅ **Resolved 2026-04-17:** cap at 3, instance-wide, seeded as `governance.jury.max_concurrent_assignments = 3` in task 50's config table. Tuneable at runtime via direct `UPDATE governance_config`.
- **[99 OQ-006](99-decisions-and-open-questions.md) — Case threshold formula.** ✅ **Resolved 2026-04-17:** multiplicative formula `base_weight × clamp(reporting_accuracy/100, clamp_min, clamp_max) × exp(-hours_old/recency_half_life_hours)` with `base_weight=1.0`, `clamp_min=0.1`, `clamp_max=2.0`, `recency_half_life_hours=168.0`, threshold `3_000_000` micros — all config-driven in task 58. `trusted_reporter` stays a visible signal only (no formula double-count).
- **[99 OQ-014](99-decisions-and-open-questions.md) — `can_sponsor` threshold value.** ✅ **Resolved 2026-04-17:** v0 uses a **config-driven gate strategy** (`config.onboarding.sponsor_gate_strategy` ∈ `'age' | 'open' | 'closed'`, default `'age'`) rather than a reputation gate. When strategy is `'age'`, `config.onboarding.sponsor_min_account_age_days = 30` is enforced; `'open'` bypasses all checks (recruitment-drive mode); `'closed'` rejects all endorsement attempts (emergency lockdown). The `can_sponsor` boolean on `reputation_snapshot` is computed (threshold = 25) but unused by any v0 strategy; v1 adds a `'reputation'` strategy that reads it, per OQ-020. Solves the cold-start bootstrap problem in fresh communities and gives operators a switch for event-driven recruitment without a code change.
- **[99 OQ-013](99-decisions-and-open-questions.md) — Endorsement-creation reputation deltas.** ✅ **Resolved 2026-04-17:** `+5` to sponsor's `endorsement_strength`, `+5` to sponsee's `participation_consistency` at endorsement creation. Config-driven (`governance.deltas.endorsement_created_sponsor|sponsee`), tuneable without code change.
- **[99 OQ-020](99-decisions-and-open-questions.md) NEW — Sponsor gate-strategy expansion.** v0 ships three strategies (`'age' | 'open' | 'closed'`) that cover the pilot-community scenarios identified 2026-04-17 (normal operation, event-driven recruitment drive, emergency lockdown). v1 adds `'age_or_surety'` (new members with ≥1 active surety bypass the age gate), `'reputation'` (reads `can_sponsor` from the snapshot column already populated in v0), and `'allowlist'` (explicit person-id allowlist, needs a new table). Each v1 strategy needs its own test matrix. Not blocking v0 — the `'open'` strategy covers the "let new members sponsor during an event" case until real data drives a more sophisticated gate.

### Not blocking v0 — explicitly deferred

- [99 OQ-001](99-decisions-and-open-questions.md) (per-community vs instance-wide reputation) — v0 uses per-community per the OQ "Current lean." The snapshot table already supports both via `community_id: Option<i32>`.
- [99 OQ-002](99-decisions-and-open-questions.md) (rule-set versioning) — v1 work.
- [99 OQ-003](99-decisions-and-open-questions.md) (restorative actions as distinct variants) — v1 work.
- [99 OQ-005](99-decisions-and-open-questions.md) (UX flows) — v1+v3 work; not in v0 scope.
- [99 OQ-009](99-decisions-and-open-questions.md) (juror anonymity in deliberation) — v1 (depends on UX in OQ-005).
- [99 OQ-010](99-decisions-and-open-questions.md) (production governance log signing keys) — v2 work.
- [99 OQ-011](99-decisions-and-open-questions.md) (first target community / persona) — v1 work.
- [99 OQ-012](99-decisions-and-open-questions.md) (project name) — defer until first public push.

---

## 9. Estimated effort

Solo-dev order-of-magnitude. **Days are working days, not calendar days. A week is 5 working days.** Pad each phase by ~25% for upstream rebases, blocked-by-OQ pauses, and reading time on Lemmy internals.

| Phase | Step in [05 §4](05-mvp-and-delivery-plan.md) | Tasks | Order-of-magnitude |
|---|---|---|---|
| Pre-flight | — | §2.1–§2.5 | **~3 days** (mostly the Pre-flight repo + Extism verification, fork setup) |
| Phase 1 | Step 1 — Schema + Diesel | 1–13 | **~2 weeks** (the trigger + hash chain are the trickiest) |
| Phase 2 | Step 2 — Read models | 14–30 | **~1 week** |
| Phase 3 | Step 3 — API common DTOs | 31–37 | **~3 days** (mostly mechanical) |
| Phase 4 | Step 4 — First five endpoints + golden test | 38–49 | **~2.5 weeks** (`submit_jury_vote` is the hardest single function in v0) |
| Phase 5a | Step 5 — Config + reputation infrastructure + endorsement | 50–55 | **~1 week** est. → **~1 day actual** (2026-04-17; 10 commits; two sessions bridged by handover file; split forced by context budget, not scope) |
| Phase 5b | Step 5 — Sponsor-liability + jury gating + founder bootstrap | 56–60 | **~1 week** est. → **~2 days actual** (2026-04-17–18; 13 branch commits + 4 cherry-picked post-merge Bucket fixes; split-plane bug in `apply_sponsor_liability` caught only at task 60 integration test) |
| Phase 5c | Step 5 — Remaining endpoints + observability + capability tests | 61–70 | **~1 week** est. → **~2 days actual** (2026-04-18–19; ~19 commits; included upstream rebase + 8-move risk-reduction pre-phase; task 70 = admin-config-write.sh + completion report) |
| Phase 6 | Step 6 — Federation outbound + advisory inbound | 71–79 | **~1.5 weeks** |
| Cross-cutting + v0 ship checklist polish | — | [05 §9](05-mvp-and-delivery-plan.md) checklist | **~1 week** |

**Total: ~10 weeks of focused solo-dev work.** Range: 8 weeks (smooth) to 14 weeks (with Lemmy-rebase pain or one major OQ requiring a redesign).

This is roughly the same envelope [99 ADR-010](99-decisions-and-open-questions.md) implies for v0 ("does the mechanic work?") and is meaningfully shorter than v1's "production-grade governance" — by design.

---

## 10. What this plan does not try to do

Restated from [05 §10](05-mvp-and-delivery-plan.md), the canonical prompt's hard constraints, and ADR-010 staging, so future-you doesn't have to re-derive the boundary:

- **No Rust code in this file.** This is the plan, not the implementation.
- **No v1/v2/v3 scope.** Anything tempting goes to "deferred to v1" (or v2 / v3 as appropriate per [99 ADR-010](99-decisions-and-open-questions.md)) and is not built.
- **No new tech choices** beyond the solo-dev stack: Postgres, Lemmy's existing Rust workspace, `diesel`, `sha2`, `ed25519-dalek`, `rs_merkle`, `webauthn-rs` (optional), Docker Compose, env-var secrets.
- **No Keycloak, no OPA, no OpenFGA, no Vault, no Kubernetes, no external log signer, no blockchain anchoring.** All v2/v3 per [99 ADR-010](99-decisions-and-open-questions.md).
- **No frontend, no UX work.** Period.
- **No revisiting any of the 15 ADRs.** If a contradiction emerges during implementation, it goes to §8 of this plan and is escalated to the user, not silently fixed in the body.

---

## 11. Cross-references

- The 11-endpoint MVP scope → [05 §2](05-mvp-and-delivery-plan.md)
- The 6-step implementation order → [05 §4](05-mvp-and-delivery-plan.md)
- Tables, structs, DTOs, routes → [04](04-data-model-and-api.md)
- Crate layout and plane separation → [03 §7](03-architecture.md)
- Hash chain, redaction, GDPR pseudonyms → [06 §2.3](06-security-and-threat-model.md), [06 §6.1](06-security-and-threat-model.md), [99 ADR-008](99-decisions-and-open-questions.md), [99 ADR-015](99-decisions-and-open-questions.md)
- Emergency-remove path → [99 ADR-013](99-decisions-and-open-questions.md), [02 §3.1](02-domain-model.md), [06 §2.2.1](06-security-and-threat-model.md)
- AGPLv3 obligations → [99 ADR-011](99-decisions-and-open-questions.md)
- Lemmy 1.0-beta + Extism → [99 ADR-012](99-decisions-and-open-questions.md)
- Federation interop with vanilla Lemmy → [99 ADR-014](99-decisions-and-open-questions.md), [03 §3.3](03-architecture.md)
- Solo-dev v0 staging → [99 ADR-010](99-decisions-and-open-questions.md), [05 §7](05-mvp-and-delivery-plan.md)
- Done-definition for v0 ship → [05 §9](05-mvp-and-delivery-plan.md)
