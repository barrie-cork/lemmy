# IMPLEMENTATION-PLAN-v0.md — Brehon-Law-Inspired Network MVP

**Status:** Draft v1 — Pre-flight complete, Phase 1 not started (2026-04-14)
**Audience:** Solo developer (you)
**Scope:** v0 only. v1/v2/v3 work referenced from [99 ADR-010](99-decisions-and-open-questions.md) and [05 §7](05-mvp-and-delivery-plan.md) is **out of scope**.

> **Progress snapshot (2026-04-14)** — see §1.1 for the full breakdown.
> - ✅ Pre-flight §2.1 (repo), §2.3 (dev env), §2.4 (fork setup tasks 1–6), AGPL notice, design-doc vendoring, PRP command suite, ralph loop, Windows cargo-check fix.
> - ❌ Pre-flight §2.4 task 7 (`tests/e2e.rs` placeholder), task 9 (CI workflow), §2.5 (Extism hook verification).
> - ❌ Phases 1–6: zero governance code. No `governance/` directories under `crates/db_schema/src/source/`, `crates/api/api_common/src/`, `crates/api/api/src/`, `crates/apub/**`, `crates/server/src/`. No new migrations. No governance tests.

This plan is the executable blueprint for v0. It is keyed to the 11-endpoint MVP scope in [05 §2](05-mvp-and-delivery-plan.md), the 6-step implementation order in [05 §4](05-mvp-and-delivery-plan.md), and the canonical agent prompt at [AGENT-PROMPT-mvp-implementation-plan.md](AGENT-PROMPT-mvp-implementation-plan.md).

If anything below contradicts an ADR in [99](99-decisions-and-open-questions.md), the ADR wins and the contradiction belongs in §8 of this plan, not in the body.

---

## 1. Executive summary

v0 ships the **vertical slice** of the Brehon governance mechanic on top of a fork of [Lemmy 1.0-beta](https://github.com/LemmyNet/lemmy) ([99 ADR-012](99-decisions-and-open-questions.md)). One member can report another's content; weighted reports open a moderation case; an admin backstop assigns a 5-juror panel; jurors vote; a simple-majority decision creates one sanction row; a public, redacted entry lands in the modlog; reputation events ripple to jurors, reporters, and (once Step 5 lands) sponsors. All 11 endpoints in [05 §2](05-mvp-and-delivery-plan.md) return 200 on the happy path; an integration test exercises the full report → decision → log → modlog flow against a real Postgres in Docker; the local hash chain over `public_case_log` and the governance log table is verifiable end-to-end. **Done is when [05 §9](05-mvp-and-delivery-plan.md) "Done-definition for v0 ship" is checked off** — feature-complete but not security-hardened (security hardening is v2 per [99 ADR-010](99-decisions-and-open-questions.md)).

Effort order-of-magnitude for a solo developer: **8–12 weeks** end-to-end, see §9.

---

## 1.1 Progress snapshot (2026-04-14)

Status ledger for this plan. Update when phases complete.

### Pre-flight (§2)

| Item | Status | Notes |
|---|---|---|
| §2.1 Repo decision | ✅ Option A chosen | Fork lives at `barrie-cork/lemmy`, working branch `governance-v0`. Local checkout at `C:\Users\barri\Developer\brehon-fork`. |
| §2.2 Project name | ⏸️ Deferred | Working title "brehon-fork" — [99 OQ-012](99-decisions-and-open-questions.md). Rename before first public push. |
| §2.3 Dev environment | ✅ Done | Rust 1.94 pinned (`rust-toolchain.toml`), workspace builds clean post-fork. Windows signal-handler fix landed in `ca3af443a`. |
| §2.4 task 1–3 Fork + branch + upstream build | ✅ Done | `main` tracks upstream, `governance-v0` is the working branch. |
| §2.4 task 4 `LICENSE` | ✅ Inherited | AGPLv3 from upstream. |
| §2.4 task 5 `AGPL-NOTICE.md` | ✅ Done | At repo root. |
| §2.4 task 6 Empty governance directory skeleton | ❌ Not started | **Blocks Phase 1.** No `governance/` subdirs exist yet. |
| §2.4 task 7 `tests/e2e.rs` placeholder | ❌ Not started | **Blocks Phase 1 task 12.** No `tests/` directory at workspace root. |
| §2.4 task 8 `migrations/` directory convention | ✅ Confirmed | Upstream Lemmy uses `migrations/{timestamp}_name/{up,down}.sql`. |
| §2.4 task 9 CI workflow | ❌ Not started | **Plan says "don't defer CI."** Upstream `.woodpecker.yml` exists but has no governance-specific step. No GitHub Actions workflow. |
| §2.5 Extism hook verification | ❌ Not started | Half-day human task. Not blocking Phase 1 (v0 uses direct Rust, not plugins). |

### Tooling and process (not in original plan but complete)

| Item | Status | Notes |
|---|---|---|
| Design docs vendored into fork | ✅ Done | `docs/brehon-law-inspired-network/` — canonical source per commit `e960a128c`. |
| PRP command suite | ✅ Installed | 13 commands in `.claude/commands/prp-core/`; 4 Tier 1 commands (`prp-plan`, `prp-implement`, `prp-prd`, `prp-review`) Brehon-customised with `<brehon-context>` preludes. |
| Ralph loop + stop hook | ✅ Wired | `.claude/prp-ralph.state.md` protocol + `.claude/hooks/prp-ralph-stop.sh` wired via `Stop` hook in `settings.json`. |
| Fork-local `CLAUDE.md` | ✅ Done | Pinned upstream SHA `811d0d09c`, branch model, command list. |

### Phases 1–6

| Phase | Step | Tasks | Status | Blocker |
|---|---|---|---|---|
| 1 | Schema + Diesel foundation | 13 (#1–13) | ❌ Not started | §2.4 task 6 (dir skeleton) + task 7 (`tests/e2e.rs`) |
| 2 | Read models | 17 (#14–30) | ❌ Not started | Phase 1 |
| 3 | API common DTOs | 7 (#31–37) | ❌ Not started | Phase 1 |
| 4 | First 5 endpoints + golden path | 12 (#38–49) | ❌ Not started | Phases 1–3 |
| 5 | Reputation + sponsorship + 11 endpoints | 14 (#50–63) | ❌ Not started | Phases 1–4 |
| 6 | Federation outbound + advisory inbound | 9 (#64–72) | ❌ Not started | Phases 1–5 |

### Next action

Per §6 (Monday-morning checklist): create `tests/e2e.rs` placeholder + empty governance-directory skeleton (§2.4 tasks 6–7), wire CI (§2.4 task 9), then run `/prp-plan "Phase 1 — Schema + Diesel foundation"` to produce the first plan file.

---

## 2. Pre-flight

These are one-time decisions and setup tasks that must happen before Step 1 of [05 §4](05-mvp-and-delivery-plan.md). Several need a human call.

### 2.1 Repo decision — ✅ RESOLVED (Option A)

The Brehon fork lives as a separate GitHub repo forked from `LemmyNet/lemmy`. Remote: `barrie-cork/lemmy`. Local checkout: `C:\Users\barri\Developer\brehon-fork`. `main` tracks upstream (weekly rebase); `governance-v0` is the working branch for all v0 feature work.

Rationale preserved for history: AGPLv3 plus the upstream-rebase burden ([99 ADR-012](99-decisions-and-open-questions.md)) made subtree more pain than gain. Design docs are **vendored into this fork** under `docs/brehon-law-inspired-network/` (commit `e960a128c`) so the fork is self-contained — a thin link-back from `homeserver` to this repo is the cross-repo seam, not the other way around.

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

1. ✅ Fork Lemmy 1.0-beta on GitHub (Option A above) — `barrie-cork/lemmy`
2. ✅ Clone, create branch `governance-v0`
3. ✅ Verify upstream `cargo check --workspace` passes — baseline clean (Windows signal-handler gate added in `ca3af443a`)
4. ✅ Add `LICENSE` if not already present (AGPLv3 — inherited from upstream)
5. ✅ Add an `AGPL-NOTICE.md` at repo root explaining the fork relationship and source-disclosure obligation per [99 ADR-011](99-decisions-and-open-questions.md)
6. ❌ Create the empty governance directory skeleton (no `.rs` files yet, just `mod.rs` placeholders) so the structure is in version control before any code lands:
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
7. ❌ Create `tests/e2e.rs` placeholder file (just the test harness boilerplate; tests come in Step 4)
8. ✅ `migrations/` directory exists at repo root — upstream Lemmy convention confirmed (`migrations/{timestamp}_name/{up,down}.sql`)
9. ❌ Wire CI early: GitHub Actions workflow that runs `cargo check --workspace` + `cargo clippy --workspace -- -D warnings` + `cargo test -p server --test e2e` on every push. **Don't defer CI — solo devs without CI ship rot.** (Upstream `.woodpecker.yml` exists but has no governance-specific step.)

### 2.5 Extism plugin system — confirm it exists — ❌ NOT STARTED

[99 ADR-012](99-decisions-and-open-questions.md) commits us to using Lemmy 1.0-beta's Extism-based plugin system "where it simplifies governance hooks." For v0, we will **not** ship governance hooks as Extism plugins — they are direct Rust code in the new crates. But before Step 1 starts, **read the Lemmy 1.0-beta Extism plugin docs and source** to confirm the `before_*` / `after_*` hook surface ([99 ADR-012](99-decisions-and-open-questions.md)) actually exists and is documented. If it does not, file an open question and decide in v1 whether to wire governance through plugins or stay with direct Rust.

This is a Step-0 verification, not a Step-1 task. Time-box to half a day. **Note:** Extism 1.20.0 + extism-convert 1.20.0 are confirmed in `Cargo.toml` (per fork-local `CLAUDE.md`), so the dependency exists — still need to verify the governance-hook surface area.

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

44. **`POST /api/v4/governance/admin/assign-jury` → `admin_assign_jury`** — handler in `crates/api/api/src/governance/admin_assign_jury.rs` per [04 §6.2](04-data-model-and-api.md). **Temporary admin-only backstop for v0** — picks 5 random eligible users (in Phase 4 "eligible" just means "not the target, not the reporter"; reputation gating arrives in Phase 5), inserts 5 `jury_assignment` rows with `status = Selected`, then immediately auto-promotes them to `Accepted` for testability. **Admin auth check: hardcoded `is_instance_admin` flag, no MFA in v0** ([05 §2](05-mvp-and-delivery-plan.md) admin backstops + [99 ADR-007](99-decisions-and-open-questions.md)).

45. **`POST /api/v4/governance/admin/close-case` → `admin_close_case`** — handler in `crates/api/api/src/governance/admin_close_case.rs` per [04 §6.2](04-data-model-and-api.md). Force-closes a case to `Closed` regardless of jury state. Required for emergency unblocking and for the `EmergencyRemove` post-facto path. Logs an audit entry. **Quorum + delay is a v2 item** ([99 ADR-010](99-decisions-and-open-questions.md)) — for v0 this is single-admin. Document in code that this is a known v0 simplification.

46. **`EmergencyRemove` wiring** — `admin_assign_jury` and `admin_close_case` must understand `CaseStatus::EmergencyRemove`. Create a thin helper `emergency_remove_open_case(target, admin_id, reason)` in `crates/api/api/src/governance/admin_emergency_remove.rs` that:
    - Removes the content (calls into Lemmy's existing remove pathway)
    - Inserts a `moderation_case` with `status = EmergencyRemove` and the removal as the initial fact
    - Triggers `admin_assign_jury` for post-facto review
    - Emits an extra-visible governance log entry per [06 §2.2.1](06-security-and-threat-model.md)
    No HTTP route is required for v0 — the function is callable from a future emergency-remove route or from a direct admin tool. It exists so the cross-cutting `EmergencyRemove` requirement (§4.3) is wired through. Per [99 ADR-013](99-decisions-and-open-questions.md): the jury **cannot un-remove** the content; document in the helper's doc comment.

47. **Server wiring** — `crates/server/src/governance.rs` per [04 §12](04-data-model-and-api.md) and [03 §7.3](03-architecture.md). **Composition root only** — registers the routes from task 38, schedules the (initially empty) background jobs (snapshot, sanction cleanup, jury timeout — these are stubs in Phase 4 and become real in Phase 5/6), and that's it. **No business logic here** ([03 §11](03-architecture.md), [04 §13.2](04-data-model-and-api.md)).

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

### Phase 5 — Reputation & sponsorship (Step 5 of [05 §4](05-mvp-and-delivery-plan.md))

**Goal:** sponsor liability, endorsement creation, reputation snapshot calculation, and `GET /reputation/me` are wired. A sanctioned user's sponsors visibly lose `endorsement_strength`. A user with zero `jury_reliability` cannot be selected by the jury picker.

**Tasks:**

50. **`crates/db_views/reputation` crate** — per [04 §4.3](04-data-model-and-api.md). View structs: `ReputationSummaryView`, `EndorsementSummaryView`. Queries: `read_reputation_summary`, `list_endorsements_for_person`, `list_sureties_for_person`. Files under `crates/db_views/reputation/`.

51. **Reputation snapshot calculator** — `crates/api/api/src/governance/reputation_snapshot.rs`. Pure function: takes a `person_id` and (optional) `community_id`, reads all `reputation_event` rows scoped to that person/community, sums deltas per `ReputationDimension`, applies the **Phase-5 stub decay** (positive events older than 90 days are halved — much simpler than the per-dimension tuned decay that's a v1 item per [05 §3](05-mvp-and-delivery-plan.md)), and writes/updates a `reputation_snapshot` row. Sets the boolean capability flags from the four counters using v0 thresholds: `jury_eligible = jury_reliability >= 50 AND no active sanctions AND age >= 60d`; `trusted_reporter = reporting_accuracy >= 50`. **Only those four counters and two booleans** — see [04 §13.2 warning](04-data-model-and-api.md).

52. **Snapshot recalculation background job** — registered in `crates/server/src/governance.rs`. Per [07 §2 cadence](07-operations-and-federation.md): every 15 min OR on event insertion. For v0, every 15 min is fine (skip the on-insertion path). Iterates all persons with new events since the last run, recomputes their snapshot.

53. **`POST /api/v4/governance/endorsement` → `create_endorsement`** — handler in `crates/api/api_crud/src/governance/create_endorsement.rs` per [04 §6.1](04-data-model-and-api.md). Validates `can_sponsor` capability (read from `reputation_snapshot.endorsement_strength` threshold), enforces max-5-active and 48h cooldown ([01 §5.1](01-vision-and-principles.md)), inserts `endorsement` row, **also inserts a `surety` row if this is the sponsee's 1st or 2nd endorsement and they're still in the Provisional state** (sponsorship is the path from Provisional → Member per [02 §2](02-domain-model.md)), enqueues snapshot recalculation for both parties, **emits governance log entry**.

54. **`GET /api/v4/governance/reputation/me` → `get_my_reputation`** — handler in `crates/api/api/src/governance/get_my_reputation.rs` per [04 §6.2](04-data-model-and-api.md). Returns `ReputationSummaryView` for the calling user, scoped to community if `community_id` is in the query. Wraps the view from task 50.

55. **Sponsor-liability flow in `submit_jury_vote`** — extend the handler from task 42 so that on `Decided` with a non-trivial sanction, the sanctioned user's active sponsors (rows in `surety` where `sponsored_id = target_person_id AND revoked_at IS NULL`) each get a `reputation_event` with `dimension = EndorsementStrength` and a negative delta proportional to severity ([01 §5.2](01-vision-and-principles.md): minor −1%, moderate −5%, severe −10% to −20%; **for v0 hardcode minor = -10, moderate = -50, severe = -200** as integer deltas; v1 tunes the units). Splits the impact across all active sponsors (delta / sponsor_count, rounded). Emits governance log entries.

56. **Wire reputation gating into `admin_assign_jury`** — extend the handler from task 44 so that "eligible" now means `jury_eligible = true` AND not the target AND not a reporter on this case. Falls back to the unfiltered pool if fewer than 5 eligible candidates exist (with a warning log) — keeps integration tests viable.

57. **Sponsor-liability integration test** — `tests/e2e.rs::sponsor_liability_propagates`. Per [05 §9](05-mvp-and-delivery-plan.md) "second integration test exists for the sponsor-liability flow". Steps: create user A; create users B and C (sponsors with `can_sponsor = true`); B and C each `POST /endorsement` for A; A becomes Member; A is reported, case opens, jury decides Sanction; assert B and C both have new `reputation_event` rows with negative `endorsement_strength` deltas; assert their snapshots reflect the loss.

58. **`POST /api/v4/governance/jury/accept` → `accept_jury_assignment`** — handler in `crates/api/api/src/governance/accept_jury_assignment.rs` per [04 §6.2](04-data-model-and-api.md). Conflict check: caller is not in the same `surety` cluster as the target (look up shared sponsors), not the reporter. For v0 the cluster check is just "shared sponsor" — the full diversity-constraint logic is v1 per [05 §3](05-mvp-and-delivery-plan.md). Updates `jury_assignment.status = Accepted`. Emits log entry.

59. **`POST /api/v4/governance/jury/decline` → `decline_jury_assignment`** — handler in `crates/api/api/src/governance/decline_jury_assignment.rs` per [04 §6.2](04-data-model-and-api.md). Updates status to `Declined`, logs reason, **triggers replacement juror selection** by re-invoking the jury-pick subroutine for one slot. Emits log entry.

60. **`POST /api/v4/governance/appeal` → `request_appeal`** — handler in `crates/api/api_crud/src/governance/request_appeal.rs` per [04 §6.1](04-data-model-and-api.md). Verifies appeal window is open (`now() < case.closed_at`), verifies caller is entitled (sanction target — for v0 just that, no original-reporter case), inserts `appeal` row with `status = Requested`, moves case status to `Appealed`. **Re-jury logic for the appeal is a Phase-5+ stub** — for v0 the appeal just sits in the `Appealed` state until an admin manually closes it via `admin_close_case`. The full "appeal heard by larger jury" flow is v1 per [05 §3](05-mvp-and-delivery-plan.md). Emits log entry.

61. **`GET /api/v4/governance/cases` → `list_cases`** — handler in `crates/api/api/src/governance/list_cases.rs` per [04 §6.2](04-data-model-and-api.md). Filters by community / status / assignee. Wraps `list_open_cases_for_community` from Phase 2 (extend the query if the filters are richer than the existing one).

62. **All 11 endpoints registered** — at end of Phase 5, the route tree in `crates/api/routes/src/governance.rs` matches [04 §7](04-data-model-and-api.md) for the 11 MVP endpoints in [05 §2](05-mvp-and-delivery-plan.md). The two admin backstops (`assign-jury`, `close-case`) are also registered but are not counted in the 11.

63. **Capability-gating integration test** — `tests/e2e.rs::ineligible_user_cannot_be_picked_for_jury`. Creates a user with all reputation counters set to zero, runs `admin_assign_jury` against a case, asserts that user is not in the resulting assignment list.

**Dependencies:** Phases 1–4. The reputation views depend on the snapshot table (Phase 1) and the snapshot calculator (this phase).

**Definition of done** (from [05 §4 Step 5](05-mvp-and-delivery-plan.md)): both integration tests in tasks 57 and 63 pass; all 11 MVP endpoints from [05 §2](05-mvp-and-delivery-plan.md) return 200 on the happy path.

---

### Phase 6 — Federation objects & activities, outbound-only (Step 6 of [05 §4](05-mvp-and-delivery-plan.md))

**Goal:** when instance A decides a case with `FederationQuarantineRecommendation` scope, instance B receives a `SanctionNotice` ActivityPub activity, verifies it, stores it in `remote_sanction_notice`, surfaces it to admin review, and **does not auto-apply** it ([99 ADR-006](99-decisions-and-open-questions.md), [05 §3](05-mvp-and-delivery-plan.md), [03 §3.3](03-architecture.md)).

**Tasks:**

64. **Migration `add_federation_attestations`** — per [04 §1 Migration 4](04-data-model-and-api.md). Tables: `federation_attestation`, `remote_sanction_notice`. Files: `crates/db_schema/migrations/{timestamp}_add_federation_attestations/{up,down}.sql`. **Was technically part of Phase 1 by ordering**, but the columns and logic land here so the migration is moved to this phase.

65. **Diesel models for federation tables** — `FederationAttestation`, `RemoteSanctionNotice`. Per [04 §3](04-data-model-and-api.md). Files: `crates/db_schema/src/source/governance/{federation_attestation,remote_sanction_notice}.rs`.

66. **AP object types** — `crates/apub/objects/src/governance/{moderation_label.rs,trust_attestation.rs,sanction_notice.rs}` per [04 §9](04-data-model-and-api.md). `ModerationLabelObject`, `TrustAttestationObject`, `SanctionNoticeObject`. Use Lemmy's existing AP serialisation conventions (`activitypub_federation` crate or whatever 1.0-beta uses). **`ap_id` not `actor_id`** per [99 ADR-012](99-decisions-and-open-questions.md).

67. **AP activity types** — `crates/apub/activities/src/governance/{publish_label.rs,publish_attestation.rs,publish_sanction_notice.rs}` per [04 §10](04-data-model-and-api.md). Wraps the objects from task 66 in `Create` activities. `Undo` variants exist as stubs but aren't wired in v0 unless trivially cheap.

68. **Outbound publisher** — `crates/apub/apub/src/governance/outbox.rs` per [04 §11](04-data-model-and-api.md). Function `send_local_sanction_notice(case_id)`: builds a `SanctionNoticeObject` from the case + sanction rows, signs the activity (Lemmy's existing key-signing path), enqueues it on the federated outbox for delivery to all peers in `Allow` or `Limit` state. Function `send_local_trust_attestation(person_id, attestation_type)` similar but for attestations. **Scope restricted to peers in the federation_attestation `Allow` / `Limit` set; no peer table for v0, so all federated instances are treated as `Allow`** — document this as a v0 simplification.

69. **Inbound receiver** — `crates/apub/apub/src/governance/inbox.rs` per [04 §11](04-data-model-and-api.md). Function `receive_remote_sanction_notice(activity)`: signature verify (reject on failure, log it); schema validate; insert `remote_sanction_notice` row with `local_case_id = NULL`; **no auto-apply** ([99 ADR-006](99-decisions-and-open-questions.md), [06 §5](06-security-and-threat-model.md)). Function `receive_remote_trust_attestation` similar.

70. **Wire `submit_jury_vote` to publish on Critical sanctions** — extend the handler from task 42 (and from task 55 in Phase 5) so that if the winning decision creates a sanction with `scope = FederatedRecommendation`, `send_local_sanction_notice(case_id)` is called as part of the post-decision flow. Single-admin v0 (no quorum + delay — that's v2 per [99 ADR-010](99-decisions-and-open-questions.md)). Emits a governance log entry for the federation send.

71. **Federation integration test** — `tests/e2e.rs::sanction_notice_round_trip`. Spins up two API server processes against two Postgres databases (different ports). Decides a case on instance A with `scope = FederatedRecommendation`. Asserts instance A's outbox queue has a `SanctionNoticeObject`. Manually delivers the activity to instance B's `receive_remote_sanction_notice`. Asserts instance B has a `remote_sanction_notice` row with the correct fields and `local_case_id IS NULL`. **Do not require a real HTTP federation transport** for this test — call the inbox function directly with a serialised object. The HTTP transport is Lemmy's existing code; we trust it.

72. **Verify `verify.rs`** — `crates/apub/apub/src/governance/verify.rs` per [04 §11](04-data-model-and-api.md). Signature verification helper used by `inbox.rs`. Likely just delegates to Lemmy's existing AP signature verifier; the file exists so that future custom verification logic has a home.

**Dependencies:** Phases 1–5. The federation tables depend on the migrations from task 64.

**Definition of done** (from [05 §4 Step 6](05-mvp-and-delivery-plan.md)): the round-trip test in task 71 passes — sanction on instance A produces a stored advisory record on instance B, never auto-applied.

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
- **Verification:** a daily background job (registered in `crates/server/src/governance.rs`, task 47 + extension in Phase 5) walks the log, recomputes the chain, and verifies signatures. Divergence triggers an error log. **Alerting hookup is a v2 item** per [07 §7.1](07-operations-and-federation.md).
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

Eight tests cover the v0 scope:

1. `can_insert_moderation_case` (Phase 1, task 12)
2. `governance_log_hash_chain_holds` (Phase 1, task 13)
3. `list_open_cases_returns_seeded_rows` + `jury_queue_view_returns_assignments` + `modlog_view_returns_published_entries` (Phase 2, task 30)
4. **`report_to_modlog_golden_path`** (Phase 4, task 48) — **the v0 acceptance test**
5. `redaction_strips_identifiers` (cross-cutting §4.2)
6. `sponsor_liability_propagates` (Phase 5, task 57)
7. `ineligible_user_cannot_be_picked_for_jury` (Phase 5, task 63)
8. `sanction_notice_round_trip` (Phase 6, task 71)

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
- **No cross-instance federation HTTP transport test** — task 71 calls the inbox function directly. Real HTTP federation testing is v1 staging work per [07 §1.3](07-operations-and-federation.md).

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

### Resolved with placeholders, must revisit before Phase 4 ships

- **[99 OQ-006](99-decisions-and-open-questions.md) — Case threshold formula.** Phase 4 task 39 uses the OQ "Current lean" as a hardcoded constant. **Must be revisited by the user** before Phase 4 ships, because the formula determines whether the e2e test in task 48 actually exercises the threshold logic. Option used in this plan: skip the threshold by using `admin_assign_jury` to force the case into the jury phase, leaving the threshold formula stubbed at the OQ default. **Decision needed:** is that acceptable for the v0 acceptance test, or should the formula be tuned now?

### Blocking Phase 4 specifically

- **[99 OQ-007](99-decisions-and-open-questions.md) — Community creation in a fork that disables moderators.** The Phase 4 e2e test (task 48) seeds a community. The OQ "Current lean" says instance admins create communities in MVP. That's the path the test will take. **Confirm with user** before writing the test fixture so the test doesn't get rewritten when the OQ resolves.
- **[99 OQ-008](99-decisions-and-open-questions.md) — Direct moderator action on a case under jury review.** Per the OQ, target is "during Step 1 of the delivery plan." **The compatibility layer ([99 ADR-009](99-decisions-and-open-questions.md)) means this can happen.** For v0 the resolution can be: "if a moderator acts directly, the case auto-transitions to a new `admin-review` state, jury voting is paused, and an admin must explicitly resume." **But the new `admin-review` state isn't in the [04 §2 CaseStatus enum](04-data-model-and-api.md).** Two options: (a) add a new enum variant to `CaseStatus` (`AdminReview`) in Phase 1; or (b) reuse `EmergencyRemove` semantically (bad — they're different concerns). **Recommend (a)** — add `CaseStatus::AdminReview` in task 8 alongside `EmergencyRemove`. Confirm with user.

### Blocking Phase 5

- **[99 OQ-004](99-decisions-and-open-questions.md) — Maximum concurrent jury assignments per user.** Per the OQ, target is "before Step 4 (Jury endpoints)." The OQ "Current lean" is "cap at 3 concurrent assignments, instance-wide." For v0 we can implement this with a hardcoded constant in `admin_assign_jury` (Phase 4 task 44 + Phase 5 task 56). **Confirm "cap at 3" with user** before Phase 5 starts; if a different cap is preferred, change the constant. Doesn't block Phase 4 — task 44's first version doesn't enforce the cap.

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
| Phase 5 | Step 5 — Reputation & sponsorship + remaining endpoints | 50–63 | **~2 weeks** |
| Phase 6 | Step 6 — Federation outbound + advisory inbound | 64–72 | **~1.5 weeks** |
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
